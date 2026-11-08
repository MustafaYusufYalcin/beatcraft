//! Sequencer + mixer render engine and the audio output thread.

use crate::model::*;
use crate::synth::{InsertFx, Voice};
use std::sync::{Arc, Mutex};

pub enum Cmd {
    NoteOn { ch: usize, key: u8, vel: f32 },
    NoteOff { ch: usize, key: u8 },
    Preview { ch: usize, key: u8 },
    Panic,
}

pub struct Shared {
    pub project: Project,
    pub playing: bool,
    pub song_mode: bool,
    pub current_pattern: usize,
    pub pos_ticks: f64,
    pub cmds: Vec<Cmd>,
    pub meters: Vec<[f32; 2]>,
    pub scope: Vec<f32>,
    pub scope_pos: usize,
    pub load: f32,
    pub sample_rate: f32,
    pub voices: usize,
    pub audio_ok: bool,
    pub device_name: String,
}

impl Shared {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            playing: false,
            song_mode: false,
            current_pattern: 0,
            pos_ticks: 0.0,
            cmds: Vec::new(),
            meters: vec![[0.0; 2]; MIXER_TRACKS],
            scope: vec![0.0; 1024],
            scope_pos: 0,
            load: 0.0,
            sample_rate: SAMPLE_RATE as f32,
            voices: 0,
            audio_ok: false,
            device_name: String::new(),
        }
    }

    pub fn loop_len(&self) -> u32 {
        if self.song_mode {
            self.project.song_length_ticks()
        } else {
            self.project
                .patterns
                .get(self.current_pattern)
                .map(|p| p.length_ticks())
                .unwrap_or(STEPS_PER_BAR * STEP_TICKS)
        }
    }
}

const CHUNK: usize = 64;

pub struct Engine {
    sr: f32,
    voices: Vec<Voice>,
    fx: Vec<Option<InsertFx>>,
    bufs: Vec<[Vec<f32>; 2]>,
    seed: u32,
}

struct Trig {
    ch: usize,
    key: u8,
    vel: f32,
    gate_ticks: u32,
}

impl Engine {
    pub fn new(sr: f32) -> Self {
        Self {
            sr,
            voices: Vec::with_capacity(256),
            fx: (0..MIXER_TRACKS).map(|_| None).collect(),
            bufs: (0..MIXER_TRACKS).map(|_| [vec![0.0; CHUNK], vec![0.0; CHUNK]]).collect(),
            seed: 12345,
        }
    }

    fn spawn(&mut self, s: &Shared, ch: usize, key: u8, vel: f32, gate_sec: f32, live: bool) {
        let Some(c) = s.project.channels.get(ch) else { return };
        if c.kind.is_drum() {
            // one voice per drum channel: choke the previous hit (like a mono sampler)
            if c.kind == InstKind::HiHat {
                // closed hat chokes the open hat
                for v in self.voices.iter_mut() {
                    let open = s.project.channels.get(v.channel).map(|oc| oc.kind == InstKind::OpenHat).unwrap_or(false);
                    if v.channel == ch || open {
                        v.done = true;
                    }
                }
            } else {
                for v in self.voices.iter_mut().filter(|v| v.channel == ch) {
                    v.done = true;
                }
            }
        }
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let key_for = if c.kind.is_drum() { ROOT_KEY } else { key };
        let v = Voice::new(ch, c.kind, c.params, key_for, c.pitch + s.project.master_pitch, vel, gate_sec, live, self.seed);
        if self.voices.len() > 160 {
            self.voices.remove(0);
        }
        self.voices.push(v);
    }

