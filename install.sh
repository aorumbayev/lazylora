#!/usr/bin/env bash
set -e

# LazyLora Installer
# This script installs LazyLora on your system

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print errors
error_exit() {
    echo -e "${RED}Error: $1${NC}" >&2
    exit 1
}

# Determine installation directory
INSTALL_DIR="/usr/local/bin"
if ! Writable "$INSTALL_DIR" && [ -d "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
fi
# Create if it doesn't exist and check writability again
mkdir -p "$INSTALL_DIR"
if ! Writable "$INSTALL_DIR"; then
    echo -e "${RED}Cannot write to $INSTALL_DIR.${NC}"
    echo -e "Please ensure the directory exists and you have permissions, or run with sudo.${NC}"
    echo -e "Alternatively, set the INSTALL_DIR environment variable to a writable path."
    exit 1
fi

# Add to PATH if needed (using a more robust check)
PATH_CMD="export PATH=\"$INSTALL_DIR:\$PATH\""
SHELL_CONFIG=""
if [ -n "$BASH_VERSION" ]; then
    SHELL_CONFIG="$HOME/.bashrc"
elif [ -n "$ZSH_VERSION" ]; then
    SHELL_CONFIG="$HOME/.zshrc"
fi

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${BLUE}Adding $INSTALL_DIR to your PATH in $SHELL_CONFIG${NC}"
    echo -e "\n# Added by LazyLora Installer\n$PATH_CMD" >> "$SHELL_CONFIG"
    echo -e "${BLUE}Please run 'source $SHELL_CONFIG' or restart your shell.${NC}"
    export PATH="$INSTALL_DIR:$PATH" # Add to current session
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
        error_exit "Unsupported architecture: $ARCH"
        ;;
esac

# Detect OS
OS_TYPE=$(uname -s)
case $OS_TYPE in
    Darwin)
        OS_TAG="darwin"
        ;;
    Linux)
        OS_TAG="linux"
        ;;
    *)
        error_exit "Unsupported OS: $OS_TYPE"
        ;;
esac

# Get latest release tag from GitHub API
API_URL="https://api.github.com/repos/aorumbayev/lazylora/releases/latest"
LATEST_TAG=$(curl -s "$API_URL" | grep '"tag_name":' | sed -E 's/.*"tag_name": "(.*)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    error_exit "Failed to determine latest release tag from $API_URL"
fi

# Construct package name and download URL based on new convention
BINARY_NAME="lazylora"
PKG_NAME="${BINARY_NAME}-${ARCH}-${OS_TAG}.tar.gz"
DOWNLOAD_URL="https://github.com/aorumbayev/lazylora/releases/download/${LATEST_TAG}/${PKG_NAME}"

echo -e "${BLUE}Installing ${BINARY_NAME} ${LATEST_TAG} for ${ARCH}-${OS_TAG}...${NC}"

# Create temp directory and ensure cleanup
TMP_DIR=$(mktemp -d)
trap 'rm -rf -- "$TMP_DIR"' EXIT
cd "$TMP_DIR"

# Download and extract
echo -e "${BLUE}Downloading from $DOWNLOAD_URL...${NC}"
curl -fsSL -o "$PKG_NAME" "$DOWNLOAD_URL" || error_exit "Download failed. Check URL or network."

echo -e "${BLUE}Extracting archive...${NC}"
tar -xzf "$PKG_NAME" || error_exit "Failed to extract archive."

# Check if binary exists after extraction
if [ ! -f "$BINARY_NAME" ]; then
    error_exit "Binary '$BINARY_NAME' not found in the archive."
fi

# Mac OS specific quarantine removal
if [ "$OS_TAG" = "darwin" ]; then
    echo -e "${BLUE}Attempting to remove quarantine attribute (macOS)...${NC}"
    xattr -d com.apple.quarantine "$BINARY_NAME" 2>/dev/null || echo -e "${BLUE}(Could not remove quarantine attribute. You might need to allow execution in System Settings)${NC}"
fi

# Install the binary
echo -e "${BLUE}Installing ${BINARY_NAME} to ${INSTALL_DIR}...${NC}"
chmod +x "$BINARY_NAME"
mv "$BINARY_NAME" "$INSTALL_DIR/"

# Clean up is handled by trap
cd - > /dev/null

echo -e "${GREEN}${BINARY_NAME} ${LATEST_TAG} has been installed to ${INSTALL_DIR}/${BINARY_NAME}${NC}"
echo -e "${GREEN}Run '${BINARY_NAME}' to get started.${NC}" 
