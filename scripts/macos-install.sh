#!/usr/bin/env bash
set -euo pipefail

REPO="nash87/parkhub"
INSTALL_DIR="/usr/local/bin"
DATA_DIR="/var/lib/parkhub"
PLIST_PATH="/Library/LaunchDaemons/com.parkhub.server.plist"

GREEN='\033[0;32m'; BLUE='\033[0;34m'; YELLOW='\033[1;33m'; RED='\033[0;31m'
BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "  ${BLUE}ℹ${NC}  $*"; }
ok()    { echo -e "  ${GREEN}✓${NC}  $*"; }
warn()  { echo -e "  ${YELLOW}⚠${NC}  $*"; }
err()   { echo -e "  ${RED}✗${NC}  $*" >&2; }
step()  { echo -e "  ${BLUE}→${NC}  $*"; }

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  ARCH="amd64" ;;
        arm64|aarch64) ARCH="arm64" ;;
        *) err "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac
}

get_latest_version() {
    step "Fetching latest release..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
        | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/') || true
    if [ -z "${VERSION:-}" ]; then
        err "Could not determine latest version"
        exit 1
    fi
    ok "Latest version: ${VERSION}"
}

download_binary() {
    local asset="parkhub-macos-${ARCH}"
    local url="https://github.com/${REPO}/releases/download/${VERSION}/${asset}"
    local tmp
    tmp=$(mktemp -d)
    trap "rm -rf $tmp" EXIT

    step "Downloading ${asset}..."
    curl -fsSL -o "${tmp}/parkhub-server" "$url"
    chmod +x "${tmp}/parkhub-server"

    step "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo cp "${tmp}/parkhub-server" "${INSTALL_DIR}/parkhub-server"
    sudo chmod +x "${INSTALL_DIR}/parkhub-server"
    ok "Installed to ${INSTALL_DIR}/parkhub-server"
}

create_launchdaemon() {
    step "Creating data directory..."
    sudo mkdir -p "${DATA_DIR}"

    step "Creating LaunchDaemon plist..."
    sudo tee "${PLIST_PATH}" > /dev/null << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.parkhub.server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/parkhub-server</string>
        <string>--headless</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>WorkingDirectory</key>
    <string>/var/lib/parkhub</string>
    <key>StandardOutPath</key>
    <string>/var/log/parkhub.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/parkhub-error.log</string>
</dict>
</plist>
PLIST

    step "Loading LaunchDaemon..."
    sudo launchctl load "${PLIST_PATH}"
    ok "LaunchDaemon loaded and started"
}

get_ip() {
    ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || echo "localhost"
}

main() {
    echo ""
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo -e "  ${GREEN}     🚗  ParkHub macOS Installer  🚗${NC}"
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo ""

    detect_arch
    info "Architecture: ${ARCH}"

    get_latest_version
    download_binary
    create_launchdaemon

    local ip
    ip=$(get_ip)

    echo ""
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo -e "  ${GREEN}  ✓ ParkHub is running!${NC}"
    echo ""
    echo -e "  ${BOLD}  🚗  http://${ip}:7878${NC}"
    echo ""
    echo -e "  ${BLUE}  Manage:${NC}"
    echo -e "    sudo launchctl stop com.parkhub.server"
    echo -e "    sudo launchctl start com.parkhub.server"
    echo -e "    sudo launchctl unload ${PLIST_PATH}"
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo ""
}

main "$@"
