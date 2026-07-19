//! Phase-package evidence for the candidate lattice.
//!
//! This module only states whether a promoted L2 transition package supports a
//! source family. Ranking and final apply policy remain outside this owner.

use super::TransitionDecisionPolicy;
use crate::candidate_contract::CorrectionSourceRole;

pub(super) fn phase_policy_rejection(
    policy: TransitionDecisionPolicy,
    source_role: CorrectionSourceRole,
    package_loaded: bool,
    operator_present: bool,
    operator_promoted: bool,
    verdict: crate::nanda_wave::PhaseVerdict,
) -> Option<&'static str> {
    if !policy.l2_phase_apply || !phase_managed_source(source_role) {
        return None;
    }
    if !package_loaded {
        return Some("l2_transition_phase_package_missing");
    }
    if !operator_present {
        return Some("l2_transition_phase_operator_missing");
    }
    if !operator_promoted {
        return Some("l2_transition_phase_shadow_only");
    }
    match verdict {
        crate::nanda_wave::PhaseVerdict::Repel => Some("l2_transition_phase_repel"),
        crate::nanda_wave::PhaseVerdict::Unknown => Some("l2_transition_phase_unknown"),
        crate::nanda_wave::PhaseVerdict::Support => None,
    }
}

fn phase_managed_source(source_role: CorrectionSourceRole) -> bool {
    matches!(
        source_role,
        CorrectionSourceRole::DeterministicTypo
            | CorrectionSourceRole::L2Surface
            | CorrectionSourceRole::L3Context
    )
}
