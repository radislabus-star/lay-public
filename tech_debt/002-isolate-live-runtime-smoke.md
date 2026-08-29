# TD-002: Isolate Live Runtime Smoke Cases

Status: `DONE`
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

## Consequence Analysis

### Baseline

The current batch owns one mutable IME lifetime and observes one shared trace.
Its cleanup is distributed across the happy path, and process selection is
based partly on names rather than a captured `/proc` identity. This makes a
PASS order-dependent and makes a failed setup capable of leaving desktop state
behind.

### Considered Designs

1. **Selected: one explicit desktop transaction plus one process-local
   transaction per case.** Capture the pre-existing service, IBus engine and
   exact process identities once; give every case a fresh config/data/trace
   directory and a fresh managed engine; restore every captured desktop
   dimension in a fail-closed cleanup stack.
2. **Rejected: retain one managed engine and add run/case IDs to the shared
   trace.** Trace attribution improves, but mutable IME state, worker state and
   pending preedit authority still cross case boundaries. It cannot establish
   order equivalence.
3. **Deferred as disproportionate: start a private D-Bus/IBus desktop session
   per case.** This offers stronger isolation but duplicates desktop plumbing,
   diverges from the client-visible session under test, and is not required to
   remove the known contamination paths.

### Impact By Contract

- Candidate/lattice retention and ranking are unchanged; the harness does not
  alter runtime correction logic or expected outputs.
- False authority can only decrease because stale traces and prior-case IME
  state are no longer admitted as current evidence.
- Per-case engine startup adds wall time outside the measured client-visible
  assertions. Existing latency deadlines and captured key/output behavior stay
  unchanged.
- CPU/RSS/allocation behavior of production code is unchanged. The harness may
  have one extra short-lived process transition between cases, never concurrent
  duplicate candidate engines.
- Config, usage, feedback and trace caches are case-owned paths. No package or
  delta artifact is rewritten; package reload identity remains the runtime's
  responsibility.
- Learning and feedback files cannot leak between cases. This intentionally
  removes batch-order learning effects from smoke authority.
- Concurrency and stale-result risk is bounded by destroying the case-owned IME
  before the next case and refusing to begin while an uncaptured Lay engine is
  present.
- On setup, assertion, timeout, signal or teardown failure, all started children
  are stopped in reverse order. Every desktop restoration action is attempted;
  incomplete restoration makes the whole smoke fail.
- IME and daemon consumers receive the same scenario inputs and expected text.
  The only changed ownership is test infrastructure, not runtime authority.
- The maintenance cost is three narrow helpers: case isolation, exact desktop
  ownership, and route execution. They replace duplicated cleanup and broad PID
  selection and can be removed together with the live harness.

### Guardrails And Rollback

- The admission boundary is `--managed-desktop`; without it no live action is
  reachable.
- A captured PID is signalled only if PID, start time, executable and argv still
  match immediately before the signal.
- The global `ibus-daemon` is never restarted and must retain exact identity.
- Candidate binaries remain repository-local; no install or runtime authority
  update is admitted.
- Rollback is the single TD-002 commit. A failed live proof must preserve its
  receipt/logs and restore the captured desktop before rollback.

## TDD Plan

1. Add Python unit tests for trace rotation/truncation and per-case selection.
2. Add a fake-process failure test for cleanup order.
3. Run two stateful cases in both orders and individually.
4. Compare receipt-bound execution order and exact semantic projections while
   retaining raw trace counts and hashes.
5. Run one real managed GTK proof.

## Acceptance Gates

- Individual and batched semantic projections are identical for the same
  cases; raw trace counts remain published.
- Reversing receipt-bound execution order does not change outputs, semantic
  trace kind counts, or raw trace counts.
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

- Commit: implementation commit containing this completion record
- Independent review: fresh-context agent
  `01a04b14-003b-7500-a6b2-a608083fe930`; initial score `5/10`, one bounded
  correction pass, final score `9/10`, no unresolved correctness or
  desktop-safety findings
- Verification: focused Python tests `45/45 PASS`; Python compilation and
  shell syntax `PASS`; `scripts/check-lay-changed.sh` `PASS`;
  `scripts/check-lay-audit-50.sh` `50/50 PASS`; architecture receipt `PASS`;
  evidence manifest `163/163 PASS`; V6 verdict reproduced byte-for-byte as
  `RUNTIME_SMOKE_ORDER_ISOLATION_PASS`
- V6 semantic projection SHA-256:
  `ffe69c98cef8049618dce771ce9b3613060a126996699465962dc3528ac20f4d`
- Runtime authority changed: `false`; candidate binaries remained
  byte-identical to the installed `lay-daemon`, `lay-ibus-engine`, and
  `lay-test-input`
- Follow-up: the isolated proof discovered, but did not repair, the
  manual-toggle visible-postcondition race now owned by `TD-009`
