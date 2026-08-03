use super::{
    adjective_form_suffixes, adjective_lemma_endings, center_contains, is_russian_consonant,
    suffix_forms, verb_form_endings, zero_noun_suffixes,
};

pub(crate) fn is_center_backed_russian_form(word: &str) -> bool {
    is_backed_russian_form(word, center_contains)
}

pub(crate) fn is_reference_backed_russian_form(word: &str) -> bool {
    is_backed_russian_form(word, |surface| {
        crate::nanda_wave::l2::l2_surface_foundation_contains(surface)
    })
}

pub(crate) fn is_full_reference_backed_russian_form(word: &str) -> bool {
    is_backed_clean_reference_form(word, |surface| {
        super::super::full_russian_dictionary().contains(surface)
            || super::super::full_russian_short_dictionary().contains(surface)
    })
}

fn is_backed_clean_reference_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    is_backed_short_noun_form(word, contains)
        || is_backed_russian_suffix_form(word, contains)
        || is_backed_short_accusative_a_form(word, contains)
        || is_backed_ka_declension_form(word, contains)
        || is_backed_short_adjective_form(word, contains)
        || is_backed_russian_verb_form(word, contains)
        || is_backed_russian_ch_verb_present_form(word, contains)
        || is_backed_russian_consonant_alternating_form(word, contains)
        || is_backed_russian_imperative_i_form(word, contains)
        || is_backed_russian_imperative_y_form(word, contains)
        || is_backed_yts_genitive_plural_form(word, contains)
}

fn is_backed_short_noun_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    if word.chars().count() != 4 {
        return false;
    }
    ["а", "я", "у", "ю", "е", "ы", "и"]
        .into_iter()
        .any(|suffix| word.strip_suffix(suffix).is_some_and(contains))
}

pub(super) fn is_backed_russian_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    is_backed_russian_suffix_form(word, contains)
        || is_backed_zero_ending_noun_form(word, contains)
        || is_backed_short_accusative_a_form(word, contains)
        || is_backed_ka_declension_form(word, contains)
        || is_backed_short_adjective_form(word, contains)
        || is_backed_russian_verb_form(word, contains)
        || is_backed_russian_ch_verb_present_form(word, contains)
        || is_backed_russian_consonant_alternating_form(word, contains)
        || is_backed_russian_imperative_i_form(word, contains)
        || is_backed_russian_imperative_y_form(word, contains)
        || is_backed_yts_genitive_plural_form(word, contains)
}

fn is_backed_russian_consonant_alternating_form(
    word: &str,
    contains: impl Fn(&str) -> bool + Copy,
) -> bool {
    if let Some(stem) = word.strip_suffix("ли") {
        if let Some(base) = stem.strip_suffix(['г', 'к']) {
            if base.chars().count() >= 2 && contains(&format!("{base}чь")) {
                return true;
            }
        }
    }

    if let Some(stem) = word.strip_suffix('у') {
        if let Some(base) = stem.strip_suffix('ж') {
            if base.chars().count() >= 2
                && ["зать", "дить", "деть"]
                    .into_iter()
                    .any(|ending| contains(&format!("{base}{ending}")))
            {
                return true;
            }
        }
    }

    false
}

fn is_backed_yts_genitive_plural_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    let Some(stem) = word.strip_suffix("йцев") else {
        return false;
    };
    stem.chars().count() >= 3 && contains(&format!("{stem}ец"))
}

fn is_backed_short_adjective_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    ["а", "о", "ы"].into_iter().any(|ending| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        stem.chars().count() >= 3
            && adjective_lemma_endings()
                .any(|lemma_ending| contains(&format!("{stem}{lemma_ending}")))
    })
}

fn is_backed_short_accusative_a_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    let Some(stem) = word.strip_suffix('у') else {
        return false;
    };
    stem.chars().count() >= 4 && contains(&format!("{stem}а"))
}

fn is_backed_ka_declension_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    let Some(stem) = word.strip_suffix("ок") else {
        return false;
    };
    stem.chars().count() >= 3 && contains(&format!("{stem}ка"))
}

