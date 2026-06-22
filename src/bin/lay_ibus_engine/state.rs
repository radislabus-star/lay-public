use lay::config::LayConfig;
use lay::manual_toggle::VisibleTailSource;
use std::time::{Duration, Instant};
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::{LayIbusEngine, RecentCommittedTailReplace};
use super::protocol::Shared;
use super::text::make_ibus_text;
use super::trace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommittedTailReplaceRequest {
    pub(crate) source: VisibleTailSource,
    pub(crate) backspaces: u32,
    pub(crate) text: String,
    pub(crate) suppress_next_autocorrect: bool,
}

impl CommittedTailReplaceRequest {
    pub(crate) fn ime_autocorrect(backspaces: u32, text: String) -> Self {
        Self {
            source: VisibleTailSource::ImeCommittedTail,
            backspaces,
            text,
            suppress_next_autocorrect: false,
        }
    }

    pub(crate) fn ime_manual_toggle(
        backspaces: u32,
        text: String,
        suppress_next_autocorrect: bool,
    ) -> Self {
        Self {
            source: VisibleTailSource::ImeCommittedTail,
            backspaces,
            text,
            suppress_next_autocorrect,
        }
    }

    pub(crate) fn daemon_bridge(
        backspaces: u32,
        text: String,
        suppress_next_autocorrect: bool,
    ) -> Self {
        Self {
            source: VisibleTailSource::DaemonWordBuffer,
            backspaces,
            text,
            suppress_next_autocorrect,
        }
    }

    fn is_noop(&self) -> bool {
        self.backspaces == 0 && self.text.is_empty()
    }
}

impl LayIbusEngine {
    pub(crate) fn new(
        path: String,
        shared: Shared,
        layout_is_ru: bool,
        managed_input: bool,
        config: LayConfig,
    ) -> Self {
        warm_runtime(&config);
        let handoff_tail_buffer = shared
            .lock()
            .expect("lay ime state poisoned")
            .handoff_tail_buffer
            .clone();
        let mut engine = Self {
            path,
            shared,
            buffer: String::new(),
            composition_cursor: 0,
            tail_buffer: handoff_tail_buffer,
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
            alt_completion_active: false,
            alt_used_as_modifier: false,
            last_shift_release_at: None,
            last_commit_at: None,
            last_tail_input_at: None,
            recent_committed_tail_replace: None,
            pending_space_committed_tail_replace: None,
            suppress_next_committed_tail_autocorrect: false,
            word_input_mode: None,
            managed_input,
            config,
        };
        engine.rebuild_preedit_fast_from_tail();
        engine
    }

    pub(super) fn reset_for_ibus_focus_change(&mut self) {
        let preserve_tail =
            self.should_preserve_focus_handoff() || self.shared_active_path_preserved();
        self.buffer.clear();
        self.composition_cursor = 0;
        self.preedit_suffix.clear();
        self.preedit_candidates.clear();
        self.preedit_candidate_index = 0;
        self.preedit_dirty = false;
        self.pending_space_committed_tail_replace = None;
        self.last_shift_release_at = None;
        if !preserve_tail {
            self.last_tail_input_at = None;
            self.recent_committed_tail_replace = None;
            self.word_input_mode = None;
            self.suppress_next_committed_tail_autocorrect = false;
            self.clear_autocorrect_suppression_handoff();
        }
        self.shift_used_as_modifier = false;
        self.alt_completion_active = false;
        self.alt_used_as_modifier = false;
        if !preserve_tail {
            self.tail_buffer.clear();
            self.preedit_fast.reset();
            self.publish_tail_handoff();
        }
    }

    pub(super) fn should_preserve_focus_handoff(&self) -> bool {
        let now = Instant::now();
        self.last_commit_at
            .is_some_and(|at| now.duration_since(at) <= Duration::from_millis(700))
            || self
                .last_tail_input_at
                .is_some_and(|at| now.duration_since(at) <= Duration::from_millis(700))
    }

