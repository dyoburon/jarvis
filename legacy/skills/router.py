import asyncio
import datetime
import inspect
import json
import logging
import subprocess

from google import genai
from google.genai import types
from rich.console import Console
from rich.live import Live
from rich.markdown import Markdown
from rich.panel import Panel

import os

import config

_log = logging.getLogger("jarvis.router")
_log.setLevel(logging.DEBUG)
_fh = logging.FileHandler(os.path.join(os.path.dirname(os.path.dirname(__file__)), "jarvis_router.log"))
_fh.setFormatter(logging.Formatter("[%(asctime)s] %(message)s", datefmt="%H:%M:%S"))
_log.addHandler(_fh)
from connectors.token_tracker import TokenTracker
from connectors.claude_proxy import ClaudeProxyClient
from skills.claude_code import ClaudeCodeSession
from skills.code_assistant import CODE_SYSTEM_PROMPT, CODE_TOOLS
from skills.code_tools import TOOL_DISPATCH

console = Console()

# Gemini-format tool declarations for the default conversation session
DEFAULT_TOOLS = [
    types.Tool(function_declarations=[
        types.FunctionDeclaration(
            name="code_assistant",
            description="Help with coding tasks: read/write/edit files, run commands, search code. Use when the user asks about code, wants to make changes to projects, run scripts, debug issues, or explore codebases.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "task": {"type": "string", "description": "What the user wants to do"},
                    "project": {"type": "string", "description": "Project name or directory if specified"},
                },
                "required": ["task"],
            },
        ),
    ])
]

DEFAULT_SYSTEM_PROMPT = (
    config.SYSTEM_PROMPT + "\n\n"
    "You have a code assistant for coding tasks. Use it when the user asks about code. "
    "For casual conversation, just respond with text."
)



