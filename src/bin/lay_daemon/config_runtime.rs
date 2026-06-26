use lay::config::{CorrectionEngine, LayConfig};
use lay::desktop::LayoutBackend;
use lay::text_backend::TextBackendPreference;
#[cfg(not(test))]
use std::sync::Mutex;
use std::sync::OnceLock;
#[cfg(not(test))]
use std::time::{Duration, Instant, SystemTime};

use super::{detect_auto_layout_backend_hint, ENTER_AUTOCORRECT_EXPERIMENT_ENV};

#[path = "config_runtime/nanda.rs"]
mod nanda;
pub(super) use nanda::active_nanda_wave_options;

static AUTO_LAYOUT_BACKEND_HINT: OnceLock<Option<LayoutBackend>> = OnceLock::new();

#[cfg(not(test))]
const CONFIG_CACHE_CHECK_INTERVAL: Duration = Duration::from_millis(250);

#[cfg(not(test))]
struct CachedLayConfig {
    config: LayConfig,
    modified: Option<SystemTime>,
    checked_at: Instant,
}

#[cfg(not(test))]
static CONFIG_CACHE: OnceLock<Mutex<CachedLayConfig>> = OnceLock::new();

fn current_config() -> LayConfig {
    #[cfg(test)]
    {
        let config = LayConfig::load();
        sync_lem_runtime(&config);
        config
    }
    #[cfg(not(test))]
    {
        let cache = CONFIG_CACHE.get_or_init(|| {
            Mutex::new(CachedLayConfig {
                config: LayConfig::load(),
                modified: config_modified_at(),
                checked_at: Instant::now(),
            })
        });
        let mut cache = cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if cache.checked_at.elapsed() < CONFIG_CACHE_CHECK_INTERVAL {
            sync_lem_runtime(&cache.config);
            return cache.config.clone();
        }
        cache.checked_at = Instant::now();
        let modified = config_modified_at();
        if modified != cache.modified {
            cache.config = LayConfig::load();
            cache.modified = modified;
        }
        sync_lem_runtime(&cache.config);
        cache.config.clone()
    }
}

fn sync_lem_runtime(config: &LayConfig) {
    lay::lem::set_runtime_enabled(config.lem_enabled && config.active_lem_weight() > 0.0);
}

#[cfg(not(test))]
fn config_modified_at() -> Option<SystemTime> {
    std::fs::metadata(lay::config::config_path())
        .and_then(|metadata| metadata.modified())
        .ok()
}

pub(super) fn active_replace_words() -> usize {
    current_config().active_replace_words()
}

pub(super) fn active_typing_assist_words() -> usize {
    current_config().active_typing_assist_words()
}

pub(super) fn active_correction_engine() -> CorrectionEngine {
    current_config().active_correction_engine()
}

pub(super) fn active_layout_backend() -> LayoutBackend {
    let config = current_config();
    let backend = config.active_layout_backend();
    let configured = config.layout_backend.trim().to_ascii_lowercase();
    if configured != "auto" {
        return backend;
    }
    if let Some(hint) = *AUTO_LAYOUT_BACKEND_HINT.get_or_init(detect_auto_layout_backend_hint) {
        return hint;
    }
    backend
}

pub(super) fn active_text_backend() -> TextBackendPreference {
    current_config().active_text_backend()
}

pub(super) fn active_auto_replace() -> bool {
    current_config().auto_replace
}

pub(super) fn active_typing_assist() -> bool {
    current_config().typing_assist
}

pub(super) fn active_enter_autocorrect() -> bool {
    active_enter_autocorrect_from_env(
        current_config().enter_autocorrect,
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
    match env_value {
        None => true,
        Some(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
    }
}

pub(super) fn active_auto_switch_layout() -> bool {
    current_config().auto_switch_layout
}

#[cfg(not(test))]
pub(super) fn active_correction_safety() -> lay::config::CorrectionSafety {
    current_config().active_correction_safety()
}

pub(super) fn active_learning_log() -> bool {
    current_config().debug_action_log && current_config().learning_log
}

pub(super) fn active_nanda_trace() -> bool {
    current_config().debug_action_log
}

pub(super) fn active_nanda_precognition() -> bool {
    let cfg = current_config();
    cfg.debug_action_log && cfg.active_nanda_precognition()
}

#[cfg(not(test))]
pub(super) fn active_nanda_autocorrect() -> bool {
    current_config().nanda_autocorrect
}

pub(super) fn active_lem_enabled_for_scope(word_count: usize) -> bool {
    current_config().lem_enabled_for_scope(word_count)
}

pub(super) fn active_lem_weight() -> f64 {
    current_config().active_lem_weight()
}

#[cfg(not(test))]
pub(super) fn active_typing_assist_pipeline_for_auto_replace(
    context: &str,
) -> Vec<lay::config::TypingAssistRuleConfig> {
    let cfg = current_config();
    let safety = cfg.active_correction_safety();
    lay::typing_context::typing_assist_pipeline_for_context(
        cfg.auto_replace,
        safety,
        &cfg.typing_assist_pipeline,
        context,
    )
}
