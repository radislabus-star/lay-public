use std::collections::{BTreeMap, BTreeSet};

use crate::candidate_contract::CandidateOrigin;
use crate::correction_core::{
    CandidateGateAction, CandidateGateDecision, CorrectionDecisionSource, MorphologySlotEvidence,
    TypingErrorClass, UnifiedCorrectionCandidate,
};
use crate::nanda_wave::l2_field::runtime::{
    CanonicalL2FieldReadout, L2FieldAuthority, StandaloneL2Field,
};
use crate::nanda_wave::lexical_grokking::restoration::{
    AbstainReason, RestorationCandidate, RestorationEvidence, RestorationReadout,
};
use crate::nanda_wave::L11SeedSurface;
use crate::text_case::apply_word_case;
use crate::typing_transition::target_evidence::{
    stable_bytes_ref, EnumerationCompletenessV1, MaterialTargetIdentityV1,
    NormalizationLayoutProfileIdV1, PreparedMaterialKeyV1, SeparatorProfileIdV1,
};
use crate::typing_transition::{action as action_operator, decision::TransitionDecisionCore};
use crate::word_reader::replace_last_text_word;

use super::calibrate::{CandidateProvenanceClassV1, ProductiveCalibratedVerdictV1};
use super::composite::{CompositeGroundedVerdictV1, CompositeL2LatticeV1, CompositeSurfaceGroupV1};
use super::packaged_runtime::{
    PackagedGroundedLemmaV1, PackagedProductiveCandidateV1, PackagedProductiveRuntimeV1,
};
use super::scene::{BoundaryKindV1, L2LocalSceneV1, LocalTokenObservationV1};

pub(super) const PRODUCTIVE_V90_SURFACE_SOURCE_ID: &str = "ProductiveL2V90Surface";
pub(super) const PRODUCTIVE_V90_GROUNDED_SOURCE_ID: &str = "ProductiveL2V90Grounded";
pub(super) const PRODUCTIVE_V90_GROUNDED_WINNER_SOURCE_ID: &str = "ProductiveL2V90GroundedWinner";
pub(super) const PRODUCTIVE_V90_LAYOUT_SOURCE_ID: &str = "ProductiveL2V90Layout";
pub(super) const PRODUCTIVE_V90_CONTOUR_SOURCE_ID: &str = "ProductiveL2V90Contour";
const MAX_ACTIVE_PACKAGE_LEMMAS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::nanda_wave::l2_field) enum CanonicalContourRelation {
    InverseGeometry,
    Identity,
    LayoutThenTypo,
    ExactLayout,
}

impl CanonicalContourRelation {
    pub(in crate::nanda_wave::l2_field) const fn tag(self) -> u8 {
        match self {
            Self::InverseGeometry => 0,
            Self::Identity => 1,
            Self::LayoutThenTypo => 2,
            Self::ExactLayout => 3,
        }
    }

