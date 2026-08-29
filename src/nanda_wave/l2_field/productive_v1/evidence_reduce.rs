use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use super::calibrate::{
    fit_calibration_table, CalibrationCandidateV1, CalibrationGroupV1, CandidateProvenanceClassV1,
    ObservableCalibrationStratumV1, AMBIGUITY_CROSS_LEMMA_BASIN,
};
use super::compiler::{
    ProductiveCompilerEvidenceV1, TrainedSlotPhaseProfileV1, TrainingCountPriorV1,
};
use super::events::{
    decode_verified_spool_record, deterministic_inner_fold, deterministic_productive_split,
    ContextContradictionEventV1, ContextOccurrenceEventV1, FeedbackEventV1, FeedbackOutcomeV1,
    LemmaSplitKeyV1, ProductiveSplitV1, TypedProductiveEventV1, VerifiedSpoolShardReaderV1,
};
use super::geometry::{
    GeometryPathIdentityV1, GeometryTerminalEvidenceV1, GeometryTraversalStateV1,
    ObservedGeometryV1,
};
use super::packaged_runtime::{
    packaged_ambiguity_kind, PackagedGroundedLemmaV1, PackagedProductiveRuntimeV1,
};
use super::phase::FittedPhaseBankV1;
use super::records::DirectionalResidualRecordV1;
use super::reduce::{ReducedLemmaReaderV1, ReducedMorphologyManifestV1};
use super::scene::{directional_scene_key, L2LocalSceneV1};
use super::score::{
    extract_feature_vector, fit_evidence_model, fixed_point_score_q16, CountEvidenceV1,
    FeatureVectorV1, PairwiseTrainingPairV1, TerminalFeatureInputV1,
};
use super::spool_sort::SortedTypedEventSpoolManifestV1;
use super::transition_reduce::{
    LemmaParadigmAssignmentReaderV1, MorphologyAxisSchemaV1, ParadigmDefinitionReaderV1,
    TransitionInductionManifestV1,
};
use super::types::{
    CanonicalL2BindingIdentityV1, MorphologySlotKeyV1, ProductiveCandidateIdentityV1,
};

#[derive(Clone, Debug)]
pub(super) struct EvidenceReduceConfigV1 {
    pub(super) maximum_record_bytes: usize,
    pub(super) maximum_context_events: usize,
    pub(super) maximum_candidates_per_group: usize,
}

#[derive(Clone, Debug)]
pub(super) struct ProductiveEvidenceReduceManifestV1 {
    pub(super) compiler_evidence: ProductiveCompilerEvidenceV1,
    pub(super) context_occurrence_events: u64,
    pub(super) direct_contradiction_events: u64,
    pub(super) feedback_events: u64,
    pub(super) proof_events: u64,
    pub(super) training_pairs: u32,
    pub(super) calibration_groups: u32,
    pub(super) phase_profiles: u32,
    pub(super) selected_phase_centers: u32,
    pub(super) exact_only_morphology_forms: u64,
    pub(super) exact_only_morphology_events: u64,
    pub(super) excluded_context_occurrence_events: u64,
    pub(super) excluded_direct_contradiction_events: u64,
}

#[derive(Clone, Debug)]
pub(super) struct ProductiveCalibrationReplayManifestV1 {
    pub(super) calibration: super::calibrate::CalibrationTableV1,
    pub(super) source_groups: u32,
    pub(super) fitted_groups: u32,
    pub(super) target_retained_groups: u32,
    pub(super) target_lost_groups: u32,
    pub(super) candidate_rows: u32,
}

#[derive(Clone, Debug)]
struct CandidateMetadataV1 {
    lemma_ref: u32,
    paradigm_id: u32,
    form_ref: u32,
    feature_mask: u32,
    slot_id: u32,
    slot: MorphologySlotKeyV1,
    surface: String,
}

#[derive(Clone, Debug)]
struct ContextGroupV1 {
    split: ProductiveSplitV1,
    lemma: LemmaSplitKeyV1,
    target: CandidateMetadataV1,
    scene: L2LocalSceneV1,
    source_event_identity: [u8; 32],
    support: u32,
    competitors: Vec<CandidateMetadataV1>,
}

#[derive(Clone, Debug, Default)]
struct EvidenceStateV1 {
    lemma_counts: BTreeMap<u32, TrainingCountPriorV1>,
    paradigm_counts: BTreeMap<u32, TrainingCountPriorV1>,
    slot_counts: BTreeMap<(u32, u32), TrainingCountPriorV1>,
    directional_counts: BTreeMap<(u32, u32, u32), TrainingCountPriorV1>,
}

#[derive(Clone, Debug)]
struct FieldMetadataV1 {
    slots: BTreeMap<MorphologySlotKeyV1, u32>,
    assignments: BTreeMap<(u32, u8), u32>,
    forms: BTreeMap<(u32, u32, u32), CandidateMetadataV1>,
    allowed_profiles: BTreeSet<(u32, u32)>,
    paradigm_pos: BTreeMap<u32, u8>,
    exact_only_morphology_forms: u64,
    exact_only_morphology_events: u64,
}

