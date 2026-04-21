#!/bin/bash
set -euo pipefail

REPO="runzhen/player"
APP_NAME="QQPlayer"
INSTALL_DIR="/Applications"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
RESET='\033[0m'

info()  { printf "${BLUE}${BOLD}==>${RESET} %s\n" "$1"; }
success() { printf "${GREEN}${BOLD}==>${RESET} %s\n" "$1"; }
warn()  { printf "${YELLOW}${BOLD}warning:${RESET} %s\n" "$1"; }
error() { printf "${RED}${BOLD}error:${RESET} %s\n" "$1" >&2; exit 1; }

# --- Pre-flight checks ---

if [ "$(uname)" != "Darwin" ]; then
    error "This installer only supports macOS. For other platforms, visit https://github.com/${REPO}/releases"
fi

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    arm64)  ASSET="QQPlayer-macos-arm64.zip" ;;
    x86_64) ASSET="QQPlayer-macos-x86_64.zip" ;;
    *)      error "Unsupported architecture: $ARCH" ;;
esac

# Check for curl
command -v curl >/dev/null 2>&1 || error "curl is required but not installed."

# --- Fetch latest release ---

info "Detecting latest ${APP_NAME} release..."

LATEST_URL="https://api.github.com/repos/${REPO}/releases/latest"
RELEASE_JSON="$(curl -fsSL "$LATEST_URL" 2>/dev/null)" || error "Failed to fetch release info. Check your network or visit https://github.com/${REPO}/releases"

VERSION="$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*: *"//;s/".*//')"
DOWNLOAD_URL="$(echo "$RELEASE_JSON" | grep '"browser_download_url"' | grep "$ASSET" | head -1 | sed 's/.*: *"//;s/".*//')"

if [ -z "$DOWNLOAD_URL" ]; then
    error "Could not find ${ASSET} in the latest release. Visit https://github.com/${REPO}/releases"
fi

info "Found ${APP_NAME} ${VERSION} for ${ARCH}"

# --- Download ---

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

info "Downloading ${ASSET}..."
curl -fSL --progress-bar -o "${TMPDIR}/${ASSET}" "$DOWNLOAD_URL"

# --- Install ---

info "Installing to ${INSTALL_DIR}..."

# Remove old version if present
if [ -d "${INSTALL_DIR}/${APP_NAME}.app" ]; then
    warn "Removing existing ${APP_NAME}.app"
    rm -rf "${INSTALL_DIR}/${APP_NAME}.app"
fi

# Unzip
unzip -qo "${TMPDIR}/${ASSET}" -d "${TMPDIR}"

# Find the .app bundle
APP_BUNDLE="$(find "${TMPDIR}" -maxdepth 2 -name "${APP_NAME}.app" -type d | head -1)"
if [ -z "$APP_BUNDLE" ]; then
    error "Could not find ${APP_NAME}.app in the downloaded archive."
fi

# Move to /Applications
mv "$APP_BUNDLE" "${INSTALL_DIR}/"

# Remove macOS quarantine attribute so Gatekeeper doesn't block it
xattr -dr com.apple.quarantine "${INSTALL_DIR}/${APP_NAME}.app" 2>/dev/null || true

# --- Done ---

echo ""
success "${APP_NAME} ${VERSION} installed to ${INSTALL_DIR}/${APP_NAME}.app"
echo ""
printf "  Launch it from ${BOLD}Launchpad${RESET}, ${BOLD}Spotlight${RESET}, or run:\n"
printf "    ${BOLD}open -a ${APP_NAME}${RESET}\n"
echo ""
