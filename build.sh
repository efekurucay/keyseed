#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIST_DIR="$ROOT_DIR/dist"
TARGET_DIR="$ROOT_DIR/target/release"
BINARY_NAME="hashit"
APP_NAME="Hashit"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
ARCHIVE_NAME="hashit-$OS-$ARCH.tar.gz"

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo bulunamadı. Önce Rust toolchain kur: https://rustup.rs" >&2
  exit 1
fi

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

cargo build --release

if [ "$OS" = "darwin" ]; then
  APP_DIR="$DIST_DIR/$APP_NAME.app"
  CONTENTS_DIR="$APP_DIR/Contents"
  MACOS_DIR="$CONTENTS_DIR/MacOS"

  mkdir -p "$MACOS_DIR"
  cp "$TARGET_DIR/$BINARY_NAME" "$MACOS_DIR/$APP_NAME"
  chmod +x "$MACOS_DIR/$APP_NAME"

  cat > "$CONTENTS_DIR/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>Hashit</string>
  <key>CFBundleIdentifier</key>
  <string>local.hashit.app</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Hashit</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

  tar -czf "$DIST_DIR/$ARCHIVE_NAME" -C "$DIST_DIR" "$APP_NAME.app"
  echo "App: $APP_DIR"
  echo "Archive: $DIST_DIR/$ARCHIVE_NAME"
else
  PACKAGE_DIR="$DIST_DIR/hashit-$OS-$ARCH"
  mkdir -p "$PACKAGE_DIR"
  cp "$TARGET_DIR/$BINARY_NAME" "$PACKAGE_DIR/$BINARY_NAME"
  cp "$ROOT_DIR/README.md" "$PACKAGE_DIR/README.md"
  tar -czf "$DIST_DIR/$ARCHIVE_NAME" -C "$DIST_DIR" "hashit-$OS-$ARCH"
  echo "Binary: $PACKAGE_DIR/$BINARY_NAME"
  echo "Archive: $DIST_DIR/$ARCHIVE_NAME"
fi
