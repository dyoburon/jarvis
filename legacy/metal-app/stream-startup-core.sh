#!/bin/bash

# Stream Startup Core — Service launcher
# This script is called by the Jarvis boot-up app (which captures stdout
# for the HUD display) or directly by stream-startup.sh --skip-jarvis.
# Background process output is suppressed so only status lines appear.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BASE="/Users/dylan/Desktop/projects/music-player"

# Validate YouTube token
echo "Validating YouTube token..."
cd "$BASE/great-firewall" && node validate-token.js 2>/dev/null
if [ $? -ne 0 ]; then
  echo "Token validation FAILED"
  exit 1
fi
echo "Token validated                              OK"
echo ""

# Clean up existing processes
echo "Cleaning up existing processes..."
"$SCRIPT_DIR/stream-teardown.sh" > /dev/null 2>&1
echo "Cleanup complete                             OK"
echo ""
echo "Starting stream services..."

# 1. OBS Audio Bridge
echo "[1/6] OBS Audio Bridge..."
cd "$BASE/obs-audio-bridge" && npm start > /dev/null 2>&1 &
sleep 1
echo "       OBS Audio Bridge                      ONLINE"

# 2. Great Firewall
echo "[2/6] Great Firewall..."
cd "$BASE/great-firewall" && npm start > /dev/null 2>&1 &
sleep 2
echo "       Great Firewall                        ONLINE"

# 3. Chat Monitor
echo "[3/6] Chat Monitor..."
cd "$BASE/chat-monitor" && npm start > /dev/null 2>&1 &
sleep 1
echo "       Chat Monitor                          ONLINE"

# 4. Firewall Monitor
echo "[4/6] Firewall Monitor..."
cd "$BASE/great-firewall/firewall-monitor" && npm start > /dev/null 2>&1 &
sleep 1
echo "       Firewall Monitor                      ONLINE"

# 5. Electron Music Player
echo "[5/6] Electron Music Player..."
cd "$BASE/electron-player" && npm run dev > /dev/null 2>&1 &
sleep 1
echo "       Electron Music Player                 ONLINE"

# 6. VibeToText
echo "[6/6] VibeToText..."
cd /Users/dylan/Desktop/projects/vibetotext && source .venv/bin/activate && python -m vibetotext > /dev/null 2>&1 &
sleep 1
echo "       VibeToText                            ONLINE"

echo ""
echo "Waiting for services to initialize..."
sleep 2

# Refresh OBS browser caches
echo "Refreshing OBS browser caches..."
"$BASE/obs-cache-refresh/.venv/bin/python3" -c "
import obsws_python as obs
cl = obs.ReqClient(host='localhost', port=4455, password='')
for src in ['chat', 'feed']:
    cl.press_input_properties_button(src, 'refreshnocache')
" 2>/dev/null && echo "       OBS caches refreshed                   OK" \
             || echo "       OBS caches skipped (not running)"

echo ""

# Validate
"$SCRIPT_DIR/stream-validate.sh" 2>/dev/null
echo ""
echo "All systems operational."
