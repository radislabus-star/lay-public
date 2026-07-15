use crate::candidate_contract::CandidateOrigin;
use crate::correction_core::TypingErrorClass;
use crate::language_action::{proof_for_origin, LanguageActionProof};
pub(crate) use crate::text_edit::TransitionOperator as EditTransitionOperator;
use crate::text_edit::TransitionOperator;
use crate::word_reader::{
    is_cyrillic_letters_only, split_edge_whitespace, split_last_trimmed_ws_token,
    split_word_punctuation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EditTransitionProof {
    pub(crate) operator: TransitionOperator,
    pub(crate) language_proof: LanguageActionProof,
    pub(crate) left_context_changed: bool,
    pub(crate) original_words: usize,
    pub(crate) replacement_words: usize,
    pub(crate) changed_tokens: usize,
    pub(crate) verified: bool,
}

impl EditTransitionProof {
    pub(crate) const fn reject_apply_reason(self) -> Option<&'static str> {
        if self.left_context_changed && !self.verified {
            Some("edit_transition_not_verified")
        } else {
            None
        }
    }
}

pub(crate) fn prove_edit_transition(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> EditTransitionProof {
    let language_proof = proof_for_origin(error_class, origin);
    let original_words = core_words(original);
    let replacement_words = core_words(replacement);
    let left_context_changed = left_context_changed(original, replacement);
    let changed_tokens = changed_token_count(&original_words, &replacement_words);

    let input = EditTransitionInput {
        original,
        replacement,
        error_class,
        proof: language_proof,
        left_context_changed,
        original_words: &original_words,
        replacement_words: &replacement_words,
        changed_tokens,
    };
    let operator = infer_operator(&input);

    EditTransitionProof {
        operator,
        language_proof,
        left_context_changed,
        original_words: original_words.len(),
        replacement_words: replacement_words.len(),
        changed_tokens,
        verified: operator_is_verified(operator),
    }
}

struct EditTransitionInput<'a> {
    original: &'a str,
    replacement: &'a str,
    error_class: TypingErrorClass,
    proof: LanguageActionProof,
    left_context_changed: bool,
    original_words: &'a [String],
    replacement_words: &'a [String],
    changed_tokens: usize,
}

fn infer_operator(input: &EditTransitionInput<'_>) -> TransitionOperator {
    if matches!(input.proof, LanguageActionProof::SafetyVeto) {
        return TransitionOperator::Protected;
    }
    if matches!(input.proof, LanguageActionProof::Completion) {
        return TransitionOperator::Completion;
    }
    if !input.left_context_changed {
        return TransitionOperator::ReplaceCurrentWord;
    }
    if contextual_layout_repair_is_verified(
        input.proof,
        input.error_class,
        input.original_words,
        input.replacement_words,
        input.changed_tokens,
    ) {
        return TransitionOperator::PhraseTokenRepair;
    }
    if layout_projection_is_verified(
        input.proof,
        input.error_class,
        input.original_words,
        input.replacement_words,
        input.changed_tokens,
    ) {
        return TransitionOperator::LayoutProjection;
    }
    if boundary_shift_is_verified(
        input.original,
        input.replacement,
        input.proof,
        input.error_class,
        input.original_words,
        input.replacement_words,
        input.changed_tokens,
    ) {
        return TransitionOperator::BoundaryShift;
    }
    if boundary_merge_split_is_verified(
        input.proof,
        input.error_class,
        input.original_words,
        input.replacement_words,
    ) {
        return TransitionOperator::BoundaryMergeSplit;
    }
    if phrase_token_repair_is_verified(
        input.proof,
        input.original_words,
        input.replacement_words,
        input.changed_tokens,
    ) {
        return TransitionOperator::PhraseTokenRepair;
    }
    if split_previous_glued_and_repair_tail_is_verified(input.original, input.replacement) {
        return TransitionOperator::SplitPreviousGluedAndRepairTail;
    }
    TransitionOperator::Unknown
}

