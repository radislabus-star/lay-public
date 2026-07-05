use lay::nanda_wave::eval::{canonical_l1_l2_shadow_report, canonical_l2_candidate_report};
use std::collections::BTreeSet;
use std::io;

use super::{arg_values, real_suite};

pub(crate) fn print_report(args: &[String]) -> io::Result<()> {
    let words = load_words(WordSource::OriginalAndExpected)?;
    let probes = {
        let explicit = arg_values(args, "--probe");
        if explicit.is_empty() {
            default_probes()
        } else {
            explicit
        }
    };
    let report = canonical_l1_l2_shadow_report(&words, &probes);

    println!("canonical_l1_l2_shadow:");
    println!("  source: lay real-suite + local lexicon shadow words");
    println!("  words: {}", report.words);
    println!("  l1_centers: {}", report.l1_centers);
    println!("  l1_word_records: {}", report.l1_word_records);
    println!("  l1_sequence_refs: {}", report.l1_sequence_refs);
    println!("  l1_hot_bytes: {}", report.l1_hot_bytes);
    println!("  l2_motifs: {}", report.l2_motifs);
    println!("  l2_word_records: {}", report.l2_word_records);
    println!("  l2_token_refs: {}", report.l2_token_refs);
    println!("  hot_bytes: {}", report.hot_bytes);
    println!("  naive_wave_bytes: {}", report.naive_wave_bytes);
    println!("  live_authority: false");
    println!("  layer_contract: L1/L2 canonical shadow only; L3/runtime not changed");
    println!("  probes:");
    for probe in report.probes {
        println!(
            "    {}: l1_ngrams={} l1_refs={} l1_residual={} wave_lanes={} l2_tokens={} l2_motifs={} l2_residual={}",
            probe.text,
            probe.l1_ngrams,
            probe.l1_refs,
            probe.l1_residual,
            probe.wave_active_lanes,
            probe.l2_tokens,
            probe.l2_motifs,
            probe.l2_residual
        );
    }

    Ok(())
}

