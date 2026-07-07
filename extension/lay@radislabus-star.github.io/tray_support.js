import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

export const CONFIG_PATH = GLib.get_home_dir() + '/.config/lay/config.json';
export const STATS_PATH = GLib.get_home_dir() + '/.local/share/lay/stats.json';
export const RECENT_ACTIONS_PATH = GLib.get_home_dir() + '/.local/share/lay/recent_actions.jsonl';
export const PROJECT_DIR = GLib.get_home_dir() + '/projects/lay';
export const UPDATE_LOG_PATH = GLib.get_home_dir() + '/.local/state/lay/update.log';
export const APP_VERSION = '0.2.143';
export const APP_DESCRIPTION = 'Альфа: RU/EN-переключатель по двойному Shift и помощь при наборе';
export const APP_RELEASE_DATE = '2026-07-07';
export const APP_LICENSE = 'Non-Commercial';
export const APP_URL = 'https://github.com/radislabus-star/lay-public';
export const APP_PLATFORM = 'Linux: GNOME, KDE, Niri, Wayland, X11';
export const APP_GNOME_SUPPORT = 'GNOME 45-47, 50';
export const APP_ICON_NAME = 'input-keyboard-symbolic';
export const PANEL_ICON_SIZE = 14;
export const MENU_ICON_SIZE = 16;
export const MENU_WIDTH = 280;
export const DEFAULT_SCOPE_WORDS = 1;
export const MIN_SCOPE_WORDS = 1;
export const MAX_SCOPE_WORDS = 3;
export const COMPACT_SUBTITLE_STYLE = 'font-weight:normal; font-size:76%; opacity:180;';
export const SEGMENT_BUTTON_STYLE = 'padding:1px 5px; border-radius:6px; min-width:0;';
export const LEARNING_LOG_TOOLTIP = 'Запоминать правки работает в два слоя:\n'
    + '• double-Shift пишет факт ручного исправления;\n'
    + '• после авто/умной замены lay ждёт до 30 секунд,\n'
    + '  удалишь ли ты результат и введёшь свой вариант.\n'
    + 'Если удалил и перепечатал — это считается твоей правкой.';
export const DEBUG_ACTION_LOG_TOOLTIP = 'Единый рубильник диагностических журналов lay:\n'
    + 'действия, backend IME/uinput, NANDA trace и прекогниция.\n'
    + 'В обычном режиме лучше держать выключенным.';
export const AUTO_REPLACE_TOOLTIP = 'Когда включено: typo-правки после пробела и точные автоподмены.\n'
    + 'Когда выключено: остаётся только безопасный авто-layout EN/RU после пробела.';
export const AUTO_SWITCH_TOOLTIP = 'После автоматической помощи при наборе lay оставляет активной\n'
    + 'раскладку исправленного текста. Двойной Shift переключает раскладку всегда.';
export const ENTER_AUTOCORRECT_TOOLTIP = 'Опционально: перед Enter lay пробует исправить текущий хвост\n'
    + 'и только потом отправляет Enter. По умолчанию выключено, потому что Enter часто отправляет сообщение.';
export const LEM_2_TOOLTIP = 'LEM-арбитр для двух слов: сравнивает готовые варианты хвоста\n'
    + 'и выбирает более естественный, не генерируя новый текст.';
export const LEM_3_TOOLTIP = 'LEM-арбитр для трех слов и длиннее: нужен для смешанных RU/EN\n'
    + 'фраз, где соседние слова помогают понять раскладку.';
export const PTAH_ALEXS_TOOLTIP = 'Жёсткая раскладка по окну: при фокусе окна lay ставит\n'
    + 'заданную раскладку, а не вспоминает последнюю случайную.';
