use evdev::uinput::VirtualDevice;
use lay::config::CorrectionEngine;
use lay::decoder::DecoderAction;
use lay::desktop::LayoutBackend;
use lay::engine::{decide_manual_correction, ManualCorrectionInput, ManualCorrectionPolicy};
use lay::keyboard::{
    keycode_to_ru_char, keycode_to_us_char, preferred_layout_for_text, replay_layout_decision,
    KeyEvent,
};
use lay::text_edit::{plan_committed_tail_replacement, replacement_plan_matches, TextReplacement};
use lay::typing_assist::{
    effective_replace_words, should_force_replay_for_short_fragment, ScopedTailOptions,
};
use lay::word_buffer::{PendingAutoUndo, UserLearningCorrection, WordBuffer};
use std::time::Instant;

use super::{
    active_auto_replace, active_auto_switch_layout, active_correction_engine,
    active_layout_backend, active_lem_enabled_for_scope, append_learning_log,
    append_user_correction_learning_log, apply_text_replacement, call_replace_text,
    emit_backspaces, insert_prepared_text_for_replacement_plan, log,
    prepare_text_insert_for_replacement_plan, record_recent_action, release_possible_modifiers,
    replay_keycodes, settle_after_physical_trigger_release, should_try_ime_text_backend,
    switch_to_target_layout, target_layout, try_ime_replace_tail, ExecutingGuard,
    GNOME_NATIVE_REPLACE_EXPERIMENTAL,
};

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
) -> Option<bool> {
    let buf_count = buf.current_len() as u32;
    log(&format!(
        "═ CROSS-CHECK: buffer.current={} events_since_word_start={}{}",
        buf_count,
        events_since_word_start,
        if buf_count != events_since_word_start {
            " ⚠ MISMATCH"
        } else {
            " ✓"
        }
    ));
    let engine = active_correction_engine();
    let auto_replace = active_auto_replace();
    let result = handle_double_shift(
        buf,
        replace_words,
        engine,
        auto_replace,
        virtual_kbd,
        executing,
    );
    log(&format!("· {label} fired with scope={replace_words}"));
    result
}

pub(super) struct ManualTextCorrectionMemory<'a> {
    pub(super) events: &'a [KeyEvent],
    pub(super) plan: &'a TextReplacement,
    pub(super) original: &'a str,
    pub(super) replacement: &'a str,
    pub(super) kind: &'a str,
    pub(super) replace_words: usize,
    pub(super) words: usize,
    pub(super) inserted_layout_is_ru: Option<bool>,
}

