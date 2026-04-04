#!/bin/sh
set -e

REPO="cerul-ai/cerul-cli"
INSTALL_DIR="${CERUL_INSTALL_DIR:-/usr/local/bin}"

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$OS" in
  darwin) PLATFORM="darwin" ;;
  linux)  PLATFORM="linux" ;;
  *)      echo "Error: Unsupported OS: $OS" >&2; exit 1 ;;
esac

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64)  ARCH="x86_64" ;;
  arm64|aarch64)  ARCH="aarch64" ;;
  *)              echo "Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

# Get latest release tag
LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*"v//' | sed 's/".*//')
if [ -z "$LATEST" ]; then
  echo "Error: Could not determine latest release." >&2
  exit 1
fi

ARTIFACT="cerul-${PLATFORM}-${ARCH}"
URL="https://github.com/$REPO/releases/download/v${LATEST}/${ARTIFACT}.tar.gz"

echo "Installing cerul v${LATEST} (${PLATFORM}/${ARCH})..."

# Download and extract
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT
curl -fsSL "$URL" -o "$TMPDIR/${ARTIFACT}.tar.gz"
tar xzf "$TMPDIR/${ARTIFACT}.tar.gz" -C "$TMPDIR"

# Install
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMPDIR/cerul" "$INSTALL_DIR/cerul"
else
  echo "Installing to $INSTALL_DIR (requires sudo)..."
  sudo mv "$TMPDIR/cerul" "$INSTALL_DIR/cerul"
fi
chmod +x "$INSTALL_DIR/cerul"

echo "Installed cerul v${LATEST} to $INSTALL_DIR/cerul"
"$INSTALL_DIR/cerul" --version
