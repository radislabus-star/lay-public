//! Shared runtime configuration.
//!
//! The daemon, GNOME tray and future desktop frontends must use one config
//! schema. Keep the schema here instead of duplicating it in desktop-specific
//! adapters.

use crate::desktop::{resolve_layout_backend, LayoutBackend};

pub const CONFIG_PATH: &str = ".config/lay/config.json";

pub const DEFAULT_TYPING_ASSIST_RULES: [(&str, i32); 19] = [
    ("moved_prefix_pair", 10),
    ("split_word_pair", 20),
    ("visual_b", 30),
    ("personal_phrase", 40),
    ("personal_token", 50),
    ("duplicate_layout_prefix", 60),
    ("layout_technical", 70),
    ("layout_ru_to_en", 80),
    ("layout_en_to_ru", 90),
    ("cyrillic_case", 100),
    ("hard_sign", 110),
    ("adjacent_transposition", 120),
    ("repeated_letter", 130),
    ("single_letter_substitution", 140),
    ("verb_ending", 150),
    ("vowel_confusion", 160),
    ("extra_letters", 170),
    ("missing_letter", 180),
    ("glued_phrase", 190),
];

pub const LAYOUT_ONLY_TYPING_ASSIST_RULES: &[&str] = &[
    "duplicate_layout_prefix",
    "layout_technical",
    "layout_ru_to_en",
    "layout_en_to_ru",
];

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TypingAssistRuleConfig {
    pub id: String,
    #[serde(default = "default_rule_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub priority: i32,
}

fn default_rule_enabled() -> bool {
    true
}

