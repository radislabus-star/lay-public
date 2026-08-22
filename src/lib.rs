//! lay — библиотечная часть. Используется и из `bin/lay` (CLI),
//! и из `bin/lay-daemon` (фоновый daemon на двойной Shift).

pub mod action_log;
#[cfg(test)]
mod architecture_contract;
pub(crate) mod candidate_contract;
pub(crate) mod candidate_explanation;
pub(crate) mod candidate_ranker;
pub mod config;
pub mod correction;
pub(crate) mod correction_bayes;
pub mod correction_core;
pub(crate) mod data_lines;
pub mod debug_log;
pub mod decoder;
pub mod desktop;
pub mod dict;
pub mod engine;
pub mod eval_cases;
pub mod exact_layout_authority;
pub mod hot_field;
pub mod ime_correction;
pub mod input_gate;
pub mod keyboard;
pub(crate) mod language_action;
pub(crate) mod layout_autoswitch;
mod lexical_surface_atoms;
pub mod lexicon;
pub mod llm;
pub(crate) mod llm_backend;
pub mod manual_toggle;
pub(crate) mod mixed_script_repair;
pub mod nanda_wave;
pub mod ngram;
pub(crate) mod phrase_candidates;
pub(crate) mod phrase_lexicon;
pub(crate) mod phrase_reader;
pub(crate) mod phrase_score;
#[doc(hidden)]
pub mod private_file;
pub(crate) mod quality;
pub(crate) mod ru_typo;
pub(crate) mod russian_chars;
pub mod russian_lexicon;
pub(crate) mod russian_prefixes;
pub(crate) mod russian_typo_candidates;
pub(crate) mod russian_typo_scoring;
pub(crate) mod scoped_tail;
mod stable_hash;
pub mod stats;
pub mod text_backend;
pub(crate) mod text_case;
pub mod text_edit;
pub mod text_metrics;
#[doc(hidden)]
pub mod time;
pub(crate) mod token_language;
pub(crate) mod transition_relation;
pub mod typing_assist;
pub(crate) mod typing_candidate;
pub mod typing_context;
pub mod typing_cpu;
pub(crate) mod typing_memory;
pub(crate) mod typing_pipeline;
pub mod typing_replacements;
pub(crate) mod typing_rule_graph;
pub(crate) mod typing_scene;
pub(crate) mod typing_transition;
pub mod word_buffer;
pub mod word_reader;
pub(crate) mod word_recognizer;
pub mod x11_layout;

#[cfg(test)]
#[path = "typing_assist_test_fixtures.rs"]
pub(crate) mod typing_assist_test_fixtures;
