# Unified IME Token Field Implementation Plan

Status: Slices 0 through 4 have scoped PASS receipts. Slice 5 Attempt 3 proves
hot material reuse: the fixed eligible replay is exact and applies `20 / 20`
boundaries with zero `NotReady`. Cold first touch remains `FAIL`: `3 / 4`
warmup boundaries fall back literally. An optimized release profile now shows
that the legacy deterministic lane contributes zero candidates to both failing
canonical Boundary decisions while consuming `43.296-131.946 ms`. A subsequent
cardinality review rejected post-DecisionCore fallback: fallback must be chosen
from a typed pre-rank material-coverage receipt so that each event has exactly
one final lattice and one DecisionCore invocation. The V3 design route has a
scoped structural `PASS`; implementation and quality proof are still pending.
Installed Lay remains `1.0.33`; runtime authority, version and deployment have
not changed.

## 1. Problem Boundary

The observed `данорм -> да норм` failure is not missing Boundary candidate
birth. The candidate is present in the full correction lattice. It is lost at
the delivery boundary when the prepared Space decision is not ready inside the
existing 8 ms wait budget.

Two independent facts must remain separate:

1. the installed Productive V90 package is unavailable because its embedded
   L1.1/canonical-L2 fingerprints do not match the active packages;
2. the IME currently runs two independent background state machines, one for
   display precognition and one for the Space correction decision.

The package incompatibility must be repaired before using latency or quality
measurements to justify a runtime refactor. A Boundary-only fast return would
hide the unavailable morphology owner and is forbidden.

## 2. Current Route

```text
printable key
-> CommitText(character)
-> update committed tail and tail_epoch
-> schedule precognition worker
   -> live completion material
   -> display-only L2/L3/L4 readout
   -> publish preedit
-> schedule Space prefetch worker
   -> full deterministic + Nanda lattice
   -> canonical Productive V90 L2
   -> common L3/L4/DecisionCore
   -> store optional correction decision

Space
-> clear preedit and cancel display generation
-> wait up to 8 ms for matching Space-prefetch key
-> Ready: validate and mutate committed tail
-> NotReady: commit the literal Space without correction
```

The two workers have separate generation counters and call different public
readout APIs. They are not interchangeable:

- precognition produces display candidates and Tab feedback;
- Space prefetch produces a candidate lattice, selected action, and mutation
  proof.

## 3. Rejected Designs

### 3.1 Boundary-only early return

Rejected because it removes lexical and morphology competitors before common
ranking, can create false singleton authority, stores an incomplete cache
entry, and hides Productive V90 admission failure.

### 3.2 One serial worker for both products

Rejected because display work and correction authority have different
deadlines and feedback semantics. Serial execution creates head-of-line
blocking: a slow display projection can delay Space, while a slow correction
projection can suppress IME suggestions.

### 3.3 Two independent full-field computations

Rejected as the final architecture because simultaneous cache misses can run
the same Productive V90 field twice, increase CPU and latency tails, and leave
two unrelated cache/generation identities.

## 4. Selected Architecture

Use one immutable canonical token-field producer with two typed projections.
Keep display and correction scheduling independent, but prohibit them from
independently constructing the expensive L1.1 -> Productive V90 field.

```text
                         one canonical token identity
                                      |
                                      v
                         single-flight L2 field owner
                         L1.1 bounded lattice
                         -> Productive V90
                         -> immutable field material
                            /                    \
                           /                      \
              display projection          correction projection
              best effort                 Space-deadline priority
              no mutation authority       full candidate competition
              -> IME preedit              -> DecisionCore lease
                                                   |
Space + exact input identity ----------------------+
-> verifier
-> one committed-tail mutator
```

There is one field owner and one correction authority route. Two background
threads are acceptable only as typed consumers of the same immutable field;
thread count is not authority count.

## 5. Data Contracts

### 5.1 CanonicalTokenKey

The reusable field key must be structural, not an untyped input string:

```text
normalized context window
normalized observed token
L1.1 package SHA-256
canonical L2 package SHA-256
Productive V90 package SHA-256
field algorithm/schema version
```

Trailing pending Space is not part of this key. Productive V90's current local
scene derives left context and the observed token and does not use a trailing
boundary-after feature. Text-edit materialization remains outside the cached
field because replacement bytes differ between unfinished and Space-finalized
input.

### 5.2 PreparedCanonicalTokenField

The immutable field contains only reusable L1.1/L2 evidence:

```text
bounded L1.1 seeds and verdict
Productive V90 composite lattice
grounded and generated surface groups
morphology-slot evidence
L2 local authority: Winner | Tied | Abstain | Unavailable
package receipt and timing receipt
```

It must not contain:

```text
IBus path or focus state
visible preedit selection
Tab/decline learning state
final L3/L4 online score
AuthorizedEdit or backend mutation state
```

L3/L4 and final ranking remain request-time operations so online deltas are not
silenced by the immutable field cache.

### 5.3 InputFrameIdentity

