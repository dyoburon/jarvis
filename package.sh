#!/bin/bash
# =============================================================================
# package.sh - Build Jarvis.app and create DMG installer
# =============================================================================
#
# Usage:
#   ./package.sh [--release] [--notarize]
#
# Options:
#   --release    Build release configuration (default: debug)
#   --notarize   Submit for notarization (requires Apple Developer cert)
#   --sign       Sign with developer certificate (requires CODE_SIGN_IDENTITY env)
#
# Requirements:
#   - Xcode Command Line Tools
#   - Swift 5.9+
#   - Python 3.10+
#   - create-dmg (brew install create-dmg) or uses hdiutil
#
# Output:
#   - build/Jarvis.app
#   - build/Jarvis-{version}.dmg
#
# =============================================================================

set -e

# =============================================================================
# CONFIGURATION
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# App metadata
APP_NAME="Jarvis"
APP_VERSION="${JARVIS_VERSION:-1.0.0}"
BUNDLE_ID="com.jarvis.app"

# Build configuration
BUILD_CONFIG="debug"
SIGN_IDENTITY="${CODE_SIGN_IDENTITY:-}"
NOTARIZE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_CONFIG="release"
            shift
            ;;
        --notarize)
            NOTARIZE=true
            shift
            ;;
        --sign)
            shift
            SIGN_IDENTITY="${1:-$SIGN_IDENTITY}"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Directories
BUILD_DIR="$SCRIPT_DIR/build"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
DMG_PATH="$BUILD_DIR/${APP_NAME}-${APP_VERSION}.dmg"
RESOURCES_DIR="$SCRIPT_DIR/resources"

# Metal app paths
METAL_APP_DIR="$SCRIPT_DIR/metal-app"
SWIFT_BUILD_DIR="$METAL_APP_DIR/.build"

# =============================================================================
# LOGGING
# =============================================================================

log_info() {
    echo -e "\033[0;36m[INFO]\033[0m $1"
}

log_success() {
    echo -e "\033[0;32m[SUCCESS]\033[0m $1"
}

log_error() {
    echo -e "\033[0;31m[ERROR]\033[0m $1"
}

log_warn() {
    echo -e "\033[0;33m[WARN]\033[0m $1"
}

# =============================================================================
# PREREQUISITES
# =============================================================================

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check for Swift
    if ! command -v swift &> /dev/null; then
        log_error "Swift not found. Please install Xcode."
        exit 1
    fi
    
    # Check for Python
    if ! command -v python3 &> /dev/null; then
        log_error "Python 3 not found."
        exit 1
    fi
    
    # Check for codesign if signing
    if [[ -n "$SIGN_IDENTITY" ]]; then
        if ! command -v codesign &> /dev/null; then
            log_error "codesign not found. Install Xcode Command Line Tools."
            exit 1
        fi
    fi
    
    log_success "Prerequisites OK"
}

# =============================================================================
# BUILD SWIFT
# =============================================================================

build_swift() {
    log_info "Building Swift app ($BUILD_CONFIG)..."
    
    cd "$METAL_APP_DIR"
    
    # Build with Swift Package Manager
    if [[ "$BUILD_CONFIG" == "release" ]]; then
        swift build -c release
    else
        swift build
    fi
    
    cd "$SCRIPT_DIR"
    
    log_success "Swift build complete"
}

# =============================================================================
# CREATE APP BUNDLE
# =============================================================================

create_app_bundle() {
    log_info "Creating app bundle..."
    
    # Clean previous build
    rm -rf "$APP_BUNDLE"
    mkdir -p "$APP_BUNDLE/Contents/MacOS"
    mkdir -p "$APP_BUNDLE/Contents/Resources"
    mkdir -p "$APP_BUNDLE/Contents/Frameworks"
    
    # Copy Swift binary
    BINARY_NAME="JarvisBootup"
    BINARY_SRC="$SWIFT_BUILD_DIR/$BUILD_CONFIG/$BINARY_NAME"
    
    if [[ ! -f "$BINARY_SRC" ]]; then
        log_error "Binary not found: $BINARY_SRC"
        exit 1
    fi
    
    cp "$BINARY_SRC" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
    chmod +x "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
    
    # Copy resources (if exist)
    if [[ -d "$RESOURCES_DIR" ]]; then
        cp -R "$RESOURCES_DIR/"* "$APP_BUNDLE/Contents/Resources/" 2>/dev/null || true
    fi
    
    # Copy Python files for bundled execution
    # Note: App will use system Python or create venv on first launch
    mkdir -p "$APP_BUNDLE/Contents/Resources/python"
    cp -R "$SCRIPT_DIR/jarvis" "$APP_BUNDLE/Contents/Resources/python/" 2>/dev/null || true
    cp -R "$SCRIPT_DIR/skills" "$APP_BUNDLE/Contents/Resources/python/" 2>/dev/null || true
    cp -R "$SCRIPT_DIR/voice" "$APP_BUNDLE/Contents/Resources/python/" 2>/dev/null || true
    cp -R "$SCRIPT_DIR/presence" "$APP_BUNDLE/Contents/Resources/python/" 2>/dev/null || true
    cp "$SCRIPT_DIR/config.py" "$APP_BUNDLE/Contents/Resources/python/" 2>/dev/null || true
    cp "$SCRIPT_DIR/main.py" "$APP_BUNDLE/Contents/Resources/python/" 2>/dev/null || true
    cp "$SCRIPT_DIR/requirements.txt" "$APP_BUNDLE/Contents/Resources/python/" 2>/dev/null || true
    
    # Copy HTML game files
    for game in pinball minesweeper tetris draw doodlejump asteroids subway videoplayer chat; do
        cp "$SCRIPT_DIR/${game}.html" "$APP_BUNDLE/Contents/Resources/" 2>/dev/null || true
    done
    
    # Copy game assets
    cp -R "$SCRIPT_DIR/data" "$APP_BUNDLE/Contents/Resources/" 2>/dev/null || true
    
    # Create Info.plist
    create_info_plist
    
    # Create PkgInfo
    echo -n "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"
    
    # Copy app icon (if exists)
    if [[ -f "$RESOURCES_DIR/icon.icns" ]]; then
        cp "$RESOURCES_DIR/icon.icns" "$APP_BUNDLE/Contents/Resources/app.icns"
    else
        # Create placeholder icon using system icon
        log_warn "No app icon found at $RESOURCES_DIR/icon.icns"
    fi
    
    log_success "App bundle created: $APP_BUNDLE"
}

