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
use super::segments::SealedSegment;
use crate::transition_relation::{TransitionOperatorKind, TransitionRelationAtoms};
use crate::typing_memory::{
    LayoutProjectionDirection, LayoutProjectionScope, TypingMemoryOperation, TypingMemoryOutcome,
    TypingTransitionIdentity,
};
use crate::typing_scene::{
    KeyboardGeometryId, LanguageId, LanguageSceneIdentity, LayoutId, SceneIdentityEvidence,
    SceneSymbol, ScriptFamily, SentenceLanguageEvidence,
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

pub(super) fn observations_from_segment(
    segment: &SealedSegment,
) -> io::Result<Vec<L4CrossSceneObservation>> {
    let mut observations = Vec::with_capacity(segment.rows.len());
    for row in &segment.rows {
        let event: RawUsageEvent = serde_json::from_value(row.clone()).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid L4 segment row: {error}"),
            )
        })?;
        if event.schema != Some(3) || event.episode_id.as_deref() != Some(&segment.episode_id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "L4 segment row identity does not match its envelope",
            ));
        }
        let observation = event
            .into_observation()
            .map_err(|()| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid typed L4 segment row")
            })?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "L4 causal segment contains a non-learning row",
                )
            })?;
        if !observation.complete_chain || observation.receipt_id == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "L4 causal segment did not produce a complete observation",
            ));
        }
        observations.push(observation);
    }
    if observations.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L4 segment contains no observations",
        ));
    }
    Ok(observations)
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
    operation_code: Option<u8>,
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
    source_language: Option<String>,
    #[serde(default)]
    source_language_id: Option<u64>,
    #[serde(default)]
    target_language: Option<String>,
    #[serde(default)]
    target_language_id: Option<u64>,
    #[serde(default)]
    source_layout: Option<String>,
    #[serde(default)]
    source_layout_id: Option<u64>,
    #[serde(default)]
    target_layout: Option<String>,
    #[serde(default)]
    target_layout_id: Option<u64>,
    #[serde(default)]
    source_script: Option<String>,
    #[serde(default)]
    source_script_code: Option<u8>,
    #[serde(default)]
    target_script: Option<String>,
    #[serde(default)]
    target_script_code: Option<u8>,
    #[serde(default)]
    keyboard_geometry: Option<String>,
    #[serde(default)]
    keyboard_geometry_id: Option<u64>,
    #[serde(default)]
    identity_evidence: Option<String>,
    #[serde(default)]
    identity_evidence_code: Option<u8>,
    #[serde(default)]
    sentence_language: Option<String>,
    #[serde(default)]
    sentence_language_id: Option<u64>,
    #[serde(default)]
    sentence_language_support_milli: Option<u16>,
    #[serde(default)]
    sentence_language_alternative_milli: Option<u16>,
    #[serde(default)]
    sentence_language_observed_tokens: Option<u8>,
    #[serde(default)]
    outcome: Option<String>,
    #[serde(default)]
    outcome_code: Option<u8>,
}

