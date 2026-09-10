use super::super::{
    russian_adjective_has_comparative_ee, russian_adjective_iy_class, RussianAdjectiveIyClass,
};
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
    if has_forbidden_y_spelling(word) {
        return false;
    }
    is_backed_short_noun_form(word, contains)
        || is_backed_regular_a_ya_noun_form(word, contains)
        || is_backed_regular_o_e_noun_form(word, contains)
        || is_backed_possessive_iy_adjective_form(word, contains)
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
    if has_forbidden_y_spelling(word) {
        return false;
    }
    is_backed_possessive_iy_adjective_form(word, contains)
        || is_backed_russian_suffix_form(word, contains)
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

fn is_backed_regular_a_ya_noun_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    const INFLECTIONS: &[(&str, &[&str])] = &[
        ("ами", &["а"]),
        ("ями", &["я"]),
        ("ой", &["а"]),
        ("ою", &["а"]),
        ("ей", &["я"]),
        ("ам", &["а"]),
        ("ям", &["я"]),
        ("ах", &["а"]),
        ("ях", &["я"]),
        ("е", &["а", "я"]),
        ("ы", &["а"]),
        ("и", &["я"]),
        ("у", &["а"]),
        ("ю", &["я"]),
    ];

    INFLECTIONS.iter().any(|(ending, lemma_endings)| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        stem.chars().count() >= 3
            && lemma_endings.iter().any(|lemma_ending| {
                !(*ending == "е" && *lemma_ending == "я" && stem.ends_with('и'))
                    && contains(&format!("{stem}{lemma_ending}"))
            })
    })
}

fn has_forbidden_y_spelling(word: &str) -> bool {
    let mut chars = word.chars().rev();
    matches!(chars.next(), Some('ы'))
        && matches!(chars.next(), Some('г' | 'к' | 'х' | 'ж' | 'ч' | 'ш' | 'щ'))
}

fn is_backed_regular_o_e_noun_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    const INFLECTIONS: &[(&str, &[&str])] = &[
        ("ами", &["о"]),
        ("ями", &["е"]),
        ("ом", &["о"]),
        ("ем", &["е"]),
        ("ам", &["о"]),
        ("ям", &["е"]),
        ("ах", &["о"]),
        ("ях", &["е"]),
        ("е", &["о"]),
        ("а", &["о"]),
        ("я", &["е"]),
        ("у", &["о"]),
        ("ю", &["е"]),
    ];

    INFLECTIONS.iter().any(|(ending, lemma_endings)| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        stem.chars().count() >= 3
            && lemma_endings
                .iter()
                .any(|lemma_ending| contains(&format!("{stem}{lemma_ending}")))
    })
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

fn is_backed_possessive_iy_adjective_form(
    word: &str,
    contains: impl Fn(&str) -> bool + Copy,
) -> bool {
    const ENDINGS: &[&str] = &[
        "ьими", "ьего", "ьему", "ьим", "ьих", "ьем", "ьей", "ья", "ье", "ьи", "ью",
    ];

    ENDINGS.iter().any(|ending| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        if stem.chars().count() < 3 {
            return false;
        }
        let lemma = format!("{stem}ий");
        contains(&lemma)
            && russian_adjective_iy_class(&lemma) == RussianAdjectiveIyClass::Possessive
    })
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
            || (adjective_suffix && is_backed_adjective_form(stem, suffix, contains))
    })
}

fn is_backed_adjective_form(
    stem: &str,
    suffix: &str,
    contains: impl Fn(&str) -> bool + Copy,
) -> bool {
    // Russian spelling makes the same adjective paradigm look partly hard and
    // partly soft after velars and sibilants. Bind every surface-suffix family
    // to both the attested lemma ending and the stem class; a single soft/hard
    // boolean admits forms from the wrong paradigm (for example, ы after к).
    let velar_stem = stem.ends_with(['г', 'к', 'х']);
    let restricted_stem = stem.ends_with(['г', 'к', 'х', 'ж', 'ч', 'ш', 'щ']);
    let backed_by = |lemma_ending| contains(&format!("{stem}{lemma_ending}"));
    let backed_by_regular_iy = || {
        let lemma = format!("{stem}ий");
        contains(&lemma) && russian_adjective_iy_class(&lemma) == RussianAdjectiveIyClass::Regular
    };
    let backed_by_comparative_ee = |lemma_ending| {
        let lemma = format!("{stem}{lemma_ending}");
        contains(&lemma) && russian_adjective_has_comparative_ee(&lemma)
    };

    match suffix {
        // -ее is an unambiguous neuter ending only for a regular non-velar
        // -ий lemma. Synthetic comparatives from hard lemmas require the
        // independent Hunspell E flag rather than adjective-class membership.
        "ее" => backed_by_comparative_ee("ый") || (!velar_stem && backed_by_regular_iy()),
        // Hard singular/neuter endings. Velar -ий lemmas use these endings too.
        "ого" | "ому" | "ом" | "ой" | "ое" => {
            backed_by("ый") || backed_by("ой") || (velar_stem && backed_by_regular_iy())
        }
        // Sibilant -ий lemmas also spell the feminine nominative with -ая.
        "ая" => backed_by("ый") || backed_by("ой") || (restricted_stem && backed_by_regular_iy()),
        // Ы-spelled plural/instrumental endings are invalid after restricted
        // stems of -ой lemmas; their surface uses the corresponding и form.
        "ыми" | "ым" | "ые" | "ых" => {
            backed_by("ый") || (!restricted_stem && backed_by("ой"))
        }
        // These и-spelled forms belong to -ий, plus restricted-stem -ой lemmas.
        "ими" | "им" | "ие" | "их" => {
            backed_by_regular_iy() || (restricted_stem && backed_by("ой"))
        }
        // Soft singular endings belong to -ий, but velar stems use the hard
        // counterparts above. Sibilants legitimately keep these soft endings.
        "его" | "ему" | "ем" | "ей" => !velar_stem && backed_by_regular_iy(),
        "яя" => !restricted_stem && backed_by_regular_iy(),
        _ => false,
    }
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
