"""Spawns fake users that send game notifications when a real user connects."""

import asyncio
import json
import uuid

import websockets

SERVER_URL = "ws://localhost:8790"

BOT_NAMES = {"Alex", "Sam", "Jordan"}

BOTS = [
    {"name": "Alex", "game": "KartBros", "delay": 2},
    {"name": "Sam", "game": "Tetris", "delay": 5},
    {"name": "Jordan", "game": "Asteroids", "delay": 8},
]


async def run_bot(name: str, game: str, delay: float):
    user_id = str(uuid.uuid4())
    async with websockets.connect(SERVER_URL) as ws:
        await ws.send(json.dumps({
            "type": "connect",
            "user_id": user_id,
            "display_name": name,
            "version": "1",
        }))
        welcome = json.loads(await ws.recv())
        print(f"[{name}] Connected. Waiting for real user...")

        # Wait until we see a non-bot user come online
        real_user_seen = False
        # Check welcome list for non-bot users
        for u in welcome.get("users", []):
            if u.get("display_name") not in BOT_NAMES:
                real_user_seen = True
                break

        while not real_user_seen:
            raw = await ws.recv()
            msg = json.loads(raw)
            if msg.get("type") == "user_online" and msg.get("display_name") not in BOT_NAMES:
                real_user_seen = True

        print(f"[{name}] Real user detected! Starting {game} in {delay}s...")
        await asyncio.sleep(delay)

        await ws.send(json.dumps({
            "type": "activity_update",
            "status": "in_game",
            "activity": game,
        }))
        print(f"[{name}] Now playing {game}")

        # Stay online for 30s then go back to online
        await asyncio.sleep(30)
        await ws.send(json.dumps({
            "type": "activity_update",
            "status": "online",
            "activity": None,
        }))
        print(f"[{name}] Stopped playing {game}")

        # Stay connected for a while
        await asyncio.sleep(60)
        await ws.send(json.dumps({"type": "disconnect"}))
        print(f"[{name}] Disconnected")


async def main():
    print("Spawning 3 bots... they'll fire game notifications when you boot Jarvis.\n")
    await asyncio.gather(*(run_bot(**bot) for bot in BOTS))


if __name__ == "__main__":
    asyncio.run(main())
