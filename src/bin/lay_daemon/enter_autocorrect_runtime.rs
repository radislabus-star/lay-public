use evdev::{uinput::VirtualDevice, KeyCode};
use lay::action_log::RecentActionGateTrace;
use lay::config::{CorrectionSafety, TypingAssistRuleConfig};
use lay::correction_core::CorrectionMode;
use lay::decoder::{decode_enter_autocorrect_tail, DecoderEditPlan};
use lay::input_gate::{decide_input_gate, InputGateRequest, InputGateTrigger};
use lay::keyboard::{map_original_events, KeyEvent};
use lay::text_edit::TransitionAudit;
use lay::word_buffer::WordBuffer;
use std::sync::atomic::Ordering;
use std::time::Instant;

use super::action_log_runtime::RecentActionRecord;
use super::correction_memory_runtime::{
    remember_assisted_text_correction, AssistedCorrectionMemory,
};

#[cfg(not(test))]
use super::active_typing_assist_pipeline_for_auto_replace;
use super::{
    active_auto_switch_layout, apply_text_replacement_pipeline, emit_key_taps_fast,
    focused_ime_engine_handles_typing, layout_switch_policy, log, read_current_layout_is_ru,
    record_recent_action, release_possible_modifiers, should_try_ime_text_backend,
    switch_or_restore_layout_after_text_edit, try_ime_replace_tail, ExecutingGuard,
    TYPING_ASSIST_RUNTIME_READY,
};

pub(super) fn enter_autocorrect_candidate(
    buf: &WordBuffer,
    replace_words: usize,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<(
    Vec<KeyEvent>,
    DecoderEditPlan,
    Option<RecentActionGateTrace>,
)> {
    let replace_words = replace_words.min(1);
    let (events, _) = buf.what_to_replay(replace_words)?;
    let original = map_original_events(&events);
    if original.trim().is_empty() {
        return None;
    }

    let original_has_trailing_space = original
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace);
    let assist_input = if original_has_trailing_space {
        original.clone()
    } else {
        format!("{original} ")
    };
    let edit = decode_enter_autocorrect_tail(
        &events,
        original_has_trailing_space,
        allow_layout_auto,
        pipeline,
    )?;
    Some((
        events,
        edit,
        enter_input_gate_trace(&assist_input, pipeline),
    ))
}

