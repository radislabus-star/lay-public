//! Research-only probe for LEM (Layout Error Metric).
//!
//! This binary does not participate in the daemon path. It builds synthetic
//! candidate sets and checks whether a scoring function can choose the intended
//! text from noisy layout/typo variants.

use lay::dict::{self, Direction};
use lay::keyboard::is_cyrillic_letter;
use lay::ngram;
use lay::word_recognizer::{
    is_ascii_technical_or_brand_token, is_mixed_cyrillic_ascii_alpha_token,
};
use std::collections::HashSet;
use std::sync::OnceLock;

#[allow(dead_code)]
#[path = "../text_metrics.rs"]
mod text_metrics;

use text_metrics::{
    common_replacement_span, damerau_levenshtein, normalized_edit_distance, without_whitespace,
};

const TARGET_CASES: usize = 12_000;
const RU_HUNSPELL: &str = "/usr/share/hunspell/ru_RU.dic";
const EN_HUNSPELL: &str = "/usr/share/hunspell/en_US.dic";
const EN_WORDS: &str = "/usr/share/dict/words";

#[derive(Clone, Debug)]
struct Case {
    kind: &'static str,
    typed: String,
    expected: String,
}

#[derive(Clone, Debug)]
struct ScoredCandidate {
    text: String,
    total: f64,
    language: f64,
    noise: f64,
    edit: f64,
    intervention: f64,
}

fn main() {
    let cases = build_cases(TARGET_CASES);
    let mut ok = 0usize;
    let mut by_kind: Vec<(&'static str, usize, usize)> = Vec::new();
    let mut failures = Vec::new();
    let mut margins = Vec::new();

    for case in &cases {
        let ranked = rank_candidates(case);
        let best = &ranked[0];
        let second = &ranked[1];
        let passed = best.text == case.expected;
        if passed {
            ok += 1;
            margins.push(best.total - second.total);
        } else if failures.len() < 20 {
            failures.push((case.clone(), ranked[..ranked.len().min(4)].to_vec()));
        }

        match by_kind.iter_mut().find(|(kind, _, _)| *kind == case.kind) {
            Some((_, passed_count, total_count)) => {
                *passed_count += usize::from(passed);
                *total_count += 1;
            }
            None => by_kind.push((case.kind, usize::from(passed), 1)),
        }
    }

    margins.sort_by(f64::total_cmp);
    let accuracy = ok as f64 * 100.0 / cases.len() as f64;
    let median_margin = percentile(&margins, 0.50);
    let p10_margin = percentile(&margins, 0.10);

    println!("LEM research probe");
    println!("cases: {}", cases.len());
    println!("passed: {ok}/{} ({accuracy:.1}%)", cases.len());
    println!("median winning margin: {median_margin:.3}");
    println!("p10 winning margin: {p10_margin:.3}");
    println!();
    println!("by kind:");
    for (kind, passed, total) in by_kind {
        let pct = passed as f64 * 100.0 / total as f64;
        println!("  {kind:24} {passed:4}/{total:<4} {pct:5.1}%");
    }

    if !failures.is_empty() {
        println!();
        println!("first failures:");
        for (case, ranked) in failures {
            println!(
                "  kind={} typed={:?} expected={:?}",
                case.kind, case.typed, case.expected
            );
            for candidate in ranked {
                println!(
                    "    {:>8.3} lang={:>7.3} noise={:>5.2} edit={:>5.2} int={:>5.2} {:?}",
                    candidate.total,
                    candidate.language,
                    candidate.noise,
                    candidate.edit,
                    candidate.intervention,
                    candidate.text
                );
            }
        }
    }
}

fn rank_candidates(case: &Case) -> Vec<ScoredCandidate> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    for candidate in candidate_texts(case) {
        if seen.insert(candidate.clone()) {
            candidates.push(score_candidate(&case.typed, candidate));
        }
    }
    candidates.sort_by(|a, b| b.total.total_cmp(&a.total));
    candidates
}

