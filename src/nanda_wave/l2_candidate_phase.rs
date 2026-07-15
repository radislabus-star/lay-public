//! Compact per-operator phase memory for typed correction transitions.
//!
//! The package stores relation centers and anti-centers, not words. Candidate
//! producers can observe the readout, while DecisionCore and the verifier keep
//! sole apply authority.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::f32::consts::TAU;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::transition_relation::{TransitionOperatorKind, TransitionRelationAtoms};

use super::mode::mix64_golden;

const MAGIC: &[u8; 8] = b"LAYPC004";
const CELLS: usize = 128;
const HEADER_BYTES: usize = 16;
const PROFILE_HEADER_BYTES: usize = 24;
const CENTER_HEADER_BYTES: usize = 4;
const CELL_BYTES: usize = 4;
const PHASE_SCALE: f32 = 16_384.0;
const CENTER_SPLIT_COHERENCE: f32 = 0.94;
const MAX_POSITIVE_CENTERS: usize = 16;
const MAX_ANTI_CENTERS: usize = 32;
const MIN_MARGIN_MICRO: i64 = 25_000;
const MIN_LEARNED_SUPPORT_MARGIN_MICRO: i64 = 1_000;
const MAX_MARGIN_MICRO: i64 = 450_000;

static DEFAULT_RUNTIME: OnceLock<Option<PhaseRuntime>> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct L2PhaseTrainingEntry {
    pub original: String,
    pub candidate: String,
    pub operation: String,
    pub accepted: bool,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PhaseVerdict {
    Support,
    Repel,
    #[default]
    Unknown,
}

impl PhaseVerdict {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Support => "support",
            Self::Repel => "repel",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PhaseReadout {
    pub(crate) package_loaded: bool,
    pub(crate) operator_present: bool,
    pub(crate) operator_promoted: bool,
    pub(crate) positive_micro: i64,
    pub(crate) anti_micro: i64,
    pub(crate) margin_micro: i64,
    pub(crate) threshold_micro: i64,
    pub(crate) positive_examples: u32,
    pub(crate) negative_examples: u32,
    pub(crate) positive_centers: u8,
    pub(crate) anti_centers: u8,
    pub(crate) covered_surfaces: u32,
    pub(crate) rejected_surfaces: u32,
    pub(crate) verdict: PhaseVerdict,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PhaseCell {
    re: f32,
    im: f32,
}

#[derive(Clone, Debug)]
struct PhaseCenter {
    sum: Vec<PhaseCell>,
    center: Vec<PhaseCell>,
    support: u32,
}

#[derive(Clone, Debug)]
struct PhaseProfile {
    operator: PhaseOperator,
    promoted: bool,
    positive: Vec<PhaseCenter>,
    negative: Vec<PhaseCenter>,
    positive_examples: u32,
    negative_examples: u32,
    covered_surfaces: u32,
    rejected_surfaces: u32,
    threshold_micro: i64,
}

#[derive(Clone, Debug, Default)]
struct PhaseRuntime {
    profiles: Vec<PhaseProfile>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct PhaseEvaluator {
    runtime: Option<PhaseRuntime>,
}

#[derive(Clone, Debug, Default)]
struct PhaseProfileBuilder {
    positive: Vec<PhaseCenter>,
    negative: Vec<PhaseCenter>,
    positive_examples: u32,
    negative_examples: u32,
    positive_surfaces: BTreeSet<u64>,
    negative_surfaces: BTreeSet<u64>,
    counterfactual_circuits: BTreeSet<String>,
    positive_vectors: Vec<Vec<PhaseCell>>,
    negative_vectors: Vec<Vec<PhaseCell>>,
}

type PhaseOperator = TransitionOperatorKind;

pub(super) fn default_phase_memory_path() -> PathBuf {
    env::var_os("LAY_NANDA_L2_PHASE_MEMORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share/lay/nanda_wave/l2_candidate_phase.nwpc")
        })
}

pub(super) fn write_phase_memory_from_entries<I>(path: &Path, entries: I) -> io::Result<usize>
where
    I: IntoIterator<Item = (String, String, String, usize)>,
{
    write_phase_memory_from_labeled_entries(
        path,
        entries.into_iter().map(
            |(original, candidate, operation, count)| L2PhaseTrainingEntry {
                original,
                candidate,
                operation,
                accepted: true,
                count,
            },
        ),
    )
}

pub(super) fn write_phase_memory_from_labeled_entries<I>(
    path: &Path,
    entries: I,
) -> io::Result<usize>
where
    I: IntoIterator<Item = L2PhaseTrainingEntry>,
{
    let entries = entries.into_iter().collect::<Vec<_>>();
    let promoted = proven_phase_operators(&entries);
    let mut runtime = train_phase_runtime(entries)?;
    for profile in &mut runtime.profiles {
        profile.promoted = promoted.contains(&profile.operator);
    }
    let bytes = runtime.to_bytes();
    crate::private_file::write_private_bytes(path, &bytes)?;
    Ok(bytes.len())
}

pub(crate) fn relation_readout(action_operator: &str, atoms: &[String]) -> PhaseReadout {
    let Some(runtime) = DEFAULT_RUNTIME.get_or_init(load_default_runtime).as_ref() else {
        return PhaseReadout::default();
    };
    runtime.readout(PhaseOperator::from_action_operator(action_operator), atoms)
}

pub(super) fn shadow_readout(original: &str, candidate: &str, operation: &str) -> PhaseReadout {
    let Some(runtime) = DEFAULT_RUNTIME.get_or_init(load_default_runtime).as_ref() else {
        return PhaseReadout::default();
    };
    readout_for_pair(runtime, original, candidate, operation)
}

pub(super) fn shadow_readout_from_path(
    original: &str,
    candidate: &str,
    operation: &str,
    path: &Path,
) -> PhaseReadout {
    let Ok(bytes) = fs::read(path) else {
        return PhaseReadout::default();
    };
    let Ok(runtime) = PhaseRuntime::from_bytes(&bytes) else {
        return PhaseReadout::default();
    };
    readout_for_pair(&runtime, original, candidate, operation)
}

impl PhaseEvaluator {
    pub(super) fn load(path: Option<&Path>) -> Self {
        let owned;
        let path = match path {
            Some(path) => path,
            None => {
                owned = default_phase_memory_path();
                &owned
            }
        };
        let runtime = fs::read(path)
            .ok()
            .and_then(|bytes| PhaseRuntime::from_bytes(&bytes).ok());
        Self { runtime }
    }

    pub(super) fn readout(&self, original: &str, candidate: &str, operation: &str) -> PhaseReadout {
        self.runtime
            .as_ref()
            .map(|runtime| readout_for_pair(runtime, original, candidate, operation))
            .unwrap_or_default()
    }
}

fn readout_for_pair(
    runtime: &PhaseRuntime,
    original: &str,
    candidate: &str,
    operation: &str,
) -> PhaseReadout {
    let operator = PhaseOperator::infer(original, candidate, operation);
    let atoms = relation_atoms(original, candidate, operator);
    runtime.readout(operator, atoms.atoms())
}

pub(super) fn phase_memory_report_json(path: &Path) -> serde_json::Value {
    let bytes = fs::read(path).unwrap_or_default();
    let runtime = PhaseRuntime::from_bytes(&bytes).ok();
    let (promoted_profiles, positive_centers, anti_centers, covered_surfaces, rejected_surfaces) =
        runtime
            .as_ref()
            .map(|runtime| {
                runtime.profiles.iter().fold(
                    (0_usize, 0_usize, 0_usize, 0_u64, 0_u64),
                    |(promoted, positive, anti, covered, rejected), profile| {
                        (
                            promoted + usize::from(profile.promoted),
                            positive + profile.positive.len(),
                            anti + profile.negative.len(),
                            covered + u64::from(profile.covered_surfaces),
                            rejected + u64::from(profile.rejected_surfaces),
                        )
                    },
                )
            })
            .unwrap_or_default();
    let profiles = runtime
        .as_ref()
        .map(|runtime| {
            runtime
                .profiles
                .iter()
                .map(|profile| {
                    serde_json::json!({
                        "operator": profile.operator.as_str(),
                        "promoted": profile.promoted,
                        "positive_examples": profile.positive_examples,
                        "negative_examples": profile.negative_examples,
                        "positive_centers": profile.positive.len(),
                        "anti_centers": profile.negative.len(),
                        "covered_surfaces": profile.covered_surfaces,
                        "rejected_surfaces": profile.rejected_surfaces,
                        "margin_threshold_micro": profile.threshold_micro,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    serde_json::json!({
        "kind": "l2_transition_phase_memory",
        "schema": "lay.l2-transition-phase-memory.v4",
        "path": path.display().to_string(),
        "loaded": runtime.is_some(),
        "hot_bytes": bytes.len(),
        "cells": CELLS,
        "profile_count": profiles.len(),
        "operator_family_target": 11,
        "operator_coverage_percent": profiles.len() as f64 / 11.0 * 100.0,
        "promoted_profiles": promoted_profiles,
        "promoted_coverage_percent": promoted_profiles as f64 / 11.0 * 100.0,
        "positive_centers": positive_centers,
        "anti_centers": anti_centers,
        "covered_surfaces": covered_surfaces,
        "rejected_surfaces": rejected_surfaces,
        "exact_traces_in_hot_package": 0,
        "profiles": profiles,
        "raw_words_stored": false,
        "anti_wave_evidence": "labeled negatives plus typed structural counterfactuals",
        "decision_authority": "TransitionDecisionCore",
        "apply_authority": false,
    })
}

pub(super) fn phase_proof_json(entries: &[L2PhaseTrainingEntry]) -> serde_json::Value {
    let prepared = prepare_phase_proof(entries);
    if prepared.training.is_empty() || prepared.heldout.is_empty() {
        return serde_json::json!({
            "kind": "l2_transition_phase_proof",
            "verdict": "WATCH",
            "reason": "insufficient_independent_surfaces",
            "training_entries": prepared.training.len(),
            "heldout_entries": prepared.heldout.len(),
        });
    }
    let Ok(runtime) = train_phase_runtime(prepared.training.clone()) else {
        return serde_json::json!({
            "kind": "l2_transition_phase_proof",
            "verdict": "WATCH",
            "reason": "phase_runtime_not_trainable",
        });
    };
    let (reports, by_operator) = collect_phase_proof_reports(&runtime, &prepared.heldout);
    let full = reports.get("full_phase").copied().unwrap_or_default();
    let destructive_best = destructive_safe_positive_support(&reports);
    let causal_drop = full.positive_support.saturating_sub(destructive_best);
    let promoted_operators = by_operator
        .iter()
        .filter(|(_, reports)| operator_proof_promoted(reports))
        .map(|(operator, _)| operator.as_str())
        .collect::<Vec<_>>();
    let false_accept_examples = prepared
        .heldout
        .iter()
        .filter(|entry| !entry.accepted)
        .filter_map(|entry| {
            let operator =
                PhaseOperator::infer(&entry.original, &entry.candidate, &entry.operation);
            let atoms = relation_atoms(&entry.original, &entry.candidate, operator);
            let readout = runtime.readout(operator, atoms.atoms());
            (readout.verdict == PhaseVerdict::Support).then(|| {
                serde_json::json!({
                    "operator": operator.as_str(),
                    "original": entry.original,
                    "candidate": entry.candidate,
                    "operation": entry.operation,
                    "margin_micro": readout.margin_micro,
                    "threshold_micro": readout.threshold_micro,
                    "atoms": atoms.atoms(),
                })
            })
        })
        .take(20)
        .collect::<Vec<_>>();
    let positive_miss_examples = prepared
        .heldout
        .iter()
        .filter(|entry| entry.accepted)
        .filter_map(|entry| {
            let operator =
                PhaseOperator::infer(&entry.original, &entry.candidate, &entry.operation);
            let atoms = relation_atoms(&entry.original, &entry.candidate, operator);
            let readout = runtime.readout(operator, atoms.atoms());
            (readout.verdict != PhaseVerdict::Support).then(|| {
                serde_json::json!({
                    "operator": operator.as_str(),
                    "original": entry.original,
                    "candidate": entry.candidate,
                    "operation": entry.operation,
                    "verdict": readout.verdict.as_str(),
                    "positive_micro": readout.positive_micro,
                    "anti_micro": readout.anti_micro,
                    "margin_micro": readout.margin_micro,
                    "threshold_micro": readout.threshold_micro,
                    "atoms": atoms.atoms(),
                })
            })
        })
        .take(20)
        .collect::<Vec<_>>();
    let verdict = if full.negative_support > 0 {
        "VETO"
    } else if full.positive_cases == 0
        || full.positive_support * 100 < full.positive_cases * 80
        || causal_drop == 0
    {
        "WATCH"
    } else {
        "PASS"
    };
    serde_json::json!({
        "kind": "l2_transition_phase_proof",
        "schema": "lay.l2-transition-phase-proof.v1",
        "verdict": verdict,
        "training_entries": prepared.training.len(),
        "heldout_entries": prepared.heldout.len(),
        "training_surfaces": prepared.training_surfaces,
        "heldout_surfaces": prepared.heldout_surfaces,
        "lexical_negative_rows_deferred_to_l2_word_center": prepared.lexical_negative_rows,
        "raw_words_stored": false,
        "exact_memory_rows_after_compile": 0,
        "full_phase_false_accepts": full.negative_support,
        "false_accept_examples": false_accept_examples,
        "positive_miss_examples": positive_miss_examples,
        "causal_positive_support_drop": causal_drop,
        "modes": reports.iter().map(|(mode, report)| (*mode, report.json())).collect::<BTreeMap<_, _>>(),
        "promoted_operators": promoted_operators,
        "by_operator": by_operator.iter().map(|(operator, reports)| (operator.as_str(), operator_proof_json(reports))).collect::<BTreeMap<_, _>>(),
        "gate": {
            "heldout_positive_support_min_percent": 80,
            "heldout_negative_false_accepts_required": 0,
            "destructive_ablation_drop_required": true,
        }
    })
}

type PhaseProofReports = BTreeMap<&'static str, PhaseProofModeReport>;

fn collect_phase_proof_reports(
    runtime: &PhaseRuntime,
    heldout: &[L2PhaseTrainingEntry],
) -> (
    PhaseProofReports,
    BTreeMap<PhaseOperator, PhaseProofReports>,
) {
    let modes = [
        PhaseAblation::Full,
        PhaseAblation::NoPhase,
        PhaseAblation::ShuffledPhase,
        PhaseAblation::MagnitudeOnly,
        PhaseAblation::RandomCenter,
        PhaseAblation::WithoutAnti,
    ];
    let mut reports = PhaseProofReports::new();
    let mut by_operator = BTreeMap::<PhaseOperator, PhaseProofReports>::new();
    for entry in heldout {
        let operator = PhaseOperator::infer(&entry.original, &entry.candidate, &entry.operation);
        let atoms = relation_atoms(&entry.original, &entry.candidate, operator);
        for mode in modes {
            let verdict = runtime.readout_ablation(operator, atoms.atoms(), mode);
            reports
                .entry(mode.as_str())
                .or_default()
                .add(entry.accepted, verdict);
            by_operator
                .entry(operator)
                .or_default()
                .entry(mode.as_str())
                .or_default()
                .add(entry.accepted, verdict);
        }
    }
    (reports, by_operator)
}

fn proven_phase_operators(entries: &[L2PhaseTrainingEntry]) -> BTreeSet<PhaseOperator> {
    let prepared = prepare_phase_proof(entries);
    let Ok(runtime) = train_phase_runtime(prepared.training.clone()) else {
        return BTreeSet::new();
    };
    let (_, by_operator) = collect_phase_proof_reports(&runtime, &prepared.heldout);
    by_operator
        .into_iter()
        .filter_map(|(operator, reports)| operator_proof_promoted(&reports).then_some(operator))
        .collect()
}

fn operator_proof_promoted(reports: &PhaseProofReports) -> bool {
    let full = reports.get("full_phase").copied().unwrap_or_default();
    let destructive_best = destructive_safe_positive_support(reports);
    let without_anti = reports.get("without_anti").copied().unwrap_or_default();
    full.positive_cases > 0
        && full.positive_support * 100 >= full.positive_cases * 80
        && full.negative_cases > 0
        && full.negative_support == 0
        && full.positive_support > destructive_best
        && without_anti.negative_support > full.negative_support
}

fn operator_proof_json(reports: &PhaseProofReports) -> serde_json::Value {
    let full = reports.get("full_phase").copied().unwrap_or_default();
    let destructive_best = destructive_safe_positive_support(reports);
    let without_anti = reports.get("without_anti").copied().unwrap_or_default();
    let mut value = full.json();
    let object = value.as_object_mut().expect("proof report is an object");
    object.insert(
        "promotion_verdict".to_string(),
        serde_json::json!(if operator_proof_promoted(reports) {
            "PROMOTED"
        } else {
            "SHADOW_ONLY"
        }),
    );
    object.insert(
        "causal_positive_support_drop".to_string(),
        serde_json::json!(full.positive_support.saturating_sub(destructive_best)),
    );
    object.insert(
        "anti_center_false_accept_prevention".to_string(),
        serde_json::json!(without_anti
            .negative_support
            .saturating_sub(full.negative_support)),
    );
    object.insert(
        "modes".to_string(),
        serde_json::json!(reports
            .iter()
            .map(|(mode, report)| (*mode, report.json()))
            .collect::<BTreeMap<_, _>>()),
    );
    value
}

fn destructive_safe_positive_support(reports: &PhaseProofReports) -> usize {
    [
        "no_phase",
        "shuffled_phase",
        "magnitude_only",
        "random_center",
    ]
    .into_iter()
    .filter_map(|mode| reports.get(mode))
    .filter(|report| report.negative_support == 0)
    .map(|report| report.positive_support)
    .max()
    .unwrap_or_default()
}

#[derive(Clone, Debug, Default)]
struct PhaseProofPrepared {
    training: Vec<L2PhaseTrainingEntry>,
    heldout: Vec<L2PhaseTrainingEntry>,
    training_surfaces: usize,
    heldout_surfaces: usize,
    lexical_negative_rows: usize,
}

fn prepare_phase_proof(entries: &[L2PhaseTrainingEntry]) -> PhaseProofPrepared {
    let mut groups =
        BTreeMap::<(PhaseOperator, bool, String), BTreeMap<u64, L2PhaseTrainingEntry>>::new();
    let mut lexical_negative_rows = 0usize;
    for entry in entries {
        if !is_trainable_pair(&entry.original, &entry.candidate) {
            continue;
        }
        let operator = PhaseOperator::infer(&entry.original, &entry.candidate, &entry.operation);
        if operator == PhaseOperator::Other {
            continue;
        }
        if !entry.accepted && !is_structural_negative(entry, operator) {
            lexical_negative_rows += 1;
            continue;
        }
        let relation = relation_atoms(&entry.original, &entry.candidate, operator);
        if entry.accepted && !relation.verifier_passed() {
            continue;
        }
        let circuit = phase_circuit_key(relation.atoms());
        let surface = concrete_surface_id(&entry.original, &entry.candidate);
        groups
            .entry((operator, entry.accepted, circuit))
            .or_default()
            .entry(surface)
            .and_modify(|stored| stored.count = stored.count.saturating_add(entry.count))
            .or_insert_with(|| entry.clone());
    }
    let mut prepared = PhaseProofPrepared::default();
    for ((_operator, accepted, _circuit), surfaces) in groups {
        let reserve = if accepted { 2 } else { 1 };
        let holdout = surfaces.len().saturating_sub(reserve).div_ceil(4);
        let split = surfaces.len().saturating_sub(holdout);
        for (index, (_surface, entry)) in surfaces.into_iter().enumerate() {
            if index >= split {
                prepared.heldout.push(entry);
                prepared.heldout_surfaces += 1;
            } else {
                prepared.training.push(entry);
                prepared.training_surfaces += 1;
            }
        }
    }
    prepared.lexical_negative_rows = lexical_negative_rows;
    prepared
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhaseAblation {
    Full,
    NoPhase,
    ShuffledPhase,
    MagnitudeOnly,
    RandomCenter,
    WithoutAnti,
}

impl PhaseAblation {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full_phase",
            Self::NoPhase => "no_phase",
            Self::ShuffledPhase => "shuffled_phase",
            Self::MagnitudeOnly => "magnitude_only",
            Self::RandomCenter => "random_center",
            Self::WithoutAnti => "without_anti",
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PhaseProofModeReport {
    positive_cases: usize,
    positive_support: usize,
    positive_repel: usize,
    negative_cases: usize,
    negative_support: usize,
    negative_repel: usize,
}

impl PhaseProofModeReport {
    fn add(&mut self, accepted: bool, verdict: PhaseVerdict) {
        if accepted {
            self.positive_cases += 1;
            self.positive_support += usize::from(verdict == PhaseVerdict::Support);
            self.positive_repel += usize::from(verdict == PhaseVerdict::Repel);
        } else {
            self.negative_cases += 1;
            self.negative_support += usize::from(verdict == PhaseVerdict::Support);
            self.negative_repel += usize::from(verdict == PhaseVerdict::Repel);
        }
    }

    fn json(self) -> serde_json::Value {
        serde_json::json!({
            "positive_cases": self.positive_cases,
            "positive_support": self.positive_support,
            "positive_support_percent": proof_percent(self.positive_support, self.positive_cases),
            "positive_repel": self.positive_repel,
            "negative_cases": self.negative_cases,
            "negative_support_false_accepts": self.negative_support,
            "negative_repel": self.negative_repel,
            "negative_repel_percent": proof_percent(self.negative_repel, self.negative_cases),
        })
    }
}

fn proof_percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        (numerator as f64 * 10_000.0 / denominator as f64).round() / 100.0
    }
}

fn load_default_runtime() -> Option<PhaseRuntime> {
    PhaseRuntime::from_bytes(&fs::read(default_phase_memory_path()).ok()?).ok()
}

fn train_phase_runtime<I>(entries: I) -> io::Result<PhaseRuntime>
where
    I: IntoIterator<Item = L2PhaseTrainingEntry>,
{
    let mut builders = BTreeMap::<PhaseOperator, PhaseProfileBuilder>::new();
    for entry in entries {
        if !is_trainable_pair(&entry.original, &entry.candidate) {
            continue;
        }
        let operator = PhaseOperator::infer(&entry.original, &entry.candidate, &entry.operation);
        if operator == PhaseOperator::Other {
            continue;
        }
        if !entry.accepted && !is_structural_negative(&entry, operator) {
            continue;
        }
        let relation = relation_atoms(&entry.original, &entry.candidate, operator);
        if entry.accepted && !relation.verifier_passed() {
            continue;
        }
        let vector = phase_vector_from_atoms(relation.atoms());
        let builder = builders.entry(operator).or_default();
        let surface_id = concrete_surface_id(&entry.original, &entry.candidate);
        let new_counterfactual_circuit = entry.accepted
            && builder
                .counterfactual_circuits
                .insert(phase_circuit_key(relation.atoms()));
        if entry.accepted {
            builder.positive_surfaces.insert(surface_id);
        } else {
            builder.negative_vectors.push(vector.clone());
        }
        let repeats = entry.count.clamp(1, 8);
        for _ in 0..repeats {
            builder.add(&vector, entry.accepted);
        }
        if entry.accepted {
            builder.positive_vectors.push(vector);
            if new_counterfactual_circuit {
                for counterfactual in typed_structural_counterfactuals(operator, relation.atoms()) {
                    let vector = phase_vector_from_atoms(&counterfactual);
                    builder.add(&vector, false);
                    builder.negative_vectors.push(vector);
                }
            }
        } else {
            builder.negative_surfaces.insert(surface_id);
        }
    }

    let profiles = builders
        .into_iter()
        .filter_map(|(operator, builder)| builder.compile(operator))
        .collect::<Vec<_>>();
    if profiles.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "not enough labeled L2 phase examples",
        ));
    }
    Ok(PhaseRuntime { profiles })
}

fn typed_structural_counterfactuals(operator: PhaseOperator, atoms: &[String]) -> Vec<Vec<String>> {
    let mut verifier_failure = atoms.to_vec();
    replace_atom(&mut verifier_failure, "verified:", "verified:false");
    replace_atom(
        &mut verifier_failure,
        "left-context:",
        "left-context:touched",
    );
    replace_atom(
        &mut verifier_failure,
        "changed-tokens:",
        "changed-tokens:2-3",
    );

    let mut shape_mismatch = atoms.to_vec();
    match operator {
        PhaseOperator::LayoutProjection | PhaseOperator::ManualToggle => {
            replace_atom(
                &mut shape_mismatch,
                "observed-edit:",
                "observed-edit:substitute-one",
            );
            replace_atom(&mut shape_mismatch, "script:", "script:same->same");
        }
        PhaseOperator::AdjacentTransposition => replace_atom(
            &mut shape_mismatch,
            "observed-edit:",
            "observed-edit:substitute-one",
        ),
        PhaseOperator::MissingLetterRepair | PhaseOperator::AcceptCompletion => replace_atom(
            &mut shape_mismatch,
            "observed-edit:",
            "observed-edit:delete-one",
        ),
        PhaseOperator::RepeatedLetterRepair | PhaseOperator::ExtraLetterRepair => replace_atom(
            &mut shape_mismatch,
            "observed-edit:",
            "observed-edit:insert-one",
        ),
        PhaseOperator::LetterSubstitution => replace_atom(
            &mut shape_mismatch,
            "observed-edit:",
            "observed-edit:insert-one",
        ),
        PhaseOperator::BoundarySplit | PhaseOperator::BoundaryMerge => {
            replace_atom(&mut shape_mismatch, "observed-edit:", "observed-edit:keep");
            replace_atom(&mut shape_mismatch, "boundary:", "boundary:same");
        }
        PhaseOperator::CompositeTypo | PhaseOperator::ContextChoice => {
            replace_atom(&mut shape_mismatch, "observed-edit:", "observed-edit:keep");
        }
        PhaseOperator::Other => return Vec::new(),
    }
    vec![verifier_failure, shape_mismatch]
}

fn replace_atom(atoms: &mut [String], prefix: &str, replacement: &str) {
    if let Some(atom) = atoms.iter_mut().find(|atom| atom.starts_with(prefix)) {
        *atom = replacement.to_string();
    }
}

impl PhaseProfileBuilder {
    fn add(&mut self, vector: &[PhaseCell], accepted: bool) {
        if accepted {
            self.positive_examples = self.positive_examples.saturating_add(1);
            add_cluster(&mut self.positive, vector, MAX_POSITIVE_CENTERS);
        } else {
            self.negative_examples = self.negative_examples.saturating_add(1);
            add_cluster(&mut self.negative, vector, MAX_ANTI_CENTERS);
        }
    }

    fn compile(self, operator: PhaseOperator) -> Option<PhaseProfile> {
        if self.positive_examples == 0 {
            return None;
        }
        let mut profile = PhaseProfile {
            operator,
            promoted: false,
            positive: self.positive,
            negative: self.negative,
            positive_examples: self.positive_examples,
            negative_examples: self.negative_examples,
            covered_surfaces: self.positive_surfaces.len() as u32,
            rejected_surfaces: self.negative_surfaces.len() as u32,
            threshold_micro: MIN_MARGIN_MICRO,
        };
        profile.threshold_micro =
            learned_margin_threshold(&profile, &self.positive_vectors, &self.negative_vectors);
        Some(profile)
    }
}

impl PhaseRuntime {
    fn readout(&self, operator: PhaseOperator, atoms: &[String]) -> PhaseReadout {
        let Some(profile) = self
            .profiles
            .iter()
            .find(|profile| profile.operator == operator)
        else {
            return PhaseReadout {
                package_loaded: true,
                ..PhaseReadout::default()
            };
        };
        let vector = phase_vector_from_atoms(atoms);
        let positive = max_coherence(&vector, &profile.positive).unwrap_or_default();
        let anti = max_coherence(&vector, &profile.negative).unwrap_or_default();
        let margin_micro = phase_micro(positive - anti);
        let evidence_ready = profile.positive_examples >= 2 && profile.negative_examples > 0;
        let verdict = if evidence_ready && margin_micro >= profile.threshold_micro {
            PhaseVerdict::Support
        } else if profile.negative_examples > 0
            && anti > positive
            && margin_micro <= -MIN_LEARNED_SUPPORT_MARGIN_MICRO
        {
            PhaseVerdict::Repel
        } else {
            PhaseVerdict::Unknown
        };
        PhaseReadout {
            package_loaded: true,
            operator_present: true,
            operator_promoted: profile.promoted,
            positive_micro: phase_micro(positive),
            anti_micro: phase_micro(anti),
            margin_micro,
            threshold_micro: profile.threshold_micro,
            positive_examples: profile.positive_examples,
            negative_examples: profile.negative_examples,
            positive_centers: profile.positive.len() as u8,
            anti_centers: profile.negative.len() as u8,
            covered_surfaces: profile.covered_surfaces,
            rejected_surfaces: profile.rejected_surfaces,
            verdict,
        }
    }

    fn readout_ablation(
        &self,
        operator: PhaseOperator,
        atoms: &[String],
        mode: PhaseAblation,
    ) -> PhaseVerdict {
        if mode == PhaseAblation::NoPhase {
            return PhaseVerdict::Unknown;
        }
        if mode == PhaseAblation::Full {
            return self.readout(operator, atoms).verdict;
        }
        let Some(profile) = self
            .profiles
            .iter()
            .find(|profile| profile.operator == operator)
        else {
            return PhaseVerdict::Unknown;
        };
        let mut vector = phase_vector_from_atoms(atoms);
        let (positive, anti) = match mode {
            PhaseAblation::ShuffledPhase => {
                vector.rotate_left(11 % CELLS);
                (
                    max_coherence(&vector, &profile.positive).unwrap_or_default(),
                    max_coherence(&vector, &profile.negative).unwrap_or_default(),
                )
            }
            PhaseAblation::MagnitudeOnly => (
                max_magnitude_overlap(&vector, &profile.positive).unwrap_or_default(),
                max_magnitude_overlap(&vector, &profile.negative).unwrap_or_default(),
            ),
            PhaseAblation::RandomCenter => (
                max_random_center_coherence(
                    &vector,
                    &profile.positive,
                    operator as u64 ^ 0x504f_5349_5449_5645,
                )
                .unwrap_or_default(),
                max_random_center_coherence(
                    &vector,
                    &profile.negative,
                    operator as u64 ^ 0x4e45_4741_5449_5645,
                )
                .unwrap_or_default(),
            ),
            PhaseAblation::WithoutAnti => (
                max_coherence(&vector, &profile.positive).unwrap_or_default(),
                0.0,
            ),
            PhaseAblation::Full | PhaseAblation::NoPhase => unreachable!(),
        };
        phase_verdict_from_scores(profile, positive, anti)
    }

    fn to_bytes(&self) -> Vec<u8> {
        let centers = self
            .profiles
            .iter()
            .map(|profile| profile.positive.len() + profile.negative.len())
            .sum::<usize>();
        let mut bytes = Vec::with_capacity(
            HEADER_BYTES
                + self.profiles.len() * PROFILE_HEADER_BYTES
                + centers * (CENTER_HEADER_BYTES + CELLS * CELL_BYTES),
        );
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&(CELLS as u16).to_le_bytes());
        bytes.extend_from_slice(&(self.profiles.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        for profile in &self.profiles {
            bytes.push(profile.operator as u8);
            bytes.push(profile.positive.len() as u8);
            bytes.push(profile.negative.len() as u8);
            bytes.push(u8::from(profile.promoted));
            bytes.extend_from_slice(&profile.positive_examples.to_le_bytes());
            bytes.extend_from_slice(&profile.negative_examples.to_le_bytes());
            bytes.extend_from_slice(&profile.covered_surfaces.to_le_bytes());
            bytes.extend_from_slice(&profile.rejected_surfaces.to_le_bytes());
            bytes.extend_from_slice(&(profile.threshold_micro as i32).to_le_bytes());
            for center in profile.positive.iter().chain(&profile.negative) {
                write_center(&mut bytes, center);
            }
        }
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < HEADER_BYTES || &bytes[..MAGIC.len()] != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid L2 phase package",
            ));
        }
        let cells = read_u16(bytes, 8)? as usize;
        let profile_count = read_u16(bytes, 10)? as usize;
        if cells != CELLS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported L2 phase width",
            ));
        }
        let mut offset = HEADER_BYTES;
        let mut profiles = Vec::with_capacity(profile_count);
        for _ in 0..profile_count {
            let header = bytes
                .get(offset..offset + PROFILE_HEADER_BYTES)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::UnexpectedEof, "truncated L2 phase profile")
                })?;
            let operator = PhaseOperator::from_code(header[0]).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "unknown L2 phase operator")
            })?;
            let positive_count = header[1] as usize;
            let negative_count = header[2] as usize;
            let promoted = header[3] == 1;
            if positive_count > MAX_POSITIVE_CENTERS || negative_count > MAX_ANTI_CENTERS {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "L2 phase center count exceeds runtime budget",
                ));
            }
            let positive_examples = u32::from_le_bytes(header[4..8].try_into().unwrap());
            let negative_examples = u32::from_le_bytes(header[8..12].try_into().unwrap());
            let covered_surfaces = u32::from_le_bytes(header[12..16].try_into().unwrap());
            let rejected_surfaces = u32::from_le_bytes(header[16..20].try_into().unwrap());
            let threshold_micro = i32::from_le_bytes(header[20..24].try_into().unwrap()) as i64;
            offset += PROFILE_HEADER_BYTES;
            let mut positive = Vec::with_capacity(positive_count);
            let mut negative = Vec::with_capacity(negative_count);
            for index in 0..positive_count + negative_count {
                let center = read_center(bytes, &mut offset)?;
                if index < positive_count {
                    positive.push(center);
                } else {
                    negative.push(center);
                }
            }
            profiles.push(PhaseProfile {
                operator,
                promoted,
                positive,
                negative,
                positive_examples,
                negative_examples,
                covered_surfaces,
                rejected_surfaces,
                threshold_micro,
            });
        }
        if offset != bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected trailing L2 phase data",
            ));
        }
        Ok(Self { profiles })
    }
}

