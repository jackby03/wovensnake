#!/usr/bin/env bash
set -euo pipefail

# ─── Colors ────────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    BOLD=$'\033[1m'; DIM=$'\033[2m'; RESET=$'\033[0m'
    CYAN=$'\033[36m'; GREEN=$'\033[32m'; RED=$'\033[31m'; YELLOW=$'\033[33m'; BLUE=$'\033[34m'; GRAY=$'\033[90m'
else
    BOLD=''; DIM=''; RESET=''; CYAN=''; GREEN=''; RED=''; YELLOW=''; BLUE=''; GRAY=''
fi

REPO="jackby03/wovensnake"
DEFAULT_INSTALL_DIR="$HOME/.wovensnake/bin"
EXE_NAME="woven"
INSTALL_DIR="$DEFAULT_INSTALL_DIR"
YES=false
NO_PATH=false

# ─── Args ───────────────────────────────────────────────────────────────────────
for arg in "$@"; do
    case "$arg" in
        -y|--yes)            YES=true ;;
        --no-modify-path)    NO_PATH=true ;;
        --install-dir=*)     INSTALL_DIR="${arg#*=}" ;;
        -h|--help)
            echo "Usage: install.sh [options]"
            echo "  -y, --yes              Non-interactive (accept all defaults)"
            echo "  --no-modify-path       Don't modify shell config"
            echo "  --install-dir=PATH     Custom install directory"
            exit 0 ;;
    esac
done

# ─── Helpers ───────────────────────────────────────────────────────────────────
info()    { printf "  ${CYAN}•${RESET} %s\n" "$*"; }
success() { printf "  ${GREEN}✓${RESET} %s\n" "$*"; }
warn()    { printf "  ${YELLOW}!${RESET} %s\n" "$*"; }
fail()    { printf "  ${RED}✗${RESET} %s\n" "$*" >&2; exit 1; }
step()    { printf "\n${BOLD}${CYAN}[%s]${RESET} %s\n" "$1" "$2"; }

confirm() {
    local prompt="$1" default="${2:-y}"
    if $YES; then return 0; fi
    local hint
    [ "$default" = "y" ] && hint="${GREEN}Y${RESET}/n" || hint="y/${GREEN}N${RESET}"
    printf "  ${BOLD}?${RESET} %s [%b] " "$prompt" "$hint"
    read -r answer </dev/tty
    answer="${answer:-$default}"
    [[ "$answer" =~ ^[Yy]$ ]]
}

spinner() {
    local pid=$1 msg="$2"
    local frames='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
    local i=0
    while kill -0 "$pid" 2>/dev/null; do
        local frame="${frames:$((i % ${#frames})):1}"
        printf "\r  ${CYAN}%s${RESET} %s..." "$frame" "$msg"
        sleep 0.1
        i=$((i + 1))
    done
    printf "\r"
}

# ─── Banner ────────────────────────────────────────────────────────────────────
printf "\n"
printf "${BOLD}${CYAN}"
printf "   ╦ ╦╔═╗╦  ╦╔═╗╔╗╔╔═╗╔╗╔╔═╗╦╔═╔═╗\n"
printf "   ║║║║ ║╚╗╔╝║╣ ║║║╚═╗║║║╠═╣╠╩╗║╣ \n"
printf "   ╚╩╝╚═╝ ╚╝ ╚═╝╝╚╝╚═╝╝╚╝╩ ╩╩ ╩╚═╝\n"
printf "${RESET}"
printf "   ${DIM}Dependencies, neatly woven.${RESET}\n"
printf "   ${GRAY}────────────────────────────────${RESET}\n\n"

# ─── 1. Detect Platform ────────────────────────────────────────────────────────
step "1/5" "Detecting your system"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        OS_NAME="Linux"
        case "$ARCH" in
            x86_64)  ASSET="woven-linux-amd64";  ARCH_NAME="x86_64" ;;
            aarch64) ASSET="woven-linux-aarch64"; ARCH_NAME="arm64" ;;
            *)       fail "Unsupported architecture: $ARCH" ;;
        esac ;;
    Darwin*)
        OS_NAME="macOS"
        case "$ARCH" in
            arm64)   ASSET="woven-macos-arm64";  ARCH_NAME="Apple Silicon (arm64)" ;;
            x86_64)  ASSET="woven-macos-amd64";  ARCH_NAME="Intel (x86_64)" ;;
            *)       fail "Unsupported architecture: $ARCH" ;;
        esac ;;
    *)
        fail "Unsupported OS: $OS. Only Linux and macOS are supported." ;;
