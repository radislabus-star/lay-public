import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk';

import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

const CONFIG_PATH = GLib.get_home_dir() + '/.config/lay/config.json';
const APP_VERSION = '0.2.19';
const APP_RELEASE_DATE = '2026-06-25';
const APP_URL = 'https://github.com/radislabus-star/lay-public';
const APP_ICON_NAME = 'input-keyboard-symbolic';
const HEADER_ICON_SIZE = 16;
const NANDA_WAVE_STATUS_FALLBACK = {
    kind: 'nanda_wave_status_unavailable',
    source: 'fallback',
    error: 'lay-nanda-wave-eval --status-json недоступен',
    cell: {},
    gate: {},
    zones: [
        {id: 'sensors', label: 'Сенсоры', layer: 'L1'},
        {id: 'candidates', label: 'Кандидаты', layer: 'L2'},
        {id: 'consensus', label: 'Согласование', layer: 'L3'},
    ],
    cells: [],
    ablation: [],
};

function loadNandaWaveStatus() {
    const bins = [
        `${GLib.get_home_dir()}/.local/bin/lay-nanda-wave-eval`,
        `${GLib.get_home_dir()}/projects/lay/target/release/lay-nanda-wave-eval`,
    ];
    for (const bin of bins) {
        if (!GLib.file_test(bin, GLib.FileTest.EXISTS))
            continue;
        try {
            const proc = Gio.Subprocess.new(
                [bin, '--status-json'],
                Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_SILENCE
            );
            const [, stdout] = proc.communicate_utf8(null, null);
            const status = JSON.parse(String(stdout ?? '').trim());
            if (status && status.kind === 'nanda_wave_status')
                return status;
        } catch(e) {}
    }
    return NANDA_WAVE_STATUS_FALLBACK;
}

function cellVisualLabel(name) {
    return {
        Utf8Cell32: 'UTF-8',
        ScriptCell32: 'Письмо',
        KeyboardCell32: 'Клавиши',
        BoundaryCell32: 'Границы',
        LayoutWordCell32: 'Раскладка',
        TechTokenCell32: 'Тех. токен',
        TechnicalContextCell32: 'Защита',
        PhraseCell32: 'Фраза',
        GrammarCell32: 'Грамматика',
        MeshConsensusCell32: 'Mesh',
    }[name] ?? name;
}

function percent(ok, total) {
    if (!Number.isFinite(ok) || !Number.isFinite(total) || total <= 0)
        return 'н/д';
    return `${(ok / total * 100).toFixed(1)}%`;
}

function nandaStatusLine(status) {
    if (status.kind !== 'nanda_wave_status')
        return status.error ?? 'статус недоступен';
    const gate = status.gate ?? {};
    return `${gate.promotion_status ?? 'unknown'} / ${gate.mode_status ?? 'unknown'}`;
}

