use lay::config::CorrectionEngine;
use lay::config::{CorrectionSafety, TypingAssistRuleConfig};
use lay::correction_core::CorrectionMode;
use lay::engine::{decide_manual_correction, ManualCorrectionInput, ManualCorrectionPolicy};
use lay::input_gate::{decide_input_gate, InputGateRequest, InputGateTrigger};
use lay::keyboard::{
    map_events_to_layout, map_original_events, mark_single_current_word_layout_if_stale,
    replay_layout_decision, KeyEvent,
};
use std::time::Instant;

use super::auto_undo_runtime::handle_pending_auto_undo;
use super::{
    active_auto_switch_layout, capture_ime_delegated_tail_lease, log,
    log_manual_trigger_cross_check, read_current_layout_is_ru, ExecutingGuard,
};

#[path = "correction_runtime/force_layout.rs"]
mod force_layout;
#[path = "correction_runtime/memory.rs"]
mod memory;
#[path = "correction_runtime/output.rs"]
mod output;
use output::{apply_manual_correction_output, ManualCorrectionOutputContext};
#[path = "correction_runtime/request.rs"]
mod request;

pub(super) use force_layout::handle_force_layout_hotkey;
pub(super) use request::{
    ManualCorrectionOutputRoute, ManualCorrectionRequest, ScopedManualCorrectionRequest,
};

pub(super) fn run_manual_correction_with_scope(
    req: ScopedManualCorrectionRequest<'_, '_>,
) -> Option<bool> {
    let label = req.label;
    let replace_words = req.manual.replace_words;
    log_manual_trigger_cross_check(req.manual.buf, req.events_since_word_start);
    let result = handle_double_shift(ManualCorrectionRequest {
        replace_words,
        output_route: req.manual.output_route,
        ..req.manual
    });
    log(&format!("· {label} fired with scope={replace_words}"));
    result
}

pub(super) fn handle_double_shift(req: ManualCorrectionRequest<'_, '_>) -> Option<bool> {
    let ManualCorrectionRequest {
        buf,
        replace_words,
        virtual_kbd,
        executing,
        input_isolated,
        text_observation,
        physical_grab,
        output_route,
    } = req;
    let started_at = Instant::now();
    if let Some(undo) = buf.take_pending_auto_undo() {
        return handle_pending_auto_undo(
            buf,
            undo,
            virtual_kbd,
            executing,
            started_at,
            input_isolated,
            text_observation,
        );
    }

    let Some((mut events, n_backspaces)) = buf.what_to_replay(replace_words) else {
        log("👆 двойной Shift, но буфер пуст");
        return None;
    };
    *executing = true; // блокируем Shift events на время выполнения
    let _executing_guard = ExecutingGuard(executing);

    if let Ok(current_layout_is_ru) = read_current_layout_is_ru() {
        if mark_single_current_word_layout_if_stale(&mut events, current_layout_is_ru) {
            log(&format!(
                "· manual tail layout resynced to {} before replay",
                if current_layout_is_ru { "ru" } else { "us" }
            ));
        }
    }

    let layout_decision = replay_layout_decision(&events);
    let target_is_ru = layout_decision.target_is_ru;
    let mixed_layouts = layout_decision.mixed_layouts;

    let mapped_orig = map_original_events(&events);
    let delegated_tail_lease = if output_route == ManualCorrectionOutputRoute::DaemonUinput {
        if !input_isolated {
            log("⚠ delegated manual replay blocked: physical input is not isolated");
            return None;
        }
        match capture_ime_delegated_tail_lease(&mapped_orig, n_backspaces) {
            Ok(lease) => Some(lease),
            Err(error) => {
                log(&format!(
                    "⚠ delegated manual replay blocked before mutation: {error}"
                ));
                return None;
            }
        }
    } else {
        None
    };
    let empty_pipeline: &[TypingAssistRuleConfig] = &[];
    let input_gate = decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::DoubleShift,
        text_tail: &mapped_orig,
        auto_replace: false,
        typing_assist: false,
        auto_switch_layout: active_auto_switch_layout(),
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: empty_pipeline,
        nanda_autocorrect: false,
        nanda_candidate_route: lay::correction_core::CandidateReadoutRoute::live_default(),
        nanda_wave_options: lay::typing_cpu::TypingCpuOptions::default(),
        correction_mode: CorrectionMode::DeterministicOnly,
    })
    .trace
    .as_ref()
    .map(lay::action_log::RecentActionGateTrace::from_input_gate);
    let mapped_target = map_events_to_layout(&events, target_is_ru);
    let chars_orig = mapped_orig.chars().count();
    let chars_target = mapped_target.chars().count();
    let words_orig = mapped_orig.split_whitespace().count();
    let mismatch = chars_orig != events.len() || chars_target != events.len();
    log(&format!(
        "👆 events={} n_bs={n_backspaces} | chars_orig={chars_orig} chars_target={chars_target} words={words_orig} {} mixed={} | orig={mapped_orig:?} → target={mapped_target:?}",
        events.len(),
        if mismatch { "⚠ MAP-MISMATCH" } else { "✓" },
        mixed_layouts,
    ));

    if mapped_target.is_empty() {
        log("⚠ mapped_target пуст — не вставляем");
        return None;
    }
    // ═══ АЛГОРИТМ: decision layer → backspace → replay/text insert ═══

    // Ordinary Double Shift is a physical layout projection, not correction.
    let force_replay_toggle = true;
    log("  deterministic: physical key replay without model or learning");
    let correction_result = decide_physical_layout_replay(&events, &mapped_orig, &mapped_target);
    apply_manual_correction_output(
        ManualCorrectionOutputContext {
            buf,
            events: &events,
            mapped_orig: &mapped_orig,
            mapped_target: &mapped_target,
            target_is_ru,
            n_backspaces,
            replace_words,
            words_orig,
            force_replay_toggle,
            started_at,
            decision: &correction_result,
            virtual_kbd,
            physical_grab,
            input_isolated,
            text_observation,
            output_route,
            delegated_tail_lease,
        },
        input_gate,
    )
}

fn decide_physical_layout_replay(
    events: &[KeyEvent],
    original: &str,
    converted: &str,
) -> lay::engine::ManualCorrectionDecision {
    decide_manual_correction(
        ManualCorrectionInput {
            events,
            original,
            converted,
        },
        ManualCorrectionPolicy {
            engine: CorrectionEngine::Replay,
            force_replay: true,
            auto_replace: false,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::decide_physical_layout_replay;
    use lay::decoder::DecoderAction;
    use lay::keyboard::{map_events_to_layout, map_original_events, text_to_key_events};

    #[test]
    fn ordinary_double_shift_is_exact_replay_without_smart_replacement() {
        let events = text_to_key_events("абвгд", true).expect("physical key events");
        let original = map_original_events(&events);
        let converted = map_events_to_layout(&events, false);

        let decision = decide_physical_layout_replay(&events, &original, &converted);

        assert_eq!(decision.action, DecoderAction::ReplayAll);
        assert_eq!(decision.output_text, converted);
        assert!(!decision.output_target_is_ru);
    }
}
