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
    stable_bytes_ref, CanonicalL1AnchorKindV1, CanonicalL1AnchorProofV1, EnumerationWorkCountersV1,
    GroundingNamespaceV1, TargetRelationV1, VerdictMembershipV1,
};
#[cfg(test)]
use crate::typing_transition::target_evidence::{
    EnumerationCompletenessV1, MaterialTargetIdentityV1, NormalizationLayoutProfileIdV1,
    PreparedMaterialKeyV1, SeparatorProfileIdV1,
};
use crate::typing_transition::{action as action_operator, decision::TransitionDecisionCore};
use crate::word_reader::{replace_last_text_word, split_edge_whitespace, split_ws_segments};

use super::calibrate::{CandidateProvenanceClassV1, ProductiveCalibratedVerdictV1};
use super::composite::{CompositeGroundedVerdictV1, CompositeL2LatticeV1, CompositeSurfaceGroupV1};
use super::contour_birth::{TypedContourBirthEnumerationV1, TypedContourBirthV1};
#[cfg(test)]
use super::material_frame::{
    prepare_context_neutral_productive_material_with_contours, CanonicalL1AnchorProofFaultV1,
    ExactPeakCandidateInputV1, ExactSearchProofFaultV1,
};
use super::material_frame::{
    prepare_context_neutral_productive_material_with_contours_and_exact_peaks,
    prepare_frame_bound_lexical_authority_material, ExactPackageTupleV1,
    ExactPeakBirthEnumerationV1, PreparedTargetMaterialShadowV1,
    FROZEN_V90_ENUMERATION_WORK_BUDGET,
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
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "report-contract discriminator retained while context-shaped authority is disabled"
        )
    )]
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
    // Context-shaped material remains the display/ranking projection.
    prepared_material: PreparedTargetMaterialShadowV1,
    // Certificate settlement consumes only this independently enumerated,
    // context-neutral material from the same cached field owner.
    authority_material: PreparedTargetMaterialShadowV1,
    material_scope: PreparedFieldMaterialScopeV1,
}

impl PreparedCanonicalTokenField {
    #[cfg(test)]
    fn from_lattice(
        observed: &str,
        contour_provenance: CanonicalContourProvenance,
        productive_package_sha256: [u8; 32],
        lattice: CompositeL2LatticeV1,
        prepared_material: PreparedTargetMaterialShadowV1,
        material_scope: PreparedFieldMaterialScopeV1,
    ) -> Self {
        let authority_material = prepared_material.clone();
        Self::from_lattice_with_authority_material(
            observed,
            contour_provenance,
            productive_package_sha256,
            lattice,
            prepared_material,
            authority_material,
            material_scope,
        )
    }

    fn from_lattice_with_authority_material(
        observed: &str,
        contour_provenance: CanonicalContourProvenance,
        productive_package_sha256: [u8; 32],
        lattice: CompositeL2LatticeV1,
        prepared_material: PreparedTargetMaterialShadowV1,
        authority_material: PreparedTargetMaterialShadowV1,
        material_scope: PreparedFieldMaterialScopeV1,
    ) -> Self {
        let common_l3_required =
            lattice_surface_count(&lattice) > 1 || lattice.exact_peak_incompleteness.is_some();
        let authority = live_authority(&lattice, common_l3_required);
        Self {
            observed: observed.to_string(),
            productive_package_sha256,
            lattice,
            common_l3_required,
            authority,
            contour_provenance,
            prepared_material,
            authority_material,
            material_scope,
        }
    }

    pub(in crate::nanda_wave::l2_field) fn observed(&self) -> &str {
        &self.observed
    }

    pub(in crate::nanda_wave::l2_field) fn productive_package_sha256(&self) -> [u8; 32] {
        self.productive_package_sha256
    }

