//! SGR (Select Graphic Rendition) parsing helper.

use tracing::trace;
use vte::Params;

use crate::grid::{Grid, TerminalColor};

impl Grid {
    /// Parse SGR (Select Graphic Rendition) parameters and apply them to
    /// `self.attrs`.
    pub(crate) fn handle_sgr(&mut self, params: &Params) {
        let mut iter = params.iter();

        // If there are no params at all, treat as SGR 0 (reset).
        let first = match iter.next() {
            Some(sub) => sub,
            None => {
                self.attrs = Default::default();
                return;
            }
        };

        // We process the first sub-param group, then continue with the rest.
        // A fixed-size stack buffer avoids heap allocation on every SGR call.
        let mut groups_buf: [&[u16]; 32] = [&[]; 32];
        groups_buf[0] = first;
        let mut groups_len = 1;
        for sub in iter {
            if groups_len < groups_buf.len() {
                groups_buf[groups_len] = sub;
                groups_len += 1;
            }
        }
        let groups = &groups_buf[..groups_len];

        let mut i = 0;
        while i < groups.len() {
            let sub = groups[i];
            let code = sub[0];
            match code {
                0 => self.attrs = Default::default(),
                1 => self.attrs.bold = true,
                2 => self.attrs.dim = true,
                3 => self.attrs.italic = true,
                4 => self.attrs.underline = true,
                5 => self.attrs.blink = true,
                7 => self.attrs.inverse = true,
                8 => self.attrs.hidden = true,
                9 => self.attrs.strikethrough = true,
                22 => {
                    self.attrs.bold = false;
                    self.attrs.dim = false;
                }
                23 => self.attrs.italic = false,
                24 => self.attrs.underline = false,
                25 => self.attrs.blink = false,
                27 => self.attrs.inverse = false,
                28 => self.attrs.hidden = false,
                29 => self.attrs.strikethrough = false,
                // Standard foreground colors (30-37).
                30..=37 => {
                    self.attrs.fg = TerminalColor::Indexed((code - 30) as u8);
                }
                // Extended foreground.
                38 => {
                    i += 1;
                    self.parse_extended_color(groups, &mut i, true);
                    continue; // i already advanced
                }
                39 => self.attrs.fg = TerminalColor::Default,
                // Standard background colors (40-47).
                40..=47 => {
                    self.attrs.bg = TerminalColor::Indexed((code - 40) as u8);
                }
                // Extended background.
                48 => {
                    i += 1;
                    self.parse_extended_color(groups, &mut i, false);
                    continue;
                }
                49 => self.attrs.bg = TerminalColor::Default,
                // Bright foreground (90-97).
                90..=97 => {
                    self.attrs.fg = TerminalColor::Indexed((code - 90 + 8) as u8);
                }
                // Bright background (100-107).
                100..=107 => {
                    self.attrs.bg = TerminalColor::Indexed((code - 100 + 8) as u8);
                }
                _ => {
                    trace!("unhandled SGR code: {code}");
                }
            }
            i += 1;
        }
    }

    /// Parse an extended color specification (used after SGR 38 or 48).
    /// `is_fg` controls whether the result is applied to foreground or
    /// background.
    ///
    /// Expected forms:
    ///   38;5;N        -- 256-color palette
    ///   38;2;R;G;B    -- 24-bit true-color
    fn parse_extended_color(&mut self, groups: &[&[u16]], i: &mut usize, is_fg: bool) {
        if *i >= groups.len() {
            return;
        }
        let mode = groups[*i][0];
        match mode {
            5 => {
                // 256-color: next param is the color index.
                *i += 1;
                if *i < groups.len() {
                    let idx = groups[*i][0] as u8;
                    let color = TerminalColor::Indexed(idx);
                    if is_fg {
                        self.attrs.fg = color;
                    } else {
                        self.attrs.bg = color;
                    }
                    *i += 1;
                }
            }
            2 => {
                // True-color: next three params are R, G, B.
                if *i + 3 <= groups.len() {
                    let r = groups[*i + 1][0] as u8;
                    let g = groups[*i + 2][0] as u8;
                    let b = groups[*i + 3][0] as u8;
                    let color = TerminalColor::Rgb(r, g, b);
                    if is_fg {
                        self.attrs.fg = color;
                    } else {
                        self.attrs.bg = color;
                    }
                    *i += 4;
                } else {
                    *i = groups.len();
                }
            }
            _ => {
                *i += 1;
            }
        }
    }
}
