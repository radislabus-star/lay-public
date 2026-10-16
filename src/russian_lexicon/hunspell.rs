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

pub(super) type AffixWitnesses = HashMap<String, HashSet<String>>;

pub(super) struct HunspellInflectionMetadata {
    pub(super) regular_iy: HashSet<String>,
    pub(super) possessive_iy: HashSet<String>,
    pub(super) comparative_ee: HashSet<String>,
    pub(super) attested_forms: AffixWitnesses,
}

pub(super) fn load_hunspell_inflection_metadata(
    path: &str,
    aff_path: &str,
    relations: &[(&str, &str)],
    max_pairs: usize,
) -> std::io::Result<HunspellInflectionMetadata> {
    let text = std::fs::read_to_string(path)?;
    let mut metadata = hunspell_adjective_classes(&text);
    if let Ok(aff) = std::fs::read_to_string(aff_path) {
        if affix_supports_attestation(&aff) {
            metadata.attested_forms = attested_suffix_witnesses(
                &text,
                &parse_simple_hunspell_suffix_rules(&aff),
                relations,
                max_pairs,
            )
            .unwrap_or_default();
        }
    }
    Ok(metadata)
}

// Validate only the new authority. The legacy reference generator deliberately
// retains its existing permissive parser and behavior.
fn affix_supports_attestation(text: &str) -> bool {
    let mut groups = HashSet::new();
    let mut group_flag = "";
    let mut remaining = 0usize;
    for line in text.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let Some(&directive) = fields.first() else {
            continue;
        };
        if directive.starts_with('#') {
            continue;
        }
        match directive {
            "AF" | "FULLSTRIP" | "NEEDAFFIX" | "PSEUDOROOT" | "CIRCUMFIX" | "FORBIDDENWORD"
            | "ONLYINCOMPOUND" => return false,
            "FLAG" | "SET" if fields.as_slice() != [directive, "UTF-8"] => return false,
            "SFX" if remaining > 0 => {
                if fields.len() < 5 || fields[1] != group_flag {
                    return false;
                }
                remaining -= 1;
            }
            "SFX" => {
                if fields.len() != 4
                    || fields[1].chars().count() != 1
                    || !matches!(fields[2], "Y" | "N")
                    || !groups.insert(fields[1])
                {
                    return false;
                }
                let Ok(count) = fields[3].parse() else {
                    return false;
                };
                group_flag = fields[1];
                remaining = count;
            }
            _ if remaining > 0 => return false,
            _ => {}
        }
    }
    remaining == 0
}

fn hunspell_adjective_classes(text: &str) -> HunspellInflectionMetadata {
    let mut regular_iy = HashSet::new();
    let mut possessive_iy = HashSet::new();
    let mut comparative_ee = HashSet::new();

    for (word, flags) in hunspell_dic_entries(text) {
        let lower = word.to_lowercase();
        let flags = flags.unwrap_or_default();
        if flags.contains('E') {
            comparative_ee.insert(lower.clone());
        }
        if !lower.ends_with("ий") {
            continue;
        }
        if flags.contains('A') {
            regular_iy.insert(lower.clone());
        }
        if flags.contains('O') {
            possessive_iy.insert(lower);
        }
    }

    HunspellInflectionMetadata {
        regular_iy,
        possessive_iy,
        comparative_ee,
        attested_forms: HashMap::new(),
    }
}

struct HunspellSuffixRule {
    strip: String,
    add: String,
    condition: Vec<HunspellConditionToken>,
    can_attest: bool,
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
                let Some(candidate) = apply_hunspell_suffix_rule(&word, rule) else {
                    continue;
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
    Ok(parse_simple_hunspell_suffix_rules(&text))
}

fn parse_simple_hunspell_suffix_rules(text: &str) -> HashMap<char, Vec<HunspellSuffixRule>> {
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
            can_attest: parts[1].chars().count() == 1 && !parts[3].contains('/'),
        });
    }

    rules
}

fn apply_hunspell_suffix_rule(word: &str, rule: &HunspellSuffixRule) -> Option<String> {
    if !hunspell_condition_matches(word, &rule.condition) {
        return None;
    }
    let stem = if rule.strip == "0" {
        word
    } else {
        word.strip_suffix(&rule.strip)?
    };
    Some(if rule.add == "0" {
        stem.to_string()
    } else {
        format!("{stem}{}", rule.add)
    })
}