Both projections use one exact GUI identity:

```text
engine object path
focus receipt
tail_epoch
exact committed tail
context prefix
observed token
active composition flag
active layout
correction-affecting config projection
```

Exact text is retained for comparison; a hash alone cannot authorize an edit.

### 5.4 PreparedCorrectionLease

The Space product is a one-token lease:

```text
InputFrameIdentity
selected candidate and complete candidate receipt
EditAction and TransitionProof
model/package receipt used by the computation
decision timing
```

The lease is single-consumer. It cannot survive a changed input identity,
focus change, backspace, next printable character, successful mutation, or
literal-Space fallback.

### 5.5 DisplayProjection

The display product contains only selected IME proposals and preserves the
existing candidate-decline state. Boundary and full-token typo edits remain
excluded from passive ghost text. Tab is still required to accept a displayed
replacement.

Display feedback and autocorrection feedback must remain separate:

- ignored suggestion is censored display evidence;
- Tab is explicit display acceptance;
- automatic Space application is not positive learning evidence;
- double-Shift rollback is explicit negative correction evidence.

## 6. Single-Flight Field State Machine

```text
Vacant
-> Computing(key, package_generation)
   -> Ready(key, Arc<PreparedCanonicalTokenField>)
   -> FailedTransient(key, error receipt) -> remove entry
   -> Superseded(package_generation) -> discard result

Ready
-> bounded LRU reuse
-> package reload -> invalidate generation and clear ready entries
```

Rules:

- compute outside the cache mutex;
- one producer computes a key while other background consumers wait on that
  exact in-flight value;
- infrastructure failure is never cached as lexical truth;
- a package reload cannot publish an old-generation result into the new cache;
- the cache remains bounded and stores immutable `Arc` values;
- no hot IBus event waits on field computation directly.

## 7. Event State Machine

### 7.1 Printable key

```text
1. commit the user's character;
2. update tail and publish the new tail_epoch;
3. capture one InputFrameIdentity;
4. schedule Space correction first;
5. schedule display projection as best effort;
6. return without materializing L1/L2/L3 on the IBus thread.
```

Scheduling correction first gives the 8 ms Space consumer priority without
serializing display behind it. Both background paths converge on the same
single-flight field.

### 7.2 Space with a ready lease

```text
1. capture the current InputFrameIdentity before mutation or cancellation;
2. take the exact matching correction lease;
3. hide display state without invalidating the captured lease;
4. revalidate path, focus receipt, tail_epoch, exact tail, layout and config;
5. run the existing verifier and backend authorization;
6. atomically replace the token and insert exactly one trailing Space;
7. arm visible-postcondition and double-Shift rollback state;
8. invalidate the consumed generation.
```

### 7.3 Space without a ready lease

```text
1. commit exactly the literal user Space;
2. invalidate the in-flight token generation;
3. never apply a late correction after the boundary;
4. record NotReady with stage timings.
```

This fallback preserves input responsiveness but is not defect closure. The
promotion gate requires the fixed replay and physical denominator to show no
NotReady losses for eligible hot cases.

### 7.4 Supersession and focus change

Backspace, a new printable key, layout transition, focus change, reset, or
package reload invalidates display publication and correction consumption for
the old identity. Background work may finish, but its result is discarded and
cannot emit IBus signals or mutate text.

## 8. Implementation Slices

Each slice has one kind of change. A failed slice is analyzed before another
implementation attempt.

### Slice 0: restore the measured baseline

- locate or remotely rebuild Productive V90 against the active L1.1 and V13
  package hashes;
- do not weaken fingerprint validation;
- require `--productive-l2-v1-status` to report `ready_live_owner`;
- warm the admitted runtime before latency measurement;
- record package hashes, bytes, RSS and cold/hot timing.

No runtime-route refactor starts while this slice fails.

#### 2026-08-16 measured V9-bound rebuild

Tested:

- exact staged L1.1 V9, canonical L2 V13, corpus and axis-schema identities;
- frozen-induction resume with shared-support recovery and `--workers 20`;
- deterministic package and recovery-sidecar compilation;
- package receipt, byte count, SHA-256, wall time, CPU and peak RSS capture.

Measured facts:

```text
release binary bytes                 10,996,232
release binary SHA-256               cb0395b74778246e8493e4fd9bfcb1fbb47dec8b553cf899d940ab0bf9ba5c5a
package bytes                        17,309,944
package SHA-256                      40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44
recovery bytes                        2,123,112
recovery SHA-256                     de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e
resume wall                               97.37 s
resume internal total                     97.346 s
resume CPU                                      93%
resume peak RSS                         627,440 KiB
shared-support recovery                    41.526 s
target-retained calibration groups          121 / 1,348
runtime authority changed                         false
```

The resume process exposed one existing performance limitation: shared-support
anchor recovery used one OS thread even though the command was given 20
workers. This did not change package bytes or proof scope. Parallel proof work
remains a separate stage.

