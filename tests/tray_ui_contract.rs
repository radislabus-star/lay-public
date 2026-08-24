use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const EXTENSION: &str = "extension/lay@radislabus-star.github.io";

#[test]
fn gnome_tray_is_a_compact_daily_surface() {
    let tray = read_extension("lay-impl.js");
    for label in [
        "Раскладка: --",
        "Lay включён",
        "Режим ввода:",
        "Помощь при наборе",
        "Автозамена",
        "Настройки",
        "Диагностика",
        "Открыть журнал",
    ] {
        assert!(tray.contains(label), "missing daily tray action: {label}");
    }
    assert!(read_extension("recent_actions_menu.js").contains("Последние действия"));

    for stale in [
        "Проверить обновления",
        "Перезапустить службы",
        "Журнал отладки действий",
        "NANDA ячейки",
        "Автокоррекция NANDA",
        "О Lay",
    ] {
        assert!(!tray.contains(stale), "stale tray action remains: {stale}");
    }
}

#[test]
fn tray_actions_have_real_runtime_owners() {
    let tray = read_extension("lay-impl.js");
    for owner in [
        "activateLayoutId(target)",
        "startDaemon();",
        "stopDaemon();",
        "this._cfg.text_backend = id;",
        "this._cfg.nanda_precognition = id !== 'uinput';",
        "applyInputChannel(id);",
        "saveConfig(this._cfg);",
        "restartDaemon();",
        "openPreferences()",
        "openDiagnosticsLog()",
        "createRecentActionsMenu(this)",
    ] {
        assert!(tray.contains(owner), "tray action has no owner: {owner}");
    }
}

#[test]
fn one_shared_settings_view_owns_both_entrypoints() {
    let standalone = read_extension("settings.js");
    let preferences = read_extension("prefs.js");
    let shared = read_extension("settings_view.js");

    for entrypoint in [&standalone, &preferences] {
        assert!(entrypoint.contains("from './settings_view.js'"));
        assert!(entrypoint.contains("createSettingsPage()"));
        assert!(!entrypoint.contains("saveConfig("));
        assert!(!entrypoint.contains("Adw.SwitchRow"));
    }
    assert!(shared.contains("export function createSettingsPage()"));
    assert!(shared.contains("export class LaySettingsView"));
}

#[test]
fn settings_expose_only_user_owned_controls() {
    let settings = read_extension("settings_view.js");
    for label in [
        "Режим ввода",
        "Помощь при наборе",
        "Автозамена",
        "Осторожность",
        "Следовать языку исправления",
        "Запоминать ручные правки",
        "Показывать хвост в скобках",
        "Исправить последнее слово",
        "Длительность нажатия Shift",
        "Интервал двойного Shift",
        "Отдельные клавиши RU / EN",
        "Среда переключения раскладки",
        "Подробный журнал действий",
        "Состояние служб",
        "Журнал Lay",
    ] {
        assert!(
            settings.contains(label),
            "missing accepted setting: {label}"
        );
    }

    for stale in [
        "Вес L2 кандидатов",
        "Вес L3 фразы",
        "NANDA ячейки",
        "Multi-tap максимум",
        "Несколько нажатий триггера",
        "Исправлять перед Enter",
        "Раскладка по окну",
        "Автокоррекция NANDA",
    ] {
        assert!(
            !settings.contains(stale),
            "internal setting remains visible: {stale}"
        );
    }
}

#[test]
fn double_shift_timing_controls_write_existing_bounded_runtime_keys() {
    let settings = read_extension("settings_view.js");
    let support = read_extension("tray_support.js");

    assert!(settings.contains("'tap_max_ms'"));
    assert!(settings.contains("'shift_window_ms'"));
    assert!(settings.contains("this._save({restart: true})"));
    assert!(support.contains("normalizeBoundedInteger(cfg.tap_max_ms"));
    assert!(support.contains("cfg.shift_window_ms, DEFAULTS.shift_window_ms"));
}

#[test]
fn model_authority_knobs_stay_out_of_user_ui() {
    let settings = read_extension("settings_view.js");
    let tray = read_extension("lay-impl.js");
    let kde = read("scripts/lay-kde-tray.py");
    for source in [&settings, &tray, &kde] {
        for key in [
            "nanda_l2_weight_percent",
            "nanda_l3_weight_percent",
            "nanda_autocorrect",
            "llmwave_shadow",
            "llmwave_apply",
            "nanda_l2_phase_shadow",
            "nanda_l2_phase_apply",
            "nanda_l3_phase_shadow",
        ] {
            assert!(
                !source.contains(key),
                "model authority leaked into UI: {key}"
            );
        }
    }
}

#[test]
fn config_writer_preserves_unexposed_runtime_keys() {
    let support = read_extension("tray_support.js");
    let save = nearby(&support, "export function saveConfig(cfg) {", 600);
    assert!(save.contains("{...readConfigObject(), ...cfg}"));
    assert!(save.contains("JSON.stringify(merged"));
    assert!(!save.contains("Object.keys(DEFAULTS)"));
}

#[test]
fn input_mode_has_one_channel_contract_everywhere() {
    let tray = read_extension("lay-impl.js");
    let settings = read_extension("settings_view.js");
    let kde = read("scripts/lay-kde-tray.py");

    assert!(tray.contains("this._cfg.nanda_precognition = id !== 'uinput';"));
    assert!(tray.contains("applyInputChannel(id);"));
    assert!(settings.contains("this.cfg.nanda_precognition = id === 'ime';"));
    assert!(settings.contains("applyInputChannel(channel);"));
    assert!(kde.contains("cfg[\"nanda_precognition\"] = value == \"ime\""));
    assert!(kde.contains("runtime_control(\"channel\", value)"));

    for source in [&tray, &settings, &kde] {
        assert!(!source.contains("IME, эксперимент"));
        assert!(!source.contains("(\"auto\", \"Авто\")"));
    }
}

