"""
Generate Jarvis boot-up voiceover using fal.ai ElevenLabs TTS endpoint.

Run once to produce the MP3, then it's baked into the app assets.

Usage:
    source /Users/dylan/Desktop/projects/prism-marketing/.venv/bin/activate
    python generate-voice.py

Requires:
    FAL_KEY in environment or in prism-marketing/.env
"""

import json
import os
import sys
import urllib.request
from pathlib import Path

# Try loading FAL_KEY from prism-marketing .env
for env_path in [
    Path(__file__).parent.parent / ".env",
    Path("/Users/dylan/Desktop/projects/prism-marketing/.env"),
]:
    if env_path.exists():
        for line in env_path.read_text().splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                key, val = line.split("=", 1)
                os.environ.setdefault(key.strip(), val.strip())

FAL_API_URL = "https://fal.run/fal-ai/elevenlabs/tts/eleven-v3"
FAL_KEY = os.environ.get("FAL_KEY")

# Voice: "Chris" — male, conversational American (same as prism-marketing reels)
VOICE_NAME = "Chris"
VOICE_SETTINGS = {
    "stability": 0.4,
    "similarity_boost": 0.78,
    "style": 0.15,
    "speed": 0.95,
}

# Short and punchy for the 5-second voiceover window
SCRIPT = "All systems online. Welcome back. Let's have a great stream."

OUTPUT_PATH = Path(__file__).parent.parent / "assets" / "audio" / "bootup-voice.mp3"


def generate():
    if not FAL_KEY:
        print("Error: FAL_KEY not set")
        print("Set it in your environment or check prism-marketing/.env")
        sys.exit(1)

    print(f"Generating voiceover with voice '{VOICE_NAME}'...")
    print(f"Script: \"{SCRIPT}\"")
    print(f"Endpoint: {FAL_API_URL}")

    payload = json.dumps({
        "text": SCRIPT,
        "voice": VOICE_NAME,
        **VOICE_SETTINGS,
    }).encode()

    req = urllib.request.Request(
        FAL_API_URL,
        data=payload,
        headers={
            "Authorization": f"Key {FAL_KEY}",
            "Content-Type": "application/json",
        },
    )

    with urllib.request.urlopen(req, timeout=120) as resp:
        result = json.loads(resp.read())

    audio_url = result.get("audio", {}).get("url")
    if not audio_url:
        print(f"Error: No audio URL in response: {result}")
        sys.exit(1)

    print(f"Downloading audio from fal.ai...")
    audio_req = urllib.request.Request(audio_url)
    with urllib.request.urlopen(audio_req) as resp:
        audio_data = resp.read()

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_bytes(audio_data)
    print(f"Saved: {OUTPUT_PATH} ({len(audio_data)} bytes)")


if __name__ == "__main__":
    generate()