The package compiler reported
`authority_blocked_by_target_loss=true` and
`PASS_shadow_suggest_only_package`. This is not a quality PASS and cannot be
used to install the package. It is only a successful compatible package build.

Not tested yet:

- the fixed `13 x 100 x 2` Productive proof;
- equality of the frozen hypothesis payload under the V9-bound package;
- installed `ready_live_owner` admission;
- daemon/IBus latency, GUI behavior or physical input.

Exact local receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/release-build.time.txt
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/resume-build-receipt.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/resume-build.time.txt
```

#### Frozen proof generation bridge

The fixed manifest is intentionally bound to the old proof generation:

```text
old Productive SHA-256               9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438
old L1.1 SHA-256                     47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9
proof spool SHA-256                  6e282474b26bf90dc61ee21c93c9dd7dd727c29a2b02650c513ffdd06746e807
proof spool bytes                    1,154,794,811
canonical L2 V13 SHA-256             cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
axis schema SHA-256                  b5b24f952e83e1e9738db0f89a9d2e9e16eaf7af754990114a562d42be3c060b
manifest entries                     1,300
manifest H                           1,280
manifest payload SHA-256             2f54844d7f7900734049d2ed2ae53150eead60da3223c0efc4256ba804b7f89b
```

The proof bridge must be an exact two-row generation table:

```text
old row: Productive 9fd8c950... + L1.1 47fa757a...
new row: Productive 40fb6a9f... + L1.1 bf5a1619...
```

Both rows require the same canonical V13, axis schema, spool identity, manifest
bytes, 1,300 case identities, `H=1,280`, oracle bindings and payload SHA-256.
Cross-pairs and unknown digests fail closed. The old row remains accepted. The
manifest is never rewritten for the V9-bound package. Any payload difference
is a proof failure and stops deployment rather than creating a new baseline.

#### Fixed proof result and deployment boundary

The first `20`-worker replay used the legacy shared replay owner. It produced
raw top-1 `272` and `2,526` base-projection failures. That receipt proves only
that the wrong proof route was selected; it is retained as non-normative
negative evidence and is not mixed with Productive semantic authority.

The corrected semantic replay on `20` workers measured:

```text
H / B / S0                            1,280 / 1,280 / 1,280
semantic / base raw top-1                 1,109 / 267
base projection failures                         0
false singleton                                  0
integrity errors                                 0
maximum class p99                           15.362 ms
```

The clean normative replay used one worker for both the old accepted V90 pair
and the V9-bound pair. All quality and safety fields are identical, including
all `2,600` probe-parity comparisons. The normalized semantic receipt identity
is `db98b950242f4564778a0d3aabbbd1148348d439c63d2bf901aa28932866ca46`
for both generations. The new maximum class p99 is `5.286 ms`; the old accepted
V90 checkpoint is `5.317 ms`.

The automatic gate remains `FAIL_measured_shadow_gates` because `5.286 ms` is
above the frozen `5.000 ms` threshold. This is not reported as a latency PASS
and does not relax the general gate. Deployment is allowed only by the existing
receipt-scoped V90 exception: the new generation is semantically byte-identical
and measured faster than the already accepted generation.

Artifact parity narrows the change further:

```text
.p2m bytes after 256-byte header     identical
.p2m payload SHA-256                 6a959bf04e5011b576c333b87cd00a0c400d5b735581a259c1af89e0fc03aeb8
.p2r bytes after 256-byte header     identical
.p2r payload SHA-256                 ad3d1c03d3a48fc81838d63644f3be51fc0d7d2405e0850f73a71ad5730a31ed
```

The `.p2m` differences are limited to the embedded L1.1 fingerprint and header
checksum. The `.p2r` differences are limited to the bound base-package SHA-256.
No morphology table, recovery program, ranking value, frontier or authority
rule changed.

Exact normative receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/baseline-v90-semantic-normative-clean-workers1-13x100.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/productive-v90-active-v9-v13-semantic-normative-clean-workers1-13x100.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_DEPLOY_DECISION_2026-08-16.json
```

#### Installed Slice 0 baseline

The proven pair was staged locally, checked by SHA-256, installed in a
fail-closed pair order, admitted by the installed `1.0.33` binary, and then
loaded by restarting only Lay-managed components. Global `ibus-daemon` remained
PID `3702`.

```text
Productive status                         ready_live_owner
installed Productive SHA-256              40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44
installed recovery SHA-256                de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e
daemon / engine package mappings          2 / 2
rollback                                  /home/ubu/.local/lib/lay/rollback/1.0.33-pre-v90-v9-20260816-064416

legacy helper cold query range            1.542-2.113 s
legacy helper hot p99 range               7.078-9.631 ms
legacy helper cached-field p99            0.799-2.690 ms
legacy helper samples                     4 x 200

daemon RSS / PSS                          402,064 / 281,279 KiB
managed engine RSS / PSS                  381,724 / 260,906 KiB
L1.1 service RSS / PSS                    183,552 / 181,497 KiB
L3 online RSS / PSS                        73,120 / 32,638 KiB
four-process PSS total                              756,319 KiB
```

