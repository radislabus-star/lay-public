use lay::action_log::RecentActionGateTrace;
use lay::config::{CorrectionEngine, CorrectionSafety, TypingAssistRuleConfig};
use lay::correction_core::CorrectionMode;
use lay::engine::{decide_manual_correction, ManualCorrectionInput, ManualCorrectionPolicy};
use lay::input_gate::{decide_input_gate, InputGateRequest, InputGateTrigger};
use lay::keyboard::{
    map_events_to_layout, map_original_events, mark_single_current_word_layout_if_stale,
    replay_layout_decision,
};
use lay::manual_toggle::recovered_initial_double_shift_replacement;
use lay::typing_assist::{
    effective_replace_words, should_force_replay_for_short_fragment, ScopedTailOptions,
};
use std::time::Instant;

use super::auto_undo_runtime::handle_pending_auto_undo;
use super::{
    active_auto_switch_layout, active_lem_enabled_for_scope, active_lem_weight, log,
    log_manual_trigger_cross_check, read_current_layout_is_ru, ExecutingGuard,
};

#[path = "correction_runtime/force_layout.rs"]
mod force_layout;
#[path = "correction_runtime/memory.rs"]
mod memory;
#[path = "correction_runtime/output.rs"]
mod output;
use output::{apply_manual_correction_output, ManualCorrectionOutputContext};
#[cfg(test)]
#[path = "correction_runtime/recovery_tests.rs"]
mod recovery_tests;
#[path = "correction_runtime/request.rs"]
mod request;

pub(super) use force_layout::handle_force_layout_hotkey;
pub(super) use request::{ManualCorrectionRequest, ScopedManualCorrectionRequest};

pub(super) fn run_manual_correction_with_scope(
    req: ScopedManualCorrectionRequest<'_, '_>,
) -> Option<bool> {
    let label = req.label;
    let replace_words = req.manual.replace_words;
    log_manual_trigger_cross_check(req.manual.buf, req.events_since_word_start);
    let result = handle_double_shift(ManualCorrectionRequest {
        buf: req.manual.buf,
        replace_words,
        engine: req.manual.engine,
        auto_replace: req.manual.auto_replace,
        virtual_kbd: req.manual.virtual_kbd,
        executing: req.manual.executing,
        input_isolated: req.manual.input_isolated,
        physical_grab: req.manual.physical_grab,
    });
    log(&format!("· {label} fired with scope={replace_words}"));
    result
}

pub(super) fn handle_double_shift(req: ManualCorrectionRequest<'_, '_>) -> Option<bool> {
    let ManualCorrectionRequest {
        buf,
        replace_words,
        engine,
        auto_replace,
        virtual_kbd,
        executing,
        input_isolated,
        physical_grab,
    } = req;
    let started_at = Instant::now();
    if let Some(undo) = buf.take_pending_auto_undo() {
        return handle_pending_auto_undo(buf, undo, virtual_kbd, executing, started_at);
    }

    let replace_words = effective_replace_words(buf, replace_words, engine, auto_replace);
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
    let input_gate = manual_toggle_gate_trace(&mapped_orig, auto_replace);
    let mapped_target = map_events_to_layout(&events, target_is_ru);
    let chars_orig = mapped_orig.chars().count();
    let chars_target = mapped_target.chars().count();
    let words_orig = mapped_orig.split_whitespace().count();
    let effective_mapped_target = recovered_initial_manual_toggle_target(
        &mapped_orig,
        &mapped_target,
        words_orig,
        n_backspaces,
    );
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

    let force_short_replay = should_force_replay_for_short_fragment(&mapped_orig);
    let force_replay_toggle =
        engine == CorrectionEngine::Smart && (buf.replay_toggle_ready() || force_short_replay);
    if force_replay_toggle {
        log("  smart: replay без модели");
    }
    let scoped_options = ScopedTailOptions {
        lem_enabled: active_lem_enabled_for_scope(words_orig),
        allow_layout_auto: active_auto_switch_layout(),
        lem_weight: active_lem_weight(),
    };
    let correction_result = decide_manual_correction(
        ManualCorrectionInput {
            events: &events,
            original: &mapped_orig,
            converted: &effective_mapped_target,
        },
        ManualCorrectionPolicy {
            engine,
            force_replay: force_replay_toggle,
            auto_replace,
            scoped_options,
        },
    );
    apply_manual_correction_output(
        ManualCorrectionOutputContext {
            buf,
            events: &events,
            mapped_orig: &mapped_orig,
            mapped_target: &effective_mapped_target,
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
        },
        input_gate,
    )
}

fn manual_toggle_gate_trace(text_tail: &str, auto_replace: bool) -> Option<RecentActionGateTrace> {
    let empty_pipeline: &[TypingAssistRuleConfig] = &[];
    let decision = decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::DoubleShift,
        text_tail,
        auto_replace,
        typing_assist: false,
        auto_switch_layout: active_auto_switch_layout(),
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: empty_pipeline,
        nanda_autocorrect: false,
        nanda_wave_options: lay::nanda_wave::WaveOptions::default(),
        correction_mode: CorrectionMode::DeterministicOnly,
    });
    decision
        .trace
        .as_ref()
        .map(RecentActionGateTrace::from_input_gate)
}

fn recovered_initial_manual_toggle_target(
    mapped_orig: &str,
    mapped_target: &str,
    words_orig: usize,
    n_backspaces: u32,
) -> String {
    if words_orig != 1 || n_backspaces != mapped_orig.chars().count() as u32 {
        return mapped_target.to_string();
    }
    recovered_initial_double_shift_replacement(mapped_orig)
        .filter(|replacement| replacement != mapped_target)
        .unwrap_or_else(|| mapped_target.to_string())
}
