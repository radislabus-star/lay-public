# L1.1 Phase 8: Sound Posting-Bound Search

Status: completed proof experiment; soundness PASS in tested scope; feasibility FAIL
Date: 2026-08-14
Owner: proof-only `L1PostingBoundSearch` under `L1PeakSearch`

## 1. Scope

Phase 8 replaces early heuristic birth with a proof-only exact posting search
over the unchanged V8 package. It does not change scoring, crystallize a corpus,
write package bytes, enter the daemon, or authorize an edit.

The phase owns four questions:

1. Can a posting range be skipped without losing any center that can reach or
   tie the retained activation boundary?
2. Which physical directory gives the tightest bound per byte and per decoded
   relation?
3. Can posting events and Phase 7 typed terminal events be merged without a
   second candidate owner?
4. Is the measured work compatible with the later `<=5 ms` hot p99 contract?

Phase 8 does not claim final nonlinear settlement parity. Phase 9 owns the
shared accumulator, Phase 10 owns settlement, and Phase 11 owns the final
completeness certificate and fault injection.

## 2. Immutable Inputs

```text
V8 package
  SHA-256 47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9
  bytes   190,139,182
  centers 852,582
  atoms   218,763
  forward relations 110,928,005

query
  -> normalized typed atom field
  -> all resolved non-anchor atoms
  -> observed atom weight, channel and position

Phase 7
  -> exact typed terminal events for all 13 fixed damage classes
  -> no generated candidate strings
```

Heldout targets, expected surfaces, class labels, and test names are not search
inputs. They are joined only after a complete result exists.

## 3. Exact Activation Algebra

For query atom `q_i` and its posting relation to center `w`, define:

```text
c_i(w) = strength_i(w)
       * weight_i
       * (256 - abs(observed_position_i - expected_position_i(w)))

M_Q(w) = sum_i c_i(w)
```

This is byte-for-byte the current forward activation mass. Missing relations
contribute zero. Surface and keyboard hit counts are separate additive fields.

For a terminal interval `N = [lo, hi)`, define one per-atom envelope:

```text
U_i(Q, N) = max { c_i(w) | w in N and posting_i contains w }, default 0

U_M(Q, N) = sum_i U_i(Q, N)
```

### Posting-bound theorem

For every `w in N`:

```text
M_Q(w) = sum_i c_i(w)
       <= sum_i max_{x in N} c_i(x)
       = U_M(Q, N)
```

Therefore `U_M` is a sound upper bound even when the maxima for different atoms
belong to different centers. It may be loose, but it cannot understate a center.

The hit envelopes use the same rule: each query posting that intersects `N`
contributes at most one hit to a center. Surface and keyboard channels are
counted independently.

## 4. Position Envelope

Observed lexical positions are quantized to the existing 16 position buckets.
For each physical posting group, retain maximum strength for every expected
position bucket. A query-conditioned contribution bound is:

```text
max over expected bucket b:
  max_strength[b]
  * query_weight
  * max_position_coherence(query_position, b)
```

`max_position_coherence` uses the nearest possible byte position inside the
bucket, not its center. This deliberately overestimates boundary buckets. The
proof adapter also computes the exact maximum from contained relations and
requires:

```text
predicted contribution upper >= exact contained maximum
```

for every tested query position, weight, posting group, and grouping size.

## 5. Search

```text
complete observed atom postings
  -> posting descriptors
  -> root WordCenter interval
  -> max-priority interval queue ordered by U_M
  -> split highest unresolved interval
  -> exactly evaluate leaf centers
  -> maintain exact K-th activation mass beta_M
  -> prune only when U_M < beta_M
  -> expand equality
  -> emit all centers with M_Q >= beta_M

Phase 7 typed terminal events
  -> union by WordCenterId
  -> cannot be erased by posting rank
```

The search result is the exact posting activation closure plus the complete
typed terminal set. It is evidence for later shared accumulation, not a final
`Winner | Tied | ABSTAIN` readout.

No queue capacity, relation budget, top-k atom selection, or answer-dependent
cutoff is permitted. Empty and zero-mass cases remain exact: if the K-th mass is
zero, equality forces retention of every zero-mass center and the result is
reported infeasible rather than silently truncated.

## 6. Physical Alternatives

All alternatives are derived read-only from current V8 bytes in Phase 8.

### A. 32-relation in-memory block directory

```text
one descriptor per current codec block
best tightness
largest startup and RSS projection
no package change
```

### B. Package-neutral terminal-range grouping

```text
reorder/group only during a future deterministic repack
no new semantic field
cannot be selected before Phase 13 round-trip proof
```

### C. 128/256/512-relation skip directory

```text
first/last terminal
relation span
16 expected-position strength maxima
compact enough to compare package and RAM projections
```

The accepted granularity must be measured. `512` is included because a packed
descriptor may fit the remaining package headroom, but it is not preferred if
its p99 bound is too loose.

### D. Decoder breadth-first reindex

