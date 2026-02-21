import asyncio
import inspect
import json

from google import genai
from google.genai import types
from rich.console import Console
from rich.live import Live
from rich.markdown import Markdown
from rich.panel import Panel

import config
from skills.code_assistant import CODE_SYSTEM_PROMPT, CODE_TOOLS
from skills.code_tools import TOOL_DISPATCH
from skills.domains import DomainSkill
from skills.firewall import FirewallSkill
from skills.papers import PaperSkill
from skills.vibetotext import VibeToTextSkill

console = Console()

# Tool definitions for OpenAI Realtime session
TOOLS = [
    {
        "type": "function",
        "name": "get_domain_dashboard",
        "description": "Get today's domain drop hunting results: matched domains, zone stats, disappeared domains",
        "parameters": {
            "type": "object",
            "properties": {
                "date": {"type": "string", "description": "Date (YYYY-MM-DD), defaults to today"},
                "min_score": {"type": "number", "description": "Minimum match score filter"},
            },
        },
    },
    {
        "type": "function",
        "name": "get_paper_dashboard",
        "description": "Get today's research paper matches from arXiv, grouped by interest area",
        "parameters": {
            "type": "object",
            "properties": {
                "date": {"type": "string", "description": "Date (YYYY-MM-DD), defaults to today"},
                "interest": {"type": "string", "description": "Filter to specific research interest"},
            },
        },
    },
    {
        "type": "function",
        "name": "get_firewall_status",
        "description": "Get chat moderation stats from the Great Firewall: recent messages, blocked count",
        "parameters": {"type": "object", "properties": {}},
    },
    {
        "type": "function",
        "name": "get_vibetotext_stats",
        "description": "Get voice transcription statistics: total words dictated, sessions, WPM",
        "parameters": {
            "type": "object",
            "properties": {
                "limit": {"type": "integer", "description": "Number of recent entries"},
            },
        },
    },
    {
        "type": "function",
        "name": "get_system_overview",
        "description": "Get a full overview across all connected systems",
        "parameters": {"type": "object", "properties": {}},
    },
    {
        "type": "function",
        "name": "code_assistant",
        "description": "Help with coding tasks: read/write/edit files, run commands, search code. Use when the user asks about code, wants to make changes to projects, run scripts, debug issues, or explore codebases.",
        "parameters": {
            "type": "object",
            "properties": {
                "task": {"type": "string", "description": "What the user wants to do"},
                "project": {"type": "string", "description": "Project name or directory if specified"},
            },
            "required": ["task"],
        },
    },
]

# Skill instances
_skills = {
    "get_domain_dashboard": DomainSkill(),
    "get_paper_dashboard": PaperSkill(),
    "get_firewall_status": FirewallSkill(),
    "get_vibetotext_stats": VibeToTextSkill(),
}


# Register data skill executors so Gemini can call them as tools
def _make_data_executor(skill):
    async def executor(**params):
        return await skill.fetch_data(**params)
    return executor


for _name, _skill in _skills.items():
    TOOL_DISPATCH[_name] = _make_data_executor(_skill)


async def _system_overview_executor(**params):
    all_data = {}
    for name, skill in _skills.items():
        try:
            all_data[skill.name] = await skill.fetch_data()
        except Exception as e:
            all_data[skill.name] = {"error": str(e)}
    return all_data


TOOL_DISPATCH["get_system_overview"] = _system_overview_executor