fn candidate_texts(case: &Case) -> Vec<String> {
    let mut out = Vec::new();
    push_candidate(&mut out, &case.typed);
    push_candidate(&mut out, &case.expected);
    push_candidate(&mut out, &dict::convert(&case.typed, Direction::Us2Ru));
    push_candidate(&mut out, &dict::convert(&case.typed, Direction::Ru2Us));
    push_candidate(&mut out, &flip_tokens(&case.typed));
    push_candidate(&mut out, &case.typed.replace(' ', ""));
    push_candidate(&mut out, &case.expected.replace(' ', ""));
    push_candidate(&mut out, &case.expected.replace("  ", " "));
    for candidate in moved_space_candidates(&case.typed) {
        push_candidate(&mut out, &candidate);
    }
    for candidate in repeated_letter_candidates(&case.typed) {
        push_candidate(&mut out, &candidate);
    }
    out
}

fn push_candidate(out: &mut Vec<String>, text: &str) {
    let text = text.trim().to_string();
    if !text.is_empty() && !out.iter().any(|item| item == &text) {
        out.push(text);
    }
}

fn score_candidate(typed: &str, candidate: String) -> ScoredCandidate {
    let language = language_score(&candidate);
    let noise = noise_cost(typed, &candidate);
    let edit = normalized_edit_distance(typed, &candidate);
    let intervention = intervention_penalty(typed, &candidate);
    let total =
        language + structure_bonus(typed, &candidate) + keep_valid_source_bonus(typed, &candidate)
            - (1.0 + noise).ln()
            - edit * 0.25
            - intervention;
    ScoredCandidate {
        text: candidate,
        total,
        language,
        noise,
        edit,
        intervention,
    }
}

fn language_score(text: &str) -> f64 {
    let mut total = 0.0;
    let mut count = 0usize;
    for token in text.split_whitespace() {
        let token = trim_token(token);
        if token.is_empty() {
            continue;
        }
        total += token_language_score(token);
        count += 1;
    }
    if count == 0 {
        -20.0
    } else {
        total / count as f64
    }
}

fn token_language_score(token: &str) -> f64 {
    let lower = token.to_lowercase();
    if is_short_russian_function_word(&lower) {
        return -5.5;
    }
    if is_mixed_cyrillic_ascii_alpha_token(token) {
        return -22.0;
    }
    if is_layout_garbage_token(token) {
        return -18.0;
    }
    if is_ascii_technical_or_brand_token(token) {
        return -5.0;
    }

    let alpha_count = token.chars().filter(|ch| ch.is_alphabetic()).count().max(1) as f64;
    let ru = ngram::ru_score(token);
    let en = ngram::en_score(token);
    let ru_norm = if ru.is_finite() {
        ru / alpha_count
    } else {
        -20.0
    };
    let en_norm = if en.is_finite() {
        en / alpha_count
    } else {
        -20.0
    };
    ru_norm.max(en_norm) + lexical_bonus(token)
}

fn noise_cost(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    if dict::convert(typed, Direction::Us2Ru) == candidate
        || dict::convert(typed, Direction::Ru2Us) == candidate
    {
        return 0.15;
    }
    if flip_tokens(typed) == candidate {
        return 0.25;
    }
    if let Some(cost) = partial_token_flip_cost(typed, candidate) {
        return cost;
    }
    if typed.replace(' ', "") == candidate.replace(' ', "") {
        return 0.35;
    }
    if removes_extra_repeated_letter(typed, candidate) {
        return 0.08;
    }

    let distance = damerau_levenshtein(typed, candidate) as f64;
    let scale = typed.chars().count().max(candidate.chars().count()).max(1) as f64;
    if distance <= 2.0 {
        return 0.10 + distance / scale * 0.25;
    }
    0.75 + distance / scale
}

fn partial_token_flip_cost(typed: &str, candidate: &str) -> Option<f64> {
    let typed_tokens: Vec<&str> = typed.split_whitespace().collect();
    let candidate_tokens: Vec<&str> = candidate.split_whitespace().collect();
    if typed_tokens.len() != candidate_tokens.len() || typed_tokens.is_empty() {
        return None;
    }

    let mut changed = 0usize;
    for (typed_token, candidate_token) in typed_tokens.iter().zip(candidate_tokens.iter()) {
        if typed_token == candidate_token {
            continue;
        }
        if dict::convert(typed_token, dict::detect_direction(typed_token)) == *candidate_token {
            changed += 1;
            continue;
        }
        return None;
    }

    (changed > 0).then_some(0.18 + changed as f64 * 0.10)
}

