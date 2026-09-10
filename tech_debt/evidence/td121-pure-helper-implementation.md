# TD-121 pure context-admission helper implementation

Date: 2026-09-05. Baseline: `cc1e2207519801ca0f9b7c6963897b55953a7751`.
Scope: new unregistered pure helper and unit/transport tests only.
`runtime_authority_changed=false`.

## Pre-code consequence analysis

The selected implementation is one metadata reducer, one bounded callback-stamp
ring, and an `ordered_stream::Join` adapter over zbus 5.15 `MessageStream`s. It
does not retain text, emit edits, own a key queue, call RPC, inspect model data,
or mutate existing engine/shared state. A successful reducer result is a typed
single-use grant containing generations and a source tail epoch; later wiring
must revalidate and copy the actual tail under the existing owner.

Baseline behavior loses the old-object prefix on new-object handoff. Two
alternatives remain rejected: context equality plus TTL cannot exclude foreign
ABA, while global ObjectServer serialization adds blocking and reentrancy risk
across unrelated callbacks. The selected local rendezvous can hold only zbus's
existing per-interface write lock for at most the callback deadline; the
observer takes a short private mutex, drops it before notification, and never
waits for engine/shared/settlement/RPC work.

- Candidate/lattice and ranking: unchanged because the helper produces no
  candidate. Unknown completeness cannot authorize correction, Tab, manual
  replay, or learning when consumers are later wired.
- Latency: fast stamp hits allocate no listener. A miss uses listen-before-
  recheck and one absolute deadline capped at 1 ms. The Space remainder helper
  subtracts callback elapsed time from the existing 3,500 us; it adds no second
  allowance.
- CPU/RSS/allocation: the stamp ring and unsettled-key set are fixed-capacity;
  identifiers are length-checked. No worker, actor, polling loop, text buffer,
  package copy, or resident model state is introduced.
- Cache/identity: connection, owner, activation, request, ticket, lineage and
  frame generations stay distinct. Foreign ABA increments irreversible
  revocation; returning to Lay cannot reduce it. Sequence remains the opaque
  zbus type and is never converted to an integer.
- Package/reload and learning: package generations are outside this helper;
  reload cannot upgrade Unknown. The grant contains no feedback authority.
- Concurrency/stale results: stamp receipt and callback settlement are separate.
  Missing/evicted/revoked/terminated rendezvous fails closed. Ticket consume is
  single-use and old owner cleanup can compare the returned generations.
- Failure/rollback/compatibility: timeout, bus loss, unsupported mode, wrong
  sender/nonce/context, unsettled keys or malformed headers deny transfer but
  do not consume/replay a key. The new module is not registered, so rollback is
  removal/non-registration; no IBus, service, config or installed binary changes.
- Maintenance: this is one removable admission source, not a second text owner
  or general actor framework. Compatibility acquisition itself remains an
  explicit later integration contract.

Expected regression surface after later wiring is false Unknown on scheduler
delay/ring pressure and up to the remaining local callback budget on a miss.
Promotion denominators remain separate: pure reducer D1, zbus merge D2, four
rendezvous schedules plus timeout/revoke, sender/budget helpers, then C01-C30,
resource measurements and client-visible restoration. No proof has run yet.

## Implementation and verification record

The helper is staged outside the repository source tree so the concurrent
TD-120 checkpoint does not bind uncommitted TD-121 source into its graph:

`/home/ubu/.cache/lay/td121-prewire-7iGwHB/context_admission.rs`

with sibling modules under
`/home/ubu/.cache/lay/td121-prewire-7iGwHB/context_admission/`.
There are no copies under `src/bin/lay_ibus_engine/` yet.

Implemented staged pure-helper pieces:

- typed connection/context, owner, activation, lineage, request, ticket and
  frame generations;
- bounded identifiers, authenticated three-role sender policy, complete callback
  header correlation and opaque `zbus::message::Sequence` production stamps;
- actual metadata-only admission reducer with factory/reflexive single-use
  tickets, source-key settlement, context reply plus marker sealing, sticky
  Unknown, irreversible foreign ABA revocation and authority-invalidating grant;
- fixed-capacity callback stamp store with duplicate/eviction/revoke/termination
  failure, fast lookup, listen-before-recheck and one absolute <=1 ms deadline;
- production `ordered_stream::Join` composition and ordered drain helper for
  zbus `MessageStream`s from one connection;
