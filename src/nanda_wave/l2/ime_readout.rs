use super::{
    form_attractor_has_authority, surface_motif_memory, L2CorrectionPeakCandidate,
    L2ImeWordCandidate, L2ImeWordCandidateKind, L2ImeWordCandidateSource,
};
use crate::keyboard::is_cyrillic_letter;
use crate::text_metrics::damerau_levenshtein;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

const LEXICAL_READOUT_CACHE_CAPACITY: usize = 128;
const NEAR_SURFACE_CACHE_CAPACITY: usize = 128;
const BOUNDARY_READOUT_CACHE_CAPACITY: usize = 128;
// The gate asks L2 for a bounded material lattice and then applies phase
// competition. Four candidates per requested display slot leave enough
// competitors for interference while avoiding a 192-node DAFSA walk per key.
const IME_L2_MATERIAL_FACTOR: usize = 4;
const TYPO_TOLERANT_PREFIX_MIN_CHARS: usize = 3;
const THIN_EXACT_PREFIX_FIELD: usize = 12;
const TYPO_TOLERANT_MATERIAL_FACTOR: usize = 6;
const TYPO_TOLERANT_MATERIAL_CAP: usize = 512;
type CachedLexicalCandidates = Arc<Vec<super::super::lexical_phase::LexicalPhaseCandidate>>;
type LexicalReadoutCache = VecDeque<(String, usize, LexicalReadoutMode, CachedLexicalCandidates)>;

static LEXICAL_READOUT_CACHE: OnceLock<Mutex<LexicalReadoutCache>> = OnceLock::new();
static READOUT_MATERIAL_CACHE: OnceLock<Mutex<ReadoutMaterialCache>> = OnceLock::new();

#[derive(Clone)]
struct NearSurfaceCacheEntry {
    generation: u64,
    normalized: String,
    limit: usize,
    surfaces: Arc<Vec<String>>,
}

#[derive(Clone)]
struct BoundaryReadoutCacheEntry {
    generation: u64,
    context_prefix: String,
    token: String,
    limit: usize,
    candidates: Arc<Vec<L2ImeWordCandidate>>,
}

struct ReadoutMaterialCache {
    near_surface_capacity: usize,
    boundary_capacity: usize,
    near_surfaces: VecDeque<NearSurfaceCacheEntry>,
    boundary_candidates: VecDeque<BoundaryReadoutCacheEntry>,
}

impl ReadoutMaterialCache {
    fn new(near_surface_capacity: usize, boundary_capacity: usize) -> Self {
        assert!(near_surface_capacity > 0);
        assert!(boundary_capacity > 0);
        Self {
            near_surface_capacity,
            boundary_capacity,
            near_surfaces: VecDeque::new(),
            boundary_candidates: VecDeque::new(),
        }
    }

    fn get_near_surfaces(
        &mut self,
        generation: u64,
        normalized: &str,
        limit: usize,
    ) -> Option<Arc<Vec<String>>> {
        let index = self.near_surfaces.iter().position(|entry| {
            entry.generation == generation && entry.normalized == normalized && entry.limit == limit
        })?;
        let entry = self.near_surfaces.remove(index)?;
        let surfaces = Arc::clone(&entry.surfaces);
        self.near_surfaces.push_back(entry);
        Some(surfaces)
    }

    fn insert_near_surfaces(
        &mut self,
        generation: u64,
        normalized: &str,
        limit: usize,
        surfaces: Arc<Vec<String>>,
    ) {
        self.near_surfaces
            .retain(|entry| entry.generation == generation);
        if let Some(index) = self.near_surfaces.iter().position(|entry| {
            entry.generation == generation && entry.normalized == normalized && entry.limit == limit
        }) {
            self.near_surfaces.remove(index);
        }
        self.near_surfaces.push_back(NearSurfaceCacheEntry {
            generation,
            normalized: normalized.to_string(),
            limit,
            surfaces,
        });
        while self.near_surfaces.len() > self.near_surface_capacity {
            self.near_surfaces.pop_front();
        }
    }

    fn get_boundary_candidates(
        &mut self,
        generation: u64,
        context_prefix: &str,
        token: &str,
        limit: usize,
    ) -> Option<Arc<Vec<L2ImeWordCandidate>>> {
        let index = self.boundary_candidates.iter().position(|entry| {
            entry.generation == generation
                && entry.context_prefix == context_prefix
                && entry.token == token
                && entry.limit == limit
        })?;
        let entry = self.boundary_candidates.remove(index)?;
        let candidates = Arc::clone(&entry.candidates);
        self.boundary_candidates.push_back(entry);
        Some(candidates)
    }

    fn insert_boundary_candidates(
        &mut self,
        generation: u64,
        context_prefix: &str,
        token: &str,
        limit: usize,
        candidates: Arc<Vec<L2ImeWordCandidate>>,
    ) {
        self.boundary_candidates
            .retain(|entry| entry.generation == generation);
        if let Some(index) = self.boundary_candidates.iter().position(|entry| {
            entry.generation == generation
                && entry.context_prefix == context_prefix
                && entry.token == token
                && entry.limit == limit
        }) {
            self.boundary_candidates.remove(index);
        }
        self.boundary_candidates
            .push_back(BoundaryReadoutCacheEntry {
                generation,
                context_prefix: context_prefix.to_string(),
                token: token.to_string(),
                limit,
                candidates,
            });
        while self.boundary_candidates.len() > self.boundary_capacity {
            self.boundary_candidates.pop_front();
        }
    }
}

fn readout_material_cache() -> &'static Mutex<ReadoutMaterialCache> {
    READOUT_MATERIAL_CACHE.get_or_init(|| {
        Mutex::new(ReadoutMaterialCache::new(
            NEAR_SURFACE_CACHE_CAPACITY,
            BOUNDARY_READOUT_CACHE_CAPACITY,
        ))
    })
}

