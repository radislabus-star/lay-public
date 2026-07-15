use lay::action_log::RecentActionGateTrace;
use lay::decoder::DecoderAction;
use lay::desktop::LayoutBackend;
use lay::keyboard::preferred_layout_for_text;
use lay::text_edit::{AuthorizedEdit, TextReplacement, TransitionAudit};

use super::super::super::action_log_runtime::RecentActionRecord;
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
    input_gate: Option<RecentActionGateTrace>,
) -> Option<NativeReplaceOutput> {
    if !should_try_ime_text_backend() {
        return None;
    }
    let (replace_text, replace_kind, is_replay) = text_for_native_replace(ctx, "ime-replay");
    let replace_target_is_ru = preferred_layout_for_text(&replace_text, ctx.target_is_ru);
    let authorized_edit = authorize_native_text_edit(
        ctx,
        &replace_text,
        replace_kind,
        is_replay,
        lay::text_edit::TextEditBackend::Ime,
        input_gate.clone(),
    )?;
    if !try_ime_replace_tail(authorized_edit, replace_kind).unwrap_or(false) {
        return None;
    }

    remember_native_replace(
        ctx,
        &replace_text,
        replace_kind,
        replace_target_is_ru,
        is_replay,
        input_gate,
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
    input_gate: Option<RecentActionGateTrace>,
) -> Option<NativeReplaceOutput> {
    if !(GNOME_NATIVE_REPLACE_EXPERIMENTAL && active_layout_backend() == LayoutBackend::Gnome) {
        return None;
    }
    let (replace_text, replace_kind, is_replay) = text_for_native_replace(ctx, "gnome-replace");
    let replace_target_is_ru = preferred_layout_for_text(&replace_text, ctx.target_is_ru);
    let authorized_edit = authorize_native_text_edit(
        ctx,
        &replace_text,
        replace_kind,
        is_replay,
        lay::text_edit::TextEditBackend::Daemon,
        input_gate.clone(),
    )?;
    let (layout_id, _) = target_layout(replace_target_is_ru);
    match call_replace_text(authorized_edit, layout_id) {
        Ok(true) => {
            remember_native_replace(
                ctx,
                &replace_text,
                replace_kind,
                replace_target_is_ru,
                is_replay,
                input_gate,
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
    input_gate: Option<RecentActionGateTrace>,
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
        record_recent_action(RecentActionRecord {
            kind: replace_kind,
            from: ctx.mapped_orig,
            to: replace_text,
            replace_words: ctx.replace_words,
            words: ctx.words_orig,
            started_at: ctx.started_at,
            input_gate,
            undo_available: true,
        });
    }
}

fn authorize_native_text_edit(
    ctx: &ManualOutputCommon<'_>,
    replace_text: &str,
    replace_kind: &'static str,
    is_replay: bool,
    backend: lay::text_edit::TextEditBackend,
    input_gate: Option<RecentActionGateTrace>,
) -> Option<AuthorizedEdit> {
    let plan = TextReplacement {
        move_left: 0,
        backspaces: ctx.mapped_orig.chars().count() as u32,
        insert: replace_text.to_string(),
        move_right: 0,
    };
    let (source_id, transition_operator, transition_proof, confidence_milli) = if is_replay {
        (
            "manual_replay",
            "manual_native_replay",
            "manual_native_replay_plan_verified",
            1000,
        )
    } else {
        (
            "manual_native_replace",
            "manual_native_replace",
            "manual_native_plan_verified",
            0,
        )
    };
    let edit_action = lay::text_edit::authorize_replacement_with_transition(
        replace_kind,
        confidence_milli,
        ctx.mapped_orig,
        replace_text,
        plan,
        Some(source_id),
        None,
        TransitionAudit::proven(
            transition_operator,
            transition_proof,
            true,
            false,
            ctx.words_orig.max(1),
        ),
    );
    lay::action_log::record_candidate_edit_action_before_apply(
        &edit_action,
        lay::action_log::MutationLogRoute::MANUAL_NATIVE_REPLACE,
        input_gate,
    );
    let backend_action = lay::text_edit::authorize_backend_edit(backend, edit_action);
    let backend = backend_action.backend;
    let reason = backend_action.reason;
    if let Some(authorized_edit) = backend_action.into_authorized() {
        return Some(authorized_edit);
    }
    log(&format!(
        "⚠ {replace_kind} native replace blocked by executor contract: reason={} backend={} original={:?} replacement={:?}",
        reason,
        backend.as_str(),
        ctx.mapped_orig,
        replace_text
    ));
    None
}

fn trailing_space_count(text: &str) -> usize {
    text.chars().rev().take_while(|ch| *ch == ' ').count()
}