pub(super) fn remember_manual_text_correction(
    buf: &mut WordBuffer,
    correction: ManualTextCorrectionMemory<'_>,
) {
    buf.remember_pending_learning_correction(
        correction.kind,
        correction.original,
        correction.replacement,
        correction.replace_words,
        correction.words,
    );
    let remembered = buf.remember_replacement_last_word_for_replay(
        correction.events,
        correction.plan,
        correction.replacement,
    ) || correction
        .inserted_layout_is_ru
        .is_some_and(|layout_is_ru| {
            buf.remember_inserted_tail_for_replay(correction.events, correction.plan, layout_is_ru)
        })
        || (correction.inserted_layout_is_ru.is_some()
            && buf.remember_inserted_last_word_for_replay(correction.events, correction.plan));
    if !remembered {
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

pub(super) struct LayoutReplayMemory<'a> {
    replace_words: usize,
    target_is_ru: bool,
    force_replay_toggle: bool,
    original: &'a str,
    replacement: &'a str,
    words: usize,
    elapsed_ms: u128,
}

fn remember_layout_replay_success(buf: &mut WordBuffer, replay: LayoutReplayMemory<'_>) {
    buf.mark_replayed_layout(replay.replace_words, replay.target_is_ru);
    if !replay.force_replay_toggle && replay.original != replay.replacement {
        append_learning_log(
            "layout-replay",
            replay.original,
            replay.replacement,
            replay.replace_words,
            replay.words,
        );
    }
    lay::action_log::record_action(
        "layout-replay",
        replay.original,
        replay.replacement,
        replay.replace_words,
        replay.words,
        replay.elapsed_ms,
        true,
    );
}

pub(super) fn handle_double_shift(
    buf: &mut WordBuffer,
    replace_words: usize,
    engine: CorrectionEngine,
    auto_replace: bool,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
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

    // 3-й счёт: попытаться смаппить каждый keycode → char в ОБЕ раскладки.
    // Если char_count != events.len() — какой-то keycode вне таблиц
    // keycode_to_*_char (значит backspace×N сотрёт лишнее ИЛИ замаппится не всё).
    let mapped_orig: String = events
        .iter()
        .filter_map(|ev| {
            if ev.layout_is_ru {
                keycode_to_ru_char(ev.keycode, ev.shift)
            } else {
                keycode_to_us_char(ev.keycode, ev.shift)
            }
        })
        .collect();
    let mapped_target: String = events
        .iter()
        .filter_map(|ev| {
            if target_is_ru {
                keycode_to_ru_char(ev.keycode, ev.shift)
            } else {
                keycode_to_us_char(ev.keycode, ev.shift)
            }
        })
        .collect();
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
    let correction_action = correction_result.action.clone();
    let correction_edit = correction_result.edit.clone();

    if should_try_ime_text_backend() {
        let (replace_text, replace_kind, is_replay) = match &correction_action {
            DecoderAction::ReplaceText {
                replacement,
                source,
            } if !replacement.trim().is_empty() => (replacement.clone(), source.log_kind(), false),
            _ => (mapped_target.clone(), "ime-replay", true),
        };
        let replace_target_is_ru = preferred_layout_for_text(&replace_text, target_is_ru);
        if try_ime_replace_tail(&mapped_orig, &replace_text, replace_kind).unwrap_or(false) {
            if is_replay {
                remember_layout_replay_success(
                    buf,
                    LayoutReplayMemory {
                        replace_words,
                        target_is_ru: replace_target_is_ru,
                        force_replay_toggle,
                        original: &mapped_orig,
                        replacement: &replace_text,
                        words: words_orig,
                        elapsed_ms: started_at.elapsed().as_millis(),
                    },
                );
            } else {
                let plan = TextReplacement {
                    move_left: 0,
                    backspaces: mapped_orig.chars().count() as u32,
                    insert: replace_text.clone(),
                    move_right: 0,
                };
                remember_manual_text_correction(
                    buf,
                    ManualTextCorrectionMemory {
                        events: &events,
                        plan: &plan,
                        original: &mapped_orig,
                        replacement: &replace_text,
                        kind: replace_kind,
                        replace_words,
                        words: words_orig,
                        inserted_layout_is_ru: None,
                    },
                );
                record_recent_action(
                    replace_kind,
                    &mapped_orig,
                    &replace_text,
                    replace_words,
                    words_orig,
                    started_at,
                    true,
                );
            }
            return match switch_to_target_layout(replace_target_is_ru) {
                Ok(layout_id) => {
                    log(&format!("  layout → {layout_id}"));
                    log(&format!(
                        "✓ done: {replace_kind}, IME replace-tail за {}ms",
                        started_at.elapsed().as_millis()
                    ));
                    Some(replace_target_is_ru)
                }
                Err(e) => {
                    log(&format!(
                        "⚠ {replace_kind} IME text committed, layout switch failed: {e}"
                    ));
                    None
                }
            };
        }
    }

    if GNOME_NATIVE_REPLACE_EXPERIMENTAL && active_layout_backend() == LayoutBackend::Gnome {
        let (replace_text, replace_kind, is_replay) = match &correction_action {
            DecoderAction::ReplaceText {
                replacement,
                source,
            } if !replacement.trim().is_empty() => (replacement.clone(), source.log_kind(), false),
            _ => (mapped_target.clone(), "gnome-replace", true),
        };
        let replace_target_is_ru = preferred_layout_for_text(&replace_text, target_is_ru);
        let (layout_id, _) = target_layout(replace_target_is_ru);
        match call_replace_text(0, n_backspaces, &replace_text, 0, layout_id) {
            Ok(true) => {
                if is_replay {
                    remember_layout_replay_success(
                        buf,
                        LayoutReplayMemory {
                            replace_words,
                            target_is_ru: replace_target_is_ru,
                            force_replay_toggle,
                            original: &mapped_orig,
                            replacement: &replace_text,
                            words: words_orig,
                            elapsed_ms: started_at.elapsed().as_millis(),
                        },
                    );
                } else {
                    let plan = TextReplacement {
                        move_left: 0,
                        backspaces: n_backspaces,
                        insert: replace_text.clone(),
                        move_right: 0,
                    };
                    remember_manual_text_correction(
                        buf,
                        ManualTextCorrectionMemory {
                            events: &events,
                            plan: &plan,
                            original: &mapped_orig,
                            replacement: &replace_text,
                            kind: replace_kind,
                            replace_words,
                            words: words_orig,
                            inserted_layout_is_ru: None,
                        },
                    );
                    record_recent_action(
                        replace_kind,
                        &mapped_orig,
                        &replace_text,
                        replace_words,
                        words_orig,
                        started_at,
                        true,
                    );
                }
                log(&format!(
                    "  1. GNOME ReplaceText: bs={} insert={:?}",
                    n_backspaces, replace_text
                ));
                log(&format!("  2. layout → {layout_id}"));
                log(&format!(
                    "✓ done: {replace_kind}, GNOME-native replace за {}ms",
                    started_at.elapsed().as_millis()
                ));
                return Some(replace_target_is_ru);
            }
            Ok(false) => log("⚠ GNOME ReplaceText returned false; fallback to uinput replay"),
            Err(e) => log(&format!(
                "⚠ GNOME ReplaceText failed: {e}; fallback to uinput replay"
            )),
        }
    }

    let kbd = match virtual_kbd {
        Some(k) => k,
        None => {
            log("⚠ нет uinput device");
            return None;
        }
    };
    settle_after_physical_trigger_release();
    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ modifier cleanup before backspace failed: {e}"));
    }

    if let DecoderAction::ReplaceText {
        replacement: text,
        source,
    } = correction_action
    {
        let kind = source.log_kind();
        if text.trim().is_empty() || text == mapped_target {
            log("  2. text decision совпал с replay — replay для сохранения toggle");
        } else {
            let mut plan = correction_edit
                .as_ref()
                .map(|edit| edit.plan.clone())
                .or_else(|| plan_committed_tail_replacement(&mapped_orig, &text))
                .unwrap_or_else(|| TextReplacement {
                    move_left: 0,
                    backspaces: n_backspaces,
                    insert: text.clone(),
                    move_right: 0,
                });
            if correction_edit
                .as_ref()
                .is_some_and(|edit| !edit.plan_matches_replacement())
                || !replacement_plan_matches(&mapped_orig, &text, &plan)
            {
                log(&format!(
                    "⚠ {kind} plan invariant failed; using full tail replace"
                ));
                plan = TextReplacement {
                    move_left: 0,
                    backspaces: n_backspaces,
                    insert: text.clone(),
                    move_right: 0,
                };
            }
            let prepared_insert =
                match prepare_text_insert_for_replacement_plan(&plan, target_is_ru) {
                    Ok(prepared) => prepared,
                    Err(e) => {
                        log(&format!("⚠ {kind} skipped before delete: {e}"));
                        return None;
                    }
                };
            if let Err(e) = apply_text_replacement(kbd, &plan) {
                log(&format!("⚠ {kind} minimal replace failed: {e}"));
                return None;
            } else {
                let insert_outcome = match insert_prepared_text_for_replacement_plan(
                    kbd,
                    &plan,
                    &text,
                    &prepared_insert,
                    kind,
                ) {
                    Ok(outcome) => outcome,
                    Err(e) => {
                        log(&format!("⚠ {kind} {e}"));
                        return None;
                    }
                };
                let insert_target_is_ru = insert_outcome.layout_is_ru;
                let layout_result = if insert_outcome.layout_already_set {
                    Ok("already-set")
                } else {
                    switch_to_target_layout(insert_target_is_ru)
                };
                remember_manual_text_correction(
                    buf,
                    ManualTextCorrectionMemory {
                        events: &events,
                        plan: &plan,
                        original: &mapped_orig,
                        replacement: &text,
                        kind,
                        replace_words,
                        words: words_orig,
                        inserted_layout_is_ru: Some(preferred_layout_for_text(
                            &plan.insert,
                            insert_target_is_ru,
                        )),
                    },
                );
                record_recent_action(
                    kind,
                    &mapped_orig,
                    &text,
                    replace_words,
                    words_orig,
                    started_at,
                    true,
                );
                log(&format!(
                    "  1. minimal replace: left={} bs={} insert={:?} right={}",
                    plan.move_left, plan.backspaces, plan.insert, plan.move_right
                ));
                return match layout_result {
                    Ok(layout_id) => {
                        log(&format!("  2. layout → {layout_id}"));
                        log(&format!(
                            "✓ done: {kind}, исправлен BAD-диапазон за {}ms",
                            started_at.elapsed().as_millis()
                        ));
                        Some(insert_target_is_ru)
                    }
                    Err(e) => {
                        log(&format!(
                            "⚠ {kind} layout switch after text insert failed: {e}"
                        ));
                        log(&format!(
                            "✓ done: {kind}, текст исправлен, layout не подтверждён за {}ms",
                            started_at.elapsed().as_millis()
                        ));
                        None
                    }
                };
            }
        }
    }

    // Сначала подтверждаем раскладку. Это важно: если GNOME/IBus/окно не
    // принимает switch, нельзя стирать видимый текст и надеяться на fallback.
    let layout_id = match switch_to_target_layout(target_is_ru) {
        Ok(layout_id) => layout_id,
        Err(e) => {
            log(&format!(
                "⚠ Этап 1 layout switch failed before destructive replay: {e}"
            ));
            log("  replay aborted: исходное слово оставлено на месте");
            return None;
        }
    };

    // ЭТАП 2: backspace через uinput (надёжно)
    if let Err(e) = emit_backspaces(kbd, n_backspaces) {
        log(&format!("⚠ Этап 2 backspaces failed: {e}"));
        return None;
    }
    log(&format!("  1. layout → {layout_id}"));
    log(&format!("  2. uinput Backspace × {n_backspaces}"));
    let (_, ibus_engine) = target_layout(target_is_ru);

    // ЭТАП 3: replay тех же keycodes — в новой раскладке дают другие символы.
    if let Err(e) = replay_keycodes(kbd, &events) {
        log(&format!("⚠ Этап 3 replay failed: {e}"));
        return Some(target_is_ru);
    }
    remember_layout_replay_success(
        buf,
        LayoutReplayMemory {
            replace_words,
            target_is_ru,
            force_replay_toggle,
            original: &mapped_orig,
            replacement: &mapped_target,
            words: words_orig,
            elapsed_ms: started_at.elapsed().as_millis(),
        },
    );
    log(&format!("  3. uinput replay × {}", events.len()));

    log(&format!(
        "✓ done: раскладка {ibus_engine}, перенабрано {} клавиш за {}ms",
        events.len(),
        started_at.elapsed().as_millis()
    ));
    Some(target_is_ru)
}

