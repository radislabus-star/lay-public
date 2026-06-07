import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk';

const CONFIG_PATH = GLib.get_home_dir() + '/.config/lay/config.json';
const APP_VERSION = '0.1.213';
const APP_RELEASE_DATE = '2026-06-05';
const APP_URL = 'https://github.com/radislabus-star/lay-public';

const OPTIONS = {
    correction_engine: [['replay', 'Replay'], ['smart', 'Smart']],
    replace_words: [['1', '1 слово'], ['2', '2 слова'], ['3', '3 слова']],
    correction_safety: [['strict', 'Осторожно'], ['normal', 'Норма'], ['experimental', 'Смелее']],
    trigger: [
        ['double-lshift', 'Double Shift'],
        ['double-ctrl', 'Ctrl x2'],
        ['double-alt', 'Alt x2'],
        ['caps-lock', 'CapsLock'],
        ['single-rshift', 'RShift'],
        ['single-rctrl', 'RCtrl'],
        ['single-ralt', 'RAlt'],
        ['single-pause', 'Pause'],
    ],
    force_key: [
        ['single-rctrl', 'RCtrl'],
        ['single-ralt', 'RAlt'],
        ['single-rshift', 'RShift'],
        ['single-pause', 'Pause'],
        ['caps-lock', 'CapsLock'],
    ],
    text_backend: [['uinput', 'uinput'], ['ime', 'IME'], ['auto', 'auto']],
    layout_backend: [['auto', 'auto'], ['gnome', 'GNOME'], ['kde', 'KDE'], ['x11', 'X11']],
};

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
    replace_words: 1,
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

function choice(value, allowed, fallback) {
    return allowed.includes(value) ? value : fallback;
}

function number(value, min, max, fallback) {
    const n = Number(value);
    return Number.isFinite(n) ? Math.max(min, Math.min(max, n)) : fallback;
}

function normalize(cfg) {
    return {
        ...DEFAULTS,
        ...cfg,
        mode: 'simple',
        correction_engine: choice(cfg?.correction_engine, OPTIONS.correction_engine.map(([id]) => id), DEFAULTS.correction_engine),
        layout_backend: choice(cfg?.layout_backend, OPTIONS.layout_backend.map(([id]) => id), DEFAULTS.layout_backend),
        text_backend: choice(cfg?.text_backend, OPTIONS.text_backend.map(([id]) => id), DEFAULTS.text_backend),
        trigger: choice(cfg?.trigger, OPTIONS.trigger.map(([id]) => id), DEFAULTS.trigger),
        force_ru_key: choice(cfg?.force_ru_key, OPTIONS.force_key.map(([id]) => id), DEFAULTS.force_ru_key),
        force_en_key: choice(cfg?.force_en_key, OPTIONS.force_key.map(([id]) => id), DEFAULTS.force_en_key),
        correction_safety: choice(cfg?.correction_safety, OPTIONS.correction_safety.map(([id]) => id), DEFAULTS.correction_safety),
        replace_words: number(cfg?.replace_words, 1, 3, DEFAULTS.replace_words),
        multi_tap_max_taps: number(cfg?.multi_tap_max_taps, 2, 4, DEFAULTS.multi_tap_max_taps),
        tap_max_ms: number(cfg?.tap_max_ms, 100, 500, DEFAULTS.tap_max_ms),
        shift_window_ms: number(cfg?.shift_window_ms, 150, 600, DEFAULTS.shift_window_ms),
    };
}

function loadConfig() {
    try {
        const [, bytes] = Gio.File.new_for_path(CONFIG_PATH).load_contents(null);
        return normalize(JSON.parse(new TextDecoder().decode(bytes)));
    } catch(e) {
        return normalize(DEFAULTS);
    }
}

function saveConfig(cfg) {
    try {
        Gio.File.new_for_path(GLib.get_home_dir() + '/.config/lay').make_directory_with_parents(null);
    } catch(e) {}
    const bytes = new TextEncoder().encode(JSON.stringify(normalize(cfg), null, 2));
    Gio.File.new_for_path(CONFIG_PATH).replace_contents(
        bytes, null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);
}