fn intervention_penalty(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    let mut protected_penalty = 0.0;
    let typed_tokens: Vec<&str> = typed.split_whitespace().collect();
    let candidate_tokens: Vec<&str> = candidate.split_whitespace().collect();
    let same_letters_with_moved_spaces = without_whitespace(typed) == without_whitespace(candidate);
    if typed_tokens.len() == candidate_tokens.len() && !same_letters_with_moved_spaces {
        for (typed_token, candidate_token) in typed_tokens.iter().zip(candidate_tokens.iter()) {
            let typed_token = trim_token(typed_token);
            let candidate_token = trim_token(candidate_token);
            if typed_token != candidate_token && is_known_word(typed_token) {
                protected_penalty += 2.0;
            }
        }
    }
    let touched = common_replacement_span(typed, candidate) as f64;
    let mut penalty = 0.08 + touched * 0.006 + protected_penalty;
    let typed_spaces = typed.chars().filter(|ch| ch.is_whitespace()).count();
    let candidate_spaces = candidate.chars().filter(|ch| ch.is_whitespace()).count();
    if candidate_spaces < typed_spaces {
        let space_loss = (typed_spaces - candidate_spaces) as f64;
        penalty += space_loss * 6.0;
    } else if candidate_spaces > typed_spaces {
        penalty += (candidate_spaces - typed_spaces) as f64 * 0.08;
    }
    penalty
}

fn structure_bonus(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    let typed_spaces = typed.chars().filter(|ch| ch.is_whitespace()).count();
    let candidate_spaces = candidate.chars().filter(|ch| ch.is_whitespace()).count();
    let same_letters_with_moved_spaces = without_whitespace(typed) == without_whitespace(candidate);
    if typed_spaces == 0 && candidate_spaces > 0 && same_letters_with_moved_spaces {
        return 1.45;
    }
    if typed_spaces == candidate_spaces && same_letters_with_moved_spaces {
        return 0.30;
    }
    if removes_extra_repeated_letter(typed, candidate) {
        return 0.25;
    }

    let typed_known = typed
        .split_whitespace()
        .filter(|token| is_known_word(trim_token(token)))
        .count();
    let candidate_known = candidate
        .split_whitespace()
        .filter(|token| is_known_word(trim_token(token)))
        .count();
    if typed_spaces == candidate_spaces && candidate_known > typed_known {
        return 0.45;
    }
    0.0
}

fn removes_extra_repeated_letter(typed: &str, candidate: &str) -> bool {
    if typed == candidate || typed.chars().count() <= candidate.chars().count() {
        return false;
    }

    let typed_chars: Vec<char> = typed.chars().collect();
    for idx in 1..typed_chars.len() {
        if typed_chars[idx] != typed_chars[idx - 1] {
            continue;
        }
        let mut repaired = String::with_capacity(typed.len());
        for (pos, ch) in typed_chars.iter().enumerate() {
            if pos != idx {
                repaired.push(*ch);
            }
        }
        if repaired == candidate {
            return true;
        }
    }
    false
}

fn keep_valid_source_bonus(typed: &str, candidate: &str) -> f64 {
    if typed == candidate && is_known_text(typed) {
        1.25
    } else {
        0.0
    }
}

