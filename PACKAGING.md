# Packaging BeatCraft

| System | How | Result |
|---|---|---|
| macOS | `bash packaging/package-macos.sh` (add `--universal` for Apple Silicon + Intel) | `dist/beatcraft-<ver>-macos-<arch>.dmg` and `.zip` |
| Windows | `packaging\package-windows.ps1` on a Windows PC with Rust | `dist\beatcraft-<ver>-windows-x64-portable.zip` (flat: `BeatCraft.exe`, `beatcraft-cli.exe`, README, licences) |
| Linux | `bash packaging/package-linux.sh` on Linux (library list inside the script) | `dist/beatcraft-<ver>-linux-<arch>.tar.gz` (`bin/`, `share/`, `install.sh`) and, on x86_64, `dist/beatcraft-<ver>-linux-<arch>.AppImage` |
| All three at once | push this folder to GitHub and run the "Package BeatCraft" workflow (or push a `v1.0.0` tag) | downloads on the run page / release |

The app icon lives in `packaging/icon.png` and `packaging/icon.ico` (drawn by `BeatCraft --write-icon <file> <size>`).
Mac builds are ad-hoc signed, not notarised; Windows builds are unsigned.

The app id is `io.github.mustafayusufyalcin.beatcraft`: it is the macOS bundle identifier, the Linux desktop file name and the window's app id.
The Linux icons are in `packaging/icons/hicolor/<N>x<N>/apps/io.github.mustafayusufyalcin.beatcraft.png`; regenerate them with
`target/debug/BeatCraft --write-icon packaging/icons/hicolor/<N>x<N>/apps/io.github.mustafayusufyalcin.beatcraft.png <N>` for N = 64, 128, 256 and 512.

The AppImage is built with appimagetool 1.9.1, downloaded and checked against a pinned SHA-256 by the script;
set `APPIMAGETOOL` to an executable to use your own. Without it, non-x86_64 machines skip the AppImage.
