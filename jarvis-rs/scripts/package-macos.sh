#!/bin/bash
# =============================================================================
# package-macos.sh — Create a macOS .app bundle for Jarvis
# =============================================================================
#
# Usage: ./scripts/package-macos.sh [--release]
#
# Creates a .app bundle in target/release/ and optionally a DMG.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

APP_NAME="Jarvis"
BINARY_NAME="jarvis"
BUNDLE_ID="com.dylan.jarvis"

# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [[ -z "$VERSION" ]]; then
    VERSION="0.1.0"
fi

echo "Packaging ${APP_NAME} v${VERSION} for macOS..."

# Build
PROFILE="${1:---release}"
if [[ "$PROFILE" == "--release" ]]; then
    cargo build --release
    BINARY_DIR="target/release"
else
    cargo build
    BINARY_DIR="target/debug"
fi

BUNDLE_DIR="${BINARY_DIR}/${APP_NAME}.app"

# Create .app structure
rm -rf "${BUNDLE_DIR}"
mkdir -p "${BUNDLE_DIR}/Contents/MacOS"
mkdir -p "${BUNDLE_DIR}/Contents/Resources"

# Copy binary
cp "${BINARY_DIR}/${BINARY_NAME}" "${BUNDLE_DIR}/Contents/MacOS/"

# Copy resources if they exist
if [[ -d "resources/themes" ]]; then
    cp -r resources/themes "${BUNDLE_DIR}/Contents/Resources/"
fi
if [[ -d "resources/fonts" ]]; then
    cp -r resources/fonts "${BUNDLE_DIR}/Contents/Resources/"
fi
if [[ -d "resources/games" ]]; then
    cp -r resources/games "${BUNDLE_DIR}/Contents/Resources/"
fi

# Generate Info.plist
cat > "${BUNDLE_DIR}/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>${BINARY_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSMicrophoneUsageDescription</key>
    <string>Jarvis uses the microphone for voice input and push-to-talk.</string>
    <key>NSCameraUsageDescription</key>
    <string>Jarvis uses the camera for screen sharing.</string>
</dict>
</plist>
PLIST

# Ad-hoc code sign
codesign --force --deep --sign - "${BUNDLE_DIR}" 2>/dev/null || true

echo ""
echo "Built: ${BUNDLE_DIR}"
echo ""

# Optionally create DMG
if command -v hdiutil &>/dev/null; then
    DMG_PATH="${BINARY_DIR}/${APP_NAME}-${VERSION}.dmg"
    hdiutil create -volname "${APP_NAME}" -srcfolder "${BUNDLE_DIR}" \
        -ov -format UDZO "${DMG_PATH}" 2>/dev/null || true
    if [[ -f "${DMG_PATH}" ]]; then
        echo "DMG: ${DMG_PATH}"
    fi
fi
