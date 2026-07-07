use lay::config::LayConfig;
use std::time::Instant;

use super::preedit::PreeditFastState;
use super::protocol::Shared;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WordInputMode {
    ManagedCommit,
    TerminalPassthrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ManualToggleAuthority {
    ImeActiveComposition,
    ImeCommittedTail,
    DaemonWordBuffer,
}

#[derive(Debug, Clone)]
pub(super) struct RecentCommittedTailReplace {
    pub(super) backspaces: u32,
    pub(super) text: String,
    pub(super) at: Instant,
}

#[derive(Debug, Clone)]
pub(super) struct PendingSpaceCommittedTailReplace {
    pub(super) backspaces: u32,
    pub(super) replacement: String,
    pub(super) original: String,
    pub(super) started_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SurroundingTextSnapshot {
    pub(super) text: String,
    pub(super) cursor_pos: u32,
    pub(super) anchor_pos: u32,
}

impl SurroundingTextSnapshot {
    pub(super) fn new(text: String, cursor_pos: u32, anchor_pos: u32) -> Self {
        Self {
            text,
            cursor_pos,
            anchor_pos,
        }
    }

    pub(super) fn suffix_before_cursor(&self, chars: usize) -> Option<String> {
        if chars == 0 {
            return Some(String::new());
        }
        let cursor = self.cursor_pos as usize;
        if cursor < chars || self.text.chars().count() < cursor {
            return None;
        }
        Some(
            self.text
                .chars()
                .take(cursor)
                .skip(cursor - chars)
                .collect(),
        )
    }

    pub(super) fn matches_delete_suffix(&self, expected: &str, chars: usize) -> bool {
        self.cursor_pos == self.anchor_pos
            && expected.chars().count() == chars
            && self
                .suffix_before_cursor(chars)
                .as_deref()
                .is_some_and(|actual| actual == expected)
    }
}

pub(crate) struct LayIbusEngine {
    pub(super) path: String,
    pub(super) shared: Shared,
    pub(super) buffer: String,
    pub(super) composition_cursor: usize,
    pub(super) tail_buffer: String,
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
    pub(super) pending_space_committed_tail_replace: Option<PendingSpaceCommittedTailReplace>,
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
}