pub(crate) fn print_candidates(args: &[String]) -> io::Result<()> {
    let Some(input) = super::arg_value(args, "--canonical-l2-candidates") else {
        eprintln!("--canonical-l2-candidates requires TEXT");
        return Ok(());
    };
    let limit = super::arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(12)
        .clamp(1, 50);
    let word_limit = super::arg_value(args, "--word-limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1_000)
        .clamp(100, 50_000);
    let words = load_words_limited(WordSource::ExpectedOnly, word_limit)?;
    let report = canonical_l2_candidate_report(&words, input, limit);

    println!("canonical_l2_candidates:");
    println!("  input: {}", report.input);
    println!("  source: lay real-suite + local lexicon shadow words");
    println!("  words: {}", report.words);
    println!("  word_limit: {}", word_limit);
    if let Some(debug_word) = super::arg_value(args, "--debug-word") {
        let debug_word = normalize_surface(debug_word);
        let position = words.iter().position(|word| word == &debug_word);
        let usage_words = lay::nanda_wave::l2_surface_words_by_usage(1_000);
        let usage_position = usage_words.iter().position(|word| word == &debug_word);
        let usage_debug = lay::nanda_wave::usage_debug_summary();
        let usage_bytes = std::env::var_os("LAY_NANDA_WORD_USAGE_EVENTS")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| {
                    std::path::PathBuf::from(home)
                        .join(".local/share/lay/nanda_wave/word_usage_events.jsonl")
                })
            })
            .and_then(|path| std::fs::metadata(path).ok())
            .map(|meta| meta.len())
            .unwrap_or(0);
        println!(
            "  debug_word: {} present={} rank={} usage_present={} usage_rank={} usage_words={} usage_bytes={} usage_debug_bytes={} usage_debug_parsed={} usage_debug_word_counts={}",
            debug_word,
            position.is_some(),
            position.map(|index| index + 1).unwrap_or(0),
            usage_position.is_some(),
            usage_position.map(|index| index + 1).unwrap_or(0),
            usage_words.len(),
            usage_bytes,
            usage_debug.0,
            usage_debug.1,
            usage_debug.2
        );
    }
    println!(
        "  input_l1: ngrams={} refs={} residual={}",
        report.l1_ngrams, report.l1_refs, report.l1_residual
    );
    println!(
        "  input_l2: tokens={} motifs={} residual={}",
        report.l2_tokens, report.l2_motifs, report.l2_residual
    );
    println!("  live_authority: false");
    println!("  candidates:");
    for candidate in report.candidates {
        println!(
            "    {} score={} distance={} l1_overlap={} l2_overlap={} motif_overlap={} prefix={}",
            candidate.word,
            candidate.score,
            surface_distance(input, &candidate.word),
            candidate.l1_overlap,
            candidate.l2_overlap,
            candidate.motif_overlap,
            candidate.prefix_match
        );
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum WordSource {
    OriginalAndExpected,
    ExpectedOnly,
}

fn load_words(source: WordSource) -> io::Result<Vec<String>> {
    load_words_limited(source, usize::MAX)
}

fn load_words_limited(source: WordSource, limit: usize) -> io::Result<Vec<String>> {
    let suite = real_suite::load()?;
    let mut words = BTreeSet::new();
    for case in &suite.cases {
        if matches!(source, WordSource::OriginalAndExpected) {
            collect_words(&case.original, &mut words);
        }
        collect_words(&case.expected, &mut words);
    }
    for word in seed_words() {
        words.insert(word.to_string());
    }
    collect_shadow_lexicon_words(&mut words);
    words.extend(lay::nanda_wave::l2_surface_words_by_usage(1_000));

    Ok(lay::nanda_wave::balanced_l2_surface_words(
        words.into_iter().collect::<Vec<_>>(),
        limit,
    ))
}

fn default_probes() -> Vec<String> {
    vec![
        "и".to_string(),
        "в".to_string(),
        "не".to_string(),
        "проверка".to_string(),
        "автозамена".to_string(),
        "переворачивает".to_string(),
    ]
}

fn collect_words(text: &str, out: &mut BTreeSet<String>) {
    for token in text.split_whitespace() {
        let word = token
            .chars()
            .filter(|ch| ch.is_alphabetic() || *ch == '-')
            .flat_map(char::to_lowercase)
            .collect::<String>();
        if let Some(word) = lay::nanda_wave::normalize_l2_surface_word(&word) {
            out.insert(word);
        }
    }
}

fn collect_shadow_lexicon_words(out: &mut BTreeSet<String>) {
    for text in SHADOW_WORD_TEXTS {
        collect_words(text, out);
    }
    collect_synthetic_expected_words(
        include_str!("../../../data/nanda_wave_synthetic_cases.tsv"),
        out,
    );
    collect_generated_positive_candidates(
        include_str!("../../../data/nanda_training/generated_cases.tsv"),
        out,
    );
}

fn collect_synthetic_expected_words(text: &str, out: &mut BTreeSet<String>) {
    for line in text
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
    {
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() >= 2 {
            collect_words(&decode_fixture_spaces(cols[1]), out);
        }
    }
}

fn collect_generated_positive_candidates(text: &str, out: &mut BTreeSet<String>) {
    for line in text.lines().skip(1) {
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() >= 6 && cols[5] == "1" {
            collect_words(&decode_fixture_spaces(cols[3]), out);
        }
    }
}

fn decode_fixture_spaces(text: &str) -> String {
    text.replace("\\s", " ")
}

fn surface_distance(left: &str, right: &str) -> usize {
    edit_distance(&normalize_surface(left), &normalize_surface(right))
}

fn normalize_surface(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_alphabetic() || *ch == '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn edit_distance(left: &str, right: &str) -> usize {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    let mut prev = (0..=right.len()).collect::<Vec<_>>();
    let mut curr = vec![0usize; right.len() + 1];
    for (left_idx, left_ch) in left.iter().enumerate() {
        curr[0] = left_idx + 1;
        for (right_idx, right_ch) in right.iter().enumerate() {
            let replace_cost = usize::from(left_ch != right_ch);
            curr[right_idx + 1] = (prev[right_idx + 1] + 1)
                .min(curr[right_idx] + 1)
                .min(prev[right_idx] + replace_cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[right.len()]
}

const SHADOW_WORD_TEXTS: &[&str] = &[
    include_str!("../../../data/lem_research/ru_words.txt"),
    include_str!("../../../data/lexicon/common_ru.txt"),
    include_str!("../../../data/lexicon/l2_surface_hot_ru.txt"),
    include_str!("../../../tests/fixtures/russian_forms.txt"),
    include_str!("../../../tests/fixtures/ngram_ru_train_words.txt"),
];

fn seed_words() -> &'static [&'static str] {
    &[
        "и", "в", "не", "на", "с", "по", "для", "как", "что", "я", "ты", "мы", "он", "она", "они",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_l1_l2_report_input_keeps_short_service_words() {
        let mut words = BTreeSet::new();
        collect_words("wave и context в поле не теряются", &mut words);
        for word in seed_words() {
            words.insert((*word).to_string());
        }

        assert!(words.contains("и"));
        assert!(words.contains("в"));
        assert!(words.contains("не"));
        assert!(words.contains("wave"));
        assert!(words.contains("context"));
    }

    #[test]
    fn canonical_l2_shadow_words_include_local_lexicon() {
        let words = load_words(WordSource::ExpectedOnly).expect("candidate words");

        assert!(words.contains(&"эксперимент".to_string()));
        assert!(words.contains(&"эффективная".to_string()));
        assert!(words.contains(&"другие".to_string()));
        assert!(words.contains(&"видеть".to_string()));
    }
}
