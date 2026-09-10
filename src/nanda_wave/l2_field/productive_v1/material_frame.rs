//! Context-neutral target material and exact frame binding.
//!
//! It owns immutable material/proof construction and replay used by the live
//! capability issuer. It has no daemon, IBus, display, mutation, cache, or
//! package-writing entrypoint and grants no authority by itself.

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::nanda_wave::lexical_grokking::{
    phase7d_semantics_digest, Phase7dCertificateClass, Phase7dCertificateEvidence,
    Phase7dCertificateOracle, Phase7dRetrievalLane,
};
use crate::text_case::apply_word_case;
#[cfg(test)]
use crate::typing_transition::target_evidence::CanonicalL1AnchorKindV1;
use crate::typing_transition::target_evidence::{
    stable_bytes_ref, BoundedTargetSetV1, CanonicalL1AnchorProofV1, CompletenessScopeV1,
    EnumerationCompletenessV1, EnumerationStateV1, EnumerationWorkBudgetV1,
    EnumerationWorkCountersV1, ExactRelationPartitionProofV1, FrameInvalidationReasonV1,
    FrameTargetIdentityV1, GroundingNamespaceV1, IncompletenessReasonV1, InputFrameIdentityV1,
    LeaseConsumerStateV1, MaterialTargetIdentityV1, NormalizationLayoutProfileIdV1,
    PreparedEvidenceTablesV1, PreparedIntegrityV1, PreparedMaterialKeyV1, PreparedMaterialLeaseV1,
    PreparedOriginalLexicalStatusV1, PreparedOriginalMaterialV1,
    PreparedOriginalPunctuationStatusV1, PreparedOriginalScriptTokenStatusV1,
    PreparedTargetMaterialV1, PreparedTargetV1, ReplacementSpanV1, SeparatorProfileIdV1,
    TargetEvidenceSetV1, TargetRelationV1, TargetWitnessV1, VerdictMembershipV1,
    MAX_LEASE_CONSUMERS_PER_FIELD, MAX_PINNED_PREPARED_FIELDS, MAX_TARGETS_PER_FIELD,
    MAX_TARGET_WITNESSES_PER_TARGET,
};

use super::boundary_birth::{
    CompositeBoundaryGroundingV1, TypedBoundaryBirthEnumerationV1, TypedBoundaryBirthV1,
    ASCII_SPACE_SEPARATOR_PROFILE,
};
use super::contour_birth::{TypedContourBirthEnumerationV1, TypedContourBirthV1};
use super::packaged_runtime::{
    ContextNeutralProductiveEnumerationV1, PackagedProductiveCandidateV1,
};

const MAX_CONTOUR_TARGETS_PER_FIELD: usize = 8;
const MAX_BOUNDARY_TARGETS_PER_FIELD: usize = 2;
const EXACT_PEAK_OPERATOR_BASE: u32 = 0x5631_0000;
const RELATION_PARTITION_POLICY_VERSION_V1: u32 = 2;

pub(in crate::nanda_wave::l2_field) const EXACT_RELATION_SEARCH_MAX_PRODUCT_STATES: usize = 100_000;
pub(in crate::nanda_wave::l2_field) const EXACT_RELATION_SEARCH_MAX_TERMINALS: usize = 16_384;
pub(in crate::nanda_wave::l2_field) const EXACT_RELATION_SEARCH_MAX_SCRATCH_BYTES: usize =
    512 * 1024;
pub(in crate::nanda_wave::l2_field) const EXACT_RELATION_SEARCH_MAX_LANES: usize = 2;

pub(super) const FROZEN_V90_ENUMERATION_WORK_BUDGET: EnumerationWorkBudgetV1 =
    EnumerationWorkBudgetV1 {
        canonical_grounding: EnumerationWorkCountersV1 {
            posting_visits: 0,
            relation_replays: 0,
            grounding_lookups: 256,
            generated_logical_targets: 0,
            operator_steps: 0,
        },
        cold_binding: EnumerationWorkCountersV1 {
            posting_visits: 131_072,
            relation_replays: 131_072,
            grounding_lookups: 0,
            generated_logical_targets: 0,
            operator_steps: 524_288,
        },
        productive_traversal: EnumerationWorkCountersV1 {
            posting_visits: 0,
            relation_replays: 8_192,
            grounding_lookups: 0,
            generated_logical_targets: 8_192,
            operator_steps: 16_384,
        },
        aggregate: EnumerationWorkCountersV1 {
            posting_visits: 131_072,
            relation_replays: 131_072,
            grounding_lookups: 256,
            generated_logical_targets: 8_192,
            operator_steps: 524_288,
        },
    };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ExactPackageTupleV1 {
    pub(super) l11_sha256: [u8; 32],
    pub(super) canonical_l2_sha256: [u8; 32],
    pub(super) productive_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum ExactPeakCertificateClassV1 {
    Identity = 1,
    PunctuationSuffix = 2,
    PrefixTruncation = 3,
    SuffixTruncation = 4,
    MissingLetter = 5,
    ExtraLetter = 6,
    Substitution = 7,
    KeyboardLayout = 8,
    AdjacentTransposition = 9,
    NonAdjacentTransposition = 10,
    RepeatedFragment = 11,
    SparseMultiOmission = 12,
    OmissionTransposition = 13,
}

impl ExactPeakCertificateClassV1 {
    const fn label(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::PunctuationSuffix => "punctuation_suffix",
            Self::PrefixTruncation => "prefix_truncation",
            Self::SuffixTruncation => "suffix_truncation",
            Self::MissingLetter => "missing_letter",
            Self::ExtraLetter => "extra_letter",
            Self::Substitution => "substitution",
            Self::KeyboardLayout => "keyboard_layout",
            Self::AdjacentTransposition => "adjacent_transposition",
            Self::NonAdjacentTransposition => "non_adjacent_transposition",
            Self::RepeatedFragment => "repeated_fragment",
            Self::SparseMultiOmission => "sparse_multi_omission",
            Self::OmissionTransposition => "omission_transposition",
        }
    }

    const fn relation(self) -> TargetRelationV1 {
        match self {
            Self::Identity | Self::PunctuationSuffix => TargetRelationV1::L11Restoration,
            Self::PrefixTruncation | Self::SuffixTruncation | Self::MissingLetter => {
                TargetRelationV1::MissingLetter
            }
            Self::ExtraLetter => TargetRelationV1::ExtraLetter,
            Self::Substitution => TargetRelationV1::Substitution,
            Self::KeyboardLayout => TargetRelationV1::ExactLayout,
            Self::AdjacentTransposition => TargetRelationV1::AdjacentTransposition,
            Self::NonAdjacentTransposition => TargetRelationV1::NonAdjacentTransposition,
            Self::RepeatedFragment => TargetRelationV1::RepeatedFragment,
            Self::SparseMultiOmission | Self::OmissionTransposition => {
                TargetRelationV1::SparseOmission
            }
        }
    }

    const fn operator_ref(self) -> u32 {
        EXACT_PEAK_OPERATOR_BASE | self as u32
    }

    const fn phase7d_class(self) -> Phase7dCertificateClass {
        match self {
            Self::Identity => Phase7dCertificateClass::Identity,
            Self::PunctuationSuffix => Phase7dCertificateClass::PunctuationSuffix,
            Self::PrefixTruncation => Phase7dCertificateClass::PrefixTruncation,
            Self::SuffixTruncation => Phase7dCertificateClass::SuffixTruncation,
            Self::MissingLetter => Phase7dCertificateClass::MissingLetter,
            Self::ExtraLetter => Phase7dCertificateClass::ExtraLetter,
            Self::Substitution => Phase7dCertificateClass::Substitution,
            Self::KeyboardLayout => Phase7dCertificateClass::KeyboardLayout,
            Self::AdjacentTransposition => Phase7dCertificateClass::AdjacentTransposition,
            Self::NonAdjacentTransposition => Phase7dCertificateClass::NonAdjacentTransposition,
            Self::RepeatedFragment => Phase7dCertificateClass::RepeatedFragment,
            Self::SparseMultiOmission => Phase7dCertificateClass::SparseMultiOmission,
            Self::OmissionTransposition => Phase7dCertificateClass::OmissionTransposition,
        }
    }
}

