const ADJECTIVE_LEMMA_ENDINGS_DATA: &str =
    include_str!("../../../data/lexicon/russian_adjective_lemma_endings.txt");
const KA_OBLIQUE_SUFFIXES_DATA: &str =
    include_str!("../../../data/lexicon/russian_ka_oblique_suffixes.txt");
const SUFFIX_FORMS_DATA: &str = include_str!("../../../data/lexicon/russian_suffix_forms.txt");
const ADJECTIVE_FORM_SUFFIXES_DATA: &str =
    include_str!("../../../data/lexicon/russian_adjective_form_suffixes.txt");
const POSSESSIVE_SUFFIXES_DATA: &str =
    include_str!("../../../data/lexicon/russian_possessive_suffixes.txt");
const ZERO_NOUN_SUFFIXES_DATA: &str =
    include_str!("../../../data/lexicon/russian_zero_noun_suffixes.txt");
const VERB_FORM_ENDINGS_DATA: &str =
    include_str!("../../../data/lexicon/russian_verb_form_endings.tsv");

fn adjective_lemma_endings() -> impl Iterator<Item = &'static str> {
    data_lines(ADJECTIVE_LEMMA_ENDINGS_DATA)
}

fn ka_oblique_suffixes() -> impl Iterator<Item = &'static str> {
    data_lines(KA_OBLIQUE_SUFFIXES_DATA)
}

fn suffix_forms() -> impl Iterator<Item = &'static str> {
    data_lines(SUFFIX_FORMS_DATA)
}

fn adjective_form_suffixes() -> impl Iterator<Item = &'static str> {
    data_lines(ADJECTIVE_FORM_SUFFIXES_DATA)
}

fn possessive_suffixes() -> impl Iterator<Item = &'static str> {
    data_lines(POSSESSIVE_SUFFIXES_DATA)
}

fn zero_noun_suffixes() -> impl Iterator<Item = &'static str> {
    data_lines(ZERO_NOUN_SUFFIXES_DATA)
}

fn verb_form_endings() -> impl Iterator<Item = (&'static str, Vec<&'static str>)> {
    data_lines(VERB_FORM_ENDINGS_DATA).filter_map(|line| {
        let (ending, lemmas) = line.split_once('\t')?;
        Some((
            ending,
            lemmas
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .collect(),
        ))
    })
}
