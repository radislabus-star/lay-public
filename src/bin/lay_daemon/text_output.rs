use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent, KeyCode};
use lay::keyboard::{preferred_layout_for_text, text_to_uinput_runs, KeyEvent, TextInputRun};
use lay::text_edit::TextReplacement;
use std::time::Duration;

use super::{log, switch_to_target_layout};

const KEY_PACE_MS: u64 = 1;
const BACKSPACE_DOWN_MS: u64 = 1;
const BACKSPACE_PACE_MS: u64 = 2;
const BACKSPACE_SETTLE_MS: u64 = 16;
const TEXT_REPLACE_KEY_PACE_MS: u64 = 1;
const TEXT_REPLACE_BACKSPACE_DOWN_MS: u64 = 1;
const TEXT_REPLACE_BACKSPACE_PACE_MS: u64 = 1;
const TEXT_REPLACE_BACKSPACE_SETTLE_MS: u64 = 1;
const TEXT_INSERT_KEY_PACE_MS: u64 = 1;
const TEXT_INSERT_SPACE_SETTLE_MS: u64 = 0;
const MODIFIER_RELEASE_ROUNDS: usize = 2;
const MODIFIER_RELEASE_PACE_MS: u64 = 3;
const MODIFIER_RELEASE_SETTLE_MS: u64 = 4;
const FAST_MODIFIER_RELEASE_PACE_MS: u64 = 0;
const FAST_MODIFIER_RELEASE_SETTLE_MS: u64 = 0;

