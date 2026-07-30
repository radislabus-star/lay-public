pub mod candidate_gate;
pub mod cell32;
pub mod context;
pub(crate) mod context_phase;
pub mod eval;
pub mod feedback;
pub mod journal;
mod journal_record;
pub mod l1;
pub mod l2;
mod l2_candidate_phase;
pub(crate) mod l2_field;
pub(crate) mod l2_wave_peak;
pub mod l3;
mod l3_context_metrics;
pub(crate) mod l3_phrase_gate;
pub(crate) mod l4_active_disambiguation;
pub mod l4_goal_state;
pub(crate) mod l4_hidden_state;
pub(crate) mod l4_phase_witness;
pub(crate) mod l4_signed_memory;
pub mod learned;
pub mod lexical_attractor;
mod lexical_grokking;
mod lexical_phase;
pub mod llmwave;
pub mod mode;
mod morphology_phase;
pub mod options;
pub mod packet;
pub mod pattern_wave;
pub(crate) mod phase_field;
pub mod precognition;
pub mod resonance_memory;
mod self_teacher_l3;
pub mod signal;
pub mod structural_relation;
mod surface_bank;
pub(crate) mod surface_damage;
mod surface_wave;
pub mod trace;
pub(crate) mod usage_prior;

pub use eval::{evaluate_wave, evaluate_wave_with_options, WaveEvalResult, WaveEvalStats};
pub use l2_candidate_phase::L2PhaseTrainingEntry;
pub(crate) use l2_candidate_phase::{PhaseReadout, PhaseVerdict};
pub use l2_field::{
    canonical_l2_status, compile_canonical_l2_package, default_l2_model_dir,
    discover_installed_l2_package, export_unseeded_l11_seed_corpus, prove_canonical_l2_package,
    query_canonical_l2_package,
};
pub use lexical_grokking::{
    admit_l11_delta, admit_l11_tombstone, analyze_l1_forward_compression,
    authoritative_restore_surface, benchmark_l1_diverse_restoration, benchmark_l1_lexical_grokking,
    build_lazy_v8_package, build_lazy_v8_package_with_shard_size, compact_depth0_package,
    crystallize_l1_lexical_grokking, crystallize_l1_lexical_grokking_with_rss_budget,
    crystallize_l1_lexical_grokking_with_surface_policy, default_l11_model_dir,
    default_l11_socket_path, discover_installed_l11_package, ensure_l11_service_started,
    export_l1_fixed_latency_surfaces, initialize_l11_composite_manifest, inspect_l1_package_header,
    l11_seed_surfaces, prove_l1_lexical_grokking, prove_l1_lexical_grokking_complete_postings,
    prove_l1_lexical_grokking_composite, prove_l1_lexical_grokking_package,
    prove_l1_lexical_grokking_scale_package, prove_l1_lexical_grokking_scale_package_range,
    query_l1_lexical_grokking, request_l11_authoritative_surface, request_l11_decoded_surfaces,
    request_l11_seed_surfaces, restore_l1_surface, send_l11_service_request,
    send_l11_service_request_with_timeout, InstalledL11Package, L11SeedSurface,
    L11ServiceEnsureReport, L1RestorationHost, L1RestorationHostStats, L1ServiceHealth,
    L1ServiceRequest, L1ServiceResponse, L1ServiceStats, ScaleTrainingSurfacePolicy,
};
pub use mode::{Mode8, ModeRole, CELL32_BYTES, MODES_PER_CELL32};
pub use morphology_phase::{
    run_embedded_russian_morphology_proof, run_russian_morphology_proof_path,
};
pub use options::WaveOptions;
pub use self_teacher_l3::{build_lay_self_teacher_l3_report, LaySelfTeacherL3Config};
pub use signal::{ActiveMode, LayerTrace, WaveDecision, WavePacket, WaveTrace, WordCandidate};
pub use trace::{run_wave_trace, run_wave_trace_with_options};
pub use usage_prior::UsagePriorSnapshot;

pub fn l3_context_report_json(
    cases: &[crate::eval_cases::EvalCase],
    full_cases: usize,
) -> serde_json::Value {
    l3_context_metrics::report_json(cases, full_cases)
}

pub fn l3_context_report_json_with_jobs(
    cases: &[crate::eval_cases::EvalCase],
    full_cases: usize,
    jobs: usize,
) -> serde_json::Value {
    l3_context_metrics::report_json_with_jobs(cases, full_cases, jobs)
}

pub fn compile_l3_context_phase_memory(
    corpus_path: &std::path::Path,
    output_path: &std::path::Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> std::io::Result<serde_json::Value> {
    compile_l3_context_phase_memory_with_progress(
        corpus_path,
        output_path,
        max_fragments,
        min_profile_support,
        0,
        |_| {},
    )
}

/// Cold compiles L3 from clean context plus compact observed surface geometry.
/// The correction JSONL is read only here; the emitted package still contains
/// phase centers and hashes, never correction strings.
pub fn compile_l3_context_phase_memory_with_surface_evidence(
    corpus_path: &std::path::Path,
    surface_evidence_path: &std::path::Path,
    output_path: &std::path::Path,
    max_fragments: usize,
    min_profile_support: u32,
    min_surface_support: u32,
) -> std::io::Result<serde_json::Value> {
    let surface_field = context_phase::surface_field_from_corrections_path(
        surface_evidence_path,
        min_surface_support,
    )?;
    let surface_report = surface_field.report();
    let corpus = std::fs::File::open(corpus_path)?;
    let (package, report) = context_phase::compile_context_phase_reader_with_surface_field(
        corpus,
        max_fragments,
        min_profile_support,
        0,
        std::sync::Arc::new(surface_field),
        |_, _| Ok(()),
    )?;
    context_phase::write_package(output_path, &package)?;
    let mut value = serde_json::to_value(report).map_err(std::io::Error::other)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("corpus".to_string(), serde_json::json!(corpus_path));
        object.insert(
            "surface_evidence".to_string(),
            serde_json::json!(surface_evidence_path),
        );
        object.insert(
            "surface_field".to_string(),
            serde_json::json!({
                "source_rows": surface_report.source_rows,
                "admitted_rows": surface_report.admitted_rows,
                "mode_count": surface_report.mode_count,
                "raw_words_stored": false,
            }),
        );
        object.insert("output".to_string(), serde_json::json!(output_path));
        object.insert(
            "artifact_bytes".to_string(),
            serde_json::json!(std::fs::metadata(output_path)
                .map(|meta| meta.len())
                .unwrap_or_default()),
        );
    }
    Ok(value)
}

