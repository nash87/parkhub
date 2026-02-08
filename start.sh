#!/bin/bash
# ParkHub Server Wrapper - handles auto-restart after updates
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SERVER="$SCRIPT_DIR/target/release/parkhub-server"
CONFIG_DIR="${PARKHUB_DATA_DIR:-$HOME/.local/share/parkhubserver}"

while true; do
    # Fix TLS config if needed
    if [ -f "$CONFIG_DIR/config.toml" ]; then
        sed -i 's/enable_tls = true/enable_tls = false/; s/encryption_enabled = true/encryption_enabled = false/' "$CONFIG_DIR/config.toml"
    fi

    echo "[ParkHub] Starting server..."
    "$SERVER" "$@"
    EXIT_CODE=$?

    echo "[ParkHub] Server exited with code $EXIT_CODE"

    if [ $EXIT_CODE -eq 0 ]; then
        echo "[ParkHub] Clean exit (update), restarting in 2 seconds..."
        sleep 2
    elif [ $EXIT_CODE -eq 42 ]; then
        echo "[ParkHub] Update restart requested, restarting in 2 seconds..."
        sleep 2
    else
        echo "[ParkHub] Unexpected exit (code $EXIT_CODE), restarting in 5 seconds..."
        sleep 5
    fi
done
