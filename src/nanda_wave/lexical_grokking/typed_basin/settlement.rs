use std::collections::BTreeMap;
use std::time::Instant;

#[cfg(any(test, feature = "lexical-compiler"))]
use sha2::{Digest, Sha256};

use super::super::atoms::{normalize_lexical_surface, AtomChannel};
use super::super::crystal::WAVE_DIMENSION;
use super::super::restoration::{self, RestorationReadout};
use super::super::runtime::{
    observed_sequence, GrokkingCandidate, LexicalGrokkingMemory, ObservedAtom, ReadoutMode,
};
use super::super::wave_basis::expand_atom;
use super::exact_reverse::ReverseBank;
#[cfg(any(test, feature = "lexical-compiler"))]
use super::exact_reverse::ReverseParityMetrics;
use super::implicit_forward::ImplicitCandidate;
use super::support::ExactSupportField;

#[cfg(any(test, feature = "lexical-compiler"))]
#[derive(Clone, Debug, Default)]
pub(super) struct SettlementAggregate {
    cases: usize,
    executed: usize,
    exact_reverse_terminals: u64,
    exact_reverse_relations: u64,
    compiler_reverse_relations: u64,
    v8_reverse_relations: u64,
    exact_compiler_reverse_mismatches: u64,
    exact_v8_reverse_mismatches: u64,
    exact_compiler_candidate_parity: usize,
    exact_compiler_readout_parity: usize,
    exact_permutation_parity: usize,
    compiler_permutation_parity: usize,
    v8_permutation_parity: usize,
    exact_phase_noop: usize,
    compiler_phase_noop: usize,
    v8_phase_noop: usize,
    complete_max_forward: usize,
    false_completeness_certificates: usize,
    exact_v8_candidate_parity: usize,
    exact_v8_readout_parity: usize,
    exact_v8_reverse_difference_cases: usize,
    exact_v8_candidate_difference_cases: usize,
    exact_v8_readout_difference_cases: usize,
    exact_us: Vec<u64>,
    compiler_us: Vec<u64>,
    v8_us: Vec<u64>,
}

#[cfg(any(test, feature = "lexical-compiler"))]
#[derive(Clone, Debug)]
pub(super) struct SettlementCaseResult {
    executed: bool,
    exact_reverse_terminals: usize,
    exact_reverse_relations: usize,
    compiler_reverse_relations: usize,
    v8_reverse_relations: usize,
    exact_compiler_reverse: ReverseParityMetrics,
    exact_v8_reverse: ReverseParityMetrics,
    exact_compiler_candidate_parity: bool,
    exact_compiler_readout_parity: bool,
    exact_permutation_parity: bool,
    compiler_permutation_parity: bool,
    v8_permutation_parity: bool,
    exact_phase_noop: bool,
    compiler_phase_noop: bool,
    v8_phase_noop: bool,
    complete_max_forward: bool,
    false_completeness_certificates: usize,
    exact_v8_candidate_parity: bool,
    exact_v8_readout_parity: bool,
    exact_us: u64,
    compiler_us: u64,
    v8_us: u64,
}

struct SettlementFingerprint {
    candidates: Vec<GrokkingCandidate>,
    readout: RestorationReadout,
    phase_noop: bool,
    elapsed_us: u64,
}

#[derive(Clone, Debug)]
pub(super) struct ExactSettlementResult {
    pub(super) candidates: Vec<GrokkingCandidate>,
    pub(super) readout: RestorationReadout,
    pub(super) phase_noop: bool,
    pub(super) reverse_terminals: usize,
    pub(super) reverse_relations: usize,
    pub(super) elapsed_us: u64,
}

#[cfg(any(test, feature = "lexical-compiler"))]
impl SettlementAggregate {
    pub(super) fn case_count(&self) -> usize {
        self.cases
    }

    pub(super) fn executed_count(&self) -> usize {
        self.executed
    }