pub(super) fn handle_enter_autocorrect(
    buf: &mut WordBuffer,
    replace_words: usize,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
) -> Option<bool> {
    if !TYPING_ASSIST_RUNTIME_READY.load(Ordering::Relaxed) {
        log("· enter-autocorrect skipped: warmup pending");
        return None;
    }
    if focused_ime_engine_handles_typing() {
        log("· enter-autocorrect skipped: focused IME engine owns boundary text");
        return None;
    }

    let started_at = Instant::now();
    let allow_layout_auto = active_auto_switch_layout();
    let context = buf
        .what_to_replay(replace_words)
        .map(|(events, _)| map_original_events(&events))
        .unwrap_or_default();
    #[cfg(test)]
    let pipeline = lay::typing_context::typing_assist_pipeline_for_context(
        true,
        lay::config::CorrectionSafety::Normal,
        &lay::config::default_typing_assist_pipeline(),
        &context,
    );
    #[cfg(not(test))]
    let pipeline = active_typing_assist_pipeline_for_auto_replace(&context);
    let (events, edit, input_gate) =
        enter_autocorrect_candidate(buf, replace_words, allow_layout_auto, &pipeline)?;
    let original = edit.original.clone();
    let replacement = edit.replacement.clone();
    let Some(plan) = edit.verified_plan_for_cursor(0) else {
        log("⚠ enter-autocorrect skipped before delete: edit plan invariant failed");
        return None;
    };
    let source_id = input_gate
        .as_ref()
        .and_then(|trace| trace.selected_source_id.as_deref());
    let error_class = input_gate
        .as_ref()
        .and_then(|trace| trace.selected_error_class.as_deref());
    let confidence_milli = input_gate
        .as_ref()
        .and_then(|trace| trace.scoreboard.as_ref())
        .and_then(|scoreboard| scoreboard.selected_bayes_posterior_milli)
        .unwrap_or(0);
    let transition = input_gate
        .as_ref()
        .map(RecentActionGateTrace::selected_transition_audit)
        .unwrap_or_else(|| {
            TransitionAudit::proven(
                "enter_autocorrect",
                "enter_boundary_plan_verified",
                true,
                false,
                original.split_whitespace().count().max(1),
            )
        });
    let edit_action = lay::text_edit::authorize_replacement_with_transition(
        "enter-autocorrect",
        confidence_milli,
        original.as_str(),
        replacement.as_str(),
        plan.clone(),
        source_id,
        error_class,
        transition,
    );
    lay::action_log::record_candidate_edit_action_before_apply(
        &edit_action,
        lay::action_log::MutationLogRoute::ENTER_AUTOCORRECT,
        input_gate.clone(),
    );
    let ime_backend_action =
        lay::text_edit::authorize_backend_edit(lay::text_edit::TextEditBackend::Ime, &edit_action);
    let daemon_backend_action = lay::text_edit::authorize_backend_edit(
        lay::text_edit::TextEditBackend::Daemon,
        &edit_action,
    );
    let ime_authorized = ime_backend_action.authorized();
    let daemon_authorized = daemon_backend_action.authorized();
    if ime_authorized.is_none() && daemon_authorized.is_none() {
        log(&format!(
            "⚠ enter-autocorrect blocked by executor contract: reason={} original={:?} replacement={:?}",
            daemon_backend_action.reason,
            original,
            replacement
        ));
        return None;
    }

    if should_try_ime_text_backend() {
        let original_layout = read_current_layout_is_ru().ok();
        if ime_authorized
            .as_ref()
            .is_some_and(|authorized| try_ime_replace_tail(authorized, "enter-autocorrect").unwrap_or(false))
        {
            let target_layout =
                layout_switch_policy::target_layout_for_replacement(&replacement, true);
            let force_target_layout =
                layout_switch_policy::force_target_layout_for_replacement(&original, &replacement);
            if let Some(kbd) = virtual_kbd {
                if let Err(e) = emit_key_taps_fast(kbd, KeyCode::KEY_ENTER, 1) {
                    log(&format!("⚠ enter-autocorrect Enter send failed: {e}"));
                }
            }
            if force_target_layout {
                switch_or_restore_layout_after_text_edit(
                    true,
                    target_layout,
                    original_layout,
                    "enter-autocorrect",
                    false,
                );
            }
            record_recent_action(RecentActionRecord {
                kind: "enter-autocorrect",
                from: &original,
                to: &replacement,
                replace_words,
                words: original.split_whitespace().count(),
                started_at,
                input_gate: input_gate.clone(),
                undo_available: false,
            });
            log(&format!(
                "✓ done: Enter autocorrect {:?} → {:?} через IME за {}ms",
                original,
                replacement,
                started_at.elapsed().as_millis()
            ));
            return Some(target_layout);
        }
    }

    let Some(kbd) = virtual_kbd else {
        log("⚠ enter-autocorrect: нет uinput device");
        return None;
    };
    let Some(daemon_authorized) = daemon_authorized else {
        log(&format!(
            "⚠ enter-autocorrect daemon output blocked by executor contract: reason={} backend={} original={:?} replacement={:?}",
            daemon_backend_action.reason,
            daemon_backend_action.backend.as_str(),
            original,
            replacement
        ));
        return None;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ enter-autocorrect modifier cleanup failed: {e}"));
    }

    let original_layout = read_current_layout_is_ru().ok();

    let insert_outcome = match apply_text_replacement_pipeline(
        kbd,
        &daemon_authorized,
        original_layout.unwrap_or(true),
        original_layout,
        "enter-autocorrect",
        false,
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            e.log("enter-autocorrect", "minimal replace failed");
            return None;
        }
    };
    let force_target_layout =
        layout_switch_policy::force_target_layout_for_replacement(&original, &replacement);
    if force_target_layout {
        switch_or_restore_layout_after_text_edit(
            true,
            insert_outcome.layout_is_ru,
            original_layout,
            "enter-autocorrect",
            insert_outcome.layout_already_set,
        );
    }

    if let Err(e) = emit_key_taps_fast(kbd, KeyCode::KEY_ENTER, 1) {
        log(&format!("⚠ enter-autocorrect Enter send failed: {e}"));
        return None;
    }

    remember_assisted_text_correction(
        buf,
        AssistedCorrectionMemory {
            events: &events,
            plan: &plan,
            original: &original,
            replacement: &replacement,
            kind: "enter-autocorrect",
            rule_id: None,
            replace_words,
            words: original.split_whitespace().count(),
            cursor_offset: 0,
        },
    );
    record_recent_action(RecentActionRecord {
        kind: "enter-autocorrect",
        from: &original,
        to: &replacement,
        replace_words,
        words: original.split_whitespace().count(),
        started_at,
        input_gate,
        undo_available: true,
    });
    log(&format!(
        "✓ done: Enter autocorrect {:?} → {:?} за {}ms",
        original,
        replacement,
        started_at.elapsed().as_millis()
    ));
    Some(insert_outcome.layout_is_ru)
}

fn enter_input_gate_trace(
    text_tail: &str,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<RecentActionGateTrace> {
    let decision = decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::Enter,
        text_tail,
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: false,
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: pipeline,
        nanda_autocorrect: false,
        nanda_wave_options: lay::nanda_wave::WaveOptions::default(),
        correction_mode: CorrectionMode::DeterministicOnly,
    });
    decision
        .trace
        .as_ref()
        .map(RecentActionGateTrace::from_input_gate)
}
