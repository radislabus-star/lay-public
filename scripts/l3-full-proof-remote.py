#!/usr/bin/env python3
"""Execute one content-bound L3 differential on a configured remote worker.

No model publication, input handling, local proof fallback or result cache.
SSH endpoints and worker installation identities live only in private config.
"""
from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import signal
import subprocess
import sys
import tarfile
import tempfile
import time

CONFIG = Path.home() / ".config/lay/l3-proof-worker.json"
SAFE_REMOTE = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_.@-]*\Z")
SAFE_PATH = re.compile(r"/[A-Za-z0-9_./-]+\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")
MUTABLE_ROLES = {"usage_events", "usage_counts", "usage_feedback", "usage_legacy"}
ENV_ROLES = {
    "lexical": "LAY_L2_LEXICAL_PHASE_MEMORY", "canonical": "LAY_L2_PACKAGE",
    "usage_events": "LAY_NANDA_WORD_USAGE_EVENTS",
    "usage_counts": "LAY_NANDA_WORD_USAGE_COUNTS",
    "usage_feedback": "LAY_NANDA_WORD_USAGE_FEEDBACK_COUNTS",
    "usage_legacy": "LAY_NANDA_USAGE_PRIOR",
}


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")
    path.chmod(0o600)


def load_config(path: Path) -> dict:
    cfg = json.loads(path.read_text())
    if not isinstance(cfg.get("remote"), str) or not SAFE_REMOTE.fullmatch(cfg["remote"]):
        raise ValueError("worker must be a plain SSH host, never command options")
    for key in ("runs_dir", "binary", "resource_guard"):
        p = cfg.get(key)
        if not isinstance(p, str) or not SAFE_PATH.fullmatch(p) or ".." in Path(p).parts or len(Path(p).parts) < 4:
            raise ValueError("worker paths must be explicit scoped absolute paths")
    for key in ("binary_sha256", "guard_sha256"):
        if not isinstance(cfg.get(key), str) or not SHA.fullmatch(cfg[key]):
            raise ValueError("worker binary and guard identities must be pinned")
    if not re.fullmatch(r"[0-9a-f]{32}", cfg.get("machine_id", "")):
        raise ValueError("worker machine identity must be pinned")
    if cfg["machine_id"] == Path("/etc/machine-id").read_text().strip():
        raise ValueError("refusing local full-proof execution")
    return cfg


def ssh(cfg: dict, argv: list[str], **kwargs):
    return subprocess.run(["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
                           "-o", "ServerAliveInterval=15", "-o", "ServerAliveCountMax=2",
                           "--", cfg["remote"], shlex.join(argv)], **kwargs)


def proof_dependencies() -> dict[str, Path]:
    home = Path.home()
    wave = home / ".local/share/lay/nanda_wave"
    data = Path(os.environ.get("XDG_DATA_HOME", home / ".local/share"))
    defaults = {
        "lexical": data / "lay/nanda_wave/l2_lexical_phase_v2.bin",
        "canonical": Path(os.environ.get("LAY_L2_MODEL_DIR", wave / "l2")) / "LAY-L2-RU-FULL-v13.bin",
        "usage_events": wave / "word_usage_events.jsonl",
        "usage_counts": wave / "word_usage_counts.json",
        "usage_feedback": wave / "word_usage_feedback_counts.json",
        "usage_legacy": home / ".local/share/lay/learning_candidates.json",
    }
    return {role: Path(os.environ.get(ENV_ROLES[role], path)).absolute()
            for role, path in defaults.items()}


def copy_input(source: Path, destination: Path, *, optional: bool = False) -> dict:
    if not source.exists():
        if not optional:
            raise ValueError("required proof input is missing")
        return {"path": str(destination.name), "source": str(source), "present": False}
    before = source.stat()
    if not source.is_file() or before.st_size > 512 * 1024 * 1024:
        raise ValueError("proof input must be a bounded regular file")
    with source.open("rb") as incoming, destination.open("xb") as outgoing:
        shutil.copyfileobj(incoming, outgoing, 1024 * 1024)
    after = source.stat()
    stamp = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
    if stamp(before) != stamp(after):
        raise ValueError("proof input changed during snapshot")
    destination.chmod(0o400)
    return {"path": destination.name, "source": str(source), "present": True,
            "bytes": after.st_size, "sha256": digest(destination)}


