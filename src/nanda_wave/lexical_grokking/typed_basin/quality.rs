use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;

use super::super::corruption::DamageExample;
use super::super::forward_decoder_index::ForwardDecoderIndex;
use super::super::proof::{
    corpus_words_from_lines, populate_sampled_ambiguity, prepare_fixed_heldout_cases,
    proof_worker_count, FixedHeldoutCase,
};
use super::super::restoration::{self, RestorationReadout};
use super::super::runtime::{
    truncate_with_reconstruction_tail, ForwardActivation, GrokkingCandidate, LexicalGrokkingMemory,
    ReadoutMode,
};
use super::super::typed_edit_traversal::phase7d_terminal_diagnostic_evidence;
use super::implicit_forward::{reconstruct_candidate, ImplicitCandidate};
use super::package_dependencies::PackageDependencyAudit;
use super::settlement::{settle_exact_case, ExactSettlementResult};
use super::support::ExactSupportField;

mod metrics;
#[cfg(test)]
mod tests;

use metrics::*;

const FULL_HELDOUT_PER_CLASS: usize = 20_000;
const TOP_K: usize = 64;
const WORKER_DIAGNOSTIC_LIMIT: usize = 16;

struct SurfaceEvaluation {
    exact: ExactSettlementResult,
    typed_certificate_classes: BTreeMap<u32, Vec<&'static str>>,
    implicit_activations: BTreeMap<u32, ForwardActivation>,
    typed_us: u64,
    implicit_us: u64,
    legacy_candidates: Vec<GrokkingCandidate>,
    legacy_readout: Option<RestorationReadout>,
    legacy_us: u64,
}

struct ProofProgress {
    label: &'static str,
    total: usize,
    interval: usize,
    completed: AtomicUsize,
    started: Instant,
}

impl ProofProgress {
    fn new(label: &'static str, total: usize) -> Self {
        Self {
            label,
            total,
            interval: (total / 20).max(1),
            completed: AtomicUsize::new(0),
            started: Instant::now(),
        }
    }

    fn advance(&self) {
        let completed = self.completed.fetch_add(1, Ordering::Relaxed) + 1;
        if completed == self.total || completed % self.interval == 0 {
            eprintln!(
                "gate_c_progress phase={} completed={}/{} elapsed_ms={}",
                self.label,
                completed,
                self.total,
                self.started.elapsed().as_millis()
            );
        }
    }
}

pub fn prove_l1_typed_basin_quality(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    clean_limit: usize,
    requested_workers: usize,
) -> io::Result<serde_json::Value> {
    prove_l1_typed_basin_quality_scoped(
        corpus_path,
        package_path,
        max_words,
        heldout_per_class,
        clean_limit,
        requested_workers,
        None,
    )
}

pub fn diagnose_l1_typed_basin_quality_class(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    requested_workers: usize,
    damage_class: &str,
) -> io::Result<serde_json::Value> {
    prove_l1_typed_basin_quality_scoped(
        corpus_path,
        package_path,
        max_words,
        heldout_per_class,
        1,
        requested_workers,
        Some(damage_class),
    )
}

