use super::super::multi_tap_scope_for_taps;
use super::context::{ManualTriggerFireContext, PendingMultiTapTimeoutContext};
use super::fire::fire_scoped_manual_trigger;

pub(crate) fn fire_expired_pending_multi_tap(ctx: PendingMultiTapTimeoutContext<'_>) {
    if !ctx
        .pending_multi_tap
        .as_ref()
        .is_some_and(|pending| pending.last_release.elapsed() >= ctx.shift_window)
    {
        return;
    }

    if let Some(pending) = ctx.pending_multi_tap.take() {
        let replace_words = multi_tap_scope_for_taps(pending.tap_count).unwrap_or(1);
        fire_scoped_manual_trigger(
            ManualTriggerFireContext {
                buffer: ctx.buffer,
                device: ctx.device,
                virtual_kbd: ctx.virtual_kbd,
                executing: ctx.executing,
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
            replace_words,
            ctx.events_since_word_start,
            "multi-tap timeout",
        );
    }
}
