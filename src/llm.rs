//! Optional local model arbiter for already-built layout candidates.
//!
//! The daemon builds deterministic candidates first. A configured backend can
//! only vote between those candidates with a short A/B answer; invalid answers
//! are ignored. The default backend is `off`. Direct GGUF loading requires the
//! `direct-llm` feature.

#[cfg(feature = "direct-llm")]
use llama_cpp::{
    standard_sampler::StandardSampler, LlamaModel, LlamaParams, LlamaSession, SessionParams,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "direct-llm")]
use std::collections::HashMap;
use std::collections::HashSet;
#[cfg(feature = "direct-llm")]
use std::path::{Path, PathBuf};
#[cfg(feature = "direct-llm")]
use std::sync::Mutex;
use std::sync::OnceLock;

const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
const DEFAULT_MODEL: &str = "smollm:135m";
const TIMEOUT_SECS: u64 = 3;
const KEEP_ALIVE: &str = "30m";
const RU_HUNSPELL: &str = "/usr/share/hunspell/ru_RU.dic";
const EN_HUNSPELL: &str = "/usr/share/hunspell/en_US.dic";
const EN_WORDS: &str = "/usr/share/dict/words";
const RU_VOWELS: &str = "аеёиоуыэюяАЕЁИОУЫЭЮЯ";
const COMMON_RUSSIAN_WORDS: &[&str] = &[
    "а",
    "в",
    "и",
    "к",
    "о",
    "с",
    "у",
    "я",
    "не",
    "на",
    "по",
    "за",
    "для",
    "это",
    "как",
    "что",
    "где",
    "или",
    "если",
    "тут",
    "там",
    "уже",
    "еще",
    "ещё",
    "надо",
    "можно",
    "нужно",
    "очень",
    "буду",
    "будешь",
    "будет",
    "будем",
    "будете",
    "будут",
];
const CHOICE_PROMPT_PREFIX: &str = "Choose the more natural text. One option may be typed in the wrong keyboard layout. Answer only A or B.\n";
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Original,
    Converted,
}

pub fn convert(text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let direction = crate::dict::detect_direction(text);
    let converted = crate::dict::convert(text, direction);
    choose_candidate(text, &converted).map(|choice| match choice {
        Some(Choice::Original) => text.to_string(),
        Some(Choice::Converted) | None => converted,
    })
}

pub fn convert_hybrid(
    original: &str,
    converted: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(repaired) = repair_mixed_script(original) {
        return Ok(Some(repaired));
    }

    if let Some(protected) = keep_protected_ascii_tokens(original, converted) {
        return Ok(Some(protected));
    }

    if all_tokens_known(original, Lang::Ru) && !all_tokens_known(converted, Lang::En) {
        return Ok(Some(original.to_string()));
    }

    if let Some(tokenwise) = choose_mixed_token_candidate(original, converted, choose_candidate)? {
        return Ok(Some(tokenwise));
    }

    if has_cyrillic(original) && has_latin(original) {
        return Ok(Some(original.to_string()));
    }

    Ok(Some(match choose_candidate(original, converted)? {
        Some(Choice::Original) | None => original.to_string(),
        Some(Choice::Converted) => converted.to_string(),
    }))
}

pub fn choose_token_hybrid(
    original: &str,
    converted: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    choose_token_hybrid_with_chooser(original, converted, choose_candidate)
}

pub fn choose_token_consensus(
    original: &str,
    converted: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    choose_token_consensus_with_chooser(original, converted, choose_candidate)
}

