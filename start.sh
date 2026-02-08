#!/bin/bash
# ParkHub Server Wrapper - handles auto-restart after updates
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SERVER="$SCRIPT_DIR/target/release/parkhub-server"
CONFIG_DIR="${PARKHUB_DATA_DIR:-$HOME/.local/share/parkhubserver}"
PIDFILE="$CONFIG_DIR/parkhub.pid"

cleanup() {
    if [ -f "$PIDFILE" ]; then
        kill "$(cat "$PIDFILE")" 2>/dev/null
        rm -f "$PIDFILE"
    fi
    exit 0
}
trap cleanup SIGTERM SIGINT

while true; do
    # Fix TLS config
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        sed -i 's/enable_tls = true/enable_tls = false/' "$CONFIG_DIR/config.toml"
        sed -i 's/encryption_enabled = true/encryption_enabled = false/' "$CONFIG_DIR/config.toml"
    fi

    echo "[ParkHub] Starting server..."
    "$SERVER" &
    SERVER_PID=$!
    echo $SERVER_PID > "$PIDFILE"

    wait $SERVER_PID
    EXIT_CODE=$?
    rm -f "$PIDFILE"

    echo "[ParkHub] Server exited with code $EXIT_CODE"

    if [ $EXIT_CODE -eq 0 ] || [ $EXIT_CODE -eq 42 ]; then
        echo "[ParkHub] Restart requested, waiting 3 seconds..."
        sleep 3
    else
        echo "[ParkHub] Unexpected exit, waiting 5 seconds..."
        sleep 5
    fi
done
