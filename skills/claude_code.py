"""Claude Code Agent SDK integration for Jarvis code assistant sessions."""

import asyncio
import logging
import os

from claude_agent_sdk import (
    ClaudeSDKClient,
    ClaudeAgentOptions,
    AssistantMessage,
    UserMessage,
    SystemMessage,
    ResultMessage,
    TextBlock,
    ToolUseBlock,
    ToolResultBlock,
)
from claude_agent_sdk.types import StreamEvent

import config

_log = logging.getLogger("jarvis.claude_code")
_log.setLevel(logging.DEBUG)
_fh = logging.FileHandler("/tmp/jarvis_claude_code.log")
_fh.setFormatter(logging.Formatter("[%(asctime)s] %(message)s", datefmt="%H:%M:%S"))
_log.addHandler(_fh)

# Claude Code tool → Jarvis activity category
_TOOL_CATEGORIES = {
    "Read": "read", "Edit": "edit", "Write": "write",
    "Bash": "run", "Grep": "search", "Glob": "list",
    "WebSearch": "search", "WebFetch": "data",
    "Task": "tool", "NotebookEdit": "edit",
}

JARVIS_SYSTEM_PROMPT = (
    "You are Jarvis, Dylan's personal AI coding assistant. "
    "The user interacts via a chat window where tool calls are shown as a "
    "color-coded activity feed. Before making tool calls, briefly explain "
    "what you're about to do and why (1 line). "
    "The projects directory is ~/Desktop/projects/. Be concise and direct. "
    "Be concise and direct. Keep responses short — this is a chat window. "
    "After completing a task, always give a brief summary of what was done."
)


def _format_tool_start(tool_name: str, tool_input: dict) -> tuple[str, str]:
    """Return (category, human_description) for a Claude Code tool call."""
    category = _TOOL_CATEGORIES.get(tool_name, "tool")
    prefix = "/Users/dylan/Desktop/projects/"

    def short(path: str) -> str:
        return path[len(prefix):] if path.startswith(prefix) else path

    if tool_name == "Read":
        return category, f"Read {short(tool_input.get('file_path', ''))}"
    elif tool_name == "Edit":
        path = short(tool_input.get("file_path", ""))
        old = tool_input.get("old_string", "")
        preview = (old[:50].replace("\n", " ") + "...") if len(old) > 50 else old.replace("\n", " ")
        return category, f"Edit {path}\n  find: {preview}"
    elif tool_name == "Write":
        path = short(tool_input.get("file_path", ""))
        size = len(tool_input.get("content", ""))
        return category, f"Write {path} ({size} chars)"
    elif tool_name == "Bash":
        return category, f"$ {tool_input.get('command', '')}"
    elif tool_name == "Grep":
        pattern = tool_input.get("pattern", "")
        path = short(tool_input.get("path", "."))
        return category, f"Search /{pattern}/ in {path}"
    elif tool_name == "Glob":
        return category, f"Glob {tool_input.get('pattern', '*')}"
    elif tool_name == "WebSearch":
        return category, f"Search: {tool_input.get('query', '')}"
    elif tool_name == "WebFetch":
        return category, f"Fetch: {tool_input.get('url', '')}"
    elif tool_name == "Task":
        desc = tool_input.get("description", tool_input.get("prompt", "")[:60])
        return category, f"Subagent: {desc}"
    return category, tool_name


def _format_tool_result(tool_name: str, content) -> str:
    """Format a tool result for display."""
    if content is None:
        return "(no output)"
    if isinstance(content, list):
        # content blocks from API — extract text, skip binary (images, etc.)
        texts = [b.get("text", "") for b in content if isinstance(b, dict) and b.get("type") == "text"]
        if texts:
            content = "\n".join(texts)
        else:
            types = [b.get("type", "?") for b in content if isinstance(b, dict)]
            return f"({', '.join(types)} content)" if types else "(no text output)"
    text = str(content)
    lines = text.split("\n")
    if len(lines) > 30:
        return "\n".join(lines[:20] + [f"  ... ({len(lines) - 23} more lines)"] + lines[-3:])
    if len(text) > 1500:
        return text[:1500] + "..."
    return text


