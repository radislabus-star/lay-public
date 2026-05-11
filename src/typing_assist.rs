//! Typing-assist and smart correction decision layer.
//!
//! This module decides *what* text should be produced. The daemon owns only
//! physical input/output: evdev buffering, uinput replay, DBus text insertion,
//! and runtime status.

use evdev::KeyCode;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use crate::config::{
    default_typing_assist_pipeline, normalize_typing_assist_pipeline, CorrectionEngine,
    TypingAssistRuleConfig,
};
use crate::correction::Correction;
use crate::keyboard::{
    is_cyrillic_letter, map_events_to_layout, map_original_events,
    mixed_visual_latin_word_target_layout, original_event_char, replay_layout_decision,
    split_event_words, KeyEvent,
};
use crate::word_buffer::{WordBuffer, MAX_REPLACE_WORDS};

pub const REPLACEMENTS_PATH: &str = ".config/lay/replacements.json";
const PROTECTED_WORDS_PATH: &str = ".config/lay/protected_words.txt";

const NGRAM_TYPO_REJECT_MARGIN: f64 = 0.25;
const NGRAM_TRANSPOSE_MARGIN: f64 = -8.0;
const NGRAM_SPLIT_REJECT_MARGIN: f64 = 0.25;
const NGRAM_NODICT_SPLIT_REJECT_MARGIN: f64 = 1.0;
const NGRAM_DICT_MISSING_LETTER_MARGIN: f64 = -8.0;
const NGRAM_MISSING_LETTER_MARGIN: f64 = 1.5;
const NGRAM_EXTRA_LETTER_MARGIN: f64 = 0.75;
const NGRAM_VOWEL_CONFUSION_MARGIN: f64 = -1.0;
const NGRAM_VERB_ENDING_MARGIN: f64 = -8.0;
const NGRAM_HARD_SIGN_MARGIN: f64 = 1.0;
const NGRAM_MOVED_PREFIX_MARGIN: f64 = 0.5;
const NGRAM_MOVED_PREFIX_RIGHT_MARGIN: f64 = 5.0;
const NGRAM_GLUED_SPLIT_MARGIN: f64 = -0.25;
const LEM_LAYOUT_AUTOSWITCH_MARGIN: f64 = 0.25;
const RU_ALPHABET: [char; 33] = [
    'а', 'б', 'в', 'г', 'д', 'е', 'ё', 'ж', 'з', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', 'р', 'с',
    'т', 'у', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я',
];
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
const COMMON_SHORT_ENGLISH_LAYOUT_WORDS: &[&str] = &[
    "api", "css", "cpu", "eng", "git", "gpu", "json", "llm", "md", "pdf", "ram", "rus", "sql",
    "ssd", "ssh", "usb", "zip",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedTailOptions {
    pub lem_enabled: bool,
    pub allow_layout_auto: bool,
}

impl Default for ScopedTailOptions {
    fn default() -> Self {
        Self {
            lem_enabled: false,
            allow_layout_auto: true,
        }
    }
}

fn should_expand_auto_replace_context(buf: &WordBuffer) -> bool {
    let Some((events, _)) = buf.what_to_replay(2) else {
        return false;
    };
    contains_visual_b_word(&map_original_events(&events))
}

pub fn apply_auto_replace(original: &str, target: &str) -> Option<String> {
    let (target_leading, target_core, target_trailing) = split_edge_whitespace(target);
    if target_core.is_empty() {
        return None;
    }

    if let Some(visual) = replace_visual_b_in_context(original, target) {
        return Some(visual);
    }

    replacement_for_token(target_core).map(|replacement| {
        let mut out = String::with_capacity(target.len().max(replacement.len()));
        out.push_str(target_leading);
        out.push_str(&replacement);
        out.push_str(target_trailing);
        out
    })
}

pub fn apply_typing_assist_exact(text: &str) -> Option<String> {
    apply_typing_assist_with_pipeline(text, false, &default_typing_assist_pipeline())
}

pub fn apply_typing_assist(text: &str, allow_layout_auto: bool) -> Option<String> {
    let pipeline = default_typing_assist_pipeline();
    apply_typing_assist_with_pipeline(text, allow_layout_auto, &pipeline)
}

pub fn apply_typing_assist_with_pipeline(
    text: &str,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<String> {
    let (leading, core, trailing) = split_edge_whitespace(text);
    if core.is_empty() {
        return None;
    }

    let (token_leading, word, token_trailing) = split_word_punctuation(core);
    let rules = normalize_typing_assist_pipeline(pipeline);
    let replacement = rules.iter().filter(|rule| rule.enabled).find_map(|rule| {
        apply_typing_assist_rule(
            &rule.id,
            core,
            word,
            token_leading,
            token_trailing,
            allow_layout_auto,
        )
    })?;

    let mut out = String::with_capacity(text.len().max(replacement.len()));
    out.push_str(leading);
    out.push_str(&replacement);
    out.push_str(trailing);
    (out != text).then_some(out)
}

fn apply_typing_assist_rule(
    id: &str,
    core: &str,
    word: &str,
    token_leading: &str,
    token_trailing: &str,
    allow_layout_auto: bool,
) -> Option<String> {
    match id {
        "moved_prefix_pair" => correct_moved_prefix_letter_pair(core),
        "split_word_pair" => correct_split_word_pair(core),
        "visual_b" => replace_visual_b_words(core, core),
        "personal_phrase" => replacement_for_token(core),
        "personal_token" => word_rule(word, token_leading, token_trailing, replacement_for_token),
        "duplicate_layout_prefix" => word_rule(
            word,
            token_leading,
            token_trailing,
            correct_duplicate_layout_prefix_on_ascii_token,
        ),
        "layout_technical" => word_rule(
            word,
            token_leading,
            token_trailing,
            correct_wrong_layout_ascii_technical_token,
        ),
        "layout_ru_to_en" if allow_layout_auto => {
            correct_wrong_layout_cyrillic_word(core).or_else(|| {
                word_rule(
                    word,
                    token_leading,
                    token_trailing,
                    correct_wrong_layout_cyrillic_word,
                )
            })
        }
        "layout_en_to_ru" if allow_layout_auto => {
            if let Some(replacement) = correct_wrong_layout_ascii_word(core) {
                Some(replacement)
            } else if ascii_layout_prefix_can_be_letter(token_leading) {
                None
            } else {
                word_rule(
                    word,
                    token_leading,
                    token_trailing,
                    correct_wrong_layout_ascii_word,
                )
            }
        }
        "cyrillic_case" => word_rule(
            word,
            token_leading,
            token_trailing,
            correct_cyrillic_word_case,
        ),
        "hard_sign" => word_rule(word, token_leading, token_trailing, correct_hard_sign_typo),
        "adjacent_transposition" => word_rule(
            word,
            token_leading,
            token_trailing,
            correct_adjacent_transposition,
        ),
        "repeated_letter" => {
            word_rule(word, token_leading, token_trailing, correct_repeated_letter)
        }
        "single_letter_substitution" => word_rule(
            word,
            token_leading,
            token_trailing,
            correct_single_letter_substitution,
        ),
        "verb_ending" => word_rule(
            word,
            token_leading,
            token_trailing,
            correct_verb_ending_confusion,
        ),
        "vowel_confusion" => {
            word_rule(word, token_leading, token_trailing, correct_vowel_confusion)
        }
        "extra_letters" => word_rule(word, token_leading, token_trailing, correct_extra_letters),
        "missing_letter" => word_rule(word, token_leading, token_trailing, correct_missing_letter),
        "glued_phrase" => word_rule(
            word,
            token_leading,
            token_trailing,
            correct_glued_russian_phrase,
        ),
        _ => None,
    }
}

fn word_rule(
    word: &str,
    token_leading: &str,
    token_trailing: &str,
    f: fn(&str) -> Option<String>,
) -> Option<String> {
    if word.is_empty() {
        return None;
    }
    f(word).map(|replacement| format!("{token_leading}{replacement}{token_trailing}"))
}

fn correct_wrong_layout_ascii_word(token: &str) -> Option<String> {
    if !is_plain_ascii_layout_token(token) || is_protected_ascii_layout_token(token) {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(token);
    if original_word.is_empty() {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Us2Ru);
    if converted == token {
        return None;
    }

    let (converted_leading, converted_word, converted_trailing) =
        split_word_punctuation(&converted);
    if converted_word.is_empty() || !is_cyrillic_word(converted_word) {
        return None;
    }

    let converted_lower = converted_word.to_lowercase();
    if !is_known_russian_layout_autoswitch_word(&converted_lower) {
        return None;
    }

    let normalized_word = apply_word_case(original_word, &converted_lower);
    let normalized = format!("{converted_leading}{normalized_word}{converted_trailing}");
    match crate::llm::choose_token_hybrid(original_word, &normalized_word) {
        Ok(Some(choice)) if choice == normalized_word => Some(normalized),
        Ok(Some(choice)) if choice == original_word => {
            allow_short_layout_word(original_word, &converted_lower).then_some(normalized)
        }
        _ => Some(normalized),
    }
}

fn ascii_layout_prefix_can_be_letter(prefix: &str) -> bool {
    prefix
        .chars()
        .any(|ch| matches!(ch, '\'' | ';' | '[' | ']' | '`' | ',' | '.'))
}

fn correct_wrong_layout_cyrillic_word(token: &str) -> Option<String> {
    if !is_plain_cyrillic_layout_token(token) {
        return None;
    }

    let (_, original_word, _) = split_word_punctuation(token);
    if original_word.is_empty() {
        return None;
    }

    let original_lower = original_word.to_lowercase();
    if is_known_russian_layout_autoswitch_word(&original_lower) {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Ru2Us);
    if converted == token {
        return None;
    }

    let (converted_leading, converted_word, converted_trailing) =
        split_word_punctuation(&converted);
    if converted_word.is_empty() || !is_plain_ascii_word_candidate(converted_word) {
        return None;
    }

    english_layout_autoswitch_candidates(converted_word)
        .into_iter()
        .find_map(|candidate_lower| {
            let candidate_word = apply_word_case(original_word, &candidate_lower);
            let candidate = format!("{converted_leading}{candidate_word}{converted_trailing}");
            lem_prefers_layout_candidate(original_word, &candidate_word).then_some(candidate)
        })
}

fn english_layout_autoswitch_candidates(converted: &str) -> Vec<String> {
    let lower = converted.to_ascii_lowercase();
    let mut out = Vec::new();
    if is_known_english_layout_autoswitch_word(&lower) {
        out.push(lower.clone());
    }

    let chars: Vec<char> = lower.chars().collect();
    if chars.len() >= 6 {
        let without_prefix: String = chars[1..].iter().collect();
        if !is_known_english_layout_autoswitch_word(&lower)
            && is_known_english_layout_autoswitch_word(&without_prefix)
        {
            out.push(without_prefix);
        }
    }

    out
}

fn lem_prefers_layout_candidate(typed: &str, candidate: &str) -> bool {
    let ranked = crate::lem::rank_candidates(typed, [typed.to_string(), candidate.to_string()]);
    let Some(best) = ranked.first() else {
        return false;
    };
    if best.text != candidate {
        return false;
    }

    let margin = ranked
        .get(1)
        .map(|second| best.total - second.total)
        .unwrap_or(f64::INFINITY);
    margin >= LEM_LAYOUT_AUTOSWITCH_MARGIN
}

fn is_plain_cyrillic_layout_token(token: &str) -> bool {
    token.chars().any(is_cyrillic_letter)
        && token.chars().all(|ch| {
            is_cyrillic_letter(ch)
                || matches!(
                    ch,
                    ',' | '.' | '!' | '?' | ':' | ';' | '$' | '%' | '&' | '#' | '@' | '-' | '_'
                )
        })
}

fn is_plain_ascii_layout_token(token: &str) -> bool {
    token.is_ascii()
        && token.chars().any(|ch| ch.is_ascii_alphabetic())
        && !token.chars().any(|ch| ch.is_ascii_digit())
        && token.chars().all(|ch| {
            ch.is_ascii_alphabetic()
                || matches!(
                    ch,
                    ',' | ';'
                        | '\''
                        | '['
                        | ']'
                        | '`'
                        | '.'
                        | '/'
                        | '?'
                        | '!'
                        | ':'
                        | '$'
                        | '%'
                        | '^'
                        | '&'
                        | '#'
                        | '@'
                        | '-'
                        | '_'
                )
        })
}

fn is_protected_ascii_layout_token(token: &str) -> bool {
    token.chars().any(|ch| ch.is_ascii_alphabetic())
        && (is_upper_ascii_layout_acronym(token) || is_mixed_case_ascii_layout_brand(token))
}

fn is_upper_ascii_layout_acronym(token: &str) -> bool {
    let letters: Vec<char> = token
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect();
    (2..=4).contains(&letters.len()) && letters.iter().all(|ch| ch.is_ascii_uppercase())
}

fn is_mixed_case_ascii_layout_brand(token: &str) -> bool {
    let letters: Vec<char> = token
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect();
    letters.len() >= 4
        && letters.iter().any(|ch| ch.is_ascii_lowercase())
        && letters.iter().skip(1).any(|ch| ch.is_ascii_uppercase())
}

fn allow_short_layout_word(original: &str, converted_lower: &str) -> bool {
    original
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .count()
        <= 3
        && russian_tiny_dictionary().contains(converted_lower)
}

fn is_known_russian_layout_autoswitch_word(word: &str) -> bool {
    let len = word.chars().filter(|ch| is_cyrillic_letter(*ch)).count();
    if len <= 3 {
        return russian_tiny_dictionary().contains(word);
    }

    is_known_russian_word_or_form(word)
        || is_known_russian_adverb_o_form(word)
        || is_known_russian_ka_oblique_form(word)
        || russian_short_dictionary().contains(word)
}

fn is_known_english_layout_autoswitch_word(word: &str) -> bool {
    let len = word.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
    if len < 4 {
        return COMMON_SHORT_ENGLISH_LAYOUT_WORDS.contains(&word);
    }
    english_dictionary().contains(word)
}

fn is_plain_ascii_word_candidate(token: &str) -> bool {
    token.is_ascii()
        && token.chars().any(|ch| ch.is_ascii_alphabetic())
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || ch == '-')
}

fn replacement_for_token(token: &str) -> Option<String> {
    if let Some(replacement) = promoted_replacement_for_token(token) {
        return Some(replacement);
    }

    if let Some(replacement) = replacement_rules().get(token) {
        return Some(replacement.clone());
    }

    let lower = token.to_lowercase();
    if lower == token {
        return None;
    }
    replacement_rules()
        .get(&lower)
        .map(|replacement| apply_phrase_case(token, replacement))
}

pub fn promoted_replacement_for_token(token: &str) -> Option<String> {
    let rules = promoted_replacement_rules().lock().ok()?;
    if let Some(replacement) = rules.get(token) {
        return Some(replacement.clone());
    }

    let lower = token.to_lowercase();
    if lower == token {
        return None;
    }
    rules
        .get(&lower)
        .map(|replacement| apply_phrase_case(token, replacement))
}

fn promoted_replacement_rules() -> &'static Mutex<HashMap<String, String>> {
    static RULES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    RULES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn remember_promoted_replacement(from: &str, to: &str) {
    if let Ok(mut rules) = promoted_replacement_rules().lock() {
        rules.insert(from.to_string(), to.to_string());
    }
}

pub fn correct_duplicate_layout_prefix_on_ascii_token(token: &str) -> Option<String> {
    let mut chars = token.chars();
    let first = chars.next()?;
    if !is_cyrillic_letter(first) {
        return None;
    }

    let rest: String = chars.collect();
    if !is_ascii_technical_token(&rest) {
        return None;
    }

    let mapped = crate::dict::convert(&first.to_string(), crate::dict::Direction::Ru2Us);
    let mut mapped_chars = mapped.chars();
    let mapped = mapped_chars.next()?;
    if mapped_chars.next().is_some() {
        return None;
    }

    let rest_first = rest.chars().next()?;
    if rest_first.is_ascii_alphabetic() && mapped.eq_ignore_ascii_case(&rest_first) {
        Some(rest)
    } else {
        None
    }
}

pub fn correct_wrong_layout_ascii_technical_token(token: &str) -> Option<String> {
    if !token.contains('-') {
        return None;
    }
    if !token.chars().any(is_cyrillic_letter) || token.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    if !token
        .chars()
        .all(|ch| is_cyrillic_letter(ch) || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.'))
    {
        return None;
    }

    let converted = crate::dict::convert(token, crate::dict::Direction::Ru2Us);
    if converted == token || !is_ascii_technical_token(&converted) {
        return None;
    }
    if !has_clear_ascii_technical_layout_signal(&converted) {
        return None;
    }

    let has_clear_separator = converted.contains('-');
    let has_short_ascii_segment = converted
        .split(['-', '_', '.'])
        .any(|part| (2..=4).contains(&part.chars().count()));
    let original_known_hyphen_word =
        token.contains('-') && is_cyrillic_hyphenated_word_for_layout(token);

    if has_clear_separator && has_short_ascii_segment && !original_known_hyphen_word {
        Some(converted)
    } else {
        None
    }
}

fn has_clear_ascii_technical_layout_signal(token: &str) -> bool {
    let alpha_total = token.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
    let alpha_segment = token
        .split(['-', '_', '.', '@', '/', '\\', ':', '+', '#'])
        .any(|part| part.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 2);

    alpha_total >= 4 && alpha_segment
}

fn correct_glued_russian_phrase(word: &str) -> Option<String> {
    let char_len = word.chars().count();
    if !(4..=24).contains(&char_len) || !word.chars().all(is_cyrillic_letter) {
        return None;
    }

    let lower = word.to_lowercase();
    if russian_dictionary().contains(&lower)
        || russian_generated_form_dictionary().contains(&lower)
        || (is_known_russian_word_or_form(&lower) && !looks_like_word_glued_to_trailing_ya(&lower))
    {
        return None;
    }

    let mut best: Option<(String, f64)> = None;
    let mut second_best = f64::NEG_INFINITY;
    for split_at in lower
        .char_indices()
        .skip(1)
        .map(|(idx, _)| idx)
        .take(char_len.saturating_sub(1))
    {
        let (left, right) = lower.split_at(split_at);
        let left_len = left.chars().count();
        let right_len = right.chars().count();
        let short_left_pronoun = left_len == 1 && is_single_letter_russian_pronoun(left);
        let short_right_function =
            right_len == 1 && left_len >= 4 && is_single_letter_russian_pronoun(right);
        if left_len == 1 && !short_left_pronoun {
            continue;
        }
        if (right_len < 3 && !short_right_function) || left_len > 8 {
            continue;
        }
        if !is_known_russian_phrase_part(left) || !is_known_russian_phrase_part(right) {
            continue;
        }

        let candidate = format!("{left} {right}");
        let margin = crate::ngram::ru_candidate_margin(&candidate, &lower);
        if !is_confident_glued_phrase_split(left, right) && margin < NGRAM_GLUED_SPLIT_MARGIN {
            continue;
        }

        match &best {
            Some((_, best_margin)) if margin <= *best_margin => {
                second_best = second_best.max(margin);
            }
            Some((_, best_margin)) => {
                second_best = second_best.max(*best_margin);
                best = Some((candidate, margin));
            }
            None => best = Some((candidate, margin)),
        }
    }

    let (candidate, best_margin) = best?;
    if best_margin - second_best < 0.40 {
        return None;
    }
    Some(apply_phrase_case(word, &candidate))
}

fn looks_like_word_glued_to_trailing_ya(word: &str) -> bool {
    let Some(left) = word.strip_suffix('я') else {
        return false;
    };
    left.chars().count() >= 4 && is_known_russian_phrase_part(left)
}

fn is_known_russian_phrase_part(word: &str) -> bool {
    let len = word.chars().count();
    if len == 1 {
        return is_one_letter_russian_function_word(word);
    }
    if len <= 3 {
        return russian_tiny_dictionary().contains(word)
            || russian_short_dictionary().contains(word);
    }
    is_known_russian_word_or_form(word)
        || is_known_russian_adverb_o_form(word)
        || is_known_russian_ka_oblique_form(word)
        || russian_short_dictionary().contains(word)
}

fn is_one_letter_russian_function_word(word: &str) -> bool {
    matches!(word, "а" | "в" | "и" | "к" | "о" | "с" | "у" | "я")
}

fn is_single_letter_russian_pronoun(word: &str) -> bool {
    word == "я"
}

fn is_confident_glued_phrase_split(left: &str, right: &str) -> bool {
    (left.chars().count() == 1 && is_single_letter_russian_pronoun(left))
        || (right.chars().count() == 1
            && left.chars().count() >= 4
            && is_single_letter_russian_pronoun(right))
        || (left.chars().count() >= 4
            && right.chars().count() >= 4
            && is_known_russian_adverb_o_form(right))
}

pub fn should_keep_plain_cyrillic_before_ascii_technical(original: &str, converted: &str) -> bool {
    original.chars().count() >= 4
        && original.chars().all(is_cyrillic_letter)
        && converted != original
        && is_ascii_technical_token(converted)
}

pub fn is_ascii_technical_token(token: &str) -> bool {
    token.is_ascii()
        && token.chars().any(|ch| ch.is_ascii_alphabetic())
        && token.chars().all(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(ch, '-' | '_' | '.' | '@' | '/' | '\\' | ':' | '+' | '#')
        })
        && token
            .chars()
            .any(|ch| matches!(ch, '-' | '_' | '.' | '@' | '/' | '\\' | ':' | '+' | '#'))
}

fn correct_split_word_pair(text: &str) -> Option<String> {
    let segments = split_ws_segments(text);
    if segments.len() != 3 || segments[0].1 || !segments[1].1 || segments[2].1 {
        return None;
    }

    let (left_leading, left, left_trailing) = split_word_punctuation(segments[0].0);
    let (right_leading, right, right_trailing) = split_word_punctuation(segments[2].0);
    if !left_leading.is_empty()
        || !left_trailing.is_empty()
        || !right_leading.is_empty()
        || left.is_empty()
        || right.is_empty()
    {
        return None;
    }

    let left_lower = left.to_lowercase();
    let right_lower = right.to_lowercase();
    if is_known_russian_phrase_part(&left_lower)
        && is_one_letter_russian_function_word(&right_lower)
    {
        return None;
    }

    let glued = format!("{left}{right}");
    if glued.chars().count() < 4 || !is_cyrillic_word(&glued) {
        return None;
    }

    let lower = glued.to_lowercase();
    if !is_known_russian_word_or_form(&lower)
        && !can_merge_split_without_dictionary(left, right, &lower, text)
    {
        return None;
    }
    if !ngram_allows_ru_candidate(&lower, text, NGRAM_SPLIT_REJECT_MARGIN) {
        return None;
    }

    Some(format!(
        "{}{}",
        apply_word_case(&glued, &lower),
        right_trailing
    ))
}

fn can_merge_split_without_dictionary(
    left: &str,
    right: &str,
    glued_lower: &str,
    text: &str,
) -> bool {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    let glued_len = glued_lower.chars().count();
    if russian_short_dictionary().contains(&right.to_lowercase()) {
        return false;
    }

    (2..=3).contains(&right_len)
        && left_len == 1
        && left.eq_ignore_ascii_case("я")
        && glued_len >= 4
        && crate::ngram::ru_candidate_margin(glued_lower, text) >= NGRAM_NODICT_SPLIT_REJECT_MARGIN
}

fn correct_moved_prefix_letter_pair(text: &str) -> Option<String> {
    let segments = split_ws_segments(text);
    if segments.len() != 3 || segments[0].1 || !segments[1].1 || segments[2].1 {
        return None;
    }

    let (left_leading, left, left_trailing) = split_word_punctuation(segments[0].0);
    let (right_leading, right, right_trailing) = split_word_punctuation(segments[2].0);
    if !left_leading.is_empty()
        || !left_trailing.is_empty()
        || !right_leading.is_empty()
        || left.is_empty()
        || right.chars().count() < 2
    {
        return None;
    }

    let mut right_chars = right.chars();
    let moved = right_chars.next()?;
    if is_known_russian_word_or_form(&right.to_lowercase()) {
        return None;
    }
    let right_rest: String = right_chars.collect();
    let left_candidate = format!("{left}{moved}");
    let candidate = format!("{left_candidate} {right_rest}");

    if !is_cyrillic_word(&left_candidate) || !is_cyrillic_word(&right_rest) {
        return None;
    }

    let left_candidate_lower = left_candidate.to_lowercase();
    let right_rest_lower = right_rest.to_lowercase();
    let right_lower = right.to_lowercase();
    let short_right_is_safe = is_safe_short_moved_prefix_right(&right_rest_lower)
        && !is_known_russian_word_or_form(&right_lower);

    if left_candidate.chars().count() >= 5
        && short_right_is_safe
        && is_known_russian_word_or_form(&left_candidate_lower)
        && ngram_allows_ru_candidate(&candidate.to_lowercase(), text, NGRAM_MOVED_PREFIX_MARGIN)
    {
        return Some(format!("{candidate}{right_trailing}"));
    }

    if let Some(left_last) = left.chars().last() {
        if short_right_is_safe
            && same_letter_ignore_case(left_last, moved)
            && is_known_russian_word_or_form(&left.to_lowercase())
        {
            let candidate = format!("{left} {right_rest}");
            if ngram_allows_ru_candidate(&candidate.to_lowercase(), text, NGRAM_MOVED_PREFIX_MARGIN)
            {
                return Some(format!("{candidate}{right_trailing}"));
            }
        }
    }

    if left_candidate.chars().count() <= 3
        && !is_known_russian_word_or_form(&left.to_lowercase())
        && (russian_tiny_dictionary().contains(&left_candidate_lower)
            || russian_short_dictionary().contains(&left_candidate_lower))
        && right_rest.chars().count() >= 5
        && is_known_russian_phrase_part(&right_rest_lower)
    {
        return Some(format!("{candidate}{right_trailing}"));
    }

    if left_candidate.chars().count() < 5 || right_rest.chars().count() < 5 {
        return None;
    }
    if !is_known_russian_word_or_form(&left_candidate_lower)
        || !is_known_russian_word_or_form(&right_rest_lower)
    {
        return None;
    }
    if crate::ngram::ru_candidate_margin(&right_rest_lower, &right_lower)
        < NGRAM_MOVED_PREFIX_RIGHT_MARGIN
    {
        return None;
    }
    if !ngram_allows_ru_candidate(&candidate.to_lowercase(), text, NGRAM_MOVED_PREFIX_MARGIN) {
        return None;
    }

    Some(format!("{candidate}{right_trailing}"))
}

fn is_safe_short_moved_prefix_right(word: &str) -> bool {
    (3..=4).contains(&word.chars().count()) && russian_short_dictionary().contains(word)
}

fn correct_cyrillic_word_case(word: &str) -> Option<String> {
    if word.chars().count() < 2 || !is_cyrillic_word(word) {
        return None;
    }
    if word
        .chars()
        .all(|ch| !ch.is_alphabetic() || !ch.is_uppercase())
        || word
            .chars()
            .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
    {
        return None;
    }

    let lower = word.to_lowercase();
    if !is_known_russian_word_or_form(&lower) {
        return None;
    }

    let normalized = if word.chars().next().is_some_and(|ch| ch.is_uppercase()) {
        capitalize_first(&lower)
    } else {
        lower
    };
    (normalized != word).then_some(normalized)
}

fn correct_hard_sign_typo(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    best_unique_ngram_candidate(
        word,
        generate_hard_sign_candidates(&lower),
        NGRAM_HARD_SIGN_MARGIN,
    )
}

fn correct_adjacent_transposition(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;
    for idx in 0..chars.len().saturating_sub(1) {
        if chars[idx] == chars[idx + 1] {
            continue;
        }

        let mut candidate = chars.clone();
        candidate.swap(idx, idx + 1);
        let candidate: String = candidate.into_iter().collect();
        if !is_known_russian_word_or_form(&candidate) {
            continue;
        }
        if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_TRANSPOSE_MARGIN) {
            continue;
        }

        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

fn correct_repeated_letter(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;
    let mut idx = 0;
    while idx < chars.len() {
        let mut end = idx + 1;
        while end < chars.len() && chars[end] == chars[idx] {
            end += 1;
        }

        if end - idx > 1 {
            for keep in 1..end - idx {
                let mut candidate = Vec::with_capacity(chars.len() - (end - idx - keep));
                candidate.extend_from_slice(&chars[..idx]);
                candidate.extend(std::iter::repeat(chars[idx]).take(keep));
                candidate.extend_from_slice(&chars[end..]);
                let candidate: String = candidate.into_iter().collect();
                if !is_known_russian_word_or_form(&candidate) {
                    continue;
                }
                if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_TYPO_REJECT_MARGIN) {
                    continue;
                }
                if found.is_some() {
                    return None;
                }
                found = Some(candidate);
            }
        }

        idx = end;
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

fn correct_single_letter_substitution(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    let chars: Vec<char> = lower.chars().collect();
    let mut found: Option<String> = None;
    for idx in 0..chars.len() {
        for replacement in RU_ALPHABET {
            if replacement == chars[idx] {
                continue;
            }
            if !are_ru_keyboard_neighbors(chars[idx], replacement) {
                continue;
            }

            let mut candidate = chars.clone();
            candidate[idx] = replacement;
            let candidate: String = candidate.into_iter().collect();
            if !is_known_russian_word_or_form(&candidate) {
                continue;
            }
            if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_TYPO_REJECT_MARGIN) {
                continue;
            }

            if found.is_some() {
                return None;
            }
            found = Some(candidate);
        }
    }

    found.map(|candidate| apply_word_case(word, &candidate))
}

pub fn correct_extra_letters(word: &str) -> Option<String> {
    if word.chars().count() < 6 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }
    if lower.ends_with("тся") {
        return None;
    }
    if missing_letter_candidate_exists(word, &lower) {
        return None;
    }

    best_unique_known_ngram_candidate(
        word,
        generate_extra_letter_candidates(&lower),
        NGRAM_EXTRA_LETTER_MARGIN,
    )
}

fn correct_vowel_confusion(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    best_unique_known_ngram_candidate(
        word,
        generate_vowel_confusion_candidates(&lower),
        NGRAM_VOWEL_CONFUSION_MARGIN,
    )
}

fn correct_verb_ending_confusion(word: &str) -> Option<String> {
    if word.chars().count() < 5 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    if let Some(stem) = lower.strip_suffix("тся") {
        if stem.chars().count() >= 3 {
            let candidate = format!("{stem}ться");
            if is_known_russian_word_or_form(&candidate)
                && ngram_allows_ru_candidate(&candidate, &lower, NGRAM_VERB_ENDING_MARGIN)
            {
                return Some(apply_word_case(word, &candidate));
            }
        }
    }

    for (from, to) in [("ешь", "ишь"), ("ет", "ит")] {
        let Some(stem) = lower.strip_suffix(from) else {
            continue;
        };
        if stem.chars().count() < 3 {
            continue;
        }
        let candidate = format!("{stem}{to}");
        if !is_known_russian_word_or_form(&candidate) {
            continue;
        }
        if !ngram_allows_ru_candidate(&candidate, &lower, NGRAM_VERB_ENDING_MARGIN) {
            continue;
        }
        return Some(apply_word_case(word, &candidate));
    }

    None
}

pub fn correct_missing_letter(word: &str) -> Option<String> {
    if word.chars().count() < 6 || !is_cyrillic_word(word) {
        return None;
    }

    let lower = word.to_lowercase();
    if is_known_russian_word_or_form(&lower) {
        return None;
    }

    if let Some(candidate) = best_unique_dictionary_candidate(
        word,
        generate_missing_letter_candidates(&lower),
        NGRAM_DICT_MISSING_LETTER_MARGIN,
    ) {
        return Some(candidate);
    }

    best_unique_ngram_candidate(
        word,
        generate_missing_letter_candidates(&lower),
        NGRAM_MISSING_LETTER_MARGIN,
    )
}

fn missing_letter_candidate_exists(word: &str, lower: &str) -> bool {
    best_unique_dictionary_candidate(
        word,
        generate_missing_letter_candidates(lower),
        NGRAM_DICT_MISSING_LETTER_MARGIN,
    )
    .is_some()
        || best_unique_ngram_candidate(
            word,
            generate_missing_letter_candidates(lower),
            NGRAM_MISSING_LETTER_MARGIN,
        )
        .is_some()
}

pub fn are_ru_keyboard_neighbors(a: char, b: char) -> bool {
    let Some((row_a, col_a)) = ru_keyboard_position(a) else {
        return false;
    };
    let Some((row_b, col_b)) = ru_keyboard_position(b) else {
        return false;
    };

    row_a == row_b && col_a.abs_diff(col_b) <= 1
}

fn ru_keyboard_position(ch: char) -> Option<(usize, usize)> {
    const ROWS: [&str; 3] = ["йцукенгшщзхъ", "фывапролджэ", "ячсмитьбю"];
    ROWS.iter()
        .enumerate()
        .find_map(|(row, keys)| keys.chars().position(|key| key == ch).map(|col| (row, col)))
}

fn ngram_allows_ru_candidate(candidate: &str, baseline: &str, min_margin: f64) -> bool {
    crate::ngram::ru_candidate_margin(candidate, baseline) >= min_margin
}

pub fn should_force_replay_for_short_fragment(text: &str) -> bool {
    let mut words = text.split_whitespace();
    let Some(word) = words.next() else {
        return false;
    };
    words.next().is_none() && (1..=2).contains(&word.chars().count())
}

pub fn effective_replace_words(
    buf: &WordBuffer,
    replace_words: usize,
    engine: CorrectionEngine,
    auto_replace: bool,
) -> usize {
    let replace_words = replace_words.clamp(1, MAX_REPLACE_WORDS);
    if engine == CorrectionEngine::Replay && auto_replace && should_expand_auto_replace_context(buf)
    {
        return replace_words.max(2);
    }
    replace_words
}

pub fn decide_correction(original: &str, converted: &str, engine: CorrectionEngine) -> Correction {
    if engine == CorrectionEngine::Replay || original == converted {
        return Correction::ReplayAll;
    }
    if original.split_whitespace().count() <= 1 {
        return Correction::ReplayAll;
    }

    match crate::llm::convert_hybrid(original, converted) {
        // Manual double-Shift is an explicit user command. If smart says
        // "original is fine", still allow the user to toggle the selected text.
        Ok(Some(text)) if text == original => Correction::ReplayAll,
        Ok(Some(text)) if text == converted => Correction::ReplayAll,
        Ok(Some(text)) if !text.trim().is_empty() => Correction::InsertText(text),
        Ok(_) => Correction::ReplayAll,
        Err(_) => Correction::ReplayAll,
    }
}

pub fn decide_scoped_tail_correction(events: &[KeyEvent]) -> Option<String> {
    decide_scoped_tail_correction_with_options(events, ScopedTailOptions::default())
}

pub fn decide_scoped_tail_correction_with_lem(
    events: &[KeyEvent],
    enabled: bool,
) -> Option<String> {
    decide_scoped_tail_correction_with_options(
        events,
        ScopedTailOptions {
            lem_enabled: enabled,
            allow_layout_auto: true,
        },
    )
}

pub fn decide_scoped_tail_correction_with_options(
    events: &[KeyEvent],
    options: ScopedTailOptions,
) -> Option<String> {
    let words = split_event_words(events)?;
    if words.len() < 2 {
        return None;
    }

    let original = map_original_events(events);
    let has_trailing_space = events
        .last()
        .is_some_and(|event| event.keycode == KeyCode::KEY_SPACE.code());
    if options.lem_enabled {
        let candidates =
            scoped_tail_lem_candidates(&words, !has_trailing_space, options.allow_layout_auto)
                .into_iter()
                .map(|candidate| {
                    if has_trailing_space {
                        format!("{candidate} ")
                    } else {
                        candidate
                    }
                });
        let ranked = crate::lem::rank_candidates(&original, candidates);
        if let Some(best) = ranked.first() {
            let margin = ranked
                .get(1)
                .map(|second| best.total - second.total)
                .unwrap_or(f64::INFINITY);
            let _ = (
                margin,
                best.language,
                best.noise,
                best.edit,
                best.intervention,
            );
            let mut best_text = best.text.clone();
            if has_trailing_space && !best_text.ends_with(' ') {
                best_text.push(' ');
            }
            if best_text != original && !best_text.trim().is_empty() {
                return Some(best_text);
            }
        }
    }

    let mut out = String::with_capacity(original.len());
    for (idx, word) in words.iter().enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        if idx + 1 == words.len() && !has_trailing_space {
            out.push_str(&flip_word_events(word));
        } else {
            out.push_str(&decide_completed_scope_word(word));
        }
    }
    if has_trailing_space {
        out.push(' ');
    }

    if out != original && !out.trim().is_empty() {
        Some(out)
    } else {
        None
    }
}

pub fn scoped_tail_lem_candidates(
    words: &[&[KeyEvent]],
    last_word_is_current: bool,
    allow_layout_auto: bool,
) -> Vec<String> {
    let mut states: Vec<Vec<String>> = Vec::with_capacity(words.len());
    for (idx, word) in words.iter().enumerate() {
        let is_current_tail = last_word_is_current && idx + 1 == words.len();
        states.push(scoped_word_lem_options(
            word,
            is_current_tail,
            allow_layout_auto,
        ));
    }

    let mut out = Vec::new();
    build_phrase_candidates(&states, 0, &mut Vec::new(), &mut out);
    out
}

fn scoped_word_lem_options(
    word: &[KeyEvent],
    is_current_tail: bool,
    allow_layout_auto: bool,
) -> Vec<String> {
    let original = map_original_events(word);
    let mut out = Vec::new();
    if is_current_tail {
        push_unique_string(&mut out, flip_word_events(word));
        return out;
    }

    if let Some(repaired) = confident_completed_scope_repair(&original) {
        push_unique_string(&mut out, repaired);
        return out;
    }

    push_unique_string(&mut out, original.clone());
    push_unique_string(&mut out, decide_completed_scope_word(word));
    if let Some(repaired) = apply_typing_assist(&format!("{original} "), allow_layout_auto) {
        push_unique_string(&mut out, repaired.trim().to_string());
    }
    let flipped = flip_word_events(word);
    if should_offer_completed_scope_flip(&original, &flipped) {
        push_unique_string(&mut out, flipped);
    }
    out
}

fn confident_completed_scope_repair(original: &str) -> Option<String> {
    crate::llm::repair_mixed_script(original)
        .or_else(|| correct_duplicate_layout_prefix_on_ascii_token(original))
        .or_else(|| correct_wrong_layout_ascii_technical_token(original))
        .or_else(|| correct_wrong_layout_ascii_word(original))
}

fn should_offer_completed_scope_flip(original: &str, flipped: &str) -> bool {
    if stable_completed_scope_original(original) {
        return false;
    }

    let (_, flipped_word, _) = split_word_punctuation(flipped);
    if flipped_word.is_empty() {
        return false;
    }

    let flipped_lower = flipped_word.to_lowercase();
    if is_cyrillic_word(flipped_word) {
        return is_known_russian_layout_autoswitch_word(&flipped_lower);
    }

    if flipped_word.is_ascii() {
        return is_known_english_layout_autoswitch_word(&flipped_word.to_ascii_lowercase())
            || is_ascii_technical_token(flipped);
    }

    false
}

fn stable_completed_scope_original(original: &str) -> bool {
    let (_, word, _) = split_word_punctuation(original);
    if word.is_empty() {
        return false;
    }

    let lower = word.to_lowercase();
    if is_cyrillic_word(word) {
        return is_known_russian_layout_autoswitch_word(&lower);
    }

    if word.is_ascii() {
        let ascii_lower = word.to_ascii_lowercase();
        return is_protected_ascii_layout_token(word)
            || is_ascii_technical_token(original)
            || is_known_english_layout_autoswitch_word(&ascii_lower);
    }

    false
}

fn build_phrase_candidates(
    states: &[Vec<String>],
    idx: usize,
    current: &mut Vec<String>,
    out: &mut Vec<String>,
) {
    if idx == states.len() {
        push_unique_string(out, current.join(" "));
        return;
    }
    for option in &states[idx] {
        current.push(option.clone());
        build_phrase_candidates(states, idx + 1, current, out);
        current.pop();
    }
}

fn push_unique_string(out: &mut Vec<String>, value: String) {
    if !value.trim().is_empty() && !out.iter().any(|item| item == &value) {
        out.push(value);
    }
}

pub fn decide_completed_scope_word(word: &[KeyEvent]) -> String {
    let original = map_original_events(word);
    if let Some(repaired) = correct_duplicate_layout_prefix_on_ascii_token(&original) {
        return repaired;
    }
    if let Some(repaired) = correct_wrong_layout_ascii_technical_token(&original) {
        return repaired;
    }
    if let Some(repaired) = correct_wrong_layout_ascii_word(&original) {
        return repaired;
    }
    let converted = flip_word_events(word);
    if should_keep_plain_cyrillic_before_ascii_technical(&original, &converted) {
        return original;
    }
    let decision = if crate::llm::model_backend_enabled() {
        crate::llm::choose_token_consensus(&original, &converted)
    } else {
        crate::llm::choose_token_hybrid(&original, &converted)
    };
    match decision {
        Ok(Some(text)) if !text.trim().is_empty() => text,
        Ok(_) | Err(_) => original,
    }
}

fn flip_word_events(word: &[KeyEvent]) -> String {
    if let Some(repaired) = repair_cyrillic_prefix_before_ascii_tail(word) {
        return repaired;
    }
    let original = map_original_events(word);
    if let Some(repaired) = correct_duplicate_layout_prefix_on_ascii_token(&original) {
        return repaired;
    }
    if let Some(target_is_ru) = mixed_visual_latin_word_target_layout(word) {
        return map_events_to_layout(word, target_is_ru);
    }
    if let Some(normalized) = normalize_mixed_word_to_last_layout(word) {
        return normalized;
    }
    let decision = replay_layout_decision(word);
    map_events_to_layout(word, decision.target_is_ru)
}

pub fn repair_cyrillic_prefix_before_ascii_tail(word: &[KeyEvent]) -> Option<String> {
    let first_event = word.first()?;
    let first = original_event_char(first_event)?;
    if !is_cyrillic_letter(first) || word.len() < 3 {
        return None;
    }

    let rest = &word[1..];
    let rest_original: String = rest.iter().filter_map(original_event_char).collect();
    if rest_original.chars().count() != rest.len()
        || !rest_original.is_ascii()
        || !rest_original.chars().any(|ch| ch.is_ascii_alphabetic())
        || !rest_original
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        return None;
    }

    let all_ru = map_events_to_layout(word, true);
    if all_ru != map_original_events(word) && is_cyrillic_hyphenated_word_for_layout(&all_ru) {
        return Some(all_ru);
    }

    let mut chars = all_ru.chars();
    let first_ru = chars.next()?;
    let second_ru = chars.next()?;
    if !same_letter_ignore_case(first_ru, second_ru) {
        return None;
    }

    let mut candidate = String::new();
    candidate.push(first_ru);
    candidate.extend(chars);
    if candidate == all_ru || candidate == map_original_events(word) {
        return None;
    }
    is_cyrillic_hyphenated_word_for_layout(&candidate).then_some(candidate)
}

