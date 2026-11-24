# Releasing

This page describes how a BeatCraft release is put together. The steps are what the files in the
repository do; nothing here runs on its own except the two workflows when a tag is pushed.

## 1. Bump the version

The version lives in one place: `version = "…"` in the `[package]` section of `Cargo.toml`.
Change it, run `cargo build` so `Cargo.lock` follows, and commit. That one number is used for:

- `BeatCraft --version` and `beatcraft-cli --version`;
- the `serverInfo.version` the MCP server reports;
- the names of the packages (`beatcraft-<version>-…`) and the macOS app's version, because the
  packaging scripts read the first `version` line of `Cargo.toml`.

## 2. Packaging scripts

Each system has a script (details in [PACKAGING.md](https://github.com/MustafaYusufYalcin/beatcraft/blob/main/PACKAGING.md)).
They write to `dist/`, which is not tracked by git.

| System | Script | Output |
|---|---|---|
| macOS | `bash packaging/package-macos.sh` (`--universal` for Apple Silicon and Intel together) | `beatcraft-<ver>-macos-<arch>.dmg` and `.zip`, a separate `beatcraft-cli-<ver>-macos-<arch>.zip`, and `SHA256SUMS-macos.txt` |
| Windows | `packaging\package-windows.ps1` | `beatcraft-<ver>-windows-x64-portable.zip` (flat: `BeatCraft.exe`, `beatcraft-cli.exe`, README and licences) and `SHA256SUMS-windows.txt` |
| Linux | `bash packaging/package-linux.sh` | `beatcraft-<ver>-linux-<arch>.tar.gz` (`bin/beatcraft`, `bin/beatcraft-cli`, `share/` with the desktop file and icons, and an `install.sh` that installs for the current user) and, on x86_64, `beatcraft-<ver>-linux-<arch>.AppImage` and `SHA256SUMS-linux.txt` |

The icon files are `packaging/icon.png`, `packaging/icon.ico` and the Linux set in `packaging/icons/hicolor/`;
`BeatCraft --write-icon <file> <size>` redraws them. The app id is `io.github.mustafayusufyalcin.beatcraft` (macOS bundle identifier,
Linux desktop file and icon names, window app id). Asset names are lowercase because that is how installers read them.

## 3. Tag and push

The workflows in `.github/workflows/` start on a tag that begins with `v` (for example `v0.1.1`),
and can also be started by hand from the Actions tab.

**Package BeatCraft** (`release.yml`)

- Builds on Windows, macOS (Apple Silicon and Intel, joined into one universal app) and Ubuntu 22.04,
  by running the matching packaging script. Linux first installs the build libraries. Each platform then
  runs a self-check that fails the job if the package is wrong (executables present, `beatcraft-cli list`
  works, the Windows `ProductVersion` matches the Cargo version, the macOS bundle id and the AppImage
  desktop file are right).
- Uploads the packages and the per-system checksum file as workflow artifacts on every run.
- On a tag it also attaches them to the GitHub release.
- A last job (`checksums`, tags only) merges the three per-system files into one
  `SHA256SUMS.txt`, sorted by file name with Unix line endings, and attaches it to the release.

**Web build** (`pages.yml`)

- Builds the browser version with trunk 0.22.1 (`trunk build --release`) into `dist-web` and
  publishes it to GitHub Pages under `/<repository name>/`.
- It is **skipped while the repository is private**, because Pages needs a public repository on
  the free plan. Once the repository is public and Pages is enabled it runs on tags too.

**CI** (`ci.yml`) is separate: it checks pushes to `main` and pull requests but builds no packages.
See [Testing](Testing).

## Checking a download

Every release has a `SHA256SUMS.txt`. Put it next to the file you downloaded and run, on Linux:

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing
```

or on macOS:

```sh
grep beatcraft-0.2.0-macos-arm64.zip SHA256SUMS.txt | shasum -a 256 -c
```

(with your own file name).

## Signing

- **macOS:** the app and the CLI are signed ad hoc (`codesign --sign -`). They are **not
  notarised**, so on first start macOS may say it cannot check the app: right-click the app and
  choose Open.
- **Windows:** the portable zip is **not signed**. SmartScreen may warn: choose More info, then Run anyway.
- **Linux:** nothing is signed; use the checksums.
