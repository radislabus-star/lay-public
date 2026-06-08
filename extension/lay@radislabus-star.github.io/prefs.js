import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk';

import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

const CONFIG_PATH = GLib.get_home_dir() + '/.config/lay/config.json';
const APP_VERSION = '0.1.218';
const APP_RELEASE_DATE = '2026-06-07';
const APP_URL = 'https://github.com/radislabus-star/lay-public';
const APP_ICON_NAME = 'input-keyboard-symbolic';
const HEADER_ICON_SIZE = 16;

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
    ['ime', 'IME'],
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
    lem_2_words: true,
    lem_3_words: true,
    ptah_alexs_mode: false,
    ptah_alexs_rules: [],
    learning_log: false,
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
    return {
        ...DEFAULTS,
        ...cfg,
        mode: 'simple',
        correction_engine: normalizeChoice(cfg?.correction_engine, ['replay', 'smart'], DEFAULTS.correction_engine),
        layout_backend: normalizeChoice(cfg?.layout_backend, LAYOUT_BACKEND_OPTIONS.map(([id]) => id), DEFAULTS.layout_backend),
        text_backend: normalizeChoice(cfg?.text_backend, BACKEND_OPTIONS.map(([id]) => id), DEFAULTS.text_backend),
        trigger: normalizeChoice(cfg?.trigger, TRIGGER_OPTIONS.map(([id]) => id), DEFAULTS.trigger),
        force_ru_key: normalizeChoice(cfg?.force_ru_key, FORCE_KEY_OPTIONS.map(([id]) => id), DEFAULTS.force_ru_key),
        force_en_key: normalizeChoice(cfg?.force_en_key, FORCE_KEY_OPTIONS.map(([id]) => id), DEFAULTS.force_en_key),
        correction_safety: normalizeChoice(cfg?.correction_safety, SAFETY_OPTIONS.map(([id]) => id), DEFAULTS.correction_safety),
        replace_words: normalizeNumber(cfg?.replace_words, 1, 3, DEFAULTS.replace_words),
        typing_assist_words: normalizeNumber(cfg?.typing_assist_words, 1, 3, DEFAULTS.typing_assist_words),
        multi_tap_max_taps: normalizeNumber(cfg?.multi_tap_max_taps, 2, 4, DEFAULTS.multi_tap_max_taps),
        tap_max_ms: normalizeNumber(cfg?.tap_max_ms, 100, 500, DEFAULTS.tap_max_ms),
        shift_window_ms: normalizeNumber(cfg?.shift_window_ms, 150, 600, DEFAULTS.shift_window_ms),
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
        ]), 1, 0, 1, 1);

        grid.attach(this._section('Арбитры и каналы', [
            this._switchRow('LEM: 2 слова', 'lem_2_words', false),
            this._switchRow('LEM: 3 слова', 'lem_3_words', false),
            this._switchRow('Раскладка по окну', 'ptah_alexs_mode', false),
            this._comboRow('Канал ввода', 'text_backend', BACKEND_OPTIONS, true),
            this._comboRow('Среда раскладки', 'layout_backend', LAYOUT_BACKEND_OPTIONS, true),
        ]), 0, 1, 1, 1);

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
            saveConfig(this._cfg);
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
