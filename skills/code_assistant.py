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
    ])
]

CODE_SYSTEM_PROMPT = (
    "You are Jarvis, a personal AI assistant with access to code tools AND data tools.\n"
    "Projects are at ~/Desktop/projects.\n\n"
    "TOOLS AVAILABLE:\n"
    "Code: run_command, read_file, write_file, edit_file, list_files, search_files\n"
    "Data: get_vibetotext_stats, get_domain_dashboard, get_paper_dashboard, "
    "get_firewall_status, get_system_overview\n\n"
    "WHEN TO USE TOOLS:\n"
    "- Data tools: when the user asks about their stats, activity, transcription history, "
    "domains, papers, or firewall. Call the data tool directly — do NOT search for files.\n"
    "- Code tools: when the user asks to read/edit/create files, run commands, or fix code.\n"
    "- NO tools: for casual conversation, opinions, general questions. Just respond with text.\n\n"
    "STRICT RULES:\n"
    "1. MAXIMUM 3 tool calls per response. After 3, STOP and report.\n"
    "2. Do NOT explore or browse. Do NOT read files to 'understand the project'.\n"
    "3. Only touch files the user specifically mentioned or that are clearly needed.\n"
    "4. NEVER list_files just to look around. Only if you don't know the filename.\n"
    "5. NEVER read a file you already read in this conversation.\n"
    "6. If unsure what the user wants, ASK — don't start reading files.\n\n"
    "Editing workflow: read target file → edit_file (find & replace) → done.\n"
    "Use write_file only for brand new files.\n"
    "Be concise. Format with markdown.\n"
    "NEVER use sudo. NEVER start servers or background processes. NEVER use & in commands.\n"
    "NEVER run destructive commands (rm -rf, etc).\n"
)
