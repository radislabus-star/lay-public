use super::tokenize::normalize_word;
use super::{CharNgramModel, Lang};

const RU_HUNSPELL: &str = "/usr/share/hunspell/ru_RU.dic";
const EN_HUNSPELL: &str = "/usr/share/hunspell/en_US.dic";
const EN_WORDS: &str = "/usr/share/dict/words";
const PROTECTED_WORDS_PATH: &str = ".config/lay/protected_words.txt";

pub fn build_ru_model_from_sources() -> CharNgramModel {
    let mut words = Vec::new();
    words.extend(load_hunspell_words(RU_HUNSPELL, Lang::Ru));
    if let Some(home) = std::env::var_os("HOME") {
        let path = std::path::PathBuf::from(home).join(PROTECTED_WORDS_PATH);
        words.extend(load_plain_words(&path, Lang::Ru));
    }
    CharNgramModel::train(Lang::Ru, words)
}

pub(super) fn build_en_model_from_sources() -> CharNgramModel {
    let mut words = load_hunspell_words(EN_HUNSPELL, Lang::En);
    if words.is_empty() {
        words.extend(load_plain_words(std::path::Path::new(EN_WORDS), Lang::En));
    }
    CharNgramModel::train(Lang::En, words)
}

fn load_hunspell_words(path: &str, lang: Lang) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .skip(1)
        .filter_map(|line| normalize_word(line.split('/').next().unwrap_or(""), lang))
        .collect()
}

fn load_plain_words(path: &std::path::Path, lang: Lang) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| normalize_word(line, lang))
        .collect()
}
