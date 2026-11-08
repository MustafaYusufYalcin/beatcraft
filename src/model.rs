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
pub const TEMPO_MIN: f32 = 10.0;
pub const TEMPO_MAX: f32 = 522.0;
pub const SAMPLES_PER_TICK_NUMERATOR: f64 = 15.0; // rate * 15 / (tempo * 24) = rate * 60 / (tempo * PPQ)
pub const REALISTIC_TEMPO_MIN: f32 = 60.0;
pub const REALISTIC_TEMPO_MAX: f32 = 240.0;
pub const SWING_MAX_DELAY: f32 = 0.5;
pub const CLIP_TEMPO_FACTOR: f64 = 0.3125;
pub const SAMPLE_RATE: u32 = 44100;
/// Channel volume/pan raw scale 0..12800: volume = raw / 10000 / 1.28, pan = raw / 12800 * 2 - 1.
pub const CHANNEL_RAW_MAX: f32 = 12800.0;
pub const CHANNEL_RAW_DEFAULT_VOLUME: f32 = 10000.0;
pub const CHANNEL_DEFAULT_VOLUME: f32 = CHANNEL_RAW_DEFAULT_VOLUME / 10000.0 / 1.28; // 0.78125
pub const CHANNEL_DEFAULT_PAN: f32 = 6400.0 / CHANNEL_RAW_MAX * 2.0 - 1.0; // 0.0
/// Mixer fader is scaled by 1.25 before the volume curve, so 0.8 is unity and the top gives headroom.
pub const FADER_SCALE: f32 = 1.25;
pub const FADER_UNITY: f32 = 1.0 / FADER_SCALE; // 0.8 -> 0 dB
/// Main swing has 128 steps (1/128 resolution).
pub const SWING_RESOLUTION: f32 = 128.0;
/// Exponential volume curve shared by channels and mixer: amp = (exp(x * K) - 1) * 0.1.
/// K = ln(11) makes x = 1 exactly 0 dB.
pub const VOLUME_CURVE_K: f32 = 2.3978953; // ln(11)
pub const VOLUME_CURVE_SCALE: f32 = 0.1;

/// Volume curve: x in 0..1(.25) -> linear amplitude.
pub fn volume_curve(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    ((x * VOLUME_CURVE_K).exp() - 1.0) * VOLUME_CURVE_SCALE
}

/// Channel volume knob (0..1) to linear gain: 100% = 0 dB, default 78.125% ≈ -5.2 dB.
pub fn channel_gain(v: f32) -> f32 {
    volume_curve(v)
}

/// Snap a swing amount to 1/128 steps.
pub fn snap_swing(v: f32) -> f32 {
    (v.clamp(0.0, 1.0) * SWING_RESOLUTION).round() / SWING_RESOLUTION
}
pub const MIXER_INSERTS: usize = 125;
pub const MIXER_TRACKS: usize = MIXER_INSERTS + 2; // master + inserts + current
pub const FX_SLOTS: usize = 10;
pub const PLAYLIST_TRACKS: usize = 500;
pub const ROOT_KEY: u8 = 60; // C5 in this app's naming

/// Mixer fader position (0..1) to linear gain: 0.8 is 0 dB, 1.0 is about +5.6 dB.
pub fn fader_gain(v: f32) -> f32 {
    volume_curve(v * FADER_SCALE)
}

pub fn fader_db(v: f32) -> f32 {
    let g = fader_gain(v);
    if g <= 0.000_01 {
        f32::NEG_INFINITY
    } else {
        20.0 * g.log10()
    }
}

/// "Circular" pan law: constant power. pan in -1..1 -> (left, right) gains, both 1/sqrt(2) at centre.
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
        matches!(
            self,
            InstKind::Kick | InstKind::Snare | InstKind::Clap | InstKind::HiHat | InstKind::OpenHat | InstKind::Tom
        )
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
        let hues: [[u8; 3]; 6] =
            [[96, 150, 200], [120, 186, 106], [200, 128, 96], [176, 112, 196], [200, 170, 80], [90, 180, 180]];
        let mut p = Self {
            name: format!("Pattern {}", index + 1),
            steps: DEFAULT_PATTERN_STEPS,
            color: hues[index % hues.len()],
            data: Vec::new(),
        };
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
    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|d| d.notes.is_empty() && d.steps.iter().all(|v| *v <= 0.0))
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
        Self {
            name,
            volume: FADER_UNITY,
            pan: 0.0,
            mute: false,
            solo: false,
            lowpass: 1.0,
            delay: 0.0,
            reverb: 0.0,
            slots,
        }
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
        // Like the factory "Basic" template: four drum channels.
        for k in [InstKind::Kick, InstKind::Clap, InstKind::HiHat, InstKind::Snare] {
            p.add_channel(k);
        }
        p.patterns.push(Pattern::new(0, p.channels.len()));
        p
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
        p.swing = 23.0 / SWING_RESOLUTION;
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
            let mel: [(u32, u8, u32); 12] = [
                (0, 76, 2), (3, 79, 1), (4, 76, 2), (6, 74, 2), (8, 72, 3), (11, 74, 1),
                (12, 76, 4), (16, 72, 2), (19, 74, 1), (20, 72, 2), (22, 71, 2), (24, 69, 6),
            ];
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

/// Tempo of a piece of audio: bpm = rate × length × 0.3125 / samples, where length is in
/// 1/192-beat units (0.3125 = 60/192). Results outside the realistic 60–240 window are refused.
pub fn clip_tempo(length_units: u64, samples: u64, rate: u32) -> Option<f32> {
    if samples == 0 {
        return None;
    }
    let bpm = (rate as f64 * length_units as f64 * CLIP_TEMPO_FACTOR / samples as f64).round() as f32;
    (REALISTIC_TEMPO_MIN..=REALISTIC_TEMPO_MAX).contains(&bpm).then_some(bpm)
}

/// Clamp a tempo to the supported range.
pub fn clamp_tempo(t: f32) -> f32 {
    t.clamp(TEMPO_MIN, TEMPO_MAX)
}

/// Samples per tick: rate * 15 / (tempo * 24), where 24 is the ticks per 1/16 step.
pub fn samples_per_tick(rate: f64, tempo: f32) -> f64 {
    rate * SAMPLES_PER_TICK_NUMERATOR / (clamp_tempo(tempo) as f64 * STEP_TICKS as f64)
}

pub fn ticks_per_second(tempo: f32) -> f64 {
    1.0 / samples_per_tick(1.0, tempo)
}
