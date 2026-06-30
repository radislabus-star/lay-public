use std::time::Instant;
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::text::make_ibus_text;
use super::trace;
use lay::correction_core::{decide_text_correction, CorrectionMode, CorrectionRequest};

pub(super) struct ActiveCompositionCommit {
    with_space: bool,
    suffix: String,
    sync_layout: bool,
    autocorrect: bool,
}

impl ActiveCompositionCommit {
    pub(super) fn plain() -> Self {
        Self {
            with_space: false,
            suffix: String::new(),
            sync_layout: false,
            autocorrect: false,
        }
    }

    pub(super) fn with_space() -> Self {
        Self {
            with_space: true,
            suffix: String::new(),
            sync_layout: true,
            autocorrect: true,
        }
    }

    pub(super) fn with_completion(suffix: String, with_space: bool) -> Self {
        Self {
            with_space,
            suffix,
            sync_layout: false,
            autocorrect: false,
        }
    }
}

impl LayIbusEngine {
    pub(super) async fn commit_active_composition(
        &mut self,
        emitter: &SignalEmitter<'_>,
        request: ActiveCompositionCommit,
    ) -> fdo::Result<()> {
        self.commit_active_composition_with_suffix(
            emitter,
            request.with_space,
            &request.suffix,
            request.sync_layout,
            request.autocorrect,
        )
        .await
    }

    pub(super) async fn accept_completion(
        &mut self,
        emitter: &SignalEmitter<'_>,
        with_space: bool,
    ) -> fdo::Result<bool> {
        if self.buffer.is_empty() {
            if self.accept_stuck_tail(emitter, with_space).await? {
                return Ok(true);
            }
            return Ok(false);
        }

        let suffix = self.selected_visible_completion_suffix();
        if suffix.is_empty() {
            return Ok(false);
        }

        let accepted_word = format!("{}{}", self.buffer, suffix);
        let context_tail = self.tail_buffer.clone();
        trace::record_completion_accept("active_composition", suffix.chars().count(), with_space);
        self.commit_active_composition(
            emitter,
            ActiveCompositionCommit::with_completion(suffix, with_space),
        )
        .await?;
        lay::nanda_wave::record_accepted_ime_usage(&context_tail, &accepted_word);
        Ok(true)
    }

    pub(super) async fn accept_completion_with_space(
        &mut self,
        emitter: &SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        let handled = self.accept_completion(emitter, true).await?;
        self.trace_key("alt_accept", 0, 0, handled, None);
        Ok(handled)
    }

    pub(super) async fn commit_managed_passthrough_char(
        &mut self,
        emitter: &SignalEmitter<'_>,
        ch: char,
    ) -> fdo::Result<()> {
        Self::commit_text(emitter, make_ibus_text(ch.to_string()))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        self.last_commit_at = Some(Instant::now());
        self.push_tail_char(ch);
        self.preedit_dirty = false;
        self.update_precognition_preedit(emitter).await
    }

    pub(super) async fn observe_terminal_passthrough_char(
        &mut self,
        emitter: &SignalEmitter<'_>,
        ch: char,
    ) -> fdo::Result<()> {
        self.push_tail_char(ch);
        self.update_precognition_preedit(emitter).await
    }

    /// Finalizes the currently active IME preedit composition.
    ///
    /// This is intentionally separate from `replace_committed_tail()`: this
    /// path commits text that the client still treats as live preedit, while
    /// committed-tail replacement edits text that is already in the widget.
    async fn commit_active_composition_with_suffix(
        &mut self,
        emitter: &SignalEmitter<'_>,
        with_space: bool,
        suffix: &str,
        sync_layout: bool,
        autocorrect: bool,
    ) -> fdo::Result<()> {
        let started_at = Instant::now();
        let clear_started_at = Instant::now();
        self.clear_preedit(emitter).await?;
        let clear_us = clear_started_at.elapsed().as_micros() as u64;
        let mut text = std::mem::take(&mut self.buffer);
        self.composition_cursor = 0;
        text.push_str(suffix);
        if with_space {
            text.push(' ');
        }
        let decision_started_at = Instant::now();
        let text = if autocorrect {
            self.autocorrect_active_composition_text(&text)
                .unwrap_or(text)
        } else {
            text
        };
        let decision_ms = decision_started_at.elapsed().as_micros() as u64;
        let output_started_at = Instant::now();
        Self::commit_text(emitter, make_ibus_text(text.clone()))
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        let output_ms = output_started_at.elapsed().as_micros() as u64;
        if sync_layout {
            self.sync_layout_after_committed_text(&text);
        }
        self.sync_tail_after_active_composition_commit(&text);
        self.last_commit_at = Some(Instant::now());
        trace::record_ime_commit(
            decision_ms,
            clear_us,
            output_ms,
            started_at.elapsed().as_micros() as u64,
        );
        Ok(())
    }

    pub(super) fn autocorrect_active_composition_text(&self, text: &str) -> Option<String> {
        self.deterministic_autocorrect_text(text)
            .or_else(|| self.nanda_autocorrect_text(text))
    }

    #[cfg(test)]
    pub(super) fn autocorrect_committed_tail_text(&self, text: &str) -> Option<String> {
        self.deterministic_autocorrect_text(text)
            .or_else(|| self.nanda_committed_tail_context_replacement(text))
            .or_else(|| self.nanda_autocorrect_text(text))
    }

