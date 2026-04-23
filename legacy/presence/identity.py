import json
import socket
import uuid
from pathlib import Path

IDENTITY_PATH = Path.home() / ".jarvis" / "identity.json"


def load_identity() -> dict:
    """Load or create a persistent Jarvis identity for this machine."""
    if IDENTITY_PATH.exists():
        data = json.loads(IDENTITY_PATH.read_text())
        if "user_id" in data and "display_name" in data:
            if "name_set" not in data:
                data["name_set"] = False
            return data

    identity = {
        "user_id": str(uuid.uuid4()),
        "display_name": socket.gethostname(),
        "name_set": False,
    }
    IDENTITY_PATH.parent.mkdir(parents=True, exist_ok=True)
    IDENTITY_PATH.write_text(json.dumps(identity, indent=2))
    return identity


def save_display_name(name: str) -> dict:
    """Update display_name and mark as explicitly set."""
    data = load_identity()
    data["display_name"] = name
    data["name_set"] = True
    IDENTITY_PATH.write_text(json.dumps(data, indent=2))
    return data