pub(super) fn cached_boundary_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
    compute: impl FnOnce() -> Vec<L2ImeWordCandidate>,
) -> Vec<L2ImeWordCandidate> {
    cached_boundary_candidates_with_generation(
        readout_material_cache(),
        context_prefix,
        token,
        limit,
        super::super::l2_field::candidate_material_generation,
        compute,
    )
}

fn cached_boundary_candidates_with_generation(
    cache_owner: &Mutex<ReadoutMaterialCache>,
    context_prefix: &str,
    token: &str,
    limit: usize,
    generation_now: impl Fn() -> u64,
    compute: impl FnOnce() -> Vec<L2ImeWordCandidate>,
) -> Vec<L2ImeWordCandidate> {
    let generation = generation_now();
    if let Some(candidates) = cache_owner.lock().ok().and_then(|mut cache| {
        cache.get_boundary_candidates(generation, context_prefix, token, limit)
    }) {
        if generation_now() != generation {
            return Vec::new();
        }
        return candidates.as_ref().clone();
    }

    let candidates = compute();
    if generation_now() != generation {
        return Vec::new();
    }
    if let Ok(mut cache) = cache_owner.lock() {
        if generation_now() != generation {
            return Vec::new();
        }
        cache.insert_boundary_candidates(
            generation,
            context_prefix,
            token,
            limit,
            Arc::new(candidates.clone()),
        );
    }
    if generation_now() != generation {
        return Vec::new();
    }
    candidates
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LexicalReadoutMode {
    FullIme,
    Correction,
}

impl LexicalReadoutMode {
    const fn includes_completion(self) -> bool {
        matches!(self, Self::FullIme)
    }
}

pub(super) fn ime_l2_word_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    l2_word_candidates_impl(context_prefix, token, limit, LexicalReadoutMode::FullIme)
}

pub(super) fn correction_l2_word_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    l2_word_candidates_impl(context_prefix, token, limit, LexicalReadoutMode::Correction)
}

pub(super) fn correction_l2_peak_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2CorrectionPeakCandidate> {
    ranked_correction_candidates(context_prefix, token, limit)
        .into_iter()
        .map(|candidate| candidate.peak)
        .collect()
}

