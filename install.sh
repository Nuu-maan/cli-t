#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Installing cli-t...${NC}\n"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

# Map architecture
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

# Map OS to target triple
case "$OS" in
    Linux*) 
        TARGET="x86_64-unknown-linux-gnu"
        BINARY_NAME="cli-t"
        ;;
    Darwin*)
        if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
            TARGET="aarch64-apple-darwin"
        else
            TARGET="x86_64-apple-darwin"
        fi
        BINARY_NAME="cli-t"
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        echo "For Windows, please use install.ps1"
        exit 1
        ;;
esac

# GitHub repository
REPO="Nuu-maan/cli-t"
VERSION="latest"

# Determine download URL
if [ "$VERSION" = "latest" ]; then
    URL="https://github.com/${REPO}/releases/latest/download/cli-t-${TARGET}.tar.gz"
else
    URL="https://github.com/${REPO}/releases/download/${VERSION}/cli-t-${TARGET}.tar.gz"
fi

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

echo -e "${YELLOW}Downloading cli-t for ${TARGET}...${NC}"
if ! curl -fsSL "$URL" -o "$TMP_DIR/cli-t.tar.gz"; then
    echo -e "${RED}Failed to download. Make sure the release exists on GitHub.${NC}"
    exit 1
fi

# Extract
echo -e "${YELLOW}Extracting...${NC}"
tar -xzf "$TMP_DIR/cli-t.tar.gz" -C "$TMP_DIR"

# Determine install directory
if [ -d "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
elif [ -d "$HOME/bin" ]; then
    INSTALL_DIR="$HOME/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Install binary
echo -e "${YELLOW}Installing to ${INSTALL_DIR}...${NC}"
chmod +x "$TMP_DIR/$BINARY_NAME"
mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/cli-t"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    SHELL_RC=""
    if [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        SHELL_RC="$HOME/.bashrc"
    fi
    
    if [ -n "$SHELL_RC" ]; then
        echo "" >> "$SHELL_RC"
        echo "# cli-t" >> "$SHELL_RC"
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_RC"
        echo -e "${GREEN}Added ${INSTALL_DIR} to PATH in ${SHELL_RC}${NC}"
        echo -e "${YELLOW}Run: source ${SHELL_RC}${NC}"
    else
        echo -e "${YELLOW}Please add ${INSTALL_DIR} to your PATH manually${NC}"
    fi
fi

echo -e "\n${GREEN}✓ cli-t installed successfully!${NC}"
echo -e "${GREEN}Run 'cli-t' to start chatting${NC}\n"

