//! Jarvis terminal emulation layer.
//!
//! Thin adapter over `alacritty_terminal`. Consumers import from here,
//! never from `alacritty_terminal` directly.
//!
//! @module jarvis-terminal

pub mod color;
pub mod event;
pub mod pty;
pub mod shell;
pub mod size;
mod tests;

// Re-export alacritty_terminal types through our public API.
// Grid and indexing.
pub use alacritty_terminal::grid::{Dimensions, Grid, Scroll};
pub use alacritty_terminal::index::{Column, Line};

// Term and cell types.
pub use alacritty_terminal::term::cell::{Cell, Flags as CellFlags};
pub use alacritty_terminal::term::color::Colors;
pub use alacritty_terminal::term::{Config as TermConfig, RenderableContent, Term};

// VTE color types.
pub use vte::ansi::{Color as VteColor, NamedColor, Rgb as VteRgb};

// VTE processor for feeding bytes into Term.
pub use vte::ansi::Processor as VteProcessor;

// Our adapters.
pub use color::{default_color, vte_color_to_rgba};
pub use event::{JarvisEventProxy, TerminalEvent};
pub use size::SizeInfo;
