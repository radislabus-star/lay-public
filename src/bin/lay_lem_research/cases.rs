use lay::dict::{self, Direction};
use lay::lem;

#[derive(Clone, Debug)]
pub(crate) struct Case {
    pub(crate) kind: &'static str,
    pub(crate) typed: String,
    pub(crate) expected: String,
}

const RU_WORDS: &[&str] = &[
    "привет",
    "слово",
    "проверка",
    "сегодня",
    "можно",
    "сделать",
    "текст",
    "ошибка",
    "быстро",
    "точно",
    "пишу",
    "дальше",
    "работает",
    "проект",
    "клавиатура",
    "раскладка",
    "помощник",
    "магазин",
    "документ",
    "таблица",
    "данные",
    "система",
    "окно",
    "фраза",
    "режим",
    "демон",
    "кнопка",
    "меню",
    "правильно",
    "хорошо",
    "плохо",
    "новый",
    "старый",
    "важно",
    "сейчас",
    "потом",
    "завтра",
    "вопрос",
    "ответ",
    "пример",
    "тест",
];

const EN_WORDS: &[&str] = &[
    "hello", "world", "test", "good", "double", "shift", "linux", "gnome", "wayland", "rust",
    "cargo", "github", "browser", "window", "file", "system", "model", "score", "layout",
    "keyboard", "input", "text", "word", "quick", "smart", "safe", "fast", "slow", "open", "close",
];

const TECH_TOKENS: &[&str] = &[
    "wi-fi", "API", "AmoCRM", "GitHub", "NTFS", "Linux", "JSON", "USB-C",
];

const BRAND_TOKENS: &[&str] = &["AmoCRM", "GitHub", "NTFS", "Linux", "JSON", "USB-C"];

const NATURAL_HYPHEN_WORDS: &[&str] = &[
    "код-дэ-вуар",
    "пара-пара",
    "рок-н-ролл",
    "чек-лист",
    "тест-кейс",
    "интернет-магазин",
];

pub(crate) fn build_cases(target: usize) -> Vec<Case> {
    let mut cases = Vec::with_capacity(target);
    for i in 0..target {
        let mut case = build_case(i);
        mark_ambiguous_valid_typo_as_keep(&mut case);
        cases.push(case);
    }
    cases
}

fn build_case(i: usize) -> Case {
    let ru_a = RU_WORDS[i % RU_WORDS.len()];
    let ru_b = RU_WORDS[(i * 7 + 3) % RU_WORDS.len()];
    let en_a = EN_WORDS[(i * 5 + 1) % EN_WORDS.len()];
    let en_b = EN_WORDS[(i * 11 + 2) % EN_WORDS.len()];
    let tech = TECH_TOKENS[i % TECH_TOKENS.len()];
    let brand = BRAND_TOKENS[i % BRAND_TOKENS.len()];
    let natural_hyphen = NATURAL_HYPHEN_WORDS[i % NATURAL_HYPHEN_WORDS.len()];

    match i % 16 {
        0 => wrong_layout_ru_pair(ru_a, ru_b),
        1 => wrong_layout_en_pair(en_a, en_b),
        2 => mixed_en_ru(en_a, ru_a),
        3 => mixed_ru_en(ru_a, en_a),
        4 => typo_case("transpose", transpose_middle(ru_a), ru_a),
        5 => typo_case("missing_letter", drop_middle(ru_a), ru_a),
        6 => typo_case("extra_letter", duplicate_middle(ru_a), ru_a),
        7 => split_word_case(ru_a, ru_b),
        8 => glued_words_case(i, ru_a, ru_b),
        9 => keep_valid_case(i, ru_a, ru_b, en_a),
        10 => unchanged_pair("technical_keep", tech, ru_a),
        11 => technical_mixed_ru(tech, ru_a),
        12 => Case {
            kind: "brand_letter",
            typed: format!("{brand} Z"),
            expected: format!("{brand} Я"),
        },
        13 => Case {
            kind: "technical_layout_token",
            typed: dict::convert(tech, Direction::Us2Ru),
            expected: tech.to_string(),
        },
        14 => Case {
            kind: "hyphen_keep",
            typed: natural_hyphen.to_string(),
            expected: natural_hyphen.to_string(),
        },
        _ => punctuation_mixed(ru_a, en_a),
    }
}

