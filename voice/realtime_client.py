import asyncio
import json

import websockets
from rich.console import Console

import config
from skills.router import SkillRouter, TOOLS

console = Console()


class RealtimeClient:
    """OpenAI Realtime API WebSocket client."""

    def __init__(self, skill_router: SkillRouter, metal_bridge=None):
        self.router = skill_router
        self.metal = metal_bridge
        self.ws = None
        self._current_transcript = ""
        self._function_call_args = {}  # call_id -> accumulated args
        # Skill chat mode
        self.skill_active = False
        self.pending_call_id: str | None = None
        self.pending_tool_name: str | None = None
        self._skill_trigger_transcript: str | None = None
        # Disconnect/reconnect state
        self._disconnected = False

    async def connect(self):
        headers = {
            "Authorization": f"Bearer {config.OPENAI_API_KEY}",
            "OpenAI-Beta": "realtime=v1",
        }
        self.ws = await websockets.connect(
            config.REALTIME_URL,
            additional_headers=headers,
            max_size=2**24,
        )
        console.print("[green]Connected to OpenAI Realtime API[/]")

        # Configure session
        await self._send({
            "type": "session.update",
            "session": {
                "modalities": ["text", "audio"],
                "instructions": config.SYSTEM_PROMPT,
                "voice": "cedar",
                "input_audio_format": "pcm16",
                "output_audio_format": "pcm16",
                "input_audio_transcription": {
                    "model": "whisper-1",
                },
                "turn_detection": {
                    "type": "server_vad",
                    "threshold": 0.5,
                    "prefix_padding_ms": 300,
                    "silence_duration_ms": 700,
                },
                "tools": TOOLS,
                "tool_choice": "auto",
            },
        })

    async def disconnect(self):
        """Disconnect WebSocket to stop billing during skill mode."""
        self._disconnected = True
        if self.ws:
            # Force-abort instead of graceful close — the concurrent
            # receive_events loop deadlocks the close handshake.
            ws = self.ws
            self.ws = None
            ws.close_timeout = 0
            try:
                await ws.close()
            except Exception:
                pass
        console.print("[yellow]OpenAI Realtime disconnected (skill mode)[/]")

    async def reconnect(self):
        """Reconnect to OpenAI Realtime after skill mode ends (fresh session)."""
        self._disconnected = False
        await self.connect()
        console.print("[green]OpenAI Realtime reconnected[/]")

    def reset_skill_state(self):
        """Reset skill mode state without sending any WebSocket messages."""
        self.skill_active = False
        self.pending_call_id = None
        self.pending_tool_name = None
        self._skill_trigger_transcript = None

    async def send_audio(self, b64_audio: str):
        """Send a base64 PCM16 audio chunk to the API."""
        if self.ws and not self._disconnected and not self.skill_active:
            await self._send({
                "type": "input_audio_buffer.append",
                "audio": b64_audio,
            })

    async def receive_events(self, on_audio_delta, on_transcript, on_interrupt=None, on_skill_input=None):
        """Main event loop — processes all events from the API."""
        while True:
            # Wait for a valid connection
            while not self.ws:
                await asyncio.sleep(0.1)
            try:
                await self._event_loop(on_audio_delta, on_transcript, on_interrupt, on_skill_input)
            except websockets.exceptions.ConnectionClosed:
                if self._disconnected:
                    console.print("[dim]Event loop paused (disconnected)[/]")
                    while self._disconnected:
                        await asyncio.sleep(0.1)
                    continue
                raise

    async def _event_loop(self, on_audio_delta, on_transcript, on_interrupt, on_skill_input):
        """Inner event processing loop."""
        async for raw in self.ws:
            event = json.loads(raw)
            event_type = event.get("type", "")

            if event_type == "session.created":
                console.print("[dim]Session ready[/]")

            elif event_type == "session.updated":
                console.print("[dim]Session configured[/]")

            elif event_type == "response.audio.delta":
                if self.skill_active:
                    pass  # mute during skill chat
                else:
                    if self.metal:
                        self.metal.send_state("speaking")
                    on_audio_delta(event["delta"])

            elif event_type == "response.audio_transcript.delta":
                pass  # streaming transcript of Jarvis speaking

            elif event_type == "response.audio_transcript.done":
                if self.skill_active:
                    pass  # suppress Jarvis speech transcripts during skill mode
                else:
                    transcript = event.get("transcript", "")
                    if transcript:
                        on_transcript("jarvis", transcript)

            elif event_type == "conversation.item.input_audio_transcription.completed":
                user_text = event.get("transcript", "")
                if user_text:
                    # Skip echo of the transcript that triggered the current skill
                    if self.skill_active and self._skill_trigger_transcript:
                        if user_text.strip().lower() == self._skill_trigger_transcript.strip().lower():
                            self._skill_trigger_transcript = None
                            continue
                    self._current_transcript = user_text
                    if self.skill_active and on_skill_input:
                        await on_skill_input("__skill_chat__", self.pending_tool_name, "", user_text)
                    else:
                        on_transcript("user", user_text)

            elif event_type == "response.function_call_arguments.delta":
                call_id = event.get("call_id", "")
                delta = event.get("delta", "")
                self._function_call_args[call_id] = self._function_call_args.get(call_id, "") + delta

            elif event_type == "response.function_call_arguments.done":
                call_id = event.get("call_id", "")
                tool_name = event.get("name", "")
                arguments = event.get("arguments", "{}")
                self._function_call_args.pop(call_id, None)

                # If already in skill mode, dismiss — Gemini handles everything
                if self.skill_active:
                    console.print(f"  [dim]Ignoring duplicate skill trigger: {tool_name}[/]")
                    # Dismiss the OpenAI function call and suppress any response
                    await self._send({
                        "type": "conversation.item.create",
                        "item": {
                            "type": "function_call_output",
                            "call_id": call_id,
                            "output": "[Handled.]",
                        },
                    })
                    # Cancel the response OpenAI would generate from this output
                    await self._send({"type": "response.cancel"})
                    continue

                console.print(f"\n[bold yellow]⚡ Skill triggered:[/] {tool_name}")

                # Enter skill chat mode — withhold function_call_output
                # OpenAI is blocked from responding, but VAD + Whisper still work
                self.skill_active = True
                self.pending_call_id = call_id
                self.pending_tool_name = tool_name
                self._skill_trigger_transcript = self._current_transcript  # for echo detection

                if on_skill_input:
                    await on_skill_input("__skill_start__", tool_name, arguments, self._current_transcript)

            elif event_type == "response.done":
                if not self.skill_active and self.metal:
                    self.metal.send_state("listening")

            elif event_type == "error":
                error = event.get("error", {})
                console.print(f"[red]API Error:[/] {error.get('message', error)}")

            elif event_type == "input_audio_buffer.speech_started":
                if on_interrupt:
                    on_interrupt()  # clear playback buffer

            elif event_type == "input_audio_buffer.speech_stopped":
                pass  # user stopped speaking

    async def exit_skill_mode(self, summary: str):
        """Send the withheld function_call_output and resume normal Jarvis mode."""
        if not self.skill_active or not self.pending_call_id:
            return

        # Cancel any in-flight response (user may have spoken during skill mode)
        await self._send({"type": "response.cancel"})

        self.skill_active = False
        self._skill_trigger_transcript = None

        await self._send({
            "type": "conversation.item.create",
            "item": {
                "type": "function_call_output",
                "call_id": self.pending_call_id,
                "output": f"[Skill chat completed. {summary}]",
            },
        })
        await self._send({"type": "response.create"})

        self.pending_call_id = None
        self.pending_tool_name = None

    async def _send(self, data: dict):
        if self.ws:
            await self.ws.send(json.dumps(data))

    async def close(self):
        if self.ws:
            await self.ws.close()
