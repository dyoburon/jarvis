//! Built-in tool definitions for AI assistants.
//!
//! Tools are functions the AI can call to interact with the system
//! (run commands, read files, search, etc.).

use crate::ToolDefinition;

/// Create the built-in tool definitions that Jarvis exposes to AI models.
pub fn builtin_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "run_command".to_string(),
            description: "Execute a shell command and return its output.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "working_directory": {
                        "type": "string",
                        "description": "Working directory for the command (optional)"
                    }
                },
                "required": ["command"]
            }),
        },
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute or relative path to the file"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file, creating it if it doesn't exist.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        ToolDefinition {
            name: "search_files".to_string(),
            description: "Search for files matching a pattern using glob.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern (e.g., '**/*.rs')"
                    },
                    "directory": {
                        "type": "string",
                        "description": "Root directory to search from"
                    }
                },
                "required": ["pattern"]
            }),
        },
        ToolDefinition {
            name: "search_content".to_string(),
            description: "Search file contents for a regex pattern.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "directory": {
                        "type": "string",
                        "description": "Root directory to search from"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "Glob pattern to filter files (e.g., '*.rs')"
                    }
                },
                "required": ["pattern"]
            }),
        },
        ToolDefinition {
            name: "list_directory".to_string(),
            description: "List files and directories at a given path.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path"
                    }
                },
                "required": ["path"]
            }),
        },
    ]
}

/// Convert a tool definition to the Claude API format.
pub fn to_claude_tool(tool: &ToolDefinition) -> serde_json::Value {
    serde_json::json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.parameters,
    })
}

/// Convert a tool definition to the Gemini API format.
pub fn to_gemini_tool(tool: &ToolDefinition) -> serde_json::Value {
    serde_json::json!({
        "name": tool.name,
        "description": tool.description,
        "parameters": tool.parameters,
    })
}