def prepare_inputs(args, root: Path, cfg: dict) -> dict:
    if "LAY_L3_REAL_L2_PROBE" in os.environ:
        raise ValueError("canonical real-L2 proof dependencies are not supported by this runner")
    inputs = root / "inputs"
    inputs.mkdir(mode=0o700)
    manifest_bytes = args.manifest.read_bytes()
    manifest = json.loads(manifest_bytes)
    if manifest.get("format") != "lay-l3-composite-v1":
        raise ValueError("unknown L3 manifest format")
    roles = {"corpus": args.corpus, "surface": args.surface_evidence, "delta": args.delta}
    roles.update(proof_dependencies())
    base = Path(manifest["base"])
    roles["base"] = base if base.is_absolute() else args.manifest.parent / base
    entries = manifest.get("deltas", [])
    for index, entry in enumerate(entries):
        path = Path(entry["path"])
        roles[f"admitted_{index}"] = path if path.is_absolute() else args.manifest.parent / path
    records = {role: copy_input(path, inputs / role, optional=role in MUTABLE_ROLES or role == "canonical")
               for role, path in roles.items()}
    frozen_manifest = {"format": manifest["format"], "base": "base", "deltas": []}
    for index, entry in enumerate(entries):
        name = f"admitted_{index}"
        if records[name]["bytes"] != entry["bytes"]:
            raise ValueError("admitted delta size mismatch")
        frozen_manifest["deltas"].append({"path": name, "bytes": entry["bytes"],
                                           "admitted_unix_ms": entry["admitted_unix_ms"]})
    write_json(inputs / "manifest.json", frozen_manifest)
    (inputs / "manifest.json").chmod(0o400)
    if args.manifest.read_bytes() != manifest_bytes:
        raise ValueError("manifest changed during snapshot")
    return {"schema": "lay.l3-remote-proof.v1", "roles": records,
            "manifest_sha256": digest(inputs / "manifest.json"),
            "origin_manifest": str(args.manifest), "origin_manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
            "config": cfg, "max_fragments": args.max_fragments,
            "min_surface_support": args.min_surface_support,
            "evaluator_sha256": args.evaluator_sha256}


def verify_inputs(root: Path, request: dict) -> None:
    inputs = root / "inputs"
    if digest(inputs / "manifest.json") != request["manifest_sha256"]:
        raise ValueError("snapshot manifest identity mismatch")
    for role, row in request["roles"].items():
        p = inputs / role
        if row["present"]:
            if not p.is_file() or p.stat().st_size != row["bytes"] or digest(p) != row["sha256"]:
                raise ValueError("frozen proof input identity mismatch")
        elif p.exists():
            raise ValueError("absent proof role became present")


def verify_origin(request: dict) -> None:
    if digest(Path(request["origin_manifest"])) != request["origin_manifest_sha256"]:
        raise ValueError("live baseline manifest changed during remote proof")
    # Feedback may evolve during the proof. Its frozen identities stay in the
    # receipt; immutable model/corpus inputs must still match the origin.
    for role, row in request["roles"].items():
        if role in MUTABLE_ROLES:
            continue
        p = Path(row["source"])
        if (row["present"] and (not p.is_file() or digest(p) != row["sha256"])) or (not row["present"] and p.exists()):
            raise ValueError("live immutable proof dependency changed")


def clear_inputs(root: Path) -> None:
    inputs = root / "inputs"
    if inputs.is_dir():
        inputs.chmod(0o700)
        shutil.rmtree(inputs)


def guarded_limits() -> dict:
    relative = next(line[3:] for line in Path("/proc/self/cgroup").read_text().splitlines()
                    if line.startswith("0::"))
    group = Path("/sys/fs/cgroup") / relative.lstrip("/")
    limits = {key:(group / key).read_text().strip() for key in
              ["cpu.max", "memory.high", "memory.max", "memory.swap.max", "pids.max"]}
    quota, period = limits["cpu.max"].split()
    if quota == "max" or int(quota) != int(period) * 20 or any(limits[key] != str(value) for key,value in {
        "memory.high":24*1024**3, "memory.max":28*1024**3,
        "memory.swap.max":1024**3, "pids.max":512}.items()):
        raise ValueError("actual worker envelope differs from dedicated-20cpu contract")
    lease = Path(f"/run/user/{os.getuid()}/lay-verification-heavy-{os.getuid()}.lock")
    # The existing outer guard must still own its shared host lease.
    with lease.open("a") as stream:
        try:
            fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            pass
        else:
            fcntl.flock(stream, fcntl.LOCK_UN)
            raise ValueError("remote heavy-execution lease is not held")
    return {"cgroup":relative, "limits":limits, "shared_lease_held":True}