fn same_letter_ignore_case(left: char, right: char) -> bool {
    left.to_lowercase().to_string() == right.to_lowercase().to_string()
}

fn is_known_cyrillic_hyphenated_word(word: &str) -> bool {
    if !is_cyrillic_word(word) {
        return false;
    }
    let dict = russian_short_dictionary();
    word.split('-')
        .all(|part| part.chars().count() >= 3 && is_known_cyrillic_hyphen_part(part, dict))
}

fn is_cyrillic_hyphenated_word_for_layout(word: &str) -> bool {
    is_known_cyrillic_hyphenated_word(word) || is_plausible_cyrillic_hyphenated_word(word)
}

fn is_plausible_cyrillic_hyphenated_word(word: &str) -> bool {
    if !word.contains('-') || !is_cyrillic_word(word) {
        return false;
    }
    let parts: Vec<&str> = word.split('-').collect();
    if parts.len() < 2 || parts.iter().any(|part| part.is_empty()) {
        return false;
    }

    let mut strong_parts = 0usize;
    for part in parts {
        let lower = part.to_lowercase();
        let len = lower.chars().count();
        if len < 2 || !lower.chars().any(is_russian_vowel) {
            return false;
        }
        if len >= 3 || is_known_cyrillic_hyphen_part(&lower, russian_short_dictionary()) {
            strong_parts += 1;
        }
    }
    strong_parts >= 2
}

