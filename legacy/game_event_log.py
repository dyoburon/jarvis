"""
Structured log collector for game window lifecycle events.

Collects game_event messages from Metal stdout and metal.log entries
into a unified, chronological timeline for debugging and testing.
"""

import json
import logging
import logging.handlers
import os
import re
import threading
import time
from datetime import datetime, timezone

# Dedicated game debug log — always writes to game_debug.log
_GAME_LOG_PATH = os.path.join(os.path.dirname(__file__), "game_debug.log")
_game_log = logging.getLogger("jarvis.game")
_game_log.setLevel(logging.DEBUG)
_game_log.propagate = False
if not _game_log.handlers:
    _gh = logging.handlers.RotatingFileHandler(
        _GAME_LOG_PATH, maxBytes=2 * 1024 * 1024, backupCount=2, encoding="utf-8",
    )
    _gh.setFormatter(logging.Formatter("%(asctime)s %(message)s", datefmt="%H:%M:%S"))
    _game_log.addHandler(_gh)


class GameEventLog:
    """Collects game_event messages and metal.log entries into a unified timeline."""

    def __init__(self, metal_log_path: str = None):
        self._events: list[dict] = []
        self._metal_log_path = metal_log_path or os.path.join(
            os.path.dirname(__file__), "metal.log"
        )
        self._lock = threading.Lock()

    def ingest(self, msg: dict) -> None:
        """Ingest a game_event message from Metal stdout."""
        if msg.get("type") != "game_event":
            return
        entry = {
            "source": "metal_stdout",
            "ts": msg.get("ts", datetime.now(timezone.utc).isoformat()),
            "event": msg.get("event"),
        }
        for k, v in msg.items():
            if k not in ("type", "event", "ts"):
                entry[k] = v
        with self._lock:
            self._events.append(entry)
        extras = {k: v for k, v in entry.items() if k not in ("source", "ts", "event")}
        _game_log.info(f"[METAL] {entry['event']} {extras}" if extras else f"[METAL] {entry['event']}")

    def log_action(self, action: str, **kwargs) -> None:
        """Log a Python-side action (command sent, state change, etc.)."""
        parts = [f"[PYTHON] {action}"]
        if kwargs:
            parts.append(str(kwargs))
        _game_log.info(" ".join(parts))

    def read_metal_log(self, since: datetime = None) -> list[dict]:
        """Parse metal.log entries since a given timestamp."""
        entries = []
        pattern = re.compile(
            r"^(\d{4}-\d{2}-\d{2}T[\d:]+Z)\s+\[METAL\]\s+(.*)$"
        )
        if not os.path.exists(self._metal_log_path):
            return entries
        with open(self._metal_log_path, "r") as f:
            for line in f:
                m = pattern.match(line.strip())
                if m:
                    ts_str, message = m.groups()
                    if since:
                        ts = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
                        if ts < since:
                            continue
                    entries.append({
                        "source": "metal_log",
                        "ts": ts_str,
                        "message": message,
                    })
        return entries

    def timeline(self, include_metal_log: bool = True,
                 since: datetime = None) -> list[dict]:
        """Merge stdout events and metal.log into chronological timeline."""
        with self._lock:
            merged = list(self._events)
        if include_metal_log:
            merged.extend(self.read_metal_log(since=since))
        merged.sort(key=lambda e: e["ts"])
        return merged

    def wait_for_event(self, event_name: str, timeout: float = 10.0,
                       poll_interval: float = 0.1) -> dict | None:
        """Block until a specific event type appears. Returns the event or None on timeout."""
        deadline = time.time() + timeout
        seen = 0
        while time.time() < deadline:
            with self._lock:
                for entry in self._events[seen:]:
                    if entry.get("event") == event_name:
                        return entry
                seen = len(self._events)
            time.sleep(poll_interval)
        return None

    def dump_json(self, path: str = None, **kwargs) -> str:
        """Dump timeline to JSON file or return as string."""
        tl = self.timeline(**kwargs)
        result = json.dumps(tl, indent=2, default=str)
        if path:
            with open(path, "w") as f:
                f.write(result)
        return result

    def pretty_print(self, **kwargs) -> None:
        """Pretty-print timeline to console."""
        for entry in self.timeline(**kwargs):
            source = entry["source"]
            ts = entry["ts"]
            if source == "metal_stdout":
                event = entry.get("event", "?")
                extras = {k: v for k, v in entry.items()
                         if k not in ("source", "ts", "event")}
                extra_str = f" {extras}" if extras else ""
                print(f"  [{ts}] STDOUT: {event}{extra_str}")
            else:
                print(f"  [{ts}] LOG:    {entry.get('message', '')}")

    def clear(self) -> None:
        """Clear all collected events."""
        with self._lock:
            self._events.clear()

    @property
    def events(self) -> list[dict]:
        """Return a copy of collected events."""
        with self._lock:
            return list(self._events)
