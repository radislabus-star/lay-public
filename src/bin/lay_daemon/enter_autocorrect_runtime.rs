use evdev::{uinput::VirtualDevice, KeyCode};
use lay::action_log::RecentActionGateTrace;
use lay::config::TypingAssistRuleConfig;
use lay::decoder::{decode_enter_autocorrect_tail, DecoderEditPlan};
use lay::keyboard::{map_original_events, KeyEvent};
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
    let edit = decode_enter_autocorrect_tail(
        &events,
        original_has_trailing_space,
        allow_layout_auto,
        pipeline,
    )?;
    let input_gate = edit.text_edit_input_gate_trace().cloned();
    Some((events, edit, input_gate))
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
    if input_gate.is_none() {
        log("· enter-autocorrect skipped: transition trace missing");
        return None;
    }
    let edit_action = edit.authorize_verified_replacement(
        "enter-autocorrect",
        original.as_str(),
        replacement.as_str(),
        plan.clone(),
    );
    lay::action_log::record_candidate_edit_action_before_apply(
        &edit_action,
        lay::action_log::MutationLogRoute::ENTER_AUTOCORRECT,
        input_gate.clone(),
    );
    if should_try_ime_text_backend() {
        let original_layout = read_current_layout_is_ru().ok();
        let ime_backend_action = lay::text_edit::authorize_backend_edit(
            lay::text_edit::TextEditBackend::Ime,
            edit_action.clone(),
        );
        let ime_reason = ime_backend_action.reason;
        let Some(ime_authorized) = ime_backend_action.into_authorized() else {
            log(&format!(
                "⚠ enter-autocorrect IME blocked before dispatch: {ime_reason}; secondary backend blocked"
            ));
            return None;
        };
        let dispatch = try_ime_replace_tail(ime_authorized, "enter-autocorrect");
        if dispatch.was_applied() {
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
        if !dispatch.permits_backend_reselection() {
            log(&format!(
                "⚠ enter-autocorrect IME dispatch ended without apply: {}; secondary backend blocked",
                dispatch.reason()
            ));
            return None;
        }
    }

    let Some(kbd) = virtual_kbd else {
        log("⚠ enter-autocorrect: нет uinput device");
        return None;
    };
    let daemon_backend_action = lay::text_edit::authorize_backend_edit(
        lay::text_edit::TextEditBackend::Daemon,
        edit_action,
    );
    let backend = daemon_backend_action.backend;
    let reason = daemon_backend_action.reason;
    let Some(daemon_authorized) = daemon_backend_action.into_authorized() else {
        log(&format!(
            "⚠ enter-autocorrect daemon output blocked by executor contract: reason={} backend={} original={:?} replacement={:?}",
            reason,
            backend.as_str(),
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
        daemon_authorized,
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
