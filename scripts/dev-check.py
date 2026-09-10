#!/usr/bin/env python3
"""One remote-only development check; existing release gates are unchanged."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tarfile
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
CONFIG = Path.home() / ".config/lay/development.json"
SAFE_REMOTE = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_.@-]*\Z")
SAFE_PATH = re.compile(r"/[A-Za-z0-9_./-]+\Z")
SNAPSHOT_EXCLUDES = ("graphify-out/memory/", "graphify-out/reflections/")
SELF_TESTS = ["tests.test_dev_check", "tests.test_focused_lanes",
              "tests.test_ime_client_harness", "tests.test_test_lanes",
              "tests.test_resource_guard"]


def git(*args: str, root: Path = ROOT) -> str:
    return subprocess.check_output(["git", *args], cwd=root).decode()


def changed_paths(root: Path, base: str | None = None) -> list[str]:
    # --no-renames preserves both deletion and addition; NUL is filename-safe.
    paths = set(git("diff", "--name-only", "--no-renames", "-z", "HEAD", "--",
                    root=root).split("\0"))
    if base:
        commit = git("rev-parse", "--verify", "--end-of-options", f"{base}^{{commit}}", root=root).strip()
        paths.update(git("diff", "--name-only", "--no-renames", "-z", commit,
                         "HEAD", "--", root=root).split("\0"))
    paths.update(git("ls-files", "--others", "--exclude-standard", "-z",
                     root=root).split("\0"))
    return sorted(paths - {""})


def targets_in(root: Path) -> list[str]:
    cargo = (root / "Cargo.toml").read_text()
    if re.search(r"^\s*(?:\[\[test\]\]|autotests\s*=)", cargo, re.MULTILINE):
        raise ValueError("custom Cargo integration layout requires broad verification")
    targets = {f"test:{path.stem}" for path in (root / "tests").glob("*.rs")}
    targets.update(f"test:{path.parent.name}" for path in (root / "tests").glob("*/main.rs"))
    return sorted(targets)


def plan(paths: list[str], root: Path = ROOT) -> dict:
    common = {"scope": "development_only_not_release", "changed": paths,
              "runtime_authority_changed": False}
    if not paths:
        return {**common, "mode": "none", "targets": [], "reason": "no changes"}
    ime = "src/bin/lay_ibus_engine"
    if all(path == ime + ".rs" or path.startswith(ime + "/") for path in paths):
        # Other binary/library imports would invalidate the private-owner rule.
        for path in (root / "src").rglob("*.rs"):
            relative = path.relative_to(root).as_posix()
            if relative == ime + ".rs" or relative.startswith(ime + "/"):
                continue
            source = path.read_text()
            if "lay_ibus_engine" in source or re.search(r"include!\s*\(\s*concat!", source):
                return {**common, "mode": "all", "targets": [],
                        "reason": f"uncertain cross-owner source dependency: {relative}"}
        try:
            integration = targets_in(root)
        except ValueError as error:
            return {**common, "mode": "all", "targets": [], "reason": str(error)}
        return {**common, "mode": "focused",
                "targets": ["bin:lay-ibus-engine", *integration],
                "reason": "private IME owner plus all integration contracts"}
    return {**common, "mode": "all", "targets": [],
            "reason": "shared, tooling, deleted/unknown or mixed change; broad verification"}


def file_list(root: Path) -> list[str]:
    names = set(git("ls-files", "-co", "--exclude-standard", "-z", root=root).split("\0"))
    return sorted(name for name in names if name and os.path.lexists(root / name)
                  and not name.startswith(SNAPSHOT_EXCLUDES)
                  and name not in {"graphify-out/.graphify_python", "graphify-out/.vocab.txt",
                                   "graphify-out/.graphify_learning.json"})


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def write_json(path: Path, payload: dict) -> None:
    with path.open("x") as stream:
        json.dump(payload, stream, indent=2, sort_keys=True)
        stream.write("\n")


def snapshot(root: Path, archive: Path, selected: dict) -> dict:
    def assert_plan_current():
        if (git("rev-parse", "HEAD", root=root).strip() != selected["source_head"]
                or changed_paths(root, selected.get("base")) != selected["changed"]):
            raise ValueError("planning inputs changed before/during snapshot; replan before executing")

    assert_plan_current()
    names = file_list(root)
    rows = []
    with tarfile.open(archive, "w") as bundle:
        for name in names:
            path = root / name
            if path.is_symlink() or not path.is_file() or ".git" in Path(name).parts:
                raise ValueError(f"unsupported snapshot member: {name}")
            data = path.read_bytes()
            info = tarfile.TarInfo(name)
            metadata = path.stat()
            info.size, info.mode = len(data), metadata.st_mode & 0o777
            info.mtime = metadata.st_mtime
            info.pax_headers["mtime"] = f"{metadata.st_mtime_ns // 1000000000}.{metadata.st_mtime_ns % 1000000000:09d}"
            bundle.addfile(info, io.BytesIO(data))
            rows.append({"path": name, "sha256": hashlib.sha256(data).hexdigest(),
                         "mode": info.mode})
        # Reject concurrent edits rather than attribute a mixed snapshot to HEAD.
        if names != file_list(root) or any(digest(root / row["path"]) != row["sha256"]
                                          or (root / row["path"]).stat().st_mode & 0o777 != row["mode"]
                                          for row in rows):
            raise ValueError("source changed during snapshot; do not execute this archive")
        assert_plan_current()
    return {"schema": "lay.development-snapshot.v1", "source_head": selected["source_head"],
            "archive_sha256": digest(archive), "files": rows, "plan": selected}


def load_config(path: Path) -> dict:
    config = json.loads(path.read_text())
    remote = config["remote"]
    if not isinstance(remote, str) or not SAFE_REMOTE.fullmatch(remote):
        raise ValueError("remote must be a plain SSH host or user@host, never options")
    for key in ("runs_dir", "target_dir"):
        value = config[key]
        if (not isinstance(value, str) or not SAFE_PATH.fullmatch(value)
                or ".." in Path(value).parts or len(Path(value).parts) < 4):
            raise ValueError(f"{key} must be an explicit scoped absolute path")
        if any(mask in value + "/" for mask in
               ("/.cache/lay/", "/.config/lay/", "/.local/share/lay/")):
            raise ValueError(f"{key} is masked by the existing test sandbox; use a projects directory")
    return config


def ssh(remote: str, argv: list[str], **kwargs):
    return subprocess.run(["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
                           "--", remote, shlex.join(argv)], check=True, **kwargs)


def assert_remote(origin: str) -> None:
    if not origin or Path("/etc/machine-id").read_text().strip() == origin:
        raise ValueError("refusing local build/test execution: distinct remote machine required")


def run_remote(args, selected: dict) -> int:
    config = load_config(args.config)
    origin = Path("/etc/machine-id").read_text().strip()
    remote_id = ssh(config["remote"], ["cat", "/etc/machine-id"],
                    capture_output=True, text=True).stdout.strip()
    if not remote_id or remote_id == origin:
        raise ValueError("SSH destination is the local machine; no tests started")
    cache = Path.home() / ".cache/lay/development"
    cache.mkdir(parents=True, exist_ok=True)
    result = Path(tempfile.mkdtemp(prefix="run-", dir=cache))
    started = time.monotonic()
    archive = result / "source.tar"
    provenance = snapshot(ROOT, archive, selected)
    provenance.update(action=args.action, origin_machine_id=origin,
                      remote_machine_id=remote_id, target_dir=config["target_dir"],
                      client_config=args.client_config,
                      workspace_dir=config["runs_dir"] + "/workspace")
    write_json(result / "request.json", provenance)
    ssh(config["remote"], ["mkdir", "-p", config["runs_dir"]])
    remote_run = ssh(config["remote"], ["mktemp", "-d", config["runs_dir"] + "/run-XXXXXX"],
                     capture_output=True, text=True).stdout.strip()
    if not remote_run.startswith(config["runs_dir"] + "/run-") or not SAFE_PATH.fullmatch(remote_run):
        raise ValueError("unexpected remote staging path")
    subprocess.run(["scp", "-q", "--", str(archive), str(result / "request.json"),
                    f"{config['remote']}:{remote_run}/"], check=True)
    bootstrap = """import hashlib,json,pathlib,tarfile
