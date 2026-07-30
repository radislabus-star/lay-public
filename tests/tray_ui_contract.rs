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
fn tray_inventory_is_compact_and_each_action_has_an_owner() {
    let tray = read("lay-impl.js");
    for label in [
        "Режим ввода:",
        "Помощь при наборе",
        "Автозамена",
        "Следовать языку исправления",
        "Настройки",
        "Диагностика",
        "Проверить обновления",
        "О Lay",
    ] {
        assert!(tray.contains(label), "missing tray action: {label}");
    }

    for route in [
        "applyInputChannel(id);",
        "restartDaemon();",
        "startDaemon();",
        "stopDaemon();",
        "startUpdate();",
        "openPreferences()",
        "openUri(APP_URL)",
    ] {
        assert!(
            tray.contains(route),
            "tray action has no runtime owner: {route}"
        );
    }
}

#[test]
fn settings_restart_the_complete_runtime_for_ime_owned_controls() {
    for file in ["settings.js", "prefs.js"] {
        let source = read(file);
        let restart = nearby(&source, "function restartDaemon()", 620);
        assert!(restart.contains("/.local/bin/lay-runtime-control"));
        assert!(restart.contains("'restart'"));
        assert!(!restart.contains("'systemctl'"));

        for control in [
            "'Следовать языку исправления', 'auto_switch_layout', true",
            "'Вес L2 кандидатов', 'nanda_l2_weight_percent', true",
            "'Вес L3 фразы', 'nanda_l3_weight_percent', true",
            "'Подсказки в [скобках]', 'ime_bracket_candidates', true",
        ] {
            assert!(source.contains(control), "{file} stale control: {control}");
        }
    }
}

#[test]
fn duplicate_force_layout_hotkeys_are_normalized_before_save() {
    for file in ["tray_support.js", "settings.js", "prefs.js"] {
        let source = read(file);
        assert!(source.contains("forceEnKey === forceRuKey"));
        assert!(source.contains("force_en_key: forceEnKey"));
    }
}

#[test]
fn nanda_button_opens_live_status_instead_of_static_help() {
    for (file, marker, chars) in [
        ("settings.js", "showNandaWindow() {", 2500),
        ("prefs.js", "_showNandaWindow() {", 2500),
    ] {
        let source = read(file);
        let window = nearby(&source, marker, chars);
        assert!(window.contains("loadNandaWaveStatus()"));
        assert!(window.contains("nandaStatusLine(status)"));
        assert!(window.contains("nandaWavePanel(status)"));
        assert!(window.contains("nandaPassportPanel(status)"));
        assert!(!window.contains("Как использовать"));
    }
}

#[test]
fn window_layout_policy_refreshes_config_on_focus_change() {
    let tray = read("lay-impl.js");
    let focus = nearby(&tray, "_onFocusWindowChanged() {", 180);
    assert!(focus.contains("normalizeConfig(loadConfig())"));
    assert!(focus.contains("_schedulePtahApply"));
}

#[test]
fn duplicate_auto_input_mode_is_legacy_only() {
    for (file, marker, chars) in [
        ("lay-impl.js", "_inputModeMenu() {", 260),
        ("settings.js", "text_backend: [", 75),
        ("prefs.js", "const BACKEND_OPTIONS = [", 120),
    ] {
        let source = read(file);
        let backend_options = nearby(&source, marker, chars);
        assert!(
            !backend_options.contains("['auto'"),
            "{file} must not expose the duplicate auto text backend"
        );
    }

    for file in ["tray_support.js", "settings.js", "prefs.js"] {
        let source = read(file);
        assert!(
            source.contains("cfg?.text_backend === 'auto' ? 'ime'"),
            "{file} must migrate the legacy auto value to IME"
        );
    }
}

#[test]
fn opening_tray_refreshes_config_and_visible_selection_state() {
    let tray = read("lay-impl.js");
    let open = nearby(&tray, "open-state-changed", 430);
    assert!(open.contains("loadConfig()"));
    assert!(open.contains("_refreshSelections()"));
    assert!(open.contains("_refreshStatus()"));

    let refresh = nearby(&tray, "_refreshSelections() {", 760);
    assert!(refresh.contains("_inputModeItem.label.text"));
    assert!(refresh.contains("setOrnament"));
    assert!(refresh.contains("setToggleState"));
}

#[test]
fn diagnostics_owns_service_logging_and_recent_actions() {
    let tray = read("lay-impl.js");
    let diagnostics = nearby(&tray, "_diagnosticsMenu() {", 900);
    assert!(diagnostics.contains("_daemonSwitchItem()"));
    assert!(diagnostics.contains("restartDaemon()"));
    assert!(diagnostics.contains("_debugLogSwitchItem()"));
    assert!(diagnostics.contains("_recentActionsMenu()"));

    let recent = read("recent_actions_menu.js");
    assert!(recent.contains("clearRecentActions()"));
    assert!(recent.contains("refreshRecentActions(indicator)"));
    assert!(recent.contains("indicator._notify"));
    assert!(
        !recent.contains("_refreshStats"),
        "recent actions must not depend on the removed about/stats surface"
    );
}

#[test]
fn tray_does_not_show_internal_research_counters_or_blink() {
    let tray = read("lay-impl.js");
    for stale in [
        "LLM ${",
        "promoted_rules",
        "_aboutStatsText",
        "_aboutConfigText",
        "_startStatusBlink",
    ] {
        assert!(!tray.contains(stale), "stale tray surface remains: {stale}");
    }
}

#[test]
fn tray_external_actions_have_installable_targets() {
    let support = read("tray_support.js");
    assert!(support.contains("/.local/bin/lay-runtime-control"));
    assert!(support.contains("PROJECT_DIR + '/update.sh'"));
    assert!(support.contains("settings.js"));

    for path in [
        "update.sh",
        "scripts/lay-runtime-control.sh",
        "extension/lay@radislabus-star.github.io/settings.js",
    ] {
        assert!(
            Path::new(ROOT).join(path).is_file(),
            "tray target is missing: {path}"
        );
    }
}

#[test]
fn runtime_restart_preserves_the_visible_layout() {
    let runtime = std::fs::read_to_string(Path::new(ROOT).join("scripts/lay-runtime-control.sh"))
        .expect("runtime control source");
    let start = nearby(&runtime, "start_ime() {", 500);
    assert!(start.contains("preferred_lay_ime"));
    assert!(start.contains("select_lay_ime \"$preferred\""));
    assert!(start.contains("select_lay_ime \"$fallback\""));
}

#[test]
fn version_refresh_reloads_model_services_without_restarting_ibus() {
    let bump = std::fs::read_to_string(Path::new(ROOT).join("scripts/bump-lay-version.sh"))
        .expect("version bump script");
    let promotion =
        std::fs::read_to_string(Path::new(ROOT).join("scripts/l3-self-teacher-promotion-gate.sh"))
            .expect("L3 promotion script");
    let reload =
        std::fs::read_to_string(Path::new(ROOT).join("scripts/reload-lay-model-services.sh"))
            .expect("model-service reload script");

    for source in [&bump, &promotion] {
        assert!(source.contains("reload-lay-model-services.sh"));
        assert!(!source.contains("pkill -x lay-ibus-engine"));
        assert!(!source.contains("ibus restart"));
    }
    assert!(reload.contains("ibus_pids_before="));
    assert!(reload.contains("ibus_pids_after="));
    assert!(reload.contains("systemctl --user restart lay-daemon.service"));
    assert!(!reload.contains("pkill"));
    assert!(!reload.contains("ibus restart"));
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
