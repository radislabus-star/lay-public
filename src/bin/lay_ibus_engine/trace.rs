use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const TRACE_CONFIG_REFRESH: Duration = Duration::from_millis(250);

#[derive(Debug)]
struct TraceConfigCache {
    enabled: bool,
    checked_at: Instant,
}

static TRACE_CONFIG: Mutex<Option<TraceConfigCache>> = Mutex::new(None);

pub(crate) fn record(line: impl AsRef<str>) {
    if !debug_enabled_cached() {
        return;
    }
    let path = trace_path();
    let _ = lay::private_file::append_private_text(&path, &format!("{}\n", line.as_ref()));
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
    if !debug_enabled_cached() {
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
    if !debug_enabled_cached() {
        return;
    }
    let text = text.map(json_string).unwrap_or_else(|| "null".to_string());
    write_record(format!(
        r#"{{"kind":"ibus_preedit","stage":"{stage}","visible":{visible},"chars":{chars},"cursor_pos":{cursor_pos},"text":{text}}}"#
    ));
}

pub(crate) fn record_cursor_location(x: i32, y: i32, w: i32, h: i32) {
    if !debug_enabled_cached() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_cursor","x":{x},"y":{y},"w":{w},"h":{h}}}"#
    ));
}

pub(crate) fn record_ime_commit(decision_us: u64, clear_us: u64, output_us: u64, elapsed_us: u64) {
    if !debug_enabled_cached() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_commit_timing","decision_us":{decision_us},"clear_us":{clear_us},"output_us":{output_us},"elapsed_us":{elapsed_us}}}"#
    ));
}

pub(crate) fn record_layout_sync(target_is_ru: bool, engine: &str, ok: bool) {
    if !debug_enabled_cached() {
        return;
    }
    write_record(format!(
        r#"{{"kind":"ibus_layout_sync","target_is_ru":{target_is_ru},"engine":"{engine}","ok":{ok}}}"#
    ));
}

fn write_record(line: impl AsRef<str>) {
    let path = trace_path();
    let _ = lay::private_file::append_private_text(&path, &format!("{}\n", line.as_ref()));
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