pub fn compile_l3_context_delta_for_manifest(
    manifest_path: &std::path::Path,
    corpus_path: &std::path::Path,
    surface_evidence_path: &std::path::Path,
    output_path: &std::path::Path,
    min_profile_support: u32,
    min_surface_support: u32,
    pairwise_only: bool,
) -> std::io::Result<serde_json::Value> {
    let baseline = context_phase::L3CompositeMemory::load_manifest(manifest_path)?;
    let signature_schema = baseline.package().signature_schema;
    let surface_field = context_phase::surface_field_from_corrections_path(
        surface_evidence_path,
        min_surface_support,
    )?;
    let surface_report = surface_field.report();
    let corpus = std::fs::File::open(corpus_path)?;
    let (mut package, report) =
        context_phase::compile_context_phase_delta_reader_with_projection_base(
            corpus,
            min_profile_support,
            signature_schema,
            std::sync::Arc::new(surface_field),
            baseline.package(),
            |_, _| Ok(()),
        )?;
    if pairwise_only {
        package.semantic_states.clear();
        package.profiles.clear();
        package.signature_profiles.clear();
    }
    context_phase::write_package(output_path, &package)?;
    let mut value = serde_json::to_value(report).map_err(std::io::Error::other)?;
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "kind".to_string(),
            serde_json::json!("l3_context_delta_compile"),
        );
        object.insert("manifest".to_string(), serde_json::json!(manifest_path));
        object.insert("corpus".to_string(), serde_json::json!(corpus_path));
        object.insert(
            "surface_evidence".to_string(),
            serde_json::json!(surface_evidence_path),
        );
        object.insert("output".to_string(), serde_json::json!(output_path));
        object.insert(
            "signature_schema".to_string(),
            serde_json::json!(signature_schema),
        );
        object.insert(
            "projection_base_semantic_states".to_string(),
            serde_json::json!(baseline.package().semantic_states.len()),
        );
        object.insert(
            "projection_base_inherited".to_string(),
            serde_json::json!(true),
        );
        object.insert(
            "delta_mode".to_string(),
            serde_json::json!(if pairwise_only {
                "pairwise_only"
            } else {
                "general"
            }),
        );
        object.insert(
            "emitted_semantic_states".to_string(),
            serde_json::json!(package.semantic_states.len()),
        );
        object.insert(
            "emitted_candidate_profiles".to_string(),
            serde_json::json!(package.profiles.len()),
        );
        object.insert(
            "emitted_signature_profiles".to_string(),
            serde_json::json!(package.signature_profiles.len()),
        );
        object.insert(
            "emitted_pair_profiles".to_string(),
            serde_json::json!(package.pair_profiles.len()),
        );
        object.insert(
            "artifact_bytes".to_string(),
            serde_json::json!(std::fs::metadata(output_path)
                .map(|meta| meta.len())
                .unwrap_or_default()),
        );
        object.insert(
            "surface_field".to_string(),
            serde_json::json!({
                "source_rows": surface_report.source_rows,
                "admitted_rows": surface_report.admitted_rows,
                "mode_count": surface_report.mode_count,
                "raw_words_stored": false,
            }),
        );
        object.insert("base_loaded".to_string(), serde_json::json!(true));
        object.insert("base_rewritten".to_string(), serde_json::json!(false));
        object.insert("runtime_authority".to_string(), serde_json::json!(false));
    }
    Ok(value)
}

pub fn compile_l3_context_phase_memory_with_progress<F>(
    corpus_path: &std::path::Path,
    output_path: &std::path::Path,
    max_fragments: usize,
    min_profile_support: u32,
    snapshot_every_fragments: usize,
    mut progress: F,
) -> std::io::Result<serde_json::Value>
where
    F: FnMut(&serde_json::Value),
{
    let corpus = std::fs::File::open(corpus_path)?;
    let (package, report) = context_phase::compile_context_phase_reader(
        corpus,
        max_fragments,
        min_profile_support,
        snapshot_every_fragments,
        |snapshot, report| {
            context_phase::write_package(output_path, snapshot)?;
            let value = serde_json::to_value(report).map_err(std::io::Error::other)?;
            progress(&value);
            Ok(())
        },
    )?;
    context_phase::write_package(output_path, &package)?;
    let mut value = serde_json::to_value(report).map_err(std::io::Error::other)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("corpus".to_string(), serde_json::json!(corpus_path));
        object.insert("output".to_string(), serde_json::json!(output_path));
        object.insert(
            "artifact_bytes".to_string(),
            serde_json::json!(std::fs::metadata(output_path)
                .map(|meta| meta.len())
                .unwrap_or_default()),
        );
    }
    Ok(value)
}

pub fn merge_l3_context_phase_shards(
    inputs: &[std::path::PathBuf],
    output_path: &std::path::Path,
    min_surface_support: u32,
) -> std::io::Result<serde_json::Value> {
    let shards = inputs
        .iter()
        .map(|path| context_phase::read_package(path))
        .collect::<std::io::Result<Vec<_>>>()?;
    if let Some(first) = shards.first() {
        if shards
            .iter()
            .any(|package| package.signature_schema != first.signature_schema)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cannot merge L3 context packages with different scene schemas",
            ));
        }
    }
    let (package, consensus) =
        context_phase::ContextPhasePackage::merge_shards_with_min_surface_support(
            shards,
            min_surface_support,
        );
    context_phase::write_package(output_path, &package)?;
    Ok(
        serde_json::json!({"kind":"l3_context_phase_shard_merge","inputs":inputs.len(),"output":output_path,"profiles":package.profiles.len(),"states":package.semantic_states.len(),"consensus":consensus}),
    )
}

