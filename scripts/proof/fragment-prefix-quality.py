#!/usr/bin/env python3
"""Fixed streaming comparison using the actual compiled letter-fragment decision.

Remote/resource guarded only. Reuses the Nov2 proof partition and TSV protocol;
never changes models, live configuration, learning or native-client state.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time


def sha(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_rows(path: Path) -> dict:
    rows = {}
    for line in path.read_text().splitlines():
        language, condition, expected, flips, first, actual, layout = line.split("\t")
        key = (language, condition, expected)
        if key in rows:
            raise ValueError("duplicate proof identity")
        rows[key] = dict(flips=int(flips), first=int(first), actual=actual,
                         layout_ru=layout == "true")
    return rows


def transition_regressions(previous: dict, current: dict, expected: str) -> set[str]:
    regressions = set()
    was_wrong = previous["flips"] > 0 and previous["actual"] != expected
    is_wrong = current["flips"] > 0 and current["actual"] != expected
    if is_wrong and not was_wrong:
        regressions.add("new_wrong_after_flip")
    if current["flips"] > max(1, previous["flips"]):
        regressions.add("new_repeated_flips")
    return regressions


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--compare", type=Path)
    parser.add_argument("--origin-machine-id", required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    if Path("/etc/machine-id").read_text().strip() == args.origin_machine_id:
        raise ValueError("proof execution must be remote")
    if (os.environ.get("LAY_RESOURCE_GUARD_ACTIVE") != "1"
            or os.environ.get("LAY_RESOURCE_PROFILE") != "dedicated-20cpu"):
        raise ValueError("required remote resource guard is absent")
    run = args.output.resolve()
    run.mkdir(parents=True, exist_ok=False)
    sys.path.insert(0, str(root / "scripts"))
    from test_lanes.cli import source_closure_identity
    before = source_closure_identity()
    started = time.monotonic()
    result = dict(status="RUNNING", scope="compiled streaming proof, not native acceptance",
                  source=before, model_sha256=sha(root / "data/layout_fragments/model.bin"))
    try:
        # Restored snapshots can predate cached build timestamps. Recompile only
        # Lay, retaining dependency artifacts; a content hash alone cannot prove
        # that Cargo incorporated the changed decision into an executable.
        with (run / "clean.log").open("w") as log:
            subprocess.run([str(root / "scripts/cargo-guard.sh"), "clean",
                            "--package", "lay", "--release"], cwd=root,
                           stdout=log, stderr=subprocess.STDOUT, check=True)
        command = [str(root / "scripts/cargo-guard.sh"), "build", "--release", "--locked",
                   "--features", "research-tools", "--example", "fragment_prefix_proof",
                   "--message-format=json"]
        with (run / "compiler.jsonl").open("w") as out, (run / "build.log").open("w") as err:
            subprocess.run(command, cwd=root, stdout=out, stderr=err, check=True)
        artifacts = [json.loads(line) for line in (run / "compiler.jsonl").read_text().splitlines()]
        library = [row for row in artifacts if row.get("reason") == "compiler-artifact"
                   and row["target"]["name"] == "lay" and "lib" in row["target"]["kind"]]
        if len(library) != 1 or library[0].get("fresh") is not False:
            raise ValueError("proof requires newly compiled Lay source")
        executables = [Path(row["executable"]) for row in artifacts
                       if row.get("reason") == "compiler-artifact" and row.get("executable")
                       and row["target"]["name"] == "fragment_prefix_proof"]
        if len(executables) != 1:
            raise ValueError("missing or ambiguous compiler artifact")
        probe = run / "probe"
        shutil.copy2(executables[0], probe)
        result["artifact"] = dict(compiler_executable=str(executables[0]), sha256=sha(probe))

        spec = importlib.util.spec_from_file_location("fragment_partition", root / "scripts/train-layout-fragments.py")
        trainer = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(trainer)
        _, heldout = trainer.load_partition(root, 120000, 10000)
        if list(map(len, heldout)) != [10000, 10000]:
            raise ValueError("fixed heldout cardinality changed")
        input_path = run / "input.tsv"
        with input_path.open("w") as stream:
            for language, words in zip(["ru", "en"], heldout):
                for condition in ["clean", "wrong"]:
                    for word in words:
                        stream.write(f"{language}\t{condition}\t{word}\n")
        result["input_sha256"] = sha(input_path)
        isolated = run / "isolated"
        isolated.mkdir()
        env = {k: v for k, v in os.environ.items() if not k.startswith("LAY_")}
        env.update(HOME=str(isolated), XDG_CONFIG_HOME=str(isolated / "config"),
                   XDG_DATA_HOME=str(isolated / "data"), XDG_CACHE_HOME=str(isolated / "cache"),
                   LAY_CONFIG_PATH=str(isolated / "config/lay.json"))
        with input_path.open() as data, (run / "rows.tsv").open("w") as out, (run / "timing.log").open("w") as err:
            subprocess.run([str(probe)], cwd=run, env=env, stdin=data, stdout=out, stderr=err, check=True)
        rows = read_rows(run / "rows.tsv")
        if len(rows) != 40000:
            raise ValueError("incomplete proof")
        groups, first_at, examples = defaultdict(Counter), defaultdict(Counter), defaultdict(list)
        for (language, condition, expected), row in rows.items():
            key = f"{language}/{condition}"
            counts = groups[key]
            counts.update(total=1, exact=int(row["actual"] == expected),
                          any_flip=int(row["flips"] > 0), multiple_flips=int(row["flips"] > 1),
                          no_flip=int(row["flips"] == 0),
                          wrong_after_flip=int(row["flips"] > 0 and row["actual"] != expected))
            first_at[key][row["first"]] += 1
            if row["flips"] and row["actual"] != expected and len(examples[key]) < 8:
                examples[key].append(dict(expected=expected, **row))
        result.update(groups=dict(groups), first_trigger_letter=dict(first_at), examples=dict(examples))
        violations = []
        if args.compare:
            baseline = json.loads((args.compare / "RESULT.json").read_text())
            if baseline.get("status") != "BASELINE_RECORDED":
                raise ValueError("comparison requires a completed baseline")
            if any(baseline[k] != result[k] for k in ["input_sha256", "model_sha256"]):
                raise ValueError("baseline input or model mismatch")
            old = read_rows(args.compare / "rows.tsv")
            if rows.keys() != old.keys():
                raise ValueError("baseline identity mismatch")
            delta = defaultdict(Counter)
            for (language, condition, expected), row in rows.items():
                previous = old[(language, condition, expected)]
                key = f"{language}/{condition}"
                if condition == "wrong":
                    delta[key]["newly_restored"] += previous["actual"] != expected and row["actual"] == expected
                    delta[key]["lost_restoration"] += previous["actual"] == expected and row["actual"] != expected
                    delta[key]["earlier"] += bool(row["first"] and previous["first"] and row["first"] < previous["first"])
                delta[key].update(transition_regressions(previous, row, expected))
            for language in ["ru", "en"]:
                clean, wrong = f"{language}/clean", f"{language}/wrong"
                if groups[clean]["any_flip"] > baseline["groups"][clean]["any_flip"]:
                    violations.append(clean + ": increased clean flips")
                if delta[wrong]["lost_restoration"]:
                    violations.append(wrong + ": lost baseline restoration")
                for key in [clean, wrong]:
                    for metric in ["new_wrong_after_flip", "new_repeated_flips"]:
                        if delta[key][metric]:
                            violations.append(key + ": " + metric)
                    for metric in ["wrong_after_flip", "multiple_flips"]:
                        if groups[key][metric] > baseline["groups"][key][metric]:
                            violations.append(key + ": increased " + metric)
            result.update(comparison=str(args.compare), delta=dict(delta), violations=violations)
        if source_closure_identity() != before:
            raise ValueError("source changed during proof")
        result["status"] = "FAIL_QUALITY" if violations else ("PASS" if args.compare else "BASELINE_RECORDED")
    except Exception as error:
        result.update(status="ERROR", error=str(error))
    finally:
        result["elapsed_s"] = round(time.monotonic() - started, 3)
        (run / "RESULT.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps({k: result.get(k) for k in ["status", "groups", "delta", "violations", "error", "elapsed_s"]}), flush=True)
    return 0 if result["status"] in ["PASS", "BASELINE_RECORDED"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
