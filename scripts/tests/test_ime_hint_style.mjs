import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {test} from 'node:test';

const source = readFileSync(new URL('../../extension/lay@radislabus-star.github.io/ime_hint_style.js', import.meta.url));
const {ImeHintStyle} = await import(`data:text/javascript;base64,${source.toString('base64')}`);

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

function fixture(engine, opacity = 255) {
    const label = new Signals();
    label.opacity = opacity;
    label.style = 'color: system-theme; font-size: 14px;';
    const bus = new Signals();
    const manager = {_candidatePopup: {_auxText: label}, _ibus: bus, _currentEngineName: engine};
    return {label, bus, manager, hint: new ImeHintStyle(manager)};
}

test('both Lay engines dim only the hint and preserve the system color', () => {
    const f = fixture('lay-ime-ru');
    assert.equal(f.label.opacity, 128);
    assert.equal(f.label.style, 'color: system-theme; font-size: 14px;');
    f.bus.emit('global-engine-changed', 'lay-ime-us');
    assert.equal(f.label.opacity, 128);
    f.hint.destroy();
    assert.equal(f.label.opacity, 255);
    assert.equal(f.bus.callbacks.size, 0);
    assert.equal(f.label.callbacks.size, 0);
});

test('engine switch immediately restores other IMEs, even before manager cache updates', () => {
    const f = fixture('lay-ime-ru');
    for (const other of ['xkb:ru::rus', 'pinyin', 'lay-ime-ru-other', null]) {
        f.bus.emit('global-engine-changed', other);
        assert.equal(f.label.opacity, 255);
        f.bus.emit('global-engine-changed', 'lay-ime-us');
        assert.equal(f.label.opacity, 128);
    }
    f.hint.destroy();
});

test('non-Lay engine remains unchanged and late discovery styles the next hint', () => {
    const f = fixture(null);
    assert.equal(f.label.opacity, 255);
    f.manager._currentEngineName = 'lay-ime-ru';
    f.label.emit('notify::text');
    assert.equal(f.label.opacity, 128);
    f.bus.emit('disconnected');
    assert.equal(f.label.opacity, 255);
    f.hint.destroy();
});

test('disable preserves a later opacity owned by another extension', () => {
    const f = fixture('lay-ime-us');
    f.label.opacity = 200;
    f.hint.destroy();
    f.hint.destroy();
    assert.equal(f.label.opacity, 200);
});

test('an already fainter hint is never made brighter', () => {
    const f = fixture('lay-ime-ru', 80);
    assert.equal(f.label.opacity, 80);
    f.hint.destroy();
    assert.equal(f.label.opacity, 80);
});

test('missing Shell display internals leave the extension usable', () => {
    for (const manager of [null, {}, {_candidatePopup: {}}, {_ibus: new Signals()}])
        new ImeHintStyle(manager).destroy();
});
