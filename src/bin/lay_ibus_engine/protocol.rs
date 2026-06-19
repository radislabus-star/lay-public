use std::sync::{Arc, Mutex};

pub(crate) const BUS_NAME: &str = "io.github.radislabus_star.LayIme";
pub(crate) const BUS_PATH: &str = "/io/github/radislabus_star/LayIme";
pub(crate) const IBUS_ENGINE_NAME: &str = "org.freedesktop.IBus.Lay";
pub(crate) const IBUS_FACTORY_PATH: &str = "/org/freedesktop/IBus/Factory";
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
const CONTROL_MASK: u32 = 1 << 2;
const MOD1_MASK: u32 = 1 << 3;
const MOD4_MASK: u32 = 1 << 6;
const SUPER_MASK: u32 = 1 << 26;
const HYPER_MASK: u32 = 1 << 27;
const META_MASK: u32 = 1 << 28;

pub(crate) fn is_key_press(state: u32) -> bool {
    state & RELEASE_MASK == 0
}

pub(crate) fn is_shift_key(keyval: u32) -> bool {
    matches!(keyval, KEY_LEFT_SHIFT | KEY_RIGHT_SHIFT)
}

pub(crate) fn is_accept_completion_with_space_key(keyval: u32) -> bool {
    matches!(keyval, KEY_LEFT_ALT | KEY_RIGHT_ALT | KEY_ISO_LEVEL3_SHIFT)
}

pub(crate) fn has_command_modifier(state: u32) -> bool {
    state & (CONTROL_MASK | MOD1_MASK | MOD4_MASK | SUPER_MASK | HYPER_MASK | META_MASK) != 0
}

#[derive(Debug, Default)]
pub(crate) struct SharedState {
    pub(crate) active_path: Option<String>,
    pub(crate) next_engine_id: u32,
}

pub(crate) type Shared = Arc<Mutex<SharedState>>;

#[cfg(test)]
mod tests {
    use super::{has_command_modifier, CONTROL_MASK, MOD1_MASK, RELEASE_MASK};

    #[test]
    fn command_modifier_detects_ctrl_and_alt_without_release_noise() {
        assert!(has_command_modifier(CONTROL_MASK));
        assert!(has_command_modifier(MOD1_MASK));
        assert!(has_command_modifier(CONTROL_MASK | RELEASE_MASK));
        assert!(!has_command_modifier(0));
        assert!(!has_command_modifier(1));
    }
}
