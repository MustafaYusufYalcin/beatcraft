//! Built-in instruments and insert effects (our own simple designs).

use crate::model::{InstKind, SynthParams};
use std::f32::consts::TAU;

pub struct Rng(u32);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }
    #[inline]
    pub fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Simple state-variable low-pass.
#[derive(Default, Clone)]
pub struct Svf {
    low: f32,
    band: f32,
}
impl Svf {
    #[inline]
    pub fn lp(&mut self, x: f32, cutoff_hz: f32, reso: f32, sr: f32) -> f32 {
        let f = (2.0 * (std::f32::consts::PI * cutoff_hz.min(sr * 0.22) / sr).sin()).min(1.2);
        let q = 1.0 - reso.clamp(0.0, 0.95);
        self.low += f * self.band;
        let high = x - self.low - q * self.band;
        self.band += f * high;
        self.low
    }
    #[inline]
    pub fn hp(&mut self, x: f32, cutoff_hz: f32, sr: f32) -> f32 {
        let f = (2.0 * (std::f32::consts::PI * cutoff_hz.min(sr * 0.22) / sr).sin()).min(1.2);
        self.low += f * self.band;
        let high = x - self.low - self.band;
        self.band += f * high;
        high
    }
}

pub fn midi_hz(key: f32) -> f32 {
    440.0 * 2f32.powf((key - 69.0) / 12.0)
}

#[inline]
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        t + t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + t + t + 1.0
    } else {
        0.0
    }
}

pub struct Voice {
    pub channel: usize,
    pub key: u8,
    pub live: bool,
    kind: InstKind,
    params: SynthParams,
    vel: f32,
    hz: f32,
    t: f32,        // seconds since start
    gate_len: f32, // seconds until release (f32::MAX for held live notes)
    released_at: Option<f32>,
    rel_level: f32,
    phases: [f32; 3],
    filt: Svf,
    filt2: Svf,
    rng: Rng,
    pub done: bool,
    pub gain_l: f32,
    pub gain_r: f32,
}

impl Voice {
    #[allow(clippy::too_many_arguments)]
    pub fn new(channel: usize, kind: InstKind, params: SynthParams, key: u8, pitch: f32, vel: f32, gate_len: f32, live: bool, seed: u32) -> Self {
        Self {
            channel,
            key,
            live,
            kind,
            params,
            vel,
            hz: midi_hz(key as f32 + pitch),
            t: 0.0,
            gate_len,
            released_at: None,
            rel_level: 0.0,
            phases: [0.0, 0.33, 0.66],
            filt: Svf::default(),
            filt2: Svf::default(),
            rng: Rng::new(seed),
            done: false,
            gain_l: 1.0,
            gain_r: 1.0,
        }
    }

    pub fn release(&mut self) {
        if self.released_at.is_none() {
            self.gate_len = self.t;
        }
    }

    fn adsr(&mut self) -> f32 {
        let p = &self.params;
        let held = |t: f32| -> f32 {
            if t < p.attack {
                t / p.attack.max(1e-4)
            } else {
                let d = t - p.attack;
                p.sustain + (1.0 - p.sustain) * (-d / p.decay.max(1e-3) * 3.0).exp()
            }
        };
        if self.t < self.gate_len {
            held(self.t)
        } else {
            if self.released_at.is_none() {
                self.released_at = Some(self.t);
                self.rel_level = held(self.gate_len);
            }
            let r = self.t - self.released_at.unwrap();
            let v = self.rel_level * (-r / p.release.max(1e-3) * 4.0).exp();
            if v < 0.0005 {
                self.done = true;
            }
            v
        }
    }

