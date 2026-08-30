use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::text_case::apply_word_case;
use crate::word_reader::is_cyrillic_word;

use super::keyboard::are_ru_keyboard_neighbors;
use super::memo::{memoized_text, WordMaterialKind};

pub(crate) fn correct_single_letter_substitution(word: &str) -> Option<String> {
    memoized_text(WordMaterialKind::SingleLetterSubstitution, word, || {
        select_single_letter_substitution(word, SubstitutionAuthority::Autocorrect)
    })
}

pub(crate) fn propose_single_letter_substitution_candidate(word: &str) -> Option<String> {
    select_single_letter_substitution(word, SubstitutionAuthority::ProposalOnly)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SubstitutionAuthority {
    Autocorrect,
    ProposalOnly,
}

fn select_single_letter_substitution(
    word: &str,
    authority: SubstitutionAuthority,
) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if authority == SubstitutionAuthority::Autocorrect && is_known_russian_word_or_form(&lower) {
        return None;
    }

    let (candidate, _) = crate::candidate_ranker::choose_best_with_gap(
        crate::nanda_wave::l2::l2_center_near_surfaces(&lower, 64),
        0.40,
        |candidate| {
            let shape_is_allowed = if authority == SubstitutionAuthority::Autocorrect {
                safe_neighbor_substitution_candidate(&lower, candidate)
            } else {
                safe_single_substitution_candidate(&lower, candidate)
            };
            if !shape_is_allowed {
                return None;
            }
            let center_prior = crate::nanda_wave::l2::l2_surface_foundation_rank(candidate)
                .map(|rank| 12.0 / (1.0 + rank as f64 / 2_000.0))
                .unwrap_or(0.0);
            let margin = crate::ngram::ru_candidate_margin(candidate, &lower);
            if std::env::var_os("LAY_TRACE_RU_TYPO").is_some() {
                eprintln!(
                    "neighbor_substitution original={lower:?} candidate={candidate:?} margin={margin:.3} center_prior={center_prior:.3}"
                );
            }
            let proposal_only_support = authority == SubstitutionAuthority::ProposalOnly
                && margin >= 0.0
                && (is_known_russian_word_or_form(candidate)
                    || crate::nanda_wave::l2::l2_decoder_contains_surface(candidate));
            (center_prior > 0.0 || proposal_only_support).then_some(margin + center_prior)
        },
    )?;
    Some(apply_word_case(word, &candidate))
}

#[cfg(test)]
mod tests {
    use super::{correct_single_letter_substitution, propose_single_letter_substitution_candidate};

    #[test]
    fn proposal_only_authority_exposes_a_boundary_split_competitor() {
        assert_eq!(correct_single_letter_substitution("парочинная"), None);
        assert_eq!(
            propose_single_letter_substitution_candidate("парочинная").as_deref(),
            Some("перочинная")
        );
    }
}

fn safe_neighbor_substitution_candidate(original: &str, candidate: &str) -> bool {
    if !is_known_russian_word_or_form(candidate) {
        return false;
    }
    let Some((left, right)) = single_substitution_pair(original, candidate) else {
        return false;
    };
    are_ru_keyboard_neighbors(left, right)
}

fn safe_single_substitution_candidate(original: &str, candidate: &str) -> bool {
    (is_known_russian_word_or_form(candidate)
        || crate::nanda_wave::l2::l2_decoder_contains_surface(candidate))
        && single_substitution_pair(original, candidate).is_some()
}

fn single_substitution_pair(original: &str, candidate: &str) -> Option<(char, char)> {
    let original_chars: Vec<char> = original.chars().collect();
    let candidate_chars: Vec<char> = candidate.chars().collect();
    if original_chars.len() != candidate_chars.len() {
        return None;
    }
    let diffs: Vec<(char, char)> = original_chars
        .into_iter()
        .zip(candidate_chars)
        .filter(|(left, right)| left != right)
        .collect();
    (diffs.len() == 1).then(|| diffs[0])
}
