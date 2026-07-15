// Candidate-source adapter for the IBus preedit surface. It may query shared
// L2/L3 memory, but does not rank candidates, mutate text, or emit IBus signals.

use lay::ime_candidate_readout::{
    is_noisy_first_russian_prefix, push_unique_ascii_known_suffix,
    should_query_llmwave_phrase_suffix,
};

impl PreeditFastState {
    fn ascii_suffixes(&self, max_suffix_chars: usize, limit: usize) -> Vec<String> {
        if self.token.chars().count() < 2
            || !self.token.chars().all(|ch| ch.is_ascii_alphabetic())
        {
            return Vec::new();
        }
        let mut suffixes = Vec::new();
        for candidate in lay::nanda_wave::candidate_gate::live_completion_candidates(
            lay::nanda_wave::candidate_gate::LiveCompletionRequest {
                context_prefix: "",
                partial: &self.token,
                max_suffix_chars,
                allow_short_lexical: true,
                limit,
            },
        ) {
            push_unique_ascii_known_suffix(&mut suffixes, &self.token, candidate.suffix);
            if suffixes.len() >= limit {
                break;
            }
        }
        suffixes
    }
}

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
        if !lay::nanda_wave::l2::ime_word_candidate_memory_is_warm() {
            lay::nanda_wave::ensure_l2_ime_warmup_started();
            return Vec::new();
        }
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
        // The shared candidate gate owns ranking. IBus only projects its
        // ordered whole-word readout into visible suffixes.
        whole_word_candidates
            .into_iter()
            .map(|candidate| candidate.suffix)
            .take(PREEDIT_RU_WAVE_CANDIDATE_LIMIT)
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
    lay::lexicon::is_common_ru_word(word)
        || (word.chars().count() >= 5
            && lay::russian_lexicon::is_known_russian_word_or_form(word))
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
                && readout.contains("llmwave_phrase_suffixes("),
            "preedit rendering must not own L2/L3 material acquisition"
        );
    }
}
