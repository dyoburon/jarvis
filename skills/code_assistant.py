"""Gemini function-calling tool declarations and system prompt for the unified Jarvis agent."""

from google.genai import types

CODE_TOOLS = [
    types.Tool(function_declarations=[
        # ── Code tools ──
        types.FunctionDeclaration(
            name="run_command",
            description="Execute a shell command. Use for git, npm, python, build tools, etc.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "The shell command to execute"},
                    "cwd": {"type": "string", "description": "Working directory relative to ~/Desktop/projects (optional)"},
                },
                "required": ["command"],
            },
        ),
        types.FunctionDeclaration(
            name="read_file",
            description="Read a file's contents. Path relative to ~/Desktop/projects or absolute.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path to read"},
                },
                "required": ["path"],
            },
        ),
        types.FunctionDeclaration(
            name="write_file",
            description="Create or overwrite a file with given content.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path to write"},
                    "content": {"type": "string", "description": "Full file content"},
                },
                "required": ["path", "content"],
            },
        ),
        types.FunctionDeclaration(
            name="edit_file",
            description="Replace the first occurrence of old_text with new_text in a file. Read the file first to get exact text.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path to edit"},
                    "old_text": {"type": "string", "description": "Exact text to find (must match precisely)"},
                    "new_text": {"type": "string", "description": "Replacement text"},
                },
                "required": ["path", "old_text", "new_text"],
            },
        ),
        types.FunctionDeclaration(
            name="list_files",
            description="List files in a directory with optional glob pattern.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Directory path (default: projects root)"},
                    "pattern": {"type": "string", "description": "Glob pattern like '*.py' or '**/*.ts' (default: *)"},
                },
            },
        ),
        types.FunctionDeclaration(
            name="search_files",
            description="Search file contents using regex pattern (ripgrep). Returns matching lines with filenames and line numbers.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Regex search pattern"},
                    "path": {"type": "string", "description": "Directory to search (default: projects root)"},
                    "file_glob": {"type": "string", "description": "File glob filter like '*.py' (optional)"},
                },
                "required": ["pattern"],
            },
        ),
        # ── Data tools (query connected systems directly) ──
        types.FunctionDeclaration(
            name="get_vibetotext_stats",
            description="Get voice transcription statistics: total words dictated, session count, WPM, sentiment, and recent transcription entries.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "description": "Number of recent entries to return (default: 10)"},
                },
            },
        ),
        types.FunctionDeclaration(
            name="get_domain_dashboard",
            description="Get today's domain drop hunting results: matched domains, zone stats, disappeared domains.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "date": {"type": "string", "description": "Date (YYYY-MM-DD), defaults to today"},
                    "min_score": {"type": "number", "description": "Minimum match score filter"},
                },
            },
        ),
        types.FunctionDeclaration(
            name="get_paper_dashboard",
            description="Get today's research paper matches from arXiv, grouped by interest area.",
            parameters_json_schema={
                "type": "object",
                "properties": {
                    "date": {"type": "string", "description": "Date (YYYY-MM-DD), defaults to today"},
                    "interest": {"type": "string", "description": "Filter to specific research interest"},
                },
            },
        ),
        types.FunctionDeclaration(
            name="get_firewall_status",
            description="Get chat moderation stats from the Great Firewall: recent messages, blocked count.",
            parameters_json_schema={
                "type": "object",
                "properties": {},
            },
        ),
        types.FunctionDeclaration(
            name="get_system_overview",
            description="Get a full overview across ALL connected systems (vibetotext, domains, papers, firewall).",
            parameters_json_schema={
                "type": "object",
                "properties": {},
            },
        ),
    ]),
    # Google Search grounding — lets Gemini search the web for docs, APIs, etc.
    types.Tool(google_search=types.GoogleSearch()),
]

CODE_SYSTEM_PROMPT = """\
You are Jarvis, Dylan's personal AI coding assistant. You run on Dylan's Mac \
from ~/Desktop/projects/jarvis/.

The user interacts with you through a chat window. Every tool call you make is \
shown in real time as a color-coded activity feed — the user sees what you read, \
edit, search, and run. Be purposeful with each action.

IMPORTANT: Before making tool calls, ALWAYS emit a short text explanation of \
what you're about to do and why. The user can see your tool calls but not your \
reasoning — narrate your thought process so they can follow along. Examples:
- "Let me read the config to see how routes are set up." → read_file
- "I'll search for where this function is called." → search_files
- "Updating the handler to fix the off-by-one error." → edit_file
Keep narration to 1 line. Do NOT narrate trivially obvious actions in sequence \
(e.g. don't say "now I'll read file X" if you just said "let me check files X and Y").

# Tone and style

Be concise and direct. Answer in 1-4 lines unless the user asks for detail. \
Do not add preamble ("Sure, I can help with that...") or postamble \
("Let me know if you need anything else"). One-word answers are fine when appropriate.

Format with markdown. Keep responses short — this is a chat window, not a document.

Do NOT add comments, docstrings, or type annotations to code unless asked. \
Only add comments where logic isn't self-evident.

ALWAYS respond with text after using tools. Never end a turn silently with only \
tool calls — the user needs to see your conclusion.

# Tool usage

NEVER propose changes to code you haven't read. Read first, then modify.

When editing, first understand the file's conventions — mimic code style, use \
existing libraries and patterns. Never assume a library is available; check the \
codebase first.

Be efficient. Use the minimum tool calls needed:
- Don't repeat a search you already did.
- Don't read a file you already read in this conversation.
- Don't list_files to browse around. Only if you need a specific filename.
- Be specific with search patterns — avoid broad sweeps.

When you need multiple independent pieces of information, describe what you're \
doing, then make your tool calls.

# Coding guidelines

Only make changes that are directly requested or clearly necessary. \
Avoid over-engineering:
- Don't add features, refactoring, or "improvements" beyond what was asked.
- Don't add error handling for scenarios that can't happen.
- Don't create abstractions for one-time operations.
- Don't design for hypothetical future requirements.
- If something is unused, delete it completely — no backwards-compat shims.

Follow security best practices. Never introduce command injection, XSS, SQL \
injection, or other vulnerabilities. Never log or expose secrets.

# Action safety

You can freely take local, reversible actions — reading files, editing code, \
running tests. For destructive or hard-to-reverse actions (deleting files, \
force operations, modifying git history), the approval gate will ask the user.

Don't use destructive actions as shortcuts. Investigate unexpected state before \
overwriting. If you discover unfamiliar files or branches, ask before deleting.

# Workflow

- Editing: read target file → edit_file (find & replace) → respond with summary
- New files: use write_file
- Commands requiring user approval will be gated automatically

NEVER use sudo. NEVER start servers or background processes. NEVER use & in commands. \
NEVER run destructive commands (rm -rf, etc).

# Connected systems

You also have data tools for Dylan's connected systems. Use them directly when asked:
- get_vibetotext_stats — voice transcription stats
- get_domain_dashboard — domain drop hunting results
- get_paper_dashboard — arXiv paper matches
- get_firewall_status — chat moderation stats
- get_system_overview — all systems at once

For casual conversation or general questions, just respond with text — no tools needed.

# Web search

You have Google Search available. Use it when you need:
- Current documentation, API references, or library usage examples
- Error messages or stack traces you don't recognize
- Recent releases, changelogs, or compatibility info
- Any factual question where your training data may be outdated

Search results are automatically grounded — just ask naturally and the search \
will be triggered when needed.
"""