pub(super) fn reduce_productive_evidence(
    sorted: &SortedTypedEventSpoolManifestV1,
    reduced: &ReducedMorphologyManifestV1,
    induction: &TransitionInductionManifestV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &MorphologyAxisSchemaV1,
    config: &EvidenceReduceConfigV1,
) -> Result<ProductiveEvidenceReduceManifestV1, String> {
    validate_config(sorted, reduced, config)?;
    let (metadata, mut state) =
        load_field_metadata(reduced, induction, config.maximum_record_bytes)?;
    let mut groups = BTreeMap::<[u8; 32], ContextGroupV1>::new();
    let mut feedback = Vec::new();
    let mut context_occurrence_events = 0_u64;
    let mut direct_contradiction_events = 0_u64;
    let mut feedback_events = 0_u64;
    let mut proof_events = 0_u64;
    let mut excluded_context_occurrence_events = 0_u64;
    let mut excluded_direct_contradiction_events = 0_u64;
    let mut excluded_context_identities = BTreeSet::new();
    let mut admitted_events = 0_usize;

    let mut reader = VerifiedSpoolShardReaderV1::open(&sorted.shards[0].path)?;
    while let Some(record) = reader.next_record()? {
        admitted_events = admitted_events
            .checked_add(1)
            .ok_or_else(|| "productive evidence event count overflow".to_string())?;
        if admitted_events > config.maximum_context_events {
            return Err("productive evidence reduce exceeds its event budget".to_string());
        }
        let event = decode_verified_spool_record(&record, sorted.split_seed)?;
        match event {
            TypedProductiveEventV1::ContextOccurrence(event) => {
                context_occurrence_events += 1;
                let identity = event_identity(&event.source_event_identity)?;
                if !target_basin_is_assigned(&event, reduced, &metadata)? {
                    excluded_context_occurrence_events += 1;
                    excluded_context_identities.insert(identity);
                    continue;
                }
                let target = resolve_target(&event, reduced, &metadata)?;
                if groups
                    .insert(
                        identity,
                        ContextGroupV1 {
                            split: record.split,
                            lemma: event.lemma,
                            target,
                            scene: event.scene,
                            source_event_identity: identity,
                            support: event.support,
                            competitors: Vec::new(),
                        },
                    )
                    .is_some()
                {
                    return Err("productive context occurrence identity repeats".to_string());
                }
            }
            TypedProductiveEventV1::ContextContradiction(event) => {
                direct_contradiction_events += 1;
                let identity = event_identity(&event.source_event_identity)?;
                if excluded_context_identities.contains(&identity) {
                    excluded_direct_contradiction_events += 1;
                    continue;
                }
                attach_direct_contradiction(
                    &mut groups,
                    event,
                    record.split,
                    reduced,
                    canonical_l2,
                    axis_schema,
                    &metadata,
                    config.maximum_candidates_per_group,
                )?;
            }
            TypedProductiveEventV1::Feedback(event) => {
                feedback_events += 1;
                if record.split == ProductiveSplitV1::Train {
                    feedback.push(event);
                }
            }
            TypedProductiveEventV1::Proof(_) => proof_events += 1,
            TypedProductiveEventV1::Morphology(_) => {
                return Err("productive evidence spool contains morphology input".to_string());
            }
        }
    }

    aggregate_context_counts(&groups, &mut state)?;
    aggregate_feedback_counts(&feedback, &mut state)?;
    let priors = evidence_priors(&state)?;
    let pairs = build_training_pairs(&groups, &state, &priors, sorted.split_seed)?;
    let evidence_model = fit_evidence_model(&pairs).map_err(str::to_string)?;
    let phase_profiles = compile_phase_profiles(&state, &metadata)?;
    let directional_residuals = compile_directional_residuals(&state)?;
    let calibration_groups = build_bootstrap_calibration_groups(
        &groups,
        &state,
        &priors,
        &evidence_model.coefficients_q16,
    )?;
    let calibration = fit_calibration_table(&calibration_groups).map_err(str::to_string)?;
    let selected_phase_centers = phase_profiles
        .iter()
        .map(|profile| {
            profile.positive.centers.len()
                + profile.anti.centers.len()
                + profile.hard_negative.centers.len()
                + profile.ambiguity.centers.len()
        })
        .sum::<usize>();
    Ok(ProductiveEvidenceReduceManifestV1 {
        compiler_evidence: ProductiveCompilerEvidenceV1 {
            evidence_model,
            evidence_priors: priors,
            lemma_counts: state.lemma_counts,
            paradigm_counts: state.paradigm_counts,
            calibration,
            phase_profiles,
            directional_residuals,
        },
        context_occurrence_events,
        direct_contradiction_events,
        feedback_events,
        proof_events,
        training_pairs: u32::try_from(pairs.len())
            .map_err(|_| "productive training pair denominator exceeds u32".to_string())?,
        calibration_groups: u32::try_from(calibration_groups.len())
            .map_err(|_| "productive calibration group denominator exceeds u32".to_string())?,
        phase_profiles: u32::try_from(state.slot_counts.len())
            .map_err(|_| "productive phase profile denominator exceeds u32".to_string())?,
        selected_phase_centers: u32::try_from(selected_phase_centers)
            .map_err(|_| "productive selected phase centers exceed u32".to_string())?,
        exact_only_morphology_forms: metadata.exact_only_morphology_forms,
        exact_only_morphology_events: metadata.exact_only_morphology_events,
        excluded_context_occurrence_events,
        excluded_direct_contradiction_events,
    })
}

pub(super) fn replay_packaged_calibration(
    sorted: &SortedTypedEventSpoolManifestV1,
    reduced: &ReducedMorphologyManifestV1,
    induction: &TransitionInductionManifestV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    packaged_runtime: &PackagedProductiveRuntimeV1,
    axis_schema: &MorphologyAxisSchemaV1,
    config: &EvidenceReduceConfigV1,
) -> Result<ProductiveCalibrationReplayManifestV1, String> {
    validate_config(sorted, reduced, config)?;
    let (metadata, _) = load_field_metadata(reduced, induction, config.maximum_record_bytes)?;
    let mut groups = BTreeMap::<[u8; 32], ContextGroupV1>::new();
    let mut excluded_context_identities = BTreeSet::new();
    let mut admitted_events = 0_usize;
    let mut reader = VerifiedSpoolShardReaderV1::open(&sorted.shards[0].path)?;
    while let Some(record) = reader.next_record()? {
        admitted_events = admitted_events
            .checked_add(1)
            .ok_or_else(|| "productive calibration replay count overflow".to_string())?;
        if admitted_events > config.maximum_context_events {
            return Err("productive calibration replay exceeds its event budget".to_string());
        }
        let event = decode_verified_spool_record(&record, sorted.split_seed)?;
        match event {
            TypedProductiveEventV1::ContextOccurrence(event)
                if record.split == ProductiveSplitV1::Calibration =>
            {
                let identity = event_identity(&event.source_event_identity)?;
                if !target_basin_is_assigned(&event, reduced, &metadata)? {
                    excluded_context_identities.insert(identity);
                    continue;
                }
                let target = resolve_target(&event, reduced, &metadata)?;
                if groups
                    .insert(
                        identity,
                        ContextGroupV1 {
                            split: record.split,
                            lemma: event.lemma,
                            target,
                            scene: event.scene,
                            source_event_identity: identity,
                            support: event.support,
                            competitors: Vec::new(),
                        },
                    )
                    .is_some()
                {
                    return Err("productive calibration occurrence identity repeats".to_string());
                }
            }
            TypedProductiveEventV1::ContextContradiction(event)
                if record.split == ProductiveSplitV1::Calibration =>
            {
                let identity = event_identity(&event.source_event_identity)?;
                if excluded_context_identities.contains(&identity) {
                    continue;
                }
                attach_direct_contradiction(
                    &mut groups,
                    event,
                    record.split,
                    reduced,
                    canonical_l2,
                    axis_schema,
                    &metadata,
                    config.maximum_candidates_per_group,
                )?;
            }
            TypedProductiveEventV1::Morphology(_) => {
                return Err("productive calibration spool contains morphology input".to_string());
            }
            _ => {}
        }
    }

    let source_groups = u32::try_from(groups.len())
        .map_err(|_| "productive calibration source denominator exceeds u32".to_string())?;
    let mut calibration_groups = Vec::new();
    let mut target_retained_groups = 0_u32;
    let mut target_lost_groups = 0_u32;
    let mut candidate_rows = 0_u32;
    for group in groups.values() {
        let mut grounded = BTreeMap::<(u32, u8), PackagedGroundedLemmaV1>::new();
        for candidate in std::iter::once(&group.target).chain(group.competitors.iter()) {
            grounded
                .entry((candidate.lemma_ref, candidate.slot.pos_domain()))
                .or_insert_with(|| PackagedGroundedLemmaV1 {
                    lemma_id: candidate.lemma_ref,
                    pos_domain: u16::from(candidate.slot.pos_domain()),
                    canonical_source_form_ref: candidate.form_ref,
                    source_slot_id: candidate.slot_id,
                    normalized_source: candidate.surface.clone(),
                    grounded_support: group.support.max(1),
                });
        }
        let readout = packaged_runtime.evaluate_shadow(
            &group.scene.current_token,
            &group.scene,
            &grounded.into_values().collect::<Vec<_>>(),
            false,
        );
        if let Some(error) = readout.integrity_error {
            return Err(format!(
                "productive calibration packaged replay failed integrity: {error}"
            ));
        }
        let leader_provenance = readout
            .candidates
            .first()
            .map(|candidate| candidate.provenance)
            .unwrap_or(CandidateProvenanceClassV1::TrainingSeenGenerated);
        let ambiguity_kind =
            packaged_ambiguity_kind(&readout.candidates, readout.logical_surface_basin_count);
        let mut candidates = readout
            .candidates
            .into_iter()
            .map(|candidate| {
                let gold_valid = candidate.identity.lemma_id == group.target.lemma_ref
                    && candidate.identity.target_slot_id == group.target.slot_id
                    && candidate.normalized_surface.as_ref() == group.target.surface;
                CalibrationCandidateV1 {
                    identity: candidate.identity,
                    normalized_surface: candidate.normalized_surface.to_string(),
                    score_q16: candidate.score_q16,
                    grounded_lemma_evidence: candidate.grounded_support,
                    exact_osa_distance: candidate
                        .geometry
                        .character_distance
                        .min(candidate.geometry.keyboard_distance),
                    exact_form: candidate.normalized_surface.as_ref() == group.scene.current_token,
                    gold_valid,
                }
            })
            .collect::<Vec<_>>();
        candidate_rows = candidate_rows
            .checked_add(
                u32::try_from(candidates.len())
                    .map_err(|_| "productive calibration candidate count exceeds u32")?,
            )
            .ok_or_else(|| "productive calibration candidate denominator overflow".to_string())?;
        candidates.sort_by(|left, right| {
            right
                .score_q16
                .cmp(&left.score_q16)
                .then_with(|| left.identity.cmp(&right.identity))
        });
        let target_retained = candidates.iter().any(|candidate| candidate.gold_valid);
        if !target_retained {
            target_lost_groups = target_lost_groups
                .checked_add(1)
                .ok_or_else(|| "productive calibration target-loss overflow".to_string())?;
            continue;
        }
        target_retained_groups = target_retained_groups
            .checked_add(1)
            .ok_or_else(|| "productive calibration retention overflow".to_string())?;
        let false_singleton = candidates.len() == 1 && !candidates[0].gold_valid;
        let wrong_unique_leader = candidates.first().is_some_and(|leader| {
            !leader.gold_valid
                && candidates
                    .get(1)
                    .is_none_or(|second| leader.score_q16 > second.score_q16)
        });
        let leader = candidates.first().ok_or_else(|| {
            "productive calibration retained target without candidates".to_string()
        })?;
        let stratum = ObservableCalibrationStratumV1::new(
            &group.scene.current_token,
            &leader.normalized_surface,
            leader_provenance,
            leader.grounded_lemma_evidence,
            ambiguity_kind,
        )
        .map_err(str::to_string)?;
        calibration_groups.push(CalibrationGroupV1 {
            split: ProductiveSplitV1::Calibration,
            group_identity: group.source_event_identity,
            stratum,
            candidates,
            false_singleton,
            grounded_winner_protection_violation: wrong_unique_leader,
        });
    }
    if target_lost_groups != 0 {
        for group in &mut calibration_groups {
            group.grounded_winner_protection_violation = true;
        }
    }
    if calibration_groups.is_empty() {
        return Err("productive packaged calibration retained no target groups".to_string());
    }
    let calibration = fit_calibration_table(&calibration_groups).map_err(str::to_string)?;
    Ok(ProductiveCalibrationReplayManifestV1 {
        calibration,
        source_groups,
        fitted_groups: u32::try_from(calibration_groups.len())
            .map_err(|_| "productive fitted calibration denominator exceeds u32".to_string())?,
        target_retained_groups,
        target_lost_groups,
        candidate_rows,
    })
}

