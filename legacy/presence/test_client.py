"""Simulates a fake Jarvis user for testing presence notifications.

Usage:
    python3 -m presence.test_client

Connects to the presence server as "TestUser" and cycles through activities
so you can see notifications appear in your real Jarvis overlay.
"""

import asyncio
import json
import uuid

import websockets

SERVER_URL = "ws://localhost:8790"
FAKE_USER = {"user_id": str(uuid.uuid4()), "display_name": "TestUser"}


async def main():
    print(f"Connecting to {SERVER_URL} as '{FAKE_USER['display_name']}'...")
    async with websockets.connect(SERVER_URL) as ws:
        # Handshake
        await ws.send(json.dumps({
            "type": "connect",
            "user_id": FAKE_USER["user_id"],
            "display_name": FAKE_USER["display_name"],
            "version": "1",
        }))
        welcome = json.loads(await ws.recv())
        others = welcome.get("users", [])
        print(f"Connected. {len(others)} other user(s) online:")
        for u in others:
            print(f"  - {u['display_name']} ({u['status']}, {u.get('activity') or 'idle'})")

        print("\nCommands:")
        print("  kart     — set activity to KartBros")
        print("  tetris   — set activity to Tetris")
        print("  code     — set activity to Code Assistant")
        print("  idle     — go idle")
        print("  online   — clear activity")
        print("  quit     — disconnect")
        print()

        # Listen for broadcasts in background
        async def listen():
            try:
                async for raw in ws:
                    msg = json.loads(raw)
                    if msg["type"] == "pong":
                        continue
                    print(f"  << {msg['type']}: {json.dumps(msg)}")
            except websockets.ConnectionClosed:
                pass

        listener = asyncio.create_task(listen())

        # Heartbeat
        async def heartbeat():
            while True:
                await asyncio.sleep(30)
                try:
                    await ws.send(json.dumps({"type": "ping"}))
                except websockets.ConnectionClosed:
                    break

        hb = asyncio.create_task(heartbeat())

        # Interactive command loop
        loop = asyncio.get_event_loop()
        while True:
            cmd = await loop.run_in_executor(None, lambda: input("> ").strip().lower())
            if cmd == "quit":
                await ws.send(json.dumps({"type": "disconnect"}))
                print("Disconnected.")
                break
            elif cmd == "kart":
                await ws.send(json.dumps({"type": "activity_update", "status": "in_game", "activity": "KartBros"}))
                print("Set: playing KartBros")
            elif cmd == "tetris":
                await ws.send(json.dumps({"type": "activity_update", "status": "in_game", "activity": "Tetris"}))
                print("Set: playing Tetris")
            elif cmd == "code":
                await ws.send(json.dumps({"type": "activity_update", "status": "in_skill", "activity": "Code Assistant"}))
                print("Set: using Code Assistant")
            elif cmd == "idle":
                await ws.send(json.dumps({"type": "activity_update", "status": "idle", "activity": None}))
                print("Set: idle")
            elif cmd == "online":
                await ws.send(json.dumps({"type": "activity_update", "status": "online", "activity": None}))
                print("Set: online")
            else:
                print(f"Unknown command: {cmd}")

        listener.cancel()
        hb.cancel()


if __name__ == "__main__":
    asyncio.run(main())
