use std::time::Instant;

use super::super::super::{
    active_auto_switch_layout, layout_switch_policy, log, read_current_layout_is_ru,
    should_try_ime_text_backend, switch_or_restore_layout_after_text_edit, try_ime_replace_tail,
};
use super::super::TypingAssistOutcome;
use super::memory::TypingAssistTiming;
use super::queued::next_correction_after_forwarded_spaces;

#[path = "ime/context.rs"]
mod context;
#[path = "ime/forward.rs"]
mod forward;
#[path = "ime/remember.rs"]
mod remember;

pub(crate) use context::ImeTypingReplacementContext;
use forward::forward_after_ime_replace;
use remember::remember_ime_typing_correction;

pub(crate) fn try_apply_ime_replacement(
    ctx: ImeTypingReplacementContext<'_, '_, '_>,
) -> Option<TypingAssistOutcome> {
    let ImeTypingReplacementContext {
        buf,
        virtual_kbd,
        physical_grab,
        events,
        original,
        replacement,
        rule_id,
        timing,
    } = ctx;
    if !should_try_ime_text_backend() {
        return None;
    }
    let original_layout = read_current_layout_is_ru().ok();
    if !try_ime_replace_tail(original, replacement, "typing-assist").unwrap_or(false) {
        return None;
    }

    let target_layout = layout_switch_policy::target_layout_for_replacement(replacement, true);
    let force_target_layout =
        layout_switch_policy::force_target_layout_for_replacement(original, replacement);
    switch_or_restore_layout_after_text_edit(
        active_auto_switch_layout() || force_target_layout,
        target_layout,
        original_layout,
        "typing-assist",
        false,
    );
    remember_ime_typing_correction(buf, events, original, replacement, rule_id, timing);
    let forwarded_spaces = forward_after_ime_replace(
        virtual_kbd,
        physical_grab,
        buf,
        target_layout,
        trailing_space_count(replacement),
    );
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} через IME за {}ms",
        original,
        replacement,
        timing.started_at.elapsed().as_millis()
    ));
    if let Some(next) = next_correction_after_forwarded_spaces(buf, forwarded_spaces) {
        let (next_original, next_replacement) =
            (next.edit.original.clone(), next.edit.replacement.clone());
        return try_apply_ime_replacement(ImeTypingReplacementContext {
            buf,
            virtual_kbd,
            physical_grab,
            events: &next.events,
            original: &next_original,
            replacement: &next_replacement,
            rule_id: next.rule_id.as_deref(),
            timing: TypingAssistTiming {
                decision_ms: next.decision_ms,
                started_at: Instant::now(),
            },
        });
    }
    Some(TypingAssistOutcome::Applied {
        layout_is_ru: target_layout,
    })
}

fn trailing_space_count(text: &str) -> usize {
    text.chars().rev().take_while(|ch| *ch == ' ').count()
}