fn flip_tokens(text: &str) -> String {
    text.split_whitespace()
        .map(|token| dict::convert(token, dict::detect_direction(token)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn moved_space_candidates(text: &str) -> Vec<String> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut out = Vec::new();
    for idx in 0..tokens.len().saturating_sub(1) {
        let left = tokens[idx];
        let right = tokens[idx + 1];
        if left.chars().count() < 2 || right.chars().count() < 2 {
            continue;
        }
        if is_ascii_technical_or_brand_token(left) {
            continue;
        }

        let mut right_chars = right.chars();
        let Some(moved) = right_chars.next() else {
            continue;
        };
        let repaired_left = format!("{left}{moved}");
        let repaired_right = right_chars.collect::<String>();
        if repaired_right.is_empty() {
            continue;
        }

        let mut candidate = tokens.clone();
        candidate[idx] = &repaired_left;
        candidate[idx + 1] = &repaired_right;
        out.push(candidate.join(" "));
    }
    out
}

fn repeated_letter_candidates(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    for idx in 1..chars.len() {
        if chars[idx] != chars[idx - 1] {
            continue;
        }
        let mut candidate = String::with_capacity(text.len());
        for (pos, ch) in chars.iter().enumerate() {
            if pos != idx {
                candidate.push(*ch);
            }
        }
        out.push(candidate);
    }
    out
}

fn trim_token(token: &str) -> &str {
    token.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-')
}

fn is_short_russian_function_word(token: &str) -> bool {
    matches!(
        token,
        "а" | "в" | "и" | "к" | "о" | "с" | "у" | "я" | "не" | "на" | "по" | "за" | "для"
    )
}

fn is_known_text(text: &str) -> bool {
    let mut saw_token = false;
    for token in text.split_whitespace().map(trim_token) {
        if token.is_empty() {
            continue;
        }
        saw_token = true;
        if !is_plausible_token(token) {
            return false;
        }
    }
    saw_token
}

fn is_plausible_token(token: &str) -> bool {
    !is_layout_garbage_token(token) && is_plausible_non_layout_token(token)
}

fn is_plausible_non_layout_token(token: &str) -> bool {
    is_known_word(token)
        || is_ascii_technical_or_brand_token(token)
        || is_natural_hyphenated_token(token)
}

fn is_natural_hyphenated_token(token: &str) -> bool {
    let mut saw_part = false;
    let mut alpha_parts = 0usize;
    let mut known_parts = 0usize;
    let mut long_parts = 0usize;
    for part in token.split('-') {
        if part.is_empty() {
            return false;
        }
        saw_part = true;
        let alpha_count = part.chars().filter(|ch| ch.is_alphabetic()).count();
        if alpha_count == 0 || alpha_count != part.chars().count() {
            return false;
        }
        if alpha_count >= 2 {
            alpha_parts += 1;
        }
        if alpha_count >= 3 {
            long_parts += 1;
        }
        if is_known_word(part) {
            known_parts += 1;
        }
    }
    saw_part && alpha_parts >= 2 && (known_parts > 0 || long_parts > 0)
}

fn lexical_bonus(token: &str) -> f64 {
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        return 0.0;
    }
    if is_known_word(token) {
        return 1.15;
    }
    if token.contains('-')
        && token
            .split('-')
            .filter(|part| !part.is_empty())
            .all(is_known_word)
    {
        return 0.90;
    }
    if is_natural_hyphenated_token(token) {
        return 0.20;
    }
    if token.chars().any(|ch| ch.is_alphabetic()) {
        return -0.55;
    }
    0.0
}

fn is_layout_garbage_token(token: &str) -> bool {
    if is_known_word(token)
        || (token.chars().any(is_cyrillic_letter) && is_natural_hyphenated_token(token))
    {
        return false;
    }
    if token.chars().filter(|ch| ch.is_alphabetic()).count() < 3 {
        return false;
    }
    let converted = dict::convert(token, dict::detect_direction(token));
    converted != token && is_plausible_non_layout_token(&converted)
}

fn is_known_word(token: &str) -> bool {
    let lower = token.to_lowercase();
    if lower.chars().all(is_cyrillic_letter) {
        return ru_words().contains(&lower) || is_known_ru_form(&lower);
    }
    if lower.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return en_words().contains(&lower);
    }
    false
}

fn is_known_ru_form(word: &str) -> bool {
    if word.chars().count() < 4 {
        return false;
    }
    if let Some(stem) = word.strip_suffix('о') {
        return ["ый", "ий", "ой"]
            .iter()
            .any(|suffix| ru_words().contains(&format!("{stem}{suffix}")));
    }
    for suffix in [
        "ками", "ках", "кой", "ки", "ке", "ку", "ок", "ыми", "ими", "ами", "ями", "ого", "его",
        "ому", "ему", "ов", "ев", "ей", "ах", "ях", "ам", "ям", "ом", "ем", "ой", "ый", "ий", "ая",
        "яя", "ое", "ее", "ые", "ие",
    ] {
        let Some(stem) = word.strip_suffix(suffix) else {
            continue;
        };
        let min_stem_len = if matches!(suffix, "ками" | "ках" | "кой" | "ки" | "ке" | "ку" | "ок")
        {
            3
        } else {
            4
        };
        if stem.chars().count() >= min_stem_len && ru_words().contains(stem) {
            return true;
        }
        if stem.chars().count() >= min_stem_len && ru_words().contains(&format!("{stem}ка")) {
            return true;
        }
    }
    for (ending, lemmas) in [
        ("шу", &["сать"][..]),
        ("ешь", &["ить", "еть"][..]),
        ("ишь", &["ить", "еть"]),
        ("ет", &["ить", "еть"]),
        ("ит", &["ить", "еть"]),
        ("ется", &["ться"]),
        ("ются", &["ться"]),
        ("ил", &["ить"]),
        ("ила", &["ить"]),
        ("или", &["ить"]),
        ("ал", &["ать"]),
        ("ала", &["ать"]),
        ("али", &["ать"]),
    ] {
        let Some(stem) = word.strip_suffix(ending) else {
            continue;
        };
        if lemmas
            .iter()
            .any(|lemma_suffix| ru_words().contains(&format!("{stem}{lemma_suffix}")))
        {
            return true;
        }
    }
    false
}

