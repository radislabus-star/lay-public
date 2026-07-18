use super::{
    form_attractor_has_authority, surface_motif_memory, L2ImeWordCandidate, L2ImeWordCandidateKind,
    L2ImeWordCandidateSource,
};
use crate::keyboard::is_cyrillic_letter;
use crate::text_metrics::damerau_levenshtein;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};

const LEXICAL_READOUT_CACHE_CAPACITY: usize = 256;
type CachedLexicalCandidates = Arc<Vec<super::super::lexical_phase::LexicalPhaseCandidate>>;
type LexicalReadoutCache = VecDeque<(String, usize, bool, CachedLexicalCandidates)>;

static LEXICAL_READOUT_CACHE: OnceLock<Mutex<LexicalReadoutCache>> = OnceLock::new();

pub(super) fn ime_l2_word_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    l2_word_candidates_impl(context_prefix, token, limit, true)
}

pub(super) fn correction_l2_word_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
) -> Vec<L2ImeWordCandidate> {
    l2_word_candidates_impl(context_prefix, token, limit, false)
}

fn l2_word_candidates_impl(
    context_prefix: &str,
    token: &str,
    limit: usize,
    include_completion: bool,
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
    let material_limit = limit.saturating_mul(8).max(limit);
    let lexical =
        cached_lexical_candidates(memory, &normalized, material_limit, include_completion);
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
                source: L2ImeWordCandidateSource::LexicalPhase,
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
    sort_and_truncate_ime_l2_candidates(&mut candidates, limit);
    candidates
}

fn cached_lexical_candidates(
    memory: &super::super::lexical_phase::LexicalPhaseMemory,
    normalized: &str,
    material_limit: usize,
    include_completion: bool,
) -> CachedLexicalCandidates {
    let cache = LEXICAL_READOUT_CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    if let Some(candidates) = cache.lock().ok().and_then(|cache| {
        cache
            .iter()
            .find(|(surface, limit, completion, _)| {
                surface == normalized
                    && *limit == material_limit
                    && *completion == include_completion
            })
            .map(|(_, _, _, candidates)| Arc::clone(candidates))
    }) {
        return candidates;
    }

    let mut candidates = memory.adjacent_transposition_candidates(normalized);
    candidates.extend(memory.surface_candidates(normalized, material_limit));
    if include_completion {
        candidates.extend(memory.completion_candidates(
            normalized,
            material_limit,
            material_limit.saturating_mul(6),
        ));
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

    if let Ok(mut cache) = cache.lock() {
        if cache.len() >= LEXICAL_READOUT_CACHE_CAPACITY {
            cache.pop_front();
        }
        cache.push_back((
            normalized.to_string(),
            material_limit,
            include_completion,
            Arc::clone(&candidates),
        ));
    }
    candidates
}

pub(crate) fn ime_l2_completion_candidates(
    context_prefix: &str,
    token: &str,
    limit: usize,
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
    let material_limit = limit.saturating_mul(8).max(limit);
    let lexical = surface_motif_memory()
        .completion_candidates(&normalized, material_limit, material_limit)
        .into_iter()
        .filter(|candidate| same_lexical_script(&normalized, &candidate.word));
    let mut candidates = lexical
        .into_iter()
        .map(|candidate| L2ImeWordCandidate {
            surface: candidate.word,
            kind: L2ImeWordCandidateKind::Completion,
            source: L2ImeWordCandidateSource::LexicalPhase,
            score: candidate.score,
            l1_overlap: candidate.l1_overlap,
            l2_overlap: candidate.l2_overlap,
            motif_overlap: candidate.motif_overlap,
            usage_prior: 0.0,
            context_prior: 0.0,
            accepted_count: 0,
        })
        .collect::<Vec<_>>();
    for candidate in &mut candidates {
        let prior = usage.candidate_prior_prepared(&usage_context, &candidate.surface);
        candidate.usage_prior = prior.word_prior;
        candidate.context_prior = prior.context_prior;
        candidate.accepted_count = prior.accepted_count;
    }
    sort_and_truncate_ime_l2_candidates(&mut candidates, limit);
    candidates
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
    surface_motif_memory().contains_decoded_surface(word)
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

fn sort_and_truncate_ime_l2_candidates(candidates: &mut Vec<L2ImeWordCandidate>, limit: usize) {
    candidates.sort_by(|left, right| {
        l2_ime_word_candidate_operator_priority(right)
            .cmp(&l2_ime_word_candidate_operator_priority(left))
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
    candidates.dedup_by(|left, right| left.surface == right.surface);
    candidates.truncate(limit);
}

fn l2_ime_word_candidate_operator_priority(candidate: &L2ImeWordCandidate) -> u8 {
    u8::from(candidate.kind == L2ImeWordCandidateKind::AdjacentTransposition)
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
