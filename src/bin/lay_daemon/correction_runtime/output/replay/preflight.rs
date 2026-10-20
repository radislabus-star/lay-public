use super::super::super::super::{
    active_layout_backend, active_text_backend, focus_guard, layout_controller,
    verify_target_layout_ready_for_replay, LayoutCapabilityPreflight,
};
use super::super::context::ManualOutputCommon;
use lay::desktop::LayoutBackend;

pub(super) fn preflight_manual_replay(ctx: &ManualOutputCommon<'_>) -> Result<(), String> {
    let mut mutation_preflight = ctx.text_observation.explicit_manual_preflight(
        ctx.buf,
        ctx.mapped_orig.to_string(),
        ctx.input_isolated,
    );
    mutation_preflight.validate_current()?;
    // Configured-backend replay has the same decoder requirement as exact-tail
    // replay. Changing only the KDE group leaves the focused Lay engine in its
    // old mode, so the replay can silently reinsert the original word. Reuse
    // the existing pinned layout transaction; this does not select a text route
    // or grant a committed-tail lease to the configured-backend executor.
    let kde_handoff = if active_layout_backend() == LayoutBackend::Kde
        && active_text_backend().should_try_ime()
    {
        let window = focus_guard::capture_exact_focused_window_identity()?;
        Some(
            layout_controller::prepare_kde_layout_once_for_manual_replay(
                ctx.target_is_ru,
                &window,
            )?,
        )
    } else {
        None
    };
    let layout_preflight = if kde_handoff.is_none() {
        Some(LayoutCapabilityPreflight::run(
            None,
            std::iter::once(ctx.target_is_ru),
            "manual replay",
        )?)
    } else {
        None
    };
    let result = (|| {
        verify_target_layout_ready_for_replay(ctx.target_is_ru)?;
        if let Some(lease) = &ctx.delegated_tail_lease {
            lease.validate_after_controlled_layout_handoff(ctx.n_backspaces, ctx.target_is_ru)?;
        }
        mutation_preflight.consume()
    })();
    if result.is_err() {
        if let Some(handoff) = &kde_handoff {
            handoff.restore_initial_best_effort("manual replay");
        } else if let Some(layout_preflight) = &layout_preflight {
            layout_preflight.restore_initial_best_effort("manual replay");
        }
    }
    result
}
