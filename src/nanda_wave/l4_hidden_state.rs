//! L4 hidden typing-state estimator and target-independent witness resolver.
//!
//! L3 supplies context relation evidence. Exact accepted/rejected transition
//! memory supplies witness observations. L4 groups extensionally identical
//! predicted states, resolves only evidence-separated classes, and otherwise
//! returns ambiguity instead of inventing a workflow rule.

use std::collections::BTreeMap;

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
    pub(crate) rank_milli: i16,
    pub(crate) context_support: bool,
    pub(crate) eligible: bool,
    pub(crate) witness_attract: u32,
    pub(crate) witness_repel: u32,
    pub(crate) witness_state_specific: bool,
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
}

#[derive(Default)]
struct SemanticClass {
    members: Vec<usize>,
    rank_milli: i16,
    relation_class: u64,
    context_support: bool,
    witness_attract: u32,
    witness_repel: u32,
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
        class.relation_class ^= candidate.relation_class.rotate_left((index % 63) as u32);
        class.context_support |= candidate.context_support;
        if candidate.witness_state_specific {
            class.witness_attract = class
                .witness_attract
                .saturating_add(candidate.witness_attract);
            class.witness_repel = class.witness_repel.saturating_add(candidate.witness_repel);
        }
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

    let witnessed = ranked
        .iter()
        .filter(|(_, class)| class.witness_attract > class.witness_repel)
        .max_by_key(|(_, class)| class.witness_attract - class.witness_repel)
        .map(|(id, class)| (*id, class.witness_attract - class.witness_repel));
    let witness_tied = witnessed.is_some_and(|(_, strength)| {
        ranked
            .iter()
            .filter(|(_, class)| {
                class.witness_attract.saturating_sub(class.witness_repel) == strength
            })
            .count()
            > 1
    });
    let best_id = ranked[0].0;
    let runner_rank = ranked.get(1).map(|(_, class)| class.rank_milli);
    let class_margin = runner_rank
        .map(|rank| ranked[0].1.rank_milli.saturating_sub(rank))
        .unwrap_or(i16::MAX);
    let phase_resolved = ranked.len() == 1 || (ranked[0].1.context_support && class_margin > 0);
    let selected = witnessed
        .filter(|_| !witness_tied)
        .map(|(id, _)| (id, L4HiddenDisposition::Witnessed))
        .or_else(|| phase_resolved.then_some((best_id, L4HiddenDisposition::Resolved)));
    let selected_witnessed =
        selected.is_some_and(|(_, disposition)| disposition == L4HiddenDisposition::Witnessed);
    let ambiguity_authoritative = selected.is_none()
        && ranked.iter().any(|(_, class)| {
            class.relation_class != 0 || class.witness_attract > 0 || class.witness_repel > 0
        });

    for (id, class) in ranked {
        let rejected = class.witness_repel > class.witness_attract && class.witness_repel > 0;
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
            };
        }
    }
    readouts
}

pub(crate) fn predicted_state_id(operator: &str, predicted_text: &str) -> u64 {
    let normalized = predicted_text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let text = super::phase_field::hash_text(&normalized);
    let operator = super::phase_field::hash_text(operator);
    crate::stable_hash::mix64_golden(text ^ operator.rotate_left(23))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(state: u64, rank: i16) -> L4HiddenCandidateInput {
        L4HiddenCandidateInput {
            predicted_state: state,
            relation_class: state,
            rank_milli: rank,
            context_support: true,
            eligible: true,
            witness_attract: 0,
            witness_repel: 0,
            witness_state_specific: false,
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
    }

    #[test]
    fn extensionally_identical_transitions_share_one_semantic_class() {
        let readouts = estimate_hidden_typing_state(&[candidate(7, 500), candidate(7, 490)]);

        assert_eq!(readouts[0].semantic_classes, 1);
        assert_eq!(readouts[1].semantic_classes, 1);
        assert_eq!(readouts[0].disposition, L4HiddenDisposition::Resolved);
        assert_eq!(readouts[1].disposition, L4HiddenDisposition::Resolved);
    }
}
