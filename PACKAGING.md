# Packaging BeatCraft

| System | How | Result |
|---|---|---|
| macOS | `bash packaging/package-macos.sh` (add `--universal` for Apple Silicon + Intel) | `dist/BeatCraft-<ver>-macos-<arch>.dmg` and `.zip` |
| Windows | `packaging\package-windows.ps1` on a Windows PC with Rust | `dist\BeatCraft-<ver>-windows-x64.zip` |
| Linux | `bash packaging/package-linux.sh` on Linux (library list inside the script) | `dist/BeatCraft-<ver>-linux-<arch>.tar.gz` |
| All three at once | push this folder to GitHub and run the "Package BeatCraft" workflow (or push a `v1.0.0` tag) | downloads on the run page / release |

The app icon lives in `packaging/icon.png` and `packaging/icon.ico` (drawn by `BeatCraft --write-icon <file> <size>`).
Mac builds are ad-hoc signed, not notarised; Windows builds are unsigned.
