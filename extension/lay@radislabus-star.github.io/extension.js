/* extension.js — LOADER (никогда не меняется)
 *
 * Копирует runtime-модули во временную папку с уникальным именем, чтобы
 * обойти кэш модулей GJS. Это позволяет обновлять код через disable → enable
 * без logout.
 *
 * GNOME Shell 45, 46, 47, 50 — ES modules
 */

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

export default class LayExtension extends Extension {

    enable() {
        const extensionPath = this.path ?? this.dir?.get_path();
        if (!extensionPath) {
            log('[lay-extension] loader: не удалось определить путь extension');
            return;
        }

        const modules = [
            'lay-impl.js',
            'tray_support.js',
            'dbus_service.js',
            'recent_actions_menu.js',
        ];

        // Уникальная папка = GJS не найдёт в кэше → свежий import, а relative
        // imports внутри lay-impl.js продолжат работать.
        const cacheDir = `${GLib.get_user_cache_dir()}/lay`;
        try { GLib.mkdir_with_parents(cacheDir, 0o700); } catch(e) {}
        this._tmpDir = `${cacheDir}/extension-${Date.now()}`;
        this._tmpImpl = `${this._tmpDir}/lay-impl.js`;

        try {
            GLib.mkdir_with_parents(this._tmpDir, 0o700);
            for (const name of modules) {
                const [, bytes] = Gio.File.new_for_path(`${extensionPath}/${name}`).load_contents(null);
                Gio.File.new_for_path(`${this._tmpDir}/${name}`).replace_contents(
                    bytes, null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);
            }
        } catch(e) {
            log(`[lay-extension] loader: не удалось скопировать runtime modules: ${e}`);
            return;
        }

        import(`file://${this._tmpImpl}`).then(mod => {
            this._impl = new mod.LayImpl(this);
            this._impl.enable();
            log('[lay-extension] impl loaded ✓');
        }).catch(e => log(`[lay-extension] loader import error: ${e}`));
    }

    disable() {
        try { this._impl?.disable(); } catch(e) {}
        this._impl = null;

        // Удаляем временные файлы.
        if (this._tmpDir) {
            for (const name of ['lay-impl.js', 'tray_support.js', 'dbus_service.js', 'recent_actions_menu.js']) {
                try { Gio.File.new_for_path(`${this._tmpDir}/${name}`).delete(null); } catch(e) {}
            }
            try { Gio.File.new_for_path(this._tmpDir).delete(null); } catch(e) {}
            this._tmpImpl = null;
            this._tmpDir = null;
        }
    }
}
