use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::anchor_recovery_package::{
    compile_anchor_recovery_package, recovery_sidecar_path, CompiledAnchorRecoveryPackageV1,
};
#[cfg(test)]
use super::anchor_recovery_package::{
    AnchorRecoveryPostingRecordV1, PreparedAnchorRecoveryProgramV1,
};
use super::calibrate::CalibrationTableV1;
use super::format::{
    encode_package, ProductiveAlgorithmModeV1, ProductivePackageBuildV1, ProductivePackageViewV1,
    ProductiveSectionBuildV1, ProductiveSectionKindV1, REQUIRED_SECTION_COUNT,
};
use super::induce::{EditOperationV1, EditTemplateV1, SourceAnchorV1};
use super::phase::FittedPhaseBankV1;
use super::records::{
    CalibrationCellRecordV1, DeltaManifestRecordV1, DirectionalResidualRecordV1,
    EvidencePriorRecordV1, ModelCoefficientRecordV1, MorphOpRecordV1, MorphOpcodeV1,
    MorphProgramHeaderRecordV1, ParadigmCenterRecordV1, ParadigmCompatibilityIndexRecordV1,
    ParadigmPostingRecordV1, PhaseCenterRecordV1, ProductiveTerminalRecordV1,
    ProductiveTrieArcOpcodeV1, ProductiveTrieArcRecordV1, ProductiveTrieNodeRecordV1,
    ProvenanceRecordV1, SlotPhaseProfileRecordV1, PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE,
};
use super::reduce::ReducedMorphologyManifestV1;
use super::score::{
    productive_feature_schema_hash_low, FittedEvidenceModelV1, PRODUCTIVE_FEATURE_COUNT,
};
use super::transition_reduce::{
    signature_bytes, ClassifiedTransitionReaderV1, LemmaParadigmAssignmentReaderV1,
    MorphologyAxisSchemaV1, ParadigmCompatibilityIndexReaderV1, ParadigmDefinitionReaderV1,
    ParadigmDefinitionV1, ParadigmPostingReaderV1, TransitionInductionManifestV1,
};
use super::trie::{
    compile_productive_trie, ProductiveTerminalAttributionV1, ProductiveTrieArcActionV1,
    TrieProgramInputV1,
};
use super::types::LemmaParadigmBindingV1;

#[derive(Clone, Debug)]
pub(super) struct TrainedSlotPhaseProfileV1 {
    pub(super) paradigm_id: u32,
    pub(super) slot_id: u32,
    pub(super) support: u32,
    pub(super) explicit_anti_support: u32,
    pub(super) positive: FittedPhaseBankV1,
    pub(super) anti: FittedPhaseBankV1,
    pub(super) hard_negative: FittedPhaseBankV1,
    pub(super) ambiguity: FittedPhaseBankV1,
}