def worker(root: Path) -> int:
    def interrupted(signum, _frame):
        raise SystemExit(128 + signum)
    for signum in (signal.SIGTERM, signal.SIGHUP, signal.SIGINT):
        signal.signal(signum, interrupted)
    request = json.loads((root / "request.json").read_text())
    cfg = request["config"]
    try:
        if Path("/etc/machine-id").read_text().strip() != cfg["machine_id"]:
            raise ValueError("worker identity mismatch")
        if any(os.environ.get(k) != v for k, v in {
            "LAY_RESOURCE_GUARD_ACTIVE": "1", "LAY_RESOURCE_LEASE_HELD": "1",
            "LAY_RESOURCE_PROFILE": "dedicated-20cpu"}.items()):
            raise ValueError("remote resource guard/lease missing")
        if digest(Path(cfg["binary"])) != request["evaluator_sha256"] or digest(Path(cfg["resource_guard"])) != cfg["guard_sha256"]:
            raise ValueError("worker evaluator/guard identity mismatch")
        envelope = guarded_limits()
        verify_inputs(root, request)
        inputs = root / "inputs"
        inputs.chmod(0o500)
        env = os.environ.copy()
        # A inherited worker setting must never switch to another proof route.
        env.pop("LAY_L3_REAL_L2_PROBE", None)
        env.update({ENV_ROLES[role]: str(inputs / role) for role in ENV_ROLES})
        env.update(LAY_L3_PROOF_WORKERS="20", MALLOC_ARENA_MAX="2")
        output = root / "receipt.json"
        argv = [cfg["binary"], "--prove-l3-context-composite-delta-full", str(inputs / "corpus"),
                "--manifest", str(inputs / "manifest.json"), "--delta", str(inputs / "delta"),
                "--surface-evidence", str(inputs / "surface"),
                "--max-fragments", str(request["max_fragments"]),
                "--min-surface-support", str(request["min_surface_support"]),
                "--out-receipt", str(output)]
        started = time.monotonic()
        with (root / "proof.log").open("wb") as log:
            result = subprocess.run(argv, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=240)
        verify_inputs(root, request)
        if digest(Path(cfg["binary"])) != request["evaluator_sha256"]:
            raise ValueError("worker evaluator changed during execution")
        proof = json.loads(output.read_text())
        if proof.get("kind") != "l3_context_phase_full_differential_proof" or not proof.get("compared_transitions"):
            raise ValueError("worker produced no complete differential proof")
        if proof.get("delta_sha256") != request["roles"]["delta"]["sha256"]:
            raise ValueError("worker proof belongs to another delta")
        if result.returncode != 0 and proof.get("verdict") != "WATCH":
            raise ValueError("evaluator failed before a complete proof")
        proof.update(executor_sha256=request["evaluator_sha256"],
                     execution="remote_dedicated_20cpu", execution_seconds=round(time.monotonic()-started, 3),
                     execution_envelope=envelope,
                     proof_input_identities={k: {f:v for f,v in row.items() if f not in {"source", "path"}}
                                             for k,row in request["roles"].items()})
        write_json(output, proof)
        return 0
    finally:
        # A dead SSH client must not leave private corpus/feedback on the worker.
        for signum in (signal.SIGTERM, signal.SIGHUP, signal.SIGINT):
            signal.signal(signum, signal.SIG_IGN)
        clear_inputs(root)