function nandaPassportText(status) {
    const cell = status.cell ?? {};
    const gate = status.gate ?? {};
    const cells = Array.isArray(status.cells) ? status.cells : [];
    const ablation = Array.isArray(status.ablation) ? status.ablation : [];
    const candidateStats = Array.isArray(status.candidate_stats) ? status.candidate_stats : [];
    const preeditLive = status.preedit_live ?? {};
    const scoreboard = status.cell_scoreboard && Array.isArray(status.cell_scoreboard.cells)
        ? status.cell_scoreboard
        : {records: 0, cells: []};
    const lines = [
        'Паспорт NANDA клеток',
        '',
        `Источник: ${status.source ?? 'неизвестно'}`,
        `Статус: ${nandaStatusLine(status)}`,
        `Сгенерировано: ${status.generated_at_unix ?? 'нет данных'}`,
        '',
        'Размер клетки',
        `  ${cell.name ?? 'NandaCell32v0'}: ${cell.bytes ? Math.round(cell.bytes / 1024) : '?'} КБ`,
        `  Mode: ${cell.mode_bytes ?? '?'} Б`,
        `  Мод в клетке: ${cell.modes ?? '?'}`,
        `  Top-K выход: ${cell.top_k ?? '?'}`,
        `  Sparse probes: ${cell.sparse_probes ?? '?'}`,
        '',
        'Последний real-suite',
        `  cases:         ${gate.cases ?? '?'}${gate.sampled ? ` / ${gate.full_cases ?? '?'} sample` : ''}`,
        `  baseline:      ${gate.baseline_ok ?? '?'} / ${gate.cases ?? '?'} · ${percent(gate.baseline_ok, gate.cases)}`,
        `  NANDA Wave:    ${gate.wave_ok ?? '?'} / ${gate.cases ?? '?'} · ${percent(gate.wave_ok, gate.cases)}`,
        `  changed:       ${gate.wave_changed ?? '?'}`,
        `  worsened:      ${gate.worsened_vs_baseline ?? '?'}`,
        '',
        'Ячейки',
    ];
    if (cells.length === 0)
        lines.push('  данных нет');
    for (const item of cells)
        lines.push(`  ${item.layer ?? '?'} ${item.label ?? item.name}: ${item.role ?? ''} · delta ${item.delta ?? 0} · ${item.alive ? 'живая' : 'след 0'}`);
    lines.push('', 'Кандидаты');
    if (candidateStats.length === 0)
        lines.push('  данных нет');
    for (const item of candidateStats)
        lines.push(`  ${item.source ?? '?'}: родила ${item.generated ?? 0}, приняла ${item.accepted ?? 0}, veto ${item.vetoed ?? 0}, keep ${item.kept ?? 0}`);
    lines.push(
        '',
        'IME подсказки',
        `  Tab/Alt принято: ${preeditLive.accepted ?? 0} / ${preeditLive.sessions ?? 0} · ${percent(preeditLive.accepted, preeditLive.sessions)}`,
        `  сброшено без принятия: ${preeditLive.abandoned ?? 0}`,
    );
    lines.push('', `Журнал клеток: ${scoreboard.records ?? 0} записей`);
    if (scoreboard.cells.length === 0)
        lines.push('  данных нет');
    for (const item of scoreboard.cells)
        lines.push(`  ${cellVisualLabel(item.cell)}: ${item.status ?? 'н/д'} · приняла ${item.accepted ?? 0}, veto ${item.vetoed ?? 0}, ok ${item.ok ?? 0}, bad ${item.bad ?? 0}`);
    lines.push('', 'Ablation');
    if (ablation.length === 0)
        lines.push('  данных нет');
    for (const item of ablation)
        lines.push(`  без ${item.cell}: ${item.ok}/${item.cases}, delta ${Number(item.delta ?? 0) >= 0 ? '+' : ''}${item.delta ?? 0}`);
    return lines.join('\n');
}

const ENGINE_OPTIONS = [
    ['replay', 'Обычный'],
    ['smart', 'Умный'],
];
const SCOPE_OPTIONS = [
    ['1', '1 слово'],
    ['2', '2 слова'],
    ['3', '3 слова'],
];
const SAFETY_OPTIONS = [
    ['strict', 'Осторожно'],
    ['normal', 'Норма'],
    ['experimental', 'Смелее'],
];
const TRIGGER_OPTIONS = [
    ['double-lshift', 'Двойной Shift'],
    ['double-ctrl', 'Двойной Ctrl'],
    ['double-alt', 'Двойной Alt'],
    ['caps-lock', 'CapsLock'],
    ['single-rshift', 'Правый Shift'],
    ['single-rctrl', 'Правый Ctrl'],
    ['single-ralt', 'Правый Alt'],
    ['single-pause', 'Pause'],
];
const FORCE_KEY_OPTIONS = [
    ['single-rctrl', 'Правый Ctrl'],
    ['single-ralt', 'Правый Alt'],
    ['single-rshift', 'Правый Shift'],
    ['single-pause', 'Pause'],
    ['caps-lock', 'CapsLock'],
];
const BACKEND_OPTIONS = [
    ['uinput', 'Быстрый ввод'],
    ['ime', 'IME, эксперимент'],
    ['auto', 'Авто'],
];
const LAYOUT_BACKEND_OPTIONS = [
    ['auto', 'Авто'],
    ['gnome', 'GNOME'],
    ['kde', 'KDE'],
    ['x11', 'X11'],
    ['niri', 'Niri'],
];

