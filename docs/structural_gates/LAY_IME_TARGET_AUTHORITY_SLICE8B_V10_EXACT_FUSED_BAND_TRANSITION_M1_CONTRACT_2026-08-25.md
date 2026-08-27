# V10 Exact Fused Band Transition M1 Contract

Date: 2026-08-25

## Decision Input

The exact V10 traversal is semantically preserved and deterministically performs:

```text
edges / transition calls       25,145,756
band cells evaluated          173,652,383
minimum cells scanned         173,652,383
pruned after transition        17,086,732  (67.95%)
B5 instructions/request        42.379 M
```

This admits a transition microproof. It does not admit a new executor, V12,
runtime integration or another C1 latency run.

## Frozen Subject

```text
V10 source SHA-256             f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c
production prefix bytes        39,047
production prefix SHA-256      ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
schedule records               382
radius                         exact 0..3, product route currently 3
query contract                 0..96 symbols
```

All implementation is test-only after the exact production prefix. The active
V11 source, installed Lay and running processes are immutable inputs.

## Compared Variants

```text
G0  exact V10 generic
    array band + loop + value() bounds/range checks
    direct query-symbol comparisons
    separate minimum() scan

G1  equality-isolated generic
    same array recurrence and separate minimum() scan
    equality bits supplied from a query-local representation

U1  exact fused radius-3 scalar
    packed u64 V10 state
    compiler-visible fixed cells, no recurrence loop
    returns packed next state and minimum/dead in one pass
    equality bits identical to G1

S1  optional SWAR/bit-sliced candidate
    same packed state, equality bits and fused output contract as U1
```

`S1` may be omitted. It may not be added after looking at U1 results unless a
new paper revision freezes it first. G0 -> G1 measures equality handling;
G1 -> U1 measures packing, unrolling and minimum fusion without crediting the
same equality change twice.

## Exact Transition Contract

Input:

```text
packed or unpacked exact previous V10 band state
query-local equality representation
edge symbol / frozen alphabet identity
radius
```

Output:

```text
exact seven V10 cells
exact depth/start/len
exact minimum(row)
exact terminal_distance(row)
exact survive/dead decision
```

Packing must preserve the existing approximately 45-bit state in one `u64`.
The `[u64; 2]` representation is permitted only for query position masks up to
96 symbols; it is not a second DP state.

## One Trace, Two Phases

Parity phase:

1. Traverse the frozen 382 schedule with G0 as the sole path authority.
2. Feed every examined edge to G0, G1, U1 and pre-registered S1 in lockstep.
3. Compare all output cells, metadata, minimum, terminal distance and pruning.
4. Require the existing terminal/peak/completeness/work/certificate parity.
5. Add deterministic stress for lengths `23..96`, radii `0..3`, boundary
   starts/lengths and all equality-bit patterns reachable by an exact row.

Physical phase:

1. Materialize the G0-authoritative ordered transition inputs once before PMU
   attachment; no candidate may generate a different event order.
2. Replay the same trace in separate fixed variant processes from one sealed
   executable identity.
3. Attach process-scoped PMU only after trace readiness and before a fixed GO
   signal; trace construction is outside the counted window.
4. Record primitive checksum and transition count in the hot loop. JSON, SHA,
   parity comparison and receipt construction occur after it.
5. Foreign Nando, btop, K1 and other normal host work remain untouched.

## Authority Metrics

```text
instructions / transition     primary physical metric
instructions / band cell      diagnostic
cycles / transition           diagnostic under loaded host
wall / transition             diagnostic under loaded host
branches and branch misses    diagnostic
trace bytes / transition      diagnostic
```

No clean-host admission is required. Background load cannot block this M1
route, because retired subject instructions and exact parity are authoritative;
cycles and wall time are not promotion denominators.

## M1 Verdict

```text
M1_PASS
  parity mismatches                         0
  unresolved                                0
  U1 instructions/transition < G0
  projected whole-query instruction saving >= 15%

M1_REJECT_FUSED
  any parity mismatch
  or projected saving < 15%

M1_BLOCKED_PROVENANCE
  frozen identity or one-trace contract violated

M1_BLOCKED_CAPABILITY
  process-scoped instructions cannot be counted
```

Projection is fixed before measurement:

```text
delta/query =
  (G0 instructions/transition - U1 instructions/transition)
  * 65,826.58638743455 transitions/query

projected saving = delta/query / 42,378,604.08638743
```

The projection is a paper admission estimate, not a latency prediction. Even
`M1_PASS` authorizes only a separate full-executor candidate contract and
preflight. It does not authorize V12, deployment, runtime promotion or a claim
that the `3 ms / 5 ms` latency gates pass.

## Forbidden Effects

```text
third loaded C1 run
clean C1 marker consumption
full B
production V10 edit
active V11 edit
installed Lay mutation or process restart
foreign process stop, tuning, affinity or priority change
adaptive rerun or post-result threshold change
V12 implementation
```

Decision input:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_PMU_DECISION_2026-08-25.json`
