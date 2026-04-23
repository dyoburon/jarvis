#!/bin/bash
#
# Jarvis One-Click Setup Script
#
# This script sets up everything needed to run Jarvis:
# - Creates Python virtual environment
# - Installs Python dependencies
# - Builds the Swift/Metal app
# - Creates config directory with defaults
#
# Usage: ./scripts/setup.sh [--dev]
#

set -e  # Exit on error

# =============================================================================
# COLORS
# =============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# =============================================================================
# PRE-FLIGHT CHECKS
# =============================================================================

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Jarvis Setup Script${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo -e "${RED}Error: Jarvis currently only supports macOS${NC}"
    exit 1
fi

# Check macOS version (12.0+)
MACOS_VERSION=$(sw_vers -productVersion | cut -d. -f1)
if [[ "$MACOS_VERSION" -lt 12 ]]; then
    echo -e "${RED}Error: macOS 12.0 (Monterey) or later is required${NC}"
    exit 1
fi
echo -e "${GREEN}✓${NC} macOS $MACOS_VERSION+"

# Check Xcode Command Line Tools
if ! xcode-select -p &>/dev/null; then
    echo -e "${YELLOW}Installing Xcode Command Line Tools...${NC}"
    xcode-select --install 2>/dev/null || true
    echo -e "${YELLOW}Please restart this script after Xcode tools installation completes.${NC}"
    exit 0
fi
echo -e "${GREEN}✓${NC} Xcode Command Line Tools"

# Check Swift
if ! command -v swift &>/dev/null; then
    echo -e "${RED}Error: Swift not found. Please install Xcode.${NC}"
    exit 1
fi
SWIFT_VERSION=$(swift --version | head -1 | grep -oE '[0-9]+\.[0-9]+' | head -1)
echo -e "${GREEN}✓${NC} Swift $SWIFT_VERSION"

# Check Python 3.10+
PYTHON_CMD=""
for cmd in python3.12 python3.11 python3.10 python3; do
    if command -v "$cmd" &>/dev/null; then
        VERSION=$("$cmd" --version 2>&1 | grep -oE '[0-9]+\.[0-9]+' | head -1)
        MAJOR=$(echo "$VERSION" | cut -d. -f1)
        MINOR=$(echo "$VERSION" | cut -d. -f2)
        if [[ "$MAJOR" -eq 3 && "$MINOR" -ge 10 ]]; then
            PYTHON_CMD="$cmd"
            echo -e "${GREEN}✓${NC} Python $VERSION"
            break
        fi
    fi
done

if [[ -z "$PYTHON_CMD" ]]; then
    echo -e "${RED}Error: Python 3.10+ is required${NC}"
    echo -e "${YELLOW}Install with: brew install python@3.12${NC}"
    exit 1
fi

# =============================================================================
# SETUP DIRECTORIES
# =============================================================================

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Creating Directory Structure${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Python module directories
mkdir -p legacy/jarvis/config
mkdir -p legacy/jarvis/commands
mkdir -p legacy/jarvis/session
mkdir -p tests

# Resources
mkdir -p resources/themes

# Config directory
CONFIG_DIR="$HOME/.config/jarvis"
mkdir -p "$CONFIG_DIR"

echo -e "${GREEN}✓${NC} Directory structure created"

# =============================================================================
# PYTHON VIRTUAL ENVIRONMENT
# =============================================================================

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Setting Up Python Environment${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [[ ! -d ".venv" ]]; then
    echo -e "${YELLOW}Creating virtual environment...${NC}"
    $PYTHON_CMD -m venv .venv
    echo -e "${GREEN}✓${NC} Virtual environment created"
else
    echo -e "${GREEN}✓${NC} Virtual environment exists"
fi

# Activate venv
source .venv/bin/activate

# Upgrade pip
echo -e "${YELLOW}Upgrading pip...${NC}"
pip install --upgrade pip >/dev/null 2>&1

# Install dependencies
echo -e "${YELLOW}Installing Python dependencies...${NC}"
pip install -r legacy/requirements.txt >/dev/null 2>&1
echo -e "${GREEN}✓${NC} Python dependencies installed"

# =============================================================================
# SWIFT BUILD
# =============================================================================

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Building Swift/Metal App${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

cd legacy/metal-app

if [[ "$1" == "--dev" ]]; then
    echo -e "${YELLOW}Building debug configuration...${NC}"
    swift build 2>&1 | tail -5
else
    echo -e "${YELLOW}Building release configuration...${NC}"
    swift build -c release 2>&1 | tail -5
fi

cd ..

if [[ -f "legacy/metal-app/.build/release/JarvisBootup" ]] || [[ -f "legacy/metal-app/.build/debug/JarvisBootup" ]]; then
    echo -e "${GREEN}✓${NC} Swift app built successfully"
else
    echo -e "${RED}Error: Swift build failed${NC}"
    exit 1
fi

# =============================================================================
# CONFIGURATION
# =============================================================================

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Creating Configuration${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

CONFIG_FILE="$CONFIG_DIR/config.yaml"

if [[ ! -f "$CONFIG_FILE" ]]; then
    echo -e "${YELLOW}Creating default config...${NC}"
    # Run Python to create default config
    source .venv/bin/activate
    PYTHONPATH=legacy python3 -c "from jarvis.config.loader import load_config; load_config()" 2>/dev/null
    echo -e "${GREEN}✓${NC} Config created at $CONFIG_FILE"
else
    echo -e "${GREEN}✓${NC} Config exists at $CONFIG_FILE"
fi

# =============================================================================
# DONE
# =============================================================================

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  ✓ Setup Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "To start Jarvis:"
echo -e "  ${BLUE}./scripts/start.sh${NC}"
echo ""
echo -e "Configuration:"
echo -e "  ${BLUE}$CONFIG_FILE${NC}"
echo ""