    fn collect(s: &Shared, a: f64, b: f64, out: &mut Vec<Trig>) {
        let p = &s.project;
        let swing_ticks = snap_swing(s.project.swing) * SWING_MAX_DELAY * STEP_TICKS as f32;
        let pat_events = |pat: &Pattern, lo: f64, hi: f64, limit: f64, out: &mut Vec<Trig>| {
            for (ci, d) in pat.data.iter().enumerate() {
                if ci >= p.channels.len() || p.channels[ci].mute {
                    continue;
                }
                for (si, vel) in d.steps.iter().enumerate() {
                    if *vel <= 0.0 {
                        continue;
                    }
                    let mut t = (si as u32 * STEP_TICKS) as f64;
                    if si % 2 == 1 {
                        t += swing_ticks as f64;
                    }
                    if t >= lo && t < hi && t < limit {
                        out.push(Trig { ch: ci, key: ROOT_KEY, vel: *vel, gate_ticks: STEP_TICKS });
                    }
                }
                for n in &d.notes {
                    let mut t = n.start as f64;
                    if n.start % STEP_TICKS == 0 && (n.start / STEP_TICKS) % 2 == 1 {
                        t += swing_ticks as f64;
                    }
                    if t >= lo && t < hi && t < limit {
                        out.push(Trig { ch: ci, key: n.key, vel: n.vel, gate_ticks: n.len });
                    }
                }
            }
        };
        if s.song_mode {
            for c in &p.clips {
                let (cs, ce) = (c.start as f64, (c.start + c.len) as f64);
                if ce <= a || cs >= b {
                    continue;
                }
                if let Some(pat) = p.patterns.get(c.pattern) {
                    // clip loops its pattern if longer than the pattern
                    let plen = pat.length_ticks() as f64;
                    let mut off = 0.0;
                    while off < c.len as f64 {
                        let lo = a - cs - off;
                        let hi = b - cs - off;
                        if hi > 0.0 && lo < plen {
                            pat_events(pat, lo, hi, (c.len as f64 - off).min(plen), out);
                        }
                        off += plen;
                    }
                }
            }
        } else if let Some(pat) = p.patterns.get(s.current_pattern) {
            pat_events(pat, a, b, f64::MAX, out);
        }
    }

