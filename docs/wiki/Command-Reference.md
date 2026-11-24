# Command Reference

BeatCraft has 19 commands. The `beatcraft-cli run` program, the MCP `execute` and `batch` tools
and (for `song.export_wav`) `beatcraft-cli render` all run the same commands. Parameters are a JSON
object; results are a JSON object.

## Conventions

- **Indexes start at 0**: channels, patterns, steps, tracks, clips and mixer tracks. A fractional
  index is cut down to a whole number (`1.9` is `1`).
- **Steps are 1/16 notes**, 16 steps per bar, 4 per beat. A bar is 4 beats.
- **Key numbers** are MIDI-style, 0 to 127, and **key 60 is C5** (so 61 is C#5 and 36 is C3).
- **Bars count from 0** in parameters (`bar: 0` is the first bar).
- **Limits**: a song can be at most 1000 bars long, a project holds at most 999 patterns, the
  playlist has 500 tracks (0 to 499), a pattern has 16, 32 or 64 steps.
- **Mixer tracks**: 0 is Master, 1 to 125 are inserts, 126 is Current. Every track has 10 effect
  slots (0 to 9). A channel can be routed to 0 to 125.
- **`velocity: 0` clears a step.** Step velocity runs 0 to 1; any velocity above 0 lights the step.
- **Effect amounts need the effect loaded.** The `lowpass`, `delay` and `reverb` amounts of
  `mixer.set` only sound while the matching effect (filter, delay, reverb) is loaded in one of the
  track's slots with `mixer.load_fx`. Master starts with the limiter in slot 0. Insert tracks start
  with empty slots.
- **Numbers must be JSON numbers.** A required number that is missing or is not a number gives
  `missing number `name``. An optional number or boolean of the wrong type is ignored without an
  error.
- **Out-of-range values** are either clamped or rejected. Each command below says which, per
  parameter.
- **A command that returns an error changes nothing**, apart from the earlier steps of a `batch`
  (see [MCP Setup](MCP-Setup)).

### Errors shared by many commands

| Message | When |
|---|---|
| `missing number `x`` | A required numeric parameter `x` is missing or not a number |
| `missing `x`` | A required text parameter `x` is missing or not a string |
| `channel 9 doesn't exist (there are 5)` | An index is out of range or negative. The same form is used for `pattern`, `step`, `track`, `clip`, `mixer track` and `slot` |
| `unknown command `x` (list them with `list`)` | The command id does not exist |

## The commands

### project.new

Starts a new project, replacing the one in memory (or, through `run`, the project in the file).

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `template` | `"empty"` or `"demo"` | no | `"empty"` | `empty` is four drum channels (Kick, Clap, Hat, Snare), one 16-step pattern, no clips. `demo` is the built-in 16-bar song |

Result: the same summary object as `project.info`. Error: ``unknown template `x` ``.

```sh
beatcraft-cli run beat.json project.new '{"template": "demo"}'
```

```json
{
  "channels": [
    {
      "index": 0,
      "insert": 1,
      "kind": "Kick",
      "mute": false,
      "name": "Kick",
      "pan": 0.0,
      "volume": 0.800000011920929
    },
    {
      "index": 1,
      "insert": 2,
…
```

### project.info

Summary of the project. No parameters. Result fields: `name`, `tempo`, `swing`, `channels` (index,
name, kind, volume, pan, mute, insert), `patterns` (index, name, steps, `bars`, `lit_steps`,
`notes`), `clips` (index, pattern, track, `bar`, `bars`) and `song_bars`. `bars` of a pattern is
its length in bars (it can be fractional, for example 1.25); a clip's `bar` is its first bar
(whole number, counted from 0) and its `bars` its length. Over MCP the same summary comes from
`inspect_project`.

```sh
beatcraft-cli run beat.json project.info
```

```json
{
  "channels": [
    {
      "index": 0,
      "insert": 1,
      "kind": "Kick",
      "mute": false,
      "name": "Kick",
      "pan": 0.0,
      "volume": 0.800000011920929
    },
    {
      "index": 1,
      "insert": 2,
…
```

### project.rename

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `name` | string | yes | | Any text. The app uses it for the saved file names |

Result: `{"name": …}`. Error: ``missing `name` ``.

```sh
beatcraft-cli run beat.json project.rename '{"name": "Night Drive"}'
```

```json
{
  "name": "Night Drive"
}
```

### project.set_tempo

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `bpm` | number | yes | | Clamped to 20 to 400. Fractions are kept |

Result: `{"tempo": …}`. Error: ``missing number `bpm` ``.

```sh
beatcraft-cli run beat.json project.set_tempo '{"bpm": 128}'
```

```json
{
  "tempo": 128.0
}
```

### project.set_swing

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `amount` | number | yes | | Clamped to 0 to 1, then snapped to the nearest whole percent. 0 is straight; 1 delays every odd 1/16 step by half a step |

Result: `{"swing": …}`, the snapped value (0.126 becomes 0.13). Error:
``missing number `amount` ``.

```sh
beatcraft-cli run beat.json project.set_swing '{"amount": 0.2}'
```

```json
{
  "swing": 0.20000000298023224
}
```

### channel.add

Adds an instrument channel at the end of the rack. It is routed to the next free mixer insert
(channel number + 1, never above 125) and every pattern gets an empty row for it. A second channel
of the same instrument is named "Kick 2", "Kick 3" and so on.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `kind` | string | yes | | `Kick`, `Snare`, `Clap`, `HiHat`, `OpenHat`, `Tom`, `Bass`, `Lead`, `Pad`, `Pluck` or `Keys`. Case, spaces, hyphens and underscores are ignored, so `open-hat` works. The display labels (`Hat`, `Open hat`, `Sub bass`, `Saw lead`, `Warm pad`, `E-keys`) work too |

Result: `{"channel": index, "name": …, "insert": …}`. Errors: ``missing `kind` ``,
``unknown instrument `x` `` (shown lower-cased with spaces and hyphens removed).

```sh
beatcraft-cli run beat.json channel.add '{"kind": "E-keys"}'
```

```json
{
  "channel": 4,
  "insert": 5,
  "name": "E-keys"
}
```

### channel.remove

Removes a channel and its steps and notes in every pattern. Later channels move down one index.
Their mixer routing does not change.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `channel` | number | yes | | Must exist |

Result: `{"channels": remaining count}`. Errors: the shared `channel N doesn't exist` form, and
`a project keeps at least one channel` when only one is left.

```sh
beatcraft-cli run beat.json channel.remove '{"channel": 3}'
```

```json
{
  "channels": 3
}
```

### channel.set

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `channel` | number | yes | | Must exist |
| `name` | string | no | unchanged | |
| `volume` | number | no | unchanged | Clamped to 0 to 1 (a new channel starts at 0.8) |
| `pan` | number | no | unchanged | Clamped to -1 (left) to 1 (right) |
| `pitch` | number | no | unchanged | Semitones, clamped to -24 to 24. It transposes the melodic instruments and the tom; the other drum sounds ignore it |
| `mute` | boolean | no | unchanged | |
| `insert` | number | no | unchanged | Mixer track, clamped to 0 to 125 (0 is Master; a negative number becomes 0) |

Result: `{"channel", "name", "volume", "pan", "pitch", "mute", "insert"}` with the stored values.

```sh
beatcraft-cli run beat.json channel.set '{"channel": 0, "name": "Big Kick", "volume": 0.9, "pan": -0.2}'
```

```json
{
  "channel": 0,
  "insert": 1,
  "mute": false,
  "name": "Big Kick",
  "pan": -0.20000000298023224,
  "pitch": 0.0,
  "volume": 0.8999999761581421
}
```

### pattern.add

Adds an empty 16-step pattern at the end. It is named "Pattern N" unless you give a name.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `name` | string | no | `Pattern N` | |

Result: `{"pattern": index, "name": …}`. Error: `a project holds at most 999 patterns`.

```sh
beatcraft-cli run beat.json pattern.add '{"name": "Verse"}'
```

```json
{
  "name": "Verse",
  "pattern": 1
}
```

### pattern.set_steps

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `pattern` | number | yes | | Must exist |
| `count` | number | yes | | Exactly 16, 32 or 64; anything else is an error. Making a pattern shorter drops the lit steps past the new end; notes are kept |

Result: `{"pattern", "steps"}`. Error: `count must be 16, 32 or 64`.

```sh
beatcraft-cli run beat.json pattern.set_steps '{"pattern": 0, "count": 32}'
```

```json
{
  "pattern": 0,
  "steps": 32
}
```

### step.set

Lights or clears one step of a channel.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `pattern` | number | yes | | Must exist |
| `channel` | number | yes | | Must exist |
| `step` | number | yes | | Must be inside the pattern (0 to steps - 1); otherwise an error |
| `velocity` | number | no | 0.8 | Clamped to 0 to 1. 0 clears the step |

Result: `{"pattern", "channel", "lit_steps": [every lit step of that channel]}`. Steps are played
together with the channel's notes, but the app's channel rack hides the step buttons of a channel
that has notes.

```sh
beatcraft-cli run beat.json step.set '{"pattern": 0, "channel": 0, "step": 4, "velocity": 0.9}'
```

```json
{
  "channel": 0,
  "lit_steps": [
    4
  ],
  "pattern": 0
}
```

### step.fill

Lights every Nth step and clears all the others of that channel in the pattern.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `pattern` | number | yes | | Must exist |
| `channel` | number | yes | | Must exist |
| `every` | number | yes | | Raised to at least 1 |
| `offset` | number | no | 0 | First lit step; negative becomes 0. Past the end of the pattern nothing is lit (and everything is cleared) |
| `velocity` | number | no | 0.8 | Clamped to 0 to 1. With 0 the result clears the whole channel |

Result: same as `step.set`.

```sh
beatcraft-cli run beat.json step.fill '{"pattern": 0, "channel": 2, "every": 4, "offset": 2}'
```

```json
{
  "channel": 2,
  "lit_steps": [
    2,
    6,
    10,
    14
  ],
  "pattern": 0
}
```

### note.add

Adds a piano-roll note. Notes are kept in order of creation, and the pattern grows to the next
whole beat past the last note.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `pattern` | number | yes | | Must exist |
| `channel` | number | yes | | Must exist |
| `key` | number | yes | | 0 to 127; outside that range is an error (60 = C5) |
| `start_step` | number | yes | | In 1/16 steps. Negative is clamped to 0. Fractions are allowed (a step is 24 ticks) |
| `length_steps` | number | no | 1 | In 1/16 steps, raised to at least 0.5 |
| `velocity` | number | no | 0.8 | Clamped to 0.05 to 1 |

Result: `{"pattern", "channel", "notes": notes in that channel, "pattern_bars": pattern length in bars}`.
Error: `the note must end before bar 1000` when start plus length passes bar 1000.

```sh
beatcraft-cli run beat.json note.add '{"pattern": 0, "channel": 4, "key": 36, "start_step": 2, "length_steps": 2}'
```

```json
{
  "channel": 4,
  "notes": 1,
  "pattern": 0,
  "pattern_bars": 1.0
}
```

### note.clear

Removes all notes and clears all steps of a channel in a pattern.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `pattern` | number | yes | | Must exist |
| `channel` | number | yes | | Must exist |

Result: `{"pattern", "channel"}`.

```sh
beatcraft-cli run beat.json note.clear '{"pattern": 0, "channel": 4}'
```

```json
{
  "channel": 4,
  "pattern": 0
}
```

### clip.add

Places a pattern on the playlist. Clips may overlap, and the same pattern can be placed many times.
In song mode every clip plays, whatever its track.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `pattern` | number | yes | | Must exist |
| `track` | number | yes | | 0 to 499; otherwise an error |
| `bar` | number | yes | | First bar, counted from 0. Negative is clamped to 0, fractions are cut down |
| `bars` | number | no | the pattern's length | Length in bars, raised to at least 0.25 |

Result: `{"clip": index, "song_bars": total length of the song in bars}`. Error:
`the playlist ends at bar 1000` when the clip would end past bar 1000.

```sh
beatcraft-cli run beat.json clip.add '{"pattern": 0, "track": 0, "bar": 0, "bars": 4}'
```

```json
{
  "clip": 0,
  "song_bars": 4
}
```

### clip.remove

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `index` | number | yes | | Position in the clips list (see `project.info`); later clips move down one |

Result: `{"clips": remaining count}`. Error: `clip N doesn't exist (there are M)`.

```sh
beatcraft-cli run beat.json clip.remove '{"index": 0}'
```

```json
{
  "clips": 0
}
```

### mixer.set

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `insert` | number | yes | | 0 (Master) to 126 (Current); otherwise an error |
| `volume` | number | no | unchanged | Fader position, clamped to 0 to 1. 0.8 is 0 dB, 1 is about +5.8 dB |
| `pan` | number | no | unchanged | Clamped to -1 to 1 |
| `lowpass` | number | no | unchanged | Clamped to 0 to 1; 1 is open. Needs the low-pass filter loaded |
| `delay` | number | no | unchanged | Wet amount, clamped to 0 to 1. Needs the delay loaded |
| `reverb` | number | no | unchanged | Wet amount, clamped to 0 to 1. Needs the reverb loaded |
| `mute` | boolean | no | unchanged | |
| `solo` | boolean | no | unchanged | Solo on Master is stored but does nothing |

Result: `{"insert", "name", "volume_db", "pan", "mute", "solo"}`. `volume_db` is `null` when the
fader is at 0. Numbers are stored as 32-bit floats and printed as their nearest 64-bit value, so
`-0.3` can appear as `-0.30000001192092896`. The effect amounts are stored but not repeated in the result.

```sh
beatcraft-cli run beat.json mixer.set '{"insert": 1, "volume": 0.6, "pan": -0.3}'
```

```json
{
  "insert": 1,
  "mute": false,
  "name": "Kick",
  "pan": -0.30000001192092896,
  "solo": false,
  "volume_db": -7.496324062347412
}
```

### mixer.load_fx

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `insert` | number | yes | | 0 to 126 |
| `slot` | number | yes | | 0 to 9 |
| `fx` | string | no | empty slot | `filter` (or `lowpass`), `delay`, `reverb`, `limiter`, or `none`; case is ignored. `none`, an empty string, or leaving `fx` out empties the slot |

Result: `{"insert", "slot", "fx": label or null}`. The labels are "Low-pass filter", "Echo delay",
"Room reverb" and "Soft limiter". Error: ``unknown effect `x` ``. The limiter only has an effect
on Master (see [Sound Engine](Sound-Engine)).

```sh
beatcraft-cli run beat.json mixer.load_fx '{"insert": 1, "slot": 0, "fx": "delay"}'
```

```json
{
  "fx": "Echo delay",
  "insert": 1,
  "slot": 0
}
```

### song.export_wav

Renders the song to a 44.1 kHz, 16-bit stereo WAV file, playing the arrangement once from the
start and adding a 2-second tail. An existing file is overwritten.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `path` | string | yes | | Where to write the file. The folder must exist |

Result: `{"path", "seconds": length including the tail, "measured_bpm"}`. `measured_bpm` is the
project's tempo measured from the rendered length, rounded to a whole number, or `null` when it
falls outside 20 to 400 (the tempo range). Errors: `the playlist is empty: add clips first`, ``missing `path` ``, the
operating system's message if the file cannot be written (for example
`No such file or directory (os error 2)`) and `the song is too long for a WAV file`.