fn phase_verdict_from_scores(profile: &PhaseProfile, positive: f32, anti: f32) -> PhaseVerdict {
    let margin_micro = phase_micro(positive - anti);
    let evidence_ready = profile.positive_examples >= 2 && profile.negative_examples > 0;
    if evidence_ready && margin_micro >= profile.threshold_micro {
        PhaseVerdict::Support
    } else if profile.negative_examples > 0
        && anti > positive
        && margin_micro <= -MIN_LEARNED_SUPPORT_MARGIN_MICRO
    {
        PhaseVerdict::Repel
    } else {
        PhaseVerdict::Unknown
    }
}

fn relation_atoms(
    original: &str,
    candidate: &str,
    operator: PhaseOperator,
) -> TransitionRelationAtoms {
    TransitionRelationAtoms::for_operator(original, candidate, operator)
}

fn add_cluster(centers: &mut Vec<PhaseCenter>, vector: &[PhaseCell], max_centers: usize) {
    let best = centers
        .iter()
        .enumerate()
        .map(|(index, center)| (index, phase_coherence(vector, &center.center)))
        .max_by(|left, right| left.1.total_cmp(&right.1));
    if let Some((index, coherence)) = best {
        if coherence >= CENTER_SPLIT_COHERENCE || centers.len() >= max_centers {
            let center = &mut centers[index];
            add_phase_vector(&mut center.sum, vector);
            center.center = phase_center_from_sum(&center.sum);
            center.support = center.support.saturating_add(1);
            return;
        }
    }
    centers.push(PhaseCenter {
        sum: vector.to_vec(),
        center: phase_center_from_sum(vector),
        support: 1,
    });
}

