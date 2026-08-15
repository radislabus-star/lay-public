use std::collections::BTreeMap;

use serde::Serialize;

use super::super::super::restoration::RestorationReadout;

const GLOBAL_DIAGNOSTIC_LIMIT: usize = 64;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ClassQuality {
    pub(super) cases: usize,
    pub(super) objective_unique_cases: usize,
    pub(super) target_retained: usize,
    pub(super) target_in_projection: usize,
    pub(super) unique_top1: usize,
    pub(super) winners: usize,
    pub(super) tied: usize,
    pub(super) abstained: usize,
    pub(super) false_authority: usize,
    pub(super) false_singleton: usize,
    pub(super) exact_candidate_total: u64,
    pub(super) exact_candidate_max: usize,
    pub(super) exact_phase_noop: usize,
    pub(super) legacy_grounded_candidates: usize,
    pub(super) legacy_grounded_losses: usize,
    pub(super) runtime_target_in_projection: usize,
    pub(super) legacy_unique_top1: usize,
    pub(super) legacy_false_authority: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CleanQuality {
    pub(super) cases: usize,
    pub(super) target_retained: usize,
    pub(super) target_rank1: usize,
    pub(super) preserved: usize,
    pub(super) mutating_winner: usize,
    pub(super) winners: usize,
    pub(super) tied: usize,
    pub(super) abstained: usize,
    pub(super) exact_phase_noop: usize,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Timings {
    pub(super) typed_us: Vec<u64>,
    pub(super) implicit_us: Vec<u64>,
    pub(super) exact_settlement_us: Vec<u64>,
    pub(super) legacy_v8_us: Vec<u64>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct QualityShard {
    pub(super) classes: BTreeMap<&'static str, ClassQuality>,
    pub(super) clean: CleanQuality,
    pub(super) timings: Timings,
    pub(super) exact_reverse_terminals: u64,
    pub(super) exact_reverse_relations: u64,
    pub(super) diagnostics: Vec<LossDiagnostic>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(super) struct LossDiagnostic {
    pub(super) scope: &'static str,
    pub(super) mechanism: &'static str,
    pub(super) class: &'static str,
    pub(super) surface: String,
    pub(super) target_terminal: u32,
    pub(super) target_surface: String,
    pub(super) target_rank: Option<usize>,
    pub(super) bounded_target_rank: Option<usize>,
    pub(super) top_terminals: Vec<u32>,
    pub(super) bounded_top_terminals: Vec<u32>,
    pub(super) authority_terminal: Option<u32>,
    pub(super) objective_terminals: Vec<u32>,
    pub(super) geometric_basin_terminals: usize,
    pub(super) legacy_grounded_losses: Vec<u32>,
    pub(super) target_evidence: Option<CandidateEvidenceDiagnostic>,
    pub(super) top_candidate_evidence: Vec<CandidateEvidenceDiagnostic>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(super) struct CandidateEvidenceDiagnostic {
    pub(super) rank: usize,
    pub(super) terminal_id: u32,
    pub(super) surface: String,
    pub(super) objective_member: bool,
    pub(super) typed_certificate_classes: Vec<&'static str>,
    pub(super) implicit_activation: Option<ImplicitActivationDiagnostic>,
    pub(super) atom_hits: u16,
    pub(super) surface_hits: u16,
    pub(super) keyboard_hits: u16,
    pub(super) forward_milli: u16,
    pub(super) backward_milli: u16,
    pub(super) structural_milli: u16,
    pub(super) sequence_milli: u16,
    pub(super) position_milli: u16,
    pub(super) length_milli: u16,
    pub(super) geometry_distance: u8,
    pub(super) reconstruction_modes: u8,
    pub(super) settled_energy: i32,
    pub(super) exact_reconstruction: bool,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub(super) struct ImplicitActivationDiagnostic {
    pub(super) mass: u64,
    pub(super) hits: u16,
    pub(super) surface_hits: u16,
    pub(super) keyboard_hits: u16,
}

impl ClassQuality {
    pub(super) fn merge(&mut self, other: Self) {
        self.cases = self.cases.saturating_add(other.cases);
        self.objective_unique_cases = self
            .objective_unique_cases
            .saturating_add(other.objective_unique_cases);
        self.target_retained = self.target_retained.saturating_add(other.target_retained);
        self.target_in_projection = self
            .target_in_projection
            .saturating_add(other.target_in_projection);
        self.unique_top1 = self.unique_top1.saturating_add(other.unique_top1);
        self.winners = self.winners.saturating_add(other.winners);
        self.tied = self.tied.saturating_add(other.tied);
        self.abstained = self.abstained.saturating_add(other.abstained);
        self.false_authority = self.false_authority.saturating_add(other.false_authority);
        self.false_singleton = self.false_singleton.saturating_add(other.false_singleton);
        self.exact_candidate_total = self
            .exact_candidate_total
            .saturating_add(other.exact_candidate_total);
        self.exact_candidate_max = self.exact_candidate_max.max(other.exact_candidate_max);
        self.exact_phase_noop = self.exact_phase_noop.saturating_add(other.exact_phase_noop);
        self.legacy_grounded_candidates = self
            .legacy_grounded_candidates
            .saturating_add(other.legacy_grounded_candidates);
        self.legacy_grounded_losses = self
            .legacy_grounded_losses
            .saturating_add(other.legacy_grounded_losses);
        self.runtime_target_in_projection = self
            .runtime_target_in_projection
            .saturating_add(other.runtime_target_in_projection);
        self.legacy_unique_top1 = self
            .legacy_unique_top1
            .saturating_add(other.legacy_unique_top1);
        self.legacy_false_authority = self
            .legacy_false_authority
            .saturating_add(other.legacy_false_authority);
    }

    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "cases": self.cases,
            "objective_unique_cases": self.objective_unique_cases,
            "target_retained_complete_field": self.target_retained,
            "target_retention_percent": percent(self.target_retained, self.cases),
            "target_in_bounded_lattice": self.target_in_projection,
            "lattice_coverage_percent": percent(self.target_in_projection, self.cases),
            "unique_top1": self.unique_top1,
            "unique_top1_percent": percent(self.unique_top1, self.objective_unique_cases),
            "readout": {
                "winner": self.winners,
                "tied": self.tied,
                "abstain": self.abstained,
            },
            "false_authority": self.false_authority,
            "false_singleton": self.false_singleton,
            "complete_exact_candidates": {
                "total": self.exact_candidate_total,
                "maximum_per_case": self.exact_candidate_max,
                "phase_noop_cases": self.exact_phase_noop,
            },
            "runtime_observer": {
                "grounded_candidates": self.legacy_grounded_candidates,
                "grounded_candidate_losses_from_exact_field": self.legacy_grounded_losses,
                "target_in_bounded_lattice": self.runtime_target_in_projection,
                "unique_top1": self.legacy_unique_top1,
                "false_authority": self.legacy_false_authority,
                "scope": "compatibility_observer_only",
            },
            "gates": {
                "unique_top1_strictly_gt_95_percent": ratio_strictly_above(
                    self.unique_top1,
                    self.objective_unique_cases,
                    95,
                    100,
                ),
                "lattice_coverage_ge_99_percent": ratio_at_least(
                    self.target_in_projection,
                    self.cases,
                    99,
                    100,
                ),
            }
        })
    }
}

impl CleanQuality {
    pub(super) fn merge(&mut self, other: Self) {
        self.cases = self.cases.saturating_add(other.cases);
        self.target_retained = self.target_retained.saturating_add(other.target_retained);
        self.target_rank1 = self.target_rank1.saturating_add(other.target_rank1);
        self.preserved = self.preserved.saturating_add(other.preserved);
        self.mutating_winner = self.mutating_winner.saturating_add(other.mutating_winner);
        self.winners = self.winners.saturating_add(other.winners);
        self.tied = self.tied.saturating_add(other.tied);
        self.abstained = self.abstained.saturating_add(other.abstained);
        self.exact_phase_noop = self.exact_phase_noop.saturating_add(other.exact_phase_noop);
    }
}

impl Timings {
    pub(super) fn merge(&mut self, mut other: Self) {
        self.typed_us.append(&mut other.typed_us);
        self.implicit_us.append(&mut other.implicit_us);
        self.exact_settlement_us
            .append(&mut other.exact_settlement_us);
        self.legacy_v8_us.append(&mut other.legacy_v8_us);
    }

    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "typed_us_p50": percentile(&self.typed_us, 50),
            "typed_us_p99": percentile(&self.typed_us, 99),
            "typed_us_max": maximum(&self.typed_us),
            "implicit_us_p50": percentile(&self.implicit_us, 50),
            "implicit_us_p99": percentile(&self.implicit_us, 99),
            "implicit_us_max": maximum(&self.implicit_us),
            "exact_settlement_us_p50": percentile(&self.exact_settlement_us, 50),
            "exact_settlement_us_p99": percentile(&self.exact_settlement_us, 99),
            "exact_settlement_us_max": maximum(&self.exact_settlement_us),
            "runtime_observer_us_p50": percentile(&self.legacy_v8_us, 50),
            "runtime_observer_us_p99": percentile(&self.legacy_v8_us, 99),
            "runtime_observer_us_max": maximum(&self.legacy_v8_us),
            "scope": "proof_throughput_not_product_hot_latency",
        })
    }
}

