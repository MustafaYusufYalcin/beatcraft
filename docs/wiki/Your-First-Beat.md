# Your First Beat

This page builds one simple beat three ways: in the app, from the terminal and as a single MCP
call. All three make the same music.

The beat is a 120 BPM four-on-the-floor house groove, one bar long, played for 8 bars:

| Channel | Steps lit (1 to 16) | Notes |
|---|---|---|
| Kick | 1, 5, 9, 13 | One on every beat |
| Clap | 5, 13 | Beats 2 and 4, with a little reverb |
| Hat | 3, 7, 11, 15 | The off-beats, a bit quieter (velocity 0.6) |
| Sub bass | step 3, 7, 11, 15 | Two steps long: C3, C3, D#3, A#2 (keys 36, 36, 39, 34) |

A bar has 16 steps. The commands count steps from 0, so "step 1" in the app is `step` 0 in a
command, and the off-beat hat on app step 3 is step 2 in the commands.

## A. In the app

1. Start BeatCraft and choose FILE, New (basic drums). The channel rack lists Kick, Clap, Hat and
   Snare. The app starts in song mode, which is silent until there are clips, so press **L** (or
   click **PAT**) to loop the pattern while you write it.
2. Set the tempo to 120: drag the tempo display down about 40 pixels (each pixel is 0.5 BPM), or
   scroll over it, until it reads 120. Double-clicking it goes back to 140.
3. In the channel rack, click the Kick steps 1, 5, 9 and 13. Each click lights one step and plays
   the drum; right-click clears a step.
4. Click the Clap steps 5 and 13.
5. Click the Hat steps 3, 7, 11 and 15. To make them quieter, scroll down over each lit step; every
   scroll event lowers its velocity by 0.05.
6. Press **Space** to hear the drums loop, and **Space** again to pause.
7. Add the bass: ADD, Sub bass (or the `+ Add channel` button in the rack). The new channel is
   selected and the piano roll shows it. Find the row labelled **C3** on the keyboard at the left.
   Click in that row at step 3 and at step 7 to draw two notes (they are 2 steps long). Click three
   rows up at step 11 for D#3, and two rows down at step 15 for A#2. Drag a note to move it, drag
   its right edge to resize, right-click it to delete. Because this channel now has notes, the
   channel rack shows a small preview of them instead of step buttons.
8. Add reverb to the Clap. In the mixer at the bottom, click the strip for the Clap insert (it is
   named after its channel). On the right, open the first effect slot and choose **Room reverb**.
   The REVERB knob is greyed out until the effect is loaded; turn it up to about a quarter.
9. Place the pattern on the playlist. Click an empty cell on track 1 at bar 1: a one-bar clip of
   the current pattern appears. Drag its right edge until it ends at bar 9 (clips resize in
   quarter bars; the ruler counts bars from 1).
10. Press **L** to switch to song mode and **Space** to play all 8 bars.
11. Save with **Ctrl/Cmd+S** and choose FILE, Export song to WAV. Both files land in
    `~/Music/BeatCraft` (see [Getting Started](Getting-Started)). The project is called `Untitled`
    because the app cannot rename projects yet.

## B. From the terminal

You need `beatcraft-cli` on your `PATH` (see [Getting Started](Getting-Started)). Every `run`
command takes a project file, creates it if it does not exist, applies one command and saves the
file. Each prints a small JSON result.

Name the project and set the tempo:

```sh
beatcraft-cli run house.beatcraft.json project.rename '{"name": "Four on the Floor"}'
```

```json
{
  "name": "Four on the Floor"
}
```

```sh
beatcraft-cli run house.beatcraft.json project.set_tempo '{"bpm": 120}'
```

```json
{
  "tempo": 120.0
}
```

Light the drum steps. `step.fill` lights every Nth step and clears the others in that channel:

```sh
beatcraft-cli run house.beatcraft.json step.fill '{"pattern": 0, "channel": 0, "every": 4}'
```

```json
{
  "channel": 0,
  "lit_steps": [
    0,
    4,
    8,
    12
  ],
  "pattern": 0
}
```

```sh
beatcraft-cli run house.beatcraft.json step.fill '{"pattern": 0, "channel": 1, "every": 8, "offset": 4}'
```

```json
{
  "channel": 1,
  "lit_steps": [
    4,
    12
  ],
  "pattern": 0
}
```