fn ru_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        load_hunspell_words(RU_HUNSPELL, |word| {
            word.chars().count() >= 2 && word.chars().all(is_cyrillic_letter)
        })
    })
}

fn en_words() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_hunspell_words(EN_HUNSPELL, |word| {
            word.chars().count() >= 2 && word.chars().all(|ch| ch.is_ascii_alphabetic())
        });
        if let Ok(text) = std::fs::read_to_string(EN_WORDS) {
            words.extend(
                text.lines()
                    .map(str::trim)
                    .map(str::to_lowercase)
                    .filter(|word| {
                        word.chars().count() >= 2 && word.chars().all(|ch| ch.is_ascii_alphabetic())
                    }),
            );
        }
        words
    })
}

fn load_hunspell_words(path: &str, keep: fn(&str) -> bool) -> HashSet<String> {
    std::fs::read_to_string(path)
        .map(|text| {
            text.lines()
                .skip(1)
                .filter_map(|line| line.split('/').next())
                .map(str::trim)
                .map(str::to_lowercase)
                .filter(|word| keep(word))
                .collect()
        })
        .unwrap_or_default()
}

fn build_cases(target: usize) -> Vec<Case> {
    let ru = [
        "привет",
        "слово",
        "проверка",
        "сегодня",
        "можно",
        "сделать",
        "текст",
        "ошибка",
        "быстро",
        "точно",
        "пишу",
        "дальше",
        "работает",
        "проект",
        "клавиатура",
        "раскладка",
        "помощник",
        "магазин",
        "документ",
        "таблица",
        "данные",
        "система",
        "окно",
        "фраза",
        "режим",
        "демон",
        "кнопка",
        "меню",
        "правильно",
        "хорошо",
        "плохо",
        "новый",
        "старый",
        "важно",
        "сейчас",
        "потом",
        "завтра",
        "вопрос",
        "ответ",
        "пример",
        "тест",
    ];
    let en = [
        "hello", "world", "test", "good", "double", "shift", "linux", "gnome", "wayland", "rust",
        "cargo", "github", "browser", "window", "file", "system", "model", "score", "layout",
        "keyboard", "input", "text", "word", "quick", "smart", "safe", "fast", "slow", "open",
        "close",
    ];

    let mut cases = Vec::with_capacity(target);
    for i in 0..target {
        let kind_idx = i % 16;
        let ru_a = ru[i % ru.len()];
        let ru_b = ru[(i * 7 + 3) % ru.len()];
        let en_a = en[(i * 5 + 1) % en.len()];
        let en_b = en[(i * 11 + 2) % en.len()];
        let tech = [
            "wi-fi", "API", "AmoCRM", "GitHub", "NTFS", "Linux", "JSON", "USB-C",
        ][i % 8];
        let brand = ["AmoCRM", "GitHub", "NTFS", "Linux", "JSON", "USB-C"][i % 6];
        let natural_hyphen = [
            "код-дэ-вуар",
            "пара-пара",
            "рок-н-ролл",
            "чек-лист",
            "тест-кейс",
            "интернет-магазин",
        ][i % 6];
        let mut case = match kind_idx {
            0 => {
                let expected = format!("{ru_a} {ru_b}");
                Case {
                    kind: "ru_wrong_layout",
                    typed: dict::convert(&expected, Direction::Ru2Us),
                    expected,
                }
            }
            1 => {
                let expected = format!("{en_a} {en_b}");
                Case {
                    kind: "en_wrong_layout",
                    typed: dict::convert(&expected, Direction::Us2Ru),
                    expected,
                }
            }
            2 => {
                let expected = format!("{en_a} {ru_a}");
                Case {
                    kind: "mixed_en_ru",
                    typed: format!("{en_a} {}", dict::convert(ru_a, Direction::Ru2Us)),
                    expected,
                }
            }
            3 => {
                let expected = format!("{ru_a} {en_a}");
                Case {
                    kind: "mixed_ru_en",
                    typed: format!("{ru_a} {}", dict::convert(en_a, Direction::Us2Ru)),
                    expected,
                }
            }
            4 => Case {
                kind: "transpose",
                typed: transpose_middle(ru_a),
                expected: ru_a.to_string(),
            },
            5 => Case {
                kind: "missing_letter",
                typed: drop_middle(ru_a),
                expected: ru_a.to_string(),
            },
            6 => Case {
                kind: "extra_letter",
                typed: duplicate_middle(ru_a),
                expected: ru_a.to_string(),
            },
            7 => {
                let expected = format!("{ru_a} {ru_b}");
                Case {
                    kind: "split_word",
                    typed: split_last_char_to_next(ru_a, ru_b),
                    expected,
                }
            }
            8 => {
                let expected = if i % 20 == 8 {
                    "я тут".to_string()
                } else {
                    format!("{ru_a} {ru_b}")
                };
                Case {
                    kind: "glued_words",
                    typed: expected.replace(' ', ""),
                    expected,
                }
            }
            9 => {
                let expected = if i % 2 == 0 {
                    format!("{ru_a} {en_a}")
                } else {
                    format!("{ru_a} {ru_b}")
                };
                Case {
                    kind: "keep_valid",
                    typed: expected.clone(),
                    expected,
                }
            }
            10 => {
                let expected = format!("{tech} {ru_a}");
                Case {
                    kind: "technical_keep",
                    typed: expected.clone(),
                    expected,
                }
            }
            11 => {
                let expected = format!("{tech} {ru_a}");
                Case {
                    kind: "technical_mixed_ru",
                    typed: format!("{tech} {}", dict::convert(ru_a, Direction::Ru2Us)),
                    expected,
                }
            }
            12 => Case {
                kind: "brand_letter",
                typed: format!("{brand} Z"),
                expected: format!("{brand} Я"),
            },
            13 => Case {
                kind: "technical_layout_token",
                typed: dict::convert(tech, Direction::Us2Ru),
                expected: tech.to_string(),
            },
            14 => Case {
                kind: "hyphen_keep",
                typed: natural_hyphen.to_string(),
                expected: natural_hyphen.to_string(),
            },
            _ => {
                let expected = format!("{ru_a}, {en_a}");
                Case {
                    kind: "punctuation_mixed",
                    typed: format!("{ru_a}, {}", dict::convert(en_a, Direction::Us2Ru)),
                    expected,
                }
            }
        };
        mark_ambiguous_valid_typo_as_keep(&mut case);
        cases.push(case);
    }
    cases
}