    pub(super) fn reset_for_ibus_soft_reset(&mut self) {
        self.buffer.clear();
        self.composition_cursor = 0;
        self.preedit_suffix.clear();
        self.preedit_candidates.clear();
        self.preedit_candidate_index = 0;
        self.preedit_dirty = false;
        self.pending_space_committed_tail_replace = None;
        self.last_shift_release_at = None;
        self.recent_committed_tail_replace = None;
        self.shift_used_as_modifier = false;
        self.alt_completion_active = false;
        self.alt_used_as_modifier = false;
        self.rebuild_preedit_fast_from_tail();
        self.publish_tail_handoff();
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
        request: CommittedTailReplaceRequest,
    ) -> fdo::Result<bool> {
        if request.is_noop() {
            return Ok(false);
        }
        let source = request.source;
        let backspaces = request.backspaces;
        let suppress_next_autocorrect = request.suppress_next_autocorrect;
        let text = request.text;
        let now = Instant::now();
        self.last_commit_at = Some(now);
        self.publish_active_path_preserve_handoff(now + Duration::from_millis(700));
        if suppress_next_autocorrect {
            self.suppress_next_committed_tail_autocorrect = true;
            self.publish_autocorrect_suppression_handoff();
        }
        if self.should_skip_duplicate_committed_tail_replace(backspaces, &text, now) {
            trace::record_committed_tail_replace(source, "duplicate_skip", backspaces, &text);
            return Ok(true);
        }
        let total_started = Instant::now();
        let clear_started = Instant::now();
        self.clear_preedit(emitter).await?;
        let clear_us = clear_started.elapsed().as_micros();
        let output_route = if self.cursor_cell_width > 0 && backspaces > 0 {
            "terminal_erase_commit"
        } else if backspaces > 0 {
            "surrounding_text_delete_commit"
        } else {
            "commit"
        };
        trace::record_committed_tail_replace(source, output_route, backspaces, &text);
        let mut delete_us = 0;
        let commit_text = if self.cursor_cell_width > 0 && backspaces > 0 {
            terminal_erase_prefix(backspaces) + &text
        } else {
            let delete_started = Instant::now();
            if backspaces > 0 {
                Self::delete_surrounding_text(emitter, -(backspaces as i32), backspaces)
                    .await
                    .map_err(|e| fdo::Error::Failed(e.to_string()))?;
            }
            delete_us = delete_started.elapsed().as_micros();
            text.clone()
        };
        let commit_started = Instant::now();
        if !commit_text.is_empty() {
            Self::commit_text(emitter, make_ibus_text(commit_text))
                .await
                .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        }
        let commit_us = commit_started.elapsed().as_micros();
        let state_started = Instant::now();
        for _ in 0..backspaces {
            self.tail_buffer.pop();
            self.preedit_fast.backspace();
        }
        self.tail_buffer.push_str(&text);
        self.preedit_fast.reset();
        for ch in text.chars() {
            self.preedit_fast.push(ch);
        }
        if text.chars().last().is_some_and(char::is_whitespace) {
            self.word_input_mode = None;
        }
        self.publish_tail_handoff();
        self.buffer.clear();
        self.composition_cursor = 0;
        self.preedit_candidates.clear();
        self.preedit_candidate_index = 0;
        self.sync_layout_after_committed_text(&text);
        let state_us = state_started.elapsed().as_micros();
        trace::record_committed_tail_replace_timing(
            source,
            output_route,
            clear_us,
            delete_us,
            commit_us,
            state_us,
            total_started.elapsed().as_micros(),
        );
        self.recent_committed_tail_replace = Some(RecentCommittedTailReplace {
            backspaces,
            text,
            at: now,
        });
        Ok(true)
    }

    fn should_skip_duplicate_committed_tail_replace(
        &self,
        backspaces: u32,
        text: &str,
        now: Instant,
    ) -> bool {
        const DUPLICATE_REPLACE_WINDOW: Duration = Duration::from_millis(900);
        self.recent_committed_tail_replace
            .as_ref()
            .is_some_and(|recent| {
                recent.backspaces == backspaces
                    && recent.text == text
                    && now.duration_since(recent.at) <= DUPLICATE_REPLACE_WINDOW
                    && self.tail_buffer.ends_with(text)
            })
    }
}

