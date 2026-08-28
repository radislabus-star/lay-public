// Candidate-source adapter for the IBus preedit surface. It may query shared
// L2/L3 memory, but does not rank candidates, mutate text, or emit IBus signals.

use lay::typing_cpu::{
    should_query_llmwave_phrase_suffix, ImeCandidateSource, LiveCompletionRequest,
    LiveCompletionTiming, TypingCpu,
};

impl LayIbusEngine {
    fn live_completion_input_is_active(&self) -> bool {
        !self.buffer.is_empty()
            || (self.last_tail_input_at.is_some() && !self.preedit_fast.token.is_empty())
    }

    #[cfg(test)]
    fn semantic_phrase_candidates(&self) -> Vec<ImeCandidateProposal> {
        self.precognition_input()
            .map(|input| semantic_phrase_candidates_for_input(&input))
            .unwrap_or_default()
    }

    #[cfg(test)]
    fn word_candidate_proposals(&self) -> Vec<ImeCandidateProposal> {
        self.precognition_input()
            .map(|input| word_candidate_proposals_for_input(&input))
            .unwrap_or_default()
    }

    fn live_word_readout_input<'a>(&self, tail: &'a str) -> Option<(&'a str, &'a str)> {
        if self.preedit_fast.is_ascii_live_candidate_token()
            && tail.ends_with(self.preedit_fast.token.as_str())
        {
            let split = tail.len().saturating_sub(self.preedit_fast.token.len());
            return Some(tail.split_at(split));
        }
        split_last_alphabetic_token(tail)
    }

    #[cfg(test)]
    fn llmwave_phrase_candidates_from_memory(
        &self,
        tail: &str,
        memory: &lay::nanda_wave::llmwave::LlmWaveMemory,
    ) -> Vec<ImeCandidateProposal> {
        let max_suffix_chars = self.precognition_max_suffix_chars();
        lay::nanda_wave::llmwave::phrase_forecast_candidates(tail, memory)
            .into_iter()
            .take(6)
            .filter_map(|candidate| {
                let suffix = lay::typing_cpu::phrase_candidate_suffix(
                    tail,
                    &candidate.text,
                    max_suffix_chars,
                )?;
                Some(ImeCandidateProposal::new(
                    suffix,
                    (candidate.energy - candidate.risk).clamp(0.0, 1.0),
                    ImeCandidateSource::L3Context,
                ))
            })
            .collect()
    }
}

fn semantic_phrase_candidates_for_input(input: &PrecognitionInput) -> Vec<ImeCandidateProposal> {
    if input.correction_safety != lay::config::CorrectionSafety::Experimental
        || input.tail.trim_end().chars().count() < 6
        || !should_query_llmwave_phrase_suffix(&input.tail)
        || !TypingCpu::phrase_memory_is_warm()
    {
        return Vec::new();
    }

    // Phrase memory is suffix-only. Whole-token replacements remain available
    // to correction routes but are not projected onto live IBus preedit.
    llmwave_phrase_candidates_for_input(&input.tail, input.max_suffix_chars)
}

#[cfg(test)]
fn word_candidate_proposals_for_input(input: &PrecognitionInput) -> Vec<ImeCandidateProposal> {
    word_candidate_readout_for_input(input).0
}

fn word_candidate_readout_for_input(
    input: &PrecognitionInput,
) -> (Vec<ImeCandidateProposal>, LiveCompletionTiming) {
    if input.correction_safety == lay::config::CorrectionSafety::Strict {
        return (Vec::new(), LiveCompletionTiming::default());
    }
    if !TypingCpu::ime_candidate_memory_is_warm() {
        TypingCpu::ensure_ime_warmup_started();
        return (Vec::new(), LiveCompletionTiming::default());
    }
    let partial_len = input.partial.chars().count();
    let ru_surface = input
        .partial
        .chars()
        .all(|ch| matches!(ch, 'а'..='я' | 'ё'));
    let ascii_layout_surface = input.partial.chars().any(|ch| ch.is_ascii_alphabetic())
        && input.partial.chars().all(|ch| {
            ch.is_ascii_alphabetic() || lay::typing_cpu::is_ascii_layout_letter_symbol(ch)
        });
    if !(PREEDIT_RU_PREFIX_MIN_CHARS..=18).contains(&partial_len)
        || !(ru_surface || ascii_layout_surface)
    {
        return (Vec::new(), LiveCompletionTiming::default());
    }

    let readout = TypingCpu::live_completion_readout(LiveCompletionRequest {
        context_prefix: &input.context_prefix,
        partial: &input.partial,
        max_suffix_chars: input.max_suffix_chars,
        active_composition: input.active_composition,
        allow_short_lexical: true,
        limit: PREEDIT_RU_WAVE_CANDIDATE_LIMIT,
    });
    let timing = readout.timing;
    // The shared candidate gate still owns and exposes replacement candidates
    // to correction routes. Live IBus preedit only projects suffix completion;
    // replacing the visible token after a background result is a disruptive UI
    // transition and is not passive completion.
    let proposals = readout
        .candidates
        .into_iter()
        .filter(|candidate| !candidate.replacement)
        .enumerate()
        .map(|(order, candidate)| {
            ImeCandidateProposal::new(
                candidate.suffix,
                candidate.score,
                ImeCandidateSource::L2Completion,
            )
            .with_authority_order(order)
        })
        .collect();
    (proposals, timing)
}

