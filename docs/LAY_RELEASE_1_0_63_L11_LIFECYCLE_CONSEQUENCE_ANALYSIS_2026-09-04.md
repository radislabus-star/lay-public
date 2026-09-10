# Lay 1.0.63 L1.1 lifecycle consequence analysis

## Scope and measured pre-repair baseline

The following baseline was captured before the installed L1.1 process was
reconciled on 2026-09-04. It is historical input to the repair, not the current
live state.

- The installed CLI, DBus extension, daemon, L3 service, and managed IME are 1.0.63.
- The one live `lay-l1.1-serve` process still executes the deleted 1.0.62 image while the installed `lay-l1.1-serve` file is 1.0.63.
- The live service is healthy and serves the admitted 77,962,328-byte Phase8I V9 package, but executable process parity is false.
- The release controller manages daemon, L3, and managed IME lifecycle, but does not manage L1.1.
- The global IBus process and selected `lay-ime-ru` engine are outside this repair's mutation authority.

Exact baseline bytes, PID, paths, modes, and hashes are pinned by
`LAY_RELEASE_1_0_63_L11_LIFECYCLE_IMPLEMENTATION_V10_2026-09-04.json` and its
`IMPLEMENTATION_PREFLIGHT_V10_RECEIPT.json`.

## Designs considered

1. Leave the healthy stale process in place — **2/10**. Lowest immediate work,
   but leaves a mixed-version runtime and reproduces `AddrInUse`/`WouldBlock`
   during frame-bound proof. Rejected.
2. Kill and restart the current PID once — **5/10**. Repairs this workstation,
   but the next release repeats the defect and failure has no proved rollback.
   Rejected.
3. Add L1.1 to the release transaction, rollback, process parity, and an
   installed-recovery entrypoint — **9/10**. This is the selected minimum
   systemic fix: one release-local mutation owner, pidfd-bound process identity,
   peer-bound health plus executable-hash parity, and recovery from the existing
   immutable snapshot.

The selected design adds no correction, ranking, or verifier route. It extends
only the existing release transaction.

## Consequences and invariants

### Candidate retention, ranking, and false authority

No candidate/lattice, ranking, `DecisionCore`, verifier, or `SafetyGate` code is
changed. L1.1 health proves service readiness, the server PID, and the reported
package path/byte count; it does not grant correction authority. A healthy
process with the wrong executable hash cannot satisfy release parity.

### Latency, CPU, RSS, and allocation behavior

The normal typing hot path is unchanged. During release or installed recovery,
there is one bounded L1.1 cold start; requests in that short window may fail
closed and abstain. The old process is terminated before the replacement loads
the package, so two full L1.1 hosts are not intentionally resident together.
The exact cold-start tail and peak RSS are not claimed by this change and remain
live observations; the gate proves bounded polling and single-process parity,
not a latency improvement.

### Cache, package, delta, and learning identity

The package file path, size, and disk SHA-256 are checked around mutation. The
peer-bound health response must report that same path and decoded byte count.
The current health schema does not expose the SHA-256 of bytes already loaded
into the process, so loaded-package hash identity is explicitly `UNKNOWN`, not
promoted from disk parity. The package and L2 trees are never written by
installed recovery. Restarting L1.1 resets only process-local uptime/request
counters; it does not alter learned data, feedback, package generations, or
delta state. A detected disk-file change fails closed.

### Concurrency and stale-result races

Exactly one six-field L1.1 invocation is captured. The guard opens a pidfd
before it revalidates the PID namespace identity, executable path, all six argv
fields, and executable SHA-256. That same pidfd receives `SIGTERM` and is polled
for at most five seconds; there is no numeric-PID TERM and no blocking shell
`wait`. Health is read from one bounded Unix connection whose `SO_PEERCRED` PID
must equal the expected process, followed by a second identity validation. No
`pkill`, wildcard signal, socket unlink, retrying TERM, or second owner is
allowed. A reused/changed PID or foreign socket peer therefore fails closed.

### Failure and rollback

Forward install success requires one ready process whose `/proc/<pid>/exe` hash
equals installed bytes. Start, health, or hash failure enters the existing
rollback. After snapshot bytes are restored, the same lifecycle reconciles
L1.1 to those bytes. Installed recovery never rewrites installed files; on
failure it starts the immutable snapshot binary with the exact captured argv
only after exit of the failed release owner is proved. Identity mismatch,
pidfd error, or the bounded TERM timeout enters a dedicated `UNVERIFIED`
terminal: there is no second TERM and no snapshot spawn beside an owner whose
exit was not proved.

### IME/daemon compatibility and second-order effects

Installed recovery must preserve the exact global IBus PID, selected engine,
daemon PID, L3 PID, and managed IME PID. Normal release retains its existing
managed-process restarts. The only expected runtime identity change in installed
recovery is the L1.1 PID and executable hash. A slower or failed package load can
temporarily reduce restoration coverage, but cannot create false correction
authority or a second text mutation.

