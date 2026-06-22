use std::time::Instant;

use super::super::super::{
    layout_switch_policy, log, should_try_ime_text_backend, try_ime_replace_tail,
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
    let replace_tail_started = Instant::now();
    if !try_ime_replace_tail(original, replacement, "typing-assist").unwrap_or(false) {
        return None;
    }
    let replace_tail_ms = replace_tail_started.elapsed().as_millis();

    let target_layout = layout_switch_policy::target_layout_for_replacement(replacement, true);
    let layout_started = Instant::now();
    log("  typing-assist layout handled by IME engine");
    let layout_ms = layout_started.elapsed().as_millis();
    let remember_started = Instant::now();
    remember_ime_typing_correction(buf, events, original, replacement, rule_id, timing);
    let remember_ms = remember_started.elapsed().as_millis();
    let forward_started = Instant::now();
    let forwarded_spaces = forward_after_ime_replace(
        virtual_kbd,
        physical_grab,
        buf,
        target_layout,
        trailing_space_count(replacement),
    );
    let forward_ms = forward_started.elapsed().as_millis();
    lay::action_log::record_timing_profile(
        "typing-assist",
        "daemon-ime",
        &[
            ("decision", timing.decision_ms),
            ("ime_replace_tail_call", replace_tail_ms),
            ("layout", layout_ms),
            ("layout_ime_internal", 1),
            ("remember", remember_ms),
            ("forward", forward_ms),
            ("total", timing.started_at.elapsed().as_millis()),
        ],
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
