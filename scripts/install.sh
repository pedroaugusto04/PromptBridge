#!/usr/bin/env bash
set -e

# PromptBridge Installation Script for Linux/macOS
# This script downloads the latest binary and sets up configuration

echo "Installing PromptBridge..."

# Detect platform and architecture
OS=$(uname -s)
ARCH=$(uname -m)

# Determine binary name based on platform
if [ "$OS" = "Linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        BINARY="promptbridge-x86_64-unknown-linux-gnu"
    elif [ "$ARCH" = "aarch64" ]; then
        BINARY="promptbridge-aarch64-unknown-linux-gnu"
    else
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
    fi
elif [ "$OS" = "Darwin" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        BINARY="promptbridge-x86_64-apple-darwin"
    elif [ "$ARCH" = "arm64" ]; then
        BINARY="promptbridge-aarch64-apple-darwin"
    else
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
    fi
else
    echo "❌ Unsupported operating system: $OS"
    exit 1
fi

# Create temporary directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Get latest release version
echo "Checking for latest release..."
LATEST_VERSION=$(curl -s https://api.github.com/repos/pedroaugusto04/PromptBridge/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
if [ -z "$LATEST_VERSION" ]; then
    echo "Warning: Could not fetch latest version from GitHub API (rate limited)"
    echo "Using fallback version: v0.2.7"
    LATEST_VERSION="v0.2.7"
fi
echo "Latest version: $LATEST_VERSION"

# Download latest release
echo "Downloading PromptBridge $BINARY..."
LATEST_URL="https://github.com/pedroaugusto04/PromptBridge/releases/download/${LATEST_VERSION}/${BINARY}.tar.gz"
echo "Downloading from: $LATEST_URL"
if ! curl -fSL "$LATEST_URL" -o "$TMP_DIR/promptbridge.tar.gz"; then
    echo "Failed to download binary from: $LATEST_URL"
    echo "   Please check if the release exists at:"
    echo "   https://github.com/pedroaugusto04/PromptBridge/releases"
    exit 1
fi

# Extract binary
echo "Extracting binary..."
tar -xzf "$TMP_DIR/promptbridge.tar.gz" -C "$TMP_DIR"

# Determine installation directory
INSTALL_DIR="/usr/local/bin"
if [ "$OS" = "Darwin" ]; then
    # On macOS, check if /usr/local/bin exists and is writable
    if [ ! -w "$INSTALL_DIR" ]; then
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi
else
    # On Linux, try /usr/local/bin first, fallback to ~/.local/bin
    if [ ! -w "$INSTALL_DIR" ]; then
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi
fi

# Install binary
echo "Installing to $INSTALL_DIR..."
if [ "$INSTALL_DIR" = "/usr/local/bin" ]; then
    sudo mv "$TMP_DIR/promptbridge" "$INSTALL_DIR/promptbridge"
else
    mv "$TMP_DIR/promptbridge" "$INSTALL_DIR/promptbridge"
    chmod +x "$INSTALL_DIR/promptbridge"
fi

# Create configuration directory
echo "Setting up configuration..."
if [ "$OS" = "Darwin" ]; then
    CONFIG_DIR="$HOME/Library/Application Support/promptbridge"
else
    CONFIG_DIR="$HOME/.config/promptbridge"
fi
mkdir -p "$CONFIG_DIR"

# Download example config if config doesn't exist
if [ ! -f "$CONFIG_DIR/promptbridge.toml" ]; then
    echo "Creating default configuration..."
    curl -fsSL "https://raw.githubusercontent.com/pedroaugusto04/PromptBridge/main/promptbridge.example.toml" -o "$CONFIG_DIR/promptbridge.toml"
else
    echo "Configuration already exists, skipping..."
fi

# Detect shell for instructions
SHELL_NAME=$(basename "$SHELL")
if [ "$SHELL_NAME" = "bash" ]; then
    CONFIG_FILE="$HOME/.bashrc"
elif [ "$SHELL_NAME" = "zsh" ]; then
    CONFIG_FILE="$HOME/.zshrc"
else
    CONFIG_FILE="$HOME/.profile"
fi

# Add to PATH if needed
NEED_SOURCE=false
case ":$PATH:" in *":$INSTALL_DIR:"*) ;;
    *)
    echo ""
    echo "Adding $INSTALL_DIR to PATH..."
    
    # Add PATH export if not already present
    if ! grep -q "export PATH=\"$INSTALL_DIR" "$CONFIG_FILE" 2>/dev/null; then
        echo "" >> "$CONFIG_FILE"
        echo "# PromptBridge" >> "$CONFIG_FILE"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$CONFIG_FILE"
        echo "Added to PATH in $CONFIG_FILE"
        NEED_SOURCE=true
    else
        echo "Already in PATH configuration"
    fi
    
    # Add to current session PATH for immediate use
    export PATH="$INSTALL_DIR:$PATH"
    ;;
esac

echo ""
echo "PromptBridge installed successfully!"
echo ""
echo "Next steps:"
echo "   1. Run: promptbridge init-config"
echo "   2. Run: promptbridge install-shortcut"
if [ "$NEED_SOURCE" = true ]; then
    echo ""
    echo "Note: PATH added to $CONFIG_FILE. Restart your terminal for permanent changes."
fi
echo ""
