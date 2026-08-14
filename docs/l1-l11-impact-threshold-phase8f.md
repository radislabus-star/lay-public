# L1.1 Phase 8F: Impact-Ordered Threshold Search

Status: V1 replay correctness rejected; V2 intrinsic executor rejected
Date: 2026-08-14
Owner: proof-only `ImpactThresholdSearch` under `L1PeakSearch`

## 1. First Shared Failure

Phase 8 WAND is exact but spends `14.655 ms` in terminal-order cursor
scheduling. Phase 8E removes seeks and scheduling, but every fixed query still
touches all `852,582` centers and decodes `5.45-9.54 million` relations. Its
best complete contour is `25.484 ms`.

The common defect is physical order:

```text
terminal-ordered postings
-> learn the strongest centers only after traversing a broad terminal domain
```

The next design must discover high contributions in score order while retaining
an exact bound over everything not read. It may not select a subset of atoms,
truncate a queue, cap posting events, prune equality, or use heldout answers.

## 2. Exact Contribution Cells

For query atom `q_i` and relation `r=(w,s,p)`, retain the frozen activation:

```text
c_i(w) = s * q_i.weight * (256 - abs(q_i.position - p))
M_Q(w) = sum_i c_i(w)
```

For one atom posting, all relations with the same `(strength, position_mode)`
have the same query-conditioned contribution. Define one static impact cell:

```text
ImpactCell(atom_id, strength, position_mode, sorted WordCenterIds)
```

At query time, cells of each resolved atom are ordered by their exact
contribution. No relation is dropped. Equal-contribution cells use a stable
structural order and all equality is consumed before the head moves below that
contribution.

The first screen may derive cells from the immutable V8 postings outside the
timed search. That preparation time and projected representation bytes are
reported separately. Excluding preparation can prove or reject only the search
executor intrinsic. It cannot by itself pass the final hot or package gate.

## 3. Residual Threshold Theorem

Let `h_i` be the exact contribution of the first unread cell in posting `i`, or
zero when that posting is exhausted. Because each posting is read in descending
exact contribution order, every unread relation in posting `i` contributes at
most `h_i`.

For a center never observed by sorted access:

```text
M_Q(w) <= T_unseen = sum_i h_i
```

For an observed center, let `L(w)` be the exact sum of consumed relations and
let `Seen(w)` identify postings in which its relation has been consumed:

```text
M_Q(w) <= U(w) = L(w) + sum_{i not in Seen(w)} h_i
```

These bounds may overstate absent relations, but cannot understate mass.

Let `beta_L` be the K-th largest partial lower mass among observed centers.
When `T_unseen < beta_L`, no never-observed center can reach the current K-th
lower boundary. Define the uncertain closure:

```text
C = every observed center with U(w) >= beta_L
```

Every true global top-K center and every equality tie is in `C`. Centers in `C`
are then replayed from the same complete terminal-ordered postings that produced
the impact cells. The exact K-th mass and equality over replayed centers are
therefore identical to the dense complete-posting result.

Strict inequality is mandatory. `T_unseen == beta_L` is unresolved and search
continues.

## 4. Search State

```text
per query posting
  ordered impact-cell descriptors
  current cell and terminal offset
  current exact head contribution

per touched WordCenterId
  partial ForwardActivation
  consumed-posting bitset
  epoch

global
  sum of current posting heads
  touched ids
  exact K-th lower boundary
```

The consumed-posting bitset is evidence state, not a candidate frontier. Its
width is determined by the complete resolved query posting count. No bit may be
discarded merely because its atom is common.

## 5. Scheduling

The executor processes complete impact cells. A max-heap chooses the posting
whose current head is largest. After a cell is exhausted, that posting advances
to its next exact cell and the global residual threshold is updated.

Certification checks occur after complete equality layers. A check may scan
touched centers to compute `beta_L` and `U(w)`. Check frequency is geometric
and changes work only; it cannot stop or prune without the theorem condition.

```text
complete impact cells
-> exact partial accumulation
-> strict unseen threshold check
-> conservative uncertain closure
-> exact terminal lookup replay for closure
-> exact K-th plus equality
-> Phase 7D typed terminal union
```