fn wrong_layout_ru_pair(left: &str, right: &str) -> Case {
    let expected = format!("{left} {right}");
    Case {
        kind: "ru_wrong_layout",
        typed: dict::convert(&expected, Direction::Ru2Us),
        expected,
    }
}

fn wrong_layout_en_pair(left: &str, right: &str) -> Case {
    let expected = format!("{left} {right}");
    Case {
        kind: "en_wrong_layout",
        typed: dict::convert(&expected, Direction::Us2Ru),
        expected,
    }
}

fn mixed_en_ru(en: &str, ru: &str) -> Case {
    let expected = format!("{en} {ru}");
    Case {
        kind: "mixed_en_ru",
        typed: format!("{en} {}", dict::convert(ru, Direction::Ru2Us)),
        expected,
    }
}

fn mixed_ru_en(ru: &str, en: &str) -> Case {
    let expected = format!("{ru} {en}");
    Case {
        kind: "mixed_ru_en",
        typed: format!("{ru} {}", dict::convert(en, Direction::Us2Ru)),
        expected,
    }
}

fn typo_case(kind: &'static str, typed: String, expected: &str) -> Case {
    Case {
        kind,
        typed,
        expected: expected.to_string(),
    }
}

fn split_word_case(left: &str, right: &str) -> Case {
    let expected = format!("{left} {right}");
    Case {
        kind: "split_word",
        typed: split_last_char_to_next(left, right),
        expected,
    }
}

fn glued_words_case(i: usize, left: &str, right: &str) -> Case {
    let expected = if i % 20 == 8 {
        "я тут".to_string()
    } else {
        format!("{left} {right}")
    };
    Case {
        kind: "glued_words",
        typed: expected.replace(' ', ""),
        expected,
    }
}

fn keep_valid_case(i: usize, ru_a: &str, ru_b: &str, en_a: &str) -> Case {
    let expected = if i % 2 == 0 {
        format!("{ru_a} {en_a}")
    } else {
        format!("{ru_a} {ru_b}")
    };
    Case {
        kind: "keep_valid",
        typed: expected.clone(),
        expected,
    }
}

fn unchanged_pair(kind: &'static str, left: &str, right: &str) -> Case {
    let expected = format!("{left} {right}");
    Case {
        kind,
        typed: expected.clone(),
        expected,
    }
}

fn technical_mixed_ru(tech: &str, ru: &str) -> Case {
    let expected = format!("{tech} {ru}");
    Case {
        kind: "technical_mixed_ru",
        typed: format!("{tech} {}", dict::convert(ru, Direction::Ru2Us)),
        expected,
    }
}

fn punctuation_mixed(ru: &str, en: &str) -> Case {
    let expected = format!("{ru}, {en}");
    Case {
        kind: "punctuation_mixed",
        typed: format!("{ru}, {}", dict::convert(en, Direction::Us2Ru)),
        expected,
    }
}

fn mark_ambiguous_valid_typo_as_keep(case: &mut Case) {
    if !matches!(case.kind, "transpose" | "missing_letter" | "extra_letter") {
        return;
    }
    if case.typed == case.expected || !lem::is_known_text(&case.typed) {
        return;
    }
    case.kind = "ambiguous_typo_keep";
    case.expected = case.typed.clone();
}

fn transpose_middle(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() > 4 {
        let idx = chars.len() / 2 - 1;
        chars.swap(idx, idx + 1);
    }
    chars.into_iter().collect()
}

fn drop_middle(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() > 4 {
        let idx = chars.len() / 2;
        chars.remove(idx);
    }
    chars.into_iter().collect()
}

fn duplicate_middle(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() > 3 {
        let idx = chars.len() / 2;
        chars.insert(idx, chars[idx]);
    }
    chars.into_iter().collect()
}

fn split_last_char_to_next(left: &str, right: &str) -> String {
    let mut left_chars: Vec<char> = left.chars().collect();
    let Some(moved) = left_chars.pop() else {
        return format!("{left} {right}");
    };
    format!(
        "{} {}{}",
        left_chars.into_iter().collect::<String>(),
        moved,
        right
    )
}
