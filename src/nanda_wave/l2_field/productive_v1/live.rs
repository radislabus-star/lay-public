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
    stable_bytes_ref, EnumerationCompletenessV1, EnumerationWorkCountersV1, GroundingNamespaceV1,
    MaterialTargetIdentityV1, NormalizationLayoutProfileIdV1, PreparedMaterialKeyV1,
    SeparatorProfileIdV1, TargetRelationV1, VerdictMembershipV1,
};
use crate::typing_transition::{action as action_operator, decision::TransitionDecisionCore};
use crate::word_reader::{replace_last_text_word, split_edge_whitespace, split_ws_segments};

use super::calibrate::{CandidateProvenanceClassV1, ProductiveCalibratedVerdictV1};
use super::composite::{CompositeGroundedVerdictV1, CompositeL2LatticeV1, CompositeSurfaceGroupV1};
use super::contour_birth::{TypedContourBirthEnumerationV1, TypedContourBirthV1};
#[cfg(test)]
use super::material_frame::prepare_context_neutral_productive_material_with_contours;
use super::material_frame::{
    prepare_context_neutral_productive_material_with_contours_and_exact_peaks, ExactPackageTupleV1,
    ExactPeakBirthEnumerationV1, PreparedTargetMaterialShadowV1,
};
use super::packaged_runtime::{
    ContextNeutralProductiveEnumerationV1, PackagedGroundedLemmaV1, PackagedProductiveCandidateV1,
    PackagedProductiveRuntimeV1,
};
use super::scene::{BoundaryKindV1, L2LocalSceneV1, LocalTokenObservationV1};

pub(super) const PRODUCTIVE_V90_SURFACE_SOURCE_ID: &str = "ProductiveL2V90Surface";
pub(super) const PRODUCTIVE_V90_GROUNDED_SOURCE_ID: &str = "ProductiveL2V90Grounded";
pub(super) const PRODUCTIVE_V90_GROUNDED_WINNER_SOURCE_ID: &str = "ProductiveL2V90GroundedWinner";
pub(super) const PRODUCTIVE_V90_LAYOUT_SOURCE_ID: &str = "ProductiveL2V90Layout";
pub(super) const PRODUCTIVE_V90_CONTOUR_SOURCE_ID: &str = "ProductiveL2V90Contour";
pub(in crate::nanda_wave::l2_field) const PRODUCTIVE_V90_TYPED_EXACT_SOURCE_ID: &str =
    "ProductiveL2V90TypedExact";
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) enum PreparedFieldMaterialScopeV1 {
    ContextNeutral,
    ContextShapedObservation,
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
    prepared_material: PreparedTargetMaterialShadowV1,
    material_scope: PreparedFieldMaterialScopeV1,
}

