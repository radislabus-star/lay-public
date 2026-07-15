use super::{l1_energy, TailContext, TokenKind, WavePacket, WordCandidate};
use crate::candidate_contract::CandidateOrigin;
use crate::lexicon::{is_common_ru_word, is_ru_one_letter_function_word};
use crate::russian_lexicon::is_known_russian_word_or_form;
use crate::word_reader::split_last_ws_token;

pub(super) fn grammar_agreement_candidates(
    tail: &str,
    context: &TailContext,
    l1: &[WavePacket],
) -> Vec<WordCandidate> {
    if context.has_technical_context() {
        return Vec::new();
    }
    let Some(previous) = context.previous() else {
        return Vec::new();
    };
    let Some(last) = context.last() else {
        return Vec::new();
    };
    if previous.kind != TokenKind::CyrillicWord || last.kind != TokenKind::CyrillicWord {
        return Vec::new();
    }
    let previous = clean_ru_token(&previous.text);
    let last_clean = clean_ru_token(&last.text);
    let Some(replacement) = preposition_case_completion(&previous, &last_clean)
        .or_else(|| agree_adjective_like_tail(&previous, &last_clean))
    else {
        return Vec::new();
    };
    if replacement == last_clean {
        return Vec::new();
    }
    let Some((prefix, _token)) = split_last_ws_token(tail.trim_end()) else {
        return Vec::new();
    };
    vec![WordCandidate {
        text: format!("{prefix}{replacement}"),
        origin: CandidateOrigin::L3Context,
        source: "GrammarCell32",
        energy: l1_energy(l1, "ScriptCell32")
            .max(l1_energy(l1, "BoundaryCell32"))
            .max(0.84),
        risk: 0.13,
        support: vec![
            "grammar-agreement".to_string(),
            format!("previous={previous:?} last={last_clean:?} replacement={replacement:?}"),
        ],
    }]
}

pub(super) fn clean_ru_token(token: &str) -> String {
    token
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || matches!(ch, '«' | '»' | '“' | '”'))
        .to_lowercase()
}

fn preposition_case_completion(previous: &str, word: &str) -> Option<String> {
    if !crate::lexicon::is_ru_short_preposition(previous)
        && !is_ru_one_letter_function_word(previous)
    {
        return None;
    }
    if word.chars().count() < 5 || is_common_ru_word(word) || is_known_russian_word_or_form(word) {
        return None;
    }
    let replacement = format!("{word}и");
    if is_known_russian_word_or_form(&replacement)
        || is_common_ru_word(&replacement)
        || word.ends_with("ани")
        || word.ends_with("ени")
    {
        return Some(replacement);
    }
    None
}

pub(super) fn agree_adjective_like_tail(previous: &str, word: &str) -> Option<String> {
    if previous.chars().count() < 4 || word.chars().count() < 6 {
        return None;
    }
    if is_common_ru_word(word) || has_russian_verb_tail(previous) {
        return None;
    }
    let stem = word
        .strip_suffix("ительные")
        .map(|stem| (stem, "ительный"))
        .or_else(|| word.strip_suffix("альные").map(|stem| (stem, "альный")))
        .or_else(|| word.strip_suffix("ные").map(|stem| (stem, "ный")))
        .or_else(|| word.strip_suffix("ые").map(|stem| (stem, "ый")))?;
    if !looks_like_singular_anchor(previous) || looks_like_plural_anchor(previous) {
        return None;
    }
    Some(format!("{}{}", stem.0, stem.1))
}

fn has_russian_verb_tail(word: &str) -> bool {
    const VERB_TAILS: &[&str] = &[
        "ет", "ит", "ют", "ут", "ат", "ят", "ем", "им", "ешь", "ишь", "ете", "ите", "ал", "ала",
        "ило", "или", "ил", "ено", "ена", "ены", "ает", "яет", "ует",
    ];
    VERB_TAILS.iter().any(|tail| word.ends_with(tail))
}

fn looks_like_plural_anchor(word: &str) -> bool {
    word.ends_with("ые")
        || word.ends_with("ие")
        || word.ends_with("ых")
        || word.ends_with("их")
        || word.ends_with("ыми")
        || word.ends_with("ими")
}

fn looks_like_singular_anchor(word: &str) -> bool {
    let Some(last) = word.chars().last() else {
        return false;
    };
    matches!(
        last,
        'б' | 'в'
            | 'г'
            | 'д'
            | 'ж'
            | 'з'
            | 'й'
            | 'к'
            | 'л'
            | 'м'
            | 'н'
            | 'п'
            | 'р'
            | 'с'
            | 'т'
            | 'ф'
            | 'х'
            | 'ц'
            | 'ч'
            | 'ш'
            | 'щ'
            | 'о'
            | 'е'
    )
}
