# V10 Exact Fused Executor E1 Contract

Date: 2026-08-25

## Admission

M1 passed exact transition parity and reduced the frozen replay from `596.362`
to `477.598 instructions/transition`. Its projected whole-query saving is
`18.448%`, above the pre-registered `15%` gate. M1 therefore admits this
full-executor candidate paper. It does not itself admit implementation,
production source changes, V12, runtime integration or deployment.

Authoritative M1 decision:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_2026-08-25/M1_DECISION.json`

SHA-256:
`f75bdc6995bcdc8553b267ae43e511321bb34fe9d4d9acb14a610104356573a1`.

## Frozen Inputs

```text
V10 source SHA-256             f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c
production prefix bytes        39,047
production prefix SHA-256      ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
M1 ELF SHA-256                 a8fb59fb3745d5b60bf455957b0c1da200a6419b2f65ceee02a4558bf03c1e89
V13 package SHA-256            cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
V10 sidecar SHA-256            a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
V7 denominator SHA-256         33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
schedule SHA-256               2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
schedule records               382
radius                         exact 0..3
query contract                 0..96 symbols
```

One new source-preserving test-only executable may be proposed only after a
separate implementation preflight passes. Source bytes `1..39,047` remain
byte-identical. The V10 sidecar format and bytes remain unchanged.

## Compared Executors

```text
E0  exact V10 authority
    exact search_typed_peaks path
    existing V13DafsaView state/edge decode
    generic BandedLevenshteinRow
    direct query comparisons
    separate minimum scan

E1  exact fused candidate
    same Phase7D oracle and retrieval lanes
    same V13DafsaView bytes and state/edge decode
    same edge order, rank_delta, stack order and budget check points
    query-local dense EqMask[alphabet][2]
    exact edge symbol -> validated dense alphabet ID
    one packed u64 band state in each product node
    exact U1 fixed-cell transition returning next state + minimum/dead
    direct terminal distance from the packed state
    same terminal merge/sort/dedup and Phase7D certificates
```

E1 must pay for alphabet validation, EqMask construction, per-edge alphabet
lookup, equality-window extraction and packed stack traffic in the measured
full executor. M1's pre-materialized equality windows cannot be reused as a
runtime shortcut.

No packed DAFSA redesign, DLA table, candidate preselection, target-specific
condition, sidecar rewrite or second DP-state representation is permitted.

## Exact Equality Contract

The dense alphabet is derived from the complete symbol set validated by the
sealed V10 sidecar. Every sidecar edge symbol must map to exactly one stable ID;
unknown or duplicate symbols are errors. Each retrieval lane prepares one
query-local position mask per alphabet ID:

```text
positions 0..63       first u64
positions 64..95      second u64
unused bits           zero
```

For each transition, the exact low-seven equality window must correspond to
the V10 columns selected by `depth/start/len`. Boundary starts at zero and the
64-bit crossing are explicit parity cases. The packed DP state remains one
`u64`; `[u64; 2]` is only the query position mask.

## Phase P: Full Semantic Parity

E0 is the sole language and order authority. E1 runs on the exact 382 schedule
and must preserve:

```text
retrieved terminal refs and order-independent set
typed peaks and certificate keys
completeness / unresolved reason
expanded product states
examined, surviving and pruned edges
rank prefixes and terminal ranks
target form and target lemma retention
false certificates = 0
maximum product states <= 35,590
maximum scratch <= frozen product budget
```

Reverse-schedule parity must also remain exact. Candidate scratch bytes may be
lower than E0 because the product node changes representation; they need not be
byte-equal, but the scratch budget and accounting formula remain authoritative.

The deterministic transition stress remains `23..96` symbols and radii `0..3`
with zero transition and packed-state mismatches. Add focused full-executor
fixtures for equality windows crossing position 64, unknown-symbol rejection,
terminal distance at both band edges and stack-budget accounting. Fixture text
must not appear in candidate runtime conditions.

Any semantic, work-count, rank or certificate mismatch is terminal rejection
before physical or latency claims.

## Phase I: Full Executor Instructions

Package, sidecar, denominator and schedule load before PMU attachment. E0 and
E1 run in separate processes from one sealed executable. Each process executes
the same 382 queries once after one complete unmeasured warmup.

The counted window includes, per query:

```text
retrieval-lane consumption
EqMask preparation for E1
all state and edge decode
all transitions and pruning
stack push/pop
terminal merge/sort/dedup
certificate materialization
```

JSON, SHA, parity comparison and receipt serialization remain outside the PMU
window. Process-scoped retired instructions are authoritative. Cycles, IPC,
branches, misses and wall time are diagnostic under the intentionally loaded
host.

```text
actual saving =
  (E0 instructions/query - E1 instructions/query)
  / E0 instructions/query