pub fn initialize_l3_context_composite_manifest(
    manifest_path: &std::path::Path,
    base_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    context_phase::initialize_manifest(manifest_path, base_path)?;
    Ok(serde_json::json!({
        "kind": "l3_composite_manifest_initialized",
        "manifest": manifest_path,
        "base": base_path,
        "base_rewritten": false,
        "runtime_authority": false,
    }))
}

pub fn admit_l3_context_delta(
    manifest_path: &std::path::Path,
    delta_path: &std::path::Path,
    proof_receipt: Option<&std::path::Path>,
    scope: Option<&str>,
) -> std::io::Result<serde_json::Value> {
    let Some(proof_receipt) = proof_receipt else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "L3 delta admission requires a targeted proof receipt",
        ));
    };
    validate_l3_targeted_proof_receipt(manifest_path, delta_path, proof_receipt)?;
    context_phase::admit_delta(manifest_path, delta_path, Some(proof_receipt), scope)
}

pub fn admit_l3_context_delta_with_full_proof(
    manifest_path: &std::path::Path,
    delta_path: &std::path::Path,
    targeted_proof_receipt: &std::path::Path,
    full_proof_receipt: &std::path::Path,
    scope: Option<&str>,
) -> std::io::Result<serde_json::Value> {
    validate_l3_targeted_proof_receipt(manifest_path, delta_path, targeted_proof_receipt)?;
    validate_l3_full_proof_receipt(manifest_path, delta_path, full_proof_receipt)?;
    context_phase::admit_delta_with_full_proof(
        manifest_path,
        delta_path,
        Some(targeted_proof_receipt),
        Some(full_proof_receipt),
        scope,
    )
}

fn validate_l3_targeted_proof_receipt(
    manifest_path: &std::path::Path,
    delta_path: &std::path::Path,
    proof_receipt: &std::path::Path,
) -> std::io::Result<()> {
    if !proof_receipt.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "L3 targeted proof receipt does not exist",
        ));
    }
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(proof_receipt)?).map_err(std::io::Error::other)?;
    let receipt_delta = receipt
        .get("delta")
        .and_then(serde_json::Value::as_str)
        .map(std::path::PathBuf::from);
    let receipt_manifest = receipt
        .get("manifest")
        .and_then(serde_json::Value::as_str)
        .map(std::path::PathBuf::from);
    let delta_matches = receipt_delta.and_then(|path| std::fs::canonicalize(path).ok())
        == std::fs::canonicalize(delta_path).ok();
    let manifest_matches = receipt_manifest.and_then(|path| std::fs::canonicalize(path).ok())
        == std::fs::canonicalize(manifest_path).ok();
    let valid_receipt = receipt.get("kind").and_then(serde_json::Value::as_str)
        == Some("l3_context_delta_targeted_proof")
        && receipt.get("verdict").and_then(serde_json::Value::as_str) == Some("PASS")
        && receipt
            .get("false_supports")
            .and_then(serde_json::Value::as_u64)
            == Some(0)
        && manifest_matches
        && delta_matches;
    if !valid_receipt {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "L3 delta admission requires a matching PASS targeted proof receipt",
        ));
    }
    Ok(())
}

fn validate_l3_full_proof_receipt(
    manifest_path: &std::path::Path,
    delta_path: &std::path::Path,
    proof_receipt: &std::path::Path,
) -> std::io::Result<()> {
    if !proof_receipt.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "L3 full differential proof receipt does not exist",
        ));
    }
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(proof_receipt)?).map_err(std::io::Error::other)?;
    let receipt_delta = receipt
        .get("delta")
        .and_then(serde_json::Value::as_str)
        .map(std::path::PathBuf::from);
    let receipt_manifest = receipt
        .get("manifest")
        .and_then(serde_json::Value::as_str)
        .map(std::path::PathBuf::from);
    let delta_matches = receipt_delta.and_then(|path| std::fs::canonicalize(path).ok())
        == std::fs::canonicalize(delta_path).ok();
    let manifest_matches = receipt_manifest.and_then(|path| std::fs::canonicalize(path).ok())
        == std::fs::canonicalize(manifest_path).ok();
    let byte_count_matches = receipt
        .get("delta_bytes")
        .and_then(serde_json::Value::as_u64)
        == std::fs::metadata(delta_path)
            .ok()
            .map(|metadata| metadata.len());
    let zero_regressions = [
        "lost_target_profiles",
        "lost_supports",
        "lost_top1",
        "new_false_supports",
        "new_false_top1",
    ]
    .into_iter()
    .all(|field| receipt.get(field).and_then(serde_json::Value::as_u64) == Some(0));
    let valid_receipt = receipt.get("kind").and_then(serde_json::Value::as_str)
        == Some("l3_context_phase_full_differential_proof")
        && receipt.get("verdict").and_then(serde_json::Value::as_str) == Some("PASS")
        && zero_regressions
        && manifest_matches
        && delta_matches
        && byte_count_matches;
    if !valid_receipt {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "L3 delta admission requires a matching zero-regression full differential receipt",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod l3_delta_admission_tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_root() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "lay-l3-delta-admission-{}-{stamp}",
            std::process::id()
        ))
    }

    #[test]
    fn full_receipt_requires_matching_delta_and_zero_regressions() {
        let root = unique_root();
        fs::create_dir_all(&root).unwrap();
        let delta = root.join("delta.nwpc");
        let manifest = root.join("manifest.json");
        let receipt = root.join("full.json");
        fs::write(&delta, b"delta-bytes").unwrap();
        fs::write(&manifest, b"{}").unwrap();
        let write_receipt = |lost_supports: u64| {
            fs::write(
                &receipt,
                serde_json::to_vec(&serde_json::json!({
                    "kind": "l3_context_phase_full_differential_proof",
                    "verdict": "PASS",
                    "manifest": manifest,
                    "delta": delta,
                    "delta_bytes": 11,
                    "lost_target_profiles": 0,
                    "lost_supports": lost_supports,
                    "lost_top1": 0,
                    "new_false_supports": 0,
                    "new_false_top1": 0,
                }))
                .unwrap(),
            )
            .unwrap();
        };

        write_receipt(0);
        validate_l3_full_proof_receipt(&manifest, &delta, &receipt).unwrap();
        let other_manifest = root.join("other-manifest.json");
        fs::write(&other_manifest, b"{}").unwrap();
        assert!(
            validate_l3_full_proof_receipt(&other_manifest, &delta, &receipt).is_err(),
            "a receipt from another composite manifest must not open admission"
        );
        write_receipt(1);
        assert!(validate_l3_full_proof_receipt(&manifest, &delta, &receipt).is_err());
        let _ = fs::remove_dir_all(root);
    }
}