fn validate_config(
    sorted: &SortedTypedEventSpoolManifestV1,
    reduced: &ReducedMorphologyManifestV1,
    config: &EvidenceReduceConfigV1,
) -> Result<(), String> {
    if sorted.shards.len() != 1
        || config.maximum_record_bytes == 0
        || config.maximum_context_events == 0
        || config.maximum_candidates_per_group == 0
    {
        return Err("productive evidence reduce has an invalid bounded contract".to_string());
    }
    if !reduced.imported_identity_verified || reduced.imported_lemma_refs.is_empty() {
        return Err("productive evidence reduce requires verified imported ownership".to_string());
    }
    if sorted.split_seed != reduced.split_seed
        || sorted.compiler_version != reduced.compiler_version
        || sorted.normalization_version != reduced.normalization_version
    {
        return Err("productive evidence and morphology manifests disagree".to_string());
    }
    Ok(())
}

fn load_field_metadata(
    reduced: &ReducedMorphologyManifestV1,
    induction: &TransitionInductionManifestV1,
    maximum_record_bytes: usize,
) -> Result<(FieldMetadataV1, EvidenceStateV1), String> {
    let slots = reduced
        .morphology_slots
        .iter()
        .copied()
        .enumerate()
        .map(|(index, slot)| (slot, index as u32 + 1))
        .collect::<BTreeMap<_, _>>();
    let mut assignment_reader = LemmaParadigmAssignmentReaderV1::open(
        &induction.lemma_bindings_path,
        maximum_record_bytes,
    )?;
    let mut assignments = BTreeMap::new();
    while let Some(assignment) = assignment_reader.next()? {
        if assignments
            .insert(
                (assignment.lemma_id, assignment.pos_domain),
                assignment.paradigm_id,
            )
            .is_some()
        {
            return Err("productive evidence repeats a lemma/POS assignment".to_string());
        }
    }
    let mut paradigm_reader =
        ParadigmDefinitionReaderV1::open(&induction.paradigms_path, maximum_record_bytes)?;
    let mut allowed_profiles = BTreeSet::new();
    let mut paradigm_pos = BTreeMap::new();
    while let Some(paradigm) = paradigm_reader.next()? {
        paradigm_pos.insert(paradigm.paradigm_id, paradigm.signature.pos_domain);
        for transition in paradigm.signature.transitions {
            let slot_id = *slots
                .get(&transition.target_slot)
                .ok_or_else(|| "productive evidence target slot has no canonical ID".to_string())?;
            allowed_profiles.insert((paradigm.paradigm_id, slot_id));
        }
    }
    let mut forms = BTreeMap::new();
    let mut state = EvidenceStateV1::default();
    let mut exact_only_morphology_forms = 0_u64;
    let mut exact_only_morphology_events = 0_u64;
    let mut reader = ReducedLemmaReaderV1::open(&reduced.path)?;
    while let Some(lemma) = reader.next_lemma()? {
        for form in lemma.forms {
            let Some(&paradigm_id) = assignments.get(&(lemma.lemma_id, form.slot.pos_domain()))
            else {
                exact_only_morphology_forms = exact_only_morphology_forms
                    .checked_add(1)
                    .ok_or_else(|| "productive exact-only form count overflow".to_string())?;
                exact_only_morphology_events = exact_only_morphology_events
                    .checked_add(u64::from(form.support))
                    .ok_or_else(|| "productive exact-only event count overflow".to_string())?;
                continue;
            };
            let slot_id = *slots
                .get(&form.slot)
                .ok_or_else(|| "productive reduced form has no slot ID".to_string())?;
            add_counts(
                &mut state.lemma_counts,
                lemma.lemma_id,
                u64::from(form.support),
                0,
            )?;
            add_counts(
                &mut state.paradigm_counts,
                paradigm_id,
                u64::from(form.support),
                0,
            )?;
            add_counts(
                &mut state.slot_counts,
                (paradigm_id, slot_id),
                u64::from(form.support),
                0,
            )?;
            let metadata = CandidateMetadataV1 {
                lemma_ref: lemma.lemma_id,
                paradigm_id,
                form_ref: form.form_ref,
                feature_mask: 0,
                slot_id,
                slot: form.slot,
                surface: form.normalized_surface,
            };
            forms.insert(
                (metadata.lemma_ref, metadata.form_ref, metadata.feature_mask),
                metadata,
            );
        }
    }
    Ok((
        FieldMetadataV1 {
            slots,
            assignments,
            forms,
            allowed_profiles,
            paradigm_pos,
            exact_only_morphology_forms,
            exact_only_morphology_events,
        },
        state,
    ))
}