fn llmwave_phrase_candidates_for_input(
    tail: &str,
    max_suffix_chars: usize,
) -> Vec<ImeCandidateProposal> {
    TypingCpu::phrase_forecast_candidates(tail)
        .into_iter()
        .take(6)
        .filter_map(|candidate| {
            let suffix =
                lay::typing_cpu::phrase_candidate_suffix(tail, &candidate.text, max_suffix_chars)?;
            Some(ImeCandidateProposal::new(
                suffix,
                candidate.score,
                ImeCandidateSource::L3Context,
            ))
        })
        .collect()
}

pub(crate) fn materialize_precognition_candidates(
    input: &PrecognitionInput,
) -> Vec<ImeCandidateProposal> {
    materialize_precognition_candidates_observed(input).candidates
}

pub(crate) struct PrecognitionMaterializationTiming {
    pub(crate) total_us: u64,
    pub(crate) word_us: u64,
    pub(crate) semantic_us: u64,
    pub(crate) word: LiveCompletionTiming,
}

pub(crate) struct PrecognitionMaterialization {
    pub(crate) candidates: Vec<ImeCandidateProposal>,
    pub(crate) timing: PrecognitionMaterializationTiming,
}

pub(crate) fn materialize_precognition_candidates_observed(
    input: &PrecognitionInput,
) -> PrecognitionMaterialization {
    let timing_enabled = trace::enabled();
    let total_started = timing_enabled.then(Instant::now);
    let semantic_started = timing_enabled.then(Instant::now);
    let semantic_candidates = semantic_phrase_candidates_for_input(input);
    let semantic_us = elapsed_us(semantic_started);

    let word_started = timing_enabled.then(Instant::now);
    let (word_candidates, word_timing) = word_candidate_readout_for_input(input);
    let word_us = elapsed_us(word_started);

    let mut proposals = Vec::with_capacity(semantic_candidates.len() + word_candidates.len());
    proposals.extend(semantic_candidates);
    proposals.extend(word_candidates);
    proposals.retain(|proposal| {
        !proposal_repeats_declined_target(&input.declined_target_surfaces, &input.partial, proposal)
    });
    let candidates = select_ime_candidate_proposals(ImeCandidateReadoutRequest {
        proposals: &proposals,
        limit: proposals.len(),
    });

    PrecognitionMaterialization {
        candidates,
        timing: PrecognitionMaterializationTiming {
            total_us: elapsed_us(total_started),
            word_us,
            semantic_us,
            word: word_timing,
        },
    }
}

fn proposal_repeats_declined_target(
    declined_target_surfaces: &[String],
    partial: &str,
    proposal: &ImeCandidateProposal,
) -> bool {
    declined_target_surfaces.iter().any(|target| {
        proposal
            .replacement
            .as_deref()
            .is_some_and(|replacement| replacement == target)
            || proposal.replacement.is_none()
                && target
                    .strip_prefix(partial)
                    .is_some_and(|suffix| suffix == proposal.suffix)
    })
}

fn elapsed_us(started: Option<Instant>) -> u64 {
    started
        .map(|started| started.elapsed().as_micros() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod preedit_readout_contract {
    #[test]
    fn preedit_rendering_does_not_own_l2_l3_material_acquisition() {
        let render = include_str!("preedit.rs");
        let readout = include_str!("preedit_readout.rs");
        let observed_field_call = concat!("TypingCpu::live_", "completion_readout(");
        let legacy_field_call = concat!("TypingCpu::live_", "completion_candidates(");

        assert!(
            !render.contains(legacy_field_call)
                && !readout.contains(legacy_field_call)
                && readout.matches(observed_field_call).count() == 1
                && !render.contains("semantic_phrase_candidates_for_input(")
                && !render.contains("word_candidate_proposals_for_input(")
                && readout.contains("pub(crate) fn materialize_precognition_candidates(")
                && readout.contains("llmwave_phrase_candidates_for_input("),
            "rendering must publish one asynchronously materialized readout"
        );
    }
}
