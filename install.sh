#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect OS and architecture
OS=$(uname -s)
ARCH=$(uname -m)

# Determine target triple and asset name
case "$OS" in
  Linux)
    if [ "$ARCH" = "x86_64" ]; then
      ASSET_NAME="kayfabe-x86_64-linux"
    else
      echo -e "${RED}Unsupported architecture: $ARCH${NC}"
      exit 1
    fi
    ;;
  Darwin)
    if [ "$ARCH" = "x86_64" ]; then
      ASSET_NAME="kayfabe-x86_64-macos"
    elif [ "$ARCH" = "arm64" ]; then
      ASSET_NAME="kayfabe-aarch64-macos"
    else
      echo -e "${RED}Unsupported architecture: $ARCH${NC}"
      exit 1
    fi
    ;;
  *)
    echo -e "${RED}Unsupported OS: $OS${NC}"
    exit 1
    ;;
esac

# Installation directory
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

echo -e "${YELLOW}Installing kayfabe...${NC}"

# Get the latest release version
LATEST_RELEASE=$(curl -s https://api.github.com/repos/ShreyeshArangath/kayfabe/releases/latest | grep tag_name | cut -d '"' -f 4)

if [ -z "$LATEST_RELEASE" ]; then
    echo -e "${RED}Error: Could not fetch latest release${NC}"
    exit 1
fi

echo "Downloading kayfabe $LATEST_RELEASE..."

# Download the binary
DOWNLOAD_URL="https://github.com/ShreyeshArangath/kayfabe/releases/download/${LATEST_RELEASE}/${ASSET_NAME}.tar.gz"
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

cd "$TEMP_DIR"
curl -fsSL "$DOWNLOAD_URL" -o kayfabe.tar.gz

if [ ! -f kayfabe.tar.gz ]; then
    echo -e "${RED}Error: Failed to download kayfabe${NC}"
    exit 1
fi

tar xzf kayfabe.tar.gz

if [ ! -f kayfabe ]; then
    echo -e "${RED}Error: Binary not found in archive${NC}"
    exit 1
fi

# Copy binary to installation directory
cp kayfabe "$INSTALL_DIR/kayfabe"
chmod +x "$INSTALL_DIR/kayfabe"

# Check if installation directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}Note: $INSTALL_DIR is not in your PATH${NC}"
    echo "Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\""
else
    echo -e "${GREEN}✓ Installation directory is in PATH${NC}"
fi

echo -e "${GREEN}✓ kayfabe $LATEST_RELEASE installed successfully!${NC}"
echo "Run 'kayfabe --help' to get started"
