use lay::config::{CorrectionEngine, LayConfig};
use lay::desktop::LayoutBackend;
use lay::text_backend::TextBackendPreference;
use std::sync::OnceLock;
#[cfg(not(test))]
use std::{
    sync::Mutex,
    time::{Duration, Instant, SystemTime},
};

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
        sync_hot_runtime(&config);
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
            sync_hot_runtime(&cache.config);
            return cache.config.clone();
        }
        cache.checked_at = Instant::now();
        let modified = config_modified_at();
        if modified != cache.modified {
            cache.config = LayConfig::load();
            cache.modified = modified;
        }
        sync_hot_runtime(&cache.config);
        cache.config.clone()
    }
}

fn sync_hot_runtime(config: &LayConfig) {
    sync_lem_runtime(config);
    sync_hot_field_runtime(config);
}

fn sync_lem_runtime(config: &LayConfig) {
    lay::lem::set_runtime_enabled(config.lem_enabled && config.active_lem_weight() > 0.0);
}

fn sync_hot_field_runtime(config: &LayConfig) {
    lay::hot_field::set_process_policy(daemon_hot_field_policy(config));
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
    let config_enabled = current_config().enter_autocorrect;
    let env_value = std::env::var(ENTER_AUTOCORRECT_EXPERIMENT_ENV).ok();
    active_enter_autocorrect_from_env(config_enabled, env_value.as_deref())
}

pub(super) fn active_enter_autocorrect_from_env(
    config_enabled: bool,
    env_value: Option<&str>,
) -> bool {
    config_enabled
        && env_value.map_or(true, |value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
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
    let cfg = current_config();
    daemon_nanda_trace_active(&cfg)
}

pub(super) fn active_nanda_precognition() -> bool {
    let cfg = current_config();
    daemon_precognition_trace_active(&cfg)
}

fn daemon_nanda_trace_active(cfg: &LayConfig) -> bool {
    // Runtime action logs remain under debug_action_log. Full NANDA wave trace
    // is heavier: in IME mode it belongs to lay-ibus-engine, not the daemon.
    cfg.debug_action_log && daemon_hot_field_policy(cfg).allows_full_nanda_authority()
}

fn daemon_precognition_trace_active(cfg: &LayConfig) -> bool {
    // IME owns live precognition display and timing. The daemon may keep this
    // trace path only for non-IME experiments, otherwise it loads a second
    // NANDA wave route on key/space events.
    cfg.nanda_precognition && daemon_nanda_trace_active(cfg)
}

#[cfg(not(test))]
pub(super) fn active_nanda_autocorrect() -> bool {
    let cfg = current_config();
    daemon_nanda_autocorrect_active(&cfg)
}

fn daemon_nanda_autocorrect_active(cfg: &LayConfig) -> bool {
    // The correction core selects compact lexical-phase material when the
    // process policy is FieldSnapshotOnly. Enabling autocorrect here no longer
    // grants the daemon access to full reference dictionaries or wave traces.
    cfg.nanda_autocorrect
}

fn daemon_hot_field_policy(cfg: &LayConfig) -> lay::hot_field::HotFieldPolicy {
    lay::hot_field::HotFieldPolicy::daemon_for_text_backend(cfg.active_text_backend())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ime_backend_does_not_enable_daemon_precognition_trace() {
        let cfg = LayConfig {
            text_backend: "ime".to_string(),
            debug_action_log: true,
            nanda_precognition: true,
            ..LayConfig::default()
        };

        assert!(!daemon_precognition_trace_active(&cfg));
    }

    #[test]
    fn ime_backend_does_not_enable_heavy_daemon_wave_trace() {
        let cfg = LayConfig {
            text_backend: "ime".to_string(),
            debug_action_log: true,
            ..LayConfig::default()
        };

        assert!(!daemon_nanda_trace_active(&cfg));
    }

    #[test]
    fn ime_backend_uses_compact_nanda_without_full_reference_authority() {
        let cfg = LayConfig {
            text_backend: "ime".to_string(),
            nanda_autocorrect: true,
            ..LayConfig::default()
        };

        assert!(daemon_nanda_autocorrect_active(&cfg));
        assert!(!daemon_hot_field_policy(&cfg).allows_full_nanda_authority());
    }

    #[test]
    fn auto_backend_uses_compact_nanda_without_full_reference_authority() {
        let cfg = LayConfig {
            text_backend: "auto".to_string(),
            nanda_autocorrect: true,
            ..LayConfig::default()
        };

        assert!(daemon_nanda_autocorrect_active(&cfg));
        assert!(!daemon_hot_field_policy(&cfg).allows_full_nanda_authority());
    }

    #[test]
    fn uinput_backend_can_keep_daemon_precognition_trace_explicitly() {
        let cfg = LayConfig {
            text_backend: "uinput".to_string(),
            debug_action_log: true,
            nanda_precognition: true,
            ..LayConfig::default()
        };

        assert!(daemon_precognition_trace_active(&cfg));
    }

    #[test]
    fn uinput_backend_can_keep_heavy_daemon_wave_trace_explicitly() {
        let cfg = LayConfig {
            text_backend: "uinput".to_string(),
            debug_action_log: true,
            ..LayConfig::default()
        };

        assert!(daemon_nanda_trace_active(&cfg));
    }

    #[test]
    fn uinput_backend_can_keep_full_daemon_nanda_autocorrect_explicitly() {
        let cfg = LayConfig {
            text_backend: "uinput".to_string(),
            nanda_autocorrect: true,
            ..LayConfig::default()
        };

        assert!(daemon_hot_field_policy(&cfg).allows_full_nanda_authority());
    }
}