fn target_basin_is_assigned(
    event: &ContextOccurrenceEventV1,
    reduced: &ReducedMorphologyManifestV1,
    metadata: &FieldMetadataV1,
) -> Result<bool, String> {
    let lemma_ref = *reduced
        .imported_lemma_refs
        .get(&(
            event.lemma.language.clone(),
            event.lemma.normalized_lemma.clone(),
        ))
        .ok_or_else(|| "productive context target has no imported lemma ref".to_string())?;
    if metadata
        .assignments
        .contains_key(&(lemma_ref, event.slot.pos_domain()))
    {
        return Ok(true);
    }
    if deterministic_productive_split(&event.lemma, reduced.split_seed) == ProductiveSplitV1::Train
    {
        return Ok(false);
    }
    let Some(&slot_id) = metadata.slots.get(&event.slot) else {
        return Ok(false);
    };
    Ok(metadata
        .allowed_profiles
        .iter()
        .any(|(paradigm_id, target_slot_id)| {
            *target_slot_id == slot_id
                && metadata.paradigm_pos.get(paradigm_id) == Some(&event.slot.pos_domain())
        }))
}

fn resolve_target(
    event: &ContextOccurrenceEventV1,
    reduced: &ReducedMorphologyManifestV1,
    metadata: &FieldMetadataV1,
) -> Result<CandidateMetadataV1, String> {
    let lemma_ref = *reduced
        .imported_lemma_refs
        .get(&(
            event.lemma.language.clone(),
            event.lemma.normalized_lemma.clone(),
        ))
        .ok_or_else(|| "productive context target has no imported lemma ref".to_string())?;
    resolve_candidate(
        CanonicalL2BindingIdentityV1 {
            lemma_ref: super::types::ImportedCanonicalL2LemmaRefV1(lemma_ref),
            form_ref: event.canonical_form_ref,
            legacy_feature_mask: event.canonical_feature_mask,
        },
        Some((&event.normalized_surface, event.slot)),
        metadata,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn attach_direct_contradiction(
    groups: &mut BTreeMap<[u8; 32], ContextGroupV1>,
    event: ContextContradictionEventV1,
    split: ProductiveSplitV1,
    reduced: &ReducedMorphologyManifestV1,
    canonical_l2: &super::super::runtime::StandaloneL2Field,
    axis_schema: &MorphologyAxisSchemaV1,
    metadata: &FieldMetadataV1,
    maximum_candidates_per_group: usize,
) -> Result<(), String> {
    let identity = event_identity(&event.source_event_identity)?;
    let group = groups
        .get_mut(&identity)
        .ok_or_else(|| "productive contradiction has no positive occurrence".to_string())?;
    if group.split != split
        || group.lemma != event.lemma
        || group.target.form_ref != event.canonical_form_ref.0
        || group.target.feature_mask != event.canonical_feature_mask
        || group.scene != event.scene
    {
        return Err("productive contradiction disagrees with its positive event".to_string());
    }
    for competitor in event.competitors {
        let Some((lemma_key, _)) = reduced
            .imported_lemma_refs
            .iter()
            .find(|(_, lemma_ref)| **lemma_ref == competitor.lemma_ref.0)
        else {
            return Err("productive competitor has no imported lemma key".to_string());
        };
        if split == ProductiveSplitV1::Train
            && super::events::deterministic_productive_split(
                &LemmaSplitKeyV1 {
                    language: lemma_key.0.clone(),
                    normalized_lemma: lemma_key.1.clone(),
                },
                reduced.split_seed,
            ) != ProductiveSplitV1::Train
        {
            continue;
        }
        let surface = canonical_l2
            .imported_surface_for_form(competitor.form_ref.0)
            .ok_or_else(|| "productive competitor surface disappeared".to_string())?;
        let labels = crate::nanda_wave::morphology_phase::canonical_feature_labels(
            competitor.legacy_feature_mask,
        )?;
        let slot = axis_schema.parse_feature_labels(&labels.join(":"))?;
        if let Ok(candidate) = resolve_candidate(competitor, Some((&surface, slot)), metadata) {
            group.competitors.push(candidate);
        }
    }
    group.competitors.sort_by(|left, right| {
        (left.lemma_ref, left.form_ref, left.feature_mask).cmp(&(
            right.lemma_ref,
            right.form_ref,
            right.feature_mask,
        ))
    });
    group.competitors.dedup_by(|left, right| {
        (left.lemma_ref, left.form_ref, left.feature_mask)
            == (right.lemma_ref, right.form_ref, right.feature_mask)
    });
    if group.competitors.len() > maximum_candidates_per_group {
        return Err("productive contradiction exceeds its candidate bound".to_string());
    }
    Ok(())
}

fn resolve_candidate(
    identity: CanonicalL2BindingIdentityV1,
    fallback: Option<(&str, MorphologySlotKeyV1)>,
    metadata: &FieldMetadataV1,
) -> Result<CandidateMetadataV1, String> {
    if let Some(candidate) = metadata.forms.get(&(
        identity.lemma_ref.0,
        identity.form_ref.0,
        identity.legacy_feature_mask,
    )) {
        return Ok(candidate.clone());
    }
    let (surface, slot) =
        fallback.ok_or_else(|| "productive candidate fallback is absent".to_string())?;
    let slot_id = *metadata
        .slots
        .get(&slot)
        .ok_or_else(|| "productive candidate slot is absent from TRAIN".to_string())?;
    let paradigm_id = metadata
        .assignments
        .get(&(identity.lemma_ref.0, slot.pos_domain()))
        .copied()
        .or_else(|| {
            metadata
                .allowed_profiles
                .iter()
                .find(|(paradigm_id, target_slot_id)| {
                    *target_slot_id == slot_id
                        && metadata.paradigm_pos.get(paradigm_id) == Some(&slot.pos_domain())
                })
                .map(|(paradigm_id, _)| *paradigm_id)
        })
        .ok_or_else(|| {
            "productive candidate has no trained or cold-compatible paradigm".to_string()
        })?;
    Ok(CandidateMetadataV1 {
        lemma_ref: identity.lemma_ref.0,
        paradigm_id,
        form_ref: identity.form_ref.0,
        feature_mask: identity.legacy_feature_mask,
        slot_id,
        slot,
        surface: surface.to_string(),
    })
}

fn aggregate_context_counts(
    groups: &BTreeMap<[u8; 32], ContextGroupV1>,
    state: &mut EvidenceStateV1,
) -> Result<(), String> {
    for group in groups
        .values()
        .filter(|group| group.split == ProductiveSplitV1::Train)
    {
        add_candidate_counts(state, &group.target, u64::from(group.support), 0)?;
        let scene_key = directional_scene_key(&group.scene).map_err(str::to_string)?;
        for competitor in &group.competitors {
            add_candidate_counts(state, competitor, 0, u64::from(group.support))?;
            add_counts(
                &mut state.directional_counts,
                (scene_key, competitor.slot_id, group.target.slot_id),
                u64::from(group.support),
                0,
            )?;
            add_counts(
                &mut state.directional_counts,
                (scene_key, group.target.slot_id, competitor.slot_id),
                0,
                u64::from(group.support),
            )?;
        }
    }
    Ok(())
}

fn aggregate_feedback_counts(
    feedback: &[FeedbackEventV1],
    state: &mut EvidenceStateV1,
) -> Result<(), String> {
    for event in feedback {
        let positive = u64::from(event.outcome == FeedbackOutcomeV1::Accept);
        let contradiction = u64::from(event.outcome.is_explicit_anti());
        if positive == 0 && contradiction == 0 {
            continue;
        }
        add_counts(
            &mut state.lemma_counts,
            event.proposed_form.lemma_id,
            positive,
            contradiction,
        )?;
        add_counts(
            &mut state.paradigm_counts,
            event.proposed_form.paradigm_id,
            positive,
            contradiction,
        )?;
        add_counts(
            &mut state.slot_counts,
            (
                event.proposed_form.paradigm_id,
                event.proposed_form.target_slot_id,
            ),
            positive,
            contradiction,
        )?;
    }
    Ok(())
}

fn add_candidate_counts(
    state: &mut EvidenceStateV1,
    candidate: &CandidateMetadataV1,
    positive: u64,
    contradiction: u64,
) -> Result<(), String> {
    add_counts(
        &mut state.lemma_counts,
        candidate.lemma_ref,
        positive,
        contradiction,
    )?;
    add_counts(
        &mut state.paradigm_counts,
        candidate.paradigm_id,
        positive,
        contradiction,
    )?;
    add_counts(
        &mut state.slot_counts,
        (candidate.paradigm_id, candidate.slot_id),
        positive,
        contradiction,
    )
}

fn add_counts<K: Ord>(
    counts: &mut BTreeMap<K, TrainingCountPriorV1>,
    key: K,
    positive: u64,
    contradiction: u64,
) -> Result<(), String> {
    let entry = counts.entry(key).or_default();
    entry.positive_count = entry
        .positive_count
        .checked_add(positive)
        .ok_or_else(|| "productive positive evidence count overflow".to_string())?;
    entry.contradiction_count = entry
        .contradiction_count
        .checked_add(contradiction)
        .ok_or_else(|| "productive contradiction evidence count overflow".to_string())?;
    Ok(())
}

fn evidence_priors(state: &EvidenceStateV1) -> Result<[TrainingCountPriorV1; 4], String> {
    Ok([
        total_counts(state.lemma_counts.values())?,
        total_counts(state.paradigm_counts.values())?,
        total_counts(state.slot_counts.values())?,
        total_counts(state.directional_counts.values())?,
    ])
}

fn total_counts<'a>(
    mut counts: impl Iterator<Item = &'a TrainingCountPriorV1>,
) -> Result<TrainingCountPriorV1, String> {
    counts.try_fold(TrainingCountPriorV1::default(), |mut total, counts| {
        total.positive_count = total
            .positive_count
            .checked_add(counts.positive_count)
            .ok_or_else(|| "productive positive prior overflow".to_string())?;
        total.contradiction_count = total
            .contradiction_count
            .checked_add(counts.contradiction_count)
            .ok_or_else(|| "productive contradiction prior overflow".to_string())?;
        Ok(total)
    })
}

fn build_training_pairs(
    groups: &BTreeMap<[u8; 32], ContextGroupV1>,
    state: &EvidenceStateV1,
    priors: &[TrainingCountPriorV1; 4],
    split_seed: u64,
) -> Result<Vec<PairwiseTrainingPairV1>, String> {
    let mut pairs = Vec::new();
    for group in groups
        .values()
        .filter(|group| group.split == ProductiveSplitV1::Train)
    {
        if group.competitors.is_empty() {
            continue;
        }
        let scene_key = directional_scene_key(&group.scene).map_err(str::to_string)?;
        let valid = feature_vector(
            &group.scene.current_token,
            &group.target,
            Some((
                scene_key,
                group.competitors[0].slot_id,
                group.target.slot_id,
            )),
            state,
            priors,
        )?;
        for competitor in &group.competitors {
            let contradicted = feature_vector(
                &group.scene.current_token,
                competitor,
                Some((scene_key, group.target.slot_id, competitor.slot_id)),
                state,
                priors,
            )?;
            let mut identity = Sha256::new();
            identity.update(b"lay-productive-pair-v1\0");
            identity.update(group.source_event_identity);
            identity.update(competitor.lemma_ref.to_le_bytes());
            identity.update(competitor.form_ref.to_le_bytes());
            identity.update(competitor.feature_mask.to_le_bytes());
            pairs.push(PairwiseTrainingPairV1 {
                group_identity: group.source_event_identity,
                stable_event_identity: identity.finalize().into(),
                inner_fold: deterministic_inner_fold(&group.lemma, split_seed),
                valid: valid.clone(),
                contradicted,
            });
        }
    }
    Ok(pairs)
}

fn feature_vector(
    observed_surface: &str,
    candidate: &CandidateMetadataV1,
    directional_key: Option<(u32, u32, u32)>,
    state: &EvidenceStateV1,
    priors: &[TrainingCountPriorV1; 4],
) -> Result<FeatureVectorV1, String> {
    let lemma = state
        .lemma_counts
        .get(&candidate.lemma_ref)
        .map(|counts| count_evidence(*counts, priors[0]));
    let paradigm = state
        .paradigm_counts
        .get(&candidate.paradigm_id)
        .map(|counts| count_evidence(*counts, priors[1]));
    let slot_counts = state
        .slot_counts
        .get(&(candidate.paradigm_id, candidate.slot_id))
        .copied();
    let slot = slot_counts.map(|counts| count_evidence(counts, priors[2]));
    let directional = directional_key
        .and_then(|key| state.directional_counts.get(&key).copied())
        .map(|counts| count_evidence(counts, priors[3]));
    extract_feature_vector(TerminalFeatureInputV1 {
        lemma,
        paradigm,
        slot,
        directional,
        geometry: geometry(observed_surface, &candidate.surface)?,
        support: slot_counts
            .and_then(|counts| u32::try_from(counts.positive_count).ok())
            .filter(|support| *support != 0),
        ..TerminalFeatureInputV1::default()
    })
    .map_err(str::to_string)
}

fn count_evidence(counts: TrainingCountPriorV1, prior: TrainingCountPriorV1) -> CountEvidenceV1 {
    CountEvidenceV1 {
        positive: u32::try_from(counts.positive_count).unwrap_or(u32::MAX),
        contradiction: u32::try_from(counts.contradiction_count).unwrap_or(u32::MAX),
        train_positive_prior: prior.positive_count as f64 + 0.5,
        train_contradiction_prior: prior.contradiction_count as f64 + 0.5,
    }
}

fn geometry(observed: &str, candidate: &str) -> Result<GeometryTerminalEvidenceV1, String> {
    let observed = ObservedGeometryV1::new(observed).map_err(str::to_string)?;
    let mut state = GeometryTraversalStateV1::new(&observed, GeometryPathIdentityV1::default())
        .map_err(str::to_string)?;
    state
        .emit_normalized_str(candidate)
        .map_err(str::to_string)?;
    Ok(state.terminal_evidence())
}

fn compile_phase_profiles(
    state: &EvidenceStateV1,
    metadata: &FieldMetadataV1,
) -> Result<Vec<TrainedSlotPhaseProfileV1>, String> {
    state
        .slot_counts
        .iter()
        .filter(|(profile, counts)| {
            metadata.allowed_profiles.contains(profile) && counts.positive_count != 0
        })
        .map(|(&(paradigm_id, slot_id), counts)| {
            Ok(TrainedSlotPhaseProfileV1 {
                paradigm_id,
                slot_id,
                support: u32::try_from(counts.positive_count)
                    .map_err(|_| "productive slot support exceeds u32")?,
                explicit_anti_support: u32::try_from(counts.contradiction_count)
                    .map_err(|_| "productive slot anti support exceeds u32")?,
                positive: FittedPhaseBankV1::default(),
                anti: FittedPhaseBankV1::default(),
                hard_negative: FittedPhaseBankV1::default(),
                ambiguity: FittedPhaseBankV1::default(),
            })
        })
        .collect()
}

fn compile_directional_residuals(
    state: &EvidenceStateV1,
) -> Result<Vec<DirectionalResidualRecordV1>, String> {
    state
        .directional_counts
        .iter()
        .map(|(&(source_scene_key, from_slot_id, to_slot_id), counts)| {
            Ok(DirectionalResidualRecordV1 {
                source_scene_key,
                from_slot_id,
                to_slot_id,
                positive_support: u32::try_from(counts.positive_count)
                    .map_err(|_| "productive directional positive support exceeds u32")?,
                explicit_anti_support: u32::try_from(counts.contradiction_count)
                    .map_err(|_| "productive directional anti support exceeds u32")?,
                flags: 0,
            })
        })
        .collect()
}

fn build_bootstrap_calibration_groups(
    groups: &BTreeMap<[u8; 32], ContextGroupV1>,
    state: &EvidenceStateV1,
    priors: &[TrainingCountPriorV1; 4],
    coefficients_q16: &[i32; super::score::PRODUCTIVE_FEATURE_COUNT],
) -> Result<Vec<CalibrationGroupV1>, String> {
    let mut calibration = Vec::new();
    for group in groups
        .values()
        .filter(|group| group.split == ProductiveSplitV1::Calibration)
    {
        let mut candidates = Vec::new();
        for (candidate, gold_valid) in std::iter::once((&group.target, true))
            .chain(group.competitors.iter().map(|candidate| (candidate, false)))
        {
            let features =
                feature_vector(&group.scene.current_token, candidate, None, state, priors)?;
            let geometry = geometry(&group.scene.current_token, &candidate.surface)?;
            candidates.push(CalibrationCandidateV1 {
                identity: ProductiveCandidateIdentityV1 {
                    lemma_id: candidate.lemma_ref,
                    paradigm_id: candidate.paradigm_id,
                    program_id: candidate.slot_id,
                    target_slot_id: candidate.slot_id,
                    normalized_surface_id: candidate
                        .form_ref
                        .checked_add(1)
                        .ok_or_else(|| "productive calibration surface ID overflow".to_string())?,
                    variant_id: 1,
                },
                normalized_surface: candidate.surface.clone(),
                score_q16: fixed_point_score_q16(coefficients_q16, features.quantize()?)
                    .map_err(str::to_string)?,
                grounded_lemma_evidence: state
                    .lemma_counts
                    .get(&candidate.lemma_ref)
                    .and_then(|counts| u32::try_from(counts.positive_count).ok())
                    .unwrap_or_default(),
                exact_osa_distance: geometry.character_distance.min(geometry.keyboard_distance),
                exact_form: false,
                gold_valid,
            });
        }
        candidates.sort_by_key(|candidate| candidate.identity);
        candidates.dedup_by_key(|candidate| candidate.identity);
        if candidates.is_empty() || !candidates.iter().any(|candidate| candidate.gold_valid) {
            continue;
        }
        let target_support = state
            .slot_counts
            .get(&(group.target.paradigm_id, group.target.slot_id))
            .and_then(|counts| u32::try_from(counts.positive_count).ok())
            .unwrap_or_default();
        let stratum = ObservableCalibrationStratumV1::new(
            &group.scene.current_token,
            &group.target.surface,
            CandidateProvenanceClassV1::TrainingSeenGenerated,
            target_support,
            u8::from(!group.competitors.is_empty()) * AMBIGUITY_CROSS_LEMMA_BASIN,
        )
        .map_err(str::to_string)?;
        calibration.push(CalibrationGroupV1 {
            split: ProductiveSplitV1::Calibration,
            group_identity: group.source_event_identity,
            stratum,
            candidates,
            false_singleton: false,
            grounded_winner_protection_violation: true,
        });
    }
    if calibration.is_empty() {
        return Err("productive bootstrap calibration has no disjoint groups".to_string());
    }
    Ok(calibration)
}

fn event_identity(bytes: &[u8]) -> Result<[u8; 32], String> {
    bytes
        .try_into()
        .map_err(|_| "productive source event identity is not SHA-256 width".to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::compiler::{compile_productive_package, ProductivePackageCompilerConfigV1};
    use super::super::events::{
        ContextOccurrenceEventV1, MorphologyEventV1, ProofEventV1, TypedEventSpoolConfigV1,
        TypedEventSpoolWriterV1,
    };
    use super::super::packaged_runtime::PackagedProductiveRuntimeV1;
    use super::super::reduce::{
        reduce_train_morphology_with_imported_ownership, TrainMorphologyReduceConfigV1,
    };
    use super::super::scene::BoundaryKindV1;
    use super::super::spool_sort::{external_sort_verified_spool, ExternalSpoolSortConfigV1};
    use super::super::transition_reduce::{induce_transition_field, TransitionReduceConfigV1};
    use super::super::types::{ImportedCanonicalL2FormRefV1, ImportedCanonicalL2LemmaRefV1};
    use super::*;

    #[derive(Clone, Debug)]
    struct FixtureLemmaV1 {
        lemma: String,
        source: String,
        target: String,
    }

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lay-productive-evidence-reduce-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ))
    }

    fn fixture_scene(observed: &str) -> L2LocalSceneV1 {
        L2LocalSceneV1 {
            current_token: observed.to_string(),
            current_normalized_scalars: observed.chars().map(u32::from).collect(),
            boundary_before: BoundaryKindV1::Token,
            boundary_after: BoundaryKindV1::Token,
            ..L2LocalSceneV1::default()
        }
    }

    fn binding_for_surface(
        field: &super::super::super::runtime::StandaloneL2Field,
        surface: &str,
    ) -> CanonicalL2BindingIdentityV1 {
        let form_ref = field
            .form_ref_for_surface(surface)
            .expect("fixture form ref");
        let bindings = field.imported_binding_identities_for_form(form_ref);
        assert_eq!(bindings.len(), 1, "fixture surfaces are lemma-unique");
        CanonicalL2BindingIdentityV1 {
            lemma_ref: ImportedCanonicalL2LemmaRefV1(bindings[0].0),
            form_ref: ImportedCanonicalL2FormRefV1(form_ref),
            legacy_feature_mask: bindings[0].1,
        }
    }

    fn append_context_pair(
        writer: &mut TypedEventSpoolWriterV1,
        field: &super::super::super::runtime::StandaloneL2Field,
        target: &FixtureLemmaV1,
        competitor: &FixtureLemmaV1,
        slot: MorphologySlotKeyV1,
        ordinal: usize,
    ) {
        let target_binding = binding_for_surface(field, &target.source);
        let competitor_binding = binding_for_surface(field, &competitor.source);
        let identity: [u8; 32] = Sha256::digest(
            format!("productive-evidence-group:{ordinal}:{}", target.lemma).as_bytes(),
        )
        .into();
        let scene = fixture_scene(&target.source);
        writer
            .append(&TypedProductiveEventV1::ContextOccurrence(
                ContextOccurrenceEventV1 {
                    lemma: LemmaSplitKeyV1 {
                        language: "ru".to_string(),
                        normalized_lemma: target.lemma.clone(),
                    },
                    normalized_surface: target.source.clone(),
                    canonical_form_ref: target_binding.form_ref,
                    canonical_feature_mask: target_binding.legacy_feature_mask,
                    slot,
                    scene: scene.clone(),
                    source_event_identity: identity.to_vec(),
                    support: 1,
                    provenance: format!("fixture:T:{ordinal}").into_bytes(),
                },
            ))
            .expect("context occurrence");
        writer
            .append(&TypedProductiveEventV1::ContextContradiction(
                ContextContradictionEventV1 {
                    lemma: LemmaSplitKeyV1 {
                        language: "ru".to_string(),
                        normalized_lemma: target.lemma.clone(),
                    },
                    normalized_surface: target.source.clone(),
                    canonical_form_ref: target_binding.form_ref,
                    canonical_feature_mask: target_binding.legacy_feature_mask,
                    slot,
                    scene,
                    competitors: vec![competitor_binding],
                    source_event_identity: identity.to_vec(),
                    support: 1,
                    provenance: format!("fixture:NT:{ordinal}").into_bytes(),
                },
            ))
            .expect("context contradiction");
    }

    #[test]
    fn evidence_reduce_fits_disjoint_evidence_and_reopens_the_compiled_package() {
        let root = temp_root();
        let split_seed = 17;
        let mut axis_schema = super::super::corpus::load_axis_schema(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("data/morphology/productive_v1_ru_axis_schema.json"),
        )
        .expect("axis schema");
        axis_schema.pos_applicability.retain(|pos, _| *pos == 2);
        axis_schema
            .labels
            .retain(|label| matches!(label.label.as_str(), "noun" | "sg" | "nom" | "gen"));
        let source_slot = axis_schema
            .parse_feature_labels("noun:nom:sg")
            .expect("source slot");
        let target_slot = axis_schema
            .parse_feature_labels("noun:gen:sg")
            .expect("target slot");
        let lemmas = (0..240)
            .map(|index| FixtureLemmaV1 {
                lemma: format!("lemma-{index:04}"),
                source: format!("stem{index:04}a"),
                target: format!("stem{index:04}y"),
            })
            .collect::<Vec<_>>();

        let mut corpus_tsv = String::new();
        let mut terminals = BTreeMap::new();
        for (index, lemma) in lemmas.iter().enumerate() {
            corpus_tsv.push_str(&format!(
                "F\t{}\t{}\tnoun:nom:sg\nF\t{}\t{}\tnoun:gen:sg\n",
                lemma.lemma, lemma.source, lemma.lemma, lemma.target
            ));
            terminals.insert(lemma.source.clone(), (index * 2 + 1) as u32);
            terminals.insert(lemma.target.clone(), (index * 2 + 2) as u32);
        }
        corpus_tsv.push_str(
            "T\tlemma-0000\tstem0000a\tnoun:nom:sg\t_ stem0001a\n\
             H\tlemma-0000\tstem0000y\tnoun:gen:sg\tstem0001a _\n",
        );
        let corpus = super::super::super::teacher::L2TeacherCorpus::parse_tsv(&corpus_tsv)
            .expect("canonical fixture corpus");
        let (canonical_package, _) =
            super::super::super::compiler::compile_l2_package(&corpus, 99, |surface| {
                terminals.get(surface).copied()
            })
            .expect("canonical fixture package");
        let canonical_l2 =
            super::super::super::runtime::StandaloneL2Field::from_package(canonical_package)
                .expect("canonical fixture field");

        let mut morphology_writer = TypedEventSpoolWriterV1::create(TypedEventSpoolConfigV1 {
            root: root.join("morphology-raw"),
            shard_count: 4,
            split_seed,
            compiler_version: 1,
            normalization_version: 1,
            write_buffer_bytes: 4096,
        })
        .expect("morphology spool");
        for lemma in &lemmas {
            for (surface, features, slot) in [
                (&lemma.source, "noun:nom:sg", source_slot),
                (&lemma.target, "noun:gen:sg", target_slot),
            ] {
                let binding = binding_for_surface(&canonical_l2, surface);
                morphology_writer
                    .append(&TypedProductiveEventV1::Morphology(MorphologyEventV1 {
                        lemma: LemmaSplitKeyV1 {
                            language: "ru".to_string(),
                            normalized_lemma: lemma.lemma.clone(),
                        },
                        normalized_surface: surface.clone(),
                        canonical_form_ref: binding.form_ref,
                        canonical_feature_mask:
                            crate::nanda_wave::morphology_phase::parse_features(features)
                                .expect("canonical feature mask"),
                        slot,
                        support: 1,
                        provenance: format!("fixture:F:{}:{features}", lemma.lemma).into_bytes(),
                    }))
                    .expect("morphology event");
            }
        }
        let morphology_raw = morphology_writer.finish().expect("morphology raw");
        let morphology_sorted = external_sort_verified_spool(
            &morphology_raw,
            &ExternalSpoolSortConfigV1 {
                root: root.join("morphology-sorted"),
                maximum_buffer_bytes: 4096,
                maximum_open_runs: 4,
                write_buffer_bytes: 1024,
            },
        )
        .expect("morphology sort");
        let reduced = reduce_train_morphology_with_imported_ownership(
            &morphology_sorted,
            &canonical_l2,
            &TrainMorphologyReduceConfigV1 {
                output_path: root.join("lemmas.p2l"),
                write_buffer_bytes: 4096,
                maximum_lemma_bytes: 4096,
            },
        )
        .expect("imported morphology reduce");
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

        let mut train_by_fold = [None; super::super::PRODUCTIVE_V1_INNER_FOLDS as usize];
        let mut train_indices = Vec::new();
        let mut calibration_indices = Vec::new();
        let mut heldout_index = None;
        for (index, lemma) in lemmas.iter().enumerate() {
            let key = LemmaSplitKeyV1 {
                language: "ru".to_string(),
                normalized_lemma: lemma.lemma.clone(),
            };
            match super::super::events::deterministic_productive_split(&key, split_seed) {
                ProductiveSplitV1::Train => {
                    train_indices.push(index);
                    let fold = deterministic_inner_fold(&key, split_seed) as usize;
                    train_by_fold[fold].get_or_insert(index);
                }
                ProductiveSplitV1::Calibration => calibration_indices.push(index),
                ProductiveSplitV1::HeldoutLemma => {
                    heldout_index.get_or_insert(index);
                }
                ProductiveSplitV1::Proof => unreachable!("lemma split never returns proof"),
            };
        }
        assert!(train_by_fold.iter().all(Option::is_some));
        assert!(!calibration_indices.is_empty());
        let heldout_index = heldout_index.expect("heldout lemma");

        let mut context_writer = TypedEventSpoolWriterV1::create(TypedEventSpoolConfigV1 {
            root: root.join("context-raw"),
            shard_count: 4,
            split_seed,
            compiler_version: 1,
            normalization_version: 1,
            write_buffer_bytes: 4096,
        })
        .expect("context spool");
        let mut ordinal = 0;
        for target_index in train_by_fold.into_iter().flatten() {
            let competitor_index = *train_indices
                .iter()
                .find(|candidate| **candidate != target_index)
                .expect("distinct TRAIN competitor");
            append_context_pair(
                &mut context_writer,
                &canonical_l2,
                &lemmas[target_index],
                &lemmas[competitor_index],
                source_slot,
                ordinal,
            );
            ordinal += 1;
        }
        for target_index in calibration_indices.iter().copied().take(3) {
            append_context_pair(
                &mut context_writer,
                &canonical_l2,
                &lemmas[target_index],
                &lemmas[train_indices[0]],
                source_slot,
                ordinal,
            );
            ordinal += 1;
        }
        let heldout = &lemmas[heldout_index];
        let heldout_binding = binding_for_surface(&canonical_l2, &heldout.source);
        context_writer
            .append(&TypedProductiveEventV1::Proof(ProofEventV1 {
                lemma: LemmaSplitKeyV1 {
                    language: "ru".to_string(),
                    normalized_lemma: heldout.lemma.clone(),
                },
                proof_identity: Sha256::digest(b"productive-evidence-proof").into(),
                observed_surface: heldout.source.clone(),
                valid_targets: vec![heldout_binding],
                explicit_invalid_competitors: Vec::new(),
                scene: fixture_scene(&heldout.source),
                provenance: b"fixture:H".to_vec(),
            }))
            .expect("proof event");
        let context_raw = context_writer.finish().expect("context raw");
        let context_sorted = external_sort_verified_spool(
            &context_raw,
            &ExternalSpoolSortConfigV1 {
                root: root.join("context-sorted"),
                maximum_buffer_bytes: 2048,
                maximum_open_runs: 4,
                write_buffer_bytes: 1024,
            },
        )
        .expect("context sort");
        let mut evidence = reduce_productive_evidence(
            &context_sorted,
            &reduced,
            &induction,
            &canonical_l2,
            &axis_schema,
            &EvidenceReduceConfigV1 {
                maximum_record_bytes: 4096,
                maximum_context_events: 128,
                maximum_candidates_per_group: 8,
            },
        )
        .expect("evidence reduce");

        assert_eq!(evidence.context_occurrence_events, ordinal as u64);
        assert_eq!(evidence.direct_contradiction_events, ordinal as u64);
        assert_eq!(evidence.proof_events, 1);
        assert_eq!(
            evidence.training_pairs,
            super::super::PRODUCTIVE_V1_INNER_FOLDS as u32
        );
        assert_eq!(evidence.calibration_groups, 3);
        assert!(evidence.compiler_evidence.evidence_priors[0].contradiction_count > 0);
        assert!(evidence.phase_profiles > 0);
        assert_eq!(evidence.selected_phase_centers, 0);

        let bootstrap = compile_productive_package(
            &reduced,
            &induction,
            &axis_schema,
            &evidence.compiler_evidence,
            &ProductivePackageCompilerConfigV1 {
                output_path: root.join("productive-bootstrap.p2m"),
                maximum_record_bytes: 4096,
                l11_package_sha256: [1; 32],
                canonical_l2_package_sha256: [2; 32],
                productive_package_byte_budget: 16 * 1024 * 1024,
                steady_rss_kib_budget: 256 * 1024,
                peak_rss_kib_budget: 512 * 1024,
                cold_publish_budget_us: 1_000_000,
                hot_p99_budget_us: 5_000,
            },
        )
        .expect("compiled bootstrap package");
        let bootstrap_runtime =
            PackagedProductiveRuntimeV1::load(&bootstrap.path, [1; 32], [2; 32])
                .expect("reopened bootstrap package");
        let replay = replay_packaged_calibration(
            &context_sorted,
            &reduced,
            &induction,
            &canonical_l2,
            &bootstrap_runtime,
            &axis_schema,
            &EvidenceReduceConfigV1 {
                maximum_record_bytes: 4096,
                maximum_context_events: 128,
                maximum_candidates_per_group: 8,
            },
        )
        .expect("packaged calibration replay");
        assert_eq!(replay.source_groups, 3);
        assert_eq!(replay.target_retained_groups, 3);
        assert_eq!(replay.target_lost_groups, 0);
        assert!(replay.candidate_rows >= replay.target_retained_groups);
        evidence.compiler_evidence.calibration = replay.calibration;
        let final_package = compile_productive_package(
            &reduced,
            &induction,
            &axis_schema,
            &evidence.compiler_evidence,
            &ProductivePackageCompilerConfigV1 {
                output_path: root.join("productive-final.p2m"),
                maximum_record_bytes: 4096,
                l11_package_sha256: [1; 32],
                canonical_l2_package_sha256: [2; 32],
                productive_package_byte_budget: 16 * 1024 * 1024,
                steady_rss_kib_budget: 256 * 1024,
                peak_rss_kib_budget: 512 * 1024,
                cold_publish_budget_us: 1_000_000,
                hot_p99_budget_us: 5_000,
            },
        )
        .expect("compiled final productive package");
        let runtime = PackagedProductiveRuntimeV1::load(&final_package.path, [1; 32], [2; 32])
            .expect("reopened final productive package");
        assert_eq!(
            runtime.package_bytes(),
            final_package.package_bytes as usize
        );
        std::fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn missing_counts_stay_neutral_and_explicit_anti_is_directional() {
        let mut state = EvidenceStateV1::default();
        add_counts(&mut state.lemma_counts, 0, 3, 0).expect("positive");
        add_counts(&mut state.lemma_counts, 1, 0, 2).expect("anti");
        add_counts(&mut state.directional_counts, (7, 2, 3), 4, 0).expect("directional positive");
        add_counts(&mut state.directional_counts, (7, 3, 2), 0, 4).expect("directional anti");
        let priors = evidence_priors(&state).expect("priors");

        assert_eq!(priors[0].positive_count, 3);
        assert_eq!(priors[0].contradiction_count, 2);
        assert_eq!(priors[3].positive_count, 4);
        assert_eq!(priors[3].contradiction_count, 4);
        assert!(!state.lemma_counts.contains_key(&2));
    }
}
