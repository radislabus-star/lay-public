use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use lay::manual_toggle::{ManualTogglePlan, ManualToggleRoute};
use lay::text_edit::VisibleTailSource;

use super::engine::InputFrameIdentity;
use super::preedit::PrecognitionMaterializationTiming;

const TRACE_CONFIG_REFRESH: Duration = Duration::from_millis(250);

#[derive(Debug)]
struct TraceConfigCache {
    enabled: bool,
    checked_at: Instant,
}

static TRACE_CONFIG: Mutex<Option<TraceConfigCache>> = Mutex::new(None);

pub(crate) fn record(line: impl AsRef<str>) {
    if !enabled() {
        return;
    }
    let path = trace_path();
    lay::debug_log::append_private_line(path, line.as_ref().to_string());
}

pub(crate) fn record_key(
    stage: &str,
    keyval: u32,
    keycode: u32,
    handled: bool,
    decoded: Option<char>,
    tail_chars: usize,
    preedit_chars: usize,
) {
    if !enabled() {
        return;
    }
    let decoded = decoded
        .map(|ch| json_string(&ch.to_string()))
        .unwrap_or_else(|| "null".to_string());
    write_record(format!(
        r#"{{"kind":"ibus_key","stage":"{stage}","keyval":{keyval},"keycode":{keycode},"handled":{handled},"decoded":{decoded},"tail_chars":{tail_chars},"preedit_chars":{preedit_chars}}}"#
    ));
}

pub(crate) fn record_preedit(
    stage: &str,
    visible: bool,
    chars: usize,
    cursor_pos: u32,
    text: Option<&str>,
) {
    if !enabled() {
        return;
    }
    let text = text.map(json_string).unwrap_or_else(|| "null".to_string());
    write_record(format!(
        r#"{{"kind":"ibus_preedit","stage":"{stage}","visible":{visible},"chars":{chars},"cursor_pos":{cursor_pos},"text":{text}}}"#
    ));
}

pub(crate) fn record_cursor_location(x: i32, y: i32, w: i32, h: i32) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_cursor","x":{x},"y":{y},"w":{w},"h":{h}}}"#
    ));
}

pub(crate) fn record_capabilities(caps: u32, surrounding_text_supported: bool) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_capabilities","caps":{caps},"surrounding_text_supported":{surrounding_text_supported}}}"#
    ));
}

pub(crate) fn record_surrounding_text_snapshot(
    text_chars: usize,
    cursor_pos: u32,
    anchor_pos: u32,
    auto_undo_retry: &str,
) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_surrounding_text","text_chars":{text_chars},"cursor_pos":{cursor_pos},"anchor_pos":{anchor_pos},"auto_undo_retry":"{auto_undo_retry}"}}"#
    ));
}

pub(crate) fn record_auto_undo_retry(status: &str) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_auto_undo_retry","status":"{status}"}}"#
    ));
}

#[expect(clippy::too_many_arguments, reason = "trace fields remain explicit")]
pub(crate) fn record_auto_undo_lifecycle(
    stage: &str,
    reason: &str,
    engine_path: &str,
    active_path_matches: bool,
    pending: bool,
    retry: bool,
    engine_tail_chars: usize,
    pending_tail_chars: usize,
    replacement_chars: usize,
    snapshot_chars: usize,
) {
    if !enabled() {
        return;
    }
    let engine_path = json_string(engine_path);
    write_record(format!(
        r#"{{"kind":"ibus_auto_undo_lifecycle","stage":"{stage}","reason":"{reason}","engine_path":{engine_path},"active_path_matches":{active_path_matches},"pending":{pending},"retry":{retry},"engine_tail_chars":{engine_tail_chars},"pending_tail_chars":{pending_tail_chars},"replacement_chars":{replacement_chars},"snapshot_chars":{snapshot_chars}}}"#
    ));
}

pub(crate) fn record_ime_commit(decision_us: u64, clear_us: u64, output_us: u64, elapsed_us: u64) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_commit_timing","decision_us":{decision_us},"clear_us":{clear_us},"output_us":{output_us},"elapsed_us":{elapsed_us}}}"#
    ));
}

