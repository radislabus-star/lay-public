use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::russian_typo_scoring::ngram_allows_ru_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::guards::looks_like_known_word_plus_one_letter_function_suffix;
use super::thresholds::NGRAM_TRANSPOSE_MARGIN;

pub(crate) fn correct_adjacent_transposition(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }
    let lower = word.to_lowercase();
    if crate::russian_lexicon::has_clean_russian_surface_certificate(&lower)
        || crate::lexicon::is_ru_live_protected_word(&lower)
        || crate::nanda_wave::l2::l2_surface_foundation_has_authority(&lower)
    {
        return None;
    }

    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;
    for idx in 0..chars.len().saturating_sub(1) {
        if chars[idx] == chars[idx + 1] {
            continue;
        }

        let mut candidate = chars.clone();
        candidate.swap(idx, idx + 1);
        let candidate: String = candidate.into_iter().collect();
        let candidate_known = is_known_russian_word_or_form(&candidate)
            || crate::nanda_wave::l2::l2_surface_foundation_contains(&candidate)
            || crate::russian_lexicon::is_exact_reference_russian_word(&candidate);
        if std::env::var_os("LAY_TRACE_RU_TYPO").is_some() {
            eprintln!(
                "adjacent_transposition original={lower:?} candidate={candidate:?} known={} margin={:.3}",
                candidate_known,
                crate::ngram::ru_candidate_margin(&candidate, &lower)
            );
        }
        if !candidate_known {
            continue;
        }
        if looks_like_known_word_plus_one_letter_function_suffix(&candidate) {
            continue;
        }
        if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_TRANSPOSE_MARGIN) {
            continue;
        }

        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}