pub fn compact_l3_context_composite(
    manifest_path: &std::path::Path,
    output_base: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    context_phase::compact_manifest(manifest_path, output_base)
}

pub fn snapshot_l3_context_composite(
    manifest_path: &std::path::Path,
    output_base: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    context_phase::snapshot_manifest(manifest_path, output_base)
}

pub fn reload_l3_context_composite() -> std::io::Result<serde_json::Value> {
    context_phase::reload_default_memory()
}

/// Proves one small delta against explicit changed scenes and fixed safety
/// sentinels. TSV rows are: `improve|safety<TAB>context<TAB>a|b<TAB>expected|-`.
pub fn prove_l3_context_delta_targeted(
    manifest_path: &std::path::Path,
    delta_path: &std::path::Path,
    cases_path: &std::path::Path,
    receipt_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    let baseline = context_phase::L3CompositeMemory::load_manifest(manifest_path)?;
    let base = baseline.package().clone();
    let delta = context_phase::read_package(delta_path)?;
    if base.signature_schema != delta.signature_schema {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "L3 delta signature schema does not match the immutable base",
        ));
    }
    let candidate = baseline.compose_delta_path(delta_path)?;
    let text = std::fs::read_to_string(cases_path)?;
    let mut improve_cases = 0_u64;
    let mut improved = 0_u64;
    let mut target_failures = 0_u64;
    let mut safety_cases = 0_u64;
    let mut false_supports = 0_u64;
    let mut failures = Vec::new();
    for (index, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid targeted L3 case at line {}", index + 1),
            ));
        }
        let candidates = fields[2]
            .split('|')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if candidates.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "targeted L3 case needs at least two candidates at line {}",
                    index + 1
                ),
            ));
        }
        let context = context_phase::tokenize_context_text(fields[1]);
        let base_readouts = base.score_candidates(&context, &candidates);
        let candidate_readouts = candidate.score_candidates(&context, &candidates);
        match fields[0] {
            "improve" => {
                improve_cases += 1;
                let expected = fields[3];
                let Some(expected_index) = candidates
                    .iter()
                    .position(|candidate| *candidate == expected)
                else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("expected candidate is absent at line {}", index + 1),
                    ));
                };
                let before = base_readouts[expected_index];
                let after = candidate_readouts[expected_index];
                if after.disposition == context_phase::ContextPhaseDisposition::Support
                    && (before.disposition != context_phase::ContextPhaseDisposition::Support
                        || after.margin_micro > before.margin_micro)
                {
                    improved += 1;
                } else {
                    target_failures += 1;
                    failures.push(serde_json::json!({
                        "line": index + 1,
                        "kind": "target_not_improved",
                        "expected": expected,
                        "before": format!("{:?}", before.disposition),
                        "after": format!("{:?}", after.disposition),
                        "before_margin_micro": before.margin_micro,
                        "after_margin_micro": after.margin_micro,
                        "after_threshold_micro": after.threshold_micro,
                        "after_competition_margin_micro": after.competition_margin_micro,
                        "after_positive_examples": after.positive_examples,
                        "after_positive_center_support": after.positive_center_support,
                        "after_pairwise_certified": after.pairwise_certified,
                        "after_pairwise_blocked": after.pairwise_blocked,
                        "after_pairwise_conflict": after.pairwise_conflict,
                        "after_pairwise_known_edges": after.pairwise_known_edges,
                        "after_pairwise_unknown_edges": after.pairwise_unknown_edges,
                        "candidate_readouts": candidates.iter().zip(&candidate_readouts).map(|(surface, readout)| serde_json::json!({
                            "surface": surface,
                            "disposition": format!("{:?}", readout.disposition),
                            "margin_micro": readout.margin_micro,
                            "threshold_micro": readout.threshold_micro,
                            "competition_margin_micro": readout.competition_margin_micro,
                            "pairwise_blocked": readout.pairwise_blocked,
                            "pairwise_certified": readout.pairwise_certified,
                        })).collect::<Vec<_>>(),
                        "pair_debug": candidate.pair_debug(&context, &candidates),
                    }));
                }
            }
            "safety" => {
                safety_cases += 1;
                let allowed = (fields[3] != "-").then_some(fields[3]);
                if allowed.is_some_and(|expected| !candidates.contains(&expected)) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("allowed safety candidate is absent at line {}", index + 1),
                    ));
                }
                for (surface, readout) in candidates.iter().zip(&candidate_readouts) {
                    if Some(*surface) != allowed
                        && readout.disposition == context_phase::ContextPhaseDisposition::Support
                    {
                        false_supports += 1;
                        failures.push(serde_json::json!({
                            "line": index + 1,
                            "kind": "false_support",
                            "candidate": surface,
                            "margin_micro": readout.margin_micro,
                        }));
                    }
                }
            }
            kind => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "unknown targeted L3 case kind {kind:?} at line {}",
                        index + 1
                    ),
                ));
            }
        }
    }
    let passed = improve_cases > 0
        && safety_cases > 0
        && improved == improve_cases
        && target_failures == 0
        && false_supports == 0;
    let report = serde_json::json!({
        "kind": "l3_context_delta_targeted_proof",
        "manifest": manifest_path,
        "delta": delta_path,
        "cases": cases_path,
        "improve_cases": improve_cases,
        "improved": improved,
        "target_failures": target_failures,
        "safety_cases": safety_cases,
        "false_supports": false_supports,
        "failures": failures,
        "base_rewritten": false,
        "full_corpus_recompiled": false,
        "runtime_authority": false,
        "verdict": if passed { "PASS" } else { "WATCH" },
    });
    let mut bytes = serde_json::to_vec_pretty(&report).map_err(std::io::Error::other)?;
    bytes.push(b'\n');
    crate::private_file::write_private_bytes(receipt_path, &bytes)?;
    Ok(report)
}

