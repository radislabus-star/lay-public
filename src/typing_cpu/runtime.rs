use crate::nanda_wave::{candidate_gate, llmwave};

pub use crate::nanda_wave::L11ServiceEnsureReport;
pub use crate::nanda_wave::WaveOptions as TypingCpuOptions;
pub use candidate_gate::{LiveCompletionCandidate, LiveCompletionRequest, LiveCompletionTiming};

#[derive(Debug, Clone, PartialEq)]
pub struct PhraseForecastCandidate {
    pub text: String,
    pub score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedSystemTransition {
    LayoutProjection,
    Correction,
}

impl ObservedSystemTransition {
    pub(crate) fn evidence_source(self) -> crate::typing_memory::TypingMemoryEvidenceSource {
        match self {
            Self::LayoutProjection => crate::typing_memory::TypingMemoryEvidenceSource::Layout,
            Self::Correction => crate::typing_memory::TypingMemoryEvidenceSource::Autocorrect,
        }
    }

    pub(crate) fn operation(self) -> crate::typing_memory::TypingMemoryOperation {
        match self {
            Self::LayoutProjection => crate::typing_memory::TypingMemoryOperation::LayoutProjection,
            Self::Correction => crate::typing_memory::TypingMemoryOperation::Replacement,
        }
    }
}

/// Stateless library front door used by live typing adapters.
pub struct TypingCpu;

impl TypingCpu {
    pub fn live_completion_candidates(
        request: LiveCompletionRequest<'_>,
    ) -> Vec<LiveCompletionCandidate> {
        candidate_gate::live_completion_candidates(request)
    }

    pub fn clear_last_live_completion_timing() {
        candidate_gate::clear_last_live_completion_timing();
    }

    pub fn last_live_completion_timing() -> LiveCompletionTiming {
        candidate_gate::last_live_completion_timing()
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

    pub fn ensure_l11_service_started() -> std::io::Result<Option<L11ServiceEnsureReport>> {
        crate::nanda_wave::ensure_l11_service_started()
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

    pub fn record_edited_completion(
        context_tail: &str,
        typed_prefix: &str,
        suggested_text: &str,
        final_text: &str,
    ) {
        let shared_morphology_identity =
            crate::nanda_wave::l2_field::surfaces_share_morphology_identity(
                suggested_text,
                final_text,
            );
        crate::nanda_wave::record_edited_ime_usage(
            context_tail,
            typed_prefix,
            suggested_text,
            final_text,
            shared_morphology_identity,
        );
    }

    pub fn completion_edit_geometry_is_linked(
        typed_prefix: &str,
        suggested_text: &str,
        final_text: &str,
    ) -> bool {
        let shared_morphology_identity =
            crate::nanda_wave::l2_field::surfaces_share_morphology_identity(
                suggested_text,
                final_text,
            );
        crate::typing_memory::completion_edit_geometry_is_linked_with_identity(
            typed_prefix,
            suggested_text,
            final_text,
            shared_morphology_identity,
        )
    }

    pub fn record_confirmed_completion_prediction(context_tail: &str, predicted_text: &str) {
        crate::nanda_wave::record_confirmed_ime_prediction_usage(context_tail, predicted_text);
    }

    pub fn learning_target_is_attested(text: &str) -> bool {
        crate::typing_memory::learning_target_is_attested(text)
    }

    pub fn record_rejected_completion(context_tail: &str, rejected_text: &str) {
        crate::nanda_wave::record_rejected_ime_usage(context_tail, rejected_text);
    }

    pub fn record_user_correction(original: &str, proposed: &str, accepted: &str, kind: &str) {
        crate::nanda_wave::record_confirmed_user_correction_usage(
            original, proposed, accepted, kind,
        );
    }

    pub fn record_observed_system_apply(
        original: &str,
        replacement: &str,
        transition: ObservedSystemTransition,
    ) {
        crate::nanda_wave::record_observed_system_apply_usage(original, replacement, transition);
    }

    pub fn record_reverted_system_apply(
        original: &str,
        rejected: &str,
        transition: ObservedSystemTransition,
    ) {
        crate::nanda_wave::record_reverted_system_apply_usage(original, rejected, transition);
    }

    pub fn record_accepted_layout_projection(original: &str, replacement: &str) {
        crate::nanda_wave::record_accepted_layout_projection_usage(original, replacement);
    }

    pub fn record_precognition_tick(stage: &str, text: &str, include_trace: bool) {
        crate::nanda_wave::precognition::record_precognition_tick(stage, text, include_trace);
        if crate::typing_memory::phrase_is_attested_for_learning(text) {
            llmwave::record_phrase_experience(stage, text);
        }
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
