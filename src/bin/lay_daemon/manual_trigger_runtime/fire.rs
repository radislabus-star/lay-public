use super::super::trigger_dispatch::{
    complete_manual_trigger_from_loop, run_configured_manual_correction,
    run_scoped_manual_correction,
};
use super::context::ManualTriggerFireContext;

pub(crate) fn fire_configured_manual_trigger(ctx: ManualTriggerFireContext<'_>) {
    let correction_result =
        run_configured_manual_correction(ctx.buffer, ctx.device, ctx.virtual_kbd, ctx.executing);
    complete_manual_trigger_with_result(correction_result, ctx);
}

pub(crate) fn fire_scoped_manual_trigger(
    ctx: ManualTriggerFireContext<'_>,
    replace_words: usize,
    events_since_word_start: u32,
    reason: &str,
) {
    let correction_result = run_scoped_manual_correction(
        ctx.buffer,
        replace_words,
        ctx.device,
        ctx.virtual_kbd,
        ctx.executing,
        events_since_word_start,
        reason,
    );
    complete_manual_trigger_with_result(correction_result, ctx);
}

fn complete_manual_trigger_with_result(
    correction_result: Option<bool>,
    ctx: ManualTriggerFireContext<'_>,
) {
    complete_manual_trigger_from_loop(
        correction_result,
        ctx.current_layout_is_ru,
        ctx.last_layout_poll,
        ctx.suppress_next_typing_assist_after_manual_replay,
        ctx.shift_state,
        ctx.dshift_state,
        ctx.pending_multi_tap,
        ctx.last_double_at,
        ctx.clear_on_next_typing,
    );
}
