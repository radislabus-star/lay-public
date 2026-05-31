use crate::keyboard::is_cyrillic_letter;
use crate::word_reader::is_cyrillic_word;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::data_lines::data_lines;

pub(super) fn load_hunspell_words_min_len(
    path: &str,
    min_chars: usize,
) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    let mut words = HashSet::new();
    for (word, _) in hunspell_dic_entries(&text) {
        if word.chars().count() >= min_chars && is_cyrillic_word(word) {
            words.insert(word.to_lowercase());
        }
    }
    Ok(words)
}

struct HunspellSuffixRule {
    strip: String,
    add: String,
    condition: Vec<HunspellConditionToken>,
}

#[derive(Clone)]
enum HunspellConditionToken {
    Literal(char),
    Class { negated: bool, chars: Vec<char> },
}

pub(super) fn load_hunspell_generated_forms_min_len(
    dic_path: &str,
    aff_path: &str,
    min_chars: usize,
) -> std::io::Result<HashSet<String>> {
    let rules = load_simple_hunspell_suffix_rules(aff_path)?;
    let text = std::fs::read_to_string(dic_path)?;
    let mut forms = HashSet::new();

    for (word, flags) in hunspell_dic_entries(&text) {
        let Some(flags) = flags else {
            continue;
        };
        let word = word.to_lowercase();
        if word.is_empty() {
            continue;
        }
        for flag in flags.chars() {
            let Some(flag_rules) = rules.get(&flag) else {
                continue;
            };
            for rule in flag_rules {
                if !hunspell_condition_matches(&word, &rule.condition) {
                    continue;
                }
                let stem = if rule.strip == "0" {
                    word.as_str()
                } else if let Some(stem) = word.strip_suffix(&rule.strip) {
                    stem
                } else {
                    continue;
                };
                let candidate = if rule.add == "0" {
                    stem.to_string()
                } else {
                    format!("{stem}{}", rule.add)
                };
                if candidate.chars().count() >= min_chars && is_cyrillic_word(&candidate) {
                    forms.insert(candidate);
                }
            }
        }
    }

    Ok(forms)
}

fn hunspell_dic_entries(text: &str) -> impl Iterator<Item = (&str, Option<&str>)> {
    text.lines().skip(1).map(str::trim).filter_map(|line| {
        if line.is_empty() {
            return None;
        }
        let (word, flags) = match line.split_once('/') {
            Some((word, flags)) => (
                word.trim(),
                Some(flags.split_whitespace().next().unwrap_or("")),
            ),
            None => (line, None),
        };
        Some((word, flags))
    })
}

fn load_simple_hunspell_suffix_rules(
    path: &str,
) -> std::io::Result<HashMap<char, Vec<HunspellSuffixRule>>> {
    let text = std::fs::read_to_string(path)?;
    let mut rules: HashMap<char, Vec<HunspellSuffixRule>> = HashMap::new();

    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 || parts[0] != "SFX" || parts[3].parse::<usize>().is_ok() {
            continue;
        }
        let Some(flag) = parts[1].chars().next() else {
            continue;
        };
        let Some(condition) = parse_hunspell_suffix_condition(parts[4]) else {
            continue;
        };
        rules.entry(flag).or_default().push(HunspellSuffixRule {
            strip: parts[2].to_string(),
            add: parts[3].split('/').next().unwrap_or(parts[3]).to_string(),
            condition,
        });
    }

    Ok(rules)
}

fn parse_hunspell_suffix_condition(condition: &str) -> Option<Vec<HunspellConditionToken>> {
    if condition == "." {
        return Some(Vec::new());
    }

    let mut tokens = Vec::new();
    let mut chars = condition.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            let negated = if chars.peek() == Some(&'^') {
                chars.next();
                true
            } else {
                false
            };
            let mut class_chars = Vec::new();
            let mut closed = false;
            for class_ch in chars.by_ref() {
                if class_ch == ']' {
                    closed = true;
                    break;
                }
                if !is_cyrillic_letter(class_ch) {
                    return None;
                }
                class_chars.push(class_ch);
            }
            if !closed || class_chars.is_empty() {
                return None;
            }
            tokens.push(HunspellConditionToken::Class {
                negated,
                chars: class_chars,
            });
        } else if is_cyrillic_letter(ch) {
            tokens.push(HunspellConditionToken::Literal(ch));
        } else {
            return None;
        }
    }

    (!tokens.is_empty()).then_some(tokens)
}

fn hunspell_condition_matches(word: &str, condition: &[HunspellConditionToken]) -> bool {
    if condition.is_empty() {
        return true;
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.len() < condition.len() {
        return false;
    }
    let start = chars.len() - condition.len();
    condition
        .iter()
        .zip(chars[start..].iter().copied())
        .all(|(token, ch)| match token {
            HunspellConditionToken::Literal(expected) => *expected == ch,
            HunspellConditionToken::Class { negated, chars } => {
                let contains = chars.contains(&ch);
                if *negated {
                    !contains
                } else {
                    contains
                }
            }
        })
}

pub(super) fn load_word_list(path: &Path) -> std::io::Result<HashSet<String>> {
    let text = std::fs::read_to_string(path)?;
    Ok(data_lines(&text).map(str::to_lowercase).collect())
}

#[cfg(test)]
mod tests {
    use super::hunspell_dic_entries;

    #[test]
    fn hunspell_entries_skip_count_header_and_extract_optional_flags() {
        let entries: Vec<_> =
            hunspell_dic_entries("3\n слово/AB extra\nтест\n\n дом/CD\n").collect();

        assert_eq!(
            entries,
            vec![("слово", Some("AB")), ("тест", None), ("дом", Some("CD"))]
        );
    }
}