#[allow(clippy::too_many_arguments)]
fn prove_l1_typed_basin_quality_scoped(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    clean_limit: usize,
    requested_workers: usize,
    damage_class_filter: Option<&str>,
) -> io::Result<serde_json::Value> {
    let wall_started = Instant::now();
    let package_sha256_before = super::file_sha256(package_path)?;
    let embedded_a2 =
        super::prove_l1_typed_basin_implicit_forward(corpus_path, package_path, max_words, 1)?;
    let embedded_a2_pass = embedded_a2
        .get("verdict")
        .and_then(serde_json::Value::as_str)
        == Some("PASS_A0_A1_B0_A2");
    if !embedded_a2_pass {
        return prerequisite_rejection(
            corpus_path,
            package_path,
            package_sha256_before,
            embedded_a2,
            wall_started,
        );
    }

    let layout = super::read_v8_layout(package_path)?;
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
    let projected_package_bytes = layout
        .base_bytes
        .saturating_add(support.metrics.projected_overflow_bytes as u64);
    let dependencies = PackageDependencyAudit::inspect(&memory.package);

    let objective_started = Instant::now();
    let fixed_cases = prepare_fixed_heldout_cases(&words, heldout_per_class, 0)?;
    let objectives = build_objectives(&words, &fixed_cases)?;
    let cases = select_damage_cases(fixed_cases, damage_class_filter)?;
    let objective_build_ms = objective_started.elapsed().as_millis();
    let damaged_workers = configured_workers(cases.len(), requested_workers);
    let clean_terminal_ids = clean_terminal_ids(words.len(), clean_limit);
    let clean_workers = configured_workers(clean_terminal_ids.len(), requested_workers);

    let damaged_started = Instant::now();
    let mut result = evaluate_damaged(
        &memory,
        &decoder_index,
        &support,
        &words,
        &cases,
        &objectives,
        damaged_workers,
    )
    .map_err(io::Error::other)?;
    let damaged_ms = damaged_started.elapsed().as_millis();

    let clean_started = Instant::now();
    let clean_result = evaluate_clean(
        &memory,
        &decoder_index,
        &support,
        &words,
        &clean_terminal_ids,
        clean_workers,
    )
    .map_err(io::Error::other)?;
    result.merge(clean_result);
    let clean_ms = clean_started.elapsed().as_millis();
    result.finish_diagnostics();

    let package_sha256_after = super::file_sha256(package_path)?;
    let package_unchanged = package_sha256_before == package_sha256_after;
    let selected_damage_class_count = if damage_class_filter.is_some() {
        1
    } else {
        super::FIXED_DAMAGE_CLASS_COUNT
    };
    let expected_damaged = heldout_per_class
        .checked_mul(selected_damage_class_count)
        .ok_or_else(|| io::Error::other("Gate C damaged denominator exceeds usize"))?;
    let aggregate = result.aggregate();
    let class_denominator_complete = result.classes.len() == selected_damage_class_count
        && aggregate.cases == expected_damaged
        && result
            .classes
            .values()
            .all(|metrics| metrics.cases == heldout_per_class);
    let clean_denominator_complete = result.clean.cases == clean_terminal_ids.len();
    let route_complete = embedded_a2_pass
        && dependencies.resolved()
        && package_unchanged
        && class_denominator_complete
        && clean_denominator_complete;
    let target_retention_complete = aggregate.target_retained == aggregate.cases;
    let every_class_unique_top1 = result.classes.values().all(|metrics| {
        ratio_strictly_above(metrics.unique_top1, metrics.objective_unique_cases, 95, 100)
    });
    let every_class_top64 = result
        .classes
        .values()
        .all(|metrics| ratio_at_least(metrics.target_top64, metrics.cases, 99, 100));
    let clean_preservation_pass =
        ratio_at_least(result.clean.preserved, result.clean.cases, 999, 1_000);
    let full_denominator = damage_class_filter.is_none()
        && heldout_per_class == FULL_HELDOUT_PER_CLASS
        && clean_limit == 0
        && result.clean.cases == words.len();
    let full_quality_pass = route_complete
        && full_denominator
        && target_retention_complete
        && every_class_unique_top1
        && every_class_top64
        && clean_preservation_pass
        && aggregate.false_authority == 0
        && aggregate.false_singleton == 0
        && aggregate.legacy_grounded_losses == 0;
    let verdict = quality_verdict(
        damage_class_filter.is_some(),
        full_denominator,
        route_complete,
        full_quality_pass,
    );

    let class_report = result
        .classes
        .iter()
        .map(|(class, metrics)| ((*class).to_string(), metrics.report()))
        .collect::<serde_json::Map<_, _>>();
    Ok(serde_json::json!({
        "schema": "lay.l11.typed-basin-quality-proof.v2",
        "verdict": verdict,
        "artifact": {
            "corpus": corpus_path,
            "package": package_path,
            "package_bytes": layout.package_bytes,
            "compact_base_bytes": layout.base_bytes,
            "projected_compact_package_bytes": projected_package_bytes,
            "package_sha256_before": package_sha256_before,
            "package_sha256_after": package_sha256_after,
            "package_bytes_unchanged": package_unchanged,
            "primary_centers": words.len(),
        },
        "configuration": {
            "heldout_per_class": heldout_per_class,
            "fixed_damage_classes": super::FIXED_DAMAGE_CLASS_COUNT,
            "selected_damage_classes": selected_damage_class_count,
            "damage_class_filter": damage_class_filter,
            "expected_damaged_cases": expected_damaged,
            "clean_limit": clean_limit,
            "clean_sampling": if clean_limit == 0 {
                "all_primary_centers"
            } else {
                "deterministic_evenly_spaced_smoke_only"
            },
            "damaged_workers": damaged_workers,
            "clean_workers": clean_workers,
            "top_k_projection": TOP_K,
        },
        "embedded_a2_prerequisite": embedded_a2,
        "exact_support": super::support_report(
            support.metrics,
            projected_package_bytes,
            support_rebuild_ms,
        ),
        "package_dependencies": dependencies.report(),
        "damaged_quality": {
            "classes": class_report,
            "aggregate": aggregate.report(),
            "exact_reverse_terminals": result.exact_reverse_terminals,
            "exact_reverse_relations": result.exact_reverse_relations,
            "timing": result.timings.report(),
        },
        "clean_quality": {
            "cases": result.clean.cases,
            "target_retained_complete_field": result.clean.target_retained,
            "target_rank1": result.clean.target_rank1,
            "preserved": result.clean.preserved,
            "preservation_percent": percent(result.clean.preserved, result.clean.cases),
            "mutating_winner": result.clean.mutating_winner,
            "readout": {
                "winner": result.clean.winners,
                "tied": result.clean.tied,
                "abstain": result.clean.abstained,
            },
            "phase_noop_cases": result.clean.exact_phase_noop,
        },
        "diagnostics": result.diagnostics,
        "gates": {
            "embedded_a2_pass": embedded_a2_pass,
            "package_dependencies_resolved": dependencies.resolved(),
            "package_isolation": package_unchanged,
            "fixed_damaged_denominator_complete": class_denominator_complete,
            "clean_denominator_complete": clean_denominator_complete,
            "full_fixed_denominator": full_denominator,
            "target_retention_complete": target_retention_complete,
            "unique_top1_every_class_strictly_gt_95_percent": every_class_unique_top1,
            "lattice_coverage_every_class_ge_99_percent": every_class_top64,
            "clean_preservation_ge_99_9_percent": clean_preservation_pass,
            "false_authority_zero": aggregate.false_authority == 0,
            "false_singleton_zero": aggregate.false_singleton == 0,
            "grounded_legacy_candidate_loss_zero": aggregate.legacy_grounded_losses == 0,
            "conjunctive_full_quality_pass": full_quality_pass,
        },
        "timing": {
            "package_load_ms": package_load_ms,
            "decoder_index_ms": decoder_index_ms,
            "exact_support_rebuild_ms": support_rebuild_ms,
            "objective_build_ms": objective_build_ms,
            "damaged_quality_ms": damaged_ms,
            "clean_quality_ms": clean_ms,
            "wall_ms": wall_started.elapsed().as_millis(),
            "scope": "proof_throughput_not_product_hot_latency",
        },
        "claim_boundary": {
            "full_quality_matrix_tested": full_denominator,
            "full_quality_claimed": full_quality_pass,
            "runtime_authority_changed": false,
            "package_format_changed": false,
            "installed_runtime_changed": false,
            "deployment_admitted": false,
        }
    }))
}

