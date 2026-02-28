//! Typography configuration types.

use serde::{Deserialize, Serialize};

/// Typography configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    pub family: String,
    /// Font size in points (valid range: 8-32).
    pub size: u32,
    /// Title font size in points (valid range: 8-48).
    pub title_size: u32,
    /// Line height multiplier (valid range: 1.0-3.0).
    pub line_height: f64,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "Menlo".into(),
            size: 13,
            title_size: 15,
            line_height: 1.6,
        }
    }
}
