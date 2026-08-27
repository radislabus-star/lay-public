# V10 E1 Remaining-Cost D1 Contract

Date: 2026-08-25

## Question

E1 preserved the exact V10 language and reduced retired instructions per query
by `44.446%`, but failed loaded latency:

```text
single search p99       3.047 ms   > 3 ms
20-worker total p99    11.512 ms   > 5 ms
worst worker p99       38.097 ms   > 5 ms
```

D1 characterizes the remaining E1 cost. It does not implement another
executor and it cannot promote E1, admit V12, or mutate the installed runtime.

Authoritative predecessor:

```text
E1 decision SHA-256   b334c047d29b21c27923fba9b38bbf17bb642cc72c9b112add1c38d8c9b0beab
E1 verdict            E1_REJECT
```

## Frozen Identity

```text
V10 source SHA-256             f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c
production prefix bytes        39,047
production prefix SHA-256      ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
E1 fragment SHA-256            1726a22ad7bf4d9212761a9ffd61660ee46f94e3148af6d61b864d554b1d410d
E1 executable SHA-256          727ba875094d3e7121330514cefee7661ecbf8dcda076a7f10631aaa2f8cd618
V13 package SHA-256            cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
V10 sidecar SHA-256            a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
V7 denominator SHA-256         33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
schedule SHA-256               2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
schedule records               382
```

One new source-preserving test executable is allowed only after the code-route
gate and implementation preflight pass. Production bytes `1..39,047` must be
byte-identical. D1 may copy E1 test-only code and add observers, but may not
change V10 production bytes, sidecar bytes, retrieval semantics, rank order,
certificate semantics, or the packed E1 transition/executor result.

## Semantic Prerequisite

The new executable must rerun the complete E0/E1 parity route before any D1
measurement. Required result:

```text
records                                      382/382
terminal / peak / completeness mismatch      0 / 0 / 0
work / rank / reverse mismatch               0 / 0 / 0
target form / lemma retention                382 / 382
false certificates                           0
transition stress 23..96 mismatch             0
maximum product states                       <= 35,590
maximum E1 scratch                           <= 6,144 B
```

Any mismatch terminates D1 before PMU or component timing.

## Component Boundary

The instrumented observer splits one exact E1 request into these phases:

```text
oracle         Phase7dCertificateOracle::new(observed)
lanes          oracle.retrieval_lanes()
eqmask         query-local EqMask preparation, summed over lanes
traversal      packed DAFSA x edit-band traversal, summed over lanes
merge          terminal concatenate + sort_unstable + dedup
certificate    package surface decode + certificate_keys + peak materialization
```

For every phase D1 records both `CLOCK_THREAD_CPUTIME_ID` nanoseconds and
monotonic wall nanoseconds. It also records outer request CPU/wall time; the
difference between outer time and summed phase time is observer/unattributed
overhead and is never reassigned to a phase.

Structural work is collected in a separate unmeasured trace before each
component route and joined by query ordinal after the measured loop:

```text
retrieval lanes
EqMask builds
expanded states
examined / surviving / pruned edges
stack pushes / pops
terminal hits before merge
terminal refs after merge
certificate calls
materialized peaks
```

Structural counters, JSON, SHA, percentile calculation and receipt generation
are forbidden inside the component measured loop. The loop may only execute
the instrumented exact request and append a fixed-size primitive sample to a
preallocated buffer.

## Component Routes

Each process loads package, sidecar, V7 and schedule once. It performs one
complete unmeasured warmup before the measured rounds.

```text
C-SINGLE
  CPU                         0
  workers                     1
  measured rounds             20
  samples                     7,640

C-FIXED
  workers                     20
  worker i CPU                i
  measured rounds             20
  samples                     7,640
  START and END barrier       every round

C-REVERSED
  workers                     20
  worker i CPU                19 - i
  measured rounds             20
  samples                     7,640
  START and END barrier       every round
```

Worker 19 owns frozen query ordinals `380..381`. Reversed mapping deliberately
moves this heavy shard from E-core CPU 19 to P-core CPU 0 while still using all
20 logical CPUs. The comparison is the joint effect of worker-to-CPU placement,
P/E class and SMT topology; it is not a pure scheduler or pure core-class proof.

