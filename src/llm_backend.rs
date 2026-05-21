//! Optional model backends for A/B candidate voting.
//!
//! This module does not generate corrections. It only asks a configured model
//! to vote between already-built candidates.

#[cfg(feature = "direct-llm")]
use llama_cpp::{
    standard_sampler::StandardSampler, LlamaModel, LlamaParams, LlamaSession, SessionParams,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
#[cfg(feature = "direct-llm")]
use std::collections::HashMap;
#[cfg(feature = "direct-llm")]
use std::path::{Path, PathBuf};
#[cfg(feature = "direct-llm")]
use std::sync::Mutex;

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

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    prompt: String,
    stream: bool,
    raw: bool,
    keep_alive: &'a str,
    options: Options<'a>,
}

#[derive(Serialize)]
struct Options<'a> {
    temperature: f32,
    top_p: f32,
    num_predict: i32,
    stop: &'a [&'a str],
}

#[derive(Deserialize)]
struct Response {
    response: String,
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

#[derive(Deserialize)]
#[cfg(feature = "direct-llm")]
struct OllamaManifest {
    layers: Vec<OllamaLayer>,
}

#[derive(Deserialize)]
#[cfg(feature = "direct-llm")]
struct OllamaLayer {
    #[serde(rename = "mediaType")]
    media_type: String,
    digest: String,
}

pub(crate) fn choose_candidate(
    original: &str,
    converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    if original == converted {
        return Ok(Some(Choice::Original));
    }

    match llm_runtime_config().backend.as_str() {
        "direct" | "gguf" | "llama.cpp" => choose_candidate_direct(original, converted),
        "ollama" | "http" => choose_candidate_ollama(original, converted),
        "openai" | "openai-chat" => choose_candidate_openai(original, converted),
        "anthropic" | "claude" => choose_candidate_anthropic(original, converted),
        "off" | "none" | "disabled" => Ok(None),
        _ => choose_candidate_direct(original, converted),
    }
}

pub fn warm_up() -> Result<(), Box<dyn std::error::Error>> {
    match llm_runtime_config().backend.as_str() {
        "direct" | "gguf" | "llama.cpp" => warm_up_direct(),
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

fn choose_candidate_ollama(
    original: &str,
    converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    let cfg = llm_runtime_config();
    let stop = ["\n"];
    let prompt = build_choice_prompt(original, converted);

    let req = Request {
        model: &cfg.model,
        prompt,
        stream: false,
        raw: true,
        keep_alive: KEEP_ALIVE,
        options: Options {
            temperature: 0.0,
            top_p: 0.9,
            num_predict: 2,
            stop: &stop,
        },
    };

    #[cfg(not(test))]
    crate::stats::record_llm_call();
    let resp: Response = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(cfg.timeout_secs))
        .build()
        .post(&cfg.ollama_url)
        .send_json(serde_json::to_value(&req)?)?
        .into_json()?;

    Ok(parse_choice(&resp.response))
}

fn choose_candidate_openai(
    original: &str,
    converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    let cfg = llm_runtime_config();
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

fn choose_candidate_anthropic(
    original: &str,
    converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    let cfg = llm_runtime_config();
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

fn choose_candidate_direct(
    original: &str,
    converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    choose_candidate_direct_impl(original, converted)
}

#[cfg(not(feature = "direct-llm"))]
fn warm_up_direct() -> Result<(), Box<dyn std::error::Error>> {
    Err("direct GGUF support is not compiled; rebuild with --features direct-llm".into())
}

#[cfg(feature = "direct-llm")]
fn warm_up_direct() -> Result<(), Box<dyn std::error::Error>> {
    direct_llm()
        .ok_or_else(|| "direct GGUF model not available".into())
        .map(|_| ())
}

#[cfg(not(feature = "direct-llm"))]
fn choose_candidate_direct_impl(
    _original: &str,
    _converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    Ok(None)
}

#[cfg(feature = "direct-llm")]
fn choose_candidate_direct_impl(
    original: &str,
    converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    let Some(model) = direct_llm() else {
        return Ok(None);
    };
    let mut model = model.lock().map_err(|_| "direct llm mutex poisoned")?;
    Ok(model.choose(original, converted))
}

#[cfg(feature = "direct-llm")]
struct DirectLlm {
    session: LlamaSession,
    cache: HashMap<(String, String), Choice>,
}

#[cfg(feature = "direct-llm")]
impl DirectLlm {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = direct_model_path().ok_or("direct GGUF model not found")?;
        let model = LlamaModel::load_from_file(path, LlamaParams::default())?;
        let mut session = model.create_session(direct_session_params())?;
        session.advance_context(CHOICE_PROMPT_PREFIX)?;
        Ok(Self {
            session,
            cache: HashMap::new(),
        })
    }

    fn choose(&mut self, original: &str, converted: &str) -> Option<Choice> {
        let cache_key = (original.to_string(), converted.to_string());
        if let Some(choice) = self.cache.get(&cache_key) {
            return Some(*choice);
        }

        #[cfg(not(test))]
        crate::stats::record_llm_call();
        let prompt = build_choice_prompt(original, converted);
        self.session.set_context(prompt).ok()?;

        let mut out = String::new();
        let completions = self
            .session
            .start_completing_with(StandardSampler::new_greedy(), 1)
            .ok()?
            .into_strings();
        for piece in completions {
            out.push_str(&piece);
            if let Some(choice) = parse_choice(&out) {
                self.remember_choice(cache_key, choice);
                return Some(choice);
            }
            if out.contains('\n') || out.chars().count() >= 16 {
                break;
            }
        }

        let choice = parse_choice(&out)?;
        self.remember_choice(cache_key, choice);
        Some(choice)
    }

    fn remember_choice(&mut self, key: (String, String), choice: Choice) {
        if self.cache.len() >= 512 {
            self.cache.clear();
        }
        self.cache.insert(key, choice);
    }
}

#[cfg(feature = "direct-llm")]
fn direct_session_params() -> SessionParams {
    let threads = std::env::var("LAY_LLM_THREADS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|threads| *threads > 0)
        .unwrap_or(4);
    SessionParams {
        n_ctx: 128,
        n_batch: 128,
        n_threads: threads,
        n_threads_batch: threads,
        seed: 1,
        ..SessionParams::default()
    }
}

#[cfg(feature = "direct-llm")]
fn direct_llm() -> Option<&'static Mutex<DirectLlm>> {
    static DIRECT_LLM: std::sync::OnceLock<Result<Mutex<DirectLlm>, String>> =
        std::sync::OnceLock::new();
    DIRECT_LLM
        .get_or_init(|| {
            DirectLlm::load()
                .map(Mutex::new)
                .map_err(|err| err.to_string())
        })
        .as_ref()
        .ok()
}

#[cfg(feature = "direct-llm")]
fn direct_model_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_GGUF_MODEL").map(PathBuf::from) {
        if is_gguf_file(&path) {
            return Some(path);
        }
    }

    let model = llm_runtime_config().model;
    ollama_model_roots()
        .into_iter()
        .find_map(|root| ollama_manifest_model_path(&root, &model))
}

#[cfg(feature = "direct-llm")]
fn ollama_model_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(root) = std::env::var_os("OLLAMA_MODELS").map(PathBuf::from) {
        roots.push(root);
    }
    roots.push(PathBuf::from("/usr/share/ollama/.ollama/models"));
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        roots.push(home.join(".ollama/models"));
    }
    roots
}

