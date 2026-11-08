# Contributing to BeatCraft

Thanks for helping! BeatCraft is a small Rust codebase, so most changes touch one or two files.

## Getting set up

```sh
git clone https://github.com/MustafaYusufYalcin/beatcraft
cd beatcraft
cargo run --bin BeatCraft          # the app
cargo run --bin beatcraft-cli list # the command layer
```

Linux needs the ALSA and windowing headers listed in the README. For the browser build see the README.

## Where things live

| File | What it holds |
|---|---|
| `src/model.rs` | Project data (channels, patterns, notes, clips, mixer) and the timing and gain rules |
| `src/synth.rs` | How the built-in instruments and insert effects sound |
| `src/engine.rs` | Audio rendering, the mixer, WAV export |
| `src/commands.rs` | Every command the CLI and MCP server can run |
| `src/bin/beatcraft-cli.rs` | The CLI and the MCP server |
| `src/app.rs`, `rack.rs`, `piano.rs`, `playlist.rs`, `mixer.rs`, `widgets.rs` | The user interface |

## Good first steps

- **A new instrument:** add a variant to `InstKind` in `src/model.rs` (name and colour) and its sound
  in `src/synth.rs`; then list it in the browser and in `channel.add`.
- **A new effect:** add a variant to `FxKind` in `src/model.rs`, its processing in `src/synth.rs`,
  and its name in `mixer.load_fx` (`src/commands.rs`).
- **A new command:** add a `CommandSpec` and a match arm in `src/commands.rs`; the CLI and MCP pick it up.
- Issues labelled **good first issue** are picked to be small and self-contained.

## Before you open a pull request

- `cargo fmt` and `cargo clippy` are clean, and `cargo build` works.
- If you changed sound, render something with `beatcraft-cli render` (or `examples/trap-beat.sh`)
  and listen to it.
- If you changed the UI, add a screenshot (`BeatCraft --screenshot shot.png` writes one).
- Keep pull requests focused: one feature or fix each.

## Ground rules

- Everything you contribute is licensed under MIT OR Apache-2.0, like the rest of the project.
- Write code yourself: don't copy code, presets, samples, artwork or other material from
  commercial music software.
- Be kind. Questions are welcome in issues.
