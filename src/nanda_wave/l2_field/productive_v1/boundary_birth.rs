//! Exact context-neutral split/merge birth for Slice 6 shadow proof.
//!
//! Boundary membership establishes a composite target identity only. It never
//! grants authority and it never repairs lexical bytes while changing a
//! separator.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::typing_transition::target_evidence::{
    stable_bytes_ref, EnumerationWorkCountersV1, GroundingNamespaceV1, IncompletenessReasonV1,
    TargetRelationV1,
};

use super::super::runtime::StandaloneL2Field;
use super::contour_birth::{
    ExactContourIdentityUnionV1, ExactContourIdentityV1, ExactContourLexiconV1,
};
use crate::nanda_wave::lexical_grokking::ExactL11SurfaceIndexV1;

pub(super) const MAX_BOUNDARY_INPUT_SCALARS: usize = 32;
pub(super) const MAX_BOUNDARY_EXACT_LOOKUPS: u64 = 64;
pub(super) const MAX_BOUNDARY_OPERATOR_STEPS: u64 = 64;
pub(super) const ASCII_SPACE_SEPARATOR_PROFILE: u32 = 1;

const OP_BOUNDARY_SPLIT: u32 = 0x5336_0001;
const OP_BOUNDARY_MERGE: u32 = 0x5336_0002;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ExactBoundaryPartGroundingV1 {
    pub(super) normalized_surface: String,
    pub(super) grounding_namespace: GroundingNamespaceV1,
    pub(super) grounding_ref: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CompositeBoundaryGroundingV1 {
    pub(super) ordered_part_groundings: Vec<ExactBoundaryPartGroundingV1>,
    pub(super) exact_segmentation_scalars: Vec<u16>,
    pub(super) separator_profile_id: u32,
    pub(super) merged_target_grounding: Option<ExactBoundaryPartGroundingV1>,
}

impl CompositeBoundaryGroundingV1 {
    pub(super) fn exact_bytes(&self) -> Vec<u8> {
        let mut bytes = b"lay-composite-boundary-grounding-v1\0".to_vec();
        bytes.extend_from_slice(&self.separator_profile_id.to_le_bytes());
        bytes.extend_from_slice(&(self.ordered_part_groundings.len() as u64).to_le_bytes());
        for part in &self.ordered_part_groundings {
            hash_len_vec(&mut bytes, part.normalized_surface.as_bytes());
            bytes.push(part.grounding_namespace as u8);
            bytes.extend_from_slice(&part.grounding_ref.to_le_bytes());
        }
        bytes.extend_from_slice(&(self.exact_segmentation_scalars.len() as u64).to_le_bytes());
        for scalar in &self.exact_segmentation_scalars {
            bytes.extend_from_slice(&scalar.to_le_bytes());
        }
        match &self.merged_target_grounding {
            Some(target) => {
                bytes.push(1);
                hash_len_vec(&mut bytes, target.normalized_surface.as_bytes());
                bytes.push(target.grounding_namespace as u8);
                bytes.extend_from_slice(&target.grounding_ref.to_le_bytes());
            }
            None => bytes.push(0),
        }
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TypedBoundaryBirthV1 {
    pub(super) normalized_surface: String,
    pub(super) relation: TargetRelationV1,
    pub(super) operator_ref: u32,
    pub(super) grounding_ref: u32,
    pub(super) derivation_ref: u32,
    pub(super) composite_grounding: CompositeBoundaryGroundingV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TypedBoundaryBirthEnumerationV1 {
    pub(super) births: Vec<TypedBoundaryBirthV1>,
    pub(super) work: EnumerationWorkCountersV1,
    pub(super) logical_match_count: usize,
    pub(super) all_seen_digest: [u64; 2],
    pub(super) overflow_reason: Option<IncompletenessReasonV1>,
}

impl TypedBoundaryBirthEnumerationV1 {
    pub(super) fn complete_empty() -> Self {
        Self {
            births: Vec::new(),
            work: EnumerationWorkCountersV1::default(),
            logical_match_count: 0,
            all_seen_digest: digest128(Sha256::digest(b"lay-boundary-birth-v1\0").into()),
            overflow_reason: None,
        }
    }

    pub(super) fn work_within_budget(&self) -> bool {
        self.work.grounding_lookups <= MAX_BOUNDARY_EXACT_LOOKUPS
            && self.work.operator_steps <= MAX_BOUNDARY_OPERATOR_STEPS
            && self.overflow_reason.is_none()
    }
}

pub(super) fn enumerate_typed_boundary_births(
    observed: &str,
    lexicon: &impl ExactContourLexiconV1,
) -> TypedBoundaryBirthEnumerationV1 {
    let normalized = observed.to_lowercase();
    if normalized.is_empty() {
        return TypedBoundaryBirthEnumerationV1::complete_empty();
    }
    if normalized.chars().count() > MAX_BOUNDARY_INPUT_SCALARS {
        let mut result = TypedBoundaryBirthEnumerationV1::complete_empty();
        result.overflow_reason = Some(IncompletenessReasonV1::WorkBudgetExceeded);
        return result;
    }

    let mut state = BoundaryEnumerationStateV1::default();
    if normalized.contains(' ') {
        enumerate_merge(&normalized, lexicon, &mut state);
    } else if normalized.chars().all(char::is_alphabetic) {
        enumerate_splits(&normalized, lexicon, &mut state);
    }
    state.finish()
}

pub(super) fn enumerate_typed_boundary_births_from_packages(
    observed: &str,
    canonical: &StandaloneL2Field,
    l11: Option<&ExactL11SurfaceIndexV1>,
) -> TypedBoundaryBirthEnumerationV1 {
    enumerate_typed_boundary_births(observed, &ExactContourIdentityUnionV1::new(canonical, l11))
}

#[derive(Default)]
struct BoundaryEnumerationStateV1 {
    births: BTreeMap<(String, TargetRelationV1, u32, u32), TypedBoundaryBirthV1>,
    work: EnumerationWorkCountersV1,
    logical_surfaces: BTreeMap<String, ()>,
    exhausted: bool,
}

impl BoundaryEnumerationStateV1 {
    fn exact_identities(
        &mut self,
        lexicon: &impl ExactContourLexiconV1,
        surface: &str,
    ) -> Vec<ExactContourIdentityV1> {
        if self.exhausted || self.work.grounding_lookups >= MAX_BOUNDARY_EXACT_LOOKUPS {
            self.exhausted = true;
            return Vec::new();
        }
        self.work.grounding_lookups += 1;
        lexicon.exact_identities(surface)
    }

    fn observe(
        &mut self,
        normalized_surface: String,
        relation: TargetRelationV1,
        operator_ref: u32,
        composite_grounding: CompositeBoundaryGroundingV1,
    ) {
        if self.exhausted || self.work.operator_steps >= MAX_BOUNDARY_OPERATOR_STEPS {
            self.exhausted = true;
            return;
        }
        self.work.operator_steps += 1;
        let composite_bytes = composite_grounding.exact_bytes();
        let grounding_ref = stable_bytes_ref(&composite_bytes);
        let derivation_ref = boundary_derivation_ref(
            &normalized_surface,
            relation,
            operator_ref,
            &composite_bytes,
        );
        self.logical_surfaces
            .entry(normalized_surface.clone())
            .or_insert(());
        self.births
            .entry((
                normalized_surface.clone(),
                relation,
                grounding_ref,
                derivation_ref,
            ))
            .or_insert(TypedBoundaryBirthV1 {
                normalized_surface,
                relation,
                operator_ref,
                grounding_ref,
                derivation_ref,
                composite_grounding,
            });
    }

    fn finish(mut self) -> TypedBoundaryBirthEnumerationV1 {
        self.work.generated_logical_targets = self.logical_surfaces.len() as u64;
        let births = self.births.into_values().collect::<Vec<_>>();
        let mut hasher = Sha256::new();
        hasher.update(b"lay-boundary-birth-v1\0");
        for birth in &births {
            hash_len_bytes(&mut hasher, birth.normalized_surface.as_bytes());
            hasher.update([birth.relation as u8]);
            hasher.update(birth.operator_ref.to_le_bytes());
            hasher.update(birth.grounding_ref.to_le_bytes());
            hasher.update(birth.derivation_ref.to_le_bytes());
            hash_len_bytes(&mut hasher, &birth.composite_grounding.exact_bytes());
        }
        TypedBoundaryBirthEnumerationV1 {
            births,
            work: self.work,
            logical_match_count: self.logical_surfaces.len(),
            all_seen_digest: digest128(hasher.finalize().into()),
            overflow_reason: self
                .exhausted
                .then_some(IncompletenessReasonV1::WorkBudgetExceeded),
        }
    }
}

fn enumerate_splits(
    observed: &str,
    lexicon: &impl ExactContourLexiconV1,
    state: &mut BoundaryEnumerationStateV1,
) {
    let chars = observed.chars().collect::<Vec<_>>();
    for split_at in 1..chars.len() {
        let left = chars[..split_at].iter().collect::<String>();
        let right = chars[split_at..].iter().collect::<String>();
        let left_identities = state.exact_identities(lexicon, &left);
        if state.exhausted {
            return;
        }
        let right_identities = state.exact_identities(lexicon, &right);
        if state.exhausted {
            return;
        }
        for left_identity in &left_identities {
            for right_identity in &right_identities {
                state.observe(
                    format!("{left} {right}"),
                    TargetRelationV1::BoundarySplit,
                    OP_BOUNDARY_SPLIT,
                    CompositeBoundaryGroundingV1 {
                        ordered_part_groundings: vec![
                            exact_part(&left, *left_identity),
                            exact_part(&right, *right_identity),
                        ],
                        exact_segmentation_scalars: vec![split_at as u16],
                        separator_profile_id: ASCII_SPACE_SEPARATOR_PROFILE,
                        merged_target_grounding: None,
                    },
                );
                if state.exhausted {
                    return;
                }
            }
        }
    }
}

fn enumerate_merge(
    observed: &str,
    lexicon: &impl ExactContourLexiconV1,
    state: &mut BoundaryEnumerationStateV1,
) {
    if observed.matches(' ').count() != 1 {
        return;
    }
    let Some((left, right)) = observed.split_once(' ') else {
        return;
    };
    if left.is_empty()
        || right.is_empty()
        || !left.chars().all(char::is_alphabetic)
        || !right.chars().all(char::is_alphabetic)
    {
        return;
    }
    let left_identities = state.exact_identities(lexicon, left);
    let right_identities = state.exact_identities(lexicon, right);
    let merged = format!("{left}{right}");
    let merged_identities = state.exact_identities(lexicon, &merged);
    if state.exhausted {
        return;
    }
    for left_identity in &left_identities {
        for right_identity in &right_identities {
            for merged_identity in &merged_identities {
                state.observe(
                    merged.clone(),
                    TargetRelationV1::BoundaryMerge,
                    OP_BOUNDARY_MERGE,
                    CompositeBoundaryGroundingV1 {
                        ordered_part_groundings: vec![
                            exact_part(left, *left_identity),
                            exact_part(right, *right_identity),
                        ],
                        exact_segmentation_scalars: vec![
                            left.chars().count() as u16,
                            left.chars().count().saturating_add(1) as u16,
                        ],
                        separator_profile_id: ASCII_SPACE_SEPARATOR_PROFILE,
                        merged_target_grounding: Some(exact_part(&merged, *merged_identity)),
                    },
                );
                if state.exhausted {
                    return;
                }
            }
        }
    }
}

fn exact_part(surface: &str, identity: ExactContourIdentityV1) -> ExactBoundaryPartGroundingV1 {
    ExactBoundaryPartGroundingV1 {
        normalized_surface: surface.to_string(),
        grounding_namespace: identity.grounding_namespace,
        grounding_ref: identity.grounding_ref,
    }
}

fn boundary_derivation_ref(
    target: &str,
    relation: TargetRelationV1,
    operator_ref: u32,
    composite_bytes: &[u8],
) -> u32 {
    let mut bytes = b"lay-boundary-derivation-v1\0".to_vec();
    bytes.push(relation as u8);
    bytes.extend_from_slice(&operator_ref.to_le_bytes());
    hash_len_vec(&mut bytes, target.as_bytes());
    hash_len_vec(&mut bytes, composite_bytes);
    stable_bytes_ref(&bytes)
}

fn hash_len_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn hash_len_vec(output: &mut Vec<u8>, bytes: &[u8]) {
    output.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    output.extend_from_slice(bytes);
}

fn digest128(bytes: [u8; 32]) -> [u64; 2] {
    [
        u64::from_le_bytes(bytes[..8].try_into().expect("digest prefix")),
        u64::from_le_bytes(bytes[8..16].try_into().expect("digest suffix")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeLexicon(BTreeMap<String, u32>);

    impl FakeLexicon {
        fn with_forms(forms: &[&str]) -> Self {
            Self(
                forms
                    .iter()
                    .enumerate()
                    .map(|(index, surface)| (surface.to_string(), index as u32 + 1))
                    .collect(),
            )
        }
    }

    impl ExactContourLexiconV1 for FakeLexicon {
        fn exact_identities(&self, surface: &str) -> Vec<ExactContourIdentityV1> {
            self.0
                .get(surface)
                .copied()
                .map(|grounding_ref| {
                    vec![ExactContourIdentityV1 {
                        grounding_namespace: GroundingNamespaceV1::CanonicalForm,
                        grounding_ref,
                    }]
                })
                .unwrap_or_default()
        }
    }

    #[test]
    fn split_requires_exact_grounding_for_both_parts() {
        let exact = FakeLexicon::with_forms(&["елена", "просит"]);
        let missing_right = FakeLexicon::with_forms(&["елена"]);

        let result = enumerate_typed_boundary_births("еленапросит", &exact);
        assert_eq!(result.logical_match_count, 1);
        assert_eq!(result.births[0].normalized_surface, "елена просит");
        assert_eq!(result.births[0].relation, TargetRelationV1::BoundarySplit);
        assert_eq!(
            result.births[0]
                .composite_grounding
                .ordered_part_groundings
                .len(),
            2
        );
        assert!(
            enumerate_typed_boundary_births("еленапросит", &missing_right)
                .births
                .is_empty()
        );
    }

    #[test]
    fn merge_requires_source_parts_and_exact_merged_target() {
        let exact = FakeLexicon::with_forms(&["дан", "орм", "данорм"]);
        let no_merged = FakeLexicon::with_forms(&["дан", "орм"]);

        let result = enumerate_typed_boundary_births("дан орм", &exact);
        assert_eq!(result.logical_match_count, 1);
        assert_eq!(result.births[0].normalized_surface, "данорм");
        assert_eq!(result.births[0].relation, TargetRelationV1::BoundaryMerge);
        assert!(result.births[0]
            .composite_grounding
            .merged_target_grounding
            .is_some());
        assert!(enumerate_typed_boundary_births("дан орм", &no_merged)
            .births
            .is_empty());
    }

    #[test]
    fn split_enumeration_keeps_all_exact_segmentations_before_storage() {
        let lexicon = FakeLexicon::with_forms(&["а", "бвг", "аб", "вг", "абв", "г"]);
        let result = enumerate_typed_boundary_births("абвг", &lexicon);

        assert_eq!(result.logical_match_count, 3);
        assert_eq!(result.births.len(), 3);
        assert!(result.overflow_reason.is_none());
    }
}
