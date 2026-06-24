import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Shell from 'gi://Shell';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {getInputSourceManager} from 'resource:///org/gnome/shell/ui/status/keyboard.js';

import {APP_VERSION} from './tray_support.js';

export function normalizeLayoutKind(id) {
    const value = String(id ?? '').trim().toLowerCase();
    if (!value)
        return '';
    if (value === 'ru' || value === 'lay-ime-ru' || value.includes(':ru') || value.includes('russian') || value.includes('rus'))
        return 'ru';
    if (value === 'us' || value === 'en' || value === 'lay-ime-us' || value.includes(':us') || value.includes('english') || value.includes('eng'))
        return 'us';
    return value;
}

export function currentLayoutKind() {
    try {
        return normalizeLayoutKind(getInputSourceManager().currentSource?.id ?? '');
    } catch(e) {
        return '';
    }
}

function imeEngineForLayoutKind(kind) {
    if (kind === 'ru')
        return 'lay-ime-ru';
    if (kind === 'us')
        return 'lay-ime-us';
    return '';
}

export function syncIbusEngineForCurrentLayout() {
    const engine = imeEngineForLayoutKind(currentLayoutKind());
    if (!engine)
        return false;
    try {
        GLib.spawn_command_line_async(`ibus engine ${engine}`);
        return true;
    } catch(e) {
        log(`[lay-extension] IBus engine sync failed for ${engine}: ${e}`);
        return false;
    }
}

function syncIbusEngineSoon() {
    GLib.timeout_add(GLib.PRIORITY_DEFAULT, 25, () => {
        syncIbusEngineForCurrentLayout();
        return GLib.SOURCE_REMOVE;
    });
}

export function activateLayoutId(id) {
    try {
        const mgr = getInputSourceManager();
        for (const i in mgr.inputSources)
            if (mgr.inputSources[i].id === id) {
                mgr.inputSources[i].activate();
                syncIbusEngineSoon();
                return true;
            }

        const targetKind = normalizeLayoutKind(id);
        for (const i in mgr.inputSources)
            if (normalizeLayoutKind(mgr.inputSources[i].id) === targetKind) {
                mgr.inputSources[i].activate();
                syncIbusEngineSoon();
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

export function focusedWindowInfo() {
    const win = focusedWindow();
    if (!win)
        return null;

    let app = null;
    try { app = Shell.WindowTracker.get_default().get_window_app(win); } catch(e) {}
    const windowId = String(win.get_id?.() ?? '').trim();
    const stableSequence = String(win.get_stable_sequence?.() ?? '').trim();
    const pid = String(win.get_pid?.() ?? '').trim();
    const description = String(win.get_description?.() ?? '').trim();
    const appId = String(app?.get_id?.() ?? '').trim();
    const appName = String(app?.get_name?.() ?? '').trim();
    const wmClass = String(win.get_wm_class?.() ?? '').trim();
    const wmClassInstance = String(win.get_wm_class_instance?.() ?? '').trim();
    const title = String(win.get_title?.() ?? '').trim();

    if (appId)
        return {kind: 'app_id', value: appId, label: appName || appId, windowId, stableSequence, pid, description, appId, wmClass, wmClassInstance, title};
    if (wmClass)
        return {kind: 'wm_class', value: wmClass, label: wmClass, windowId, stableSequence, pid, description, appId, wmClass, wmClassInstance, title};
    if (wmClassInstance)
        return {kind: 'wm_class_instance', value: wmClassInstance, label: wmClassInstance, windowId, stableSequence, pid, description, appId, wmClass, wmClassInstance, title};
    return null;
}

const DBUS_XML = `
<node>
  <interface name="io.github.radislabus_star.LayDaemon">
    <method name="Ping"><arg name="reply" direction="out" type="s"/></method>
    <method name="Version"><arg name="version" direction="out" type="s"/></method>
    <method name="TypeText"><arg name="text" direction="in" type="s"/></method>
    <method name="ReplaceText">
      <arg name="move_left" direction="in" type="u"/>
      <arg name="backspaces" direction="in" type="u"/>
      <arg name="text" direction="in" type="s"/>
      <arg name="move_right" direction="in" type="u"/>
      <arg name="layout_id" direction="in" type="s"/>
      <arg name="success" direction="out" type="b"/>
    </method>
    <method name="ActivateLayout">
      <arg name="id" direction="in" type="s"/>
      <arg name="success" direction="out" type="b"/>
    </method>
    <method name="CurrentLayout"><arg name="id" direction="out" type="s"/></method>
    <method name="NextLayout"><arg name="success" direction="out" type="b"/></method>
    <method name="ListLayouts"><arg name="layouts" direction="out" type="s"/></method>
    <method name="FocusedWindowInfo"><arg name="json" direction="out" type="s"/></method>
  </interface>
</node>`;

const DBUS_PATH = '/io/github/radislabus_star/LayDaemon';

export class LayDaemonService {
    enable() {
        const seat = Clutter.get_default_backend().get_default_seat();
        this._vdev = seat.create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
        this._dbus = Gio.DBusExportedObject.wrapJSObject(DBUS_XML, this);
        this._dbus.export(Gio.DBus.session, DBUS_PATH);
    }

    disable() {
        this._dbus?.unexport();
        this._dbus = null;
        this._vdev = null;
    }

    Ping() {
        return 'pong from lay-extension';
    }

    Version() {
        return APP_VERSION;
    }

    TypeText(text) {
        if (Main.inputMethod?.commit) {
            try {
                Main.inputMethod.commit(text);
                return;
            } catch(e) {}
        }
        this._typeTextByKeyvals(text);
    }

    ReplaceText(moveLeft, backspaces, text, moveRight, layoutId) {
        try {
            this._tapKeyval(Clutter.KEY_Left, Number(moveLeft));
            this._tapKeyval(Clutter.KEY_BackSpace, Number(backspaces));
            if (layoutId)
                activateLayoutId(layoutId);
            this.TypeText(text);
            this._tapKeyval(Clutter.KEY_Right, Number(moveRight));
            return true;
        } catch(e) {
            log(`[lay-extension] ReplaceText failed: ${e}`);
            return false;
        }
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
            if (!kv)
                continue;
            this._tapKeyval(kv, 1);
        }
    }

    ActivateLayout(id) {
        return activateLayoutId(id);
    }

    CurrentLayout() {
        try {
            return getInputSourceManager().currentSource?.id ?? '';
        } catch(e) {
            return '';
        }
    }

    NextLayout() {
        try {
            const mgr = getInputSourceManager();
            const ids = Object.keys(mgr.inputSources).sort((a, b) => a - b);
            const cur = ids.findIndex(i => mgr.inputSources[i].id === mgr.currentSource.id);
            mgr.inputSources[ids[(cur + 1) % ids.length]].activate();
            return true;
        } catch(e) {
            return false;
        }
    }

    ListLayouts() {
        try {
            const mgr = getInputSourceManager();
            return Object.keys(mgr.inputSources).sort((a, b) => a - b)
                .map(i => `${i}:${mgr.inputSources[i].type}:${mgr.inputSources[i].id}${mgr.inputSources[i].id === mgr.currentSource.id ? '*' : ''}`)
                .join(',');
        } catch(e) {
            return 'error:' + e;
        }
    }

    FocusedWindowInfo() {
        try {
            return JSON.stringify(focusedWindowInfo() ?? {});
        } catch(e) {
            return '{}';
        }
    }
}
