import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {test} from 'node:test';

const source = readFileSync(new URL('../../extension/lay@radislabus-star.github.io/ime_hint_style.js', import.meta.url));
const {ImeHintStyle, InlineHintProjection, InlineHintView} = await import(`data:text/javascript;base64,${source.toString('base64')}`);

test('inline projection preserves owner/revision and Unicode for the shared protocol', () => {
    const projection = new InlineHintProjection();
    const cases = JSON.parse(readFileSync(new URL('./fixtures/inline-hint-frames-v1.json', import.meta.url)));
    for (const item of cases) {
        if ('owner' in item) {
            projection.reset(item.owner);
            continue;
        }
        const result = projection.apply(JSON.stringify(item.frame));
        assert.equal(result.accepted, item.accepted, JSON.stringify(item));
        if (result.accepted) {
            assert.equal(result.frame !== null, item.visible);
            if (result.frame)
                assert.equal(result.frame.text, item.frame.text);
        }
    }
});

test('invalid inline data cannot consume a revision', () => {
    const projection = new InlineHintProjection();
    projection.reset(':1.8');
    for (const raw of ['null', '[]', '{', 'x'.repeat(16385),
        JSON.stringify({version: 1, stream: ':1.8', revision: '9'.repeat(21)})])
        assert.equal(projection.apply(raw).accepted, false);
    assert.equal(projection.revision, -1n);
});

class Signals {
    callbacks = new Map();
    connect(name, callback) {
        const id = (this.nextId ?? 0) + 1;
        this.nextId = id;
        this.callbacks.set(id, {name, callback});
        return id;
    }
    disconnect(id) { assert.equal(this.callbacks.delete(id), true); }
    emit(name, ...args) {
        for (const item of this.callbacks.values()) {
            if (item.name === name)
                item.callback(this, ...args);
        }
    }
}

function inlineViewFixture({height = 24, monospace = true} = {}) {
    const display = new Signals();
    const overview = Object.assign(new Signals(), {visible: false});
    const sessionMode = Object.assign(new Signals(), {isLocked: false});
    let label, appeared, vanished, signal;
    const pending = [];
    let callCount = 0;
    const connection = {
        signal_subscribe(...args) { signal = args.at(-1); return 1; },
        signal_unsubscribe() {},
        call(...args) { callCount++; pending.push(args.at(-1)); },
        call_finish(raw) {
            if (raw instanceof Error) throw raw;
            return {deep_unpack: () => [raw]};
        },
    };
    const Gio = {
        BusType: {SESSION: 0}, BusNameWatcherFlags: {NONE: 0},
        DBusSignalFlags: {NONE: 0}, DBusCallFlags: {NONE: 0},
        bus_watch_name(...args) { [appeared, vanished] = args.slice(-2); return 1; },
        bus_unwatch_name() {},
    };
    const St = {Label: class {
        constructor() { label = this; this.visible = false; }
        hide() { this.visible = false; }
        show() { this.visible = true; }
        set_position(x, y) { this.position = [x, y]; }
        get_preferred_height() { return [16, 16]; }
        set_clip() {}
        destroy() { this.visible = false; }
    }};
    const Main = {uiGroup: {add_child() {}}, overview, sessionMode,
        layoutManager: {monitors: [{x: 0, y: 0, width: 1000, height: 1000}]}};
    const view = new InlineHintView({Gio, GLib: {VariantType: class {}}, St, Main, display});
    appeared(connection, 'service', ':1.8');
    const frame = (revision = '1') => JSON.stringify({version: 1, stream: ':1.8', revision,
        owner: '/field', text: 'ерка', rect: [10, 20, 0, height], monospace});
    return {view, label, display, overview, sessionMode,
        finish: (revision = '1') => {
            assert.ok(pending.length > 0, 'expected a real pending snapshot call');
            pending.shift()(connection, revision instanceof Error ? revision : frame(revision));
        },
        signal: (revision = '1') => signal(null, null, null, null, null,
            {deep_unpack: () => [frame(revision)]}),
        pendingCount: () => pending.length,
        callCount: () => callCount,
        vanished: () => vanished()};
}

