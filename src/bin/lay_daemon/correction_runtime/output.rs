#[path = "output/context.rs"]
mod context;
#[path = "output/native.rs"]
mod native;
#[path = "output/replay.rs"]
mod replay;
#[path = "output/text_replace.rs"]
mod text_replace;

pub(super) use self::context::ManualCorrectionOutputContext;
use self::context::{ManualOutputCommon, OutputFlow};
use self::native::{try_gnome_native_replace_output, try_ime_replace_output};
use self::replay::apply_layout_replay;
use self::text_replace::try_manual_text_replacement;

use super::super::{log, release_possible_modifiers, settle_after_physical_trigger_release};

pub(super) fn apply_manual_correction_output(
    ctx: ManualCorrectionOutputContext<'_>,
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
        input_isolated,
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
    };

    if let Some(result) = try_ime_replace_output(&mut common) {
        return result;
    }
    if let Some(result) = try_gnome_native_replace_output(&mut common) {
        return result;
    }

    let kbd = match virtual_kbd {
        Some(k) => k,
        None => {
            log("⚠ нет uinput device");
            return None;
        }
    };
    if common.input_isolated {
        log("  input isolated: skip trigger settle");
        if let Err(e) = super::super::release_possible_modifiers_fast(kbd) {
            log(&format!(
                "⚠ fast modifier cleanup before backspace failed: {e}"
            ));
        }
    } else {
        settle_after_physical_trigger_release();
        if let Err(e) = release_possible_modifiers(kbd) {
            log(&format!("⚠ modifier cleanup before backspace failed: {e}"));
        }
    }

    match try_manual_text_replacement(&mut common, kbd) {
        OutputFlow::Return(result) => result,
        OutputFlow::ContinueReplay => apply_layout_replay(&mut common, kbd),
    }
}
