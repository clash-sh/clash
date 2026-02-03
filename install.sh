#!/bin/sh

# Clash installation script
# Usage: curl -fsSL https://clash.sh/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO="clash-sh/clash"
BINARY_NAME="clash"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        darwin)
            PLATFORM="apple-darwin"
            ;;
        linux)
            PLATFORM="unknown-linux-gnu"
            ;;
        *)
            echo "${RED}Unsupported operating system: $OS${NC}"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="aarch64"
            ;;
        *)
            echo "${RED}Unsupported architecture: $ARCH${NC}"
            exit 1
            ;;
    esac

    TARGET="${ARCH}-${PLATFORM}"
}

# Get the latest release version from GitHub
get_latest_version() {
    VERSION="${VERSION:-latest}"

    if [ "$VERSION" = "latest" ]; then
        echo "Fetching latest version..."
        VERSION=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

        if [ -z "$VERSION" ]; then
            echo "${YELLOW}No releases found. Installing from main branch...${NC}"
            install_from_source
            exit 0
        fi
    fi

    echo "Installing Clash ${VERSION}..."
}

# Download and install binary
install_binary() {
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/clash-${TARGET}.tar.gz"
    TEMP_DIR=$(mktemp -d)

    echo "Downloading from: $DOWNLOAD_URL"

    if ! curl -sL --fail "$DOWNLOAD_URL" -o "$TEMP_DIR/clash.tar.gz"; then
        echo "${YELLOW}Binary not available for $TARGET. Falling back to source installation...${NC}"
        rm -rf "$TEMP_DIR"
        install_from_source
        return
    fi

    echo "Extracting binary..."
    tar -xzf "$TEMP_DIR/clash.tar.gz" -C "$TEMP_DIR"

    echo "Installing to $INSTALL_DIR..."
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TEMP_DIR/$BINARY_NAME" "$INSTALL_DIR/"
    else
        echo "${YELLOW}Need sudo access to install to $INSTALL_DIR${NC}"
        sudo mv "$TEMP_DIR/$BINARY_NAME" "$INSTALL_DIR/"
    fi

    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    rm -rf "$TEMP_DIR"
}

# Install from source using cargo
install_from_source() {
    if ! command -v cargo >/dev/null 2>&1; then
        echo "${RED}Cargo not found. Please install Rust first:${NC}"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    echo "Installing from source using cargo..."
    cargo install --git "https://github.com/${REPO}.git" --branch main
}

# Verify installation
verify_installation() {
    if command -v clash >/dev/null 2>&1; then
        echo "${GREEN}✅ Clash installed successfully!${NC}"
        echo ""
        clash --version
        echo ""
        echo "Run 'clash --help' to get started"
        echo "Visit https://github.com/clash-sh/clash for documentation"
    else
        echo "${RED}Installation failed. Please check the error messages above.${NC}"
        exit 1
    fi
}

# Main installation flow
main() {
    echo "⚔️  Installing Clash..."
    echo ""

    detect_platform
    echo "Platform: $OS ($ARCH)"
    echo "Target: $TARGET"
    echo ""

    get_latest_version
    install_binary
    verify_installation
}

main "$@"