fn mark_ambiguous_valid_typo_as_keep(case: &mut Case) {
    if !matches!(case.kind, "transpose" | "missing_letter" | "extra_letter") {
        return;
    }
    if case.typed == case.expected || !is_known_text(&case.typed) {
        return;
    }
    case.kind = "ambiguous_typo_keep";
    case.expected = case.typed.clone();
}

fn transpose_middle(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() > 4 {
        let idx = chars.len() / 2 - 1;
        chars.swap(idx, idx + 1);
    }
    chars.into_iter().collect()
}

fn drop_middle(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() > 4 {
        let idx = chars.len() / 2;
        chars.remove(idx);
    }
    chars.into_iter().collect()
}

fn duplicate_middle(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() > 3 {
        let idx = chars.len() / 2;
        chars.insert(idx, chars[idx]);
    }
    chars.into_iter().collect()
}

fn split_last_char_to_next(left: &str, right: &str) -> String {
    let mut left_chars: Vec<char> = left.chars().collect();
    let Some(moved) = left_chars.pop() else {
        return format!("{left} {right}");
    };
    format!(
        "{} {}{}",
        left_chars.into_iter().collect::<String>(),
        moved,
        right
    )
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let idx = ((values.len() - 1) as f64 * p).round() as usize;
    values[idx]
}