const DEFAULTS = {
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
    replace_words: 1,
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
    ptah_alexs_mode: false,
    ptah_alexs_rules: [],
    debug_action_log: false,
    learning_log: false,
    nanda_autocorrect: false,
    nanda_trace: false,
    nanda_trace_text: false,
    nanda_precognition: false,
    ime_bracket_candidates: false,
};

function normalizeChoice(value, allowed, fallback) {
    return allowed.includes(value) ? value : fallback;
}

function normalizeNumber(value, min, max, fallback) {
    const number = Number(value);
    if (!Number.isFinite(number))
        return fallback;
    return Math.max(min, Math.min(max, number));
}

function normalizeConfig(cfg) {
    const textBackend = normalizeChoice(cfg?.text_backend, BACKEND_OPTIONS.map(([id]) => id), DEFAULTS.text_backend);
    return {
        ...DEFAULTS,
        ...cfg,
        mode: 'simple',
        correction_engine: normalizeChoice(cfg?.correction_engine, ['replay', 'smart'], DEFAULTS.correction_engine),
        layout_backend: normalizeChoice(cfg?.layout_backend, LAYOUT_BACKEND_OPTIONS.map(([id]) => id), DEFAULTS.layout_backend),
        text_backend: textBackend,
        nanda_precognition: !!cfg?.nanda_precognition,
        ime_bracket_candidates: !!cfg?.ime_bracket_candidates,
        trigger: normalizeChoice(cfg?.trigger, TRIGGER_OPTIONS.map(([id]) => id), DEFAULTS.trigger),
        force_ru_key: normalizeChoice(cfg?.force_ru_key, FORCE_KEY_OPTIONS.map(([id]) => id), DEFAULTS.force_ru_key),
        force_en_key: normalizeChoice(cfg?.force_en_key, FORCE_KEY_OPTIONS.map(([id]) => id), DEFAULTS.force_en_key),
        correction_safety: normalizeChoice(cfg?.correction_safety, SAFETY_OPTIONS.map(([id]) => id), DEFAULTS.correction_safety),
        replace_words: normalizeNumber(cfg?.replace_words, 1, 3, DEFAULTS.replace_words),
        typing_assist_words: normalizeNumber(cfg?.typing_assist_words, 1, 3, DEFAULTS.typing_assist_words),
        multi_tap_max_taps: normalizeNumber(cfg?.multi_tap_max_taps, 2, 4, DEFAULTS.multi_tap_max_taps),
        tap_max_ms: normalizeNumber(cfg?.tap_max_ms, 100, 500, DEFAULTS.tap_max_ms),
        shift_window_ms: normalizeNumber(cfg?.shift_window_ms, 150, 600, DEFAULTS.shift_window_ms),
        lem_weight_percent: normalizeNumber(cfg?.lem_weight_percent, 0, 200, DEFAULTS.lem_weight_percent),
        nanda_l2_weight_percent: normalizeNumber(cfg?.nanda_l2_weight_percent, 0, 200, DEFAULTS.nanda_l2_weight_percent),
        nanda_l3_weight_percent: normalizeNumber(cfg?.nanda_l3_weight_percent, 0, 200, DEFAULTS.nanda_l3_weight_percent),
    };
}

