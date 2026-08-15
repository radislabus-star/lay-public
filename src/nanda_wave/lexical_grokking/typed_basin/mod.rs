//! Shared typed restoration basin with proof and physical-runtime owners.

mod exact_reverse;
mod implicit_forward;
#[cfg(any(test, feature = "lexical-compiler"))]
mod oracle;
#[cfg(any(test, feature = "lexical-compiler"))]
mod package_dependencies;
#[cfg(any(test, feature = "lexical-compiler"))]
mod quality;
mod runtime;
mod settlement;
mod support;

pub(super) use runtime::TypedBasinRuntime;
pub(super) use support::ExactSupportField;

#[cfg(feature = "lexical-compiler")]
pub use quality::{diagnose_l1_typed_basin_quality_class, prove_l1_typed_basin_quality};

use std::collections::BTreeMap;
#[cfg(any(test, feature = "lexical-compiler"))]
use std::fs::File;
#[cfg(any(test, feature = "lexical-compiler"))]
use std::io::{self, Read, Seek, SeekFrom};
#[cfg(any(test, feature = "lexical-compiler"))]
use std::path::Path;
#[cfg(any(test, feature = "lexical-compiler"))]
use std::time::Instant;

#[cfg(any(test, feature = "lexical-compiler"))]
use sha2::{Digest, Sha256};

use super::atoms::AtomChannel;
#[cfg(any(test, feature = "lexical-compiler"))]
use super::forward_decoder_index::ForwardDecoderIndex;
#[cfg(any(test, feature = "lexical-compiler"))]
use super::proof::{corpus_words_from_lines, prepare_fixed_heldout_cases, FixedHeldoutCase};
use super::runtime::{LexicalGrokkingMemory, ObservedAtom};
#[cfg(any(test, feature = "lexical-compiler"))]
use super::typed_edit_traversal::phase7d_terminal_evidence;
#[cfg(any(test, feature = "lexical-compiler"))]
use implicit_forward::{candidates_equal, reconstruct_candidate};
#[cfg(any(test, feature = "lexical-compiler"))]
use oracle::{OracleParityMetrics, V8QueryOracle};
#[cfg(any(test, feature = "lexical-compiler"))]
use package_dependencies::PackageDependencyAudit;
#[cfg(any(test, feature = "lexical-compiler"))]
use settlement::{evaluate_settlement_case, SettlementAggregate, SettlementCaseResult};
#[cfg(any(test, feature = "lexical-compiler"))]
use support::ExactSupportMetrics;

#[cfg(any(test, feature = "lexical-compiler"))]
const PACKAGE_LIMIT_BYTES: u64 = 195 * 1024 * 1024;
#[cfg(any(test, feature = "lexical-compiler"))]
const IMPLICIT_LATENCY_LIMIT_US: u64 = 2_500;
#[cfg(any(test, feature = "lexical-compiler"))]
const COMBINED_LATENCY_LIMIT_US: u64 = 5_000;
#[cfg(any(test, feature = "lexical-compiler"))]
const FIXED_DAMAGE_CLASS_COUNT: usize = 13;

#[cfg(any(test, feature = "lexical-compiler"))]
#[derive(Default)]
struct ClassMetrics {
    cases: usize,
    target_retained: usize,
    implicit_target_retained: usize,
    candidate_permutation_parity: usize,
    typed_terminal_ids: u64,
    maximum_typed_terminal_ids: usize,
    typed_states_expanded: u64,
    maximum_typed_queue: usize,
    implicit_relations: u64,
    parity: OracleParityMetrics,
    typed_us: Vec<u64>,
    implicit_us: Vec<u64>,
    combined_us: Vec<u64>,
    oracle_us: Vec<u64>,
    settlement: SettlementAggregate,
}

#[cfg(any(test, feature = "lexical-compiler"))]
struct CaseOutcome {
    class: &'static str,
    target_retained: bool,
    implicit_target_retained: bool,
    candidate_permutation_parity: bool,
    typed_terminal_ids: usize,
    typed_states_expanded: u64,
    typed_queue_peak: usize,
    implicit_relations: usize,
    parity: OracleParityMetrics,
    typed_us: u64,
    implicit_us: u64,
    combined_us: u64,
    oracle_us: u64,
    settlement: Option<SettlementCaseResult>,
}

