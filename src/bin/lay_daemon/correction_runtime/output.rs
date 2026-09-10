#[path = "output/context.rs"]
mod context;
#[path = "output/native.rs"]
mod native;
#[path = "output/native_stage.rs"]
mod native_stage;
#[path = "output/replay.rs"]
mod replay;
#[path = "output/text_replace.rs"]
mod text_replace;
#[path = "output/uinput_prepare.rs"]
mod uinput_prepare;

use lay::action_log::RecentActionGateTrace;

pub(super) use self::context::ManualCorrectionOutputContext;
use self::context::{ManualOutputCommon, OutputFlow};
use self::native_stage::try_native_output_stage;
use self::replay::apply_layout_replay;
use self::text_replace::try_manual_text_replacement;
use self::uinput_prepare::prepare_uinput_output;

use super::super::{log, suppress_next_ime_autocorrect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UinputOutputEffect {
    SuppressBeforeOutput,
    Prepare,
    TryTextReplacement,
    Replay,
}

enum UinputOutputEffectResult {
    Applied,
    TextFlow(OutputFlow),
    ReplayResult(Option<bool>),
}

fn run_uinput_output_branch(
    mut apply: impl FnMut(UinputOutputEffect) -> UinputOutputEffectResult,
) -> Option<bool> {
    if !matches!(
        apply(UinputOutputEffect::SuppressBeforeOutput),
        UinputOutputEffectResult::Applied
    ) {
        unreachable!("suppression effect must complete before output")
    }
    if !matches!(
        apply(UinputOutputEffect::Prepare),
        UinputOutputEffectResult::Applied
    ) {
        unreachable!("preparation effect must complete before output")
    }
    let UinputOutputEffectResult::TextFlow(flow) = apply(UinputOutputEffect::TryTextReplacement)
    else {
        unreachable!("text replacement effect must return its branch flow")
    };
    match flow {
        OutputFlow::Return(result) => result,
        OutputFlow::ContinueReplay => match apply(UinputOutputEffect::Replay) {
            UinputOutputEffectResult::ReplayResult(result) => result,
            _ => unreachable!("replay effect must return its result"),
        },
    }
}

fn run_after_native_output(
    native_result: Option<Option<bool>>,
    run_uinput: impl FnOnce() -> Option<bool>,
) -> Option<bool> {
    match native_result {
        Some(result) => result,
        None => run_uinput(),
    }
}

pub(super) fn apply_manual_correction_output(
    ctx: ManualCorrectionOutputContext<'_, '_>,
    input_gate: Option<RecentActionGateTrace>,
) -> Option<bool> {
    let ManualCorrectionOutputContext {
        buf,
        events,
        mapped_orig,
        mapped_target,
        target_is_ru,
        n_backspaces,
        replace_words,
        words_orig,
        force_replay_toggle,
        started_at,
        decision,
        virtual_kbd,
        physical_grab,
        input_isolated,
        text_observation,
        output_route,
        delegated_tail_lease,
    } = ctx;
    let mut common = ManualOutputCommon {
        buf,
        events,
        mapped_orig,
        mapped_target,
        target_is_ru,
        n_backspaces,
        replace_words,
        words_orig,
        force_replay_toggle,
        started_at,
        decision,
        input_isolated,
        text_observation,
        output_route,
        delegated_tail_lease,
    };

    let mut virtual_kbd = virtual_kbd;
    let mut physical_grab = physical_grab;

    let native_result = if common.output_route.allows_native_stage() {
        try_native_output_stage(
            &mut common,
            &mut virtual_kbd,
            &mut physical_grab,
            input_gate.clone(),
        )
    } else {
        log("· explicit IME delegation selected daemon uinput output");
        None
    };
    run_after_native_output(native_result, || {
        let kbd = match virtual_kbd {
            Some(k) => k,
            None => {
                log("⚠ нет uinput device");
                return None;
            }
        };
        run_uinput_output_branch(|effect| match effect {
            UinputOutputEffect::SuppressBeforeOutput => {
                suppress_next_ime_autocorrect();
                UinputOutputEffectResult::Applied
            }
            UinputOutputEffect::Prepare => {
                prepare_uinput_output(kbd, common.input_isolated);
                UinputOutputEffectResult::Applied
            }
            UinputOutputEffect::TryTextReplacement => UinputOutputEffectResult::TextFlow(
                try_manual_text_replacement(&mut common, kbd, input_gate.clone()),
            ),
            UinputOutputEffect::Replay => UinputOutputEffectResult::ReplayResult(
                apply_layout_replay(&mut common, kbd, input_gate.clone()),
            ),
        })
    })
}

#[cfg(test)]
mod td120_v1_output_branch_tests {
    use super::*;
    fn execute(
        text_flow: OutputFlow,
        replay_result: Option<bool>,
    ) -> (Option<bool>, Vec<UinputOutputEffect>) {
        let mut text_flow = Some(text_flow);
        let mut effects = Vec::new();
        let result = run_uinput_output_branch(|effect| {
            effects.push(effect);
            match effect {
                UinputOutputEffect::SuppressBeforeOutput | UinputOutputEffect::Prepare => {
                    UinputOutputEffectResult::Applied
                }
                UinputOutputEffect::TryTextReplacement => {
                    UinputOutputEffectResult::TextFlow(text_flow.take().expect("one text branch"))
                }
                UinputOutputEffect::Replay => UinputOutputEffectResult::ReplayResult(replay_result),
            }
        });
        (result, effects)
    }

    #[test]
    fn td120_replace_text_success_and_failure_are_before_only_v1_schedules() {
        for result in [Some(true), None] {
            let (actual_result, effects) = execute(OutputFlow::Return(result), None);
            assert_eq!(actual_result, result);
            assert_eq!(
                effects,
                [
                    UinputOutputEffect::SuppressBeforeOutput,
                    UinputOutputEffect::Prepare,
                    UinputOutputEffect::TryTextReplacement,
                ]
            );
        }
    }

    #[test]
    fn td120_replayall_runs_existing_branch_after_before_v1_request() {
        let (result, effects) = execute(OutputFlow::ContinueReplay, Some(true));
        assert_eq!(result, Some(true));
        assert_eq!(
            effects,
            [
                UinputOutputEffect::SuppressBeforeOutput,
                UinputOutputEffect::Prepare,
                UinputOutputEffect::TryTextReplacement,
                UinputOutputEffect::Replay,
            ]
        );
    }
}