- remaining Space allowance as `3500us.saturating_sub(callback_elapsed)`, not a
  second wait budget.

Prepared test manifest: 19 test functions (the original 17 plus two contract
regressions). P121-1 D1 holds foreign/return events
until after Get, checks Pending and irreversible rejection, and separately checks
one positive metadata transfer of source tail `l` with zero edit/guard/feedback
and no second consume. The added regressions require consume to preserve the
exact sealed word-lineage generation and permit reply/focus sequence equality
only for a Native FocusIn payload; CompatibilityProperty remains strict-later.
P121-2 D2 creates an actual zbus 5.15 Unix p2p connection
pair, captures actual received opaque `Sequence`s, gates the earlier lifecycle
stream, makes the marker branch ready first, includes an idle stream whose
`NoneBefore` permits progress, asserts foreign/return/marker order, then feeds
those messages to the same reducer. Four rendezvous publication positions,
absolute timeout, owner revocation, ring eviction and duplicate headers are
covered. A target key arriving before admission is also checked as one literal
delivery with transferred completeness poisoned to Unknown. P121-3 uses mock
complete headers for the separately authenticated IBus owner,
`org.freedesktop.DBus` callback sender, own marker sender and rewritten foreign
sender, plus exact Space remainder cases. Source settlement, Unknown boundary
semantics, acquisition timeout and reflexive ticket use are separate tests.

The deployed three-sender fact was supplied by the parent from a separate 23/23
isolated real-IBus run: engine/factory lifecycle sender
`org.freedesktop.DBus`, `GlobalEngineChanged` sender the authenticated IBus owner,
and marker sender this connection's unique name. This helper does not claim or
duplicate that runtime proof.

Static formatting/parser check: `rustfmt --edition 2021 --check` returned
success for all four staged Rust files. Guarded remote compilation and all 19
tests then passed against the checkout-locked zbus 5.15.0. Scoped verdicts and
the exact proof record follow below.

| Staged file | SHA-256 |
|---|---|
| `context_admission.rs` | `77f3412f76d1c97ff12de98f5870f129d9ee65e9788cec450bd1f1810c2c719a` |
| `context_admission/ordered_merge.rs` | `07a189ca12f842d8931d2dea0aaa798a038e96214550240b8d432973a14174f9` |
| `context_admission/rendezvous.rs` | `29bb37bbce511b5a4a21f4ff701dfd8df2ac1133e8f555b5f3a388d6a9598739` |
| `context_admission/tests.rs` | `737bd249faf788ad8a1df149c6dcac835749290215fffb96c7966b36e004ee0c` |

## Guarded pure-helper proof

Two contract defects were found before the compile gate and repaired in the
staged source, not in the repository checkout:

1. `consume()` allocated a new `LineageGeneration`, which broke the promised
   identity continuity of the word sealed by the source. It now assigns the
   exact `seal.lineage`; owner, activation and frame generations still rotate.
2. `try_ready()` required `reply_position > target_focus_position` for both
   receipt origins. A native context receipt is carried by the FocusIn message
   itself and therefore has the same opaque received `Sequence`. Equality is
   now admitted only for `ReceiptOrigin::Native`; a compatibility property Get
   must remain strictly later.

Local proof harness and artifacts:

- harness: `/home/ubu/.cache/lay/td121-proof-local-uDNdLD52/`;
- fresh archive: `td121-proof.tar`, SHA-256
  `a8400bcc56cc82822a685d4e64916ebe71602db234b8dcec458c4f11ae39030b`;
- final transcript: `td121-test-zbus-5.15.0.log`, SHA-256
  `78d533125613d56fe23453a08a768dd9edaae0a23e346221adfeef1e5cf9df3f`.

Remote extracted proof crate and matching transcript:

- `/home/e/projects/td121-proof-JGik2wUq/` on `e@192.168.3.94`;
- `/home/e/projects/td121-proof-JGik2wUq/td121-test-zbus-5.15.0.log`;
- source and transcript SHA-256 values were checked after transfer and matched
  the local artifacts.

The harness pins `zbus = { version = "=5.15.0", features = ["p2p"] }`.
An initial passing harness run exposed that an unpinned `version = "5.15.0"`
fresh lock resolves to zbus 5.19.0. That run is not the version-parity proof;
the manifest was pinned to the checkout's `Cargo.lock` version and the complete
suite was rerun. The final lock reports zbus 5.15.0.

