import Adw from 'gi://Adw';

import {APP_ICON_NAME} from './tray_support.js';
import {createSettingsPage} from './settings_view.js';

const app = new Adw.Application({
    application_id: 'io.github.radislabus_star.LaySettings',
});

let settingsWindow = null;

app.connect('activate', () => {
    if (settingsWindow) {
        settingsWindow.present();
        return;
    }

    const window = new Adw.PreferencesWindow({
        application: app,
        title: 'Настройки Lay',
        default_width: 640,
        default_height: 780,
    });
    window.set_icon_name(APP_ICON_NAME);
    window.add(createSettingsPage());
    window.connect('close-request', () => {
        settingsWindow = null;
        return false;
    });
    settingsWindow = window;
    window.present();
});

app.run([]);
