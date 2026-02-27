use crate::keymap::{KeyBind, Modifier};

pub(super) const MOD_CTRL: u8 = 0b0001;
pub(super) const MOD_ALT: u8 = 0b0010;
pub(super) const MOD_SHIFT: u8 = 0b0100;
pub(super) const MOD_SUPER: u8 = 0b1000;

/// A canonical key representation for fast HashMap lookup.
///
/// Modifiers are stored as a bitmask for O(1) comparison rather than
/// sorting a `Vec<Modifier>` on every event.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    /// Bitmask: Ctrl=1, Alt=2, Shift=4, Super=8.
    pub mods: u8,
    /// Normalized key name (e.g. "G", "Enter", "F1").
    pub key: String,
}

impl KeyCombo {
    /// Build from a parsed [`KeyBind`].
    pub fn from_keybind(kb: &KeyBind) -> Self {
        let mut mods = 0u8;
        for m in &kb.modifiers {
            mods |= match m {
                Modifier::Ctrl => MOD_CTRL,
                Modifier::Alt => MOD_ALT,
                Modifier::Shift => MOD_SHIFT,
                Modifier::Super => MOD_SUPER,
            };
        }
        Self {
            mods,
            key: kb.key.clone(),
        }
    }

    /// Build from raw modifier booleans and a normalized key name.
    ///
    /// Use this to convert winit keyboard events into a `KeyCombo` for lookup.
    pub fn from_winit(ctrl: bool, alt: bool, shift: bool, super_key: bool, key: String) -> Self {
        let mut mods = 0u8;
        if ctrl {
            mods |= MOD_CTRL;
        }
        if alt {
            mods |= MOD_ALT;
        }
        if shift {
            mods |= MOD_SHIFT;
        }
        if super_key {
            mods |= MOD_SUPER;
        }
        Self { mods, key }
    }

    /// Reconstruct a [`KeyBind`] for display purposes.
    pub(super) fn to_keybind(&self) -> KeyBind {
        let mut modifiers = Vec::new();
        if self.mods & MOD_CTRL != 0 {
            modifiers.push(Modifier::Ctrl);
        }
        if self.mods & MOD_ALT != 0 {
            modifiers.push(Modifier::Alt);
        }
        if self.mods & MOD_SHIFT != 0 {
            modifiers.push(Modifier::Shift);
        }
        if self.mods & MOD_SUPER != 0 {
            modifiers.push(Modifier::Super);
        }
        KeyBind {
            modifiers,
            key: self.key.clone(),
        }
    }
}