fn contextual_layout_repair_is_verified(
    proof: LanguageActionProof,
    error_class: TypingErrorClass,
    original_words: &[String],
    replacement_words: &[String],
    changed_tokens: usize,
) -> bool {
    if !matches!(proof, LanguageActionProof::Layout)
        || !matches!(
            error_class,
            TypingErrorClass::WrongLayout
                | TypingErrorClass::PartialLayout
                | TypingErrorClass::MixedScript
        )
        || original_words.len() < 2
        || original_words.len() != replacement_words.len()
        || changed_tokens != 1
    {
        return false;
    }

    original_words
        .iter()
        .zip(replacement_words)
        .enumerate()
        .find(|(_, (original, replacement))| original != replacement)
        .is_some_and(|(index, (original, replacement))| {
            index + 1 < original_words.len() && exact_layout_projection(original, replacement)
        })
}

fn exact_layout_projection(original: &str, replacement: &str) -> bool {
    let direction = if original.chars().any(|ch| ch.is_ascii_alphabetic()) {
        crate::dict::Direction::Us2Ru
    } else {
        crate::dict::Direction::Ru2Us
    };
    crate::dict::convert(original, direction) == replacement
}

fn operator_is_verified(operator: TransitionOperator) -> bool {
    !matches!(
        operator,
        TransitionOperator::Unknown | TransitionOperator::Protected
    )
}

fn left_context_changed(original: &str, replacement: &str) -> bool {
    let Some((original_prefix, _)) = split_last_trimmed_ws_token(original) else {
        return false;
    };
    let Some((replacement_prefix, _)) = split_last_trimmed_ws_token(replacement) else {
        return false;
    };
    original_prefix != replacement_prefix
}

fn layout_projection_is_verified(
    proof: LanguageActionProof,
    error_class: TypingErrorClass,
    original_words: &[String],
    replacement_words: &[String],
    changed_tokens: usize,
) -> bool {
    matches!(proof, LanguageActionProof::Layout)
        && matches!(
            error_class,
            TypingErrorClass::WrongLayout
                | TypingErrorClass::PartialLayout
                | TypingErrorClass::MixedScript
        )
        && original_words.len() >= 2
        && original_words.len() == replacement_words.len()
        && changed_tokens == original_words.len()
}

fn boundary_shift_is_verified(
    original: &str,
    replacement: &str,
    proof: LanguageActionProof,
    error_class: TypingErrorClass,
    original_words: &[String],
    replacement_words: &[String],
    changed_tokens: usize,
) -> bool {
    matches!(proof, LanguageActionProof::Boundary)
        && error_class == TypingErrorClass::BoundaryShift
        && original_words.len() >= 2
        && original_words.len() == replacement_words.len()
        && changed_tokens == 2
        && crate::text_edit::safety::surface_preserving_right_to_left_boundary_shift(
            original,
            replacement,
        )
        && changed_replacement_tokens_have_lexical_mass(original_words, replacement_words)
}

fn changed_replacement_tokens_have_lexical_mass(
    original_words: &[String],
    replacement_words: &[String],
) -> bool {
    original_words
        .iter()
        .zip(replacement_words)
        .filter(|(original, replacement)| original != replacement)
        .all(|(_, replacement)| {
            let (_, word, _) = split_word_punctuation(replacement);
            !word.is_empty()
                && (!is_cyrillic_letters_only(word)
                    || crate::phrase_lexicon::is_known_russian_phrase_part(&word.to_lowercase()))
        })
}

fn boundary_merge_split_is_verified(
    proof: LanguageActionProof,
    error_class: TypingErrorClass,
    original_words: &[String],
    replacement_words: &[String],
) -> bool {
    matches!(proof, LanguageActionProof::Boundary)
        && matches!(
            error_class,
            TypingErrorClass::SplitWord | TypingErrorClass::GluedWords
        )
        && original_words.len().abs_diff(replacement_words.len()) == 1
        && has_one_merge_or_split(original_words, replacement_words)
}