class SkillRouter:
    def __init__(self, metal_bridge=None):
        self.gemini = genai.Client(api_key=config.GOOGLE_API_KEY) if config.GOOGLE_API_KEY else None
        self.claude_proxy = ClaudeProxyClient()
        self.metal = metal_bridge
        self.token_tracker = TokenTracker(config.TOKEN_USAGE_DB)
        # Default conversation session (Gemini Flash)
        self.default_chat = None
        # Per-panel sessions: panel_id → session state
        self._panels: dict[int, dict] = {}
        # Session-level token/cost accumulators (shared across panels)
        self._session_prompt_tokens: int = 0
        self._session_completion_tokens: int = 0
        self._session_cost: float = 0.0
        self._session_model: str = ""

    def _get_panel(self, panel: int) -> dict:
        """Get or create panel session state."""
        if panel not in self._panels:
            self._panels[panel] = {
                "chat": None,          # Gemini chat (for non-code skills)
                "session": None,       # ClaudeCodeSession (for code assistant)
                "is_code": False,
                "skill_name": None,
                "cancelled": False,
                "pending_approval": None,
                "pending_command": None,
            }
        return self._panels[panel]

    def _build_env_context(self, project: str = "") -> str:
        """Build dynamic environment context injected into the first message."""
        lines = [
            f"Date: {datetime.date.today()}",
            f"Platform: macOS (Darwin)",
            f"Shell: zsh",
            f"Projects directory: {config.PROJECTS_DIR}",
        ]
        # Git status for the target project or jarvis by default
        git_dir = config.PROJECTS_DIR / (project if project else "jarvis")
        if git_dir.is_dir():
            try:
                result = subprocess.run(
                    ["git", "status", "--short", "--branch"],
                    cwd=str(git_dir), capture_output=True, text=True, timeout=5,
                )
                if result.returncode == 0:
                    lines.append(f"Git ({git_dir.name}): {result.stdout.strip()}")
            except Exception:
                pass
            try:
                result = subprocess.run(
                    ["git", "log", "--oneline", "-5"],
                    cwd=str(git_dir), capture_output=True, text=True, timeout=5,
                )
                if result.returncode == 0:
                    lines.append(f"Recent commits:\n{result.stdout.strip()}")
            except Exception:
                pass
        return "\n".join(lines)

    def _metal_hud(self, text: str):
        if self.metal:
            self.metal.send_hud(text)

    def _metal_state(self, state: str, name: str = None):
        if self.metal:
            self.metal.send_state(state, name)

    def _record_usage(self, chunk, model: str, session_type: str):
        """Extract usage_metadata from final stream chunk and record it."""
        if chunk is None:
            return
        meta = getattr(chunk, "usage_metadata", None)
        if meta is None:
            return
        prompt = meta.prompt_token_count or 0
        completion = meta.candidates_token_count or 0
        total = meta.total_token_count or 0
        self.token_tracker.record(
            model=model,
            session_type=session_type,
            prompt_tokens=prompt,
            completion_tokens=completion,
            total_tokens=total,
        )
        # Accumulate session totals
        self._session_prompt_tokens += prompt
        self._session_completion_tokens += completion
        self._session_model = model
        pricing = config.GEMINI_PRICING.get(model, {"input": 0, "output": 0})
        self._session_cost += (prompt * pricing["input"] + completion * pricing["output"]) / 1_000_000

    # ── Default conversation (Gemini Flash) ──

    def start_default_session(self):
        """Create the persistent Gemini Flash chat for default conversation."""
        if not self.gemini:
            console.print("[yellow]Gemini unavailable — no GOOGLE_API_KEY[/]")
            return
        self.default_chat = self.gemini.aio.chats.create(
            model=config.GEMINI_MODEL_DEFAULT,
            config=types.GenerateContentConfig(
                system_instruction=DEFAULT_SYSTEM_PROMPT,
                tools=DEFAULT_TOOLS,
            ),
        )
        console.print("[green]Default Gemini Flash session ready[/]")

    async def send_default_message(self, user_text: str) -> tuple[str, dict | None]:
        """Send message to default Flash session with tool-calling loop.

        Returns (response_text, skill_trigger_or_None).
        If a skill trigger is returned, the caller should enter skill mode.
        skill_trigger = {"tool_name": str, "arguments": str, "user_text": str}
        """
        if not self.default_chat:
            self.start_default_session()
        if not self.default_chat:
            return ("Gemini is unavailable — set GOOGLE_API_KEY in .env to enable voice routing.", None)

        full_response = ""
        message = user_text
        max_tool_calls = 3
        max_iterations = 3
        total_tool_calls = 0

        for _ in range(max_iterations):
            turn_text = ""
            pending_function_calls = []

            try:
                stream = await asyncio.wait_for(
                    self.default_chat.send_message_stream(message),
                    timeout=30.0,
                )
                last_chunk = None
                async for chunk in stream:
                    last_chunk = chunk
                    if chunk.text:
                        turn_text += chunk.text
                    if chunk.candidates:
                        for candidate in chunk.candidates:
                            if candidate.content and candidate.content.parts:
                                for part in candidate.content.parts:
                                    if part.function_call:
                                        pending_function_calls.append(part.function_call)
                self._record_usage(last_chunk, config.GEMINI_MODEL_DEFAULT, "default")
            except asyncio.TimeoutError:
                console.print("[yellow]Default session timed out (30s)[/]")
                full_response += turn_text + "\n*(Timed out.)*"
                break

            full_response += turn_text

            if not pending_function_calls:
                break

            # Process function calls
            function_response_parts = []
            for fc in pending_function_calls:
                tool_name = fc.name
                tool_args = dict(fc.args) if fc.args else {}
                total_tool_calls += 1

                # code_assistant → signal skill mode (don't execute here)
                if tool_name == "code_assistant":
                    console.print(f"\n[bold yellow]Skill triggered:[/] {tool_name}")
                    return full_response, {
                        "tool_name": tool_name,
                        "arguments": json.dumps(tool_args),
                        "user_text": user_text,
                    }

                # Data tools → execute inline
                console.print(f"  [dim]Default tool: {tool_name}[/]")
                executor = TOOL_DISPATCH.get(tool_name)
                if executor:
                    try:
                        if asyncio.iscoroutinefunction(executor):
                            result = await executor(**tool_args)
                        else:
                            result = executor(**tool_args)
                    except Exception as e:
                        result = {"error": str(e)}
                else:
                    result = {"error": f"Unknown tool: {tool_name}"}

                function_response_parts.append(
                    types.Part.from_function_response(name=tool_name, response=result)
                )

                if total_tool_calls >= max_tool_calls:
                    break

            if not function_response_parts:
                break

            message = function_response_parts

        return full_response, None

    # ── Skill sessions ──

    async def start_skill_session(self, tool_name: str, arguments: str, user_transcript: str, panel: int = 0, on_chunk=None, on_tool_activity=None) -> str:
        """Start a streaming skill chat session. Returns the full initial response."""
        if tool_name == "code_assistant":
            return await self._start_code_session(arguments, user_transcript, panel=panel, on_chunk=on_chunk, on_tool_activity=on_tool_activity)
        return f"Unknown skill: {tool_name}"

    async def send_followup(self, user_text: str, panel: int = 0, on_chunk=None, on_tool_activity=None) -> str:
        """Send a follow-up message to a specific panel's chat session."""
        ps = self._get_panel(panel)

        # Claude Code sessions
        if ps["is_code"]:
            session: ClaudeCodeSession = ps.get("session")
            if not session:
                return "No active Claude Code session"
            return await session.run(user_text, on_chunk=on_chunk, on_tool_activity=on_tool_activity)

        # Gemini skill sessions
        if not ps["chat"]:
            return "No active chat session"

        full_response = ""
        try:
            stream = await asyncio.wait_for(
                ps["chat"].send_message_stream(user_text),
                timeout=60.0,
            )
            last_chunk = None
            async for chunk in stream:
                last_chunk = chunk
                text = chunk.text
                if text:
                    full_response += text
                    if on_chunk:
                        on_chunk(text)
            self._record_usage(last_chunk, config.GEMINI_MODEL_DEFAULT, "skill")
        except asyncio.TimeoutError:
            console.print("[yellow]Gemini followup timed out (60s)[/]")
            if on_chunk:
                on_chunk("\n\n*(Request timed out.)*")

        return full_response

    def approve_command(self, approved: bool, panel: int = 0):
        """Resolve a pending run_command approval for a specific panel."""
        ps = self._get_panel(panel)
        if ps["pending_approval"] and not ps["pending_approval"].done():
            ps["pending_approval"].set_result(approved)

    def has_pending_approval(self, panel: int = 0) -> bool:
        """Check if a panel has a pending command approval."""
        ps = self._panels.get(panel, {})
        approval = ps.get("pending_approval")
        return approval is not None and not approval.done()

    def get_pending_command(self, panel: int = 0) -> str:
        """Get the pending command string for a panel."""
        return self._panels.get(panel, {}).get("pending_command", "")

    def cancel_panel(self, panel: int):
        """Cancel a specific panel's running operation."""
        ps = self._panels.get(panel)
        if ps:
            ps["cancelled"] = True
            ps["chat"] = None
            # Cancel Claude Code session if active
            session = ps.get("session")
            if session:
                asyncio.ensure_future(session.cancel())
            if ps.get("pending_approval") and not ps["pending_approval"].done():
                ps["pending_approval"].cancel()

    def close_panel(self, panel: int) -> str:
        """Close a specific panel's session and renumber remaining panels."""
        ps = self._panels.pop(panel, {})
        name = ps.get("skill_name", "Skill")
        # Close Claude Code session
        session = ps.get("session")
        if session:
            asyncio.ensure_future(session.close())
        # Renumber: shift panels above the closed one down by 1
        new_panels: dict[int, dict] = {}
        for pid, state in self._panels.items():
            new_panels[pid - 1 if pid > panel else pid] = state
        self._panels = new_panels
        return f"{name} session closed."

    def close_session(self) -> str:
        """Close all panel sessions."""
        for ps in self._panels.values():
            session = ps.get("session")
            if session:
                asyncio.ensure_future(session.close())
        self._panels.clear()
        self._session_prompt_tokens = 0
        self._session_completion_tokens = 0
        self._session_cost = 0.0
        self._session_model = ""
        return "All sessions closed."

    def get_session_status(self) -> str:
        """Format status line for the chat window."""
        # Check if any panel has a Claude Code session
        for ps in self._panels.values():
            session = ps.get("session")
            if session and isinstance(session, ClaudeCodeSession):
                return session.get_status()

        # Fallback to Gemini stats
        model = self._session_model or config.GEMINI_MODEL_DEFAULT
        session_total = self._session_prompt_tokens + self._session_completion_tokens
        if session_total >= 1_000_000:
            tokens_str = f"{session_total / 1_000_000:.1f}M"
        elif session_total >= 1_000:
            tokens_str = f"{session_total / 1_000:.1f}K"
        else:
            tokens_str = str(session_total)
        return f"{model} | {tokens_str} tokens"

    async def start_code_session_idle(self, arguments: str, user_transcript: str, panel: int = 0):
        """Initialize a Claude Code session without sending any message yet."""
        ps = self._get_panel(panel)
        ps["skill_name"] = "Bench 1"
        ps["is_code"] = True
        ps["cancelled"] = False
        console.print(f"  [dim]Starting Claude Code session (panel {panel})...[/]")

        session = ClaudeCodeSession(model=config.CLAUDE_CODE_MODEL, cwd=str(config.PROJECTS_DIR))
        await session.connect()
        ps["session"] = session
        console.print(f"  [dim]Claude Code session ready (panel {panel})[/]")

    async def send_code_initial(self, user_text: str, panel: int = 0, on_chunk=None, on_tool_activity=None) -> str:
        """Send the first message to a Claude Code session."""
        ps = self._get_panel(panel)
        session: ClaudeCodeSession = ps.get("session")
        if not session:
            return "No active Claude Code session"
        return await session.run(user_text, on_chunk=on_chunk, on_tool_activity=on_tool_activity)

    async def _start_code_session(self, arguments: str, user_transcript: str, panel: int = 0, on_chunk=None, on_tool_activity=None) -> str:
        """Start a coding agent chat session with function-calling tools."""
        params = json.loads(arguments) if arguments else {}
        task = params.get("task", user_transcript)
        project = params.get("project", "")

        if not self.gemini:
            return "Gemini is unavailable — set GOOGLE_API_KEY in .env to enable code sessions."

        ps = self._get_panel(panel)
        ps["skill_name"] = "Bench 1"
        ps["is_code"] = True
        ps["cancelled"] = False
        console.print(f"  [dim]Starting Gemini code session (panel {panel})...[/]")

        ps["chat"] = self.gemini.aio.chats.create(
            model=config.GEMINI_MODEL_CODE,
            config=types.GenerateContentConfig(
                system_instruction=CODE_SYSTEM_PROMPT,
                tools=CODE_TOOLS,
            ),
        )

        env = self._build_env_context(project)
        prompt = f"[Environment]\n{env}\n\nUser request: {task}"
        if project:
            prompt += f"\nProject: {project}"

        return await self._run_code_turn(prompt, panel, on_chunk, on_tool_activity)

    async def _run_code_turn(self, message, panel: int = 0, on_chunk=None, on_tool_activity=None) -> str:
        """Execute the agentic tool-calling loop for a specific panel.

        Streams text to on_chunk. When function calls are detected, executes
        them, notifies on_tool_activity, sends results back to Gemini, and
        repeats until Gemini returns a text-only response.
        """
        ps = self._get_panel(panel)
        ps["cancelled"] = False
        full_response = ""
        total_tool_calls = 0
        max_tool_calls = 50
        max_iterations = 30

        for iteration in range(max_iterations):
            if ps["cancelled"]:
                _log.debug("panel %d turn %d: cancelled before start", panel, iteration + 1)
                break

            turn_text = ""
            pending_function_calls = []

            try:
                if not ps["chat"]:
                    _log.warning("panel %d turn %d: no active chat", panel, iteration + 1)
                    console.print(f"[yellow]No active chat (panel {panel}) — loop exiting[/]")
                    break

                stream_timeout = 300.0  # 5 min total time for this turn
                turn_start = asyncio.get_event_loop().time()
                _log.debug("panel %d turn %d: sending to Gemini...", panel, iteration + 1)

                stream = await asyncio.wait_for(
                    ps["chat"].send_message_stream(message),
                    timeout=30.0,
                )
                _log.debug("panel %d turn %d: stream opened, reading chunks...", panel, iteration + 1)
                last_chunk = None
                chunk_count = 0
                async for chunk in stream:
                    chunk_count += 1
                    # Check total turn timeout
                    elapsed = asyncio.get_event_loop().time() - turn_start
                    if elapsed > stream_timeout:
                        _log.warning("panel %d turn %d: stream exceeded %.0fs after %d chunks", panel, iteration + 1, stream_timeout, chunk_count)
                        console.print(f"[yellow]Stream exceeded {stream_timeout}s (panel {panel}) — stopping[/]")
                        if on_chunk:
                            on_chunk("\n\n*(Stream timed out.)*")
                        break

                    last_chunk = chunk
                    if ps["cancelled"]:
                        _log.debug("panel %d turn %d: cancelled mid-stream at chunk %d", panel, iteration + 1, chunk_count)
                        break
                    if chunk.text:
                        turn_text += chunk.text
                        if on_chunk:
                            on_chunk(chunk.text)
                    if chunk.candidates:
                        for candidate in chunk.candidates:
                            if candidate.content and candidate.content.parts:
                                for part in candidate.content.parts:
                                    if part.function_call:
                                        pending_function_calls.append(part.function_call)

                elapsed = asyncio.get_event_loop().time() - turn_start
                _log.debug("panel %d turn %d: stream done — %d chunks, %d func calls, %.1fs, text=%d chars",
                           panel, iteration + 1, chunk_count, len(pending_function_calls), elapsed, len(turn_text))
                self._record_usage(last_chunk, config.GEMINI_MODEL_CODE, "code")
            except asyncio.TimeoutError:
                _log.warning("panel %d turn %d: TIMEOUT waiting for Gemini", panel, iteration + 1)
                console.print(f"[yellow]Gemini request timed out (panel {panel}, turn {iteration + 1})[/]")
                if on_chunk:
                    on_chunk("\n\n*(Request timed out.)*")
                break
            except Exception as e:
                if ps["cancelled"]:
                    _log.debug("panel %d turn %d: exception after cancel: %s", panel, iteration + 1, e)
                    break
                _log.error("panel %d turn %d: stream error: %s", panel, iteration + 1, e, exc_info=True)
                console.print(f"[red]Stream error (panel {panel}, turn {iteration + 1}):[/] {e}")
                if on_chunk:
                    on_chunk(f"\n\n*(Error: {e})*")
                break

            if ps["cancelled"]:
                break

            full_response += turn_text

            if not pending_function_calls:
                _log.debug("panel %d turn %d: no function calls — done", panel, iteration + 1)
                break

            remaining = max_tool_calls - total_tool_calls
            if remaining <= 0:
                if on_chunk:
                    on_chunk("\n\n*(Tool limit reached — ask me to continue if needed.)*")
                break
            pending_function_calls = pending_function_calls[:remaining]

            function_response_parts = []
            for fc in pending_function_calls:
                if ps["cancelled"]:
                    break

                tool_name = fc.name
                tool_args = dict(fc.args) if fc.args else {}
                total_tool_calls += 1

                if on_tool_activity:
                    on_tool_activity("start", tool_name, tool_args)

                # Gate: require user approval for run_command
                if tool_name == "run_command":
                    ps["pending_command"] = tool_args.get("command", "")
                    ps["pending_approval"] = asyncio.get_event_loop().create_future()
                    if on_tool_activity:
                        on_tool_activity("approval_request", tool_name, tool_args)
                    try:
                        approved = await ps["pending_approval"]
                    except (asyncio.CancelledError, Exception):
                        ps["pending_approval"] = None
                        ps["pending_command"] = None
                        break
                    ps["pending_approval"] = None
                    ps["pending_command"] = None
                    if not approved:
                        result = {"error": "Command denied by user."}
                        if on_tool_activity:
                            on_tool_activity("result", tool_name, result)
                        console.print(f"  [red]Command denied (panel {panel})[/]")
                        function_response_parts.append(
                            types.Part.from_function_response(name=tool_name, response=result)
                        )
                        continue

                _log.debug("panel %d tool %d/%d: executing %s", panel, total_tool_calls, max_tool_calls, tool_name)
                executor = TOOL_DISPATCH.get(tool_name)
                if executor:
                    try:
                        if asyncio.iscoroutinefunction(executor):
                            result = await asyncio.wait_for(executor(**tool_args), timeout=45.0)
                        else:
                            result = executor(**tool_args)
                    except asyncio.TimeoutError:
                        _log.warning("panel %d tool %s timed out (45s)", panel, tool_name)
                        result = {"error": f"Tool {tool_name} timed out (45s)"}
                    except Exception as e:
                        _log.error("panel %d tool %s error: %s", panel, tool_name, e)
                        result = {"error": str(e)}
                else:
                    result = {"error": f"Unknown tool: {tool_name}"}

                has_error = "error" in result if isinstance(result, dict) else False
                _log.debug("panel %d tool %s done — error=%s", panel, tool_name, has_error)

                if on_tool_activity:
                    on_tool_activity("result", tool_name, result)

                console.print(f"  [dim]Tool {tool_name} ({total_tool_calls}/{max_tool_calls}) [panel {panel}][/]")
                function_response_parts.append(
                    types.Part.from_function_response(name=tool_name, response=result)
                )

            if ps["cancelled"] or not function_response_parts:
                _log.debug("panel %d turn %d: loop ending — cancelled=%s, parts=%d",
                           panel, iteration + 1, ps["cancelled"], len(function_response_parts))
                break

            # Approaching limits — tell Gemini to wrap up
            if iteration >= max_iterations - 2 or total_tool_calls >= max_tool_calls - 2:
                function_response_parts.append(
                    types.Part.from_text(
                        "[SYSTEM: You are approaching the tool call limit. "
                        "Summarize your findings and respond to the user NOW with text.]"
                    )
                )
                _log.debug("panel %d turn %d: injected wrap-up nudge (iter=%d, tools=%d)",
                           panel, iteration + 1, iteration, total_tool_calls)

            message = function_response_parts

        _log.debug("panel %d _run_code_turn done — %d turns, %d tool calls, response=%d chars",
                    panel, iteration + 1, total_tool_calls, len(full_response))
        return full_response