The benchmark above is retained only as route-drift evidence. Source inspection
proved that `query_live_canonical_l2()` calls
`standalone_surface_field_readout_with_productive_limit()`, while both live IME
projections call `canonical_owned_text_candidates() -> Productive V90`. The
legacy timing therefore cannot prove or fail live-owner latency. A cold traced
live call spent `3.077-3.523 s` including first package admission, while the
inner Productive stages reported sub-millisecond work; a repeatable
same-process hot benchmark is required before the single-flight change.

Exact live receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_LIVE_DEPLOY_2026-08-16.json`

#### Route-bound same-process live-owner benchmark

Tested:

- a release binary built on `e@192.168.3.94` from a SHA-matched isolated
  source snapshot;
- direct repeated calls to
  `canonical_owned_text_candidates() -> live_productive_v1_readout()`;
- `200` calls per process with the outer materialized-readout cache bypassed;
- a clean morphology token, a glued token and a damaged token;
- cold package admission separately from the following `199` hot calls.

Measured facts:

```text
benchmark binary bytes                         11,008,104
benchmark binary SHA-256             aff43b74bce2334abc0de2f67d3d187ceb2c1a72869ed960361f374310223f2c

surface             cold        hot p50    hot p95    hot p99    hot max
морфология       3,296.860 ms      3.295      4.239      4.643      5.010 ms
данорм           3,221.897 ms      2.495      3.616      4.046      4.125 ms
рабоает          3,098.270 ms      5.440      6.621      8.060      8.165 ms

морфология authority                              Winner
данорм authority                                    Tied
рабоает authority                                    Tied
peak benchmark RSS                            316,096 KiB
runtime authority changed                            false
```

`рабоатют` returned `EmptyL11Lattice` after `1.944 ms` on its first call. It
is retained as target-availability evidence and is excluded from Productive
V90 hot-latency statistics because the Productive owner was not reached.

Not tested by this benchmark:

- GUI scheduling, the 8 ms Space lease deadline or physical application input;
- cache/single-flight behavior, because Slice 2 does not exist yet;
- aggregate latency over the fixed restoration corpus;
- candidate quality beyond the returned route receipt.

Verdict scope:

`ROUTE_BOUND_OWNER_PROVEN_LATENCY_MIXED_NO_AUTHORITY_CHANGE`.

The benchmark-routing defect is closed: the command names both the producer
and owner symbols and bypasses the unrelated outer cache. The general 5 ms
gate remains open because the damaged-token sample is slower. Slice 1 may
proceed because it is a parity-only extraction intended to expose and reuse the
expensive material; this measurement does not authorize deployment.

Exact receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/route-bound-morphology.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/route-bound-glued.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/route-bound-damaged-v2.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/route-bound-damaged.json
```

### Slice 1: pure Productive V90 field extraction

Files:

- `src/nanda_wave/l2_field/productive_v1/live.rs`
- `src/nanda_wave/l2_field/bridge.rs`
- `src/nanda_wave/l2_field/cache.rs`
- focused tests in the same modules

Changes:

- split Productive V90 evaluation from text replacement materialization;
- expose an immutable internal field result;
- prove old and new materialized candidate surfaces, evidence, authority and
  verdict are identical;
- make no ranking, weight, frontier or verifier change.

#### Slice 1 result

Implemented:

- `PreparedCanonicalTokenField` now owns the immutable Productive V90
  composite lattice, local authority, exact observed-token identity and
  Productive package fingerprint;
- Productive evaluation and text replacement materialization are separate;
- a token identity mismatch fails closed before candidate materialization;
- the public live readout remains a composition of those two operations.

Measured facts:

```text
focused live-module tests                         5 / 5 PASS
old/new live semantic projections                 4 / 4 PASS
candidate/authority projection mismatches                  0
release binary bytes                              11,010,024
runtime authority changed                               false
```

The four runtime comparisons cover Winner, Tied and `EmptyL11Lattice`
availability. Their normalized projections include status, availability,
authority, ordered replacements, source IDs, error classes and gate results.
The unit parity test compares the complete in-memory
`CanonicalL2FieldReadout`, including morphology evidence.

Not tested in this slice:

- the full fixed Productive proof;
- GUI event order or the Space lease deadline;
- cache concurrency or package-generation supersession;
- installed runtime behavior.

Verdict scope:

