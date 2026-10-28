"""Real Git diff regressions; no runtime input, network, or user data."""
import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import field_integration_scope as scope

SPEC = importlib.util.spec_from_file_location("scope_test_dev", ROOT / "scripts/dev-check.py")
dev = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(dev)


class FieldScopeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.chrome = scope.IME + "kde/chrome.rs"
        self.gnome = scope.IME + "gnome/display.rs"
        self.common = scope.IME + "engine.rs"
        self.tray = "extension/lay@radislabus-star.github.io/tray_support.js"
        self.write(self.chrome, "// chrome\n")
        self.write(self.gnome, "// gnome\n")
        self.write(self.common, "// common owner\n")
        self.write(scope.SELF, "# existing scope marker\n")
        self.write("Cargo.toml", '[package]\nname = "lay"\nversion = "1.0.110"\n[dependencies]\nx = "1"\n')
        self.write("Cargo.lock", '[[package]]\nname = "lay"\nversion = "1.0.110"\n\n[[package]]\nname = "x"\nversion = "1"\n')
        self.write(self.tray, "export const APP_VERSION = '1.0.110';\nexport const enabled = true;\n")
        self.git("init", "-q")
        self.git("add", ".")
        self.git("-c", "user.name=scope", "-c", "user.email=scope@invalid", "-c", "commit.gpgsign=false", "commit", "-qm", "baseline")
        self.base = self.git("rev-parse", "HEAD").strip()
        self.adr = "docs/architecture/decisions/scope-test.json"
        self.contract = {"schema": "lay.field-integration-scope.v1", "base": self.base,
                         "components": ["kde-chrome"], "shared_paths": [], "decision": self.adr}
        self.write(self.adr, json.dumps({"schema": "lay.architecture-decision.v1", "protected_paths": []}))
        self.save_contract()

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, stderr=subprocess.DEVNULL).decode()

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def save_contract(self):
        self.write(scope.CONTRACT, json.dumps(self.contract))

    def receipt(self):
        return scope.build_receipt(self.root, self.base, dev.changed_paths)

    def request(self):
        receipt = self.receipt()
        files = [{"path": p, "sha256": scope.sha((self.root / p).read_bytes()),
                  "mode": (self.root / p).stat().st_mode & 0o777} for p in dev.file_list(self.root)]
        return {"source_head": self.base, "plan": {"changed": dev.changed_paths(self.root)},
                "files": files, "field_integration_scope": receipt}

    def test_chrome_edit_is_allowed_but_gnome_and_common_are_rejected(self):
        self.write(self.chrome, "// changed chrome\n")
        self.assertIn(self.chrome, [r["path"] for r in self.receipt()["changes"]])
        for path in (self.gnome, self.common):
            with self.subTest(path=path):
                old = (self.root / path).read_text()
                self.write(path, "// accidental edit\n")
                with self.assertRaisesRegex(ValueError, "outside declared"):
                    self.receipt()
                self.write(path, old)

    def test_untracked_unknown_runtime_and_deleted_common_file_are_rejected(self):
        for unknown in ("src/new_delivery.rs", "src/new_delivery.md"):
            self.write(unknown, "// unknown\n")
            with self.assertRaisesRegex(ValueError, "new_delivery"):
                self.receipt()
            (self.root / unknown).unlink()
        (self.root / self.common).unlink()
        with self.assertRaisesRegex(ValueError, "engine.rs"):
            self.receipt()

    def test_rename_does_not_hide_deletion_or_unknown_destination(self):
        destination = self.root / "src/hidden.rs"
        (self.root / self.chrome).rename(destination)
        with self.assertRaisesRegex(ValueError, "hidden.rs"):
            self.receipt()

    def test_own_versions_pass_but_dependency_and_tray_code_do_not(self):
        for path in ("Cargo.toml", "Cargo.lock", self.tray):
            with self.subTest(path=path):
                old = (self.root / path).read_text()
                self.write(path, old.replace("1.0.110", "1.0.111"))
                self.receipt()
                self.write(path, old.replace("true", "false") if path == self.tray
                           else old.replace('"1"', '"2"'))
                with self.assertRaisesRegex(ValueError, "outside declared"):
                    self.receipt()
                self.write(path, old)

    def test_metadata_version_cannot_hide_other_extension_changes(self):
        path = "extension/test/metadata.json"
        self.assertTrue(scope.version_only(path, b'{"version":1,"name":"Lay"}', b'{"version":2,"name":"Lay"}'))
        self.assertFalse(scope.version_only(path, b'{"version":1,"name":"Lay"}', b'{"version":2,"name":"Other"}'))

    def test_executable_bit_is_not_ordinary_client_or_version_metadata(self):
        (self.root / self.chrome).chmod(0o755)
        with self.assertRaisesRegex(ValueError, "outside declared"):
            self.receipt()

    def test_shared_paths_need_exact_owning_decision_coverage(self):
        self.write(self.common, "// explicitly scoped common seam\n")
        self.contract["shared_paths"] = [self.common]
        self.save_contract()
        with self.assertRaisesRegex(ValueError, "not covered"):
            self.receipt()
        self.write(self.adr, json.dumps({"schema": "lay.architecture-decision.v1", "protected_paths": [self.common]}))
        self.receipt()
        self.contract["shared_paths"] = ["src/*"]
        self.save_contract()
        with self.assertRaisesRegex(ValueError, "globs"):
            self.receipt()

    def test_missing_contract_and_unknown_base_fail_closed(self):
        (self.root / scope.CONTRACT).unlink()
        with self.assertRaises(FileNotFoundError):
            self.receipt()
        self.contract["base"] = "0" * 40
        self.save_contract()
        with self.assertRaises(subprocess.CalledProcessError):
            self.receipt()

    def test_frozen_receipt_rejects_missing_forged_and_changed_file_claims(self):
        self.write(self.chrome, "// changed\n")
        req = self.request()
        scope.verify_snapshot_receipt(req, self.root)
        for mutation in ("missing", "source", "base", "contract", "file", "omitted"):
            with self.subTest(mutation=mutation):
                bad = copy.deepcopy(req)
                if mutation == "missing":
                    del bad["field_integration_scope"]
                elif mutation == "source":
                    bad["field_integration_scope"]["source_head"] = "0" * 40
                elif mutation == "base":
                    bad["field_integration_scope"]["base"] = "0" * 40
                elif mutation == "contract":
                    bad["field_integration_scope"]["contract_sha256"] = "0" * 64
                elif mutation == "file":
                    next(r for r in bad["field_integration_scope"]["changes"] if r["path"] == self.chrome)["sha256"] = "0" * 64
                else:
                    bad["field_integration_scope"]["changes"] = [r for r in bad["field_integration_scope"]["changes"] if r["path"] != self.chrome]
                with self.assertRaises(ValueError):
                    scope.verify_snapshot_receipt(bad, self.root)

    def test_snapshot_entrypoint_enforces_scope_before_remote_transport(self):
        self.write(self.gnome, "// accidental gnome change\n")
        selected = {"source_head": self.base, "changed": dev.changed_paths(self.root)}
        archive = self.root.parent / (self.root.name + "-rejected.tar")
        self.addCleanup(archive.unlink, missing_ok=True)
        with self.assertRaisesRegex(ValueError, "outside declared"):
            dev.snapshot(self.root, archive, selected)

    def test_snapshot_seals_the_receipt_into_the_existing_request(self):
        self.write(self.chrome, "// changed\n")
        selected = {"source_head": self.base, "changed": dev.changed_paths(self.root)}
        archive = self.root.parent / (self.root.name + "-scope.tar")
        self.addCleanup(archive.unlink, missing_ok=True)
        req = dev.snapshot(self.root, archive, selected)
        scope.verify_snapshot_receipt(req, self.root)


if __name__ == "__main__":
    unittest.main()