    #[cfg(test)]
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

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn common_completeness(&self) -> EnumerationCompletenessV1 {
        self.lattice.common_completeness()
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn prepared_material(
        &self,
    ) -> &PreparedTargetMaterialShadowV1 {
        &self.prepared_material
    }

    pub(in crate::nanda_wave::l2_field) fn authority_material(
        &self,
    ) -> &PreparedTargetMaterialShadowV1 {
        &self.authority_material
    }

    pub(in crate::nanda_wave::l2_field) const fn material_scope(
        &self,
    ) -> PreparedFieldMaterialScopeV1 {
        self.material_scope
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn corrupt_canonical_l1_anchor_proof_for_test(
        &mut self,
        fault: CanonicalL1AnchorProofFaultV1,
    ) {
        self.authority_material
            .corrupt_canonical_l1_anchor_proof_for_test(fault);
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn authority_partition_valid_for_test(&self) -> bool {
        self.authority_material.validates_relation_partition_proof()
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn corrupt_observed_canonical_l1_anchor_for_test(
        &mut self,
    ) {
        self.authority_material
            .corrupt_observed_canonical_l1_anchor_for_test();
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

    #[cfg(test)]
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
    let l11_seeds = contour_seeds
        .iter()
        .map(|evidence| evidence.seed.clone())
        .collect::<Vec<_>>();
    let restoration = l11_restoration_readout(observed, &l11_seeds);
    let (groundings, contour_provenance, preparatory_work) = package_known_groundings(
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
        &exact_peaks,
        runtime.l11_package_sha256(),
        runtime.canonical_l2_package_sha256(),
    )?;
    let authority_enumeration = runtime.enumerate_context_neutral_material(
        observed,
        &groundings,
        &[],
        preparatory_work,
        FROZEN_V90_ENUMERATION_WORK_BUDGET,
    );
    let authority_material = prepare_frame_bound_lexical_authority_material(
        observed,
        ExactPackageTupleV1 {
            l11_sha256: runtime.l11_package_sha256(),
            canonical_l2_sha256: runtime.canonical_l2_package_sha256(),
            productive_sha256: runtime.package_sha256(),
        },
        authority_enumeration,
        contour_births.clone(),
        exact_peaks.clone(),
    )?;
    let (exact_peaks, exact_peak_incompleteness) =
        exact_peaks.common_field_projection(observed, runtime.canonical_l2_package_sha256());
    let normalized_observed = super::super::compositional::normalize_surface(observed);
    let exact_peak_surfaces = exact_peaks
        .normalized_surfaces()
        .filter(|surface| !surface.eq_ignore_ascii_case(&normalized_observed))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
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
    lattice.merge_exact_peak_surfaces(exact_peak_surfaces, exact_peak_incompleteness)?;
    if !lattice.grounded_winner_is_preserved() {
        return Err("productive V90 dropped the grounded L1.1 winner".to_string());
    }

    Ok(
        PreparedCanonicalTokenField::from_lattice_with_authority_material(
            observed,
            contour_provenance,
            runtime.package_sha256(),
            lattice,
            prepared_material,
            authority_material,
            PreparedFieldMaterialScopeV1::ContextNeutral,
        ),
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "contour and package proof identities remain explicit"
)]
fn shared_field_contour_births(
    observed: &str,
    restoration: &RestorationReadout,
    canonical_index: &StandaloneL2Field,
    contour_seeds: &[CanonicalContourSeed],
    form_groundings: &[CanonicalFormGrounding],
    surface_groundings: &[CanonicalSurfaceGrounding],
    exact_peaks: &ExactPeakBirthEnumerationV1,
    l11_sha256: [u8; 32],
    canonical_l2_sha256: [u8; 32],
) -> Result<TypedContourBirthEnumerationV1, String> {
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
    let mut canonical_l1_anchor_proofs = BTreeMap::<u32, CanonicalL1AnchorProofV1>::new();
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
    if let Some(form_ref) = canonical_index.form_ref_for_surface(observed) {
        if canonical_index
            .decode_form_ref(form_ref)
            .is_some_and(|decoded| decoded.as_bytes() == observed.as_bytes())
        {
            if let Some(anchor) = canonical_index.l1_lexical_anchor_for_form_ref(form_ref) {
                insert_shared_canonical_l1_anchor_birth(
                    &mut births,
                    &mut canonical_l1_anchor_proofs,
                    observed,
                    observed,
                    observed,
                    form_ref,
                    anchor,
                    l11_sha256,
                    canonical_l2_sha256,
                    TargetRelationV1::L11Restoration,
                    VerdictMembershipV1::Grounded,
                    0,
                )?;
            }
        }
    }
    if let Some(partition_targets) =
        exact_peaks.validated_authority_target_rows(observed, canonical_l2_sha256)
    {
        for (form_ref, normalized_surface) in partition_targets {
            let Some(decoded_surface) = canonical_index.decode_form_ref(form_ref) else {
                continue;
            };
            if decoded_surface.as_bytes() != normalized_surface.as_bytes() {
                continue;
            }
            let Some(anchor) = canonical_index.l1_lexical_anchor_for_form_ref(form_ref) else {
                continue;
            };
            insert_shared_canonical_l1_anchor_birth(
                &mut births,
                &mut canonical_l1_anchor_proofs,
                observed,
                observed,
                &normalized_surface,
                form_ref,
                anchor,
                l11_sha256,
                canonical_l2_sha256,
                TargetRelationV1::L11Restoration,
                VerdictMembershipV1::Grounded,
                0,
            )?;
        }
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
    let canonical_l1_anchor_proofs = canonical_l1_anchor_proofs.into_values().collect::<Vec<_>>();
    let logical_match_count = births
        .iter()
        .map(|birth| birth.normalized_surface.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let mut digest_bytes = b"lay-shared-canonical-field-contours-v1\0".to_vec();
    for birth in &births {
        digest_bytes.extend_from_slice(&(birth.normalized_surface.len() as u64).to_le_bytes());
        digest_bytes.extend_from_slice(birth.normalized_surface.as_bytes());
        digest_bytes.push(birth.grounding_namespace as u8);
        digest_bytes.extend_from_slice(&birth.grounding_ref.to_le_bytes());
        digest_bytes.push(birth.relation as u8);
        digest_bytes.push(birth.verdict_membership as u8);
    }
    for proof in &canonical_l1_anchor_proofs {
        digest_bytes.extend_from_slice(&proof.proof_ref.to_le_bytes());
        let identity = proof.canonical_identity_bytes();
        digest_bytes.extend_from_slice(&(identity.len() as u64).to_le_bytes());
        digest_bytes.extend_from_slice(&identity);
    }
    let first = stable_bytes_ref(&digest_bytes) as u64;
    digest_bytes.push(1);
    let second = stable_bytes_ref(&digest_bytes) as u64;
    Ok(TypedContourBirthEnumerationV1 {
        logical_match_count,
        births,
        canonical_l1_anchor_proofs,
        work: EnumerationWorkCountersV1::default(),
        all_seen_digest: [first, second],
        overflow_reason: None,
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "the producer binds every canonical anchor identity explicitly"
)]
fn insert_shared_canonical_l1_anchor_birth(
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
    proofs: &mut BTreeMap<u32, CanonicalL1AnchorProofV1>,
    observed: &str,
    query_surface: &str,
    target: &str,
    target_form_ref: u32,
    anchor: crate::nanda_wave::l2_field::runtime::CanonicalL1LexicalAnchorV1,
    l11_sha256: [u8; 32],
    canonical_l2_sha256: [u8; 32],
    relation: TargetRelationV1,
    membership: VerdictMembershipV1,
    support_milli: u32,
) -> Result<(), String> {
    let kind = if anchor.is_direct() {
        CanonicalL1AnchorKindV1::Direct
    } else {
        CanonicalL1AnchorKindV1::SameLemma
    };
    let proof = CanonicalL1AnchorProofV1::new(
        target.to_string(),
        target_form_ref,
        anchor.anchor_form_ref(),
        anchor.lemma_id(),
        anchor.terminal_id(),
        kind,
        l11_sha256,
        canonical_l2_sha256,
    )
    .ok_or_else(|| "canonical L1 anchor proof identity is invalid".to_string())?;
    if let Some(retained) = proofs.get(&proof.proof_ref) {
        if retained != &proof {
            return Err("canonical L1 anchor compact reference is ambiguous".to_string());
        }
    } else {
        proofs.insert(proof.proof_ref, proof.clone());
    }
    insert_shared_contour_birth(
        births,
        observed,
        query_surface,
        target,
        GroundingNamespaceV1::CanonicalL1Anchor,
        proof.proof_ref,
        relation,
        membership,
        support_milli,
    );
    Ok(())
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
        if group.exact_peak_birth {
            candidate = candidate.with_canonical_l2_exact_partition_membership();
        }
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
    protected_grounded_winner
        || matches!(
            authority,
            L2FieldAuthority::Winner { surface }
                if surface.eq_ignore_ascii_case(normalized_surface)
        )
        || (origin == CandidateOrigin::Layout
            && (!normalized_surface.is_ascii()
                || !crate::layout_autoswitch::english_layout_target_requires_context(
                    normalized_surface,
                )))
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
) -> Result<
    (
        Vec<PackagedGroundedLemmaV1>,
        CanonicalContourProvenance,
        EnumerationWorkCountersV1,
    ),
    String,
> {
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
    let mut grounding_lookups = 0_u64;
    for (lemma_id, (seed_support, relation)) in ranked {
        lemma_relations.insert(lemma_id, relation);
        grounding_lookups = grounding_lookups
            .checked_add(1)
            .ok_or_else(|| "productive V90 grounding work counter overflow".to_string())?;
        for descriptor in runtime.grounding_descriptors(lemma_id)? {
            grounding_lookups = grounding_lookups
                .checked_add(1)
                .ok_or_else(|| "productive V90 grounding work counter overflow".to_string())?;
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
        EnumerationWorkCountersV1 {
            grounding_lookups,
            ..EnumerationWorkCountersV1::default()
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
    use crate::nanda_wave::lexical_grokking::Phase7dCertificateOracle;

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

    fn certified_exact_peaks(
        observed: &str,
        surfaces: &[&str],
        canonical_l2_sha256: [u8; 32],
    ) -> ExactPeakBirthEnumerationV1 {
        let oracle = Phase7dCertificateOracle::new(observed).expect("test oracle");
        ExactPeakBirthEnumerationV1::from_candidates(
            surfaces
                .iter()
                .map(|surface| ExactPeakCandidateInputV1 {
                    form_ref: test_canonical_index()
                        .form_ref_for_surface(surface)
                        .expect("test exact surface must be canonical"),
                    normalized_surface: (*surface).to_string(),
                    certificates: oracle
                        .certificate_evidence(surface)
                        .expect("test exact certificate"),
                })
                .collect(),
        )
        .expect("test exact peaks")
        .with_test_search_proof(observed, canonical_l2_sha256)
        .expect("test exact search proof")
    }

    fn test_canonical_index() -> &'static StandaloneL2Field {
        static CANONICAL: std::sync::OnceLock<StandaloneL2Field> = std::sync::OnceLock::new();
        CANONICAL.get_or_init(|| {
            let corpus = crate::nanda_wave::l2_field::teacher::L2TeacherCorpus::parse_tsv(
                "F\tlemma-a\tформа\tnoun:nom:sg\n\
                 F\tlemma-a\tформу\tnoun:acc:sg\n\
                 F\tlemma-a\tформы\tnoun:gen:sg\n\
                 T\tlemma-a\tформа\tnoun:nom:sg\t_ формы\n\
                 H\tlemma-a\tформы\tnoun:gen:sg\tформа _\n",
            )
            .expect("canonical test corpus");
            let (package, _) =
                crate::nanda_wave::l2_field::compiler::compile_l2_package(&corpus, 7, |surface| {
                    match surface {
                        "форма" => Some(17),
                        "формы" => Some(18),
                        _ => None,
                    }
                })
                .expect("canonical test package");
            StandaloneL2Field::from_package(package).expect("canonical test index")
        })
    }

    fn test_anchor_replay(
    ) -> super::super::cohort_compare::CanonicalL1AnchorReplayContextV1<'static> {
        super::super::cohort_compare::CanonicalL1AnchorReplayContextV1::new(
            test_canonical_index(),
            [7; 32],
            [7; 32],
        )
    }

    fn test_anchor_birth(
        source: &str,
        target: &str,
        membership: VerdictMembershipV1,
        support_milli: u16,
    ) -> (TypedContourBirthV1, CanonicalL1AnchorProofV1) {
        let canonical = test_canonical_index();
        let target_form_ref = canonical
            .form_ref_for_surface(target)
            .expect("anchor target form");
        let anchor = canonical
            .l1_lexical_anchor_for_form_ref(target_form_ref)
            .expect("anchor target identity");
        let proof = CanonicalL1AnchorProofV1::new(
            target.to_string(),
            target_form_ref,
            anchor.anchor_form_ref(),
            anchor.lemma_id(),
            anchor.terminal_id(),
            if anchor.is_direct() {
                CanonicalL1AnchorKindV1::Direct
            } else {
                CanonicalL1AnchorKindV1::SameLemma
            },
            [7; 32],
            [7; 32],
        )
        .expect("canonical anchor proof");
        (
            TypedContourBirthV1 {
                normalized_surface: target.to_string(),
                grounding_namespace: GroundingNamespaceV1::CanonicalL1Anchor,
                grounding_ref: proof.proof_ref,
                relation: TargetRelationV1::L11Restoration,
                operator_ref: super::super::cohort_compare::shared_witness_operator_ref(
                    TargetRelationV1::L11Restoration,
                ),
                derivation_ref: super::super::cohort_compare::shared_witness_derivation_ref(
                    source, source, target,
                ),
                verdict_membership: membership,
                support_milli,
            },
            proof,
        )
    }

    fn td117_singleton_field(
        exact_peaks: ExactPeakBirthEnumerationV1,
    ) -> PreparedCanonicalTokenField {
        let replacement = productive_candidate(17, 1, 101, "форма");
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
        let l11 = RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance: None,
            candidates: Vec::new(),
        };
        let lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive.clone(), None)
            .expect("singleton productive lattice");
        let (grounded_birth, anchor_proof) =
            test_anchor_birth("форм", "форма", VerdictMembershipV1::Grounded, 1_000);
        let package_tuple = ExactPackageTupleV1 {
            l11_sha256: [7; 32],
            canonical_l2_sha256: [7; 32],
            productive_sha256: [7; 32],
        };
        let contour_births = TypedContourBirthEnumerationV1 {
            births: vec![grounded_birth],
            canonical_l1_anchor_proofs: vec![anchor_proof],
            work: EnumerationWorkCountersV1::default(),
            logical_match_count: 1,
            all_seen_digest: [17, 117],
            overflow_reason: None,
        };
        let neutral_enumeration = ContextNeutralProductiveEnumerationV1 {
            readout: productive.clone(),
            productive_work: EnumerationWorkCountersV1::default(),
            aggregate_work: EnumerationWorkCountersV1::default(),
            work_budget_exceeded: false,
        };
        let prepared_material = prepare_context_neutral_productive_material_with_contours(
            "форм",
            package_tuple,
            neutral_enumeration.clone(),
            contour_births.clone(),
        )
        .expect("complete context-neutral material");
        let authority_material = prepare_frame_bound_lexical_authority_material(
            "форм",
            package_tuple,
            neutral_enumeration,
            contour_births,
            exact_peaks,
        )
        .expect("relation-partition authority material");
        PreparedCanonicalTokenField::from_lattice_with_authority_material(
            "форм",
            CanonicalContourProvenance::default(),
            [7; 32],
            lattice,
            prepared_material,
            authority_material,
            PreparedFieldMaterialScopeV1::ContextNeutral,
        )
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
    fn short_layout_materialization_retains_targets_without_inventing_authority() {
        use crate::correction_core::TypingErrorEvent;
        use crate::typing_transition::decision::{DecisionEvidenceMode, TransitionDecisionPolicy};

        for target in ["bb", "dog", "lay", "hello", "дом"] {
            let needs_context = matches!(target, "bb" | "dog");
            if target.is_ascii() {
                assert!(crate::layout_autoswitch::is_known_english_layout_autoswitch_word(target));
            }
            let direction = if target.is_ascii() {
                crate::dict::Direction::Us2Ru
            } else {
                crate::dict::Direction::Ru2Us
            };
            let observed = crate::dict::convert(target, direction);
            let original = format!(" {observed} ");
            for state in [
                "abstain",
                "tied",
                "overflow",
                "grounded_winner",
                "productive_winner",
                "other_winner",
            ] {
                let grounded = RestorationCandidate {
                    terminal_id: 7,
                    evidence: RestorationEvidence::default(),
                };
                let unchanged = RestorationCandidate {
                    terminal_id: 8,
                    evidence: RestorationEvidence::default(),
                };
                let l11 = match state {
                    "grounded_winner" => RestorationReadout::Winner {
                        candidate: grounded,
                    },
                    "other_winner" => RestorationReadout::Winner {
                        candidate: unchanged,
                    },
                    "tied" => RestorationReadout::Tied {
                        geometry_distance: 0,
                        candidates: vec![grounded, unchanged],
                    },
                    "overflow" => RestorationReadout::TiedOverflow {
                        geometry_distance: 0,
                        total_candidates: 33,
                        candidates: vec![grounded, unchanged],
                    },
                    _ => RestorationReadout::Abstain {
                        reason: AbstainReason::NoCandidates,
                        geometry_distance: None,
                        candidates: vec![grounded],
                    },
                };
                let productive = productive_candidate(17, 1, 101, target);
                let verdict = if state == "productive_winner" {
                    ProductiveCalibratedVerdictV1::Winner {
                        candidate: readout_candidate(&productive),
                        calibration_stratum_id: 1,
                    }
                } else {
                    ProductiveCalibratedVerdictV1::Abstain {
                        suggestions: vec![readout_candidate(&productive)],
                        productive_overflow: false,
                    }
                };
                let lattice = CompositeL2LatticeV1::assemble(
                    &l11,
                    |id| match id {
                        7 => Some(target.to_string()),
                        8 => Some(observed.clone()),
                        _ => None,
                    },
                    PackagedProductiveReadoutV1 {
                        verdict,
                        candidates: vec![productive],
                        logical_terminal_count: 1,
                        logical_surface_basin_count: 1,
                        integrity_error: None,
                    },
                    None,
                )
                .expect("complete bounded test lattice");
                let provenance = CanonicalContourProvenance {
                    surface_relations: [(
                        target.to_string(),
                        CanonicalContourRelation::ExactLayout,
                    )]
                    .into_iter()
                    .collect(),
                    ..CanonicalContourProvenance::default()
                };
                let candidates = materialize_live_candidates(
                    &original,
                    &observed,
                    &lattice,
                    lattice_surface_count(&lattice) > 1,
                    &provenance,
                    &BTreeSet::new(),
                )
                .expect("retained layout target");
                assert_eq!(candidates.len(), 1, "{target}/{state}");
                let candidate = &candidates[0];
                assert_eq!(candidate.replacement, format!(" {target} "));
                assert_eq!(candidate.origin, CandidateOrigin::Layout);
                assert_eq!(candidate.error_class, TypingErrorClass::WrongLayout);
                assert_eq!(candidate.evidence_count(), 1);
                assert_eq!(candidate.evidence[0].gate, candidate.gate);
                assert!(!candidate.morphology_slot_evidence.is_empty());
                let defer =
                    needs_context && !matches!(state, "grounded_winner" | "productive_winner");
                if defer {
                    let event = TypingErrorEvent {
                        original: original.clone(),
                        core: observed.clone(),
                        current_word: observed.clone(),
                        input_class: TypingErrorClass::WrongLayout,
                    };
                    let batch = TransitionDecisionCore::evaluate_candidates(
                        &event,
                        &candidates,
                        TransitionDecisionPolicy::default(),
                        DecisionEvidenceMode::FullField(None),
                    );
                    assert_eq!(batch.evaluations.len(), 1);
                    assert!(batch.evaluations[0].action.verifier_passed);
                    assert!(!batch.evaluations[0].signals.l3_pairwise_certified);
                    assert!(!batch.evaluations[0]
                        .transition
                        .l4_signed_signal
                        .exact_positive());
                    assert!(
                        batch.selected_transition.is_none(),
                        "{target}/{state}: {batch:#?}"
                    );
                    assert_eq!(batch.selected_index, None);
                }
                assert_eq!(
                    candidate.gate.action,
                    if defer {
                        CandidateGateAction::SuggestOnly
                    } else {
                        CandidateGateAction::Eligible
                    },
                    "{target}/{state}"
                );
            }
        }
    }

    #[test]
    fn ordinary_layout_authority_counts_ascii_letters_and_preserves_explicit_winners() {
        for (target, ordinary_authority) in [
            ("BB", false),
            ("dog", false),
            ("a-1234", false),
            ("dog-1234", false),
            ("LAY", true),
            ("hello", true),
            ("дом", true),
        ] {
            for authority in [
                L2FieldAuthority::Unavailable,
                L2FieldAuthority::Abstain,
                L2FieldAuthority::Tied {
                    surfaces: vec![target.to_string()],
                },
                L2FieldAuthority::Winner {
                    surface: "other".to_string(),
                },
            ] {
                assert_eq!(
                    candidate_has_live_authority(
                        &authority,
                        CandidateOrigin::Layout,
                        false,
                        target,
                    ),
                    ordinary_authority,
                    "{target}/{authority:?}"
                );
                assert!(candidate_has_live_authority(
                    &authority,
                    CandidateOrigin::Layout,
                    true,
                    target,
                ));
            }
            assert!(candidate_has_live_authority(
                &L2FieldAuthority::Winner {
                    surface: target.to_lowercase()
                },
                CandidateOrigin::Layout,
                false,
                target,
            ));
        }
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
            .merge_exact_peak_surfaces(["тяжёл".to_string()], None)
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
        assert!(candidates[0].belongs_to_canonical_l2_exact_partition());
        assert_eq!(
            candidates[0].gate.reason,
            "productive_v90_lattice_abstained"
        );
    }

    #[test]
    fn exact_overflow_defers_productive_singleton_and_preserves_grounded_winner() {
        use crate::correction_core::TypingErrorEvent;
        use crate::typing_transition::decision::{DecisionEvidenceMode, TransitionDecisionPolicy};
        use crate::typing_transition::target_evidence::{
            EnumerationStateV1, IncompletenessReasonV1,
        };

        for raw_count in [1, 113] {
            for grounded_winner in [false, true] {
                for overflow in [false, true] {
                    let target = productive_candidate(17, 1, 1, "проверка");
                    let productive = PackagedProductiveReadoutV1 {
                        verdict: ProductiveCalibratedVerdictV1::Winner {
                            candidate: readout_candidate(&target),
                            calibration_stratum_id: 1,
                        },
                        candidates: vec![target],
                        logical_terminal_count: 1,
                        logical_surface_basin_count: 1,
                        integrity_error: None,
                    };
                    let l11 = if grounded_winner {
                        RestorationReadout::Winner {
                            candidate: RestorationCandidate {
                                terminal_id: 7,
                                evidence: RestorationEvidence::default(),
                            },
                        }
                    } else {
                        RestorationReadout::Abstain {
                            reason: AbstainReason::NoCandidates,
                            geometry_distance: None,
                            candidates: Vec::new(),
                        }
                    };
                    let material = prepared_test_material("проврка", [7; 32], &productive);
                    let mut lattice = CompositeL2LatticeV1::assemble(
                        &l11,
                        |_| Some("проверка".to_string()),
                        productive,
                        None,
                    )
                    .expect("singleton field");
                    lattice
                        .merge_exact_peak_surfaces(
                            std::iter::empty(),
                            overflow.then(|| {
                                EnumerationCompletenessV1::overflow(
                                    0,
                                    raw_count,
                                    IncompletenessReasonV1::StorageCapacity,
                                    [79, 83],
                                )
                            }),
                        )
                        .expect("bounded exact projection");
                    let field = PreparedCanonicalTokenField::from_lattice(
                        "проврка",
                        CanonicalContourProvenance::default(),
                        [7; 32],
                        lattice,
                        material,
                        PreparedFieldMaterialScopeV1::ContextNeutral,
                    );
                    assert_eq!(field.replacement_lattice_surfaces(), vec!["проверка"]);
                    let expected_state = if overflow {
                        EnumerationStateV1::Overflow
                    } else {
                        EnumerationStateV1::Complete
                    };
                    assert_eq!(field.common_completeness().state(), expected_state);
                    if overflow {
                        assert_eq!(
                            usize::from(field.common_completeness().logical_count_lower_bound()),
                            raw_count
                        );
                        assert_eq!(
                            field.common_completeness().reason(),
                            IncompletenessReasonV1::StorageCapacity
                        );
                    }
                    let eligible = grounded_winner || !overflow;
                    assert_eq!(
                        matches!(field.legacy_authority(), L2FieldAuthority::Winner { surface } if surface == "проверка"),
                        eligible
                    );
                    let candidates =
                        materialize_live_productive_v1_field("проврка ", "проврка", &field)
                            .expect("real candidate projection")
                            .candidates;
                    assert_eq!(candidates.len(), 1);
                    assert_eq!(candidates[0].replacement, "проверка ");
                    assert_eq!(
                        candidates[0].gate.action,
                        if eligible {
                            CandidateGateAction::Eligible
                        } else {
                            CandidateGateAction::SuggestOnly
                        }
                    );
                    assert!(candidates[0].frame_bound_lexical_capability().is_none());
                    if grounded_winner {
                        assert_eq!(field.replacement_grounded_l11_surfaces(), vec!["проверка"]);
                    }
                    if !eligible {
                        let event = TypingErrorEvent {
                            original: "проврка ".into(),
                            core: "проврка".into(),
                            current_word: "проврка".into(),
                            input_class: TypingErrorClass::MissingLetter,
                        };
                        for correction_safety in [
                            crate::config::CorrectionSafety::Normal,
                            crate::config::CorrectionSafety::Strict,
                            crate::config::CorrectionSafety::Experimental,
                        ] {
                            let decision =
                                TransitionDecisionCore::evaluate_candidates_with_authority_context(
                                    &event,
                                    &candidates,
                                    TransitionDecisionPolicy {
                                        l2_phase_apply: false,
                                        correction_safety,
                                    },
                                    DecisionEvidenceMode::FullField(None),
                                    &Default::default(),
                                );
                            assert_eq!(decision.selected_index, None);
                            assert!(decision.evaluations[0].action.verifier_passed);
                        }
                    }
                }
            }
        }
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
        let package_tuple = ExactPackageTupleV1 {
            l11_sha256: [7; 32],
            canonical_l2_sha256: [7; 32],
            productive_sha256: [7; 32],
        };
        let enumeration = ContextNeutralProductiveEnumerationV1 {
            readout: productive.clone(),
            productive_work: EnumerationWorkCountersV1::default(),
            aggregate_work: EnumerationWorkCountersV1::default(),
            work_budget_exceeded: false,
        };
        let (nominative_birth, nominative_proof) =
            test_anchor_birth("форм", "форма", VerdictMembershipV1::Grounded, 1_000);
        let (genitive_birth, genitive_proof) =
            test_anchor_birth("форм", "формы", VerdictMembershipV1::Grounded, 1_000);
        let mut anchor_proofs = vec![nominative_proof, genitive_proof];
        anchor_proofs.sort_by_key(|proof| proof.proof_ref);
        let contour_births = TypedContourBirthEnumerationV1 {
            births: vec![nominative_birth, genitive_birth],
            canonical_l1_anchor_proofs: anchor_proofs,
            work: EnumerationWorkCountersV1::default(),
            logical_match_count: 2,
            all_seen_digest: [17, 18],
            overflow_reason: None,
        };
        let prepared_material = prepare_context_neutral_productive_material_with_contours(
            "форм",
            package_tuple,
            enumeration.clone(),
            contour_births.clone(),
        )
        .expect("complete tied display material");
        let authority_material = prepare_frame_bound_lexical_authority_material(
            "форм",
            package_tuple,
            enumeration,
            contour_births,
            certified_exact_peaks("форм", &["форма", "формы"], [7; 32]),
        )
        .expect("complete tied relation-partition material");
        let lattice = CompositeL2LatticeV1::assemble(&l11, |_| None, productive, None)
            .expect("two-slot productive lattice");
        let field = PreparedCanonicalTokenField::from_lattice_with_authority_material(
            "форм",
            CanonicalContourProvenance::default(),
            [7; 32],
            lattice,
            prepared_material,
            authority_material,
            PreparedFieldMaterialScopeV1::ContextNeutral,
        );
        let authority_before = field.legacy_authority().clone();
        let frame = lexical_frame("нужна ", "форм", true);

        let compare = super::super::cohort_compare::compare_shared_canonical_cohort(
            &field,
            Some(&frame),
            91,
            test_anchor_replay(),
        );

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
            test_anchor_replay(),
        );
        assert_eq!(
            context_shaped_compare.material_scope,
            PreparedFieldMaterialScopeV1::ContextShapedObservation
        );
        assert!(!context_shaped_compare.complete_for_authority);
        assert_eq!(context_shaped.legacy_authority(), &authority_before);
    }

    #[test]
    fn td117_complete_singleton_issues_one_frame_bound_lexical_capability() {
        let field = std::sync::Arc::new(td117_singleton_field(certified_exact_peaks(
            "форм",
            &["форма"],
            [7; 32],
        )));
        let frame = lexical_frame("нужна ", "форм", true);

        let settled = super::super::cohort_compare::settle_shared_canonical_cohort(
            field,
            Some(&frame),
            91,
            "нужна форм ",
            test_anchor_replay(),
        );

        assert!(settled.authority_context.is_certified());
        assert_eq!(settled.capability_count(), 1);
        assert_eq!(settled.winner_replacement(), Some("нужна форма "));
    }

    #[test]
    fn td117_unresolved_or_mismatched_exact_search_proof_issues_zero_capabilities() {
        let unresolved = td117_singleton_field(ExactPeakBirthEnumerationV1::incomplete(
            crate::typing_transition::target_evidence::IncompletenessReasonV1::WorkBudgetExceeded,
        ));
        let package_mismatch =
            td117_singleton_field(certified_exact_peaks("форм", &["форма"], [8; 32]));
        let mut sidecar_mismatch =
            td117_singleton_field(certified_exact_peaks("форм", &["форма"], [7; 32]));
        sidecar_mismatch
            .authority_material
            .corrupt_exact_search_proof_for_test(ExactSearchProofFaultV1::Sidecar);
        let mut semantics_mismatch =
            td117_singleton_field(certified_exact_peaks("форм", &["форма"], [7; 32]));
        semantics_mismatch
            .authority_material
            .corrupt_exact_search_proof_for_test(ExactSearchProofFaultV1::Semantics);
        let mut package_tamper =
            td117_singleton_field(certified_exact_peaks("форм", &["форма"], [7; 32]));
        package_tamper
            .authority_material
            .corrupt_exact_search_proof_for_test(ExactSearchProofFaultV1::CanonicalPackage);
        let frame = lexical_frame("нужна ", "форм", true);

        for (case, field) in [
            ("unresolved", unresolved),
            ("package-mismatch", package_mismatch),
            ("sidecar-mismatch", sidecar_mismatch),
            ("semantics-mismatch", semantics_mismatch),
            ("package-tamper", package_tamper),
        ] {
            let settled = super::super::cohort_compare::settle_shared_canonical_cohort(
                std::sync::Arc::new(field),
                Some(&frame),
                91,
                "нужна форм ",
                test_anchor_replay(),
            );
            assert_eq!(settled.capability_count(), 0, "{case}");
            assert!(!settled.authority_context.is_certified(), "{case}");
        }
    }

    #[test]
    fn td117_display_only_surface_is_missing_from_authority_material() {
        let neutral_target = productive_candidate(17, 1, 101, "форма");
        let display_only_target = productive_candidate(18, 2, 102, "формы");
        let display_productive = PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Winner {
                candidate: readout_candidate(&neutral_target),
                calibration_stratum_id: 1,
            },
            candidates: vec![neutral_target.clone(), display_only_target],
            logical_terminal_count: 2,
            logical_surface_basin_count: 2,
            integrity_error: None,
        };
        let neutral_productive = PackagedProductiveReadoutV1 {
            verdict: ProductiveCalibratedVerdictV1::Winner {
                candidate: readout_candidate(&neutral_target),
                calibration_stratum_id: 1,
            },
            candidates: vec![neutral_target],
            logical_terminal_count: 1,
            logical_surface_basin_count: 1,
            integrity_error: None,
        };
        let l11 = RestorationReadout::Abstain {
            reason: AbstainReason::NoCandidates,
            geometry_distance: None,
            candidates: Vec::new(),
        };
        let lattice =
            CompositeL2LatticeV1::assemble(&l11, |_| None, display_productive.clone(), None)
                .expect("two-surface display lattice");
        let package_tuple = ExactPackageTupleV1 {
            l11_sha256: [7; 32],
            canonical_l2_sha256: [7; 32],
            productive_sha256: [7; 32],
        };
        let prepared_material = prepared_test_material("форм", [7; 32], &display_productive);
        let authority_material = prepare_frame_bound_lexical_authority_material(
            "форм",
            package_tuple,
            ContextNeutralProductiveEnumerationV1 {
                readout: neutral_productive,
                productive_work: EnumerationWorkCountersV1::default(),
                aggregate_work: EnumerationWorkCountersV1::default(),
                work_budget_exceeded: false,
            },
            TypedContourBirthEnumerationV1 {
                births: vec![TypedContourBirthV1 {
                    normalized_surface: "форма".to_string(),
                    grounding_namespace: GroundingNamespaceV1::L11Terminal,
                    grounding_ref: 17,
                    relation: TargetRelationV1::L11Restoration,
                    operator_ref: 0x5348_0000,
                    derivation_ref: 0x117,
                    verdict_membership: VerdictMembershipV1::Grounded,
                    support_milli: 1_000,
                }],
                canonical_l1_anchor_proofs: Vec::new(),
                work: EnumerationWorkCountersV1::default(),
                logical_match_count: 1,
                all_seen_digest: [17, 117],
                overflow_reason: None,
            },
            ExactPeakBirthEnumerationV1::complete_empty(),
        )
        .expect("one-target neutral authority material");
        let field = std::sync::Arc::new(
            PreparedCanonicalTokenField::from_lattice_with_authority_material(
                "форм",
                CanonicalContourProvenance::default(),
                [7; 32],
                lattice,
                prepared_material,
                authority_material,
                PreparedFieldMaterialScopeV1::ContextNeutral,
            ),
        );
        let frame = lexical_frame("нужна ", "форм", true);

        let settled = super::super::cohort_compare::settle_shared_canonical_cohort(
            field,
            Some(&frame),
            91,
            "нужна форм ",
            test_anchor_replay(),
        );

        assert_eq!(settled.capability_count(), 0);
        assert!(settled
            .compare
            .unretained_field_candidate_surfaces
            .iter()
            .any(|surface| surface == "формы"));
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
                canonical_l1_anchor_proofs: Vec::new(),
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

        let compare = super::super::cohort_compare::compare_shared_canonical_cohort(
            &field,
            Some(&frame),
            91,
            test_anchor_replay(),
        );

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

        let missing = super::super::cohort_compare::compare_shared_canonical_cohort(
            &field,
            None,
            91,
            test_anchor_replay(),
        );
        let no_coordinates = lexical_frame("нужна ", "форм", false);
        let incomplete = super::super::cohort_compare::compare_shared_canonical_cohort(
            &field,
            Some(&no_coordinates),
            91,
            test_anchor_replay(),
        );
        let wrong_token = lexical_frame("нужна ", "форма", true);
        let mismatch = super::super::cohort_compare::compare_shared_canonical_cohort(
            &field,
            Some(&wrong_token),
            91,
            test_anchor_replay(),
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