Exact static check:

```bash
rustfmt --edition 2021 --check \
  /home/ubu/.cache/lay/td121-prewire-7iGwHB/context_admission.rs \
  /home/ubu/.cache/lay/td121-prewire-7iGwHB/context_admission/ordered_merge.rs \
  /home/ubu/.cache/lay/td121-prewire-7iGwHB/context_admission/rendezvous.rs \
  /home/ubu/.cache/lay/td121-prewire-7iGwHB/context_admission/tests.rs
```

Exact final remote proof command, run from the extracted proof crate:

```bash
cd /home/e/projects/td121-proof-JGik2wUq
env \
  LAY_RESOURCE_PROFILE=dedicated-20cpu \
  CARGO_BUILD_JOBS=20 \
  RUST_TEST_THREADS=1 \
  CARGO_TARGET_DIR=/home/e/projects/lay-td119-gate-v1/target \
  /home/e/projects/lay-td120-121-SUdh2I/scripts/lay-resource-guard.sh -- \
  /home/e/projects/lay-td120-121-SUdh2I/scripts/cargo-guard.sh \
  test --manifest-path /home/e/projects/td121-proof-JGik2wUq/Cargo.toml -- \
  --test-threads=1
```

Measured result: `19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered
out`. The final receipt replay was cached (`0.04s` build, `0.01s` tests). The
guard reported `profile=dedicated-20cpu`, `jobs=20`, `test_threads=1`, a 2000%
CPU quota, 24 GiB memory-high and 28 GiB memory-max. The shared target grew from
3.4 GiB before compilation to 3.9 GiB after proof. Compilation emitted only
unused/dead-code warnings expected from an isolated unregistered module; there
were no compilation or test failures.

Scoped verdicts, with separate denominators:

- **P121-1 PASS — 10/10 reducer/authority tests.** Actual reducer held-FIFO
  foreign ABA rejection, Get-alone denial, positive single-use metadata
  transfer, exact sealed-lineage preservation, Native equality versus strict
  CompatibilityProperty order, nonce, source settlement, Unknown boundary,
  early target input and reflexive ownership behavior passed.
- **P121-2 PASS — 8/8 transport/rendezvous tests.** Actual zbus 5.15.0 Unix p2p
  `MessageStream`/`Join` marker-ready-first ordering, four lost-wake schedules,
  absolute timeout/revocation, ring eviction and duplicate-header rejection
  passed.
- **P121-3 PASS — 1/1 sender/budget test.** Authenticated three-role sender
  distinction and exact saturating subtraction from the existing 3,500 us
  Space budget passed.

These are pure-helper proof verdicts only. Quality, production authority and
runtime behavior remain `UNKNOWN`: the module is still unregistered and no
runtime route was exercised.

## Exact dependency and registration handoff

The parent must add these direct dependencies; relying on zbus transitive crates
is intentionally unsupported:

```toml
async-io = "2.6.0"
event-listener = "5.4.1"
ordered-stream = "0.2.0"
zbus = { version = "5.15.0", features = ["p2p"] }
```

The last line replaces the current plain `zbus = "5.15.0"`; `p2p` is needed by
D2. Add
`#[path = "lay_ibus_engine/context_admission.rs"] mod context_admission;` to
`src/bin/lay_ibus_engine.rs`, then copy the staged file/subdirectory preserving
their sibling layout. No zbus re-export is assumed and `Sequence` is never
numerically constructed or converted.

Minimal later integration contract: subscribe and bootstrap on the existing
IBus connection before publishing the factory; derive all three sender bindings
from that authenticated bus epoch; continuously drain the joined narrow streams,
advance reducer metadata synchronously, then publish the matching stamp before
callback settlement. A callback captures `Instant` and `#[zbus(header)]`,
performs the bounded rendezvous, records key start, then settles separately after
its existing text revision. Later wiring must also make the reducer's per-owner
word lineage follow the authoritative settled `WordScope`; the present external
`WordScope` helper alone cannot promote a reducer lineage that began Unknown.
Acquisition performs one Get when required, emits one exact private marker, and
presents the ordered reply and marker to the reducer. Initial activation and a
new input field need this non-transfer Get-plus-marker acquisition path too;
the helper intentionally does not invent bootstrap authority for them. Space
passes callback elapsed time to the remaining-budget helper. No observer/
admission/Shared mutex may cross an await; only zbus's existing per-interface
lock spans the local rendezvous.