fn is_russian_vowel(ch: char) -> bool {
    matches!(
        ch,
        'а' | 'е'
            | 'ё'
            | 'и'
            | 'о'
            | 'у'
            | 'ы'
            | 'э'
            | 'ю'
            | 'я'
            | 'А'
            | 'Е'
            | 'Ё'
            | 'И'
            | 'О'
            | 'У'
            | 'Ы'
            | 'Э'
            | 'Ю'
            | 'Я'
    )
}

fn is_known_cyrillic_hyphen_part(part: &str, dict: &HashSet<String>) -> bool {
    let lower = part.to_lowercase();
    dict.contains(&lower)
        || russian_generated_form_dictionary().contains(&lower)
        || is_known_short_accusative_a_form(&lower, dict)
}

fn is_known_short_accusative_a_form(word: &str, dict: &HashSet<String>) -> bool {
    let Some(stem) = word.strip_suffix('у') else {
        return false;
    };
    if stem.chars().count() < 2 {
        return false;
    }
    let lemma = format!("{stem}а");
    dict.contains(&lemma)
}

pub fn is_known_russian_word_or_form(word: &str) -> bool {
    russian_dictionary().contains(word)
        || russian_generated_form_dictionary().contains(word)
        || is_known_russian_suffix_form(word)
        || is_known_russian_ka_declension_form(word)
        || is_known_russian_verb_form(word)
}

