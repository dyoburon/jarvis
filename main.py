import asyncio
import glob
import json
import os
import random
import re
import subprocess

import aiohttp
from rich.console import Console
from rich.panel import Panel

import config
from skills.claude_code import _format_tool_start as _cc_format_tool_start, _format_tool_result as _cc_format_tool_result, _TOOL_CATEGORIES as _CC_TOOL_CATEGORIES
from skills.router import SkillRouter, _skills
from voice.audio import MicCapture, SkillMicCapture
from voice.whisper_client import WhisperClient

console = Console()

METAL_APP = "/Users/dylan/Desktop/projects/music-player/jarvis-bootup/.build/debug/JarvisBootup"
BASE_PATH = "/Users/dylan/Desktop/projects/music-player"


class MetalBridge:
    """Sends JSON commands to the Metal app via stdin."""

    def __init__(self):
        self.proc = None

    def launch(self):
        self.proc = subprocess.Popen(
            [METAL_APP, "--jarvis", "--base", BASE_PATH],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )

    def send(self, data: dict):
        if self.proc and self.proc.stdin and self.proc.poll() is None:
            try:
                self.proc.stdin.write((json.dumps(data) + "\n").encode())
                self.proc.stdin.flush()
            except (BrokenPipeError, OSError) as e:
                console.print(f"[dim]Metal bridge write error: {e}[/]")

    def send_audio_level(self, level: float):
        self.send({"type": "audio", "level": level})

    def send_state(self, state: str, name: str = None):
        msg = {"type": "state", "value": state}
        if name:
            msg["name"] = name
        self.send(msg)

    def send_hud(self, text: str):
        self.send({"type": "hud", "text": text})

    def send_hud_clear(self):
        self.send({"type": "hud_clear"})

    def send_chat_start(self, skill_name: str):
        self.send({"type": "chat_start", "skill": skill_name})

    def send_chat_message(self, speaker: str, text: str, panel: int = None):
        msg = {"type": "chat_message", "speaker": speaker, "text": text}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def send_chat_split(self, title: str):
        self.send({"type": "chat_split", "title": title})

    def send_chat_close_panel(self):
        self.send({"type": "chat_close_panel"})

    def send_chat_status(self, text: str, panel: int = None):
        msg = {"type": "chat_status", "text": text}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def send_chat_end(self):
        self.send({"type": "chat_end"})

    def send_chat_overlay(self, text: str):
        self.send({"type": "chat_overlay", "text": text})

    def send_chat_image(self, path: str, panel: int = None):
        msg = {"type": "chat_image", "path": path}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def send_chat_iframe(self, url: str, panel: int = None, height: int = 400):
        msg = {"type": "chat_iframe", "url": url, "height": height}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def send_web_panel(self, url: str, title: str = "Web"):
        self.send({"type": "web_panel", "url": url, "title": title})

    def send_chat_input_text(self, text: str, panel: int = None):
        msg = {"type": "chat_input_set", "text": text}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def quit(self):
        self.send({"type": "quit"})
        if self.proc:
            try:
                self.proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.proc.kill()


