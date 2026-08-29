use lay::config::LayConfig;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use super::preedit::PreeditFastState;
use super::protocol::{ExactManualToggleSuppression, Shared};

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

#[path = "engine/types.rs"]
mod types;
pub(super) use types::{
    DeferredLayoutAction, DeferredLearningAction, InputFrameIdentity, ManualToggleAuthority,
    PendingImeCompletionLearning, PendingSystemOutcomeFeedback, PendingVisiblePostcondition,
    RecentCommittedTailReplace, SurroundingTextSnapshot, SystemOutcomeKind, WordInputMode,
};

#[derive(Clone)]
pub(crate) struct LayIbusEngine {
    pub(super) path: String,
    pub(super) shared: Shared,
    pub(super) buffer: String,
    pub(super) composition_cursor: usize,
    pub(super) tail_buffer: String,
    pub(super) tail_epoch: u64,
    pub(super) focus_receipt: Option<String>,
    pub(super) focus_serial: u64,
    pub(super) runtime_owner_lease_identity: u64,
    pub(super) preedit_visible: bool,
    pub(super) preedit_suffix: String,
    pub(super) preedit_candidates: Vec<String>,
    pub(super) preedit_replacement_targets: Vec<Option<String>>,
    pub(super) preedit_candidate_index: usize,
    pub(super) preedit_fast: PreeditFastState,
    pub(super) preedit_dirty: bool,
    pub(super) pending_display_frame: Option<InputFrameIdentity>,
    pub(super) pending_passthrough_preedit_clear: bool,
    pub(super) cursor_cell_width: i32,
    pub(super) content_purpose: u32,
    pub(super) content_hints: u32,
    pub(super) surrounding_text_supported: bool,
    pub(super) surrounding_text_snapshot: Option<SurroundingTextSnapshot>,
    pub(super) surrounding_observation_revision: u64,
    pub(super) factory_engine_profile: lay::exact_layout_authority::FactoryEngineProfile,
    pub(super) layout_is_ru: bool,
    pub(super) layout_generation: u64,
    pub(super) shift_active: bool,
    pub(super) shift_used_as_modifier: bool,
    pub(super) shift_pressed_at: Option<Instant>,
    pub(super) alt_completion_active: bool,
    pub(super) alt_used_as_modifier: bool,
    pub(super) handled_press_keycodes: BTreeSet<u32>,
    pub(super) last_shift_release_at: Option<Instant>,
    pub(super) last_commit_at: Option<Instant>,
    pub(super) last_tail_input_at: Option<Instant>,
    pub(super) recent_committed_tail_replace: Option<RecentCommittedTailReplace>,
    pub(super) pending_manual_toggle: bool,
    pub(super) pending_visible_postcondition: Option<PendingVisiblePostcondition>,
    pub(super) pending_ime_completion_learning: Option<PendingImeCompletionLearning>,
    pub(super) suppress_next_committed_tail_autocorrect: bool,
    pub(super) exact_manual_toggle_suppression: Option<ExactManualToggleSuppression>,
    pub(super) word_input_mode: Option<WordInputMode>,
    pub(super) managed_input: bool,
    pub(super) config: LayConfig,
    pub(super) atomic_route_active: bool,
    pub(super) atomic_speculation: bool,
    pub(super) deferred_layout_actions: Vec<DeferredLayoutAction>,
    pub(super) deferred_learning_actions: Vec<DeferredLearningAction>,
}

impl LayIbusEngine {
    pub(super) fn set_layout_is_ru(&mut self, target_is_ru: bool) {
        if self.layout_is_ru != target_is_ru {
            self.layout_is_ru = target_is_ru;
            self.layout_generation = next_input_identity();
        }
    }

    pub(super) fn initial_word_input_mode(&self) -> WordInputMode {
        if self.cursor_cell_width > 0
            && self.cursor_cell_width <= 3
            && !self.surrounding_text_supported
        {
            WordInputMode::TerminalPassthrough
        } else {
            WordInputMode::ManagedCommit
        }
    }

    pub(super) fn preedit_waits_for_cursor_ack(&self) -> bool {
        self.focus_receipt.is_none()
            && !self.surrounding_text_supported
            && self.cursor_cell_width > 0
    }

