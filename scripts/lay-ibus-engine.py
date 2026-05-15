#!/usr/bin/env python3
"""Experimental IBus bridge for lay.

This process is intentionally small: the Rust daemon still owns correction
decisions. When this IBus engine is active and focused, the daemon can ask it
over DBus to delete the committed tail before the caret and commit replacement
text through the real input-method channel.
"""

from __future__ import annotations

import argparse
import os
import re
import signal
import sys
from typing import Optional

import gi

gi.require_version("IBus", "1.0")
from gi.repository import GLib, Gio, IBus  # noqa: E402


BUS_NAME = "io.github.radislabus_star.LayIme"
BUS_PATH = "/io/github/radislabus_star/LayIme"
BUS_IFACE = "io.github.radislabus_star.LayIme"

DBUS_XML = f"""
<node>
  <interface name="{BUS_IFACE}">
    <method name="Ping">
      <arg name="reply" direction="out" type="s"/>
    </method>
    <method name="ReplaceTail">
      <arg name="backspaces" direction="in" type="u"/>
      <arg name="text" direction="in" type="s"/>
      <arg name="success" direction="out" type="b"/>
    </method>
    <method name="Focused">
      <arg name="focused" direction="out" type="b"/>
    </method>
  </interface>
</node>
"""


class LayEngine(IBus.Engine):
    active: Optional["LayEngine"] = None

    def __init__(self, bus: IBus.Bus, object_path: str, engine_name: str) -> None:
        super().__init__(connection=bus.get_connection(), object_path=object_path)
        self.engine_name = engine_name
        self.focused = False

    def do_focus_in(self) -> None:
        self.focused = True
        LayEngine.active = self

    def do_focus_out(self) -> None:
        self.focused = False
        if LayEngine.active is self:
            LayEngine.active = None

    def do_process_key_event(self, _keyval: int, _keycode: int, _state: int) -> bool:
        # The Rust daemon still listens to physical keys. This engine is only a
        # focused text-edit bridge, so normal typing passes through untouched.
        return False

    def replace_tail(self, backspaces: int, text: str) -> bool:
        if backspaces < 0:
            return False
        if backspaces == 0 and not text:
            return False
        if backspaces:
            self.delete_surrounding_text(-backspaces, backspaces)
        if text:
            self.commit_text(IBus.Text.new_from_string(text))
        return True


class LayFactory(IBus.Factory):
    def __init__(self, bus: IBus.Bus) -> None:
        super().__init__(connection=bus.get_connection(), object_path=IBus.PATH_FACTORY)
        self.bus = bus
        self.engine_id = 0

    def do_create_engine(self, engine_name: str) -> LayEngine:
        safe_name = re.sub(r"[^a-zA-Z0-9_/]", "_", engine_name)
        path = f"/io/github/radislabus_star/LayIme/engine/{safe_name}/{self.engine_id}"
        self.engine_id += 1
        return LayEngine(self.bus, path, engine_name)


class LayImeDbus:
    def __init__(self) -> None:
        self.node = Gio.DBusNodeInfo.new_for_xml(DBUS_XML)
        self.connection = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        self.registration_id = self.connection.register_object(
            BUS_PATH,
            self.node.interfaces[0],
            self._handle_method_call,
            None,
            None,
        )
        self.owner_id = Gio.bus_own_name_on_connection(
            self.connection,
            BUS_NAME,
            Gio.BusNameOwnerFlags.REPLACE,
            None,
            None,
        )

    def _handle_method_call(
        self,
        _connection: Gio.DBusConnection,
        _sender: str,
        _object_path: str,
        _interface_name: str,
        method_name: str,
        parameters: GLib.Variant,
        invocation: Gio.DBusMethodInvocation,
    ) -> None:
        try:
            if method_name == "Ping":
                focused = "focused" if LayEngine.active is not None else "no-focus"
                invocation.return_value(GLib.Variant("(s)", (f"lay-ibus-engine {focused}",)))
                return
            if method_name == "Focused":
                invocation.return_value(GLib.Variant("(b)", (LayEngine.active is not None,)))
                return
            if method_name == "ReplaceTail":
                backspaces, text = parameters.unpack()
                engine = LayEngine.active
                ok = bool(engine and engine.replace_tail(int(backspaces), str(text)))
                invocation.return_value(GLib.Variant("(b)", (ok,)))
                return
            invocation.return_dbus_error(BUS_IFACE + ".UnknownMethod", method_name)
        except Exception as exc:  # pragma: no cover - defensive DBus boundary.
            invocation.return_dbus_error(BUS_IFACE + ".Error", str(exc))

    def close(self) -> None:
        if self.registration_id:
            self.connection.unregister_object(self.registration_id)
            self.registration_id = 0
        if self.owner_id:
            Gio.bus_unown_name(self.owner_id)
            self.owner_id = 0