/// Runs the heldout and ablation proof for an existing package without
/// rebuilding it from the evaluation corpus.
pub fn prove_l3_context_phase_package(
    corpus_path: &std::path::Path,
    package_path: &std::path::Path,
    max_fragments: usize,
    min_profile_support: u32,
    surface_evidence_path: Option<&std::path::Path>,
    min_surface_support: u32,
) -> std::io::Result<serde_json::Value> {
    let (report, surface_report) = if let Some(path) = surface_evidence_path {
        let surface_field =
            context_phase::surface_field_from_corrections_path(path, min_surface_support)?;
        let surface_report = surface_field.report();
        (
            context_phase::prove_context_phase_package_path_with_surface_field(
                corpus_path,
                package_path,
                max_fragments,
                min_profile_support,
                &surface_field,
            )?,
            Some(surface_report),
        )
    } else {
        (
            context_phase::prove_context_phase_package_path(
                corpus_path,
                package_path,
                max_fragments,
                min_profile_support,
            )?,
            None,
        )
    };
    let mut value = serde_json::to_value(report).map_err(std::io::Error::other)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("corpus".to_string(), serde_json::json!(corpus_path));
        object.insert("package".to_string(), serde_json::json!(package_path));
        object.insert(
            "surface_evidence".to_string(),
            serde_json::json!(surface_evidence_path),
        );
        object.insert(
            "surface_field".to_string(),
            surface_report.map_or(serde_json::Value::Null, |report| {
                serde_json::json!({
                    "source_rows": report.source_rows,
                    "admitted_rows": report.admitted_rows,
                    "mode_count": report.mode_count,
                    "raw_words_stored": false,
                })
            }),
        );
        object.insert(
            "read_as".to_string(),
            serde_json::json!("frozen package against separate heldout surface; no training"),
        );
    }
    Ok(value)
}

pub fn prove_l3_context_phase_delta_full(
    corpus_path: &std::path::Path,
    baseline_path: &std::path::Path,
    candidate_path: &std::path::Path,
    surface_evidence_path: &std::path::Path,
    max_fragments: usize,
    min_surface_support: u32,
    receipt_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    let surface_field = context_phase::surface_field_from_corrections_path(
        surface_evidence_path,
        min_surface_support,
    )?;
    let surface_report = surface_field.report();
    let report = context_phase::prove_context_phase_package_delta_path(
        corpus_path,
        baseline_path,
        candidate_path,
        max_fragments,
        &surface_field,
    )?;
    let mut value = serde_json::to_value(report).map_err(std::io::Error::other)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("corpus".to_string(), serde_json::json!(corpus_path));
        object.insert("baseline".to_string(), serde_json::json!(baseline_path));
        object.insert("candidate".to_string(), serde_json::json!(candidate_path));
        object.insert(
            "surface_evidence".to_string(),
            serde_json::json!(surface_evidence_path),
        );
        object.insert(
            "surface_field".to_string(),
            serde_json::json!({
                "source_rows": surface_report.source_rows,
                "admitted_rows": surface_report.admitted_rows,
                "mode_count": surface_report.mode_count,
                "raw_words_stored": false,
            }),
        );
    }
    let mut bytes = serde_json::to_vec_pretty(&value).map_err(std::io::Error::other)?;
    bytes.push(b'\n');
    crate::private_file::write_private_bytes(receipt_path, &bytes)?;
    Ok(value)
}

/// Rebuilds the private live L3 packet from the canonical package plus
/// explicit IME accepts/rejections. The packet remains a compact hashed phase
/// memory; the human text log remains only the training source.
pub fn compile_l3_context_feedback_overlay_memory(
    base_path: &std::path::Path,
    usage_events_path: &std::path::Path,
    output_path: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    let mut package = context_phase::read_package(base_path)?;
    let events = std::fs::read_to_string(usage_events_path)?;
    let report = context_phase::apply_feedback_overlay(&mut package, &events)?;
    context_phase::write_package(output_path, &package)?;
    let mut value = serde_json::to_value(report).map_err(std::io::Error::other)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("base".to_string(), serde_json::json!(base_path));
        object.insert(
            "usage_events".to_string(),
            serde_json::json!(usage_events_path),
        );
        object.insert("output".to_string(), serde_json::json!(output_path));
        object.insert(
            "artifact_bytes".to_string(),
            serde_json::json!(std::fs::metadata(output_path)
                .map(|meta| meta.len())
                .unwrap_or_default()),
        );
    }
    Ok(value)
}

/// Extracts a private clean-text L3 corpus from explicit accepted IME outcomes.
/// It does not build or install a package; callers can inspect the corpus
/// receipt, merge it with a clean external corpus, and run cold training.
pub fn build_l3_context_feedback_corpus(
    usage_events_path: &std::path::Path,
    output_path: &std::path::Path,
    max_repeat_per_phrase: usize,
) -> std::io::Result<serde_json::Value> {
    let events = std::fs::read_to_string(usage_events_path)?;
    let (corpus, report) = context_phase::build_feedback_corpus(&events, max_repeat_per_phrase)?;
    std::fs::write(output_path, corpus)?;
    let mut value = serde_json::to_value(report).map_err(std::io::Error::other)?;
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "usage_events".to_string(),
            serde_json::json!(usage_events_path),
        );
        object.insert("output".to_string(), serde_json::json!(output_path));
        object.insert(
            "artifact_bytes".to_string(),
            serde_json::json!(std::fs::metadata(output_path)
                .map(|meta| meta.len())
                .unwrap_or_default()),
        );
    }
    Ok(value)
}

