use super::super::super::super::{
    verify_target_layout_ready_for_replay, LayoutCapabilityPreflight,
};
use super::super::context::ManualOutputCommon;

pub(super) fn preflight_manual_replay(ctx: &ManualOutputCommon<'_>) -> Result<(), String> {
    let mut mutation_preflight = ctx.text_observation.explicit_manual_preflight(
        ctx.buf,
        ctx.mapped_orig.to_string(),
        ctx.input_isolated,
    );
    mutation_preflight.validate_current()?;
    let layout_preflight =
        LayoutCapabilityPreflight::run(None, std::iter::once(ctx.target_is_ru), "manual replay")?;
    if let Err(error) = verify_target_layout_ready_for_replay(ctx.target_is_ru) {
        layout_preflight.restore_initial_best_effort("manual replay readiness failure");
        return Err(error);
    }
    if let Err(error) = mutation_preflight.consume() {
        layout_preflight.restore_initial_best_effort("manual replay");
        return Err(error);
    }
    Ok(())
}
