# L1.1 Phase 8E: Exact Sharded Epoch Accumulator

Status: implemented and rejected; correctness PASS, feasibility FAIL
Date: 2026-08-14
Owner: proof-only `ShardedEpochAccumulator` under `L1PeakSearch`

## 1. Problem

Phase 8 proved exact WAND sound on the tested full-package matrix but rejected
it at `16.310 ms` against the `<=5 ms` gate. Its dominant work is cursor
scheduling, not relation arithmetic:

```text
13 queries
  scheduler iterations  1,114,694
  posting seeks         14,902,819
  decoded relations      6,298,185
```

The next design must remove scheduler and seek amplification. It may not reduce
the observed atom field, truncate postings, change activation algebra, prune
equality, or use heldout answers.

## 2. Hypothesis

Sequential relation accumulation can be faster than exact WAND even when it
decodes more relations, because it performs one predictable update per relation
and no per-candidate multi-cursor scheduling.

```text
complete query postings
  -> disjoint terminal shards
  -> sequential relation updates
  -> persistent epoch arrays
  -> touched WordCenterIds only
  -> exact K-th mass plus equality
  -> typed evidence union
```

This is not the current legacy birth route. Legacy uses the same useful epoch
scratch shape only after `select_birth_atoms` and `birth_posting_budget` have
discarded part of the field. Phase 8E consumes every resolved non-anchor query
atom and every relation in its complete posting.

## 3. Exactness

For query postings `P_i`, retain the Phase 8 contribution unchanged:

```text
c_i(w) = strength_i(w)
       * weight_i
       * (256 - abs(observed_position_i - expected_position_i(w)))

M_Q(w) = sum_i c_i(w)
```

All factors are non-negative. A center has positive activation only if at least
one complete query posting contains it.

### Touched-center lemma

After every relation in every complete query posting has been accumulated:

```text
M_Q(w) > 0  implies  w is in touched
```

Therefore, if the exact K-th mass among touched centers is positive, every
untouched center has mass zero and cannot reach or tie the retained boundary.
Selecting all touched centers with `M_Q(w) >= beta_K` is identical to selecting
over all `WordCenterId`s.

### Zero-mass branch

If fewer than K positive centers exist, or `beta_K == 0`, every untouched zero
center ties the boundary. The proof must emit all terminal IDs or return an
explicit infeasible zero-mass result. It may never silently return only touched
IDs.

### Shard lemma

Partition the terminal domain into disjoint half-open intervals. Each relation
belongs to exactly one interval by `peer_id`. Because addition is associative
for the bounded non-overflowing activation domain and no center is shared
between shards, parallel shard accumulation is byte-identical to one sequential
accumulator.

The proof still compares all activation fields, not mass alone:

```text
mass
hits
surface_hits
keyboard_hits
```

## 4. State Layout

Each shard owns:

```text
terminal interval [low, high)
ForwardActivation[high-low]
u32 epoch[high-low]
Vec<u32> touched
current epoch
```

At query start, increment the epoch and clear only `touched`. On first access to
a center in the current epoch, reset that activation and append its ID. On epoch
wrap, clear the epoch array once and continue at epoch 1.

For `852,582` centers, the fixed arrays project to approximately:

```text
ForwardActivation  16 bytes x 852,582 = 13.01 MiB
epoch               4 bytes x 852,582 =  3.25 MiB
touched capacity    4 bytes x 852,582 =  3.25 MiB worst case
total fixed/worst                         19.51 MiB plus Vec overhead
```

This is a runtime projection, not an accepted RSS measurement.

## 5. Query Algorithm

```text
normalized query
  -> all resolved lexical atoms
  -> complete checked posting slices
  -> for each terminal shard in parallel
       for each posting
         two binary partition points for shard range
         sequentially accumulate every contained relation
  -> concatenate touched activations
  -> exact K-th mass
  -> preserve every equality tie
  -> Phase 7D typed terminal evidence
  -> one source-mask union by WordCenterId
```

There is no queue capacity, posting budget, atom top-k, answer-dependent cutoff,
literal word rule, or generated candidate string.

## 6. Ownership

```text
proof CLI
  -> Phase8E proof orchestrator
       -> complete posting adapter       producer
       -> ShardedEpochAccumulator        posting producer
       -> L1TypedEditTraversal           typed producer
       -> terminal evidence union        one evidence owner
       -> exact K-th readout              one rank owner
  -> dense complete accumulator          independent proof owner
  -> parity and feasibility receipt      observer

production runtime -X-> Phase8E during proof
proof result       -X-> edit authority
legacy birth route -X-> Phase8E proof result
```

## 7. Metrics

Report separately:

```text
query posting count
relations total / decoded
partition-point operations
touched centers
positive centers
zero-mass full scans
epoch wraps
accumulation us
K-th/equality readout us
typed traversal us
evidence union us
complete contour us
closure parity and field parity
package SHA before/after
process RSS and wall/CPU
```

