"""Claude Proxy client — async streaming chat via CLIProxyAPI (OpenAI-compatible)."""

import json
import logging
from typing import AsyncGenerator

import httpx

import config

_log = logging.getLogger("jarvis.claude_proxy")


class ClaudeProxyClient:
    """Async streaming client for Claude models via CLIProxyAPI."""

    def __init__(
        self,
        base_url: str = None,
        api_key: str = None,
        model: str = None,
    ):
        self.base_url = (base_url or config.CLAUDE_PROXY_BASE_URL).rstrip("/")
        self.api_key = api_key or config.CLAUDE_PROXY_API_KEY
        self.model = model or config.CLAUDE_PROXY_MODEL
        self._client = httpx.AsyncClient(timeout=120.0)

    async def stream_chat(
        self,
        messages: list[dict],
        system: str = None,
        model: str = None,
        max_tokens: int = 4096,
    ) -> AsyncGenerator[str, None]:
        """Stream a chat completion, yielding text chunks.

        Args:
            messages: List of {"role": "user"|"assistant", "content": "..."}
            system: Optional system prompt (prepended as system message)
            model: Override model for this call
            max_tokens: Max response tokens
        """
        api_messages = []
        if system:
            api_messages.append({"role": "system", "content": system})
        api_messages.extend(messages)

        payload = {
            "model": model or self.model,
            "messages": api_messages,
            "max_tokens": max_tokens,
            "stream": True,
        }

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        async with self._client.stream(
            "POST",
            f"{self.base_url}/chat/completions",
            json=payload,
            headers=headers,
        ) as response:
            if response.status_code != 200:
                body = await response.aread()
                _log.error("Proxy error %d: %s", response.status_code, body[:500])
                yield f"*(Proxy error: {response.status_code})*"
                return

            async for line in response.aiter_lines():
                if not line.startswith("data: "):
                    continue
                data = line[6:]
                if data == "[DONE]":
                    break
                try:
                    chunk = json.loads(data)
                    delta = chunk["choices"][0].get("delta", {})
                    text = delta.get("content", "")
                    if text:
                        yield text
                except (json.JSONDecodeError, KeyError, IndexError):
                    continue

    async def chat(
        self,
        messages: list[dict],
        system: str = None,
        model: str = None,
        max_tokens: int = 4096,
    ) -> str:
        """Non-streaming chat completion. Returns full response text."""
        chunks = []
        async for chunk in self.stream_chat(messages, system, model, max_tokens):
            chunks.append(chunk)
        return "".join(chunks)

    async def ask(self, prompt: str, system: str = None, model: str = None) -> str:
        """Quick single-turn question. Returns response text."""
        return await self.chat(
            [{"role": "user", "content": prompt}],
            system=system,
            model=model,
        )

    async def close(self):
        await self._client.aclose()