#[test]
fn diagnostics_are_observational_and_enablement_has_one_explicit_owner() {
    let tray = read_extension("lay-impl.js");
    let settings = read_extension("settings_view.js");
    let kde = read("scripts/lay-kde-tray.py");
    for source in [&tray, &settings, &kde] {
        assert!(source.contains("Открыть журнал") || source.contains("Журнал Lay"));
        assert!(!source.contains("Перезапустить демон"));
        assert!(!source.contains("Перезапустить службы"));
        assert!(!source.contains("Демон включён"));
    }
    assert_eq!(
        tray.lines()
            .filter(|line| line.trim() == "startDaemon();")
            .count(),
        1
    );
    assert_eq!(
        tray.lines()
            .filter(|line| line.trim() == "stopDaemon();")
            .count(),
        1
    );
    assert!(kde.contains("runtime_control(\"start\" if enabled else \"stop\")"));
}

#[test]
fn disabled_lay_cannot_be_reactivated_by_input_source_sync() {
    let tray = read_extension("lay-impl.js");
    let source_change = nearby(&tray, "current-source-changed", 320);
    assert!(source_change.contains("this._daemonActive === true"));
    assert!(source_change.contains("this._cfg.text_backend === 'ime'"));
    assert!(source_change.contains("syncIbusEngineForCurrentLayout()"));

    let enabled = nearby(&tray, "_enabledSwitchItem() {", 750);
    assert!(enabled.contains("this._daemonActive = state;"));
    assert!(enabled.contains("startDaemon();"));
    assert!(enabled.contains("stopDaemon();"));
}

#[test]
fn stale_service_status_cannot_override_a_new_enablement_choice() {
    let tray = read_extension("lay-impl.js");
    let enabled = nearby(&tray, "_enabledSwitchItem() {", 800);
    assert!(enabled.contains("this._statusGeneration ="));

    let refresh = nearby(&tray, "_refreshStatus() {", 850);
    assert!(refresh.contains("const generation ="));
    assert!(refresh.contains("generation !== this._statusGeneration"));
    assert!(refresh.contains("return;"));
}

#[test]
fn debug_journal_switch_keeps_trace_flags_together() {
    let settings = read_extension("settings_view.js");
    let debug = nearby(&settings, "_debugLogRow() {", 650);
    assert!(debug.contains("debug_action_log"));
    assert!(debug.contains("nanda_trace"));
    assert!(debug.contains("nanda_trace_text"));
    assert!(!debug.contains("nanda_precognition"));
}

#[test]
fn duplicate_force_layout_hotkeys_are_normalized_before_save() {
    let support = read_extension("tray_support.js");
    assert!(support.contains("cfg.force_en_key === cfg.force_ru_key"));
    assert!(support.contains("cfg.force_en_key ="));
}

#[test]
fn opening_tray_refreshes_config_and_visible_state() {
    let tray = read_extension("lay-impl.js");
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
fn runtime_restart_preserves_visible_layout_and_global_ibus() {
    let runtime = read("scripts/lay-runtime-control.sh");
    let start = nearby(&runtime, "start_ime() {", 500);
    assert!(start.contains("preferred_lay_ime"));
    assert!(start.contains("select_lay_ime \"$preferred\""));
    assert!(start.contains("select_lay_ime \"$fallback\""));
    assert!(!runtime.contains("ibus restart"));
}

#[test]
fn changing_input_mode_cannot_reenable_a_disabled_runtime() {
    let runtime = read("scripts/lay-runtime-control.sh");
    let channel = nearby(&runtime, "channel)", 330);
    assert!(channel.contains("systemctl --user is-active --quiet lay-daemon.service"));
    assert!(channel.contains("apply_channel"));
    assert!(channel.contains("stop_ime"));

    let restart = nearby(&runtime, "restart)", 330);
    assert!(restart.contains("systemctl --user is-active --quiet lay-daemon.service"));
    assert!(restart.contains("systemctl --user restart lay-daemon.service"));
    assert!(restart.contains("stop_ime"));
}

#[test]
fn kde_tray_has_a_real_ru_en_layout_action() {
    let kde = read("scripts/lay-kde-tray.py");
    assert!(kde.contains("Переключить раскладку RU / EN"));
    assert!(kde.contains("def switch_kde_layout()"));
    assert!(kde.contains("/Layouts\", \"getLayout"));
    assert!(kde.contains("/Layouts\", \"setLayout"));
}

#[test]
fn settings_desktop_entry_and_tray_use_the_same_standalone_entrypoint() {
    let desktop = read("extension/lay-settings.desktop");
    let support = read_extension("tray_support.js");
    assert!(desktop.contains("settings.js"));
    assert!(support.contains("settings.js"));
    assert!(Path::new(ROOT)
        .join("extension/lay@radislabus-star.github.io/settings_view.js")
        .is_file());
}

fn read_extension(file: &str) -> String {
    read(&format!("{EXTENSION}/{file}"))
}

fn read(file: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(file)).expect("project source")
}

fn nearby(source: &str, marker: &str, chars: usize) -> String {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker {marker}"));
    source[start..].chars().take(chars).collect()
}