    pub(super) fn reverse_parity_passes(&self) -> bool {
        self.cases != 0 && self.exact_compiler_reverse_mismatches == 0
    }

    pub(super) fn record(&mut self, result: &SettlementCaseResult) {
        self.cases += 1;
        self.executed += usize::from(result.executed);
        self.exact_reverse_terminals = self
            .exact_reverse_terminals
            .saturating_add(result.exact_reverse_terminals as u64);
        self.exact_reverse_relations = self
            .exact_reverse_relations
            .saturating_add(result.exact_reverse_relations as u64);
        self.compiler_reverse_relations = self
            .compiler_reverse_relations
            .saturating_add(result.compiler_reverse_relations as u64);
        self.v8_reverse_relations = self
            .v8_reverse_relations
            .saturating_add(result.v8_reverse_relations as u64);
        self.exact_compiler_reverse_mismatches = self
            .exact_compiler_reverse_mismatches
            .saturating_add(result.exact_compiler_reverse.mismatches() as u64);
        self.exact_v8_reverse_mismatches = self
            .exact_v8_reverse_mismatches
            .saturating_add(result.exact_v8_reverse.mismatches() as u64);
        self.exact_compiler_candidate_parity += usize::from(result.exact_compiler_candidate_parity);
        self.exact_compiler_readout_parity += usize::from(result.exact_compiler_readout_parity);
        self.exact_permutation_parity += usize::from(result.exact_permutation_parity);
        self.compiler_permutation_parity += usize::from(result.compiler_permutation_parity);
        self.v8_permutation_parity += usize::from(result.v8_permutation_parity);
        self.exact_phase_noop += usize::from(result.exact_phase_noop);
        self.compiler_phase_noop += usize::from(result.compiler_phase_noop);
        self.v8_phase_noop += usize::from(result.v8_phase_noop);
        self.complete_max_forward += usize::from(result.complete_max_forward);
        self.false_completeness_certificates = self
            .false_completeness_certificates
            .saturating_add(result.false_completeness_certificates);
        self.exact_v8_candidate_parity += usize::from(result.exact_v8_candidate_parity);
        self.exact_v8_readout_parity += usize::from(result.exact_v8_readout_parity);
        self.exact_v8_reverse_difference_cases +=
            usize::from(result.exact_v8_reverse.mismatches() != 0);
        self.exact_v8_candidate_difference_cases += usize::from(!result.exact_v8_candidate_parity);
        self.exact_v8_readout_difference_cases += usize::from(!result.exact_v8_readout_parity);
        self.exact_us.push(result.exact_us);
        self.compiler_us.push(result.compiler_us);
        self.v8_us.push(result.v8_us);
    }