`PASS_SCOPED_IMMUTABLE_FIELD_MATERIALIZATION_PARITY`. This permits Slice 2 but
does not authorize deployment.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice1-materialization-parity.json`

### Slice 2: bounded single-flight reuse

Files:

- `src/nanda_wave/l2_field/cache.rs`
- `src/nanda_wave/l2_field/mod.rs`
- `src/nanda_wave/candidate_gate.rs` only for adapter consumption/invalidation

Changes:

- replace exact raw-text cache identity with `CanonicalTokenKey`;
- deduplicate simultaneous field preparation;
- retain L3/L4 and final readout outside the immutable cache;
- invalidate on productive/canonical/L1 package generation changes;
- add counters for producer computations, waiters, hits, transient failures and
  superseded publications.

#### Slice 2 result

Implemented:

- one bounded single-flight cache now owns immutable
  `PreparedCanonicalTokenField` values;
- the structural key binds exact scene bytes, ordered L1.1 seed-lattice SHA,
  all three package identities and the field schema version;
- the cache is bounded at `128 Ready`, `32 Computing` and `8` waiters per key;
- transient failures and panics are removed, while package reload increments a
  generation and prevents stale publication or materialization;
- text and IME projections share the field but retain separate materialization;
- Productive package SHA-256 is now computed once when the immutable mmap view
  is admitted. The rejected intermediate recomputed the full `16.5 MiB` digest
  twice before every lookup and produced `160.398 ms` hot p99.

Measured facts:

```text
format identity tests                              7 / 7 PASS
single-flight cache tests                          5 / 5 PASS
immutable live-field tests                         5 / 5 PASS
baseline/new semantic projections                  4 / 4 PASS
ordered candidate/authority mismatches                     0

case             baseline p99     Slice 2 p99     cache disposition
morphology          4.643 ms         4.287 ms      1 producer + 199 hits
glued                4.046 ms         3.083 ms      1 producer + 199 hits
damaged-v2           8.060 ms         8.187 ms      1 producer + 199 hits
empty-l11            unchanged        unchanged     no field birth

release binary                                     11,030,088 B
standalone peak RSS                            316,004-319,368 KiB
runtime authority changed                                  false
```

The `damaged-v2` route still fails the general `<=5 ms` contract. A traced hot
sample separates approximately `1.8-1.9 ms` of L1.1 socket work from about
`3.1-3.2 ms` of request-time proposal admission. Slice 2 therefore restores
the pre-split route envelope and proves one field producer; it does not claim a
general latency PASS.

The broader remote `l2_field` test filter reported `192/194`. Both failures
were live L1.1-dependent bridge fixtures while the remote test host had no
L1.1 service. A local live probe retained the sparse-omission target; the
installed L1.1 still returned `EmptyL11Lattice` for the short-participle
fixture. Neither failure is hidden or counted as cache evidence.

Verdict scope:

`PASS_SCOPED_SINGLE_FLIGHT_REUSE_GENERAL_LATENCY_OPEN`. This permits Slice 3
identity and scheduling work, but it does not authorize deployment.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice2-single-flight-reuse.json`

### Slice 3: shared GUI identity and scheduling order

Files:

- `src/bin/lay_ibus_engine/engine/types.rs`
- `src/bin/lay_ibus_engine/preedit.rs`
- `src/bin/lay_ibus_engine/precognition_worker.rs`
- `src/bin/lay_ibus_engine/space_autocorrect_prefetch.rs`
- `src/bin/lay_ibus_engine/composition_commit.rs`
- `src/bin/lay_ibus_engine/managed.rs`
- `src/bin/lay_ibus_engine/state.rs`

Changes:

- introduce one `InputFrameIdentity` constructor;
- remove the two partially overlapping identity definitions;
- schedule correction before display after a printable commit;
- preserve separate typed outputs and feedback semantics;
- make display cancellation distinct from correction-lease invalidation;
- take the Space lease before closing the word-boundary display generation.

No scoring or candidate code changes in this slice.

Measured Slice 3 result:

```text
shared InputFrameIdentity definitions          1
managed printable identity captures/event     1
Space correction schedule before display      PASS
Space lease take before display close          PASS
lease consumers                                1
identity/config/focus/tail/layout fault tests  PASS
remote lay-ibus-engine tests                   202 / 202 PASS
runtime authority changed                      false
installed version                              1.0.33
```

The first full target run was `201 / 202`: passive IME display exposed a
whitespace-bearing Boundary replacement as `->...`. The failing test did not
exercise the new identity or lease code; it exposed an existing adapter
contract violation. It was repaired separately at the operation boundary:
multi-token replacements remain in the L2/Space correction lattice but are not
projected as passive preedit candidates. No literal surface or producer
`source_id` condition was added.