#[derive(Clone, Debug)]
pub(super) struct ProductiveCompilerEvidenceV1 {
    pub(super) evidence_model: FittedEvidenceModelV1,
    pub(super) evidence_priors: [TrainingCountPriorV1; 4],
    pub(super) lemma_counts: BTreeMap<u32, TrainingCountPriorV1>,
    pub(super) paradigm_counts: BTreeMap<u32, TrainingCountPriorV1>,
    pub(super) calibration: CalibrationTableV1,
    pub(super) phase_profiles: Vec<TrainedSlotPhaseProfileV1>,
    pub(super) directional_residuals: Vec<DirectionalResidualRecordV1>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TrainingCountPriorV1 {
    pub(super) positive_count: u64,
    pub(super) contradiction_count: u64,
}

#[derive(Clone, Debug)]
pub(super) struct ProductivePackageCompilerConfigV1 {
    pub(super) output_path: PathBuf,
    pub(super) maximum_record_bytes: usize,
    pub(super) l11_package_sha256: [u8; 32],
    pub(super) canonical_l2_package_sha256: [u8; 32],
    pub(super) productive_package_byte_budget: u64,
    pub(super) steady_rss_kib_budget: u32,
    pub(super) peak_rss_kib_budget: u32,
    pub(super) cold_publish_budget_us: u32,
    pub(super) hot_p99_budget_us: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompiledProductivePackageV1 {
    pub(super) path: PathBuf,
    pub(super) package_sha256: [u8; 32],
    pub(super) package_bytes: u64,
    pub(super) paradigm_count: u32,
    pub(super) binding_count: u32,
    pub(super) program_count: u32,
    pub(super) operation_count: u32,
    pub(super) trie_node_count: u32,
    pub(super) trie_arc_count: u32,
    pub(super) terminal_count: u32,
    pub(super) anchor_recovery: Option<CompiledAnchorRecoveryPackageV1>,
}

#[derive(Clone, Debug)]
struct ParadigmDraftV1 {
    definition: ParadigmDefinitionV1,
    program_start: u32,
    program_count: u32,
    target_slots: Vec<u32>,
}

#[derive(Clone, Debug)]
struct PendingBindingV1 {
    record: LemmaParadigmBindingV1,
    observed_slots: Vec<u32>,
}

pub(super) fn compile_productive_package(
    reduced: &ReducedMorphologyManifestV1,
    induction: &TransitionInductionManifestV1,
    axis_schema: &MorphologyAxisSchemaV1,
    evidence: &ProductiveCompilerEvidenceV1,
    config: &ProductivePackageCompilerConfigV1,
) -> Result<CompiledProductivePackageV1, String> {
    if config.maximum_record_bytes == 0
        || config.l11_package_sha256 == [0; 32]
        || config.canonical_l2_package_sha256 == [0; 32]
    {
        return Err(
            "productive package compiler has an invalid identity or record bound".to_string(),
        );
    }
    let axis_schema_sha256 = axis_schema.validate_for_slots(&reduced.morphology_slots)?;
    if axis_schema_sha256 != induction.axis_schema_sha256 {
        return Err("productive package axis schema disagrees with induction".to_string());
    }
    let slot_ids = reduced
        .morphology_slots
        .iter()
        .copied()
        .enumerate()
        .map(|(index, slot)| (slot, index as u32 + 1))
        .collect::<BTreeMap<_, _>>();

    let paradigms = read_paradigms(induction, config.maximum_record_bytes)?;
    if paradigms.len() != induction.paradigm_count as usize {
        return Err("productive paradigm spool count disagrees with its manifest".to_string());
    }
    let representative_lengths =
        representative_anchor_lengths(induction, config.maximum_record_bytes)?;
    let mut segments = collect_segments(induction, &paradigms, config.maximum_record_bytes)?;
    let (mut segment_pool, mut segment_refs) = build_segment_pool(&segments)?;

    let packaged_calibration = evidence
        .calibration
        .package_rows()
        .map_err(str::to_string)?;
    let calibration_records = packaged_calibration
        .cells
        .iter()
        .map(|row| {
            Ok(CalibrationCellRecordV1 {
                stratum_key_id: row.stratum_key_id,
                winner_margin_q16: match row.cell.winner_margin_q16 {
                    Some(value) => i32::try_from(value)
                        .map_err(|_| "productive calibration winner margin exceeds i32")?,
                    None => i32::MIN,
                },
                tie_radius_q16: i32::try_from(row.cell.tie_radius_q16)
                    .map_err(|_| "productive calibration tie radius exceeds i32")?,
                support: row.cell.support,
                correct_winner_count: row.cell.correct_winner_count,
                false_winner_count: row.cell.false_winner_count,
                tied_count: row.cell.tied_count,
                flags: 0,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let fallback_calibration = packaged_calibration.generated_fallback_row;

    let feature_schema_hash = productive_feature_schema_hash_low().map_err(str::to_string)?;
    if evidence.evidence_model.training_pair_count == 0 {
        return Err("productive evidence model has no measured training denominator".to_string());
    }
    let coefficient_records = evidence
        .evidence_model
        .coefficients_q16
        .iter()
        .enumerate()
        .map(|(index, coefficient_q16)| ModelCoefficientRecordV1 {
            feature_id: index as u16 + 1,
            flags: 0,
            coefficient_q16: *coefficient_q16,
            train_support: evidence.evidence_model.training_pair_count,
            feature_schema_hash_low: feature_schema_hash,
        })
        .collect::<Vec<_>>();
    if coefficient_records.len() != PRODUCTIVE_FEATURE_COUNT {
        return Err("productive compiler lost a model coefficient".to_string());
    }
    let evidence_prior_records = compile_evidence_priors(&evidence.evidence_priors)?;

    let mut program_headers = Vec::new();
    let mut operations = Vec::new();
    let mut trie_programs = Vec::new();
    let mut drafts = Vec::with_capacity(paradigms.len());
    let mut terminal_hashes = BTreeSet::new();
    for paradigm in paradigms {
        let program_start = checked_u32(program_headers.len(), "program start")?;
        let anchor_scalar_len = *representative_lengths
            .get(&paradigm.paradigm_id)
            .ok_or_else(|| "productive paradigm lacks a representative anchor".to_string())?;
        let mut variants = BTreeMap::<u32, u16>::new();
        let mut target_slots = BTreeSet::new();
        for transition in &paradigm.signature.transitions {
            let source_slot_id = *slot_ids
                .get(&transition.source_slot)
                .ok_or_else(|| "productive transition source slot is unknown".to_string())?;
            let target_slot_id = *slot_ids
                .get(&transition.target_slot)
                .ok_or_else(|| "productive transition target slot is unknown".to_string())?;
            let variant_id = variants
                .entry(target_slot_id)
                .and_modify(|variant| *variant = variant.saturating_add(1))
                .or_insert(1);
            if *variant_id == u16::MAX {
                return Err("productive paradigm variant reaches the wire ceiling".to_string());
            }
            target_slots.insert(target_slot_id);
            let template = EditTemplateV1 {
                source_slot_id,
                target_slot_id,
                source_slot: transition.source_slot,
                target_slot: transition.target_slot,
                variant_id: *variant_id,
                operations: transition
                    .operations
                    .iter()
                    .cloned()
                    .chain([EditOperationV1::Terminate {
                        slot_id: target_slot_id,
                        variant_id: *variant_id,
                    }])
                    .collect(),
                transferable: true,
            };
            let program_id = checked_u32(program_headers.len() + 1, "program identity")?;
            append_program(
                &template,
                None,
                &segment_refs,
                &mut program_headers,
                &mut operations,
            )?;
            let evidence_ref = stable_u32(
                b"lay-productive-evidence-v1\0",
                &transition.canonical_bytes(),
            )?;
            let mut terminal_key = paradigm.paradigm_id.to_le_bytes().to_vec();
            terminal_key.extend_from_slice(&program_id.to_le_bytes());
            terminal_key.extend_from_slice(&target_slot_id.to_le_bytes());
            terminal_key.extend_from_slice(&variant_id.to_le_bytes());
            terminal_key.extend_from_slice(&transition.canonical_bytes());
            let stable_identity_hash = reserve_stable_u32(
                b"lay-productive-terminal-v1\0",
                &terminal_key,
                &mut terminal_hashes,
            )?;
            trie_programs.push(TrieProgramInputV1 {
                paradigm_id: paradigm.paradigm_id,
                anchor_scalar_len,
                template,
                exact_allomorph_surface: None,
                terminal: ProductiveTerminalAttributionV1 {
                    program_id,
                    target_slot_id,
                    variant_id: *variant_id,
                    decoder_ref: 0,
                    evidence_ref,
                    calibration_class: fallback_calibration,
                    provenance_ref: 0,
                    stable_identity_hash,
                },
            });
        }
        drafts.push(ParadigmDraftV1 {
            program_count: checked_u32(program_headers.len(), "program end")?
                .checked_sub(program_start)
                .ok_or_else(|| "productive paradigm program range underflow".to_string())?,
            program_start,
            target_slots: target_slots.into_iter().collect(),
            definition: paradigm,
        });
    }

    let forest = compile_productive_trie(&trie_programs).map_err(str::to_string)?;
    admit_compacted_trie_segments(
        &forest,
        &mut segments,
        &mut segment_pool,
        &mut segment_refs,
        &mut operations,
    )?;
    let (trie_nodes, trie_arcs, terminals) = flatten_trie(&forest, &segment_refs)?;
    let (pending_bindings, observed_slot_sets) = compile_bindings_and_local_programs(
        induction,
        config.maximum_record_bytes,
        &segment_refs,
        &evidence.lemma_counts,
        &mut program_headers,
        &mut operations,
    )?;
    let (axis_pool, observed_slot_refs) = build_axis_pool(axis_schema, &observed_slot_sets)?;
    let bindings = pending_bindings
        .into_iter()
        .map(|pending| {
            let mut record = pending.record;
            record.observed_slot_set_ref = *observed_slot_refs
                .get(&pending.observed_slots)
                .ok_or_else(|| {
                    "productive observed slot set lost its pool reference".to_string()
                })?;
            Ok(record)
        })
        .collect::<Result<Vec<_>, String>>()?;

    let phase_inputs = evidence
        .phase_profiles
        .iter()
        .map(|profile| ((profile.paradigm_id, profile.slot_id), profile))
        .collect::<BTreeMap<_, _>>();
    if phase_inputs.len() != evidence.phase_profiles.len() {
        return Err("productive phase evidence repeats a paradigm/slot profile".to_string());
    }
    let mut used_phase_inputs = BTreeSet::new();
    let mut phase_profiles = Vec::new();
    let mut positive_centers = Vec::new();
    let mut anti_centers = Vec::new();
    let mut hard_negative_centers = Vec::new();
    let mut ambiguity_centers = Vec::new();
    let mut paradigm_centers = Vec::with_capacity(drafts.len());
    for draft in &drafts {
        let profile_start = checked_u32(phase_profiles.len(), "phase profile start")?;
        for slot_id in &draft.target_slots {
            let trained = phase_inputs
                .get(&(draft.definition.paradigm_id, *slot_id))
                .copied();
            if trained.is_some() {
                used_phase_inputs.insert((draft.definition.paradigm_id, *slot_id));
            }
            let support = trained.map_or(draft.definition.lemma_support, |profile| profile.support);
            if support == 0 {
                return Err("productive phase profile has zero support".to_string());
            }
            phase_profiles.push(SlotPhaseProfileRecordV1 {
                slot_id: *slot_id,
                feature_schema_id: feature_schema_hash,
                positive_start: checked_u32(positive_centers.len(), "positive phase start")?,
                anti_start: checked_u32(anti_centers.len(), "anti phase start")?,
                hard_negative_start: checked_u32(
                    hard_negative_centers.len(),
                    "hard-negative phase start",
                )?,
                ambiguity_start: checked_u32(ambiguity_centers.len(), "ambiguity phase start")?,
                positive_count: append_phase_bank(
                    trained.map(|profile| &profile.positive),
                    &mut positive_centers,
                )?,
                anti_count: append_phase_bank(
                    trained.map(|profile| &profile.anti),
                    &mut anti_centers,
                )?,
                hard_negative_count: append_phase_bank(
                    trained.map(|profile| &profile.hard_negative),
                    &mut hard_negative_centers,
                )?,
                ambiguity_count: append_phase_bank(
                    trained.map(|profile| &profile.ambiguity),
                    &mut ambiguity_centers,
                )?,
                calibration_class: fallback_calibration,
                flags: 0,
                support,
                explicit_anti_support: trained.map_or(0, |profile| profile.explicit_anti_support),
            });
        }
        let signature_hash_low = stable_u32(
            b"lay-productive-signature-v1\0",
            &signature_bytes(&draft.definition.signature)?,
        )?;
        paradigm_centers.push(ParadigmCenterRecordV1 {
            pos_domain: u16::from(draft.definition.signature.pos_domain),
            flags: 0,
            root_node: *forest
                .roots_by_paradigm
                .get(&draft.definition.paradigm_id)
                .ok_or_else(|| "productive trie lacks a paradigm root".to_string())?,
            transition_start: draft.program_start,
            transition_count: draft.program_count,
            slot_profile_start: profile_start,
            slot_profile_count: checked_u32(phase_profiles.len(), "phase profile end")?
                .checked_sub(profile_start)
                .ok_or_else(|| "productive phase profile range underflow".to_string())?,
            program_start: draft.program_start,
            program_count: draft.program_count,
            support: evidence
                .paradigm_counts
                .get(&draft.definition.paradigm_id)
                .map(|counts| counts.positive_count)
                .map(|support| {
                    u32::try_from(support)
                        .map_err(|_| "productive paradigm evidence support exceeds u32")
                })
                .transpose()?
                .unwrap_or(draft.definition.lemma_support),
            stability: 0,
            calibration_class: fallback_calibration,
            provenance_ref: 0,
            signature_hash_low,
        });
    }
    if used_phase_inputs.len() != phase_inputs.len() {
        return Err("productive phase evidence references no compiled paradigm slot".to_string());
    }

    let (compatibility_indexes, compatibility_postings) =
        compile_compatibility(induction, config.maximum_record_bytes)?;
    let mut directional_residuals = evidence.directional_residuals.clone();
    directional_residuals.sort_unstable_by_key(|record| {
        (
            record.source_scene_key,
            record.from_slot_id,
            record.to_slot_id,
        )
    });
    if directional_residuals.windows(2).any(|pair| {
        (
            pair[0].source_scene_key,
            pair[0].from_slot_id,
            pair[0].to_slot_id,
        ) >= (
            pair[1].source_scene_key,
            pair[1].from_slot_id,
            pair[1].to_slot_id,
        )
    }) {
        return Err("productive directional residuals repeat an identity".to_string());
    }
    let provenance = compile_provenance(reduced)?;
    let training_manifest_sha256 = training_manifest_sha(reduced, induction, axis_schema_sha256);
    let delta_manifest = vec![DeltaManifestRecordV1 {
        base_package_sha256: [0; 32],
        previous_generation_sha256: [0; 32],
        generation: 1,
        event_start: 0,
        event_end: reduced.train_event_count,
        section_count_ref: REQUIRED_SECTION_COUNT as u64,
        coefficient_generation: 1,
        calibration_generation: 1,
        proof_receipt_sha256: [0; 32],
        requested_authority_scope: 1,
        flags: 0,
        payload_sha256: training_manifest_sha256,
    }];

    let maximum_generated_scalars =
        maximum_generated_scalars(reduced.maximum_observed_scalars, &drafts)?;
    let mut build = ProductivePackageBuildV1::with_empty_required_sections(
        ProductiveAlgorithmModeV1::ProductiveV1Model,
    );
    build.l11_package_sha256 = config.l11_package_sha256;
    build.canonical_l2_package_sha256 = config.canonical_l2_package_sha256;
    build.training_manifest_sha256 = training_manifest_sha256;
    build.maximum_observed_scalars = reduced.maximum_observed_scalars;
    build.maximum_generated_scalars = maximum_generated_scalars;
    build.maximum_program_operations = induction.maximum_program_operations.max(2);
    build.split_seed = reduced.split_seed;
    build.normalization_version = reduced.normalization_version;
    build.compiler_version = reduced.compiler_version;
    build.productive_package_byte_budget = config.productive_package_byte_budget;
    build.steady_rss_kib_budget = config.steady_rss_kib_budget;
    build.peak_rss_kib_budget = config.peak_rss_kib_budget;
    build.cold_publish_budget_us = config.cold_publish_budget_us;
    build.hot_p99_budget_us = config.hot_p99_budget_us;

    set_variable_section(
        &mut build,
        ProductiveSectionKindV1::AxisDictionaries,
        axis_pool,
        axis_schema.labels.len() + observed_slot_sets.len(),
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::SlotKeys,
        &reduced.morphology_slots,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::ParadigmCenters,
        &paradigm_centers,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::LemmaBindings,
        &bindings,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::ParadigmCompatibilityIndex,
        &compatibility_indexes,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::ParadigmPostings,
        &compatibility_postings,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::MorphProgramHeaders,
        &program_headers,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::MorphOperations,
        &operations,
    )?;
    set_variable_section(
        &mut build,
        ProductiveSectionKindV1::SegmentPool,
        segment_pool,
        segments.len(),
    )?;
    set_fixed_section(&mut build, ProductiveSectionKindV1::TrieNodes, &trie_nodes)?;
    set_fixed_section(&mut build, ProductiveSectionKindV1::TrieArcs, &trie_arcs)?;
    set_fixed_section(&mut build, ProductiveSectionKindV1::Terminals, &terminals)?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::SlotPhaseProfiles,
        &phase_profiles,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::PositivePhaseCenters,
        &positive_centers,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::AntiPhaseCenters,
        &anti_centers,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::HardNegativePhaseCenters,
        &hard_negative_centers,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::AmbiguityPhaseCenters,
        &ambiguity_centers,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::DirectionalResiduals,
        &directional_residuals,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::ModelCoefficients,
        &coefficient_records,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::EvidencePriors,
        &evidence_prior_records,
    )?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::CalibrationCells,
        &calibration_records,
    )?;
    set_fixed_section(&mut build, ProductiveSectionKindV1::Provenance, &provenance)?;
    set_fixed_section(
        &mut build,
        ProductiveSectionKindV1::DeltaManifest,
        &delta_manifest,
    )?;

    let bytes = encode_package(&build)?;
    let package_sha256: [u8; 32] = Sha256::digest(&bytes).into();
    write_atomic(&config.output_path, &bytes)?;
    let view = ProductivePackageViewV1::load(&config.output_path)?;
    if !view.mmap_backed() || view.backing_bytes() != bytes.len() {
        return Err("productive published package did not reopen through mmap".to_string());
    }
    let anchor_recovery = induction
        .anchor_recovery
        .as_ref()
        .map(|manifest| {
            compile_anchor_recovery_package(
                manifest,
                &recovery_sidecar_path(&config.output_path),
                package_sha256,
                reduced.split_seed,
                reduced.maximum_observed_scalars,
                maximum_generated_scalars,
                config.productive_package_byte_budget,
                bytes.len() as u64,
            )
        })
        .transpose()?;
    Ok(CompiledProductivePackageV1 {
        path: config.output_path.clone(),
        package_sha256,
        package_bytes: bytes.len() as u64,
        paradigm_count: checked_u32(paradigm_centers.len(), "paradigm count")?,
        binding_count: checked_u32(bindings.len(), "binding count")?,
        program_count: checked_u32(program_headers.len(), "program count")?,
        operation_count: checked_u32(operations.len(), "operation count")?,
        trie_node_count: checked_u32(trie_nodes.len(), "trie node count")?,
        trie_arc_count: checked_u32(trie_arcs.len(), "trie arc count")?,
        terminal_count: checked_u32(terminals.len(), "terminal count")?,
        anchor_recovery,
    })
}

fn read_paradigms(
    induction: &TransitionInductionManifestV1,
    maximum_record_bytes: usize,
) -> Result<Vec<ParadigmDefinitionV1>, String> {
    let mut reader =
        ParadigmDefinitionReaderV1::open(&induction.paradigms_path, maximum_record_bytes)?;
    let mut paradigms = Vec::new();
    while let Some(paradigm) = reader.next()? {
        if paradigm.paradigm_id != paradigms.len() as u32 + 1 {
            return Err("productive paradigm identities are not contiguous".to_string());
        }
        paradigms.push(paradigm);
    }
    Ok(paradigms)
}

fn representative_anchor_lengths(
    induction: &TransitionInductionManifestV1,
    maximum_record_bytes: usize,
) -> Result<BTreeMap<u32, u16>, String> {
    let mut assignments = LemmaParadigmAssignmentReaderV1::open(
        &induction.lemma_bindings_path,
        maximum_record_bytes,
    )?;
    let mut classified = ClassifiedTransitionReaderV1::open(
        &induction.classified_transitions_path,
        maximum_record_bytes,
    )?;
    let mut assignment = assignments.next()?;
    let mut row = classified.next()?;
    let mut lengths = BTreeMap::new();
    while let Some(current) = row.take() {
        let basin = (
            current.lemma_id,
            current.transition.source_slot.pos_domain(),
        );
        let source_len = u16::try_from(current.source_surface.chars().count())
            .map_err(|_| "productive representative anchor exceeds u16".to_string())?;
        if assignment.is_some_and(|binding| (binding.lemma_id, binding.pos_domain) < basin) {
            return Err("productive paradigm assignment has no classified lemma".to_string());
        }
        if let Some(binding) =
            assignment.filter(|binding| (binding.lemma_id, binding.pos_domain) == basin)
        {
            lengths.entry(binding.paradigm_id).or_insert(source_len);
            assignment = assignments.next()?;
        }
        row = classified.next()?;
        while row
            .as_ref()
            .is_some_and(|next| (next.lemma_id, next.transition.source_slot.pos_domain()) == basin)
        {
            row = classified.next()?;
        }
    }
    if assignment.is_some() {
        return Err("productive paradigm assignment remains unmatched".to_string());
    }
    Ok(lengths)
}

fn collect_segments(
    induction: &TransitionInductionManifestV1,
    paradigms: &[ParadigmDefinitionV1],
    maximum_record_bytes: usize,
) -> Result<BTreeSet<String>, String> {
    let mut segments = BTreeSet::new();
    for paradigm in paradigms {
        for transition in &paradigm.signature.transitions {
            collect_program_segments(&transition.operations, &mut segments)?;
        }
    }
    let mut classified = ClassifiedTransitionReaderV1::open(
        &induction.classified_transitions_path,
        maximum_record_bytes,
    )?;
    while let Some(row) = classified.next()? {
        if row.transfer_lemma_support < 2 {
            segments.insert(row.target_surface);
        }
    }
    Ok(segments)
}

fn collect_program_segments(
    operations: &[EditOperationV1],
    segments: &mut BTreeSet<String>,
) -> Result<(), String> {
    let mut emitted_run = String::new();
    for operation in operations {
        match operation {
            EditOperationV1::EmitSegment { segment } => {
                if segment.is_empty() {
                    return Err("productive program contains an empty segment".to_string());
                }
                segments.insert(segment.clone());
                emitted_run.push_str(segment);
            }
            EditOperationV1::ReplaceSourceRange { segment, .. } => {
                insert_emitted_run(segments, &mut emitted_run);
                if !segment.is_empty() {
                    segments.insert(segment.clone());
                    emitted_run.push_str(segment);
                }
            }
            _ => insert_emitted_run(segments, &mut emitted_run),
        }
    }
    insert_emitted_run(segments, &mut emitted_run);
    Ok(())
}

fn insert_emitted_run(segments: &mut BTreeSet<String>, emitted_run: &mut String) {
    if !emitted_run.is_empty() {
        segments.insert(std::mem::take(emitted_run));
    }
}

fn build_segment_pool(
    segments: &BTreeSet<String>,
) -> Result<(Vec<u8>, BTreeMap<String, u32>), String> {
    let mut bytes = b"SPV1".to_vec();
    bytes.extend_from_slice(&checked_u32(segments.len(), "segment count")?.to_le_bytes());
    let mut refs = BTreeMap::new();
    for segment in segments {
        let reference = checked_u32(bytes.len(), "segment reference")?;
        bytes.extend_from_slice(&checked_u32(segment.len(), "segment bytes")?.to_le_bytes());
        bytes.extend_from_slice(
            &u16::try_from(segment.chars().count())
                .map_err(|_| "productive segment scalar count exceeds u16".to_string())?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(segment.as_bytes());
        bytes.resize((bytes.len() + 7) & !7, 0);
        refs.insert(segment.clone(), reference);
    }
    Ok((bytes, refs))
}

fn admit_compacted_trie_segments(
    forest: &super::trie::ProductiveTrieForestV1,
    segments: &mut BTreeSet<String>,
    segment_pool: &mut Vec<u8>,
    segment_refs: &mut BTreeMap<String, u32>,
    operations: &mut [MorphOpRecordV1],
) -> Result<(), String> {
    let previous_count = segments.len();
    for node in &forest.nodes {
        for arc in &node.arcs {
            if let ProductiveTrieArcActionV1::EmitSegment { segment } = &arc.action {
                if segment.is_empty() {
                    return Err("productive compacted trie emitted an empty segment".to_string());
                }
                segments.insert(segment.clone());
            }
        }
    }
    if segments.len() == previous_count {
        return Ok(());
    }

    let old_segments = segment_refs
        .iter()
        .map(|(segment, reference)| (*reference, segment.clone()))
        .collect::<BTreeMap<_, _>>();
    let (new_pool, new_refs) = build_segment_pool(segments)?;
    for operation in operations {
        let reference = match operation.decoded_opcode().map_err(str::to_string)? {
            MorphOpcodeV1::EmitSegment | MorphOpcodeV1::EmitExactAllomorph => {
                Some(&mut operation.arg1)
            }
            MorphOpcodeV1::ReplaceSourceRange if operation.arg2 != 0 => Some(&mut operation.arg2),
            _ => None,
        };
        let Some(reference) = reference else {
            continue;
        };
        let segment = old_segments.get(reference).ok_or_else(|| {
            "productive operation references an unknown prior segment".to_string()
        })?;
        *reference = *new_refs
            .get(segment)
            .ok_or_else(|| "productive operation segment disappeared during repack".to_string())?;
    }
    *segment_pool = new_pool;
    *segment_refs = new_refs;
    Ok(())
}

fn append_program(
    template: &EditTemplateV1,
    exact_surface: Option<&str>,
    segment_refs: &BTreeMap<String, u32>,
    headers: &mut Vec<MorphProgramHeaderRecordV1>,
    operations: &mut Vec<MorphOpRecordV1>,
) -> Result<(), String> {
    let op_start = checked_u32(operations.len(), "operation start")?;
    for operation in &template.operations {
        operations.push(match operation {
            EditOperationV1::CopySourceRange {
                start_anchor,
                start_delta,
                scalar_count,
            } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::CopySourceRange as u8,
                anchor: *start_anchor as u8,
                flags: 0,
                arg0: i32::from(*start_delta),
                arg1: u32::from(*scalar_count),
                arg2: 0,
            },
            EditOperationV1::DropSourcePrefix { scalar_count } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::DropSourcePrefix as u8,
                arg1: u32::from(*scalar_count),
                ..MorphOpRecordV1::default()
            },
            EditOperationV1::DropSourceSuffix { scalar_count } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::DropSourceSuffix as u8,
                arg1: u32::from(*scalar_count),
                ..MorphOpRecordV1::default()
            },
            EditOperationV1::EmitSegment { segment } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::EmitSegment as u8,
                arg1: required_segment_ref(segment_refs, segment, "morph_emit")?,
                ..MorphOpRecordV1::default()
            },
            EditOperationV1::ReplaceSourceRange {
                end_relative_offset,
                delete_count,
                segment,
            } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::ReplaceSourceRange as u8,
                anchor: SourceAnchorV1::End as u8,
                arg0: i32::from(*end_relative_offset),
                arg1: u32::from(*delete_count),
                arg2: replacement_segment_ref(segment_refs, segment)?,
                ..MorphOpRecordV1::default()
            },
            EditOperationV1::EmitExactAllomorph { .. } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::EmitExactAllomorph as u8,
                arg1: required_segment_ref(
                    segment_refs,
                    exact_surface.ok_or_else(|| {
                        "productive exact allomorph lacks its decoder surface".to_string()
                    })?,
                    "exact_allomorph",
                )?,
                ..MorphOpRecordV1::default()
            },
            EditOperationV1::Terminate {
                slot_id,
                variant_id,
            } => MorphOpRecordV1 {
                opcode: MorphOpcodeV1::Terminate as u8,
                arg1: *slot_id,
                arg2: u32::from(*variant_id),
                ..MorphOpRecordV1::default()
            },
        });
    }
    headers.push(MorphProgramHeaderRecordV1 {
        source_slot_id: template.source_slot_id,
        target_slot_id: template.target_slot_id,
        op_start,
        op_count: u16::try_from(template.operations.len())
            .map_err(|_| "productive program operation count exceeds u16".to_string())?,
        flags: 0,
    });
    Ok(())
}

