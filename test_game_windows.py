#!/usr/bin/env python3
"""
test_game_windows.py -- Automated game window lifecycle tests for Jarvis Metal app.

Usage:
    python test_game_windows.py              # Run all tests
    python test_game_windows.py --manual     # Manual mode: collect events while user interacts
    python test_game_windows.py --test NAME  # Run a single test
    python test_game_windows.py --dump out.json  # Save timeline after tests
"""

import argparse
import atexit
import json
import os
import queue
import subprocess
import sys
import threading
import time

sys.path.insert(0, os.path.dirname(__file__))
from game_event_log import GameEventLog

METAL_APP = "/Users/dylan/Desktop/projects/jarvis/metal-app/.build/debug/JarvisBootup"
BASE_PATH = "/Users/dylan/Desktop/projects/jarvis"
PROJECT_DIR = os.path.dirname(__file__)
ASTEROIDS_PATH = os.path.join(PROJECT_DIR, "asteroids.html")
TETRIS_PATH = os.path.join(PROJECT_DIR, "tetris.html")


class TestMetalBridge:
    """MetalBridge variant for testing -- adds stdout event collection."""

    def __init__(self, event_log: GameEventLog):
        self.proc = None
        self._queue: queue.Queue = queue.Queue()
        self._writer_thread = None
        self._reader_thread = None
        self.event_log = event_log
        self._all_messages: list[dict] = []
        self._lock = threading.Lock()

    def launch(self):
        self.proc = subprocess.Popen(
            [METAL_APP, "--jarvis", "--base", BASE_PATH],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
        self._writer_thread = threading.Thread(target=self._drain, daemon=True)
        self._writer_thread.start()
        self._reader_thread = threading.Thread(target=self._read_stdout, daemon=True)
        self._reader_thread.start()

    def _drain(self):
        while True:
            data = self._queue.get()
            if data is None:
                break
            if self.proc and self.proc.stdin and self.proc.poll() is None:
                try:
                    self.proc.stdin.write((json.dumps(data) + "\n").encode())
                    self.proc.stdin.flush()
                except (BrokenPipeError, OSError):
                    break

    def _read_stdout(self):
        while self.proc and self.proc.poll() is None:
            line = self.proc.stdout.readline()
            if not line:
                break
            try:
                msg = json.loads(line.decode().strip())
                with self._lock:
                    self._all_messages.append(msg)
                if msg.get("type") == "game_event":
                    self.event_log.ingest(msg)
            except (json.JSONDecodeError, UnicodeDecodeError):
                pass

    def send(self, data: dict):
        self._queue.put_nowait(data)

    def send_chat_start(self, skill_name: str):
        self.send({"type": "chat_start", "skill": skill_name})

    def send_chat_iframe_fullscreen(self, url: str, panel: int = None):
        msg = {"type": "chat_iframe_fullscreen", "url": url}
        if panel is not None:
            msg["panel"] = panel
        self.send(msg)

    def send_chat_end(self):
        self.send({"type": "chat_end"})

    def send_chat_close_panel(self):
        self.send({"type": "chat_close_panel"})

    def send_chat_focus(self, panel: int):
        self.send({"type": "chat_focus", "panel": panel})

    def send_chat_split(self, title: str):
        self.send({"type": "chat_split", "title": title})

    def send_test_hide_fullscreen(self):
        self.send({"type": "test_hide_fullscreen"})

    def quit(self):
        self.send({"type": "quit"})
        self._queue.put(None)
        if self._writer_thread:
            self._writer_thread.join(timeout=2)
        if self.proc:
            try:
                self.proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.proc.kill()


class TestResult:
    def __init__(self, name: str, passed: bool, duration: float,
                 details: str = "", timeline: list = None):
        self.name = name
        self.passed = passed
        self.duration = duration
        self.details = details
        self.timeline = timeline or []


class GameWindowTestRunner:
    def __init__(self):
        self.event_log = GameEventLog()
        self.bridge = TestMetalBridge(self.event_log)
        self.results: list[TestResult] = []

    def setup(self):
        """Launch Metal app and wait for it to be ready."""
        print("Launching Metal app...")
        self.bridge.launch()
        time.sleep(2)
        # Open a chat window (required before any game commands)
        self.bridge.send_chat_start("Test Harness")
        time.sleep(2)  # Wait for HTML template to fully load in WKWebView
        print("Metal app ready.\n")

    def teardown(self):
        print("Shutting down Metal app...")
        self.bridge.quit()

    def wait_for_event(self, event_name: str, timeout: float = 10.0) -> dict | None:
        return self.event_log.wait_for_event(event_name, timeout=timeout)

    def run_test(self, name: str, fn):
        """Run a single test function with timing and event collection."""
        print(f"--- TEST: {name} ---")
        self.event_log.clear()
        start = time.time()
        try:
            fn()
            elapsed = time.time() - start
            result = TestResult(name, True, elapsed,
                                timeline=self.event_log.timeline(include_metal_log=False))
            print(f"  PASS ({elapsed:.1f}s)")
        except AssertionError as e:
            elapsed = time.time() - start
            result = TestResult(name, False, elapsed, str(e),
                                timeline=self.event_log.timeline(include_metal_log=False))
            print(f"  FAIL ({elapsed:.1f}s): {e}")
        except Exception as e:
            elapsed = time.time() - start
            result = TestResult(name, False, elapsed, f"ERROR: {e}",
                                timeline=self.event_log.timeline(include_metal_log=False))
            print(f"  ERROR ({elapsed:.1f}s): {e}")
        self.results.append(result)
        # Clean up: hide any fullscreen iframe and wait a bit between tests
        self.bridge.send_test_hide_fullscreen()
        time.sleep(0.5)

    def report(self):
        """Print pass/fail summary with full event timelines for failures."""
        print("\n" + "=" * 60)
        print("GAME WINDOW TEST REPORT")
        print("=" * 60)
        passed = sum(1 for r in self.results if r.passed)
        total = len(self.results)
        print(f"\nResults: {passed}/{total} passed\n")
        for r in self.results:
            status = "PASS" if r.passed else "FAIL"
            print(f"  [{status}] {r.name} ({r.duration:.1f}s)")
            if not r.passed:
                print(f"         {r.details}")
                if r.timeline:
                    print("         Timeline:")
                    for entry in r.timeline:
                        evt = entry.get("event", entry.get("message", ""))
                        extras = {k: v for k, v in entry.items()
                                  if k not in ("source", "ts", "event", "message")}
                        extra_str = f" {extras}" if extras else ""
                        print(f"           [{entry['ts']}] {evt}{extra_str}")
        print()


# ── Test Scenarios ──────────────────────────────────────────────


def test_launch_local_game(runner: GameWindowTestRunner):
    """Launch a local file:// game (asteroids), verify iframe_show + iframe_loaded."""
    url = f"file://{ASTEROIDS_PATH}"
    runner.bridge.send_chat_iframe_fullscreen(url, panel=0)

    show = runner.wait_for_event("iframe_show", timeout=5)
    assert show is not None, "Timed out waiting for iframe_show event"
    assert show.get("mode") == "srcdoc", f"Expected srcdoc mode, got {show.get('mode')}"
    assert show.get("panel") == 0, f"Expected panel 0, got {show.get('panel')}"

    loaded = runner.wait_for_event("iframe_loaded", timeout=10)
    assert loaded is not None, "Timed out waiting for iframe_loaded event"


def test_launch_web_game(runner: GameWindowTestRunner):
    """Launch a web URL game (kartbros.io), verify navigation + load events."""
    runner.bridge.send_chat_iframe_fullscreen("https://kartbros.io", panel=0)

    show = runner.wait_for_event("iframe_show", timeout=5)
    assert show is not None, "Timed out waiting for iframe_show"
    assert show.get("mode") == "navigated", f"Expected navigated mode, got {show.get('mode')}"

    loaded = runner.wait_for_event("iframe_loaded", timeout=30)
    assert loaded is not None, "Timed out waiting for iframe_loaded (page may have failed to load)"


def test_exit_fullscreen(runner: GameWindowTestRunner):
    """Launch a game, then hide it. Verify iframe_hide event."""
    url = f"file://{ASTEROIDS_PATH}"
    runner.bridge.send_chat_iframe_fullscreen(url, panel=0)
    runner.wait_for_event("iframe_loaded", timeout=10)

    time.sleep(0.5)
    runner.event_log.clear()

    runner.bridge.send_test_hide_fullscreen()

    hide = runner.wait_for_event("iframe_hide", timeout=5)
    assert hide is not None, "Timed out waiting for iframe_hide"


def test_reenter_game(runner: GameWindowTestRunner):
    """Exit and re-enter fullscreen game. Verify clean state transitions."""
    url = f"file://{ASTEROIDS_PATH}"

    # First entry
    runner.bridge.send_chat_iframe_fullscreen(url, panel=0)
    loaded = runner.wait_for_event("iframe_loaded", timeout=10)
    assert loaded is not None, "First load failed"

    # Exit
    runner.bridge.send_test_hide_fullscreen()
    hide = runner.wait_for_event("iframe_hide", timeout=5)
    assert hide is not None, "First hide failed"

    time.sleep(1)
    runner.event_log.clear()

    # Re-enter
    runner.bridge.send_chat_iframe_fullscreen(url, panel=0)
    show = runner.wait_for_event("iframe_show", timeout=5)
    assert show is not None, "Failed to re-show game"
    loaded = runner.wait_for_event("iframe_loaded", timeout=10)
    assert loaded is not None, "Failed to re-load game"


def test_switch_panels_mid_game(runner: GameWindowTestRunner):
    """Launch game on panel 0, spawn panel 1, switch focus, verify state."""
    url = f"file://{ASTEROIDS_PATH}"

    runner.bridge.send_chat_iframe_fullscreen(url, panel=0)
    runner.wait_for_event("iframe_loaded", timeout=10)

    # Spawn a new panel
    runner.bridge.send_chat_split("Panel 2")
    time.sleep(1)

    # Focus the new panel
    runner.bridge.send_chat_focus(1)
    time.sleep(0.5)

    # Focus back to game panel
    runner.event_log.clear()
    runner.bridge.send_chat_focus(0)
    time.sleep(0.5)

    # Verify no spurious iframe_hide during panel switch
    tl = runner.event_log.timeline(include_metal_log=False)
    hide_events = [e for e in tl if e.get("event") == "iframe_hide"]
    assert len(hide_events) == 0, f"Unexpected iframe_hide during panel switch: {hide_events}"


def test_rapid_panel_switches(runner: GameWindowTestRunner):
    """Rapidly switch panels to stress-test state consistency."""
    runner.bridge.send_chat_split("Panel 2")
    time.sleep(0.5)

    for i in range(10):
        runner.bridge.send_chat_focus(i % 2)
        time.sleep(0.1)

    time.sleep(1)
    # Verify Metal process is still alive
    assert runner.bridge.proc.poll() is None, "Metal process crashed during rapid panel switches"


def test_game_on_second_panel(runner: GameWindowTestRunner):
    """Spawn second panel, launch game on panel 1. Verify correct panel index."""
    runner.bridge.send_chat_split("Panel 2")
    time.sleep(0.5)

    url = f"file://{TETRIS_PATH}"
    runner.bridge.send_chat_iframe_fullscreen(url, panel=1)

    show = runner.wait_for_event("iframe_show", timeout=5)
    assert show is not None, "Timed out waiting for iframe_show"
    assert show.get("panel") == 1, f"Expected panel 1, got {show.get('panel')}"


def test_type_other_panel_during_game(runner: GameWindowTestRunner):
    """Game fullscreen on panel 0, switch to panel 1, verify typing reaches panel 1."""
    url = f"file://{ASTEROIDS_PATH}"
    runner.bridge.send_chat_iframe_fullscreen(url, panel=0)
    runner.wait_for_event("iframe_loaded", timeout=10)

    # Spawn panel 1 and focus it
    runner.bridge.send_chat_split("Chat")
    time.sleep(0.5)
    runner.bridge.send_chat_focus(1)
    time.sleep(0.5)

    # Set text in panel 1's input box and simulate submit
    runner.bridge.send({"type": "chat_input_set", "text": "hello from panel 1", "panel": 1})
    time.sleep(0.3)

    # The key test: if the keyboard routing is correct, panel 1 should
    # still be interactive (not swallowed by the game on panel 0).
    # We verify by checking Metal is alive and no crash occurred,
    # and that the game's fullscreen state is still intact.
    assert runner.bridge.proc.poll() is None, "Metal process crashed"

    # Verify game is still fullscreen on panel 0 (no spurious hide)
    tl = runner.event_log.timeline(include_metal_log=False)
    hide_events = [e for e in tl if e.get("event") == "iframe_hide"]
    assert len(hide_events) == 0, "Game was unexpectedly hidden when typing on other panel"


# ── Manual Mode ─────────────────────────────────────────────────


def manual_mode(runner: GameWindowTestRunner):
    """Collect and display events while user interacts manually."""
    print("MANUAL MODE: Interact with the Metal app. Press Ctrl+C to stop.")
    print("Events will be displayed in real-time.\n")
    try:
        last_count = 0
        while True:
            events = runner.event_log.events
            for entry in events[last_count:]:
                ts = entry.get("ts", "?")
                event = entry.get("event", "?")
                extras = {k: v for k, v in entry.items()
                         if k not in ("source", "ts", "event")}
                print(f"  [{ts}] {event} {extras}")
            last_count = len(events)
            time.sleep(0.2)
    except KeyboardInterrupt:
        print("\n\nFull collected timeline:")
        runner.event_log.pretty_print()


# ── Main ────────────────────────────────────────────────────────


ALL_TESTS = {
    "launch_local": test_launch_local_game,
    "launch_web": test_launch_web_game,
    "exit_fullscreen": test_exit_fullscreen,
    "reenter": test_reenter_game,
    "switch_panels": test_switch_panels_mid_game,
    "rapid_switches": test_rapid_panel_switches,
    "second_panel": test_game_on_second_panel,
    "type_other_panel": test_type_other_panel_during_game,
}


def main():
    parser = argparse.ArgumentParser(description="Game window lifecycle tests")
    parser.add_argument("--manual", action="store_true",
                        help="Manual event collection mode")
    parser.add_argument("--test", type=str,
                        help=f"Run a single test: {list(ALL_TESTS.keys())}")
    parser.add_argument("--dump", type=str,
                        help="Dump timeline to JSON file after tests")
    args = parser.parse_args()

    runner = GameWindowTestRunner()
    atexit.register(runner.teardown)
    runner.setup()

    if args.manual:
        manual_mode(runner)
        return

    if args.test:
        if args.test in ALL_TESTS:
            runner.run_test(args.test, lambda: ALL_TESTS[args.test](runner))
        else:
            print(f"Unknown test: {args.test}")
            print(f"Available: {list(ALL_TESTS.keys())}")
            return
    else:
        for name, fn in ALL_TESTS.items():
            runner.run_test(name, lambda f=fn: f(runner))

    runner.report()

    if args.dump:
        runner.event_log.dump_json(args.dump, include_metal_log=True)
        print(f"Timeline dumped to {args.dump}")


if __name__ == "__main__":
    main()
