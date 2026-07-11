use super::{LayConfig, TypingAssistRuleConfig};

pub fn default_typing_assist_rules() -> Vec<(&'static str, i32)> {
    crate::typing_rule_graph::typing_rule_definitions()
        .iter()
        .filter_map(|rule| rule.default_priority.map(|priority| (rule.id, priority)))
        .collect()
}

pub fn default_typing_assist_pipeline() -> Vec<TypingAssistRuleConfig> {
    default_typing_assist_rules()
        .into_iter()
        .map(|(id, priority)| TypingAssistRuleConfig {
            id: id.to_string(),
            enabled: true,
            priority,
        })
        .collect()
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
            typing_assist_words: 2,
            auto_replace: false,
            typing_assist: false,
            correction_safety: "normal".into(),
            enter_autocorrect: false,
            auto_switch_layout: true,
            lem_enabled: true,
            lem_2_words: true,
            lem_3_words: true,
            lem_weight_percent: 80,
            nanda_l2_weight_percent: 20,
            nanda_l3_weight_percent: 8,
            llm_backend: "off".into(),
            llm_model: "smollm:135m".into(),
            llm_ollama_url: "http://localhost:11434/api/generate".into(),
            llm_openai_url: "https://api.openai.com/v1/chat/completions".into(),
            llm_anthropic_url: "https://api.anthropic.com/v1/messages".into(),
            llm_timeout_secs: 3,
            typing_assist_pipeline: default_typing_assist_pipeline(),
            debug_action_log: false,
            learning_log: false,
            nanda_autocorrect: false,
            nanda_trace: false,
            nanda_trace_text: false,
            nanda_precognition: false,
            ime_bracket_candidates: false,
            llmwave_shadow: true,
            llmwave_apply: true,
            nanda_l2_phase_shadow: true,
            nanda_l2_phase_apply: true,
            nanda_l3_phase_shadow: true,
        }
    }
}
