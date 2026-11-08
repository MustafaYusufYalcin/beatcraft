//! The command layer: every project change as a named command with JSON parameters.
//! The CLI and the MCP server run these; each returns a small JSON result.

use crate::model::*;
use serde_json::{json, Value};

pub struct CommandSpec {
    pub id: &'static str,
    pub params: &'static str,
    pub help: &'static str,
}

pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec { id: "project.new", params: "{template?: \"empty\"|\"demo\"}", help: "Start a new project (empty = four drum channels)." },
    CommandSpec { id: "project.info", params: "{}", help: "Summary: tempo, swing, channels, patterns, clips, song length." },
    CommandSpec { id: "project.rename", params: "{name}", help: "Rename the project." },
    CommandSpec { id: "project.set_tempo", params: "{bpm}", help: "Set the tempo (clamped to 10..522 BPM)." },
    CommandSpec { id: "project.set_swing", params: "{amount: 0..1}", help: "Set the main swing (stored in 1/128 steps)." },
    CommandSpec { id: "channel.add", params: "{kind: Kick|Snare|Clap|HiHat|OpenHat|Tom|Bass|Lead|Pad|Pluck|Keys}", help: "Add an instrument channel; returns its index." },
    CommandSpec { id: "channel.remove", params: "{channel}", help: "Remove a channel." },
    CommandSpec { id: "channel.set", params: "{channel, name?, volume?: 0..1, pan?: -1..1, pitch?: semitones, mute?, insert?: 0..125}", help: "Change a channel's settings." },
    CommandSpec { id: "pattern.add", params: "{name?}", help: "Add a pattern; returns its index." },
    CommandSpec { id: "pattern.set_steps", params: "{pattern, count: 16|32|64}", help: "Set how many steps a pattern shows." },
    CommandSpec { id: "step.set", params: "{pattern, channel, step, velocity?: 0..1}", help: "Light a step (velocity 0 clears it; default 0.8)." },
    CommandSpec { id: "step.fill", params: "{pattern, channel, every, offset?, velocity?}", help: "Light every Nth step, clearing the others." },
    CommandSpec { id: "note.add", params: "{pattern, channel, key: 0..127, start_step, length_steps?, velocity?}", help: "Add a piano-roll note (60 = C5)." },
    CommandSpec { id: "note.clear", params: "{pattern, channel}", help: "Remove all notes and steps of a channel in a pattern." },
    CommandSpec { id: "clip.add", params: "{pattern, track, bar, bars?}", help: "Place a pattern on the playlist (bars counted from 0)." },
    CommandSpec { id: "clip.remove", params: "{index}", help: "Remove a playlist clip." },
    CommandSpec { id: "mixer.set", params: "{insert, volume?: 0..1, pan?, mute?, solo?, lowpass?, delay?, reverb?}", help: "Change a mixer track (0 = Master)." },
    CommandSpec { id: "mixer.load_fx", params: "{insert, slot: 0..9, fx: filter|delay|reverb|limiter|none}", help: "Load an effect into a mixer slot." },
    CommandSpec { id: "song.export_wav", params: "{path}", help: "Render the song to a 44.1 kHz stereo WAV file." },
];

pub fn list_json() -> Value {
    Value::Array(COMMANDS.iter().map(|c| json!({"id": c.id, "params": c.params, "help": c.help})).collect())
}

fn num(p: &Value, k: &str) -> Result<f64, String> {
    p.get(k).and_then(Value::as_f64).ok_or_else(|| format!("missing number `{k}`"))
}
fn opt_num(p: &Value, k: &str) -> Option<f64> {
    p.get(k).and_then(Value::as_f64)
}
fn idx(p: &Value, k: &str, len: usize, what: &str) -> Result<usize, String> {
    let i = num(p, k)? as i64;
    if i < 0 || i as usize >= len {
        return Err(format!("{what} {i} doesn't exist (there are {len})"));
    }
    Ok(i as usize)
}
fn kind_of(s: &str) -> Result<InstKind, String> {
    let s = s.to_lowercase().replace([' ', '-', '_'], "");
    InstKind::ALL
        .iter()
        .copied()
        .find(|k| format!("{k:?}").to_lowercase() == s || k.label().to_lowercase().replace(' ', "") == s)
        .ok_or_else(|| format!("unknown instrument `{s}`"))
}

