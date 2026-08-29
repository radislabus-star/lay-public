use evdev::{uinput::VirtualDevice, Device, KeyCode};
use lay::manual_toggle::ImeManualToggleOutcome;
use lay::word_buffer::WordBuffer;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::pending_typing_assist::PendingTypingAssist;

use super::physical_input_grab::PhysicalInputGrab;
use super::{
    active_replace_words, capture_ime_committed_tail_replay, execute_exact_ime_tail_replay,
    handle_double_shift, lock_virtual_keyboard, run_manual_correction_with_scope,
    try_ime_manual_toggle, wait_for_ime_committed_tail_settlement, ImeCommittedTailReplay,
    ManualCorrectionOutputRoute, ManualCorrectionRequest, ScopedManualCorrectionRequest,
};
use super::{DShiftState, DaemonTextObservation, MultiTapPending, ShiftState};

pub(super) fn trigger_key_from_config(trigger: &str) -> KeyCode {
    match trigger {
        "double-ctrl" => KeyCode::KEY_LEFTCTRL,
        "double-alt" => KeyCode::KEY_LEFTALT,
        "caps-lock" => KeyCode::KEY_CAPSLOCK,
        "single-rshift" => KeyCode::KEY_RIGHTSHIFT,
        "single-rctrl" => KeyCode::KEY_RIGHTCTRL,
        "single-ralt" => KeyCode::KEY_RIGHTALT,
        "single-pause" => KeyCode::KEY_PAUSE,
        _ => KeyCode::KEY_LEFTSHIFT,
    }
}

pub(super) fn is_single_trigger_id(trigger: &str) -> bool {
    trigger.starts_with("single-")
}

pub(super) fn run_configured_manual_correction(
    buffer: &mut WordBuffer,
    device: &mut Device,
    virtual_kbd: &Arc<Mutex<Option<VirtualDevice>>>,
    executing: &mut bool,
    text_observation: DaemonTextObservation<'_>,
    output_route: ManualCorrectionOutputRoute,
) -> Option<bool> {
    let mut physical_grab = if !output_route.requires_physical_grab() {
        // The IME committed-tail executor completes without key replay. Keep
        // subsequent physical Shift taps in the normal event stream so every
        // complete press-release pair remains a separate toggle.
        PhysicalInputGrab::new(None)
    } else {
        PhysicalInputGrab::new(Some(device))
    };
    let input_isolated = physical_grab.is_active();
    let mut g = lock_virtual_keyboard(virtual_kbd);
    handle_double_shift(ManualCorrectionRequest {
        buf: buffer,
        replace_words: active_replace_words(),
        virtual_kbd: g.as_mut(),
        executing,
        input_isolated,
        text_observation,
        physical_grab: Some(&mut physical_grab),
        output_route,
    })
}

pub(super) fn run_exact_ime_tail_replay(
    buffer: &mut WordBuffer,
    device: &mut Device,
    virtual_kbd: &Arc<Mutex<Option<VirtualDevice>>>,
    executing: &mut bool,
    replay: ImeCommittedTailReplay,
) -> Option<bool> {
    let mut keyboard = lock_virtual_keyboard(virtual_kbd);
    let mut physical_grab = PhysicalInputGrab::new(Some(device));
    let input_isolated = physical_grab.is_active();
    let mut result =
        execute_exact_ime_tail_replay(buffer, keyboard.as_mut(), executing, input_isolated, replay);
    if let (Some(layout_is_ru), Some(virtual_keyboard)) = (result, keyboard.as_mut()) {
        let mut replay_queued_manual_toggle =
            |queued_keyboard: &mut VirtualDevice, queued_buffer: &mut WordBuffer| {
                let Some(expected_tail) =
                    queued_buffer.visible_tail_text(lay::word_buffer::MAX_REPLACE_WORDS)
                else {
                    super::log("warning: queued Double Shift has no settled daemon tail");
                    return None;
                };
                if let Err(error) =
                    wait_for_ime_committed_tail_settlement(&expected_tail, layout_is_ru)
                {
                    super::log(&format!(
                        "warning: queued Double Shift settlement failed: {error}"
                    ));
                    return None;
                }
                match try_ime_manual_toggle() {
                    Ok(ImeManualToggleOutcome::DelegateExactImeTail) => {}
                    Ok(ImeManualToggleOutcome::Handled {
                        target_layout_is_ru,
                    }) => return Some(target_layout_is_ru),
                    Ok(other) => {
                        super::log(&format!(
                            "warning: queued Double Shift IME admission rejected: {other:?}"
                        ));
                        return None;
                    }
                    Err(error) => {
                        super::log(&format!(
                            "warning: queued Double Shift IME admission failed: {error}"
                        ));
                        return None;
                    }
                }
                let replay = match capture_ime_committed_tail_replay() {
                    Ok(replay) => replay,
                    Err(error) => {
                        super::log(&format!(
                            "warning: queued Double Shift exact tail capture failed: {error}"
                        ));
                        return None;
                    }
                };
                execute_exact_ime_tail_replay(
                    queued_buffer,
                    Some(queued_keyboard),
                    executing,
                    input_isolated,
                    replay,
                )
            };
        let forwarded = physical_grab.forward_queued_typing_with_manual_toggles(
            virtual_keyboard,
            buffer,
            layout_is_ru,
            "exact-ime-tail-replay",
            0,
            true,
            &mut replay_queued_manual_toggle,
        );
        if forwarded.last_manual_toggle_layout_is_ru.is_some() {
            result = forwarded.last_manual_toggle_layout_is_ru;
        }
    }
    drop(physical_grab);
    result
}

