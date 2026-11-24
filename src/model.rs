//! Project data: channels, patterns, playlist, mixer, and the timing and gain rules BeatCraft uses.

use serde::{Deserialize, Serialize};

// ---- timing, range and gain constants ----
pub const DEFAULT_TEMPO: f32 = 140.0;
pub const PPQ: u32 = 96; // ticks per quarter note
pub const STEPS_PER_BEAT: u32 = 4; // 1/16 steps
pub const MAX_PATTERNS: usize = 999;
pub const BEATS_PER_BAR: u32 = 4;
pub const STEP_TICKS: u32 = PPQ / 4; // 24
pub const STEPS_PER_BAR: u32 = BEATS_PER_BAR * PPQ / STEP_TICKS; // 16
pub const DEFAULT_PATTERN_STEPS: usize = 16;
pub const BAR_TICKS: u32 = STEPS_PER_BAR * STEP_TICKS;
pub const MAX_SONG_BARS: u32 = 1000;
pub const MAX_TICKS: u32 = MAX_SONG_BARS * BAR_TICKS;
pub const TEMPO_MIN: f32 = 20.0;
pub const TEMPO_MAX: f32 = 400.0;
pub const SWING_MAX_DELAY: f32 = 0.5;
pub const SAMPLE_RATE: u32 = 44100;
pub const CHANNEL_DEFAULT_VOLUME: f32 = 0.8;
pub const CHANNEL_DEFAULT_PAN: f32 = 0.0;
/// Fader position that is 0 dB; the top of the fader is (1 / FADER_UNITY)^3, about +5.8 dB.
pub const FADER_UNITY: f32 = 0.8;

/// Cubic taper: fine control at low levels. x in 0..1 (and a little above) -> linear gain, 1 is 0 dB.
pub fn taper(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    x * x * x
}

/// Channel volume knob (0..1) to linear gain: 1.0 is 0 dB.
pub fn channel_gain(v: f32) -> f32 {
    taper(v)
}

/// Snap a swing amount to whole percent.
pub fn snap_swing(v: f32) -> f32 {
    (v.clamp(0.0, 1.0) * 100.0).round() / 100.0
}
pub const MIXER_INSERTS: usize = 125;
pub const MIXER_TRACKS: usize = MIXER_INSERTS + 2; // master + inserts + current
pub const FX_SLOTS: usize = 10;
pub const PLAYLIST_TRACKS: usize = 500;
pub const ROOT_KEY: u8 = 60; // C5 in this app's naming

/// Mixer fader position (0..1) to linear gain: FADER_UNITY (0.8) is 0 dB, the top is about +5.8 dB.
pub fn fader_gain(v: f32) -> f32 {
    taper(v / FADER_UNITY)
}

pub fn fader_db(v: f32) -> f32 {
    let g = fader_gain(v);
    if g <= 0.000_01 {
        f32::NEG_INFINITY
    } else {
        20.0 * g.log10()
    }
}