fn choose_token_hybrid_with_chooser<F>(
    original: &str,
    converted: &str,
    _chooser: F,
) -> Result<Option<String>, Box<dyn std::error::Error>>
where
    F: Fn(&str, &str) -> Result<Option<Choice>, Box<dyn std::error::Error>>,
{
    if original == converted {
        return Ok(Some(original.to_string()));
    }

    if let Some(repaired) = repair_mixed_script(original) {
        return Ok(Some(repaired));
    }

    if is_protected_ascii_token(original) {
        return Ok(Some(original.to_string()));
    }

    if let Some(choice) = obvious_token_choice(original, converted) {
        return Ok(Some(match choice {
            Choice::Original => original.to_string(),
            Choice::Converted => converted.to_string(),
        }));
    }

    Ok(Some(original.to_string()))
}

fn choose_token_consensus_with_chooser<F>(
    original: &str,
    converted: &str,
    chooser: F,
) -> Result<Option<String>, Box<dyn std::error::Error>>
where
    F: Fn(&str, &str) -> Result<Option<Choice>, Box<dyn std::error::Error>>,
{
    if original == converted {
        return Ok(Some(original.to_string()));
    }

    if let Some(repaired) = repair_mixed_script(original) {
        return Ok(Some(repaired));
    }

    if is_protected_ascii_token(original) {
        return Ok(Some(original.to_string()));
    }

    let Some(choice) = obvious_token_choice(original, converted) else {
        return Ok(Some(original.to_string()));
    };

    match choice {
        Choice::Original => Ok(Some(original.to_string())),
        Choice::Converted => match chooser(original, converted) {
            Ok(Some(Choice::Converted)) | Err(_) => Ok(Some(converted.to_string())),
            Ok(Some(Choice::Original)) | Ok(None) => Ok(Some(original.to_string())),
        },
    }
}

pub fn warm_up() -> Result<(), Box<dyn std::error::Error>> {
    match configured_llm_backend().as_str() {
        "direct" | "gguf" | "llama.cpp" => warm_up_direct(),
        "off" | "none" | "disabled" => Ok(()),
        _ => choose_candidate("A", "B").map(|_| ()),
    }
}

pub fn repair_mixed_script(text: &str) -> Option<String> {
    if !has_cyrillic(text) || !has_latin(text) {
        return None;
    }

    let mut out = String::with_capacity(text.len());
    let mut token = String::new();
    for ch in text.chars() {
        if ch.is_alphabetic() {
            token.push(ch);
        } else {
            push_repaired_token(&mut out, &token);
            token.clear();
            out.push(ch);
        }
    }
    push_repaired_token(&mut out, &token);

    if out != text {
        Some(out)
    } else {
        None
    }
}

fn choose_candidate(
    original: &str,
    converted: &str,
) -> Result<Option<Choice>, Box<dyn std::error::Error>> {
    if original == converted {
        return Ok(Some(Choice::Original));
    }

    match configured_llm_backend().as_str() {
        "direct" | "gguf" | "llama.cpp" => return choose_candidate_direct(original, converted),
        "ollama" | "http" => {}
        "off" | "none" | "disabled" => return Ok(None),
        _ => return choose_candidate_direct(original, converted),
    }

    choose_candidate_ollama(original, converted)
}

fn configured_llm_backend() -> String {
    std::env::var("LAY_LLM_BACKEND").unwrap_or_else(|_| "off".to_string())
}

