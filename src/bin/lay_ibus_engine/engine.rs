use lay::config::LayConfig;
use std::time::Instant;

use super::preedit::PreeditFastState;
use super::protocol::Shared;

const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 5;

#[path = "engine/types.rs"]
mod types;
pub(super) use types::{
    ManualToggleAuthority, PendingVisiblePostcondition, RecentCommittedTailReplace,
    SurroundingTextSnapshot, WordInputMode,
};

pub(crate) struct LayIbusEngine {
    pub(super) path: String,
    pub(super) shared: Shared,
    pub(super) buffer: String,
    pub(super) composition_cursor: usize,
    pub(super) tail_buffer: String,
    pub(super) tail_epoch: u64,
    pub(super) preedit_suffix: String,
    pub(super) preedit_candidates: Vec<String>,
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
