#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Installing cli-t...${NC}\n"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo is not installed.${NC}"
    echo -e "${YELLOW}Please install Rust from https://rustup.rs/ first.${NC}"
    echo -e "${YELLOW}Or wait for pre-built releases to be available.${NC}"
    exit 1
fi

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

# Map architecture
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

# GitHub repository
REPO="Nuu-maan/cli-t"

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

echo -e "${YELLOW}Building cli-t from source...${NC}"

# Clone repository
echo -e "${YELLOW}Cloning repository...${NC}"
cd "$TMP_DIR"
git clone --depth 1 https://github.com/${REPO}.git cli-t-src || {
    echo -e "${RED}Failed to clone repository.${NC}"
    exit 1
}

cd cli-t-src/client

# Build
echo -e "${YELLOW}Building client (this may take a few minutes)...${NC}"
cargo build --release || {
    echo -e "${RED}Build failed.${NC}"
    exit 1
}

BINARY_PATH="$TMP_DIR/cli-t-src/target/release/cli-t"
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Binary not found after build.${NC}"
    exit 1
fi

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
chmod +x "$BINARY_PATH"
cp "$BINARY_PATH" "$INSTALL_DIR/cli-t"

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