fn is_known_russian_suffix_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }

    const SUFFIXES: &[&str] = &[
        "ыми", "ими", "ами", "ями", "ого", "его", "ому", "ему", "ов", "ев", "ей", "ах", "ях", "ам",
        "ям", "ом", "ем", "ой", "ый", "ий", "ая", "яя", "ое", "ее", "ые", "ие", "а", "я", "у", "ю",
        "е", "ы", "и",
    ];

    SUFFIXES.iter().any(|suffix| {
        let Some(stem) = word.strip_suffix(suffix) else {
            return false;
        };
        if stem.chars().count() < 3 {
            return false;
        }
        if russian_dictionary().contains(stem) {
            return true;
        }
        matches!(*suffix, "ами" | "ями")
            && (russian_short_dictionary().contains(stem)
                || russian_dictionary().contains(&format!("{stem}о")))
    })
}

fn is_known_russian_adverb_o_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    let Some(stem) = word.strip_suffix('о') else {
        return false;
    };
    if stem.chars().count() < 3 {
        return false;
    }

    ["ый", "ий", "ой"]
        .iter()
        .any(|suffix| russian_dictionary().contains(&format!("{stem}{suffix}")))
}

fn is_known_russian_ka_declension_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    let Some(stem) = word.strip_suffix("ок") else {
        return false;
    };
    stem.chars().count() >= 3 && russian_dictionary().contains(&format!("{stem}ка"))
}

