use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

use sha2::{Digest, Sha256};

use super::anchor_recovery_reduce::{
    audit_existing_anchor_recovery_field, induce_anchor_recovery_field,
    induce_shared_support_anchor_recovery_field,
};
use super::compiler::{compile_productive_package, ProductivePackageCompilerConfigV1};
use super::corpus::{
    load_axis_schema, replay_productive_context_spool, run_productive_raw_corpus_pass,
    ProductiveRawCorpusConfigV1,
};
use super::events::{TypedEventSpoolConfigV1, TypedEventSpoolShardV1};
use super::evidence_reduce::{
    reduce_productive_evidence, replay_packaged_calibration, EvidenceReduceConfigV1,
};
use super::packaged_runtime::PackagedProductiveRuntimeV1;
use super::reduce::{
    reduce_train_morphology_with_imported_ownership, reopen_imported_reduced_morphology,
    TrainMorphologyReduceConfigV1,
};
use super::spool_sort::{
    external_sort_verified_spool_with_workers, ExternalSpoolSortConfigV1,
    SortedTypedEventSpoolManifestV1,
};
use super::transition_reduce::{
    induce_transition_field, reopen_transition_induction, TransitionReduceConfigV1,
};
use super::PRODUCTIVE_V1_SCHEMA_VERSION;

const SPLIT_SEED_V1: u64 = 0x4c32_5052_4f44_5631;
const COMPILER_VERSION_V1: u32 = 1;
const NORMALIZATION_VERSION_V1: u32 = 1;
const SORT_BUFFER_BYTES_PER_WORKER: usize = 32 * 1024 * 1024;
const WRITE_BUFFER_BYTES: usize = 8 * 1024 * 1024;
const MAXIMUM_RECORD_BYTES: usize = 4 * 1024 * 1024;
const MAXIMUM_LEMMA_BYTES: usize = 64 * 1024 * 1024;
const MAXIMUM_OPEN_RUNS: usize = 32;
const MAXIMUM_LEMMA_TRANSITIONS: usize = 65_536;
const MAXIMUM_CONTEXT_EVENTS: usize = 10_000_000;
const MAXIMUM_CANDIDATES_PER_GROUP: usize = 64;
const PRODUCTIVE_PACKAGE_BYTE_BUDGET: u64 = 81_688_382;
const STEADY_RSS_KIB_BUDGET: u32 = 314_888;
const PEAK_RSS_KIB_BUDGET: u32 = 337_016;
const COLD_PUBLISH_BUDGET_US: u32 = 1_000_000;
const HOT_P99_BUDGET_US: u32 = 5_000;

#[derive(Clone, Debug)]
pub(crate) struct ProductiveOrchestratorConfigV1 {
    pub(crate) l11_package_path: PathBuf,
    pub(crate) canonical_l2_path: PathBuf,
    pub(crate) corpus_path: PathBuf,
    pub(crate) axis_schema_path: PathBuf,
    pub(crate) work_root: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) expected_corpus_sha256: [u8; 32],
    pub(crate) expected_corpus_bytes: u64,
    pub(crate) workers: usize,
    pub(crate) shared_support_recovery: bool,
}

pub(crate) fn audit_productive_anchor_recovery_v1(
    axis_schema_path: &Path,
    work_root: &Path,
    scratch_root: &Path,
) -> Result<serde_json::Value, String> {
    let induction_root = work_root.join("induction");
    if scratch_root == induction_root {
        return Err("anchor recovery audit scratch root aliases frozen induction".to_string());
    }
    let axis_schema = load_axis_schema(axis_schema_path)?;
    audit_existing_anchor_recovery_field(
        &induction_root,
        &axis_schema,
        &TransitionReduceConfigV1 {
            root: scratch_root.to_path_buf(),
            maximum_buffer_bytes: SORT_BUFFER_BYTES_PER_WORKER,
            maximum_open_runs: MAXIMUM_OPEN_RUNS,
            write_buffer_bytes: WRITE_BUFFER_BYTES,
            maximum_record_bytes: MAXIMUM_RECORD_BYTES,
            maximum_lemma_transitions: MAXIMUM_LEMMA_TRANSITIONS,
        },
    )
}

