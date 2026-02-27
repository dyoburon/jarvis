use clap::Parser;

/// Jarvis — a GPU-accelerated terminal emulator with AI integration.
#[derive(Parser, Debug)]
#[command(name = "jarvis", version, about)]
pub struct Args {
    /// Execute a command instead of the default shell.
    #[arg(short = 'e', long)]
    pub execute: Option<String>,

    /// Working directory to start in.
    #[arg(short = 'd', long)]
    pub directory: Option<String>,

    /// Config file path override.
    #[arg(long)]
    pub config: Option<String>,

    /// Log level override (debug, info, warn, error).
    #[arg(long)]
    pub log_level: Option<String>,
}

pub fn parse() -> Args {
    Args::parse()
}