#[cfg(any(test, feature = "lexical-compiler"))]
#[derive(Clone, Copy)]
struct V8Layout {
    package_bytes: u64,
    base_bytes: u64,
}

#[cfg(any(test, feature = "lexical-compiler"))]
impl ClassMetrics {
    fn record(&mut self, outcome: CaseOutcome) {
        self.cases += 1;
        self.target_retained += usize::from(outcome.target_retained);
        self.implicit_target_retained += usize::from(outcome.implicit_target_retained);
        self.candidate_permutation_parity += usize::from(outcome.candidate_permutation_parity);
        self.typed_terminal_ids = self
            .typed_terminal_ids
            .saturating_add(outcome.typed_terminal_ids as u64);
        self.maximum_typed_terminal_ids = self
            .maximum_typed_terminal_ids
            .max(outcome.typed_terminal_ids);
        self.typed_states_expanded = self
            .typed_states_expanded
            .saturating_add(outcome.typed_states_expanded);
        self.maximum_typed_queue = self.maximum_typed_queue.max(outcome.typed_queue_peak);
        self.implicit_relations = self
            .implicit_relations
            .saturating_add(outcome.implicit_relations as u64);
        add_parity(&mut self.parity, outcome.parity);
        self.typed_us.push(outcome.typed_us);
        self.implicit_us.push(outcome.implicit_us);
        self.combined_us.push(outcome.combined_us);
        self.oracle_us.push(outcome.oracle_us);
        if let Some(settlement) = outcome.settlement.as_ref() {
            self.settlement.record(settlement);
        }
    }

    fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "cases": self.cases,
            "target_retained": self.target_retained,
            "implicit_target_retained": self.implicit_target_retained,
            "candidate_permutation_parity": self.candidate_permutation_parity,
            "typed_terminal_ids": self.typed_terminal_ids,
            "maximum_typed_terminal_ids": self.maximum_typed_terminal_ids,
            "typed_states_expanded": self.typed_states_expanded,
            "maximum_typed_queue": self.maximum_typed_queue,
            "implicit_relations": self.implicit_relations,
            "relation_parity": parity_report(self.parity),
            "timing": {
                "typed_us_p50": percentile(&self.typed_us, 50),
                "typed_us_p95": percentile(&self.typed_us, 95),
                "typed_us_p99": percentile(&self.typed_us, 99),
                "typed_us_max": maximum(&self.typed_us),
                "implicit_us_p50": percentile(&self.implicit_us, 50),
                "implicit_us_p95": percentile(&self.implicit_us, 95),
                "implicit_us_p99": percentile(&self.implicit_us, 99),
                "implicit_us_max": maximum(&self.implicit_us),
                "combined_us_p50": percentile(&self.combined_us, 50),
                "combined_us_p95": percentile(&self.combined_us, 95),
                "combined_us_p99": percentile(&self.combined_us, 99),
                "combined_us_max": maximum(&self.combined_us),
                "oracle_us_p99": percentile(&self.oracle_us, 99),
                "oracle_us_max": maximum(&self.oracle_us),
            },
            "nonlinear_settlement": self.settlement.report(),
        })
    }
}