pub(crate) fn compile_productive_paradigm_field_v1(
    config: &ProductiveOrchestratorConfigV1,
    progress: &mut dyn FnMut(&str),
) -> Result<serde_json::Value, String> {
    validate_config(config)?;
    fs::create_dir_all(&config.work_root).map_err(|error| error.to_string())?;
    if let Some(parent) = config.output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let total_started = Instant::now();
    let mut stage_timings_ms = serde_json::Map::new();

    progress("raw_corpus_pass");
    let started = Instant::now();
    let raw = run_productive_raw_corpus_pass(&ProductiveRawCorpusConfigV1 {
        corpus_path: config.corpus_path.clone(),
        canonical_l2_path: config.canonical_l2_path.clone(),
        axis_schema_path: config.axis_schema_path.clone(),
        raw_context_path: config.work_root.join("raw-context.p2r"),
        source_role: format!(
            "productive-v1:{}",
            config
                .corpus_path
                .canonicalize()
                .unwrap_or_else(|_| config.corpus_path.clone())
                .display()
        ),
        expected_corpus_sha256: config.expected_corpus_sha256,
        expected_corpus_bytes: config.expected_corpus_bytes,
        morphology_spool: TypedEventSpoolConfigV1 {
            root: config.work_root.join("morphology-raw"),
            shard_count: config.workers,
            split_seed: SPLIT_SEED_V1,
            compiler_version: COMPILER_VERSION_V1,
            normalization_version: NORMALIZATION_VERSION_V1,
            write_buffer_bytes: WRITE_BUFFER_BYTES,
        },
        raw_context_write_buffer_bytes: WRITE_BUFFER_BYTES,
    })?;
    record_timing(&mut stage_timings_ms, "raw_corpus_pass", started);

    progress("morphology_external_sort");
    let started = Instant::now();
    let morphology_sorted = external_sort_verified_spool_with_workers(
        &raw.morphology_spool,
        &ExternalSpoolSortConfigV1 {
            root: config.work_root.join("morphology-sorted"),
            maximum_buffer_bytes: SORT_BUFFER_BYTES_PER_WORKER,
            maximum_open_runs: MAXIMUM_OPEN_RUNS,
            write_buffer_bytes: WRITE_BUFFER_BYTES,
        },
        config.workers,
    )?;
    record_timing(&mut stage_timings_ms, "morphology_external_sort", started);

    progress("imported_ownership_reduce");
    let started = Instant::now();
    let canonical_l2 = super::super::runtime::StandaloneL2Field::load(&config.canonical_l2_path)?;
    let reduced = reduce_train_morphology_with_imported_ownership(
        &morphology_sorted,
        &canonical_l2,
        &TrainMorphologyReduceConfigV1 {
            output_path: config.work_root.join("reduced-lemmas.p2l"),
            write_buffer_bytes: WRITE_BUFFER_BYTES,
            maximum_lemma_bytes: MAXIMUM_LEMMA_BYTES,
        },
    )?;
    record_timing(&mut stage_timings_ms, "imported_ownership_reduce", started);

    progress("transition_paradigm_induction");
    let started = Instant::now();
    let mut induction = induce_transition_field(
        &reduced,
        &raw.axis_schema,
        &TransitionReduceConfigV1 {
            root: config.work_root.join("induction"),
            maximum_buffer_bytes: SORT_BUFFER_BYTES_PER_WORKER,
            maximum_open_runs: MAXIMUM_OPEN_RUNS,
            write_buffer_bytes: WRITE_BUFFER_BYTES,
            maximum_record_bytes: MAXIMUM_RECORD_BYTES,
            maximum_lemma_transitions: MAXIMUM_LEMMA_TRANSITIONS,
        },
    )?;
    if config.shared_support_recovery {
        induction.anchor_recovery = Some(induce_shared_support_anchor_recovery_field(
            &induction,
            &raw.axis_schema,
            &TransitionReduceConfigV1 {
                root: config.work_root.join("anchor-recovery-shared-support-v1"),
                maximum_buffer_bytes: SORT_BUFFER_BYTES_PER_WORKER,
                maximum_open_runs: MAXIMUM_OPEN_RUNS,
                write_buffer_bytes: WRITE_BUFFER_BYTES,
                maximum_record_bytes: MAXIMUM_RECORD_BYTES,
                maximum_lemma_transitions: MAXIMUM_LEMMA_TRANSITIONS,
            },
        )?);
    }
    record_timing(
        &mut stage_timings_ms,
        "transition_paradigm_induction",
        started,
    );

    progress("context_typed_replay");
    let started = Instant::now();
    let context = replay_productive_context_spool(
        &raw.raw_context_path,
        &reduced,
        &canonical_l2,
        &raw.axis_schema,
        TypedEventSpoolConfigV1 {
            root: config.work_root.join("context-raw"),
            shard_count: config.workers,
            split_seed: SPLIT_SEED_V1,
            compiler_version: COMPILER_VERSION_V1,
            normalization_version: NORMALIZATION_VERSION_V1,
            write_buffer_bytes: WRITE_BUFFER_BYTES,
        },
    )?;
    record_timing(&mut stage_timings_ms, "context_typed_replay", started);

    progress("context_external_sort");
    let started = Instant::now();
    let context_sorted = external_sort_verified_spool_with_workers(
        &context.event_spool,
        &ExternalSpoolSortConfigV1 {
            root: config.work_root.join("context-sorted"),
            maximum_buffer_bytes: SORT_BUFFER_BYTES_PER_WORKER,
            maximum_open_runs: MAXIMUM_OPEN_RUNS,
            write_buffer_bytes: WRITE_BUFFER_BYTES,
        },
        config.workers,
    )?;
    record_timing(&mut stage_timings_ms, "context_external_sort", started);

    let evidence_config = EvidenceReduceConfigV1 {
        maximum_record_bytes: MAXIMUM_RECORD_BYTES,
        maximum_context_events: MAXIMUM_CONTEXT_EVENTS,
        maximum_candidates_per_group: MAXIMUM_CANDIDATES_PER_GROUP,
    };
    progress("evidence_reduce_and_fit");
    let started = Instant::now();
    let mut evidence = reduce_productive_evidence(
        &context_sorted,
        &reduced,
        &induction,
        &canonical_l2,
        &raw.axis_schema,
        &evidence_config,
    )?;
    record_timing(&mut stage_timings_ms, "evidence_reduce_and_fit", started);

    let l11_package_sha256 = sha256_file(&config.l11_package_path)?;
    let package_config = |output_path: PathBuf| ProductivePackageCompilerConfigV1 {
        output_path,
        maximum_record_bytes: MAXIMUM_RECORD_BYTES,
        l11_package_sha256,
        canonical_l2_package_sha256: raw.canonical_l2_sha256,
        productive_package_byte_budget: PRODUCTIVE_PACKAGE_BYTE_BUDGET,
        steady_rss_kib_budget: STEADY_RSS_KIB_BUDGET,
        peak_rss_kib_budget: PEAK_RSS_KIB_BUDGET,
        cold_publish_budget_us: COLD_PUBLISH_BUDGET_US,
        hot_p99_budget_us: HOT_P99_BUDGET_US,
    };

    progress("bootstrap_package");
    let started = Instant::now();
    let bootstrap = compile_productive_package(
        &reduced,
        &induction,
        &raw.axis_schema,
        &evidence.compiler_evidence,
        &package_config(config.work_root.join("productive-bootstrap.p2m")),
    )?;
    let bootstrap_runtime = PackagedProductiveRuntimeV1::load(
        &bootstrap.path,
        l11_package_sha256,
        raw.canonical_l2_sha256,
    )?;
    record_timing(&mut stage_timings_ms, "bootstrap_package", started);

    progress("actual_candidate_calibration");
    let started = Instant::now();
    let calibration = replay_packaged_calibration(
        &context_sorted,
        &reduced,
        &induction,
        &canonical_l2,
        &bootstrap_runtime,
        &raw.axis_schema,
        &evidence_config,
    )?;
    evidence.compiler_evidence.calibration = calibration.calibration.clone();
    record_timing(
        &mut stage_timings_ms,
        "actual_candidate_calibration",
        started,
    );

    progress("final_deterministic_package");
    let started = Instant::now();
    let final_package = compile_productive_package(
        &reduced,
        &induction,
        &raw.axis_schema,
        &evidence.compiler_evidence,
        &package_config(config.output_path.clone()),
    )?;
    let final_runtime = PackagedProductiveRuntimeV1::load(
        &final_package.path,
        l11_package_sha256,
        raw.canonical_l2_sha256,
    )?;
    if !final_runtime.mmap_backed()
        || final_runtime.package_bytes() != final_package.package_bytes as usize
    {
        return Err("productive final package failed mmap publication parity".to_string());
    }
    record_timing(
        &mut stage_timings_ms,
        "final_deterministic_package",
        started,
    );

    Ok(serde_json::json!({
        "kind": "l2_productive_paradigm_field_v1_compile",
        "verdict": "PASS_shadow_suggest_only_package",
        "runtime_authority_changed": false,
        "corpus": {
            "path": config.corpus_path,
            "bytes": raw.corpus_bytes,
            "sha256": hex_sha256(raw.corpus_sha256),
            "morphology_rows": raw.morphology_rows,
            "admitted_morphology_rows": raw.admitted_morphology_rows,
            "ungrounded_morphology_rows": raw.ungrounded_morphology_rows,
            "context_rows": raw.context_rows,
            "admitted_context_rows": raw.admitted_context_rows,
            "ungrounded_context_rows": raw.ungrounded_context_rows,
        },
        "split": {
            "seed": SPLIT_SEED_V1,
            "compiler_version": COMPILER_VERSION_V1,
            "normalization_version": NORMALIZATION_VERSION_V1,
        },
        "workers": config.workers,
        "anchor_recovery_support_policy": if config.shared_support_recovery {
            "fine_applicability_with_shared_certificate"
        } else {
            "fine_support_only"
        },
        "sort_buffer_bytes_per_worker": SORT_BUFFER_BYTES_PER_WORKER,
        "morphology_sort": {
            "input_records": morphology_sorted.input_records,
            "unique_records": morphology_sorted.unique_records,
            "initial_runs": morphology_sorted.initial_runs,
            "merge_passes": morphology_sorted.merge_passes,
        },
        "reduced": {
            "train_lemmas": reduced.lemma_count,
            "train_forms": reduced.form_count,
            "train_events": reduced.train_event_count,
            "imported_identity_verified": reduced.imported_identity_verified,
        },
        "induction": {
            "transition_observations": induction.transition_observations,
            "transferable_observations": induction.transferable_observations,
            "exact_allomorph_observations": induction.exact_allomorph_observations,
            "paradigms": induction.paradigm_count,
            "bound_lemmas": induction.bound_lemma_count,
        },
        "evidence": {
            "context_occurrence_events": evidence.context_occurrence_events,
            "direct_contradiction_events": evidence.direct_contradiction_events,
            "feedback_events": evidence.feedback_events,
            "proof_events": evidence.proof_events,
            "training_pairs": evidence.training_pairs,
            "bootstrap_calibration_groups": evidence.calibration_groups,
            "phase_profiles": evidence.phase_profiles,
            "selected_phase_centers": evidence.selected_phase_centers,
            "exact_only_morphology_forms": evidence.exact_only_morphology_forms,
            "exact_only_morphology_events": evidence.exact_only_morphology_events,
            "excluded_context_occurrence_events": evidence.excluded_context_occurrence_events,
            "excluded_direct_contradiction_events": evidence.excluded_direct_contradiction_events,
        },
        "actual_calibration": {
            "source_groups": calibration.source_groups,
            "fitted_groups": calibration.fitted_groups,
            "target_retained_groups": calibration.target_retained_groups,
            "target_lost_groups": calibration.target_lost_groups,
            "candidate_rows": calibration.candidate_rows,
            "authority_blocked_by_target_loss": calibration.target_lost_groups != 0,
        },
        "package": {
            "path": final_package.path,
            "bytes": final_package.package_bytes,
            "sha256": hex_sha256(final_package.package_sha256),
            "paradigms": final_package.paradigm_count,
            "bindings": final_package.binding_count,
            "programs": final_package.program_count,
            "operations": final_package.operation_count,
            "trie_nodes": final_package.trie_node_count,
            "trie_arcs": final_package.trie_arc_count,
            "terminals": final_package.terminal_count,
            "anchor_recovery": final_package.anchor_recovery.as_ref().map(|recovery| serde_json::json!({
                "path": recovery.path,
                "bytes": recovery.package_bytes,
                "sha256": hex_sha256(recovery.package_sha256),
                "indexes": recovery.index_count,
                "postings": recovery.posting_count,
                "shared_support_certified_postings": recovery.shared_support_certified_posting_count,
                "programs": recovery.program_count,
                "operations": recovery.operation_count,
            })),
            "mmap_backed": final_runtime.mmap_backed(),
            "resident_cache_bytes": final_runtime.resident_cache_bytes(),
        },
        "budgets": {
            "package_bytes": PRODUCTIVE_PACKAGE_BYTE_BUDGET,
            "steady_rss_kib": STEADY_RSS_KIB_BUDGET,
            "peak_rss_kib": PEAK_RSS_KIB_BUDGET,
            "cold_publish_us": COLD_PUBLISH_BUDGET_US,
            "hot_p99_us": HOT_P99_BUDGET_US,
        },
        "stage_timings_ms": stage_timings_ms,
        "total_ms": total_started.elapsed().as_millis() as u64,
        "proof_status": "PENDING_fixed_read_only_proof",
    }))
}

