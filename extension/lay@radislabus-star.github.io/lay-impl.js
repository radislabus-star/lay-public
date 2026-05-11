/* lay-impl.js — реализация extension (меняется свободно, без logout)
 * Загружается через loader в extension.js с уникальным URL → нет кэша GJS.
 */

import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import Pango from 'gi://Pango';
import Shell from 'gi://Shell';
import St from 'gi://St';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {getInputSourceManager} from 'resource:///org/gnome/shell/ui/status/keyboard.js';

// ─── Config ────────────────────────────────────────────────

const CONFIG_PATH = GLib.get_home_dir() + '/.config/lay/config.json';
const STATS_PATH = GLib.get_home_dir() + '/.local/share/lay/stats.json';
const APP_VERSION = '0.1.136';
const APP_DESCRIPTION = 'Double Shift layout rescue for Linux/GNOME Wayland';
const APP_RELEASE_DATE = '2026-05-12';
const APP_LICENSE = 'MIT';
const APP_URL = 'https://github.com/radislabus-star/lay-public';
const APP_PLATFORM = 'GNOME Wayland';
const APP_GNOME_SUPPORT = 'GNOME 45-47, 50';
const MENU_WIDTH = 360;
const COMPACT_SUBTITLE_STYLE = 'font-weight:normal; font-size:76%; opacity:180;';
const SEGMENT_BUTTON_STYLE = 'padding:2px 8px; border-radius:6px; min-width:0;';
const LEARNING_LOG_TOOLTIP = 'Запоминать правки работает в два слоя:\n'
    + '• double-Shift пишет факт ручного исправления;\n'
    + '• после auto/smart lay ждёт до 30 секунд,\n'
    + '  удалишь ли ты результат и введёшь свой вариант.\n'
    + 'Если удалил и перепечатал — это считается твоей правкой.';
const AUTO_REPLACE_TOOLTIP = 'Когда включено: typo-правки после пробела и точные автоподмены.\n'
    + 'Когда выключено: остаётся только безопасный авто-layout EN/RU после пробела.';
const AUTO_SWITCH_TOOLTIP = 'После автоматической помощи при наборе lay оставляет активной\n'
    + 'раскладку исправленного текста. Double Shift переключает раскладку всегда.';
const LEM_2_TOOLTIP = 'LEM-арбитр для двух слов: сравнивает готовые варианты хвоста\n'
    + 'и выбирает более естественный, не генерируя новый текст.';
const LEM_3_TOOLTIP = 'LEM-арбитр для трех слов и длиннее: нужен для смешанных RU/EN\n'
    + 'фраз, где соседние слова помогают понять раскладку.';
const PTAH_ALEXS_TOOLTIP = 'Жёсткая раскладка по окну: при фокусе окна lay ставит\n'
    + 'заданную раскладку, а не вспоминает последнюю случайную.';
