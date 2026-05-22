use evdev::{uinput::VirtualDevice, Device, EventType, InputEvent, KeyCode};
use lay::config::TypingAssistRuleConfig;
use lay::decoder::{
    decode_enter_autocorrect_tail, decode_typing_assist_tail, CorrectionSource, DecoderEditPlan,
};
use lay::keyboard::{is_typing_key, map_original_events, preferred_layout_for_text, KeyEvent};
use lay::text_edit::{
    offset_replacement_plan_for_cursor, plan_committed_whitespace_insertions, TextReplacement,
};
use lay::word_buffer::WordBuffer;
use std::sync::atomic::Ordering;
use std::time::Instant;

#[cfg(not(test))]
use super::active_typing_assist_pipeline_for_auto_replace;
use super::{
    active_auto_switch_layout, apply_text_replacement, emit_key_taps_fast,
    insert_prepared_text_for_replacement_plan, log, prepare_text_insert_for_replacement_plan,
    read_current_layout_is_ru, record_recent_action, release_possible_modifiers,
    release_possible_modifiers_fast, should_try_ime_text_backend, switch_to_target_layout,
    try_ime_replace_tail, ExecutingGuard, TYPING_ASSIST_RUNTIME_READY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TypingAssistOutcome {
    Applied,
    NoCorrection,
    Deferred,
}

pub(super) fn handle_typing_assist_after_space(
    buf: &mut WordBuffer,
    mut virtual_kbd: Option<&mut VirtualDevice>,
    physical_device: Option<&mut Device>,
    executing: &mut bool,
    cursor_offset: u32,
) -> TypingAssistOutcome {
    if !TYPING_ASSIST_RUNTIME_READY.load(Ordering::Relaxed) {
        log("· typing-assist skipped: warmup pending");
        return TypingAssistOutcome::NoCorrection;
    }

    let started_at = Instant::now();
    let allow_layout_auto = active_auto_switch_layout();
    let correction = [2, 1].into_iter().find_map(|word_count| {
        let events = buf.last_completed_words_events(word_count)?;
        let context = typing_assist_context_for_completed_tail(buf, word_count, &events);
        #[cfg(test)]
        let pipeline = lay::typing_context::typing_assist_pipeline_for_context(
            true,
            lay::config::CorrectionSafety::Normal,
            &lay::config::default_typing_assist_pipeline(),
            &context,
        );
        #[cfg(not(test))]
        let pipeline = active_typing_assist_pipeline_for_auto_replace(&context);
        let edit = decode_typing_assist_tail(
            &events,
            allow_layout_auto,
            &pipeline,
            CorrectionSource::TypingAssist,
        )?;
        Some((events, edit))
    });
    let Some((events, edit)) = correction else {
        return TypingAssistOutcome::NoCorrection;
    };
    let mut physical_grab = PhysicalInputGrab::new(physical_device);
    let original = edit.original.clone();
    let replacement = edit.replacement.clone();
    let defer_complex_live_edit = cursor_offset == 0
        && !physical_grab.is_active()
        && should_defer_immediate_typing_edit(&edit);

    if cursor_offset == 0 && should_try_ime_text_backend() {
        let original_layout = read_current_layout_is_ru().ok();
        if try_ime_replace_tail(&original, &replacement, "typing-assist").unwrap_or(false) {
            let target_layout = preferred_layout_for_text(&replacement, true);
            switch_or_restore_layout_after_text_edit(
                target_layout,
                original_layout,
                "typing-assist",
                false,
            );
            if let Some(kbd) = virtual_kbd.as_deref_mut() {
                physical_grab.forward_queued_typing(kbd, buf, target_layout, "typing-assist");
            }
            let words = original.split_whitespace().count();
            remember_assisted_text_correction(
                buf,
                AssistedCorrectionMemory {
                    events: &events,
                    plan: &TextReplacement {
                        move_left: 0,
                        backspaces: original.chars().count() as u32,
                        insert: replacement.clone(),
                        move_right: 0,
                    },
                    original: &original,
                    replacement: &replacement,
                    kind: "typing-assist",
                    replace_words: words,
                    words,
                    cursor_offset: 0,
                },
            );
            record_recent_action(
                "typing-assist",
                &original,
                &replacement,
                words,
                words,
                started_at,
                true,
            );
            log(&format!(
                "✓ done: помощь при наборе {:?} → {:?} через IME за {}ms",
                original,
                replacement,
                started_at.elapsed().as_millis()
            ));
            return TypingAssistOutcome::Applied;
        }
        if defer_complex_live_edit {
            log("· typing-assist deferred: complex live edit needs safe boundary");
            return TypingAssistOutcome::Deferred;
        }
    }

    if defer_complex_live_edit {
        log("· typing-assist deferred: complex live edit needs safe boundary");
        return TypingAssistOutcome::Deferred;
    }

    let Some(kbd) = virtual_kbd else {
        log("⚠ typing-assist: нет uinput device");
        return TypingAssistOutcome::NoCorrection;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers_fast(kbd) {
        log(&format!("⚠ typing-assist modifier cleanup failed: {e}"));
    }

    let original_layout = read_current_layout_is_ru().ok();
    if let Some(space_plans) =
        plan_committed_whitespace_insertions(&original, &replacement, cursor_offset)
            .filter(|plans| plans.len() == 1)
    {
        log(&format!(
            "  typing-assist whitespace plans: count={}",
            space_plans.len()
        ));
        for plan in &space_plans {
            if let Err(e) = apply_text_replacement(kbd, plan) {
                log(&format!("⚠ typing-assist space insert move failed: {e}"));
                return TypingAssistOutcome::NoCorrection;
            }
            if let Err(e) = emit_key_taps_fast(kbd, KeyCode::KEY_SPACE, 1) {
                log(&format!("⚠ typing-assist space insert failed: {e}"));
                return TypingAssistOutcome::NoCorrection;
            }
            if let Err(e) = emit_key_taps_fast(kbd, KeyCode::KEY_RIGHT, plan.move_right) {
                log(&format!("⚠ typing-assist cursor restore failed: {e}"));
                return TypingAssistOutcome::NoCorrection;
            }
        }
        switch_or_restore_layout_after_text_edit(true, original_layout, "typing-assist", false);
        physical_grab.forward_queued_typing(kbd, buf, true, "typing-assist");
        remember_assisted_text_correction(
            buf,
            AssistedCorrectionMemory {
                events: &events,
                plan: &edit.plan,
                original: &original,
                replacement: &replacement,
                kind: "typing-assist",
                replace_words: original.split_whitespace().count().max(1),
                words: original.split_whitespace().count(),
                cursor_offset,
            },
        );
        record_recent_action(
            "typing-assist",
            &original,
            &replacement,
            original.split_whitespace().count().max(1),
            original.split_whitespace().count(),
            started_at,
            true,
        );
        log(&format!(
            "✓ done: помощь при наборе {:?} → {:?} через whitespace insertions за {}ms",
            original,
            replacement,
            started_at.elapsed().as_millis()
        ));
        return TypingAssistOutcome::Applied;
    }
    let plan = offset_replacement_plan_for_cursor(&edit.plan, cursor_offset);

    log(&format!(
        "  typing-assist plan: left={} bs={} insert={:?} right={}",
        plan.move_left, plan.backspaces, plan.insert, plan.move_right
    ));
    let prepared_insert = match prepare_text_insert_for_replacement_plan(&plan, true) {
        Ok(prepared) => prepared,
        Err(e) => {
            log(&format!("⚠ typing-assist skipped before delete: {e}"));
            return TypingAssistOutcome::NoCorrection;
        }
    };
    if let Err(e) = apply_text_replacement(kbd, &plan) {
        log(&format!("⚠ typing-assist minimal replace failed: {e}"));
        return TypingAssistOutcome::NoCorrection;
    }

    let insert_outcome = match insert_prepared_text_for_replacement_plan(
        kbd,
        &plan,
        &replacement,
        &prepared_insert,
        "typing-assist",
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            log(&format!("⚠ typing-assist {e}"));
            return TypingAssistOutcome::NoCorrection;
        }
    };
    switch_or_restore_layout_after_text_edit(
        insert_outcome.layout_is_ru,
        original_layout,
        "typing-assist",
        insert_outcome.layout_already_set,
    );
    physical_grab.forward_queued_typing(kbd, buf, insert_outcome.layout_is_ru, "typing-assist");

    let words = original.split_whitespace().count();
    remember_assisted_text_correction(
        buf,
        AssistedCorrectionMemory {
            events: &events,
            plan: &plan,
            original: &original,
            replacement: &replacement,
            kind: "typing-assist",
            replace_words: words,
            words,
            cursor_offset,
        },
    );
    record_recent_action(
        "typing-assist",
        &original,
        &replacement,
        words,
        words,
        started_at,
        true,
    );
    log(&format!(
        "✓ done: помощь при наборе {:?} → {:?} за {}ms",
        original,
        replacement,
        started_at.elapsed().as_millis()
    ));
    TypingAssistOutcome::Applied
}

fn should_defer_immediate_typing_edit(edit: &DecoderEditPlan) -> bool {
    edit.plan.move_right > 0
        && edit.plan.backspaces > 0
        && edit.plan.insert.chars().any(char::is_whitespace)
}

fn typing_assist_context_for_completed_tail(
    buf: &WordBuffer,
    word_count: usize,
    events: &[KeyEvent],
) -> String {
    if word_count == 1 {
        if let Some(context_events) = buf.last_completed_words_events(2) {
            return map_original_events(&context_events);
        }
    }
    map_original_events(events)
}

struct PhysicalInputGrab<'a> {
    device: Option<&'a mut Device>,
    active: bool,
}

