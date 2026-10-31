"""Selection/execution contracts for the small component entrypoints."""
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
SPEC = importlib.util.spec_from_file_location("component_checks", ROOT / "scripts/check-lay-component.py")
component = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(component)


class ComponentChecks(unittest.TestCase):
    def test_kde_focus_covers_the_actual_implementation_in_both_binaries(self):
        manifest = json.loads(component.MANIFEST.read_text())
        families = ((component.DAEMON, "kde_focus::tests::"),
                    (component.ENGINE, "kde_focus::tests::"),
                    (component.DAEMON, "layout_controller::kde_exact::tests::"),
                    (component.DAEMON, "focus_guard::"),
                    (component.DAEMON, "tests::layout_backend::"))
        expected = set()
        for target, prefix in families:
            rows = {(r["target"], r["name"]) for r in manifest["tests"]
                    if r["target"] == target and r["name"].startswith(prefix)
                    and r["lane"] in {"correctness", "package"}}
            self.assertTrue(rows, (target, prefix))
            expected.update(rows)
        actual = {(r["target"], r["name"])
                  for r in component.select_rows(manifest, "kde-focus")}
        self.assertEqual(expected, actual)

    def test_kde_focus_refuses_a_missing_implementation_family(self):
        manifest = json.loads(component.MANIFEST.read_text())
        for target, prefix in ((component.DAEMON, "kde_focus::tests::"),
                               (component.ENGINE, "kde_focus::tests::"),
                               (component.DAEMON, "layout_controller::kde_exact::tests::")):
            with self.subTest(target=target, prefix=prefix):
                incomplete = {"tests": [r for r in manifest["tests"]
                    if not (r["target"] == target and r["name"].startswith(prefix))]}
                with self.assertRaisesRegex(ValueError, "empty component selector"):
                    component.select_rows(incomplete, "kde-focus")

    def test_each_recipe_selects_nonzero_canonical_test_identities(self):
        manifest = json.loads(component.MANIFEST.read_text())
        for name in component.RECIPES:
            with self.subTest(component=name):
                rows = component.select_rows(manifest, name)
                self.assertGreater(len(rows), 0)
                self.assertEqual(len(rows), len({(r["target"], r["name"]) for r in rows}))
                self.assertTrue(all(r["lane"] in {"correctness", "package"} for r in rows))

    def test_empty_or_misspelled_selection_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "empty component selector"):
            component.select_rows({"tests": []}, "kde-chrome")
        with self.assertRaisesRegex(ValueError, "unknown component"):
            component.select_rows({"tests": []}, "kde-chorme")

    def test_real_failure_is_reported_and_manifest_drift_never_runs_tests(self):
        row = {"target": component.DAEMON, "name": "layout_controller::ime_bridge::contract",
               "lane": "correctness", "isolation": "target", "kind": "test"}
        manifest = {"tests": [row]}
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for drift in (False, True):
                with self.subTest(drift=drift), mock.patch.dict(os.environ, {"LAY_RESOURCE_GUARD_ACTIVE": "1"}), \
                     mock.patch.object(component, "source_closure_identity", return_value={"sha256": "frozen"}), \
                     mock.patch.object(component, "file_sha256", return_value="manifest"), \
                     mock.patch.object(component, "load_json", return_value=manifest), \
                     mock.patch.object(component, "cargo_discovery", return_value=({"tests": []} if drift else manifest, [{"target": component.DAEMON, "executable": "owned-harness"}])), \
                     mock.patch.object(component, "run_target", return_value=([{"target": component.DAEMON, "test": row["name"]}], 0.1)) as execute:
                    if drift:
                        with self.assertRaisesRegex(ValueError, "differs from canonical"):
                            component.run("shared-replay", root / "target", root / "drift")
                        execute.assert_not_called()
                    else:
                        result = component.run("shared-replay", root / "target", root / "failure")
                        self.assertEqual(("FAIL", 1, 0, 1), (result["verdict"], result["executed"], result["passed"], result["failed"]))
                        self.assertEqual([row], execute.call_args.args[1])


if __name__ == "__main__":
    unittest.main()
