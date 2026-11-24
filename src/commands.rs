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
    CommandSpec { id: "project.set_tempo", params: "{bpm}", help: "Set the tempo (clamped to 20..400 BPM)." },
    CommandSpec { id: "project.set_swing", params: "{amount: 0..1}", help: "Set the main swing (snapped to whole percent)." },
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
        .find(|k| format!("{k:?}").to_lowercase() == s || k.label().to_lowercase().replace([' ', '-', '_'], "") == s)
        .ok_or_else(|| format!("unknown instrument `{s}`"))
}

pub fn info(p: &Project) -> Value {
    let bar = STEPS_PER_BAR * STEP_TICKS;
    json!({
        "name": p.name, "tempo": p.tempo, "swing": p.swing,
        "channels": p.channels.iter().enumerate().map(|(i, c)| json!({"index": i, "name": c.name, "kind": format!("{:?}", c.kind), "volume": c.volume, "pan": c.pan, "mute": c.mute, "insert": c.insert})).collect::<Vec<_>>(),
        "patterns": p.patterns.iter().enumerate().map(|(i, pt)| json!({
            "index": i, "name": pt.name, "steps": pt.steps, "bars": pt.length_ticks() as f32 / bar as f32,
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
            if p.patterns.len() >= MAX_PATTERNS {
                return Err(format!("a project holds at most {MAX_PATTERNS} patterns"));
            }
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
                    *v = if s >= off && (s - off).is_multiple_of(every) { vel } else { 0.0 };
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
            let start = (num(a, "start_step")?.max(0.0) * STEP_TICKS as f64).floor();
            let len = ((opt_num(a, "length_steps").unwrap_or(1.0).max(0.5)) * STEP_TICKS as f64).floor();
            if start + len > MAX_TICKS as f64 {
                return Err(format!("the note must end before bar {MAX_SONG_BARS}"));
            }
            let (start, len) = (start as u32, len as u32);
            let vel = opt_num(a, "velocity").unwrap_or(0.8).clamp(0.05, 1.0) as f32;
            let nch = p.channels.len();
            let pat = &mut p.patterns[pi];
            pat.ensure_channels(nch);
            pat.data[ci].notes.push(Note { key: key as u8, start, len, vel });
            Ok(json!({"pattern": pi, "channel": ci, "notes": pat.data[ci].notes.len(), "pattern_bars": pat.length_ticks() as f32 / bar as f32}))
        }
        "note.clear" => {
            let pi = idx(a, "pattern", p.patterns.len(), "pattern")?;
            let ci = idx(a, "channel", p.channels.len(), "channel")?;
            let nch = p.channels.len();
            p.patterns[pi].ensure_channels(nch);
            let d = &mut p.patterns[pi].data[ci];
            d.notes.clear();
            d.steps.iter_mut().for_each(|s| *s = 0.0);
            Ok(json!({"pattern": pi, "channel": ci}))
        }
        "clip.add" => {
            let pi = idx(a, "pattern", p.patterns.len(), "pattern")?;
            let track = idx(a, "track", PLAYLIST_TRACKS, "track")?;
            let start = num(a, "bar")?.max(0.0).floor() * bar as f64;
            let len = match opt_num(a, "bars") {
                Some(b) => ((b.max(0.25)) * bar as f64).floor(),
                None => p.patterns[pi].length_ticks() as f64,
            };
            if start + len > MAX_TICKS as f64 {
                return Err(format!("the playlist ends at bar {MAX_SONG_BARS}"));
            }
            let (start, len) = (start as u32, len as u32);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run_ok(p: &mut Project, id: &str, params: Value) -> Value {
        run(p, id, &params).unwrap_or_else(|e| panic!("{id} {params} failed: {e}"))
    }

    fn run_err(p: &mut Project, id: &str, params: Value) -> String {
        match run(p, id, &params) {
            Ok(v) => panic!("{id} {params} should have failed, returned {v}"),
            Err(e) => e,
        }
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("beatcraft-test-{}-{name}", std::process::id()))
    }

    #[test]
    fn only_export_takes_a_path() {
        for c in COMMANDS.iter().filter(|c| c.id != "song.export_wav") {
            assert!(!c.params.contains("path"), "{} takes a path: add it to the MCP file-access check", c.id);
        }
    }

    #[test]
    fn every_listed_command_is_known() {
        for c in COMMANDS {
            let mut p = Project::demo();
            if let Err(e) = run(&mut p, c.id, &json!({})) {
                assert!(!e.contains("unknown command"), "{}: {e}", c.id);
            }
        }
        let e = run_err(&mut Project::demo(), "no.such.command", json!({}));
        assert!(e.contains("unknown command"), "{e}");
    }

    #[test]
    fn list_json_describes_every_command() {
        let v = list_json();
        let list = v.as_array().unwrap();
        assert_eq!(list.len(), COMMANDS.len());
        for c in list {
            for k in ["id", "params", "help"] {
                assert!(c[k].as_str().is_some_and(|s| !s.is_empty()), "{c}");
            }
        }
    }

    #[test]
    fn project_level_commands() {
        let mut p = Project::empty();
        run_ok(&mut p, "project.set_tempo", json!({"bpm": 5}));
        assert_eq!(p.tempo, TEMPO_MIN);
        run_ok(&mut p, "project.set_tempo", json!({"bpm": 9999}));
        assert_eq!(p.tempo, TEMPO_MAX);
        assert!(run_err(&mut p, "project.set_tempo", json!({})).contains("bpm"));
        run_ok(&mut p, "project.set_swing", json!({"amount": 0.126}));
        assert_eq!(p.swing, 0.13);
        run_ok(&mut p, "project.rename", json!({"name": "Night"}));
        assert_eq!(p.name, "Night");
        run_ok(&mut p, "project.new", json!({"template": "demo"}));
        assert!(!p.clips.is_empty());
        assert!(run_err(&mut p, "project.new", json!({"template": "weird"})).contains("unknown template"));
    }

    #[test]
    fn channel_commands() {
        let mut p = Project::empty();
        for name in ["bass", "Sub bass", "open-hat", "OpenHat", "KEYS"] {
            run_ok(&mut p, "channel.add", json!({"kind": name}));
        }
        for k in InstKind::ALL {
            for name in [format!("{k:?}"), k.label().to_string()] {
                let r = run_ok(&mut p, "channel.add", json!({"kind": name}));
                assert_eq!(p.channels[r["channel"].as_u64().unwrap() as usize].kind, k, "{name}");
            }
        }
        assert!(run_err(&mut p, "channel.add", json!({"kind": "banjo"})).contains("unknown instrument"));
        assert!(run_err(&mut p, "channel.remove", json!({"channel": 99})).contains("doesn't exist"));
        run_ok(&mut p, "channel.set", json!({"channel": 0, "volume": 5, "pan": -3, "pitch": 99, "insert": 999, "name": "Boom"}));
        let c = &p.channels[0];
        assert_eq!((c.volume, c.pan, c.pitch, c.insert), (1.0, -1.0, 24.0, MIXER_INSERTS));
        assert_eq!(c.name, "Boom");
        while p.channels.len() > 1 {
            run_ok(&mut p, "channel.remove", json!({"channel": 0}));
        }
        assert!(run_err(&mut p, "channel.remove", json!({"channel": 0})).contains("at least one"));
    }

    #[test]
    fn pattern_commands() {
        let mut p = Project::empty();
        let r = run_ok(&mut p, "pattern.add", json!({"name": "Verse"}));
        assert_eq!(r["pattern"], json!(1));
        assert_eq!(p.patterns[1].name, "Verse");
        for bad in [0, 8, 48, 128] {
            run_err(&mut p, "pattern.set_steps", json!({"pattern": 0, "count": bad}));
        }
        for n in [32, 64, 16] {
            run_ok(&mut p, "pattern.set_steps", json!({"pattern": 0, "count": n}));
            assert!(p.patterns[0].data.iter().all(|d| d.steps.len() == n));
        }
    }

    #[test]
    fn b3_pattern_add_at_the_limit_errors_and_keeps_names() {
        let mut p = Project::empty();
        while p.patterns.len() < MAX_PATTERNS {
            p.add_pattern();
        }
        let before: Vec<String> = p.patterns.iter().map(|x| x.name.clone()).collect();
        let err = run_err(&mut p, "pattern.add", json!({"name": "Overwrite"}));
        assert!(err.contains("at most"), "{err}");
        let after: Vec<String> = p.patterns.iter().map(|x| x.name.clone()).collect();
        assert_eq!(before, after);
    }

    #[test]
    fn step_commands() {
        let mut p = Project::empty();
        run_ok(&mut p, "step.set", json!({"pattern": 0, "channel": 0, "step": 3}));
        assert!((p.patterns[0].data[0].steps[3] - 0.8).abs() < 1e-6);
        let r = run_ok(&mut p, "step.set", json!({"pattern": 0, "channel": 0, "step": 3, "velocity": 0}));
        assert_eq!(r["lit_steps"], json!([]));
        assert!(run_err(&mut p, "step.set", json!({"pattern": 0, "channel": 0, "step": 16})).contains("doesn't exist"));
        let r = run_ok(&mut p, "step.fill", json!({"pattern": 0, "channel": 1, "every": 4, "offset": 2}));
        assert_eq!(r["lit_steps"], json!([2, 6, 10, 14]));
    }

    #[test]
    fn b1_note_add_far_past_the_end_is_an_error() {
        let mut p = Project::empty();
        let err = run_err(&mut p, "note.add", json!({"pattern": 0, "channel": 0, "key": 60, "start_step": 1e12}));
        assert!(err.contains("bar 1000"), "{err}");
        assert!(p.patterns[0].data[0].notes.is_empty());
    }

    #[test]
    fn note_commands() {
        let mut p = Project::empty();
        assert!(run_err(&mut p, "note.add", json!({"pattern": 0, "channel": 0, "key": 128, "start_step": 0})).contains("key"));
        run_ok(&mut p, "note.add", json!({"pattern": 0, "channel": 0, "key": 60, "start_step": 0}));
        let n = p.patterns[0].data[0].notes[0];
        assert_eq!((n.len, n.vel), (24, 0.8));
        let r = run_ok(&mut p, "note.add", json!({"pattern": 0, "channel": 0, "key": 60, "start_step": 16, "length_steps": 16}));
        assert_eq!(r["pattern_bars"], json!(2.0));
        run_ok(&mut p, "step.set", json!({"pattern": 0, "channel": 0, "step": 1}));
        run_ok(&mut p, "note.clear", json!({"pattern": 0, "channel": 0}));
        let d = &p.patterns[0].data[0];
        assert!(d.notes.is_empty() && d.steps.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn info_reports_fractional_pattern_bars() {
        let mut p = Project::empty();
        let r = run_ok(&mut p, "note.add", json!({"pattern": 0, "channel": 0, "key": 60, "start_step": 16}));
        assert_eq!(r["pattern_bars"], json!(1.25));
        assert_eq!(info(&p)["patterns"][0]["bars"], json!(1.25));
    }

    #[test]
    fn note_clear_survives_short_pattern_data() {
        let mut p = Project::empty();
        p.channels.push(Channel::new(InstKind::Tom, 5));
        run_ok(&mut p, "note.clear", json!({"pattern": 0, "channel": 4}));
    }

    #[test]
    fn b2_clip_add_far_past_the_end_is_an_error() {
        let mut p = Project::empty();
        let err = run_err(&mut p, "clip.add", json!({"pattern": 0, "track": 0, "bar": 1e9}));
        assert!(err.contains("bar 1000"), "{err}");
        assert!(p.clips.is_empty());
    }

    #[test]
    fn clip_commands() {
        let mut p = Project::empty();
        let r = run_ok(&mut p, "clip.add", json!({"pattern": 0, "track": 0, "bar": 2}));
        assert_eq!((p.clips[0].start, p.clips[0].len), (768, 384));
        assert_eq!(r["song_bars"], json!(3));
        run_ok(&mut p, "clip.add", json!({"pattern": 0, "track": 1, "bar": 0, "bars": 0.1}));
        assert_eq!(p.clips[1].len, 96);
        assert!(run_err(&mut p, "clip.add", json!({"pattern": 0, "track": 500, "bar": 0})).contains("doesn't exist"));
        run_ok(&mut p, "clip.remove", json!({"index": 0}));
        assert_eq!(p.clips.len(), 1);
        run_err(&mut p, "clip.remove", json!({"index": 5}));
    }

    #[test]
    fn mixer_commands() {
        let mut p = Project::empty();
        let r = run_ok(&mut p, "mixer.set", json!({"insert": 1, "volume": 0.8, "pan": 4, "reverb": 7}));
        assert!(r["volume_db"].as_f64().unwrap().abs() < 0.05);
        assert_eq!((p.mixer[1].pan, p.mixer[1].reverb), (1.0, 1.0));
        for (name, kind) in [("filter", FxKind::Filter), ("delay", FxKind::Delay), ("reverb", FxKind::Reverb), ("limiter", FxKind::Limiter)] {
            run_ok(&mut p, "mixer.load_fx", json!({"insert": 2, "slot": 3, "fx": name}));
            assert_eq!(p.mixer[2].slots[3], Some(kind));
        }
        run_ok(&mut p, "mixer.load_fx", json!({"insert": 2, "slot": 3, "fx": "none"}));
        assert_eq!(p.mixer[2].slots[3], None);
        assert!(run_err(&mut p, "mixer.load_fx", json!({"insert": 2, "slot": 3, "fx": "phaser"})).contains("unknown effect"));
        run_err(&mut p, "mixer.load_fx", json!({"insert": 2, "slot": 10, "fx": "delay"}));
    }

    #[test]
    fn export_wav_command() {
        let mut p = Project::empty();
        let path = temp_path("export.wav");
        let path_str = path.to_str().unwrap();
        assert!(run_err(&mut p, "song.export_wav", json!({"path": path_str})).contains("empty"));
        p.patterns[0].data[0].steps[0] = 1.0;
        run_ok(&mut p, "clip.add", json!({"pattern": 0, "track": 0, "bar": 0}));
        let r = run_ok(&mut p, "song.export_wav", json!({"path": path_str}));
        let bytes = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        let u16_at = |o: usize| u16::from_le_bytes([bytes[o], bytes[o + 1]]);
        let u32_at = |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!((u16_at(20), u16_at(22), u32_at(24), u16_at(34)), (1, 2, 44100, 16));
        assert_eq!(u32_at(40) as usize, bytes.len() - 44);
        assert_eq!(r["measured_bpm"], json!(140.0));
    }
}
