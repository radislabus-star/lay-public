//! Language-level action operators.
//!
//! This layer names what the correction pipeline wants to do. Output backends
//! still own how text is deleted/inserted.

use crate::correction_core::TypingErrorClass;

pub const LANGUAGE_ACTION_OPERATOR_COUNT: usize = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageActionOperator {
    KeepOriginal,
    SuggestOnly,
    FixTypo,
    FixTransposition,
    ReplaceLetter,
    RemoveExtraLetter,
    RestoreMissingLetter,
    NormalizeCase,
    FixGrammarForm,
    FlipLayout,
    FixMixedLayout,
    CompleteWord,
    SplitGluedWords,
    JoinBrokenWord,
    ApplyContextChoice,
    SyncLayoutState,
    Veto,
}

impl LanguageActionOperator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::KeepOriginal => "keep_original",
            Self::SuggestOnly => "suggest_only",
            Self::FixTypo => "fix_typo",
            Self::FixTransposition => "fix_transposition",
            Self::ReplaceLetter => "replace_letter",
            Self::RemoveExtraLetter => "remove_extra_letter",
            Self::RestoreMissingLetter => "restore_missing_letter",
            Self::NormalizeCase => "normalize_case",
            Self::FixGrammarForm => "fix_grammar_form",
            Self::FlipLayout => "flip_layout",
            Self::FixMixedLayout => "fix_mixed_layout",
            Self::CompleteWord => "complete_word",
            Self::SplitGluedWords => "split_glued_words",
            Self::JoinBrokenWord => "join_broken_word",
            Self::ApplyContextChoice => "apply_context_choice",
            Self::SyncLayoutState => "sync_layout_state",
            Self::Veto => "veto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageActionProof {
    None,
    Layout,
    Typo,
    Boundary,
    Completion,
    Context,
    Grammar,
    SafetyVeto,
}

impl LanguageActionProof {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Layout => "layout",
            Self::Typo => "typo",
            Self::Boundary => "boundary",
            Self::Completion => "completion",
            Self::Context => "context",
            Self::Grammar => "grammar",
            Self::SafetyVeto => "safety_veto",
        }
    }
}

pub fn operator_for_candidate(
    error_class: TypingErrorClass,
    source_id: &str,
) -> LanguageActionOperator {
    if source_id.starts_with("layout_then_") {
        return LanguageActionOperator::FixMixedLayout;
    }
    match error_class {
        TypingErrorClass::WrongLayout => LanguageActionOperator::FlipLayout,
        TypingErrorClass::PartialLayout | TypingErrorClass::MixedScript => {
            LanguageActionOperator::FixMixedLayout
        }
        TypingErrorClass::MissingLetter => LanguageActionOperator::RestoreMissingLetter,
        TypingErrorClass::ExtraLetter | TypingErrorClass::RepeatedLetter => {
            LanguageActionOperator::RemoveExtraLetter
        }
        TypingErrorClass::AdjacentTransposition => LanguageActionOperator::FixTransposition,
        TypingErrorClass::LetterSubstitution => LanguageActionOperator::ReplaceLetter,
        TypingErrorClass::CaseNoise => LanguageActionOperator::NormalizeCase,
        TypingErrorClass::CompositeTypo => context_or_typo_operator(source_id),
        TypingErrorClass::Unknown => LanguageActionOperator::SuggestOnly,
        TypingErrorClass::SplitWord => LanguageActionOperator::JoinBrokenWord,
        TypingErrorClass::GluedWords => LanguageActionOperator::SplitGluedWords,
        TypingErrorClass::GrammarAgreement => LanguageActionOperator::FixGrammarForm,
        TypingErrorClass::CompletionOnly => LanguageActionOperator::CompleteWord,
        TypingErrorClass::TechnicalToken | TypingErrorClass::ProtectedToken => {
            LanguageActionOperator::KeepOriginal
        }
    }
}

pub fn proof_for_candidate(error_class: TypingErrorClass, source_id: &str) -> LanguageActionProof {
    if source_id.starts_with("layout_then_") {
        return LanguageActionProof::Layout;
    }
    match error_class {
        TypingErrorClass::WrongLayout | TypingErrorClass::PartialLayout => {
            LanguageActionProof::Layout
        }
        TypingErrorClass::MixedScript => LanguageActionProof::Layout,
        TypingErrorClass::SplitWord | TypingErrorClass::GluedWords => LanguageActionProof::Boundary,
        TypingErrorClass::GrammarAgreement => LanguageActionProof::Grammar,
        TypingErrorClass::CompletionOnly => LanguageActionProof::Completion,
        TypingErrorClass::TechnicalToken | TypingErrorClass::ProtectedToken => {
            LanguageActionProof::SafetyVeto
        }
        TypingErrorClass::CompositeTypo if is_context_source(source_id) => {
            LanguageActionProof::Context
        }
        TypingErrorClass::LetterSubstitution
        | TypingErrorClass::CompositeTypo
        | TypingErrorClass::MissingLetter
        | TypingErrorClass::ExtraLetter
        | TypingErrorClass::RepeatedLetter
        | TypingErrorClass::AdjacentTransposition
        | TypingErrorClass::CaseNoise => LanguageActionProof::Typo,
        TypingErrorClass::Unknown => LanguageActionProof::None,
    }
}

fn context_or_typo_operator(source_id: &str) -> LanguageActionOperator {
    if is_context_source(source_id) {
        LanguageActionOperator::ApplyContextChoice
    } else {
        LanguageActionOperator::FixTypo
    }
}

fn is_context_source(source_id: &str) -> bool {
    matches!(
        source_id,
        "PhraseForecastCell32" | "PhraseMemoryCell32" | "PhraseCell32" | "SemanticWordCell32"
    )
}

#[cfg(test)]
mod tests {
    use super::{LanguageActionOperator, LANGUAGE_ACTION_OPERATOR_COUNT};

    #[test]
    fn action_operator_contract_has_seventeen_public_actions() {
        let operators = [
            LanguageActionOperator::KeepOriginal,
            LanguageActionOperator::SuggestOnly,
            LanguageActionOperator::FixTypo,
            LanguageActionOperator::FixTransposition,
            LanguageActionOperator::ReplaceLetter,
            LanguageActionOperator::RemoveExtraLetter,
            LanguageActionOperator::RestoreMissingLetter,
            LanguageActionOperator::NormalizeCase,
            LanguageActionOperator::FixGrammarForm,
            LanguageActionOperator::FlipLayout,
            LanguageActionOperator::FixMixedLayout,
            LanguageActionOperator::CompleteWord,
            LanguageActionOperator::SplitGluedWords,
            LanguageActionOperator::JoinBrokenWord,
            LanguageActionOperator::ApplyContextChoice,
            LanguageActionOperator::SyncLayoutState,
            LanguageActionOperator::Veto,
        ];

        assert_eq!(operators.len(), LANGUAGE_ACTION_OPERATOR_COUNT);
    }
}