pub fn model_backend_enabled() -> bool {
    !matches!(
        configured_llm_backend().as_str(),
        "off" | "none" | "disabled"
    )
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
    let model = std::env::var("LAY_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let stop = ["\n"];
    let prompt = build_choice_prompt(original, converted);

    let req = Request {
        model: &model,
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
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .post(OLLAMA_URL)
        .send_json(serde_json::to_value(&req)?)?
        .into_json()?;

    Ok(parse_choice(&resp.response))
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
    static DIRECT_LLM: OnceLock<Result<Mutex<DirectLlm>, String>> = OnceLock::new();
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

    let model = std::env::var("LAY_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
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

fn keep_protected_ascii_tokens(original: &str, converted: &str) -> Option<String> {
    let original_segments = split_ws_segments(original);
    let converted_segments = split_ws_segments(converted);
    if original_segments.len() != converted_segments.len() {
        return None;
    }

    let mut protected_count = 0;
    let mut converted_count = 0;
    let mut out = String::with_capacity(original.len().max(converted.len()));

    for ((orig, orig_ws), (conv, conv_ws)) in
        original_segments.iter().zip(converted_segments.iter())
    {
        if orig_ws != conv_ws {
            return None;
        }
        if *orig_ws {
            out.push_str(orig);
        } else if is_protected_ascii_token(orig) {
            protected_count += 1;
            out.push_str(orig);
        } else {
            match obvious_token_choice(orig, conv).unwrap_or(Choice::Original) {
                Choice::Original => out.push_str(orig),
                Choice::Converted => {
                    if orig != conv {
                        converted_count += 1;
                    }
                    out.push_str(conv);
                }
            }
        }
    }

    if protected_count > 0 && converted_count > 0 && out != original && out != converted {
        Some(out)
    } else {
        None
    }
}

fn choose_mixed_token_candidate<F>(
    original: &str,
    converted: &str,
    mut chooser: F,
) -> Result<Option<String>, Box<dyn std::error::Error>>
where
    F: FnMut(&str, &str) -> Result<Option<Choice>, Box<dyn std::error::Error>>,
{
    let original_segments = split_ws_segments(original);
    let converted_segments = split_ws_segments(converted);
    if original_segments.len() != converted_segments.len() {
        return Ok(None);
    }

    let mut word_count = 0;
    let mut kept_original = false;
    let mut used_converted = false;
    let mut used_chooser = false;
    let mut out = String::with_capacity(original.len().max(converted.len()));

    for ((orig, orig_ws), (conv, conv_ws)) in
        original_segments.iter().zip(converted_segments.iter())
    {
        if orig_ws != conv_ws {
            return Ok(None);
        }
        if *orig_ws {
            out.push_str(orig);
            continue;
        }

        word_count += 1;
        if orig == conv {
            out.push_str(orig);
            continue;
        }

        let choice = match obvious_token_choice(orig, conv) {
            Some(choice) => Some(choice),
            None => {
                used_chooser = true;
                chooser(orig, conv)?
            }
        };

        match choice {
            Some(Choice::Original) => {
                kept_original = true;
                out.push_str(orig);
            }
            Some(Choice::Converted) => {
                used_converted = true;
                out.push_str(conv);
            }
            None => return Ok(None),
        }
    }

    let deterministic_choice = word_count > 0 && !used_chooser;
    let mixed_choice =
        word_count >= 2 && kept_original && used_converted && out != original && out != converted;
    if deterministic_choice || mixed_choice {
        Ok(Some(out))
    } else {
        Ok(None)
    }
}

fn obvious_token_choice(original: &str, converted: &str) -> Option<Choice> {
    let original_cyr = has_cyrillic(original);
    let original_lat = has_latin(original);
    let converted_cyr = has_cyrillic(converted);
    let converted_lat = has_latin(converted);

    if (original_cyr && original_lat) || (converted_cyr && converted_lat) {
        return Some(Choice::Original);
    }

    if original_cyr && !original_lat && converted_lat && !converted_cyr {
        let original_known = is_known_ru_token(original);
        let converted_known = is_known_en_token(converted);
        if original_known != converted_known {
            return Some(if original_known {
                Choice::Original
            } else {
                Choice::Converted
            });
        }

        let original_ru = crate::quality::score(original, "ru");
        let converted_en = crate::quality::score(converted, "en");
        return obvious_quality_choice(original_ru, converted_en)
            .or_else(|| short_unknown_prefers_original(original, converted));
    }

    if original_lat && !original_cyr && converted_cyr && !converted_lat {
        let original_known = is_known_en_token(original);
        let converted_known = is_known_ru_token(converted);
        if original_known != converted_known {
            return Some(if original_known {
                Choice::Original
            } else {
                Choice::Converted
            });
        }
        if is_single_ascii_letter(original) && is_single_cyrillic_letter(converted) {
            return Some(Choice::Converted);
        }
        if !original_known && is_long_upper_ascii_word(original) {
            let converted_ru = crate::quality::score(converted, "ru");
            return Some(if converted_ru >= 0.7 {
                Choice::Converted
            } else {
                Choice::Original
            });
        }

        let original_en = crate::quality::score(original, "en");
        let converted_ru = crate::quality::score(converted, "ru");
        return obvious_quality_choice(original_en, converted_ru)
            .or_else(|| short_unknown_prefers_original(original, converted));
    }

    None
}

fn short_unknown_prefers_original(original: &str, converted: &str) -> Option<Choice> {
    let original_len = original.chars().filter(|ch| ch.is_alphabetic()).count();
    let converted_len = converted.chars().filter(|ch| ch.is_alphabetic()).count();
    (original_len <= 3 && converted_len <= 3).then_some(Choice::Original)
}

fn obvious_quality_choice(original_score: f32, converted_score: f32) -> Option<Choice> {
    if original_score >= 0.99 && converted_score < 0.7 {
        Some(Choice::Original)
    } else if original_score < 0.7 && converted_score >= 0.99 {
        Some(Choice::Converted)
    } else {
        None
    }
}

fn is_known_ru_token(token: &str) -> bool {
    let Some(word) = normalized_token_core(token, Lang::Ru) else {
        return false;
    };
    is_known_ru_word(&word)
}

fn is_known_en_token(token: &str) -> bool {
    if is_protected_ascii_token(token) {
        return true;
    }
    let Some(word) = normalized_token_core(token, Lang::En) else {
        return false;
    };
    en_dictionary().contains(&word)
}

#[derive(Clone, Copy)]
enum Lang {
    Ru,
    En,
}

fn normalized_token_core(token: &str, lang: Lang) -> Option<String> {
    let word = token
        .trim_matches(|ch: char| !ch.is_alphabetic() && ch != '-')
        .to_lowercase();
    if word.is_empty() {
        return None;
    }

    let valid = match lang {
        Lang::Ru => word.chars().all(|ch| matches!(ch, 'а'..='я' | 'ё' | '-')),
        Lang::En => word.chars().all(|ch| ch.is_ascii_alphabetic() || ch == '-'),
    };
    valid.then_some(word)
}

fn all_tokens_known(text: &str, lang: Lang) -> bool {
    let mut found = false;
    for (segment, is_ws) in split_ws_segments(text) {
        if is_ws {
            continue;
        }
        let Some(word) = normalized_token_core(segment, lang) else {
            return false;
        };
        found = true;
        let known = match lang {
            Lang::Ru => is_known_ru_word(&word),
            Lang::En => en_dictionary().contains(&word),
        };
        if !known {
            return false;
        }
    }
    found
}

fn is_known_ru_word(word: &str) -> bool {
    if ru_dictionary().contains(word) {
        return true;
    }
    let len = word.chars().count();
    if len < 4 {
        return false;
    }

    const SUFFIXES: &[&str] = &[
        "ыми", "ими", "ами", "ями", "ого", "его", "ому", "ему", "ах", "ях", "ам", "ям", "ом", "ем",
        "ой", "ей", "ый", "ий", "ая", "яя", "ое", "ее", "ые", "ие", "а", "я", "у", "ю", "е", "ы",
        "и",
    ];
    SUFFIXES.iter().any(|suffix| {
        let Some(stem) = word.strip_suffix(suffix) else {
            return false;
        };
        stem.chars().count() >= 3 && ru_dictionary().contains(stem)
    }) || is_known_ru_verb_form(word)
}

fn is_known_ru_verb_form(word: &str) -> bool {
    for (ending, lemmas) in [("айте", &["ать"][..]), ("ай", &["ать"][..])] {
        let Some(stem) = word.strip_suffix(ending) else {
            continue;
        };
        if stem.chars().count() >= 3
            && lemmas
                .iter()
                .any(|lemma_suffix| ru_dictionary().contains(&format!("{stem}{lemma_suffix}")))
        {
            return true;
        }
    }
    false
}

fn ru_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(RU_HUNSPELL, Lang::Ru).unwrap_or_default();
        words.extend(COMMON_RUSSIAN_WORDS.iter().copied().map(str::to_string));
        #[cfg(test)]
        words.extend(
            ["главное", "главная", "дом"]
                .into_iter()
                .map(str::to_string),
        );
        words
    })
}

fn en_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(EN_HUNSPELL, Lang::En).unwrap_or_default();
        if words.is_empty() {
            words.extend(load_plain_words(EN_WORDS, Lang::En).unwrap_or_default());
        }
        words
    })
}

