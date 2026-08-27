# Slice 8B V10 E1 W1 DAFSA Typed View M3 V1

Date: 2026-08-27

```text
task_id        slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-v1-20260827
transaction_id 03ae2d28e6c943c4f20aad58dc4160550314e14bff057ecc4fd60d97c69e35de
```

## Question

Can the validated V13 DAFSA be materialized once as typed state and edge arrays
so the W1 hot loop no longer repeats byte-slice decoding and recoverable-error
plumbing, while preserving the exact candidate language, product frontier,
rank order and every output?

This is the only mechanism tested by M3. It is a constant-factor DAFSA decode
experiment, not a frontier-reduction experiment.

## Immutable History

M2R1 rejected both fused-minimum candidates after a complete, valid one-shot
experiment:

```text
baseline B                       26.023155 ns/edge
M1 guarded G                     27.343825 ns/edge
interleaved I                    27.362606 ns/edge
verdict                          W1_FUSED_MINIMUM_MECHANISM_REJECTED
terminal receipt SHA-256         98660957aeb31eb17b332868212cbb3ca295f35b979b511ed093fe807e0ea469
retry                            forbidden
next action                      new DAFSA-decode paper only
```

No M2 or M2R1 marker, build or scientific route may be reused. M3 requires a
fresh namespace, source fragment, build and marker ledger.

The authoritative W1 comparison point remains the one-worker CPU-0 envelope:

```text
D7 W1 traversal                  25.923669775527927 ns/edge
D7 W1 instructions               361.20658023962375 / edge
D7 W1 cycles                     103.44625099483647 / edge
D7 W1 effective frequency        3.791 GHz
D7 terminal SHA-256              db8f8fbb2ab0bbf6ba45ca9b4d2ce7c394c3de826d82961ce938adea79024f3e
```

## Existing Decode Evidence

The sealed sidecar is locally available with its frozen identity:

```text
path                              /home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f/artifacts/LAY-L2-RU-FULL-v13.dafsa
SHA-256                           a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
bytes                             3,689,884
format                            LAYV13D2 / version 2
header / state / edge bytes       256 / 12 / 12
states / edges / forms            81,128 / 226,341 / 1,875,032
terminal states                   13,083
alphabet symbols                  34
root state                        81,127
maximum static degree             32
```

The byte view currently decodes every visited state through four `read_u*`
operations and every examined edge through three `read_u32` operations. Each
operation constructs a checked byte subslice and returns through `Result` even
though the complete sidecar and all state/edge references were already
validated during load.

The static state-degree census shows that this is a broad hot path, not one
exceptional high-degree state:

```text
degree <= 1                       47.434% states / 17.001% edges
degree <= 3                       73.457% states / 38.848% edges
degree <= 5                       85.544% states / 58.329% edges
degree <= 8                       96.160% states / 83.925% edges
static mean                       2.789925 edges/state
```

Sealed A2 structural evidence for the 382-query schedule reports:

```text
expanded states                  8,059,788 total / 21,098.921 per request
examined edges                  25,145,756 total / 65,826.586 per request
dynamic aggregate fanout         3.119903 edges/state
surviving / pruned edges          8,059,024 / 17,086,732
band cells                     173,652,383 / 454,587.390 per request
physical model SHA-256            4b507c2d836c3ae1933492a90fe8b4b262f22b24b2abf1513ef7d947b5189570
```

Per-query work is broad: the top 10% of requests own only `13.1067%` of
examined edges. M3 therefore does not target an outlier list or query-specific
branch.

D4 attributed `16.7380%` of traversal task-clock samples to
`DAFSA_DECODE_MEMORY`, but that result belongs to a different D2 ELF. It is
mechanism-ordering evidence only, not a predicted M3 gain or an address map for
the M3 build.

## Frozen Mechanism

M3 has exactly two machine owners:

```text
B BYTE_VIEW_BASELINE
T PREVALIDATED_TYPED_VIEW
```

`B` is the exact current `V13DafsaView` path.

`T` is constructed before warmup from the already validated byte view:

```text
states: Box<[M3PackedState]>
edges:  Box<[M3PackedEdge]>

M3PackedState = first_edge:u32 + suffix_count:u32 + edge_count:u16 + flags:u16
M3PackedEdge  = symbol:u32 + target:u32 + rank_delta:u32
```

Required construction invariants:

```text
size_of(M3PackedState)            12
size_of(M3PackedEdge)             12
state count                       81,128
edge count                        226,341
typed payload bytes               3,689,628
root / identity / symbol digest   exact baseline values
every typed state                 field-equal to byte decode
every typed edge                  field-equal to byte decode
```

The timed `T` traversal uses direct safe typed indexing and contiguous edge
slices. No `unsafe`, unchecked indexing, sidecar-format change, symbol remap,
new cache, query-specific table or runtime fallback is admitted.

## Explicit Non-Changes

M3 must preserve exactly:

```text
V10 production prefix and source bytes
package / sidecar / V7 denominator / schedule
radius and query-length limits
DP recurrence and minimum computation
product-state stack and traversal order
edge order and rank-prefix arithmetic
terminal collection and certificate construction
expanded / examined / surviving / pruned work
candidate set and all downstream outputs
```