## Process-Scoped PMU

Uninstrumented exact E1 runs in separate processes for each frozen mapping and
event group. Each process uses the same 20 workers, one fixed-shard warmup burst
and 20 barriered measured rounds. Workers are created before PMU enable and
remain alive until after PMU disable.

The controller protocol is:

```text
subject-ready
perf enable + acknowledgement
controller-enabled
20 measured START/END rounds
subject-done
perf disable + acknowledgement
controller-disabled
```

Package load, worker creation, warmup, thread destruction and serialization are
outside the counted window. Process-scoped inheritance must count all workers.
Hybrid `cpu_core` and `cpu_atom` rows are summed only after each PMU has complete
coverage.

Frozen groups, each run once for FIXED and once for REVERSED:

```text
G0  instructions, cycles, branches, branch-misses
G2  L1-dcache-loads, LLC-loads, LLC-load-misses
G3  dTLB-loads, dTLB-load-misses
```

No unsupported event substitution, multiplexed result, adaptive rerun or
post-result group change is allowed. A capability miss is retained as
`BLOCKED_CAPABILITY` for that group.

## Environment

The host is intentionally measured under its ordinary concurrent workload.
Nando, btop, K1 and other foreign processes remain running and are neither a
readiness veto nor D1-owned controls. Before and after every process, record
topology, affinity, load, PSI, temperature, throttle counters and top CPU
processes. D1 may not stop, renice, re-affine or tune them.

Active hardware thermal-throttle counter drift invalidates only the affected
process as `BLOCKED_THERMAL`. High load, PSI or foreign CPU usage is evidence,
not a blocker.

## Fixed Aggregation

Nearest-rank p50/p95/p99 uses `ceil(n * p / 100) - 1`. Publish:

```text
per phase pooled wall and thread-CPU p50/p95/p99
per phase summed wall and thread-CPU share
outer wall and thread-CPU p50/p95/p99
wall minus thread-CPU scheduling/wait residual
per query component p99 and structural work
per worker component p99 for fixed and reversed mapping
query 381 and worker 19 fixed/reversed comparison
PMU counts per request and per examined edge
fixed/reversed PMU ratios
```

The ranked phase report is determined by summed thread-CPU share. Mapping
sensitivity is reported numerically; no causal label or threshold is added
after observing the result.

## Verdicts and Claim Boundary

```text
D1_OBSERVED
  new-executable parity PASS
  all three component routes complete with zero errors/unresolved
  all six PMU routes complete with valid non-multiplexed counters
  immutable comparison published

D1_OBSERVED_WITH_CAPABILITY_GAP
  parity and component routes complete, but a frozen PMU group is unsupported

D1_REJECT_PARITY
  any semantic or work mismatch

BLOCKED_PROVENANCE
  any frozen identity, source prefix or one-build contract fails

BLOCKED_THERMAL
  active throttle drift occurs during a D1 process
```

D1 proves only measured phase costs, structural denominators and fixed versus
reversed placement effects for exact E1 on the loaded target mini-PC. It cannot
prove clean-host behavior, end-to-end installed Lay latency, a unique causal
mechanism, a production design, formal B, or V12 necessity/admission.

## One-Shot Sequence

```text
paper structural PASS, authority_ready=false
  -> code-route PASS
  -> implementation preflight READY_TO_IMPLEMENT
  -> one guarded source-preserving build in a disjoint remote namespace
  -> parity
  -> C-SINGLE -> C-FIXED -> C-REVERSED
  -> FIXED-G0 -> REVERSED-G0
  -> FIXED-G2 -> REVERSED-G2
  -> FIXED-G3 -> REVERSED-G3
  -> immutable decision publication
  -> STOP
```

Every executable route consumes its own marker before execution. Failure is
terminal for that marker and cannot be retried without a new paper correction.

## Forbidden Effects

```text
production prefix, V10 sidecar or input mutation
active V11 source mutation
installed Lay mutation or process restart
foreign process stop, affinity, priority or policy change
host governor, SMT, turbo or thermal-policy tuning
third loaded E0/V10 replication
clean C1 marker use
E1 executable or marker reuse
full B
adaptive rerun or event substitution
executor optimization, V12 implementation or admission
runtime integration, deployment or authority change
```