fn attested_suffix_witnesses(
    dictionary: &str,
    rules: &HashMap<char, Vec<HunspellSuffixRule>>,
    relations: &[(&str, &str)],
    max_pairs: usize,
) -> Option<AffixWitnesses> {
    let mut witnesses = AffixWitnesses::new();
    let mut pairs = 0usize;
    for (lemma, flags) in hunspell_dic_entries(dictionary) {
        let Some(flags) = flags else {
            continue;
        };
        let lemma = lemma.to_lowercase();
        for &(ending, lemma_ending) in relations {
            let Some(stem) = lemma.strip_suffix(lemma_ending) else {
                continue;
            };
            let surface = format!("{stem}{ending}");
            if !is_cyrillic_word(&surface) {
                continue;
            }
            let licensed = flags
                .chars()
                .filter_map(|flag| rules.get(&flag))
                .flatten()
                .any(|rule| {
                    rule.can_attest
                        && (rule.strip == "0"
                            || lemma
                                .strip_suffix(&rule.strip)
                                .is_some_and(|stem| !stem.is_empty()))
                        && apply_hunspell_suffix_rule(&lemma, rule).as_deref()
                            == Some(surface.as_str())
                });
            if licensed && witnesses.entry(lemma.clone()).or_default().insert(surface) {
                pairs += 1;
                if pairs > max_pairs {
                    return None;
                }
            }
        }
    }
    Some(witnesses)
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
    use super::{
        attested_suffix_witnesses, hunspell_adjective_classes, hunspell_dic_entries,
        load_hunspell_inflection_metadata, parse_simple_hunspell_suffix_rules,
    };
    use std::collections::HashSet;

    #[test]
    fn hunspell_entries_skip_count_header_and_extract_optional_flags() {
        let entries: Vec<_> =
            hunspell_dic_entries("3\n слово/AB extra\nтест\n\n дом/CD\n").collect();

        assert_eq!(
            entries,
            vec![("слово", Some("AB")), ("тест", None), ("дом", Some("CD"))]
        );
    }

    #[test]
    fn adjective_iy_classes_follow_hunspell_inflection_flags() {
        let classes = hunspell_adjective_classes(
            "7\n синий/A\n русский/AZ\n лисий/O\n точный/E\n почтовый/A\n слово/O\n неизвестный\n",
        );

        assert_eq!(
            classes.regular_iy,
            HashSet::from(["синий".to_string(), "русский".to_string()])
        );
        assert_eq!(classes.possessive_iy, HashSet::from(["лисий".to_string()]));
        assert_eq!(
            classes.comparative_ee,
            HashSet::from(["точный".to_string()])
        );
        assert!(load_hunspell_inflection_metadata(
            "/definitely/missing/lay-hunspell-adjective-classes.dic",
            "/missing.aff",
            &[],
            4096
        )
        .is_err());
    }

    #[test]
    fn affix_witness_requires_exact_lemma_flag_condition_strip_and_surface() {
        let dictionary =
            "8\nнести/M\nнайти/M\nгрести/Y\nпочти\nвезти/M\nползти/L\nслово/M\nнести/M\n";
        let rules = parse_simple_hunspell_suffix_rules(
            "SFX M Y 3\nSFX M ти у [сз]ти\nSFX M ти ду йти\nSFX M x у .\nSFX Y сти бу сти\nSFX L ти ут [сз]ти\n",
        );
        let witnesses =
            attested_suffix_witnesses(dictionary, &rules, &[("у", "ти")], 4096).unwrap();
        assert_eq!(witnesses.len(), 2);
        assert_eq!(witnesses["нести"], HashSet::from(["несу".to_string()]));
        assert_eq!(witnesses["везти"], HashSet::from(["везу".to_string()]));
        for rejected in ["найти", "грести", "почти", "ползти", "слово"] {
            assert!(
                !witnesses.contains_key(rejected),
                "unexpected witness for {rejected}"
            );
        }
        for unsupported in ["SFX MM ти у [сз]ти\n", "SFX M ти у/X [сз]ти\n"] {
            let rules = parse_simple_hunspell_suffix_rules(unsupported);
            assert!(
                attested_suffix_witnesses(dictionary, &rules, &[("у", "ти")], 4096)
                    .unwrap()
                    .is_empty()
            );
        }
    }

    #[test]
    fn affix_witness_budget_is_unique_streamed_and_order_independent() {
        let rules = parse_simple_hunspell_suffix_rules("SFX M ти у .\n");
        let entries: Vec<_> = (0..4097)
            .map(|mut number| {
                let mut stem = "слов".to_string();
                for _ in 0..3 {
                    stem.push(char::from_u32('а' as u32 + number % 32).unwrap());
                    number /= 32;
                }
                format!("{stem}ти/M")
            })
            .collect();
        let dictionary = |rows: Vec<&str>| format!("{}\n{}\n", rows.len(), rows.join("\n"));
        let mut at_limit: Vec<_> = entries[..4096].iter().map(String::as_str).collect();
        at_limit.extend(entries[..20].iter().map(String::as_str));
        let witnesses =
            attested_suffix_witnesses(&dictionary(at_limit), &rules, &[("у", "ти")], 4096).unwrap();
        assert_eq!(witnesses.values().map(HashSet::len).sum::<usize>(), 4096);
        for reversed in [false, true] {
            let mut rows: Vec<_> = entries.iter().map(String::as_str).collect();
            if reversed {
                rows.reverse();
            }
            assert!(
                attested_suffix_witnesses(&dictionary(rows), &rules, &[("у", "ти")], 4096)
                    .is_none()
            );
        }
    }

    #[test]
    fn malformed_suffix_groups_and_aliases_cannot_attest_grammar() {
        let root = std::env::temp_dir().join(format!("lay-affix-structure-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let dictionary = root.join("ru.dic");
        let affix = root.join("ru.aff");
        std::fs::write(&dictionary, "3\nсиний/A\nнести/M\nнести/1\n").unwrap();
        for text in [
            "SET UTF-8\nAF 1\nAF X\nSFX 1 Y 1\nSFX 1 ти у [сз]ти\n",
            "SET UTF-8\nSFX M ти у [сз]ти\n",
            "SET UTF-8\nSFX M Y 0\nSFX M ти у [сз]ти\n",
            "SET UTF-8\nSFX M Y 2\nSFX M ти у [сз]ти\n",
            "SET UTF-8\nSFX M X 1\nSFX M ти у [сз]ти\n",
            "SET UTF-8\nSFX M Y 1\nSFX X ти у [сз]ти\n",
            "SET UTF-8\nSFX M Y 1\nSFX M ти у [сз]ти\nSFX M Y 1\nSFX M ти у [сз]ти\n",
        ] {
            std::fs::write(&affix, text).unwrap();
            let metadata = load_hunspell_inflection_metadata(
                dictionary.to_str().unwrap(),
                affix.to_str().unwrap(),
                &[("у", "ти")],
                4096,
            )
            .unwrap();
            assert!(
                metadata.attested_forms.is_empty(),
                "invalid affix authority: {text}"
            );
            assert!(metadata.regular_iy.contains("синий"));
        }
        std::fs::write(&affix, "SET UTF-8\nSFX M Y 1\nSFX M ти у [сз]ти\n").unwrap();
        let metadata = load_hunspell_inflection_metadata(
            dictionary.to_str().unwrap(),
            affix.to_str().unwrap(),
            &[("у", "ти")],
            4096,
        )
        .unwrap();
        assert!(metadata.attested_forms["нести"].contains("несу"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn whole_lemma_strip_is_not_an_attestation_witness() {
        let root = std::env::temp_dir().join(format!("lay-affix-fullstrip-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let dictionary = root.join("ru.dic");
        let affix = root.join("ru.aff");
        std::fs::write(&dictionary, "2\nсиний/A\nнести/M\n").unwrap();
        for prefix in ["", "FULLSTRIP\n"] {
            std::fs::write(
                &affix,
                format!("SET UTF-8\n{prefix}SFX M Y 1\nSFX M нести несу .\n"),
            )
            .unwrap();
            let metadata = load_hunspell_inflection_metadata(
                dictionary.to_str().unwrap(),
                affix.to_str().unwrap(),
                &[("у", "ти")],
                4096,
            )
            .unwrap();
            assert!(
                metadata.attested_forms.is_empty(),
                "whole-lemma strip: {prefix}"
            );
            assert!(metadata.regular_iy.contains("синий"));
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_or_unsupported_affix_evidence_preserves_adjective_classes() {
        let root = std::env::temp_dir().join(format!("lay-inflection-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let dictionary = root.join("ru.dic");
        let affix = root.join("ru.aff");
        std::fs::write(&dictionary, "2\nсиний/A\nнести/M\n").unwrap();
        for contents in [
            None,
            Some("FLAG long\nSFX M ти у .\n"),
            Some("SFX M ти у [invalid\n"),
            Some("NEEDAFFIX X\nSFX M ти у .\n"),
        ] {
            if let Some(text) = contents {
                std::fs::write(&affix, text).unwrap();
            }
            let metadata = load_hunspell_inflection_metadata(
                dictionary.to_str().unwrap(),
                affix.to_str().unwrap(),
                &[("у", "ти")],
                4096,
            )
            .unwrap();
            assert_eq!(metadata.regular_iy, HashSet::from(["синий".to_string()]));
            assert!(metadata.attested_forms.is_empty());
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
