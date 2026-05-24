use evdev::uinput::VirtualDevice;
use lay::keyboard::{preferred_layout_for_text, KeyEvent};
use lay::text_edit::TextReplacement;
use lay::word_buffer::WordBuffer;
use std::time::Instant;

use super::super::super::physical_input_grab::PhysicalInputGrab;
use super::super::super::{
    active_auto_switch_layout, log, read_current_layout_is_ru, should_try_ime_text_backend,
    switch_or_restore_layout_after_text_edit, try_ime_replace_tail,
};
use super::super::TypingAssistOutcome;
use super::memory::remember_typing_assist_correction;

pub(crate) fn try_apply_ime_replacement(
    buf: &mut WordBuffer,
    virtual_kbd: &mut Option<&mut VirtualDevice>,
    physical_grab: &mut PhysicalInputGrab<'_>,
    events: &[KeyEvent],
    original: &str,
    replacement: &str,
    started_at: Instant,
) -> Option<TypingAssistOutcome> {
    if !should_try_ime_text_backend() {
        return None;
    }
    let original_layout = read_current_layout_is_ru().ok();
    if !try_ime_replace_tail(original, replacement, "typing-assist").unwrap_or(false) {
        return None;
    }

    let target_layout = preferred_layout_for_text(replacement, true);
    switch_or_restore_layout_after_text_edit(
        active_auto_switch_layout(),
        target_layout,
        original_layout,
        "typing-assist",
        false,
    );
    remember_typing_assist_correction(
        buf,
        events,
        &TextReplacement {
            move_left: 0,
            backspaces: original.chars().count() as u32,
            insert: replacement.to_string(),
            move_right: 0,
        },
        original,
        replacement,
        0,
        started_at,
    );
    if let Some(kbd) = virtual_kbd.as_deref_mut() {
        physical_grab.forward_queued_typing(kbd, buf, target_layout, "typing-assist");
    }
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} через IME за {}ms",
        original,
        replacement,
        started_at.elapsed().as_millis()
    ));
    Some(TypingAssistOutcome::Applied {
        layout_is_ru: target_layout,
    })
}