export const PTAH_RULE_LIMIT = 80;
export const TYPING_RULES = [
    {id: 'moved_prefix_pair', label: 'Перенос буквы'},
    {id: 'split_word_pair', label: 'Разбитое слово'},
    {id: 'visual_b', label: 'b → в/и'},
    {id: 'personal_phrase', label: 'Правила: фраза'},
    {id: 'personal_token', label: 'Правила: слово'},
    {id: 'duplicate_layout_prefix', label: 'Лишняя первая буква'},
    {id: 'mixed_script_layout', label: 'Смешанный ввод'},
    {id: 'layout_technical', label: 'Тех. токены'},
    {id: 'layout_ru_to_en', label: 'RU → EN'},
    {id: 'layout_en_to_ru', label: 'EN → RU'},
    {id: 'cyrillic_case', label: 'Регистр RU'},
    {id: 'hard_sign', label: 'ь/ъ'},
    {id: 'adjacent_transposition', label: 'Буквы местами'},
    {id: 'repeated_letter', label: 'Повтор буквы'},
    {id: 'single_letter_substitution', label: 'Соседняя клавиша'},
    {id: 'verb_ending', label: 'Окончание'},
    {id: 'vowel_confusion', label: 'Гласные'},
    {id: 'extra_letters', label: 'Лишние буквы'},
    {id: 'missing_letter', label: 'Пропущенная буква'},
    {id: 'glued_phrase', label: 'Склейка слов'},
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
export const SAFETY_OPTIONS = [
    ['strict', 'строго'],
    ['normal', 'норма'],
    ['experimental', 'эксп.'],
];
export const SAFETY_STEPS = [
    {id: 'strict', label: 'Осторожно', value: 0},
    {id: 'normal', label: 'Норма', value: 1},
    {id: 'experimental', label: 'Смелее', value: 2},
];
export const LAYOUT_BACKEND_OPTIONS = [
    ['auto', 'Авто'],
    ['gnome', 'GNOME'],
    ['kde', 'KDE'],
    ['x11', 'X11'],
    ['niri', 'Niri'],
];
export const DEFAULT_TYPING_PIPELINE = TYPING_RULES.map((rule, idx) => ({
    id: rule.id,
    enabled: true,
    priority: (idx + 1) * 10,
}));
export const DEFAULTS = {
    mode: 'simple',
    correction_engine: 'replay',
    layout_backend: 'auto',
    text_backend: 'uinput',
    trigger: 'double-lshift',
    force_layout_hotkeys: false,
    force_ru_key: 'single-rctrl',
    force_en_key: 'single-ralt',
    multi_tap_scope: false,
    multi_tap_max_taps: 4,
    tap_max_ms: 200,
    shift_window_ms: 250,
    debounce_ms: 50,
    replace_words: DEFAULT_SCOPE_WORDS,
    typing_assist_words: 2,
    auto_replace: false,
    typing_assist: false,
    correction_safety: 'normal',
    enter_autocorrect: false,
    auto_switch_layout: true,
    lem_enabled: true,
    lem_2_words: true,
    lem_3_words: true,
    lem_weight_percent: 80,
    nanda_l2_weight_percent: 20,
    nanda_l3_weight_percent: 8,
    llmwave_shadow: true,
    llmwave_apply: true,
    nanda_l2_phase_shadow: true,
    nanda_l2_phase_apply: false,
    nanda_l3_phase_shadow: true,
    ptah_alexs_mode: false,
    ptah_alexs_rules: [],
    typing_assist_pipeline: DEFAULT_TYPING_PIPELINE,
    debug_action_log: false,
    learning_log: false,
    nanda_autocorrect: false,
    nanda_trace: false,
    nanda_trace_text: false,
    nanda_precognition: false,
    ime_bracket_candidates: false,
};

export function loadConfig() {
    try {
        const [, bytes] = Gio.File.new_for_path(CONFIG_PATH).load_contents(null);
        const parsed = JSON.parse(new TextDecoder().decode(bytes));
        const cfg = normalizeConfig({...DEFAULTS, ...parsed});
        if (parsed.correction_engine === undefined)
            cfg.correction_engine = parsed.mode === 'llm' ? 'smart' : 'replay';
        cfg.typing_assist_pipeline = normalizeTypingPipeline(parsed.typing_assist_pipeline);
        cfg.ptah_alexs_mode = !!cfg.ptah_alexs_mode;
        cfg.ptah_alexs_rules = normalizePtahRules(parsed.ptah_alexs_rules);
        return cfg;
    } catch(e) {
        return {
            ...normalizeConfig(DEFAULTS),
            typing_assist_pipeline: normalizeTypingPipeline(DEFAULT_TYPING_PIPELINE),
            ptah_alexs_rules: [],
        };
    }
}
export function saveConfig(cfg) {
    try { Gio.File.new_for_path(GLib.get_home_dir() + '/.config/lay').make_directory_with_parents(null); } catch(e) {}
    cfg = normalizeConfig(cfg);
    cfg.typing_assist_pipeline = normalizeTypingPipeline(cfg.typing_assist_pipeline);
    cfg.ptah_alexs_rules = normalizePtahRules(cfg.ptah_alexs_rules);
    const bytes = new TextEncoder().encode(JSON.stringify(cfg, null, 2));
    Gio.File.new_for_path(CONFIG_PATH).replace_contents(
        bytes, null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);
}
export function clampNumber(value, min, max, fallback) {
    const number = Number(value);
    if (!Number.isFinite(number))
        return fallback;
    return Math.max(min, Math.min(max, number));
}
export function normalizeScope(value) {
    return clampNumber(value, MIN_SCOPE_WORDS, MAX_SCOPE_WORDS, DEFAULT_SCOPE_WORDS);
}
export function normalizeChoice(value, allowed, fallback) {
    return allowed.includes(value) ? value : fallback;
}
export function normalizeConfig(cfg) {
    const textBackend = normalizeChoice(cfg?.text_backend, ['uinput', 'ime', 'auto'], DEFAULTS.text_backend);
    return {
        ...DEFAULTS,
        ...cfg,
        replace_words: normalizeScope(cfg?.replace_words),
        typing_assist_words: normalizeScope(cfg?.typing_assist_words),
        correction_engine: normalizeChoice(cfg?.correction_engine, ['replay', 'smart'], DEFAULTS.correction_engine),
        layout_backend: normalizeChoice(cfg?.layout_backend, LAYOUT_BACKEND_OPTIONS.map(([id]) => id), DEFAULTS.layout_backend),
        text_backend: textBackend,
        nanda_precognition: !!cfg?.nanda_precognition,
        llmwave_shadow: cfg?.llmwave_shadow !== false,
        llmwave_apply: cfg?.llmwave_apply !== false,
        nanda_l2_phase_shadow: cfg?.nanda_l2_phase_shadow !== false,
        nanda_l2_phase_apply: !!cfg?.nanda_l2_phase_apply,
        nanda_l3_phase_shadow: cfg?.nanda_l3_phase_shadow !== false,
        ime_bracket_candidates: !!cfg?.ime_bracket_candidates,
        correction_safety: normalizeChoice(cfg?.correction_safety, SAFETY_OPTIONS.map(([id]) => id), DEFAULTS.correction_safety),
        ptah_alexs_mode: !!cfg?.ptah_alexs_mode,
        multi_tap_scope: !!cfg?.multi_tap_scope,
        multi_tap_max_taps: clampNumber(cfg?.multi_tap_max_taps, 2, 4, DEFAULTS.multi_tap_max_taps),
        lem_weight_percent: clampNumber(cfg?.lem_weight_percent, 0, 200, DEFAULTS.lem_weight_percent),
        nanda_l2_weight_percent: clampNumber(cfg?.nanda_l2_weight_percent, 0, 200, DEFAULTS.nanda_l2_weight_percent),
        nanda_l3_weight_percent: clampNumber(cfg?.nanda_l3_weight_percent, 0, 200, DEFAULTS.nanda_l3_weight_percent),
        mode: 'simple',
    };
}
export function optionLabel(options, id, fallback) {
    return options.find(([value]) => value === id)?.[1] ?? fallback;
}
export function normalizePtahLayout(layout) {
    const id = String(layout ?? '').trim().toLowerCase();
    if (['ru', 'rus', 'russian'].includes(id))
        return 'ru';
    if (['us', 'en', 'eng', 'english'].includes(id))
        return 'us';
    if (id === 'keep')
        return 'keep';
    return '';
}
export function normalizePtahRules(saved) {
    if (!Array.isArray(saved))
        return [];
    const out = [];
    const seen = new Set();
    for (const item of saved) {
        if (!item)
            continue;
        const kind = String(item.kind ?? '').trim();
        const value = String(item.value ?? '').trim();
        const layout = normalizePtahLayout(item.layout);
        if (!['app_id', 'wm_class', 'wm_class_instance'].includes(kind) || !value || !layout)
            continue;
        const key = `${kind}:${value.toLowerCase()}`;
        if (seen.has(key))
            continue;
        seen.add(key);
        out.push({
            kind,
            value,
            layout,
            label: String(item.label ?? value).trim().slice(0, 80) || value,
        });
        if (out.length >= PTAH_RULE_LIMIT)
            break;
    }
    return out;
}
export function normalizeTypingPipeline(saved) {
    const byId = new Map(DEFAULT_TYPING_PIPELINE.map(rule => [
        rule.id,
        {...rule},
    ]));
    if (Array.isArray(saved)) {
        for (const item of saved) {
            if (!item || !byId.has(item.id))
                continue;
            const rule = byId.get(item.id);
            rule.enabled = item.enabled !== false;
            if (Number.isFinite(item.priority) && item.priority > 0)
                rule.priority = item.priority;
        }
    }
    return [...byId.values()]
        .sort((a, b) => a.priority - b.priority)
        .map((rule, idx) => ({...rule, priority: (idx + 1) * 10}));
}
export function typingRuleLabel(id) {
    return TYPING_RULES.find(rule => rule.id === id)?.label ?? id;
}
export function loadStats() {
    try {
        const [, bytes] = Gio.File.new_for_path(STATS_PATH).load_contents(null);
        return JSON.parse(new TextDecoder().decode(bytes));
    } catch(e) {
        return {};
    }
}
export function loadRecentActions(limit = 5) {
    try {
        const [, bytes] = Gio.File.new_for_path(RECENT_ACTIONS_PATH).load_contents(null);
        return new TextDecoder().decode(bytes)
            .split('\n')
            .filter(line => line.trim().length > 0)
            .slice(-limit)
            .map(line => JSON.parse(line))
            .reverse();
    } catch(e) {
        return [];
    }
}
export function clearRecentActions() {
    try {
        Gio.File.new_for_path(RECENT_ACTIONS_PATH).replace_contents(
            new TextEncoder().encode(''),
            null,
            false,
            Gio.FileCreateFlags.REPLACE_DESTINATION,
            null
        );
        return true;
    } catch(e) {
        log(`[lay-extension] clear recent actions failed: ${e}`);
        return false;
    }
}
export function summarizeRecentActions(actions) {
    const total = actions.length;
    if (total === 0)
        return 'нет действий';
    const counts = {};
    let elapsed = 0;
    for (const action of actions) {
        const kind = String(action.kind ?? 'action');
        counts[kind] = (counts[kind] ?? 0) + 1;
        elapsed += Number(action.elapsed_ms ?? 0);
    }
    const top = Object.entries(counts)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 3)
        .map(([kind, count]) => `${actionKindLabel(kind)}:${count}`)
        .join(' · ');
    return `${total} действий · среднее ${Math.round(elapsed / total)}мс · ${top}`;
}
export function actionKindLabel(kind) {
    return {
        'layout-replay': 'Двойной Shift',
        'smart-text': 'Умная замена',
        'auto-replace': 'Автоподмена',
        'typing-assist': 'Помощь',
        'enter-autocorrect': 'Enter',
        'layout-text-fallback': 'Резерв',
        'auto-undo': 'Откат',
    }[kind] ?? String(kind ?? 'action');
}
export function restartDaemon() {
    daemonCommand('restart');
}
export function startDaemon() {
    daemonCommand('start');
}
export function stopDaemon() {
    stopLayRuntime();
}
export function daemonCommand(action) {
    runRuntimeControl(action);
}
export function stopLayRuntime() {
    runRuntimeControl('stop');
}
export function applyInputChannel(channel) {
    if (!['ime', 'uinput', 'auto'].includes(channel))
        return;
    runRuntimeControl(`channel ${shellQuote(channel)}`);
}
function runRuntimeControl(args) {
    const helper = `${GLib.get_home_dir()}/.local/bin/lay-runtime-control`;
    runShell(`${shellQuote(helper)} ${args}`);
}
function runShell(command) {
    try {
        Gio.Subprocess.new(['bash', '-lc', command], Gio.SubprocessFlags.NONE);
    } catch(e) {}
}
export function firstExistingCommand(names) {
    for (const name of names)
        if (GLib.find_program_in_path(name))
            return name;
    return null;
}
export function shellQuote(value) {
    return "'" + String(value).replaceAll("'", "'\"'\"'") + "'";
}
export function startUpdate() {
    const updateScript = PROJECT_DIR + '/update.sh';
    if (!GLib.file_test(updateScript, GLib.FileTest.EXISTS))
        return [false, `Не найден update.sh: ${updateScript}`];

    try { GLib.mkdir_with_parents(GLib.path_get_dirname(UPDATE_LOG_PATH), 0o755); } catch(e) {}
    const projectArg = shellQuote(PROJECT_DIR);
    const logArg = shellQuote(UPDATE_LOG_PATH);
    const updateCommand = 'cd ' + projectArg + ' && '
        + 'bash update.sh 2>&1 | tee ' + logArg + '; '
        + 'code=${PIPESTATUS[0]}; '
        + 'printf "\\nЛог: %s\\n\\n" ' + logArg + '; '
        + 'read -r -p "Нажми Enter, чтобы закрыть окно..."; '
        + 'exit ${code}';

    const terminal = firstExistingCommand(['kgx', 'gnome-terminal', 'konsole', 'xterm']);
    try {
        if (terminal === 'kgx') {
            Gio.Subprocess.new(
                ['kgx', '--working-directory', PROJECT_DIR, '--', 'bash', '-lc', updateCommand],
                Gio.SubprocessFlags.NONE);
            return [true, `Проверка открыта в терминале. Лог: ${UPDATE_LOG_PATH}`];
        }
        if (terminal === 'gnome-terminal') {
            Gio.Subprocess.new(
                ['gnome-terminal', '--working-directory', PROJECT_DIR, '--', 'bash', '-lc', updateCommand],
                Gio.SubprocessFlags.NONE);
            return [true, `Проверка открыта в терминале. Лог: ${UPDATE_LOG_PATH}`];
        }
        if (terminal === 'konsole') {
            Gio.Subprocess.new(
                ['konsole', '--workdir', PROJECT_DIR, '-e', 'bash', '-lc', updateCommand],
                Gio.SubprocessFlags.NONE);
            return [true, `Проверка открыта в терминале. Лог: ${UPDATE_LOG_PATH}`];
        }
        if (terminal === 'xterm') {
            Gio.Subprocess.new(['xterm', '-e', 'bash', '-lc', updateCommand], Gio.SubprocessFlags.NONE);
            return [true, `Проверка открыта в терминале. Лог: ${UPDATE_LOG_PATH}`];
        }

        const backgroundCommand = 'cd ' + projectArg + ' && bash update.sh > ' + logArg + ' 2>&1';
        Gio.Subprocess.new(['bash', '-lc', backgroundCommand], Gio.SubprocessFlags.NONE);
        return [true, `Терминал не найден, проверка запущена в фоне. Лог: ${UPDATE_LOG_PATH}`];
    } catch(e) {
        return [false, String(e)];
    }
}
export function openUri(uri) {
    try {
        Gio.AppInfo.launch_default_for_uri(uri, global.create_app_launch_context(0, -1));
    } catch(e) {
        try { Gio.Subprocess.new(['xdg-open', uri], Gio.SubprocessFlags.NONE); } catch(_e) {}
    }
}
export function openPreferences() {
    const runtimePath = `${GLib.get_home_dir()}/.local/share/gnome-shell/extensions/lay@radislabus-star.github.io/settings.js`;
    const projectPath = `${PROJECT_DIR}/extension/lay@radislabus-star.github.io/settings.js`;
    const scriptPath = GLib.file_test(runtimePath, GLib.FileTest.EXISTS) ? runtimePath : projectPath;
    try {
        Gio.Subprocess.new(
            ['gjs', '-m', scriptPath],
            Gio.SubprocessFlags.NONE
        );
    } catch(_e) {}
}