pub(crate) fn record_space_autocorrect_timing(
    status: &str,
    decision_us: u128,
    replacement_us: u128,
    total_us: u128,
) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_space_autocorrect_timing","status":"{status}","decision_us":{decision_us},"replacement_us":{replacement_us},"total_us":{total_us}}}"#
    ));
}

pub(crate) fn record_space_key_timing(
    route: &str,
    setup_us: u128,
    autocorrect_us: u128,
    commit_us: u128,
    total_us: u128,
) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_space_key_timing","route":"{route}","setup_us":{setup_us},"autocorrect_us":{autocorrect_us},"commit_us":{commit_us},"total_us":{total_us}}}"#
    ));
}

pub(crate) fn record_printable_key_timing(route: &str, total_us: u128) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_printable_key_timing","route":"{route}","total_us":{total_us}}}"#
    ));
}

pub(crate) fn record_layout_sync(target_is_ru: bool, engine: &str, ok: bool) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_layout_sync","target_is_ru":{target_is_ru},"engine":"{engine}","ok":{ok}}}"#
    ));
}

pub(crate) fn record_layout_sync_requested(target_is_ru: bool, engine: &str) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_layout_sync_requested","target_is_ru":{target_is_ru},"engine":"{engine}"}}"#
    ));
}

pub(crate) fn record_manual_toggle_plan(plan: &ManualTogglePlan) {
    if !enabled() {
        return;
    }
    let route = manual_toggle_route(plan.route);
    let source = plan.edit.source.source_id();
    let original_token = json_string(&plan.edit.original_token);
    let insert_text = json_string(&plan.edit.insert_text);
    write_record(format!(
        r#"{{"kind":"ibus_manual_toggle_plan","route":"{route}","source":"{source}","original_token":{original_token},"delete_chars":{},"insert_text":{insert_text},"target_layout_is_ru":{},"suppress_next_autocorrect":{}}}"#,
        plan.edit.delete_chars, plan.edit.target_layout_is_ru, plan.suppress_next_autocorrect
    ));
}

pub(crate) fn record_committed_tail_replace(
    source: VisibleTailSource,
    output_route: &str,
    backspaces: u32,
    text: &str,
) {
    if !enabled() {
        return;
    }
    let source = source.source_id();
    let text = json_string(text);
    write_record(format!(
        r#"{{"kind":"ibus_committed_tail_replace","source":"{source}","output_route":"{output_route}","backspaces":{backspaces},"text":{text}}}"#
    ));
}

pub(crate) fn record_committed_tail_replace_guard(
    source: VisibleTailSource,
    reason: &str,
    backspaces: u32,
    expected: &str,
    actual: &str,
) {
    if !enabled() {
        return;
    }
    let source = source.source_id();
    let expected = json_string(expected);
    let actual = json_string(actual);
    write_record(format!(
        r#"{{"kind":"ibus_committed_tail_replace_guard","source":"{source}","reason":"{reason}","backspaces":{backspaces},"expected":{expected},"actual":{actual}}}"#
    ));
}

pub(crate) fn record_committed_tail_replace_timing(
    source: VisibleTailSource,
    output_route: &str,
    clear_us: u128,
    delete_us: u128,
    commit_us: u128,
    state_us: u128,
    total_us: u128,
) {
    if !enabled() {
        return;
    }
    let source = source.source_id();
    write_record(format!(
        r#"{{"kind":"ibus_committed_tail_replace_timing","source":"{source}","output_route":"{output_route}","clear_us":{clear_us},"delete_us":{delete_us},"commit_us":{commit_us},"state_us":{state_us},"total_us":{total_us}}}"#
    ));
}

