use std::time::Instant;
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::text::{make_ibus_text, make_preedit_ibus_text};
use super::trace;
use lay::russian_lexicon::is_known_russian_word_or_form;

const PREEDIT_TAIL_LIMIT: usize = 160;
const PREEDIT_TOKEN_LIMIT: usize = 32;
const PREEDIT_ASCII_CANDIDATE_LIMIT: usize = 12;
const PREEDIT_RU_WAVE_CANDIDATE_LIMIT: usize = 12;
const PREEDIT_RU_WAVE_SCAN_LIMIT: usize = 128;
const PREEDIT_RU_PREFIX_MIN_CHARS: usize = 2;
#[cfg(test)]
const PREEDIT_PROBE_SYMBOL: &str = "*";
const PREEDIT_MODE_CLEAR: u32 = 0;

#[derive(Debug, Default)]
pub(crate) struct PreeditFastState {
    token: String,
}

#[derive(Debug, Clone)]
struct RankedPreeditSuffix {
    suffix: String,
    score: f32,
    order: usize,
}

impl PreeditFastState {
    pub(crate) fn reset(&mut self) {
        self.token.clear();
    }

    pub(crate) fn push(&mut self, ch: char) {
        if ch.is_whitespace() || ch.is_ascii_punctuation() {
            self.reset();
            return;
        }
        self.token.push(ch);
        trim_tail_buffer_to(&mut self.token, PREEDIT_TOKEN_LIMIT);
    }

    pub(crate) fn backspace(&mut self) {
        self.token.pop();
    }

    #[cfg(test)]
    pub(crate) fn token(&self) -> &str {
        &self.token
    }

    fn ascii_suffixes(&self, max_suffix_chars: usize, limit: usize) -> Vec<String> {
        if self.token.chars().count() < 2 {
            return Vec::new();
        }
        if self.token.chars().all(|ch| ch.is_ascii_alphabetic()) {
            let mut suffixes = Vec::new();
            for suffix in lay::lexicon::common_en_technical_prefix_completions(
                &self.token,
                max_suffix_chars,
                limit,
            ) {
                push_unique_ascii_known_suffix(&mut suffixes, &self.token, suffix);
                if suffixes.len() >= limit {
                    break;
                }
            }
            if lay::nanda_wave::context_wave::prefix_wave_memory_is_warm() {
                for suffix in lay::nanda_wave::context_wave::en_word_prefix_completion_suffixes(
                    &self.token,
                    max_suffix_chars,
                    limit,
                ) {
                    push_unique_suffix(&mut suffixes, Some(suffix));
                    if suffixes.len() >= limit {
                        break;
                    }
                }
            }
            return suffixes;
        }
        Vec::new()
    }
}

impl LayIbusEngine {
    pub(super) async fn flush_dirty_preedit(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        if !self.preedit_dirty {
            return Ok(());
        }
        self.preedit_dirty = false;
        self.update_precognition_preedit(emitter).await
    }

