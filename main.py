import asyncio
import json
import os
import subprocess
import time

from rich.console import Console
from rich.panel import Panel

import config
from skills.router import SkillRouter, _skills
from voice.audio import AudioPlayer, MicCapture, SkillMicCapture
from voice.realtime_client import RealtimeClient
from voice.whisper_client import WhisperClient

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

    def send_chat_message(self, speaker: str, text: str):
        self.send({"type": "chat_message", "speaker": speaker, "text": text})

    def send_chat_split(self, title: str):
        self.send({"type": "chat_split", "title": title})

    def send_chat_close_panel(self):
        self.send({"type": "chat_close_panel"})

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
    skill_mic = SkillMicCapture(source_rate=config.SAMPLE_RATE, target_rate=config.WHISPER_SAMPLE_RATE)
    whisper_client = WhisperClient()

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
    panel_count: int = 0
    active_panel: int = 0  # 0-indexed, which panel gets input
    pending_code_message: str | None = None  # held until user confirms
    code_confirmation_pending: bool = False  # gate flag (independent of message content)

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

    def _summarize_tool_args(tool_name: str, args: dict) -> str:
        if tool_name == "run_command":
            return args.get("command", "")
        elif tool_name == "read_file":
            return args.get("path", "")
        elif tool_name == "write_file":
            return f"{args.get('path', '')} ({len(args.get('content', ''))} chars)"
        elif tool_name == "edit_file":
            return args.get("path", "")
        elif tool_name == "list_files":
            return f"{args.get('path', '.')} ({args.get('pattern', '*')})"
        elif tool_name == "search_files":
            return f"/{args.get('pattern', '')}/ in {args.get('path', '.')}"
        return str(args)[:100]

    def _summarize_tool_result(tool_name: str, data: dict) -> str:
        if "error" in data:
            return f"Error: {data['error']}"
        if tool_name == "run_command":
            code = data.get("exit_code", -1)
            out = data.get("stdout", "").strip()
            lines = out.split("\n") if out else []
            if len(lines) > 8:
                preview = "\n".join(lines[:4] + ["..."] + lines[-3:])
            else:
                preview = out
            return f"exit {code}\n{preview}" if preview else f"exit {code}"
        if tool_name == "read_file":
            return f"{data.get('lines', 0)} lines"
        if tool_name == "write_file":
            return f"wrote {data.get('bytes_written', 0)} bytes"
        if tool_name == "edit_file":
            return f"replaced {data.get('replacements', 0)} occurrence(s)"
        if tool_name == "list_files":
            return f"{data.get('count', 0)} files"
        if tool_name == "search_files":
            results = data.get("results", "")
            count = len(results.strip().split("\n")) if results.strip() else 0
            return f"{count} matches"
        return str(data)[:200]

    def on_tool_activity(event: str, tool_name: str, data: dict):
        """Display tool execution activity in the chat window."""
        if event == "start":
            summary = _summarize_tool_args(tool_name, data)
            metal.send_chat_message("tool_start", f"{tool_name}: {summary}")
            console.print(f"  [yellow]Tool>[/] {tool_name}: {summary}")
        elif event == "result":
            summary = _summarize_tool_result(tool_name, data)
            metal.send_chat_message("tool_result", summary)

    async def on_skill_input(event_type: str, tool_name: str, arguments: str, user_text: str):
        nonlocal skill_task, panel_count, active_panel, pending_code_message, code_confirmation_pending

        if event_type == "__skill_start__":
            # If already in a skill session, treat as a split instead of new session
            if panel_count > 0 and panel_count < 6:
                panel_count += 1
                metal.send_chat_split(f"Panel {panel_count}")
                console.print(f"[bold cyan]Window spawned ({panel_count}/6)[/]")
                return

            player.clear()
            panel_count = 1

            # Disconnect OpenAI to stop billing during skill mode
            await client.disconnect()

            # Resolve skill display name
            if tool_name == "code_assistant":
                skill_name = "Code Assistant"
            elif tool_name == "get_system_overview":
                skill_name = "System Overview"
            else:
                skill = _skills.get(tool_name)
                skill_name = skill.name if skill else tool_name

            metal.send_chat_start(skill_name)
            console.print(f"\n[bold cyan]Chat window opened:[/] {skill_name}")

            def on_chunk(text: str):
                metal.send_chat_message("gemini", text)

            # Don't auto-send to Gemini — show the transcript and wait for confirmation
            if tool_name == "code_assistant":
                await router.start_code_session_idle(arguments, user_text)
                if user_text == "__hotkey__":
                    # Hotkey-triggered: skip confirmation gate, ready for direct input
                    code_confirmation_pending = False
                    pending_code_message = None
                    metal.send_chat_message("gemini", "**Code Assistant ready.** Type or speak your request.")
                    console.print(f"  [dim]Code session ready (hotkey)[/]")
                else:
                    code_confirmation_pending = True
                    # user_text may be empty if transcript hasn't arrived yet (race condition)
                    pending_code_message = user_text if user_text else None
                    if pending_code_message:
                        metal.send_chat_message("gemini", f"**Ready.** Your request:\n\n> {user_text}\n\nSay **yes** to send.")
                    console.print(f"  [dim]Code session ready, awaiting confirmation[/]")
            else:
                async def run_initial():
                    try:
                        await router.start_skill_session(
                            tool_name, arguments, user_text,
                            on_chunk=on_chunk, on_tool_activity=on_tool_activity,
                        )
                    except Exception as e:
                        console.print(f"[red]Skill error:[/] {e}")
                        metal.send_chat_message("gemini", f"\nError: {e}")

                skill_task = asyncio.create_task(run_initial())

        elif event_type == "__skill_chat__":
            # Escape key: cancel stream + close focused panel
            if user_text == "__escape__":
                # Kill the Gemini session immediately (breaks any in-flight stream)
                router.cancelled = True
                router.active_chat = None
                # Cancel the asyncio task
                if skill_task and not skill_task.done():
                    skill_task.cancel()
                    try:
                        await asyncio.wait_for(skill_task, timeout=1.0)
                    except (asyncio.CancelledError, asyncio.TimeoutError, Exception):
                        pass
                    skill_task = None
                    console.print("[yellow]Stream cancelled[/]")

                # Close focused panel (or end session if last)
                if panel_count > 1:
                    panel_count -= 1
                    metal.send_chat_close_panel()
                    if active_panel >= panel_count:
                        active_panel = panel_count - 1
                    metal.send({"type": "chat_focus", "panel": active_panel})
                    console.print(f"[bold cyan]Closed panel ({panel_count} remaining)[/]")
                else:
                    panel_count = 0
                    code_confirmation_pending = False
                    pending_code_message = None
                    router.close_session()
                    metal.send_chat_end()
                    client.reset_skill_state()
                    await client.reconnect()
                    metal.send_state("listening")
                    console.print("[green]Jarvis resumed (fresh session).[/]")
                return

            if _is_split_command(user_text):
                if panel_count < 6:
                    panel_count += 1
                    active_panel = panel_count - 1
                    metal.send_chat_split(f"Panel {panel_count}")
                    metal.send({"type": "chat_focus", "panel": active_panel})
                    console.print(f"[bold cyan]Window spawned ({panel_count}/6), focus → {panel_count}[/]")
                else:
                    console.print("[yellow]Max 6 windows reached[/]")
                return

            if _is_close_command(user_text):
                # Multiple panels: close the last one only
                if panel_count > 1:
                    panel_count -= 1
                    metal.send_chat_close_panel()
                    if active_panel >= panel_count:
                        active_panel = panel_count - 1
                    metal.send({"type": "chat_focus", "panel": active_panel})
                    console.print(f"[bold cyan]Closed panel ({panel_count} remaining)[/]")
                    return

                # Last panel: close the whole session
                console.print("[bold cyan]Closing chat window...[/]")

                if skill_task and not skill_task.done():
                    skill_task.cancel()
                    try:
                        await skill_task
                    except asyncio.CancelledError:
                        pass

                panel_count = 0
                code_confirmation_pending = False
                pending_code_message = None
                router.close_session()
                metal.send_chat_end()
                client.reset_skill_state()
                await client.reconnect()
                metal.send_state("listening")
                console.print("[green]Jarvis resumed (fresh session).[/]")
                return

            # ── Gate: code confirmation pending — NOTHING passes through to Gemini ──
            if code_confirmation_pending:
                normalized = user_text.lower().strip().rstrip(".")

                # "Yes" variants → send the pending (or current) message
                if normalized in ("yes", "yeah", "yep", "sure", "go", "go ahead", "send it", "do it"):
                    msg_to_send = pending_code_message or user_text
                    code_confirmation_pending = False
                    pending_code_message = None
                    metal.send_chat_message("user", msg_to_send)
                    console.print(f"[white]Chat>[/] {msg_to_send}")

                    def on_chunk(text: str):
                        metal.send_chat_message("gemini", text)

                    async def run_confirmed():
                        try:
                            await router.send_code_initial(msg_to_send, on_chunk=on_chunk, on_tool_activity=on_tool_activity)
                        except Exception as e:
                            console.print(f"[red]Skill error:[/] {e}")
                            metal.send_chat_message("gemini", f"\nError: {e}")

                    skill_task = asyncio.create_task(run_confirmed())
                    return

                # First transcript arriving (pending was empty from race condition)
                if not pending_code_message:
                    pending_code_message = user_text
                    metal.send_chat_message("gemini", f"**Ready.** Your request:\n\n> {user_text}\n\nSay **yes** to send.")
                    console.print(f"  [dim]Code session ready: {user_text}[/]")
                    return

                # Echo of the same transcript — ignore silently
                if user_text.strip().lower() == pending_code_message.strip().lower():
                    return

                # Different text — update pending request
                pending_code_message = user_text
                metal.send_chat_message("gemini", f"**Updated request:**\n\n> {user_text}\n\nSay **yes** to send.")
                console.print(f"  [dim]Pending request updated: {user_text}[/]")
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
                    await router.send_followup(
                        user_text, on_chunk=on_chunk, on_tool_activity=on_tool_activity,
                    )
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
                if chunk and not client._disconnected and not client.skill_active:
                    await client.send_audio(chunk)
                await asyncio.sleep(config.CHUNK_MS / 1000)

        def on_interrupt():
            player.clear()

        ptt_active = False

        async def handle_fn_key(pressed: bool):
            nonlocal ptt_active

            if not client.skill_active:
                return  # fn key only matters during skill mode

            if pressed and not ptt_active:
                if not whisper_client.is_available():
                    console.print("[red]vibetotext socket not available — start vibetotext first[/]")
                    metal.send_chat_message("gemini", "Local transcription unavailable. Start vibetotext.")
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
                metal.send_state("chat")

                if len(audio) == 0:
                    return

                console.print(f"[dim]PTT: {len(audio)} samples, transcribing...[/]")
                text = await whisper_client.transcribe(audio, sample_rate=config.WHISPER_SAMPLE_RATE)

                if not text:
                    console.print("[dim]PTT: empty transcription[/]")
                    return

                console.print(f"[white]PTT>[/] {text}")
                await on_skill_input("__skill_chat__", client.pending_tool_name, "", text)

        async def read_metal_stdout():
            """Read typed input and fn key events from Metal WebView via stdout."""
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
                    elif msg.get("type") == "fn_key":
                        await handle_fn_key(msg.get("pressed", False))
                    elif msg.get("type") == "hotkey":
                        if msg.get("action") == "split" and client.skill_active:
                            # Cmd+T — spawn new panel
                            await on_skill_input("__skill_start__", client.pending_tool_name, "{}", "__hotkey__")
                        elif msg.get("skill") and not client.skill_active:
                            skill = msg["skill"]
                            console.print(f"\n[bold yellow]Hotkey:[/] {skill}")
                            client.skill_active = True
                            client.pending_tool_name = skill
                            try:
                                await on_skill_input("__skill_start__", skill, "{}", "__hotkey__")
                            except Exception as e:
                                console.print(f"[red]Hotkey skill error:[/] {e}")
                                import traceback
                                traceback.print_exc()
                                client.skill_active = False
                                client.pending_tool_name = None
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
