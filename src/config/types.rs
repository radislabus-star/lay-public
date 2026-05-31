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

impl CorrectionSafety {
    pub(crate) fn allows_typing_rule_requirement(
        self,
        required: crate::typing_rule_graph::TypingRuleRequiredSafety,
    ) -> bool {
        use crate::typing_rule_graph::TypingRuleRequiredSafety;

        match required {
            TypingRuleRequiredSafety::Strict => true,
            TypingRuleRequiredSafety::Normal => self != CorrectionSafety::Strict,
            TypingRuleRequiredSafety::Experimental => self == CorrectionSafety::Experimental,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct LayConfig {
    pub mode: String,
    pub correction_engine: Option<String>,
    pub layout_backend: String,
    pub text_backend: String,
    pub trigger: String,
    pub force_layout_hotkeys: bool,
    pub force_ru_key: String,
    pub force_en_key: String,
    pub multi_tap_scope: bool,
    pub multi_tap_max_taps: u8,
    pub tap_max_ms: u64,
    pub shift_window_ms: u64,
    pub debounce_ms: u64,
    pub replace_words: usize,
    pub auto_replace: bool,
    pub typing_assist: bool,
    pub correction_safety: String,
    pub enter_autocorrect: bool,
    pub auto_switch_layout: bool,
    pub lem_2_words: bool,
    pub lem_3_words: bool,
    pub llm_backend: String,
    pub llm_model: String,
    pub llm_ollama_url: String,
    pub llm_openai_url: String,
    pub llm_anthropic_url: String,
    pub llm_timeout_secs: u64,
    #[serde(default = "super::defaults::default_typing_assist_pipeline")]
    pub typing_assist_pipeline: Vec<TypingAssistRuleConfig>,
    pub learning_log: bool,
}
