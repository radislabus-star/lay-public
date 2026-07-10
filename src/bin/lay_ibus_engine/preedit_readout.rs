// Candidate-source adapter for the IBus preedit surface. It may query shared
// L2/L3 memory, but does not rank candidates, mutate text, or emit IBus signals.

use lay::ime_candidate_readout::{
    compare_suffix_len_for_prefix, is_noisy_first_russian_prefix, should_query_llmwave_phrase_suffix,
};

impl LayIbusEngine {
    fn semantic_phrase_suffixes(&self) -> Vec<String> {
        if self.config.active_correction_safety() != lay::config::CorrectionSafety::Experimental {
            return Vec::new();
        }
        let raw_tail = self.tail_buffer.as_str();
        let tail = raw_tail.trim_end();
        if tail.chars().count() < 6 {
            return Vec::new();
        }

        // Preedit can only show text after the cursor. Full-token typo repair
        // belongs to Space autocorrect; running it here burns latency and cannot
        // produce a right-side suffix for the current token.
        let mut suffixes = Vec::new();
        let context_wave_allowed = split_last_alphabetic_token(tail)
            .map(|(prefix, token)| {
                token.chars().count() >= 3 || prefix.split_whitespace().count() >= 3
            })
            .unwrap_or_else(|| tail.ends_with(char::is_whitespace));
        if context_wave_allowed {
            if let Some(wave) = lay::nanda_wave::context_wave::context_wave_for_tail(tail) {
                suffixes.extend(
                    lay::nanda_wave::context_wave::candidate_interferences(&wave)
                        .into_iter()
                        .take(5)
                        .filter(|candidate| candidate.projection >= 0.22)
                        .filter_map(|candidate| {
                            let text = format!("{}{}", wave.prefix, candidate.candidate);
                            let suffix = text.strip_prefix(tail)?;
                            (!suffix.is_empty()
                                && suffix.chars().count() <= self.precognition_max_suffix_chars())
                            .then(|| suffix.to_string())
                        }),
                );
            }
        }
        if should_query_llmwave_phrase_suffix(raw_tail)
            && lay::nanda_wave::llmwave::default_memory_is_warm()
        {
            suffixes.extend(self.llmwave_phrase_suffixes(raw_tail));
        }
        suffixes
    }

    fn ru_l2_word_attractor_suffixes(&self) -> Vec<String> {
        if self.config.active_correction_safety() == lay::config::CorrectionSafety::Strict {
            return Vec::new();
        }
        let tail = self.tail_buffer.as_str().trim_end();
        let Some((prefix, partial)) = split_last_alphabetic_token(tail) else {
            return Vec::new();
        };
        let partial = partial.to_lowercase();
        let partial_len = partial.chars().count();
        let has_left_context = prefix.split_whitespace().next().is_some();
        if has_left_context
            && lay::nanda_wave::llmwave::tokenize(prefix)
                .last()
                .is_some_and(|previous| previous == &partial)
        {
            return Vec::new();
        }
        let min_prefix_chars = PREEDIT_RU_PREFIX_MIN_CHARS;
        if !(min_prefix_chars..=12).contains(&partial_len)
            || !partial.chars().all(|ch| matches!(ch, 'а'..='я' | 'ё'))
            || is_noisy_first_russian_prefix(&partial)
            || is_complete_russian_word(&partial)
        {
            return Vec::new();
        }
        let max_suffix_chars = self.precognition_max_suffix_chars();
        let whole_word_candidates = lay::nanda_wave::candidate_gate::live_completion_candidates(
            lay::nanda_wave::candidate_gate::LiveCompletionRequest {
                context_prefix: prefix,
                partial: &partial,
                max_suffix_chars,
                // Candidate authority belongs to the shared L2/L3/L4 gate.
                // IME only renders its approved result, including in a phrase.
                allow_short_lexical: true,
                limit: PREEDIT_RU_WAVE_CANDIDATE_LIMIT * 2,
            },
        );
        let mut ranked = whole_word_candidates
            .into_iter()
            .map(|candidate| (candidate.suffix, candidate.score))
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            right
                .1
                .total_cmp(&left.1)
                .then_with(|| compare_suffix_len_for_prefix(partial_len, &left.0, &right.0))
                .then_with(|| left.0.cmp(&right.0))
        });
        ranked
            .into_iter()
            .take(PREEDIT_RU_WAVE_CANDIDATE_LIMIT)
            .map(|(suffix, _score)| suffix)
            .collect()
    }

    fn llmwave_phrase_suffixes(&self, tail: &str) -> Vec<String> {
        lay::nanda_wave::llmwave::with_default_memory(|memory| {
            self.llmwave_phrase_suffixes_from_memory(tail, memory)
        })
    }

    fn llmwave_phrase_suffixes_from_memory(
        &self,
        tail: &str,
        memory: &lay::nanda_wave::llmwave::LlmWaveMemory,
    ) -> Vec<String> {
        let max_suffix_chars = self.precognition_max_suffix_chars();
        lay::nanda_wave::llmwave::phrase_forecast_candidates(tail, memory)
            .into_iter()
            .take(6)
            .filter_map(|candidate| {
                lay::ime_candidate_readout::phrase_candidate_suffix(
                    tail,
                    &candidate.text,
                    max_suffix_chars,
                )
            })
            .collect()
    }
}

fn is_complete_russian_word(word: &str) -> bool {
    lay::russian_lexicon::is_known_russian_word_or_form(word)
        || lay::lexicon::is_common_ru_word(word)
}

#[cfg(test)]
mod preedit_readout_contract {
    #[test]
    fn preedit_rendering_does_not_own_l2_l3_material_acquisition() {
        let render = include_str!("preedit.rs");
        let readout = include_str!("preedit_readout.rs");

        assert!(
            !render.contains("context_wave_for_tail(")
                && !render.contains("live_completion_candidates(")
                && readout.contains("context_wave_for_tail(")
                && readout.contains("live_completion_candidates("),
            "preedit rendering must not own L2/L3 material acquisition"
        );
    }
}
