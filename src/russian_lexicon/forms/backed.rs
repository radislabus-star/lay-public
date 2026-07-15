use super::{
    adjective_form_suffixes, adjective_lemma_endings, center_contains, suffix_forms,
    verb_form_endings,
};

pub(crate) fn is_center_backed_russian_form(word: &str) -> bool {
    is_backed_russian_form(word, center_contains)
}

pub(crate) fn is_reference_backed_russian_form(word: &str) -> bool {
    is_backed_russian_form(word, |surface| {
        crate::nanda_wave::l2::l2_surface_foundation_contains(surface)
    })
}

fn is_backed_russian_form(word: &str, contains: impl Fn(&str) -> bool + Copy) -> bool {
    is_backed_russian_suffix_form(word, contains)
        || is_backed_russian_verb_form(word, contains)
        || is_backed_russian_ch_verb_present_form(word, contains)
        || is_backed_russian_imperative_i_form(word, contains)
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
        let adjective_suffix = adjective_form_suffixes().any(|candidate| candidate == suffix);
        (!adjective_suffix && contains(stem))
            || (matches!(suffix, "я" | "ю" | "ем" | "ями" | "ях")
                && stem.ends_with('и')
                && contains(&format!("{stem}е")))
            || (adjective_suffix
                && adjective_lemma_endings().any(|ending| contains(&format!("{stem}{ending}"))))
    })
}

pub(super) fn is_backed_russian_verb_form(
    word: &str,
    contains: impl Fn(&str) -> bool + Copy,
) -> bool {
    if word.chars().count() < 5 {
        return false;
    }

    verb_form_endings().any(|(ending, lemmas)| {
        let Some(stem) = word.strip_suffix(ending) else {
            return false;
        };
        stem.chars().count() >= 3
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
    stem.chars().count() >= 4 && contains(&format!("{stem}ить"))
}