fn learned_margin_threshold(
    profile: &PhaseProfile,
    positives: &[Vec<PhaseCell>],
    negatives: &[Vec<PhaseCell>],
) -> i64 {
    if positives.is_empty() || negatives.is_empty() || profile.negative.is_empty() {
        return MAX_MARGIN_MICRO;
    }
    let mut positive_margins = positives
        .iter()
        .map(|vector| phase_margin_micro(profile, vector))
        .collect::<Vec<_>>();
    positive_margins.sort_unstable();
    let positive_floor = positive_margins[0];
    let negative_ceiling = negatives
        .iter()
        .map(|vector| phase_margin_micro(profile, vector))
        .max()
        .unwrap_or_default();
    let threshold = if positive_floor > negative_ceiling {
        negative_ceiling + (positive_floor - negative_ceiling) / 2
    } else {
        negative_ceiling.saturating_add(1_000)
    };
    threshold.clamp(MIN_LEARNED_SUPPORT_MARGIN_MICRO, MAX_MARGIN_MICRO)
}

fn phase_margin_micro(profile: &PhaseProfile, vector: &[PhaseCell]) -> i64 {
    let positive = max_coherence(vector, &profile.positive).unwrap_or_default();
    let negative = max_coherence(vector, &profile.negative).unwrap_or_default();
    phase_micro(positive - negative)
}