impl<'a> PhysicalInputGrab<'a> {
    fn new(device: Option<&'a mut Device>) -> Self {
        let Some(device) = device else {
            return Self {
                device: None,
                active: false,
            };
        };

        match device.grab() {
            Ok(()) => Self {
                device: Some(device),
                active: true,
            },
            Err(e) => {
                log(&format!(
                    "⚠ physical device grab failed: {e}; continuing without input isolation"
                ));
                Self {
                    device: Some(device),
                    active: false,
                }
            }
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn forward_queued_typing(
        &mut self,
        virtual_kbd: &mut VirtualDevice,
        buf: &mut WordBuffer,
        layout_is_ru: bool,
        label: &str,
    ) {
        if !self.active {
            return;
        }

        let Some(device) = self.device.as_deref_mut() else {
            return;
        };

        let mut shift_active = false;
        let mut forwarded = 0usize;
        loop {
            let events = match device.fetch_events() {
                Ok(events) => events.collect::<Vec<_>>(),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    log(&format!("⚠ {label} passthrough read failed: {e}"));
                    break;
                }
            };
            if events.is_empty() {
                break;
            }

            for event in events {
                if event.event_type() != EventType::KEY {
                    continue;
                }
                let key = KeyCode::new(event.code());
                let value = event.value();

                match key {
                    KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT => {
                        shift_active = value != 0;
                        continue;
                    }
                    _ => {}
                }

                if value != 1 && value != 2 {
                    continue;
                }

                if key == KeyCode::KEY_SPACE {
                    if let Err(e) = emit_key_taps_fast(virtual_kbd, KeyCode::KEY_SPACE, 1) {
                        log(&format!("⚠ {label} passthrough space failed: {e}"));
                        continue;
                    }
                    buf.handle_space();
                    forwarded += 1;
                    continue;
                }

                if !is_typing_key(key) {
                    continue;
                }

                if let Err(e) = emit_forwarded_key_tap(virtual_kbd, key, shift_active) {
                    log(&format!("⚠ {label} passthrough key failed: {e}"));
                    continue;
                }
                buf.push(KeyEvent {
                    keycode: event.code(),
                    shift: shift_active,
                    layout_is_ru,
                });
                forwarded += 1;
            }
        }

        if forwarded > 0 {
            log(&format!(
                "· {label} passthrough forwarded {forwarded} queued keys"
            ));
        }
    }
}

impl Drop for PhysicalInputGrab<'_> {
    fn drop(&mut self) {
        if self.active {
            if let Some(device) = self.device.as_deref_mut() {
                if let Err(e) = device.ungrab() {
                    log(&format!("⚠ physical device ungrab failed: {e}"));
                }
            }
        }
    }
}