fn is_backed_zero_ending_noun_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    let word_len = word.chars().count();
    if word_len < 4 || !word.chars().last().is_some_and(is_russian_consonant) {
        return false;
    }

    zero_noun_suffixes()
        .any(|suffix| (word_len >= 5 || suffix == "о") && contains(&format!("{word}{suffix}")))
}

fn is_backed_russian_suffix_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    if word.chars().count() < 5 {
        return false;
    }
    if let Some(stem) = word.strip_suffix("кой") {
        if stem.chars().count() >= 3 && contains(&format!("{stem}ка")) {
            return true;
        }
    }
    suffix_forms().any(|suffix| {
        let Some(stem) = word.strip_suffix(suffix) else {
            return false;
        };
        if stem.chars().count() < 3 {
            return false;
        }
        if matches!(suffix, "ы" | "и")
            && adjective_lemma_endings().any(|ending| stem.ends_with(ending))
        {
            return false;
        }
        let adjective_suffix = adjective_form_suffixes().any(|candidate| candidate == suffix);
        (!adjective_suffix && contains(stem))
            || (suffix == "а" && contains(&format!("{stem}о")))
            || (suffix == "я" && contains(&format!("{stem}е")))
            || (matches!(suffix, "ы" | "и")
                && (contains(&format!("{stem}а")) || contains(&format!("{stem}я"))))
            || (matches!(suffix, "ами" | "ями") && contains(&format!("{stem}о")))
            || (matches!(suffix, "я" | "ю" | "ем" | "ями" | "ях")
                && stem.ends_with('и')
                && contains(&format!("{stem}е")))
            || (matches!(suffix, "и" | "ю" | "ей" | "ям" | "ями" | "ях")
                && stem.ends_with('и')
                && contains(&format!("{stem}я")))
            || (adjective_suffix
                && adjective_lemma_endings().any(|ending| contains(&format!("{stem}{ending}"))))
    })
}

pub(super) fn is_backed_russian_verb_form(
    word: &str,
    contains: impl Fn(&str) -> bool + Copy,
) -> bool {
    if word.chars().count() < 4 {
        return false;
    }

    verb_form_endings().any(|(ending, lemmas)| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        let min_stem_len = if ending == "шу" { 2 } else { 3 };
        stem.chars().count() >= min_stem_len
            && lemmas
                .into_iter()
                .any(|lemma_suffix| contains(&format!("{stem}{lemma_suffix}")))
    })
}

pub(super) fn is_backed_russian_ch_verb_present_form(
    word: &str,
    contains: impl Fn(&str) -> bool + Copy,
) -> bool {
    const ENDINGS: &[&str] = &["ешь", "ет", "ем", "ете", "ёшь", "ёт", "ём", "ёте"];
    if word.chars().count() < 5 {
        return false;
    }
    ENDINGS.iter().any(|ending| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        let Some(base) = stem.strip_suffix('ж') else {
            return false;
        };
        base.chars().count() >= 2 && contains(&format!("{base}чь"))
    })
}

pub(super) fn is_backed_russian_imperative_i_form(
    word: &str,
    contains: impl Fn(&str) -> bool + Copy,
) -> bool {
    let Some(stem) = word.strip_suffix('и') else {
        return false;
    };
    if stem.chars().count() < 3 {
        return false;
    }
    if ["ить", "еть", "ать"]
        .into_iter()
        .any(|lemma_suffix| contains(&format!("{stem}{lemma_suffix}")))
    {
        return true;
    }
    let Some(base) = stem.strip_suffix('ш') else {
        return false;
    };
    base.chars().count() >= 2
        && ['с', 'х']
            .into_iter()
            .any(|alternation| contains(&format!("{base}{alternation}ать")))
}

pub(super) fn is_backed_russian_imperative_y_form(
    word: &str,
    contains: impl Fn(&str) -> bool + Copy,
) -> bool {
    let Some(stem) = word.strip_suffix('й') else {
        return false;
    };
    stem.chars().count() >= 4
        && stem
            .chars()
            .last()
            .is_some_and(|ch| matches!(ch, 'а' | 'я'))
        && contains(&format!("{stem}ть"))
}
