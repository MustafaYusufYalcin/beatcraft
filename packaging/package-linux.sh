#!/bin/bash
# Builds BeatCraft for Linux: dist/beatcraft-<version>-linux-<arch>.tar.gz and, on x86_64, an .AppImage.
# Build needs (Debian/Ubuntu): sudo apt install build-essential pkg-config libasound2-dev \
#   libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev
# The AppImage needs appimagetool: set APPIMAGETOOL to an executable, or it downloads the pinned 1.9.1 (x86_64 only).
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
ARCH="$(uname -m)"
APP_ID="io.github.mustafayusufyalcin.beatcraft"
BASE="beatcraft-$VERSION-linux-$ARCH"
OUT="dist/$BASE"
TOOL_URL="https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage"
TOOL_SHA256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"

cargo build --release
mkdir -p dist

write_desktop() { # $1 = file, $2 = Exec value
  cat > "$1" <<DESK
[Desktop Entry]
Type=Application
Name=BeatCraft
Comment=Pattern-based music studio
Exec=$2
Icon=$APP_ID
Categories=AudioVideo;Audio;Music;Sequencer;
Terminal=false
StartupWMClass=$APP_ID
DESK
}

install_tree() { # $1 = prefix holding bin/ and share/
  mkdir -p "$1/bin" "$1/share/applications"
  cp "target/release/BeatCraft" "$1/bin/beatcraft"
  cp "target/release/beatcraft-cli" "$1/bin/beatcraft-cli"
  strip "$1/bin/beatcraft" "$1/bin/beatcraft-cli" || true
  chmod 755 "$1/bin/beatcraft" "$1/bin/beatcraft-cli"
  for n in 128 256 512; do
    mkdir -p "$1/share/icons/hicolor/${n}x${n}/apps"
    cp "packaging/icons/hicolor/${n}x${n}/apps/$APP_ID.png" "$1/share/icons/hicolor/${n}x${n}/apps/$APP_ID.png"
  done
  write_desktop "$1/share/applications/$APP_ID.desktop" beatcraft
}

# --- tarball ---
rm -rf "$OUT"
install_tree "$OUT"
mkdir -p "$OUT/share/doc/beatcraft"
cp packaging/README.txt LICENSE-MIT LICENSE-APACHE NOTICE "$OUT/share/doc/beatcraft/"
cat > "$OUT/install.sh" <<'INST'
#!/bin/bash
# Installs BeatCraft for the current user (no root needed).
set -e
cd "$(dirname "$0")"
mkdir -p ~/.local
cp -R bin share ~/.local/
sed -i.bak "s|^Exec=.*|Exec=$HOME/.local/bin/beatcraft|" ~/.local/share/applications/io.github.mustafayusufyalcin.beatcraft.desktop
rm -f ~/.local/share/applications/io.github.mustafayusufyalcin.beatcraft.desktop.bak
echo "BeatCraft installed. Find it in your app menu, or run ~/.local/bin/beatcraft"
INST
chmod +x "$OUT/install.sh"
tar -C dist -czf "$OUT.tar.gz" "$BASE"
rm -rf "$OUT"
FILES=("$BASE.tar.gz")
echo "Done: $OUT.tar.gz"

# --- AppImage ---
if [[ "$ARCH" != "x86_64" && -z "${APPIMAGETOOL:-}" ]]; then
  echo "Skipping the AppImage: this machine is $ARCH and APPIMAGETOOL is not set."
else
  TOOL="${APPIMAGETOOL:-}"
  DOWNLOADED=""
  if [[ -z "$TOOL" ]]; then
    TOOL="dist/appimagetool-x86_64.AppImage"
    DOWNLOADED="$TOOL"
    curl -fsSL -o "$TOOL" "$TOOL_URL"
    GOT="$( (sha256sum "$TOOL" 2>/dev/null || shasum -a 256 "$TOOL") | cut -d' ' -f1)"
    if [[ "$GOT" != "$TOOL_SHA256" ]]; then
      rm -f "$TOOL"
      echo "appimagetool checksum mismatch: $GOT" >&2
      exit 1
    fi
    chmod +x "$TOOL"
  fi
  APPDIR="dist/BeatCraft.AppDir"
  rm -rf "$APPDIR"
  install_tree "$APPDIR/usr"
  cat > "$APPDIR/AppRun" <<'RUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/beatcraft" "$@"
RUN
  chmod +x "$APPDIR/AppRun"
  write_desktop "$APPDIR/$APP_ID.desktop" beatcraft
  cp "packaging/icons/hicolor/256x256/apps/$APP_ID.png" "$APPDIR/$APP_ID.png"
  ln -s "$APP_ID.png" "$APPDIR/.DirIcon"
  ARCH=x86_64 "$TOOL" --appimage-extract-and-run "$APPDIR" "dist/beatcraft-$VERSION-linux-$ARCH.AppImage"
  chmod +x "dist/beatcraft-$VERSION-linux-$ARCH.AppImage"
  rm -rf "$APPDIR"
  [[ -n "$DOWNLOADED" ]] && rm -f "$DOWNLOADED"
  FILES+=("beatcraft-$VERSION-linux-$ARCH.AppImage")
  echo "Done: dist/beatcraft-$VERSION-linux-$ARCH.AppImage"
fi

(cd dist && sha256sum "${FILES[@]}" > "SHA256SUMS-linux.txt")
