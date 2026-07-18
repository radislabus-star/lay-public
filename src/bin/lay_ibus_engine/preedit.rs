use std::time::Instant;
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use lay::typing_cpu::{
    is_command_like_long_tail, preedit_suffix_context_and_word, select_ime_candidate_proposals,
    ImeCandidateProposal, ImeCandidateReadoutRequest,
};
use lay::word_reader::split_last_alphabetic_token;

#[cfg(test)]
use lay::typing_cpu::push_unique_suffix;
#[cfg(test)]
use lay::typing_cpu::{is_allowed_visible_completion_suffix, phrase_candidate_suffix};

use super::engine::LayIbusEngine;
use super::text::{make_ibus_text, make_preedit_ibus_text};
use super::trace;

const PREEDIT_TAIL_LIMIT: usize = 160;
const PREEDIT_TOKEN_LIMIT: usize = 32;
const PREEDIT_ASCII_CANDIDATE_LIMIT: usize = 12;
const PREEDIT_RU_WAVE_CANDIDATE_LIMIT: usize = 12;
const PREEDIT_RU_PREFIX_MIN_CHARS: usize = 1;
#[cfg(test)]
const PREEDIT_PROBE_SYMBOL: &str = "*";
const PREEDIT_MODE_CLEAR: u32 = 0;

include!("preedit_readout.rs");

#[derive(Debug, Default)]
pub(crate) struct PreeditFastState {
    token: String,
    target_surface: Option<String>,
    observed_prediction_target: Option<String>,
}

impl PreeditFastState {
    pub(crate) fn reset(&mut self) {
        self.token.clear();
        self.target_surface = None;
        self.observed_prediction_target = None;
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

    fn target_surface(&self) -> Option<&str> {
        self.target_surface.as_deref()
    }

    fn remember_target(&mut self, target: Option<String>) {
        self.target_surface = target;
    }

    /// Keep the first visible full-word prediction until the word boundary.
    /// Readout may change its suffix while the user keeps typing, but that
    /// initial target is the prediction whose outcome must be learned.
    fn observe_prediction_target(&mut self, target: Option<String>) {
        if self.observed_prediction_target.is_none() {
            self.observed_prediction_target = target.filter(|target| !target.is_empty());
        }
    }

    fn observed_prediction_target(&self) -> Option<&str> {
        self.observed_prediction_target.as_deref()
    }

    fn clear_target(&mut self) {
        self.target_surface = None;
    }

    pub(crate) fn clear_candidate_tracking(&mut self) {
        self.target_surface = None;
        self.observed_prediction_target = None;
    }

    #[cfg(test)]
    pub(crate) fn token(&self) -> &str {
        &self.token
    }
}

impl LayIbusEngine {
    pub(super) async fn refresh_precognition_after_visible_input(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        if self.preedit_waits_for_cursor_ack() {
            self.preedit_dirty = true;
            return Ok(());
        }
        self.preedit_dirty = false;
        self.update_precognition_preedit(emitter).await
    }

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
        self.preedit_replacement_targets.clear();
        self.preedit_candidate_index = 0;
        self.preedit_fast.clear_target();
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

