use std::time::Instant;
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::text::make_ibus_text;
use super::trace;

pub(super) struct ActiveCompositionCommit {
    with_space: bool,
    suffix: String,
    sync_layout: bool,
}

impl ActiveCompositionCommit {
    pub(super) fn plain() -> Self {
        Self {
            with_space: false,
            suffix: String::new(),
            sync_layout: false,
        }
    }

    pub(super) fn with_space() -> Self {
        Self {
            with_space: true,
            suffix: String::new(),
            sync_layout: true,
        }
    }

    pub(super) fn with_completion(suffix: String, with_space: bool) -> Self {
        Self {
            with_space,
            suffix,
            sync_layout: false,
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
        )
        .await
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
        let text = if with_space {
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
        if !(self.config.auto_replace
            || self.config.typing_assist
            || self.config.auto_switch_layout)
        {
            return None;
        }
        let pipeline = lay::typing_context::typing_assist_pipeline_for_context(
            self.config.auto_replace,
            self.config.active_correction_safety(),
            &self.config.typing_assist_pipeline,
            text,
        );
        lay::typing_assist::apply_typing_assist_with_pipeline(
            text,
            self.config.auto_switch_layout,
            &pipeline,
        )
    }

    fn sync_tail_after_active_composition_commit(&mut self, text: &str) {
        self.sync_tail_after_composition_commit(text);
    }
}
