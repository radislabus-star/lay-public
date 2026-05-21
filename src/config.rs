//! Shared runtime configuration.
//!
//! The daemon, GNOME tray and future desktop frontends must use one config
//! schema. Keep the schema here instead of duplicating it in desktop-specific
//! adapters.

use crate::desktop::{resolve_layout_backend, LayoutBackend};
use crate::text_backend::TextBackendPreference;

pub const CONFIG_PATH: &str = ".config/lay/config.json";

pub const DEFAULT_TYPING_ASSIST_RULES: [(&str, i32); 20] = [
    ("moved_prefix_pair", 10),
    ("split_word_pair", 20),
    ("visual_b", 30),
    ("personal_phrase", 40),
    ("personal_token", 50),
    ("duplicate_layout_prefix", 60),
    ("mixed_script_layout", 70),
    ("layout_technical", 80),
    ("layout_ru_to_en", 90),
    ("layout_en_to_ru", 100),
    ("cyrillic_case", 110),
    ("hard_sign", 120),
    ("adjacent_transposition", 130),
    ("repeated_letter", 140),
    ("single_letter_substitution", 150),
    ("verb_ending", 160),
    ("vowel_confusion", 170),
    ("extra_letters", 180),
    ("missing_letter", 190),
    ("glued_phrase", 200),
];

pub const LAYOUT_ONLY_TYPING_ASSIST_RULES: &[&str] = &[
    "duplicate_layout_prefix",
    "mixed_script_layout",
    "layout_technical",
    "layout_ru_to_en",
    "layout_en_to_ru",
];

const LIVE_AUTO_REPLACE_DISABLED_RULES: &[&str] = &[
    "layout_en_to_ru",
    "single_letter_substitution",
    "verb_ending",
    "vowel_confusion",
    "extra_letters",
];
const STRICT_CORRECTION_DISABLED_RULES: &[&str] = &[
    "repeated_letter",
    "single_letter_substitution",
    "verb_ending",
    "vowel_confusion",
    "extra_letters",
    "missing_letter",
    "glued_phrase",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionSafety {
    Strict,
    Normal,
    Experimental,
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
    /// Text edit backend: uinput | ime | auto.
    pub text_backend: String,
    /// Trigger: double-* | caps-lock | single-*.
    pub trigger: String,
    /// Optional direct RU/EN hotkeys, independent from the correction trigger.
    pub force_layout_hotkeys: bool,
    pub force_ru_key: String,
    pub force_en_key: String,
    /// Optional Shift tap-count scope: 2/3/4 taps => 1/2/3 words.
    pub multi_tap_scope: bool,
    pub multi_tap_max_taps: u8,
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
    /// Typing-assist policy: strict | normal | experimental.
    pub correction_safety: String,
    /// Optional correction attempt on Enter before submitting/sending text.
    pub enter_autocorrect: bool,
    /// Keep active layout aligned to typing-assist result.
    pub auto_switch_layout: bool,
    /// Use LEM arbiter for a two-word smart tail.
    pub lem_2_words: bool,
    /// Use LEM arbiter for three or more smart-tail words.
    pub lem_3_words: bool,
    /// Optional model arbiter backend: off | direct | ollama | openai | anthropic.
    pub llm_backend: String,
    /// Optional model name for configured backend. API keys stay in env.
    pub llm_model: String,
    /// Optional custom Ollama generate endpoint.
    pub llm_ollama_url: String,
    /// Optional custom OpenAI-compatible chat completions endpoint.
    pub llm_openai_url: String,
    /// Optional custom Anthropic messages endpoint.
    pub llm_anthropic_url: String,
    /// Model HTTP timeout in seconds.
    pub llm_timeout_secs: u64,
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
            text_backend: "uinput".into(),
            trigger: "double-lshift".into(),
            force_layout_hotkeys: false,
            force_ru_key: "single-rctrl".into(),
            force_en_key: "single-ralt".into(),
            multi_tap_scope: false,
            multi_tap_max_taps: 4,
            tap_max_ms: 200,
            shift_window_ms: 250,
            debounce_ms: 50,
            replace_words: 1,
            auto_replace: false,
            typing_assist: false,
            correction_safety: "normal".into(),
            enter_autocorrect: false,
            auto_switch_layout: true,
            lem_2_words: true,
            lem_3_words: true,
            llm_backend: "off".into(),
            llm_model: "smollm:135m".into(),
            llm_ollama_url: "http://localhost:11434/api/generate".into(),
            llm_openai_url: "https://api.openai.com/v1/chat/completions".into(),
            llm_anthropic_url: "https://api.anthropic.com/v1/messages".into(),
            llm_timeout_secs: 3,
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

    pub fn active_multi_tap_max_taps(&self) -> u8 {
        self.multi_tap_max_taps.clamp(2, 4)
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

    pub fn active_text_backend(&self) -> TextBackendPreference {
        TextBackendPreference::parse(&self.text_backend)
    }

    pub fn active_typing_assist_pipeline(&self) -> Vec<TypingAssistRuleConfig> {
        normalize_typing_assist_pipeline(&self.typing_assist_pipeline)
    }

    pub fn active_correction_safety(&self) -> CorrectionSafety {
        match self.correction_safety.trim().to_ascii_lowercase().as_str() {
            "strict" | "safe" | "ultra-safe" | "ultrasafe" => CorrectionSafety::Strict,
            "experimental" | "exp" => CorrectionSafety::Experimental,
            _ => CorrectionSafety::Normal,
        }
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
    typing_assist_pipeline_for_policy(auto_replace, CorrectionSafety::Normal, configured)
}

pub fn typing_assist_pipeline_for_policy(
    auto_replace: bool,
    safety: CorrectionSafety,
    configured: &[TypingAssistRuleConfig],
) -> Vec<TypingAssistRuleConfig> {
    let mut rules = normalize_typing_assist_pipeline(configured);
    if !auto_replace {
        for rule in &mut rules {
            rule.enabled =
                rule.enabled && LAYOUT_ONLY_TYPING_ASSIST_RULES.contains(&rule.id.as_str());
        }
    } else if safety != CorrectionSafety::Experimental {
        for rule in &mut rules {
            if LIVE_AUTO_REPLACE_DISABLED_RULES.contains(&rule.id.as_str()) {
                rule.enabled = false;
            }
        }
    }
    if safety == CorrectionSafety::Strict {
        for rule in &mut rules {
            if STRICT_CORRECTION_DISABLED_RULES.contains(&rule.id.as_str()) {
                rule.enabled = false;
            }
        }
    }
    rules
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
