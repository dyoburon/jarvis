use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self { r, g, b, a: 255 })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self { r, g, b, a })
            }
            _ => None,
        }
    }

    pub fn from_rgba_string(s: &str) -> Option<Self> {
        let s = s.trim();
        let inner = s.strip_prefix("rgba(")?.strip_suffix(')')?;
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() != 4 {
            return None;
        }
        let r = parts[0].trim().parse::<u8>().ok()?;
        let g = parts[1].trim().parse::<u8>().ok()?;
        let b = parts[2].trim().parse::<u8>().ok()?;
        let a = parts[3].trim().parse::<u8>().ok()?;
        Some(Self { r, g, b, a })
    }

    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    pub fn to_rgba_string(&self) -> String {
        format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaneId(pub u32);

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pane-{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneKind {
    Terminal,
    WebView,
    ExternalApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppState {
    Starting,
    Running,
    ShuttingDown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_clone_and_equality() {
        let r = Rect {
            x: 10.0,
            y: 20.0,
            width: 800.0,
            height: 600.0,
        };
        let r2 = r;
        assert_eq!(r, r2);
    }

    #[test]
    fn rect_serialization() {
        let r = Rect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        let json = serde_json::to_string(&r).unwrap();
        let deserialized: Rect = serde_json::from_str(&json).unwrap();
        assert_eq!(r, deserialized);
    }

    #[test]
    fn color_from_hex_6() {
        let c = Color::from_hex("#ff8800").unwrap();
        assert_eq!(c, Color::from_rgba(255, 136, 0, 255));
    }

    #[test]
    fn color_from_hex_8() {
        let c = Color::from_hex("#ff880080").unwrap();
        assert_eq!(c, Color::from_rgba(255, 136, 0, 128));
    }

    #[test]
    fn color_from_hex_no_hash() {
        let c = Color::from_hex("00ff00").unwrap();
        assert_eq!(c, Color::from_rgba(0, 255, 0, 255));
    }

    #[test]
    fn color_from_hex_invalid() {
        assert!(Color::from_hex("zzzzzz").is_none());
        assert!(Color::from_hex("#abc").is_none());
        assert!(Color::from_hex("").is_none());
    }

    #[test]
    fn color_from_rgba_string() {
        let c = Color::from_rgba_string("rgba(10,20,30,255)").unwrap();
        assert_eq!(c, Color::from_rgba(10, 20, 30, 255));
    }

    #[test]
    fn color_from_rgba_string_with_spaces() {
        let c = Color::from_rgba_string("rgba( 10 , 20 , 30 , 128 )").unwrap();
        assert_eq!(c, Color::from_rgba(10, 20, 30, 128));
    }

    #[test]
    fn color_from_rgba_string_invalid() {
        assert!(Color::from_rgba_string("rgb(10,20,30)").is_none());
        assert!(Color::from_rgba_string("rgba(10,20,30)").is_none());
        assert!(Color::from_rgba_string("rgba(10,20,30,40,50)").is_none());
    }

    #[test]
    fn color_to_hex_opaque() {
        let c = Color::from_rgba(255, 0, 128, 255);
        assert_eq!(c.to_hex(), "#ff0080");
    }

    #[test]
    fn color_to_hex_with_alpha() {
        let c = Color::from_rgba(255, 0, 128, 128);
        assert_eq!(c.to_hex(), "#ff008080");
    }

    #[test]
    fn color_to_rgba_string() {
        let c = Color::from_rgba(10, 20, 30, 255);
        assert_eq!(c.to_rgba_string(), "rgba(10,20,30,255)");
    }

    #[test]
    fn color_roundtrip_hex() {
        let original = Color::from_rgba(171, 205, 239, 255);
        let hex = original.to_hex();
        let parsed = Color::from_hex(&hex).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn pane_id_display() {
        let id = PaneId(42);
        assert_eq!(id.to_string(), "pane-42");
    }

    #[test]
    fn pane_id_hash_and_eq() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PaneId(1));
        set.insert(PaneId(2));
        set.insert(PaneId(1));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn pane_id_serialization() {
        let id = PaneId(7);
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: PaneId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn pane_kind_variants() {
        let kinds = [PaneKind::Terminal, PaneKind::WebView, PaneKind::ExternalApp];
        for kind in &kinds {
            let json = serde_json::to_string(kind).unwrap();
            let deserialized: PaneKind = serde_json::from_str(&json).unwrap();
            assert_eq!(*kind, deserialized);
        }
    }

    #[test]
    fn app_state_variants() {
        let states = [
            AppState::Starting,
            AppState::Running,
            AppState::ShuttingDown,
        ];
        for state in &states {
            let json = serde_json::to_string(state).unwrap();
            let deserialized: AppState = serde_json::from_str(&json).unwrap();
            assert_eq!(*state, deserialized);
        }
    }
}