p=pathlib.Path(__import__('sys').argv[1]); req=json.loads((p/'request.json').read_text())
h=hashlib.sha256()
with (p/'source.tar').open('rb') as f:
 for chunk in iter(lambda:f.read(1048576),b''): h.update(chunk)
assert h.hexdigest()==req['archive_sha256'],'snapshot hash mismatch'
with tarfile.open(p/'source.tar') as t:
 names=t.getnames()
 assert len(names)==len(set(names)),'duplicate snapshot members'
 assert all(m.isfile() and not pathlib.PurePosixPath(m.name).is_absolute() and '..' not in pathlib.PurePosixPath(m.name).parts and '.git' not in pathlib.PurePosixPath(m.name).parts for m in t),'unsafe archive member'
 t.extractall(p/'source')
"""
    ssh(config["remote"], ["python3", "-c", bootstrap, remote_run])
    command = ["env", "LAY_RESOURCE_PROFILE=dedicated-20cpu", "CARGO_BUILD_JOBS=20",
               "RUST_TEST_THREADS=1", "CARGO_TARGET_DIR=" + config["target_dir"],
               remote_run + "/source/scripts/lay-resource-guard.sh", "--", "python3",
               remote_run + "/source/scripts/dev-check.py", "_worker", "--request",
               remote_run + "/request.json"]
    with (result / "run.log").open("x") as log:
        completed = subprocess.run(["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
                                    "--", config["remote"], shlex.join(command)],
                                   stdout=log, stderr=subprocess.STDOUT)
    # Artifacts stay remote; fetch only the compact authoritative completion.
    fetched = subprocess.run(["scp", "-q", "--", f"{config['remote']}:{remote_run}/RESULT.json",
                              str(result / "RESULT.json")], capture_output=True)
    write_json(result / "transport.json", {"remote_run": remote_run,
               "remote": config["remote"], "returncode": completed.returncode,
               "result_fetched": fetched.returncode == 0,
               "elapsed_seconds": time.monotonic() - started})
    print(f"development_check rc={completed.returncode} local={result} remote={remote_run}", flush=True)
    if fetched.returncode == 0:
        receipt = json.loads((result / "RESULT.json").read_text())
        print(json.dumps({key: receipt.get(key) for key in
                          ("verdict", "scope", "elapsed_seconds", "commands", "error")}, indent=2))
    return completed.returncode or (0 if fetched.returncode == 0 else 1)


def prepare_workspace(req: dict, request_path: Path) -> None:
    """Reuse one Cargo source path, exclusively inside the existing heavy lease."""
    workspace = Path(req["workspace_dir"])
    expected = request_path.parent.parent / "workspace"
    if workspace != expected or workspace.is_symlink():
        raise ValueError("workspace must be the runner-owned sibling of this run")
    marker = workspace.parent / ".workspace-owner"
    owner = "lay-development-snapshot-v1\n"
    if not marker.exists():
        if workspace.exists():
            raise ValueError("refusing an existing unowned workspace")
        with marker.open("x") as stream:
            stream.write(owner)
    if marker.is_symlink() or marker.read_text() != owner:
        raise ValueError("invalid workspace ownership marker")
    if ROOT == workspace:
        return
    workspace.mkdir(exist_ok=True)
    # This is a disposable owned mirror, NOT a user checkout. Prior source
    # archives stay in sibling run directories for recovery and attribution.
    subprocess.run(["rsync", "-a", "--checksum", "--delete", "--exclude=.git", "--",
                    str(ROOT) + "/", str(workspace) + "/"], check=True)
    os.execv(sys.executable, [sys.executable, str(workspace / "scripts/dev-check.py"),
                             "_worker", "--request", str(request_path)])


def worker(request_path: Path) -> int:
    req = json.loads(request_path.read_text())
    assert_remote(req["origin_machine_id"])
    if any(os.environ.get(key) != value for key, value in {
        "LAY_RESOURCE_GUARD_ACTIVE": "1", "LAY_RESOURCE_LEASE_HELD": "1",
        "LAY_RESOURCE_PROFILE": "dedicated-20cpu", "CARGO_BUILD_JOBS": "20",
        "RUST_TEST_THREADS": "1", "CARGO_TARGET_DIR": req["target_dir"],
    }.items()):
        raise ValueError("remote resource guard contract missing")
    run = request_path.parent
    if (run / "RESULT.json").exists():
        raise ValueError("run already completed; create a fresh run")
    for row in req["files"]:
        path = ROOT / row["path"]
        if digest(path) != row["sha256"] or path.stat().st_mode & 0o777 != row["mode"]:
            raise ValueError(f"remote snapshot drift: {row['path']}")
    prepare_workspace(req, request_path)
    with (run / "EXECUTION_STARTED").open("x") as stream:
        stream.write("single execution; a failed run is never retried in place\n")
    # Existing lane provenance needs a Git root. This empty metadata commit is
    # explicitly NOT the source commit; canonical source identity is in request.
    if not (ROOT / ".git").exists():
        subprocess.run(["git", "init", "-q", str(ROOT)], check=True)
        subprocess.run(["git", "-c", "user.name=Lay development snapshot", "-c",
                        "user.email=lay-snapshot@invalid", "-c", "commit.gpgsign=false",
                        "commit", "--allow-empty", "-qm", "Snapshot metadata; source identity in request.json"],
                       cwd=ROOT, check=True)
    selected = req["plan"]
    result = {"scope": "development_only_not_release", "source_head": req["source_head"],
              "archive_sha256": req["archive_sha256"], "plan": selected,
              "runtime_authority_changed": False, "commands": []}
    started = time.monotonic()
    if req["action"] == "self-test":
        commands = [["python3", "-m", "unittest", "-v", *SELF_TESTS]]
    elif req["action"] == "client":
        commands = [["python3", "scripts/proof/ime-client/run.py", "--remote-worker", "--config",
                     req["client_config"], "--output", str(run / "client")]]
    else:
        commands = [["scripts/cargo-guard.sh", "fmt", "--all", "--check"]]
        if selected["mode"] == "focused":
            command = ["scripts/check-lay-tests.sh", "focused"]
            for target in selected["targets"]:
                command += ["--target", target]
        else:
            commands.append(["scripts/check-lay-tests.sh", "self-test"])
            command = ["scripts/check-lay-tests.sh", "all"]
        commands.append([*command, "--target-dir", req["target_dir"],
                         "--results-dir", str(run / "tests")])
    status = 0
    try:
        for command in commands:
            tick = time.monotonic()
            print("development_command=" + shlex.join(command), flush=True)
            status = subprocess.run(command, cwd=ROOT).returncode
            result["commands"].append({"argv": command, "returncode": status,
                                       "elapsed_seconds": time.monotonic() - tick})
            if status:
                break
        for row in req["files"]:
            if digest(ROOT / row["path"]) != row["sha256"]:
                raise ValueError(f"source changed during execution: {row['path']}")
    except (OSError, ValueError) as error:
        result["error"] = str(error)
        status = 1
    result.update(returncode=status, verdict="PASS" if status == 0 else "FAIL",
                  elapsed_seconds=time.monotonic() - started)
    write_json(run / "RESULT.json", result)
    return status


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["plan", "check", "self-test", "client", "_worker"])
    parser.add_argument("--config", type=Path, default=CONFIG)
    parser.add_argument("--base", help="also include committed changes since this Git ref")
    parser.add_argument("--target", action="append", default=[], help="explicit development scope, not affected-closure proof")
    parser.add_argument("--client-config", help="absolute configuration path on the remote host")
    parser.add_argument("--request", type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    try:
        if args.action == "_worker":
            return worker(args.request)
        head = git("rev-parse", "HEAD").strip()
        selected = plan(changed_paths(ROOT, args.base))
        if head != git("rev-parse", "HEAD").strip():
            raise ValueError("HEAD changed during planning")
        selected.update(source_head=head, base=args.base)
        if args.target:
            if any(not re.fullmatch(r"(?:bin|lib|test):[A-Za-z0-9_-]+", item) for item in args.target):
                raise ValueError("invalid target identity")
            selected.update(mode="focused", targets=sorted(set(args.target)),
                            reason="explicit user-selected scope; not full affected-closure coverage")
        if args.action == "plan":
            print(json.dumps(selected, indent=2))
            return 0
        if args.action == "client" and (not args.client_config or not SAFE_PATH.fullmatch(args.client_config)):
            raise ValueError("client requires --client-config with an absolute remote path")
        if args.action == "check" and selected["mode"] == "none":
            print("NO_CHANGES: no tests executed; use --target or --base for an explicit scope")
            return 0
        return run_remote(args, selected)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"development_check=BLOCKED error={error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
