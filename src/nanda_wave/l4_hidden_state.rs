//! L4 hidden typing-state estimator and target-independent witness resolver.
//!
//! L3 supplies context relation evidence. Exact accepted/rejected transition
//! memory supplies witness observations. L4 groups extensionally identical
//! predicted states, resolves only evidence-separated classes, and otherwise
//! returns ambiguity instead of inventing a workflow rule.

use std::collections::BTreeMap;

use super::l4_active_disambiguation::{
    resolve_active_hypotheses, L4ActiveHypothesis, L4ActiveResolutionStatus,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum L4HiddenDisposition {
    Resolved,
    Witnessed,
    Ambiguous,
    Rejected,
    #[default]
    Unobserved,
}

impl L4HiddenDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Witnessed => "witnessed",
            Self::Ambiguous => "ambiguous",
            Self::Rejected => "rejected",
            Self::Unobserved => "unobserved",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct L4HiddenCandidateInput {
    pub(crate) predicted_state: u64,
    pub(crate) relation_class: u64,
    pub(crate) operator_class: u64,
    pub(crate) verifier_passed: bool,
    pub(crate) rank_milli: i16,
    pub(crate) context_support: bool,
    pub(crate) pairwise_context_witness: bool,
    pub(crate) eligible: bool,
    pub(crate) witness_attract: u32,
    pub(crate) witness_repel: u32,
    pub(crate) witness_state_specific: bool,
    pub(crate) phase_witness_milli: i16,
    pub(crate) phase_witness_supported: bool,
    pub(crate) operator_consensus_witness: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct L4HiddenStateReadout {
    pub(crate) disposition: L4HiddenDisposition,
    pub(crate) semantic_classes: u16,
    pub(crate) unresolved_classes: u16,
    pub(crate) selected_class: u64,
    pub(crate) relation_class: u64,
    pub(crate) class_margin_milli: i16,
    pub(crate) witness_count: u32,
    pub(crate) ambiguity_authoritative: bool,
    pub(crate) selected_witnessed: bool,
    pub(crate) witness_plan_commitment: u64,
    pub(crate) witness_receipts: u8,
    pub(crate) witness_probe: &'static str,
    pub(crate) certificate_valid: bool,
}

#[derive(Default)]
struct SemanticClass {
    members: Vec<usize>,
    rank_milli: i16,
    relation_class: u64,
    operator_class: u64,
    verifier_passed: bool,
    context_support: bool,
    pairwise_context_witness: bool,
    witness_attract: u32,
    witness_repel: u32,
    witness_state_specific: bool,
    phase_witness_milli: i16,
    phase_witness_supported: bool,
    operator_consensus_witness: bool,
}

pub(crate) fn estimate_hidden_typing_state(
    candidates: &[L4HiddenCandidateInput],
) -> Vec<L4HiddenStateReadout> {
    let mut classes = BTreeMap::<u64, SemanticClass>::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if !candidate.eligible {
            continue;
        }
        let class = classes.entry(candidate.predicted_state).or_default();
        class.members.push(index);
        class.rank_milli = class.rank_milli.max(candidate.rank_milli);
        class.relation_class = merge_identity(class.relation_class, candidate.relation_class);
        class.operator_class = merge_identity(class.operator_class, candidate.operator_class);
        class.verifier_passed |= candidate.verifier_passed;
        class.context_support |= candidate.context_support;
        class.pairwise_context_witness |= candidate.pairwise_context_witness;
        if candidate.witness_state_specific {
            class.witness_state_specific = true;
            class.witness_attract = class
                .witness_attract
                .saturating_add(candidate.witness_attract);
            class.witness_repel = class.witness_repel.saturating_add(candidate.witness_repel);
        }
        if candidate.phase_witness_supported {
            class.phase_witness_milli = merge_phase_margin(
                class.phase_witness_milli,
                candidate.phase_witness_milli,
                class.phase_witness_supported,
            );
            class.phase_witness_supported = true;
        }
        class.operator_consensus_witness |= candidate.operator_consensus_witness;
    }
    let mut ranked = classes
        .iter()
        .map(|(id, class)| (*id, class))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .rank_milli
            .cmp(&left.1.rank_milli)
            .then_with(|| left.0.cmp(&right.0))
    });
    let class_count = ranked.len().min(u16::MAX as usize) as u16;
    let mut readouts = vec![L4HiddenStateReadout::default(); candidates.len()];
    if ranked.is_empty() {
        return readouts;
    }

    let best_id = ranked[0].0;
    let runner_rank = ranked.get(1).map(|(_, class)| class.rank_milli);
    let class_margin = runner_rank
        .map(|rank| ranked[0].1.rank_milli.saturating_sub(rank))
        .unwrap_or(i16::MAX);
    let hypotheses = ranked
        .iter()
        .map(|(id, class)| L4ActiveHypothesis {
            class_id: *id,
            relation_class: class.relation_class,
            operator_class: class.operator_class,
            verifier_passed: class.verifier_passed,
            context_support: class.context_support,
            pairwise_context_witness: class.pairwise_context_witness,
            witness_attract: class.witness_attract,
            witness_repel: class.witness_repel,
            witness_state_specific: class.witness_state_specific,
            phase_witness_milli: class.phase_witness_milli,
            phase_witness_supported: class.phase_witness_supported,
            operator_consensus_witness: class.operator_consensus_witness,
        })
        .collect::<Vec<_>>();
    let certificate = resolve_active_hypotheses(&hypotheses);
    let selected = certificate.selected_class.and_then(|selected_id| {
        let selected_class = classes.get(&selected_id)?;
        if class_is_rejected(selected_class) {
            return None;
        }
        let disposition = match certificate.status {
            L4ActiveResolutionStatus::Unique => L4HiddenDisposition::Resolved,
            L4ActiveResolutionStatus::Witnessed => L4HiddenDisposition::Witnessed,
            L4ActiveResolutionStatus::Ambiguous => return None,
        };
        Some((selected_id, disposition))
    });
    let selected_witnessed =
        selected.is_some_and(|(_, disposition)| disposition == L4HiddenDisposition::Witnessed);
    // A relation id is present for every candidate: it is an L3/L4 addressing
    // key, not evidence learned about this transition. Only a state-specific
    // witness, phase witness, or operator witness may turn unresolved classes
    // into a blocking ambiguity. Otherwise L4 remains advisory and the typed
    // verifier plus field decides the local transition.
    let ambiguity_authoritative = certificate.certificate_valid
        && selected.is_none()
        && ranked.iter().any(|(_, class)| {
            class.witness_state_specific
                || class.phase_witness_supported
                || class.pairwise_context_witness
                || class.operator_consensus_witness
        });

    for (id, class) in ranked {
        let rejected = class_is_rejected(class);
        let disposition = if rejected {
            L4HiddenDisposition::Rejected
        } else if let Some((selected_id, selected_disposition)) = selected {
            if selected_id == id {
                selected_disposition
            } else {
                L4HiddenDisposition::Unobserved
            }
        } else if selected.is_none() {
            L4HiddenDisposition::Ambiguous
        } else {
            L4HiddenDisposition::Unobserved
        };
        for &member in &class.members {
            readouts[member] = L4HiddenStateReadout {
                disposition,
                semantic_classes: class_count,
                unresolved_classes: if selected.is_some() {
                    0
                } else {
                    class_count.saturating_sub(1)
                },
                selected_class: selected.map(|(id, _)| id).unwrap_or_default(),
                relation_class: class.relation_class,
                class_margin_milli: if id == best_id {
                    class_margin
                } else {
                    -class_margin
                },
                witness_count: class.witness_attract.saturating_add(class.witness_repel),
                ambiguity_authoritative,
                selected_witnessed,
                witness_plan_commitment: certificate.plan.plan_commitment,
                witness_receipts: certificate.receipts.len().min(u8::MAX as usize) as u8,
                witness_probe: certificate
                    .receipts
                    .iter()
                    .rev()
                    .find(|receipt| receipt.observed_outcome.is_some())
                    .map(|receipt| receipt.probe.as_str())
                    .unwrap_or("none"),
                certificate_valid: certificate.certificate_valid,
            };
        }
    }
    readouts
}

