# Notes for agents working on BeatCraft

- Layout: one crate, two binaries (`BeatCraft` GUI in `src/main.rs`, `beatcraft-cli` in `src/bin/`), sharing
  `model.rs`, `synth.rs`, `engine.rs` and `commands.rs` through `#[path]` includes. See `docs/wiki/Architecture.md`.
- Checks before calling a change done (CI runs the same, `.github/workflows/ci.yml`):
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `cargo check --target wasm32-unknown-unknown --bin BeatCraft`.
- Unit tests live at the bottom of the shared modules and run through the `beatcraft-cli` bin
  (the GUI bin has `test = false`); CLI and MCP tests are in `tests/cli.rs`. See `docs/wiki/Testing.md`.
- GUI smoke check: `target/debug/BeatCraft --screenshot out.png` opens the window, saves a picture and exits.
- A new or changed command needs a test in `src/commands.rs` and its section in `docs/wiki/Command-Reference.md`;
  the command count is asserted in `tests/cli.rs` and stated in `README.md`.
- Every sound, icon and demo song is generated in code. Do not add third-party samples, presets or artwork,
  and do not name other companies' products in user-facing text.
