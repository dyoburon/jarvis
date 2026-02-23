import base64
import collections
import queue
import threading

import numpy as np
import sounddevice as sd

import config


class SkillMicCapture:
    """Push-to-talk audio capture for skill mode (local Whisper).

    Taps into the existing MicCapture stream and collects raw float32
    audio, then resamples from 24kHz to 16kHz for Whisper on stop.
    """

    SMOOTHING = 0.7  # 70% previous, 30% new (matches vibetotext)
    SILENCE_THRESHOLD = 0.08

    def __init__(self, source_rate: int = 24000, target_rate: int = 16000):
        self.source_rate = source_rate
        self.target_rate = target_rate
        self._recording = False
        self._chunks: list[np.ndarray] = []
        self.on_level: callable = None  # callback(float) — RMS audio level 0-1
        self._prev_level: float = 0.0  # For temporal smoothing

    def start_recording(self):
        self._chunks = []
        self._prev_level = 0.0
        self._recording = True

    def feed_audio(self, float32_mono: np.ndarray):
        """Called from MicCapture callback with raw float32 data."""
        if self._recording:
            self._chunks.append(float32_mono.copy())
            if self.on_level:
                rms = float(np.sqrt(np.mean(float32_mono ** 2)))
                level = min(rms * 4.0, 1.0)

                # Silence gate — smooth decay when below threshold
                if level < self.SILENCE_THRESHOLD:
                    self._prev_level *= self.SMOOTHING
                    self.on_level(self._prev_level)
                    return

                # Temporal smoothing (70% previous, 30% new)
                level = self._prev_level * self.SMOOTHING + level * (1 - self.SMOOTHING)
                self._prev_level = level
                self.on_level(level)

    def stop_recording(self) -> np.ndarray:
        """Stop and return captured audio at 16kHz float32."""
        self._recording = False
        if not self._chunks:
            return np.array([], dtype=np.float32)

        audio = np.concatenate(self._chunks)
        self._chunks = []

        # Resample 24kHz -> 16kHz
        if self.source_rate != self.target_rate:
            ratio = self.target_rate / self.source_rate
            new_length = int(len(audio) * ratio)
            indices = np.linspace(0, len(audio) - 1, new_length)
            audio = np.interp(indices, np.arange(len(audio)), audio).astype(np.float32)

        return audio


class MicCapture:
    """Captures audio from the microphone at 24kHz mono PCM16."""

    def __init__(self):
        self.audio_queue: queue.Queue[bytes] = queue.Queue()
        self._stream = None
        self._skill_capture: SkillMicCapture | None = None

    def _callback(self, indata, frames, time_info, status):
        mono = indata[:, 0]
        # Feed skill capture if active (raw float32)
        if self._skill_capture:
            self._skill_capture.feed_audio(mono)
        pcm16 = (mono * 32767).astype(np.int16)
        self.audio_queue.put(pcm16.tobytes())

    def start(self):
        self._stream = sd.InputStream(
            samplerate=config.SAMPLE_RATE,
            channels=config.CHANNELS,
            dtype="float32",
            blocksize=int(config.SAMPLE_RATE * 0.1),  # 100ms blocks
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
