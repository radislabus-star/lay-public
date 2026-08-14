# L1.1 Phase 8G: Modal Residual Field

Status: projection complete; exact algebra PASS; head-certificate family REJECTED
Date: 2026-08-14
Owner: proof-only `ModalResidualProjection` under `L1PeakSearch`

## 1. Why Phase 8F Failed

Phase 8F proved exact impact ordering but rejected the executor by three orders
of magnitude:

```text
correctness                              PASS
impact events consumed       1,009,567-4,159,131
centers touched                346,554-643,970
uncertain closure               40,281-446,368
exact replay probes          6,485,241-61,306,659
search plus replay max             2,128.770 ms
intrinsic limit                         2.500 ms
```

The first shared mechanism is not binary-search speed. Some exact impact cells
contain all `852,582` centers. Enumerating a score-identical contribution cannot
change ranking, but Phase 8F materializes it for every center and then carries a
loose residual bound into exact replay.

Phase 8G removes this common mode algebraically before any search. It does not
select atoms, truncate relations, cap candidates, or approximate scores.

## 2. Static Atom States

For atom `i`, every center has exactly one static state relevant to the frozen
forward activation algebra:

```text
state_i(w) = absent
           | relation(strength, position_mode)
```

The complete forward posting already proves this partition because terminal IDs
are strictly increasing and each `(atom, center)` relation occurs at most once.
`phase_relation` and relation flags are not part of this state because the
frozen Phase 8 activation uses only `strength` and `position_mode`; changing that
activation algebra invalidates the projection contract.
For `N` centers, count every relation cell and the implicit absent state:

```text
count(absent) = N - posting_degree
count(s, p)   = number of relations in exact cell (s, p)
```

Choose one deterministic modal state:

```text
d_i = arg max_state (count(state), deterministic_state_order)
```

Ties prefer `absent`, then higher strength, then lower position. This order is
independent of the query, damage class, expected target, and heldout label. Tie
order changes representation only; it cannot change reconstructed activations.

## 3. Rank-Preservation Theorem

The frozen output is not only scalar mass. For query atom `q_i`, define the
complete activation vector:

```text
A_i(w,Q) = (mass_i, hits_i, surface_hits_i, keyboard_hits_i)

A_i(absent,Q)   = (0, 0, 0, 0)
A_i(relation,Q) = (exact_contribution(q_i, relation),
                    1,
                    0 if keyboard channel else 1,
                    1 if keyboard channel else 0)
```

Let `D_i(Q)=A_i(d_i,Q)` and use signed intermediate coordinates:

```text
R_i(w,Q) = A_i(w,Q) - D_i(Q)
D(Q)     = sum_i D_i(Q)
A_Q(w)   = D(Q) + sum_i R_i(w,Q)
```

All four residual coordinates are signed. Reconstruction must occur in a wide
signed type and only then be range-checked and converted to `ForwardActivation`.
Underflow, overflow, or a negative reconstructed coordinate is a correctness
failure.

Let `B(Q)=D(Q).mass` and `delta_i(w,Q)=R_i(w,Q).mass`. `D(Q)` is identical for
every center. Therefore for all centers `a,b`:

```text
M_Q(a) > M_Q(b)  iff  sum delta_i(a,Q) > sum delta_i(b,Q)
M_Q(a) = M_Q(b)  iff  sum delta_i(a,Q) = sum delta_i(b,Q)
```

The complete top-K, its K-th mass, every equality tie, and all four activation
fields are preserved exactly. Removing the modal component is coordinate
translation, not pruning.

## 4. Residual Representation Bound

Store the modal state once per atom and one terminal/state record for every
non-modal center. Let `m_i=count(d_i)` and `R_i=N-m_i`.

If `absent` is modal, `R_i=posting_degree`. If a relation cell is modal, its
count is at least the absent count, so:

```text
R_i = N - m_i <= N - count(absent) = posting_degree
```

Thus modal factorization never increases relation-event count:

```text
sum_i R_i <= complete_forward_relations
```

The physical size can still grow through headers or duplicated indices, so this
event theorem is not a package-size proof. The first projection reports those
bytes separately and does not claim a V9 format.

## 5. Signed Residual Field

Mass residuals may be positive or negative:

```text
delta_i > 0   stronger than the modal state
delta_i = 0   modal state, no stored event
delta_i < 0   weaker or absent relative to the modal state
```

The three hit residuals may also be negative when a relation state is modal and
the center is absent. Negative residuals are evidence, not noise. They must
remain available for exact activation reconstruction. No implementation may
discard them because they do not expose a positive mass peak.

For an upper bound define:

```text
positive_i(w) = max(delta_i(w), 0)
negative_i(w) = min(delta_i(w), 0)

sum delta_i(w) <= sum positive_i(w)
```

