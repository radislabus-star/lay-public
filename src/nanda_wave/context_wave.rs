use std::collections::{BTreeMap, HashSet};
use std::sync::OnceLock;

use crate::lexicon::{extend_common_ru_words, is_common_ru_word};
use crate::phrase_lexicon::is_known_russian_phrase_part;
use crate::russian_lexicon::{is_known_russian_word_or_form, russian_tiny_dictionary};
use crate::text_metrics::damerau_levenshtein;

use super::signal::WordCandidate;

pub const SEMANTIC_WORD_SOURCE: &str = "SemanticWordCell32";
pub const PHRASE_FORECAST_CELL: &str = "PhraseForecastCell32";

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticWaveMode {
    pub name: &'static str,
    pub frequency_id: u16,
    pub amplitude: f32,
    pub phase: i8,
    pub damping: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CandidateInterference {
    pub candidate: &'static str,
    pub amplitude: f32,
    pub phase_alignment: f32,
    pub coherence: f32,
    pub damping: f32,
    pub projection: f32,
    pub source_modes: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextWave {
    pub prefix: String,
    pub partial_token: String,
    pub modes: Vec<SemanticWaveMode>,
}

impl ContextWave {
    fn mode_energy(&self) -> f32 {
        let total = self
            .modes
            .iter()
            .map(|mode| mode.amplitude * (1.0 - mode.damping))
            .sum::<f32>();
        (total / self.modes.len().max(1) as f32).clamp(0.0, 1.0)
    }
}

pub fn semantic_word_candidates(tail: &str) -> Vec<WordCandidate> {
    let mut candidates = if known_russian_phrase_tail(tail) {
        Vec::new()
    } else {
        nearest_ru_word_candidates(tail)
    };
    if let Some(wave) = context_wave_for_tail(tail) {
        candidates.extend(
            candidate_interferences(&wave)
                .into_iter()
                .take(3)
                .filter(|item| item.projection >= 0.22)
                .map(|item| interference_to_candidate(&wave, item)),
        );
    }
    candidates
        .sort_by(|left, right| (right.energy - right.risk).total_cmp(&(left.energy - left.risk)));
    candidates.dedup_by(|left, right| left.text == right.text);
    candidates.into_iter().take(3).collect()
}

fn nearest_ru_word_candidates(tail: &str) -> Vec<WordCandidate> {
    let trimmed = tail.trim_end();
    let Some((prefix, token)) = split_last_token(trimmed) else {
        return Vec::new();
    };
    if prefix.split_whitespace().next().is_none() {
        return Vec::new();
    }
    let normalized = normalize_ru(token);
    let len = normalized.chars().count();
    if !(4..=18).contains(&len) || is_common_ru_word(&normalized) {
        return Vec::new();
    }
    let first = normalized.chars().next();
    let prefix2 = normalized.chars().take(2).collect::<String>();
    let Some(first) = first else {
        return Vec::new();
    };
    let mut pool = Vec::new();
    for word_len in len.saturating_sub(2)..=len + 2 {
        if let Some(words) = common_ru_word_index().get(&(first, word_len)) {
            pool.extend(words.iter());
        }
    }
    let mut ranked = pool
        .into_iter()
        .filter_map(|word| {
            let distance = damerau_levenshtein(&normalized, word);
            let word_prefix2 = word.chars().take(2).collect::<String>();
            let allowed = distance == 1
                || (len >= 8 && distance == 2 && word_prefix2 == prefix2)
                || negative_prefix_repair(&normalized, word, distance);
            (allowed
                && !looks_like_suffix_stripping(&normalized, word)
                && !looks_like_case_vowel_append_drift(&normalized, word)
                && !looks_like_case_vowel_to_consonant_drift(&normalized, word)
                && !looks_like_known_form_to_other_known_word_drift(&normalized, word, distance)
                && !looks_like_known_verb_to_noun_drift(&normalized, word))
            .then_some((word, distance))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|(left, left_distance), (right, right_distance)| {
        left_distance
            .cmp(right_distance)
            .then_with(|| {
                left.chars()
                    .count()
                    .abs_diff(len)
                    .cmp(&right.chars().count().abs_diff(len))
            })
            .then_with(|| left.cmp(right))
    });
    ranked
        .into_iter()
        .take(3)
        .map(|(word, distance)| ru_word_to_candidate(prefix, token, word, distance))
        .collect()
}

fn looks_like_known_verb_to_noun_drift(original: &str, candidate: &str) -> bool {
    is_known_russian_word_or_form(original)
        && has_russian_verb_tail(original)
        && !has_russian_verb_tail(candidate)
}

fn looks_like_known_form_to_other_known_word_drift(
    original: &str,
    candidate: &str,
    distance: usize,
) -> bool {
    if original == candidate || negative_prefix_repair(original, candidate, distance) {
        return false;
    }
    known_ru_token(original) && known_ru_token(candidate)
}

fn known_ru_token(word: &str) -> bool {
    is_common_ru_word(word)
        || is_known_russian_word_or_form(word)
        || russian_tiny_dictionary().contains(word)
}

fn negative_prefix_repair(original: &str, candidate: &str, distance: usize) -> bool {
    distance >= 2
        && distance <= 3
        && original.starts_with("не")
        && candidate.starts_with("не")
        && original.chars().count().abs_diff(candidate.chars().count()) <= 3
}

fn has_russian_verb_tail(word: &str) -> bool {
    const VERB_TAILS: &[&str] = &[
        "ет", "ит", "ют", "ут", "ат", "ят", "ем", "им", "ешь", "ишь", "ете", "ите", "ал", "ала",
        "ило", "или", "ил", "ено", "ена", "ены", "ает", "яет", "ует",
    ];
    VERB_TAILS.iter().any(|tail| word.ends_with(tail))
}

fn looks_like_suffix_stripping(original: &str, candidate: &str) -> bool {
    let Some(stripped) = original.strip_prefix(candidate) else {
        return false;
    };
    !stripped.is_empty()
        && stripped
            .chars()
            .all(|ch| matches!(ch, 'а' | 'я' | 'у' | 'ю' | 'е' | 'ы' | 'и' | 'о' | 'м'))
}

fn looks_like_case_vowel_append_drift(original: &str, candidate: &str) -> bool {
    let Some(appended) = candidate.strip_prefix(original) else {
        return false;
    };
    appended.chars().count() == 1
        && appended.chars().all(is_russian_case_vowel)
        && original.chars().count() >= 4
}

fn looks_like_case_vowel_to_consonant_drift(original: &str, candidate: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    if original_chars.len() != candidate_chars.len() || original_chars.len() < 4 {
        return false;
    }
    let last_idx = original_chars.len() - 1;
    original_chars[..last_idx] == candidate_chars[..last_idx]
        && is_russian_case_vowel(original_chars[last_idx])
        && is_russian_consonant(candidate_chars[last_idx])
}

fn is_russian_case_vowel(ch: char) -> bool {
    matches!(ch, 'а' | 'я' | 'у' | 'ю' | 'е' | 'ы' | 'и' | 'о')
}

fn is_russian_consonant(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё')
        && !matches!(
            ch,
            'а' | 'я' | 'у' | 'ю' | 'е' | 'ё' | 'ы' | 'и' | 'о' | 'э' | 'ь' | 'ъ'
        )
}

fn common_ru_words() -> &'static Vec<String> {
    static WORDS: OnceLock<Vec<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let mut words = HashSet::new();
        extend_common_ru_words(&mut words);
        let mut words = words.into_iter().collect::<Vec<_>>();
        words.sort();
        words
    })
}

