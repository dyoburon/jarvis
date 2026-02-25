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
            return data

    identity = {
        "user_id": str(uuid.uuid4()),
        "display_name": socket.gethostname(),
    }
    IDENTITY_PATH.parent.mkdir(parents=True, exist_ok=True)
    IDENTITY_PATH.write_text(json.dumps(identity, indent=2))
    return identity
