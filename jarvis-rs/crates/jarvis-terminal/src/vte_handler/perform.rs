//! `vte::Perform` implementation for Grid: print, execute, and DCS hooks.
//! CSI dispatch is in `csi_dispatch.rs`, ESC/OSC dispatch in `esc_osc.rs`.

use tracing::trace;
use vte::{Params, Perform};

use crate::grid::Grid;

impl Perform for Grid {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => self.backspace(), // BS
            0x09 => self.tab(),       // HT
            0x0A..=0x0C => {
                // LF, VT, FF
                self.newline();
            }
            0x0D => self.carriage_return(), // CR
            0x07 => {
                // BEL
                trace!("BEL received");
            }
            _ => {
                trace!("unhandled execute byte: 0x{byte:02X}");
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        self.dispatch_csi(params, intermediates, action);
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        self.dispatch_esc(intermediates, byte);
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        self.dispatch_osc(params);
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS hook - no-op for now.
    }

    fn put(&mut self, _byte: u8) {
        // DCS put - no-op for now.
    }

    fn unhook(&mut self) {
        // DCS unhook - no-op for now.
    }
}
