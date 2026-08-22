//! Complete frame-bound conflict cohort shadow.
//!
//! This module owns no live rank, display, certificate, admission or mutation.

use std::cmp::Ordering;

use sha2::{Digest, Sha256};

use super::candidate_state::{validate_bound_target, CandidateValidityShadowV1};
use super::material_frame::{
    validate_lease, BoundFrameTargetV1, ExactInputFrameV1, PreparedTargetMaterialShadowV1,
};
use crate::typing_transition::target_evidence::{
    stable_bytes_ref, AbsoluteAuthorityBlockerV1, CandidateStateV1, CohortAbstainReasonV1,
    CohortVerdictV1, EditFootprintV1, EnumerationCompletenessV1, EnumerationStateV1,
    FrameOriginalPreservationV1, FrameOriginalPreservationVerdictV1, MaterialTargetIdentityV1,
    PreparedMaterialLeaseV1, TargetWitnessV1, MAX_TARGETS_PER_FIELD,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticRootKeyV1([u8; 16]);

impl From<TargetWitnessV1> for SemanticRootKeyV1 {
    fn from(witness: TargetWitnessV1) -> Self {
        let mut bytes = [0_u8; 16];
        bytes[0] = witness.relation as u8;
        bytes[1] = witness.grounding_namespace as u8;
        bytes[2] = witness.verdict_membership as u8;
        bytes[3] = witness.flags;
        bytes[4..8].copy_from_slice(&witness.operator_ref.to_le_bytes());
        bytes[8..12].copy_from_slice(&witness.grounding_ref.to_le_bytes());
        bytes[12..16].copy_from_slice(&witness.derivation_ref.to_le_bytes());
        Self(bytes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConflictCohortMemberShadowV1 {
    pub(super) material_target_ref: u16,
    pub(super) state: CandidateStateV1,
    pub(super) footprint: EditFootprintV1,
    pub(super) projected_target: String,
    pub(super) authority_blockers: Vec<AbsoluteAuthorityBlockerV1>,
    material_identity: MaterialTargetIdentityV1,
    semantic_roots: Vec<SemanticRootKeyV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConflictCohortShadowV1 {
    pub(super) verdict: CohortVerdictV1,
    pub(super) cohort_hash: [u64; 2],
    pub(super) canonical_member_refs: Vec<u16>,
    pub(super) grounded_member_count: usize,
    pub(super) born_member_count: usize,
    pub(super) rejected_member_count: usize,
    pub(super) component_count: usize,
    pub(super) complete_for_authority: bool,
}

pub(super) fn derive_conflict_cohort_shadow(
    material: &PreparedTargetMaterialShadowV1,
    lease: PreparedMaterialLeaseV1,
    frame: &ExactInputFrameV1,
    bound_candidates: &[(&BoundFrameTargetV1, &CandidateValidityShadowV1)],
    original: Option<FrameOriginalPreservationV1>,
    now_monotonic_ns: u64,
) -> Result<ConflictCohortShadowV1, &'static str> {
    validate_lease(material, lease, frame, now_monotonic_ns)
        .map_err(|_| "conflict cohort material lease is invalid")?;
    if bound_candidates.len() != material.compact().targets.len() {
        return Err("conflict cohort does not cover every retained material target");
    }
    if let Some(preservation) = original {
        if preservation.prepared_material_lease_id != lease.lease_identity {
            return Err("original preservation belongs to a different material lease");
        }
    }

    let mut seen = [false; MAX_TARGETS_PER_FIELD];
    let mut members = Vec::with_capacity(bound_candidates.len());
    for (bound, validity) in bound_candidates {
        validate_bound_target(frame, bound)?;
        let target_ref = usize::from(bound.identity.material_target_ref);
        if target_ref >= seen.len() || seen[target_ref] {
            return Err("conflict cohort target reference is duplicated or outside the field");
        }
        seen[target_ref] = true;
        if validity.material_target_ref != bound.identity.material_target_ref
            || validity.exact_projected_target_hash
                != digest128(Sha256::digest(bound.projected_target.as_bytes()).into())
        {
            return Err("candidate validity does not belong to the bound target");
        }
        let target = material
            .compact()
            .targets
            .as_slice()
            .get(target_ref)
            .ok_or("conflict cohort target is outside prepared material")?;
        let mut semantic_roots = target
            .witnesses
            .witnesses()
            .iter()
            .copied()
            .map(SemanticRootKeyV1::from)
            .collect::<Vec<_>>();
        semantic_roots.sort_unstable();
        semantic_roots.dedup();
        let mut blockers = validity.authority_blockers.clone();
        canonicalize_blockers(&mut blockers);
        members.push(ConflictCohortMemberShadowV1 {
            material_target_ref: bound.identity.material_target_ref,
            state: validity.state,
            footprint: exact_footprint(frame, bound),
            projected_target: bound.projected_target.clone(),
            authority_blockers: blockers,
            material_identity: target.identity,
            semantic_roots,
        });
    }
    if seen[..material.compact().targets.len()]
        .iter()
        .any(|present| !present)
    {
        return Err("conflict cohort retained target coverage has a hole");
    }

    settle_conflict_members(members, material.completeness(), original)
}

fn settle_conflict_members(
    members: Vec<ConflictCohortMemberShadowV1>,
    completeness: EnumerationCompletenessV1,
    original: Option<FrameOriginalPreservationV1>,
) -> Result<ConflictCohortShadowV1, &'static str> {
    let mut members = merge_exact_duplicate_targets(members);
    members.sort_unstable_by(canonical_member_cmp);
    let rejected_member_count = members
        .iter()
        .filter(|member| matches!(member.state, CandidateStateV1::Rejected(_)))
        .count();
    let active = members
        .iter()
        .filter(|member| !matches!(member.state, CandidateStateV1::Rejected(_)))
        .collect::<Vec<_>>();
    let grounded = active
        .iter()
        .copied()
        .filter(|member| member.state == CandidateStateV1::Grounded)
        .collect::<Vec<_>>();
    let born_member_count = active
        .iter()
        .filter(|member| member.state == CandidateStateV1::Born)
        .count();
    let component_count = conflict_component_count(&active);
    let blocker_present = active
        .iter()
        .any(|member| !member.authority_blockers.is_empty());
    let complete_for_authority = completeness.state() == EnumerationStateV1::Complete
        && born_member_count == 0
        && !blocker_present
        && component_count <= 1
        && matches!(
            original.map(|preservation| preservation.verdict),
            Some(FrameOriginalPreservationVerdictV1::ReplacePermitted)
        );

    let verdict = if matches!(
        original.map(|preservation| preservation.verdict),
        Some(FrameOriginalPreservationVerdictV1::Preserve)
    ) {
        CohortVerdictV1::Abstain(CohortAbstainReasonV1::PreserveOriginal)
    } else if original.is_none() {
        CohortVerdictV1::Abstain(CohortAbstainReasonV1::UnresolvedConflict)
    } else if component_count > 1 {
        CohortVerdictV1::Abstain(CohortAbstainReasonV1::MultipleEditComponents)
    } else if grounded.is_empty() {
        let reason =
            if born_member_count > 0 || completeness.state() != EnumerationStateV1::Complete {
                CohortAbstainReasonV1::IncompleteEnumeration
            } else {
                CohortAbstainReasonV1::NoGroundedTarget
            };
        CohortVerdictV1::Abstain(reason)
    } else if !complete_for_authority {
        if grounded.len() >= 2 {
            tied_verdict(&grounded, completeness)
        } else {
            CohortVerdictV1::Abstain(CohortAbstainReasonV1::IncompleteEnumeration)
        }
    } else if grounded.len() == 1 {
        CohortVerdictV1::Winner(grounded[0].material_target_ref)
    } else {
        tied_verdict(&grounded, completeness)
    };

    let canonical_member_refs = members
        .iter()
        .map(|member| member.material_target_ref)
        .collect::<Vec<_>>();
    let cohort_hash = cohort_digest(&members, completeness, original);
    Ok(ConflictCohortShadowV1 {
        verdict,
        cohort_hash,
        canonical_member_refs,
        grounded_member_count: grounded.len(),
        born_member_count,
        rejected_member_count,
        component_count,
        complete_for_authority,
    })
}

fn tied_verdict(
    grounded: &[&ConflictCohortMemberShadowV1],
    completeness: EnumerationCompletenessV1,
) -> CohortVerdictV1 {
    let mut members = [0_u16; MAX_TARGETS_PER_FIELD];
    for (slot, member) in members.iter_mut().zip(grounded.iter()) {
        *slot = member.material_target_ref;
    }
    CohortVerdictV1::Tied {
        members,
        member_count: grounded.len() as u16,
        completeness,
    }
}

fn merge_exact_duplicate_targets(
    mut members: Vec<ConflictCohortMemberShadowV1>,
) -> Vec<ConflictCohortMemberShadowV1> {
    members.sort_unstable_by(|left, right| {
        duplicate_key_cmp(left, right).then_with(|| canonical_member_cmp(left, right))
    });
    let mut merged = Vec::<ConflictCohortMemberShadowV1>::with_capacity(members.len());
    for member in members {
        if let Some(current) = merged
            .last_mut()
            .filter(|current| duplicate_key_cmp(current, &member) == Ordering::Equal)
        {
            current.material_target_ref =
                current.material_target_ref.min(member.material_target_ref);
            current.state = merged_candidate_state(current.state, member.state);
            current.semantic_roots.extend(member.semantic_roots);
            current.semantic_roots.sort_unstable();
            current.semantic_roots.dedup();
            current.authority_blockers.extend(member.authority_blockers);
            canonicalize_blockers(&mut current.authority_blockers);
        } else {
            merged.push(member);
        }
    }
    merged
}

fn merged_candidate_state(left: CandidateStateV1, right: CandidateStateV1) -> CandidateStateV1 {
    match (left, right) {
        (CandidateStateV1::Grounded, _) | (_, CandidateStateV1::Grounded) => {
            CandidateStateV1::Grounded
        }
        (CandidateStateV1::Born, _) | (_, CandidateStateV1::Born) => CandidateStateV1::Born,
        (rejected, _) => rejected,
    }
}

fn conflict_component_count(active: &[&ConflictCohortMemberShadowV1]) -> usize {
    let mut component = (0..active.len()).collect::<Vec<_>>();
    for left in 0..active.len() {
        for right in (left + 1)..active.len() {
            if footprints_conflict(active[left].footprint, active[right].footprint) {
                union_components(&mut component, left, right);
            }
        }
    }
    let mut roots = Vec::new();
    for index in 0..active.len() {
        let root = component_root(&mut component, index);
        if !roots.contains(&root) {
            roots.push(root);
        }
    }
    roots.len()
}

fn footprints_conflict(left: EditFootprintV1, right: EditFootprintV1) -> bool {
    if left.source_snapshot_ref != right.source_snapshot_ref {
        return false;
    }
    if left.consumed_separator_mask & right.consumed_separator_mask != 0 {
        return true;
    }
    let left_end = left.scalar_start.saturating_add(left.scalar_len);
    let right_end = right.scalar_start.saturating_add(right.scalar_len);
    if left.scalar_len == 0 || right.scalar_len == 0 {
        return left.scalar_start == right.scalar_start;
    }
    left.scalar_start < right_end && right.scalar_start < left_end
}

fn union_components(components: &mut [usize], left: usize, right: usize) {
    let left_root = component_root(components, left);
    let right_root = component_root(components, right);
    if left_root != right_root {
        components[right_root] = left_root;
    }
}

fn component_root(components: &mut [usize], index: usize) -> usize {
    let mut root = index;
    while components[root] != root {
        root = components[root];
    }
    let mut cursor = index;
    while components[cursor] != cursor {
        let next = components[cursor];
        components[cursor] = root;
        cursor = next;
    }
    root
}

fn exact_footprint(frame: &ExactInputFrameV1, bound: &BoundFrameTargetV1) -> EditFootprintV1 {
    let span = bound.identity.replacement_span;
    let mut hasher = Sha256::new();
    hasher.update(b"lay-edit-footprint-v1\0");
    hasher.update(frame.source_window().as_bytes());
    hasher.update(span.scalar_start.to_le_bytes());
    hasher.update(span.scalar_len.to_le_bytes());
    hasher.update(bound.projected_target.as_bytes());
    let projected_scalar_len = bound.projected_target.chars().count() as u32;
    hasher.update(projected_scalar_len.to_le_bytes());
    EditFootprintV1 {
        source_snapshot_ref: stable_bytes_ref(frame.source_window().as_bytes()),
        scalar_start: span.scalar_start,
        scalar_len: span.scalar_len,
        projected_scalar_len,
        consumed_separator_mask: 0,
        exact_footprint_digest: digest128(hasher.finalize().into()),
    }
}

fn duplicate_key_cmp(
    left: &ConflictCohortMemberShadowV1,
    right: &ConflictCohortMemberShadowV1,
) -> Ordering {
    footprint_bytes(left.footprint)
        .cmp(&footprint_bytes(right.footprint))
        .then_with(|| {
            left.projected_target
                .as_bytes()
                .cmp(right.projected_target.as_bytes())
        })
}

fn canonical_member_cmp(
    left: &ConflictCohortMemberShadowV1,
    right: &ConflictCohortMemberShadowV1,
) -> Ordering {
    duplicate_key_cmp(left, right)
        .then_with(|| {
            material_identity_bytes(left.material_identity)
                .cmp(&material_identity_bytes(right.material_identity))
        })
        .then_with(|| left.semantic_roots.cmp(&right.semantic_roots))
        .then_with(|| left.material_target_ref.cmp(&right.material_target_ref))
}

fn cohort_digest(
    members: &[ConflictCohortMemberShadowV1],
    completeness: EnumerationCompletenessV1,
    original: Option<FrameOriginalPreservationV1>,
) -> [u64; 2] {
    let mut hasher = Sha256::new();
    hasher.update(b"lay-conflict-cohort-shadow-v1\0");
    for member in members {
        hasher.update(footprint_bytes(member.footprint));
        hash_len_bytes(&mut hasher, member.projected_target.as_bytes());
        hasher.update(material_identity_bytes(member.material_identity));
        for root in &member.semantic_roots {
            hasher.update(root.0);
        }
        hasher.update([candidate_state_tag(member.state)]);
        for blocker in &member.authority_blockers {
            hasher.update([*blocker as u8]);
        }
    }
    hasher.update(completeness_bytes(completeness));
    hasher.update([match original.map(|value| value.verdict) {
        None => 0,
        Some(FrameOriginalPreservationVerdictV1::Preserve) => 1,
        Some(FrameOriginalPreservationVerdictV1::ReplacePermitted) => 2,
    }]);
    digest128(hasher.finalize().into())
}

fn footprint_bytes(footprint: EditFootprintV1) -> [u8; 36] {
    let mut bytes = [0_u8; 36];
    bytes[0..4].copy_from_slice(&footprint.source_snapshot_ref.to_le_bytes());
    bytes[4..8].copy_from_slice(&footprint.scalar_start.to_le_bytes());
    bytes[8..12].copy_from_slice(&footprint.scalar_len.to_le_bytes());
    bytes[12..16].copy_from_slice(&footprint.projected_scalar_len.to_le_bytes());
    bytes[16..20].copy_from_slice(&footprint.consumed_separator_mask.to_le_bytes());
    bytes[20..28].copy_from_slice(&footprint.exact_footprint_digest[0].to_le_bytes());
    bytes[28..36].copy_from_slice(&footprint.exact_footprint_digest[1].to_le_bytes());
    bytes
}

fn material_identity_bytes(identity: MaterialTargetIdentityV1) -> [u8; 24] {
    let mut bytes = [0_u8; 24];
    bytes[0..4].copy_from_slice(&identity.normalized_scalars_ref.to_le_bytes());
    bytes[4..8].copy_from_slice(&identity.canonical_bytes_ref.to_le_bytes());
    bytes[8..12].copy_from_slice(&identity.normalization_layout_profile_id.0.to_le_bytes());
    bytes[12..16].copy_from_slice(&identity.separator_profile_id.0.to_le_bytes());
    bytes[16..18].copy_from_slice(&identity.exact_scalar_count.to_le_bytes());
    bytes[18..20].copy_from_slice(&identity.flags.to_le_bytes());
    bytes[20..24].copy_from_slice(&identity.accelerator.to_le_bytes());
    bytes
}

fn completeness_bytes(completeness: EnumerationCompletenessV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(32);
    bytes.push(completeness.state() as u8);
    bytes.push(completeness.reason() as u8);
    bytes.push(completeness.scope().kind() as u8);
    bytes.extend_from_slice(&completeness.retained_count().to_le_bytes());
    bytes.extend_from_slice(&completeness.logical_count_lower_bound().to_le_bytes());
    bytes.extend_from_slice(&completeness.all_seen_digest()[0].to_le_bytes());
    bytes.extend_from_slice(&completeness.all_seen_digest()[1].to_le_bytes());
    bytes.extend_from_slice(
        &completeness
            .scope()
            .exhaustive_partition_proof_ref()
            .map(|value| value.get())
            .unwrap_or_default()
            .to_le_bytes(),
    );
    bytes
}

fn candidate_state_tag(state: CandidateStateV1) -> u8 {
    match state {
        CandidateStateV1::Born => 0,
        CandidateStateV1::Grounded => 1,
        CandidateStateV1::Rejected(reason) => 2 + reason as u8,
    }
}

fn canonicalize_blockers(blockers: &mut Vec<AbsoluteAuthorityBlockerV1>) {
    blockers.sort_unstable_by_key(|blocker| *blocker as u8);
    blockers.dedup();
}

fn hash_len_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
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
    use crate::typing_transition::target_evidence::{
        CompletenessScopeV1, IncompletenessReasonV1, NormalizationLayoutProfileIdV1,
        SeparatorProfileIdV1, TargetRejectionReasonV1,
    };

    fn preservation(verdict: FrameOriginalPreservationVerdictV1) -> FrameOriginalPreservationV1 {
        FrameOriginalPreservationV1 {
            prepared_material_lease_id: 1,
            exact_frame_identity_ref: 1,
            prepared_original_material_hash: [1, 2],
            verdict,
            reserved: [0; 3],
        }
    }

    fn member(
        target_ref: u16,
        state: CandidateStateV1,
        start: u32,
        len: u32,
        surface: &str,
    ) -> ConflictCohortMemberShadowV1 {
        ConflictCohortMemberShadowV1 {
            material_target_ref: target_ref,
            state,
            footprint: EditFootprintV1 {
                source_snapshot_ref: 1,
                scalar_start: start,
                scalar_len: len,
                projected_scalar_len: surface.chars().count() as u32,
                consumed_separator_mask: 0,
                exact_footprint_digest: [u64::from(start), u64::from(len)],
            },
            projected_target: surface.to_string(),
            authority_blockers: Vec::new(),
            material_identity: MaterialTargetIdentityV1 {
                normalized_scalars_ref: target_ref as u32 + 1,
                canonical_bytes_ref: target_ref as u32 + 10,
                normalization_layout_profile_id: NormalizationLayoutProfileIdV1(1),
                separator_profile_id: SeparatorProfileIdV1(1),
                exact_scalar_count: surface.chars().count() as u16,
                flags: 0,
                accelerator: target_ref as u32,
            },
            semantic_roots: vec![SemanticRootKeyV1([target_ref as u8; 16])],
        }
    }

    fn complete(count: usize) -> EnumerationCompletenessV1 {
        EnumerationCompletenessV1::complete(count, [1, 2])
    }

    fn settle(
        members: Vec<ConflictCohortMemberShadowV1>,
        completeness: EnumerationCompletenessV1,
    ) -> ConflictCohortShadowV1 {
        settle_conflict_members(
            members,
            completeness,
            Some(preservation(
                FrameOriginalPreservationVerdictV1::ReplacePermitted,
            )),
        )
        .unwrap()
    }

    #[test]
    fn one_complete_grounded_component_is_winner() {
        let result = settle(
            vec![member(7, CandidateStateV1::Grounded, 0, 4, "word")],
            complete(1),
        );
        assert_eq!(result.verdict, CohortVerdictV1::Winner(7));
        assert!(result.complete_for_authority);
    }

    #[test]
    fn grounded_and_conflicting_born_cannot_be_singleton() {
        let result = settle(
            vec![
                member(1, CandidateStateV1::Grounded, 0, 4, "word"),
                member(2, CandidateStateV1::Born, 0, 4, "ward"),
            ],
            complete(2),
        );
        assert_eq!(
            result.verdict,
            CohortVerdictV1::Abstain(CohortAbstainReasonV1::IncompleteEnumeration)
        );
    }

    #[test]
    fn two_complete_grounded_members_are_tied_in_canonical_order() {
        let result = settle(
            vec![
                member(9, CandidateStateV1::Grounded, 0, 4, "zeta"),
                member(3, CandidateStateV1::Grounded, 0, 4, "alpha"),
            ],
            complete(2),
        );
        match result.verdict {
            CohortVerdictV1::Tied {
                members,
                member_count,
                completeness,
            } => {
                assert_eq!(member_count, 2);
                assert_eq!(&members[..2], &[9, 3]);
                assert_eq!(completeness.state(), EnumerationStateV1::Complete);
            }
            other => panic!("unexpected verdict: {other:?}"),
        }
    }

    #[test]
    fn incomplete_field_never_issues_winner() {
        let result = settle(
            vec![member(1, CandidateStateV1::Grounded, 0, 4, "word")],
            EnumerationCompletenessV1::overflow(
                1,
                2,
                IncompletenessReasonV1::UpstreamIncomplete,
                [1, 2],
            ),
        );
        assert_eq!(
            result.verdict,
            CohortVerdictV1::Abstain(CohortAbstainReasonV1::IncompleteEnumeration)
        );
    }

    #[test]
    fn preservation_is_consumed_before_winner() {
        let result = settle_conflict_members(
            vec![member(1, CandidateStateV1::Grounded, 0, 4, "word")],
            complete(1),
            Some(preservation(FrameOriginalPreservationVerdictV1::Preserve)),
        )
        .unwrap();
        assert_eq!(
            result.verdict,
            CohortVerdictV1::Abstain(CohortAbstainReasonV1::PreserveOriginal)
        );
    }

    #[test]
    fn independent_components_are_not_composed() {
        let result = settle(
            vec![
                member(1, CandidateStateV1::Grounded, 0, 2, "ab"),
                member(2, CandidateStateV1::Grounded, 4, 2, "cd"),
            ],
            complete(2),
        );
        assert_eq!(result.component_count, 2);
        assert_eq!(
            result.verdict,
            CohortVerdictV1::Abstain(CohortAbstainReasonV1::MultipleEditComponents)
        );
    }

    #[test]
    fn rejected_target_is_not_a_tie_member() {
        let result = settle(
            vec![
                member(1, CandidateStateV1::Grounded, 0, 4, "word"),
                member(
                    2,
                    CandidateStateV1::Rejected(TargetRejectionReasonV1::CompleteNoValidGeometry),
                    0,
                    4,
                    "ward",
                ),
            ],
            complete(2),
        );
        assert_eq!(result.verdict, CohortVerdictV1::Winner(1));
        assert_eq!(result.rejected_member_count, 1);
    }

    #[test]
    fn exact_duplicates_merge_before_settlement_and_input_order_is_irrelevant() {
        let left = member(8, CandidateStateV1::Grounded, 0, 4, "word");
        let mut right = member(2, CandidateStateV1::Born, 0, 4, "word");
        right.footprint = left.footprint;
        let forward = settle(vec![left.clone(), right.clone()], complete(2));
        let reverse = settle(vec![right, left], complete(2));
        assert_eq!(forward, reverse);
        assert_eq!(forward.canonical_member_refs, vec![2]);
        assert_eq!(forward.verdict, CohortVerdictV1::Winner(2));
    }

    #[test]
    fn unresolved_original_cannot_issue_winner() {
        let result = settle_conflict_members(
            vec![member(1, CandidateStateV1::Grounded, 0, 4, "word")],
            complete(1),
            None,
        )
        .unwrap();
        assert_eq!(
            result.verdict,
            CohortVerdictV1::Abstain(CohortAbstainReasonV1::UnresolvedConflict)
        );
    }

    #[test]
    fn narrow_completeness_scope_retains_its_partition_identity() {
        let scope = CompletenessScopeV1::edit_footprint_partition(77).unwrap();
        let completeness = EnumerationCompletenessV1::complete_in_scope(1, [1, 2], scope);
        let result = settle(
            vec![member(1, CandidateStateV1::Grounded, 0, 4, "word")],
            completeness,
        );
        assert_eq!(result.verdict, CohortVerdictV1::Winner(1));
        assert_ne!(result.cohort_hash, [0; 2]);
    }
}