    pub(super) async fn update_precognition_preedit(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        if !self.precognition_preedit_enabled() {
            return self.clear_preedit(emitter).await;
        }
        self.refresh_precognition_candidates();
        let Some(suffix) = self
            .preedit_candidates
            .get(self.preedit_candidate_index)
            .cloned()
        else {
            return self.clear_preedit(emitter).await;
        };
        self.preedit_suffix = suffix;
        let (preedit_text, cursor_pos) = self.inactive_preedit_payload();
        trace::record_preedit(
            "show",
            true,
            preedit_text.chars().count(),
            cursor_pos,
            Some(&preedit_text),
        );
        Self::show_preedit_text(emitter)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        Self::update_preedit_text(
            emitter,
            make_preedit_ibus_text(preedit_text),
            cursor_pos,
            true,
            PREEDIT_MODE_CLEAR,
        )
        .await
        .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    pub(crate) async fn clear_preedit(&mut self, emitter: &SignalEmitter<'_>) -> fdo::Result<()> {
        if !self.preedit_clear_needed() {
            return Ok(());
        }
        trace::record_preedit("clear", false, 0, 0, None);
        self.preedit_suffix.clear();
        self.preedit_candidates.clear();
        self.preedit_candidate_index = 0;
        Self::update_preedit_text(
            emitter,
            make_ibus_text(String::new()),
            0,
            false,
            PREEDIT_MODE_CLEAR,
        )
        .await
        .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        Self::hide_preedit_text(emitter)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    fn preedit_clear_needed(&self) -> bool {
        !self.buffer.is_empty()
            || !self.preedit_suffix.is_empty()
            || !self.preedit_candidates.is_empty()
    }

    pub(super) async fn update_composition_preedit(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        if self.buffer.is_empty() {
            return self.clear_preedit(emitter).await;
        }
        self.refresh_precognition_candidates();
        let (text, cursor_pos) = self.composition_preedit_payload();
        trace::record_preedit(
            "compose",
            true,
            text.chars().count(),
            cursor_pos,
            Some(&text),
        );
        Self::show_preedit_text(emitter)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        Self::update_preedit_text(
            emitter,
            make_preedit_ibus_text(text),
            cursor_pos,
            true,
            PREEDIT_MODE_CLEAR,
        )
        .await
        .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    pub(super) fn precognition_suffix(&self) -> Option<String> {
        self.precognition_suffix_candidates().into_iter().next()
    }

    fn inactive_preedit_payload(&self) -> (String, u32) {
        let suffix = if self.preedit_suffix == "*" {
            String::new()
        } else {
            self.preedit_suffix.clone()
        };
        (self.visible_precognition_suffix(suffix), 0)
    }

    fn composition_preedit_payload(&mut self) -> (String, u32) {
        let cursor_pos = self.composition_cursor.min(self.buffer.chars().count()) as u32;
        let suffix = if cursor_pos as usize == self.buffer.chars().count()
            && !self.composition_has_pending_autocorrect()
        {
            self.selected_visible_completion_suffix()
        } else {
            String::new()
        };
        self.preedit_suffix = suffix.clone();
        let mut text = self.buffer.clone();
        text.push_str(&self.visible_precognition_suffix(suffix));
        (text, cursor_pos)
    }

    fn visible_precognition_suffix(&self, suffix: String) -> String {
        if suffix.is_empty() || !self.config.ime_bracket_candidates {
            return suffix;
        }
        format!("[{suffix}]")
    }

    pub(super) fn selected_precognition_suffix(&self) -> Option<String> {
        self.preedit_candidates
            .get(self.preedit_candidate_index)
            .cloned()
            .or_else(|| self.precognition_suffix())
    }

    pub(super) fn refresh_precognition_candidates(&mut self) {
        let previous = self
            .preedit_candidates
            .get(self.preedit_candidate_index)
            .cloned();
        self.preedit_candidates = self.precognition_suffix_candidates();
        self.preedit_candidate_index = previous
            .as_ref()
            .and_then(|suffix| {
                self.preedit_candidates
                    .iter()
                    .position(|candidate| candidate == suffix)
            })
            .unwrap_or(0);
    }

    pub(super) fn cycle_precognition_candidate(&mut self, step: isize) -> bool {
        self.refresh_precognition_candidates();
        let len = self.preedit_candidates.len();
        if len < 2 {
            return false;
        }
        let len = len as isize;
        self.preedit_candidate_index =
            (self.preedit_candidate_index as isize + step).rem_euclid(len) as usize;
        true
    }

    fn precognition_suffix_candidates(&self) -> Vec<String> {
        if !self.precognition_preedit_enabled() {
            return Vec::new();
        }
        if self.composition_has_pending_autocorrect() {
            return Vec::new();
        }
        let timing_enabled = trace::enabled();
        let total_started = timing_enabled.then(Instant::now);
        let tail = self.tail_buffer.as_str();
        let partial_len = split_last_token(tail.trim_end())
            .map(|(_, token)| token.chars().count())
            .unwrap_or(0);
        let mut candidates = Vec::with_capacity(16);

        let semantic_started = timing_enabled.then(Instant::now);
        for suffix in self.semantic_phrase_suffixes() {
            push_unique_ranked_suffix(
                &mut candidates,
                Some(suffix.clone()),
                preedit_suffix_bayes_score(tail, &suffix, 0.72),
            );
        }
        let semantic_us = elapsed_us(semantic_started);

        let ru_started = timing_enabled.then(Instant::now);
        for suffix in self.ru_wave_lexical_suffixes() {
            push_unique_ranked_suffix(
                &mut candidates,
                Some(suffix.clone()),
                preedit_suffix_bayes_score(tail, &suffix, 0.48),
            );
        }
        let ru_us = elapsed_us(ru_started);

        let ascii_started = timing_enabled.then(Instant::now);
        for suffix in self.preedit_fast.ascii_suffixes(
            self.precognition_max_suffix_chars(),
            PREEDIT_ASCII_CANDIDATE_LIMIT,
        ) {
            push_unique_ranked_suffix(
                &mut candidates,
                Some(suffix.clone()),
                preedit_suffix_bayes_score(tail, &suffix, 0.80),
            );
        }
        let ascii_us = elapsed_us(ascii_started);

        candidates.sort_by(|left: &RankedPreeditSuffix, right: &RankedPreeditSuffix| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.order.cmp(&right.order))
                .then_with(|| {
                    compare_suffix_len_for_prefix(partial_len, &left.suffix, &right.suffix)
                })
                .then_with(|| left.suffix.cmp(&right.suffix))
        });
        let candidates = candidates
            .into_iter()
            .map(|candidate| candidate.suffix)
            .collect::<Vec<_>>();

        if let Some(started) = total_started {
            trace::record_precognition_timing(
                started.elapsed().as_micros() as u64,
                ascii_us,
                ru_us,
                semantic_us,
                candidates.len(),
            );
        }
        candidates
    }

    fn composition_has_pending_autocorrect(&self) -> bool {
        if self.buffer.is_empty() {
            return false;
        }
        let original = format!("{} ", self.buffer);
        self.autocorrect_active_composition_text(&original)
            .is_some_and(|replacement| replacement.trim_end() != self.buffer.trim_end())
    }

    fn precognition_max_suffix_chars(&self) -> usize {
        match self.config.active_correction_safety() {
            lay::config::CorrectionSafety::Strict => 3,
            lay::config::CorrectionSafety::Normal => 16,
            lay::config::CorrectionSafety::Experimental => 24,
        }
    }

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
        if should_query_llmwave_phrase_suffix(raw_tail)
            && lay::nanda_wave::llmwave::default_memory_is_warm()
        {
            suffixes.extend(self.llmwave_phrase_suffixes(raw_tail));
        }
        suffixes
    }