#[derive(Debug, Clone)]
pub(super) struct PreparedTextInsert {
    runs: Vec<TextInputRun>,
    insert_layout_is_ru: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TextInsertOutcome {
    pub layout_is_ru: bool,
    pub layout_already_set: bool,
}

pub(super) fn replay_keycodes(dev: &mut VirtualDevice, events: &[KeyEvent]) -> std::io::Result<()> {
    replay_keycodes_with_pace(dev, events, KEY_PACE_MS, 0)
}

fn replay_text_insert_keycodes(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
) -> std::io::Result<()> {
    replay_keycodes_with_pace(
        dev,
        events,
        TEXT_INSERT_KEY_PACE_MS,
        TEXT_INSERT_SPACE_SETTLE_MS,
    )
}

fn replay_keycodes_with_pace(
    dev: &mut VirtualDevice,
    events: &[KeyEvent],
    key_pace_ms: u64,
    space_settle_ms: u64,
) -> std::io::Result<()> {
    let shift_l = KeyCode::KEY_LEFTSHIFT.code();

    // CRITICAL: при быстром двойном Shift physical Shift_L юзера может ещё
    // быть «зажат» в kernel/mutter modifier state (FSM сработал по release
    // event, но modifier применяется async). Если не сбросить — все
    // последующие keys получат CAPS от висящего modifier, дают «GHBDTn».
    // Принудительно emit Shift_L/Shift_R release ДО replay — overrides
    // любой stuck physical state.
    if key_pace_ms == 0 && space_settle_ms == 0 {
        release_possible_modifiers_fast(dev)?;
    } else {
        release_possible_modifiers(dev)?;
    }

    for ev in events {
        if ev.shift {
            dev.emit(&[
                InputEvent::new(EventType::KEY.0, shift_l, 1),
                InputEvent::new(EventType::KEY.0, ev.keycode, 1),
                InputEvent::new(EventType::KEY.0, ev.keycode, 0),
                InputEvent::new(EventType::KEY.0, shift_l, 0),
            ])?;
        } else {
            dev.emit(&[
                InputEvent::new(EventType::KEY.0, ev.keycode, 1),
                InputEvent::new(EventType::KEY.0, ev.keycode, 0),
            ])?;
        }
        let settle_ms = if ev.keycode == KeyCode::KEY_SPACE.code() && space_settle_ms > 0 {
            space_settle_ms
        } else {
            key_pace_ms
        };
        std::thread::sleep(Duration::from_millis(settle_ms));
    }
    Ok(())
}

pub(super) fn prepare_text_insert_for_replacement_plan(
    plan: &TextReplacement,
    fallback_layout_is_ru: bool,
) -> Result<PreparedTextInsert, String> {
    let insert_layout_is_ru = preferred_layout_for_text(&plan.insert, fallback_layout_is_ru);
    let runs = text_to_uinput_runs(&plan.insert, insert_layout_is_ru)
        .ok_or_else(|| "text insert requires unsafe TypeText fallback".to_string())?;
    for run in &runs {
        switch_to_target_layout(run.target_is_ru)
            .map_err(|e| format!("layout preflight failed before destructive edit: {e}"))?;
    }
    Ok(PreparedTextInsert {
        runs,
        insert_layout_is_ru,
    })
}

pub(super) fn apply_text_replacement(
    dev: &mut VirtualDevice,
    plan: &TextReplacement,
) -> std::io::Result<()> {
    emit_key_taps(
        dev,
        KeyCode::KEY_LEFT,
        plan.move_left,
        TEXT_REPLACE_KEY_PACE_MS,
    )?;
    emit_backspaces_for_text_replace(dev, plan.backspaces)?;
    Ok(())
}

pub(super) fn insert_prepared_text_for_replacement_plan(
    dev: &mut VirtualDevice,
    plan: &TextReplacement,
    replacement: &str,
    prepared: &PreparedTextInsert,
    label: &str,
) -> Result<TextInsertOutcome, String> {
    for run in &prepared.runs {
        switch_to_target_layout(run.target_is_ru)?;
        replay_text_insert_keycodes(dev, &run.events).map_err(|e| e.to_string())?;
    }
    if let Err(e) = emit_key_taps_fast(dev, KeyCode::KEY_RIGHT, plan.move_right) {
        return Err(format!("cursor restore failed: {e}"));
    }
    log(&format!("  {label} insert backend: prepared uinput replay"));
    Ok(TextInsertOutcome {
        layout_is_ru: layout_after_replacement_plan(
            plan,
            replacement,
            prepared.insert_layout_is_ru,
        ),
        layout_already_set: true,
    })
}

pub(super) fn layout_after_replacement_plan(
    plan: &TextReplacement,
    replacement: &str,
    insert_layout_is_ru: bool,
) -> bool {
    if plan.move_right == 0 {
        insert_layout_is_ru
    } else {
        preferred_layout_for_text(replacement, insert_layout_is_ru)
    }
}

pub(super) fn emit_key_taps_fast(
    dev: &mut VirtualDevice,
    key: KeyCode,
    n: u32,
) -> std::io::Result<()> {
    emit_key_taps(dev, key, n, 0)
}

fn emit_key_taps(
    dev: &mut VirtualDevice,
    key: KeyCode,
    n: u32,
    pace_ms: u64,
) -> std::io::Result<()> {
    let code = key.code();
    for _ in 0..n {
        dev.emit(&[
            InputEvent::new(EventType::KEY.0, code, 1),
            InputEvent::new(EventType::KEY.0, code, 0),
        ])?;
        if pace_ms > 0 {
            std::thread::sleep(Duration::from_millis(pace_ms));
        }
    }
    Ok(())
}

pub(super) fn release_possible_modifiers(dev: &mut VirtualDevice) -> std::io::Result<()> {
    release_possible_modifiers_with_pace(dev, MODIFIER_RELEASE_PACE_MS, MODIFIER_RELEASE_SETTLE_MS)
}

pub(super) fn release_possible_modifiers_fast(dev: &mut VirtualDevice) -> std::io::Result<()> {
    release_possible_modifiers_with_pace(
        dev,
        FAST_MODIFIER_RELEASE_PACE_MS,
        FAST_MODIFIER_RELEASE_SETTLE_MS,
    )
}

fn release_possible_modifiers_with_pace(
    dev: &mut VirtualDevice,
    pace_ms: u64,
    settle_ms: u64,
) -> std::io::Result<()> {
    let modifiers = [
        KeyCode::KEY_LEFTSHIFT.code(),
        KeyCode::KEY_RIGHTSHIFT.code(),
        KeyCode::KEY_LEFTCTRL.code(),
        KeyCode::KEY_RIGHTCTRL.code(),
        KeyCode::KEY_LEFTALT.code(),
        KeyCode::KEY_RIGHTALT.code(),
    ];
    let events: Vec<_> = modifiers
        .iter()
        .map(|code| InputEvent::new(EventType::KEY.0, *code, 0))
        .collect();

    for _ in 0..MODIFIER_RELEASE_ROUNDS {
        dev.emit(&events)?;
        if pace_ms > 0 {
            std::thread::sleep(Duration::from_millis(pace_ms));
        }
    }
    if settle_ms > 0 {
        std::thread::sleep(Duration::from_millis(settle_ms));
    }
    Ok(())
}

pub(super) fn emit_backspaces(dev: &mut VirtualDevice, n: u32) -> std::io::Result<()> {
    let bs = KeyCode::KEY_BACKSPACE.code();

    // Длинный batch может частично теряться в Mutter/GTK при сотнях клавиш.
    // Пейсинг делает удаление детерминированным для длинных слов.
    for _ in 0..n {
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 1)])?;
        std::thread::sleep(Duration::from_millis(BACKSPACE_DOWN_MS));
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 0)])?;
        std::thread::sleep(Duration::from_millis(BACKSPACE_PACE_MS));
    }
    std::thread::sleep(Duration::from_millis(BACKSPACE_SETTLE_MS));
    Ok(())
}

