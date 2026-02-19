import asyncio
import json
import os
import subprocess
import time

from rich.console import Console
from rich.panel import Panel

import config
from skills.router import SkillRouter, _skills
from voice.audio import AudioPlayer, MicCapture
from voice.realtime_client import RealtimeClient

console = Console()

MAX_RUNTIME_SECONDS = 300  # 5 minutes hard limit

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
            except BrokenPipeError:
                pass

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

    def send_chat_message(self, speaker: str, text: str):
        self.send({"type": "chat_message", "speaker": speaker, "text": text})

    def send_chat_split(self, title: str):
        self.send({"type": "chat_split", "title": title})

    def send_chat_end(self):
        self.send({"type": "chat_end"})

    def quit(self):
        self.send({"type": "quit"})
        if self.proc:
            try:
                self.proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.proc.kill()


async def main():
    if not config.OPENAI_API_KEY:
        console.print("[red]OPENAI_API_KEY not set in .env[/]")
        return
    if not config.GOOGLE_API_KEY:
        console.print("[red]GOOGLE_API_KEY not set in .env[/]")
        return

    start_time = time.time()
    minutes = MAX_RUNTIME_SECONDS // 60

    # Launch Metal display
    metal = MetalBridge()
    metal.launch()

    console.print(Panel(
        f"[bold cyan]JARVIS[/] — Personal AI Assistant\n"
        f"[dim]Metal display active. Auto-shutdown in {minutes} min.[/]",
        border_style="cyan",
    ))

    router = SkillRouter(metal_bridge=metal)
    mic = MicCapture()
    player = AudioPlayer()
    client = RealtimeClient(router, metal_bridge=metal)

    def on_audio_delta(b64_data: str):
        player.add_audio(b64_data)

    def on_transcript(speaker: str, text: str):
        if speaker == "user":
            console.print(f"\n[bold white]You:[/] {text}")
            metal.send_hud_clear()
            metal.send_hud(f"> {text}")
        else:
            console.print(f"[bold cyan]Jarvis:[/] {text}")
            metal.send_hud(f"JARVIS: {text}")

    skill_task: asyncio.Task | None = None

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

    async def on_skill_input(event_type: str, tool_name: str, arguments: str, user_text: str):
        nonlocal skill_task

        if event_type == "__skill_start__":
            player.clear()

            # Resolve skill display name
            skill = _skills.get(tool_name)
            skill_name = skill.name if skill else "System Overview" if tool_name == "get_system_overview" else tool_name

            metal.send_chat_start(skill_name)
            console.print(f"\n[bold cyan]Chat window opened:[/] {skill_name}")

            def on_chunk(text: str):
                metal.send_chat_message("gemini", text)

            async def run_initial():
                try:
                    await router.start_skill_session(tool_name, arguments, user_text, on_chunk=on_chunk)
                except Exception as e:
                    console.print(f"[red]Skill error:[/] {e}")
                    metal.send_chat_message("gemini", f"\nError: {e}")

            skill_task = asyncio.create_task(run_initial())

        elif event_type == "__skill_chat__":
            if _is_split_command(user_text):
                metal.send_chat_split("Panel 2")
                console.print("[bold cyan]Split window spawned[/]")
                return

            if _is_close_command(user_text):
                console.print("[bold cyan]Closing chat window...[/]")

                if skill_task and not skill_task.done():
                    skill_task.cancel()
                    try:
                        await skill_task
                    except asyncio.CancelledError:
                        pass

                summary = router.close_session()
                metal.send_chat_end()
                await client.exit_skill_mode(summary)
                metal.send_state("listening")
                console.print("[green]Jarvis resumed.[/]")
                return

            # Show user message in chat window
            metal.send_chat_message("user", user_text)
            console.print(f"[white]Chat>[/] {user_text}")

            # Wait for any in-flight response to finish
            if skill_task and not skill_task.done():
                await skill_task

            def on_chunk(text: str):
                metal.send_chat_message("gemini", text)

            async def run_followup():
                try:
                    await router.send_followup(user_text, on_chunk=on_chunk)
                except Exception as e:
                    console.print(f"[red]Followup error:[/] {e}")
                    metal.send_chat_message("gemini", f"\nError: {e}")

            skill_task = asyncio.create_task(run_followup())

    async def watchdog():
        """Monitors timeout and Metal process health."""
        while True:
            await asyncio.sleep(2)
            # Kill Python if Metal window was closed (Escape)
            if metal.proc and metal.proc.poll() is not None:
                console.print("[dim]Metal display closed. Shutting down.[/]")
                os._exit(0)
            # 5-minute hard limit
            elapsed = time.time() - start_time
            remaining = MAX_RUNTIME_SECONDS - elapsed
            if remaining <= 60 and remaining > 50:
                console.print("[yellow]1 minute remaining...[/]")
                metal.send_hud("[ 1 minute remaining ]")
            if remaining <= 0:
                console.print("[bold red]5-minute limit reached. Shutting down.[/]")
                metal.quit()
                os._exit(0)

    try:
        await client.connect()
        mic.start()
        player.start()
        metal.send_state("listening")
        console.print("[green]Listening...[/]\n")

        async def send_audio_loop():
            while True:
                chunk = mic.get_chunk_b64()
                if chunk:
                    await client.send_audio(chunk)
                await asyncio.sleep(config.CHUNK_MS / 1000)

        def on_interrupt():
            player.clear()

        async def read_metal_stdout():
            """Read typed input from Metal WebView via stdout."""
            loop = asyncio.get_event_loop()
            while metal.proc and metal.proc.poll() is None:
                line = await loop.run_in_executor(None, metal.proc.stdout.readline)
                if not line:
                    break
                try:
                    msg = json.loads(line.decode().strip())
                    if msg.get("type") == "chat_input" and client.skill_active:
                        text = msg.get("text", "")
                        if text:
                            await on_skill_input("__skill_chat__", client.pending_tool_name, "", text)
                except (json.JSONDecodeError, UnicodeDecodeError):
                    pass

        await asyncio.gather(
            send_audio_loop(),
            client.receive_events(on_audio_delta, on_transcript, on_interrupt, on_skill_input),
            watchdog(),
            read_metal_stdout(),
        )

    except KeyboardInterrupt:
        pass
    except Exception as e:
        console.print(f"[red]Error:[/] {e}")
    finally:
        console.print("\n[dim]Shutting down...[/]")
        mic.stop()
        player.stop()
        await client.close()
        metal.quit()


if __name__ == "__main__":
    asyncio.run(main())
