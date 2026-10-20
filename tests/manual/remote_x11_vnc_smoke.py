#!/usr/bin/python3
"""Owned GTK/RFB smoke. Never uses existing fields, sends Enter or restarts IBus.

Run under the project resource guard with the target KDE session environment.
RFB gestures are real VNC input; they do not certify physical evdev input or a
full browser matrix. The output directory is immutable per attempt.
"""
import argparse
import ctypes
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import time


def wait_stable_target(expected, read, pump, *, timeout=2, settle=.15,
                       clock=time.monotonic):
    """Require a settled target; a transient match is not an acceptance receipt."""
    end = clock()+timeout
    matched = None
    while clock() < end:
        pump(.002)
        current = read()
        if current['visible'] == expected:
            if matched is None:
                matched = clock()
            if clock()-matched >= settle:
                return current, matched
        elif matched is not None:
            raise AssertionError(f'target {expected!r} reverted to {current!r}')
    raise AssertionError(f'target {expected!r}, got {read()!r}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--port', type=int, default=5905)
    args = parser.parse_args()
    assert os.environ.get('LAY_RESOURCE_GUARD_ACTIVE') == '1'
    args.output.mkdir(parents=True, exist_ok=False)
    import gi
    gi.require_version('Gtk', '3.0')
    gi.require_version('Gdk', '3.0')
    gi.require_version('GdkX11', '3.0')
    from gi.repository import Gtk, Gdk, GdkX11
    window = Gtk.Window(title='Lay — owned VNC input check')
    window.set_default_size(900, 500)
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
    box.set_border_width(24)
    box.pack_start(Gtk.Label(label='Lay: isolated test field, no messages sent'), False, False, 0)
    entry = Gtk.Entry()
    box.pack_start(entry, False, False, 0)
    window.add(box)
    preedit = ['']
    entry.connect('preedit-changed', lambda _entry, text: preedit.__setitem__(0, text))
    records = []
    result = dict(scope='OWNED_GTK_REAL_RFB', phase='SETUP', physical_evdev='NOT_TESTED',
                  full_browser_matrix='NOT_TESTED', rows=records, status='RUNNING')
    result['seats'] = [dict(keyboard=s.get_keyboard().get_name(),
                           keyboard_id=s.get_keyboard().get_property('device-id'),
                           default=s == Gdk.Display.get_default().get_default_seat())
                       for s in Gdk.Display.get_default().list_seats() if s.get_keyboard()]
    focus_events = []
    window.connect('focus-in-event', lambda w,e: focus_events.append(dict(kind='in',device=e.get_device().get_name() if e.get_device() else None)) or False)
    window.connect('focus-out-event', lambda w,e: focus_events.append(dict(kind='out',device=e.get_device().get_name() if e.get_device() else None)) or False)
    result['focus_events'] = focus_events
    key_events = []
    for signal in ('key-press-event', 'key-release-event'):
        window.connect(signal, lambda _w, e, kind=signal: key_events.append(
            dict(kind=kind, keyval=e.keyval, code=e.hardware_keycode,
                 state=int(e.state))) or False)
    result['client_key_events'] = key_events
    def xi_focus():
        libx = ctypes.CDLL('libX11.so.6')
        libxi = ctypes.CDLL('libXi.so.6')
        libx.XOpenDisplay.argtypes = [ctypes.c_char_p]
        libx.XOpenDisplay.restype = ctypes.c_void_p
        libx.XCloseDisplay.argtypes = [ctypes.c_void_p]
        libxi.XIGetFocus.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.POINTER(ctypes.c_ulong)]
        display = libx.XOpenDisplay(None)
        assert display
        try:
            found = {}
            for seat in result['seats']:
                target = ctypes.c_ulong()
                libxi.XIGetFocus(display, seat['keyboard_id'], ctypes.byref(target))
                found[seat['keyboard']] = target.value
            return found
        finally:
            libx.XCloseDisplay(display)

    def core_held_keys():
        libx = ctypes.CDLL('libX11.so.6')
        libx.XOpenDisplay.argtypes = [ctypes.c_char_p]
        libx.XOpenDisplay.restype = ctypes.c_void_p
        libx.XCloseDisplay.argtypes = [ctypes.c_void_p]
        libx.XQueryKeymap.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
        display = libx.XOpenDisplay(None)
        assert display
        try:
            keys = (ctypes.c_ubyte*32)()
            libx.XQueryKeymap(display, keys)
            return [key for key in range(256) if keys[key//8] & (1 << (key % 8))]
        finally:
            libx.XCloseDisplay(display)
    prior = subprocess.check_output(['xdotool', 'getactivewindow'], text=True).strip()
    sock = None
    held = set()

    def pump(seconds=0):
        deadline = time.monotonic() + seconds
        while True:
            while Gtk.events_pending():
                Gtk.main_iteration_do(False)
            if time.monotonic() >= deadline:
                break
            time.sleep(.002)

    def state():
        return dict(committed=entry.get_text(), preedit=preedit[0],
                    visible=entry.get_text()+preedit[0], cursor=entry.get_position())

    def focus():
        active = int(subprocess.check_output(['xdotool', 'getactivewindow'], text=True))
        details = dict(window_active=window.is_active(), entry_focus=entry.has_focus(),
                       entry_is_focus=entry.is_focus(), active=active, owned=xid,
                       core_focus=subprocess.check_output(['xdotool', 'getwindowfocus'], text=True).strip())
        assert window.is_active() and entry.has_focus() and active == xid, details

    def receive(count):
        data = b''
        while len(data) < count:
            part = sock.recv(count-len(data))
            assert part, 'RFB closed'
            data += part
        return data

    def send(symbol, down):
        sock.sendall(struct.pack('!BBHI', 4, int(down), 0, symbol))
        if down:
            held.add(symbol)
        else:
            held.discard(symbol)

    def tap(symbol):
        focus()
        send(symbol, True)
        pump(.012)
        send(symbol, False)
        released = time.monotonic()
        pump(.012)
        return released

    def type_text(text):
        for char in text:
            tap(Gdk.unicode_to_keyval(ord(char)))

    def wait_target(expected):
        return wait_stable_target(expected, state, pump)

    def save_keymap(name):
        snapshot = subprocess.check_output(
            ['xkbcomp', '-w0', '-xkb', os.environ['DISPLAY'], '-'], timeout=3)
        (args.output/(name+'.xkb')).write_bytes(snapshot)

    def gesture(before, expected):
        focus()
        wait_target(before)
        for index in range(2):
            started = tap(0xffe1)
            if index == 0:
                pump(.045)
        try:
            actual, observed = wait_target(expected)
            return dict(status='PASS', expected=expected, actual=actual,
                        send_to_observed_ms=(observed-started)*1000, settle_ms=150)
        except AssertionError as error:
            return dict(status='FAIL', expected=expected, actual=state(), error=str(error))

    try:
        window.show_all()
        window.present()
        entry.grab_focus()
        pump(.4)
        xid = window.get_window().get_xid()
        subprocess.run(['xdotool', 'windowactivate', '--sync', str(xid)], check=True, timeout=3)
        result['setup_focus_before'] = subprocess.check_output(['xdotool', 'getwindowfocus'], text=True).strip()
        subprocess.run(['xdotool', 'windowfocus', '--sync', str(xid)], check=True, timeout=3)
        result['xi_focus_setup'] = xi_focus()
        sock = socket.create_connection(('127.0.0.1', args.port), 3)
        sock.settimeout(3)
        assert receive(12).startswith(b'RFB 003.')
        sock.sendall(b'RFB 003.008\n')
        count = receive(1)[0]
        assert 1 in receive(count), 'requires the preconfigured local VNC route'
        sock.sendall(b'\x01')
        assert receive(4) == b'\0'*4
        sock.sendall(b'\x01')
        init = receive(24)
        receive(struct.unpack('!I', init[20:24])[0])
        assert int(subprocess.check_output(['xdotool', 'getactivewindow'], text=True)) == xid
        origin = window.get_window().get_origin()
        rect = entry.get_allocation()
        point = (origin[-2]+rect.x+rect.width//2, origin[-1]+rect.y+rect.height//2)
        result['setup_click'] = point
        for mask in (0, 1, 0):
            sock.sendall(struct.pack('!BBHH', 5, mask, *point))
            pump(.05)
        end = time.monotonic()+2
        while not (window.is_active() and entry.has_focus()) and time.monotonic() < end:
            pump(.01)
        focus()
        # Select the configured RU engine in the actual focused field. KDE may
        # restore its per-window mode when a new window receives focus.
        activation = subprocess.run(['ibus', 'engine', 'lay-ime-ru'],
                                    capture_output=True, text=True, timeout=5)
        assert activation.returncode == 0, activation.stderr
        deadline = time.monotonic()+15
        while True:
            pump(.05)
            engine = subprocess.check_output(['ibus', 'engine'], text=True, timeout=3).strip()
            pid = subprocess.check_output(['systemctl', '--user', 'show',
                'lay-daemon.service', '-p', 'MainPID', '--value'], text=True, timeout=3).strip()
            journal = subprocess.check_output(['journalctl', '--user', '_PID='+pid,
                '--no-pager', '-o', 'cat'], text=True, timeout=3)
            if engine == 'lay-ime-ru' and 'X11 session keyboard ready' in journal:
                pump(.15)
                if subprocess.check_output(['ibus', 'engine'], text=True, timeout=3).strip() == engine:
                    break
            assert time.monotonic() < deadline, 'focused RU/native daemon setup did not settle'
        result['focused_engine'] = engine
        result['native_daemon_pid'] = pid
        save_keymap('focused')
        focus()
        result['core_held_before'] = core_held_keys()
        assert not result['core_held_before'], 'server has held core keys before test'
        result['phase'] = 'INPUT'
        cases = [('привет', 'ghbdtn'), ('а', 'f'),
                 ('ПрИвЕт', 'GhBdTn'), ('параллелепипед', 'gfhfkktktgbgtl')]
        result['expected_rows'] = len(cases)
        result['expected_gestures'] = len(cases)*6
        result['cadence_ms'] = dict(hold=12, release_gap=12, double_shift_gap=45,
                                    between_gestures=50, wait_for_each_target=True)
        for seed, other in cases:
            entry.set_text('')  # Setup affects only this owned widget.
            pump(.1)
            type_text(seed)
            wait_target(seed)
            save_keymap('seed-'+str(len(records)))
            row = dict(seed=seed, boundary='active', gestures=[])
            records.append(row)
            for index in range(6):
                expected = other if index % 2 == 0 else seed
                before = seed if index % 2 == 0 else other
                action = gesture(before, expected)
                row['gestures'].append(action)
                if action['status'] != 'PASS':
                    save_keymap('failure')
                    break  # Retain the first failure, never retry until green.
                pump(.05)
            row['status'] = 'PASS' if len(row['gestures']) == 6 and all(
                x['status'] == 'PASS' for x in row['gestures']) else 'FAIL'
            if row['status'] != 'PASS':
                break
        pump(.1)
        if records and records[-1]['status'] == 'PASS':
            assert state()['visible'] == records[-1]['seed'], 'late final mutation'
        result['status'] = 'PASS' if len(records) == len(cases) and all(
            r['status'] == 'PASS' for r in records) else 'FAIL'
        image = Gdk.pixbuf_get_from_window(window.get_window(), 0, 0,
                    window.get_allocated_width(), window.get_allocated_height())
        image.savev(str(args.output/'field.png'), 'png', [], [])
    except Exception as error:
        result.update(status='BLOCKED' if result['phase'] == 'SETUP' else 'FAIL',
                      error=str(error), last_state=state())
    finally:
        if window.get_window():
            image = Gdk.pixbuf_get_from_window(window.get_window(), 0, 0,
                        window.get_allocated_width(), window.get_allocated_height())
            image.savev(str(args.output/'final-field.png'), 'png', [], [])
        if sock:
            for symbol in held.copy():
                send(symbol, False)
            sock.close()
        result['release_messages_sent'] = not held
        pump(.15)
        result['core_held_after'] = core_held_keys()
        result['core_keys_released'] = not result['core_held_after']
        if not result['core_keys_released']:
            result.update(status='FAIL', release_error='server still has held core keys')
        window.destroy()
        pump()
        subprocess.run(['xdotool', 'windowactivate', prior], capture_output=True, timeout=3)
        (args.output/'result.json').write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps(dict(status=result['status'], rows=len(records), receipt=str(args.output/'result.json'))))
    return int(result['status'] != 'PASS')


if __name__ == '__main__':
    raise SystemExit(main())
