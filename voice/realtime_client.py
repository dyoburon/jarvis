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

    async def send_audio(self, b64_audio: str):
        """Send a base64 PCM16 audio chunk to the API."""
        if self.ws:
            await self._send({
                "type": "input_audio_buffer.append",
                "audio": b64_audio,
            })

    async def receive_events(self, on_audio_delta, on_transcript, on_interrupt=None):
        """Main event loop — processes all events from the API."""
        async for raw in self.ws:
            event = json.loads(raw)
            event_type = event.get("type", "")

            if event_type == "session.created":
                console.print("[dim]Session ready[/]")

            elif event_type == "session.updated":
                console.print("[dim]Session configured[/]")

            elif event_type == "response.audio.delta":
                if self.metal:
                    self.metal.send_state("speaking")
                on_audio_delta(event["delta"])

            elif event_type == "response.audio_transcript.delta":
                pass  # streaming transcript of Jarvis speaking

            elif event_type == "response.audio_transcript.done":
                transcript = event.get("transcript", "")
                if transcript:
                    on_transcript("jarvis", transcript)

            elif event_type == "conversation.item.input_audio_transcription.completed":
                user_text = event.get("transcript", "")
                if user_text:
                    self._current_transcript = user_text
                    on_transcript("user", user_text)

            elif event_type == "response.function_call_arguments.delta":
                call_id = event.get("call_id", "")
                delta = event.get("delta", "")
                self._function_call_args[call_id] = self._function_call_args.get(call_id, "") + delta

            elif event_type == "response.function_call_arguments.done":
                call_id = event.get("call_id", "")
                tool_name = event.get("name", "")
                arguments = event.get("arguments", "{}")

                console.print(f"\n[bold yellow]⚡ Skill triggered:[/] {tool_name}")

                # Execute skill
                result = await self.router.handle_tool_call(
                    tool_name, arguments, self._current_transcript
                )

                # Send result back to OpenAI
                await self._send({
                    "type": "conversation.item.create",
                    "item": {
                        "type": "function_call_output",
                        "call_id": call_id,
                        "output": result,
                    },
                })
                # Ask OpenAI to respond with the result
                await self._send({"type": "response.create"})

                # Clean up
                self._function_call_args.pop(call_id, None)

            elif event_type == "response.done":
                if self.metal:
                    self.metal.send_state("listening")

            elif event_type == "error":
                error = event.get("error", {})
                console.print(f"[red]API Error:[/] {error.get('message', error)}")

            elif event_type == "input_audio_buffer.speech_started":
                if on_interrupt:
                    on_interrupt()  # clear playback buffer

            elif event_type == "input_audio_buffer.speech_stopped":
                pass  # user stopped speaking

    async def _send(self, data: dict):
        if self.ws:
            await self.ws.send(json.dumps(data))

    async def close(self):
        if self.ws:
            await self.ws.close()