impl PreparedCanonicalTokenField {
    fn from_lattice(
        observed: &str,
        contour_provenance: CanonicalContourProvenance,
        productive_package_sha256: [u8; 32],
        lattice: CompositeL2LatticeV1,
        prepared_material: PreparedTargetMaterialShadowV1,
        material_scope: PreparedFieldMaterialScopeV1,
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
            prepared_material,
            material_scope,
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

    pub(in crate::nanda_wave::l2_field) fn prepared_material(
        &self,
    ) -> &PreparedTargetMaterialShadowV1 {
        &self.prepared_material
    }

    pub(in crate::nanda_wave::l2_field) const fn material_scope(
        &self,
    ) -> PreparedFieldMaterialScopeV1 {
        self.material_scope
    }

    pub(in crate::nanda_wave::l2_field) fn legacy_authority(&self) -> &L2FieldAuthority {
        &self.authority
    }

    pub(in crate::nanda_wave::l2_field) fn replacement_lattice_surfaces(&self) -> Vec<&str> {
        self.lattice
            .surface_groups
            .iter()
            .filter(|group| {
                !group
                    .normalized_surface
                    .eq_ignore_ascii_case(&self.observed)
            })
            .map(|group| group.normalized_surface.as_str())
            .collect()
    }

    pub(in crate::nanda_wave::l2_field) fn replacement_grounded_l11_surfaces(&self) -> Vec<&str> {
        self.lattice
            .grounded_candidates
            .iter()
            .filter(|candidate| {
                !candidate
                    .normalized_surface
                    .eq_ignore_ascii_case(&self.observed)
            })
            .map(|candidate| candidate.normalized_surface.as_str())
            .collect()
    }

    pub(in crate::nanda_wave::l2_field) fn original_has_grounded_l11_evidence(&self) -> bool {
        self.lattice.grounded_candidates.iter().any(|candidate| {
            candidate
                .normalized_surface
                .eq_ignore_ascii_case(&self.observed)
        })
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn exact_peak_candidate_rows(&self) -> Vec<(u32, String)> {
        self.prepared_material.exact_peak_candidate_rows()
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn exact_peak_certificate_rows(
        &self,
    ) -> Vec<(u32, String, u8, String)> {
        self.prepared_material.exact_peak_certificate_rows()
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn exact_peak_material_completeness(
        &self,
    ) -> EnumerationCompletenessV1 {
        self.prepared_material.completeness()
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn exact_peak_lattice_surfaces(&self) -> Vec<&str> {
        self.lattice
            .surface_groups
            .iter()
            .filter(|group| group.exact_peak_birth)
            .map(|group| group.normalized_surface.as_str())
            .collect()
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn exact_peak_surface_has_independent_authority(
        &self,
        surface: &str,
    ) -> bool {
        matches!(
            &self.authority,
            L2FieldAuthority::Winner { surface: winner }
                if winner.eq_ignore_ascii_case(surface)
        )
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
    prepare_live_productive_v1_field_inner(
        context_prefix,
        observed,
        canonical_index,
        runtime,
        contour_seeds,
        form_groundings,
        surface_groundings,
        ExactPeakBirthEnumerationV1::complete_empty(),
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
pub(in crate::nanda_wave::l2_field) fn prepare_live_productive_v1_field_with_exact_peaks(
    context_prefix: &str,
    observed: &str,
    canonical_index: &StandaloneL2Field,
    runtime: &PackagedProductiveRuntimeV1,
    contour_seeds: &[CanonicalContourSeed],
    form_groundings: &[CanonicalFormGrounding],
    surface_groundings: &[CanonicalSurfaceGrounding],
    exact_peaks: ExactPeakBirthEnumerationV1,
) -> Result<PreparedCanonicalTokenField, String> {
    prepare_live_productive_v1_field_inner(
        context_prefix,
        observed,
        canonical_index,
        runtime,
        contour_seeds,
        form_groundings,
        surface_groundings,
        exact_peaks,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn prepare_live_productive_v1_field_inner(
    context_prefix: &str,
    observed: &str,
    canonical_index: &StandaloneL2Field,
    runtime: &PackagedProductiveRuntimeV1,
    contour_seeds: &[CanonicalContourSeed],
    form_groundings: &[CanonicalFormGrounding],
    surface_groundings: &[CanonicalSurfaceGrounding],
    exact_peaks: ExactPeakBirthEnumerationV1,
) -> Result<PreparedCanonicalTokenField, String> {
    let normalized_observed = super::super::compositional::normalize_surface(observed);
    let exact_peak_surfaces = exact_peaks
        .normalized_surfaces()
        .filter(|surface| !surface.eq_ignore_ascii_case(&normalized_observed))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
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

    let contour_births = shared_field_contour_births(
        observed,
        &restoration,
        canonical_index,
        contour_seeds,
        form_groundings,
        surface_groundings,
    );
    let prepared_material =
        prepare_context_neutral_productive_material_with_contours_and_exact_peaks(
            observed,
            ExactPackageTupleV1 {
                l11_sha256: runtime.l11_package_sha256(),
                canonical_l2_sha256: runtime.canonical_l2_package_sha256(),
                productive_sha256: runtime.package_sha256(),
            },
            ContextNeutralProductiveEnumerationV1 {
                readout: productive.clone(),
                productive_work: EnumerationWorkCountersV1::default(),
                aggregate_work: EnumerationWorkCountersV1::default(),
                work_budget_exceeded: false,
            },
            contour_births,
            exact_peaks,
        )?;
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
    lattice.merge_exact_peak_surfaces(exact_peak_surfaces)?;
    if !lattice.grounded_winner_is_preserved() {
        return Err("productive V90 dropped the grounded L1.1 winner".to_string());
    }

    Ok(PreparedCanonicalTokenField::from_lattice(
        observed,
        contour_provenance,
        runtime.package_sha256(),
        lattice,
        prepared_material,
        PreparedFieldMaterialScopeV1::ContextShapedObservation,
    ))
}

fn shared_field_contour_births(
    observed: &str,
    restoration: &RestorationReadout,
    canonical_index: &StandaloneL2Field,
    contour_seeds: &[CanonicalContourSeed],
    form_groundings: &[CanonicalFormGrounding],
    surface_groundings: &[CanonicalSurfaceGrounding],
) -> TypedContourBirthEnumerationV1 {
    let mut births = BTreeMap::<
        (
            String,
            GroundingNamespaceV1,
            u32,
            TargetRelationV1,
            VerdictMembershipV1,
        ),
        TypedContourBirthV1,
    >::new();
    let l11_tied = match restoration {
        RestorationReadout::Tied { candidates, .. } => candidates
            .iter()
            .map(|candidate| candidate.terminal_id)
            .collect::<BTreeSet<_>>(),
        _ => BTreeSet::new(),
    };
    for evidence in contour_seeds {
        let Some(terminal_id) = evidence.seed.terminal_id else {
            continue;
        };
        let membership = if evidence.seed.authority {
            VerdictMembershipV1::L11Winner
        } else if l11_tied.contains(&terminal_id) {
            VerdictMembershipV1::L11Tied
        } else {
            VerdictMembershipV1::Grounded
        };
        insert_shared_contour_birth(
            &mut births,
            observed,
            &evidence.query_surface,
            &evidence.seed.surface,
            GroundingNamespaceV1::L11Terminal,
            terminal_id,
            shared_target_relation(evidence.relation),
            membership,
            evidence.seed.score_milli,
        );
    }
    for grounding in form_groundings {
        insert_shared_contour_birth(
            &mut births,
            observed,
            observed,
            &grounding.normalized_surface,
            GroundingNamespaceV1::CanonicalForm,
            grounding.form_ref,
            shared_target_relation(grounding.relation),
            VerdictMembershipV1::Grounded,
            grounding.support_milli,
        );
    }
    for grounding in surface_groundings {
        let Some(form_ref) = canonical_index.form_ref_for_surface(&grounding.normalized_surface)
        else {
            continue;
        };
        insert_shared_contour_birth(
            &mut births,
            observed,
            observed,
            &grounding.normalized_surface,
            GroundingNamespaceV1::CanonicalForm,
            form_ref,
            shared_target_relation(grounding.relation),
            VerdictMembershipV1::Grounded,
            grounding.support_milli,
        );
    }
    let births = births.into_values().collect::<Vec<_>>();
    let mut digest_bytes = b"lay-shared-canonical-field-contours-v1\0".to_vec();
    for birth in &births {
        digest_bytes.extend_from_slice(&(birth.normalized_surface.len() as u64).to_le_bytes());
        digest_bytes.extend_from_slice(birth.normalized_surface.as_bytes());
        digest_bytes.push(birth.grounding_namespace as u8);
        digest_bytes.extend_from_slice(&birth.grounding_ref.to_le_bytes());
        digest_bytes.push(birth.relation as u8);
        digest_bytes.push(birth.verdict_membership as u8);
    }
    let first = stable_bytes_ref(&digest_bytes) as u64;
    digest_bytes.push(1);
    let second = stable_bytes_ref(&digest_bytes) as u64;
    TypedContourBirthEnumerationV1 {
        logical_match_count: births.len(),
        births,
        work: EnumerationWorkCountersV1::default(),
        all_seen_digest: [first, second],
        overflow_reason: None,
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
fn insert_shared_contour_birth(
    births: &mut BTreeMap<
        (
            String,
            GroundingNamespaceV1,
            u32,
            TargetRelationV1,
            VerdictMembershipV1,
        ),
        TypedContourBirthV1,
    >,
    observed: &str,
    query_surface: &str,
    surface: &str,
    namespace: GroundingNamespaceV1,
    grounding_ref: u32,
    relation: TargetRelationV1,
    membership: VerdictMembershipV1,
    support_milli: u32,
) {
    let normalized_surface = super::super::compositional::normalize_surface(surface);
    if normalized_surface.is_empty() {
        return;
    }
    let mut derivation_bytes = b"lay-shared-canonical-field-derivation-v1\0".to_vec();
    for value in [observed, query_surface, normalized_surface.as_str()] {
        derivation_bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
        derivation_bytes.extend_from_slice(value.as_bytes());
    }
    let operator_ref = 0x5348_0000_u32 | u32::from(relation as u8);
    let derivation_ref = stable_bytes_ref(&derivation_bytes);
    let key = (
        normalized_surface.clone(),
        namespace,
        grounding_ref,
        relation,
        membership,
    );
    births.entry(key).or_insert(TypedContourBirthV1 {
        normalized_surface,
        grounding_namespace: namespace,
        grounding_ref,
        relation,
        operator_ref,
        derivation_ref,
        verdict_membership: membership,
        support_milli: support_milli.min(u32::from(u16::MAX)) as u16,
    });
}

const fn shared_target_relation(relation: CanonicalContourRelation) -> TargetRelationV1 {
    match relation {
        CanonicalContourRelation::ExactLayout => TargetRelationV1::ExactLayout,
        CanonicalContourRelation::LayoutThenTypo => TargetRelationV1::LayoutThenTypo,
        CanonicalContourRelation::Identity | CanonicalContourRelation::InverseGeometry => {
            TargetRelationV1::L11Restoration
        }
    }
}

pub(in crate::nanda_wave::l2_field) fn materialize_live_productive_v1_field(
    original: &str,
    observed: &str,
    field: &PreparedCanonicalTokenField,
) -> Result<CanonicalL2FieldReadout, String> {
    if field.observed != observed {
        return Err("productive V90 field token identity mismatch".to_string());
    }
    let exact_layout_surfaces = field.prepared_material.exact_peak_layout_surfaces();
    let candidates = materialize_live_candidates(
        original,
        observed,
        &field.lattice,
        field.common_l3_required,
        &field.contour_provenance,
        &exact_layout_surfaces,
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
    exact_layout_surfaces: &BTreeSet<String>,
) -> Result<Vec<UnifiedCorrectionCandidate>, String> {
    let trace_stages = std::env::var_os("LAY_L2_FIELD_TRACE").is_some();
    #[cfg(test)]
    let admission_trace_session =
        crate::typing_transition::proposal_admission::begin_admission_trace_session()?;
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
        let replacement = if group.exact_peak_birth
            && exact_layout_surfaces.contains(&group.normalized_surface)
        {
            replace_last_exact_layout_token(original, &projected)
        } else {
            replace_last_text_word(original, &projected)
        }
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
        #[cfg(test)]
        let post_override_started = admission_trace_session.post_override_started();
        let live_authority_override = !candidate_has_live_authority(
            &field_authority,
            origin,
            is_protected,
            &group.normalized_surface,
        ) && gate.action == CandidateGateAction::Eligible;
        if live_authority_override {
            gate = CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: live_authority_deferral_reason(&field_authority),
            };
        }
        #[cfg(test)]
        if let Some(started) = post_override_started {
            crate::typing_transition::proposal_admission::record_live_authority_override(
                started.elapsed(),
                live_authority_override,
                &gate,
            );
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
        } else if group.exact_peak_birth {
            PRODUCTIVE_V90_TYPED_EXACT_SOURCE_ID
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
    #[cfg(test)]
    let admission_trace_line =
        admission_trace_session.finish_line(lattice.surface_groups.len(), candidates.len())?;
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
    #[cfg(test)]
    if let Some(line) = admission_trace_line {
        eprintln!("{line}");
    }
    Ok(candidates)
}

fn replace_last_exact_layout_token(text: &str, replacement: &str) -> Option<String> {
    let (leading_ws, core, trailing_ws) = split_edge_whitespace(text);
    let segments = split_ws_segments(core);
    let replace_index = segments
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, (_, is_whitespace))| (!*is_whitespace).then_some(index))?;
    let mut output = String::with_capacity(text.len().saturating_add(replacement.len()));
    output.push_str(leading_ws);
    for (index, (segment, _)) in segments.iter().enumerate() {
        if index == replace_index {
            output.push_str(replacement);
        } else {
            output.push_str(segment);
        }
    }
    output.push_str(trailing_ws);
    Some(output)
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

    #[test]
    fn exact_layout_replacement_consumes_physical_boundary_keys() {
        assert_eq!(
            replace_last_exact_layout_token("  уже [elt.ob[  ", "худеющих").as_deref(),
            Some("  уже худеющих  ")
        );
    }

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

    fn prepared_test_material(
        observed: &str,
        package_sha256: [u8; 32],
        readout: &PackagedProductiveReadoutV1,
    ) -> PreparedTargetMaterialShadowV1 {
        prepare_context_neutral_productive_material_with_contours(
            observed,
            ExactPackageTupleV1 {
                l11_sha256: package_sha256,
                canonical_l2_sha256: package_sha256,
                productive_sha256: package_sha256,
            },
            ContextNeutralProductiveEnumerationV1 {
                readout: readout.clone(),
                productive_work: EnumerationWorkCountersV1::default(),
                aggregate_work: EnumerationWorkCountersV1::default(),
                work_budget_exceeded: false,
            },
            TypedContourBirthEnumerationV1::complete_empty(),
        )
        .expect("test material must use the production preparation contract")
    }

    fn lexical_frame(
        context: &str,
        observed: &str,
        with_coordinates: bool,
    ) -> crate::lexical_authority_frame::LexicalAuthorityFrameV1 {
        let config = crate::config::LayConfig::default();
        let config_identity =
            crate::lexical_authority_frame::LexicalAuthorityConfigIdentityV1::from_config(&config);
        let scalar_count = observed.chars().count() as u32;
        let coordinates = with_coordinates.then(|| {
            crate::lexical_authority_frame::LexicalAuthorityCoordinatesV1::new(
                41,
                [41, 42],
                43,
                observed.to_string(),
                context.to_string(),
                scalar_count,
                (scalar_count, scalar_count),
                String::new(),
                0,
                44,
                config_identity.identity_fingerprint(),
            )
            .expect("valid test coordinates")
        });
        crate::lexical_authority_frame::LexicalAuthorityFrameV1::from_exact_parts(
            "/test/cohort".to_string(),
            Some("focus".to_string()),
            42,
            format!("{context}{observed}"),
            context.to_string(),
            observed.to_string(),
            false,
            true,
            crate::exact_layout_authority::FactoryEngineProfile::Ru,
            None,
            1,
            2,
            config_identity,
        )
        .with_coordinates(coordinates)
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
            exact_peak_birth: false,
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
    fn shared_contour_carries_exact_original_root_to_preservation_material() {
        let mut births = BTreeMap::new();
        insert_shared_contour_birth(
            &mut births,
            "форм",
            "форм",
            "форм",
            GroundingNamespaceV1::L11Terminal,
            7,
            TargetRelationV1::L11Restoration,
            VerdictMembershipV1::L11Winner,
            1_000,
        );

        let birth = births.values().next().expect("exact original root");
        assert_eq!(birth.normalized_surface, "форм");
        assert_eq!(birth.grounding_namespace, GroundingNamespaceV1::L11Terminal);
        assert_eq!(birth.verdict_membership, VerdictMembershipV1::L11Winner);
        assert_eq!(birth.support_milli, 1_000);
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
            &BTreeSet::new(),
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
    fn exact_peak_without_independent_authority_is_suggestion_only() {
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
        let mut lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive, None)
            .expect("empty base lattice");
        lattice
            .merge_exact_peak_surfaces(["тяжёл".to_string()])
            .expect("one exact peak");

        let candidates = materialize_live_candidates(
            "тжял",
            "тжял",
            &lattice,
            false,
            &CanonicalContourProvenance::default(),
            &BTreeSet::new(),
        )
        .expect("exact peak materialization");

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].source_id,
            PRODUCTIVE_V90_TYPED_EXACT_SOURCE_ID
        );
        assert_eq!(candidates[0].gate.action, CandidateGateAction::SuggestOnly);
        assert_eq!(
            candidates[0].gate.reason,
            "productive_v90_lattice_abstained"
        );
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
        let prepared_material = prepared_test_material("форм", [7; 32], &productive);
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
                &BTreeSet::new(),
            )
            .expect("direct materialization"),
            live_authority(&lattice, common_l3_required),
        );
        let field = PreparedCanonicalTokenField::from_lattice(
            "форм",
            provenance,
            [7; 32],
            lattice,
            prepared_material,
            PreparedFieldMaterialScopeV1::ContextNeutral,
        );

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
        let prepared_material = prepared_test_material("форм", [0; 32], &productive);
        let lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive, None)
            .expect("empty productive lattice");
        let field = PreparedCanonicalTokenField::from_lattice(
            "форм",
            CanonicalContourProvenance::default(),
            [0; 32],
            lattice,
            prepared_material,
            PreparedFieldMaterialScopeV1::ContextNeutral,
        );

        let error = materialize_live_productive_v1_field("нужна форма", "форма", &field)
            .expect_err("mismatched token identity must fail closed");

        assert_eq!(error, "productive V90 field token identity mismatch");
    }

    #[test]
    fn cohort_compare_reuses_one_field_and_preserves_live_authority() {
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
        let prepared_material = prepared_test_material("форм", [7; 32], &productive);
        let lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive, None)
            .expect("two-slot productive lattice");
        let field = PreparedCanonicalTokenField::from_lattice(
            "форм",
            CanonicalContourProvenance::default(),
            [7; 32],
            lattice,
            prepared_material,
            PreparedFieldMaterialScopeV1::ContextNeutral,
        );
        let authority_before = field.legacy_authority().clone();
        let frame = lexical_frame("нужна ", "форм", true);

        let compare =
            super::super::cohort_compare::compare_shared_canonical_cohort(&field, Some(&frame), 91);

        assert_eq!(
            compare.status,
            super::super::cohort_compare::CohortCompareStatusV1::Ready
        );
        assert_eq!(compare.field_candidate_count, 2);
        assert_eq!(compare.material_target_count, 2);
        assert_eq!(compare.retained_field_candidate_count, 2);
        assert_eq!(compare.grounded_l11_loss_count, 0);
        assert!(compare.complete_for_authority);
        assert_eq!(compare.first_divergence, None);
        assert_eq!(field.legacy_authority(), &authority_before);

        let mut context_shaped = field.clone();
        context_shaped.material_scope = PreparedFieldMaterialScopeV1::ContextShapedObservation;
        let context_shaped_compare = super::super::cohort_compare::compare_shared_canonical_cohort(
            &context_shaped,
            Some(&frame),
            91,
        );
        assert_eq!(
            context_shaped_compare.material_scope,
            PreparedFieldMaterialScopeV1::ContextShapedObservation
        );
        assert!(!context_shaped_compare.complete_for_authority);
        assert_eq!(context_shaped.legacy_authority(), &authority_before);
    }

    #[test]
    fn cohort_compare_keeps_original_grounding_outside_replacement_membership() {
        let replacement = productive_candidate(17, 2, 102, "формы");
        let productive = PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Winner {
                candidate: readout_candidate(&replacement),
                calibration_stratum_id: 1,
            },
            candidates: vec![replacement],
            logical_terminal_count: 1,
            logical_surface_basin_count: 1,
            integrity_error: None,
        };
        let original_seed = L11SeedSurface {
            terminal_id: Some(7),
            surface: "форм".to_string(),
            authority: true,
            score_milli: 1_000,
        };
        let l11 = l11_restoration_readout("форм", std::slice::from_ref(&original_seed));
        let lattice = CompositeL2LatticeV1::assemble(
            &l11,
            |terminal_id| (terminal_id == 7).then(|| "форм".to_string()),
            productive.clone(),
            None,
        )
        .expect("original plus replacement lattice");
        let original_birth = TypedContourBirthV1 {
            normalized_surface: "форм".to_string(),
            grounding_namespace: GroundingNamespaceV1::L11Terminal,
            grounding_ref: 7,
            relation: TargetRelationV1::L11Restoration,
            operator_ref: 701,
            derivation_ref: 702,
            verdict_membership: VerdictMembershipV1::L11Winner,
            support_milli: 1_000,
        };
        let prepared_material = prepare_context_neutral_productive_material_with_contours(
            "форм",
            ExactPackageTupleV1 {
                l11_sha256: [7; 32],
                canonical_l2_sha256: [7; 32],
                productive_sha256: [7; 32],
            },
            ContextNeutralProductiveEnumerationV1 {
                readout: productive,
                productive_work: EnumerationWorkCountersV1::default(),
                aggregate_work: EnumerationWorkCountersV1::default(),
                work_budget_exceeded: false,
            },
            TypedContourBirthEnumerationV1 {
                births: vec![original_birth],
                work: EnumerationWorkCountersV1::default(),
                logical_match_count: 1,
                all_seen_digest: [71, 73],
                overflow_reason: None,
            },
        )
        .expect("separate original material");
        let field = PreparedCanonicalTokenField::from_lattice(
            "форм",
            CanonicalContourProvenance::default(),
            [7; 32],
            lattice,
            prepared_material,
            PreparedFieldMaterialScopeV1::ContextNeutral,
        );
        let frame = lexical_frame("нужна ", "форм", true);

        let compare =
            super::super::cohort_compare::compare_shared_canonical_cohort(&field, Some(&frame), 91);

        assert_eq!(
            compare.status,
            super::super::cohort_compare::CohortCompareStatusV1::Ready
        );
        assert_eq!(compare.field_candidate_count, 1);
        assert_eq!(compare.material_target_count, 1);
        assert_eq!(compare.retained_field_candidate_count, 1);
        assert_eq!(compare.grounded_l11_loss_count, 0);
        assert!(compare.unretained_field_candidate_surfaces.is_empty());
        assert!(compare.lost_grounded_l11_surfaces.is_empty());
        assert!(field.original_has_grounded_l11_evidence());
        assert!(field
            .prepared_material()
            .original_has_grounded_l11_evidence());
    }

    #[test]
    fn unavailable_cohort_comparison_cannot_change_live_authority() {
        let candidate = productive_candidate(17, 1, 101, "форма");
        let productive = PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Winner {
                candidate: readout_candidate(&candidate),
                calibration_stratum_id: 1,
            },
            candidates: vec![candidate],
            logical_terminal_count: 1,
            logical_surface_basin_count: 1,
            integrity_error: None,
        };
        let l11 = RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance: None,
            candidates: Vec::new(),
        };
        let prepared_material = prepared_test_material("форм", [7; 32], &productive);
        let lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive, None)
            .expect("single-slot productive lattice");
        let field = PreparedCanonicalTokenField::from_lattice(
            "форм",
            CanonicalContourProvenance::default(),
            [7; 32],
            lattice,
            prepared_material,
            PreparedFieldMaterialScopeV1::ContextNeutral,
        );
        let authority_before = field.legacy_authority().clone();

        let missing =
            super::super::cohort_compare::compare_shared_canonical_cohort(&field, None, 91);
        let no_coordinates = lexical_frame("нужна ", "форм", false);
        let incomplete = super::super::cohort_compare::compare_shared_canonical_cohort(
            &field,
            Some(&no_coordinates),
            91,
        );
        let wrong_token = lexical_frame("нужна ", "форма", true);
        let mismatch = super::super::cohort_compare::compare_shared_canonical_cohort(
            &field,
            Some(&wrong_token),
            91,
        );

        assert_eq!(
            missing.status,
            super::super::cohort_compare::CohortCompareStatusV1::MissingFrame
        );
        assert_eq!(
            incomplete.status,
            super::super::cohort_compare::CohortCompareStatusV1::MissingCoordinates
        );
        assert_eq!(
            mismatch.status,
            super::super::cohort_compare::CohortCompareStatusV1::FrameMismatch
        );
        assert!(!missing.complete_for_authority);
        assert!(!incomplete.complete_for_authority);
        assert!(!mismatch.complete_for_authority);
        assert_eq!(field.legacy_authority(), &authority_before);
    }
}
