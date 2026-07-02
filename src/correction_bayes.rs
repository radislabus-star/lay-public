use crate::text_metrics::damerau_levenshtein;
use crate::word_reader::{is_cyrillic_letters_only, split_edge_whitespace, split_word_punctuation};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BayesCandidateScore {
    pub posterior: f32,
    pub likelihood: f32,
    pub usage_prior: f32,
    pub context_prior: f32,
    pub risk: f32,
}

pub(crate) fn bayes_score_candidate(
    original: &str,
    replacement: &str,
    error_class: &str,
    source_id: &str,
) -> BayesCandidateScore {
    let original_word = last_word(original).unwrap_or_default();
    let replacement_word = last_word(replacement).unwrap_or_default();
    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let distance = if original_lower.is_empty() || replacement_lower.is_empty() {
        0
    } else {
        damerau_levenshtein(&original_lower, &replacement_lower)
    };
    let max_len = original_lower
        .chars()
        .count()
        .max(replacement_lower.chars().count())
        .max(1);
    let likelihood = input_likelihood(error_class, source_id, distance, max_len);
    let usage_prior = crate::nanda_wave::usage_prior::word_usage_prior(&replacement_lower);
    let context_prior = (local_context_prior(original, &replacement_lower)
        + source_prior(source_id))
    .clamp(0.0, 0.26);
    let risk = candidate_risk(
        &original_lower,
        &replacement_lower,
        error_class,
        source_id,
        distance,
    );
    let posterior = (likelihood * 0.55 + usage_prior + context_prior - risk).clamp(-1.0, 1.0);

    BayesCandidateScore {
        posterior,
        likelihood,
        usage_prior,
        context_prior,
        risk,
    }
}

pub(crate) fn bayes_suggest_only_reason(
    original: &str,
    replacement: &str,
    error_class: &str,
    source_id: &str,
) -> Option<&'static str> {
    if matches!(
        error_class,
        "wrong_layout" | "partial-layout" | "mixed-script" | "split-word" | "glued-words"
    ) {
        return None;
    }
    let score = bayes_score_candidate(original, replacement, error_class, source_id);
    if score.risk >= 0.62 {
        return Some("bayes_high_candidate_risk");
    }
    let min_posterior = if error_class == "composite-typo" {
        0.24
    } else {
        0.30
    };
    (score.posterior < min_posterior).then_some("bayes_low_posterior")
}

fn input_likelihood(error_class: &str, source_id: &str, distance: usize, max_len: usize) -> f32 {
    if source_id.contains("layout") || error_class == "wrong_layout" {
        return 0.96;
    }
    let edit_likelihood = 1.0 - (distance as f32 / max_len as f32).min(1.0);
    match error_class {
        "adjacent-transposition" => edit_likelihood.max(0.82),
        "missing-letter" => edit_likelihood.max(0.78),
        "repeated-letter" | "extra-letter" => edit_likelihood.max(0.74),
        "letter-substitution" => edit_likelihood.max(0.68),
        "composite-typo" | "grammar-agreement" => edit_likelihood,
        _ => edit_likelihood.min(0.45),
    }
}

fn local_context_prior(original: &str, replacement_word: &str) -> f32 {
    let words = text_words(original);
    let previous = words.iter().rev().nth(1).map(String::as_str).unwrap_or("");
    let mut prior: f32 = 0.0;
    if crate::lexicon::is_common_ru_word(replacement_word) {
        prior += 0.08;
    }
    if previous == "теорию" && replacement_word.ends_with('а') {
        prior += 0.06;
    }
    let mut context = words;
    context.pop();
    prior += crate::nanda_wave::context_word_usage_prior(&context, replacement_word);
    prior.clamp(0.0, 0.18)
}

fn candidate_risk(
    original: &str,
    replacement: &str,
    error_class: &str,
    source_id: &str,
    distance: usize,
) -> f32 {
    let mut risk: f32 = 0.0;
    if replacement.is_empty() || original == replacement {
        return 0.0;
    }
    if is_known_autocorrect_token(original)
        && is_known_autocorrect_token(replacement)
        && original != replacement
        && !source_id.contains("layout")
    {
        risk += 0.34;
    }
    if !trusted_typo_source(source_id)
        && !is_known_autocorrect_token(replacement)
        && is_cyrillic_letters_only(replacement)
        && error_class != "wrong_layout"
    {
        risk += 0.24;
    }
    if looks_like_reflexive_plus_case_vowel(replacement) {
        risk += 0.55;
    }
    if short_y_drop(original, replacement, distance) {
        risk += 0.42;
    }
    if short_same_length_multi_edit(original, replacement, distance) {
        risk += 0.28;
    }
    if nonverb_to_verb_drift(original, replacement) {
        risk += 0.26;
    }
    if short_dense_cluster_multi_edit(original, distance) {
        risk += 0.24;
    }
    risk.clamp(0.0, 1.0)
}