What remains untested at this point is explicit: fixed immediate-Space replay,
physical GUI input, latency percentiles, aggregate quality/false-split proof,
release build and deployment.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice3-shared-gui-identity.json`

### Slice 4: observability and duplicate removal

Files:

- `src/bin/lay_ibus_engine/trace.rs`
- route-parity tests
- obsolete identity/cache code discovered by the parity proof

Required trace fields:

```text
frame generation and tail_epoch
field producer count
field cache/single-flight disposition
L1.1, V90, display L3 and correction L3 timings
Space lookup wait
lease outcome: ready | not_ready | stale | unauthorized | applied
```

Only code proven unreachable or duplicate is removed. The display worker is
not deleted merely to reduce thread count; it is deleted only if a later
measurement proves that one executor improves both deadlines without coupling
feedback or authority.

Measured Slice 4 result:

```text
trace projection kinds                         2
closed Space lease outcomes                    5
summed field producers per frame             <=1
remote lay-ibus-engine tests            205 / 205 PASS
observed-source nodes / edges              26 / 38
observed-source routes                          15
source evidence                          64 / 64 PASS
route issues / warnings                       0 / 0
runtime authority changed                    false
installed version                           1.0.33
```

The first full run was `204 / 205`: its static architecture test still required
the removed `live_completion_candidates()` wrapper. The implemented route had
one `live_completion_readout()` and zero legacy calls. Only the test contract
was corrected. The first observed-source packet also correctly returned
`VETO`: it mislabeled `PreparedCorrectionLease` as a producer after the
candidate-rank owner. The lease is an orchestrator carrying an already selected
readout, so the paper role and authority terminal were corrected without a code
change.

This slice proves schema, source-route cardinality and unit behavior. It does
not yet prove physical per-frame trace cardinality, eligible immediate-Space
delivery, latency percentiles or aggregate restoration/false-split quality.

Exact receipts:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice4-observability-and-duplicate-removal.json`

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice4-observed-source-route.json`

### Slice 5: fixed immediate-Space delivery

The fixed proof preserves an inter-letter pause but emits Space immediately
after the last printable character. It uses `4` warmup boundaries and a fixed
eligible denominator of `20` boundaries. Attempt 1 did not reach runtime
because the smoke harness incorrectly required a local `lay-daemon`; it is not
counted as delivery evidence. Attempt 2 exercised the staged engine and then
restored the installed runtime.

Measured Attempt 2 facts:

```text
eligible Space events                         20
eligible NotReady                        20 / 20
warmup NotReady                           3 / 4
Space p50 / p90 / p99 / max     8.182 / 8.226 / 8.245 / 8.245 ms
printable p50 / p90 / p99 / max 0.105 / 0.160 / 0.194 / 0.244 ms
display projections                       20 / 20
correction projections                     4 / 20
producer-budget failures                        0
late edits after literal fallback               0
runtime authority changed                    false
```

The trace establishes the first common failure mechanism:

```text
printable N
-> correction worker starts generation N
-> deterministic candidate materialization cannot be preempted
-> printable N+1 replaces desired work but cannot stop running work N
-> final token either remains queued or starts a 108.974-125.444 ms run
-> Space waits the unchanged bounded 8 ms
-> literal fallback invalidates the frame lease
-> late pure work is discarded as superseded
```

For the `20` eligible final frames, `16` correction projections never started
before fallback. The remaining `4` finished only after fallback and were
correctly rejected as superseded. Their measured `correction_total_us` values
were `125444`, `120292`, `108974` and `122908`. In those same records:

- `decision_total_us` was only `311-588`;
- `correction_l3_us` was only `20-67`;
- `l11_us` and `productive_v90_us` were zero;
- `field_cache_disposition` was `not_requested`.

Therefore this is not an L1.1, Productive V90, L3, verifier or Space-wait
budget failure. The expensive work is deterministic candidate materialization
before `DecisionCore`, and the current single correction worker has
head-of-line blocking. Increasing the wait budget, spawning an unbounded worker
per key, applying a late edit, or adding surface-specific fast paths is
rejected.

The V2 preflight then authorized timing-only instrumentation. A remote
optimized focused run isolated the candidate stage further:

```text
deterministic candidate total             309.176 ms
Boundary birth                              0.001 ms
primary typing-rule pass                  262.995 ms
composite candidate pass                   46.160 ms
slowest primary rule:
  experimental_layout_ru_to_en            186.075 ms
slowest word-only composite rule:
  single_letter_substitution               25.014 ms
