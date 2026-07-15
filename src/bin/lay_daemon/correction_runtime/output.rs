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
    };

    let mut virtual_kbd = virtual_kbd;
    let mut physical_grab = physical_grab;

    if let Some(result) = try_native_output_stage(
        &mut common,
        &mut virtual_kbd,
        &mut physical_grab,
        input_gate.clone(),
    ) {
        return result;
    }

    let kbd = match virtual_kbd {
        Some(k) => k,
        None => {
            log("⚠ нет uinput device");
            return None;
        }
    };
    suppress_next_ime_autocorrect();
    prepare_uinput_output(kbd, common.input_isolated);
    match try_manual_text_replacement(&mut common, kbd, input_gate.clone()) {
        OutputFlow::Return(result) => result,
        OutputFlow::ContinueReplay => apply_layout_replay(&mut common, kbd, input_gate),
    }
}
