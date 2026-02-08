#!/usr/bin/env bash
set -euo pipefail

VERSION="1.0.0"
REPO="nash87/parkhub"
INSTALL_DIR="/usr/local/bin"
DEFAULT_PORT=7878
DEFAULT_DATA_DIR="$HOME/.local/share/parkhubserver"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'
BOLD='\033[1m'; DIM='\033[2m'; NC='\033[0m'; CYAN='\033[0;36m'

info()    { echo -e "  ${BLUE}ℹ${NC}  $*"; }
ok()      { echo -e "  ${GREEN}✓${NC}  $*"; }
warn()    { echo -e "  ${YELLOW}⚠${NC}  $*"; }
err()     { echo -e "  ${RED}✗${NC}  $*" >&2; }
step()    { echo -e "  ${CYAN}→${NC}  $*"; }
header()  { echo -e "\n  ${BOLD}$*${NC}\n"; }

# Progress spinner
spinner() {
    local pid=$1 msg=$2
    local frames=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
    local i=0
    while kill -0 "$pid" 2>/dev/null; do
        printf "\r  ${BLUE}${frames[$i]}${NC}  %s" "$msg"
        i=$(( (i + 1) % ${#frames[@]} ))
        sleep 0.1
    done
    printf "\r"
}

# Detect IP
detect_ip() {
    local ip=""
    if command -v hostname &>/dev/null; then
        ip=$(hostname -I 2>/dev/null | awk '{print $1}')
    fi
    if [ -z "$ip" ] && command -v ip &>/dev/null; then
        ip=$(ip route get 1.1.1.1 2>/dev/null | awk '{print $7; exit}')
    fi
    if [ -z "$ip" ]; then
        ip="localhost"
    fi
    echo "$ip"
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)   OS="linux" ;;
        Darwin*)  OS="macos" ;;
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

# Check dependencies
check_deps() {
    local missing=()
    for cmd in curl tar; do
        if ! command -v "$cmd" &>/dev/null; then
            missing+=("$cmd")
        fi
    done
    if [ ${#missing[@]} -gt 0 ]; then
        err "Missing dependencies: ${missing[*]}"
        info "Install them first:"
        if [ "$OS" = "linux" ]; then
            info "  sudo apt install ${missing[*]}  (Debian/Ubuntu)"
            info "  sudo dnf install ${missing[*]}  (Fedora)"
        else
            info "  brew install ${missing[*]}"
        fi
        exit 1
    fi
    ok "Dependencies OK"
}

# Get latest release
get_latest_version() {
    step "Fetching latest release..."
    RELEASE_VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/') || true
    if [ -z "${RELEASE_VERSION:-}" ]; then
        RELEASE_VERSION="v${VERSION}"
        warn "Could not fetch latest version, using ${RELEASE_VERSION}"
    else
        ok "Latest version: ${RELEASE_VERSION}"
    fi
}

# Download and install binary
install_binary() {
    local ext="tar.gz"
    local filename="parkhub-${OS}-${ARCH}.${ext}"
    local url="https://github.com/${REPO}/releases/download/${RELEASE_VERSION}/${filename}"

    step "Downloading ${filename}..."
    local tmpdir
    tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT

    if curl -fsSL -o "${tmpdir}/${filename}" "$url" 2>/dev/null; then
        ok "Downloaded"
    else
        warn "Download failed (release may not exist yet)"
        warn "Building from source would be needed for development"
        return 1
    fi

    tar xzf "${tmpdir}/${filename}" -C "$tmpdir"

    local binary="parkhub-server"
    local src
    src=$(find "$tmpdir" -name "$binary" -type f | head -1)
    if [ -z "$src" ]; then
        src=$(find "$tmpdir" -name "parkhub-server-*" -type f | head -1)
    fi

    if [ -z "$src" ]; then
        err "Binary not found in archive"
        return 1
    fi

    chmod +x "$src"

    if [ -w "$INSTALL_DIR" ]; then
        cp "$src" "${INSTALL_DIR}/parkhub-server"
    else
        step "Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo cp "$src" "${INSTALL_DIR}/parkhub-server"
        sudo chmod +x "${INSTALL_DIR}/parkhub-server"
    fi

    ok "Installed parkhub-server to ${INSTALL_DIR}/parkhub-server"
}

# Create default config
create_config() {
    local port="${1:-$DEFAULT_PORT}"
    local data_dir="${2:-$DEFAULT_DATA_DIR}"
    local config_file="${data_dir}/config.toml"

    mkdir -p "$data_dir"

    if [ ! -f "$config_file" ]; then
        cat > "$config_file" << EOF
# ParkHub Configuration
[server]
port = ${port}
host = "0.0.0.0"

[storage]
data_dir = "${data_dir}"

[auth]
session_timeout = 86400
EOF
        ok "Created config at ${config_file}"
    else
        info "Config already exists at ${config_file}"
    fi
}

# Apply custom config
apply_custom_config() {
    local port="$1" data_dir="$2" tls="$3" admin_user="$4" admin_pass="$5"
    local use_case="$6" org_name="$7" self_reg="$8" demo_data="$9"
    local config_file="${data_dir}/config.toml"

    mkdir -p "$data_dir"

    cat > "$config_file" << EOF
# ParkHub Configuration (Custom)
[server]
port = ${port}
host = "0.0.0.0"
EOF

    if [ "$tls" = "y" ]; then
        cat >> "$config_file" << EOF

[tls]
enabled = true
cert_file = "${data_dir}/cert.pem"
key_file = "${data_dir}/key.pem"
EOF
    fi

    cat >> "$config_file" << EOF

[storage]
data_dir = "${data_dir}"

[auth]
session_timeout = 86400
allow_registration = ${self_reg}

[setup]
admin_username = "${admin_user}"
organization_name = "${org_name}"
use_case = "${use_case}"
load_demo_data = ${demo_data}
EOF

    ok "Created custom config at ${config_file}"
}

# Generate random password
gen_password() {
    tr -dc 'A-Za-z0-9!@#$%' </dev/urandom | head -c 16 2>/dev/null || echo "ParkHub$(date +%s)"
}

# Start server
start_server() {
    local port="$1" data_dir="$2"
    local ip
    ip=$(detect_ip)

    local protocol="http"
    local url="${protocol}://${ip}:${port}"

    if command -v parkhub-server &>/dev/null; then
        step "Starting ParkHub server..."
        PARKHUB_DATA_DIR="$data_dir" PARKHUB_PORT="$port" nohup parkhub-server > "${data_dir}/parkhub.log" 2>&1 &
        local pid=$!
        sleep 2

        if kill -0 "$pid" 2>/dev/null; then
            ok "ParkHub is running (PID: ${pid})"
            echo "$pid" > "${data_dir}/parkhub.pid"
        else
            warn "Server may have failed to start. Check ${data_dir}/parkhub.log"
        fi
    else
        warn "parkhub-server not found in PATH"
        info "Start manually after building: cargo run --release"
    fi

    echo ""
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo -e "  ${GREEN}  ✓ ParkHub is ready!${NC}"
    echo ""
    echo -e "  ${BOLD}  🚗  ${url}${NC}"
    echo ""
    echo -e "  ${BLUE}  Open this URL in your browser to start${NC}"
    echo -e "  ${BLUE}  the onboarding wizard.${NC}"
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo ""
}

# Quick Start
quick_start() {
    header "🚀 Quick Start"

    detect_os
    detect_arch
    info "Detected: ${OS}/${ARCH}"
    check_deps
    get_latest_version
    install_binary || true
    create_config "$DEFAULT_PORT" "$DEFAULT_DATA_DIR"
    start_server "$DEFAULT_PORT" "$DEFAULT_DATA_DIR"
    setup_service "$DEFAULT_DATA_DIR" "$DEFAULT_PORT"
}

# Custom Installation
custom_install() {
    header "⚙️  Custom Installation"

    detect_os
    detect_arch
    info "Detected: ${OS}/${ARCH}"
    check_deps
    echo ""

    # Port
    read -rp "  Port [${DEFAULT_PORT}]: " port
    port="${port:-$DEFAULT_PORT}"

    # Data directory
    read -rp "  Data directory [${DEFAULT_DATA_DIR}]: " data_dir
    data_dir="${data_dir:-$DEFAULT_DATA_DIR}"

    # TLS
    read -rp "  Enable TLS? [y/N]: " tls
    tls="${tls:-n}"
    tls=$(echo "$tls" | tr '[:upper:]' '[:lower:]')

    # Admin username
    read -rp "  Admin username [admin]: " admin_user
    admin_user="${admin_user:-admin}"

    # Admin password
    local default_pass
    default_pass=$(gen_password)
    read -rsp "  Admin password [auto-generate]: " admin_pass
    echo ""
    if [ -z "$admin_pass" ]; then
        admin_pass="$default_pass"
        info "Generated password: ${YELLOW}${admin_pass}${NC}"
    fi

    # Use case
    echo ""
    echo -e "  ${BOLD}Use-case type:${NC}"
    echo "    [1] Corporate"
    echo "    [2] Residential"
    echo "    [3] Family"
    echo "    [4] Rental"
    echo "    [5] Public"
    read -rp "  Your choice [1]: " use_case_num
    use_case_num="${use_case_num:-1}"
    case "$use_case_num" in
        1) use_case="corporate" ;;
        2) use_case="residential" ;;
        3) use_case="family" ;;
        4) use_case="rental" ;;
        5) use_case="public" ;;
        *) use_case="corporate" ;;
    esac

    # Organization name
    read -rp "  Organization name [My Parking]: " org_name
    org_name="${org_name:-My Parking}"

    # Self-registration
    read -rp "  Enable self-registration? [Y/n]: " self_reg
    self_reg="${self_reg:-y}"
    if [[ "$self_reg" =~ ^[Yy] ]]; then self_reg="true"; else self_reg="false"; fi

    # Demo data
    read -rp "  Load demo data? [y/N]: " demo_data
    demo_data="${demo_data:-n}"
    if [[ "$demo_data" =~ ^[Yy] ]]; then demo_data="true"; else demo_data="false"; fi

    echo ""
    header "📋 Summary"
    info "Port:              ${port}"
    info "Data directory:    ${data_dir}"
    info "TLS:               ${tls}"
    info "Admin:             ${admin_user}"
    info "Use-case:          ${use_case}"
    info "Organization:      ${org_name}"
    info "Self-registration: ${self_reg}"
    info "Demo data:         ${demo_data}"
    echo ""

    read -rp "  Proceed? [Y/n]: " confirm
    confirm="${confirm:-y}"
    if [[ ! "$confirm" =~ ^[Yy] ]]; then
        warn "Aborted."
        exit 0
    fi

    get_latest_version
    install_binary || true
    apply_custom_config "$port" "$data_dir" "$tls" "$admin_user" "$admin_pass" \
                        "$use_case" "$org_name" "$self_reg" "$demo_data"
    start_server "$port" "$data_dir"
    setup_service "$data_dir" "$port"

    echo -e "  ${BOLD}Admin credentials:${NC}"
    echo -e "    Username: ${CYAN}${admin_user}${NC}"
    echo -e "    Password: ${YELLOW}${admin_pass}${NC}"
    echo ""
    echo -e "  ${RED}⚠  Save these credentials! Change password after first login.${NC}"
    echo ""
}

