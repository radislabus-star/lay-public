use evdev::{uinput::VirtualDevice, KeyCode};
use lay::keyboard::{preferred_layout_for_text, text_to_uinput_runs, TextInputRun};
use lay::text_edit::{AuthorizedEdit, TextEditBackend, TextReplacement};
use std::time::Instant;

use super::super::{log, switch_to_target_layout};
use super::key_emit::{
    emit_backspaces_for_text_replace, emit_backspaces_for_text_replace_fast, emit_key_taps,
    emit_key_taps_fast, replay_text_insert_keycodes,
    replay_text_insert_keycodes_fast_after_modifier_cleanup,
};

const TEXT_REPLACE_KEY_PACE_MS: u64 = 1;

#[derive(Debug, Clone)]
struct PreparedTextInsert {
    runs: Vec<TextInputRun>,
    insert_layout_is_ru: bool,
    preflight_final_layout_is_ru: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextInsertOutcome {
    pub layout_is_ru: bool,
    pub layout_already_set: bool,
}

#[derive(Debug)]
pub(crate) enum TextReplacementPipelineError {
    Preflight(String),
    Delete(std::io::Error),
    Insert(String),
}

impl TextReplacementPipelineError {
    pub(crate) fn log(self, label: &str, delete_failure_label: &str) {
        match self {
            Self::Preflight(e) => log(&format!("⚠ {label} skipped before delete: {e}")),
            Self::Delete(e) => log(&format!("⚠ {label} {delete_failure_label}: {e}")),
            Self::Insert(e) => log(&format!("⚠ {label} {e}")),
        }
    }
}

fn prepare_text_insert_for_replacement_plan(
    plan: &TextReplacement,
    fallback_layout_is_ru: bool,
    known_current_layout_is_ru: Option<bool>,
) -> Result<PreparedTextInsert, String> {
    let insert_layout_is_ru = preferred_layout_for_text(&plan.insert, fallback_layout_is_ru);
    let runs = text_to_uinput_runs(&plan.insert, insert_layout_is_ru)
        .ok_or_else(|| "text insert requires unsafe TypeText fallback".to_string())?;
    let mut preflight_final_layout_is_ru = None;
    let mut current_layout_is_ru = known_current_layout_is_ru;
    for run in &runs {
        if current_layout_is_ru == Some(run.target_is_ru) {
            preflight_final_layout_is_ru = current_layout_is_ru;
            continue;
        }
        switch_to_target_layout(run.target_is_ru)
            .map_err(|e| format!("layout preflight failed before destructive edit: {e}"))?;
        current_layout_is_ru = Some(run.target_is_ru);
        preflight_final_layout_is_ru = Some(run.target_is_ru);
    }
    Ok(PreparedTextInsert {
        runs,
        insert_layout_is_ru,
        preflight_final_layout_is_ru,
    })
}

fn apply_text_replacement(
    dev: &mut VirtualDevice,
    plan: &TextReplacement,
    fast_output: bool,
) -> std::io::Result<()> {
    emit_key_taps(
        dev,
        KeyCode::KEY_LEFT,
        plan.move_left,
        if fast_output {
            0
        } else {
            TEXT_REPLACE_KEY_PACE_MS
        },
    )?;
    if fast_output {
        emit_backspaces_for_text_replace_fast(dev, plan.backspaces)?;
    } else {
        emit_backspaces_for_text_replace(dev, plan.backspaces)?;
    }
    Ok(())
}

pub(crate) fn apply_text_replacement_pipeline(
    dev: &mut VirtualDevice,
    authorized: AuthorizedEdit,
    fallback_layout_is_ru: bool,
    known_current_layout_is_ru: Option<bool>,
    label: &str,
    fast_output: bool,
) -> Result<TextInsertOutcome, TextReplacementPipelineError> {
    if authorized.backend() != TextEditBackend::Daemon {
        return Err(TextReplacementPipelineError::Preflight(format!(
            "authorized backend mismatch: expected daemon, got {}",
            authorized.backend().as_str()
        )));
    }
    let action = authorized.action();
    let plan = action.plan.as_ref().ok_or_else(|| {
        TextReplacementPipelineError::Preflight(
            "authorized edit has no replacement plan".to_string(),
        )
    })?;
    let replacement = action.to_text.as_str();
    let pipeline_started = Instant::now();
    let prepare_started = Instant::now();
    let prepared_insert = prepare_text_insert_for_replacement_plan(
        plan,
        fallback_layout_is_ru,
        known_current_layout_is_ru,
    )
    .map_err(TextReplacementPipelineError::Preflight)?;
    let prepare_ms = prepare_started.elapsed().as_millis();
    let delete_started = Instant::now();
    apply_text_replacement(dev, plan, fast_output).map_err(TextReplacementPipelineError::Delete)?;
    let delete_ms = delete_started.elapsed().as_millis();
    let insert_started = Instant::now();
    let outcome = insert_prepared_text_for_replacement_plan(
        dev,
        plan,
        replacement,
        &prepared_insert,
        label,
        fast_output,
    )
    .map_err(TextReplacementPipelineError::Insert)?;
    let insert_ms = insert_started.elapsed().as_millis();
    let pipeline_ms = pipeline_started.elapsed().as_millis();
    log(&format!(
        "  {label} timing: prepare={}ms delete={}ms insert={}ms pipeline={}ms",
        prepare_ms, delete_ms, insert_ms, pipeline_ms
    ));
    lay::action_log::record_timing_profile(
        label,
        if fast_output {
            "uinput-fast-replace"
        } else {
            "uinput-replace"
        },
        &[
            ("prepare", prepare_ms),
            ("delete", delete_ms),
            ("insert", insert_ms),
            ("pipeline", pipeline_ms),
        ],
    );
    Ok(outcome)
}

fn insert_prepared_text_for_replacement_plan(
    dev: &mut VirtualDevice,
    plan: &TextReplacement,
    replacement: &str,
    prepared: &PreparedTextInsert,
    label: &str,
    fast_output: bool,
) -> Result<TextInsertOutcome, String> {
    let mut current_layout = prepared.preflight_final_layout_is_ru;
    for run in &prepared.runs {
        if current_layout != Some(run.target_is_ru) {
            switch_to_target_layout(run.target_is_ru)?;
            current_layout = Some(run.target_is_ru);
        }
        if fast_output {
            replay_text_insert_keycodes_fast_after_modifier_cleanup(dev, &run.events)
                .map_err(|e| e.to_string())?;
        } else {
            replay_text_insert_keycodes(dev, &run.events).map_err(|e| e.to_string())?;
        }
    }
    if let Err(e) = emit_key_taps_fast(dev, KeyCode::KEY_RIGHT, plan.move_right) {
        return Err(format!("cursor restore failed: {e}"));
    }
    log(&format!("  {label} insert backend: prepared uinput replay"));
    let actual_layout_is_ru = prepared
        .runs
        .last()
        .map(|run| run.target_is_ru)
        .unwrap_or(prepared.insert_layout_is_ru);
    let layout_is_ru =
        layout_after_replacement_plan(plan, replacement, prepared.insert_layout_is_ru);
    Ok(TextInsertOutcome {
        layout_is_ru,
        layout_already_set: actual_layout_is_ru == layout_is_ru,
    })
}

pub(crate) fn switch_or_restore_layout_after_text_edit(
    auto_switch_layout: bool,
    target_layout: bool,
    original_layout: Option<bool>,
    label: &str,
    target_layout_already_set: bool,
) {
    if auto_switch_layout {
        if target_layout_already_set {
            log(&format!("  {label} layout already set by text insert"));
        } else {
            match switch_to_target_layout(target_layout) {
                Ok(layout_id) => log(&format!("  {label} layout -> {layout_id}")),
                Err(e) => log(&format!("⚠ {label} layout switch failed: {e}")),
            }
        }
    } else if let Some(layout_is_ru) = original_layout {
        match switch_to_target_layout(layout_is_ru) {
            Ok(layout_id) => log(&format!("  {label} layout restored -> {layout_id}")),
            Err(e) => log(&format!("⚠ {label} layout restore failed: {e}")),
        }
    }
}

fn layout_after_replacement_plan(
    plan: &TextReplacement,
    replacement: &str,
    insert_layout_is_ru: bool,
) -> bool {
    if plan.move_right == 0 {
        insert_layout_is_ru
    } else if replacement
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace)
    {
        continuation_layout_after_completed_tail(replacement, insert_layout_is_ru)
    } else {
        preferred_layout_for_text(replacement, insert_layout_is_ru)
    }
}

fn continuation_layout_after_completed_tail(text: &str, fallback_is_ru: bool) -> bool {
    text.split_whitespace()
        .rev()
        .nth(1)
        .map(|context| preferred_layout_for_text(context, fallback_is_ru))
        .unwrap_or_else(|| preferred_layout_for_text(text, fallback_is_ru))
}

#[cfg(test)]
#[path = "replacement_tests.rs"]
mod tests;