    fn ru_wave_lexical_suffixes(&self) -> Vec<String> {
        if self.config.active_correction_safety() == lay::config::CorrectionSafety::Strict {
            return Vec::new();
        }
        let tail = self.tail_buffer.as_str().trim_end();
        let Some((prefix, partial)) = split_last_token(tail) else {
            return Vec::new();
        };
        let partial = partial.to_lowercase();
        let partial_len = partial.chars().count();
        let has_left_context = prefix.split_whitespace().next().is_some();
        let min_prefix_chars = if has_left_context {
            self.ru_lexical_min_prefix_chars()
        } else if self.config.active_correction_safety()
            == lay::config::CorrectionSafety::Experimental
        {
            PREEDIT_RU_PREFIX_MIN_CHARS
        } else {
            self.ru_lexical_min_prefix_chars().max(4)
        };
        if !(min_prefix_chars..=12).contains(&partial_len)
            || !partial.chars().all(|ch| matches!(ch, 'а'..='я' | 'ё'))
            || (!has_left_context
                && self.config.active_correction_safety()
                    != lay::config::CorrectionSafety::Experimental)
            || is_noisy_first_russian_prefix(&partial)
            || is_known_russian_word_or_form(&partial)
        {
            return Vec::new();
        }

        let max_suffix_chars = self.precognition_max_suffix_chars();
        let mut suffixes = Vec::new();
        for word in lay::lexicon::common_ru_prefix_completion_words(
            &partial,
            max_suffix_chars,
            PREEDIT_RU_WAVE_SCAN_LIMIT,
        ) {
            push_unique_suffix(
                &mut suffixes,
                word.strip_prefix(&partial).map(str::to_string),
            );
            if suffixes.len() >= PREEDIT_RU_WAVE_CANDIDATE_LIMIT {
                break;
            }
        }

        if lay::nanda_wave::context_wave::prefix_wave_memory_is_warm() {
            let max_bucket_entries = match partial_len {
                0..=3 => 24,
                4 => 96,
                _ => 256,
            };
            for suffix in
                lay::nanda_wave::context_wave::ru_word_prefix_completion_suffixes_if_bucket_at_most(
                    &partial,
                    max_suffix_chars,
                    PREEDIT_RU_WAVE_SCAN_LIMIT,
                    max_bucket_entries,
                )
            {
                push_unique_suffix(&mut suffixes, Some(suffix));
                if suffixes.len() >= PREEDIT_RU_WAVE_CANDIDATE_LIMIT {
                    break;
                }
            }
        }

        let prefix_tokens = lay::nanda_wave::llmwave::tokenize(prefix);
        let allow_short_lexical =
            self.config.active_correction_safety() == lay::config::CorrectionSafety::Experimental;
        let mut ranked = suffixes
            .into_iter()
            .filter_map(|suffix| {
                let word = format!("{partial}{suffix}");
                let score = l3_or_lexical_precognition_score(
                    &prefix_tokens,
                    &word,
                    partial_len,
                    allow_short_lexical,
                )?;
                Some((suffix, score))
            })
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
                phrase_candidate_suffix(tail, &candidate.text, max_suffix_chars)
            })
            .collect()
    }

    fn precognition_preedit_enabled(&self) -> bool {
        self.config.active_nanda_precognition()
    }

    fn ru_lexical_min_prefix_chars(&self) -> usize {
        if self.config.ime_bracket_candidates {
            4
        } else {
            PREEDIT_RU_PREFIX_MIN_CHARS
        }
    }

    pub(super) fn push_tail_char(&mut self, ch: char) {
        self.tail_buffer.push(ch);
        self.preedit_fast.push(ch);
        self.last_tail_input_at = Some(Instant::now());
        if ch.is_whitespace() {
            self.preedit_dirty = false;
            self.word_input_mode = None;
            lay::nanda_wave::record_typed_tail_usage(&self.tail_buffer);
        }
        trim_tail_buffer(&mut self.tail_buffer);
        self.publish_tail_handoff();
    }

    #[cfg(test)]
    fn preedit_text_for_client(&self) -> (String, u32) {
        self.inactive_preedit_payload()
    }
}

fn elapsed_us(started: Option<Instant>) -> u64 {
    started
        .map(|started| started.elapsed().as_micros() as u64)
        .unwrap_or(0)
}

fn push_unique_suffix(candidates: &mut Vec<String>, suffix: Option<String>) {
    let Some(suffix) = suffix else {
        return;
    };
    if suffix.is_empty()
        || !is_allowed_visible_completion_suffix(&suffix)
        || candidates.iter().any(|candidate| candidate == &suffix)
    {
        return;
    }
    candidates.push(suffix);
}

