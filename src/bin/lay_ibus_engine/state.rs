use super::output::EngineOutput;
use lay::config::LayConfig;
use lay::text_edit::{
    decide_text_transition, EditAction, LatentTextTransitionCandidate, TextEditBackend,
    TextTransitionDecision, TextTransitionIntent, VisibleFieldState, VisibleTailSnapshot,
    VisibleTailSource,
};
use std::time::{Duration, Instant};
use zbus::fdo;

use super::engine::{
    LayIbusEngine, PendingSystemOutcomeFeedback, RecentCommittedTailReplace,
    SurroundingTextSnapshot,
};
use super::protocol::{Shared, KEY_LEFT, KEY_RIGHT, RELEASE_MASK};
use super::text::make_ibus_text;
use super::trace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommittedTailOutputProfile {
    CommitOnly,
    TerminalErase,
    SurroundingText,
    Unavailable,
}

impl CommittedTailOutputProfile {
    fn select(cursor_cell_width: i32, surrounding_text_supported: bool, backspaces: u32) -> Self {
        if backspaces == 0 {
            return Self::CommitOnly;
        }
        if surrounding_text_supported {
            return Self::SurroundingText;
        }
        if cursor_cell_width > 0 {
            return Self::TerminalErase;
        }
        Self::Unavailable
    }

