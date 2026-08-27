//! Candidate proposal admission owned by the Typing Transition CPU.
//!
//! Producers supply a typed origin and a proposed surface transition. This
//! module classifies its maximum authority before the learned decision field
//! performs final selection.

use super::{action as action_operator, verifier as edit_transition};
use crate::candidate_contract::{CandidateOrigin, CorrectionSourceRole};
use crate::candidate_explanation::explain_candidate;
use crate::correction_core::TypingErrorClass;
use crate::russian_typo_candidates::{
    inserted_char_position_for_missing_letter, repeated_run_deletion_candidates,
};
use crate::text_metrics::{damerau_levenshtein, has_cyrillic};
use crate::word_reader::{
    is_cyrillic_letters_only, last_text_word, split_edge_whitespace, split_word_punctuation,
};
const REPEATED_DELETE_SURFACE_MARGIN: f64 = 0.25;

include!("proposal_admission/trace.rs");

#[derive(Debug)]
struct AdmissionWordFacts {
    word: String,
    lower: std::cell::OnceCell<String>,
    cyrillic_letters_only: std::cell::OnceCell<bool>,
    char_len: std::cell::OnceCell<usize>,
    known_russian: std::cell::OnceCell<bool>,
    protected_current: std::cell::OnceCell<bool>,
}

impl AdmissionWordFacts {
    fn new(word: String) -> Self {
        Self {
            word,
            lower: std::cell::OnceCell::new(),
            cyrillic_letters_only: std::cell::OnceCell::new(),
            char_len: std::cell::OnceCell::new(),
            known_russian: std::cell::OnceCell::new(),
            protected_current: std::cell::OnceCell::new(),
        }
    }

    fn word(&self) -> &str {
        &self.word
    }

    fn lower(&self) -> &str {
        self.lower.get_or_init(|| self.word.to_lowercase())
    }

    fn is_cyrillic_letters_only(&self) -> bool {
        *self
            .cyrillic_letters_only
            .get_or_init(|| is_cyrillic_letters_only(&self.word))
    }

    fn char_len(&self) -> usize {
        *self.char_len.get_or_init(|| self.lower().chars().count())
    }

    fn is_known_russian(&self) -> bool {
        *self
            .known_russian
            .get_or_init(|| known_russian_autocorrect_token(self.lower()))
    }

    fn is_protected_current(&self) -> bool {
        *self.protected_current.get_or_init(|| {
            let field = crate::hot_field::HotFieldSnapshot::current();
            self.is_known_russian()
                || crate::phrase_lexicon::is_known_russian_phrase_part(self.lower())
                || field
                    .input_surface_readout(self.lower())
                    .has_phase_authority()
                || field.word_readout(self.lower()).is_known()
                || crate::nanda_wave::l2::l2_surface_foundation_contains(self.lower())
                || crate::nanda_wave::l2::l2_surface_foundation_has_authority(self.lower())
                || crate::russian_lexicon::is_reference_backed_russian_form(self.lower())
                || crate::russian_lexicon::is_center_backed_russian_form(self.lower())
                || crate::russian_lexicon::is_reference_known_russian_word_or_form(self.lower())
                || crate::russian_lexicon::russian_dictionary().contains(self.lower())
                || crate::russian_lexicon::russian_short_dictionary().contains(self.lower())
        })
    }
}

#[cfg(test)]
#[derive(Debug, Default, PartialEq, Eq)]
struct AdmissionLexicalFactSnapshot {
    original_word: bool,
    replacement_word: bool,
    original_lower: bool,
    replacement_lower: bool,
    original_known: bool,
    replacement_known: bool,
    original_protected: bool,
    replacement_protected: bool,
}

struct AdmissionLexicalFacts<'a> {
    original: &'a str,
    replacement: &'a str,
    reuse: bool,
    original_word: std::cell::OnceCell<Option<AdmissionWordFacts>>,
    replacement_word: std::cell::OnceCell<Option<AdmissionWordFacts>>,
}

impl<'a> AdmissionLexicalFacts<'a> {
    fn new(original: &'a str, replacement: &'a str) -> Self {
        #[cfg(test)]
        let reuse = configured_admission_fact_reuse();
        #[cfg(not(test))]
        let reuse = true;

        Self::with_mode(original, replacement, reuse)
    }

    fn with_mode(original: &'a str, replacement: &'a str, reuse: bool) -> Self {
        Self {
            original,
            replacement,
            reuse,
            original_word: std::cell::OnceCell::new(),
            replacement_word: std::cell::OnceCell::new(),
        }
    }

    fn reuses_facts(&self) -> bool {
        self.reuse
    }

    fn assert_pair(&self, original: &str, replacement: &str) {
        debug_assert_eq!(
            self.original, original,
            "lexical fact original identity drift"
        );
        debug_assert_eq!(
            self.replacement, replacement,
            "lexical fact replacement identity drift"
        );
    }

    fn original_word(&self) -> Option<&AdmissionWordFacts> {
        self.original_word
            .get_or_init(|| last_text_word(self.original).map(AdmissionWordFacts::new))
            .as_ref()
    }