fn push_unique_ranked_suffix(
    candidates: &mut Vec<RankedPreeditSuffix>,
    suffix: Option<String>,
    score: f32,
) {
    let Some(suffix) = suffix else {
        return;
    };
    if suffix.is_empty() || !is_allowed_visible_completion_suffix(&suffix) {
        return;
    }
    if let Some(existing) = candidates
        .iter_mut()
        .find(|candidate| candidate.suffix == suffix)
    {
        if score > existing.score {
            existing.score = score;
        }
        return;
    }
    let order = candidates.len();
    candidates.push(RankedPreeditSuffix {
        suffix,
        score,
        order,
    });
}

fn push_unique_ascii_known_suffix(candidates: &mut Vec<String>, token: &str, suffix: String) {
    if suffix.is_empty() || candidates.iter().any(|candidate| candidate == &suffix) {
        return;
    }
    let completed = format!("{token}{suffix}").to_ascii_lowercase();
    let one_ascii_char =
        suffix.chars().count() == 1 && suffix.chars().all(|ch| ch.is_ascii_alphabetic());
    if one_ascii_char && !lay::lexicon::is_common_en_technical_word(&completed) {
        return;
    }
    if one_ascii_char || is_allowed_visible_completion_suffix(&suffix) {
        candidates.push(suffix);
    }
}

fn is_allowed_visible_completion_suffix(suffix: &str) -> bool {
    let trimmed = suffix.trim();
    let mut chars = trimmed.chars();
    let Some(ch) = chars.next() else {
        return false;
    };
    if chars.next().is_some() {
        return true;
    }
    matches!(ch, 'а' | 'в' | 'и' | 'к' | 'о' | 'с' | 'у' | 'I' | 'a')
}

fn is_noisy_first_russian_prefix(prefix: &str) -> bool {
    matches!(prefix, "нев" | "инт")
}

fn compare_suffix_len_for_prefix(
    partial_len: usize,
    left: &str,
    right: &str,
) -> std::cmp::Ordering {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    if partial_len <= 3 {
        return right_len.cmp(&left_len);
    }
    left_len.cmp(&right_len)
}

fn preedit_suffix_bayes_score(tail: &str, suffix: &str, base: f32) -> f32 {
    let Some((context, word)) = preedit_suffix_context_and_word(tail, suffix) else {
        return base;
    };
    (base
        + lay::nanda_wave::cached_word_usage_prior(&word)
        + lay::nanda_wave::cached_context_word_usage_prior(&context, &word))
    .clamp(0.0, 1.0)
}

fn preedit_suffix_context_and_word(tail: &str, suffix: &str) -> Option<(Vec<String>, String)> {
    let tail = tail.trim_end();
    let suffix_starts_new_word = suffix.chars().next().is_some_and(char::is_whitespace);
    if suffix_starts_new_word || tail.is_empty() {
        let word = suffix.split_whitespace().next()?.to_lowercase();
        let context = lay::nanda_wave::llmwave::tokenize(tail);
        return Some((context, word));
    }

    let (prefix, partial) = split_last_token(tail)?;
    let suffix_word_part = suffix.split_whitespace().next().unwrap_or(suffix);
    let word = format!(
        "{}{}",
        partial.to_lowercase(),
        suffix_word_part.to_lowercase()
    );
    let context = lay::nanda_wave::llmwave::tokenize(prefix);
    Some((context, word))
}

fn l3_or_lexical_precognition_score(
    prefix_tokens: &[String],
    word: &str,
    partial_len: usize,
    allow_short_lexical: bool,
) -> Option<f32> {
    let min_lexical_prefix = if allow_short_lexical { 2 } else { 4 };
    let word_len = word.chars().count();
    let lexical_backoff_allowed = partial_len >= min_lexical_prefix
        && (partial_len >= 4 || word_len.saturating_sub(partial_len) <= 5);
    let usage_prior = lay::nanda_wave::cached_word_usage_prior(word);
    let context_usage_prior = lay::nanda_wave::cached_context_word_usage_prior(prefix_tokens, word);
    if lay::nanda_wave::llmwave::default_memory_is_warm() {
        return lay::nanda_wave::llmwave::with_default_memory(|memory| {
            if let Some(report) = memory.score_next_token_report(prefix_tokens, word) {
                return (report.score >= 0.18).then_some(
                    (0.62 + report.score * 0.34 + usage_prior + context_usage_prior)
                        .clamp(0.0, 1.0),
                );
            }
            lexical_backoff_allowed.then_some(
                (0.28 + partial_len as f32 * 0.035 + usage_prior + context_usage_prior)
                    .clamp(0.0, 0.70),
            )
        });
    }
    lexical_backoff_allowed.then_some(
        (0.28 + partial_len as f32 * 0.035 + usage_prior + context_usage_prior).clamp(0.0, 0.70),
    )
}

fn phrase_candidate_suffix(tail: &str, candidate: &str, max_suffix_chars: usize) -> Option<String> {
    let suffix = candidate.strip_prefix(tail)?;
    let suffix = if tail.ends_with(char::is_whitespace) {
        suffix.trim_start_matches(char::is_whitespace)
    } else {
        suffix
    };
    let suffix = next_word_suffix(suffix)?;
    (!suffix.is_empty() && suffix.chars().count() <= max_suffix_chars).then_some(suffix)
}