fn phase_vector_from_atoms(atoms: &[String]) -> Vec<PhaseCell> {
    let mut vector = vec![PhaseCell::default(); CELLS];
    for atom in atoms.iter().filter(|atom| causal_phase_atom(atom)) {
        let relation = atom.split_once(':').map_or(atom.as_str(), |(name, _)| name);
        for lane in 0..3_u64 {
            let cell_hash = stable_hash64(relation.as_bytes(), lane);
            let phase_hash = stable_hash64(atom.as_bytes(), lane ^ 0x0050_4841_5345);
            let cell = (cell_hash as usize) % CELLS;
            let angle =
                (mix64_golden(phase_hash ^ 0x9e37_79b9_7f4a_7c15) as f32 / u64::MAX as f32) * TAU;
            vector[cell].re += angle.cos();
            vector[cell].im += angle.sin();
        }
    }
    phase_center_from_sum(&vector)
}

fn causal_phase_atom(atom: &str) -> bool {
    !atom.starts_with("word-count:")
        && !atom.starts_with("len:")
        && !atom.starts_with("prefix:")
        && !atom.starts_with("suffix:")
        && !atom.starts_with("verified:")
}

fn phase_circuit_key(atoms: &[String]) -> String {
    atoms
        .iter()
        .filter(|atom| causal_phase_atom(atom))
        .cloned()
        .collect::<Vec<_>>()
        .join("|")
}