pub(super) fn run_scoped_manual_correction(
    ctx: ScopedManualCorrectionContext<'_>,
    replace_words: usize,
    events_since_word_start: u32,
    reason: &str,
    output_route: ManualCorrectionOutputRoute,
) -> Option<bool> {
    let mut physical_grab = if !output_route.requires_physical_grab() {
        PhysicalInputGrab::new(None)
    } else {
        PhysicalInputGrab::new(Some(ctx.device))
    };
    let input_isolated = physical_grab.is_active();
    let mut g = lock_virtual_keyboard(ctx.virtual_kbd);
    run_manual_correction_with_scope(ScopedManualCorrectionRequest {
        manual: ManualCorrectionRequest {
            buf: ctx.buffer,
            replace_words,
            virtual_kbd: g.as_mut(),
            executing: ctx.executing,
            input_isolated,
            text_observation: ctx.text_observation,
            physical_grab: Some(&mut physical_grab),
            output_route,
        },
        events_since_word_start,
        label: reason,
    })
}

pub(super) struct ScopedManualCorrectionContext<'a> {
    pub(super) buffer: &'a mut WordBuffer,
    pub(super) device: &'a mut Device,
    pub(super) virtual_kbd: &'a Arc<Mutex<Option<VirtualDevice>>>,
    pub(super) executing: &'a mut bool,
    pub(super) text_observation: DaemonTextObservation<'a>,
}

pub(super) fn apply_manual_correction_result(
    correction_result: Option<bool>,
    current_layout_is_ru: &mut bool,
    last_layout_poll: &mut Instant,
    suppress_next_typing_assist_after_manual_replay: &mut bool,
    pending_typing_assist_after_space: &mut Option<PendingTypingAssist>,
) {
    if let Some(is_ru) = correction_result {
        *current_layout_is_ru = is_ru;
        *last_layout_poll = Instant::now();
        *suppress_next_typing_assist_after_manual_replay = true;
        pending_typing_assist_after_space.take();
    }
}

pub(super) struct ManualTriggerCompletion<'a> {
    pub(super) current_layout_is_ru: &'a mut bool,
    pub(super) last_layout_poll: &'a mut Instant,
    pub(super) suppress_next_typing_assist_after_manual_replay: &'a mut bool,
    pub(super) pending_typing_assist_after_space: &'a mut Option<PendingTypingAssist>,
    pub(super) shift_state: &'a mut ShiftState,
    pub(super) dshift_state: &'a mut DShiftState,
    pub(super) pending_multi_tap: &'a mut Option<MultiTapPending>,
    pub(super) last_double_at: &'a mut Option<Instant>,
    pub(super) clear_on_next_typing: &'a mut bool,
}

pub(super) fn complete_manual_trigger(
    correction_result: Option<bool>,
    ctx: ManualTriggerCompletion<'_>,
) {
    apply_manual_correction_result(
        correction_result,
        ctx.current_layout_is_ru,
        ctx.last_layout_poll,
        ctx.suppress_next_typing_assist_after_manual_replay,
        ctx.pending_typing_assist_after_space,
    );
    ctx.shift_state.clear_shifts();
    *ctx.dshift_state = DShiftState::Idle;
    *ctx.pending_multi_tap = None;
    *ctx.last_double_at = Some(Instant::now());
    *ctx.clear_on_next_typing = true;
}

pub(super) fn reject_manual_trigger(ctx: ManualTriggerCompletion<'_>) {
    ctx.shift_state.clear_shifts();
    *ctx.dshift_state = DShiftState::Idle;
    *ctx.pending_multi_tap = None;
    *ctx.last_double_at = Some(Instant::now());
    *ctx.clear_on_next_typing = true;
}
