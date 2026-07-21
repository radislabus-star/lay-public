use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

use super::context_phase::{self, ContextPhaseDisposition};

const DEFAULT_MAX_PHRASES: usize = 256;
const DEFAULT_MAX_PAIRS: usize = 2_000;
const DEFAULT_MIN_PROFILE_SUPPORT: u32 = 2;
const DEFAULT_MIN_SURFACE_SUPPORT: u32 = 1;
const CORPUS_SUPPORT_REPEATS: usize = 2;
const PROJECT_CLEAN_SEED: &str = "data/nanda_llmwave_seed_phrases.txt";

#[derive(Clone, Debug)]
pub struct LaySelfTeacherL3Config {
    pub output_dir: PathBuf,
    pub clean_corpus: Option<PathBuf>,
    pub usage_events: Option<PathBuf>,
    pub include_default_live_feedback: bool,
    pub max_phrases: usize,
    pub max_pairs: usize,
    pub max_fragments: usize,
    pub min_profile_support: u32,
    pub min_surface_support: u32,
}

impl Default for LaySelfTeacherL3Config {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            clean_corpus: None,
            usage_events: default_usage_events_path().filter(|path| path.exists()),
            include_default_live_feedback: true,
            max_phrases: DEFAULT_MAX_PHRASES,
            max_pairs: DEFAULT_MAX_PAIRS,
            max_fragments: 0,
            min_profile_support: DEFAULT_MIN_PROFILE_SUPPORT,
            min_surface_support: DEFAULT_MIN_SURFACE_SUPPORT,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct DirtyExample {
    class: &'static str,
    dirty_phrase: String,
    clean_phrase: String,
    dirty_token: Option<String>,
    clean_token: Option<String>,
    token_index: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ShadowMetrics {
    cases: usize,
    skipped_context_too_short: usize,
    evidence_hit: usize,
    signature_hit: usize,
    authority: usize,
    output_changed: usize,
    target_top1: usize,
    false_top1: usize,
    false_authority: usize,
    pairwise_certified: usize,
    pairwise_blocked_wrong: usize,
    candidate_order_stable: usize,
    candidate_order_changed: usize,
}

pub fn build_lay_self_teacher_l3_report(
    config: LaySelfTeacherL3Config,
) -> io::Result<serde_json::Value> {
    fs::create_dir_all(&config.output_dir)?;

    let clean_phrases = clean_phrases(&config)?;
    let dirty_examples = dirty_examples(&clean_phrases, config.max_pairs);
    let by_error_class = count_by_class(&dirty_examples);

    let clean_corpus_path = config.output_dir.join("clean_context_corpus.txt");
    let surface_evidence_path = config.output_dir.join("surface_evidence.jsonl");
    let eval_cases_path = config.output_dir.join("dirty_eval_cases.jsonl");
    let package_path = config.output_dir.join("l3_self_teacher_shadow.nwpc");

    write_clean_corpus(&clean_corpus_path, &clean_phrases)?;
    write_surface_evidence(&surface_evidence_path, &dirty_examples)?;
    write_eval_cases(&eval_cases_path, &dirty_examples)?;

    let compile_report = super::compile_l3_context_phase_memory_with_surface_evidence(
        &clean_corpus_path,
        &surface_evidence_path,
        &package_path,
        config.max_fragments,
        config.min_profile_support,
        config.min_surface_support,
    )?;
    let package = context_phase::read_package(&package_path)?;
    let shadow = shadow_metrics(&package, &clean_phrases, &dirty_examples);
    let verdict = shadow_verdict(&shadow);

    Ok(json!({
        "kind": "lay_self_teacher_l3_report",
        "architecture": "offline_clean_surface_teacher_to_context_phase_v1",
        "runtime_authority": false,
        "runtime_installed": false,
        "external_llm_used": false,
        "raw_words_stored_in_hot_package": false,
        "read_as": "offline teacher/proof artifact; never live authority until a separate promotion gate passes",
        "config": {
            "output_dir": config.output_dir,
            "clean_corpus": config.clean_corpus,
            "usage_events": config.usage_events,
            "include_default_live_feedback": config.include_default_live_feedback,
            "max_phrases": config.max_phrases,
            "max_pairs": config.max_pairs,
            "max_fragments": config.max_fragments,
            "min_profile_support": config.min_profile_support,
            "min_surface_support": config.min_surface_support,
            "corpus_support_repeats": CORPUS_SUPPORT_REPEATS,
            "project_clean_seed": PROJECT_CLEAN_SEED,
        },
        "teacher": {
            "clean_phrases": clean_phrases.len(),
            "dirty_pairs": dirty_examples.len(),
            "by_error_class": by_error_class,
        },
        "artifacts": {
            "clean_context_corpus": clean_corpus_path,
            "surface_evidence": surface_evidence_path,
            "dirty_eval_cases": eval_cases_path,
            "shadow_package": package_path,
        },
        "compile": compile_report,
        "shadow": {
            "verdict": verdict,
            "cases": shadow.cases,
            "skipped_context_too_short": shadow.skipped_context_too_short,
            "evidence_hit": shadow.evidence_hit,
            "evidence_hit_percent": percent(shadow.evidence_hit, shadow.cases),
            "signature_hit": shadow.signature_hit,
            "signature_hit_percent": percent(shadow.signature_hit, shadow.cases),
            "authority": shadow.authority,
            "authority_percent": percent(shadow.authority, shadow.cases),
            "output_changed": shadow.output_changed,
            "output_changed_percent": percent(shadow.output_changed, shadow.cases),
            "target_top1": shadow.target_top1,
            "target_top1_percent": percent(shadow.target_top1, shadow.cases),
            "false_top1": shadow.false_top1,
            "false_top1_percent": percent(shadow.false_top1, shadow.cases),
            "false_authority": shadow.false_authority,
            "false_authority_percent": percent(shadow.false_authority, shadow.cases),
            "pairwise_certified": shadow.pairwise_certified,
            "pairwise_blocked_wrong": shadow.pairwise_blocked_wrong,
            "candidate_order_stable": shadow.candidate_order_stable,
            "candidate_order_changed": shadow.candidate_order_changed,
        },
        "promotion_gate": {
            "package_published": false,
            "requires": [
                "shadow verdict PASS",
                "false_authority == 0",
                "target_top1 improves against baseline",
                "authority > baseline L3 authority",
                "candidate_order_changed == 0",
                "separate live replay before install"
            ],
        },
    }))
}

fn default_output_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/share/lay/self_teacher/l3")
}

fn default_usage_events_path() -> Option<PathBuf> {
    Some(
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local/share/lay/nanda_wave/word_usage_events.jsonl"),
    )
}

fn clean_phrases(config: &LaySelfTeacherL3Config) -> io::Result<Vec<String>> {
    let mut seen = BTreeSet::new();
    let mut phrases = Vec::new();
    for phrase in DEFAULT_CLEAN_PHRASES {
        push_clean_phrase(*phrase, &mut seen, &mut phrases, config.max_phrases);
    }
    if Path::new(PROJECT_CLEAN_SEED).exists() {
        for raw in fs::read_to_string(PROJECT_CLEAN_SEED)?.lines() {
            push_clean_phrase(raw, &mut seen, &mut phrases, config.max_phrases);
            if phrases.len() >= config.max_phrases {
                break;
            }
        }
    }
    if config.include_default_live_feedback {
        if let Some(path) = &config.usage_events {
            if path.exists() {
                let events = fs::read_to_string(path)?;
                let (feedback_corpus, _) = context_phase::build_feedback_corpus(&events, 3)?;
                for raw in feedback_corpus.lines() {
                    push_clean_phrase(raw, &mut seen, &mut phrases, config.max_phrases);
                    if phrases.len() >= config.max_phrases {
                        break;
                    }
                }
            }
        }
    }
    if let Some(path) = &config.clean_corpus {
        for raw in fs::read_to_string(path)?.lines() {
            push_clean_phrase(raw, &mut seen, &mut phrases, config.max_phrases);
            if phrases.len() >= config.max_phrases {
                break;
            }
        }
    }
    Ok(phrases)
}

fn push_clean_phrase(
    phrase: &str,
    seen: &mut BTreeSet<String>,
    phrases: &mut Vec<String>,
    limit: usize,
) {
    if phrases.len() >= limit {
        return;
    }
    let normalized = normalize_phrase(phrase);
    let token_count = super::llmwave::tokenize(&normalized).len();
    if (3..=64).contains(&token_count) && seen.insert(normalized.clone()) {
        phrases.push(normalized);
    }
}

fn normalize_phrase(phrase: &str) -> String {
    phrase
        .split_whitespace()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn write_clean_corpus(path: &Path, phrases: &[String]) -> io::Result<()> {
    let mut text = String::new();
    for phrase in phrases {
        for _ in 0..CORPUS_SUPPORT_REPEATS {
            text.push_str(phrase);
            text.push('\n');
        }
    }
    fs::write(path, text)
}

fn write_surface_evidence(path: &Path, examples: &[DirtyExample]) -> io::Result<()> {
    let mut text = String::new();
    for example in examples {
        let (Some(from), Some(to)) = (&example.dirty_token, &example.clean_token) else {
            continue;
        };
        text.push_str(&serde_json::to_string(&json!({
            "from": from,
            "to": to,
            "class": example.class,
        }))?);
        text.push('\n');
    }
    fs::write(path, text)
}

fn write_eval_cases(path: &Path, examples: &[DirtyExample]) -> io::Result<()> {
    let mut text = String::new();
    for example in examples {
        text.push_str(&serde_json::to_string(example)?);
        text.push('\n');
    }
    fs::write(path, text)
}

fn dirty_examples(clean_phrases: &[String], limit: usize) -> Vec<DirtyExample> {
    let mut examples = Vec::new();
    let mut seen = BTreeSet::new();
    for phrase in clean_phrases {
        let tokens = super::llmwave::tokenize(phrase);
        for (index, token) in tokens.iter().enumerate() {
            for (class, dirty) in word_damages(token) {
                let mut dirty_tokens = tokens.clone();
                dirty_tokens[index] = dirty.clone();
                push_dirty_example(
                    DirtyExample {
                        class,
                        dirty_phrase: dirty_tokens.join(" "),
                        clean_phrase: tokens.join(" "),
                        dirty_token: Some(dirty),
                        clean_token: Some(token.clone()),
                        token_index: Some(index),
                    },
                    &mut seen,
                    &mut examples,
                    limit,
                );
            }
            if token.chars().count() >= 6 {
                let split_at = token.chars().count() / 2;
                let left = token.chars().take(split_at).collect::<String>();
                let right = token.chars().skip(split_at).collect::<String>();
                if !left.is_empty() && !right.is_empty() {
                    let mut dirty_tokens = tokens.clone();
                    dirty_tokens.splice(index..=index, [left, right]);
                    push_dirty_example(
                        DirtyExample {
                            class: "premature_space",
                            dirty_phrase: dirty_tokens.join(" "),
                            clean_phrase: tokens.join(" "),
                            dirty_token: None,
                            clean_token: Some(token.clone()),
                            token_index: Some(index),
                        },
                        &mut seen,
                        &mut examples,
                        limit,
                    );
                }
            }
            if index + 1 < tokens.len() {
                let mut dirty_tokens = tokens.clone();
                let glued = format!("{}{}", dirty_tokens[index], dirty_tokens[index + 1]);
                dirty_tokens.splice(index..=index + 1, [glued]);
                push_dirty_example(
                    DirtyExample {
                        class: "glued_words",
                        dirty_phrase: dirty_tokens.join(" "),
                        clean_phrase: tokens.join(" "),
                        dirty_token: None,
                        clean_token: None,
                        token_index: Some(index),
                    },
                    &mut seen,
                    &mut examples,
                    limit,
                );
            }
            if examples.len() >= limit {
                return examples;
            }
        }
    }
    examples
}

fn push_dirty_example(
    example: DirtyExample,
    seen: &mut BTreeSet<(String, String, &'static str)>,
    examples: &mut Vec<DirtyExample>,
    limit: usize,
) {
    if examples.len() >= limit {
        return;
    }
    if example.dirty_phrase == example.clean_phrase {
        return;
    }
    let key = (
        example.dirty_phrase.clone(),
        example.clean_phrase.clone(),
        example.class,
    );
    if seen.insert(key) {
        examples.push(example);
    }
}

fn word_damages(token: &str) -> Vec<(&'static str, String)> {
    let mut result = Vec::new();
    if !is_word_token(token) {
        return result;
    }
    if let Some(value) = missing_letter(token) {
        result.push(("missing_letter", value));
    }
    if let Some(value) = adjacent_transposition(token) {
        result.push(("adjacent_transposition", value));
    }
    if let Some(value) = extra_letter(token) {
        result.push(("extra_letter", value));
    }
    if let Some(value) = layout_projection(token) {
        result.push(("layout_projection", value));
    }
    result
}

fn missing_letter(token: &str) -> Option<String> {
    let chars = token.chars().collect::<Vec<_>>();
    if chars.len() < 5 {
        return None;
    }
    let index = (chars.len() / 2 + 1).min(chars.len() - 1);
    Some(
        chars
            .iter()
            .enumerate()
            .filter_map(|(position, ch)| (position != index).then_some(*ch))
            .collect(),
    )
}

fn adjacent_transposition(token: &str) -> Option<String> {
    let mut chars = token.chars().collect::<Vec<_>>();
    if chars.len() < 5 {
        return None;
    }
    let index = chars.len() / 2;
    if index + 1 >= chars.len() {
        return None;
    }
    if chars[index] == chars[index + 1] {
        return None;
    }
    chars.swap(index, index + 1);
    Some(chars.into_iter().collect())
}

fn extra_letter(token: &str) -> Option<String> {
    let chars = token.chars().collect::<Vec<_>>();
    if chars.len() < 5 {
        return None;
    }
    let index = chars.len() / 2;
    let mut result = String::new();
    for (position, ch) in chars.iter().enumerate() {
        result.push(*ch);
        if position == index {
            result.push(*ch);
        }
    }
    Some(result)
}

fn layout_projection(token: &str) -> Option<String> {
    let cyrillic = token.chars().all(crate::keyboard::is_cyrillic_letter);
    let ascii = token.chars().all(|ch| ch.is_ascii_alphabetic());
    if !cyrillic && !ascii {
        return None;
    }
    let events = crate::keyboard::text_to_key_events(token, cyrillic)?;
    let projected = crate::keyboard::map_events_to_layout(&events, !cyrillic);
    (projected != token && is_word_token(&projected)).then_some(projected)
}

fn is_word_token(token: &str) -> bool {
    !token.is_empty() && token.chars().all(char::is_alphabetic)
}

fn count_by_class(examples: &[DirtyExample]) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for example in examples {
        *counts.entry(example.class).or_default() += 1;
    }
    counts
}

fn shadow_metrics(
    package: &context_phase::ContextPhasePackage,
    clean_phrases: &[String],
    examples: &[DirtyExample],
) -> ShadowMetrics {
    let vocabulary = clean_vocabulary(clean_phrases);
    let mut metrics = ShadowMetrics::default();
    for example in examples {
        let (Some(target), Some(dirty), Some(index)) = (
            example.clean_token.as_deref(),
            example.dirty_token.as_deref(),
            example.token_index,
        ) else {
            continue;
        };
        let clean_tokens = super::llmwave::tokenize(&example.clean_phrase);
        if index < 2 || index > clean_tokens.len() {
            metrics.skipped_context_too_short += 1;
            continue;
        }
        let context = clean_tokens[..index].to_vec();
        let candidates = shadow_candidates(target, dirty, &vocabulary);
        let candidate_refs = candidates.iter().map(String::as_str).collect::<Vec<_>>();
        let readouts = package.score_candidates(&context, &candidate_refs);
        let winner = top_candidate(&candidates, &readouts);
        let target_readout = candidates
            .iter()
            .position(|candidate| candidate == target)
            .and_then(|target_index| readouts.get(target_index));
        let Some(target_readout) = target_readout else {
            continue;
        };
        metrics.cases += 1;
        if target_readout.profile_present {
            metrics.evidence_hit += 1;
        }
        if target_readout.signature_profile_present {
            metrics.signature_hit += 1;
        }
        if target_readout.disposition == ContextPhaseDisposition::Support {
            metrics.authority += 1;
        }
        if target_readout.disposition == ContextPhaseDisposition::Support && target != dirty {
            metrics.output_changed += 1;
        }
        if winner.as_deref() == Some(target) {
            metrics.target_top1 += 1;
        } else if winner.is_some() {
            metrics.false_top1 += 1;
        }
        if readouts
            .iter()
            .zip(&candidates)
            .any(|(readout, candidate)| {
                candidate != target && readout.disposition == ContextPhaseDisposition::Support
            })
        {
            metrics.false_authority += 1;
        }
        if target_readout.pairwise_certified {
            metrics.pairwise_certified += 1;
        }
        if readouts
            .iter()
            .zip(&candidates)
            .any(|(readout, candidate)| candidate != target && readout.pairwise_blocked)
        {
            metrics.pairwise_blocked_wrong += 1;
        }
        if candidate_order_is_stable(package, &context, &candidates, winner.as_deref()) {
            metrics.candidate_order_stable += 1;
        } else {
            metrics.candidate_order_changed += 1;
        }
    }
    metrics
}

fn clean_vocabulary(clean_phrases: &[String]) -> Vec<String> {
    let mut words = BTreeSet::new();
    for phrase in clean_phrases {
        for token in super::llmwave::tokenize(phrase) {
            if is_word_token(&token) {
                words.insert(token);
            }
        }
    }
    words.into_iter().collect()
}

fn shadow_candidates(target: &str, dirty: &str, vocabulary: &[String]) -> Vec<String> {
    let mut candidates = vec![dirty.to_string(), target.to_string()];
    let target_len = target.chars().count();
    for word in vocabulary {
        if word == target || word == dirty {
            continue;
        }
        let len = word.chars().count();
        if len.abs_diff(target_len) <= 2
            && word
                .chars()
                .next()
                .zip(target.chars().next())
                .is_some_and(|(left, right)| left == right)
        {
            candidates.push(word.clone());
        }
        if candidates.len() >= 6 {
            break;
        }
    }
    candidates
}

fn top_candidate(
    candidates: &[String],
    readouts: &[context_phase::ContextPhaseReadout],
) -> Option<String> {
    candidates
        .iter()
        .zip(readouts)
        .max_by(|(left_candidate, left), (right_candidate, right)| {
            left.margin_micro
                .cmp(&right.margin_micro)
                .then_with(|| left.positive_micro.cmp(&right.positive_micro))
                .then_with(|| right_candidate.cmp(left_candidate))
        })
        .map(|(candidate, _)| candidate.clone())
}

fn candidate_order_is_stable(
    package: &context_phase::ContextPhasePackage,
    context: &[String],
    candidates: &[String],
    expected_winner: Option<&str>,
) -> bool {
    let mut reversed = candidates.to_vec();
    reversed.reverse();
    let refs = reversed.iter().map(String::as_str).collect::<Vec<_>>();
    let readouts = package.score_candidates(context, &refs);
    top_candidate(&reversed, &readouts).as_deref() == expected_winner
}

fn shadow_verdict(metrics: &ShadowMetrics) -> &'static str {
    if metrics.cases == 0 {
        return "WATCH_NO_CASES";
    }
    if metrics.false_top1 == 0
        && metrics.false_authority == 0
        && metrics.candidate_order_changed == 0
        && metrics.authority > 0
        && metrics.output_changed > 0
    {
        "PASS_shadow"
    } else {
        "WATCH_shadow"
    }
}

fn percent(value: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        ((value as f64 * 10_000.0 / total as f64).round()) / 100.0
    }
}

