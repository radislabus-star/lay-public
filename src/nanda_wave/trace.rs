use super::feedback::derive_l3_feedback;
use super::l1::run_l1_with_options;
use super::l2::{run_l2_refined_with_feedback, run_l2_with_options};
use super::l3::run_l3_with_options;
use super::options::WaveOptions;
use super::signal::WaveTrace;

pub fn run_wave_trace(original: &str) -> WaveTrace {
    run_wave_trace_with_options(original, &WaveOptions::default())
}

pub fn run_wave_trace_with_options(original: &str, options: &WaveOptions) -> WaveTrace {
    let l1 = run_l1_with_options(original, options);
    let initial_l2 = run_l2_with_options(original, &l1, options);
    let (mut l3_feedback, feedback) = derive_l3_feedback(original, &initial_l2, options);
    let l2_candidates = run_l2_refined_with_feedback(original, &l1, options, &feedback);
    let (mut l3, decision) = run_l3_with_options(original, &l2_candidates, options);
    l3_feedback.append(&mut l3);
    WaveTrace {
        original: original.to_string(),
        l1,
        l2_candidates,
        l3: l3_feedback,
        decision,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_has_all_layers() {
        let trace = run_wave_trace("html djn ");
        assert!(!trace.l1.is_empty());
        assert!(!trace.l2_candidates.is_empty());
        assert!(!trace.l3.is_empty());
    }

    #[test]
    fn trace_runs_l3_feedback_before_final_l3_decision() {
        let trace = run_wave_trace("html djn ");
        assert_eq!(
            trace.l3.first().map(|item| item.name),
            Some(crate::nanda_wave::feedback::L3_FEEDBACK_CELL)
        );
        assert!(trace.l2_candidates.iter().any(|candidate| candidate
            .support
            .iter()
            .any(|item| item.starts_with("l3-feedback:"))));
    }
}
