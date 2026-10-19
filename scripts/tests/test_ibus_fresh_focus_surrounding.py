"""Compile actual upstream function bodies against a recording transport.

Run only on the remote development host under its resource guard. Source
contracts do not establish real-client acceptance or upstream release quality.
"""
from pathlib import Path
import argparse
import datetime
import hashlib
import json
import os
import re
import shlex
import subprocess


def extract(source, name):
    match = re.search(r"(?:static\s+)?void\s+" + re.escape(name) + r"\s*\(", source)
    if not match:
        return None
    start = source.index("{", match.end())
    depth = 1
    end = start + 1
    while depth:
        depth += (source[end] == "{") - (source[end] == "}")
        end += 1
    return source[match.start():end]


def extract_reset(source):
    reset = extract(source, "bus_engine_proxy_reset")
    if reset is not None:
        return reset
    # Execute upstream's actual Reset-generating macro for the RED control.
    macro = re.search(r"#define DEFINE_FUNCTION\(Name, name\).*?(?=\n\nDEFINE_FUNCTION)",
                      source, re.DOTALL)
    assert macro and "DEFINE_FUNCTION (Reset, reset)" in source
    return macro.group() + "\nDEFINE_FUNCTION (Reset, reset)\n#undef DEFINE_FUNCTION"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--lifetime-only", action="store_true")
    args = parser.parse_args()
    args.output.mkdir(exist_ok=False)
    proxy = args.source / "bus/engineproxy.c"
    client = args.source / "client/wayland/ibuswaylandim.c"
    if args.lifetime_only:
        setter = args.source / "src/ibusinputcontext.c"
        names = ((setter, "ibus_input_context_set_surrounding_text"),
                 (client, "_context_require_surrounding_text_cb"),
                 (client, "handle_surrounding_text"))
        parts = [extract(p.read_text(), name) for p, name in names]
        assert all(parts), "Ownership regression requires the actual Require handler"
        parts.append(extract_reset(proxy.read_text()))
        (args.output / "actual-lifetime-functions.h").write_text("\n".join(parts) + "\n")
        fixture = Path(__file__).with_name("ibus-fresh-focus-lifetime.c")
        flags = shlex.split(subprocess.check_output(["pkg-config", "--cflags", "--libs", "ibus-1.0"], text=True))
        command = ["cc", "-std=c11", "-Werror=implicit-function-declaration", "-I", str(args.output),
                   str(fixture), "-o", str(args.output / "lifetime"), *flags]
        build = subprocess.run(command, text=True, capture_output=True)
        (args.output / "build.log").write_text(build.stdout + build.stderr)
        assert build.returncode == 0, build.stderr[-2000:]
        run = subprocess.run([str(args.output / "lifetime")], env=dict(os.environ, G_DEBUG="fatal-warnings"),
                             text=True, capture_output=True)
        receipt = dict(status="PASS" if run.returncode == 0 else "FAIL", returncode=run.returncode,
                       at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                       output=run.stdout, stderr=run.stderr,
                       source={str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                               for p in (setter, client, proxy, fixture, Path(__file__))},
                       scope="real IBusText/GObject and actual InputContext setter; recording D-Bus; native NOT_TESTED")
        (args.output / "RESULT.json").write_text(json.dumps(receipt, indent=2) + "\n")
        print(json.dumps(receipt))
        return 0 if run.returncode == 0 else 1
    parts = []
    for name in ("bus_engine_proxy_set_surrounding_text", "bus_engine_proxy_focus_in",
                 "bus_engine_proxy_focus_out"):
        part = extract(proxy.read_text(), name)
        assert part, name
        parts.append(part)
    parts.append(extract_reset(proxy.read_text()))
    part = extract(client.read_text(), "_context_require_surrounding_text_cb")
    # The baseline has no handler: its Require signal delivers nothing. Keep
    # this absence executable so RED is behavioral, not a compilation failure.
    parts.append(part or "static void _context_require_surrounding_text_cb(IBusInputContext *c, gpointer p) {}")
    parts.append(extract(client.read_text(), "handle_surrounding_text"))
    (args.output / "actual-upstream-functions.h").write_text("\n".join(parts) + "\n")
    fixture = Path(__file__).with_name("ibus-fresh-focus-surrounding.c")
    flags = shlex.split(subprocess.check_output(["pkg-config", "--cflags", "--libs", "glib-2.0"], text=True))
    command = ["cc", "-std=c11", "-Wall", "-Werror=implicit-function-declaration",
               "-Wno-unused-function", "-I", str(args.output), str(fixture),
               "-o", str(args.output / "regression"), *flags]
    build = subprocess.run(command, text=True, capture_output=True)
    (args.output / "build.log").write_text(build.stdout + build.stderr)
    assert build.returncode == 0, build.stderr[-2000:]
    rows = []
    for case in range(1, 13):
        result = subprocess.run([str(args.output / "regression"), str(case)], text=True, capture_output=True)
        rows.append(dict(case=case, returncode=result.returncode, output=result.stdout.strip(), stderr=result.stderr))
    passed = sum(row["returncode"] == 0 for row in rows)
    receipt = dict(status="PASS" if passed == 12 else "FAIL", at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                   passed=passed, failed=12-passed, rows=rows,
                   source={str(p): hashlib.sha256(p.read_bytes()).hexdigest()
                           for p in (proxy, client, fixture, Path(__file__))},
                   scope="extracted actual upstream function decisions with recording transport; native NOT_TESTED")
    (args.output / "RESULT.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))
    return 0 if passed == 12 else 1


if __name__ == "__main__":
    raise SystemExit(main())
