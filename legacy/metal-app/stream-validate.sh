#!/bin/bash

# Stream Validation Script
# Checks if all streaming services are running

echo "Checking stream services..."
echo ""

FAILED=0

# 1. OBS Audio Bridge (port 3456)
if lsof -i :3456 >/dev/null 2>&1; then
  echo "✅ OBS Audio Bridge    (port 3456)"
else
  echo "❌ OBS Audio Bridge    (port 3456) - NOT RUNNING"
  FAILED=$((FAILED + 1))
fi

# 2. Great Firewall (port 3457)
if lsof -i :3457 >/dev/null 2>&1; then
  echo "✅ Great Firewall      (port 3457)"
else
  echo "❌ Great Firewall      (port 3457) - NOT RUNNING"
  FAILED=$((FAILED + 1))
fi

# 3. Electron Player (port 5173)
if lsof -i :5173 >/dev/null 2>&1; then
  echo "✅ Electron Player     (port 5173)"
else
  echo "❌ Electron Player     (port 5173) - NOT RUNNING"
  FAILED=$((FAILED + 1))
fi

# 4. Firewall Monitor (Electron app)
if pgrep -f "firewall-monitor/node_modules" >/dev/null 2>&1; then
  echo "✅ Firewall Monitor    (Electron)"
else
  echo "❌ Firewall Monitor    (Electron) - NOT RUNNING"
  FAILED=$((FAILED + 1))
fi

# 5. Chat Monitor (check via Great Firewall API)
CHAT_STATUS=$(curl -s http://localhost:3457/api/youtube/status 2>/dev/null | grep -o '"chatMonitor":{[^}]*"connected":true' || echo "")
if [ -n "$CHAT_STATUS" ]; then
  echo "✅ Chat Monitor        (Restream)"
else
  echo "❌ Chat Monitor        (Restream) - NOT RUNNING"
  FAILED=$((FAILED + 1))
fi

# 6. VibeToText
if pgrep -f "vibetotext" >/dev/null 2>&1; then
  echo "✅ VibeToText          (voice-to-text)"
else
  echo "❌ VibeToText          (voice-to-text) - NOT RUNNING"
  FAILED=$((FAILED + 1))
fi

echo ""

if [ $FAILED -eq 0 ]; then
  echo "All services running! Ready to stream."
  exit 0
else
  echo "$FAILED service(s) not running."
  exit 1
fi
