use super::super::super::{
    apply_text_replacement_pipeline, layout_switch_policy, log,
    switch_or_restore_layout_after_text_edit, tail_replace_policy,
};
use super::super::TypingAssistOutcome;
use super::memory::{remember_typing_assist_correction, TypingAssistMemoryContext};
use lay::decoder::CorrectionTrigger;

#[path = "minimal/context.rs"]
mod context;
pub(crate) use context::MinimalTypingReplacementContext;
pub(crate) fn apply_minimal_typing_replacement(
    ctx: MinimalTypingReplacementContext<'_, '_>,
) -> TypingAssistOutcome {
    let MinimalTypingReplacementContext {
        buf,
        events,
        edit,
        original,
        replacement,
        rule_id,
        input_gate,
        cursor_offset,
        timing,
        physical_grab,
        kbd,
        original_layout,
        prefer_full_token_plan,
    } = ctx;
    let plan = if tail_replace_policy::full_tail_replace_required(original) {
        Some(lay::text_edit::TextReplacement {
            move_left: 0,
            backspaces: original.chars().count() as u32,
            insert: replacement.to_string(),
            move_right: 0,
        })
    } else if prefer_full_token_plan && matches!(edit.trigger, CorrectionTrigger::AfterSpace) {
        edit.verified_full_token_plan_for_cursor(cursor_offset)
    } else {
        edit.verified_plan_for_cursor(cursor_offset)
    };
    let Some(plan) = plan else {
        log("⚠ typing-assist skipped before delete: edit plan invariant failed");
        return TypingAssistOutcome::NoCorrection;
    };
    let edit_action =
        edit.authorize_verified_replacement("typing-assist", original, replacement, plan.clone());
    lay::action_log::record_candidate_edit_action_before_apply(
        &edit_action,
        lay::action_log::MutationLogRoute::TYPING_ASSIST_MINIMAL,
        input_gate.clone(),
    );
    let backend_action = lay::text_edit::authorize_backend_edit(
        lay::text_edit::TextEditBackend::Daemon,
        edit_action,
    );
    let backend = backend_action.backend;
    let reason = backend_action.reason;
    let Some(authorized_edit) = backend_action.into_authorized() else {
        log(&format!(
            "⚠ typing-assist output blocked by executor contract: reason={} backend={} original={:?} replacement={:?}",
            reason,
            backend.as_str(),
            original,
            replacement
        ));
        return TypingAssistOutcome::NoCorrection;
    };
    log(&format!(
        "  typing-assist plan: left={} bs={} insert={:?} right={}",
        plan.move_left, plan.backspaces, plan.insert, plan.move_right
    ));
    let fast_output = physical_grab.is_active();
    let insert_outcome = match apply_text_replacement_pipeline(
        kbd,
        authorized_edit,
        original_layout.unwrap_or(true),
        original_layout,
        "typing-assist",
        fast_output,
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            e.log("typing-assist", "minimal replace failed");
            return TypingAssistOutcome::NoCorrection;
        }
    };
    remember_typing_assist_correction(TypingAssistMemoryContext {
        buf,
        events,
        plan: &plan,
        original,
        replacement,
        rule_id,
        input_gate,
        cursor_offset,
        timing,
    });
    let force_target_layout =
        layout_switch_policy::force_target_layout_for_replacement(original, replacement);
    if force_target_layout {
        switch_or_restore_layout_after_text_edit(
            true,
            insert_outcome.layout_is_ru,
            original_layout,
            "typing-assist",
            insert_outcome.layout_already_set,
        );
    }
    let _ = physical_grab.forward_queued_typing(
        kbd,
        buf,
        insert_outcome.layout_is_ru,
        "typing-assist",
        lay::word_reader::trailing_whitespace_char_count(replacement),
    );
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} за {}ms",
        original,
        replacement,
        timing.started_at.elapsed().as_millis()
    ));
    TypingAssistOutcome::Applied {
        layout_is_ru: insert_outcome.layout_is_ru,
    }
}
