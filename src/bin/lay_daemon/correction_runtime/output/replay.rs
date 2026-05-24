use evdev::uinput::VirtualDevice;

use super::super::super::{
    emit_backspaces, log, replay_keycodes, switch_to_target_layout, target_layout,
};
use super::super::memory::{remember_layout_replay_success, LayoutReplayMemory};
use super::context::ManualOutputCommon;

pub(crate) fn apply_layout_replay(
    ctx: &mut ManualOutputCommon<'_>,
    kbd: &mut VirtualDevice,
) -> Option<bool> {
    let layout_id = match switch_to_target_layout(ctx.target_is_ru) {
        Ok(layout_id) => layout_id,
        Err(e) => {
            log(&format!(
                "⚠ Этап 1 layout switch failed before destructive replay: {e}"
            ));
            log("  replay aborted: исходное слово оставлено на месте");
            return None;
        }
    };

    if let Err(e) = emit_backspaces(kbd, ctx.n_backspaces) {
        log(&format!("⚠ Этап 2 backspaces failed: {e}"));
        return None;
    }
    log(&format!("  1. layout → {layout_id}"));
    log(&format!("  2. uinput Backspace × {}", ctx.n_backspaces));
    let (_, ibus_engine) = target_layout(ctx.target_is_ru);

    if let Err(e) = replay_keycodes(kbd, ctx.events) {
        log(&format!("⚠ Этап 3 replay failed: {e}"));
        return Some(ctx.target_is_ru);
    }
    remember_layout_replay_success(
        ctx.buf,
        LayoutReplayMemory {
            replace_words: ctx.replace_words,
            target_is_ru: ctx.target_is_ru,
            force_replay_toggle: ctx.force_replay_toggle,
            original: ctx.mapped_orig,
            replacement: ctx.mapped_target,
            words: ctx.words_orig,
            elapsed_ms: ctx.started_at.elapsed().as_millis(),
        },
    );
    log(&format!("  3. uinput replay × {}", ctx.events.len()));

    log(&format!(
        "✓ done: раскладка {ibus_engine}, перенабрано {} клавиш за {}ms",
        ctx.events.len(),
        ctx.started_at.elapsed().as_millis()
    ));
    Some(ctx.target_is_ru)
}
