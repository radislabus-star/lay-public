import {ExtensionPreferences} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

import {createSettingsPage} from './settings_view.js';

export default class LayPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        window.set_title('Настройки Lay');
        window.set_default_size(640, 780);
        window.add(createSettingsPage());
    }
}
