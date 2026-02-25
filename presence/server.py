"""Standalone WebSocket presence server for Jarvis.

Run with:  python -m presence.server
"""

import asyncio
import json
import logging
import time
from dataclasses import dataclass, field

import websockets

log = logging.getLogger("presence.server")

HEARTBEAT_TIMEOUT = 90   # seconds without heartbeat → evict
OFFLINE_GRACE = 10       # seconds before broadcasting offline (allows reconnect)
PORT = 8790


@dataclass
class UserPresence:
    user_id: str
    display_name: str
    status: str = "online"
    activity: str | None = None
    connected_at: float = 0.0
    last_heartbeat: float = 0.0
    metadata: dict = field(default_factory=dict)


class PresenceServer:
    def __init__(self):
        self.users: dict[str, UserPresence] = {}
        self.sockets: dict[str, websockets.WebSocketServerProtocol] = {}
        self.grace_tasks: dict[str, asyncio.Task] = {}

    async def handler(self, ws):
        user_id = None
        try:
            async for raw in ws:
                try:
                    msg = json.loads(raw)
                except json.JSONDecodeError:
                    await ws.send(json.dumps({"type": "error", "message": "Invalid JSON"}))
                    continue

                msg_type = msg.get("type")

                if msg_type == "connect":
                    user_id = msg.get("user_id")
                    if not user_id:
                        await ws.send(json.dumps({"type": "error", "message": "Missing user_id"}))
                        continue

                    # Cancel any pending offline grace period
                    grace = self.grace_tasks.pop(user_id, None)
                    if grace:
                        grace.cancel()

                    is_new = user_id not in self.users
                    now = time.time()
                    self.users[user_id] = UserPresence(
                        user_id=user_id,
                        display_name=msg.get("display_name", "Unknown"),
                        connected_at=now,
                        last_heartbeat=now,
                    )
                    self.sockets[user_id] = ws

                    # Send welcome with current user list
                    await ws.send(json.dumps({
                        "type": "welcome",
                        "your_id": user_id,
                        "users": [
                            {"user_id": u.user_id, "display_name": u.display_name,
                             "status": u.status, "activity": u.activity}
                            for u in self.users.values() if u.user_id != user_id
                        ],
                    }))

                    if is_new:
                        log.info("User connected: %s (%s)", self.users[user_id].display_name, user_id[:8])
                        await self.broadcast({
                            "type": "user_online",
                            "user_id": user_id,
                            "display_name": self.users[user_id].display_name,
                            "ts": now,
                        }, exclude=user_id)

                elif msg_type == "ping":
                    if user_id and user_id in self.users:
                        self.users[user_id].last_heartbeat = time.time()
                    await ws.send(json.dumps({"type": "pong"}))

                elif msg_type == "activity_update":
                    if user_id and user_id in self.users:
                        u = self.users[user_id]
                        u.status = msg.get("status", "online")
                        u.activity = msg.get("activity")
                        u.metadata = msg.get("metadata", {})
                        u.last_heartbeat = time.time()
                        log.info("Activity: %s → %s %s", u.display_name, u.status, u.activity or "")
                        await self.broadcast({
                            "type": "activity_changed",
                            "user_id": user_id,
                            "display_name": u.display_name,
                            "status": u.status,
                            "activity": u.activity,
                            "ts": time.time(),
                        }, exclude=user_id)

                elif msg_type == "disconnect":
                    break

        except websockets.ConnectionClosed:
            pass
        finally:
            if user_id:
                self.sockets.pop(user_id, None)
                self.grace_tasks[user_id] = asyncio.create_task(
                    self._offline_grace(user_id)
                )

    async def _offline_grace(self, user_id: str):
        """Wait before broadcasting offline to allow quick reconnects."""
        await asyncio.sleep(OFFLINE_GRACE)
        user = self.users.pop(user_id, None)
        self.grace_tasks.pop(user_id, None)
        if user:
            log.info("User offline: %s (%s)", user.display_name, user_id[:8])
            await self.broadcast({
                "type": "user_offline",
                "user_id": user_id,
                "display_name": user.display_name,
                "ts": time.time(),
            })

    async def broadcast(self, msg: dict, exclude: str = None):
        raw = json.dumps(msg)
        for uid, ws in list(self.sockets.items()):
            if uid != exclude:
                try:
                    await ws.send(raw)
                except websockets.ConnectionClosed:
                    pass

    async def reaper(self):
        """Periodically evict users with stale heartbeats."""
        while True:
            await asyncio.sleep(30)
            now = time.time()
            for uid, user in list(self.users.items()):
                if uid in self.sockets and now - user.last_heartbeat > HEARTBEAT_TIMEOUT:
                    log.info("Reaping stale connection: %s", user.display_name)
                    ws = self.sockets.pop(uid, None)
                    if ws:
                        try:
                            await ws.close()
                        except Exception:
                            pass
                    self.grace_tasks[uid] = asyncio.create_task(
                        self._offline_grace(uid)
                    )

    async def run(self, host="0.0.0.0", port=PORT):
        asyncio.create_task(self.reaper())
        log.info("Presence server starting on %s:%d", host, port)
        async with websockets.serve(self.handler, host, port):
            await asyncio.Future()  # run forever


def main():
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
        datefmt="%H:%M:%S",
    )
    server = PresenceServer()
    asyncio.run(server.run())


if __name__ == "__main__":
    main()
