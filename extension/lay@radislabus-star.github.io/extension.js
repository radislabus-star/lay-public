/* extension.js — LOADER (никогда не меняется)
 *
 * Копирует lay-impl.js во временный файл с уникальным именем,
 * чтобы обойти кэш модулей GJS. Это позволяет обновлять код
 * через disable → enable без logout.
 *
 * GNOME Shell 45, 46, 47, 50 — ES modules
 */

import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';

export default class LayExtension extends Extension {

    enable() {
        const implSrc  = `${this.path}/lay-impl.js`;
        // Уникальный путь = GJS не найдёт в кэше → свежий import
        const cacheDir = `${GLib.get_user_cache_dir()}/lay`;
        try { GLib.mkdir_with_parents(cacheDir, 0o700); } catch(e) {}
        this._tmpImpl  = `${cacheDir}/lay-impl-${Date.now()}.js`;

        try {
            const [, bytes] = Gio.File.new_for_path(implSrc).load_contents(null);
            Gio.File.new_for_path(this._tmpImpl).replace_contents(
                bytes, null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);
        } catch(e) {
            log(`[lay-extension] loader: не удалось скопировать impl: ${e}`);
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

        // Удаляем временный файл
        if (this._tmpImpl) {
            try { Gio.File.new_for_path(this._tmpImpl).delete(null); } catch(e) {}
            this._tmpImpl = null;
        }
    }
}
