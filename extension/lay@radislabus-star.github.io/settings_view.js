import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import Gtk from 'gi://Gtk';

import {
    APP_RELEASE_DATE,
    APP_URL,
    APP_VERSION,
    FORCE_KEY_OPTIONS,
    INPUT_MODE_OPTIONS,
    LAYOUT_BACKEND_OPTIONS,
    SAFETY_OPTIONS,
    TRIGGER_OPTIONS,
    applyInputChannel,
    loadConfig,
    openDiagnosticsLog,
    openUri,
    restartDaemon,
    saveConfig,
} from './tray_support.js';

function runStatusCommand(args) {
    try {
        const process = Gio.Subprocess.new(args, Gio.SubprocessFlags.STDOUT_PIPE);
        const [, stdout] = process.communicate_utf8(null, null);
        return process.get_successful() ? String(stdout ?? '').trim() : '';
    } catch(e) {
        return '';
    }
}

function runtimeStatus() {
    const daemon = runStatusCommand(['systemctl', '--user', 'is-active', 'lay-daemon.service']);
    const engine = runStatusCommand(['ibus', 'engine']);
    const daemonLabel = daemon === 'active' ? 'работает' : 'остановлен';
    return `Демон ${daemonLabel}${engine ? ` · ${engine}` : ''}`;
}

function addSuffixButton(row, label, iconName, callback) {
    const button = new Gtk.Button({
        icon_name: iconName,
        tooltip_text: label,
        valign: Gtk.Align.CENTER,
        css_classes: ['flat'],
    });
    button.connect('clicked', callback);
    row.add_suffix(button);
    row.activatable_widget = button;
    return button;
}

export class LaySettingsView {
    constructor() {
        this.cfg = loadConfig();
        this.page = new Adw.PreferencesPage({
            title: 'Lay',
            icon_name: 'input-keyboard-symbolic',
        });
        this._buildInputGroup();
        this._buildShortcutsGroup();
        this._buildCompatibilityGroup();
        this._buildDiagnosticsGroup();
        this._buildAboutGroup();
    }

    _save({restart = false, channel = null} = {}) {
        saveConfig(this.cfg);
        if (channel)
            applyInputChannel(channel);
        else if (restart)
            restartDaemon();
    }

    _switchRow(title, subtitle, key, options = {}) {
        const row = new Adw.SwitchRow({
            title,
            subtitle,
            active: !!this.cfg[key],
        });
        row.connect('notify::active', () => {
            this.cfg[key] = row.active;
            this._save(options);
        });
        return row;
    }

    _debugLogRow() {
        const row = new Adw.SwitchRow({
            title: 'Подробный журнал действий',
            subtitle: 'Записывать решения, backend и NANDA trace',
            active: !!this.cfg.debug_action_log,
        });
        row.connect('notify::active', () => {
            this.cfg.debug_action_log = row.active;
            this.cfg.nanda_trace = row.active;
            this.cfg.nanda_trace_text = row.active;
            this._save();
        });
        return row;
    }

    _comboRow(title, subtitle, key, options, saveOptions = {}) {
        const model = Gtk.StringList.new(options.map(([, label]) => label));
        const row = new Adw.ComboRow({title, subtitle, model});
        const selected = Math.max(0, options.findIndex(([id]) => id === this.cfg[key]));
        row.selected = selected;
        row.connect('notify::selected', () => {
            const id = options[row.selected]?.[0];
            if (!id || id === this.cfg[key])
                return;
            this.cfg[key] = id;
            if (key === 'text_backend')
                this.cfg.nanda_precognition = id === 'ime';
            this._save({...saveOptions, channel: key === 'text_backend' ? id : null});
        });
        return row;
    }

    _timingRow(title, subtitle, key, minimum, maximum, step) {
        const row = Adw.SpinRow.new_with_range(minimum, maximum, step);
        row.title = title;
        row.subtitle = subtitle;
        row.value = this.cfg[key];
        row.connect('notify::value', () => {
            const value = Math.round(row.value);
            if (value === this.cfg[key])
                return;
            this.cfg[key] = value;
            this._save({restart: true});
        });
        return row;
    }