# Setup systemd service (optional, asked after install)
setup_systemd() {
    if [ "$OS" != "linux" ] || ! command -v systemctl &>/dev/null; then
        return
    fi

    echo ""
    read -rp "  Create systemd service? [y/N]: " yn
    case "$yn" in
        [Yy]*)
            local data_dir="${1:-$DEFAULT_DATA_DIR}"
            local port="${2:-$DEFAULT_PORT}"
            sudo mkdir -p "$data_dir"
            sudo useradd -r -s /usr/sbin/nologin -d "$data_dir" parkhub 2>/dev/null || true
            sudo chown parkhub:parkhub "$data_dir"

            sudo tee /etc/systemd/system/parkhub.service > /dev/null << EOF
[Unit]
Description=ParkHub - Parking Management
After=network.target

[Service]
Type=simple
User=parkhub
Group=parkhub
ExecStart=/usr/local/bin/parkhub-server
WorkingDirectory=${data_dir}
Environment=PARKHUB_DATA_DIR=${data_dir}
Environment=PARKHUB_PORT=${port}
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
            info "Manage: sudo systemctl {start|stop|restart|status} parkhub"
            info "Logs:   sudo journalctl -u parkhub -f"
            ;;
    esac
}

# Main

# Setup launchd service (optional, asked after install on macOS)
setup_launchd() {
    if [ "$OS" != "macos" ]; then
        return
    fi

    echo ""
    read -rp "  Create launchd service (auto-start on boot)? [y/N]: " yn
    case "$yn" in
        [Yy]*)
            local data_dir="${1:-$DEFAULT_DATA_DIR}"
            sudo mkdir -p /var/lib/parkhub

            sudo tee /Library/LaunchDaemons/com.parkhub.server.plist > /dev/null << PLIST
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

            sudo launchctl load /Library/LaunchDaemons/com.parkhub.server.plist

            ok "LaunchDaemon created and loaded"
            info "Manage: sudo launchctl {start|stop} com.parkhub.server"
            info "Logs:   tail -f /var/log/parkhub.log"
            ;;
    esac
}

# Offer service installation based on OS
setup_service() {
    setup_systemd "$@"
    setup_launchd "$@"
}
main() {
    clear 2>/dev/null || true
    echo ""
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo -e "  ${GREEN}     🚗  ParkHub Installer v${VERSION}  🚗${NC}"
    echo -e "  ${GREEN}═══════════════════════════════════════════${NC}"
    echo ""
    echo -e "  Choose installation mode:"
    echo ""
    echo -e "    ${BOLD}[1]${NC} Quick Start ${DIM}(recommended)${NC}"
    echo -e "        ${DIM}Default settings, ready in 2 minutes${NC}"
    echo ""
    echo -e "    ${BOLD}[2]${NC} Custom Installation"
    echo -e "        ${DIM}Configure settings before first start${NC}"
    echo ""
    echo -e "    ${BOLD}[q]${NC} Quit"
    echo ""
    read -rp "  Your choice [1]: " choice
    choice="${choice:-1}"

    case "$choice" in
        1)  quick_start ;;
        2)  custom_install ;;
        q|Q) echo ""; info "Bye! 👋"; exit 0 ;;
        *)  err "Invalid choice"; exit 1 ;;
    esac
}

main "$@"