/// Extracts confirmed IME outcomes into a bounded lexical training source for
/// the next cold L2 phase compile. This writes a corpus only: hot runtime
/// still loads compact centers from a separately proved artifact.
pub fn build_l2_lexical_feedback_corpus(
    usage_events_path: &std::path::Path,
    output_path: &std::path::Path,
    max_repeat_per_phrase: usize,
    max_repeat_per_word: usize,
) -> std::io::Result<serde_json::Value> {
    let events = std::fs::read_to_string(usage_events_path)?;
    let (phrases, phrase_report) =
        context_phase::build_feedback_corpus(&events, max_repeat_per_phrase)?;
    let max_repeat_per_word = max_repeat_per_word.max(1);
    let mut word_counts = std::collections::BTreeMap::<String, usize>::new();
    for raw in phrases.split_whitespace() {
        let Some(word) = normalize_l2_surface_word(raw) else {
            continue;
        };
        let count = word_counts.entry(word).or_default();
        *count = count.saturating_add(1);
    }

    let mut lines = Vec::new();
    for (word, count) in &word_counts {
        for _ in 0..(*count).min(max_repeat_per_word) {
            lines.push(word.as_str());
        }
    }
    let mut corpus = lines.join("\n");
    if !corpus.is_empty() {
        corpus.push('\n');
    }
    std::fs::write(output_path, corpus)?;

    Ok(serde_json::json!({
        "kind": "l2_lexical_phase_feedback_corpus",
        "usage_events": usage_events_path,
        "output": output_path,
        "source_events": phrase_report.source_events,
        "accepted_source_events": phrase_report.accepted_source_events,
        "rejected_source_events": phrase_report.rejected_source_events,
        "accepted_phrases": phrase_report.corpus_lines,
        "unique_words": word_counts.len(),
        "emitted_word_rows": lines.len(),
        "max_repeat_per_word": max_repeat_per_word,
        "runtime_authority": false,
        "raw_words_stored_in_runtime": false,
    }))
}

pub fn l3_context_phase_status_json(path: Option<&std::path::Path>) -> serde_json::Value {
    if let Some(path) = path {
        return context_phase::package_report(path);
    }
    let manifest = context_phase::default_manifest_path();
    if manifest.is_file() {
        return context_phase::L3CompositeMemory::load_manifest(&manifest)
            .map(|memory| memory.report())
            .unwrap_or_else(|error| {
                serde_json::json!({
                    "kind": "l3_composite_memory",
                    "manifest": manifest,
                    "loaded": false,
                    "error": error.to_string(),
                })
            });
    }
    context_phase::package_report(&context_phase::default_memory_path())
}