Explicitly unimplemented here: actual match-rule list and observer task
lifecycle; authenticated owner/mode/property bootstrap; Get/native convergence,
marker emission and acquisition timer; authoritative settled-`WordScope` update
of the reducer lineage; non-transfer initial/new-field activation acquisition;
callback/factory/server/engine/shared, bridge, atomic, layout and trace-counter
wiring; text copy at grant consume; resource measurement; C01-C30/H expansions;
positive client/physical proof; install, release, push and architecture refresh.
These are production promotion work, not simulated by this helper.

No local Cargo, runtime probe, service action, config edit, keyboard input,
installation, release, graph refresh or Git write was made. Only the isolated
remote pure-helper compile/test proof above ran. The remote build lease was
released after confirming no Cargo/rustc process remained; the parent owns all
runtime integration and promotion work.

## Pre-code adapter addendum (2026-09-06)

This addendum admits the next bounded implementation slice before code. It adds
one same-IBus-connection metadata adapter and closes reducer API gaps, still
outside the repository source tree. It does not wire `server`, `factory`, an
engine object, Shared/atomic/layout state or the session-bus bridge.
`runtime_authority_changed=false` remains true.

### Authority and ownership consequences

- The adapter receives an already-authenticated zbus `Connection`; it never
  opens a second IBus connection. Subscription setup precedes bootstrap reads.
  Bootstrap authenticates the current IBus owner with `GetNameOwner`, captures
  this connection's actual unique sender, validates the engine-callback sender
  role, and establishes global-engine mode/profile for one connection epoch.
- Narrow `MessageStream`s are merged only through the existing production
  `ordered_stream::Join`. The event-driven observer owns and drops those streams.
  It may await only stream readiness; after a message arrives it performs a
  short metadata update, releases the mutex, and notifies rendezvous waiters.
  It never waits for engine/Shared locks, callback settlement, a method reply or
  other RPC.
- Compatibility acquisition is one bounded Properties.Get of
  `CurrentInputContext` followed by one exact private self-signal marker on the
  same connection. Native acquisition skips Get but uses the same marker fence.
  Cancellation, timeout, stream loss, sender/header mismatch or lifecycle
  revocation fails closed with no retry. A source-free initial or new-field
  activation must traverse this same receipt-plus-marker reducer contract and
  begins `UnknownStart`; property equality alone cannot call
  `establish_source` or create transfer authority.
- A settled key update binds the current owner, complete callback header, exact
  monotonic tail revision and authoritative `WordScope` lineage atomically in
  reducer metadata. Source sealing accepts only that exact settled revision.
  An old owner or stale revision returns refusal without changing the new
  owner's lineage.
- CreateEngine opens an unbound factory request from the callback's actual
  receive stamp. The produced engine path is bound later by ticket identity;
  callback completion order is never substituted for receive order.
- A bridge fence uses the actual IBus variant-echo `Ping` contract, then the
  same marker, and returns a typed admission token containing connection,
  revocation, owner, activation and lineage generations. Session-bus and IBus
  `Sequence`s are never compared. The caller must revalidate the token before
  and after its authority-bearing operation.

### Planned staged module surface

Reducer additions in `context_admission.rs`:

- `SettledWordState` and a settlement method binding owner/header/tail/lineage;
- exact-tail `seal_source` validation and stale-owner no-write behavior;
- split `open_factory_request` / `bind_factory_target` operations;
- source-free begin/reply/marker/consume activation yielding an
  `ActivationGrant` with `UnknownStart` and no text payload;
- `AdmissionToken` capture/revalidation for callback/bridge callers.

New `context_admission/adapter.rs`:

- authenticated bootstrap values and narrow match-rule construction;
- owned joined observer with explicit cancellation/drop behavior;
- one-shot native or compatibility acquisition with one absolute deadline;
- callback header rendezvous begin, settlement forwarding and token
  revalidation;
- CreateEngine begin/bind helpers and the bounded variant-Ping marker fence.

The adapter owns no text, key queue, retry loop, worker thread, generic actor or
second reducer. Runtime callers remain responsible for tail copying only after
a transfer grant and for all engine/Shared/atomic/layout effects.

### Proof plan and honest limits