fn load_hunspell_words(path: &str, lang: Lang) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .skip(1)
        .filter_map(|line| normalized_token_core(line.split('/').next().unwrap_or(""), lang))
        .collect())
}

fn load_plain_words(path: &str, lang: Lang) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .filter_map(|line| normalized_token_core(line, lang))
        .collect())
}

fn split_ws_segments(text: &str) -> Vec<(&str, bool)> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut current_ws: Option<bool> = None;

    for (idx, ch) in text.char_indices() {
        let ws = ch.is_whitespace();
        match current_ws {
            Some(prev) if prev != ws => {
                segments.push((&text[start..idx], prev));
                start = idx;
                current_ws = Some(ws);
            }
            None => current_ws = Some(ws),
            _ => {}
        }
    }

    if let Some(ws) = current_ws {
        segments.push((&text[start..], ws));
    }
    segments
}

fn is_protected_ascii_token(token: &str) -> bool {
    token.chars().any(|ch| ch.is_ascii_alphabetic())
        && token.is_ascii()
        && (token.contains('.')
            || token.contains('@')
            || token.contains("://")
            || token.contains('/')
            || token.contains('\\')
            || is_upper_ascii_acronym(token)
            || is_mixed_case_ascii_brand(token))
}

fn push_repaired_token(out: &mut String, token: &str) {
    if token.is_empty() {
        return;
    }

    let token_has_cyr = has_cyrillic(token);
    let token_has_lat = has_latin(token);
    if token_has_cyr && token_has_lat {
        if let Some(ascii) = repair_mixed_ascii_token(token) {
            out.push_str(&ascii);
        } else if let Some(russian) = repair_mixed_russian_token(token) {
            out.push_str(&russian);
        } else {
            out.push_str(token);
        }
    } else if token_has_lat && should_convert_latin_island(token) {
        out.push_str(&crate::dict::convert(token, crate::dict::Direction::Us2Ru));
    } else {
        out.push_str(token);
    }
}

