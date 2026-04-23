"""Built-in Whisper transcription server for Jarvis.

Runs a Unix domain socket server with a whisper.cpp backend,
eliminating the external vibetotext dependency.

Protocol (newline-delimited JSON):
  Request:  {"type": "transcribe", "audio_b64": "<base64 float32>", "sample_rate": 16000}
  Response: {"type": "result", "text": "...", "duration_ms": 450}
  Error:    {"type": "error", "message": "..."}
"""

import base64
import json
import logging
import os
import re
import socket
import threading
import time

import numpy as np
from pywhispercpp.model import Model

log = logging.getLogger("jarvis.whisper_server")

# Vocabulary biasing prompt — helps Whisper recognize technical terms
TECH_PROMPT = (
    "This is a software engineer dictating code and technical documentation. "
    "They frequently discuss: APIs, databases, frontend frameworks, backend services, "
    "cloud infrastructure, and AI/ML systems. Use programming terminology and proper "
    "capitalization for technical terms.\n\n"
    "Common terms: Firebase, Firestore, MongoDB, PostgreSQL, MySQL, Redis, SQLite, "
    "API, REST, GraphQL, gRPC, WebSocket, JSON, YAML, XML, HTML, CSS, SCSS, "
    "JavaScript, TypeScript, Python, Rust, Go, Java, C++, Swift, Kotlin, "
    "React, Vue, Angular, Svelte, Next.js, Nuxt, Remix, Astro, "
    "Node.js, Deno, Bun, npm, yarn, pnpm, webpack, Vite, esbuild, Rollup, "
    "Docker, Kubernetes, K8s, Helm, Terraform, Ansible, Jenkins, CircleCI, "
    "AWS, S3, EC2, Lambda, DynamoDB, CloudFront, Route53, ECS, EKS, "
    "GCP, BigQuery, Cloud Run, Cloud Functions, Pub/Sub, "
    "Azure, Vercel, Netlify, Railway, Render, Fly.io, Cloudflare, "
    "Git, GitHub, GitLab, Bitbucket, PR, pull request, merge, rebase, cherry-pick, "
    "CI/CD, DevOps, SRE, microservices, monorepo, serverless, edge functions, "
    "useState, useEffect, useContext, useRef, useMemo, useCallback, useReducer, "
    "Redux, Zustand, Jotai, Recoil, MobX, XState, "
    "Prisma, Drizzle, TypeORM, Sequelize, Knex, SQLAlchemy, "
    "tRPC, Zod, Yup, Joi, Express, Fastify, Hono, FastAPI, Flask, Django, "
    "Tailwind, styled-components, Emotion, CSS Modules, Sass, "
    "Jest, Vitest, Cypress, Playwright, Testing Library, "
    "ESLint, Prettier, Biome, TypeScript, TSConfig, "
    "OAuth, JWT, session, cookie, CORS, CSRF, XSS, SQL injection, "
    "Claude, Anthropic, OpenAI, GPT, Gemini, Llama, Mistral, "
    "LLM, embedding, vector database, Pinecone, Weaviate, ChromaDB, Qdrant, "
    "RAG, retrieval, chunking, tokenization, fine-tuning, RLHF, prompt engineering, "
    "Whisper, transcription, TTS, speech-to-text, ASR, NLP, NLU, "
    "regex, cron, UUID, Base64, SHA, MD5, RSA, AES, TLS, SSL, HTTPS."
)

# Whisper hallucination artifacts to filter out
_ARTIFACT_RE = re.compile(
    r"\[(?:end|blank_audio|silence|music|applause)\]",
    re.IGNORECASE,
)


class WhisperTranscriber:
    """Whisper.cpp transcriber with lazy model loading."""

    def __init__(self, model_name: str = "small"):
        self.model_name = model_name
        self._model = None

    @property
    def model(self):
        if self._model is None:
            log.info(f"Loading whisper.cpp model '{self.model_name}'...")
            start = time.time()
            self._model = Model(self.model_name, print_progress=False)
            log.info(f"Model loaded in {time.time() - start:.1f}s")
        return self._model

    def transcribe(self, audio: np.ndarray, sample_rate: int = 16000) -> str:
        if len(audio) == 0:
            return ""
        audio = audio.astype(np.float32)
        segments = self.model.transcribe(audio, language="en", initial_prompt=TECH_PROMPT)
        text = " ".join(seg.text for seg in segments).strip()
        text = _ARTIFACT_RE.sub("", text)
        text = re.sub(r"\s+", " ", text).strip()
        return text


class WhisperServer:
    """Unix domain socket server for Whisper transcription."""

    def __init__(self, socket_path: str, model_name: str = "small"):
        self.socket_path = socket_path
        self._transcriber = WhisperTranscriber(model_name=model_name)
        self._lock = threading.Lock()
        self._server_socket = None
        self._running = False

    def start(self):
        """Start the socket server in a daemon thread."""
        if os.path.exists(self.socket_path):
            os.unlink(self.socket_path)

        self._server_socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self._server_socket.bind(self.socket_path)
        self._server_socket.listen(2)
        self._server_socket.settimeout(1.0)
        self._running = True

        thread = threading.Thread(target=self._serve_loop, daemon=True)
        thread.start()
        log.info(f"Whisper server listening on {self.socket_path}")

    def stop(self):
        """Stop the server and clean up the socket file."""
        self._running = False
        if self._server_socket:
            self._server_socket.close()
        if os.path.exists(self.socket_path):
            os.unlink(self.socket_path)

    def _serve_loop(self):
        while self._running:
            try:
                conn, _ = self._server_socket.accept()
                threading.Thread(
                    target=self._handle_client, args=(conn,), daemon=True
                ).start()
            except socket.timeout:
                continue
            except OSError:
                break

    def _handle_client(self, conn: socket.socket):
        try:
            conn.settimeout(30.0)
            data = b""
            while True:
                chunk = conn.recv(65536)
                if not chunk:
                    break
                data += chunk
                if b"\n" in data:
                    break

            line = data.split(b"\n")[0]
            request = json.loads(line)

            if request.get("type") != "transcribe":
                self._send(conn, {"type": "error", "message": f"Unknown type: {request.get('type')}"})
                return

            audio_b64 = request.get("audio_b64", "")
            sample_rate = request.get("sample_rate", 16000)
            raw = base64.b64decode(audio_b64)
            audio = np.frombuffer(raw, dtype=np.float32)

            if len(audio) == 0:
                self._send(conn, {"type": "error", "message": "Empty audio"})
                return

            start = time.time()
            with self._lock:
                text = self._transcriber.transcribe(audio, sample_rate=sample_rate)
            duration_ms = int((time.time() - start) * 1000)

            self._send(conn, {"type": "result", "text": text, "duration_ms": duration_ms})

        except Exception as e:
            log.error(f"Client handler error: {e}", exc_info=True)
            try:
                self._send(conn, {"type": "error", "message": str(e)})
            except Exception:
                pass
        finally:
            conn.close()

    def _send(self, conn: socket.socket, data: dict):
        try:
            conn.sendall((json.dumps(data) + "\n").encode())
        except Exception:
            pass
