//! L2 phase-center evidence bridge.
//!
//! The phase package is compact learned field evidence. It annotates and may
//! suppress candidates, but it never mutates text or bypasses the transition
//! verifier.

use super::{WaveOptions, WordCandidate, L2_SURFACE_COMPLETION_CELL, L2_SURFACE_MOTIF_CELL};

pub(super) fn apply_l2_phase_shadow(
    original: &str,
    candidates: &mut Vec<WordCandidate>,
    options: &WaveOptions,
) {
    if !options.l2_phase_shadow() {
        return;
    }
    let mut package_loaded_any = false;
    for candidate in candidates.iter_mut() {
        let operation = l2_phase_operation(candidate.source);
        let (loaded, margin_micro, admitted) =
            super::super::l2_candidate_phase_shadow(original, &candidate.text, operation);
        package_loaded_any |= loaded;
        candidate.support.push(format!(
            "l2-phase:loaded={} margin={} admitted={}",
            loaded, margin_micro, admitted
        ));
    }
    if !options.l2_phase_apply() || !package_loaded_any {
        return;
    }
    for candidate in candidates.iter_mut() {
        if candidate_has_l2_phase_admission(candidate) {
            candidate.energy = (candidate.energy + 0.025).min(1.0);
        }
    }
    candidates.retain(|candidate| {
        candidate_has_l2_phase_admission(candidate) || !l2_phase_apply_source(candidate.source)
    });
}

fn candidate_has_l2_phase_admission(candidate: &WordCandidate) -> bool {
    candidate
        .support
        .iter()
        .any(|item| item.contains("l2-phase:loaded=true") && item.contains("admitted=true"))
}

fn l2_phase_operation(source: &str) -> &'static str {
    match source {
        "LayoutWordCell32" | "LearnedMemoryCell32" => "layout",
        "BoundaryCell32" | "PhraseMemoryCell32" => "split",
        L2_SURFACE_COMPLETION_CELL => "completion",
        _ => "typo",
    }
}

fn l2_phase_apply_source(source: &str) -> bool {
    matches!(
        source,
        L2_SURFACE_MOTIF_CELL
            | L2_SURFACE_COMPLETION_CELL
            | "CommonRuFixCell32"
            | "PhraseCell32"
            | "GrammarCell32"
    )
}