### Maintenance and removal

The code remains in the versioned 1.0.63 controller; the shared runtime helper
stays byte-identical. This avoids creating a second long-lived lifecycle owner.
The release-local code can be removed when a separately designed managed L1.1
unit or generic release controller owns capture, rollback, health, and process
parity with equivalent tests.

## Proof denominators and promotion gates

- Isolated controller suite: all lifecycle identity, fault, rollback, and parity
  cases must pass; no live process is mutated by unit tests.
- Source scope: shared runtime controller, Rust correction sources, package,
  installed trees, and rollback snapshot remain byte-identical during source
  repair and installed recovery.
- Live process parity: exactly one L1.1 process, ready health, exact package path
  and decoded byte count, matching `SO_PEERCRED` PID, and running executable
  SHA-256 equal to installed 1.0.63 bytes.
- Desktop projection: exact pre/post IBus PID, engine, daemon PID, L3 PID, and
  managed IME PID equality for installed recovery.
- Product proof: the frame-bound TD-117 E2E is a separate `1/1` gate. A service
  lifecycle PASS does not by itself prove correction quality.
- Final changed-source gates, bounded independent review, documentation,
  commit, and push remain required before this lifecycle follow-up is closed.

## What is not tested by this design document

This document does not prove L1 quality percentages, cold-start latency,
loaded-package SHA-256, package quality, or the TD-117 text transition. It only
bounds the lifecycle change and names the later tests that own those claims.

## Independent rejection and bounded repair

The first implementation snapshot (`fbbabf5b...`) was rejected `4/10` with
High 2 / Medium 4 because numeric PID signalling, blocking `wait`, unbound
socket health, compressed failure topology, and mocked-only interleavings did
not satisfy its claims. It was never run live. Design route V9 separates
release success, cleanup, and snapshot fallback. Preflight V11 retained the
first incomplete paper contract as `BLOCKED_BEFORE_CODE`; V12 is the corrected
`READY_TO_IMPLEMENT` contract. The implemented repair is limited to the pidfd
and peer guard, controller routing, executable tests, and this evidence; it does
not alter Rust correction authority or the shared runtime controller.

## Observed post-repair state

The stale process image was replaced on 2026-09-04. The first replacement was
started as a background child of an interactive controller session. It passed
process and health parity, but disappeared when that parent execution cgroup
ended. This was not an L1.1 crash and did not invalidate the binary or package;
it exposed that shell detachment did not provide a durable lifecycle owner.

The systemic repair moves release starts to one uniquely named transient
user-systemd service. The controller requires systemd `MainPID` ownership in
addition to readiness. A real opt-in integration ends the controller parent,
then proves that the exact six-field process remains active with the expected
executable path and SHA-256. The test removes its transient unit afterward.

If `systemd-run` succeeds but `MainPID` cannot be bound, the controller stops
only the unit it just created. Exact `inactive` is the sole clean-stop proof.
States `failed`, `active`, an unreadable state, or a stop timeout preserve RC
`70`; no snapshot fallback may start beside a potentially surviving process.
This distinction is required because `SendSIGKILL=no` deliberately forbids
systemd from converting a graceful-stop timeout into an unreviewed kill.

Current live projection after repair:

```text
Lay CLI                         1.0.63
L1.1 unit                       lay-l11-release-1063.service
L1.1 PID                        2201795
L1.1 state                      active/running
running executable SHA-256      856df7ab1d8b512f83c89aa5b21461a03ca7370e2b30025d2a4fe1932738ab35
installed executable SHA-256    856df7ab1d8b512f83c89aa5b21461a03ca7370e2b30025d2a4fe1932738ab35
package disk SHA-256            bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7
global IBus PID                 4715
daemon PID                      1304592
L3 PID                          1304516
selected engine                 lay-ime-ru
```

The focused product proof remains a separate denominator. Its exact ignored
test passed `1/1`: `плозо -> плохо` was a complete lexical `Winner`, capability
`1`, and a verified current-word transition. The same execution honestly
retained `востанавливать` as a complete two-target tie with no authority. Total
test time was `61.69 s`, including a cold Productive/L1.1 path; this is measured
P1 startup-latency debt, not hidden inside the lifecycle PASS.

The two allowed durability-review correction passes are recorded in
`docs/structural_gates/receipts/LAY_RELEASE_1_0_63_2026-09-04/INDEPENDENT_L11_DURABILITY_REVIEW.md`.
Their objective final suite passed `31/31`, plus syntax, ShellCheck, Python
compile, controller self-test, and diff checks. The final dedicated-20-CPU
changed gate passed all `2,497/2,497` selected correctness/package tests with
zero known semantic or infrastructure failures, plus `cargo check`, transition
replay, and the unsafe-edit scoreboard. The review history and final gates are
kept separate: a routing/lifecycle PASS is not a broad language-quality claim,
and the loaded package SHA-256 remains `UNKNOWN` because the health protocol
does not expose it.