fn phrase_token_repair_is_verified(
    proof: LanguageActionProof,
    original_words: &[String],
    replacement_words: &[String],
    changed_tokens: usize,
) -> bool {
    matches!(proof, LanguageActionProof::Context)
        && original_words.len() >= 2
        && original_words.len() == replacement_words.len()
        && changed_tokens == 1
}

fn split_previous_glued_and_repair_tail_is_verified(original: &str, replacement: &str) -> bool {
    let original_words = core_words(original);
    let replacement_words = core_words(replacement);
    if original_words.is_empty() || replacement_words.len() != original_words.len() + 1 {
        return false;
    }

    for split_idx in 0..original_words.len() {
        let Some(left) = replacement_words.get(split_idx) else {
            continue;
        };
        let Some(right) = replacement_words.get(split_idx + 1) else {
            continue;
        };
        let merged = format!("{left}{right}");
        if !same_cyrillic_token(&original_words[split_idx], &merged) {
            continue;
        }
        if original_words[..split_idx] != replacement_words[..split_idx] {
            continue;
        }
        let original_after = &original_words[split_idx + 1..];
        let replacement_after = &replacement_words[split_idx + 2..];
        if original_after.len() != replacement_after.len() {
            continue;
        }
        let changed_after = original_after
            .iter()
            .zip(replacement_after.iter())
            .enumerate()
            .filter(|(_, (left, right))| left != right)
            .map(|(idx, _)| idx)
            .collect::<Vec<_>>();
        if changed_after.is_empty()
            || changed_after.as_slice() == [original_after.len().saturating_sub(1)]
        {
            return true;
        }
    }
    false
}

fn has_one_merge_or_split(original_words: &[String], replacement_words: &[String]) -> bool {
    if original_words.len() + 1 == replacement_words.len() {
        return has_one_split(original_words, replacement_words);
    }
    if replacement_words.len() + 1 == original_words.len() {
        return has_one_split(replacement_words, original_words);
    }
    false
}

fn has_one_split(shorter: &[String], longer: &[String]) -> bool {
    for split_idx in 0..shorter.len() {
        let Some(left) = longer.get(split_idx) else {
            continue;
        };
        let Some(right) = longer.get(split_idx + 1) else {
            continue;
        };
        let merged = format!("{left}{right}");
        if !same_cyrillic_token(&shorter[split_idx], &merged) {
            continue;
        }
        if shorter[..split_idx] == longer[..split_idx]
            && shorter[split_idx + 1..] == longer[split_idx + 2..]
        {
            return true;
        }
    }
    false
}

fn changed_token_count(original_words: &[String], replacement_words: &[String]) -> usize {
    if original_words.len() != replacement_words.len() {
        return original_words.len().max(replacement_words.len());
    }
    original_words
        .iter()
        .zip(replacement_words.iter())
        .filter(|(left, right)| left != right)
        .count()
}

fn core_words(text: &str) -> Vec<String> {
    let (_, core, _) = split_edge_whitespace(text);
    core.split_whitespace().map(str::to_string).collect()
}