pub(crate) fn record_precognition_timing(
    outcome: &str,
    worker_generation: u64,
    identity: &InputFrameIdentity,
    timing: &PrecognitionMaterializationTiming,
    candidates: usize,
    token: Option<&str>,
    top: Option<&str>,
) {
    if !enabled() {
        return;
    }
    let token = token.map(json_string).unwrap_or_else(|| "null".to_string());
    let top = top.map(json_string).unwrap_or_else(|| "null".to_string());
    write_record(format!(
        r#"{{"kind":"ibus_precognition_timing","total_us":{},"ascii_us":0,"ru_us":{},"semantic_us":{},"ru_cache_hit":{},"ru_l2_material_us":{},"ru_l3_context_us":{},"ru_decision_us":{},"candidates":{candidates},"token":{token},"top":{top}}}"#,
        timing.total_us,
        timing.word_us,
        timing.semantic_us,
        timing.word.cache_hit,
        timing.word.l2_material_us,
        timing.word.l3_context_us,
        timing.word.decision_us,
    ));
    write_record(token_field_route_line(TokenFieldRouteRecord {
        projection: "display",
        outcome,
        worker_generation,
        identity,
        field_producer_count: timing.word.field_producer_count,
        field_cache_disposition: timing
            .word
            .field_cache_disposition
            .unwrap_or("not_requested"),
        field_generation: timing.word.field_generation,
        l11_us: timing.word.l11_us,
        productive_v90_us: timing.word.productive_v90_us,
        display_l3_us: timing.word.l3_context_us,
        semantic_l3_us: timing.semantic_us,
        correction_l3_us: 0,
        space_lookup_wait_us: 0,
        decision_total_us: timing.word.decision_us,
        correction_total_us: timing.total_us,
        candidates,
    }));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpaceCorrectionLeaseOutcome {
    Ready,
    NotReady,
    Stale,
    Unauthorized,
    Applied,
}

impl SpaceCorrectionLeaseOutcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NotReady => "not_ready",
            Self::Stale => "stale",
            Self::Unauthorized => "unauthorized",
            Self::Applied => "applied",
        }
    }
}

pub(crate) fn record_space_correction_lease_outcome(
    outcome: SpaceCorrectionLeaseOutcome,
    worker_generation: u64,
    identity: &InputFrameIdentity,
    lookup_wait_us: u128,
) {
    if !enabled() {
        return;
    }
    write_record(space_lease_line(
        outcome,
        worker_generation,
        identity,
        lookup_wait_us,
    ));
}

pub(crate) fn record_correction_projection_timing(
    outcome: &str,
    worker_generation: u64,
    identity: &InputFrameIdentity,
    telemetry: lay::ime_correction::ActiveCompositionAutocorrectTelemetry,
) {
    if !enabled() {
        return;
    }
    write_record(token_field_route_line(TokenFieldRouteRecord {
        projection: "correction",
        outcome,
        worker_generation,
        identity,
        field_producer_count: telemetry.field_producer_count,
        field_cache_disposition: telemetry.field_cache_disposition,
        field_generation: telemetry.field_generation,
        l11_us: telemetry.l11_us,
        productive_v90_us: telemetry.productive_v90_us,
        display_l3_us: 0,
        semantic_l3_us: 0,
        correction_l3_us: telemetry.correction_l3_us,
        space_lookup_wait_us: 0,
        decision_total_us: telemetry.decision_total_us,
        correction_total_us: telemetry.total_us,
        candidates: 0,
    }));
}

struct TokenFieldRouteRecord<'a> {
    projection: &'static str,
    outcome: &'a str,
    worker_generation: u64,
    identity: &'a InputFrameIdentity,
    field_producer_count: u64,
    field_cache_disposition: &'static str,
    field_generation: u64,
    l11_us: u64,
    productive_v90_us: u64,
    display_l3_us: u64,
    semantic_l3_us: u64,
    correction_l3_us: u64,
    space_lookup_wait_us: u128,
    decision_total_us: u64,
    correction_total_us: u64,
    candidates: usize,
}