fn emit_backspaces_for_text_replace(dev: &mut VirtualDevice, n: u32) -> std::io::Result<()> {
    let bs = KeyCode::KEY_BACKSPACE.code();
    for _ in 0..n {
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 1)])?;
        std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_DOWN_MS));
        dev.emit(&[InputEvent::new(EventType::KEY.0, bs, 0)])?;
        std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_PACE_MS));
    }
    std::thread::sleep(Duration::from_millis(TEXT_REPLACE_BACKSPACE_SETTLE_MS));
    Ok(())
}

pub(super) fn make_virtual_keyboard() -> std::io::Result<VirtualDevice> {
    use KeyCode as K;
    // Перечисляем все клавиши которые виртуальное устройство сможет генерировать.
    let mut keys = AttributeSet::new();
    let typing = [
        K::KEY_A,
        K::KEY_B,
        K::KEY_C,
        K::KEY_D,
        K::KEY_E,
        K::KEY_F,
        K::KEY_G,
        K::KEY_H,
        K::KEY_I,
        K::KEY_J,
        K::KEY_K,
        K::KEY_L,
        K::KEY_M,
        K::KEY_N,
        K::KEY_O,
        K::KEY_P,
        K::KEY_Q,
        K::KEY_R,
        K::KEY_S,
        K::KEY_T,
        K::KEY_U,
        K::KEY_V,
        K::KEY_W,
        K::KEY_X,
        K::KEY_Y,
        K::KEY_Z,
        K::KEY_1,
        K::KEY_2,
        K::KEY_3,
        K::KEY_4,
        K::KEY_5,
        K::KEY_6,
        K::KEY_7,
        K::KEY_8,
        K::KEY_9,
        K::KEY_0,
        K::KEY_SPACE,
        K::KEY_SEMICOLON,
        K::KEY_APOSTROPHE,
        K::KEY_COMMA,
        K::KEY_DOT,
        K::KEY_LEFTBRACE,
        K::KEY_RIGHTBRACE,
        K::KEY_GRAVE,
        K::KEY_SLASH,
        K::KEY_BACKSLASH,
        K::KEY_MINUS,
        K::KEY_EQUAL,
        K::KEY_LEFTSHIFT,
        K::KEY_RIGHTSHIFT,
        K::KEY_LEFTALT,
        K::KEY_RIGHTALT,
        K::KEY_LEFTCTRL,
        K::KEY_RIGHTCTRL,
        K::KEY_INSERT,
        K::KEY_LEFT,
        K::KEY_RIGHT,
        K::KEY_BACKSPACE, // для удаления слова с экрана (cut этап)
    ];
    for k in typing.iter() {
        keys.insert(*k);
    }
    VirtualDevice::builder()?
        .name("lay-virtual-keyboard")
        .with_keys(&keys)?
        .build()
}