fn emit_forwarded_key_tap(
    dev: &mut VirtualDevice,
    key: KeyCode,
    shift: bool,
) -> std::io::Result<()> {
    if shift {
        dev.emit(&[
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_LEFTSHIFT.code(), 1),
            InputEvent::new(EventType::KEY.0, key.code(), 1),
            InputEvent::new(EventType::KEY.0, key.code(), 0),
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_LEFTSHIFT.code(), 0),
        ])
    } else {
        emit_key_taps_fast(dev, key, 1)
    }
}

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

pub(super) struct AssistedCorrectionMemory<'a> {
    events: &'a [KeyEvent],
    plan: &'a TextReplacement,
    original: &'a str,
    replacement: &'a str,
    kind: &'a str,
    replace_words: usize,
    words: usize,
    cursor_offset: u32,
}

fn remember_assisted_text_correction(
    buf: &mut WordBuffer,
    correction: AssistedCorrectionMemory<'_>,
) {
    buf.remember_pending_learning_correction(
        correction.kind,
        correction.original,
        correction.replacement,
        correction.replace_words,
        correction.words,
    );
    let remembered = if correction.cursor_offset > 0 {
        buf.remember_completed_replacement_words_for_replay(correction.replacement)
    } else {
        buf.remember_replacement_last_word_for_replay(
            correction.events,
            correction.plan,
            correction.replacement,
        )
    };
    if !remembered && correction.cursor_offset == 0 {
        buf.reset_all();
    }
    buf.remember_pending_auto_undo(
        correction.kind,
        correction.original,
        correction.replacement,
        correction.replace_words,
        correction.words,
    );
}

