#!/usr/bin/env python3
"""Interactive DM test bot for Jarvis encrypted chat.

Connects to Supabase Realtime as a synthetic peer, appears in the
Jarvis chat panel's online-users list, and exchanges E2E-encrypted
DMs with a real Jarvis instance.

Usage:
    .venv/bin/python3 test_dm_bot.py [--nick BotUser] [--verbose]
"""

import argparse
import asyncio
import base64
import hashlib
import json
import os
import sys
import time
import uuid
from datetime import datetime, timezone

try:
    import websockets
    from cryptography.hazmat.primitives.asymmetric import ec, utils
    from cryptography.hazmat.primitives.ciphers.aead import AESGCM
    from cryptography.hazmat.primitives import hashes, serialization
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("Run with the project venv:  .venv/bin/python3 test_dm_bot.py")
    sys.exit(1)

# ── constants ────────────────────────────────────────────────────

SUPABASE_PROJECT = "ojmqzagktzkualzgpcbq"
SUPABASE_KEY = (
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9."
    "eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im9qbXF6YWdrdHprdWFsemdwY2JxIiwi"
    "cm9sZSI6ImFub24iLCJpYXQiOjE3NzE5ODY1ODIsImV4cCI6MjA4NzU2MjU4Mn0."
    "WkDiksXkye-YyL1RSbAYv1iVW_Sv5zwST0RcloN_0jQ"
)
WSS_URL = (
    f"wss://{SUPABASE_PROJECT}.supabase.co"
    f"/realtime/v1/websocket?apikey={SUPABASE_KEY}&vsn=1.0.0"
)
PRIMARY_CHANNEL = "jarvis-livechat"
DM_PREFIX = "jarvis-dm-"
HEARTBEAT_SEC = 25
VERBOSE = False  # set via --verbose


# ── crypto identity ──────────────────────────────────────────────

class CryptoIdentity:
    """P-256 ECDSA + ECDH identity that matches the Rust CryptoService."""

    def __init__(self):
        self._ecdsa_key = ec.generate_private_key(ec.SECP256R1())
        self._ecdh_key = ec.generate_private_key(ec.SECP256R1())

        self.pubkey_b64 = self._export_spki(self._ecdsa_key)
        self.dh_pubkey_b64 = self._export_spki(self._ecdh_key)
        self.fingerprint = self._compute_fp(self.pubkey_b64)

    # ── public key helpers ──

    @staticmethod
    def _export_spki(private_key) -> str:
        der = private_key.public_key().public_bytes(
            serialization.Encoding.DER,
            serialization.PublicFormat.SubjectPublicKeyInfo,
        )
        return base64.b64encode(der).decode()

    @staticmethod
    def _compute_fp(spki_b64: str) -> str:
        der = base64.b64decode(spki_b64)
        h = hashlib.sha256(der).digest()
        return ":".join(f"{b:02x}" for b in h[:8])

    # ── ECDH shared key ──

    def derive_shared_key(self, other_dh_spki_b64: str) -> bytes:
        other_pub = serialization.load_der_public_key(
            base64.b64decode(other_dh_spki_b64)
        )
        raw = self._ecdh_key.exchange(ec.ECDH(), other_pub)
        return hashlib.sha256(raw).digest()

    # ── AES-256-GCM ──

    @staticmethod
    def encrypt(plaintext: str, key: bytes) -> tuple[str, str]:
        iv = os.urandom(12)
        ct = AESGCM(key).encrypt(iv, plaintext.encode(), None)
        return base64.b64encode(iv).decode(), base64.b64encode(ct).decode()

    @staticmethod
    def decrypt(iv_b64: str, ct_b64: str, key: bytes) -> str:
        iv = base64.b64decode(iv_b64)
        ct = base64.b64decode(ct_b64)
        return AESGCM(key).decrypt(iv, ct, None).decode()

    # ── ECDSA-SHA256 (P1363 format) ──

    def sign(self, data: str) -> str:
        der_sig = self._ecdsa_key.sign(data.encode(), ec.ECDSA(hashes.SHA256()))
        r, s = utils.decode_dss_signature(der_sig)
        raw = r.to_bytes(32, "big") + s.to_bytes(32, "big")
        return base64.b64encode(raw).decode()

    @staticmethod
    def verify(data: str, sig_b64: str, pubkey_b64: str) -> bool:
        try:
            sig_bytes = base64.b64decode(sig_b64)
            r = int.from_bytes(sig_bytes[:32], "big")
            s = int.from_bytes(sig_bytes[32:], "big")
            der_sig = utils.encode_dss_signature(r, s)
            pub = serialization.load_der_public_key(base64.b64decode(pubkey_b64))
            pub.verify(der_sig, data.encode(), ec.ECDSA(hashes.SHA256()))
            return True
        except Exception:
            return False

    # ── build a complete message payload ──

    def build_message(self, text: str, user_id: str, nick: str) -> dict:
        """Not used directly — the bot encrypts per-target with a shared key."""
        raise NotImplementedError("Use encrypt + sign individually per DM target")