fn required_segment_ref(
    refs: &BTreeMap<String, u32>,
    segment: &str,
    owner: &str,
) -> Result<u32, String> {
    refs.get(segment).copied().ok_or_else(|| {
        format!(
            "productive program segment has no pool reference owner={owner} bytes={} scalars={}",
            segment.len(),
            segment.chars().count()
        )
    })
}

fn replacement_segment_ref(refs: &BTreeMap<String, u32>, segment: &str) -> Result<u32, String> {
    if segment.is_empty() {
        Ok(0)
    } else {
        required_segment_ref(refs, segment, "morph_replace")
    }
}

type FlatTrieSections = (
    Vec<ProductiveTrieNodeRecordV1>,
    Vec<ProductiveTrieArcRecordV1>,
    Vec<ProductiveTerminalRecordV1>,
);

fn flatten_trie(
    forest: &super::trie::ProductiveTrieForestV1,
    segment_refs: &BTreeMap<String, u32>,
) -> Result<FlatTrieSections, String> {
    let mut nodes = Vec::with_capacity(forest.nodes.len());
    let mut arcs = Vec::new();
    let mut terminals = Vec::new();
    for node in &forest.nodes {
        let arc_start = checked_u32(arcs.len(), "trie arc start")?;
        let mut node_arcs = node.arcs.clone();
        node_arcs.sort_unstable_by_key(|arc| arc.stable_order);
        for arc in node_arcs {
            arcs.push(trie_arc_record(arc, segment_refs)?);
        }
        let terminal_start = checked_u32(terminals.len(), "trie terminal start")?;
        let mut node_terminals = node.terminals.clone();
        node_terminals.sort_unstable();
        for terminal in node_terminals {
            terminals.push(ProductiveTerminalRecordV1 {
                program_id: terminal.program_id,
                target_slot_id: terminal.target_slot_id,
                variant_id: terminal.variant_id,
                flags: PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE,
                decoder_ref: 0,
                evidence_ref: terminal.evidence_ref,
                calibration_class: terminal.calibration_class,
                provenance_ref: terminal.provenance_ref,
                stable_identity_hash: terminal.stable_identity_hash,
            });
        }
        nodes.push(ProductiveTrieNodeRecordV1 {
            arc_start,
            arc_count: u16::try_from(arcs.len() - arc_start as usize)
                .map_err(|_| "productive trie node arc count exceeds u16".to_string())?,
            terminal_count: u16::try_from(terminals.len() - terminal_start as usize)
                .map_err(|_| "productive trie node terminal count exceeds u16".to_string())?,
            terminal_start,
            flags: 0,
        });
    }
    Ok((nodes, arcs, terminals))
}