fn l2_word_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
    mode: LexicalReadoutMode,
) -> Vec<L2ImeWordCandidate> {
    if mode == LexicalReadoutMode::Correction {
        return ranked_correction_candidates(context_prefix, token, limit)
            .into_iter()
            .map(|candidate| {
                let morphology_slots =
                    super::super::l2_field::morphology_slot_identities_for_surface(
                        &candidate.peak.surface,
                    );
                L2ImeWordCandidate {
                    surface: candidate.peak.surface,
                    kind: candidate.peak.kind,
                    source: L2ImeWordCandidateSource::LexicalPhase,
                    score: candidate.peak.score,
                    l1_overlap: candidate.peak.l1_overlap,
                    l2_overlap: candidate.peak.l2_overlap,
                    motif_overlap: candidate.peak.motif_overlap,
                    usage_prior: candidate.usage_prior,
                    context_prior: candidate.context_prior,
                    accepted_count: candidate.accepted_count,
                    target_evidence: Default::default(),
                    morphology_slots,
                }
            })
            .collect();
    }
    if limit == 0 {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    let token_len = normalized.chars().count();
    if !(1..=18).contains(&token_len) || !is_supported_lexical_surface(&normalized) {
        return Vec::new();
    }
    let context_tokens = super::super::llmwave::tokenize(context_prefix);
    let usage = super::super::usage_prior::cached_usage_prior_snapshot();
    let usage_context = usage.prepare_hot_context(&context_tokens);
    let memory = surface_motif_memory();
    let material_limit = limit.saturating_mul(IME_L2_MATERIAL_FACTOR).max(limit);
    let lexical = cached_lexical_candidates(memory, &normalized, material_limit, mode);
    let include_completion = mode.includes_completion();
    let lexical = lexical
        .iter()
        .filter(|candidate| same_lexical_script(&normalized, &candidate.word))
        .filter(|candidate| {
            include_completion
                || !candidate.word.starts_with(&normalized)
                || candidate.word.chars().count() <= token_len
        })
        .cloned();
    let mut candidates = lexical
        .map(|candidate| {
            let morphology_slots =
                super::super::l2_field::morphology_slot_identities_for_surface(&candidate.word);
            let candidate_len = candidate.word.chars().count();
            let corrected_prefix = mode != LexicalReadoutMode::Correction
                && candidate.reconstructed
                && candidate_len > token_len
                && !candidate.word.starts_with(&normalized);
            let kind =
                if crate::text_metrics::is_adjacent_transposition(&normalized, &candidate.word) {
                    L2ImeWordCandidateKind::AdjacentTransposition
                } else if candidate.word.starts_with(&normalized) && candidate_len > token_len {
                    L2ImeWordCandidateKind::Completion
                } else {
                    L2ImeWordCandidateKind::Replacement
                };
            let prior = usage.candidate_prior_prepared(&usage_context, &candidate.word);
            L2ImeWordCandidate {
                surface: candidate.word,
                kind,
                source: if corrected_prefix {
                    L2ImeWordCandidateSource::CorrectedPrefixPhase
                } else {
                    L2ImeWordCandidateSource::LexicalPhase
                },
                score: candidate.score,
                l1_overlap: candidate.l1_overlap,
                l2_overlap: candidate.l2_overlap,
                motif_overlap: candidate.motif_overlap,
                usage_prior: prior.word_prior,
                context_prior: prior.context_prior,
                accepted_count: prior.accepted_count,
                // Reconstruction is a property of the broad lexical search,
                // not proof that this exact target repairs the observed token.
                // The shared live gate binds target evidence after it verifies
                // the concrete edit operator and target center.
                target_evidence: Default::default(),
                morphology_slots,
            }
        })
        .collect::<Vec<_>>();
    sort_and_truncate_ime_l2_candidates(&normalized, &mut candidates, limit);
    candidates
}

struct RankedCorrectionCandidate {
    peak: L2CorrectionPeakCandidate,
    usage_prior: f32,
    context_prior: f32,
    accepted_count: u32,
    geometry_priority: u8,
    rank_score: u32,
    surface_char_len: usize,
}

fn ranked_correction_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<RankedCorrectionCandidate> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = token.to_lowercase();
    let token_len = normalized.chars().count();
    if !(1..=18).contains(&token_len) || !is_supported_lexical_surface(&normalized) {
        return Vec::new();
    }
    let context_tokens = super::super::llmwave::tokenize(context_prefix);
    let usage = super::super::usage_prior::cached_usage_prior_snapshot();
    let usage_context = usage.prepare_hot_context(&context_tokens);
    let material_limit = limit.saturating_mul(IME_L2_MATERIAL_FACTOR).max(limit);
    let lexical = cached_lexical_candidates(
        surface_motif_memory(),
        &normalized,
        material_limit,
        LexicalReadoutMode::Correction,
    );
    let mut candidates = lexical
        .iter()
        .filter(|candidate| same_lexical_script(&normalized, &candidate.word))
        // Completion is a separate operator and cannot enter a correction peak.
        .filter(|candidate| {
            !candidate.word.starts_with(&normalized) || candidate.word.chars().count() <= token_len
        })
        .map(|candidate| {
            let geometry_priority =
                crate::text_metrics::typed_damage_geometry_priority(&normalized, &candidate.word);
            let surface_char_len = candidate.word.chars().count();
            let kind =
                if crate::text_metrics::is_adjacent_transposition(&normalized, &candidate.word) {
                    L2ImeWordCandidateKind::AdjacentTransposition
                } else {
                    L2ImeWordCandidateKind::Replacement
                };
            let prior = usage.candidate_prior_prepared(&usage_context, &candidate.word);
            let rank_score = correction_candidate_rank_score(
                candidate.score,
                prior.word_prior,
                prior.context_prior,
                prior.accepted_count,
            );
            RankedCorrectionCandidate {
                peak: L2CorrectionPeakCandidate {
                    surface: candidate.word.clone(),
                    kind,
                    score: candidate.score,
                    l1_overlap: candidate.l1_overlap,
                    l2_overlap: candidate.l2_overlap,
                    motif_overlap: candidate.motif_overlap,
                },
                usage_prior: prior.word_prior,
                context_prior: prior.context_prior,
                accepted_count: prior.accepted_count,
                geometry_priority,
                rank_score,
                surface_char_len,
            }
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .geometry_priority
            .cmp(&left.geometry_priority)
            .then_with(|| right.rank_score.cmp(&left.rank_score))
            .then_with(|| right.peak.motif_overlap.cmp(&left.peak.motif_overlap))
            .then_with(|| right.peak.l2_overlap.cmp(&left.peak.l2_overlap))
            .then_with(|| right.peak.l1_overlap.cmp(&left.peak.l1_overlap))
            .then_with(|| left.surface_char_len.cmp(&right.surface_char_len))
            .then_with(|| left.peak.surface.cmp(&right.peak.surface))
    });
    let mut seen = HashSet::new();
    candidates.retain(|candidate| seen.insert(candidate.peak.surface.clone()));
    candidates.truncate(limit);
    candidates
}

fn correction_candidate_rank_score(
    structural_score: u32,
    usage_prior: f32,
    context_prior: f32,
    accepted_count: u32,
) -> u32 {
    let prior = ((usage_prior * 1600.0 + context_prior * 2600.0)
        .round()
        .clamp(0.0, 820.0) as u32)
        .saturating_add(accepted_count.min(40) * 18);
    structural_score.saturating_add(prior)
}

fn cached_lexical_candidates(
    memory: &super::super::lexical_phase::LexicalPhaseMemory,
    normalized: &str,
    material_limit: usize,
    mode: LexicalReadoutMode,
) -> CachedLexicalCandidates {
    let cache = LEXICAL_READOUT_CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    if let Some(candidates) = cache.lock().ok().and_then(|cache| {
        cache
            .iter()
            .find(|(surface, limit, cached_mode, _)| {
                surface == normalized && *limit == material_limit && *cached_mode == mode
            })
            .map(|(_, _, _, candidates)| Arc::clone(candidates))
    }) {
        return candidates;
    }
    if let Some(candidates) = projected_lexical_candidates(cache, normalized, material_limit, mode)
    {
        return candidates;
    }

    let mut candidates = Vec::new();
    if mode.includes_completion() {
        let exact_completions = memory.completion_candidates(
            normalized,
            material_limit,
            material_limit.saturating_mul(6),
        );
        let exact_prefix_count = exact_completions
            .iter()
            .filter(|candidate| candidate.word.starts_with(normalized))
            .count();
        candidates.extend(exact_completions);
        let exact_prefix_field_is_thin =
            exact_prefix_count < material_limit.min(THIN_EXACT_PREFIX_FIELD);
        if exact_prefix_field_is_thin {
            candidates.extend(memory.adjacent_transposition_candidates(normalized));
            candidates.extend(memory.surface_candidates(normalized, material_limit));
        }
        if should_probe_typo_tolerant_prefix(normalized, exact_prefix_count) {
            let fuzzy = projected_fuzzy_lexical_candidates(cache, normalized, material_limit, mode)
                .map(|candidates| candidates.as_ref().clone())
                .unwrap_or_else(|| {
                    typo_tolerant_completion_candidates(memory, normalized, material_limit)
                });
            candidates.extend(fuzzy);
        }
    } else {
        candidates.extend(memory.adjacent_transposition_candidates(normalized));
        candidates.extend(memory.surface_candidates(normalized, material_limit));
    }
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.rank.cmp(&right.rank))
            .then_with(|| left.word.cmp(&right.word))
    });
    candidates.dedup_by(|left, right| {
        if left.word != right.word {
            return false;
        }
        // A surface may be born both by the broad motif field and by the
        // explicit one-edit prefix traversal. Preserve the latter witness so
        // operator-conditioned reserves see the real birth topology.
        left.reconstructed |= right.reconstructed;
        left.prefix_match &= right.prefix_match;
        true
    });
    if std::env::var_os("LAY_L2_FIELD_TRACE").is_some() {
        eprintln!(
            "live_ime_lexical_trace token_chars={} reconstructed={:?}",
            normalized.chars().count(),
            candidates
                .iter()
                .filter(|candidate| candidate.reconstructed)
                .take(64)
                .map(|candidate| (&candidate.word, candidate.score, candidate.rank))
                .collect::<Vec<_>>()
        );
    }
    let candidates = Arc::new(candidates);

    store_lexical_candidates(
        cache,
        normalized,
        material_limit,
        mode,
        Arc::clone(&candidates),
    );
    candidates
}

