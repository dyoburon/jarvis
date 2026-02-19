import base64
import collections
import queue
import threading

import numpy as np
import sounddevice as sd

import config


class MicCapture:
    """Captures audio from the microphone at 24kHz mono PCM16."""

    def __init__(self):
        self.audio_queue: queue.Queue[bytes] = queue.Queue()
        self._stream = None

    def _callback(self, indata, frames, time_info, status):
        pcm16 = (indata[:, 0] * 32767).astype(np.int16)
        self.audio_queue.put(pcm16.tobytes())

    def start(self):
        self._stream = sd.InputStream(
            samplerate=config.SAMPLE_RATE,
            channels=config.CHANNELS,
            dtype="float32",
            blocksize=int(config.SAMPLE_RATE * config.CHUNK_MS / 1000),
            callback=self._callback,
        )
        self._stream.start()

    def stop(self):
        if self._stream:
            self._stream.stop()
            self._stream.close()
            self._stream = None

    def get_chunk_b64(self) -> str | None:
        """Get next audio chunk as base64. Non-blocking, returns None if empty."""
        try:
            data = self.audio_queue.get_nowait()
            return base64.b64encode(data).decode()
        except queue.Empty:
            return None


class AudioPlayer:
    """Plays PCM16 audio from OpenAI Realtime using a continuous byte buffer.

    Uses a deque of raw bytes as a ring buffer. The sounddevice callback
    pulls from it continuously, outputting silence only when truly empty.
    """

    def __init__(self):
        self._stream = None
        self._lock = threading.Lock()
        self._buf = bytearray()

    def start(self):
        self._stream = sd.OutputStream(
            samplerate=config.SAMPLE_RATE,
            channels=config.CHANNELS,
            dtype="int16",
            blocksize=1200,  # 50ms blocks for smooth playback
            callback=self._callback,
        )
        self._stream.start()

    def _callback(self, outdata, frames, time_info, status):
        bytes_needed = frames * 2  # int16 = 2 bytes per sample
        with self._lock:
            available = len(self._buf)
            if available >= bytes_needed:
                raw = bytes(self._buf[:bytes_needed])
                del self._buf[:bytes_needed]
            elif available > 0:
                # Partial — use what we have, pad rest with silence
                raw = bytes(self._buf) + b'\x00' * (bytes_needed - available)
                self._buf.clear()
            else:
                raw = b'\x00' * bytes_needed

        outdata[:, 0] = np.frombuffer(raw, dtype=np.int16)

    def add_audio(self, b64_data: str):
        """Add base64-encoded PCM16 audio to the playback buffer."""
        raw = base64.b64decode(b64_data)
        with self._lock:
            self._buf.extend(raw)

    def clear(self):
        """Clear the playback buffer (e.g. on interruption)."""
        with self._lock:
            self._buf.clear()

    def stop(self):
        if self._stream:
            self._stream.stop()
            self._stream.close()
            self._stream = None
