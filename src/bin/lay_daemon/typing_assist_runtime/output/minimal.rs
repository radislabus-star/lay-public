use super::super::super::{
    apply_text_replacement_pipeline, layout_switch_policy, log,
    switch_or_restore_layout_after_text_edit, tail_replace_policy,
};
use super::super::TypingAssistOutcome;
use super::memory::{
    remember_typing_assist_correction, TypingAssistMemoryContext, TypingAssistTiming,
};
use super::queued::next_correction_after_forwarded_spaces;
use lay::decoder::CorrectionTrigger;
use std::time::Instant;

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
    let source_id = input_gate
        .as_ref()
        .and_then(|trace| trace.selected_source_id.as_deref());
    let error_class = input_gate
        .as_ref()
        .and_then(|trace| trace.selected_error_class.as_deref());
    let confidence_milli = input_gate
        .as_ref()
        .and_then(|trace| trace.scoreboard.as_ref())
        .and_then(|scoreboard| scoreboard.selected_bayes_posterior_milli)
        .unwrap_or(0);
    let transition = input_gate
        .as_ref()
        .map(lay::action_log::RecentActionGateTrace::selected_transition_audit)
        .unwrap_or_default();
    let edit_action = lay::text_edit::authorize_replacement_with_transition(
        "typing-assist",
        confidence_milli,
        original,
        replacement,
        plan.clone(),
        source_id,
        error_class,
        transition,
    );
    lay::action_log::record_candidate_edit_action_before_apply(&edit_action, input_gate.clone());
    if !edit_action.allow_apply() {
        log(&format!(
            "⚠ typing-assist output blocked by edit-plan safety: reason={} original={:?} replacement={:?}",
            edit_action.safety_reason(),
            original,
            replacement
        ));
        return TypingAssistOutcome::NoCorrection;
    }
    log(&format!(
        "  typing-assist plan: left={} bs={} insert={:?} right={}",
        plan.move_left, plan.backspaces, plan.insert, plan.move_right
    ));
    let fast_output = physical_grab.is_active();
    let insert_outcome = match apply_text_replacement_pipeline(
        kbd,
        &plan,
        replacement,
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
    let forwarded = physical_grab.forward_queued_typing(
        kbd,
        buf,
        insert_outcome.layout_is_ru,
        "typing-assist",
        trailing_space_count(replacement),
    );
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} за {}ms",
        original,
        replacement,
        timing.started_at.elapsed().as_millis()
    ));
    if let Some(next) = next_correction_after_forwarded_spaces(buf, forwarded.spaces) {
        let (next_original, next_replacement) =
            (next.edit.original.clone(), next.edit.replacement.clone());
        return apply_minimal_typing_replacement(MinimalTypingReplacementContext {
            buf,
            events: &next.events,
            edit: &next.edit,
            original: &next_original,
            replacement: &next_replacement,
            rule_id: next.rule_id.as_deref(),
            input_gate: next.input_gate,
            cursor_offset: 0,
            timing: TypingAssistTiming {
                decision_ms: next.decision_ms,
                started_at: Instant::now(),
            },
            physical_grab,
            kbd,
            original_layout,
            prefer_full_token_plan,
        });
    }
    TypingAssistOutcome::Applied {
        layout_is_ru: insert_outcome.layout_is_ru,
    }
}
fn trailing_space_count(text: &str) -> usize {
    text.chars().rev().take_while(|ch| *ch == ' ').count()
}