fn token_field_route_line(record: TokenFieldRouteRecord<'_>) -> String {
    let projection = json_string(record.projection);
    let outcome = json_string(record.outcome);
    let path = json_string(&record.identity.path);
    let cache_disposition = json_string(record.field_cache_disposition);
    format!(
        r#"{{"kind":"ibus_token_field_route","projection":{projection},"outcome":{outcome},"worker_generation":{},"tail_epoch":{},"engine_path":{path},"field_producer_count":{},"field_cache_disposition":{cache_disposition},"field_generation":{},"l11_us":{},"productive_v90_us":{},"display_l3_us":{},"semantic_l3_us":{},"correction_l3_us":{},"space_lookup_wait_us":{},"decision_total_us":{},"correction_total_us":{},"candidates":{}}}"#,
        record.worker_generation,
        record.identity.tail_epoch,
        record.field_producer_count,
        record.field_generation,
        record.l11_us,
        record.productive_v90_us,
        record.display_l3_us,
        record.semantic_l3_us,
        record.correction_l3_us,
        record.space_lookup_wait_us,
        record.decision_total_us,
        record.correction_total_us,
        record.candidates,
    )
}

fn space_lease_line(
    outcome: SpaceCorrectionLeaseOutcome,
    worker_generation: u64,
    identity: &InputFrameIdentity,
    lookup_wait_us: u128,
) -> String {
    let outcome = json_string(outcome.as_str());
    let path = json_string(&identity.path);
    format!(
        r#"{{"kind":"ibus_space_correction_lease","outcome":{outcome},"worker_generation":{worker_generation},"tail_epoch":{},"engine_path":{path},"space_lookup_wait_us":{lookup_wait_us}}}"#,
        identity.tail_epoch,
    )
}

pub(crate) fn record_completion_accept(source: &str, suffix_chars: usize, with_space: bool) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_completion_accept","source":"{source}","suffix_chars":{suffix_chars},"with_space":{with_space}}}"#
    ));
}

fn manual_toggle_route(route: ManualToggleRoute) -> &'static str {
    match route {
        ManualToggleRoute::Daemon => "daemon",
        ManualToggleRoute::ImeActiveComposition => "ime_active_composition",
        ManualToggleRoute::ImeCommittedTail => "ime_committed_tail",
    }
}

pub(crate) fn enabled() -> bool {
    debug_enabled_cached()
}

fn write_record(line: impl AsRef<str>) {
    let path = trace_path();
    lay::debug_log::append_private_line(path, line.as_ref().to_string());
}

fn debug_enabled_cached() -> bool {
    let now = Instant::now();
    let mut cache = TRACE_CONFIG.lock().expect("lay ime trace config poisoned");
    if let Some(state) = cache.as_ref() {
        if now.duration_since(state.checked_at) < TRACE_CONFIG_REFRESH {
            return state.enabled;
        }
    }
    let enabled = lay::config::LayConfig::load().debug_action_log;
    *cache = Some(TraceConfigCache {
        enabled,
        checked_at: now,
    });
    enabled
}

