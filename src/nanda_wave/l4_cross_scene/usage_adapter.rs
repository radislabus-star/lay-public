use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

use super::compiler::{compile_observations, CrossSceneCompileConfig};
use super::encoder::{
    candidate_relation_id, context_signal_from_text, keep_relation_id, relation_class_from_context,
};
use super::format::write_package;
use super::model::{L4CrossSceneL2Signal, L4CrossSceneObservation, L4CrossSceneProfileKey};
use crate::transition_relation::{TransitionOperatorKind, TransitionRelationAtoms};
use crate::typing_memory::{
    LayoutProjectionDirection, LayoutProjectionScope, TypingMemoryOutcome, TypingTransitionIdentity,
};

pub(crate) fn compile_usage_events_path(
    input: &Path,
    output: &Path,
    config: CrossSceneCompileConfig,
) -> io::Result<super::model::CrossSceneCompileReport> {
    compile_usage_events_with_corrections_path(input, None, output, config)
}

pub(crate) fn compile_usage_events_with_corrections_path(
    input: &Path,
    corrections: Option<&Path>,
    output: &Path,
    config: CrossSceneCompileConfig,
) -> io::Result<super::model::CrossSceneCompileReport> {
    let live_text = fs::read_to_string(input)?;
    let (backfill_text, backfilled_revert_receipts) = corrections
        .map(fs::read_to_string)
        .transpose()?
        .map(|text| crate::nanda_wave::usage_prior::exact_reverted_system_apply_usage_jsonl(&text))
        .unwrap_or_default();
    let mut observations = Vec::new();
    let mut invalid_lines = 0_u32;
    let mut ignored_lines = 0_u32;
    collect_observations(
        &live_text,
        &mut observations,
        &mut ignored_lines,
        &mut invalid_lines,
    );
    collect_observations(
        &backfill_text,
        &mut observations,
        &mut ignored_lines,
        &mut invalid_lines,
    );
    let live_source_observations = nonempty_line_count(&live_text);
    let backfilled_revert_observations = nonempty_line_count(&backfill_text);
    let (package, mut report) = compile_observations(&observations, config);
    report.live_source_observations = live_source_observations;
    report.backfilled_revert_receipts = backfilled_revert_receipts;
    report.backfilled_revert_observations = backfilled_revert_observations;
    report.source_observations =
        live_source_observations.saturating_add(backfilled_revert_observations);
    report.ignored_observations = ignored_lines;
    report.invalid_observations = invalid_lines;
    write_package(output, &package)?;
    report.logical_center_bytes = fs::metadata(output)?.len();
    Ok(report)
}

fn collect_observations(
    text: &str,
    observations: &mut Vec<L4CrossSceneObservation>,
    ignored_lines: &mut u32,
    invalid_lines: &mut u32,
) {
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str::<RawUsageEvent>(line) {
            Ok(event) => match event.into_observation() {
                Ok(Some(observation)) => observations.push(observation),
                Ok(None) => *ignored_lines = ignored_lines.saturating_add(1),
                Err(()) => *invalid_lines = invalid_lines.saturating_add(1),
            },
            Err(_) => *invalid_lines = invalid_lines.saturating_add(1),
        }
    }
}

fn nonempty_line_count(text: &str) -> u32 {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .count()
        .min(u32::MAX as usize) as u32
}

#[derive(Debug, Default, Deserialize)]
struct RawUsageEvent {
    #[serde(default)]
    schema: Option<u8>,
    #[serde(default)]
    episode_id: Option<String>,
    #[serde(default)]
    context: Vec<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    operation: Option<String>,
    #[serde(default)]
    operator: Option<String>,
    #[serde(default)]
    operator_code: Option<u8>,
    #[serde(default)]
    layout_direction: Option<String>,
    #[serde(default)]
    layout_direction_code: Option<u8>,
    #[serde(default)]
    layout_scope: Option<String>,
    #[serde(default)]
    layout_scope_code: Option<u8>,
    #[serde(default)]
    outcome: Option<String>,
    #[serde(default)]
    outcome_code: Option<u8>,
}