Retain the existing 19/19 reducer, ordered-merge, rendezvous and sender/budget
tests. Add controlled actual-zbus-5.15 Unix p2p cases for method header
correlation, unbound factory path binding, native and compatibility source-free
activation through Get-plus-marker, authenticated bootstrap fields, observer
cancellation/stream drop, bounded burst draining, and variant Ping-plus-marker
token fencing. Controlled peers may route explicit bus-style headers but do not
claim to be the deployed IBus daemon. Unsupported zbus or protocol APIs will be
reported instead of replaced by a fake sequencing test.

Still outside this slice: production task spawning/lifecycle ownership,
installed IBus compatibility, runtime callback integration, actual tail copy,
C01-C30/H proofs, client/physical restoration, resource/RSS measurements,
independent review, release/install/push and graph refresh.

## Adapter implementation and controlled proof (2026-09-06)

The admitted pre-code slice is now implemented and frozen outside the
repository at `/home/ubu/.cache/lay/td121-prewire-7iGwHB/`. The exact-source
handoff is
`/home/ubu/.cache/lay/td121-prewire-7iGwHB/FROZEN_STAGE_MANIFEST.md`.
Repository source, Cargo files, README and graph artifacts were not changed;
`runtime_authority_changed=false`.

### What was tested

- Six new pure reducer regressions cover source-free initial `UnknownStart`,
  different-field fallback through the same receipt-plus-marker contract,
  authoritative `WordScope` settlement and exact-tail sealing, stale revision
  refusal, old-owner no-write behavior, unbound Factory refusal, and admission
  token invalidation after lineage/owner/revocation changes.
- Five controlled Unix p2p tests use actual zbus 5.15.0 messages,
  `MessageStream` subscriptions and the production `ordered_stream::Join`.
  They cover bootstrap `GetNameOwner`/global mode/nested GlobalEngine variant,
  compatibility Get-plus-marker, native marker with zero Get, variant-echo
  Ping-plus-marker token fencing, actual callback rendezvous, CreateEngine
  stamp before later target binding, cancellation wake/drop, and a 32-callback
  bounded burst.
- Compilation exposed and fixed the zbus 5.15 `MatchRule::build()` return shape,
  a nested `Value` lifetime, and a mutable stream-borrow lifetime. The observer
  now races stream readiness against a dedicated cancellation event, so
  cancellation wakes a blocked `process_next()` before the streams are dropped.
- Review of the new owner transition exposed a real reducer defect: late
  callbacks from the consumed source owner could revoke the current target.
  Old-owner key/focus/disable/seal calls now return refusal without changing the
  current owner, lineage or revocation generation.

### Measured facts and receipt

The guarded remote run used exact zbus `5.15.0`, dedicated 20-CPU profile,
`CARGO_BUILD_JOBS=20`, `RUST_TEST_THREADS=1`, and the guarded shared target.
Result: **30/30 passed** in 0.04 seconds: 25 reducer/merge/rendezvous cases and
5 controlled adapter p2p cases.

- Local immutable log:
  `/home/ubu/.cache/lay/td121-proof-local-x3YPura4/td121-adapter-test-zbus-5.15.0.log`
- Log SHA-256:
  `b56a1d7ec2d5b3f465e07c37d192a0bf154b971c787e437c759f108779e84300`
- Exact proof archive:
  `/home/ubu/.cache/lay/td121-proof-local-x3YPura4/td121-proof.tar`
- Archive SHA-256:
  `de0ca6a875fcd86a9bdd4838615c2bd7e65b5387145cee253d1696ea3bf6be4a`
- Remote tree and log:
  `/home/e/projects/td121-proof-XU19ZyCn/`

The proof-only Cargo manifest enables `bus-impl` so the controlled peer can
assign and route explicit bus-style headers; that feature is not required by the
adapter's production code. The controlled peer is not a deployed IBus daemon,
so this result proves adapter compilation and the declared transport contracts,
not installed compatibility or client restoration.

### What was not tested and verdict scope

Still untested: repository/runtime wiring, production observer task ownership,
real IBus bootstrap against the installed daemon, engine/Factory/Shared/atomic/
layout consumers, actual tail copy, C01-C30/H, positive physical restoration,
resource/RSS and production latency, independent review, release, install and
push. Quality and deployed behavior remain `UNKNOWN`; no production authority
changed. The next wiring step must copy only the hashes in the frozen manifest
and rerun proof if any staged source changes.