esac

info "OS:   ${BOLD}$OS_NAME${RESET}"
info "Arch: ${BOLD}$ARCH_NAME${RESET}"
info "Dir:  ${BOLD}$INSTALL_DIR${RESET}"

# ─── 2. Confirm ────────────────────────────────────────────────────────────────
step "2/5" "Confirm installation"

if ! confirm "Install woven to $INSTALL_DIR?"; then
    printf "\n  ${GRAY}Installation cancelled.${RESET}\n\n"
    exit 0
fi

# ─── 3. Resolve Asset URL ──────────────────────────────────────────────────────
step "3/5" "Resolving download"

BASE_URL="https://github.com/$REPO/releases/latest/download"
URL="$BASE_URL/$ASSET"

# On macOS arm64: check if native binary exists, fall back to amd64 (Rosetta 2)
if [ "$ARCH" = "arm64" ] && [ "$OS" = "Darwin" ]; then
    HTTP_CODE="$(curl -o /dev/null -s -w "%{http_code}" -L "$URL" 2>/dev/null || echo "000")"
    if [ "$HTTP_CODE" != "200" ]; then
        warn "Native arm64 binary not available yet (HTTP $HTTP_CODE)"
        warn "Falling back to amd64 — will run via Rosetta 2"
        ASSET="woven-macos-amd64"
        URL="$BASE_URL/$ASSET"
    else
        success "Native arm64 binary found"
    fi
fi

info "Asset: ${BOLD}$ASSET${RESET}"

# ─── 4. Download & Install ─────────────────────────────────────────────────────
step "4/5" "Downloading & installing"

mkdir -p "$INSTALL_DIR"
TMP_FILE="$(mktemp)"
trap 'rm -f "$TMP_FILE"' EXIT

if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$TMP_FILE" &
    CURL_PID=$!
    spinner "$CURL_PID" "Downloading $ASSET"
    wait "$CURL_PID" || fail "Download failed. Check your connection or try again."
elif command -v wget >/dev/null 2>&1; then
    wget -q "$URL" -O "$TMP_FILE" || fail "Download failed. Check your connection or try again."
else
    fail "Neither curl nor wget found. Please install one and retry."
fi

mv "$TMP_FILE" "$INSTALL_DIR/$EXE_NAME"
chmod +x "$INSTALL_DIR/$EXE_NAME"
success "Binary installed to ${BOLD}$INSTALL_DIR/$EXE_NAME${RESET}"

# Verify binary runs
if "$INSTALL_DIR/$EXE_NAME" --help >/dev/null 2>&1; then
    success "Binary verified and working"
else
    warn "Could not verify binary (it may still work — try running 'woven --help')"
fi

# ─── 5. Setup PATH ─────────────────────────────────────────────────────────────
step "5/5" "Setting up PATH"

if $NO_PATH; then
    info "Skipping PATH modification (--no-modify-path)"
else
    case "${SHELL:-}" in
        */zsh)  CONFIG="$HOME/.zshrc" ;;
        */bash) CONFIG="$HOME/.bashrc" ;;
        *)      CONFIG="$HOME/.profile" ;;
    esac

    if grep -q "$INSTALL_DIR" "$CONFIG" 2>/dev/null; then
        success "Already in PATH ($CONFIG)"
    else
        if confirm "Add woven to PATH in $CONFIG?"; then
            printf '\n# WovenSnake\nexport PATH="$PATH:%s"\n' "$INSTALL_DIR" >> "$CONFIG"
            success "PATH updated in ${BOLD}$CONFIG${RESET}"
            info "Run ${BOLD}source $CONFIG${RESET} or open a new terminal to activate"
        else
            warn "Skipped PATH update — add manually:"
            printf "      ${GRAY}export PATH=\"\$PATH:%s\"${RESET}\n" "$INSTALL_DIR"
        fi
    fi
fi

# ─── Done ──────────────────────────────────────────────────────────────────────
printf "\n${BOLD}${GREEN}  ✨ WovenSnake installed successfully!${RESET}\n\n"
printf "  ${GRAY}Get started:${RESET}\n"
printf "    ${CYAN}woven init${RESET}              ${GRAY}# create a new project${RESET}\n"
printf "    ${CYAN}woven install requests${RESET}  ${GRAY}# add and install a package${RESET}\n"
printf "    ${CYAN}woven install${RESET}           ${GRAY}# install all dependencies${RESET}\n"
printf "    ${CYAN}woven run <cmd>${RESET}         ${GRAY}# run inside the virtual env${RESET}\n"
printf "\n"