pub fn prove_l3_context_phase_memory(
    corpus_path: &std::path::Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> std::io::Result<serde_json::Value> {
    serde_json::to_value(context_phase::prove_context_phase_path(
        corpus_path,
        max_fragments,
        min_profile_support,
    )?)
    .map_err(std::io::Error::other)
}

/// Builds the candidate field from the fixed support partition and publishes
/// it only when the untouched heldout partition proves the phase and anti-wave
/// contribution. A WATCH result never replaces the runtime package.
pub fn build_and_prove_l3_context_phase_memory(
    corpus_path: &std::path::Path,
    output_path: &std::path::Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> std::io::Result<serde_json::Value> {
    let (package, heldout) = context_phase::build_and_prove_context_phase_path(
        corpus_path,
        max_fragments,
        min_profile_support,
    )?;
    let package_published = heldout.verdict == "PASS";
    if package_published {
        context_phase::write_package(output_path, &package)?;
    }
    let positive_centers = package
        .profiles
        .iter()
        .map(|profile| profile.positive.len())
        .sum::<usize>();
    let anti_centers = package
        .profiles
        .iter()
        .map(|profile| profile.negative.len() + profile.hard_negative.len())
        .sum::<usize>();
    let artifact_bytes = package_published
        .then(|| std::fs::metadata(output_path).map(|meta| meta.len()))
        .transpose()?
        .unwrap_or_default();
    Ok(serde_json::json!({
        "kind": "l3_context_phase_build_and_prove",
        "architecture": "online_relation_phase_v4_role_scene_lattice",
        "signature_schema": package.signature_schema,
        "corpus": corpus_path,
        "output": output_path,
        "package_published": package_published,
        "raw_words_stored": false,
        "artifact_bytes": artifact_bytes,
        "corpus_fragments": package.corpus_fragments,
        "transitions": package.transitions,
        "semantic_states": package.semantic_states.len(),
        "candidate_profiles": package.profiles.len(),
        "pair_profiles": package.pair_profiles.len(),
        "exact_pair_profiles": package.pair_profile_counts().0,
        "generalized_pair_profiles": package.pair_profile_counts().1,
        "pair_centers": package
            .pair_profiles
            .iter()
            .map(|profile| {
                profile.low_wins.len()
                    + profile.high_wins.len()
                    + profile.hard_low_wins.len()
                    + profile.hard_high_wins.len()
            })
            .sum::<usize>(),
        "positive_centers": positive_centers,
        "anti_centers": anti_centers,
        "global_threshold_micro": package.global_threshold_micro,
        "competition_threshold_micro": package.competition_threshold_micro,
        "min_profile_support": min_profile_support.max(2),
        "heldout": heldout,
    }))
}

/// Builds and proves L3 against the same learned surface field used during
/// training. A failed proof does not write the runtime package.
pub fn build_and_prove_l3_context_phase_memory_with_surface_evidence(
    corpus_path: &std::path::Path,
    surface_evidence_path: &std::path::Path,
    output_path: &std::path::Path,
    max_fragments: usize,
    min_profile_support: u32,
    min_surface_support: u32,
) -> std::io::Result<serde_json::Value> {
    let surface_field = context_phase::surface_field_from_corrections_path(
        surface_evidence_path,
        min_surface_support,
    )?;
    let surface_report = surface_field.report();
    let (package, heldout) = context_phase::build_and_prove_context_phase_path_with_surface_field(
        corpus_path,
        max_fragments,
        min_profile_support,
        std::sync::Arc::new(surface_field),
    )?;
    let package_published = heldout.verdict == "PASS";
    if package_published {
        context_phase::write_package(output_path, &package)?;
    }
    Ok(serde_json::json!({
        "kind": "l3_context_phase_build_and_prove",
        "architecture": "online_relation_phase_v4_role_scene_lattice_learned_surface_field",
        "signature_schema": package.signature_schema,
        "corpus": corpus_path,
        "surface_evidence": surface_evidence_path,
        "surface_field": {
            "source_rows": surface_report.source_rows,
            "admitted_rows": surface_report.admitted_rows,
            "mode_count": surface_report.mode_count,
            "raw_words_stored": false,
        },
        "output": output_path,
        "package_published": package_published,
        "raw_words_stored": false,
        "artifact_bytes": package_published.then(|| std::fs::metadata(output_path).map(|meta| meta.len())).transpose()?.unwrap_or_default(),
        "corpus_fragments": package.corpus_fragments,
        "transitions": package.transitions,
        "candidate_profiles": package.profiles.len(),
        "pair_profiles": package.pair_profiles.len(),
        "heldout": heldout,
    }))
}

static L2_IME_WARMUP_STARTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(test)]
const SEMANTIC_WORD_SOURCE: &str = "SemanticWordCell32";
const PHRASE_FORECAST_CELL: &str = "PhraseForecastCell32";

pub fn word_usage_prior(word: &str) -> f32 {
    usage_prior::word_usage_prior(word)
}

pub fn context_word_usage_prior(context: &[String], word: &str) -> f32 {
    usage_prior::context_word_usage_prior(context, word)
}

pub fn cached_word_usage_prior(word: &str) -> f32 {
    usage_prior::word_usage_prior_cached(word)
}

pub fn cached_context_word_usage_prior(context: &[String], word: &str) -> f32 {
    usage_prior::context_word_usage_prior_cached(context, word)
}

pub fn cached_usage_prior_snapshot() -> UsagePriorSnapshot {
    usage_prior::cached_usage_prior_snapshot()
}

pub fn compile_usage_feedback_snapshot(
    input: &std::path::Path,
    output: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    usage_prior::compile_usage_feedback_snapshot(input, output)
}

pub fn l2_surface_words_by_usage(limit: usize) -> Vec<String> {
    usage_prior::l2_surface_words_by_usage(limit)
}

pub fn default_l2_candidate_phase_memory_path() -> std::path::PathBuf {
    l2_candidate_phase::default_phase_memory_path()
}

pub fn write_l2_candidate_phase_memory<I>(
    path: &std::path::Path,
    entries: I,
) -> std::io::Result<usize>
where
    I: IntoIterator<Item = (String, String, String, usize)>,
{
    l2_candidate_phase::write_phase_memory_from_entries(path, entries)
}

pub fn write_l2_candidate_phase_memory_labeled<I>(
    path: &std::path::Path,
    entries: I,
) -> std::io::Result<usize>
where
    I: IntoIterator<Item = L2PhaseTrainingEntry>,
{
    l2_candidate_phase::write_phase_memory_from_labeled_entries(path, entries)
}

pub fn infer_l2_transition_operator(
    original: &str,
    candidate: &str,
    operation: &str,
) -> &'static str {
    crate::transition_relation::TransitionOperatorKind::infer(original, candidate, operation)
        .as_str()
}

