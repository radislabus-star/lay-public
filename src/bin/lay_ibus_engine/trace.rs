use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use lay::manual_toggle::{ManualTogglePlan, ManualToggleRoute, VisibleTailSource};

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

pub(crate) fn record_ime_commit(decision_us: u64, clear_us: u64, output_us: u64, elapsed_us: u64) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_commit_timing","decision_us":{decision_us},"clear_us":{clear_us},"output_us":{output_us},"elapsed_us":{elapsed_us}}}"#
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

pub(crate) fn record_manual_toggle_plan(plan: &ManualTogglePlan) {
    if !enabled() {
        return;
    }
    let route = manual_toggle_route(plan.route);
    let source = visible_tail_source(plan.edit.source);
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
    let source = visible_tail_source(source);
    let text = json_string(text);
    write_record(format!(
        r#"{{"kind":"ibus_committed_tail_replace","source":"{source}","output_route":"{output_route}","backspaces":{backspaces},"text":{text}}}"#
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
    let source = visible_tail_source(source);
    write_record(format!(
        r#"{{"kind":"ibus_committed_tail_replace_timing","source":"{source}","output_route":"{output_route}","clear_us":{clear_us},"delete_us":{delete_us},"commit_us":{commit_us},"state_us":{state_us},"total_us":{total_us}}}"#
    ));
}

pub(crate) fn record_precognition_timing(
    total_us: u64,
    ascii_us: u64,
    ru_us: u64,
    semantic_us: u64,
    candidates: usize,
) {
    if !enabled() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_precognition_timing","total_us":{total_us},"ascii_us":{ascii_us},"ru_us":{ru_us},"semantic_us":{semantic_us},"candidates":{candidates}}}"#
    ));
}

fn visible_tail_source(source: VisibleTailSource) -> &'static str {
    match source {
        VisibleTailSource::DaemonWordBuffer => "daemon_word_buffer",
        VisibleTailSource::ImeActiveComposition => "ime_active_composition",
        VisibleTailSource::ImeCommittedTail => "ime_committed_tail",
    }
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