    pub(super) fn bind_focus_receipt(&mut self, object_path: String, client: String) -> bool {
        let receipt = format!("{object_path}\u{1f}{client}");
        if self.focus_receipt.as_deref() == Some(receipt.as_str()) {
            return false;
        }

        let replaces_existing_focus = self.focus_receipt.replace(receipt).is_some();
        self.focus_serial = next_input_identity();
        self.runtime_owner_lease_identity = next_input_identity();
        if replaces_existing_focus {
            self.buffer.clear();
            self.composition_cursor = 0;
            self.clear_preedit_completion_state();
            self.pending_passthrough_preedit_clear = false;
            self.close_committed_tail_field();
        }
        true
    }

    /// Claims the IBus engine path as a fallback focus receipt for clients
    /// that never send FocusInId. A different path cannot inherit a tail.
    pub(super) fn bind_focus_path(&mut self) -> bool {
        let next_epoch = self.tail_epoch.wrapping_add(1);
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

        self.focus_serial = next_input_identity();
        self.runtime_owner_lease_identity = next_input_identity();

        self.buffer.clear();
        self.composition_cursor = 0;
        self.clear_preedit_completion_state();
        self.pending_passthrough_preedit_clear = false;
        if let Some((tail, epoch, focus_receipt)) = preserved_handoff {
            self.tail_buffer = tail;
            self.tail_epoch = epoch;
            self.focus_receipt = focus_receipt;
            self.rebuild_preedit_fast_from_tail();
        } else {
            self.tail_buffer.clear();
            self.tail_epoch = next_epoch;
            self.preedit_fast.reset();
            self.word_input_mode = None;
            self.last_tail_input_at = None;
            self.recent_committed_tail_replace = None;
            self.pending_ime_completion_learning = None;
            self.suppress_next_committed_tail_autocorrect = false;
            self.exact_manual_toggle_suppression = None;
            self.focus_receipt
                .get_or_insert_with(|| format!("engine:{}", self.path));
        }
        true
    }

    pub(super) fn manual_toggle_authority(&self) -> ManualToggleAuthority {
        if !self.buffer.is_empty() {
            return ManualToggleAuthority::ImeActiveComposition;
        }
        let committed_tail_chars = self.last_tail_token_text().chars().count() as u32;
        // Generic cursor geometry is not proof that CommitText control
        // characters can delete client text. An explicit terminal purpose plus
        // an executable terminal-erase profile is such proof for terminals
        // that do not expose SurroundingText (notably Kitty).
        let terminal_erase_supported = self.content_purpose == IBUS_INPUT_PURPOSE_TERMINAL
            && self.can_replace_committed_tail(committed_tail_chars);
        if committed_tail_chars > 0 && (self.surrounding_text_supported || terminal_erase_supported)
        {
            return ManualToggleAuthority::ImeCommittedTail;
        }
        ManualToggleAuthority::DaemonWordBuffer
    }

    pub(super) fn live_composition_enabled(&self) -> bool {
        self.managed_input && self.config.active_text_backend().should_try_ime()
    }

    pub(super) const fn legacy_key_route_allowed(&self) -> bool {
        !self.atomic_route_active
    }

    pub(super) fn has_live_composition_state(&self) -> bool {
        self.preedit_visible
            || !self.buffer.is_empty()
            || !self.preedit_suffix.is_empty()
            || !self.preedit_candidates.is_empty()
            || self.preedit_dirty
    }

    pub(super) fn clear_preedit_completion_state(&mut self) {
        self.preedit_suffix.clear();
        self.preedit_candidates.clear();
        self.preedit_replacement_targets.clear();
        self.preedit_candidate_index = 0;
        self.preedit_fast.clear_candidate_tracking();
        self.preedit_dirty = false;
        self.pending_display_frame = None;
    }

    pub(super) fn set_client_capabilities(&mut self, caps: u32) {
        let surrounding_text_was_supported = self.surrounding_text_supported;
        self.surrounding_text_supported = caps & IBUS_CAP_SURROUNDING_TEXT != 0;
        if surrounding_text_was_supported != self.surrounding_text_supported {
            self.advance_surrounding_observation_revision();
        }
        if !surrounding_text_was_supported
            && self.surrounding_text_supported
            && self.word_input_mode == Some(WordInputMode::TerminalPassthrough)
        {
            self.word_input_mode = Some(WordInputMode::ManagedCommit);
            self.pending_passthrough_preedit_clear = true;
        }
        if !self.surrounding_text_supported {
            self.surrounding_text_snapshot = None;
            self.pending_manual_toggle = false;
        }
    }

