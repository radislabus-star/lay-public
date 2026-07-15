//! Public compatibility facade for typing-assist and smart correction.
//!
//! Keep rule execution in `typing_pipeline` and smart manual scope correction in
//! `scoped_tail`. This facade preserves the old API surface for CLI, tests and
//! daemon modules.

mod auto_undo;

pub use crate::layout_autoswitch::{
    correct_duplicate_layout_prefix_on_ascii_token, correct_wrong_layout_ascii_technical_token,
    should_keep_plain_cyrillic_before_ascii_technical,
};
pub use crate::ru_typo::{
    are_ru_keyboard_neighbors, correct_extra_letters, correct_missing_letter,
};
pub use crate::russian_lexicon::{
    is_known_russian_word_or_form, russian_generated_form_dictionary,
};
pub use crate::scoped_tail::{
    decide_completed_scope_word, decide_correction, decide_scoped_tail_correction,
    decide_scoped_tail_correction_with_lem, decide_scoped_tail_correction_with_options,
    effective_replace_words, repair_cyrillic_prefix_before_ascii_tail, scoped_tail_lem_candidates,
    should_force_replay_for_short_fragment, ScopedTailOptions,
};
pub use crate::typing_pipeline::{
    explain_typing_assist_with_pipeline, select_typing_assist, select_typing_assist_exact,
    select_typing_assist_with_pipeline, warm_up, warm_up_hot, TypingAssistExplanation,
    TypingRuleEvaluation,
};
pub use crate::typing_replacements::{
    apply_auto_replace, apply_manual_replay_auto_replace, contains_visual_b_word,
    promoted_replacement_for_token, remember_promoted_replacement, safe_promoted_replacement,
    REPLACEMENTS_PATH,
};
pub use crate::word_reader::{is_cyrillic_word, split_edge_whitespace, split_ws_segments};
pub use crate::word_recognizer::is_ascii_technical_token;
pub use auto_undo::{
    typing_correction_should_skip_auto_undo, typing_rule_should_skip_auto_undo, NANDA_WAVE_RULE_ID,
};
