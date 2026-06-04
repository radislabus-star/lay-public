use evdev::{uinput::VirtualDevice, KeyCode};
use lay::config::TypingAssistRuleConfig;
use lay::decoder::{decode_enter_autocorrect_tail, DecoderEditPlan};
use lay::keyboard::{map_original_events, preferred_layout_for_text, KeyEvent};
use lay::word_buffer::WordBuffer;
use std::sync::atomic::Ordering;
use std::time::Instant;

use super::correction_memory_runtime::{
    remember_assisted_text_correction, AssistedCorrectionMemory,
};

#[cfg(not(test))]
use super::active_typing_assist_pipeline_for_auto_replace;
use super::{
    active_auto_switch_layout, apply_text_replacement_pipeline, emit_key_taps_fast, log,
    read_current_layout_is_ru, record_recent_action, release_possible_modifiers,
    should_try_ime_text_backend, switch_or_restore_layout_after_text_edit, try_ime_replace_tail,
    ExecutingGuard, TYPING_ASSIST_RUNTIME_READY,
};

pub(super) fn enter_autocorrect_candidate(
    buf: &WordBuffer,
    replace_words: usize,
    allow_layout_auto: bool,
    pipeline: &[TypingAssistRuleConfig],
) -> Option<(Vec<KeyEvent>, DecoderEditPlan)> {
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
    Some((events, edit))
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
    let (events, edit) =
        enter_autocorrect_candidate(buf, replace_words, allow_layout_auto, &pipeline)?;
    let original = edit.original.clone();
    let replacement = edit.replacement.clone();

    if should_try_ime_text_backend() {
        let original_layout = read_current_layout_is_ru().ok();
        if try_ime_replace_tail(&original, &replacement, "enter-autocorrect").unwrap_or(false) {
            let target_layout = preferred_layout_for_text(&replacement, true);
            if let Some(kbd) = virtual_kbd {
                if let Err(e) = emit_key_taps_fast(kbd, KeyCode::KEY_ENTER, 1) {
                    log(&format!("⚠ enter-autocorrect Enter send failed: {e}"));
                }
            }
            switch_or_restore_layout_after_text_edit(
                active_auto_switch_layout(),
                target_layout,
                original_layout,
                "enter-autocorrect",
                false,
            );
            record_recent_action(
                "enter-autocorrect",
                &original,
                &replacement,
                replace_words,
                original.split_whitespace().count(),
                started_at,
                false,
            );
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

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ enter-autocorrect modifier cleanup failed: {e}"));
    }

    let original_layout = read_current_layout_is_ru().ok();
    let Some(plan) = edit.verified_plan_for_cursor(0) else {
        log("⚠ enter-autocorrect skipped before delete: edit plan invariant failed");
        return None;
    };

    let insert_outcome = match apply_text_replacement_pipeline(
        kbd,
        &plan,
        &replacement,
        true,
        "enter-autocorrect",
        false,
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            e.log("enter-autocorrect", "minimal replace failed");
            return None;
        }
    };
    switch_or_restore_layout_after_text_edit(
        active_auto_switch_layout(),
        insert_outcome.layout_is_ru,
        original_layout,
        "enter-autocorrect",
        insert_outcome.layout_already_set,
    );

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
            replace_words,
            words: original.split_whitespace().count(),
            cursor_offset: 0,
        },
    );
    record_recent_action(
        "enter-autocorrect",
        &original,
        &replacement,
        replace_words,
        original.split_whitespace().count(),
        started_at,
        true,
    );
    log(&format!(
        "✓ done: Enter autocorrect {:?} → {:?} за {}ms",
        original,
        replacement,
        started_at.elapsed().as_millis()
    ));
    Some(insert_outcome.layout_is_ru)
}