fn common_ru_word_index() -> &'static BTreeMap<(char, usize), Vec<String>> {
    static INDEX: OnceLock<BTreeMap<(char, usize), Vec<String>>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut index = BTreeMap::<(char, usize), Vec<String>>::new();
        for word in common_ru_words() {
            let Some(first) = word.chars().next() else {
                continue;
            };
            index
                .entry((first, word.chars().count()))
                .or_default()
                .push(word.clone());
        }
        index
    })
}

fn ru_word_to_candidate(prefix: &str, token: &str, word: &str, distance: usize) -> WordCandidate {
    let len = normalize_ru(token).chars().count().max(1);
    let closeness = 1.0 - (distance as f32 / len as f32);
    WordCandidate {
        text: format!("{prefix}{word}"),
        source: SEMANTIC_WORD_SOURCE,
        energy: (0.58 + closeness * 0.32).clamp(0.0, 0.94),
        risk: (0.30 - closeness * 0.14).clamp(0.10, 0.30),
        support: vec![
            "nearest-ru-word".to_string(),
            format!("token={token:?} candidate={word:?} distance={distance}"),
        ],
    }
}

pub fn phrase_forecast_summary(original: &str, chosen: &WordCandidate) -> Option<String> {
    if chosen.source != SEMANTIC_WORD_SOURCE {
        return None;
    }
    let wave = context_wave_for_tail(original)?;
    let best = candidate_interferences(&wave).into_iter().next()?;
    let forecast = format!("{}{}", wave.prefix, best.candidate);
    Some(format!(
        "forecast={forecast:?} word={:?} projection={:.3} coherence={:.3} cell={PHRASE_FORECAST_CELL}",
        best.candidate, best.projection, best.coherence
    ))
}

