# Testing

## Running the tests

```sh
cargo test
```

This runs 61 tests and takes a few seconds. They need no audio device, no network and no window.
Tests that write files use names like `beatcraft-test-<process id>-<name>` in your temporary folder
and remove them when they finish.

Before you open a pull request run the same checks CI runs:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo check --target wasm32-unknown-unknown --bin BeatCraft
```

The formatting rules are in `rustfmt.toml` at the repository root.

## Why the unit tests run through `beatcraft-cli`

`model.rs`, `synth.rs`, `engine.rs` and `commands.rs` are compiled into the `beatcraft-cli` binary
(see [Architecture](Architecture)). The `BeatCraft` app binary also compiles the first three, so
its entry in `Cargo.toml` has `test = false`: that way each shared module's tests run once, in the
CLI binary, instead of twice. Unit tests live in a `#[cfg(test)] mod tests` at the bottom of each
file. Integration tests are in `tests/cli.rs` and run the real binary through
`env!("CARGO_BIN_EXE_beatcraft-cli")`.

## What the 61 tests cover

| File | Tests | Covers |
|---|---|---|
| `src/model.rs` | 17 | The volume taper (0 dB points, the default level, rising levels), the pan law, timing constants and conversions, swing snapping, pattern length rounding, adding and removing channels and patterns, the tempo-from-audio window, key names, that `sanitize` leaves good projects unchanged and is idempotent, that it repairs a deliberately broken project, non-finite numbers, missing channels and patterns, rejecting garbage JSON, and the JSON round trip of the demo |
| `src/synth.rs` | 6 | Every instrument makes finite sound and ends, note pitch, the random source, the delay time of the echo, the low-pass filter and the reverb tail |
| `src/engine.rs` | 11 | Silence for an empty song, steps landing on the grid, swing, mute and solo, no re-triggering after the song ends (twice: through `render_song` and through the exported file), clip looping and cutting, a deterministic and in-range demo render, effects only working when loaded, pattern mode looping, and rendering a repaired broken project |
| `src/commands.rs` | 16 | Every command id is known, the list output, tempo, swing, rename and templates, channels, patterns, steps, notes, clips, mixer and effects, the limit and overflow errors, the WAV header of an export, and that only `song.export_wav` takes a path (the MCP file-access check relies on it) |
| `tests/cli.rs` | 11 | `list` prints 19 commands, `run` creates and `info` reads a project, `info` and `render` refuse a missing file, bad input and exit codes, the trap-beat example script renders a WAV (Unix only), a hand-broken project file still works, a full MCP session over stdio, and the confined MCP server (roots, `..` and symlink escapes, read-only roots, bad arguments) |

## CI

`.github/workflows/ci.yml` runs on every push to `main` and every pull request, on Ubuntu 22.04
with the stable Rust toolchain. It installs the Linux build libraries and then runs the four checks
above in order: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and
`cargo check --target wasm32-unknown-unknown --bin BeatCraft`. A warning from clippy fails the build.

## Writing an audio test

Tests render a project to samples and look at them; they never use a sound card. The tools are
in `src/engine.rs`:

- `render_song(&project)` plays the arrangement once from tick 0 with looping off, then a
  2-second tail, and returns `RenderedSong { pcm, play_frames }`. `pcm` is interleaved stereo in
  -1 to 1; `play_frames` is the length of the song without the tail.
- Build the project from `Project::empty()` (channels 0 to 3 are Kick, Clap, Hat, Snare) and add a
  clip so there is a song.
- Use the **hat** for timing tests: it decays in a few milliseconds, so its first non-zero sample
  is a clean onset.
- Find the onset as the first non-zero sample of the left channel (`pcm[i * 2]`).
- Compare with a **tolerance of 64 frames**, because the sequencer works in 64-frame chunks and
  starts a sound at the beginning of the chunk. At 140 BPM and 44.1 kHz one bar is 75600 frames and
  a step is 4725.

```rust
#[test]
fn hat_on_step_8_lands_mid_bar() {
    let mut p = Project::empty();
    p.patterns[0].data[2].steps[8] = 1.0; // channel 2 is the hat
    p.clips.push(Clip { track: 0, pattern: 0, start: 0, len: BAR_TICKS });
    let song = render_song(&p);
    let onset = (0..song.pcm.len() / 2).find(|i| song.pcm[i * 2] != 0.0).unwrap();
    assert!(onset.abs_diff(75600 / 2) <= 64);
}
```

For a command test, call `run(&mut project, "command.id", &json!({...}))` and assert on the `Ok`
value or the `Err` text, as the tests in `src/commands.rs` do. A new command needs a test there and
a section in the [Command Reference](Command-Reference).

## What is not tested

- The window (`app.rs`, `rack.rs`, `piano.rs`, `playlist.rs`, `mixer.rs`, `widgets.rs`): there are
  no UI tests. Check UI changes by hand and attach a screenshot (`BeatCraft --screenshot shot.png`).
- Real audio devices and the audio callback: tests use the offline renderer.
- The Windows and Linux packages and the macOS bundle: CI does not build them.
- The browser build beyond `cargo check --target wasm32-unknown-unknown`: it is not run in a
  browser.
- Sound quality: the tests check timing, levels and that the output is finite, not how it sounds.
  If you change a sound, render something and listen to it.
