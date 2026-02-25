"""Presence client — connects to the presence server from a Jarvis instance."""

import asyncio
import json
import logging
import time
from typing import Callable

import websockets

log = logging.getLogger("presence.client")

HEARTBEAT_INTERVAL = 30  # seconds
MAX_BACKOFF = 30          # seconds


class PresenceClient:
    def __init__(self, server_url: str, user_id: str, display_name: str):
        self.server_url = server_url
        self.user_id = user_id
        self.display_name = display_name
        self.on_notification: Callable[[str, dict], None] | None = None
        self._ws: websockets.WebSocketClientProtocol | None = None
        self._connected = False
        self._closing = False

    async def update_activity(self, status: str, activity: str = None, metadata: dict = None):
        if not self._ws or not self._connected:
            return
        msg = {"type": "activity_update", "status": status, "activity": activity}
        if metadata:
            msg["metadata"] = metadata
        try:
            await self._ws.send(json.dumps(msg))
        except websockets.ConnectionClosed:
            self._connected = False

    async def send_invite(self, game: str, code: str):
        """Broadcast a game invite to all other users."""
        if not self._ws or not self._connected:
            return
        try:
            await self._ws.send(json.dumps({"type": "game_invite", "game": game, "code": code}))
        except websockets.ConnectionClosed:
            self._connected = False

    async def disconnect(self):
        self._closing = True
        if self._ws and self._connected:
            try:
                await self._ws.send(json.dumps({"type": "disconnect"}))
                await self._ws.close()
            except Exception:
                pass
        self._connected = False

    async def run(self):
        """Main loop: connect, receive, auto-reconnect."""
        backoff = 1
        while not self._closing:
            try:
                async with websockets.connect(self.server_url) as ws:
                    self._ws = ws
                    # Handshake
                    await ws.send(json.dumps({
                        "type": "connect",
                        "user_id": self.user_id,
                        "display_name": self.display_name,
                        "version": "1",
                    }))
                    self._connected = True
                    backoff = 1  # reset on successful connect
                    log.info("Connected to presence server")

                    # Run heartbeat and receive concurrently
                    heartbeat = asyncio.create_task(self._heartbeat_loop(ws))
                    try:
                        async for raw in ws:
                            try:
                                msg = json.loads(raw)
                            except json.JSONDecodeError:
                                continue
                            msg_type = msg.get("type")
                            if msg_type == "pong":
                                continue
                            if msg_type == "welcome":
                                log.info("Online users: %d", len(msg.get("users", [])))
                                continue
                            # Broadcast events → notify callback
                            if msg_type in ("user_online", "user_offline", "activity_changed", "game_invite"):
                                if self.on_notification:
                                    try:
                                        self.on_notification(msg_type, msg)
                                    except Exception:
                                        log.exception("Notification callback error")
                    finally:
                        heartbeat.cancel()
                        self._connected = False

            except (OSError, websockets.ConnectionClosed, asyncio.TimeoutError) as e:
                self._connected = False
                if self._closing:
                    break
                log.debug("Presence connection lost (%s), reconnecting in %ds", e, backoff)
                await asyncio.sleep(backoff)
                backoff = min(backoff * 2, MAX_BACKOFF)

    async def _heartbeat_loop(self, ws):
        try:
            while True:
                await asyncio.sleep(HEARTBEAT_INTERVAL)
                await ws.send(json.dumps({"type": "ping"}))
        except (asyncio.CancelledError, websockets.ConnectionClosed):
            pass
