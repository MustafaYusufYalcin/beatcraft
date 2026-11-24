# Architecture

BeatCraft is one Rust crate with two programs and no library target.

| Program | Entry | What it is |
|---|---|---|
| `BeatCraft` | `src/main.rs` | The app: an `eframe` / `egui` window on macOS, Windows and Linux, and the same code compiled to WebAssembly for the browser |
| `beatcraft-cli` | `src/bin/beatcraft-cli.rs` | The command-line program and MCP server. It has no window |

## How the binaries share code

There is no library crate. Each binary lists the modules it needs, and the CLI pulls the shared
files in by path:

```rust
#[path = "../model.rs"]    mod model;
#[path = "../synth.rs"]    mod synth;
#[path = "../engine.rs"]   mod engine;
#[path = "../commands.rs"] mod commands;
```

So `model.rs`, `synth.rs` and `engine.rs` are compiled once into each binary. `commands.rs` is
compiled only into `beatcraft-cli`; the app does not use it. Shared modules must therefore compile
in both, and the unit tests run once, through the CLI binary (see [Testing](Testing)).

## Modules

| File | What it holds |
|---|---|
| `src/model.rs` | Project data (channels, patterns, notes, clips, mixer), the timing and gain rules, `Project::demo`, `sanitize` and `from_json` |
| `src/synth.rs` | The instrument voices (`Voice`), the random source, the filter and the insert effects (`InsertFx`) |
| `src/engine.rs` | `Shared` (state shared with the audio thread), `Engine` (sequencer and mixer), audio output, `render_song` and `export_wav` |
| `src/commands.rs` | The 19 commands (`COMMANDS`, `run`, `info`) |
| `src/bin/beatcraft-cli.rs` | The CLI arguments, loading and saving files, the MCP server |
| `src/main.rs` | Start-up arguments, window options, and the WebAssembly entry point |
| `src/app.rs` | The `App` state, top bar and transport, keyboard, browser panel, save, load and export, tap tempo |
| `src/rack.rs` | The channel rack and channel settings window |
| `src/piano.rs` | The piano roll and velocity lane |
| `src/playlist.rs` | The playlist |
| `src/mixer.rs` | The mixer strips and the effect slots |
| `src/widgets.rs` | Knobs, faders, meters, buttons and the colour palette |
| `src/icon.rs` | The app icon, drawn in code (not built for the browser) |
| `build.rs` | Embeds the icon in the Windows `.exe`; does nothing elsewhere |
| `tests/cli.rs` | Integration tests that drive the `beatcraft-cli` binary |

## How the window and the audio talk

The window thread and the audio thread never share the project directly. They meet in one
`Shared` value behind a mutex:

```
                      +--------------------+
   App (egui thread)  |  Arc<Mutex<Shared>>|  audio callback (cpal thread)
   ------------------ |  ----------------  |  ---------------------------
   project (edited)   |  project (copy)    |  Engine::render(&mut Shared, out)
   playing, song_mode |  playing, ...      |    drains cmds
   current_pattern    |  cmds: Vec<Cmd>    |    advances the sequencer
   meters, scope ...  |  meters, scope ... |    mixes and writes meters, scope
                      +--------------------+
        push_to_engine ->        <- sync_from_engine
        send(Cmd) -------------> cmds
```

- The `App` owns the project being edited. Every change sets a `dirty` flag. Once per frame
  `push_to_engine` copies the whole project into `Shared` if it is dirty, and always copies
  `playing`, `song_mode` and `current_pattern`.
- Once per frame `sync_from_engine` reads back the playhead (`pos_ticks`), meters, scope samples,
  CPU load, voice count and whether an audio device is running.
- One-off events go through the `Cmd` queue (`Cmd::NoteOn`, `NoteOff`, `Preview`, `Panic`),
  pushed by `App::send`. The audio callback drains the queue at the start of each buffer. Typing
  on the computer keyboard, clicking a channel name and previewing piano-roll notes use it.
- The audio callback locks the mutex for every buffer. If the lock is poisoned it writes silence.

### Audio output and the fallback

`start_audio` opens the default output device with `cpal`, but only if its sample format is 32-bit
float. If there is no device, or it cannot be opened, a plain thread is started instead (not in the
browser) that renders 441 frames every 10 ms into a buffer that is thrown away. The app then still
runs, the playhead moves and the meters work, but nothing is heard; the transport shows
"NO AUDIO". The same `Engine` renders in both cases.

### The browser build

`src/main.rs` has a second `main` for `wasm32` that finds the `<canvas id="beatcraft">` from
`index.html`, starts `eframe::WebRunner` and removes the loading text. Sound goes through `cpal`
with its `wasm-bindgen` feature; browsers keep it suspended until the user clicks, and the app calls
`AudioOut::resume` when playback starts. There is no filesystem in the browser, so saving and WAV
export do not work there.

### The command layer

`commands.rs` is how the CLI and MCP change a project. **The app does not go through it yet**: its
buttons edit `App::project` directly. Routing every button through `commands.rs`, plus a control
channel for the running window, is roadmap item 1 in the README.

## Where to add things

These match [CONTRIBUTING.md](https://github.com/MustafaYusufYalcin/beatcraft/blob/main/CONTRIBUTING.md).

- **An instrument:** add a variant to `InstKind` in `model.rs` (add it to `ALL`, `label`, `is_drum`,
  `default_params` and `default_color`) and its sound to `Voice::sample` in `synth.rs`. `channel.add`
  and the browser pick it up from `InstKind::ALL`.
- **An effect:** add a variant to `FxKind` in `model.rs`, its processing to `InsertFx` in
  `synth.rs`, the way the engine reads it in `engine.rs` (`effective_fx`), the choices in
  `mixer.load_fx` in `commands.rs`, and the lists in `mixer.rs` and the browser in `app.rs`.
- **A command:** add a `CommandSpec` to `COMMANDS` and a match arm in `run` in `commands.rs`. The
  CLI and MCP pick it up. Add a test in `src/commands.rs` and a section in the
  [Command Reference](Command-Reference), and update the command count in `tests/cli.rs`.