fn trace_path() -> PathBuf {
    if let Some(path) = std::env::var_os("LAY_IBUS_TRACE_PATH") {
        return PathBuf::from(path);
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".local/share/lay/ibus_engine_debug.jsonl")
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use lay::config::LayConfig;
    use serde_json::Value;

    use super::*;

    const TOKEN_FIELD_REQUIRED_FIELDS: &[&str] = &[
        "kind",
        "projection",
        "outcome",
        "worker_generation",
        "tail_epoch",
        "engine_path",
        "field_producer_count",
        "field_cache_disposition",
        "field_generation",
        "l11_us",
        "productive_v90_us",
        "display_l3_us",
        "semantic_l3_us",
        "correction_l3_us",
        "space_lookup_wait_us",
        "decision_total_us",
        "correction_total_us",
        "candidates",
    ];

    fn identity() -> InputFrameIdentity {
        InputFrameIdentity::new(
            "/engine/test".to_string(),
            Some("focus-test".to_string()),
            41,
            "context token".to_string(),
            "context ".to_string(),
            "token".to_string(),
            true,
            true,
            &LayConfig::default(),
        )
    }

    fn route_line(projection: &'static str, producer_count: u64) -> String {
        let identity = identity();
        token_field_route_line(TokenFieldRouteRecord {
            projection,
            outcome: "prepared",
            worker_generation: 7,
            identity: &identity,
            field_producer_count: producer_count,
            field_cache_disposition: if producer_count == 0 {
                "hit"
            } else {
                "producer"
            },
            field_generation: 11,
            l11_us: 101,
            productive_v90_us: 202,
            display_l3_us: u64::from(projection == "display") * 303,
            semantic_l3_us: u64::from(projection == "display") * 404,
            correction_l3_us: u64::from(projection == "correction") * 505,
            space_lookup_wait_us: 0,
            decision_total_us: 606,
            correction_total_us: 707,
            candidates: usize::from(projection == "display") * 8,
        })
    }

    fn parse(line: &str) -> Value {
        serde_json::from_str(line).expect("trace line must be valid JSON")
    }

    #[test]
    fn token_field_projection_trace_has_every_required_field() {
        for projection in ["display", "correction"] {
            let value = parse(&route_line(projection, u64::from(projection == "display")));
            for field in TOKEN_FIELD_REQUIRED_FIELDS {
                assert!(
                    value.get(field).is_some(),
                    "{projection} projection is missing {field}"
                );
            }
            assert_eq!(
                value.get("kind").and_then(Value::as_str),
                Some("ibus_token_field_route")
            );
            assert_eq!(
                value.get("projection").and_then(Value::as_str),
                Some(projection)
            );
        }
    }

    #[test]
    fn one_frame_has_one_projection_per_role_and_at_most_one_field_producer() {
        let values = [route_line("display", 1), route_line("correction", 0)]
            .into_iter()
            .map(|line| parse(&line))
            .collect::<Vec<_>>();
        let frame = (
            values[0].get("engine_path").and_then(Value::as_str),
            values[0].get("tail_epoch").and_then(Value::as_u64),
        );
        let mut projection_counts = BTreeMap::new();
        let mut producer_count = 0;

        for value in &values {
            assert_eq!(
                (
                    value.get("engine_path").and_then(Value::as_str),
                    value.get("tail_epoch").and_then(Value::as_u64),
                ),
                frame
            );
            let projection = value
                .get("projection")
                .and_then(Value::as_str)
                .expect("projection");
            *projection_counts.entry(projection).or_insert(0_u64) += 1;
            producer_count += value
                .get("field_producer_count")
                .and_then(Value::as_u64)
                .expect("field producer count");
        }

        assert_eq!(projection_counts.get("display"), Some(&1));
        assert_eq!(projection_counts.get("correction"), Some(&1));
        assert!(producer_count <= 1);
    }

    #[test]
    fn space_lease_trace_uses_the_closed_outcome_vocabulary() {
        let outcomes = [
            SpaceCorrectionLeaseOutcome::Ready,
            SpaceCorrectionLeaseOutcome::NotReady,
            SpaceCorrectionLeaseOutcome::Stale,
            SpaceCorrectionLeaseOutcome::Unauthorized,
            SpaceCorrectionLeaseOutcome::Applied,
        ];
        let encoded = outcomes
            .into_iter()
            .map(|outcome| {
                let value = parse(&space_lease_line(outcome, 7, &identity(), 123));
                assert_eq!(
                    value.get("kind").and_then(Value::as_str),
                    Some("ibus_space_correction_lease")
                );
                for field in [
                    "outcome",
                    "worker_generation",
                    "tail_epoch",
                    "engine_path",
                    "space_lookup_wait_us",
                ] {
                    assert!(value.get(field).is_some(), "lease trace is missing {field}");
                }
                value
                    .get("outcome")
                    .and_then(Value::as_str)
                    .expect("lease outcome")
                    .to_string()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            encoded,
            ["ready", "not_ready", "stale", "unauthorized", "applied"]
        );
    }
}