```text
improves typed child traversal locality
does not by itself bound posting activation
measured separately from posting alternatives
```

Phase 8 may reject all four alternatives. It may not tune scoring or reduce
completeness to make an index appear feasible.

## 7. Proof Layers

### 8A. Descriptor soundness

Exhaustive tiny postings and sampled/full V8 blocks prove:

```text
first/last terminal coverage exact
relation count exact
position-strength envelope violations 0
malformed range acceptance 0
schedule-dependent descriptors 0
```

### 8B. Exact posting closure

For exhaustive tiny packages and a fixed full-package matrix:

```text
optimized terminal mass == dense posting accumulator mass
optimized top-K plus equality == dense top-K plus equality
upper-bound violations 0
closure misses 0
tie-boundary losses 0
```

### 8C. Typed union

```text
posting event order x typed event order
  -> byte-identical WordCenterId evidence map

typed-only terminal remains present
posting-only terminal remains present
same terminal merges evidence once
```

### 8D. Feasibility

Report, without promotion:

```text
relations total / decoded / skipped
groups total / expanded / pruned
intervals generated / expanded / pruned
WordCenterIds exactly evaluated / retained
typed events generated / merged
certificate success / unresolved
p50 / p95 / p99 by language, length, frequency, class and position
directory build wall / CPU / RSS
directory RAM bytes
projected package bytes
```

## 8. Gates

### Gate A: correctness

```text
upper-bound violations          0
dense posting closure misses    0
false completeness certificates 0
tie-boundary losses             0
typed terminal losses           0
schedule parity                 100%
package SHA change              no
production reachability         no
```

### Gate B: feasibility

```text
projected hot p99               <=5 ms
projected package               <=195 MiB
RSS                             within Phase 0 ceiling
full-center p99 scan            no
```

Gate A may pass while Gate B fails. That result rejects the physical index but
preserves the theorem, oracle, matrix, and negative receipt.

## 9. Failure Semantics

```text
one bound understatement
  -> reject the descriptor revision

one closure or equality miss
  -> reject the search revision

loose but sound p99
  -> reject the physical alternative

corrupt descriptor or package mismatch
  -> Unresolved, never Certified

proof command missing from the built binary
  -> stop before execution

unknown CLI flag or fallback to training
  -> hard failure; no default action is permitted
```

Every proof command must be an explicit parser branch that exits nonzero on an
unknown flag. The Phase 7D accidental legacy training invocation is retained as
the negative reason for this contract.

## 10. Ownership

```text
proof CLI
  -> L1PostingBoundSearch             one execution owner
       -> posting descriptor adapter  evidence only
       -> interval branch-and-bound   completion owner
       -> L1TypedEditTraversal        typed evidence only
       -> deterministic evidence map  output
  -> dense posting accumulator        independent proof owner
  -> Phase 8 receipt                  observer

production runtime -X-> L1PostingBoundSearch during Phase 8
proof owner       -X-> edit authority
posting adapter   -X-> final ranking or fallback
typed traversal   -X-> erase posting evidence
```

## 11. Promotion Boundary

Phase 8 completion admits only Phase 9 shared-accumulator work. It does not
admit package-format changes, daemon shadow workers, live routing, installation,
or an owner flip. Those remain gated by Phases 10-14.

## 12. Measured Configuration

All heavy work ran on the remote proof host. The installed local Lay, daemon,
IBus route, and package remained unchanged.

```text
corpus              852,582 centers
package             unchanged V8, 190,139,182 bytes
package SHA-256      47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9
atoms                218,763
forward relations    110,928,005
closure K            128 plus all equality ties
latency matrix       13 classes x 1 case
terminal shards      bounded sweep 1, 2, 4, 8, 16, 32
remote workers       16 requested for proof search
runtime authority    unchanged
package format       unchanged
```

The `13 x 1` matrix is an architecture feasibility screen, not the accepted
`13 x 20,000` quality proof. A design already above `5 ms` on this bounded
screen is rejected before the expensive fixed proof.

## 13. Interval Search Rejection

The original sum-of-max interval branch-and-bound is sound but too loose.

| Relations/group | Upper violations | Tie losses | Centers evaluated/query | Maximum latency |
|---:|---:|---:|---:|---:|
| 32 | 0 | 0 | 300k-852k | 83-1,141 ms across sweep |
| 128 | 0 | 0 | 300k-852k | within the same rejected range |
| 256 | 0 | 0 | 300k-852k | within the same rejected range |
| 512 | 0 | 0 | 300k-852k | within the same rejected range |

The implementation was removed after rejection. The theorem, descriptor
builder, dense oracle, tiny descriptor tests, and receipts remain as negative
evidence. No scoring coefficient or equality rule was changed.

## 14. Exact WAND Ablations

Every row below used `13 x 1`, retained equality at the K-th mass, matched the
dense closure, and produced zero upper-bound violations and zero tie losses.

