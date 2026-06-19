use evdev::uinput::VirtualDevice;
use lay::config::CorrectionEngine;
use lay::engine::{decide_manual_correction, ManualCorrectionInput, ManualCorrectionPolicy};
use lay::keyboard::{map_events_to_layout, map_original_events, replay_layout_decision};
use lay::typing_assist::{
    effective_replace_words, should_force_replay_for_short_fragment, ScopedTailOptions,
};
use lay::word_buffer::WordBuffer;
use std::time::Instant;

use super::auto_undo_runtime::handle_pending_auto_undo;
use super::physical_input_grab::PhysicalInputGrab;
use super::{
    active_auto_replace, active_auto_switch_layout, active_correction_engine,
    active_lem_enabled_for_scope, log, log_manual_trigger_cross_check, release_possible_modifiers,
    settle_after_physical_trigger_release, switch_to_target_layout, ExecutingGuard,
};

#[path = "correction_runtime/memory.rs"]
mod memory;

#[path = "correction_runtime/output.rs"]
mod output;
use output::{apply_manual_correction_output, ManualCorrectionOutputContext};

pub(super) fn handle_force_layout_hotkey(
    target_is_ru: bool,
    buf: &mut WordBuffer,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
) -> Option<bool> {
    let started_at = Instant::now();
    settle_after_physical_trigger_release();
    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Some(kbd) = virtual_kbd {
        if let Err(e) = release_possible_modifiers(kbd) {
            log(&format!("⚠ force-layout modifier cleanup failed: {e}"));
        }
    }

    match switch_to_target_layout(target_is_ru) {
        Ok(layout_id) => {
            buf.reset_all();
            log(&format!(
                "✓ force-layout → {layout_id} за {}ms",
                started_at.elapsed().as_millis()
            ));
            Some(target_is_ru)
        }
        Err(e) => {
            log(&format!("⚠ force-layout switch failed: {e}"));
            None
        }
    }
}

pub(super) fn run_manual_correction_with_scope(
    buf: &mut WordBuffer,
    replace_words: usize,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
    events_since_word_start: u32,
    label: &str,
    input_isolated: bool,
    physical_grab: Option<&mut PhysicalInputGrab<'_>>,
) -> Option<bool> {
    log_manual_trigger_cross_check(buf, events_since_word_start);
    let engine = active_correction_engine();
    let auto_replace = active_auto_replace();
    let result = handle_double_shift(
        buf,
        replace_words,
        engine,
        auto_replace,
        virtual_kbd,
        executing,
        input_isolated,
        physical_grab,
    );
    log(&format!("· {label} fired with scope={replace_words}"));
    result
}

pub(super) fn handle_double_shift(
    buf: &mut WordBuffer,
    replace_words: usize,
    engine: CorrectionEngine,
    auto_replace: bool,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
    input_isolated: bool,
    physical_grab: Option<&mut PhysicalInputGrab<'_>>,
) -> Option<bool> {
    let started_at = Instant::now();
    if let Some(undo) = buf.take_pending_auto_undo() {
        return handle_pending_auto_undo(buf, undo, virtual_kbd, executing, started_at);
    }

    let replace_words = effective_replace_words(buf, replace_words, engine, auto_replace);
    let Some((events, n_backspaces)) = buf.what_to_replay(replace_words) else {
        log("👆 двойной Shift, но буфер пуст");
        return None;
    };
    *executing = true; // блокируем Shift events на время выполнения
    let _executing_guard = ExecutingGuard(executing);

    let layout_decision = replay_layout_decision(&events);
    let target_is_ru = layout_decision.target_is_ru;
    let mixed_layouts = layout_decision.mixed_layouts;

    let mapped_orig = map_original_events(&events);
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
    if buf.should_consume_auto_layout_replay_guard(&mapped_orig, &mapped_target) {
        log("· double Shift consumed: layout word was already fixed by auto-replace");
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
    };
    let correction_result = decide_manual_correction(
        ManualCorrectionInput {
            events: &events,
            original: &mapped_orig,
            converted: &mapped_target,
        },
        ManualCorrectionPolicy {
            engine,
            force_replay: force_replay_toggle,
            auto_replace,
            scoped_options,
        },
    );
    apply_manual_correction_output(ManualCorrectionOutputContext {
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
    })
}
