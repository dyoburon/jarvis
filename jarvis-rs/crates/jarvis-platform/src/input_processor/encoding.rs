/// Encode a key press into terminal escape sequences / bytes.
pub fn encode_key_for_terminal(key: &str, ctrl: bool, alt: bool, _shift: bool) -> Vec<u8> {
    let alt_prefix: &[u8] = if alt { b"\x1b" } else { b"" };

    match key {
        // Editing keys
        "Enter" => [alt_prefix, b"\r"].concat(),
        "Backspace" => [alt_prefix, b"\x7f"].concat(),
        "Tab" => b"\t".to_vec(),
        "Escape" => b"\x1b".to_vec(),
        "Space" => [alt_prefix, b" "].concat(),
        "Delete" => b"\x1b[3~".to_vec(),
        "Insert" => b"\x1b[2~".to_vec(),

        // Arrow keys
        "Up" => b"\x1b[A".to_vec(),
        "Down" => b"\x1b[B".to_vec(),
        "Right" => b"\x1b[C".to_vec(),
        "Left" => b"\x1b[D".to_vec(),

        // Navigation
        "Home" => b"\x1b[H".to_vec(),
        "End" => b"\x1b[F".to_vec(),
        "PageUp" => b"\x1b[5~".to_vec(),
        "PageDown" => b"\x1b[6~".to_vec(),

        // Function keys
        "F1" => b"\x1bOP".to_vec(),
        "F2" => b"\x1bOQ".to_vec(),
        "F3" => b"\x1bOR".to_vec(),
        "F4" => b"\x1bOS".to_vec(),
        "F5" => b"\x1b[15~".to_vec(),
        "F6" => b"\x1b[17~".to_vec(),
        "F7" => b"\x1b[18~".to_vec(),
        "F8" => b"\x1b[19~".to_vec(),
        "F9" => b"\x1b[20~".to_vec(),
        "F10" => b"\x1b[21~".to_vec(),
        "F11" => b"\x1b[23~".to_vec(),
        "F12" => b"\x1b[24~".to_vec(),

        _ => {
            if key.len() == 1 {
                let ch = key.chars().next().unwrap();
                if ctrl && ch.is_ascii_alphabetic() {
                    let ctrl_byte = (ch.to_ascii_lowercase() as u8) - b'a' + 1;
                    [alt_prefix, &[ctrl_byte]].concat()
                } else if ctrl && ch == '[' {
                    b"\x1b".to_vec()
                } else if ctrl && ch == '\\' {
                    vec![0x1c]
                } else if ctrl && ch == ']' {
                    vec![0x1d]
                } else {
                    [alt_prefix, key.as_bytes()].concat()
                }
            } else {
                Vec::new()
            }
        }
    }
}
