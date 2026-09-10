use evdev::uinput::VirtualDevice;
use lay::action_log::RecentActionGateTrace;
use lay::decoder::DecoderAction;
use lay::keyboard::preferred_layout_for_text;
use lay::text_edit::{plan_committed_tail_replacement, replacement_plan_matches, TextReplacement};

use super::super::super::action_log_runtime::RecentActionRecord;
use super::super::super::correction_memory_runtime::{
    remember_manual_text_correction, ManualTextCorrectionMemory,
};
use super::super::super::{
    apply_text_replacement_pipeline, log, record_recent_action, switch_to_target_layout,
    tail_replace_policy,
};
use super::context::{ManualOutputCommon, OutputFlow};

struct ManualTextExecution {
    layout_is_ru: bool,
    layout_already_set: bool,
}

pub(crate) fn try_manual_text_replacement(
    ctx: &mut ManualOutputCommon<'_>,
    kbd: &mut VirtualDevice,
    input_gate: Option<RecentActionGateTrace>,
) -> OutputFlow {
    execute_manual_text_replacement_with_effects(
        ctx,
        input_gate,
        |ctx, authorized_edit, kind| {
            let preflight = ctx.text_observation.explicit_manual_preflight(
                ctx.buf,
                ctx.mapped_orig.to_string(),
                ctx.input_isolated,
            );
            match apply_text_replacement_pipeline(
                kbd,
                authorized_edit,
                ctx.target_is_ru,
                None,
                kind,
                ctx.input_isolated,
                preflight,
            ) {
                Ok(outcome) => Ok(ManualTextExecution {
                    layout_is_ru: outcome.layout_is_ru,
                    layout_already_set: outcome.layout_already_set,
                }),
                Err(error) => {
                    error.log(kind, "minimal replace failed");
                    Err(())
                }
            }
        },
        switch_to_target_layout,
    )
}

