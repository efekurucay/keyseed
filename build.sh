#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIST_DIR="$ROOT_DIR/dist"
TARGET_DIR="$ROOT_DIR/target/release"
BINARY_NAME="hashit"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
VERSION="$(grep '^version = ' "$ROOT_DIR/Cargo.toml" | head -n1 | cut -d '"' -f2)"
PACKAGE_DIR="$DIST_DIR/hashit-v$VERSION-$OS-$ARCH"
ARCHIVE_NAME="hashit-v$VERSION-$OS-$ARCH.tar.gz"

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo bulunamadı. Önce Rust toolchain kur: https://rustup.rs" >&2
  exit 1
fi

rm -rf "$DIST_DIR"
mkdir -p "$PACKAGE_DIR"

cargo build --release

cp "$TARGET_DIR/$BINARY_NAME" "$PACKAGE_DIR/$BINARY_NAME"
cp "$ROOT_DIR/README.md" "$ROOT_DIR/LICENSE" "$PACKAGE_DIR/"

tar -czf "$DIST_DIR/$ARCHIVE_NAME" -C "$DIST_DIR" "hashit-v$VERSION-$OS-$ARCH"

echo "Binary: $PACKAGE_DIR/$BINARY_NAME"
echo "Archive: $DIST_DIR/$ARCHIVE_NAME"
