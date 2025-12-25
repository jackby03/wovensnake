#!/bin/sh
set -e

REPO="jackby03/wovensnake"
INSTALL_DIR="$HOME/.wovensnake/bin"
EXE_NAME="wovensnake"

echo "🧶 WovenSnake Installer"
echo "-----------------------"

# 1. Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux*)     ASSET="wovensnake-linux-amd64";;
    Darwin*)    ASSET="wovensnake-macos-amd64";;
    *)          echo "Unsupported OS: $OS"; exit 1;;
esac

URL="https://github.com/$REPO/releases/latest/download/$ASSET"

# 2. Prepare Directory
mkdir -p "$INSTALL_DIR"

# 3. Download
echo "Downloading $ASSET..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$INSTALL_DIR/$EXE_NAME"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$URL" -O "$INSTALL_DIR/$EXE_NAME"
else
    echo "Error: Neither curl nor wget found."
    exit 1
fi

chmod +x "$INSTALL_DIR/$EXE_NAME"
echo "Installed to $INSTALL_DIR/$EXE_NAME"

# 4. Update Shell Config
case "$SHELL" in
    */zsh) CONFIG="$HOME/.zshrc";;
    */bash) CONFIG="$HOME/.bashrc";;
    *) CONFIG="$HOME/.profile";;
esac

if ! grep -q "$INSTALL_DIR" "$CONFIG"; then
    echo "Adding to PATH in $CONFIG..."
    echo "" >> "$CONFIG"
    echo "# WovenSnake" >> "$CONFIG"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$CONFIG"
    echo "Path updated. Please run: source $CONFIG"
else
    echo "Already in PATH."
fi

echo ""
echo "✨ WovenSnake successfully installed!"
echo "Run 'wovensnake --help' to start."