    /// Render one mono sample.
    #[inline]
    pub fn sample(&mut self, sr: f32) -> f32 {
        let dt = 1.0 / sr;
        let t = self.t;
        let tone = self.params.tone;
        let out = match self.kind {
            InstKind::Kick => {
                let f = 45.0 + (120.0 + 160.0 * tone) * (-t * 28.0).exp();
                self.phases[0] = (self.phases[0] + f * dt) % 1.0;
                let body = (self.phases[0] * TAU).sin() * (-t * 6.5).exp();
                let click = self.rng.next() * (-t * 300.0).exp() * 0.25;
                if t > 0.9 {
                    self.done = true;
                }
                (body * 1.3 + click).tanh()
            }
            InstKind::Snare => {
                let f = 180.0 + 60.0 * tone;
                self.phases[0] = (self.phases[0] + f * dt) % 1.0;
                let body = (self.phases[0] * TAU).sin() * (-t * 22.0).exp() * 0.6;
                let n = self.filt.hp(self.rng.next(), 1500.0, sr) * (-t * 14.0).exp();
                if t > 0.6 {
                    self.done = true;
                }
                body + n * 0.9
            }
            InstKind::Clap => {
                let env = if t < 0.03 {
                    let ph = (t / 0.01).fract();
                    (-ph * 6.0).exp()
                } else {
                    (-(t - 0.03) * 18.0).exp()
                };
                let n = self.filt.hp(self.rng.next(), 900.0 + 800.0 * tone, sr);
                let n = self.filt2.lp(n, 5000.0, 0.3, sr);
                if t > 0.6 {
                    self.done = true;
                }
                n * env * 1.6
            }
            InstKind::HiHat | InstKind::OpenHat => {
                let decay = if self.kind == InstKind::HiHat { 45.0 } else { 7.0 };
                let n = self.filt.hp(self.rng.next(), 6500.0 + 2500.0 * tone, sr);
                if t > 1.2 {
                    self.done = true;
                }
                n * (-t * decay).exp() * 0.8
            }
            InstKind::Tom => {
                let f = self.hz * 0.5 * (1.0 + 0.6 * (-t * 20.0).exp());
                self.phases[0] = (self.phases[0] + f * dt) % 1.0;
                if t > 1.0 {
                    self.done = true;
                }
                (self.phases[0] * TAU).sin() * (-t * 7.0).exp()
            }
            _ => {
                let env = self.adsr();
                let p = self.params;
                let hz = self.hz;
                let raw = match self.kind {
                    InstKind::Bass => {
                        self.phases[0] = (self.phases[0] + hz * dt) % 1.0;
                        self.phases[1] = (self.phases[1] + hz * 0.5 * dt) % 1.0;
                        let saw = 2.0 * self.phases[0] - 1.0 - poly_blep(self.phases[0], hz * dt);
                        (self.phases[1] * TAU).sin() * 0.9 + saw * 0.35
                    }
                    InstKind::Lead | InstKind::Pad => {
                        let det = if self.kind == InstKind::Pad { 0.006 } else { 0.004 } * (0.5 + tone);
                        let mut s = 0.0;
                        for (i, m) in [1.0 - det, 1.0, 1.0 + det].iter().enumerate() {
                            let f = hz * m;
                            self.phases[i] = (self.phases[i] + f * dt) % 1.0;
                            s += 2.0 * self.phases[i] - 1.0 - poly_blep(self.phases[i], f * dt);
                        }
                        s * 0.4
                    }
                    InstKind::Pluck => {
                        self.phases[0] = (self.phases[0] + hz * dt) % 1.0;
                        let ph = self.phases[0];
                        let sq = if ph < 0.5 { 1.0 } else { -1.0 } + poly_blep(ph, hz * dt) - poly_blep((ph + 0.5) % 1.0, hz * dt);
                        let saw = 2.0 * ph - 1.0 - poly_blep(ph, hz * dt);
                        sq * 0.4 + saw * 0.4
                    }
                    _ => {
                        // E-keys: two-operator FM
                        self.phases[1] = (self.phases[1] + hz * 2.0 * dt) % 1.0;
                        let m = (self.phases[1] * TAU).sin() * (1.6 * (-t * 3.0).exp() + 0.2);
                        self.phases[0] = (self.phases[0] + hz * dt) % 1.0;
                        ((self.phases[0] * TAU) + m).sin() * 0.8
                    }
                };
                // filter with a short envelope sweep for plucks/leads
                let sweep = match self.kind {
                    InstKind::Pluck => 1.0 + 6.0 * (-t * 14.0).exp(),
                    InstKind::Lead => 1.0 + 1.5 * (-t * 6.0).exp(),
                    InstKind::Bass => 1.0 + 3.0 * (-t * 10.0).exp(),
                    _ => 1.0,
                };
                let cut = (60.0 * 2f32.powf(p.cutoff * 8.5)) * sweep;
                self.filt.lp(raw, cut, p.reso, sr) * env
            }
        };
        self.t += dt;
        out * self.vel
    }
}

