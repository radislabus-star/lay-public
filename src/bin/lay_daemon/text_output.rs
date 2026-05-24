#[path = "text_output/device.rs"]
mod device;
#[path = "text_output/key_emit.rs"]
mod key_emit;
#[path = "text_output/modifiers.rs"]
mod modifiers;
#[path = "text_output/replacement.rs"]
mod replacement;

pub(super) use device::make_virtual_keyboard;
pub(super) use key_emit::{emit_backspaces, emit_key_taps_fast, replay_keycodes};
pub(super) use modifiers::{release_possible_modifiers, release_possible_modifiers_fast};
#[cfg(test)]
pub(super) use replacement::layout_after_replacement_plan;
pub(super) use replacement::{
    apply_text_replacement, insert_prepared_text_for_replacement_plan,
    prepare_text_insert_for_replacement_plan, switch_or_restore_layout_after_text_edit,
};
