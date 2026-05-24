//! Optional model backends for A/B candidate voting.
//!
//! This module does not generate corrections. It only asks a configured model
//! to vote between already-built candidates.

#[path = "llm_backend/direct.rs"]
mod direct;
#[path = "llm_backend/http.rs"]
mod http;

#[cfg(test)]
pub(crate) use http::{AnthropicResponse, OpenAiChatResponse};

const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
const OPENAI_CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";
const ANTHROPIC_MESSAGES_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "smollm:135m";
const TIMEOUT_SECS: u64 = 3;
const KEEP_ALIVE: &str = "30m";
const CHOICE_PROMPT_PREFIX: &str = "Choose the more natural text. One option may be typed in the wrong keyboard layout. Answer only A or B.\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Choice {
    Original,
    Converted,
}

#[derive(Debug, Clone)]
struct LlmRuntimeConfig {
    backend: String,
    model: String,
    ollama_url: String,
    openai_url: String,
    anthropic_url: String,
    timeout_secs: u64,
}

pub(crate) fn choose_candidate(
    original: &str,
    converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    if original == converted {
        return Ok(Some(Choice::Original));
    }

    let cfg = llm_runtime_config();
    match cfg.backend.as_str() {
        "direct" | "gguf" | "llama.cpp" => direct::choose_candidate_direct(original, converted),
        "ollama" | "http" => http::choose_candidate_ollama(original, converted, &cfg),
        "openai" | "openai-chat" => http::choose_candidate_openai(original, converted, &cfg),
        "anthropic" | "claude" => http::choose_candidate_anthropic(original, converted, &cfg),
        "off" | "none" | "disabled" => Ok(None),
        _ => direct::choose_candidate_direct(original, converted),
    }
}

pub fn warm_up() -> Result<(), Box<dyn std::error::Error>> {
    match llm_runtime_config().backend.as_str() {
        "direct" | "gguf" | "llama.cpp" => direct::warm_up_direct(),
        "off" | "none" | "disabled" => Ok(()),
        _ => choose_candidate("A", "B").map(|_| ()),
    }
}

pub fn model_backend_enabled() -> bool {
    let backend = llm_runtime_config().backend;
    !matches!(backend.as_str(), "off" | "none" | "disabled")
}

fn llm_runtime_config() -> LlmRuntimeConfig {
    let cfg = crate::config::LayConfig::load();
    LlmRuntimeConfig {
        backend: env_or_config("LAY_LLM_BACKEND", &cfg.llm_backend, "off").to_ascii_lowercase(),
        model: env_or_config("LAY_MODEL", &cfg.llm_model, DEFAULT_MODEL),
        ollama_url: env_or_config("LAY_OLLAMA_URL", &cfg.llm_ollama_url, OLLAMA_URL),
        openai_url: env_or_config("LAY_OPENAI_URL", &cfg.llm_openai_url, OPENAI_CHAT_URL),
        anthropic_url: env_or_config(
            "LAY_ANTHROPIC_URL",
            &cfg.llm_anthropic_url,
            ANTHROPIC_MESSAGES_URL,
        ),
        timeout_secs: std::env::var("LAY_LLM_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .or_else(|| (cfg.llm_timeout_secs > 0).then_some(cfg.llm_timeout_secs))
            .unwrap_or(TIMEOUT_SECS),
    }
}

fn env_or_config(env_key: &str, configured: &str, default: &str) -> String {
    std::env::var(env_key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| (!configured.trim().is_empty()).then(|| configured.to_string()))
        .unwrap_or_else(|| default.to_string())
}

fn build_choice_prompt(original: &str, converted: &str) -> String {
    format!(
        "{CHOICE_PROMPT_PREFIX}A {} B {} =>",
        prompt_safe(original),
        prompt_safe(converted),
    )
}

fn prompt_safe(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn parse_choice(response: &str) -> Option<Choice> {
    let trimmed = response.trim();

    for token in trimmed.split(|c: char| !c.is_ascii_alphabetic()) {
        if token.eq_ignore_ascii_case("A") {
            return Some(Choice::Original);
        }
        if token.eq_ignore_ascii_case("B") {
            return Some(Choice::Converted);
        }
    }

    None
}