    pub(super) fn close_precognition_word_boundary(&mut self) {
        self.clear_preedit_completion_state();
        self.preedit_fast.reset();
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
        if let Some(replacement) = self
            .selected_precognition_replacement()
            .map(ToOwned::to_owned)
        {
            self.preedit_suffix.clear();
            return (replacement.clone(), replacement.chars().count() as u32);
        }
        let cursor_pos = self.composition_cursor.min(self.buffer.chars().count()) as u32;
        let suffix = if cursor_pos as usize == self.buffer.chars().count() {
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

    pub(super) fn selected_precognition_replacement(&self) -> Option<&str> {
        self.preedit_replacement_targets
            .get(self.preedit_candidate_index)
            .and_then(|target| target.as_deref())
    }

    pub(super) fn refresh_precognition_candidates(&mut self) {
        let partial = split_last_alphabetic_token(self.tail_buffer.trim_end())
            .map(|(_, token)| token.to_lowercase())
            .unwrap_or_default();
        let proposals = self.precognition_candidates();
        self.preedit_replacement_targets = proposals
            .iter()
            .map(|proposal| proposal.replacement.clone())
            .collect();
        self.preedit_candidates = proposals
            .into_iter()
            .map(|proposal| proposal.display_text().to_string())
            .collect();
        self.preedit_candidate_index = stable_candidate_index(
            self.preedit_fast.target_surface(),
            &partial,
            &self.preedit_candidates,
        );
        self.remember_selected_target(&partial);
    }

    pub(super) fn cycle_precognition_candidate(&mut self, step: isize) -> bool {
        self.refresh_precognition_candidates();
        self.advance_precognition_candidate(step)
    }

    fn advance_precognition_candidate(&mut self, step: isize) -> bool {
        let len = self.preedit_candidates.len();
        if len < 2 {
            return false;
        }
        let len = len as isize;
        self.preedit_candidate_index =
            (self.preedit_candidate_index as isize + step).rem_euclid(len) as usize;
        let partial = split_last_alphabetic_token(self.tail_buffer.trim_end())
            .map(|(_, token)| token.to_lowercase())
            .unwrap_or_default();
        self.remember_selected_target(&partial);
        true
    }

    fn remember_selected_target(&mut self, partial: &str) {
        let target = self
            .selected_precognition_replacement()
            .map(ToOwned::to_owned)
            .or_else(|| {
                self.preedit_candidates
                    .get(self.preedit_candidate_index)
                    .map(|suffix| format!("{partial}{suffix}"))
            });
        self.preedit_fast.observe_prediction_target(target.clone());
        self.preedit_fast.remember_target(target);
    }

    fn precognition_candidates(&self) -> Vec<ImeCandidateProposal> {
        if !self.precognition_preedit_enabled() {
            return Vec::new();
        }
        if self.buffer.is_empty() && self.tail_buffer.ends_with(char::is_whitespace) {
            return Vec::new();
        }
        if self.buffer.is_empty()
            && self
                .tail_buffer
                .trim_end()
                .chars()
                .last()
                .is_some_and(is_hard_precognition_boundary)
        {
            return Vec::new();
        }
        let timing_enabled = trace::enabled();
        let total_started = timing_enabled.then(Instant::now);
        let tail = self.tail_buffer.as_str();
        if is_command_like_long_tail(tail.trim_end()) {
            return Vec::new();
        }
        let semantic_started = timing_enabled.then(Instant::now);
        let semantic_candidates = self.semantic_phrase_candidates();
        let semantic_us = elapsed_us(semantic_started);

        let ru_started = timing_enabled.then(Instant::now);
        let ru_l2_candidates = self.ru_l2_word_attractor_candidates();
        let ru_us = elapsed_us(ru_started);

        let ascii_started = timing_enabled.then(Instant::now);
        let ascii_candidates = self.preedit_fast.ascii_candidates(
            self.precognition_max_suffix_chars(),
            PREEDIT_ASCII_CANDIDATE_LIMIT,
        );
        let ascii_us = elapsed_us(ascii_started);

        let mut proposals = Vec::with_capacity(
            semantic_candidates.len() + ru_l2_candidates.len() + ascii_candidates.len(),
        );
        proposals.extend(semantic_candidates);
        proposals.extend(ru_l2_candidates);
        proposals.extend(ascii_candidates);
        let candidate_limit = proposals.len();
        let candidates = select_ime_candidate_proposals(ImeCandidateReadoutRequest {
            proposals: &proposals,
            limit: candidate_limit,
        });

        if let Some(started) = total_started {
            let token = split_last_alphabetic_token(tail.trim_end()).map(|(_, token)| token);
            trace::record_precognition_timing(
                started.elapsed().as_micros() as u64,
                ascii_us,
                ru_us,
                semantic_us,
                candidates.len(),
                token,
                candidates.first().map(|candidate| candidate.display_text()),
            );
        }
        candidates
    }

    fn precognition_suffix_candidates(&self) -> Vec<String> {
        self.precognition_candidates()
            .into_iter()
            .filter(|proposal| !proposal.is_replacement())
            .map(|proposal| proposal.suffix)
            .collect()
    }

    fn precognition_max_suffix_chars(&self) -> usize {
        match self.config.active_correction_safety() {
            lay::config::CorrectionSafety::Strict => 3,
            lay::config::CorrectionSafety::Normal => 16,
            lay::config::CorrectionSafety::Experimental => 24,
        }
    }

    fn precognition_preedit_enabled(&self) -> bool {
        self.config.active_nanda_precognition()
    }

    pub(super) fn push_tail_char(&mut self, ch: char) {
        let is_boundary = ch.is_whitespace() || is_hard_precognition_boundary(ch);
        let tail_before_boundary = is_boundary.then(|| self.tail_buffer.clone());
        // Whitespace resets the fast preedit state. Preserve the prediction
        // first so the boundary can turn it into supervised feedback.
        let prediction_before_boundary = is_boundary
            .then(|| {
                self.preedit_fast
                    .observed_prediction_target()
                    .map(str::to_owned)
            })
            .flatten();
        self.surrounding_text_snapshot = None;
        self.tail_buffer.push(ch);
        self.preedit_fast.push(ch);
        self.last_tail_input_at = Some(Instant::now());
        if is_boundary {
            if let Some(tail_before_boundary) = tail_before_boundary.as_deref() {
                self.record_ignored_precognition_at_boundary(
                    tail_before_boundary,
                    prediction_before_boundary.as_deref(),
                );
            }
            self.close_precognition_word_boundary();
            if ch.is_whitespace() {
                self.word_input_mode = None;
                lay::typing_cpu::TypingCpu::record_typed_tail(&self.tail_buffer);
            }
        }
        trim_tail_buffer(&mut self.tail_buffer);
        self.publish_tail_handoff();
    }

    fn record_ignored_precognition_at_boundary(
        &self,
        tail_before_boundary: &str,
        prediction_before_boundary: Option<&str>,
    ) {
        let Some((prefix, observed_word)) = split_last_alphabetic_token(tail_before_boundary)
        else {
            return;
        };
        let observed_word = observed_word.to_lowercase();
        let context = lay::nanda_wave::llmwave::tokenize(prefix);
        let predicted_word = prediction_before_boundary.map(str::to_owned).or_else(|| {
            let suffix = self.selected_visible_completion_suffix();
            preedit_suffix_context_and_word(tail_before_boundary, &suffix)
                .map(|(_, predicted)| predicted)
        });
        let Some(predicted_word) = predicted_word.filter(|word| !word.is_empty()) else {
            return;
        };
        if predicted_word == observed_word.to_lowercase() {
            lay::typing_cpu::TypingCpu::record_confirmed_completion_prediction(
                &context.join(" "),
                &predicted_word,
            );
            return;
        }
        lay::typing_cpu::TypingCpu::record_rejected_completion(&context.join(" "), &predicted_word);
    }

    #[cfg(test)]
    fn preedit_text_for_client(&self) -> (String, u32) {
        self.inactive_preedit_payload()
    }
}

fn candidate_index_for_target(
    previous_target: &str,
    partial: &str,
    candidates: &[String],
) -> Option<usize> {
    let expected_suffix = previous_target.strip_prefix(partial)?;
    candidates
        .iter()
        .position(|candidate| candidate == expected_suffix)
}

fn stable_candidate_index(
    previous_target: Option<&str>,
    partial: &str,
    candidates: &[String],
) -> usize {
    previous_target
        .and_then(|target| candidate_index_for_target(target, partial, candidates))
        .unwrap_or(0)
}

fn elapsed_us(started: Option<Instant>) -> u64 {
    started
        .map(|started| started.elapsed().as_micros() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
fn push_unique_ru_known_suffix(
    candidates: &mut Vec<String>,
    partial: &str,
    word: &str,
    suffix: Option<String>,
) {
    let Some(suffix) = suffix else {
        return;
    };
    if suffix.is_empty() || candidates.iter().any(|candidate| candidate == &suffix) {
        return;
    }
    let single_cyrillic =
        suffix.chars().count() == 1 && suffix.chars().all(|ch| matches!(ch, 'а'..='я' | 'ё'));
    let strong_single_letter_completion = single_cyrillic
        && partial.chars().count() >= 3
        && !is_ime_complete_russian_word(partial)
        && is_ime_candidate_russian_word(word);
    if strong_single_letter_completion || is_allowed_visible_completion_suffix(&suffix) {
        candidates.push(suffix);
    }
}

#[cfg(test)]
fn is_ime_complete_russian_word(word: &str) -> bool {
    lay::lexicon::is_common_ru_word(word)
        || (word.chars().count() >= 5 && lay::russian_lexicon::is_known_russian_word_or_form(word))
}

#[cfg(test)]
fn is_ime_candidate_russian_word(word: &str) -> bool {
    lay::lexicon::is_common_ru_word(word)
        || lay::russian_lexicon::is_known_russian_word_or_form(word)
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

fn is_hard_precognition_boundary(ch: char) -> bool {
    matches!(
        ch,
        '.' | ',' | '!' | '?' | ':' | ';' | ')' | ']' | '}' | '…'
    )
}

#[cfg(test)]
#[path = "preedit/tests.rs"]
mod tests;
