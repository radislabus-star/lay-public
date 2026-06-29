/* lay-impl.js — реализация extension (меняется свободно, без logout)
 * Загружается через loader в extension.js с уникальным URL → нет кэша GJS.
 */

import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import St from 'gi://St';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import {getInputSourceManager} from 'resource:///org/gnome/shell/ui/status/keyboard.js';

import {
    LayDaemonService,
    activateLayoutId,
    currentLayoutKind,
    focusedWindowInfo,
    syncIbusEngineForCurrentLayout,
} from './dbus_service.js';
import {
    createRecentActionsMenu,
    populateRecentActionsMenu,
    refreshRecentActions,
} from './recent_actions_menu.js';

// ─── Tray support ──────────────────────────────────────────

import {
    APP_DESCRIPTION,
    APP_GNOME_SUPPORT,
    APP_ICON_NAME,
    APP_LICENSE,
    APP_PLATFORM,
    APP_RELEASE_DATE,
    APP_URL,
    APP_VERSION,
    COMPACT_SUBTITLE_STYLE,
    MENU_ICON_SIZE,
    MENU_WIDTH,
    PANEL_ICON_SIZE,
    SAFETY_OPTIONS,
    TRIGGER_OPTIONS,
    actionKindLabel,
    loadConfig,
    loadRecentActions,
    loadStats,
    normalizeConfig,
    normalizePtahRules,
    openPreferences,
    openUri,
    optionLabel,
    startDaemon,
    startUpdate,
    stopDaemon,
    summarizeRecentActions,
} from './tray_support.js';

function persistentSwitchItem(label, active, params = {}) {
    const item = new PopupMenu.PopupSwitchMenuItem(label, active, params);
    item.activate = function() {
        this.toggle();
    };
    return item;
}

// ─── Tray Indicator ────────────────────────────────────────
// Уникальный GTypeName предотвращает ошибку "already registered"
// при повторном disable→enable в одной сессии.

const _uid = Date.now();

