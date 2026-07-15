pub mod candidate_gate;
pub mod cell32;
pub mod context;
pub mod eval;
pub mod feedback;
pub mod journal;
mod journal_record;
pub mod l1;
pub mod l2;
mod l2_candidate_phase;
pub(crate) mod l2_wave_peak;
pub mod l3;
pub(crate) mod l3_phrase_gate;
pub mod l4_goal_state;
pub(crate) mod l4_signed_memory;
pub(crate) mod l4_signed_outcome;
pub mod learned;
pub mod lexical_attractor;
mod lexical_phase;
pub mod llmwave;
pub mod mode;
pub mod options;
pub mod packet;
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
pub use l2_candidate_phase::L2PhaseTrainingEntry;
pub(crate) use l2_candidate_phase::PhaseReadout;
pub use mode::{Mode8, ModeRole, CELL32_BYTES, MODES_PER_CELL32};
pub use options::WaveOptions;
pub use signal::{ActiveMode, LayerTrace, WaveDecision, WavePacket, WaveTrace, WordCandidate};
pub use trace::{run_wave_trace, run_wave_trace_with_options};
pub use usage_prior::UsagePriorSnapshot;

static L2_IME_WARMUP_STARTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(test)]
const SEMANTIC_WORD_SOURCE: &str = "SemanticWordCell32";
const PHRASE_FORECAST_CELL: &str = "PhraseForecastCell32";

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

pub fn write_l2_candidate_phase_memory_labeled<I>(
    path: &std::path::Path,
    entries: I,
) -> std::io::Result<usize>
where
    I: IntoIterator<Item = L2PhaseTrainingEntry>,
{
    l2_candidate_phase::write_phase_memory_from_labeled_entries(path, entries)
}

pub fn infer_l2_transition_operator(
    original: &str,
    candidate: &str,
    operation: &str,
) -> &'static str {
    crate::transition_relation::TransitionOperatorKind::infer(original, candidate, operation)
        .as_str()
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct L2TransitionPhaseShadowReadout {
    pub package_loaded: bool,
    pub operator_present: bool,
    pub operator_promoted: bool,
    pub verdict: &'static str,
    pub positive_micro: i64,
    pub anti_micro: i64,
    pub margin_micro: i64,
    pub threshold_micro: i64,
    pub positive_examples: u32,
    pub negative_examples: u32,
    pub positive_centers: u8,
    pub anti_centers: u8,
    pub covered_surfaces: u32,
    pub rejected_surfaces: u32,
}

#[derive(Clone, Debug)]
pub struct L2TransitionPhaseShadowEvaluator {
    inner: l2_candidate_phase::PhaseEvaluator,
}

impl L2TransitionPhaseShadowEvaluator {
    pub fn load(path: Option<&std::path::Path>) -> Self {
        Self {
            inner: l2_candidate_phase::PhaseEvaluator::load(path),
        }
    }

    pub fn readout(
        &self,
        original: &str,
        candidate: &str,
        operation: &str,
    ) -> L2TransitionPhaseShadowReadout {
        phase_shadow_readout(self.inner.readout(original, candidate, operation))
    }
}

pub fn l2_transition_phase_shadow_readout(
    original: &str,
    candidate: &str,
    operation: &str,
    path: Option<&std::path::Path>,
) -> L2TransitionPhaseShadowReadout {
    let readout = match path {
        Some(path) => {
            l2_candidate_phase::shadow_readout_from_path(original, candidate, operation, path)
        }
        None => l2_candidate_phase::shadow_readout(original, candidate, operation),
    };
    phase_shadow_readout(readout)
}

fn phase_shadow_readout(
    readout: l2_candidate_phase::PhaseReadout,
) -> L2TransitionPhaseShadowReadout {
    L2TransitionPhaseShadowReadout {
        package_loaded: readout.package_loaded,
        operator_present: readout.operator_present,
        operator_promoted: readout.operator_promoted,
        verdict: readout.verdict.as_str(),
        positive_micro: readout.positive_micro,
        anti_micro: readout.anti_micro,
        margin_micro: readout.margin_micro,
        threshold_micro: readout.threshold_micro,
        positive_examples: readout.positive_examples,
        negative_examples: readout.negative_examples,
        positive_centers: readout.positive_centers,
        anti_centers: readout.anti_centers,
        covered_surfaces: readout.covered_surfaces,
        rejected_surfaces: readout.rejected_surfaces,
    }
}

pub fn l2_candidate_phase_shadow(
    original: &str,
    candidate: &str,
    operation: &str,
) -> (bool, i64, bool) {
    let shadow = l2_transition_phase_shadow_readout(original, candidate, operation, None);
    (
        shadow.package_loaded,
        shadow.margin_micro,
        shadow.verdict == "support",
    )
}

pub fn l2_transition_phase_report_json(path: Option<&std::path::Path>) -> serde_json::Value {
    let owned;
    let path = match path {
        Some(path) => path,
        None => {
            owned = l2_candidate_phase::default_phase_memory_path();
            &owned
        }
    };
    l2_candidate_phase::phase_memory_report_json(path)
}

pub fn l2_transition_phase_proof_json(entries: &[L2PhaseTrainingEntry]) -> serde_json::Value {
    l2_candidate_phase::phase_proof_json(entries)
}

pub(crate) fn l2_transition_phase_readout(
    action_operator: &str,
    atoms: &[String],
) -> l2_candidate_phase::PhaseReadout {
    l2_candidate_phase::relation_readout(action_operator, atoms)
}

pub fn usage_debug_summary() -> (u64, usize, usize) {
    usage_prior::usage_debug_summary()
}

pub fn usage_memory_learned_report_json() -> serde_json::Value {
    usage_prior::usage_memory_learned_report_json()
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

pub fn record_rejected_ime_usage(context_tail: &str, rejected_text: &str) {
    usage_prior::record_rejected_ime_if_enabled(context_tail, rejected_text);
}

pub fn record_rejected_candidate_usage(
    context_tail: &str,
    rejected_text: &str,
    source: &str,
    operation: &str,
) {
    usage_prior::record_rejected_candidate_if_enabled(
        context_tail,
        rejected_text,
        source,
        operation,
    );
}

pub fn warm_up() {
    l2::warm_up_surface_motif_memory();
    let _ = llmwave::load_default_memory();
}

pub fn warm_up_for_ime() {
    l2::warm_up_surface_motif_memory();
    let _ = llmwave::load_default_memory();
}

pub fn warm_up_l2_for_ime() {
    L2_IME_WARMUP_STARTED.store(true, std::sync::atomic::Ordering::Relaxed);
    l2::warm_up_ime_word_candidate_memory();
    candidate_gate::warm_up_live_candidate_readout();
}

pub fn warm_up_l3_phrase_memory() {
    let _ = llmwave::load_default_memory();
}

pub fn ensure_l2_ime_warmup_started() {
    if L2_IME_WARMUP_STARTED
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
        )
        .is_ok()
    {
        std::thread::spawn(warm_up_l2_for_ime);
    }
}
