"""Expose the existing offline diagnostic suite to canonical Python discovery."""
import importlib.util
from pathlib import Path

_SOURCE = Path(__file__).resolve().parents[1] / "scripts/tests/test_native_diagnostics.py"
_SPEC = importlib.util.spec_from_file_location("lay_native_diagnostic_tests", _SOURCE)
_MODULE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MODULE)


def load_tests(loader, _tests, _pattern):
    suite = loader.loadTestsFromModule(_MODULE)
    if suite.countTestCases() == 0:
        raise ValueError("native diagnostic suite is empty")
    return suite