#[cfg(feature = "direct-llm")]
fn ollama_manifest_model_path(root: &Path, model: &str) -> Option<PathBuf> {
    let manifest_path = root
        .join("manifests")
        .join(ollama_manifest_relative_path(model)?);
    let manifest = std::fs::read_to_string(manifest_path).ok()?;
    let manifest: OllamaManifest = serde_json::from_str(&manifest).ok()?;
    let layer = manifest
        .layers
        .into_iter()
        .find(|layer| layer.media_type == "application/vnd.ollama.image.model")?;
    let digest = layer.digest.strip_prefix("sha256:")?;
    let path = root.join("blobs").join(format!("sha256-{digest}"));
    is_gguf_file(&path).then_some(path)
}

#[cfg(feature = "direct-llm")]
fn ollama_manifest_relative_path(model: &str) -> Option<PathBuf> {
    let (name, tag) = model.rsplit_once(':').unwrap_or((model, "latest"));
    if name.is_empty() || tag.is_empty() {
        return None;
    }

    let mut path = PathBuf::new();
    if name.contains('/') {
        for part in name.split('/') {
            if part.is_empty() {
                return None;
            }
            path.push(part);
        }
    } else {
        path.push("registry.ollama.ai");
        path.push("library");
        path.push(name);
    }
    path.push(tag);
    Some(path)
}

#[cfg(feature = "direct-llm")]
fn is_gguf_file(path: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut magic = [0u8; 4];
    use std::io::Read;
    file.read_exact(&mut magic).is_ok() && magic == *b"GGUF"
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