    pub(super) fn merge(&mut self, other: &Self) {
        self.cases = self.cases.saturating_add(other.cases);
        self.executed = self.executed.saturating_add(other.executed);
        self.exact_reverse_terminals = self
            .exact_reverse_terminals
            .saturating_add(other.exact_reverse_terminals);
        self.exact_reverse_relations = self
            .exact_reverse_relations
            .saturating_add(other.exact_reverse_relations);
        self.compiler_reverse_relations = self
            .compiler_reverse_relations
            .saturating_add(other.compiler_reverse_relations);
        self.v8_reverse_relations = self
            .v8_reverse_relations
            .saturating_add(other.v8_reverse_relations);
        self.exact_compiler_reverse_mismatches = self
            .exact_compiler_reverse_mismatches
            .saturating_add(other.exact_compiler_reverse_mismatches);
        self.exact_v8_reverse_mismatches = self
            .exact_v8_reverse_mismatches
            .saturating_add(other.exact_v8_reverse_mismatches);
        self.exact_compiler_candidate_parity = self
            .exact_compiler_candidate_parity
            .saturating_add(other.exact_compiler_candidate_parity);
        self.exact_compiler_readout_parity = self
            .exact_compiler_readout_parity
            .saturating_add(other.exact_compiler_readout_parity);
        self.exact_permutation_parity = self
            .exact_permutation_parity
            .saturating_add(other.exact_permutation_parity);
        self.compiler_permutation_parity = self
            .compiler_permutation_parity
            .saturating_add(other.compiler_permutation_parity);
        self.v8_permutation_parity = self
            .v8_permutation_parity
            .saturating_add(other.v8_permutation_parity);
        self.exact_phase_noop = self.exact_phase_noop.saturating_add(other.exact_phase_noop);
        self.compiler_phase_noop = self
            .compiler_phase_noop
            .saturating_add(other.compiler_phase_noop);
        self.v8_phase_noop = self.v8_phase_noop.saturating_add(other.v8_phase_noop);
        self.complete_max_forward = self
            .complete_max_forward
            .saturating_add(other.complete_max_forward);
        self.false_completeness_certificates = self
            .false_completeness_certificates
            .saturating_add(other.false_completeness_certificates);
        self.exact_v8_candidate_parity = self
            .exact_v8_candidate_parity
            .saturating_add(other.exact_v8_candidate_parity);
        self.exact_v8_readout_parity = self
            .exact_v8_readout_parity
            .saturating_add(other.exact_v8_readout_parity);
        self.exact_v8_reverse_difference_cases = self
            .exact_v8_reverse_difference_cases
            .saturating_add(other.exact_v8_reverse_difference_cases);
        self.exact_v8_candidate_difference_cases = self
            .exact_v8_candidate_difference_cases
            .saturating_add(other.exact_v8_candidate_difference_cases);
        self.exact_v8_readout_difference_cases = self
            .exact_v8_readout_difference_cases
            .saturating_add(other.exact_v8_readout_difference_cases);
        self.exact_us.extend_from_slice(&other.exact_us);
        self.compiler_us.extend_from_slice(&other.compiler_us);
        self.v8_us.extend_from_slice(&other.v8_us);
    }

    pub(super) fn passes(&self) -> bool {
        self.cases != 0
            && self.executed == self.cases
            && self.exact_compiler_reverse_mismatches == 0
            && self.exact_compiler_candidate_parity == self.cases
            && self.exact_compiler_readout_parity == self.cases
            && self.exact_permutation_parity == self.cases
            && self.compiler_permutation_parity == self.cases
            && self.v8_permutation_parity == self.cases
            && self.exact_phase_noop == self.cases
            && self.compiler_phase_noop == self.cases
            && self.v8_phase_noop == self.cases
            && self.complete_max_forward == self.cases
            && self.false_completeness_certificates == 0
    }

    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "cases": self.cases,
            "executed": self.executed,
            "compiler_exact": {
                "reverse_terminals": self.exact_reverse_terminals,
                "exact_reverse_relations": self.exact_reverse_relations,
                "compiler_reference_relations": self.compiler_reverse_relations,
                "reverse_mismatches": self.exact_compiler_reverse_mismatches,
                "candidate_state_and_order_parity": self.exact_compiler_candidate_parity,
                "readout_parity": self.exact_compiler_readout_parity,
                "exact_permutation_parity": self.exact_permutation_parity,
                "compiler_reference_permutation_parity": self.compiler_permutation_parity,
                "exact_phase_evidence_noop": self.exact_phase_noop,
                "compiler_phase_evidence_noop": self.compiler_phase_noop,
                "complete_max_forward": self.complete_max_forward,
                "false_completeness_certificates": self.false_completeness_certificates,
                "pass": self.passes(),
            },
            "current_v8_fingerprint": {
                "reverse_relations": self.v8_reverse_relations,
                "reverse_mismatches_vs_exact": self.exact_v8_reverse_mismatches,
                "reverse_difference_cases": self.exact_v8_reverse_difference_cases,
                "candidate_state_and_order_parity_vs_exact": self.exact_v8_candidate_parity,
                "candidate_difference_cases": self.exact_v8_candidate_difference_cases,
                "readout_parity_vs_exact": self.exact_v8_readout_parity,
                "readout_difference_cases": self.exact_v8_readout_difference_cases,
                "internal_permutation_parity": self.v8_permutation_parity,
                "phase_evidence_noop": self.v8_phase_noop,
                "scope": "compatibility_observation_not_a2_gate",
            },
            "timing": {
                "exact_us_p50": percentile(&self.exact_us, 50),
                "exact_us_p99": percentile(&self.exact_us, 99),
                "exact_us_max": maximum(&self.exact_us),
                "compiler_reference_us_p99": percentile(&self.compiler_us, 99),
                "compiler_reference_us_max": maximum(&self.compiler_us),
                "current_v8_us_p99": percentile(&self.v8_us, 99),
                "current_v8_us_max": maximum(&self.v8_us),
                "scope": "proof_only_settlement_not_product_hot_p99",
            },
        })
    }
}