fn concrete_surface_id(original: &str, candidate: &str) -> u64 {
    let mut bytes = original.trim().as_bytes().to_vec();
    bytes.push(0xff);
    bytes.extend_from_slice(candidate.trim().as_bytes());
    stable_hash64(&bytes, 0x0053_5552_4641_4345)
}

fn add_phase_vector(target: &mut [PhaseCell], source: &[PhaseCell]) {
    for (target, source) in target.iter_mut().zip(source) {
        target.re += source.re;
        target.im += source.im;
    }
}

fn phase_center_from_sum(values: &[PhaseCell]) -> Vec<PhaseCell> {
    values.iter().copied().map(phase_unit).collect()
}

fn phase_unit(value: PhaseCell) -> PhaseCell {
    let norm = value.re.hypot(value.im);
    if norm == 0.0 {
        PhaseCell::default()
    } else {
        PhaseCell {
            re: value.re / norm,
            im: value.im / norm,
        }
    }
}

fn phase_coherence(vector: &[PhaseCell], center: &[PhaseCell]) -> f32 {
    let mut score = 0.0;
    let mut active = 0usize;
    for (left, right) in vector.iter().zip(center) {
        if left.re != 0.0 || left.im != 0.0 {
            active += 1;
            score += left.re * right.re + left.im * right.im;
        }
    }
    if active == 0 {
        0.0
    } else {
        score / active as f32
    }
}

