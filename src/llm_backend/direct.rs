#[cfg(not(feature = "direct-llm"))]
use super::Choice;
#[cfg(feature = "direct-llm")]
use super::{build_choice_prompt, llm_runtime_config, parse_choice, Choice, CHOICE_PROMPT_PREFIX};

#[cfg(feature = "direct-llm")]
use llama_cpp::{
    standard_sampler::StandardSampler, LlamaModel, LlamaParams, LlamaSession, SessionParams,
};
#[cfg(feature = "direct-llm")]
use serde::Deserialize;
#[cfg(feature = "direct-llm")]
use std::collections::HashMap;
#[cfg(feature = "direct-llm")]
use std::path::{Path, PathBuf};
#[cfg(feature = "direct-llm")]
use std::sync::Mutex;

pub(super) fn choose_candidate_direct(
    original: &str,
    converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    choose_candidate_direct_impl(original, converted)
}

#[cfg(not(feature = "direct-llm"))]
pub(super) fn warm_up_direct() -> Result<(), Box<dyn std::error::Error>> {
    Err("direct GGUF support is not compiled; rebuild with --features direct-llm".into())
}

#[cfg(feature = "direct-llm")]
pub(super) fn warm_up_direct() -> Result<(), Box<dyn std::error::Error>> {
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
