use evdev::uinput::VirtualDevice;
use lay::action_log::RecentActionGateTrace;
use std::time::Instant;

#[path = "replay/action.rs"]
mod action;
#[path = "replay/preflight.rs"]
mod preflight;

use super::super::super::{
    emit_backspaces, log, replay_keycodes, suppress_next_ime_autocorrect, target_layout,
};
use super::super::memory::{remember_layout_replay_success, LayoutReplayMemory};
use super::context::ManualOutputCommon;
use action::manual_replay_action;
use preflight::preflight_manual_replay;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplayEffect {
    Preflight,
    Backspaces,
    Replay,
    SuppressAfterSuccess,
}

fn run_replay_effects(
    mut apply: impl FnMut(ReplayEffect) -> Result<(), String>,
) -> Result<(), (ReplayEffect, String)> {
    for effect in [
        ReplayEffect::Preflight,
        ReplayEffect::Backspaces,
        ReplayEffect::Replay,
        ReplayEffect::SuppressAfterSuccess,
    ] {
        apply(effect).map_err(|error| (effect, error))?;
    }
    Ok(())
}

pub(crate) fn apply_layout_replay(
    ctx: &mut ManualOutputCommon<'_>,
    kbd: &mut VirtualDevice,
    input_gate: Option<RecentActionGateTrace>,
) -> Option<bool> {
    let authorized_edit = manual_replay_action(ctx, input_gate)?;
    let action = authorized_edit.action();
    let Some(plan) = action.plan() else {
        log("⚠ manual replay blocked: AuthorizedEdit has no replacement plan");
        return None;
    };
    if plan.backspaces != ctx.n_backspaces || action.to_text() != ctx.mapped_target {
        log("⚠ manual replay blocked: AuthorizedEdit does not match replay state");
        return None;
    }
    let layout_started = Instant::now();
    let (layout_id, ibus_engine) = target_layout(ctx.target_is_ru);
    let mut layout_ms = 0;
    let mut backspace_started = None;
    let mut backspace_ms = 0;
    let mut replay_started = None;
    if let Err((failed, error)) = run_replay_effects(|effect| match effect {
        ReplayEffect::Preflight => {
            let result = preflight_manual_replay(ctx);
            layout_ms = layout_started.elapsed().as_millis();
            result
        }
        ReplayEffect::Backspaces => {
            backspace_started = Some(Instant::now());
            let result = emit_backspaces(kbd, ctx.n_backspaces).map_err(|error| error.to_string());
            backspace_ms = backspace_started
                .expect("backspace stage started")
                .elapsed()
                .as_millis();
            result
        }
        ReplayEffect::Replay => {
            replay_started = Some(Instant::now());
            replay_keycodes(kbd, ctx.events).map_err(|error| error.to_string())
        }
        ReplayEffect::SuppressAfterSuccess => {
            suppress_next_ime_autocorrect();
            Ok(())
        }
    }) {
        match failed {
            ReplayEffect::Preflight => {
                log(&format!(
                    "⚠ manual replay blocked before Backspace: {error}"
                ));
                log("  replay aborted: исходное слово оставлено на месте");
                return None;
            }
            ReplayEffect::Backspaces => {
                log(&format!("⚠ Этап 2 backspaces failed: {error}"));
                return None;
            }
            ReplayEffect::Replay => {
                log(&format!("⚠ Этап 3 replay failed: {error}"));
                return Some(ctx.target_is_ru);
            }
            ReplayEffect::SuppressAfterSuccess => {
                log(&format!("⚠ autocorrect suppression failed: {error}"));
                return Some(ctx.target_is_ru);
            }
        }
    }
    log(&format!("  1. layout → {layout_id}"));
    log(&format!("  2. uinput Backspace × {}", ctx.n_backspaces));
    let replay_ms = replay_started
        .expect("successful replay stage started")
        .elapsed()
        .as_millis();
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

#[cfg(test)]
mod td120_v1_replay_effect_tests {
    use super::*;

    fn execute(fail_at: Option<ReplayEffect>) -> Vec<ReplayEffect> {
        let mut effects = Vec::new();
        let _ = run_replay_effects(|effect| {
            effects.push(effect);
            if fail_at == Some(effect) {
                Err("injected failure".to_string())
            } else {
                Ok(())
            }
        });
        effects
    }

    #[test]
    fn td120_preflight_backspace_and_replay_failures_never_issue_after_v1_call() {
        for failed in [
            ReplayEffect::Preflight,
            ReplayEffect::Backspaces,
            ReplayEffect::Replay,
        ] {
            let effects = execute(Some(failed));
            assert_eq!(effects.last(), Some(&failed));
            assert!(!effects.contains(&ReplayEffect::SuppressAfterSuccess));
        }
    }

    #[test]
    fn td120_successful_replay_issues_after_v1_call_once() {
        let effects = execute(None);
        assert_eq!(
            effects,
            [
                ReplayEffect::Preflight,
                ReplayEffect::Backspaces,
                ReplayEffect::Replay,
                ReplayEffect::SuppressAfterSuccess,
            ]
        );
    }
}