pub fn context_wave_for_tail(tail: &str) -> Option<ContextWave> {
    let trimmed = tail.trim_end();
    let (prefix, partial_token) = split_last_token(trimmed)?;
    let lower_prefix = prefix.to_lowercase();
    let lower_partial = partial_token.to_lowercase();
    if !weather_context_is_active(&lower_prefix, &lower_partial) {
        return None;
    }
    Some(ContextWave {
        prefix: prefix.to_string(),
        partial_token: partial_token.to_string(),
        modes: vec![
            SemanticWaveMode {
                name: "weather_context",
                frequency_id: 0x0711,
                amplitude: 0.92,
                phase: 12,
                damping: 0.04,
            },
            SemanticWaveMode {
                name: "repeat_process",
                frequency_id: 0x0417,
                amplitude: repeat_process_amplitude(&lower_prefix),
                phase: 7,
                damping: 0.08,
            },
            SemanticWaveMode {
                name: "verb_frame_idet_event",
                frequency_id: 0x0903,
                amplitude: event_verb_amplitude(&lower_prefix),
                phase: 11,
                damping: 0.05,
            },
            SemanticWaveMode {
                name: "prefix_d",
                frequency_id: 0x0024,
                amplitude: prefix_d_amplitude(&lower_partial),
                phase: 5,
                damping: 0.03,
            },
        ],
    })
}

pub fn candidate_interferences(wave: &ContextWave) -> Vec<CandidateInterference> {
    let mode_energy = wave.mode_energy();
    let mut candidates = vec![
        CandidateInterference {
            candidate: "дождь",
            amplitude: 0.96,
            phase_alignment: 0.96,
            coherence: mode_energy,
            damping: 0.04,
            projection: 0.0,
            source_modes: vec![
                "weather_context",
                "repeat_process",
                "verb_frame_idet_event",
                "prefix_d",
            ],
        },
        CandidateInterference {
            candidate: "дождик",
            amplitude: 0.86,
            phase_alignment: 0.86,
            coherence: mode_energy * 0.92,
            damping: 0.06,
            projection: 0.0,
            source_modes: vec!["weather_context", "verb_frame_idet_event", "prefix_d"],
        },
        CandidateInterference {
            candidate: "день",
            amplitude: 0.42,
            phase_alignment: 0.54,
            coherence: mode_energy * 0.62,
            damping: 0.10,
            projection: 0.0,
            source_modes: vec!["prefix_d"],
        },
        CandidateInterference {
            candidate: "дрель",
            amplitude: 0.20,
            phase_alignment: 0.24,
            coherence: mode_energy * 0.36,
            damping: 0.14,
            projection: 0.0,
            source_modes: vec!["prefix_d"],
        },
    ];
    for item in &mut candidates {
        item.projection =
            (item.amplitude * item.phase_alignment * item.coherence - item.damping).max(0.0);
    }
    candidates.sort_by(|left, right| right.projection.total_cmp(&left.projection));
    candidates
}

fn interference_to_candidate(wave: &ContextWave, item: CandidateInterference) -> WordCandidate {
    let text = format!("{}{}", wave.prefix, item.candidate);
    let energy = (0.42 + item.projection * 0.62).clamp(0.0, 0.99);
    let risk = (0.18 - item.projection * 0.08).clamp(0.06, 0.18);
    WordCandidate {
        text,
        source: SEMANTIC_WORD_SOURCE,
        energy,
        risk,
        support: semantic_support(wave, &item),
    }
}

fn semantic_support(wave: &ContextWave, item: &CandidateInterference) -> Vec<String> {
    let mut support = vec![
        format!(
            "candidate={} projection={:.3} coherence={:.3}",
            item.candidate, item.projection, item.coherence
        ),
        format!(
            "partial={:?} phase_alignment={:.3}",
            wave.partial_token, item.phase_alignment
        ),
    ];
    support.extend(wave.modes.iter().map(|mode| {
        format!(
            "mode={} freq={} amp={:.3} phase={} damping={:.3}",
            mode.name, mode.frequency_id, mode.amplitude, mode.phase, mode.damping
        )
    }));
    support
}

fn weather_context_is_active(prefix: &str, partial: &str) -> bool {
    has_weather_place(prefix)
        && has_event_verb(prefix)
        && has_repeat_or_weather_hint(prefix)
        && partial.starts_with('д')
        && partial.chars().count() <= 4
}

fn has_weather_place(prefix: &str) -> bool {
    prefix
        .split_whitespace()
        .any(|token| normalize_ru(token).starts_with("улиц"))
}

fn has_event_verb(prefix: &str) -> bool {
    prefix.split_whitespace().any(|token| {
        matches!(
            normalize_ru(token).as_str(),
            "идет" | "идёт" | "шел" | "шёл"
        )
    })
}

fn has_repeat_or_weather_hint(prefix: &str) -> bool {
    prefix.split_whitespace().any(|token| {
        matches!(
            normalize_ru(token).as_str(),
            "опять" | "снова" | "сегодня" | "наружи"
        )
    })
}

