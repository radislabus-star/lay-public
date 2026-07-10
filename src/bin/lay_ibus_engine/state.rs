use lay::config::LayConfig;
use lay::text_edit::{
    decide_text_transition, LatentTextTransitionCandidate, TextTransitionDecision,
    TextTransitionIntent, VisibleFieldState, VisibleTailSnapshot, VisibleTailSource,
};
use std::time::{Duration, Instant};
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::engine::{LayIbusEngine, RecentCommittedTailReplace};
use super::protocol::Shared;
use super::text::make_ibus_text;
use super::trace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommittedTailOutputProfile {
    CommitOnly,
    KittyTerminalErase,
    WechatSurroundingText,
    GenericSurroundingText,
}

impl CommittedTailOutputProfile {
    fn select(cursor_cell_width: i32, surrounding_text_supported: bool, backspaces: u32) -> Self {
        if backspaces == 0 {
            return Self::CommitOnly;
        }
        if surrounding_text_supported {
            return Self::WechatSurroundingText;
        }
        if cursor_cell_width > 0 {
            return Self::KittyTerminalErase;
        }
        Self::GenericSurroundingText
    }

    fn output_route(self) -> &'static str {
        match self {
            Self::CommitOnly => "commit",
            Self::KittyTerminalErase => "kitty_terminal_erase_commit",
            Self::WechatSurroundingText => "wechat_surrounding_text_delete_commit",
            Self::GenericSurroundingText => "surrounding_text_delete_commit",
        }
    }

    fn uses_terminal_erase(self) -> bool {
        matches!(self, Self::KittyTerminalErase)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommittedTailReplaceRequest {
    pub(crate) source: VisibleTailSource,
    pub(crate) backspaces: u32,
    pub(crate) text: String,
    pub(crate) intent: TextTransitionIntent,
    pub(crate) suppress_next_autocorrect: bool,
    pub(crate) expected_tail: Option<VisibleTailSnapshot>,
}

impl CommittedTailReplaceRequest {
    pub(crate) fn ime_manual_toggle(
        backspaces: u32,
        text: String,
        suppress_next_autocorrect: bool,
    ) -> Self {
        Self {
            source: VisibleTailSource::ImeCommittedTail,
            backspaces,
            text,
            intent: TextTransitionIntent::ImeManualToggle,
            suppress_next_autocorrect,
            expected_tail: None,
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
            intent: TextTransitionIntent::DaemonBridge,
            suppress_next_autocorrect,
            expected_tail: None,
        }
    }

    pub(crate) fn with_expected_tail(mut self, expected_tail: VisibleTailSnapshot) -> Self {
        self.expected_tail = Some(expected_tail);
        self
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
            surrounding_text_snapshot: None,
            layout_is_ru,
            shift_active: false,
            shift_used_as_modifier: false,
            alt_completion_active: false,
            alt_used_as_modifier: false,
            last_shift_release_at: None,
            last_commit_at: None,
            last_tail_input_at: None,
            recent_committed_tail_replace: None,
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
        self.surrounding_text_snapshot = None;
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
        self.last_shift_release_at = None;
        self.recent_committed_tail_replace = None;
        self.shift_used_as_modifier = false;
        self.alt_completion_active = false;
        self.alt_used_as_modifier = false;
        self.surrounding_text_snapshot = None;
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
        let visible_state =
            VisibleFieldState::committed_tail(self.tail_buffer.clone(), Some(self.path.clone()))
                .with_external_tail_before_cursor(
                    self.surrounding_text_snapshot
                        .as_ref()
                        .and_then(|snapshot| snapshot.suffix_before_cursor(backspaces as usize)),
                    self.surrounding_text_snapshot
                        .as_ref()
                        .is_some_and(|snapshot| snapshot.has_selection()),
                );
        let transition_candidate = LatentTextTransitionCandidate::new(
            source,
            backspaces,
            request.text,
            request.intent,
            request.expected_tail,
        );
        let (plan, edit_action) = match decide_text_transition(&visible_state, transition_candidate)
        {
            TextTransitionDecision::Apply { plan, action } => (plan, action),
            TextTransitionDecision::Reject { rejection, action } => {
                if let Some(action) = action.as_ref() {
                    lay::action_log::record_candidate_edit_action_before_apply(
                        action,
                        lay::action_log::MutationLogRoute::IME_COMMITTED_TAIL,
                        None,
                    );
                    trace::record_committed_tail_replace(
                        source,
                        action.safety_reason(),
                        backspaces,
                        &action.to_text,
                    );
                } else if rejection.reason() != "noop_transition" {
                    trace::record_committed_tail_replace_guard(
                        source,
                        rejection.reason(),
                        backspaces,
                        rejection.expected(),
                        rejection.actual(),
                    );
                }
                return Ok(false);
            }
        };
        lay::action_log::record_candidate_edit_action_before_apply(
            &edit_action,
            lay::action_log::MutationLogRoute::IME_COMMITTED_TAIL,
            None,
        );
        let text = plan.insert.clone();
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
        let output_profile = CommittedTailOutputProfile::select(
            self.cursor_cell_width,
            self.surrounding_text_supported,
            backspaces,
        );
        let output_route = output_profile.output_route();
        trace::record_committed_tail_replace(source, output_route, backspaces, &text);
        let mut delete_us = 0;
        let commit_text = if output_profile.uses_terminal_erase() {
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
        self.clear_preedit_completion_state();
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
    lay::hot_field::set_process_policy(lay::hot_field::HotFieldPolicy::ime());
    #[cfg(test)]
    {
        lay::lem::set_runtime_enabled(config.lem_enabled && config.active_lem_weight() > 0.0);
    }
    #[cfg(not(test))]
    {
        lay::lem::set_runtime_enabled(config.lem_enabled && config.active_lem_weight() > 0.0);
        if config.active_text_backend().should_try_ime() {
            lay::nanda_wave::ensure_l2_ime_warmup_started();
        } else {
            lay::lexicon::warm_up();
        }
        if !config.active_text_backend().should_try_ime()
            && (config.auto_replace || config.typing_assist || config.auto_switch_layout)
        {
            lay::typing_assist::warm_up();
            lay::lem::warm_up();
            std::thread::spawn(lay::nanda_wave::warm_up);
        }
    }
}

fn terminal_erase_prefix(count: u32) -> String {
    "\u{7f}".repeat(count as usize)
}

#[cfg(test)]
mod tests {
    use super::{
        CommittedTailOutputProfile, CommittedTailReplaceRequest, LayIbusEngine,
        RecentCommittedTailReplace,
    };
    use lay::config::LayConfig;
    use lay::manual_toggle::VisibleTailSource;
    use lay::text_edit::TextTransitionIntent;
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
        assert_eq!(request.intent, TextTransitionIntent::DaemonBridge);
        assert!(request.suppress_next_autocorrect);
        assert!(request.expected_tail.is_none());
    }

    #[test]
    fn ime_manual_toggle_request_keeps_ime_committed_tail_source() {
        let request = CommittedTailReplaceRequest::ime_manual_toggle(4, "djn ".to_string(), true);

        assert_eq!(request.source, VisibleTailSource::ImeCommittedTail);
        assert_eq!(request.backspaces, 4);
        assert_eq!(request.text, "djn ");
        assert_eq!(request.intent, TextTransitionIntent::ImeManualToggle);
        assert!(request.suppress_next_autocorrect);
        assert!(request.expected_tail.is_none());
    }

    #[test]
    fn committed_tail_noop_requires_empty_delete_and_empty_insert() {
        assert!(CommittedTailReplaceRequest::ime_manual_toggle(0, String::new(), false).is_noop());
        assert!(!CommittedTailReplaceRequest::ime_manual_toggle(1, String::new(), false).is_noop());
        assert!(
            !CommittedTailReplaceRequest::ime_manual_toggle(0, "x".to_string(), false).is_noop()
        );
    }

    #[test]
    fn terminal_erase_prefix_is_delete_control_run_only() {
        assert_eq!(super::terminal_erase_prefix(3), "\u{7f}\u{7f}\u{7f}");
    }

    #[test]
    fn kitty_profile_keeps_existing_terminal_erase_route() {
        let profile = CommittedTailOutputProfile::select(9, false, 7);

        assert_eq!(profile, CommittedTailOutputProfile::KittyTerminalErase);
        assert_eq!(profile.output_route(), "kitty_terminal_erase_commit");
        assert!(profile.uses_terminal_erase());
    }

    #[test]
    fn wechat_profile_prefers_surrounding_text_over_terminal_erase() {
        let profile = CommittedTailOutputProfile::select(9, true, 7);

        assert_eq!(profile, CommittedTailOutputProfile::WechatSurroundingText);
        assert_eq!(
            profile.output_route(),
            "wechat_surrounding_text_delete_commit"
        );
        assert!(!profile.uses_terminal_erase());
    }

    #[test]
    fn plain_commits_do_not_select_delete_profile() {
        let profile = CommittedTailOutputProfile::select(9, true, 0);

        assert_eq!(profile, CommittedTailOutputProfile::CommitOnly);
        assert_eq!(profile.output_route(), "commit");
        assert!(!profile.uses_terminal_erase());
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

    #[test]
    fn committed_tail_replace_state_sync_clears_stale_preedit_suffix() {
        let mut engine = engine();
        engine.preedit_suffix = "а".to_string();
        engine.preedit_candidates = vec!["а".to_string()];
        engine.preedit_candidate_index = 0;
        engine.preedit_dirty = true;

        engine.clear_preedit_completion_state();

        assert!(engine.preedit_suffix.is_empty());
        assert!(engine.preedit_candidates.is_empty());
        assert_eq!(engine.preedit_candidate_index, 0);
        assert!(!engine.preedit_dirty);
    }
}
