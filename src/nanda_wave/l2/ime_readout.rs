use super::{
    form_attractor_has_authority, surface_motif_memory, L2ImeWordCandidate, L2ImeWordCandidateKind,
    L2ImeWordCandidateSource,
};
use crate::keyboard::is_cyrillic_letter;
use crate::text_metrics::damerau_levenshtein;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

const LEXICAL_READOUT_CACHE_CAPACITY: usize = 128;
// The gate asks L2 for a bounded material lattice and then applies phase
// competition. Four candidates per requested display slot leave enough
// competitors for interference while avoiding a 192-node DAFSA walk per key.
const IME_L2_MATERIAL_FACTOR: usize = 4;
const TYPO_TOLERANT_PREFIX_MIN_CHARS: usize = 7;
type CachedLexicalCandidates = Arc<Vec<super::super::lexical_phase::LexicalPhaseCandidate>>;
type LexicalReadoutCache = VecDeque<(String, usize, LexicalReadoutMode, CachedLexicalCandidates)>;

static LEXICAL_READOUT_CACHE: OnceLock<Mutex<LexicalReadoutCache>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LexicalReadoutMode {
    CompletionOnly,
    FullIme,
    Correction,
}

impl LexicalReadoutMode {
    const fn includes_completion(self) -> bool {
        matches!(self, Self::CompletionOnly | Self::FullIme)
    }
}

pub(super) fn ime_l2_word_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    l2_word_candidates_impl(context_prefix, token, limit, LexicalReadoutMode::FullIme)
}

pub(super) fn ime_l2_completion_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    l2_word_candidates_impl(
        context_prefix,
        token,
        limit,
        LexicalReadoutMode::CompletionOnly,
    )
}

pub(super) fn correction_l2_word_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    l2_word_candidates_impl(context_prefix, token, limit, LexicalReadoutMode::Correction)
}

fn l2_word_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
    mode: LexicalReadoutMode,
) -> Vec<L2ImeWordCandidate> {
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
            let candidate_len = candidate.word.chars().count();
            let corrected_prefix = mode == LexicalReadoutMode::CompletionOnly
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
            }
        })
        .collect::<Vec<_>>();
    sort_and_truncate_ime_l2_candidates(&normalized, &mut candidates, limit);
    candidates
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

    let mut candidates = match mode {
        LexicalReadoutMode::CompletionOnly => Vec::new(),
        LexicalReadoutMode::FullIme | LexicalReadoutMode::Correction => {
            let mut candidates = memory.adjacent_transposition_candidates(normalized);
            candidates.extend(memory.surface_candidates(normalized, material_limit));
            candidates
        }
    };
    if mode.includes_completion() {
        candidates.extend(memory.completion_candidates(
            normalized,
            material_limit,
            material_limit.saturating_mul(6),
        ));
        if normalized.chars().count() >= TYPO_TOLERANT_PREFIX_MIN_CHARS
            && normalized.chars().all(is_cyrillic_letter)
        {
            candidates.extend(typo_tolerant_completion_candidates(
                memory,
                normalized,
                material_limit,
            ));
        }
    }
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.rank.cmp(&right.rank))
            .then_with(|| left.word.cmp(&right.word))
    });
    candidates.dedup_by(|left, right| left.word == right.word);
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

fn typo_tolerant_completion_candidates(
    memory: &super::super::lexical_phase::LexicalPhaseMemory,
    damaged_prefix: &str,
    material_limit: usize,
) -> Vec<super::super::lexical_phase::LexicalPhaseCandidate> {
    memory
        .one_edit_prefix_completion_candidates(damaged_prefix, material_limit, material_limit)
        .into_iter()
        // This lane is only for an unfinished token. Same-size and shorter
        // typo repairs stay on the Space/autocorrect route.
        .filter(|candidate| candidate.word.chars().count() > damaged_prefix.chars().count())
        .collect()
}

pub(super) fn warm_up_lexical_readout_cache(prefixes: &[String], material_limit: usize) {
    let memory = surface_motif_memory();
    for prefix in prefixes {
        if !is_supported_lexical_surface(prefix) {
            continue;
        }
        let _ = cached_lexical_candidates(
            memory,
            prefix,
            material_limit,
            LexicalReadoutMode::CompletionOnly,
        );
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
    surface_motif_memory()
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
        .collect()
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
    let corrected_prefix_reserve = candidates
        .iter()
        .filter(|candidate| candidate.source == L2ImeWordCandidateSource::CorrectedPrefixPhase)
        .take(8)
        .cloned()
        .collect::<Vec<_>>();
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
    candidates.truncate(limit);
    for candidate in corrected_prefix_reserve {
        if candidates
            .iter()
            .any(|current| current.surface == candidate.surface)
        {
            continue;
        }
        if candidates.len() == limit {
            candidates.pop();
        }
        candidates.push(candidate);
    }
}

fn l2_ime_word_candidate_operator_priority(input: &str, candidate: &L2ImeWordCandidate) -> u8 {
    if candidate.kind == L2ImeWordCandidateKind::AdjacentTransposition {
        return 4;
    }
    if crate::text_metrics::sparse_internal_omission_count(input, &candidate.surface).is_some() {
        return 3;
    }
    if single_missing_letter_shape(input, &candidate.surface) {
        return 2;
    }
    0
}

fn single_missing_letter_shape(input: &str, candidate: &str) -> bool {
    candidate.chars().count() == input.chars().count() + 1
        && damerau_levenshtein(input, candidate) == 1
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
            LexicalReadoutMode::CompletionOnly,
            Arc::new(vec![
                candidate("остановка"),
                candidate("остановить"),
                candidate("остановлю"),
            ]),
        );

        let projected =
            projected_lexical_candidates(&cache, "остан", 3, LexicalReadoutMode::CompletionOnly)
                .expect("dense continuation must reuse the already born lattice");
        assert!(projected.iter().all(|item| item.word.starts_with("остан")));

        assert!(projected_lexical_candidates(
            &cache,
            "остановк",
            3,
            LexicalReadoutMode::CompletionOnly
        )
        .is_none());
    }

    #[test]
    fn missing_decoder_artifact_cannot_crash_feedback_admission() {
        assert!(!optional_decoder_contains_surface(None, "прекрасно"));
    }
}