    fn replacement_word(&self) -> Option<&AdmissionWordFacts> {
        self.replacement_word
            .get_or_init(|| last_text_word(self.replacement).map(AdmissionWordFacts::new))
            .as_ref()
    }

    #[cfg(test)]
    fn snapshot(&self) -> AdmissionLexicalFactSnapshot {
        let original = self.original_word.get().and_then(Option::as_ref);
        let replacement = self.replacement_word.get().and_then(Option::as_ref);
        AdmissionLexicalFactSnapshot {
            original_word: self.original_word.get().is_some(),
            replacement_word: self.replacement_word.get().is_some(),
            original_lower: original.is_some_and(|word| word.lower.get().is_some()),
            replacement_lower: replacement.is_some_and(|word| word.lower.get().is_some()),
            original_known: original.is_some_and(|word| word.known_russian.get().is_some()),
            replacement_known: replacement.is_some_and(|word| word.known_russian.get().is_some()),
            original_protected: original.is_some_and(|word| word.protected_current.get().is_some()),
            replacement_protected: replacement
                .is_some_and(|word| word.protected_current.get().is_some()),
        }
    }
}

macro_rules! admission_fact_call {
    ($facts:expr, $cached:ident, $uncached:ident $(, $arg:expr)* $(,)?) => {
        $cached($($arg,)* $facts)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateGateAction {
    /// Producer supplied evidence and no structural constraint blocked it. Only
    /// TransitionDecisionCore may turn this into a physical Apply.
    Eligible,
    SuggestOnly,
    KeepOriginal,
    Veto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGateDecision {
    pub action: CandidateGateAction,
    pub reason: &'static str,
}

#[cfg(test)]
pub(crate) fn gate_candidate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
) -> CandidateGateDecision {
    gate_candidate_with_origin(
        original,
        replacement,
        error_class,
        CandidateOrigin::DeterministicTypo,
    )
}

/// Compatibility adapter for legacy fixtures. Runtime producers must supply a
/// typed origin and cannot route authority through a diagnostic source name.
#[cfg(test)]
pub(crate) fn gate_candidate_with_source(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> CandidateGateDecision {
    gate_candidate_with_origin(
        original,
        replacement,
        error_class,
        fixture_origin(source_id),
    )
}

#[cfg(test)]
fn fixture_origin(source_id: &str) -> CandidateOrigin {
    if source_id.starts_with("layout_then_") {
        CandidateOrigin::LayoutThenTypo
    } else if source_id.contains("layout")
        || matches!(source_id, "LayoutWordCell32" | "ShortTokenCell32")
    {
        CandidateOrigin::Layout
    } else if matches!(
        source_id,
        "BoundaryCell32" | "BoundaryShiftCell32" | "layout_phrase"
    ) {
        CandidateOrigin::Boundary
    } else if matches!(
        source_id,
        "PhraseForecastCell32" | "L2SurfaceCompletionCell32"
    ) {
        CandidateOrigin::Completion
    } else if matches!(source_id, "L2SurfaceMotifCell32" | "L2WordAttractorCell32") {
        CandidateOrigin::L2Surface
    } else if matches!(
        source_id,
        "PhraseCell32" | "PhraseMemoryCell32" | "SemanticWordCell32"
    ) {
        CandidateOrigin::L3Context
    } else if source_id == "TechTokenCell32" {
        CandidateOrigin::Technical
    } else {
        CandidateOrigin::DeterministicTypo
    }
}

pub(crate) fn gate_candidate_with_origin(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> CandidateGateDecision {
    #[cfg(test)]
    if admission_trace_active() {
        let started = std::time::Instant::now();
        let decision = candidate_admission(original, replacement, error_class, origin);
        record_admission(started.elapsed());
        return decision;
    }
    candidate_admission(original, replacement, error_class, origin)
}

fn candidate_admission(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> CandidateGateDecision {
    let lexical_facts = AdmissionLexicalFacts::new(original, replacement);
    candidate_admission_with_facts(original, replacement, error_class, origin, &lexical_facts)
}

fn candidate_admission_with_facts(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    lexical_facts: &AdmissionLexicalFacts<'_>,
) -> CandidateGateDecision {
    if admission_trace_bool!(Unchanged, original == replacement) {
        return CandidateGateDecision {
            action: CandidateGateAction::KeepOriginal,
            reason: "unchanged",
        };
    }
    let (explanation, explanation_blocks_apply) = admission_trace_value!(
        ExplainCandidate,
        {
            let explanation = explain_candidate(original, replacement, error_class, origin);
            let blocks_apply = explanation.blocks_apply();
            (explanation, blocks_apply)
        },
        |value: &(crate::candidate_explanation::CandidateExplanation, bool)| value.1
    );
    if explanation_blocks_apply {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unexplained_signal_loss",
        };
    }
    drop(explanation);
    if admission_trace_bool!(
        ReplacementGluesSeparateWords,
        replacement_glues_separate_words_without_boundary_class(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "word_count_shrink_requires_boundary_class",
        };
    }
    if admission_trace_bool!(
        BoundaryGluesShortFunctionTail,
        boundary_candidate_glues_short_function_tail(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unsafe_boundary_glue_short_function_tail",
        };
    }
    if admission_trace_bool!(
        BoundaryEatsKnownCurrentWord,
        boundary_candidate_eats_known_current_word(original, replacement, error_class, origin)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "moved_prefix_eats_known_current_word",
        };
    }
    if admission_trace_bool!(
        BoundaryChangesNonWhitespaceSurface,
        boundary_operator_changes_non_whitespace_surface(
            original,
            replacement,
            error_class,
            origin
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "boundary_operator_changes_surface",
        };
    }
    if admission_trace_bool!(
        MultiwordLastVowelCompletion,
        multi_word_candidate_only_completes_last_vowel(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unsafe_multi_word_vowel_completion",
        };
    }
    if admission_trace_bool!(
        AdjacentTranspositionBoundaryCompetition,
        adjacent_transposition_competes_with_single_letter_boundary(
            original,
            replacement,
            error_class,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "single_letter_boundary_beats_transposition",
        };
    }
    if admission_trace_bool!(
        BoundarySplitsKnownWord,
        boundary_candidate_splits_known_russian_word(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_single_word_boundary_split",
        };
    }
    if admission_trace_bool!(
        BoundarySplitsWeakTail,
        boundary_candidate_splits_to_short_function_and_weak_tail(
            original,
            replacement,
            error_class,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "weak_boundary_split_tail",
        };
    }
    if admission_trace_bool!(
        ReflexiveSuffixRequiresGrammar,
        reflexive_suffix_candidate_requires_grammar_proof(original, replacement, error_class)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "reflexive_suffix_requires_grammar_proof",
        };
    }
    if admission_trace_bool!(
        KnownCurrentSurfaceDrift,
        admission_fact_call!(
            lexical_facts,
            known_current_word_gets_unproven_surface_drift_with_facts,
            known_current_word_gets_unproven_surface_drift,
            original,
            replacement,
            error_class,
            origin,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "known_current_word_surface_drift",
        };
    }
    let action_blocker = admission_trace_value!(
        VerifyActionOperator,
        action_operator::verify_action_operator(original, replacement, error_class, origin)
            .apply_blocker(),
        |value: &Option<&'static str>| value.is_some()
    );
    if let Some(reason) = action_blocker {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason,
        };
    }
    if admission_trace_bool!(
        SurfaceChangesLeftContext,
        surface_candidate_changes_left_context(original, replacement, origin)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "surface_left_context_apply_blocked",
        };
    }
    if admission_trace_bool!(
        L2SurfaceStemTruncation,
        l2_surface_candidate_truncates_to_stem_without_deletion_proof(
            original,
            replacement,
            origin,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "l2_surface_stem_truncation_low",
        };
    }
    if let Some(decision) =
        structural_context_gate(original, replacement, error_class, origin, lexical_facts)
    {
        return decision;
    }
    if admission_trace_bool!(
        UnprovenStableSurfaceShape,
        admission_fact_call!(
            lexical_facts,
            unproven_stable_surface_shape_drift_with_facts,
            unproven_stable_surface_shape_drift,
            original,
            replacement,
            error_class,
            origin,
        )
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "unproven_stable_surface_shape_drift",
        };
    }
    if admission_trace_bool!(
        SemanticSurfaceAuthority,
        semantic_candidate_lacks_surface_authority(original, replacement, origin)
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "semantic_wave_surface_authority_low",
        };
    }
    if admission_trace_bool!(
        CompletionOnly,
        error_class == TypingErrorClass::CompletionOnly
    ) {
        return CandidateGateDecision {
            action: CandidateGateAction::SuggestOnly,
            reason: "completion_is_not_autocorrect",
        };
    }
    admission_trace_value!(
        FinalClassDispatch,
        match error_class {
            TypingErrorClass::TechnicalToken | TypingErrorClass::ProtectedToken => {
                CandidateGateDecision {
                    action: CandidateGateAction::Veto,
                    reason: "protected_or_technical",
                }
            }
            TypingErrorClass::RepeatedLetter | TypingErrorClass::ExtraLetter
                if replacement_last_word_is_unknown_cyrillic(original, replacement) =>
            {
                CandidateGateDecision {
                    action: CandidateGateAction::SuggestOnly,
                    reason: "single_step_typo_still_unknown",
                }
            }
            TypingErrorClass::Unknown => CandidateGateDecision {
                action: CandidateGateAction::SuggestOnly,
                reason: "unknown_error_class",
            },
            _ => CandidateGateDecision {
                action: CandidateGateAction::Eligible,
                reason: "class_allows_apply",
            },
        },
        |_value: &CandidateGateDecision| true
    )
}

// Keep predicates in this module to preserve the established API surface.
include!("proposal_admission/structural_guards.rs");
include!("proposal_admission/surface_support.rs");

#[cfg(test)]
#[path = "proposal_admission/tests.rs"]
mod tests;