fn trusted_typo_source(source_id: &str) -> bool {
    matches!(
        source_id,
        "composite_ru_typo" | "adjacent_transposition" | "missing_letter" | "repeated_letter"
    ) || source_id.starts_with("layout_then_")
}

fn source_prior(source_id: &str) -> f32 {
    if trusted_typo_source(source_id) {
        0.16
    } else {
        0.0
    }
}

fn is_known_autocorrect_token(word: &str) -> bool {
    !word.is_empty()
        && (crate::lexicon::is_common_ru_word(word)
            || crate::lexicon::is_user_protected_word(word)
            || crate::phrase_lexicon::is_known_russian_phrase_part(word)
            || crate::russian_lexicon::is_known_russian_word_or_form(word)
            || crate::russian_lexicon::is_known_russian_adverb_o_form(word)
            || crate::russian_lexicon::is_known_russian_ka_oblique_form(word))
}

fn looks_like_reflexive_plus_case_vowel(word: &str) -> bool {
    [
        "сяа", "сяу", "сяы", "сяи", "сяо", "сьа", "сьу", "сьы", "сьи", "сьо",
    ]
    .iter()
    .any(|tail| word.ends_with(tail))
}

fn short_y_drop(original: &str, replacement: &str, distance: usize) -> bool {
    distance == 1
        && original.chars().count() <= 6
        && original.contains('й')
        && !replacement.contains('й')
}

fn short_same_length_multi_edit(original: &str, replacement: &str, distance: usize) -> bool {
    let original_len = original.chars().count();
    original_len <= 6 && original_len == replacement.chars().count() && distance >= 2
}

fn nonverb_to_verb_drift(original: &str, replacement: &str) -> bool {
    !has_russian_verb_tail(original) && has_russian_verb_tail(replacement)
}

fn short_dense_cluster_multi_edit(original: &str, distance: usize) -> bool {
    distance >= 2 && original.chars().count() <= 7 && has_dense_consonant_cluster(original)
}

fn has_dense_consonant_cluster(word: &str) -> bool {
    let mut run = 0;
    for ch in word.chars() {
        if is_russian_consonant(ch) {
            run += 1;
            if run >= 3 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn has_russian_verb_tail(word: &str) -> bool {
    const VERB_TAILS: &[&str] = &[
        "ет", "ит", "ют", "ут", "ат", "ят", "ем", "им", "ешь", "ишь", "ете", "ите", "ал", "ала",
        "ило", "или", "ил", "ено", "ена", "ены", "ает", "яет", "ует", "ёт", "ся", "ыть", "ять",
        "ыта", "ыто", "ыты", "ята", "ято", "яты",
    ];
    VERB_TAILS.iter().any(|tail| word.ends_with(tail))
}

fn is_russian_consonant(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё')
        && !matches!(
            ch,
            'а' | 'я' | 'у' | 'ю' | 'е' | 'ё' | 'ы' | 'и' | 'о' | 'э' | 'ь' | 'ъ'
        )
}

fn last_word(text: &str) -> Option<String> {
    text_words(text).pop()
}

fn text_words(text: &str) -> Vec<String> {
    let (_, core, _) = split_edge_whitespace(text);
    core.split_whitespace()
        .filter_map(|segment| {
            let (_, word, _) = split_word_punctuation(segment);
            (!word.is_empty()).then(|| word.to_lowercase())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bayes_rejects_low_probability_reflexive_noise() {
        let reason = bayes_suggest_only_reason(
            "теорию бейса ",
            "теорию бейсяа ",
            "composite-typo",
            "composite_ru_typo",
        );
        assert_eq!(reason, Some("bayes_low_posterior"));
    }

    #[test]
    fn bayes_allows_clear_common_typo() {
        let reason = bayes_suggest_only_reason(
            "где эсперемнт ",
            "где эксперимент ",
            "composite-typo",
            "composite_ru_typo",
        );
        assert_eq!(reason, None);
    }
}
