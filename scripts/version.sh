#!/usr/bin/env bash
# Automatic CalVer versioning for ParkHub
# Format: YYYY.MM.DD-N (N increments per day)

set -euo pipefail

TODAY=$(date -u +%Y.%m.%d)

# Find the highest tag number for today
LAST_N=$(git tag -l "v${TODAY}-*" 2>/dev/null | sed "s/v${TODAY}-//" | sort -n | tail -1)

if [ -z "$LAST_N" ]; then
    NEXT_N=1
else
    NEXT_N=$((LAST_N + 1))
fi

VERSION="${TODAY}-${NEXT_N}"
echo "$VERSION"
