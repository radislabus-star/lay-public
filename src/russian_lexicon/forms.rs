use crate::data_lines::data_lines;
use crate::keyboard::is_cyrillic_letter;
use crate::russian_chars::is_russian_vowel;
use crate::russian_prefixes::derivational_prefixes;

use super::{
    russian_dictionary, russian_generated_form_dictionary, russian_short_dictionary,
    russian_tiny_dictionary, WordSet,
};

include!("forms/data.rs");

#[path = "forms/backed.rs"]
mod backed;
pub(crate) use backed::{is_center_backed_russian_form, is_reference_backed_russian_form};

pub(crate) fn is_known_russian_form(word: &str) -> bool {
    is_known_russian_suffix_form(word)
        || is_known_russian_short_accusative_a_form(word)
        || is_known_russian_zero_ending_noun_form(word)
        || is_known_russian_ka_declension_form(word)
        || is_known_russian_ka_oblique_form(word)
        || is_known_russian_prefixed_form(word)
        || is_known_russian_verb_form(word)
        || is_known_russian_ch_verb_present_form(word)
        || is_known_russian_imperative_i_form(word)
}

pub(crate) fn is_known_russian_adverb_o_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    let Some(stem) = word.strip_suffix('о') else {
        return false;
    };
    if stem.chars().count() < 3 {
        return false;
    }

    adjective_lemma_endings()
        .any(|suffix| russian_dictionary().contains(&format!("{stem}{suffix}")))
}

pub(crate) fn is_known_russian_ka_oblique_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    for suffix in ka_oblique_suffixes() {
        if let Some(stem) = word.strip_suffix(suffix) {
            return stem.chars().count() >= 3 && known_runtime_lemma(&format!("{stem}ка"));
        }
    }
    false
}

pub(crate) fn is_known_cyrillic_hyphen_part(part: &str, dict: &WordSet) -> bool {
    let lower = part.to_lowercase();
    dict.contains(&lower)
        || russian_generated_form_dictionary().contains(&lower)
        || is_known_short_accusative_a_form(&lower, dict)
}

pub(crate) fn looks_like_russian_adjective_lemma(word: &str) -> bool {
    adjective_lemma_endings().any(|ending| word.ends_with(ending))
}

fn is_known_russian_suffix_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }

    suffix_forms().any(|suffix| {
        let Some(stem) = word.strip_suffix(suffix) else {
            return false;
        };
        if stem.chars().count() < 3 {
            return false;
        }
        if matches!(suffix, "ы" | "и") && looks_like_russian_adjective_lemma(stem) {
            return false;
        }
        if is_known_russian_adjective_form(stem, suffix) {
            return true;
        }
        if known_runtime_lemma(stem) {
            return true;
        }
        if matches!(suffix, "я" | "ю" | "ем" | "ями" | "ях")
            && stem.ends_with('и')
            && known_runtime_lemma(&format!("{stem}е"))
        {
            return true;
        }
        matches!(suffix, "ами" | "ями")
            && (russian_short_dictionary().contains(stem)
                || known_runtime_lemma(&format!("{stem}о")))
    })
}

fn is_known_russian_adjective_form(stem: &str, suffix: &str) -> bool {
    if stem.chars().count() < 3 {
        return false;
    }
    if !adjective_form_suffixes().any(|candidate| candidate == suffix) {
        return false;
    }
    adjective_lemma_endings().any(|ending| known_runtime_lemma(&format!("{stem}{ending}")))
        || possessive_suffixes().any(|suffix| {
            let Some(noun_stem) = stem.strip_suffix(suffix) else {
                return false;
            };
            noun_stem.chars().count() >= 3
                && (russian_tiny_dictionary().contains(noun_stem)
                    || russian_dictionary().contains(noun_stem))
        })
}

fn is_known_russian_zero_ending_noun_form(word: &str) -> bool {
    let word_len = word.chars().count();
    if word_len < 4 || !word.chars().last().is_some_and(is_russian_consonant) {
        return false;
    }

    zero_noun_suffixes().any(|suffix| {
        if word_len < 5 && suffix != "о" {
            return false;
        }
        let lemma = format!("{word}{suffix}");
        known_runtime_lemma(&lemma)
    })
}

fn is_known_russian_short_accusative_a_form(word: &str) -> bool {
    let Some(stem) = word.strip_suffix('у') else {
        return false;
    };
    if stem.chars().count() < 4 {
        return false;
    }
    let lemma = format!("{stem}а");
    known_runtime_lemma(&lemma)
}

fn is_known_russian_ka_declension_form(word: &str) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    let Some(stem) = word.strip_suffix("ок") else {
        return false;
    };
    stem.chars().count() >= 3 && known_runtime_lemma(&format!("{stem}ка"))
}

fn is_known_russian_prefixed_form(word: &str) -> bool {
    derivational_prefixes().any(|prefix| {
        let Some(rest) = word.strip_prefix(prefix) else {
            return false;
        };
        rest.chars().count() >= 5
            && (russian_dictionary().contains(rest) || is_known_russian_verb_form(rest))
    })
}

fn is_known_russian_verb_form(word: &str) -> bool {
    backed::is_backed_russian_verb_form(word, known_runtime_lemma)
}

fn is_known_russian_ch_verb_present_form(word: &str) -> bool {
    backed::is_backed_russian_ch_verb_present_form(word, known_runtime_lemma)
}

fn known_runtime_lemma(lemma: &str) -> bool {
    center_contains(lemma)
        || russian_dictionary().contains(lemma)
        || russian_short_dictionary().contains(lemma)
}

fn center_contains(surface: &str) -> bool {
    crate::nanda_wave::l2::l2_surface_foundation_has_authority(surface)
}

fn is_known_russian_imperative_i_form(word: &str) -> bool {
    backed::is_backed_russian_imperative_i_form(word, known_runtime_lemma)
}

fn is_known_short_accusative_a_form(word: &str, dict: &WordSet) -> bool {
    let Some(stem) = word.strip_suffix('у') else {
        return false;
    };
    if stem.chars().count() < 2 {
        return false;
    }
    let lemma = format!("{stem}а");
    dict.contains(&lemma)
}

fn is_russian_consonant(ch: char) -> bool {
    is_cyrillic_letter(ch) && !is_russian_vowel(ch) && !matches!(ch, 'ь' | 'Ь' | 'ъ' | 'Ъ')
}

#[cfg(test)]
mod tests {
    use super::is_known_russian_form;

    #[test]
    fn center_lemmas_produce_regular_noun_and_adjective_forms() {
        for word in ["действия", "доставкой", "лучшее", "слов"] {
            assert!(
                is_known_russian_form(word),
                "missing center-backed form: {word}"
            );
        }
    }
}
