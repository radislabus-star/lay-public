use super::super::super::{
    layout_switch_policy, log, should_try_ime_text_backend,
    switch_or_restore_layout_after_text_edit, try_ime_replace_tail,
};
use super::super::TypingAssistOutcome;
#[path = "ime/context.rs"]
mod context;
#[path = "ime/forward.rs"]
mod forward;
#[path = "ime/remember.rs"]
mod remember;
#[path = "ime/timing.rs"]
mod timing_profile;
pub(crate) use context::ImeTypingReplacementContext;
use forward::{forward_after_ime_replace, trailing_space_count};
use lay::text_edit::TextReplacement;
use remember::remember_ime_typing_correction;
use timing_profile::record_ime_timing;
pub(crate) fn try_apply_ime_replacement(
    ctx: ImeTypingReplacementContext<'_, '_, '_>,
) -> Option<TypingAssistOutcome> {
    let ImeTypingReplacementContext {
        buf,
        virtual_kbd,
        physical_grab,
        events,
        edit,
        original,
        replacement,
        rule_id,
        input_gate,
        timing,
    } = ctx;
    if !should_try_ime_text_backend() {
        return None;
    }
    let plan = TextReplacement {
        move_left: 0,
        backspaces: original.chars().count() as u32,
        insert: replacement.to_string(),
        move_right: 0,
    };
    let edit_action = edit.authorize_verified_replacement(
        "typing-assist-ime",
        original,
        replacement,
        plan.clone(),
    );
    lay::action_log::record_candidate_edit_action_before_apply(
        &edit_action,
        lay::action_log::MutationLogRoute::TYPING_ASSIST_IME,
        input_gate.clone(),
    );
    let backend_action =
        lay::text_edit::authorize_backend_edit(lay::text_edit::TextEditBackend::Ime, &edit_action);
    let Some(_authorized_edit) = backend_action.authorized() else {
        log(&format!(
            "⚠ typing-assist IME blocked by executor contract: reason={} backend={} original={:?} replacement={:?}",
            backend_action.reason,
            backend_action.backend.as_str(),
            original,
            replacement
        ));
        return None;
    };
    let replace_tail_started = std::time::Instant::now();
    if !try_ime_replace_tail(original, replacement, "typing-assist").unwrap_or(false) {
        return None;
    }
    let replace_tail_ms = replace_tail_started.elapsed().as_millis();
    let target_layout = layout_switch_policy::target_layout_for_replacement(replacement, true);
    let layout_started = std::time::Instant::now();
    let force_layout =
        layout_switch_policy::force_target_layout_for_replacement(original, replacement);
    if force_layout {
        switch_or_restore_layout_after_text_edit(true, target_layout, None, "typing-assist", false);
    }
    let layout_ms = layout_started.elapsed().as_millis();
    let remember_started = std::time::Instant::now();
    remember_ime_typing_correction(
        buf,
        events,
        original,
        replacement,
        rule_id,
        input_gate,
        timing,
    );
    let remember_ms = remember_started.elapsed().as_millis();
    let forward_started = std::time::Instant::now();
    let _ = forward_after_ime_replace(
        virtual_kbd,
        physical_grab,
        buf,
        target_layout,
        trailing_space_count(replacement),
    );
    let forward_ms = forward_started.elapsed().as_millis();
    record_ime_timing(timing, replace_tail_ms, layout_ms, remember_ms, forward_ms);
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} через IME за {}ms",
        original,
        replacement,
        timing.started_at.elapsed().as_millis()
    ));
    Some(TypingAssistOutcome::Applied {
        layout_is_ru: target_layout,
    })
}
