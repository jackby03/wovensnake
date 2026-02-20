#!/usr/bin/env bash
set -euo pipefail

# ─── Colors ────────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    BOLD='\033[1m'; DIM='\033[2m'; RESET='\033[0m'
    CYAN='\033[36m'; GREEN='\033[32m'; RED='\033[31m'; YELLOW='\033[33m'; GRAY='\033[90m'
else
    BOLD=''; DIM=''; RESET=''; CYAN=''; GREEN=''; RED=''; YELLOW=''; GRAY=''
fi

YES=false

for arg in "$@"; do
    case "$arg" in
        -y|--yes) YES=true ;;
        -h|--help)
            echo "Usage: uninstall.sh [-y|--yes]"
            echo "  -y, --yes   Skip confirmation prompts"
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
    local prompt="$1"
    if $YES; then return 0; fi
    printf "  ${BOLD}?${RESET} %s [y/N] " "$prompt"
    read -r answer </dev/tty
    [[ "${answer:-n}" =~ ^[Yy]$ ]]
}

# ─── Banner ────────────────────────────────────────────────────────────────────
printf "\n"
printf "${BOLD}${CYAN}"
printf "   ╦ ╦╔═╗╦  ╦╔═╗╔╗╔╔═╗╔╗╔╔═╗╦╔═╔═╗\n"
printf "   ║║║║ ║╚╗╔╝║╣ ║║║╚═╗║║║╠═╣╠╩╗║╣ \n"
printf "   ╚╩╝╚═╝ ╚╝ ╚═╝╝╚╝╚═╝╝╚╝╩ ╩╩ ╩╚═╝\n"
printf "${RESET}"
printf "   ${DIM}Uninstaller${RESET}\n"
printf "   ${GRAY}────────────────────────────────${RESET}\n\n"

# ─── 1. Find binary ────────────────────────────────────────────────────────────
step "1/3" "Locating installation"

BINARY=""
SEARCH_PATHS=(
    "$HOME/.wovensnake/bin/woven"
    "$HOME/.local/bin/woven"
    "/usr/local/bin/woven"
    "/usr/bin/woven"
)

# Also check what's currently in PATH
if command -v woven >/dev/null 2>&1; then
    SEARCH_PATHS=("$(command -v woven)" "${SEARCH_PATHS[@]}")
fi

for path in "${SEARCH_PATHS[@]}"; do
    if [ -f "$path" ]; then
        BINARY="$path"
        break
    fi
done

DATA_DIR="$HOME/.wovensnake"

if [ -n "$BINARY" ]; then
    info "Binary found: ${BOLD}$BINARY${RESET}"
else
    warn "woven binary not found in standard locations."
    warn "The global data directory may still exist."
fi

if [ -d "$DATA_DIR" ]; then
    info "Data dir:     ${BOLD}$DATA_DIR${RESET}"
fi

# ─── 2. Confirm ────────────────────────────────────────────────────────────────
step "2/3" "Confirm removal"

if [ -z "$BINARY" ] && [ ! -d "$DATA_DIR" ]; then
    printf "\n  ${GRAY}Nothing to uninstall.${RESET}\n\n"
    exit 0
fi

if ! confirm "Remove WovenSnake from this machine?"; then
    printf "\n  ${GRAY}Uninstall cancelled.${RESET}\n\n"
    exit 0
fi

# ─── 3. Remove ─────────────────────────────────────────────────────────────────
step "3/3" "Removing WovenSnake"

# Remove binary
if [ -n "$BINARY" ] && [ -f "$BINARY" ]; then
    rm -f "$BINARY"
    success "Removed binary: $BINARY"
    # Remove parent dir if empty and it's our bin dir
    BINARY_DIR="$(dirname "$BINARY")"
    if [ "$(basename "$BINARY_DIR")" = "bin" ] && [ -z "$(ls -A "$BINARY_DIR" 2>/dev/null)" ]; then
        rmdir "$BINARY_DIR" 2>/dev/null || true
    fi
fi

# Remove data directory (~/.wovensnake — cache, managed Pythons, etc.)
# Only remove if it won't delete the binary we just removed's parent
if [ -d "$DATA_DIR" ]; then
    rm -rf "$DATA_DIR"
    success "Removed data directory: $DATA_DIR"
fi

# Clean PATH entries from shell rc files
RC_FILES=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile" "$HOME/.bash_profile")
for rc in "${RC_FILES[@]}"; do
    if [ ! -f "$rc" ]; then continue; fi
    if grep -q "wovensnake\|\.wovensnake/bin" "$rc" 2>/dev/null; then
        # Remove lines containing our PATH entry and the preceding # WovenSnake comment
        local_tmp="$(mktemp)"
        grep -v "wovensnake\|\.wovensnake/bin\|# WovenSnake" "$rc" > "$local_tmp" || true
        mv "$local_tmp" "$rc"
        success "Cleaned PATH entry from $rc"
    fi
done

# ─── Done ──────────────────────────────────────────────────────────────────────
printf "\n${BOLD}${GREEN}  ✓ WovenSnake has been uninstalled.${RESET}\n"
printf "  ${GRAY}Restart your terminal to apply PATH changes.${RESET}\n\n"