fn max_coherence(vector: &[PhaseCell], centers: &[PhaseCenter]) -> Option<f32> {
    centers
        .iter()
        .map(|center| phase_coherence(vector, &center.center))
        .max_by(f32::total_cmp)
}

fn max_magnitude_overlap(vector: &[PhaseCell], centers: &[PhaseCenter]) -> Option<f32> {
    centers
        .iter()
        .map(|center| magnitude_overlap(vector, &center.center))
        .max_by(f32::total_cmp)
}

fn magnitude_overlap(vector: &[PhaseCell], center: &[PhaseCell]) -> f32 {
    let mut active = 0usize;
    let mut overlap = 0usize;
    for (left, right) in vector.iter().zip(center) {
        if left.re != 0.0 || left.im != 0.0 {
            active += 1;
            overlap += usize::from(right.re != 0.0 || right.im != 0.0);
        }
    }
    if active == 0 {
        0.0
    } else {
        overlap as f32 / active as f32
    }
}

fn randomize_active_phases(vector: &mut [PhaseCell], seed: u64) {
    for (index, cell) in vector.iter_mut().enumerate() {
        if cell.re == 0.0 && cell.im == 0.0 {
            continue;
        }
        let hash = mix64_golden(seed ^ (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let angle = (hash as f32 / u64::MAX as f32) * TAU;
        cell.re = angle.cos();
        cell.im = angle.sin();
    }
}

fn max_random_center_coherence(
    vector: &[PhaseCell],
    centers: &[PhaseCenter],
    seed: u64,
) -> Option<f32> {
    centers
        .iter()
        .enumerate()
        .map(|(index, center)| {
            let mut randomized = center.center.clone();
            randomize_active_phases(
                &mut randomized,
                seed ^ (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
            );
            phase_coherence(vector, &randomized)
        })
        .max_by(f32::total_cmp)
}

fn phase_micro(value: f32) -> i64 {
    (value.clamp(-1.0, 1.0) * 1_000_000.0).round() as i64
}

fn stable_hash64(bytes: &[u8], lane: u64) -> u64 {
    let hash = bytes.iter().fold(
        0xcbf2_9ce4_8422_2325_u64 ^ lane.wrapping_mul(0x1000_0000_01b3),
        |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3),
    );
    mix64_golden(hash)
}

fn write_center(bytes: &mut Vec<u8>, center: &PhaseCenter) {
    bytes.extend_from_slice(&center.support.to_le_bytes());
    for cell in &center.center {
        bytes.extend_from_slice(&quantize(cell.re).to_le_bytes());
        bytes.extend_from_slice(&quantize(cell.im).to_le_bytes());
    }
}

fn read_center(bytes: &[u8], offset: &mut usize) -> io::Result<PhaseCenter> {
    let support = read_u32(bytes, *offset)?;
    *offset += CENTER_HEADER_BYTES;
    let mut center = Vec::with_capacity(CELLS);
    for _ in 0..CELLS {
        let re = read_i16(bytes, *offset)?;
        let im = read_i16(bytes, *offset + 2)?;
        *offset += CELL_BYTES;
        center.push(PhaseCell {
            re: f32::from(re) / PHASE_SCALE,
            im: f32::from(im) / PHASE_SCALE,
        });
    }
    Ok(PhaseCenter {
        sum: center.clone(),
        center,
        support,
    })
}

fn quantize(value: f32) -> i16 {
    (value * PHASE_SCALE)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

fn read_u16(bytes: &[u8], offset: usize) -> io::Result<u16> {
    Ok(u16::from_le_bytes(read_array(bytes, offset)?))
}

fn read_u32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(read_array(bytes, offset)?))
}

fn read_i16(bytes: &[u8], offset: usize) -> io::Result<i16> {
    Ok(i16::from_le_bytes(read_array(bytes, offset)?))
}

fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> io::Result<[u8; N]> {
    bytes
        .get(offset..offset + N)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "truncated L2 phase data"))?
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid L2 phase field"))
}

