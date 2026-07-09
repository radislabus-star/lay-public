//! L4 hidden typing-state estimator.
//!
//! This is a Kalman-like control loop in spirit: predict the current typing
//! route, update it with verifier/observation evidence, then expose confidence
//! and desync risk. The state is discrete/wave-like, so this stays a bounded
//! estimator instead of pretending to be a linear Kalman filter.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L4ObservationKind {
    KeyInput,
    SpaceBoundary,
    Backspace,
    ImeComposition,
    CandidateApply,
    CandidateReject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L4EstimatedRoute {
    Unknown,
    StableTyping,
    ActiveComposition,
    BoundaryCommit,
    DesyncRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct L4StateObservation {
    pub(crate) kind: L4ObservationKind,
    pub(crate) has_active_composition: bool,
    pub(crate) boundary_seen: bool,
    pub(crate) left_context_changed: bool,
    pub(crate) word_count_changed: bool,
    pub(crate) verifier_passed: bool,
    pub(crate) l4_negative: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct L4StateEstimate {
    pub(crate) route: L4EstimatedRoute,
    pub(crate) confidence_milli: u16,
    pub(crate) desync_risk_milli: u16,
    pub(crate) apply_allowed: bool,
}

pub(crate) struct L4StateEstimator;

impl L4StateEstimator {
    pub(crate) fn estimate(observation: L4StateObservation) -> L4StateEstimate {
        let mut confidence = predicted_confidence(observation.kind);
        let mut desync_risk = predicted_desync_risk(observation.kind);

        if observation.verifier_passed {
            confidence += 250;
            desync_risk -= 80;
        } else {
            confidence -= 120;
            desync_risk += 120;
        }

        if observation.boundary_seen {
            confidence += 80;
            desync_risk -= 40;
        }

        if observation.has_active_composition {
            confidence += 60;
            desync_risk += 40;
        }

        if observation.left_context_changed {
            if observation.verifier_passed {
                confidence -= 40;
                desync_risk += 220;
            } else {
                confidence -= 260;
                desync_risk += 460;
            }
        }

        if observation.word_count_changed && !observation.boundary_seen {
            confidence -= 120;
            desync_risk += 260;
        }

        if observation.l4_negative {
            confidence -= 300;
            desync_risk += 420;
        }

        let confidence_milli = clamp_milli(confidence);
        let desync_risk_milli = clamp_milli(desync_risk);
        let route = estimate_route(observation, desync_risk_milli);
        let apply_allowed = observation.verifier_passed
            && !observation.l4_negative
            && desync_risk_milli < 500
            && route != L4EstimatedRoute::DesyncRisk;

        L4StateEstimate {
            route,
            confidence_milli,
            desync_risk_milli,
            apply_allowed,
        }
    }
}

const fn predicted_confidence(kind: L4ObservationKind) -> i32 {
    match kind {
        L4ObservationKind::KeyInput => 480,
        L4ObservationKind::SpaceBoundary => 560,
        L4ObservationKind::Backspace => 420,
        L4ObservationKind::ImeComposition => 500,
        L4ObservationKind::CandidateApply => 520,
        L4ObservationKind::CandidateReject => 380,
    }
}

const fn predicted_desync_risk(kind: L4ObservationKind) -> i32 {
    match kind {
        L4ObservationKind::KeyInput => 120,
        L4ObservationKind::SpaceBoundary => 100,
        L4ObservationKind::Backspace => 180,
        L4ObservationKind::ImeComposition => 160,
        L4ObservationKind::CandidateApply => 140,
        L4ObservationKind::CandidateReject => 260,
    }
}

const fn estimate_route(
    observation: L4StateObservation,
    desync_risk_milli: u16,
) -> L4EstimatedRoute {
    if desync_risk_milli >= 500
        || (observation.left_context_changed && !observation.verifier_passed)
    {
        return L4EstimatedRoute::DesyncRisk;
    }
    if observation.has_active_composition {
        return L4EstimatedRoute::ActiveComposition;
    }
    if observation.boundary_seen {
        return L4EstimatedRoute::BoundaryCommit;
    }
    match observation.kind {
        L4ObservationKind::CandidateReject => L4EstimatedRoute::Unknown,
        _ => L4EstimatedRoute::StableTyping,
    }
}

const fn clamp_milli(value: i32) -> u16 {
    if value < 0 {
        0
    } else if value > 1000 {
        1000
    } else {
        value as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> L4StateObservation {
        L4StateObservation {
            kind: L4ObservationKind::CandidateApply,
            has_active_composition: false,
            boundary_seen: false,
            left_context_changed: false,
            word_count_changed: false,
            verifier_passed: true,
            l4_negative: false,
        }
    }

    #[test]
    fn verified_current_word_transition_is_stable() {
        let estimate = L4StateEstimator::estimate(base());

        assert_eq!(estimate.route, L4EstimatedRoute::StableTyping);
        assert!(estimate.apply_allowed);
        assert!(estimate.confidence_milli >= 700);
        assert!(estimate.desync_risk_milli < 100);
    }

    #[test]
    fn boundary_commit_has_own_route() {
        let estimate = L4StateEstimator::estimate(L4StateObservation {
            kind: L4ObservationKind::SpaceBoundary,
            boundary_seen: true,
            ..base()
        });

        assert_eq!(estimate.route, L4EstimatedRoute::BoundaryCommit);
        assert!(estimate.apply_allowed);
    }

    #[test]
    fn left_context_change_is_desync_risk() {
        let estimate = L4StateEstimator::estimate(L4StateObservation {
            left_context_changed: true,
            verifier_passed: false,
            ..base()
        });

        assert_eq!(estimate.route, L4EstimatedRoute::DesyncRisk);
        assert!(!estimate.apply_allowed);
        assert!(estimate.desync_risk_milli >= 500);
    }

    #[test]
    fn verified_left_context_transition_is_allowed_with_risk() {
        let estimate = L4StateEstimator::estimate(L4StateObservation {
            left_context_changed: true,
            verifier_passed: true,
            ..base()
        });

        assert_ne!(estimate.route, L4EstimatedRoute::DesyncRisk);
        assert!(estimate.apply_allowed);
        assert!(estimate.desync_risk_milli < 500);
    }

    #[test]
    fn negative_l4_memory_blocks_apply() {
        let estimate = L4StateEstimator::estimate(L4StateObservation {
            l4_negative: true,
            ..base()
        });

        assert!(!estimate.apply_allowed);
        assert!(estimate.desync_risk_milli >= 450);
    }
}