fn repair_mixed_ascii_token(token: &str) -> Option<String> {
    if !starts_with_latin_letter(token) {
        return None;
    }

    let candidate: String = token
        .chars()
        .map(|ch| {
            if has_cyrillic_char(ch) {
                crate::dict::convert(&ch.to_string(), crate::dict::Direction::Ru2Us)
            } else {
                ch.to_string()
            }
        })
        .collect();

    (candidate != token && is_protected_ascii_token(&candidate)).then_some(candidate)
}

fn starts_with_latin_letter(token: &str) -> bool {
    token
        .chars()
        .find(|ch| ch.is_alphabetic())
        .is_some_and(|ch| ch.is_ascii_alphabetic())
}

fn repair_mixed_russian_token(token: &str) -> Option<String> {
    let candidate = latin_chars_to_ru(token);
    if candidate == token {
        return None;
    }
    if is_known_ru_token(&candidate) {
        return Some(candidate);
    }

    let latin_count = token.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
    if latin_count == 1 && crate::quality::score(&candidate, "ru") >= 0.99 {
        return Some(candidate);
    }

    should_repair_trailing_latin_as_ru(token, &candidate).then_some(candidate)
}

fn should_repair_trailing_latin_as_ru(token: &str, candidate: &str) -> bool {
    let mut prefix = String::new();
    let mut latin_tail = String::new();
    let mut seen_latin = false;

    for ch in token.chars() {
        if ch.is_ascii_alphabetic() {
            seen_latin = true;
            latin_tail.push(ch);
        } else if has_cyrillic_char(ch) {
            if seen_latin {
                return false;
            }
            prefix.push(ch);
        } else {
            return false;
        }
    }

    let prefix_len = prefix.chars().count();
    let tail_len = latin_tail.chars().count();
    if prefix_len < 3 || !(2..=4).contains(&tail_len) {
        return false;
    }

    let converted_tail = latin_chars_to_ru(&latin_tail);
    if converted_tail == latin_tail {
        return false;
    }

    let tail_has_vowel = converted_tail.chars().any(|ch| RU_VOWELS.contains(ch));
    if !tail_has_vowel {
        return false;
    }

    let prefix_lower = prefix.to_lowercase();
    let tail_lower = converted_tail.to_lowercase();
    if prefix_lower.ends_with(&tail_lower) {
        return false;
    }

    crate::quality::score(candidate, "ru") >= 0.99
}

