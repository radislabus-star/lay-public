use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use rayon::prelude::*;

use crate::correction_core::{MorphologySlotEvidence, UnifiedCorrectionCandidate};

use super::compositional::{
    prepared_normalized_similarity_at_least_milli, prepared_normalized_similarity_milli,
    prepared_surface_atom_profile, prepared_surface_atom_similarity_milli, surface_scoring_profile,
    LemmaWaveIndex, RuntimeLemmaWaveIndex, SurfaceGeometryWorkspace,
};
use super::context::{context_mode, scene_wave};
use super::model::{
    L2FieldPackage, MorphBinding, SlotPhaseCenter, TieCalibration,
    COMPETITION_FLAG_EXPLICIT_NEIGHBOR, L2_PHASE_CELLS, NO_L1_TERMINAL,
};
use super::productive::{
    directional_evidence_margin, package_canonical_source, package_lemma,
    PreparedProductiveGeneration, ProductiveBirthStatus, ProductiveContextPairEvidence,
    ProductiveForm, ProductiveMorphologyIndex, ProductiveMorphologySource,
};
use super::runtime_storage::RuntimeL2Package;
use super::CANONICAL_L2_ATOM_RELATION_LIMIT;

const MAX_ACTIVE_LEMMAS: usize = 4;
const INHERITED_L1_ATTENUATION_MILLI: i32 = 240;

pub(crate) const CANONICAL_L2_SURFACE_SOURCE_ID: &str = "CanonicalL2FieldSurface";
pub(crate) const CANONICAL_L2_READOUT_SOURCE_ID: &str = "CanonicalL2FieldReadout";
pub(crate) const CANONICAL_L2_PRODUCTIVE_SOURCE_ID: &str = "CanonicalL2ProductiveSurface";

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct CanonicalL2FieldReadout {
    pub(crate) candidates: Vec<UnifiedCorrectionCandidate>,
    pub(crate) authority: L2FieldAuthority,
    pub(crate) availability: L2FieldAvailability,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum CanonicalFieldCacheDisposition {
    #[default]
    NotRequested,
    Produced,
    Waited,
    ReadyHit,
    Failed,
}

impl CanonicalFieldCacheDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::Produced => "produced",
            Self::Waited => "waited",
            Self::ReadyHit => "ready_hit",
            Self::Failed => "failed",
        }
    }

    pub(crate) const fn producer_count(self) -> u64 {
        match self {
            Self::Produced => 1,
            _ => 0,
        }
    }
}

/// Observation-only receipt for one canonical field request. It is returned
/// beside the semantic readout so Rayon execution cannot detach telemetry from
/// the caller that owns the input frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CanonicalFieldTelemetry {
    pub(crate) l11_us: u64,
    pub(crate) productive_v90_us: u64,
    pub(crate) total_us: u64,
    pub(crate) field_producer_count: u64,
    pub(crate) cache_disposition: CanonicalFieldCacheDisposition,
    pub(crate) field_generation: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ObservedCanonicalL2FieldReadout {
    pub(crate) readout: CanonicalL2FieldReadout,
    pub(crate) telemetry: CanonicalFieldTelemetry,
    pub(in crate::nanda_wave::l2_field) cohort_compare:
        Option<super::productive_v1::LexicalCohortCompareV1>,
}

impl CanonicalL2FieldReadout {
    pub(crate) fn new(
        candidates: Vec<UnifiedCorrectionCandidate>,
        authority: L2FieldAuthority,
    ) -> Self {
        Self {
            candidates,
            authority,
            availability: L2FieldAvailability::Ready,
        }
    }

    pub(crate) fn abstain(availability: L2FieldAvailability) -> Self {
        Self {
            candidates: Vec::new(),
            authority: L2FieldAuthority::Abstain,
            availability,
        }
    }