const LayIndicator = GObject.registerClass(
{GTypeName: `LayIndicator_${_uid}`},
class LayIndicator extends PanelMenu.Button {

    _init(service = null) {
        super._init(0.0, 'lay');
        this._service = service;
        this._cfg = normalizeConfig(loadConfig());

        this._panelBox = new St.BoxLayout({
            style: 'spacing:4px; padding:0 2px;',
        });
        this._panelIcon = new St.Icon({
            icon_name: APP_ICON_NAME,
            icon_size: PANEL_ICON_SIZE,
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
            if (isOpen) {
                this._refreshStats();
                this._refreshRecentActions();
            }
        });

        this._mgr = getInputSourceManager();
        this._srcId = this._mgr.connect('current-source-changed', () => {
            syncIbusEngineForCurrentLayout();
            this._refreshLayout();
        });
        this._focusId = global.display.connect('notify::focus-window', () => this._onFocusWindowChanged());
        syncIbusEngineForCurrentLayout();
        this._refreshLayout();
        this._schedulePtahApply(80);
    }

    _buildMenu() {
        this.menu.box.style = `min-width:${MENU_WIDTH}px; padding:2px 0;`;
        this._statusRefreshIds = [];

        this._statusItem = this._headerItem();
        this.menu.addMenuItem(this._statusItem);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        this.menu.addMenuItem(this._preferencesItem());
        this.menu.addMenuItem(this._recentActionsMenu());
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        this.menu.addMenuItem(this._daemonSwitchItem());
        this.menu.addMenuItem(this._updateItem());
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        this.menu.addMenuItem(this._aboutMenu());

        this._refreshSelections();
        this._refreshStatus();
    }

    _headerItem() {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:5px 8px 4px 8px;';
        const card = new St.BoxLayout({
            x_expand: true,
            style: 'spacing:8px;',
        });
        const icon = new St.Icon({
            icon_name: APP_ICON_NAME,
            icon_size: MENU_ICON_SIZE,
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

    _recentActionsMenu() {
        return createRecentActionsMenu(this);
    }

    _populateRecentActionsMenu(item) {
        populateRecentActionsMenu(this, item);
    }

    _refreshRecentActions() {
        refreshRecentActions(this);
    }

    _mutedTextRow(text) {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:3px 8px;';
        item.add_child(new St.Label({
            text,
            style: COMPACT_SUBTITLE_STYLE,
        }));
        return item;
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

    _daemonSwitchItem() {
        const item = persistentSwitchItem('Демон включён', false);
        item.connect('toggled', (_item, state) => {
            if (this._updatingDaemonSwitch)
                return;
            this._toggleDaemonService(state);
        });
        this._daemonSwitch = item;
        return item;
    }

    _updateItem() {
        const item = new PopupMenu.PopupMenuItem('Проверить обновления');
        item.connect('activate', () => this._runUpdate());
        return item;
    }

    _preferencesItem() {
        const item = new PopupMenu.PopupMenuItem('Настройки...');
        item.connect('activate', () => openPreferences());
        return item;
    }

    _aboutMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('О программе', false);
        const block = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        block.reactive = false;
        block.can_focus = false;
        block.style = 'padding:8px 8px 10px 8px;';

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
            text: 'GitHub проекта',
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

    _refreshSelections() {
        this._cfg = normalizeConfig(this._cfg);
        if (this._aboutConfigLabel)
            this._aboutConfigLabel.text = `Настройки: ${this._aboutConfigText()}`;
        this._refreshStats();
    }

    _toggleDaemonService(shouldStart = null) {
        if (shouldStart === null)
            shouldStart = this._daemonActive === false;
        if (shouldStart) {
            startDaemon();
            this._setDaemonBusy('запуск...');
        } else {
            stopDaemon();
            this._setDaemonBusy('остановка...');
        }
        this._scheduleStatusRefreshes();
    }

    _runUpdate() {
        const [ok, message] = startUpdate();
        this._notify(ok ? 'Проверка обновлений запущена' : 'Проверка не запущена', message, !ok);
    }

    _notify(title, message, isError = false) {
        try {
            if (isError && Main.notifyError)
                Main.notifyError('lay', `${title}\n${message}`);
            else
                Main.notify('lay', `${title}\n${message}`);
        } catch(e) {
            log(`[lay-extension] ${title}: ${message}`);
        }
    }

    _triggerLabel(id) {
        return optionLabel(TRIGGER_OPTIONS, id, 'Двойной Shift');
    }

    _safetyLabel() {
        return optionLabel(SAFETY_OPTIONS, this._cfg.correction_safety, 'норма');
    }

    _actionKindLabel(kind) {
        return actionKindLabel(kind);
    }

    _shortActionText(value) {
        const text = String(value ?? '').replace(/\s+/g, ' ').trim();
        if (text.length <= 46)
            return text;
        return `${text.slice(0, 43)}...`;
    }

    _aboutConfigText() {
        const autoSwitch = this._cfg.auto_switch_layout ? 'авто-раскладка' : 'раскладка вручную';
        const lem = this._cfg.lem_enabled
            ? `LEM ${this._cfg.lem_2_words ? '2' : '-'}${this._cfg.lem_3_words ? '/3' : ''}`
            : 'LEM выкл';
        const weights = `вес ${this._cfg.lem_weight_percent ?? 80}/${this._cfg.nanda_l2_weight_percent ?? 20}/${this._cfg.nanda_l3_weight_percent ?? 8}`;
        const force = this._cfg.force_layout_hotkeys ? 'RU/EN хоткеи' : 'RU/EN выкл';
        return `${this._engineLabel()} · ${this._safetyLabel()} · ${this._cfg.replace_words} сл. · ${lem} · ${weights} · ${autoSwitch} · ${force} · ${this._triggerLabel(this._cfg.trigger)}`;
    }

    _aboutStatsText() {
        const stats = loadStats();
        const actions = summarizeRecentActions(loadRecentActions(20));
        return `LLM ${stats.llm_calls ?? 0}${this._lastTime(stats.last_llm_ts)} · `
            + `правки ${stats.learning_log_entries ?? 0}${this._lastTime(stats.last_learning_ts)} · `
            + `правил ${stats.promoted_rules ?? 0}${this._lastTime(stats.last_promotion_ts)} · `
            + `действия: ${actions}`;
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
        return this._cfg.correction_engine === 'smart' ? 'Умный' : 'Обычный';
    }

    _refreshLayout() {
        try {
            const isRu = currentLayoutKind() === 'ru';
            this._label.text = isRu ? 'RU' : 'EN';
        } catch(e) { this._label.text = '--'; }
    }

    _refreshStatus() {
        try {
            const p = Gio.Subprocess.new(
                ['/usr/bin/systemctl', '--user', 'is-active', '--quiet', 'lay-daemon.service'],
                Gio.SubprocessFlags.NONE);
            p.wait_check_async(null, (proc, res) => {
                try {
                    this._applyDaemonStatus(proc.wait_check_finish(res));
                } catch(e) {
                    this._applyDaemonStatus(false);
                }
            });
        } catch(e) {}
    }

    _applyDaemonStatus(active) {
        this._daemonActive = active;
        if (this._statusLabel)
            this._statusLabel.text = active ? 'демон работает' : 'демон остановлен';
        this._setDaemonStatus(active);
        this._refreshDaemonAction(active);
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
        this._indicator = new LayIndicator(this._service);
        Main.panel.addToStatusArea('lay', this._indicator, 0, 'right');
    }

    disable() {
        this._indicator?.destroy(); this._indicator = null;
        this._service?.disable();   this._service   = null;
    }
}