fn is_known_russian_ka_oblique_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    for suffix in ["ками", "ках", "кой", "ки", "ке", "ку"] {
        if let Some(stem) = word.strip_suffix(suffix) {
            return stem.chars().count() >= 3
                && russian_dictionary().contains(&format!("{stem}ка"));
        }
    }
    false
}

fn is_known_russian_verb_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }

    const ENDINGS: &[(&str, &[&str])] = &[
        ("айте", &["ать"]),
        ("ишь", &["ить", "еть"]),
        ("ай", &["ать"]),
        ("ит", &["ить", "еть"]),
        ("ает", &["ать"]),
        ("ают", &["ать"]),
        ("аешь", &["ать"]),
        ("аете", &["ать"]),
        ("ется", &["ться"]),
        ("ются", &["ться"]),
        ("ился", &["иться"]),
        ("илась", &["иться"]),
        ("ились", &["иться"]),
        ("илось", &["иться"]),
        ("ался", &["аться"]),
        ("алась", &["аться"]),
        ("ались", &["аться"]),
        ("алось", &["аться"]),
        ("ил", &["ить"]),
        ("ила", &["ить"]),
        ("или", &["ить"]),
        ("ило", &["ить"]),
        ("ал", &["ать"]),
        ("ала", &["ать"]),
        ("али", &["ать"]),
        ("ало", &["ать"]),
    ];

    ENDINGS.iter().any(|(ending, lemmas)| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        stem.chars().count() >= 3
            && lemmas
                .iter()
                .any(|lemma_suffix| russian_dictionary().contains(&format!("{stem}{lemma_suffix}")))
    })
}