fn quality_verdict(
    class_diagnostic: bool,
    full_denominator: bool,
    route_complete: bool,
    full_quality_pass: bool,
) -> &'static str {
    if class_diagnostic {
        return if route_complete {
            "DIAGNOSTIC_C_CLASS"
        } else {
            "REJECT_C_HARNESS"
        };
    }
    if full_denominator {
        if full_quality_pass {
            "PASS_C_QUALITY"
        } else {
            "REJECT_C_QUALITY"
        }
    } else if route_complete {
        "PASS_C_SMOKE"
    } else {
        "REJECT_C_HARNESS"
    }
}

fn prerequisite_rejection(
    corpus_path: &Path,
    package_path: &Path,
    package_sha256_before: String,
    embedded_a2: serde_json::Value,
    wall_started: Instant,
) -> io::Result<serde_json::Value> {
    let package_sha256_after = super::file_sha256(package_path)?;
    Ok(serde_json::json!({
        "schema": "lay.l11.typed-basin-quality-proof.v2",
        "verdict": "REJECT_C_PREREQUISITE",
        "artifact": {
            "corpus": corpus_path,
            "package": package_path,
            "package_sha256_before": package_sha256_before,
            "package_sha256_after": package_sha256_after,
            "package_bytes_unchanged": package_sha256_before == package_sha256_after,
        },
        "embedded_a2_prerequisite": embedded_a2,
        "gates": {
            "embedded_a2_pass": false,
            "quality_workers_started": false,
            "conjunctive_full_quality_pass": false,
        },
        "timing": {
            "wall_ms": wall_started.elapsed().as_millis(),
        },
        "claim_boundary": {
            "full_quality_matrix_tested": false,
            "full_quality_claimed": false,
            "runtime_authority_changed": false,
            "installed_runtime_changed": false,
        }
    }))
}

