pub mod candidate_gate;
pub mod cell32;
pub mod context;
pub(crate) mod context_phase;
pub mod eval;
pub mod feedback;
pub mod journal;
mod journal_record;
pub mod l1;
pub mod l2;
mod l2_candidate_phase;
pub(crate) mod l2_wave_peak;
pub mod l3;
mod l3_context_metrics;
pub(crate) mod l3_phrase_gate;
pub(crate) mod l4_active_disambiguation;
pub mod l4_goal_state;
pub(crate) mod l4_hidden_state;
pub(crate) mod l4_phase_witness;
pub(crate) mod l4_signed_memory;
pub mod learned;
pub mod lexical_attractor;
mod lexical_phase;
pub mod llmwave;
pub mod mode;
pub mod options;
pub mod packet;
pub mod pattern_wave;
pub(crate) mod phase_field;
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
pub(crate) use l2_candidate_phase::{PhaseReadout, PhaseVerdict};
pub use mode::{Mode8, ModeRole, CELL32_BYTES, MODES_PER_CELL32};
pub use options::WaveOptions;
pub use signal::{ActiveMode, LayerTrace, WaveDecision, WavePacket, WaveTrace, WordCandidate};
pub use trace::{run_wave_trace, run_wave_trace_with_options};
pub use usage_prior::UsagePriorSnapshot;

pub fn l3_context_report_json(
    cases: &[crate::eval_cases::EvalCase],
    full_cases: usize,
) -> serde_json::Value {
    l3_context_metrics::report_json(cases, full_cases)
}

pub fn compile_l3_context_phase_memory(
    corpus_path: &std::path::Path,
    lexicon_path: &std::path::Path,
    output_path: &std::path::Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> std::io::Result<serde_json::Value> {
    let corpus_text = std::fs::read_to_string(corpus_path)?;
    let lexicon_text = std::fs::read_to_string(lexicon_path)?;
    let (package, report) =
        context_phase::compile_context_phase(context_phase::ContextPhaseCompileInput {
            corpus_text: &corpus_text,
            lexicon_text: &lexicon_text,
            max_fragments,
            min_profile_support,
        });
    context_phase::write_package(output_path, &package)?;
    let mut value = serde_json::to_value(report).map_err(std::io::Error::other)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("corpus".to_string(), serde_json::json!(corpus_path));
        object.insert("lexicon".to_string(), serde_json::json!(lexicon_path));
        object.insert("output".to_string(), serde_json::json!(output_path));
        object.insert(
            "artifact_bytes".to_string(),
            serde_json::json!(std::fs::metadata(output_path)
                .map(|meta| meta.len())
                .unwrap_or_default()),
        );
    }
    Ok(value)
}

pub fn l3_context_phase_status_json(path: Option<&std::path::Path>) -> serde_json::Value {
    let path = path
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(context_phase::default_memory_path);
    context_phase::package_report(&path)
}

pub fn prove_l3_context_phase_memory(
    corpus_path: &std::path::Path,
    lexicon_path: &std::path::Path,
    max_fragments: usize,
    min_profile_support: u32,
) -> std::io::Result<serde_json::Value> {
    let corpus_text = std::fs::read_to_string(corpus_path)?;
    let lexicon_text = std::fs::read_to_string(lexicon_path)?;
    serde_json::to_value(context_phase::prove_context_phase(
        context_phase::ContextPhaseCompileInput {
            corpus_text: &corpus_text,
            lexicon_text: &lexicon_text,
            max_fragments,
            min_profile_support,
        },
    ))
    .map_err(std::io::Error::other)
}

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

pub fn compile_usage_feedback_snapshot(
    input: &std::path::Path,
    output: &std::path::Path,
) -> std::io::Result<serde_json::Value> {
    usage_prior::compile_usage_feedback_snapshot(input, output)
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

pub fn transition_surface_key(
    original: &str,
    candidate: &str,
    source: &str,
    operation: &str,
) -> String {
    crate::typing_memory::transition_surface_key(original, candidate, source, operation)
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
    pub(crate) lexical_positive_micro: i64,
    pub(crate) lexical_anti_micro: i64,
    pub(crate) lexical_margin_micro: i64,
    pub(crate) lexical_threshold_micro: i64,
    pub(crate) lexical_positive_examples: u32,
    pub(crate) lexical_negative_examples: u32,
    pub(crate) lexical_positive_centers: u8,
    pub(crate) lexical_anti_centers: u8,
    pub(crate) lexical_competition_ready: bool,
    pub(crate) lexical_verdict: &'static str,
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
        lexical_positive_micro: readout.lexical_positive_micro,
        lexical_anti_micro: readout.lexical_anti_micro,
        lexical_margin_micro: readout.lexical_margin_micro,
        lexical_threshold_micro: readout.lexical_threshold_micro,
        lexical_positive_examples: readout.lexical_positive_examples,
        lexical_negative_examples: readout.lexical_negative_examples,
        lexical_positive_centers: readout.lexical_positive_centers,
        lexical_anti_centers: readout.lexical_anti_centers,
        lexical_competition_ready: readout.lexical_competition_ready,
        lexical_verdict: readout.lexical_verdict.as_str(),
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
    original: &str,
    candidate: &str,
) -> l2_candidate_phase::PhaseReadout {
    l2_candidate_phase::relation_readout(action_operator, atoms, original, candidate)
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
    surface_bank::normalize_surface_bank_word(word)
}

pub fn record_typed_tail_usage(tail: &str) {
    usage_prior::record_typed_tail_if_enabled(tail);
}

pub fn record_accepted_fix_usage(from: &str, to: &str) {
    usage_prior::record_accepted_fix_if_enabled(from, to);
}

pub fn record_accepted_layout_projection_usage(from: &str, to: &str) {
    usage_prior::record_accepted_layout_projection_if_enabled(from, to);
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
    context_phase::warm_default_memory();
    let _ = llmwave::load_default_memory();
}

pub fn warm_up_for_ime() {
    l2::warm_up_surface_motif_memory();
    context_phase::warm_default_memory();
    let _ = llmwave::load_default_memory();
}

pub fn warm_up_l2_for_ime() {
    L2_IME_WARMUP_STARTED.store(true, std::sync::atomic::Ordering::Relaxed);
    l2::warm_up_ime_word_candidate_memory();
    candidate_gate::warm_up_live_candidate_readout();
}

pub fn warm_up_l3_phrase_memory() {
    context_phase::warm_default_memory();
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