impl QualityShard {
    pub(super) fn merge(&mut self, mut other: Self) {
        for (class, metrics) in other.classes {
            self.classes.entry(class).or_default().merge(metrics);
        }
        self.clean.merge(other.clean);
        self.timings.merge(other.timings);
        self.exact_reverse_terminals = self
            .exact_reverse_terminals
            .saturating_add(other.exact_reverse_terminals);
        self.exact_reverse_relations = self
            .exact_reverse_relations
            .saturating_add(other.exact_reverse_relations);
        self.diagnostics.append(&mut other.diagnostics);
    }

    pub(super) fn aggregate(&self) -> ClassQuality {
        let mut aggregate = ClassQuality::default();
        for metrics in self.classes.values().cloned() {
            aggregate.merge(metrics);
        }
        aggregate
    }

    pub(super) fn finish_diagnostics(&mut self) {
        self.diagnostics.sort_unstable_by(|left, right| {
            left.scope
                .cmp(right.scope)
                .then_with(|| left.mechanism.cmp(right.mechanism))
                .then_with(|| left.class.cmp(right.class))
                .then_with(|| left.surface.cmp(&right.surface))
                .then_with(|| left.target_terminal.cmp(&right.target_terminal))
        });
        self.diagnostics.truncate(GLOBAL_DIAGNOSTIC_LIMIT);
    }
}

