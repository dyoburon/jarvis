#!/bin/bash
# jarvis login — refreshes Claude Code OAuth token and updates .env
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="$SCRIPT_DIR/.env"

echo "Logging into Claude Code..."
CLAUDECODE= claude auth login

# Pull new token from macOS Keychain → claudeAiOauth.accessToken
OAUTH_TOKEN=$(security find-generic-password -s "Claude Code-credentials" -w 2>/dev/null \
    | python3.12 -c "import sys,json; print(json.load(sys.stdin)['claudeAiOauth']['accessToken'])")

if [ -z "$OAUTH_TOKEN" ]; then
    echo "Could not extract OAuth token from Keychain"
    exit 1
fi

# Update .env file
if grep -q "CLAUDE_CODE_OAUTH_TOKEN" "$ENV_FILE" 2>/dev/null; then
    sed -i '' "s|^CLAUDE_CODE_OAUTH_TOKEN=.*|CLAUDE_CODE_OAUTH_TOKEN=$OAUTH_TOKEN|" "$ENV_FILE"
else
    echo "CLAUDE_CODE_OAUTH_TOKEN=$OAUTH_TOKEN" >> "$ENV_FILE"
fi

echo "Token updated in .env (${OAUTH_TOKEN:0:15}...)"
