#!/bin/bash
# ParkHub Self-Update Script
set -e

REPO_URL="${PARKHUB_REPO:-https://github.com/frostplexx/parkhub}"
INSTALL_DIR="${PARKHUB_DIR:-$(dirname "$0")}"
DATA_DIR="${PARKHUB_DATA:-$HOME/.local/share/parkhubserver}"

cd "$INSTALL_DIR"

echo "Checking for updates..."

CURRENT=$(./parkhub-server --version 2>/dev/null | grep -oP '[\d.]+' || echo "unknown")
LATEST=$(curl -sL "$REPO_URL/releases/latest" -o /dev/null -w '%{url_effective}' | grep -oP 'v[\d.]+$' | sed 's/^v//' || echo "")

if [ -z "$LATEST" ]; then
  echo "Could not determine latest version. Check your internet connection."
  exit 1
fi

if [ "$CURRENT" = "$LATEST" ]; then
  echo "Already up to date ($CURRENT)"
  exit 0
fi

echo "Updating $CURRENT -> $LATEST"

# Backup data if directory exists
if [ -d "$DATA_DIR" ]; then
  echo "Backing up data..."
  cp -r "$DATA_DIR" "${DATA_DIR}.backup.$(date +%Y%m%d%H%M%S)"
fi

# Try to download pre-built binary
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
BINARY_URL="$REPO_URL/releases/download/v${LATEST}/parkhub-server-${OS}-${ARCH}"

echo "Trying binary download from: $BINARY_URL"
if curl -sLf "$BINARY_URL" -o parkhub-server.new 2>/dev/null; then
  chmod +x parkhub-server.new
  mv parkhub-server parkhub-server.old 2>/dev/null || true
  mv parkhub-server.new parkhub-server
  echo "Updated to $LATEST (binary)"
  echo "Restart the server to apply: systemctl restart parkhub"
else
  echo "Binary not available, trying source build..."
  if command -v cargo &> /dev/null; then
    git fetch --tags
    git checkout "v${LATEST}" 2>/dev/null || git pull origin main
    cargo build --release -p parkhub-server --no-default-features --features headless
    cp target/release/parkhub-server .
    echo "Updated to $LATEST (built from source)"
    echo "Restart the server to apply: systemctl restart parkhub"
  else
    echo "Error: cargo not found. Install Rust or download a pre-built binary."
    exit 1
  fi
fi
