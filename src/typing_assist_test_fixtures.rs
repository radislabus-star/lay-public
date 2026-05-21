use std::collections::HashMap;

const TEST_RUSSIAN_FORMS_DATA: &str = include_str!("../tests/fixtures/russian_forms.txt");
const TEST_REPLACEMENT_RULES_DATA: &str = include_str!("../tests/fixtures/replacement_rules.tsv");

pub fn russian_forms() -> impl Iterator<Item = &'static str> {
    TEST_RUSSIAN_FORMS_DATA
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

pub fn replacement_rules() -> HashMap<String, String> {
    TEST_REPLACEMENT_RULES_DATA
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('\t'))
        .map(|(from, to)| (from.to_string(), to.to_string()))
        .collect()
}
