# V10 E1 Traversal D7 Worker Topology Sweep V1

Date: 2026-08-26

## Purpose

D7 is the smallest intervention that can separate the known `+18.77 ns/edge`
concurrency inflation into package load, P-core SMT, mixed P/E load and full
logical-CPU saturation. It creates one test-only diagnostic ELF and executes
five one-shot worker placements. It does not edit production Rust, install a
binary, restart a process, repeat D5 sampling or change runtime authority.

The D6 conclusion remains the predecessor fact:

```text
single traversal                 25.96501044152341 ns/edge
twenty-worker traversal          44.735012045372585 ns/edge
delta                            18.770001603849174 ns/edge
instructions factor              1.0002408799519764
inverse IPC factor               1.3751543904775503
inverse frequency factor         1.2544341315295626
D6 model residual                0.0664744185479762 ns/edge
```

D7 isolates where those IPC/frequency losses appear. It does not assume that
production Lay internally creates twenty traversal workers.

## Pinned Inputs

```text
D6 paper SHA-256                 c23f1ffd52b08683a43984ed91f28cb3daf28f98d86e72967b67a27ba2a8567d
D6 receipt SHA-256               cc1fc1c7e74258cd7fec7eed5a113bbaeb3a4bf8ee3b269825f4cd282f5755dc
D1 decision SHA-256              80530f9f5787f846ce2cf222c1b60e3ae42887ce95a11ac153ec7271cce98baf
E1 decision SHA-256              b334c047d29b21c27923fba9b38bbf17bb642cc72c9b112add1c38d8c9b0beab
D1 test fragment SHA-256         bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665
recovered V10 source SHA-256     f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c
production prefix bytes          39,047
production prefix SHA-256        ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
package SHA-256                  cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
sidecar SHA-256                  a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
V7 denominator SHA-256           33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
schedule SHA-256                 2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
```

The D7 Rust fragment is derived from the exact D1 fragment by one sealed,
deterministic insertion after the existing test-only bytes. The full V10
production prefix must remain byte-identical. The active worktree source is not
an input to the diagnostic ELF.

## Consequence Analysis

No candidate, lattice, rank, SafetyGate, verifier, package format, cache key,
learning state or IME/daemon route changes in D7. The only CPU/RSS effect is the
bounded diagnostic process: one mapped package, one test ELF and at most twenty
pinned worker threads. Existing host processes, governors, priorities and
affinities are untouched.

Two direct alternatives are rejected before code:

1. Repeating D5 task-clock sampling cannot isolate topology and would violate
   the terminal one-shot namespace.
2. Installing a guessed six-worker cap cannot be justified because production
   twenty-worker concurrency is unproven and queueing/throughput consequences
   are not measured.

D7 is chosen because it changes only the diagnostic worker count while keeping
the exact source, package, schedule, traversal, total work and PMU context. The
experiment is removable with its controller and receipts; it creates no new
runtime owner or fallback.

Future packages, machines and scheduling environments may move the frontier.
The result is authoritative only for the pinned target, bytes and loaded-host
envelope. A production policy still requires a separate consumer/concurrency
contract and its own latency proof.

## One Build And Route Graph

```text
BUILD
PARITY
W1
W6
W12
W14
W20
TERMINAL-AUDIT
```

No route may be added after implementation. Markers are created only after the
paper, structural review, implementation preflight, exact remote topology and
remote namespace absence pass. Every marker is atomically renamed from
`available` to `consumed-before-exec` before Cargo or subject/perf execution.
Failure retains the consumed marker and all evidence; no marker is recreated
and no route is retried.

## Exact Placements

The target topology is frozen as six P physical cores with sibling pairs and
eight singleton E cores:

```text
P sibling pairs                  (0,1) (2,3) (4,5) (6,7) (8,9) (10,11)
E singleton CPUs                 12 13 14 15 16 17 18 19

W1                               [0]
W6                               [0,2,4,6,8,10]
W12                              [0,1,2,3,4,5,6,7,8,9,10,11]
W14                              [0,2,4,6,8,10,12,13,14,15,16,17,18,19]
W20                              [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19]
```

`W1 -> W6` exposes package/all-P-core loading, `W6 -> W12` exposes P-core SMT,
`W6 -> W14` adds E cores without P siblings, and `W14 -> W20` adds the six P
siblings under full physical-core saturation.

## Frozen Work

Every route executes one warmup plus twenty measured rounds over the same 382
queries. The measured denominator is always:

```text
examined edges per round          25,145,756
measured rounds                   20
measured examined edges           502,915,120
component records                 7,640 x 118 bytes
```

Contiguous chunks use the existing D1 rule generalized to the route worker
count:

```text
chunk_size  = ceil(382 / workers)
chunk_start = worker * chunk_size
chunk_end   = min(chunk_start + chunk_size, 382)
```

This reproduces the exact historical W20 chunks of twenty queries with two in
the final chunk. No query is duplicated or omitted. Workers use barriers for
the warmup and each measured round, exact singleton affinity and zero migration
deltas.

## Physical Measurement

Component clocks remain enabled exactly as in D1:

```text
wall clock                       monotonic Instant
thread CPU                       CLOCK_THREAD_CPUTIME_ID
phase order                      oracle lanes eqmask traversal merge certificate
```

One inherited process-scoped `perf stat` wraps each route. FIFO control enables
events after warmup and disables them after the twentieth measured round:

```text
perf stat --json-output --no-big-num --delay=-1
events                           instructions,cycles,branches,branch-misses,task-clock
control                          subject-ready -> controller-enabled
                                 subject-done -> controller-disabled
```

Core-only routes require counted `cpu_core` rows and no counted `cpu_atom` rows.
Mixed routes require exactly one counted core and one counted atom row for each
hardware event. `task-clock` requires one complete software-event row. Any
unsupported, missing, scaled, ambiguous or duplicate row is
`BLOCKED_CAPABILITY`.

## Hard Gates

Each route requires:

```text
queries / rounds / records         382 / 20 / 7,640
errors / unresolved                0 / 0
structural mismatch                0
worker affinity                    exact singleton CPU
worker migration delta             0
thermal throttle drift             0
PMU event rows                      exact and unscaled
```

Cross-route validity requires:

```text
abs(W1 - 25.96501044152341) / baseline       <= 5%
abs(W20 - 44.735012045372585) / baseline     <= 5%
```

The result publishes for every route: traversal CPU/edge, total measured wall,
aggregate edge throughput, instructions/edge, cycles/edge, IPC, effective
frequency, inflation from W1 and thermal/affinity evidence.

## Decision Rule

D7 publishes two separate frontiers:

```text
latency-preserving capacity       largest worker point <= 5% above W1 CPU/edge
throughput point                  maximum measured aggregate edges/second
Pareto points                     no other point has both lower CPU/edge and
                                  higher aggregate throughput
```

These frontiers are observations, not an automatic production setting. If no
multiworker point preserves W1 within 5%, D7 establishes that the full
`+18.77` cannot be removed by placement alone. If a point does, a separate
runtime-concurrency paper must identify an actual producer and include queueing
latency before any code edit.

## Verdicts

```text
D7_WORKER_TOPOLOGY_SWEEP_COMPLETE
BLOCKED_PROVENANCE
BLOCKED_BUILD
BLOCKED_PARITY
BLOCKED_CAPABILITY
BLOCKED_MEASUREMENT
BLOCKED_THERMAL
```

No verdict grants production edit, integration, install, restart, deployment,
new executor, SWAR, package change, D5 retry or runtime-authority change.

