/* lay-impl.js — реализация extension (меняется свободно, без logout)
 * Загружается через loader в extension.js с уникальным URL → нет кэша GJS.
 */

import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import Pango from 'gi://Pango';
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
    AUTO_REPLACE_TOOLTIP,
    AUTO_SWITCH_TOOLTIP,
    COMPACT_SUBTITLE_STYLE,
    DEBUG_ACTION_LOG_TOOLTIP,
    ENTER_AUTOCORRECT_TOOLTIP,
    FORCE_KEY_OPTIONS,
    LEM_2_TOOLTIP,
    LEM_3_TOOLTIP,
    LAYOUT_BACKEND_OPTIONS,
    LEARNING_LOG_TOOLTIP,
    MENU_ICON_SIZE,
    MENU_WIDTH,
    PANEL_ICON_SIZE,
    PTAH_ALEXS_TOOLTIP,
    SAFETY_OPTIONS,
    SAFETY_STEPS,
    SEGMENT_BUTTON_STYLE,
    TRIGGER_OPTIONS,
    actionKindLabel,
    loadConfig,
    loadRecentActions,
    loadStats,
    normalizeConfig,
    normalizePtahRules,
    normalizeTypingPipeline,
    openPreferences,
    openUri,
    applyInputChannel,
    optionLabel,
    restartDaemon,
    saveConfig,
    startDaemon,
    startUpdate,
    stopDaemon,
    summarizeRecentActions,
    typingRuleLabel,
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
        this._srcId = this._mgr.connect('current-source-changed', () => this._refreshLayout());
        this._focusId = global.display.connect('notify::focus-window', () => this._onFocusWindowChanged());
        this._refreshLayout();
        this._schedulePtahApply(80);
    }

    _buildMenu() {
        this.menu.box.style = `min-width:${MENU_WIDTH}px; padding:2px 0;`;
        this._engineButtons = {};
        this._scopeButtons = {};
        this._backendButtons = {};
        this._layoutBackendButtons = {};
        this._safetyButtons = {};
        this._triggerButtons = {};
        this._triggerItems = {};
        this._forceRuItems = {};
        this._forceEnItems = {};
        this._toggleButtons = {};
        this._safetySliderLabel = null;
        this._safetySliderThumbs = [];
        this._statusRefreshIds = [];
        this._cfg.typing_assist_pipeline = normalizeTypingPipeline(this._cfg.typing_assist_pipeline);

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

    _switchItem(label, key, restart = false, tooltip = null) {
        const item = persistentSwitchItem(label, !!this._cfg[key]);
        item.connect('toggled', (_item, state) => {
            this._cfg[key] = state;
            this._saveAndRefresh();
            if (restart) {
                restartDaemon();
                this._setDaemonBusy('перезапуск...');
                this._scheduleStatusRefreshes();
            }
        });
        if (tooltip)
            this._attachTooltip(item, tooltip);
        this._toggleButtons[key] = item;
        return item;
    }

    _debugLogsItem() {
        const item = persistentSwitchItem('Журнал отладки lay', !!this._cfg.debug_action_log);
        item.connect('toggled', (_item, state) => {
            this._cfg.debug_action_log = state;
            this._cfg.nanda_trace = state;
            this._cfg.nanda_trace_text = state;
            this._saveAndRefresh();
        });
        this._attachTooltip(item, DEBUG_ACTION_LOG_TOOLTIP);
        this._toggleButtons.debug_action_log = item;
        return item;
    }

    _inlinePreeditItem() {
        const active = !!this._cfg.nanda_precognition && this._cfg.text_backend === 'ime';
        const item = persistentSwitchItem('Серые подсказки (IME)', active);
        item.connect('toggled', (_item, state) => {
            this._cfg.nanda_precognition = !!state;
            if (state)
                this._cfg.text_backend = 'ime';
            else
                this._cfg.text_backend = 'uinput';
            this._saveAndRefresh();
            applyInputChannel(this._cfg.text_backend);
            restartDaemon();
            this._setDaemonBusy('перезапуск...');
            this._scheduleStatusRefreshes();
        });
        this._attachTooltip(item, 'Inline-подсказки работают только через экспериментальный IME-канал. Быстрый uinput выводит текст без серого preedit.');
        this._inlinePreeditSwitch = item;
        return item;
    }

    _behaviorMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Поведение', false);
        item.menu.addMenuItem(this._switchItem(
            'Помощь при наборе',
            'typing_assist',
            true
        ));
        item.menu.addMenuItem(this._switchItem(
            'Автоподмена',
            'auto_replace',
            true,
            AUTO_REPLACE_TOOLTIP
        ));
        item.menu.addMenuItem(this._switchItem(
            'Запоминать правки',
            'learning_log',
            false,
            LEARNING_LOG_TOOLTIP
        ));
        item.menu.addMenuItem(this._debugLogsItem());
        item.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        item.menu.addMenuItem(this._switchItem(
            'Автораскладка после пробела',
            'auto_switch_layout',
            false,
            AUTO_SWITCH_TOOLTIP
        ));
        item.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        item.menu.addMenuItem(this._switchItem(
            'Исправлять перед Enter',
            'enter_autocorrect',
            true,
            ENTER_AUTOCORRECT_TOOLTIP
        ));
        return item;
    }

    _expertMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Экспертное', false);
        item.menu.addMenuItem(this._arbiterMenu());
        item.menu.addMenuItem(this._correctionPipelineMenu());
        item.menu.addMenuItem(this._ptahAlexsMenu());
        item.menu.addMenuItem(this._triggerMenu());
        item.menu.addMenuItem(this._forceLayoutMenu());
        item.menu.addMenuItem(this._timingMenu());
        item.menu.addMenuItem(this._backendMenu());
        return item;
    }

    _arbiterMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Арбитры', false);
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
        item.menu.addMenuItem(this._inlinePreeditItem());
        return item;
    }

    _engineOptions() {
        return [
            ['replay', 'Обычный', () => this._setConfigValue('correction_engine', 'replay')],
            ['smart', 'Умный', () => this._setConfigValue('correction_engine', 'smart')],
        ];
    }

    _scopeOptions() {
        return [
            ['1', '1 слово', () => this._setConfigValue('replace_words', 1)],
            ['2', '2 слова', () => this._setConfigValue('replace_words', 2)],
            ['3', '3 слова', () => this._setConfigValue('replace_words', 3)],
        ];
    }

    _safetyOptions() {
        return SAFETY_OPTIONS.map(([id, label]) => [
            id,
            label[0].toUpperCase() + label.slice(1),
            () => this._setConfigValue('correction_safety', id, true),
        ]);
    }

    _backendOptions() {
        return [
            ['uinput', 'Быстрый', () => this._setTextBackend('uinput')],
            ['ime', 'IME, эксперимент', () => this._setTextBackend('ime')],
        ];
    }

    _layoutBackendOptions() {
        return LAYOUT_BACKEND_OPTIONS.map(([id, label]) => [
            id,
            label,
            () => this._setConfigValue('layout_backend', id, true),
        ]);
    }

    _backendMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Каналы ввода', false);
        item.menu.addMenuItem(this._segmentedRow('Раскладка', this._layoutBackendOptions(), this._layoutBackendButtons));
        item.menu.addMenuItem(this._segmentedRow('Ввод', this._backendOptions(), this._backendButtons));
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

    _ptahAlexsMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Раскладка по окну', false);
        const mode = persistentSwitchItem(
            'Жёстко по окну',
            !!this._cfg.ptah_alexs_mode
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
        current.style = 'padding:3px 8px 2px 8px;';
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
        item.style = 'padding:4px 8px;';
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
        item.style = 'padding:3px 8px;';

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
        item.style = 'padding:3px 8px;';
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
        const item = new PopupMenu.PopupSubMenuMenuItem('Правила коррекции', false);
        for (const [idx, rule] of this._cfg.typing_assist_pipeline.entries())
            item.menu.addMenuItem(this._pipelineRuleRow(rule, idx));
        return item;
    }

    _pipelineRuleRow(rule, idx) {
        const item = persistentSwitchItem(
            `${idx + 1}. ${typingRuleLabel(rule.id)}`,
            rule.enabled
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
        this._rebuildMenuKeepingOpen();
    }

    _rebuildMenuKeepingOpen() {
        const openedSubmenus = this._openedSubmenuLabels();
        this.menu.removeAll();
        this._buildMenu();
        this._restoreOpenSubmenus(openedSubmenus);
    }

    _openedSubmenuLabels() {
        try {
            return (this.menu._getMenuItems?.() ?? [])
                .filter(item => item instanceof PopupMenu.PopupSubMenuMenuItem && item.menu.isOpen)
                .map(item => item.label.text);
        } catch(e) {
            return [];
        }
    }

    _restoreOpenSubmenus(labels) {
        if (!labels.length)
            return;
        const wanted = new Set(labels);
        try {
            for (const item of this.menu._getMenuItems?.() ?? []) {
                if (item instanceof PopupMenu.PopupSubMenuMenuItem && wanted.has(item.label.text))
                    item.menu.open();
            }
        } catch(e) {}
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

    _safetySliderRow() {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:5px 8px;';

        const box = new St.BoxLayout({
            vertical: true,
            x_expand: true,
            style: 'spacing:4px;',
        });
        const titleRow = new St.BoxLayout({
            x_expand: true,
            style: 'spacing:8px;',
        });
        titleRow.add_child(new St.Label({
            text: 'Агрессивность',
            y_align: Clutter.ActorAlign.CENTER,
            x_expand: true,
            style: 'font-weight:bold;',
        }));
        this._safetySliderLabel = new St.Label({
            text: this._safetySliderLabelText(),
            y_align: Clutter.ActorAlign.CENTER,
            style: COMPACT_SUBTITLE_STYLE,
        });
        titleRow.add_child(this._safetySliderLabel);
        box.add_child(titleRow);

        const track = new St.BoxLayout({
            x_expand: true,
            style: 'spacing:4px;',
        });
        this._safetySliderThumbs = [];
        for (const step of SAFETY_STEPS) {
            const button = new St.Button({
                label: step.label,
                reactive: true,
                can_focus: true,
                toggle_mode: true,
                style_class: 'button flat',
                style: 'padding:1px 5px; border-radius:999px; min-width:0;',
                x_expand: true,
            });
            button.connect('clicked', () => this._setSafetyLevel(step.id));
            this._safetySliderThumbs.push([step.id, button]);
            track.add_child(button);
        }
        box.add_child(track);
        item.add_child(box);
        this._attachTooltip(item,
            'Настраивает, насколько смело lay исправляет после пробела.\n'
            + 'Осторожно: только самые безопасные правила.\n'
            + 'Норма: текущий стабильный режим.\n'
            + 'Смелее: больше исправлений, выше риск ложной замены.'
        );
        return item;
    }

    _setSafetyLevel(id) {
        this._setConfigValue('correction_safety', id, true);
    }

    _safetySliderLabelText() {
        return SAFETY_STEPS.find(step => step.id === this._cfg.correction_safety)?.label ?? 'Норма';
    }

    _segmentedRow(title, options, target) {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:4px 8px;';

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
        const item = new PopupMenu.PopupSubMenuMenuItem('Триггер', false);
        this._triggerMenuItem = item;
        for (const [id, label] of TRIGGER_OPTIONS) {
            const row = new PopupMenu.PopupMenuItem(label);
            row.connect('activate', () => this._setTrigger(id));
            this._triggerItems[id] = row;
            item.menu.addMenuItem(row);
        }
        item.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        item.menu.addMenuItem(this._switchItem(
            'Несколько нажатий Shift',
            'multi_tap_scope',
            true
        ));
        return item;
    }

    _forceLayoutMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Прямой язык', false);
        item.menu.addMenuItem(this._switchItem(
            'Хоткеи RU / EN',
            'force_layout_hotkeys',
            true
        ));
        item.menu.addMenuItem(this._forceKeyMenu('RU', 'force_ru_key', this._forceRuItems));
        item.menu.addMenuItem(this._forceKeyMenu('EN', 'force_en_key', this._forceEnItems));
        return item;
    }

    _forceKeyMenu(title, key, target) {
        const item = new PopupMenu.PopupSubMenuMenuItem(`${title}: ${this._forceKeyLabel(this._cfg[key])}`, false);
        item._layConfigKey = key;
        for (const [id, label] of FORCE_KEY_OPTIONS) {
            const row = new PopupMenu.PopupMenuItem(label);
            row.connect('activate', () => this._setForceKey(key, id));
            target[id] = row;
            item.menu.addMenuItem(row);
        }
        if (key === 'force_ru_key')
            this._forceRuMenuItem = item;
        else
            this._forceEnMenuItem = item;
        return item;
    }

    _timingMenu() {
        const item = new PopupMenu.PopupSubMenuMenuItem('Тайминг', false);
        item.menu.addMenuItem(this._timingCompactRow('Тап', 'tap_max_ms', 'мс', [100,150,200,250,300,350,400]));
        item.menu.addMenuItem(this._timingCompactRow('Окно', 'shift_window_ms', 'мс', [150,200,250,300,400,500]));
        return item;
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

    _timingCompactRow(title, key, suffix, steps) {
        const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
        item.reactive = false;
        item.can_focus = false;
        item.style = 'padding:4px 8px;';
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
        this._cfg = normalizeConfig(this._cfg);
        if (this._triggerMenuItem)
            this._triggerMenuItem.label.text = `Триггер: ${this._triggerLabel(this._cfg.trigger)}`;
        if (this._forceRuMenuItem)
            this._forceRuMenuItem.label.text = `RU: ${this._forceKeyLabel(this._cfg.force_ru_key)}`;
        if (this._forceEnMenuItem)
            this._forceEnMenuItem.label.text = `EN: ${this._forceKeyLabel(this._cfg.force_en_key)}`;
        if (this._aboutConfigLabel)
            this._aboutConfigLabel.text = `Настройки: ${this._aboutConfigText()}`;
        if (this._ptahWindowLabel)
            this._ptahWindowLabel.text = this._ptahCurrentWindowText();
        if (this._safetySliderLabel)
            this._safetySliderLabel.text = this._safetySliderLabelText();
        this._refreshStats();
        for (const [id, button] of Object.entries(this._engineButtons ?? {}))
            this._setButtonActive(button, id === this._cfg.correction_engine);
        for (const [id, button] of Object.entries(this._scopeButtons ?? {}))
            this._setButtonActive(button, Number(id) === this._cfg.replace_words);
        for (const [id, button] of Object.entries(this._backendButtons ?? {}))
            this._setButtonActive(button, id === this._cfg.text_backend);
        if (this._inlinePreeditSwitch?.setToggleState)
            this._inlinePreeditSwitch.setToggleState(
                !!this._cfg.nanda_precognition && this._cfg.text_backend === 'ime'
            );
        for (const [id, button] of Object.entries(this._layoutBackendButtons ?? {}))
            this._setButtonActive(button, id === this._cfg.layout_backend);
        for (const [id, button] of Object.entries(this._safetyButtons ?? {}))
            this._setButtonActive(button, id === this._cfg.correction_safety);
        for (const [id, button] of this._safetySliderThumbs ?? [])
            this._setButtonActive(button, id === this._cfg.correction_safety);
        for (const [id, button] of Object.entries(this._triggerButtons ?? {}))
            this._setButtonActive(button, id === this._cfg.trigger);
        for (const [id, row] of Object.entries(this._triggerItems ?? {}))
            row.setOrnament(id === this._cfg.trigger ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NONE);
        for (const [id, row] of Object.entries(this._forceRuItems ?? {}))
            row.setOrnament(id === this._cfg.force_ru_key ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NONE);
        for (const [id, row] of Object.entries(this._forceEnItems ?? {}))
            row.setOrnament(id === this._cfg.force_en_key ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NONE);
        for (const [key, button] of Object.entries(this._toggleButtons ?? {})) {
            if (button.setToggleState)
                button.setToggleState(!!this._cfg[key]);
            else
                this._setButtonActive(button, !!this._cfg[key]);
        }
    }

    _saveAndRefresh() {
        this._cfg = normalizeConfig(this._cfg);
        this._cfg.ptah_alexs_rules = normalizePtahRules(this._cfg.ptah_alexs_rules);
        if (this._cfg.force_ru_key === this._cfg.force_en_key)
            this._cfg.force_layout_hotkeys = false;
        this._refreshSelections();
        saveConfig(this._cfg);
    }

    _setConfigValue(key, value, restart = false) {
        if (this._cfg[key] === value) {
            this._refreshSelections();
            return;
        }
        this._cfg[key] = value;
        this._saveAndRefresh();
        if (restart) {
            restartDaemon();
            this._setDaemonBusy('перезапуск...');
            this._scheduleStatusRefreshes();
        }
    }

    _setTextBackend(value) {
        if (this._cfg.text_backend === value) {
            this._refreshSelections();
            applyInputChannel(value);
            return;
        }
        this._cfg.text_backend = value;
        if (value !== 'ime')
            this._cfg.nanda_precognition = false;
        this._saveAndRefresh();
        applyInputChannel(value);
        restartDaemon();
        this._setDaemonBusy('перезапуск...');
        this._scheduleStatusRefreshes();
    }

    _setTrigger(id) {
        if (this._cfg.trigger === id) {
            this._refreshSelections();
            return;
        }
        this._cfg.trigger = id;
        this._saveAndRefresh();
        restartDaemon();
        this._setDaemonBusy('перезапуск...');
        this._scheduleStatusRefreshes();
    }

    _setForceKey(key, id) {
        if (this._cfg[key] === id) {
            this._refreshSelections();
            return;
        }
        this._cfg[key] = id;
        this._saveAndRefresh();
        restartDaemon();
        this._setDaemonBusy('перезапуск...');
        this._scheduleStatusRefreshes();
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

    _setButtonActive(button, active) {
        button.set_style_class_name(active ? 'button' : 'button flat');
        button.style = SEGMENT_BUTTON_STYLE;
        if (button.set_checked)
            button.set_checked(active);
    }

    _triggerLabel(id) {
        return optionLabel(TRIGGER_OPTIONS, id, 'Двойной Shift');
    }

    _forceKeyLabel(id) {
        return optionLabel(FORCE_KEY_OPTIONS, id, 'Правый Ctrl');
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
        const lem = `LEM ${this._cfg.lem_2_words ? '2' : '-'}${this._cfg.lem_3_words ? '/3' : ''}`;
        const force = this._cfg.force_layout_hotkeys ? 'RU/EN хоткеи' : 'RU/EN выкл';
        return `${this._engineLabel()} · ${this._safetyLabel()} · ${this._cfg.replace_words} сл. · ${lem} · ${autoSwitch} · ${force} · ${this._triggerLabel(this._cfg.trigger)}`;
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
            const isRu = this._mgr.currentSource?.id === 'ru';
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
        this._indicator = new LayIndicator(this._service);
        Main.panel.addToStatusArea('lay', this._indicator, 0, 'right');
    }

    disable() {
        this._indicator?.destroy(); this._indicator = null;
        this._service?.disable();   this._service   = null;
    }
}