M3 does not add suffix-depth bounds, subtree pruning, radix edges, grouped
transition masks, a new candidate index, a DAFSA format revision, SWAR, affinity
policy or any production authority.

## Build And Route Graph

One test-only source is assembled from the exact recovered V10 prefix plus one
M3 fragment. One fresh symbolized build owns all candidates.

The complete executable graph is:

```text
BUILD
PARITY
B0-BYTE-VIEW
T0-TYPED-VIEW
T1-TYPED-VIEW
B1-BYTE-VIEW
```

No other Cargo, subject, perf or candidate route is reachable. The one-shot
ledger contains the same six route names. Every marker is consumed by atomic
rename before its external action, and no failure grants a retry.

## Semantic Parity

Parity runs before physical routes over the full frozen 382-query forward and
reverse schedules. Hard equality includes:

```text
decoded state and edge fields
terminal predicates
packed DP state and minimum
survive / prune decision
stack and edge order
rank and terminal references
candidate peaks and certificates
errors / unresolved
all structural counters
maximum states and scratch
```

Any mismatch is terminal `BLOCKED_PARITY`.

## Physical Envelope

Every physical route uses the exact one-worker envelope:

```text
worker                            1
CPU                               0
warmup rounds                     1
measured rounds                   20
queries per round                 382
examined edges per round          25,145,756
measured examined edges           502,915,120
component records                 7,640
thread migration delta            0
```

One inherited process-scoped `perf stat` is enabled after warmup and disabled
after the twentieth measured round:

```text
events                            instructions,cycles,branches,branch-misses,task-clock
output                            JSON, no scaling, no big numbers
control                           FIFO enable / disable handshake
perf record                       forbidden
```

Typed-view construction time and bytes are recorded outside the measured
region. They are evidence for a later lifetime/RSS decision, not part of the
W1 traversal denominator.

## Validity And Decision

Every route requires exact semantic/structural closure, complete unscaled
`cpu_core` rows, one complete `task-clock` row, CPU `[0]`, zero migrations and
zero thermal throttle drift.

Pair spread is computed independently for traversal CPU/edge, cycles/edge and
instructions/edge:

```text
pair spread <= 2%                  required for B and T
```

Baseline validity requires:

```text
abs(B traversal - D7 W1) / D7 W1  <= 5%
abs(B instructions - D7 W1)       <= 1%
```

Candidate values are:

```text
CPU gain          = (B traversal - T traversal) / B traversal
cycle gain        = (B cycles - T cycles) / B cycles
instruction delta = (T instructions - B instructions) / B instructions
frequency delta   = abs(T frequency - B frequency) / B frequency
```

`T` passes only when:

```text
CPU gain                            >= 5%
cycle gain                          >= 5%
instruction delta                   <= 1%
frequency delta                     <= 3%
all semantic and validity gates     PASS
```

This is a deterministic engineering gate for the pinned host and workload, not
a population-level statistical claim.

## Verdicts

```text
W1_DAFSA_TYPED_VIEW_PASS
W1_DAFSA_TYPED_VIEW_REJECTED
BLOCKED_PROVENANCE
BLOCKED_BUILD
BLOCKED_PARITY
BLOCKED_CAPABILITY
BLOCKED_MEASUREMENT
BLOCKED_THERMAL
BLOCKED_PERTURBATION
```

Failure priority is provenance, build/parity as applicable, thermal,
capability, measurement completeness, then perturbation. Incomplete or
ambiguous observations are `BLOCKED_PROVENANCE`.

## Authority Boundary

Positive M3 permits only a separate source-lifetime/RSS decision paper for the
typed representation. It does not permit a production edit, install, restart,
deployment or claim that the diagnostic executor is production authority.

Rejected M3 closes byte-decode materialization as a primary W1 lever. Only then
may a new architectural-frontier paper ask whether an exact precomputed
admissibility index can reduce the `65,826.586` examined edges/request without
changing the candidate set. M3 itself cannot implement or test that index.

## Structural Closure

The aggregate 11-route worksheet returned size-only `WATCH` and was not used as
authority. The packet was split without raising limits. Evidence/mechanism,
execution/decision, authority-boundary and the explicit all-routes claim gate
all passed with no weak triad, conflict, evidence gap or foreign pull:

```text
Route A receipt SHA-256            7add9d3dd5645ecc9bf1ba9c37f3b8a07835c160296869e19a0387f798137ee5
Route B receipt SHA-256            f30a6bf5d7af71f95bf6fee7888b8e2f797ed4b6e175317a6eb2c6e255e9f3bb
Route C receipt SHA-256            46e2d136b622b519d4567b5f439a9cf5d733e7527d23769d867240d790c87f43
all-routes receipt SHA-256         72b7789d5c7175a2ae8d26c5bb1bea0b9af31ff0c29f30af73c7d89575c174db
structural status                  STRUCTURALLY_ACCEPTED_WITH_SPLIT
implementation authority           absent pending implementation preflight
```