fn trie_arc_record(
    arc: super::trie::ProductiveTrieArcV1,
    segment_refs: &BTreeMap<String, u32>,
) -> Result<ProductiveTrieArcRecordV1, String> {
    let (opcode, anchor, arg0, arg1, arg2) = match arc.action {
        ProductiveTrieArcActionV1::CopySourceRange {
            source_anchor,
            source_delta,
            scalar_count,
        } => (
            ProductiveTrieArcOpcodeV1::CopySourceRange,
            source_anchor as u8,
            i32::from(source_delta),
            u32::from(scalar_count),
            0,
        ),
        ProductiveTrieArcActionV1::CopyToRetainedEdge {
            source_anchor,
            source_delta,
            retained_end_delta,
        } => (
            ProductiveTrieArcOpcodeV1::CopyToRetainedEdge,
            source_anchor as u8,
            i32::from(source_delta),
            u32::from(u16::from_le_bytes(retained_end_delta.to_le_bytes())),
            0,
        ),
        ProductiveTrieArcActionV1::DropSourcePrefix { scalar_count } => (
            ProductiveTrieArcOpcodeV1::DropSourcePrefix,
            0,
            0,
            u32::from(scalar_count),
            0,
        ),
        ProductiveTrieArcActionV1::DropSourceSuffix { scalar_count } => (
            ProductiveTrieArcOpcodeV1::DropSourceSuffix,
            0,
            0,
            u32::from(scalar_count),
            0,
        ),
        ProductiveTrieArcActionV1::EmitSegment { segment } => (
            ProductiveTrieArcOpcodeV1::EmitSegment,
            0,
            0,
            required_segment_ref(segment_refs, &segment, "trie_emit")?,
            0,
        ),
        ProductiveTrieArcActionV1::ReplaceSourceStart {
            end_relative_offset,
            delete_count,
        } => (
            ProductiveTrieArcOpcodeV1::ReplaceSourceStart,
            SourceAnchorV1::End as u8,
            i32::from(end_relative_offset),
            u32::from(delete_count),
            0,
        ),
        ProductiveTrieArcActionV1::EmitExactAllomorph { .. } => {
            return Err("lemma-local exact allomorph leaked into the paradigm trie".to_string())
        }
    };
    Ok(ProductiveTrieArcRecordV1 {
        child_node: arc.child_node,
        stable_order: arc.stable_order,
        opcode: opcode as u8,
        anchor,
        flags: 0,
        arg0,
        arg1,
        arg2,
    })
}