pub fn default_typing_assist_pipeline() -> Vec<TypingAssistRuleConfig> {
    DEFAULT_TYPING_ASSIST_RULES
        .iter()
        .map(|(id, priority)| TypingAssistRuleConfig {
            id: (*id).to_string(),
            enabled: true,
            priority: *priority,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionEngine {
    Replay,
    Smart,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct LayConfig {
    /// Legacy field: simple | llm. New UI writes correction_engine instead.
    pub mode: String,
    /// Engine for double-Shift correction: replay | smart.
    pub correction_engine: Option<String>,
    /// Layout backend: auto | gnome | kde | x11.
    pub layout_backend: String,
    /// Trigger: double-* | caps-lock | single-*.
    pub trigger: String,
    /// Maximum duration of each tap in milliseconds.
    pub tap_max_ms: u64,
    /// Window between two taps in milliseconds.
    pub shift_window_ms: u64,
    /// Debounce after correction in milliseconds.
    pub debounce_ms: u64,
    /// How many last words to process: 1..3 independently from engine.
    pub replace_words: usize,
    /// Exact personal auto-replacements after ordinary layout replay.
    pub auto_replace: bool,
    /// Safe typing assistance after Space.
    pub typing_assist: bool,
    /// Keep active layout aligned to typing-assist result.
    pub auto_switch_layout: bool,
    /// Use LEM arbiter for a two-word smart tail.
    pub lem_2_words: bool,
    /// Use LEM arbiter for three or more smart-tail words.
    pub lem_3_words: bool,
    /// Typing-assist rule pipeline: id + enabled + priority.
    #[serde(default = "default_typing_assist_pipeline")]
    pub typing_assist_pipeline: Vec<TypingAssistRuleConfig>,
    /// Local opt-in correction log for future learning.
    pub learning_log: bool,
}

impl Default for LayConfig {
    fn default() -> Self {
        Self {
            mode: "simple".into(),
            correction_engine: None,
            layout_backend: "auto".into(),
            trigger: "double-lshift".into(),
            tap_max_ms: 200,
            shift_window_ms: 250,
            debounce_ms: 50,
            replace_words: 1,
            auto_replace: false,
            typing_assist: false,
            auto_switch_layout: true,
            lem_2_words: true,
            lem_3_words: true,
            typing_assist_pipeline: default_typing_assist_pipeline(),
            learning_log: false,
        }
    }
}

impl LayConfig {
    pub fn load() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = format!("{home}/{CONFIG_PATH}");
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
                eprintln!("[lay] config parse error: {e}, using defaults");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    pub fn active_replace_words(&self) -> usize {
        self.replace_words.clamp(1, 3)
    }

    pub fn active_correction_engine(&self) -> CorrectionEngine {
        match self.correction_engine.as_deref() {
            Some("smart") => CorrectionEngine::Smart,
            Some("replay") => CorrectionEngine::Replay,
            // Compatibility with configs written before correction_engine existed.
            _ if self.mode == "llm" => CorrectionEngine::Smart,
            _ => CorrectionEngine::Replay,
        }
    }

    pub fn active_layout_backend(&self) -> LayoutBackend {
        resolve_layout_backend(
            &self.layout_backend,
            std::env::var("XDG_CURRENT_DESKTOP").ok().as_deref(),
            std::env::var("DESKTOP_SESSION").ok().as_deref(),
            std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        )
    }

    pub fn active_typing_assist_pipeline(&self) -> Vec<TypingAssistRuleConfig> {
        normalize_typing_assist_pipeline(&self.typing_assist_pipeline)
    }

    pub fn lem_enabled_for_scope(&self, word_count: usize) -> bool {
        match word_count {
            0 | 1 => false,
            2 => self.lem_2_words,
            _ => self.lem_3_words,
        }
    }
}

pub fn normalize_typing_assist_pipeline(
    configured: &[TypingAssistRuleConfig],
) -> Vec<TypingAssistRuleConfig> {
    let mut rules = default_typing_assist_pipeline();
    for saved in configured {
        if let Some(rule) = rules.iter_mut().find(|rule| rule.id == saved.id) {
            rule.enabled = saved.enabled;
            if saved.priority > 0 {
                rule.priority = saved.priority;
            }
        }
    }
    rules.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.id.cmp(&b.id)));
    rules
}

pub fn typing_assist_pipeline_for_auto_replace(
    auto_replace: bool,
    configured: &[TypingAssistRuleConfig],
) -> Vec<TypingAssistRuleConfig> {
    let mut rules = normalize_typing_assist_pipeline(configured);
    if !auto_replace {
        for rule in &mut rules {
            rule.enabled =
                rule.enabled && LAYOUT_ONLY_TYPING_ASSIST_RULES.contains(&rule.id.as_str());
        }
    }
    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_preserve_public_runtime_behavior() {
        let cfg = LayConfig::default();
        assert_eq!(cfg.mode, "simple");
        assert_eq!(cfg.active_replace_words(), 1);
        assert_eq!(cfg.active_correction_engine(), CorrectionEngine::Replay);
        assert!(cfg.auto_switch_layout);
        assert!(cfg.lem_enabled_for_scope(2));
        assert!(cfg.lem_enabled_for_scope(3));
        assert_eq!(
            cfg.active_typing_assist_pipeline().len(),
            DEFAULT_TYPING_ASSIST_RULES.len()
        );
    }

    #[test]
    fn legacy_llm_mode_maps_to_smart_only_without_explicit_engine() {
        let legacy = LayConfig {
            mode: "llm".into(),
            ..LayConfig::default()
        };
        let explicit_replay = LayConfig {
            mode: "llm".into(),
            correction_engine: Some("replay".into()),
            ..LayConfig::default()
        };

        assert_eq!(legacy.active_correction_engine(), CorrectionEngine::Smart);
        assert_eq!(
            explicit_replay.active_correction_engine(),
            CorrectionEngine::Replay
        );
    }

    #[test]
    fn auto_replace_off_keeps_layout_only_rules() {
        let pipeline =
            typing_assist_pipeline_for_auto_replace(false, &default_typing_assist_pipeline());
        assert!(pipeline
            .iter()
            .find(|rule| rule.id == "layout_en_to_ru")
            .is_some_and(|rule| rule.enabled));
        assert!(pipeline
            .iter()
            .find(|rule| rule.id == "missing_letter")
            .is_some_and(|rule| !rule.enabled));
    }
}
