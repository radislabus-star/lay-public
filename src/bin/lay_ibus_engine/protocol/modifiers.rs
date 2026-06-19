const CONTROL_MASK: u32 = 1 << 2;
const MOD1_MASK: u32 = 1 << 3;
const MOD4_MASK: u32 = 1 << 6;
const SUPER_MASK: u32 = 1 << 26;
const HYPER_MASK: u32 = 1 << 27;
const META_MASK: u32 = 1 << 28;

pub(crate) fn has_command_modifier(state: u32) -> bool {
    state & (CONTROL_MASK | MOD1_MASK | MOD4_MASK | SUPER_MASK | HYPER_MASK | META_MASK) != 0
}

#[cfg(test)]
mod tests {
    use super::{has_command_modifier, CONTROL_MASK, MOD1_MASK};
    use crate::protocol::RELEASE_MASK;

    #[test]
    fn command_modifier_detects_ctrl_and_alt_without_release_noise() {
        assert!(has_command_modifier(CONTROL_MASK));
        assert!(has_command_modifier(MOD1_MASK));
        assert!(has_command_modifier(CONTROL_MASK | RELEASE_MASK));
        assert!(!has_command_modifier(0));
        assert!(!has_command_modifier(1));
    }
}
