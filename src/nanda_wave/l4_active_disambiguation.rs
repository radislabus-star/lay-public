//! Target-independent active disambiguation for L4 typing-state hypotheses.
//!
//! The plan is committed before witness evidence is inspected. A witness may
//! remove hypotheses, but it never grants edit authority; DecisionCore and the
//! transition verifier retain that responsibility.

use std::collections::{BTreeMap, BTreeSet};

const MAX_WITNESSES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum L4WitnessProbe {
    TransitionHistory = 1,
    ContextRelation = 2,
    VerifierResult = 3,
    PhaseRelation = 4,
}

impl L4WitnessProbe {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::TransitionHistory => "transition_history",
            Self::ContextRelation => "context_relation",
            Self::VerifierResult => "verifier_result",
            Self::PhaseRelation => "phase_relation",
        }
    }

    const ALL: [Self; 4] = [
        Self::TransitionHistory,
        Self::ContextRelation,
        Self::VerifierResult,
        Self::PhaseRelation,
    ];
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct L4ActiveHypothesis {
    pub(crate) class_id: u64,
    pub(crate) relation_class: u64,
    pub(crate) operator_class: u64,
    pub(crate) verifier_passed: bool,
    pub(crate) context_support: bool,
    pub(crate) witness_attract: u32,
    pub(crate) witness_repel: u32,
    pub(crate) witness_state_specific: bool,
    pub(crate) phase_witness_milli: i16,
    pub(crate) phase_witness_supported: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct L4WitnessPlan {
    pub(crate) candidate_commitment: u64,
    pub(crate) plan_commitment: u64,
    probes: Vec<L4WitnessProbe>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct L4WitnessReceipt {
    pub(crate) probe: L4WitnessProbe,
    pub(crate) observed_outcome: Option<u64>,
    pub(crate) classes_before: Vec<u64>,
    pub(crate) classes_after: Vec<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum L4ActiveResolutionStatus {
    Unique,
    Witnessed,
    #[default]
    Ambiguous,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct L4ResolutionCertificate {
    pub(crate) plan: L4WitnessPlan,
    pub(crate) receipts: Vec<L4WitnessReceipt>,
    pub(crate) selected_class: Option<u64>,
    pub(crate) unresolved_classes: u16,
    pub(crate) status: L4ActiveResolutionStatus,
    pub(crate) certificate_valid: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct HypothesisClass {
    relation_class: u64,
    operator_class: u64,
    verifier_passed: bool,
    context_support: bool,
    witness_attract: u32,
    witness_repel: u32,
    witness_state_specific: bool,
    phase_witness_milli: i16,
    phase_witness_supported: bool,
}

pub(crate) fn resolve_active_hypotheses(
    hypotheses: &[L4ActiveHypothesis],
) -> L4ResolutionCertificate {
    let classes = normalize_classes(hypotheses);
    let plan = prepare_witness_plan(&classes);
    let mut active = classes.keys().copied().collect::<Vec<_>>();
    let initial_len = active.len();
    let mut receipts = Vec::new();
    let mut used = BTreeSet::new();

    while active.len() > 1 && receipts.len() < MAX_WITNESSES {
        let Some(probe) = select_witness(&active, &classes, &plan.probes, &used) else {
            break;
        };
        used.insert(probe);
        let before = active.clone();
        let observed_outcome = observe_probe(probe, &active, &classes);
        if let Some(observed) = observed_outcome {
            active.retain(|class_id| {
                classes
                    .get(class_id)
                    .is_some_and(|class| predicted_outcome(probe, *class_id, class) == observed)
            });
        }
        receipts.push(L4WitnessReceipt {
            probe,
            observed_outcome,
            classes_before: before,
            classes_after: active.clone(),
        });
    }

    let selected_class = (active.len() == 1).then(|| active[0]);
    let status = if initial_len <= 1 {
        L4ActiveResolutionStatus::Unique
    } else if selected_class.is_some()
        && receipts
            .iter()
            .any(|receipt| receipt.observed_outcome.is_some())
    {
        L4ActiveResolutionStatus::Witnessed
    } else {
        L4ActiveResolutionStatus::Ambiguous
    };
    let unresolved_classes = if selected_class.is_some() {
        0
    } else {
        active.len().saturating_sub(1).min(u16::MAX as usize) as u16
    };
    let mut certificate = L4ResolutionCertificate {
        plan,
        receipts,
        selected_class,
        unresolved_classes,
        status,
        certificate_valid: false,
    };
    certificate.certificate_valid = verify_resolution_certificate(hypotheses, &certificate);
    certificate
}

pub(crate) fn verify_resolution_certificate(
    hypotheses: &[L4ActiveHypothesis],
    certificate: &L4ResolutionCertificate,
) -> bool {
    let classes = normalize_classes(hypotheses);
    let plan = prepare_witness_plan(&classes);
    if certificate.plan != plan || certificate.receipts.len() > MAX_WITNESSES {
        return false;
    }
    let mut active = classes.keys().copied().collect::<Vec<_>>();
    let initial_len = active.len();
    let mut used = BTreeSet::new();
    for receipt in &certificate.receipts {
        if receipt.classes_before != active {
            return false;
        }
        let selected = select_witness(&active, &classes, &plan.probes, &used);
        if selected != Some(receipt.probe) {
            return false;
        }
        used.insert(receipt.probe);
        let observed = observe_probe(receipt.probe, &active, &classes);
        if observed != receipt.observed_outcome {
            return false;
        }
        if let Some(observed) = observed {
            active.retain(|class_id| {
                classes.get(class_id).is_some_and(|class| {
                    predicted_outcome(receipt.probe, *class_id, class) == observed
                })
            });
        }
        if receipt.classes_after != active {
            return false;
        }
    }

    let selected = (active.len() == 1).then(|| active[0]);
    let status = if initial_len <= 1 {
        L4ActiveResolutionStatus::Unique
    } else if selected.is_some()
        && certificate
            .receipts
            .iter()
            .any(|receipt| receipt.observed_outcome.is_some())
    {
        L4ActiveResolutionStatus::Witnessed
    } else {
        L4ActiveResolutionStatus::Ambiguous
    };
    certificate.selected_class == selected
        && certificate.status == status
        && certificate.unresolved_classes
            == if selected.is_some() {
                0
            } else {
                active.len().saturating_sub(1).min(u16::MAX as usize) as u16
            }
}

fn normalize_classes(hypotheses: &[L4ActiveHypothesis]) -> BTreeMap<u64, HypothesisClass> {
    let mut classes = BTreeMap::<u64, HypothesisClass>::new();
    for hypothesis in hypotheses {
        let class = classes.entry(hypothesis.class_id).or_default();
        class.relation_class = merge_identity(class.relation_class, hypothesis.relation_class);
        class.operator_class = merge_identity(class.operator_class, hypothesis.operator_class);
        class.verifier_passed |= hypothesis.verifier_passed;
        class.context_support |= hypothesis.context_support;
        if hypothesis.witness_state_specific {
            class.witness_state_specific = true;
            class.witness_attract = class
                .witness_attract
                .saturating_add(hypothesis.witness_attract);
            class.witness_repel = class.witness_repel.saturating_add(hypothesis.witness_repel);
        }
        if hypothesis.phase_witness_supported {
            class.phase_witness_milli = merge_phase_margin(
                class.phase_witness_milli,
                hypothesis.phase_witness_milli,
                class.phase_witness_supported,
            );
            class.phase_witness_supported = true;
        }
    }
    classes
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

fn prepare_witness_plan(classes: &BTreeMap<u64, HypothesisClass>) -> L4WitnessPlan {
    let candidate_commitment = classes
        .iter()
        .fold(0x4c34_4341_4e44_4944, |hash, (id, class)| {
            let shape =
                id ^ class.relation_class.rotate_left(11) ^ class.operator_class.rotate_left(23);
            crate::stable_hash::mix64_golden(hash ^ shape)
        });
    let probes = L4WitnessProbe::ALL.to_vec();
    let plan_commitment = probes.iter().fold(candidate_commitment, |hash, probe| {
        crate::stable_hash::mix64_golden(hash ^ u64::from(*probe as u8))
    });
    L4WitnessPlan {
        candidate_commitment,
        plan_commitment,
        probes,
    }
}

fn select_witness(
    active: &[u64],
    classes: &BTreeMap<u64, HypothesisClass>,
    probes: &[L4WitnessProbe],
    used: &BTreeSet<L4WitnessProbe>,
) -> Option<L4WitnessProbe> {
    let mut best = None::<(usize, std::cmp::Reverse<usize>, u8, L4WitnessProbe)>;
    for probe in probes.iter().copied() {
        if used.contains(&probe) {
            continue;
        }
        let mut buckets = BTreeMap::<u64, usize>::new();
        for class_id in active {
            let Some(class) = classes.get(class_id) else {
                continue;
            };
            *buckets
                .entry(predicted_outcome(probe, *class_id, class))
                .or_default() += 1;
        }
        if buckets.len() < 2 {
            continue;
        }
        let largest = buckets.values().copied().max().unwrap_or(usize::MAX);
        let score = (
            largest,
            std::cmp::Reverse(buckets.len()),
            probe as u8,
            probe,
        );
        if best.as_ref().map_or(true, |current| score < *current) {
            best = Some(score);
        }
    }
    best.map(|entry| entry.3)
}

fn predicted_outcome(probe: L4WitnessProbe, class_id: u64, class: &HypothesisClass) -> u64 {
    match probe {
        L4WitnessProbe::TransitionHistory => class_id,
        L4WitnessProbe::ContextRelation => class.relation_class,
        L4WitnessProbe::VerifierResult => u64::from(class.verifier_passed),
        L4WitnessProbe::PhaseRelation => class_id,
    }
}

fn observe_probe(
    probe: L4WitnessProbe,
    active: &[u64],
    classes: &BTreeMap<u64, HypothesisClass>,
) -> Option<u64> {
    match probe {
        L4WitnessProbe::TransitionHistory => unique_outcome(active, classes, |id, class| {
            (class.witness_state_specific && class.witness_attract > class.witness_repel)
                .then_some(id)
        }),
        L4WitnessProbe::ContextRelation => unique_outcome(active, classes, |_, class| {
            (class.context_support && class.relation_class != 0).then_some(class.relation_class)
        }),
        L4WitnessProbe::VerifierResult => active
            .iter()
            .any(|id| classes.get(id).is_some_and(|class| class.verifier_passed))
            .then_some(1),
        L4WitnessProbe::PhaseRelation => unique_outcome(active, classes, |id, class| {
            (class.phase_witness_supported && class.phase_witness_milli > 0).then_some(id)
        }),
    }
}

fn unique_outcome(
    active: &[u64],
    classes: &BTreeMap<u64, HypothesisClass>,
    outcome: impl Fn(u64, &HypothesisClass) -> Option<u64>,
) -> Option<u64> {
    let outcomes = active
        .iter()
        .filter_map(|id| classes.get(id).and_then(|class| outcome(*id, class)))
        .collect::<BTreeSet<_>>();
    (outcomes.len() == 1).then(|| *outcomes.first().expect("one outcome"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hypothesis(class_id: u64) -> L4ActiveHypothesis {
        L4ActiveHypothesis {
            class_id,
            relation_class: class_id + 100,
            operator_class: 7,
            verifier_passed: true,
            context_support: false,
            witness_attract: 0,
            witness_repel: 0,
            witness_state_specific: false,
            phase_witness_milli: 0,
            phase_witness_supported: false,
        }
    }

    #[test]
    fn plan_is_target_independent_but_receipt_resolves_selected_state() {
        let first = [hypothesis(1), hypothesis(2)];
        let mut second = first;
        second[1].witness_state_specific = true;
        second[1].witness_attract = 3;

        let unresolved = resolve_active_hypotheses(&first);
        let resolved = resolve_active_hypotheses(&second);

        assert_eq!(unresolved.plan, resolved.plan);
        assert_eq!(unresolved.status, L4ActiveResolutionStatus::Ambiguous);
        assert_eq!(resolved.status, L4ActiveResolutionStatus::Witnessed);
        assert_eq!(resolved.selected_class, Some(2));
        assert!(resolved.certificate_valid);
    }

    #[test]
    fn context_relation_can_resolve_without_a_word_rule() {
        let mut candidates = [hypothesis(10), hypothesis(20)];
        candidates[1].context_support = true;

        let resolved = resolve_active_hypotheses(&candidates);

        assert_eq!(resolved.status, L4ActiveResolutionStatus::Witnessed);
        assert_eq!(resolved.selected_class, Some(20));
        assert!(resolved.receipts.len() <= MAX_WITNESSES);
    }

    #[test]
    fn verifier_observation_removes_unverified_hypothesis() {
        let mut candidates = [hypothesis(1), hypothesis(2)];
        candidates[0].relation_class = 0;
        candidates[1].relation_class = 0;
        candidates[1].verifier_passed = false;

        let resolved = resolve_active_hypotheses(&candidates);

        assert_eq!(resolved.status, L4ActiveResolutionStatus::Witnessed);
        assert_eq!(resolved.selected_class, Some(1));
    }

    #[test]
    fn phase_relation_resolves_from_generalized_signed_memory() {
        let mut candidates = [hypothesis(1), hypothesis(2)];
        candidates[1].phase_witness_supported = true;
        candidates[1].phase_witness_milli = 420;

        let resolved = resolve_active_hypotheses(&candidates);

        assert_eq!(resolved.status, L4ActiveResolutionStatus::Witnessed);
        assert_eq!(resolved.selected_class, Some(2));
        assert!(resolved
            .receipts
            .iter()
            .any(|receipt| receipt.probe == L4WitnessProbe::PhaseRelation));
    }

    #[test]
    fn tampered_receipt_fails_independent_replay() {
        let mut candidates = [hypothesis(1), hypothesis(2)];
        candidates[0].witness_state_specific = true;
        candidates[0].witness_attract = 2;
        let mut certificate = resolve_active_hypotheses(&candidates);
        certificate.receipts[0].classes_after.clear();

        assert!(!verify_resolution_certificate(&candidates, &certificate));
    }
}
