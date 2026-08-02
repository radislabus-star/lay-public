//! Scoring helpers for Russian typo candidates.
//!
//! Correction rules generate candidates; this module owns the repeated
//! dictionary + n-gram confidence checks.

use std::collections::HashSet;

use crate::candidate_ranker::choose_best_with_gap;
use crate::russian_chars::is_russian_vowel;
use crate::russian_lexicon::{
    is_known_russian_word_or_form, is_reference_backed_russian_form, russian_dictionary,
    russian_short_dictionary,
};
use crate::russian_typo_candidates::inserted_char_position_for_missing_letter;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

pub(crate) fn ngram_allows_ru_candidate(candidate: &str, baseline: &str, min_margin: f64) -> bool {
    crate::ngram::ru_candidate_margin(candidate, baseline) >= min_margin
}

pub(crate) fn best_ranked_dictionary_candidate<I>(
    original: &str,
    candidates: I,
    min_margin: f64,
    min_gap: f64,
) -> Option<String>
where
    I: IntoIterator<Item = String>,
{
    let lower = original.to_lowercase();
    let (candidate, _) = choose_best_with_gap(candidates, min_gap, |candidate| {
        if std::env::var_os("LAY_TRACE_RU_TYPO").is_some() && candidate.contains("свойств") {
            eprintln!(
                "rank_candidate original={lower:?} candidate={candidate:?} runtime={} foundation={} exact_reference={} morphology={}",
                is_known_russian_word_or_form(candidate),
                crate::nanda_wave::l2::l2_surface_foundation_contains(candidate),
                crate::russian_lexicon::is_exact_reference_russian_word(candidate),
                is_reference_backed_russian_form(candidate)
            );
        }
        if candidate == &lower || !is_rankable_russian_candidate(candidate) {
            return None;
        }
        let margin = crate::ngram::ru_candidate_margin(candidate, &lower);
        let common_bonus = if crate::lexicon::is_common_ru_word(candidate) {
            12.0
        } else {
            0.0
        };
        let score = margin + missing_letter_candidate_bonus(&lower, candidate) + common_bonus;
        if is_reference_only_candidate(candidate) && score < 0.0 {
            return None;
        }
        if score < min_margin {
            return None;
        }
        Some(score)
    })?;
    Some(apply_word_case(original, &candidate))
}

pub(crate) fn best_unique_known_ngram_candidate<I>(
    original: &str,
    candidates: I,
    min_margin: f64,
) -> Option<String>
where
    I: IntoIterator<Item = String>,
{
    let lower = original.to_lowercase();
    let mut seen = HashSet::new();
    let mut found: Option<String> = None;

    for candidate in candidates {
        if candidate == lower || !seen.insert(candidate.clone()) {
            continue;
        }
        if !is_cyrillic_word(&candidate) || !is_rankable_russian_candidate(&candidate) {
            continue;
        }

        let margin = crate::ngram::ru_candidate_margin(&candidate, &lower);
        if is_reference_only_candidate(&candidate) && margin < 0.0 {
            continue;
        }
        if margin < min_margin {
            continue;
        }

        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(original, &candidate))
}

pub(crate) fn missing_letter_candidate_bonus(lower: &str, candidate: &str) -> f64 {
    let Some((_, inserted)) = inserted_char_position_for_missing_letter(lower, candidate) else {
        return 0.0;
    };
    let center_support = if crate::nanda_wave::l2::l2_surface_foundation_has_authority(candidate)
        || crate::russian_lexicon::is_center_backed_russian_form(candidate)
        || is_reference_backed_russian_form(candidate)
    {
        12.0
    } else {
        0.0
    };
    center_support
        + if is_russian_vowel(inserted) {
            4.0
        } else if inserted == 'й' && looks_like_known_y_noun_form(candidate) {
            12.0
        } else if inserted == 'й' {
            2.0
        } else {
            0.0
        }
}

fn is_rankable_russian_candidate(candidate: &str) -> bool {
    is_known_russian_word_or_form(candidate)
        || crate::nanda_wave::l2::l2_surface_foundation_contains(candidate)
        || crate::russian_lexicon::is_exact_reference_russian_word(candidate)
        || is_reference_backed_russian_form(candidate)
}

fn is_reference_only_candidate(candidate: &str) -> bool {
    !is_known_russian_word_or_form(candidate)
        && !crate::nanda_wave::l2::l2_surface_foundation_contains(candidate)
        && !crate::russian_lexicon::is_exact_reference_russian_word(candidate)
        && is_reference_backed_russian_form(candidate)
}

fn looks_like_known_y_noun_form(candidate: &str) -> bool {
    ["ом", "ем", "а", "у", "ы", "е"].into_iter().any(|suffix| {
        let Some(stem) = candidate.strip_suffix(suffix) else {
            return false;
        };
        stem.chars().count() >= 4
            && stem.contains('й')
            && (russian_dictionary().contains(stem) || russian_short_dictionary().contains(stem))
    })
}
