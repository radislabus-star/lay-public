# TD-002: Isolate Live Runtime Smoke Cases

Status: `READY`
Priority: `P0`
Class: end-to-end test reliability
Size: `M`
Depends on: TD-001

## Why Now

The live GTK harness is the only proof for several visual IME behaviors, but a
multi-case run shares one managed IBus session and a global trace. A good result
can therefore depend on case order or stale process state.

## Evidence

- `managed_ime_session(...)` wraps the whole selected case loop.
- Trace verification counts records from a file offset without a per-case
  transaction identity.
- Cases create isolated daemon config only when overrides are present.
- The harness invokes raw `cargo build` when binaries are absent instead of the
  repository Cargo guard.
- Process cleanup is manual and error paths can leave ambiguous desktop state.
- The current harness stops the user `lay-daemon`, terminates all discovered Lay
  IBus engine PIDs, and restarts the service without first proving process
  ownership.

## Target State

Every case has a unique run ID, isolated daemon/config/device state, bounded
trace projection, and deterministic cleanup. Batched execution is equivalent to
running each case alone in any order. The harness mutates only processes it
started or an explicitly captured Lay-managed service and restores exact
pre-test service, engine, layout, and trace state.

## Scope

- Add a run/case identity to trace filtering or use a dedicated trace file per
  case.
- Reset or recreate managed IME state at the proven isolation boundary.
- Always isolate config/data paths for test-owned daemons.
- Route builds through `scripts/cargo-guard.sh`.
- Make cleanup idempotent for success, assertion failure, timeout, and signal.
- Preserve the existing global `ibus-daemon` whenever the managed contract says
  to preserve it.
- Capture pre-test `lay-daemon` unit state/MainPID, Lay IBus engine set, active
  engine/layout, trace path, and harness process group before mutation.
- Require explicit managed-desktop mode before stopping a pre-existing
  Lay-managed service; ordinary self-tests stay process-local.
- Never signal an unrelated or identity-ambiguous PID even when its command line
  contains `lay`.

## Non-Goals

- Do not restart unrelated desktop processes.
- Do not install candidate binaries globally.
- Do not replace physical input injection with direct method calls.
- Do not loosen expected text or toggle counts.

## TDD Plan

1. Add Python unit tests for trace rotation/truncation and per-case selection.
2. Add a fake-process failure test for cleanup order.
3. Run two stateful cases in both orders and individually.
4. Compare exact JSON receipts.
5. Run one real managed GTK proof.

## Acceptance Gates

- Individual and batched results are identical for the same cases.
- Reversing case order does not change outputs or trace counts.
- Timeout leaves no test daemon, virtual device, or candidate IBus engine.
- Every started child is gone and every pre-existing non-owned PID is unchanged.
- The pre-test `lay-daemon` enabled/active state, active engine/layout, and trace
  destination are restored exactly; the global `ibus-daemon` PID is preserved.
- No raw `cargo` invocation remains in the harness.

## Risks And Guardrails

- Per-case IBus restart can itself create flakes. Use the narrowest proven reset
  boundary and record process identities.
- Do not read unrelated user trace records.
- Treat incomplete restoration as a failed smoke, not cleanup noise.

## Independent Review Brief

Force failures at dialog start, device creation, daemon start, sender timeout,
and trace read. Verify cleanup and state restoration. Score 1-10.

## Completion Record

- Commit: pending
- Review score: pending
- Verification: pending
