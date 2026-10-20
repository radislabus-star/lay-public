use evdev::KeyCode;

/// XKB physical names, independent of the active symbols/layout and X keycode offset.
pub(super) fn physical_key(name: [u8; 4]) -> KeyCode {
    use KeyCode as K;
    const ROWS: &[(&[u8; 2], &[KeyCode])] = &[
        (
            b"AE",
            &[
                K::KEY_1,
                K::KEY_2,
                K::KEY_3,
                K::KEY_4,
                K::KEY_5,
                K::KEY_6,
                K::KEY_7,
                K::KEY_8,
                K::KEY_9,
                K::KEY_0,
                K::KEY_MINUS,
                K::KEY_EQUAL,
            ],
        ),
        (
            b"AD",
            &[
                K::KEY_Q,
                K::KEY_W,
                K::KEY_E,
                K::KEY_R,
                K::KEY_T,
                K::KEY_Y,
                K::KEY_U,
                K::KEY_I,
                K::KEY_O,
                K::KEY_P,
                K::KEY_LEFTBRACE,
                K::KEY_RIGHTBRACE,
            ],
        ),
        (
            b"AC",
            &[
                K::KEY_A,
                K::KEY_S,
                K::KEY_D,
                K::KEY_F,
                K::KEY_G,
                K::KEY_H,
                K::KEY_J,
                K::KEY_K,
                K::KEY_L,
                K::KEY_SEMICOLON,
                K::KEY_APOSTROPHE,
            ],
        ),
        (
            b"AB",
            &[
                K::KEY_Z,
                K::KEY_X,
                K::KEY_C,
                K::KEY_V,
                K::KEY_B,
                K::KEY_N,
                K::KEY_M,
                K::KEY_COMMA,
                K::KEY_DOT,
                K::KEY_SLASH,
            ],
        ),
    ];
    for (prefix, keys) in ROWS {
        if &name[..2] == *prefix && name[2].is_ascii_digit() && name[3].is_ascii_digit() {
            let index = usize::from((name[2] - b'0') * 10 + name[3] - b'0');
            return index
                .checked_sub(1)
                .and_then(|i| keys.get(i))
                .copied()
                .unwrap_or(K::KEY_UNKNOWN);
        }
    }
    match &name {
        b"TLDE" => K::KEY_GRAVE,
        b"BKSL" => K::KEY_BACKSLASH,
        b"LSGT" => K::KEY_102ND,
        b"SPCE" => K::KEY_SPACE,
        b"LFSH" => K::KEY_LEFTSHIFT,
        b"RTSH" => K::KEY_RIGHTSHIFT,
        b"LCTL" => K::KEY_LEFTCTRL,
        b"RCTL" => K::KEY_RIGHTCTRL,
        b"LALT" => K::KEY_LEFTALT,
        b"RALT" => K::KEY_RIGHTALT,
        b"LWIN" => K::KEY_LEFTMETA,
        b"RWIN" => K::KEY_RIGHTMETA,
        b"RTRN" => K::KEY_ENTER,
        b"TAB\0" => K::KEY_TAB,
        b"ESC\0" => K::KEY_ESC,
        b"BKSP" => K::KEY_BACKSPACE,
        b"DELE" => K::KEY_DELETE,
        b"INS\0" => K::KEY_INSERT,
        b"HOME" => K::KEY_HOME,
        b"END\0" => K::KEY_END,
        b"PGUP" => K::KEY_PAGEUP,
        b"PGDN" => K::KEY_PAGEDOWN,
        b"LEFT" => K::KEY_LEFT,
        b"RGHT" => K::KEY_RIGHT,
        b"UP\0\0" => K::KEY_UP,
        b"DOWN" => K::KEY_DOWN,
        b"CAPS" => K::KEY_CAPSLOCK,
        b"NMLK" => K::KEY_NUMLOCK,
        b"PAUS" => K::KEY_PAUSE,
        _ => K::KEY_UNKNOWN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_physical_rows_without_assuming_xorg_keycode_offsets() {
        assert_eq!(physical_key(*b"AD01"), KeyCode::KEY_Q);
        assert_eq!(physical_key(*b"AE10"), KeyCode::KEY_0);
        assert_eq!(physical_key(*b"AC11"), KeyCode::KEY_APOSTROPHE);
        assert_eq!(physical_key(*b"AB10"), KeyCode::KEY_SLASH);
        assert_eq!(physical_key(*b"AE00"), KeyCode::KEY_UNKNOWN);
        assert_eq!(physical_key(*b"ZZ01"), KeyCode::KEY_UNKNOWN);
        assert_eq!(physical_key(*b"LFSH"), KeyCode::KEY_LEFTSHIFT);
        assert_eq!(physical_key(*b"PAUS"), KeyCode::KEY_PAUSE);
    }
}