    pub(crate) fn unavailable(availability: L2FieldAvailability) -> Self {
        debug_assert!(availability.is_transient());
        Self {
            candidates: Vec::new(),
            authority: L2FieldAuthority::Unavailable,
            availability,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum L2FieldAvailability {
    Ready,
    UnsupportedInput,
    EmptyL11Lattice,
    #[default]
    L11ServiceUnavailable,
    CanonicalPackageUnavailable,
    ProductivePackageUnavailable,
    ProductiveReadoutError,
}

impl L2FieldAvailability {
    pub(crate) const fn is_transient(self) -> bool {
        matches!(
            self,
            Self::L11ServiceUnavailable
                | Self::CanonicalPackageUnavailable
                | Self::ProductivePackageUnavailable
                | Self::ProductiveReadoutError
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum L2FieldAuthority {
    #[default]
    Unavailable,
    Winner {
        surface: String,
    },
    Tied {
        surfaces: Vec<String>,
    },
    Abstain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct L2LexicalSeed {
    pub(crate) terminal_id: Option<u32>,
    pub(crate) surface: Option<String>,
    pub(crate) evidence_milli: i32,
    pub(crate) origin: L2LexicalSeedOrigin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum L2LexicalSeedOrigin {
    GroundedL11,
    CompositionalMorphology,
    InverseGeometry,
}

impl L2LexicalSeedOrigin {
    fn is_grounded_input(self) -> bool {
        matches!(self, Self::GroundedL11 | Self::CompositionalMorphology)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompositionalFormBirth {
    pub(crate) form_ref: u32,
    pub(crate) lemma_id: u32,
    pub(crate) evidence_milli: u16,
    pub(crate) geometry_evidence_milli: u16,
    pub(crate) atom_evidence_milli: u16,
    pub(crate) lemma_evidence_milli: u16,
    pub(crate) wave_distance: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompositionalLemmaBirth {
    pub(crate) lemma_id: u32,
    pub(crate) atom_evidence: u32,
    pub(crate) atom_evidence_milli: u16,
    pub(crate) wave_distance: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LexicalExactSourceObservationV1 {
    pub(crate) form_ref: u32,
    pub(crate) feature_mask: u32,
    pub(crate) support: u16,
    pub(crate) canonical_preference: u8,
    pub(crate) normalized_surface: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LexicalLemmaObservationV1 {
    pub(crate) lemma_id: u32,
    pub(crate) known_pos_domains: Vec<u16>,
    pub(crate) exact_source_forms: Vec<LexicalExactSourceObservationV1>,
    pub(crate) canonical_source_form_ref: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveL2FormBirth {
    pub(super) surface: String,
    pub(super) lemma_id: u32,
    pub(super) source_form_ref: u32,
    pub(super) source_feature_mask: u32,
    pub(super) target_feature_mask: u32,
    pub(super) geometry_evidence_milli: u16,
    pub(super) profile_evidence_milli: u16,
    pub(super) slot_evidence_milli: i32,
    pub(super) context_positive_support: u32,
    pub(super) context_unlabeled_alternative_support: u32,
    pub(super) context_posterior_milli: u16,
    pub(super) context_observed: bool,
    pub(super) context_pair_evidence: Vec<ProductiveL2ContextPairEvidence>,
    pub(super) joint_evidence_milli: u16,
    pub(super) positive_support: u32,
    pub(super) anti_support: u32,
    pub(super) family_specificity: u8,
    pub(super) lemma_atom_evidence_milli: u16,
    pub(super) lemma_wave_distance: u16,
    pub(super) exact_surface_form_ref: Option<u32>,
    pub(super) status: ProductiveBirthStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveL2ContextPairEvidence {
    pub(super) competitor_feature_mask: u32,
    pub(super) evidence: ProductiveContextPairEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ProductiveL2Readout {
    Winner { surface: String },
    Tied { surfaces: Vec<String> },
    Abstain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContextualLemmaRank {
    birth: CompositionalLemmaBirth,
    total_evidence_milli: i32,
    slot_evidence_milli: i32,
    neighbor_evidence_milli: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContextualFeatureRank {
    feature_mask: u32,
    total_evidence_milli: i32,
    surface_evidence_milli: u16,
    slot_evidence_milli: i32,
    neighbor_evidence_milli: i32,
    support: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CompositionalFormCandidate {
    form_ref: u32,
    lemma_id: u32,
    lemma_evidence_milli: u16,
    wave_distance: u16,
    geometry_evidence_milli: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PreparedCompositionalForm {
    form_ref: u32,
    geometry_evidence_milli: Option<u16>,
}

impl CompositionalFormBirth {
    pub(crate) fn rank_evidence(self) -> (u16, u16, u16, u16, std::cmp::Reverse<u16>) {
        (
            self.evidence_milli,
            self.lemma_evidence_milli,
            self.atom_evidence_milli,
            self.geometry_evidence_milli,
            std::cmp::Reverse(self.wave_distance),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct L2LocalCandidate {
    pub(crate) form_ref: u32,
    pub(crate) l1_terminal_id: Option<u32>,
    pub(crate) surface: String,
    pub(crate) l1_evidence_milli: i32,
    pub(crate) slot_phase_milli: i32,
    pub(crate) neighbor_pressure: i32,
    pub(crate) competition_pressure: i32,
    pub(crate) explicit_competition_pressure: i32,
    pub(crate) local_score: i32,
    pub(crate) lemma_ids: Vec<u32>,
    pub(crate) feature_masks: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum L2LocalVerdict {
    Winner { form_ref: u32 },
    Tied { form_refs: Vec<u32> },
    Abstain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StandaloneL2Readout {
    pub(crate) verdict: L2LocalVerdict,
    pub(crate) candidates: Vec<L2LocalCandidate>,
    pub(crate) context_mode_id: Option<u32>,
}

#[derive(Clone, Debug)]
pub(crate) struct StandaloneL2Field {
    package: RuntimeL2Package,
    lemma_wave_index: RuntimeLemmaWaveIndex,
    lemma_wave_source: &'static str,
    form_by_terminal: Vec<(u32, u32)>,
    binding_offsets_by_form: Vec<u32>,
    binding_indices_by_form: Vec<u32>,
    productive_source_by_lemma:
        std::sync::Arc<[std::sync::OnceLock<Option<(u16, ProductiveForm)>>]>,
    context_by_key: BTreeMap<u32, u32>,
    slot_centers_by_mode_feature: BTreeMap<(u32, u32), Vec<u32>>,
    neighbor_couplings_by_mode_lemma_feature: BTreeMap<(u32, u32, u32), Vec<u32>>,
}

impl StandaloneL2Field {
    pub(crate) fn load(path: &Path) -> Result<Self, String> {
        Self::from_runtime_package(RuntimeL2Package::load(path)?)
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        Self::from_owned_bytes(bytes.to_vec())
    }

    pub(crate) fn from_package(package: L2FieldPackage) -> Result<Self, String> {
        Self::from_runtime_package(RuntimeL2Package::from_reference(package))
    }

    fn from_owned_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        Self::from_runtime_package(RuntimeL2Package::from_bytes(bytes)?)
    }

    fn from_runtime_package(mut package: RuntimeL2Package) -> Result<Self, String> {
        let (lemma_wave_index, lemma_wave_source) = match package.take_lemma_wave_index() {
            Some(index) => {
                let source = if package.mmap_backed() {
                    "compact_v2_mmap_view"
                } else {
                    "compact_v2_owned_view"
                };
                (index, source)
            }
            None => (
                RuntimeLemmaWaveIndex::from_owned(LemmaWaveIndex::build(&package)?),
                "runtime_rebuilt",
            ),
        };
        let mut form_by_terminal = Vec::new();
        for form_ref in 0..package.form_count() {
            let form = package
                .form(form_ref)
                .ok_or_else(|| format!("missing L2 form record {form_ref}"))?;
            if form.l1_terminal_id == NO_L1_TERMINAL {
                continue;
            }
            form_by_terminal.push((form.l1_terminal_id, form_ref as u32));
        }
        form_by_terminal.sort_unstable();
        if let Some(duplicate) = form_by_terminal
            .windows(2)
            .find(|pair| pair[0].0 == pair[1].0)
        {
            return Err(format!(
                "duplicate L1.1 terminal ID {} in L2 package",
                duplicate[0].0
            ));
        }

        let mut binding_offsets_by_form = vec![0_u32; package.form_count() + 1];
        for binding_index in 0..package.binding_count() {
            let binding = package
                .binding(binding_index)
                .ok_or_else(|| format!("missing L2 morphology binding {binding_index}"))?;
            binding_offsets_by_form[binding.form_center_ref as usize + 1] += 1;
        }
        for index in 1..binding_offsets_by_form.len() {
            binding_offsets_by_form[index] =
                binding_offsets_by_form[index].saturating_add(binding_offsets_by_form[index - 1]);
        }
        let mut binding_indices_by_form = vec![0_u32; package.binding_count()];
        let mut next = binding_offsets_by_form[..package.form_count()].to_vec();
        for binding_index in 0..package.binding_count() {
            let binding = package
                .binding(binding_index)
                .ok_or_else(|| format!("missing L2 morphology binding {binding_index}"))?;
            let form_ref = binding.form_center_ref as usize;
            let output = next[form_ref] as usize;
            binding_indices_by_form[output] = binding_index as u32;
            next[form_ref] += 1;
        }
        let productive_source_by_lemma = (0..package.lemma_centers().len())
            .map(|_| std::sync::OnceLock::new())
            .collect::<Vec<_>>()
            .into();
        let context_by_key = package
            .context_modes()
            .iter()
            .enumerate()
            .map(|(index, mode)| (mode.stable_key, index as u32))
            .collect();
        let mut slot_centers_by_mode_feature = BTreeMap::<(u32, u32), Vec<u32>>::new();
        for (index, center) in package.slot_centers().iter().enumerate() {
            slot_centers_by_mode_feature
                .entry((center.context_mode_id, center.feature_mask))
                .or_default()
                .push(index as u32);
        }
        let mut neighbor_couplings_by_mode_lemma_feature =
            BTreeMap::<(u32, u32, u32), Vec<u32>>::new();
        for (index, coupling) in package.neighbor_couplings().iter().enumerate() {
            neighbor_couplings_by_mode_lemma_feature
                .entry((
                    coupling.context_mode_id,
                    coupling.target_lemma_id,
                    coupling.target_feature_mask,
                ))
                .or_default()
                .push(index as u32);
        }
        Ok(Self {
            package,
            lemma_wave_index,
            lemma_wave_source,
            form_by_terminal,
            binding_offsets_by_form,
            binding_indices_by_form,
            productive_source_by_lemma,
            context_by_key,
            slot_centers_by_mode_feature,
            neighbor_couplings_by_mode_lemma_feature,
        })
    }

    pub(crate) fn l1_package_fingerprint(&self) -> u64 {
        self.package.l1_package_fingerprint()
    }

    pub(crate) fn package_counts(&self) -> (usize, usize, usize, usize, usize, usize) {
        (
            self.package.form_count(),
            self.form_by_terminal.len(),
            self.package.lemma_centers().len(),
            self.package.binding_count(),
            self.package.competition_edges().len(),
            self.package.raw_decoder_bytes(),
        )
    }

    pub(crate) fn form_count(&self) -> usize {
        self.package.form_count()
    }

    pub(crate) fn package_storage(&self) -> (&'static str, usize) {
        (self.package.storage_kind(), self.package.backing_bytes())
    }

    pub(crate) fn package_mmap_backed(&self) -> bool {
        self.package.mmap_backed()
    }

    pub(crate) fn compositional_index_bytes(&self) -> usize {
        self.lemma_wave_index.owned_resident_bytes()
    }

    pub(crate) fn compositional_index_view_bytes(&self) -> usize {
        self.lemma_wave_index.backing_view_bytes()
    }

    pub(crate) fn compositional_index_source(&self) -> &'static str {
        self.lemma_wave_source
    }

    pub(crate) fn compositional_form_births(
        &self,
        observed_surface: &str,
        lemma_limit: usize,
        form_limit: usize,
    ) -> Vec<CompositionalFormBirth> {
        self.compositional_form_births_with_atom_relation_limit(
            observed_surface,
            lemma_limit,
            form_limit,
            CANONICAL_L2_ATOM_RELATION_LIMIT,
        )
    }

    pub(super) fn train_productive_morphology(
        &self,
        include_lemma: impl Fn(u32) -> bool,
        minimum_support: u32,
    ) -> Result<ProductiveMorphologyIndex, String> {
        ProductiveMorphologyIndex::train_from_package(&self.package, include_lemma, minimum_support)
    }

    pub(super) fn productive_form_births_from_lemmas<I: ProductiveMorphologySource + ?Sized>(
        &self,
        index: &I,
        context: &str,
        observed_surface: &str,
        lemma_births: &[CompositionalLemmaBirth],
        feature_limit: usize,
        form_limit: usize,
    ) -> Vec<ProductiveL2FormBirth> {
        self.productive_form_births_from_lemmas_impl(
            index,
            context,
            observed_surface,
            lemma_births,
            feature_limit,
            form_limit,
            true,
        )
    }

    pub(super) fn productive_form_births_from_lemmas_exact_masked<
        I: ProductiveMorphologySource + ?Sized,
    >(
        &self,
        index: &I,
        context: &str,
        observed_surface: &str,
        lemma_births: &[CompositionalLemmaBirth],
        feature_limit: usize,
        form_limit: usize,
    ) -> Vec<ProductiveL2FormBirth> {
        self.productive_form_births_from_lemmas_impl(
            index,
            context,
            observed_surface,
            lemma_births,
            feature_limit,
            form_limit,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn productive_form_births_from_lemmas_impl<I: ProductiveMorphologySource + ?Sized>(
        &self,
        index: &I,
        context: &str,
        observed_surface: &str,
        lemma_births: &[CompositionalLemmaBirth],
        feature_limit: usize,
        form_limit: usize,
        annotate_exact_surface: bool,
    ) -> Vec<ProductiveL2FormBirth> {
        if feature_limit == 0 || form_limit == 0 {
            return Vec::new();
        }
        let mode = context_mode(context);
        let context_mode_id = self.context_by_key.get(&mode.stable_key).copied();
        let wave = scene_wave(context);
        let observed_profile = surface_scoring_profile(observed_surface);
        let source_keys = lemma_births
            .iter()
            .filter_map(|lemma_birth| {
                self.productive_canonical_source(lemma_birth.lemma_id)
                    .map(|(primary_pos, source)| (*primary_pos, source.feature_mask))
            })
            .collect::<BTreeSet<_>>();
        let target_features_by_source = source_keys
            .into_iter()
            .map(|(primary_pos, source_feature_mask)| {
                let mut target_features = index
                    .target_features_vec(primary_pos, source_feature_mask)
                    .into_iter()
                    .filter(|target| *target != source_feature_mask)
                    .map(|target_feature_mask| {
                        let context_evidence =
                            index.context_slot_evidence_for(context, target_feature_mask);
                        let slot_features =
                            crate::nanda_wave::morphology_phase::contextual_slot_features(
                                target_feature_mask,
                            );
                        let slot_evidence_milli = context_mode_id
                            .and_then(|context_mode_id| {
                                self.slot_centers_by_mode_feature
                                    .get(&(context_mode_id, slot_features))
                            })
                            .into_iter()
                            .flatten()
                            .filter_map(|index| self.package.slot_centers().get(*index as usize))
                            .map(|center| slot_center_score(center, &wave))
                            .max()
                            .unwrap_or_default();
                        (target_feature_mask, context_evidence, slot_evidence_milli)
                    })
                    .collect::<Vec<_>>();
                let has_context_compatible_slot = target_features
                    .iter()
                    .any(|(_, evidence, _)| evidence.positive_support > 0);
                if has_context_compatible_slot {
                    target_features.retain(|(_, evidence, _)| evidence.positive_support > 0);
                }
                target_features.sort_unstable_by(|left, right| {
                    right.2.cmp(&left.2).then_with(|| left.0.cmp(&right.0))
                });
                target_features.truncate(feature_limit);
                ((primary_pos, source_feature_mask), target_features)
            })
            .collect::<HashMap<_, _>>();
        let candidates = lemma_births
            .par_iter()
            .map_init(
                || {
                    (
                        HashMap::<String, u16>::new(),
                        SurfaceGeometryWorkspace::default(),
                    )
                },
                |(geometry_by_surface, geometry_workspace), lemma_birth| {
                    let Some((primary_pos, source)) =
                        self.productive_canonical_source(lemma_birth.lemma_id)
                    else {
                        return Vec::new();
                    };
                    let source_chars = source.surface.chars().collect::<Vec<_>>();
                    let family_suffixes =
                        super::productive::productive_family_suffixes(&source_chars);
                    let family_lane_starts = super::productive::productive_family_lane_starts(
                        &source.surface,
                        observed_surface,
                    );
                    let prepared = PreparedProductiveGeneration {
                        observed_surface,
                        observed_profile: &observed_profile,
                        source_surface: &source.surface,
                        source_chars: &source_chars,
                        family_suffixes: &family_suffixes,
                        family_lane_starts: &family_lane_starts,
                    };
                    let Some(target_features) =
                        target_features_by_source.get(&(*primary_pos, source.feature_mask))
                    else {
                        return Vec::new();
                    };
                    let mut candidates = Vec::new();
                    for &(target_feature_mask, context_evidence, slot_evidence_milli) in
                        target_features.iter()
                    {
                        for birth in index.generate_forms_prepared(
                            &prepared,
                            *primary_pos,
                            source.feature_mask,
                            target_feature_mask,
                            form_limit,
                            geometry_by_surface,
                            geometry_workspace,
                        ) {
                            let joint_evidence_milli = productive_joint_evidence_milli(
                                lemma_birth.atom_evidence_milli,
                                birth.profile_evidence_milli,
                                birth.geometry_evidence_milli,
                            );
                            candidates.push(ProductiveL2FormBirth {
                                exact_surface_form_ref: annotate_exact_surface
                                    .then(|| self.form_ref_for_surface(&birth.surface))
                                    .flatten(),
                                surface: birth.surface,
                                lemma_id: lemma_birth.lemma_id,
                                source_form_ref: source.form_ref,
                                source_feature_mask: birth.source_feature_mask,
                                target_feature_mask: birth.target_feature_mask,
                                geometry_evidence_milli: birth.geometry_evidence_milli,
                                profile_evidence_milli: birth.profile_evidence_milli,
                                slot_evidence_milli,
                                context_positive_support: context_evidence.positive_support,
                                context_unlabeled_alternative_support: context_evidence
                                    .unlabeled_alternative_support,
                                context_posterior_milli: context_evidence.posterior_milli,
                                context_observed: context_evidence.context_observed,
                                context_pair_evidence: Vec::new(),
                                joint_evidence_milli,
                                positive_support: birth.positive_support,
                                anti_support: birth.anti_support,
                                family_specificity: birth.family_specificity,
                                lemma_atom_evidence_milli: lemma_birth.atom_evidence_milli,
                                lemma_wave_distance: lemma_birth.wave_distance,
                                status: birth.status,
                            });
                        }
                    }
                    candidates
                },
            )
            .flatten()
            .collect::<Vec<_>>();
        let mut by_surface = HashMap::<(u32, String), ProductiveL2FormBirth>::new();
        for candidate in candidates {
            by_surface
                .entry((candidate.lemma_id, candidate.surface.clone()))
                .and_modify(|current| {
                    if productive_l2_birth_rank(&candidate) > productive_l2_birth_rank(current) {
                        *current = candidate.clone();
                    }
                })
                .or_insert(candidate);
        }
        let mut births = by_surface.into_values().collect::<Vec<_>>();
        births.sort_by(|left, right| {
            productive_l2_birth_rank(right)
                .cmp(&productive_l2_birth_rank(left))
                .then_with(|| left.surface.cmp(&right.surface))
                .then_with(|| left.lemma_id.cmp(&right.lemma_id))
        });
        births.truncate(form_limit);
        attach_productive_context_pair_evidence(index, context, &mut births);
        births
    }

    pub(super) fn productive_lemma(
        &self,
        lemma_id: u32,
    ) -> Result<super::productive::ProductiveLemma, String> {
        package_lemma(&self.package, lemma_id)
    }

    fn productive_canonical_source(&self, lemma_id: u32) -> Option<&(u16, ProductiveForm)> {
        self.productive_source_by_lemma
            .get(lemma_id as usize)?
            .get_or_init(|| package_canonical_source(&self.package, lemma_id).ok())
            .as_ref()
    }

    pub(super) fn lemma_count(&self) -> usize {
        self.package.lemma_centers().len()
    }

    pub(super) fn imported_binding_pairs_for_lemma(&self, lemma_ref: u32) -> Vec<(u32, u32)> {
        self.bindings_for_lemma(lemma_ref)
            .map(|binding| (binding.form_center_ref, binding.feature_mask))
            .collect()
    }

    pub(super) fn imported_binding_identities_for_form(&self, form_ref: u32) -> Vec<(u32, u32)> {
        self.bindings_for_form(form_ref)
            .map(|binding| (binding.lemma_center_id, binding.feature_mask))
            .collect()
    }

    pub(super) fn imported_surface_for_form(&self, form_ref: u32) -> Option<String> {
        self.package
            .surface(form_ref as usize)
            .map(|surface| surface.into_owned())
    }

    pub(crate) fn lexical_lemma_observation_v1(
        &self,
        lemma_id: u32,
    ) -> Result<Option<LexicalLemmaObservationV1>, String> {
        let Some(center) = self.package.lemma_centers().get(lemma_id as usize).copied() else {
            return Ok(None);
        };
        let mut exact_source_forms = self
            .bindings_for_lemma(lemma_id)
            .map(|binding| {
                let normalized_surface = self
                    .package
                    .surface(binding.form_center_ref as usize)
                    .ok_or_else(|| {
                        format!(
                            "canonical L2 lexical observation lacks form {}",
                            binding.form_center_ref
                        )
                    })?
                    .into_owned();
                Ok(LexicalExactSourceObservationV1 {
                    form_ref: binding.form_center_ref,
                    feature_mask: binding.feature_mask,
                    support: binding.support,
                    canonical_preference:
                        crate::nanda_wave::morphology_phase::productive_source_priority(
                            binding.feature_mask,
                        ),
                    normalized_surface,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        exact_source_forms.sort_by(|left, right| {
            (
                left.canonical_preference,
                left.normalized_surface.chars().count(),
                left.feature_mask,
                &left.normalized_surface,
                left.form_ref,
            )
                .cmp(&(
                    right.canonical_preference,
                    right.normalized_surface.chars().count(),
                    right.feature_mask,
                    &right.normalized_surface,
                    right.form_ref,
                ))
        });
        exact_source_forms.dedup_by(|duplicate, retained| {
            if (duplicate.form_ref, duplicate.feature_mask)
                != (retained.form_ref, retained.feature_mask)
            {
                return false;
            }
            retained.support = retained.support.max(duplicate.support);
            true
        });
        let canonical_source_form_ref = exact_source_forms.first().map(|source| source.form_ref);
        let mut known_pos_domains = exact_source_forms
            .iter()
            .map(|source| {
                crate::nanda_wave::morphology_phase::feature_primary_pos(source.feature_mask)
            })
            .filter(|pos| *pos != 0)
            .collect::<Vec<_>>();
        known_pos_domains.push(center.primary_pos);
        known_pos_domains.sort_unstable();
        known_pos_domains.dedup();
        Ok(Some(LexicalLemmaObservationV1 {
            lemma_id,
            known_pos_domains,
            exact_source_forms,
            canonical_source_form_ref,
        }))
    }

    pub(crate) fn compositional_form_births_with_atom_relation_limit(
        &self,
        observed_surface: &str,
        lemma_limit: usize,
        form_limit: usize,
        atom_relation_limit: usize,
    ) -> Vec<CompositionalFormBirth> {
        if lemma_limit == 0 || form_limit == 0 {
            return Vec::new();
        }
        let lemma_births = self.compositional_lemma_births_with_atom_relation_limit(
            observed_surface,
            lemma_limit,
            atom_relation_limit,
        );
        self.compositional_form_births_from_lemmas(observed_surface, &lemma_births, form_limit)
    }

    pub(crate) fn compositional_lemma_births_with_atom_relation_limit(
        &self,
        observed_surface: &str,
        lemma_limit: usize,
        atom_relation_limit: usize,
    ) -> Vec<CompositionalLemmaBirth> {
        self.lemma_wave_index
            .rank_lemmas_with_atom_relation_limit(
                observed_surface,
                lemma_limit,
                atom_relation_limit,
            )
            .into_iter()
            .map(|lemma_match| CompositionalLemmaBirth {
                lemma_id: lemma_match.lemma_id,
                atom_evidence: lemma_match.atom_evidence,
                atom_evidence_milli: lemma_match.atom_evidence_milli,
                wave_distance: lemma_match.wave_distance,
            })
            .collect()
    }

    pub(crate) fn compositional_lemma_births(
        &self,
        observed_surface: &str,
        lemma_limit: usize,
    ) -> Vec<CompositionalLemmaBirth> {
        self.compositional_lemma_births_with_atom_relation_limit(
            observed_surface,
            lemma_limit,
            CANONICAL_L2_ATOM_RELATION_LIMIT,
        )
    }

    pub(crate) fn contextual_compositional_lemma_births(
        &self,
        context: &str,
        lemma_births: &[CompositionalLemmaBirth],
        active_limit: usize,
    ) -> Vec<CompositionalLemmaBirth> {
        if active_limit == 0 || lemma_births.is_empty() {
            return Vec::new();
        }
        if active_limit >= lemma_births.len() {
            return lemma_births.to_vec();
        }
        let mode = context_mode(context);
        let Some(context_mode_id) = self.context_by_key.get(&mode.stable_key).copied() else {
            return lemma_births.iter().copied().take(active_limit).collect();
        };
        let wave = scene_wave(context);
        let mut ranked = lemma_births
            .iter()
            .copied()
            .map(|birth| {
                let slot_evidence_milli =
                    self.best_lemma_context_evidence(birth.lemma_id, Some(context_mode_id), &wave);
                let neighbor_evidence_milli =
                    self.best_lemma_neighbor_evidence(birth.lemma_id, context_mode_id);
                ContextualLemmaRank {
                    birth,
                    total_evidence_milli: i32::from(birth.atom_evidence_milli)
                        .saturating_add(slot_evidence_milli)
                        .saturating_add(neighbor_evidence_milli),
                    slot_evidence_milli,
                    neighbor_evidence_milli,
                }
            })
            .collect::<Vec<_>>();
        ranked.sort_unstable_by(|left, right| {
            right
                .total_evidence_milli
                .cmp(&left.total_evidence_milli)
                .then_with(|| {
                    right
                        .neighbor_evidence_milli
                        .cmp(&left.neighbor_evidence_milli)
                })
                .then_with(|| right.slot_evidence_milli.cmp(&left.slot_evidence_milli))
                .then_with(|| {
                    right
                        .birth
                        .atom_evidence_milli
                        .cmp(&left.birth.atom_evidence_milli)
                })
                .then_with(|| right.birth.atom_evidence.cmp(&left.birth.atom_evidence))
                .then_with(|| left.birth.wave_distance.cmp(&right.birth.wave_distance))
                .then_with(|| left.birth.lemma_id.cmp(&right.birth.lemma_id))
        });
        ranked.truncate(active_limit);
        ranked.into_iter().map(|ranked| ranked.birth).collect()
    }

    pub(crate) fn compositional_form_births_from_lemmas(
        &self,
        observed_surface: &str,
        lemma_births: &[CompositionalLemmaBirth],
        form_limit: usize,
    ) -> Vec<CompositionalFormBirth> {
        if form_limit == 0 {
            return Vec::new();
        }
        let lemma_forms = lemma_births
            .iter()
            .copied()
            .map(|lemma_match| {
                let mut form_refs = self
                    .bindings_for_lemma(lemma_match.lemma_id)
                    .map(|binding| PreparedCompositionalForm {
                        form_ref: binding.form_center_ref,
                        geometry_evidence_milli: None,
                    })
                    .collect::<Vec<_>>();
                form_refs.sort_unstable_by_key(|form| form.form_ref);
                form_refs.dedup_by_key(|form| form.form_ref);
                (lemma_match, form_refs)
            })
            .collect::<Vec<_>>();
        self.compositional_form_births_from_lemma_forms(observed_surface, lemma_forms, form_limit)
    }

    pub(crate) fn contextual_compositional_form_births_from_lemmas(
        &self,
        context: &str,
        observed_surface: &str,
        lemma_births: &[CompositionalLemmaBirth],
        feature_limit: usize,
        form_limit: usize,
    ) -> Vec<CompositionalFormBirth> {
        if feature_limit == 0 || form_limit == 0 {
            return Vec::new();
        }
        let trace = std::env::var_os("LAY_L2_FIELD_TRACE").is_some();
        let started = std::time::Instant::now();
        let mode = context_mode(context);
        let context_mode_id = self.context_by_key.get(&mode.stable_key).copied();
        let wave = scene_wave(context);
        let observed_profile = surface_scoring_profile(observed_surface);
        let lemma_forms = lemma_births
            .par_iter()
            .copied()
            .map(|lemma_match| {
                let bindings = self
                    .bindings_for_lemma(lemma_match.lemma_id)
                    .collect::<Vec<_>>();
                let mut by_feature = BTreeMap::<u32, ContextualFeatureRank>::new();
                let mut geometry_by_form = BTreeMap::<u32, u16>::new();
                for binding in &bindings {
                    let slot_evidence_milli =
                        self.binding_slot_context_evidence(*binding, context_mode_id, &wave);
                    let neighbor_evidence_milli =
                        self.binding_neighbor_context_evidence(*binding, context_mode_id);
                    let surface_evidence_milli = *geometry_by_form
                        .entry(binding.form_center_ref)
                        .or_insert_with(|| {
                            self.decode_form_ref(binding.form_center_ref)
                                .map(|surface| surface_scoring_profile(&surface))
                                .map(|expected_profile| {
                                    prepared_normalized_similarity_milli(
                                        &observed_profile,
                                        &expected_profile,
                                    )
                                })
                                .unwrap_or_default()
                        });
                    let rank = ContextualFeatureRank {
                        feature_mask: binding.feature_mask,
                        total_evidence_milli: slot_evidence_milli
                            .saturating_add(neighbor_evidence_milli),
                        surface_evidence_milli,
                        slot_evidence_milli,
                        neighbor_evidence_milli,
                        support: binding.support,
                    };
                    by_feature
                        .entry(binding.feature_mask)
                        .and_modify(|existing| {
                            if contextual_feature_rank(rank) > contextual_feature_rank(*existing) {
                                *existing = rank;
                            }
                        })
                        .or_insert(rank);
                }
                let mut ranked_features = by_feature.into_values().collect::<Vec<_>>();
                ranked_features.sort_unstable_by(|left, right| {
                    contextual_feature_rank(*right)
                        .cmp(&contextual_feature_rank(*left))
                        .then_with(|| left.feature_mask.cmp(&right.feature_mask))
                });
                ranked_features.truncate(feature_limit);
                let selected_features = ranked_features
                    .into_iter()
                    .map(|rank| rank.feature_mask)
                    .collect::<BTreeSet<_>>();
                let mut form_refs = bindings
                    .into_iter()
                    .filter(|binding| selected_features.contains(&binding.feature_mask))
                    .map(|binding| binding.form_center_ref)
                    .collect::<Vec<_>>();
                form_refs.sort_unstable();
                form_refs.dedup();
                let form_refs = form_refs
                    .into_iter()
                    .map(|form_ref| PreparedCompositionalForm {
                        form_ref,
                        geometry_evidence_milli: geometry_by_form.get(&form_ref).copied(),
                    })
                    .collect::<Vec<_>>();
                (lemma_match, form_refs)
            })
            .collect::<Vec<_>>();
        let selected = std::time::Instant::now();
        if trace {
            eprintln!(
                "l2_form_feature_trace lemmas={} selected_forms={} select_us={}",
                lemma_forms.len(),
                lemma_forms
                    .iter()
                    .map(|(_, form_refs)| form_refs.len())
                    .sum::<usize>(),
                selected.duration_since(started).as_micros(),
            );
        }
        self.compositional_form_births_from_lemma_forms(observed_surface, lemma_forms, form_limit)
    }

    fn compositional_form_births_from_lemma_forms(
        &self,
        observed_surface: &str,
        lemma_forms: Vec<(CompositionalLemmaBirth, Vec<PreparedCompositionalForm>)>,
        form_limit: usize,
    ) -> Vec<CompositionalFormBirth> {
        let trace = std::env::var_os("LAY_L2_FIELD_TRACE").is_some();
        let started = std::time::Instant::now();
        let mut by_form = BTreeMap::<u32, CompositionalFormCandidate>::new();
        let observed_profile = surface_scoring_profile(observed_surface);
        let normalized_observed = observed_profile.normalized();
        let exact_form_ref = self
            .form_ref_for_surface(observed_surface)
            .or_else(|| self.form_ref_for_surface(normalized_observed));
        if let Some(form_ref) = exact_form_ref {
            if let Some(lemma_id) = self
                .bindings_for_form(form_ref)
                .map(|binding| binding.lemma_center_id)
                .min()
            {
                by_form.insert(
                    form_ref,
                    CompositionalFormCandidate {
                        form_ref,
                        lemma_id,
                        lemma_evidence_milli: 1_000,
                        wave_distance: 0,
                        geometry_evidence_milli: Some(1_000),
                    },
                );
            }
        }
        for (lemma_match, form_refs) in lemma_forms {
            for form in form_refs {
                let candidate = CompositionalFormCandidate {
                    form_ref: form.form_ref,
                    lemma_id: lemma_match.lemma_id,
                    lemma_evidence_milli: lemma_match.atom_evidence_milli,
                    wave_distance: lemma_match.wave_distance,
                    geometry_evidence_milli: form.geometry_evidence_milli,
                };
                by_form
                    .entry(form.form_ref)
                    .and_modify(|existing| {
                        if compositional_candidate_rank(candidate)
                            > compositional_candidate_rank(*existing)
                        {
                            *existing = candidate;
                        }
                    })
                    .or_insert(candidate);
            }
        }
        let merged = std::time::Instant::now();
        let merged_form_count = by_form.len();
        let mut prefix_frontier =
            std::collections::BinaryHeap::<std::cmp::Reverse<(u16, u16)>>::with_capacity(
                form_limit.saturating_add(1),
            );
        let mut births = Vec::with_capacity(form_limit.saturating_mul(4));
        for candidate in by_form.into_values() {
            let minimum_geometry = if prefix_frontier.len() < form_limit {
                1
            } else {
                let (geometry, lemma) = prefix_frontier
                    .peek()
                    .map(|rank| rank.0)
                    .unwrap_or_default();
                if candidate.lemma_evidence_milli >= lemma {
                    geometry.max(1)
                } else {
                    geometry.saturating_add(1).max(1)
                }
            };
            let geometry_evidence_milli = match candidate.geometry_evidence_milli {
                Some(evidence) => evidence,
                None => {
                    let Some(surface) = self.decode_form_ref(candidate.form_ref) else {
                        continue;
                    };
                    let expected_profile = surface_scoring_profile(&surface);
                    prepared_normalized_similarity_at_least_milli(
                        &observed_profile,
                        &expected_profile,
                        minimum_geometry,
                    )
                }
            };
            if geometry_evidence_milli == 0 {
                continue;
            }
            let prefix = (geometry_evidence_milli, candidate.lemma_evidence_milli);
            if prefix_frontier.len() < form_limit {
                prefix_frontier.push(std::cmp::Reverse(prefix));
            } else if prefix_frontier
                .peek()
                .is_some_and(|minimum| prefix > minimum.0)
            {
                prefix_frontier.pop();
                prefix_frontier.push(std::cmp::Reverse(prefix));
            }
            births.push(CompositionalFormBirth {
                form_ref: candidate.form_ref,
                lemma_id: candidate.lemma_id,
                evidence_milli: geometry_evidence_milli,
                geometry_evidence_milli,
                atom_evidence_milli: 0,
                lemma_evidence_milli: candidate.lemma_evidence_milli,
                wave_distance: candidate.wave_distance,
            });
        }
        let geometry_ready = std::time::Instant::now();
        let atom_cutoff = if births.len() <= form_limit {
            Some((0_u16, 0_u16))
        } else {
            let mut prefixes = births
                .iter()
                .copied()
                .map(compositional_form_rank_prefix)
                .collect::<Vec<_>>();
            prefixes.sort_unstable_by(|left, right| right.cmp(left));
            prefixes.get(form_limit - 1).copied()
        };
        if let Some(atom_cutoff) = atom_cutoff {
            let observed_atom_profile = prepared_surface_atom_profile(&observed_profile);
            for birth in &mut births {
                if compositional_form_rank_prefix(*birth) < atom_cutoff {
                    continue;
                }
                let Some(surface) = self.decode_form_ref(birth.form_ref) else {
                    continue;
                };
                let expected_profile = surface_scoring_profile(&surface);
                let expected_atom_profile = prepared_surface_atom_profile(&expected_profile);
                birth.atom_evidence_milli = prepared_surface_atom_similarity_milli(
                    &observed_atom_profile,
                    &expected_atom_profile,
                );
            }
        }
        let atom_ready = std::time::Instant::now();
        births.sort_by(|left, right| {
            right
                .rank_evidence()
                .cmp(&left.rank_evidence())
                .then_with(|| left.lemma_id.cmp(&right.lemma_id))
                .then_with(|| left.form_ref.cmp(&right.form_ref))
        });
        births.truncate(form_limit);
        if trace {
            let finished = std::time::Instant::now();
            eprintln!(
                "l2_form_reduce_trace merged_forms={} retained_forms={} merge_us={} geometry_us={} atom_us={} sort_us={}",
                merged_form_count,
                births.len(),
                merged.duration_since(started).as_micros(),
                geometry_ready.duration_since(merged).as_micros(),
                atom_ready.duration_since(geometry_ready).as_micros(),
                finished.duration_since(atom_ready).as_micros(),
            );
        }
        births
    }

    pub(crate) fn bound_form_refs(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (0..self.package.form_count()).filter_map(|form_ref| {
            let form = self.package.form(form_ref)?;
            (form.l1_terminal_id != NO_L1_TERMINAL)
                .then_some((u32::try_from(form_ref).ok()?, form.l1_terminal_id))
        })
    }

    pub(crate) fn form_ref_for_surface(&self, surface: &str) -> Option<u32> {
        self.package.form_ref_for_surface(surface)
    }

    pub(crate) fn decode_form_ref(&self, form_ref: u32) -> Option<std::borrow::Cow<'_, str>> {
        self.package.surface(form_ref as usize)
    }

    pub(super) fn exact_morphology_slot_evidence<I: ProductiveMorphologySource + ?Sized>(
        &self,
        index: &I,
        context: &str,
        surface: &str,
    ) -> Vec<MorphologySlotEvidence> {
        let Some(form_ref) = self.form_ref_for_surface(surface) else {
            return Vec::new();
        };
        let mode = context_mode(context);
        let context_mode_id = self.context_by_key.get(&mode.stable_key).copied();
        let wave = scene_wave(context);
        let mut evidence = self
            .bindings_for_form(form_ref)
            .filter_map(|binding| {
                let context_evidence =
                    index.context_slot_evidence_for(context, binding.feature_mask);
                Some(MorphologySlotEvidence {
                    lemma_id: binding.lemma_center_id,
                    // An exact package form is already grounded in this slot. It does not
                    // need the productive generator's canonical source form, and decoding
                    // the complete lemma here would put an unbounded loop on the IME path.
                    source_feature_mask:
                        crate::nanda_wave::morphology_phase::productive_context_slot_features(
                            binding.feature_mask,
                        ),
                    target_feature_mask:
                        crate::nanda_wave::morphology_phase::productive_context_slot_features(
                            binding.feature_mask,
                        ),
                    context_positive_support: context_evidence.positive_support,
                    context_alternative_support: context_evidence.unlabeled_alternative_support,
                    context_posterior_milli: context_evidence.posterior_milli,
                    slot_evidence_milli: self.binding_slot_context_evidence(
                        binding,
                        context_mode_id,
                        &wave,
                    ),
                    joint_evidence_milli: 0,
                    generated: false,
                })
            })
            .collect::<Vec<_>>();
        evidence.sort_unstable_by_key(|item| {
            (
                item.lemma_id,
                item.source_feature_mask,
                item.target_feature_mask,
            )
        });
        evidence.dedup_by_key(|item| {
            (
                item.lemma_id,
                item.source_feature_mask,
                item.target_feature_mask,
            )
        });
        evidence
    }

    pub(crate) fn lemma_ids_for_form_feature(&self, form_ref: u32, feature_mask: u32) -> Vec<u32> {
        self.bindings_for_form(form_ref)
            .filter(|binding| binding.feature_mask == feature_mask)
            .map(|binding| binding.lemma_center_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(crate) fn context_mode_known(&self, context: &str) -> bool {
        self.context_by_key
            .contains_key(&context_mode(context).stable_key)
    }

    pub(crate) fn single_edit_form_refs(&self, surface: &str, limit: usize) -> Vec<u32> {
        if limit == 0 || self.form_ref_for_surface(surface).is_some() {
            return Vec::new();
        }
        let chars = surface.chars().collect::<Vec<_>>();
        if chars.len() < 2 {
            return Vec::new();
        }

        let mut found = BTreeSet::new();
        for remove_at in 0..chars.len() {
            let candidate = chars
                .iter()
                .enumerate()
                .filter_map(|(index, ch)| (index != remove_at).then_some(*ch))
                .collect::<String>();
            if let Some(form_ref) = self.form_ref_for_surface(&candidate) {
                found.insert(form_ref);
            }
        }

        let alphabet = if chars
            .iter()
            .all(|ch| crate::keyboard::is_cyrillic_letter(*ch))
        {
            "абвгдеёжзийклмнопрстуфхцчшщъыьэюя"
        } else if chars.iter().all(|ch| ch.is_ascii_alphabetic()) {
            "abcdefghijklmnopqrstuvwxyz"
        } else {
            ""
        };
        for insert_at in 0..=chars.len() {
            for inserted in alphabet.chars() {
                let mut candidate = String::with_capacity(surface.len() + inserted.len_utf8());
                for (index, ch) in chars.iter().enumerate() {
                    if index == insert_at {
                        candidate.push(inserted);
                    }
                    candidate.push(*ch);
                }
                if insert_at == chars.len() {
                    candidate.push(inserted);
                }
                if let Some(form_ref) = self.form_ref_for_surface(&candidate) {
                    found.insert(form_ref);
                }
            }
        }

        for replace_at in 0..chars.len() {
            for replacement in alphabet
                .chars()
                .filter(|replacement| *replacement != chars[replace_at])
            {
                let candidate = chars
                    .iter()
                    .enumerate()
                    .map(|(index, ch)| {
                        if index == replace_at {
                            replacement
                        } else {
                            *ch
                        }
                    })
                    .collect::<String>();
                if let Some(form_ref) = self.form_ref_for_surface(&candidate) {
                    found.insert(form_ref);
                }
            }
        }

        for swap_at in 0..chars.len().saturating_sub(1) {
            if chars[swap_at] == chars[swap_at + 1] {
                continue;
            }
            let mut candidate = chars.clone();
            candidate.swap(swap_at, swap_at + 1);
            if let Some(form_ref) =
                self.form_ref_for_surface(&candidate.into_iter().collect::<String>())
            {
                found.insert(form_ref);
            }
        }

        found.into_iter().take(limit).collect()
    }

    pub(crate) fn l1_terminal_for_form_ref(&self, form_ref: u32) -> Option<u32> {
        let terminal_id = self.package.form(form_ref as usize)?.l1_terminal_id;
        (terminal_id != NO_L1_TERMINAL).then_some(terminal_id)
    }

    pub(crate) fn readout(
        &self,
        context: &str,
        seeds: &[L2LexicalSeed],
        candidate_limit: usize,
    ) -> StandaloneL2Readout {
        self.readout_internal(context, None, seeds, candidate_limit)
    }

    pub(crate) fn readout_observed(
        &self,
        context: &str,
        observed_surface: &str,
        seeds: &[L2LexicalSeed],
        candidate_limit: usize,
    ) -> StandaloneL2Readout {
        self.readout_internal(context, Some(observed_surface), seeds, candidate_limit)
    }

    fn readout_internal(
        &self,
        context: &str,
        observed_surface: Option<&str>,
        seeds: &[L2LexicalSeed],
        candidate_limit: usize,
    ) -> StandaloneL2Readout {
        let mode = context_mode(context);
        let context_mode_id = self.context_by_key.get(&mode.stable_key).copied();
        let wave = scene_wave(context);
        let mut resolved_seeds = Vec::new();
        for seed in seeds {
            let form_ref = seed
                .terminal_id
                .and_then(|terminal_id| {
                    self.form_by_terminal
                        .binary_search_by_key(&terminal_id, |(terminal_id, _)| *terminal_id)
                        .ok()
                        .map(|index| self.form_by_terminal[index].1)
                })
                .or_else(|| {
                    seed.surface
                        .as_deref()
                        .and_then(|surface| self.form_ref_for_surface(surface))
                });
            let Some(form_ref) = form_ref else {
                continue;
            };
            resolved_seeds.push((form_ref, seed));
        }
        let has_l11_grounding = resolved_seeds
            .iter()
            .any(|(_, seed)| seed.origin == L2LexicalSeedOrigin::GroundedL11);
        let grounded_l11_form_refs = resolved_seeds
            .iter()
            .filter(|(_, seed)| seed.origin == L2LexicalSeedOrigin::GroundedL11)
            .map(|(form_ref, _)| *form_ref)
            .collect::<BTreeSet<_>>();
        let exact_grounded_form_refs = observed_surface
            .map(str::to_lowercase)
            .map(|observed_surface| {
                preferred_exact_geometry_form_refs(
                    &observed_surface,
                    resolved_seeds
                        .iter()
                        .filter(|(_, seed)| seed.origin.is_grounded_input())
                        .filter_map(|(form_ref, _)| {
                            self.decode_form_ref(*form_ref)
                                .map(|surface| (*form_ref, surface.into_owned()))
                        }),
                )
            })
            .unwrap_or_default();
        let grounded_peak = resolved_seeds
            .iter()
            .filter(|(_, seed)| seed.origin.is_grounded_input())
            .map(|(_, seed)| seed.evidence_milli)
            .max();
        let Some(grounded_peak) = grounded_peak else {
            return StandaloneL2Readout {
                verdict: L2LocalVerdict::Abstain,
                candidates: Vec::new(),
                context_mode_id,
            };
        };
        let inherited_l1_floor = grounded_peak.saturating_sub(INHERITED_L1_ATTENUATION_MILLI);
        let mut grounded_seed_evidence = BTreeMap::<u32, i32>::new();
        let mut seed_evidence = BTreeMap::<u32, i32>::new();
        for (form_ref, seed) in resolved_seeds {
            let evidence_milli = match seed.origin {
                L2LexicalSeedOrigin::GroundedL11 => seed.evidence_milli,
                L2LexicalSeedOrigin::CompositionalMorphology => seed.evidence_milli,
                L2LexicalSeedOrigin::InverseGeometry => seed.evidence_milli.min(inherited_l1_floor),
            };
            seed_evidence
                .entry(form_ref)
                .and_modify(|evidence| *evidence = (*evidence).max(evidence_milli))
                .or_insert(evidence_milli);
            if seed.origin.is_grounded_input() {
                grounded_seed_evidence
                    .entry(form_ref)
                    .and_modify(|evidence| *evidence = (*evidence).max(seed.evidence_milli))
                    .or_insert(seed.evidence_milli);
            }
        }
        let mut direct_seed_common_lemmas = None::<BTreeSet<u32>>;
        for form_ref in grounded_seed_evidence.keys() {
            let form_lemmas = self
                .bindings_for_form(*form_ref)
                .map(|binding| binding.lemma_center_id)
                .collect::<BTreeSet<_>>();
            direct_seed_common_lemmas = Some(match direct_seed_common_lemmas {
                Some(common) => common.intersection(&form_lemmas).copied().collect(),
                None => form_lemmas,
            });
        }
        let direct_seed_common_lemmas = direct_seed_common_lemmas.unwrap_or_default();
        let mut active_forms = seed_evidence.keys().copied().collect::<BTreeSet<_>>();
        let mut lemma_seed_evidence = BTreeMap::<u32, (i32, u16, u16)>::new();
        for form_ref in grounded_seed_evidence.keys() {
            let evidence = grounded_seed_evidence
                .get(form_ref)
                .copied()
                .unwrap_or_default();
            let mut form_lemmas = BTreeMap::<u32, u16>::new();
            for binding in self.bindings_for_form(*form_ref) {
                form_lemmas
                    .entry(binding.lemma_center_id)
                    .and_modify(|support| *support = (*support).max(binding.support))
                    .or_insert(binding.support);
            }
            for (lemma_id, binding_support) in form_lemmas {
                let hypothesis = lemma_seed_evidence
                    .entry(lemma_id)
                    .or_insert((i32::MIN, 0, 0));
                hypothesis.0 = hypothesis.0.max(evidence);
                hypothesis.1 = hypothesis.1.max(binding_support);
                hypothesis.2 = hypothesis.2.saturating_add(1);
            }
        }
        let mut lemma_hypotheses = lemma_seed_evidence
            .into_iter()
            .map(
                |(lemma_id, (seed_evidence, binding_support, seed_support))| {
                    let context_evidence =
                        self.best_lemma_context_evidence(lemma_id, context_mode_id, &wave);
                    (
                        lemma_id,
                        seed_evidence.saturating_add(context_evidence),
                        seed_evidence,
                        context_evidence,
                        binding_support,
                        seed_support,
                    )
                },
            )
            .collect::<Vec<_>>();
        lemma_hypotheses.sort_by(
            |(left_id, left_total, left_seed, left_context, left_support, left_seed_support),
             (
                right_id,
                right_total,
                right_seed,
                right_context,
                right_support,
                right_seed_support,
            )| {
                right_total
                    .cmp(left_total)
                    .then_with(|| right_context.cmp(left_context))
                    .then_with(|| right_seed.cmp(left_seed))
                    .then_with(|| right_seed_support.cmp(left_seed_support))
                    .then_with(|| right_support.cmp(left_support))
                    .then_with(|| left_id.cmp(right_id))
            },
        );
        let seed_lemmas = lemma_hypotheses
            .into_iter()
            .take(MAX_ACTIVE_LEMMAS)
            .map(|(lemma_id, ..)| lemma_id)
            .collect::<BTreeSet<_>>();
        for lemma_id in &seed_lemmas {
            active_forms.extend(
                self.bindings_for_lemma(*lemma_id)
                    .map(|binding| binding.form_center_ref),
            );
            if let (Some(context_mode_id), Some(lemma)) = (
                context_mode_id,
                self.package.lemma_centers().get(*lemma_id as usize),
            ) {
                let start = lemma.competition_start as usize;
                let end = start.saturating_add(lemma.competition_count as usize);
                for edge in self
                    .package
                    .competition_edges()
                    .get(start..end)
                    .unwrap_or_default()
                    .iter()
                    .filter(|edge| edge.context_mode_id == context_mode_id)
                {
                    active_forms.insert(edge.left_form_ref);
                    active_forms.insert(edge.right_form_ref);
                }
            }
        }
        let mut candidates = active_forms
            .into_iter()
            .filter_map(|form_ref| {
                self.score_form(
                    form_ref,
                    context_mode_id,
                    &wave,
                    &grounded_seed_evidence,
                    &direct_seed_common_lemmas,
                    seed_evidence
                        .get(&form_ref)
                        .copied()
                        .unwrap_or(inherited_l1_floor),
                )
            })
            .collect::<Vec<_>>();
        candidates.sort_by(local_candidate_order);
        let mut direct_evidence_reserve_refs = grounded_l11_form_refs.clone();
        direct_evidence_reserve_refs.extend(exact_grounded_form_refs.iter().copied());
        truncate_with_grounded_l11_reserve(
            &mut candidates,
            &direct_evidence_reserve_refs,
            candidate_limit.max(1),
        );
        let verdict = if has_l11_grounding {
            classify_exact_grounded_geometry(
                &candidates,
                self.package.calibration(),
                &exact_grounded_form_refs,
            )
            .unwrap_or_else(|| classify_local(&candidates, self.package.calibration()))
        } else {
            L2LocalVerdict::Abstain
        };
        StandaloneL2Readout {
            verdict,
            candidates,
            context_mode_id,
        }
    }

    fn score_form(
        &self,
        form_ref: u32,
        context_mode_id: Option<u32>,
        wave: &[i8; L2_PHASE_CELLS],
        direct_seed_evidence: &BTreeMap<u32, i32>,
        direct_seed_common_lemmas: &BTreeSet<u32>,
        l1_evidence_milli: i32,
    ) -> Option<L2LocalCandidate> {
        let form = self.package.form(form_ref as usize)?;
        let surface = self.package.surface(form_ref as usize)?.into_owned();
        let bindings = self.bindings_for_form(form_ref).collect::<Vec<_>>();
        let lemma_ids = bindings
            .iter()
            .map(|binding| binding.lemma_center_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let feature_masks = bindings
            .iter()
            .map(|binding| binding.feature_mask)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let slot_phase_milli = context_mode_id
            .map(|context_mode_id| {
                bindings
                    .iter()
                    .flat_map(|binding| {
                        let slot_features =
                            crate::nanda_wave::morphology_phase::contextual_slot_features(
                                binding.feature_mask,
                            );
                        self.slot_centers_by_mode_feature
                            .get(&(context_mode_id, slot_features))
                            .into_iter()
                            .flatten()
                            .filter_map(|index| self.package.slot_centers().get(*index as usize))
                    })
                    .map(|center| slot_center_score(center, wave))
                    .max()
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let neighbor_pressure = context_mode_id
            .map(|context_mode_id| {
                bindings
                    .iter()
                    .flat_map(|binding| {
                        self.neighbor_couplings_by_mode_lemma_feature
                            .get(&(
                                context_mode_id,
                                binding.lemma_center_id,
                                binding.feature_mask,
                            ))
                            .into_iter()
                            .flatten()
                            .filter_map(|index| {
                                self.package.neighbor_couplings().get(*index as usize)
                            })
                    })
                    .map(|coupling| i32::from(coupling.support - coupling.repel) * 32)
                    .max()
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let (competition_pressure, explicit_competition_pressure) = context_mode_id
            .map(|context_mode_id| {
                let competition_unit = INHERITED_L1_ATTENUATION_MILLI
                    .saturating_add(
                        self.package
                            .calibration()
                            .minimum_margin
                            .max(self.package.calibration().tie_window)
                            .max(1),
                    )
                    .saturating_add(1);
                lemma_ids
                    .iter()
                    .filter_map(|lemma_id| self.package.lemma_centers().get(*lemma_id as usize))
                    .map(|lemma| {
                        let start = lemma.competition_start as usize;
                        let end = start.saturating_add(lemma.competition_count as usize);
                        self.package
                            .competition_edges()
                            .get(start..end)
                            .unwrap_or_default()
                            .iter()
                            .filter(|edge| edge.context_mode_id == context_mode_id)
                            .fold((0_i32, 0_i32), |(total, explicit), edge| {
                                let opposing_form_ref = if edge.left_form_ref == form_ref {
                                    edge.right_form_ref
                                } else if edge.right_form_ref == form_ref {
                                    edge.left_form_ref
                                } else {
                                    return (total, explicit);
                                };
                                let unambiguous_lemma_support = lemma_ids
                                    .iter()
                                    .any(|lemma_id| direct_seed_common_lemmas.contains(lemma_id));
                                if !direct_seed_evidence.contains_key(&opposing_form_ref)
                                    && !unambiguous_lemma_support
                                {
                                    return (total, explicit);
                                }
                                let pressure = if edge.left_form_ref == form_ref {
                                    i32::from(edge.support_delta.min(16))
                                        .saturating_mul(competition_unit)
                                } else if edge.right_form_ref == form_ref {
                                    -i32::from(edge.anti_delta.min(16))
                                        .saturating_mul(competition_unit)
                                } else {
                                    0
                                };
                                (
                                    total.saturating_add(pressure),
                                    explicit.saturating_add(
                                        if edge.flags & COMPETITION_FLAG_EXPLICIT_NEIGHBOR != 0 {
                                            pressure
                                        } else {
                                            0
                                        },
                                    ),
                                )
                            })
                    })
                    .max_by_key(|(total, explicit)| (*total, *explicit))
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        Some(L2LocalCandidate {
            form_ref,
            l1_terminal_id: (form.l1_terminal_id != NO_L1_TERMINAL).then_some(form.l1_terminal_id),
            surface,
            l1_evidence_milli,
            slot_phase_milli,
            neighbor_pressure,
            competition_pressure,
            explicit_competition_pressure,
            local_score: l1_evidence_milli
                .saturating_add(slot_phase_milli)
                .saturating_add(neighbor_pressure)
                .saturating_add(competition_pressure),
            lemma_ids,
            feature_masks,
        })
    }

    fn best_lemma_context_evidence(
        &self,
        lemma_id: u32,
        context_mode_id: Option<u32>,
        wave: &[i8; L2_PHASE_CELLS],
    ) -> i32 {
        self.bindings_for_lemma(lemma_id)
            .map(|binding| self.binding_slot_context_evidence(binding, context_mode_id, wave))
            .max()
            .unwrap_or_default()
    }

    fn best_lemma_neighbor_evidence(&self, lemma_id: u32, context_mode_id: u32) -> i32 {
        self.bindings_for_lemma(lemma_id)
            .map(|binding| self.binding_neighbor_context_evidence(binding, Some(context_mode_id)))
            .max()
            .unwrap_or_default()
    }

    fn binding_slot_context_evidence(
        &self,
        binding: MorphBinding,
        context_mode_id: Option<u32>,
        wave: &[i8; L2_PHASE_CELLS],
    ) -> i32 {
        let Some(context_mode_id) = context_mode_id else {
            return 0;
        };
        let slot_features =
            crate::nanda_wave::morphology_phase::contextual_slot_features(binding.feature_mask);
        self.slot_centers_by_mode_feature
            .get(&(context_mode_id, slot_features))
            .into_iter()
            .flatten()
            .filter_map(|index| self.package.slot_centers().get(*index as usize))
            .map(|center| slot_center_score(center, wave))
            .max()
            .unwrap_or_default()
    }

    fn binding_neighbor_context_evidence(
        &self,
        binding: MorphBinding,
        context_mode_id: Option<u32>,
    ) -> i32 {
        let Some(context_mode_id) = context_mode_id else {
            return 0;
        };
        self.neighbor_couplings_by_mode_lemma_feature
            .get(&(
                context_mode_id,
                binding.lemma_center_id,
                binding.feature_mask,
            ))
            .into_iter()
            .flatten()
            .filter_map(|index| self.package.neighbor_couplings().get(*index as usize))
            .map(|coupling| i32::from(coupling.support - coupling.repel).saturating_mul(32))
            .max()
            .unwrap_or_default()
    }

    fn bindings_for_form(&self, form_ref: u32) -> impl Iterator<Item = MorphBinding> + '_ {
        let start = self
            .binding_offsets_by_form
            .get(form_ref as usize)
            .copied()
            .unwrap_or_default() as usize;
        let end = self
            .binding_offsets_by_form
            .get(form_ref as usize + 1)
            .copied()
            .unwrap_or(start as u32) as usize;
        self.binding_indices_by_form
            .get(start..end)
            .unwrap_or_default()
            .iter()
            .filter_map(|index| self.package.binding(*index as usize))
    }

    fn bindings_for_lemma(&self, lemma_id: u32) -> impl Iterator<Item = MorphBinding> + '_ {
        let range = self
            .package
            .lemma_centers()
            .get(lemma_id as usize)
            .map(|lemma| {
                let start = lemma.form_start as usize;
                start..start.saturating_add(lemma.form_count as usize)
            })
            .unwrap_or(0..0);
        range.filter_map(|index| self.package.binding(index))
    }
}

fn local_candidate_order(left: &L2LocalCandidate, right: &L2LocalCandidate) -> std::cmp::Ordering {
    right
        .local_score
        .cmp(&left.local_score)
        .then_with(|| right.slot_phase_milli.cmp(&left.slot_phase_milli))
        .then_with(|| left.form_ref.cmp(&right.form_ref))
}

fn truncate_with_grounded_l11_reserve(
    candidates: &mut Vec<L2LocalCandidate>,
    grounded_l11_form_refs: &BTreeSet<u32>,
    limit: usize,
) {
    if candidates.len() <= limit {
        return;
    }
    let mut retained = candidates
        .iter()
        .filter(|candidate| grounded_l11_form_refs.contains(&candidate.form_ref))
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let remaining = limit.saturating_sub(retained.len());
    retained.extend(
        candidates
            .iter()
            .filter(|candidate| !grounded_l11_form_refs.contains(&candidate.form_ref))
            .take(remaining)
            .cloned(),
    );
    retained.sort_by(local_candidate_order);
    *candidates = retained;
}

fn contextual_feature_rank(rank: ContextualFeatureRank) -> (i32, u16, i32, i32, u16) {
    (
        rank.total_evidence_milli,
        rank.surface_evidence_milli,
        rank.neighbor_evidence_milli,
        rank.slot_evidence_milli,
        rank.support,
    )
}

fn preferred_exact_geometry_form_refs(
    observed_surface: &str,
    forms: impl IntoIterator<Item = (u32, String)>,
) -> BTreeSet<u32> {
    let exact = forms
        .into_iter()
        .filter(|(_, surface)| {
            crate::text_metrics::damerau_levenshtein(observed_surface, surface) == 1
        })
        .map(|(form_ref, surface)| {
            (
                form_ref,
                crate::text_metrics::typed_damage_geometry_priority(observed_surface, &surface),
            )
        })
        .collect::<Vec<_>>();
    let strongest_priority = exact
        .iter()
        .map(|(_, priority)| *priority)
        .max()
        .unwrap_or_default();
    exact
        .into_iter()
        .filter(|(_, priority)| strongest_priority == 0 || *priority == strongest_priority)
        .map(|(form_ref, _)| form_ref)
        .collect()
}

fn classify_exact_grounded_geometry(
    candidates: &[L2LocalCandidate],
    calibration: TieCalibration,
    exact_grounded_form_refs: &BTreeSet<u32>,
) -> Option<L2LocalVerdict> {
    let winner = candidates.first()?;
    if winner
        .slot_phase_milli
        .max(winner.neighbor_pressure)
        .max(winner.competition_pressure)
        >= calibration.minimum_positive
    {
        return None;
    }
    let form_refs = candidates
        .iter()
        .filter(|candidate| exact_grounded_form_refs.contains(&candidate.form_ref))
        .map(|candidate| candidate.form_ref)
        .collect::<Vec<_>>();
    match form_refs.as_slice() {
        [] => None,
        [form_ref] => Some(L2LocalVerdict::Winner {
            form_ref: *form_ref,
        }),
        _ => Some(L2LocalVerdict::Tied { form_refs }),
    }
}

fn compositional_form_rank_prefix(birth: CompositionalFormBirth) -> (u16, u16) {
    (birth.evidence_milli, birth.lemma_evidence_milli)
}

fn compositional_candidate_rank(
    candidate: CompositionalFormCandidate,
) -> (u16, std::cmp::Reverse<u16>) {
    (
        candidate.lemma_evidence_milli,
        std::cmp::Reverse(candidate.wave_distance),
    )
}

fn attach_productive_context_pair_evidence<I: ProductiveMorphologySource + ?Sized>(
    index: &I,
    context: &str,
    births: &mut [ProductiveL2FormBirth],
) {
    let competitors = births
        .iter()
        .map(|birth| (birth.lemma_id, birth.target_feature_mask))
        .collect::<Vec<_>>();
    for birth in births {
        birth.context_pair_evidence.clear();
        for (competitor_lemma_id, competitor_feature_mask) in &competitors {
            if *competitor_lemma_id != birth.lemma_id
                || *competitor_feature_mask == birth.target_feature_mask
            {
                continue;
            }
            let evidence = index.context_pair_evidence_for(
                context,
                birth.target_feature_mask,
                *competitor_feature_mask,
            );
            if evidence.context_observed
                && (evidence.positive_support > 0 || evidence.anti_support > 0)
                && !birth
                    .context_pair_evidence
                    .iter()
                    .any(|current| current.competitor_feature_mask == *competitor_feature_mask)
            {
                birth
                    .context_pair_evidence
                    .push(ProductiveL2ContextPairEvidence {
                        competitor_feature_mask: *competitor_feature_mask,
                        evidence,
                    });
            }
        }
        birth
            .context_pair_evidence
            .sort_by_key(|edge| edge.competitor_feature_mask);
    }
}

pub(super) fn productive_l2_birth_rank(
    birth: &ProductiveL2FormBirth,
) -> (
    u16,
    u16,
    u16,
    u16,
    i32,
    u8,
    u32,
    std::cmp::Reverse<u32>,
    std::cmp::Reverse<u16>,
) {
    (
        birth.joint_evidence_milli,
        birth.lemma_atom_evidence_milli,
        birth.geometry_evidence_milli,
        birth.profile_evidence_milli,
        birth.slot_evidence_milli,
        birth.family_specificity,
        birth.positive_support,
        std::cmp::Reverse(birth.anti_support),
        std::cmp::Reverse(birth.lemma_wave_distance),
    )
}

pub(super) fn productive_l2_readout(
    observed_surface: &str,
    births: &[ProductiveL2FormBirth],
) -> ProductiveL2Readout {
    if births.is_empty() || births.iter().any(|birth| birth.surface == observed_surface) {
        return ProductiveL2Readout::Abstain;
    }
    let surfaces = births
        .iter()
        .enumerate()
        .filter(|(index, candidate)| {
            !births.iter().enumerate().any(|(other_index, other)| {
                other_index != *index && productive_evidence_dominates(other, candidate)
            })
        })
        .map(|(_, birth)| birth.surface.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    match surfaces.as_slice() {
        [] => ProductiveL2Readout::Abstain,
        [surface] => ProductiveL2Readout::Winner {
            surface: surface.clone(),
        },
        _ => ProductiveL2Readout::Tied { surfaces },
    }
}

fn productive_evidence_dominates(
    left: &ProductiveL2FormBirth,
    right: &ProductiveL2FormBirth,
) -> bool {
    // Productive L2 owns morphology-slot competition inside a lemma basin.
    // Cross-lemma authority requires independent L1.1 or L3 evidence.
    if left.lemma_id != right.lemma_id {
        return false;
    }
    if let Some(evidence) = left
        .context_pair_evidence
        .iter()
        .find(|edge| edge.competitor_feature_mask == right.target_feature_mask)
        .map(|edge| edge.evidence)
    {
        if evidence.positive_support > 0 || evidence.anti_support > 0 {
            return directional_evidence_margin(evidence) == std::cmp::Ordering::Greater;
        }
    }
    let left_context = u8::from(left.context_positive_support > 0);
    let right_context = u8::from(right.context_positive_support > 0);
    let no_weaker_axis = left.lemma_atom_evidence_milli >= right.lemma_atom_evidence_milli
        && left.geometry_evidence_milli >= right.geometry_evidence_milli
        && left_context >= right_context;
    let stronger_axis = left.lemma_atom_evidence_milli > right.lemma_atom_evidence_milli
        || left.geometry_evidence_milli > right.geometry_evidence_milli
        || left_context > right_context;
    no_weaker_axis && stronger_axis
}

fn productive_joint_evidence_milli(
    lemma_evidence_milli: u16,
    profile_evidence_milli: u16,
    geometry_evidence_milli: u16,
) -> u16 {
    let product = u64::from(lemma_evidence_milli)
        .saturating_mul(u64::from(profile_evidence_milli))
        .saturating_mul(u64::from(geometry_evidence_milli));
    (product / 1_000_000).min(1_000) as u16
}

fn classify_local(candidates: &[L2LocalCandidate], calibration: TieCalibration) -> L2LocalVerdict {
    const MAX_SUPPORT_UNCERTAINTY: i32 = 16 * 8;

    let Some(winner) = candidates.first() else {
        return L2LocalVerdict::Abstain;
    };
    if winner
        .slot_phase_milli
        .max(winner.neighbor_pressure)
        .max(winner.competition_pressure)
        < calibration.minimum_positive
    {
        let lexical_peak = winner.l1_evidence_milli;
        let form_refs = candidates
            .iter()
            .take_while(|candidate| candidate.l1_evidence_milli == lexical_peak)
            .map(|candidate| candidate.form_ref)
            .collect::<Vec<_>>();
        return if form_refs.len() > 1 {
            L2LocalVerdict::Tied { form_refs }
        } else {
            L2LocalVerdict::Abstain
        };
    }
    if let Some(verdict) =
        cross_lemma_authority_safety_verdict(candidates, winner, MAX_SUPPORT_UNCERTAINTY)
    {
        return verdict;
    }
    let equivalent_slot = candidates
        .iter()
        .filter(|candidate| {
            let same_lemma = candidate
                .lemma_ids
                .iter()
                .any(|lemma| winner.lemma_ids.contains(lemma));
            let equivalent_positive_slot = candidate.slot_phase_milli
                >= calibration.minimum_positive
                && winner.slot_phase_milli.abs_diff(candidate.slot_phase_milli)
                    <= MAX_SUPPORT_UNCERTAINTY as u32;
            let inclusive_imperative_tie =
                candidate.feature_masks.iter().any(|candidate_features| {
                    winner.feature_masks.iter().any(|winner_features| {
                        crate::nanda_wave::morphology_phase::same_inclusive_imperative_family(
                            *winner_features,
                            *candidate_features,
                        )
                    })
                });
            let finite_agreement_tie = winner.neighbor_pressure <= 0
                && winner.competition_pressure == 0
                && candidate.feature_masks.iter().any(|candidate_features| {
                    winner.feature_masks.iter().any(|winner_features| {
                        crate::nanda_wave::morphology_phase::same_finite_agreement_family(
                            *winner_features,
                            *candidate_features,
                        )
                    })
                });
            same_lemma
                && (equivalent_positive_slot || inclusive_imperative_tie || finite_agreement_tie)
        })
        .map(|candidate| candidate.form_ref)
        .collect::<Vec<_>>();
    if equivalent_slot.len() > 1 {
        let mut form_refs = equivalent_slot;
        for form_ref in candidates
            .iter()
            .take_while(|candidate| {
                winner.local_score.saturating_sub(candidate.local_score) <= calibration.tie_window
            })
            .map(|candidate| candidate.form_ref)
        {
            if !form_refs.contains(&form_ref) {
                form_refs.push(form_ref);
            }
        }
        return L2LocalVerdict::Tied { form_refs };
    }
    let margin = winner.local_score.saturating_sub(
        candidates
            .get(1)
            .map(|candidate| candidate.local_score)
            .unwrap_or_default(),
    );
    if margin <= calibration.tie_window.max(calibration.minimum_margin - 1) {
        return L2LocalVerdict::Tied {
            form_refs: candidates
                .iter()
                .take_while(|candidate| {
                    winner.local_score.saturating_sub(candidate.local_score)
                        <= calibration.tie_window
                })
                .map(|candidate| candidate.form_ref)
                .collect(),
        };
    }
    L2LocalVerdict::Winner {
        form_ref: winner.form_ref,
    }
}

fn cross_lemma_authority_safety_verdict(
    candidates: &[L2LocalCandidate],
    winner: &L2LocalCandidate,
    support_uncertainty: i32,
) -> Option<L2LocalVerdict> {
    let strongest_lexical_seed = candidates
        .iter()
        .map(|candidate| candidate.l1_evidence_milli)
        .max()
        .unwrap_or_default();
    let winner_has_independent_lemma_seed = candidates.iter().any(|candidate| {
        candidate.l1_evidence_milli == strongest_lexical_seed
            && candidate
                .lemma_ids
                .iter()
                .any(|lemma_id| winner.lemma_ids.contains(lemma_id))
    });
    if winner.explicit_competition_pressure > 0 && winner_has_independent_lemma_seed {
        return None;
    }
    let independent_score = |candidate: &L2LocalCandidate| {
        candidate
            .l1_evidence_milli
            .saturating_add(candidate.neighbor_pressure)
    };
    let winner_independent = independent_score(winner);
    let strongest_foreign = candidates
        .iter()
        .filter(|candidate| {
            !candidate
                .lemma_ids
                .iter()
                .any(|lemma_id| winner.lemma_ids.contains(lemma_id))
        })
        .map(independent_score)
        .max()?;
    if winner_independent.saturating_sub(strongest_foreign) > support_uncertainty {
        return None;
    }
    let form_refs = candidates
        .iter()
        .map(|candidate| candidate.form_ref)
        .collect::<Vec<_>>();
    Some(if form_refs.len() > 1 {
        L2LocalVerdict::Tied { form_refs }
    } else {
        L2LocalVerdict::Abstain
    })
}

fn slot_center_score(center: &SlotPhaseCenter, wave: &[i8; L2_PHASE_CELLS]) -> i32 {
    coherence_milli(&center.cells, wave)
        .saturating_mul(i32::from(center.polarity))
        .saturating_add(i32::from(center.support.min(16)) * 8)
}

fn coherence_milli(left: &[i8; L2_PHASE_CELLS], right: &[i8; L2_PHASE_CELLS]) -> i32 {
    let mut dot = 0_i64;
    let mut left_norm = 0_i64;
    let mut right_norm = 0_i64;
    for (left, right) in left.iter().zip(right) {
        let left = i64::from(*left);
        let right = i64::from(*right);
        dot = dot.saturating_add(left.saturating_mul(right));
        left_norm = left_norm.saturating_add(left.saturating_mul(left));
        right_norm = right_norm.saturating_add(right.saturating_mul(right));
    }
    if left_norm == 0 || right_norm == 0 {
        return 0;
    }
    let denominator = ((left_norm as f64).sqrt() * (right_norm as f64).sqrt()).max(1.0);
    ((dot as f64 * 1_000.0) / denominator)
        .round()
        .clamp(-1_000.0, 1_000.0) as i32
}

#[cfg(test)]
mod standalone_tests {
    use super::super::compact_format::encode_package as encode_compact_package;
    use super::super::compiler::compile_l2_package;
    use super::super::format::encode_package;
    use super::super::teacher::L2TeacherCorpus;
    use super::*;

    fn productive_birth(
        surface: &str,
        lemma_id: u32,
        lemma_evidence: u16,
        geometry_evidence: u16,
    ) -> ProductiveL2FormBirth {
        ProductiveL2FormBirth {
            surface: surface.to_string(),
            lemma_id,
            source_form_ref: 0,
            source_feature_mask: 0,
            target_feature_mask: 0,
            geometry_evidence_milli: geometry_evidence,
            profile_evidence_milli: 1_000,
            slot_evidence_milli: 1_000,
            context_positive_support: 1,
            context_unlabeled_alternative_support: 0,
            context_posterior_milli: 1_000,
            context_observed: true,
            context_pair_evidence: Vec::new(),
            joint_evidence_milli: lemma_evidence.min(geometry_evidence),
            positive_support: 1,
            anti_support: 0,
            family_specificity: 1,
            lemma_atom_evidence_milli: lemma_evidence,
            lemma_wave_distance: 0,
            exact_surface_form_ref: None,
            status: ProductiveBirthStatus::ShadowUnverified,
        }
    }

    #[test]
    fn productive_readout_cannot_collapse_distinct_lemma_basins() {
        let births = vec![
            productive_birth("strong", 1, 900, 900),
            productive_birth("retained", 2, 500, 500),
        ];

        assert_eq!(
            productive_l2_readout("damaged", &births),
            ProductiveL2Readout::Tied {
                surfaces: vec!["retained".to_string(), "strong".to_string()]
            }
        );
    }

    #[test]
    fn productive_readout_can_settle_a_form_inside_one_lemma_basin() {
        let births = vec![
            productive_birth("strong", 1, 900, 900),
            productive_birth("weak", 1, 500, 500),
        ];

        assert_eq!(
            productive_l2_readout("damaged", &births),
            ProductiveL2Readout::Winner {
                surface: "strong".to_string()
            }
        );
    }

    #[test]
    fn directional_context_pair_can_settle_only_its_same_lemma_competitor() {
        let mut preferred = productive_birth("preferred", 1, 700, 700);
        preferred.target_feature_mask = 11;
        preferred.context_pair_evidence = vec![ProductiveL2ContextPairEvidence {
            competitor_feature_mask: 22,
            evidence: ProductiveContextPairEvidence {
                positive_support: 3,
                anti_support: 1,
                posterior_milli: 666,
                context_observed: true,
                exact_positive_support: 3,
                exact_anti_support: 1,
                ..ProductiveContextPairEvidence::default()
            },
        }];
        let mut competitor = productive_birth("competitor", 1, 900, 900);
        competitor.target_feature_mask = 22;
        competitor.context_pair_evidence = vec![ProductiveL2ContextPairEvidence {
            competitor_feature_mask: 11,
            evidence: ProductiveContextPairEvidence {
                positive_support: 1,
                anti_support: 3,
                posterior_milli: 333,
                context_observed: true,
                exact_positive_support: 1,
                exact_anti_support: 3,
                ..ProductiveContextPairEvidence::default()
            },
        }];
        let mut other_lemma = productive_birth("other-lemma", 2, 500, 500);
        other_lemma.target_feature_mask = 22;

        assert_eq!(
            productive_l2_readout(
                "damaged",
                &[preferred.clone(), competitor, other_lemma.clone()]
            ),
            ProductiveL2Readout::Tied {
                surfaces: vec!["other-lemma".to_string(), "preferred".to_string()]
            }
        );
        assert_eq!(
            productive_l2_readout("damaged", &[preferred, other_lemma]),
            ProductiveL2Readout::Tied {
                surfaces: vec!["other-lemma".to_string(), "preferred".to_string()]
            }
        );
    }

    #[test]
    fn productive_readout_abstains_when_input_is_already_a_generated_form() {
        let births = vec![
            productive_birth("observed", 1, 900, 900),
            productive_birth("alternative", 1, 500, 500),
        ];

        assert_eq!(
            productive_l2_readout("observed", &births),
            ProductiveL2Readout::Abstain
        );
    }

    #[test]
    fn exact_geometry_prefers_a_unique_stronger_operator() {
        let preferred = preferred_exact_geometry_form_refs(
            "acbd",
            [(1, "abcd".to_string()), (2, "axbd".to_string())],
        );

        assert_eq!(preferred, BTreeSet::from([1]));
    }

    #[test]
    fn exact_geometry_keeps_same_operator_ambiguity_tied() {
        let preferred = preferred_exact_geometry_form_refs(
            "abcd",
            [(1, "bacd".to_string()), (2, "acbd".to_string())],
        );

        assert_eq!(preferred, BTreeSet::from([1, 2]));
    }

    #[test]
    fn exact_geometry_keeps_untyped_distance_ambiguity_tied() {
        let preferred = preferred_exact_geometry_form_refs(
            "abcd",
            [(1, "abed".to_string()), (2, "abfd".to_string())],
        );

        assert_eq!(preferred, BTreeSet::from([1, 2]));
    }

    #[test]
    fn bounded_readout_reserves_grounded_l11_forms_before_compositional_fill() {
        let mut candidates = (0_u32..40)
            .map(|form_ref| L2LocalCandidate {
                form_ref,
                l1_terminal_id: Some(form_ref),
                surface: format!("surface-{form_ref}"),
                l1_evidence_milli: 1_000 - form_ref as i32,
                slot_phase_milli: 0,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 1_000 - form_ref as i32,
                lemma_ids: Vec::new(),
                feature_masks: Vec::new(),
            })
            .collect::<Vec<_>>();
        let grounded = BTreeSet::from([39]);

        truncate_with_grounded_l11_reserve(&mut candidates, &grounded, 32);

        assert_eq!(candidates.len(), 32);
        assert!(candidates.iter().any(|candidate| candidate.form_ref == 39));
        assert!(!candidates.iter().any(|candidate| candidate.form_ref == 31));
        assert!(candidates
            .windows(2)
            .all(|pair| local_candidate_order(&pair[0], &pair[1]).is_le()));
    }

    #[test]
    fn reference_and_compact_runtime_readouts_are_exactly_equal() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             F\tдома\tдома\tnoun:nom:pl\n\
             F\tпосмотреть\tпосмотреть\tverb:inf:perf\n\
             F\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\n\
             F\tпросмотреть\tпросмотри\tverb:imp_excl:sg:imp:perf\n\
             T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
             T\tдом\tдома\tnoun:gen:sg\tнет _\n\
             T\tдома\tдома\tnoun:nom:pl\tработаю _\n\
             T\tпосмотреть\tпосмотреть\tverb:inf:perf\tхочу _\n\
             H\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\n\
             NT\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n\
             NH\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([
            ("дом", 7),
            ("дома", 11),
            ("посмотреть", 17),
            ("просмотри", 23),
        ]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let reference_bytes = encode_package(&package).expect("reference encode");
        let (compact_bytes, _) = encode_compact_package(&package).expect("compact encode");
        let reference = StandaloneL2Field::from_bytes(&reference_bytes).expect("reference load");
        let compact = StandaloneL2Field::from_bytes(&compact_bytes).expect("compact load");
        let mmap_path = std::env::temp_dir().join(format!(
            "lay-l2-compact-mmap-{}-{}.bin",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::write(&mmap_path, &compact_bytes).expect("write mmap fixture");
        let mapped = StandaloneL2Field::load(&mmap_path).expect("mmap load");
        std::fs::remove_file(&mmap_path).expect("remove mmap fixture");

        assert_eq!(reference.package_counts(), compact.package_counts());
        assert_eq!(reference.package_storage().0, "reference_v2_owned");
        assert_eq!(
            compact.package_storage(),
            ("compact_v2_compositional", compact_bytes.len())
        );
        assert_eq!(
            compact.compositional_index_source(),
            "compact_v2_owned_view"
        );
        assert!(!compact.package_mmap_backed());
        assert!(compact.compositional_index_bytes() > 0);
        assert!(compact.compositional_index_bytes() < compact.compositional_index_view_bytes());
        assert!(compact.compositional_index_view_bytes() > 0);
        #[cfg(target_os = "linux")]
        {
            assert!(mapped.package_mmap_backed());
            assert_eq!(mapped.compositional_index_source(), "compact_v2_mmap_view");
        }
        assert_eq!(
            mapped.compositional_index_bytes(),
            compact.compositional_index_bytes()
        );
        assert_eq!(
            mapped.compositional_index_view_bytes(),
            compact.compositional_index_view_bytes()
        );
        for runtime in [&reference, &compact, &mapped] {
            for lemma_id in 0..runtime.package.lemma_centers().len() as u32 {
                let (primary_pos, fast) = package_canonical_source(&runtime.package, lemma_id)
                    .expect("fast canonical source");
                let lemma = package_lemma(&runtime.package, lemma_id).expect("full lemma");
                let full = super::super::productive::canonical_source(&lemma.forms)
                    .expect("full canonical source");
                assert_eq!(primary_pos, lemma.primary_pos);
                assert_eq!(&fast, full);
            }
        }
        assert_eq!(
            reference.single_edit_form_refs("дмо", 16),
            compact.single_edit_form_refs("дмо", 16)
        );

        let probes = [
            (
                "нет _",
                vec![L2LexicalSeed {
                    terminal_id: Some(7),
                    surface: None,
                    evidence_milli: 900,
                    origin: L2LexicalSeedOrigin::GroundedL11,
                }],
            ),
            (
                "_ сюда",
                vec![
                    L2LexicalSeed {
                        terminal_id: Some(17),
                        surface: None,
                        evidence_milli: 1_000,
                        origin: L2LexicalSeedOrigin::GroundedL11,
                    },
                    L2LexicalSeed {
                        terminal_id: Some(23),
                        surface: None,
                        evidence_milli: 960,
                        origin: L2LexicalSeedOrigin::GroundedL11,
                    },
                ],
            ),
            (
                "неизвестная сцена _",
                vec![
                    L2LexicalSeed {
                        terminal_id: None,
                        surface: Some("дом".to_string()),
                        evidence_milli: 1_000,
                        origin: L2LexicalSeedOrigin::GroundedL11,
                    },
                    L2LexicalSeed {
                        terminal_id: None,
                        surface: Some("дома".to_string()),
                        evidence_milli: 1_000,
                        origin: L2LexicalSeedOrigin::GroundedL11,
                    },
                ],
            ),
        ];
        for (context, seeds) in probes {
            assert_eq!(
                reference.readout(context, &seeds, 8),
                compact.readout(context, &seeds, 8),
                "runtime parity failed for context {context:?}"
            );
            assert_eq!(
                reference.readout(context, &seeds, 8),
                mapped.readout(context, &seeds, 8),
                "mmap runtime parity failed for context {context:?}"
            );
        }
    }

    #[test]
    fn compact_runtime_indexes_preserve_terminal_form_and_lemma_bindings() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             F\tдома\tдома\tnoun:nom:pl\n\
             F\tдома\tдомой\tnoun:dat:pl\n\
             T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
             T\tдом\tдома\tnoun:gen:sg\tнет _\n\
             T\tдома\tдома\tnoun:nom:pl\tработаю _\n\
             H\tдом\tдома\tnoun:gen:sg\tоколо _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("дом", 31), ("дома", 7), ("домой", 19)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");

        let terminal_form = |terminal_id| {
            field
                .form_by_terminal
                .binary_search_by_key(&terminal_id, |(terminal_id, _)| *terminal_id)
                .ok()
                .map(|index| field.form_by_terminal[index].1)
        };
        assert_eq!(
            terminal_form(7)
                .and_then(|form_ref| field.decode_form_ref(form_ref))
                .as_deref(),
            Some("дома")
        );
        assert_eq!(terminal_form(999), None);

        let shared_form = field.form_ref_for_surface("дома").expect("shared form");
        let shared_lemmas = field
            .bindings_for_form(shared_form)
            .map(|binding| binding.lemma_center_id)
            .collect::<Vec<_>>();
        assert_eq!(shared_lemmas.len(), 2);
        assert_ne!(shared_lemmas[0], shared_lemmas[1]);

        for lemma_id in shared_lemmas {
            let bindings = field.bindings_for_lemma(lemma_id).collect::<Vec<_>>();
            assert!(!bindings.is_empty());
            assert!(bindings
                .iter()
                .all(|binding| binding.lemma_center_id == lemma_id));
            assert!(bindings
                .iter()
                .any(|binding| binding.form_center_ref == shared_form));
        }
    }

    #[test]
    fn lexical_lemma_observation_exposes_complete_typed_read_only_sources() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tlemma-a\tform-a\tnoun:nom:sg\n\
             F\tlemma-a\tform-b\tnoun:gen:sg\n\
             F\tlemma-a\tform-c\tverb:inf\n\
             T\tlemma-a\tform-a\tnoun:nom:sg\t_ context\n\
             H\tlemma-a\tform-b\tnoun:gen:sg\theldout _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("form-a", 1), ("form-b", 2), ("form-c", 3)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");

        let observation = field
            .lexical_lemma_observation_v1(0)
            .expect("observation")
            .expect("known lemma");

        assert_eq!(observation.lemma_id, 0);
        assert_eq!(observation.known_pos_domains, vec![1, 2]);
        assert_eq!(observation.exact_source_forms.len(), 3);
        assert_eq!(
            observation.canonical_source_form_ref,
            observation
                .exact_source_forms
                .first()
                .map(|source| source.form_ref)
        );
        assert!(observation.exact_source_forms.windows(2).all(|pair| {
            (
                pair[0].canonical_preference,
                pair[0].normalized_surface.chars().count(),
                pair[0].feature_mask,
                &pair[0].normalized_surface,
                pair[0].form_ref,
            ) <= (
                pair[1].canonical_preference,
                pair[1].normalized_surface.chars().count(),
                pair[1].feature_mask,
                &pair[1].normalized_surface,
                pair[1].form_ref,
            )
        }));
        assert_eq!(
            field
                .lexical_lemma_observation_v1(1)
                .expect("unknown lemma"),
            None
        );
    }

    #[test]
    fn compositional_birth_recovers_an_unbound_exact_paradigm_surface() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tпроверять\tпроверять\tverb:inf:imperf\n\
             F\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\n\
             F\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\n\
             T\tпроверять\tпроверять\tverb:inf:imperf\tнужно _\n\
             H\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\tя _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("проверять", 17)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");

        let births = field.compositional_form_births("провряю", 4, 8);
        for limit in 1..=births.len() {
            assert_eq!(
                field.compositional_form_births("провряю", 4, limit),
                births.iter().copied().take(limit).collect::<Vec<_>>(),
                "deferred atom scoring must preserve eager full-lattice order"
            );
        }
        let target = field
            .form_ref_for_surface("проверяю")
            .expect("exact target surface");
        assert!(births.iter().any(|birth| birth.form_ref == target));
        assert_eq!(field.l1_terminal_for_form_ref(target), None);

        let seeds = births
            .iter()
            .map(|birth| L2LexicalSeed {
                terminal_id: None,
                surface: field
                    .decode_form_ref(birth.form_ref)
                    .map(|value| value.into_owned()),
                evidence_milli: i32::from(birth.evidence_milli),
                origin: L2LexicalSeedOrigin::CompositionalMorphology,
            })
            .collect::<Vec<_>>();
        let readout = field.readout("я _", &seeds, 8);
        assert_eq!(readout.verdict, L2LocalVerdict::Abstain);
        assert!(readout
            .candidates
            .iter()
            .any(|candidate| candidate.form_ref == target));

        let clean = field.compositional_form_births("ПРОВЕРЯЕТ!", 1, 1);
        let clean_target = field
            .form_ref_for_surface("проверяет")
            .expect("normalized clean target");
        assert_eq!(
            clean.first().map(|birth| birth.form_ref),
            Some(clean_target)
        );
        assert_eq!(clean[0].evidence_milli, 1_000);
        assert_eq!(clean[0].wave_distance, 0);
    }

    #[test]
    fn contextual_lemma_reduction_uses_trained_slot_evidence_before_form_expansion() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tпроверка\tпроверка\tnoun:nom:sg\n\
             F\tпроверка\tпроверки\tnoun:gen:sg\n\
             F\tпроверять\tпроверять\tverb:inf:imperf\n\
             F\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\n\
             T\tпроверка\tпроверки\tnoun:gen:sg\tнет _\n\
             T\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tон _\n\
             H\tпроверка\tпроверка\tnoun:nom:sg\t_ готова\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([
            ("проверка", 11),
            ("проверки", 13),
            ("проверять", 17),
            ("проверяет", 19),
        ]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let noun_lemma = field
            .form_ref_for_surface("проверка")
            .and_then(|form_ref| field.bindings_for_form(form_ref).next())
            .map(|binding| binding.lemma_center_id)
            .expect("noun lemma");
        let verb_lemma = field
            .form_ref_for_surface("проверяет")
            .and_then(|form_ref| field.bindings_for_form(form_ref).next())
            .map(|binding| binding.lemma_center_id)
            .expect("verb lemma");
        let broad = vec![
            CompositionalLemmaBirth {
                lemma_id: noun_lemma,
                atom_evidence: 100,
                atom_evidence_milli: 1_000,
                wave_distance: 1,
            },
            CompositionalLemmaBirth {
                lemma_id: verb_lemma,
                atom_evidence: 90,
                atom_evidence_milli: 900,
                wave_distance: 2,
            },
        ];

        let active = field.contextual_compositional_lemma_births("он _", &broad, 1);
        assert_eq!(active.first().map(|birth| birth.lemma_id), Some(verb_lemma));
        assert_eq!(
            field.contextual_compositional_lemma_births("он _", &broad, broad.len()),
            broad,
            "a fully retained lemma lattice must not pay for or change a 256 -> 256 rerank"
        );

        let unmatched = field.contextual_compositional_lemma_births("совсем другой _", &broad, 1);
        assert_eq!(
            unmatched.first().map(|birth| birth.lemma_id),
            Some(noun_lemma)
        );
    }

    #[test]
    fn contextual_form_expansion_selects_trained_slots_without_dropping_lemma_basins() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tпроверять\tпроверять\tverb:inf:imperf\n\
             F\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\n\
             F\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\n\
             T\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\tя _\n\
             T\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tон _\n\
             H\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tон _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("проверять", 17), ("проверяю", 19), ("проверяет", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let first_person = field
            .form_ref_for_surface("проверяю")
            .expect("first-person form");
        let third_person = field
            .form_ref_for_surface("проверяет")
            .expect("third-person form");
        let lemma_id = field
            .bindings_for_form(third_person)
            .next()
            .map(|binding| binding.lemma_center_id)
            .expect("verb lemma");
        let broad = [CompositionalLemmaBirth {
            lemma_id,
            atom_evidence: 100,
            atom_evidence_milli: 1_000,
            wave_distance: 1,
        }];

        let one_slot = field.contextual_compositional_form_births_from_lemmas(
            "он _",
            "провераю",
            &broad,
            1,
            8,
        );
        assert!(one_slot.iter().any(|birth| birth.form_ref == third_person));
        assert!(!one_slot.iter().any(|birth| birth.form_ref == first_person));

        let two_slots = field.contextual_compositional_form_births_from_lemmas(
            "он _",
            "провераю",
            &broad,
            2,
            8,
        );
        assert!(two_slots.iter().any(|birth| birth.form_ref == third_person));
        assert!(two_slots.iter().any(|birth| birth.form_ref == first_person));
    }

    #[test]
    fn uncontextualized_feature_selection_uses_surface_geometry_before_support() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\n\
             F\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\n\
             T\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tон _\n\
             T\tпроверять\tпроверяет\tverb:sg:p3:pres:ind:imperf\tпроцесс _\n\
             H\tпроверять\tпроверяю\tverb:sg:p1:pres:ind:imperf\tя _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("проверяю", 19), ("проверяет", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let first_person = field
            .form_ref_for_surface("проверяю")
            .expect("first-person form");
        let lemma_id = field
            .bindings_for_form(first_person)
            .next()
            .map(|binding| binding.lemma_center_id)
            .expect("verb lemma");
        let broad = [CompositionalLemmaBirth {
            lemma_id,
            atom_evidence: 100,
            atom_evidence_milli: 1_000,
            wave_distance: 1,
        }];
        let unknown_context = "неизвестная сцена _";
        assert!(!field.context_mode_known(unknown_context));

        let births = field.contextual_compositional_form_births_from_lemmas(
            unknown_context,
            "провераю",
            &broad,
            1,
            8,
        );

        assert_eq!(
            births.first().map(|birth| birth.form_ref),
            Some(first_person)
        );
        assert!(births.iter().all(|birth| birth.form_ref == first_person));
    }

    #[test]
    fn inverse_single_edit_lane_finds_package_forms_without_scanning_the_field() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tокно\tокно\tnoun:nom:sg\n\
             F\tокно\tокне\tnoun:prep:sg\n\
             F\tперспективный\tперспективнее\tadj:comp\n\
             F\tотвлекаться\tотвлекайся\tverb:imp:p2:sg:imperf\n\
             F\tперехватить\tперехвачу\tverb:fut:ind:p1:sg:perf\n\
             T\tокно\tокно\tnoun:nom:sg\t_ открыто\n\
             T\tокно\tокне\tnoun:prep:sg\tв _\n\
             H\tокно\tокне\tnoun:prep:sg\tна _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([
            ("окно", 7),
            ("окне", 11),
            ("перехвачу", 13),
            ("перспективнее", 17),
            ("отвлекайся", 19),
        ]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");

        let surfaces = |damaged| {
            field
                .single_edit_form_refs(damaged, 16)
                .into_iter()
                .filter_map(|form_ref| field.decode_form_ref(form_ref))
                .collect::<Vec<_>>()
        };
        assert_eq!(surfaces("окное"), vec!["окне", "окно"]);
        assert_eq!(surfaces("перхвачу"), vec!["перехвачу"]);
        assert_eq!(surfaces("переспективнее"), vec!["перспективнее"]);
        assert_eq!(surfaces("отвликайся"), vec!["отвлекайся"]);
        assert!(surfaces("окне").is_empty(), "clean forms must not fan out");
    }

    #[test]
    fn inverse_geometry_birth_is_attenuated_and_cannot_exist_without_grounded_l11() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tкод\tкод\tnoun:nom:sg\n\
             F\tкот\tкот\tnoun:nom:sg\n\
             T\tкод\tкод\tnoun:nom:sg\t_ работает\n\
             T\tкот\tкот\tnoun:nom:sg\t_ спит\n\
             H\tкод\tкод\tnoun:nom:sg\tпроверяю _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("код", 17), ("кот", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let grounded = L2LexicalSeed {
            terminal_id: Some(17),
            surface: None,
            evidence_milli: 1_000,
            origin: L2LexicalSeedOrigin::GroundedL11,
        };
        let inverse = L2LexicalSeed {
            terminal_id: Some(23),
            surface: None,
            evidence_milli: 1_000,
            origin: L2LexicalSeedOrigin::InverseGeometry,
        };

        let readout = field.readout("неизвестная сцена _", &[grounded, inverse.clone()], 8);
        let grounded_candidate = readout
            .candidates
            .iter()
            .find(|candidate| candidate.surface == "код")
            .expect("grounded candidate");
        let inverse_candidate = readout
            .candidates
            .iter()
            .find(|candidate| candidate.surface == "кот")
            .expect("inverse candidate");
        assert_eq!(grounded_candidate.l1_evidence_milli, 1_000);
        assert_eq!(inverse_candidate.l1_evidence_milli, 760);

        let inverse_only = field.readout("_ спит", &[inverse], 8);
        assert_eq!(inverse_only.verdict, L2LocalVerdict::Abstain);
        assert!(inverse_only.candidates.is_empty());
    }

    #[test]
    fn observed_readout_promotes_only_a_unique_exact_compositional_basin() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tсигнал\tсигнал\tnoun:nom:sg\n\
             F\tсигнал\tсигналы\tnoun:nom:pl\n\
             T\tсигнал\tсигнал\tnoun:nom:sg\t_ принят\n\
             H\tсигнал\tсигнал\tnoun:nom:sg\t_ получен\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("сигналы", 17)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let grounded = L2LexicalSeed {
            terminal_id: Some(17),
            surface: None,
            evidence_milli: 1_000,
            origin: L2LexicalSeedOrigin::GroundedL11,
        };
        let compositional = L2LexicalSeed {
            terminal_id: None,
            surface: Some("сигнал".to_string()),
            evidence_milli: 900,
            origin: L2LexicalSeedOrigin::CompositionalMorphology,
        };

        let readout = field.readout_observed(
            "неизвестная сцена _",
            "сигна",
            &[grounded, compositional.clone()],
            8,
        );
        let target = field.form_ref_for_surface("сигнал").expect("target form");
        assert_eq!(readout.verdict, L2LocalVerdict::Winner { form_ref: target });

        let composition_only =
            field.readout_observed("неизвестная сцена _", "сигна", &[compositional], 8);
        assert_eq!(composition_only.verdict, L2LocalVerdict::Abstain);
    }

    #[test]
    fn observed_readout_keeps_multiple_exact_basins_tied() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tкод\tкод\tnoun:nom:sg\n\
             F\tкод\tкоды\tnoun:nom:pl\n\
             F\tкот\tкот\tnoun:nom:sg\n\
             T\tкод\tкод\tnoun:nom:sg\t_ работает\n\
             T\tкот\tкот\tnoun:nom:sg\t_ спит\n\
             H\tкод\tкод\tnoun:nom:sg\tпроверен _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("коды", 17), ("кот", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let seeds = [
            L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 1_000,
                origin: L2LexicalSeedOrigin::GroundedL11,
            },
            L2LexicalSeed {
                terminal_id: None,
                surface: Some("код".to_string()),
                evidence_milli: 900,
                origin: L2LexicalSeedOrigin::CompositionalMorphology,
            },
            L2LexicalSeed {
                terminal_id: None,
                surface: Some("кот".to_string()),
                evidence_milli: 900,
                origin: L2LexicalSeedOrigin::CompositionalMorphology,
            },
        ];

        let readout = field.readout_observed("неизвестная сцена _", "кок", &seeds, 8);
        let code = field.form_ref_for_surface("код").expect("code form");
        let cat = field.form_ref_for_surface("кот").expect("cat form");
        let tied = match readout.verdict {
            L2LocalVerdict::Tied { form_refs } => form_refs,
            verdict => panic!(
                "expected exact geometry tie for {code} and {cat}, got {verdict:?}; candidates={:?}",
                readout.candidates
            ),
        };
        assert_eq!(tied.len(), 2);
        assert!(tied.contains(&code));
        assert!(tied.contains(&cat));
    }

    #[test]
    fn standalone_field_walks_from_l1_seed_to_contextual_same_lemma_form() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
             T\tдом\tдома\tnoun:gen:sg\tнет _\n\
             H\tдом\tдома\tnoun:gen:sg\tоколо _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("дом", 17), ("дома", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let bytes = encode_package(&package).expect("encode");
        let field = StandaloneL2Field::from_bytes(&bytes).expect("load");
        let readout = field.readout(
            "нет _",
            &[L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 900,
                origin: L2LexicalSeedOrigin::GroundedL11,
            }],
            8,
        );

        assert_eq!(field.l1_package_fingerprint(), 99);
        assert_eq!(readout.verdict, L2LocalVerdict::Winner { form_ref: 1 });
        assert_eq!(readout.candidates[0].surface, "дома");
    }

    #[test]
    fn standalone_field_materializes_a_form_that_is_absent_from_l1() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
             T\tдом\tдома\tnoun:gen:sg\tнет _\n\
             H\tдом\tдома\tnoun:gen:sg\tоколо _\n",
        )
        .expect("teacher");
        let (package, report) =
            compile_l2_package(&corpus, 99, |surface| (surface == "дом").then_some(17))
                .expect("compile");
        assert_eq!(report.l1_bound_forms, 1);
        assert_eq!(report.admitted_forms, 2);
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "нет _",
            &[L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 900,
                origin: L2LexicalSeedOrigin::GroundedL11,
            }],
            8,
        );

        let L2LocalVerdict::Winner { form_ref } = readout.verdict else {
            panic!("context should settle the generated form");
        };
        let winner = readout
            .candidates
            .iter()
            .find(|candidate| candidate.form_ref == form_ref)
            .expect("winner candidate");
        assert_eq!(winner.surface, "дома");
        assert_eq!(winner.l1_terminal_id, None);
    }

    #[test]
    fn standalone_field_resolves_append_only_l1_seed_by_surface() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tрефакторинг\tрефакторинг\tnoun:nom:sg\n\
             F\tрефакторинг\tрефакторинга\tnoun:gen:sg\n\
             T\tрефакторинг\tрефакторинг\tnoun:nom:sg\t_ нужен\n\
             H\tрефакторинг\tрефакторинга\tnoun:gen:sg\tпроект _\n",
        )
        .expect("teacher");
        let (package, _) = compile_l2_package(&corpus, 99, |surface| {
            (surface == "рефакторинг").then_some(17)
        })
        .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "проект _",
            &[L2LexicalSeed {
                terminal_id: Some(900_000),
                surface: Some("рефакторинга".to_string()),
                evidence_milli: 1_000,
                origin: L2LexicalSeedOrigin::GroundedL11,
            }],
            8,
        );

        assert!(readout
            .candidates
            .iter()
            .any(|candidate| candidate.surface == "рефакторинга"));
    }

    #[test]
    fn learned_competition_overcomes_one_reconstruction_attenuation() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tпосмотреть\tпосмотреть\tverb:inf:perf\n\
             F\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\n\
             F\tпросмотреть\tпросмотреть\tverb:inf:perf\n\
             F\tпросмотреть\tпросмотри\tverb:imp_excl:sg:imp:perf\n\
             T\tпосмотреть\tпосмотреть\tverb:inf:perf\tхочу _\n\
             H\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\n\
             NT\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n\
             NH\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("посмотреть", 17), ("просмотреть", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "_ сюда",
            &[L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 1_000,
                origin: L2LexicalSeedOrigin::GroundedL11,
            }],
            8,
        );

        let L2LocalVerdict::Winner { form_ref } = readout.verdict else {
            panic!("learned competition should settle one reconstruction: {readout:#?}");
        };
        assert_eq!(field.decode_form_ref(form_ref).as_deref(), Some("посмотри"));
    }

    #[test]
    fn standalone_field_abstains_when_context_mode_is_unknown() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
             H\tдом\tдома\tnoun:gen:sg\tнет _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("дом", 17), ("дома", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "совсем неизвестная сцена _",
            &[L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 900,
                origin: L2LexicalSeedOrigin::GroundedL11,
            }],
            8,
        );
        assert_eq!(readout.verdict, L2LocalVerdict::Abstain);
    }

    #[test]
    fn unknown_context_keeps_multiple_direct_surface_seeds_tied() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tкод\tкод\tnoun:nom:sg\n\
             F\tкот\tкот\tnoun:nom:sg\n\
             T\tкод\tкод\tnoun:nom:sg\t_ работает\n\
             T\tкот\tкот\tnoun:nom:sg\t_ спит\n\
             H\tкод\tкод\tnoun:nom:sg\tпроверяю _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("код", 17), ("кот", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "совсем неизвестная сцена _",
            &[
                L2LexicalSeed {
                    terminal_id: None,
                    surface: Some("код".to_string()),
                    evidence_milli: 1_000,
                    origin: L2LexicalSeedOrigin::GroundedL11,
                },
                L2LexicalSeed {
                    terminal_id: None,
                    surface: Some("кот".to_string()),
                    evidence_milli: 1_000,
                    origin: L2LexicalSeedOrigin::GroundedL11,
                },
            ],
            8,
        );

        assert_eq!(
            readout.verdict,
            L2LocalVerdict::Tied {
                form_refs: vec![0, 1]
            }
        );
    }

    #[test]
    fn contextual_multi_lemma_birth_can_select_a_weaker_seeded_lemma() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             F\tдома\tдома\tnoun:nom:sg\n\
             F\tдома\tдомик\tnoun:acc:sg\n\
             T\tдома\tдомик\tnoun:acc:sg\tвижу _\n\
             NT\tдома\tдомик\tnoun:acc:sg\tвижу _\tдома\n\
             H\tдом\tдома\tnoun:gen:sg\tнет _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("дом", 17), ("дома", 23), ("домик", 31)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "вижу _",
            &[
                L2LexicalSeed {
                    terminal_id: Some(17),
                    surface: None,
                    evidence_milli: 1_000,
                    origin: L2LexicalSeedOrigin::GroundedL11,
                },
                L2LexicalSeed {
                    terminal_id: Some(23),
                    surface: None,
                    evidence_milli: 1_000,
                    origin: L2LexicalSeedOrigin::GroundedL11,
                },
            ],
            8,
        );

        assert!(
            readout
                .candidates
                .iter()
                .any(|candidate| candidate.surface == "домик"),
            "{readout:#?}"
        );
        let L2LocalVerdict::Winner { form_ref } = readout.verdict else {
            panic!("contextual slot should settle the weaker seeded lemma");
        };
        assert_eq!(
            readout
                .candidates
                .iter()
                .find(|candidate| candidate.form_ref == form_ref)
                .map(|candidate| candidate.surface.as_str()),
            Some("домик")
        );
    }

    #[test]
    fn equal_slot_evidence_within_one_lemma_cannot_become_false_singleton() {
        let candidates = vec![
            L2LocalCandidate {
                form_ref: 17,
                l1_terminal_id: Some(17),
                surface: "первый".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_128,
                neighbor_pressure: 0,
                competition_pressure: 128,
                explicit_competition_pressure: 0,
                local_score: 2_256,
                lemma_ids: vec![3, 7],
                feature_masks: vec![11],
            },
            L2LocalCandidate {
                form_ref: 23,
                l1_terminal_id: Some(23),
                surface: "второй".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_128,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 2_128,
                lemma_ids: vec![7],
                feature_masks: vec![11],
            },
        ];

        assert_eq!(
            classify_local(
                &candidates,
                TieCalibration {
                    minimum_positive: 1,
                    minimum_margin: 1,
                    tie_window: 1,
                    ..TieCalibration::default()
                },
            ),
            L2LocalVerdict::Tied {
                form_refs: vec![17, 23]
            }
        );
    }

    #[test]
    fn competition_alone_cannot_create_cross_lemma_authority() {
        let candidates = vec![
            L2LocalCandidate {
                form_ref: 17,
                l1_terminal_id: Some(17),
                surface: "чужая".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_000,
                neighbor_pressure: 0,
                competition_pressure: 500,
                explicit_competition_pressure: 0,
                local_score: 2_500,
                lemma_ids: vec![2],
                feature_masks: vec![11],
            },
            L2LocalCandidate {
                form_ref: 23,
                l1_terminal_id: Some(23),
                surface: "целевая".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_000,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 2_000,
                lemma_ids: vec![1],
                feature_masks: vec![11],
            },
        ];

        assert_eq!(
            classify_local(
                &candidates,
                TieCalibration {
                    minimum_positive: 1,
                    minimum_margin: 1,
                    tie_window: 1,
                    ..TieCalibration::default()
                },
            ),
            L2LocalVerdict::Tied {
                form_refs: vec![17, 23]
            }
        );
    }

    #[test]
    fn explicit_competition_without_a_winner_lemma_seed_stays_tied() {
        let candidates = vec![
            L2LocalCandidate {
                form_ref: 17,
                l1_terminal_id: None,
                surface: "чужая".to_string(),
                l1_evidence_milli: 760,
                slot_phase_milli: 1_000,
                neighbor_pressure: 0,
                competition_pressure: 500,
                explicit_competition_pressure: 500,
                local_score: 2_260,
                lemma_ids: vec![2],
                feature_masks: vec![11],
            },
            L2LocalCandidate {
                form_ref: 23,
                l1_terminal_id: Some(23),
                surface: "целевая".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_000,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 2_000,
                lemma_ids: vec![1],
                feature_masks: vec![11],
            },
        ];

        assert_eq!(
            classify_local(
                &candidates,
                TieCalibration {
                    minimum_positive: 1,
                    minimum_margin: 1,
                    tie_window: 1,
                    ..TieCalibration::default()
                },
            ),
            L2LocalVerdict::Tied {
                form_refs: vec![17, 23]
            }
        );
    }

    #[test]
    fn inclusive_imperative_variants_in_one_lemma_are_tied() {
        let inclusive_singular =
            crate::nanda_wave::morphology_phase::parse_features("verb:imp_incl:sg:imp:perf")
                .expect("inclusive singular");
        let inclusive_plural =
            crate::nanda_wave::morphology_phase::parse_features("verb:imp_incl:pl:imp:perf")
                .expect("inclusive plural");
        let candidates = vec![
            L2LocalCandidate {
                form_ref: 17,
                l1_terminal_id: Some(17),
                surface: "первый".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_088,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 2_088,
                lemma_ids: vec![7],
                feature_masks: vec![inclusive_singular],
            },
            L2LocalCandidate {
                form_ref: 23,
                l1_terminal_id: Some(23),
                surface: "второй".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 0,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 1_000,
                lemma_ids: vec![7],
                feature_masks: vec![inclusive_plural],
            },
        ];

        assert_eq!(
            classify_local(
                &candidates,
                TieCalibration {
                    minimum_positive: 1,
                    minimum_margin: 1,
                    tie_window: 1,
                    ..TieCalibration::default()
                },
            ),
            L2LocalVerdict::Tied {
                form_refs: vec![17, 23]
            }
        );
    }
}
