import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

export const CONFIG_PATH = GLib.get_home_dir() + '/.config/lay/config.json';
export const RECENT_ACTIONS_PATH = GLib.get_home_dir() + '/.local/share/lay/recent_actions.jsonl';
export const PROJECT_DIR = GLib.get_home_dir() + '/projects/lay';
export const APP_VERSION = '1.0.40';
export const APP_RELEASE_DATE = '2026-08-24';
export const APP_URL = 'https://github.com/radislabus-star/lay-public';
export const APP_ICON_NAME = 'input-keyboard-symbolic';
export const PANEL_ICON_SIZE = 14;
export const MENU_WIDTH = 280;
export const COMPACT_SUBTITLE_STYLE = 'font-weight:normal; font-size:76%; opacity:180;';

export const INPUT_MODE_OPTIONS = [
    ['uinput', 'Быстрый ввод'],
    ['ime', 'IME-подсказки'],
];
export const SAFETY_OPTIONS = [
    ['strict', 'Осторожно'],
    ['normal', 'Норма'],
    ['experimental', 'Смелее'],
];
export const TRIGGER_OPTIONS = [
    ['double-lshift', 'Двойной Shift'],
    ['double-ctrl', 'Двойной Ctrl'],
    ['double-alt', 'Двойной Alt'],
    ['caps-lock', 'CapsLock'],
    ['single-rshift', 'Правый Shift'],
    ['single-rctrl', 'Правый Ctrl'],
    ['single-ralt', 'Правый Alt'],
    ['single-pause', 'Pause'],
];
export const FORCE_KEY_OPTIONS = [
    ['single-rctrl', 'Правый Ctrl'],
    ['single-ralt', 'Правый Alt'],
    ['single-rshift', 'Правый Shift'],
    ['single-pause', 'Pause'],
    ['caps-lock', 'CapsLock'],
];
export const LAYOUT_BACKEND_OPTIONS = [
    ['auto', 'Автоматически'],
    ['gnome', 'GNOME'],
    ['kde', 'KDE / Plasma'],
    ['x11', 'X11'],
    ['niri', 'Niri'],
];

const DEFAULTS = {
    layout_backend: 'auto',
    text_backend: 'uinput',
    trigger: 'double-lshift',
    force_layout_hotkeys: false,
    force_ru_key: 'single-rctrl',
    force_en_key: 'single-ralt',
    auto_replace: false,
    typing_assist: false,
    correction_safety: 'normal',
    auto_switch_layout: true,
    debug_action_log: false,
    learning_log: false,
    nanda_trace: false,
    nanda_trace_text: false,
    nanda_precognition: false,
    ime_bracket_candidates: false,
};

function readConfigObject() {
    try {
        const [, bytes] = Gio.File.new_for_path(CONFIG_PATH).load_contents(null);
        const parsed = JSON.parse(new TextDecoder().decode(bytes));
        return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
    } catch(e) {
        return {};
    }
}

export function normalizeChoice(value, options, fallback) {
    const allowed = options.map(([id]) => id);
    return allowed.includes(value) ? value : fallback;
}

export function normalizeConfig(input) {
    const cfg = {...DEFAULTS, ...(input ?? {})};
    const legacyTextBackend = cfg.text_backend === 'auto' ? 'ime' : cfg.text_backend;
    cfg.text_backend = normalizeChoice(legacyTextBackend, INPUT_MODE_OPTIONS, DEFAULTS.text_backend);
    cfg.layout_backend = normalizeChoice(cfg.layout_backend, LAYOUT_BACKEND_OPTIONS, DEFAULTS.layout_backend);
    cfg.trigger = normalizeChoice(cfg.trigger, TRIGGER_OPTIONS, DEFAULTS.trigger);
    cfg.correction_safety = normalizeChoice(cfg.correction_safety, SAFETY_OPTIONS, DEFAULTS.correction_safety);

    cfg.force_ru_key = normalizeChoice(cfg.force_ru_key, FORCE_KEY_OPTIONS, DEFAULTS.force_ru_key);
    cfg.force_en_key = normalizeChoice(cfg.force_en_key, FORCE_KEY_OPTIONS, DEFAULTS.force_en_key);
    if (cfg.force_en_key === cfg.force_ru_key)
        cfg.force_en_key = cfg.force_ru_key === DEFAULTS.force_en_key ? DEFAULTS.force_ru_key : DEFAULTS.force_en_key;

    for (const key of [
        'force_layout_hotkeys',
        'auto_replace',
        'typing_assist',
        'auto_switch_layout',
        'debug_action_log',
        'learning_log',
        'nanda_trace',
        'nanda_trace_text',
        'nanda_precognition',
        'ime_bracket_candidates',
    ])
        cfg[key] = !!cfg[key];
    return cfg;
}

