#!/bin/bash
# =============================================================================
# package-linux.sh — Create a .deb package for Jarvis
# =============================================================================
#
# Usage: ./scripts/package-linux.sh
#
# Creates a .deb package in target/release/.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

APP_NAME="jarvis"
ARCH="$(dpkg --print-architecture 2>/dev/null || echo amd64)"

# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [[ -z "$VERSION" ]]; then
    VERSION="0.1.0"
fi

echo "Packaging ${APP_NAME} v${VERSION} for Linux (${ARCH})..."

# Build release
cargo build --release

# Create .deb package structure
DEB_DIR="target/release/deb-staging"
rm -rf "${DEB_DIR}"
mkdir -p "${DEB_DIR}/DEBIAN"
mkdir -p "${DEB_DIR}/usr/bin"
mkdir -p "${DEB_DIR}/usr/share/applications"
mkdir -p "${DEB_DIR}/usr/share/${APP_NAME}"

# Copy binary
cp "target/release/${APP_NAME}" "${DEB_DIR}/usr/bin/"

# Copy resources
if [[ -d "resources/themes" ]]; then
    cp -r resources/themes "${DEB_DIR}/usr/share/${APP_NAME}/"
fi

# Create control file
cat > "${DEB_DIR}/DEBIAN/control" << CTRL
Package: ${APP_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libx11-6, libxcb1, libxkbcommon0, libwayland-client0, libgtk-3-0
Maintainer: Dylan <dylan@example.com>
Description: Jarvis — GPU-accelerated terminal emulator
 A GPU-accelerated terminal emulator with AI integration,
 tiling window management, social features, and voice input.
 Built with Rust using wgpu for cross-platform GPU rendering.
CTRL

# Create .desktop file
cat > "${DEB_DIR}/usr/share/applications/${APP_NAME}.desktop" << DESKTOP
[Desktop Entry]
Name=Jarvis
Comment=GPU-accelerated terminal emulator with AI integration
Exec=jarvis
Terminal=false
Type=Application
Categories=System;TerminalEmulator;
Keywords=terminal;shell;ai;tiling;
DESKTOP

# Build .deb
DEB_PATH="target/release/${APP_NAME}_${VERSION}_${ARCH}.deb"
dpkg-deb --build "${DEB_DIR}" "${DEB_PATH}"

# Clean up staging
rm -rf "${DEB_DIR}"

echo ""
echo "Built: ${DEB_PATH}"
echo "Install with: sudo dpkg -i ${DEB_PATH}"