pub(crate) fn resume_productive_paradigm_field_v1(
    config: &ProductiveOrchestratorConfigV1,
    progress: &mut dyn FnMut(&str),
) -> Result<serde_json::Value, String> {
    reuse_productive_paradigm_field_v1(config, progress, false)
}

pub(crate) fn reinduce_productive_paradigm_field_v1(
    config: &ProductiveOrchestratorConfigV1,
    progress: &mut dyn FnMut(&str),
) -> Result<serde_json::Value, String> {
    reuse_productive_paradigm_field_v1(config, progress, true)
}

fn reuse_productive_paradigm_field_v1(
    config: &ProductiveOrchestratorConfigV1,
    progress: &mut dyn FnMut(&str),
    rebuild_induction: bool,
) -> Result<serde_json::Value, String> {
    validate_config(config)?;
    let total_started = Instant::now();
    let mut stage_timings_ms = serde_json::Map::new();

    progress("resume_reopen_reduced_morphology");
    let started = Instant::now();
    let axis_schema = load_axis_schema(&config.axis_schema_path)?;
    let canonical_l2 = super::super::runtime::StandaloneL2Field::load(&config.canonical_l2_path)?;
    let reduced = reopen_imported_reduced_morphology(
        &config.work_root.join("reduced-lemmas.p2l"),
        &config
            .work_root
            .join("morphology-sorted/sorted-events-global.p2s"),
        &canonical_l2,
        SPLIT_SEED_V1,
        COMPILER_VERSION_V1,
        NORMALIZATION_VERSION_V1,
    )?;
    record_timing(
        &mut stage_timings_ms,
        "resume_reopen_reduced_morphology",
        started,
    );

    let induction_stage = if rebuild_induction {
        "reinduce_transition_paradigm_field"
    } else {
        "resume_reopen_induction"
    };
    progress(induction_stage);
    let started = Instant::now();
    let mut induction = if rebuild_induction {
        induce_transition_field(
            &reduced,
            &axis_schema,
            &TransitionReduceConfigV1 {
                root: config.work_root.join("induction"),
                maximum_buffer_bytes: SORT_BUFFER_BYTES_PER_WORKER,
                maximum_open_runs: MAXIMUM_OPEN_RUNS,
                write_buffer_bytes: WRITE_BUFFER_BYTES,
                maximum_record_bytes: MAXIMUM_RECORD_BYTES,
                maximum_lemma_transitions: MAXIMUM_LEMMA_TRANSITIONS,
            },
        )?
    } else {
        reopen_transition_induction(
            &reduced,
            &axis_schema,
            &config.work_root.join("induction"),
            MAXIMUM_RECORD_BYTES,
        )?
    };
    record_timing(&mut stage_timings_ms, induction_stage, started);
    if config.shared_support_recovery || induction.anchor_recovery.is_none() {
        let stage = if config.shared_support_recovery {
            "resume_induce_shared_support_anchor_recovery"
        } else {
            "resume_induce_anchor_recovery"
        };
        progress(stage);
        let started = Instant::now();
        let recovery_config = TransitionReduceConfigV1 {
            root: if config.shared_support_recovery {
                config.work_root.join("anchor-recovery-shared-support-v1")
            } else {
                config.work_root.join("induction")
            },
            maximum_buffer_bytes: SORT_BUFFER_BYTES_PER_WORKER,
            maximum_open_runs: MAXIMUM_OPEN_RUNS,
            write_buffer_bytes: WRITE_BUFFER_BYTES,
            maximum_record_bytes: MAXIMUM_RECORD_BYTES,
            maximum_lemma_transitions: MAXIMUM_LEMMA_TRANSITIONS,
        };
        induction.anchor_recovery = Some(if config.shared_support_recovery {
            induce_shared_support_anchor_recovery_field(&induction, &axis_schema, &recovery_config)?
        } else {
            induce_anchor_recovery_field(&induction, &axis_schema, &recovery_config)?
        });
        record_timing(&mut stage_timings_ms, stage, started);
    }

    let context_sorted_path = config
        .work_root
        .join("context-sorted/sorted-events-global.p2s");
    if !context_sorted_path.is_file() {
        return Err("productive reuse is missing the sorted context spool".to_string());
    }
    let context_sorted = SortedTypedEventSpoolManifestV1 {
        schema_version: PRODUCTIVE_V1_SCHEMA_VERSION,
        split_seed: SPLIT_SEED_V1,
        compiler_version: COMPILER_VERSION_V1,
        normalization_version: NORMALIZATION_VERSION_V1,
        shards: vec![TypedEventSpoolShardV1 {
            path: context_sorted_path,
            record_count: 0,
        }],
        input_records: 0,
        unique_records: 0,
        initial_runs: 0,
        merge_passes: 0,
    };
    let canonical_l2_package_sha256 = sha256_file(&config.canonical_l2_path)?;
    let l11_package_sha256 = sha256_file(&config.l11_package_path)?;
    let evidence_config = EvidenceReduceConfigV1 {
        maximum_record_bytes: MAXIMUM_RECORD_BYTES,
        maximum_context_events: MAXIMUM_CONTEXT_EVENTS,
        maximum_candidates_per_group: MAXIMUM_CANDIDATES_PER_GROUP,
    };

    progress("evidence_reduce_and_fit");
    let started = Instant::now();
    let mut evidence = reduce_productive_evidence(
        &context_sorted,
        &reduced,
        &induction,
        &canonical_l2,
        &axis_schema,
        &evidence_config,
    )?;
    record_timing(&mut stage_timings_ms, "evidence_reduce_and_fit", started);

    let package_config = |output_path: PathBuf| ProductivePackageCompilerConfigV1 {
        output_path,
        maximum_record_bytes: MAXIMUM_RECORD_BYTES,
        l11_package_sha256,
        canonical_l2_package_sha256,
        productive_package_byte_budget: PRODUCTIVE_PACKAGE_BYTE_BUDGET,
        steady_rss_kib_budget: STEADY_RSS_KIB_BUDGET,
        peak_rss_kib_budget: PEAK_RSS_KIB_BUDGET,
        cold_publish_budget_us: COLD_PUBLISH_BUDGET_US,
        hot_p99_budget_us: HOT_P99_BUDGET_US,
    };

    progress("bootstrap_package");
    let started = Instant::now();
    let bootstrap_name = if rebuild_induction {
        "productive-bootstrap-reinduce.p2m"
    } else {
        "productive-bootstrap-resume.p2m"
    };
    let bootstrap = compile_productive_package(
        &reduced,
        &induction,
        &axis_schema,
        &evidence.compiler_evidence,
        &package_config(config.work_root.join(bootstrap_name)),
    )?;
    let bootstrap_runtime = PackagedProductiveRuntimeV1::load(
        &bootstrap.path,
        l11_package_sha256,
        canonical_l2_package_sha256,
    )?;
    record_timing(&mut stage_timings_ms, "bootstrap_package", started);

    progress("actual_candidate_calibration");
    let started = Instant::now();
    let calibration = replay_packaged_calibration(
        &context_sorted,
        &reduced,
        &induction,
        &canonical_l2,
        &bootstrap_runtime,
        &axis_schema,
        &evidence_config,
    )?;
    evidence.compiler_evidence.calibration = calibration.calibration.clone();
    record_timing(
        &mut stage_timings_ms,
        "actual_candidate_calibration",
        started,
    );

    progress("final_deterministic_package");
    let started = Instant::now();
    let final_package = compile_productive_package(
        &reduced,
        &induction,
        &axis_schema,
        &evidence.compiler_evidence,
        &package_config(config.output_path.clone()),
    )?;
    let final_runtime = PackagedProductiveRuntimeV1::load(
        &final_package.path,
        l11_package_sha256,
        canonical_l2_package_sha256,
    )?;
    record_timing(
        &mut stage_timings_ms,
        "final_deterministic_package",
        started,
    );

    let kind = if rebuild_induction {
        "l2_productive_paradigm_field_v1_reinduce_from_reduced"
    } else {
        "l2_productive_paradigm_field_v1_resume_after_induction"
    };
    let reused_complete_stages = if rebuild_induction {
        vec![
            "raw_corpus_pass",
            "morphology_external_sort",
            "imported_ownership_reduce",
            "context_typed_replay",
            "context_external_sort",
        ]
    } else {
        vec![
            "raw_corpus_pass",
            "morphology_external_sort",
            "imported_ownership_reduce",
            "transition_paradigm_induction",
            "context_typed_replay",
            "context_external_sort",
        ]
    };
    Ok(serde_json::json!({
        "kind": kind,
        "verdict": "PASS_shadow_suggest_only_package",
        "runtime_authority_changed": false,
        "workers": config.workers,
        "anchor_recovery_support_policy": if config.shared_support_recovery {
            "fine_applicability_with_shared_certificate"
        } else {
            "fine_support_only"
        },
        "reused_complete_stages": reused_complete_stages,
        "reduced": {
            "train_lemmas": reduced.lemma_count,
            "train_forms": reduced.form_count,
            "train_events": reduced.train_event_count,
            "imported_identity_verified": reduced.imported_identity_verified,
        },
        "induction": {
            "transition_observations": induction.transition_observations,
            "transferable_observations": induction.transferable_observations,
            "exact_allomorph_observations": induction.exact_allomorph_observations,
            "paradigms": induction.paradigm_count,
            "bound_lemma_pos_basins": induction.bound_lemma_count,
            "anchor_recovery_definitions": induction.anchor_recovery.as_ref().map_or(0, |recovery| recovery.definition_count),
        },
        "evidence": {
            "context_occurrence_events": evidence.context_occurrence_events,
            "direct_contradiction_events": evidence.direct_contradiction_events,
            "feedback_events": evidence.feedback_events,
            "proof_events": evidence.proof_events,
            "training_pairs": evidence.training_pairs,
            "bootstrap_calibration_groups": evidence.calibration_groups,
            "phase_profiles": evidence.phase_profiles,
            "selected_phase_centers": evidence.selected_phase_centers,
            "exact_only_morphology_forms": evidence.exact_only_morphology_forms,
            "exact_only_morphology_events": evidence.exact_only_morphology_events,
            "excluded_context_occurrence_events": evidence.excluded_context_occurrence_events,
            "excluded_direct_contradiction_events": evidence.excluded_direct_contradiction_events,
        },
        "actual_calibration": {
            "source_groups": calibration.source_groups,
            "fitted_groups": calibration.fitted_groups,
            "target_retained_groups": calibration.target_retained_groups,
            "target_lost_groups": calibration.target_lost_groups,
            "candidate_rows": calibration.candidate_rows,
            "authority_blocked_by_target_loss": calibration.target_lost_groups != 0,
        },
        "package": {
            "path": final_package.path,
            "bytes": final_package.package_bytes,
            "sha256": hex_sha256(final_package.package_sha256),
            "paradigms": final_package.paradigm_count,
            "bindings": final_package.binding_count,
            "programs": final_package.program_count,
            "operations": final_package.operation_count,
            "trie_nodes": final_package.trie_node_count,
            "trie_arcs": final_package.trie_arc_count,
            "terminals": final_package.terminal_count,
            "anchor_recovery": final_package.anchor_recovery.as_ref().map(|recovery| serde_json::json!({
                "path": recovery.path,
                "bytes": recovery.package_bytes,
                "sha256": hex_sha256(recovery.package_sha256),
                "indexes": recovery.index_count,
                "postings": recovery.posting_count,
                "shared_support_certified_postings": recovery.shared_support_certified_posting_count,
                "programs": recovery.program_count,
                "operations": recovery.operation_count,
            })),
            "mmap_backed": final_runtime.mmap_backed(),
            "resident_cache_bytes": final_runtime.resident_cache_bytes(),
        },
        "stage_timings_ms": stage_timings_ms,
        "total_ms": total_started.elapsed().as_millis() as u64,
        "proof_status": "PENDING_fixed_read_only_proof",
    }))
}

fn validate_config(config: &ProductiveOrchestratorConfigV1) -> Result<(), String> {
    if config.workers == 0
        || config.workers > 256
        || config.expected_corpus_sha256 == [0; 32]
        || config.expected_corpus_bytes == 0
        || !config.l11_package_path.is_file()
        || !config.canonical_l2_path.is_file()
        || !config.corpus_path.is_file()
        || !config.axis_schema_path.is_file()
    {
        return Err("productive orchestrator has an invalid input manifest".to_string());
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<[u8; 32], String> {
    let mut reader =
        BufReader::with_capacity(1024 * 1024, File::open(path).map_err(|e| e.to_string())?);
    let mut buffer = vec![0_u8; 1024 * 1024];
    let mut hasher = Sha256::new();
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().into())
}

fn record_timing(
    output: &mut serde_json::Map<String, serde_json::Value>,
    stage: &str,
    started: Instant,
) {
    output.insert(
        stage.to_string(),
        serde_json::Value::from(started.elapsed().as_millis() as u64),
    );
}

fn hex_sha256(value: [u8; 32]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}
