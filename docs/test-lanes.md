# Rust Test Lanes

Lay's Rust tests use one discovery and execution entrypoint:

```bash
scripts/check-lay-tests.sh all
```

## Workstation resource ownership

Heavy local verification is fail-closed before Cargo starts. The changed gate,
full gate, and direct compile/discovery test-lane actions enter
`scripts/lay-resource-guard.sh` exactly once. Nested test-lane and Cargo calls
reuse that owner.

The default interactive profile provides three independent protections:

- a non-blocking lease under the user runtime directory allows only one heavy
  Lay verification owner across all Lay worktrees on the host;
- Cargo defaults to two compiler jobs and Rust harness execution remains one
  test thread;
- one user cgroup bounds the complete descendant tree to 200% CPU,
  `MemoryHigh=4G`, `MemoryMax=6G`, `MemorySwapMax=1G`, and `TasksMax=128`.

The wrapper also runs at niceness 10 and requests best-effort IO priority 7. It
requests `CPUWeight=10` and `IOWeight=10`; on the current workstation the CPU
controller is delegated and enforces that weight, while the user-slice IO
controller is not delegated (`io.weight` is absent). `ionice` is only a
scheduler hint here: it is not a measured bandwidth cap, and the active NVMe
scheduler is `none`. The concrete IO-risk controls are the two-job compiler
ceiling, swap cap, single-flight lease, and IO-PSI start preflight.

Before opening the scope, the wrapper requires at least 8 GiB available memory,
at least 4 GiB free swap when swap exists, memory full PSI avg10 at or below 2%,
and IO full PSI avg10 at or below 10%. A busy lease or unsafe pressure exits 75
as `BLOCKED_RESOURCE` before the test child starts. This is infrastructure
status, not a semantic test failure.

The default lock and limits are host policy, not part of a worktree. A larger
job count is rejected under the workstation profile, including by direct
`cargo-guard.sh` calls. Release orchestration on
the provisioned 20-CPU/31-GiB host explicitly selects
`LAY_RESOURCE_PROFILE=dedicated-20cpu`. That profile couples 20 Cargo jobs to a
2000% CPU quota, `MemoryHigh=24G`, `MemoryMax=28G`, `MemorySwapMax=1G`, and
`TasksMax=512`; it refuses hosts with fewer than 20 visible CPUs and cannot use
direct mode. Hermetic Cargo discovery carries this profile, its job count, and
the existing owner markers instead of silently reverting to two jobs.

CI without a user systemd manager must explicitly use
`LAY_RESOURCE_GUARD_MODE=direct`; the repository workflow does so. `CI=true`
alone never disables the cgroup boundary. Direct mode retains the host-wide
lease, two-job default, serialized Rust tests, niceness, IO priority, and
process-group cleanup, but it has no cgroup or pressure-preflight claim and is
reserved for disposable CI runners.

Signals are forwarded to one supervised process group. The wrapper sends TERM,
waits for a bounded grace period, then sends KILL and verifies the group is
gone. After normal completion and signal handling it also empties the transient
cgroup with bounded TERM-to-KILL escalation. The scope uses
`KillMode=control-group`, so a descendant that creates a new session still
remains bounded and is removed before the workstation lease is released.
`OOMPolicy` is intentionally absent: it is a service-unit setting and systemd
249 rejects it for `--scope`. `MemoryMax` still bounds aggregate cgroup memory;
when the supervised command returns after any child failure, the outer bounded
cleanup empties the remaining control group before releasing the host-wide
lease. This contract does not claim that every individual child OOM must itself
terminate Cargo.

Resource contracts are included in the lightweight self-test:

```bash
scripts/check-lay-tests.sh self-test
```

They use fake host metrics, Cargo, and systemd commands; they do not compile Lay
or create a real load spike. A harmless real-scope probe is a separate release
preflight and runs only `/usr/bin/true` or an equivalent read-only inspection.

Cargo first builds every default-feature test target with `--locked --offline
--all-targets --no-run --message-format=json` inside a networkless filesystem
sandbox. Dependency acquisition is a separate, non-authoritative step:

```bash
scripts/check-lay-tests.sh fetch
```

Each emitted test executable is then queried with the Rust harness `--list`;
the exact target/test union is
compared with [`manifest.json`](../scripts/test-lanes/manifest.json).

The manifest has four disjoint lanes:

- `correctness`: library and binary unit contracts without timing authority;
- `package`: explicit fixture/binary integration targets, with every external
  repository fixture pinned by size and SHA-256 in the manifest;
- `performance`: eleven explicit latency/resource budget tests, serialized and
  opt-in;
- `ignored`: fourteen externally admitted proof/helper tests, never promoted by
  the ordinary runner.

The current manifest contains 2,474 rows: 2,413 correctness,
 36 package, 11 performance, and 14 ignored. Twenty-seven rows are
process-isolated; the other 2,447 use target isolation. The process set is
fail-closed: sixteen
explicit environment/singleton mutators plus all eleven performance routes must
remain present in the registry.

Correctness and package targets run one process at a time with one Rust test
thread. Tests that mutate process-global environment or package state are
listed explicitly in the manifest, skipped from the target batch, and run in
fresh one-test processes. Bubblewrap makes the repository and host filesystem
read-only, masks the current user's Lay state, supplies fresh HOME/XDG paths,
and removes the network namespace. `/run` is an empty tmpfs, so host D-Bus,
Wayland, agent, and daemon Unix sockets are unreachable. Cargo discovery rejects
external `$CARGO_HOME` or ancestor configuration before compiling; any
repository-owned Cargo configuration is size/SHA-pinned in the manifest. The
runtime environment is allowlisted with a fixed system `PATH`, UTF-8 locale,
and UTC timezone. D-Bus, display, Wayland, SSH-agent, installed user commands,
and inherited project authority are absent. Raw target logs and a structured summary stay under
`target/test-lanes-results/`.

Wall-clock assertions that express a product budget live in `performance`.
Bounded waits used only as deadlock watchdogs remain functional correctness
contracts and do not claim latency authority.

The temporary [known-failure contract](../scripts/test-lanes/known_failures.json)
belongs only to TD-007. Its target/test set and normalized failure signatures
must match exactly: a new failure, renamed failure, changed outcome, or
unexpected fix makes the lane fail. Harness thread IDs, source line movement,
and the standard backtrace note are excluded from the signature. Exact known
failures produce `PASS_WITH_EXACT_KNOWN_FAILURES`, not a claim that their
semantics are correct. The sealed TD-006 denominator was 116 failures: 96
correctness and 20 package. Milestones 1-4 partition and close that exact set;
the current contract is empty and is bound to the milestone-4 zero-failure
observation. Volatile event timestamps and causal episode IDs remain excluded
from signatures; event kind, payload, outcome, and assertions remain bound.

Performance is explicit:

```bash
scripts/check-lay-tests.sh performance
```

It uses one test thread, warmup already owned by each test, and enables the IME
latency assertion. It is not ordinary CI authority because shared-host
contention can invalidate wall-clock budgets.

The sealed TD-006 closure observation is deliberately red: 8 of 11 performance
contracts pass and 3 report `BLOCKED_PERFORMANCE`. The failures are the
CanonicalL2Field p99 budget, exact-English guard RSS budget, and one unique
prefix cold-materialization budget. No budget was relaxed, and this opt-in
result does not convert a correctness or infrastructure failure into PASS.

Live desktop mutation remains a separate opt-in route:

```bash
LAY_RUNTIME_SMOKE_MANAGED_DESKTOP=1 scripts/check-lay-tests.sh live
```

Neither `all` nor `performance` can install, restart, or connect to the live
desktop session.