function loadConfig() {
    try {
        const [, bytes] = Gio.File.new_for_path(CONFIG_PATH).load_contents(null);
        return normalizeConfig(JSON.parse(new TextDecoder().decode(bytes)));
    } catch(e) {
        return normalizeConfig(DEFAULTS);
    }
}

function saveConfig(cfg) {
    try {
        Gio.File.new_for_path(GLib.get_home_dir() + '/.config/lay').make_directory_with_parents(null);
    } catch(e) {}
    const bytes = new TextEncoder().encode(JSON.stringify(normalizeConfig(cfg), null, 2));
    Gio.File.new_for_path(CONFIG_PATH).replace_contents(
        bytes, null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);
}

function restartDaemon() {
    try {
        Gio.Subprocess.new(['systemctl', '--user', 'restart', 'lay-daemon'], Gio.SubprocessFlags.NONE);
    } catch(e) {}
}

function applyInputChannel(channel) {
    if (!['ime', 'uinput', 'auto'].includes(channel))
        return;
    try {
        Gio.Subprocess.new(
            [GLib.get_home_dir() + '/.local/bin/lay-runtime-control', 'channel', channel],
            Gio.SubprocessFlags.NONE
        );
    } catch(e) {}
}

class LayPrefsView {
    constructor() {
        this._cfg = loadConfig();
    }

    widget() {
        const root = new Gtk.Box({
            orientation: Gtk.Orientation.VERTICAL,
            spacing: 24,
            margin_top: 18,
            margin_bottom: 18,
            margin_start: 18,
            margin_end: 18,
        });

        const grid = new Gtk.Grid({
            column_spacing: 16,
            row_spacing: 16,
            column_homogeneous: true,
            vexpand: true,
        });
        grid.attach(this._section('Основное', [
            this._switchRow('Помощь при наборе', 'typing_assist', true),
            this._switchRow('Автоподмена', 'auto_replace', true),
            this._switchRow('Запоминать правки', 'learning_log', false),
            this._debugLogsRow('Журнал отладки lay'),
            this._switchRow('Автораскладка после пробела', 'auto_switch_layout', false),
            this._comboRow('Режим', 'correction_engine', ENGINE_OPTIONS, false),
            this._comboRow('Область', 'replace_words', SCOPE_OPTIONS, false),
            this._comboRow('Осторожность', 'correction_safety', SAFETY_OPTIONS, true),
        ]), 0, 0, 1, 1);

        grid.attach(this._section('Управление', [
            this._comboRow('Триггер', 'trigger', TRIGGER_OPTIONS, true),
            this._switchRow('Несколько нажатий Shift', 'multi_tap_scope', true),
            this._switchRow('Исправлять перед Enter', 'enter_autocorrect', true),
            this._switchRow('Хоткеи RU / EN', 'force_layout_hotkeys', true),
            this._comboRow('RU хоткей', 'force_ru_key', FORCE_KEY_OPTIONS, true),
            this._comboRow('EN хоткей', 'force_en_key', FORCE_KEY_OPTIONS, true),
        ]), 0, 1, 1, 1);

        grid.attach(this._section('Арбитры и каналы', [
            this._switchRow('Контур LEM', 'lem_enabled', false),
            this._switchRow('LEM: 2 слова', 'lem_2_words', false),
            this._switchRow('LEM: 3 слова', 'lem_3_words', false),
            this._weightRow('Вес LEM', 'lem_weight_percent', false),
            this._weightRow('Вес L2 кандидатов', 'nanda_l2_weight_percent', false),
            this._weightRow('Вес L3 фразы', 'nanda_l3_weight_percent', false),
            this._switchRow('Подсказки NANDA', 'nanda_precognition', false),
            this._switchRow('Подсказки в [скобках]', 'ime_bracket_candidates', false),
            this._switchRow('Автокоррекция NANDA', 'nanda_autocorrect', false),
            this._buttonRow('NANDA ячейки', 'Открыть', () => this._showNandaWindow()),
            this._switchRow('Раскладка по окну', 'ptah_alexs_mode', false),
            this._comboRow('Режим ввода', 'text_backend', BACKEND_OPTIONS, true),
            this._comboRow('Среда раскладки', 'layout_backend', LAYOUT_BACKEND_OPTIONS, true),
        ]), 1, 0, 1, 1);

        grid.attach(this._section('Тайминг', [
            this._spinRow('Тап', 'tap_max_ms', 'мс', 100, 500, 25, true),
            this._spinRow('Окно Shift', 'shift_window_ms', 'мс', 150, 600, 25, true),
            this._spinRow('Multi-tap максимум', 'multi_tap_max_taps', '', 2, 4, 1, true),
        ]), 1, 1, 1, 1);

        root.append(grid);
        root.append(this._aboutBox());
        return root;
    }

