// Candidate-source adapter for the IBus preedit surface. It may query shared
// L2/L3 memory, but does not rank candidates, mutate text, or emit IBus signals.

use lay::typing_cpu::{
    push_unique_ascii_known_suffix, should_query_llmwave_phrase_suffix, ImeCandidateSource,
    LiveCompletionRequest, TypingCpu,
};

impl PreeditFastState {
    fn ascii_candidates(&self, max_suffix_chars: usize, limit: usize) -> Vec<ImeCandidateProposal> {
        if self.token.is_empty() || !self.token.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return Vec::new();
        }
        let mut suffixes = Vec::new();
        let mut proposals = Vec::new();
        for (order, candidate) in TypingCpu::live_completion_candidates(LiveCompletionRequest {
            context_prefix: "",
            partial: &self.token,
            max_suffix_chars,
            active_composition: true,
            allow_short_lexical: true,
            limit,
        })
        .into_iter()
        .enumerate()
        {
            let suffix = candidate.suffix;
            let before = suffixes.len();
            push_unique_ascii_known_suffix(&mut suffixes, &self.token, suffix.clone());
            if suffixes.len() > before {
                proposals.push(
                    ImeCandidateProposal::new(
                        suffix,
                        candidate.score,
                        ImeCandidateSource::L2Completion,
                    )
                    .with_authority_order(order),
                );
            }
            if proposals.len() >= limit {
                break;
            }
        }
        proposals
    }
}

impl LayIbusEngine {
    fn live_completion_input_is_active(&self) -> bool {
        !self.buffer.is_empty()
            || (self.last_tail_input_at.is_some() && !self.preedit_fast.token.is_empty())
    }

    fn semantic_phrase_candidates(&self) -> Vec<ImeCandidateProposal> {
        if self.config.active_correction_safety() != lay::config::CorrectionSafety::Experimental {
            return Vec::new();
        }
        let raw_tail = self.tail_buffer.as_str();
        let tail = raw_tail.trim_end();
        if tail.chars().count() < 6 {
            return Vec::new();
        }

        // Phrase memory is suffix-only. L2 replacement proposals travel through
        // the typed IME readout and remain display-only until explicit Tab.
        let mut suffixes = Vec::new();
        if should_query_llmwave_phrase_suffix(raw_tail) && TypingCpu::phrase_memory_is_warm() {
            suffixes.extend(self.llmwave_phrase_candidates(raw_tail));
        }
        suffixes
    }

    fn ru_l2_word_attractor_candidates(&self) -> Vec<ImeCandidateProposal> {
        if self.config.active_correction_safety() == lay::config::CorrectionSafety::Strict {
            return Vec::new();
        }
        let tail = self.tail_buffer.as_str().trim_end();
        if !TypingCpu::ime_candidate_memory_is_warm() {
            TypingCpu::ensure_ime_warmup_started();
            return Vec::new();
        }
        let Some((prefix, partial)) = split_last_alphabetic_token(tail) else {
            return Vec::new();
        };
        let partial = partial.to_lowercase();
        let partial_len = partial.chars().count();
        let min_prefix_chars = PREEDIT_RU_PREFIX_MIN_CHARS;
        if !(min_prefix_chars..=12).contains(&partial_len)
            || !partial.chars().all(|ch| matches!(ch, 'а'..='я' | 'ё'))
        {
            return Vec::new();
        }
        let max_suffix_chars = self.precognition_max_suffix_chars();
        let whole_word_candidates = TypingCpu::live_completion_candidates(LiveCompletionRequest {
            context_prefix: prefix,
            partial: &partial,
            max_suffix_chars,
            // Managed IME clients commit each typed grapheme immediately. The
            // token still remains an active input trajectory until its word
            // boundary, even though the traditional preedit buffer is empty.
            active_composition: self.live_completion_input_is_active(),
            // Candidate authority belongs to the shared L2/L3/L4 gate.
            // IME only renders its approved result, including in a phrase.
            allow_short_lexical: true,
            limit: PREEDIT_RU_WAVE_CANDIDATE_LIMIT * 2,
        });
        // The shared candidate gate owns ranking. IBus projects typed suffix or
        // full-token replacement proposals without gaining mutation authority.
        whole_word_candidates
            .into_iter()
            .enumerate()
            // A committed tail needs a distinct verified replacement route.
            // Never let an inactive preedit turn a whole-token candidate into
            // an append-only Tab action.
            .filter(|(_, candidate)| !self.buffer.is_empty() || !candidate.suffix.is_empty())
            .map(|(order, candidate)| {
                if candidate.suffix.is_empty() {
                    ImeCandidateProposal::replacement(
                        candidate.surface,
                        candidate.score,
                        ImeCandidateSource::L2Replacement,
                    )
                    .with_authority_order(order)
                } else {
                    ImeCandidateProposal::new(
                        candidate.suffix,
                        candidate.score,
                        ImeCandidateSource::L2Completion,
                    )
                    .with_authority_order(order)
                }
            })
            .take(PREEDIT_RU_WAVE_CANDIDATE_LIMIT)
            .collect()
    }

    fn llmwave_phrase_candidates(&self, tail: &str) -> Vec<ImeCandidateProposal> {
        let max_suffix_chars = self.precognition_max_suffix_chars();
        TypingCpu::phrase_forecast_candidates(tail)
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
                    candidate.score,
                    ImeCandidateSource::L3Context,
                ))
            })
            .collect()
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

#[cfg(test)]
mod preedit_readout_contract {
    #[test]
    fn preedit_rendering_does_not_own_l2_l3_material_acquisition() {
        let render = include_str!("preedit.rs");
        let readout = include_str!("preedit_readout.rs");

        assert!(
            !render.contains("live_completion_candidates(")
                && readout.contains("live_completion_candidates(")
                && readout.contains("llmwave_phrase_candidates("),
            "preedit rendering must not own L2/L3 material acquisition"
        );
    }
}