    /// Render interleaved stereo (or n-channel) output.
    pub fn render(&mut self, s: &mut Shared, out: &mut [f32], out_ch: usize) {
        let t_start = web_time::Instant::now();
        let frames = out.len() / out_ch;
        // commands
        for cmd in std::mem::take(&mut s.cmds) {
            match cmd {
                Cmd::NoteOn { ch, key, vel } => self.spawn(s, ch, key, vel, f32::MAX, true),
                Cmd::NoteOff { ch, key } => {
                    for v in self.voices.iter_mut().filter(|v| v.live && v.channel == ch && v.key == key) {
                        v.release();
                    }
                }
                Cmd::Preview { ch, key } => self.spawn(s, ch, key, 0.8, 0.25, false),
                Cmd::Panic => self.voices.clear(),
            }
        }
        let mut trigs = Vec::new();
        let mut done = 0;
        while done < frames {
            let n = CHUNK.min(frames - done);
            // sequencer
            if s.playing {
                let tps = ticks_per_second(s.project.tempo);
                let adv = n as f64 / samples_per_tick(self.sr as f64, s.project.tempo);
                let len = s.loop_len() as f64;
                if s.pos_ticks >= len {
                    s.pos_ticks %= len;
                }
                let a = s.pos_ticks;
                let b = a + adv;
                trigs.clear();
                if b <= len {
                    Self::collect(s, a, b, &mut trigs);
                } else {
                    Self::collect(s, a, len, &mut trigs);
                    Self::collect(s, 0.0, b - len, &mut trigs);
                }
                for t in trigs.drain(..) {
                    let gate = t.gate_ticks as f64 / tps;
                    self.spawn(s, t.ch, t.key, t.vel, gate as f32, false);
                }
                s.pos_ticks = if b >= len { b - len } else { b };
            }
            // clear buffers
            for b in self.bufs.iter_mut() {
                b[0][..n].fill(0.0);
                b[1][..n].fill(0.0);
            }
            let mut active = vec![false; MIXER_TRACKS];
            // voices
            for v in self.voices.iter_mut() {
                let Some(c) = s.project.channels.get(v.channel) else {
                    v.done = true;
                    continue;
                };
                let g = channel_gain(c.volume) * 0.8; // headroom so a few channels at default don't clip
                let (pl, pr) = pan_gains(c.pan);
                let ins = c.insert.min(MIXER_TRACKS - 1);
                active[ins] = true;
                let [bl, br] = &mut self.bufs[ins];
                for i in 0..n {
                    let x = v.sample(self.sr) * g;
                    bl[i] += x * pl;
                    br[i] += x * pr;
                    if v.done {
                        break;
                    }
                }
            }
            self.voices.retain(|v| !v.done);
            // inserts
            let any_solo = s.project.mixer.iter().skip(1).any(|m| m.solo);
            let delay_samples = (60.0 / s.project.tempo * 0.75 * self.sr) as usize;
            let (master, rest) = self.bufs.split_at_mut(1);
            let [ml, mr] = &mut master[0];
            for (k, buf) in rest.iter_mut().enumerate() {
                let idx = k + 1;
                let ins = &s.project.mixer[idx];
                let (lp, dl, rv) = effective_fx(ins);
                let wants_fx = dl > 0.001 || rv > 0.001 || lp < 0.999;
                if !active[idx] && !(wants_fx && self.fx[idx].is_some()) {
                    s.meters[idx][0] *= 0.9;
                    s.meters[idx][1] *= 0.9;
                    continue;
                }
                let [bl, br] = buf;
                if wants_fx {
                    let fx = self.fx[idx].get_or_insert_with(|| InsertFx::new(self.sr));
                    fx.process(&mut bl[..n], &mut br[..n], lp, dl, rv, delay_samples, self.sr);
                }
                let g = fader_gain(ins.volume);
                let (pl, pr) = pan_gains(ins.pan);
                let (pl, pr) = (pl * std::f32::consts::SQRT_2, pr * std::f32::consts::SQRT_2);
                let silenced = ins.mute || (any_solo && !ins.solo);
                let mut pk = [0.0f32; 2];
                for i in 0..n {
                    let l = bl[i] * g * pl;
                    let r = br[i] * g * pr;
                    pk[0] = pk[0].max(l.abs());
                    pk[1] = pk[1].max(r.abs());
                    if !silenced {
                        ml[i] += l;
                        mr[i] += r;
                    }
                }
                let m = &mut s.meters[idx];
                m[0] = (m[0] * 0.92).max(if silenced { 0.0 } else { pk[0] });
                m[1] = (m[1] * 0.92).max(if silenced { 0.0 } else { pk[1] });
            }
            // master
            let mi = &s.project.mixer[0];
            {
                let fx = self.fx[0].get_or_insert_with(|| InsertFx::new(self.sr));
                let (lp, dl, rv) = effective_fx(mi);
                fx.process(&mut ml[..n], &mut mr[..n], lp, dl, rv, delay_samples, self.sr);
            }
            let g = fader_gain(mi.volume) * fader_gain(s.project.master_volume);
            let (pl, pr) = pan_gains(mi.pan);
            let (pl, pr) = (pl * std::f32::consts::SQRT_2, pr * std::f32::consts::SQRT_2);
            let limiter = mi.slots.iter().any(|x| *x == Some(FxKind::Limiter));
            let mut pk = [0.0f32; 2];
            for i in 0..n {
                let mut l = ml[i] * g * pl;
                let mut r = mr[i] * g * pr;
                if mi.mute {
                    l = 0.0;
                    r = 0.0;
                }
                if limiter {
                    l = l.tanh();
                    r = r.tanh();
                } else {
                    l = l.clamp(-1.0, 1.0);
                    r = r.clamp(-1.0, 1.0);
                }
                pk[0] = pk[0].max(l.abs());
                pk[1] = pk[1].max(r.abs());
                let o = (done + i) * out_ch;
                out[o] = l;
                if out_ch > 1 {
                    out[o + 1] = r;
                    for c in 2..out_ch {
                        out[o + c] = 0.0;
                    }
                }
                let sp = s.scope_pos;
                s.scope[sp] = (l + r) * 0.5;
                s.scope_pos = (sp + 1) % s.scope.len();
            }
            let m = &mut s.meters[0];
            m[0] = (m[0] * 0.92).max(pk[0]);
            m[1] = (m[1] * 0.92).max(pk[1]);
            done += n;
        }
        s.voices = self.voices.len();
        let budget = frames as f32 / self.sr;
        let used = t_start.elapsed().as_secs_f32();
        s.load = s.load * 0.9 + 0.1 * (used / budget).min(1.0);
    }
}

/// An effect only works while its slot is loaded on the insert.
pub fn effective_fx(ins: &Insert) -> (f32, f32, f32) {
    let has = |k: FxKind| ins.slots.iter().any(|x| *x == Some(k));
    (
        if has(FxKind::Filter) { ins.lowpass } else { 1.0 },
        if has(FxKind::Delay) { ins.delay } else { 0.0 },
        if has(FxKind::Reverb) { ins.reverb } else { 0.0 },
    )
}

