#!/usr/bin/env python3
"""Check the complete working diff against the predeclared field scope."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys
import subprocess

from field_integration_scope import build_receipt, load_contract

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fetch-base", action="store_true", help="explicit CI-only shallow baseline fetch")
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location("scope_dev_check", ROOT / "scripts/dev-check.py")
    dev = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(dev)
    try:
        if args.fetch_base:
            base = load_contract(ROOT)["base"]
            exists = subprocess.run(["git", "cat-file", "-e", base + "^{commit}"], cwd=ROOT,
                                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0
            if not exists:
                subprocess.run(["git", "fetch", "--no-tags", "--depth=1", "origin", base], cwd=ROOT, check=True)
        receipt = build_receipt(ROOT, dev.git("rev-parse", "HEAD").strip(), dev.changed_paths)
        if receipt is None:
            raise ValueError("this checkout has no field scope declaration")
        print(json.dumps({"status": "SOURCE_SCOPE_VERIFIED_NOT_NATIVE_PROOF", **receipt}, sort_keys=True))
        return 0
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print("field integration scope: " + str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