impl From<Phase7dCertificateClass> for ExactPeakCertificateClassV1 {
    fn from(value: Phase7dCertificateClass) -> Self {
        match value {
            Phase7dCertificateClass::Identity => Self::Identity,
            Phase7dCertificateClass::PunctuationSuffix => Self::PunctuationSuffix,
            Phase7dCertificateClass::PrefixTruncation => Self::PrefixTruncation,
            Phase7dCertificateClass::SuffixTruncation => Self::SuffixTruncation,
            Phase7dCertificateClass::MissingLetter => Self::MissingLetter,
            Phase7dCertificateClass::ExtraLetter => Self::ExtraLetter,
            Phase7dCertificateClass::Substitution => Self::Substitution,
            Phase7dCertificateClass::KeyboardLayout => Self::KeyboardLayout,
            Phase7dCertificateClass::AdjacentTransposition => Self::AdjacentTransposition,
            Phase7dCertificateClass::NonAdjacentTransposition => Self::NonAdjacentTransposition,
            Phase7dCertificateClass::RepeatedFragment => Self::RepeatedFragment,
            Phase7dCertificateClass::SparseMultiOmission => Self::SparseMultiOmission,
            Phase7dCertificateClass::OmissionTransposition => Self::OmissionTransposition,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct ExactPeakCandidateInputV1 {
    pub(in crate::nanda_wave::l2_field) form_ref: u32,
    pub(in crate::nanda_wave::l2_field) normalized_surface: String,
    pub(in crate::nanda_wave::l2_field) certificates: Vec<Phase7dCertificateEvidence>,
}

/// The only Phase-7D geometries admitted into the TD-117 authority partition.
/// Both exact-search materialization and frame-time witness replay call this
/// predicate, so the producer and consumer cannot silently drift.
pub(super) fn supported_phase7d_relation(
    evidence: &Phase7dCertificateEvidence,
) -> Option<TargetRelationV1> {
    match evidence.class {
        Phase7dCertificateClass::PrefixTruncation
        | Phase7dCertificateClass::SuffixTruncation
        | Phase7dCertificateClass::MissingLetter => Some(TargetRelationV1::MissingLetter),
        Phase7dCertificateClass::ExtraLetter => Some(TargetRelationV1::ExtraLetter),
        Phase7dCertificateClass::Substitution
            if evidence
                .canonical_key
                .contains("\"kind\":\"single_substitution\"") =>
        {
            Some(TargetRelationV1::Substitution)
        }
        Phase7dCertificateClass::KeyboardLayout => Some(TargetRelationV1::ExactLayout),
        Phase7dCertificateClass::AdjacentTransposition => {
            Some(TargetRelationV1::AdjacentTransposition)
        }
        Phase7dCertificateClass::Identity
        | Phase7dCertificateClass::PunctuationSuffix
        | Phase7dCertificateClass::Substitution
        | Phase7dCertificateClass::NonAdjacentTransposition
        | Phase7dCertificateClass::RepeatedFragment
        | Phase7dCertificateClass::SparseMultiOmission
        | Phase7dCertificateClass::OmissionTransposition => None,
    }
}

pub(in crate::nanda_wave::l2_field) fn exact_relation_search_budget_identity() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-td117-exact-relation-search-budget-v2\0");
    hasher.update((EXACT_RELATION_SEARCH_MAX_LANES as u64).to_le_bytes());
    hasher.update((EXACT_RELATION_SEARCH_MAX_PRODUCT_STATES as u64).to_le_bytes());
    hasher.update((EXACT_RELATION_SEARCH_MAX_TERMINALS as u64).to_le_bytes());
    hasher.update((EXACT_RELATION_SEARCH_MAX_SCRATCH_BYTES as u64).to_le_bytes());
    // Proof search has no elapsed deadline and uses the canonical forward,
    // non-verification traversal schedule.
    hasher.update([0, 0, 0]);
    hasher.finalize().into()
}

fn exact_search_work_within_budget(
    work: ExactSearchStructuralWorkV1,
    expected_lane_count: usize,
    retrieved_form_count: usize,
) -> bool {
    let Some(expected_lane_count) = u64::try_from(expected_lane_count).ok() else {
        return false;
    };
    let Some(retrieved_form_count) = u64::try_from(retrieved_form_count).ok() else {
        return false;
    };
    let Some(maximum_expanded_product_states) = work
        .maximum_lane_product_states
        .checked_mul(expected_lane_count)
    else {
        return false;
    };
    let Some(maximum_retrieved_forms) =
        work.maximum_lane_terminals.checked_mul(expected_lane_count)
    else {
        return false;
    };
    work.lane_count == expected_lane_count
        && work.lane_count <= EXACT_RELATION_SEARCH_MAX_LANES as u64
        && work.maximum_lane_product_states <= EXACT_RELATION_SEARCH_MAX_PRODUCT_STATES as u64
        && work.maximum_lane_terminals <= EXACT_RELATION_SEARCH_MAX_TERMINALS as u64
        && work.maximum_lane_product_states <= work.expanded_product_states
        && work.expanded_product_states <= maximum_expanded_product_states
        && work.maximum_lane_terminals <= retrieved_form_count
        && retrieved_form_count <= maximum_retrieved_forms
        && work.maximum_scratch_bytes <= EXACT_RELATION_SEARCH_MAX_SCRATCH_BYTES as u64
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct ExactSearchStructuralWorkV1 {
    pub(in crate::nanda_wave::l2_field) lane_count: u64,
    pub(in crate::nanda_wave::l2_field) maximum_lane_product_states: u64,
    pub(in crate::nanda_wave::l2_field) maximum_lane_terminals: u64,
    pub(in crate::nanda_wave::l2_field) expanded_product_states: u64,
    pub(in crate::nanda_wave::l2_field) maximum_scratch_bytes: u64,
    pub(in crate::nanda_wave::l2_field) expanded_states: u64,
    pub(in crate::nanda_wave::l2_field) examined_edges: u64,
    pub(in crate::nanda_wave::l2_field) surviving_edges: u64,
    pub(in crate::nanda_wave::l2_field) pruned_edges: u64,
    pub(in crate::nanda_wave::l2_field) stack_pushes: u64,
    pub(in crate::nanda_wave::l2_field) stack_pops: u64,
    pub(in crate::nanda_wave::l2_field) terminal_hits: u64,
    pub(in crate::nanda_wave::l2_field) transition_checks: u64,
    pub(in crate::nanda_wave::l2_field) terminal_distance_checks: u64,
    pub(in crate::nanda_wave::l2_field) rank_prefix_count: u64,
    pub(in crate::nanda_wave::l2_field) terminal_rank_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct ExactPeakSearchProofV1 {
    exact_observed_sha256: [u8; 32],
    query_lanes_sha256: [u8; 32],
    canonical_l2_sha256: [u8; 32],
    sidecar_sha256: [u8; 32],
    phase7d_semantics_sha256: [u8; 32],
    search_budget_sha256: [u8; 32],
    retrieved_form_refs_sha256: [u8; 32],
    supported_births_sha256: [u8; 32],
    structural_trace_sha256: [u8; 32],
    retrieved_form_count: u32,
    supported_target_count: u16,
    supported_certificate_count: u16,
    work: ExactSearchStructuralWorkV1,
}

impl ExactPeakSearchProofV1 {
    fn canonical_digest(&self) -> [u64; 2] {
        let mut hasher = Sha256::new();
        hasher.update(b"lay-td117-exact-peak-search-proof-v2\0");
        for digest in [
            self.exact_observed_sha256,
            self.query_lanes_sha256,
            self.canonical_l2_sha256,
            self.sidecar_sha256,
            self.phase7d_semantics_sha256,
            self.search_budget_sha256,
            self.retrieved_form_refs_sha256,
            self.supported_births_sha256,
            self.structural_trace_sha256,
        ] {
            hasher.update(digest);
        }
        hasher.update(self.retrieved_form_count.to_le_bytes());
        hasher.update(self.supported_target_count.to_le_bytes());
        hasher.update(self.supported_certificate_count.to_le_bytes());
        for value in exact_search_work_values(self.work) {
            hasher.update(value.to_le_bytes());
        }
        digest128(hasher.finalize().into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ExactPeakBirthV1 {
    form_ref: u32,
    normalized_surface: String,
    class: ExactPeakCertificateClassV1,
    canonical_key: String,
    derivation_ref: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct ExactPeakBirthEnumerationV1 {
    births: Vec<ExactPeakBirthV1>,
    retrieved_form_refs: Vec<u32>,
    search_proof: Option<ExactPeakSearchProofV1>,
    work: EnumerationWorkCountersV1,
    logical_match_count: usize,
    all_seen_digest: [u64; 2],
    overflow_reason: Option<IncompletenessReasonV1>,
}

impl ExactPeakBirthEnumerationV1 {
    pub(in crate::nanda_wave::l2_field) fn complete_empty() -> Self {
        Self {
            births: Vec::new(),
            retrieved_form_refs: Vec::new(),
            search_proof: None,
            work: EnumerationWorkCountersV1::default(),
            logical_match_count: 0,
            all_seen_digest: digest128(Sha256::digest(b"lay-exact-peak-birth-v1\0").into()),
            overflow_reason: None,
        }
    }

    pub(in crate::nanda_wave::l2_field) fn incomplete(reason: IncompletenessReasonV1) -> Self {
        debug_assert!(reason != IncompletenessReasonV1::None);
        Self {
            births: Vec::new(),
            retrieved_form_refs: Vec::new(),
            search_proof: None,
            work: EnumerationWorkCountersV1::default(),
            logical_match_count: 0,
            all_seen_digest: digest128(
                Sha256::digest(b"lay-exact-peak-birth-v1\0incomplete").into(),
            ),
            overflow_reason: Some(reason),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.births.is_empty()
    }

    pub(in crate::nanda_wave::l2_field) fn capacity_exceeded(&self) -> bool {
        self.overflow_reason == Some(IncompletenessReasonV1::StorageCapacity)
    }

    pub(super) fn common_field_projection(
        self,
        observed: &str,
        canonical_l2_sha256: [u8; 32],
    ) -> (Self, Option<EnumerationCompletenessV1>) {
        if !self.capacity_exceeded() {
            return (self, None);
        }
        let mut projection = self
            .authority_partition(observed, canonical_l2_sha256)
            .unwrap_or_else(|| Self::incomplete(IncompletenessReasonV1::StorageCapacity));
        let incompleteness = EnumerationCompletenessV1::overflow(
            projection.logical_match_count,
            self.logical_match_count,
            IncompletenessReasonV1::StorageCapacity,
            self.all_seen_digest,
        );
        // The common field retains a bounded view, not another search proof.
        // Raw identities keep its incompleteness visible and prevent reissuance.
        projection.search_proof = None;
        projection.logical_match_count = self.logical_match_count;
        projection.all_seen_digest = self.all_seen_digest;
        projection.work = self.work;
        projection.overflow_reason = Some(IncompletenessReasonV1::StorageCapacity);
        (projection, Some(incompleteness))
    }

    fn raw_identity_matches_held_births(&self) -> bool {
        self.logical_match_count == exact_peak_root_counts(&self.births).len()
            && self.all_seen_digest == exact_peak_set_digest(&self.births)
    }

    pub(super) fn normalized_surfaces(&self) -> impl Iterator<Item = &str> {
        self.births
            .iter()
            .map(|birth| birth.normalized_surface.as_str())
    }

    pub(super) fn validated_authority_target_rows(
        &self,
        observed: &str,
        canonical_l2_sha256: [u8; 32],
    ) -> Option<Vec<(u32, String)>> {
        let partition = self.authority_partition(observed, canonical_l2_sha256)?;
        let rows = partition
            .births
            .into_iter()
            .map(|birth| (birth.form_ref, birth.normalized_surface))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let maximum_rows = MAX_TARGETS_PER_FIELD.checked_mul(MAX_TARGET_WITNESSES_PER_TARGET)?;
        (rows.len() <= maximum_rows).then_some(rows)
    }

    pub(in crate::nanda_wave::l2_field) fn diagnostic_json(&self) -> serde_json::Value {
        let candidates = self
            .births
            .iter()
            .map(|birth| (birth.form_ref, birth.normalized_surface.as_str()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|(form_ref, normalized_surface)| {
                serde_json::json!({
                    "form_ref": form_ref,
                    "normalized_surface": normalized_surface,
                })
            })
            .collect::<Vec<_>>();
        let certificates = self
            .births
            .iter()
            .map(|birth| {
                serde_json::json!({
                    "form_ref": birth.form_ref,
                    "normalized_surface": birth.normalized_surface,
                    "class": birth.class.label(),
                    "canonical_key": birth.canonical_key,
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "status": if self.overflow_reason.is_none() { "complete" } else { "incomplete" },
            "incompleteness_reason": self.overflow_reason.map(|reason| format!("{reason:?}")),
            "candidate_count": candidates.len(),
            "certificate_count": certificates.len(),
            "logical_match_count": self.logical_match_count,
            "all_seen_digest": self.all_seen_digest,
            "work": {
                "posting_visits": self.work.posting_visits,
                "relation_replays": self.work.relation_replays,
                "grounding_lookups": self.work.grounding_lookups,
                "generated_logical_targets": self.work.generated_logical_targets,
                "operator_steps": self.work.operator_steps,
            },
            "candidates": candidates,
            "certificates": certificates,
        })
    }

    pub(in crate::nanda_wave::l2_field) fn from_candidates(
        candidates: Vec<ExactPeakCandidateInputV1>,
    ) -> Result<Self, String> {
        let mut form_surfaces = BTreeMap::<u32, String>::new();
        let mut certificate_refs = BTreeMap::<u32, String>::new();
        let mut births = BTreeSet::<ExactPeakBirthV1>::new();
        let mut retrieved_form_refs = BTreeSet::<u32>::new();
        for candidate in candidates {
            let normalized_surface =
                super::super::compositional::normalize_surface(&candidate.normalized_surface);
            if normalized_surface.is_empty() || candidate.certificates.is_empty() {
                return Err("exact peak candidate has an empty surface or certificate set".into());
            }
            if form_surfaces
                .insert(candidate.form_ref, normalized_surface.clone())
                .is_some_and(|retained| retained != normalized_surface)
            {
                return Err("exact peak form_ref maps to multiple normalized surfaces".into());
            }
            retrieved_form_refs.insert(candidate.form_ref);
            for certificate in candidate.certificates {
                if certificate.canonical_key.is_empty() {
                    return Err("exact peak certificate has an empty canonical key".into());
                }
                let derivation_ref = stable_bytes_ref(certificate.canonical_key.as_bytes());
                if certificate_refs
                    .insert(derivation_ref, certificate.canonical_key.clone())
                    .is_some_and(|retained| retained != certificate.canonical_key)
                {
                    return Err("exact peak canonical certificate reference collision".into());
                }
                births.insert(ExactPeakBirthV1 {
                    form_ref: candidate.form_ref,
                    normalized_surface: normalized_surface.clone(),
                    class: certificate.class.into(),
                    canonical_key: certificate.canonical_key,
                    derivation_ref,
                });
            }
        }
        let births = births.into_iter().collect::<Vec<_>>();
        let roots_by_surface = exact_peak_root_counts(&births);
        let logical_match_count = roots_by_surface.len();
        let capacity_exceeded = logical_match_count > MAX_TARGETS_PER_FIELD
            || roots_by_surface
                .values()
                .any(|count| *count > MAX_TARGET_WITNESSES_PER_TARGET);
        Ok(Self {
            work: EnumerationWorkCountersV1 {
                grounding_lookups: u64::try_from(form_surfaces.len()).unwrap_or(u64::MAX),
                generated_logical_targets: u64::try_from(logical_match_count).unwrap_or(u64::MAX),
                operator_steps: u64::try_from(births.len()).unwrap_or(u64::MAX),
                ..EnumerationWorkCountersV1::default()
            },
            all_seen_digest: exact_peak_set_digest(&births),
            overflow_reason: capacity_exceeded.then_some(IncompletenessReasonV1::StorageCapacity),
            births,
            retrieved_form_refs: retrieved_form_refs.into_iter().collect(),
            search_proof: None,
            logical_match_count,
        })
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the exact search proof keeps every producer identity explicit"
    )]
    pub(in crate::nanda_wave::l2_field) fn with_completed_search_proof(
        mut self,
        observed: &str,
        canonical_l2_sha256: [u8; 32],
        sidecar_sha256: [u8; 32],
        query_lanes: &[Phase7dRetrievalLane],
        mut retrieved_form_refs: Vec<u32>,
        work: ExactSearchStructuralWorkV1,
        rank_prefixes: &[u32],
        terminal_ranks: &[u32],
    ) -> Result<Self, String> {
        if (self.overflow_reason.is_some() && !self.capacity_exceeded())
            || sidecar_sha256 == [0; 32]
            || !self.raw_identity_matches_held_births()
        {
            return Err("incomplete exact peaks cannot own a completed search proof".to_string());
        }
        retrieved_form_refs.sort_unstable();
        retrieved_form_refs.dedup();
        if !exact_search_work_within_budget(work, query_lanes.len(), retrieved_form_refs.len()) {
            return Err("incomplete exact peaks cannot own a completed search proof".to_string());
        }
        if self
            .births
            .iter()
            .any(|birth| retrieved_form_refs.binary_search(&birth.form_ref).is_err())
        {
            return Err("exact peak birth is absent from the exhaustive terminal set".to_string());
        }
        let supported_births = replayed_supported_births(observed, &self.births)?;
        let supported_roots = exact_peak_root_counts(&supported_births);
        let supported_target_count = supported_roots.len();
        if supported_target_count > MAX_TARGETS_PER_FIELD
            || supported_roots
                .values()
                .any(|count| *count > MAX_TARGET_WITNESSES_PER_TARGET)
        {
            return Err("supported exact partition exceeds storage capacity".to_string());
        }
        let proof = ExactPeakSearchProofV1 {
            exact_observed_sha256: Sha256::digest(observed.as_bytes()).into(),
            query_lanes_sha256: query_lanes_digest(query_lanes),
            canonical_l2_sha256,
            sidecar_sha256,
            phase7d_semantics_sha256: phase7d_semantics_digest(),
            search_budget_sha256: exact_relation_search_budget_identity(),
            retrieved_form_refs_sha256: u32_sequence_digest(
                b"lay-td117-retrieved-form-refs-v1\0",
                &retrieved_form_refs,
            ),
            supported_births_sha256: exact_peak_set_sha256(&supported_births),
            structural_trace_sha256: structural_trace_digest(rank_prefixes, terminal_ranks),
            retrieved_form_count: u32::try_from(retrieved_form_refs.len())
                .map_err(|_| "exact search terminal count exceeds u32".to_string())?,
            supported_target_count: u16::try_from(supported_target_count)
                .map_err(|_| "supported exact target count exceeds u16".to_string())?,
            supported_certificate_count: u16::try_from(supported_births.len())
                .map_err(|_| "supported exact certificate count exceeds u16".to_string())?,
            work,
        };
        self.retrieved_form_refs = retrieved_form_refs;
        self.search_proof = Some(proof);
        Ok(self)
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn with_test_search_proof(
        self,
        observed: &str,
        canonical_l2_sha256: [u8; 32],
    ) -> Result<Self, String> {
        let lanes = Phase7dCertificateOracle::new(observed)?.retrieval_lanes();
        let retrieved_form_refs = self.retrieved_form_refs.clone();
        let work = ExactSearchStructuralWorkV1 {
            lane_count: u64::try_from(lanes.len())
                .map_err(|_| "test exact search lane count exceeds u64")?,
            maximum_lane_terminals: u64::try_from(retrieved_form_refs.len())
                .map_err(|_| "test exact search terminal count exceeds u64")?,
            ..ExactSearchStructuralWorkV1::default()
        };
        self.with_completed_search_proof(
            observed,
            canonical_l2_sha256,
            [0x5a; 32],
            &lanes,
            retrieved_form_refs,
            work,
            &[],
            &[],
        )
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn with_corrupted_first_class_for_test(
        mut self,
        class: Phase7dCertificateClass,
    ) -> Self {
        if let Some(birth) = self.births.first_mut() {
            birth.class = class.into();
        }
        self
    }

    fn authority_partition(&self, observed: &str, canonical_l2_sha256: [u8; 32]) -> Option<Self> {
        let proof = self.search_proof.as_ref()?;
        let lanes = Phase7dCertificateOracle::new(observed)
            .ok()?
            .retrieval_lanes();
        let supported_births = replayed_supported_births(observed, &self.births).ok()?;
        let supported_roots = exact_peak_root_counts(&supported_births);
        let supported_target_count = supported_roots.len();
        if (self.overflow_reason.is_some() && !self.capacity_exceeded())
            || !self.raw_identity_matches_held_births()
            || supported_target_count > MAX_TARGETS_PER_FIELD
            || supported_roots
                .values()
                .any(|count| *count > MAX_TARGET_WITNESSES_PER_TARGET)
            || proof.exact_observed_sha256 != <[u8; 32]>::from(Sha256::digest(observed.as_bytes()))
            || proof.query_lanes_sha256 != query_lanes_digest(&lanes)
            || proof.canonical_l2_sha256 != canonical_l2_sha256
            || proof.sidecar_sha256 == [0; 32]
            || proof.phase7d_semantics_sha256 != phase7d_semantics_digest()
            || proof.search_budget_sha256 != exact_relation_search_budget_identity()
            || proof.retrieved_form_refs_sha256
                != u32_sequence_digest(
                    b"lay-td117-retrieved-form-refs-v1\0",
                    &self.retrieved_form_refs,
                )
            || usize::try_from(proof.retrieved_form_count).ok()
                != Some(self.retrieved_form_refs.len())
            || proof.supported_births_sha256 != exact_peak_set_sha256(&supported_births)
            || usize::from(proof.supported_target_count) != supported_target_count
            || usize::from(proof.supported_certificate_count) != supported_births.len()
            || proof.structural_trace_sha256 == [0; 32]
            || !exact_search_work_within_budget(
                proof.work,
                lanes.len(),
                self.retrieved_form_refs.len(),
            )
        {
            return None;
        }
        Some(Self {
            logical_match_count: supported_target_count,
            all_seen_digest: exact_peak_set_digest(&supported_births),
            births: supported_births,
            retrieved_form_refs: self.retrieved_form_refs.clone(),
            search_proof: Some(proof.clone()),
            work: self.work,
            overflow_reason: None,
        })
    }
}

fn exact_peak_root_counts(births: &[ExactPeakBirthV1]) -> BTreeMap<&str, usize> {
    let mut roots = BTreeMap::new();
    for birth in births {
        *roots.entry(birth.normalized_surface.as_str()).or_default() += 1;
    }
    roots
}

fn replayed_supported_births(
    observed: &str,
    births: &[ExactPeakBirthV1],
) -> Result<Vec<ExactPeakBirthV1>, String> {
    let oracle = Phase7dCertificateOracle::new(observed)?;
    let mut actual_by_surface = BTreeMap::<&str, Vec<Phase7dCertificateEvidence>>::new();
    let mut supported = Vec::new();
    for birth in births {
        let evidence = Phase7dCertificateEvidence {
            class: birth.class.phase7d_class(),
            canonical_key: birth.canonical_key.clone(),
        };
        let Some(relation) = supported_phase7d_relation(&evidence) else {
            continue;
        };
        let actual = match actual_by_surface.entry(birth.normalized_surface.as_str()) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(oracle.certificate_evidence(&birth.normalized_surface)?)
            }
        };
        if relation != birth.class.relation()
            || birth.derivation_ref != stable_bytes_ref(birth.canonical_key.as_bytes())
            || !actual.contains(&evidence)
        {
            return Err(
                "exact peak certificate does not replay under current Phase-7D semantics"
                    .to_string(),
            );
        }
        supported.push(birth.clone());
    }
    supported.sort();
    supported.dedup();
    Ok(supported)
}

fn query_lanes_digest(lanes: &[Phase7dRetrievalLane]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-td117-phase7d-query-lanes-v1\0");
    hasher.update((lanes.len() as u64).to_le_bytes());
    for lane in lanes {
        hasher.update([lane.maximum_levenshtein_distance]);
        hasher.update((lane.symbols.len() as u64).to_le_bytes());
        for symbol in lane.symbols.iter() {
            hasher.update(symbol.to_le_bytes());
        }
    }
    hasher.finalize().into()
}

fn u32_sequence_digest(domain: &[u8], values: &[u32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update((values.len() as u64).to_le_bytes());
    for value in values {
        hasher.update(value.to_le_bytes());
    }
    hasher.finalize().into()
}

fn structural_trace_digest(rank_prefixes: &[u32], terminal_ranks: &[u32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-td117-exact-search-structural-trace-v1\0");
    hasher.update(u32_sequence_digest(b"rank-prefixes\0", rank_prefixes));
    hasher.update(u32_sequence_digest(b"terminal-ranks\0", terminal_ranks));
    hasher.finalize().into()
}

const fn exact_search_work_values(work: ExactSearchStructuralWorkV1) -> [u64; 16] {
    [
        work.lane_count,
        work.maximum_lane_product_states,
        work.maximum_lane_terminals,
        work.expanded_product_states,
        work.maximum_scratch_bytes,
        work.expanded_states,
        work.examined_edges,
        work.surviving_edges,
        work.pruned_edges,
        work.stack_pushes,
        work.stack_pops,
        work.terminal_hits,
        work.transition_checks,
        work.terminal_distance_checks,
        work.rank_prefix_count,
        work.terminal_rank_count,
    ]
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ExactWitnessRootV1 {
    relation: TargetRelationV1,
    grounding_namespace: GroundingNamespaceV1,
    verdict_membership: VerdictMembershipV1,
    operator_ref: u32,
    grounding_ref: u32,
    derivation_ref: u32,
}

impl ExactWitnessRootV1 {
    fn compact(&self, support_milli: u16, annotations: u16) -> TargetWitnessV1 {
        TargetWitnessV1::new(
            self.relation,
            self.grounding_namespace,
            self.verdict_membership,
            0,
            self.operator_ref,
            self.grounding_ref,
            self.derivation_ref,
            support_milli,
            annotations,
        )
    }

    fn canonical_bytes(&self) -> [u8; 16] {
        let mut bytes = [0_u8; 16];
        bytes[0] = self.relation as u8;
        bytes[1] = self.grounding_namespace as u8;
        bytes[2] = self.verdict_membership as u8;
        bytes[4..8].copy_from_slice(&self.operator_ref.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.grounding_ref.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.derivation_ref.to_le_bytes());
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactMaterialTargetV1 {
    normalized_scalars: String,
    canonical_bytes: Vec<u8>,
    witness_roots: Vec<ExactWitnessRootV1>,
    compact: PreparedTargetV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) struct PreparedTargetMaterialShadowV1 {
    compact: PreparedTargetMaterialV1,
    exact_observed: String,
    package_tuple: ExactPackageTupleV1,
    exact_original: Option<ExactMaterialTargetV1>,
    exact_targets: Vec<ExactMaterialTargetV1>,
    canonical_l1_anchor_proofs: Vec<CanonicalL1AnchorProofV1>,
    exact_peak_births: Vec<ExactPeakBirthV1>,
    exact_retrieved_form_refs: Vec<u32>,
    exact_search_proof: Option<ExactPeakSearchProofV1>,
    boundary_groundings: Vec<CompositeBoundaryGroundingV1>,
    work: EnumerationWorkCountersV1,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) enum ExactSearchProofFaultV1 {
    CanonicalPackage,
    Sidecar,
    Semantics,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::l2_field) enum CanonicalL1AnchorProofFaultV1 {
    TargetBytes,
    TargetForm,
    AnchorForm,
    Lemma,
    Terminal,
    Kind,
    L11Package,
    CanonicalL2Package,
    Missing,
    Extra,
    Ambiguous,
    RawTerminalSubstitution,
}

impl PreparedTargetMaterialShadowV1 {
    pub(super) fn compact(&self) -> &PreparedTargetMaterialV1 {
        &self.compact
    }

    pub(super) fn exact_target_surfaces(&self) -> impl Iterator<Item = &str> {
        self.exact_targets
            .iter()
            .map(|target| target.normalized_scalars.as_str())
    }

    pub(super) fn exact_target_surface(&self, target_ref: usize) -> Option<&str> {
        self.exact_targets
            .get(target_ref)
            .map(|target| target.normalized_scalars.as_str())
    }

    pub(super) fn exact_witness_root_matches(
        &self,
        target_ref: usize,
        witness_ref: usize,
        witness: TargetWitnessV1,
    ) -> bool {
        let Some(exact_target) = self.exact_targets.get(target_ref) else {
            return false;
        };
        let Some(compact_target) = self.compact.targets.as_slice().get(target_ref) else {
            return false;
        };
        if exact_target.compact != *compact_target
            || stable_bytes_ref(exact_target.normalized_scalars.as_bytes())
                != compact_target.identity.normalized_scalars_ref
            || stable_bytes_ref(&exact_target.canonical_bytes)
                != compact_target.identity.canonical_bytes_ref
            || exact_target.normalized_scalars.as_bytes() != exact_target.canonical_bytes
        {
            return false;
        }
        let Some(compact_witness) = compact_target.witnesses.witnesses().get(witness_ref) else {
            return false;
        };
        let Some(exact_root) = exact_target.witness_roots.get(witness_ref) else {
            return false;
        };
        *compact_witness == witness
            && exact_root.compact(
                compact_witness.support_milli,
                compact_witness.provenance_annotations,
            ) == witness
    }

    pub(super) fn canonical_l1_anchor_proof_for_target_witness(
        &self,
        target_ref: usize,
        witness_ref: usize,
        witness: TargetWitnessV1,
    ) -> Option<&CanonicalL1AnchorProofV1> {
        if witness.grounding_namespace != GroundingNamespaceV1::CanonicalL1Anchor
            || !self.exact_witness_root_matches(target_ref, witness_ref, witness)
            || !self.canonical_l1_anchor_proof_table_is_valid()
        {
            return None;
        }
        let mut matching = self
            .canonical_l1_anchor_proofs
            .iter()
            .filter(|proof| proof.proof_ref == witness.grounding_ref);
        let proof = matching.next()?;
        matching.next().is_none().then_some(proof)
    }

    pub(super) fn original_canonical_l1_anchor_proof(&self) -> Option<&CanonicalL1AnchorProofV1> {
        if !self.canonical_l1_anchor_proof_table_is_valid() {
            return None;
        }
        let original = self.exact_original.as_ref()?;
        if original.canonical_bytes.as_slice() != self.exact_observed.as_bytes()
            || original.compact.witnesses.witnesses().len() != original.witness_roots.len()
        {
            return None;
        }
        let mut proof = None;
        for (root, witness) in original
            .witness_roots
            .iter()
            .zip(original.compact.witnesses.witnesses())
        {
            if root.grounding_namespace != GroundingNamespaceV1::CanonicalL1Anchor
                || !is_grounded_membership(root.verdict_membership)
            {
                continue;
            }
            if root.compact(witness.support_milli, witness.provenance_annotations) != *witness
                || proof.is_some()
            {
                return None;
            }
            let retained = self
                .canonical_l1_anchor_proofs
                .iter()
                .find(|candidate| candidate.proof_ref == root.grounding_ref)?;
            proof = Some(retained);
        }
        proof
    }

    fn canonical_l1_anchor_proof_table_is_valid(&self) -> bool {
        let roots = self
            .exact_original
            .iter()
            .chain(&self.exact_targets)
            .flat_map(|target| {
                target
                    .witness_roots
                    .iter()
                    .filter(|root| {
                        root.grounding_namespace == GroundingNamespaceV1::CanonicalL1Anchor
                    })
                    .map(|root| (root.grounding_ref, target.canonical_bytes.as_slice()))
            })
            .collect::<Vec<_>>();
        if roots.len() != self.canonical_l1_anchor_proofs.len()
            || self
                .canonical_l1_anchor_proofs
                .windows(2)
                .any(|pair| pair[0].proof_ref >= pair[1].proof_ref)
        {
            return false;
        }
        for proof in &self.canonical_l1_anchor_proofs {
            if !proof.has_canonical_ref()
                || proof.l11_sha256 != self.package_tuple.l11_sha256
                || proof.canonical_l2_sha256 != self.package_tuple.canonical_l2_sha256
            {
                return false;
            }
            let matching = roots
                .iter()
                .filter(|(proof_ref, target_bytes)| {
                    *proof_ref == proof.proof_ref
                        && *target_bytes == proof.exact_target_bytes.as_bytes()
                })
                .count();
            if matching != 1 {
                return false;
            }
        }
        roots.iter().all(|(proof_ref, target_bytes)| {
            self.canonical_l1_anchor_proofs
                .iter()
                .filter(|proof| {
                    proof.proof_ref == *proof_ref
                        && proof.exact_target_bytes.as_bytes() == *target_bytes
                })
                .count()
                == 1
        })
    }

    pub(super) fn exact_digest(&self) -> [u64; 2] {
        self.compact.integrity.exact_digest
    }

    pub(super) fn completeness(&self) -> EnumerationCompletenessV1 {
        self.compact.completeness
    }

    pub(super) fn validates_relation_partition_proof(&self) -> bool {
        let retained = self.compact.evidence_tables.exact_relation_partition;
        let Some(expected) = build_relation_partition_proof(self) else {
            return false;
        };
        let Some(scope_ref) = self
            .compact
            .completeness
            .scope()
            .exhaustive_partition_proof_ref()
        else {
            return false;
        };
        self.compact.completeness.state() == EnumerationStateV1::Complete
            && self.compact.completeness.scope().kind()
                == crate::typing_transition::target_evidence::CompletenessScopeKindV1::RelationPartition
            && scope_ref.get() == retained.proof_ref
            && self.compact.completeness.retained_count()
                == retained.retained_target_count
            && self.compact.completeness.logical_count_lower_bound()
                == retained.logical_target_count
            && self.compact.completeness.all_seen_digest()
                == retained.target_root_set_digest
            && retained.is_present()
            && retained == expected
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn corrupt_exact_search_proof_for_test(
        &mut self,
        fault: ExactSearchProofFaultV1,
    ) {
        let proof = self
            .exact_search_proof
            .as_mut()
            .expect("test fault requires a completed exact search proof");
        match fault {
            ExactSearchProofFaultV1::CanonicalPackage => {
                proof.canonical_l2_sha256[0] ^= u8::MAX;
            }
            ExactSearchProofFaultV1::Sidecar => {
                proof.sidecar_sha256[0] ^= u8::MAX;
            }
            ExactSearchProofFaultV1::Semantics => {
                proof.phase7d_semantics_sha256[0] ^= u8::MAX;
            }
        }
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn corrupt_canonical_l1_anchor_proof_for_test(
        &mut self,
        fault: CanonicalL1AnchorProofFaultV1,
    ) {
        let original_relation_proof = self.compact.evidence_tables.exact_relation_partition;
        match fault {
            CanonicalL1AnchorProofFaultV1::Missing => {
                self.canonical_l1_anchor_proofs.remove(0);
            }
            CanonicalL1AnchorProofFaultV1::Extra => {
                let retained = self.canonical_l1_anchor_proofs[0].clone();
                let mut extra = CanonicalL1AnchorProofV1::new(
                    format!("{}-extra", retained.exact_target_bytes),
                    retained.target_form_ref.wrapping_add(101),
                    retained.target_form_ref.wrapping_add(101),
                    retained.lemma_id,
                    retained.terminal_id.wrapping_add(101),
                    CanonicalL1AnchorKindV1::Direct,
                    retained.l11_sha256,
                    retained.canonical_l2_sha256,
                )
                .expect("test extra proof");
                while self
                    .canonical_l1_anchor_proofs
                    .iter()
                    .any(|proof| proof.proof_ref == extra.proof_ref)
                {
                    extra.terminal_id = extra.terminal_id.wrapping_add(1);
                    extra.proof_ref = stable_bytes_ref(&extra.canonical_identity_bytes());
                }
                self.canonical_l1_anchor_proofs.push(extra);
                self.canonical_l1_anchor_proofs
                    .sort_by_key(|proof| proof.proof_ref);
            }
            CanonicalL1AnchorProofFaultV1::Ambiguous => {
                let duplicate = self.canonical_l1_anchor_proofs[0].clone();
                self.canonical_l1_anchor_proofs.push(duplicate);
                self.canonical_l1_anchor_proofs
                    .sort_by_key(|proof| proof.proof_ref);
            }
            CanonicalL1AnchorProofFaultV1::RawTerminalSubstitution => {
                let proof = self.canonical_l1_anchor_proofs.remove(0);
                rewrite_anchor_roots(
                    self,
                    proof.proof_ref,
                    proof.terminal_id,
                    GroundingNamespaceV1::L11Terminal,
                );
            }
            fault => {
                let proof = &mut self.canonical_l1_anchor_proofs[0];
                let old_ref = proof.proof_ref;
                match fault {
                    CanonicalL1AnchorProofFaultV1::TargetBytes => {
                        proof.exact_target_bytes.push('!');
                    }
                    CanonicalL1AnchorProofFaultV1::TargetForm => {
                        proof.target_form_ref = proof.target_form_ref.wrapping_add(7);
                    }
                    CanonicalL1AnchorProofFaultV1::AnchorForm => {
                        proof.anchor_form_ref = proof.anchor_form_ref.wrapping_add(7);
                    }
                    CanonicalL1AnchorProofFaultV1::Lemma => {
                        proof.lemma_id = Some(proof.lemma_id.unwrap_or_default().wrapping_add(7));
                    }
                    CanonicalL1AnchorProofFaultV1::Terminal => {
                        proof.terminal_id = proof.terminal_id.wrapping_add(7);
                    }
                    CanonicalL1AnchorProofFaultV1::Kind => {
                        proof.kind = match proof.kind {
                            CanonicalL1AnchorKindV1::Direct => CanonicalL1AnchorKindV1::SameLemma,
                            CanonicalL1AnchorKindV1::SameLemma => CanonicalL1AnchorKindV1::Direct,
                        };
                    }
                    CanonicalL1AnchorProofFaultV1::L11Package => {
                        proof.l11_sha256[0] ^= u8::MAX;
                    }
                    CanonicalL1AnchorProofFaultV1::CanonicalL2Package => {
                        proof.canonical_l2_sha256[0] ^= u8::MAX;
                    }
                    CanonicalL1AnchorProofFaultV1::Missing
                    | CanonicalL1AnchorProofFaultV1::Extra
                    | CanonicalL1AnchorProofFaultV1::Ambiguous
                    | CanonicalL1AnchorProofFaultV1::RawTerminalSubstitution => unreachable!(),
                }
                proof.proof_ref = stable_bytes_ref(&proof.canonical_identity_bytes());
                let new_ref = proof.proof_ref;
                rewrite_anchor_roots(
                    self,
                    old_ref,
                    new_ref,
                    GroundingNamespaceV1::CanonicalL1Anchor,
                );
                self.canonical_l1_anchor_proofs
                    .sort_by_key(|proof| proof.proof_ref);
            }
        }
        reseal_anchor_fault_material(self, original_relation_proof);
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn corrupt_observed_canonical_l1_anchor_for_test(
        &mut self,
    ) {
        let original_relation_proof = self.compact.evidence_tables.exact_relation_partition;
        let old_ref = self
            .exact_original
            .as_ref()
            .and_then(|original| {
                original.witness_roots.iter().find_map(|root| {
                    (root.grounding_namespace == GroundingNamespaceV1::CanonicalL1Anchor)
                        .then_some(root.grounding_ref)
                })
            })
            .expect("test observed anchor root");
        let proof = self
            .canonical_l1_anchor_proofs
            .iter_mut()
            .find(|proof| proof.proof_ref == old_ref)
            .expect("test observed anchor proof");
        proof.terminal_id = proof.terminal_id.wrapping_add(7);
        proof.proof_ref = stable_bytes_ref(&proof.canonical_identity_bytes());
        let new_ref = proof.proof_ref;
        rewrite_anchor_roots(
            self,
            old_ref,
            new_ref,
            GroundingNamespaceV1::CanonicalL1Anchor,
        );
        self.canonical_l1_anchor_proofs
            .sort_by_key(|proof| proof.proof_ref);
        reseal_anchor_fault_material(self, original_relation_proof);
    }

    pub(super) fn work(&self) -> EnumerationWorkCountersV1 {
        self.work
    }

    pub(super) fn exact_observed(&self) -> &str {
        &self.exact_observed
    }

    pub(super) fn original_has_grounded_l11_evidence(&self) -> bool {
        self.exact_original.as_ref().is_some_and(|original| {
            original.witness_roots.iter().any(|root| {
                matches!(
                    root.grounding_namespace,
                    GroundingNamespaceV1::L11Terminal | GroundingNamespaceV1::CanonicalL1Anchor
                ) && is_grounded_membership(root.verdict_membership)
            })
        })
    }

    pub(super) fn boundary_groundings(&self) -> &[CompositeBoundaryGroundingV1] {
        &self.boundary_groundings
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn exact_peak_candidate_rows(&self) -> Vec<(u32, String)> {
        self.exact_peak_births
            .iter()
            .map(|birth| (birth.form_ref, birth.normalized_surface.clone()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    #[cfg(test)]
    pub(in crate::nanda_wave::l2_field) fn exact_peak_certificate_rows(
        &self,
    ) -> Vec<(u32, String, u8, String)> {
        self.exact_peak_births
            .iter()
            .map(|birth| {
                (
                    birth.form_ref,
                    birth.normalized_surface.clone(),
                    birth.class as u8,
                    birth.canonical_key.clone(),
                )
            })
            .collect()
    }

    pub(super) fn exact_peak_layout_surfaces(&self) -> BTreeSet<String> {
        self.exact_peak_births
            .iter()
            .filter(|birth| birth.class == ExactPeakCertificateClassV1::KeyboardLayout)
            .map(|birth| birth.normalized_surface.clone())
            .collect()
    }
}

#[cfg(test)]
fn rewrite_anchor_roots(
    material: &mut PreparedTargetMaterialShadowV1,
    old_ref: u32,
    new_ref: u32,
    namespace: GroundingNamespaceV1,
) {
    fn rewrite_target(
        target: &mut ExactMaterialTargetV1,
        old_ref: u32,
        new_ref: u32,
        namespace: GroundingNamespaceV1,
    ) {
        let mut roots = target
            .witness_roots
            .iter()
            .cloned()
            .zip(target.compact.witnesses.witnesses().iter().copied())
            .map(|(mut root, witness)| {
                if root.grounding_namespace == GroundingNamespaceV1::CanonicalL1Anchor
                    && root.grounding_ref == old_ref
                {
                    root.grounding_namespace = namespace;
                    root.grounding_ref = new_ref;
                }
                (
                    root,
                    (witness.support_milli, witness.provenance_annotations),
                )
            })
            .collect::<Vec<_>>();
        roots.sort_by(|left, right| left.0.cmp(&right.0));
        target.witness_roots = roots.iter().map(|(root, _)| root.clone()).collect();
        target.compact.witnesses = roots.into_iter().fold(
            TargetEvidenceSetV1::complete_empty(),
            |evidence, (root, (support, annotations))| {
                evidence.merge(TargetEvidenceSetV1::from_one(
                    root.compact(support, annotations),
                ))
            },
        );
    }

    if let Some(original) = material.exact_original.as_mut() {
        rewrite_target(original, old_ref, new_ref, namespace);
    }
    for target in &mut material.exact_targets {
        rewrite_target(target, old_ref, new_ref, namespace);
    }
    let mut bounded = BoundedTargetSetV1::default();
    for target in &material.exact_targets {
        bounded
            .push(target.compact)
            .expect("test anchor rewrite remains within bounded targets");
    }
    material.compact.targets = bounded;
}

#[cfg(test)]
fn reseal_anchor_fault_material(
    material: &mut PreparedTargetMaterialShadowV1,
    original_relation_proof: ExactRelationPartitionProofV1,
) {
    material.compact.evidence_tables = evidence_table_identities(
        material.exact_original.as_ref(),
        &material.exact_targets,
        &material.canonical_l1_anchor_proofs,
    );
    material.compact.evidence_tables.exact_relation_partition = original_relation_proof;
    if let Some(proof) = build_relation_partition_proof(material) {
        material.compact.evidence_tables.exact_relation_partition = proof;
        material.compact.completeness = EnumerationCompletenessV1::complete_in_scope(
            material.exact_targets.len(),
            proof.target_root_set_digest,
            CompletenessScopeV1::relation_partition(proof.proof_ref)
                .expect("test relation proof ref"),
        );
        material.compact.original = prepared_relation_partition_original(
            &material.exact_observed,
            material.exact_original.as_ref(),
            &material.exact_targets,
            true,
        );
    }
    refresh_material_integrity(material);
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MaterialTargetAccumulatorV1 {
    normalized_scalars: String,
    canonical_bytes: Vec<u8>,
    roots: BTreeMap<ExactWitnessRootV1, (u16, u16)>,
}

impl MaterialTargetAccumulatorV1 {
    fn observe(&mut self, root: ExactWitnessRootV1, support_milli: u16, annotations: u16) {
        self.roots
            .entry(root)
            .and_modify(|retained| {
                retained.0 = retained.0.max(support_milli);
                retained.1 |= annotations;
            })
            .or_insert((support_milli, annotations));
    }

    fn finish(self) -> ExactMaterialTargetV1 {
        let witness_roots = self.roots.keys().cloned().collect::<Vec<_>>();
        let witnesses = self.roots.into_iter().fold(
            TargetEvidenceSetV1::complete_empty(),
            |evidence, (root, (support, annotations))| {
                evidence.merge(TargetEvidenceSetV1::from_one(
                    root.compact(support, annotations),
                ))
            },
        );
        let normalized_ref = stable_bytes_ref(self.normalized_scalars.as_bytes());
        let canonical_ref = stable_bytes_ref(&self.canonical_bytes);
        let compact = PreparedTargetV1 {
            identity: MaterialTargetIdentityV1 {
                normalized_scalars_ref: normalized_ref,
                canonical_bytes_ref: canonical_ref,
                normalization_layout_profile_id: NormalizationLayoutProfileIdV1(1),
                separator_profile_id: SeparatorProfileIdV1(
                    if self.normalized_scalars.contains(' ') {
                        ASCII_SPACE_SEPARATOR_PROFILE
                    } else {
                        0
                    },
                ),
                exact_scalar_count: self
                    .normalized_scalars
                    .chars()
                    .count()
                    .min(usize::from(u16::MAX)) as u16,
                flags: 0,
                accelerator: normalized_ref,
            },
            witnesses,
        };
        ExactMaterialTargetV1 {
            normalized_scalars: self.normalized_scalars,
            canonical_bytes: self.canonical_bytes,
            witness_roots,
            compact,
        }
    }
}

pub(super) fn prepare_context_neutral_productive_material(
    observed: &str,
    package_tuple: ExactPackageTupleV1,
    enumeration: ContextNeutralProductiveEnumerationV1,
) -> Result<PreparedTargetMaterialShadowV1, String> {
    prepare_context_neutral_productive_material_with_contours(
        observed,
        package_tuple,
        enumeration,
        TypedContourBirthEnumerationV1::complete_empty(),
    )
}

pub(super) fn prepare_context_neutral_productive_material_with_contours(
    observed: &str,
    package_tuple: ExactPackageTupleV1,
    enumeration: ContextNeutralProductiveEnumerationV1,
    contour_births: TypedContourBirthEnumerationV1,
) -> Result<PreparedTargetMaterialShadowV1, String> {
    prepare_context_neutral_productive_material_with_contours_and_boundaries(
        observed,
        package_tuple,
        enumeration,
        contour_births,
        TypedBoundaryBirthEnumerationV1::complete_empty(),
    )
}

pub(super) fn prepare_context_neutral_productive_material_with_contours_and_exact_peaks(
    observed: &str,
    package_tuple: ExactPackageTupleV1,
    enumeration: ContextNeutralProductiveEnumerationV1,
    contour_births: TypedContourBirthEnumerationV1,
    exact_peaks: ExactPeakBirthEnumerationV1,
) -> Result<PreparedTargetMaterialShadowV1, String> {
    prepare_context_neutral_productive_material_all(
        observed,
        package_tuple,
        enumeration,
        contour_births,
        TypedBoundaryBirthEnumerationV1::complete_empty(),
        exact_peaks,
    )
}

pub(super) fn prepare_frame_bound_lexical_authority_material(
    observed: &str,
    package_tuple: ExactPackageTupleV1,
    mut enumeration: ContextNeutralProductiveEnumerationV1,
    mut contour_births: TypedContourBirthEnumerationV1,
    exact_peaks: ExactPeakBirthEnumerationV1,
) -> Result<PreparedTargetMaterialShadowV1, String> {
    // Productive remains the honest whole-field display/ranking diagnostic. It
    // cannot widen this narrow authority membership partition.
    enumeration.readout.candidates.clear();
    enumeration.readout.logical_terminal_count = 0;
    enumeration.readout.logical_surface_basin_count = 0;
    enumeration.readout.integrity_error = None;
    enumeration.productive_work = EnumerationWorkCountersV1::default();
    enumeration.aggregate_work = EnumerationWorkCountersV1::default();
    enumeration.work_budget_exceeded = false;

    let contour_surface_count = contour_births
        .births
        .iter()
        .map(|birth| super::super::compositional::normalize_surface(&birth.normalized_surface))
        .collect::<BTreeSet<_>>()
        .len();
    let grounding_complete = contour_births.overflow_reason.is_none()
        && contour_surface_count == contour_births.logical_match_count;
    let authority_peaks =
        exact_peaks.authority_partition(observed, package_tuple.canonical_l2_sha256);
    let partition_ready = grounding_complete && authority_peaks.is_some();
    let authority_peaks = authority_peaks.unwrap_or_else(|| {
        ExactPeakBirthEnumerationV1::incomplete(IncompletenessReasonV1::IntegrityFailure)
    });
    let partition_surfaces = authority_peaks
        .births
        .iter()
        .map(|birth| birth.normalized_surface.as_bytes().to_vec())
        .collect::<BTreeSet<_>>();

    // L1.1 owns grounding only. Retain a grounded original for preservation and
    // grounded roots whose surface is already a member of the exact partition.
    contour_births.births.retain(|birth| {
        let normalized = super::super::compositional::normalize_surface(&birth.normalized_surface);
        birth.grounding_namespace == GroundingNamespaceV1::CanonicalL1Anchor
            && is_grounded_membership(birth.verdict_membership)
            && (normalized.as_bytes() == observed.as_bytes()
                || partition_surfaces.contains(normalized.as_bytes()))
    });
    let retained_anchor_refs = contour_births
        .births
        .iter()
        .map(|birth| birth.grounding_ref)
        .collect::<BTreeSet<_>>();
    contour_births
        .canonical_l1_anchor_proofs
        .retain(|proof| retained_anchor_refs.contains(&proof.proof_ref));
    contour_births.logical_match_count = contour_births
        .births
        .iter()
        .map(|birth| birth.normalized_surface.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    contour_births.overflow_reason = None;

    let mut material = prepare_context_neutral_productive_material_all(
        observed,
        package_tuple,
        enumeration,
        if partition_ready {
            contour_births
        } else {
            TypedContourBirthEnumerationV1::complete_empty()
        },
        TypedBoundaryBirthEnumerationV1::complete_empty(),
        authority_peaks,
    )?;

    let relation_proof = partition_ready
        .then(|| build_relation_partition_proof(&material))
        .flatten();
    if let Some(proof) = relation_proof {
        let scope = CompletenessScopeV1::relation_partition(proof.proof_ref)
            .expect("relation proof references are canonical and non-zero");
        material.compact.completeness = EnumerationCompletenessV1::complete_in_scope(
            material.exact_targets.len(),
            proof.target_root_set_digest,
            scope,
        );
        material.compact.evidence_tables.exact_relation_partition = proof;
        material.compact.original = prepared_relation_partition_original(
            observed,
            material.exact_original.as_ref(),
            &material.exact_targets,
            true,
        );
    } else {
        material.compact.completeness =
            EnumerationCompletenessV1::failed(IncompletenessReasonV1::IntegrityFailure);
        material.compact.evidence_tables.exact_relation_partition =
            ExactRelationPartitionProofV1::default();
        material.compact.original = prepared_relation_partition_original(
            observed,
            material.exact_original.as_ref(),
            &material.exact_targets,
            false,
        );
    }
    refresh_material_integrity(&mut material);
    Ok(material)
}

pub(super) fn prepare_context_neutral_productive_material_with_contours_and_boundaries(
    observed: &str,
    package_tuple: ExactPackageTupleV1,
    enumeration: ContextNeutralProductiveEnumerationV1,
    contour_births: TypedContourBirthEnumerationV1,
    boundary_births: TypedBoundaryBirthEnumerationV1,
) -> Result<PreparedTargetMaterialShadowV1, String> {
    prepare_context_neutral_productive_material_all(
        observed,
        package_tuple,
        enumeration,
        contour_births,
        boundary_births,
        ExactPeakBirthEnumerationV1::complete_empty(),
    )
}

fn prepare_context_neutral_productive_material_all(
    observed: &str,
    package_tuple: ExactPackageTupleV1,
    enumeration: ContextNeutralProductiveEnumerationV1,
    contour_births: TypedContourBirthEnumerationV1,
    boundary_births: TypedBoundaryBirthEnumerationV1,
    exact_peaks: ExactPeakBirthEnumerationV1,
) -> Result<PreparedTargetMaterialShadowV1, String> {
    validate_contour_anchor_proof_inputs(&contour_births, package_tuple)?;
    let mut targets = BTreeMap::<Vec<u8>, MaterialTargetAccumulatorV1>::new();
    let mut productive_surface_rank = BTreeMap::<Vec<u8>, usize>::new();
    for (rank, candidate) in enumeration.readout.candidates.iter().enumerate() {
        let normalized =
            super::super::compositional::normalize_surface(&candidate.normalized_surface);
        productive_surface_rank
            .entry(normalized.into_bytes())
            .or_insert(rank);
    }
    for candidate in &enumeration.readout.candidates {
        insert_productive_candidate(&mut targets, candidate);
    }
    let productive_observed_present = targets.contains_key(observed.as_bytes());
    let productive_logical_count = targets
        .len()
        .saturating_sub(usize::from(productive_observed_present));
    let mut productive_surfaces = targets.keys().cloned().collect::<BTreeSet<_>>();
    productive_surfaces.remove(observed.as_bytes());
    for birth in &exact_peaks.births {
        insert_exact_peak_birth(&mut targets, birth);
    }
    let mut exact_peak_surfaces = exact_peaks
        .births
        .iter()
        .map(|birth| birth.normalized_surface.as_bytes().to_vec())
        .collect::<BTreeSet<_>>();
    exact_peak_surfaces.remove(observed.as_bytes());
    for birth in &contour_births.births {
        insert_contour_birth(&mut targets, birth);
    }
    let mut contour_surfaces = contour_births
        .births
        .iter()
        .map(|birth| birth.normalized_surface.as_bytes().to_vec())
        .collect::<BTreeSet<_>>();
    contour_surfaces.remove(observed.as_bytes());
    for birth in &boundary_births.births {
        insert_boundary_birth(&mut targets, birth);
    }
    let mut boundary_surfaces = boundary_births
        .births
        .iter()
        .map(|birth| birth.normalized_surface.as_bytes().to_vec())
        .collect::<BTreeSet<_>>();
    boundary_surfaces.remove(observed.as_bytes());
    let exact_original = targets
        .remove(observed.as_bytes())
        .map(MaterialTargetAccumulatorV1::finish);
    let logical_count = targets.len();
    let mut all_exact_targets = targets
        .into_values()
        .map(MaterialTargetAccumulatorV1::finish)
        .collect::<Vec<_>>();
    sort_exact_targets(&mut all_exact_targets);
    let base_all_seen_digest = combined_target_set_digest(
        &all_exact_targets,
        contour_births.all_seen_digest,
        boundary_births.all_seen_digest,
    );
    let all_seen_digest = if exact_peaks.is_empty() && !exact_peaks.capacity_exceeded() {
        base_all_seen_digest
    } else {
        combined_target_set_digest_with_exact_peaks(
            base_all_seen_digest,
            exact_peaks.all_seen_digest,
        )
    };

    let mut mandatory_targets = Vec::new();
    let mut optional_productive_targets = Vec::new();
    let mut contour_only_targets = Vec::new();
    let mut boundary_only_targets = Vec::new();
    for target in all_exact_targets {
        let grounded_l11_target = target.witness_roots.iter().any(|root| {
            matches!(
                root.grounding_namespace,
                GroundingNamespaceV1::L11Terminal | GroundingNamespaceV1::CanonicalL1Anchor
            ) && matches!(
                root.verdict_membership,
                VerdictMembershipV1::Grounded
                    | VerdictMembershipV1::L11Winner
                    | VerdictMembershipV1::L11Tied
            )
        });
        if exact_peak_surfaces.contains(&target.canonical_bytes) || grounded_l11_target {
            mandatory_targets.push(target);
        } else if productive_surfaces.contains(&target.canonical_bytes) {
            optional_productive_targets.push(target);
        } else if boundary_surfaces.contains(&target.canonical_bytes) {
            boundary_only_targets.push(target);
        } else if contour_surfaces.contains(&target.canonical_bytes) {
            contour_only_targets.push(target);
        }
    }
    let contour_logical_count = contour_only_targets.len();
    let boundary_logical_count = boundary_only_targets.len();
    let mut retained_contours = retain_contour_targets(contour_only_targets);
    sort_exact_targets(&mut boundary_only_targets);
    boundary_only_targets.truncate(MAX_BOUNDARY_TARGETS_PER_FIELD);
    sort_exact_targets(&mut mandatory_targets);
    if mandatory_targets.len() > MAX_TARGETS_PER_FIELD {
        return Err("exact and grounded material exceeds common 74-target capacity".to_string());
    }

    let mut exact_targets = mandatory_targets;
    let mut remaining = MAX_TARGETS_PER_FIELD.saturating_sub(exact_targets.len());
    boundary_only_targets.truncate(remaining);
    remaining = remaining.saturating_sub(boundary_only_targets.len());
    retained_contours.truncate(remaining);
    remaining = remaining.saturating_sub(retained_contours.len());
    optional_productive_targets.sort_by(|left, right| {
        productive_surface_rank
            .get(&left.canonical_bytes)
            .copied()
            .unwrap_or(usize::MAX)
            .cmp(
                &productive_surface_rank
                    .get(&right.canonical_bytes)
                    .copied()
                    .unwrap_or(usize::MAX),
            )
            .then_with(|| left.canonical_bytes.cmp(&right.canonical_bytes))
            .then_with(|| left.normalized_scalars.cmp(&right.normalized_scalars))
    });
    optional_productive_targets.truncate(remaining);
    exact_targets.extend(boundary_only_targets);
    exact_targets.extend(retained_contours);
    exact_targets.extend(optional_productive_targets);
    sort_exact_targets(&mut exact_targets);
    let storage_overflow = contour_logical_count > MAX_CONTOUR_TARGETS_PER_FIELD
        || boundary_logical_count > MAX_BOUNDARY_TARGETS_PER_FIELD
        || logical_count > MAX_TARGETS_PER_FIELD
        || exact_peaks.capacity_exceeded();
    exact_targets.truncate(MAX_TARGETS_PER_FIELD);

    let mut bounded = BoundedTargetSetV1::default();
    for target in &exact_targets {
        bounded
            .push(target.compact)
            .map_err(|_| "material target capacity disagrees with deterministic truncation")?;
    }
    let completeness = if enumeration.readout.integrity_error.is_some()
        || exact_peaks.overflow_reason == Some(IncompletenessReasonV1::IntegrityFailure)
    {
        EnumerationCompletenessV1::failed(IncompletenessReasonV1::IntegrityFailure)
    } else if enumeration.work_budget_exceeded
        || contour_births.overflow_reason.is_some()
        || boundary_births.overflow_reason.is_some()
        || matches!(
            exact_peaks.overflow_reason,
            Some(IncompletenessReasonV1::WorkBudgetExceeded)
                | Some(IncompletenessReasonV1::UpstreamIncomplete)
        )
    {
        let reason = exact_peaks
            .overflow_reason
            .filter(|reason| {
                matches!(
                    reason,
                    IncompletenessReasonV1::WorkBudgetExceeded
                        | IncompletenessReasonV1::UpstreamIncomplete
                )
            })
            .unwrap_or(IncompletenessReasonV1::WorkBudgetExceeded);
        EnumerationCompletenessV1::overflow(
            bounded.len(),
            logical_count
                .max(contour_births.logical_match_count)
                .max(boundary_births.logical_match_count)
                .max(bounded.len().saturating_add(1)),
            reason,
            all_seen_digest,
        )
    } else if storage_overflow {
        EnumerationCompletenessV1::overflow(
            bounded.len(),
            logical_count.max(exact_peaks.logical_match_count),
            IncompletenessReasonV1::StorageCapacity,
            all_seen_digest,
        )
    } else if (enumeration.readout.logical_surface_basin_count as usize)
        .saturating_sub(usize::from(productive_observed_present))
        > productive_logical_count
    {
        EnumerationCompletenessV1::overflow(
            bounded.len(),
            (enumeration.readout.logical_surface_basin_count as usize)
                .saturating_sub(usize::from(productive_observed_present)),
            IncompletenessReasonV1::UpstreamIncomplete,
            all_seen_digest,
        )
    } else {
        EnumerationCompletenessV1::complete(logical_count, all_seen_digest)
    };
    let package_generation = u64::from_le_bytes(
        package_tuple.productive_sha256[..8]
            .try_into()
            .expect("SHA-256 prefix"),
    );
    let mut exact_package_digest_prefix = [0_u8; 16];
    exact_package_digest_prefix.copy_from_slice(&package_tuple.productive_sha256[..16]);
    let key = PreparedMaterialKeyV1 {
        observed_contour_ref: stable_bytes_ref(observed.as_bytes()),
        normalization_layout_profile_id: NormalizationLayoutProfileIdV1(1),
        package_generation,
        exact_package_digest_prefix,
    };
    let retained_anchor_refs = exact_original
        .iter()
        .chain(&exact_targets)
        .flat_map(|target| {
            target
                .witness_roots
                .iter()
                .filter(|root| root.grounding_namespace == GroundingNamespaceV1::CanonicalL1Anchor)
                .map(|root| root.grounding_ref)
        })
        .collect::<BTreeSet<_>>();
    let canonical_l1_anchor_proofs = contour_births
        .canonical_l1_anchor_proofs
        .iter()
        .filter(|proof| retained_anchor_refs.contains(&proof.proof_ref))
        .cloned()
        .collect::<Vec<_>>();
    if canonical_l1_anchor_proofs.len() != retained_anchor_refs.len() {
        return Err("retained canonical L1 anchor proof table is incomplete".to_string());
    }
    let original = prepared_original_material(observed, exact_original.as_ref(), completeness);
    let evidence_tables = evidence_table_identities(
        exact_original.as_ref(),
        &exact_targets,
        &canonical_l1_anchor_proofs,
    );
    let aggregate_work = enumeration
        .aggregate_work
        .checked_add(contour_births.work)
        .and_then(|work| work.checked_add(boundary_births.work))
        .and_then(|work| work.checked_add(exact_peaks.work))
        .ok_or_else(|| "material aggregate work counter overflow".to_string())?;
    let retained_boundary_roots = exact_targets
        .iter()
        .flat_map(|target| target.compact.witnesses.witnesses())
        .filter(|witness| witness.grounding_namespace == GroundingNamespaceV1::CompositeBoundary)
        .map(|witness| (witness.grounding_ref, witness.derivation_ref))
        .collect::<BTreeSet<_>>();
    let mut boundary_groundings = boundary_births
        .births
        .iter()
        .filter(|birth| {
            retained_boundary_roots.contains(&(birth.grounding_ref, birth.derivation_ref))
        })
        .map(|birth| birth.composite_grounding.clone())
        .collect::<Vec<_>>();
    boundary_groundings.sort();
    boundary_groundings.dedup();
    let integrity_digest = material_integrity_digest(
        observed,
        package_tuple,
        original,
        exact_original.as_ref(),
        &exact_targets,
        &canonical_l1_anchor_proofs,
        &boundary_groundings,
        completeness,
        evidence_tables,
        aggregate_work,
    );
    Ok(PreparedTargetMaterialShadowV1 {
        compact: PreparedTargetMaterialV1 {
            key,
            original,
            targets: bounded,
            completeness,
            evidence_tables,
            integrity: PreparedIntegrityV1 {
                exact_digest: integrity_digest,
            },
        },
        exact_observed: observed.to_string(),
        package_tuple,
        exact_original,
        exact_targets,
        canonical_l1_anchor_proofs,
        exact_peak_births: exact_peaks.births,
        exact_retrieved_form_refs: exact_peaks.retrieved_form_refs,
        exact_search_proof: exact_peaks.search_proof,
        boundary_groundings,
        work: aggregate_work,
    })
}

fn validate_contour_anchor_proof_inputs(
    contour_births: &TypedContourBirthEnumerationV1,
    package_tuple: ExactPackageTupleV1,
) -> Result<(), String> {
    let anchor_births = contour_births
        .births
        .iter()
        .filter(|birth| birth.grounding_namespace == GroundingNamespaceV1::CanonicalL1Anchor)
        .collect::<Vec<_>>();
    if anchor_births.len() != contour_births.canonical_l1_anchor_proofs.len() {
        return Err("canonical L1 anchor proof table cardinality mismatch".to_string());
    }
    let mut previous_ref = None;
    for proof in &contour_births.canonical_l1_anchor_proofs {
        if previous_ref.is_some_and(|previous| previous >= proof.proof_ref)
            || !proof.has_canonical_ref()
            || proof.l11_sha256 != package_tuple.l11_sha256
            || proof.canonical_l2_sha256 != package_tuple.canonical_l2_sha256
        {
            return Err("canonical L1 anchor proof table is malformed or ambiguous".to_string());
        }
        previous_ref = Some(proof.proof_ref);
        if anchor_births
            .iter()
            .filter(|birth| {
                birth.grounding_ref == proof.proof_ref
                    && super::super::compositional::normalize_surface(&birth.normalized_surface)
                        .as_bytes()
                        == proof.exact_target_bytes.as_bytes()
            })
            .count()
            != 1
        {
            return Err("canonical L1 anchor proof does not own one exact birth".to_string());
        }
    }
    if anchor_births.iter().any(|birth| {
        !is_grounded_membership(birth.verdict_membership)
            || contour_births
                .canonical_l1_anchor_proofs
                .iter()
                .filter(|proof| {
                    proof.proof_ref == birth.grounding_ref
                        && proof.exact_target_bytes.as_bytes()
                            == super::super::compositional::normalize_surface(
                                &birth.normalized_surface,
                            )
                            .as_bytes()
                })
                .count()
                != 1
    }) {
        return Err("canonical L1 anchor birth has no unique full proof".to_string());
    }
    Ok(())
}

fn sort_exact_targets(targets: &mut [ExactMaterialTargetV1]) {
    targets.sort_by(|left, right| {
        left.canonical_bytes
            .cmp(&right.canonical_bytes)
            .then_with(|| left.normalized_scalars.cmp(&right.normalized_scalars))
    });
}

fn retain_contour_targets(targets: Vec<ExactMaterialTargetV1>) -> Vec<ExactMaterialTargetV1> {
    let mut partitions = BTreeMap::<TargetRelationV1, Vec<ExactMaterialTargetV1>>::new();
    for target in targets {
        let relation = target
            .witness_roots
            .iter()
            .map(|root| root.relation)
            .min()
            .unwrap_or(TargetRelationV1::Unsupported);
        partitions.entry(relation).or_default().push(target);
    }
    for partition in partitions.values_mut() {
        sort_exact_targets(partition);
        partition.reverse();
    }

    let mut retained = Vec::with_capacity(MAX_CONTOUR_TARGETS_PER_FIELD);
    if let Some(direct_layout) = partitions.get_mut(&TargetRelationV1::ExactLayout) {
        while retained.len() < MAX_CONTOUR_TARGETS_PER_FIELD {
            let Some(target) = direct_layout.pop() else {
                break;
            };
            retained.push(target);
        }
    }
    partitions.remove(&TargetRelationV1::ExactLayout);

    while retained.len() < MAX_CONTOUR_TARGETS_PER_FIELD {
        let mut progressed = false;
        for partition in partitions.values_mut() {
            if retained.len() == MAX_CONTOUR_TARGETS_PER_FIELD {
                break;
            }
            if let Some(target) = partition.pop() {
                retained.push(target);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    retained
}

fn prepared_original_material(
    observed: &str,
    original: Option<&ExactMaterialTargetV1>,
    completeness: EnumerationCompletenessV1,
) -> PreparedOriginalMaterialV1 {
    let exact_grounded = original.is_some_and(|target| {
        target.canonical_bytes.as_slice() == observed.as_bytes()
            && target
                .witness_roots
                .iter()
                .any(|root| is_grounded_membership(root.verdict_membership))
    });
    let lexical_status = if completeness.state()
        != crate::typing_transition::target_evidence::EnumerationStateV1::Complete
    {
        PreparedOriginalLexicalStatusV1::Unknown
    } else if exact_grounded {
        PreparedOriginalLexicalStatusV1::Clean
    } else {
        PreparedOriginalLexicalStatusV1::Damaged
    };
    let script_token_status = if observed.is_empty() || observed.chars().any(char::is_control) {
        PreparedOriginalScriptTokenStatusV1::Unsupported
    } else {
        PreparedOriginalScriptTokenStatusV1::Supported
    };
    let punctuation_status = if observed.chars().all(char::is_alphabetic) {
        PreparedOriginalPunctuationStatusV1::Stable
    } else {
        PreparedOriginalPunctuationStatusV1::Unknown
    };
    PreparedOriginalMaterialV1 {
        exact_observed_scalars_ref: stable_bytes_ref(
            &observed
                .chars()
                .flat_map(|scalar| u32::from(scalar).to_le_bytes())
                .collect::<Vec<_>>(),
        ),
        exact_observed_bytes_ref: stable_bytes_ref(observed.as_bytes()),
        preservation_schema: 1,
        lexical_status,
        script_token_status,
        punctuation_status,
        reserved: 0,
    }
}

fn prepared_relation_partition_original(
    observed: &str,
    original: Option<&ExactMaterialTargetV1>,
    targets: &[ExactMaterialTargetV1],
    partition_valid: bool,
) -> PreparedOriginalMaterialV1 {
    let mut prepared = prepared_original_material(
        observed,
        original,
        EnumerationCompletenessV1::complete(0, [0; 2]),
    );
    let original_grounded = original.is_some_and(|target| {
        target.canonical_bytes.as_slice() == observed.as_bytes()
            && target.witness_roots.iter().any(|root| {
                root.grounding_namespace == GroundingNamespaceV1::CanonicalL1Anchor
                    && is_grounded_membership(root.verdict_membership)
            })
    });
    let partition_target_grounded = partition_valid
        && targets.iter().any(|target| {
            target.witness_roots.iter().any(|root| {
                root.grounding_namespace == GroundingNamespaceV1::CanonicalL1Anchor
                    && is_grounded_membership(root.verdict_membership)
            })
        });
    prepared.lexical_status = if original_grounded {
        PreparedOriginalLexicalStatusV1::Clean
    } else if partition_target_grounded {
        PreparedOriginalLexicalStatusV1::Damaged
    } else {
        PreparedOriginalLexicalStatusV1::Unknown
    };
    prepared
}

const fn is_grounded_membership(membership: VerdictMembershipV1) -> bool {
    matches!(
        membership,
        VerdictMembershipV1::Grounded
            | VerdictMembershipV1::L11Winner
            | VerdictMembershipV1::L11Tied
    )
}

fn insert_productive_candidate(
    targets: &mut BTreeMap<Vec<u8>, MaterialTargetAccumulatorV1>,
    candidate: &PackagedProductiveCandidateV1,
) {
    let canonical_bytes = candidate.normalized_surface.as_bytes().to_vec();
    let target =
        targets
            .entry(canonical_bytes.clone())
            .or_insert_with(|| MaterialTargetAccumulatorV1 {
                normalized_scalars: candidate.normalized_surface.to_string(),
                canonical_bytes,
                roots: BTreeMap::new(),
            });
    for identity in &candidate.equivalent_identities {
        target.observe(
            ExactWitnessRootV1 {
                relation: TargetRelationV1::MorphologySlot,
                grounding_namespace: GroundingNamespaceV1::ProductiveSurface,
                verdict_membership: if candidate.grounded_support > 0 {
                    VerdictMembershipV1::Grounded
                } else {
                    VerdictMembershipV1::Born
                },
                operator_ref: identity.program_id,
                grounding_ref: identity.normalized_surface_id,
                derivation_ref: identity
                    .lemma_id
                    .rotate_left(7)
                    .wrapping_add(identity.paradigm_id)
                    .wrapping_add(identity.target_slot_id.rotate_left(17)),
            },
            candidate
                .minimum_independent_support
                .min(u32::from(u16::MAX)) as u16,
            1,
        );
    }
}

fn insert_exact_peak_birth(
    targets: &mut BTreeMap<Vec<u8>, MaterialTargetAccumulatorV1>,
    birth: &ExactPeakBirthV1,
) {
    let canonical_bytes = birth.normalized_surface.as_bytes().to_vec();
    let target =
        targets
            .entry(canonical_bytes.clone())
            .or_insert_with(|| MaterialTargetAccumulatorV1 {
                normalized_scalars: birth.normalized_surface.clone(),
                canonical_bytes,
                roots: BTreeMap::new(),
            });
    target.observe(
        ExactWitnessRootV1 {
            relation: birth.class.relation(),
            grounding_namespace: GroundingNamespaceV1::CanonicalForm,
            verdict_membership: VerdictMembershipV1::Born,
            operator_ref: birth.class.operator_ref(),
            grounding_ref: birth.form_ref,
            derivation_ref: birth.derivation_ref,
        },
        0,
        0,
    );
}

fn insert_contour_birth(
    targets: &mut BTreeMap<Vec<u8>, MaterialTargetAccumulatorV1>,
    birth: &TypedContourBirthV1,
) {
    let canonical_bytes = birth.normalized_surface.as_bytes().to_vec();
    let target =
        targets
            .entry(canonical_bytes.clone())
            .or_insert_with(|| MaterialTargetAccumulatorV1 {
                normalized_scalars: birth.normalized_surface.clone(),
                canonical_bytes,
                roots: BTreeMap::new(),
            });
    target.observe(
        ExactWitnessRootV1 {
            relation: birth.relation,
            grounding_namespace: birth.grounding_namespace,
            verdict_membership: birth.verdict_membership,
            operator_ref: birth.operator_ref,
            grounding_ref: birth.grounding_ref,
            derivation_ref: birth.derivation_ref,
        },
        birth.support_milli,
        0,
    );
}

fn insert_boundary_birth(
    targets: &mut BTreeMap<Vec<u8>, MaterialTargetAccumulatorV1>,
    birth: &TypedBoundaryBirthV1,
) {
    let canonical_bytes = birth.normalized_surface.as_bytes().to_vec();
    let target =
        targets
            .entry(canonical_bytes.clone())
            .or_insert_with(|| MaterialTargetAccumulatorV1 {
                normalized_scalars: birth.normalized_surface.clone(),
                canonical_bytes,
                roots: BTreeMap::new(),
            });
    target.observe(
        ExactWitnessRootV1 {
            relation: birth.relation,
            grounding_namespace: GroundingNamespaceV1::CompositeBoundary,
            verdict_membership: VerdictMembershipV1::Born,
            operator_ref: birth.operator_ref,
            grounding_ref: birth.grounding_ref,
            derivation_ref: birth.derivation_ref,
        },
        0,
        0,
    );
}

fn target_set_digest(targets: &[ExactMaterialTargetV1]) -> [u64; 2] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-context-neutral-target-set-v1\0");
    for target in targets {
        hash_len_bytes(&mut hasher, &target.canonical_bytes);
        for root in &target.witness_roots {
            hasher.update(root.canonical_bytes());
        }
    }
    digest128(hasher.finalize().into())
}

fn relation_partition_policy_identity() -> [u64; 2] {
    digest128(
        Sha256::digest(
            b"lay-td117-relation-partition-policy-v2\0missing-letter\0extra-letter\0single-substitution\0exact-layout\0adjacent-transposition\0canonical-l1-anchor-proof\0",
        )
        .into(),
    )
}

fn relation_partition_target_root_digest(
    targets: &[ExactMaterialTargetV1],
    canonical_l1_anchor_proofs: &[CanonicalL1AnchorProofV1],
) -> [u64; 2] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-td117-relation-partition-target-roots-v2\0");
    hasher.update((targets.len() as u64).to_le_bytes());
    for target in targets {
        hash_len_bytes(&mut hasher, &target.canonical_bytes);
        hasher.update((target.witness_roots.len() as u64).to_le_bytes());
        for root in &target.witness_roots {
            hasher.update(root.canonical_bytes());
        }
    }
    hasher.update((canonical_l1_anchor_proofs.len() as u64).to_le_bytes());
    for proof in canonical_l1_anchor_proofs {
        hasher.update(proof.proof_ref.to_le_bytes());
        hash_len_bytes(&mut hasher, &proof.canonical_identity_bytes());
    }
    digest128(hasher.finalize().into())
}

fn search_proof_matches_material(material: &PreparedTargetMaterialShadowV1) -> bool {
    let Some(proof) = material.exact_search_proof.as_ref() else {
        return false;
    };
    let Ok(oracle) = Phase7dCertificateOracle::new(&material.exact_observed) else {
        return false;
    };
    let lanes = oracle.retrieval_lanes();
    let Ok(supported_births) =
        replayed_supported_births(&material.exact_observed, &material.exact_peak_births)
    else {
        return false;
    };
    let supported_target_count = supported_births
        .iter()
        .map(|birth| birth.normalized_surface.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let mut canonical_refs = material.exact_retrieved_form_refs.clone();
    canonical_refs.sort_unstable();
    canonical_refs.dedup();
    canonical_refs == material.exact_retrieved_form_refs
        && material
            .exact_peak_births
            .iter()
            .all(|birth| canonical_refs.binary_search(&birth.form_ref).is_ok())
        && proof.exact_observed_sha256
            == <[u8; 32]>::from(Sha256::digest(material.exact_observed.as_bytes()))
        && proof.query_lanes_sha256 == query_lanes_digest(&lanes)
        && proof.canonical_l2_sha256 == material.package_tuple.canonical_l2_sha256
        && proof.sidecar_sha256 != [0; 32]
        && proof.phase7d_semantics_sha256 == phase7d_semantics_digest()
        && proof.search_budget_sha256 == exact_relation_search_budget_identity()
        && proof.retrieved_form_refs_sha256
            == u32_sequence_digest(
                b"lay-td117-retrieved-form-refs-v1\0",
                &material.exact_retrieved_form_refs,
            )
        && usize::try_from(proof.retrieved_form_count).ok()
            == Some(material.exact_retrieved_form_refs.len())
        && proof.supported_births_sha256 == exact_peak_set_sha256(&supported_births)
        && usize::from(proof.supported_target_count) == supported_target_count
        && usize::from(proof.supported_certificate_count) == supported_births.len()
        && proof.structural_trace_sha256 != [0; 32]
        && exact_search_work_within_budget(
            proof.work,
            lanes.len(),
            material.exact_retrieved_form_refs.len(),
        )
}

fn build_relation_partition_proof(
    material: &PreparedTargetMaterialShadowV1,
) -> Option<ExactRelationPartitionProofV1> {
    if !search_proof_matches_material(material)
        || !material.canonical_l1_anchor_proof_table_is_valid()
        || material.exact_targets.len() > MAX_TARGETS_PER_FIELD
    {
        return None;
    }
    let proof = material.exact_search_proof.as_ref()?;
    let mut expected_surfaces = material
        .exact_peak_births
        .iter()
        .map(|birth| birth.normalized_surface.as_bytes().to_vec())
        .collect::<BTreeSet<_>>();
    expected_surfaces.remove(material.exact_observed.as_bytes());
    let actual_surfaces = material
        .exact_targets
        .iter()
        .map(|target| target.canonical_bytes.clone())
        .collect::<BTreeSet<_>>();
    if expected_surfaces != actual_surfaces {
        return None;
    }

    let mut per_target_root_counts = [0_u8; MAX_TARGETS_PER_FIELD];
    let mut logical_certificate_count = 0_usize;
    let mut retained_certificate_count = 0_usize;
    for (index, target) in material.exact_targets.iter().enumerate() {
        if target.canonical_bytes.as_slice() != target.normalized_scalars.as_bytes()
            || target.compact != material.compact.targets.as_slice()[index]
            || target.witness_roots.is_empty()
            || target.witness_roots.len() > MAX_TARGET_WITNESSES_PER_TARGET
            || target.compact.witnesses.state() != EnumerationStateV1::Complete
            || target.compact.witnesses.logical_count() as usize != target.witness_roots.len()
            || target.compact.witnesses.retained_count() as usize != target.witness_roots.len()
            || target.compact.witnesses.witnesses().len() != target.witness_roots.len()
        {
            return None;
        }
        let mut owns_exact_membership = false;
        for (root_index, root) in target.witness_roots.iter().enumerate() {
            if root.compact(
                target.compact.witnesses.witnesses()[root_index].support_milli,
                target.compact.witnesses.witnesses()[root_index].provenance_annotations,
            ) != target.compact.witnesses.witnesses()[root_index]
            {
                return None;
            }
            if root.grounding_namespace == GroundingNamespaceV1::CanonicalForm
                && root.verdict_membership == VerdictMembershipV1::Born
            {
                let matches_birth = material.exact_peak_births.iter().any(|birth| {
                    birth.normalized_surface.as_bytes() == target.canonical_bytes
                        && birth.form_ref == root.grounding_ref
                        && birth.class.relation() == root.relation
                        && birth.class.operator_ref() == root.operator_ref
                        && birth.derivation_ref == root.derivation_ref
                });
                if !matches_birth {
                    return None;
                }
                owns_exact_membership = true;
            } else if root.grounding_namespace != GroundingNamespaceV1::CanonicalL1Anchor
                || !is_grounded_membership(root.verdict_membership)
                || material
                    .canonical_l1_anchor_proofs
                    .iter()
                    .filter(|proof| {
                        proof.proof_ref == root.grounding_ref
                            && proof.exact_target_bytes.as_bytes() == target.canonical_bytes
                    })
                    .count()
                    != 1
            {
                return None;
            }
        }
        if !owns_exact_membership {
            return None;
        }
        per_target_root_counts[index] = u8::try_from(target.witness_roots.len()).ok()?;
        logical_certificate_count =
            logical_certificate_count.checked_add(target.witness_roots.len())?;
        retained_certificate_count =
            retained_certificate_count.checked_add(target.compact.witnesses.witnesses().len())?;
    }
    if logical_certificate_count != retained_certificate_count {
        return None;
    }

    let target_root_set_digest = relation_partition_target_root_digest(
        &material.exact_targets,
        &material.canonical_l1_anchor_proofs,
    );
    let mut relation_proof = ExactRelationPartitionProofV1 {
        proof_ref: 1,
        policy_version: RELATION_PARTITION_POLICY_VERSION_V1,
        policy_identity: relation_partition_policy_identity(),
        exact_search_proof_digest: proof.canonical_digest(),
        target_root_set_digest,
        logical_target_count: u16::try_from(material.exact_targets.len()).ok()?,
        retained_target_count: u16::try_from(material.compact.targets.len()).ok()?,
        logical_certificate_count: u16::try_from(logical_certificate_count).ok()?,
        retained_certificate_count: u16::try_from(retained_certificate_count).ok()?,
        per_target_root_counts,
    };
    let digest = relation_partition_proof_digest(relation_proof);
    relation_proof.proof_ref = u32::from_le_bytes(
        digest[..4]
            .try_into()
            .expect("SHA-256 prefix has four bytes"),
    )
    .max(1);
    Some(relation_proof)
}

fn relation_partition_proof_digest(proof: ExactRelationPartitionProofV1) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-td117-relation-partition-proof-v2\0");
    hasher.update(proof.policy_version.to_le_bytes());
    for digest in [
        proof.policy_identity,
        proof.exact_search_proof_digest,
        proof.target_root_set_digest,
    ] {
        hasher.update(digest[0].to_le_bytes());
        hasher.update(digest[1].to_le_bytes());
    }
    hasher.update(proof.logical_target_count.to_le_bytes());
    hasher.update(proof.retained_target_count.to_le_bytes());
    hasher.update(proof.logical_certificate_count.to_le_bytes());
    hasher.update(proof.retained_certificate_count.to_le_bytes());
    hasher.update(proof.per_target_root_counts);
    hasher.finalize().into()
}

fn combined_target_set_digest(
    targets: &[ExactMaterialTargetV1],
    contour_digest: [u64; 2],
    boundary_digest: [u64; 2],
) -> [u64; 2] {
    let target_digest = target_set_digest(targets);
    let mut hasher = Sha256::new();
    hasher.update(b"lay-context-neutral-target-set-with-contours-v1\0");
    hasher.update(target_digest[0].to_le_bytes());
    hasher.update(target_digest[1].to_le_bytes());
    hasher.update(contour_digest[0].to_le_bytes());
    hasher.update(contour_digest[1].to_le_bytes());
    hasher.update(boundary_digest[0].to_le_bytes());
    hasher.update(boundary_digest[1].to_le_bytes());
    digest128(hasher.finalize().into())
}

fn exact_peak_set_digest(births: &[ExactPeakBirthV1]) -> [u64; 2] {
    digest128(exact_peak_set_sha256(births))
}

fn exact_peak_set_sha256(births: &[ExactPeakBirthV1]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-exact-peak-birth-v1\0");
    for birth in births {
        hasher.update(birth.form_ref.to_le_bytes());
        hash_len_bytes(&mut hasher, birth.normalized_surface.as_bytes());
        hasher.update([birth.class as u8]);
        hash_len_bytes(&mut hasher, birth.canonical_key.as_bytes());
        hasher.update(birth.derivation_ref.to_le_bytes());
    }
    hasher.finalize().into()
}

fn combined_target_set_digest_with_exact_peaks(
    base_digest: [u64; 2],
    exact_peak_digest: [u64; 2],
) -> [u64; 2] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-context-neutral-target-set-with-exact-peaks-v1\0");
    hasher.update(base_digest[0].to_le_bytes());
    hasher.update(base_digest[1].to_le_bytes());
    hasher.update(exact_peak_digest[0].to_le_bytes());
    hasher.update(exact_peak_digest[1].to_le_bytes());
    digest128(hasher.finalize().into())
}

fn evidence_table_identities(
    original: Option<&ExactMaterialTargetV1>,
    targets: &[ExactMaterialTargetV1],
    canonical_l1_anchor_proofs: &[CanonicalL1AnchorProofV1],
) -> PreparedEvidenceTablesV1 {
    let mut relations = Sha256::new();
    let mut groundings = Sha256::new();
    let mut derivations = Sha256::new();
    for target in original.into_iter().chain(targets) {
        for root in &target.witness_roots {
            relations.update([root.relation as u8]);
            groundings.update([root.grounding_namespace as u8]);
            groundings.update(root.grounding_ref.to_le_bytes());
            derivations.update(root.operator_ref.to_le_bytes());
            derivations.update(root.derivation_ref.to_le_bytes());
        }
    }
    for proof in canonical_l1_anchor_proofs {
        groundings.update(proof.proof_ref.to_le_bytes());
        groundings.update(proof.canonical_identity_bytes());
    }
    PreparedEvidenceTablesV1 {
        relation_table_identity: digest64(relations.finalize().into()),
        grounding_table_identity: digest64(groundings.finalize().into()),
        derivation_table_identity: digest64(derivations.finalize().into()),
        exact_relation_partition: ExactRelationPartitionProofV1::default(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "sealed identity tuple remains explicit"
)]
fn material_integrity_digest(
    observed: &str,
    package_tuple: ExactPackageTupleV1,
    original: PreparedOriginalMaterialV1,
    exact_original: Option<&ExactMaterialTargetV1>,
    targets: &[ExactMaterialTargetV1],
    canonical_l1_anchor_proofs: &[CanonicalL1AnchorProofV1],
    boundary_groundings: &[CompositeBoundaryGroundingV1],
    completeness: EnumerationCompletenessV1,
    evidence_tables: PreparedEvidenceTablesV1,
    work: EnumerationWorkCountersV1,
) -> [u64; 2] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-prepared-target-material-v2\0");
    hash_len_bytes(&mut hasher, observed.as_bytes());
    hasher.update(package_tuple.l11_sha256);
    hasher.update(package_tuple.canonical_l2_sha256);
    hasher.update(package_tuple.productive_sha256);
    hasher.update(original.exact_observed_scalars_ref.to_le_bytes());
    hasher.update(original.exact_observed_bytes_ref.to_le_bytes());
    hasher.update(original.preservation_schema.to_le_bytes());
    hasher.update([
        original.lexical_status as u8,
        original.script_token_status as u8,
        original.punctuation_status as u8,
    ]);
    hasher.update([u8::from(exact_original.is_some())]);
    if let Some(exact_original) = exact_original {
        hash_len_bytes(&mut hasher, &exact_original.canonical_bytes);
        for root in &exact_original.witness_roots {
            hasher.update(root.canonical_bytes());
        }
    }
    for target in targets {
        hash_len_bytes(&mut hasher, &target.canonical_bytes);
        for root in &target.witness_roots {
            hasher.update(root.canonical_bytes());
        }
    }
    hasher.update((canonical_l1_anchor_proofs.len() as u64).to_le_bytes());
    for proof in canonical_l1_anchor_proofs {
        hasher.update(proof.proof_ref.to_le_bytes());
        hash_len_bytes(&mut hasher, &proof.canonical_identity_bytes());
    }
    for grounding in boundary_groundings {
        hash_len_bytes(&mut hasher, &grounding.exact_bytes());
    }
    hasher.update([completeness.state() as u8, completeness.reason() as u8]);
    hasher.update([completeness.scope().kind() as u8]);
    hasher.update(
        completeness
            .scope()
            .exhaustive_partition_proof_ref()
            .map(std::num::NonZeroU32::get)
            .unwrap_or_default()
            .to_le_bytes(),
    );
    hasher.update(completeness.retained_count().to_le_bytes());
    hasher.update(completeness.logical_count_lower_bound().to_le_bytes());
    hasher.update(completeness.all_seen_digest()[0].to_le_bytes());
    hasher.update(completeness.all_seen_digest()[1].to_le_bytes());
    hash_evidence_tables(&mut hasher, evidence_tables);
    for value in [
        work.posting_visits,
        work.relation_replays,
        work.grounding_lookups,
        work.generated_logical_targets,
        work.operator_steps,
    ] {
        hasher.update(value.to_le_bytes());
    }
    digest128(hasher.finalize().into())
}

fn hash_evidence_tables(hasher: &mut Sha256, tables: PreparedEvidenceTablesV1) {
    hasher.update(tables.relation_table_identity.to_le_bytes());
    hasher.update(tables.grounding_table_identity.to_le_bytes());
    hasher.update(tables.derivation_table_identity.to_le_bytes());
    let proof = tables.exact_relation_partition;
    hasher.update(proof.proof_ref.to_le_bytes());
    hasher.update(relation_partition_proof_digest(proof));
}

fn refresh_material_integrity(material: &mut PreparedTargetMaterialShadowV1) {
    material.compact.integrity.exact_digest = material_integrity_digest(
        &material.exact_observed,
        material.package_tuple,
        material.compact.original,
        material.exact_original.as_ref(),
        &material.exact_targets,
        &material.canonical_l1_anchor_proofs,
        &material.boundary_groundings,
        material.compact.completeness,
        material.compact.evidence_tables,
        material.work,
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ExactInputFrameV1 {
    identity: InputFrameIdentityV1,
    source_window: String,
    left_context: String,
    preedit: String,
}

impl ExactInputFrameV1 {
    #[expect(
        clippy::too_many_arguments,
        reason = "existing explicit boundary contract"
    )]
    pub(super) fn new(
        focus_serial: u64,
        tail_epoch: u64,
        source_window: String,
        left_context: String,
        caret_scalar: u32,
        selection: (u32, u32),
        preedit: String,
        preedit_cursor_scalar: u32,
        layout_generation: u64,
        config_generation: u64,
        package_generation: u64,
        field_generation: u64,
    ) -> Result<Self, FrameInvalidationReasonV1> {
        let source_scalar_count = u32::try_from(source_window.chars().count())
            .map_err(|_| FrameInvalidationReasonV1::SourceWindow)?;
        let preedit_scalars = u32::try_from(preedit.chars().count())
            .map_err(|_| FrameInvalidationReasonV1::Preedit)?;
        if focus_serial == 0
            || caret_scalar > source_scalar_count
            || selection.0 > selection.1
            || selection.1 > source_scalar_count
            || preedit_cursor_scalar > preedit_scalars
            || layout_generation == 0
            || config_generation == 0
            || package_generation == 0
            || field_generation == 0
        {
            return Err(FrameInvalidationReasonV1::SourceWindow);
        }
        Ok(Self {
            identity: InputFrameIdentityV1 {
                focus_serial,
                tail_epoch,
                exact_source_window_ref: stable_bytes_ref(source_window.as_bytes()),
                exact_left_context_ref: stable_bytes_ref(left_context.as_bytes()),
                source_scalar_count,
                caret_scalar,
                selection_start_scalar: selection.0,
                selection_end_scalar: selection.1,
                exact_preedit_bytes_ref: stable_bytes_ref(preedit.as_bytes()),
                preedit_cursor_scalar,
                layout_generation,
                config_generation,
                package_generation,
                field_generation,
            },
            source_window,
            left_context,
            preedit,
        })
    }

    pub(super) fn compare_exact(&self, current: &Self) -> Result<(), FrameInvalidationReasonV1> {
        let checks = [
            (
                self.identity.focus_serial == current.identity.focus_serial,
                FrameInvalidationReasonV1::Focus,
            ),
            (
                self.identity.tail_epoch == current.identity.tail_epoch,
                FrameInvalidationReasonV1::TailEpoch,
            ),
            (
                self.identity.exact_source_window_ref == current.identity.exact_source_window_ref
                    && self.identity.source_scalar_count == current.identity.source_scalar_count
                    && self.source_window.as_bytes() == current.source_window.as_bytes(),
                FrameInvalidationReasonV1::SourceWindow,
            ),
            (
                self.identity.exact_left_context_ref == current.identity.exact_left_context_ref
                    && self.left_context.as_bytes() == current.left_context.as_bytes(),
                FrameInvalidationReasonV1::LeftContext,
            ),
            (
                self.identity.caret_scalar == current.identity.caret_scalar,
                FrameInvalidationReasonV1::Caret,
            ),
            (
                self.identity.selection_start_scalar == current.identity.selection_start_scalar
                    && self.identity.selection_end_scalar == current.identity.selection_end_scalar,
                FrameInvalidationReasonV1::Selection,
            ),
            (
                self.identity.exact_preedit_bytes_ref == current.identity.exact_preedit_bytes_ref
                    && self.identity.preedit_cursor_scalar
                        == current.identity.preedit_cursor_scalar
                    && self.preedit.as_bytes() == current.preedit.as_bytes(),
                FrameInvalidationReasonV1::Preedit,
            ),
            (
                self.identity.layout_generation == current.identity.layout_generation,
                FrameInvalidationReasonV1::LayoutGeneration,
            ),
            (
                self.identity.config_generation == current.identity.config_generation,
                FrameInvalidationReasonV1::ConfigGeneration,
            ),
            (
                self.identity.package_generation == current.identity.package_generation,
                FrameInvalidationReasonV1::PackageGeneration,
            ),
            (
                self.identity.field_generation == current.identity.field_generation,
                FrameInvalidationReasonV1::FieldGeneration,
            ),
        ];
        checks
            .into_iter()
            .find_map(|(valid, reason)| (!valid).then_some(Err(reason)))
            .unwrap_or(Ok(()))
    }

    pub(super) fn identity(&self) -> InputFrameIdentityV1 {
        self.identity
    }

    pub(super) fn source_window(&self) -> &str {
        &self.source_window
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BoundFrameTargetV1 {
    pub(super) identity: FrameTargetIdentityV1,
    pub(super) projected_target: String,
    pub(super) replayed_source_window: String,
}

#[expect(
    clippy::too_many_arguments,
    reason = "sealed evidence inputs remain explicit"
)]
pub(super) fn bind_exact_frame_target(
    material: &PreparedTargetMaterialShadowV1,
    lease: PreparedMaterialLeaseV1,
    expected: &ExactInputFrameV1,
    current: &ExactInputFrameV1,
    target_ref: usize,
    scalar_start: usize,
    scalar_len: usize,
    case_projection_id: u32,
    punctuation_projection_id: u32,
    now_monotonic_ns: u64,
) -> Result<BoundFrameTargetV1, FrameInvalidationReasonV1> {
    validate_lease(material, lease, expected, now_monotonic_ns)?;
    expected.compare_exact(current)?;
    let target = material
        .exact_targets
        .get(target_ref)
        .ok_or(FrameInvalidationReasonV1::ProjectionReplay)?;
    if target.compact != material.compact.targets.as_slice()[target_ref]
        || stable_bytes_ref(target.normalized_scalars.as_bytes())
            != target.compact.identity.normalized_scalars_ref
        || stable_bytes_ref(&target.canonical_bytes) != target.compact.identity.canonical_bytes_ref
        || target.normalized_scalars.as_bytes() != target.canonical_bytes
    {
        return Err(FrameInvalidationReasonV1::ProjectionReplay);
    }
    let source_scalars = current.source_window.chars().collect::<Vec<_>>();
    let end = scalar_start
        .checked_add(scalar_len)
        .filter(|end| *end <= source_scalars.len())
        .ok_or(FrameInvalidationReasonV1::ProjectionReplay)?;
    let source_slice = source_scalars[scalar_start..end].iter().collect::<String>();
    let projected_target = match case_projection_id {
        0 => target.normalized_scalars.clone(),
        1 => apply_word_case(&source_slice, &target.normalized_scalars),
        _ => return Err(FrameInvalidationReasonV1::ProjectionReplay),
    };
    if punctuation_projection_id != 0 {
        return Err(FrameInvalidationReasonV1::ProjectionReplay);
    }
    let replayed_source_window = source_scalars[..scalar_start]
        .iter()
        .chain(projected_target.chars().collect::<Vec<_>>().iter())
        .chain(source_scalars[end..].iter())
        .collect::<String>();
    let exact_source_slice_hash = digest128(Sha256::digest(source_slice.as_bytes()).into());
    let replacement_span = ReplacementSpanV1 {
        scalar_start: scalar_start as u32,
        scalar_len: scalar_len as u32,
        source_scalar_len: source_scalars.len() as u32,
        exact_source_slice_hash,
    };
    let mut frame_hasher = Sha256::new();
    frame_hasher.update(b"lay-frame-target-identity-v1\0");
    frame_hasher.update(material.exact_digest()[0].to_le_bytes());
    frame_hasher.update(material.exact_digest()[1].to_le_bytes());
    hash_len_bytes(&mut frame_hasher, current.source_window.as_bytes());
    hash_len_bytes(&mut frame_hasher, projected_target.as_bytes());
    frame_hasher.update((target_ref as u64).to_le_bytes());
    frame_hasher.update((scalar_start as u64).to_le_bytes());
    frame_hasher.update((scalar_len as u64).to_le_bytes());
    let identity = FrameTargetIdentityV1 {
        material_target_ref: target_ref as u16,
        reserved: 0,
        replacement_span,
        exact_projected_target_bytes_ref: stable_bytes_ref(projected_target.as_bytes()),
        case_projection_id,
        punctuation_projection_id,
        frame_identity_hash: digest128(frame_hasher.finalize().into()),
    };
    Ok(BoundFrameTargetV1 {
        identity,
        projected_target,
        replayed_source_window,
    })
}

pub(super) fn validate_lease(
    material: &PreparedTargetMaterialShadowV1,
    lease: PreparedMaterialLeaseV1,
    frame: &ExactInputFrameV1,
    now_monotonic_ns: u64,
) -> Result<(), FrameInvalidationReasonV1> {
    if lease.material_key != material.compact.key
        || lease.integrity_digest != material.exact_digest()
        || lease.field_generation != frame.identity.field_generation
        || lease.runtime_owner_lease_identity == 0
        || lease.monotonic_epoch_identity == [0; 2]
        || lease.consumer != LeaseConsumerStateV1::FrameSettlement
        || lease.expires_at_monotonic_ns < now_monotonic_ns
        || lease.lease_identity == 0
        || frame.identity.package_generation != material.compact.key.package_generation
    {
        return Err(FrameInvalidationReasonV1::Lease);
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn set_original_status_for_test(
    material: &mut PreparedTargetMaterialShadowV1,
    lexical_status: crate::typing_transition::target_evidence::PreparedOriginalLexicalStatusV1,
    script_token_status: crate::typing_transition::target_evidence::PreparedOriginalScriptTokenStatusV1,
    punctuation_status: crate::typing_transition::target_evidence::PreparedOriginalPunctuationStatusV1,
) {
    material.compact.original.lexical_status = lexical_status;
    material.compact.original.script_token_status = script_token_status;
    material.compact.original.punctuation_status = punctuation_status;
}

#[derive(Clone, Debug)]
struct LeaseFieldStateV1 {
    material_key: PreparedMaterialKeyV1,
    integrity_digest: [u64; 2],
    field_generation: u64,
    allocation_identity: u64,
    consumers: u8,
}

#[derive(Clone, Debug, Default)]
pub(super) struct PreparedMaterialLeaseArenaV1 {
    fields: Vec<LeaseFieldStateV1>,
    issued_leases: Vec<PreparedMaterialLeaseV1>,
    next_allocation_identity: u64,
    next_lease_identity: u64,
}

impl PreparedMaterialLeaseArenaV1 {
    pub(super) fn pin(
        &mut self,
        material: &PreparedTargetMaterialShadowV1,
        field_generation: u64,
        runtime_owner_lease_identity: u64,
        monotonic_epoch_identity: [u64; 2],
        expires_at_monotonic_ns: u64,
        consumer: LeaseConsumerStateV1,
    ) -> Option<PreparedMaterialLeaseV1> {
        let index = self.fields.iter().position(|field| {
            field.material_key == material.compact.key
                && field.integrity_digest == material.exact_digest()
                && field.field_generation == field_generation
        });
        let index = match index {
            Some(index) => index,
            None if self.fields.len() < MAX_PINNED_PREPARED_FIELDS => {
                self.next_allocation_identity =
                    self.next_allocation_identity.wrapping_add(1).max(1);
                self.fields.push(LeaseFieldStateV1 {
                    material_key: material.compact.key,
                    integrity_digest: material.exact_digest(),
                    field_generation,
                    allocation_identity: self.next_allocation_identity,
                    consumers: 0,
                });
                self.fields.len() - 1
            }
            None => return None,
        };
        let field = &mut self.fields[index];
        if usize::from(field.consumers) == MAX_LEASE_CONSUMERS_PER_FIELD {
            return None;
        }
        field.consumers += 1;
        self.next_lease_identity = self.next_lease_identity.wrapping_add(1).max(1);
        let lease = PreparedMaterialLeaseV1::new(
            field.material_key,
            field.integrity_digest,
            field.field_generation,
            field.allocation_identity,
            runtime_owner_lease_identity,
            monotonic_epoch_identity,
            expires_at_monotonic_ns,
            self.next_lease_identity,
            consumer,
        )?;
        self.issued_leases.push(lease);
        Some(lease)
    }

    pub(super) fn contains(&self, lease: PreparedMaterialLeaseV1) -> bool {
        self.issued_leases.contains(&lease)
    }
}

fn hash_len_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn digest64(digest: [u8; 32]) -> u64 {
    u64::from_le_bytes(digest[..8].try_into().expect("SHA-256 prefix"))
}

fn digest128(digest: [u8; 32]) -> [u64; 2] {
    [
        u64::from_le_bytes(digest[..8].try_into().expect("SHA-256 prefix")),
        u64::from_le_bytes(digest[8..16].try_into().expect("SHA-256 prefix")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::l2_field::productive_v1::boundary_birth::{
        ExactBoundaryPartGroundingV1, TypedBoundaryBirthEnumerationV1, TypedBoundaryBirthV1,
    };
    use crate::nanda_wave::l2_field::productive_v1::calibrate::{
        CandidateProvenanceClassV1, CandidateRankOriginV1, ProductiveCalibratedVerdictV1,
    };
    use crate::nanda_wave::l2_field::productive_v1::geometry::GeometryTerminalEvidenceV1;
    use crate::nanda_wave::l2_field::productive_v1::packaged_runtime::PackagedProductiveReadoutV1;
    use crate::nanda_wave::l2_field::productive_v1::types::ProductiveCandidateIdentityV1;
    use crate::typing_transition::target_evidence::EnumerationStateV1;

    fn package_tuple() -> ExactPackageTupleV1 {
        ExactPackageTupleV1 {
            l11_sha256: [1; 32],
            canonical_l2_sha256: [2; 32],
            productive_sha256: [3; 32],
        }
    }

    fn candidate(surface: &str, id: u32) -> PackagedProductiveCandidateV1 {
        let identity = ProductiveCandidateIdentityV1 {
            lemma_id: id,
            paradigm_id: id + 10,
            program_id: id + 20,
            target_slot_id: id + 30,
            normalized_surface_id: stable_bytes_ref(surface.as_bytes()),
            variant_id: 1,
        };
        PackagedProductiveCandidateV1 {
            identity,
            equivalent_identities: vec![identity],
            normalized_surface: surface.into(),
            score_q16: 0,
            geometry: GeometryTerminalEvidenceV1::default(),
            provenance: CandidateProvenanceClassV1::ColdLemmaBinding,
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

    fn enumeration(
        candidates: Vec<PackagedProductiveCandidateV1>,
    ) -> ContextNeutralProductiveEnumerationV1 {
        let count = candidates.len() as u64;
        ContextNeutralProductiveEnumerationV1 {
            readout: PackagedProductiveReadoutV1 {
                verdict: ProductiveCalibratedVerdictV1::Abstain {
                    suggestions: Vec::new(),
                    productive_overflow: false,
                },
                candidates,
                logical_terminal_count: count,
                logical_surface_basin_count: count,
                integrity_error: None,
            },
            productive_work: EnumerationWorkCountersV1 {
                relation_replays: count,
                generated_logical_targets: count,
                ..EnumerationWorkCountersV1::default()
            },
            aggregate_work: EnumerationWorkCountersV1 {
                relation_replays: count,
                generated_logical_targets: count,
                ..EnumerationWorkCountersV1::default()
            },
            work_budget_exceeded: false,
        }
    }

    fn contour_birth(
        surface: &str,
        relation: TargetRelationV1,
        identity: u32,
    ) -> TypedContourBirthV1 {
        TypedContourBirthV1 {
            normalized_surface: surface.to_string(),
            grounding_namespace: GroundingNamespaceV1::L11Terminal,
            grounding_ref: identity,
            relation,
            operator_ref: identity.wrapping_add(100),
            derivation_ref: identity.wrapping_add(200),
            verdict_membership: VerdictMembershipV1::Born,
            support_milli: 0,
        }
    }

    fn contour_enumeration(births: Vec<TypedContourBirthV1>) -> TypedContourBirthEnumerationV1 {
        TypedContourBirthEnumerationV1 {
            logical_match_count: births.len(),
            births,
            canonical_l1_anchor_proofs: Vec::new(),
            work: EnumerationWorkCountersV1::default(),
            all_seen_digest: [17, 19],
            overflow_reason: None,
        }
    }

    fn canonical_anchor_contour_enumeration(
        surface: &str,
        target_form_ref: u32,
        anchor_form_ref: u32,
        lemma_id: Option<u32>,
        terminal_id: u32,
        kind: CanonicalL1AnchorKindV1,
        membership: VerdictMembershipV1,
    ) -> TypedContourBirthEnumerationV1 {
        let proof = CanonicalL1AnchorProofV1::new(
            surface.to_string(),
            target_form_ref,
            anchor_form_ref,
            lemma_id,
            terminal_id,
            kind,
            package_tuple().l11_sha256,
            package_tuple().canonical_l2_sha256,
        )
        .expect("test anchor proof");
        TypedContourBirthEnumerationV1 {
            births: vec![TypedContourBirthV1 {
                normalized_surface: surface.to_string(),
                grounding_namespace: GroundingNamespaceV1::CanonicalL1Anchor,
                grounding_ref: proof.proof_ref,
                relation: TargetRelationV1::L11Restoration,
                operator_ref: 0x5348_0000 | u32::from(TargetRelationV1::L11Restoration as u8),
                derivation_ref: stable_bytes_ref(surface.as_bytes()),
                verdict_membership: membership,
                support_milli: 1_000,
            }],
            canonical_l1_anchor_proofs: vec![proof],
            work: EnumerationWorkCountersV1::default(),
            logical_match_count: 1,
            all_seen_digest: [31, 37],
            overflow_reason: None,
        }
    }

    fn boundary_birth(
        surface: &str,
        relation: TargetRelationV1,
        identity: u32,
    ) -> TypedBoundaryBirthV1 {
        let parts = surface
            .split(' ')
            .map(|part| ExactBoundaryPartGroundingV1 {
                normalized_surface: part.to_string(),
                grounding_namespace: GroundingNamespaceV1::CanonicalForm,
                grounding_ref: stable_bytes_ref(part.as_bytes()),
            })
            .collect::<Vec<_>>();
        TypedBoundaryBirthV1 {
            normalized_surface: surface.to_string(),
            relation,
            operator_ref: identity.wrapping_add(300),
            grounding_ref: identity.wrapping_add(400),
            derivation_ref: identity.wrapping_add(500),
            composite_grounding: CompositeBoundaryGroundingV1 {
                ordered_part_groundings: parts,
                exact_segmentation_scalars: vec![1],
                separator_profile_id: ASCII_SPACE_SEPARATOR_PROFILE,
                merged_target_grounding: None,
            },
        }
    }

    fn boundary_enumeration(births: Vec<TypedBoundaryBirthV1>) -> TypedBoundaryBirthEnumerationV1 {
        let logical_match_count = births
            .iter()
            .map(|birth| birth.normalized_surface.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        TypedBoundaryBirthEnumerationV1 {
            births,
            work: EnumerationWorkCountersV1::default(),
            logical_match_count,
            all_seen_digest: [23, 29],
            overflow_reason: None,
        }
    }

    fn exact_peak_enumeration(surface: &str, form_ref: u32) -> ExactPeakBirthEnumerationV1 {
        ExactPeakBirthEnumerationV1::from_candidates(vec![ExactPeakCandidateInputV1 {
            form_ref,
            normalized_surface: surface.to_string(),
            certificates: vec![Phase7dCertificateEvidence {
                class: Phase7dCertificateClass::MissingLetter,
                canonical_key: format!("missing-letter:{surface}"),
            }],
        }])
        .expect("valid exact peak")
    }

    fn certified_exact_peak_enumeration(
        observed: &str,
        surface: &str,
        form_ref: u32,
    ) -> ExactPeakBirthEnumerationV1 {
        let certificates = Phase7dCertificateOracle::new(observed)
            .expect("test oracle")
            .certificate_evidence(surface)
            .expect("test certificate");
        ExactPeakBirthEnumerationV1::from_candidates(vec![ExactPeakCandidateInputV1 {
            form_ref,
            normalized_surface: surface.to_string(),
            certificates,
        }])
        .expect("valid exact peak")
        .with_test_search_proof(observed, package_tuple().canonical_l2_sha256)
        .expect("completed test exact search")
    }

    fn material() -> PreparedTargetMaterialShadowV1 {
        prepare_context_neutral_productive_material(
            "слово",
            package_tuple(),
            enumeration(vec![candidate("форма", 1)]),
        )
        .expect("material")
    }

    fn insertion_surfaces(observed: &str) -> Vec<String> {
        let original = observed.chars().collect::<Vec<_>>();
        let mut surfaces = BTreeSet::new();
        for position in 0..=original.len() {
            for symbol in 'а'..='я' {
                let mut target = original.clone();
                target.insert(position, symbol);
                surfaces.insert(target.iter().collect::<String>());
            }
        }
        surfaces.into_iter().collect()
    }

    #[test]
    fn bounded_common_exact_view_preserves_raw_identity_without_proof_authority() {
        use super::super::cohort_compare::test_raw_peaks_with_unsupported_overflow;

        for targets in [vec!["форма"], Vec::new()] {
            let raw = test_raw_peaks_with_unsupported_overflow("форм", &targets)
                .with_test_search_proof("форм", package_tuple().canonical_l2_sha256)
                .expect("complete search over an overfull raw set");
            let raw_count = raw.logical_match_count;
            let raw_digest = raw.all_seen_digest;
            let (view, incomplete) = raw
                .clone()
                .common_field_projection("форм", package_tuple().canonical_l2_sha256);
            let incomplete = incomplete.expect("raw overflow remains visible");
            assert_eq!(view.normalized_surfaces().collect::<Vec<_>>(), targets);
            assert!(view.search_proof.is_none());
            assert_eq!(view.logical_match_count, raw_count);
            assert_eq!(view.all_seen_digest, raw_digest);
            assert_eq!(incomplete.all_seen_digest(), raw_digest);
            assert_eq!(
                usize::from(incomplete.logical_count_lower_bound()),
                raw_count
            );
            assert!(view
                .clone()
                .with_test_search_proof("форм", package_tuple().canonical_l2_sha256)
                .is_err());
            let mut reattached = view.clone();
            reattached.search_proof = raw.search_proof.clone();
            assert!(reattached
                .authority_partition("форм", package_tuple().canonical_l2_sha256)
                .is_none());

            let common = prepare_context_neutral_productive_material_with_contours_and_exact_peaks(
                "форм",
                package_tuple(),
                enumeration(Vec::new()),
                TypedContourBirthEnumerationV1::complete_empty(),
                view,
            )
            .expect("bounded common material");
            assert_eq!(common.completeness().state(), EnumerationStateV1::Overflow);
            assert_eq!(
                common.completeness().reason(),
                IncompletenessReasonV1::StorageCapacity
            );
            assert_eq!(common.exact_target_surfaces().collect::<Vec<_>>(), targets);
            assert_eq!(
                usize::from(common.completeness().logical_count_lower_bound()),
                raw_count
            );
            assert!(common.exact_search_proof.is_none());
            assert!(!common.validates_relation_partition_proof());
            if targets.is_empty() {
                let empty =
                    prepare_context_neutral_productive_material_with_contours_and_exact_peaks(
                        "форм",
                        package_tuple(),
                        enumeration(Vec::new()),
                        TypedContourBirthEnumerationV1::complete_empty(),
                        ExactPeakBirthEnumerationV1::complete_empty(),
                    )
                    .expect("no exact enumeration");
                assert_ne!(
                    common.completeness().all_seen_digest(),
                    empty.completeness().all_seen_digest()
                );
                assert_eq!(
                    common.completeness().all_seen_digest(),
                    combined_target_set_digest_with_exact_peaks(
                        empty.completeness().all_seen_digest(),
                        raw_digest
                    )
                );
            }

            for mutation in 0..3 {
                let mut broken = raw.clone();
                match mutation {
                    0 => broken.logical_match_count -= 1,
                    1 => broken.all_seen_digest[0] ^= 1,
                    _ => {
                        broken.births.pop();
                    }
                }
                assert!(broken
                    .authority_partition("форм", package_tuple().canonical_l2_sha256)
                    .is_none());
                assert!(broken
                    .with_test_search_proof("форм", package_tuple().canonical_l2_sha256)
                    .is_err());
            }
            for reason in [
                IncompletenessReasonV1::WorkBudgetExceeded,
                IncompletenessReasonV1::UpstreamIncomplete,
                IncompletenessReasonV1::IntegrityFailure,
            ] {
                let mut incomplete = raw.clone();
                incomplete.overflow_reason = Some(reason);
                assert!(incomplete
                    .authority_partition("форм", package_tuple().canonical_l2_sha256)
                    .is_none());
                assert!(incomplete
                    .with_test_search_proof("форм", package_tuple().canonical_l2_sha256)
                    .is_err());
            }
        }
    }

    #[test]
    fn completed_exact_partition_refuses_supported_target_and_root_overflow() {
        use super::super::cohort_compare::test_raw_peaks_with_unsupported_overflow;

        let surfaces = insertion_surfaces("форм");
        assert!(surfaces.len() > MAX_TARGETS_PER_FIELD);
        let targets = surfaces
            .iter()
            .take(MAX_TARGETS_PER_FIELD + 1)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let target_overflow = test_raw_peaks_with_unsupported_overflow("форм", &targets);
        let root_overflow = test_raw_peaks_with_unsupported_overflow("аааа", &["ааааа"]);
        for (observed, raw, target_count, root_count) in [
            ("форм", target_overflow, MAX_TARGETS_PER_FIELD + 1, None),
            (
                "аааа",
                root_overflow,
                1,
                Some(MAX_TARGET_WITNESSES_PER_TARGET + 1),
            ),
        ] {
            let supported = replayed_supported_births(observed, &raw.births)
                .expect("actual certificate replay");
            let roots = exact_peak_root_counts(&supported);
            assert_eq!(roots.len(), target_count);
            if let Some(root_count) = root_count {
                assert_eq!(roots.values().copied().max(), Some(root_count));
            }
            assert!(raw.capacity_exceeded());
            let error = raw
                .clone()
                .with_test_search_proof(observed, package_tuple().canonical_l2_sha256)
                .expect_err("no authority from a truncated supported partition");
            assert!(
                error.contains("supported exact partition exceeds storage capacity"),
                "{error}"
            );
            let (view, marker) =
                raw.common_field_projection(observed, package_tuple().canonical_l2_sha256);
            assert!(view.is_empty());
            assert!(view.search_proof.is_none());
            assert!(marker.is_some());
            assert!(view
                .authority_partition(observed, package_tuple().canonical_l2_sha256)
                .is_none());
        }
    }

    #[test]
    fn exact_projection_preserves_combined_target_and_anchor_root_capacity() {
        use super::super::cohort_compare::test_raw_peaks_with_unsupported_overflow;

        let surfaces = insertion_surfaces("форм");
        let exact_count = MAX_TARGETS_PER_FIELD - 4;
        let targets = surfaces
            .iter()
            .take(exact_count)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let raw = test_raw_peaks_with_unsupported_overflow("форм", &targets)
            .with_test_search_proof("форм", package_tuple().canonical_l2_sha256)
            .expect("supported lane alone fits");
        for grounded_count in [4, 5] {
            let (view, _) = raw
                .clone()
                .common_field_projection("форм", package_tuple().canonical_l2_sha256);
            let grounded = (0..grounded_count)
                .map(|id| {
                    let mut birth = contour_birth(
                        &format!("grounded-{id}"),
                        TargetRelationV1::L11Restoration,
                        id,
                    );
                    birth.verdict_membership = VerdictMembershipV1::L11Tied;
                    birth
                })
                .collect();
            let result = prepare_context_neutral_productive_material_with_contours_and_exact_peaks(
                "форм",
                package_tuple(),
                enumeration(Vec::new()),
                contour_enumeration(grounded),
                view,
            );
            if grounded_count == 4 {
                let material = result.expect("mandatory union at capacity");
                let retained = material.exact_target_surfaces().collect::<BTreeSet<_>>();
                assert_eq!(retained.len(), MAX_TARGETS_PER_FIELD);
                assert!(targets.iter().all(|target| retained.contains(target)));
                for id in 0..grounded_count {
                    assert!(retained.contains(format!("grounded-{id}").as_str()));
                }
                assert_eq!(
                    material.completeness().state(),
                    EnumerationStateV1::Overflow
                );
            } else {
                assert!(result
                    .expect_err("mandatory union cannot be silently truncated")
                    .contains("74-target capacity"));
            }
        }

        let raw = test_raw_peaks_with_unsupported_overflow("ааа", &["аааа"])
            .with_test_search_proof("ааа", package_tuple().canonical_l2_sha256)
            .expect("four supported roots fit before grounding");
        let supported = raw
            .authority_partition("ааа", package_tuple().canonical_l2_sha256)
            .expect("bounded supported roots");
        assert_eq!(supported.births.len(), MAX_TARGET_WITNESSES_PER_TARGET);
        let form_ref = supported.births[0].form_ref;
        let material = prepare_frame_bound_lexical_authority_material(
            "ааа",
            package_tuple(),
            enumeration(Vec::new()),
            canonical_anchor_contour_enumeration(
                "аааа",
                form_ref,
                form_ref,
                Some(7),
                71,
                CanonicalL1AnchorKindV1::Direct,
                VerdictMembershipV1::Grounded,
            ),
            raw,
        )
        .expect("root overflow remains an explicit failed material");
        assert_eq!(material.exact_targets.len(), 1);
        assert_eq!(
            material.exact_targets[0].witness_roots.len(),
            MAX_TARGET_WITNESSES_PER_TARGET + 1
        );
        assert!(!material.validates_relation_partition_proof());
        assert_eq!(material.completeness().state(), EnumerationStateV1::Failed);
    }

    fn grounded_relation_partition_material() -> PreparedTargetMaterialShadowV1 {
        prepare_frame_bound_lexical_authority_material(
            "targe",
            package_tuple(),
            enumeration(Vec::new()),
            canonical_anchor_contour_enumeration(
                "target",
                41,
                41,
                Some(7),
                71,
                CanonicalL1AnchorKindV1::Direct,
                VerdictMembershipV1::L11Winner,
            ),
            certified_exact_peak_enumeration("targe", "target", 41),
        )
        .expect("grounded exact relation partition")
    }

    fn frame(material: &PreparedTargetMaterialShadowV1) -> ExactInputFrameV1 {
        ExactInputFrameV1::new(
            11,
            7,
            "Слово,".to_string(),
            "левый контекст".to_string(),
            5,
            (5, 5),
            String::new(),
            0,
            3,
            4,
            material.compact.key.package_generation,
            9,
        )
        .expect("frame")
    }

    fn completed_empty_search_with_work(
        observed: &str,
        work: ExactSearchStructuralWorkV1,
        retrieved_form_count: usize,
    ) -> Result<ExactPeakBirthEnumerationV1, String> {
        let lanes = Phase7dCertificateOracle::new(observed)
            .expect("test oracle")
            .retrieval_lanes();
        let retrieved_form_refs = (0..retrieved_form_count)
            .map(|form_ref| u32::try_from(form_ref).expect("test form ref fits u32"))
            .collect();
        ExactPeakBirthEnumerationV1::complete_empty().with_completed_search_proof(
            observed,
            package_tuple().canonical_l2_sha256,
            [0x5a; 32],
            &lanes,
            retrieved_form_refs,
            work,
            &[],
            &[],
        )
    }

    #[test]
    fn td117_completed_exact_search_replays_every_lane_local_budget() {
        let lanes = Phase7dCertificateOracle::new("ghbdtn")
            .expect("layout oracle")
            .retrieval_lanes();
        assert_eq!(lanes.len(), EXACT_RELATION_SEARCH_MAX_LANES);
        let aggregate_terminal_limit = EXACT_RELATION_SEARCH_MAX_TERMINALS
            .checked_mul(lanes.len())
            .expect("test aggregate terminal limit");
        let at_limit = ExactSearchStructuralWorkV1 {
            lane_count: lanes.len() as u64,
            maximum_lane_product_states: EXACT_RELATION_SEARCH_MAX_PRODUCT_STATES as u64,
            maximum_lane_terminals: EXACT_RELATION_SEARCH_MAX_TERMINALS as u64,
            expanded_product_states: (EXACT_RELATION_SEARCH_MAX_PRODUCT_STATES * lanes.len())
                as u64,
            maximum_scratch_bytes: EXACT_RELATION_SEARCH_MAX_SCRATCH_BYTES as u64,
            ..ExactSearchStructuralWorkV1::default()
        };
        assert!(
            completed_empty_search_with_work("ghbdtn", at_limit, aggregate_terminal_limit,).is_ok()
        );

        for (dimension, forged, retrieved_form_count) in [
            (
                "product-states",
                ExactSearchStructuralWorkV1 {
                    maximum_lane_product_states: (EXACT_RELATION_SEARCH_MAX_PRODUCT_STATES + 1)
                        as u64,
                    ..at_limit
                },
                aggregate_terminal_limit,
            ),
            (
                "expanded-product-states",
                ExactSearchStructuralWorkV1 {
                    expanded_product_states: at_limit.expanded_product_states + 1,
                    ..at_limit
                },
                aggregate_terminal_limit,
            ),
            (
                "terminals",
                ExactSearchStructuralWorkV1 {
                    maximum_lane_terminals: (EXACT_RELATION_SEARCH_MAX_TERMINALS + 1) as u64,
                    ..at_limit
                },
                aggregate_terminal_limit,
            ),
            (
                "retrieved-terminal-population",
                at_limit,
                aggregate_terminal_limit + 1,
            ),
            (
                "scratch",
                ExactSearchStructuralWorkV1 {
                    maximum_scratch_bytes: (EXACT_RELATION_SEARCH_MAX_SCRATCH_BYTES + 1) as u64,
                    ..at_limit
                },
                aggregate_terminal_limit,
            ),
            (
                "lane-count",
                ExactSearchStructuralWorkV1 {
                    lane_count: (EXACT_RELATION_SEARCH_MAX_LANES + 1) as u64,
                    ..at_limit
                },
                aggregate_terminal_limit,
            ),
        ] {
            assert!(
                completed_empty_search_with_work("ghbdtn", forged, retrieved_form_count).is_err(),
                "over-budget {dimension} proof was accepted"
            );
        }
    }

    #[test]
    fn prepared_material_is_exact_and_order_independent() {
        let left = prepare_context_neutral_productive_material(
            "слово",
            package_tuple(),
            enumeration(vec![candidate("форма", 1), candidate("формы", 2)]),
        )
        .unwrap();
        let right = prepare_context_neutral_productive_material(
            "слово",
            package_tuple(),
            enumeration(vec![candidate("формы", 2), candidate("форма", 1)]),
        )
        .unwrap();
        assert_eq!(left, right);
        assert_eq!(left.completeness().state(), EnumerationStateV1::Complete);
    }

    #[test]
    fn owned_witness_root_match_rejects_mismatched_or_stale_compact_projection() {
        let material = prepare_context_neutral_productive_material_with_contours(
            "source",
            package_tuple(),
            enumeration(Vec::new()),
            contour_enumeration(vec![contour_birth(
                "target",
                TargetRelationV1::MissingLetter,
                17,
            )]),
        )
        .unwrap();
        let witness = material.compact().targets.as_slice()[0]
            .witnesses
            .witnesses()[0];

        assert!(material.exact_witness_root_matches(0, 0, witness));

        let wrong_grounding = TargetWitnessV1::new(
            witness.relation,
            witness.grounding_namespace,
            witness.verdict_membership,
            witness.flags,
            witness.operator_ref,
            witness.grounding_ref.wrapping_add(1),
            witness.derivation_ref,
            witness.support_milli,
            witness.provenance_annotations,
        );
        assert!(!material.exact_witness_root_matches(0, 0, wrong_grounding));

        let mut stale = witness;
        stale.semantic_root_accelerator ^= u32::MAX;
        assert!(!material.exact_witness_root_matches(0, 0, stale));
    }

    #[test]
    fn frame_bound_authority_material_uses_complete_exact_relation_partition() {
        let mut l11_only = contour_birth("l11-only", TargetRelationV1::L11Restoration, 77);
        l11_only.verdict_membership = VerdictMembershipV1::L11Winner;
        let material = prepare_frame_bound_lexical_authority_material(
            "targe",
            package_tuple(),
            enumeration(vec![candidate("neutral-target", 1)]),
            contour_enumeration(vec![l11_only]),
            certified_exact_peak_enumeration("targe", "target", 41),
        )
        .unwrap();

        assert_eq!(
            material.exact_target_surfaces().collect::<Vec<_>>(),
            vec!["target"]
        );
        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Complete
        );
        assert_eq!(
            material.completeness().scope().kind(),
            crate::typing_transition::target_evidence::CompletenessScopeKindV1::RelationPartition
        );
        assert!(material
            .completeness()
            .scope()
            .exhaustive_partition_proof_ref()
            .is_some());
        assert!(material.validates_relation_partition_proof());
    }

    #[test]
    fn td117_relation_partition_rejects_proof_ref_without_full_proof() {
        let valid = grounded_relation_partition_material();
        assert!(valid.validates_relation_partition_proof());
        let proof_ref = valid
            .compact
            .evidence_tables
            .exact_relation_partition
            .proof_ref;

        let mut forged = valid;
        forged.compact.evidence_tables.exact_relation_partition = ExactRelationPartitionProofV1 {
            proof_ref,
            ..ExactRelationPartitionProofV1::default()
        };
        refresh_material_integrity(&mut forged);

        assert!(!forged.validates_relation_partition_proof());
    }

    #[test]
    fn td117_relation_partition_rejects_every_bound_proof_dimension_tamper() {
        let valid = grounded_relation_partition_material();
        assert!(valid.validates_relation_partition_proof());
        let base = valid.compact.evidence_tables.exact_relation_partition;
        let mut mutations = Vec::new();

        let mut proof = base;
        proof.policy_version = proof.policy_version.wrapping_add(1);
        mutations.push(("policy-version", proof));
        let mut proof = base;
        proof.policy_identity[0] ^= u64::MAX;
        mutations.push(("policy-identity", proof));
        let mut proof = base;
        proof.exact_search_proof_digest[0] ^= u64::MAX;
        mutations.push(("search-proof-digest", proof));
        let mut proof = base;
        proof.target_root_set_digest[0] ^= u64::MAX;
        mutations.push(("target-root-digest", proof));
        let mut proof = base;
        proof.logical_target_count = proof.logical_target_count.wrapping_add(1);
        mutations.push(("logical-target-count", proof));
        let mut proof = base;
        proof.retained_target_count = proof.retained_target_count.wrapping_add(1);
        mutations.push(("retained-target-count", proof));
        let mut proof = base;
        proof.logical_certificate_count = proof.logical_certificate_count.wrapping_add(1);
        mutations.push(("logical-certificate-count", proof));
        let mut proof = base;
        proof.retained_certificate_count = proof.retained_certificate_count.wrapping_add(1);
        mutations.push(("retained-certificate-count", proof));
        let mut proof = base;
        proof.per_target_root_counts[0] = proof.per_target_root_counts[0].wrapping_add(1);
        mutations.push(("per-target-root-count", proof));

        for (dimension, proof) in mutations {
            let mut forged = valid.clone();
            forged.compact.evidence_tables.exact_relation_partition = proof;
            refresh_material_integrity(&mut forged);
            assert!(
                !forged.validates_relation_partition_proof(),
                "tampered {dimension} was accepted"
            );
        }
    }

    #[test]
    fn td117_relation_partition_original_is_clean_damaged_or_unknown_from_grounding() {
        let clean = prepare_frame_bound_lexical_authority_material(
            "targe",
            package_tuple(),
            enumeration(Vec::new()),
            canonical_anchor_contour_enumeration(
                "targe",
                40,
                40,
                Some(7),
                70,
                CanonicalL1AnchorKindV1::Direct,
                VerdictMembershipV1::L11Winner,
            ),
            certified_exact_peak_enumeration("targe", "target", 41),
        )
        .expect("clean-original partition");
        let damaged = grounded_relation_partition_material();
        let unknown = prepare_frame_bound_lexical_authority_material(
            "targe",
            package_tuple(),
            enumeration(Vec::new()),
            TypedContourBirthEnumerationV1::complete_empty(),
            certified_exact_peak_enumeration("targe", "target", 41),
        )
        .expect("ungrounded partition");

        assert!(clean.validates_relation_partition_proof());
        assert!(damaged.validates_relation_partition_proof());
        assert!(unknown.validates_relation_partition_proof());
        assert_eq!(
            clean.compact.original.lexical_status,
            PreparedOriginalLexicalStatusV1::Clean
        );
        assert_eq!(
            damaged.compact.original.lexical_status,
            PreparedOriginalLexicalStatusV1::Damaged
        );
        assert_eq!(
            unknown.compact.original.lexical_status,
            PreparedOriginalLexicalStatusV1::Unknown
        );
    }

    #[test]
    fn exact_peak_birth_enters_complete_material_with_certificate_identity() {
        let material = prepare_context_neutral_productive_material_with_contours_and_exact_peaks(
            "source",
            package_tuple(),
            enumeration(Vec::new()),
            TypedContourBirthEnumerationV1::complete_empty(),
            exact_peak_enumeration("target", 41),
        )
        .unwrap();

        assert_eq!(
            material.exact_peak_candidate_rows(),
            vec![(41, "target".to_string())]
        );
        assert_eq!(
            material.exact_peak_certificate_rows(),
            vec![(
                41,
                "target".to_string(),
                ExactPeakCertificateClassV1::MissingLetter as u8,
                "missing-letter:target".to_string(),
            )]
        );
        assert_eq!(
            material.exact_target_surfaces().collect::<Vec<_>>(),
            vec!["target"]
        );
        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Complete
        );
    }

    #[test]
    fn material_capacity_retains_exact_and_grounded_before_productive_tail() {
        let productive = (1..=32)
            .map(|id| candidate(&format!("productive-{id:02}"), id))
            .collect::<Vec<_>>();
        let grounded = (1..=13)
            .map(|id| {
                let mut birth = contour_birth(
                    &format!("grounded-{id:02}"),
                    TargetRelationV1::L11Restoration,
                    id,
                );
                birth.verdict_membership = VerdictMembershipV1::L11Tied;
                birth.support_milli = 1_000;
                birth
            })
            .collect::<Vec<_>>();
        let exact_peaks = ExactPeakBirthEnumerationV1::from_candidates(
            (1..=56)
                .map(|id| ExactPeakCandidateInputV1 {
                    form_ref: 1_000 + id,
                    normalized_surface: format!("exact-{id:02}"),
                    certificates: vec![Phase7dCertificateEvidence {
                        class: Phase7dCertificateClass::MissingLetter,
                        canonical_key: format!("missing-letter:exact-{id:02}"),
                    }],
                })
                .collect(),
        )
        .expect("exact peaks");
        let material = prepare_context_neutral_productive_material_with_contours_and_exact_peaks(
            "source",
            package_tuple(),
            enumeration(productive),
            contour_enumeration(grounded),
            exact_peaks,
        )
        .expect("bounded mandatory-first material");
        let retained = material.exact_target_surfaces().collect::<BTreeSet<_>>();

        assert_eq!(retained.len(), MAX_TARGETS_PER_FIELD);
        for id in 1..=13 {
            assert!(retained.contains(format!("grounded-{id:02}").as_str()));
        }
        for id in 1..=56 {
            assert!(retained.contains(format!("exact-{id:02}").as_str()));
        }
        for id in 1..=5 {
            assert!(retained.contains(format!("productive-{id:02}").as_str()));
        }
        assert!(!retained.contains("productive-06"));
        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Overflow
        );
        assert_eq!(
            material.completeness().reason(),
            IncompletenessReasonV1::StorageCapacity
        );
    }

    #[test]
    fn incomplete_exact_route_preserves_grounded_l11_target() {
        let mut grounded = contour_birth("grounded", TargetRelationV1::L11Restoration, 71);
        grounded.verdict_membership = VerdictMembershipV1::L11Winner;
        grounded.support_milli = 1_000;
        let material = prepare_context_neutral_productive_material_with_contours_and_exact_peaks(
            "source",
            package_tuple(),
            enumeration(Vec::new()),
            contour_enumeration(vec![grounded]),
            ExactPeakBirthEnumerationV1::incomplete(IncompletenessReasonV1::WorkBudgetExceeded),
        )
        .unwrap();

        assert_eq!(
            material.exact_target_surfaces().collect::<Vec<_>>(),
            vec!["grounded"]
        );
        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Overflow
        );
        assert_eq!(
            material.completeness().reason(),
            IncompletenessReasonV1::WorkBudgetExceeded
        );
        assert!(material.exact_targets[0]
            .witness_roots
            .iter()
            .any(|root| root.verdict_membership == VerdictMembershipV1::L11Winner));
    }

    #[test]
    fn storage_overflow_retains_the_same_seventy_four_target_prefix() {
        let candidates = (1..=75)
            .map(|id| candidate(&format!("surface-{id:03}"), id))
            .collect::<Vec<_>>();
        let material = prepare_context_neutral_productive_material(
            "source",
            package_tuple(),
            enumeration(candidates),
        )
        .unwrap();
        assert_eq!(material.compact.targets.len(), 74);
        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Overflow
        );
        assert_eq!(
            material.completeness().reason(),
            IncompletenessReasonV1::StorageCapacity
        );
        assert_eq!(material.exact_target_surfaces().last(), Some("surface-074"));
    }

    #[test]
    fn exact_original_root_is_integrity_bound_outside_replacement_capacity() {
        let replacements = || {
            (1..=MAX_TARGETS_PER_FIELD as u32)
                .map(|id| candidate(&format!("replacement-{id:03}"), id))
                .collect::<Vec<_>>()
        };
        let original_birth = |grounding_ref| {
            let mut birth =
                contour_birth("source", TargetRelationV1::L11Restoration, grounding_ref);
            birth.verdict_membership = VerdictMembershipV1::L11Winner;
            birth.support_milli = 1_000;
            birth
        };
        let left = prepare_context_neutral_productive_material_with_contours(
            "source",
            package_tuple(),
            enumeration(replacements()),
            contour_enumeration(vec![original_birth(901)]),
        )
        .unwrap();
        let right = prepare_context_neutral_productive_material_with_contours(
            "source",
            package_tuple(),
            enumeration(replacements()),
            contour_enumeration(vec![original_birth(902)]),
        )
        .unwrap();

        assert_eq!(left.completeness().state(), EnumerationStateV1::Complete);
        assert_eq!(left.compact.targets.len(), MAX_TARGETS_PER_FIELD);
        assert!(!left
            .exact_target_surfaces()
            .any(|surface| surface == "source"));
        assert_eq!(
            left.compact.original.lexical_status,
            PreparedOriginalLexicalStatusV1::Clean
        );
        assert!(left.original_has_grounded_l11_evidence());
        assert_eq!(left.compact.targets, right.compact.targets);
        assert_ne!(left.exact_digest(), right.exact_digest());
    }

    #[test]
    fn contour_reserve_prefers_direct_layout_then_round_robins_relations() {
        let mut births = vec![contour_birth(
            "zz-direct-layout",
            TargetRelationV1::ExactLayout,
            1,
        )];
        let relations = [
            TargetRelationV1::MissingLetter,
            TargetRelationV1::ExtraLetter,
            TargetRelationV1::Substitution,
            TargetRelationV1::AdjacentTransposition,
        ];
        for (relation_index, relation) in relations.into_iter().enumerate() {
            for member in 0..4 {
                births.push(contour_birth(
                    &format!("r{relation_index}-member-{member}"),
                    relation,
                    10 + (relation_index * 4 + member) as u32,
                ));
            }
        }

        let material = prepare_context_neutral_productive_material_with_contours(
            "source",
            package_tuple(),
            enumeration(Vec::new()),
            contour_enumeration(births),
        )
        .unwrap();
        let retained = material.exact_target_surfaces().collect::<BTreeSet<_>>();

        assert_eq!(retained.len(), MAX_CONTOUR_TARGETS_PER_FIELD);
        assert!(retained.contains("zz-direct-layout"));
        assert!(retained.contains("r1-member-1"));
        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Overflow
        );
        assert_eq!(
            material.completeness().reason(),
            IncompletenessReasonV1::StorageCapacity
        );
        assert!(material.compact.targets.as_slice().iter().all(|target| {
            target
                .witnesses
                .witnesses()
                .iter()
                .all(|witness| witness.verdict_membership == VerdictMembershipV1::Born)
        }));
    }

    #[test]
    fn contour_root_merges_into_productive_surface_without_consuming_reserve() {
        let mut births = vec![contour_birth("shared", TargetRelationV1::ExtraLetter, 1)];
        births.extend((0..MAX_CONTOUR_TARGETS_PER_FIELD).map(|index| {
            contour_birth(
                &format!("novel-{index}"),
                TargetRelationV1::MissingLetter,
                index as u32 + 2,
            )
        }));
        let material = prepare_context_neutral_productive_material_with_contours(
            "source",
            package_tuple(),
            enumeration(vec![candidate("shared", 1)]),
            contour_enumeration(births),
        )
        .unwrap();

        assert_eq!(
            material.exact_target_surfaces().count(),
            MAX_CONTOUR_TARGETS_PER_FIELD + 1
        );
        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Complete
        );
        let shared = material
            .exact_targets
            .iter()
            .find(|target| target.normalized_scalars == "shared")
            .unwrap();
        assert!(shared
            .witness_roots
            .iter()
            .any(|root| root.verdict_membership == VerdictMembershipV1::Grounded));
        assert!(shared
            .witness_roots
            .iter()
            .any(|root| root.verdict_membership == VerdictMembershipV1::Born));
    }

    #[test]
    fn boundary_reserve_is_two_surfaces_and_overflow_is_whole_field() {
        let births = (0..3)
            .map(|index| {
                boundary_birth(
                    &format!("left{index} right{index}"),
                    TargetRelationV1::BoundarySplit,
                    index + 1,
                )
            })
            .collect();
        let material = prepare_context_neutral_productive_material_with_contours_and_boundaries(
            "source",
            package_tuple(),
            enumeration(Vec::new()),
            TypedContourBirthEnumerationV1::complete_empty(),
            boundary_enumeration(births),
        )
        .unwrap();

        assert_eq!(
            material.exact_target_surfaces().count(),
            MAX_BOUNDARY_TARGETS_PER_FIELD
        );
        assert_eq!(
            material.boundary_groundings().len(),
            MAX_BOUNDARY_TARGETS_PER_FIELD
        );
        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Overflow
        );
        assert_eq!(
            material.completeness().reason(),
            IncompletenessReasonV1::StorageCapacity
        );
        assert!(material.compact.targets.as_slice().iter().all(|target| {
            target.identity.separator_profile_id
                == SeparatorProfileIdV1(ASCII_SPACE_SEPARATOR_PROFILE)
                && target.witnesses.witnesses().iter().all(|witness| {
                    witness.grounding_namespace == GroundingNamespaceV1::CompositeBoundary
                        && witness.verdict_membership == VerdictMembershipV1::Born
                })
        }));
    }

    #[test]
    fn productive_boundary_dedup_does_not_consume_a_boundary_slot() {
        let births = vec![
            boundary_birth("merged", TargetRelationV1::BoundaryMerge, 1),
            boundary_birth("left one", TargetRelationV1::BoundarySplit, 2),
            boundary_birth("right two", TargetRelationV1::BoundarySplit, 3),
        ];
        let material = prepare_context_neutral_productive_material_with_contours_and_boundaries(
            "source",
            package_tuple(),
            enumeration(vec![candidate("merged", 1)]),
            TypedContourBirthEnumerationV1::complete_empty(),
            boundary_enumeration(births),
        )
        .unwrap();

        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Complete
        );
        assert_eq!(material.exact_target_surfaces().count(), 3);
        let merged = material
            .exact_targets
            .iter()
            .find(|target| target.normalized_scalars == "merged")
            .unwrap();
        assert!(merged
            .witness_roots
            .iter()
            .any(|root| { root.grounding_namespace == GroundingNamespaceV1::ProductiveSurface }));
        assert!(merged.witness_roots.iter().any(|root| {
            root.grounding_namespace == GroundingNamespaceV1::CompositeBoundary
                && root.verdict_membership == VerdictMembershipV1::Born
        }));
    }

    #[test]
    fn work_overflow_cannot_be_complete() {
        let mut enumeration = enumeration(vec![candidate("форма", 1)]);
        enumeration.work_budget_exceeded = true;
        let material =
            prepare_context_neutral_productive_material("слово", package_tuple(), enumeration)
                .unwrap();
        assert_eq!(
            material.completeness().state(),
            EnumerationStateV1::Overflow
        );
        assert_eq!(
            material.completeness().reason(),
            IncompletenessReasonV1::WorkBudgetExceeded
        );
    }

    #[test]
    fn exact_frame_projection_replays_scalar_span_and_preserves_punctuation() {
        let material = material();
        let expected = frame(&material);
        let mut arena = PreparedMaterialLeaseArenaV1::default();
        let lease = arena
            .pin(
                &material,
                9,
                17,
                [19, 23],
                1_000,
                LeaseConsumerStateV1::FrameSettlement,
            )
            .expect("lease");
        let bound =
            bind_exact_frame_target(&material, lease, &expected, &expected, 0, 0, 5, 1, 0, 500)
                .expect("bound target");
        assert_eq!(bound.projected_target, "Форма");
        assert_eq!(bound.replayed_source_window, "Форма,");
    }

    #[test]
    fn every_frame_identity_dimension_rejects_stale_reuse() {
        let material = material();
        let expected = frame(&material);
        let variants = [
            (FrameInvalidationReasonV1::Focus, 0),
            (FrameInvalidationReasonV1::TailEpoch, 1),
            (FrameInvalidationReasonV1::SourceWindow, 2),
            (FrameInvalidationReasonV1::LeftContext, 3),
            (FrameInvalidationReasonV1::Caret, 4),
            (FrameInvalidationReasonV1::Selection, 5),
            (FrameInvalidationReasonV1::Preedit, 6),
            (FrameInvalidationReasonV1::LayoutGeneration, 7),
            (FrameInvalidationReasonV1::ConfigGeneration, 8),
            (FrameInvalidationReasonV1::PackageGeneration, 9),
            (FrameInvalidationReasonV1::FieldGeneration, 10),
        ];
        for (reason, index) in variants {
            let mut current = expected.clone();
            match index {
                0 => current.identity.focus_serial += 1,
                1 => current.identity.tail_epoch += 1,
                2 => current.source_window.push('!'),
                3 => current.left_context.push('!'),
                4 => current.identity.caret_scalar -= 1,
                5 => current.identity.selection_start_scalar -= 1,
                6 => current.preedit.push('!'),
                7 => current.identity.layout_generation += 1,
                8 => current.identity.config_generation += 1,
                9 => current.identity.package_generation += 1,
                10 => current.identity.field_generation += 1,
                _ => unreachable!(),
            }
            assert_eq!(expected.compare_exact(&current), Err(reason));
        }
    }

    #[test]
    fn lease_arena_is_bounded_by_fields_and_consumers() {
        let material = material();
        let mut arena = PreparedMaterialLeaseArenaV1::default();
        for _ in 0..MAX_LEASE_CONSUMERS_PER_FIELD {
            assert!(arena
                .pin(
                    &material,
                    9,
                    17,
                    [19, 23],
                    1_000,
                    LeaseConsumerStateV1::FrameSettlement,
                )
                .is_some());
        }
        assert!(arena
            .pin(
                &material,
                9,
                17,
                [19, 23],
                1_000,
                LeaseConsumerStateV1::FrameSettlement,
            )
            .is_none());
    }

    #[test]
    fn td117_lease_arena_rejects_same_id_with_modified_expiry_owner_or_consumer() {
        let material = material();
        let mut arena = PreparedMaterialLeaseArenaV1::default();
        let lease = arena
            .pin(
                &material,
                9,
                17,
                [19, 23],
                1_000,
                LeaseConsumerStateV1::FrameSettlement,
            )
            .expect("issued lease");
        assert!(arena.contains(lease));

        let mut changed_expiry = lease;
        changed_expiry.expires_at_monotonic_ns = 1_001;
        let mut changed_owner = lease;
        changed_owner.runtime_owner_lease_identity = 18;
        let mut changed_consumer = lease;
        changed_consumer.consumer = LeaseConsumerStateV1::EventPlan;

        for forged in [changed_expiry, changed_owner, changed_consumer] {
            assert_eq!(forged.lease_identity, lease.lease_identity);
            assert!(!arena.contains(forged));
        }
    }
}
