#[path = "text_output/device.rs"]
mod device;
#[path = "text_output/key_emit.rs"]
mod key_emit;
#[path = "text_output/layout_preflight.rs"]
mod layout_preflight;
#[path = "text_output/modifiers.rs"]
mod modifiers;
#[path = "text_output/observable_state.rs"]
mod observable_state;
#[path = "text_output/replacement.rs"]
mod replacement;

pub(super) use device::make_virtual_keyboard;
pub(super) use key_emit::{
    emit_backspaces, emit_backspaces_fast, emit_key_taps_fast, replay_keycodes,
    replay_keycodes_fast_after_modifier_cleanup,
};
pub(super) use layout_preflight::LayoutCapabilityPreflight;
pub(super) use modifiers::{release_possible_modifiers, release_possible_modifiers_fast};
pub(super) use observable_state::{
    DaemonTextContext, DaemonTextContextObserver, DaemonTextObservation,
};
pub(super) use replacement::{
    apply_text_replacement_pipeline, switch_or_restore_layout_after_text_edit,
};
