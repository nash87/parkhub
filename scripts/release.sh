#!/usr/bin/env bash
# Tag and prepare a release
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VERSION=$("$SCRIPT_DIR/version.sh")

echo "📦 Releasing ParkHub v${VERSION}"

# Update VERSION file
echo "$VERSION" > "$SCRIPT_DIR/../VERSION"

# Update version in package.json (parkhub-web)
if [ -f parkhub-web/package.json ]; then
    sed -i "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" parkhub-web/package.json
fi

# Update CHANGELOG
if ! grep -q "## v${VERSION}" CHANGELOG.md 2>/dev/null; then
    DATE_HUMAN=$(date -u +%Y-%m-%d)
    sed -i "/^# Changelog/a\\\\n## v${VERSION} (${DATE_HUMAN})\n" CHANGELOG.md
fi

# Commit version bump
git add -A
git commit -m "release: v${VERSION}" --allow-empty

# Create git tag
git tag -a "v${VERSION}" -m "Release v${VERSION}"

echo "✅ Tagged v${VERSION}"
echo "Run 'git push origin main --tags' to publish"
