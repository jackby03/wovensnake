#!/bin/sh
set -e

REPO="jackby03/wovensnake"
INSTALL_DIR="$HOME/.wovensnake/bin"
EXE_NAME="woven"

echo "🧶 WovenSnake Installer"
echo "-----------------------"

# 1. Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        case "$ARCH" in
            x86_64)  ASSET="woven-linux-amd64";;
            aarch64) ASSET="woven-linux-aarch64";;
            *)       echo "Unsupported architecture: $ARCH"; exit 1;;
        esac
        ;;
    Darwin*)
        case "$ARCH" in
            arm64)   ASSET="woven-macos-arm64";;
            x86_64)  ASSET="woven-macos-amd64";;
            *)       echo "Unsupported architecture: $ARCH"; exit 1;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"; exit 1;;
esac

BASE_URL="https://github.com/$REPO/releases/latest/download"
URL="$BASE_URL/$ASSET"

# On macOS arm64: check if native arm64 binary exists (follows redirects); fall back to amd64 (Rosetta 2)
if [ "$ARCH" = "arm64" ] && [ "$OS" = "Darwin" ]; then
    HTTP_CODE=""
    if command -v curl >/dev/null 2>&1; then
        HTTP_CODE="$(curl -o /dev/null -s -w "%{http_code}" -L "$URL")"
    fi
    if [ "$HTTP_CODE" != "200" ]; then
        echo "Note: No native arm64 binary found (HTTP $HTTP_CODE), using amd64 via Rosetta 2."
        ASSET="woven-macos-amd64"
        URL="$BASE_URL/$ASSET"
    fi
fi

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
echo "Run 'woven --help' to start."
