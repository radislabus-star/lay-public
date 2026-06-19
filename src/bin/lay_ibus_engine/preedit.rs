use lay::text_backend::TextBackendPreference;
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::text::{make_ibus_text, make_preedit_ibus_text};
use super::trace;

const PREEDIT_TAIL_LIMIT: usize = 160;
const PREEDIT_TOKEN_LIMIT: usize = 32;
const PREEDIT_PROBE_SYMBOL: &str = "*";
const PREEDIT_MODE_CLEAR: u32 = 0;

#[derive(Debug, Default)]
pub(crate) struct PreeditFastState {
    token: String,
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

    pub(crate) fn token(&self) -> &str {
        &self.token
    }

    fn ascii_suffix(&self, max_suffix_chars: usize) -> Option<String> {
        if self.token.chars().count() < 2 {
            return None;
        }
        if self.token.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return lay::lexicon::common_en_technical_prefix_completion(
                &self.token,
                max_suffix_chars,
            );
        }
        None
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
        let Some(suffix) = self.precognition_suffix() else {
            return self.clear_preedit(emitter).await;
        };
        self.preedit_suffix = suffix;
        let (preedit_text, cursor_pos) = ("".to_string(), 0);
        trace::record_preedit(
            "show",
            false,
            preedit_text.chars().count(),
            cursor_pos,
            Some(&preedit_text),
        );
        Self::update_preedit_text(
            emitter,
            make_preedit_ibus_text(preedit_text),
            cursor_pos,
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

    fn composition_preedit_payload(&mut self) -> (String, u32) {
        self.preedit_suffix.clear();
        let text = self.buffer.clone();
        let cursor_pos = self.composition_cursor.min(self.buffer.chars().count()) as u32;
        (text, cursor_pos)
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
        if self.composition_has_pending_autocorrect() {
            return Vec::new();
        }
        let mut candidates = Vec::new();
        push_unique_suffix(
            &mut candidates,
            self.preedit_fast
                .ascii_suffix(self.precognition_max_suffix_chars()),
        );
        push_unique_suffix(&mut candidates, self.lexical_ru_suffix());
        for suffix in self.semantic_phrase_suffixes() {
            push_unique_suffix(&mut candidates, Some(suffix));
        }
        push_unique_suffix(&mut candidates, self.probe_suffix_if_tail_is_ready());
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

    fn lexical_ru_suffix(&self) -> Option<String> {
        let token = self.preedit_fast.token();
        if token.chars().count() < 2 || token.chars().all(|ch| ch.is_ascii_alphabetic()) {
            return None;
        }
        if token.chars().count() > 4 {
            return None;
        }
        let word = lay::lexicon::common_ru_prefix_completion_word(
            token,
            self.precognition_max_suffix_chars(),
        )?;
        if self.should_veto_lexical_completion(token, &word) {
            return None;
        }
        word.get(token.to_lowercase().len()..).map(str::to_string)
    }

    fn should_veto_lexical_completion(&self, token: &str, word: &str) -> bool {
        let token_len = token.chars().count();
        let has_left_context = self
            .tail_buffer
            .trim_end()
            .strip_suffix(token)
            .is_some_and(|left| left.split_whitespace().next().is_some());
        token_len <= 3 && has_left_context && lay::lexicon::is_ru_greeting_word(word)
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
        let tail = self.tail_buffer.trim_end();
        if tail.chars().count() < 6 {
            return Vec::new();
        }
        lay::nanda_wave::context_wave::semantic_word_candidates(tail)
            .into_iter()
            .filter(|candidate| {
                candidate.source == lay::nanda_wave::context_wave::SEMANTIC_WORD_SOURCE
                    && candidate.energy - candidate.risk >= 0.58
            })
            .filter_map(|candidate| {
                let suffix = candidate.text.strip_prefix(tail)?;
                (!suffix.is_empty()
                    && suffix.chars().count() <= self.precognition_max_suffix_chars())
                .then(|| suffix.to_string())
            })
            .collect()
    }

    fn precognition_preedit_enabled(&self) -> bool {
        self.config.nanda_precognition
            || self.config.active_text_backend() == TextBackendPreference::Ime
    }

    fn probe_suffix_if_tail_is_ready(&self) -> Option<String> {
        let tail = self.tail_buffer.trim_end();
        if tail.chars().count() < 2 {
            return None;
        }
        Some(PREEDIT_PROBE_SYMBOL.to_string())
    }

    pub(super) fn push_tail_char(&mut self, ch: char) {
        self.tail_buffer.push(ch);
        self.preedit_fast.push(ch);
        trim_tail_buffer(&mut self.tail_buffer);
    }

    #[cfg(test)]
    fn preedit_text_for_client(&self) -> (String, u32) {
        let anchor = if self.preedit_suffix == PREEDIT_PROBE_SYMBOL {
            None
        } else {
            self.tail_buffer.chars().last()
        };
        let visible_suffix = if self.preedit_suffix == PREEDIT_PROBE_SYMBOL {
            ""
        } else {
            self.preedit_suffix.as_str()
        };
        let mut text =
            String::with_capacity(anchor.map(char::len_utf8).unwrap_or(0) + visible_suffix.len());
        if let Some(anchor) = anchor {
            text.push(anchor);
        }
        text.push_str(visible_suffix);
        let cursor_pos = if anchor.is_some() {
            1
        } else {
            text.chars().count() as u32
        };
        (text, cursor_pos)
    }
}

fn push_unique_suffix(candidates: &mut Vec<String>, suffix: Option<String>) {
    let Some(suffix) = suffix else {
        return;
    };
    if suffix.is_empty() || candidates.iter().any(|candidate| candidate == &suffix) {
        return;
    }
    candidates.push(suffix);
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
    fn precognition_suffix_uses_fast_prefix_completion_without_wave_trace() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                nanda_precognition: true,
                ..LayConfig::default()
            },
        );
        for ch in "прив".chars() {
            engine.push_tail_char(ch);
        }
        assert_eq!(engine.precognition_suffix().as_deref(), Some("ет"));
    }

    #[test]
    fn normal_precognition_can_complete_current_word() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                nanda_precognition: true,
                correction_safety: "normal".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "пров".chars() {
            engine.push_tail_char(ch);
        }
        assert_eq!(engine.precognition_suffix().as_deref(), Some("ерка"));
    }

    #[test]
    fn long_russian_prefix_does_not_hold_inline_suffix() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                nanda_precognition: true,
                correction_safety: "normal".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "следую".chars() {
            engine.push_tail_char(ch);
        }
        assert_eq!(
            engine.precognition_suffix().as_deref(),
            Some(PREEDIT_PROBE_SYMBOL)
        );
    }

    #[test]
    fn composition_preedit_keeps_candidate_out_of_visible_state() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                nanda_precognition: true,
                correction_safety: "normal".to_string(),
                ..LayConfig::default()
            },
        );
        engine.buffer = "пров".to_string();
        engine.composition_cursor = engine.buffer.chars().count();
        engine.preedit_suffix = "ерка".to_string();
        let (text, cursor_pos) = engine.composition_preedit_payload();

        assert_eq!(text, "пров");
        assert_eq!(cursor_pos, 4);
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
                nanda_precognition: true,
                correction_safety: "normal".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "смотрим что будет происходить когда при".chars()
        {
            engine.push_tail_char(ch);
        }
        assert_eq!(
            engine.precognition_suffix().as_deref(),
            Some(PREEDIT_PROBE_SYMBOL)
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
                nanda_precognition: true,
                correction_safety: "strict".to_string(),
                ..LayConfig::default()
            },
        );
        for ch in "пров".chars() {
            engine.push_tail_char(ch);
        }
        assert_eq!(
            engine.precognition_suffix().as_deref(),
            Some(PREEDIT_PROBE_SYMBOL)
        );
    }

    #[test]
    fn experimental_precognition_can_use_phrase_wave() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
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
    fn experimental_precognition_candidates_can_be_cycled() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
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
    fn ime_backend_enables_probe_preedit_without_separate_flag() {
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
        assert!(engine.precognition_preedit_enabled());
        assert_eq!(
            engine.precognition_suffix().as_deref(),
            Some(PREEDIT_PROBE_SYMBOL)
        );
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
        assert_eq!(engine.preedit_text_for_client(), ("ивет".to_string(), 1));
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

        assert_eq!(engine.preedit_text_for_client(), ("ка".to_string(), 1));
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