class ClaudeCodeSession:
    """Manages a Claude Code Agent SDK session for a single Jarvis panel."""

    def __init__(self, model: str | None = None, cwd: str | None = None):
        self.model = model or config.CLAUDE_CODE_MODEL
        self.cwd = cwd or str(config.PROJECTS_DIR)
        self._client: ClaudeSDKClient | None = None
        self._connected = False
        self.session_id: str | None = None
        self.total_cost: float = 0.0
        self.total_turns: int = 0
        self.cancelled = False
        self._has_pending_result = False
        self._subagent_depth: int = 0
        self._subagent_op_count: int = 0
        self._task_tool_ids: set[str] = set()  # ToolUseBlock IDs for Task tools

    async def connect(self):
        """Initialize the SDK client."""
        # Strip nesting protection vars and API key so OAuth token is used
        env = {
            "CLAUDECODE": "",
            "CLAUDE_CODE_ENTRYPOINT": "",
            "ANTHROPIC_API_KEY": "",  # Clear so OAuth token takes precedence
        }
        # Pass through OAuth token
        oauth = os.environ.get("CLAUDE_CODE_OAUTH_TOKEN")
        if oauth:
            env["CLAUDE_CODE_OAUTH_TOKEN"] = oauth

        options = ClaudeAgentOptions(
            model=self.model,
            system_prompt={"type": "preset", "preset": "claude_code", "append": JARVIS_SYSTEM_PROMPT},
            allowed_tools=["Read", "Write", "Edit", "Bash", "Glob", "Grep",
                           "WebSearch", "WebFetch", "NotebookEdit"],
            permission_mode="acceptEdits",
            cwd=self.cwd,
            max_turns=30,
            env=env,
            include_partial_messages=True,
        )

        self._client = ClaudeSDKClient(options=options)
        await self._client.connect()
        self._connected = True
        _log.debug("Claude Code session connected (model=%s, cwd=%s)", self.model, self.cwd)

    async def run(self, prompt: str, on_chunk=None, on_tool_activity=None) -> str:
        """Send a prompt and stream results back via callbacks.

        on_chunk(text) — called with text fragments as they arrive
        on_tool_activity(event, tool_name, data) — called for tool start/result
        """
        if not self._connected:
            await self.connect()

        self.cancelled = False
        self._subagent_depth = 0
        self._subagent_op_count = 0
        self._task_tool_ids.clear()
        full_text = ""

        # Drain any stale ResultMessage from a previous turn before sending.
        # Subagent (Task) tool runs can take minutes, so use a generous timeout.
        if self._has_pending_result:
            _log.debug("Draining stale ResultMessage from previous turn")
            try:
                drain_iter = self._client.receive_response().__aiter__()
                while True:
                    msg = await asyncio.wait_for(drain_iter.__anext__(), timeout=120.0)
                    _log.debug("Drained: %s", type(msg).__name__)
                    if isinstance(msg, ResultMessage):
                        # Only update session_id from parent results
                        if not self.session_id or msg.session_id == self.session_id:
                            self.session_id = msg.session_id
                            if msg.total_cost_usd:
                                self.total_cost += msg.total_cost_usd
                            self.total_turns += msg.num_turns
                            break
                        else:
                            _log.debug("Drained subagent ResultMessage (session=%s) — continuing",
                                       msg.session_id)
            except (StopAsyncIteration, asyncio.TimeoutError, Exception) as e:
                _log.debug("Drain finished: %s", e)
            self._has_pending_result = False

        _log.debug("Sending prompt: %s", prompt[:200])
        await self._client.query(prompt)

        got_result = False
        response_iter = self._client.receive_response().__aiter__()
        while True:
            try:
                message = await response_iter.__anext__()
            except StopAsyncIteration:
                break
            except asyncio.CancelledError:
                _log.debug("Task cancelled")
                break
            except Exception as e:
                err_str = str(e)
                if "unknown message type" in err_str.lower():
                    _log.debug("Skipping unknown message type: %s", err_str)
                    continue
                # Fatal error — log and surface to user
                _log.error("Error in receive loop: %s", e, exc_info=True)
                if "rate_limit" in err_str.lower():
                    if on_chunk:
                        on_chunk("\n\n*Rate limited — wait a moment and try again.*")
                else:
                    if on_chunk:
                        on_chunk(f"\n\n*(Error: {e})*")
                break

            # Log every message type for debugging
            _log.debug("MSG type=%s repr=%s", type(message).__name__, repr(message)[:300])

            if self.cancelled:
                _log.debug("Cancelled — breaking out of receive loop")
                break

            if isinstance(message, SystemMessage):
                if message.subtype == "init" and hasattr(message, "data"):
                    sid = message.data.get("session_id")
                    if sid:
                        self.session_id = sid
                        _log.debug("Session ID: %s", sid)
                elif message.subtype == "task_started" and hasattr(message, "data"):
                    desc = message.data.get("description", "subagent")
                    _log.debug("Subagent started: %s (task_id=%s)", desc, message.data.get("task_id"))
                    # Don't send activity — ToolUseBlock already displayed "Subagent: ..."

            elif isinstance(message, AssistantMessage):
                for block in message.content:
                    if isinstance(block, ToolUseBlock):
                        if on_tool_activity:
                            if self._subagent_depth == 0:
                                on_tool_activity("start", block.name, block.input)
                            else:
                                # Inside a sub-agent — collapse into progress update
                                self._subagent_op_count += 1
                                on_tool_activity("subagent_tool", block.name, block.input)
                        if block.name == "Task":
                            self._subagent_depth += 1
                            self._subagent_op_count = 0
                            self._task_tool_ids.add(block.id)
                            _log.debug("Task tool started (id=%s) — depth now %d", block.id, self._subagent_depth)
                        _log.debug("Tool use: %s %s (depth=%d)", block.name, str(block.input)[:200], self._subagent_depth)
                    elif isinstance(block, TextBlock) and block.text:
                        # Text already streamed via StreamEvent text_deltas
                        # (include_partial_messages=True); skip to avoid double-send
                        _log.debug("Assistant text block (already streamed): %s", block.text[:200])

            elif isinstance(message, UserMessage):
                if hasattr(message, "content"):
                    content = message.content if isinstance(message.content, list) else [message.content]
                    for block in content:
                        if isinstance(block, ToolResultBlock):
                            is_task_result = hasattr(block, 'tool_use_id') and block.tool_use_id in self._task_tool_ids
                            result_text = _format_tool_result("", block.content)
                            if is_task_result:
                                self._task_tool_ids.discard(block.tool_use_id)
                                _log.debug("Task tool result (id=%s): %s", block.tool_use_id, result_text[:200])
                                # Surface a summary of subagent's final output
                                if on_tool_activity:
                                    on_tool_activity("subagent_result", "", {"summary": result_text, "is_error": block.is_error})
                            elif self._subagent_depth > 0:
                                # Forward subagent internal tool results instead of suppressing
                                if on_tool_activity:
                                    on_tool_activity("subagent_result", "", {"summary": result_text, "is_error": block.is_error, "depth": self._subagent_depth})
                            elif on_tool_activity:
                                on_tool_activity("result", "", {"summary": result_text, "is_error": block.is_error})
                            _log.debug("Tool result (error=%s, depth=%d): %s", block.is_error, self._subagent_depth, str(block.content)[:200])

            elif isinstance(message, StreamEvent):
                event = message.event
                if isinstance(event, dict):
                    delta = event.get("delta", {})
                    if delta.get("type") == "text_delta":
                        text = delta.get("text", "")
                        if text:
                            full_text += text
                            if on_chunk:
                                on_chunk(text)

            elif isinstance(message, ResultMessage):
                is_parent = not self.session_id or message.session_id == self.session_id
                if is_parent:
                    self.session_id = message.session_id
                    got_result = True
                else:
                    # Sub-agent completed — decrement depth
                    self._subagent_depth = max(0, self._subagent_depth - 1)
                    _log.debug("Subagent ResultMessage — depth now %d, ops=%d",
                               self._subagent_depth, self._subagent_op_count)
                    if self._subagent_depth == 0 and on_tool_activity:
                        on_tool_activity("subagent_done", "", {"op_count": self._subagent_op_count})
                if message.total_cost_usd:
                    self.total_cost += message.total_cost_usd
                self.total_turns += message.num_turns
                _log.debug("Result: parent=%s, turns=%d, cost=$%.4f, error=%s",
                           is_parent, message.num_turns, message.total_cost_usd or 0, message.is_error)

        # Track if ResultMessage was missed — will drain it before next query
        if not got_result:
            _log.debug("No ResultMessage received — will drain before next query")
            self._has_pending_result = True

        return full_text

    async def cancel(self):
        """Cancel the current operation."""
        self.cancelled = True
        if self._client:
            try:
                await self._client.interrupt()
            except Exception as e:
                _log.debug("Interrupt error (ok): %s", e)

    async def close(self):
        """Disconnect the client."""
        if self._client:
            try:
                await self._client.disconnect()
            except Exception as e:
                _log.debug("Disconnect error (ok): %s", e)
            self._client = None
            self._connected = False

    def get_status(self) -> str:
        """Format status string for the chat window."""
        model = f"claude-{self.model}"
        return f"{model} | {self.total_turns} turns"
