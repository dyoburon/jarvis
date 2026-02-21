"""Tool executors for the coding agent. Each function is called by Gemini
via function calling and returns a dict suitable for FunctionResponse."""

import asyncio
from pathlib import Path

PROJECTS_DIR = Path("/Users/dylan/Desktop/projects")
MAX_OUTPUT_CHARS = 12_000
COMMAND_TIMEOUT = 30


def _resolve_path(path: str) -> Path:
    """Resolve path relative to PROJECTS_DIR, jail-checked."""
    p = Path(path).expanduser()
    if not p.is_absolute():
        p = PROJECTS_DIR / p
    resolved = p.resolve()
    if not str(resolved).startswith(str(PROJECTS_DIR.resolve())):
        raise ValueError(f"Path outside allowed directory: {path}")
    return resolved


def _truncate(text: str) -> str:
    if len(text) > MAX_OUTPUT_CHARS:
        return text[:MAX_OUTPUT_CHARS] + f"\n... (truncated, {len(text)} total chars)"
    return text


# --- Tool executors ---


_BLOCKED_PATTERNS = [
    "http.server", "SimpleHTTPServer",  # no spawning servers
    "sudo ", "rm -rf", "rm -r /",       # no destructive/root commands
    "mkfs", "dd if=",                   # no disk operations
    "curl | sh", "curl | bash",         # no pipe-to-shell
    "wget | sh", "wget | bash",
    "> /dev/sd", "shutdown", "reboot",
]


async def run_command(command: str, cwd: str | None = None) -> dict:
    """Execute a shell command."""
    # Block background processes (& at end)
    stripped = command.rstrip()
    if stripped.endswith("&"):
        return {"error": "Background processes (&) are not allowed."}
    # Block dangerous patterns
    cmd_lower = command.lower()
    for pattern in _BLOCKED_PATTERNS:
        if pattern in cmd_lower:
            return {"error": f"Blocked command pattern: {pattern}"}
    work_dir = _resolve_path(cwd) if cwd else PROJECTS_DIR
    proc = await asyncio.create_subprocess_shell(
        command,
        cwd=str(work_dir),
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    try:
        stdout, stderr = await asyncio.wait_for(proc.communicate(), timeout=COMMAND_TIMEOUT)
    except asyncio.TimeoutError:
        proc.kill()
        return {"error": f"Command timed out after {COMMAND_TIMEOUT}s"}

    out = stdout.decode(errors="replace")
    err = stderr.decode(errors="replace")
    result = {"exit_code": proc.returncode, "stdout": _truncate(out)}
    if err.strip():
        result["stderr"] = _truncate(err)
    return result


def read_file(path: str) -> dict:
    """Read a file's contents."""
    resolved = _resolve_path(path)
    if not resolved.exists():
        return {"error": f"File not found: {path}"}
    if not resolved.is_file():
        return {"error": f"Not a file: {path}"}
    content = resolved.read_text(errors="replace")
    return {"path": str(resolved), "content": _truncate(content), "lines": content.count("\n") + 1}


def write_file(path: str, content: str) -> dict:
    """Write content to a file (creates or overwrites)."""
    resolved = _resolve_path(path)
    resolved.parent.mkdir(parents=True, exist_ok=True)
    resolved.write_text(content)
    return {"path": str(resolved), "bytes_written": len(content.encode())}


def edit_file(path: str, old_text: str, new_text: str) -> dict:
    """Replace the first occurrence of old_text with new_text."""
    resolved = _resolve_path(path)
    if not resolved.exists():
        return {"error": f"File not found: {path}"}
    content = resolved.read_text(errors="replace")
    if old_text not in content:
        return {"error": "old_text not found in file"}
    updated = content.replace(old_text, new_text, 1)
    resolved.write_text(updated)
    return {"path": str(resolved), "replacements": 1}


def list_files(path: str = ".", pattern: str = "*") -> dict:
    """List files in a directory with optional glob pattern."""
    resolved = _resolve_path(path)
    if not resolved.is_dir():
        return {"error": f"Not a directory: {path}"}
    files = sorted(
        str(p.relative_to(resolved))
        for p in resolved.glob(pattern)
        if not any(part.startswith(".") for part in p.relative_to(resolved).parts)
    )[:100]
    return {"directory": str(resolved), "files": files, "count": len(files)}


async def search_files(pattern: str, path: str = ".", file_glob: str = "") -> dict:
    """Search file contents using ripgrep."""
    resolved = _resolve_path(path)
    cmd = f"rg --no-heading -n -e {_shell_quote(pattern)}"
    if file_glob:
        cmd += f" -g {_shell_quote(file_glob)}"
    cmd += " ."
    result = await run_command(cmd, cwd=str(resolved))
    return {
        "pattern": pattern,
        "results": result.get("stdout", ""),
        "exit_code": result.get("exit_code", -1),
    }


def _shell_quote(s: str) -> str:
    """Simple shell quoting."""
    return "'" + s.replace("'", "'\\''") + "'"


# Dispatch map: tool_name -> executor function
TOOL_DISPATCH = {
    "run_command": run_command,
    "read_file": read_file,
    "write_file": write_file,
    "edit_file": edit_file,
    "list_files": list_files,
    "search_files": search_files,
}
