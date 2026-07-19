use lay::config::LayConfig;
use std::time::Instant;

use super::preedit::PreeditFastState;
use super::protocol::Shared;

const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 5;

#[path = "engine/types.rs"]
mod types;
pub(super) use types::{
    ManualToggleAuthority, PendingImeCompletionLearning, PendingSystemOutcomeFeedback,
    PendingVisiblePostcondition, RecentCommittedTailReplace, SurroundingTextSnapshot,
    SystemOutcomeKind, WordInputMode,
};

pub(crate) struct LayIbusEngine {
    pub(super) path: String,
    pub(super) shared: Shared,
    pub(super) buffer: String,
    pub(super) composition_cursor: usize,
    pub(super) tail_buffer: String,
    pub(super) tail_epoch: u64,
    pub(super) focus_receipt: Option<String>,
    pub(super) preedit_suffix: String,
    pub(super) preedit_candidates: Vec<String>,
    pub(super) preedit_replacement_targets: Vec<Option<String>>,
    pub(super) preedit_candidate_index: usize,
    pub(super) preedit_fast: PreeditFastState,
    pub(super) preedit_dirty: bool,
    pub(super) cursor_cell_width: i32,
    pub(super) surrounding_text_supported: bool,
    pub(super) surrounding_text_snapshot: Option<SurroundingTextSnapshot>,
    pub(super) layout_is_ru: bool,
    pub(super) shift_active: bool,
    pub(super) shift_used_as_modifier: bool,
    pub(super) alt_completion_active: bool,
    pub(super) alt_used_as_modifier: bool,
    pub(super) last_shift_release_at: Option<Instant>,
    pub(super) last_commit_at: Option<Instant>,
    pub(super) last_tail_input_at: Option<Instant>,
    pub(super) recent_committed_tail_replace: Option<RecentCommittedTailReplace>,
    pub(super) pending_visible_postcondition: Option<PendingVisiblePostcondition>,
    pub(super) pending_ime_completion_learning: Option<PendingImeCompletionLearning>,
    pub(super) suppress_next_committed_tail_autocorrect: bool,
    pub(super) word_input_mode: Option<WordInputMode>,
    pub(super) managed_input: bool,
    pub(super) config: LayConfig,
}

impl LayIbusEngine {
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
        if replaces_existing_focus {
            self.buffer.clear();
            self.composition_cursor = 0;
            self.clear_preedit_completion_state();
            self.close_committed_tail_field();
        }
        true
    }

    /// Claims the IBus engine path as a fallback focus receipt for clients
    /// that never send FocusInId. A different path cannot inherit a tail.
    pub(super) fn bind_focus_path(&mut self) -> bool {
        let next_epoch = self.tail_epoch.wrapping_add(1);
        let changed = {
            let mut state = self.shared.lock().expect("lay ime state poisoned");
            if state.active_path.as_deref() == Some(self.path.as_str()) {
                false
            } else {
                state.active_path = Some(self.path.clone());
                state.handoff_tail_buffer.clear();
                state.handoff_tail_epoch = next_epoch;
                state.handoff_focus_receipt = None;
                state.suppress_next_committed_tail_autocorrect = false;
                state.preserve_active_path_until = None;
                true
            }
        };
        if !changed {
            return false;
        }

        self.buffer.clear();
        self.composition_cursor = 0;
        self.tail_buffer.clear();
        self.tail_epoch = next_epoch;
        self.clear_preedit_completion_state();
        self.preedit_fast.reset();
        self.word_input_mode = None;
        self.last_tail_input_at = None;
        self.recent_committed_tail_replace = None;
        self.pending_ime_completion_learning = None;
        self.suppress_next_committed_tail_autocorrect = false;
        self.focus_receipt
            .get_or_insert_with(|| format!("engine:{}", self.path));
        true
    }

    pub(super) fn manual_toggle_authority(&self) -> ManualToggleAuthority {
        if !self.buffer.is_empty() {
            return ManualToggleAuthority::ImeActiveComposition;
        }
        if !self.last_tail_token_text().is_empty() {
            return ManualToggleAuthority::ImeCommittedTail;
        }
        ManualToggleAuthority::DaemonWordBuffer
    }

    pub(super) fn live_composition_enabled(&self) -> bool {
        self.managed_input && self.config.active_text_backend().should_try_ime()
    }

    pub(super) fn has_live_composition_state(&self) -> bool {
        !self.buffer.is_empty()
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
    }

    pub(super) fn set_client_capabilities(&mut self, caps: u32) {
        self.surrounding_text_supported = caps & IBUS_CAP_SURROUNDING_TEXT != 0;
        if !self.surrounding_text_supported {
            self.surrounding_text_snapshot = None;
        }
    }
}

#[cfg(test)]
#[path = "engine/profile_tests.rs"]
mod profile_tests;

#[cfg(test)]
mod tests {
    use super::{LayIbusEngine, ManualToggleAuthority};
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
    fn manual_toggle_uses_ime_authority_for_known_committed_tail() {
        let mut engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: true,
            ..LayConfig::default()
        });
        engine.tail_buffer.push_str("вот ");

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

        engine.set_client_capabilities(1 | 1 << 3);
        assert!(!engine.surrounding_text_supported);
        assert!(engine.surrounding_text_snapshot.is_none());
    }

    #[test]
    fn local_tail_input_invalidates_external_surrounding_snapshot() {
        let mut engine = engine(LayConfig::default());
        engine.surrounding_text_snapshot =
            Some(super::SurroundingTextSnapshot::new(String::new(), 0, 0));

        engine.push_tail_char('x');

        assert!(engine.surrounding_text_snapshot.is_none());
        assert_eq!(engine.tail_buffer, "x");
    }
}
