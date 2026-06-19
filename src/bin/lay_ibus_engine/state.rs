use lay::config::LayConfig;
use std::time::{Duration, Instant};
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::LayIbusEngine;
use super::protocol::Shared;
use super::text::make_ibus_text;

impl LayIbusEngine {
    pub(crate) fn new(
        path: String,
        shared: Shared,
        layout_is_ru: bool,
        managed_input: bool,
        config: LayConfig,
    ) -> Self {
        warm_runtime(&config);
        Self {
            path,
            shared,
            buffer: String::new(),
            composition_cursor: 0,
            tail_buffer: String::new(),
            preedit_suffix: String::new(),
            preedit_candidates: Vec::new(),
            preedit_candidate_index: 0,
            preedit_fast: Default::default(),
            preedit_dirty: false,
            cursor_cell_width: 0,
            surrounding_text_supported: false,
            layout_is_ru,
            shift_active: false,
            shift_used_as_modifier: false,
            last_shift_release_at: None,
            last_commit_at: None,
            managed_input,
            config,
        }
    }

    pub(super) fn reset_for_ibus_focus_change(&mut self) {
        let preserve_tail = self
            .last_commit_at
            .is_some_and(|at| Instant::now().duration_since(at) <= Duration::from_millis(700));
        self.buffer.clear();
        self.composition_cursor = 0;
        self.preedit_suffix.clear();
        self.preedit_candidates.clear();
        self.preedit_candidate_index = 0;
        self.preedit_dirty = false;
        self.last_shift_release_at = None;
        self.shift_used_as_modifier = false;
        if !preserve_tail {
            self.tail_buffer.clear();
            self.preedit_fast.reset();
        }
    }

    /// Replaces text that has already been committed into the focused client.
    ///
    /// Keep this route separate from active composition commits in
    /// `managed.rs`: committed text can be deleted with surrounding-text APIs
    /// or terminal erase before `CommitText`, while live preedit composition
    /// has different client-side caret semantics.
    pub(crate) async fn replace_committed_tail(
        &mut self,
        emitter: &SignalEmitter<'_>,
        backspaces: u32,
        text: String,
    ) -> fdo::Result<bool> {
        if backspaces == 0 && text.is_empty() {
            return Ok(false);
        }
        self.clear_preedit(emitter).await?;
        let commit_text = if self.cursor_cell_width > 0 && backspaces > 0 {
            terminal_erase_prefix(backspaces) + &text
        } else {
            if backspaces > 0 {
                Self::delete_surrounding_text(emitter, -(backspaces as i32), backspaces)
                    .await
                    .map_err(|e| fdo::Error::Failed(e.to_string()))?;
            }
            text.clone()
        };
        if !commit_text.is_empty() {
            Self::commit_text(emitter, make_ibus_text(commit_text))
                .await
                .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        }
        for _ in 0..backspaces {
            self.tail_buffer.pop();
            self.preedit_fast.backspace();
        }
        self.tail_buffer.push_str(&text);
        self.preedit_fast.reset();
        for ch in text.chars() {
            self.preedit_fast.push(ch);
        }
        self.buffer.clear();
        self.composition_cursor = 0;
        self.preedit_candidates.clear();
        self.preedit_candidate_index = 0;
        self.last_commit_at = Some(Instant::now());
        Ok(true)
    }
}

fn warm_runtime(config: &LayConfig) {
    lay::lexicon::warm_up();
    if config.auto_replace || config.typing_assist || config.auto_switch_layout {
        lay::typing_assist::warm_up();
        lay::lem::warm_up();
    }
}

fn terminal_erase_prefix(count: u32) -> String {
    "\u{7f}".repeat(count as usize)
}
