//! Shared bounded target-evidence vocabulary.
//!
//! Slice 1 introduces data types and compatibility projections only. None of
//! these values grants display or mutation authority until later migration
//! slices bind exact prepared material, an input frame, and a conflict cohort.

use std::cmp::Ordering;
use std::num::NonZeroU32;

pub(crate) const MAX_TARGETS_PER_FIELD: usize = 74;
pub(crate) const MAX_TARGET_WITNESSES_PER_TARGET: usize = 4;
pub(crate) const MAX_PINNED_PREPARED_FIELDS: usize = 32;
pub(crate) const MAX_LEASE_CONSUMERS_PER_FIELD: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct EnumerationWorkCountersV1 {
    pub(crate) posting_visits: u64,
    pub(crate) relation_replays: u64,
    pub(crate) grounding_lookups: u64,
    pub(crate) generated_logical_targets: u64,
    pub(crate) operator_steps: u64,
}

impl EnumerationWorkCountersV1 {
    pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            posting_visits: self.posting_visits.checked_add(other.posting_visits)?,
            relation_replays: self.relation_replays.checked_add(other.relation_replays)?,
            grounding_lookups: self
                .grounding_lookups
                .checked_add(other.grounding_lookups)?,
            generated_logical_targets: self
                .generated_logical_targets
                .checked_add(other.generated_logical_targets)?,
            operator_steps: self.operator_steps.checked_add(other.operator_steps)?,
        })
    }

    pub(crate) const fn within(self, ceiling: Self) -> bool {
        self.posting_visits <= ceiling.posting_visits
            && self.relation_replays <= ceiling.relation_replays
            && self.grounding_lookups <= ceiling.grounding_lookups
            && self.generated_logical_targets <= ceiling.generated_logical_targets
            && self.operator_steps <= ceiling.operator_steps
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct EnumerationWorkBudgetV1 {
    pub(crate) canonical_grounding: EnumerationWorkCountersV1,
    pub(crate) cold_binding: EnumerationWorkCountersV1,
    pub(crate) productive_traversal: EnumerationWorkCountersV1,
    pub(crate) aggregate: EnumerationWorkCountersV1,
}

const LEGACY_L2_OPERATOR_BASE: u32 = 0x4c32_0000;
const LEGACY_LIVE_OPERATOR_BASE: u32 = 0x4c56_0000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub(crate) enum TargetRelationV1 {
    #[default]
    Unsupported = 0,
    ExactLayout = 1,
    MissingLetter = 2,
    ExtraLetter = 3,
    Substitution = 4,
    AdjacentTransposition = 5,
    NonAdjacentTransposition = 6,
    SparseOmission = 7,
    RepeatedFragment = 8,
    MixedLayout = 9,
    LayoutThenTypo = 10,
    MorphologySlot = 11,
    BoundarySplit = 12,
    BoundaryMerge = 13,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub(crate) enum GroundingNamespaceV1 {
    #[default]
    None = 0,
    L11Terminal = 1,
    CanonicalForm = 2,
    ProductiveSurface = 3,
    ComposedSurface = 4,
    CompositeBoundary = 5,
    LegacyL2Ime = 6,
    LegacyLiveReplacement = 7,
    LegacyCorrectionCandidate = 8,
    ReferenceSurface = 9,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub(crate) enum VerdictMembershipV1 {
    #[default]
    None = 0,
    Born = 1,
    Grounded = 2,
    L11Winner = 3,
    L11Tied = 4,
}

/// Compact field-local witness reference. Exact equality is defined by the
/// referenced relation/operator/grounding/derivation tables. The accelerator
/// is never consulted by `same_semantic_root`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct TargetWitnessV1 {
    pub(crate) relation: TargetRelationV1,
    pub(crate) grounding_namespace: GroundingNamespaceV1,
    pub(crate) verdict_membership: VerdictMembershipV1,
    pub(crate) flags: u8,
    pub(crate) operator_ref: u32,
    pub(crate) grounding_ref: u32,
    pub(crate) derivation_ref: u32,
    pub(crate) support_milli: u16,
    pub(crate) provenance_annotations: u16,
    pub(crate) semantic_root_accelerator: u32,
}

impl TargetWitnessV1 {
    pub(crate) fn new(
        relation: TargetRelationV1,
        grounding_namespace: GroundingNamespaceV1,
        verdict_membership: VerdictMembershipV1,
        flags: u8,
        operator_ref: u32,
        grounding_ref: u32,
        derivation_ref: u32,
        support_milli: u16,
        provenance_annotations: u16,
    ) -> Self {
        let mut witness = Self {
            relation,
            grounding_namespace,
            verdict_membership,
            flags,
            operator_ref,
            grounding_ref,
            derivation_ref,
            support_milli,
            provenance_annotations,
            semantic_root_accelerator: 0,
        };
        witness.semantic_root_accelerator = witness.semantic_root_digest() as u32;
        witness
    }

    pub(crate) fn same_semantic_root(&self, other: &Self) -> bool {
        self.semantic_cmp(other) == Ordering::Equal
    }

    fn semantic_cmp(&self, other: &Self) -> Ordering {
        (
            self.relation,
            self.grounding_namespace,
            self.verdict_membership,
            self.flags,
            self.operator_ref,
            self.grounding_ref,
            self.derivation_ref,
        )
            .cmp(&(
                other.relation,
                other.grounding_namespace,
                other.verdict_membership,
                other.flags,
                other.operator_ref,
                other.grounding_ref,
                other.derivation_ref,
            ))
    }

    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.semantic_cmp(other)
            .then_with(|| other.support_milli.cmp(&self.support_milli))
            .then_with(|| {
                self.provenance_annotations
                    .cmp(&other.provenance_annotations)
            })
    }

    fn merge_alias(&mut self, alias: Self) {
        debug_assert!(self.same_semantic_root(&alias));
        self.support_milli = self.support_milli.max(alias.support_milli);
        self.provenance_annotations |= alias.provenance_annotations;
    }

    fn semantic_root_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        for byte in [
            self.relation as u8,
            self.grounding_namespace as u8,
            self.verdict_membership as u8,
            self.flags,
        ] {
            digest = fnv1a_byte(digest, byte);
        }
        for value in [self.operator_ref, self.grounding_ref, self.derivation_ref] {
            for byte in value.to_le_bytes() {
                digest = fnv1a_byte(digest, byte);
            }
        }
        digest
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum EnumerationStateV1 {
    #[default]
    Complete = 0,
    Overflow = 1,
    Failed = 2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum CompletenessScopeKindV1 {
    #[default]
    WholePreparedField = 0,
    EditFootprintPartition = 1,
    RelationPartition = 2,
}

/// Authority scope for one completeness claim. Narrow scopes cannot be
/// represented without a non-zero reference to the exhaustive partition proof
/// that established them before truncation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompletenessScopeV1 {
    WholePreparedField,
    EditFootprintPartition {
        exhaustive_partition_proof_ref: NonZeroU32,
    },
    RelationPartition {
        exhaustive_partition_proof_ref: NonZeroU32,
    },
}

impl Default for CompletenessScopeV1 {
    fn default() -> Self {
        Self::WholePreparedField
    }
}

impl CompletenessScopeV1 {
    pub(crate) fn edit_footprint_partition(exhaustive_partition_proof_ref: u32) -> Option<Self> {
        Some(Self::EditFootprintPartition {
            exhaustive_partition_proof_ref: NonZeroU32::new(exhaustive_partition_proof_ref)?,
        })
    }

    pub(crate) fn relation_partition(exhaustive_partition_proof_ref: u32) -> Option<Self> {
        Some(Self::RelationPartition {
            exhaustive_partition_proof_ref: NonZeroU32::new(exhaustive_partition_proof_ref)?,
        })
    }

    pub(crate) const fn kind(self) -> CompletenessScopeKindV1 {
        match self {
            Self::WholePreparedField => CompletenessScopeKindV1::WholePreparedField,
            Self::EditFootprintPartition { .. } => CompletenessScopeKindV1::EditFootprintPartition,
            Self::RelationPartition { .. } => CompletenessScopeKindV1::RelationPartition,
        }
    }

    pub(crate) const fn exhaustive_partition_proof_ref(self) -> Option<NonZeroU32> {
        match self {
            Self::WholePreparedField => None,
            Self::EditFootprintPartition {
                exhaustive_partition_proof_ref,
            }
            | Self::RelationPartition {
                exhaustive_partition_proof_ref,
            } => Some(exhaustive_partition_proof_ref),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum IncompletenessReasonV1 {
    #[default]
    None = 0,
    StorageCapacity = 1,
    WorkBudgetExceeded = 2,
    UpstreamIncomplete = 3,
    IntegrityFailure = 4,
}

/// Fixed-size witness set. Package generation is deliberately absent: it
/// belongs to material validity and cannot manufacture witness independence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct TargetEvidenceSetV1 {
    witnesses: [TargetWitnessV1; MAX_TARGET_WITNESSES_PER_TARGET],
    witness_count: u8,
    state: EnumerationStateV1,
    reason: IncompletenessReasonV1,
    scope: CompletenessScopeV1,
    logical_count: u16,
    retained_count: u16,
    all_seen_digest: [u64; 2],
}

impl Default for TargetEvidenceSetV1 {
    fn default() -> Self {
        Self::complete_empty()
    }
}

impl TargetEvidenceSetV1 {
    pub(crate) const fn complete_empty() -> Self {
        Self {
            witnesses: [TargetWitnessV1 {
                relation: TargetRelationV1::Unsupported,
                grounding_namespace: GroundingNamespaceV1::None,
                verdict_membership: VerdictMembershipV1::None,
                flags: 0,
                operator_ref: 0,
                grounding_ref: 0,
                derivation_ref: 0,
                support_milli: 0,
                provenance_annotations: 0,
                semantic_root_accelerator: 0,
            }; MAX_TARGET_WITNESSES_PER_TARGET],
            witness_count: 0,
            state: EnumerationStateV1::Complete,
            reason: IncompletenessReasonV1::None,
            scope: CompletenessScopeV1::WholePreparedField,
            logical_count: 0,
            retained_count: 0,
            all_seen_digest: [0; 2],
        }
    }

    pub(crate) fn from_one(witness: TargetWitnessV1) -> Self {
        Self::from_bounded([witness])
    }

    pub(crate) fn from_bounded<const N: usize>(mut witnesses: [TargetWitnessV1; N]) -> Self {
        witnesses.sort_by(TargetWitnessV1::canonical_cmp);
        Self::from_sorted_slice(&witnesses)
    }

    pub(crate) fn failed(reason: IncompletenessReasonV1) -> Self {
        Self {
            state: EnumerationStateV1::Failed,
            reason,
            ..Self::complete_empty()
        }
    }

    pub(crate) fn merge(self, other: Self) -> Self {
        let mut known = [TargetWitnessV1::default(); MAX_TARGET_WITNESSES_PER_TARGET * 2];
        let mut known_len = 0;
        for witness in self.witnesses().iter().chain(other.witnesses()) {
            known[known_len] = *witness;
            known_len += 1;
        }
        let mut merged = Self::from_slice(&known[..known_len]);
        merged.all_seen_digest = union_set_digest(self.all_seen_digest, other.all_seen_digest);
        merged.scope = self.scope;

        if self.scope != other.scope {
            merged.state = EnumerationStateV1::Failed;
            merged.reason = IncompletenessReasonV1::IntegrityFailure;
            merged.scope = CompletenessScopeV1::WholePreparedField;
        } else if self.state == EnumerationStateV1::Failed
            || other.state == EnumerationStateV1::Failed
        {
            merged.state = EnumerationStateV1::Failed;
            merged.reason = merged_incomplete_reason(
                self,
                other,
                EnumerationStateV1::Failed,
                IncompletenessReasonV1::IntegrityFailure,
            );
        } else if self.state == EnumerationStateV1::Overflow
            || other.state == EnumerationStateV1::Overflow
        {
            merged.state = EnumerationStateV1::Overflow;
            merged.reason = merged_incomplete_reason(
                self,
                other,
                EnumerationStateV1::Overflow,
                IncompletenessReasonV1::UpstreamIncomplete,
            );
            // Once a producer overflowed, omitted roots cannot be compared for
            // equality. Keep only a proof-safe lower bound; overflow itself
            // already blocks singleton authority.
            merged.logical_count = merged
                .logical_count
                .max(self.logical_count)
                .max(other.logical_count);
        }
        merged
    }

    pub(crate) fn witnesses(&self) -> &[TargetWitnessV1] {
        &self.witnesses[..usize::from(self.witness_count)]
    }

    pub(crate) const fn state(&self) -> EnumerationStateV1 {
        self.state
    }

    pub(crate) const fn reason(&self) -> IncompletenessReasonV1 {
        self.reason
    }

    pub(crate) const fn scope(&self) -> CompletenessScopeV1 {
        self.scope
    }

    pub(crate) fn with_exhaustive_scope(mut self, scope: CompletenessScopeV1) -> Self {
        self.scope = scope;
        self
    }

    pub(crate) const fn logical_count(&self) -> u16 {
        self.logical_count
    }

    pub(crate) const fn retained_count(&self) -> u16 {
        self.retained_count
    }

    pub(crate) const fn all_seen_digest(&self) -> [u64; 2] {
        self.all_seen_digest
    }

    pub(crate) fn try_single_complete_witness(
        &self,
    ) -> Result<TargetWitnessV1, LegacyProjectionErrorV1> {
        if self.state != EnumerationStateV1::Complete {
            return Err(LegacyProjectionErrorV1::Incomplete);
        }
        if self.witness_count != 1 {
            return Err(LegacyProjectionErrorV1::NotSingleton);
        }
        Ok(self.witnesses[0])
    }

    fn from_slice(witnesses: &[TargetWitnessV1]) -> Self {
        let mut sorted = [TargetWitnessV1::default(); MAX_TARGET_WITNESSES_PER_TARGET * 2];
        assert!(
            witnesses.len() <= sorted.len(),
            "bounded target evidence adapter exceeded eight input witnesses"
        );
        sorted[..witnesses.len()].copy_from_slice(witnesses);
        sorted[..witnesses.len()].sort_by(TargetWitnessV1::canonical_cmp);

        Self::from_sorted_slice(&sorted[..witnesses.len()])
    }

    fn from_sorted_slice(witnesses: &[TargetWitnessV1]) -> Self {
        let mut result = Self::complete_empty();
        let mut index = 0;
        let mut logical_count = 0_u16;
        let mut all_seen_digest = [0_u64; 2];
        while index < witnesses.len() {
            let mut merged = witnesses[index];
            index += 1;
            while index < witnesses.len() && merged.same_semantic_root(&witnesses[index]) {
                merged.merge_alias(witnesses[index]);
                index += 1;
            }
            merged.semantic_root_accelerator = merged.semantic_root_digest() as u32;
            logical_count = logical_count.saturating_add(1);
            all_seen_digest = union_set_digest(
                all_seen_digest,
                semantic_root_set_digest(merged.semantic_root_digest()),
            );
            if usize::from(result.witness_count) < MAX_TARGET_WITNESSES_PER_TARGET {
                result.witnesses[usize::from(result.witness_count)] = merged;
                result.witness_count += 1;
            }
        }
        result.logical_count = logical_count;
        result.retained_count = u16::from(result.witness_count);
        result.all_seen_digest = all_seen_digest;
        if usize::from(logical_count) > MAX_TARGET_WITNESSES_PER_TARGET {
            result.state = EnumerationStateV1::Overflow;
            result.reason = IncompletenessReasonV1::StorageCapacity;
        }
        result
    }
}

fn merged_incomplete_reason(
    left: TargetEvidenceSetV1,
    right: TargetEvidenceSetV1,
    state: EnumerationStateV1,
    conflicting_reason: IncompletenessReasonV1,
) -> IncompletenessReasonV1 {
    match (left.state == state, right.state == state) {
        (true, false) => left.reason,
        (false, true) => right.reason,
        (true, true) if left.reason == right.reason => left.reason,
        (true, true) => conflicting_reason,
        (false, false) => IncompletenessReasonV1::None,
    }
}

/// A compact idempotent set sketch. Overflow blocks authority, so this value is
/// used for deterministic integrity/parity checks only; exact equality remains
/// owned by retained witness roots and their immutable tables.
fn semantic_root_set_digest(root_digest: u64) -> [u64; 2] {
    let mixed = mix_u64(root_digest ^ 0x9e37_79b9_7f4a_7c15);
    let left = (1_u64 << (root_digest & 63))
        | (1_u64 << ((root_digest >> 17) & 63))
        | (1_u64 << ((root_digest >> 41) & 63));
    let right =
        (1_u64 << (mixed & 63)) | (1_u64 << ((mixed >> 19) & 63)) | (1_u64 << ((mixed >> 43) & 63));
    [left, right]
}

fn union_set_digest(left: [u64; 2], right: [u64; 2]) -> [u64; 2] {
    [left[0] | right[0], left[1] | right[1]]
}

fn mix_u64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LegacyProjectionErrorV1 {
    Incomplete,
    NotSingleton,
    WrongNamespace,
    UnsupportedVariant,
}

/// Compatibility vocabulary owned here until all legacy consumers migrate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum L2ImeTargetEvidence {
    #[default]
    None,
    LexicalReconstruction,
    ContextBoundEdit,
    CanonicalWinner,
    LayoutRepair,
    ExactLayout,
    Boundary,
}

impl L2ImeTargetEvidence {
    pub(crate) fn to_common(self) -> TargetEvidenceSetV1 {
        let Some((relation, variant)) = self.common_parts() else {
            return TargetEvidenceSetV1::complete_empty();
        };
        TargetEvidenceSetV1::from_one(TargetWitnessV1::new(
            relation,
            GroundingNamespaceV1::LegacyL2Ime,
            VerdictMembershipV1::Grounded,
            u8::from(matches!(self, Self::Boundary)),
            LEGACY_L2_OPERATOR_BASE | variant,
            variant,
            variant,
            1_000,
            1,
        ))
    }

    pub(crate) fn try_from_common(
        evidence: &TargetEvidenceSetV1,
    ) -> Result<Self, LegacyProjectionErrorV1> {
        if evidence.state() == EnumerationStateV1::Complete && evidence.witnesses().is_empty() {
            return Ok(Self::None);
        }
        let witness = evidence.try_single_complete_witness()?;
        if witness.grounding_namespace != GroundingNamespaceV1::LegacyL2Ime {
            return Err(LegacyProjectionErrorV1::WrongNamespace);
        }
        match witness.operator_ref.checked_sub(LEGACY_L2_OPERATOR_BASE) {
            Some(1) => Ok(Self::LexicalReconstruction),
            Some(2) => Ok(Self::ContextBoundEdit),
            Some(3) => Ok(Self::CanonicalWinner),
            Some(4) => Ok(Self::LayoutRepair),
            Some(5) => Ok(Self::ExactLayout),
            Some(6) => Ok(Self::Boundary),
            _ => Err(LegacyProjectionErrorV1::UnsupportedVariant),
        }
    }

    fn common_parts(self) -> Option<(TargetRelationV1, u32)> {
        match self {
            Self::None => None,
            Self::LexicalReconstruction => Some((TargetRelationV1::Unsupported, 1)),
            Self::ContextBoundEdit => Some((TargetRelationV1::Unsupported, 2)),
            Self::CanonicalWinner => Some((TargetRelationV1::Unsupported, 3)),
            Self::LayoutRepair => Some((TargetRelationV1::LayoutThenTypo, 4)),
            Self::ExactLayout => Some((TargetRelationV1::ExactLayout, 5)),
            Self::Boundary => Some((TargetRelationV1::BoundarySplit, 6)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ReplacementTargetEvidence {
    #[default]
    None,
    ExactLayoutProjection,
    VerifiedLexicalEdit,
    ContextBoundLexicalEdit,
    VerifiedBoundary,
}

impl ReplacementTargetEvidence {
    pub(crate) const fn authorizes(self) -> bool {
        !matches!(self, Self::None)
    }

    pub(crate) fn to_common(self) -> TargetEvidenceSetV1 {
        let Some((relation, variant)) = self.common_parts() else {
            return TargetEvidenceSetV1::complete_empty();
        };
        TargetEvidenceSetV1::from_one(TargetWitnessV1::new(
            relation,
            GroundingNamespaceV1::LegacyLiveReplacement,
            VerdictMembershipV1::Grounded,
            u8::from(matches!(self, Self::VerifiedBoundary)),
            LEGACY_LIVE_OPERATOR_BASE | variant,
            variant,
            variant,
            1_000,
            1,
        ))
    }

    pub(crate) fn try_from_common(
        evidence: &TargetEvidenceSetV1,
    ) -> Result<Self, LegacyProjectionErrorV1> {
        if evidence.state() == EnumerationStateV1::Complete && evidence.witnesses().is_empty() {
            return Ok(Self::None);
        }
        let witness = evidence.try_single_complete_witness()?;
        if witness.grounding_namespace != GroundingNamespaceV1::LegacyLiveReplacement {
            return Err(LegacyProjectionErrorV1::WrongNamespace);
        }
        match witness.operator_ref.checked_sub(LEGACY_LIVE_OPERATOR_BASE) {
            Some(1) => Ok(Self::ExactLayoutProjection),
            Some(2) => Ok(Self::VerifiedLexicalEdit),
            Some(3) => Ok(Self::ContextBoundLexicalEdit),
            Some(4) => Ok(Self::VerifiedBoundary),
            _ => Err(LegacyProjectionErrorV1::UnsupportedVariant),
        }
    }

    fn common_parts(self) -> Option<(TargetRelationV1, u32)> {
        match self {
            Self::None => None,
            Self::ExactLayoutProjection => Some((TargetRelationV1::ExactLayout, 1)),
            Self::VerifiedLexicalEdit => Some((TargetRelationV1::Unsupported, 2)),
            Self::ContextBoundLexicalEdit => Some((TargetRelationV1::Unsupported, 3)),
            Self::VerifiedBoundary => Some((TargetRelationV1::BoundarySplit, 4)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct NormalizationLayoutProfileIdV1(pub(crate) u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct SeparatorProfileIdV1(pub(crate) u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct MaterialTargetIdentityV1 {
    pub(crate) normalized_scalars_ref: u32,
    pub(crate) canonical_bytes_ref: u32,
    pub(crate) normalization_layout_profile_id: NormalizationLayoutProfileIdV1,
    pub(crate) separator_profile_id: SeparatorProfileIdV1,
    pub(crate) exact_scalar_count: u16,
    pub(crate) flags: u16,
    pub(crate) accelerator: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct PreparedMaterialKeyV1 {
    pub(crate) observed_contour_ref: u32,
    pub(crate) normalization_layout_profile_id: NormalizationLayoutProfileIdV1,
    pub(crate) package_generation: u64,
    pub(crate) exact_package_digest_prefix: [u8; 16],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct PreparedOriginalMaterialV1 {
    pub(crate) exact_observed_scalars_ref: u32,
    pub(crate) exact_observed_bytes_ref: u32,
    pub(crate) preservation_schema: u32,
    pub(crate) lexical_status: PreparedOriginalLexicalStatusV1,
    pub(crate) script_token_status: PreparedOriginalScriptTokenStatusV1,
    pub(crate) punctuation_status: PreparedOriginalPunctuationStatusV1,
    pub(crate) reserved: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum PreparedOriginalLexicalStatusV1 {
    #[default]
    Unknown = 0,
    Damaged = 1,
    Clean = 2,
    Protected = 3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum PreparedOriginalScriptTokenStatusV1 {
    #[default]
    Unknown = 0,
    Supported = 1,
    Unsupported = 2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum PreparedOriginalPunctuationStatusV1 {
    #[default]
    Unknown = 0,
    Stable = 1,
    Protected = 2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct EnumerationCompletenessV1 {
    state: EnumerationStateV1,
    reason: IncompletenessReasonV1,
    scope: CompletenessScopeV1,
    retained_count: u16,
    logical_count_lower_bound: u16,
    all_seen_digest: [u64; 2],
}

impl EnumerationCompletenessV1 {
    pub(crate) fn complete(logical_count: usize, digest: [u64; 2]) -> Self {
        Self::complete_in_scope(
            logical_count,
            digest,
            CompletenessScopeV1::WholePreparedField,
        )
    }

    pub(crate) fn complete_in_scope(
        logical_count: usize,
        digest: [u64; 2],
        scope: CompletenessScopeV1,
    ) -> Self {
        Self {
            state: EnumerationStateV1::Complete,
            scope,
            retained_count: saturating_u16(logical_count),
            logical_count_lower_bound: saturating_u16(logical_count),
            all_seen_digest: digest,
            ..Self::default()
        }
    }

    pub(crate) fn overflow(
        retained_count: usize,
        logical_count_lower_bound: usize,
        reason: IncompletenessReasonV1,
        digest: [u64; 2],
    ) -> Self {
        Self::overflow_in_scope(
            retained_count,
            logical_count_lower_bound,
            reason,
            digest,
            CompletenessScopeV1::WholePreparedField,
        )
    }

    pub(crate) fn overflow_in_scope(
        retained_count: usize,
        logical_count_lower_bound: usize,
        reason: IncompletenessReasonV1,
        digest: [u64; 2],
        scope: CompletenessScopeV1,
    ) -> Self {
        Self {
            state: EnumerationStateV1::Overflow,
            reason,
            scope,
            retained_count: saturating_u16(retained_count),
            logical_count_lower_bound: saturating_u16(logical_count_lower_bound),
            all_seen_digest: digest,
            ..Self::default()
        }
    }

    pub(crate) fn failed(reason: IncompletenessReasonV1) -> Self {
        Self {
            state: EnumerationStateV1::Failed,
            reason,
            ..Self::default()
        }
    }

    pub(crate) const fn state(self) -> EnumerationStateV1 {
        self.state
    }

    pub(crate) const fn reason(self) -> IncompletenessReasonV1 {
        self.reason
    }

    pub(crate) const fn scope(self) -> CompletenessScopeV1 {
        self.scope
    }

    pub(crate) const fn retained_count(self) -> u16 {
        self.retained_count
    }

    pub(crate) const fn logical_count_lower_bound(self) -> u16 {
        self.logical_count_lower_bound
    }

    pub(crate) const fn all_seen_digest(self) -> [u64; 2] {
        self.all_seen_digest
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct PreparedTargetV1 {
    pub(crate) identity: MaterialTargetIdentityV1,
    pub(crate) witnesses: TargetEvidenceSetV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BoundedTargetSetV1 {
    targets: [PreparedTargetV1; MAX_TARGETS_PER_FIELD],
    len: u16,
}

impl Default for BoundedTargetSetV1 {
    fn default() -> Self {
        Self {
            targets: [PreparedTargetV1::default(); MAX_TARGETS_PER_FIELD],
            len: 0,
        }
    }
}

impl BoundedTargetSetV1 {
    pub(crate) fn push(&mut self, target: PreparedTargetV1) -> Result<(), TargetSetOverflowV1> {
        let index = usize::from(self.len);
        if index == MAX_TARGETS_PER_FIELD {
            return Err(TargetSetOverflowV1);
        }
        self.targets[index] = target;
        self.len += 1;
        Ok(())
    }

    pub(crate) fn as_slice(&self) -> &[PreparedTargetV1] {
        &self.targets[..usize::from(self.len)]
    }

    pub(crate) const fn len(&self) -> usize {
        self.len as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TargetSetOverflowV1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct PreparedEvidenceTablesV1 {
    pub(crate) relation_table_identity: u64,
    pub(crate) grounding_table_identity: u64,
    pub(crate) derivation_table_identity: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct PreparedIntegrityV1 {
    pub(crate) exact_digest: [u64; 2],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PreparedTargetMaterialV1 {
    pub(crate) key: PreparedMaterialKeyV1,
    pub(crate) original: PreparedOriginalMaterialV1,
    pub(crate) targets: BoundedTargetSetV1,
    pub(crate) completeness: EnumerationCompletenessV1,
    pub(crate) evidence_tables: PreparedEvidenceTablesV1,
    pub(crate) integrity: PreparedIntegrityV1,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum LeaseConsumerStateV1 {
    #[default]
    Display = 0,
    FrameSettlement = 1,
    EventPlan = 2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct PreparedMaterialLeaseV1 {
    pub(crate) material_key: PreparedMaterialKeyV1,
    pub(crate) integrity_digest: [u64; 2],
    pub(crate) field_generation: u64,
    pub(crate) allocation_identity: u64,
    pub(crate) runtime_owner_lease_identity: u64,
    pub(crate) monotonic_epoch_identity: [u64; 2],
    pub(crate) expires_at_monotonic_ns: u64,
    pub(crate) lease_identity: u64,
    pub(crate) consumer: LeaseConsumerStateV1,
    reserved: [u8; 7],
}

impl PreparedMaterialLeaseV1 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        material_key: PreparedMaterialKeyV1,
        integrity_digest: [u64; 2],
        field_generation: u64,
        allocation_identity: u64,
        runtime_owner_lease_identity: u64,
        monotonic_epoch_identity: [u64; 2],
        expires_at_monotonic_ns: u64,
        lease_identity: u64,
        consumer: LeaseConsumerStateV1,
    ) -> Option<Self> {
        (field_generation != 0
            && allocation_identity != 0
            && runtime_owner_lease_identity != 0
            && monotonic_epoch_identity != [0; 2]
            && expires_at_monotonic_ns != 0
            && lease_identity != 0)
            .then_some(Self {
                material_key,
                integrity_digest,
                field_generation,
                allocation_identity,
                runtime_owner_lease_identity,
                monotonic_epoch_identity,
                expires_at_monotonic_ns,
                lease_identity,
                consumer,
                reserved: [0; 7],
            })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct InputFrameIdentityV1 {
    pub(crate) focus_serial: u64,
    pub(crate) tail_epoch: u64,
    pub(crate) exact_source_window_ref: u32,
    pub(crate) exact_left_context_ref: u32,
    pub(crate) source_scalar_count: u32,
    pub(crate) caret_scalar: u32,
    pub(crate) selection_start_scalar: u32,
    pub(crate) selection_end_scalar: u32,
    pub(crate) exact_preedit_bytes_ref: u32,
    pub(crate) preedit_cursor_scalar: u32,
    pub(crate) layout_generation: u64,
    pub(crate) config_generation: u64,
    pub(crate) package_generation: u64,
    pub(crate) field_generation: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum FrameInvalidationReasonV1 {
    #[default]
    Focus = 0,
    TailEpoch = 1,
    SourceWindow = 2,
    LeftContext = 3,
    Caret = 4,
    Selection = 5,
    Preedit = 6,
    LayoutGeneration = 7,
    ConfigGeneration = 8,
    PackageGeneration = 9,
    FieldGeneration = 10,
    Lease = 11,
    ProjectionReplay = 12,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct ReplacementSpanV1 {
    pub(crate) scalar_start: u32,
    pub(crate) scalar_len: u32,
    pub(crate) source_scalar_len: u32,
    pub(crate) exact_source_slice_hash: [u64; 2],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct FrameTargetIdentityV1 {
    pub(crate) material_target_ref: u16,
    pub(crate) reserved: u16,
    pub(crate) replacement_span: ReplacementSpanV1,
    pub(crate) exact_projected_target_bytes_ref: u32,
    pub(crate) case_projection_id: u32,
    pub(crate) punctuation_projection_id: u32,
    pub(crate) frame_identity_hash: [u64; 2],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct EditFootprintV1 {
    pub(crate) source_snapshot_ref: u32,
    pub(crate) scalar_start: u32,
    pub(crate) scalar_len: u32,
    pub(crate) projected_scalar_len: u32,
    pub(crate) consumed_separator_mask: u32,
    pub(crate) exact_footprint_digest: [u64; 2],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CandidateStateV1 {
    #[default]
    Born,
    Grounded,
    Rejected(TargetRejectionReasonV1),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum WitnessRejectionReasonV1 {
    GroundingRefMismatch = 1,
    GeometryReplayMismatch = 2,
    MalformedEvidenceRoot = 3,
    StaleWitnessGeneration = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum TargetRejectionReasonV1 {
    TargetAbsentFromGrounding = 1,
    CompleteNoValidGeometry = 2,
    UnsupportedTargetIdentity = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum AbsoluteAuthorityBlockerV1 {
    WitnessOverflow = 1,
    TargetSetOverflow = 2,
    UpstreamEnumerationIncomplete = 3,
    EvidenceIntegrityIncomplete = 4,
    MultipleEditComponents = 5,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum FrameOriginalPreservationVerdictV1 {
    #[default]
    Preserve = 0,
    ReplacePermitted = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct FrameOriginalPreservationV1 {
    pub(crate) prepared_material_lease_id: u64,
    pub(crate) exact_frame_identity_ref: u32,
    pub(crate) prepared_original_material_hash: [u64; 2],
    pub(crate) verdict: FrameOriginalPreservationVerdictV1,
    pub(crate) reserved: [u8; 3],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum CohortAbstainReasonV1 {
    #[default]
    NoGroundedTarget = 0,
    PreserveOriginal = 1,
    IncompleteEnumeration = 2,
    MultipleEditComponents = 3,
    UnresolvedConflict = 4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CohortVerdictV1 {
    Winner(u16),
    Tied {
        members: [u16; MAX_TARGETS_PER_FIELD],
        member_count: u16,
        completeness: EnumerationCompletenessV1,
    },
    Abstain(CohortAbstainReasonV1),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub(crate) struct AuthorityCertificateCoreV1 {
    pub(crate) prepared_material_lease_id: u64,
    pub(crate) exact_frame_identity_ref: u32,
    pub(crate) exact_framed_target_ref: u32,
    pub(crate) exact_cohort_and_preservation_ref: u32,
    pub(crate) schema_versions: u32,
    pub(crate) frame_identity_hash: [u64; 2],
    pub(crate) exact_projected_target_hash: [u64; 2],
    pub(crate) evidence_hash: [u64; 2],
    pub(crate) cohort_hash: [u64; 2],
    pub(crate) completeness_hash: [u64; 2],
    pub(crate) material_generation: u64,
    pub(crate) frame_generation: u64,
    pub(crate) monotonic_epoch_identity: [u64; 2],
    pub(crate) expires_at_monotonic_ns: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuthorityCertificateV1 {
    L2Certified(AuthorityCertificateCoreV1),
    ContextCertified {
        core: AuthorityCertificateCoreV1,
        context_hash: [u64; 2],
        selector_overlay_generation: u64,
    },
}

pub(crate) fn target_relation_from_error_class_tag(tag: &str) -> TargetRelationV1 {
    match tag {
        "wrong_layout" => TargetRelationV1::ExactLayout,
        "partial-layout" | "mixed-script" => TargetRelationV1::MixedLayout,
        "missing-letter" => TargetRelationV1::MissingLetter,
        "sparse-internal-multi-omission" => TargetRelationV1::SparseOmission,
        "extra-letter" => TargetRelationV1::ExtraLetter,
        "repeated-letter" => TargetRelationV1::RepeatedFragment,
        "adjacent-transposition" => TargetRelationV1::AdjacentTransposition,
        "letter-substitution" => TargetRelationV1::Substitution,
        "boundary-shift" | "split-word" => TargetRelationV1::BoundarySplit,
        "glued-words" => TargetRelationV1::BoundaryMerge,
        "grammar-agreement" => TargetRelationV1::MorphologySlot,
        _ => TargetRelationV1::Unsupported,
    }
}

pub(crate) fn stable_bytes_ref(bytes: &[u8]) -> u32 {
    let mut digest = 0x811c_9dc5_u32;
    for byte in bytes {
        digest ^= u32::from(*byte);
        digest = digest.wrapping_mul(0x0100_0193);
    }
    digest
}

fn fnv1a_byte(digest: u64, byte: u8) -> u64 {
    (digest ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
}

fn saturating_u16(value: usize) -> u16 {
    value.min(usize::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    fn witness(relation: TargetRelationV1, grounding_ref: u32) -> TargetWitnessV1 {
        TargetWitnessV1::new(
            relation,
            GroundingNamespaceV1::CanonicalForm,
            VerdictMembershipV1::Grounded,
            0,
            relation as u32,
            grounding_ref,
            grounding_ref + 10,
            700,
            1,
        )
    }

    #[test]
    fn target_evidence_layout_budget_is_bounded() {
        let witness_bytes = size_of::<TargetWitnessV1>();
        let evidence_set_bytes = size_of::<TargetEvidenceSetV1>();
        let lease_bytes = size_of::<PreparedMaterialLeaseV1>();
        let prepared_field_bytes = size_of::<PreparedTargetMaterialV1>();
        let evidence_payload_bytes = size_of::<TargetEvidenceSetV1>() * MAX_TARGETS_PER_FIELD;
        let active_lease_metadata_bytes = lease_bytes * MAX_PINNED_PREPARED_FIELDS;
        let delta_160_bytes = prepared_field_bytes * 160;
        eprintln!(
            "target_evidence_layout witness_bytes={witness_bytes} evidence_set_bytes={evidence_set_bytes} lease_bytes={lease_bytes} evidence_payload_bytes={evidence_payload_bytes} prepared_field_bytes={prepared_field_bytes} active_lease_metadata_bytes={active_lease_metadata_bytes} delta_160_bytes={delta_160_bytes}"
        );

        assert!(witness_bytes <= 24);
        assert!(evidence_set_bytes <= 128);
        assert!(lease_bytes <= 128);
        assert!(evidence_payload_bytes <= 9_472);
        assert!(prepared_field_bytes <= 12_288);
        assert!(active_lease_metadata_bytes <= 4_096);
        assert!(delta_160_bytes <= 1_966_080);
        assert_eq!(MAX_TARGETS_PER_FIELD, 74);
        assert_eq!(MAX_TARGET_WITNESSES_PER_TARGET, 4);
        assert_eq!(MAX_PINNED_PREPARED_FIELDS, 32);
        assert_eq!(MAX_LEASE_CONSUMERS_PER_FIELD, 8);
    }

    #[test]
    fn witness_merge_is_permutation_invariant_and_aliases_do_not_vote() {
        let mut alias = witness(TargetRelationV1::MissingLetter, 7);
        alias.support_milli = 900;
        alias.provenance_annotations = 4;
        let left = TargetEvidenceSetV1::from_bounded([
            witness(TargetRelationV1::ExtraLetter, 4),
            alias,
            witness(TargetRelationV1::MissingLetter, 7),
        ]);
        let right = TargetEvidenceSetV1::from_bounded([
            witness(TargetRelationV1::MissingLetter, 7),
            witness(TargetRelationV1::ExtraLetter, 4),
            alias,
        ]);
        assert_eq!(left, right);
        assert_eq!(left.logical_count(), 2);
        assert_eq!(left.retained_count(), 2);
        let merged = left
            .witnesses()
            .iter()
            .find(|witness| witness.relation == TargetRelationV1::MissingLetter)
            .unwrap();
        assert_eq!(merged.support_milli, 900);
        assert_eq!(merged.provenance_annotations, 5);
    }

    #[test]
    fn malformed_accelerators_are_canonicalized_before_retention() {
        let canonical = witness(TargetRelationV1::MissingLetter, 7);
        let mut malformed = canonical;
        malformed.semantic_root_accelerator ^= u32::MAX;

        let expected = TargetEvidenceSetV1::from_one(canonical);
        assert_eq!(TargetEvidenceSetV1::from_one(malformed), expected);
        assert_eq!(
            TargetEvidenceSetV1::from_bounded([malformed, canonical]),
            TargetEvidenceSetV1::from_bounded([canonical, malformed])
        );
        assert_eq!(
            expected.witnesses()[0].semantic_root_accelerator,
            canonical.semantic_root_digest() as u32
        );
    }

    #[test]
    fn overflow_is_explicit_and_keeps_a_deterministic_prefix() {
        let left = TargetEvidenceSetV1::from_bounded([
            witness(TargetRelationV1::MissingLetter, 1),
            witness(TargetRelationV1::ExtraLetter, 2),
            witness(TargetRelationV1::Substitution, 3),
            witness(TargetRelationV1::AdjacentTransposition, 4),
            witness(TargetRelationV1::SparseOmission, 5),
        ]);
        let right = TargetEvidenceSetV1::from_bounded([
            witness(TargetRelationV1::SparseOmission, 5),
            witness(TargetRelationV1::AdjacentTransposition, 4),
            witness(TargetRelationV1::Substitution, 3),
            witness(TargetRelationV1::ExtraLetter, 2),
            witness(TargetRelationV1::MissingLetter, 1),
        ]);
        assert_eq!(left, right);
        assert_eq!(left.state(), EnumerationStateV1::Overflow);
        assert_eq!(left.logical_count(), 5);
        assert_eq!(left.retained_count(), 4);
        assert_eq!(left.all_seen_digest(), right.all_seen_digest());
    }

    #[test]
    fn overflow_merge_is_partition_and_producer_order_invariant() {
        let witnesses = [
            witness(TargetRelationV1::MissingLetter, 1),
            witness(TargetRelationV1::ExtraLetter, 2),
            witness(TargetRelationV1::Substitution, 3),
            witness(TargetRelationV1::AdjacentTransposition, 4),
            witness(TargetRelationV1::SparseOmission, 5),
            witness(TargetRelationV1::RepeatedFragment, 6),
        ];
        let partitioned =
            TargetEvidenceSetV1::from_bounded([witnesses[0], witnesses[1], witnesses[2]]).merge(
                TargetEvidenceSetV1::from_bounded([witnesses[3], witnesses[4], witnesses[5]]),
            );
        let reversed =
            TargetEvidenceSetV1::from_bounded([witnesses[5], witnesses[2], witnesses[0]]).merge(
                TargetEvidenceSetV1::from_bounded([witnesses[4], witnesses[1], witnesses[3]]),
            );

        assert_eq!(partitioned, reversed);
        assert_eq!(partitioned.state(), EnumerationStateV1::Overflow);
        assert_eq!(partitioned.logical_count(), 6);

        let folded = witnesses
            .into_iter()
            .fold(TargetEvidenceSetV1::complete_empty(), |set, witness| {
                set.merge(TargetEvidenceSetV1::from_one(witness))
            });
        let reverse_folded = witnesses
            .into_iter()
            .rev()
            .fold(TargetEvidenceSetV1::complete_empty(), |set, witness| {
                set.merge(TargetEvidenceSetV1::from_one(witness))
            });
        assert_eq!(folded, reverse_folded);
        assert_eq!(folded.all_seen_digest(), partitioned.all_seen_digest());
        assert_eq!(folded.logical_count(), 5);
    }

    #[test]
    fn incomplete_merge_preserves_reason_and_rejects_scope_mismatch() {
        let mut overflow = TargetEvidenceSetV1::from_bounded([
            witness(TargetRelationV1::MissingLetter, 1),
            witness(TargetRelationV1::ExtraLetter, 2),
            witness(TargetRelationV1::Substitution, 3),
            witness(TargetRelationV1::AdjacentTransposition, 4),
            witness(TargetRelationV1::SparseOmission, 5),
        ]);
        overflow.reason = IncompletenessReasonV1::WorkBudgetExceeded;
        let merged = TargetEvidenceSetV1::complete_empty().merge(overflow);
        assert_eq!(merged.state(), EnumerationStateV1::Overflow);
        assert_eq!(merged.reason(), IncompletenessReasonV1::WorkBudgetExceeded);

        assert!(CompletenessScopeV1::relation_partition(0).is_none());
        assert!(CompletenessScopeV1::edit_footprint_partition(0).is_none());
        let other_scope = TargetEvidenceSetV1::complete_empty().with_exhaustive_scope(
            CompletenessScopeV1::relation_partition(7).expect("non-zero proof reference"),
        );
        let rejected = merged.merge(other_scope);
        let rejected_reversed = other_scope.merge(merged);
        assert_eq!(rejected, rejected_reversed);
        assert_eq!(rejected.state(), EnumerationStateV1::Failed);
        assert_eq!(rejected.reason(), IncompletenessReasonV1::IntegrityFailure);
        assert_eq!(rejected.scope(), CompletenessScopeV1::WholePreparedField);
    }

    #[test]
    fn l2_legacy_singleton_roundtrip_is_exact() {
        for legacy in [
            L2ImeTargetEvidence::None,
            L2ImeTargetEvidence::LexicalReconstruction,
            L2ImeTargetEvidence::ContextBoundEdit,
            L2ImeTargetEvidence::CanonicalWinner,
            L2ImeTargetEvidence::LayoutRepair,
            L2ImeTargetEvidence::ExactLayout,
            L2ImeTargetEvidence::Boundary,
        ] {
            assert_eq!(
                L2ImeTargetEvidence::try_from_common(&legacy.to_common()),
                Ok(legacy)
            );
        }
    }

    #[test]
    fn live_legacy_singleton_roundtrip_is_exact() {
        for legacy in [
            ReplacementTargetEvidence::None,
            ReplacementTargetEvidence::ExactLayoutProjection,
            ReplacementTargetEvidence::VerifiedLexicalEdit,
            ReplacementTargetEvidence::ContextBoundLexicalEdit,
            ReplacementTargetEvidence::VerifiedBoundary,
        ] {
            assert_eq!(
                ReplacementTargetEvidence::try_from_common(&legacy.to_common()),
                Ok(legacy)
            );
        }
    }

    #[test]
    fn legacy_reverse_projection_fails_closed() {
        let multiple = TargetEvidenceSetV1::from_bounded([
            witness(TargetRelationV1::MissingLetter, 1),
            witness(TargetRelationV1::ExtraLetter, 2),
        ]);
        assert_eq!(
            L2ImeTargetEvidence::try_from_common(&multiple),
            Err(LegacyProjectionErrorV1::NotSingleton)
        );
        let overflow = TargetEvidenceSetV1::from_bounded([
            witness(TargetRelationV1::MissingLetter, 1),
            witness(TargetRelationV1::ExtraLetter, 2),
            witness(TargetRelationV1::Substitution, 3),
            witness(TargetRelationV1::AdjacentTransposition, 4),
            witness(TargetRelationV1::SparseOmission, 5),
        ]);
        assert_eq!(
            ReplacementTargetEvidence::try_from_common(&overflow),
            Err(LegacyProjectionErrorV1::Incomplete)
        );
        assert_eq!(
            ReplacementTargetEvidence::try_from_common(&TargetEvidenceSetV1::failed(
                IncompletenessReasonV1::IntegrityFailure
            )),
            Err(LegacyProjectionErrorV1::Incomplete)
        );
    }

    #[test]
    fn package_generation_is_validity_not_witness_independence() {
        let witness = witness(TargetRelationV1::MorphologySlot, 17);
        let evidence = TargetEvidenceSetV1::from_bounded([witness, witness]);
        assert_eq!(evidence.logical_count(), 1);

        let left = PreparedMaterialKeyV1 {
            package_generation: 1,
            ..PreparedMaterialKeyV1::default()
        };
        let right = PreparedMaterialKeyV1 {
            package_generation: 2,
            ..PreparedMaterialKeyV1::default()
        };
        assert_ne!(left, right);
    }

    #[test]
    fn target_storage_refuses_the_seventy_fifth_member() {
        let mut targets = BoundedTargetSetV1::default();
        for _ in 0..MAX_TARGETS_PER_FIELD {
            targets.push(PreparedTargetV1::default()).unwrap();
        }
        assert_eq!(targets.as_slice().len(), MAX_TARGETS_PER_FIELD);
        assert_eq!(
            targets.push(PreparedTargetV1::default()),
            Err(TargetSetOverflowV1)
        );
    }

    #[test]
    fn prepared_material_schema_excludes_frame_bound_values() {
        let PreparedTargetMaterialV1 {
            key,
            original,
            targets,
            completeness,
            evidence_tables,
            integrity,
        } = PreparedTargetMaterialV1::default();

        let rebuilt = PreparedTargetMaterialV1 {
            key,
            original,
            targets,
            completeness,
            evidence_tables,
            integrity,
        };
        assert_eq!(rebuilt, PreparedTargetMaterialV1::default());
    }

    #[test]
    fn completeness_states_roundtrip_without_truncation() {
        let complete = EnumerationCompletenessV1::complete(4, [11, 12]);
        let overflow = EnumerationCompletenessV1::overflow(
            4,
            9,
            IncompletenessReasonV1::WorkBudgetExceeded,
            [21, 22],
        );
        let failed = EnumerationCompletenessV1::failed(IncompletenessReasonV1::IntegrityFailure);
        assert_eq!(complete.state(), EnumerationStateV1::Complete);
        assert_eq!(complete.logical_count_lower_bound(), 4);
        assert_eq!(overflow.state(), EnumerationStateV1::Overflow);
        assert_eq!(overflow.logical_count_lower_bound(), 9);
        assert_eq!(overflow.all_seen_digest(), [21, 22]);
        assert_eq!(failed.state(), EnumerationStateV1::Failed);
    }

    #[test]
    fn narrow_completeness_scope_requires_and_retains_partition_proof() {
        assert!(CompletenessScopeV1::relation_partition(0).is_none());
        assert!(CompletenessScopeV1::edit_footprint_partition(0).is_none());

        let scope = CompletenessScopeV1::edit_footprint_partition(41)
            .expect("non-zero exhaustive partition proof reference");
        let complete = EnumerationCompletenessV1::complete_in_scope(3, [31, 32], scope);
        let overflow = EnumerationCompletenessV1::overflow_in_scope(
            3,
            7,
            IncompletenessReasonV1::WorkBudgetExceeded,
            [41, 42],
            scope,
        );

        assert_eq!(complete.scope(), scope);
        assert_eq!(overflow.scope(), scope);
        assert_eq!(
            scope.kind(),
            CompletenessScopeKindV1::EditFootprintPartition
        );
        assert_eq!(
            scope.exhaustive_partition_proof_ref().map(NonZeroU32::get),
            Some(41)
        );
        assert_eq!(
            overflow.reason(),
            IncompletenessReasonV1::WorkBudgetExceeded
        );
    }
}
