# Getting Started

## Build and run

You need a recent stable [Rust](https://rustup.rs).

```sh
git clone https://github.com/MustafaYusufYalcin/beatcraft
cd beatcraft
cargo run --release --bin BeatCraft
```

The same works on macOS, Windows and Linux. On Linux install the ALSA and windowing headers first
(Debian/Ubuntu):

```sh
sudo apt install libasound2-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev
```

The command-line program is a second binary in the same crate:

```sh
cargo build --release --bin beatcraft-cli
./target/release/beatcraft-cli list
```

Put `beatcraft-cli` on your `PATH` to use the examples in this wiki as written. The Windows and
Linux packages already include it (the zip root on Windows, `bin/` in the Linux tarball and AppImage); on macOS it is a separate `beatcraft-cli` zip.
See [Releasing](Releasing).

### In the browser

```sh
rustup target add wasm32-unknown-unknown
# install trunk first: https://trunkrs.dev
trunk serve --release
```

Then open http://127.0.0.1:8080 and click the page once, because browsers keep sound suspended
until you interact with the page. The web build cannot save projects or export WAV files yet.

### Start-up options

`BeatCraft` accepts a few arguments:

| Argument | Does |
|---|---|
| `my-beat.beatcraft.json` (any argument ending in `.json`), or `--open <file>` | Opens that project at start |
| `--version` | Prints the version |
| `--export-demo <out.wav>` | Renders the built-in demo song to a WAV file and exits |
| `--screenshot <out.png>` | Starts the app playing the demo song, saves one picture and exits |
| `--write-icon <file> [size]` | Writes the app icon as a PNG and exits |

The app always starts on the demo song, in song mode, with the message "Welcome! Press Space to
play the demo song."

## Opening, saving and exporting

- **Save** is Ctrl+S (Cmd+S on macOS) or FILE, Save. It writes `<project name>.beatcraft.json` into
  the `BeatCraft` folder inside your `Music` folder: `~/Music/BeatCraft` on macOS and Linux, and
  `Music\BeatCraft` under `%USERPROFILE%` on Windows (BeatCraft reads `HOME` first and falls back
  to `USERPROFILE`). An existing file with the same name is
  overwritten without asking.
- **Export song to WAV** (FILE menu) writes `<project name>.wav` into the same folder.
- **Open saved** (FILE menu) and the Projects list in the browser panel list the
  `*.beatcraft.json` files in that folder. To open a file from anywhere else, pass it on the
  command line.
- The app has no field for renaming a project, so a project made with FILE, New is saved as
  `Untitled.beatcraft.json`. Rename it with `project.rename` from the CLI or MCP (see
  [Command Reference](Command-Reference)) or by editing the `name` field in the file.
- File names keep letters, digits, spaces and hyphens; any other character in the project name
  becomes `_`.
- If a file cannot be read, the status line says "Could not open that file". Files are repaired on
  load; see [Project File Format](Project-File-Format).

## The window

| Area | Where | What it is |
|---|---|---|
| Menu bar | top | FILE, ADD (new channel), PATTERNS (new, copy of the current pattern, clear), VIEW, HELP; the status line and the project name |
| Transport | top | PAT / SONG, play, stop, a record button that does nothing yet, tempo, TAP, song position, pattern selector, master VOL / PITCH / SWING knobs, scope, CPU and POLY readouts, "AUDIO ON" / "NO AUDIO", and buttons for the four panels |
| Browser | left | Patterns, channels, instruments (click to add a channel), effects (click to load into the first empty slot of the selected mixer track), projects |
| Playlist | centre | The arrangement: clips on tracks, 24 of the 500 tracks shown |
| Channel rack | floating window | One row per channel with mute, pan, volume, mixer route, name and the step buttons |
| Piano roll | floating window | Notes of the selected channel in the current pattern, with a velocity lane |
| Mixer | bottom | Master, 125 inserts and Current, with the effect slots of the selected track on the right |
| Channel settings | floating window | Name, instrument, volume, pan, pitch, tone and envelope knobs |

Open channel settings by double-clicking a channel name in the rack, or from its right-click menu.

### Controls

| Where | Gesture |
|---|---|
| Channel rack, step | Click toggles it on (velocity 0.8) or off. Right-click clears it. Scroll over a lit step changes its velocity in 0.05 steps |
| Channel rack, channel name | Click selects the channel and plays its sound. Right-click: open in piano roll, channel settings, fill every 4 or 2 steps, clear, delete channel |
| Channel rack, mixer route box | Drag up or down to change the insert, click to show it in the mixer |
| Channel rack, steps 16 / 32 / 64 | Sets the pattern length |
| Piano roll | Click an empty cell to add a note (it is as long as the last one you drew or resized; 2 steps at first), drag a note to move it, drag its right edge to resize, right-click to delete. Click the keyboard on the left to hear a key. The velocity lane below sets velocity: click or drag near a note's start. Quantize snaps all note starts to the step grid |
| Playlist | Click an empty cell to place the current pattern at that bar. Drag a clip to move it, drag its right edge to resize (in quarter bars), right-click to delete. Click a clip to make its pattern the current one. Click the ruler to move the song position |
| Knobs | Drag up or down; hold Shift for fine control; scroll over them; double-click resets |
| Faders | Drag; Shift for fine control; double-click resets to 0 dB (0.8) |
| Tempo display | Drag up or down (0.5 BPM per pixel, whole BPM; Shift for 0.05 steps), scroll for 1 BPM, double-click for 140. Range 20 to 400 |
| Pattern selector | Arrows step through patterns, `+` adds one, right-click the name for a list |

## Keyboard shortcuts

Shortcuts are ignored while a text field has focus, except Ctrl/Cmd+S.

| Key | Does |
|---|---|
| Space | Play, or pause and resume at the same position |
| L | Switch between pattern and song mode, and return to the start |
| F5 or F6 | Show or hide the channel rack |
| F7 | Show or hide the piano roll |
| F8 | Show or hide the browser |
| F9 | Show or hide the mixer |
| Z S X D C V G B H N J M | Play the selected channel: C, C#, D, D#, E, F, F#, G, G#, A, A#, B of the keyboard octave |
| Comma, Q 2 W 3 E R 5 T 6 Y 7 U I | The next octave, from C up to the following C (`,` and `Q` are both the C) |
| `-` / `=` | Keyboard octave down / up (1 to 8, starts at 5, where Z is key 60 = C5) |
| Ctrl/Cmd + S | Save |

Playing keys sends notes to the selected channel at velocity 0.85 and holds them while the key is
down. On a drum channel every key plays the same drum sound.

## Pattern mode and song mode

- **Pattern mode (PAT)** loops the current pattern. Use it while you are writing steps and notes.
- **Song mode (SONG)** plays the clips on the playlist from the start and loops at the end of the
  last clip, rounded up to whole bars. With an empty playlist song mode is silent, so a project
  made with FILE, New needs clips before it makes sound in song mode, or press L for pattern mode.

Space and the play button pause and resume at the same position. The stop button rewinds to the start
and silences notes that are still ringing.

## Tap tempo

Press TAP in time with the music. After the third tap BeatCraft averages the gaps between all taps
made within the last 2.5 seconds and sets the tempo, rounded to a whole BPM. Results outside
20 to 400 BPM are refused with "Tap tempo is out of range" and the tempo stays as it was. Stop tapping for
2.5 seconds to start over.

Next: [Your First Beat](Your-First-Beat).
