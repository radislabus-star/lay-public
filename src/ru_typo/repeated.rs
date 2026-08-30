use crate::phrase_lexicon::is_short_russian_function_word;
use crate::russian_chars::is_russian_vowel;
use crate::russian_typo_candidates::repeated_run_deletion_candidates;
use crate::russian_typo_scoring::best_ranked_dictionary_candidate;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::memo::{memoized_text, WordMaterialKind};
use super::thresholds::NGRAM_TYPO_REJECT_MARGIN;

pub(crate) fn correct_repeated_letter(word: &str) -> Option<String> {
    memoized_text(WordMaterialKind::RepeatedLetter, word, || {
        select_repeated_letter_candidate(word, RepeatedLetterAuthority::Autocorrect)
    })
}

pub(crate) fn propose_repeated_letter_candidate(word: &str) -> Option<String> {
    select_repeated_letter_candidate(word, RepeatedLetterAuthority::ProposalOnly)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RepeatedLetterAuthority {
    Autocorrect,
    ProposalOnly,
}

fn select_repeated_letter_candidate(
    word: &str,
    authority: RepeatedLetterAuthority,
) -> Option<String> {
    if !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if crate::nanda_wave::l2::l2_surface_foundation_has_authority(&lower)
        || crate::russian_lexicon::is_exact_reference_russian_word(&lower)
        || crate::lexicon::is_ru_live_protected_word(&lower)
        || crate::lexicon::is_user_protected_word(&lower)
    {
        return None;
    }
    if let Some(candidate) = correct_short_repeated_function_word(word, &lower) {
        return Some(candidate);
    }
    if word.chars().count() < 5 {
        return None;
    }
    let chars = lower.chars().collect::<Vec<_>>();
    if chars.len() >= 2
        && chars[chars.len() - 1] == chars[chars.len() - 2]
        && is_russian_vowel(chars[chars.len() - 1])
    {
        return None;
    }

    let mut candidates = repeated_run_deletion_candidates(&lower)
        .into_iter()
        .filter(|candidate| {
            authority == RepeatedLetterAuthority::ProposalOnly
                || repeated_letter_autocorrect_has_authority(candidate)
        })
        .collect::<Vec<_>>();
    candidates.sort_unstable();
    candidates.dedup();
    best_ranked_dictionary_candidate(word, candidates, NGRAM_TYPO_REJECT_MARGIN, 0.40)
}

fn repeated_letter_autocorrect_has_authority(candidate: &str) -> bool {
    crate::russian_lexicon::is_exact_reference_russian_word(candidate)
        || crate::russian_lexicon::is_center_backed_russian_form(candidate)
        || crate::russian_lexicon::is_reference_backed_short_passive_participle(candidate)
        || crate::lexicon::is_common_ru_word(candidate)
}

#[cfg(test)]
mod tests {
    use super::{correct_repeated_letter, propose_repeated_letter_candidate};

    #[test]
    fn repeated_letter_requires_autocorrect_authority_for_the_replacement() {
        assert_eq!(correct_repeated_letter("русским"), None);
        assert_eq!(correct_repeated_letter("медленно"), None);
        assert_eq!(
            correct_repeated_letter("исправленно").as_deref(),
            Some("исправлено")
        );
        assert_eq!(
            correct_repeated_letter("исправленнно").as_deref(),
            Some("исправлено")
        );
        assert_eq!(
            correct_repeated_letter("ОФФИЦИАЛЬНОМ").as_deref(),
            Some("ОФИЦИАЛЬНОМ")
        );
    }

    #[test]
    fn repeated_candidate_supports_a_split_word_merge() {
        assert_eq!(
            correct_repeated_letter("печатаеттся").as_deref(),
            Some("печатается")
        );
        assert_eq!(
            propose_repeated_letter_candidate("печатаеттся").as_deref(),
            Some("печатается")
        );
    }
}

fn correct_short_repeated_function_word(original: &str, lower: &str) -> Option<String> {
    if !(3..=4).contains(&lower.chars().count()) {
        return None;
    }

    let mut found: Option<String> = None;
    for candidate in repeated_run_deletion_candidates(lower) {
        if candidate.chars().count() < 2 {
            continue;
        }
        if !is_short_russian_function_word(&candidate) {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(original, &candidate))
}