model realization =
  actual instruction delta/query
  / 7,817,831.651832459
```

The physical model conjunct is:

```text
E1 instructions/query < E0 instructions/query
actual whole-query saving >= 15%
```

No post-result event substitution, rerun or threshold change is permitted.

## Phase L: Loaded Candidate Latency

Latency runs after parity and Phase I even if the instruction conjunct misses,
so the product result cannot be hidden by the model. Only E1 is timed; a third
loaded E0/V10 C1 replication remains forbidden.

The installed host workload remains untouched. Before and after every process,
record topology, affinity, load, PSI, temperature and throttle counters. Normal
Nando, btop and K1 activity is neither blocked nor controlled. Active thermal
throttle drift makes the process `BLOCKED_THERMAL`, not a latency result.

Use the frozen C1 process sequence and denominators:

```text
S1 -> T1 -> T2 -> S2 -> S3 -> T3 -> T4 -> S4 -> S5 -> T5

S process:  one complete 382-query warmup, then 100 schedule rounds
T process:  one 20-worker fixed-shard warmup burst, then 250 barriered rounds
workers:    0..18 receive 20 fixed queries, worker 19 receives 2
barriers:   START and END on every T round
```

Authoritative samples are E1's internal `search_elapsed_us` and
`total_elapsed_us`, timed at the same semantic boundaries as exact V10.
External call wall time is diagnostic only. Measured loops record primitive
samples into preallocated buffers; no JSON, SHA, certificate comparison or
receipt construction occurs between requests.

Nearest-rank p99 is `ceil(n * 99 / 100) - 1`.

## Decision

```text
E1_PROMOTION_CANDIDATE
  Phase P exact parity                         PASS
  errors / unresolved                         0 / 0
  actual whole-query instruction saving       >= 15%
  S pooled search p99                          <= 3 ms
  S pooled total p99                           <= 5 ms
  every S-run search / total p99               <= 3 / 5 ms
  T pooled total p99                           <= 5 ms
  every T-run pooled total p99                 <= 5 ms
  max(run x worker total p99)                  <= 5 ms

E1_REJECT
  parity passes but any physical or latency conjunct fails

BLOCKED_PROVENANCE
  any frozen identity or one-executable contract fails

BLOCKED_CAPABILITY
  process-scoped instructions cannot be counted

BLOCKED_THERMAL
  active throttle drift invalidates a latency process
```

Worst-query p99, T search p99, maximum latency, client spread, cycles and model
realization are mandatory diagnostics but are not added as hard conjuncts after
the result.

`E1_PROMOTION_CANDIDATE` authorizes only a separate production-integration
paper and preflight. It is not deployment authority and does not by itself
admit V12, installed Lay mutation or runtime promotion.

## One-Shot Sequence

```text
paper structural PASS, authority_ready=false
  -> implementation preflight READY_TO_IMPLEMENT
  -> one guarded source-preserving build
  -> sealed executable identity
  -> Phase P parity
  -> E0 PMU
  -> E1 PMU
  -> loaded E1 latency sequence
  -> immutable decision publication
  -> STOP
```

Every executable route consumes its marker before execution. Build or route
failure is terminal; any retry requires a new paper correction and disjoint
state. Partial evidence is retained, not rewritten.

## Forbidden Effects

```text
production prefix or V10 sidecar mutation
active V11 source mutation
installed Lay mutation or process restart
foreign process stop, affinity, priority or policy change
third loaded E0/V10 C1 run
clean C1 marker consumption
full B or formal B promotion
S1/SWAR addition
adaptive rerun or threshold change
V12 implementation or admission
runtime integration, deployment or authority change
```
