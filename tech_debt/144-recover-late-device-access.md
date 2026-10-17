# TD-144 — Recover late input-device permissions (GitHub #49)

Status: DONE_SOURCE (341/341); INSTALLED_1.0.87 (identity/device readiness verified); native NOT TESTED. Priority: P1. Baseline: e0a13039, source 1.0.86.
Patch source version: 1.0.87; package/extension version metadata is synchronized.
User authorized the patch, verification, push and closing public issue #49.
The initial source-publication request did not include installation, host permission
changes or a new binary release. The later explicit instruction "обнови" authorizes
installation of the same patch; the result is recorded below.

## Confirmed cause and scope

At the 1.0.86 baseline, `startup_runtime::make_virtual_keyboard_for_runtime`
attempted creation once;
failure remains `None` in the existing shared mutex. Keyboard discovery runs
once and excludes inaccessible devices. A live daemon therefore cannot acquire
devices whose permissions become available later. Restart=on-failure cannot
repair the live process. Its warning also uses the debug-only logger.

The existing main thread only joins listeners and has no readiness maintenance.
Use that same thread to maintain device availability; do not add a semantic
owner, output route, retry of a text edit, background timer thread or global
IME restart. Existing listener state, trigger FSM, edit verifier, text executors
and IME contracts retain ownership.

## Minimal implementation

1. Replace blocking startup joins with a bounded one-second maintenance loop
   honoring the existing SHUTDOWN_REQUESTED flag. Retain normal joins and final
   virtual-key release at shutdown.
2. Retry missing uinput under the existing shared device mutex. Serialize create
   and initial key release; never replace an available device. Skip a busy mutex
   so readiness monitoring cannot wait behind text output. Detect-only never
   creates an output device. No text edit is replayed when readiness returns.
3. Rescan accessible keyboard/pointer paths using existing discovery and device
   exclusions. Keep one listener per (kind,path); reap finished listeners before
   restarting them. Support the explicit --device path and late initial access.
   Advance the existing field-context epoch on keyboard listener changes so a
   replaced listener cannot reuse old word/gesture context.
4. Warn once per unavailable/recovered state without requiring debug logging;
   warnings contain device status only, not typed text. No chmod/usermod/udev edits.

## Required proof and review

- Controlled-violation RED: disable missing-device acquisition in a frozen remote
  source copy. Label this as a deliberate violation, not an unmodified native
  baseline reproduction. A real permission-denied temporary file then becomes
  accessible; the production recovery helper must acquire once and retain it.
- Detect-only and already-ready controls; busy/poisoned shared mutex controls.
- Real listener lifecycle: inaccessible/absent then available; no duplicate live
  listener, finished listener recovery, distinct paths, virtual-device exclusion.
- Shutdown remains bounded and does not spawn/reacquire after shutdown starts.
- Run focused regressions, daemon and affected integration contracts remotely
  under existing resource/Cargo guards. Refresh the canonical graph and record
  evidence in the owning document. Physical Bluefin/Wayland acceptance is separate.
- Fresh independent plan review and code review, 1–10 score; at most two code
  review/repair passes. Publish tested source to both working branches; close #49
  only after verifying the public commit and posting its exact verification scope.

## Alternatives already compared

- 9/10: readiness recovery in the current daemon main thread (selected).
- 6/10: startup-only waiting; misses subsequent permission/device changes.
- 3/10: installer-only permission workaround; does not repair runtime recovery.

## Evidence

Initial source analysis: `/home/ubu/.cache/lay/development/issue-triage-20261016/CHECK49-SOURCE86.json`.
Native permission recovery and new installed behavior: NOT TESTED.

Focused 10/10 PASS: `/home/ubu/.cache/lay/development/td144-source-subset-6muuli82/RESULT.json`.
Controlled publication violation: expected 1/1 FAIL after permissions become
available and a recovered handle is deliberately discarded;
`/home/ubu/.cache/lay/development/td144-controlled-red-xkfmtywl/RESULT.json`.
The first mutation attempt did not compile and is not RED evidence; retained at
`td144-controlled-red-zhebrtuv`. Production source was restored in each attempt.

Plan review: 9/10, fresh reviewer `/root/issue49_device_access_plan_review`.
Incorporated absent-keyboard startup, independent pointer discovery, live-handle
retention, explicit path recovery, shutdown checks, immediate lifetime epoch
invalidation and idle consumption before deferred work. Physical held-modifier
tracking remains intact when stale gesture intent is cancelled.