fn is_trainable_pair(original: &str, candidate: &str) -> bool {
    let original = original.trim();
    let candidate = candidate.trim();
    !original.is_empty()
        && !candidate.is_empty()
        && original.chars().count() <= 96
        && candidate.chars().count() <= 96
        && !original.chars().any(char::is_control)
        && !candidate.chars().any(char::is_control)
}

fn is_structural_negative(entry: &L2PhaseTrainingEntry, claimed_operator: PhaseOperator) -> bool {
    let observed_operator = PhaseOperator::infer(&entry.original, &entry.candidate, "");
    observed_operator != claimed_operator
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        original: &str,
        candidate: &str,
        operation: &str,
        accepted: bool,
    ) -> L2PhaseTrainingEntry {
        L2PhaseTrainingEntry {
            original: original.to_string(),
            candidate: candidate.to_string(),
            operation: operation.to_string(),
            accepted,
            count: 1,
        }
    }

    fn transposition_runtime() -> PhaseRuntime {
        train_phase_runtime([
            entry("пукнт", "пункт", "transposition", true),
            entry("слвоо", "слово", "transposition", true),
            entry("копрус", "корпус", "transposition", true),
            entry("пукнт", "пукат", "transposition", false),
            entry("слвоо", "слава", "transposition", false),
            entry("копрус", "компас", "transposition", false),
        ])
        .expect("phase field trains")
    }

    #[test]
    fn per_operator_phase_field_transfers_to_unseen_surface() {
        let runtime = transposition_runtime();
        let operator = PhaseOperator::AdjacentTransposition;
        let relation = relation_atoms("гркоинг", "грокинг", operator);
        let readout = runtime.readout(operator, relation.atoms());

        assert!(readout.operator_present);
        assert_eq!(readout.positive_centers, 1);
        assert!(readout.anti_centers > 0);
        assert!(readout.positive_examples >= 3);
        assert_eq!(readout.verdict, PhaseVerdict::Support, "{readout:?}");
    }

    #[test]
    fn anti_center_repels_same_operator_near_miss() {
        let runtime = transposition_runtime();
        let operator = PhaseOperator::AdjacentTransposition;
        let relation = relation_atoms("пукнт", "пукат", operator);
        let readout = runtime.readout(operator, relation.atoms());

        assert_eq!(readout.verdict, PhaseVerdict::Repel, "{readout:?}");
        assert!(readout.anti_micro > readout.positive_micro);
    }

    #[test]
    fn phase_package_roundtrips_multiple_operator_profiles() {
        let runtime = train_phase_runtime([
            entry("пукнт", "пункт", "transposition", true),
            entry("слвоо", "слово", "transposition", true),
            entry("пукнт", "пукат", "transposition", false),
            entry("ghbdtn", "привет", "layout", true),
            entry("djn", "вот", "layout", true),
            entry("ghbdtn", "ghbdtn", "layout", false),
        ])
        .expect("phase field trains");
        let bytes = runtime.to_bytes();
        let loaded = PhaseRuntime::from_bytes(&bytes).expect("phase field loads");

        assert_eq!(loaded.profiles.len(), 2);
        assert_eq!(loaded.to_bytes(), bytes);
    }

    #[test]
    fn phase_training_atoms_do_not_store_concrete_words() {
        let relation = relation_atoms("мы пукнт", "мы пункт", PhaseOperator::AdjacentTransposition);

        assert!(relation.atoms().iter().all(|atom| !atom.contains("пукнт")));
        assert!(relation.atoms().iter().all(|atom| !atom.contains("пункт")));
    }

    #[test]
    fn verifier_outcome_is_not_a_causal_phase_input() {
        let relation = relation_atoms("пукнт", "пункт", PhaseOperator::AdjacentTransposition);
        let mut flipped = relation.atoms().to_vec();
        replace_atom(&mut flipped, "verified:", "verified:false");

        assert!(relation
            .atoms()
            .iter()
            .any(|atom| atom.starts_with("verified:")));
        assert_eq!(
            phase_vector_from_atoms(relation.atoms()),
            phase_vector_from_atoms(&flipped)
        );
        assert_eq!(
            phase_circuit_key(relation.atoms()),
            phase_circuit_key(&flipped)
        );
    }

    #[test]
    fn typed_counterfactuals_create_anti_wave_without_negative_words() {
        let runtime = train_phase_runtime([
            entry("кот", "кит", "substitution", true),
            entry("дом", "дым", "substitution", true),
            entry("луг", "лог", "substitution", true),
        ])
        .expect("phase field trains from positive surfaces");
        let operator = PhaseOperator::LetterSubstitution;
        let supported = runtime.readout(operator, relation_atoms("мак", "мок", operator).atoms());
        let mismatched = runtime.readout(operator, relation_atoms("мак", "мака", operator).atoms());

        assert_eq!(supported.verdict, PhaseVerdict::Support, "{supported:?}");
        assert_eq!(mismatched.verdict, PhaseVerdict::Repel, "{mismatched:?}");
        assert!(supported.negative_examples > 0);
        assert!(supported.anti_centers > 0);
    }
}
