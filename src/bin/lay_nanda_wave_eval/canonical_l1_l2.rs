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
    println!("  source: lay real-suite words + seed service words");
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
    let words = load_words(WordSource::ExpectedOnly)?;
    let report = canonical_l2_candidate_report(&words, input, limit);

    println!("canonical_l2_candidates:");
    println!("  input: {}", report.input);
    println!("  source: lay real-suite words + seed service words");
    println!("  words: {}", report.words);
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
            "    {} score={} l1_overlap={} l2_overlap={} motif_overlap={} prefix={}",
            candidate.word,
            candidate.score,
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

    Ok(words.into_iter().collect())
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
        if !word.is_empty() {
            out.insert(word);
        }
    }
}

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
}