    _section(title, rows) {
        const box = new Gtk.Box({
            orientation: Gtk.Orientation.VERTICAL,
            spacing: 10,
            hexpand: true,
            vexpand: true,
        });
        box.append(new Gtk.Label({
            label: title,
            xalign: 0,
            css_classes: ['heading'],
        }));

        const list = new Gtk.ListBox({
            selection_mode: Gtk.SelectionMode.NONE,
            css_classes: ['boxed-list'],
        });
        for (const row of rows)
            list.append(row);
        box.append(list);
        return box;
    }

    _row(label, control) {
        const row = new Gtk.ListBoxRow({
            selectable: false,
            activatable: false,
        });
        const box = new Gtk.Box({
            orientation: Gtk.Orientation.HORIZONTAL,
            spacing: 12,
            margin_top: 8,
            margin_bottom: 8,
            margin_start: 10,
            margin_end: 10,
        });
        const text = new Gtk.Label({
            label,
            xalign: 0,
            hexpand: true,
            wrap: true,
        });
        control.halign = Gtk.Align.END;
        control.valign = Gtk.Align.CENTER;
        box.append(text);
        box.append(control);
        row.set_child(box);
        return row;
    }

    _switchRow(label, key, needsRestart) {
        const toggle = new Gtk.Switch({active: !!this._cfg[key]});
        toggle.connect('notify::active', () => {
            this._cfg[key] = toggle.active;
            saveConfig(this._cfg);
            if (needsRestart)
                restartDaemon();
        });
        return this._row(label, toggle);
    }

    _debugLogsRow(label) {
        const toggle = new Gtk.Switch({
            active: !!this._cfg.debug_action_log,
        });
        toggle.connect('notify::active', () => {
            this._cfg.debug_action_log = toggle.active;
            this._cfg.nanda_trace = toggle.active;
            this._cfg.nanda_trace_text = toggle.active;
            saveConfig(this._cfg);
        });
        return this._row(label, toggle);
    }

    _infoRow(label, value) {
        const text = new Gtk.Label({
            label: value,
            xalign: 0,
            wrap: true,
            max_width_chars: 34,
            css_classes: ['dim-label'],
        });
        return this._row(label, text);
    }

    _buttonRow(label, buttonLabel, callback) {
        const button = new Gtk.Button({label: buttonLabel});
        button.connect('clicked', callback);
        return this._row(label, button);
    }

    _comboRow(label, key, options, needsRestart) {
        const combo = new Gtk.ComboBoxText();
        let active = 0;
        const value = String(this._cfg[key]);
        for (let i = 0; i < options.length; i++) {
            const [id, text] = options[i];
            combo.append(id, text);
            if (String(id) === value)
                active = i;
        }
        combo.set_active(active);
        combo.connect('changed', () => {
            const id = combo.get_active_id();
            if (!id)
                return;
            this._cfg[key] = /^\d+$/.test(id) ? Number(id) : id;
            if (key === 'text_backend' && id === 'ime')
                this._cfg.nanda_precognition = true;
            saveConfig(this._cfg);
            if (key === 'text_backend')
                applyInputChannel(id);
            if (needsRestart)
                restartDaemon();
        });
        return this._row(label, combo);
    }