class LayImeApp:
    def __init__(self, exec_by_ibus: bool) -> None:
        self.mainloop = GLib.MainLoop()
        self.bus = IBus.Bus()
        self.bus.connect("disconnected", self._quit)
        self.factory = LayFactory(self.bus)
        self.dbus = LayImeDbus()
        if exec_by_ibus:
            self.bus.request_name("org.freedesktop.IBus.Lay", 0)
        else:
            component = IBus.Component(
                name="org.freedesktop.IBus.Lay",
                description="Lay IME bridge",
                version="0.1.0",
                license="MIT",
                author="radislabus",
                homepage="https://github.com/radislabus-star/lay-public",
                textdomain="lay",
            )
            component.add_engine(
                IBus.EngineDesc(
                    name="lay-ime-us",
                    longname="Lay IME US",
                    description="Lay IME US input-method bridge",
                    language="en",
                    license="MIT",
                    author="radislabus",
                    icon="input-keyboard",
                    layout="us",
                    symbol="lay",
                )
            )
            component.add_engine(
                IBus.EngineDesc(
                    name="lay-ime-ru",
                    longname="Lay IME RU",
                    description="Lay IME RU input-method bridge",
                    language="ru",
                    license="MIT",
                    author="radislabus",
                    icon="input-keyboard",
                    layout="ru",
                    symbol="lay",
                )
            )
            self.bus.register_component(component)

    def _quit(self, *_args: object) -> None:
        self.mainloop.quit()

    def run(self) -> None:
        signal.signal(signal.SIGINT, lambda *_args: self._quit())
        signal.signal(signal.SIGTERM, lambda *_args: self._quit())
        self.mainloop.run()


def component_xml(exec_path: str) -> str:
    return f"""<?xml version="1.0" encoding="utf-8"?>
<component>
  <name>org.freedesktop.IBus.Lay</name>
  <description>Lay input-method bridge</description>
  <exec>{exec_path} --ibus</exec>
  <version>0.1.0</version>
  <author>radislabus</author>
  <license>MIT</license>
  <homepage>https://github.com/radislabus-star/lay-public</homepage>
  <textdomain>lay</textdomain>
  <engines>
    <engine>
      <name>lay-ime-us</name>
      <language>en</language>
      <license>MIT</license>
      <author>radislabus</author>
      <icon>input-keyboard</icon>
      <layout>us</layout>
      <longname>Lay IME US</longname>
      <description>Lay IME US input-method bridge</description>
      <rank>50</rank>
    </engine>
    <engine>
      <name>lay-ime-ru</name>
      <language>ru</language>
      <license>MIT</license>
      <author>radislabus</author>
      <icon>input-keyboard</icon>
      <layout>ru</layout>
      <longname>Lay IME RU</longname>
      <description>Lay IME RU input-method bridge</description>
      <rank>50</rank>
    </engine>
  </engines>
</component>
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ibus", action="store_true", help="run when started by ibus-daemon")
    parser.add_argument("--xml", action="store_true", help="print component XML")
    args = parser.parse_args()

    if args.xml:
        exec_path = os.path.expanduser("~/.local/bin/lay-ibus-engine")
        print(component_xml(exec_path), end="")
        return 0

    IBus.init()
    LayImeApp(exec_by_ibus=args.ibus).run()
    return 0


if __name__ == "__main__":
    sys.exit(main())
