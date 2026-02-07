#!/usr/bin/env bash
set -euo pipefail

REPO="nash87/parkhub"
INSTALL_DIR="/usr/local/bin"
SERVICE_NAME="parkhub"
DATA_DIR="/var/lib/parkhub"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'

info()  { echo -e "${BLUE}ℹ${NC}  $*"; }
ok()    { echo -e "${GREEN}✓${NC}  $*"; }
warn()  { echo -e "${YELLOW}⚠${NC}  $*"; }
err()   { echo -e "${RED}✗${NC}  $*" >&2; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)   OS="linux" ;;
        Darwin*)  OS="macos" ;;
        MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
        *)        err "Unsupported OS: $(uname -s)"; exit 1 ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)       ARCH="x86_64" ;;
        aarch64|arm64)      ARCH="aarch64" ;;
        *)                  err "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac
}

# Get latest release tag
get_latest_version() {
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        err "Could not determine latest version"; exit 1
    fi
    info "Latest version: ${VERSION}"
}

# Download and install
install_binary() {
    local ext="tar.gz"
    [ "$OS" = "windows" ] && ext="zip"

    local filename="parkhub-${OS}-${ARCH}.${ext}"
    local url="https://github.com/${REPO}/releases/download/${VERSION}/${filename}"

    info "Downloading ${filename}..."
    local tmpdir
    tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT

    curl -fsSL -o "${tmpdir}/${filename}" "$url"

    if [ "$ext" = "tar.gz" ]; then
        tar xzf "${tmpdir}/${filename}" -C "$tmpdir"
    else
        unzip -q "${tmpdir}/${filename}" -d "$tmpdir"
    fi

    local binary="parkhub-server"
    [ "$OS" = "windows" ] && binary="parkhub-server.exe"

    # Find the binary (may be nested)
    local src
    src=$(find "$tmpdir" -name "$binary" -type f | head -1)
    if [ -z "$src" ]; then
        # Fallback: the tarball contains a differently named binary
        src=$(find "$tmpdir" -name "parkhub-server-*" -type f | head -1)
    fi

    if [ -z "$src" ]; then
        err "Binary not found in archive"; exit 1
    fi

    chmod +x "$src"

    if [ -w "$INSTALL_DIR" ]; then
        cp "$src" "${INSTALL_DIR}/parkhub-server"
    else
        info "Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo cp "$src" "${INSTALL_DIR}/parkhub-server"
        sudo chmod +x "${INSTALL_DIR}/parkhub-server"
    fi

    ok "Installed parkhub-server to ${INSTALL_DIR}/parkhub-server"
}

# Optional systemd service
setup_systemd() {
    if [ "$OS" != "linux" ] || ! command -v systemctl &>/dev/null; then
        return
    fi

    echo ""
    read -rp "Create systemd service? [y/N] " yn
    case "$yn" in
        [Yy]*)
            sudo mkdir -p "$DATA_DIR"
            sudo useradd -r -s /usr/sbin/nologin -d "$DATA_DIR" parkhub 2>/dev/null || true
            sudo chown parkhub:parkhub "$DATA_DIR"

            sudo tee /etc/systemd/system/parkhub.service > /dev/null << EOF
[Unit]
Description=ParkHub - Parking Management
After=network.target

[Service]
Type=simple
User=parkhub
Group=parkhub
ExecStart=/usr/local/bin/parkhub-server
WorkingDirectory=${DATA_DIR}
Environment=PARKHUB_DATA_DIR=${DATA_DIR}
Environment=PARKHUB_PORT=8080
Environment=RUST_LOG=info
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

            sudo systemctl daemon-reload
            sudo systemctl enable parkhub
            sudo systemctl start parkhub

            ok "Systemd service created and started"
            info "Manage with: sudo systemctl {start|stop|restart|status} parkhub"
            info "View logs:   sudo journalctl -u parkhub -f"
            ;;
    esac
}

# Main
main() {
    echo -e "${GREEN}"
    echo "  🅿️  ParkHub Installer"
    echo -e "${NC}"

    detect_os
    detect_arch
    info "Detected: ${OS}/${ARCH}"

    get_latest_version
    install_binary
    setup_systemd

    echo ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "  ${GREEN}✓ ParkHub installed successfully!${NC}"
    echo ""
    echo -e "  Start:   ${BLUE}parkhub-server${NC}"
    echo -e "  Open:    ${BLUE}http://localhost:8080${NC}"
    echo -e "  Login:   ${YELLOW}admin / admin${NC}"
    echo ""
    echo -e "  ${RED}⚠  Change your admin password immediately!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

main "$@"
