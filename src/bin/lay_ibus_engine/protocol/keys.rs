pub(crate) const KEY_BACKSPACE: u32 = 0xff08;
pub(crate) const KEY_TAB: u32 = 0xff09;
pub(crate) const KEY_ENTER: u32 = 0xff0d;
pub(crate) const KEY_KP_ENTER: u32 = 0xff8d;
pub(crate) const KEY_LEFT: u32 = 0xff51;
pub(crate) const KEY_UP: u32 = 0xff52;
pub(crate) const KEY_RIGHT: u32 = 0xff53;
pub(crate) const KEY_DOWN: u32 = 0xff54;
pub(crate) const KEY_LEFT_ALT: u32 = 0xffe9;
pub(crate) const KEY_RIGHT_ALT: u32 = 0xffea;
pub(crate) const KEY_ISO_LEVEL3_SHIFT: u32 = 0xfe03;
pub(crate) const KEY_LEFT_SHIFT: u32 = 0xffe1;
pub(crate) const KEY_RIGHT_SHIFT: u32 = 0xffe2;
pub(crate) const KEY_SPACE: u32 = 0x20;
pub(crate) const RELEASE_MASK: u32 = 1 << 30;

pub(crate) fn is_key_press(state: u32) -> bool {
    state & RELEASE_MASK == 0
}

pub(crate) fn is_shift_key(keyval: u32) -> bool {
    matches!(keyval, KEY_LEFT_SHIFT | KEY_RIGHT_SHIFT)
}

pub(crate) fn is_accept_completion_with_space_key(keyval: u32) -> bool {
    matches!(keyval, KEY_LEFT_ALT | KEY_RIGHT_ALT | KEY_ISO_LEVEL3_SHIFT)
}