impl RawUsageEvent {
    fn into_observation(self) -> Result<Option<L4CrossSceneObservation>, ()> {
        if self.schema.is_some_and(|schema| schema > 3) {
            return Err(());
        }
        let typed_v3_scene = (self.schema == Some(3))
            .then(|| self.decode_v3_scene())
            .transpose()?;
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
        if self.schema == Some(3) {
            let operation_code = self.operation_code.ok_or(())?;
            let operation = self.operation.as_deref().ok_or(())?;
            if TypingMemoryOperation::from_legacy(operation).code() != operation_code {
                return Err(());
            }
            decode_strict_required(
                self.operator_code,
                self.operator.as_deref(),
                TransitionOperatorKind::from_code,
                TransitionOperatorKind::from_str,
            )
            .ok_or(())?;
            decode_strict_required(
                self.outcome_code,
                self.outcome.as_deref(),
                TypingMemoryOutcome::from_code,
                TypingMemoryOutcome::from_str,
            )
            .ok_or(())?;
            decode_strict_optional(
                self.layout_direction_code,
                self.layout_direction.as_deref(),
                LayoutProjectionDirection::from_code,
                LayoutProjectionDirection::from_str,
            )
            .ok_or(())?;
            decode_strict_optional(
                self.layout_scope_code,
                self.layout_scope.as_deref(),
                LayoutProjectionScope::from_code,
                LayoutProjectionScope::from_str,
            )
            .ok_or(())?;
        }
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
        let context = if self.context.is_empty() {
            crate::typing_memory::transition_context_words(&from_text, &to_text)
        } else {
            self.context
        };
        let (scene, sentence_language, mut scene_symbols) = typed_v3_scene.unwrap_or_else(|| {
            let sentence_language = SentenceLanguageEvidence::script_only(&context, &to_text);
            (
                inferred.scene,
                sentence_language,
                inferred.scene.known_symbols(),
            )
        });
        if let Some(label) = sentence_language.language.known_label() {
            scene_symbols.push(SceneSymbol::language(label).expect("known sentence language"));
        }
        scene_symbols.sort();
        scene_symbols.dedup();
        let profile = L4CrossSceneProfileKey::new(operator, direction, scope)
            .with_scene(scene, sentence_language);
        let relation = TransitionRelationAtoms::for_operator(&from_text, &to_text, operator);
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
            sentence_language,
            scene_symbols,
            outcome,
        }))
    }

    fn decode_v3_scene(
        &self,
    ) -> Result<
        (
            LanguageSceneIdentity,
            SentenceLanguageEvidence,
            Vec<SceneSymbol>,
        ),
        (),
    > {
        let source_language = decode_strict_symbol(
            self.source_language_id,
            self.source_language.as_deref(),
            LanguageId::from_label,
            LanguageId::code,
        )?;
        let target_language = decode_strict_symbol(
            self.target_language_id,
            self.target_language.as_deref(),
            LanguageId::from_label,
            LanguageId::code,
        )?;
        let source_layout = decode_strict_symbol(
            self.source_layout_id,
            self.source_layout.as_deref(),
            LayoutId::from_label,
            LayoutId::code,
        )?;
        let target_layout = decode_strict_symbol(
            self.target_layout_id,
            self.target_layout.as_deref(),
            LayoutId::from_label,
            LayoutId::code,
        )?;
        let keyboard_geometry = decode_strict_symbol(
            self.keyboard_geometry_id,
            self.keyboard_geometry.as_deref(),
            KeyboardGeometryId::from_label,
            KeyboardGeometryId::code,
        )?;
        let sentence_language = decode_strict_symbol(
            self.sentence_language_id,
            self.sentence_language.as_deref(),
            LanguageId::from_label,
            LanguageId::code,
        )?;
        let source_script = decode_strict_optional(
            self.source_script_code,
            self.source_script.as_deref(),
            ScriptFamily::from_code,
            ScriptFamily::from_str,
        )
        .ok_or(())?
        .unwrap_or_default();
        let target_script = decode_strict_optional(
            self.target_script_code,
            self.target_script.as_deref(),
            ScriptFamily::from_code,
            ScriptFamily::from_str,
        )
        .ok_or(())?
        .unwrap_or_default();
        let evidence = decode_strict_optional(
            self.identity_evidence_code,
            self.identity_evidence.as_deref(),
            SceneIdentityEvidence::from_code,
            SceneIdentityEvidence::from_str,
        )
        .ok_or(())?
        .unwrap_or_default();
        let support_milli = self.sentence_language_support_milli.ok_or(())?;
        let alternative_milli = self.sentence_language_alternative_milli.ok_or(())?;
        let observed_tokens = self.sentence_language_observed_tokens.ok_or(())?;
        if support_milli > 1_000
            || alternative_milli > 1_000
            || support_milli.saturating_add(alternative_milli) > 1_000
            || (observed_tokens == 0 && (support_milli != 0 || alternative_milli != 0))
            || (observed_tokens > 0 && support_milli == 0 && alternative_milli == 0)
        {
            return Err(());
        }
        let mut symbols = Vec::new();
        for label in [
            self.source_language.as_deref(),
            self.target_language.as_deref(),
            self.sentence_language.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            symbols.push(SceneSymbol::language(label).ok_or(())?);
        }
        for label in [self.source_layout.as_deref(), self.target_layout.as_deref()]
            .into_iter()
            .flatten()
        {
            symbols.push(SceneSymbol::layout(label).ok_or(())?);
        }
        if let Some(label) = self.keyboard_geometry.as_deref() {
            symbols.push(SceneSymbol::keyboard_geometry(label).ok_or(())?);
        }
        symbols.sort();
        symbols.dedup();
        Ok((
            LanguageSceneIdentity {
                source_language,
                target_language,
                source_layout,
                target_layout,
                source_script,
                target_script,
                keyboard_geometry,
                evidence,
            },
            SentenceLanguageEvidence {
                language: sentence_language,
                support_milli,
                alternative_milli,
                observed_tokens,
            },
            symbols,
        ))
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

fn decode_strict_required<T: Copy + PartialEq>(
    code: Option<u8>,
    label: Option<&str>,
    from_code: impl Fn(u8) -> Option<T>,
    from_label: impl Fn(&str) -> Option<T>,
) -> Option<T> {
    let (Some(code), Some(label)) = (code, label) else {
        return None;
    };
    let left = from_code(code)?;
    let right = from_label(label)?;
    (left == right).then_some(left)
}

fn decode_strict_optional<T: Copy + PartialEq>(
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
        _ => None,
    }
}