fn class_is_rejected(class: &SemanticClass) -> bool {
    class.witness_state_specific
        && class.witness_repel > class.witness_attract
        && class.witness_repel > 0
}

fn merge_identity(current: u64, incoming: u64) -> u64 {
    if current == 0 {
        incoming
    } else if incoming == 0 || current == incoming {
        current
    } else {
        crate::stable_hash::mix64_golden(current ^ incoming)
    }
}

fn merge_phase_margin(current: i16, incoming: i16, current_present: bool) -> i16 {
    if !current_present
        || incoming.abs() > current.abs()
        || (incoming.abs() == current.abs() && incoming < current)
    {
        incoming
    } else {
        current
    }
}

pub(crate) fn predicted_state_id(state_before: u64, operator: &str, predicted_text: &str) -> u64 {
    let normalized = predicted_text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let text = super::phase_field::hash_text(&normalized);
    let operator = super::phase_field::hash_text(operator);
    crate::stable_hash::mix64_golden(state_before.rotate_left(7) ^ text ^ operator.rotate_left(23))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(state: u64, rank: i16) -> L4HiddenCandidateInput {
        L4HiddenCandidateInput {
            predicted_state: state,
            relation_class: state,
            operator_class: 7,
            verifier_passed: true,
            rank_milli: rank,
            context_support: true,
            pairwise_context_witness: false,
            eligible: true,
            witness_attract: 0,
            witness_repel: 0,
            witness_state_specific: false,
            phase_witness_milli: 0,
            phase_witness_supported: false,
            operator_consensus_witness: false,
        }
    }

    #[test]
    fn unresolved_semantic_classes_abstain_instead_of_tie_breaking() {
        let mut first = candidate(1, 500);
        let mut second = candidate(2, 500);
        first.context_support = false;
        second.context_support = false;
        let readouts = estimate_hidden_typing_state(&[first, second]);

        assert!(readouts
            .iter()
            .all(|readout| readout.disposition == L4HiddenDisposition::Ambiguous));
        assert!(readouts
            .iter()
            .all(|readout| !readout.ambiguity_authoritative));
    }

    #[test]
    fn target_independent_transition_witness_resolves_class() {
        let mut first = candidate(1, 500);
        let mut second = candidate(2, 499);
        second.witness_state_specific = true;
        second.witness_attract = 3;
        first.witness_state_specific = true;
        first.witness_repel = 2;
        let readouts = estimate_hidden_typing_state(&[first, second]);

        assert_eq!(readouts[0].disposition, L4HiddenDisposition::Rejected);
        assert_eq!(readouts[1].disposition, L4HiddenDisposition::Witnessed);
        assert!(readouts[1].certificate_valid);
        assert_eq!(readouts[1].witness_probe, "transition_history");
    }

    #[test]
    fn extensionally_identical_transitions_share_one_semantic_class() {
        let readouts = estimate_hidden_typing_state(&[candidate(7, 500), candidate(7, 490)]);

        assert_eq!(readouts[0].semantic_classes, 1);
        assert_eq!(readouts[1].semantic_classes, 1);
        assert_eq!(readouts[0].disposition, L4HiddenDisposition::Resolved);
        assert_eq!(readouts[1].disposition, L4HiddenDisposition::Resolved);
        assert!(readouts[0].certificate_valid);
    }

    #[test]
    fn learned_phase_anti_center_makes_competing_states_authoritatively_ambiguous() {
        let mut first = candidate(9, 700);
        let second = candidate(10, 690);
        first.phase_witness_supported = true;
        first.phase_witness_milli = -380;

        let readouts = estimate_hidden_typing_state(&[first, second]);

        assert!(readouts
            .iter()
            .all(|readout| readout.disposition == L4HiddenDisposition::Ambiguous));
        assert!(readouts[0].ambiguity_authoritative);
        assert!(readouts[0].certificate_valid);
    }
}
