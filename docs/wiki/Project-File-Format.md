# Project File Format

A BeatCraft project is one JSON file, usually named `something.beatcraft.json`. The app, the CLI
and the MCP server all read and write the same format. There is **no version field yet**: the
format is whatever the current code writes, and it may change while BeatCraft is in alpha.

You can edit files by hand. Every field below must be present (a missing one gives
`not a BeatCraft project: missing field `tempo` …`); unknown extra fields are ignored. Values that
are out of range are repaired on load (see [What is repaired on load](#what-is-repaired-on-load)).

## Units

- **Time** inside patterns and on the playlist is counted in **ticks**: 96 ticks per quarter
  note, 24 ticks per 1/16 step, 384 ticks per bar (16 steps).
- Note and clip positions are whole numbers of ticks. A pattern note starts at `start` ticks from
  the start of its pattern; a clip starts at `start` ticks from the start of the song.
- The last allowed tick is just before bar 1000 (384000 ticks).
- **Volumes and pans** are knob positions, not decibels (see [Sound Engine](Sound-Engine)).

## Fields

### Project (the file's top level)

| Field | Type | Meaning |
|---|---|---|
| `name` | string | Project name |
| `tempo` | number | BPM, 20 to 400 |
| `swing` | number | 0 to 1, in whole percent (0.18 is 18%) |
| `master_volume` | number | Master knob, 0 to 1 (0.8 is 0 dB) |
| `master_pitch` | number | Semitones, -12 to 12, applied to every channel |
| `channels` | array of Channel | The instruments, in rack order |
| `patterns` | array of Pattern | Up to 999 |
| `clips` | array of Clip | The playlist |
| `track_names` | array of string | Exactly 500 playlist track names |
| `mixer` | array of Insert | Exactly 127: index 0 is Master, 1 to 125 inserts, 126 Current |

### Channel

| Field | Type | Meaning |
|---|---|---|
| `name` | string | Shown in the rack |
| `kind` | string | `Kick`, `Snare`, `Clap`, `HiHat`, `OpenHat`, `Tom`, `Bass`, `Lead`, `Pad`, `Pluck` or `Keys` |
| `volume` | number | 0 to 1 |
| `pan` | number | -1 (left) to 1 (right) |
| `pitch` | number | Semitones, -24 to 24 |
| `mute` | boolean | |
| `insert` | number | Mixer track the channel plays into, 0 to 125 |
| `color` | `[r, g, b]` | Three numbers 0 to 255 |
| `params` | SynthParams | Sound settings |

### SynthParams

| Field | Meaning |
|---|---|
| `attack` | Envelope attack time in seconds (melodic instruments) |
| `decay` | Envelope decay time in seconds |
| `sustain` | Envelope sustain level, 0 to 1 |
| `release` | Envelope release time in seconds |
| `cutoff` | Low-pass cutoff of the instrument, 0 to 1 |
| `reso` | Filter resonance, 0 to 1 |
| `tone` | Drum tone, or oscillator detune, 0 to 1 |

### Pattern

| Field | Type | Meaning |
|---|---|---|
| `name` | string | |
| `steps` | number | 16, 32 or 64 |
| `color` | `[r, g, b]` | |
| `data` | array of ChannelPattern | One entry per channel, in channel order |

### ChannelPattern

| Field | Type | Meaning |
|---|---|---|
| `steps` | array of number | One velocity per step, 0 to 1. 0 is off. The length equals the pattern's `steps` |
| `notes` | array of Note | Piano-roll notes |

### Note

| Field | Type | Meaning |
|---|---|---|
| `key` | number | 0 to 127 (60 = C5) |
| `start` | number | Ticks from the start of the pattern |
| `len` | number | Length in ticks |
| `vel` | number | Velocity, 0.05 to 1 |

### Clip

| Field | Type | Meaning |
|---|---|---|
| `track` | number | Playlist track, 0 to 499 |
| `pattern` | number | Index into `patterns` |
| `start` | number | Ticks from the start of the song |
| `len` | number | Length in ticks. If longer than the pattern, the pattern repeats; if shorter, it is cut |

### Insert (a mixer track)

| Field | Type | Meaning |
|---|---|---|
| `name` | string | |
| `volume` | number | Fader position, 0 to 1 (0.8 is 0 dB) |
| `pan` | number | -1 to 1 |
| `mute` | boolean | |
| `solo` | boolean | |
| `lowpass` | number | 0 to 1; 1 is open. Only used while a `Filter` is in a slot |
| `delay` | number | Wet amount 0 to 1. Only used while a `Delay` is in a slot |
| `reverb` | number | Wet amount 0 to 1. Only used while a `Reverb` is in a slot |
| `slots` | array of 10 | Each is `null` or `"Filter"`, `"Delay"`, `"Reverb"` or `"Limiter"` |

## A real excerpt

From the project built in [Your First Beat](Your-First-Beat):

```json
{
  "name": "Four on the Floor",
  "tempo": 120.0,
  "swing": 0.0,
  "master_volume": 0.8,
  "master_pitch": 0.0,
  "channels": [
    …
    {
      "name": "Sub bass",
      "kind": "Bass",
      "volume": 0.8,
      "pan": 0.0,
      "pitch": 0.0,
      "mute": false,
      "insert": 5,
      "color": [90, 170, 235],
      "params": {
        "attack": 0.003, "decay": 0.25, "sustain": 0.8,
        "release": 0.08, "cutoff": 0.35, "reso": 0.0, "tone": 0.5
      }
    }
  ],
  "patterns": [
    {
      "name": "Pattern 1",
      "steps": 16,
      "color": [96, 150, 200],
      "data": [
        {
          "steps": [0.8, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0],
          "notes": []
        },
        …
        {
          "steps": [0.0, …],
          "notes": [
            {"key": 36, "start": 48, "len": 48, "vel": 0.8},
            {"key": 36, "start": 144, "len": 48, "vel": 0.8},
            …
          ]
        }
      ]
    }
  ],
  "clips": [{"track": 0, "pattern": 0, "start": 0, "len": 3072}],
  "track_names": ["Track 1", "Track 2", …],
  "mixer": [
    {"name": "Master", "volume": 0.8, "pan": 0.0, "mute": false, "solo": false, "lowpass": 1.0,
     "delay": 0.0, "reverb": 0.0, "slots": ["Limiter", null, null, null, null, null, null, null, null, null]},
    …
    {"name": "Clap", "volume": 0.8, "pan": 0.0, "mute": false, "solo": false, "lowpass": 1.0,
     "delay": 0.0, "reverb": 0.25, "slots": ["Reverb", null, null, null, null, null, null, null, null, null]},
    …
  ]
}
```

(The real file is pretty-printed with one value per line and is longer; `…` marks parts left
out here.) The clip is 3072 ticks, 8 bars of 384.

## What is repaired on load

Every project is passed through `Project::sanitize` when it is opened by the app, the CLI or the
MCP server. It leaves a good project unchanged, and running it twice changes nothing more. The
rules, in order:

**Project**
- `tempo`: a value that is not a finite number becomes 140, then it is clamped to 20 to 400.
- `swing`: not finite becomes 0, then it is clamped to 0 to 1 and snapped to whole percent.
- `master_volume`: not finite becomes 0.8, then clamped to 0 to 1.
- `master_pitch`: not finite becomes 0, then clamped to -12 to 12.

**Mixer**
- `mixer` is cut or padded with fresh default tracks to exactly 127 entries. Each has exactly 10
  slots (padded with empty ones).
- `volume`, `lowpass`, `delay` and `reverb` are clamped to 0 to 1 and `pan` to -1 to 1. A value that
  is not finite is replaced by that track's default (volume 0.8, pan 0, lowpass 1, delay 0, reverb 0).

**Playlist**
- `track_names` is cut or padded (`Track 1`, `Track 2`, …) to exactly 500.

**Channels**
- An empty `channels` list gets one Kick.
- `insert` is limited to 125. `volume` is clamped to 0 to 1 (not finite becomes 0.8), `pan` to
  -1 to 1 (not finite becomes 0), `pitch` to -24 to 24 (not finite becomes 0).
- If any `params` value is not finite, all `params` of that channel are replaced by the
  instrument's defaults.

**Patterns**
- An empty `patterns` list gets one empty pattern; more than 999 are cut.
- `steps` becomes 16, 32 or 64: the smallest of those that is at least the stored number, or 64.
- `data` is cut to the number of channels and padded with empty rows, and every `steps` list is
  cut or padded with zeros to the pattern's length.
- Step velocities are clamped to 0 to 1 (not finite becomes 0).
- Notes: `key` is limited to 127; `vel` is clamped to 0.05 to 1 (not finite becomes 0.8); `start`
  is limited to the last tick before bar 1000; `len` is clamped to at least 1 tick and so that the
  note ends no later than bar 1000.

**Clips**
- A clip is dropped if its `pattern` does not exist, its `track` is 500 or more, or its `start` is
  at or past bar 1000.
- The remaining clips get `len` clamped to at least 1 tick and so that they end no later than
  bar 1000.

So anything past bar 1000 is clamped (notes and clip lengths) or dropped (clips that start there).
Values are not checked for names, colours or `params` ranges beyond the rules above.