fn decode_strict_symbol<T: Copy + Default>(
    code: Option<u64>,
    label: Option<&str>,
    from_label: impl Fn(&str) -> Option<T>,
    to_code: impl Fn(T) -> u64,
) -> Result<T, ()> {
    match (code, label) {
        (None, None) => Ok(T::default()),
        (Some(code), Some(label)) => {
            let value = from_label(label).ok_or(())?;
            (to_code(value) == code).then_some(value).ok_or(())
        }
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generic_v3_event() -> serde_json::Value {
        serde_json::json!({
            "schema": 3,
            "episode_id": "v3-de-episode",
            "context": ["wir", "arbeiten"],
            "from": "hause",
            "to": "hause",
            "operation": "replacement",
            "operation_code": 2,
            "operator": "other",
            "operator_code": 255,
            "source_language": "de",
            "source_language_id": LanguageId::from_label("de").unwrap().code(),
            "target_language": "de",
            "target_language_id": LanguageId::from_label("de").unwrap().code(),
            "source_layout": "xkb:de",
            "source_layout_id": LayoutId::from_label("xkb:de").unwrap().code(),
            "target_layout": "xkb:de",
            "target_layout_id": LayoutId::from_label("xkb:de").unwrap().code(),
            "source_script": "latin",
            "source_script_code": ScriptFamily::Latin.code(),
            "target_script": "latin",
            "target_script_code": ScriptFamily::Latin.code(),
            "keyboard_geometry": "pc105",
            "keyboard_geometry_id": KeyboardGeometryId::PC105.code(),
            "identity_evidence": "package",
            "identity_evidence_code": SceneIdentityEvidence::Package.code(),
            "sentence_language": "de",
            "sentence_language_id": LanguageId::from_label("de").unwrap().code(),
            "sentence_language_support_milli": 900,
            "sentence_language_alternative_milli": 100,
            "sentence_language_observed_tokens": 2,
            "outcome": "confirmed_positive",
            "outcome_code": 1
        })
    }

    #[test]
    fn strict_v3_preserves_generic_language_and_registry_symbols() {
        let event: RawUsageEvent = serde_json::from_value(generic_v3_event()).unwrap();
        let observation = event.into_observation().unwrap().unwrap();
        let german = LanguageId::from_label("de").unwrap();

        assert_eq!(observation.profile.scene.source_language, german);
        assert_eq!(observation.profile.scene.target_language, german);
        assert_eq!(observation.profile.sentence_language, german);
        assert_eq!(observation.profile.sentence_evidence_bucket, 1);
        assert!(observation.scene_symbols.iter().any(|symbol| {
            symbol.kind == crate::typing_scene::SceneSymbolKind::Language && symbol.label == "de"
        }));
        assert!(observation.scene_symbols.iter().any(|symbol| {
            symbol.kind == crate::typing_scene::SceneSymbolKind::Layout && symbol.label == "xkb:de"
        }));
    }

    #[test]
    fn strict_v3_rejects_symbol_id_or_label_drift() {
        let mut value = generic_v3_event();
        value["target_language_id"] = serde_json::json!(LanguageId::RUSSIAN.code());
        let event: RawUsageEvent = serde_json::from_value(value).unwrap();
        assert!(event.into_observation().is_err());

        let mut missing = generic_v3_event();
        missing.as_object_mut().unwrap().remove("source_layout_id");
        let event: RawUsageEvent = serde_json::from_value(missing).unwrap();
        assert!(event.into_observation().is_err());
    }

    #[test]
    fn sealed_segment_rows_use_the_same_strict_v3_adapter() {
        let row = generic_v3_event();
        let segment = SealedSegment {
            segment_id: 1,
            episode_id: "v3-de-episode".to_string(),
            rows: vec![row],
        };

        let observations = observations_from_segment(&segment).unwrap();

        assert_eq!(observations.len(), 1);
        assert!(observations[0].complete_chain);
        assert_eq!(
            observations[0].profile.scene.target_language,
            LanguageId::from_label("de").unwrap()
        );
    }

    #[test]
    fn malformed_v3_segment_row_fails_the_whole_segment() {
        let mut row = generic_v3_event();
        row["target_language_id"] = serde_json::json!(LanguageId::RUSSIAN.code());
        let segment = SealedSegment {
            segment_id: 1,
            episode_id: "v3-de-episode".to_string(),
            rows: vec![row],
        };

        assert!(observations_from_segment(&segment).is_err());
    }

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