```sh
beatcraft-cli run beat.json song.export_wav '{"path": "beat.wav"}'
```

```json
{
  "measured_bpm": 140.0,
  "path": "beat.wav",
  "seconds": 3.7142856121063232
}
```

## The beatcraft-cli program

```
beatcraft-cli list                                   list every command and its parameters
beatcraft-cli run <project.json> <command> [json]    run one command (the file is created if missing)
beatcraft-cli info <project.json>                    print a project summary
beatcraft-cli render <project.json> <out.wav>        render the song to a WAV file
beatcraft-cli mcp [--automation-read-root DIR]... [--automation-write-root DIR]...
                                                     serve the commands as an MCP server on stdio,
                                                     optionally confined to folders
beatcraft-cli --version
```

- **`list`** prints a JSON array of `{"id", "params", "help"}` objects, one per command, in the
  order of this page. The ids are the 19 headings above.
- **`run`** loads the project file, runs one command, and saves the file again, pretty-printed,
  after every command that succeeds. If the file does not exist `run` starts from an empty
  project and creates the file. The JSON parameters are optional (`{}` is used) and must be one
  argument, so quote them. The result is printed as JSON. A command that fails prints an error
  and leaves the file as it was.
- **`info`** prints the `project.info` summary. It does not create files: a missing file is an
  error.
- **`render`** is `song.export_wav` for a project file. It does not create the project and does
  not change it: a missing file is an error.
- **`mcp`** serves the commands as an MCP server (see [MCP Setup](MCP-Setup)), optionally confined to folders with `--automation-read-root` and `--automation-write-root`.
- **`--version`** prints `beatcraft-cli 0.2.0` (the crate version).

Files are repaired when they load (see [Project File Format](Project-File-Format)). A file that is
not a BeatCraft project gives `not a BeatCraft project: …` with the reason from the parser.

Exit codes:

| Code | Meaning |
|---|---|
| 0 | Success (the result is on stdout) |
| 1 | The command, the parameters or the file failed. The message is on stderr after `error: ` |
| 2 | The arguments were not understood; the usage text is on stderr |

```sh
$ beatcraft-cli info nope.json
error: no such project file: nope.json
$ beatcraft-cli run beat.json project.rename '{bad'
error: bad JSON parameters: key must be a string at line 1 column 2
$ beatcraft-cli run beat.json no.such
error: unknown command `no.such` (list them with `list`)
```