```

This is a cold optimized unit measurement, not the physical replay latency
denominator. It proves mechanism ownership only. `BoundaryCell32` is not the
cost center. The repeated layout guard expands the complete Russian typo set,
then the normal typo rules expand it again, and the composite lane runs a
second word-only pass. The selected implementation therefore uses one bounded
typed memo of pure word-repair material. It may retain negative and positive
candidate material by operation kind, token and package generation; it may not
retain ranking or edit authority.

When immediate Space times out, an already running or queued final-token job
may finish as `material_only` so a later hot request can reuse its pure
material. Its original worker generation remains superseded and cannot publish
a correction lease. A new printable may overwrite that one bounded queued
material-only job; no second worker or unbounded queue is introduced.

The next implementation slice must separate reusable pure candidate material
from per-frame authority:

```text
typed token/local-structure key + rule/package generations
-> bounded single-flight pure candidate material
-> current full-context projection
-> current L3/L4/DecisionCore
-> exact InputFrameIdentity lease
-> verifier and one committed-tail mutator
```

Only pure candidate material may survive frame supersession. A selected
winner, online context score, `AuthorizedEdit`, `InputFrameIdentity`, feedback
state or mutation permission must never be cached. Cold first-touch behavior
and hot reuse must be reported separately.

#### Attempt 3: corrected proof and hot reuse

The managed proof configuration previously left `nanda_autocorrect=false` and
the analyzer treated a ready lease without an edit as a successful correction.
Attempt 3 repairs both proof defects: the GTK harness now records exact output,
the analyzer requires the exact expected text, and every eligible boundary must
have an `Applied` lease outcome.

Measured facts:

```text
warmup exact output                         FAIL 1 / 4
warmup NotReady                                  3 / 4
eligible exact output                       PASS 20 / 20
eligible Applied                            PASS 20 / 20
eligible NotReady                                0 / 20
projection cardinality                     PASS
producer / generation parity               PASS
late edits after literal fallback               0
Space p99 / max                       0.290 / 0.290 ms
printable p99 / max                   0.206 / 0.303 ms
runtime authority changed                       false
installed version                                 1.0.33
```

This result proves hot reuse only. It does not satisfy cold first-touch, because
the warmup GTK output remained `данорм мнесбросили Еленапросит коде` instead of
`да норм мне сбросили Елена просит коде`.

Exact receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice5/immediate-space-full-authority-attempt3.jsonl
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice5/immediate-space-full-authority-attempt3.harness.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice5/immediate-space-full-authority-attempt3.receipt.json
```

#### Release profile and next route experiment

The optimized focused profile separates initialization noise from the
per-request mechanism:

```text
short-left Boundary:
  deterministic candidates                         0
  deterministic total                        131.946 ms
  primary typing rules                       121.839 ms
  experimental_layout_ru_to_en                99.724 ms
  composite pass                              10.091 ms

two-content Boundary:
  deterministic candidates                         0
  deterministic total                         43.296 ms
  primary typing rules                        36.185 ms
  glued_phrase                                13.005 ms
  composite pass                               7.102 ms
```

In both cases the selected replacement already comes from
`CanonicalL2FieldBoundary`; the legacy lane adds no competitor or supporting
evidence. Operation-level memoization can make a later identical request fast,
but cannot make an unseen final token meet the unchanged `8 ms` lease budget.

The first canonical-first sketch was incomplete. `L2FieldAuthority::Winner`
proves a local lexical/morphology winner; it does not prove that every operator
domain relevant to the observed input has been covered. In particular, layout,
mixed-script, technical-token and structural producers may carry independent
evidence. Skipping them merely because a lexical winner exists would exchange
latency for unmeasured false authority.

The next experiment is therefore driven by a typed material-coverage receipt:

```text
observed input + correction config
-> required operator domains

canonical L1.1 -> Productive V90
-> canonical candidates + local authority + availability
-> covered operator domains

required domains subset of covered domains
and canonical material is complete
  -> final canonical lattice
else
  -> complete deterministic fallback material
  -> merge canonical material with the old ordering and evidence semantics
  -> final fallback lattice

exactly one final lattice
-> exactly one common DecisionCore invocation
-> zero or one correction lease
```

`DecisionCore` refusal is terminal and fail-closed. It never triggers a second
fallback readout. A final safety refusal means that the complete candidate
material did not earn apply authority; it is not evidence that another producer
must be run.

The coverage receipt is source-neutral and contains no literal word, phrase,
test id or producer string as authority:

```text
input profile
required domains: lexical | morphology | boundary | layout | technical
canonical availability
canonical local authority: Winner | Tied | Abstain | Unavailable
candidate role and error-class bitsets
eligible candidate groups after ordinary proposal admission
winner-surface retention or exact clean-preservation receipt
coverage disposition: Complete | NeedsFallback(reason)
```

The initial proof hypothesis is deliberately conservative. ASCII layout,
mixed-script, technical/unsupported input, transient package failure, missing
winner material and unresolved lexical authority require the complete old
fallback. Pure Cyrillic material may be marked complete only when the ready
canonical field retains its typed winner, proves exact clean preservation, or
contains an independently admitted canonical Boundary operator. These are
feature buckets, not word exceptions. A bucket that diverges is rejected as a
whole and returned to fallback; individual failures are never patched.

The old full route remains the fixed comparison baseline, but parity has two
different contracts:

```text
NeedsFallback branch:
  exact ordered candidate/evidence lattice parity
  exact verdict, transition proof and safety effect parity

Complete branch:
  exact target retention
  exact final verdict and selected surface
  exact transition operator/proof and safety effect
  every omitted deterministic candidate bucketed by typed role/class/gate
  zero omitted independent authority that changes the final readout
```

Global candidate-surface equality is not a valid requirement on the Complete
branch because removing irrelevant legacy material is the intended change.
Candidate-set shrinkage is measured, not hidden. The experiment is rejected if
any aggregate proof class regresses, any false-authority/false-singleton
divergence appears, a canonical target is lost, an independently authoritative
operator is omitted, or fallback cardinality becomes unbounded. L1's fixed
`260,000` damage proof remains necessary but is not sufficient: layout,
mixed-script, technical and boundary/false-split denominators are reported as
separate route domains.

