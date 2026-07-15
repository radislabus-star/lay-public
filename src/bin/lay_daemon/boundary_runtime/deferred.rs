use std::time::Instant;

use super::super::{
    active_typing_assist, apply_prepared_typing_assist_after_space, lock_virtual_keyboard, log,
    pending_typing_assist::PendingTypingAssist, should_run_deferred_typing_assist_after_space,
    typing_assist_worker::WorkerPoll, DaemonTextObservation, TypingAssistOutcome,
};

#[path = "deferred/context.rs"]
mod context;
pub(crate) use context::DeferredTypingAssistContext;

pub(crate) fn try_handle_deferred_typing_assist(ctx: DeferredTypingAssistContext<'_>) -> bool {
    if !should_run_deferred_typing_assist_after_space(
        ctx.pending_typing_assist_after_space.is_some(),
        active_typing_assist(),
        ctx.shift_state.any(),
    ) {
        return false;
    }

    let Some(pending) = ctx.pending_typing_assist_after_space.as_mut() else {
        return false;
    };
    if let Some(request_id) = pending.request_id() {
        match ctx.typing_assist_worker.poll(request_id) {
            WorkerPoll::Pending => return false,
            WorkerPoll::Completed(Some(correction)) => pending.resolve(*correction),
            WorkerPoll::Completed(None) => {
                ctx.pending_typing_assist_after_space.take();
                return false;
            }
        }
    }
    if !pending.ready_to_apply() {
        return false;
    }
    let Some(pending) = ctx.pending_typing_assist_after_space.take() else {
        return false;
    };
    let Some((correction, cursor_offset, expected_text_context)) = pending.into_parts() else {
        return false;
    };
    let retry_correction = correction.clone();
    let retry_text_context = expected_text_context.clone();
    let mut g = lock_virtual_keyboard(ctx.virtual_kbd);
    let outcome = apply_prepared_typing_assist_after_space(
        ctx.buffer,
        g.as_mut(),
        Some(ctx.device),
        ctx.executing,
        cursor_offset,
        correction,
        DaemonTextObservation::new(expected_text_context, ctx.text_observer),
    );
    match outcome {
        TypingAssistOutcome::Applied { layout_is_ru } => {
            *ctx.current_layout_is_ru = layout_is_ru;
            *ctx.last_layout_poll = Instant::now();
        }
        TypingAssistOutcome::Deferred => {
            *ctx.pending_typing_assist_after_space = Some(PendingTypingAssist::with_cursor_offset(
                retry_correction,
                cursor_offset,
                retry_text_context,
            ));
            log("· typing-assist deferred again: prepared edit still complex");
        }
        TypingAssistOutcome::NoCorrection => {}
    }
    true
}

#[cfg(test)]
mod route_contract {
    #[test]
    fn focused_ime_does_not_drop_verified_deferred_boundary_edit() {
        let source = include_str!("deferred.rs");
        let forbidden_ime_owner = ["focused_ime_engine", "_handles_typing"].concat();

        assert!(source.contains("apply_prepared_typing_assist_after_space("));
        assert!(!source.contains(&forbidden_ime_owner));
    }
}