    _spinRow(label, key, suffix, min, max, step, needsRestart) {
        const spin = new Gtk.SpinButton({
            adjustment: new Gtk.Adjustment({
                lower: min,
                upper: max,
                step_increment: step,
                page_increment: step,
                value: Number(this._cfg[key] ?? min),
            }),
            numeric: true,
            width_chars: 4,
        });
        spin.connect('value-changed', () => {
            this._cfg[key] = spin.get_value_as_int();
            saveConfig(this._cfg);
            if (needsRestart)
                restartDaemon();
        });

        const box = new Gtk.Box({
            orientation: Gtk.Orientation.HORIZONTAL,
            spacing: 6,
        });
        box.append(spin);
        if (suffix) {
            box.append(new Gtk.Label({
                label: suffix,
                css_classes: ['dim-label'],
            }));
        }
        return this._row(label, box);
    }

    _weightRow(label, key, needsRestart) {
        const scale = Gtk.Scale.new_with_range(Gtk.Orientation.HORIZONTAL, 0, 200, 5);
        scale.set_value(Number(this._cfg[key] ?? DEFAULTS[key]));
        scale.set_digits(0);
        scale.set_draw_value(true);
        scale.set_value_pos(Gtk.PositionType.RIGHT);
        scale.set_size_request(190, -1);
        scale.hexpand = true;
        scale.connect('value-changed', () => {
            this._cfg[key] = Math.round(scale.get_value());
            saveConfig(this._cfg);
            if (needsRestart)
                restartDaemon();
        });
        return this._row(label, scale);
    }