| Revision | Maximum latency | Verdict |
|---|---:|---|
| initial exact WAND plus per-query bound verification | 163.332 ms | reject |
| move complete envelope verification to index build | 152.282 ms | reject |
| cache current posting terminal | 133.560 ms | reject |
| insertion-order cursor repair | 167.457 ms | reject regression |
| changed-prefix merge scheduler | 119.673 ms | retain mechanism |
| galloping posting seek | 34.792 ms | retain mechanism |
| rare-posting warm seed | 38.373 ms | reject regression |
| 16 terminal shards, global envelopes | 15.607 ms | best basic WAND |
| 16 shards plus 512-relation descriptors | 14.692 ms | reject byte/latency trade |

The 512-relation directory projects to `9,969,702` bytes. Package plus
directory would be `200,108,884` bytes (`190.84 MiB`), within the `195 MiB`
ceiling, but the latency gain is too small to justify a format change.

## 15. Terminal-Shard Sweep

| Shards | Maximum latency | Scheduler iterations | Posting seeks |
|---:|---:|---:|---:|
| 1 | 33.503 ms | 237,754 | 7,087,185 |
| 2 | 26.884 ms | 309,499 | 8,179,037 |
| 4 | 21.783 ms | 606,304 | 11,377,047 |
| 8 | 16.329 ms | 803,319 | 13,089,832 |
| 16 | 14.287 ms | 1,114,694 | 14,902,819 |
| 32 | 15.619 ms | 1,562,431 | 16,716,432 |

Sixteen shards are the measured optimum in this bounded sweep. Increasing the
shard count reduces wall time only until cursor scheduling and seeks dominate.

## 16. Typed Evidence Integration

Phase 7D typed events now enter one deterministic evidence map after the exact
posting closure. Each `WordCenterId` carries a source mask:

```text
posting-only  1,574
typed-only      175
dual-source     113
merged total  1,862 across 13 cases
```

Posting-first and typed-first schedules produced byte-identical maps for all
13 cases. Typed target losses, merged target losses, closure misses, and tie
losses were all zero.

An exact typed-terminal beta seed was separately tested and rejected. Only one
of 13 cases had at least 128 typed terminals. It reduced aggregate scheduler
iterations `1,114,694 -> 1,058,125`, seeks `14,902,819 -> 14,731,341`, and
decoded relations `6,298,185 -> 6,054,211`, but the complete maximum latency was
`17.244 ms`. The final proof code therefore keeps the typed evidence union and
removes the rejected seed.

Final fail-closed v3 no-seed typed-union screen:

```text
overall proof verdict        FAIL
correctness verdict          PASS
feasibility verdict          FAIL
maximum typed traversal      2.299 ms
maximum WAND                 14.655 ms
maximum complete contour     16.310 ms
upper-bound violations       0
closure misses               0
tie-boundary losses          0
typed terminal losses        0
merged target losses         0
union schedule parity        100%
proof maximum RSS            1,030,276 KiB
package SHA before/after     identical
```

## 17. Verdict Scope

### Measured facts

- All `110,928,005` forward relations passed complete descriptor-envelope
  verification during index build.
- Tiny exhaustive descriptor and WAND tests pass.
- Every measured full-package `13 x 1` WAND variant matches the independent
  dense posting closure, including K-th equality.
- The best complete typed-union contour is still about three times the `5 ms`
  gate.
- No package byte, runtime route, score, settlement rule, daemon, or IBus state
  changed.

### Not tested

- `13 x 20,000` WAND latency or closure parity;
- final nonlinear settlement parity;
- production hot p99 or daemon RSS;
- a package-format round trip containing a new directory;
- live authority, shadow routing, installation, or deployment.

The omitted full proof cannot reverse the feasibility rejection: the bounded
screen already fails the latency gate. It is therefore intentionally not run.

### Gate result

```text
Gate A, soundness in tested scope   PASS
Gate B, <=5 ms feasibility          FAIL
package <=195 MiB                   PASS unchanged; descriptor variant also fits
promotion to Phase 9                BLOCKED by Phase 8 stop condition
runtime authority                   unchanged
```

## 18. Evidence

```text
remote full evidence
  /home/e/build/lay-l1-exact-phase2d-evidence/phase8-2026-08-14/

local raw evidence cache
  /home/ubu/.cache/lay/l1-peak-search-phase8-2026-08-14/

repository summary receipt
  docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_8_2026-08-14/phase-8.json

final fail-closed typed-union proof
  wand-final-v3-s16-13x1.json
  SHA-256 387cc6caa75b82195fff496721f14b80c7c5a49af6f262464214f17a8b625986

rejected typed-seed proof
  wand-typed-seed-s16-13x1.json
  SHA-256 a12643bbfe3f129b8a5a43fcf6b293ff819aa26c54a6b48c38f71b42dada2117
```

Phase 8 is complete as a rejected physical architecture. The next legal action
is a new, separately preflighted feasibility design. It must reduce posting
seek/scheduler work structurally; Phase 9 cannot proceed on this executor.