pub fn info(p: &Project) -> Value {
    let bar = STEPS_PER_BAR * STEP_TICKS;
    json!({
        "name": p.name, "tempo": p.tempo, "swing": p.swing,
        "channels": p.channels.iter().enumerate().map(|(i, c)| json!({"index": i, "name": c.name, "kind": format!("{:?}", c.kind), "volume": c.volume, "pan": c.pan, "mute": c.mute, "insert": c.insert})).collect::<Vec<_>>(),
        "patterns": p.patterns.iter().enumerate().map(|(i, pt)| json!({
            "index": i, "name": pt.name, "steps": pt.steps, "bars": pt.length_ticks() / bar,
            "lit_steps": pt.data.iter().map(|d| d.steps.iter().filter(|v| **v > 0.0).count()).sum::<usize>(),
            "notes": pt.data.iter().map(|d| d.notes.len()).sum::<usize>(),
        })).collect::<Vec<_>>(),
        "clips": p.clips.iter().enumerate().map(|(i, c)| json!({"index": i, "pattern": c.pattern, "track": c.track, "bar": c.start / bar, "bars": c.len as f32 / bar as f32})).collect::<Vec<_>>(),
        "song_bars": p.song_length_ticks() / bar,
    })
}

/// Run one command against a project.
pub fn run(p: &mut Project, id: &str, params: &Value) -> Result<Value, String> {
    let empty = json!({});
    let a = if params.is_null() { &empty } else { params };
    let bar = STEPS_PER_BAR * STEP_TICKS;
    match id {
        "project.new" => {
            *p = match a.get("template").and_then(Value::as_str).unwrap_or("empty") {
                "demo" => Project::demo(),
                "empty" => Project::empty(),
                t => return Err(format!("unknown template `{t}`")),
            };
            Ok(info(p))
        }
        "project.info" => Ok(info(p)),
        "project.rename" => {
            p.name = a.get("name").and_then(Value::as_str).ok_or("missing `name`")?.to_string();
            Ok(json!({"name": p.name}))
        }
        "project.set_tempo" => {
            p.tempo = clamp_tempo(num(a, "bpm")? as f32);
            Ok(json!({"tempo": p.tempo}))
        }
        "project.set_swing" => {
            p.swing = snap_swing(num(a, "amount")? as f32);
            Ok(json!({"swing": p.swing}))
        }
        "channel.add" => {
            let k = kind_of(a.get("kind").and_then(Value::as_str).ok_or("missing `kind`")?)?;
            let i = p.add_channel(k);
            Ok(json!({"channel": i, "name": p.channels[i].name, "insert": p.channels[i].insert}))
        }
        "channel.remove" => {
            let i = idx(a, "channel", p.channels.len(), "channel")?;
            if p.channels.len() == 1 {
                return Err("a project keeps at least one channel".into());
            }
            p.remove_channel(i);
            Ok(json!({"channels": p.channels.len()}))
        }
        "channel.set" => {
            let i = idx(a, "channel", p.channels.len(), "channel")?;
            let c = &mut p.channels[i];
            if let Some(n) = a.get("name").and_then(Value::as_str) {
                c.name = n.to_string();
            }
            if let Some(v) = opt_num(a, "volume") {
                c.volume = (v as f32).clamp(0.0, 1.0);
            }
            if let Some(v) = opt_num(a, "pan") {
                c.pan = (v as f32).clamp(-1.0, 1.0);
            }
            if let Some(v) = opt_num(a, "pitch") {
                c.pitch = (v as f32).clamp(-24.0, 24.0);
            }
            if let Some(v) = a.get("mute").and_then(Value::as_bool) {
                c.mute = v;
            }
            if let Some(v) = opt_num(a, "insert") {
                c.insert = (v as usize).min(MIXER_INSERTS);
            }
            Ok(json!({"channel": i, "name": c.name, "volume": c.volume, "pan": c.pan, "pitch": c.pitch, "mute": c.mute, "insert": c.insert}))
        }
        "pattern.add" => {
            let i = p.add_pattern();
            if let Some(n) = a.get("name").and_then(Value::as_str) {
                p.patterns[i].name = n.to_string();
            }
            Ok(json!({"pattern": i, "name": p.patterns[i].name}))
        }
        "pattern.set_steps" => {
            let i = idx(a, "pattern", p.patterns.len(), "pattern")?;
            let n = num(a, "count")? as usize;
            if !matches!(n, 16 | 32 | 64) {
                return Err("count must be 16, 32 or 64".into());
            }
            let nch = p.channels.len();
            p.patterns[i].steps = n;
            p.patterns[i].ensure_channels(nch);
            Ok(json!({"pattern": i, "steps": n}))
        }
        "step.set" | "step.fill" => {
            let pi = idx(a, "pattern", p.patterns.len(), "pattern")?;
            let ci = idx(a, "channel", p.channels.len(), "channel")?;
            let nch = p.channels.len();
            let pat = &mut p.patterns[pi];
            pat.ensure_channels(nch);
            let vel = opt_num(a, "velocity").unwrap_or(0.8).clamp(0.0, 1.0) as f32;
            let steps = &mut pat.data[ci].steps;
            if id == "step.set" {
                let s = idx(a, "step", steps.len(), "step")?;
                steps[s] = vel;
            } else {
                let every = (num(a, "every")? as usize).max(1);
                let off = opt_num(a, "offset").unwrap_or(0.0) as usize;
                for (s, v) in steps.iter_mut().enumerate() {
                    *v = if s >= off && (s - off) % every == 0 { vel } else { 0.0 };
                }
            }
            let lit: Vec<usize> = steps.iter().enumerate().filter(|(_, v)| **v > 0.0).map(|(s, _)| s).collect();
            Ok(json!({"pattern": pi, "channel": ci, "lit_steps": lit}))
        }
        "note.add" => {
            let pi = idx(a, "pattern", p.patterns.len(), "pattern")?;
            let ci = idx(a, "channel", p.channels.len(), "channel")?;
            let key = num(a, "key")?;
            if !(0.0..=127.0).contains(&key) {
                return Err("key must be 0..127".into());
            }
            let start = (num(a, "start_step")?.max(0.0) * STEP_TICKS as f64) as u32;
            let len = ((opt_num(a, "length_steps").unwrap_or(1.0).max(0.5)) * STEP_TICKS as f64) as u32;
            let vel = opt_num(a, "velocity").unwrap_or(0.8).clamp(0.05, 1.0) as f32;
            let nch = p.channels.len();
            let pat = &mut p.patterns[pi];
            pat.ensure_channels(nch);
            pat.data[ci].notes.push(Note { key: key as u8, start, len, vel });
            Ok(json!({"pattern": pi, "channel": ci, "notes": pat.data[ci].notes.len(), "pattern_bars": pat.length_ticks() / bar}))
        }
        "note.clear" => {
            let pi = idx(a, "pattern", p.patterns.len(), "pattern")?;
            let ci = idx(a, "channel", p.channels.len(), "channel")?;
            let d = &mut p.patterns[pi].data[ci];
            d.notes.clear();
            d.steps.iter_mut().for_each(|s| *s = 0.0);
            Ok(json!({"pattern": pi, "channel": ci}))
        }
        "clip.add" => {
            let pi = idx(a, "pattern", p.patterns.len(), "pattern")?;
            let track = idx(a, "track", PLAYLIST_TRACKS, "track")?;
            let start = (num(a, "bar")?.max(0.0) as u32) * bar;
            let len = match opt_num(a, "bars") {
                Some(b) => ((b.max(0.25)) * bar as f64) as u32,
                None => p.patterns[pi].length_ticks(),
            };
            p.clips.push(Clip { track, pattern: pi, start, len });
            Ok(json!({"clip": p.clips.len() - 1, "song_bars": p.song_length_ticks() / bar}))
        }
        "clip.remove" => {
            let i = idx(a, "index", p.clips.len(), "clip")?;
            p.clips.remove(i);
            Ok(json!({"clips": p.clips.len()}))
        }
        "mixer.set" => {
            let i = idx(a, "insert", MIXER_TRACKS, "mixer track")?;
            let m = &mut p.mixer[i];
            for (k, slot) in [("volume", &mut m.volume), ("lowpass", &mut m.lowpass), ("delay", &mut m.delay), ("reverb", &mut m.reverb)] {
                if let Some(v) = opt_num(a, k) {
                    *slot = (v as f32).clamp(0.0, 1.0);
                }
            }
            if let Some(v) = opt_num(a, "pan") {
                m.pan = (v as f32).clamp(-1.0, 1.0);
            }
            if let Some(v) = a.get("mute").and_then(Value::as_bool) {
                m.mute = v;
            }
            if let Some(v) = a.get("solo").and_then(Value::as_bool) {
                m.solo = v;
            }
            Ok(json!({"insert": i, "name": m.name, "volume_db": fader_db(m.volume), "pan": m.pan, "mute": m.mute, "solo": m.solo}))
        }
        "mixer.load_fx" => {
            let i = idx(a, "insert", MIXER_TRACKS, "mixer track")?;
            let s = idx(a, "slot", FX_SLOTS, "slot")?;
            let fx = match a.get("fx").and_then(Value::as_str).unwrap_or("").to_lowercase().as_str() {
                "filter" | "lowpass" => Some(FxKind::Filter),
                "delay" => Some(FxKind::Delay),
                "reverb" => Some(FxKind::Reverb),
                "limiter" => Some(FxKind::Limiter),
                "none" | "" => None,
                f => return Err(format!("unknown effect `{f}`")),
            };
            p.mixer[i].slots[s] = fx;
            Ok(json!({"insert": i, "slot": s, "fx": fx.map(|f| f.label())}))
        }
        "song.export_wav" => {
            let path = a.get("path").and_then(Value::as_str).ok_or("missing `path`")?;
            if p.clips.is_empty() {
                return Err("the playlist is empty: add clips first".into());
            }
            let info = crate::engine::export_wav(p, std::path::Path::new(path)).map_err(|e| e.to_string())?;
            Ok(json!({"path": path, "seconds": info.seconds, "measured_bpm": info.measured_bpm}))
        }
        _ => Err(format!("unknown command `{id}` (list them with `list`)")),
    }
}
