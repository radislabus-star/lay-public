use evdev::{uinput::VirtualDevice, AttributeSet, KeyCode};

pub(crate) fn make_virtual_keyboard() -> std::io::Result<VirtualDevice> {
    use KeyCode as K;
    let mut keys = AttributeSet::new();
    let typing = [
        K::KEY_A,
        K::KEY_B,
        K::KEY_C,
        K::KEY_D,
        K::KEY_E,
        K::KEY_F,
        K::KEY_G,
        K::KEY_H,
        K::KEY_I,
        K::KEY_J,
        K::KEY_K,
        K::KEY_L,
        K::KEY_M,
        K::KEY_N,
        K::KEY_O,
        K::KEY_P,
        K::KEY_Q,
        K::KEY_R,
        K::KEY_S,
        K::KEY_T,
        K::KEY_U,
        K::KEY_V,
        K::KEY_W,
        K::KEY_X,
        K::KEY_Y,
        K::KEY_Z,
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
        K::KEY_SPACE,
        K::KEY_SEMICOLON,
        K::KEY_APOSTROPHE,
        K::KEY_COMMA,
        K::KEY_DOT,
        K::KEY_LEFTBRACE,
        K::KEY_RIGHTBRACE,
        K::KEY_GRAVE,
        K::KEY_SLASH,
        K::KEY_BACKSLASH,
        K::KEY_MINUS,
        K::KEY_EQUAL,
        K::KEY_LEFTSHIFT,
        K::KEY_RIGHTSHIFT,
        K::KEY_LEFTALT,
        K::KEY_RIGHTALT,
        K::KEY_LEFTCTRL,
        K::KEY_RIGHTCTRL,
        K::KEY_INSERT,
        K::KEY_LEFT,
        K::KEY_RIGHT,
        K::KEY_BACKSPACE,
    ];
    for k in typing.iter() {
        keys.insert(*k);
    }
    VirtualDevice::builder()?
        .name("lay-virtual-keyboard")
        .with_keys(&keys)?
        .build()
}
