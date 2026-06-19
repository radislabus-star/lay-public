#!/usr/bin/env python3
"""Small focused GTK entry used by live runtime smoke tests."""

from __future__ import annotations

import argparse
import sys

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

        def done(_entry: Gtk.Entry) -> None:
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