This permits positive mass-residual cells to discover possible peaks while exact
readout includes the complete signed activation vector.

## 6. Projection Before Executor

Phase 8G-0 adds metrics only. It may derive modal states from complete immutable
V8 postings outside any hot timing. It must not implement a search owner yet.

For every fixed query report:

```text
query postings
original relation events
modal absent postings
modal relation-cell postings
modal baseline B(Q)
total residual events
positive residual events
negative residual events
zero-mass residual events
largest residual state
largest positive residual state cell
event reduction ratio
terminal/state payload projection
modal header projection
oracle-greedy positive cells and events consumed
oracle-greedy unique centers touched
largest consumed positive equality layer
fractional head-reduction event lower bound
exact signed lookup probes and hits for oracle-touched centers
complete replacement package byte ledger
```

The proof-side dense oracle may provide the exact K-th mass only to calculate an
optimistic lower-bound screen. It cannot feed a future executor.

### Oracle-assisted head screen

For every query posting, group positive mass residuals by exact `delta`, order
the cells in descending `delta`, and let `h_i` be the first unread positive
delta, or zero after exhaustion. A center not encountered in any consumed
positive cell satisfies:

```text
mass(w) <= U_untouched = B(Q) + sum_i h_i
```

The projection receives exact `beta_dense` only from the independent dense
proof owner. Its deterministic greedy screen repeatedly consumes every current
head cell in the maximum `h_i` equality layer until:

```text
U_untouched < beta_dense
```

It reports consumed cells, events, equality layers, and unique centers. This is
oracle-assisted work for one precisely defined scheduler. It is optimistic
relative to a live executor because it does not discover `beta`, but it is not
claimed to be the minimum over every possible posting schedule.

### True lower-bound screen

For each transition from positive head `d_j` to `d_(j+1)`, define head reduction
`d_j-d_(j+1)` and cell cost equal to its complete event count. Ignore posting
precedence, allow fractional cells, and solve the resulting fractional minimum
cost needed to make `U_untouched < beta_dense`. Removing precedence and
indivisibility can only reduce required work, so this cost is a valid lower bound
for every executor using this head certificate.

```text
fractional lower bound > budget
  -> reject this certificate family

fractional lower bound <= budget < oracle-greedy work
  -> scheduler unresolved; no executor code

oracle-greedy work <= budget
  -> positive discovery screen may proceed to signed-readout feasibility
```

## 7. Evidence-Based Work Budget

Phase 8F measured about `15-20 ns` per consumed impact event before closure and
replay. A `2.5 ms` intrinsic budget can therefore afford at most roughly
`125,000-166,000` such events with no time left for readout.

Phase 8G-0 uses the following preregistered screens:

```text
maximum oracle-greedy positive events          <=100,000
maximum consumed positive equality layer       <=100,000
maximum fractional event lower bound             measured
maximum oracle-greedy unique centers touched     measured
maximum complete signed residual events          measured
maximum exact signed lookup probes               measured
```

`100,000` is not a runtime truncation. It is a preregistered admission threshold
for the measured scheduler. Every event is still counted by the projection. A
fractional lower bound above it rejects the head-certificate family; a greedy
result above it with a lower bound below it is `WATCH`, not a universal reject.

## 8. Candidate and Default Cohorts

Positive residual events are field deviations, not heuristic candidate births.
A future executor may name centers only after they emerge from complete residual
evidence.

Centers with no positive mass residual have score at most the modal baseline
`B(Q)`.
They are excluded only when the exact K-th score satisfies:

```text
beta_exact > B(Q)
```

If `beta_exact == B(Q)`, untouched default centers may belong to the equality
lattice. The projection must report this as unresolved; an executor may not hide
the cohort behind a top-K cap. If `beta_exact < B(Q)`, the proposed positive-only
route is invalid.

## 9. Possible Executor After Projection PASS

Phase 8G-0 can admit the simple Phase 8G-1A executor only when every query's
complete signed residual stream itself fits the event budget:

```text
resolved query atoms
-> modal baseline B(Q)
-> scan every non-modal residual record once
-> exact signed residual accumulation
-> touched-center exact readout
-> explicit default-cohort comparison
-> exact K-th plus equality
-> Phase 7D typed evidence union
```

If complete signed residual events remain above budget but the positive screen
passes, no executor is admitted yet. A possible Phase 8G-1B must separately
prove how negative and zero-mass residuals are located for touched centers. The
projection reports the naive cost
`unique_touched_centers * query_postings` and exact lookup hits so the Phase 8F
replay failure cannot be hidden. A center-local adjunct, bitmap, or second lane
must pay its full package bytes. That is a separate paper contract and cannot be
silently substituted into 8G-0.

## 10. Physical Format Boundary

