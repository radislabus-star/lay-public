use lay::config::{CorrectionEngine, LayConfig};
use lay::desktop::LayoutBackend;
use lay::text_backend::TextBackendPreference;
use std::sync::OnceLock;

use super::{detect_auto_layout_backend_hint, ENTER_AUTOCORRECT_EXPERIMENT_ENV};

static AUTO_LAYOUT_BACKEND_HINT: OnceLock<Option<LayoutBackend>> = OnceLock::new();

pub(super) fn active_replace_words() -> usize {
    LayConfig::load().active_replace_words()
}

pub(super) fn active_correction_engine() -> CorrectionEngine {
    LayConfig::load().active_correction_engine()
}

pub(super) fn active_layout_backend() -> LayoutBackend {
    let config = LayConfig::load();
    let backend = config.active_layout_backend();
    let configured = config.layout_backend.trim().to_ascii_lowercase();
    if configured != "auto" || backend != LayoutBackend::Gnome {
        return backend;
    }

    if let Some(hint) = *AUTO_LAYOUT_BACKEND_HINT.get_or_init(detect_auto_layout_backend_hint) {
        return hint;
    }
    backend
}

pub(super) fn active_text_backend() -> TextBackendPreference {
    LayConfig::load().active_text_backend()
}

pub(super) fn active_auto_replace() -> bool {
    LayConfig::load().auto_replace
}

pub(super) fn active_typing_assist() -> bool {
    LayConfig::load().typing_assist
}

pub(super) fn active_enter_autocorrect() -> bool {
    let cfg = LayConfig::load();
    active_enter_autocorrect_from_env(
        cfg.enter_autocorrect,
        std::env::var(ENTER_AUTOCORRECT_EXPERIMENT_ENV)
            .ok()
            .as_deref(),
    )
}

pub(super) fn active_enter_autocorrect_from_env(
    config_enabled: bool,
    env_value: Option<&str>,
) -> bool {
    if !config_enabled {
        return false;
    }
    env_value
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(true)
}

pub(super) fn active_auto_switch_layout() -> bool {
    LayConfig::load().auto_switch_layout
}

pub(super) fn active_learning_log() -> bool {
    LayConfig::load().learning_log
}

pub(super) fn active_lem_enabled_for_scope(word_count: usize) -> bool {
    LayConfig::load().lem_enabled_for_scope(word_count)
}

#[cfg(not(test))]
pub(super) fn active_typing_assist_pipeline_for_auto_replace(
    context: &str,
) -> Vec<lay::config::TypingAssistRuleConfig> {
    let cfg = LayConfig::load();
    lay::typing_context::typing_assist_pipeline_for_context(
        cfg.auto_replace,
        cfg.active_correction_safety(),
        &cfg.typing_assist_pipeline,
        context,
    )
}
