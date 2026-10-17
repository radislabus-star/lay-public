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

fn parse_verb_form_ending(line: &str) -> Option<(&str, Vec<&str>, bool)> {
    let mut fields = line.split('\t');
    let ending = fields.next()?.trim();
    let lemmas = fields.next()?.trim();
    let require_affix_witness = match fields.next() {
        None => false,
        Some("affix") => true,
        Some(_) => return None,
    };
    if fields.next().is_some() || ending.is_empty() || lemmas.is_empty() {
        return None;
    }
    let lemmas: Vec<_> = lemmas.split(',').map(str::trim).collect();
    if lemmas.iter().any(|lemma| lemma.is_empty()) {
        return None;
    }
    Some((ending, lemmas, require_affix_witness))
}

fn verb_form_endings() -> impl Iterator<Item = (&'static str, Vec<&'static str>, bool)> {
    // Preserve tabs: trimming the full line would turn an empty third column
    // into an absent column and accidentally grant unmarked legacy authority.
    VERB_FORM_ENDINGS_DATA.lines().filter_map(parse_verb_form_ending)
}

pub(super) fn attested_verb_relations() -> Vec<(&'static str, &'static str)> {
    verb_form_endings()
        .filter(|(_, _, attested)| *attested)
        .flat_map(|(ending, lemmas, _)| lemmas.into_iter().map(move |lemma| (ending, lemma)))
        .collect()
}

#[cfg(test)]
mod verb_relation_parser_tests {
    use super::parse_verb_form_ending;

    #[test]
    fn evidence_markers_fail_closed_including_empty_columns() {
        assert_eq!(parse_verb_form_ending("у\tить"), Some(("у", vec!["ить"], false)));
        assert_eq!(parse_verb_form_ending("у\tти\taffix"), Some(("у", vec!["ти"], true)));
        for invalid in ["у\tти\t", "у\tти\tunknown", "у\tти\taffix\t", "у\tти,\taffix", "\tти"] {
            assert!(parse_verb_form_ending(invalid).is_none(), "{invalid:?}");
        }
    }
}