Code review pass 1: 8/10, fresh reviewer
`/root/issue49_device_access_code_review`. One P2: field-epoch invalidation
cancelled other pending triggers but retained force-RU/EN press timestamps.
The existing ForceLayoutHotkeys owner now cancels those timestamps and its
other-key flag; its configuration and held physical modifiers remain intact.
A regression presses each configured force key, changes the epoch, then releases
it and requires no layout-trigger completion or modifier reset.

Final focused regressions: 11/11 PASS,
`/home/ubu/.cache/lay/development/td144-source-subset-_a2xm0gj/RESULT.json`.
Code review pass 2/2: 9/10; the same independent reviewer confirmed the P2 fix
and found no remaining material defects. No further review pass is planned.
Final daemon/affected-contract gate: 341/341 PASS (290 daemon + 51 contract
tests); `/home/ubu/.cache/lay/development/run-7ggr8s_q/RESULT.json`.
Source archive SHA-256:
`b334c6a2c17c63eb9e77774b5ec47036cf30bd0732a9f47990b4910a5ffda6db`.
This is the selected development gate, not the full release/client matrix.
Canonical graph/checks PASS; result:
`/home/ubu/.cache/lay/development/td144-final-graph-20261016-2/GRAPH_RESULT.json`.

First canonical run: graph invariants 11/11 PASS, but the complete gate FAILed
on `startup_runtime.rs:unowned_thread_sleep`;
`/home/ubu/.cache/lay/development/td144-final-graph-20261016-1/GRAPH_RESULT.json`.
The maintenance wait now reuses the existing input-loop poll/timeout helper
with no active fd (negative fd is ignored by poll). No new delay owner or guard
exception was added. This small wiring change follows code review pass 2;
the final source gate (341/341) and canonical checks PASS on the adjusted bytes.

Held-modifier preservation is proved for an existing listener. A modifier already
held before a recovered evdev fd is opened is NOT TESTED; new listeners still
use the pre-existing ShiftState initialization. The retry cadence does not bound
a blocking kernel acquisition call or establish native input latency.

Final source parity: 742 Rust files plus Cargo.toml/Cargo.lock match the
341/341 tested snapshot; `/home/ubu/.cache/lay/development/td144-source-parity-final.json`.
The earlier pre-wait-adjustment 341/341 receipt remains at `run-4wyqx1a4/RESULT.json`.
Publication and issue-state verification are recorded at
`/home/ubu/.cache/lay/development/td144-publication-20261016/PUBLISH.json`;
installed runtime and latest GitHub Release remain 1.0.86.

Canonical evidence: 11/11 graph invariants, 26 graph guard tests, 2 scope
projection tests and 15 canon tests PASS; exact changed/new-path decision
coverage PASS. Generated graph artifacts were fetched from the guarded worker.
No architecture guard, allowlist, input authority, model or delivery route changed.

## User-directed installation — 2026-10-16

The user explicitly requested "обнови" after source publication. The installed
source is `50d314b382c0383764ea8e738a2d2cd274c2628e`, version 1.0.87.
The existing three-core-binary transaction installed CLI/daemon/IME and the two
extension version files. Binaries were built once on 192.168.3.94 under the
shared dedicated-20cpu resource/Cargo guards, using all 742 tested Rust hashes
and exact tested Cargo.toml/Cargo.lock. The 341/341 source proof is reused;
unchanged functional tests and native key sequences were not repeated.

Receipts:
- `/home/ubu/.cache/lay/development/td144-install87-20261016/BUILD.json`
- `/home/ubu/.cache/lay/development/td144-install87-20261016/INSTALL.json`
- `/home/ubu/.cache/lay/development/td144-install87-20261016/POST-INSTALL.json`

Loaded daemon and IME SHA-256 match the installed build. CLI and GNOME panel
report 1.0.87. The daemon has uinput open and 8 physical input-device fds.
The selected `lay-ime-ru`, mode `ru`, input sources, configuration, global IBus,
L1.1/L3 process identities and 24 protected files are preserved. The prior 1.0.86
files and modes were copied and hash-verified in the transaction's `backup/`
directory before replacement. No host permission or model change occurred.

This establishes installation identity and device readiness. New physical-client
behavior and native latency remain NOT TESTED / NOT MEASURED. The historical
349 PASS / 51 FAIL / 32 BLOCKED browser matrix does not certify 1.0.87.
