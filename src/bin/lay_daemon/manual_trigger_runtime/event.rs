use evdev::KeyCode;
use std::time::{Duration, Instant};

use super::super::{
    log, log_manual_trigger_cross_check, multi_tap_scope_for_taps, DShiftRelease, DShiftState,
    MultiTapPending,
};
use super::context::ManualTriggerEventContext;
use super::fire::{fire_configured_manual_trigger, fire_scoped_manual_trigger};

pub(crate) fn handle_manual_trigger_event(mut ctx: ManualTriggerEventContext<'_>) -> bool {
    if ignored_side_trigger_key(&ctx) {
        cancel_pending_manual_trigger_on_other_key(&mut ctx);
        return true;
    }

    if handle_single_trigger_event(&mut ctx) {
        return true;
    }

    if handle_caps_trigger_event(&mut ctx) {
        return true;
    }

    if handle_double_or_multi_tap_event(&mut ctx) {
        return true;
    }

    cancel_pending_manual_trigger_on_other_key(&mut ctx);
    false
}

fn handle_single_trigger_event(ctx: &mut ManualTriggerEventContext<'_>) -> bool {
    if !ctx.is_single_trigger {
        return false;
    }

    if ctx.key == ctx.trigger_key {
        match ctx.value {
            1 => {
                *ctx.single_pressed_at = Some(Instant::now());
                *ctx.single_other_key = false;
            }
            0 => {
                if let Some(t) = ctx.single_pressed_at.take() {
                    let held = t.elapsed();
                    if !*ctx.single_other_key
                        && held <= ctx.shift_tap_max
                        && !trigger_debounce_active(*ctx.last_double_at, ctx.debounce_window)
                    {
                        log_manual_trigger_cross_check(ctx.buffer, ctx.events_since_word_start);
                        fire_configured_manual_trigger(ctx.fire_context());
                        log(&format!(
                            "· single-trigger fired (held {}ms)",
                            held.as_millis()
                        ));
                    }
                }
            }
            _ => {}
        }
        return true;
    }

    if ctx.value == 1 {
        *ctx.single_other_key = true;
    }
    false
}

fn handle_caps_trigger_event(ctx: &mut ManualTriggerEventContext<'_>) -> bool {
    if !(ctx.is_caps_trigger && ctx.key == KeyCode::KEY_CAPSLOCK && ctx.value == 1) {
        return false;
    }
    if trigger_debounce_active(*ctx.last_double_at, ctx.debounce_window) {
        return true;
    }

    log_manual_trigger_cross_check(ctx.buffer, ctx.events_since_word_start);
    fire_configured_manual_trigger(ctx.fire_context());
    log("· CAPS LOCK triggered");
    true
}

fn handle_double_or_multi_tap_event(ctx: &mut ManualTriggerEventContext<'_>) -> bool {
    if ctx.key != ctx.trigger_key || ctx.is_caps_trigger {
        return false;
    }
    if trigger_debounce_active(*ctx.last_double_at, ctx.debounce_window) {
        return true;
    }

    let now = Instant::now();
    if pending_multi_tap_can_continue(ctx, now) {
        ctx.dshift_state.begin_additional_press();
        if ctx.verbose {
            log("· FSM: multi-tap waiting → AdditionalPress");
        }
        return true;
    }

    match ctx.value {
        1 => {
            let before = *ctx.dshift_state;
            ctx.dshift_state.trigger_press(now, ctx.shift_window);
            if ctx.verbose && before != *ctx.dshift_state {
                log(&format!("· FSM: {before:?} → {:?}", *ctx.dshift_state));
            }
        }
        0 => match ctx.dshift_state.trigger_release(now) {
            DShiftRelease::Double => handle_confirmed_double_shift(ctx, now),
            DShiftRelease::Additional => handle_additional_multi_tap_release(ctx, now),
            DShiftRelease::None => {}
        },
        _ => {}
    }
    true
}

fn handle_confirmed_double_shift(ctx: &mut ManualTriggerEventContext<'_>, now: Instant) {
    if ctx.multi_tap_scope {
        *ctx.pending_multi_tap = Some(MultiTapPending {
            tap_count: 2,
            last_release: now,
        });
        *ctx.dshift_state = DShiftState::Idle;
        if ctx.verbose {
            log("· FSM: DOUBLE captured, wait for optional 3rd tap");
        }
        return;
    }

    log_manual_trigger_cross_check(ctx.buffer, ctx.events_since_word_start);
    fire_configured_manual_trigger(ctx.fire_context());
    log("· FSM: DOUBLE! (p→r→p→r)");
}

fn handle_additional_multi_tap_release(ctx: &mut ManualTriggerEventContext<'_>, now: Instant) {
    if let Some(mut pending) = ctx.pending_multi_tap.take() {
        pending.tap_count = pending.tap_count.saturating_add(1);
        if pending.tap_count >= ctx.multi_tap_max_taps {
            let replace_words = multi_tap_scope_for_taps(pending.tap_count).unwrap_or(3);
            let events_since_word_start = ctx.events_since_word_start;
            fire_scoped_manual_trigger(
                ctx.fire_context(),
                replace_words,
                events_since_word_start,
                "multi-tap max",
            );
        } else {
            pending.last_release = now;
            *ctx.pending_multi_tap = Some(pending);
            *ctx.dshift_state = DShiftState::Idle;
            if ctx.verbose {
                log("· FSM: multi-tap captured, wait for next tap");
            }
        }
    } else {
        *ctx.dshift_state = DShiftState::Idle;
    }
}

fn ignored_side_trigger_key(ctx: &ManualTriggerEventContext<'_>) -> bool {
    !ctx.is_single_trigger && matches!(ctx.key, KeyCode::KEY_RIGHTSHIFT | KeyCode::KEY_RIGHTALT)
}

fn pending_multi_tap_can_continue(ctx: &ManualTriggerEventContext<'_>, now: Instant) -> bool {
    ctx.multi_tap_scope
        && ctx.value == 1
        && ctx
            .pending_multi_tap
            .as_ref()
            .is_some_and(|pending| now.duration_since(pending.last_release) <= ctx.shift_window)
}

fn cancel_pending_manual_trigger_on_other_key(ctx: &mut ManualTriggerEventContext<'_>) {
    if !ctx.dshift_state.is_idle() && ctx.value == 1 {
        if ctx.verbose {
            log(&format!("· FSM: cancel → Idle (key {})", ctx.code));
        }
        ctx.dshift_state.cancel();
    }
    if ctx.pending_multi_tap.is_some() && ctx.value == 1 {
        *ctx.pending_multi_tap = None;
        if ctx.verbose {
            log(&format!("· FSM: multi-tap cancel (key {})", ctx.code));
        }
    }
}

fn trigger_debounce_active(last_double_at: Option<Instant>, debounce_window: Duration) -> bool {
    last_double_at.is_some_and(|last| last.elapsed() < debounce_window)
}
