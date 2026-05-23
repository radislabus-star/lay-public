//! Stable shared core API for desktop frontends.
//!
//! Frontends such as GNOME Shell or a future KDE/Plasma integration should use
//! this facade for layout conversion, candidate scoring and backend detection
//! instead of depending on daemon-private implementation details.

pub use crate::config::{
    default_typing_assist_pipeline, normalize_typing_assist_pipeline,
    typing_assist_pipeline_for_auto_replace, CorrectionEngine, LayConfig, TypingAssistRuleConfig,
};
pub use crate::correction::Correction;
pub use crate::decoder::{
    choose_ranked_scoped_tail, decode_enter_autocorrect_tail, decode_manual_tail,
    decode_typing_assist_tail, rank_scoped_tail_candidates, CorrectionSource, CorrectionTrigger,
    DecoderAction, DecoderEditPlan, ManualDecodeRequest, ManualDecodeResult,
    RankedDecoderCandidate, RankedDecoderDecision,
};
pub use crate::desktop::{
    is_ru_layout_id, normalize_layout_id, parse_setxkbmap_layout, resolve_layout_backend,
    LayoutBackend,
};
pub use crate::dict::{convert, detect_direction, Direction};
pub use crate::engine::{
    decide_manual_correction, ManualCorrectionDecision, ManualCorrectionInput,
    ManualCorrectionPolicy,
};
pub use crate::keyboard::{
    is_typing_key, keycode_to_ru_char, keycode_to_us_char, map_events_to_layout,
    map_opposite_events, map_original_events, mixed_visual_latin_word_target_layout,
    original_event_char, preferred_layout_for_text, replay_layout_decision, split_event_words,
    text_to_key_events, text_to_uinput_runs, KeyEvent, ReplayLayoutDecision, TextInputRun,
};
pub use crate::lem::{best_candidate, rank_candidates, ScoredCandidate};
pub use crate::ngram::{
    en_score, ru_candidate_is_better, ru_candidate_margin, ru_score, tokenize_text, Lang,
};
pub use crate::phrase_reader::{
    correct_contextual_glued_tail, correct_glued_russian_phrase, correct_moved_prefix_letter_pair,
    correct_split_word_pair,
};
pub use crate::quality::score as quality_score;
pub use crate::russian_lexicon::{
    is_known_russian_word_or_form, russian_dictionary, russian_generated_form_dictionary,
    russian_short_dictionary, russian_tiny_dictionary,
};
pub use crate::text_backend::{
    ImeReplaceRequest, TextBackendCapabilities, TextBackendPreference, TextReplaceCapability,
};
pub use crate::text_edit::{
    apply_replacement_plan_to_text, committed_separator_is_preserved,
    plan_committed_tail_replacement, plan_text_replacement, replacement_plan_matches, tail_chars,
    TextReplacement,
};
pub use crate::typing_candidate::{
    classify_typing_confidence, rank_typing_candidates, TypingCandidate, TypingCandidateDecision,
    TypingCandidateFamily, TypingCandidateScore, TypingDecisionConfidence,
};
pub use crate::word_buffer::{
    PendingAutoUndo, UserLearningCorrection, WordBuffer, MAX_REPLACE_WORDS,
};
pub use crate::word_reader::{
    cyrillic_word_segmentations, cyrillic_word_splits, is_cyrillic_letters_only, is_cyrillic_word,
    split_edge_whitespace, split_word_punctuation, split_ws_segments, WordSplit,
};
pub use crate::word_recognizer::{
    is_plain_layout_autocorrect_risky, is_probably_completed_natural_word, recognize_token,
    WordIdentity, WordKind, WordLanguage, WordScript,
};

#[cfg(test)]
#[path = "core_tests.rs"]
mod tests;