function restartDaemon() {
    try {
        Gio.Subprocess.new(['systemctl', '--user', 'restart', 'lay-daemon'], Gio.SubprocessFlags.NONE);
    } catch(e) {}
}

function optionRow(label, control) {
    const row = new Gtk.ListBoxRow({selectable: false, activatable: false});
    const box = new Gtk.Box({
        orientation: Gtk.Orientation.HORIZONTAL,
        spacing: 12,
        margin_top: 8,
        margin_bottom: 8,
        margin_start: 10,
        margin_end: 10,
    });
    box.append(new Gtk.Label({label, xalign: 0, hexpand: true, wrap: true}));
    control.halign = Gtk.Align.END;
    control.valign = Gtk.Align.CENTER;
    box.append(control);
    row.set_child(box);
    return row;
}

class SettingsView {
    constructor() {
        this.cfg = loadConfig();
    }

    build() {
        const root = new Gtk.Box({
            orientation: Gtk.Orientation.VERTICAL,
            spacing: 4,
            margin_top: 18,
            margin_bottom: 18,
            margin_start: 18,
            margin_end: 18,
        });
        const grid = new Gtk.Grid({
            column_spacing: 16,
            row_spacing: 10,
            column_homogeneous: true,
            vexpand: true,
        });

        grid.attach(this.section('Основное', [
            this.switchRow('Помощь при наборе', 'typing_assist', true),
            this.switchRow('Автоподмена', 'auto_replace', true),
            this.switchRow('Запоминать правки', 'learning_log', false),
            this.switchRow('Авто-layout после пробела', 'auto_switch_layout', false),
            this.comboRow('Режим', 'correction_engine', OPTIONS.correction_engine, false),
            this.comboRow('Область', 'replace_words', OPTIONS.replace_words, false),
            this.comboRow('Осторожность', 'correction_safety', OPTIONS.correction_safety, true),
        ]), 0, 0, 1, 1);
        grid.attach(this.section('Управление', [
            this.comboRow('Триггер', 'trigger', OPTIONS.trigger, true),
            this.switchRow('Multi-tap scope', 'multi_tap_scope', true),
            this.switchRow('Исправлять перед Enter', 'enter_autocorrect', true),
            this.switchRow('Хоткеи RU / EN', 'force_layout_hotkeys', true),
            this.comboRow('RU хоткей', 'force_ru_key', OPTIONS.force_key, true),
            this.comboRow('EN хоткей', 'force_en_key', OPTIONS.force_key, true),
        ]), 1, 0, 1, 1);
        grid.attach(this.section('Тайминг', [
            this.spinRow('Тап', 'tap_max_ms', 'мс', 100, 500, 25, true),
            this.spinRow('Окно Shift', 'shift_window_ms', 'мс', 150, 600, 25, true),
            this.spinRow('Multi-tap максимум', 'multi_tap_max_taps', '', 2, 4, 1, true),
        ]), 0, 1, 1, 1);
        grid.attach(this.section('Арбитры и каналы', [
            this.switchRow('LEM: 2 слова', 'lem_2_words', false),
            this.switchRow('LEM: 3 слова', 'lem_3_words', false),
            this.switchRow('Раскладка по окну', 'ptah_alexs_mode', false),
            this.comboRow('Канал ввода', 'text_backend', OPTIONS.text_backend, true),
            this.comboRow('Desktop backend', 'layout_backend', OPTIONS.layout_backend, true),
        ]), 1, 1, 1, 1);

        root.append(grid);
        root.append(this.about());
        return root;
    }

