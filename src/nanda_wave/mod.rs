pub mod candidate_gate;
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
mod l2_broad_index;
mod l2_candidate_phase;
mod l2_center_memory;
pub mod l3;
pub(crate) mod l3_phrase_gate;
pub mod l4_goal_state;
pub(crate) mod l4_signed_memory;
pub(crate) mod l4_signed_outcome;
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
mod surface_bank;
mod surface_wave;
pub mod trace;
pub(crate) mod usage_prior;

pub use eval::{evaluate_wave, evaluate_wave_with_options, WaveEvalResult, WaveEvalStats};
pub use mode::{Mode8, ModeRole, CELL32_BYTES, MODES_PER_CELL32};
pub use options::WaveOptions;
pub use signal::{ActiveMode, LayerTrace, WaveDecision, WavePacket, WaveTrace, WordCandidate};
pub use trace::{run_wave_trace, run_wave_trace_with_options};
pub use usage_prior::UsagePriorSnapshot;

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

pub fn cached_usage_prior_snapshot() -> UsagePriorSnapshot {
    usage_prior::cached_usage_prior_snapshot()
}

pub fn l2_surface_words_by_usage(limit: usize) -> Vec<String> {
    usage_prior::l2_surface_words_by_usage(limit)
}

pub fn default_l2_candidate_phase_memory_path() -> std::path::PathBuf {
    l2_candidate_phase::default_phase_memory_path()
}

pub fn write_l2_candidate_phase_memory<I>(
    path: &std::path::Path,
    entries: I,
) -> std::io::Result<usize>
where
    I: IntoIterator<Item = (String, String, String, usize)>,
{
    l2_candidate_phase::write_phase_memory_from_entries(path, entries)
}

pub fn l2_candidate_phase_shadow(
    original: &str,
    candidate: &str,
    operation: &str,
) -> (bool, i64, bool) {
    let shadow = l2_candidate_phase::shadow_admission(original, candidate, operation);
    (shadow.package_loaded, shadow.margin_micro, shadow.admitted)
}

pub fn usage_debug_summary() -> (u64, usize, usize) {
    usage_prior::usage_debug_summary()
}

pub fn balanced_l2_surface_words<I>(source: I, limit: usize) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    surface_bank::balanced_l2_surface_words(source, limit)
}

pub fn normalize_l2_surface_word(word: &str) -> Option<String> {
    surface_bank::normalize_l2_surface_word(word)
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
