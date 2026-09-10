# Short maintenance loop

Development checks are not release acceptance. Build/test execution is remote
only. Nothing here installs binaries, restarts the desktop or accepts TD-121.

## One-time machine setup

Put the following in `~/.config/lay/development.json` (already configured on the
current workstation). Paths belong to your worker, not to runtime source code:

```json
{
  "remote": "e@192.168.3.94",
  "runs_dir": "/home/e/projects/lay-development-runner",
  "target_dir": "/home/e/projects/lay-td119-gate-v1/target"
}
```

SSH must work noninteractively. The worker needs the existing pinned Rust
toolchain/dependency cache, Python >=3.10, rsync, bubblewrap and user systemd.
Another worker is selected with `--config /absolute/config.json`. A worker
whose machine-id matches the origin is rejected; there is no local fallback.
Keep `runs_dir` and `target_dir` outside Lay's `.cache/lay`, `.config/lay` and
`.local/share/lay`: existing test isolation deliberately masks these locations.

## Everyday commands, from the checkout being edited

```sh
# Read-only plan; includes staged, unstaged, added, deleted and renamed paths.
python3 scripts/dev-check.py plan

# Automatic conservative affected scope, copied and checked remotely.
python3 scripts/dev-check.py check

# One explicit inner-loop target. This is NOT all affected-contract coverage.
python3 scripts/dev-check.py check --target bin:lay-ibus-engine

# Include committed branch work since a chosen base as well as dirty work.
python3 scripts/dev-check.py check --base origin/main

# Regression tests for this development tooling and the existing guards.
python3 scripts/dev-check.py self-test
```

The automatic selector knows one private source boundary: IME-only changes
select its binary plus **all integration-test targets**, including tests that
inspect source dynamically. A cross-owner reference, dynamic cross-owner code
include, custom Cargo test layout or any unknown/shared/mixed change broadens
to existing correctness/package lanes. It does not claim to be a Rust
dependency parser. No changes means `NO_CHANGES`, not a test PASS.

For a small known regression use explicit `--target` during editing, then the
automatic affected check when ready. Multiple `--target` flags form an explicit
union. Cargo may compile supporting binaries for integration targets; selected
test execution and compilation cost are different measurements.

Focused discovery compiles only requested Cargo test targets, discovers the
entire live test set and reuses existing sandbox/isolation rules. Correctness
and package tests must execute exactly as discovered. Zero selected tests,
missing targets and every selected failure fail the command. Performance and
ignored tests are listed as excluded. New tests/targets are reported as drift
from the canonical manifest without silently rewriting it. Full release lanes
continue to require their canonical manifest and existing acceptance rules.

## What happens automatically

1. Freeze a complete source snapshot and its SHA-256; reject concurrent edits.
2. Transfer into a fresh run directory and verify the archive remotely.
3. Acquire the existing host-wide heavy-execution lease and resource limits.
4. Update only the runner-owned disposable mirror, preserving one source path
   for Cargo reuse. Unowned directories are refused. Earlier run archives are
   preserved; no user checkout is overwritten.
5. Verify every source file hash/mode, run the selected checks, verify source
   stability and save the result even when a check fails.
6. Print compact status plus local and remote evidence locations.

Limits remain CPU 2000%, Cargo jobs 20, Rust test threads 1, MemoryHigh 24G,
MemoryMax 28G, swap 1G, TasksMax 512 and Cargo target <=12GiB. Resource refusal
is an error, never a silent smaller/local run. A fresh run is required after a
failure; the tool never retries until green.

Local output: `~/.cache/lay/development/run-*/{request.json,run.log,RESULT.json,transport.json}`.
Remote output: configured `runs_dir/run-*/`, including full test manifests and
logs. `request.json` owns source commit, complete file hashes/modes, selection
and archive identity. The remote mirror has an empty metadata Git commit for
existing provenance tooling; that synthetic commit is NOT the source commit.
Do not commit/push from that disposable mirror.

Archive/transfer/setup, Cargo discovery/build and test execution have different
costs. Compare their recorded timings separately; a focused PASS is not a
measured speedup of the entire development cycle.

## Actual IME client, without reconstructing a cache harness

Use the versioned [harness](scripts/proof/ime-client/README.md), its example
configuration and an exact candidate/dependency identity. Store the configured
JSON on the remote worker once; when candidate bytes change, update its
expected hash explicitly.

```sh
python3 scripts/dev-check.py client \
  --client-config /home/e/projects/lay-development-runner/boundary-deps-UjRxy5/client-config-admitted.json
```

The current config uses the nine-role manifest, including lexical phase memory.
The historical TD-124 eight-role config is retained as evidence and is now
refused before launch. This dependency correction fixes the private candidate
abort, not the still-open cold correction or terminal-consumer acceptance.

The wrapper creates a fresh output automatically. The client uses a private
IBus/D-Bus and preserves the original five scenarios, assertions, deadlines
and cleanup. Its own smaller envelope remains CPU200%,1536M,swap0,Tasks128,90s.
Tooling tests and a reproduced product failure are separate verdicts. Never
replace a failing cold case with sleeps, retries or a warm-only success.

## What this deliberately does not solve

- TD-121 product/client acceptance and release 1.0.66 remain separate work.
- No automatic package/candidate admission, install or production restart.
- No broad runtime rewrite, new ownership controller or crate split.
- Removal of obsolete compatibility routes and separation of research/runtime
  code need their own evidence and a separate decision. Large files alone are
  not a reason to create more modules.

Before release, run the existing mandatory full/release and actual-client
checks on the remote worker. This development command cannot waive them.

## Existing opt-in diagnostics

IME tracing uses `debug_action_log` in `~/.config/lay/config.json` and writes
`~/.local/share/lay/ibus_engine_debug.jsonl` (or `LAY_IBUS_TRACE_PATH`). The
IME refreshes its logging configuration; this option does not require a new
global IBus or keyboard-daemon restart. Preserve a private snapshot promptly:
the existing 500KiB rotating tail can overwrite the first failure.

The keyboard daemon already supports `--debug-log` (diagnostic stderr/journal),
`--verbose` (per-key output) and `LAY_DEBUG_LOG=1`. These logs can include typed
text; keep captures private and do not commit them. Never start a second daemon
beside the existing owner; changing its startup flags requires the separately
authorized keyboard-safe service procedure.

The separate `LAY_IME_DEBUG` recorder experiment was shelved before compilation
or installation. Those variables are not implemented in the current source.
Process liveness or an enabled log is not proof of functioning input admission.