## 6. Distinction From Rejected MaxScore

The rejected `maxscore_global_max` experiment retained terminal-ordered posting
cursors. It required `200,397-678,229` scheduler iterations and decoded
`0.98-3.39 million` relations per query.

Phase 8F changes the physical search primitive:

```text
rejected MaxScore     terminal cursor -> pivot/seek -> exact candidate
Phase 8F              impact cell -> residual threshold -> uncertain closure
```

Reusing its name or metrics as Phase 8F evidence is forbidden.

## 7. Representation Screen

The proof must measure at least:

```text
impact cells per atom and globally
cell headers
terminal-id payload bytes
delta-varint bytes within each cell
zstd shard bytes for a deterministic impact-ordered prototype
query descriptor bytes
query-local preparation time
```

The first intrinsic screen may keep V8 unchanged. If intrinsic search passes,
one full-field deterministic encoder projection is required before Phase 8 can
pass. A package projection above `195 MiB` rejects the representation even when
search is fast.

The proof may compare exact `(strength,position)` cells with coarser cells, but
a coarse cell must retain a conservative contribution maximum and must consume
the whole cell before lowering its head.

## 8. Exact Posting Replay

V1 assumed that `activation_for_terminal()` was an exact inverse of the complete
forward field. The focused tiny-package proof rejected that assumption. Forward
compilation emits one lexical relation per `(atom, center)` with an averaged
position. Existing reverse compilation emits one relation per observed atom
occurrence and then truncates the lexical tail to 96 relations. Reverse replay
can therefore both duplicate and omit complete forward evidence.

V2 replays one closure center by binary-searching its terminal ID in every
resolved complete terminal-ordered posting. A hit contributes the exact stored
`strength` and `position_mode` through the frozen forward algebra. This route
reads relation tuples, not dense activation values, expected targets, or
heldout labels. Replay parity requires all four fields:

```text
mass
hits
surface_hits
keyboard_hits
```

The first intrinsic screen may retain references to the existing complete V8
postings outside the timed preparation phase. Gate B1 must account for the
physical terminal-lookup lane together with impact-order payload; B0 cannot
promote a representation that has no bounded physical realization.

## 9. Metrics

Report separately:

```text
cell preparation us and bytes
query postings and cells
sorted cells consumed
relation events consumed
unique centers touched
threshold checks
unseen threshold at certification
K-th lower boundary
uncertain closure size
terminal posting probes
exact posting replay hits
exact posting replay us
impact search us
typed traversal us
evidence union us
complete contour us
dense field / closure parity
package SHA before / after
```

Do not combine intrinsic search time with excluded preparation and call the sum
hot latency. Both numbers remain visible.

## 10. Gates

### Gate A: correctness

```text
impact-cell relation omissions          0
impact order violations                 0
residual upper-bound violations         0
dense activation closure misses         0
tie-boundary losses                     0
exact posting replay field mismatches   0
typed terminal losses                   0
union schedule parity                  100%
package SHA change                      no
runtime authority change                no
```

### Gate B0: intrinsic feasibility screen

```text
13 x 1 maximum impact search + replay   <=2.5 ms
13 x 1 maximum complete contour         <=5.0 ms
full-center scan in p99                 no
uncertain closure p99                   measured, not capped
```

`2.5 ms` is a screen allocation, not a user-visible relaxed gate. Phase 7D
typed traversal already consumes up to about `2.3 ms` in the fixed screen.

### Gate B1: physical feasibility

```text
deterministic full-field representation <=195 MiB package
cold startup and steady RSS             Phase 0 limits
hot complete contour                    <=5.0 ms with preparation included
```

Gate B1 cannot pass using query-local derived cells. It requires an actual
bounded representation or startup index with measured bytes and preparation.

## 11. Failure Semantics

```text
cell relation missing or duplicated
  -> correctness FAIL

head lowered before equality cell exhausted
  -> correctness FAIL

residual threshold equals K-th lower
  -> continue, never certify

no finite certificate before full exhaustion
  -> exact result plus feasibility FAIL

exact posting replay mismatch
  -> reject replay owner, preserve diagnostic receipt

intrinsic search above budget
  -> reject Phase 8F before global repack

package projection above limit
  -> reject representation, no installed mutation
```