async def main():
    if not config.GOOGLE_API_KEY:
        console.print("[red]GOOGLE_API_KEY not set in .env[/]")
        return

    # Launch Metal display
    metal = MetalBridge()
    metal.launch()

    console.print(Panel(
        "[bold cyan]JARVIS[/] — Personal AI Assistant\n"
        "[dim]Metal display active. PTT: Left Control[/]",
        border_style="cyan",
    ))

    router = SkillRouter(metal_bridge=metal)
    mic = MicCapture()
    skill_mic = SkillMicCapture(source_rate=config.SAMPLE_RATE, target_rate=config.WHISPER_SAMPLE_RATE)
    whisper_client = WhisperClient()

    # Start default Gemini Flash conversation session
    router.start_default_session()

    # Local skill state (replaces RealtimeClient state)
    skill_active = False
    pending_tool_name: str | None = None

    skill_tasks: dict[int, asyncio.Task] = {}  # panel_id → running task
    panel_count: int = 0
    active_panel: int = 0

    def _panel_name(idx: int) -> str:
        return f"Bench {idx + 1}"

    _IMAGE_PATH_RE = re.compile(r'(/\S+\.(?:png|jpg|jpeg|gif|webp|bmp|tiff|heic))', re.IGNORECASE)

    def _extract_image_paths(text: str) -> tuple[list[str], str]:
        """Extract image file paths from text. Returns (paths, cleaned_text)."""
        paths = []
        for m in _IMAGE_PATH_RE.finditer(text):
            p = m.group(1)
            if os.path.isfile(p):
                paths.append(p)
        cleaned = _IMAGE_PATH_RE.sub('', text).strip()
        # Collapse multiple spaces
        cleaned = re.sub(r'  +', ' ', cleaned)
        return paths, cleaned

    def _is_close_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        close_phrases = [
            "close window", "close the window", "close chat",
            "close the chat", "exit chat", "exit window",
            "close this", "that's all", "done with this",
            "go back", "never mind", "nevermind",
        ]
        return any(phrase in normalized for phrase in close_phrases)

    def _is_split_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        split_phrases = [
            "new window", "spawn window", "split window",
            "open new window", "spawn new window",
        ]
        return any(phrase in normalized for phrase in split_phrases)

    PINBALL_PATH = os.path.join(os.path.dirname(__file__), "data", "pinball.html")

    def _is_pinball_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        pinball_phrases = [
            "pinball", "play pinball", "launch pinball",
            "open pinball", "start pinball",
        ]
        return any(phrase == normalized or normalized.startswith(phrase) for phrase in pinball_phrases)

    MEMES_DIR = os.path.join(os.path.dirname(__file__), "data", "memes")

    def _is_meme_command(text: str) -> bool:
        normalized = text.lower().strip().rstrip(".")
        meme_phrases = [
            "show me a meme", "meme me", "random meme",
            "show meme", "gimme a meme", "show a meme",
        ]
        return any(phrase in normalized for phrase in meme_phrases)

    def _pick_random_meme() -> str | None:
        """Pick a random meme image from data/memes/. Returns path or None."""
        if not os.path.isdir(MEMES_DIR):
            return None
        files = glob.glob(os.path.join(MEMES_DIR, "*"))
        images = [f for f in files if f.lower().endswith((".png", ".jpg", ".jpeg", ".gif", ".webp"))]
        return random.choice(images) if images else None

    # Tool type categories for UI color-coding
    _TOOL_CATEGORIES = {
        "read_file": "read", "edit_file": "edit", "write_file": "write",
        "list_files": "list", "search_files": "search", "run_command": "run",
        "get_domain_dashboard": "data", "get_paper_dashboard": "data",
        "get_firewall_status": "data", "get_vibetotext_stats": "data",
        "get_system_overview": "data",
    }

    def _short_path(path: str) -> str:
        prefix = "/Users/dylan/Desktop/projects/"
        return path[len(prefix):] if path.startswith(prefix) else path

    def _format_tool_start(tool_name: str, args: dict) -> tuple[str, str]:
        """Return (category, human_description) for a tool call."""
        category = _TOOL_CATEGORIES.get(tool_name, "tool")
        if tool_name == "read_file":
            return category, f"Read {_short_path(args.get('path', ''))}"
        elif tool_name == "edit_file":
            path = _short_path(args.get("path", ""))
            old = args.get("old_text", "")
            old_preview = (old[:50].replace("\n", " ") + "...") if len(old) > 50 else old.replace("\n", " ")
            return category, f"Edit {path}\n  find: {old_preview}"
        elif tool_name == "write_file":
            path = _short_path(args.get("path", ""))
            size = len(args.get("content", ""))
            return category, f"Write {path} ({size} chars)"
        elif tool_name == "list_files":
            path = _short_path(args.get("path", "."))
            pattern = args.get("pattern", "*")
            return category, f"List {path}/{pattern}"
        elif tool_name == "search_files":
            pattern = args.get("pattern", "")
            path = _short_path(args.get("path", "."))
            return category, f"Search /{pattern}/ in {path}"
        elif tool_name == "run_command":
            return category, f"$ {args.get('command', '')}"
        elif tool_name.startswith("get_"):
            nice = tool_name.replace("get_", "").replace("_", " ").title()
            return category, f"Fetch {nice}"
        return category, tool_name

    def _summarize_tool_result(tool_name: str, data: dict) -> str:
        if "error" in data:
            return f"Error: {data['error']}"
        if tool_name == "run_command":
            code = data.get("exit_code", -1)
            out = data.get("stdout", "").strip()
            err = data.get("stderr", "").strip()
            lines = out.split("\n") if out else []
            if len(lines) > 15:
                preview = "\n".join(lines[:8] + ["  ..."] + lines[-4:])
            else:
                preview = out
            result = f"exit {code}"
            if preview:
                result += f"\n{preview}"
            if err and code != 0:
                result += f"\nstderr: {err[:200]}"
            return result
        if tool_name == "read_file":
            line_count = data.get("lines", 0)
            content = data.get("content", "")
            content_lines = content.split("\n")
            if len(content_lines) > 8:
                preview = "\n".join(content_lines[:8]) + f"\n  ... ({line_count} lines total)"
            elif content:
                preview = content[:500]
            else:
                preview = "(empty file)"
            return preview
        if tool_name == "write_file":
            return f"Wrote {data.get('bytes_written', 0)} bytes"
        if tool_name == "edit_file":
            return f"{data.get('replacements', 0)} replacement(s)"
        if tool_name == "list_files":
            files = data.get("files", [])
            if not files:
                return "(empty directory)"
            if len(files) <= 20:
                return "\n".join(files)
            return "\n".join(files[:15] + [f"  ... +{len(files) - 15} more"])
        if tool_name == "search_files":
            results = data.get("results", "")
            lines = results.strip().split("\n") if results.strip() else []
            count = len(lines)
            if count == 0:
                return "No matches"
            if count <= 12:
                return f"{count} matches\n{results.strip()}"
            preview = "\n".join(lines[:10] + [f"  ... +{count - 10} more"])
            return f"{count} matches\n{preview}"
        return str(data)[:300]

    def make_tool_activity_cb(target_panel: int):
        """Create a tool activity callback bound to a specific panel."""
        def on_tool_activity(event: str, tool_name: str, data: dict):
            if event == "start":
                # Claude Code tools use PascalCase (Read, Edit, Bash, etc.)
                if tool_name in _CC_TOOL_CATEGORIES:
                    category, description = _cc_format_tool_start(tool_name, data)
                else:
                    category, description = _format_tool_start(tool_name, data)
                metal.send_chat_message(f"tool_{category}", description, panel=target_panel)
                console.print(f"  [yellow]{description}[/]")
            elif event == "result":
                # Claude Code results have "summary" key
                if "summary" in data:
                    summary = data["summary"]
                else:
                    summary = _summarize_tool_result(tool_name, data)
                metal.send_chat_message("tool_result", summary, panel=target_panel)
            elif event == "approval_request":
                cmd = data.get("command", "")
                metal.send_chat_message("approval", f"`{cmd}`\nPress **Enter** to run or say **no** to deny.", panel=target_panel)
                console.print(f"  [bold yellow]APPROVAL NEEDED:[/] {cmd}")
        return on_tool_activity

    def broadcast_status():
        """Send session status to all open panels."""
        status = router.get_session_status()
        for p in range(panel_count):
            metal.send_chat_status(status, panel=p)

    async def on_skill_input(event_type: str, tool_name: str, arguments: str, user_text: str):
        nonlocal skill_tasks, panel_count, active_panel
        nonlocal skill_active, pending_tool_name

        if event_type == "__skill_start__":
            # If already in a skill session, spawn a new panel with its own session
            if panel_count > 0 and panel_count < 5:
                new_panel = panel_count
                panel_count += 1
                active_panel = new_panel
                metal.send_chat_split(_panel_name(new_panel))
                metal.send({"type": "chat_focus", "panel": new_panel})
                console.print(f"[bold cyan]Window spawned: {_panel_name(new_panel)} ({panel_count}/5)[/]")

                # Start an independent code session for the new panel
                if tool_name == "code_assistant":
                    target_panel = new_panel

                    def on_chunk(text: str, _p=target_panel):
                        metal.send_chat_message("gemini", text, panel=_p)

                    on_tool_activity = make_tool_activity_cb(target_panel)
                    await router.start_code_session_idle(arguments, user_text, panel=target_panel)
                    metal.send_chat_message("gemini", "**Bench 1 ready.** Type or speak your request.", panel=target_panel)
                    console.print(f"  [dim]Code session ready (panel {target_panel})[/]")
                return

            panel_count = 1
            skill_active = True
            pending_tool_name = tool_name

            # Resolve skill display name
            if tool_name == "code_assistant":
                skill_name = "Bench 1"
            elif tool_name == "get_system_overview":
                skill_name = "System Overview"
            else:
                skill = _skills.get(tool_name)
                skill_name = skill.name if skill else tool_name

            metal.send_chat_start(skill_name)
            console.print(f"\n[bold cyan]Chat window opened:[/] {skill_name}")

            target_panel = 0  # First panel is always 0

            def on_chunk(text: str, _p=target_panel):
                metal.send_chat_message("gemini", text, panel=_p)

            on_tool_activity = make_tool_activity_cb(target_panel)

            if tool_name == "code_assistant":
                await router.start_code_session_idle(arguments, user_text, panel=target_panel)
                if user_text and user_text != "__hotkey__":
                    # Voice-triggered with a request — send immediately
                    metal.send_chat_message("user", user_text, panel=target_panel)
                    console.print(f"[white]Chat>[/] {user_text}")

                    async def run_code_initial(_p=target_panel, _chunk=on_chunk, _ta=on_tool_activity, _text=user_text):
                        try:
                            result = await router.send_code_initial(_text, panel=_p, on_chunk=_chunk, on_tool_activity=_ta)
                            if not result or not result.strip():
                                metal.send_chat_message("gemini", "*(Reached turn limit with no text response. Try a more specific request.)*", panel=_p)
                                console.print(f"[yellow]Empty response after tool loop (panel {_p}) — hit iteration limit[/]")
                        except Exception as e:
                            console.print(f"[red]Skill error (panel {_p}):[/] {e}")
                            metal.send_chat_message("gemini", f"\nError: {e}", panel=_p)
                        broadcast_status()

                    skill_tasks[target_panel] = asyncio.create_task(run_code_initial())
                else:
                    # Hotkey or no transcript — just ready for input
                    metal.send_chat_message("gemini", "**Bench 1 ready.** Type or speak your request.", panel=target_panel)
                    console.print(f"  [dim]Code session ready[/]")
            else:
                async def run_initial(_p=target_panel, _chunk=on_chunk, _ta=on_tool_activity):
                    try:
                        await router.start_skill_session(
                            tool_name, arguments, user_text, panel=_p,
                            on_chunk=_chunk, on_tool_activity=_ta,
                        )
                    except Exception as e:
                        console.print(f"[red]Skill error (panel {_p}):[/] {e}")
                        metal.send_chat_message("gemini", f"\nError: {e}", panel=_p)
                    broadcast_status()

                skill_tasks[target_panel] = asyncio.create_task(run_initial())

        elif event_type == "__skill_chat__":
            # Escape key: cancel stream + close focused panel
            if user_text == "__escape__":
                # Cancel the focused panel's task and session
                router.cancel_panel(active_panel)
                task = skill_tasks.pop(active_panel, None)
                if task and not task.done():
                    task.cancel()
                    try:
                        await asyncio.wait_for(task, timeout=1.0)
                    except (asyncio.CancelledError, asyncio.TimeoutError, Exception):
                        pass
                    console.print(f"[yellow]Stream cancelled (panel {active_panel})[/]")

                if panel_count > 1:
                    router.close_panel(active_panel)
                    panel_count -= 1
                    metal.send_chat_close_panel()
                    # Renumber: shift tasks for panels above the closed one
                    new_tasks: dict[int, asyncio.Task] = {}
                    for pid, t in skill_tasks.items():
                        new_tasks[pid - 1 if pid > active_panel else pid] = t
                    skill_tasks = new_tasks
                    if active_panel >= panel_count:
                        active_panel = panel_count - 1
                    metal.send({"type": "chat_focus", "panel": active_panel})
                    console.print(f"[bold cyan]Closed panel ({panel_count} remaining)[/]")
                else:
                    panel_count = 0
                    skill_active = False
                    pending_tool_name = None
                    skill_tasks.clear()
                    router.close_session()
                    metal.send_chat_end()
                    metal.send_state("listening")
                    console.print("[green]Jarvis resumed.[/]")
                return

            if _is_split_command(user_text):
                if panel_count < 5:
                    new_panel = panel_count
                    panel_count += 1
                    active_panel = new_panel
                    metal.send_chat_split(_panel_name(new_panel))
                    metal.send({"type": "chat_focus", "panel": new_panel})
                    console.print(f"[bold cyan]Window spawned: {_panel_name(new_panel)} ({panel_count}/5)[/]")

                    # Auto-create code session for new panel
                    if pending_tool_name == "code_assistant":
                        await router.start_code_session_idle("{}", "", panel=new_panel)
                        metal.send_chat_message("gemini", "**Bench 1 ready.** Type or speak your request.", panel=new_panel)
                        console.print(f"  [dim]Code session ready (panel {new_panel})[/]")
                else:
                    console.print("[yellow]Max 5 windows reached[/]")
                return

            if _is_pinball_command(user_text):
                metal.send_web_panel(f"file://{PINBALL_PATH}", title="Pinball")
                console.print("[bold cyan]Launched Pinball[/]")
                return

            if _is_meme_command(user_text):
                meme_path = _pick_random_meme()
                if meme_path:
                    metal.send_chat_image(meme_path, panel=active_panel)
                    console.print(f"[bold cyan]Meme:[/] {os.path.basename(meme_path)}")
                else:
                    metal.send_chat_message("gemini", "No memes found. Add images to `data/memes/`.", panel=active_panel)
                return

            if _is_close_command(user_text):
                if panel_count > 1:
                    # Cancel and close focused panel
                    router.cancel_panel(active_panel)
                    task = skill_tasks.pop(active_panel, None)
                    if task and not task.done():
                        task.cancel()
                    router.close_panel(active_panel)
                    panel_count -= 1
                    metal.send_chat_close_panel()
                    # Renumber tasks
                    new_tasks = {}
                    for pid, t in skill_tasks.items():
                        new_tasks[pid - 1 if pid > active_panel else pid] = t
                    skill_tasks = new_tasks
                    if active_panel >= panel_count:
                        active_panel = panel_count - 1
                    metal.send({"type": "chat_focus", "panel": active_panel})
                    console.print(f"[bold cyan]Closed panel ({panel_count} remaining)[/]")
                    return

                console.print("[bold cyan]Closing chat window...[/]")

                # Cancel all panel tasks
                for pid, t in skill_tasks.items():
                    if not t.done():
                        t.cancel()
                        try:
                            await t
                        except asyncio.CancelledError:
                            pass

                panel_count = 0
                skill_active = False
                pending_tool_name = None
                skill_tasks.clear()
                router.close_session()
                metal.send_chat_end()
                metal.send_state("listening")
                console.print("[green]Jarvis resumed.[/]")
                return

            # ── Gate: command approval pending on active panel — resolve yes/no ──
            if router.has_pending_approval(active_panel):
                normalized = user_text.lower().strip().rstrip(".")
                approve_phrases = ("yes", "yeah", "yep", "sure", "go", "go ahead", "approve", "run it", "do it", "ok", "okay", "")
                if normalized in approve_phrases or user_text == "\n" or not user_text:
                    cmd = router.get_pending_command(active_panel)
                    router.approve_command(True, panel=active_panel)
                    metal.send_chat_message("tool_result", "Approved", panel=active_panel)
                    console.print(f"  [green]Command approved (panel {active_panel}):[/] {cmd}")
                else:
                    router.approve_command(False, panel=active_panel)
                    metal.send_chat_message("tool_result", "Denied", panel=active_panel)
                    console.print(f"  [red]Command denied (panel {active_panel})[/]")
                return

            if not user_text.strip():
                return

            # Gate: ignore input while THIS panel's response is still streaming
            panel_task = skill_tasks.get(active_panel)
            if panel_task and not panel_task.done():
                console.print(f"[dim]Panel {active_panel} busy — ignoring input: {user_text}[/]")
                metal.send_chat_message("gemini", "*Wait for response to finish...*", panel=active_panel)
                return

            # Show user message in chat window (with image preview if paths detected)
            target_panel = active_panel
            image_paths, display_text = _extract_image_paths(user_text)
            for img_path in image_paths:
                metal.send_chat_image(img_path, panel=target_panel)
            metal.send_chat_message("user", display_text or user_text, panel=target_panel)
            console.print(f"[white]Chat>[/] {user_text} [panel {target_panel}]")

            def on_chunk(text: str, _p=target_panel):
                metal.send_chat_message("gemini", text, panel=_p)

            _on_tool_activity = make_tool_activity_cb(target_panel)

            async def run_followup(_p=target_panel, _chunk=on_chunk, _ta=_on_tool_activity, _text=user_text):
                try:
                    result = await router.send_followup(
                        _text, panel=_p, on_chunk=_chunk, on_tool_activity=_ta,
                    )
                    if not result or not result.strip():
                        metal.send_chat_message("gemini", "*(Reached turn limit with no text response. Try a more specific request.)*", panel=_p)
                        console.print(f"[yellow]Empty response after tool loop (panel {_p}) — hit iteration limit[/]")
                except Exception as e:
                    console.print(f"[red]Followup error (panel {_p}):[/] {e}")
                    metal.send_chat_message("gemini", f"\nError: {e}", panel=_p)
                broadcast_status()

            skill_tasks[target_panel] = asyncio.create_task(run_followup())

    async def chat_monitor():
        """Connects to Great Firewall SSE and shows chat in Metal overlay."""
        chat_buffer: list[str] = []
        max_lines = 8
        url = f"{config.FIREWALL_API}/stream/events"

        while True:
            try:
                async with aiohttp.ClientSession() as session:
                    async with session.get(url, timeout=aiohttp.ClientTimeout(total=None)) as resp:
                        console.print("[dim]Chat monitor connected[/]")
                        async for line in resp.content:
                            text = line.decode("utf-8", errors="replace").strip()
                            if not text.startswith("data:"):
                                continue
                            try:
                                data = json.loads(text[5:].strip())
                                username = data.get("username", "")
                                msg = data.get("text", "")
                                if username and msg:
                                    chat_buffer.append(f"{username}: {msg}")
                                    if len(chat_buffer) > max_lines:
                                        chat_buffer = chat_buffer[-max_lines:]
                                    metal.send_chat_overlay("\n".join(chat_buffer))
                            except (json.JSONDecodeError, KeyError):
                                pass
            except (aiohttp.ClientError, asyncio.TimeoutError, OSError):
                pass
            await asyncio.sleep(5)

    async def watchdog():
        """Monitors Metal process health."""
        while True:
            await asyncio.sleep(2)
            if metal.proc and metal.proc.poll() is not None:
                console.print("[dim]Metal display closed. Shutting down.[/]")
                os._exit(0)

    try:
        mic.start()
        metal.send_state("listening")
        console.print("[green]Listening... (hold Left Control to talk)[/]\n")

        ptt_active = False
        default_task: asyncio.Task | None = None

        async def handle_fn_key(pressed: bool):
            nonlocal ptt_active, skill_active, pending_tool_name, default_task

            if pressed and not ptt_active:
                if not whisper_client.is_available():
                    console.print("[red]vibetotext socket not available — start vibetotext first[/]")
                    if skill_active:
                        metal.send_chat_message("gemini", "Local transcription unavailable. Start vibetotext.")
                    else:
                        metal.send_hud("vibetotext not running")
                    return
                ptt_active = True
                mic._skill_capture = skill_mic
                skill_mic.start_recording()
                metal.send_state("recording")
                console.print("[yellow]PTT recording...[/]")

            elif not pressed and ptt_active:
                ptt_active = False
                audio = skill_mic.stop_recording()
                mic._skill_capture = None

                if skill_active:
                    metal.send_state("chat")
                else:
                    metal.send_state("speaking")

                if len(audio) == 0:
                    if not skill_active:
                        metal.send_state("listening")
                    return

                console.print(f"[dim]PTT: {len(audio)} samples, transcribing...[/]")
                text = await whisper_client.transcribe(audio, sample_rate=config.WHISPER_SAMPLE_RATE)

                if not text:
                    console.print("[dim]PTT: empty transcription[/]")
                    if not skill_active:
                        metal.send_state("listening")
                    return

                console.print(f"[white]PTT>[/] {text}")

                if skill_active:
                    # In skill mode: put transcription in input box for review
                    metal.send_chat_input_text(text, panel=active_panel)
                    metal.send_state("chat")
                else:
                    # Quick commands before hitting Gemini
                    if _is_pinball_command(text):
                        metal.send_web_panel(f"file://{PINBALL_PATH}", title="Pinball")
                        metal.send_state("listening")
                        console.print("[bold cyan]Launched Pinball[/]")
                        return

                    # Default mode: send to Gemini Flash
                    console.print(f"\n[bold white]You:[/] {text}")
                    metal.send_hud_clear()
                    metal.send_hud(f"> {text}")

                    # Wait for any previous default task to finish
                    if default_task and not default_task.done():
                        await default_task

                    async def run_default():
                        nonlocal skill_active, pending_tool_name
                        try:
                            response, trigger = await router.send_default_message(text)

                            if trigger:
                                # Gemini Flash wants to open a skill
                                tool = trigger["tool_name"]
                                args = trigger["arguments"]
                                user = trigger["user_text"]
                                console.print(f"\n[bold yellow]Skill triggered:[/] {tool}")
                                await on_skill_input("__skill_start__", tool, args, user)
                            elif response:
                                console.print(f"[bold cyan]Jarvis:[/] {response}")
                                metal.send_hud(f"JARVIS: {response}")
                                metal.send_state("listening")
                            else:
                                metal.send_state("listening")
                        except Exception as e:
                            console.print(f"[red]Error:[/] {e}")
                            metal.send_hud(f"Error: {e}")
                            metal.send_state("listening")

                    default_task = asyncio.create_task(run_default())

        async def read_metal_stdout():
            """Read typed input and fn key events from Metal WebView via stdout."""
            nonlocal skill_active, pending_tool_name, active_panel
            loop = asyncio.get_event_loop()
            while metal.proc and metal.proc.poll() is None:
                line = await loop.run_in_executor(None, metal.proc.stdout.readline)
                if not line:
                    break
                try:
                    msg = json.loads(line.decode().strip())
                    if msg.get("type") == "panel_focus":
                        active_panel = msg.get("panel", 0)
                    elif msg.get("type") == "chat_input" and skill_active:
                        text = msg.get("text", "")
                        panel_idx = msg.get("panel")
                        if panel_idx is not None:
                            active_panel = panel_idx
                        await on_skill_input("__skill_chat__", pending_tool_name, "", text)
                    elif msg.get("type") == "fn_key":
                        await handle_fn_key(msg.get("pressed", False))
                    elif msg.get("type") == "hotkey":
                        if msg.get("action") == "split" and skill_active:
                            await on_skill_input("__skill_start__", pending_tool_name, "{}", "__hotkey__")
                        elif msg.get("skill") and not skill_active:
                            skill = msg["skill"]
                            console.print(f"\n[bold yellow]Hotkey:[/] {skill}")
                            try:
                                await on_skill_input("__skill_start__", skill, "{}", "__hotkey__")
                            except Exception as e:
                                console.print(f"[red]Hotkey skill error:[/] {e}")
                                import traceback
                                traceback.print_exc()
                                skill_active = False
                                pending_tool_name = None
                except (json.JSONDecodeError, UnicodeDecodeError):
                    pass

        await asyncio.gather(
            watchdog(),
            read_metal_stdout(),
            chat_monitor(),
        )

    except KeyboardInterrupt:
        pass
    except Exception as e:
        console.print(f"[red]Error:[/] {e}")
        import traceback
        traceback.print_exc()
    finally:
        console.print("\n[dim]Shutting down...[/]")
        mic.stop()
        metal.quit()


if __name__ == "__main__":
    asyncio.run(main())