# ── phoenix websocket client ─────────────────────────────────────

class PhoenixClient:
    """Minimal Phoenix Channels v1 client over WebSocket."""

    def __init__(self):
        self._ws = None
        self._ref = 0
        self._join_refs: dict[str, str] = {}  # topic -> join_ref

    def _next_ref(self) -> str:
        self._ref += 1
        return str(self._ref)

    async def connect(self):
        self._ws = await websockets.connect(WSS_URL)

    async def close(self):
        if self._ws:
            await self._ws.close()

    async def _send(self, topic: str, event: str, payload: dict, ref: str = None):
        ref = ref or self._next_ref()
        msg = {"topic": topic, "event": event, "payload": payload, "ref": ref}
        raw = json.dumps(msg)
        if VERBOSE:
            print(f"  >> {raw[:200]}")
        await self._ws.send(raw)
        return ref

    async def join(self, topic: str, config: dict, wait_ack: bool = False) -> str:
        ref = self._next_ref()
        self._join_refs[topic] = ref
        await self._send(
            f"realtime:{topic}", "phx_join", {"config": config}, ref
        )
        if wait_ack:
            # Wait for phx_reply with matching ref
            while True:
                reply = await self.recv()
                if reply.get("event") == "phx_reply" and reply.get("ref") == ref:
                    status = reply.get("payload", {}).get("status")
                    if status == "ok":
                        return ref
                    else:
                        raise RuntimeError(f"Join failed for {topic}: {reply.get('payload')}")
                # Stash non-reply messages (presence_state etc may arrive)
                if hasattr(self, '_stashed'):
                    self._stashed.append(reply)
                else:
                    self._stashed = [reply]
        return ref

    async def leave(self, topic: str):
        await self._send(f"realtime:{topic}", "phx_leave", {})
        self._join_refs.pop(topic, None)

    async def broadcast(self, topic: str, event: str, payload: dict):
        await self._send(f"realtime:{topic}", "broadcast", {
            "type": "broadcast",
            "event": event,
            "payload": payload,
        })

    async def track(self, topic: str, metadata: dict):
        await self._send(f"realtime:{topic}", "presence", {
            "type": "presence",
            "event": "track",
            "payload": metadata,
        })

    async def heartbeat(self):
        await self._send("phoenix", "heartbeat", {})

    async def recv(self) -> dict:
        raw = await self._ws.recv()
        if VERBOSE:
            print(f"  << {raw[:300]}")
        return json.loads(raw)


# ── bot logic ────────────────────────────────────────────────────