## 12. Ownership

```text
proof CLI
  -> Phase8F orchestrator
       -> complete posting adapter
       -> impact-cell builder          proof preparation only
       -> impact threshold search      one posting search owner
       -> exact posting replay         one exact score owner
       -> exact K-th readout            one rank owner
       -> typed evidence union          one evidence owner
  -> dense complete accumulator         independent proof owner
  -> Phase8F receipt                    observer

production runtime -X-> Phase8F during proof
proof result       -X-> edit authority
dense oracle       -X-> exact replay input
heldout target     -X-> search input
```

## 13. Test Plan

1. Tiny postings: every impact cell contains every relation exactly once.
2. Every observed position `0..=255`: cell ordering is non-increasing.
3. Random posting schedules: residual bounds never understate dense mass.
4. Equality fixtures: search cannot stop on equal threshold.
5. Missing relations: absent posting components remain safely overbounded.
6. Repeated queries and epoch wrap: no stale partial mass or bitset.
7. Tiny package: closure and all activation fields match dense for every K.
8. Exact posting replay: closure activations match complete forward activations.
9. Full package `13 x 1`: exactness and intrinsic feasibility receipt.
10. Only after B0 PASS: full-field deterministic size projection.

## 14. Promotion Boundary

Phase 8F remains proof-only. Gate A and B0 PASS admit the physical
representation screen, not Phase 9 or deployment. Phase 9 remains blocked until
Gate B1 passes with preparation included and the package/runtime budgets are
measured.

## 15. V1 Reverse Replay Result

Focused command:

```bash
scripts/cargo-guard.sh test --lib posting_bounds \
  --features lexical-compiler -j16
```

Measured on the 12-center fixture:

```text
tests passed before replay parity assertions    16
tests failed                                     2
dense K-th mass                           3,611,552
reverse-replayed K-th mass                3,906,680
second dense K-th mass                      856,118
second reverse-replayed K-th mass          1,097,598
```

The higher K-th mass cannot be caused by closure pruning: the K-th value of a
subset cannot exceed the K-th value of the complete set when both use identical
scores. Source inspection located the first differing mechanism in compiler
ownership:

```text
forward: stats_by_atom -> one averaged relation
reverse: each clean atom occurrence -> duplicate lexical relations
reverse: truncate anchors + 96 lexical relations
```

Verdict: V1 `reverse_exact_replay` is rejected. Threshold search, equality
handling, and residual bounds are not rejected by this failure. The installed
package SHA remained
`47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9`;
runtime authority and package format did not change.

## 16. V2 Exact Posting Replay Result

V2 replaced the invalid reverse owner with exact binary lookup over the same
complete forward postings. The focused suite passed `18/18`; the fixed full
package `13 x 1` proof then separated correctness from feasibility:

```text
correctness verdict                         PASS
closure parity                             13/13
upper-bound violations                         0
activation field mismatches                    0
tie-boundary losses                            0
package SHA change                             no

impact relation events               1,009,567-4,159,131
unique centers touched                 346,554-643,970
uncertain closure                       40,281-446,368
exact replay posting probes          6,485,241-61,306,659
impact accumulation max                    74.595 ms
closure scan max                           37.366 ms
exact replay max                        2,028.051 ms
search plus replay max                  2,128.770 ms
complete contour max                    2,130.512 ms
```

The intrinsic limit is `2.5 ms` and the complete limit is `5.0 ms`; this is a
systemic rejection, not measurement noise. The first shared mechanism is the
unfactored common mode:

```text
one equality cell may contain all 852,582 centers
-> millions of score-identical activations are materialized
-> touched-center residual bounds remain loose
-> tens or hundreds of thousands of centers require exact replay
```

V2 is therefore rejected before physical repacking. The next legal experiment
must remove score-identical common mass algebraically before traversal; merely
optimizing binary search or widening a frontier cannot close an `851x` intrinsic
gap.
