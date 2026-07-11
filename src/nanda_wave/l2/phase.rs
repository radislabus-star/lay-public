//! L2 phase-center evidence bridge.
//!
//! The phase package is compact learned field evidence. This producer bridge
//! only annotates candidates; DecisionCore owns all ranking and admission.

use super::{WaveOptions, WordCandidate, L2_SURFACE_COMPLETION_CELL};

pub(super) fn apply_l2_phase_shadow(
    original: &str,
    candidates: &mut [WordCandidate],
    options: &WaveOptions,
) {
    if !options.l2_phase_shadow() {
        return;
    }
    for candidate in candidates.iter_mut() {
        let operation = l2_phase_operation(candidate.source);
        let (loaded, margin_micro, admitted) =
            super::super::l2_candidate_phase_shadow(original, &candidate.text, operation);
        candidate.support.push(format!(
            "l2-phase:loaded={} margin={} admitted={}",
            loaded, margin_micro, admitted
        ));
    }
    if options.l2_phase_apply() {
        for candidate in candidates.iter_mut() {
            candidate
                .support
                .push("l2-phase:apply-deferred-to-decision-core".to_string());
        }
    }
}

fn l2_phase_operation(source: &str) -> &'static str {
    match source {
        "LayoutWordCell32" | "LearnedMemoryCell32" => "layout",
        "BoundaryCell32" | "PhraseMemoryCell32" => "split",
        L2_SURFACE_COMPLETION_CELL => "completion",
        _ => "typo",
    }
}