The V3 design route records the corrected ownership chain:

```text
canonical producer [-> deterministic fallback producer when required]
-> one final candidate-lattice evidence owner
-> one DecisionCore rank owner
-> verifier authorization owner
-> committed-tail mutation owner
```

Exact design artifacts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/preflights/LAY_IME_CANONICAL_FIRST_CORRECTION_ROUTE_2026-08-16.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_CANONICAL_FIRST_CORRECTION_ROUTE_DESIGN_V3_COVERAGE_2026-08-16.json
```

Exact release profile:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice5/immediate-space-release-deterministic-profile-attempt3.json`

Implementation preflight:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/immediate-space-material-reuse-preflight-v2.json`

Exact receipts:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice5/immediate-space-replay-attempt2.json`

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice5/immediate-space-runtime-attempt2.jsonl`

## 9. Consequence Matrix

| Dimension | Required effect | Regression risk | Control |
| --- | --- | --- | --- |
| Candidate quality | Same complete L2 lattice | field extraction drops evidence | before/after lattice parity |
| False authority | No new singleton winner | Boundary bypasses competitors | full common DecisionCore only |
| Morphology | Productive V90 remains live owner | fast path hides failed package | package gate before route work |
| IME quality | Display candidates remain available | correction blocks display | independent typed projections |
| Space latency | No field work on IBus thread | lease misses 8 ms deadline | correction-first scheduling and single-flight |
| CPU | One V90 field computation per key | two cache-miss producers race | producer-count invariant |
| RSS | Bounded immutable cache | unbounded in-flight or retained scenes | fixed limits and Arc ownership tests |
| Learning | Existing accept/censor/reject meanings | outputs share feedback state | separate display/correction receipts |
| Concurrency | Stale work never publishes | focus/epoch race | exact identity and supersession tests |
| Rollback | Old binary/packages remain usable | schema migration is irreversible | no persistent schema change in first slices |
| Maintenance | Fewer identities and field owners | generic coordinator monolith | domain field and adapters remain separate |

## 10. Promotion Gates

All gates are conjunctive.

1. Productive V90 status is `ready_live_owner` with exact active package hashes.
2. Productive field extraction has byte/semantic parity for surfaces, authority,
   morphology evidence and verdicts on the fixed proof corpus.
3. Every required restoration class remains `unique top-1 > 95%`; clean
   preservation, lattice coverage, false certainty, package/RSS and latency are
   reported separately.
4. False authority and false singleton remain zero on the fixed proof.
5. Glued-word recall and false-split denominators are reported; selected
   examples cannot substitute for the aggregate proof.
6. Exactly one Productive V90 field producer runs per `InputFrameIdentity`.
7. No L1/L2/L3 materialization occurs synchronously on the printable or Space
   IBus thread.
8. Fixed immediate-Space replay has zero eligible NotReady losses; physical GUI
   trace separately reports NotReady frequency and Space p50/p90/p99/max.
9. Stale generation, focus change, backspace, layout change and package reload
   cannot publish display or apply mutation.
10. Space applies exactly one separator and never performs a delayed edit after
    literal fallback.
11. Autocorrect followed by double Shift restores the exact original token.
12. WeChat and Telegram physical checks show no stuck key, repeated character,
    repeated Space or lost separator.

## 11. Build, Deployment, and Rollback

- Cargo builds and proof runs execute on `e@192.168.3.94`, using its available
  CPU, not on the local workstation.
- Local work is limited to source inspection, documentation, small static
  gates, installation and physical GUI validation.
- Keep the installed release and package hashes as rollback artifacts before
  any deployment.
- Do not restart the global `ibus-daemon` during ordinary deployment. Restart
  only the Lay-managed component required by the changed artifact.
- Version, documentation, graphify update, commit and push occur only after all
  promotion gates pass.

## 12. Current Verdict

`SLICE_4_OBSERVABILITY_AND_SOURCE_ROUTE_PASS_SCOPED_IMMEDIATE_SPACE_AND_GENERAL_LATENCY_OPEN`.

The exact Productive package binding remains fail-closed. Slice 1 preserved the
complete materialized readout, and Slice 2 now reuses one immutable field with
exact package-generation identity. Four baseline/new semantic projections have
zero candidate or authority mismatches. The measured hot p99 values are
`4.287`, `3.083` and `8.187 ms`; the damaged-prefix case therefore keeps the
general latency gate open. Slice 3 now gives display and correction one exact
GUI identity, schedules correction first, and makes Space lease consumption
single-shot before display cancellation. Slice 4 carries field evidence through
typed per-call telemetry and separates display, correction, Space lease,
mutation, observation and proof ownership. The remote IBus target is `205 / 205
PASS`; the observed-source route is `PASS` with `64 / 64` markers. Runtime
authority is unchanged; fixed immediate-Space replay, physical GUI evidence,
general latency, aggregate quality and deployment are not yet authorized.
