import Pango from 'gi://Pango';
import St from 'gi://St';

import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

import {
    COMPACT_SUBTITLE_STYLE,
    RECENT_ACTIONS_PATH,
    clearRecentActions,
    loadRecentActions,
    summarizeRecentActions,
} from './tray_support.js';

export function createRecentActionsMenu(indicator) {
    const item = new PopupMenu.PopupSubMenuMenuItem('Последние действия', false);
    indicator._recentActionsItem = item;
    populateRecentActionsMenu(indicator, item);
    return item;
}

export function populateRecentActionsMenu(indicator, item) {
    item.menu.removeAll();
    const actions = loadRecentActions(5);
    if (actions.length === 0) {
        item.menu.addMenuItem(indicator._mutedTextRow('пока нет действий'));
        return;
    }
    item.menu.addMenuItem(indicator._mutedTextRow(summarizeRecentActions(loadRecentActions(20))));
    item.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
    item.menu.addMenuItem(clearRecentActionsItem(indicator));
    item.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
    for (const action of actions)
        item.menu.addMenuItem(recentActionRow(indicator, action));
}

export function refreshRecentActions(indicator) {
    if (indicator._recentActionsItem)
        populateRecentActionsMenu(indicator, indicator._recentActionsItem);
}

function recentActionRow(indicator, action) {
    const item = new PopupMenu.PopupBaseMenuItem({activate: false, reactive: false, can_focus: false});
    item.reactive = false;
    item.can_focus = false;
    item.style = 'padding:4px 12px;';
    const box = new St.BoxLayout({
        vertical: true,
        x_expand: true,
        style: 'spacing:2px;',
    });
    const title = new St.Label({
        text: `${indicator._actionKindLabel(action.kind)} · ${Number(action.elapsed_ms ?? 0)}мс`,
        style: 'font-weight:bold; font-size:86%;',
    });
    const text = new St.Label({
        text: `${indicator._shortActionText(action.from)} → ${indicator._shortActionText(action.to)}`,
        style: COMPACT_SUBTITLE_STYLE,
    });
    text.clutter_text.line_wrap = true;
    text.clutter_text.line_wrap_mode = Pango.WrapMode.WORD_CHAR;
    box.add_child(title);
    box.add_child(text);
    item.add_child(box);
    return item;
}

function clearRecentActionsItem(indicator) {
    const item = new PopupMenu.PopupMenuItem('Очистить последние действия');
    item.connect('activate', () => {
        if (clearRecentActions()) {
            refreshRecentActions(indicator);
            indicator._refreshStats();
            indicator._notify('Журнал очищен', 'Последние действия удалены.');
        } else {
            indicator._notify('Не удалось очистить', RECENT_ACTIONS_PATH, true);
        }
    });
    return item;
}