fn switch_or_restore_layout_after_text_edit(
    target_layout: bool,
    original_layout: Option<bool>,
    label: &str,
    target_layout_already_set: bool,
) {
    if active_auto_switch_layout() {
        if target_layout_already_set {
            log(&format!("  {label} layout already set by text insert"));
        } else {
            match switch_to_target_layout(target_layout) {
                Ok(layout_id) => log(&format!("  {label} layout → {layout_id}")),
                Err(e) => log(&format!("⚠ {label} layout switch failed: {e}")),
            }
        }
    } else if let Some(layout_is_ru) = original_layout {
        match switch_to_target_layout(layout_is_ru) {
            Ok(layout_id) => log(&format!("  {label} layout restored → {layout_id}")),
            Err(e) => log(&format!("⚠ {label} layout restore failed: {e}")),
        }
    }
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
    let plan = edit.plan.clone();

    let prepared_insert = match prepare_text_insert_for_replacement_plan(&plan, true) {
        Ok(prepared) => prepared,
        Err(e) => {
            log(&format!("⚠ enter-autocorrect skipped before delete: {e}"));
            return None;
        }
    };
    if let Err(e) = apply_text_replacement(kbd, &plan) {
        log(&format!("⚠ enter-autocorrect minimal replace failed: {e}"));
        return None;
    }

    let insert_outcome = match insert_prepared_text_for_replacement_plan(
        kbd,
        &plan,
        &replacement,
        &prepared_insert,
        "enter-autocorrect",
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            log(&format!("⚠ enter-autocorrect {e}"));
            return None;
        }
    };
    switch_or_restore_layout_after_text_edit(
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