fn should_convert_latin_island(token: &str) -> bool {
    let len = token.chars().count();
    len == 1 && !is_upper_ascii_acronym(token)
}

fn latin_chars_to_ru(token: &str) -> String {
    token
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphabetic() {
                crate::dict::convert(&ch.to_string(), crate::dict::Direction::Us2Ru)
            } else {
                ch.to_string()
            }
        })
        .collect()
}

fn has_cyrillic(text: &str) -> bool {
    text.chars().any(has_cyrillic_char)
}

fn has_cyrillic_char(ch: char) -> bool {
    matches!(ch, 'А'..='я' | 'ё' | 'Ё')
}

fn has_latin(text: &str) -> bool {
    text.chars().any(|ch| ch.is_ascii_alphabetic())
}

fn is_upper_ascii_acronym(token: &str) -> bool {
    let len = token.chars().count();
    (2..=4).contains(&len) && token.chars().all(|ch| ch.is_ascii_uppercase())
}

fn is_long_upper_ascii_word(token: &str) -> bool {
    token.chars().count() > 4 && token.chars().all(|ch| ch.is_ascii_uppercase())
}

fn is_single_ascii_letter(token: &str) -> bool {
    let mut chars = token.chars();
    matches!((chars.next(), chars.next()), (Some(ch), None) if ch.is_ascii_alphabetic())
}

fn is_single_cyrillic_letter(token: &str) -> bool {
    let mut chars = token.chars();
    matches!((chars.next(), chars.next()), (Some(ch), None) if has_cyrillic_char(ch))
}

fn is_mixed_case_ascii_brand(token: &str) -> bool {
    let letters: Vec<char> = token
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect();
    letters.len() >= 4
        && letters.iter().any(|ch| ch.is_ascii_lowercase())
        && letters.iter().skip(1).any(|ch| ch.is_ascii_uppercase())
}