fn execute_manual_text_replacement_with_effects(
    ctx: &mut ManualOutputCommon<'_>,
    input_gate: Option<RecentActionGateTrace>,
    execute: impl FnOnce(
        &mut ManualOutputCommon<'_>,
        lay::text_edit::AuthorizedEdit,
        &'static str,
    ) -> Result<ManualTextExecution, ()>,
    switch_layout: impl FnOnce(bool) -> Result<&'static str, String>,
) -> OutputFlow {
    let (text, source) = match &ctx.decision.action {
        DecoderAction::ReplaceText {
            replacement,
            source,
        } => (replacement.clone(), *source),
        DecoderAction::KeepOriginal | DecoderAction::ReplayAll => {
            return OutputFlow::ContinueReplay;
        }
    };
    let kind = source.log_kind();
    if text.trim().is_empty() || text == ctx.mapped_target {
        log("  2. text decision совпал с replay — replay для сохранения toggle");
        return OutputFlow::ContinueReplay;
    }

    let plan = manual_text_replacement_plan(ctx, &text, kind);
    let edit_action = lay::text_edit::plan_manual_edit(
        kind,
        0,
        ctx.mapped_orig,
        text.as_str(),
        plan.clone(),
        ctx.words_orig,
    );
    lay::action_log::record_candidate_edit_action_before_apply(
        &edit_action,
        lay::action_log::MutationLogRoute::MANUAL_TEXT_REPLACE,
        input_gate.clone(),
    );
    let backend_action = lay::text_edit::authorize_backend_edit(
        lay::text_edit::TextEditBackend::Daemon,
        edit_action,
    );
    let backend = backend_action.backend;
    let reason = backend_action.reason;
    let Some(authorized_edit) = backend_action.into_authorized() else {
        log(&format!(
            "⚠ {kind} blocked by executor contract: reason={} backend={} original={:?} replacement={:?}; fallback to replay",
            reason,
            backend.as_str(),
            ctx.mapped_orig,
            text
        ));
        return OutputFlow::ContinueReplay;
    };
    if let Some(lease) = &ctx.delegated_tail_lease {
        if let Err(error) = lease.validate_current(ctx.n_backspaces) {
            log(&format!(
                "⚠ {kind} blocked by delegated tail lease: {error}"
            ));
            return OutputFlow::Return(None);
        }
    }
    let insert_outcome = match execute(ctx, authorized_edit, kind) {
        Ok(outcome) => outcome,
        Err(()) => return OutputFlow::Return(None),
    };
    let insert_target_is_ru = insert_outcome.layout_is_ru;
    let layout_result = if insert_outcome.layout_already_set {
        Ok("already-set")
    } else {
        switch_layout(insert_target_is_ru)
    };
    remember_text_replacement(ctx, &plan, &text, kind, insert_target_is_ru, input_gate);
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
    input_gate: Option<RecentActionGateTrace>,
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
    record_recent_action(RecentActionRecord {
        kind,
        from: ctx.mapped_orig,
        to: text,
        replace_words: ctx.replace_words,
        words: ctx.words_orig,
        started_at: ctx.started_at,
        input_gate,
        undo_available: true,
    });
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

#[cfg(test)]
mod td120_replace_text_branch_tests {
    use super::*;
    use crate::{DaemonTextContext, DaemonTextContextObserver, DaemonTextObservation};
    use lay::decoder::{CorrectionSource, DecoderAction};
    use lay::engine::ManualCorrectionDecision;
    use lay::text_edit::TextEditBackend;
    use lay::word_buffer::WordBuffer;
    use std::sync::atomic::AtomicU64;

    fn with_common<T>(run: impl FnOnce(&mut ManualOutputCommon<'_>) -> T) -> T {
        let mut buffer = WordBuffer::new();
        let epoch = AtomicU64::new(11);
        let observation = DaemonTextObservation::new(
            DaemonTextContext::new(Some("td120-replace-field".to_string()), 11),
            DaemonTextContextObserver::new(Some("td120-replace-field"), &epoch),
        );
        let decision = ManualCorrectionDecision {
            action: DecoderAction::ReplaceText {
                replacement: "почитай".to_string(),
                source: CorrectionSource::SmartText,
            },
            edit: None,
            replay_target_is_ru: true,
            replay_mixed_layouts: false,
            output_text: "почитай".to_string(),
            output_target_is_ru: true,
        };
        let mut common = ManualOutputCommon {
            buf: &mut buffer,
            events: &[],
            mapped_orig: "gjxbnfq",
            mapped_target: "привет",
            target_is_ru: true,
            n_backspaces: 7,
            replace_words: 1,
            words_orig: 1,
            force_replay_toggle: false,
            started_at: std::time::Instant::now(),
            decision: &decision,
            input_isolated: true,
            text_observation: observation,
            output_route: super::super::super::ManualCorrectionOutputRoute::DaemonUinput,
            delegated_tail_lease: None,
        };
        run(&mut common)
    }

    fn execute(success: bool) -> (Option<bool>, usize, usize, usize, usize) {
        with_common(|common| {
            let mut v1_calls = 0usize;
            let mut pipeline_calls = 0usize;
            let mut switch_calls = 0usize;
            let mut replay_calls = 0usize;
            let result = super::super::run_uinput_output_branch(|effect| match effect {
                super::super::UinputOutputEffect::SuppressBeforeOutput => {
                    v1_calls += 1;
                    super::super::UinputOutputEffectResult::Applied
                }
                super::super::UinputOutputEffect::Prepare => {
                    super::super::UinputOutputEffectResult::Applied
                }
                super::super::UinputOutputEffect::TryTextReplacement => {
                    super::super::UinputOutputEffectResult::TextFlow(
                        execute_manual_text_replacement_with_effects(
                            common,
                            None,
                            |_, authorized, kind| {
                                pipeline_calls += 1;
                                assert_eq!(authorized.backend(), TextEditBackend::Daemon);
                                assert_eq!(authorized.action().to_text(), "почитай");
                                assert_eq!(kind, "smart-text");
                                if success {
                                    Ok(ManualTextExecution {
                                        layout_is_ru: true,
                                        layout_already_set: false,
                                    })
                                } else {
                                    Err(())
                                }
                            },
                            |target_is_ru| {
                                switch_calls += 1;
                                assert!(target_is_ru);
                                Ok("ru")
                            },
                        ),
                    )
                }
                super::super::UinputOutputEffect::Replay => {
                    replay_calls += 1;
                    super::super::UinputOutputEffectResult::ReplayResult(Some(false))
                }
            });
            (result, v1_calls, pipeline_calls, switch_calls, replay_calls)
        })
    }

    #[test]
    fn td120_replace_text_executes_real_decoder_branch_before_only_without_replay() {
        assert_eq!(execute(true), (Some(true), 1, 1, 1, 0));
        assert_eq!(execute(false), (None, 1, 1, 0, 0));
    }
}
