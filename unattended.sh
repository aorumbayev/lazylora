#!/usr/bin/env bash
set -e

# LazyLora Unattended Installer
# This script installs LazyLora on your system without any user interaction

# Parse arguments
VERSION=""
INSTALL_DIR="/usr/local/bin"
GITHUB_PAT=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --version)
      VERSION="$2"
      shift 2
      ;;
    --dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    --token)
      GITHUB_PAT="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Set auth header if token is provided
AUTH_HEADER=""
if [ -n "$GITHUB_PAT" ]; then
  AUTH_HEADER="-H \"Authorization: token $GITHUB_PAT\""
fi

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Detect OS
OS=$(uname | tr '[:upper:]' '[:lower:]')
case $OS in
    darwin)
        OS="apple-darwin"
        ;;
    linux)
        OS="unknown-linux-gnu"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Get latest version from GitHub if not specified
if [ -z "$VERSION" ]; then
    VERSION=$(curl -s $AUTH_HEADER https://api.github.com/repos/aorumbayev/lazylora/releases/latest | grep -oP '"tag_name": "\K(.*)(?=")')
    if [ -z "$VERSION" ]; then
        echo "Failed to determine latest version"
        exit 1
    fi
fi
VERSION=${VERSION#v}

echo "Installing LazyLora $VERSION for $ARCH-$OS..."

# Download URL
BINARY_NAME="lazylora"
PKG_NAME="lazylora-$VERSION-$ARCH-$OS.tar.gz"
DOWNLOAD_URL="https://github.com/aorumbayev/lazylora/releases/download/v$VERSION/$PKG_NAME"

# Create temp directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Download and extract
echo "Downloading from $DOWNLOAD_URL..."
if [ -n "$GITHUB_PAT" ]; then
    curl -L -H "Authorization: token $GITHUB_PAT" -o "$PKG_NAME" "$DOWNLOAD_URL"
else
    curl -L -o "$PKG_NAME" "$DOWNLOAD_URL"
fi
tar -xzf "$PKG_NAME"

# Mac OS specific quarantine removal
if [ "$OS" = "apple-darwin" ]; then
    echo "Removing quarantine attribute..."
    xattr -d com.apple.quarantine "$BINARY_NAME" 2>/dev/null || true
fi

# Install the binary
chmod +x "$BINARY_NAME"
mv "$BINARY_NAME" "$INSTALL_DIR/"

# Clean up
cd - > /dev/null
rm -rf "$TMP_DIR"

echo "LazyLora $VERSION has been installed to $INSTALL_DIR/$BINARY_NAME"
echo "Installation completed successfully." 