class SkillRouter:
    def __init__(self, metal_bridge=None):
        self.gemini = genai.Client(api_key=config.GOOGLE_API_KEY)
        self.metal = metal_bridge
        # Streaming chat session for skill mode
        self.active_chat = None
        self.active_skill_name: str | None = None
        self.active_chat_is_code: bool = False
        self.cancelled = False  # set True to abort _run_code_turn

    def _metal_hud(self, text: str):
        if self.metal:
            self.metal.send_hud(text)

    def _metal_state(self, state: str, name: str = None):
        if self.metal:
            self.metal.send_state(state, name)

    async def start_skill_session(self, tool_name: str, arguments: str, user_transcript: str, on_chunk=None, on_tool_activity=None) -> str:
        """Start a streaming skill chat session. Returns the full initial response."""
        if tool_name == "code_assistant":
            return await self._start_code_session(arguments, user_transcript, on_chunk, on_tool_activity)

        params = json.loads(arguments) if arguments else {}

        if tool_name == "get_system_overview":
            all_data = {}
            for name, skill in _skills.items():
                try:
                    all_data[skill.name] = await skill.fetch_data()
                except Exception as e:
                    all_data[skill.name] = {"error": str(e)}
            skill_name = "System Overview"
            prompt = (
                "You are Jarvis, reporting on all connected systems.\n"
                f"User asked: \"{user_transcript}\"\n\n"
                f"Data:\n{json.dumps(all_data, indent=2, default=str)}\n\n"
                "Provide a detailed analysis. Use bullet points. Be specific with numbers."
            )
        else:
            skill = _skills.get(tool_name)
            if not skill:
                return f"Unknown skill: {tool_name}"
            skill_name = skill.name
            data = await skill.fetch_data(**params)
            if "error" in data:
                return data["error"]
            prompt = skill.format_prompt(data, user_transcript or tool_name)

        self.active_skill_name = skill_name
        console.print(f"  [dim]Starting Gemini chat for {skill_name}...[/]")

        self.active_chat = self.gemini.aio.chats.create(
            model=config.GEMINI_MODEL,
            config={"system_instruction": (
                "You are Jarvis, a personal AI assistant. You are in a text chat window "
                "displayed on screen. The user can see your responses as text. "
                "Be detailed but well-formatted. Use bullet points and short paragraphs. "
                "The user may ask follow-up questions about the data.\n\n"
                "When data has interesting patterns, include a visualization using a "
                "fenced code block with language 'chart' containing JSON:\n"
                "```chart\n"
                '{"type":"bar","title":"Chart Title","labels":["A","B","C"],"values":[10,20,30]}\n'
                "```\n"
                "Supported types: bar, line, pie. Keep labels short. "
                "Always include text analysis alongside charts, never just a chart alone."
            )},
        )

        full_response = ""
        try:
            stream = await asyncio.wait_for(
                self.active_chat.send_message_stream(prompt),
                timeout=60.0,
            )
            async for chunk in stream:
                text = chunk.text
                if text:
                    full_response += text
                    if on_chunk:
                        on_chunk(text)
        except asyncio.TimeoutError:
            console.print("[yellow]Gemini request timed out (60s)[/]")
            if on_chunk:
                on_chunk("\n\n*(Request timed out.)*")

        return full_response

    async def send_followup(self, user_text: str, on_chunk=None, on_tool_activity=None) -> str:
        """Send a follow-up message in the active chat session."""
        if not self.active_chat:
            return "No active chat session"

        if self.active_chat_is_code:
            # If previous turn ended mid-tool-loop (chat expects function_response
            # but we're sending user text), reset the chat to avoid 400 errors.
            try:
                return await self._run_code_turn(user_text, on_chunk, on_tool_activity)
            except Exception as e:
                if "function response turn" in str(e).lower():
                    console.print("  [yellow]Resetting code session after interrupted tool loop[/]")
                    # Recreate the chat session (preserves nothing but avoids the error)
                    self.active_chat = self.gemini.aio.chats.create(
                        model=config.GEMINI_MODEL,
                        config=types.GenerateContentConfig(
                            system_instruction=CODE_SYSTEM_PROMPT,
                            tools=CODE_TOOLS,
                        ),
                    )
                    return await self._run_code_turn(user_text, on_chunk, on_tool_activity)
                raise

        full_response = ""
        try:
            stream = await asyncio.wait_for(
                self.active_chat.send_message_stream(user_text),
                timeout=60.0,
            )
            async for chunk in stream:
                text = chunk.text
                if text:
                    full_response += text
                    if on_chunk:
                        on_chunk(text)
        except asyncio.TimeoutError:
            console.print("[yellow]Gemini followup timed out (60s)[/]")
            if on_chunk:
                on_chunk("\n\n*(Request timed out.)*")

        return full_response

    def close_session(self) -> str:
        """Close the active chat session."""
        name = self.active_skill_name or "Skill"
        self.active_chat = None
        self.active_skill_name = None
        self.active_chat_is_code = False
        return f"{name} session closed."

    async def start_code_session_idle(self, arguments: str, user_transcript: str):
        """Initialize a code session without sending any message yet."""
        self.active_skill_name = "Code Assistant"
        self.active_chat_is_code = True
        console.print(f"  [dim]Starting Gemini code session...[/]")

        self.active_chat = self.gemini.aio.chats.create(
            model=config.GEMINI_MODEL,
            config=types.GenerateContentConfig(
                system_instruction=CODE_SYSTEM_PROMPT,
                tools=CODE_TOOLS,
            ),
        )

    async def send_code_initial(self, user_text: str, on_chunk=None, on_tool_activity=None) -> str:
        """Send the first message to an already-initialized code session."""
        prompt = f"User request: {user_text}"
        return await self._run_code_turn(prompt, on_chunk, on_tool_activity)

    async def _start_code_session(self, arguments: str, user_transcript: str, on_chunk=None, on_tool_activity=None) -> str:
        """Start a coding agent chat session with function-calling tools."""
        params = json.loads(arguments) if arguments else {}
        task = params.get("task", user_transcript)
        project = params.get("project", "")

        self.active_skill_name = "Code Assistant"
        self.active_chat_is_code = True
        console.print(f"  [dim]Starting Gemini code session...[/]")

        self.active_chat = self.gemini.aio.chats.create(
            model=config.GEMINI_MODEL,
            config=types.GenerateContentConfig(
                system_instruction=CODE_SYSTEM_PROMPT,
                tools=CODE_TOOLS,
            ),
        )

        prompt = f"User request: {task}"
        if project:
            prompt += f"\nProject: {project}"
        prompt += f'\n\nOriginal voice transcript: "{user_transcript}"'

        return await self._run_code_turn(prompt, on_chunk, on_tool_activity)

    async def _run_code_turn(self, message, on_chunk=None, on_tool_activity=None) -> str:
        """Execute the agentic tool-calling loop.

        Streams text to on_chunk. When function calls are detected, executes
        them, notifies on_tool_activity, sends results back to Gemini, and
        repeats until Gemini returns a text-only response.

        Hard limit: 8 total tool calls per turn to prevent runaway exploration.
        """
        self.cancelled = False
        full_response = ""
        total_tool_calls = 0
        max_tool_calls = 3
        max_iterations = 3

        for _ in range(max_iterations):
            if self.cancelled:
                break

            turn_text = ""
            pending_function_calls = []

            try:
                if not self.active_chat:
                    break
                stream = await asyncio.wait_for(
                    self.active_chat.send_message_stream(message),
                    timeout=60.0,
                )
                async for chunk in stream:
                    if self.cancelled:
                        break
                    # Extract text parts
                    if chunk.text:
                        turn_text += chunk.text
                        if on_chunk:
                            on_chunk(chunk.text)
                    # Collect function calls from response parts
                    if chunk.candidates:
                        for candidate in chunk.candidates:
                            if candidate.content and candidate.content.parts:
                                for part in candidate.content.parts:
                                    if part.function_call:
                                        pending_function_calls.append(part.function_call)
            except asyncio.TimeoutError:
                console.print("[yellow]Gemini request timed out (60s)[/]")
                if on_chunk:
                    on_chunk("\n\n*(Request timed out.)*")
                break
            except Exception:
                if self.cancelled:
                    break
                raise

            if self.cancelled:
                break

            full_response += turn_text

            if not pending_function_calls:
                break

            # Enforce hard cap on total tool calls
            remaining = max_tool_calls - total_tool_calls
            if remaining <= 0:
                if on_chunk:
                    on_chunk("\n\n*(Tool limit reached — ask me to continue if needed.)*")
                break
            # Trim to what we can still execute
            pending_function_calls = pending_function_calls[:remaining]

            # Execute each function call
            function_response_parts = []
            for fc in pending_function_calls:
                if self.cancelled:
                    break

                tool_name = fc.name
                tool_args = dict(fc.args) if fc.args else {}
                total_tool_calls += 1

                if on_tool_activity:
                    on_tool_activity("start", tool_name, tool_args)

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

                if on_tool_activity:
                    on_tool_activity("result", tool_name, result)

                console.print(f"  [dim]Tool {tool_name} ({total_tool_calls}/{max_tool_calls})[/]")
                function_response_parts.append(
                    types.Part.from_function_response(name=tool_name, response=result)
                )

            if self.cancelled or not function_response_parts:
                break

            # Send function results back as the next message
            message = function_response_parts

        return full_response

    async def handle_tool_call(self, tool_name: str, arguments: str, user_transcript: str = "") -> str:
        """Execute a skill and return Gemini's summary."""
        params = json.loads(arguments) if arguments else {}

        if tool_name == "get_system_overview":
            return await self._system_overview(user_transcript)

        skill = _skills.get(tool_name)
        if not skill:
            return f"Unknown skill: {tool_name}"

        # Show on Metal HUD
        self._metal_state("skill", skill.name)
        self._metal_hud(f"Fetching {skill.name}...")

        console.print(Panel(
            f"[bold cyan]Skill:[/] {skill.name}\n[dim]Fetching data...[/]",
            title="[bold yellow]⚡ JARVIS SKILL[/]",
            border_style="cyan",
        ))

        # Fetch data
        data = await skill.fetch_data(**params)

        if "error" in data:
            self._metal_hud(f"Error: {data['error']}")
            console.print(f"  [red]Error:[/] {data['error']}")
            self._metal_state("listening")
            return data["error"]

        source = data.get("source", "unknown")
        self._metal_hud(f"Data fetched via {source}")
        console.print(f"  [green]Data fetched[/] via {source}")

        # Gemini summarize
        self._metal_hud("Gemini analyzing...")
        console.print("  [dim]Gemini analyzing...[/]")
        prompt = skill.format_prompt(data, user_transcript or tool_name)

        response = self.gemini.models.generate_content(
            model=config.GEMINI_MODEL,
            contents=prompt,
        )
        summary = response.text

        # Show summary on Metal HUD
        if self.metal:
            self.metal.send_hud_clear()
        for line in summary.split("\n"):
            if line.strip():
                self._metal_hud(line.strip())

        console.print(Panel(
            Markdown(summary),
            title=f"[bold green]{skill.name}[/]",
            border_style="green",
        ))

        self._metal_state("listening")
        return summary

    async def _system_overview(self, user_query: str) -> str:
        """Fetch from all skills and produce a combined summary."""
        self._metal_state("skill", "System Overview")
        console.print(Panel(
            "[bold cyan]Running system overview...[/]",
            title="[bold yellow]⚡ JARVIS OVERVIEW[/]",
            border_style="cyan",
        ))

        all_data = {}
        for name, skill in _skills.items():
            self._metal_hud(f"Fetching {skill.name}...")
            console.print(f"  [dim]Fetching {skill.name}...[/]")
            try:
                data = await skill.fetch_data()
                all_data[skill.name] = data
            except Exception as e:
                all_data[skill.name] = {"error": str(e)}

        prompt = (
            "You are Jarvis, giving a morning briefing across all connected systems.\n"
            f"User asked: \"{user_query}\"\n\n"
            f"Data from all systems:\n{json.dumps(all_data, indent=2, default=str)}\n\n"
            "Give a concise briefing covering each system. Use bullet points. "
            "Skip systems with errors — just note they're offline."
        )

        self._metal_hud("Gemini analyzing all systems...")
        console.print("  [dim]Gemini analyzing all systems...[/]")
        response = self.gemini.models.generate_content(
            model=config.GEMINI_MODEL,
            contents=prompt,
        )
        summary = response.text

        if self.metal:
            self.metal.send_hud_clear()
        for line in summary.split("\n"):
            if line.strip():
                self._metal_hud(line.strip())

        console.print(Panel(
            Markdown(summary),
            title="[bold green]System Overview[/]",
            border_style="green",
        ))

        self._metal_state("listening")
        return summary
