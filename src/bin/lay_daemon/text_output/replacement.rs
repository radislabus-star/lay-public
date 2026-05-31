use evdev::{uinput::VirtualDevice, KeyCode};
use lay::keyboard::{preferred_layout_for_text, text_to_uinput_runs, TextInputRun};
use lay::text_edit::TextReplacement;

use super::super::{log, switch_to_target_layout};
use super::key_emit::{
    emit_backspaces_for_text_replace, emit_key_taps, emit_key_taps_fast,
    replay_text_insert_keycodes,
};

const TEXT_REPLACE_KEY_PACE_MS: u64 = 1;

#[derive(Debug, Clone)]
struct PreparedTextInsert {
    runs: Vec<TextInputRun>,
    insert_layout_is_ru: bool,
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
) -> Result<PreparedTextInsert, String> {
    let insert_layout_is_ru = preferred_layout_for_text(&plan.insert, fallback_layout_is_ru);
    let runs = text_to_uinput_runs(&plan.insert, insert_layout_is_ru)
        .ok_or_else(|| "text insert requires unsafe TypeText fallback".to_string())?;
    for run in &runs {
        switch_to_target_layout(run.target_is_ru)
            .map_err(|e| format!("layout preflight failed before destructive edit: {e}"))?;
    }
    Ok(PreparedTextInsert {
        runs,
        insert_layout_is_ru,
    })
}

fn apply_text_replacement(dev: &mut VirtualDevice, plan: &TextReplacement) -> std::io::Result<()> {
    emit_key_taps(
        dev,
        KeyCode::KEY_LEFT,
        plan.move_left,
        TEXT_REPLACE_KEY_PACE_MS,
    )?;
    emit_backspaces_for_text_replace(dev, plan.backspaces)?;
    Ok(())
}

pub(crate) fn apply_text_replacement_pipeline(
    dev: &mut VirtualDevice,
    plan: &TextReplacement,
    replacement: &str,
    fallback_layout_is_ru: bool,
    label: &str,
) -> Result<TextInsertOutcome, TextReplacementPipelineError> {
    let prepared_insert = prepare_text_insert_for_replacement_plan(plan, fallback_layout_is_ru)
        .map_err(TextReplacementPipelineError::Preflight)?;
    apply_text_replacement(dev, plan).map_err(TextReplacementPipelineError::Delete)?;
    insert_prepared_text_for_replacement_plan(dev, plan, replacement, &prepared_insert, label)
        .map_err(TextReplacementPipelineError::Insert)
}

fn insert_prepared_text_for_replacement_plan(
    dev: &mut VirtualDevice,
    plan: &TextReplacement,
    replacement: &str,
    prepared: &PreparedTextInsert,
    label: &str,
) -> Result<TextInsertOutcome, String> {
    for run in &prepared.runs {
        switch_to_target_layout(run.target_is_ru)?;
        replay_text_insert_keycodes(dev, &run.events).map_err(|e| e.to_string())?;
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
