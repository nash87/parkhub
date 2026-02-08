#!/usr/bin/env bash
set -euo pipefail

REPO="nash87/parkhub"
INSTALL_DIR="/usr/local/bin"
DATA_DIR="/var/lib/parkhub"
SERVICE_FILE="/etc/systemd/system/parkhub.service"

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
        aarch64|arm64) ARCH="arm64" ;;
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
    local asset="parkhub-linux-${ARCH}"
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

create_systemd_service() {
    step "Creating parkhub user..."
    sudo useradd -r -s /usr/sbin/nologin -d "${DATA_DIR}" parkhub 2>/dev/null || true

    step "Creating data directory..."
    sudo mkdir -p "${DATA_DIR}"
    sudo chown parkhub:parkhub "${DATA_DIR}"

    step "Creating systemd unit..."
    sudo tee "${SERVICE_FILE}" > /dev/null << EOF
[Unit]
Description=ParkHub Parking Server
After=network.target

[Service]
Type=simple
User=parkhub
Group=parkhub
ExecStart=/usr/local/bin/parkhub-server --headless --unattended
WorkingDirectory=${DATA_DIR}
Environment=RUST_LOG=info
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

    step "Enabling and starting service..."
    sudo systemctl daemon-reload
    sudo systemctl enable parkhub
    sudo systemctl start parkhub
    ok "systemd service created and started"
}

get_ip() {
    hostname -I 2>/dev/null | awk '{print $1}' || ip route get 1.1.1.1 2>/dev/null | awk '{print $7; exit}' || echo "localhost"
}

main() {
    echo ""
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo -e "  ${GREEN}     🚗  ParkHub Linux Installer  🚗${NC}"
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo ""

    detect_arch
    info "Architecture: ${ARCH}"

    get_latest_version
    download_binary
    create_systemd_service

    local ip
    ip=$(get_ip)

    echo ""
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo -e "  ${GREEN}  ✓ ParkHub is running!${NC}"
    echo ""
    echo -e "  ${BOLD}  🚗  http://${ip}:7878${NC}"
    echo ""
    echo -e "  ${BLUE}  Manage:${NC}"
    echo -e "    sudo systemctl {start|stop|restart|status} parkhub"
    echo -e "    sudo journalctl -u parkhub -f"
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo ""
}

main "$@"
