use evdev::{uinput::VirtualDevice, AttributeSet, KeyCode};

pub(super) const VIRTUAL_KEYBOARD_KEYS: &[KeyCode] = &[
    KeyCode::KEY_A,
    KeyCode::KEY_B,
    KeyCode::KEY_C,
    KeyCode::KEY_D,
    KeyCode::KEY_E,
    KeyCode::KEY_F,
    KeyCode::KEY_G,
    KeyCode::KEY_H,
    KeyCode::KEY_I,
    KeyCode::KEY_J,
    KeyCode::KEY_K,
    KeyCode::KEY_L,
    KeyCode::KEY_M,
    KeyCode::KEY_N,
    KeyCode::KEY_O,
    KeyCode::KEY_P,
    KeyCode::KEY_Q,
    KeyCode::KEY_R,
    KeyCode::KEY_S,
    KeyCode::KEY_T,
    KeyCode::KEY_U,
    KeyCode::KEY_V,
    KeyCode::KEY_W,
    KeyCode::KEY_X,
    KeyCode::KEY_Y,
    KeyCode::KEY_Z,
    KeyCode::KEY_1,
    KeyCode::KEY_2,
    KeyCode::KEY_3,
    KeyCode::KEY_4,
    KeyCode::KEY_5,
    KeyCode::KEY_6,
    KeyCode::KEY_7,
    KeyCode::KEY_8,
    KeyCode::KEY_9,
    KeyCode::KEY_0,
    KeyCode::KEY_SPACE,
    KeyCode::KEY_SEMICOLON,
    KeyCode::KEY_APOSTROPHE,
    KeyCode::KEY_COMMA,
    KeyCode::KEY_DOT,
    KeyCode::KEY_LEFTBRACE,
    KeyCode::KEY_RIGHTBRACE,
    KeyCode::KEY_GRAVE,
    KeyCode::KEY_SLASH,
    KeyCode::KEY_BACKSLASH,
    KeyCode::KEY_MINUS,
    KeyCode::KEY_EQUAL,
    KeyCode::KEY_LEFTSHIFT,
    KeyCode::KEY_RIGHTSHIFT,
    KeyCode::KEY_LEFTALT,
    KeyCode::KEY_RIGHTALT,
    KeyCode::KEY_LEFTCTRL,
    KeyCode::KEY_RIGHTCTRL,
    KeyCode::KEY_INSERT,
    KeyCode::KEY_LEFT,
    KeyCode::KEY_RIGHT,
    KeyCode::KEY_BACKSPACE,
];

pub(crate) fn make_virtual_keyboard() -> std::io::Result<VirtualDevice> {
    let mut keys = AttributeSet::new();
    for k in VIRTUAL_KEYBOARD_KEYS {
        keys.insert(*k);
    }
    VirtualDevice::builder()?
        .name("lay-virtual-keyboard")
        .with_keys(&keys)?
        .build()
}
