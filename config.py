import os
from pathlib import Path
from dotenv import load_dotenv

load_dotenv()

# API Keys
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")
GOOGLE_API_KEY = os.getenv("GOOGLE_API_KEY")

# OpenAI Realtime
REALTIME_MODEL = "gpt-realtime"
REALTIME_URL = f"wss://api.openai.com/v1/realtime?model={REALTIME_MODEL}"

# Gemini
GEMINI_MODEL = "gemini-3-flash-preview"

# Audio
SAMPLE_RATE = 24000
CHANNELS = 1
CHUNK_MS = 100  # send audio every 100ms

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

SYSTEM_PROMPT = """You are Jarvis. Ultra-brief. 1-2 short sentences max. No filler.
Dry wit when appropriate. Use tools when asked about systems. Never ramble."""
