//! Bounded context-neutral contour birth for Slice 5 shadow proof.
//!
//! Canonical vocabulary membership establishes target identity only. Every
//! output remains `Born` until a separate grounding stage supplies evidence.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::nanda_wave::lexical_grokking::ExactL11SurfaceIndexV1;
use crate::typing_transition::target_evidence::{
    stable_bytes_ref, EnumerationWorkCountersV1, GroundingNamespaceV1, IncompletenessReasonV1,
    TargetRelationV1, VerdictMembershipV1,
};

use super::super::runtime::StandaloneL2Field;

pub(super) const MAX_CONTOUR_INPUT_SCALARS: usize = 32;
pub(super) const MAX_CONTOUR_BASES: usize = 3;
pub(super) const MAX_CONTOUR_EXACT_LOOKUPS: u64 = 16_384;
pub(super) const MAX_CONTOUR_OPERATOR_STEPS: u64 = 16_384;
pub(super) const CONTOUR_COMPOUND_EDIT_RADIUS: usize = 4;

const OP_EXACT_LAYOUT: u32 = 0x5335_0001;
const OP_IDENTITY_SINGLE_EDIT: u32 = 0x5335_0002;
const OP_LAYOUT_SINGLE_EDIT: u32 = 0x5335_0003;
const OP_IDENTITY_LOCAL_COMPOUND: u32 = 0x5335_0004;
const OP_LAYOUT_LOCAL_COMPOUND: u32 = 0x5335_0005;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BaseContourKindV1 {
    Identity,
    ExactLayout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BaseContourV1 {
    kind: BaseContourKindV1,
    surface: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TypedContourBirthV1 {
    pub(super) normalized_surface: String,
    pub(super) grounding_namespace: GroundingNamespaceV1,
    pub(super) grounding_ref: u32,
    pub(super) relation: TargetRelationV1,
    pub(super) operator_ref: u32,
    pub(super) derivation_ref: u32,
    pub(super) verdict_membership: VerdictMembershipV1,
    pub(super) support_milli: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TypedContourBirthEnumerationV1 {
    pub(super) births: Vec<TypedContourBirthV1>,
    pub(super) work: EnumerationWorkCountersV1,
    pub(super) logical_match_count: usize,
    pub(super) all_seen_digest: [u64; 2],
    pub(super) overflow_reason: Option<IncompletenessReasonV1>,
}

impl TypedContourBirthEnumerationV1 {
    pub(super) fn complete_empty() -> Self {
        Self {
            births: Vec::new(),
            work: EnumerationWorkCountersV1::default(),
            logical_match_count: 0,
            all_seen_digest: digest128(Sha256::digest(b"lay-contour-birth-v1\0").into()),
            overflow_reason: None,
        }
    }

    pub(super) fn work_within_budget(&self) -> bool {
        self.work.grounding_lookups <= MAX_CONTOUR_EXACT_LOOKUPS
            && self.work.operator_steps <= MAX_CONTOUR_OPERATOR_STEPS
            && self.overflow_reason.is_none()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ExactContourIdentityV1 {
    pub(super) grounding_namespace: GroundingNamespaceV1,
    pub(super) grounding_ref: u32,
}

pub(super) trait ExactContourLexiconV1 {
    fn exact_identities(&self, surface: &str) -> Vec<ExactContourIdentityV1>;
}

pub(super) struct ExactContourIdentityUnionV1<'a> {
    canonical: &'a StandaloneL2Field,
    l11: Option<&'a ExactL11SurfaceIndexV1>,
}

impl<'a> ExactContourIdentityUnionV1<'a> {
    pub(super) fn new(
        canonical: &'a StandaloneL2Field,
        l11: Option<&'a ExactL11SurfaceIndexV1>,
    ) -> Self {
        Self { canonical, l11 }
    }
}

impl ExactContourLexiconV1 for ExactContourIdentityUnionV1<'_> {
    fn exact_identities(&self, surface: &str) -> Vec<ExactContourIdentityV1> {
        let mut identities = Vec::with_capacity(3);
        if let Some(form_ref) = self.canonical.form_ref_for_surface(surface) {
            identities.push(ExactContourIdentityV1 {
                grounding_namespace: GroundingNamespaceV1::CanonicalForm,
                grounding_ref: form_ref,
            });
        }
        if let Some(terminal_id) = self
            .l11
            .and_then(|index| index.terminal_for_surface(surface))
        {
            identities.push(ExactContourIdentityV1 {
                grounding_namespace: GroundingNamespaceV1::L11Terminal,
                grounding_ref: terminal_id,
            });
        }
        if embedded_reference_surface(surface) {
            identities.push(ExactContourIdentityV1 {
                grounding_namespace: GroundingNamespaceV1::ReferenceSurface,
                grounding_ref: stable_bytes_ref(surface.as_bytes()),
            });
        }
        identities.sort_unstable();
        identities.dedup();
        identities
    }
}

pub(super) fn enumerate_typed_contour_births(
    observed: &str,
    canonical: &StandaloneL2Field,
) -> TypedContourBirthEnumerationV1 {
    enumerate_with_lexicon(observed, &ExactContourIdentityUnionV1::new(canonical, None))
}

pub(super) fn enumerate_typed_contour_births_with_l11(
    observed: &str,
    canonical: &StandaloneL2Field,
    l11: &ExactL11SurfaceIndexV1,
) -> TypedContourBirthEnumerationV1 {
    enumerate_with_lexicon(
        observed,
        &ExactContourIdentityUnionV1::new(canonical, Some(l11)),
    )
}

fn enumerate_with_lexicon(
    observed: &str,
    canonical: &impl ExactContourLexiconV1,
) -> TypedContourBirthEnumerationV1 {
    let normalized = observed.to_lowercase();
    let scalar_count = normalized.chars().count();
    if normalized.is_empty() {
        return TypedContourBirthEnumerationV1::complete_empty();
    }
    if scalar_count > MAX_CONTOUR_INPUT_SCALARS {
        let mut result = TypedContourBirthEnumerationV1::complete_empty();
        result.overflow_reason = Some(IncompletenessReasonV1::WorkBudgetExceeded);
        return result;
    }

    let mut state = EnumerationStateV1::default();
    for base in base_contours(&normalized) {
        if base.kind == BaseContourKindV1::ExactLayout {
            state.consider(
                canonical,
                &normalized,
                &base.surface,
                TargetRelationV1::ExactLayout,
                OP_EXACT_LAYOUT,
                &[],
            );
        }
        if state.exhausted() {
            break;
        }
        enumerate_single_edits(&normalized, &base, canonical, &mut state);
        if state.exhausted() {
            break;
        }
        enumerate_local_compound_edits(&normalized, &base, canonical, &mut state);
        if state.exhausted() {
            break;
        }
    }
    state.finish()
}

#[derive(Default)]
struct EnumerationStateV1 {
    births: BTreeMap<
        (
            String,
            TargetRelationV1,
            u32,
            u32,
            GroundingNamespaceV1,
            u32,
        ),
        TypedContourBirthV1,
    >,
    work: EnumerationWorkCountersV1,
    logical_match_count: usize,
    exhausted: bool,
}

impl EnumerationStateV1 {
    fn exhausted(&self) -> bool {
        self.exhausted
    }

    fn consider(
        &mut self,
        canonical: &impl ExactContourLexiconV1,
        observed: &str,
        candidate: &str,
        relation: TargetRelationV1,
        operator_ref: u32,
        geometry: &[u16],
    ) {
        if self.exhausted || candidate.is_empty() || candidate == observed {
            return;
        }
        if self.work.operator_steps >= MAX_CONTOUR_OPERATOR_STEPS
            || self.work.grounding_lookups >= MAX_CONTOUR_EXACT_LOOKUPS
        {
            self.exhausted = true;
            return;
        }
        self.work.operator_steps += 1;
        self.work.grounding_lookups += 1;
        let identities = canonical.exact_identities(candidate);
        if identities.is_empty() {
            return;
        }
        self.logical_match_count += 1;
        let derivation_ref = contour_derivation_ref(observed, candidate, operator_ref, geometry);
        for identity in identities {
            let birth = TypedContourBirthV1 {
                normalized_surface: candidate.to_string(),
                grounding_namespace: identity.grounding_namespace,
                grounding_ref: identity.grounding_ref,
                relation,
                operator_ref,
                derivation_ref,
                verdict_membership: VerdictMembershipV1::Born,
                support_milli: 0,
            };
            self.births
                .entry((
                    candidate.to_string(),
                    relation,
                    operator_ref,
                    derivation_ref,
                    identity.grounding_namespace,
                    identity.grounding_ref,
                ))
                .or_insert(birth);
        }
    }

    fn finish(self) -> TypedContourBirthEnumerationV1 {
        let births = self.births.into_values().collect::<Vec<_>>();
        let mut hasher = Sha256::new();
        hasher.update(b"lay-contour-birth-v1\0");
        for birth in &births {
            hash_len_bytes(&mut hasher, birth.normalized_surface.as_bytes());
            hasher.update([birth.grounding_namespace as u8]);
            hasher.update(birth.grounding_ref.to_le_bytes());
            hasher.update([birth.relation as u8]);
            hasher.update(birth.operator_ref.to_le_bytes());
            hasher.update(birth.derivation_ref.to_le_bytes());
            hasher.update([birth.verdict_membership as u8]);
            hasher.update(birth.support_milli.to_le_bytes());
        }
        TypedContourBirthEnumerationV1 {
            births,
            work: self.work,
            logical_match_count: self.logical_match_count,
            all_seen_digest: digest128(hasher.finalize().into()),
            overflow_reason: self
                .exhausted
                .then_some(IncompletenessReasonV1::WorkBudgetExceeded),
        }
    }
}

fn base_contours(observed: &str) -> Vec<BaseContourV1> {
    let mut bases = vec![BaseContourV1 {
        kind: BaseContourKindV1::Identity,
        surface: observed.to_string(),
    }];
    let all_ascii = observed.chars().all(|scalar| scalar.is_ascii_alphabetic());
    let all_cyrillic = observed.chars().all(crate::keyboard::is_cyrillic_letter);
    let has_ascii = observed.chars().any(|scalar| scalar.is_ascii_alphabetic());
    let has_cyrillic = observed.chars().any(crate::keyboard::is_cyrillic_letter);
    if all_ascii || (has_ascii && has_cyrillic) {
        push_base(
            &mut bases,
            crate::dict::convert(observed, crate::dict::Direction::Us2Ru).to_lowercase(),
        );
    }
    if all_cyrillic || (has_ascii && has_cyrillic) {
        push_base(
            &mut bases,
            crate::dict::convert(observed, crate::dict::Direction::Ru2Us).to_lowercase(),
        );
    }
    bases.truncate(MAX_CONTOUR_BASES);
    bases
}

fn push_base(bases: &mut Vec<BaseContourV1>, surface: String) {
    if surface.is_empty()
        || bases
            .iter()
            .any(|base| base.surface.eq_ignore_ascii_case(&surface))
    {
        return;
    }
    bases.push(BaseContourV1 {
        kind: BaseContourKindV1::ExactLayout,
        surface,
    });
}

fn enumerate_single_edits(
    observed: &str,
    base: &BaseContourV1,
    canonical: &impl ExactContourLexiconV1,
    state: &mut EnumerationStateV1,
) {
    let chars = base.surface.chars().collect::<Vec<_>>();
    let alphabet = contour_alphabet(&chars);
    let operator_ref = match base.kind {
        BaseContourKindV1::Identity => OP_IDENTITY_SINGLE_EDIT,
        BaseContourKindV1::ExactLayout => OP_LAYOUT_SINGLE_EDIT,
    };
    for remove_at in 0..chars.len() {
        state.consider(
            canonical,
            observed,
            &without_index(&chars, remove_at),
            relation_for_single_edit(base.kind, TargetRelationV1::ExtraLetter),
            operator_ref,
            &[remove_at as u16],
        );
        if state.exhausted() {
            return;
        }
    }
    for insert_at in 0..=chars.len() {
        for inserted in alphabet.chars() {
            state.consider(
                canonical,
                observed,
                &with_inserted(&chars, insert_at, inserted),
                relation_for_single_edit(base.kind, TargetRelationV1::MissingLetter),
                operator_ref,
                &[insert_at as u16, inserted as u16],
            );
            if state.exhausted() {
                return;
            }
        }
    }
    for replace_at in 0..chars.len() {
        for replacement in alphabet.chars().filter(|value| *value != chars[replace_at]) {
            let mut candidate = chars.clone();
            candidate[replace_at] = replacement;
            state.consider(
                canonical,
                observed,
                &candidate.into_iter().collect::<String>(),
                relation_for_single_edit(base.kind, TargetRelationV1::Substitution),
                operator_ref,
                &[replace_at as u16, replacement as u16],
            );
            if state.exhausted() {
                return;
            }
        }
    }
    for swap_at in 0..chars.len().saturating_sub(1) {
        if chars[swap_at] == chars[swap_at + 1] {
            continue;
        }
        let mut candidate = chars.clone();
        candidate.swap(swap_at, swap_at + 1);
        state.consider(
            canonical,
            observed,
            &candidate.into_iter().collect::<String>(),
            relation_for_single_edit(base.kind, TargetRelationV1::AdjacentTransposition),
            operator_ref,
            &[swap_at as u16],
        );
        if state.exhausted() {
            return;
        }
    }
}

fn enumerate_local_compound_edits(
    observed: &str,
    base: &BaseContourV1,
    canonical: &impl ExactContourLexiconV1,
    state: &mut EnumerationStateV1,
) {
    let chars = base.surface.chars().collect::<Vec<_>>();
    let alphabet = contour_alphabet(&chars);
    let operator_ref = match base.kind {
        BaseContourKindV1::Identity => OP_IDENTITY_LOCAL_COMPOUND,
        BaseContourKindV1::ExactLayout => OP_LAYOUT_LOCAL_COMPOUND,
    };
    let relation = match base.kind {
        BaseContourKindV1::Identity => TargetRelationV1::SparseOmission,
        BaseContourKindV1::ExactLayout => TargetRelationV1::LayoutThenTypo,
    };
    for remove_at in 0..chars.len() {
        let reduced = chars
            .iter()
            .enumerate()
            .filter_map(|(index, scalar)| (index != remove_at).then_some(*scalar))
            .collect::<Vec<_>>();
        if reduced.is_empty() {
            continue;
        }
        let start = remove_at.saturating_sub(CONTOUR_COMPOUND_EDIT_RADIUS);
        let end = remove_at
            .saturating_add(CONTOUR_COMPOUND_EDIT_RADIUS)
            .min(reduced.len() - 1);
        for replace_at in start..=end {
            for replacement in alphabet
                .chars()
                .filter(|value| *value != reduced[replace_at])
            {
                let mut candidate = reduced.clone();
                candidate[replace_at] = replacement;
                state.consider(
                    canonical,
                    observed,
                    &candidate.into_iter().collect::<String>(),
                    relation,
                    operator_ref,
                    &[remove_at as u16, replace_at as u16, replacement as u16],
                );
                if state.exhausted() {
                    return;
                }
            }
        }
    }
}

fn relation_for_single_edit(
    base: BaseContourKindV1,
    identity_relation: TargetRelationV1,
) -> TargetRelationV1 {
    match base {
        BaseContourKindV1::Identity => identity_relation,
        BaseContourKindV1::ExactLayout => TargetRelationV1::LayoutThenTypo,
    }
}

fn contour_alphabet(chars: &[char]) -> &'static str {
    if chars
        .iter()
        .all(|scalar| crate::keyboard::is_cyrillic_letter(*scalar))
    {
        "абвгдеёжзийклмнопрстуфхцчшщъыьэюя"
    } else if chars.iter().all(|scalar| scalar.is_ascii_alphabetic()) {
        "abcdefghijklmnopqrstuvwxyz"
    } else {
        ""
    }
}

fn embedded_reference_surface(surface: &str) -> bool {
    crate::lexicon::is_common_ru_word(surface)
        || crate::lexicon::is_common_en_technical_word(surface)
}

fn without_index(chars: &[char], remove_at: usize) -> String {
    chars
        .iter()
        .enumerate()
        .filter_map(|(index, scalar)| (index != remove_at).then_some(*scalar))
        .collect()
}

fn with_inserted(chars: &[char], insert_at: usize, inserted: char) -> String {
    let mut candidate = String::with_capacity(chars.len().saturating_add(1));
    for (index, scalar) in chars.iter().enumerate() {
        if index == insert_at {
            candidate.push(inserted);
        }
        candidate.push(*scalar);
    }
    if insert_at == chars.len() {
        candidate.push(inserted);
    }
    candidate
}

fn contour_derivation_ref(
    observed: &str,
    candidate: &str,
    operator_ref: u32,
    geometry: &[u16],
) -> u32 {
    let mut bytes = b"lay-contour-derivation-v1\0".to_vec();
    bytes.extend_from_slice(&operator_ref.to_le_bytes());
    hash_len_vec(&mut bytes, observed.as_bytes());
    hash_len_vec(&mut bytes, candidate.as_bytes());
    for value in geometry {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
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
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;
    use std::time::Instant;

    use super::*;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    #[derive(Default)]
    struct FakeLexicon {
        forms: BTreeMap<String, u32>,
    }

    impl FakeLexicon {
        fn with_forms(forms: &[&str]) -> Self {
            Self {
                forms: forms
                    .iter()
                    .enumerate()
                    .map(|(index, surface)| (surface.to_string(), index as u32 + 1))
                    .collect(),
            }
        }
    }

    impl ExactContourLexiconV1 for FakeLexicon {
        fn exact_identities(&self, surface: &str) -> Vec<ExactContourIdentityV1> {
            self.forms
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

    fn surfaces(result: &TypedContourBirthEnumerationV1) -> BTreeSet<&str> {
        result
            .births
            .iter()
            .map(|birth| birth.normalized_surface.as_str())
            .collect()
    }

    #[test]
    fn typed_contours_cover_layout_single_and_local_compound_geometry() {
        let lexicon = FakeLexicon::with_forms(&["не", "автозамена", "читай"]);

        assert!(surfaces(&enumerate_with_lexicon("yt", &lexicon)).contains("не"));
        assert!(surfaces(&enumerate_with_lexicon("dnjpfvtyf", &lexicon)).contains("автозамена"));
        assert!(surfaces(&enumerate_with_lexicon("fавтозамена", &lexicon)).contains("автозамена"));
        assert!(surfaces(&enumerate_with_lexicon("читайл", &lexicon)).contains("читай"));
        assert!(surfaces(&enumerate_with_lexicon("автозаменет", &lexicon)).contains("автозамена"));
    }

    #[test]
    fn enumeration_is_deterministic_and_never_uses_form_ref_order_as_a_frontier() {
        let left = FakeLexicon::with_forms(&["мало", "мыло", "мило", "мело"]);
        let right = FakeLexicon {
            forms: left
                .forms
                .keys()
                .rev()
                .enumerate()
                .map(|(index, surface)| (surface.clone(), 100 - index as u32))
                .collect(),
        };
        let left_result = enumerate_with_lexicon("мло", &left);
        let right_result = enumerate_with_lexicon("мло", &right);

        assert_eq!(surfaces(&left_result), surfaces(&right_result));
        assert_eq!(left_result.work, right_result.work);
    }

    #[test]
    fn input_budget_overflow_is_explicit_and_empty() {
        let result = enumerate_with_lexicon(
            &"а".repeat(MAX_CONTOUR_INPUT_SCALARS + 1),
            &FakeLexicon::default(),
        );

        assert!(result.births.is_empty());
        assert_eq!(
            result.overflow_reason,
            Some(IncompletenessReasonV1::WorkBudgetExceeded)
        );
        assert!(!result.work_within_budget());
    }

    #[derive(Deserialize)]
    struct V7AuditReceipt {
        packages: V7AuditPackages,
        live_cohort_compare_shadow: V7LiveCohort,
    }

    #[derive(Deserialize)]
    struct V7AuditPackages {
        canonical_l2: String,
    }

    #[derive(Deserialize)]
    struct V7LiveCohort {
        schema: String,
        no_field_cases: usize,
        no_field_records: Vec<V7NoFieldRecord>,
    }

    #[derive(Deserialize)]
    struct V7NoFieldRecord {
        availability: String,
        class: String,
        damaged_surface: String,
        proof_identity: [u8; 32],
        target_surface: String,
    }

    struct TargetBlindDiscovery {
        births: TypedContourBirthEnumerationV1,
        born_lemmas: BTreeSet<u32>,
        enumeration_us: u64,
        discovery_us: u64,
    }

    fn target_blind_discovery(
        damaged_surface: &str,
        canonical: &StandaloneL2Field,
    ) -> TargetBlindDiscovery {
        let discovery_started = Instant::now();
        let enumeration_started = Instant::now();
        let births = enumerate_typed_contour_births(damaged_surface, canonical);
        let enumeration_us = elapsed_us(enumeration_started);
        let born_lemmas = births
            .births
            .iter()
            .filter(|birth| birth.grounding_namespace == GroundingNamespaceV1::CanonicalForm)
            .flat_map(|birth| {
                canonical
                    .imported_binding_identities_for_form(birth.grounding_ref)
                    .into_iter()
                    .map(|(lemma_id, _)| lemma_id)
            })
            .collect::<BTreeSet<_>>();
        let discovery_us = elapsed_us(discovery_started);
        TargetBlindDiscovery {
            births,
            born_lemmas,
            enumeration_us,
            discovery_us,
        }
    }

    fn elapsed_us(started: Instant) -> u64 {
        started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
    }

    #[derive(Default, Serialize)]
    struct DiscoveryBucket {
        records: usize,
        exact_target: usize,
        same_lemma_proxy_only: usize,
        no_same_lemma: usize,
        target_form_missing: usize,
        target_lemma_missing: usize,
        overflow: usize,
        births_total: u64,
        births_maximum: usize,
        grounding_lookups_total: u64,
        grounding_lookups_maximum: u64,
        operator_steps_total: u64,
        operator_steps_maximum: u64,
    }

    impl DiscoveryBucket {
        fn record(
            &mut self,
            discovery: &TargetBlindDiscovery,
            exact_target: bool,
            same_lemma: bool,
            target_form_missing: bool,
            target_lemma_missing: bool,
        ) {
            self.records += 1;
            if exact_target {
                self.exact_target += 1;
            } else if same_lemma {
                self.same_lemma_proxy_only += 1;
            } else {
                self.no_same_lemma += 1;
            }
            self.target_form_missing += usize::from(target_form_missing);
            self.target_lemma_missing += usize::from(target_lemma_missing);
            self.overflow += usize::from(discovery.births.overflow_reason.is_some());
            self.births_total = self
                .births_total
                .saturating_add(discovery.births.births.len() as u64);
            self.births_maximum = self.births_maximum.max(discovery.births.births.len());
            self.grounding_lookups_total = self
                .grounding_lookups_total
                .saturating_add(discovery.births.work.grounding_lookups);
            self.grounding_lookups_maximum = self
                .grounding_lookups_maximum
                .max(discovery.births.work.grounding_lookups);
            self.operator_steps_total = self
                .operator_steps_total
                .saturating_add(discovery.births.work.operator_steps);
            self.operator_steps_maximum = self
                .operator_steps_maximum
                .max(discovery.births.work.operator_steps);
        }

        fn partition_conserved(&self) -> bool {
            self.exact_target
                .saturating_add(self.same_lemma_proxy_only)
                .saturating_add(self.no_same_lemma)
                == self.records
        }
    }

    fn percentile(values: &mut [u64], percent: usize) -> u64 {
        if values.is_empty() {
            return 0;
        }
        values.sort_unstable();
        let nearest_rank = values
            .len()
            .saturating_mul(percent)
            .saturating_add(99)
            .checked_div(100)
            .unwrap_or(1)
            .clamp(1, values.len());
        values[nearest_rank - 1]
    }

    #[test]
    #[ignore = "proof-only audit over the saved Slice 8B V7 NoField denominator"]
    fn slice8b_v8_target_blind_contour_lemma_audit() {
        let receipt_path = std::env::var("LAY_SLICE8B_V7_RECEIPT")
            .expect("LAY_SLICE8B_V7_RECEIPT must name the saved V7 receipt");
        let receipt_bytes = std::fs::read(&receipt_path).expect("read V7 receipt");
        let receipt_sha256 = Sha256::digest(&receipt_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let receipt: V7AuditReceipt =
            serde_json::from_slice(&receipt_bytes).expect("decode V7 receipt");
        assert_eq!(
            receipt.live_cohort_compare_shadow.schema,
            "lay.live-cohort-compare-shadow.v3"
        );
        assert_eq!(
            receipt.live_cohort_compare_shadow.no_field_records.len(),
            receipt.live_cohort_compare_shadow.no_field_cases
        );

        let canonical_path =
            std::env::var("LAY_SLICE8B_V13_PACKAGE").unwrap_or(receipt.packages.canonical_l2);
        let canonical = StandaloneL2Field::load(Path::new(&canonical_path))
            .expect("load canonical V13 package");
        let mut attempts = BTreeSet::new();
        let mut global = DiscoveryBucket::default();
        let mut by_availability = BTreeMap::<String, DiscoveryBucket>::new();
        let mut by_damage_class = BTreeMap::<String, DiscoveryBucket>::new();
        let mut enumeration_latency_us = Vec::new();
        let mut discovery_latency_us = Vec::new();

        for record in receipt.live_cohort_compare_shadow.no_field_records {
            assert!(attempts.insert((record.class.clone(), record.proof_identity)));

            // Target bytes are intentionally unavailable to this function.
            let discovery = target_blind_discovery(&record.damaged_surface, &canonical);
            enumeration_latency_us.push(discovery.enumeration_us);
            discovery_latency_us.push(discovery.discovery_us);

            // Labels enter only after the complete birth set and work record exist.
            let target_form_ref = canonical.form_ref_for_surface(&record.target_surface);
            let target_lemmas = target_form_ref
                .map(|form_ref| {
                    canonical
                        .imported_binding_identities_for_form(form_ref)
                        .into_iter()
                        .map(|(lemma_id, _)| lemma_id)
                        .collect::<BTreeSet<_>>()
                })
                .unwrap_or_default();
            let exact_target = discovery
                .births
                .births
                .iter()
                .any(|birth| birth.normalized_surface == record.target_surface);
            let same_lemma = !target_lemmas.is_disjoint(&discovery.born_lemmas);
            let target_form_missing = target_form_ref.is_none();
            let target_lemma_missing = target_lemmas.is_empty();

            global.record(
                &discovery,
                exact_target,
                same_lemma,
                target_form_missing,
                target_lemma_missing,
            );
            by_availability
                .entry(record.availability)
                .or_default()
                .record(
                    &discovery,
                    exact_target,
                    same_lemma,
                    target_form_missing,
                    target_lemma_missing,
                );
            by_damage_class.entry(record.class).or_default().record(
                &discovery,
                exact_target,
                same_lemma,
                target_form_missing,
                target_lemma_missing,
            );
        }

        assert_eq!(attempts.len(), global.records);
        assert!(global.partition_conserved());
        assert!(by_availability
            .values()
            .all(|bucket| bucket.partition_conserved()));
        assert!(by_damage_class
            .values()
            .all(|bucket| bucket.partition_conserved()));
        assert_eq!(global.target_form_missing, 0);
        let source_no_field_cases = receipt.live_cohort_compare_shadow.no_field_cases;
        assert_eq!(global.records, source_no_field_cases);

        let report = serde_json::json!({
            "schema": "lay.slice8b-v8-target-blind-contour-lemma-audit.v1",
            "source": {
                "v7_receipt": receipt_path,
                "v7_receipt_sha256": receipt_sha256,
                "v13_package": canonical_path,
                "v7_live_cohort_schema": "lay.live-cohort-compare-shadow.v3",
            },
            "measured": {
                "no_field_records": global.records,
                "unique_attempt_identities": attempts.len(),
                "global": &global,
                "by_availability": &by_availability,
                "by_damage_class": &by_damage_class,
                "latency_us": {
                    "scope": "closed target-blind V13 contour enumeration; no queue, Productive V90, L3 or IPC",
                    "enumeration": {
                        "p50": percentile(&mut enumeration_latency_us.clone(), 50),
                        "p95": percentile(&mut enumeration_latency_us.clone(), 95),
                        "p99": percentile(&mut enumeration_latency_us.clone(), 99),
                        "maximum": enumeration_latency_us.iter().copied().max().unwrap_or(0),
                    },
                    "enumeration_plus_form_to_lemma_mapping": {
                        "p50": percentile(&mut discovery_latency_us.clone(), 50),
                        "p95": percentile(&mut discovery_latency_us.clone(), 95),
                        "p99": percentile(&mut discovery_latency_us.clone(), 99),
                        "maximum": discovery_latency_us.iter().copied().max().unwrap_or(0),
                    },
                },
            },
            "conservation": {
                "records_equal_source_no_field": global.records == source_no_field_cases,
                "unique_attempts_equal_records": attempts.len() == global.records,
                "global_discovery_partition": global.partition_conserved(),
                "availability_partitions": by_availability.values().all(|bucket| bucket.partition_conserved()),
                "damage_class_partitions": by_damage_class.values().all(|bucket| bucket.partition_conserved()),
                "target_form_missing": global.target_form_missing,
            },
            "claim_boundary": {
                "target_is_generation_input": false,
                "runtime_authority_changed": false,
                "promotion_eligible": false,
                "product_latency_proven": false,
                "productive_readout_proven": false,
            },
            "verdict": "PASS_TARGET_BLIND_DISCOVERY_MEASUREMENT_ONLY",
        });
        println!(
            "LAY_SLICE8B_V8_AUDIT_JSON={}",
            serde_json::to_string(&report).expect("serialize V8 audit")
        );
    }
}
