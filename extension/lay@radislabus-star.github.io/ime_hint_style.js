// GNOME's auxiliary label discards IBusText attributes. Style only that
// suggestion actor, like the inline suffix; never color user-entered text.
const HINT_STYLE = 'color: #000000; background-color: #ffffff;';

// The bus owner's unique name and monotonic revision reject late snapshots
// and signals after an engine restart. This is presentation, never edit proof.
export class InlineHintProjection {
    constructor() {
        this.reset('');
    }

    reset(owner) {
        this.owner = owner;
        this.revision = -1n;
    }

    apply(raw) {
        if (typeof raw !== 'string' || raw.length > 16384)
            return {accepted: false};
        let value;
        try {
            value = JSON.parse(raw);
            if (value?.version !== 1 || !this.owner || value.stream !== this.owner ||
                typeof value.revision !== 'string' || !/^\d{1,20}$/.test(value.revision))
                return {accepted: false};
            const revision = BigInt(value.revision);
            if (typeof value.text !== 'string' ||
                [...value.text].length > 512 || /[\u0000-\u001f\u007f-\u009f]/.test(value.text) ||
                typeof value.monospace !== 'boolean')
                return {accepted: false};
            const r = value.rect;
            if (r !== null && (!Array.isArray(r) || r.length !== 4 || !r.every(Number.isInteger) ||
                Math.abs(r[0]) > 100000 || Math.abs(r[1]) > 100000 ||
                r[2] < 0 || r[2] > 4096 || r[3] <= 0 || r[3] > 512))
                return {accepted: false};
            if (revision <= this.revision)
                return {accepted: false, valid: true};
            this.revision = revision;
            return {accepted: true, valid: true, frame: value.text && r !== null ? value : null};
        } catch (_) {
            return {accepted: false};
        }
    }
}

// Dependencies are injected so protocol tests remain ordinary Node tests and
// an already-loaded extension can stage this existing support module.
export class InlineHintView {
    constructor({Gio, GLib, St, Main, display}) {
        this._Gio = Gio;
        this._projection = new InlineHintProjection();
        this._focusSerial = 0;
        this._barrierRequired = false;
        this._snapshotRequest = null;
        this._label = new St.Label({reactive: false, can_focus: false, visible: false});
        Main.uiGroup.add_child(this._label);
        const invalidate = () => {
            this._focusSerial++;
            this._barrierRequired = true;
            this._label.hide();
            requestSnapshot();
        };
        this._connections = [
            [display, display.connect('notify::focus-window', invalidate)],
            [Main.overview, Main.overview.connect('showing', invalidate)],
            [Main.sessionMode, Main.sessionMode.connect('updated', invalidate)],
        ];
        const service = 'io.github.radislabus_star.LayIme';
        const path = '/io/github/radislabus_star/LayIme';
        const receive = (raw, paint = true) => {
            const result = this._projection.apply(raw);
            if (!result.accepted)
                return result.valid === true;
            const f = result.frame;
            if (!paint || this._barrierRequired || !f || Main.overview.visible || Main.sessionMode.isLocked) {
                this._label.hide();
                return true;
            }
            const [x, y, , height] = f.rect;
            const monitor = Main.layoutManager.monitors.find(m =>
                x >= m.x && x < m.x + m.width && y >= m.y && y < m.y + m.height);
            if (!monitor) {
                this._label.hide();
                return true;
            }
            this._label.text = f.text;
            this._label.style = `${HINT_STYLE} padding: 0; margin: 0; border-width: 0;${f.monospace ? ' font-family: monospace;' : ''}`;
            // Some clients report the whole input's height as the cursor
            // rectangle. Keep the theme font for such an area, not giant text.
            const naturalHeight = this._label.get_preferred_height(-1)[1];
            const lineSized = height <= naturalHeight * 2;
            if (lineSized)
                this._label.style += ` font-size: ${Math.max(8, Math.floor(height * 0.78))}px;`;
            const labelHeight = this._label.get_preferred_height(-1)[1];
            const offsetY = lineSized ? 0 : Math.max(0, Math.floor((height - labelHeight) / 2));
            this._label.set_position(x, y + offsetY);
            this._label.set_clip(0, 0, monitor.x + monitor.width - x, height);
            this._label.show();
            return true;
        };
        const requestSnapshot = () => {
            const owner = this._projection.owner;
            if (!owner || this._snapshotRequest)
                return;
            const request = {owner, focusSerial: this._focusSerial};
            this._snapshotRequest = request;
            this._connection.call(owner, path, service, 'InlineHintSnapshotV1', null,
                new GLib.VariantType('(s)'), Gio.DBusCallFlags.NONE, 1000, null,
                (c, result) => {
                    if (this._snapshotRequest !== request || this._projection.owner !== owner)
                        return;
                    this._snapshotRequest = null;
                    let valid = false;
                    const current = this._focusSerial === request.focusSerial;
                    try { valid = receive(c.call_finish(result).deep_unpack()[0], current); }
                    catch (_) { /* failed barrier keeps the view hidden, without retries */ }
                    if (!valid)
                        return;
                    if (current)
                        this._barrierRequired = false;
                    else
                        requestSnapshot(); // coalesce changes while the single request was pending
                });
        };
        const clear = () => {
            this._label.hide();
            this._snapshotRequest = null;
            this._barrierRequired = false;
            this._projection.reset('');
            if (this._signalId)
                this._connection.signal_unsubscribe(this._signalId);
            this._signalId = 0;
        };
        this._watchId = Gio.bus_watch_name(Gio.BusType.SESSION, service,
            Gio.BusNameWatcherFlags.NONE, (connection, _name, owner) => {
                clear();
                this._connection = connection;
                this._projection.reset(owner);
                this._signalId = connection.signal_subscribe(owner, service, 'InlineHintV1',
                    path, null, Gio.DBusSignalFlags.NONE,
                    (_c, _sender, _p, _i, _member, parameters) => receive(parameters.deep_unpack()[0]));
                requestSnapshot();
            }, clear);
    }

    destroy() {
        this._Gio.bus_unwatch_name(this._watchId);
        if (this._signalId)
            this._connection.signal_unsubscribe(this._signalId);
        this._projection.reset('');
        this._snapshotRequest = null;
        for (const [object, id] of this._connections)
            object.disconnect(id);
        this._label.destroy();
    }
}

export class ImeHintStyle {
    constructor(manager) {
        this._manager = manager;
        this._label = manager?._candidatePopup?._auxText;
        this._bus = manager?._ibus;
        this._appliedStyle = null;
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
        if (this._appliedStyle !== null)
            return;
        this._originalStyle = this._label.style;
        this._appliedStyle = `${this._originalStyle ?? ''}; ${HINT_STYLE}`;
        this._label.style = this._appliedStyle;
    }

    _restore() {
        if (this._appliedStyle === null)
            return;
        if (this._label.style === this._appliedStyle)
            this._label.style = this._originalStyle;
        this._appliedStyle = null;
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