    pub(super) fn set_content_type_state(&mut self, purpose: u32, hints: u32) {
        if self.content_purpose == purpose && self.content_hints == hints {
            return;
        }
        self.content_purpose = purpose;
        self.content_hints = hints;
        self.invalidate_input_frame_background_work();
        self.clear_preedit_completion_state();
        if self.content_is_sensitive() {
            self.buffer.clear();
            self.composition_cursor = 0;
            self.surrounding_text_snapshot = None;
            self.close_committed_tail_field();
        }
    }

    pub(super) fn content_is_sensitive(&self) -> bool {
        matches!(
            self.content_purpose,
            IBUS_INPUT_PURPOSE_PASSWORD | IBUS_INPUT_PURPOSE_PIN
        ) || self.content_hints & (IBUS_INPUT_HINT_PRIVATE | IBUS_INPUT_HINT_HIDDEN_TEXT) != 0
    }

    pub(super) fn content_allows_text_assistance(&self) -> bool {
        !self.content_is_sensitive()
    }

    pub(super) fn observe_external_surrounding_text(
        &mut self,
        snapshot: Option<SurroundingTextSnapshot>,
    ) {
        self.advance_surrounding_observation_revision();
        self.surrounding_text_supported = true;
        self.surrounding_text_snapshot = if self.content_is_sensitive() {
            None
        } else {
            snapshot
        };
    }

    fn advance_surrounding_observation_revision(&mut self) {
        self.surrounding_observation_revision =
            self.surrounding_observation_revision.saturating_add(1);
    }

    pub(super) fn remember_handled_press(&mut self, keycode: u32, handled: bool) {
        if handled {
            self.handled_press_keycodes.insert(keycode);
        }
    }

    pub(super) fn consume_handled_release(&mut self, keycode: u32) -> bool {
        self.handled_press_keycodes.remove(&keycode)
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
        engine.buffer.push_str("ghbdtn");

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
        engine.tail_buffer.push_str("вот ");
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
        engine.tail_buffer.push_str("typed ");
        engine.cursor_cell_width = 9;

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
        engine.tail_buffer.push_str("typed ");
        engine.cursor_cell_width = 11;
        engine.set_content_type_state(IBUS_INPUT_PURPOSE_TERMINAL, 0);

        assert_eq!(
            engine.manual_toggle_authority(),
            ManualToggleAuthority::ImeCommittedTail
        );
    }

    #[test]
    fn client_capabilities_control_surrounding_text_authority() {
        let mut engine = engine(LayConfig::default());
        engine.surrounding_text_snapshot = Some(super::SurroundingTextSnapshot::new(
            "visible".to_string(),
            7,
            7,
        ));

        engine.set_client_capabilities(1 << 5);
        assert!(engine.surrounding_text_supported);
        assert!(engine.surrounding_text_snapshot.is_some());
        assert_eq!(engine.surrounding_observation_revision, 1);

        engine.set_client_capabilities(1 | 1 << 3);
        assert!(!engine.surrounding_text_supported);
        assert!(engine.surrounding_text_snapshot.is_none());
        assert_eq!(engine.surrounding_observation_revision, 2);
    }

    #[test]
    fn acquired_atomic_route_blocks_legacy_key_mutation_for_focus() {
        let mut engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        });
        assert!(engine.legacy_key_route_allowed());

        engine.atomic_route_active = true;

        assert!(!engine.legacy_key_route_allowed());
    }

    #[test]
    fn local_tail_input_invalidates_external_surrounding_snapshot() {
        let mut engine = engine(LayConfig::default());
        engine.surrounding_text_snapshot =
            Some(super::SurroundingTextSnapshot::new(String::new(), 0, 0));

        engine.push_tail_char('x');

        assert!(engine.surrounding_text_snapshot.is_none());
        assert_eq!(engine.surrounding_observation_revision, 0);
        assert_eq!(engine.tail_buffer, "x");
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
