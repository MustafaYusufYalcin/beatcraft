# Changelog

## 0.2.0 (unreleased)

- Fixed `note.add` and `clip.add` crashing the CLI and MCP server on huge positions; both now return an
  error past bar 1000.
- Fixed `pattern.add` renaming an existing pattern when the 999-pattern limit is reached; it now errors.
- Fixed exported WAV files ending with a repeat of the song's first notes in the tail.
- Pattern lengths in `project.info` and `note.add` results are now exact (for example 1.25 bars).
- `info` and `render` now report a missing project file instead of using an empty project.
- Project files are repaired on load (out-of-range or missing values), so a hand-edited file cannot crash
  the app, the CLI or the MCP server.
- Every instrument label works as a `channel.add` kind, including "E-keys".
- Space and the play button now pause and resume at the same position; Stop rewinds.
- Exporting from the app with an empty playlist now says so instead of writing a silent file.
- Projects are saved under `%USERPROFILE%` on Windows.
- Added a test suite, CI (format, lints, tests, browser build check) and a wiki in `docs/wiki/`.
- Packaging: asset names are lowercase (`beatcraft-<version>-<os>-<arch>…`), Windows ships a portable zip,
  Linux ships an AppImage and a tarball with `bin/` and `share/`, and the app id is
  `io.github.mustafayusufyalcin.beatcraft`.
- Yusuf Arslan is listed as co-author and maintainer.

### Added

- `beatcraft-cli mcp` accepts `--automation-read-root` and `--automation-write-root` to confine the files the MCP server may read and write (unconfined without them).

### Changed

- The volume taper, default channel level, swing steps, tempo range (now 20 to 400 BPM) and tap tempo follow BeatCraft's own rules, described in the wiki's Sound Engine page. Projects saved with 0.1.0 still open; their levels differ slightly.
- Before 0.2.0, BeatCraft used a volume-curve formula, several parameter and timing values, and a
  tap-tempo error message obtained by inspecting a commercial DAW's binary and bundled project templates.
  In 0.2.0, we replaced the formula, values and message with our own. Details:
  [docs/PROVENANCE.md](docs/PROVENANCE.md).
- Levels: a project saved with 0.1.0 plays about 1 dB quieter at default channel settings, and more
  at low channel volumes, because of the new taper.

## 0.1.0 - 2026-11-08

First release: step sequencer, piano roll, playlist, mixer, eleven built-in instruments,
WAV export, the `beatcraft-cli` command line and MCP server, and a browser build.
