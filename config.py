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
PROJECTS_DIR = Path("/Users/dylan/Desktop/projects")

# Domain Drop Hunter
DOMAIN_HUNTER_DB = PROJECTS_DIR / "domain-drop-hunter/data/domains.db"
DOMAIN_HUNTER_API = "http://localhost:5199"

# Paper Feeder
PAPER_FEEDER_DB = PROJECTS_DIR / "paper-feeder/data/papers.db"
PAPER_FEEDER_API = "http://localhost:5200"

# Great Firewall
FIREWALL_API = "http://localhost:3457"

# VibeToText
VIBETOTEXT_DB = Path.home() / ".vibetotext/history.db"
VIBETOTEXT_SOCKET = "/tmp/vibetotext.sock"
WHISPER_SAMPLE_RATE = 16000

# Token usage tracking
TOKEN_USAGE_DB = Path(__file__).parent / "data" / "token_usage.db"
GEMINI_PRICING = {
    "gemini-3-flash-preview": {"input": 0.50, "output": 3.00},
    "gemini-3.1-pro-preview": {"input": 2.00, "output": 12.00},
}

SYSTEM_PROMPT = """You are Jarvis. Ultra-brief. 1-2 short sentences max. No filler.
Dry wit when appropriate. Use tools when asked about systems. Never ramble."""