#[cfg(any(test, feature = "lexical-compiler"))]
pub fn prove_l1_typed_basin_implicit_forward(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
) -> io::Result<serde_json::Value> {
    let wall_started = Instant::now();
    let package_sha256_before = file_sha256(package_path)?;
    let layout = read_v8_layout(package_path)?;
    let words = corpus_words_from_lines(&std::fs::read_to_string(corpus_path)?, max_words);

    let load_started = Instant::now();
    let memory = LexicalGrokkingMemory::load(package_path).map_err(io::Error::other)?;
    let package_load_ms = load_started.elapsed().as_millis();
    if words.len() != memory.package.centers.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "corpus/package terminal count differs: {} != {}",
                words.len(),
                memory.package.centers.len()
            ),
        ));
    }

    let decoder_started = Instant::now();
    let decoder_index = ForwardDecoderIndex::build(&memory.package).map_err(io::Error::other)?;
    let decoder_index_ms = decoder_started.elapsed().as_millis();

    let support_started = Instant::now();
    let support = ExactSupportField::rebuild(&memory.package, &words).map_err(io::Error::other)?;
    let support_rebuild_ms = support_started.elapsed().as_millis();
    let package_dependencies = PackageDependencyAudit::inspect(&memory.package);
    let package_dependencies_resolved = package_dependencies.resolved();

    let cases = prepare_fixed_heldout_cases(&words, heldout_per_class, 0)?;
    let mut classes = BTreeMap::<&'static str, ClassMetrics>::new();
    for case in &cases {
        let outcome = evaluate_case(
            &memory,
            &decoder_index,
            &support,
            case,
            package_dependencies_resolved,
        )
        .map_err(io::Error::other)?;
        classes.entry(outcome.class).or_default().record(outcome);
    }

    let mut totals = ClassMetrics::default();
    for metrics in classes.values() {
        totals.cases = totals.cases.saturating_add(metrics.cases);
        totals.target_retained = totals
            .target_retained
            .saturating_add(metrics.target_retained);
        totals.implicit_target_retained = totals
            .implicit_target_retained
            .saturating_add(metrics.implicit_target_retained);
        totals.candidate_permutation_parity = totals
            .candidate_permutation_parity
            .saturating_add(metrics.candidate_permutation_parity);
        totals.typed_terminal_ids = totals
            .typed_terminal_ids
            .saturating_add(metrics.typed_terminal_ids);
        totals.maximum_typed_terminal_ids = totals
            .maximum_typed_terminal_ids
            .max(metrics.maximum_typed_terminal_ids);
        totals.typed_states_expanded = totals
            .typed_states_expanded
            .saturating_add(metrics.typed_states_expanded);
        totals.maximum_typed_queue = totals.maximum_typed_queue.max(metrics.maximum_typed_queue);
        totals.implicit_relations = totals
            .implicit_relations
            .saturating_add(metrics.implicit_relations);
        add_parity(&mut totals.parity, metrics.parity);
        totals.typed_us.extend_from_slice(&metrics.typed_us);
        totals.implicit_us.extend_from_slice(&metrics.implicit_us);
        totals.combined_us.extend_from_slice(&metrics.combined_us);
        totals.oracle_us.extend_from_slice(&metrics.oracle_us);
        totals.settlement.merge(&metrics.settlement);
    }

    let package_sha256_after = file_sha256(package_path)?;
    let package_unchanged = package_sha256_before == package_sha256_after;
    let projected_package_bytes = layout
        .base_bytes
        .saturating_add(support.metrics.projected_overflow_bytes as u64);
    let support_pass = support.metrics.centers_decoded == memory.package.centers.len()
        && support.metrics.corpus_surface_mismatches == 0
        && support.metrics.stored_support_mismatches == 0
        && projected_package_bytes <= PACKAGE_LIMIT_BYTES;
    let relation_pass = totals.parity.relation_mismatches() == 0;
    let activation_pass = totals.parity.activation_mismatches() == 0;
    let expected_cases = heldout_per_class
        .checked_mul(FIXED_DAMAGE_CLASS_COUNT)
        .ok_or_else(|| io::Error::other("fixed heldout denominator exceeds usize"))?;
    let denominator_pass = classes.len() == FIXED_DAMAGE_CLASS_COUNT
        && totals.cases == expected_cases
        && classes
            .values()
            .all(|metrics| metrics.cases == heldout_per_class);
    let typed_pass = denominator_pass
        && totals.target_retained == totals.cases
        && totals.implicit_target_retained == totals.cases
        && totals.candidate_permutation_parity == totals.cases;
    let implicit_latency_pass = maximum(&totals.implicit_us) <= IMPLICIT_LATENCY_LIMIT_US;
    let combined_latency_pass = maximum(&totals.combined_us) <= COMBINED_LATENCY_LIMIT_US;
    let settlement_denominator_pass = totals.settlement.case_count() == totals.cases
        && totals.settlement.executed_count() == totals.cases;
    let settlement_reverse_pass = totals.settlement.reverse_parity_passes();
    let settlement_pass = settlement_denominator_pass && totals.settlement.passes();
    let verdict = if !package_unchanged {
        "REJECT_PACKAGE_ISOLATION"
    } else if !support_pass {
        "REJECT_SUPPORT_REPRESENTATION"
    } else if !relation_pass {
        "REJECT_IMPLICIT_RELATION_PARITY"
    } else if !activation_pass {
        "REJECT_ACTIVATION_PARITY"
    } else if !typed_pass {
        "REJECT_TYPED_ADMISSION"
    } else if !implicit_latency_pass || !combined_latency_pass {
        "REJECT_FEASIBILITY"
    } else if !package_dependencies_resolved {
        "REJECT_A2_PACKAGE_DEPENDENCIES"
    } else if !settlement_reverse_pass {
        "REJECT_A2_REVERSE_PARITY"
    } else if !settlement_pass {
        "REJECT_A2_SETTLEMENT_PARITY"
    } else {
        "PASS_A0_A1_B0_A2"
    };

    let class_report = classes
        .iter()
        .map(|(class, metrics)| ((*class).to_string(), metrics.report()))
        .collect::<serde_json::Map<_, _>>();
    Ok(serde_json::json!({
        "schema": "lay.l11.typed-basin-implicit-forward-proof.v2",
        "verdict": verdict,
        "artifact": {
            "corpus": corpus_path.display().to_string(),
            "package": package_path.display().to_string(),
            "package_bytes": layout.package_bytes,
            "compact_base_bytes": layout.base_bytes,
            "package_sha256_before": package_sha256_before,
            "package_sha256_after": package_sha256_after,
            "package_bytes_unchanged": package_unchanged,
            "centers": memory.package.centers.len(),
            "atoms": memory.package.atoms.len(),
        },
        "configuration": {
            "heldout_per_class": heldout_per_class,
            "expected_classes": FIXED_DAMAGE_CLASS_COUNT,
            "expected_cases": expected_cases,
            "classes": classes.len(),
            "cases": totals.cases,
            "implicit_latency_limit_us": IMPLICIT_LATENCY_LIMIT_US,
            "combined_latency_limit_us": COMBINED_LATENCY_LIMIT_US,
            "package_limit_bytes": PACKAGE_LIMIT_BYTES,
            "runtime_authority_changed": false,
            "package_format_changed": false,
            "settlement_executed": totals.settlement.executed_count() != 0,
        },
        "exact_support": support_report(
            support.metrics,
            projected_package_bytes,
            support_rebuild_ms,
        ),
        "package_dependencies": package_dependencies.report(),
        "classes": class_report,
        "aggregate": totals.report(),
        "gates": {
            "support_sufficiency": support_pass,
            "relation_parity": relation_pass,
            "activation_parity": activation_pass,
            "fixed_denominator_complete": denominator_pass,
            "typed_target_and_permutation_parity": typed_pass,
            "implicit_latency_max_le_2500_us": implicit_latency_pass,
            "combined_latency_max_le_5000_us": combined_latency_pass,
            "package_projection_le_195_mib": projected_package_bytes <= PACKAGE_LIMIT_BYTES,
            "package_isolation": package_unchanged,
            "a2_package_dependencies_resolved": package_dependencies_resolved,
            "a2_exact_reverse_parity": settlement_reverse_pass,
            "a2_fixed_denominator_complete": settlement_denominator_pass,
            "a2_nonlinear_settlement_parity": settlement_pass,
        },
        "setup_timing": {
            "package_load_ms": package_load_ms,
            "decoder_index_ms": decoder_index_ms,
            "exact_support_rebuild_ms": support_rebuild_ms,
            "wall_ms": wall_started.elapsed().as_millis(),
        },
        "claim_boundary": {
            "typed_restoration_admission_tested": true,
            "implicit_forward_state_tested": true,
            "nonlinear_settlement_tested": settlement_denominator_pass,
            "full_quality_matrix_tested": heldout_per_class == 20_000,
            "full_quality_claimed": false,
            "runtime_latency_claimed": false,
            "physical_package_roundtrip_tested": false,
            "runtime_authority_changed": false,
            "installed_runtime_changed": false,
        }
    }))
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn evaluate_case(
    memory: &LexicalGrokkingMemory,
    decoder_index: &ForwardDecoderIndex,
    support: &ExactSupportField,
    case: &FixedHeldoutCase,
    settlement_enabled: bool,
) -> Result<CaseOutcome, String> {
    let typed_started = Instant::now();
    let typed =
        phase7d_terminal_evidence(decoder_index, &memory.package.decoder_nodes, &case.surface)?;
    let typed_us = elapsed_us(typed_started);

    let implicit_started = Instant::now();
    let observed = observed_lexical_atoms(memory, &case.surface);
    let mut candidates = typed
        .terminal_ids
        .iter()
        .copied()
        .map(|terminal_id| reconstruct_candidate(&memory.package, support, &observed, terminal_id))
        .collect::<Result<Vec<_>, String>>()?;
    candidates.sort_unstable_by_key(|candidate| candidate.terminal_id);
    let implicit_us = elapsed_us(implicit_started);
    let combined_us = typed_us.saturating_add(implicit_us);

    let mut reversed_ids = typed.terminal_ids.clone();
    reversed_ids.reverse();
    let mut permuted = reversed_ids
        .into_iter()
        .map(|terminal_id| reconstruct_candidate(&memory.package, support, &observed, terminal_id))
        .collect::<Result<Vec<_>, String>>()?;
    permuted.sort_unstable_by_key(|candidate| candidate.terminal_id);
    let candidate_permutation_parity = candidates_equal(&candidates, &permuted);

    let oracle_started = Instant::now();
    let oracle = V8QueryOracle::build(memory, &observed, &candidates)?;
    let parity = oracle.compare(&observed, &candidates);
    let oracle_us = elapsed_us(oracle_started);
    let implicit_relations = candidates
        .iter()
        .map(|candidate| candidate.relations.len())
        .sum();
    let target_retained = typed.terminal_ids.binary_search(&case.terminal_id).is_ok();
    let implicit_target_retained = candidates
        .binary_search_by_key(&case.terminal_id, |candidate| candidate.terminal_id)
        .is_ok();
    let settlement = settlement_enabled
        .then(|| evaluate_settlement_case(memory, support, &case.surface, &candidates))
        .transpose()?;
    Ok(CaseOutcome {
        class: case.class,
        target_retained,
        implicit_target_retained,
        candidate_permutation_parity,
        typed_terminal_ids: typed.terminal_ids.len(),
        typed_states_expanded: typed.states_expanded,
        typed_queue_peak: typed.queue_peak,
        implicit_relations,
        parity,
        typed_us,
        implicit_us,
        combined_us,
        oracle_us,
        settlement,
    })
}

