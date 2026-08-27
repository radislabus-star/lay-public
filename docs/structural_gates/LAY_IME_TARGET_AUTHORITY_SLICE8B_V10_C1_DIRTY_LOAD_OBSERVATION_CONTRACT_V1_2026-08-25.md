# Slice 8B V10 C1 Dirty-Load Direct Latency Observation V1

Date: 2026-08-25

## Purpose

The installed Lay is expected to operate while other applications consume CPU
and IO. This route therefore measures the already sealed C1 exact-search
executable under the mini-PC's current Nando, btop and other background load.
It does not ask the operator to create a quiet host and does not stop or tune
any foreign process.

The result is a product diagnostic named `DIRTY_LOAD_OBSERVATION`. It compares
the observed direct per-request latency distribution with the frozen C1
thresholds, but it cannot become `C1_PASS` or `C1_FAIL`. Formal clean C1 remains
`NOT_MEASURED / BLOCKED_ENVIRONMENT` and all clean latency rights remain intact.

## Reused sealed subject

No build or parity rerun is admitted. The route consumes the existing sealed C1
executable and its completed semantic prerequisite:

```text
ELF SHA-256       ead184029a2923cfd24c5d02e91e91f9d69fb01c7daa0fb21ba69f267234c93c
ELF Build ID      665458be5064a689951c6f074276ab0bb4d2beb4
ELF bytes         20,531,304
parity records    382/382
mismatches        0
false certs       0
```

The package, V10 sidecar, V7 denominator and 382-entry schedule retain the exact
C1 identities. Production timing authority remains only:

```text
result.search_elapsed_us
result.total_elapsed_us
```

External process wall time and environment samples are diagnostics only.

## Route separation

The dirty route owns new paths:

```text
remote state   /home/e/.local/state/lay/slice8b-v10-c1-dirty-direct-latency-v1-20260825
remote result  /home/e/.local/share/lay/provenance/slice8b-v10-c1-dirty-direct-latency-v1-20260825
local index    docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_DIRECT_LATENCY_V1_2026-08-25/
```

Before execution and after publication the controller must prove that the clean
C1 markers are byte-for-byte and name-for-name unchanged:

```text
S1.available T1.available T2.available S2.available S3.available
T3.available T4.available S4.available S5.available T5.available
build.consumed-before-exec parity.consumed-before-exec
```

The clean C1 final path must remain absent. The controller may not call the
clean C1 controller's `remote_run_matrix()` route.

## Fixed workload

The process order and implementation are unchanged:

```text
S1 -> T1 -> T2 -> S2 -> S3 -> T3 -> T4 -> S4 -> S5 -> T5
```

Each S process performs one 382-query warmup and 100 measured complete rounds,
producing 38,200 samples. Each T process uses twenty pinned workers, one
fixed-shard warmup burst and 250 START/END-barrier rounds, producing 95,500
samples. The total denominators are:

```text
S pooled       191,000
T pooled       477,500
```

Each dirty S/T route has a create-new marker consumed before subject execution.
No retry, replacement process, route reordering or adaptive repetition is
permitted after a sample is visible.

## Loaded environment

Quiet-host admission is intentionally absent. CPU, memory and IO PSI, running
processes, per-process CPU deltas, temperature, throttle counters, topology,
governor, EPP and turbo state are recorded before and after every route.

Nando, btop and other background activity are expected evidence, not blockers.
The controller must not stop, suspend, reprioritize, re-affinitize or otherwise
modify them. It may terminate only its own failed subject process group.

Executable/input identity failure, subject failure, malformed samples, affinity
or migration mismatch, publication failure, or clean-marker drift makes the
dirty observation invalid. Background pressure, high PSI and foreign CPU use
do not. Temperature and throttle behavior are reported as observed product
conditions and never silently removed from the result.

## Aggregation and threshold comparison

Percentiles use the frozen nearest-rank formula:

```text
index = ceil(n * 99 / 100) - 1
```

The same preregistered comparisons are calculated without changing their
meaning:

```text
S pooled search p99                         <=3,000 us
S pooled total p99                          <=5,000 us
each S1..S5 search p99                      <=3,000 us
each S1..S5 total p99                       <=5,000 us
T pooled total p99                          <=5,000 us
each T1..T5 total p99                       <=5,000 us
max(run x worker total p99)                 <=5,000 us
errors / unresolved                         0 / 0
```

The receipt reports `thresholds_would_pass=true` only when every comparison is
true. Per-query p99, T search p99, maxima and worker spread remain diagnostics.
No threshold or conjunct may be added after samples are observed.

## Verdict and consequences

The only successful observation verdict is:

```text
verdict                  DIRTY_LOAD_OBSERVATION
thresholds_would_pass    true | false
```

This is directly relevant to the user's decision about Lay under ordinary
machine load. It is not a laboratory attribution result and cannot prove which
fraction is caused by Nando, cache pressure, scheduling or the V10 executor.

Neither value admits runtime integration, formal B, PMU work, V12, installation
or daemon restart. A later user decision may use the result to choose the next
paper route.

## Forbidden effects

```text
Cargo/rustc/build/check/test               forbidden
parity rerun                               forbidden
perf/PMU                                   forbidden
host policy or affinity tuning             forbidden
foreign process stop/suspend/kill           forbidden
clean C1 marker consumption                 forbidden
clean C1 final publication                  forbidden
adaptive rerun                              forbidden
installed Lay/runtime mutation              forbidden
V12 action                                  forbidden
```

Remote evidence is sealed read-only with a complete `SHA256SUMS` before one
same-filesystem final rename. The local index is published by the same rule.
No final-path mutation is permitted after either rename.