pub(super) fn record_readout(
    readout: &RestorationReadout,
    winners: &mut usize,
    tied: &mut usize,
    abstained: &mut usize,
) {
    match readout {
        RestorationReadout::Winner { .. } => *winners += 1,
        RestorationReadout::Tied { .. } | RestorationReadout::TiedOverflow { .. } => *tied += 1,
        RestorationReadout::Abstain { .. } => *abstained += 1,
    }
}

pub(super) fn authority_terminal(readout: &RestorationReadout) -> Option<u32> {
    match readout {
        RestorationReadout::Winner { candidate } => Some(candidate.terminal_id),
        _ => None,
    }
}

pub(super) fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    numerator as f64 * 100.0 / denominator as f64
}

pub(super) fn ratio_at_least(
    numerator: usize,
    denominator: usize,
    required_numerator: usize,
    required_denominator: usize,
) -> bool {
    denominator != 0
        && numerator.saturating_mul(required_denominator)
            >= denominator.saturating_mul(required_numerator)
}

pub(super) fn ratio_strictly_above(
    numerator: usize,
    denominator: usize,
    required_numerator: usize,
    required_denominator: usize,
) -> bool {
    denominator != 0
        && numerator.saturating_mul(required_denominator)
            > denominator.saturating_mul(required_numerator)
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1).saturating_mul(percentile.min(100)) / 100]
}

fn maximum(values: &[u64]) -> u64 {
    values.iter().copied().max().unwrap_or_default()
}