fn normalize_mixed_word_to_last_layout(word: &[KeyEvent]) -> Option<String> {
    let target_is_ru = word.last()?.layout_is_ru;
    if word.iter().all(|event| event.layout_is_ru == target_is_ru) {
        return None;
    }

    let mut out = String::new();
    let mut run_start = 0;
    let mut current_layout = word.first()?.layout_is_ru;
    for (idx, event) in word.iter().enumerate() {
        if event.layout_is_ru != current_layout {
            let run = map_events_to_layout(&word[run_start..idx], target_is_ru);
            push_with_overlap(&mut out, &run);
            run_start = idx;
            current_layout = event.layout_is_ru;
        }
    }
    let run = map_events_to_layout(&word[run_start..], target_is_ru);
    push_with_overlap(&mut out, &run);

    (!out.is_empty()).then_some(out)
}

fn push_with_overlap(out: &mut String, next: &str) {
    if out.is_empty() || next.is_empty() {
        out.push_str(next);
        return;
    }

    let out_chars: Vec<char> = out.chars().collect();
    let next_chars: Vec<char> = next.chars().collect();
    let max_overlap = out_chars.len().min(next_chars.len());
    let overlap = (1..=max_overlap)
        .rev()
        .find(|len| {
            out_chars[out_chars.len() - len..]
                .iter()
                .zip(&next_chars[..*len])
                .all(|(left, right)| left == right)
        })
        .unwrap_or(0);
    out.push_str(&next_chars[overlap..].iter().collect::<String>());
}

fn best_unique_ngram_candidate<I>(original: &str, candidates: I, min_margin: f64) -> Option<String>
where
    I: IntoIterator<Item = String>,
{
    let lower = original.to_lowercase();
    let mut best: Option<(String, f64)> = None;
    let mut second_best = f64::NEG_INFINITY;

    for candidate in candidates {
        if candidate == lower || !is_cyrillic_word(&candidate) {
            continue;
        }
        let margin = crate::ngram::ru_candidate_margin(&candidate, &lower);
        if margin < min_margin {
            continue;
        }

        match &best {
            Some((_, best_margin)) if margin <= *best_margin => {
                second_best = second_best.max(margin);
            }
            Some((_, best_margin)) => {
                second_best = second_best.max(*best_margin);
                best = Some((candidate, margin));
            }
            None => best = Some((candidate, margin)),
        }
    }

    let (candidate, best_margin) = best?;
    if best_margin - second_best < 0.40 {
        return None;
    }
    Some(apply_word_case(original, &candidate))
}

