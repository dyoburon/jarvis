#!/bin/bash
# jarvis start — boots up Jarvis
set -e
cd "$(dirname "$0")"

# Create venv if it doesn't exist
if [ ! -d ".venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv .venv
fi

source .venv/bin/activate

# Install/update dependencies
pip install -q -r requirements.txt

# Build Metal app if binary is missing or sources changed
METAL_BIN="metal-app/.build/debug/JarvisBootup"
if [ ! -f "$METAL_BIN" ] || [ -n "$(find metal-app/Sources -newer "$METAL_BIN" 2>/dev/null)" ]; then
    echo "Building Metal app..."
    cd metal-app && swift build 2>&1 | tail -1 && cd ..
fi

python main.py