fn select_damage_cases(
    cases: Vec<FixedHeldoutCase>,
    damage_class_filter: Option<&str>,
) -> io::Result<Vec<FixedHeldoutCase>> {
    let Some(filter) = damage_class_filter else {
        return Ok(cases);
    };
    let available = cases.iter().map(|case| case.class).collect::<BTreeSet<_>>();
    let selected = cases
        .into_iter()
        .filter(|case| case.class == filter)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "unknown Gate C damage class {filter:?}; available: {}",
                available.into_iter().collect::<Vec<_>>().join(", ")
            ),
        ));
    }
    Ok(selected)
}

fn build_objectives(
    words: &[String],
    cases: &[FixedHeldoutCase],
) -> io::Result<HashMap<String, BTreeSet<u32>>> {
    let heldout = cases
        .iter()
        .map(|case| {
            (
                case.terminal_id,
                DamageExample {
                    class: case.class,
                    surface: case.surface.clone(),
                },
            )
        })
        .collect::<Vec<_>>();
    let mut objectives = HashMap::new();
    populate_sampled_ambiguity(words, &heldout, &mut objectives);
    for case in cases {
        let objective = objectives.get(case.surface.as_str()).ok_or_else(|| {
            io::Error::other(format!(
                "Gate C objective builder omitted surface {:?}",
                case.surface
            ))
        })?;
        if !objective.contains(&case.terminal_id) {
            return Err(io::Error::other(format!(
                "Gate C objective omitted fixed target {} for {:?}",
                case.terminal_id, case.surface
            )));
        }
    }
    Ok(objectives)
}