fn handle_pending_auto_undo(
    buf: &mut WordBuffer,
    undo: PendingAutoUndo,
    virtual_kbd: Option<&mut VirtualDevice>,
    executing: &mut bool,
    started_at: Instant,
) -> Option<bool> {
    let Some(kbd) = virtual_kbd else {
        log("⚠ auto-undo: нет uinput device");
        return None;
    };

    *executing = true;
    let _executing_guard = ExecutingGuard(executing);

    if let Err(e) = release_possible_modifiers(kbd) {
        log(&format!("⚠ auto-undo modifier cleanup failed: {e}"));
    }

    let plan = pending_auto_undo_plan(&undo);
    if !replacement_plan_matches(&undo.replacement, &undo.original, &plan) {
        log("⚠ auto-undo skipped before delete: edit plan invariant failed");
        return None;
    }
    let prepared_insert = match prepare_text_insert_for_replacement_plan(&plan, true) {
        Ok(prepared) => prepared,
        Err(e) => {
            log(&format!("⚠ auto-undo skipped before delete: {e}"));
            return None;
        }
    };
    if let Err(e) = apply_text_replacement(kbd, &plan) {
        log(&format!("⚠ auto-undo delete failed: {e}"));
        return None;
    }

    let insert_outcome = match insert_prepared_text_for_replacement_plan(
        kbd,
        &plan,
        &undo.original,
        &prepared_insert,
        "auto-undo",
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            log(&format!("⚠ auto-undo {e}"));
            return None;
        }
    };
    if insert_outcome.layout_already_set {
        log("  auto-undo layout already set by text insert");
    } else {
        match switch_to_target_layout(insert_outcome.layout_is_ru) {
            Ok(layout_id) => log(&format!("  auto-undo layout → {layout_id}")),
            Err(e) => log(&format!("⚠ auto-undo layout switch failed: {e}")),
        }
    }

    append_user_correction_learning_log(&UserLearningCorrection {
        lay_kind: undo.lay_kind.clone(),
        lay_from: undo.original.clone(),
        lay_to: undo.replacement.clone(),
        from: undo.replacement.clone(),
        to: undo.original.clone(),
        replace_words: undo.replace_words,
        words: undo.words,
    });
    record_recent_action(
        "auto-undo",
        &undo.replacement,
        &undo.original,
        undo.replace_words,
        undo.words,
        started_at,
        false,
    );
    buf.clear_pending_learning();
    if !buf.remember_visible_text_for_correction(&undo.original) {
        buf.reset_all();
    }
    log(&format!(
        "✓ done: auto-undo {:?} → {:?} за {}ms",
        undo.replacement,
        undo.original,
        started_at.elapsed().as_millis()
    ));
    Some(insert_outcome.layout_is_ru)
}

pub(super) fn pending_auto_undo_plan(undo: &PendingAutoUndo) -> TextReplacement {
    TextReplacement {
        move_left: 0,
        backspaces: undo.replacement.chars().count() as u32,
        insert: undo.original.clone(),
        move_right: 0,
    }
}