    _buildInputGroup() {
        const group = new Adw.PreferencesGroup({
            title: 'Ввод',
            description: 'Подсказки и автоматические исправления.',
        });
        group.add(this._comboRow(
            'Режим ввода',
            'Быстрый ввод или подсказки внутри строки',
            'text_backend',
            INPUT_MODE_OPTIONS
        ));
        group.add(this._switchRow(
            'Помощь при наборе',
            'Предлагать продолжения и исправления во время ввода',
            'typing_assist',
            {restart: true}
        ));
        group.add(this._switchRow(
            'Автозамена',
            'Применять доказанные исправления после пробела',
            'auto_replace',
            {restart: true}
        ));
        group.add(this._comboRow(
            'Осторожность',
            'Сколько независимых доказательств нужно для замены',
            'correction_safety',
            SAFETY_OPTIONS,
            {restart: true}
        ));
        group.add(this._switchRow(
            'Следовать языку исправления',
            'Оставлять раскладку исправленного текста активной',
            'auto_switch_layout',
            {restart: true}
        ));
        group.add(this._switchRow(
            'Запоминать ручные правки',
            'Использовать принятые и отклонённые варианты для локального обучения',
            'learning_log'
        ));
        group.add(this._switchRow(
            'Показывать хвост в скобках',
            'Форматировать IME-продолжение как [хвост]',
            'ime_bracket_candidates',
            {restart: true}
        ));
        this.page.add(group);
    }

    _buildShortcutsGroup() {
        const group = new Adw.PreferencesGroup({
            title: 'Клавиши',
            description: 'Ручное исправление и прямой выбор языка.',
        });
        group.add(this._comboRow(
            'Исправить последнее слово',
            'Основная команда ручного восстановления; повтор отменяет автозамену',
            'trigger',
            TRIGGER_OPTIONS,
            {restart: true}
        ));
        group.add(this._timingRow(
            'Длительность одиночной клавиши',
            'Лимит для одиночных Shift/Ctrl/Alt; Double Shift от удержания не зависит',
            'tap_max_ms',
            100,
            800,
            25
        ));
        group.add(this._timingRow(
            'Интервал двойного Shift',
            'Максимальная пауза между двумя нажатиями, мс',
            'shift_window_ms',
            150,
            1000,
            25
        ));
        const forceSwitch = this._switchRow(
            'Отдельные клавиши RU / EN',
            'Включить прямое переключение на русский и английский',
            'force_layout_hotkeys',
            {restart: true}
        );
        const ruKey = this._comboRow('Русский', '', 'force_ru_key', FORCE_KEY_OPTIONS, {restart: true});
        const enKey = this._comboRow('Английский', '', 'force_en_key', FORCE_KEY_OPTIONS, {restart: true});
        const syncSensitivity = () => {
            ruKey.sensitive = forceSwitch.active;
            enKey.sensitive = forceSwitch.active;
        };
        forceSwitch.connect('notify::active', syncSensitivity);
        syncSensitivity();
        group.add(forceSwitch);
        group.add(ruKey);
        group.add(enKey);
        this.page.add(group);
    }

    _buildCompatibilityGroup() {
        const group = new Adw.PreferencesGroup({
            title: 'Совместимость',
            description: 'Обычно эти параметры определяются автоматически.',
        });
        group.add(this._comboRow(
            'Среда переключения раскладки',
            'Менять только при неверном автоматическом определении среды',
            'layout_backend',
            LAYOUT_BACKEND_OPTIONS,
            {restart: true}
        ));
        this.page.add(group);
    }

    _buildDiagnosticsGroup() {
        const group = new Adw.PreferencesGroup({
            title: 'Диагностика',
            description: 'Включайте журнал только на время разбора проблемы.',
        });
        group.add(this._debugLogRow());

        const statusRow = new Adw.ActionRow({
            title: 'Состояние служб',
            subtitle: runtimeStatus(),
        });
        addSuffixButton(statusRow, 'Обновить состояние', 'view-refresh-symbolic', () => {
            statusRow.subtitle = runtimeStatus();
        });
        group.add(statusRow);

        const logsRow = new Adw.ActionRow({
            title: 'Журнал Lay',
            subtitle: 'Последние сообщения daemon и онлайн-полей',
        });
        addSuffixButton(logsRow, 'Открыть журнал', 'utilities-terminal-symbolic', () => openDiagnosticsLog());
        group.add(logsRow);
        this.page.add(group);
    }

    _buildAboutGroup() {
        const group = new Adw.PreferencesGroup({title: 'О программе'});
        const row = new Adw.ActionRow({
            title: `Lay ${APP_VERSION}`,
            subtitle: `Версия от ${APP_RELEASE_DATE}`,
        });
        addSuffixButton(row, 'Открыть GitHub', 'web-browser-symbolic', () => openUri(APP_URL));
        group.add(row);
        this.page.add(group);
    }
}

export function createSettingsPage() {
    return new LaySettingsView().page;
}
