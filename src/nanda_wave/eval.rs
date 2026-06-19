use crate::eval_cases::EvalCase;

use super::options::WaveOptions;
use super::trace::run_wave_trace_with_options;

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
}