class DMBot:
    """Orchestrates presence, DM channels, crypto, and CLI."""

    def __init__(self, nick: str):
        self.nick = nick
        self.user_id = str(uuid.uuid4())
        self.identity = CryptoIdentity()
        self.phoenix = PhoenixClient()

        # peer tracking: presence_key -> {nick, fingerprint, dhPubkey}
        self.peers: dict[str, dict] = {}
        # dm channel state: channel_topic -> {nick, fingerprint, shared_key}
        self.dm_channels: dict[str, dict] = {}
        # current DM target nick (for CLI send)
        self.dm_target: str | None = None

        self._running = False

    # ── lifecycle ──

    async def start(self):
        self._running = True
        print(f"Identity:")
        print(f"  nick:        {self.nick}")
        print(f"  user_id:     {self.user_id}")
        print(f"  fingerprint: {self.identity.fingerprint}")
        print(f"  pubkey:      {self.identity.pubkey_b64[:40]}...")
        print(f"  dhPubkey:    {self.identity.dh_pubkey_b64[:40]}...")
        print()

        print(f"Connecting to Supabase Realtime...")
        await self.phoenix.connect()
        print("WebSocket connected.")

        # Join primary channel with presence — wait for server ack
        ref = await self.phoenix.join(PRIMARY_CHANNEL, {
            "broadcast": {"self": False, "ack": True},
            "presence": {"key": self.user_id},
        }, wait_ack=True)
        print(f"Joined {PRIMARY_CHANNEL} (ref={ref}), tracking presence...")

        # Track presence with crypto keys
        await self.phoenix.track(PRIMARY_CHANNEL, {
            "nick": self.nick,
            "online_at": datetime.now(timezone.utc).isoformat(),
            "pubkey": self.identity.pubkey_b64,
            "fingerprint": self.identity.fingerprint,
            "dhPubkey": self.identity.dh_pubkey_b64,
        })
        print("Presence tracked.")

        # Process any messages that arrived during join handshake
        for stashed in getattr(self.phoenix, '_stashed', []):
            await self._handle_message(stashed)
        self.phoenix._stashed = []

        # Start background tasks
        tasks = [
            asyncio.create_task(self._heartbeat_loop()),
            asyncio.create_task(self._receive_loop()),
            asyncio.create_task(self._cli_loop()),
        ]

        try:
            await asyncio.gather(*tasks)
        except asyncio.CancelledError:
            pass
        finally:
            await self.phoenix.close()
            print("Disconnected.")

    async def stop(self):
        self._running = False

    # ── background loops ──

    async def _heartbeat_loop(self):
        while self._running:
            await asyncio.sleep(HEARTBEAT_SEC)
            try:
                await self.phoenix.heartbeat()
            except Exception:
                break

    async def _receive_loop(self):
        while self._running:
            try:
                msg = await self.phoenix.recv()
            except websockets.ConnectionClosed:
                print("\n[!] Connection lost.")
                self._running = False
                break
            await self._handle_message(msg)

    # ── message dispatch ──

    async def _handle_message(self, msg: dict):
        topic = msg.get("topic", "")
        event = msg.get("event", "")
        payload = msg.get("payload", {})

        if event == "phx_reply":
            status = payload.get("status", "")
            if status != "ok":
                print(f"[!] phx_reply error on {topic}: {payload}")
            return

        if event == "phx_error" or event == "phx_close":
            print(f"[!] {event} on {topic}")
            return

        if event == "presence_state":
            self._handle_presence_state(payload)
            return

        if event == "presence_diff":
            await self._handle_presence_diff(payload)
            return

        if event == "broadcast":
            inner_event = payload.get("event", "")
            inner_payload = payload.get("payload", {})
            if inner_event == "message":
                await self._handle_chat_message(topic, inner_payload)
            return

    # ── presence handling ──

    def _handle_presence_state(self, state: dict):
        for key, val in state.items():
            metas = val.get("metas", [])
            if not metas:
                continue
            meta = metas[0]
            nick = meta.get("nick", key)
            fp = meta.get("fingerprint")
            dh = meta.get("dhPubkey")
            if key == self.user_id:
                continue
            self.peers[key] = {"nick": nick, "fingerprint": fp, "dhPubkey": dh}
            if fp and dh:
                print(f"  [+] {nick}  fp={fp}")

        count = len(self.peers)
        print(f"Presence: {count} other user(s) online.")
        # Preemptively join DM channels for all peers with keys
        asyncio.create_task(self._subscribe_all_dm_channels())

    async def _handle_presence_diff(self, diff: dict):
        joins = diff.get("joins", {})
        leaves = diff.get("leaves", {})

        for key, val in joins.items():
            metas = val.get("metas", [])
            if not metas or key == self.user_id:
                continue
            meta = metas[0]
            nick = meta.get("nick", key)
            fp = meta.get("fingerprint")
            dh = meta.get("dhPubkey")
            self.peers[key] = {"nick": nick, "fingerprint": fp, "dhPubkey": dh}
            print(f"\n  [+] {nick} joined  fp={fp or '(none)'}")
            if fp and dh:
                await self._subscribe_dm_channel(nick, fp, dh)
            print("> ", end="", flush=True)

        for key, val in leaves.items():
            peer = self.peers.pop(key, None)
            if peer:
                print(f"\n  [-] {peer['nick']} left")
                print("> ", end="", flush=True)

    # ── DM channel management ──

    def _dm_channel_name(self, other_fp: str) -> str:
        a = self.identity.fingerprint.replace(":", "")
        b = other_fp.replace(":", "")
        parts = sorted([a, b])
        return f"{DM_PREFIX}{parts[0]}-{parts[1]}"

    async def _subscribe_all_dm_channels(self):
        for key, peer in self.peers.items():
            fp = peer.get("fingerprint")
            dh = peer.get("dhPubkey")
            if fp and dh:
                await self._subscribe_dm_channel(peer["nick"], fp, dh)

    async def _subscribe_dm_channel(self, nick: str, fp: str, dh_pubkey: str):
        ch = self._dm_channel_name(fp)
        if ch in self.dm_channels:
            return

        try:
            shared_key = self.identity.derive_shared_key(dh_pubkey)
        except Exception as e:
            print(f"  [!] ECDH failed for {nick}: {e}")
            return

        await self.phoenix.join(ch, {
            "broadcast": {"self": False, "ack": True},
        })
        self.dm_channels[ch] = {
            "nick": nick,
            "fingerprint": fp,
            "shared_key": shared_key,
        }
        print(f"  [dm] Subscribed to DM channel with {nick}")

    # ── incoming DM messages ──

    async def _handle_chat_message(self, topic: str, payload: dict):
        # Strip the "realtime:" prefix if present
        channel = topic.removeprefix("realtime:")

        # Check if it's on a DM channel we know about
        dm_info = self.dm_channels.get(channel)

        # Could also be on the primary channel (group chat) — skip for now
        if not dm_info:
            # Group chat message — try room key decryption
            if channel == PRIMARY_CHANNEL:
                await self._handle_group_message(payload)
            return

        iv = payload.get("iv")
        ct = payload.get("ct")
        if not iv or not ct:
            return

        sender = payload.get("nick", "Unknown")
        sender_id = payload.get("userId", "?")

        # Skip own messages
        if sender_id == self.user_id:
            return

        try:
            plaintext = self.identity.decrypt(iv, ct, dm_info["shared_key"])
        except Exception as e:
            print(f"\n  [!] Decrypt failed from {sender}: {e}")
            print("> ", end="", flush=True)
            return

        # Verify signature if present
        verified = ""
        sig = payload.get("sig")
        pubkey = payload.get("pubkey")
        if sig and pubkey:
            canonical = "|".join(str(x) for x in [
                payload.get("id", ""),
                sender_id,
                payload.get("nick", ""),
                payload.get("ts", ""),
                iv, ct,
            ])
            if self.identity.verify(canonical, sig, pubkey):
                verified = " ✓"
            else:
                verified = " ✗ BAD SIG"

        ts = payload.get("ts", 0)
        time_str = datetime.fromtimestamp(ts / 1000).strftime("%H:%M:%S") if ts else "??:??:??"
        print(f"\n  [{time_str}] {sender}{verified}: {plaintext}")
        print("> ", end="", flush=True)

    async def _handle_group_message(self, payload: dict):
        """Decrypt a group chat message using PBKDF2 room key."""
        iv = payload.get("iv")
        ct = payload.get("ct")
        sender = payload.get("nick", "Unknown")
        sender_id = payload.get("userId", "?")
        if not iv or not ct or sender_id == self.user_id:
            return

        # Derive room key via PBKDF2
        room_key = hashlib.pbkdf2_hmac(
            "sha256",
            PRIMARY_CHANNEL.encode(),
            b"jarvis-livechat-salt-v1",
            100_000,
            dklen=32,
        )
        try:
            plaintext = self.identity.decrypt(iv, ct, room_key)
        except Exception:
            return

        ts = payload.get("ts", 0)
        time_str = datetime.fromtimestamp(ts / 1000).strftime("%H:%M:%S") if ts else "??:??:??"
        print(f"\n  [{time_str}] #{PRIMARY_CHANNEL} {sender}: {plaintext}")
        print("> ", end="", flush=True)

    # ── sending DMs ──

    async def send_dm(self, text: str):
        if not self.dm_target:
            print("No DM target. Use: dm <nick>")
            return

        # Find the DM channel for this target
        target_channel = None
        target_info = None
        for ch, info in self.dm_channels.items():
            if info["nick"] == self.dm_target:
                target_channel = ch
                target_info = info
                break

        if not target_channel or not target_info:
            print(f"No DM channel for {self.dm_target}. Are they online?")
            return

        # Encrypt
        iv_b64, ct_b64 = self.identity.encrypt(text, target_info["shared_key"])

        # Build payload
        msg_id = str(uuid.uuid4())
        ts = int(time.time() * 1000)

        canonical = "|".join([msg_id, self.user_id, self.nick, str(ts), iv_b64, ct_b64])
        sig = self.identity.sign(canonical)

        payload = {
            "id": msg_id,
            "userId": self.user_id,
            "nick": self.nick,
            "ts": ts,
            "iv": iv_b64,
            "ct": ct_b64,
            "sig": sig,
            "pubkey": self.identity.pubkey_b64,
            "fingerprint": self.identity.fingerprint,
        }

        await self.phoenix.broadcast(target_channel, "message", payload)

        time_str = datetime.fromtimestamp(ts / 1000).strftime("%H:%M:%S")
        print(f"  [{time_str}] (you → {self.dm_target}): {text}")

    # ── CLI ──

    async def _cli_loop(self):
        loop = asyncio.get_event_loop()

        print()
        print("Commands:")
        print("  users       — list online users")
        print("  dm <nick>   — set DM target")
        print("  close       — clear DM target")
        print("  identity    — show crypto identity")
        print("  quit        — disconnect")
        print("  <text>      — send DM to current target")
        print()

        while self._running:
            try:
                line = await loop.run_in_executor(None, lambda: input("> "))
            except (EOFError, KeyboardInterrupt):
                self._running = False
                break

            line = line.strip()
            if not line:
                continue

            if line == "quit":
                self._running = False
                break

            elif line == "users":
                if not self.peers:
                    print("  No other users online.")
                else:
                    for key, p in self.peers.items():
                        fp = p.get("fingerprint") or "(no keys)"
                        dm_status = "DM ready" if p.get("dhPubkey") else "no ECDH"
                        marker = " ← target" if p["nick"] == self.dm_target else ""
                        print(f"  {p['nick']}  fp={fp}  [{dm_status}]{marker}")

            elif line.startswith("dm "):
                target_nick = line[3:].strip()
                found = False
                for key, p in self.peers.items():
                    if p["nick"] == target_nick:
                        if not p.get("dhPubkey"):
                            print(f"  {target_nick} has no ECDH key — cannot DM.")
                        else:
                            self.dm_target = target_nick
                            print(f"  DM target set to {target_nick}. Type to send messages.")
                        found = True
                        break
                if not found:
                    print(f"  User '{target_nick}' not found. Use 'users' to list.")

            elif line == "close":
                if self.dm_target:
                    print(f"  Closed DM with {self.dm_target}.")
                    self.dm_target = None
                else:
                    print("  No active DM.")

            elif line == "identity":
                print(f"  nick:        {self.nick}")
                print(f"  user_id:     {self.user_id}")
                print(f"  fingerprint: {self.identity.fingerprint}")
                print(f"  pubkey:      {self.identity.pubkey_b64}")
                print(f"  dhPubkey:    {self.identity.dh_pubkey_b64}")

            else:
                # Treat as a DM message
                await self.send_dm(line)


# ── entry point ──────────────────────────────────────────────────

def main():
    global VERBOSE
    parser = argparse.ArgumentParser(description="Jarvis DM test bot")
    parser.add_argument("--nick", default="BotUser", help="Bot nickname")
    parser.add_argument("--verbose", "-v", action="store_true", help="Log all wire messages")
    args = parser.parse_args()
    VERBOSE = args.verbose

    bot = DMBot(args.nick)
    try:
        asyncio.run(bot.start())
    except KeyboardInterrupt:
        print("\nBye.")


if __name__ == "__main__":
    main()
