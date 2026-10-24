import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';

// Evaluate the actual bridge with GI imports replaced at the platform boundary.
const source = readFileSync(new URL('../../extension/lay@radislabus-star.github.io/dbus_service.js', import.meta.url), 'utf8')
    .replace(/^import .*;\n/gm, '')
    .replace(/^export /gm, '');

function bridge(stage, Clutter) {
    const events = [];
    const context = vm.createContext({
        global: {stage}, Clutter,
        Gio: {DBus: {session: 'session'}, DBusExportedObject: {wrapJSObject: () => ({
            export: () => events.push('export'), unexport: () => events.push('unexport'),
        })}},
    });
    vm.runInContext(source + '\nthis.service = new LayDaemonService();', context);
    return {service: context.service, events};
}

function seat() {
    return {create_virtual_device: kind => ({kind})};
}

test('GNOME 51 creates the keyboard through the stage context without the removed API', () => {
    const {service, events} = bridge({context: {get_backend: () => ({get_default_seat: seat})}},
        {InputDeviceType: {KEYBOARD_DEVICE: 1}});
    service.enable();
    assert.equal(service._vdev.kind, 1);
    assert.deepEqual(events, ['export']);
    service.disable();
    assert.deepEqual(events, ['export', 'unexport']);
    assert.equal(service._vdev, null);
});

test('older supported Shell keeps its existing Clutter backend', () => {
    const {service, events} = bridge({}, {
        get_default_backend: () => ({get_default_seat: seat}),
        InputDeviceType: {KEYBOARD_DEVICE: 1},
    });
    service.enable();
    assert.equal(service._vdev.kind, 1);
    assert.deepEqual(events, ['export']);
});

test('when both APIs exist the context owns the keyboard and is not bypassed', () => {
    const {service} = bridge({context: {get_backend: () => ({get_default_seat: seat})}}, {
        get_default_backend: () => { throw new Error('obsolete backend used'); },
        InputDeviceType: {KEYBOARD_DEVICE: 1},
    });
    service.enable();
    assert.equal(service._vdev.kind, 1);
});

test('missing keyboard backend fails before exporting a working-looking endpoint', () => {
    const {service, events} = bridge({}, {InputDeviceType: {KEYBOARD_DEVICE: 1}});
    assert.throws(() => service.enable());
    assert.deepEqual(events, []);
});

for (const modern of [false, true]) {
    test(`recent-action layout supports ${modern ? 'GNOME 51 orientation' : 'older vertical property'}`, () => {
        let box;
        const recent = readFileSync(new URL('../../extension/lay@radislabus-star.github.io/recent_actions_menu.js', import.meta.url), 'utf8')
            .replace(/^import[\s\S]*?;\n/gm, '').replace(/^export /gm, '');
        const context = vm.createContext({
            St: {
                BoxLayout: class {
                    constructor(props) {
                        assert.equal('vertical' in props, false);
                        if (modern) this.orientation = 0;
                        box = this;
                    }
                    add_child() {}
                },
                Label: class { clutter_text = {}; },
            },
            Clutter: {Orientation: {VERTICAL: 1}},
            Pango: {WrapMode: {WORD_CHAR: 1}},
            PopupMenu: {PopupBaseMenuItem: class { add_child() {} }},
            COMPACT_SUBTITLE_STYLE: '',
        });
        vm.runInContext(recent, context);
        context.recentActionRow({_actionKindLabel: () => '', _shortActionText: () => ''}, {});
        assert.equal(modern ? box.orientation : box.vertical, modern ? 1 : true);
    });
}
