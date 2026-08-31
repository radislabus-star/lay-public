use lay::config::LayConfig;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use super::protocol::Shared;

const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 5;
const IBUS_INPUT_PURPOSE_PASSWORD: u32 = 8;
const IBUS_INPUT_PURPOSE_PIN: u32 = 9;
const IBUS_INPUT_PURPOSE_TERMINAL: u32 = 10;
const IBUS_INPUT_HINT_PRIVATE: u32 = 1 << 11;
const IBUS_INPUT_HINT_HIDDEN_TEXT: u32 = 1 << 12;

pub(super) fn next_input_identity() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed).max(1)
}

#[path = "engine/state_groups.rs"]
mod state_groups;
#[path = "engine/types.rs"]
mod types;
pub(super) use state_groups::{
    AtomicRouteState, ClientContextState, CommittedTailState, CompositionState, LayoutGestureState,
};
pub(super) use types::{
    DeferredLayoutAction, DeferredLearningAction, InputFrameIdentity, ManualToggleAuthority,
    PendingImeCompletionLearning, PendingSystemOutcomeFeedback, PendingVisiblePostcondition,
    RecentCommittedTailReplace, SurroundingTextSnapshot, SystemOutcomeKind, WordInputMode,
};

#[derive(Clone)]
pub(crate) struct LayIbusEngine {
    pub(super) path: String,
    pub(super) shared: Shared,
    pub(super) composition: CompositionState,
    pub(super) committed_tail: CommittedTailState,
    pub(super) client_context: ClientContextState,
    pub(super) layout_gesture: LayoutGestureState,
    pub(super) config: LayConfig,
    pub(super) atomic: AtomicRouteState,
}

impl LayIbusEngine {
    pub(super) fn set_layout_is_ru(&mut self, target_is_ru: bool) {
        if self.layout_gesture.layout_is_ru != target_is_ru {
            self.layout_gesture.layout_is_ru = target_is_ru;
            self.layout_gesture.layout_generation = next_input_identity();
        }
    }

    pub(super) fn initial_word_input_mode(&self) -> WordInputMode {
        if self.client_context.cursor_cell_width > 0
            && self.client_context.cursor_cell_width <= 3
            && !self.client_context.surrounding_text_supported
        {
            WordInputMode::TerminalPassthrough
        } else {
            WordInputMode::ManagedCommit
        }
    }

    pub(super) fn preedit_waits_for_cursor_ack(&self) -> bool {
        self.client_context.focus_receipt.is_none()
            && !self.client_context.surrounding_text_supported
            && self.client_context.cursor_cell_width > 0
    }

    pub(super) fn bind_focus_receipt(&mut self, object_path: String, client: String) -> bool {
        let receipt = format!("{object_path}\u{1f}{client}");
        if self.client_context.focus_receipt.as_deref() == Some(receipt.as_str()) {
            return false;
        }

        let replaces_existing_focus = self.client_context.focus_receipt.replace(receipt).is_some();
        self.client_context.focus_serial = next_input_identity();
        self.client_context.runtime_owner_lease_identity = next_input_identity();
        if replaces_existing_focus {
            self.composition.buffer.clear();
            self.composition.cursor = 0;
            self.clear_preedit_completion_state();
            self.composition.pending_passthrough_preedit_clear = false;
            self.close_committed_tail_field();
        }
        true
    }

