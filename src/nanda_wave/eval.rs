use super::lexical_phase::{compile_words, LexicalPhaseMemory};
use super::options::WaveOptions;
use super::surface_wave::{SurfaceWave4096, SURFACE_WAVE_BYTES};
use super::trace::run_wave_trace_with_options;
use crate::eval_cases::EvalCase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveEvalResult {
    pub output: String,
    pub ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveEvalStats {
    pub cases: usize,
    pub ok: usize,
    pub changed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalL1L2Report {
    pub words: usize,
    pub l1_centers: usize,
    pub l1_word_records: usize,
    pub l1_sequence_refs: usize,
    pub l1_hot_bytes: usize,
    pub l2_motifs: usize,
    pub l2_word_records: usize,
    pub l2_token_refs: usize,
    pub hot_bytes: usize,
    pub naive_wave_bytes: usize,
    pub probes: Vec<CanonicalL1L2Probe>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalL1L2Probe {
    pub text: String,
    pub l1_ngrams: usize,
    pub l1_refs: usize,
    pub l1_residual: usize,
    pub wave_active_lanes: usize,
    pub l2_tokens: usize,
    pub l2_motifs: usize,
    pub l2_residual: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalL2CandidateReport {
    pub input: String,
    pub words: usize,
    pub l1_ngrams: usize,
    pub l1_refs: usize,
    pub l1_residual: usize,
    pub l2_tokens: usize,
    pub l2_motifs: usize,
    pub l2_residual: usize,
    pub candidates: Vec<CanonicalL2Candidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalL2Candidate {
    pub word: String,
    pub score: u32,
    pub l1_overlap: usize,
    pub l2_overlap: usize,
    pub motif_overlap: usize,
    pub prefix_match: bool,
}

pub struct CanonicalL2CandidateEngine {
    words: usize,
    memory: LexicalPhaseMemory,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RuntimeL2LexicalMemoryStats {
    pub source_words: usize,
    pub l1_centers: usize,
    pub l1_postings: usize,
    pub l2_word_centers: usize,
    pub grapheme_nodes: usize,
    pub grapheme_arcs: usize,
    pub decoder_states: usize,
    pub decoder_arcs: usize,
    pub training_surfaces: usize,
    pub hot_bytes: usize,
    pub mmap_backed: bool,
    pub raw_word_table: bool,
}

impl CanonicalL2CandidateEngine {
    pub fn new(words: &[String]) -> Self {
        let memory = canonical_l2_memory(words);
        Self {
            words: words.len(),
            memory,
        }
    }

    pub fn candidate_report(&self, input: &str, limit: usize) -> CanonicalL2CandidateReport {
        canonical_l2_candidate_report_with_memory(self.words, &self.memory, input, limit)
    }
}

pub fn evaluate_wave(cases: &[EvalCase]) -> (Vec<WaveEvalResult>, WaveEvalStats) {
    evaluate_wave_with_options(cases, &WaveOptions::default())
}

pub fn evaluate_wave_with_options(
    cases: &[EvalCase],
    options: &WaveOptions,
) -> (Vec<WaveEvalResult>, WaveEvalStats) {
    let results = cases
        .iter()
        .map(|case| {
            let trace = run_wave_trace_with_options(&case.original, options);
            let output = trace
                .output()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| case.original.clone());
            WaveEvalResult {
                ok: output == case.expected,
                output,
            }
        })
        .collect::<Vec<_>>();
    let stats = WaveEvalStats {
        cases: results.len(),
        ok: results.iter().filter(|result| result.ok).count(),
        changed: results
            .iter()
            .zip(cases)
            .filter(|(result, case)| result.output != case.original)
            .count(),
    };
    (results, stats)
}

pub fn canonical_l1_l2_shadow_report(words: &[String], probes: &[String]) -> CanonicalL1L2Report {
    let memory = canonical_l2_memory(words);
    let probes = probes
        .iter()
        .map(|probe| {
            let readout = memory.phase_readout(probe);
            let wave_active_lanes = SurfaceWave4096::compile(probe).active_lanes();
            CanonicalL1L2Probe {
                text: probe.clone(),
                l1_ngrams: readout.atom_count,
                l1_refs: readout.center_hits,
                l1_residual: readout.atom_count.saturating_sub(readout.center_hits),
                wave_active_lanes,
                l2_tokens: readout.atom_count,
                l2_motifs: readout.center_hits,
                l2_residual: readout.atom_count.saturating_sub(readout.center_hits),
            }
        })
        .collect();

    let stats = memory.stats();

    CanonicalL1L2Report {
        words: words.len(),
        l1_centers: stats.l1_centers,
        l1_word_records: 0,
        l1_sequence_refs: stats.l1_postings,
        l1_hot_bytes: stats.hot_bytes,
        l2_motifs: stats.l2_word_centers,
        l2_word_records: stats.l2_word_centers,
        l2_token_refs: stats.l1_postings,
        hot_bytes: stats.hot_bytes,
        naive_wave_bytes: words.len() * SURFACE_WAVE_BYTES,
        probes,
    }
}

pub fn canonical_l2_candidate_report(
    words: &[String],
    input: &str,
    limit: usize,
) -> CanonicalL2CandidateReport {
    CanonicalL2CandidateEngine::new(words).candidate_report(input, limit)
}

pub fn runtime_l2_lexical_memory_stats() -> Option<RuntimeL2LexicalMemoryStats> {
    let stats = super::lexical_phase::default_memory()?.stats();
    Some(RuntimeL2LexicalMemoryStats {
        source_words: stats.source_words,
        l1_centers: stats.l1_centers,
        l1_postings: stats.l1_postings,
        l2_word_centers: stats.l2_word_centers,
        grapheme_nodes: stats.grapheme_nodes,
        grapheme_arcs: stats.grapheme_arcs,
        decoder_states: stats.decoder_states,
        decoder_arcs: stats.decoder_arcs,
        training_surfaces: stats.training_surfaces,
        hot_bytes: stats.hot_bytes,
        mmap_backed: stats.mmap_backed,
        raw_word_table: stats.raw_word_table,
    })
}

fn canonical_l2_candidate_report_with_memory(
    words_len: usize,
    memory: &LexicalPhaseMemory,
    input: &str,
    limit: usize,
) -> CanonicalL2CandidateReport {
    let readout = memory.phase_readout(input);
    let candidates = memory
        .surface_candidates(input, limit)
        .into_iter()
        .map(|candidate| CanonicalL2Candidate {
            word: candidate.word,
            score: candidate.score,
            l1_overlap: candidate.l1_overlap,
            l2_overlap: candidate.l2_overlap,
            motif_overlap: candidate.motif_overlap,
            prefix_match: candidate.prefix_match,
        })
        .collect::<Vec<_>>();

    CanonicalL2CandidateReport {
        input: input.to_string(),
        words: words_len,
        l1_ngrams: readout.atom_count,
        l1_refs: readout.center_hits,
        l1_residual: readout.atom_count.saturating_sub(readout.center_hits),
        l2_tokens: readout.atom_count,
        l2_motifs: readout.center_hits,
        l2_residual: readout.atom_count.saturating_sub(readout.center_hits),
        candidates,
    }
}

fn canonical_l2_memory(words: &[String]) -> LexicalPhaseMemory {
    let bytes = compile_words(words.iter().map(String::as_str))
        .expect("canonical L1/L2 eval requires at least one valid word");
    LexicalPhaseMemory::from_bytes(bytes).expect("compiled canonical L1/L2 artifact must load")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_wave_cases() {
        let cases = vec![EvalCase {
            original: "html djn ".to_string(),
            expected: "html вот ".to_string(),
            reason: "wave_smoke".to_string(),
        }];
        let (_results, stats) = evaluate_wave(&cases);
        assert_eq!(stats.cases, 1);
    }

    #[test]
    fn canonical_l1_l2_report_keeps_short_words_visible() {
        let words = vec![
            "и".to_string(),
            "в".to_string(),
            "не".to_string(),
            "проверка".to_string(),
            "проверки".to_string(),
            "проверяем".to_string(),
        ];
        let probes = vec!["и".to_string(), "в".to_string(), "проверка".to_string()];
        let report = canonical_l1_l2_shadow_report(&words, &probes);
        assert!(report.l1_centers > 0);
        assert!(report.probes.iter().all(|probe| probe.l1_ngrams > 0));
        assert!(report.probes.iter().all(|probe| probe.l1_refs > 0));
    }

    #[test]
    fn canonical_l2_candidates_rank_center_mass() {
        let words = vec![
            "проверка".to_string(),
            "проверки".to_string(),
            "проверяем".to_string(),
            "автозамена".to_string(),
            "переворачивает".to_string(),
        ];
        let report = canonical_l2_candidate_report(&words, "проверк", 3);
        assert_eq!(report.input, "проверк");
        assert!(!report.candidates.is_empty());
        assert_eq!(report.candidates[0].word, "проверка");
        assert!(report.candidates[0].score > 0);
    }

    #[test]
    fn canonical_l2_candidates_use_surface_distance_as_shadow_signal() {
        let words = vec![
            "теперь".to_string(),
            "проблема".to_string(),
            "эксперимент".to_string(),
            "эффективная".to_string(),
        ];
        let report = canonical_l2_candidate_report(&words, "эсперемнт", 4);

        assert_eq!(report.candidates[0].word, "эксперимент");
        assert!(report
            .candidates
            .get(1)
            .is_none_or(|runner_up| { report.candidates[0].score > runner_up.score }));
    }
}
