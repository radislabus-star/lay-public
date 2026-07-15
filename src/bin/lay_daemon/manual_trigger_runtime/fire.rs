use super::super::trigger_dispatch::{
    complete_manual_trigger, run_configured_manual_correction, run_scoped_manual_correction,
    ManualTriggerCompletion, ScopedManualCorrectionContext,
};
use super::context::ManualTriggerFireContext;
use super::ime::run_ime_manual_toggle;

pub(crate) fn fire_configured_manual_trigger(ctx: ManualTriggerFireContext<'_>) {
    if let Some(result) = run_ime_manual_toggle() {
        complete_manual_trigger_with_result(Some(result), ctx);
        return;
    }
    let correction_result = run_configured_manual_correction(
        ctx.buffer,
        ctx.device,
        ctx.virtual_kbd,
        ctx.executing,
        ctx.text_observation.clone(),
    );
    complete_manual_trigger_with_result(correction_result, ctx);
}

pub(crate) fn fire_scoped_manual_trigger(
    ctx: ManualTriggerFireContext<'_>,
    replace_words: usize,
    events_since_word_start: u32,
    reason: &str,
) {
    if let Some(result) = run_ime_manual_toggle() {
        complete_manual_trigger_with_result(Some(result), ctx);
        return;
    }
    let correction_result = run_scoped_manual_correction(
        ScopedManualCorrectionContext {
            buffer: ctx.buffer,
            device: ctx.device,
            virtual_kbd: ctx.virtual_kbd,
            executing: ctx.executing,
            text_observation: ctx.text_observation.clone(),
        },
        replace_words,
        events_since_word_start,
        reason,
    );
    complete_manual_trigger_with_result(correction_result, ctx);
}

fn complete_manual_trigger_with_result(
    correction_result: Option<bool>,
    ctx: ManualTriggerFireContext<'_>,
) {
    complete_manual_trigger(
        correction_result,
        ManualTriggerCompletion {
            current_layout_is_ru: ctx.current_layout_is_ru,
            last_layout_poll: ctx.last_layout_poll,
            suppress_next_typing_assist_after_manual_replay: ctx
                .suppress_next_typing_assist_after_manual_replay,
            pending_typing_assist_after_space: ctx.pending_typing_assist_after_space,
            shift_state: ctx.shift_state,
            dshift_state: ctx.dshift_state,
            pending_multi_tap: ctx.pending_multi_tap,
            last_double_at: ctx.last_double_at,
            clear_on_next_typing: ctx.clear_on_next_typing,
        },
    );
}