    /// Claims the IBus engine path as a fallback focus receipt for clients
    /// that never send FocusInId. A different path cannot inherit a tail.
    pub(super) fn bind_focus_path(&mut self) -> bool {
        let next_epoch = self.committed_tail.epoch.wrapping_add(1);
        let (changed, preserved_handoff) = {
            let mut state = self.shared.lock().expect("lay ime state poisoned");
            if state.active_path.as_deref() == Some(self.path.as_str()) {
                (false, None)
            } else {
                let now = Instant::now();
                let preserve_handoff = state
                    .preserve_active_path_until
                    .is_some_and(|until| now <= until);
                if !preserve_handoff {
                    state.preserve_active_path_until = None;
                    state.exact_manual_toggle_handoff_epoch = None;
                    state.exact_manual_toggle_handoff_path = None;
                }
                state.active_path = Some(self.path.clone());
                let handoff = preserve_handoff.then(|| {
                    (
                        state.handoff_tail_buffer.clone(),
                        state.handoff_tail_epoch,
                        state.handoff_focus_receipt.clone(),
                    )
                });
                if !preserve_handoff {
                    state.handoff_tail_buffer.clear();
                    state.handoff_tail_epoch = next_epoch;
                    state.handoff_focus_receipt = None;
                    state.suppress_next_committed_tail_autocorrect = false;
                    state.exact_manual_toggle_suppression = None;
                    state.pending_auto_undo = None;
                    state.pending_auto_undo_retry = None;
                    state.shift_gesture_handoff = None;
                }
                (true, handoff)
            }
        };
        if !changed {
            return false;
        }

        self.client_context.focus_serial = next_input_identity();
        self.client_context.runtime_owner_lease_identity = next_input_identity();

        self.composition.buffer.clear();
        self.composition.cursor = 0;
        self.clear_preedit_completion_state();
        self.composition.pending_passthrough_preedit_clear = false;
        if let Some((tail, epoch, focus_receipt)) = preserved_handoff {
            self.committed_tail.buffer = tail;
            self.committed_tail.epoch = epoch;
            self.client_context.focus_receipt = focus_receipt;
            self.rebuild_preedit_fast_from_tail();
        } else {
            self.committed_tail.buffer.clear();
            self.committed_tail.epoch = next_epoch;
            self.composition.preedit_fast.reset();
            self.composition.word_input_mode = None;
            self.committed_tail.last_input_at = None;
            self.committed_tail.recent_replace = None;
            self.committed_tail.pending_completion_learning = None;
            self.committed_tail.suppress_next_autocorrect = false;
            self.committed_tail.exact_manual_toggle_suppression = None;
            self.client_context
                .focus_receipt
                .get_or_insert_with(|| format!("engine:{}", self.path));
        }
        true
    }

    pub(super) fn manual_toggle_authority(&self) -> ManualToggleAuthority {
        if !self.composition.buffer.is_empty() {
            return ManualToggleAuthority::ImeActiveComposition;
        }
        let committed_tail_chars = self.last_tail_token_text().chars().count() as u32;
        // Generic cursor geometry is not proof that CommitText control
        // characters can delete client text. An explicit terminal purpose plus
        // an executable terminal-erase profile is such proof for terminals
        // that do not expose SurroundingText (notably Kitty).
        let terminal_erase_supported = self.terminal_committed_tail_executor_available();
        if committed_tail_chars > 0
            && (self.client_context.surrounding_text_supported || terminal_erase_supported)
        {
            return ManualToggleAuthority::ImeCommittedTail;
        }
        ManualToggleAuthority::DaemonWordBuffer
    }

    pub(super) fn terminal_committed_tail_executor_available(&self) -> bool {
        let committed_tail_chars = self.last_tail_token_text().chars().count() as u32;
        committed_tail_chars > 0
            && self.client_context.content_purpose == IBUS_INPUT_PURPOSE_TERMINAL
            && !self.client_context.surrounding_text_supported
            && self.can_replace_committed_tail(committed_tail_chars)
    }

    pub(super) fn live_composition_enabled(&self) -> bool {
        self.client_context.managed_input && self.config.active_text_backend().should_try_ime()
    }

    pub(super) const fn legacy_key_route_allowed(&self) -> bool {
        !self.atomic.active
    }

    pub(super) fn has_live_composition_state(&self) -> bool {
        self.composition.preedit_visible
            || !self.composition.buffer.is_empty()
            || !self.composition.preedit_suffix.is_empty()
            || !self.composition.preedit_candidates.is_empty()
            || self.composition.preedit_dirty
    }

    pub(super) fn clear_preedit_completion_state(&mut self) {
        self.composition.preedit_suffix.clear();
        self.composition.preedit_candidates.clear();
        self.composition.preedit_replacement_targets.clear();
        self.composition.preedit_candidate_index = 0;
        self.composition.preedit_display_only_pending = false;
        self.composition.preedit_fast.clear_candidate_tracking();
        self.composition.preedit_dirty = false;
        self.composition.pending_display_frame = None;
    }