pub(crate) fn same_cyrillic_token(original: &str, candidate: &str) -> bool {
    let (_, original_word, _) = split_word_punctuation(original);
    let (_, candidate_word, _) = split_word_punctuation(candidate);
    !original_word.is_empty()
        && !candidate_word.is_empty()
        && is_cyrillic_letters_only(original_word)
        && is_cyrillic_letters_only(candidate_word)
        && original_word.to_lowercase() == candidate_word.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proof(
        original: &str,
        replacement: &str,
        error_class: TypingErrorClass,
        origin: CandidateOrigin,
    ) -> EditTransitionProof {
        prove_edit_transition(original, replacement, error_class, origin)
    }

    #[test]
    fn proves_current_word_replacement() {
        let proof = proof(
            "что получилось содержкой ",
            "что получилось содержать ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::L2Surface,
        );

        assert_eq!(proof.operator, TransitionOperator::ReplaceCurrentWord);
        assert!(proof.verified);
        assert_eq!(proof.reject_apply_reason(), None);
    }

    #[test]
    fn rejects_surface_extra_context_as_unverified_future() {
        let proof = proof(
            "содержкой ",
            "что получилось вроде хороший ввод и даже фикс был шикарный но с содержать ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::L2Surface,
        );

        assert_eq!(proof.operator, TransitionOperator::Unknown);
        assert_eq!(
            proof.reject_apply_reason(),
            Some("edit_transition_not_verified")
        );
    }

    #[test]
    fn proves_multiword_layout_projection() {
        let proof = proof(
            "HF<JNF NTCN CFV ",
            "РАБОТА ТЕСТ САМ ",
            TypingErrorClass::WrongLayout,
            CandidateOrigin::Layout,
        );

        assert_eq!(proof.operator, TransitionOperator::LayoutProjection);
        assert!(proof.verified);
    }

    #[test]
    fn single_word_layout_cannot_import_context() {
        let proof = proof(
            "uрафике ",
            "на графике ",
            TypingErrorClass::WrongLayout,
            CandidateOrigin::Layout,
        );

        assert_eq!(proof.operator, TransitionOperator::Unknown);
        assert_eq!(
            proof.reject_apply_reason(),
            Some("edit_transition_not_verified")
        );
    }

    #[test]
    fn proves_phrase_token_repair() {
        let proof = proof(
            "Поставщик говорит что цена до склада нашего покупателя но таможен мы! ",
            "Поставщик говорит что цена до склада нашего покупателя но таможим мы! ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::L3Context,
        );

        assert_eq!(proof.operator, TransitionOperator::PhraseTokenRepair);
        assert!(proof.verified);
    }

    #[test]
    fn proves_surface_preserving_boundary_shift() {
        for (original, replacement) in [
            ("допусти мнабираю ", "допустим набираю "),
            ("во тты смотри ", "вот ты смотри "),
        ] {
            let proof = proof(
                original,
                replacement,
                TypingErrorClass::BoundaryShift,
                CandidateOrigin::Boundary,
            );

            assert_eq!(proof.operator, TransitionOperator::BoundaryShift);
            assert!(proof.verified);
            assert!(proof.left_context_changed);
            assert_eq!(proof.changed_tokens, 2);
        }
    }

    #[test]
    fn boundary_shift_cannot_change_letters_or_import_context() {
        for replacement in [
            "допустим выбираю ",
            "мы допустим набираю ",
            "допустим, набираю ",
            "Допустим набираю ",
            "допустим  набираю ",
        ] {
            let proof = proof(
                "допусти мнабираю ",
                replacement,
                TypingErrorClass::BoundaryShift,
                CandidateOrigin::Boundary,
            );
            assert_ne!(
                proof.operator,
                TransitionOperator::BoundaryShift,
                "replacement={replacement:?} proof={proof:?}"
            );
            assert!(
                !proof.verified,
                "replacement={replacement:?} proof={proof:?}"
            );
        }
    }

    #[test]
    fn proves_split_previous_glued_plus_current_tail_repair() {
        let proof = proof(
            "ее простозальет свтеом ",
            "ее просто зальет светом ",
            TypingErrorClass::AdjacentTransposition,
            CandidateOrigin::DeterministicTypo,
        );

        assert_eq!(
            proof.operator,
            TransitionOperator::SplitPreviousGluedAndRepairTail
        );
        assert!(proof.verified);
    }

    #[test]
    fn safety_veto_is_not_an_apply_transition_proof() {
        let proof = proof(
            "curl file ",
            "curl файл ",
            TypingErrorClass::ProtectedToken,
            CandidateOrigin::Technical,
        );

        assert_eq!(proof.operator, TransitionOperator::Protected);
        assert!(!proof.verified);
    }
}