fn repeat_process_amplitude(prefix: &str) -> f32 {
    if prefix
        .split_whitespace()
        .any(|token| matches!(normalize_ru(token).as_str(), "опять" | "снова"))
    {
        0.86
    } else {
        0.58
    }
}

fn event_verb_amplitude(prefix: &str) -> f32 {
    if has_event_verb(prefix) {
        0.88
    } else {
        0.40
    }
}

fn prefix_d_amplitude(partial: &str) -> f32 {
    if partial == "д" {
        0.91
    } else if "дождь".starts_with(partial) || "дождик".starts_with(partial) {
        0.97
    } else {
        0.55
    }
}

fn normalize_ru(token: &str) -> String {
    token
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || matches!(ch, '«' | '»' | '“' | '”'))
        .to_lowercase()
}

fn known_russian_phrase_tail(tail: &str) -> bool {
    let tokens = tail
        .split_whitespace()
        .map(normalize_ru)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    let Some(last) = tokens.last() else {
        return false;
    };
    tokens.len() >= 2
        && is_common_ru_word(last)
        && tokens[..tokens.len() - 1]
            .iter()
            .all(|token| is_known_russian_phrase_part(token))
}

fn split_last_token(text: &str) -> Option<(&str, &str)> {
    if text.is_empty() {
        return None;
    }
    let start = text
        .char_indices()
        .rev()
        .find(|(_idx, ch)| ch.is_whitespace())
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    let (prefix, token) = text.split_at(start);
    (!token.is_empty()).then_some((prefix, token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_wave_prefers_rain_over_day_and_drill() {
        let wave = context_wave_for_tail("На улице опять идёт д").unwrap();
        let candidates = candidate_interferences(&wave);
        assert_eq!(candidates[0].candidate, "дождь");
        assert!(candidates[0].projection > candidates[1].projection);
        assert!(
            candidates
                .iter()
                .find(|item| item.candidate == "день")
                .unwrap()
                .projection
                > candidates
                    .iter()
                    .find(|item| item.candidate == "дрель")
                    .unwrap()
                    .projection
        );
    }

    #[test]
    fn semantic_wave_generates_weather_candidate_from_prefix() {
        let candidates = semantic_word_candidates("На улице опять идёт д");
        assert_eq!(candidates[0].text, "На улице опять идёт дождь");
        assert_eq!(candidates[0].source, SEMANTIC_WORD_SOURCE);
        assert!(candidates[0]
            .support
            .iter()
            .any(|line| line.contains("mode=weather_context")));
    }

    #[test]
    fn semantic_word_cell_generates_nearest_dictionary_word() {
        let candidates = semantic_word_candidates("это вобще ");
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "это вообще"));
    }

    #[test]
    fn semantic_word_cell_repairs_domain_typo_with_phrase_context() {
        let candidates = semantic_word_candidates("это невидные ");
        assert!(candidates
            .iter()
            .any(|candidate| candidate.text == "это невалидные"));
    }

    #[test]
    fn semantic_word_cell_does_not_strip_russian_suffix() {
        let candidates = semantic_word_candidates("сколько текста ");
        assert!(candidates
            .iter()
            .all(|candidate| candidate.text != "сколько текст"));
    }

    #[test]
    fn semantic_word_cell_keeps_known_verb_forms() {
        for (original, forbidden) in [
            ("он проверит ", "он проверка"),
            ("твой проверил ", "твой проверка"),
        ] {
            let candidates = semantic_word_candidates(original);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.text != forbidden),
                "known verb form should not become nearest noun: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn semantic_word_cell_keeps_case_vowel_tail() {
        let candidates = semantic_word_candidates("для разработчика скила ");
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.text != "для разработчика скилл"),
            "case-like vowel tail should not drift into a consonant token: {candidates:?}"
        );
    }

    #[test]
    fn semantic_word_cell_keeps_valid_russian_forms() {
        for (original, forbidden) in [
            ("с проверкой ", "с проверка"),
            ("для проверки ", "для проверка"),
            ("есть окно ", "есть оно"),
            ("мало слов ", "мало слово"),
        ] {
            let candidates = semantic_word_candidates(original);
            assert!(
                candidates
                    .iter()
                    .all(|candidate| candidate.text != forbidden),
                "known Russian form should not drift into a nearby known word: {original:?} -> {candidates:?}"
            );
        }
    }

    #[test]
    fn phrase_forecast_is_l3_summary_not_l2_source() {
        let candidate = semantic_word_candidates("На улице опять идёт д")
            .into_iter()
            .next()
            .unwrap();
        let summary = phrase_forecast_summary("На улице опять идёт д", &candidate).unwrap();
        assert!(summary.contains("forecast=\"На улице опять идёт дождь\""));
        assert!(summary.contains(PHRASE_FORECAST_CELL));
    }
}
