use crate::word_reader::is_cyrillic_word;

use super::guards::read_plain_phrase_pair;

struct BoundaryShiftProposal {
    replacement: String,
    direct_apply_mass: bool,
}

/// Generates a surface-preserving boundary transition for the decision core.
/// It does not grant apply authority.
pub(crate) fn propose_moved_prefix_letter_pair(text: &str) -> Option<String> {
    boundary_shift_proposal(text).map(|proposal| proposal.replacement)
}

/// Compatibility path for callers that do not pass through the decision core.
/// Ambiguous proposals remain visible only to the typed transition route.
pub fn correct_moved_prefix_letter_pair(text: &str) -> Option<String> {
    let proposal = boundary_shift_proposal(text)?;
    proposal.direct_apply_mass.then_some(proposal.replacement)
}

fn boundary_shift_proposal(text: &str) -> Option<BoundaryShiftProposal> {
    let pair = read_plain_phrase_pair(text)?;
    let mut right_chars = pair.right.chars();
    let moved = right_chars.next()?;
    let right_rest = right_chars.collect::<String>();
    if right_rest.is_empty() {
        return None;
    }

    let left_candidate = format!("{}{}", pair.left, moved);
    if !is_cyrillic_word(&left_candidate) || !is_cyrillic_word(&right_rest) {
        return None;
    }

    let original_right = pair.right.to_lowercase();
    let original_left = pair.left.to_lowercase();
    let left_candidate_lower = left_candidate.to_lowercase();
    let right_candidate_lower = right_rest.to_lowercase();
    let field = crate::hot_field::HotFieldSnapshot::current();
    let readout = field.boundary_shift_readout(
        &original_left,
        &original_right,
        &left_candidate_lower,
        &right_candidate_lower,
    );
    if std::env::var_os("LAY_DEBUG_DECISION_CORE").is_some() {
        eprintln!(
            "boundary-shift original={:?} {:?} candidate={:?} {:?} readout={readout:?}",
            original_left, original_right, left_candidate_lower, right_candidate_lower
        );
    }
    if !readout.candidate_settles() {
        return None;
    }
    let candidate = format!("{left_candidate}{}{right_rest}", pair.separator);
    Some(BoundaryShiftProposal {
        replacement: format!("{}{}", candidate, pair.right_trailing),
        direct_apply_mass: readout.has_direct_apply_mass(),
    })
}

#[cfg(test)]
fn has_form_center(word: &str) -> bool {
    crate::hot_field::HotFieldSnapshot::current()
        .form_readout(word)
        .has_structural_center()
}

#[cfg(test)]
fn has_exact_surface_center(word: &str) -> bool {
    crate::hot_field::HotFieldSnapshot::current()
        .surface_phase_readout(word)
        .exact_center
}

#[cfg(test)]
mod tests {
    use super::{
        correct_moved_prefix_letter_pair, has_exact_surface_center, has_form_center,
        propose_moved_prefix_letter_pair,
    };

    #[test]
    fn boundary_shift_lexical_centers_cover_inflected_words_not_corrupt_surfaces() {
        for word in ["на", "допустим", "вот", "ты"] {
            assert!(has_form_center(word), "missing lexical center: {word}");
        }
        for candidate in ["постоянку", "набираю"] {
            assert!(
                has_form_center(candidate),
                "missing form center: {candidate}"
            );
        }
        for word in ["апостоянку", "мнабираю", "тты"] {
            assert!(
                !has_exact_surface_center(word),
                "corrupt surface center is known: {word}"
            );
        }
    }

    #[test]
    fn boundary_shift_uses_hot_field_snapshot_without_cold_dictionary_authority() {
        assert!(!has_exact_surface_center("мнабираю"));
        assert!(has_form_center("допустим"));
        assert!(has_form_center("набираю"));
        assert_eq!(
            correct_moved_prefix_letter_pair("допусти мнабираю"),
            Some("допустим набираю".to_string())
        );
        assert_eq!(
            propose_moved_prefix_letter_pair("сейча сна"),
            Some("сейчас на".to_string())
        );
    }
}
