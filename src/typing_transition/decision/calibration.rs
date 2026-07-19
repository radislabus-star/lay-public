#[derive(Debug, Clone, Copy)]
pub(super) struct AdmissionCalibration {
    pub(super) l3_strong_milli: i16,
    pub(super) l4_strong_milli: i16,
    pub(super) learned_prior_floor: f32,
    pub(super) high_risk_floor: f32,
    pub(super) transition_posterior_floor: f32,
    pub(super) composite_margin_floor: f32,
    pub(super) l2_peak_milli: i16,
    pub(super) l2_peak_uncertainty_milli: i16,
    pub(super) structural_preservation_gain_milli: i16,
    pub(super) structural_loss_reduction_milli: i16,
    pub(super) structural_rank_proximity: f32,
    pub(super) l2_competitor_gap_milli: i16,
    pub(super) phase_competitor_gap_milli: i16,
}

/// One calibration owner for learned signal admission.
///
/// These values preserve the currently released behavior. Safety invariants
/// remain structural and do not live in this profile; future replay/eval can
/// replace this profile without editing candidate-specific branches.
pub(super) const CURRENT: AdmissionCalibration = AdmissionCalibration {
    l3_strong_milli: 420,
    l4_strong_milli: 120,
    learned_prior_floor: 0.080,
    high_risk_floor: 0.62,
    transition_posterior_floor: 0.20,
    composite_margin_floor: 0.08,
    l2_peak_milli: 650,
    l2_peak_uncertainty_milli: 450,
    structural_preservation_gain_milli: 120,
    structural_loss_reduction_milli: 120,
    structural_rank_proximity: 0.08,
    l2_competitor_gap_milli: 100,
    phase_competitor_gap_milli: 10,
};

/// Context support is a calibrated signal, never a structural verifier result.
pub(super) fn known_word_context_state_support(
    context_prior: f32,
    l3_phrase_milli: i16,
    l4_signed_milli: i16,
) -> bool {
    context_prior >= CURRENT.learned_prior_floor
        || l3_phrase_milli >= CURRENT.l3_strong_milli
        || l4_signed_milli >= CURRENT.l4_strong_milli
}

pub(super) fn known_word_drift_has_authority(
    strong_state_support: bool,
    exact_state_support: bool,
) -> bool {
    exact_state_support || strong_state_support
}