#[cfg(any(test, feature = "lexical-compiler"))]
pub(super) fn evaluate_settlement_case(
    memory: &LexicalGrokkingMemory,
    support: &ExactSupportField,
    surface: &str,
    implicit: &[ImplicitCandidate],
) -> Result<SettlementCaseResult, String> {
    let terminal_ids = implicit
        .iter()
        .map(|candidate| candidate.terminal_id)
        .collect::<Vec<_>>();
    let exact = ReverseBank::from_precomputed(
        implicit
            .iter()
            .map(|candidate| (candidate.terminal_id, candidate.exact_reverse.clone())),
    )?;
    let compiler = ReverseBank::compiler_reference(&memory.package, support, &terminal_ids)?;
    let current_v8 = ReverseBank::current_v8(&memory.package, &terminal_ids)?;
    let exact_compiler_reverse = exact.compare(&compiler);
    let exact_v8_reverse = exact.compare(&current_v8);
    let base = SettlementCaseResult {
        executed: false,
        exact_reverse_terminals: exact.len(),
        exact_reverse_relations: exact.relation_count(),
        compiler_reverse_relations: compiler.relation_count(),
        v8_reverse_relations: current_v8.relation_count(),
        exact_compiler_reverse,
        exact_v8_reverse,
        exact_compiler_candidate_parity: false,
        exact_compiler_readout_parity: false,
        exact_permutation_parity: false,
        compiler_permutation_parity: false,
        v8_permutation_parity: false,
        exact_phase_noop: false,
        compiler_phase_noop: false,
        v8_phase_noop: false,
        complete_max_forward: false,
        false_completeness_certificates: 0,
        exact_v8_candidate_parity: false,
        exact_v8_readout_parity: false,
        exact_us: 0,
        compiler_us: 0,
        v8_us: 0,
    };
    if exact_compiler_reverse.mismatches() != 0 || exact.fingerprint() != compiler.fingerprint() {
        return Ok(base);
    }

    let max_forward = implicit
        .iter()
        .map(|candidate| candidate.activation.mass)
        .max()
        .unwrap_or(1)
        .max(1);
    let mut reversed_ids = terminal_ids.clone();
    reversed_ids.reverse();
    let exact_forward = run_fingerprint(
        memory,
        surface,
        implicit,
        &terminal_ids,
        &exact,
        max_forward,
    )?;
    let exact_reversed = run_fingerprint(
        memory,
        surface,
        implicit,
        &reversed_ids,
        &exact,
        max_forward,
    )?;
    let compiler_forward = run_fingerprint(
        memory,
        surface,
        implicit,
        &terminal_ids,
        &compiler,
        max_forward,
    )?;
    let compiler_reversed = run_fingerprint(
        memory,
        surface,
        implicit,
        &reversed_ids,
        &compiler,
        max_forward,
    )?;
    let v8_forward = run_fingerprint(
        memory,
        surface,
        implicit,
        &terminal_ids,
        &current_v8,
        max_forward,
    )?;
    let v8_reversed = run_fingerprint(
        memory,
        surface,
        implicit,
        &reversed_ids,
        &current_v8,
        max_forward,
    )?;

    let complete = exact_forward.candidates.len() == implicit.len()
        && compiler_forward.candidates.len() == implicit.len()
        && v8_forward.candidates.len() == implicit.len();
    Ok(SettlementCaseResult {
        executed: true,
        exact_compiler_candidate_parity: candidate_fingerprint(&exact_forward.candidates)
            == candidate_fingerprint(&compiler_forward.candidates),
        exact_compiler_readout_parity: exact_forward.readout == compiler_forward.readout,
        exact_permutation_parity: fingerprints_equal(&exact_forward, &exact_reversed),
        compiler_permutation_parity: fingerprints_equal(&compiler_forward, &compiler_reversed),
        v8_permutation_parity: fingerprints_equal(&v8_forward, &v8_reversed),
        exact_phase_noop: exact_forward.phase_noop && exact_reversed.phase_noop,
        compiler_phase_noop: compiler_forward.phase_noop && compiler_reversed.phase_noop,
        v8_phase_noop: v8_forward.phase_noop && v8_reversed.phase_noop,
        complete_max_forward: max_forward
            == implicit
                .iter()
                .map(|candidate| candidate.activation.mass)
                .max()
                .unwrap_or(1)
                .max(1),
        false_completeness_certificates: usize::from(!complete),
        exact_v8_candidate_parity: candidate_fingerprint(&exact_forward.candidates)
            == candidate_fingerprint(&v8_forward.candidates),
        exact_v8_readout_parity: exact_forward.readout == v8_forward.readout,
        exact_us: exact_forward
            .elapsed_us
            .saturating_add(exact_reversed.elapsed_us),
        compiler_us: compiler_forward
            .elapsed_us
            .saturating_add(compiler_reversed.elapsed_us),
        v8_us: v8_forward.elapsed_us.saturating_add(v8_reversed.elapsed_us),
        ..base
    })
}

