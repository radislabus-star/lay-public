#!/usr/bin/env python3
"""Small focused GTK entry used by live runtime smoke tests."""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import GLib, Gtk  # noqa: E402


class EntryCapture(Gtk.Application):
    def __init__(self, title: str, text: str) -> None:
        super().__init__(application_id=None)
        self._title = title
        self._text = text

    def do_activate(self) -> None:
        window = Gtk.ApplicationWindow(application=self)
        window.set_title(self._title)
        window.set_default_size(520, 110)

        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        box.set_margin_top(12)
        box.set_margin_bottom(12)
        box.set_margin_start(12)
        box.set_margin_end(12)

        label = Gtk.Label(label=self._text)
        label.set_xalign(0)
        entry = Gtk.Entry()

        if os.environ.get("LAY_GTK_ENTRY_CAPTURE_NATIVE") == "1":
            # Native client signals, separate from the engine's output trace.
            # Exclusive creation prevents one case from overwriting another.
            trace = Path(os.environ["LAY_IBUS_TRACE_PATH"]).with_suffix(".gtk-native.jsonl")
            self._native_log = trace.open("x", encoding="utf-8")
            delegate = entry.get_delegate()

            def observe(kind: str, preedit: str | None = None, keyval: int | None = None) -> None:
                self._native_log.write(json.dumps({
                    "kind": kind,
                    "monotonic_ns": time.monotonic_ns(),
                    "pid": os.getpid(),
                    "title": self._title,
                    "text": entry.get_text(),
                    "caret": entry.get_position(),
                    "active": window.is_active(),
                    "focused": delegate.has_focus(),
                    "preedit": preedit,
                    "keyval": keyval,
                }, ensure_ascii=False) + "\n")
                self._native_log.flush()

            entry.connect("changed", lambda _entry: observe("changed"))
            delegate.connect("preedit-changed", lambda _text, preedit: observe("preedit", preedit))
            window.connect("notify::is-active", lambda *_args: observe("window-active"))
            keys = Gtk.EventControllerKey.new()
            keys.set_propagation_phase(Gtk.PropagationPhase.CAPTURE)
            keys.connect("key-released", lambda _keys, keyval, *_args: observe("key-release", keyval=keyval))
            entry.add_controller(keys)
            self._native_observe = observe

        def done(_entry: Gtk.Entry) -> None:
            if hasattr(self, "_native_observe"):
                self._native_observe("accepted")
            if self._show_position:
                print(
                    f"{_entry.get_text()}\tpos={_entry.get_position()}",
                    flush=True,
                )
            else:
                print(_entry.get_text(), flush=True)
            self.quit()

        entry.connect("activate", done)
        box.append(label)
        box.append(entry)
        window.set_child(box)
        window.present()

        def focus_entry() -> bool:
            entry.grab_focus()
            return False

        GLib.idle_add(focus_entry)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--title", default="Lay runtime smoke")
    parser.add_argument("--text", default="Runtime smoke")
    parser.add_argument("--show-position", action="store_true")
    args = parser.parse_args()
    app = EntryCapture(args.title, args.text)
    app._show_position = args.show_position
    return app.run([])


if __name__ == "__main__":
    raise SystemExit(main())