export function loadConfig() {
    return normalizeConfig(readConfigObject());
}

export function saveConfig(cfg) {
    try {
        Gio.File.new_for_path(GLib.get_home_dir() + '/.config/lay').make_directory_with_parents(null);
    } catch(e) {}

    // Keep runtime-owned and future config keys that this UI does not expose.
    const merged = normalizeConfig({...readConfigObject(), ...cfg});
    const bytes = new TextEncoder().encode(JSON.stringify(merged, null, 2) + '\n');
    Gio.File.new_for_path(CONFIG_PATH).replace_contents(
        bytes, null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);
}

export function loadRecentActions(limit = 5) {
    try {
        const [, bytes] = Gio.File.new_for_path(RECENT_ACTIONS_PATH).load_contents(null);
        return new TextDecoder().decode(bytes).split('\n').filter(Boolean).slice(-limit).reverse()
            .map(line => {
                try { return JSON.parse(line); } catch(e) { return null; }
            }).filter(Boolean);
    } catch(e) {
        return [];
    }
}

export function clearRecentActions() {
    try {
        Gio.File.new_for_path(RECENT_ACTIONS_PATH).replace_contents(
            new Uint8Array(), null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);
        return true;
    } catch(e) {
        return false;
    }
}

export function summarizeRecentActions(actions) {
    if (actions.length === 0)
        return 'нет действий';
    const counts = new Map();
    let elapsed = 0;
    let undo = 0;
    for (const action of actions) {
        const kind = String(action.kind ?? 'action');
        counts.set(kind, (counts.get(kind) ?? 0) + 1);
        elapsed += Number(action.elapsed_ms ?? 0);
        if (action.undo_available)
            undo += 1;
    }
    const top = [...counts.entries()].sort((a, b) => b[1] - a[1]).slice(0, 3)
        .map(([kind, count]) => `${actionKindLabel(kind)}:${count}`).join(' · ');
    return `${actions.length} действий · среднее ${Math.round(elapsed / actions.length)}мс · undo ${undo} · ${top}`;
}

export function actionKindLabel(kind) {
    return {
        'layout-replay': 'Двойной Shift',
        'smart-text': 'Умная замена',
        'auto-replace': 'Автозамена',
        'typing-assist': 'Помощь',
        'enter-autocorrect': 'Enter',
        'layout-text-fallback': 'Резерв',
        'auto-undo': 'Откат',
    }[kind] ?? String(kind ?? 'action');
}

function runtimeControl(action, argument = null) {
    const args = [GLib.get_home_dir() + '/.local/bin/lay-runtime-control', action];
    if (argument)
        args.push(argument);
    try { Gio.Subprocess.new(args, Gio.SubprocessFlags.NONE); } catch(e) {}
}

export function restartDaemon() {
    runtimeControl('restart');
}

export function startDaemon() {
    runtimeControl('start');
}

export function stopDaemon() {
    runtimeControl('stop');
}

export function applyInputChannel(channel) {
    if (['ime', 'uinput'].includes(channel))
        runtimeControl('channel', channel);
}

export function openUri(uri) {
    try {
        Gio.AppInfo.launch_default_for_uri(uri, null);
    } catch(e) {
        try { Gio.Subprocess.new(['xdg-open', uri], Gio.SubprocessFlags.NONE); } catch(_e) {}
    }
}

export function openPreferences() {
    const installed = `${GLib.get_home_dir()}/.local/share/gnome-shell/extensions/lay@radislabus-star.github.io/settings.js`;
    const project = `${PROJECT_DIR}/extension/lay@radislabus-star.github.io/settings.js`;
    const script = GLib.file_test(installed, GLib.FileTest.EXISTS) ? installed : project;
    try { Gio.Subprocess.new(['gjs', '-m', script], Gio.SubprocessFlags.NONE); } catch(e) {}
}

export function openDiagnosticsLog() {
    const command = 'journalctl --user -u lay-daemon.service -u lay-l3-online.service '
        + '-u lay-l2-online.service -n 250 --no-pager; '
        + 'printf "\\nНажми Enter, чтобы закрыть окно..."; read -r';
    for (const [binary, args] of [
        ['gnome-terminal', ['--', 'bash', '-lc', command]],
        ['kgx', ['--', 'bash', '-lc', command]],
        ['konsole', ['-e', 'bash', '-lc', command]],
        ['xterm', ['-e', 'bash', '-lc', command]],
    ]) {
        if (!GLib.find_program_in_path(binary))
            continue;
        try {
            Gio.Subprocess.new([binary, ...args], Gio.SubprocessFlags.NONE);
            return true;
        } catch(e) {}
    }
    return false;
}