pub(super) fn settle_exact_case(
    memory: &LexicalGrokkingMemory,
    _support: &ExactSupportField,
    surface: &str,
    implicit: &[ImplicitCandidate],
) -> Result<ExactSettlementResult, String> {
    let started = Instant::now();
    if implicit.is_empty() {
        return Ok(ExactSettlementResult {
            candidates: Vec::new(),
            readout: restoration::classify(&[], memory.package.restoration_calibration),
            phase_noop: true,
            reverse_terminals: 0,
            reverse_relations: 0,
            elapsed_us: started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64,
        });
    }

    let terminal_ids = implicit
        .iter()
        .map(|candidate| candidate.terminal_id)
        .collect::<Vec<_>>();
    let reverse = ReverseBank::from_precomputed(
        implicit
            .iter()
            .map(|candidate| (candidate.terminal_id, candidate.exact_reverse.clone())),
    )?;
    let max_forward = implicit
        .iter()
        .map(|candidate| candidate.activation.mass)
        .max()
        .unwrap_or(1)
        .max(1);
    let fingerprint = run_fingerprint(
        memory,
        surface,
        implicit,
        &terminal_ids,
        &reverse,
        max_forward,
    )?;
    if fingerprint.candidates.len() != implicit.len() {
        return Err(format!(
            "Gate C exact settlement truncated the complete basin: {} != {}",
            fingerprint.candidates.len(),
            implicit.len()
        ));
    }
    Ok(ExactSettlementResult {
        candidates: fingerprint.candidates,
        readout: fingerprint.readout,
        phase_noop: fingerprint.phase_noop,
        reverse_terminals: reverse.len(),
        reverse_relations: reverse.relation_count(),
        elapsed_us: started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64,
    })
}