def execute(args) -> int:
    cfg = load_config(args.config)
    if cfg["binary_sha256"] != args.evaluator_sha256:
        raise ValueError("configured evaluator differs from requesting trainer")
    args.out_receipt.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    args.out_receipt.unlink(missing_ok=True)
    remote_root = None
    with tempfile.TemporaryDirectory(prefix=".remote-proof-", dir=args.out_receipt.parent) as temporary:
        local = Path(temporary)
        request = prepare_inputs(args, local, cfg)
        write_json(local / "request.json", request)
        audit = args.out_receipt.with_suffix(".request.json")
        write_json(audit, request)
        shutil.copyfile(__file__, local / "runner.py")
        archive = local / "inputs.tar"
        with tarfile.open(archive, "w") as tar:
            for name in ["inputs", "request.json", "runner.py"]:
                tar.add(local / name, arcname=name)
        try:
            result = ssh(cfg, ["mktemp", "-d", cfg["runs_dir"] + "/l3-proof-XXXXXXXX"],
                         check=True, capture_output=True, text=True, timeout=20)
            remote_root = result.stdout.strip()
            if not remote_root.startswith(cfg["runs_dir"] + "/l3-proof-") or not SAFE_PATH.fullmatch(remote_root):
                raise ValueError("worker returned an unsafe job directory")
            # stdin streaming avoids SCP's remote-shell path interpretation.
            with archive.open("rb") as data:
                ssh(cfg, ["tar", "--no-same-owner", "-xf", "-", "-C", remote_root],
                    stdin=data, stdout=subprocess.DEVNULL, check=True, timeout=90)
            command = ["timeout", "--signal=TERM", "--kill-after=10s", "300s", "env",
                       "-u", "LAY_RESOURCE_GUARD_ACTIVE", "-u", "LAY_RESOURCE_LEASE_HELD",
                       "-u", "LAY_RESOURCE_LOCK_PATH", "-u", "LAY_RESOURCE_SYSTEMD_RUN", "-u", "LAY_RESOURCE_SYSTEMCTL",
                       "LAY_RESOURCE_PROFILE=dedicated-20cpu", "CARGO_BUILD_JOBS=20", "RUST_TEST_THREADS=1",
                       "LAY_RESOURCE_GUARD_MODE=scope", "LAY_RESOURCE_CPU_QUOTA=2000%",
                       "LAY_RESOURCE_MEMORY_HIGH=24G", "LAY_RESOURCE_MEMORY_MAX=28G",
                       "LAY_RESOURCE_MEMORY_SWAP_MAX=1G", "LAY_RESOURCE_TASKS_MAX=512",
                       cfg["resource_guard"], "--", "python3", remote_root + "/runner.py", "_worker", remote_root]
            with local.joinpath("transport.log").open("wb") as log:
                result = ssh(cfg, command, stdout=log, stderr=subprocess.STDOUT, timeout=330)
            shutil.copyfile(local / "transport.log", args.out_receipt.with_suffix(".transport.log"))
            if result.returncode:
                raise ValueError("remote full-proof execution failed")
            result = ssh(cfg, ["cat", remote_root + "/receipt.json"], check=True,
                         capture_output=True, timeout=20)
            proof = json.loads(result.stdout)
            if proof.get("executor_sha256") != args.evaluator_sha256 or proof.get("proof_input_identities") != {
                k:{f:v for f,v in row.items() if f not in {"source", "path"}} for k,row in request["roles"].items()}:
                raise ValueError("returned proof is not bound to the request")
            verify_origin(request)
            # Keep remote manifest/delta paths. The existing admission therefore
            # requires portable current-baseline/delta content equality.
            write_json(args.out_receipt, proof)
            print(json.dumps({"kind":"l3_full_proof_remote", "verdict":proof.get("verdict"),
                              "seconds":proof.get("execution_seconds")}))
            return 0
        finally:
            if remote_root:
                try:
                    cleanup = "import pathlib,shutil,sys;p=pathlib.Path(sys.argv[1]);i=p/'inputs';i.chmod(0o700) if i.is_dir() else None;shutil.rmtree(p)"
                    ssh(cfg, ["python3", "-c", cleanup, remote_root], stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL, timeout=20, check=True)
                except (subprocess.SubprocessError, OSError):
                    # Worker itself cleans private inputs even if transport dies.
                    pass


def main(argv=None) -> int:
    os.umask(0o077)
    if len(sys.argv) == 3 and sys.argv[1] == "_worker":
        return worker(Path(sys.argv[2]).resolve())
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, default=CONFIG)
    for name in ["manifest", "delta", "corpus", "surface-evidence", "out-receipt"]:
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--max-fragments", type=int, required=True)
    parser.add_argument("--min-surface-support", type=int, required=True)
    parser.add_argument("--evaluator-sha256", required=True)
    args = parser.parse_args(argv)
    if args.max_fragments <= 0 or args.min_surface_support < 2 or not SHA.fullmatch(args.evaluator_sha256):
        parser.error("invalid proof dimensions or evaluator identity")
    return execute(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        if len(sys.argv) == 3 and sys.argv[1] == "_worker":
            write_json(Path(sys.argv[2]) / "failure.json", {
                "error_type":type(error).__name__, "reason":str(error)[:1200]})
        # Protected transport logs/requests carry detail; never expose endpoint,
        # feedback text or credentials through the live system journal.
        print("L3 remote proof failed; pending evidence retained", file=sys.stderr)
        raise SystemExit(1)