## 8. Gates

### Gate A: correctness

```text
dense activation field mismatches   0
closure misses                      0
tie-boundary losses                 0
zero-mass semantic losses           0
typed terminal losses               0
union schedule parity               100%
package SHA change                  no
runtime authority change            no
```

### Gate B: feasibility screen

```text
13 x 1 maximum complete contour     <=5 ms
package                             <=195 MiB unchanged
fixed accumulator projection        <=32 MiB
full-center scan in positive p99     no
```

If the bounded screen passes, run a larger fixed matrix before Phase 9. If it
fails, preserve the receipt and reject this executor. Do not tune scoring or
reduce completeness.

## 9. Failure Semantics

```text
invalid posting range
  -> proof error, no partial result

epoch mismatch or stale activation
  -> parity failure, reject implementation

beta_K == 0
  -> exact all-terminal equality result or explicit infeasible result

one field mismatch
  -> Gate A FAIL

correct but >5 ms
  -> Gate B FAIL, no Phase 9 promotion
```

## 10. Test Plan

1. Tiny package parity against the dense oracle for every shard count from 1 to
   terminal count.
2. Repeated queries prove epoch reset and no stale activation revival.
3. Forced epoch wrap proves full epoch-array reset.
4. Fewer-than-K and zero-mass queries prove full equality semantics.
5. Posting-order and shard-order permutations prove byte-identical closure.
6. Full-package `13 x 1` screen reports all work and latency dimensions.

## 11. Promotion Boundary

Phase 8E may replace only the rejected Phase 8 proof executor after both gates
pass. It does not enter daemon, IBus, service, package format, or edit authority.
Phase 9 remains blocked until an exact Phase 8 executor satisfies `<=5 ms`.

## 12. Measured Result

The implementation added a fail-closed `--posting-search epoch|wand` selector,
a persistent sharded epoch accumulator, complete field parity, zero-mass
handling, and separate epoch work/timing metrics. Tiny proof coverage passed:

```text
focused tests                         12/12 PASS
shard counts                         1..=terminal_count
all activation fields                exact
repeated query stale revival         0
forced epoch-wrap losses             0
zero-mass equality losses            0
posting-order losses                 0
```

The fixed full-package `13 x 1` screen used `852,582` centers,
`110,928,005` complete forward relations, `K=128`, and 16 remote workers.

| Shards | Correctness | Maximum complete contour | Accumulation max | Readout max |
|---:|---|---:|---:|---:|
| 1 | PASS | 51.037 ms | 34.734 ms | 15.235 ms |
| 2 | PASS | 49.117 ms | 33.343 ms | 15.573 ms |
| 4 | PASS | 33.287 ms | 17.158 ms | 17.429 ms |
| 8 | PASS | 28.338 ms | 12.452 ms | 14.158 ms |
| 16 | PASS | 25.484 ms | 8.555 ms | 16.150 ms |

Every tested query touched all `852,582` centers. Per query, exact epoch
accumulation decoded between `5,446,385` and `9,544,967` relations. This is the
shared mechanism behind the failure: terminal-order complete accumulation
cannot avoid either the multi-million relation pass or the all-center readout.

Measured gates for the best 16-shard run:

```text
overall verdict                       FAIL
correctness verdict                   PASS
feasibility verdict                   FAIL
dense activation field mismatches        0
closure / equality losses                0
complete relation decode                yes
zero-mass full scans                       0
accumulator resident bytes       21,245,944
accumulator limit bytes          33,554,432
package bytes                   190,139,182
package limit bytes             204,472,320
runtime authority changed              false
package bytes changed                  false
```

The remote host had an unrelated service consuming about one core during the
sweep. That cannot explain a `5x` gate miss, and the measured `8.555 ms`
accumulation plus `16.150 ms` readout independently exceed the remaining search
budget. No larger proof is justified.

## 13. Final Verdict

Phase 8E is rejected as a physical executor. Its exactness theorem, state-reset
tests, metrics, selector, and negative receipts remain useful proof evidence.
It is not promoted to Phase 9 and is not reachable from production.

The next legal Phase 8 design must avoid terminal-order full accumulation. The
admitted Phase 8F hypothesis is an impact-ordered threshold search:

```text
(strength, expected-position) impact cells
-> descending query-conditioned cell heads
-> exact residual threshold
-> bounded uncertain center closure
-> exact reverse replay only for that closure
```

Raw evidence:

```text
/home/ubu/.cache/lay/l1-peak-search-phase8e-2026-08-14/

best raw receipt
  epoch-s16-13x1.json
  SHA-256 c243cdae7105defe45efa0d6418686ffecd461280801bcdde033c139d81c5d25
```
