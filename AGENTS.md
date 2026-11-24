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
- CI uses the newest stable Rust, which can be newer than the local toolchain: a clippy lint may fail there
  that passes here. Run `rustup update` before trusting a local clippy run, and watch the CI run after a push.
- Releasing: push a `v*` tag; `release.yml` builds, checks and attaches the packages and `SHA256SUMS.txt`.
  The web build deploys from `main` only (`pages.yml`). Asset names follow the ArtCraft Launcher's
  `beatcraft-<version>-<os>-<arch>[-portable].<ext>`; see `PACKAGING.md` before renaming anything.
- Design numbers are our own or come from public documentation. Never take values, text or assets from
  another program's files. Record the source of any new shared convention in `docs/PROVENANCE.md`.