/// Constant-power pan law. pan in -1..1 -> (left, right) gains, both 1/sqrt(2) at centre.
pub fn pan_gains(pan: f32) -> (f32, f32) {
    let a = (pan.clamp(-1.0, 1.0) + 1.0) * std::f32::consts::FRAC_PI_4;
    (a.cos(), a.sin())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstKind {
    Kick,
    Snare,
    Clap,
    HiHat,
    OpenHat,
    Tom,
    Bass,
    Lead,
    Pad,
    Pluck,
    Keys,
}

impl InstKind {
    pub const ALL: [InstKind; 11] = [
        InstKind::Kick,
        InstKind::Snare,
        InstKind::Clap,
        InstKind::HiHat,
        InstKind::OpenHat,
        InstKind::Tom,
        InstKind::Bass,
        InstKind::Lead,
        InstKind::Pad,
        InstKind::Pluck,
        InstKind::Keys,
    ];
    pub fn label(self) -> &'static str {
        match self {
            InstKind::Kick => "Kick",
            InstKind::Snare => "Snare",
            InstKind::Clap => "Clap",
            InstKind::HiHat => "Hat",
            InstKind::OpenHat => "Open hat",
            InstKind::Tom => "Tom",
            InstKind::Bass => "Sub bass",
            InstKind::Lead => "Saw lead",
            InstKind::Pad => "Warm pad",
            InstKind::Pluck => "Pluck",
            InstKind::Keys => "E-keys",
        }
    }
    pub fn is_drum(self) -> bool {
        matches!(self, InstKind::Kick | InstKind::Snare | InstKind::Clap | InstKind::HiHat | InstKind::OpenHat | InstKind::Tom)
    }
    pub fn default_params(self) -> SynthParams {
        let mut p = SynthParams::default();
        match self {
            InstKind::Bass => {
                p.attack = 0.003;
                p.decay = 0.25;
                p.sustain = 0.8;
                p.release = 0.08;
                p.cutoff = 0.35;
            }
            InstKind::Lead => {
                p.attack = 0.005;
                p.decay = 0.3;
                p.sustain = 0.6;
                p.release = 0.18;
                p.cutoff = 0.65;
                p.reso = 0.3;
            }
            InstKind::Pad => {
                p.attack = 0.45;
                p.decay = 0.8;
                p.sustain = 0.8;
                p.release = 0.9;
                p.cutoff = 0.45;
            }
            InstKind::Pluck => {
                p.attack = 0.001;
                p.decay = 0.22;
                p.sustain = 0.0;
                p.release = 0.15;
                p.cutoff = 0.7;
                p.reso = 0.2;
            }
            InstKind::Keys => {
                p.attack = 0.002;
                p.decay = 1.2;
                p.sustain = 0.25;
                p.release = 0.35;
                p.cutoff = 0.8;
            }
            _ => {}
        }
        p
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SynthParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub cutoff: f32, // 0..1
    pub reso: f32,   // 0..1
    pub tone: f32,   // drum tone / osc detune 0..1
}

impl Default for SynthParams {
    fn default() -> Self {
        Self { attack: 0.001, decay: 0.3, sustain: 0.0, release: 0.1, cutoff: 1.0, reso: 0.0, tone: 0.5 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub kind: InstKind,
    pub volume: f32,
    pub pan: f32,
    pub pitch: f32, // semitones
    pub mute: bool,
    pub insert: usize, // 0 = master, 1..=125 inserts
    pub color: [u8; 3],
    pub params: SynthParams,
}

impl Channel {
    pub fn new(kind: InstKind, insert: usize) -> Self {
        Self {
            name: kind.label().to_string(),
            kind,
            volume: CHANNEL_DEFAULT_VOLUME,
            pan: CHANNEL_DEFAULT_PAN,
            pitch: 0.0,
            mute: false,
            insert,
            color: default_color(kind),
            params: kind.default_params(),
        }
    }
}

fn default_color(kind: InstKind) -> [u8; 3] {
    match kind {
        InstKind::Kick => [235, 120, 90],
        InstKind::Snare => [235, 170, 80],
        InstKind::Clap => [230, 200, 90],
        InstKind::HiHat | InstKind::OpenHat => [150, 210, 110],
        InstKind::Tom => [210, 140, 200],
        InstKind::Bass => [90, 170, 235],
        InstKind::Lead => [120, 140, 240],
        InstKind::Pad => [170, 120, 230],
        InstKind::Pluck => [90, 210, 200],
        InstKind::Keys => [220, 110, 160],
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub key: u8,
    pub start: u32, // ticks
    pub len: u32,   // ticks
    pub vel: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChannelPattern {
    pub steps: Vec<f32>, // velocity, 0 = off
    pub notes: Vec<Note>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub steps: usize,
    pub color: [u8; 3],
    pub data: Vec<ChannelPattern>, // index = channel
}

impl Pattern {
    pub fn new(index: usize, channels: usize) -> Self {
        let hues: [[u8; 3]; 6] = [[96, 150, 200], [120, 186, 106], [200, 128, 96], [176, 112, 196], [200, 170, 80], [90, 180, 180]];
        let mut p = Self { name: format!("Pattern {}", index + 1), steps: DEFAULT_PATTERN_STEPS, color: hues[index % hues.len()], data: Vec::new() };
        p.ensure_channels(channels);
        p
    }
    pub fn ensure_channels(&mut self, n: usize) {
        while self.data.len() < n {
            self.data.push(ChannelPattern::default());
        }
        for d in &mut self.data {
            d.steps.resize(self.steps, 0.0);
        }
    }
    /// Length in ticks: whole bars covering steps and notes.
    pub fn length_ticks(&self) -> u32 {
        let mut end = self.steps as u32 * STEP_TICKS;
        for d in &self.data {
            for n in &d.notes {
                end = end.max(n.start + n.len);
            }
        }
        // Auto length: round the content up to whole beats and count STEPS_PER_BEAT
        // steps per beat. An empty pattern still plays one bar.
        let steps = end.div_ceil(PPQ) * STEPS_PER_BEAT;
        (steps * STEP_TICKS).max(STEPS_PER_BAR * STEP_TICKS)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Clip {
    pub track: usize,
    pub pattern: usize,
    pub start: u32, // ticks
    pub len: u32,   // ticks
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Insert {
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub lowpass: f32, // 0..1 (1 = open)
    pub delay: f32,   // wet 0..1
    pub reverb: f32,  // wet 0..1
    pub slots: Vec<Option<FxKind>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FxKind {
    Filter,
    Delay,
    Reverb,
    Limiter,
}

impl FxKind {
    pub fn label(self) -> &'static str {
        match self {
            FxKind::Filter => "Low-pass filter",
            FxKind::Delay => "Echo delay",
            FxKind::Reverb => "Room reverb",
            FxKind::Limiter => "Soft limiter",
        }
    }
}

impl Insert {
    pub fn new(index: usize) -> Self {
        let name = if index == 0 {
            "Master".to_string()
        } else if index == MIXER_TRACKS - 1 {
            "Current".to_string()
        } else {
            format!("Insert {}", index)
        };
        let mut slots = vec![None; FX_SLOTS];
        if index == 0 {
            slots[0] = Some(FxKind::Limiter);
        }
        Self { name, volume: FADER_UNITY, pan: 0.0, mute: false, solo: false, lowpass: 1.0, delay: 0.0, reverb: 0.0, slots }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub tempo: f32,
    pub swing: f32,
    pub master_volume: f32,
    pub master_pitch: f32,
    pub channels: Vec<Channel>,
    pub patterns: Vec<Pattern>,
    pub clips: Vec<Clip>,
    pub track_names: Vec<String>,
    pub mixer: Vec<Insert>,
}

impl Project {
    pub fn empty() -> Self {
        let mut p = Self {
            name: "Untitled".into(),
            tempo: DEFAULT_TEMPO,
            swing: 0.0,
            master_volume: FADER_UNITY,
            master_pitch: 0.0,
            channels: Vec::new(),
            patterns: Vec::new(),
            clips: Vec::new(),
            track_names: (1..=PLAYLIST_TRACKS).map(|i| format!("Track {}", i)).collect(),
            mixer: (0..MIXER_TRACKS).map(Insert::new).collect(),
        };
        // Four drum channels to start from.
        for k in [InstKind::Kick, InstKind::Clap, InstKind::HiHat, InstKind::Snare] {
            p.add_channel(k);
        }
        p.patterns.push(Pattern::new(0, p.channels.len()));
        p
    }

    /// Make a deserialized project safe for the engine, the UI and the commands.
    pub fn sanitize(&mut self) {
        self.tempo = if self.tempo.is_finite() { self.tempo } else { DEFAULT_TEMPO };
        self.tempo = clamp_tempo(self.tempo);
        self.swing = snap_swing(if self.swing.is_finite() { self.swing } else { 0.0 });
        self.master_volume = if self.master_volume.is_finite() { self.master_volume.clamp(0.0, 1.0) } else { FADER_UNITY };
        self.master_pitch = if self.master_pitch.is_finite() { self.master_pitch.clamp(-12.0, 12.0) } else { 0.0 };

        self.mixer.truncate(MIXER_TRACKS);
        for i in self.mixer.len()..MIXER_TRACKS {
            self.mixer.push(Insert::new(i));
        }
        for (i, ins) in self.mixer.iter_mut().enumerate() {
            let d = Insert::new(i);
            ins.slots.resize(FX_SLOTS, None);
            let unit = |v: f32, def: f32| if v.is_finite() { v.clamp(0.0, 1.0) } else { def };
            ins.volume = unit(ins.volume, d.volume);
            ins.lowpass = unit(ins.lowpass, d.lowpass);
            ins.delay = unit(ins.delay, d.delay);
            ins.reverb = unit(ins.reverb, d.reverb);
            ins.pan = if ins.pan.is_finite() { ins.pan.clamp(-1.0, 1.0) } else { d.pan };
        }

        self.track_names.truncate(PLAYLIST_TRACKS);
        for i in self.track_names.len()..PLAYLIST_TRACKS {
            self.track_names.push(format!("Track {}", i + 1));
        }

        if self.channels.is_empty() {
            self.add_channel(InstKind::Kick);
        }
        for c in &mut self.channels {
            c.insert = c.insert.min(MIXER_INSERTS);
            c.volume = if c.volume.is_finite() { c.volume.clamp(0.0, 1.0) } else { CHANNEL_DEFAULT_VOLUME };
            c.pan = if c.pan.is_finite() { c.pan.clamp(-1.0, 1.0) } else { 0.0 };
            c.pitch = if c.pitch.is_finite() { c.pitch.clamp(-24.0, 24.0) } else { 0.0 };
            let q = c.params;
            if ![q.attack, q.decay, q.sustain, q.release, q.cutoff, q.reso, q.tone].iter().all(|v| v.is_finite()) {
                c.params = c.kind.default_params();
            }
        }

        let n = self.channels.len();
        if self.patterns.is_empty() {
            self.patterns.push(Pattern::new(0, n));
        }
        self.patterns.truncate(MAX_PATTERNS);
        for pat in &mut self.patterns {
            pat.steps = match pat.steps {
                0..=16 => 16,
                17..=32 => 32,
                _ => 64,
            };
            pat.data.truncate(n);
            pat.ensure_channels(n);
            for d in &mut pat.data {
                for v in &mut d.steps {
                    *v = if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 };
                }
                for note in &mut d.notes {
                    note.key = note.key.min(127);
                    note.vel = if note.vel.is_finite() { note.vel.clamp(0.05, 1.0) } else { 0.8 };
                    note.start = note.start.min(MAX_TICKS - 1);
                    note.len = note.len.clamp(1, MAX_TICKS - note.start);
                }
            }
        }

        let patterns = self.patterns.len();
        self.clips.retain(|c| c.pattern < patterns && c.track < PLAYLIST_TRACKS && c.start < MAX_TICKS);
        for c in &mut self.clips {
            c.len = c.len.clamp(1, MAX_TICKS - c.start);
        }
    }

    pub fn from_json(text: &str) -> Result<Project, String> {
        let mut p: Project = serde_json::from_str(text).map_err(|e| format!("not a BeatCraft project: {e}"))?;
        p.sanitize();
        Ok(p)
    }

    pub fn add_channel(&mut self, kind: InstKind) -> usize {
        let insert = (self.channels.len() + 1).min(MIXER_INSERTS);
        let mut ch = Channel::new(kind, insert);
        let same = self.channels.iter().filter(|c| c.kind == kind).count();
        if same > 0 {
            ch.name = format!("{} {}", kind.label(), same + 1);
        }
        if self.mixer[insert].name.starts_with("Insert ") {
            self.mixer[insert].name = ch.name.clone();
        }
        self.channels.push(ch);
        let n = self.channels.len();
        for p in &mut self.patterns {
            p.ensure_channels(n);
        }
        n - 1
    }

    pub fn remove_channel(&mut self, idx: usize) {
        if idx >= self.channels.len() || self.channels.len() <= 1 {
            return;
        }
        self.channels.remove(idx);
        for p in &mut self.patterns {
            if idx < p.data.len() {
                p.data.remove(idx);
            }
        }
    }

    pub fn add_pattern(&mut self) -> usize {
        // Pattern indexes go up to MAX_PATTERNS.
        if self.patterns.len() >= MAX_PATTERNS {
            return self.patterns.len() - 1;
        }
        let i = self.patterns.len();
        self.patterns.push(Pattern::new(i, self.channels.len()));
        i
    }

    pub fn song_length_ticks(&self) -> u32 {
        let bar = STEPS_PER_BAR * STEP_TICKS;
        let end = self.clips.iter().map(|c| c.start + c.len).max().unwrap_or(0);
        end.div_ceil(bar).max(1) * bar
    }

    /// An original demo beat so the app makes sound right away.
    pub fn demo() -> Self {
        let mut p = Self::empty();
        p.name = "Night Drive (demo)".into();
        p.tempo = DEFAULT_TEMPO;
        p.swing = 0.18;
        let bass = p.add_channel(InstKind::Bass);
        let lead = p.add_channel(InstKind::Pluck);
        let pad = p.add_channel(InstKind::Pad);
        let oh = p.add_channel(InstKind::OpenHat);
        p.channels[2].volume = 0.55; // hat
        p.channels[oh].volume = 0.45;
        p.channels[pad].volume = 0.55;
        p.mixer[p.channels[pad].insert].reverb = 0.45;
        p.mixer[p.channels[lead].insert].delay = 0.3;
        p.mixer[p.channels[lead].insert].reverb = 0.2;
        p.mixer[p.channels[lead].insert].slots[0] = Some(FxKind::Delay);
        p.mixer[p.channels[lead].insert].slots[1] = Some(FxKind::Reverb);
        p.mixer[p.channels[pad].insert].slots[0] = Some(FxKind::Reverb);
        p.mixer[p.channels[1].insert].reverb = 0.15;
        p.mixer[p.channels[1].insert].slots[0] = Some(FxKind::Reverb);

        // Pattern 1: drums + bass groove (two bars)
        {
            let d = &mut p.patterns[0];
            d.name = "Beat".into();
            d.steps = 32;
            d.ensure_channels(p.channels.len());
            for bar in 0..2 {
                let o = bar * 16;
                for s in [0, 7, 10] {
                    d.data[0].steps[o + s] = 1.0;
                }
                for s in [4, 12] {
                    d.data[1].steps[o + s] = 0.9;
                }
                for s in (0..16).step_by(2) {
                    d.data[2].steps[o + s] = if s % 4 == 2 { 0.9 } else { 0.6 };
                }
                d.data[2].steps[o + 15] = 0.5;
                d.data[3].steps[o + 12] = 0.5;
                d.data[oh].steps[o + 14] = 0.7;
            }
            d.data[3].steps[31] = 0.35;
            d.data[0].steps[30] = 0.7;
            let roots = [45u8, 45, 41, 43]; // A, A, F, G (one per half bar)
            for (i, r) in roots.iter().enumerate() {
                let base = i as u32 * 8 * STEP_TICKS;
                for (o, len, oct) in [(0u32, 3u32, 0u8), (3, 1, 0), (4, 2, 12), (6, 2, 0)] {
                    d.data[bass].notes.push(Note { key: *r + oct, start: base + o * STEP_TICKS, len: len * STEP_TICKS, vel: if oct > 0 { 0.7 } else { 0.95 } });
                }
            }
        }
        // Pattern 2: pad chords (two bars)
        let pb = p.add_pattern();
        {
            let d = &mut p.patterns[pb];
            d.name = "Chords".into();
            d.steps = 32;
            d.ensure_channels(p.channels.len());
            let chords: [(u32, u32, [u8; 4]); 3] = [(0, 16, [57, 64, 69, 72]), (16, 8, [53, 60, 65, 69]), (24, 8, [55, 62, 67, 71])];
            for (st, len, ch) in chords {
                for k in ch {
                    d.data[pad].notes.push(Note { key: k, start: st * STEP_TICKS, len: len * STEP_TICKS, vel: 0.7 });
                }
            }
        }
        // Pattern 3: pluck melody (two bars)
        let pm = p.add_pattern();
        {
            let d = &mut p.patterns[pm];
            d.name = "Melody".into();
            d.steps = 32;
            d.ensure_channels(p.channels.len());
            let mel: [(u32, u8, u32); 12] =
                [(0, 76, 2), (3, 79, 1), (4, 76, 2), (6, 74, 2), (8, 72, 3), (11, 74, 1), (12, 76, 4), (16, 72, 2), (19, 74, 1), (20, 72, 2), (22, 71, 2), (24, 69, 6)];
            for (s, k, l) in mel {
                d.data[lead].notes.push(Note { key: k, start: s * STEP_TICKS, len: l * STEP_TICKS, vel: 0.85 });
            }
        }
        // Arrangement: 16 bars
        let bar = STEPS_PER_BAR * STEP_TICKS;
        p.track_names[0] = "Beat".into();
        p.track_names[1] = "Chords".into();
        p.track_names[2] = "Melody".into();
        for b in (2..16u32).step_by(2) {
            p.clips.push(Clip { track: 0, pattern: 0, start: b * bar, len: 2 * bar });
        }
        for b in (0..16u32).step_by(2) {
            p.clips.push(Clip { track: 1, pattern: pb, start: b * bar, len: 2 * bar });
        }
        for b in (4..16u32).step_by(2) {
            if b != 10 {
                p.clips.push(Clip { track: 2, pattern: pm, start: b * bar, len: 2 * bar });
            }
        }
        p
    }
}

pub fn key_name(key: u8) -> String {
    const N: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    format!("{}{}", N[(key % 12) as usize], key / 12)
}

/// Tempo of a piece of audio: `beats` played in `frames` samples at `rate`, rounded to a whole BPM.
/// `None` when there is nothing to measure or the result is outside the supported tempo range.
pub fn measure_tempo(beats: f64, frames: u64, rate: u32) -> Option<f32> {
    if frames == 0 || beats <= 0.0 {
        return None;
    }
    let bpm = (60.0 * beats * rate as f64 / frames as f64).round();
    (TEMPO_MIN as f64..=TEMPO_MAX as f64).contains(&bpm).then_some(bpm as f32)
}

/// Clamp a tempo to the supported range.
pub fn clamp_tempo(t: f32) -> f32 {
    t.clamp(TEMPO_MIN, TEMPO_MAX)
}

/// Samples per tick: rate * 60 / (tempo * PPQ).
pub fn samples_per_tick(rate: f64, tempo: f32) -> f64 {
    rate * 60.0 / (clamp_tempo(tempo) as f64 * PPQ as f64)
}

pub fn ticks_per_second(tempo: f32) -> f64 {
    1.0 / samples_per_tick(1.0, tempo)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

    fn near(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    fn broken_json() -> serde_json::Value {
        let mut v = serde_json::to_value(Project::empty()).unwrap();
        v["mixer"] = json!(v["mixer"].as_array().unwrap()[..3].to_vec());
        v["track_names"] = json!([]);
        v["channels"][0]["insert"] = json!(9999);
        v["tempo"] = json!(0);
        v["patterns"][0]["steps"] = json!(7);
        v["patterns"][0]["data"][0]["notes"] = json!([{"key": 200, "start": u32::MAX, "len": u32::MAX, "vel": 5.0}]);
        v["clips"] = json!([
            {"track": 0, "pattern": 99, "start": 0, "len": 96},
            {"track": 0, "pattern": 0, "start": u32::MAX, "len": 96},
            {"track": 0, "pattern": 0, "start": 384, "len": 384},
        ]);
        v
    }

    pub(crate) fn broken_project() -> Project {
        Project::from_json(&broken_json().to_string()).unwrap()
    }

    #[test]
    fn volume_curve_anchors() {
        assert!(near(channel_gain(1.0), 1.0, 1e-6));
        assert!(near(fader_gain(FADER_UNITY), 1.0, 1e-6));
        assert_eq!(fader_db(0.0), f32::NEG_INFINITY);
        assert!((5.5..6.0).contains(&fader_db(1.0)));
        let g = channel_gain(CHANNEL_DEFAULT_VOLUME);
        assert!(g < 1.0 && g > 0.4);
        let levels: Vec<f32> = (1..=10).map(|i| taper(i as f32 / 10.0)).collect();
        assert!(levels.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn pan_law_is_constant_power() {
        for pan in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let (l, r) = pan_gains(pan);
            assert!(near(l * l + r * r, 1.0, 1e-5), "pan {pan}");
        }
        let (l, r) = pan_gains(0.0);
        assert!(near(l, std::f32::consts::FRAC_1_SQRT_2, 1e-3) && near(r, std::f32::consts::FRAC_1_SQRT_2, 1e-3));
        let (l, r) = pan_gains(-1.0);
        assert!(near(l, 1.0, 1e-5) && near(r, 0.0, 1e-5));
    }

    #[test]
    fn timing_constants_and_conversions() {
        assert_eq!(STEP_TICKS, 24);
        assert_eq!(STEPS_PER_BAR, 16);
        assert_eq!(BAR_TICKS, 384);
        assert!((samples_per_tick(44100.0, 140.0) - 196.875).abs() < 1e-9);
        assert!((ticks_per_second(120.0) - 192.0).abs() < 1e-9);
        assert_eq!(samples_per_tick(44100.0, 1.0), samples_per_tick(44100.0, TEMPO_MIN));
    }

    #[test]
    fn swing_snaps_to_whole_percent() {
        assert_eq!(snap_swing(0.126), 0.13);
        assert_eq!(snap_swing(2.0), 1.0);
        assert_eq!(snap_swing(-1.0), 0.0);
    }

    #[test]
    fn pattern_length_rounds_to_whole_beats() {
        let mut p = Pattern::new(0, 1);
        assert_eq!(p.length_ticks(), 384);
        p.steps = 32;
        assert_eq!(p.length_ticks(), 768);
        p.steps = 16;
        p.data[0].notes.push(Note { key: 60, start: 380, len: 5, vel: 0.8 });
        assert_eq!(p.length_ticks(), 480);
    }

    #[test]
    fn add_channel_names_and_routes() {
        let mut p = Project::empty();
        assert_eq!(p.channels.len(), 4);
        for (i, c) in p.channels.iter().enumerate() {
            assert_eq!(c.insert, i + 1);
        }
        let i = p.add_channel(InstKind::Kick);
        assert_eq!(p.channels[i].name, "Kick 2");
        assert_eq!(p.channels[i].insert, 5);
        assert!(p.patterns.iter().all(|pt| pt.data.len() == 5));
    }

    #[test]
    fn remove_channel_keeps_the_last_one_and_pattern_data_aligned() {
        let mut p = Project::empty();
        while p.channels.len() > 1 {
            p.remove_channel(0);
            assert_eq!(p.patterns[0].data.len(), p.channels.len());
        }
        p.remove_channel(0);
        assert_eq!(p.channels.len(), 1);
        assert_eq!(p.patterns[0].data.len(), 1);
    }

    #[test]
    fn add_pattern_stops_at_the_limit() {
        let mut p = Project::empty();
        for _ in 0..MAX_PATTERNS + 5 {
            p.add_pattern();
        }
        assert_eq!(p.patterns.len(), MAX_PATTERNS);
    }

    #[test]
    fn measured_tempo_window() {
        assert_eq!(measure_tempo(8.0, (44100.0 * 60.0 * 8.0 / 140.0) as u64, 44100), Some(140.0));
        assert_eq!(measure_tempo(1.0, 0, 44100), None);
        assert_eq!(measure_tempo(1.0, 44100 / 20, 44100), None);
        assert_eq!(measure_tempo(1.0, 44100 * 10, 44100), None);
    }

    #[test]
    fn key_names() {
        assert_eq!(key_name(60), "C5");
        assert_eq!(key_name(61), "C#5");
    }

    #[test]
    fn sanitize_leaves_good_projects_alone() {
        for p in [Project::empty(), Project::demo()] {
            let before = serde_json::to_value(&p).unwrap();
            let mut q = p.clone();
            q.sanitize();
            assert_eq!(before, serde_json::to_value(&q).unwrap());
        }
    }

    #[test]
    fn sanitize_is_idempotent() {
        let p = broken_project();
        let once = serde_json::to_value(&p).unwrap();
        let mut q = p.clone();
        q.sanitize();
        assert_eq!(once, serde_json::to_value(&q).unwrap());
    }

    #[test]
    fn sanitize_repairs_a_broken_project() {
        let p = broken_project();
        assert_eq!(p.mixer.len(), MIXER_TRACKS);
        assert!(p.mixer.iter().all(|m| m.slots.len() == FX_SLOTS));
        assert_eq!(p.track_names.len(), PLAYLIST_TRACKS);
        assert_eq!(p.track_names[0], "Track 1");
        assert!(p.channels.iter().all(|c| c.insert <= MIXER_INSERTS));
        assert_eq!(p.channels[0].insert, MIXER_INSERTS);
        assert_eq!(p.tempo, TEMPO_MIN);
        let pat = &p.patterns[0];
        assert_eq!(pat.steps, 16);
        assert!(pat.data.iter().all(|d| d.steps.len() == 16));
        let n = pat.data[0].notes[0];
        assert_eq!(n.key, 127);
        assert!(n.start as u64 + n.len as u64 <= MAX_TICKS as u64);
        assert!(n.len >= 1 && n.vel <= 1.0);
        assert_eq!(p.clips, vec![Clip { track: 0, pattern: 0, start: 384, len: 384 }]);
    }

    #[test]
    fn sanitize_handles_non_finite_numbers() {
        let mut p = Project::empty();
        p.tempo = f32::NAN;
        p.swing = f32::INFINITY;
        p.master_volume = f32::NAN;
        p.channels[0].volume = f32::NAN;
        p.channels[0].params.cutoff = f32::NAN;
        p.mixer[1].volume = f32::NAN;
        p.mixer[1].pan = f32::INFINITY;
        p.patterns[0].data[0].steps[0] = f32::NAN;
        p.sanitize();
        assert_eq!(p.tempo, DEFAULT_TEMPO);
        assert_eq!(p.swing, 0.0);
        assert_eq!(p.master_volume, FADER_UNITY);
        assert_eq!(p.channels[0].volume, CHANNEL_DEFAULT_VOLUME);
        assert!(p.channels[0].params.cutoff.is_finite());
        assert_eq!(p.mixer[1].volume, FADER_UNITY);
        assert_eq!(p.mixer[1].pan, 0.0);
        assert_eq!(p.patterns[0].data[0].steps[0], 0.0);
    }

    #[test]
    fn sanitize_adds_missing_channels_and_patterns() {
        let mut v = serde_json::to_value(Project::empty()).unwrap();
        v["channels"] = json!([]);
        v["patterns"] = json!([]);
        let p = Project::from_json(&v.to_string()).unwrap();
        assert_eq!(p.channels.len(), 1);
        assert_eq!(p.patterns.len(), 1);
        assert_eq!(p.patterns[0].data.len(), 1);
    }

    #[test]
    fn from_json_rejects_garbage() {
        let e = Project::from_json("{\"nope\": 1}").unwrap_err();
        assert!(e.starts_with("not a BeatCraft project"), "{e}");
    }

    #[test]
    fn json_round_trip_is_stable() {
        let a = serde_json::to_string(&Project::demo()).unwrap();
        let b = serde_json::to_string(&Project::from_json(&a).unwrap()).unwrap();
        assert_eq!(a, b);
    }
}
