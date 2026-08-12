use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use sha2::Digest;

use super::format::{ProductivePackageViewV1, ProductiveSectionKindV1};
use super::induce::{SourceAnchorV1, COPY_TO_RETAINED_EDGE};
use super::packaged_runtime::{SharedHypothesisReplayAuditV1, SharedReplayConstraintV1};
use super::proof::{collect_fixed_shared_replay_audits_v1, FixedSharedReplayAuditCaseV1};
use super::records::{
    MorphOpRecordV1, MorphOpcodeV1, MorphProgramHeaderRecordV1, ParadigmCenterRecordV1,
};
use super::runtime::resolve_source_offset;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RawOperationKeyV1 {
    opcode: u8,
    anchor: u8,
    flags: u16,
    arg0: i32,
    arg1: u32,
    arg2: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RawProgramKeyV1 {
    pos_domain: u16,
    source_slot_id: u32,
    target_slot_id: u32,
    flags: u16,
    operations: Vec<RawOperationKeyV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SemanticPieceV1 {
    Copy {
        anchor: u8,
        start_delta: i32,
        scalar_count: u32,
    },
    DropPrefix(u32),
    DropSuffix(u32),
    ReplaceConsume {
        end_relative_offset: i32,
        scalar_count: u32,
    },
    Literal(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticTransformKeyV1 {
    pos_domain: u16,
    source_slot_id: u32,
    target_slot_id: u32,
    pieces: Vec<SemanticPieceV1>,
}

#[derive(Clone, Debug)]
struct SemanticTransformRecordV1 {
    key: SemanticTransformKeyV1,
    owners: Vec<u32>,
    suffix_drop: u16,
    output_length_shape: SemanticLengthShapeV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticLengthShapeV1 {
    source_coefficient: i32,
    constant: i32,
}

impl SemanticLengthShapeV1 {
    fn output_len(self, source_len: usize) -> Option<usize> {
        let source_len = i32::try_from(source_len).ok()?;
        usize::try_from(
            self.source_coefficient
                .checked_mul(source_len)?
                .checked_add(self.constant)?,
        )
        .ok()
    }
}

#[derive(Default)]
struct CollectedSemanticProgramsV1 {
    full_programs: BTreeSet<RawProgramKeyV1>,
    variant_insensitive_programs: BTreeSet<RawProgramKeyV1>,
    transforms: BTreeMap<SemanticTransformKeyV1, SemanticTransformAccumulatorV1>,
    program_semantic_keys: Vec<Option<SemanticTransformKeyV1>>,
    raw_owner_memberships: u64,
    exact_allomorph_programs: u64,
    paradigms_by_pos: BTreeMap<u16, u64>,
}

#[derive(Clone, Debug)]
struct SemanticTransformAccumulatorV1 {
    owners: BTreeSet<u32>,
    suffix_drop: u16,
}

#[derive(Clone, Debug)]
pub(super) struct SemanticExecutionIndexV1 {
    transforms_by_target:
        BTreeMap<(u16, u32, u32), BTreeMap<SemanticLengthShapeV1, Vec<SemanticTransformRecordV1>>>,
    transforms_per_source: BTreeMap<(u16, u32), usize>,
    transforms_per_source_target: BTreeMap<(u16, u32, u32), usize>,
    execution_class_by_program: Box<[u32]>,
    execution_class_count: u32,
    paradigm_count: usize,
    maximum_observed_scalars: usize,
    maximum_generated_scalars: usize,
}

#[derive(Default)]
struct PrefixNodeV1 {
    children: BTreeMap<SemanticDagTokenV1, usize>,
    terminal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SemanticDagTokenV1 {
    Domain {
        pos_domain: u16,
        source_slot_id: u32,
    },
    Piece(SemanticPieceV1),
    TargetSlot(u32),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CanonicalStateKeyV1 {
    terminal: bool,
    children: Vec<(SemanticDagTokenV1, usize)>,
}

#[derive(Clone, Debug, Serialize)]
struct CardinalitySummaryV1 {
    minimum: u64,
    p50: u64,
    p95: u64,
    p99: u64,
    maximum: u64,
}

pub(crate) fn estimate_productive_semantic_transducer_v1(
    package_path: &Path,
) -> Result<serde_json::Value, String> {
    let package = ProductivePackageViewV1::load(package_path)?;
    let paradigms = decode_records::<ParadigmCenterRecordV1>(
        &package,
        ProductiveSectionKindV1::ParadigmCenters,
    )?;
    let program_count = package.record_count(ProductiveSectionKindV1::MorphProgramHeaders);
    let operation_count = package.record_count(ProductiveSectionKindV1::MorphOperations);

    let collected =
        collect_semantic_programs(&package, &paradigms, program_count, operation_count)?;
    let full_programs = collected.full_programs;
    let variant_insensitive_programs = collected.variant_insensitive_programs;
    let semantic_owners = collected
        .transforms
        .iter()
        .map(|(key, value)| (key.clone(), value.owners.clone()))
        .collect::<BTreeMap<_, _>>();
    let raw_owner_memberships = collected.raw_owner_memberships;
    let exact_allomorph_programs = collected.exact_allomorph_programs;
    let paradigms_by_pos = collected.paradigms_by_pos;

    if exact_allomorph_programs != 0 {
        return Err(format!(
            "productive semantic estimator found {exact_allomorph_programs} exact allomorph programs in transferable ranges"
        ));
    }

    let unique_semantic_transforms = semantic_owners.len() as u64;
    let semantic_paradigm_memberships = semantic_owners
        .values()
        .map(|owners| owners.len() as u64)
        .sum::<u64>();
    let owner_cardinalities = semantic_owners
        .values()
        .map(|owners| owners.len() as u64)
        .collect::<Vec<_>>();
    let owner_cardinality = cardinality_summary(owner_cardinalities);

    let (prefix_nodes, prefix_arcs, minimized_dag_states, minimized_dag_arcs) =
        semantic_dag_counts(semantic_owners.keys());
    let canonical_piece_payload_bytes = semantic_owners
        .keys()
        .map(semantic_key_payload_bytes)
        .sum::<u64>();
    let sparse_owner_bytes = semantic_owners
        .values()
        .map(|owners| 8_u64 + owners.len() as u64 * 4)
        .sum::<u64>();
    let dense_owner_bytes = semantic_owners
        .iter()
        .map(|(transform, _)| {
            let paradigms = paradigms_by_pos
                .get(&transform.pos_domain)
                .copied()
                .unwrap_or_default();
            8_u64 + paradigms.div_ceil(64) * 8
        })
        .sum::<u64>();
    let hybrid_owner_bytes = semantic_owners
        .iter()
        .map(|(transform, owners)| {
            let sparse = 8_u64 + owners.len() as u64 * 4;
            let paradigms = paradigms_by_pos
                .get(&transform.pos_domain)
                .copied()
                .unwrap_or_default();
            let dense = 8_u64 + paradigms.div_ceil(64) * 8;
            sparse.min(dense)
        })
        .sum::<u64>();
    let estimated_semantic_sidecar_bytes = 192_u64
        + unique_semantic_transforms * 32
        + canonical_piece_payload_bytes
        + hybrid_owner_bytes
        + minimized_dag_states * 16
        + minimized_dag_arcs * 24;

    Ok(serde_json::json!({
        "kind": "l2_productive_v80_semantic_transducer_structural_estimate",
        "scope": "validated existing package; no package write; no runtime authority",
        "package": package_path,
        "package_bytes": std::fs::metadata(package_path).map_err(|error| error.to_string())?.len(),
        "package_sha256": hex_sha256(package.package_sha256()),
        "package_compiler_version": package.header.compiler_version,
        "package_normalization_version": package.header.normalization_version,
        "paradigms": paradigms.len(),
        "pos_domains": paradigms_by_pos.len(),
        "package_program_records": program_count,
        "package_operation_records": operation_count,
        "transferable_owner_programs": raw_owner_memberships,
        "unique_full_programs": full_programs.len(),
        "unique_variant_insensitive_programs": variant_insensitive_programs.len(),
        "variant_identity_only_duplicates_removed": full_programs.len().saturating_sub(variant_insensitive_programs.len()),
        "unique_conservative_semantic_transforms": unique_semantic_transforms,
        "instruction_decomposition_duplicates_removed": variant_insensitive_programs.len().saturating_sub(semantic_owners.len()),
        "semantic_paradigm_memberships": semantic_paradigm_memberships,
        "semantic_owner_cardinality": owner_cardinality,
        "prefix_trie": {
            "nodes": prefix_nodes,
            "arcs": prefix_arcs,
        },
        "minimized_semantic_dag": {
            "states": minimized_dag_states,
            "arcs": minimized_dag_arcs,
        },
        "storage_estimate": {
            "canonical_piece_payload_bytes": canonical_piece_payload_bytes,
            "sparse_owner_bytes": sparse_owner_bytes,
            "dense_owner_bytes": dense_owner_bytes,
            "hybrid_owner_bytes": hybrid_owner_bytes,
            "semantic_sidecar_bytes": estimated_semantic_sidecar_bytes,
            "formula": "192 + transforms*32 + canonical_piece_payload + hybrid_owners + dag_states*16 + dag_arcs*24",
        },
        "safety": {
            "segment_refs_are_package_global_canonical": true,
            "terminate_variant_excluded_from_execution_semantics": true,
            "target_slot_retained": true,
            "full_key_equality_not_hash_only": true,
            "transferable_exact_allomorph_programs": exact_allomorph_programs,
            "runtime_authority_changed": false,
        },
        "measured": [
            "validated package structure",
            "conservative semantic transform cardinality",
            "owner cardinality and representation byte estimates",
            "prefix trie and minimized DAG structural counts",
        ],
        "not_tested": [
            "fixed proof cases or candidate parity",
            "per-request invariant selectivity",
            "semantic executor output parity",
            "cold or hot latency",
            "L1.1/L2/L3/L4/verifier integration",
            "daemon, IBus, or physical input",
        ],
        "verdict": "STRUCTURAL_ESTIMATE_ONLY",
        "runtime_authority_changed": false,
    }))
}

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct SemanticWorkMetricsV1 {
    pub(super) hypotheses: u64,
    pub(super) constraints: u64,
    pub(super) transforms_in_source_basin: u64,
    pub(super) target_slot_filtered: u64,
    pub(super) output_length_index_filtered: u64,
    pub(super) fixed_literal_edge_filtered: u64,
    pub(super) transforms_executed: u64,
    pub(super) invalid_source_guards: u64,
    pub(super) canonical_token_steps: u64,
    pub(super) emitted_scalars: u64,
    pub(super) output_length_rejected: u64,
    pub(super) exact_output_lookups: u64,
    pub(super) exact_terminal_matches: u64,
    pub(super) owner_memberships_visited: u64,
    pub(super) owner_bitset_word_updates: u64,
    pub(super) owner_bitset_intersection_words: u64,
    pub(super) owner_sparse_unique_memberships: u64,
    pub(super) owner_sparse_intersection_probes: u64,
    pub(super) final_owner_candidates: u64,
    pub(super) owner_parity_failures: u64,
    pub(super) missing_direct_owners: u64,
    pub(super) extra_semantic_owners: u64,
}

impl SemanticWorkMetricsV1 {
    fn merge(&mut self, other: &Self) {
        self.hypotheses += other.hypotheses;
        self.constraints += other.constraints;
        self.transforms_in_source_basin += other.transforms_in_source_basin;
        self.target_slot_filtered += other.target_slot_filtered;
        self.output_length_index_filtered += other.output_length_index_filtered;
        self.fixed_literal_edge_filtered += other.fixed_literal_edge_filtered;
        self.transforms_executed += other.transforms_executed;
        self.invalid_source_guards += other.invalid_source_guards;
        self.canonical_token_steps += other.canonical_token_steps;
        self.emitted_scalars += other.emitted_scalars;
        self.output_length_rejected += other.output_length_rejected;
        self.exact_output_lookups += other.exact_output_lookups;
        self.exact_terminal_matches += other.exact_terminal_matches;
        self.owner_memberships_visited += other.owner_memberships_visited;
        self.owner_bitset_word_updates += other.owner_bitset_word_updates;
        self.owner_bitset_intersection_words += other.owner_bitset_intersection_words;
        self.owner_sparse_unique_memberships += other.owner_sparse_unique_memberships;
        self.owner_sparse_intersection_probes += other.owner_sparse_intersection_probes;
        self.final_owner_candidates += other.final_owner_candidates;
        self.owner_parity_failures += other.owner_parity_failures;
        self.missing_direct_owners += other.missing_direct_owners;
        self.extra_semantic_owners += other.extra_semantic_owners;
    }
}

struct SemanticCaseResultV1 {
    class: &'static str,
    metrics: SemanticWorkMetricsV1,
    latency_us: u64,
    failure_examples: Vec<serde_json::Value>,
}

struct SemanticExecutionV1 {
    output: String,
    emitted_scalars: u64,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn estimate_productive_semantic_transducer_heldout_v1(
    l1_package_path: &Path,
    l2_package_path: &Path,
    productive_package_path: &Path,
    axis_schema_path: &Path,
    work_dir: &Path,
    heldout_per_class: usize,
    requested_workers: usize,
) -> io::Result<serde_json::Value> {
    let total_started = Instant::now();
    let structural = estimate_productive_semantic_transducer_v1(productive_package_path)
        .map_err(io::Error::other)?;
    let index_started = Instant::now();
    let index =
        SemanticExecutionIndexV1::load(productive_package_path).map_err(io::Error::other)?;
    let index_load_us = index_started.elapsed().as_micros() as u64;
    let audit_corpus = collect_fixed_shared_replay_audits_v1(
        l1_package_path,
        l2_package_path,
        productive_package_path,
        axis_schema_path,
        work_dir,
        heldout_per_class,
        requested_workers,
    )?;
    let semantic_started = Instant::now();
    let workers = requested_workers
        .max(1)
        .min(
            std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
        )
        .min(audit_corpus.cases.len().max(1));
    let mut worker_cases = vec![Vec::new(); workers];
    for case_index in 0..audit_corpus.cases.len() {
        worker_cases[case_index % workers].push(case_index);
    }
    let case_results = std::thread::scope(|scope| {
        let cases = &audit_corpus.cases;
        let index = &index;
        worker_cases
            .iter()
            .map(|indices| {
                scope.spawn(move || {
                    indices
                        .iter()
                        .map(|case_index| evaluate_semantic_case(index, &cases[*case_index]))
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .map_err(|_| "productive semantic estimator worker panicked".to_string())?
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .map_err(io::Error::other)?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let semantic_evaluation_us = semantic_started.elapsed().as_micros() as u64;

    let mut aggregate = SemanticWorkMetricsV1::default();
    let mut by_class = BTreeMap::<&'static str, (SemanticWorkMetricsV1, Vec<u64>)>::new();
    let mut case_latencies = Vec::with_capacity(case_results.len());
    let mut failure_examples = Vec::new();
    for result in &case_results {
        aggregate.merge(&result.metrics);
        let class = by_class.entry(result.class).or_default();
        class.0.merge(&result.metrics);
        class.1.push(result.latency_us);
        case_latencies.push(result.latency_us);
        if failure_examples.len() < 20 {
            failure_examples.extend(
                result
                    .failure_examples
                    .iter()
                    .take(20 - failure_examples.len())
                    .cloned(),
            );
        }
    }
    let classes = by_class
        .into_iter()
        .map(|(class, (metrics, latencies))| {
            (
                class,
                serde_json::json!({
                    "cases": latencies.len(),
                    "work": metrics,
                    "semantic_prepared_case_latency_us": latency_summary(latencies),
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let parity_exact = aggregate.owner_parity_failures == 0;
    let binding_parity_exact = audit_corpus.binding_parity_failures == 0;
    let binding_failure_examples = audit_corpus
        .cases
        .iter()
        .filter(|case| !case.binding_parity_exact)
        .take(20)
        .map(|case| {
            serde_json::json!({
                "class": case.class,
                "proof_identity": hex_sha256(case.proof_identity),
                "legacy_binding_count": case.legacy_binding_count,
                "semantic_binding_count": case.semantic_binding_count,
            })
        })
        .collect::<Vec<_>>();
    let legacy_grounding_latency = latency_summary(
        audit_corpus
            .cases
            .iter()
            .map(|case| case.legacy_grounding_us)
            .collect(),
    );
    let semantic_grounding_latency = latency_summary(
        audit_corpus
            .cases
            .iter()
            .map(|case| case.semantic_grounding_us)
            .collect(),
    );
    let forecast_multiplier = 100_u64.div_ceil(heldout_per_class as u64);
    let total_us = total_started.elapsed().as_micros() as u64;
    Ok(serde_json::json!({
        "kind": "l2_productive_v80_semantic_transducer_fixed_heldout_estimate",
        "scope": "read-only V70 package and frozen LEMMA_HELDOUT proof; no package write; no runtime authority",
        "structural": structural,
        "packages": {
            "l11": l1_package_path,
            "canonical_l2": l2_package_path,
            "productive_v1": productive_package_path,
            "axis_schema": axis_schema_path,
        },
        "proof_spool": work_dir.join("context-sorted/sorted-events-global.p2s"),
        "heldout_per_class": heldout_per_class,
        "lemma_heldout_cases": audit_corpus.cases.len(),
        "scanned_proof_events": audit_corpus.scanned_proof_events,
        "sampled_events_by_cohort": audit_corpus.sampled_events_by_cohort,
        "workers": workers,
        "work": aggregate,
        "classes": classes,
        "semantic_prepared_case_latency_us": latency_summary(case_latencies),
        "full_13x100_linear_forecast": {
            "multiplier_from_sample": forecast_multiplier,
            "hypotheses": aggregate.hypotheses.saturating_mul(forecast_multiplier),
            "output_length_index_filtered": aggregate.output_length_index_filtered.saturating_mul(forecast_multiplier),
            "fixed_literal_edge_filtered": aggregate.fixed_literal_edge_filtered.saturating_mul(forecast_multiplier),
            "transforms_executed": aggregate.transforms_executed.saturating_mul(forecast_multiplier),
            "canonical_token_steps": aggregate.canonical_token_steps.saturating_mul(forecast_multiplier),
            "emitted_scalars": aggregate.emitted_scalars.saturating_mul(forecast_multiplier),
            "exact_output_lookups": aggregate.exact_output_lookups.saturating_mul(forecast_multiplier),
            "owner_bitset_word_updates": aggregate.owner_bitset_word_updates.saturating_mul(forecast_multiplier),
            "owner_bitset_intersection_words": aggregate.owner_bitset_intersection_words.saturating_mul(forecast_multiplier),
            "owner_sparse_unique_memberships": aggregate.owner_sparse_unique_memberships.saturating_mul(forecast_multiplier),
            "owner_sparse_intersection_probes": aggregate.owner_sparse_intersection_probes.saturating_mul(forecast_multiplier),
            "final_owner_candidates": aggregate.final_owner_candidates.saturating_mul(forecast_multiplier),
        },
        "owner_parity": {
            "comparison": "full exact paradigm owner set for every V70 shared hypothesis",
            "exact": parity_exact,
            "failures": aggregate.owner_parity_failures,
            "missing_direct_owners": aggregate.missing_direct_owners,
            "extra_semantic_owners": aggregate.extra_semantic_owners,
            "failure_examples": failure_examples,
        },
        "binding_parity": {
            "comparison": "complete ordered ColdLemmaBinding vector from legacy and semantic proof-authority runtimes",
            "exact": binding_parity_exact,
            "comparisons": audit_corpus.binding_parity_comparisons,
            "failures": audit_corpus.binding_parity_failures,
            "legacy_grounding_us": audit_corpus.legacy_grounding_us,
            "semantic_grounding_us": audit_corpus.semantic_grounding_us,
            "legacy_case_latency_us": legacy_grounding_latency,
            "semantic_case_latency_us": semantic_grounding_latency,
            "failure_examples": binding_failure_examples,
        },
        "latency": {
            "semantic_index_load_us": index_load_us,
            "proof_runtime_cold_load_us": audit_corpus.cold_load_us,
            "fixed_proof_sampling_us": audit_corpus.sampling_us,
            "v70_trace_preparation_us": audit_corpus.preparation_us,
            "semantic_evaluation_us": semantic_evaluation_us,
            "total_estimator_us": total_us,
            "measurement_boundary": "semantic case latency begins after V70 has materialized the read-only shared-hypothesis audit trace",
            "isolated_v80_end_to_end_cold_request_measured": false,
        },
        "measured": [
            "same deterministic 13-class LEMMA_HELDOUT sample",
            "V70 eligible and direct-exact paradigm owners per shared hypothesis",
            "semantic source/target filtering, exact output lookup, owner-set intersection",
            "prepared-trace semantic latency and complete owner parity",
        ],
        "not_tested": [
            "isolated V80 cold request including shared-hypothesis birth",
            "real minimized-DAG traversal arcs",
            "candidate ordering or final readout parity under a V80 runtime path",
            "one-worker product p99",
            "daemon, IBus, or physical input",
        ],
        "verdict": if parity_exact && binding_parity_exact { "OWNER_AND_BINDING_PARITY_PASS_RUNTIME_NOT_AUTHORIZED" } else { "OWNER_OR_BINDING_PARITY_FAIL" },
        "runtime_authority_changed": false,
    }))
}

impl SemanticExecutionIndexV1 {
    pub(super) fn load(package_path: &Path) -> Result<Self, String> {
        Self::from_package(&ProductivePackageViewV1::load(package_path)?)
    }

    pub(super) fn from_package(package: &ProductivePackageViewV1) -> Result<Self, String> {
        let paradigms = decode_records::<ParadigmCenterRecordV1>(
            package,
            ProductiveSectionKindV1::ParadigmCenters,
        )?;
        let program_count = package.record_count(ProductiveSectionKindV1::MorphProgramHeaders);
        let operation_count = package.record_count(ProductiveSectionKindV1::MorphOperations);
        let collected =
            collect_semantic_programs(package, &paradigms, program_count, operation_count)?;
        if collected.exact_allomorph_programs != 0 {
            return Err(
                "productive semantic execution index contains exact allomorphs".to_string(),
            );
        }
        let mut transforms_by_target = BTreeMap::<
            (u16, u32, u32),
            BTreeMap<SemanticLengthShapeV1, Vec<SemanticTransformRecordV1>>,
        >::new();
        let mut transforms_per_source = BTreeMap::<(u16, u32), usize>::new();
        let mut transforms_per_source_target = BTreeMap::<(u16, u32, u32), usize>::new();
        let execution_classes = collected
            .transforms
            .keys()
            .enumerate()
            .map(|(index, key)| {
                let class_id = u32::try_from(index + 1)
                    .map_err(|_| "productive semantic execution class exceeds u32")?;
                Ok((key.clone(), class_id))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        let execution_class_by_program = collected
            .program_semantic_keys
            .iter()
            .map(|key| {
                key.as_ref().map_or(Ok(0), |key| {
                    execution_classes
                        .get(key)
                        .copied()
                        .ok_or_else(|| "productive program semantic class disappeared".to_string())
                })
            })
            .collect::<Result<Vec<_>, String>>()?
            .into_boxed_slice();
        let execution_class_count = u32::try_from(execution_classes.len())
            .map_err(|_| "productive semantic execution class count exceeds u32")?;
        for (key, value) in collected.transforms {
            let output_length_shape = semantic_output_length_shape(&key.pieces, value.suffix_drop)?;
            *transforms_per_source
                .entry((key.pos_domain, key.source_slot_id))
                .or_default() += 1;
            *transforms_per_source_target
                .entry((key.pos_domain, key.source_slot_id, key.target_slot_id))
                .or_default() += 1;
            transforms_by_target
                .entry((key.pos_domain, key.source_slot_id, key.target_slot_id))
                .or_default()
                .entry(output_length_shape)
                .or_default()
                .push(SemanticTransformRecordV1 {
                    key,
                    owners: value.owners.into_iter().collect(),
                    suffix_drop: value.suffix_drop,
                    output_length_shape,
                });
        }
        Ok(Self {
            transforms_by_target,
            transforms_per_source,
            transforms_per_source_target,
            execution_class_by_program,
            execution_class_count,
            paradigm_count: paradigms.len(),
            maximum_observed_scalars: usize::from(package.header.maximum_observed_scalars),
            maximum_generated_scalars: usize::from(package.header.maximum_generated_scalars),
        })
    }

    pub(super) fn execution_class_count(&self) -> usize {
        self.execution_class_count as usize
    }

    pub(super) fn execution_class_for_program(&self, program_index: usize) -> Result<u32, String> {
        self.execution_class_by_program
            .get(program_index)
            .copied()
            .filter(|class_id| *class_id != 0 && *class_id <= self.execution_class_count)
            .ok_or_else(|| "productive program semantic class is invalid".to_string())
    }

    pub(super) fn exact_owners(
        &self,
        pos_domain: u16,
        anchor_slot_id: u32,
        normalized_source: &str,
        constraints: &[SharedReplayConstraintV1],
        eligible_paradigm_ids: &[u32],
    ) -> Result<(Vec<u32>, SemanticWorkMetricsV1), String> {
        evaluate_shared_hypothesis(
            self,
            &SharedHypothesisReplayAuditV1 {
                pos_domain,
                anchor_slot_id,
                normalized_source: normalized_source.to_string(),
                constraints: constraints.to_vec(),
                eligible_paradigm_ids: eligible_paradigm_ids.to_vec(),
                direct_exact_paradigm_ids: Vec::new(),
            },
        )
    }
}

fn evaluate_semantic_case(
    index: &SemanticExecutionIndexV1,
    case: &FixedSharedReplayAuditCaseV1,
) -> Result<SemanticCaseResultV1, String> {
    let started = Instant::now();
    let mut metrics = SemanticWorkMetricsV1::default();
    let mut failure_examples = Vec::new();
    for audit in &case.audits {
        let (semantic_owners, audit_metrics) = evaluate_shared_hypothesis(index, audit)?;
        metrics.merge(&audit_metrics);
        if semantic_owners != audit.direct_exact_paradigm_ids {
            metrics.owner_parity_failures += 1;
            let direct = audit
                .direct_exact_paradigm_ids
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            let semantic = semantic_owners.iter().copied().collect::<BTreeSet<_>>();
            let missing = direct.difference(&semantic).copied().collect::<Vec<_>>();
            let extra = semantic.difference(&direct).copied().collect::<Vec<_>>();
            metrics.missing_direct_owners += missing.len() as u64;
            metrics.extra_semantic_owners += extra.len() as u64;
            if failure_examples.len() < 4 {
                failure_examples.push(serde_json::json!({
                    "class": case.class,
                    "proof_identity": hex_sha256(case.proof_identity),
                    "pos_domain": audit.pos_domain,
                    "anchor_slot_id": audit.anchor_slot_id,
                    "source_sha256": hex_sha256(sha2::Sha256::digest(audit.normalized_source.as_bytes()).into()),
                    "direct_owner_count": direct.len(),
                    "semantic_owner_count": semantic.len(),
                    "missing_owner_ids": missing,
                    "extra_owner_ids": extra,
                }));
            }
        }
    }
    Ok(SemanticCaseResultV1 {
        class: case.class,
        metrics,
        latency_us: started.elapsed().as_micros() as u64,
        failure_examples,
    })
}

fn evaluate_shared_hypothesis(
    index: &SemanticExecutionIndexV1,
    audit: &SharedHypothesisReplayAuditV1,
) -> Result<(Vec<u32>, SemanticWorkMetricsV1), String> {
    let mut metrics = SemanticWorkMetricsV1 {
        hypotheses: 1,
        constraints: audit.constraints.len() as u64,
        ..SemanticWorkMetricsV1::default()
    };
    let words = (index.paradigm_count + 64) / 64;
    let mut eligible = vec![0_u64; words];
    for paradigm_id in &audit.eligible_paradigm_ids {
        set_owner_bit(&mut eligible, *paradigm_id)?;
    }
    let mut owners_by_constraint = vec![Vec::<u32>::new(); audit.constraints.len()];
    let mut constraints_by_slot = BTreeMap::<u32, BTreeMap<&str, usize>>::new();
    let mut constraint_lengths_by_slot = BTreeMap::<u32, BTreeSet<usize>>::new();
    let source = audit.normalized_source.chars().collect::<Vec<_>>();
    for (index, constraint) in audit.constraints.iter().enumerate() {
        constraints_by_slot
            .entry(constraint.slot_id)
            .or_default()
            .insert(&constraint.normalized_surface, index);
        constraint_lengths_by_slot
            .entry(constraint.slot_id)
            .or_default()
            .insert(constraint.normalized_surface.chars().count());
    }
    let source_key = (audit.pos_domain, audit.anchor_slot_id);
    let source_transform_count = index
        .transforms_per_source
        .get(&source_key)
        .copied()
        .unwrap_or_default();
    metrics.transforms_in_source_basin += source_transform_count as u64;
    let exposed_slots = constraints_by_slot.keys().copied().collect::<BTreeSet<_>>();
    let exposed_transform_count = exposed_slots
        .iter()
        .map(|slot_id| {
            index
                .transforms_per_source_target
                .get(&(audit.pos_domain, audit.anchor_slot_id, *slot_id))
                .copied()
                .unwrap_or_default()
        })
        .sum::<usize>();
    metrics.target_slot_filtered +=
        source_transform_count.saturating_sub(exposed_transform_count) as u64;

    let mut shape_transform_count = 0_usize;
    for (slot_id, expected_lengths) in constraint_lengths_by_slot {
        let transform_shapes =
            index
                .transforms_by_target
                .get(&(audit.pos_domain, audit.anchor_slot_id, slot_id));
        let slot_constraints = constraints_by_slot
            .get(&slot_id)
            .ok_or("productive semantic constraint slot disappeared")?;
        if let Some(transform_shapes) = transform_shapes {
            for (shape, transforms) in transform_shapes {
                if !shape
                    .output_len(source.len())
                    .is_some_and(|output_len| expected_lengths.contains(&output_len))
                {
                    continue;
                }
                shape_transform_count += transforms.len();
                for transform in transforms {
                    let edge_compatible = slot_constraints
                        .keys()
                        .any(|surface| semantic_fixed_edges_match(transform, surface));
                    if !edge_compatible {
                        metrics.fixed_literal_edge_filtered += 1;
                        continue;
                    }
                    metrics.transforms_executed += 1;
                    metrics.canonical_token_steps += transform.key.pieces.len() as u64 + 2;
                    let Some(execution) = execute_semantic_transform(index, transform, &source)?
                    else {
                        metrics.invalid_source_guards += 1;
                        continue;
                    };
                    metrics.emitted_scalars += execution.emitted_scalars;
                    debug_assert_eq!(
                        Some(execution.output.chars().count()),
                        transform.output_length_shape.output_len(source.len())
                    );
                    metrics.exact_output_lookups += 1;
                    let Some(constraint_index) =
                        slot_constraints.get(execution.output.as_str()).copied()
                    else {
                        continue;
                    };
                    metrics.exact_terminal_matches += 1;
                    for paradigm_id in &transform.owners {
                        metrics.owner_memberships_visited += 1;
                        if owner_bit(&eligible, *paradigm_id)? {
                            owners_by_constraint[constraint_index].push(*paradigm_id);
                        }
                    }
                }
            }
        }
    }
    metrics.output_length_index_filtered +=
        exposed_transform_count.saturating_sub(shape_transform_count) as u64;

    for owners in &mut owners_by_constraint {
        owners.sort_unstable();
        owners.dedup();
        metrics.owner_sparse_unique_memberships += owners.len() as u64;
    }
    owners_by_constraint.sort_unstable_by_key(Vec::len);
    let mut owners = owners_by_constraint.first().cloned().unwrap_or_else(|| {
        let mut eligible = audit.eligible_paradigm_ids.clone();
        eligible.sort_unstable();
        eligible.dedup();
        eligible
    });
    for constraint_owners in owners_by_constraint.iter().skip(1) {
        owners.retain(|owner| {
            metrics.owner_sparse_intersection_probes += 1;
            constraint_owners.binary_search(owner).is_ok()
        });
        if owners.is_empty() {
            break;
        }
    }
    metrics.final_owner_candidates += owners.len() as u64;
    Ok((owners, metrics))
}

fn execute_semantic_transform(
    index: &SemanticExecutionIndexV1,
    transform: &SemanticTransformRecordV1,
    source: &[char],
) -> Result<Option<SemanticExecutionV1>, String> {
    if source.len() > index.maximum_observed_scalars {
        return Ok(None);
    }
    let Some(retained_end) = source.len().checked_sub(usize::from(transform.suffix_drop)) else {
        return Ok(None);
    };
    let mut output = String::new();
    let mut cursor = 0_usize;
    for (piece_index, piece) in transform.key.pieces.iter().enumerate() {
        match piece {
            SemanticPieceV1::Copy {
                anchor,
                start_delta,
                scalar_count,
            } => {
                let delta = i16::try_from(*start_delta)
                    .map_err(|_| "productive semantic source delta exceeds i16")?;
                let Some(start) =
                    resolve_source_offset(source.len(), decode_semantic_anchor(*anchor)?, delta)
                else {
                    return Ok(None);
                };
                let end = if *scalar_count == u32::from(COPY_TO_RETAINED_EDGE) {
                    if let Some(SemanticPieceV1::ReplaceConsume {
                        end_relative_offset,
                        ..
                    }) = transform.key.pieces.get(piece_index + 1)
                    {
                        let Some(end) = source
                            .len()
                            .checked_add_signed(*end_relative_offset as isize)
                        else {
                            return Ok(None);
                        };
                        end
                    } else {
                        retained_end
                    }
                } else {
                    let Some(end) = start.checked_add(*scalar_count as usize) else {
                        return Ok(None);
                    };
                    end
                };
                if start != cursor || end <= start || end > retained_end {
                    return Ok(None);
                }
                output.extend(source[start..end].iter());
                cursor = end;
            }
            SemanticPieceV1::DropPrefix(count) => {
                let count = *count as usize;
                if cursor != 0 || count == 0 || count > source.len() {
                    return Ok(None);
                }
                cursor = count;
            }
            SemanticPieceV1::DropSuffix(count) => {
                let count = *count as usize;
                if count == 0 || cursor.checked_add(count) != Some(source.len()) {
                    return Ok(None);
                }
                cursor = source.len();
            }
            SemanticPieceV1::ReplaceConsume {
                end_relative_offset,
                scalar_count,
            } => {
                let Some(start) = source
                    .len()
                    .checked_add_signed(*end_relative_offset as isize)
                else {
                    return Ok(None);
                };
                let Some(end) = start.checked_add(*scalar_count as usize) else {
                    return Ok(None);
                };
                if start != cursor || end > source.len() {
                    return Ok(None);
                }
                cursor = end;
            }
            SemanticPieceV1::Literal(bytes) => {
                output.push_str(
                    std::str::from_utf8(bytes)
                        .map_err(|_| "productive semantic literal is not UTF-8")?,
                );
            }
        }
    }
    let emitted_scalars = output.chars().count();
    if cursor != source.len()
        || emitted_scalars == 0
        || emitted_scalars > index.maximum_generated_scalars
    {
        return Ok(None);
    }
    Ok(Some(SemanticExecutionV1 {
        output,
        emitted_scalars: emitted_scalars as u64,
    }))
}

fn semantic_output_length_shape(
    pieces: &[SemanticPieceV1],
    suffix_drop: u16,
) -> Result<SemanticLengthShapeV1, String> {
    let mut source_coefficient = 0_i32;
    let mut constant = 0_i32;
    for (piece_index, piece) in pieces.iter().enumerate() {
        match piece {
            SemanticPieceV1::Copy {
                anchor,
                start_delta,
                scalar_count,
            } if *scalar_count == u32::from(COPY_TO_RETAINED_EDGE) => {
                let start_delta = i16::try_from(*start_delta)
                    .map_err(|_| "productive semantic source delta exceeds i16")?;
                let end_constant = if let Some(SemanticPieceV1::ReplaceConsume {
                    end_relative_offset,
                    ..
                }) = pieces.get(piece_index + 1)
                {
                    *end_relative_offset
                } else {
                    -i32::from(suffix_drop)
                };
                let start_coefficient = match decode_semantic_anchor(*anchor)? {
                    SourceAnchorV1::Start => 0_i32,
                    SourceAnchorV1::End => 1_i32,
                };
                source_coefficient = source_coefficient
                    .checked_add(1 - start_coefficient)
                    .ok_or("productive semantic length coefficient overflow")?;
                constant = constant
                    .checked_add(end_constant)
                    .and_then(|value| value.checked_sub(i32::from(start_delta)))
                    .ok_or("productive semantic length constant overflow")?;
            }
            SemanticPieceV1::Copy { scalar_count, .. } => {
                constant = constant
                    .checked_add(
                        i32::try_from(*scalar_count)
                            .map_err(|_| "productive semantic copy length exceeds i32")?,
                    )
                    .ok_or("productive semantic length constant overflow")?;
            }
            SemanticPieceV1::Literal(bytes) => {
                let scalars = std::str::from_utf8(bytes)
                    .map_err(|_| "productive semantic literal is not UTF-8")?
                    .chars()
                    .count();
                constant = constant
                    .checked_add(
                        i32::try_from(scalars)
                            .map_err(|_| "productive semantic literal length exceeds i32")?,
                    )
                    .ok_or("productive semantic length constant overflow")?;
            }
            SemanticPieceV1::DropPrefix(_)
            | SemanticPieceV1::DropSuffix(_)
            | SemanticPieceV1::ReplaceConsume { .. } => {}
        }
    }
    Ok(SemanticLengthShapeV1 {
        source_coefficient,
        constant,
    })
}

fn semantic_fixed_edges_match(transform: &SemanticTransformRecordV1, expected: &str) -> bool {
    let expected = expected.as_bytes();
    if let Some(SemanticPieceV1::Literal(prefix)) = transform.key.pieces.first() {
        if !expected.starts_with(prefix) {
            return false;
        }
    }
    if let Some(SemanticPieceV1::Literal(suffix)) = transform.key.pieces.last() {
        if !expected.ends_with(suffix) {
            return false;
        }
    }
    true
}

fn decode_semantic_anchor(value: u8) -> Result<SourceAnchorV1, String> {
    match value {
        1 => Ok(SourceAnchorV1::Start),
        2 => Ok(SourceAnchorV1::End),
        _ => Err("productive semantic transform has an invalid source anchor".to_string()),
    }
}

fn set_owner_bit(words: &mut [u64], paradigm_id: u32) -> Result<(), String> {
    let index = paradigm_id as usize;
    let Some(word) = words.get_mut(index / 64) else {
        return Err("productive semantic owner identity exceeds bitset".to_string());
    };
    *word |= 1_u64 << (index % 64);
    Ok(())
}

fn owner_bit(words: &[u64], paradigm_id: u32) -> Result<bool, String> {
    let index = paradigm_id as usize;
    let Some(word) = words.get(index / 64) else {
        return Err("productive semantic owner identity exceeds bitset".to_string());
    };
    Ok(*word & (1_u64 << (index % 64)) != 0)
}

fn latency_summary(mut values: Vec<u64>) -> CardinalitySummaryV1 {
    values.sort_unstable();
    CardinalitySummaryV1 {
        minimum: values.first().copied().unwrap_or_default(),
        p50: percentile(&values, 50),
        p95: percentile(&values, 95),
        p99: percentile(&values, 99),
        maximum: values.last().copied().unwrap_or_default(),
    }
}

fn collect_semantic_programs(
    package: &ProductivePackageViewV1,
    paradigms: &[ParadigmCenterRecordV1],
    program_count: usize,
    operation_count: usize,
) -> Result<CollectedSemanticProgramsV1, String> {
    let mut collected = CollectedSemanticProgramsV1 {
        program_semantic_keys: vec![None; program_count],
        ..CollectedSemanticProgramsV1::default()
    };
    for (index, paradigm) in paradigms.iter().copied().enumerate() {
        let paradigm_id = u32::try_from(index + 1)
            .map_err(|_| "productive semantic estimator paradigm identity exceeds u32")?;
        *collected
            .paradigms_by_pos
            .entry(paradigm.pos_domain)
            .or_default() += 1;
        let start = paradigm.transition_start as usize;
        let end = start
            .checked_add(paradigm.transition_count as usize)
            .filter(|end| *end <= program_count)
            .ok_or("productive semantic estimator transition range is invalid")?;
        for program_index in start..end {
            collected.raw_owner_memberships += 1;
            let program = package.record::<MorphProgramHeaderRecordV1>(
                ProductiveSectionKindV1::MorphProgramHeaders,
                program_index,
            )?;
            let operations = program_operations(package, program, operation_count)?;
            collected.full_programs.insert(raw_program_key(
                paradigm.pos_domain,
                program,
                &operations,
                false,
            )?);
            collected
                .variant_insensitive_programs
                .insert(raw_program_key(
                    paradigm.pos_domain,
                    program,
                    &operations,
                    true,
                )?);
            let key =
                match semantic_transform_key(package, paradigm.pos_domain, program, &operations) {
                    Ok(key) => key,
                    Err(error) if error == "transferable program contains exact allomorph" => {
                        collected.exact_allomorph_programs += 1;
                        continue;
                    }
                    Err(error) => return Err(error),
                };
            let program_key = collected
                .program_semantic_keys
                .get_mut(program_index)
                .ok_or("productive semantic program identity exceeds package")?;
            if let Some(existing) = program_key.as_ref() {
                if existing != &key {
                    return Err(
                        "productive program maps to conflicting semantic classes".to_string()
                    );
                }
            } else {
                *program_key = Some(key.clone());
            }
            let mut suffix_drop = 0_u32;
            for operation in &operations {
                if operation.decoded_opcode().map_err(str::to_string)?
                    == MorphOpcodeV1::DropSourceSuffix
                {
                    suffix_drop = suffix_drop
                        .checked_add(operation.arg1)
                        .ok_or("productive semantic suffix drop overflows u32")?;
                }
            }
            let suffix_drop = u16::try_from(suffix_drop)
                .map_err(|_| "productive semantic suffix drop exceeds u16")?;
            collected
                .transforms
                .entry(key)
                .and_modify(|entry| {
                    entry.owners.insert(paradigm_id);
                })
                .or_insert_with(|| SemanticTransformAccumulatorV1 {
                    owners: BTreeSet::from([paradigm_id]),
                    suffix_drop,
                });
        }
    }
    Ok(collected)
}

fn decode_records<T: super::records::FixedRecordV1>(
    package: &ProductivePackageViewV1,
    section: ProductiveSectionKindV1,
) -> Result<Vec<T>, String> {
    (0..package.record_count(section))
        .map(|index| package.record::<T>(section, index))
        .collect()
}

fn program_operations(
    package: &ProductivePackageViewV1,
    program: MorphProgramHeaderRecordV1,
    operation_count: usize,
) -> Result<Vec<MorphOpRecordV1>, String> {
    let start = program.op_start as usize;
    let end = start
        .checked_add(program.op_count as usize)
        .filter(|end| *end <= operation_count)
        .ok_or("productive semantic estimator operation range is invalid")?;
    (start..end)
        .map(|index| {
            package.record::<MorphOpRecordV1>(ProductiveSectionKindV1::MorphOperations, index)
        })
        .collect()
}

fn raw_program_key(
    pos_domain: u16,
    program: MorphProgramHeaderRecordV1,
    operations: &[MorphOpRecordV1],
    ignore_variant: bool,
) -> Result<RawProgramKeyV1, String> {
    let mut output = Vec::with_capacity(operations.len());
    for operation in operations.iter().copied() {
        let opcode = operation.decoded_opcode().map_err(str::to_string)?;
        output.push(RawOperationKeyV1 {
            opcode: operation.opcode,
            anchor: operation.anchor,
            flags: operation.flags,
            arg0: operation.arg0,
            arg1: operation.arg1,
            arg2: if ignore_variant && opcode == MorphOpcodeV1::Terminate {
                0
            } else {
                operation.arg2
            },
        });
    }
    Ok(RawProgramKeyV1 {
        pos_domain,
        source_slot_id: program.source_slot_id,
        target_slot_id: program.target_slot_id,
        flags: program.flags,
        operations: output,
    })
}

fn semantic_transform_key(
    package: &ProductivePackageViewV1,
    pos_domain: u16,
    program: MorphProgramHeaderRecordV1,
    operations: &[MorphOpRecordV1],
) -> Result<SemanticTransformKeyV1, String> {
    let mut pieces = Vec::new();
    let mut terminated = false;
    for operation in operations.iter().copied() {
        if terminated {
            return Err("transferable operation follows terminate".to_string());
        }
        match operation.decoded_opcode().map_err(str::to_string)? {
            MorphOpcodeV1::CopySourceRange => pieces.push(SemanticPieceV1::Copy {
                anchor: operation.anchor,
                start_delta: operation.arg0,
                scalar_count: operation.arg1,
            }),
            MorphOpcodeV1::DropSourcePrefix => {
                pieces.push(SemanticPieceV1::DropPrefix(operation.arg1));
            }
            MorphOpcodeV1::DropSourceSuffix => {
                pieces.push(SemanticPieceV1::DropSuffix(operation.arg1));
            }
            MorphOpcodeV1::EmitSegment => {
                push_literal(&mut pieces, package.segment(operation.arg1)?.as_bytes());
            }
            MorphOpcodeV1::ReplaceSourceRange => {
                pieces.push(SemanticPieceV1::ReplaceConsume {
                    end_relative_offset: operation.arg0,
                    scalar_count: operation.arg1,
                });
                if operation.arg2 != 0 {
                    push_literal(&mut pieces, package.segment(operation.arg2)?.as_bytes());
                }
            }
            MorphOpcodeV1::EmitExactAllomorph => {
                return Err("transferable program contains exact allomorph".to_string());
            }
            MorphOpcodeV1::Terminate => {
                if operation.arg1 != program.target_slot_id || operation.arg2 == 0 {
                    return Err("transferable terminate identity is invalid".to_string());
                }
                terminated = true;
            }
        }
    }
    if !terminated {
        return Err("transferable program has no terminate".to_string());
    }
    Ok(SemanticTransformKeyV1 {
        pos_domain,
        source_slot_id: program.source_slot_id,
        target_slot_id: program.target_slot_id,
        pieces,
    })
}

fn push_literal(pieces: &mut Vec<SemanticPieceV1>, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    if let Some(SemanticPieceV1::Literal(existing)) = pieces.last_mut() {
        existing.extend_from_slice(bytes);
    } else {
        pieces.push(SemanticPieceV1::Literal(bytes.to_vec()));
    }
}

fn semantic_dag_counts<'a>(
    transforms: impl Iterator<Item = &'a SemanticTransformKeyV1>,
) -> (u64, u64, u64, u64) {
    let mut nodes = vec![PrefixNodeV1::default()];
    for transform in transforms {
        let mut current = 0_usize;
        let mut tokens = Vec::with_capacity(transform.pieces.len() + 2);
        tokens.push(SemanticDagTokenV1::Domain {
            pos_domain: transform.pos_domain,
            source_slot_id: transform.source_slot_id,
        });
        tokens.extend(
            transform
                .pieces
                .iter()
                .cloned()
                .map(SemanticDagTokenV1::Piece),
        );
        tokens.push(SemanticDagTokenV1::TargetSlot(transform.target_slot_id));
        for token in tokens {
            let next = if let Some(index) = nodes[current].children.get(&token).copied() {
                index
            } else {
                let index = nodes.len();
                nodes.push(PrefixNodeV1::default());
                nodes[current].children.insert(token, index);
                index
            };
            current = next;
        }
        nodes[current].terminal = true;
    }
    let prefix_arcs = nodes.iter().map(|node| node.children.len() as u64).sum();

    let mut canonical_states = BTreeMap::<CanonicalStateKeyV1, usize>::new();
    let mut state_ids = vec![0_usize; nodes.len()];
    for node_index in (0..nodes.len()).rev() {
        let key = CanonicalStateKeyV1 {
            terminal: nodes[node_index].terminal,
            children: nodes[node_index]
                .children
                .iter()
                .map(|(token, child)| (token.clone(), state_ids[*child]))
                .collect(),
        };
        let next_id = canonical_states.len();
        let state_id = *canonical_states.entry(key).or_insert(next_id);
        state_ids[node_index] = state_id;
    }
    let minimized_arcs = canonical_states
        .keys()
        .map(|state| state.children.len() as u64)
        .sum();
    (
        nodes.len() as u64,
        prefix_arcs,
        canonical_states.len() as u64,
        minimized_arcs,
    )
}

fn semantic_key_payload_bytes(key: &SemanticTransformKeyV1) -> u64 {
    10 + key
        .pieces
        .iter()
        .map(|piece| match piece {
            SemanticPieceV1::Copy { .. } => 13,
            SemanticPieceV1::DropPrefix(_) | SemanticPieceV1::DropSuffix(_) => 5,
            SemanticPieceV1::ReplaceConsume { .. } => 9,
            SemanticPieceV1::Literal(bytes) => 5 + bytes.len() as u64,
        })
        .sum::<u64>()
}

fn cardinality_summary(mut values: Vec<u64>) -> CardinalitySummaryV1 {
    values.sort_unstable();
    CardinalitySummaryV1 {
        minimum: values.first().copied().unwrap_or_default(),
        p50: percentile(&values, 50),
        p95: percentile(&values, 95),
        p99: percentile(&values, 99),
        maximum: values.last().copied().unwrap_or_default(),
    }
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = (values.len() - 1).saturating_mul(percentile).div_ceil(100);
    values[index.min(values.len() - 1)]
}

fn hex_sha256(value: [u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in value {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_literals_share_one_semantic_piece() {
        let mut pieces = Vec::new();
        push_literal(&mut pieces, "при".as_bytes());
        push_literal(&mut pieces, "мер".as_bytes());
        assert_eq!(
            pieces,
            vec![SemanticPieceV1::Literal("пример".as_bytes().to_vec())]
        );
    }

    #[test]
    fn minimized_dag_shares_equal_suffix_states() {
        let first = SemanticTransformKeyV1 {
            pos_domain: 1,
            source_slot_id: 2,
            target_slot_id: 3,
            pieces: vec![
                SemanticPieceV1::DropSuffix(1),
                SemanticPieceV1::Literal(b"a".to_vec()),
            ],
        };
        let second = SemanticTransformKeyV1 {
            pos_domain: 1,
            source_slot_id: 2,
            target_slot_id: 3,
            pieces: vec![
                SemanticPieceV1::DropSuffix(2),
                SemanticPieceV1::Literal(b"a".to_vec()),
            ],
        };
        let (nodes, _, states, _) = semantic_dag_counts([&first, &second].into_iter());
        assert!(states < nodes);
    }

    #[test]
    fn output_length_shape_matches_copy_to_replacement_edge() {
        let pieces = vec![
            SemanticPieceV1::Copy {
                anchor: SourceAnchorV1::Start as u8,
                start_delta: 0,
                scalar_count: u32::from(COPY_TO_RETAINED_EDGE),
            },
            SemanticPieceV1::ReplaceConsume {
                end_relative_offset: -2,
                scalar_count: 1,
            },
            SemanticPieceV1::Literal("яя".as_bytes().to_vec()),
        ];

        let shape = semantic_output_length_shape(&pieces, 1).unwrap();

        assert_eq!(shape.output_len(8), Some(8));
    }

    #[test]
    fn fixed_literal_edges_reject_only_proven_contradictions() {
        let transform = SemanticTransformRecordV1 {
            key: SemanticTransformKeyV1 {
                pos_domain: 1,
                source_slot_id: 2,
                target_slot_id: 3,
                pieces: vec![
                    SemanticPieceV1::Literal("при".as_bytes().to_vec()),
                    SemanticPieceV1::DropPrefix(1),
                    SemanticPieceV1::Literal("ом".as_bytes().to_vec()),
                ],
            },
            owners: vec![1],
            suffix_drop: 0,
            output_length_shape: SemanticLengthShapeV1 {
                source_coefficient: 0,
                constant: 5,
            },
        };

        assert!(semantic_fixed_edges_match(&transform, "приом"));
        assert!(!semantic_fixed_edges_match(&transform, "проом"));
        assert!(!semantic_fixed_edges_match(&transform, "прием"));
    }

    #[test]
    fn owner_intersection_requires_every_exposed_constraint() {
        let identity = SemanticTransformRecordV1 {
            key: SemanticTransformKeyV1 {
                pos_domain: 1,
                source_slot_id: 10,
                target_slot_id: 20,
                pieces: vec![SemanticPieceV1::Copy {
                    anchor: SourceAnchorV1::Start as u8,
                    start_delta: 0,
                    scalar_count: u32::from(COPY_TO_RETAINED_EDGE),
                }],
            },
            owners: vec![1, 2],
            suffix_drop: 0,
            output_length_shape: SemanticLengthShapeV1 {
                source_coefficient: 1,
                constant: 0,
            },
        };
        let inflection = SemanticTransformRecordV1 {
            key: SemanticTransformKeyV1 {
                pos_domain: 1,
                source_slot_id: 10,
                target_slot_id: 30,
                pieces: vec![
                    SemanticPieceV1::Copy {
                        anchor: SourceAnchorV1::Start as u8,
                        start_delta: 0,
                        scalar_count: u32::from(COPY_TO_RETAINED_EDGE),
                    },
                    SemanticPieceV1::DropSuffix(1),
                    SemanticPieceV1::Literal(b"x".to_vec()),
                ],
            },
            owners: vec![2, 3],
            suffix_drop: 1,
            output_length_shape: SemanticLengthShapeV1 {
                source_coefficient: 1,
                constant: 0,
            },
        };
        let index = SemanticExecutionIndexV1 {
            transforms_by_target: BTreeMap::from([
                (
                    (1, 10, 20),
                    BTreeMap::from([(identity.output_length_shape, vec![identity])]),
                ),
                (
                    (1, 10, 30),
                    BTreeMap::from([(inflection.output_length_shape, vec![inflection])]),
                ),
            ]),
            transforms_per_source: BTreeMap::from([((1, 10), 2)]),
            transforms_per_source_target: BTreeMap::from([((1, 10, 20), 1), ((1, 10, 30), 1)]),
            execution_class_by_program: Vec::new().into_boxed_slice(),
            execution_class_count: 0,
            paradigm_count: 3,
            maximum_observed_scalars: 16,
            maximum_generated_scalars: 16,
        };
        let audit = SharedHypothesisReplayAuditV1 {
            pos_domain: 1,
            anchor_slot_id: 10,
            normalized_source: "ab".to_string(),
            constraints: vec![
                super::super::packaged_runtime::SharedReplayConstraintV1 {
                    slot_id: 20,
                    normalized_surface: "ab".to_string(),
                },
                super::super::packaged_runtime::SharedReplayConstraintV1 {
                    slot_id: 30,
                    normalized_surface: "ax".to_string(),
                },
            ],
            eligible_paradigm_ids: vec![1, 2, 3],
            direct_exact_paradigm_ids: vec![2],
        };

        let (owners, metrics) = evaluate_shared_hypothesis(&index, &audit).unwrap();

        assert_eq!(owners, vec![2]);
        assert_eq!(metrics.exact_terminal_matches, 2);
        assert_eq!(metrics.final_owner_candidates, 1);
    }

    #[test]
    fn semantic_executor_rejects_incompatible_source_guard() {
        let transform = SemanticTransformRecordV1 {
            key: SemanticTransformKeyV1 {
                pos_domain: 1,
                source_slot_id: 10,
                target_slot_id: 20,
                pieces: vec![
                    SemanticPieceV1::DropPrefix(4),
                    SemanticPieceV1::Literal(b"x".to_vec()),
                ],
            },
            owners: vec![1],
            suffix_drop: 0,
            output_length_shape: SemanticLengthShapeV1 {
                source_coefficient: 0,
                constant: 1,
            },
        };
        let index = SemanticExecutionIndexV1 {
            transforms_by_target: BTreeMap::new(),
            transforms_per_source: BTreeMap::new(),
            transforms_per_source_target: BTreeMap::new(),
            execution_class_by_program: Vec::new().into_boxed_slice(),
            execution_class_count: 0,
            paradigm_count: 1,
            maximum_observed_scalars: 16,
            maximum_generated_scalars: 16,
        };

        assert!(execute_semantic_transform(&index, &transform, &['a', 'b'])
            .unwrap()
            .is_none());
    }
}