    _nandaWavePanel(status) {
        const zones = status.zones ?? NANDA_WAVE_STATUS_FALLBACK.zones;
        const cells = Array.isArray(status.cells) ? status.cells : [];
        const area = new Gtk.DrawingArea({
            hexpand: true,
            height_request: 500,
        });
        area.set_content_width(620);
        area.set_content_height(500);
        area.set_draw_func((_area, cr, width, height) => {
            cr.setSourceRGBA(0.08, 0.09, 0.10, 0.04);
            cr.paint();

            const laneDefaults = {
                sensors: ['Сенсоры', 82, 0.16, 0.42, 0.85],
                candidates: ['Кандидаты', 240, 0.18, 0.68, 0.34],
                consensus: ['Согласование', 392, 0.86, 0.48, 0.20],
            };
            const laneMap = new Map();
            for (const zone of zones) {
                const fallback = laneDefaults[zone.id] ?? laneDefaults.consensus;
                laneMap.set(zone.id, [zone.label ?? fallback[0], fallback[1], fallback[2], fallback[3], fallback[4]]);
            }
            for (const [id, value] of Object.entries(laneDefaults)) {
                if (!laneMap.has(id))
                    laneMap.set(id, value);
            }
            const left = 190;
            const right = width - 24;
            const span = Math.max(1, right - left);

            cr.selectFontFace('Sans', 0, 0);
            for (const [name, y, r, g, b] of laneMap.values()) {
                cr.setSourceRGBA(r, g, b, 0.08);
                cr.rectangle(10, y - 66, width - 20, 132);
                cr.fill();
                cr.setSourceRGBA(r, g, b, 0.28);
                cr.setLineWidth(1);
                cr.rectangle(10, y - 66, width - 20, 132);
                cr.stroke();
                cr.setSourceRGBA(0.08, 0.08, 0.08, 0.82);
                cr.setFontSize(16);
                cr.moveTo(22, y - 44);
                cr.showText(name);
            }

            for (const cell of cells) {
                const zone = cell.zone ?? (cell.layer === 'L1' ? 'sensors' : cell.layer === 'L2' ? 'candidates' : 'consensus');
                const lane = (laneMap.get(zone) ?? laneMap.get('consensus'))[1];
                const peers = cells.filter(item => (item.zone ?? (item.layer === 'L1' ? 'sensors' : item.layer === 'L2' ? 'candidates' : 'consensus')) === zone);
                const index = Math.max(0, peers.findIndex(item => item.name === cell.name));
                const step = zone === 'sensors' ? 38 : zone === 'candidates' ? 48 : 42;
                const offset = (index - (peers.length - 1) / 2) * step;
                const y0 = lane + offset;
                const active = !!cell.alive || Number(cell.delta ?? 0) !== 0;
                cr.setSourceRGBA(active ? 0.06 : 0.28, active ? 0.44 : 0.32, active ? 0.88 : 0.38, active ? 0.86 : 0.52);
                cr.setLineWidth(active ? 2.2 : 1.2);
                for (let x = 0; x <= span; x++) {
                    const t = x / span;
                    const freq = cell.layer === 'L1' ? 7.0 : cell.layer === 'L2' ? 10.0 : 5.8;
                    const y = y0 + Math.sin(t * Math.PI * 2 * freq + Number(cell.phase ?? 0))   * (1.4 + Number(cell.amp ?? 0.25) * 1.8);
                    if (x === 0)
                        cr.moveTo(left + x, y);
                    else
                        cr.lineTo(left + x, y);
                }
                cr.stroke();

                cr.setSourceRGBA(0.06, 0.06, 0.06, 0.88);
                cr.setFontSize(12);
                cr.moveTo(left, y0 - 10);
                cr.showText(`${cell.label ?? cellVisualLabel(cell.name)} · ${Number(cell.delta ?? 0) === 0 ? 'след 0' : `живая ${cell.delta}`}`);
                cr.setFontSize(10);
                cr.setSourceRGBA(0.18, 0.18, 0.18, 0.70);
                cr.moveTo(left, y0 + 13);
                cr.showText(String(cell.role ?? ''));
            }

            const mid = height - 52;
            cr.setSourceRGBA(0.02, 0.02, 0.02, 0.88);
            cr.setLineWidth(3.0);
            for (let x = 0; x <= span; x++) {
                const t = x / span;
                let sum = 0;
                for (const cell of cells)
                    sum += Math.sin(t * Math.PI * 2 * (cell.layer === 'L2' ? 10.0 : cell.layer === 'L1' ? 7.0 : 5.8) + Number(cell.phase ?? 0)) * Number(cell.amp ?? 0.25);
                const y = mid + sum / Math.max(1, cells.length)  * 9;
                if (x === 0)
                    cr.moveTo(left + x, y);
                else
                    cr.lineTo(left + x, y);
            }
            cr.stroke();

            cr.setSourceRGBA(0.02, 0.02, 0.02, 0.88);
            cr.setFontSize(11);
            cr.moveTo(18, height - 50);
            cr.showText('несущая');
            cr.moveTo(18, height - 34);
            cr.showText('мода');
            cr.moveTo(18, height - 18);
            cr.showText('ансамбля');
        });
        return new Gtk.Frame({child: area});
    }

    _nandaPassportPanel(status) {
        const label = new Gtk.Label({
            label: `<tt>${GLib.markup_escape_text(nandaPassportText(status), -1)}</tt>`,
            use_markup: true,
            xalign: 0,
            wrap: false,
            selectable: true,
            css_classes: ['dim-label'],
        });
        const viewport = new Gtk.ScrolledWindow({
            hscrollbar_policy: Gtk.PolicyType.AUTOMATIC,
            vscrollbar_policy: Gtk.PolicyType.NEVER,
            min_content_height: 520,
        });
        viewport.set_child(label);
        return new Gtk.Frame({child: viewport});
    }

