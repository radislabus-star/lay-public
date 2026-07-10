use evdev::uinput::VirtualDevice;
use lay::action_log::RecentActionGateTrace;
use lay::text_edit::{AuthorizedEdit, TextReplacement, TransitionAudit};
use std::time::Instant;

use super::super::super::{
    emit_backspaces, emit_backspaces_fast, log, replay_keycodes,
    replay_keycodes_fast_after_modifier_cleanup, suppress_next_ime_autocorrect,
    switch_to_target_layout, target_layout,
};
use super::super::memory::{remember_layout_replay_success, LayoutReplayMemory};
use super::context::ManualOutputCommon;

pub(crate) fn apply_layout_replay(
    ctx: &mut ManualOutputCommon<'_>,
    kbd: &mut VirtualDevice,
    input_gate: Option<RecentActionGateTrace>,
) -> Option<bool> {
    let authorized_edit = manual_replay_action(ctx, input_gate)?;
    let action = authorized_edit.action();
    let Some(plan) = action.plan.as_ref() else {
        log("⚠ manual replay blocked: AuthorizedEdit has no replacement plan");
        return None;
    };
    if plan.backspaces != ctx.n_backspaces || action.to_text != ctx.mapped_target {
        log("⚠ manual replay blocked: AuthorizedEdit does not match replay state");
        return None;
    }
    let layout_started = Instant::now();
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
    let layout_ms = layout_started.elapsed().as_millis();

    let backspace_started = Instant::now();
    let backspace_result = if ctx.input_isolated {
        emit_backspaces_fast(kbd, ctx.n_backspaces)
    } else {
        emit_backspaces(kbd, ctx.n_backspaces)
    };
    if let Err(e) = backspace_result {
        log(&format!("⚠ Этап 2 backspaces failed: {e}"));
        return None;
    }
    let backspace_ms = backspace_started.elapsed().as_millis();
    log(&format!("  1. layout → {layout_id}"));
    log(&format!("  2. uinput Backspace × {}", ctx.n_backspaces));
    let (_, ibus_engine) = target_layout(ctx.target_is_ru);

    let replay_started = Instant::now();
    let replay_result = if ctx.input_isolated {
        replay_keycodes_fast_after_modifier_cleanup(kbd, ctx.events)
    } else {
        replay_keycodes(kbd, ctx.events)
    };
    if let Err(e) = replay_result {
        log(&format!("⚠ Этап 3 replay failed: {e}"));
        return Some(ctx.target_is_ru);
    }
    suppress_next_ime_autocorrect();
    let replay_ms = replay_started.elapsed().as_millis();
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
        "  timing: layout={}ms backspace={}ms replay={}ms total={}ms input_isolated={}",
        layout_ms,
        backspace_ms,
        replay_ms,
        ctx.started_at.elapsed().as_millis(),
        ctx.input_isolated
    ));

    log(&format!(
        "✓ done: раскладка {ibus_engine}, перенабрано {} клавиш за {}ms",
        ctx.events.len(),
        ctx.started_at.elapsed().as_millis()
    ));
    Some(ctx.target_is_ru)
}

fn manual_replay_action(
    ctx: &ManualOutputCommon<'_>,
    input_gate: Option<RecentActionGateTrace>,
) -> Option<AuthorizedEdit> {
    let plan = TextReplacement {
        move_left: 0,
        backspaces: ctx.n_backspaces,
        insert: ctx.mapped_target.to_string(),
        move_right: 0,
    };
    let edit_action = lay::text_edit::authorize_replacement_with_transition(
        "manual-replay",
        1000,
        ctx.mapped_orig,
        ctx.mapped_target,
        plan,
        Some("manual_replay"),
        None,
        TransitionAudit::proven(
            "manual_replay",
            "manual_replay_plan_verified",
            true,
            false,
            ctx.words_orig.max(1),
        ),
    );
    lay::action_log::record_candidate_edit_action_before_apply(
        &edit_action,
        lay::action_log::MutationLogRoute::MANUAL_TEXT_REPLACE,
        input_gate,
    );
    let backend_action = lay::text_edit::authorize_backend_edit(
        lay::text_edit::TextEditBackend::Daemon,
        &edit_action,
    );
    if let Some(authorized_edit) = backend_action.authorized() {
        return Some(authorized_edit);
    }
    log(&format!(
        "⚠ manual replay blocked by executor contract: reason={} backend={} original={:?} replacement={:?}",
        backend_action.reason,
        backend_action.backend.as_str(),
        ctx.mapped_orig,
        ctx.mapped_target
    ));
    None
}
