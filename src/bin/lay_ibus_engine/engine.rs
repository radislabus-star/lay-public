use lay::config::LayConfig;
use std::time::{Duration, Instant};

use super::preedit::PreeditFastState;
use super::protocol::Shared;

pub(super) const DOUBLE_SHIFT_WINDOW: Duration = Duration::from_millis(650);

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
    pub(super) layout_is_ru: bool,
    pub(super) shift_active: bool,
    pub(super) shift_used_as_modifier: bool,
    pub(super) alt_completion_active: bool,
    pub(super) alt_used_as_modifier: bool,
    pub(super) last_shift_release_at: Option<Instant>,
    pub(super) last_commit_at: Option<Instant>,
    pub(super) managed_input: bool,
    pub(super) config: LayConfig,
}

impl LayIbusEngine {
    pub(super) fn live_composition_enabled(&self) -> bool {
        self.managed_input
            && self.config.nanda_precognition
            && self.config.active_text_backend() == lay::text_backend::TextBackendPreference::Ime
    }

    pub(super) fn has_live_composition_state(&self) -> bool {
        !self.buffer.is_empty()
            || !self.preedit_suffix.is_empty()
            || !self.preedit_candidates.is_empty()
            || self.preedit_dirty
    }
}

#[cfg(test)]
mod tests {
    use super::LayIbusEngine;
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
    fn ime_backend_alone_does_not_capture_plain_typing() {
        let engine = engine(LayConfig {
            text_backend: "ime".to_string(),
            nanda_precognition: false,
            ..LayConfig::default()
        });
        assert!(!engine.live_composition_enabled());
    }

    #[test]
    fn gray_precognition_enables_live_composition_only_in_ime_backend() {
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
}