    _showNandaWindow() {
        if (this._nandaWindow) {
            this._nandaWindow.present();
            return;
        }

        const window = new Gtk.Window({
            title: 'NANDA',
            default_width: 520,
            default_height: 360,
        });
        this._nandaWindow = window;
        window.connect('close-request', () => {
            this._nandaWindow = null;
            return false;
        });

        const scroll = new Gtk.ScrolledWindow({
            hscrollbar_policy: Gtk.PolicyType.NEVER,
            vscrollbar_policy: Gtk.PolicyType.AUTOMATIC,
        });
        const inner = new Gtk.Box({
            orientation: Gtk.Orientation.VERTICAL,
            spacing: 12,
            margin_top: 16,
            margin_bottom: 16,
            margin_start: 16,
            margin_end: 16,
        });

        inner.append(new Gtk.Label({
            label: 'NANDA',
            xalign: 0,
            wrap: true,
            css_classes: ['heading'],
        }));
        inner.append(new Gtk.Label({
            label: 'Экспериментальный локальный слой автокоррекции. NANDA смотрит на хвост ввода, рождает варианты исправления и пропускает их через защитные проверки перед заменой текста.',
            xalign: 0,
            wrap: true,
            max_width_chars: 58,
            css_classes: ['dim-label'],
        }));
        inner.append(new Gtk.Label({label: 'Как использовать', xalign: 0, css_classes: ['heading']}));
        inner.append(new Gtk.Label({
            label: 'Включи “Автокоррекция NANDA”, если хочешь тестировать этот слой в живом вводе. “Журнал отладки lay” нужен только для разбора ошибок.',
            xalign: 0,
            wrap: true,
            max_width_chars: 58,
            css_classes: ['dim-label'],
        }));
        inner.append(new Gtk.Label({label: 'Важно', xalign: 0, css_classes: ['heading']}));
        inner.append(new Gtk.Label({
            label: 'NANDA не печатает напрямую в окна и не является внешней LLM. Она только помогает выбрать исправление; сама вставка всё равно идёт через безопасный pipeline lay.',
            xalign: 0,
            wrap: true,
            max_width_chars: 58,
            css_classes: ['dim-label'],
        }));
        scroll.set_child(inner);
        window.set_child(scroll);
        window.present();
    }

    _aboutBox() {
        const box = new Gtk.Box({
            orientation: Gtk.Orientation.VERTICAL,
            spacing: 10,
        });
        box.append(new Gtk.Label({
            label: 'О программе',
            xalign: 0,
            css_classes: ['heading'],
        }));

        const inner = new Gtk.Box({
            orientation: Gtk.Orientation.VERTICAL,
            spacing: 6,
            margin_top: 10,
            margin_bottom: 10,
            margin_start: 12,
            margin_end: 12,
        });
        inner.append(new Gtk.Label({
            label: `Lay ${APP_VERSION}`,
            xalign: 0,
            css_classes: ['heading'],
        }));
        inner.append(new Gtk.Label({
            label: `Дата версии: ${APP_RELEASE_DATE}. RU/EN-переключатель для GNOME, KDE, Wayland и X11.`,
            xalign: 0,
            wrap: true,
            css_classes: ['dim-label'],
        }));
        inner.append(new Gtk.LinkButton({
            uri: APP_URL,
            label: APP_URL,
            halign: Gtk.Align.START,
        }));

        box.append(new Gtk.Frame({child: inner}));
        return box;
    }
}

export default class LayPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        const page = new Adw.PreferencesPage();
        const group = new Adw.PreferencesGroup();
        group.set_header_suffix(new Gtk.Image({
            icon_name: APP_ICON_NAME,
            pixel_size: HEADER_ICON_SIZE,
        }));
        group.add(new LayPrefsView().widget());
        page.add(group);
        window.add(page);
        window.set_title('Lay');
        window.set_icon_name?.(APP_ICON_NAME);
        window.set_default_size(800, 640);
    }
}
