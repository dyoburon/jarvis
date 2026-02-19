import asyncio
import json
import os
import subprocess
import time

from rich.console import Console
from rich.panel import Panel

import config
from skills.router import SkillRouter
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
            stdout=subprocess.DEVNULL,
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

        await asyncio.gather(
            send_audio_loop(),
            client.receive_events(on_audio_delta, on_transcript, on_interrupt),
            watchdog(),
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