    fn output_route(self) -> &'static str {
        match self {
            Self::CommitOnly => "commit",
            Self::TerminalErase => "terminal_erase_commit",
            Self::SurroundingText => "surrounding_text_delete_commit",
            Self::Unavailable => "no_proven_delete_backend",
        }
    }

    fn uses_terminal_erase(self) -> bool {
        matches!(self, Self::TerminalErase)
    }

    fn can_execute(self) -> bool {
        !matches!(self, Self::Unavailable)
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
    boundary_elided_external_snapshot: bool,
    causal_precondition_external_snapshot: Option<String>,
    /// Authority selected before this adapter boundary. When present, the
    /// committed-tail backend must preserve this exact action after structural
    /// verification instead of reconstructing a replacement from strings.
    winner_action: Option<EditAction>,
    outcome_feedback: Option<PendingSystemOutcomeFeedback>,
    layout_postcondition_owner: LayoutPostconditionOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutPostconditionOwner {
    Caller,
    Ime,
}

impl CommittedTailReplaceRequest {
    pub(crate) fn ime_autocorrect(backspaces: u32, text: String) -> Self {
        Self {
            source: VisibleTailSource::ImeCommittedTail,
            backspaces,
            text,
            intent: TextTransitionIntent::ImeAutocorrect,
            suppress_next_autocorrect: false,
            expected_tail: None,
            boundary_elided_external_snapshot: false,
            causal_precondition_external_snapshot: None,
            winner_action: None,
            outcome_feedback: None,
            layout_postcondition_owner: LayoutPostconditionOwner::Ime,
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
            intent: TextTransitionIntent::ImeManualToggle,
            suppress_next_autocorrect,
            expected_tail: None,
            boundary_elided_external_snapshot: false,
            causal_precondition_external_snapshot: None,
            winner_action: None,
            outcome_feedback: None,
            layout_postcondition_owner: LayoutPostconditionOwner::Ime,
        }
    }

    pub(crate) fn ime_auto_undo(backspaces: u32, text: String) -> Self {
        Self {
            source: VisibleTailSource::ImeCommittedTail,
            backspaces,
            text,
            intent: TextTransitionIntent::ImeAutoUndo,
            suppress_next_autocorrect: true,
            expected_tail: None,
            boundary_elided_external_snapshot: false,
            causal_precondition_external_snapshot: None,
            winner_action: None,
            outcome_feedback: None,
            layout_postcondition_owner: LayoutPostconditionOwner::Ime,
        }
    }

    pub(crate) fn ime_candidate_accept(backspaces: u32, text: String) -> Self {
        Self {
            source: VisibleTailSource::ImeCommittedTail,
            backspaces,
            text,
            intent: TextTransitionIntent::ImeCandidateAccept,
            suppress_next_autocorrect: true,
            expected_tail: None,
            boundary_elided_external_snapshot: false,
            causal_precondition_external_snapshot: None,
            winner_action: None,
            outcome_feedback: None,
            layout_postcondition_owner: LayoutPostconditionOwner::Ime,
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
            boundary_elided_external_snapshot: false,
            causal_precondition_external_snapshot: None,
            winner_action: None,
            outcome_feedback: None,
            layout_postcondition_owner: LayoutPostconditionOwner::Caller,
        }
    }

    pub(crate) fn with_expected_tail(mut self, expected_tail: VisibleTailSnapshot) -> Self {
        self.expected_tail = Some(expected_tail);
        self
    }

    pub(crate) fn with_boundary_elided_external_snapshot(mut self, allowed: bool) -> Self {
        self.boundary_elided_external_snapshot = allowed;
        self
    }

    pub(crate) fn with_causal_precondition_external_snapshot(
        mut self,
        precondition: String,
    ) -> Self {
        self.causal_precondition_external_snapshot = Some(precondition);
        self
    }

    pub(crate) fn with_winner_action(mut self, winner_action: EditAction) -> Self {
        self.winner_action = Some(winner_action);
        self
    }

    pub(crate) fn with_outcome_feedback(
        mut self,
        outcome_feedback: PendingSystemOutcomeFeedback,
    ) -> Self {
        self.outcome_feedback = Some(outcome_feedback);
        self
    }

    fn is_noop(&self) -> bool {
        self.backspaces == 0 && self.text.is_empty()
    }

    fn ime_owns_layout_postcondition(&self) -> bool {
        self.layout_postcondition_owner == LayoutPostconditionOwner::Ime
    }
}

impl LayIbusEngine {
    pub(crate) fn can_replace_committed_tail(&self, backspaces: u32) -> bool {
        CommittedTailOutputProfile::select(
            self.cursor_cell_width,
            self.surrounding_text_supported,
            backspaces,
        )
        .can_execute()
    }

    #[cfg(test)]
    pub(crate) fn new(
        path: String,
        shared: Shared,
        layout_is_ru: bool,
        managed_input: bool,
        config: LayConfig,
    ) -> Self {
        let profile = if layout_is_ru {
            lay::exact_layout_authority::FactoryEngineProfile::Ru
        } else {
            lay::exact_layout_authority::FactoryEngineProfile::UsQwerty
        };
        Self::new_with_factory_profile(path, shared, profile, managed_input, config)
    }

    pub(crate) fn new_from_component(
        path: String,
        shared: Shared,
        component_name: &str,
        managed_input: bool,
        config: LayConfig,
    ) -> Self {
        Self::new_with_factory_profile(
            path,
            shared,
            lay::exact_layout_authority::FactoryEngineProfile::from_component_name(component_name),
            managed_input,
            config,
        )
    }

    fn new_with_factory_profile(
        path: String,
        shared: Shared,
        factory_engine_profile: lay::exact_layout_authority::FactoryEngineProfile,
        managed_input: bool,
        config: LayConfig,
    ) -> Self {
        warm_runtime(&config);
        let (handoff_tail_buffer, handoff_tail_epoch, handoff_focus_receipt) = {
            let state = shared.lock().expect("lay ime state poisoned");
            (
                state.handoff_tail_buffer.clone(),
                state.handoff_tail_epoch,
                state.handoff_focus_receipt.clone(),
            )
        };
        let mut engine = Self {
            path,
            shared,
            buffer: String::new(),
            composition_cursor: 0,
            tail_buffer: handoff_tail_buffer,
            tail_epoch: handoff_tail_epoch,
            focus_receipt: handoff_focus_receipt,
            preedit_suffix: String::new(),
            preedit_candidates: Vec::new(),
            preedit_replacement_targets: Vec::new(),
            preedit_candidate_index: 0,
            preedit_fast: Default::default(),
            preedit_dirty: false,
            pending_display_frame: None,
            pending_passthrough_preedit_clear: false,
            cursor_cell_width: 0,
            surrounding_text_supported: false,
            surrounding_text_snapshot: None,
            surrounding_observation_revision: 0,
            factory_engine_profile,
            layout_is_ru: factory_engine_profile.initial_layout_is_ru(),
            shift_active: false,
            shift_used_as_modifier: false,
            shift_pressed_at: None,
            alt_completion_active: false,
            alt_used_as_modifier: false,
            handled_press_keycodes: Default::default(),
            last_shift_release_at: None,
            last_commit_at: None,
            last_tail_input_at: None,
            recent_committed_tail_replace: None,
            pending_visible_postcondition: None,
            pending_ime_completion_learning: None,
            suppress_next_committed_tail_autocorrect: false,
            word_input_mode: None,
            managed_input,
            config,
            atomic_route_active: false,
            atomic_speculation: false,
            deferred_layout_actions: Vec::new(),
            deferred_learning_actions: Vec::new(),
        };
        engine.rebuild_preedit_fast_from_tail();
        engine
    }

    pub(super) fn reset_for_ibus_focus_change(&mut self) {
        self.invalidate_input_frame_background_work();
        self.pending_ime_completion_learning = None;
        let preserve_tail =
            self.should_preserve_focus_handoff() || self.shared_active_path_preserved();
        self.buffer.clear();
        self.composition_cursor = 0;
        self.preedit_suffix.clear();
        self.preedit_candidates.clear();
        self.preedit_replacement_targets.clear();
        self.preedit_candidate_index = 0;
        self.preedit_fast.clear_candidate_tracking();
        self.preedit_dirty = false;
        self.pending_display_frame = None;
        self.pending_passthrough_preedit_clear = false;
        self.shift_pressed_at = None;
        self.last_shift_release_at = None;
        if !preserve_tail {
            self.last_tail_input_at = None;
            self.recent_committed_tail_replace = None;
            self.word_input_mode = None;
            self.suppress_next_committed_tail_autocorrect = false;
            self.clear_autocorrect_suppression_handoff();
        }
        self.shift_active = false;
        self.shift_used_as_modifier = false;
        self.alt_completion_active = false;
        self.alt_used_as_modifier = false;
        self.handled_press_keycodes.clear();
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
        self.invalidate_input_frame_background_work();
        // GTK resets the IBus context after an unhandled committed-tail
        // Backspace. Keep only an edit trajectory that was armed immediately
        // before that Backspace; focus changes still clear it unconditionally.
        if !self
            .pending_ime_completion_learning
            .as_ref()
            .is_some_and(|pending| pending.editing)
        {
            self.pending_ime_completion_learning = None;
        }
        self.buffer.clear();
        self.composition_cursor = 0;
        self.preedit_suffix.clear();
        self.preedit_candidates.clear();
        self.preedit_replacement_targets.clear();
        self.preedit_candidate_index = 0;
        self.preedit_fast.reset();
        self.preedit_dirty = false;
        self.pending_display_frame = None;
        self.pending_passthrough_preedit_clear = false;
        self.shift_pressed_at = None;
        self.last_shift_release_at = None;
        self.recent_committed_tail_replace = None;
        self.shift_active = false;
        self.shift_used_as_modifier = false;
        self.alt_completion_active = false;
        self.alt_used_as_modifier = false;
        self.handled_press_keycodes.clear();
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
        emitter: &mut EngineOutput<'_, '_>,
        request: CommittedTailReplaceRequest,
    ) -> fdo::Result<bool> {
        if request.is_noop() {
            return Ok(false);
        }
        let source = request.source;
        let backspaces = request.backspaces;
        let intent = request.intent;
        let boundary_elided_external_snapshot = request.boundary_elided_external_snapshot;
        let causal_precondition_external_snapshot =
            request.causal_precondition_external_snapshot.clone();
        if !matches!(
            intent,
            TextTransitionIntent::ImeAutocorrect | TextTransitionIntent::ImeAutoUndo
        ) {
            self.clear_pending_ime_auto_undo("non_undo_committed_tail_replace");
        }
        let suppress_next_autocorrect = request.suppress_next_autocorrect;
        let ime_owns_layout_postcondition = request.ime_owns_layout_postcondition();
        let winner_action = request.winner_action.clone();
        let outcome_feedback = request.outcome_feedback.clone().or_else(|| {
            winner_action
                .as_ref()
                .map(|action| PendingSystemOutcomeFeedback::from_winner(source, action))
        });
        let mut visible_state =
            VisibleFieldState::committed_tail(self.tail_buffer.clone(), Some(self.path.clone()))
                .with_epoch(self.tail_epoch);
        if let Some(observation) = committed_tail_external_observation(
            source,
            intent,
            self.surrounding_text_snapshot.as_ref(),
            &self.tail_buffer,
            backspaces as usize,
            boundary_elided_external_snapshot,
            causal_precondition_external_snapshot.as_deref(),
        ) {
            visible_state = if observation.trailing_boundary_elided {
                visible_state.with_boundary_elided_external_tail_before_cursor(
                    observation.tail_before_cursor,
                    observation.has_selection,
                )
            } else {
                visible_state.with_external_tail_before_cursor(
                    observation.tail_before_cursor,
                    observation.has_selection,
                )
            };
        }
        let transition_candidate = LatentTextTransitionCandidate::new(
            source,
            backspaces,
            request.text,
            intent,
            request.expected_tail,
        );
        let transition_candidate = if let Some(winner_action) = winner_action {
            transition_candidate.with_selected_action(winner_action)
        } else {
            transition_candidate
        };
        let (plan, edit_action) = match decide_text_transition(&visible_state, transition_candidate)
        {
            TextTransitionDecision::AlreadyApplied => {
                trace::record_committed_tail_replace(
                    source,
                    "target_state_already_observed",
                    backspaces,
                    "",
                );
                return Ok(true);
            }
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
                        rejection.reason(),
                        backspaces,
                        action.to_text(),
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
        let logical_text = edit_action.to_text().to_string();
        lay::action_log::record_candidate_edit_action_before_apply(
            &edit_action,
            lay::action_log::MutationLogRoute::IME_COMMITTED_TAIL,
            None,
        );
        let backend_action =
            lay::text_edit::authorize_backend_edit(TextEditBackend::Ime, edit_action.clone());
        let backend_reason = backend_action.reason;
        let Some(authorized_edit) = backend_action.into_authorized() else {
            trace::record_committed_tail_replace(
                source,
                backend_reason,
                backspaces,
                edit_action.to_text(),
            );
            return Ok(false);
        };
        let Some(authorized_plan) = authorized_edit.action().plan() else {
            trace::record_committed_tail_replace(
                source,
                "authorized_edit_without_plan",
                backspaces,
                "",
            );
            return Ok(false);
        };
        if authorized_plan != &plan {
            trace::record_committed_tail_replace(
                source,
                "authorized_edit_plan_mismatch",
                backspaces,
                "",
            );
            return Ok(false);
        }
        let surrounding_postcondition_available = self.surrounding_text_supported;
        let output_profile = CommittedTailOutputProfile::select(
            self.cursor_cell_width,
            surrounding_postcondition_available,
            backspaces,
        );
        if !output_profile.can_execute() {
            trace::record_committed_tail_replace_guard(
                source,
                output_profile.output_route(),
                backspaces,
                "surrounding_text_snapshot_or_terminal_route",
                "unavailable",
            );
            return Ok(false);
        }
        if (authorized_plan.move_left != 0 || authorized_plan.move_right != 0)
            && output_profile != CommittedTailOutputProfile::SurroundingText
        {
            trace::record_committed_tail_replace(
                source,
                "cursor_plan_requires_surrounding_text",
                backspaces,
                &logical_text,
            );
            return Ok(false);
        }
        let text = authorized_plan.insert.clone();
        let now = Instant::now();
        self.last_commit_at = Some(now);
        self.publish_active_path_preserve_handoff(now + Duration::from_millis(700));
        if suppress_next_autocorrect {
            self.suppress_next_committed_tail_autocorrect = true;
            self.publish_autocorrect_suppression_handoff();
        }
        if self.should_skip_duplicate_committed_tail_replace(backspaces, &logical_text, now) {
            trace::record_committed_tail_replace(
                source,
                "duplicate_skip",
                backspaces,
                &logical_text,
            );
            return Ok(true);
        }
        let total_started = Instant::now();
        let clear_started = Instant::now();
        self.clear_preedit(emitter).await?;
        let clear_us = clear_started.elapsed().as_micros();
        let output_route = output_profile.output_route();
        trace::record_committed_tail_replace(source, output_route, backspaces, &logical_text);
        let mut delete_us = 0;
        let commit_text = if output_profile.uses_terminal_erase() {
            terminal_erase_prefix(backspaces) + &text
        } else {
            forward_cursor_steps(emitter, KEY_LEFT, authorized_plan.move_left).await?;
            let delete_started = Instant::now();
            if authorized_plan.backspaces > 0 {
                emitter
                    .delete_surrounding_text(
                        -(authorized_plan.backspaces as i32),
                        authorized_plan.backspaces,
                    )
                    .await
                    .map_err(|e| fdo::Error::Failed(e.to_string()))?;
            }
            delete_us = delete_started.elapsed().as_micros();
            text.clone()
        };
        let commit_started = Instant::now();
        if !commit_text.is_empty() {
            emitter
                .commit_text(make_ibus_text(commit_text))
                .await
                .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        }
        forward_cursor_steps(emitter, KEY_RIGHT, authorized_plan.move_right).await?;
        let commit_us = commit_started.elapsed().as_micros();
        let state_started = Instant::now();
        for _ in 0..backspaces {
            self.tail_buffer.pop();
            self.preedit_fast.backspace();
        }
        self.surrounding_text_snapshot = None;
        self.tail_buffer.push_str(&logical_text);
        self.preedit_fast.reset();
        for ch in logical_text.chars() {
            self.preedit_fast.push(ch);
        }
        if logical_text.chars().last().is_some_and(char::is_whitespace) {
            self.word_input_mode = None;
        }
        self.publish_tail_handoff();
        self.buffer.clear();
        self.composition_cursor = 0;
        self.clear_preedit_completion_state();
        let deferred_layout_sync_text = (ime_owns_layout_postcondition
            && surrounding_postcondition_available)
            .then(|| logical_text.clone());
        trace::record(format!(
            r#"{{"kind":"ibus_layout_postcondition_route","intent":"{intent:?}","surrounding_dispatch":{surrounding_postcondition_available},"deferred":{}}}"#,
            deferred_layout_sync_text.is_some(),
        ));
        if ime_owns_layout_postcondition && deferred_layout_sync_text.is_none() {
            self.sync_layout_after_committed_text(&logical_text, "committed_tail_immediate");
        }
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
            text: logical_text,
            at: now,
        });
        if surrounding_postcondition_available {
            self.arm_visible_postcondition_from_surrounding_dispatch(
                now,
                outcome_feedback,
                deferred_layout_sync_text,
            );
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommittedTailExternalObservation {
    tail_before_cursor: Option<String>,
    has_selection: bool,
    trailing_boundary_elided: bool,
}

fn committed_tail_external_observation(
    source: VisibleTailSource,
    intent: TextTransitionIntent,
    snapshot: Option<&SurroundingTextSnapshot>,
    visible_tail: &str,
    backspaces: usize,
    allow_trailing_boundary_elision: bool,
    causal_precondition: Option<&str>,
) -> Option<CommittedTailExternalObservation> {
    let snapshot = snapshot?;
    if intent == TextTransitionIntent::ImeAutoUndo
        && !snapshot.has_selection()
        && causal_precondition.is_some_and(|precondition| {
            matches!(
                surrounding_snapshot_match_for_suffix(snapshot, precondition),
                ExternalSuffixMatch::Exact | ExternalSuffixMatch::TrailingBoundaryElided
            )
        })
    {
        trace::record(
            r#"{"kind":"ibus_auto_undo_authority","evidence":"causal_precondition_snapshot"}"#,
        );
        return None;
    }
    if source == VisibleTailSource::ImeCommittedTail
        && snapshot.text.is_empty()
        && !snapshot.has_selection()
    {
        return None;
    }

    if allow_trailing_boundary_elision
        && source == VisibleTailSource::ImeCommittedTail
        && intent == TextTransitionIntent::ImeAutoUndo
        && !snapshot.has_selection()
    {
        let without_boundary = visible_tail.trim_end_matches(char::is_whitespace);
        let boundary_chars = visible_tail
            .chars()
            .count()
            .saturating_sub(without_boundary.chars().count());
        if boundary_chars > 0
            && backspaces > boundary_chars
            && snapshot
                .suffix_before_cursor(without_boundary.chars().count())
                .as_deref()
                == Some(without_boundary)
        {
            return Some(CommittedTailExternalObservation {
                tail_before_cursor: snapshot.suffix_before_cursor(backspaces - boundary_chars),
                has_selection: false,
                trailing_boundary_elided: true,
            });
        }
    }

    Some(CommittedTailExternalObservation {
        tail_before_cursor: snapshot.suffix_before_cursor(backspaces),
        has_selection: snapshot.has_selection(),
        trailing_boundary_elided: false,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExternalSuffixMatch {
    Exact,
    TrailingBoundaryElided,
    Missing,
}

fn surrounding_snapshot_match_for_suffix(
    snapshot: &SurroundingTextSnapshot,
    expected_suffix: &str,
) -> ExternalSuffixMatch {
    if snapshot
        .suffix_before_cursor(expected_suffix.chars().count())
        .as_deref()
        == Some(expected_suffix)
    {
        return ExternalSuffixMatch::Exact;
    }
    let without_boundary = expected_suffix.trim_end_matches(char::is_whitespace);
    if without_boundary.len() != expected_suffix.len()
        && !without_boundary.is_empty()
        && snapshot
            .suffix_before_cursor(without_boundary.chars().count())
            .as_deref()
            == Some(without_boundary)
    {
        ExternalSuffixMatch::TrailingBoundaryElided
    } else {
        ExternalSuffixMatch::Missing
    }
}

async fn forward_cursor_steps(
    emitter: &mut EngineOutput<'_, '_>,
    keyval: u32,
    count: u32,
) -> fdo::Result<()> {
    for _ in 0..count {
        emitter
            .forward_key_event(keyval, 0, 0)
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        emitter
            .forward_key_event(keyval, 0, RELEASE_MASK)
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
    }
    Ok(())
}

fn warm_runtime(config: &LayConfig) {
    lay::config::publish_runtime_config(config);
    lay::hot_field::set_process_policy(lay::hot_field::HotFieldPolicy::ime());
    #[cfg(not(test))]
    {
        if config.active_text_backend().should_try_ime() {
            lay::typing_cpu::TypingCpu::ensure_ime_warmup_started();
        } else {
            lay::lexicon::warm_up();
        }
        if !config.active_text_backend().should_try_ime()
            && (config.auto_replace || config.typing_assist || config.auto_switch_layout)
        {
            lay::typing_assist::warm_up();
            std::thread::spawn(lay::typing_cpu::TypingCpu::warm_all);
        }
    }
}

fn terminal_erase_prefix(count: u32) -> String {
    "\u{7f}".repeat(count as usize)
}

#[cfg(test)]
mod tests {
    use super::{
        committed_tail_external_observation, CommittedTailExternalObservation,
        CommittedTailOutputProfile, CommittedTailReplaceRequest, LayIbusEngine,
        RecentCommittedTailReplace, SurroundingTextSnapshot,
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
        assert!(!request.ime_owns_layout_postcondition());
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
        assert!(request.ime_owns_layout_postcondition());
    }

    #[test]
    fn ime_auto_undo_request_keeps_recorded_undo_transition() {
        let request = CommittedTailReplaceRequest::ime_auto_undo(9, "проверрка ".to_string());

        assert_eq!(request.source, VisibleTailSource::ImeCommittedTail);
        assert_eq!(request.intent, TextTransitionIntent::ImeAutoUndo);
        assert!(request.suppress_next_autocorrect);
    }

    #[test]
    fn empty_surrounding_text_is_optional_for_ime_owned_manual_toggle() {
        let snapshot = SurroundingTextSnapshot::new(String::new(), 0, 0);

        assert_eq!(
            committed_tail_external_observation(
                VisibleTailSource::ImeCommittedTail,
                TextTransitionIntent::ImeManualToggle,
                Some(&snapshot),
                "abc",
                3,
                false,
                None,
            ),
            None
        );
    }

    #[test]
    fn empty_surrounding_text_still_guards_daemon_bridge_edits() {
        let snapshot = SurroundingTextSnapshot::new(String::new(), 0, 0);

        assert_eq!(
            committed_tail_external_observation(
                VisibleTailSource::DaemonWordBuffer,
                TextTransitionIntent::DaemonBridge,
                Some(&snapshot),
                "abc",
                3,
                false,
                None,
            ),
            Some(CommittedTailExternalObservation {
                tail_before_cursor: None,
                has_selection: false,
                trailing_boundary_elided: false,
            })
        );
    }

    #[test]
    fn recorded_undo_classifies_an_exact_full_tail_boundary_elision() {
        let snapshot = SurroundingTextSnapshot::new("собака".to_string(), 6, 6);

        assert_eq!(
            committed_tail_external_observation(
                VisibleTailSource::ImeCommittedTail,
                TextTransitionIntent::ImeAutoUndo,
                Some(&snapshot),
                "собака ",
                7,
                true,
                None,
            ),
            Some(CommittedTailExternalObservation {
                tail_before_cursor: Some("собака".to_string()),
                has_selection: false,
                trailing_boundary_elided: true,
            })
        );
    }

    #[test]
    fn recorded_undo_classifies_boundary_elision_after_sentence_context() {
        let snapshot = SurroundingTextSnapshot::new("до собака".to_string(), 9, 9);

        assert_eq!(
            committed_tail_external_observation(
                VisibleTailSource::ImeCommittedTail,
                TextTransitionIntent::ImeAutoUndo,
                Some(&snapshot),
                "собака ",
                7,
                true,
                None,
            ),
            Some(CommittedTailExternalObservation {
                tail_before_cursor: Some("собака".to_string()),
                has_selection: false,
                trailing_boundary_elided: true,
            })
        );
    }

    #[test]
    fn automatic_correction_cannot_claim_boundary_elision() {
        let snapshot = SurroundingTextSnapshot::new("собака".to_string(), 6, 6);

        assert_eq!(
            committed_tail_external_observation(
                VisibleTailSource::ImeCommittedTail,
                TextTransitionIntent::ImeAutocorrect,
                Some(&snapshot),
                "собака ",
                7,
                true,
                None,
            ),
            Some(CommittedTailExternalObservation {
                tail_before_cursor: None,
                has_selection: false,
                trailing_boundary_elided: false,
            })
        );
    }

    #[test]
    fn recorded_undo_accepts_exact_causal_precondition_from_a_lagging_client() {
        let snapshot = SurroundingTextSnapshot::new("до cj,frf".to_string(), 9, 9);

        assert_eq!(
            committed_tail_external_observation(
                VisibleTailSource::ImeCommittedTail,
                TextTransitionIntent::ImeAutoUndo,
                Some(&snapshot),
                "до собака ",
                7,
                false,
                Some("cj,frf "),
            ),
            None
        );
    }

    #[test]
    fn non_undo_edits_cannot_claim_causal_precondition_authority() {
        let snapshot = SurroundingTextSnapshot::new("cj,frf".to_string(), 6, 6);

        assert_eq!(
            committed_tail_external_observation(
                VisibleTailSource::ImeCommittedTail,
                TextTransitionIntent::ImeAutocorrect,
                Some(&snapshot),
                "собака ",
                7,
                false,
                Some("cj,frf "),
            ),
            Some(CommittedTailExternalObservation {
                tail_before_cursor: None,
                has_selection: false,
                trailing_boundary_elided: false,
            })
        );
    }

    #[test]
    fn unrelated_snapshot_cannot_claim_causal_precondition_authority() {
        let snapshot = SurroundingTextSnapshot::new("другой текст".to_string(), 12, 12);

        assert_eq!(
            committed_tail_external_observation(
                VisibleTailSource::ImeCommittedTail,
                TextTransitionIntent::ImeAutoUndo,
                Some(&snapshot),
                "собака ",
                7,
                false,
                Some("cj,frf "),
            ),
            Some(CommittedTailExternalObservation {
                tail_before_cursor: Some("й текст".to_string()),
                has_selection: false,
                trailing_boundary_elided: false,
            })
        );
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
    fn terminal_profile_keeps_existing_erase_route() {
        let profile = CommittedTailOutputProfile::select(9, false, 7);

        assert_eq!(profile, CommittedTailOutputProfile::TerminalErase);
        assert_eq!(profile.output_route(), "terminal_erase_commit");
        assert!(profile.uses_terminal_erase());
    }

    #[test]
    fn proven_surrounding_text_precedes_terminal_erase() {
        let profile = CommittedTailOutputProfile::select(9, true, 7);

        assert_eq!(profile, CommittedTailOutputProfile::SurroundingText);
        assert_eq!(profile.output_route(), "surrounding_text_delete_commit");
        assert!(!profile.uses_terminal_erase());
    }

    #[test]
    fn unproven_generic_delete_backend_is_unavailable() {
        let profile = CommittedTailOutputProfile::select(0, false, 7);

        assert_eq!(profile, CommittedTailOutputProfile::Unavailable);
        assert!(!profile.can_execute());
    }

    #[test]
    fn chromium_style_engine_without_delete_capability_rejects_bridge_preflight() {
        let mut engine = engine();
        engine.cursor_cell_width = 0;
        engine.surrounding_text_supported = false;

        assert!(!engine.can_replace_committed_tail(7));
        assert!(engine.can_replace_committed_tail(0));
    }

    #[test]
    fn surrounding_text_capability_admits_bridge_preflight() {
        let mut engine = engine();
        engine.cursor_cell_width = 0;
        engine.surrounding_text_supported = true;

        assert!(engine.can_replace_committed_tail(7));
    }

    #[test]
    fn advertised_surrounding_text_proves_the_delete_backend() {
        let profile = CommittedTailOutputProfile::select(9, true, 7);

        assert_eq!(profile, CommittedTailOutputProfile::SurroundingText);
        assert!(profile.can_execute());
    }

    #[test]
    fn plain_commits_do_not_select_delete_profile() {
        let profile = CommittedTailOutputProfile::select(9, false, 0);

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
