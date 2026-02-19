"""Gemini function-calling tool declarations and system prompt for the coding agent."""

from google.genai import types

CODE_TOOLS = [
    types.Tool(function_declarations=[
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
    ])
]

CODE_SYSTEM_PROMPT = (
    "You are Jarvis, a coding assistant. Projects are at ~/Desktop/projects.\n\n"
    "STRICT RULES — you MUST follow these:\n"
    "1. MAXIMUM 3 tool calls per response. After 3, STOP and explain what you found.\n"
    "2. Do NOT explore. Do NOT read files to 'understand the project'. Only touch files "
    "the user specifically asked about.\n"
    "3. If the user says 'edit X' — read X, edit X, done. That's 2 tool calls.\n"
    "4. If the user says 'tell me about X' — read X, then answer. That's 1 tool call.\n"
    "5. NEVER list_files just to browse. Only use it if you don't know the filename.\n"
    "6. NEVER read a file you already read in this conversation.\n\n"
    "Editing workflow: read target file → edit_file (find & replace) → done.\n"
    "Use write_file only for brand new files.\n"
    "Be concise. Format with markdown. Never use sudo.\n"
)