fn run_fingerprint(
    memory: &LexicalGrokkingMemory,
    surface: &str,
    implicit: &[ImplicitCandidate],
    terminal_order: &[u32],
    reverse: &ReverseBank,
    max_forward: u64,
) -> Result<SettlementFingerprint, String> {
    let started = Instant::now();
    let resolved = memory.resolve_surface(surface);
    if resolved.is_empty() {
        return Err("A2 settlement query contains no resolved atoms".to_string());
    }
    let character_sequence = observed_sequence(&resolved, AtomChannel::CharacterAnchor);
    let observed_char_count = normalize_lexical_surface(surface)
        .chars()
        .count()
        .min(u8::MAX as usize) as u8;
    let observed = resolved
        .iter()
        .filter(|(_, atom)| atom.channel != AtomChannel::CharacterAnchor)
        .copied()
        .collect::<BTreeMap<u32, ObservedAtom>>();
    let mut surface_re = [0_i32; WAVE_DIMENSION];
    let mut surface_im = [0_i32; WAVE_DIMENSION];
    for (atom_id, atom) in &observed {
        let record = memory
            .package
            .atoms
            .get(*atom_id as usize)
            .ok_or_else(|| format!("A2 observed atom is invalid: {atom_id}"))?;
        expand_atom(
            &memory.package.basis,
            record.wave_code,
            &mut surface_re,
            &mut surface_im,
            i32::from(atom.weight),
        );
    }
    let by_terminal = implicit
        .iter()
        .map(|candidate| (candidate.terminal_id, candidate))
        .collect::<BTreeMap<_, _>>();
    if by_terminal.len() != implicit.len() || terminal_order.len() != implicit.len() {
        return Err("A2 settlement domain contains duplicate or missing terminals".to_string());
    }
    let mut candidates = Vec::with_capacity(implicit.len());
    for terminal_id in terminal_order.iter().copied() {
        let implicit = by_terminal.get(&terminal_id).ok_or_else(|| {
            format!("A2 settlement order contains foreign terminal: {terminal_id}")
        })?;
        let relations = reverse
            .get(terminal_id)
            .ok_or_else(|| format!("A2 exact reverse is missing terminal: {terminal_id}"))?;
        let candidate = memory
            .settle_candidate_with_reverse(
                terminal_id,
                implicit.activation,
                max_forward,
                &observed,
                &surface_re,
                &surface_im,
                &character_sequence,
                observed_char_count,
                ReadoutMode::Full,
                relations,
            )
            .ok_or_else(|| format!("A2 settlement rejected valid terminal: {terminal_id}"))?;
        candidates.push(candidate);
    }
    memory.apply_restoration_geometry_with_explicit_reverse(
        surface,
        &mut candidates,
        |terminal_id| reverse.get(terminal_id),
    )?;
    memory.finalize_candidates_after_geometry(
        candidates.len(),
        ReadoutMode::Full,
        &surface_re,
        &surface_im,
        &mut candidates,
    );
    let before_phase = candidates.clone();
    memory.apply_l11_phase_evidence(surface, &mut candidates);
    let phase_noop = before_phase == candidates;
    let readout = restoration::classify(&candidates, memory.package.restoration_calibration);
    Ok(SettlementFingerprint {
        candidates,
        readout,
        phase_noop,
        elapsed_us: started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64,
    })
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn fingerprints_equal(left: &SettlementFingerprint, right: &SettlementFingerprint) -> bool {
    candidate_fingerprint(&left.candidates) == candidate_fingerprint(&right.candidates)
        && left.readout == right.readout
        && left.phase_noop == right.phase_noop
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn candidate_fingerprint(candidates: &[GrokkingCandidate]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"lay.l11.phase8i.settled-candidates.v1");
    for candidate in candidates {
        hasher.update(format!("{candidate:?}").as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1).saturating_mul(percentile.min(100)) / 100]
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn maximum(values: &[u64]) -> u64 {
    values.iter().copied().max().unwrap_or_default()
}
