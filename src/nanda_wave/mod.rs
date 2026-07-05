pub mod cell32;
pub mod context;
pub mod context_wave;
pub mod eval;
pub mod feedback;
pub mod journal;
mod journal_record;
pub mod l1;
mod l1_center_memory;
pub mod l2;
mod l2_center_memory;
pub mod l3;
pub(crate) mod l3_phrase_gate;
pub mod learned;
pub mod lexical_attractor;
pub mod llmwave;
pub mod mode;
pub mod options;
pub mod packet;
pub mod pattern_memory;
pub mod pattern_wave;
pub mod precognition;
pub mod resonance_memory;
pub mod signal;
pub mod structural_relation;
mod surface_wave;
pub mod trace;
pub(crate) mod usage_prior;

pub use eval::{evaluate_wave, evaluate_wave_with_options, WaveEvalResult, WaveEvalStats};
pub use mode::{Mode8, ModeRole, CELL32_BYTES, MODES_PER_CELL32};
pub use options::WaveOptions;
pub use signal::{ActiveMode, LayerTrace, WaveDecision, WavePacket, WaveTrace, WordCandidate};
pub use trace::{run_wave_trace, run_wave_trace_with_options};

pub fn word_usage_prior(word: &str) -> f32 {
    usage_prior::word_usage_prior(word)
}

pub fn context_word_usage_prior(context: &[String], word: &str) -> f32 {
    usage_prior::context_word_usage_prior(context, word)
}

pub fn cached_word_usage_prior(word: &str) -> f32 {
    usage_prior::word_usage_prior_cached(word)
}

pub fn cached_context_word_usage_prior(context: &[String], word: &str) -> f32 {
    usage_prior::context_word_usage_prior_cached(context, word)
}

pub fn record_typed_tail_usage(tail: &str) {
    usage_prior::record_typed_tail_if_enabled(tail);
}

pub fn record_accepted_fix_usage(from: &str, to: &str) {
    usage_prior::record_accepted_fix_if_enabled(from, to);
}

pub fn record_accepted_ime_usage(context_tail: &str, accepted_text: &str) {
    usage_prior::record_accepted_ime_if_enabled(context_tail, accepted_text);
}

pub fn warm_up() {
    context_wave::warm_up();
    l2::warm_up_surface_motif_memory();
    let _ = llmwave::load_default_memory();
}

pub fn warm_up_for_ime() {
    context_wave::warm_up_prefix_completion_indexes();
    l2::warm_up_surface_motif_memory();
    let _ = llmwave::load_default_memory();
}

pub fn warm_up_l2_for_ime() {
    l2::warm_up_ime_word_candidate_memory();
}