    section(title, rows) {
        const box = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL, spacing: 10, hexpand: true});
        box.append(new Gtk.Label({label: title, xalign: 0, css_classes: ['heading']}));
        const list = new Gtk.ListBox({selection_mode: Gtk.SelectionMode.NONE, css_classes: ['boxed-list']});
        for (const row of rows)
            list.append(row);
        box.append(list);
        return box;
    }

    switchRow(label, key, needsRestart) {
        const sw = new Gtk.Switch({active: !!this.cfg[key]});
        sw.connect('notify::active', () => {
            this.cfg[key] = sw.active;
            saveConfig(this.cfg);
            if (needsRestart)
                restartDaemon();
        });
        return optionRow(label, sw);
    }

    comboRow(label, key, options, needsRestart) {
        const combo = new Gtk.ComboBoxText();
        const current = String(this.cfg[key]);
        let active = 0;
        options.forEach(([id, text], idx) => {
            combo.append(id, text);
            if (String(id) === current)
                active = idx;
        });
        combo.set_active(active);
        combo.connect('changed', () => {
            const id = combo.get_active_id();
            this.cfg[key] = /^\d+$/.test(id) ? Number(id) : id;
            saveConfig(this.cfg);
            if (needsRestart)
                restartDaemon();
        });
        return optionRow(label, combo);
    }

    spinRow(label, key, suffix, min, max, step, needsRestart) {
        const spin = new Gtk.SpinButton({
            adjustment: new Gtk.Adjustment({
                lower: min,
                upper: max,
                step_increment: step,
                page_increment: step,
                value: Number(this.cfg[key] ?? min),
            }),
            numeric: true,
            width_chars: 4,
        });
        spin.connect('value-changed', () => {
            this.cfg[key] = spin.get_value_as_int();
            saveConfig(this.cfg);
            if (needsRestart)
                restartDaemon();
        });
        const box = new Gtk.Box({orientation: Gtk.Orientation.HORIZONTAL, spacing: 6});
        box.append(spin);
        if (suffix)
            box.append(new Gtk.Label({label: suffix, css_classes: ['dim-label']}));
        return optionRow(label, box);
    }

    about() {
        const box = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL, spacing: 10});
        box.hexpand = true;
        box.append(new Gtk.Label({label: 'О программе', xalign: 0, css_classes: ['heading']}));
        const inner = new Gtk.Box({
            orientation: Gtk.Orientation.VERTICAL,
            spacing: 6,
            hexpand: true,
            margin_top: 10,
            margin_bottom: 10,
            margin_start: 12,
            margin_end: 12,
        });
        const title = new Gtk.Label({label: `Lay ${APP_VERSION}`, xalign: 0, css_classes: ['heading']});
        title.hexpand = true;
        title.wrap = true;
        title.max_width_chars = 48;
        inner.append(title);

        const version = new Gtk.Label({
            label: `Версия: ${APP_VERSION} от ${APP_RELEASE_DATE}`,
            xalign: 0,
            hexpand: true,
            wrap: true,
            max_width_chars: 48,
            css_classes: ['dim-label'],
        });
        inner.append(version);

        const description = new Gtk.Label({
            label: 'RU/EN layout helper',
            xalign: 0,
            hexpand: true,
            wrap: true,
            max_width_chars: 48,
            css_classes: ['dim-label'],
        });
        inner.append(description);

        const link = new Gtk.LinkButton({
            uri: APP_URL,
            label: 'GitHub',
            halign: Gtk.Align.START,
        });
        link.hexpand = false;
        inner.append(link);

        const frame = new Gtk.Frame({child: inner});
        frame.hexpand = true;
        box.append(frame);
        return box;
    }
}

const app = new Adw.Application({
    application_id: 'io.github.radislabus_star.LaySettings',
    flags: Gio.ApplicationFlags.NON_UNIQUE,
});

app.connect('activate', () => {
    const win = new Adw.ApplicationWindow({
        application: app,
        title: 'Lay',
        default_width: 800,
        default_height: 980,
    });

    const toolbar = new Adw.ToolbarView();
    toolbar.add_top_bar(new Adw.HeaderBar({
        title_widget: new Gtk.Label({
            label: 'Lay',
            css_classes: ['heading'],
        }),
    }));
    toolbar.set_content(new Gtk.ScrolledWindow({
        hscrollbar_policy: Gtk.PolicyType.NEVER,
        vscrollbar_policy: Gtk.PolicyType.AUTOMATIC,
        child: new SettingsView().build(),
    }));
    win.set_content(toolbar);
    win.present();
});

app.run([]);