fn should_query_llmwave_phrase_suffix(tail: &str) -> bool {
    if tail.ends_with(char::is_whitespace) {
        return true;
    }
    let trimmed = tail.trim_end();
    let Some((left, token)) = trimmed.rsplit_once(char::is_whitespace) else {
        return false;
    };
    let token_chars = token.chars().count();
    (1..=6).contains(&token_chars)
        && left.split_whitespace().count() >= 1
        && token.chars().all(|ch| ch.is_alphabetic())
}

fn next_word_suffix(suffix: &str) -> Option<String> {
    let leading_space = suffix.chars().next().is_some_and(char::is_whitespace);
    let word = suffix.split_whitespace().next()?;
    if leading_space {
        Some(format!(" {word}"))
    } else {
        Some(word.to_string())
    }
}

fn split_last_token(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return None;
    }
    let end = trimmed
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| ch.is_alphabetic().then_some(idx + ch.len_utf8()))?;
    let start = trimmed[..end]
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (!ch.is_alphabetic()).then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    let (prefix, rest) = trimmed.split_at(start);
    let token = &rest[..end - start];
    (!token.is_empty()).then_some((prefix, token))
}

fn trim_tail_buffer(buffer: &mut String) {
    trim_tail_buffer_to(buffer, PREEDIT_TAIL_LIMIT);
}