const PTAH_RULE_LIMIT = 80;
const TYPING_RULES = [
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
const DEFAULT_TYPING_PIPELINE = TYPING_RULES.map((rule, idx) => ({
    id: rule.id,
    enabled: true,
    priority: (idx + 1) * 10,
}));
const DEFAULTS = {
    mode: 'simple',
    correction_engine: 'replay',
    layout_backend: 'auto',
    trigger: 'double-lshift',
    tap_max_ms: 200,
    shift_window_ms: 250,
    debounce_ms: 50,
    replace_words: 1,
    auto_replace: false,
    typing_assist: false,
    auto_switch_layout: true,
    lem_2_words: true,
    lem_3_words: true,
    ptah_alexs_mode: false,
    ptah_alexs_rules: [],
    typing_assist_pipeline: DEFAULT_TYPING_PIPELINE,
    learning_log: false,
};

function loadConfig() {
    try {
        const [, bytes] = Gio.File.new_for_path(CONFIG_PATH).load_contents(null);
        const parsed = JSON.parse(new TextDecoder().decode(bytes));
        const cfg = {...DEFAULTS, ...parsed};
        if (parsed.correction_engine === undefined)
            cfg.correction_engine = parsed.mode === 'llm' ? 'smart' : 'replay';
        cfg.typing_assist_pipeline = normalizeTypingPipeline(parsed.typing_assist_pipeline);
        cfg.ptah_alexs_mode = !!cfg.ptah_alexs_mode;
        cfg.ptah_alexs_rules = normalizePtahRules(parsed.ptah_alexs_rules);
        return cfg;
    } catch(e) {
        return {
            ...DEFAULTS,
            typing_assist_pipeline: normalizeTypingPipeline(DEFAULT_TYPING_PIPELINE),
            ptah_alexs_rules: [],
        };
    }
}
function saveConfig(cfg) {
    try { Gio.File.new_for_path(GLib.get_home_dir() + '/.config/lay').make_directory_with_parents(null); } catch(e) {}
    cfg.typing_assist_pipeline = normalizeTypingPipeline(cfg.typing_assist_pipeline);
    cfg.ptah_alexs_rules = normalizePtahRules(cfg.ptah_alexs_rules);
    const bytes = new TextEncoder().encode(JSON.stringify(cfg, null, 2));
    Gio.File.new_for_path(CONFIG_PATH).replace_contents(
        bytes, null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);
}
function normalizePtahLayout(layout) {
    const id = String(layout ?? '').trim().toLowerCase();
    if (['ru', 'rus', 'russian'].includes(id))
        return 'ru';
    if (['us', 'en', 'eng', 'english'].includes(id))
        return 'us';
    if (id === 'keep')
        return 'keep';
    return '';
}
function normalizePtahRules(saved) {
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
function normalizeTypingPipeline(saved) {
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
function typingRuleLabel(id) {
    return TYPING_RULES.find(rule => rule.id === id)?.label ?? id;
}
function loadStats() {
    try {
        const [, bytes] = Gio.File.new_for_path(STATS_PATH).load_contents(null);
        return JSON.parse(new TextDecoder().decode(bytes));
    } catch(e) {
        return {};
    }
}
function restartDaemon() {
    daemonCommand('restart');
}
function startDaemon() {
    daemonCommand('start');
}
function stopDaemon() {
    daemonCommand('stop');
}
function daemonCommand(action) {
    try { Gio.Subprocess.new(['systemctl','--user',action,'lay-daemon'], Gio.SubprocessFlags.NONE); } catch(e) {}
}
function openUri(uri) {
    try {
        Gio.AppInfo.launch_default_for_uri(uri, global.create_app_launch_context(0, -1));
    } catch(e) {
        try { Gio.Subprocess.new(['xdg-open', uri], Gio.SubprocessFlags.NONE); } catch(_e) {}
    }
}
function normalizeLayoutKind(id) {
    const value = String(id ?? '').trim().toLowerCase();
    if (!value)
        return '';
    if (value === 'ru' || value.includes(':ru') || value.includes('russian') || value.includes('rus'))
        return 'ru';
    if (value === 'us' || value === 'en' || value.includes(':us') || value.includes('english') || value.includes('eng'))
        return 'us';
    return value;
}
function currentLayoutKind() {
    try {
        return normalizeLayoutKind(getInputSourceManager().currentSource?.id ?? '');
    } catch(e) {
        return '';
    }
}
function activateLayoutId(id) {
    try {
        const mgr = getInputSourceManager();
        for (const i in mgr.inputSources)
            if (mgr.inputSources[i].id === id) {
                mgr.inputSources[i].activate();
                return true;
            }

        const targetKind = normalizeLayoutKind(id);
        for (const i in mgr.inputSources)
            if (normalizeLayoutKind(mgr.inputSources[i].id) === targetKind) {
                mgr.inputSources[i].activate();
                return true;
            }
    } catch(e) {}
    return false;
}
function focusedWindow() {
    try {
        return global.display.focus_window ?? global.display.get_focus_window?.() ?? null;
    } catch(e) {
        return null;
    }
}
function focusedWindowInfo() {
    const win = focusedWindow();
    if (!win)
        return null;

    let app = null;
    try { app = Shell.WindowTracker.get_default().get_window_app(win); } catch(e) {}
    const appId = String(app?.get_id?.() ?? '').trim();
    const appName = String(app?.get_name?.() ?? '').trim();
    const wmClass = String(win.get_wm_class?.() ?? '').trim();
    const wmClassInstance = String(win.get_wm_class_instance?.() ?? '').trim();

    if (appId)
        return {kind: 'app_id', value: appId, label: appName || appId, appId, wmClass, wmClassInstance};
    if (wmClass)
        return {kind: 'wm_class', value: wmClass, label: wmClass, appId, wmClass, wmClassInstance};
    if (wmClassInstance)
        return {kind: 'wm_class_instance', value: wmClassInstance, label: wmClassInstance, appId, wmClass, wmClassInstance};
    return null;
}

// ─── DBus ──────────────────────────────────────────────────

const DBUS_XML = `
<node>
  <interface name="io.github.radislabus_star.LayDaemon">
    <method name="Ping"><arg name="reply" direction="out" type="s"/></method>
    <method name="TypeText"><arg name="text" direction="in" type="s"/></method>
    <method name="ActivateLayout">
      <arg name="id" direction="in" type="s"/>
      <arg name="success" direction="out" type="b"/>
    </method>
    <method name="CurrentLayout"><arg name="id" direction="out" type="s"/></method>
    <method name="NextLayout"><arg name="success" direction="out" type="b"/></method>
    <method name="ListLayouts"><arg name="layouts" direction="out" type="s"/></method>
  </interface>
</node>`;

const DBUS_PATH = '/io/github/radislabus_star/LayDaemon';

class LayDaemonService {
    enable() {
        const seat = Clutter.get_default_backend().get_default_seat();
        this._vdev = seat.create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
        this._dbus = Gio.DBusExportedObject.wrapJSObject(DBUS_XML, this);
        this._dbus.export(Gio.DBus.session, DBUS_PATH);
        log('[lay-extension] DBus enabled');
    }
    disable() {
        this._dbus?.unexport(); this._dbus = null; this._vdev = null;
        log('[lay-extension] DBus disabled');
    }
    Ping() { return 'pong from lay-extension'; }
    TypeText(text) {
        if (Main.inputMethod?.commit) { try { Main.inputMethod.commit(text); return; } catch(e) {} }
        this._typeTextByKeyvals(text);
    }
    _tapKeyval(keyval, count) {
        for (let i = 0; i < count; i++) {
            this._vdev?.notify_keyval(Clutter.CURRENT_TIME, keyval, Clutter.KeyState.PRESSED);
            this._vdev?.notify_keyval(Clutter.CURRENT_TIME, keyval, Clutter.KeyState.RELEASED);
        }
    }
    _typeTextByKeyvals(text) {
        for (const ch of text) {
            const kv = Clutter.unicode_to_keysym(ch.codePointAt(0));
            if (!kv) continue;
            this._tapKeyval(kv, 1);
        }
    }
    ActivateLayout(id) {
        return activateLayoutId(id);
    }
    CurrentLayout() {
        try {
            return getInputSourceManager().currentSource?.id ?? '';
        } catch(e) { return ''; }
    }
    NextLayout() {
        try {
            const mgr = getInputSourceManager();
            const ids = Object.keys(mgr.inputSources).sort((a,b)=>a-b);
            const cur = ids.findIndex(i => mgr.inputSources[i].id === mgr.currentSource.id);
            mgr.inputSources[ids[(cur+1)%ids.length]].activate();
            return true;
        } catch(e) { return false; }
    }
    ListLayouts() {
        try {
            const mgr = getInputSourceManager();
            return Object.keys(mgr.inputSources).sort((a,b)=>a-b)
                .map(i=>`${i}:${mgr.inputSources[i].type}:${mgr.inputSources[i].id}${mgr.inputSources[i].id===mgr.currentSource.id?'*':''}`)
                .join(',');
        } catch(e) { return 'error:'+e; }
    }
}

// ─── Tray Indicator ────────────────────────────────────────
// Уникальный GTypeName предотвращает ошибку "already registered"
// при повторном disable→enable в одной сессии.

const _uid = Date.now();

const LayIndicator = GObject.registerClass(
{GTypeName: `LayIndicator_${_uid}`},
class LayIndicator extends PanelMenu.Button {

    _init() {
        super._init(0.0, 'lay');
        this._cfg = loadConfig();
        this._cfg.replace_words = Math.max(1, Math.min(3, this._cfg.replace_words));
        this._cfg.correction_engine = this._cfg.correction_engine === 'smart' ? 'smart' : 'replay';

        this._panelBox = new St.BoxLayout({
            style: 'spacing:4px; padding:0 2px;',
        });
        this._panelIcon = new St.Icon({
            icon_name: 'input-keyboard-symbolic',
            style_class: 'system-status-icon',
            y_align: Clutter.ActorAlign.CENTER,
        });
        this._label = new St.Label({
            text: '--',
            y_align: Clutter.ActorAlign.CENTER,
            style: 'font-weight:bold; padding:0 2px;',
        });
        this._panelBox.add_child(this._panelIcon);
        this._panelBox.add_child(this._label);
        this.add_child(this._panelBox);

        this._buildMenu();
        this.menu.connect('open-state-changed', (_menu, isOpen) => {
            if (isOpen)
                this._refreshStats();
        });

        this._mgr = getInputSourceManager();
        this._srcId = this._mgr.connect('current-source-changed', () => this._refreshLayout());
        this._focusId = global.display.connect('notify::focus-window', () => this._onFocusWindowChanged());
        this._refreshLayout();
        this._schedulePtahApply(80);
    }

    _buildMenu() {
        this.menu.box.style = `min-width:${MENU_WIDTH}px; padding:2px 0;`;
        this._engineButtons = {};
        this._scopeButtons = {};
        this._triggerButtons = {};
        this._triggerItems = {};
        this._toggleButtons = {};
        this._statusRefreshIds = [];
        this._cfg.typing_assist_pipeline = normalizeTypingPipeline(this._cfg.typing_assist_pipeline);

        this._statusItem = this._headerItem();
        this.menu.addMenuItem(this._statusItem);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        this.menu.addMenuItem(this._switchItem('Помощь при наборе', 'typing_assist', true));
        this.menu.addMenuItem(this._switchItem(
            'Автоподмена',
            'auto_replace',
            true,
            AUTO_REPLACE_TOOLTIP
        ));
        this.menu.addMenuItem(this._switchItem(
            'Запоминать правки',
            'learning_log',
            false,
            LEARNING_LOG_TOOLTIP
        ));
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        this.menu.addMenuItem(this._segmentedRow('Режим', [
            ['replay', 'Replay', () => {
                this._cfg.correction_engine = 'replay';
                this._cfg.mode = 'simple';
                this._saveAndRefresh();
            }],
            ['smart', 'Smart', () => {
                this._cfg.correction_engine = 'smart';
                this._cfg.mode = 'simple';
                this._saveAndRefresh();
            }],
        ], this._engineButtons));

        this.menu.addMenuItem(this._segmentedRow('Область', [
            ['1', '1 слово', () => {
                this._cfg.replace_words = 1;
                this._saveAndRefresh();
            }],
            ['2', '2 слова', () => {
                this._cfg.replace_words = 2;
                this._saveAndRefresh();
            }],
            ['3', '3 слова', () => {
                this._cfg.replace_words = 3;
                this._saveAndRefresh();
            }],
        ], this._scopeButtons));

        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        this.menu.addMenuItem(this._arbiterMenu());
        this.menu.addMenuItem(this._ptahAlexsMenu());
        this.menu.addMenuItem(this._correctionPipelineMenu());
        this.menu.addMenuItem(this._triggerMenu());
        this.menu.addMenuItem(this._timingMenu());
        this.menu.addMenuItem(this._daemonSwitchItem());
        this.menu.addMenuItem(this._aboutMenu());

        this._refreshSelections();
        this._refreshStatus();
    }

    _headerItem() {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:5px 12px 4px 12px;';
        const card = new St.BoxLayout({
            x_expand: true,
            style: 'spacing:8px;',
        });
        const icon = new St.Icon({
            icon_name: 'input-keyboard-symbolic',
            y_align: Clutter.ActorAlign.CENTER,
            style_class: 'popup-menu-icon',
        });
        const titleBox = new St.BoxLayout({x_expand: true, style: 'spacing:6px;'});
        const title = new St.Label({
            text: `Lay ${APP_VERSION}`,
            y_align: Clutter.ActorAlign.CENTER,
            x_expand: true,
            style: 'font-weight:bold;',
        });
        this._statusLabel = new St.Label({
            text: 'проверка...',
            y_align: Clutter.ActorAlign.CENTER,
            style: COMPACT_SUBTITLE_STYLE,
        });
        this._statusDot = new St.Label({
            text: '●',
            y_align: Clutter.ActorAlign.CENTER,
            style: 'font-size:90%; color:#f6c343;',
        });
        titleBox.add_child(title);
        titleBox.add_child(this._statusLabel);
        card.add_child(icon);
        card.add_child(titleBox);
        card.add_child(this._statusDot);
        item.add_child(card);
        return item;
    }

    _switchItem(label, key, restart = false, tooltip = null) {
        const item = new PopupMenu.PopupSwitchMenuItem(label, !!this._cfg[key], {});
        item.connect('toggled', (_item, state) => {
            this._cfg[key] = state;
            this._saveAndRefresh();
            if (restart) {
                restartDaemon();
                this._setDaemonBusy('restarting...');
                this._scheduleStatusRefreshes();
            }
        });
        if (tooltip)
            this._attachTooltip(item, tooltip);
        this._toggleButtons[key] = item;
        return item;
    }

    _arbiterMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Арбитр', false);
        item.menu.addMenuItem(this._switchItem(
            'Авто-layout после пробела',
            'auto_switch_layout',
            false,
            AUTO_SWITCH_TOOLTIP
        ));
        item.menu.addMenuItem(this._switchItem(
            'LEM: 2 слова',
            'lem_2_words',
            false,
            LEM_2_TOOLTIP
        ));
        item.menu.addMenuItem(this._switchItem(
            'LEM: 3 слова',
            'lem_3_words',
            false,
            LEM_3_TOOLTIP
        ));
        return item;
    }

    _ptahAlexsMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('ptah_alexs', false);
        const mode = new PopupMenu.PopupSwitchMenuItem(
            'Жёстко по окну',
            !!this._cfg.ptah_alexs_mode,
            {}
        );
        mode.connect('toggled', (_item, state) => {
            this._cfg.ptah_alexs_mode = state;
            this._saveAndRefresh();
            this._schedulePtahApply(20);
        });
        this._toggleButtons.ptah_alexs_mode = mode;
        this._attachTooltip(mode, PTAH_ALEXS_TOOLTIP);
        item.menu.addMenuItem(mode);

        this._ptahWindowLabel = new St.Label({
            text: this._ptahCurrentWindowText(),
            style: COMPACT_SUBTITLE_STYLE,
        });
        const current = new PopupMenu.PopupBaseMenuItem({
            activate: false,
            reactive: false,
            can_focus: false,
        });
        current.reactive = false;
        current.can_focus = false;
        current.style = 'padding:3px 12px 2px 12px;';
        current.add_child(this._ptahWindowLabel);
        item.menu.addMenuItem(current);

        item.menu.addMenuItem(this._ptahAssignRow());

        const rules = normalizePtahRules(this._cfg.ptah_alexs_rules);
        if (rules.length > 0) {
            item.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
            for (const rule of rules.slice(0, 8))
                item.menu.addMenuItem(this._ptahRuleRow(rule));
            if (rules.length > 8)
                item.menu.addMenuItem(this._mutedTextRow(`ещё правил: ${rules.length - 8}`));
        } else {
            item.menu.addMenuItem(this._mutedTextRow('правил пока нет'));
        }
        return item;
    }

    _ptahAssignRow() {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:4px 12px;';
        item.add_child(new St.Label({
            text: 'Текущее окно',
            y_align: Clutter.ActorAlign.CENTER,
            x_expand: true,
            style: 'font-weight:bold;',
        }));

        const controls = new St.BoxLayout({style: 'spacing:4px;'});
        controls.add_child(this._textStepButton('EN', () => this._setPtahRuleForFocusedWindow('us')));
        controls.add_child(this._textStepButton('RU', () => this._setPtahRuleForFocusedWindow('ru')));
        controls.add_child(this._textStepButton('keep', () => this._setPtahRuleForFocusedWindow('keep')));
        controls.add_child(this._textStepButton('×', () => this._removePtahRuleForFocusedWindow()));
        item.add_child(controls);
        return item;
    }

    _ptahRuleRow(rule) {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:3px 12px;';

        const label = new St.Label({
            text: `${rule.label} → ${this._ptahLayoutLabel(rule.layout)}`,
            y_align: Clutter.ActorAlign.CENTER,
            x_expand: true,
            style: COMPACT_SUBTITLE_STYLE,
        });
        item.add_child(label);
        item.add_child(this._textStepButton('×', () => this._removePtahRule(rule.kind, rule.value)));
        return item;
    }

    _mutedTextRow(text) {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:3px 12px;';
        item.add_child(new St.Label({
            text,
            style: COMPACT_SUBTITLE_STYLE,
        }));
        return item;
    }

    _ptahCurrentWindowText() {
        const info = focusedWindowInfo();
        if (!info)
            return 'текущее окно: не определено';
        const rule = this._findPtahRule(info);
        const suffix = rule ? ` → ${this._ptahLayoutLabel(rule.layout)}` : '';
        return `текущее: ${info.label}${suffix}`;
    }

    _ptahLayoutLabel(layout) {
        return {
            us: 'EN',
            ru: 'RU',
            keep: 'не трогать',
        }[layout] ?? String(layout ?? '');
    }

    _setPtahRuleForFocusedWindow(layout) {
        const info = focusedWindowInfo();
        if (!info)
            return;
        const normalized = normalizePtahLayout(layout);
        if (!normalized)
            return;
        const rules = normalizePtahRules(this._cfg.ptah_alexs_rules)
            .filter(rule => !this._samePtahIdentity(rule, info));
        rules.push({
            kind: info.kind,
            value: info.value,
            layout: normalized,
            label: info.label,
        });
        this._cfg.ptah_alexs_rules = normalizePtahRules(rules);
        this._cfg.ptah_alexs_mode = true;
        this._saveAndRebuildMenu();
        this._schedulePtahApply(20);
    }

    _removePtahRuleForFocusedWindow() {
        const info = focusedWindowInfo();
        if (!info)
            return;
        this._removePtahRule(info.kind, info.value);
    }

    _removePtahRule(kind, value) {
        this._cfg.ptah_alexs_rules = normalizePtahRules(this._cfg.ptah_alexs_rules)
            .filter(rule => !(rule.kind === kind && rule.value.toLowerCase() === String(value).toLowerCase()));
        this._saveAndRebuildMenu();
        this._schedulePtahApply(20);
    }

    _findPtahRule(info) {
        for (const rule of normalizePtahRules(this._cfg.ptah_alexs_rules)) {
            if (this._samePtahIdentity(rule, info))
                return rule;
            if (rule.kind === 'app_id' && rule.value === info.appId)
                return rule;
            if (rule.kind === 'wm_class' && rule.value === info.wmClass)
                return rule;
            if (rule.kind === 'wm_class_instance' && rule.value === info.wmClassInstance)
                return rule;
        }
        return null;
    }

    _samePtahIdentity(rule, info) {
        return rule.kind === info.kind && rule.value.toLowerCase() === info.value.toLowerCase();
    }

    _onFocusWindowChanged() {
        if (this._ptahWindowLabel)
            this._ptahWindowLabel.text = this._ptahCurrentWindowText();
        this._schedulePtahApply(50);
    }

    _schedulePtahApply(delayMs = 50) {
        if (this._ptahApplyId)
            GLib.Source.remove(this._ptahApplyId);
        this._ptahApplyId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, delayMs, () => {
            this._ptahApplyId = 0;
            this._applyPtahAlexsPolicy();
            return false;
        });
    }

    _applyPtahAlexsPolicy() {
        if (!this._cfg.ptah_alexs_mode)
            return;
        const info = focusedWindowInfo();
        if (!info)
            return;
        const rule = this._findPtahRule(info);
        if (!rule || rule.layout === 'keep')
            return;
        if (currentLayoutKind() === rule.layout)
            return;
        if (activateLayoutId(rule.layout))
            this._refreshLayout();
    }

    _correctionPipelineMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Коррекция', false);
        for (const [idx, rule] of this._cfg.typing_assist_pipeline.entries())
            item.menu.addMenuItem(this._pipelineRuleRow(rule, idx));
        return item;
    }

    _pipelineRuleRow(rule, idx) {
        const item = new PopupMenu.PopupSwitchMenuItem(
            `${idx + 1}. ${typingRuleLabel(rule.id)}`,
            rule.enabled,
            {}
        );
        item.connect('toggled', (_item, state) => {
            this._setTypingRuleEnabled(rule.id, state);
        });
        item.add_child(this._smallOrderButton('↑', () => this._moveTypingRule(rule.id, -1)));
        item.add_child(this._smallOrderButton('↓', () => this._moveTypingRule(rule.id, 1)));
        return item;
    }

    _smallOrderButton(label, onClick) {
        const button = new St.Button({
            label,
            reactive: true,
            can_focus: true,
            style_class: 'button flat',
            style: 'padding:1px 6px; border-radius:6px; min-width:0;',
        });
        button.connect('clicked', () => {
            onClick();
            return Clutter.EVENT_STOP;
        });
        return button;
    }

    _setTypingRuleEnabled(id, enabled) {
        this._cfg.typing_assist_pipeline = normalizeTypingPipeline(
            this._cfg.typing_assist_pipeline.map(rule =>
                rule.id === id ? {...rule, enabled} : rule)
        );
        this._saveAndRebuildMenu();
    }

    _moveTypingRule(id, delta) {
        const rules = normalizeTypingPipeline(this._cfg.typing_assist_pipeline);
        const idx = rules.findIndex(rule => rule.id === id);
        const target = Math.max(0, Math.min(rules.length - 1, idx + delta));
        if (idx < 0 || target === idx)
            return;
        const [rule] = rules.splice(idx, 1);
        rules.splice(target, 0, rule);
        this._cfg.typing_assist_pipeline = rules.map((item, order) => ({
            ...item,
            priority: (order + 1) * 10,
        }));
        this._saveAndRebuildMenu();
    }

    _saveAndRebuildMenu() {
        saveConfig(this._cfg);
        this.menu.removeAll();
        this._buildMenu();
    }

    _attachTooltip(actor, text) {
        actor.connect('enter-event', () => {
            this._showTooltip(actor, text);
            return Clutter.EVENT_PROPAGATE;
        });
        actor.connect('leave-event', () => {
            this._hideTooltip();
            return Clutter.EVENT_PROPAGATE;
        });
    }

    _showTooltip(anchor, text) {
        this._hideTooltip();

        const tooltip = new St.Label({
            text,
            style_class: 'dash-label',
            style: 'padding:8px 10px; border:1px solid rgba(255,255,255,0.28); border-radius:8px;',
        });
        tooltip.width = 420;
        tooltip.clutter_text.line_wrap = true;
        tooltip.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
        Main.uiGroup.add_child(tooltip);

        const [x, y] = anchor.get_transformed_position();
        const [width] = anchor.get_transformed_size();
        const [, tooltipWidth] = tooltip.get_preferred_width(-1);
        const [, tooltipHeight] = tooltip.get_preferred_height(tooltipWidth);
        let tx = x + width + 8;
        if (tx + tooltipWidth > global.stage.width - 8)
            tx = Math.max(8, x - tooltipWidth - 8);
        const ty = Math.max(8, Math.min(y - 2, global.stage.height - tooltipHeight - 8));
        tooltip.set_position(Math.round(tx), Math.round(ty));
        tooltip.opacity = 255;
        this._tooltip = tooltip;
    }

    _hideTooltip() {
        if (!this._tooltip)
            return;
        this._tooltip.destroy();
        this._tooltip = null;
    }

    _segmentedRow(title, options, target) {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:4px 12px;';

        const label = new St.Label({
            text: title,
            y_align: Clutter.ActorAlign.CENTER,
            x_expand: true,
            style: 'font-weight:bold;',
        });
        item.add_child(label);

        const controls = new St.BoxLayout({style: 'spacing:4px;'});
        for (const [id, text, onClick] of options) {
            const button = new St.Button({
                label: text,
                reactive: true,
                can_focus: true,
                toggle_mode: true,
                style_class: 'button flat',
                style: SEGMENT_BUTTON_STYLE,
            });
            button.connect('clicked', onClick);
            target[id] = button;
            controls.add_child(button);
        }
        item.add_child(controls);
        return item;
    }

    _triggerMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('', false);
        this._triggerMenuItem = item;
        for (const [id, label] of [
            ['double-lshift', 'Double Shift'],
            ['double-ctrl', 'Ctrl×2'],
            ['double-alt', 'Alt×2'],
            ['caps-lock', 'CapsLock'],
            ['single-rshift', 'RShift'],
            ['single-rctrl', 'RCtrl'],
            ['single-ralt', 'RAlt'],
        ]) {
            const row = new PopupMenu.PopupMenuItem(label);
            row.connect('activate', () => this._setTrigger(id));
            this._triggerItems[id] = row;
            item.menu.addMenuItem(row);
        }
        return item;
    }

    _timingMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Тайминг', false);
        item.menu.addMenuItem(this._timingCompactRow('Тап', 'tap_max_ms', 'мс', [100,150,200,250,300,350,400]));
        item.menu.addMenuItem(this._timingCompactRow('Окно', 'shift_window_ms', 'мс', [150,200,250,300,400,500]));
        return item;
    }

    _daemonSwitchItem() {
        const item = new PopupMenu.PopupSwitchMenuItem('Daemon', false, {});
        item.connect('toggled', (_item, state) => {
            if (this._updatingDaemonSwitch)
                return;
            this._toggleDaemonService(state);
        });
        this._daemonSwitch = item;
        return item;
    }

    _aboutMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('О программе', false);
        const block = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        block.reactive = false;
        block.can_focus = false;
        block.style = 'padding:8px 12px 10px 12px;';

        const box = new St.BoxLayout({
            vertical: true,
            x_expand: true,
            style: 'spacing:3px;',
        });
        box.add_child(new St.Label({
            text: `Lay ${APP_VERSION}`,
            style: 'font-weight:bold;',
        }));
        box.add_child(new St.Label({
            text: APP_DESCRIPTION,
            style: COMPACT_SUBTITLE_STYLE,
        }));
        box.add_child(new St.Label({
            text: `Дата версии: ${APP_RELEASE_DATE}`,
            style: COMPACT_SUBTITLE_STYLE,
        }));
        box.add_child(new St.Label({
            text: APP_PLATFORM,
            style: COMPACT_SUBTITLE_STYLE,
        }));
        box.add_child(new St.Label({
            text: `Совместимость: ${APP_GNOME_SUPPORT}`,
            style: COMPACT_SUBTITLE_STYLE,
        }));
        box.add_child(new St.Label({
            text: `Лицензия: ${APP_LICENSE}`,
            style: COMPACT_SUBTITLE_STYLE,
        }));
        this._aboutConfigLabel = new St.Label({
            text: `Настройки: ${this._aboutConfigText()}`,
            style: COMPACT_SUBTITLE_STYLE,
        });
        box.add_child(this._aboutConfigLabel);
        this._aboutStatsLabel = new St.Label({
            text: `Статистика: ${this._aboutStatsText()}`,
            style: COMPACT_SUBTITLE_STYLE,
        });
        box.add_child(this._aboutStatsLabel);
        const link = new St.Label({
            text: APP_URL,
            reactive: true,
            can_focus: true,
            style: `${COMPACT_SUBTITLE_STYLE}; text-decoration: underline;`,
        });
        link.connect('button-release-event', () => {
            openUri(APP_URL);
            return Clutter.EVENT_STOP;
        });
        box.add_child(link);
        block.add_child(box);
        item.menu.addMenuItem(block);
        return item;
    }

    _timingCompactRow(title, key, suffix, steps) {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:4px 12px;';
        item.add_child(new St.Label({
            text: title,
            y_align: Clutter.ActorAlign.CENTER,
            x_expand: true,
        }));

        const value = new St.Label({
            text: `${this._cfg[key]}${suffix}`,
            y_align: Clutter.ActorAlign.CENTER,
            style: 'font-feature-settings:"tnum";',
        });
        const controls = new St.BoxLayout({style: 'spacing:4px;'});
        controls.add_child(this._textStepButton('−', () => this._stepTiming(key, steps, -1, value, suffix)));
        controls.add_child(value);
        controls.add_child(this._textStepButton('+', () => this._stepTiming(key, steps, 1, value, suffix)));
        item.add_child(controls);
        return item;
    }

    _textStepButton(label, onClick) {
        const button = new St.Button({
            label,
            reactive: true,
            can_focus: true,
            style_class: 'button flat',
            style: 'padding:1px 7px; border-radius:999px; min-width:0;',
        });
        button.connect('clicked', onClick);
        return button;
    }

    _stepTiming(key, steps, delta, value, suffix) {
        const idx = steps.indexOf(this._cfg[key]);
        const ni = Math.max(0, Math.min(steps.length - 1, idx + delta));
        if (ni === idx)
            return;
        this._cfg[key] = steps[ni];
        value.text = `${this._cfg[key]}${suffix}`;
        saveConfig(this._cfg);
        restartDaemon();
    }

    _refreshSelections() {
        if (this._triggerMenuItem)
            this._triggerMenuItem.label.text = `Триггер: ${this._triggerLabel(this._cfg.trigger)}`;
        if (this._aboutConfigLabel)
            this._aboutConfigLabel.text = `Настройки: ${this._aboutConfigText()}`;
        if (this._ptahWindowLabel)
            this._ptahWindowLabel.text = this._ptahCurrentWindowText();
        this._refreshStats();
        for (const [id, button] of Object.entries(this._engineButtons ?? {}))
            this._setButtonActive(button, id === this._cfg.correction_engine);
        for (const [id, button] of Object.entries(this._scopeButtons ?? {}))
            this._setButtonActive(button, Number(id) === this._cfg.replace_words);
        for (const [id, button] of Object.entries(this._triggerButtons ?? {}))
            this._setButtonActive(button, id === this._cfg.trigger);
        for (const [id, row] of Object.entries(this._triggerItems ?? {}))
            row.setOrnament(id === this._cfg.trigger ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NONE);
        for (const [key, button] of Object.entries(this._toggleButtons ?? {})) {
            if (button.setToggleState)
                button.setToggleState(!!this._cfg[key]);
            else
                this._setButtonActive(button, !!this._cfg[key]);
        }
    }

    _saveAndRefresh() {
        this._cfg.replace_words = Math.max(1, Math.min(3, this._cfg.replace_words));
        this._cfg.correction_engine = this._cfg.correction_engine === 'smart' ? 'smart' : 'replay';
        this._cfg.ptah_alexs_mode = !!this._cfg.ptah_alexs_mode;
        this._cfg.ptah_alexs_rules = normalizePtahRules(this._cfg.ptah_alexs_rules);
        this._cfg.mode = 'simple';
        this._refreshSelections();
        saveConfig(this._cfg);
    }

    _setTrigger(id) {
        if (this._cfg.trigger === id) {
            this._refreshSelections();
            return;
        }
        this._cfg.trigger = id;
        this._saveAndRefresh();
        restartDaemon();
        this._setDaemonBusy('restarting...');
        this._scheduleStatusRefreshes();
    }

    _toggleDaemonService(shouldStart = null) {
        if (shouldStart === null)
            shouldStart = this._daemonActive === false;
        if (shouldStart) {
            startDaemon();
            this._setDaemonBusy('starting...');
        } else {
            stopDaemon();
            this._setDaemonBusy('stopping...');
        }
        this._scheduleStatusRefreshes();
    }

    _setButtonActive(button, active) {
        button.set_style_class_name(active ? 'button' : 'button flat');
        button.style = SEGMENT_BUTTON_STYLE;
        if (button.set_checked)
            button.set_checked(active);
    }

    _triggerLabel(id) {
        return {
            'double-lshift': 'Double Shift',
            'double-ctrl': 'Ctrl×2',
            'double-alt': 'Alt×2',
            'caps-lock': 'CapsLock',
            'single-rshift': 'RShift',
            'single-rctrl': 'RCtrl',
            'single-ralt': 'RAlt',
        }[id] ?? 'Double Shift';
    }

    _aboutConfigText() {
        const autoSwitch = this._cfg.auto_switch_layout ? 'авто-layout' : 'layout вручную';
        const lem = `LEM ${this._cfg.lem_2_words ? '2' : '-'}${this._cfg.lem_3_words ? '/3' : ''}`;
        const ptah = this._cfg.ptah_alexs_mode ? 'ptah on' : 'ptah off';
        return `${this._engineLabel()} · ${this._cfg.replace_words} сл. · ${lem} · ${autoSwitch} · ${ptah} · ${this._triggerLabel(this._cfg.trigger)}`;
    }

    _aboutStatsText() {
        const stats = loadStats();
        return `LLM ${stats.llm_calls ?? 0}${this._lastTime(stats.last_llm_ts)} · `
            + `правки ${stats.learning_log_entries ?? 0}${this._lastTime(stats.last_learning_ts)} · `
            + `правил ${stats.promoted_rules ?? 0}${this._lastTime(stats.last_promotion_ts)}`;
    }

    _refreshStats() {
        if (this._aboutStatsLabel)
            this._aboutStatsLabel.text = `Статистика: ${this._aboutStatsText()}`;
    }

    _lastTime(ts) {
        if (!ts)
            return '';
        try {
            const date = new Date(ts * 1000);
            return `, ${date.toLocaleTimeString([], {hour: '2-digit', minute: '2-digit'})}`;
        } catch(e) {
            return '';
        }
    }

    _engineLabel() {
        return this._cfg.correction_engine === 'smart' ? 'Smart' : 'Replay';
    }

    _refreshLayout() {
        try {
            const isRu = this._mgr.currentSource?.id === 'ru';
            this._label.text = isRu ? 'RU' : 'EN';
        } catch(e) { this._label.text = '--'; }
    }

    _refreshStatus() {
        try {
            const p = Gio.Subprocess.new(
                ['systemctl','--user','is-active','lay-daemon'],
                Gio.SubprocessFlags.STDOUT_PIPE);
            p.communicate_utf8_async(null, null, (proc, res) => {
                try {
                    const [, out] = proc.communicate_utf8_finish(res);
                    const ok = out.trim() === 'active';
                    this._daemonActive = ok;
                    this._statusLabel.text = ok ? 'daemon active' : 'daemon stopped';
                    this._setDaemonStatus(ok);
                    this._refreshDaemonAction(ok);
                } catch(e) {}
            });
        } catch(e) {}
    }

    _setDaemonBusy(text) {
        this._stopStatusBlink();
        if (this._statusLabel)
            this._statusLabel.text = text;
        if (this._statusDot) {
            this._statusDot.opacity = 255;
            this._statusDot.style = 'font-size:90%; color:#f6c343;';
        }
    }

    _refreshDaemonAction(active) {
        if (this._daemonSwitch?.setToggleState) {
            this._updatingDaemonSwitch = true;
            this._daemonSwitch.setToggleState(active);
            this._updatingDaemonSwitch = false;
        }
    }

    _scheduleStatusRefreshes() {
        this._clearStatusRefreshes();
        for (const delay of [700, 1500, 3000]) {
            const id = GLib.timeout_add(GLib.PRIORITY_DEFAULT, delay, () => {
                this._statusRefreshIds = this._statusRefreshIds.filter(existing => existing !== id);
                this._refreshStatus();
                return false;
            });
            this._statusRefreshIds.push(id);
        }
    }

    _clearStatusRefreshes() {
        for (const id of this._statusRefreshIds ?? [])
            GLib.Source.remove(id);
        this._statusRefreshIds = [];
    }

    _setDaemonStatus(active) {
        if (!this._statusDot)
            return;

        if (active) {
            this._statusDot.style = 'font-size:90%; color:#26a269;';
            this._startStatusBlink();
        } else {
            this._stopStatusBlink();
            this._statusDot.opacity = 255;
            this._statusDot.style = 'font-size:90%; color:#c01c28;';
        }
    }

    _startStatusBlink() {
        if (this._blinkId)
            return;

        this._blinkBright = true;
        this._statusDot.opacity = 255;
        this._blinkId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 650, () => {
            if (!this._statusDot) {
                this._blinkId = 0;
                return false;
            }
            this._blinkBright = !this._blinkBright;
            this._statusDot.opacity = this._blinkBright ? 255 : 95;
            return true;
        });
    }

    _stopStatusBlink() {
        if (this._blinkId) {
            GLib.Source.remove(this._blinkId);
            this._blinkId = 0;
        }
    }

    destroy() {
        this._clearStatusRefreshes();
        this._stopStatusBlink();
        this._hideTooltip();
        if (this._ptahApplyId) {
            GLib.Source.remove(this._ptahApplyId);
            this._ptahApplyId = 0;
        }
        if (this._srcId) { this._mgr.disconnect(this._srcId); this._srcId = 0; }
        if (this._focusId) { global.display.disconnect(this._focusId); this._focusId = 0; }
        super.destroy();
    }
});

// ─── Entry point ───────────────────────────────────────────

export class LayImpl {
    constructor(_ext) {}

    enable() {
        this._service = new LayDaemonService();
        this._service.enable();
        this._indicator = new LayIndicator();
        Main.panel.addToStatusArea(`lay-${_uid}`, this._indicator, 0, 'right');
        log('[lay-extension] LayImpl enabled ✓');
    }

    disable() {
        this._indicator?.destroy(); this._indicator = null;
        this._service?.disable();   this._service   = null;
    }
}