impl RawUsageEvent {
    fn into_observation(self) -> Result<Option<L4CrossSceneObservation>, ()> {
        let outcome = match (self.outcome_code, self.outcome.as_deref()) {
            (None, None) => return Ok(None),
            _ => decode_required(
                self.outcome_code,
                self.outcome.as_deref(),
                TypingMemoryOutcome::from_code,
                TypingMemoryOutcome::from_str,
            )
            .ok_or(())?,
        };
        if outcome == TypingMemoryOutcome::Censored {
            return Ok(None);
        }
        let (Some(from), Some(to)) = (self.from, self.to) else {
            return Ok(None);
        };
        let from_text = from.trim().to_string();
        let to_text = to.trim().to_string();
        if from_text.is_empty() || to_text.is_empty() {
            return Err(());
        }
        let inferred = TypingTransitionIdentity::observed(
            &from_text,
            &to_text,
            self.operation.as_deref().unwrap_or("replacement"),
        );
        let operator = match (self.operator_code, self.operator.as_deref()) {
            (None, None) => inferred.operator,
            _ => decode_required(
                self.operator_code,
                self.operator.as_deref(),
                TransitionOperatorKind::from_code,
                TransitionOperatorKind::from_str,
            )
            .ok_or(())?,
        };
        let direction = decode_optional(
            self.layout_direction_code,
            self.layout_direction.as_deref(),
            LayoutProjectionDirection::from_code,
            LayoutProjectionDirection::from_str,
        )
        .ok_or(())?;
        let scope = decode_optional(
            self.layout_scope_code,
            self.layout_scope.as_deref(),
            LayoutProjectionScope::from_code,
            LayoutProjectionScope::from_str,
        )
        .ok_or(())?;
        if self.schema == Some(2)
            && (self.operator_code.is_none()
                || self.outcome_code.is_none()
                || (operator == TransitionOperatorKind::LayoutProjection
                    && (self.layout_direction_code.is_none() || self.layout_scope_code.is_none())))
        {
            return Err(());
        }
        let direction = direction.or(inferred.layout_direction);
        let scope = scope.or(inferred.layout_scope);
        let profile = L4CrossSceneProfileKey::new(operator, direction, scope);
        let relation = TransitionRelationAtoms::for_operator(&from_text, &to_text, operator);
        let context = if self.context.is_empty() {
            crate::typing_memory::transition_context_words(&from_text, &to_text)
        } else {
            self.context
        };
        let episode = self.episode_id;
        Ok(Some(L4CrossSceneObservation {
            receipt_id: episode
                .as_deref()
                .map(|value| {
                    crate::nanda_wave::phase_field::stable_hash64(
                        value.as_bytes(),
                        0x4c34_5243_5054,
                    )
                })
                .unwrap_or_default(),
            complete_chain: episode.is_some() && outcome != TypingMemoryOutcome::Censored,
            profile,
            context: context.clone(),
            from_text,
            to_text: to_text.clone(),
            relation_atoms: relation.atoms().to_vec(),
            candidate_relation_id: candidate_relation_id(relation.atoms()),
            keep_relation_id: keep_relation_id(),
            l3_relation_class: relation_class_from_context(&context, &to_text),
            context_signal: context_signal_from_text(&context, &to_text),
            l2_signal: L4CrossSceneL2Signal::Unknown,
            outcome,
        }))
    }
}

fn decode_required<T: Copy + PartialEq>(
    code: Option<u8>,
    label: Option<&str>,
    from_code: impl Fn(u8) -> Option<T>,
    from_label: impl Fn(&str) -> Option<T>,
) -> Option<T> {
    match (code.and_then(&from_code), label.and_then(&from_label)) {
        (Some(left), Some(right)) if left == right => Some(left),
        (Some(value), None) | (None, Some(value)) => Some(value),
        _ => None,
    }
}

fn decode_optional<T: Copy + PartialEq>(
    code: Option<u8>,
    label: Option<&str>,
    from_code: impl Fn(u8) -> Option<T>,
    from_label: impl Fn(&str) -> Option<T>,
) -> Option<Option<T>> {
    match (code, label) {
        (None, None) => Some(None),
        (Some(code), Some(label)) => {
            let left = from_code(code)?;
            let right = from_label(label)?;
            (left == right).then_some(Some(left))
        }
        (Some(code), None) => from_code(code).map(Some),
        (None, Some(label)) => from_label(label).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_typed_identity_fails_closed() {
        let event = serde_json::from_str::<RawUsageEvent>(
            r#"{"schema":2,"episode_id":"e1","from":"ghbdtn","to":"привет","operation":"replacement","operator":"layout_projection:en_to_ru:current_token","operator_code":6,"layout_direction":"en_to_ru","layout_direction_code":1,"layout_scope":"current_token","layout_scope_code":2,"outcome":"confirmed_positive","outcome_code":1}"#,
        )
        .unwrap();

        assert!(event.into_observation().is_err());
    }

    #[test]
    fn censored_transport_row_is_ignored_not_learned_negative() {
        let event = serde_json::from_str::<RawUsageEvent>(
            r#"{"schema":2,"operation":"replacement","operation_code":2,"operator":"layout_projection:en_to_ru:current_token","operator_code":1,"layout_direction":"en_to_ru","layout_direction_code":1,"layout_scope":"current_token","layout_scope_code":2,"outcome":"censored","outcome_code":5}"#,
        )
        .unwrap();

        assert!(matches!(event.into_observation(), Ok(None)));
    }

    #[test]
    fn exact_legacy_rollback_is_the_only_correction_backfilled_into_l4() {
        let root =
            std::env::temp_dir().join(format!("lay-l4-revert-backfill-{}", std::process::id()));
        let usage = root.join("usage.jsonl");
        let corrections = root.join("corrections.jsonl");
        let package = root.join("l4.bin");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &usage,
            r#"{"episode_id":"live-positive","from":"провекра","to":"проверка","operation":"replacement","outcome":"confirmed_positive"}
"#,
        )
        .unwrap();
        fs::write(
            &corrections,
            concat!(
                r#"{"kind":"user-correction","lay_kind":"typing-assist","lay_from":"проверрка ","lay_to":"проверка ","from":"проверка ","to":"проверрка ","user_target":"проверрка "}"#,
                "\n",
                r#"{"kind":"user-correction","lay_kind":"typing-assist","lay_from":"смотри ","lay_to":"смотрин ","from":"ин ","to":"и ","user_target":"смотри "}"#,
                "\n"
            ),
        )
        .unwrap();

        let report = compile_usage_events_with_corrections_path(
            &usage,
            Some(&corrections),
            &package,
            CrossSceneCompileConfig::default(),
        )
        .unwrap();

        assert_eq!(report.live_source_observations, 1);
        assert_eq!(report.backfilled_revert_receipts, 1);
        assert_eq!(report.backfilled_revert_observations, 1);
        assert_eq!(report.source_observations, 2);
        assert_eq!(report.joined_observations, 2);
        assert_eq!(report.positive_observations, 1);
        assert_eq!(report.reverted_observations, 1);
        assert_eq!(report.invalid_observations, 0);
        assert_eq!(report.ignored_observations, 0);
        assert!(fs::metadata(&package).unwrap().len() > 0);

        let _ = fs::remove_dir_all(root);
    }
}
