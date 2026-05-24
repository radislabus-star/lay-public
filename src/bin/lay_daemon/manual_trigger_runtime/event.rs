use evdev::KeyCode;
use std::time::{Duration, Instant};

use super::super::{
    log, log_manual_trigger_cross_check, multi_tap_scope_for_taps, DShiftState, MultiTapPending,
};
use super::context::ManualTriggerEventContext;
use super::fire::{fire_configured_manual_trigger, fire_scoped_manual_trigger};

pub(crate) fn handle_manual_trigger_event(mut ctx: ManualTriggerEventContext<'_>) -> bool {
    if !ctx.is_single_trigger
        && (ctx.key == KeyCode::KEY_RIGHTSHIFT || ctx.key == KeyCode::KEY_RIGHTALT)
    {
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
    if ctx.multi_tap_scope
        && ctx.pending_multi_tap.is_some()
        && ctx.value == 1
        && ctx
            .pending_multi_tap
            .as_ref()
            .is_some_and(|pending| now.duration_since(pending.last_release) <= ctx.shift_window)
    {
        *ctx.dshift_state = DShiftState::AdditionalPress { pressed_at: now };
        if ctx.verbose {
            log("· FSM: multi-tap waiting → AdditionalPress");
        }
        return true;
    }

    match (ctx.value, *ctx.dshift_state) {
        (1, DShiftState::Idle) => {
            *ctx.dshift_state = DShiftState::FirstPress { pressed_at: now };
            if ctx.verbose {
                log("· FSM: Idle → FirstPress");
            }
        }
        (1, DShiftState::WaitingSecond { first_release }) => {
            if now.duration_since(first_release) <= ctx.shift_window {
                *ctx.dshift_state = DShiftState::SecondPress { second_press: now };
                if ctx.verbose {
                    log("· FSM: WaitingSecond → SecondPress");
                }
            } else {
                *ctx.dshift_state = DShiftState::FirstPress { pressed_at: now };
                if ctx.verbose {
                    log("· FSM: timeout → FirstPress");
                }
            }
        }
        (1, _) => {}
        (0, DShiftState::FirstPress { pressed_at }) => {
            let held = now.duration_since(pressed_at);
            if held <= ctx.shift_tap_max {
                *ctx.dshift_state = DShiftState::WaitingSecond { first_release: now };
                if ctx.verbose {
                    log(&format!(
                        "· FSM: FirstPress → WaitingSecond (held {}ms)",
                        held.as_millis()
                    ));
                }
            } else {
                *ctx.dshift_state = DShiftState::Idle;
                if ctx.verbose {
                    log(&format!(
                        "· FSM: FirstPress → Idle (held {}ms, заглавная)",
                        held.as_millis()
                    ));
                }
            }
        }
        (0, DShiftState::SecondPress { second_press, .. }) => {
            let held = now.duration_since(second_press);
            if held <= ctx.shift_tap_max {
                handle_confirmed_double_shift(ctx, now);
            } else {
                *ctx.dshift_state = DShiftState::Idle;
                if ctx.verbose {
                    log(&format!(
                        "· FSM: SecondPress → Idle (held {}ms, не тап)",
                        held.as_millis()
                    ));
                }
            }
        }
        (0, DShiftState::AdditionalPress { pressed_at }) => {
            handle_additional_multi_tap_release(ctx, now, pressed_at);
        }
        (0, _) => {
            *ctx.dshift_state = DShiftState::Idle;
        }
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

fn handle_additional_multi_tap_release(
    ctx: &mut ManualTriggerEventContext<'_>,
    now: Instant,
    pressed_at: Instant,
) {
    let held = now.duration_since(pressed_at);
    if held > ctx.shift_tap_max {
        *ctx.pending_multi_tap = None;
        *ctx.dshift_state = DShiftState::Idle;
        if ctx.verbose {
            log(&format!(
                "· FSM: AdditionalPress → Idle (held {}ms, не тап)",
                held.as_millis()
            ));
        }
        return;
    }

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

fn cancel_pending_manual_trigger_on_other_key(ctx: &mut ManualTriggerEventContext<'_>) {
    if !matches!(
        *ctx.dshift_state,
        DShiftState::Idle | DShiftState::SecondPress { .. }
    ) && ctx.value == 1
    {
        if ctx.verbose {
            log(&format!("· FSM: cancel → Idle (key {})", ctx.code));
        }
        *ctx.dshift_state = DShiftState::Idle;
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
