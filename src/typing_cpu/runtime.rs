use crate::nanda_wave::{candidate_gate, llmwave};

pub use crate::nanda_wave::WaveOptions as TypingCpuOptions;
pub use candidate_gate::{LiveCompletionCandidate, LiveCompletionRequest};

#[derive(Debug, Clone, PartialEq)]
pub struct PhraseForecastCandidate {
    pub text: String,
    pub score: f32,
}

/// Stateless library front door used by live typing adapters.
pub struct TypingCpu;

impl TypingCpu {
    pub fn live_completion_candidates(
        request: LiveCompletionRequest<'_>,
    ) -> Vec<LiveCompletionCandidate> {
        candidate_gate::live_completion_candidates(request)
    }

    pub fn ime_candidate_memory_is_warm() -> bool {
        crate::nanda_wave::l2::ime_word_candidate_memory_is_warm()
    }

    pub fn ensure_ime_warmup_started() {
        crate::nanda_wave::ensure_l2_ime_warmup_started();
    }

    pub fn warm_l2_for_ime() {
        crate::nanda_wave::warm_up_l2_for_ime();
    }

    pub fn warm_l3_phrase_memory() {
        crate::nanda_wave::warm_up_l3_phrase_memory();
    }

    pub fn warm_all() {
        crate::nanda_wave::warm_up();
    }

    pub fn phrase_memory_is_warm() -> bool {
        llmwave::default_memory_is_warm()
    }

    pub fn phrase_context_tokens(text: &str) -> Vec<String> {
        llmwave::tokenize(text)
    }

    pub fn phrase_forecast_candidates(text: &str) -> Vec<PhraseForecastCandidate> {
        llmwave::with_default_memory(|memory| {
            llmwave::phrase_forecast_candidates(text, memory)
                .into_iter()
                .map(|candidate| PhraseForecastCandidate {
                    text: candidate.text,
                    score: (candidate.energy - candidate.risk).clamp(0.0, 1.0),
                })
                .collect()
        })
    }

    pub fn record_typed_tail(tail: &str) {
        crate::nanda_wave::record_typed_tail_usage(tail);
    }

    pub fn record_accepted_completion(context_tail: &str, accepted_text: &str) {
        crate::nanda_wave::record_accepted_ime_usage(context_tail, accepted_text);
    }

    pub fn record_confirmed_completion_prediction(context_tail: &str, predicted_text: &str) {
        crate::nanda_wave::record_confirmed_ime_prediction_usage(context_tail, predicted_text);
    }

    pub fn record_rejected_completion(context_tail: &str, rejected_text: &str) {
        crate::nanda_wave::record_rejected_ime_usage(context_tail, rejected_text);
    }

    pub fn record_user_correction(original: &str, proposed: &str, accepted: &str, kind: &str) {
        if accepted != proposed {
            crate::nanda_wave::record_rejected_candidate_usage(
                original,
                proposed,
                "user_correction",
                kind,
            );
        }
        crate::nanda_wave::record_accepted_fix_usage(original, accepted);
    }

    pub fn record_accepted_layout_projection(original: &str, replacement: &str) {
        crate::nanda_wave::record_accepted_layout_projection_usage(original, replacement);
    }

    pub fn record_precognition_tick(stage: &str, text: &str, include_trace: bool) {
        crate::nanda_wave::precognition::record_precognition_tick(stage, text, include_trace);
        llmwave::record_phrase_experience(stage, text);
    }

    pub fn record_typing_assist_trace(
        original: &str,
        replacement: &str,
        options: &TypingCpuOptions,
        include_text: bool,
    ) {
        let trace = crate::nanda_wave::run_wave_trace_with_options(original, options);
        crate::nanda_wave::journal::record_runtime_trace_with_text_policy(
            "runtime:typing-assist",
            "typing-assist",
            &trace,
            Some(replacement),
            include_text,
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn live_ime_sources_use_the_typing_cpu_front_door() {
        for source in [
            include_str!("../bin/lay_ibus_engine/preedit_readout.rs"),
            include_str!("../bin/lay_ibus_engine/composition_commit.rs"),
            include_str!("../bin/lay_ibus_engine/server.rs"),
            include_str!("../bin/lay_ibus_engine/state.rs"),
        ] {
            let production_source = source.split("#[cfg(test)]").next().unwrap_or(source);
            assert!(
                !production_source.contains("lay::nanda_wave::"),
                "production IME source bypasses TypingCpu"
            );
        }
    }
}
