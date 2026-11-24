#!/bin/bash
# Builds BeatCraft.app and wraps it in a .dmg and a .zip under dist/.
# Usage: packaging/package-macos.sh [--universal]
#   --universal  also builds for Intel Macs and joins both (needs: rustup target add x86_64-apple-darwin)
set -euo pipefail
cd "$(dirname "$0")/.."

NAME="BeatCraft"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
DIST="dist"
APP="$DIST/$NAME.app"

if [[ "${1:-}" == "--universal" ]]; then
  cargo build --release --target aarch64-apple-darwin
  cargo build --release --target x86_64-apple-darwin
  BIN="$DIST/.bin-universal"
  mkdir -p "$DIST"
  lipo -create -output "$BIN" \
    "target/aarch64-apple-darwin/release/$NAME" "target/x86_64-apple-darwin/release/$NAME"
  CLI="$DIST/.cli-universal"
  lipo -create -output "$CLI" \
    "target/aarch64-apple-darwin/release/beatcraft-cli" "target/x86_64-apple-darwin/release/beatcraft-cli"
  ARCH="universal"
else
  cargo build --release
  BIN="target/release/$NAME"
  CLI="target/release/beatcraft-cli"
  ARCH="$(uname -m)"
fi

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/$NAME"
strip -x "$APP/Contents/MacOS/$NAME"

# Icon: build an .icns from the 1024 px picture
ICONSET="$DIST/$NAME.iconset"
rm -rf "$ICONSET"; mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z $s $s packaging/icon.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  d=$((s * 2))
  sips -z $d $d packaging/icon.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/$NAME.icns"
rm -rf "$ICONSET"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>$NAME</string>
  <key>CFBundleDisplayName</key><string>$NAME</string>
  <key>CFBundleIdentifier</key><string>io.github.mustafayusufyalcin.beatcraft</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleExecutable</key><string>$NAME</string>
  <key>CFBundleIconFile</key><string>$NAME</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.music</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Ad-hoc signature so macOS runs it locally (not notarised).
codesign --force --deep --sign - "$APP"

ZIP="$DIST/beatcraft-$VERSION-macos-$ARCH.zip"
DMG="$DIST/beatcraft-$VERSION-macos-$ARCH.dmg"
rm -f "$ZIP" "$DMG"
ditto -c -k --keepParent "$APP" "$ZIP"
STAGE="$DIST/.dmg-stage"
rm -rf "$STAGE"; mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
hdiutil create -volname "$NAME" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$STAGE"
CLIZIP="$DIST/beatcraft-cli-$VERSION-macos-$ARCH.zip"
rm -f "$CLIZIP"; CLISTAGE="$DIST/.cli-stage"; rm -rf "$CLISTAGE"; mkdir -p "$CLISTAGE"
cp "$CLI" "$CLISTAGE/beatcraft-cli"; strip -x "$CLISTAGE/beatcraft-cli"; codesign --force --sign - "$CLISTAGE/beatcraft-cli"
ditto -c -k --norsrc --noextattr "$CLISTAGE" "$CLIZIP"; rm -rf "$CLISTAGE"
[[ "$ARCH" == "universal" ]] && rm -f "$BIN" "$CLI"
(cd "$DIST" && shasum -a 256 "$(basename "$ZIP")" "$(basename "$DMG")" "$(basename "$CLIZIP")" > "SHA256SUMS-macos.txt")

echo "Done:"
ls -lh "$ZIP" "$DMG" "$CLIZIP" "$DIST/SHA256SUMS-macos.txt"