fn compile_bindings_and_local_programs(
    induction: &TransitionInductionManifestV1,
    maximum_record_bytes: usize,
    segment_refs: &BTreeMap<String, u32>,
    lemma_counts: &BTreeMap<u32, TrainingCountPriorV1>,
    program_headers: &mut Vec<MorphProgramHeaderRecordV1>,
    operations: &mut Vec<MorphOpRecordV1>,
) -> Result<(Vec<PendingBindingV1>, BTreeSet<Vec<u32>>), String> {
    let mut assignments = LemmaParadigmAssignmentReaderV1::open(
        &induction.lemma_bindings_path,
        maximum_record_bytes,
    )?;
    let mut assignment = assignments.next()?;
    let mut classified = ClassifiedTransitionReaderV1::open(
        &induction.classified_transitions_path,
        maximum_record_bytes,
    )?;
    let mut row = classified.next()?;
    let mut bindings = Vec::new();
    let mut observed_sets = BTreeSet::new();
    while let Some(first) = row.take() {
        let lemma_id = first.lemma_id;
        let pos_domain = first.transition.source_slot.pos_domain();
        let basin = (lemma_id, pos_domain);
        let mut group = vec![first];
        row = classified.next()?;
        while row
            .as_ref()
            .is_some_and(|next| (next.lemma_id, next.transition.source_slot.pos_domain()) == basin)
        {
            group.push(row.take().expect("classified lemma row"));
            row = classified.next()?;
        }
        if assignment.is_some_and(|binding| (binding.lemma_id, binding.pos_domain) < basin) {
            return Err("productive assignment has no classified lemma".to_string());
        }
        let Some(matched) =
            assignment.filter(|binding| (binding.lemma_id, binding.pos_domain) == basin)
        else {
            continue;
        };
        assignment = assignments.next()?;
        let source_form_ref = group[0].source_form_ref;
        if group
            .iter()
            .any(|item| item.source_form_ref != source_form_ref)
        {
            return Err("productive lemma has multiple canonical source forms".to_string());
        }
        let mut observed_slots = group
            .iter()
            .map(|item| item.target_slot_id)
            .collect::<Vec<_>>();
        observed_slots.sort_unstable();
        observed_slots.dedup();
        let positive_support = group.iter().try_fold(0_u32, |total, item| {
            total
                .checked_add(item.target_support)
                .ok_or_else(|| "productive lemma binding support exceeds u32".to_string())
        })?;
        let program_start = checked_u32(program_headers.len(), "lemma-local program start")?;
        for item in group.iter().filter(|item| item.transfer_lemma_support < 2) {
            append_program(
                &item.template(),
                Some(&item.target_surface),
                segment_refs,
                program_headers,
                operations,
            )?;
        }
        let program_count = program_headers.len() - program_start as usize;
        let measured = lemma_counts.get(&lemma_id).copied();
        bindings.push(PendingBindingV1 {
            record: LemmaParadigmBindingV1 {
                lemma_id,
                paradigm_id: matched.paradigm_id,
                canonical_source_form_ref: source_form_ref,
                observed_slot_set_ref: 0,
                positive_support: measured
                    .map(|counts| {
                        u32::try_from(counts.positive_count)
                            .map_err(|_| "productive lemma positive support exceeds u32")
                    })
                    .transpose()?
                    .unwrap_or(positive_support),
                explicit_anti_support: measured
                    .map(|counts| {
                        u32::try_from(counts.contradiction_count)
                            .map_err(|_| "productive lemma anti support exceeds u32")
                    })
                    .transpose()?
                    .unwrap_or_default(),
                stability: 0,
                flags: 0,
                program_start,
                program_count: u16::try_from(program_count)
                    .map_err(|_| "productive lemma-local program count exceeds u16".to_string())?,
                provenance_ref: 0,
            },
            observed_slots: observed_slots.clone(),
        });
        observed_sets.insert(observed_slots);
    }
    if assignment.is_some() {
        return Err("productive assignment remains unmatched after binding reduce".to_string());
    }
    Ok((bindings, observed_sets))
}

type AxisPool = (Vec<u8>, BTreeMap<Vec<u32>, u32>);

fn build_axis_pool(
    schema: &MorphologyAxisSchemaV1,
    observed_sets: &BTreeSet<Vec<u32>>,
) -> Result<AxisPool, String> {
    let count = schema
        .labels
        .len()
        .checked_add(observed_sets.len())
        .ok_or_else(|| "productive axis pool count overflow".to_string())?;
    let mut bytes = b"ADV1".to_vec();
    bytes.extend_from_slice(&checked_u32(count, "axis pool count")?.to_le_bytes());
    for label in &schema.labels {
        let mut payload = vec![1, label.axis, label.value, 0];
        payload.extend_from_slice(label.label.as_bytes());
        append_pool_entry(
            &mut bytes,
            u16::try_from(label.label.chars().count())
                .map_err(|_| "productive axis label exceeds u16 scalars".to_string())?,
            &payload,
        )?;
    }
    let mut refs = BTreeMap::new();
    for slots in observed_sets {
        let reference = checked_u32(bytes.len(), "observed slot set reference")?;
        let mut payload = vec![2, 0, 0, 0];
        payload.extend_from_slice(&checked_u32(slots.len(), "observed slot count")?.to_le_bytes());
        for slot in slots {
            payload.extend_from_slice(&slot.to_le_bytes());
        }
        append_pool_entry(&mut bytes, 0, &payload)?;
        refs.insert(slots.clone(), reference);
    }
    Ok((bytes, refs))
}

