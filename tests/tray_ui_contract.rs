use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const EXTENSION: &str = "extension/lay@radislabus-star.github.io";

#[test]
fn tray_has_one_input_mode_owner_for_live_suggestions() {
    let tray = read("lay-impl.js");
    assert!(tray.contains("Режим ввода:"), "tray must expose input mode");
    assert!(
        tray.contains("this._cfg.text_backend = id;")
            && tray.contains("this._cfg.nanda_precognition = id !== 'uinput';")
            && tray.contains("applyInputChannel(id);"),
        "tray input mode must synchronize text_backend, nanda_precognition, and runtime channel"
    );

    for file in ["settings.js", "prefs.js"] {
        let source = read(file);
        assert!(
            source.contains("Режим ввода")
                && source.contains("if (key === 'text_backend')")
                && source.contains("nanda_precognition = id !== 'uinput'")
                && source.contains("applyInputChannel(id)"),
            "{file} must keep input mode as the preferences owner for live suggestions"
        );
    }
}

#[test]
fn debug_log_switch_is_not_a_hidden_precognition_toggle() {
    let tray = read("lay-impl.js");
    let tray_debug = nearby(&tray, "_debugLogSwitchItem() {", 520);
    assert!(tray_debug.contains("debug_action_log"));
    assert!(tray_debug.contains("nanda_trace"));
    assert!(tray_debug.contains("nanda_trace_text"));
    assert!(
        !tray_debug.contains("nanda_precognition"),
        "tray debug switch must not toggle live suggestions"
    );

    for (file, marker) in [
        ("settings.js", "debugLogsRow(label)"),
        ("prefs.js", "_debugLogsRow(label)"),
    ] {
        let source = read(file);
        let debug = nearby(&source, marker, 520);
        assert!(debug.contains("debug_action_log"));
        assert!(debug.contains("nanda_trace"));
        assert!(debug.contains("nanda_trace_text"));
        assert!(
            !debug.contains("nanda_precognition"),
            "{file} debug switch must not toggle live suggestions"
        );
    }
}

#[test]
fn stale_gray_suggestion_ui_stays_deleted() {
    for file in ["lay-impl.js", "settings.js", "prefs.js", "tray_support.js"] {
        let source = read(file);
        assert!(
            !source.contains("Серые подсказки")
                && !source.contains("серые подсказки")
                && !source.contains("gray suggestions")
                && !source.contains("grey suggestions"),
            "{file} must not revive the old separate gray-suggestions switch"
        );
    }
}

#[test]
fn debug_log_label_is_action_journal_everywhere() {
    for file in ["lay-impl.js", "settings.js", "prefs.js"] {
        let source = read(file);
        assert!(
            source.contains("Журнал отладки действий"),
            "{file} must use the explicit action debug journal label"
        );
        assert!(
            !source.contains("Журнал отладки lay"),
            "{file} must not use the old broad debug-log label"
        );
    }
}

fn read(file: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(EXTENSION).join(file)).expect("extension source")
}

fn nearby(source: &str, marker: &str, chars: usize) -> String {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker {marker}"));
    source[start..].chars().take(chars).collect()
}
