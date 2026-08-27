# M3 Typed View Actual-Owner Consequence V1

Date: 2026-08-27

Status: `ACTUAL_OWNER_DESIGN_STRUCTURALLY_ACCEPTED`

## Scope

This paper consumes the sealed M3 typed-view result and the audited local
test-source integration. It decides the smallest test-only route that may pass
exact V13 candidates and their Phase 7D certificates through the source that
owns `PreparedCanonicalTokenField`. It does not activate the live bridge,
install a sidecar, change runtime authority, admit deployment, or inherit the
M3 microbenchmark gain as an end-to-end result.

Immutable predecessors:

```text
M3 terminal receipt SHA-256
  a84355e42bad335d45b379c7e76d2b353bed6c23c30593e1c721be0c0058f324
M3 source/lifetime decision SHA-256
  e7b0f66170776677c2b153254aa01a303fdf8538aec273678196dec723715b24
M3 test-source integration receipt SHA-256
  1e3e372c858e09e571590be4262e4d923a48e32dede937aac8f18556f10dfe99
M3 test-source integration audit SHA-256
  5a6deb02bc6ec703afe375e25c1cc3d40b5a1d012d90ce63914cad0d17a4ae3d
```

## Source Facts

The current production call graph is:

```text
canonical_text_readout_observed_with_frame
  -> installed_l2_field()
  -> installed_productive_l2_v1()
  -> CanonicalTokenKey
  -> cache::get_or_prepare(...)
  -> prepare_live_productive_v1_field(...)
  -> PreparedCanonicalTokenField
  -> materialize_live_productive_v1_field(...)
```

`installed_l2_field()` owns one process-lifetime `StandaloneL2Field` through a
`OnceLock`. Productive V90 is a separately reloadable `Arc` generation. Its
reload clears the prepared-field, bridge and completion caches. The
per-token `CanonicalTokenKey` already binds L1.1, canonical V13 and Productive
V90 package identities; it is a borrower and must not own the 3.69 MB typed
DAFSA view.

The current M3 engine is still guarded by `#[cfg(test)]`. No installed DAFSA
artifact exists in the active model directory, and no production source calls
`V13DafsaView`, `TypedExactView` or the `LAYV13D3` format.

## Discovered Integration Mismatch

The existing contour path is not a valid substitute for the V13 exact peak
lane:

```text
Composite contour lane maximum             8 surfaces
material contour-only retention maximum    8 targets
common prepared target maximum             74 targets
compact witnesses per target               4 roots
```

`shared_field_contour_births` also deduplicates by surface, namespace,
grounding ref, coarse relation and membership before material construction. A
Phase 7D target may carry multiple distinct certificates. Reusing this lane
would therefore impose the wrong denominator and may collapse independently
proved certificate roots.

The repair must not raise any capacity. A candidate or certificate set that
does not fit the existing common material contract remains incomplete and
cannot acquire authority.

## Selected Test-Only Design

The selected design adds a separate exact-peak adapter to the existing owner
preparation, reachable only from tests:

```text
one validated V13 byte generation
  -> one safe typed materialization
  -> typed exact target-blind search
  -> structured Phase 7D certificate evidence
  -> exact-peak Born enumeration
  -> existing Productive V90 preparation
  -> existing PreparedCanonicalTokenField owner
  -> exact material and composite-lattice observations
```

The normal live wrapper passes an empty exact-peak enumeration and must remain
behaviorally and source-route equivalent. The new test entry may pass a
complete exact-peak enumeration to the same private preparation core. The
bridge callsite, cache key, installed package discovery and reload functions
are outside the edit set.

Each exact certificate birth carries:

```text
canonical form_ref
normalized surface
structured Phase 7D certificate class
canonical serialized certificate key
TargetRelationV1 projection
stable operator ref
stable derivation ref
Born membership
zero grounding support
```

The canonical serialized key is retained in the exact shadow table. Compact
refs are accelerators only. Any collision between unequal keys is an integrity
failure. The full exact table, not a 32-bit ref alone, owns certificate parity.

## Candidate And Certificate Semantics

The exact-peak lane is discovery evidence only:

```text
V13 exact peak             -> Born
V13 exact peak             -/-> Winner
V13 exact peak             -/-> Eligible mutation
Productive V90             remains candidate-rank owner
common L3                  remains downstream selection owner
DecisionCore and verifier  remain authorization and safety owners
```

An existing grounded L1.1 winner remains protected. Multiple Born surfaces
force downstream comparison; they cannot erase the grounded winner or create a
second direct winner. Candidate materialization from this test lane must remain
`SuggestOnly` unless independent existing authority already admits the same
surface.

Certificate classes project to the existing coarse relation vocabulary while
the exact serialized key retains the full operator parameters. Identity and
punctuation-preservation certificates remain lexical restoration evidence;
missing, extra, substitution, layout, transposition, repeated-fragment and
sparse-omission classes use their existing `TargetRelationV1` families.

## Completeness And Capacity

The owner proof must compare the complete forward and reversed M3 results for
all 382 fixed cases. For each case it freezes the exact candidate/certificate
set before owner preparation and then requires:

```text
input candidates == retained exact candidates
input certificate keys == retained exact certificate keys
forward set == reversed set
unknown form refs == 0
surface mismatches == 0
certificate-key collisions == 0
candidate truncation == 0
certificate-root overflow == 0
false completeness == 0
```