    const fn candidate_origin(self) -> CandidateOrigin {
        match self {
            Self::ExactLayout => CandidateOrigin::Layout,
            Self::LayoutThenTypo => CandidateOrigin::LayoutThenTypo,
            Self::Identity | Self::InverseGeometry => CandidateOrigin::L2Surface,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct CanonicalContourSeed {
    pub(in crate::nanda_wave::l2_field) query_surface: String,
    pub(in crate::nanda_wave::l2_field) seed: L11SeedSurface,
    pub(in crate::nanda_wave::l2_field) relation: CanonicalContourRelation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct CanonicalFormGrounding {
    pub(in crate::nanda_wave::l2_field) form_ref: u32,
    pub(in crate::nanda_wave::l2_field) normalized_surface: String,
    pub(in crate::nanda_wave::l2_field) support_milli: u32,
    pub(in crate::nanda_wave::l2_field) relation: CanonicalContourRelation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct CanonicalSurfaceGrounding {
    pub(in crate::nanda_wave::l2_field) normalized_surface: String,
    pub(in crate::nanda_wave::l2_field) support_milli: u32,
    pub(in crate::nanda_wave::l2_field) relation: CanonicalContourRelation,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CanonicalContourProvenance {
    surface_relations: BTreeMap<String, CanonicalContourRelation>,
    lemma_relations: BTreeMap<u32, CanonicalContourRelation>,
}

/// Immutable L1.1 -> Productive V90 field material. Text replacement and
/// request-time L3/L4 ranking are intentionally outside this value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct PreparedCanonicalTokenField {
    observed: String,
    productive_package_sha256: [u8; 32],
    lattice: CompositeL2LatticeV1,
    common_l3_required: bool,
    authority: L2FieldAuthority,
    contour_provenance: CanonicalContourProvenance,
}

impl PreparedCanonicalTokenField {
    fn from_lattice(
        observed: &str,
        contour_provenance: CanonicalContourProvenance,
        productive_package_sha256: [u8; 32],
        lattice: CompositeL2LatticeV1,
    ) -> Self {
        let common_l3_required = lattice_surface_count(&lattice) > 1;
        let authority = live_authority(&lattice, common_l3_required);
        Self {
            observed: observed.to_string(),
            productive_package_sha256,
            lattice,
            common_l3_required,
            authority,
            contour_provenance,
        }
    }

    pub(in crate::nanda_wave::l2_field) fn observed(&self) -> &str {
        &self.observed
    }

    pub(in crate::nanda_wave::l2_field) fn productive_package_sha256(&self) -> [u8; 32] {
        self.productive_package_sha256
    }

    pub(in crate::nanda_wave::l2_field) fn common_material_key(&self) -> PreparedMaterialKeyV1 {
        let mut generation_bytes = [0_u8; 8];
        generation_bytes.copy_from_slice(&self.productive_package_sha256[..8]);
        let mut exact_package_digest_prefix = [0_u8; 16];
        exact_package_digest_prefix.copy_from_slice(&self.productive_package_sha256[..16]);
        PreparedMaterialKeyV1 {
            observed_contour_ref: stable_bytes_ref(self.observed.as_bytes()),
            normalization_layout_profile_id: NormalizationLayoutProfileIdV1(1),
            package_generation: u64::from_le_bytes(generation_bytes),
            exact_package_digest_prefix,
        }
    }

    pub(in crate::nanda_wave::l2_field) fn common_completeness(&self) -> EnumerationCompletenessV1 {
        self.lattice.common_completeness()
    }

    pub(in crate::nanda_wave::l2_field) fn common_material_target_identity(
        &self,
        surface: &str,
        separator_profile_id: Option<u32>,
    ) -> MaterialTargetIdentityV1 {
        let normalized = super::super::compositional::normalize_surface(surface);
        let normalized_ref = stable_bytes_ref(normalized.as_bytes());
        MaterialTargetIdentityV1 {
            normalized_scalars_ref: normalized_ref,
            canonical_bytes_ref: normalized_ref,
            normalization_layout_profile_id: NormalizationLayoutProfileIdV1(1),
            separator_profile_id: SeparatorProfileIdV1(separator_profile_id.unwrap_or(0)),
            exact_scalar_count: normalized.chars().count().min(usize::from(u16::MAX)) as u16,
            flags: u16::from(separator_profile_id.is_some()),
            accelerator: normalized_ref,
        }
    }
}

/// Prepares the only live L2 field owner. Canonical L2 is a read-only identity
/// index; its historical local verdict is deliberately absent from this path.
pub(in crate::nanda_wave::l2_field) fn prepare_live_productive_v1_field(
    context_prefix: &str,
    observed: &str,
    canonical_index: &StandaloneL2Field,
    runtime: &PackagedProductiveRuntimeV1,
    contour_seeds: &[CanonicalContourSeed],
    form_groundings: &[CanonicalFormGrounding],
    surface_groundings: &[CanonicalSurfaceGrounding],
) -> Result<PreparedCanonicalTokenField, String> {
    let l11_seeds = contour_seeds
        .iter()
        .map(|evidence| evidence.seed.clone())
        .collect::<Vec<_>>();
    let restoration = l11_restoration_readout(observed, &l11_seeds);
    let (groundings, contour_provenance) = package_known_groundings(
        canonical_index,
        runtime,
        contour_seeds,
        form_groundings,
        surface_groundings,
    )?;
    let scene = live_scene(context_prefix, observed, canonical_index);
    let grounded_winner_present = matches!(restoration, RestorationReadout::Winner { .. });
    let trace_stages = std::env::var_os("LAY_L2_FIELD_TRACE").is_some();
    let (productive, telemetry) = if trace_stages {
        let (readout, telemetry) = runtime.evaluate_shadow_with_cold_bindings_profiled(
            observed,
            &scene,
            &groundings,
            &[],
            grounded_winner_present,
        );
        (readout, Some(telemetry))
    } else {
        (
            runtime.evaluate_shadow_with_cold_bindings(
                observed,
                &scene,
                &groundings,
                &[],
                grounded_winner_present,
            ),
            None,
        )
    };
    if let Some(telemetry) = telemetry {
        eprintln!(
            "productive_v90_stage_trace token_chars={} l11_seeds={} groundings={} active_bindings={} setup_us={} binding_us={} traversal_us={} reduce_us={} readout_us={} logical_terminals={} surface_basins={} selected={}",
            observed.chars().count(),
            l11_seeds.len(),
            groundings.len(),
            telemetry.active_binding_count,
            telemetry.setup_us,
            telemetry.binding_preparation_us,
            telemetry.traversal_us,
            telemetry.surface_reduce_us,
            telemetry.final_readout_us,
            telemetry.logical_terminal_count,
            telemetry.logical_surface_basin_count,
            telemetry.selected_candidate_count,
        );
    }
    if let Some(error) = productive.integrity_error.as_deref() {
        return Err(format!("productive V90 integrity error: {error}"));
    }

    let surface_by_terminal = l11_seeds
        .iter()
        .filter_map(|seed| {
            seed.terminal_id
                .map(|terminal_id| (terminal_id, seed.surface.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut lattice = CompositeL2LatticeV1::assemble(
        &restoration,
        |terminal_id| surface_by_terminal.get(&terminal_id).cloned(),
        productive,
        None,
    )?;
    lattice.merge_contour_surfaces(
        surface_groundings
            .iter()
            .map(|grounding| grounding.normalized_surface.clone()),
    )?;
    if !lattice.grounded_winner_is_preserved() {
        return Err("productive V90 dropped the grounded L1.1 winner".to_string());
    }

    Ok(PreparedCanonicalTokenField::from_lattice(
        observed,
        contour_provenance,
        runtime.package_sha256(),
        lattice,
    ))
}

pub(in crate::nanda_wave::l2_field) fn materialize_live_productive_v1_field(
    original: &str,
    observed: &str,
    field: &PreparedCanonicalTokenField,
) -> Result<CanonicalL2FieldReadout, String> {
    if field.observed != observed {
        return Err("productive V90 field token identity mismatch".to_string());
    }
    let candidates = materialize_live_candidates(
        original,
        observed,
        &field.lattice,
        field.common_l3_required,
        &field.contour_provenance,
    )?;
    Ok(CanonicalL2FieldReadout::new(
        candidates,
        field.authority.clone(),
    ))
}

pub(in crate::nanda_wave::l2_field) fn canonical_live_scene_bytes(
    context_prefix: &str,
    observed: &str,
    canonical_index: &StandaloneL2Field,
) -> Vec<u8> {
    live_scene(context_prefix, observed, canonical_index).canonical_bytes()
}

fn materialize_live_candidates(
    original: &str,
    observed: &str,
    lattice: &CompositeL2LatticeV1,
    common_l3_required: bool,
    contour_provenance: &CanonicalContourProvenance,
) -> Result<Vec<UnifiedCorrectionCandidate>, String> {
    let trace_stages = std::env::var_os("LAY_L2_FIELD_TRACE").is_some();
    let setup_started = trace_stages.then(std::time::Instant::now);
    let protected_surface = lattice
        .grounded_candidates
        .iter()
        .find(|candidate| candidate.protected_winner)
        .map(|candidate| candidate.normalized_surface.as_str());
    let field_authority = live_authority(lattice, common_l3_required);
    let productive_winner = match (&lattice.productive_verdict, common_l3_required) {
        (ProductiveCalibratedVerdictV1::Winner { candidate, .. }, false) => {
            Some(candidate.normalized_surface.as_str())
        }
        (ProductiveCalibratedVerdictV1::Winner { .. }, true)
        | (ProductiveCalibratedVerdictV1::Tied { .. }, _)
        | (ProductiveCalibratedVerdictV1::Abstain { .. }, _) => None,
    };
    let productive_by_surface = lattice.productive_candidates.iter().fold(
        BTreeMap::<&str, Vec<&PackagedProductiveCandidateV1>>::new(),
        |mut map, candidate| {
            map.entry(candidate.normalized_surface.as_ref())
                .or_default()
                .push(candidate);
            map
        },
    );
    let grounded_lemmas = lattice
        .productive_candidates
        .iter()
        .filter(|candidate| candidate.grounded_support > 0)
        .map(|candidate| candidate.identity.lemma_id)
        .collect::<BTreeSet<_>>();
    let setup_us = setup_started
        .map(|started| started.elapsed().as_micros())
        .unwrap_or_default();

    let mut candidates = Vec::with_capacity(lattice.surface_groups.len());
    let mut projection_us = 0_u128;
    let mut classify_us = 0_u128;
    let mut gate_us = 0_u128;
    let mut evidence_us = 0_u128;
    for group in &lattice.surface_groups {
        if group.normalized_surface.eq_ignore_ascii_case(observed) {
            continue;
        }
        let stage_started = trace_stages.then(std::time::Instant::now);
        let projected = apply_word_case(observed, &group.normalized_surface);
        let replacement = replace_last_text_word(original, &projected)
            .ok_or_else(|| "productive V90 cannot replace the active word".to_string())?;
        let productive_nodes = productive_by_surface
            .get(group.normalized_surface.as_str())
            .cloned()
            .unwrap_or_default();
        let origin = live_candidate_origin(contour_provenance, group);
        let same_lemma_slot = productive_nodes.iter().any(|candidate| {
            candidate
                .equivalent_identities
                .iter()
                .any(|identity| grounded_lemmas.contains(&identity.lemma_id))
        });
        let declared_class = if origin == CandidateOrigin::Layout {
            TypingErrorClass::WrongLayout
        } else if origin == CandidateOrigin::LayoutThenTypo {
            TypingErrorClass::CompositeTypo
        } else if same_lemma_slot {
            TypingErrorClass::GrammarAgreement
        } else {
            TypingErrorClass::Unknown
        };
        projection_us += stage_started
            .map(|started| started.elapsed().as_micros())
            .unwrap_or_default();
        let stage_started = trace_stages.then(std::time::Instant::now);
        let error_class = action_operator::classify_token_transition(
            original,
            &replacement,
            origin,
            declared_class,
        );
        classify_us += stage_started
            .map(|started| started.elapsed().as_micros())
            .unwrap_or_default();
        let stage_started = trace_stages.then(std::time::Instant::now);
        let mut gate = TransitionDecisionCore::admit_candidate_proposal(
            original,
            &replacement,
            error_class,
            origin,
        );
        let is_protected = protected_surface == Some(group.normalized_surface.as_str());
        if !candidate_has_live_authority(
            &field_authority,
            origin,
            is_protected,
            &group.normalized_surface,
        ) && gate.action == CandidateGateAction::Eligible
        {
            gate = CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: live_authority_deferral_reason(&field_authority),
            };
        }
        gate_us += stage_started
            .map(|started| started.elapsed().as_micros())
            .unwrap_or_default();
        let stage_started = trace_stages.then(std::time::Instant::now);
        let source_id = if matches!(
            origin,
            CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
        ) {
            PRODUCTIVE_V90_LAYOUT_SOURCE_ID
        } else if is_protected {
            PRODUCTIVE_V90_GROUNDED_WINNER_SOURCE_ID
        } else if group.contour_grounding {
            PRODUCTIVE_V90_CONTOUR_SOURCE_ID
        } else if !productive_nodes.is_empty() {
            PRODUCTIVE_V90_SURFACE_SOURCE_ID
        } else {
            PRODUCTIVE_V90_GROUNDED_SOURCE_ID
        };
        let mut candidate = UnifiedCorrectionCandidate::new(
            replacement,
            CorrectionDecisionSource::Nanda,
            origin,
            source_id,
            error_class,
            gate,
        );
        candidate.extend_morphology_slot_evidence(productive_slot_evidence(
            &productive_nodes,
            productive_winner,
        ));
        candidates.push(candidate);
        evidence_us += stage_started
            .map(|started| started.elapsed().as_micros())
            .unwrap_or_default();
    }
    if trace_stages {
        eprintln!(
            "productive_v90_materialization_trace surfaces={} emitted={} setup_us={} projection_us={} classify_us={} gate_us={} evidence_us={}",
            lattice.surface_groups.len(),
            candidates.len(),
            setup_us,
            projection_us,
            classify_us,
            gate_us,
            evidence_us,
        );
    }
    Ok(candidates)
}

fn candidate_has_live_authority(
    authority: &L2FieldAuthority,
    origin: CandidateOrigin,
    protected_grounded_winner: bool,
    normalized_surface: &str,
) -> bool {
    origin == CandidateOrigin::Layout
        || protected_grounded_winner
        || matches!(
            authority,
            L2FieldAuthority::Winner { surface }
                if surface.eq_ignore_ascii_case(normalized_surface)
        )
}

fn live_authority_deferral_reason(authority: &L2FieldAuthority) -> &'static str {
    match authority {
        L2FieldAuthority::Tied { .. } => "productive_v90_lattice_requires_common_l3",
        L2FieldAuthority::Abstain => "productive_v90_lattice_abstained",
        L2FieldAuthority::Unavailable => "productive_v90_lattice_unavailable",
        L2FieldAuthority::Winner { .. } => "productive_v90_non_winner_requires_common_l3",
    }
}

fn live_candidate_origin(
    contour_provenance: &CanonicalContourProvenance,
    group: &CompositeSurfaceGroupV1,
) -> CandidateOrigin {
    if let Some(relation) = contour_provenance
        .surface_relations
        .get(&group.normalized_surface)
        .copied()
    {
        return relation.candidate_origin();
    }
    group
        .productive_identities
        .iter()
        .filter_map(|identity| {
            contour_provenance
                .lemma_relations
                .get(&identity.lemma_id)
                .copied()
        })
        .max()
        .filter(|relation| {
            matches!(
                relation,
                CanonicalContourRelation::ExactLayout | CanonicalContourRelation::LayoutThenTypo
            )
        })
        .map(|_| CandidateOrigin::LayoutThenTypo)
        .unwrap_or(CandidateOrigin::L2Surface)
}

fn productive_slot_evidence(
    candidates: &[&PackagedProductiveCandidateV1],
    productive_winner: Option<&str>,
) -> Vec<MorphologySlotEvidence> {
    let mut identities = BTreeSet::new();
    let mut evidence = Vec::new();
    for candidate in candidates {
        let selected = productive_winner == Some(candidate.normalized_surface.as_ref());
        for identity in &candidate.equivalent_identities {
            if !identities.insert((identity.lemma_id, identity.target_slot_id)) {
                continue;
            }
            evidence.push(MorphologySlotEvidence {
                lemma_id: identity.lemma_id,
                source_feature_mask: 0,
                target_feature_mask: identity.target_slot_id,
                context_positive_support: if selected {
                    candidate.grounded_support.max(1)
                } else {
                    0
                },
                context_alternative_support: if selected { 0 } else { 1 },
                context_posterior_milli: if selected { 1_000 } else { 0 },
                slot_evidence_milli: if selected { 1_000 } else { 0 },
                joint_evidence_milli: if selected { 1_000 } else { 0 },
                generated: candidate.provenance != CandidateProvenanceClassV1::Exact,
            });
        }
    }
    evidence
}

fn lattice_surface_count(lattice: &CompositeL2LatticeV1) -> usize {
    lattice.surface_groups.len()
}

fn live_authority(lattice: &CompositeL2LatticeV1, common_l3_required: bool) -> L2FieldAuthority {
    if let CompositeGroundedVerdictV1::Winner { terminal_id } = lattice.original_l11_verdict {
        if let Some(surface) = lattice
            .grounded_candidates
            .iter()
            .find(|candidate| candidate.candidate.terminal_id == terminal_id)
            .map(|candidate| candidate.normalized_surface.clone())
        {
            return L2FieldAuthority::Winner { surface };
        }
    }
    if common_l3_required {
        return L2FieldAuthority::Tied {
            surfaces: lattice
                .surface_groups
                .iter()
                .map(|group| group.normalized_surface.clone())
                .collect(),
        };
    }
    match &lattice.productive_verdict {
        ProductiveCalibratedVerdictV1::Winner { candidate, .. } => L2FieldAuthority::Winner {
            surface: candidate.normalized_surface.clone(),
        },
        ProductiveCalibratedVerdictV1::Tied { candidates, .. } => L2FieldAuthority::Tied {
            surfaces: candidates
                .iter()
                .map(|candidate| candidate.normalized_surface.clone())
                .collect(),
        },
        ProductiveCalibratedVerdictV1::Abstain { .. } => {
            let surfaces = lattice
                .grounded_candidates
                .iter()
                .map(|candidate| candidate.normalized_surface.clone())
                .collect::<Vec<_>>();
            if matches!(
                lattice.original_l11_verdict,
                CompositeGroundedVerdictV1::Tied { .. }
                    | CompositeGroundedVerdictV1::TiedOverflow { .. }
            ) && !surfaces.is_empty()
            {
                L2FieldAuthority::Tied { surfaces }
            } else {
                L2FieldAuthority::Abstain
            }
        }
    }
}

fn package_known_groundings(
    canonical_index: &StandaloneL2Field,
    runtime: &PackagedProductiveRuntimeV1,
    contour_seeds: &[CanonicalContourSeed],
    form_groundings: &[CanonicalFormGrounding],
    surface_groundings: &[CanonicalSurfaceGrounding],
) -> Result<(Vec<PackagedGroundedLemmaV1>, CanonicalContourProvenance), String> {
    let mut evidence_by_lemma = BTreeMap::<u32, (u32, CanonicalContourRelation)>::new();
    let mut surface_relations = BTreeMap::<String, CanonicalContourRelation>::new();
    for evidence in contour_seeds {
        let normalized_surface =
            super::super::compositional::normalize_surface(&evidence.seed.surface);
        merge_relation(
            &mut surface_relations,
            normalized_surface,
            evidence.relation,
        );
        let Some(form_ref) = canonical_index.form_ref_for_surface(&evidence.seed.surface) else {
            continue;
        };
        for (lemma_id, _) in canonical_index.imported_binding_identities_for_form(form_ref) {
            merge_lemma_evidence(
                &mut evidence_by_lemma,
                lemma_id,
                evidence.seed.score_milli.max(1),
                evidence.relation,
            );
        }
    }
    for grounding in form_groundings {
        let decoded = canonical_index
            .imported_surface_for_form(grounding.form_ref)
            .ok_or_else(|| "typed contour grounding references an unknown form".to_string())?;
        if !decoded.eq_ignore_ascii_case(&grounding.normalized_surface) {
            return Err("typed contour grounding surface does not match its form ref".to_string());
        }
        merge_relation(
            &mut surface_relations,
            super::super::compositional::normalize_surface(&grounding.normalized_surface),
            grounding.relation,
        );
        for (lemma_id, _) in
            canonical_index.imported_binding_identities_for_form(grounding.form_ref)
        {
            merge_lemma_evidence(
                &mut evidence_by_lemma,
                lemma_id,
                grounding.support_milli.max(1),
                grounding.relation,
            );
        }
    }
    for grounding in surface_groundings {
        merge_relation(
            &mut surface_relations,
            super::super::compositional::normalize_surface(&grounding.normalized_surface),
            grounding.relation,
        );
    }
    let mut ranked = evidence_by_lemma.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
             .0
            .cmp(&left.1 .0)
            .then_with(|| right.1 .1.cmp(&left.1 .1))
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked.truncate(MAX_ACTIVE_PACKAGE_LEMMAS);

    let mut grounded = Vec::new();
    let mut lemma_relations = BTreeMap::new();
    for (lemma_id, (seed_support, relation)) in ranked {
        lemma_relations.insert(lemma_id, relation);
        for descriptor in runtime.grounding_descriptors(lemma_id)? {
            let normalized_source = canonical_index
                .imported_surface_for_form(descriptor.canonical_source_form_ref)
                .ok_or_else(|| {
                    "productive V90 grounding lacks its canonical source surface".to_string()
                })?;
            grounded.push(PackagedGroundedLemmaV1 {
                lemma_id: descriptor.lemma_id,
                pos_domain: descriptor.pos_domain,
                canonical_source_form_ref: descriptor.canonical_source_form_ref,
                source_slot_id: descriptor.source_slot_id,
                normalized_source,
                grounded_support: descriptor.grounded_support.max(seed_support),
            });
        }
    }
    grounded.sort_by(|left, right| {
        (left.lemma_id, left.pos_domain, left.source_slot_id).cmp(&(
            right.lemma_id,
            right.pos_domain,
            right.source_slot_id,
        ))
    });
    grounded.dedup_by(|left, right| {
        (left.lemma_id, left.pos_domain) == (right.lemma_id, right.pos_domain)
    });
    Ok((
        grounded,
        CanonicalContourProvenance {
            surface_relations,
            lemma_relations,
        },
    ))
}

fn merge_relation(
    relations: &mut BTreeMap<String, CanonicalContourRelation>,
    surface: String,
    relation: CanonicalContourRelation,
) {
    relations
        .entry(surface)
        .and_modify(|retained| *retained = (*retained).max(relation))
        .or_insert(relation);
}

fn merge_lemma_evidence(
    evidence_by_lemma: &mut BTreeMap<u32, (u32, CanonicalContourRelation)>,
    lemma_id: u32,
    support_milli: u32,
    relation: CanonicalContourRelation,
) {
    evidence_by_lemma
        .entry(lemma_id)
        .and_modify(|retained| {
            retained.0 = retained.0.max(support_milli);
            retained.1 = retained.1.max(relation);
        })
        .or_insert((support_milli, relation));
}

fn l11_restoration_readout(observed: &str, seeds: &[L11SeedSurface]) -> RestorationReadout {
    let mut seen = BTreeSet::new();
    let candidates = seeds
        .iter()
        .filter_map(|seed| {
            let terminal_id = seed.terminal_id?;
            seen.insert(terminal_id).then_some(RestorationCandidate {
                terminal_id,
                evidence: RestorationEvidence {
                    geometry_distance: crate::text_metrics::damerau_levenshtein(
                        observed,
                        &seed.surface,
                    )
                    .min(u8::MAX as usize) as u8,
                    positive_milli: seed.score_milli.min(u32::from(u16::MAX)) as u16,
                    backward_milli: seed.score_milli.min(u32::from(u16::MAX)) as u16,
                    ..RestorationEvidence::default()
                },
            })
        })
        .collect::<Vec<_>>();
    let authoritative = seeds
        .iter()
        .filter(|seed| seed.authority)
        .filter_map(|seed| seed.terminal_id)
        .collect::<BTreeSet<_>>();
    if authoritative.len() == 1 {
        let terminal_id = *authoritative.first().expect("one authoritative terminal");
        if let Some(candidate) = candidates
            .iter()
            .find(|candidate| candidate.terminal_id == terminal_id)
        {
            return RestorationReadout::Winner {
                candidate: *candidate,
            };
        }
    }
    let geometry_distance = candidates
        .iter()
        .map(|candidate| candidate.evidence.geometry_distance)
        .min();
    if candidates.len() >= 2 {
        RestorationReadout::Tied {
            geometry_distance: geometry_distance.unwrap_or_default(),
            candidates,
        }
    } else {
        RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance,
            candidates,
        }
    }
}

fn live_scene(
    context_prefix: &str,
    observed: &str,
    canonical_index: &StandaloneL2Field,
) -> L2LocalSceneV1 {
    let left = context_prefix
        .split_whitespace()
        .rev()
        .filter_map(normalize_context_token)
        .take(2)
        .collect::<Vec<_>>();
    let token_observation = |surface: Option<String>| {
        surface.map(|normalized_surface| {
            let lemma_ids = canonical_index
                .form_ref_for_surface(&normalized_surface)
                .into_iter()
                .flat_map(|form_ref| canonical_index.imported_binding_identities_for_form(form_ref))
                .map(|(lemma_id, _)| lemma_id)
                .collect::<BTreeSet<_>>();
            LocalTokenObservationV1 {
                normalized_surface,
                lemma_id: (lemma_ids.len() == 1)
                    .then(|| *lemma_ids.first().expect("one contextual lemma")),
                morphology_slot: None,
            }
        })
    };
    L2LocalSceneV1 {
        current_token: observed.to_string(),
        current_normalized_scalars: observed.chars().map(u32::from).collect(),
        left_tokens: [
            token_observation(left.get(1).cloned()),
            token_observation(left.first().cloned()),
        ],
        boundary_before: if context_prefix.trim().is_empty() {
            BoundaryKindV1::None
        } else {
            BoundaryKindV1::Token
        },
        ..L2LocalSceneV1::default()
    }
}

fn normalize_context_token(token: &str) -> Option<String> {
    let normalized = token
        .trim_matches(|character: char| !character.is_alphanumeric() && character != '-')
        .to_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::super::calibrate::{
        CandidateProvenanceClassV1, CandidateRankOriginV1, ReadoutCandidateV1,
    };
    use super::super::geometry::GeometryTerminalEvidenceV1;
    use super::super::packaged_runtime::PackagedProductiveReadoutV1;
    use super::super::types::ProductiveCandidateIdentityV1;
    use super::*;

    fn productive_candidate(
        lemma_id: u32,
        target_slot_id: u32,
        normalized_surface_id: u32,
        surface: &str,
    ) -> PackagedProductiveCandidateV1 {
        let identity = ProductiveCandidateIdentityV1 {
            lemma_id,
            paradigm_id: 11,
            program_id: target_slot_id,
            target_slot_id,
            normalized_surface_id,
            variant_id: 1,
        };
        PackagedProductiveCandidateV1 {
            identity,
            equivalent_identities: vec![identity],
            normalized_surface: surface.into(),
            score_q16: 1_000 - i64::from(target_slot_id),
            geometry: GeometryTerminalEvidenceV1::default(),
            provenance: CandidateProvenanceClassV1::TrainingSeenGenerated,
            minimum_independent_support: 2,
            grounded_support: 2,
            ambiguity_center_cosine: 0,
            equivalent_identity_count: 1,
            equivalent_paradigm_count: 1,
            minimum_equivalent_support: 2,
            maximum_equivalent_support: 2,
            rank_origin: CandidateRankOriginV1::BaseV64,
            cross_lane_certified: false,
        }
    }

    fn readout_candidate(candidate: &PackagedProductiveCandidateV1) -> ReadoutCandidateV1 {
        ReadoutCandidateV1 {
            identity: candidate.identity,
            equivalent_identities: candidate.equivalent_identities.clone(),
            normalized_surface: candidate.normalized_surface.to_string(),
            score_q16: candidate.score_q16,
            grounded_lemma_evidence: candidate.grounded_support,
            exact_osa_distance: 0,
            exact_form: false,
            cross_lemma_ownership_satisfied: false,
            rank_origin: candidate.rank_origin,
            cross_lane_certified: candidate.cross_lane_certified,
        }
    }

    fn surface_group(
        surface: &str,
        grounded: bool,
        productive_identities: Vec<ProductiveCandidateIdentityV1>,
    ) -> CompositeSurfaceGroupV1 {
        CompositeSurfaceGroupV1 {
            normalized_surface: surface.to_string(),
            grounded_terminal_ids: grounded.then_some(7).into_iter().collect(),
            productive_identities,
            grounded_protection: false,
            contour_grounding: false,
        }
    }

    #[test]
    fn physical_layout_origin_is_bound_to_cross_script_grounding() {
        let identity = productive_candidate(17, 2, 102, "собаки").identity;
        let provenance = CanonicalContourProvenance {
            surface_relations: [("собака".to_string(), CanonicalContourRelation::ExactLayout)]
                .into_iter()
                .collect(),
            lemma_relations: [(17, CanonicalContourRelation::ExactLayout)]
                .into_iter()
                .collect(),
        };
        assert_eq!(
            live_candidate_origin(&provenance, &surface_group("собака", true, Vec::new())),
            CandidateOrigin::Layout
        );
        assert_eq!(
            live_candidate_origin(&provenance, &surface_group("собаки", false, vec![identity]),),
            CandidateOrigin::LayoutThenTypo
        );
        assert_eq!(
            live_candidate_origin(&provenance, &surface_group("tyn", true, Vec::new()),),
            CandidateOrigin::L2Surface
        );
        assert_eq!(
            live_candidate_origin(
                &CanonicalContourProvenance::default(),
                &surface_group("собака", true, Vec::new()),
            ),
            CandidateOrigin::L2Surface
        );
        assert!(candidate_has_live_authority(
            &L2FieldAuthority::Tied {
                surfaces: vec!["собака".to_string(), "собаки".to_string()],
            },
            CandidateOrigin::Layout,
            false,
            "собака",
        ));
        assert!(!candidate_has_live_authority(
            &L2FieldAuthority::Tied {
                surfaces: vec!["собака".to_string(), "собаки".to_string()],
            },
            CandidateOrigin::LayoutThenTypo,
            false,
            "собаки",
        ));
    }

    #[test]
    fn l11_authority_is_derived_from_typed_seed_not_candidate_order() {
        let readout = l11_restoration_readout(
            "проврека",
            &[
                L11SeedSurface {
                    terminal_id: Some(3),
                    surface: "проверка".to_string(),
                    authority: false,
                    score_milli: 900,
                },
                L11SeedSurface {
                    terminal_id: Some(7),
                    surface: "проврека".to_string(),
                    authority: true,
                    score_milli: 800,
                },
            ],
        );
        assert!(matches!(
            readout,
            RestorationReadout::Winner {
                candidate: RestorationCandidate { terminal_id: 7, .. }
            }
        ));
    }

    #[test]
    fn non_authoritative_single_seed_remains_abstain() {
        let readout = l11_restoration_readout(
            "форма",
            &[L11SeedSurface {
                terminal_id: Some(3),
                surface: "формы".to_string(),
                authority: false,
                score_milli: 700,
            }],
        );
        assert!(matches!(readout, RestorationReadout::Abstain { .. }));
    }

    #[test]
    fn multiple_productive_surfaces_defer_slot_selection_to_common_l3() {
        let nominative = productive_candidate(17, 1, 101, "форма");
        let genitive = productive_candidate(17, 2, 102, "формы");
        let productive = PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Winner {
                candidate: readout_candidate(&nominative),
                calibration_stratum_id: 1,
            },
            candidates: vec![nominative, genitive],
            logical_terminal_count: 2,
            logical_surface_basin_count: 2,
            integrity_error: None,
        };
        let l11 = RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance: None,
            candidates: Vec::new(),
        };
        let lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive, None)
            .expect("two-slot productive lattice");

        let common_l3_required = lattice_surface_count(&lattice) > 1;
        assert!(common_l3_required);
        assert!(matches!(
            live_authority(&lattice, common_l3_required),
            L2FieldAuthority::Tied { ref surfaces }
                if surfaces == &["форма".to_string(), "формы".to_string()]
        ));

        let candidates = materialize_live_candidates(
            "нужна форм",
            "форм",
            &lattice,
            common_l3_required,
            &CanonicalContourProvenance::default(),
        )
        .expect("common L3 candidates");
        assert_eq!(candidates.len(), 2);
        assert!(candidates
            .iter()
            .all(|candidate| candidate.gate.action == CandidateGateAction::SuggestOnly));
        assert!(candidates.iter().all(|candidate| {
            candidate
                .morphology_slot_evidence
                .iter()
                .any(|evidence| evidence.lemma_id == 17)
        }));
    }

    #[test]
    fn immutable_field_materialization_preserves_the_complete_readout() {
        let nominative = productive_candidate(17, 1, 101, "форма");
        let genitive = productive_candidate(17, 2, 102, "формы");
        let productive = PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Winner {
                candidate: readout_candidate(&nominative),
                calibration_stratum_id: 1,
            },
            candidates: vec![nominative, genitive],
            logical_terminal_count: 2,
            logical_surface_basin_count: 2,
            integrity_error: None,
        };
        let l11 = RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance: None,
            candidates: Vec::new(),
        };
        let lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive, None)
            .expect("two-slot productive lattice");
        let common_l3_required = lattice_surface_count(&lattice) > 1;
        let provenance = CanonicalContourProvenance::default();
        let expected = CanonicalL2FieldReadout::new(
            materialize_live_candidates(
                "нужна форм",
                "форм",
                &lattice,
                common_l3_required,
                &provenance,
            )
            .expect("direct materialization"),
            live_authority(&lattice, common_l3_required),
        );
        let field = PreparedCanonicalTokenField::from_lattice("форм", provenance, [7; 32], lattice);

        let actual = materialize_live_productive_v1_field("нужна форм", "форм", &field)
            .expect("immutable field materialization");

        assert_eq!(actual, expected);
        assert_eq!(field.observed(), "форм");
        assert_eq!(field.productive_package_sha256(), [7; 32]);
        assert_eq!(
            field.common_material_key(),
            PreparedMaterialKeyV1 {
                observed_contour_ref: stable_bytes_ref("форм".as_bytes()),
                normalization_layout_profile_id: NormalizationLayoutProfileIdV1(1),
                package_generation: u64::from_le_bytes([7; 8]),
                exact_package_digest_prefix: [7; 16],
            }
        );
        let completeness = field.common_completeness();
        assert_eq!(
            completeness.state(),
            crate::typing_transition::target_evidence::EnumerationStateV1::Complete
        );
        assert_eq!(completeness.logical_count_lower_bound(), 2);
        assert_eq!(
            field.common_material_target_identity("формы", Some(11)),
            MaterialTargetIdentityV1 {
                normalized_scalars_ref: stable_bytes_ref("формы".as_bytes()),
                canonical_bytes_ref: stable_bytes_ref("формы".as_bytes()),
                normalization_layout_profile_id: NormalizationLayoutProfileIdV1(1),
                separator_profile_id: SeparatorProfileIdV1(11),
                exact_scalar_count: 5,
                flags: 1,
                accelerator: stable_bytes_ref("формы".as_bytes()),
            }
        );
    }

    #[test]
    fn immutable_field_rejects_a_different_observed_token() {
        let productive = PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Abstain {
                suggestions: Vec::new(),
                productive_overflow: false,
            },
            candidates: Vec::new(),
            logical_terminal_count: 0,
            logical_surface_basin_count: 0,
            integrity_error: None,
        };
        let l11 = RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance: None,
            candidates: Vec::new(),
        };
        let lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive, None)
            .expect("empty productive lattice");
        let field = PreparedCanonicalTokenField::from_lattice(
            "форм",
            CanonicalContourProvenance::default(),
            [0; 32],
            lattice,
        );

        let error = materialize_live_productive_v1_field("нужна форма", "форма", &field)
            .expect_err("mismatched token identity must fail closed");

        assert_eq!(error, "productive V90 field token identity mismatch");
    }
}