    fn deterministic_autocorrect_text(&self, text: &str) -> Option<String> {
        decide_text_correction(self.correction_request(text, CorrectionMode::DeterministicOnly))
            .map(|decision| decision.replacement)
    }

    #[cfg(test)]
    fn nanda_committed_tail_context_replacement(&self, text: &str) -> Option<String> {
        if !self.config.nanda_autocorrect {
            return None;
        }
        let context = format!("{} ", self.tail_buffer.trim_end());
        let prefix = context.strip_suffix(text)?;
        if prefix.is_empty() {
            return None;
        }
        let output = self.nanda_autocorrect_text(&context)?;
        (output != context && output.starts_with(prefix))
            .then(|| output[prefix.len()..].to_string())
    }

    fn nanda_autocorrect_text(&self, text: &str) -> Option<String> {
        decide_text_correction(self.correction_request(text, CorrectionMode::NandaOnly))
            .map(|decision| decision.replacement)
    }

    fn correction_request<'a>(
        &'a self,
        text: &'a str,
        mode: CorrectionMode,
    ) -> CorrectionRequest<'a> {
        CorrectionRequest {
            text,
            auto_replace: self.config.auto_replace,
            typing_assist: self.config.typing_assist,
            auto_switch_layout: self.config.auto_switch_layout,
            correction_safety: self.config.active_correction_safety(),
            typing_assist_pipeline: &self.config.typing_assist_pipeline,
            nanda_autocorrect: self.config.nanda_autocorrect,
            mode,
        }
    }

    fn sync_tail_after_active_composition_commit(&mut self, text: &str) {
        self.sync_tail_after_composition_commit(text);
    }
}

#[cfg(test)]
mod tests {
    use super::LayIbusEngine;
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    fn engine() -> LayIbusEngine {
        LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                auto_replace: true,
                typing_assist: true,
                auto_switch_layout: true,
                correction_safety: "experimental".to_string(),
                nanda_autocorrect: true,
                nanda_precognition: true,
                ..LayConfig::default()
            },
        )
    }

    #[test]
    fn active_composition_autocorrect_can_use_nanda_fallback() {
        let engine = engine();
        assert_eq!(
            engine
                .autocorrect_active_composition_text("тфтвф ")
                .as_deref(),
            Some("nanda ")
        );
    }

    #[test]
    fn completion_with_space_does_not_trigger_autocorrect() {
        let completion = super::ActiveCompositionCommit::with_completion("ерка".to_string(), true);
        assert!(completion.with_space);
        assert_eq!(completion.suffix, "ерка");
        assert!(!completion.autocorrect);

        let real_space = super::ActiveCompositionCommit::with_space();
        assert!(real_space.with_space);
        assert!(real_space.autocorrect);
    }

    #[test]
    fn committed_tail_autocorrect_can_use_tail_context_for_nanda() {
        let mut engine = engine();
        for ch in "file ghjdthrf".chars() {
            engine.push_tail_char(ch);
        }
        assert_eq!(
            engine
                .autocorrect_committed_tail_text("ghjdthrf ")
                .as_deref(),
            Some("проверка ")
        );
    }

    #[test]
    fn committed_tail_autocorrect_handles_ascii_tail_after_russian_context() {
        let mut engine = engine();
        for ch in "проверка ghjdthrf".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine
                .autocorrect_committed_tail_text("ghjdthrf ")
                .as_deref(),
            Some("проверка ")
        );
    }

    #[test]
    fn committed_tail_autocorrect_handles_autozamena_layout_word() {
        let mut engine = engine();
        for ch in "fdnjpfvtyf".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine
                .autocorrect_committed_tail_text("fdnjpfvtyf ")
                .as_deref(),
            Some("автозамена ")
        );
    }

    #[test]
    fn committed_tail_autocorrect_repairs_layout_word_with_missing_initial_letter() {
        let mut engine = engine();
        for ch in "dnjpfvtyf".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine
                .autocorrect_committed_tail_text("dnjpfvtyf ")
                .as_deref(),
            Some("автозамена ")
        );
    }

    #[test]
    fn committed_tail_autocorrect_repairs_autozamena_mixed_prefix() {
        let mut engine = engine();
        for ch in "fвтозамена".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine
                .autocorrect_committed_tail_text("fвтозамена ")
                .as_deref(),
            Some("автозамена ")
        );
    }

    #[test]
    fn committed_tail_autocorrect_repairs_duplicate_latin_prefix_before_russian_word() {
        let mut engine = engine();
        for ch in "fавтозамена".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine
                .autocorrect_committed_tail_text("fавтозамена ")
                .as_deref(),
            Some("автозамена ")
        );
    }

    #[test]
    fn committed_tail_autocorrect_handles_plain_en_to_ru_layout_words() {
        let mut engine = engine();
        for ch in "ghbdtn".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine.autocorrect_committed_tail_text("ghbdtn ").as_deref(),
            Some("привет ")
        );
    }

    #[test]
    fn committed_tail_autocorrect_keeps_ascii_layout_punctuation_in_token() {
        let mut engine = engine();
        for ch in "ghj,ktvf".chars() {
            engine.push_tail_char(ch);
        }

        assert_eq!(
            engine
                .autocorrect_committed_tail_text("ghj,ktvf ")
                .as_deref(),
            Some("проблема ")
        );
    }
}