const DEFAULT_CLEAN_PHRASES: &[&str] = &[
    "на улице снова начался дождь",
    "за окном весь вечер идет дождь",
    "вечером ребенку пора спать",
    "сервер работает на постоянку",
    "нужно проверить скрытое состояние",
    "можно подключить локальную память",
    "давай обновим фазовый пакет",
    "автозамена должна менять только токен",
    "подсказка должна исчезать после пробела",
    "контекст должен выбирать правильное слово",
    "telegram нормально принимает ime",
    "firefox показывает подсказку медленнее",
    "wechat требует другой backend удаления",
    "download файл не должен переворачиваться",
    "file должен остаться английским словом",
    "layout projection чинит неправильную раскладку",
    "надо переподключить новый демон",
    "мы проверяем грязный ввод",
    "волна восстанавливает поврежденные формы",
    "фазовый центр выбирает верный кандидат",
    "анти волна подавляет ложный кандидат",
    "пользователь завершает слово пробелом",
    "кандидат должен меняться по хвосту",
    "после таба слово принимается явно",
    "после пробела ime очищается полностью",
    "правильная подсказка приходит сразу",
    "русское слово восстанавливается из шума",
    "английское слово остается английским",
    "клавиатурный перевертыш чинится волной",
    "контекст фразы усиливает смысл",
    "память пользователя усиливает выбор",
    "отрицательный опыт создает антифазу",
    "решение проходит независимый verifier",
    "физическое изменение требует capability",
    "daemon исполняет только authorized edit",
    "ime показывает только незавершенный хвост",
    "boundary исправление идет отдельным маршрутом",
    "грязные логи дают обучающие примеры",
    "чистый корпус дает правильные связи",
    "l3 должен видеть всю сцену",
    "l4 запоминает принятый исход",
    "l2 рождает решетку кандидатов",
    "l1 кодирует поверхность слова",
    "bayes поднимает частый вариант",
    "ошибочный победитель уходит в антицентр",
    "новый пакет проходит shadow replay",
    "runtime получает пакет только после pass",
    "мы не храним сырой текст в hot памяти",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_generic_word_damage_classes() {
        let damages = word_damages("время");
        assert!(damages
            .iter()
            .any(|(class, value)| *class == "adjacent_transposition" && value == "врмея"));
        assert!(damages
            .iter()
            .any(|(class, value)| *class == "missing_letter" && value == "врея"));
    }

    #[test]
    fn layout_projection_uses_keyboard_mapping() {
        assert_eq!(layout_projection("давай").as_deref(), Some("lfdfq"));
        assert_eq!(layout_projection("file").as_deref(), Some("ашду"));
    }

    #[test]
    fn dirty_examples_include_phrase_boundary_damage_without_runtime_rules() {
        let examples = dirty_examples(&["нужно проверить скрытое состояние".to_string()], 64);
        assert!(examples
            .iter()
            .any(|example| example.class == "glued_words"));
        assert!(examples
            .iter()
            .any(|example| example.class == "premature_space"));
    }
}
