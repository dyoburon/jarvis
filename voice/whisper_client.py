"""Client for vibetotext Unix domain socket transcription service."""

import asyncio
import base64
import json
import os
import socket

import numpy as np

import config

TIMEOUT = 10.0


class WhisperClient:
    """Sends audio to vibetotext's local Whisper model for transcription."""

    async def transcribe(self, audio: np.ndarray, sample_rate: int = 16000) -> str:
        """Send audio to vibetotext and get transcription.

        Args:
            audio: float32 mono numpy array
            sample_rate: sample rate (vibetotext expects 16000)

        Returns:
            Transcribed text, or empty string on error.
        """
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self._transcribe_sync, audio, sample_rate)

    def _transcribe_sync(self, audio: np.ndarray, sample_rate: int) -> str:
        """Synchronous transcription via Unix socket."""
        try:
            sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            sock.settimeout(TIMEOUT)
            sock.connect(config.VIBETOTEXT_SOCKET)

            audio_f32 = audio.astype(np.float32)
            audio_b64 = base64.b64encode(audio_f32.tobytes()).decode()

            request = json.dumps({
                "type": "transcribe",
                "audio_b64": audio_b64,
                "sample_rate": sample_rate,
            }) + "\n"
            sock.sendall(request.encode())
            sock.shutdown(socket.SHUT_WR)

            data = b""
            while True:
                chunk = sock.recv(65536)
                if not chunk:
                    break
                data += chunk

            sock.close()

            response = json.loads(data.split(b"\n")[0])
            if response.get("type") == "result":
                return response.get("text", "")
            else:
                print(f"[WHISPER] Error: {response.get('message', 'unknown')}")
                return ""

        except Exception as e:
            print(f"[WHISPER] Socket error: {e}")
            return ""

    def is_available(self) -> bool:
        """Check if the vibetotext socket server is running."""
        return os.path.exists(config.VIBETOTEXT_SOCKET)
