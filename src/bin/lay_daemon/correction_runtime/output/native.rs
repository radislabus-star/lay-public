use lay::decoder::DecoderAction;
use lay::desktop::LayoutBackend;
use lay::keyboard::preferred_layout_for_text;
use lay::text_edit::TextReplacement;

use super::super::super::correction_memory_runtime::{
    remember_manual_text_correction, ManualTextCorrectionMemory,
};
use super::super::super::{
    active_layout_backend, call_replace_text, log, record_recent_action,
    should_try_ime_text_backend, switch_to_target_layout, target_layout, try_ime_replace_tail,
    GNOME_NATIVE_REPLACE_EXPERIMENTAL,
};
use super::super::memory::{remember_layout_replay_success, LayoutReplayMemory};
use super::context::ManualOutputCommon;

pub(crate) struct NativeReplaceOutput {
    pub(crate) result: Option<bool>,
    pub(crate) layout_is_ru: bool,
    pub(crate) trailing_spaces: usize,
}

pub(crate) fn try_ime_replace_output(
    ctx: &mut ManualOutputCommon<'_>,
) -> Option<NativeReplaceOutput> {
    if !should_try_ime_text_backend() {
        return None;
    }
    let (replace_text, replace_kind, is_replay) = text_for_native_replace(ctx, "ime-replay");
    let replace_target_is_ru = preferred_layout_for_text(&replace_text, ctx.target_is_ru);
    if !native_text_edit_action_allowed(ctx, &replace_text, replace_kind, is_replay) {
        return None;
    }
    if !try_ime_replace_tail(ctx.mapped_orig, &replace_text, replace_kind).unwrap_or(false) {
        return None;
    }

    remember_native_replace(
        ctx,
        &replace_text,
        replace_kind,
        replace_target_is_ru,
        is_replay,
    );
    let result = match switch_to_target_layout(replace_target_is_ru) {
        Ok(layout_id) => {
            log(&format!("  layout → {layout_id}"));
            log(&format!(
                "✓ done: {replace_kind}, IME replace-tail за {}ms",
                ctx.started_at.elapsed().as_millis()
            ));
            Some(replace_target_is_ru)
        }
        Err(e) => {
            log(&format!(
                "⚠ {replace_kind} IME text committed, layout switch failed: {e}"
            ));
            None
        }
    };
    Some(NativeReplaceOutput {
        result,
        layout_is_ru: replace_target_is_ru,
        trailing_spaces: trailing_space_count(&replace_text),
    })
}

pub(crate) fn try_gnome_native_replace_output(
    ctx: &mut ManualOutputCommon<'_>,
) -> Option<NativeReplaceOutput> {
    if !(GNOME_NATIVE_REPLACE_EXPERIMENTAL && active_layout_backend() == LayoutBackend::Gnome) {
        return None;
    }
    let (replace_text, replace_kind, is_replay) = text_for_native_replace(ctx, "gnome-replace");
    let replace_target_is_ru = preferred_layout_for_text(&replace_text, ctx.target_is_ru);
    if !native_text_edit_action_allowed(ctx, &replace_text, replace_kind, is_replay) {
        return None;
    }
    let (layout_id, _) = target_layout(replace_target_is_ru);
    match call_replace_text(0, ctx.n_backspaces, &replace_text, 0, layout_id) {
        Ok(true) => {
            remember_native_replace(
                ctx,
                &replace_text,
                replace_kind,
                replace_target_is_ru,
                is_replay,
            );
            log(&format!(
                "  1. GNOME ReplaceText: bs={} insert={:?}",
                ctx.n_backspaces, replace_text
            ));
            log(&format!("  2. layout → {layout_id}"));
            log(&format!(
                "✓ done: {replace_kind}, GNOME-native replace за {}ms",
                ctx.started_at.elapsed().as_millis()
            ));
            Some(NativeReplaceOutput {
                result: Some(replace_target_is_ru),
                layout_is_ru: replace_target_is_ru,
                trailing_spaces: trailing_space_count(&replace_text),
            })
        }
        Ok(false) => {
            log("⚠ GNOME ReplaceText returned false; fallback to uinput replay");
            None
        }
        Err(e) => {
            log(&format!(
                "⚠ GNOME ReplaceText failed: {e}; fallback to uinput replay"
            ));
            None
        }
    }
}

fn text_for_native_replace(
    ctx: &ManualOutputCommon<'_>,
    replay_kind: &'static str,
) -> (String, &'static str, bool) {
    match &ctx.decision.action {
        DecoderAction::ReplaceText {
            replacement,
            source,
        } if !replacement.trim().is_empty() => (replacement.clone(), source.log_kind(), false),
        _ => (ctx.mapped_target.to_string(), replay_kind, true),
    }
}

fn remember_native_replace(
    ctx: &mut ManualOutputCommon<'_>,
    replace_text: &str,
    replace_kind: &'static str,
    replace_target_is_ru: bool,
    is_replay: bool,
) {
    if is_replay {
        remember_layout_replay_success(
            ctx.buf,
            LayoutReplayMemory {
                replace_words: ctx.replace_words,
                target_is_ru: replace_target_is_ru,
                force_replay_toggle: ctx.force_replay_toggle,
                original: ctx.mapped_orig,
                replacement: replace_text,
                words: ctx.words_orig,
                elapsed_ms: ctx.started_at.elapsed().as_millis(),
            },
        );
    } else {
        let plan = TextReplacement {
            move_left: 0,
            backspaces: ctx.mapped_orig.chars().count() as u32,
            insert: replace_text.to_string(),
            move_right: 0,
        };
        remember_manual_text_correction(
            ctx.buf,
            ManualTextCorrectionMemory {
                events: ctx.events,
                plan: &plan,
                original: ctx.mapped_orig,
                replacement: replace_text,
                kind: replace_kind,
                replace_words: ctx.replace_words,
                words: ctx.words_orig,
                inserted_layout_is_ru: None,
            },
        );
        record_recent_action(
            replace_kind,
            ctx.mapped_orig,
            replace_text,
            ctx.replace_words,
            ctx.words_orig,
            ctx.started_at,
            true,
        );
    }
}

fn native_text_edit_action_allowed(
    ctx: &ManualOutputCommon<'_>,
    replace_text: &str,
    replace_kind: &'static str,
    is_replay: bool,
) -> bool {
    if is_replay {
        return true;
    }
    let plan = TextReplacement {
        move_left: 0,
        backspaces: ctx.mapped_orig.chars().count() as u32,
        insert: replace_text.to_string(),
        move_right: 0,
    };
    let edit_action = lay::text_edit::authorize_replacement(
        replace_kind,
        0,
        ctx.mapped_orig,
        replace_text,
        plan,
        Some("manual_native_replace"),
        None,
    );
    if edit_action.allow_apply() {
        return true;
    }
    log(&format!(
        "⚠ {replace_kind} native replace blocked by EditAction safety: reason={} original={:?} replacement={:?}",
        edit_action.safety_reason(),
        ctx.mapped_orig,
        replace_text
    ));
    false
}

fn trailing_space_count(text: &str) -> usize {
    text.chars().rev().take_while(|ch| *ch == ' ').count()
}
