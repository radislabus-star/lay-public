use super::*;
use evdev::KeyCode;
use lay::config::{
    default_typing_assist_pipeline, default_typing_assist_rules, normalize_typing_assist_pipeline,
    typing_assist_pipeline_for_auto_replace, CorrectionEngine, LayConfig,
};
use lay::correction::Correction;
use lay::decoder::{decode_manual_tail, CorrectionSource, DecoderAction, ManualDecodeRequest};
use lay::keyboard::{
    is_cyrillic_letter, is_layout_decision_key, map_events_to_layout, map_opposite_events,
    map_original_events, preferred_layout_for_text, replay_layout_decision, text_to_uinput_runs,
    ReplayLayoutDecision,
};
use lay::text_edit::{plan_committed_tail_replacement, plan_text_replacement, TextReplacement};
use lay::typing_assist::{
    are_ru_keyboard_neighbors, correct_extra_letters, correct_missing_letter,
    correct_wrong_layout_ascii_technical_token, decide_completed_scope_word, decide_correction,
    decide_scoped_tail_correction, effective_replace_words, is_ascii_technical_token,
    is_known_russian_word_or_form, promoted_replacement_for_token,
    reference_russian_generated_form_dictionary, select_typing_assist_with_pipeline,
    should_force_replay_for_short_fragment, should_keep_plain_cyrillic_before_ascii_technical,
};
use lay::word_buffer::{UserLearningCorrection, WordBuffer};
use std::collections::{BTreeMap, HashSet};
use std::time::{Duration, Instant};

#[path = "tests/fixtures.rs"]
mod fixtures;
use fixtures::{first_fixture_row, fixture_lines, fixture_row_by_id, fixture_rows};
#[path = "tests/harness.rs"]
mod harness;
use harness::*;
#[path = "tests/typing_assist_harness.rs"]
mod typing_assist_harness;
use typing_assist_harness::*;

#[path = "tests/config_contract.rs"]
mod config_contract;
#[path = "tests/enter_autocorrect.rs"]
mod enter_autocorrect;
#[path = "tests/field_context.rs"]
mod field_context;
#[path = "tests/layout_backend.rs"]
mod layout_backend;
#[path = "tests/learning.rs"]
mod learning;
#[path = "tests/learning_log.rs"]
mod learning_log;
#[path = "tests/runtime_state.rs"]
mod runtime_state;
#[path = "tests/scoped_tail.rs"]
mod scoped_tail;
#[path = "tests/text_output_contract.rs"]
mod text_output_contract;
#[path = "tests/typing_assist_deferred.rs"]
mod typing_assist_deferred;
#[path = "tests/typing_assist_rules.rs"]
mod typing_assist_rules;
