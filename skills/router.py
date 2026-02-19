import json

from google import genai
from rich.console import Console
from rich.live import Live
from rich.markdown import Markdown
from rich.panel import Panel

import config
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
]

# Skill instances
_skills = {
    "get_domain_dashboard": DomainSkill(),
    "get_paper_dashboard": PaperSkill(),
    "get_firewall_status": FirewallSkill(),
    "get_vibetotext_stats": VibeToTextSkill(),
}


class SkillRouter:
    def __init__(self, metal_bridge=None):
        self.gemini = genai.Client(api_key=config.GOOGLE_API_KEY)
        self.metal = metal_bridge

    def _metal_hud(self, text: str):
        if self.metal:
            self.metal.send_hud(text)

    def _metal_state(self, state: str, name: str = None):
        if self.metal:
            self.metal.send_state(state, name)

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
