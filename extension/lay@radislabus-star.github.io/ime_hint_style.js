// GNOME's auxiliary label discards IBusText attributes. Dim that display
// actor while keeping its system color; the engine still owns the text.
const HINT_OPACITY = 128;

export class ImeHintStyle {
    constructor(manager) {
        this._manager = manager;
        this._label = manager?._candidatePopup?._auxText;
        this._bus = manager?._ibus;
        this._appliedOpacity = null;
        if (!this._label || !this._bus)
            return;

        this._engineId = this._bus.connect('global-engine-changed', (_bus, name) => {
            this._update(name);
        });
        this._disconnectId = this._bus.connect('disconnected', () => this._update(null));
        // Covers an already-active engine discovered asynchronously on connect.
        this._textId = this._label.connect('notify::text', () => {
            this._update(this._manager._currentEngineName);
        });
        this._update(manager._currentEngineName);
    }

    _update(name) {
        if (name !== 'lay-ime-ru' && name !== 'lay-ime-us') {
            this._restore();
            return;
        }
        if (this._appliedOpacity !== null)
            return;
        this._originalOpacity = this._label.opacity;
        this._appliedOpacity = Math.min(this._originalOpacity, HINT_OPACITY);
        this._label.opacity = this._appliedOpacity;
    }

    _restore() {
        if (this._appliedOpacity === null)
            return;
        if (this._label.opacity === this._appliedOpacity)
            this._label.opacity = this._originalOpacity;
        this._appliedOpacity = null;
    }

    destroy() {
        if (this._engineId)
            this._bus.disconnect(this._engineId);
        if (this._disconnectId)
            this._bus.disconnect(this._disconnectId);
        if (this._textId)
            this._label.disconnect(this._textId);
        this._engineId = this._disconnectId = this._textId = 0;
        this._restore();
    }
}
