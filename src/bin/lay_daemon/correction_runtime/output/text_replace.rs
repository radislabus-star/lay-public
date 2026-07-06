use evdev::uinput::VirtualDevice;
use lay::decoder::DecoderAction;
use lay::keyboard::preferred_layout_for_text;
use lay::text_edit::{plan_committed_tail_replacement, replacement_plan_matches, TextReplacement};

use super::super::super::correction_memory_runtime::{
    remember_manual_text_correction, ManualTextCorrectionMemory,
};
use super::super::super::{
    apply_text_replacement_pipeline, log, record_recent_action, switch_to_target_layout,
    tail_replace_policy,
};
use super::context::{ManualOutputCommon, OutputFlow};

pub(crate) fn try_manual_text_replacement(
    ctx: &mut ManualOutputCommon<'_>,
    kbd: &mut VirtualDevice,
) -> OutputFlow {
    let DecoderAction::ReplaceText {
        replacement: text,
        source,
    } = &ctx.decision.action
    else {
        return OutputFlow::ContinueReplay;
    };
    let kind = source.log_kind();
    if text.trim().is_empty() || text == ctx.mapped_target {
        log("  2. text decision совпал с replay — replay для сохранения toggle");
        return OutputFlow::ContinueReplay;
    }

    let plan = manual_text_replacement_plan(ctx, text, kind);
    let edit_action = lay::text_edit::authorize_replacement(
        kind,
        0,
        ctx.mapped_orig,
        text.as_str(),
        plan.clone(),
        Some("manual_toggle"),
        None,
    );
    lay::action_log::record_candidate_edit_action_before_apply(&edit_action, None);
    if !edit_action.allow_apply() {
        log(&format!(
            "⚠ {kind} blocked by EditAction safety: reason={} original={:?} replacement={:?}; fallback to replay",
            edit_action.safety_reason(),
            ctx.mapped_orig,
            text
        ));
        return OutputFlow::ContinueReplay;
    }
    let insert_outcome = match apply_text_replacement_pipeline(
        kbd,
        &plan,
        text,
        ctx.target_is_ru,
        kind,
        ctx.input_isolated,
    ) {
        Ok(outcome) => outcome,
        Err(e) => {
            e.log(kind, "minimal replace failed");
            return OutputFlow::Return(None);
        }
    };
    let insert_target_is_ru = insert_outcome.layout_is_ru;
    let layout_result = if insert_outcome.layout_already_set {
        Ok("already-set")
    } else {
        switch_to_target_layout(insert_target_is_ru)
    };
    remember_text_replacement(ctx, &plan, text, kind, insert_target_is_ru);
    log(&format!(
        "  1. minimal replace: left={} bs={} insert={:?} right={}",
        plan.move_left, plan.backspaces, plan.insert, plan.move_right
    ));
    OutputFlow::Return(match layout_result {
        Ok(layout_id) => {
            log(&format!("  2. layout → {layout_id}"));
            log(&format!(
                "✓ done: {kind}, исправлен BAD-диапазон за {}ms",
                ctx.started_at.elapsed().as_millis()
            ));
            Some(insert_target_is_ru)
        }
        Err(e) => {
            log(&format!(
                "⚠ {kind} layout switch after text insert failed: {e}"
            ));
            log(&format!(
                "✓ done: {kind}, текст исправлен, layout не подтверждён за {}ms",
                ctx.started_at.elapsed().as_millis()
            ));
            None
        }
    })
}

fn remember_text_replacement(
    ctx: &mut ManualOutputCommon<'_>,
    plan: &TextReplacement,
    text: &str,
    kind: &'static str,
    insert_target_is_ru: bool,
) {
    remember_manual_text_correction(
        ctx.buf,
        ManualTextCorrectionMemory {
            events: ctx.events,
            plan,
            original: ctx.mapped_orig,
            replacement: text,
            kind,
            replace_words: ctx.replace_words,
            words: ctx.words_orig,
            inserted_layout_is_ru: Some(preferred_layout_for_text(
                &plan.insert,
                insert_target_is_ru,
            )),
        },
    );
    record_recent_action(
        kind,
        ctx.mapped_orig,
        text,
        ctx.replace_words,
        ctx.words_orig,
        ctx.started_at,
        true,
    );
}

fn manual_text_replacement_plan(
    ctx: &ManualOutputCommon<'_>,
    text: &str,
    kind: &'static str,
) -> TextReplacement {
    if tail_replace_policy::full_tail_replace_required(ctx.mapped_orig) {
        return TextReplacement {
            move_left: 0,
            backspaces: ctx.n_backspaces,
            insert: text.to_string(),
            move_right: 0,
        };
    }
    let mut plan = ctx
        .decision
        .edit
        .as_ref()
        .map(|edit| edit.plan.clone())
        .or_else(|| plan_committed_tail_replacement(ctx.mapped_orig, text))
        .unwrap_or_else(|| TextReplacement {
            move_left: 0,
            backspaces: ctx.n_backspaces,
            insert: text.to_string(),
            move_right: 0,
        });
    if ctx
        .decision
        .edit
        .as_ref()
        .is_some_and(|edit| !edit.plan_matches_replacement())
        || !replacement_plan_matches(ctx.mapped_orig, text, &plan)
    {
        log(&format!(
            "⚠ {kind} plan invariant failed; using full tail replace"
        ));
        plan = TextReplacement {
            move_left: 0,
            backspaces: ctx.n_backspaces,
            insert: text.to_string(),
            move_right: 0,
        };
    }
    plan
}
