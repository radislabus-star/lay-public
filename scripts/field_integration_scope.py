"""Predeclared source scope, sealed into the existing remote snapshot.

This is an accidental-edit guard, not a sandbox or a second runtime owner.
The canon and native acceptance remain separate, mandatory evidence.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess

CONTRACT = "docs/development/active-field-integration-scope.json"
SELF = "scripts/field_integration_scope.py"
IME = "src/bin/lay_ibus_engine/"
COMPONENTS = {
    "kde-chrome": (IME + "kde/chrome.rs", IME + "kde/chrome/"),
    "kde-terminal": (IME + "kde/terminal.rs",),
    "kde-focus": ("src/bin/lay_daemon/kde/",),
    "kde-common": (IME + "kde/",),
    "gnome": (IME + "gnome/", "src/bin/lay_daemon/layout_controller/gnome_dbus.rs",
              "src/bin/lay_daemon/layout_controller/gnome_dbus_tests.rs", "extension/"),
    "correction": ("src/nanda_wave/", "src/phrase_reader/", "src/layout_autoswitch/"),
    "tooling": ("tests/", "scripts/tests/", "scripts/test_lanes/", "scripts/test-lanes/",
                SELF, "scripts/check-field-integration-scope.py",
                "scripts/check-lay-component.py", "scripts/check-lay-component.sh"),
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(root: Path, *args: str) -> bytes:
    return subprocess.check_output(["git", *args], cwd=root, stderr=subprocess.DEVNULL)


def safe_path(value: str) -> bool:
    return (isinstance(value, str) and bool(value) and "\\" not in value
            and not PurePosixPath(value).is_absolute()
            and all(part not in (".", "..", ".git") for part in value.split("/"))
            and str(PurePosixPath(value)) == value)


def required(root: Path) -> bool:
    # Deleting the declaration cannot turn the guard off. Tiny legacy fixture
    # repositories without either tracked/current marker retain their old API.
    if any((root / name).exists() for name in (CONTRACT, SELF)):
        return True
    if not (root / ".git").exists():
        return False
    return bool(git(root, "ls-files", "--", CONTRACT, SELF))


def load_contract(root: Path) -> dict:
    spec = json.loads((root / CONTRACT).read_text())
    if spec.get("schema") != "lay.field-integration-scope.v1":
        raise ValueError("invalid field integration scope schema")
    base = spec.get("base", "")
    if not isinstance(base, str) or not re.fullmatch(r"[0-9a-f]{40}", base):
        raise ValueError("scope base must be an immutable full commit SHA")
    if (not isinstance(spec.get("components"), list) or not spec["components"]
            or any(item not in COMPONENTS for item in spec["components"])):
        raise ValueError("scope needs known, nonempty components")
    paths = spec.get("shared_paths")
    if not isinstance(paths, list) or any(not safe_path(p) for p in paths):
        raise ValueError("shared paths must be exact repository paths")
    if any(any(c in p for c in "*?[") for p in paths):
        raise ValueError("shared paths cannot contain globs")
    adr_path = spec.get("decision", "")
    if not safe_path(adr_path) or not adr_path.startswith("docs/architecture/decisions/"):
        raise ValueError("scope needs an owning architecture decision")
    adr = json.loads((root / adr_path).read_text())
    if (adr.get("schema") != "lay.architecture-decision.v1"
            or not set(paths).issubset(set(adr.get("protected_paths", [])))):
        raise ValueError("scope shared paths are not covered by its architecture decision")
    return spec


def category(path: str) -> str:
    if path in {"scripts/test-lanes/manifest.json", "scripts/test-lanes/known_failures.json"}:
        return "shared"
    # Specific clients precede the KDE facade. A Chrome declaration therefore
    # cannot authorize a terminal/focus/common edit (or vice versa).
    for component, prefixes in COMPONENTS.items():
        if any(path.startswith(p) if p.endswith("/") else path == p for p in prefixes):
            return component
    if (path.startswith(("docs/", "graphify-out/"))
            or ("/" not in path and path.endswith(".md")) or path.endswith("/README.md")
            or path == "src/generated/architecture_graph_receipt.json"):
        return "documentation"
    return "shared"


def version_only(path: str, before: bytes | None, after: bytes | None) -> bool:
    if before is None or after is None:
        return False
    if path.endswith("/metadata.json") and path.startswith("extension/"):
        try:
            left, right = json.loads(before), json.loads(after)
            for key in ("version", "version-name"):
                left.pop(key, None)
                right.pop(key, None)
            return left == right
        except (ValueError, AttributeError):
            return False
    patterns = {
        "Cargo.toml": rb'(\[package\]\s*\n(?:(?!\n\[).)*?\nversion\s*=\s*)"[^"\n]+"',
        "Cargo.lock": rb'(\[\[package\]\]\nname = "lay"\nversion = )"[^"\n]+"',
        "extension/lay@radislabus-star.github.io/tray_support.js":
            rb"(export const APP_VERSION = )'[^'\n]+'",
    }
    pattern = patterns.get(path)
    if pattern is None:
        return False
    def normalized(data):
        value, count = re.subn(pattern, rb'\1"<lay-version>"', data, flags=re.DOTALL)
        return value if count == 1 else None
    old, new = normalized(before), normalized(after)
    return old is not None and old == new


def build_receipt(root: Path, source_head: str, changed_paths) -> dict | None:
    if not required(root):
        return None
    spec = load_contract(root)
    base = git(root, "rev-parse", "--verify", "--end-of-options", spec["base"] + "^{commit}").decode().strip()
    if git(root, "ls-tree", "-z", base, "--", spec["decision"]):
        raise ValueError("shared scope requires a new decision after its declared baseline")
    # Reuse the canonical planner's NUL-safe tracked/untracked/deleted union.
    # Never compute this set from the synthetic remote workspace HEAD.
    changed = changed_paths(root, base)
    rows, errors = [], []
    for name in changed:
        if not safe_path(name):
            raise ValueError("unsafe changed path")
        path = root / name
        if path.is_symlink() or (path.exists() and not path.is_file()):
            raise ValueError("unsupported scoped file: " + name)
        after = path.read_bytes() if path.exists() else None
        entry = git(root, "ls-tree", "-z", base, "--", name)
        before = git(root, "show", base + ":" + name) if entry else None
        old_exec = bool(entry and entry.split(b" ", 1)[0] == b"100755")
        new_exec = bool(path.exists() and path.stat().st_mode & 0o111)
        owner = category(name)
        if version_only(name, before, after) and old_exec == new_exec:
            owner = "version-metadata"
        allowed = (name in spec["shared_paths"] or
                   (old_exec == new_exec and owner in {"documentation", "version-metadata", *spec["components"]}))
        if not allowed:
            errors.append(name + " (" + owner + ")")
        rows.append({"path": name, "category": owner,
                     "sha256": sha(after) if after is not None else None,
                     "mode": path.stat().st_mode & 0o777 if path.exists() else None})
    if errors:
        raise ValueError("outside declared field scope: " + ", ".join(errors))
    return {"schema": "lay.field-integration-scope-receipt.v1", "source_head": source_head,
            "base": base, "contract_sha256": sha((root / CONTRACT).read_bytes()), "changes": rows}


def verify_snapshot_receipt(req: dict, root: Path) -> None:
    ledger = {row["path"]: row for row in req["files"]}
    receipt = req.get("field_integration_scope")
    if CONTRACT not in ledger and SELF not in ledger:
        if receipt is not None:
            raise ValueError("scope receipt without its source declaration")
        return
    if not isinstance(receipt, dict) or CONTRACT not in ledger or SELF not in ledger:
        raise ValueError("snapshot is missing its field scope declaration/receipt")
    declaration = load_contract(root)
    if (receipt.get("schema") != "lay.field-integration-scope-receipt.v1"
            or receipt.get("source_head") != req["source_head"]
            or receipt.get("base") != declaration["base"]
            or receipt.get("contract_sha256") != ledger[CONTRACT]["sha256"]):
        raise ValueError("scope receipt does not bind this snapshot")
    changes = receipt.get("changes")
    if not isinstance(changes, list) or len({row["path"] for row in changes}) != len(changes):
        raise ValueError("invalid scope changed-file inventory")
    # The scope may span more commits than the selected development plan.
    if not set(req["plan"]["changed"]).issubset({row["path"] for row in changes}):
        raise ValueError("scope receipt omits a development change")
    for row in changes:
        frozen = ledger.get(row["path"])
        if frozen is None:
            if row["sha256"] is not None or row["mode"] is not None:
                raise ValueError("scope receipt is missing a frozen file")
        elif any(row[key] != frozen[key] for key in ("sha256", "mode")):
            raise ValueError("scope receipt/file drift: " + row["path"])