    pub(super) fn set_client_capabilities(&mut self, caps: u32) {
        let surrounding_text_was_supported = self.client_context.surrounding_text_supported;
        self.client_context.surrounding_text_supported = caps & IBUS_CAP_SURROUNDING_TEXT != 0;
        if surrounding_text_was_supported != self.client_context.surrounding_text_supported {
            self.advance_surrounding_observation_revision();
        }
        if !surrounding_text_was_supported
            && self.client_context.surrounding_text_supported
            && self.composition.word_input_mode == Some(WordInputMode::TerminalPassthrough)
        {
            self.composition.word_input_mode = Some(WordInputMode::ManagedCommit);
            self.composition.pending_passthrough_preedit_clear = true;
        }
        if !self.client_context.surrounding_text_supported {
            self.client_context.surrounding_text_snapshot = None;
            self.layout_gesture.pending_manual_toggle = false;
        }
    }

    pub(super) fn set_content_type_state(&mut self, purpose: u32, hints: u32) {
        if self.client_context.content_purpose == purpose
            && self.client_context.content_hints == hints
        {
            return;
        }
        self.client_context.content_purpose = purpose;
        self.client_context.content_hints = hints;
        self.invalidate_input_frame_background_work();
        self.clear_preedit_completion_state();
        if self.content_is_sensitive() {
            self.composition.buffer.clear();
            self.composition.cursor = 0;
            self.client_context.surrounding_text_snapshot = None;
            self.close_committed_tail_field();
        }
    }

    pub(super) fn content_is_sensitive(&self) -> bool {
        matches!(
            self.client_context.content_purpose,
            IBUS_INPUT_PURPOSE_PASSWORD | IBUS_INPUT_PURPOSE_PIN
        ) || self.client_context.content_hints
            & (IBUS_INPUT_HINT_PRIVATE | IBUS_INPUT_HINT_HIDDEN_TEXT)
            != 0
    }

    pub(super) fn content_allows_text_assistance(&self) -> bool {
        !self.content_is_sensitive()
    }

    pub(super) fn observe_external_surrounding_text(
        &mut self,
        snapshot: Option<SurroundingTextSnapshot>,
    ) {
        self.advance_surrounding_observation_revision();
        self.client_context.surrounding_text_supported = true;
        self.client_context.surrounding_text_snapshot = if self.content_is_sensitive() {
            None
        } else {
            snapshot
        };
    }

    fn advance_surrounding_observation_revision(&mut self) {
        self.client_context.surrounding_observation_revision = self
            .client_context
            .surrounding_observation_revision
            .saturating_add(1);
    }

    pub(super) fn remember_handled_press(&mut self, keycode: u32, handled: bool) {
        if handled {
            self.layout_gesture.handled_press_keycodes.insert(keycode);
        }
    }

    pub(super) fn consume_handled_release(&mut self, keycode: u32) -> bool {
        self.layout_gesture.handled_press_keycodes.remove(&keycode)
    }
}

#[cfg(test)]
#[path = "engine/profile_tests.rs"]
mod profile_tests;

