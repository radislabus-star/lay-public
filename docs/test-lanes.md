# Rust Test Lanes

Lay's Rust tests use one discovery and execution entrypoint:

```bash
scripts/check-lay-tests.sh all
```

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
- `ignored`: thirteen externally admitted proof/helper tests, never promoted by
  the ordinary runner.

The current TD-007 milestone-1 manifest contains 2,367 rows: 2,308 correctness,
35 package, 11 performance, and 13 ignored. Twenty-seven rows are
process-isolated; the other 2,340 use target isolation. The process set is
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
must match exactly: a new failure, renamed failure, changed outcome, or unexpected
fix makes the lane fail. Harness thread IDs, source line movement, and the
standard backtrace note are excluded from the signature. Exact known failures produce
`PASS_WITH_EXACT_KNOWN_FAILURES`, not a claim that their semantics are correct.
The sealed TD-006 denominator is 116 failures: 96 correctness and 20 package.
Two unchanged runs have identical target/test identities and normalized
signatures. Volatile event timestamps and causal episode IDs are excluded from
the signature; event kind, payload, outcome, and assertions remain bound.

Performance is explicit:

```bash
scripts/check-lay-tests.sh performance
```

It uses one test thread, warmup already owned by each test, and enables the IME
latency assertion. It is not ordinary CI authority because shared-host
contention can invalidate wall-clock budgets.

The TD-006 closure observation is deliberately red: 8 of 11 performance
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
