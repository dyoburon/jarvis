import os
from pathlib import Path
from dotenv import load_dotenv

load_dotenv()

# API Keys
GOOGLE_API_KEY = os.getenv("GOOGLE_API_KEY")

# Gemini
GEMINI_MODEL_DEFAULT = "gemini-3-flash-preview"  # Default conversation
GEMINI_MODEL_CODE = "gemini-3.1-pro-preview"     # Code assistant (fallback)

# Claude Code (via Agent SDK — uses Max subscription)
CLAUDE_CODE_MODEL = "opus"  # "sonnet", "opus", or "haiku"

# Claude Proxy (CLIProxyAPI — exposes Max subscription as OpenAI-compatible API)
CLAUDE_PROXY_BASE_URL = "http://127.0.0.1:8317/v1"
CLAUDE_PROXY_API_KEY = "your-api-key-1"
CLAUDE_PROXY_MODEL = "claude-sonnet-4-6"  # default model for proxy calls

# Audio
SAMPLE_RATE = 24000
CHANNELS = 1

# Project paths
JARVIS_DIR = Path(__file__).parent.resolve()
PROJECTS_DIR = JARVIS_DIR.parent

# Great Firewall (chat overlay)
FIREWALL_API = "http://localhost:3457"

# Whisper
WHISPER_SAMPLE_RATE = 16000
WHISPER_SOCKET = "/tmp/jarvis_whisper.sock"
WHISPER_MODEL = "small"

# Token usage tracking
TOKEN_USAGE_DB = Path(__file__).parent / "data" / "token_usage.db"
GEMINI_PRICING = {
    "gemini-3-flash-preview": {"input": 0.50, "output": 3.00},
    "gemini-3.1-pro-preview": {"input": 2.00, "output": 12.00},
}

# Presence
PRESENCE_URL = os.getenv("PRESENCE_URL", "wss://jarvis-presence-htl4ur3tvq-uc.a.run.app")

SYSTEM_PROMPT = """You are Jarvis. Ultra-brief. 1-2 short sentences max. No filler.
Dry wit when appropriate. Use tools when asked about systems. Never ramble."""