```sh
beatcraft-cli run house.beatcraft.json step.fill '{"pattern": 0, "channel": 2, "every": 4, "offset": 2, "velocity": 0.6}'
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

Add the bass channel (it becomes channel 4, on mixer insert 5) and its four notes. `start_step`
and `length_steps` are in 1/16 steps:

```sh
beatcraft-cli run house.beatcraft.json channel.add '{"kind": "Bass"}'
```

```json
{
  "channel": 4,
  "insert": 5,
  "name": "Sub bass"
}
```

```sh
beatcraft-cli run house.beatcraft.json note.add '{"pattern": 0, "channel": 4, "key": 36, "start_step": 2, "length_steps": 2}'
```

```json
{
  "channel": 4,
  "notes": 1,
  "pattern": 0,
  "pattern_bars": 1.0
}
```

```sh
beatcraft-cli run house.beatcraft.json note.add '{"pattern": 0, "channel": 4, "key": 36, "start_step": 6, "length_steps": 2}'
beatcraft-cli run house.beatcraft.json note.add '{"pattern": 0, "channel": 4, "key": 39, "start_step": 10, "length_steps": 2}'
beatcraft-cli run house.beatcraft.json note.add '{"pattern": 0, "channel": 4, "key": 34, "start_step": 14, "length_steps": 2}'
```

Load a reverb on the Clap's mixer insert (channel 1 sits on insert 2) and turn it up. The amount
only sounds while the effect is loaded:

```sh
beatcraft-cli run house.beatcraft.json mixer.load_fx '{"insert": 2, "slot": 0, "fx": "reverb"}'
```

```json
{
  "fx": "Room reverb",
  "insert": 2,
  "slot": 0
}
```

```sh
beatcraft-cli run house.beatcraft.json mixer.set '{"insert": 2, "reverb": 0.25}'
```

```json
{
  "insert": 2,
  "mute": false,
  "name": "Clap",
  "pan": 0.0,
  "solo": false,
  "volume_db": 0.0
}
```

Place pattern 0 on the playlist for 8 bars:

```sh
beatcraft-cli run house.beatcraft.json clip.add '{"pattern": 0, "track": 0, "bar": 0, "bars": 8}'
```

```json
{
  "clip": 0,
  "song_bars": 8
}
```

Check the project, then render it:

```sh
beatcraft-cli info house.beatcraft.json
```

```json
{
  "channels": [
    {
  …
      "pan": 0.0,
      "volume": 0.800000011920929
    }
  ],
  "clips": [
    {
      "bar": 0,
      "bars": 8.0,
      "index": 0,
      "pattern": 0,
      "track": 0
    }
  ],
  "name": "Four on the Floor",
  "patterns": [
    {
      "bars": 1.0,
      "index": 0,
      "lit_steps": 10,
      "name": "Pattern 1",
      "notes": 4,
      "steps": 16
    }
  ],
  "song_bars": 8,
  "swing": 0.0,
  "tempo": 120.0
}
```

```sh
beatcraft-cli render house.beatcraft.json house.wav
```

```json
{
  "measured_bpm": 120.0,
  "path": "house.wav",
  "seconds": 18.0
}
```

The file is 3175244 bytes: 18 seconds of 44.1 kHz stereo 16-bit audio (8 bars at 120 BPM is 16 seconds, plus the 2-second tail).

`measured_bpm` is the tempo measured back from the rendered audio, 120 as expected.

## C. As one MCP call

An assistant connected through [MCP Setup](MCP-Setup) can send the whole beat as one `batch` tool
call. These are the `arguments` of that call:

```json
{"steps": [
  {"command": "project.rename", "params": {"name": "Four on the Floor"}},
  {"command": "project.set_tempo", "params": {"bpm": 120}},
  {"command": "step.fill", "params": {"pattern": 0, "channel": 0, "every": 4}},
  {"command": "step.fill", "params": {"pattern": 0, "channel": 1, "every": 8, "offset": 4}},
  {"command": "step.fill", "params": {"pattern": 0, "channel": 2, "every": 4, "offset": 2, "velocity": 0.6}},
  {"command": "channel.add", "params": {"kind": "Bass"}},
  {"command": "note.add", "params": {"pattern": 0, "channel": 4, "key": 36, "start_step": 2, "length_steps": 2}},
  {"command": "note.add", "params": {"pattern": 0, "channel": 4, "key": 36, "start_step": 6, "length_steps": 2}},
  {"command": "note.add", "params": {"pattern": 0, "channel": 4, "key": 39, "start_step": 10, "length_steps": 2}},
  {"command": "note.add", "params": {"pattern": 0, "channel": 4, "key": 34, "start_step": 14, "length_steps": 2}},
  {"command": "mixer.load_fx", "params": {"insert": 2, "slot": 0, "fx": "reverb"}},
  {"command": "mixer.set", "params": {"insert": 2, "reverb": 0.25}},
  {"command": "clip.add", "params": {"pattern": 0, "track": 0, "bar": 0, "bars": 8}}
]}
```

Send `open_project` first with a path such as `house.beatcraft.json`, so the batch is saved to that
file, then `batch` with the steps above, then `export_wav` with a path. The batch result is an
array with the result of each of the 13 steps, for example the third is
`{"channel": 0, "lit_steps": [0, 4, 8, 12], "pattern": 0}` and the last is
`{"clip": 0, "song_bars": 8}`. The `export_wav` result is:

```json
{
  "measured_bpm": 120.0,
  "path": "mcp-house.wav",
  "seconds": 18.0
}
```

The project made this way has the same content as the one from part B, and the WAV file is byte-for-byte identical.
You can also just ask the assistant, for example: "Make a 120 BPM house beat with a four-on-the-floor
kick, claps on 2 and 4, off-beat hats and a simple bass line, 8 bars, and export it."

## Next

[examples/trap-beat.sh](https://github.com/MustafaYusufYalcin/beatcraft/blob/main/examples/trap-beat.sh)
builds a 140 BPM trap beat with a deep sub bass and a pad using the same commands, then renders
it. Read it, change the numbers and run it:

```sh
BEATCRAFT_CLI=./target/release/beatcraft-cli bash examples/trap-beat.sh my-trap.beatcraft.json my-trap.wav
```

All commands are described in the [Command Reference](Command-Reference).