fn evaluate_surface(
    memory: &LexicalGrokkingMemory,
    decoder_index: &ForwardDecoderIndex,
    support: &ExactSupportField,
    surface: &str,
    observe_legacy: bool,
) -> Result<SurfaceEvaluation, String> {
    let typed_started = Instant::now();
    let typed = phase7d_terminal_diagnostic_evidence(
        decoder_index,
        &memory.package.decoder_nodes,
        surface,
    )?;
    let typed_us = elapsed_us(typed_started);

    let implicit_started = Instant::now();
    let observed = super::observed_lexical_atoms(memory, surface);
    let mut implicit = typed
        .terminal
        .terminal_ids
        .iter()
        .copied()
        .map(|terminal_id| reconstruct_candidate(&memory.package, support, &observed, terminal_id))
        .collect::<Result<Vec<ImplicitCandidate>, String>>()?;
    implicit.sort_unstable_by_key(|candidate| candidate.terminal_id);
    if implicit
        .windows(2)
        .any(|pair| pair[0].terminal_id == pair[1].terminal_id)
    {
        return Err("Gate C typed basin contains duplicate terminals".to_string());
    }
    let implicit_activations = implicit
        .iter()
        .map(|candidate| (candidate.terminal_id, candidate.activation))
        .collect();
    let implicit_us = elapsed_us(implicit_started);
    let exact = settle_exact_case(memory, support, surface, &implicit)?;

    let legacy_started = Instant::now();
    let (legacy_candidates, legacy_readout) = if observe_legacy {
        let mut candidates = memory.readout(surface, TOP_K, ReadoutMode::Full);
        let readout = memory.classify_restoration(
            surface,
            &mut candidates,
            memory.package.restoration_calibration,
        );
        (candidates, Some(readout))
    } else {
        (Vec::new(), None)
    };
    let legacy_us = observe_legacy
        .then(|| elapsed_us(legacy_started))
        .unwrap_or_default();
    Ok(SurfaceEvaluation {
        exact,
        typed_certificate_classes: typed.certificate_classes,
        implicit_activations,
        typed_us,
        implicit_us,
        legacy_candidates,
        legacy_readout,
        legacy_us,
    })
}