fn warm_runtime(config: &LayConfig) {
    lay::lem::set_runtime_enabled(config.lem_enabled && config.active_lem_weight() > 0.0);
    lay::lexicon::warm_up();
    if config.auto_replace || config.typing_assist || config.auto_switch_layout {
        lay::typing_assist::warm_up();
        lay::lem::warm_up();
        std::thread::spawn(lay::nanda_wave::warm_up);
    }
}

fn terminal_erase_prefix(count: u32) -> String {
    "\u{7f}".repeat(count as usize)
}

#[cfg(test)]
mod tests {
    use super::{CommittedTailReplaceRequest, LayIbusEngine, RecentCommittedTailReplace};
    use lay::config::LayConfig;
    use lay::manual_toggle::VisibleTailSource;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    fn engine() -> LayIbusEngine {
        LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        )
    }

    #[test]
    fn daemon_bridge_request_is_not_confused_with_ime_sources() {
        let request = CommittedTailReplaceRequest::daemon_bridge(6, "привет".to_string(), true);

        assert_eq!(request.source, VisibleTailSource::DaemonWordBuffer);
        assert_eq!(request.backspaces, 6);
        assert_eq!(request.text, "привет");
        assert!(request.suppress_next_autocorrect);
    }

    #[test]
    fn ime_manual_toggle_request_keeps_ime_committed_tail_source() {
        let request = CommittedTailReplaceRequest::ime_manual_toggle(4, "djn ".to_string(), true);

        assert_eq!(request.source, VisibleTailSource::ImeCommittedTail);
        assert_eq!(request.backspaces, 4);
        assert_eq!(request.text, "djn ");
        assert!(request.suppress_next_autocorrect);
    }

    #[test]
    fn committed_tail_noop_requires_empty_delete_and_empty_insert() {
        assert!(CommittedTailReplaceRequest::ime_autocorrect(0, String::new()).is_noop());
        assert!(!CommittedTailReplaceRequest::ime_autocorrect(1, String::new()).is_noop());
        assert!(!CommittedTailReplaceRequest::ime_autocorrect(0, "x".to_string()).is_noop());
    }

    #[test]
    fn terminal_erase_prefix_is_delete_control_run_only() {
        assert_eq!(super::terminal_erase_prefix(3), "\u{7f}\u{7f}\u{7f}");
    }

    #[test]
    fn duplicate_replace_gate_skips_same_recent_visible_result() {
        let now = Instant::now();
        let mut engine = engine();
        engine.tail_buffer = "ладно ".to_string();
        engine.recent_committed_tail_replace = Some(RecentCommittedTailReplace {
            backspaces: 6,
            text: "ладно ".to_string(),
            at: now,
        });

        assert!(engine.should_skip_duplicate_committed_tail_replace(6, "ладно ", now));
    }

    #[test]
    fn duplicate_replace_gate_allows_same_edit_for_new_original_tail() {
        let now = Instant::now();
        let mut engine = engine();
        engine.tail_buffer = "ладно kflyj ".to_string();
        engine.recent_committed_tail_replace = Some(RecentCommittedTailReplace {
            backspaces: 6,
            text: "ладно ".to_string(),
            at: now,
        });

        assert!(!engine.should_skip_duplicate_committed_tail_replace(6, "ладно ", now));
    }

    #[test]
    fn duplicate_replace_gate_expires_quickly() {
        let now = Instant::now();
        let mut engine = engine();
        engine.tail_buffer = "ладно ".to_string();
        engine.recent_committed_tail_replace = Some(RecentCommittedTailReplace {
            backspaces: 6,
            text: "ладно ".to_string(),
            at: now - Duration::from_millis(901),
        });

        assert!(!engine.should_skip_duplicate_committed_tail_replace(6, "ладно ", now));
    }
}