fn append_pool_entry(bytes: &mut Vec<u8>, scalar_count: u16, payload: &[u8]) -> Result<(), String> {
    bytes.extend_from_slice(&checked_u32(payload.len(), "pool entry bytes")?.to_le_bytes());
    bytes.extend_from_slice(&scalar_count.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes.resize((bytes.len() + 7) & !7, 0);
    Ok(())
}

fn append_phase_bank(
    bank: Option<&FittedPhaseBankV1>,
    output: &mut Vec<PhaseCenterRecordV1>,
) -> Result<u16, String> {
    let Some(bank) = bank else {
        return Ok(0);
    };
    let count = u16::try_from(bank.centers.len())
        .map_err(|_| "productive phase bank center count exceeds u16".to_string())?;
    output.extend(bank.centers.iter().copied().map(PhaseCenterRecordV1::from));
    Ok(count)
}

fn compile_compatibility(
    induction: &TransitionInductionManifestV1,
    maximum_record_bytes: usize,
) -> Result<
    (
        Vec<ParadigmCompatibilityIndexRecordV1>,
        Vec<ParadigmPostingRecordV1>,
    ),
    String,
> {
    let mut index_reader = ParadigmCompatibilityIndexReaderV1::open(
        &induction.compatibility_index_path,
        maximum_record_bytes,
    )?;
    let mut indexes = Vec::new();
    while let Some(index) = index_reader.next()? {
        indexes.push(ParadigmCompatibilityIndexRecordV1 {
            pos_domain: u16::from(index.pos_domain),
            flags: 0,
            source_slot_id: index.source_slot_id,
            posting_start: u32::try_from(index.posting_start)
                .map_err(|_| "productive compatibility posting start exceeds u32".to_string())?,
            posting_count: index.posting_count,
        });
    }
    let mut posting_reader =
        ParadigmPostingReaderV1::open(&induction.paradigm_postings_path, maximum_record_bytes)?;
    let mut postings = Vec::new();
    while let Some(paradigm_id) = posting_reader.next()? {
        postings.push(ParadigmPostingRecordV1 { paradigm_id });
    }
    if indexes.len() != induction.compatibility_index_count as usize
        || postings.len() as u64 != induction.compatibility_posting_count
    {
        return Err("productive compatibility spool denominator mismatch".to_string());
    }
    Ok((indexes, postings))
}

fn compile_provenance(
    reduced: &ReducedMorphologyManifestV1,
) -> Result<Vec<ProvenanceRecordV1>, String> {
    let mut records = Vec::new();
    let source_hash_prefix = u64::from_le_bytes(
        reduced.payload_sha256[0..8]
            .try_into()
            .expect("SHA-256 prefix"),
    );
    let mut start = 0_u64;
    while start < reduced.train_event_count {
        let remaining = reduced.train_event_count - start;
        let count = remaining.min(u64::from(u32::MAX)) as u32;
        records.push(ProvenanceRecordV1 {
            source_kind: 1,
            flags: 0,
            source_id: 1,
            event_start: start,
            event_count: count,
            source_hash_prefix,
        });
        start += u64::from(count);
    }
    if records.is_empty() {
        return Err("productive compiler has no TRAIN provenance".to_string());
    }
    Ok(records)
}

fn maximum_generated_scalars(
    maximum_observed: u16,
    paradigms: &[ParadigmDraftV1],
) -> Result<u16, String> {
    let maximum_growth = paradigms
        .iter()
        .flat_map(|paradigm| &paradigm.definition.signature.transitions)
        .map(|transition| {
            transition
                .operations
                .iter()
                .try_fold(0_usize, |growth, operation| {
                    let emitted = match operation {
                        EditOperationV1::EmitSegment { segment }
                        | EditOperationV1::ReplaceSourceRange { segment, .. } => {
                            segment.chars().count()
                        }
                        _ => 0,
                    };
                    growth
                        .checked_add(emitted)
                        .ok_or_else(|| "productive generated growth overflow".to_string())
                })
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .max()
        .unwrap_or(0);
    let maximum = usize::from(maximum_observed)
        .checked_add(maximum_growth)
        .ok_or_else(|| "productive maximum generated scalars overflow".to_string())?;
    if maximum >= u16::MAX as usize {
        return Err("productive maximum generated scalars reaches wire ceiling".to_string());
    }
    Ok(maximum as u16)
}

fn training_manifest_sha(
    reduced: &ReducedMorphologyManifestV1,
    induction: &TransitionInductionManifestV1,
    axis_schema_sha256: [u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-productive-training-manifest-v1\0");
    hasher.update(reduced.payload_sha256);
    hasher.update(axis_schema_sha256);
    hasher.update(induction.transition_observations.to_le_bytes());
    hasher.update(induction.transferable_observations.to_le_bytes());
    hasher.update(induction.exact_allomorph_observations.to_le_bytes());
    hasher.update(induction.paradigm_count.to_le_bytes());
    hasher.finalize().into()
}

fn compile_evidence_priors(
    priors: &[TrainingCountPriorV1; 4],
) -> Result<Vec<EvidencePriorRecordV1>, String> {
    priors
        .iter()
        .enumerate()
        .map(|(index, prior)| {
            let positive_prior_twice = prior
                .positive_count
                .checked_mul(2)
                .and_then(|value| value.checked_add(1))
                .ok_or_else(|| "productive positive TRAIN prior reaches u64".to_string())?;
            let contradiction_prior_twice = prior
                .contradiction_count
                .checked_mul(2)
                .and_then(|value| value.checked_add(1))
                .ok_or_else(|| "productive contradiction TRAIN prior reaches u64".to_string())?;
            Ok(EvidencePriorRecordV1 {
                channel_id: index as u16 + 1,
                flags: 0,
                positive_prior_twice,
                contradiction_prior_twice,
                reserved: 0,
            })
        })
        .collect()
}

fn set_fixed_section<T: super::records::FixedRecordV1>(
    build: &mut ProductivePackageBuildV1,
    kind: ProductiveSectionKindV1,
    records: &[T],
) -> Result<(), String> {
    let section = ProductiveSectionBuildV1::fixed_records(kind, records)?;
    replace_section(build, section)
}

fn set_variable_section(
    build: &mut ProductivePackageBuildV1,
    kind: ProductiveSectionKindV1,
    bytes: Vec<u8>,
    count: usize,
) -> Result<(), String> {
    replace_section(
        build,
        ProductiveSectionBuildV1 {
            kind,
            flags: 0,
            record_size: 0,
            count: checked_u32(count, "variable section count")?,
            bytes,
        },
    )
}

fn replace_section(
    build: &mut ProductivePackageBuildV1,
    section: ProductiveSectionBuildV1,
) -> Result<(), String> {
    let target = build
        .sections
        .iter_mut()
        .find(|candidate| candidate.kind == section.kind)
        .ok_or_else(|| "productive compiler lost a required section".to_string())?;
    *target = section;
    Ok(())
}

fn stable_u32(domain: &[u8], payload: &[u8]) -> Result<u32, String> {
    let value = stable_u32_probe(domain, payload, 0);
    if value == 0 {
        return Err("productive stable typed hash is zero".to_string());
    }
    Ok(value)
}

fn reserve_stable_u32(
    domain: &[u8],
    payload: &[u8],
    used: &mut BTreeSet<u32>,
) -> Result<u32, String> {
    for probe in 0..=u32::MAX {
        let value = stable_u32_probe(domain, payload, probe);
        if value != 0 && used.insert(value) {
            return Ok(value);
        }
    }
    Err("productive stable u32 identity space is exhausted".to_string())
}

fn stable_u32_probe(domain: &[u8], payload: &[u8], probe: u32) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(payload);
    if probe != 0 {
        hasher.update(b"\0collision-probe\0");
        hasher.update(probe.to_le_bytes());
    }
    let digest = hasher.finalize();
    u32::from_le_bytes(digest[0..4].try_into().expect("SHA-256 prefix"))
}

fn checked_u32(value: usize, label: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("productive {label} exceeds u32"))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "productive package output has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::super::calibrate::{
        fit_calibration_table, CalibrationCandidateV1, CalibrationGroupV1,
        CandidateProvenanceClassV1, ObservableCalibrationStratumV1,
        AMBIGUITY_SAME_LEMMA_MULTI_LABEL,
    };
    use super::super::events::{
        deterministic_productive_split, LemmaSplitKeyV1, MorphologyEventV1, ProductiveSplitV1,
        TypedEventSpoolConfigV1, TypedEventSpoolWriterV1, TypedProductiveEventV1,
    };
    use super::super::packaged_runtime::{
        ColdLemmaSourceV1, PackagedGroundedLemmaV1, PackagedProductiveRuntimeV1,
        PreparedMorphProgramV1,
    };
    use super::super::records::decode_records;
    use super::super::reduce::{reduce_train_morphology, TrainMorphologyReduceConfigV1};
    use super::super::scene::L2LocalSceneV1;
    use super::super::score::{fit_evidence_model, FeatureVectorV1, PairwiseTrainingPairV1};
    use super::super::spool_sort::{external_sort_verified_spool, ExternalSpoolSortConfigV1};
    use super::super::transition_reduce::{
        induce_transition_field, MorphologyAxisLabelV1, TransitionReduceConfigV1,
    };
    use super::super::types::{
        MorphologyApplicabilityMaskV1, MorphologySlotKeyV1, ProductiveCandidateIdentityV1,
        AXIS_INAPPLICABLE,
    };
    use super::*;

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lay-productive-compiler-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ))
    }

    fn slot(number: u8) -> MorphologySlotKeyV1 {
        MorphologySlotKeyV1::new(
            2,
            number,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
            AXIS_INAPPLICABLE,
        )
    }

    fn schema() -> MorphologyAxisSchemaV1 {
        MorphologyAxisSchemaV1 {
            schema_version: 1,
            pos_applicability: [(2, MorphologyApplicabilityMaskV1::new(0b11).expect("mask"))]
                .into_iter()
                .collect(),
            labels: vec![
                MorphologyAxisLabelV1 {
                    axis: 0,
                    value: 2,
                    label: "fixture-pos".to_string(),
                },
                MorphologyAxisLabelV1 {
                    axis: 1,
                    value: 2,
                    label: "fixture-source".to_string(),
                },
                MorphologyAxisLabelV1 {
                    axis: 1,
                    value: 3,
                    label: "fixture-target".to_string(),
                },
            ],
        }
    }

    fn morphology_event(
        lemma: &str,
        surface: &str,
        number: u8,
        support: u32,
    ) -> TypedProductiveEventV1 {
        TypedProductiveEventV1::Morphology(MorphologyEventV1 {
            lemma: LemmaSplitKeyV1 {
                language: "fixture".to_string(),
                normalized_lemma: lemma.to_string(),
            },
            normalized_surface: surface.to_string(),
            canonical_form_ref: super::super::types::ImportedCanonicalL2FormRefV1(1),
            canonical_feature_mask: 1,
            slot: slot(number),
            support,
            provenance: format!("compiler-fixture:{lemma}:{surface}").into_bytes(),
        })
    }

    fn fitted_evidence_model() -> FittedEvidenceModelV1 {
        let pairs = (0..super::super::PRODUCTIVE_V1_INNER_FOLDS as u8)
            .map(|fold| {
                let mut group_identity = [0_u8; 32];
                group_identity[0] = fold;
                let mut event_identity = group_identity;
                event_identity[1] = 1;
                let mut valid = FeatureVectorV1::default();
                valid.0[0] = 1.0 + f64::from(fold) / 10.0;
                let mut contradicted = FeatureVectorV1::default();
                contradicted.0[1] = 1.0 + f64::from(fold) / 10.0;
                PairwiseTrainingPairV1 {
                    group_identity,
                    stable_event_identity: event_identity,
                    inner_fold: fold,
                    valid,
                    contradicted,
                }
            })
            .collect::<Vec<_>>();
        fit_evidence_model(&pairs).expect("fitted evidence model")
    }

    fn identity(index: u32) -> ProductiveCandidateIdentityV1 {
        ProductiveCandidateIdentityV1 {
            lemma_id: index + 1,
            paradigm_id: 1,
            program_id: index + 1,
            target_slot_id: 2,
            normalized_surface_id: index + 1,
            variant_id: 1,
        }
    }

    fn fitted_calibration() -> CalibrationTableV1 {
        let stratum = ObservableCalibrationStratumV1::new(
            "fixture-observed",
            "fixture-candidate",
            CandidateProvenanceClassV1::TrainingSeenGenerated,
            300,
            AMBIGUITY_SAME_LEMMA_MULTI_LABEL,
        )
        .expect("calibration stratum");
        let groups = (0..220_u32)
            .map(|index| {
                let mut group_identity = [0_u8; 32];
                group_identity[0..4].copy_from_slice(&index.to_le_bytes());
                CalibrationGroupV1 {
                    split: ProductiveSplitV1::Calibration,
                    group_identity,
                    stratum: stratum.clone(),
                    candidates: vec![
                        CalibrationCandidateV1 {
                            identity: identity(index * 2),
                            normalized_surface: format!("valid-{index}"),
                            score_q16: 1_000,
                            grounded_lemma_evidence: 0,
                            exact_osa_distance: 0,
                            exact_form: false,
                            gold_valid: true,
                        },
                        CalibrationCandidateV1 {
                            identity: identity(index * 2 + 1),
                            normalized_surface: format!("alternative-{index}"),
                            score_q16: 900,
                            grounded_lemma_evidence: 0,
                            exact_osa_distance: 0,
                            exact_form: false,
                            gold_valid: false,
                        },
                    ],
                    false_singleton: false,
                    grounded_winner_protection_violation: false,
                }
            })
            .collect::<Vec<_>>();
        fit_calibration_table(&groups).expect("fitted calibration")
    }

    #[test]
    fn stable_terminal_ids_resolve_u32_collisions_deterministically() {
        let mut first_used = BTreeSet::new();
        let first = reserve_stable_u32(b"fixture-domain", b"fixture-terminal", &mut first_used)
            .expect("first identity");
        let collision = reserve_stable_u32(b"fixture-domain", b"fixture-terminal", &mut first_used)
            .expect("collision probe");
        assert_ne!(first, 0);
        assert_ne!(collision, 0);
        assert_ne!(first, collision);

        let mut replay_used = BTreeSet::new();
        assert_eq!(
            reserve_stable_u32(b"fixture-domain", b"fixture-terminal", &mut replay_used)
                .expect("replayed first"),
            first
        );
        assert_eq!(
            reserve_stable_u32(b"fixture-domain", b"fixture-terminal", &mut replay_used)
                .expect("replayed collision"),
            collision
        );
    }

    #[test]
    fn segment_pool_covers_radix_compacted_emit_runs() {
        let operations = vec![
            EditOperationV1::EmitSegment {
                segment: "ab".to_string(),
            },
            EditOperationV1::EmitSegment {
                segment: "cd".to_string(),
            },
            EditOperationV1::ReplaceSourceRange {
                end_relative_offset: -1,
                delete_count: 1,
                segment: "x".to_string(),
            },
            EditOperationV1::EmitSegment {
                segment: "y".to_string(),
            },
        ];
        let mut segments = BTreeSet::new();
        collect_program_segments(&operations, &mut segments).expect("segment collection");
        assert_eq!(
            segments,
            ["ab", "abcd", "cd", "x", "xy", "y"]
                .into_iter()
                .map(str::to_string)
                .collect()
        );
    }

    #[test]
    fn actual_trie_segments_repack_existing_operation_references() {
        let mut segments = BTreeSet::from(["z".to_string()]);
        let (mut pool, mut refs) = build_segment_pool(&segments).expect("initial pool");
        let old_reference = refs["z"];
        let mut operations = vec![MorphOpRecordV1 {
            opcode: MorphOpcodeV1::EmitSegment as u8,
            arg1: old_reference,
            ..MorphOpRecordV1::default()
        }];
        let mut forest = super::super::trie::ProductiveTrieForestV1::default();
        forest.nodes.push(super::super::trie::ProductiveTrieNodeV1 {
            arcs: vec![super::super::trie::ProductiveTrieArcV1 {
                action: ProductiveTrieArcActionV1::EmitSegment {
                    segment: "a".to_string(),
                },
                child_node: 1,
                order_target_slot_id: 1,
                order_variant_id: 1,
                stable_order: 1,
            }],
            ..super::super::trie::ProductiveTrieNodeV1::default()
        });

        admit_compacted_trie_segments(
            &forest,
            &mut segments,
            &mut pool,
            &mut refs,
            &mut operations,
        )
        .expect("actual trie segment admission");

        assert!(refs.contains_key("a"));
        assert_ne!(refs["z"], old_reference);
        assert_eq!(operations[0].arg1, refs["z"]);
    }

    #[test]
    fn compiler_is_deterministic_and_reopens_a_deep_valid_mmap_package() {
        let root = root();
        let split_seed = 17;
        let mut writer = TypedEventSpoolWriterV1::create(TypedEventSpoolConfigV1 {
            root: root.join("raw"),
            shard_count: 1,
            split_seed,
            compiler_version: 1,
            normalization_version: 1,
            write_buffer_bytes: 4096,
        })
        .expect("event writer");

        let mut admitted = 0_u32;
        let mut regular_lemmas = Vec::new();
        for index in 0..2_000_u32 {
            let lemma = format!("lemma-{index:04}");
            let split_key = LemmaSplitKeyV1 {
                language: "fixture".to_string(),
                normalized_lemma: lemma.clone(),
            };
            if deterministic_productive_split(&split_key, split_seed) != ProductiveSplitV1::Train {
                continue;
            }
            writer
                .append(&morphology_event(
                    &lemma,
                    &format!("stem{index:04}abc"),
                    2,
                    4,
                ))
                .expect("source morphology");
            writer
                .append(&morphology_event(
                    &lemma,
                    &format!("stem{index:04}ac"),
                    3,
                    3,
                ))
                .expect("target morphology");
            regular_lemmas.push((
                lemma,
                format!("stem{index:04}abc"),
                format!("stem{index:04}ac"),
            ));
            admitted += 1;
            if admitted == 300 {
                break;
            }
        }
        assert_eq!(admitted, 300);

        let irregular_lemma = (0..2_000_u32)
            .map(|index| format!("irregular-{index:04}"))
            .find(|lemma| {
                deterministic_productive_split(
                    &LemmaSplitKeyV1 {
                        language: "fixture".to_string(),
                        normalized_lemma: lemma.clone(),
                    },
                    split_seed,
                ) == ProductiveSplitV1::Train
            })
            .expect("TRAIN irregular lemma");
        writer
            .append(&morphology_event(&irregular_lemma, "irregulara", 2, 4))
            .expect("irregular source");
        writer
            .append(&morphology_event(&irregular_lemma, "irregularzz", 3, 3))
            .expect("irregular target");

        let raw = writer.finish().expect("raw spool");
        let sorted = external_sort_verified_spool(
            &raw,
            &ExternalSpoolSortConfigV1 {
                root: root.join("sorted"),
                maximum_buffer_bytes: 1024,
                maximum_open_runs: 4,
                write_buffer_bytes: 1024,
            },
        )
        .expect("sorted spool");
        let reduced = reduce_train_morphology(
            &sorted,
            &TrainMorphologyReduceConfigV1 {
                output_path: root.join("lemmas.p2l"),
                write_buffer_bytes: 1024,
                maximum_lemma_bytes: 4096,
            },
        )
        .expect("morphology reduce");
        let axis_schema = schema();
        let induction = induce_transition_field(
            &reduced,
            &axis_schema,
            &TransitionReduceConfigV1 {
                root: root.join("induction"),
                maximum_buffer_bytes: 4096,
                maximum_open_runs: 4,
                write_buffer_bytes: 1024,
                maximum_record_bytes: 4096,
                maximum_lemma_transitions: 16,
            },
        )
        .expect("transition induction");
        assert_eq!(induction.bound_lemma_count, 301);
        assert!(induction.transferable_observations >= 601);
        assert_eq!(induction.exact_allomorph_observations, 1);

        let evidence = ProductiveCompilerEvidenceV1 {
            evidence_model: fitted_evidence_model(),
            evidence_priors: [
                TrainingCountPriorV1 {
                    positive_count: 1_000,
                    contradiction_count: 10,
                },
                TrainingCountPriorV1 {
                    positive_count: 600,
                    contradiction_count: 10,
                },
                TrainingCountPriorV1 {
                    positive_count: 600,
                    contradiction_count: 20,
                },
                TrainingCountPriorV1 {
                    positive_count: 100,
                    contradiction_count: 10,
                },
            ],
            lemma_counts: BTreeMap::new(),
            paradigm_counts: BTreeMap::new(),
            calibration: fitted_calibration(),
            phase_profiles: Vec::new(),
            directional_residuals: Vec::new(),
        };
        let base_config = ProductivePackageCompilerConfigV1 {
            output_path: root.join("productive-a.p2m"),
            maximum_record_bytes: 4096,
            l11_package_sha256: [1; 32],
            canonical_l2_package_sha256: [2; 32],
            productive_package_byte_budget: 1 << 20,
            steady_rss_kib_budget: 256 * 1024,
            peak_rss_kib_budget: 512 * 1024,
            cold_publish_budget_us: 1_000_000,
            hot_p99_budget_us: 5_000,
        };
        let first =
            compile_productive_package(&reduced, &induction, &axis_schema, &evidence, &base_config)
                .expect("first package");
        let second_config = ProductivePackageCompilerConfigV1 {
            output_path: root.join("productive-b.p2m"),
            ..base_config.clone()
        };
        let second = compile_productive_package(
            &reduced,
            &induction,
            &axis_schema,
            &evidence,
            &second_config,
        )
        .expect("second package");

        assert_eq!(first.package_sha256, second.package_sha256);
        assert_eq!(first.package_bytes, second.package_bytes);
        assert_eq!(
            fs::read(&first.path).expect("first bytes"),
            fs::read(&second.path).expect("second bytes")
        );
        assert_eq!(first.paradigm_count, 2);
        assert_eq!(first.binding_count, 301);
        assert!(first.terminal_count >= 3);
        let first_recovery = first
            .anchor_recovery
            .as_ref()
            .expect("first anchor recovery sidecar");
        let second_recovery = second
            .anchor_recovery
            .as_ref()
            .expect("second anchor recovery sidecar");
        assert_eq!(
            first_recovery.package_sha256,
            second_recovery.package_sha256
        );
        assert_eq!(first_recovery.package_bytes, second_recovery.package_bytes);
        assert_eq!(
            fs::read(&first_recovery.path).expect("first recovery bytes"),
            fs::read(&second_recovery.path).expect("second recovery bytes")
        );
        assert!(first_recovery.program_count > 0);

        let view = ProductivePackageViewV1::load(&first.path).expect("deep mmap reopen");
        assert!(view.mmap_backed());
        assert_eq!(
            view.header.mode,
            ProductiveAlgorithmModeV1::ProductiveV1Model
        );
        let terminals = decode_records::<ProductiveTerminalRecordV1>(
            view.section(ProductiveSectionKindV1::Terminals),
        )
        .expect("terminal records");
        assert!(terminals.iter().all(|terminal| {
            terminal.flags == PRODUCTIVE_TERMINAL_FLAG_SURFACE_FROM_TRIE
                && terminal.decoder_ref == 0
        }));
        let operations = decode_records::<MorphOpRecordV1>(
            view.section(ProductiveSectionKindV1::MorphOperations),
        )
        .expect("operation records");
        assert!(operations.iter().any(|operation| {
            operation.decoded_opcode() == Ok(MorphOpcodeV1::EmitExactAllomorph)
                && operation.arg1 != 0
        }));
        assert!(operations.iter().any(|operation| {
            operation.decoded_opcode() == Ok(MorphOpcodeV1::ReplaceSourceRange)
                && operation.arg1 == 1
                && operation.arg2 == 0
        }));

        let mut ordered_lemma_names = regular_lemmas
            .iter()
            .map(|(lemma, _, _)| lemma.clone())
            .chain([irregular_lemma.clone()])
            .collect::<Vec<_>>();
        ordered_lemma_names.sort();
        let (runtime_lemma, runtime_source, runtime_target) = &regular_lemmas[0];
        let runtime_lemma_id = ordered_lemma_names
            .binary_search(runtime_lemma)
            .expect("runtime lemma identity") as u32
            + 1;
        let runtime_binding = (0..view.record_count(ProductiveSectionKindV1::LemmaBindings))
            .map(|index| {
                view.record::<LemmaParadigmBindingV1>(ProductiveSectionKindV1::LemmaBindings, index)
                    .expect("runtime binding row")
            })
            .find(|binding| binding.lemma_id == runtime_lemma_id)
            .expect("runtime lemma binding");
        let source_slot_id = (0..view.record_count(ProductiveSectionKindV1::SlotKeys))
            .find(|index| {
                view.record::<MorphologySlotKeyV1>(ProductiveSectionKindV1::SlotKeys, *index)
                    .expect("runtime slot row")
                    == slot(2)
            })
            .expect("runtime source slot") as u32
            + 1;
        let target_slot_id = (0..view.record_count(ProductiveSectionKindV1::SlotKeys))
            .find(|index| {
                view.record::<MorphologySlotKeyV1>(ProductiveSectionKindV1::SlotKeys, *index)
                    .expect("runtime slot row")
                    == slot(3)
            })
            .expect("runtime target slot") as u32
            + 1;
        let runtime = PackagedProductiveRuntimeV1::load(&first.path, [1; 32], [2; 32])
            .expect("trained mmap runtime");
        assert!(runtime.mmap_backed());
        assert_eq!(runtime.package_bytes(), first.package_bytes as usize);
        assert_eq!(
            runtime.anchor_recovery_package_bytes(),
            first_recovery.package_bytes as usize
        );
        assert_eq!(
            runtime.anchor_recovery_path_count(),
            first_recovery.posting_count as usize
        );
        let expected_cache_bytes = 124
            + first.paradigm_count as usize * std::mem::size_of::<ParadigmCenterRecordV1>()
            + first.program_count as usize * std::mem::size_of::<PreparedMorphProgramV1>()
            + first.operation_count as usize * std::mem::size_of::<MorphOpRecordV1>()
            + first.terminal_count as usize * std::mem::size_of::<ProductiveTerminalRecordV1>()
            + view.record_count(ProductiveSectionKindV1::SlotPhaseProfiles)
                * std::mem::size_of::<SlotPhaseProfileRecordV1>()
            + first.program_count as usize * std::mem::size_of::<u32>()
            + first_recovery.index_count as usize
                * std::mem::size_of::<ParadigmCompatibilityIndexRecordV1>()
            + first_recovery.posting_count as usize
                * std::mem::size_of::<AnchorRecoveryPostingRecordV1>()
            + first_recovery.program_count as usize
                * std::mem::size_of::<PreparedAnchorRecoveryProgramV1>()
            + first_recovery.operation_count as usize * std::mem::size_of::<MorphOpRecordV1>();
        assert_eq!(runtime.resident_cache_bytes(), expected_cache_bytes);
        assert!(PackagedProductiveRuntimeV1::from_bytes(
            fs::read(&first.path).expect("runtime owned package"),
            [9; 32],
            [2; 32],
        )
        .is_err());
        let scene = L2LocalSceneV1 {
            current_token: runtime_target.clone(),
            current_normalized_scalars: runtime_target.chars().map(u32::from).collect(),
            ..L2LocalSceneV1::default()
        };
        let grounded = vec![PackagedGroundedLemmaV1 {
            lemma_id: runtime_lemma_id,
            pos_domain: 2,
            canonical_source_form_ref: runtime_binding.canonical_source_form_ref,
            source_slot_id,
            normalized_source: runtime_source.clone(),
            grounded_support: 300,
        }];
        let readout = runtime.evaluate_shadow(runtime_target, &scene, &grounded, false);
        assert_eq!(readout.integrity_error, None);
        assert_eq!(readout.logical_terminal_count, 2);
        assert!(readout
            .candidates
            .iter()
            .any(|candidate| candidate.normalized_surface.as_ref() == runtime_source));
        assert!(readout.candidates.iter().any(|candidate| {
            candidate.normalized_surface.as_ref() == runtime_target
                && candidate.identity.lemma_id == runtime_lemma_id
                && candidate.identity.normalized_surface_id != 0
        }));
        assert!(readout
            .candidates
            .iter()
            .all(|candidate| candidate.normalized_surface.as_ref() != "irregularzz"));

        let cold_lemma_id = u32::MAX - 1;
        let cold_bindings = runtime
            .derive_cold_lemma_bindings(
                cold_lemma_id,
                &[
                    ColdLemmaSourceV1 {
                        pos_domain: 2,
                        canonical_source_form_ref: 777,
                        source_slot_id,
                        normalized_source: "newabc".to_string(),
                        grounded_support: 7,
                        canonical_preference: 0,
                        canonical_source: true,
                    },
                    ColdLemmaSourceV1 {
                        pos_domain: 2,
                        canonical_source_form_ref: 778,
                        source_slot_id: target_slot_id,
                        normalized_source: "newac".to_string(),
                        grounded_support: 5,
                        canonical_preference: 1,
                        canonical_source: false,
                    },
                ],
            )
            .expect("cold bindings");
        assert!(!cold_bindings.is_empty());
        let exposed_sources = vec![
            ColdLemmaSourceV1 {
                pos_domain: 2,
                canonical_source_form_ref: 777,
                source_slot_id,
                normalized_source: "newabc".to_string(),
                grounded_support: 7,
                canonical_preference: 0,
                canonical_source: true,
            },
            ColdLemmaSourceV1 {
                pos_domain: 2,
                canonical_source_form_ref: 778,
                source_slot_id: target_slot_id,
                normalized_source: "newac".to_string(),
                grounded_support: 5,
                canonical_preference: 1,
                canonical_source: false,
            },
        ];
        for binding in &cold_bindings {
            runtime
                .verify_exposed_replay_parity(binding, &exposed_sources)
                .expect("exposed replay parity");
        }
        let (recovered_only, recovery_diagnostics) = runtime
            .derive_cold_lemma_bindings_with_diagnostics(
                cold_lemma_id,
                &[ColdLemmaSourceV1 {
                    pos_domain: 2,
                    canonical_source_form_ref: 778,
                    source_slot_id: target_slot_id,
                    normalized_source: "newac".to_string(),
                    grounded_support: 5,
                    canonical_preference: 1,
                    canonical_source: false,
                }],
            )
            .expect("recovery-only cold binding");
        assert!(!recovered_only.is_empty());
        assert!(recovery_diagnostics.recovery_path_count > 0);
        assert!(recovery_diagnostics.recovered_anchor_count > 0);
        assert!(
            recovery_diagnostics.recovery_post_frontier_anchor_count
                <= super::super::runtime::PRODUCTIVE_PHYSICAL_TOP_K
        );
        assert!(
            recovery_diagnostics.recovery_post_frontier_anchor_count
                <= recovery_diagnostics.recovery_unique_anchor_count
        );
        assert!(recovery_diagnostics
            .recovery_exact_paradigm_ids
            .contains(&runtime_binding.paradigm_id));
        assert!(runtime
            .cold_binding_has_slot(&recovered_only[0], source_slot_id)
            .expect("recovered canonical slot"));
        let base_only_runtime = PackagedProductiveRuntimeV1::load_without_anchor_recovery(
            &first.path,
            [1; 32],
            [2; 32],
        )
        .expect("base-only mmap runtime");
        assert_eq!(base_only_runtime.anchor_recovery_package_bytes(), 0);
        assert_eq!(base_only_runtime.anchor_recovery_path_count(), 0);
        assert_eq!(
            runtime.evaluate_shadow(runtime_target, &scene, &grounded, false),
            base_only_runtime.evaluate_shadow(runtime_target, &scene, &grounded, false),
            "prepared direct execution must equal the independent complete-trie oracle"
        );
        assert!(base_only_runtime
            .derive_cold_lemma_bindings(
                cold_lemma_id,
                &[ColdLemmaSourceV1 {
                    pos_domain: 2,
                    canonical_source_form_ref: 778,
                    source_slot_id: target_slot_id,
                    normalized_source: "newac".to_string(),
                    grounded_support: 5,
                    canonical_preference: 1,
                    canonical_source: false,
                }],
            )
            .expect("base-only recovery-disabled binding")
            .is_empty());
        let conflicting_bindings = runtime
            .derive_cold_lemma_bindings(
                cold_lemma_id,
                &[
                    ColdLemmaSourceV1 {
                        pos_domain: 2,
                        canonical_source_form_ref: 777,
                        source_slot_id,
                        normalized_source: "newabc".to_string(),
                        grounded_support: 7,
                        canonical_preference: 0,
                        canonical_source: true,
                    },
                    ColdLemmaSourceV1 {
                        pos_domain: 2,
                        canonical_source_form_ref: 778,
                        source_slot_id: target_slot_id,
                        normalized_source: "newzz".to_string(),
                        grounded_support: 5,
                        canonical_preference: 1,
                        canonical_source: false,
                    },
                ],
            )
            .expect("conflicting cold bindings");
        assert!(conflicting_bindings.is_empty());
        let cold_scene = L2LocalSceneV1 {
            current_token: "newac".to_string(),
            current_normalized_scalars: "newac".chars().map(u32::from).collect(),
            ..L2LocalSceneV1::default()
        };
        for binding in &cold_bindings {
            runtime
                .verify_cold_execution_parity("newac", &cold_scene, binding)
                .expect("direct versus complete-trie cold execution parity");
        }
        let cold_readout = runtime.evaluate_shadow_with_cold_bindings(
            "newac",
            &cold_scene,
            &[],
            &cold_bindings,
            false,
        );
        assert_eq!(cold_readout.integrity_error, None);
        assert!(
            cold_readout.candidates.iter().any(|candidate| {
                candidate.identity.lemma_id == cold_lemma_id
                    && candidate.normalized_surface.as_ref() == "newac"
                    && candidate.provenance == CandidateProvenanceClassV1::ColdLemmaBinding
            }),
            "cold candidates: {:?}; logical={}",
            cold_readout.candidates,
            cold_readout.logical_terminal_count
        );
        let calibration_rows = decode_records::<CalibrationCellRecordV1>(
            view.section(ProductiveSectionKindV1::CalibrationCells),
        )
        .expect("calibration records")
        .len();
        let package_sha256 = first
            .package_sha256
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        println!(
            "productive_compiler_micro package_bytes={} package_sha256={} paradigms={} bindings={} programs={} operations={} trie_nodes={} trie_arcs={} terminals={} calibration_rows={} runtime_logical_terminals={} runtime_cache_bytes={}",
            first.package_bytes,
            package_sha256,
            first.paradigm_count,
            first.binding_count,
            first.program_count,
            first.operation_count,
            first.trie_node_count,
            first.trie_arc_count,
            first.terminal_count,
            calibration_rows,
            readout.logical_terminal_count,
            runtime.resident_cache_bytes(),
        );

        drop(view);
        fs::remove_dir_all(root).expect("cleanup");
    }
}