/// Insert effects: low-pass, feedback delay, small reverb.
pub struct InsertFx {
    lp_l: Svf,
    lp_r: Svf,
    delay_l: Vec<f32>,
    delay_r: Vec<f32>,
    dpos: usize,
    combs: Vec<(Vec<f32>, usize, f32)>,
    allp: Vec<(Vec<f32>, usize)>,
}

impl InsertFx {
    pub fn new(sr: f32) -> Self {
        let max_delay = (sr * 2.0) as usize;
        let comb_ms = [29.7f32, 37.1, 41.1, 43.7, 31.3, 39.9];
        let ap_ms = [5.0f32, 1.7];
        Self {
            lp_l: Svf::default(),
            lp_r: Svf::default(),
            delay_l: vec![0.0; max_delay],
            delay_r: vec![0.0; max_delay],
            dpos: 0,
            combs: comb_ms.iter().map(|ms| (vec![0.0; (sr * ms / 1000.0) as usize], 0usize, 0.0f32)).collect(),
            allp: ap_ms.iter().map(|ms| (vec![0.0; (sr * ms / 1000.0) as usize], 0usize)).collect(),
        }
    }

    /// Process in place. `delay_samples` is a dotted 1/8 at the current tempo.
    pub fn process(&mut self, l: &mut [f32], r: &mut [f32], lowpass: f32, delay: f32, reverb: f32, delay_samples: usize, sr: f32) {
        let use_lp = lowpass < 0.999;
        let cut = 60.0 * 2f32.powf(lowpass * 8.5);
        let ds = delay_samples.clamp(1, self.delay_l.len() - 1);
        for i in 0..l.len() {
            let mut a = l[i];
            let mut b = r[i];
            if use_lp {
                a = self.lp_l.lp(a, cut, 0.1, sr);
                b = self.lp_r.lp(b, cut, 0.1, sr);
            }
            if delay > 0.001 {
                let n = self.delay_l.len();
                let rp = (self.dpos + n - ds) % n;
                let dl = self.delay_l[rp];
                let dr = self.delay_r[rp];
                // ping-pong
                self.delay_l[self.dpos] = a + dr * 0.45;
                self.delay_r[self.dpos] = b + dl * 0.45;
                self.dpos = (self.dpos + 1) % n;
                a += dl * delay;
                b += dr * delay;
            }
            if reverb > 0.001 {
                let input = (a + b) * 0.5 * 0.3;
                let mut acc_l = 0.0;
                let mut acc_r = 0.0;
                for (k, (buf, pos, store)) in self.combs.iter_mut().enumerate() {
                    let y = buf[*pos];
                    *store = y * 0.7 + *store * 0.3;
                    buf[*pos] = input + *store * 0.82;
                    *pos = (*pos + 1) % buf.len();
                    if k % 2 == 0 {
                        acc_l += y;
                    } else {
                        acc_r += y;
                    }
                }
                let mut y = (acc_l + acc_r) * 0.5;
                for (buf, pos) in self.allp.iter_mut() {
                    let bv = buf[*pos];
                    let o = -y + bv;
                    buf[*pos] = y + bv * 0.5;
                    *pos = (*pos + 1) % buf.len();
                    y = o;
                }
                a += (y * 0.6 + acc_l * 0.2) * reverb;
                b += (y * 0.6 + acc_r * 0.2) * reverb;
            }
            l[i] = a;
            r[i] = b;
        }
    }
}