pub fn transition_surface_key(
    original: &str,
    candidate: &str,
    source: &str,
    operation: &str,
) -> String {
    crate::typing_memory::transition_surface_key(original, candidate, source, operation)
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct L2TransitionPhaseShadowReadout {
    pub package_loaded: bool,
    pub operator_present: bool,
    pub operator_promoted: bool,
    pub verdict: &'static str,
    pub positive_micro: i64,
    pub anti_micro: i64,
    pub margin_micro: i64,
    pub threshold_micro: i64,
    pub positive_examples: u32,
    pub negative_examples: u32,
    pub positive_centers: u8,
    pub anti_centers: u8,
    pub covered_surfaces: u32,
    pub rejected_surfaces: u32,
    pub(crate) lexical_positive_micro: i64,
    pub(crate) lexical_anti_micro: i64,
    pub(crate) lexical_margin_micro: i64,
    pub(crate) lexical_threshold_micro: i64,
    pub(crate) lexical_positive_examples: u32,
    pub(crate) lexical_negative_examples: u32,
    pub(crate) lexical_positive_centers: u8,
    pub(crate) lexical_anti_centers: u8,
    pub(crate) lexical_competition_ready: bool,
    pub(crate) lexical_verdict: &'static str,
}

#[derive(Clone, Debug)]
pub struct L2TransitionPhaseShadowEvaluator {
    inner: l2_candidate_phase::PhaseEvaluator,
}

impl L2TransitionPhaseShadowEvaluator {
    pub fn load(path: Option<&std::path::Path>) -> Self {
        Self {
            inner: l2_candidate_phase::PhaseEvaluator::load(path),
        }
    }

    pub fn readout(
        &self,
        original: &str,
        candidate: &str,
        operation: &str,
    ) -> L2TransitionPhaseShadowReadout {
        phase_shadow_readout(self.inner.readout(original, candidate, operation))
    }
}

pub fn l2_transition_phase_shadow_readout(
    original: &str,
    candidate: &str,
    operation: &str,
    path: Option<&std::path::Path>,
) -> L2TransitionPhaseShadowReadout {
    let readout = match path {
        Some(path) => {
            l2_candidate_phase::shadow_readout_from_path(original, candidate, operation, path)
        }
        None => l2_candidate_phase::shadow_readout(original, candidate, operation),
    };
    phase_shadow_readout(readout)
}

fn phase_shadow_readout(
    readout: l2_candidate_phase::PhaseReadout,
) -> L2TransitionPhaseShadowReadout {
    L2TransitionPhaseShadowReadout {
        package_loaded: readout.package_loaded,
        operator_present: readout.operator_present,
        operator_promoted: readout.operator_promoted,
        verdict: readout.verdict.as_str(),
        positive_micro: readout.positive_micro,
        anti_micro: readout.anti_micro,
        margin_micro: readout.margin_micro,
        threshold_micro: readout.threshold_micro,
        positive_examples: readout.positive_examples,
        negative_examples: readout.negative_examples,
        positive_centers: readout.positive_centers,
        anti_centers: readout.anti_centers,
        covered_surfaces: readout.covered_surfaces,
        rejected_surfaces: readout.rejected_surfaces,
        lexical_positive_micro: readout.lexical_positive_micro,
        lexical_anti_micro: readout.lexical_anti_micro,
        lexical_margin_micro: readout.lexical_margin_micro,
        lexical_threshold_micro: readout.lexical_threshold_micro,
        lexical_positive_examples: readout.lexical_positive_examples,
        lexical_negative_examples: readout.lexical_negative_examples,
        lexical_positive_centers: readout.lexical_positive_centers,
        lexical_anti_centers: readout.lexical_anti_centers,
        lexical_competition_ready: readout.lexical_competition_ready,
        lexical_verdict: readout.lexical_verdict.as_str(),
    }
}

pub fn l2_candidate_phase_shadow(
    original: &str,
    candidate: &str,
    operation: &str,
) -> (bool, i64, bool) {
    let shadow = l2_transition_phase_shadow_readout(original, candidate, operation, None);
    (
        shadow.package_loaded,
        shadow.margin_micro,
        shadow.verdict == "support",
    )
}

pub fn l2_transition_phase_report_json(path: Option<&std::path::Path>) -> serde_json::Value {
    let owned;
    let path = match path {
        Some(path) => path,
        None => {
            owned = l2_candidate_phase::default_phase_memory_path();
            &owned
        }
    };
    l2_candidate_phase::phase_memory_report_json(path)
}

pub fn l2_transition_phase_proof_json(entries: &[L2PhaseTrainingEntry]) -> serde_json::Value {
    l2_candidate_phase::phase_proof_json(entries)
}

pub(crate) fn l2_transition_phase_readout(
    action_operator: &str,
    atoms: &[String],
    original: &str,
    candidate: &str,
) -> l2_candidate_phase::PhaseReadout {
    l2_candidate_phase::relation_readout(action_operator, atoms, original, candidate)
}

pub fn usage_debug_summary() -> (u64, usize, usize) {
    usage_prior::usage_debug_summary()
}

pub fn usage_memory_learned_report_json() -> serde_json::Value {
    usage_prior::usage_memory_learned_report_json()
}

pub fn balanced_l2_surface_words<I>(source: I, limit: usize) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    surface_bank::balanced_l2_surface_words(source, limit)
}

pub fn normalize_l2_surface_word(word: &str) -> Option<String> {
    surface_bank::normalize_surface_bank_word(word)
}

pub fn record_typed_tail_usage(tail: &str) {
    usage_prior::record_typed_tail_if_enabled(tail);
}

pub fn record_accepted_fix_usage(from: &str, to: &str) {
    usage_prior::record_accepted_fix_if_enabled(from, to);
}

pub fn record_accepted_layout_projection_usage(from: &str, to: &str) {
    usage_prior::record_accepted_layout_projection_if_enabled(from, to);
}

pub fn record_accepted_ime_usage(context_tail: &str, accepted_text: &str) {
    usage_prior::record_accepted_ime_if_enabled(context_tail, accepted_text);
}

pub fn record_confirmed_ime_prediction_usage(context_tail: &str, predicted_text: &str) {
    usage_prior::record_confirmed_ime_prediction_if_enabled(context_tail, predicted_text);
}

pub fn record_rejected_ime_usage(context_tail: &str, rejected_text: &str) {
    usage_prior::record_rejected_ime_if_enabled(context_tail, rejected_text);
}

pub fn record_rejected_candidate_usage(
    context_tail: &str,
    rejected_text: &str,
    source: &str,
    operation: &str,
) {
    usage_prior::record_rejected_candidate_if_enabled(
        context_tail,
        rejected_text,
        source,
        operation,
    );
}

pub fn warm_up() {
    l2::warm_up_surface_motif_memory();
    context_phase::warm_default_memory();
    let _ = llmwave::load_default_memory();
}

pub fn warm_up_for_ime() {
    l2::warm_up_surface_motif_memory();
    context_phase::warm_default_memory();
    let _ = llmwave::load_default_memory();
}

pub fn warm_up_l2_for_ime() {
    L2_IME_WARMUP_STARTED.store(true, std::sync::atomic::Ordering::Relaxed);
    // The IME gate evaluates L2 and L3 together. Do not publish an L2-ready
    // state while the first context-phase readout could still fault in the
    // phase package on the user's keystroke.
    context_phase::warm_default_memory();
    l2::warm_up_ime_word_candidate_memory();
    candidate_gate::warm_up_live_candidate_readout();
}

pub fn warm_up_l3_phrase_memory() {
    context_phase::warm_default_memory();
    let _ = llmwave::load_default_memory();
}

pub fn ensure_l2_ime_warmup_started() {
    if L2_IME_WARMUP_STARTED
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
        )
        .is_ok()
    {
        std::thread::spawn(warm_up_l2_for_ime);
    }
}