fn parse_choice(response: &str) -> Option<Choice> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_letter_choices() {
        assert_eq!(parse_choice("A"), Some(Choice::Original));
        assert_eq!(parse_choice(" B"), Some(Choice::Converted));
        assert_eq!(parse_choice("A:"), Some(Choice::Original));
        assert_eq!(parse_choice("Answer: B"), Some(Choice::Converted));
        assert_eq!(parse_choice("To convert"), None);
    }

    #[test]
    fn token_consensus_lets_llm_veto_converted_choice() {
        let veto = |_original: &str, _converted: &str| Ok(Some(Choice::Original));
        assert_eq!(
            choose_token_consensus_with_chooser("dsdjlbv", "выводим", veto).unwrap(),
            Some("dsdjlbv".to_string())
        );
    }

    #[test]
    fn token_consensus_accepts_converted_when_llm_agrees() {
        let agree = |_original: &str, _converted: &str| Ok(Some(Choice::Converted));
        assert_eq!(
            choose_token_consensus_with_chooser("dsdjlbv", "выводим", agree).unwrap(),
            Some("выводим".to_string())
        );
    }

    #[test]
    fn token_consensus_keeps_protected_ascii_without_model() {
        let panic_chooser =
            |_original: &str, _converted: &str| panic!("protected token must not ask model");
        assert_eq!(
            choose_token_consensus_with_chooser("AmoCRM", "ФьщСКЬ", panic_chooser).unwrap(),
            Some("AmoCRM".to_string())
        );
    }

    #[test]
    fn repairs_mixed_russian_with_latin_islands() {
        assert_eq!(
            repair_mixed_script("добавm d LLM"),
            Some("добавь в LLM".to_string())
        );
        assert_eq!(
            repair_mixed_script("ПРОВTHM WORD"),
            Some("ПРОВЕРЬ WORD".to_string())
        );
    }

    #[test]
    fn does_not_glue_long_latin_tail_to_russian_word() {
        assert_eq!(repair_mixed_script("проверкаhrf ghj"), None);
        assert_eq!(
            convert_hybrid("проверкаhrf ghj", "ghjdthrfhrf ghj").unwrap(),
            Some("проверкаhrf ghj".to_string())
        );
    }

    #[test]
    fn does_not_treat_cyrillic_layout_word_with_latin_tail_as_ascii_brand() {
        assert_eq!(repair_mixed_script("ВщгиDo"), None);
        assert_eq!(
            convert_hybrid("в ВщгиDo", "d DoubDo").unwrap(),
            Some("в ВщгиDo".to_string())
        );
    }

    #[test]
    fn repairs_mixed_ascii_brand_tokens_before_layout_islands() {
        assert_eq!(
            repair_mixed_script("AьщСКЬ Z"),
            Some("AmoCRM Я".to_string())
        );
        assert_eq!(
            repair_mixed_script("AmoСКЬ Z"),
            Some("AmoCRM Я".to_string())
        );
    }

    #[test]
    fn keeps_plain_bilingual_text() {
        assert_eq!(repair_mixed_script("hello мир"), None);
        assert_eq!(repair_mixed_script("API для LLM"), None);
    }

    #[test]
    fn hybrid_keeps_plain_bilingual_text_without_model() {
        assert_eq!(
            convert_hybrid("hello мир", "руддщ vbh").unwrap(),
            Some("hello мир".to_string())
        );
        assert_eq!(
            convert_hybrid("API для LLM", "ФЗШ lkz ДДЬ").unwrap(),
            Some("API для LLM".to_string())
        );
    }

    #[test]
    fn hybrid_keeps_valid_russian_phrase_without_partial_single_letter_flip() {
        assert_eq!(
            convert_hybrid("в доме", "d ljvt").unwrap(),
            Some("в доме".to_string())
        );
    }

    #[test]
    fn hybrid_keeps_domain_and_converts_neighbor_word() {
        assert_eq!(
            convert_hybrid("conecargo.ru cj,bhfq", "сщтусфкпщюкг собирай").unwrap(),
            Some("conecargo.ru собирай".to_string())
        );
    }

    #[test]
    fn hybrid_keeps_mixed_case_ascii_brand_and_converts_neighbor_letter() {
        assert_eq!(
            convert_hybrid("AmoCRM Z", "ФьщСКЬ Я").unwrap(),
            Some("AmoCRM Я".to_string())
        );
    }

    #[test]
    fn tokenwise_hybrid_keeps_good_word_and_converts_bad_neighbor() {
        let result = choose_mixed_token_candidate(
            "Главная Вщгиду",
            "Ukfdyfz Double",
            |original, _| {
                Ok(Some(if original == "Главная" {
                    Choice::Original
                } else {
                    Choice::Converted
                }))
            },
        )
        .unwrap();

        assert_eq!(result, Some("Главная Double".to_string()));
    }

    #[test]
    fn tokenwise_hybrid_converts_unknown_long_all_caps_neighbor() {
        let result = choose_mixed_token_candidate(
            "DOUBLE DUBLE",
            "ВЩГИДУ ВГИДУ",
            |original, converted| {
                panic!("model should not be called for {original:?} -> {converted:?}");
            },
        )
        .unwrap();

        assert_eq!(result, Some("DOUBLE ВГИДУ".to_string()));
    }

    #[test]
    fn tokenwise_hybrid_keeps_unknown_all_caps_brand_when_converted_is_garbage() {
        let result =
            choose_mixed_token_candidate("AMOCRM Z", "ФЬЩСКЬ Я", |original, converted| {
                panic!("model should not be called for {original:?} -> {converted:?}");
            })
            .unwrap();

        assert_eq!(result, Some("AMOCRM Я".to_string()));
    }

    #[test]
    fn tokenwise_hybrid_converts_all_obvious_layout_garbage() {
        let result =
            choose_mixed_token_candidate("руддщ цщкдв", "hello world", |_, _| {
                Ok(Some(Choice::Converted))
            })
            .unwrap();

        assert_eq!(result, Some("hello world".to_string()));
    }

    #[test]
    fn tokenwise_hybrid_converts_all_obviously_bad_words_without_model() {
        let result = choose_mixed_token_candidate(
            "dsdjlbv ldf",
            "выводим два",
            |original, converted| {
                panic!("model should not be called for {original:?} -> {converted:?}");
            },
        )
        .unwrap();

        assert_eq!(result, Some("выводим два".to_string()));
    }

    #[test]
    fn tokenwise_hybrid_keeps_all_obviously_good_words_without_model() {
        let result = choose_mixed_token_candidate(
            "выводим два",
            "dsdjlbv ldf",
            |original, converted| {
                panic!("model should not be called for {original:?} -> {converted:?}");
            },
        )
        .unwrap();

        assert_eq!(result, Some("выводим два".to_string()));
    }

    #[test]
    fn tokenwise_hybrid_keeps_obviously_good_russian_without_asking_model() {
        let result = choose_mixed_token_candidate(
            "Главная Вщгиду",
            "Ukfdyfz Double",
            |original, _| {
                assert_ne!(original, "Главная");
                Ok(Some(Choice::Converted))
            },
        )
        .unwrap();

        assert_eq!(result, Some("Главная Double".to_string()));
    }

    #[test]
    fn tokenwise_hybrid_uses_dictionaries_before_model() {
        let result = choose_mixed_token_candidate(
            "Главное Вщгиду",
            "Ukfdyjt Double",
            |original, converted| {
                panic!("model should not be called for {original:?} -> {converted:?}");
            },
        )
        .unwrap();

        assert_eq!(result, Some("Главное Double".to_string()));
    }

    #[test]
    fn token_hybrid_keeps_good_previous_word_or_converts_bad_one() {
        assert_eq!(
            choose_token_hybrid("в", "d").unwrap(),
            Some("в".to_string())
        );
        assert_eq!(
            choose_token_hybrid("ghbdtn", "привет").unwrap(),
            Some("привет".to_string())
        );
        assert_eq!(
            choose_token_hybrid("DOUBLE", "ВЩГИДУ").unwrap(),
            Some("DOUBLE".to_string())
        );
    }

    #[test]
    fn tokenwise_hybrid_converts_bad_mixed_layout_neighbor_only() {
        let result =
            choose_mixed_token_candidate("рка ghj", "hrf про", |original, converted| {
                panic!("model should not be called for {original:?} -> {converted:?}");
            })
            .unwrap();

        assert_eq!(result, Some("рка про".to_string()));
    }
}
