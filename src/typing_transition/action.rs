use super::verifier;
use crate::correction_core::TypingErrorClass;
use crate::correction_source_contract::CandidateOrigin;
use crate::language_action::{
    operator_for_origin, proof_for_origin, LanguageActionOperator, LanguageActionProof,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CorrectionActionOperatorReport {
    pub(crate) operator: LanguageActionOperator,
    pub(crate) proof: LanguageActionProof,
    pub(crate) edit_operator: verifier::EditTransitionOperator,
    pub(crate) edit_proof: LanguageActionProof,
    pub(crate) verifier_required: bool,
    pub(crate) verifier_passed: bool,
    pub(crate) left_context_changed: bool,
    pub(crate) changed_tokens: usize,
    blocker: Option<&'static str>,
}

impl CorrectionActionOperatorReport {
    pub(crate) const fn apply_blocker(self) -> Option<&'static str> {
        if !self.verifier_required {
            return None;
        }
        self.blocker
    }
}

pub(crate) fn verify_action_operator(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> CorrectionActionOperatorReport {
    let operator = operator_for_origin(error_class, origin);
    let proof = proof_for_origin(error_class, origin);
    let transition = verifier::prove_edit_transition(original, replacement, error_class, origin);

    CorrectionActionOperatorReport {
        operator,
        proof,
        edit_operator: transition.operator,
        edit_proof: transition.language_proof,
        verifier_required: original != replacement,
        verifier_passed: transition.verified,
        left_context_changed: transition.left_context_changed,
        changed_tokens: transition.changed_tokens,
        blocker: transition.reject_apply_reason(),
    }
}

pub(crate) fn classify_token_transition(
    original: &str,
    replacement: &str,
    origin: CandidateOrigin,
    declared: TypingErrorClass,
) -> TypingErrorClass {
    if !matches!(
        origin.source_role(),
        crate::correction_source_contract::CorrectionSourceRole::L2Surface
            | crate::correction_source_contract::CorrectionSourceRole::DeterministicTypo
    ) {
        return declared;
    }
    let Some(original_word) = normalized_last_word(original) else {
        return declared;
    };
    let Some(replacement_word) = normalized_last_word(replacement) else {
        return declared;
    };
    if original_word == replacement_word {
        return declared;
    }
    let original_chars = original_word.chars().collect::<Vec<_>>();
    let replacement_chars = replacement_word.chars().collect::<Vec<_>>();
    if is_adjacent_transposition(&original_chars, &replacement_chars) {
        return TypingErrorClass::AdjacentTransposition;
    }
    if collapses_repeated_runs_to(&original_chars, &replacement_chars) {
        return TypingErrorClass::RepeatedLetter;
    }
    if removes_one_char_to(&original_chars, &replacement_chars) {
        return if removes_repeated_char_to(&original_chars, &replacement_chars) {
            TypingErrorClass::RepeatedLetter
        } else {
            TypingErrorClass::ExtraLetter
        };
    }
    if removes_one_char_to(&replacement_chars, &original_chars) {
        return TypingErrorClass::MissingLetter;
    }
    if original_chars.len() == replacement_chars.len()
        && crate::text_metrics::damerau_levenshtein(&original_word, &replacement_word) == 1
    {
        return TypingErrorClass::LetterSubstitution;
    }
    if matches!(
        declared,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::LetterSubstitution
    ) {
        TypingErrorClass::CompositeTypo
    } else {
        declared
    }
}

fn normalized_last_word(text: &str) -> Option<String> {
    crate::word_reader::last_text_word(text).and_then(|token| {
        let (_, word, _) = crate::word_reader::split_word_punctuation(&token);
        (!word.is_empty()).then(|| word.to_lowercase())
    })
}

fn removes_one_char_to(longer: &[char], shorter: &[char]) -> bool {
    if longer.len() != shorter.len() + 1 {
        return false;
    }
    (0..longer.len()).any(|skip| {
        longer
            .iter()
            .enumerate()
            .filter_map(|(index, ch)| (index != skip).then_some(*ch))
            .eq(shorter.iter().copied())
    })
}

fn removes_repeated_char_to(longer: &[char], shorter: &[char]) -> bool {
    if longer.len() != shorter.len() + 1 {
        return false;
    }
    (0..longer.len()).any(|skip| {
        let repeated = (skip > 0 && longer[skip - 1] == longer[skip])
            || (skip + 1 < longer.len() && longer[skip + 1] == longer[skip]);
        repeated
            && longer
                .iter()
                .enumerate()
                .filter_map(|(index, ch)| (index != skip).then_some(*ch))
                .eq(shorter.iter().copied())
    })
}

fn collapses_repeated_runs_to(longer: &[char], shorter: &[char]) -> bool {
    let longer_runs = run_lengths(longer);
    let shorter_runs = run_lengths(shorter);
    longer_runs.len() == shorter_runs.len()
        && longer_runs.iter().zip(shorter_runs.iter()).all(
            |((left_ch, left_count), (right_ch, right_count))| {
                left_ch == right_ch && right_count <= left_count
            },
        )
        && longer_runs
            .iter()
            .zip(shorter_runs.iter())
            .any(|((_, left_count), (_, right_count))| right_count < left_count)
}

fn run_lengths(chars: &[char]) -> Vec<(char, usize)> {
    let mut runs = Vec::new();
    for ch in chars {
        if let Some((last, count)) = runs.last_mut() {
            if last == ch {
                *count += 1;
                continue;
            }
        }
        runs.push((*ch, 1));
    }
    runs
}

fn is_adjacent_transposition(left: &[char], right: &[char]) -> bool {
    if left.len() != right.len() || left.len() < 2 {
        return false;
    }
    let differences = left
        .iter()
        .zip(right)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    matches!(differences.as_slice(), [first, second]
        if *second == *first + 1
            && left[*first] == right[*second]
            && left[*second] == right[*first])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_verified_layout_projection() {
        let report = verify_action_operator(
            "HF<JNF NTCN CFV ",
            "РАБОТА ТЕСТ САМ ",
            TypingErrorClass::WrongLayout,
            crate::correction_source_contract::CandidateOrigin::Layout,
        );

        assert_eq!(report.operator, LanguageActionOperator::FlipLayout);
        assert_eq!(
            report.edit_operator,
            verifier::EditTransitionOperator::LayoutProjection
        );
        assert!(report.verifier_required);
        assert!(report.verifier_passed);
        assert_eq!(report.apply_blocker(), None);
    }

    #[test]
    fn blocks_unverified_left_context_import() {
        let report = verify_action_operator(
            "содержкой ",
            "что получилось вроде хороший ввод и даже фикс был шикарный но с содержать ",
            TypingErrorClass::CompositeTypo,
            crate::correction_source_contract::CandidateOrigin::L2Surface,
        );

        assert_eq!(report.operator, LanguageActionOperator::FixTypo);
        assert_eq!(
            report.edit_operator,
            verifier::EditTransitionOperator::Unknown
        );
        assert!(report.verifier_required);
        assert!(!report.verifier_passed);
        assert_eq!(report.apply_blocker(), Some("edit_transition_not_verified"));
    }

    #[test]
    fn typo_operator_is_inferred_from_the_observed_transition() {
        for (original, replacement, expected) in [
            ("dowenload ", "download ", TypingErrorClass::ExtraLetter),
            ("руских ", "русских ", TypingErrorClass::MissingLetter),
            ("длеай ", "делай ", TypingErrorClass::AdjacentTransposition),
        ] {
            assert_eq!(
                classify_token_transition(
                    original,
                    replacement,
                    CandidateOrigin::L2Surface,
                    TypingErrorClass::CompositeTypo,
                ),
                expected
            );
        }
        assert_eq!(
            classify_token_transition(
                "ТРУССС ",
                "ТРУС ",
                CandidateOrigin::DeterministicTypo,
                TypingErrorClass::CompositeTypo,
            ),
            TypingErrorClass::RepeatedLetter
        );
    }
}