The deterministic projection format replaces the existing V8 forward-posting
section; it is never added beside it. The unchanged compact V7 base remains
byte-identical. For the pinned package the measured input ledger is:

```text
V8 file bytes                         190,139,182
compact base bytes                     77,960,560
atom count                                218,763
terminal count                            852,582
forward relations                     110,928,005
atom index bytes                         3,500,208
shard index bytes                          109,392
compressed posting bytes              108,568,894
```

The projection uses exactly `32` atoms per shard and zstd level `19`, matching
the measured V8 shard topology. One projected raw atom lane is:

```text
default_state_id                       unsigned LEB128
for residual centers in terminal order:
  terminal_delta                       unsigned LEB128
  state_id                             unsigned LEB128

state_id(absent) = 0
state_id(relation(s,p)) = 1 + (s << 8) + p
```

The 16-byte atom index carries `shard_id`, raw offset, raw byte length, and
residual count; the 16-byte shard index carries compressed location and lengths.
The full projection is:

```text
128-byte outer header
+ unchanged compact base
+ 16 * atom_count
+ 16 * ceil(atom_count / 32)
+ deterministic compressed residual shards
```

Every atom, including an empty residual lane, emits its modal state. The
projection must actually encode and compress all lanes; multiplying a sampled
ratio is forbidden. It reports every term above, alignment bytes, raw bytes,
compressed bytes, and total bytes. A terminal-ordered lane supports exact lookup
without a second full reverse field, but lookup work remains a separate latency
screen.

Storing both a terminal lane and a duplicated impact lane is not admitted unless
their combined deterministic projection remains within `195 MiB`.

## 11. Gates

### Gate A: algebra and accounting

```text
state partition omissions                         0
state partition duplicates                        0
modal reconstruction mismatches                   0
signed score mismatches against dense             0
signed hit-field mismatches                        0
K-th or equality mismatches                        0
residual events > original relations               0
package SHA change                                 no
runtime authority change                           no
```

### Gate P0: projection admission

```text
fixed denominator                                  13 x 1
oracle threshold correctness                          PASS
oracle-greedy positive events max                  <=100,000
largest consumed positive equality layer max       <=100,000
beta_dense > modal baseline                            13/13
deterministic projected replacement package       <=195 MiB
```

Gate P0 alone does not permit an executor. Phase 8G-1A additionally requires
complete signed residual events `<=100,000` for every fixed query. Otherwise the
result is `DISCOVERY_PASS_READOUT_OPEN`, and Phase 8G-1B needs a new paper and
preflight. Neither result passes the latency gate.

### Gate B0: future intrinsic executor

```text
search plus exact signed readout max             <=2.5 ms
complete contour max                             <=5.0 ms
full-center fallback scans                              0
correctness losses                                      0
```

### Gate B1: physical feasibility

```text
actual package                                  <=195 MiB
cold startup and steady RSS              Phase 0 limits
hot contour including preparation                <=5 ms
```

## 12. Failure Semantics

```text
modal reconstruction differs from forward state
  -> correctness FAIL

negative residual omitted from exact score
  -> correctness FAIL

oracle beta used by executor
  -> proof leak, correctness FAIL

fractional lower bound above projection budget
  -> reject the head-certificate family before executor code

oracle-greedy work above budget but lower bound below budget
  -> scheduler WATCH, no executor code

default cohort can reach equality boundary
  -> unresolved, no bounded readout claim

projected full replacement package above 195 MiB
  -> reject physical representation

positive discovery passes but signed stream exceeds budget
  -> readout OPEN, require a separate 8G-1B contract

executor above latency budget
  -> preserve negative receipt, no runtime promotion
```

## 13. Ownership

```text
proof CLI
  -> modal projection owner
       -> complete posting state partition
       -> modal/default selector
       -> residual accounting
       -> oracle-greedy positive threshold screen
       -> fractional lower-bound screen
       -> signed-readout work projection
       -> deterministic replacement-package projection
  -> dense complete accumulator       independent proof owner
  -> Phase 8G receipt                 observer

production runtime -X-> Phase 8G during projection
dense beta         -X-> future executor input
heldout target     -X-> modal selection
projection budget -X-> relation truncation
```

## 14. Decision Tree

```text
Phase 8F V2                          REJECTED
  |
  v
Phase 8G-0 modal projection          NEXT
  |-- algebra/accounting FAIL      -> reject theorem implementation
  |-- fractional lower >100k     -> reject head-certificate family
  |-- greedy work >100k          -> scheduler WATCH
  |-- beta <= baseline            -> design explicit default-cohort owner
  |-- size >195 MiB               -> reject representation
  |-- signed residuals >100k      -> 8G-1B readout paper, no executor
  `-- all projection + scan gates PASS
        |
        v