fn best_unique_dictionary_candidate<I>(
    original: &str,
    candidates: I,
    min_margin: f64,
) -> Option<String>
where
    I: IntoIterator<Item = String>,
{
    let lower = original.to_lowercase();
    let mut found: Option<String> = None;

    for candidate in candidates {
        if candidate == lower || !is_known_russian_word_or_form(&candidate) {
            continue;
        }
        if crate::ngram::ru_candidate_margin(&candidate, &lower) < min_margin {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(original, &candidate))
}

fn best_unique_known_ngram_candidate<I>(
    original: &str,
    candidates: I,
    min_margin: f64,
) -> Option<String>
where
    I: IntoIterator<Item = String>,
{
    let lower = original.to_lowercase();
    let mut seen = HashSet::new();
    let mut found: Option<String> = None;

    for candidate in candidates {
        if candidate == lower || !seen.insert(candidate.clone()) {
            continue;
        }
        if !is_cyrillic_word(&candidate) || !is_known_russian_word_or_form(&candidate) {
            continue;
        }

        let margin = crate::ngram::ru_candidate_margin(&candidate, &lower);
        if margin < min_margin {
            continue;
        }

        if found.is_some() {
            return None;
        }
        found = Some(candidate);
    }

    found.map(|candidate| apply_word_case(original, &candidate))
}

fn generate_missing_letter_candidates(lower: &str) -> impl Iterator<Item = String> + '_ {
    let chars: Vec<char> = lower.chars().collect();
    (0..=chars.len()).flat_map(move |idx| {
        RU_ALPHABET.into_iter().map({
            let chars = chars.clone();
            move |inserted| {
                let mut candidate = String::with_capacity(lower.len() + inserted.len_utf8());
                candidate.extend(chars[..idx].iter());
                candidate.push(inserted);
                candidate.extend(chars[idx..].iter());
                candidate
            }
        })
    })
}

fn generate_extra_letter_candidates(lower: &str) -> Vec<String> {
    let chars: Vec<char> = lower.chars().collect();
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    for idx in 0..chars.len() {
        if is_russian_vowel(chars[idx]) {
            continue;
        }
        let mut candidate = String::with_capacity(lower.len());
        candidate.extend(chars[..idx].iter());
        candidate.extend(chars[idx + 1..].iter());
        if seen.insert(candidate.clone()) {
            candidates.push(candidate);
        }
    }

    if chars.len() >= 7 {
        for idx in 0..=chars.len() - 2 {
            if idx + 2 == chars.len() {
                continue;
            }
            if chars[idx..idx + 2].iter().all(|ch| is_russian_vowel(*ch)) {
                continue;
            }
            let mut candidate = String::with_capacity(lower.len());
            candidate.extend(chars[..idx].iter());
            candidate.extend(chars[idx + 2..].iter());
            if seen.insert(candidate.clone()) {
                candidates.push(candidate);
            }
        }
    }

    let mut idx = 0usize;
    while idx < chars.len() {
        let mut end = idx + 1;
        while end < chars.len() && chars[end] == chars[idx] {
            end += 1;
        }

        let run_len = end - idx;
        if run_len > 1 {
            for keep in 1..run_len {
                let mut candidate = String::with_capacity(lower.len());
                candidate.extend(chars[..idx].iter());
                candidate.extend(std::iter::repeat(chars[idx]).take(keep));
                candidate.extend(chars[end..].iter());
                if seen.insert(candidate.clone()) {
                    candidates.push(candidate);
                }
            }
        }

        idx = end;
    }

    candidates
}

fn generate_vowel_confusion_candidates(lower: &str) -> Vec<String> {
    let chars: Vec<char> = lower.chars().collect();
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    for idx in 0..chars.len() {
        for replacement in ru_vowel_confusion_replacements(chars[idx]).iter().copied() {
            let mut candidate = chars.clone();
            candidate[idx] = replacement;
            let candidate: String = candidate.into_iter().collect();
            if seen.insert(candidate.clone()) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

fn ru_vowel_confusion_replacements(ch: char) -> &'static [char] {
    match ch {
        'а' => &['о'],
        'о' => &['а'],
        'е' => &['и', 'ё'],
        'и' => &['е'],
        'ё' => &['е'],
        _ => &[],
    }
}

fn generate_hard_sign_candidates(lower: &str) -> impl Iterator<Item = String> + '_ {
    let chars: Vec<char> = lower.chars().collect();
    (0..chars.len().saturating_sub(1)).filter_map(move |idx| {
        if chars[idx] != 'ь' || !matches!(chars[idx + 1], 'е' | 'ё' | 'ю' | 'я') {
            return None;
        }
        let mut candidate = chars.clone();
        candidate[idx] = 'ъ';
        Some(candidate.into_iter().collect())
    })
}

fn split_word_punctuation(token: &str) -> (&str, &str, &str) {
    let start = token
        .char_indices()
        .find(|(_, ch)| ch.is_alphanumeric())
        .map(|(idx, _)| idx)
        .unwrap_or(token.len());
    let end = token
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_alphanumeric())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(start);

    (&token[..start], &token[start..end], &token[end..])
}

pub fn is_cyrillic_word(word: &str) -> bool {
    word.chars()
        .all(|ch| matches!(ch, 'А'..='я' | 'ё' | 'Ё' | '-'))
}

fn apply_phrase_case(original: &str, replacement_lower: &str) -> String {
    if original.chars().next().is_some_and(|ch| ch.is_uppercase()) {
        capitalize_first(replacement_lower)
    } else {
        replacement_lower.to_string()
    }
}

fn apply_word_case(original: &str, replacement_lower: &str) -> String {
    if original
        .chars()
        .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
    {
        replacement_lower.to_uppercase()
    } else if original.chars().next().is_some_and(|ch| ch.is_uppercase()) {
        capitalize_first(replacement_lower)
    } else {
        replacement_lower.to_string()
    }
}

fn capitalize_first(text: &str) -> String {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::new();
    out.extend(first.to_uppercase());
    out.push_str(chars.as_str());
    out
}

fn english_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = load_ascii_hunspell_words_min_len("/usr/share/hunspell/en_US.dic", 4)
            .unwrap_or_default();
        if let Ok(extra) = load_ascii_word_list_min_len("/usr/share/dict/words", 4) {
            words.extend(extra);
        }
        words
    })
}

fn russian_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words =
            load_hunspell_words_min_len("/usr/share/hunspell/ru_RU.dic", 5).unwrap_or_default();
        if let Some(home) = std::env::var_os("HOME") {
            let path = std::path::PathBuf::from(home).join(PROTECTED_WORDS_PATH);
            if let Ok(custom) = load_word_list(&path) {
                words.extend(custom);
            }
        }
        #[cfg(test)]
        words.extend(test_russian_forms().into_iter().map(str::to_string));
        words.extend(COMMON_RUSSIAN_WORDS.iter().copied().map(str::to_string));
        words
    })
}

fn russian_short_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let words =
            load_hunspell_words_min_len("/usr/share/hunspell/ru_RU.dic", 3).unwrap_or_default();
        #[cfg(test)]
        {
            let mut words = words;
            words.extend(test_russian_forms().into_iter().map(str::to_string));
            words.insert("пара".to_string());
            words.extend(COMMON_RUSSIAN_WORDS.iter().copied().map(str::to_string));
            words
        }
        #[cfg(not(test))]
        {
            let mut words = words;
            words.extend(COMMON_RUSSIAN_WORDS.iter().copied().map(str::to_string));
            words
        }
    })
}

fn russian_tiny_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let words =
            load_hunspell_words_min_len("/usr/share/hunspell/ru_RU.dic", 2).unwrap_or_default();
        #[cfg(test)]
        {
            let mut words = words;
            words.extend(test_russian_forms().into_iter().map(str::to_string));
            words.insert("не".to_string());
            words.extend(COMMON_RUSSIAN_WORDS.iter().copied().map(str::to_string));
            words
        }
        #[cfg(not(test))]
        {
            let mut words = words;
            words.extend(COMMON_RUSSIAN_WORDS.iter().copied().map(str::to_string));
            words
        }
    })
}

pub fn russian_generated_form_dictionary() -> &'static HashSet<String> {
    static WORDS: OnceLock<HashSet<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        load_hunspell_generated_forms_min_len(
            "/usr/share/hunspell/ru_RU.dic",
            "/usr/share/hunspell/ru_RU.aff",
            4,
        )
        .unwrap_or_default()
    })
}

#[cfg(test)]
fn test_russian_forms() -> [&'static str; 16] {
    [
        "библиотеку",
        "приблизительные",
        "привет",
        "проверка",
        "работает",
        "расчеты",
        "нормально",
        "ошибка",
        "ошибся",
        "явно",
        "исправлено",
        "исправляет",
        "ладно",
        "можно",
        "дальше",
        "правильно",
    ]
}

fn load_ascii_hunspell_words_min_len(
    path: &str,
    min_chars: usize,
) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut words = HashSet::new();
    for line in text.lines().skip(1) {
        let word = line.split('/').next().unwrap_or("").trim().to_lowercase();
        if word.chars().count() >= min_chars && is_plain_ascii_word_candidate(&word) {
            words.insert(word);
        }
    }
    Ok(words)
}

fn load_ascii_word_list_min_len(path: &str, min_chars: usize) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut words = HashSet::new();
    for line in text.lines() {
        let word = line.trim().to_lowercase();
        if word.chars().count() >= min_chars && is_plain_ascii_word_candidate(&word) {
            words.insert(word);
        }
    }
    Ok(words)
}