fn should_probe_typo_tolerant_prefix(normalized: &str, exact_prefix_count: usize) -> bool {
    exact_prefix_count == 0
        && normalized.chars().count() >= TYPO_TOLERANT_PREFIX_MIN_CHARS
        && normalized.chars().all(is_cyrillic_letter)
}

fn typo_tolerant_completion_candidates(
    memory: &super::super::lexical_phase::LexicalPhaseMemory,
    damaged_prefix: &str,
    material_limit: usize,
) -> Vec<super::super::lexical_phase::LexicalPhaseCandidate> {
    let traversal_limit = material_limit
        .saturating_mul(TYPO_TOLERANT_MATERIAL_FACTOR)
        .min(TYPO_TOLERANT_MATERIAL_CAP)
        .max(material_limit);
    memory
        .one_edit_prefix_completion_candidates(damaged_prefix, traversal_limit, traversal_limit)
        .into_iter()
        // This lane is only for an unfinished token. Same-size and shorter
        // typo repairs stay on the Space/autocorrect route.
        .filter(|candidate| candidate.word.chars().count() > damaged_prefix.chars().count())
        .collect()
}

fn projected_fuzzy_lexical_candidates(
    cache: &Mutex<LexicalReadoutCache>,
    normalized: &str,
    material_limit: usize,
    mode: LexicalReadoutMode,
) -> Option<CachedLexicalCandidates> {
    let projected = cache.lock().ok().and_then(|cache| {
        cache
            .iter()
            .filter(|(surface, limit, cached_mode, _)| {
                *limit == material_limit
                    && *cached_mode == mode
                    && normalized.starts_with(surface)
                    && normalized.len() > surface.len()
            })
            .max_by_key(|(surface, _, _, _)| surface.len())
            .map(|(_, _, _, candidates)| {
                candidates
                    .iter()
                    .filter(|candidate| {
                        !candidate.word.starts_with(normalized)
                            && candidate.word.chars().count() > normalized.chars().count()
                            && prefix_is_one_edit_from_surface(normalized, &candidate.word)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
    })?;
    (!projected.is_empty()).then(|| Arc::new(projected))
}

fn prefix_is_one_edit_from_surface(prefix: &str, surface: &str) -> bool {
    let prefix_len = prefix.chars().count();
    let surface_chars = surface.chars().collect::<Vec<_>>();
    [
        prefix_len.saturating_sub(1),
        prefix_len,
        prefix_len.saturating_add(1),
    ]
    .into_iter()
    .filter(|candidate_len| *candidate_len >= 2 && *candidate_len <= surface_chars.len())
    .any(|candidate_len| {
        let surface_prefix = surface_chars[..candidate_len].iter().collect::<String>();
        damerau_levenshtein(prefix, &surface_prefix) == 1
    })
}

pub(super) fn warm_up_lexical_readout_cache(prefixes: &[String], material_limit: usize) {
    let memory = surface_motif_memory();
    for prefix in prefixes {
        if !is_supported_lexical_surface(prefix) {
            continue;
        }
        let _ =
            cached_lexical_candidates(memory, prefix, material_limit, LexicalReadoutMode::FullIme);
    }
}

/// Projects a previously settled L2 lattice into the next typed prefix. This
/// is a phase-field continuation, not a second prefix index: every returned
/// surface was already born by the same lexical centers. A thin projection
/// falls back to the full lattice so it cannot silently reduce coverage.
fn projected_lexical_candidates(
    cache: &Mutex<LexicalReadoutCache>,
    normalized: &str,
    material_limit: usize,
    mode: LexicalReadoutMode,
) -> Option<CachedLexicalCandidates> {
    let mut projected = cache.lock().ok().and_then(|cache| {
        cache
            .iter()
            .filter(|(surface, limit, cached_mode, _)| {
                *limit == material_limit
                    && *cached_mode == mode
                    && normalized.starts_with(surface)
                    && normalized.len() > surface.len()
            })
            .max_by_key(|(surface, _, _, _)| surface.len())
            .map(|(_, _, _, candidates)| {
                candidates
                    .iter()
                    .filter(|candidate| candidate.word.starts_with(normalized))
                    .cloned()
                    .collect::<Vec<_>>()
            })
    })?;
    let minimum = material_limit.min(12);
    if projected.len() < minimum {
        return None;
    }
    projected.truncate(material_limit);
    let projected = Arc::new(projected);
    store_lexical_candidates(
        cache,
        normalized,
        material_limit,
        mode,
        Arc::clone(&projected),
    );
    Some(projected)
}

fn store_lexical_candidates(
    cache: &Mutex<LexicalReadoutCache>,
    normalized: &str,
    material_limit: usize,
    mode: LexicalReadoutMode,
    candidates: CachedLexicalCandidates,
) {
    let Ok(mut cache) = cache.lock() else {
        return;
    };
    if let Some(index) = cache.iter().position(|(surface, limit, cached_mode, _)| {
        surface == normalized && *limit == material_limit && *cached_mode == mode
    }) {
        cache.remove(index);
    }
    if cache.len() >= LEXICAL_READOUT_CACHE_CAPACITY {
        cache.pop_front();
    }
    cache.push_back((normalized.to_string(), material_limit, mode, candidates));
}

pub(crate) fn l2_center_near_surfaces(text: &str, limit: usize) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized = text.to_lowercase();
    let len = normalized.chars().count();
    if !(3..=18).contains(&len) || !normalized.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }
    let generation = super::super::l2_field::candidate_material_generation();
    if let Some(surfaces) = readout_material_cache()
        .lock()
        .ok()
        .and_then(|mut cache| cache.get_near_surfaces(generation, &normalized, limit))
    {
        return surfaces.as_ref().clone();
    }
    let surfaces = surface_motif_memory()
        .surface_candidates(&normalized, limit.saturating_mul(8))
        .into_iter()
        .filter(|candidate| {
            let distance = damerau_levenshtein(&normalized, &candidate.word);
            (1..=3).contains(&distance)
                && len.abs_diff(candidate.word.chars().count()) <= 3
                && form_attractor_has_authority(
                    &normalized,
                    &candidate.word,
                    len,
                    distance,
                    candidate.score,
                )
        })
        .take(limit)
        .map(|candidate| candidate.word)
        .collect::<Vec<_>>();
    if super::super::l2_field::candidate_material_generation() != generation {
        return Vec::new();
    }
    if let Ok(mut cache) = readout_material_cache().lock() {
        cache.insert_near_surfaces(generation, &normalized, limit, Arc::new(surfaces.clone()));
    }
    surfaces
}

pub(crate) fn l2_center_contains_surface(word: &str) -> bool {
    surface_motif_memory().contains_surface(word)
}

pub(crate) fn l2_decoder_contains_surface(word: &str) -> bool {
    optional_decoder_contains_surface(super::super::lexical_phase::default_memory(), word)
}

fn optional_decoder_contains_surface(
    memory: Option<&super::super::lexical_phase::LexicalPhaseMemory>,
    word: &str,
) -> bool {
    memory.is_some_and(|memory| memory.contains_decoded_surface(word))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct L2SurfacePhaseReadout {
    pub(crate) exact_center: bool,
    pub(crate) l1_refs: usize,
    pub(crate) motif_refs: usize,
    pub(crate) covered_l1_refs: usize,
    pub(crate) residual_l1_refs: usize,
}

impl L2SurfacePhaseReadout {
    pub(crate) fn coherence_milli(self) -> u32 {
        if self.l1_refs == 0 {
            return 0;
        }
        ((self.covered_l1_refs.saturating_mul(1_000)) / self.l1_refs).min(u32::MAX as usize) as u32
    }
}

pub(crate) fn l2_surface_phase_readout(word: &str) -> L2SurfacePhaseReadout {
    let readout = surface_motif_memory().phase_readout(word);
    L2SurfacePhaseReadout {
        exact_center: readout.exact_center,
        l1_refs: readout.atom_count,
        motif_refs: readout.center_hits,
        covered_l1_refs: readout.center_hits,
        residual_l1_refs: readout.atom_count.saturating_sub(readout.center_hits),
    }
}

fn is_supported_lexical_surface(surface: &str) -> bool {
    !surface.is_empty()
        && (surface.chars().all(is_cyrillic_letter)
            || surface.chars().all(|ch| ch.is_ascii_alphabetic()))
}

fn same_lexical_script(left: &str, right: &str) -> bool {
    (left.chars().all(is_cyrillic_letter) && right.chars().all(is_cyrillic_letter))
        || (left.chars().all(|ch| ch.is_ascii_alphabetic())
            && right.chars().all(|ch| ch.is_ascii_alphabetic()))
}

fn sort_and_truncate_ime_l2_candidates(
    input: &str,
    candidates: &mut Vec<L2ImeWordCandidate>,
    limit: usize,
) {
    candidates.sort_by(|left, right| {
        l2_ime_word_candidate_operator_priority(input, right)
            .cmp(&l2_ime_word_candidate_operator_priority(input, left))
            .then_with(|| {
                l2_ime_word_candidate_score(right).cmp(&l2_ime_word_candidate_score(left))
            })
            .then_with(|| right.motif_overlap.cmp(&left.motif_overlap))
            .then_with(|| right.l2_overlap.cmp(&left.l2_overlap))
            .then_with(|| right.l1_overlap.cmp(&left.l1_overlap))
            .then_with(|| {
                left.surface
                    .chars()
                    .count()
                    .cmp(&right.surface.chars().count())
            })
            .then_with(|| left.surface.cmp(&right.surface))
    });
    let mut seen = HashSet::new();
    candidates.retain(|candidate| seen.insert(candidate.surface.clone()));
    if candidates.len() <= limit {
        return;
    }

    // Exact and typo-tolerant prefixes are independent birth lanes. A strong
    // corrected-prefix basin may reorder the field, but it must not evict the
    // entire exact-prefix lane before L3/L4 can compare their evidence.
    let exact_reserve_limit = limit.saturating_div(3).max(1);
    let corrected_reserve_limit = limit.saturating_div(2).max(1);
    let exact_reserve = candidates
        .iter()
        .filter(|candidate| {
            candidate.kind == L2ImeWordCandidateKind::Completion
                && candidate.surface.starts_with(input)
        })
        .take(exact_reserve_limit)
        .cloned()
        .collect::<Vec<_>>();
    let corrected_ranked = candidates
        .iter()
        .filter(|candidate| candidate.source == L2ImeWordCandidateSource::CorrectedPrefixPhase)
        .cloned()
        .collect::<Vec<_>>();
    let corrected_reserve =
        diverse_corrected_prefix_reserve(input, &corrected_ranked, corrected_reserve_limit);
    let ranked = std::mem::take(candidates);
    let mut bounded = Vec::with_capacity(limit);
    for candidate in exact_reserve
        .into_iter()
        .chain(corrected_reserve)
        .chain(ranked)
    {
        if bounded
            .iter()
            .any(|current: &L2ImeWordCandidate| current.surface == candidate.surface)
        {
            continue;
        }
        bounded.push(candidate);
        if bounded.len() == limit {
            break;
        }
    }
    bounded.sort_by(|left, right| {
        l2_ime_word_candidate_operator_priority(input, right)
            .cmp(&l2_ime_word_candidate_operator_priority(input, left))
            .then_with(|| {
                l2_ime_word_candidate_score(right).cmp(&l2_ime_word_candidate_score(left))
            })
            .then_with(|| right.motif_overlap.cmp(&left.motif_overlap))
            .then_with(|| right.l2_overlap.cmp(&left.l2_overlap))
            .then_with(|| right.l1_overlap.cmp(&left.l1_overlap))
            .then_with(|| {
                left.surface
                    .chars()
                    .count()
                    .cmp(&right.surface.chars().count())
            })
            .then_with(|| left.surface.cmp(&right.surface))
    });
    *candidates = bounded;
}

fn diverse_corrected_prefix_reserve(
    input: &str,
    candidates: &[L2ImeWordCandidate],
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    let mut selected = Vec::with_capacity(limit);
    let mut seen_slots = HashSet::new();
    for candidate in candidates {
        let novel = candidate
            .morphology_slots
            .iter()
            .copied()
            .any(|identity| !seen_slots.contains(&identity));
        if !novel {
            continue;
        }
        seen_slots.extend(candidate.morphology_slots.iter().copied());
        selected.push(candidate.clone());
        if selected.len() == limit {
            return selected;
        }
    }

    // Unbound lexical surfaces still need bounded diversity. This fallback is
    // topology-only and is never used when the package exposes a typed slot.
    let mut seen_endings = HashSet::new();
    for candidate in candidates {
        if !candidate.morphology_slots.is_empty()
            || selected
                .iter()
                .any(|current| current.surface == candidate.surface)
        {
            continue;
        }
        let Some(ending) = one_edit_prefix_ending(input, &candidate.surface) else {
            continue;
        };
        if seen_endings.insert(ending) {
            selected.push(candidate.clone());
            if selected.len() == limit {
                return selected;
            }
        }
    }
    for candidate in candidates {
        if selected
            .iter()
            .any(|current| current.surface == candidate.surface)
        {
            continue;
        }
        selected.push(candidate.clone());
        if selected.len() == limit {
            break;
        }
    }
    selected
}

fn one_edit_prefix_ending(input: &str, surface: &str) -> Option<String> {
    let input_len = input.chars().count();
    let surface_chars = surface.chars().collect::<Vec<_>>();
    [
        input_len.saturating_sub(1),
        input_len,
        input_len.saturating_add(1),
    ]
    .into_iter()
    .filter(|prefix_len| *prefix_len >= 2 && *prefix_len <= surface_chars.len())
    .find_map(|prefix_len| {
        let prefix = surface_chars[..prefix_len].iter().collect::<String>();
        (damerau_levenshtein(input, &prefix) == 1)
            .then(|| surface_chars[prefix_len..].iter().collect::<String>())
    })
}

fn l2_ime_word_candidate_operator_priority(input: &str, candidate: &L2ImeWordCandidate) -> u8 {
    crate::text_metrics::typed_damage_geometry_priority(input, &candidate.surface)
}

fn l2_ime_word_candidate_score(candidate: &L2ImeWordCandidate) -> u32 {
    let prior = ((candidate.usage_prior * 1600.0 + candidate.context_prior * 2600.0)
        .round()
        .clamp(0.0, 820.0) as u32)
        .saturating_add(candidate.accepted_count.min(40) * 18);
    let kind_bonus = match candidate.kind {
        L2ImeWordCandidateKind::AdjacentTransposition => 0,
        L2ImeWordCandidateKind::Completion => 80,
        L2ImeWordCandidateKind::Replacement => 0,
    };
    candidate
        .score
        .saturating_add(prior)
        .saturating_add(kind_bonus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_phase::LexicalPhaseCandidate;

    fn candidate(word: &str) -> LexicalPhaseCandidate {
        LexicalPhaseCandidate {
            word: word.to_string(),
            score: 1,
            l1_overlap: 1,
            l2_overlap: 1,
            motif_overlap: 1,
            prefix_match: true,
            rank: 0,
            phase_coherence_milli: 1_000,
            reconstructed: false,
        }
    }

    #[test]
    fn compact_correction_peak_material_matches_the_full_readout() {
        super::super::super::warm_up_l2_for_ime();
        for (context, token) in [
            ("", "звгрузи"),
            ("", "плозо"),
            ("нужно ", "обьяснить"),
            ("в ", "протколах"),
        ] {
            let compact = correction_l2_peak_candidates_impl(context, token, 16);
            let full = correction_l2_word_candidates_impl(context, token, 16);
            let compact_projection = compact
                .iter()
                .map(|candidate| {
                    (
                        candidate.surface.as_str(),
                        candidate.kind,
                        candidate.score,
                        candidate.l1_overlap,
                        candidate.l2_overlap,
                        candidate.motif_overlap,
                    )
                })
                .collect::<Vec<_>>();
            let full_projection = full
                .iter()
                .map(|candidate| {
                    (
                        candidate.surface.as_str(),
                        candidate.kind,
                        candidate.score,
                        candidate.l1_overlap,
                        candidate.l2_overlap,
                        candidate.motif_overlap,
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(compact_projection, full_projection, "token={token:?}");
        }
    }

    #[test]
    fn correction_rank_keys_match_the_original_direct_calculations() {
        super::super::super::warm_up_l2_for_ime();
        let normalized = "звгрузи";
        let candidates = ranked_correction_candidates("", normalized, 16);
        assert!(!candidates.is_empty());
        for candidate in candidates {
            assert_eq!(
                candidate.geometry_priority,
                crate::text_metrics::typed_damage_geometry_priority(
                    normalized,
                    &candidate.peak.surface,
                )
            );
            assert_eq!(
                candidate.rank_score,
                correction_candidate_rank_score(
                    candidate.peak.score,
                    candidate.usage_prior,
                    candidate.context_prior,
                    candidate.accepted_count,
                )
            );
            assert_eq!(
                candidate.surface_char_len,
                candidate.peak.surface.chars().count()
            );
        }
    }

    fn ime_candidate(
        surface: String,
        source: L2ImeWordCandidateSource,
        score: u32,
    ) -> L2ImeWordCandidate {
        L2ImeWordCandidate {
            surface,
            kind: if source == L2ImeWordCandidateSource::CorrectedPrefixPhase {
                L2ImeWordCandidateKind::Replacement
            } else {
                L2ImeWordCandidateKind::Completion
            },
            source,
            score,
            l1_overlap: 4,
            l2_overlap: 4,
            motif_overlap: 2,
            usage_prior: 0.0,
            context_prior: 0.0,
            accepted_count: 0,
            target_evidence: Default::default(),
            morphology_slots: Vec::new(),
        }
    }

    #[test]
    fn boundary_readout_cache_reuses_positive_and_empty_material() {
        let mut cache = ReadoutMaterialCache::new(2, 2);
        let positive = vec![ime_candidate(
            "ты почитай".to_string(),
            L2ImeWordCandidateSource::BoundaryPhase,
            1_584,
        )];
        cache.insert_boundary_candidates(7, "", "тыпочитай", 2, Arc::new(positive.clone()));
        assert_eq!(
            cache
                .get_boundary_candidates(7, "", "тыпочитай", 2)
                .expect("positive boundary cache hit")
                .as_ref(),
            &positive
        );

        cache.insert_boundary_candidates(7, "", "безграницы", 2, Arc::new(Vec::new()));
        assert!(cache
            .get_boundary_candidates(7, "", "безграницы", 2)
            .expect("empty boundary cache hit")
            .is_empty());
    }

    #[test]
    fn boundary_readout_cache_key_binds_context_token_case_and_limit() {
        let mut cache = ReadoutMaterialCache::new(2, 4);
        cache.insert_boundary_candidates(
            11,
            "перед ",
            "Словами",
            2,
            Arc::new(vec![ime_candidate(
                "С ловами".to_string(),
                L2ImeWordCandidateSource::BoundaryPhase,
                1_500,
            )]),
        );

        assert!(cache
            .get_boundary_candidates(11, "другой ", "Словами", 2)
            .is_none());
        assert!(cache
            .get_boundary_candidates(11, "перед ", "словами", 2)
            .is_none());
        assert!(cache
            .get_boundary_candidates(11, "перед ", "Словами", 1)
            .is_none());
        assert!(cache
            .get_boundary_candidates(11, "перед ", "Словами", 2)
            .is_some());
    }

    #[test]
    fn boundary_readout_cache_is_lru_bounded_and_generation_scoped() {
        let mut cache = ReadoutMaterialCache::new(1, 2);
        for token in ["первый", "второй"] {
            cache.insert_boundary_candidates(3, "", token, 2, Arc::new(Vec::new()));
        }
        assert!(cache.get_boundary_candidates(3, "", "первый", 2).is_some());
        cache.insert_boundary_candidates(3, "", "третий", 2, Arc::new(Vec::new()));
        assert_eq!(cache.boundary_candidates.len(), 2);
        assert!(cache.get_boundary_candidates(3, "", "второй", 2).is_none());
        assert!(cache.get_boundary_candidates(3, "", "первый", 2).is_some());

        cache.insert_boundary_candidates(4, "", "новый", 2, Arc::new(Vec::new()));
        assert_eq!(cache.boundary_candidates.len(), 1);
        assert!(cache.get_boundary_candidates(3, "", "первый", 2).is_none());
        assert!(cache.get_boundary_candidates(4, "", "новый", 2).is_some());
    }

    #[test]
    fn boundary_readout_cache_hit_rejects_generation_turnover() {
        use std::cell::Cell;

        let cache = Mutex::new(ReadoutMaterialCache::new(1, 1));
        cache
            .lock()
            .expect("local cache")
            .insert_boundary_candidates(
                7,
                "",
                "generation-turnover",
                2,
                Arc::new(vec![ime_candidate(
                    "stale".to_string(),
                    L2ImeWordCandidateSource::BoundaryPhase,
                    1_000,
                )]),
            );
        let generation_reads = Cell::new(0usize);
        let candidates = cached_boundary_candidates_with_generation(
            &cache,
            "",
            "generation-turnover",
            2,
            || {
                let read = generation_reads.get();
                generation_reads.set(read + 1);
                if read == 0 {
                    7
                } else {
                    8
                }
            },
            || panic!("a cache hit must not recompute material"),
        );

        assert_eq!(generation_reads.get(), 2);
        assert!(
            candidates.is_empty(),
            "a generation change between hit lookup and return must fail closed"
        );
    }

    #[test]
    fn live_boundary_readout_reuses_one_material_computation() {
        use std::cell::Cell;

        let calls = Cell::new(0usize);
        let first =
            cached_boundary_candidates("td113-cache-context", "TD113-cache-token", 2, || {
                calls.set(calls.get() + 1);
                vec![ime_candidate(
                    "cached".to_string(),
                    L2ImeWordCandidateSource::BoundaryPhase,
                    1_000,
                )]
            });
        let second =
            cached_boundary_candidates("td113-cache-context", "TD113-cache-token", 2, || {
                calls.set(calls.get() + 1);
                Vec::new()
            });

        assert_eq!(calls.get(), 1);
        assert_eq!(second, first);
    }

    #[test]
    fn corrected_prefix_rank_cannot_erase_exact_prefix_birth_lane() {
        let mut candidates = (0..12)
            .map(|index| {
                ime_candidate(
                    format!("corrected{index}"),
                    L2ImeWordCandidateSource::CorrectedPrefixPhase,
                    2_000 - index,
                )
            })
            .chain((0..6).map(|index| {
                ime_candidate(
                    format!("prefixexact{index}"),
                    L2ImeWordCandidateSource::LexicalPhase,
                    1_000 - index,
                )
            }))
            .collect::<Vec<_>>();

        sort_and_truncate_ime_l2_candidates("prefix", &mut candidates, 6);

        assert_eq!(candidates.len(), 6);
        assert!(
            candidates
                .iter()
                .filter(|candidate| candidate.surface.starts_with("prefix"))
                .count()
                >= 2,
            "candidates={candidates:?}"
        );
        assert!(candidates.iter().any(|candidate| {
            candidate.source == L2ImeWordCandidateSource::CorrectedPrefixPhase
        }));
    }

    #[test]
    fn one_edit_prefix_field_keeps_ranked_missing_letter_basin() {
        super::super::super::warm_up_l2_for_ime();
        let candidates =
            surface_motif_memory().one_edit_prefix_completion_candidates("предскз", 24, 96);

        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word.starts_with("предсказ")),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn exact_prefix_field_suppresses_only_the_redundant_fuzzy_fallback() {
        assert!(!should_probe_typo_tolerant_prefix("оста", 1));
        assert!(should_probe_typo_tolerant_prefix("предскз", 0));
        assert!(!should_probe_typo_tolerant_prefix("пр", 0));
        assert!(!should_probe_typo_tolerant_prefix("abc", 0));
    }

    #[test]
    fn one_edit_prefix_field_keeps_infrequent_form_inside_bounded_material() {
        super::super::super::warm_up_l2_for_ime();
        let candidates =
            surface_motif_memory().one_edit_prefix_completion_candidates("переспективн", 512, 512);
        let rank = candidates
            .iter()
            .position(|candidate| candidate.word == "перспективнее");
        eprintln!(
            "перспективнее fuzzy material rank={rank:?} count={}",
            candidates.len()
        );
        assert!(rank.is_some(), "candidates={candidates:?}");
    }

    #[test]
    fn corrected_prefix_frontier_contains_attested_comparative() {
        super::super::super::warm_up_l2_for_ime();
        let candidates = surface_motif_memory().completion_candidates("персп", 96, 576);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "перспективнее"),
            "персп frontier: {:?}",
            candidates
                .iter()
                .map(|candidate| candidate.word.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn projects_settled_lattice_only_when_the_next_prefix_stays_dense() {
        let cache = Mutex::new(VecDeque::new());
        store_lexical_candidates(
            &cache,
            "оста",
            3,
            LexicalReadoutMode::FullIme,
            Arc::new(vec![
                candidate("остановка"),
                candidate("остановить"),
                candidate("остановлю"),
            ]),
        );

        let projected =
            projected_lexical_candidates(&cache, "остан", 3, LexicalReadoutMode::FullIme)
                .expect("dense continuation must reuse the already born lattice");
        assert!(projected.iter().all(|item| item.word.starts_with("остан")));

        assert!(
            projected_lexical_candidates(&cache, "остановк", 3, LexicalReadoutMode::FullIme)
                .is_none()
        );
    }

    #[test]
    fn missing_decoder_artifact_cannot_crash_feedback_admission() {
        assert!(!optional_decoder_contains_surface(None, "прекрасно"));
    }

    #[test]
    fn near_surface_material_cache_is_lru_bounded() {
        let mut cache = ReadoutMaterialCache::new(2, 1);
        cache.insert_near_surfaces(7, "один", 32, Arc::new(vec!["одна".to_string()]));
        cache.insert_near_surfaces(7, "два", 32, Arc::new(vec!["две".to_string()]));
        assert!(cache.get_near_surfaces(7, "один", 32).is_some());
        cache.insert_near_surfaces(7, "три", 32, Arc::new(Vec::new()));

        assert_eq!(cache.near_surfaces.len(), 2);
        assert!(cache.get_near_surfaces(7, "один", 32).is_some());
        assert!(cache.get_near_surfaces(7, "два", 32).is_none());
        assert!(cache.get_near_surfaces(7, "три", 32).is_some());
    }

    #[test]
    fn near_surface_material_cache_drops_previous_generation() {
        let mut cache = ReadoutMaterialCache::new(4, 1);
        cache.insert_near_surfaces(3, "форма", 32, Arc::new(vec!["формы".to_string()]));
        cache.insert_near_surfaces(4, "форма", 32, Arc::new(Vec::new()));

        assert_eq!(cache.near_surfaces.len(), 1);
        assert!(cache.get_near_surfaces(3, "форма", 32).is_none());
        assert!(cache
            .get_near_surfaces(4, "форма", 32)
            .is_some_and(|surfaces| surfaces.is_empty()));
    }
}