Phase 8G-1A exact residual scan executor
  |-- correctness FAIL            -> reject implementation
  |-- latency FAIL                -> reject executor
  `-- B0 PASS
        |
        v
one deterministic physical package and B1
```

No new crystallization, package mutation, daemon restart, IBus restart, version
bump, or runtime owner change is allowed in Phase 8G-0.

## 15. Measured Result

The fixed remote `13 x 1` projection completed against the unchanged V8
package and fixed case manifest.

```text
schema                              lay.l11.posting-search-phase8-proof.v6
correctness                         PASS
final verdict                       REJECT_HEAD_CERTIFICATE
runtime authority changed           no
package changed                     no
current package                     190,139,182 B
projected replacement package       175,504,286 B
package gate                        PASS
original relations                 110,928,005
residual events                    107,517,677
modal absent atoms                     218,759
modal relation atoms                         4
projection build                        14.098 s
whole proof                              23.840 s
```

The package representation is physically feasible, but the event topology is
not:

```text
fractional event lower bound             150,315  FAIL >100,000
oracle-greedy events                    3,978,146  FAIL
largest consumed equality layer          804,493  FAIL
complete signed query residuals        6,134,639  FAIL
default cohort resolved                    13/13
oracle threshold certified                 13/13
activation mismatches                          0
K/equality mismatches                          0
upper-bound violations                         0
```

Per-class work maxima:

| Class | Query postings | Residual events | Greedy events | Fractional lower bound | Equality layer | Signed lookup probes |
|---|---:|---:|---:|---:|---:|---:|
| adjacent transposition | 166 | 4,786,735 | 2,396,001 | 9,387 | 804,493 | 75,922,922 |
| double substitution | 116 | 2,799,684 | 1,901,481 | 62,432 | 371,332 | 67,963,124 |
| extra letter | 182 | 5,093,418 | 2,823,775 | 70,201 | 787,470 | 95,357,444 |
| layout projection | 157 | 2,788,050 | 1,929,769 | 44,206 | 268,866 | 98,433,505 |
| letter substitution | 87 | 2,893,097 | 1,305,872 | 18,077 | 476,833 | 33,603,489 |
| missing letter | 161 | 4,278,664 | 1,751,276 | 4,246 | 620,663 | 56,045,066 |
| non-adjacent transposition | 141 | 6,134,639 | 3,978,146 | 92,530 | 448,024 | 89,498,199 |
| omission transposition | 85 | 3,655,343 | 1,426,641 | 39,080 | 537,565 | 36,157,895 |
| prefix truncation | 152 | 4,152,696 | 2,652,797 | 39,526 | 513,804 | 86,356,976 |
| punctuation suffix | 184 | 5,774,298 | 3,589,762 | 75,246 | 606,917 | 112,680,680 |
| repeated fragment | 96 | 2,043,010 | 1,122,252 | 18,104 | 271,789 | 31,845,408 |
| sparse multi-omission | 80 | 2,036,057 | 879,021 | 14,650 | 218,800 | 25,032,240 |
| suffix truncation | 185 | 4,658,864 | 3,372,477 | 150,315 | 374,030 | 111,861,360 |

### First shared failure mechanism

Global modal factorization is exact but almost never finds a non-absent global
mode. Only four of `218,763` atoms have a relation state as their global mode.
For the other `218,759` sparse atoms, `absent` remains modal, so the residual
lane is the original posting. Total relation events fall by only `3.075%`.

This is not an executor optimization problem. Equal positive deviations still
create an `804,493`-event layer, and a precedence-free fractional relaxation
still needs `150,315` events before exact signed readout. An implementation of
the proposed executor therefore cannot satisfy the preregistered physical
screen.

### Host load and verdict scope

The proof used `278.45 s` user CPU in `23.91 s` wall time, or an average of
`1,175%` CPU, with `1,992,760 KiB` maximum RSS. An unrelated Nando service used
about one additional core during the run. That background load cannot explain
an event lower bound above the gate or multi-million exact readout work.

Tested:

- complete package projection and byte ledger;
- complete state partition and signed activation reconstruction;
- exact K-th mass and equality parity for the fixed `13 x 1` screen;
- oracle-greedy and fractional head-certificate work;
- package immutability and runtime non-reachability.

Not tested, because projection admission failed:

- a Phase 8G executor;
- hot executor latency;
- production RSS/startup for a modal package;
- `13 x 20,000` quality proof;
- runtime, daemon, IBus, or installed-package behavior.

Evidence:

```text
docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_8G_2026-08-14/evidence/phase-8g-13x1.json
SHA-256 3644e103754ded16a062dc45b0c3180d9a07ee852a9b362c1a8831c78d5b4482
```

Phase 8G remains as a proved negative architecture point. Its exact translation
theorem may be reused, but the global-modal head certificate and executor are
closed. The next design must factor relation topology conditionally before
query execution.
