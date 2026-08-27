# Slice 8B V10 Structural Work A1

Date: 2026-08-25

## Decision

Loaded latency failed twice and the loaded PMU route measured about `42.38M`
instructions per query, but the sealed subject exposed only expanded product
states. A1 supplies the missing deterministic work denominator before any V12
or fused-executor implementation.

Nando, btop, K1 and other foreign work remain running. They are neither an
admission blocker nor a presumed cause. A1 contains no latency or PMU
acceptance claim, so a quiet host is not required.

## Frozen Inputs

```text
host                       e-MEGA-MINI-M1-13th
recovered V10 source       f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c
production prefix          lines 1..1148, 39,047 bytes
production prefix SHA      ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
V13 package SHA            cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
V10 sidecar SHA            a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
V7 denominator SHA         33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
schedule SHA               2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
queries                    382, exact frozen order
radius                     exact V10 Phase7D lane radius
```

The new source may replace only the historical `#[cfg(test)]` module. Bytes
`0..39,047` must remain byte-identical. The active V11 source, installed Lay,
package, sidecar, schedule and every prior receipt are immutable inputs.

## Exact Observer

The test-only observer reproduces `search_typed_peaks()` with the same:

```text
Phase7dCertificateOracle::new
retrieval_lanes
V13 state and edge decode
BandedLevenshteinRow transition
rank_prefix arithmetic
DFS stack schedule
terminal merge/sort/dedup
certificate_keys materialization
```

For every query it records primitive structural counts:

```text
retrieval lanes
expanded states and state decodes
edge ranges
edges examined and edge decodes
rank-delta additions
transition calls
band cells evaluated
query-symbol equality comparisons
band value lookups
minimum cells scanned
surviving and pruned edges
stack pushes and pops
terminal-state checks
terminal-distance calls
terminal refs pre/post dedup
surface decodes and certificate calls
emitted certified peaks
allocator alloc/realloc/dealloc calls and requested bytes
maximum live requested bytes on the measured thread
```

Allocator counts are test-process observations from a test-only global wrapper
around `System`, enabled only for the current observer thread and only from
oracle construction through peak materialization. They are not an installed
runtime allocator proof. Traversal vector capacity-growth events are also
recorded separately so allocator totals are not misread as stack-only traffic.

## Admission And Sequence

```text
paper contract
  -> NANDA structural PASS, authority_ready=false
  -> named implementation preflight READY_TO_IMPLEMENT
  -> implement controller and test-only observer, UNRUN
  -> controller self-check
  -> ONE guarded remote release test build
  -> seal executable identity
  -> ONE structural run on the loaded host, NO perf
  -> per-query exact parity against production search, 382/382
  -> deterministic aggregate and per-query receipt
  -> immutable remote and local publication
  -> owning architecture document update
  -> STOP
```

The build and run rights are one-shot and consumed before Cargo or subject
execution. Failure retains the consumed marker and requires a new paper route;
no adaptive retry is allowed.

## Acceptance

Structural evidence is accepted only when:

```text
records                                      382
terminal mismatch                              0
peak mismatch                                  0
completeness mismatch                          0
expanded-state mismatch                        0
scratch mismatch                               0
errors / unresolved                            0 / 0
target form retained                         382/382
target lemma retained                        382/382
false certificates                             0
maximum product states                    35,590
maximum scratch bytes                       6,656
all additive aggregate identities             PASS
production prefix byte identity                PASS
```

No performance threshold applies to this instrumented run.

## Claim Boundary

A1 may report exact structural totals, distributions and derived ratios using
the already sealed PMU instruction count, including instructions per examined
edge, transition and evaluated band cell. It may reject a proposed optimization
whose theoretical zero-cost removal cannot close a frozen latency budget.

A1 cannot prove latency, clean-host behavior, PMU attribution, causal ownership
by Nando or the scheduler, allocator behavior of installed Lay, end-to-end IME
latency, V12 correctness, V12 admission, runtime integration or deployment.

The only successful terminal verdict is
`STRUCTURAL_WORK_OBSERVED_NO_PROMOTION`. Runtime authority remains unchanged.