If total material exceeds 74 targets, any target exceeds four semantic roots,
or an upstream search is unresolved, the only valid result is
`BLOCKED_CAPACITY`. No top-k selection, target-conditioned retention or silent
certificate coalescing is admitted.

## Consequence Matrix

### Candidate and lattice retention

Exact peaks enter one dedicated Born lane. They are merged by exact normalized
surface only after every form ref and certificate identity is preserved. The
proof reports candidates, surfaces, certificates and compact completeness as
separate denominators.

### Ranking and false authority

No V13 score or traversal order is introduced. Productive V90, common L3,
DecisionCore and the verifier remain unchanged. Every exact-only emitted
candidate is `SuggestOnly` in this proof.

### Latency and deadlines

This step records owner preparation wall time only as diagnostic evidence. It
does not set or pass the end-to-end p99 gate. A separate measured-region paper
is required after parity.

### CPU, RSS and allocation

The typed DAFSA materialization remains once per proof generation, not once per
request. The proof records its 3,689,628-byte typed payload plus exact
certificate-table bytes. Process PSS/RSS admission remains a later gate because
the active runtime does not yet install this generation.

### Cache identity and invalidation

No new global cache is created. The local proof owns one immutable generation
and all 382 requests borrow it. A future live generation must add the exact
sidecar identity to the canonical generation and token key atomically. This
paper does not implement that reload contract.

### Package and delta reload

Canonical V13 currently has process-lifetime ownership, while Productive V90
can reload and invalidate dependent token fields. The proof does not create an
independently reloadable DAFSA owner. Live V13/DAFSA reload remains deferred to
a separate atomic-generation decision.

### Learning and feedback

No feedback, acceptance count, calibration or learning input changes. Exact
peaks are observed only in the fixed proof and cannot write runtime state.

### Concurrency and stale results

No thread, daemon or single-flight behavior changes. Future readers must hold
one immutable generation identity, and stale token fields must be rejected
after generation replacement. This proof checks identity fields but does not
exercise a live reload race.

### Failure and rollback

All errors fail before publication of a positive receipt. Existing runtime
continues unchanged because the route is test-only. No marker, installed file,
package or service state exists to roll back.

### IME and daemon consumers

The daemon, IBus engine, queue, bridge entrypoint and replacement behavior are
not edited or executed. Their quality and latency remain untested by this step.

### Maintenance and removal

The adapter is private and test-only. It can be removed with its proof without
format migration. Production promotion later requires an installed sidecar
contract, atomic generation owner and explicit removal/replacement plan.

## Rejected Alternatives

1. Reusing the contour lane is rejected because its eight-target reservation
   and coarse pre-material dedup are not the exact candidate denominator.
2. Storing the typed view in `PreparedCanonicalTokenField` is rejected because
   that is a per-token cache value and would duplicate generation-wide bytes.
3. Adding a second global DAFSA `OnceLock` is rejected because V13 bytes and the
   typed view could diverge across independent admission paths.
4. Activating the bridge directly is rejected because no installed sidecar,
   atomic reload, RSS or end-to-end latency proof exists.
5. Raising target or witness capacity is rejected. Capacity failure is a
   scientific result, not permission to change the denominator.

## Admitted Edit And Proof Scope

Only a future implementation preflight may admit:

```text
structured Phase 7D certificate projection under cfg(test/compiler)
private exact-peak Born enumeration
test-only actual-owner preparation entry
exact material/lattice audit projection
one focused local cargo-guard proof over the fixed 382 cases and two schedules
immutable local receipt and architecture update
```

The edit must not change `l2_field/mod.rs`, `bridge.rs`, `cache.rs`, Cargo
inputs, installed files, runtime environment, network state, services or
package formats.

## Verdicts

Positive terminal verdict:

```text
M3_ACTUAL_OWNER_PARITY_PASS
```

Failure verdicts:

```text
BLOCKED_PROVENANCE
BLOCKED_SEMANTIC
BLOCKED_CAPACITY
BLOCKED_CERTIFICATE
BLOCKED_OWNER_IDENTITY
BLOCKED_BUILD
```

No failure grants a capacity increase, alternate lane or automatic rerun.

## Structural Gate Result

The consequence was checked as separate ownership, evidence, authority and
claim-boundary routes. Ownership route A, evidence route B and claim route C2
passed without repairs. The original combined route C was vetoed because it
placed distinct downstream owners in one group. It remains immutable evidence
and was superseded by split routes C1/C2. C1 V1 passed with a residual shared
evidence-span repair; C1 V2 bound every owner transfer to a distinct span and
passed with an empty repair queue.

Code-route V1 was vetoed because it incorrectly represented immutable
generation data as a backward authority edge. Code-route V2 removed that edge,
kept execution calls separate, left `PreparedCanonicalTokenField` as the only
evidence owner in the material route, and passed with no issues.

These PASS receipts are structural only. Source edits still require an exact
implementation preflight.

## Claim Boundary And Next Tree

```text
M3_TEST_SOURCE_INTEGRATION_AUDITED
  -> actual-owner structural gate
  -> code-route gate
  -> implementation preflight
  -> test-only owner adapter
  -> one fixed local parity proof
  -> M3_ACTUAL_OWNER_PARITY_PASS
  -> separate end-to-end latency/RSS/reload preflight
  -> production authority decision
```

Until `M3_ACTUAL_OWNER_PARITY_PASS` is sealed:

```text
production authority change       false
live bridge activation            forbidden
installed sidecar                 absent
runtime/package reload edit       forbidden
end-to-end performance claim      not proved
deployment/install/restart        forbidden
```