fn observed_lexical_atoms(
    memory: &LexicalGrokkingMemory,
    surface: &str,
) -> BTreeMap<u32, ObservedAtom> {
    memory
        .resolve_surface(surface)
        .into_iter()
        .filter(|(_, atom)| atom.channel != AtomChannel::CharacterAnchor)
        .collect()
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn add_parity(target: &mut OracleParityMetrics, value: OracleParityMetrics) {
    target.candidates_compared = target
        .candidates_compared
        .saturating_add(value.candidates_compared);
    target.canonical_relations_expected = target
        .canonical_relations_expected
        .saturating_add(value.canonical_relations_expected);
    target.implicit_relations_compared = target
        .implicit_relations_compared
        .saturating_add(value.implicit_relations_compared);
    target.implicit_relations_missing_from_v8 = target
        .implicit_relations_missing_from_v8
        .saturating_add(value.implicit_relations_missing_from_v8);
    target.v8_relations_missing_implicitly = target
        .v8_relations_missing_implicitly
        .saturating_add(value.v8_relations_missing_implicitly);
    target.canonical_relations_missing_from_v8 = target
        .canonical_relations_missing_from_v8
        .saturating_add(value.canonical_relations_missing_from_v8);
    target.implicit_relations_outside_canonical_surface = target
        .implicit_relations_outside_canonical_surface
        .saturating_add(value.implicit_relations_outside_canonical_surface);
    target.relation_state_mismatches = target
        .relation_state_mismatches
        .saturating_add(value.relation_state_mismatches);
    target.activation_mass_mismatches = target
        .activation_mass_mismatches
        .saturating_add(value.activation_mass_mismatches);
    target.activation_hits_mismatches = target
        .activation_hits_mismatches
        .saturating_add(value.activation_hits_mismatches);
    target.activation_surface_hits_mismatches = target
        .activation_surface_hits_mismatches
        .saturating_add(value.activation_surface_hits_mismatches);
    target.activation_keyboard_hits_mismatches = target
        .activation_keyboard_hits_mismatches
        .saturating_add(value.activation_keyboard_hits_mismatches);
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn parity_report(metrics: OracleParityMetrics) -> serde_json::Value {
    serde_json::json!({
        "candidates_compared": metrics.candidates_compared,
        "canonical_relations_expected": metrics.canonical_relations_expected,
        "implicit_relations_compared": metrics.implicit_relations_compared,
        "implicit_relations_missing_from_v8": metrics.implicit_relations_missing_from_v8,
        "v8_relations_missing_implicitly": metrics.v8_relations_missing_implicitly,
        "canonical_relations_missing_from_v8": metrics.canonical_relations_missing_from_v8,
        "implicit_relations_outside_canonical_surface": metrics.implicit_relations_outside_canonical_surface,
        "relation_state_mismatches": metrics.relation_state_mismatches,
        "activation_mass_mismatches": metrics.activation_mass_mismatches,
        "activation_hits_mismatches": metrics.activation_hits_mismatches,
        "activation_surface_hits_mismatches": metrics.activation_surface_hits_mismatches,
        "activation_keyboard_hits_mismatches": metrics.activation_keyboard_hits_mismatches,
    })
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn support_report(
    metrics: ExactSupportMetrics,
    projected_package_bytes: u64,
    rebuild_ms: u128,
) -> serde_json::Value {
    serde_json::json!({
        "centers_decoded": metrics.centers_decoded,
        "corpus_surface_mismatches": metrics.corpus_surface_mismatches,
        "encoded_atom_occurrences": metrics.encoded_atom_occurrences,
        "stored_saturated_atoms": metrics.stored_saturated_atoms,
        "exact_overflow_atoms": metrics.exact_overflow_atoms,
        "stored_support_mismatches": metrics.stored_support_mismatches,
        "maximum_exact_support": metrics.maximum_exact_support,
        "projected_overflow_bytes": metrics.projected_overflow_bytes,
        "projected_compact_package_bytes": projected_package_bytes,
        "rebuild_ms": rebuild_ms,
    })
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn elapsed_us(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let index = (sorted.len() - 1).saturating_mul(percentile.min(100)) / 100;
    sorted[index]
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn maximum(values: &[u64]) -> u64 {
    values.iter().copied().max().unwrap_or_default()
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn read_v8_layout(path: &Path) -> io::Result<V8Layout> {
    let mut file = File::open(path)?;
    let mut header = [0_u8; 128];
    file.read_exact(&mut header)?;
    if header.get(..8) != Some(b"LAYL1V8\0") || read_u32(&header, 8)? != 8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Phase 8I proof requires a V8 package",
        ));
    }
    let package_bytes = read_u64(&header, 16)?;
    let base_offset = read_u64(&header, 24)?;
    let base_bytes = read_u64(&header, 32)?;
    if package_bytes != file.metadata()?.len()
        || base_offset.saturating_add(base_bytes) > package_bytes
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid V8 base layout",
        ));
    }
    file.seek(SeekFrom::Start(base_offset))?;
    let mut base_magic = [0_u8; 8];
    file.read_exact(&mut base_magic)?;
    if &base_magic != b"LAYL1C07" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Phase 8I V8 base is not compact depth-0 V7",
        ));
    }
    Ok(V8Layout {
        package_bytes,
        base_bytes,
    })
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn read_u32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "truncated u32"))?;
    Ok(u32::from_le_bytes(raw.try_into().expect("four bytes")))
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn read_u64(bytes: &[u8], offset: usize) -> io::Result<u64> {
    let raw = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "truncated u64"))?;
    Ok(u64::from_le_bytes(raw.try_into().expect("eight bytes")))
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn file_sha256(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::compiler::{
        compile_with_policy, ForwardPostingPolicy,
    };
    use crate::nanda_wave::lexical_grokking::training_corpus::TrainingWord;

    fn tiny_memory() -> LexicalGrokkingMemory {
        let words = ["aaaa", "abaa", "каска", "касса", "test", "text"]
            .into_iter()
            .enumerate()
            .map(|(terminal_id, surface)| TrainingWord {
                terminal_id: terminal_id as u32,
                surface: surface.to_string(),
                training_surfaces: Vec::new(),
            })
            .collect::<Vec<_>>();
        let package = compile_with_policy(&words, ForwardPostingPolicy::Complete)
            .expect("compile tiny complete package")
            .package;
        LexicalGrokkingMemory::from_package(package)
    }

    fn tiny_surfaces(memory: &LexicalGrokkingMemory) -> Vec<String> {
        (0..memory.package.terminal_count())
            .map(|terminal_id| {
                memory
                    .decode_terminal(terminal_id)
                    .expect("decode terminal")
            })
            .collect()
    }

    fn tiny_support(memory: &LexicalGrokkingMemory) -> ExactSupportField {
        ExactSupportField::rebuild(&memory.package, &tiny_surfaces(memory))
            .expect("rebuild support")
    }

    fn tiny_a2_memory() -> LexicalGrokkingMemory {
        let mut memory = tiny_memory();
        memory.package.anti_centers.clear();
        memory.package.pair_profiles.clear();
        memory.package.pair_centers.clear();
        memory.package.center_phase_profiles.clear();
        memory.package.positive_subcenters.clear();
        memory.package.anti_subcenters.clear();
        memory.package.hard_negative_subcenters.clear();
        memory.package.ambiguity_subcenters.clear();
        memory.package.keyboard_geometry_units.clear();
        for center in &mut memory.package.centers {
            center.anti_start = 0;
            center.anti_count = 0;
        }
        memory
    }

    #[test]
    fn exact_support_counts_occurrences_not_posting_degree() {
        let memory = tiny_memory();
        let support = tiny_support(&memory);
        assert_eq!(support.metrics.stored_support_mismatches, 0);
        assert_eq!(
            support.metrics.centers_decoded,
            memory.package.centers.len()
        );
        assert!(support
            .values()
            .iter()
            .enumerate()
            .any(|(atom_id, exact)| { *exact > memory.forward_degree(atom_id as u32) as u32 }));
    }

    #[test]
    fn implicit_relation_state_matches_complete_compiler_postings() {
        let memory = tiny_memory();
        let support = tiny_support(&memory);
        for terminal_id in 0..memory.package.terminal_count() {
            let observed = observed_lexical_atoms(
                &memory,
                &memory
                    .decode_terminal(terminal_id)
                    .expect("decode terminal"),
            );
            let candidate =
                reconstruct_candidate(&memory.package, &support, &observed, terminal_id)
                    .expect("reconstruct candidate");
            for implicit in candidate.relations {
                let posting = memory
                    .complete_forward_couplings(implicit.atom_id)
                    .expect("complete posting");
                let index = posting
                    .binary_search_by_key(&terminal_id, |relation| relation.peer_id)
                    .expect("implicit relation in posting");
                assert_eq!(implicit.coupling, posting[index]);
            }
        }
    }

    #[test]
    fn implicit_activation_matches_complete_posting_oracle() {
        let memory = tiny_memory();
        let support = tiny_support(&memory);
        for query in ["aaaa", "abaa", "каса", "tesr"] {
            let observed = observed_lexical_atoms(&memory, query);
            let candidates = (0..memory.package.terminal_count())
                .map(|terminal_id| {
                    reconstruct_candidate(&memory.package, &support, &observed, terminal_id)
                })
                .collect::<Result<Vec<_>, _>>()
                .expect("reconstruct all candidates");
            let oracle = V8QueryOracle::build(&memory, &observed, &candidates)
                .expect("build complete oracle");
            let metrics = oracle.compare(&observed, &candidates);
            assert_eq!(metrics.relation_mismatches(), 0);
            assert_eq!(metrics.activation_mismatches(), 0);
        }
    }

    #[test]
    fn candidate_iteration_order_cannot_change_implicit_bytes() {
        let memory = tiny_memory();
        let support = tiny_support(&memory);
        let observed = observed_lexical_atoms(&memory, "каса");
        let forward = (0..memory.package.terminal_count())
            .map(|terminal_id| {
                reconstruct_candidate(&memory.package, &support, &observed, terminal_id)
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("forward candidates");
        let mut reverse = (0..memory.package.terminal_count())
            .rev()
            .map(|terminal_id| {
                reconstruct_candidate(&memory.package, &support, &observed, terminal_id)
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("reverse candidates");
        reverse.sort_unstable_by_key(|candidate| candidate.terminal_id);
        assert!(candidates_equal(&forward, &reverse));
    }

    #[test]
    fn stored_support_fault_is_detected_before_query_execution() {
        let mut memory = tiny_memory();
        memory.package.atoms[0].support = memory.package.atoms[0].support.saturating_add(1);
        let support = tiny_support(&memory);
        assert_eq!(support.metrics.stored_support_mismatches, 1);
    }

    #[test]
    fn corpus_terminal_reordering_is_detected_before_query_execution() {
        let memory = tiny_memory();
        let mut surfaces = tiny_surfaces(&memory);
        surfaces.swap(0, 1);
        let support = ExactSupportField::rebuild(&memory.package, &surfaces)
            .expect("rebuild support with reordered corpus");
        assert_eq!(support.metrics.corpus_surface_mismatches, 2);
    }

    #[test]
    fn oracle_detects_a_canonical_relation_removed_from_implicit_output() {
        let memory = tiny_memory();
        let support = tiny_support(&memory);
        let observed = observed_lexical_atoms(&memory, "каса");
        let mut candidates = (0..memory.package.terminal_count())
            .map(|terminal_id| {
                reconstruct_candidate(&memory.package, &support, &observed, terminal_id)
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("reconstruct candidates");
        let oracle =
            V8QueryOracle::build(&memory, &observed, &candidates).expect("build complete oracle");
        candidates
            .iter_mut()
            .find(|candidate| !candidate.relations.is_empty())
            .expect("candidate with lexical relations")
            .relations
            .pop();

        let metrics = oracle.compare(&observed, &candidates);

        assert_eq!(metrics.v8_relations_missing_implicitly, 1);
        assert_ne!(metrics.relation_mismatches(), 0);
    }

    #[test]
    fn package_dependency_audit_rejects_an_unowned_learned_bank() {
        let mut memory = tiny_a2_memory();
        assert!(PackageDependencyAudit::inspect(&memory.package).resolved());
        memory.package.positive_subcenters.push(Default::default());

        let audit = PackageDependencyAudit::inspect(&memory.package);

        assert!(!audit.resolved());
        assert!(audit
            .unresolved
            .iter()
            .any(|reason| reason.contains("positive subcenter")));
    }

    #[test]
    fn exact_reverse_matches_the_independent_compiler_reference() {
        let memory = tiny_a2_memory();
        let support = tiny_support(&memory);
        let terminal_ids = (0..memory.package.terminal_count()).collect::<Vec<_>>();
        let exact = exact_reverse::ReverseBank::exact(&memory.package, &support, &terminal_ids)
            .expect("exact reverse bank");
        let reference = exact_reverse::ReverseBank::compiler_reference(
            &memory.package,
            &support,
            &terminal_ids,
        )
        .expect("compiler reverse bank");

        assert_eq!(exact.compare(&reference).mismatches(), 0);
        assert_eq!(exact.fingerprint(), reference.fingerprint());
    }

    #[test]
    fn explicit_geometry_cannot_fall_back_when_reverse_is_missing() {
        let memory = tiny_a2_memory();
        let mut candidates = vec![super::super::runtime::GrokkingCandidate {
            terminal_id: 0,
            ..Default::default()
        }];

        let result = memory.apply_restoration_geometry_with_explicit_reverse(
            "aaaa",
            &mut candidates,
            |_| None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn exact_settlement_is_compiler_and_permutation_invariant() {
        let memory = tiny_a2_memory();
        let support = tiny_support(&memory);
        let observed = observed_lexical_atoms(&memory, "каса");
        let implicit = (0..memory.package.terminal_count())
            .map(|terminal_id| {
                reconstruct_candidate(&memory.package, &support, &observed, terminal_id)
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("implicit candidates");
        let result = evaluate_settlement_case(&memory, &support, "каса", &implicit)
            .expect("evaluate exact settlement");
        let mut aggregate = SettlementAggregate::default();
        aggregate.record(&result);

        assert!(aggregate.passes(), "{}", aggregate.report());
    }
}
