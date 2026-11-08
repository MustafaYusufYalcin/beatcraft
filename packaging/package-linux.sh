#!/bin/bash
# Builds BeatCraft for Linux and packs it as dist/BeatCraft-<version>-linux-<arch>.tar.gz
# Build needs (Debian/Ubuntu): sudo apt install build-essential pkg-config libasound2-dev \
#   libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev
set -euo pipefail
cd "$(dirname "$0")/.."

NAME="BeatCraft"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
ARCH="$(uname -m)"
OUT="dist/$NAME-$VERSION-linux-$ARCH"

cargo build --release
rm -rf "$OUT"; mkdir -p "$OUT"
cp "target/release/$NAME" "$OUT/$NAME"
strip "$OUT/$NAME" || true
cp "target/release/beatcraft-cli" "$OUT/beatcraft-cli"
strip "$OUT/beatcraft-cli" || true
cp packaging/icon.png "$OUT/beatcraft.png"
cp packaging/README.txt "$OUT/README.txt"
cp LICENSE-MIT LICENSE-APACHE NOTICE "$OUT/"
cat > "$OUT/beatcraft.desktop" <<DESK
[Desktop Entry]
Type=Application
Name=BeatCraft
Comment=Pattern-based music studio
Exec=BeatCraft
Icon=beatcraft
Categories=AudioVideo;Audio;Music;
Terminal=false
StartupWMClass=beatcraft
DESK
cat > "$OUT/install.sh" <<'INST'
#!/bin/bash
# Installs BeatCraft for the current user (no root needed).
set -e
cd "$(dirname "$0")"
mkdir -p ~/.local/bin ~/.local/share/applications ~/.local/share/icons/hicolor/1024x1024/apps
install -m 755 BeatCraft ~/.local/bin/BeatCraft
install -m 755 beatcraft-cli ~/.local/bin/beatcraft-cli
install -m 644 beatcraft.png ~/.local/share/icons/hicolor/1024x1024/apps/beatcraft.png
sed "s|^Exec=.*|Exec=$HOME/.local/bin/BeatCraft|" beatcraft.desktop > ~/.local/share/applications/beatcraft.desktop
echo "BeatCraft installed. Find it in your app menu, or run ~/.local/bin/BeatCraft"
INST
chmod +x "$OUT/install.sh" "$OUT/$NAME"
tar -C dist -czf "$OUT.tar.gz" "$(basename "$OUT")"
(cd dist && sha256sum "$(basename "$OUT").tar.gz" > "SHA256SUMS-linux.txt")
echo "Done: $OUT.tar.gz"