#[cfg(test)]
mod tests {
    use super::{LayIbusEngine, ManualToggleAuthority, IBUS_INPUT_PURPOSE_TERMINAL};
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    fn engine(config: LayConfig) -> LayIbusEngine {
        LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            config,
        )
    }

    #[test]
    fn ime_backend_enables_live_composition_without_precognition() {
        let engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: false,
            ..LayConfig::default()
        });
        assert!(engine.live_composition_enabled());
    }

    #[test]
    fn ime_backend_enables_live_composition_independently_from_gray_precognition() {
        let ime_engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        });
        assert!(ime_engine.live_composition_enabled());

        let uinput_engine = engine(LayConfig {
            text_backend: "uinput".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        });
        assert!(!uinput_engine.live_composition_enabled());
    }

    #[test]
    fn auto_backend_enables_live_composition_for_running_ibus_engine() {
        let engine = engine(LayConfig {
            text_backend: "auto".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        });
        assert!(engine.live_composition_enabled());
        assert!(engine.config.active_nanda_precognition());
    }

    #[test]
    fn zero_nanda_weights_do_not_disable_ime_input_backend() {
        let engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            nanda_l2_weight_percent: 0,
            nanda_l3_weight_percent: 0,
            ..LayConfig::default()
        });
        assert!(engine.live_composition_enabled());
        assert!(!engine.config.active_nanda_precognition());
    }

    #[test]
    fn manual_toggle_uses_daemon_authority_when_ime_does_not_own_composition() {
        let engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        });

        assert_eq!(
            engine.manual_toggle_authority(),
            ManualToggleAuthority::DaemonWordBuffer
        );
    }

    #[test]
    fn manual_toggle_uses_ime_authority_only_for_active_composition() {
        let mut engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        });
        engine.composition.buffer.push_str("ghbdtn");

        assert_eq!(
            engine.manual_toggle_authority(),
            ManualToggleAuthority::ImeActiveComposition
        );
    }

    #[test]
    fn manual_toggle_uses_ime_authority_for_proven_committed_tail() {
        let mut engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        });
        engine.committed_tail.buffer.push_str("вот ");
        engine.set_client_capabilities(1 << 5);

        assert_eq!(
            engine.manual_toggle_authority(),
            ManualToggleAuthority::ImeCommittedTail
        );
    }

    #[test]
    fn manual_toggle_delegates_unproven_committed_tail_despite_cursor_geometry() {
        let mut engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        });
        engine.committed_tail.buffer.push_str("typed ");
        engine.client_context.cursor_cell_width = 9;

        assert_eq!(
            engine.manual_toggle_authority(),
            ManualToggleAuthority::DaemonWordBuffer
        );
    }

    #[test]
    fn manual_toggle_uses_terminal_erase_authority_for_terminal_committed_tail() {
        let mut engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        });
        engine.committed_tail.buffer.push_str("typed ");
        engine.client_context.cursor_cell_width = 11;
        engine.set_content_type_state(IBUS_INPUT_PURPOSE_TERMINAL, 0);

        assert_eq!(
            engine.manual_toggle_authority(),
            ManualToggleAuthority::ImeCommittedTail
        );
    }

    #[test]
    fn client_capabilities_control_surrounding_text_authority() {
        let mut engine = engine(LayConfig::default());
        engine.client_context.surrounding_text_snapshot = Some(
            super::SurroundingTextSnapshot::new("visible".to_string(), 7, 7),
        );

        engine.set_client_capabilities(1 << 5);
        assert!(engine.client_context.surrounding_text_supported);
        assert!(engine.client_context.surrounding_text_snapshot.is_some());
        assert_eq!(engine.client_context.surrounding_observation_revision, 1);

        engine.set_client_capabilities(1 | 1 << 3);
        assert!(!engine.client_context.surrounding_text_supported);
        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert_eq!(engine.client_context.surrounding_observation_revision, 2);
    }

    #[test]
    fn acquired_atomic_route_blocks_legacy_key_mutation_for_focus() {
        let mut engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        });
        assert!(engine.legacy_key_route_allowed());

        engine.atomic.active = true;

        assert!(!engine.legacy_key_route_allowed());
    }

    #[test]
    fn local_tail_input_invalidates_external_surrounding_snapshot() {
        let mut engine = engine(LayConfig::default());
        engine.client_context.surrounding_text_snapshot =
            Some(super::SurroundingTextSnapshot::new(String::new(), 0, 0));

        engine.push_tail_char('x');

        assert!(engine.client_context.surrounding_text_snapshot.is_none());
        assert_eq!(engine.client_context.surrounding_observation_revision, 0);
        assert_eq!(engine.committed_tail.buffer, "x");
    }

    #[test]
    fn handled_press_owns_its_matching_release() {
        let mut engine = engine(LayConfig::default());

        engine.remember_handled_press(57, true);
        assert!(engine.consume_handled_release(57));
        assert!(!engine.consume_handled_release(57));

        engine.remember_handled_press(30, false);
        assert!(!engine.consume_handled_release(30));
    }
}