test('line-sized cursor retains inline font sizing without dimming', () => {
    const f = inlineViewFixture();
    f.finish();
    assert.match(f.label.style, /color: #000000; background-color: #ffffff;/);
    assert.match(f.label.style, /font-size: 18px;/);
    assert.match(f.label.style, /font-family: monospace;/);
    assert.doesNotMatch(f.label.style, /opacity/);
    assert.deepEqual(f.label.position, [10, 20]);
    f.view.destroy();
});

test('tall input rectangle uses the theme font centered beside entered text', () => {
    const f = inlineViewFixture({height: 360, monospace: false});
    f.finish();
    assert.equal(f.label.visible, true);
    assert.doesNotMatch(f.label.style, /font-size|font-family|opacity/);
    assert.deepEqual(f.label.position, [10, 192]);
    f.view.destroy();
});

test('queued positive signal cannot repaint after focus or session invalidation', () => {
    for (const [object, event] of [['display', 'notify::focus-window'],
        ['overview', 'showing'], ['sessionMode', 'updated']]) {
        const f = inlineViewFixture();
        f[object].emit(event);
        f.signal('2');
        assert.equal(f.label.visible, false, 'old queued positive must stay hidden');
        f.finish();
        assert.equal(f.label.visible, false, event);
        assert.equal(f.pendingCount(), 1, 'latest focus needs its own revision barrier');
        f.signal('3');
        assert.equal(f.label.visible, false, 'signals remain hidden until the barrier replies');
        f.finish('3');
        f.signal('2');
        assert.equal(f.label.visible, false, 'positive older than barrier remains rejected');
        f.signal('4');
        assert.equal(f.label.visible, true, 'fresh engine signal remains usable');
        f.view.destroy();
        assert.equal(f.display.callbacks.size + f.overview.callbacks.size + f.sessionMode.callbacks.size, 0);
    }
});

test('focus bursts coalesce revision barriers instead of accumulating calls', () => {
    const f = inlineViewFixture();
    f.finish();
    assert.equal(f.label.visible, true);
    f.display.emit('notify::focus-window');
    f.display.emit('notify::focus-window');
    assert.equal(f.pendingCount(), 1);
    assert.equal(f.callCount(), 2);
    f.finish('2');
    assert.equal(f.pendingCount(), 1, 'one barrier for the final focus');
    f.signal('3');
    assert.equal(f.label.visible, false);
    f.finish('3');
    assert.equal(f.pendingCount(), 0);
    f.signal('4');
    assert.equal(f.label.visible, true);
    f.view.destroy();
});

test('failed focus barrier stays hidden without retries and recovers on a new focus', () => {
    const f = inlineViewFixture();
    f.finish();
    f.display.emit('notify::focus-window');
    f.finish(new Error('bus failure'));
    f.signal('2');
    assert.equal(f.label.visible, false);
    assert.equal(f.pendingCount(), 0);
    assert.equal(f.callCount(), 2);
    f.display.emit('notify::focus-window');
    f.finish('2');
    f.signal('3');
    assert.equal(f.label.visible, true);
    f.view.destroy();
});

test('current inline snapshot paints but owner loss or destruction rejects late replies', () => {
    const current = inlineViewFixture();
    current.finish();
    assert.equal(current.label.visible, true);
    assert.equal(current.label.text, 'ерка');
    current.view.destroy();
    for (const action of ['vanished', 'destroy']) {
        const f = inlineViewFixture();
        if (action === 'destroy') f.view.destroy(); else f.vanished();
        f.finish();
        assert.equal(f.label.visible, false, action);
        if (action !== 'destroy') f.view.destroy();
    }
});

function fixture(engine, opacity = 255) {
    const label = new Signals();
    label.opacity = opacity;
    label.style = 'color: system-theme; font-size: 14px;';
    const bus = new Signals();
    const manager = {_candidatePopup: {_auxText: label}, _ibus: bus, _currentEngineName: engine};
    return {label, bus, manager, hint: new ImeHintStyle(manager)};
}

test('both Lay engines use black on white only on the hint and retain other styles', () => {
    const f = fixture('lay-ime-ru');
    assert.equal(f.label.opacity, 255);
    assert.match(f.label.style, /font-size: 14px;.*color: #000000; background-color: #ffffff;$/);
    f.bus.emit('global-engine-changed', 'lay-ime-us');
    assert.equal(f.label.opacity, 255);
    assert.match(f.label.style, /color: #000000; background-color: #ffffff;$/);
    f.hint.destroy();
    assert.equal(f.label.opacity, 255);
    assert.equal(f.label.style, 'color: system-theme; font-size: 14px;');
    assert.equal(f.bus.callbacks.size, 0);
    assert.equal(f.label.callbacks.size, 0);
});

test('engine switch immediately restores other IMEs, even before manager cache updates', () => {
    const f = fixture('lay-ime-ru');
    for (const other of ['xkb:ru::rus', 'pinyin', 'lay-ime-ru-other', null]) {
        f.bus.emit('global-engine-changed', other);
        assert.equal(f.label.opacity, 255);
        assert.equal(f.label.style, 'color: system-theme; font-size: 14px;');
        f.bus.emit('global-engine-changed', 'lay-ime-us');
        assert.equal(f.label.opacity, 255);
        assert.match(f.label.style, /color: #000000; background-color: #ffffff;$/);
    }
    f.hint.destroy();
});

test('non-Lay engine remains unchanged and late discovery styles the next hint', () => {
    const f = fixture(null);
    assert.equal(f.label.opacity, 255);
    f.manager._currentEngineName = 'lay-ime-ru';
    f.label.emit('notify::text');
    assert.equal(f.label.opacity, 255);
    assert.match(f.label.style, /color: #000000; background-color: #ffffff;$/);
    f.bus.emit('disconnected');
    assert.equal(f.label.opacity, 255);
    assert.equal(f.label.style, 'color: system-theme; font-size: 14px;');
    f.hint.destroy();
});

test('disable preserves a later style and opacity owned by another extension', () => {
    const f = fixture('lay-ime-us');
    f.label.opacity = 200;
    f.label.style = 'color: red;';
    f.hint.destroy();
    f.hint.destroy();
    assert.equal(f.label.opacity, 200);
    assert.equal(f.label.style, 'color: red;');
});

test('hint colors do not change opacity provided by another owner', () => {
    const f = fixture('lay-ime-ru', 80);
    assert.equal(f.label.opacity, 80);
    assert.match(f.label.style, /color: #000000; background-color: #ffffff;$/);
    f.hint.destroy();
    assert.equal(f.label.opacity, 80);
});

test('missing Shell display internals leave the extension usable', () => {
    for (const manager of [null, {}, {_candidatePopup: {}}, {_ibus: new Signals()}])
        new ImeHintStyle(manager).destroy();
});