fn load_hunspell_words_min_len(path: &str, min_chars: usize) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut words = HashSet::new();
    for line in text.lines().skip(1) {
        let word = line.split('/').next().unwrap_or("").trim();
        if word.chars().count() >= min_chars && is_cyrillic_word(word) {
            words.insert(word.to_lowercase());
        }
    }
    Ok(words)
}

struct HunspellSuffixRule {
    strip: String,
    add: String,
    condition: Vec<HunspellConditionToken>,
}

#[derive(Clone)]
enum HunspellConditionToken {
    Literal(char),
    Class { negated: bool, chars: Vec<char> },
}

fn load_hunspell_generated_forms_min_len(
    dic_path: &str,
    aff_path: &str,
    min_chars: usize,
) -> std::io::Result<HashSet<String>> {
    let rules = load_simple_hunspell_suffix_rules(aff_path)?;
    let text = std::fs::read_to_string(dic_path)?;
    let mut forms = HashSet::new();

    for line in text.lines().skip(1) {
        let line = line.trim();
        let Some((word, flags)) = line.split_once('/') else {
            continue;
        };
        let word = word.trim().to_lowercase();
        if word.is_empty() {
            continue;
        }
        let flags = flags.split_whitespace().next().unwrap_or("");
        for flag in flags.chars() {
            let Some(flag_rules) = rules.get(&flag) else {
                continue;
            };
            for rule in flag_rules {
                if !hunspell_condition_matches(&word, &rule.condition) {
                    continue;
                }
                let stem = if rule.strip == "0" {
                    word.as_str()
                } else if let Some(stem) = word.strip_suffix(&rule.strip) {
                    stem
                } else {
                    continue;
                };
                let candidate = if rule.add == "0" {
                    stem.to_string()
                } else {
                    format!("{stem}{}", rule.add)
                };
                if candidate.chars().count() >= min_chars && is_cyrillic_word(&candidate) {
                    forms.insert(candidate);
                }
            }
        }
    }

    Ok(forms)
}

fn load_simple_hunspell_suffix_rules(
    path: &str,
) -> std::io::Result<HashMap<char, Vec<HunspellSuffixRule>>> {
    let text = std::fs::read_to_string(path)?;
    let mut rules: HashMap<char, Vec<HunspellSuffixRule>> = HashMap::new();

    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 || parts[0] != "SFX" || parts[3].parse::<usize>().is_ok() {
            continue;
        }
        let Some(flag) = parts[1].chars().next() else {
            continue;
        };
        let Some(condition) = parse_hunspell_suffix_condition(parts[4]) else {
            continue;
        };
        rules.entry(flag).or_default().push(HunspellSuffixRule {
            strip: parts[2].to_string(),
            add: parts[3].split('/').next().unwrap_or(parts[3]).to_string(),
            condition,
        });
    }

    Ok(rules)
}

fn parse_hunspell_suffix_condition(condition: &str) -> Option<Vec<HunspellConditionToken>> {
    if condition == "." {
        return Some(Vec::new());
    }

    let mut tokens = Vec::new();
    let mut chars = condition.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            let negated = if chars.peek() == Some(&'^') {
                chars.next();
                true
            } else {
                false
            };
            let mut class_chars = Vec::new();
            let mut closed = false;
            for class_ch in chars.by_ref() {
                if class_ch == ']' {
                    closed = true;
                    break;
                }
                if !is_cyrillic_letter(class_ch) {
                    return None;
                }
                class_chars.push(class_ch);
            }
            if !closed || class_chars.is_empty() {
                return None;
            }
            tokens.push(HunspellConditionToken::Class {
                negated,
                chars: class_chars,
            });
        } else if is_cyrillic_letter(ch) {
            tokens.push(HunspellConditionToken::Literal(ch));
        } else {
            return None;
        }
    }

    (!tokens.is_empty()).then_some(tokens)
}

fn hunspell_condition_matches(word: &str, condition: &[HunspellConditionToken]) -> bool {
    if condition.is_empty() {
        return true;
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.len() < condition.len() {
        return false;
    }
    let start = chars.len() - condition.len();
    condition
        .iter()
        .zip(chars[start..].iter().copied())
        .all(|(token, ch)| match token {
            HunspellConditionToken::Literal(expected) => *expected == ch,
            HunspellConditionToken::Class { negated, chars } => {
                let contains = chars.contains(&ch);
                if *negated {
                    !contains
                } else {
                    contains
                }
            }
        })
}

fn load_word_list(path: &std::path::Path) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_lowercase)
        .collect())
}

fn replace_visual_b_in_context(original: &str, target: &str) -> Option<String> {
    if !contains_visual_b_word(original) {
        return None;
    }

    let base = if has_cyrillic_text(original) {
        original
    } else {
        target
    };
    replace_visual_b_words(original, base)
}

fn replace_visual_b_words(original: &str, base: &str) -> Option<String> {
    let original_segments = split_ws_segments(original);
    let base_segments = split_ws_segments(base);
    if original_segments.len() != base_segments.len() {
        return None;
    }

    let mut changed = false;
    let mut out = String::with_capacity(base.len());
    for (idx, ((orig, orig_ws), (base_part, base_ws))) in original_segments
        .iter()
        .zip(base_segments.iter())
        .enumerate()
    {
        if orig_ws != base_ws {
            return None;
        }
        if *orig_ws {
            out.push_str(base_part);
            continue;
        }

        let replacement = match *orig {
            "b" => Some(visual_b_replacement(
                &original_segments,
                &base_segments,
                idx,
                false,
            )),
            "B" => Some(visual_b_replacement(
                &original_segments,
                &base_segments,
                idx,
                true,
            )),
            _ => None,
        };
        if let Some(replacement) = replacement {
            changed = true;
            out.push_str(&replacement);
        } else {
            out.push_str(base_part);
        }
    }

    if changed {
        Some(out)
    } else {
        None
    }
}

fn visual_b_replacement(
    original_segments: &[(&str, bool)],
    base_segments: &[(&str, bool)],
    idx: usize,
    uppercase: bool,
) -> String {
    let prev = previous_word_segment(original_segments, idx);
    let base = base_segments.get(idx).map(|(text, _)| *text).unwrap_or("");

    let wants_layout_i = prev.is_some_and(is_ascii_word_token)
        || (base.eq_ignore_ascii_case("и") && prev.is_some_and(is_ascii_word_token));
    let replacement = if wants_layout_i { "и" } else { "в" };
    if uppercase {
        replacement.to_uppercase()
    } else {
        replacement.to_string()
    }
}

fn previous_word_segment<'a>(segments: &'a [(&str, bool)], idx: usize) -> Option<&'a str> {
    segments[..idx]
        .iter()
        .rev()
        .find_map(|(text, is_ws)| (!*is_ws).then_some(*text))
}

fn is_ascii_word_token(token: &str) -> bool {
    let (_, core, _) = split_word_punctuation(token);
    !core.is_empty()
        && core.chars().any(|ch| ch.is_ascii_alphabetic())
        && core
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || matches!(ch, '-' | '_' | '.'))
}

pub fn split_ws_segments(text: &str) -> Vec<(&str, bool)> {
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

pub fn contains_visual_b_word(text: &str) -> bool {
    text.split_whitespace()
        .any(|word| word == "b" || word == "B")
}

fn has_cyrillic_text(text: &str) -> bool {
    text.chars().any(|ch| matches!(ch, 'А'..='я' | 'ё' | 'Ё'))
}

fn replacement_rules() -> &'static HashMap<String, String> {
    static RULES: OnceLock<HashMap<String, String>> = OnceLock::new();
    RULES.get_or_init(|| {
        let mut rules = HashMap::new();
        #[cfg(test)]
        rules.extend(test_replacement_rules());

        if let Some(home) = std::env::var_os("HOME") {
            let path = std::path::PathBuf::from(home).join(REPLACEMENTS_PATH);
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(custom) = serde_json::from_str::<HashMap<String, String>>(&text) {
                    rules.extend(custom);
                }
            }
        }

        rules
    })
}

#[cfg(test)]
fn test_replacement_rules() -> HashMap<String, String> {
    HashMap::from([
        ("подлючись".to_string(), "подключись".to_string()),
        ("надйи".to_string(), "найди".to_string()),
        ("нуда".to_string(), "ну да".to_string()),
        ("вчем".to_string(), "в чем".to_string()),
        ("можн".to_string(), "можно".to_string()),
        ("дльше".to_string(), "дальше".to_string()),
        ("дальг".to_string(), "дальше".to_string()),
        ("првильно".to_string(), "правильно".to_string()),
    ])
}

pub fn split_edge_whitespace(text: &str) -> (&str, &str, &str) {
    let start = text
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(idx, _)| idx)
        .unwrap_or(text.len());
    let end = text
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(start);

    (&text[..start], &text[start..end], &text[end..])
}