fn trim_tail_buffer_to(buffer: &mut String, limit: usize) {
    let chars = buffer.chars().count();
    if chars <= limit {
        return;
    }
    let skip = chars - limit;
    let byte_idx = buffer
        .char_indices()
        .nth(skip)
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    buffer.drain(..byte_idx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    #[test]
    fn whitespace_cancels_pending_inactive_preedit_flush() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );

        engine.push_tail_char('п');
        engine.preedit_dirty = true;
        engine.push_tail_char(' ');

        assert!(
            !engine.preedit_dirty,
            "word boundary must not resurrect previous word suffix on cursor flush"
        );
        assert_eq!(engine.preedit_fast.token(), "");
    }

    #[test]
    fn bare_russian_prefixes_do_not_generate_precognition() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "нев".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert_eq!(engine.precognition_suffix(), None);
        assert!(
            engine.preedit_candidates.is_empty(),
            "raw Russian prefix memory leaked into preedit: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn russian_fast_lexical_prior_generates_contextual_suffix() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "я хочу пров".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            engine.preedit_candidates.iter().any(|suffix| {
                let word = format!("пров{suffix}");
                word.starts_with("провер") || word.starts_with("прове")
            }),
            "expected contextual Russian wave candidates for 'я хочу пров', got {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn ambiguous_short_russian_prefix_does_not_emit_dictionary_noise() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "я без за".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            engine
                .preedit_candidates
                .iter()
                .all(|suffix| !format!("за{suffix}").contains("запят")),
            "ambiguous prefix should not suggest project/chat noise: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn three_letter_russian_prefix_does_not_emit_long_lexical_tail_without_l3() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "интересно инт".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            engine
                .preedit_candidates
                .iter()
                .all(|suffix| suffix != "алия"),
            "short prefix must not leak long dictionary-only tails: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn bracketed_mode_suppresses_three_letter_russian_lexical_noise() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                ime_bracket_candidates: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "интересно инт".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            engine.preedit_candidates.is_empty(),
            "bracket mode must not show weak three-letter Russian guesses: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn known_russian_word_does_not_get_extended_by_precognition() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                ime_bracket_candidates: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "просто просто".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            engine.preedit_candidates.is_empty(),
            "known word must not be extended by weak suffixes: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn short_russian_prefix_stays_fast_without_dropping_valid_candidates() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "ка сло".chars() {
            engine.push_tail_char(ch);
        }

        let started = std::time::Instant::now();
        engine.refresh_precognition_candidates();
        let elapsed_us = started.elapsed().as_micros();

        assert!(
            elapsed_us < 5_000,
            "short prefix 'сло' must stay cheap, took {elapsed_us}us"
        );
        assert!(
            engine
                .preedit_candidates
                .iter()
                .all(|suffix| suffix.chars().count() != 1
                    || is_allowed_visible_completion_suffix(suffix)),
            "short prefix candidates must keep the single-letter guard: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn four_letter_russian_prefix_can_use_wave_lookup() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "ка слов".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            engine
                .preedit_candidates
                .iter()
                .all(|suffix| suffix.chars().count() != 1
                    || is_allowed_visible_completion_suffix(suffix)),
            "single-letter suffix guard must still apply: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn cold_english_wave_memory_does_not_block_precognition() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "this exam".chars() {
            engine.push_tail_char(ch);
        }
        let started = std::time::Instant::now();
        engine.refresh_precognition_candidates();
        let elapsed_us = started.elapsed().as_micros();

        assert!(
            elapsed_us < 5_000,
            "cold English wave memory must not block IME, took {elapsed_us}us"
        );
    }

    #[test]
    fn ascii_known_word_completion_allows_single_letter_suffix() {
        let mut fast = PreeditFastState::default();
        for ch in "exi".chars() {
            fast.push(ch);
        }

        let suffixes = fast.ascii_suffixes(16, 8);

        assert_eq!(suffixes.first().map(String::as_str), Some("t"));
        assert!(
            !suffixes.iter().any(|suffix| suffix == "il"),
            "known technical completion must outrank noisy wave suffixes: {suffixes:?}"
        );
    }

    #[test]
    fn long_russian_prefix_does_not_hold_inline_suffix() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "normal".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "следую".chars() {
            engine.push_tail_char(ch);
        }
        assert_eq!(engine.precognition_suffix(), None);
    }

    #[test]
    fn composition_preedit_does_not_complete_from_raw_russian_prefix() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "normal".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "пров".chars() {
            engine.insert_composition_char(ch);
        }
        engine.composition_cursor = engine.buffer.chars().count();
        engine.refresh_precognition_candidates();
        let (text, cursor_pos) = engine.composition_preedit_payload();

        assert_eq!(text, "пров");
        assert_eq!(cursor_pos, 4);
        assert_eq!(engine.preedit_suffix, "");
    }

    #[test]
    fn composition_preedit_suppresses_suffix_when_autocorrect_is_pending() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                auto_replace: true,
                typing_assist: true,
                nanda_precognition: true,
                correction_safety: "normal".to_string(),
                ..LayConfig::default()
            },
        );
        engine.buffer = "следущий".to_string();
        engine.composition_cursor = engine.buffer.chars().count();
        engine.preedit_suffix = "ий".to_string();
        let (text, cursor_pos) = engine.composition_preedit_payload();

        assert_eq!(text, "следущий");
        assert_eq!(cursor_pos, 8);
        assert!(engine.preedit_suffix.is_empty());
    }

    #[test]
    fn active_composition_requires_preedit_clear_even_without_suffix() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                ..LayConfig::default()
            },
        );
        engine.buffer = "следущий".to_string();
        engine.preedit_suffix.clear();
        engine.preedit_candidates.clear();

        assert!(engine.preedit_clear_needed());
    }

    #[test]
    fn pending_autocorrect_suppresses_completion_suffix() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                auto_replace: true,
                typing_assist: true,
                nanda_precognition: true,
                correction_safety: "normal".to_string(),
                ..LayConfig::default()
            },
        );
        engine.buffer = "следущий".to_string();
        engine.composition_cursor = engine.buffer.chars().count();
        assert!(engine.composition_has_pending_autocorrect());
        assert_eq!(engine.precognition_suffix(), None);
    }

    #[test]
    fn mid_sentence_short_prefix_does_not_suggest_greeting() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "normal".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "смотрим что будет происходить когда при".chars()
        {
            engine.push_tail_char(ch);
        }
        assert_eq!(engine.precognition_suffix().as_deref(), None);
    }

    #[test]
    fn experimental_short_russian_prefix_gets_lexical_candidates() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "смотрим что будет происходить когда при".chars()
        {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            !engine.preedit_candidates.is_empty(),
            "experimental L2 should not stay silent for contextual prefix 'при'"
        );
    }

    #[test]
    fn first_russian_word_prefix_gets_precognition_candidate() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "русс".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            engine
                .preedit_candidates
                .iter()
                .any(|suffix| suffix == "кий" || suffix == "ких"),
            "first Russian prefix should produce a useful word suffix: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn quoted_russian_prefix_gets_precognition_candidate() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "\"писа".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        assert!(
            engine.preedit_candidates.iter().any(|suffix| {
                let word = format!("писа{suffix}");
                word == "писать" || word.starts_with("писа")
            }),
            "punctuation before Russian prefix must not silence IME: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn first_active_russian_word_prefix_gets_precognition_candidate_after_four_chars() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "пров".chars() {
            engine.insert_composition_char(ch);
        }
        engine.composition_cursor = engine.buffer.chars().count();
        engine.refresh_precognition_candidates();

        assert!(
            engine.preedit_candidates.iter().any(|suffix| {
                let word = format!("пров{suffix}");
                word.starts_with("провер")
            }),
            "first active Russian word should produce a useful suffix after four chars: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn first_active_russian_word_prefix_gets_precognition_candidate_after_three_chars() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "при".chars() {
            engine.insert_composition_char(ch);
        }
        engine.composition_cursor = engine.buffer.chars().count();
        engine.refresh_precognition_candidates();

        assert!(
            !engine.preedit_candidates.is_empty(),
            "first active Russian word should produce suffixes after three chars"
        );
    }

    #[test]
    fn experimental_first_active_russian_word_prefix_gets_bayes_candidates_after_two_chars() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "пр".chars() {
            engine.insert_composition_char(ch);
        }
        engine.composition_cursor = engine.buffer.chars().count();
        engine.refresh_precognition_candidates();

        assert!(
            !engine.preedit_candidates.is_empty(),
            "experimental Bayes-backed IME should not stay silent after two Russian chars"
        );
    }

    #[test]
    fn short_russian_prefix_prefers_informative_suffix_over_tiny_tail() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "мало под".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();

        let first = engine
            .preedit_candidates
            .first()
            .map(String::as_str)
            .unwrap_or("");
        assert!(
            first.chars().count() > 2,
            "short Russian prefix should not rank tiny suffix first: {:?}",
            engine.preedit_candidates
        );
    }

    #[test]
    fn strict_precognition_keeps_short_suffix_limit() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "strict".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "пров".chars() {
            engine.push_tail_char(ch);
        }
        assert_eq!(engine.precognition_suffix().as_deref(), None);
    }

    #[test]
    fn experimental_precognition_can_use_phrase_wave() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "На улице опять идёт д".chars() {
            engine.push_tail_char(ch);
        }
        assert_eq!(engine.precognition_suffix().as_deref(), Some("ождь"));
    }

    #[test]
    fn experimental_precognition_can_suggest_next_word_from_l3_memory_after_space() {
        let engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        let memory = lay::nanda_wave::llmwave::LlmWaveMemory::from_text(
            "я хочу проверить подсказки\nя хочу проверить ввод",
        );

        let suffixes = engine.llmwave_phrase_suffixes_from_memory("я хочу ", &memory);

        assert!(
            suffixes.iter().any(|suffix| suffix == "проверить"),
            "expected next-word L2 suffix from L3 memory, got {:?}",
            suffixes
        );
    }

    #[test]
    fn experimental_precognition_keeps_l3_word_after_user_started_it() {
        let engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        let memory = lay::nanda_wave::llmwave::LlmWaveMemory::from_text("у нас мало слов");

        let suffixes = engine.llmwave_phrase_suffixes_from_memory("у нас мало с", &memory);

        assert!(
            suffixes.iter().any(|suffix| suffix == "лов"),
            "expected L3 suffix to survive started next word, got {:?}",
            suffixes
        );
    }

    #[test]
    fn experimental_precognition_uses_sentence_context_for_word_ending() {
        let engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        let memory = lay::nanda_wave::llmwave::LlmWaveMemory::from_text(
            "я хочу проверить подсказки\nя хочу проверить ввод",
        );

        let suffixes = engine.llmwave_phrase_suffixes_from_memory("я хочу пров", &memory);

        assert!(
            suffixes.iter().any(|suffix| suffix == "ерить"),
            "expected sentence-aware word ending, got {:?}",
            suffixes
        );
    }

    #[test]
    fn phrase_candidate_suffix_preserves_word_boundary_before_space() {
        assert_eq!(
            phrase_candidate_suffix("я хочу", "я хочу проверить", 24).as_deref(),
            Some(" проверить")
        );
        assert_eq!(
            phrase_candidate_suffix("я хочу ", "я хочу проверить", 24).as_deref(),
            Some("проверить")
        );
    }

    #[test]
    fn experimental_precognition_candidates_can_be_cycled() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                correction_safety: "experimental".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "На улице опять идёт д".chars() {
            engine.push_tail_char(ch);
        }
        engine.refresh_precognition_candidates();
        assert!(
            engine.preedit_candidates.len() >= 2,
            "expected NANDA phrase candidates, got {:?}",
            engine.preedit_candidates
        );
        assert_eq!(
            engine.selected_precognition_suffix().as_deref(),
            Some("ождь")
        );
        assert!(engine.cycle_precognition_candidate(1));
        assert_eq!(
            engine.selected_precognition_suffix().as_deref(),
            Some("ождик")
        );
        assert!(engine.cycle_precognition_candidate(-1));
        assert_eq!(
            engine.selected_precognition_suffix().as_deref(),
            Some("ождь")
        );
    }

    #[test]
    fn ime_backend_without_precognition_does_not_enable_probe_preedit() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: false,
                ..LayConfig::default()
            },
        );
        engine.tail_buffer = "ab".to_string();
        assert!(!engine.precognition_preedit_enabled());
        assert_eq!(engine.precognition_suffix(), None);
    }

    #[test]
    fn ime_backend_with_zero_nanda_weights_does_not_show_precognition() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                nanda_l2_weight_percent: 0,
                nanda_l3_weight_percent: 0,
                ..LayConfig::default()
            },
        );
        engine.tail_buffer = "пров".to_string();
        for ch in "пров".chars() {
            engine.preedit_fast.push(ch);
        }

        assert!(!engine.precognition_preedit_enabled());
        engine.refresh_precognition_candidates();
        assert!(engine.preedit_candidates.is_empty());
        assert_eq!(engine.precognition_suffix(), None);
    }

    #[test]
    fn preedit_for_plain_ime_client_hides_probe_marker() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.tail_buffer = "ab".to_string();
        engine.preedit_suffix = PREEDIT_PROBE_SYMBOL.to_string();

        assert_eq!(engine.preedit_text_for_client(), ("".to_string(), 0));
    }

    #[test]
    fn preedit_completion_has_no_visible_debug_marker() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.tail_buffer = "при".to_string();
        engine.preedit_suffix = "вет".to_string();
        assert_eq!(engine.preedit_text_for_client(), ("вет".to_string(), 0));
    }

    #[test]
    fn bracketed_precognition_is_display_only() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                nanda_precognition: true,
                ime_bracket_candidates: true,
                nanda_l2_weight_percent: 200,
                ..LayConfig::default()
            },
        );
        engine.buffer = "хоро".to_string();
        engine.composition_cursor = 4;
        engine.preedit_candidates = vec!["шо".to_string()];

        assert_eq!(
            engine.composition_preedit_payload(),
            ("хоро[шо]".to_string(), 4)
        );
        assert_eq!(engine.selected_visible_completion_suffix().as_str(), "шо");
    }

    #[test]
    fn preedit_candidates_suppress_noisy_single_letter_suffixes() {
        let mut candidates = Vec::new();

        push_unique_suffix(&mut candidates, Some("е".to_string()));
        push_unique_suffix(&mut candidates, Some("щ".to_string()));
        push_unique_suffix(&mut candidates, Some("и".to_string()));
        push_unique_suffix(&mut candidates, Some(" в".to_string()));
        push_unique_suffix(&mut candidates, Some("ет".to_string()));

        assert_eq!(
            candidates,
            vec!["и".to_string(), " в".to_string(), "ет".to_string()]
        );
    }

    #[test]
    fn preedit_completion_does_not_duplicate_anchor() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.tail_buffer = "проверк".to_string();
        engine.preedit_suffix = "а".to_string();

        assert_eq!(engine.preedit_text_for_client(), ("а".to_string(), 0));
    }

    #[test]
    fn precognition_candidate_generation_stays_under_budget() {
        let samples = [
            "пр",
            "пров",
            "file",
            "html d",
            "На улице опять идёт д",
            "смотрим что будет происходить когда при",
        ];
        let mut timings = Vec::new();
        let mut sample_max = Vec::new();
        for sample in samples {
            let mut engine = LayIbusEngine::new(
                "/test".to_string(),
                Arc::new(Mutex::new(Default::default())),
                true,
                true,
                LayConfig {
                    text_backend: "ime".to_string(),
                    nanda_precognition: true,
                    correction_safety: "experimental".to_string(),
                    ..LayConfig::default()
                },
            );
            for ch in sample.chars() {
                engine.push_tail_char(ch);
            }
            for _ in 0..20 {
                engine.refresh_precognition_candidates();
            }
            let sample_start = timings.len();
            for _ in 0..2000 {
                let started = Instant::now();
                engine.refresh_precognition_candidates();
                timings.push(started.elapsed().as_micros() as u64);
            }
            let mut local = timings[sample_start..].to_vec();
            local.sort_unstable();
            let local_p50 = percentile(&local, 50);
            let local_p90 = percentile(&local, 90);
            let local_p99 = percentile(&local, 99);
            let local_max = *local.last().unwrap_or(&0);
            eprintln!(
                "precognition sample {:?}: p50={}us p90={}us p99={}us max={}us candidates={} stages={:?}",
                sample,
                local_p50,
                local_p90,
                local_p99,
                local_max,
                engine.preedit_candidates.len(),
                measured_precognition_stages(&engine)
            );
            sample_max.push((sample, local_max));
        }
        timings.sort_unstable();
        let p50 = percentile(&timings, 50);
        let p90 = percentile(&timings, 90);
        let p99 = percentile(&timings, 99);
        let max = *timings.last().unwrap_or(&0);
        eprintln!(
            "precognition candidate generation: n={} p50={}us p90={}us p99={}us max={}us",
            timings.len(),
            p50,
            p90,
            p99,
            max
        );
        if let Some((sample, sample_max)) = sample_max.iter().max_by_key(|(_, max)| *max) {
            eprintln!(
                "precognition worst sample {:?}: max={}us",
                sample, sample_max
            );
        }
        let p90_budget_us = if cfg!(debug_assertions) {
            50_000
        } else {
            10_000
        };
        assert!(
            p90 <= p90_budget_us,
            "p90={p90}us exceeds budget {p90_budget_us}us"
        );
    }

    fn percentile(values: &[u64], percentile: usize) -> u64 {
        if values.is_empty() {
            return 0;
        }
        let idx = ((values.len() - 1) * percentile) / 100;
        values[idx]
    }

    fn measured_precognition_stages(engine: &LayIbusEngine) -> Vec<(&'static str, u128, usize)> {
        let semantic_started = Instant::now();
        let semantic = engine.semantic_phrase_suffixes();
        let semantic_us = semantic_started.elapsed().as_micros();

        let ru_started = Instant::now();
        let ru = engine.ru_wave_lexical_suffixes();
        let ru_us = ru_started.elapsed().as_micros();

        let ascii_started = Instant::now();
        let ascii = engine.preedit_fast.ascii_suffixes(
            engine.precognition_max_suffix_chars(),
            PREEDIT_ASCII_CANDIDATE_LIMIT,
        );
        let ascii_us = ascii_started.elapsed().as_micros();

        vec![
            ("semantic", semantic_us, semantic.len()),
            ("ru", ru_us, ru.len()),
            ("ascii", ascii_us, ascii.len()),
        ]
    }

    #[test]
    fn preedit_for_surrounding_text_client_hides_probe_marker() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        engine.surrounding_text_supported = true;
        engine.tail_buffer = "ab".to_string();
        engine.preedit_suffix = PREEDIT_PROBE_SYMBOL.to_string();

        assert_eq!(engine.preedit_text_for_client(), ("".to_string(), 0));
    }

    #[test]
    fn tail_buffer_stays_bounded() {
        let mut text = "x".repeat(PREEDIT_TAIL_LIMIT + 10);
        trim_tail_buffer(&mut text);
        assert_eq!(text.chars().count(), PREEDIT_TAIL_LIMIT);
    }
}
