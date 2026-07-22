use super::feedback::derive_l3_feedback;
use super::l1::run_l1_with_options;
use super::l2::{run_l2_refined_with_feedback, run_l2_with_options};
use super::l3::run_l3_with_options;
use super::l4_goal_state::derive_l4_goal_state_trace;
use super::llmwave::derive_llmwave_feedback;
use super::options::WaveOptions;
use super::signal::WaveTrace;

pub fn run_wave_trace(original: &str) -> WaveTrace {
    run_wave_trace_with_options(original, &WaveOptions::default())
}

pub fn run_wave_trace_with_options(original: &str, options: &WaveOptions) -> WaveTrace {
    let l1 = run_l1_with_options(original, options);
    let initial_l2 = run_l2_with_options(original, &l1, options);
    let (mut l3_feedback, feedback) = derive_l3_feedback(original, &initial_l2, options);
    let (mut llmwave_trace, llmwave_feedback) =
        derive_llmwave_feedback(original, &initial_l2, options);
    if let Some(l4_trace) = derive_l4_goal_state_trace(original, options) {
        llmwave_trace.push(l4_trace);
    }
    if options.l3_phase_shadow() {
        llmwave_trace.push(super::signal::LayerTrace {
            name: "L3PhaseContextShadow",
            summary: format!(
                "status=WATCH-no-package initial_l2_candidates={}",
                initial_l2.len()
            ),
        });
    }
    let feedback = merge_feedback(feedback, llmwave_feedback);
    let l2_candidates = run_l2_refined_with_feedback(original, &l1, options, &feedback);
    let (mut l3, decision) = run_l3_with_options(original, &l2_candidates, options);
    l3_feedback.append(&mut llmwave_trace);
    l3_feedback.append(&mut l3);
    WaveTrace {
        original: original.to_string(),
        l1,
        l2_candidates,
        l3: l3_feedback,
        decision,
    }
}

fn merge_feedback(
    mut left: super::feedback::L3Feedback,
    mut right: super::feedback::L3Feedback,
) -> super::feedback::L3Feedback {
    left.adjustments.append(&mut right.adjustments);
    left
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

    #[test]
    fn llmwave_shadow_adds_trace_without_changing_default_decision() {
        let plain = run_wave_trace("html djn ");
        let options = WaveOptions::default().with_llmwave_shadow(true);
        let shadow = run_wave_trace_with_options("html djn ", &options);
        assert_eq!(plain.decision, shadow.decision);
        assert!(shadow
            .l3
            .iter()
            .any(|layer| layer.name == crate::nanda_wave::llmwave::LLMWAVE_CELL));
    }

    #[test]
    fn l3_phase_shadow_is_watch_only_trace() {
        let plain = run_wave_trace("html djn ");
        let options = WaveOptions::default().with_l3_phase_shadow(true);
        let shadow = run_wave_trace_with_options("html djn ", &options);
        assert_eq!(plain.decision, shadow.decision);
        assert!(shadow.l3.iter().any(|layer| {
            layer.name == "L3PhaseContextShadow" && layer.summary.contains("WATCH-no-package")
        }));
    }

    #[test]
    fn trace_prefers_hidden_boundary_over_single_word_typo() {
        let trace = run_wave_trace("влогах ");
        assert_eq!(trace.output(), Some("в логах "));
        assert!(trace.l2_candidates.iter().any(|candidate| {
            candidate.source == "BoundaryCell32" && candidate.text == "в логах"
        }));
    }

    #[test]
    fn trace_prefers_direct_boundary_phrase_over_word_form_attractor() {
        let trace = run_wave_trace("тоесть ");
        assert_eq!(trace.output(), Some("то есть "));
        assert!(trace.l2_candidates.iter().any(|candidate| {
            candidate.source == "BoundaryCell32" && candidate.text == "то есть"
        }));
    }

    #[test]
    fn trace_layout_flip_works_in_both_directions() {
        let en_to_ru = run_wave_trace("ghbdtn ");
        assert_eq!(en_to_ru.output(), Some("привет "));
        assert!(en_to_ru
            .l2_candidates
            .iter()
            .any(|candidate| candidate.source == "LayoutWordCell32"));

        let ru_to_en = run_wave_trace("руддщ ");
        assert_eq!(ru_to_en.output(), Some("hello "));
        assert!(ru_to_en.l2_candidates.iter().any(|candidate| {
            matches!(
                candidate.source,
                "LayoutWordCell32" | "layout_then_l2_word_center"
            )
        }));
    }

    #[test]
    fn trace_layout_flip_uses_exact_english_reference_centers() {
        for (input, expected) in [("зщке ", "port "), ("сфкпщ ", "cargo ")] {
            let trace = run_wave_trace(input);
            assert_eq!(trace.output(), Some(expected), "trace={trace:?}");
            assert!(trace.l2_candidates.iter().any(|candidate| {
                matches!(
                    candidate.source,
                    "LayoutWordCell32" | "layout_then_l2_word_center"
                ) && candidate.text == expected.trim()
            }));
        }
    }

    #[test]
    fn trace_repairs_internal_char_move_geometry() {
        let trace = run_wave_trace("ктороый ");
        assert_eq!(trace.output(), Some("который "), "trace={trace:?}");
    }

    #[test]
    fn trace_keeps_single_all_caps_russian_term() {
        let trace = run_wave_trace("БЕЙСОВКИ ");
        assert_eq!(trace.output(), None);
        assert!(trace
            .l2_candidates
            .iter()
            .all(|candidate| candidate.source != "PhraseCell32"));
    }

    #[test]
    fn trace_keeps_live_log_semantic_word_drifts() {
        for original in ["модель генерит ", "окончанием слов "] {
            let trace = run_wave_trace(original);
            assert_eq!(trace.output(), None, "original={original:?}: {trace:?}");
        }
    }

    #[test]
    fn trace_keeps_live_log_l2_boundary_and_attractor_drifts() {
        for original in ["улетели ", "кодировании ", "тестируй ", "отправляй "]
        {
            let trace = run_wave_trace(original);
            assert_eq!(trace.output(), None, "original={original:?}: {trace:?}");
        }
    }

    #[test]
    fn trace_still_repairs_clear_semantic_transposition() {
        let trace = run_wave_trace("делай инстурменты ");
        assert_eq!(trace.output(), Some("делай инструменты "));
    }
}
