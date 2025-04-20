#!/usr/bin/env bash
set -e

# LazyLora Installer
# This script installs LazyLora on your system

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Directories
BIN_DIR="/usr/local/bin"
if [ ! -d "$BIN_DIR" ]; then
    BIN_DIR="$HOME/.local/bin"
    mkdir -p "$BIN_DIR"
fi

# Add to PATH if needed
case ":$PATH:" in
  *":$BIN_DIR:"*) : ;; # already in PATH
  *) echo "export PATH=$BIN_DIR:\$PATH" >> ~/.bashrc
     echo "export PATH=$BIN_DIR:\$PATH" >> ~/.zshrc
     export PATH="$BIN_DIR:$PATH" ;;
esac

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
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
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
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

# Get latest version from GitHub
VERSION=$(curl -s https://api.github.com/repos/aorumbayev/lazylora/releases/latest | grep -oP '"tag_name": "\K(.*)(?=")')
if [ -z "$VERSION" ]; then
    echo -e "${RED}Failed to determine latest version${NC}"
    exit 1
fi
VERSION=${VERSION#v}

echo -e "${BLUE}Installing LazyLora $VERSION for $ARCH-$OS...${NC}"

# Download URL
BINARY_NAME="lazylora"
PKG_NAME="lazylora-$VERSION-$ARCH-$OS.tar.gz"
DOWNLOAD_URL="https://github.com/aorumbayev/lazylora/releases/download/v$VERSION/$PKG_NAME"

# Create temp directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Download and extract
echo -e "${BLUE}Downloading from $DOWNLOAD_URL...${NC}"
curl -L -o "$PKG_NAME" "$DOWNLOAD_URL"
tar -xzf "$PKG_NAME"

# Mac OS specific quarantine removal
if [ "$OS" = "apple-darwin" ]; then
    echo -e "${BLUE}Removing quarantine attribute...${NC}"
    xattr -d com.apple.quarantine "$BINARY_NAME" 2>/dev/null || true
    
    # Notarization validation
    codesign --verify --verbose "$BINARY_NAME" 2>/dev/null || echo -e "${BLUE}Binary is not code signed. This is expected for open source tools.${NC}"
fi

# Install the binary
chmod +x "$BINARY_NAME"
mv "$BINARY_NAME" "$BIN_DIR/"

# Clean up
cd - > /dev/null
rm -rf "$TMP_DIR"

echo -e "${GREEN}LazyLora $VERSION has been installed to $BIN_DIR/$BINARY_NAME${NC}"
echo -e "${GREEN}Run 'lazylora' to get started.${NC}" 
