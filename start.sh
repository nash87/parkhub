#!/bin/bash
cd /home/florian/parkhub
CONFIG=~/.local/share/parkhubserver/config.toml
if [ -f "$CONFIG" ]; then
  sed -i 's/enable_tls = true/enable_tls = false/; s/encryption_enabled = true/encryption_enabled = false/' "$CONFIG"
fi
exec ./target/release/parkhub-server "$@"
