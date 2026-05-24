use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    build_choice_prompt, parse_choice, prompt_safe, Choice, LlmRuntimeConfig, ANTHROPIC_VERSION,
    DEFAULT_MODEL, KEEP_ALIVE,
};

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: String,
    stream: bool,
    raw: bool,
    keep_alive: &'a str,
    options: OllamaOptions<'a>,
}

#[derive(Serialize)]
struct OllamaOptions<'a> {
    temperature: f32,
    top_p: f32,
    num_predict: i32,
    stop: &'a [&'a str],
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Deserialize)]
pub(crate) struct OpenAiChatResponse {
    pub(crate) choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
pub(crate) struct OpenAiChoice {
    pub(crate) message: OpenAiMessage,
}

#[derive(Deserialize)]
pub(crate) struct OpenAiMessage {
    pub(crate) content: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct AnthropicResponse {
    pub(crate) content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
pub(crate) struct AnthropicContent {
    #[serde(rename = "type")]
    pub(crate) kind: String,
    pub(crate) text: Option<String>,
}

pub(super) fn choose_candidate_ollama(
    original: &str,
    converted: &str,
    cfg: &LlmRuntimeConfig,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    let stop = ["\n"];
    let prompt = build_choice_prompt(original, converted);

    let req = OllamaRequest {
        model: &cfg.model,
        prompt,
        stream: false,
        raw: true,
        keep_alive: KEEP_ALIVE,
        options: OllamaOptions {
            temperature: 0.0,
            top_p: 0.9,
            num_predict: 2,
            stop: &stop,
        },
    };

    #[cfg(not(test))]
    crate::stats::record_llm_call();
    let resp: OllamaResponse = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(cfg.timeout_secs))
        .build()
        .post(&cfg.ollama_url)
        .send_json(serde_json::to_value(&req)?)?
        .into_json()?;

    Ok(parse_choice(&resp.response))
}

pub(super) fn choose_candidate_openai(
    original: &str,
    converted: &str,
    cfg: &LlmRuntimeConfig,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    let api_key = std::env::var("LAY_OPENAI_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .map_err(|_| "LAY_OPENAI_API_KEY or OPENAI_API_KEY is required")?;
    if cfg.model == DEFAULT_MODEL {
        return Err("set LAY_MODEL or llm_model for OpenAI backend".into());
    }

    let body = json!({
        "model": cfg.model,
        "messages": [
            {
                "role": "system",
                "content": "Choose the more natural text. One option may be typed in the wrong keyboard layout. Answer only A or B."
            },
            {
                "role": "user",
                "content": format!("A {}\nB {}", prompt_safe(original), prompt_safe(converted))
            }
        ],
        "temperature": 0,
        "max_completion_tokens": 2
    });

    #[cfg(not(test))]
    crate::stats::record_llm_call();
    let resp: OpenAiChatResponse = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(cfg.timeout_secs))
        .build()
        .post(&cfg.openai_url)
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("Content-Type", "application/json")
        .send_json(body)?
        .into_json()?;

    Ok(resp
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_deref())
        .and_then(parse_choice))
}

pub(super) fn choose_candidate_anthropic(
    original: &str,
    converted: &str,
    cfg: &LlmRuntimeConfig,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    let api_key = std::env::var("LAY_ANTHROPIC_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .map_err(|_| "LAY_ANTHROPIC_API_KEY or ANTHROPIC_API_KEY is required")?;
    if cfg.model == DEFAULT_MODEL {
        return Err("set LAY_MODEL or llm_model for Anthropic backend".into());
    }

    let body = json!({
        "model": cfg.model,
        "max_tokens": 2,
        "temperature": 0,
        "system": "Choose the more natural text. One option may be typed in the wrong keyboard layout. Answer only A or B.",
        "messages": [
            {
                "role": "user",
                "content": format!("A {}\nB {}", prompt_safe(original), prompt_safe(converted))
            }
        ]
    });

    #[cfg(not(test))]
    crate::stats::record_llm_call();
    let resp: AnthropicResponse = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(cfg.timeout_secs))
        .build()
        .post(&cfg.anthropic_url)
        .set("x-api-key", &api_key)
        .set("anthropic-version", ANTHROPIC_VERSION)
        .set("Content-Type", "application/json")
        .send_json(body)?
        .into_json()?;

    Ok(resp
        .content
        .iter()
        .filter(|part| part.kind == "text")
        .filter_map(|part| part.text.as_deref())
        .find_map(parse_choice))
}