# =============================================================================
# CREATE INFO.PLIST
# =============================================================================

create_info_plist() {
    cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIconFile</key>
    <string>app</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$APP_VERSION</string>
    <key>CFBundleVersion</key>
    <string>$APP_VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2026 Jarvis. All rights reserved.</string>
    <key>NSMicrophoneUsageDescription</key>
    <string>Jarvis uses the microphone for voice commands and transcription.</string>
    <key>NSCameraUsageDescription</key>
    <string>Jarvis may use the camera for video chat features.</string>
    <key>SUPublicEDKey</key>
    <string>YOUR_SPARKLE_PUBLIC_KEY_HERE</string>
    <key>SUFeedURL</key>
    <string>https://your-domain.com/appcast.xml</string>
    <key>LSUIElement</key>
    <false/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
EOF
}

# =============================================================================
# SIGN APP
# =============================================================================

sign_app() {
    if [[ -z "$SIGN_IDENTITY" ]]; then
        log_info "Signing with ad-hoc signature..."
        codesign --force --deep --sign - "$APP_BUNDLE"
    else
        log_info "Signing with identity: $SIGN_IDENTITY"
        codesign --force --deep --sign "$SIGN_IDENTITY" "$APP_BUNDLE"
    fi
    
    log_success "App signed"
}

# =============================================================================
# CREATE DMG
# =============================================================================

create_dmg() {
    log_info "Creating DMG..."
    
    rm -f "$DMG_PATH"
    
    # Use create-dmg if available, otherwise hdiutil
    if command -v create-dmg &> /dev/null; then
        create-dmg \
            --volname "$APP_NAME" \
            --volicon "$RESOURCES_DIR/dmg-icon.icns" \
            --background "$RESOURCES_DIR/dmg-background.png" \
            --window-pos 200 120 \
            --window-size 800 450 \
            --icon-size 100 \
            --icon "$APP_NAME.app" 200 190 \
            --hide-extension "$APP_NAME.app" \
            --app-drop-link 600 185 \
            "$DMG_PATH" \
            "$APP_BUNDLE"
    else
        # Fallback: simple hdiutil DMG
        log_warn "create-dmg not found, using hdiutil"
        
        TMP_DMG="$BUILD_DIR/tmp.dmg"
        TMP_DIR="$BUILD_DIR/dmg_contents"
        
        rm -rf "$TMP_DIR"
        mkdir -p "$TMP_DIR"
        cp -R "$APP_BUNDLE" "$TMP_DIR/"
        
        # Create symlinks
        ln -sf /Applications "$TMP_DIR/Applications"
        
        hdiutil create -volname "$APP_NAME" \
            -srcfolder "$TMP_DIR" \
            -ov -format UDZO \
            "$DMG_PATH"
        
        rm -rf "$TMP_DIR"
    fi
    
    # Sign DMG if signing
    if [[ -n "$SIGN_IDENTITY" ]]; then
        codesign --sign "$SIGN_IDENTITY" "$DMG_PATH"
    fi
    
    log_success "DMG created: $DMG_PATH"
}

# =============================================================================
# NOTARIZATION
# =============================================================================

notarize_dmg() {
    if [[ "$NOTARIZE" != true ]]; then
        return
    fi
    
    if [[ -z "$APPLE_ID" ]] || [[ -z "$APPLE_PASSWORD" ]] || [[ -z "$TEAM_ID" ]]; then
        log_error "Notarization requires APPLE_ID, APPLE_PASSWORD, and TEAM_ID env vars"
        exit 1
    fi
    
    log_info "Submitting for notarization..."
    
    # Submit
    SUBMIT_OUTPUT=$(xcrun notarytool submit "$DMG_PATH" \
        --apple-id "$APPLE_ID" \
        --password "$APPLE_PASSWORD" \
        --team-id "$TEAM_ID" \
        --wait)
    
    if echo "$SUBMIT_OUTPUT" | grep -q "status: Accepted"; then
        log_success "Notarization accepted"
        
        # Staple
        xcrun stapler staple "$DMG_PATH"
        log_success "DMG stapled"
    else
        log_error "Notarization failed"
        echo "$SUBMIT_OUTPUT"
        exit 1
    fi
}

# =============================================================================
# MAIN
# =============================================================================

main() {
    log_info "Building $APP_NAME v$APP_VERSION ($BUILD_CONFIG)"
    
    mkdir -p "$BUILD_DIR"
    
    check_prerequisites
    build_swift
    create_app_bundle
    sign_app
    create_dmg
    notarize_dmg
    
    log_success "Build complete!"
    echo ""
    echo "App bundle: $APP_BUNDLE"
    echo "DMG installer: $DMG_PATH"
    echo ""
    echo "To install: open \"$DMG_PATH\""
}

main "$@"