pub type SharedRef = Arc<Mutex<Shared>>;

/// Keeps the audio stream alive.
pub struct AudioOut {
    _stream: Option<cpal::Stream>,
}

impl AudioOut {
    /// Start (or restart) the output. Browsers keep audio suspended until the page is clicked,
    /// so the app calls this again when playback starts.
    pub fn resume(&self) {
        use cpal::traits::StreamTrait;
        if let Some(s) = &self._stream {
            let _ = s.play();
        }
    }
}

pub fn start_audio(shared: SharedRef, want_device: bool) -> AudioOut {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    if want_device {
        let host = cpal::default_host();
        if let Some(dev) = host.default_output_device() {
            if let Ok(cfg) = dev.default_output_config() {
                let sr = cfg.sample_rate().0 as f32;
                let ch = cfg.channels() as usize;
                let name = dev.name().unwrap_or_else(|_| "Output".into());
                if cfg.sample_format() == cpal::SampleFormat::F32 {
                    let mut engine = Engine::new(sr);
                    let sh = shared.clone();
                    let config: cpal::StreamConfig = cfg.into();
                    let stream = dev.build_output_stream(
                        &config,
                        move |data: &mut [f32], _| {
                            if let Ok(mut s) = sh.lock() {
                                engine.render(&mut s, data, ch);
                            } else {
                                data.fill(0.0);
                            }
                        },
                        |e| eprintln!("audio error: {e}"),
                        None,
                    );
                    if let Ok(stream) = stream {
                        if stream.play().is_ok() {
                            let mut s = shared.lock().unwrap();
                            s.sample_rate = sr;
                            s.audio_ok = true;
                            s.device_name = name;
                            return AudioOut { _stream: Some(stream) };
                        }
                    }
                }
            }
        }
    }
    // No device: keep the engine running in real time so the app still works (silently).
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let sr = SAMPLE_RATE as f32;
        let mut engine = Engine::new(sr);
        let mut buf = vec![0.0f32; 441 * 2];
        loop {
            {
                let mut s = shared.lock().unwrap();
                s.sample_rate = sr;
                engine.render(&mut s, &mut buf, 2);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    AudioOut { _stream: None }
}

/// Render the whole song (song mode, from the start) to a 16-bit stereo WAV.
pub struct ExportInfo {
    pub seconds: f32,
    /// The song's tempo measured back from the rendered audio (clip-tempo rule).
    pub measured_bpm: Option<f32>,
}

pub fn export_wav(project: &Project, path: &std::path::Path) -> std::io::Result<ExportInfo> {
    let sr = SAMPLE_RATE as f32;
    let mut s = Shared::new(project.clone());
    s.song_mode = true;
    s.playing = true;
    let song_ticks = s.loop_len() as f64;
    let secs = song_ticks / ticks_per_second(project.tempo) + 2.0;
    let total = (secs * sr as f64) as usize;
    let mut engine = Engine::new(sr);
    let mut pcm: Vec<i16> = Vec::with_capacity(total * 2);
    let mut buf = vec![0.0f32; 512 * 2];
    let mut rendered = 0;
    let play_frames = (song_ticks / ticks_per_second(project.tempo) * sr as f64) as usize;
    while rendered < total {
        if rendered >= play_frames {
            s.playing = false;
        }
        engine.render(&mut s, &mut buf, 2);
        for x in &buf {
            pcm.push((x.clamp(-1.0, 1.0) * 32767.0) as i16);
        }
        rendered += 512;
    }
    use std::io::Write;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    let data_len = (pcm.len() * 2) as u32;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&SAMPLE_RATE.to_le_bytes())?;
    f.write_all(&(SAMPLE_RATE * 4).to_le_bytes())?;
    f.write_all(&4u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    for v in pcm {
        f.write_all(&v.to_le_bytes())?;
    }
    let beats = song_ticks as u64 / PPQ as u64;
    let measured_bpm = clip_tempo(beats * 192, play_frames as u64, SAMPLE_RATE);
    Ok(ExportInfo { seconds: secs as f32, measured_bpm })
}
