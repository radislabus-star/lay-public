//! lay — библиотечная часть. Используется и из `bin/lay` (CLI),
//! и из `bin/lay-daemon` (фоновый daemon на двойной Shift).

pub mod action_log;
pub(crate) mod candidate_ranker;
pub mod config;
pub mod core;
pub mod correction;
pub(crate) mod data_lines;
pub mod decoder;
pub mod desktop;
pub mod dict;
pub mod engine;
pub mod eval_cases;
pub mod keyboard;
pub mod layout_autoswitch;
pub mod lem;
pub mod lexicon;
pub mod llm;
pub(crate) mod llm_backend;
pub mod mixed_script_repair;
pub mod nanda_wave;
pub mod ngram;
pub(crate) mod phrase_candidates;
pub(crate) mod phrase_lexicon;
pub mod phrase_reader;
pub(crate) mod phrase_score;
#[doc(hidden)]
pub mod private_file;
pub mod quality;
pub mod ru_typo;
pub(crate) mod russian_chars;
pub mod russian_lexicon;
pub(crate) mod russian_prefixes;
pub(crate) mod russian_typo_candidates;
pub(crate) mod russian_typo_scoring;
pub mod scoped_tail;
pub mod stats;
pub mod text_backend;
pub(crate) mod text_case;
pub mod text_edit;
pub mod text_metrics;
pub(crate) mod token_language;
pub mod typing_assist;
pub mod typing_candidate;
pub mod typing_context;
pub(crate) mod typing_pipeline;
pub mod typing_replacements;
pub(crate) mod typing_rule_graph;
pub mod word_buffer;
pub mod word_reader;
pub mod word_recognizer;
pub mod x11_layout;

#[cfg(test)]
#[path = "typing_assist_test_fixtures.rs"]
pub(crate) mod typing_assist_test_fixtures;