fn evaluate_damaged(
    memory: &LexicalGrokkingMemory,
    decoder_index: &ForwardDecoderIndex,
    support: &ExactSupportField,
    words: &[String],
    cases: &[FixedHeldoutCase],
    objectives: &HashMap<String, BTreeSet<u32>>,
    workers: usize,
) -> Result<QualityShard, String> {
    let progress = ProofProgress::new("damaged", cases.len());
    let shards = thread::scope(|scope| {
        (0..workers)
            .map(|worker| {
                let progress = &progress;
                scope.spawn(move || -> Result<QualityShard, String> {
                    let mut shard = QualityShard::default();
                    for case in cases.iter().skip(worker).step_by(workers) {
                        // Labels are deliberately unavailable to traversal and settlement.
                        let output =
                            evaluate_surface(memory, decoder_index, support, &case.surface, true)?;
                        let objective = objectives.get(case.surface.as_str()).ok_or_else(|| {
                            format!("Gate C objective missing after readout: {:?}", case.surface)
                        })?;
                        record_damaged(&mut shard, words, case, objective, &output);
                        progress.advance();
                    }
                    Ok(shard)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "Gate C damaged worker panicked".to_string())?
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    let mut merged = QualityShard::default();
    for shard in shards {
        merged.merge(shard);
    }
    Ok(merged)
}

fn evaluate_clean(
    memory: &LexicalGrokkingMemory,
    decoder_index: &ForwardDecoderIndex,
    support: &ExactSupportField,
    words: &[String],
    terminal_ids: &[u32],
    workers: usize,
) -> Result<QualityShard, String> {
    let progress = ProofProgress::new("clean", terminal_ids.len());
    let shards = thread::scope(|scope| {
        (0..workers)
            .map(|worker| {
                let progress = &progress;
                scope.spawn(move || -> Result<QualityShard, String> {
                    let mut shard = QualityShard::default();
                    for terminal_id in terminal_ids.iter().skip(worker).step_by(workers).copied() {
                        let surface = words
                            .get(terminal_id as usize)
                            .ok_or_else(|| format!("clean terminal is invalid: {terminal_id}"))?;
                        let output =
                            evaluate_surface(memory, decoder_index, support, surface, false)?;
                        record_clean(&mut shard, words, terminal_id, surface, &output);
                        progress.advance();
                    }
                    Ok(shard)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "Gate C clean worker panicked".to_string())?
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    let mut merged = QualityShard::default();
    for shard in shards {
        merged.merge(shard);
    }
    Ok(merged)
}

fn bounded_projection(candidates: &[GrokkingCandidate]) -> Vec<GrokkingCandidate> {
    let mut bounded = candidates.to_vec();
    truncate_with_reconstruction_tail(&mut bounded, TOP_K);
    bounded
}

fn record_damaged(
    shard: &mut QualityShard,
    words: &[String],
    case: &FixedHeldoutCase,
    objective: &BTreeSet<u32>,
    output: &SurfaceEvaluation,
) {
    let exact = &output.exact.candidates;
    let complete_target_rank = exact
        .iter()
        .position(|candidate| candidate.terminal_id == case.terminal_id);
    let bounded = bounded_projection(exact);
    let bounded_target_rank = bounded
        .iter()
        .position(|candidate| candidate.terminal_id == case.terminal_id);
    let authority = authority_terminal(&output.exact.readout);
    let geometric_basin_terminals = restoration::geometric_basin(exact).len();
    let false_authority = authority.is_some_and(|terminal| !objective.contains(&terminal));
    let false_singleton = authority.is_some() && geometric_basin_terminals > 1;
    let exact_terminals = exact
        .iter()
        .map(|candidate| candidate.terminal_id)
        .collect::<BTreeSet<_>>();
    let legacy_grounded_losses = output
        .legacy_candidates
        .iter()
        .filter(|candidate| candidate.reconstruction_modes != 0)
        .filter(|candidate| !exact_terminals.contains(&candidate.terminal_id))
        .map(|candidate| candidate.terminal_id)
        .collect::<Vec<_>>();
    let legacy_authority = output.legacy_readout.as_ref().and_then(authority_terminal);
    let metrics = shard.classes.entry(case.class).or_default();
    metrics.cases += 1;
    metrics.objective_unique_cases += usize::from(objective.len() == 1);
    metrics.target_retained += usize::from(complete_target_rank.is_some());
    metrics.target_top64 += usize::from(bounded_target_rank.is_some());
    metrics.unique_top1 += usize::from(
        objective.len() == 1
            && exact
                .first()
                .is_some_and(|candidate| objective.contains(&candidate.terminal_id)),
    );
    record_readout(
        &output.exact.readout,
        &mut metrics.winners,
        &mut metrics.tied,
        &mut metrics.abstained,
    );
    metrics.false_authority += usize::from(false_authority);
    metrics.false_singleton += usize::from(false_singleton);
    metrics.exact_candidate_total = metrics
        .exact_candidate_total
        .saturating_add(exact.len() as u64);
    metrics.exact_candidate_max = metrics.exact_candidate_max.max(exact.len());
    metrics.exact_phase_noop += usize::from(output.exact.phase_noop);
    metrics.legacy_grounded_candidates += output
        .legacy_candidates
        .iter()
        .filter(|candidate| candidate.reconstruction_modes != 0)
        .count();
    metrics.legacy_grounded_losses += legacy_grounded_losses.len();
    metrics.legacy_target_top64 += usize::from(
        output
            .legacy_candidates
            .iter()
            .any(|candidate| candidate.terminal_id == case.terminal_id),
    );
    metrics.legacy_unique_top1 += usize::from(
        objective.len() == 1
            && output
                .legacy_candidates
                .first()
                .is_some_and(|candidate| objective.contains(&candidate.terminal_id)),
    );
    metrics.legacy_false_authority +=
        usize::from(legacy_authority.is_some_and(|terminal| !objective.contains(&terminal)));
    shard.exact_reverse_terminals = shard
        .exact_reverse_terminals
        .saturating_add(output.exact.reverse_terminals as u64);
    shard.exact_reverse_relations = shard
        .exact_reverse_relations
        .saturating_add(output.exact.reverse_relations as u64);
    shard.timings.typed_us.push(output.typed_us);
    shard.timings.implicit_us.push(output.implicit_us);
    shard
        .timings
        .exact_settlement_us
        .push(output.exact.elapsed_us);
    shard.timings.legacy_v8_us.push(output.legacy_us);

    for mechanism in [
        complete_target_rank
            .is_none()
            .then_some("typed_basin_target_missing"),
        (complete_target_rank.is_some() && bounded_target_rank.is_none())
            .then_some("target_missing_from_bounded_projection"),
        (objective.len() == 1
            && !exact
                .first()
                .is_some_and(|candidate| objective.contains(&candidate.terminal_id)))
        .then_some("unique_objective_not_rank1"),
        false_authority.then_some("false_authority"),
        false_singleton.then_some("false_singleton"),
        (!legacy_grounded_losses.is_empty()).then_some("grounded_legacy_candidate_loss"),
    ]
    .into_iter()
    .flatten()
    {
        push_diagnostic(
            shard,
            loss_diagnostic(
                "damaged",
                mechanism,
                case.class,
                &case.surface,
                case.terminal_id,
                words,
                complete_target_rank,
                bounded_target_rank,
                exact,
                &bounded,
                authority,
                objective,
                geometric_basin_terminals,
                &legacy_grounded_losses,
                &output.typed_certificate_classes,
                &output.implicit_activations,
            ),
        );
    }
}

fn record_clean(
    shard: &mut QualityShard,
    words: &[String],
    terminal_id: u32,
    surface: &str,
    output: &SurfaceEvaluation,
) {
    let exact = &output.exact.candidates;
    let target_rank = exact
        .iter()
        .position(|candidate| candidate.terminal_id == terminal_id);
    let authority = authority_terminal(&output.exact.readout);
    let mutating_winner = authority.is_some_and(|winner| winner != terminal_id);
    let rank1 = target_rank == Some(0);
    shard.clean.cases += 1;
    shard.clean.target_retained += usize::from(target_rank.is_some());
    shard.clean.target_rank1 += usize::from(rank1);
    shard.clean.preserved += usize::from(rank1 && !mutating_winner);
    shard.clean.mutating_winner += usize::from(mutating_winner);
    shard.clean.exact_phase_noop += usize::from(output.exact.phase_noop);
    record_readout(
        &output.exact.readout,
        &mut shard.clean.winners,
        &mut shard.clean.tied,
        &mut shard.clean.abstained,
    );
    shard.exact_reverse_terminals = shard
        .exact_reverse_terminals
        .saturating_add(output.exact.reverse_terminals as u64);
    shard.exact_reverse_relations = shard
        .exact_reverse_relations
        .saturating_add(output.exact.reverse_relations as u64);
    shard.timings.typed_us.push(output.typed_us);
    shard.timings.implicit_us.push(output.implicit_us);
    shard
        .timings
        .exact_settlement_us
        .push(output.exact.elapsed_us);
    if !rank1 || mutating_winner {
        push_diagnostic(
            shard,
            loss_diagnostic(
                "clean",
                if mutating_winner {
                    "clean_mutating_winner"
                } else {
                    "clean_target_not_rank1"
                },
                "clean",
                surface,
                terminal_id,
                words,
                target_rank,
                target_rank,
                exact,
                exact,
                authority,
                &BTreeSet::from([terminal_id]),
                restoration::geometric_basin(exact).len(),
                &[],
                &output.typed_certificate_classes,
                &output.implicit_activations,
            ),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn loss_diagnostic(
    scope: &'static str,
    mechanism: &'static str,
    class: &'static str,
    surface: &str,
    target_terminal: u32,
    words: &[String],
    target_rank: Option<usize>,
    bounded_target_rank: Option<usize>,
    candidates: &[GrokkingCandidate],
    bounded_candidates: &[GrokkingCandidate],
    authority_terminal: Option<u32>,
    objective: &BTreeSet<u32>,
    geometric_basin_terminals: usize,
    legacy_grounded_losses: &[u32],
    typed_certificate_classes: &BTreeMap<u32, Vec<&'static str>>,
    implicit_activations: &BTreeMap<u32, ForwardActivation>,
) -> LossDiagnostic {
    LossDiagnostic {
        scope,
        mechanism,
        class,
        surface: surface.to_string(),
        target_terminal,
        target_surface: words
            .get(target_terminal as usize)
            .cloned()
            .unwrap_or_default(),
        target_rank: target_rank.map(|rank| rank + 1),
        bounded_target_rank: bounded_target_rank.map(|rank| rank + 1),
        top_terminals: candidates
            .iter()
            .take(8)
            .map(|candidate| candidate.terminal_id)
            .collect(),
        bounded_top_terminals: bounded_candidates
            .iter()
            .take(8)
            .map(|candidate| candidate.terminal_id)
            .collect(),
        authority_terminal,
        objective_terminals: objective.iter().copied().take(8).collect(),
        geometric_basin_terminals,
        legacy_grounded_losses: legacy_grounded_losses.iter().copied().take(8).collect(),
        target_evidence: target_rank
            .and_then(|rank| candidates.get(rank).map(|candidate| (rank, candidate)))
            .map(|(rank, candidate)| {
                candidate_evidence(
                    rank,
                    candidate,
                    words,
                    objective,
                    typed_certificate_classes,
                    implicit_activations,
                )
            }),
        top_candidate_evidence: candidates
            .iter()
            .take(8)
            .enumerate()
            .map(|(index, candidate)| {
                candidate_evidence(
                    index,
                    candidate,
                    words,
                    objective,
                    typed_certificate_classes,
                    implicit_activations,
                )
            })
            .collect(),
    }
}

fn candidate_evidence(
    index: usize,
    candidate: &GrokkingCandidate,
    words: &[String],
    objective: &BTreeSet<u32>,
    typed_certificate_classes: &BTreeMap<u32, Vec<&'static str>>,
    implicit_activations: &BTreeMap<u32, ForwardActivation>,
) -> CandidateEvidenceDiagnostic {
    CandidateEvidenceDiagnostic {
        rank: index + 1,
        terminal_id: candidate.terminal_id,
        surface: words
            .get(candidate.terminal_id as usize)
            .cloned()
            .unwrap_or_default(),
        objective_member: objective.contains(&candidate.terminal_id),
        typed_certificate_classes: typed_certificate_classes
            .get(&candidate.terminal_id)
            .cloned()
            .unwrap_or_default(),
        implicit_activation: implicit_activations
            .get(&candidate.terminal_id)
            .copied()
            .map(|activation| ImplicitActivationDiagnostic {
                mass: activation.mass,
                hits: activation.hits,
                surface_hits: activation.surface_hits,
                keyboard_hits: activation.keyboard_hits,
            }),
        atom_hits: candidate.atom_hits,
        surface_hits: candidate.surface_hits,
        keyboard_hits: candidate.keyboard_hits,
        forward_milli: candidate.forward_milli,
        backward_milli: candidate.backward_milli,
        structural_milli: candidate.structural_milli,
        sequence_milli: candidate.sequence_milli,
        position_milli: candidate.position_milli,
        length_milli: candidate.length_milli,
        geometry_distance: candidate.geometry_distance,
        reconstruction_modes: candidate.reconstruction_modes,
        settled_energy: candidate.settled_energy,
        exact_reconstruction: candidate.exact_reconstruction,
    }
}

fn push_diagnostic(shard: &mut QualityShard, diagnostic: LossDiagnostic) {
    if shard.diagnostics.len() < WORKER_DIAGNOSTIC_LIMIT {
        shard.diagnostics.push(diagnostic);
    }
}

fn clean_terminal_ids(terminal_count: usize, clean_limit: usize) -> Vec<u32> {
    let selected = if clean_limit == 0 {
        terminal_count
    } else {
        clean_limit.min(terminal_count)
    };
    (0..selected)
        .map(|index| {
            if selected == terminal_count {
                index as u32
            } else {
                index
                    .saturating_mul(terminal_count)
                    .saturating_div(selected) as u32
            }
        })
        .collect()
}

fn configured_workers(case_count: usize, requested: usize) -> usize {
    if requested == 0 {
        proof_worker_count(case_count)
    } else {
        requested.clamp(1, 32).min(case_count.max(1))
    }
}

fn elapsed_us(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}
