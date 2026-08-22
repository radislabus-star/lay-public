# IME Backend Atomic Frame V2

Date: 2026-08-20
Status: revised paper architecture; isolated Slices 1 and 2 proved; GNOME/Mutter production owner selected on paper; V22 implementation preflight READY; implementation not started
Runtime authority changed: false
Installed runtime touched: false

## 1. Decision

The selected route is:

```text
physical input event
-> client freezes exact downstream profile, event lease and event identity
-> one ProcessKeyEventAtomic request
-> IBus daemon validates the complete record and creates one daemon envelope
-> one atomic engine RPC gives Lay the same complete record before decision
-> Lay returns one typed effect proposal in that RPC reply, never mutation signals
-> daemon validates/seals the proposal or discards it as a whole
-> client validates the complete frame before any downstream request
-> one capability-specific downstream transaction owner submits the frame
-> handled disposition is derived only after refusal or submission is known
-> visible postcondition, not engine signal emission, authorizes learning
```

The frame is returned in the same D-Bus method reply. There is no second
`PostProcessKeyEvent` property fetch, sealed-frame store, stale take or duplicate
take state.

The downstream owner is not generic. V1 admits only an adapter whose native
protocol can submit every operation in the frame as one transaction. A callback
is an implementation mechanism, not capability evidence.

The typed validator built in Slice 1 remains useful, but it is not allowed to
capture anonymous engine signals. Existing `CommitText`, `DeleteSurroundingText`
and preedit signals carry no event or transaction identity; a collector cannot
prove that such a signal belongs to the active atomic call. The atomic engine
method therefore returns the bounded effect vector directly. The daemon feeds
that vector through the proved validator before forming the client reply.

## 2. Facts That Changed The V1 Design

Source inspection established four facts.

1. `IBusInputContext::delete-surrounding-text` has a `void` return type. The
   libibus layer therefore cannot observe the boolean result that the GTK
   adapter obtains from `GtkIMContext::delete-surrounding`.
2. Even if that boolean were preserved, GTK applies delete and commit through
   two synchronous signals. A crash between them can expose a deleted prefix.
   A successful delete callback is not a transaction receipt for a following
   commit callback.
3. Wayland input-method v2 is different. `delete_surrounding_text`,
   `commit_string` and `set_preedit_string` modify pending state, and one
   `commit(serial)` applies the pending state in a specified order. This is a
   real downstream transaction boundary.
4. Lay currently can emit committed-tail mutation from `SetSurroundingText`.
   That callback is outside `ProcessKeyEvent`; it cannot participate in an
   input-event frame and must only record a deferred intent.

Evidence owners:

- `/home/ubu/projects/ibus-lay-atomic-proof/src/ibusinputcontext.c`
- `/home/ubu/projects/ibus-lay-atomic-proof/client/gtk3/ibusimcontext.c`
- `/home/ubu/projects/ibus-lay-atomic-proof/client/wayland/ibuswaylandim.c`
- `/home/ubu/projects/ibus-lay-atomic-proof/client/wayland/input-method-unstable-v2-client-protocol.h`
- `/home/ubu/projects/lay-l1-exact-peak-search/src/bin/lay_ibus_engine/ibus_interface.rs`

## 3. Rejected Alternatives

### 3.1 Generic sequential libibus batch

```text
sealed frame
-> emit delete-surrounding-text
-> emit commit-text
```

Rejected. The frame is complete in memory, but the external effects remain two
independent callbacks. Completeness of input material is not atomicity of
output.

### 3.2 Delete boolean followed by commit

Rejected. It prevents commit after an explicit delete refusal, but it does not
prevent process death after a successful delete and before commit.

### 3.3 Two-call ProcessKeyEvent then TakeFrame

Rejected. It creates fetch failure, stale identity, duplicate take, retained
sealed-frame state and another synchronous D-Bus round trip. None is necessary
when the complete bounded vector can be returned in the original method reply.

### 3.4 Universal capability bit

Rejected. Commit-only, GTK callbacks, Wayland v1, Wayland v2, Qt replacement
events, terminal input and Electron/X11 are not equivalent mutation surfaces.
Capability is an exact adapter/profile contract, never a global IBus claim.

### 3.5 Late capability rejection

Rejected. Lay changes its internal composition and tail state while processing
the event. If unsupported output is discovered only after engine return, native
replay and Lay state can diverge. The downstream operation profile must reach
Lay through client capabilities before the event is processed.

## 4. Protocol Shape

V2 adds a new method and leaves legacy `ProcessKeyEvent` unchanged:

```text
ProcessKeyEventAtomicV1(
    keyval,
    keycode,
    modifiers,
    input_event_identity,
    backend_atomic_capability_record,
    backend_atomic_capability_digest,
    previous_transaction_receipt
) -> AtomicProcessReplyV1
```

Conceptual reply:

```text
AtomicProcessReplyV1 {
    disposition,
    transaction_id,
    input_event_identity,
    input_context_identity,
    focus_epoch,
    engine_identity,
    downstream_capability_identity,
    ordered_effect_vector,
    effect_vector_digest,
}
```

The first pure slice froze an insufficient request type, `uuuttay`. That type
carried only an epoch and digest, so neither daemon nor Lay could inspect the
capability that the digest allegedly named. It is retained as failed design
evidence and must not be integrated.

The corrected client-to-daemon method input has D-Bus signature
`uuut((uuayuuyu)(tttttbay))ay(ytay)`:

```text
u    keyval
u    keycode
u    modifiers
t    input_event_identity
((uuayuuyu)(tttttbay)) backend atomic capability record
ay   aggregate capability-record digest (exactly 32 bytes)
(ytay) previous transaction receipt
```

The previous receipt is `(result, transaction_id, effect_vector_digest)`. For
`None`, result and transaction are zero and the byte array is empty. For every
non-`None` result, transaction is non-zero and the digest is exactly 32 bytes.
This acknowledges the previous frame on the next event without a second fetch
RPC and without retaining the previous effect vector in the daemon.

Daemon-to-engine is a separate atomic method with signature
`uuu(ttttttay)((uuayuuyu)(tttttbay))(ytay)`:

```text
u u u       keyval, keycode, modifiers
(ttttttay)  transaction, event, input-context, daemon-focus, engine,
             capability epoch and capability-record digest
((...)(...)) the exact client capability record
(ytay)       validated previous transaction receipt
```

The engine reply is exactly `(ya(yv))`: one proposal disposition and one typed
effect vector. `ProcessKeyEventAtomicV1` must emit zero legacy mutation signals.
Only the daemon can turn this proposal into `AtomicProcessReplyV1`.

Method outputs have D-Bus signature `yttttttaya(yv)ay`; the corresponding GLib
reply tuple type is `(yttttttaya(yv)ay)`:

```text
y       disposition
t       transaction_id
t       input_event_identity
t       input_context_identity
t       focus_epoch
t       engine_identity
t       downstream_capability_epoch
ay      downstream_capability_digest (exactly 32 bytes)
a(yv)   ordered_effect_vector
ay      effect_vector_digest (exactly 32 bytes)
```

Every numeric identity is non-zero. `engine_identity` is a daemon-local engine
instance epoch, not an engine name. The capability digest binds the complete
frozen capability record; its separate epoch rejects reuse within one input
context. The complete tuple is returned by the original method invocation. The
old nested-return limitation does not require a second property read for a new
method.

The client validates the complete normal-form variant, count, operation schema,
identity echoes, capability identity and digest before issuing the first native
protocol request.

## 5. Typed Operation Vocabulary

The legacy queue encodes delete and forwarded keys as comma-separated text and
encodes one preedit update as two queue entries. V2 does not reuse that wire
shape.

Each semantic operation is one typed variant:

```text
CommitText(text)
DeleteSurrounding(offset_chars, length_chars)
SetPreedit(text, cursor_begin, cursor_end, mode)
HidePreedit
```

The semantic tags are stable, but `HidePreedit` has two deliberately distinct
wire encodings on the two protocol legs:

```text
tag  operation          Lay -> IBus engine leg    IBus -> Shell/Mutter leg
1    CommitText         v contains s               v contains s
2    DeleteSurrounding  v contains (iu)            v contains (iu)
3    SetPreedit         v contains (suuu)           v contains (suuu)
4    HidePreedit        v contains b=false          v contains ()
```

The IBus engine decoder accepts only `b=false` for tag 4 and re-encodes the
semantic effect as `()` for the Shell/Mutter frame. The outer-frame decoder
accepts only `()`. Neither decoder accepts both representations on one leg.
This strict translation exists because Rust `zvariant` cannot construct GLib's
empty tuple as the engine proposal payload; `b=false` is a typed transport
marker, not an additional semantic value.

Tags `0` and `5..255` are reserved and rejected. Text is plain UTF-8 without
`IBusText` attributes and is bounded to 4096 bytes per effect. Empty commit and
empty set-preedit payloads are rejected. Delete is the exact committed suffix
form only: `length_chars` is `1..4096` and `offset_chars == -length_chars`.
This deliberately excludes cursor-bearing and non-suffix deletion from V1.
Set-preedit cursor bounds are Unicode-character offsets satisfying
`begin <= end <= text_length`; mode is exactly
`IBUS_ENGINE_PREEDIT_CLEAR (0)` or `IBUS_ENGINE_PREEDIT_COMMIT (1)`.

V1 explicitly excludes:

```text
ForwardKeyEvent
terminal erase characters
layout mutation
synthetic key output
arbitrary repeated commit or delete operations
mutation emitted outside the active atomic event
```

Admitted canonical vectors are deliberately small:

```text
[CommitText]
[SetPreedit | HidePreedit]
[CommitText, SetPreedit | HidePreedit]
[DeleteSurrounding, CommitText]
[DeleteSurrounding, CommitText, SetPreedit | HidePreedit]
```

The implementation may admit a smaller subset per adapter. It must reject the
whole frame before output for duplicate, reordered, unsupported or malformed
operations. The first proof does not inherit the legacy limit of 30 arbitrary
entries; its semantic-operation bound is the maximum admitted canonical vector.
The frozen bound is therefore three effects, even when a client advertises a
smaller bound. An advertised value of zero or greater than three is invalid.

The effect digest is SHA-256 over this byte sequence:

```text
"IBusAtomicEffectVectorV1\0"
|| "a(yv)\0"
|| normal-form serialized bytes of ordered_effect_vector
```

The decoder accepts only an exact normal-form top-level type, exact payload
types, exact digest lengths, a matching digest and one of the admitted vectors.
It never coerces, truncates, reorders or ignores an unknown record.

## 6. Capability And Event-Lease Contract

```text
BackendAtomicCapabilityRecordV1 {
    profile: {
        protocol_version,
        adapter_kind,
        adapter_build_contract_digest,
        downstream_transaction_kind,
        event_supported_effect_mask,
        maximum_semantic_effect_count,
        guarantee_flags,
    },
    lease: {
        adapter_instance_identity,
        input_context_identity,
        capability_epoch,
        focus_lineage_identity,
        native_transaction_epoch,
        surrounding_snapshot_present,
        surrounding_snapshot_digest,
    },
}
```

Its exact GVariant type is `((uuayuuyu)(tttttbay))`. Both digests are exactly
32 bytes. The adapter build field is a build-contract identity generated from
the audited adapter/profile implementation, not a claim that a self-reported
hash cryptographically attests a running binary.

The daemon has an immutable admission table keyed by protocol version, adapter
kind, build-contract digest and transaction kind. The table provides the
maximum mask/count and required guarantees. The event mask/count may narrow
that profile but can never enlarge it. This table admits a known inline record;
it is not a mutable digest registry and is never queried by another RPC.

The initial admitted guarantee flags require: one native commit, zero native
requests on refusal, input-context and focus binding, native-transaction epoch
binding, and exact surrounding-snapshot binding for every delete operation.
An event that lacks a snapshot must remove `DeleteSurrounding` from its event
mask. Unknown flags and unknown profile records are rejected.

`input_context_identity` is a random non-zero client context identity pinned by
the authorized `BusInputContext` on its first valid atomic call.
`adapter_instance_identity` is pinned to the same context. A reused capability
epoch must carry byte-identical normal-form record bytes and digest; a lower
epoch is stale. A higher epoch may change the lease.

The surrounding digest is SHA-256 over the exact normal-form `(suu)` tuple of
UTF-8 text, cursor character offset and anchor character offset, with no Unicode
normalization. The client, daemon and Lay cache this digest when surrounding
text changes. Lay may propose a delete only when its exact cached digest equals
the lease; the client repeats the comparison before issuing its first native
request.

The aggregate capability digest is SHA-256 over:

```text
ASCII "IBusBackendAtomicCapabilityRecordV1" || NUL
|| ASCII "((uuayuuyu)(tttttbay))" || NUL
|| normal-form serialized bytes of the complete record
```

The client freezes the complete normal-form value before calling the daemon.
The daemon recomputes the digest, admits the profile, pins the dynamic lease,
and passes the same bytes and digest inline to Lay before decision. Lay may
prepare only an edit representable by the event mask and exact snapshot.

Capability is fail-closed:

- absent, stale or unknown capability means legacy/native handling;
- capability for one adapter build cannot authorize another build;
- support for commit-only cannot authorize delete plus commit;
- support for Wayland input-method v2 cannot authorize Wayland v1;
- a snapshot boolean without an exact matching snapshot digest authorizes no delete;
- capability loss during the call invalidates the complete frame.

## 7. Client Matrix

### GNOME Shell / Mutter text-input v3

Selected current-host multi-effect candidate. The capability is owned by the
current `ClutterInputFocus`, not inferred from the IBus client name. A new
synchronous Mutter composite API must validate one complete canonical vector,
the exact focus lineage, native transaction epoch and surrounding snapshot
before it emits any protocol event. `MetaWaylandTextInputFocus` then emits the
complete `delete_surrounding_text`, `commit_string` and optional preedit update
followed by exactly one text-input-v3 `done` without returning to the main loop.

The existing pair of `Clutter.InputMethod.delete_surrounding()` and
`Clutter.InputMethod.commit()` calls is not this owner: those calls enqueue two
independent `CLUTTER_IM_*` events and can be separated by a forced `done`.

### Wayland input-method v2

Future independent multi-effect candidate. The adapter prevalidates all character-to-byte
offsets and protocol state, emits pending delete/commit/preedit requests, then
emits exactly one `zwp_input_method_v2_commit(serial)`.

```text
death before protocol commit -> pending state is not applied
death after protocol commit  -> one complete transaction was submitted
```

No individual libibus mutation callbacks may run for an admitted frame.

### GTK 2/3/4

Commit-only can be evaluated because one commit signal is one mutation effect.
Generic `DeleteSurrounding + CommitText` is not admitted. A boolean delete
result does not remove the crash point between two application callbacks.

Active-composition autocorrect should therefore collapse clear-preedit plus
commit into one committed text effect whenever GTK is the downstream owner.

### Qt

Potential candidate: one `QInputMethodEvent` can carry commit text and a
replacement range. This requires an independent ibus-qt source and physical
receipt. It is not promoted by the Wayland proof.

### Terminal and X11/XIM

No multi-effect replacement capability is assumed. Terminal erase and
forwarded-key routes remain excluded from V2.

### Electron/Chromium

Classify by the actual active backend. Ozone Wayland may inherit a proved
Wayland transaction; X11 or GTK callback paths do not. Product name alone is
not capability evidence.

## 8. Collector State Machine

```text
IDLE
  -> VALIDATING(event, capability-record, focus-epoch)
      -> ENGINE_PROPOSAL_PENDING
      -> VALIDATING_PROPOSAL
      -> RETURNED_FRAME_REPLY
      -> DISCARDED_NATIVE_SAFE
      -> DISCARDED_LINEAGE_TERMINATED
  -> IDLE
```

Rules:

1. One input context has at most one atomic call and one bounded pending receipt.
2. The validator is reset before the engine call starts.
3. The atomic engine reply is the only proposal source. Any legacy mutation
   signal during the call aborts the event and is never captured as evidence.
4. Engine error, unsupported operation, count overflow, malformed operation,
   focus-epoch change or capability change discards every collected operation.
5. Engine `handled=false` with mutation operations is invalid and discarded.
6. The complete vector is serialized directly into the one method reply; no
   sealed frame remains available for a later take.
7. Legacy clients and legacy `ProcessKeyEvent` retain their old route until a
   separate migration gate removes it.
8. In atomic-only Lay mode, commit/delete emitted as engine signals are
   suppressed. `SetSurroundingText` may update observation state, settle an
   exact postcondition and enqueue an intent, but cannot mutate text.
9. `finish` moves directly from collecting to a returned reply and clears all
   owned text/effects before returning. There is no retained `SEALED_REPLY`
   object and therefore no take, retry or stale-frame state.
10. After the first malformed, unsupported or overflowing proposal record, the
    validator is terminally aborted; sealing emits a zero-effect disposition.
11. The daemon retains only pending transaction identity and digests, never the
    sealed frame or effect vector. The next atomic request must settle that
    identity before a new engine decision. Focus loss discards it fail-closed.

## 9. Disposition And Receipt Semantics

A boolean is insufficient for focus loss and uncertain downstream submission.
V2 uses explicit dispositions:

```text
NativeUnhandled
FrameReady
ConsumedNoEffect
FocusLineageTerminated
```

The client derives the public handled result:

```text
NativeUnhandled                         -> handled=false
FrameReady + refused before any request -> handled=false
FrameReady + submitted one transaction  -> handled=true
ConsumedNoEffect                        -> handled=true
FocusLineageTerminated                  -> no cross-focus replay
submission state uncertain              -> handled=true, no retry, no learning
```

The backend boundary can prove `SubmittedAtomic`, not `AppliedExact`. Native
protocol submission has no universal application acknowledgement. Exact
visible surrounding-text postcondition may later promote the event to
`ObservedExact` and authorize learning.

```text
BackendAtomicReceiptV2 {
    transaction_id,
    input_event_identity,
    adapter_identity,
    effect_vector_digest,
    result:
        RefusedZeroEffect
      | SubmittedAtomic
      | ConsumedNoEffect
      | FocusLineageTerminated
      | SubmissionUncertainNoRetry,
}
```

Engine signal emission, daemon frame sealing and D-Bus reply delivery are not
success receipts.

The receipt transport is deliberately not a second frame call. The client
stores only the prior transaction ID, effect digest and terminal result, then
includes them in its next atomic request. The daemon verifies them against its
single pending identity and gives the result to Lay before the next decision.
Lay may commit or roll back speculative per-event state, but `SubmittedAtomic`
still cannot authorize learning. If no next event arrives, exact surrounding
observation may settle the postcondition; focus loss simply discards the
speculative state.

## 10. Lay Operation Consequences

The current families divide as follows:

```text
active composition commit/autocorrect   commit-only after preedit collapse
stuck-tail suffix completion            commit-only
committed-tail full replacement         requires atomic replace capability
Space correction of committed text      requires atomic replace capability
double-Shift rollback                    requires atomic replace capability
cursor-bearing committed-tail edit       excluded from V2
SetSurroundingText auto-undo              must become deferred intent
```

Unsupported replacement does not fall back to terminal erase, forwarded keys,
old committed-tail mutation or sequential delete/commit. It remains a
suggestion or native-unhandled event according to the owning UI route.

## 11. Proof Ladder

### A. Pure frame codec and collector

- typed encode/decode/normal-form parity;
- input-event, context, focus, capability and effect-digest identity parity;
- every malformed or unsupported vector fails before output;
- operation-bound overflow discards the whole frame;
- atomic abort never falls through to a legacy signal.

### B. Isolated daemon plus fake engine

- kill after every emitted-operation prefix;
- engine error, cancellation and connection loss;
- concurrent legacy and atomic calls;
- direct engine proposal reply and zero legacy mutation signals;
- full capability-record byte parity across both D-Bus hops;
- known profile, unknown profile and profile-narrowing cases;
- exact surrounding-snapshot digest match and mismatch;
- prior-receipt match, mismatch, absence and duplicate replay;
- focus and capability epoch changes;
- exactly one complete reply or no frame;
- zero legacy mutation signals on every failed atomic event.

### C. Fake downstream transaction owner

- validation failure issues zero native requests;
- kill before native commit applies zero pending effects;
- kill after native commit exposes one complete vector;
- handled=true never exists without a terminal disposition receipt.

### D. GNOME Shell / Mutter text-input-v3 composite owner

- exact focus/profile lease reaches Lay before decision;
- stale focus, stale native epoch, multiple active resources, an unknown vector
  or invalid surrounding bounds issue zero frame effects;
- all canonical effects and exactly one `done` are emitted in one synchronous
  `MetaWaylandTextInputFocus` call;
- kill before `done` exposes no applied text-input-v3 mutation;
- one `done` produces one client replacement event;
- no legacy libibus mutation callback or separate Clutter IM event runs for the
  admitted frame.

Wayland input-method v2 remains a separate future adapter proof. It cannot
inherit this result and is not the current GNOME host route.

### E. Lay shadow integration

- capability reaches Lay before decision;
- intended edit and returned effect vector match exactly;
- `SetSurroundingText` cannot mutate;
- no old mutation fallback;
- no learning before `ObservedExact` or another admitted receipt.

### F. Physical promotion

Each client class passes independently. Required gates remain:

```text
partial external effect                                      0
duplicate or cross-focus native replay                        0
unsupported-client committed-tail mutation                    0
success or learning without admitted receipt                  0
atomic RPC p99 / max                                  <=2 / <8 ms
integrated hot p99 / max                              <=5 / <8 ms
installed runtime and package hashes                    explicit gate
```

## 12. Implementation Slices

1. Freeze V2 types, GVariant schema, operation grammar and dispositions.
2. Add a pure codec/validator module with exhaustive unit and fault tests.
3. Add the corrected capability-record codec and invalidate the old
   `uuuttay` request contract while preserving proved reply/effect semantics.
4. Add both `ProcessKeyEventAtomicV1` hops in isolated IBus. The engine hop
   returns `(ya(yv))`; no engine mutation signal may participate.
5. Add client-side full-reply validation without applying operations.
6. Build fake engine and fake transaction-owner killpoint proofs in a private
   `dbus-run-session`.
7. Implement first the GNOME Shell / Mutter text-input-v3 composite owner.
   Keep Wayland input-method v2 as an independent future profile.
8. Add capability propagation and transactional speculative state to Lay, and
   defer all observation-callback
   mutation.
9. Integrate in shadow mode, compare exact frames and receipts, then run
   physical gates.
10. Promote only proved client/operation pairs. Keep every unsupported pair
   fail-closed.

Every slice receives a fresh implementation preflight. No installed IBus, Lay
binary, daemon or desktop session is changed by the paper decision.

## 13. Superseded V1 Evidence

`ime-backend-atomic-receipt-v1-2026-08-20.md` and its V1-V3 implementation
preflights remain retained. Their design baseline hash is intentionally changed
by the supersession notice, so the old `READY_TO_IMPLEMENT` receipt cannot be
used to authorize source edits under V2.

## 14. Structural Result

The first V3 route packet was retained with `VETO`. It incorrectly drew the
authorization owner as a producer of frame evidence and connected the live
Wayland mutation owner directly to the proof route.

The corrected V4 design packet passed the earlier abstract route. The
source-observed capability analysis then found that it did not model either the
missing second capability hop or the two anonymous legacy mutation paths.

The retained current-source V5 packet verifies all 26 source markers and returns
`VETO` for the expected legacy mechanism:

```text
Lay delete signal -> daemon forward -> Wayland delete -> commit(serial)
Lay commit signal -> daemon forward -> Wayland commit -> commit(serial)
execution paths to external effect                         2
mutation owners                                            2
```

The first revised V5 design was also retained with `VETO` because its
observation route incorrectly started at the mutation owner. The corrected V6
starts observation at the submission receipt and passes:

```text
verdict                                      PASS
issues / warnings                             0 / 0
execution/authority/observation/proof        separated
nodes / edges                                  14 / 21
source evidence verified                     false
implementation correctness proven            false
implementation preflight still required      true
```

Receipts:

- `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_DESIGN_2026-08-20/route-design-v3-receipt.json`
- `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_DESIGN_2026-08-20/route-design-v4-receipt.json`
- `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_DESIGN_2026-08-20/current-observed-v5-receipt.json`
- `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_DESIGN_2026-08-20/route-design-v5-veto-receipt.json`
- `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_DESIGN_2026-08-20/route-design-v6-receipt.json`

The PASS authorizes preparation of a new V2 implementation preflight only. It
does not authorize source edits, build, installation, daemon restart or live
input testing.

## 15. Slice 1 Pure Codec/Collector Result

Slices 1 and 2 of the original implementation list passed on 2026-08-21,
scoped only to the pure typed reply/effect schema, codec and validator logic.

Before code, the first Slice 1 preflight was invalidated because its proposed
reply omitted `downstream_capability_epoch` while the method input carried that
epoch. The retained V1 receipt therefore cannot authorize the implemented
schema. The corrected V2 preflight pins `(yttttttaya(yv)ay)` and passed with
`READY_TO_IMPLEMENT`, `safe_to_implement=true`, and zero blockers.

The later source-observed analysis invalidates only the request-side
`(uuuttay)` contract and the plan to capture anonymous engine signals. The
proved reply type, effect grammar, digest, bounds and pure validator results
remain scoped evidence. They do not authorize either D-Bus hop until the new
capability codec and direct engine-reply contract pass a fresh preflight.

Implemented isolated source:

- `/home/ubu/projects/ibus-lay-atomic-proof/src/ibusatomicframeprivate.h`
- `/home/ubu/projects/ibus-lay-atomic-proof/src/ibusatomicframe.c`
- `/home/ubu/projects/ibus-lay-atomic-proof/src/tests/ibus-atomic-frame.c`
- private source/test registration in the two existing `Makefile.am` files

What was tested:

```text
remote host                         e@192.168.3.94, 20 logical CPUs
compiler                            gcc 11.4.0, gnu11, warnings as errors
registered tests                    12
tag sequences length 0..4           341
accepted canonical sequences          8
rejected sequences                  333
normal result                     12/12 PASS
normal elapsed / max RSS          0.01 s / 3,200 KiB
ASan+UBSan+LSan result             12/12 PASS, zero findings
sanitized elapsed / max RSS       0.03 s / 20,480 KiB
```

One retained intermediate sanitizer run found 70,205 leaked bytes in 1,117
allocations. Per-test isolation reduced it to one extra hard `GVariant`
reference per encoded test reply. Removing the pre-increment before `@a(yv)`
and passing the collector vector as a floating child removed the complete leak;
wire bytes and state-machine semantics did not change.

Measured preservation:

- local and remote SHA-256 match for all five compiler inputs;
- `bus/inputcontext.c`, `src/ibusinputcontext.c` and
  `src/ibusinputcontext.h` retain their pinned hashes;
- installed IBus daemon, libibus, GTK modules and Lay daemon retain their
  pinned hashes;
- no live process was restarted or signalled;
- the exact GLib development package was extracted only under the 13 MiB
  remote task sandbox; no host package was installed.

Not tested and not claimed:

- a full IBus Autotools build;
- `ProcessKeyEventAtomicV1` or any D-Bus integration;
- direct engine reply, client reply validation or killpoints;
- Wayland/GTK transaction submission;
- Lay capability propagation, physical clients, latency gates or deployment.

Runtime authority changed: **no**. The installed runtime remains Lay `1.0.33`
with the existing IBus build.

Exact receipt:

- `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_SLICE1_2026-08-21/slice1-proof-receipt.json`
- SHA-256 `f03ac8a9f71553854b215b6de1b5bbc70e6971bb673857df2b87383cb8dd7ca2`

The next allowed step is a separate implementation preflight for the corrected
inline capability record and direct isolated `ProcessKeyEventAtomicV1` hops.
Slice 1 PASS does not authorize that code.

## 16. 2026-08-21 Max Analysis Verdict

Three transports were compared.

```text
full record inline on both hops   selected
mutable preregistration          rejected: stale/focus/restart cleanup state
digest-only distributed registry rejected: version skew and no dynamic lease
```

The selected hybrid has one immutable daemon admission table but carries the
complete profile and dynamic lease on every event. Measured normal-form GVariant
body sizes are 218 bytes for a first client request and 250 bytes when carrying
a 32-byte prior receipt; daemon-to-engine bodies are 260 and 292 bytes. The
maximum two-hop request material is therefore 542 bytes before D-Bus headers.
This is measured serialization size, not a latency result. Latency remains
unclaimed until the isolated round-trip proof.

The architecture now closes the first shared failure mechanisms:

- no digest can name fields that daemon or Lay cannot inspect;
- no delete can target an unbound surrounding snapshot;
- no anonymous engine signal can enter an atomic event frame;
- no unknown profile can enlarge an admitted operation set;
- no sealed frame, take API or retryable output remains;
- no submitted receipt can be promoted to exact learning evidence.

Runtime authority changed: **no**. No installed binary, daemon, IBus process,
input device or desktop session was touched by this analysis.

## 17. Slice 2 Capability-Hop Implementation Preflight

The first paper-only Slice 2 implementation preflight passed on 2026-08-21.
It checked the corrected inline capability record, both isolated D-Bus hops,
the direct typed engine proposal and the prior-receipt carry contract before
any source edit.

What was tested:

- exact bytes and modes for 20 source, contract, receipt and installed-runtime
  baselines;
- 18 forbidden effect classes against the reused atomic codec, using
  Python-compatible static veto expressions;
- all six state-machine transitions and their terminal failure states;
- eight producer/consumer identity contracts;
- eleven invariants and 25 named post-edit or fault-injection tests.

Measured preflight result:

```text
verdict                         READY_TO_IMPLEMENT
safe_to_implement              true
blockers                       0
baseline checks                20
preserved artifacts            10
identity contracts              8
invariants                     11
mapped tests                   25
runtime comparisons             0 (out of scope for the isolated slice)
```

Not tested and not claimed:

- no capability-hop source was implemented by this preflight;
- no compiler, remote build or private D-Bus proof was run;
- no serialization latency or end-to-end round-trip latency was measured;
- no Wayland, GTK, Lay integration, physical input or runtime quality was
  tested;
- no installation or deployment authority was granted.

Exact V1 evidence:

- manifest:
  `docs/structural_gates/preflights/LAY_IME_BACKEND_ATOMIC_FRAME_V2_SLICE2_CAPABILITY_HOPS_V1_2026-08-21.json`
- manifest SHA-256:
  `e450444075e4536caf15c749d4b4e9c1b323e3bc51b953f76d2f79eb0bc4bee8`
- receipt:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_SLICE2_2026-08-21/implementation-preflight-receipt-v1.json`
- receipt SHA-256:
  `72425e6617af434148836e18bbe47346020410f790cb934a232571027c11d435`

Appending this measured result changes the pinned architecture-paper bytes and
therefore deliberately consumes the V1 authorization. No source edit may start
until a V2 manifest pins this revised paper and the retained V1 evidence, and a
fresh V2 receipt independently returns `READY_TO_IMPLEMENT` with
`safe_to_implement=true` and zero blockers.

Runtime authority changed: **no**. Installed hashes and live process identities
remain preservation baselines only; they were not mutated or restarted.

## 18. V3 Exact Wire ABI Freeze

The V2 implementation preflight exposed a paper defect during source mapping:
it fixed tuple types but did not assign every byte-valued enum, guarantee bit,
admission constant or digest domain. No C source was changed under that receipt.
The V2 receipt is retained as evidence of an incomplete assumption and cannot
authorize implementation. This section is the minimum complete V3 wire ABI.

### 18.1 Exact type strings

```text
capability record       ((uuayuuyu)(tttttbay))
prior receipt           (ytay)
client request          (uuut((uuayuuyu)(tttttbay))ay(ytay))
engine envelope         (ttttttay)
engine request          (uuu(ttttttay)((uuayuuyu)(tttttbay))(ytay))
engine proposal         (ya(yv))
daemon/client reply     (yttttttaya(yv)ay)
effect vector           a(yv)
surrounding snapshot    (suu)
```

The strings above include the outer GLib tuple used for each D-Bus body. No
alias, nested return wrapper, dictionary, maybe type or second fetch method is
admitted.

### 18.2 Numeric enums

Final daemon dispositions retain the proved Slice 1 values:

```text
0  NativeUnhandled
1  FrameReady
2  ConsumedNoEffect
3  FocusLineageTerminated
```

The engine proposal byte has a smaller independent vocabulary:

```text
0  ProposalNativeUnhandled       requires an empty vector
1  ProposalFrameReady            requires one canonical non-empty vector
2  ProposalConsumedNoEffect      requires an empty vector
3..255                           invalid
```

`FocusLineageTerminated` is daemon-owned and can never be self-reported by an
engine proposal.

The prior-receipt result byte is:

```text
0  None
1  RefusedZeroEffect
2  SubmittedAtomic
3  ConsumedNoEffect
4  FocusLineageTerminated
5  SubmissionUncertainNoRetry
6..255 invalid
```

`None` is exactly `(0, 0, empty-ay)`. Every non-`None` result requires a
non-zero transaction ID and an exact 32-byte effect-vector digest. Effect tags
remain `1 CommitText`, `2 DeleteSurrounding`, `3 SetPreedit`, `4 HidePreedit`;
preedit modes remain `0 Clear`, `1 Commit`.

### 18.3 Guarantee bits

```text
bit 0 / 0x01  SINGLE_NATIVE_TRANSACTION
bit 1 / 0x02  ZERO_NATIVE_REQUESTS_ON_REFUSAL
bit 2 / 0x04  INPUT_CONTEXT_IDENTITY_BOUND
bit 3 / 0x08  FOCUS_LINEAGE_BOUND
bit 4 / 0x10  NATIVE_TRANSACTION_EPOCH_BOUND
bit 5 / 0x20  DELETE_REQUIRES_EXACT_SNAPSHOT
known mask    0x3f
```

Unknown bits are invalid. The first profile requires the complete `0x3f` mask;
flags are guarantees, not optional feature requests. Effect-mask bits remain
`Commit=0x01`, `Delete=0x02`, `SetPreedit=0x04`, `HidePreedit=0x08`, with known
mask `0x0f`. An event mask and count may narrow an admitted profile only when
at least one canonical effect vector remains representable.

### 18.4 Private proof profile

Slice 2 admits no production client. Its only known profile is compiled in
under `IBUS_ATOMIC_PRIVATE_PROOF_PROFILE`; a normal build has an empty admission
table and rejects every atomic profile.

```text
protocol_version                    1
adapter_kind               0xffff0001 = 4294901761
downstream_transaction_kind
                           0xffff1001 = 4294905857
profile maximum effect mask       0x0f
profile maximum effect count          3
required guarantee flags          0x3f
```

The two high-range kind values are private-proof values, not allocations for a
Wayland, GTK, Qt, terminal or Electron adapter. The exact build-contract digest
is SHA-256 over these 133 bytes:

```text
ASCII "IBusBackendAtomicPrivateProofProfileV1" || NUL
|| ASCII "protocol=1;adapter=4294901761;transaction=4294905857;max-mask=15;max-count=3;required-flags=63"
```

Expected digest:

```text
a463a8dc3a1a7676009e30e3b5f70c1cef2334fd28285ace1ef663f09e154260
```

The private-proof binary is sandbox-only and non-installable. A future
production Wayland profile requires its own audited adapter source, different
kind and build-contract digest, physical receipt and fresh promotion preflight.

### 18.5 Digest grammars and absence

All serialized variants below must already be exact normal form. Digests are
computed without Unicode normalization, field coercion or re-encoding.

```text
capability-record digest =
    SHA-256(
      ASCII "IBusBackendAtomicCapabilityRecordV1" || NUL
      || ASCII "((uuayuuyu)(tttttbay))" || NUL
      || exact normal-form capability-record bytes)

surrounding-snapshot digest =
    SHA-256(
      ASCII "IBusBackendAtomicSurroundingSnapshotV1" || NUL
      || ASCII "(suu)" || NUL
      || exact normal-form (UTF-8 text, cursor_chars, anchor_chars) bytes)

effect-vector digest =
    SHA-256(
      ASCII "IBusAtomicEffectVectorV1" || NUL
      || ASCII "a(yv)" || NUL
      || exact normal-form ordered effect-vector bytes)
```

The adapter build-contract digest and aggregate capability digest are always
exactly 32 bytes. Snapshot absence is represented only as `false` plus an empty
byte array; snapshot presence is `true` plus exactly 32 bytes. A false snapshot
with a non-empty digest, a true snapshot with a non-32-byte digest, or any event
mask containing `DeleteSurrounding` without a present daemon-matching snapshot
is invalid.

The daemon caches the digest derived from the latest admitted
`SetSurroundingText` text/cursor/anchor tuple. It compares the lease at admission
and again after engine return. A changed tuple invalidates the complete proposal.

### 18.6 Identity and epoch rules

The first admitted call pins non-zero `adapter_instance_identity` and
`input_context_identity` to the authorized `BusInputContext`. They cannot
change during that object's lifetime. `focus_lineage_identity` is pinned for
one daemon focus epoch and may change only after an observed focus transition.

For admitted events, all three client counters are strictly increasing:

```text
input_event_identity
capability_epoch
native_transaction_epoch
```

Equal values are duplicate replay, lower values are stale, and neither is
processed as a new event. The complete capability record and its aggregate
digest are frozen before the call. The daemon focus generation, engine-slot
generation and transaction ID are independent non-zero daemon-local values;
they change on their owning transitions and are never derived from pointers.

### 18.7 Pending receipt state

The daemon retains no effect vector. It retains at most one tuple of prior
disposition, transaction ID and effect digest, and only after `FrameReady` or
`ConsumedNoEffect`.

```text
prior FrameReady:
  accepts RefusedZeroEffect | SubmittedAtomic |
          FocusLineageTerminated | SubmissionUncertainNoRetry

prior ConsumedNoEffect:
  accepts ConsumedNoEffect | FocusLineageTerminated
```

The receipt must match transaction and digest exactly and is consumed once
before a new engine decision. A duplicate when no receipt is pending rejects
that event. Missing, mismatched or incompatible settlement while one is pending
returns `NativeUnhandled`, invokes neither engine method, discards pending
metadata and poisons only the atomic lineage until the next real focus cycle.
Legacy `ProcessKeyEvent` remains unchanged and available; the daemon never
calls it as same-event fallback. Focus loss, engine replacement or context
destruction discards pending metadata and every cached event lease.

The isolated fake engine is stateless. Binding a future stateful Lay engine's
speculative state to this poison/reset transition is explicitly a later Lay
integration gate and is not claimed by Slice 2.

### 18.8 Exact refusal matrix

```text
wrong D-Bus body type or unusable zero identity
  -> InvalidArgs, zero engine calls, zero effects

well-typed unknown profile, stale event, stale capability or stale native epoch
  -> NativeUnhandled, zero engine calls, zero effects

receipt mismatch while pending
  -> NativeUnhandled, zero engine calls, zero effects, atomic lineage poisoned

missing or mismatched surrounding snapshot
  -> NativeUnhandled, zero engine calls, zero effects

engine error, cancellation, timeout or malformed proposal
  -> NativeUnhandled, zero effects

legacy engine mutation signal during an atomic call
  -> suppress signal, discard proposal, NativeUnhandled, atomic lineage poisoned

focus or engine generation changes while the engine call is pending
  -> FocusLineageTerminated, zero effects

valid ProposalNativeUnhandled
  -> NativeUnhandled, zero effects

valid ProposalConsumedNoEffect
  -> ConsumedNoEffect, zero effects, one pending receipt identity

valid ProposalFrameReady
  -> FrameReady, exact bounded vector, one pending receipt identity
```

No refusal invokes legacy `ProcessKeyEvent`, captures a mutation signal, keeps
a sealed frame, exposes a take method or issues a second frame RPC.

### 18.9 V3 implementation boundary

V3 may implement only the pure capability/receipt/snapshot codecs, the isolated
InputContext and EngineProxy methods, signal suppression for the pending atomic
call, and a fake client/engine proof under a private D-Bus session. It may build
only on the checked remote host.

It does not authorize a production admission profile, client-side mutation,
Wayland requests, Lay speculative state, installation, service restart, live
session-bus access, physical input or runtime quality claims. Source work may
start only after a new Slice 2 V3 implementation preflight pins this section
and independently returns `READY_TO_IMPLEMENT` with zero blockers.

Runtime authority changed: **no**.

## 19. Slice 2 Isolated Implementation Result

Slice 2 was implemented and closed in the isolated IBus source tree on
2026-08-21. The implementation follows the V4 capability-hop preflight and
does not admit a production client profile.

Implemented surface:

- strict capability, prior-receipt, surrounding-snapshot, effect-vector and
  engine-proposal codecs;
- direct client-to-daemon and daemon-to-engine `ProcessKeyEventAtomicV1` hops;
- receipt-first admission, strictly increasing event/capability/native epochs,
  snapshot binding, single flight, poison and focus-reset semantics;
- suppression of all six legacy mutation-signal families during an atomic
  call;
- normal-build empty admission and one compile-time private proof profile;
- separate fake-client and fake-engine D-Bus connections, including real
  engine-disconnect coverage;
- legacy `ProcessKeyEvent` parity for exact boolean replies, exact engine-call
  cardinality and observed `CommitText -> ForwardKeyEvent` signal order.

Measured final proof:

```text
normal codec                              18 / 18 PASS
private codec                             18 / 18 PASS
normal real-hop matrix                      1 / 1 PASS
private real-hop matrix                     1 / 1 PASS
V4 test IDs mapped to evidence            32 / 32 PASS
observed-source route markers             36 / 36 PASS
observed-source route issues / warnings      0 / 0
local/remote changed-input hashes            8 / 8 equal
installed-runtime hashes                     5 / 5 equal to V4
leftover task-owned proof processes                 0
```

The pinned target set (`libibus-1.0`, `ibus-daemon`, codec test and real-hop
test) built with `-Werror` on the checked 20-CPU remote host. The final
incremental target timings were `0.16 s` for normal and `0.57 s` for private,
with measured peak command RSS of 4,000 KiB and 54,820 KiB respectively.
These are build-command measurements, not runtime or per-key latency claims.

One broader `make all-recursive` attempt reached disabled portal code and
failed because the intentionally minimal sandbox has no `gio-unix` headers.
That log is retained. The failure did not alter the pinned Slice 2 targets,
did not trigger a local build or package installation and did not weaken a
gate. The subsequent exact target set passed in both configurations.

The final remote observation reported:

```text
host                                  e-MEGA-MINI-M1-13th
logical CPUs                                           20
load 1 / 5 / 15 min                      1.59 / 1.62 / 1.66
available memory                              24,681,712 KiB
unrelated L1 metrics service                         active
unrelated L1 metrics service PID                    342773
```

Installed bytes remain equal to the V4 baselines for `/usr/bin/ibus-daemon`,
`libibus-1.0`, both GTK input modules and the installed `lay-daemon`. The sole
live Lay and IBus processes are PID `1971272` and PID `2076194`; both started
on 2026-08-19, before the V4 manifest timestamp
`2026-08-21 02:47:27 +0300`, and remained alive after the proof. No installed
file, service, input device, live session bus or desktop process was modified
or restarted.

Not tested and not claimed:

- no production Wayland, GTK, Qt, terminal or Electron adapter/profile;
- no Lay capability propagation or speculative-state poison binding;
- no physical keyboard or application behavior;
- no serialization or production end-to-end latency;
- no installation, live-session integration, deployment or runtime-quality
  promotion.

Exact final evidence:

- observed-source route contract:
  `docs/structural_gates/preflights/LAY_IME_BACKEND_ATOMIC_FRAME_SLICE2_OBSERVED_V10_2026-08-21.json`
- observed-source route contract SHA-256:
  `c31f615e06d8621e682a072eb076f98563951c98fb7f19142cc3791df032034d`
- observed-source route receipt:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_SLICE2_2026-08-21/slice2-observed-v10-receipt.json`
- observed-source route receipt SHA-256:
  `0379f3fef6ad441b4a78c73a89bd60f5771bc99b1ec7e0385417db4c4da2c8ad`
- implementation receipt:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_SLICE2_2026-08-21/slice2-implementation-receipt-v1.json`
- implementation receipt SHA-256:
  `a0952422827d4f193784df53f953e5eab10cbdf2583133979ef512563759b647`
- final normal raw logs:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_SLICE2_2026-08-21/proof-final-normal-20260821/`
- final private raw logs:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_SLICE2_2026-08-21/proof-final-private-20260821/`
- retained failed experiments:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_V2_SLICE2_2026-08-21/failed-experiments/`

Slice 2 isolated implementation verdict: **PASS**. Runtime authority changed:
**no**. Deployment performed: **no**. A production profile and deployment
remain a separate preflight and physical-proof stage.

## 20. Production Owner Selection Blocker

The first read-only production-backend observation was performed after the
isolated Slice 2 PASS on 2026-08-21. It invalidates one scheduling assumption,
not the atomic protocol: implementing IBus `ibus-wayland` first would not prove
or improve the currently observed WeChat route.

Measured facts:

```text
desktop session                                  GNOME / Wayland
installed ibus-wayland executable                         absent
live Lay engine PID                                     2076619
WeChat main PID                                         3471530
WeChatAppEx direct-libibus PID                           3471726
recent Lay capability classes                  caps 41 and caps 9
capability observations with client build identity              0
```

The WeChat main process inherits `QT_IM_MODULE=ibus` and `XMODIFIERS=@im=ibus`
and maps X11/XCB libraries, but at the observation point it mapped neither
`libibus` nor either system Qt IBus plugin. Its `WeChatAppEx` child mapped the
system `libibus`, Wayland libraries and X11/XCB libraries, but did not map the
system Qt5 or Qt6 IBus plugin. Therefore neither application name, session
type, environment variable, surrounding-text capability nor loaded `libibus`
is evidence for one exact downstream transaction owner.

The current Lay trace further exposes two capability classes, `caps=41` with
surrounding text and `caps=9` without it, but omits the `FocusInId` client and
object-path values. The classes cannot be bound to an adapter build from the
retained trace. Adding an atomic profile by guessing this binding would repeat
the rejected universal-capability design.

The production problem must now be split by both operation class and adapter:

```text
active-composition commit-only
  -> may need only one commit effect
  -> still requires an exact adapter/build proof

committed-tail replacement
  -> requires delete plus commit in one native transaction
  -> cannot inherit authority from generic libibus callbacks

system Qt5 / system Qt6 plugin
  -> QInputMethodEvent replacement is a candidate owner
  -> not proved to own the observed WeChat editing surface

direct libibus consumer
  -> exact client callback implementation is unknown
  -> no generic multi-effect production profile

IBus Wayland v2
  -> protocol is a valid future transaction owner
  -> executable is absent and route is not active on this host
```

Verdict: **BLOCKED_BEFORE_CODE** for production-profile selection. Slice 2
remains PASS. The next paper decision must select independently proved profiles
for the real client classes and keep unsupported classes on legacy or
suggestion-only behavior. No code, installed file, process, service, keyboard
or runtime authority changed during this observation.

Exact receipt:

`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/observed-backend-v1.json`

## 21. GNOME/Mutter Production Owner Decision

The Section 20 blocker is resolved at design scope, not at implementation or
deployment scope. The selected current-host owner is a new synchronous
composite-frame API in Mutter for `MetaWaylandTextInputFocus`, reached through a
modified GNOME Shell IBus bridge. Generic libibus callbacks, the current pair of
Clutter IM events and application-name heuristics remain rejected.

```text
physical key
-> current ClutterInputFocus mints an exact profile and one focus lease
-> GNOME Shell single-flight event owner
-> ProcessKeyEventAtomicV1
-> daemon and Lay validate the same lease and return one complete frame
-> GNOME Shell validates the reply against the still-current focus lease
-> one synchronous Mutter composite call
-> MetaWaylandTextInputFocus emits the complete text-input-v3 event group
-> exactly one done
-> SubmittedAtomic receipt
-> later matching surrounding text may become ObservedExact
```

This is a three-component implementation boundary: isolated IBus, GNOME Shell
and Mutter. It is still one runtime mutation route. GNOME Shell transfers the
profile and serializes events; it does not become a second mutator. Mutter is
the sole native mutation owner.

### 21.1 Measured installed baseline

```text
GNOME Shell package                         50.1-0ubuntu1.1
GNOME Shell libshell Build ID              36a08634e0b6c9bbde7645a0bc1d558f901df623
inputMethod.js resource SHA-256            f835a1eae1d1ff7340d7a9017ca18e76a5437e921d32fe9b3c94904c020c2fb0
Mutter package                             50.1-0ubuntu2.2
libmutter Build ID                         25d36850030c8329363d10f1fc32e90baf7e14c1
IBus package                               1.5.34~rc2-1
installed runtime changed                  no
service or desktop process restarted       no
```

GNOME Shell creates one IBus context named `gnome-shell`. Its
`vfunc_focus_in()` replaces `_currentFocus` but reuses that context. Therefore
`gnome-shell`, `caps=41`, `caps=9`, a Wayland session and a loaded libibus
library cannot identify the current focus implementation.

The installed source exposes exactly two relevant `ClutterInputFocus`
subclasses:

```text
MetaWaylandTextInputFocus   external Wayland text-input-v3 surface
ClutterTextInputFocus       GNOME Shell internal ClutterText widget
```

The profile must be returned by a typed `ClutterInputFocus` virtual capability.
GNOME Shell must not compare a GType name, application name, executable path,
capability bit pattern or environment variable.

### 21.2 Why the current queue is not a transaction

The current route is:

```text
IBus delete-surrounding-text callback
-> GNOME Shell delete_surrounding()
-> one CLUTTER_IM_DELETE
-> one clutter_event_put()

IBus commit-text callback
-> GNOME Shell commit()
-> one CLUTTER_IM_COMMIT
-> another clutter_event_put()
```

`clutter_event_put()` locks the global `GAsyncQueue` for one push and releases
it immediately. There is no lock, event identity or batch marker covering both
pushes. The callbacks are also separate IBus signal deliveries.

`MetaWaylandTextInputFocus` schedules `done` at
`CLUTTER_PRIORITY_EVENTS + 1`, explicitly grouping only IM events already seen.
It forcibly flushes a pending `done` on:

- focus reset;
- an unfiltered key event;
- selected button or touch transitions on the focused surface.

One valid counterexample is therefore:

```text
CLUTTER_IM_DELETE
-> pending delete + deferred done
-> unfiltered key or focus-reset event
-> forced done applies deletion
-> CLUTTER_IM_COMMIT
-> a second done applies insertion
```

Even if GNOME Shell called the two existing methods synchronously from one new
libibus callback, another queue producer could still place the flushing event
between the two separately locked pushes. Queue adjacency is not an admitted
guarantee.

### 21.3 Native transaction semantics

Wayland text-input-v3 provides the required external boundary. Its
`preedit_string`, `delete_surrounding_text` and `commit_string` events modify
pending state. The client applies them only when it receives `done`, in this
specified order:

```text
replace old preedit
-> delete surrounding text
-> insert committed text
-> install new preedit
```

Qt 6 confirms the material implementation: its text-input-v3 adapter stores
delete and commit in pending fields, then creates one `QInputMethodEvent` with
one replacement range from `done()`. This Qt observation is supporting evidence
for that client, not authority for every text-input-v3 implementation. The
protocol contract is the cross-client authority and each promoted physical
client still receives its own gate.

### 21.4 Required Mutter API

The first implementation must add one bounded synchronous API conceptually
equivalent to:

```text
query_atomic_profile(current_focus)
  -> profile | unsupported
  -> focus_lineage_identity
  -> native_transaction_epoch
  -> cached surrounding snapshot digest, if present

submit_atomic_frame(
    expected_focus_lineage,
    expected_native_transaction_epoch,
    expected_surrounding_digest,
    canonical_effect_vector)
  -> RefusedZeroEffect
   | SubmittedAtomic
   | FocusLineageTerminated
```

The concrete C/GI shape may use a boxed record or a normal-form `GVariant`, but
it must preserve the already frozen `a(yv)` bytes. It must not expose a
`begin/delete/commit/end` public sequence that callers can abandon halfway.

The same single-flight boundary must also close an unhandled key before the
next RPC starts. The current `notify_key_event(event, false)` only places a copy
on the Clutter queue, so using it and immediately launching the next atomic
request would permit the next reply to overtake native replay. The first design
therefore requires one focus-bound synchronous replay operation in Mutter:

```text
replay_native_key(expected_focus_lineage, copied_original_event)
  -> ReplayedToSameFocus
   | FocusLineageTerminated
```

It marks the copied event as input-method replay, validates the same focus
lineage, dispatches it to that focus before returning and cannot be refiltered
through IBus. Reusing the current unacknowledged queue-only replay is forbidden
for the atomic single-flight route.

The base `ClutterInputFocus` default is unsupported. The first multi-effect
implementation exists only on `MetaWaylandTextInputFocus`. Before any native
event, it validates all of the following:

1. the focus object is still the current focus;
2. the focus lineage equals the frozen lease;
3. the native transaction epoch is current and one-shot;
4. exactly one active text-input-v3 resource owns the first profile;
5. the complete vector is one admitted canonical vector;
6. every UTF-8 character-to-byte conversion is in bounds;
7. every delete uses the exact cached surrounding snapshot;
8. no unresolved pending commit or delete belongs to an earlier frame.

Password, PIN, hidden-text, sensitive-data and terminal purposes never advertise
committed-tail delete authority. A content-purpose or hint change increments the
capability epoch before another event can be admitted.

Validation failure emits zero frame effects. A valid call completes any earlier
non-mutating pending `done` boundary, rechecks the lease, then runs without
returning to the GLib main loop:

```text
optional old-preedit clear
-> delete_surrounding_text
-> commit_string
-> optional new preedit_string
-> one done(serial)
```

The implementation must track the kind of pending IM state. It may flush a
known preedit-only or client-state `done` before the frame. If an earlier
commit/delete is pending or the pending kind is unknown, it refuses the new
frame with zero effects. It must never silently merge two event receipts into
one `done`.

If GNOME Shell or Mutter dies before `done`, text-input-v3 clients have no
protocol authority to apply the pending delete or commit. If the complete
ordered stream including `done` is queued, the result is `SubmittedAtomic`.
That is still not `AppliedExact`; only a later matching surrounding snapshot
can establish the visible postcondition.

### 21.5 Focus, event and cancellation state machine

Mutter owns `focus_lineage_identity` and `native_transaction_epoch`. GNOME Shell
owns the IBus context instance and input-event sequence. The daemon and Lay may
validate these values but may not mint replacements for them.

```text
NO_FOCUS
  -> FOCUSED(profile, focus-lineage, native-epoch, surrounding-digest?)
      -> QUEUED(event-id)
      -> RPC_IN_FLIGHT(event-id, frozen-lease)
          -> REPLY_VALIDATING
              -> NATIVE_SUBMITTED(receipt)
              -> REFUSED_ZERO_EFFECT
              -> LINEAGE_TERMINATED
      -> FOCUSED(next-native-epoch, pending-receipt?)
  -> NO_FOCUS
```

Rules:

1. One GNOME Shell context has at most one atomic RPC in flight.
2. GNOME Shell takes an owned copy of each intercepted event. Later physical
   events wait in original order in a queue bounded at 64.
3. A reply may act only on the exact event, context, focus lineage, capability
   epoch, native epoch and surrounding digest frozen for that call.
4. Focus out, focus replacement, capability narrowing, queue overflow or the
   hard RPC deadline terminates the lineage and cancels the call.
5. Events from a terminated lineage are never replayed into a new focus.
6. Overflow or deadline termination may consume the affected event with no
   text effect; it must not trade data loss for cross-focus output. Physical
   promotion requires zero such terminations in the stress denominator.
7. A refused frame can become native-unhandled only before any native frame
   request, while the same focus lineage is still current, and through the
   focus-bound synchronous replay operation.
8. Once submission is possible but its terminal result is uncertain, the event
   is handled, never retried and never used for learning.
9. The next event carries the prior terminal receipt. No second frame fetch,
   sealed-frame cache or legacy retry exists.
10. The next queued RPC starts only after either synchronous frame submission or
    synchronous native replay reaches a terminal result.
11. Focus loss discards speculative Lay state and any unsettled learning state.

The queue bound is a failure-containment bound, not a latency allowance. The
promotion target remains one event normally in flight and an empty queue after
each response.

### 21.6 Profile matrix

```text
focus/client class                         commit/preedit   delete+commit
MetaWaylandTextInputFocus, patched v3      candidate        selected candidate
ClutterTextInputFocus                      candidate        not first profile
direct GTK 2/3/4 IBus                      separate proof   unsupported
direct Qt 5/6 IBus                         separate proof   unsupported
X11/XIM, terminal, unknown                 native/legacy    unsupported
IBus Wayland input-method v2               future profile   future profile
```

`unsupported` means no committed-tail mutation authority. The result remains a
suggestion or a native-unhandled event. It never falls through to terminal
erase, synthetic Backspace, forwarded replacement keys or the old sequential
delete/commit route.

An atomic-capable GNOME Shell context always uses the atomic method, including
commit-only events. Per-context legacy support may remain for unmodified IBus
clients, but Lay must remove delete-plus-commit authority from every legacy
context. There is no same-event fallback from atomic to legacy mutation.

The adapter build-contract digest names the audited combined contract across
the exact IBus, GNOME Shell and Mutter implementations. It is not runtime
cryptographic attestation. Deployment separately pins installed package bytes,
Build IDs and rollback packages.

### 21.7 Active-composition-first is a separate slice

The source observation found the systemic cause of frequent committed-tail
replacement. In `process_pressed_key()`, a managed printable character with an
empty composition buffer calls `commit_managed_passthrough_char()`. The buffer
therefore remains empty, and normal Space correction must delete text already
owned by the application.

The separate optimization is:

```text
first admitted alphabetic character
-> insert into active composition buffer
-> publish preedit
-> update the same bounded L1.1 -> L2 -> L3 -> L4 field on every character
-> Space selects one authorized surface
-> one commit of selected-word + Space
```

This reduces multi-effect traffic and makes the normal correction path
commit-only. It is not part of the transport slice because it changes visible
typing semantics, candidate timing, focus-reset behavior and learning. It needs
its own route contract, implementation preflight and physical proof across:

- first-character visibility and every-character refresh;
- Tab acceptance and candidate rejection by continued typing;
- Backspace, arrows, punctuation, Enter and Space;
- no duplicated or consumed Space;
- focus loss and preedit reset/commit modes;
- terminals and clients without a proved preedit profile;
- latency per printable event and on Space.

It does not remove the need for the composite frame. Double-Shift rollback
after a committed autocorrection, a late correction and a full committed-tail
replacement still require exact delete-plus-commit authority. Such a rollback
also requires the corrected surface to be `ObservedExact`; `SubmittedAtomic`
alone does not prove what the application currently contains.

### 21.8 Consequence analysis

Quality and authority:

- L1.1/L2/L3/L4 ranking and lattice membership do not change in the transport
  slice.
- The verifier still authorizes the intended edit; the new client admission
  can only narrow it to the current profile.
- Unsupported clients lose automatic committed-tail replacement rather than
  receiving unsafe authority. Suggestions remain available.
- A submitted receipt cannot mint learning evidence. Only `ObservedExact` or an
  independently admitted postcondition can do that.

Latency and resources:

- the already measured maximum two-hop request material remains 542 bytes
  before D-Bus headers;
- focus/profile and snapshot digests are cached on focus or surrounding-state
  changes, not recomputed by scanning the full document on every key;
- the composite native operation is bounded to three semantic effects and one
  active text-input resource in the first profile;
- steady state adds one in-flight record, one prior receipt and a normally empty
  bounded event queue per context;
- required gates remain atomic RPC p99 `<=2 ms`, integrated hot p99 `<=5 ms`
  and max `<8 ms`; queue wait is included in integrated latency.

Concurrency and invalidation:

- package reload, IBus reconnect, focus change, capability change and
  surrounding update invalidate the frozen lease;
- no result from an old model generation, focus or adapter instance can mutate;
- no background worker may become an output owner;
- no pending frame survives process restart.

Maintenance and rollback:

- this design patches three upstream components, so it is more expensive to
  maintain than active-composition-only;
- the cost is accepted because strict committed-tail atomicity is impossible in
  generic callbacks and the current GNOME route has no external
  input-method-v2 owner;
- builds must be versioned local packages based on exact Ubuntu source versions,
  never ad-hoc replacement of individual system libraries;
- rollback must restore the complete IBus + GNOME Shell + Mutter package set;
- a GNOME Wayland deployment requires a user-approved session restart and an
  out-of-session recovery path. No automatic IBus or GNOME Shell restart is
  authorized by this paper decision.

### 21.9 Proof and promotion gates

The implementation ladder is now:

```text
Mutter pure frame validator and focus-epoch tests
-> fake MetaWaylandTextInputFocus killpoint proof
-> real text-input-v3 protocol harness
-> GNOME Shell single-flight and cancellation proof
-> isolated IBus + Shell + Mutter round trip
-> Lay shadow frame parity
-> physical applications by focus profile
-> live authority only after every required gate passes
```

Required denominators include:

```text
malformed/stale/refused frames with native effect                  0
partial delete without matching commit and one done                0
duplicate submission or hidden retry                               0
cross-focus replay                                                 0
legacy committed-tail mutation on unsupported profile              0
learning from SubmittedAtomic without ObservedExact                0
deadline or queue-overflow lineage terminations in stress          0
atomic RPC p99 / max                                      <=2 / <8 ms
integrated hot p99 / max                                  <=5 / <8 ms
```

The first production-owner design route initially returned `VETO` because its
observation route started at the mutation owner. The corrected route begins at
the submission receipt and keeps execution, authority, observation and proof
separate. It returns `PASS` with one execution path, one authorization owner and
one mutation owner. This structural PASS authorizes only an implementation
preflight.

Exact evidence:

- source analysis:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/gnome-mutter-owner-analysis-v2.json`
- design route:
  `docs/structural_gates/preflights/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_ROUTE_V2_2026-08-21.json`

Current verdict: **DESIGN_SELECTED_PREFLIGHT_REQUIRED**. Runtime authority
changed: **no**. No production code, installed package, service, input method,
desktop process or physical keyboard route was changed.

### 21.10 Mutter Slice 3A implementation preflight, 2026-08-21

Measured facts:

- the production-owner route was rerun against the current V2 contract and
  returned `PASS`: 14 nodes, 20 edges, no issues and no warnings;
- `atomic_frame_submitted` has one authorization owner
  (`shell_reply_admission`) and one mutation owner
  (`mutter_text_input_v3_frame`);
- `frame_refused` has the same authorization owner and one separate mutation
  owner (`mutter_focus_bound_replay`), so refused native replay cannot race a
  queued generic callback;
- all 19 measured baseline entries matched after correcting the preregistered
  `clutter/clutter/meson.build` baseline to 10,244 bytes and SHA-256
  `d86af636054fd7518ea0c1b40063772577f5c8e315793887539e706805e8fb88`;
- implementation preflight V1 returned `BLOCKED_BEFORE_CODE` with 30 manifest
  blockers: 20 missing forbidden-effect scan bindings, five foreign scan
  names, four unsupported test kinds and one identity test that was not typed
  as parity;
- V1 is retained unchanged with its failing receipt. V2 repaired those paper
  bindings without removing a prohibition or changing a production baseline;
- V2 returned `READY_TO_IMPLEMENT`: 19 baseline checks, 21 forbidden effects,
  two reused-source checks with zero forbidden matches, four identity
  contracts, nine invariants, 18 tests and zero blockers.

What was not tested:

- no Mutter source was implemented or compiled;
- no local or remote build, sanitizer run, protocol harness or physical client
  proof was executed;
- no installed library, Lay binary, service, IBus process, GNOME Shell process,
  input device or desktop session was modified or restarted;
- implementation correctness, runtime behavior, latency and deployment
  authority remain unproven.

Exact evidence:

- V1 blocked receipt:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/mutter-slice3a-implementation-v1-receipt.json`
- V2 manifest:
  `docs/structural_gates/preflights/LAY_IME_BACKEND_ATOMIC_FRAME_MUTTER_SLICE3A_IMPLEMENTATION_V2_2026-08-21.json`
- V2 ready receipt:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/mutter-slice3a-implementation-v2-receipt.json`
- final post-recording V3 manifest:
  `docs/structural_gates/preflights/LAY_IME_BACKEND_ATOMIC_FRAME_MUTTER_SLICE3A_IMPLEMENTATION_V3_2026-08-21.json`
- final post-recording V3 receipt:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/mutter-slice3a-implementation-v3-receipt.json`

The V3 gate must pin the final bytes of this section and return both
`verdict=READY_TO_IMPLEMENT` and `safe_to_implement=true` before Slice 3A code
starts. Its scope is only the isolated pure Mutter API, validator, fake-focus
proof and Meson registration declared by the manifest. Runtime authority
changed: **no**.

### 21.11 Mutter Slice 3A implementation proof, 2026-08-21

Implemented source scope:

- added the pure `ClutterInputAtomicFrame` decoder and validator with the exact
  `a(yv)` wire schema, SHA-256 domain, eight canonical vectors and fixed bounds
  of three effects, 4,096 text bytes and 4,096 deleted characters;
- added profile, focus-lineage, capability-epoch, native-epoch and frozen-lease
  state to `ClutterInputFocus` and `ClutterInputMethod`;
- added one-shot synchronous submit and same-lease replay with `OPEN`,
  `IN_FRAME`, `FINALIZING` and `REPLAY_ONLY` state transitions;
- invalidated leases on focus replacement, focus reset, surrounding state,
  content hints, content purpose and preedit-capability changes;
- retained the default focus profile as unsupported and added no production
  `MetaWaylandTextInputFocus` implementation;
- registered one isolated fake-focus executable. The final executable contains
  twelve semantic tests, including explicit legacy commit/delete/preedit
  dispatch parity, all five capability-invalidation classes and three
  reentrancy cases.

Measured facts:

- all compilation ran on `e@192.168.3.94`, which exposed 20 logical CPUs; no
  local Mutter build directory or local Mutter compiler process was created;
- the isolated Ubuntu 26.04 toolchain used Meson `1.10.1`, Ninja `1.13.2`,
  GLib `2.88.0` and GCC `15.2.0`; `dpkg-checkbuilddeps` returned clean;
- normal Meson setup configured 403 targets with tests enabled, production
  optional features enabled, docs/profiler/installed-tests disabled and
  `debugoptimized` build type;
- the selected normal target built `221/221`; the final fail-fast incremental
  rebuild completed `4/4` with no compiler warning;
- registered normal proof: `12/12 PASS`, `1.04 s`, with
  `G_DEBUG=fatal-warnings`;
- independent sanitized setup used `b_sanitize=address,undefined`, debug build
  type and the same feature matrix; the selected target built `221/221` and
  the final fail-fast incremental rebuild completed `5/5`;
- direct sanitized proof: `12/12 PASS`, exit `0`, with ASan, UBSan and LSan all
  fail-fast and leak detection enabled; no sanitizer diagnostic was emitted;
- final normal executable is 123,176 bytes with SHA-256
  `3533b21493cef0ac53491b7a2516a6bc3bcf2b2b8e0a0231d40454091b46733c`;
- final sanitized executable is 230,688 bytes with SHA-256
  `a5358858c0d2d6de4e15424c456aa40e7c35eb288735bb2ec8355206ff999cb5`;
- normal and sanitized build directories occupy 48 MiB and 81 MiB. The remote
  rootfs occupies 2.1 GiB; 18 GiB remained free after proof;
- local and remote source trees compare byte-identical after final sync;
- `src/wayland/meta-wayland-text-input.c` remains exactly 32,623 bytes with
  SHA-256 `8a388a0be267c5d0744920f862e36ff09f595589573bbfc2efd53177a336f4cb`;
- installed Mutter, GNOME Shell, IBus and Lay hashes still equal the preflight
  baselines.

The first registered normal run exposed one fixture defect: the replay test
constructed a key event with a null source device. The fix was a general valid
fake keyboard device, not a runtime exception. A later legacy parity addition
first used public methods that require a global Clutter context; the final test
instead sends canonical IM events through `clutter_input_focus_process_event`.

Post-proof review then found two state-machine defects. A recursive completion
during `FINALIZING` could clear the active lease, and a submit/replay vfunc could
change focus while the caller retained only a borrowed pointer. The repair holds
a strong focus reference across each vfunc, rejects recursive completion without
mutating the outer transaction and rechecks state, active lease and current focus
after the vfunc. Three tests prove completion preservation and submit/replay
focus-lineage termination. A second review found that focus reset did not advance
the capability epoch. Reset now shares the same invalidation mechanism as
surrounding, hints, purpose and preedit capability, and one parameterized test
proves all five paths before any vfunc effect.

One Meson attempt timed out before the test binary started because the PRoot
wrapper lacked the host UID mapping. One later test-only compile failure was
followed by a stale binary run because that intermediate shell lacked `set -e`.
Both attempts are excluded. The final normal and sanitized commands use
`set -euo pipefail`; each successful build precedes its recorded test.

Sanitizer harness scope is explicit. Mutter's global Meson wrapper preloads
`libumockdev` before `libasan`, so it cannot start an ASan executable. PRoot is
ptrace-based, so LSan cannot inspect threads inside PRoot. The final sanitizer
proof therefore executes the same remotely built test binary directly through
the Ubuntu 26.04 dynamic loader and exact rootfs library path, without either
wrapper. This changes no tested code or sanitizer instrumentation.

What was not tested:

- no production focus advertises the atomic profile in Slice 3A;
- no `MetaWaylandTextInputFocus` native apply owner exists yet;
- no text-input-v3 protocol harness, GNOME Shell queue/cancellation path,
  patched IBus round trip, physical application or latency denominator ran;
- no claim is made about atomic RPC p99, integrated hot p99, queue overflow,
  deadline behavior or visible typing semantics;
- no package was built for installation and no installed file, service, IBus
  process, GNOME Shell process, input device or desktop session was changed or
  restarted.

Exact evidence:

- remote sandbox: `/home/e/lay-proof/mutter-slice3a-20260821`;
- normal setup/build/test logs:
  `/home/e/lay-proof/mutter-slice3a-20260821/logs/meson-normal-setup.log`,
  `/home/e/lay-proof/mutter-slice3a-20260821/logs/normal-final-build-v4.log`,
  `/home/e/lay-proof/mutter-slice3a-20260821/logs/normal-final-test-v4.log`,
  `/home/e/lay-proof/mutter-slice3a-20260821/logs/normal-final-test-detail-v4.log`;
- sanitized setup/build/test logs:
  `/home/e/lay-proof/mutter-slice3a-20260821/logs/meson-sanitized-setup.log`,
  `/home/e/lay-proof/mutter-slice3a-20260821/logs/sanitized-final-build-v3.log`,
  `/home/e/lay-proof/mutter-slice3a-20260821/logs/direct-sanitized-final-test-v3.log`;
- final source hash log:
  `/home/e/lay-proof/mutter-slice3a-20260821/logs/source-sha256-final-v3.txt`;
- implementation proof receipt:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/mutter-slice3a-implementation-proof-v1.json`.

Verdict: **SLICE3A_IMPLEMENTATION_PROOF_PASS** for the isolated pure Mutter
frame API, validator, focus/epoch state machine and fake-focus proof only.
Implementation correctness is proven only inside that scope. Deployment
authority: **false**. Runtime authority changed: **no**. Slice 3B must implement
and prove the real `MetaWaylandTextInputFocus` owner before any Shell, IBus,
package or desktop deployment work can begin.

### 21.12 Slice 3A replay-copy correction preflight, 2026-08-21

The Slice 3B source review found one mismatch between the proved API and the
paper contract. `clutter_input_method_replay_native_key()` passed the caller's
original key event directly to the focus vfunc. The proof checked key value,
keycode and timestamp, but did not prove that the replay carried
`CLUTTER_EVENT_FLAG_INPUT_METHOD`. A production `MetaWaylandTextInputFocus`
would therefore have to repair the event itself or risk a second input-method
filter pass. Both outcomes violate the single-owner contract.

The correction remains inside Slice 3A and changes no native Wayland owner:

```text
active same-focus lease
-> validate replay input is KEY_PRESS or KEY_RELEASE
-> construct one owned Clutter key event
   -> preserve type, existing flags, timestamp and source device
   -> preserve raw pressed/latched/locked modifiers
   -> preserve state, keysym, event code, keycode and Unicode value
   -> add CLUTTER_EVENT_FLAG_INPUT_METHOD
-> invoke exactly one focus replay vfunc synchronously
-> free the owned copy
-> recheck state, lease and current focus
-> consume the one-shot lease
```

A non-key event is an invalid replay request. After identity and current-focus
validation it closes the active lease, returns `RefusedZeroEffect` with
`INVALID_ARGUMENT`, invokes no replay vfunc and permits the next lease. A stale
or foreign lease retains identity-mismatch precedence. Recursive completion and
focus replacement during the vfunc retain the already proved `FINALIZING`
semantics.

The existing exact-replay test must additionally compare event type, complete
flags, source device, raw modifiers, state, event code and Unicode value. A new
negative test must prove that a non-key event reaches no vfunc and strands no
lease. Normal and ASan+UBSan+LSan proofs must be rebuilt remotely with
`set -euo pipefail`; stale binaries are never evidence.

Scope and claim boundary:

- allowed source edits: `clutter-input-method.c` and the isolated atomic-frame
  test only;
- `meta-wayland-text-input.c`, installed Mutter, GNOME Shell, IBus and Lay remain
  byte-identical;
- no profile is advertised and runtime authority remains false;
- the correction is accepted only after a fresh implementation preflight,
  normal proof, sanitized proof, architecture update and exact receipt.

### 21.13 Slice 3A replay-copy correction result, 2026-08-21

The focused V4 implementation preflight failed closed before source edits with
`source_veto_patterns_missing`: the reused test source had no local veto
pattern. V4 and its blocked receipt are retained. V5 added the missing
`/dev/input|uinput` veto and passed with `READY_TO_IMPLEMENT`,
`safe_to_implement=true` and zero blockers:

- manifest:
  `docs/structural_gates/preflights/LAY_IME_BACKEND_ATOMIC_FRAME_MUTTER_SLICE3A_REPLAY_COPY_V5_2026-08-21.json`;
- manifest SHA-256:
  `77d0a1a4f94d2c59dd9a0525a1eaacaf132e675f881b21564398a3be2a5c9b41`;
- receipt:
  `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/mutter-slice3a-replay-copy-v5-preflight-receipt.json`;
- receipt SHA-256:
  `301238a05ae817c21a11bf20fba43f9b2a407053b5d45d27104cb3341340b136`.

Implemented source scope:

- `clutter-input-method.c` now validates the active same-focus lease first,
  rejects non-key events, reconstructs one owned key event with all key identity
  fields preserved, adds `CLUTTER_EVENT_FLAG_INPUT_METHOD`, invokes exactly one
  synchronous focus vfunc, frees the copy and retains the existing post-vfunc
  lineage check;
- the isolated fake-focus proof compares copy identity, type, all flags, source
  device, raw pressed/latched/locked modifiers, state, keysym, event code,
  keycode, Unicode and timestamp;
- `/atomic-input/non-key-replay-zero-effect` proves `INVALID_ARGUMENT`, zero
  vfunc calls, consumed lease, stale-lease identity precedence and successful
  acquisition of a fresh lease.

Measured remote proof facts:

| Dimension | Result |
|---|---:|
| remote host | `e@192.168.3.94`, 20 logical CPUs |
| normal incremental build | `5/5`, no compiler warning |
| normal semantic proof | `13/13 PASS` |
| normal Meson wrapper | `1/1 PASS`, `1.80 s` |
| normal binary | `135456 B` |
| normal binary SHA-256 | `1970409ee7a7447b508131adccc8e7e3d7ee44d6d9a49717423a7354c470bc24` |
| sanitized incremental build | `5/5`, no compiler warning |
| ASan + UBSan + LSan proof | `13/13 PASS` |
| sanitizer diagnostics | `0` |
| sanitized binary | `262152 B` |
| sanitized binary SHA-256 | `490fba4bc776c15777d500a12e53d85b9468ecde87cb84a1fc6fac3c0ba0ad6d` |

Final changed-source identities:

- `clutter/clutter/clutter-input-method.c`: `28810 B`, SHA-256
  `00fdd4a758f2867b8ba7478db5c584bc09b515890204beff75871b72a9a7a40d`;
- `src/tests/wayland-atomic-input-frame-tests.c`: `46413 B`, SHA-256
  `52989a5f4eee08fc69be966d1754d40ea6da16d13320dab27757ad673a0e1b6e`.

Exact new logs:

- `/home/e/lay-proof/mutter-slice3a-20260821/logs/normal-replay-copy-v5-build.log`;
- `/home/e/lay-proof/mutter-slice3a-20260821/logs/normal-replay-copy-v5-test.log`;
- `/home/e/lay-proof/mutter-slice3a-20260821/logs/normal-replay-copy-v5-test-detail.log`;
- `/home/e/lay-proof/mutter-slice3a-20260821/logs/sanitized-replay-copy-v5-build.log`;
- `/home/e/lay-proof/mutter-slice3a-20260821/logs/direct-sanitized-replay-copy-v5-test.log`.

Not tested in this correction: a production `MetaWaylandTextInputFocus`, a real
text-input-v3 client, Shell or IBus integration, physical application typing,
hot-path latency, installable packages, desktop restart or rollback. The
Wayland owner source and installed Mutter, GNOME Shell, IBus and Lay bytes remain
at their frozen hashes. Runtime authority changed: **no**. Deployment authority:
**false**. Verdict: **SLICE3A_REPLAY_COPY_CORRECTION_PASS** inside the isolated
pure Mutter fake-focus proof only. Slice 3B remains the next required proof.

### 21.14 Mutter Slice 3B real Wayland owner contract, 2026-08-21

Slice 3B makes `MetaWaylandTextInputFocus` the first real native atomic-frame
owner. It does not patch GNOME Shell or IBus, does not install a Mutter package
and does not grant deployment authority. The proof target is one real
text-input-v3 client and one synchronous Mutter owner reached through the Slice
3A `ClutterInputMethod` lease API.

#### 21.14.1 Facts found before implementation

The existing `MetaWaylandTextInput` has one global `enabled` flag, separate
resource and focused-resource lists, pending client state, one committed
surrounding string and one undifferentiated `done_idle_id`. Legacy delete,
commit and preedit callbacks all defer `done`; client-state `commit()` also
defers it. Therefore `done_idle_id != 0` does not identify whether an earlier
native mutation is already pending. Treating every pending `done` as flushable
would allow an atomic frame to merge with an earlier legacy delete or commit.

The existing content-type fields are pending values and are reset after each
client `commit()`. They cannot authorize a later atomic profile. Slice 3B must
retain committed content hints and purpose separately. The existing surrounding
state also lacks a cached validity bit and exact digest; a profile cannot infer
those facts on every key.

Replay already has one suitable synchronous primitive:
`meta_wayland_seat_handle_event(text_input->seat, event)`. The Slice 3A owner
passes it an owned key copy carrying `CLUTTER_EVENT_FLAG_INPUT_METHOD`, so the
seat accepts the replay independently of physical-device identity and routes it
straight to the current Wayland keyboard handler without a second IM filter.

#### 21.14.2 Profile identity

The first production-profile vocabulary is fixed as:

```text
protocol_version                         1
adapter_kind                      0x00030001 = 196609
downstream_transaction_kind       0x00030002 = 196610
maximum effect mask                      0x0f
maximum effect count                        3
required guarantee flags                 0x3f
maximum admitted surrounding bytes      65536
```

The adapter build-contract digest is SHA-256 over exactly 162 bytes:

```text
ASCII "LayGnomeTextInputV3AtomicProfileV1" || NUL
|| ASCII "protocol=1;adapter=196609;transaction=196610;max-mask=15;max-count=3;required-flags=63;surrounding=v1;pending=v1;hide-marker=b0"
```

Expected digest:

```text
1475b580ff9600ccfa84e43eb9bd50a61fa0bca88b11d33a104877e36b39adac
```

Mutter, the GNOME Shell fixture, IBus and Lay must pin this exact profile
vocabulary. A source or semantic change that alters the adapter contract
requires a new digest and invalidates an older consumer admission; it is not
silently compatible. A normal build admits exactly one production digest.

#### 21.14.3 Exact surrounding snapshot

The committed snapshot cache is refreshed only when a focused client commits a
new surrounding state. It is valid only when:

- text length is at most 65,536 bytes;
- the complete string is valid UTF-8;
- cursor and anchor are in byte bounds and both lie on UTF-8 boundaries;
- cursor equals anchor for delete authority.

The digest grammar is:

```text
ASCII "MutterTextInputV3SurroundingSnapshotV1" || NUL
|| uint32 little-endian text_byte_length
|| uint32 little-endian cursor_byte_offset
|| uint32 little-endian anchor_byte_offset
|| exact UTF-8 text bytes
```

For `рабатает`, byte length 16, cursor 16 and anchor 16, the expected digest is
`d29997c354a1ddd4541e8d54b13d29982a3aaaf129fddc90c42f55b7b46d0cad`.
The cache is recomputed on client surrounding updates, never by scanning the
document in the key hot path.

#### 21.14.4 Profile admission

`query_atomic_profile()` returns unsupported unless every condition below is
true before a lease is minted:

1. the focus object is currently focused by one `ClutterInputMethod`;
2. `text_input->enabled` is committed and true;
3. exactly one resource is in `focus_resource_list`;
4. that resource belongs to the current text-input surface client;
5. the text-input surface equals the seat's current keyboard input surface;
6. the content purpose is not password, PIN-equivalent or terminal;
7. hidden-text and sensitive-data hints are absent;
8. no legacy delete or commit is waiting for `done`;
9. the active profile mask still represents at least one canonical vector.

Normal admitted state always exposes commit, set-preedit and hide-preedit. It
adds delete only when the exact collapsed-cursor surrounding snapshot is valid.
Thus commit/preedit can remain atomic without inventing a delete snapshot, while
every delete vector is still rejected by the Slice 3A decoder unless the frozen
lease says that the exact snapshot is present.

Resource-count, enable, committed content type, surrounding state and pending
legacy mutation transitions invalidate the current capability epoch. A second
resource cannot share or borrow the first resource's profile.

#### 21.14.5 Pending-output classifier

Slice 3B adds a bounded internal classifier independent of client pending state:

```text
NONE
PREEDIT_ONLY
CLIENT_STATE_ONLY
LEGACY_DELETE_PENDING
LEGACY_COMMIT_PENDING
mixed state containing DELETE or COMMIT
```

Legacy delete, commit and preedit producers mark the classifier before deferring
`done`; client-state `commit()` marks `CLIENT_STATE_ONLY`. Sending or flushing
the matching `done` clears it. Starting any pending output invalidates a lease
that was frozen before that transition.

An atomic submit may synchronously flush only a pre-existing PREEDIT_ONLY or
CLIENT_STATE_ONLY boundary. It then rechecks the one-resource, same-surface and
profile conditions without returning to the main loop. Any classifier state
containing legacy delete or commit makes the profile unsupported and causes
zero atomic protocol requests.

#### 21.14.6 Complete prevalidation and emission

Before the first protocol event the owner prepares the complete frame:

- verifies the same active resource and surface identity;
- rechecks committed content exclusions and pending-output class;
- verifies the frozen snapshot digest for delete;
- converts the exact negative suffix deletion to bounded before/after bytes;
- converts every preedit cursor and anchor from characters to UTF-8 bytes;
- determines the final preedit state and all event order.

No `g_return_if_fail()` after the first native event is an admissible validation
mechanism. A validation failure returns `REFUSED_ZERO_EFFECT` before any v3
request. After prevalidation, Wayland event emission is synchronous and cannot
return a partial semantic result:

```text
optional old-preedit clear
-> optional delete_surrounding_text(before_bytes, 0)
-> optional commit_string
-> optional final preedit_string
-> exactly one done(current resource serial)
```

The final preedit is also synchronized into `ClutterInputFocus` private state so
focus reset semantics do not retain stale preedit material. This requires one
private state helper; it must not call a second protocol vfunc and must not mint
a second mutation route.

The owner returns `SUBMITTED` only after all events including `done` have been
queued to the one resource. Client application is still not proved until a
later exact surrounding postcondition. A disconnected client, process crash or
lineage termination is handled with no retry and no learning.

#### 21.14.7 Focus-bound replay

The production replay vfunc accepts only the Slice 3A marked key copy and only
while the text-input surface remains the seat keyboard focus. It invokes
`meta_wayland_seat_handle_event()` once. Success means one synchronous route to
the current Wayland keyboard; failure is terminal zero-effect. It never calls
`meta_wayland_text_input_update()`, `clutter_input_focus_filter_event()` or an
IBus callback, so it cannot loop through the input method again.

#### 21.14.8 Required real protocol proof

The Slice 3B harness is a real Wayland client binding
`zwp_text_input_manager_v3`, one surface, one seat and the test driver. A
separate focused Mutter server test installs an isolated no-filter
`ClutterInputMethod`, uses the public freeze/submit/replay API and observes only
protocol events received by the client.

Required proof groups:

```text
all eight canonical vectors                         exact order, one done each
delete UTF-8 conversion                             exact byte count
profile with no snapshot                            no delete bit
selection or invalid/oversized UTF-8                no delete authority
password, terminal, hidden, sensitive               unsupported profile
zero or two focused resources                       unsupported profile
legacy delete/commit pending                        zero atomic requests
preedit-only/client-state pending                    earlier done then one frame done
surrounding/content/resource change after lease     lineage terminated, zero frame
same-focus native replay                            one keyboard event, zero IM refilter
focus/resource loss during replay                   zero cross-focus replay
normal build with fatal GLib warnings               PASS
ASan + UBSan + LSan                                 zero diagnostics
```

Files in the initial implementation allowlist are limited to the real Wayland
owner, one private preedit-state helper, one focused server test, one client and
their two Meson registrations. `meta-wayland-seat.c`, `meta-wayland-input.c`,
the Slice 3A validator/state machine, GNOME Shell, IBus, Lay and installed bytes
remain preserved.

Verdict: **SLICE3B_PAPER_READY_FOR_ROUTE_GATE**. Runtime authority changed:
**no**. Deployment authority: **false**. Code may begin only after a focused
route gate and implementation preflight both pass without WATCH or blockers.

#### 21.14.9 Measured Slice 3B route gate

The first route packet is retained as a negative receipt. It returned `VETO`
because it used authority/data relations as physical calls, assigned an
arbitrary string to `scope`, connected incompatible authority roles and named
an undeclared replay-refusal event. Its four execution routes consequently had
zero paths. No source edit followed that result.

Route V2 separates the four graph kinds. Physical atomic submit, atomic
refusal, native replay and legacy IM calls use only `delegates`; profile and
replay authorization remain in authority routes; postcondition observation and
proof are independent routes. The measured result is:

```text
verdict                                      PASS
ready_for_implementation_preflight           true
issues / warnings                             0 / 0
declared routes                              12
routes with exactly one path                 12 / 12
manifest SHA-256  dd366b839118c6a61954ac96552bea53f505515a5f495c8c7e015c5454c4c65c
receipt SHA-256   c01eae0377a16845eb700d1c763059be4f70acea3a2680fc654b202d9f90493b
```

Manifest:
`docs/structural_gates/preflights/LAY_IME_BACKEND_ATOMIC_FRAME_MUTTER_SLICE3B_ROUTE_V2_2026-08-21.json`.
Receipt:
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/mutter-slice3b-route-v2-receipt.json`.

This result proves only internal coherence of the proposed route graph. It has
not tested source completeness, compilation, real text-input-v3 events,
sanitizers, Shell or IBus integration, latency, installed packages or physical
typing. Runtime authority changed: **no**. Deployment authority: **false**.
Verdict: **SLICE3B_ROUTE_GATE_PASS**. A focused implementation preflight remains
mandatory before the first source edit.

#### 21.14.10 Submitted-frame cache invalidation correction

Implementation prevalidation exposed one conflict that the first Slice 3B
paper contract did not close. A successful commit or delete-plus-commit changes
the client document, so the previous committed surrounding snapshot cannot
authorize a second lease while Mutter waits for the client's next committed
surrounding state. Keeping that snapshot available creates stale delete
authority.

Invalidating the capability inside the owner also cannot be combined with the
old generic finalizer rule that re-queries the pre-submit profile after every
vfunc result. Once protocol requests have been queued, converting `SUBMITTED`
to `FOCUS_LINEAGE_TERMINATED` because the owner deliberately invalidated its
old snapshot would report zero authority after a real mutation.

The corrected finalization contract is:

```text
owner complete prevalidation
-> synchronous protocol emission
-> SUBMITTED
-> invalidate old capability and mark surrounding awaiting client commit
-> generic finalizer verifies active state and exact lease identity
-> generic finalizer consumes the lease without re-authorizing the old profile
```

The relaxed re-query applies only to the exact `SUBMITTED` result. A
`REFUSED_ZERO_EFFECT` result still requires the current focus/profile to match
before the lease becomes replay-only. A focus-out or reentrant state-machine
change still changes the active state or lease and therefore remains
`FOCUS_LINEAGE_TERMINATED` for either result.

Only atomic vectors containing commit text make the surrounding snapshot
awaiting. Set-preedit and hide-preedit do not mutate document text. While
awaiting, `query_atomic_profile()` is unsupported. Authority returns only after
the same focused client commits a new valid surrounding snapshot; a commit with
no surrounding update does not re-enable it.

This correction expands the Slice 3B implementation allowlist by one narrowly
scoped file: `clutter/clutter/clutter-input-method.c`. Slice 3A replay-copy,
decoder, lease identity, refusal and replay-only transitions remain unchanged.
The route graph does not gain another owner or call path. A revised
implementation preflight must bind this exact finalizer branch before the edit.
Runtime authority changed: **no**. Deployment authority: **false**. Verdict:
**SLICE3B_PAPER_CORRECTION_REQUIRES_PREFLIGHT_V3**.

#### 21.14.11 Sanitizer baseline correction before promotion

The first Slice 3B sanitized real-Wayland run reached Mutter but stopped before
the protocol assertions in `wrapper_source_prepare()` at
`src/backends/native/meta-thread.c`. When the wrapped main context initially
has no poll descriptors, the existing source declares a variable-length stack
array with bound zero:

```c
int old_n_fds = wrapper_source->n_fds;
GPollFD old_fds[wrapper_source->n_fds];
```

UBSan reports `variable length array bound evaluates to non-positive value 0`.
This is a real baseline defect exposed by the new server proof, not an atomic
frame result. The normal Slice 3B protocol proof remains `5/5 PASS`; the old
Slice 3A sanitizer proof also remains clean because its smaller fake-focus
harness does not enter this zero-descriptor timing state. Therefore neither
result can waive the diagnostic for the real owner harness.

The bounded systemic correction is to allocate the snapshot at the already
fixed owner capacity:

```c
GPollFD old_fds[G_N_ELEMENTS (wrapper_source->fds)];
```

`old_n_fds` still determines every copy and comparison length. The change adds
no heap allocation, route, authority, I/O or mutation owner; it reserves 2048
bytes on this prepare frame instead of a runtime-sized VLA whose maximum was
already 2048 bytes. The complete normal and sanitized Slice 3A and Slice 3B
proofs must rerun after the correction. Any ASan, UBSan, LSan or fatal GLib
diagnostic remains a failure.

Tested in this section: the original Slice 3B sanitized harness reproduces the
zero-bound diagnostic, and the preserved Slice 3A sanitized harness passes in
the same build. Not tested yet: corrected compilation, corrected sanitizer
result, Shell/IBus integration, package installation, desktop restart, physical
typing or production latency. Runtime authority changed: **no**. Deployment
authority: **false**. Verdict:
**SLICE3B_SANITIZER_BASELINE_CORRECTION_REQUIRES_PREFLIGHT_V4**.

Preflight V4 returned `BLOCKED_BEFORE_CODE` with 12 baseline/source blockers.
Its added `meta-thread.c` baseline was exact, but the inherited V3 entries still
described the bytes before the already authorized Slice 3B implementation.
V4 is retained as a negative receipt and grants no edit authority. V5 must pin
the current Slice 3B sources, both new protocol test sources and the exact V3
`READY_TO_IMPLEMENT` receipt, while retaining the same one-file correction
scope. Runtime authority changed: **no**. Deployment authority: **false**.

Preflight V5 returned `READY_TO_IMPLEMENT` with zero blockers. The bounded
array correction compiled in both normal and sanitized builds. Normal Slice 3A
and Slice 3B passed, and sanitized Slice 3A passed. Sanitized Slice 3B no longer
reports the zero-bound VLA; it reaches and completes the real protocol client.

That run exposed a separate test-harness ownership defect: LeakSanitizer reports
2152 bytes in 60 allocations in the child Wayland client. The exact accounting
is 328 bytes of dmabuf modifier arrays plus nineteen 96-byte Wayland proxies.
Fifteen proxies belong to the shared `WaylandDisplay` utility, whose finalizer
disconnects the display without first destroying its client proxies. Four
belong to the new client: its second registry, text-input manager, seat and
keyboard. The production Mutter owner has no reported leak in this result, but
the full sanitized proof remains failed because the child exits non-zero.

The correct proof-harness repair is ownership-complete teardown, not a leak
suppression: the shared utility must destroy every proxy and dmabuf modifier
array it owns before `wl_display_disconnect()`, and the new client must destroy
its four additional proxies. This changes test code only, but the shared helper
has broad test scope; therefore normal atomic proofs, sanitized atomic proofs
and the affected Wayland client suite must pass before the result is accepted.
Not tested yet: corrected client teardown, broad Wayland parity, Shell/IBus
integration, package installation, desktop restart, physical typing or
production latency. Runtime authority changed: **no**. Deployment authority:
**false**. Verdict: **SLICE3B_LSAN_CLIENT_CLEANUP_REQUIRES_PREFLIGHT_V6**.

#### 21.14.12 Slice 3B isolated implementation proof

Preflight V6 returned `READY_TO_IMPLEMENT`, `safe_to_implement=true`, with zero
blockers. The final source was built only on `e@192.168.3.94` with 20 jobs. All
eleven allowlisted local source files exactly match the remote proof sandbox by
SHA-256. The installed Mutter, GNOME Shell, IBus and Lay binaries remain
byte-identical to the frozen baseline.

Measured dynamic result:

```text
normal real Wayland owner groups                         5 / 5 PASS
sanitized real Wayland owner groups                      5 / 5 PASS
preserved Slice 3A normal / sanitized                    1 / 1, 1 / 1 PASS
broad normal Wayland regression suite                   17 / 17 PASS
ASan / UBSan / LSan / fatal GLib diagnostics                         0
normal real-owner test time                                      1.48 s
sanitized real-owner test time                                   2.21 s
broad Wayland suite wall time                                  64.683 s
```

The real client proves all eight canonical effect vectors, exact ordered
delete/commit/preedit/done emission, profile and surrounding digests, sensitive
profile refusal, snapshot and selection handling, legacy pending refusal,
preedit flush, same-focus native replay and resource loss before replay. The
correct resource-loss result is `REFUSED_ZERO_EFFECT + IDENTITY_MISMATCH`
because focus-out already consumed the active lease.

The sanitizer sequence retained two real negative results. First, the harness
exposed the zero-bound VLA in `wrapper_source_prepare()`; the fixed-capacity
snapshot removed it without changing route semantics. Second, LeakSanitizer
found 2152 bytes of client-owned test allocations; ownership-complete teardown
removed all 60 leaks. Neither failure was suppressed or reclassified as PASS.

A 65537-byte Wayland string cannot reach the owner: the Wayland wire rejects
the malformed message before Mutter. The production 65536-byte owner limit was
not weakened to accommodate that fixture. This stronger upstream veto is
recorded separately from the owner scenarios.

Observed-source route V3 also passed:

```text
source evidence                                         35 / 35 verified
declared routes with exactly one path                    12 / 12
issues / warnings                                           0 / 0
manifest SHA-256  2ad91a0acada2e66503bc56359f9a11427662d677d584bad179f42ee9eb44fa9
receipt SHA-256   32c4df47ff0e84fbb5e5761f068f9799b8b26d2125e6525b6fe700fab4eb2623
```

The complete measured receipt is
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/mutter-slice3b-implementation-proof-v1.json`
with SHA-256
`7a935e8fc62577b0a22cb44d307675ec3a199244fa6baa5fce010ae2f2863716`.
Remote raw logs remain under
`/home/e/lay-proof/mutter-slice3a-20260821/logs/`.

This proof establishes the real Mutter text-input-v3 owner in isolation. It
does not establish the GNOME Shell to Mutter capability hop, the IBus to Shell
submission hop, install/rollback, desktop restart, physical typing or
production latency. Runtime authority changed: **no**. Deployment authority:
**false**. Verdict: **SLICE3B_IMPLEMENTATION_PROOF_PASS**. The next gate is one
isolated end-to-end Shell/IBus/Mutter integration, followed by one rollbackable
live-owner install with no shadow or fallback route.

## 22. Slice 4 Single-Flight Shell Adapter Contract

The production target is one route only:

```text
Lay engine
-> IBus ProcessKeyEventAtomicV1
-> GNOME Shell single-flight adapter
-> one Mutter submit or one same-focus native replay
-> MetaWaylandTextInputFocus
```

There is no shadow observation, same-event legacy retry, sequential
delete/commit path, synthetic Backspace owner or second mutation worker. An
unsupported focus is returned to native handling before an atomic lease is
opened. Once an admitted event opens a lease, it terminates only as submitted,
same-focus replayed, consumed with no effect or focus-lineage terminated.

### 22.1 Exact Shell baseline

The installed package remains `gnome-shell 50.1-0ubuntu1.1`. Its complete
`/org/gnome/shell/misc/inputMethod.js` resource was extracted from
`/usr/lib/gnome-shell/libshell-18.so` and has SHA-256
`f835a1eae1d1ff7340d7a9017ca18e76a5437e921d32fe9b3c94904c020c2fb0`.
The source file in Ubuntu source package `50.1-0ubuntu1.2` is byte-identical.
The `1.2` packaging delta does not touch this source file, so it is the selected
reproducible source base. No installed package or process changed during this
comparison.

### 22.2 Epoch compatibility correction

Cross-component inspection found one pre-integration contract defect. The
current IBus Slice 2 check requires all of these values to increase strictly:

```text
input_event_identity
capability_epoch
native_transaction_epoch
```

Mutter correctly gives every frozen event a new `native_transaction_epoch`,
but `capability_epoch` names capability state and changes only when focus,
content purpose, hints, surrounding snapshot, resource ownership or another
profile property changes. Two ordinary keys under one unchanged focus
therefore carry equal capability epochs. The old IBus `<=` check would reject
the second key before the engine call.

The corrected monotonic contract is:

```text
input_event_identity             strictly increasing
native_transaction_epoch        strictly increasing
capability_epoch                nondecreasing
equal capability_epoch          same capability generation is allowed
lower capability_epoch          stale and refused
higher capability_epoch         changed capability generation
```

This does not weaken stale-frame protection. The complete capability record,
including the per-event native epoch and surrounding digest, is SHA-256 bound;
Mutter revalidates the original frozen lease before submit or replay. IBus must
add an explicit two-consecutive-event proof with an unchanged capability epoch
and increasing native epochs, plus a regression refusal case.

### 22.3 Shell event state machine

GNOME Shell keeps one context generation, one in-flight event and a FIFO of at
most 64 owned `ClutterEvent` copies. It freezes a Mutter lease only when an
event reaches the head, serializes the exact capability record, calls the
private atomic D-Bus method with a bounded deadline and validates the complete
reply before invoking Mutter.

```text
IDLE
-> freeze current lease
-> RPC_IN_FLIGHT(event, context-generation, lease)
   -> FRAME_READY -> submit_atomic_frame -> SUBMITTED | TERMINATED
   -> NATIVE_UNHANDLED -> replay_native_key -> REPLAYED | TERMINATED
   -> CONSUMED_NO_EFFECT -> complete_atomic_no_effect
   -> LINEAGE_TERMINATED -> discard lineage
   -> malformed/error/deadline -> close lease fail-closed, discard lineage
-> start next queued event only after the terminal native result
```

Focus out, context replacement, capability invalidation, queue overflow and
deadline cancel the in-flight call and discard queued events from that lineage.
They never replay an event into a replacement focus. Error cleanup attempts
only `complete_atomic_no_effect` for the exact still-current lease; failure to
close means lineage termination, not legacy fallback.

The capability aggregate digest is computed over exact normal-form GVariant
bytes with the frozen domain and type strings. Reply validation covers the
disposition, transaction, event, input-context, daemon focus, engine,
capability epoch, capability digest, canonical effect vector and effect digest.
Only the original Mutter lease is passed back to Mutter; Shell cannot mint a
focus lineage or native epoch.

### 22.4 Slice 4 proof and promotion gates

Implementation may begin only after separate READY preflights for the IBus
epoch correction and Shell adapter. Build and dynamic proof run only on
`e@192.168.3.94` using all 20 CPUs. The isolated denominator must prove:

```text
unchanged-capability consecutive keys accepted                    PASS
regressed capability/native/event identity rejected               PASS
one in-flight RPC and FIFO terminal ordering                       PASS
all four dispositions terminate the exact lease                   PASS
focus/context/deadline/overflow failures cause native effects         0
same-event legacy fallback or duplicate mutation                      0
cross-focus replay                                                    0
malformed or mismatched reply reaching Mutter                         0
atomic RPC p99 / max                                         <=2 / <8 ms
integrated hot p99 / max                                     <=5 / <8 ms
```

Only after the isolated IBus, Shell and Mutter route passes may rollbackable
Ubuntu packages be produced. Live installation requires a complete package
snapshot, an out-of-session keyboard recovery command and a controlled GNOME
session restart. Promotion is directly to the single live owner requested by
the product contract; there is no shadow deployment stage. Runtime authority
changed in this section: **no**. Current verdict:
**SLICE4_PAPER_CONTRACT_READY_FOR_PREFLIGHT**.

### 22.5 IBus epoch compatibility result

Preflight V1 returned `READY_TO_IMPLEMENT`, `safe_to_implement=true`, with zero
blockers. The production change is one comparison in `bus/inputcontext.c`:
capability generations lower than the last admitted generation are stale;
equality is valid while event identity and native transaction epoch remain
strictly increasing. The focused proof now contains two consecutive admitted
events with one unchanged capability epoch, plus lower-capability, repeated
event and repeated-native refusal cases.

All compilation and proof ran on `e@192.168.3.94` using the existing isolated
normal and private build profiles with 20 jobs:

```text
normal build                                          0.43 s
private build                                         0.33 s
normal codec / real hop                         18/18, 1/1 PASS
private codec / real hop                        18/18, 1/1 PASS
local/remote edited source parity                       2/2 PASS
preserved codec source parity                           1/1 PASS
leftover task processes                                      0
```

The normal profile still admits no private production profile. The private
profile proves the exact capability-generation behavior without changing the
wire schema, digest grammar, receipt contract, event/native strictness or
legacy path. Shell/Mutter integration, installation, physical input and live
latency were not tested by this slice. Runtime authority changed: **no**.
Verdict: **IBUS_EPOCH_COMPATIBILITY_PASS**.

### 22.6 Production profile admission correction

Cross-component inspection before Shell implementation found that the proved
Mutter producer and the current IBus admission consumer still name different
profiles. `MetaWaylandTextInputFocus` advertises the production vocabulary
frozen in Section 21.14.2:

```text
adapter_kind                         0x00030001
downstream_transaction_kind          0x00030002
adapter build-contract digest
1475b580ff9600ccfa84e43eb9bd50a61fa0bca88b11d33a104877e36b39adac
```

The IBus normal build currently admits no profile, while its private proof
build admits only `0xffff0001 / 0xffff1001` with a different digest. Therefore
the first real Shell request would be well-formed but always return
`NATIVE_UNHANDLED`. The isolated IBus and Mutter PASS results remain valid in
their measured scopes; they do not establish cross-component profile parity.

The correction is at the admission boundary, not in Mutter and not in the
Shell serializer:

```text
IBus normal build
-> admit exactly the Section 21.14.2 production profile
-> allow only per-event mask/count narrowing already validated by the codec
-> reject every unknown kind, digest, enlarged mask/count or guarantee set

IBus private proof build
-> admit the same production profile
-> additionally admit the existing private proof profile
```

The private profile remains test-only and non-installable. This change does not
add an effect producer, replay owner or mutation owner; it only makes the IBus
consumer recognize the exact profile already minted and enforced by Mutter.
Normal and private proofs must both show production acceptance, unknown-profile
refusal and unchanged private-profile isolation before Shell code may rely on
the production profile. Runtime authority changed: **no**. Current verdict:
**PRODUCTION_PROFILE_ADMISSION_REQUIRES_PREFLIGHT**.

The V2 preflight returned `READY_TO_IMPLEMENT`, `safe_to_implement=true`, with
zero blockers. IBus now admits the exact Section 21.14.2 production vocabulary
in normal and private builds; the old private profile remains available only
under its compile-time proof flag. The focused remote proof on the 20-CPU host
passed `19/19` codec/admission cases in both builds, including exact digest
parity, normal/private isolation, unknown-profile refusal and bounded per-event
mask/count narrowing. Local and remote edited source hashes matched `3/3` and
the Mutter profile producer remained byte-identical.

Receipt:
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/ibus-production-profile-admission-proof-v1.json`
(`SHA-256 e38b65ec7c0ff7d3495eec3e0a98947f6d603c9a0c024055fb56fce45641fafd`).

The production capability has not yet traversed the complete IBus process hop;
that denominator belongs to the isolated Shell/IBus/Mutter integration. A
preserved private process diagnostic also exposed an old five-second watchdog
race around a deliberately non-returning fake engine proposal and is not
counted as a production failure or a PASS. Installation, physical input and
latency remain untested. Runtime authority changed: **no**. Verdict:
**IBUS_PRODUCTION_PROFILE_ADMISSION_PASS**.

## 23. Slice 5 Stateful Lay Engine Transaction

### 23.1 Measured pre-implementation boundary

The first normal production-profile process proof on 2026-08-21 established
that the corrected IBus daemon admits the exact Mutter profile and carries it
to one fake-engine call. The remote normal test passed `1/1`; the engine call
count increased by one, the returned commit was `accepted`, and the legacy
`ProcessKeyEvent` call count remained zero. The Shell build also completed all
remaining `9/9` targets and its focused state-machine proof passed five
subtests in `0.13 s`.

That result exposed the next real boundary rather than completing it. The
working Lay engine exports legacy `ProcessKeyEvent`, but does not export
engine-side `ProcessKeyEventAtomicV1`. Installing IBus, Shell and Mutter at
this point would therefore create a well-formed route that always refuses at
the real engine. Runtime authority remains unchanged.

The preserved private process diagnostic still times out in its old
deliberately non-returning fake-engine case at
`finish_pending_call`. It is neither production PASS nor product failure and
must not be mixed into the normal production denominator.

### 23.2 Rejected shortcuts

The following implementations are rejected before code:

1. Calling legacy `ProcessKeyEvent` from the atomic method and collecting its
   emitted signals. The IBus daemon intentionally suppresses those signals,
   and a collector would retain a second mutation route.
2. Mutating the live Lay state before Mutter settles the frame. A refused or
   focus-terminated frame would leave the engine ahead of visible text.
3. Cloning only `LayIbusEngine` while sharing its `SharedState`. Speculative
   tail, undo and layout handoff writes would leak into the live route.
4. Treating a proposal reply as successful application. Proposal authority and
   native submission authority are separate; only the terminal receipt may
   settle state.
5. Replaying the event through legacy processing after malformed output,
   timeout or uncertain submission. Every such case is fail-closed for that
   focus lineage.

### 23.3 Accepted transaction shape

Lay becomes an atomic proposal producer, not a native text mutator:

```text
validated engine envelope + capability + prior receipt
-> settle or discard the previous speculative image
-> deep-copy engine-local state and SharedState
-> execute one key against the isolated image
-> collect semantic output in one bounded AtomicEffectBuilder
-> canonicalize to [] | [commit] | [preedit] | [hide]
                  | [commit, preedit|hide]
                  | [delete, commit, preedit|hide]
-> return NativeUnhandled | ConsumedNoEffect | FrameReady
-> retain the speculative image only for a receipt-bearing disposition
-> next exact receipt commits or discards that image
```

The speculative image must not share mutable tail, undo, focus, layout or
learning authority with the live engine. Read-only caches may be shared.
Prefetch and precognition work remains identity-bound material only; stale work
cannot publish an authority result. Telemetry may record proposal attempts but
must label them as proposals. Layout switching, accepted-learning records and
other externally visible postconditions are deferred until successful receipt.

The exact settlement matrix is:

```text
SubmittedAtomic / ConsumedNoEffect + exact transaction and digest
  -> commit speculative engine and SharedState, run deferred postconditions

RefusedZeroEffect / FocusLineageTerminated / SubmissionUncertainNoRetry
  -> discard speculative state and deferred postconditions

missing, duplicate or mismatched receipt
  -> discard speculative state, poison atomic lineage, zero new proposal

FocusOut / Disable / context destruction with pending proposal
  -> discard speculative state and deferred postconditions
```

The legacy method remains only for clients that never acquired an atomic
Mutter lease. It is not callable as same-event fallback after atomic admission.
The live GNOME Shell physical route calls only the atomic method.

### 23.4 Required proof before integration

Implementation may start only after a dedicated stateful-engine preflight
returns `READY_TO_IMPLEMENT`. The fixed proof must establish:

```text
exact production engine request ABI                                  PASS
effect-vector canonicalization matrix                                PASS
one proposal, zero legacy signals                                    PASS
accepted receipt commits engine and SharedState once                 PASS
refused/uncertain/focus-lost receipt commits external effects            0
mismatched or duplicate receipt proposals                                0
layout and accepted-learning side effects before receipt                 0
normal IBus -> real Lay engine process hop                            PASS
Shell -> IBus -> Lay -> Mutter -> text-input-v3 isolated round trip PENDING
RPC p99 / integrated p99                                        <=2 / <=5 ms
integrated maximum                                                   <8 ms
```

Only the complete isolated round trip can authorize package production. Only
rollbackable packages, an out-of-session recovery command and a complete
installed-file snapshot can authorize the controlled session restart. There
is no shadow deployment stage.

### 23.5 Real Lay engine process proof, 2026-08-21

The stateful-engine V2 preflight returned `READY_TO_IMPLEMENT` with
`safe_to_implement=true`. The production IBus daemon was then connected to the
real release `lay-ibus-engine` in an isolated session D-Bus and IBus process
tree. The proof selected `lay-ime-ru`, established focus and surrounding state
after engine selection, and submitted 514 production-profile atomic events.

Two rejected assumptions were corrected before the final proof:

1. IBus `focus_epoch` and Mutter `focus_lineage_identity` are independent
   non-zero identity namespaces; they must not be numerically equal. Pending
   engine state now names `daemon_focus_epoch`, while Mutter lineage remains a
   separate admission check.
2. Frozen `AtomicProfile.maximum_effect_count` is `u8`, not `u32`. The exact
   production request signature is `((uuayuuyu)(tttttbay))`, not
   `((uuayuuuu)(tttttbay))`.

Measured result:

```text
real atomic replies                         514/514 PASS
first real commits                              ф, ы PASS
continuous receipt settlement               513/513 PASS
legacy mutation signals                            0
atomic RPC p50                              0.276 ms
atomic RPC p99                              0.633 ms
atomic RPC max                              0.707 ms
test RSS                                    9,440 KiB
test elapsed                                    3.28 s
fake-engine regression                          1/1 PASS
```

The RPC contract `p99 <= 2 ms` and `max < 8 ms` passed. This denominator ends
at the real Lay engine reply. It does not include GNOME Shell, Mutter or the
text-input-v3 client and therefore is not the integrated latency gate.

Receipt:
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/real-lay-engine-process-proof-v1.json`.

Not tested: the complete isolated Shell/IBus/Lay/Mutter/text-input-v3 route,
rollbackable packages, session restart or physical typing. Runtime authority
changed: **no**. Current verdict:
**REAL_LAY_ENGINE_PROCESS_PASS_INTEGRATION_PENDING**.

### 23.6 Full-route V15 harness boundary

The first single-flight isolated launch reached the real GNOME Shell process
and reused the one prelaunched patched IBus daemon. It did not reach the atomic
event denominator. The harness set `IBUS_COMPONENT_PATH` to the Lay-only
component directory, which replaces rather than extends the standard IBus
registry. GNOME Shell therefore could not activate its normal startup engine
`xkb:us::eng` before the test selected `lay-ime-ru`.

Measured V15 facts:

```text
prelaunched IBus socket reused                         PASS
second IBus daemon created                                0
system xkb:us::eng visible                                no
integrated atomic samples                                  0
LAY_ATOMIC_INTEGRATED marker                               0
task-owned leftovers after runner exit                     0
```

This is a proof-harness failure, not a product-quality or atomic-RPC result.
The full-route quality and latency denominator remains unmeasured. The
preserved log is
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/full-route-v15-single-flight-fail.log`
(`SHA-256 ec122ab03762faf58817c2ccd66c0cfcd2801208d7bd068c2c7466a6a5ef79d6`).

The accepted correction keeps one IBus process and one socket while loading
both registries with the documented colon-separated search path:

```text
IBUS_COMPONENT_PATH=/proof-runtime/components:/usr/share/ibus/component
```

`prelaunch-patched-ibus-daemon.sh` now refuses startup unless it can find both
`lay-ime-ru` and `xkb:us::eng`. The bounded remote runner records process
status, the integrated marker, registry readiness, daemon reuse, frozen binary
and model hashes, and leftovers before and after cleanup. It does not install
or alter any live component. Runtime authority changed: **no**. Deployment
authority: **false**. Current verdict:
**FULL_ROUTE_V15_HARNESS_REGISTRY_INCOMPLETE_V16_READY**.

### 23.7 Full-route V16 isolation boundary

V16 loaded the combined Lay and system IBus registries successfully. The
separate daemon service log contains
`LAY_PROOF_IBUS_REGISTRY_READY lay=true xkb=true`, and the V15
`Cannot find engine xkb:us::eng` failure disappeared. The process still did
not reach the automation script: the `proot`-hosted Shell stopped after
PipeWire activation and was terminated by the bounded 90-second timeout.

Measured V16 facts:

```text
combined Lay + system IBus registry                    PASS
single prelaunched IBus socket reused                  PASS
xkb engine lookup failures                                0
GNOME Shell automation entered                            no
integrated atomic samples                                  0
process status                                            137
task-owned leftovers before / after cleanup             0 / 0
```

This again provides no integrated latency or mutation denominator. The exact
log, daemon log, metrics, frozen-input hashes and cleanup evidence are
preserved under
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/`
with the `full-route-v16-system-registry` prefix. The main log SHA-256 is
`dc996df6802422c84090c49c7d2e324321f25ef054aec5fde147adda2e2e256e`;
the registry service log SHA-256 is
`64a62f045bc05872029f9571862dc16650ecbea027e96ceee68ff92009d5e399`.

Cross-run comparison localizes the failure to the isolation mechanism. The
same Shell/Mutter build reached `GNOME Shell started` in V10, V11 and V12 under
`bubblewrap`, while both V15 and V16 stopped at the same pre-automation point
under `proot`. `proot` was introduced to supply the matching `ibus` CLI and Lay
model files, not as a product dependency. Those inputs are now present in the
same disposable rootfs and can be mounted read-only by `bubblewrap`.

The next bounded proof therefore restores the previously working bubblewrap
namespace and retains all later contracts: matching rootfs `ibus`, read-only
model bindings, combined component registry, one fixed IBus socket, one
prelaunched daemon and a reuse-only Shell fallback. No product source or live
runtime changes are involved. Runtime authority changed: **no**. Deployment
authority: **false**. Current verdict:
**FULL_ROUTE_V16_REGISTRY_PASS_PROOT_REJECTED_BWRAP_READY**.

### 23.8 Full-route V17 first-event boundary

V17 restored the previously working `bubblewrap` namespace while preserving
the V16 registry, model, socket and single-daemon contracts. GNOME Shell
started, created the text-input-v3 test client, obtained an IBus input context,
selected `lay-ime-ru`, focused and enabled the real Lay engine, and supplied an
exact surrounding-text snapshot. The first injected key-down nevertheless
reached the GTK test client as the native `a`, not the expected atomic `ф`.

Measured V17 facts:

```text
GNOME Shell automation entered                         PASS
combined Lay + system IBus registry                    PASS
single prelaunched IBus socket reused                  PASS
Lay engine focus / enable                              PASS
surrounding-text snapshot                              PASS
first native test-window text                             a
expected first atomic text                                ф
ProcessKeyEventAtomicV1 engine events                     0
legacy ProcessKeyEvent calls                              0
integrated atomic samples                                 0
process status                                             1
task-owned leftovers before / after cleanup             1 / 0
```

The sole pre-cleanup process was the task-owned managed Lay engine; the runner
terminated it and proved zero leftovers after cleanup. The engine trace has
nine lifecycle/snapshot records and no `ibus_key` record. Therefore this run
does not test the atomic engine reply, Mutter frame submission, mutation
cardinality or integrated latency. It localizes the first unproved hop to the
event admission boundary before the engine RPC:

```text
Mutter text-input event filter
-> GNOME Shell InputMethod.vfunc_filter_key_event()
-> AtomicInputMethodAdapter.enqueue()
-> freeze_atomic_lease()
-> ProcessKeyEventAtomicV1
```

V17 does not yet distinguish whether the event missed
`vfunc_filter_key_event()`, was rejected by the Shell context/source/ignored
mask checks, or reached `enqueue()` and failed lease freeze. V18 must add
observation-only markers at those boundaries without changing event handling,
fallback, text mutation or runtime authority. The exact V17 log, metrics,
engine trace, frozen-input hashes, daemon log and cleanup evidence are
preserved under
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/`
with the `atomic-full-route-v17-bwrap-full-route` prefix. The main log SHA-256
is `71bfa068390ce158adfc8fe308a2b1ab8d40347a0d23279dd0da517d60f844ee`.

No product source or live runtime changed. Runtime authority changed: **no**.
Deployment authority: **false**. Current verdict:
**FULL_ROUTE_V17_PRE_ENGINE_EVENT_ADMISSION_UNRESOLVED**.

### 23.9 Full-route V18 existing-debug boundary

V18 enabled only the already compiled Mutter `input` and `input-events` debug
topics. It changed no Shell, Mutter, IBus, Lay or test source, performed no
build, and reused the exact V17 binaries and models. The 20-CPU proof host had
a load average below 2 immediately before launch.

The first test event was observed below the Shell/IBus boundary:

```text
virtual keyboard device created                         PASS
virtual key press emitted                         key 0x1e
queued Clutter event                           key-press
event modifier state                                none
virtual key release emitted                       PASS
first test-window text                                 a
ProcessKeyEventAtomicV1 engine events                  0
legacy ProcessKeyEvent calls                           0
integrated atomic samples                              0
task-owned leftovers before / after cleanup          1 / 0
```

This excludes a missing virtual input event and excludes an IBus forwarded or
ignored modifier on the generated key. It does not prove whether Mutter called
the Shell input-method vfunc, whether the vfunc reached
`AtomicInputMethodAdapter.enqueue()`, or whether lease freeze returned false.
The existing Mutter topics do not log those boundaries, so V18 remains
diagnostic-only and grants no implementation or latency authority.

The next bounded diagnostic must modify only the directly executed Shell test
script. It will observe the current source and adapter state and wrap the
adapter's `enqueue` and `freezeLease` delegates without changing their
arguments, return values or ordering. No Shell or Mutter production source and
no compiled binary may change.

The exact V18 log, metrics, engine trace, frozen-input hashes, daemon log and
cleanup evidence are preserved under
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/`
with the `atomic-full-route-v18-event-admission-debug` prefix. The main log
SHA-256 is
`0edb032230e41062a648ac1f0bebd423ed127ae1a9a9db261f7741acf53861ef`.

Runtime authority changed: **no**. Deployment authority: **false**. Current
verdict: **FULL_ROUTE_V18_MUTTER_EVENT_PASS_SHELL_ADMISSION_UNRESOLVED**.

### 23.10 Full-route V19 Shell adapter admission boundary

V19 changed only the directly executed Shell test script. Transparent
observers wrapped the existing adapter `enqueue()` and `freezeLease()`
delegates while preserving their arguments, return values, call ordering and
single-flight behavior. Shell, Mutter, IBus and Lay production sources and all
compiled binaries remained byte-identical to V18. The 20-CPU proof host had a
load average below 2 before the run.

Measured V19 facts:

```text
Shell pre-event context / source / adapter enabled       true / true / true
adapter queue / in-flight before event                         0 / false
enqueue reached with modifier state                                  0
freeze_atomic_lease result                                         true
enqueue result                                                     true
first test-window text                                                a
expected first atomic text                                            ф
ProcessKeyEventAtomicV1 engine events                                 0
legacy ProcessKeyEvent calls                                          0
integrated atomic samples                                             0
task-owned leftovers before / after cleanup                         1 / 0
```

The sole pre-cleanup process was the task-owned managed Lay engine and the
runner removed it. The nine-record engine trace contains focus, capability,
cursor and surrounding-text observations but no key event. The patched IBus
daemon log contains registry readiness and no atomic-handler error. Therefore
V19 proves that Mutter delivered the event to Shell, Shell admitted it into the
atomic adapter, the lease froze successfully and the adapter accepted exactly
one in-flight item. It does not prove that the Shell D-Bus proxy call entered
the patched IBus input-context handler.

The first unproved hop is now bounded to:

```text
AtomicInputMethodAdapter._pump()
-> invokeAtomic(context, request, 8 ms, cancellable)
-> context.call("ProcessKeyEventAtomicV1", ...)
-> patched IBus input-context handler
-> engine proxy
```

The native `a` appeared approximately 500 ms after the original key press. It
must not be interpreted as a direct Shell bypass: `enqueue()` returned true and
froze the original event. V19 does not distinguish an immediate proxy/ABI
rejection from an 8 ms RPC timeout followed by native replay, and the delayed
event may include compositor repeat behavior. The next diagnostic must observe
the existing `invokeAtomic()` delegate once, recording proxy identity, call
start and either resolution or exact error domain, code and message. It must
not issue another D-Bus call or alter failure handling.

The exact V19 log, metrics, engine trace, frozen-input hashes, daemon log and
cleanup evidence are preserved under
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/`
with the `atomic-full-route-v19-test-observer` prefix. The main log SHA-256 is
`cac3bd2fed460fef9463fc1dddad122744ccb610c998238d2e1f866f38c2257c`.

This debug-enabled run grants no latency authority. Runtime authority changed:
**no**. Deployment authority: **false**. Current verdict:
**FULL_ROUTE_V19_SHELL_ADMISSION_PASS_PROXY_CALL_UNRESOLVED**.

### 23.11 Full-route V20 proxy call boundary

V20 wrapped only the existing Shell `invokeAtomic` delegate. The wrapper made
no D-Bus call of its own, invoked the original delegate exactly once, returned
the original Promise unchanged and observed its settlement. Production Shell,
Mutter, IBus and Lay sources and binaries remained unchanged.

Measured V20 facts:

```text
proxy name                                      org.freedesktop.IBus
proxy object path             /org/freedesktop/IBus/InputContext_2
proxy interface                     org.freedesktop.IBus.InputContext
request type          (uuut((uuayuuyu)(tttttbay))ay(ytay))
configured timeout                                               8 ms
Promise result                                              resolved
Promise elapsed                                             1.195 ms
reply type                   (yttttttaya(yv)ay)
first test-window text                                                a
ProcessKeyEventAtomicV1 engine events                                 0
legacy ProcessKeyEvent calls                                          0
task-owned leftovers before / after cleanup                         1 / 0
```

The reply decoded successfully in the existing Shell adapter: no proxy error,
ABI error or adapter error was logged, and the adapter replayed native `a`.
Because the engine trace contains no key event, the resolved frame is the IBus
daemon's `NATIVE_UNHANDLED` response produced before the engine proxy call. V20
therefore closes the Shell-to-IBus proxy hop and moves the first loss into the
patched IBus input-context admission predicate:

```text
_ic_process_key_event_atomic_v1()
-> decode request and capability                         PASS
-> construct frame identity                             PASS
-> compound admission predicate                     one term false
-> atomic_return_native_unhandled()
-> Shell native replay
```

The compound predicate currently combines in-flight state, poisoned lineage,
prior receipt, capability admission, focus, engine presence, fake context,
pinned identities, focus lineage, event monotonicity, capability epoch, native
transaction epoch and surrounding-snapshot parity. V20 does not identify which
term rejected the request. The next diagnostic must classify those terms
without changing their values or branch result; broadening the predicate is not
permitted before that evidence exists.

The exact V20 log, metrics, engine trace, frozen-input hashes, test observer,
daemon log and cleanup evidence are preserved under
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/`
with the `atomic-full-route-v20-proxy-call-observer` prefix. The main log
SHA-256 is
`7ee84dfe4afb6f885f3a5dce6dd63a8833cf3d28c2461ce82e5a19acaab8f72d`.

This debug-enabled run grants no latency authority. Runtime authority changed:
**no**. Deployment authority: **false**. Current verdict:
**FULL_ROUTE_V20_PROXY_ABI_PASS_IBUS_ADMISSION_REJECTED**.

### 23.12 Full-route V21 IBus admission predicate

V21 replaced the single compound IBus admission expression in the isolated
proof source with an order-preserving diagnostic classifier. All thirteen
terms retained their original short-circuit order, and
`atomic_consume_prior_receipt()` remained a single stateful call at the same
position. The refusal result and downstream control flow were unchanged.

The scoped remote IBus daemon rebuild completed in 0.47 seconds with 56,800 KiB
peak RSS. Local and remote diagnostic `inputcontext.c` SHA-256 matched exactly.
The 20-CPU host load remained below 2 after the build.

Measured first-event classification:

```text
first rejection                                      capability_profile
input event / previous event                                  1 / 0
capability epoch / previous epoch                              8 / 0
native epoch / previous epoch                                  1 / 0
IBus focus / engine / fake                                 1 / 1 / 0
lease snapshot / context snapshot                          1 / 1
proxy resolution                                           1.113 ms
ProcessKeyEventAtomicV1 engine events                              0
legacy ProcessKeyEvent calls                                       0
task-owned leftovers before / after cleanup                      1 / 0
```

The release event independently repeated the same rejection with event `2`,
capability epoch `12` and native epoch `2`. Thus focus, engine presence,
snapshot availability and monotonic identities are not the first loss. IBus
rejects the profile before attempting the engine proxy.

Cross-component source inspection then found an exact identity contradiction:

```text
canonical Section 21.14.2 digest
1475b580ff9600ccfa84e43eb9bd50a61fa0bca88b11d33a104877e36b39adac

current IBus production admission digest
1475b580ff9600ccfa84e43eb9bd50a61fa0bca88b11d33a104877e36b39adac

actual Mutter producer digest
85742aec30f9d1f63bef5c3963f3af325efbc5b34da74cce2a942876ea354aa7

current Shell unit-fixture digest
85742aec30f9d1f63bef5c3963f3af325efbc5b34da74cce2a942876ea354aa7
```

The canonical `1475...adac` digest is reproducible from the exact 162-byte
grammar recorded in Section 21.14.2. The older `8574...4aa7` bytes remain in
the actual Mutter producer and the Shell unit fixture. Therefore the statement
in Section 22.6 that the unchanged Mutter producer already advertised
`1475...adac` is falsified by the integrated route. The isolated IBus profile
proof remains valid for its codec/admission scope, but its receipt explicitly
listed the complete production process hop as untested; it did not establish
producer/consumer parity.

No admission wildcard or second accepted digest may be added. Before changing
the identity-bearing producer, the next paper gate must establish whether the
canonical contract remains `1475...adac` and enumerate the Mutter build,
focused tests, Shell fixture, IBus parity, engine validator and full-route
artifacts invalidated by that producer correction.

The exact V21 log, metrics, engine trace, diagnostic IBus source, scoped build
log/time, daemon log, frozen-input hashes and cleanup evidence are preserved
under
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/`
with the `atomic-full-route-v21-ibus-admission-predicate` prefix. The main log
SHA-256 is
`fe40ad0c82d19f97ed83fa23dc7ca55202597a2c78aa3c3eb6f0844a3b325362`.

This diagnostic grants no latency or deployment authority. Runtime authority
changed: **no**. Deployment authority: **false**. Current verdict:
**FULL_ROUTE_V21_CANONICAL_PROFILE_IDENTITY_CONFLICT**.

### 23.13 V22 canonical profile identity paper decision

The V21 conflict was caused by a missed coordinated identity update, not by a
hash collision or a runtime-dependent value. Both digests reproduce exactly:

```text
domain plus NUL                                               35 bytes
old descriptor without hide marker                           112 bytes
old digest input                                              147 bytes
old digest       85742aec30f9d1f63bef5c3963f3af325efbc5b34da74cce2a942876ea354aa7

canonical descriptor with ;hide-marker=b0                    127 bytes
canonical digest input                                        162 bytes
canonical digest   1475b580ff9600ccfa84e43eb9bd50a61fa0bca88b11d33a104877e36b39adac
```

The extra descriptor term is required by an actual wire incompatibility. Lay
encodes `HidePreedit` as `b=false` on the engine RPC because Rust `zvariant`
cannot construct GLib's empty tuple payload. IBus admits only that marker on
the engine leg, converts it into the semantic effect, and emits `()` on the
Shell/Mutter leg. Mutter admits only `()` there. The profile digest is the
end-to-end compatibility identity named `LayGnomeTextInputV3AtomicProfileV1`,
so it must bind this translation.

Decision:

```text
production digest                         exactly 1475...adac
production digests admitted at once                         1
8574...4aa7 in mutable production source                    0
wildcard or dual-digest migration                            forbidden
per-leg HidePreedit decoder alternatives                     0
```

Three alternatives were rejected:

1. Admitting both digests would hide a partial component deployment and let a
   profile identity stop proving the exact engine payload contract.
2. Reverting IBus and Lay to `8574...4aa7` while retaining `b=false` would make
   the digest omit a real ABI distinction. Reverting the marker itself would
   restore the Rust construction failure.
3. Adding a second engine-leg digest would separate the two protocol legs, but
   it requires a capability schema revision and a new migration protocol. It
   adds no safety to V1 because one propagated end-to-end token already fails
   closed at every partial-deployment boundary. Such a split belongs only in a
   future protocol version.

#### Invalidation matrix

| Artifact or claim | Measured state | V22 treatment |
|---|---|---|
| Mutter producer `meta-wayland-text-input.c` | source hash `99debf...38f2`, emits `8574...4aa7` | replace only the 32 digest bytes; rebuild the isolated Mutter target |
| Mutter profile fixture `wayland-text-input-atomic-frame-tests.c` | source hash `668e35...2da`, expects `8574...4aa7` | replace only the 32 expected bytes and rerun the focused normal and sanitized proof |
| Shell unit fixture `inputMethodAtomic.js` | source hash `adf513...bb9`, expects `8574...4aa7` | replace only the 32 expected bytes and rerun the unit suite; live Shell adapter source is unchanged |
| IBus codec and engine-transport decoder | source hash `f05d05...d58f`, already pins `1475...adac` and strict `b=false` | source is read-only; rerun the 19/19 admission proof and marker translation proof |
| IBus admission receipt `ibus-production-profile-admission-proof-v1.json` | historical PASS names `8574...4aa7` and pre-correction source hash | retain unchanged as historical evidence; it cannot authorize V22 and must be superseded |
| Lay `atomic.rs` and `output.rs` | hashes `45b065...319` and `7ef339...e1bc`, already pin `1475...adac` and emit `b=false` | source is read-only; rerun focused validator/output proof only |
| IBus `inputcontext.c` V21 classifier | hash `28dbed...491e`, diagnostic-only ordered predicate expansion | restore the exact preserved V20 source bytes, hash `4081c8...f90c`, before latency proof |
| Mutter Slice 3B proof | valid for the old source and its measured effect/frame behavior | retain unaffected semantic evidence; supersede its profile-identity and source-parity claims |
| V17 through V21 full-route artifacts | valid localization evidence; no integrated quality or latency authority | retain unchanged; the final V22 denominator starts fresh |
| remote Mutter, Shell and IBus build products | built from the source hashes above | invalidate and rebuild only in isolated remote sandboxes after local/remote source parity |
| installed Mutter, Shell, IBus and Lay | hashes remain `fec902...e5`, `94e81e...b73`, `e1977b...ec1`, `dabd0b...8c8a` | preserve byte-identically throughout V22; deployment requires a later independent gate |

Historical receipts are append-only evidence and are not rewritten. A receipt
whose pinned source or digest is superseded remains valid only for its original
scope and cannot be cited as current producer/consumer parity.

#### Partial deployment and rollback matrix

```text
Mutter 8574 -> IBus 1475 -> Lay 1475
  IBus rejects capability; engine events 0; Shell performs native replay.

Mutter 1475 -> IBus 8574
  old IBus rejects capability; no atomic effect reaches Mutter.

Mutter 1475 -> IBus 1475 -> Lay 8574
  Lay rejects the request and returns NativeUnhandled; no effect is committed.

Mutter 1475 -> IBus 1475 -> Lay 1475
  identity gate may pass; all remaining lease, frame and verifier gates still
  apply.
```

Thus a partial update degrades to native input instead of mutating text under
an ambiguous contract. This fail-closed property does not authorize piecemeal
deployment: the final packages and rollback set must still be coordinated.

#### Bounded V22 implementation

The implementation scope is fixed before code:

1. Change the digest byte array in the Mutter producer, its exact profile
   fixture and the Shell unit fixture from `8574...4aa7` to `1475...adac`.
2. Do not change the profile schema, mask, count, guarantees, effect vocabulary,
   Shell runtime adapter, IBus codec/admission, Lay validator or Lay output.
3. Restore IBus `inputcontext.c` byte-for-byte to the preserved V20 compound
   predicate, removing the V21 classifier and hot-path diagnostic message.
4. Prove exact grammar hashing, one mutable production digest, strict per-leg
   hide payload types, and local/remote source parity.
5. On the 20-CPU remote host, run the focused Mutter normal/sanitized test,
   Shell unit test, IBus 19/19 profile and marker tests, and focused Lay tests.
6. Run a fresh isolated full route with exactly 514 admitted events, 514 expected
   commits, duplicate mutations `0`, legacy calls `0`, integrated hot
   `p99 <= 5 ms`, maximum `< 8 ms`, and task-owned leftovers `0`.
7. Only an aggregate PASS may open a separate rollbackable package and live
   deployment preflight. V22 itself installs nothing and restarts no session.

The implementation preflight is owned by:

`docs/structural_gates/preflights/LAY_IME_ATOMIC_CANONICAL_PROFILE_IDENTITY_REPAIR_V22_2026-08-21.json`

Its receipt is written to:

`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-atomic-canonical-profile-identity-repair-v22-preflight-receipt.json`

#### V22 implementation-preflight results

The first preflight failed closed exactly as required. V1 returned
`BLOCKED_BEFORE_CODE`, `safe_to_implement=false`, with 14 paper blockers:
ten missing forbidden-effect scans, three missing fault-injection bindings and
one missing identity-parity binding. No source edit was authorized by V1.

V1 manifest:

`docs/structural_gates/preflights/LAY_IME_ATOMIC_CANONICAL_PROFILE_IDENTITY_REPAIR_V22_2026-08-21.json`

V1 receipt:

`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-atomic-canonical-profile-identity-repair-v22-preflight-receipt.json`

V2 repaired those paper omissions without relaxing the implementation scope,
forbidden effects, identity contract, latency gate or deployment boundary. It
returned:

```text
verdict                                      READY_TO_IMPLEMENT
safe_to_implement                                          true
blockers                                                       0
gate-reported manifest SHA-256  60ea0e8cbac119efe59b51d9f80f9e6c479736d4d04a15a3fa7112c609218ca9
baseline checks                                               19
forbidden side-effect kinds                                   19
identity contracts                                             5
invariants                                                     8
preserved artifacts                                           15
mapped tests                                                  22
```

V2 manifest:

`docs/structural_gates/preflights/LAY_IME_ATOMIC_CANONICAL_PROFILE_IDENTITY_REPAIR_V22_V2_2026-08-21.json`

V2 receipt:

`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-atomic-canonical-profile-identity-repair-v22-v2-preflight-receipt.json`

This V2 result authorizes only the bounded four-file source implementation and
its mapped proofs. It does not prove implementation correctness and grants no
packaging, deployment, session-restart or physical-input authority.

What was tested in this paper phase: exact source bytes, both digest grammars,
the two strict HidePreedit encodings, local/remote proof-source hashes, current
installed hashes and remote process/load state. What was not tested: edited
source, any rebuild, component tests, the integrated route, latency, packaging,
installation, session restart or physical input. Runtime authority changed:
**no**. Deployment authority: **false**. Paper verdict:
**CANONICAL_PROFILE_1475_V22_READY_TO_IMPLEMENT**.

### 23.14 V22 implementation and integrated snapshot rejection

V22 changed only the preregistered profile identity sites and restored the
IBus admission predicate to the preserved V20 bytes before latency proof. The
20-CPU isolated host produced the following component evidence:

```text
Mutter normal real-owner groups                         5 / 5 PASS
Mutter ASan + UBSan + LSan real-owner groups            5 / 5 PASS
GNOME Shell inputMethodAtomic                           5 / 5 PASS
IBus normal codec/admission                            19 / 19 PASS
IBus private codec/admission                           19 / 19 PASS
Lay atomic route tests                                  7 / 7 PASS
Lay output vector tests                                 2 / 2 PASS
```

The direct sanitized Mutter run disables `umockdev` through the existing
`META_DBUS_RUNNER_DISABLE_UMOCKDEV=1` proof contract. This keeps `libasan` as
the first runtime of the actual test process and avoids attributing Python,
GObject-introspection or compiler-process leaks to Mutter. The test completed
with exit `0`; no ASan, UBSan or LSan finding was emitted.

The fresh integrated V22 denominator did not pass and grants no aggregate or
latency authority:

```text
profile identity admission                              PASS
first IBus rejection                    surrounding_snapshot
lease snapshot present                                      1
IBus cached snapshot present                                1
ProcessKeyEventAtomicV1 engine calls                         0
legacy ProcessKeyEvent calls                                 0
task-owned leftovers after cleanup                           0
integrated samples admitted                                  0
```

The first atomic RPC resolved in `0.779 ms`, but IBus correctly returned
`NativeUnhandled`; native replay committed `a` instead of the expected Lay
engine result `ф`. A single bounded replay with the already approved V21
order-preserving classifier identified `surrounding_snapshot` as the first
rejecting predicate. The diagnostic source was then removed, V20
`bus/inputcontext.c` was restored to SHA-256
`4081c8277ec9308b85bdd8e32af8a36da69278bbd35de2c66cde03fb13ecf90c`,
and the normal daemon was rebuilt. No installed file or desktop session was
changed.

The failure is a protocol contradiction, not timing or stale build output:

```text
Mutter lease digest =
  SHA-256(
    "MutterTextInputV3SurroundingSnapshotV1" || NUL
    || uint32_le(text_byte_length)
    || uint32_le(cursor_byte_offset)
    || uint32_le(anchor_byte_offset)
    || exact UTF-8 bytes)

IBus cached digest =
  SHA-256(
    "IBusBackendAtomicSurroundingSnapshotV1" || NUL
    || "(suu)" || NUL
    || normal-form GVariant(text, cursor_chars, anchor_chars))
```

Both component implementations satisfy their own fixtures, but the two byte
grammars cannot produce the same digest. Therefore the Section 18.5 IBus
snapshot grammar and the Section 21.14.3 Mutter snapshot grammar are mutually
inconsistent. Component parity did not prove cross-component snapshot parity.
The V22 profile correction remains valid for its `HidePreedit` translation
scope, but `1475...adac` cannot authorize a production route whose declared
`surrounding=v1` has two meanings.

Evidence is preserved under
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/`
with the `v22-` and `atomic-full-route-v22-` prefixes. Runtime authority
changed: **no**. Deployment authority: **false**. Verdict:
**V22_COMPONENT_PASS_INTEGRATED_SNAPSHOT_CONTRACT_FAIL**.

### 23.15 V23 canonical surrounding-snapshot decision

The canonical owner remains Mutter's frozen text-input-v3 lease. IBus remains
an independent verifier and must reproduce the owner's digest from the latest
admitted `SetSurroundingText` tuple. Shell transports the digest byte-exactly;
it does not recompute or translate snapshot authority. Lay treats the snapshot
digest as an opaque capability identity and never scans client text to mint a
replacement.

IBus receives cursor and anchor in Unicode-character coordinates. It must
validate UTF-8, bound text to 65,536 bytes, reject out-of-range character
indices, deterministically convert both indices to UTF-8 byte offsets, and
then apply the exact Mutter grammar from Section 21.14.3. This conversion is
bijective at admitted UTF-8 boundaries and does not create a second document
owner.

The following alternatives are forbidden:

1. Skipping snapshot equality or trusting the lease without independent IBus
   replay would allow stale delete authority.
2. Recomputing a replacement digest in Shell would make Shell a second
   snapshot owner and add a race between the frozen lease and surrounding
   observation.
3. Accepting both snapshot grammars or both profile digests would hide partial
   deployment and make profile identity ambiguous.
4. Suppressing delete authority permanently would make the current commit-only
   example pass but would not complete the production atomic-edit contract.
5. Reusing `1475...adac` after changing the meaning of `surrounding=v1` would
   make the build-contract digest false.

V23 therefore changes the profile descriptor term from `surrounding=v1` to
the explicit `surrounding=mutter-text-input-v3-byte-v1`. The exact grammar is:

```text
ASCII "LayGnomeTextInputV3AtomicProfileV1" || NUL
|| ASCII "protocol=1;adapter=196609;transaction=196610;max-mask=15;max-count=3;required-flags=63;surrounding=mutter-text-input-v3-byte-v1;pending=v1;hide-marker=b0"
```

Measured descriptor length is `153` bytes; total digest input is `188` bytes.
The required profile digest is:

```text
ecf43b4c0c4cebae8db15602a8c14450cb8989c273c5208cb13a452647074af7
```

The bounded implementation scope is:

1. IBus header, codec and codec fixture: replace the old `(suu)` snapshot
   digest with exact Mutter byte grammar, including multibyte and bounds tests.
2. Mutter producer and profile fixture: replace only the profile digest bytes;
   the existing snapshot producer algorithm remains unchanged.
3. GNOME Shell unit fixture: replace only the profile digest bytes; the runtime
   adapter remains an opaque transporter.
4. Lay atomic validator: replace only the profile digest bytes and rerun its
   focused authority tests; effect production remains unchanged.
5. Keep the byte-exact V20 IBus admission control flow, strict snapshot
   equality, one profile digest, one native mutation owner and all latency
   thresholds unchanged.
6. Rebuild and prove the affected component targets, then run a fresh 514-event
   integrated denominator. Packaging or deployment remains a separate gate.

Required promotion remains conjunctive:

```text
component source parity                                  PASS
normal and sanitized component tests                     PASS
normal/private IBus codec admission                 19/19 each
Lay atomic/output focused tests                           PASS
integrated admitted events                                514
integrated expected commits                               514
duplicate mutations                                         0
legacy key calls                                             0
hot p99                                               <=5 ms
maximum                                                 <8 ms
task-owned leftovers after cleanup                           0
```

Runtime authority changed: **no**. Deployment authority: **false**. Paper
verdict: **V23_CANONICAL_SNAPSHOT_REQUIRES_IMPLEMENTATION_PREFLIGHT**.

### 23.16 V23 implementation, proof admission and measured hot-route failure

V23 implemented the canonical snapshot decision without changing mutation
ownership. Mutter still creates the frozen lease and snapshot digest, Shell
still transports it byte-exactly, IBus now independently reproduces Mutter's
UTF-8 byte grammar, and Lay still treats the digest as an opaque capability
identity. The old V22 profile is refused.

Cross-component fixtures use the following exact values:

```text
ASCII text / byte offsets                    text / 4 / 4
ASCII digest        80129ad9644736276c1d128fc16c34d956928563472f5c5c4f5ff6d4d304186c
Cyrillic text / byte offsets              рабатает / 16 / 16
Cyrillic digest     d29997c354a1ddd4541e8d54b13d29982a3aaaf129fddc90c42f55b7b46d0cad
V23 profile digest  ecf43b4c0c4cebae8db15602a8c14450cb8989c273c5208cb13a452647074af7
```

The bounded component gates on the 20-CPU remote host passed:

```text
Mutter normal                                      5 / 5 PASS
Mutter ASan + UBSan + LSan                         5 / 5 PASS
GNOME Shell inputMethodAtomic                      5 / 5 PASS
IBus normal                                      19 / 19 PASS
IBus private                                     19 / 19 PASS
Lay atomic                                         7 / 7 PASS
Lay output                                         2 / 2 PASS
```

The proof engine used by the isolated route is not the installed desktop
engine:

```text
proof engine SHA-256  e06fd36dfff94e81754ee34e67d95c001d3ebe59a1ef60fb6a23a6013a430a4a
proof engine bytes                                                6,672,976
installed engine SHA-256  dabd0bd89fdcd481d4493afb0c3d4f272ae1af56bf1d290bf320325790958c8a
```

The first integrated V23 attempt proved that snapshot admission reached the
engine but exposed a proof-fixture configuration defect. The fixture omitted
`text_backend`; serde therefore inherited the production default `uinput`,
and the existing `live_composition_enabled()` predicate correctly refused the
managed IME route. No runtime gate was weakened. The proof-only configuration
now declares `text_backend: "ime"`.

The first implementation preflight for this proof-only change failed closed
because two file lengths were guessed incorrectly and six forbidden effects
lacked static scans. No code or proof configuration was changed under that
receipt. V2 pinned the measured lengths and complete veto coverage:

```text
verdict                                      READY_TO_IMPLEMENT
safe_to_implement                                          true
blockers                                                       0
manifest SHA-256  de0c60b2f66327fbab05b1e9e64988d8bdd4fea7f8383f79583e13a41d7544c2
```

Manifests and receipts:

```text
docs/structural_gates/preflights/LAY_IME_ATOMIC_PROOF_BACKEND_ADMISSION_V23_2026-08-21.json
docs/structural_gates/preflights/LAY_IME_ATOMIC_PROOF_BACKEND_ADMISSION_V23_V2_2026-08-21.json
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-atomic-proof-backend-admission-v23-preflight-receipt.json
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-atomic-proof-backend-admission-v23-v2-preflight-receipt.json
```

Two fresh full-route runs with the corrected fixture both restored the exact
514-character target and kept the legacy mutation route at zero. Neither run
is an aggregate PASS because the strict maximum remained above 8 ms:

```text
run                                          v1          v2
expected committed characters               514         514
observed committed characters               514         514
printable managed commits                    514         514
legacy key calls                               0           0
hot p50                                   1.051 ms    1.122 ms
hot p99                                   2.390 ms    3.045 ms
hot maximum                              49.037 ms   11.523 ms
Lay printable mean                       0.030 ms    0.032 ms
Lay printable maximum                    0.101 ms    0.195 ms
task-owned leftovers after cleanup              0           0
aggregate verdict                              FAIL        FAIL
```

The first loss is outside Lay. In V1 the 49.7 ms wall interval occurred on the
third key while the Lay engine completed that key in 12 us. In V2 the early
route again contained 36.2 ms and 12.0 ms wall intervals while the Lay engine
maximum across all 514 printable commits was 195 us. After the first five
events, every press-to-release interval in both runs remained below 3.7 ms.
This localizes the remaining latency problem to early Shell/Mutter/application
settlement on the freshly started headless compositor, not morphology, L1-L4,
candidate generation or the Lay atomic producer.

The logs also contain intermittent `current focus has no atomic profile`
messages on release events. All 514 printable presses were committed exactly
once, but only 508 and 500 releases respectively reached the engine. This is a
separate release/profile-settlement observation and must not be hidden by the
printable cardinality result.

What was tested: canonical cross-component snapshot parity, component gates,
proof-only backend projection, two isolated 514-character routes, printable
cardinality, legacy-route absence, engine timing and post-failure cleanup.
What was not tested: a quiesced hot-route denominator, package installation,
physical applications, rollback or live desktop behavior. Runtime authority
changed: **no**. Deployment authority: **false**. Current verdict:
**V23_CARDINALITY_AND_P99_PASS_MAX_AND_RELEASE_PROFILE_FAIL**.

### 23.17 Quiesced hot PASS and IBus child-lifecycle failure

The two failed V23 latency runs localized every over-8-ms interval to the
freshly started headless compositor's first five events. The Lay engine itself
remained below 0.2 ms. The hot benchmark therefore received one fixed
quiescence barrier after the two existing correctness commits. This did not
change the route, discard any observed latency, add retries, reduce the 512
hot denominator or relax either latency threshold.

The first quiescence preflight failed closed before code because its own
denominator veto regex matched the first two digits of the valid value `512`.
V2 added the missing token boundary and passed:

```text
verdict                                      READY_TO_IMPLEMENT
safe_to_implement                                          true
blockers                                                       0
manifest SHA-256  9ee3d29f35c703a98e95fd75389157b916d5c6ba64cab493b3abdd8563174819
```

The approved fixture change is restricted to
`tests/shell/atomicInputMethodRoute.js`. It waits for exact surrounding text,
an empty atomic queue and stable focus/context/engine, then waits for Shell
leisure and one fixed 100 ms settlement interval before creating the unchanged
512-sample hot array. Runtime Shell sources remain byte-identical. Local and
remote fixture SHA-256 is:

```text
4224ac0372d0aff81446554b3940ee1be0bdd92780d6373e024c7405163c82ab
```

Preflight artifacts:

```text
docs/structural_gates/preflights/LAY_IME_ATOMIC_HOT_MEASUREMENT_QUIESCENCE_V23_2026-08-21.json
docs/structural_gates/preflights/LAY_IME_ATOMIC_HOT_MEASUREMENT_QUIESCENCE_V23_V2_2026-08-21.json
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-atomic-hot-measurement-quiescence-v23-preflight-receipt.json
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-atomic-hot-measurement-quiescence-v23-v2-preflight-receipt.json
```

The fresh route then passed cardinality and every latency threshold:

```text
proof process status                                         0
integrated marker                                             1
expected committed characters                              514
observed committed characters                              514
printable managed commits                                  514
legacy key calls                                              0
hot p50                                                1.091 ms
hot p99                                                2.049 ms
hot maximum                                            4.119 ms
required hot p99 / maximum                     <=5 ms / <8 ms
```

This is not yet an aggregate PASS. After the compositor, test client and IBus
daemon completed normally, one proof-owned `lay-ibus-engine --ibus --managed`
process remained. The harness terminated it, producing:

```text
task-owned leftovers before forced cleanup                    1
task-owned leftovers after forced cleanup                     0
aggregate verdict                                           FAIL
```

The first owner of this failure is `src/bin/lay_ibus_engine/server.rs`.
`server::run()` builds the IBus and session-bus connections, starts warmup and
then awaits `std::future::pending::<()>()` forever. The component's `--ibus`
flag is parsed but ignored. On the installed desktop the active engine's parent
is the active `ibus-daemon`, so an IBus-owned engine must terminate when that
parent disappears; manual non-IBus starts must remain unaffected.

The bounded lifecycle repair is therefore process-lifecycle only: arm Linux
parent-death termination before opening either bus, and only when `--ibus` is
present. The implementation must close the arm-before-parent-exit race, must
not add a polling worker or a second text-mutation route, and must preserve all
atomic, snapshot, latency and cardinality behavior. The successful hot receipt
is:

```text
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/atomic-full-route-v23-canonical-snapshot-quiesced-hot-v1.log
```

What was tested: fixed non-adaptive quiescence, exact 514-character output,
512 hot latencies, legacy-route absence, process status and cleanup. What was
not tested: automatic IBus-child termination, packaging, live deployment,
physical clients or rollback. Runtime authority changed: **no**. Deployment
authority: **false**. Current verdict:
**V23_HOT_ROUTE_PASS_CHILD_LIFECYCLE_FAIL**.

### 23.18 IBus child lifecycle and settled aggregate PASS

The lifecycle V1 implementation preflight stopped before code because two
identity contracts referenced unit tests rather than parity tests. The failed
manifest and receipt remain immutable. V2 added explicit parent-identity and
manual-mode parity tests and passed:

```text
verdict                                      READY_TO_IMPLEMENT
safe_to_implement                                          true
blockers                                                       0
manifest SHA-256  e32138ec14ed21a2b27226cb00f2c462d5ccbf514cce47137ba4e7cf9e4d2e04
```

`lay-ibus-engine` now arms Linux `PR_SET_PDEATHSIG(SIGTERM)` only after the
`--xml` early return and only for `--ibus`. It captures the direct parent PID,
rejects PID 0 or 1, arms the signal and immediately requires the same parent
PID. A changed parent or failed `prctl` aborts startup before either D-Bus
connection is opened. Manual mode, XML mode, key handling, atomic authority
and output authority are unchanged. The route has no polling thread, timer,
signal-handler override or key-path lifecycle check.

Focused remote tests and source parity passed:

```text
parent identity stable across arm                           PASS
init parent rejected                                       PASS
changed parent rejected                                    PASS
prctl failure propagated                                   PASS
manual mode does not inspect or arm parent                 PASS
focused tests                                               5/5
remote release build                                      51.04 s
release engine bytes                                    6,676,816
release engine SHA-256  4f007010a264374cf35b9e93309e37d3bdf47a558d961ca2e2558b2f6761e735
atomic.rs SHA-256       0ac32eef663638c826f95a0e0293e9da8eb99377bca30d326b297e4febd73188
output.rs SHA-256       7ef339f78503de6d779c882c879021c22ef9f65b1d07cc09fb7b0d765131e1bc
```

An external `/proc` observation in the isolated route confirmed the intended
direct ownership while both processes were live:

```text
lay-ibus-engine PID 3223119 -> ibus-daemon PID 3222083
```

The first lifecycle run restored all 514 characters, passed p99 and maximum,
and left zero processes, but Shell aborted during unordered test-window
teardown. It also retained fewer trace rows because the new parent-death
termination correctly stopped the process before the asynchronous one-second
debug-log flush. Those retained-row counts were not interpreted as input
loss. Two subsequent diagnostic runs were not promoted: one was contaminated
by a process watcher that matched the proof ownership mask, and one exceeded
the strict maximum latency gate.

A separate proof-only post-route-settlement preflight then passed with no
runtime-source changes. After the unchanged 512-sample denominator and both
strict latency checks have completed, the fixture waits a fixed 1,100 ms for
trace durability, destroys the test window, observes focus release and lets
Shell settle. The final isolated result is:

```text
proof process status                                         0
committed characters                                   514/514
printable managed commits                              514/514
printable timing records                               514/514
surrounding-text records                               514/514
legacy key calls                                              0
hot samples                                                  512
hot p50                                                1.121 ms
hot p99                                                2.723 ms
hot maximum                                            3.732 ms
required hot p99 / maximum                     <=5 ms / <8 ms
task-owned leftovers before cleanup                            0
task-owned leftovers after cleanup                             0
aggregate verdict                                           PASS
```

The separate release/profile observation remains open and was not hidden:
508 of 514 releases reached the engine trace, while Shell emitted eight
`current focus has no atomic profile` messages. All 514 presses reached the
managed commit owner exactly once; the release gap does not grant deployment
authority and remains a required follow-up compatibility proof.

Receipts:

```text
docs/structural_gates/preflights/LAY_IME_IBUS_PARENT_LIFECYCLE_V23_2026-08-21.json
docs/structural_gates/preflights/LAY_IME_IBUS_PARENT_LIFECYCLE_V23_V2_2026-08-21.json
docs/structural_gates/preflights/LAY_IME_ATOMIC_POST_ROUTE_SETTLEMENT_V23_2026-08-21.json
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-ibus-parent-lifecycle-v23-preflight-receipt.json
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-ibus-parent-lifecycle-v23-v2-preflight-receipt.json
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-atomic-post-route-settlement-v23-preflight-receipt.json
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/lay-ime-ibus-parent-lifecycle-v23-implementation-receipt.json
docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_FRAME_PRODUCTION_OWNER_2026-08-21/atomic-full-route-v23-canonical-snapshot-lifecycle-settled-v4.log
```

What was tested: Linux parent-death race closure, manual-mode separation,
remote source parity and build, exact managed press cardinality, fixed hot
latency, orderly proof teardown and zero automatic leftovers. What was not
tested: packaging, installation, rollback, live desktop restart, physical
clients or complete release-event delivery. Runtime authority changed: **no**.
Deployment authority: **false**. Current verdict:
**V23_AGGREGATE_PASS_DEPLOYMENT_PENDING_RELEASE_PROFILE_FOLLOWUP**.

### 23.19 Paired release ownership design

The remaining `508/514` release result is not random profile expiry. An atomic
text frame deliberately sets `awaiting_surrounding_after_atomic` until the
Wayland client acknowledges the new text with a fresh surrounding snapshot.
The physical release can arrive inside that interval. Mutter must reject a
new mutation-capable lease there, so broadening the atomic profile would
weaken the one-frame transaction boundary.

The current fallback is also not a complete final design. Shell returns the
release to the native route while the corresponding press was consumed by the
atomic frame, and Lay retains that keycode in `handled_press_keycodes`. A later
unhandled press of the same keycode can therefore meet stale release state.

The proposed ownership split is:

```text
FRAME_READY press
  -> Shell records one exact (keycode -> paired release) receipt
  -> Mutter submits the atomic text frame
  -> Lay removes its temporary handled-release marker for that frame

matching non-modifier release while surrounding refresh is pending
  -> Shell consumes exactly one matching release locally
  -> no second lease, no Lay RPC, no native release, no text effect

modifier release or release without a FRAME_READY pair
  -> unchanged atomic route
  -> Shift/Alt and double-Shift semantics stay owned by Lay
```

The local release ledger is bounded by physical keycodes, cleared on context
or focus invalidation, populated only after `SUBMITTED` `FRAME_READY`, and
consumed exactly once. `CONSUMED_NO_EFFECT`, `NATIVE_UNHANDLED`, failed,
cancelled and modifier events cannot populate it. This does not add a second
text-mutation owner; it closes the already consumed press/release pair at the
Shell event boundary.

Promotion gates:

```text
design code-route gate                                      PASS required
implementation preflight                    READY_TO_IMPLEMENT required
FRAME_READY printable press -> local release pair            514/514
release RPCs for those paired printable presses                     0
native releases for those paired printable presses                  0
modifier atomic press/release parity                              PASS
double Shift exact autocorrect undo                               PASS
all existing 514 press commits                                    PASS
legacy text mutation calls                                           0
p99 / maximum                                         <=5 ms / <8 ms
automatic lifecycle leftovers                                    0/0
```

This section is a design contract only. Runtime authority has not changed.

### 23.20 Physical double-Shift bridge consequence analysis

The V24 undo proof exposed a production-route regression rather than a Shell
fixture limitation. Modifier events correctly bypass the paired-release ledger,
but the daemon's confirmed double-Shift handler currently returns without an
action whenever the active text backend is IME. The exact autocorrect snapshot
is owned by the focused IME engine, so the daemon cannot restore it from its
independent `WordBuffer`.

Two viable repairs were compared:

1. Export the IME pending-undo state and visible tail to the daemon, reconstruct
   the decision there and send a checked `ReplaceTailV4` command back. This
   duplicates state identity across processes, adds a new protocol and stale
   snapshot race, and cannot reuse the existing active-composition commit path.
2. Restore the existing `ManualToggleV2` event bridge. The daemon remains the
   sole owner of the physical gesture; the focused IME remains the sole owner of
   its pending undo, visible composition and mutation. Both use the shared
   `manual_toggle` planner. No candidate source, ranking route or fallback is
   added.

Option 2 is selected. The route is:

```text
physical Shift press/release pair x2
-> daemon trigger FSM
-> one synchronous ManualToggleV2 D-Bus request
-> focused IME pending exact undo first
-> otherwise shared manual-toggle planner for the IME-owned visible target
-> one authorized IME mutation
-> target layout synchronization
```

Consequences and invariants:

- candidate/lattice retention, L1-L4 ranking and false authority are unchanged;
- the request exists only on explicit double Shift, outside printable and Space
  latency deadlines; it adds no work to ordinary key handling;
- CPU, RSS, allocation, package and model/delta reload behavior are unchanged;
- no cache is added; focus path, tail epoch and pending undo stay inside the IME;
- daemon pending undo keeps precedence when it exists, so uinput ownership and
  its learning receipt are preserved;
- a D-Bus error or an IME `NotHandled` result performs no text mutation and does
  not silently replay through a second backend while IME owns text;
- the synchronous request is serialized by the existing trigger FSM and focused
  engine object; no worker, queue or stale asynchronous result is introduced;
- rollback is deletion of the two adapter modules and their module declarations;
  no data or package migration is required;
- maintenance cost is bounded to the already published `ManualToggleV2`
  interface; introducing a second undo-specific D-Bus API is explicitly rejected.

Required proof denominators remain conjunctive: physical daemon FSM dispatch,
exact `ghbdtn -> привет -> ghbdtn` restoration, zero modifier ledger entries,
the unchanged 514 printable paired-release route, strict latency thresholds and
zero proof-owned leftovers. Runtime authority changes only after deployment;
this analysis and the isolated proof do not modify the installed desktop.

The first composite attempts also exposed a proof-cleanup ownership gap. On a
headless compositor timeout, the private D-Bus mocks, PipeWire and WirePlumber
process tree can be reparented to PID 1 with generic command lines. The old
cleanup mask did not count them even though their environment retained
`MUTTER_TEST_LOG_DIR=/proof-output`; later headless startups could then stall
before the first input event. The harness now uses that exact proof-only
environment marker in addition to its command-line mask. It does not match or
terminate the user's normal desktop services. Forty-eight confirmed orphan
processes from the two failed sessions were terminated, with zero marker-owned
processes remaining. Failed runs v13 and v15 are retained as startup failures;
v14 is retained separately as a stale capability/native-replay failure before
undo-record creation.

### 23.21 Concurrent preedit and frozen-lease compatibility analysis

The current-source V24 undo runs v14 and v16 fail deterministically after the
first atomic key succeeds and before the third printable key can be replayed:

```text
first key lease frozen                                      true
first ProcessAtomicKeyEvent reply                      1.775 / 2.079 ms
failure       focus lineage or capability changed before replay
autocorrect record created                                  false
double-Shift proof reached                                  false
```

The first shared mechanism is not the word `ghbdtn` and is not paired-release
ownership. `lay-ibus-engine` schedules precognition after visible input. Its
background worker later acquires the focused engine and emits
`UpdatePreeditText` through `EngineOutput::legacy`. In Mutter,
`meta_wayland_text_input_focus_set_preedit_text()` classifies that output as
`META_WAYLAND_PENDING_OUTPUT_PREEDIT`; `mark_pending_output()` then increments
the atomic capability epoch even when a lease has already been frozen. The
pending preedit can therefore land between Shell's lease freeze and atomic
submit and terminate an otherwise unchanged focus lineage.

This conflicts with the submit contract in the same owner. Atomic submit
already classifies pending `PREEDIT` as flushable, flushes its `done`, validates
the unchanged profile and snapshot, and then publishes the atomic frame. The
profile bytes do not contain preedit text or pending-preedit state. Legacy
commit and delete are separately classified as mutations and refused.

Three repairs were compared:

1. Suppress background precognition once `atomic_route_active` is set. That flag
   denotes route ownership, not a single in-flight lease, so this would disable
   idle completion updates for the rest of the focus lifetime.
2. Add a cross-process in-flight lease protocol so the worker can defer output.
   This adds a second synchronization API, cancellation state and a stale
   acknowledgement race to every key.
3. Make pending preedit lease-compatible inside the existing Mutter owner while
   retaining invalidation for every state or text mutation. This matches the
   existing flush-before-frame submit path and needs no new route.

Option 3 is selected. The exact contract is:

```text
freeze lease
-> optional same-focus legacy PREEDIT becomes coherent pending display output
-> atomic proposal returns
-> submit revalidates focus, profile and surrounding snapshot
-> pending PREEDIT is flushed once
-> atomic frame is applied once
```

Consequences and invariants:

- preedit may change visible composition only; it cannot commit, delete or
  alter surrounding text;
- legacy commit/delete still invalidate the capability epoch and remain a
  zero-effect refusal at submit;
- client-state commits, focus/resource changes, content type, enabled state,
  surrounding snapshots and post-atomic awaiting state remain invalidators;
- the frozen profile, snapshot digest, focus lineage and native epoch remain
  byte-for-byte checked at submit and replay;
- no Lay candidate, ranking, correction, undo or learning policy changes;
- no queue, worker, cache, fallback, retry or additional D-Bus call is added;
- ordinary latency work is unchanged; the repair removes a failed replay and
  uses the submit path that already flushes pending preedit;
- rollback is the scoped pending-preedit classification plus its focused test;
  it requires no model, package or user-data migration.

The required focused proof is ordered, not timing-based:
`freeze -> set preedit -> dispatch before done -> submit`. It must prove that
the final atomic preedit replaces the flushed older preedit. Existing
legacy-commit and legacy-delete refusal checks remain conjunctive. Promotion
still requires the composite autocorrect/double-Shift proof, the unchanged
514-event paired-release route, strict latency gates and zero task-owned
leftovers.

What was tested before this decision: v14 and v16 reproduce the same capability
termination; source tracing proves the background legacy preedit path and the
contradictory invalidation/flush classifiers. What was not yet tested: the
scoped classifier repair, focused Mutter tests, rebuilt full route, installed
runtime or physical desktop. Runtime authority changed: **no**. Current verdict:
**V25_READY_FOR_STRUCTURAL_GATE**.

### 23.22 V25 proof-engine artifact provenance correction

The first V25 composite rerun did not prove a Shell-to-Lay request-validation
failure. Comparing the last working undo run with the failing runs exposed a
proof-artifact substitution:

```text
run                                      engine SHA-256  bytes      ibus_key
v24 paired-release undo v12              011bc0f1...     6,676,816  7
v24 paired-release undo v14/v16           3f37217c...     6,491,216  0
v25 concurrent-preedit undo v2            3f37217c...     6,491,216  0
```

The `011bc0f1...` engine accepted all seven non-modifier events and applied
`ghbdtn -> привет`. The replacement `3f37217c...` engine came from the older
August 16 build tree
`/home/e/build/lay-immediate-space-material-reuse-20260816-v1`. That source
tree contains neither `src/bin/lay_ibus_engine/atomic.rs`, the `mod atomic`
declaration nor the `ProcessKeyEventAtomicV1` method. A later daemon release
build reused its mutable `target/release` directory and overwrote the atomic
engine that had previously been copied there for the proof.

The resulting observed route was therefore:

```text
Shell ProcessKeyEventAtomicV1
-> IBus broker
-> stale engine has no ProcessKeyEventAtomicV1 implementation
-> broker returns NATIVE_UNHANDLED
-> Shell replays the physical event to the same focus
-> surrounding text grows from one through seven characters
-> no Lay key event, autocorrect state or undo receipt is created
```

Consequently, `engine_key_events=0` did not identify a mismatching field in
Lay `valid_request()`. No request reached that validator. The V25 pending
preedit repair remains independently supported by its ordered focused proof,
but the composite V25 result is invalid as evidence about the atomic Lay
consumer.

The selected correction changes proof-artifact ownership, not runtime policy:

```text
current Lay source snapshot
-> byte-parity remote source mirror
-> cargo-guard focused tests and release build
-> immutable proof/runtime/bin/lay-ibus-engine
-> exact SHA-256 manifest plus required-method admission
-> bwrap read-only bind
-> composite proof
```

The full-route driver must fail before starting GNOME Shell when the staged
binary hash does not match its manifest, the source manifest is absent, or the
binary does not contain the exact `ProcessKeyEventAtomicV1` method. It must not
read an engine directly from any mutable Cargo `target/` directory. Building a
daemon may no longer change the artifact selected by an already staged proof.

Consequences and invariants:

- Lay request validation, profile constants, IBus admission, Mutter identity
  checks, candidate policy and edit authority remain unchanged;
- the installed `1.0.33` runtime remains byte-identical until deployment;
- a stale, non-atomic or partially copied engine terminates the proof before
  input rather than manufacturing native-replay evidence;
- source parity and binary parity are separate receipts;
- proof logs record the immutable engine, source-manifest and provenance-file
  hashes used by that exact run;
- the V24/V25 failed receipts remain historical evidence and are not rewritten;
- rollback removes the immutable staging admission and restores the prior
  driver bytes; it does not alter user data or installed runtime state.

Required promotion remains conjunctive: focused atomic-engine tests, exact
`ghbdtn -> привет -> double Shift -> ghbdtn`, unchanged 514-event paired
release proof, `p99 <= 5 ms`, `max < 8 ms`, zero legacy calls and zero
proof-owned leftovers. What is not yet tested at this point: the immutable
staging implementation or either rebuilt composite denominator. Runtime
authority changed: **no**. Current verdict:
**V25_ARTIFACT_PROVENANCE_READY_FOR_STRUCTURAL_GATE**.

#### V25 source-closure correction

The first immutable staging attempt stopped before producing an engine. Remote
Cargo compiled the current source but could not read
`data/morphology/russian_noun_cases_small.tsv`, which is embedded by
`morphology_phase/proof.rs`. This proves that the V2 mirror inventory was not a
closed set of compiler inputs: focused binary tests also compile the library in
test configuration, including repository fixtures and compile-time proof data.

The correction expands the SHA-addressed mirror to the small complete build
surface: Cargo metadata, `src`, `scripts`, `tests`, `data/lexicon`,
`data/test_input`, `data/morphology/russian_noun_cases_small.tsv`, and
`data/nanda_llmwave_seed_phrases.txt`. It explicitly does not copy the large
runtime model directories under `data/morphology`, `data/lexical_grokking`, or
`data/l2`. Local and remote aggregates still cover exactly the same relative
paths before Cargo starts.

Measured fact: the failed attempt exited with Cargo status 101 before artifact
staging or composite input. What was not tested: focused tests, release build,
undo proof, 514-event proof, deployment, or physical input. Runtime authority
changed: **no**. Current verdict:
**V25_SOURCE_CLOSURE_READY_FOR_IMPLEMENTATION_PREFLIGHT**.

The corrected source closure then passed focused atomic tests and produced the
immutable release engine `7752a90072249c6620c9b3d8aa1c56c2d618c72d984e924e2fedfb48188e4e55`
from source aggregate
`3eb36ce7199d014c2ec0f47ae6a4ee7f66a0b044342ab7001577988761217f94`.
Its admitted size is 6,676,944 bytes and the source manifest contains 1,028
files. A deliberately corrupted temporary copy was rejected with status 65
and `atomic proof engine hash mismatch`. The installed `1.0.33` engine remained
`dabd0bd8...c8a`.

The first rebuilt undo denominator, suffix `undo-v3`, was not a semantic result.
The immutable engine was admitted, but headless GNOME Shell did not reach its
`GNOME Shell started` marker or send an input event before the outer 90-second
timeout. The trace therefore contained zero engine events. Its GVC/audio startup
failed only at teardown, and cleanup removed all 28 processes from that aborted
headless session. Inspection also found and removed a distinct nine-hour-old
proof-owned PipeWire orphan, PID 2624633, whose runtime directory was under
`/tmp/mutter-testroot-*` and whose output descriptor pointed into this proof.
This failed suffix is retained and must not be counted as an undo quality or
protocol denominator. Runtime authority changed: **no**.

Suffix `undo-v4`, run after removal of the stale PipeWire process, reproduced
the same pre-input timeout: one IBus reuse marker, no `GNOME Shell started`, no
engine trace and complete `28 -> 0` cleanup. The orphan was therefore a valid
cleanup defect but not the startup root cause. The common boundary is the
headless user session's unrelated `quickSettings` audio/camera initialization,
which activates PipeWire before the IME proof starts.

The selected harness correction is a dedicated external `lay-proof` session
mode. It inherits normal user window behavior, but declares no panel items and
no background session components. Input focus, Shell's atomic adapter, Mutter's
text-input implementation, the immutable Lay engine and
`atomicInputMethodRoute.js` stay unchanged. The proof still has to execute the
same undo and 514-event denominators. This isolates the measured route from
audio startup rather than relaxing a timeout or counting a partial run.

What was tested: two consecutive rebuilt startup attempts reproduced the same
zero-event timeout. What was not tested: the minimal proof session or either
composite denominator. Runtime authority changed: **no**. Current verdict:
**V25_MINIMAL_PROOF_SESSION_READY_FOR_IMPLEMENTATION_PREFLIGHT**.

The first minimal-session run, suffix `minimal-session-undo-v5`, removed the
90-second startup wait and failed in under two seconds with an exact dependency
error: `KeyboardManager._xkbOptions` was undefined when IBus became ready. The
empty panel had also removed GNOME Shell's keyboard owner, which initializes
those options. This run had zero input events and `0 -> 0` leftovers and is not
an undo denominator.

The corrected proof mode therefore keeps exactly the `keyboard` panel item and
no other panel or component. It restores the required XKB/InputSourceManager
initialization without restoring `quickSettings`, volume, camera, PipeWire or
any production session configuration. What was not yet tested: keyboard-only
session startup or the undo denominator. Runtime authority changed: **no**.
Current verdict:
**V25_KEYBOARD_ONLY_PROOF_SESSION_READY_FOR_IMPLEMENTATION_PREFLIGHT**.

### 23.23 V26 exact-layout lease failure and V27 paper correction

V25 then isolated a distinct head-of-line blocker inside Lay. The final
`ghbdtn` decision completed in `0.511 ms`, but the single prefetch worker was
still occupied by stale `ghbdt` work for `20.053 ms`. Space waited `8.080 ms`
inside an `8 ms` Shell RPC deadline and completed in `8.159 ms`. Increasing the
RPC timeout was rejected because it would preserve stale-work ownership rather
than remove it.

V26 proposed a bounded exact-layout lease through the existing lattice,
DecisionCore and verifier. Its first remote focused test rejected the
implementation before staging: `dnjpfvtyf` mapped literally to `втозамена` and
incorrectly received exact authority. A separate field probe showed why:
`втозамена` had `21/21` covered atoms and zero residuals despite lacking an
exact lexical terminal. The V26 predicate had merged exact surface membership,
morphology/form settlement and n-gram coherence into one boolean. Complete
atom coverage is candidate plausibility, not exact surface authority.

V27 is therefore paper-first and narrower. Its fast lease is EN-to-RU only,
requires a nonblocking exact terminal in an already warm lexical package,
rejects an exact/known ASCII source, generated-only forms, layout+typo,
protected/composite inputs and edge-symbol ambiguity, and preserves arbitrary
left phrase context while changing only the current token. The exact lease is
prepared on the printable frame and merely selected on Space; Space cannot
trigger package initialization. Full and exact producers still use one
L2CandidateLattice, one TransitionDecisionCore, one verifier and one atomic
mutation owner.

The first V27 design PASS was then manually rejected as semantically
incomplete. The same ranker over an exact-only lattice and a full lattice does
not prove equivalent decisions; a ready full `Tied | ABSTAIN | rejected`
outcome was also indistinguishable from pending work. The corrected V27
contract adds an exact closed-contour certificate, requires the full route to
birth the same typed candidate, stores `FullTerminal::Apply | NoApply`, and
gives every ready full terminal precedence over exact. Exact preparation is now
explicitly inline and queue-free on the printable frame, with one
linearization point at Space arbitration.

The corrected identity contract separates keyboard-map, RU-terminal,
EN-source-guard and protection-policy fingerprints. The product gate now also
requires exactly one Space on every apply/no-apply path, common double-Shift
undo and feedback semantics, zero stale applies across input/profile/package
changes, zero hot-path I/O and bounded one-slot state. Design route V2 is
recorded in
`docs/structural_gates/preflights/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_ROUTE_V27_V2_2026-08-22.json`;
its receipt `route-design-v5.json` is `PASS` with zero issues and warnings.

That formal PASS and implementation preflight V2 were then manually rejected:
the full terminal preparation route incorrectly required the authorization
verifier even for rank-level `Tied | ABSTAIN`. Final design V3 separates full
Apply, rank NoApply and verifier-rejected NoApply; only Apply owns an authority
edge. It also forbids the inline exact lane from building L2 peak context,
reading mutable L3/L4/usage donors or waiting on a second queue. Design V3 is
`docs/structural_gates/preflights/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_ROUTE_V27_V3_2026-08-22.json`;
`route-design-v6.json` is `PASS` with zero issues and warnings. The V2 positive
receipts remain as superseded evidence, not implementation authority.

A second post-PASS critique tightened three remaining assumptions. First,
`layout_is_ru=false` is not proof of US QWERTY, so exact authority now requires
an admitted source-layout profile fingerprint and rejects unknown German/Dvorak
or other Latin profiles. Second, finite context fixtures cannot prove that a
late mutable L2/L3/L4 score will never move the target; the final contract
requires a property-level dominance invariant over generated competing
lattices and score perturbations. Third, the decision store is one
process-wide active-focus slot, not one retained slot per historical engine
context. A subsequent manual audit found that V3 omitted the declared
`NoApply(stage=Infrastructure)` execution branch. Route V4 adds the direct
full-producer failure terminal without rank, verifier or authority edges:
`docs/structural_gates/preflights/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_ROUTE_V27_V4_2026-08-22.json`.
Its `route-design-v7.json` receipt is `PASS` with zero issues and warnings.

The same audit rejected implementation-preflight V4 before code. It referenced
two undeclared tests, retained the superseded phrase `one slot per context`,
did not map the separate Space lookup and full-wait latency budgets, and did
not byte-pin the lexical inputs behind the fixed quality denominator. V4 and
its blocked receipt remain negative evidence. V5 must close those gaps and
bind non-US profile, cross-focus replacement, atomic slot publication and
process-wide resource tests before implementation can begin.

The complete contract, adversarial critique, denominators and promotion ladder
are recorded in
`docs/ime-atomic-exact-layout-lease-v27-paper-contract-2026-08-22.md`.
What was tested: V26 remote compilation and the first semantic negative gate,
plus the corrected V27 V4 design graph. The failed V26 result and superseded V27
design/preflight receipts are retained. What was not tested: V27 implementation, corpus
parity, latency, composite input, deployment or physical input. Runtime
authority changed: **no**. Installed runtime remains Lay `1.0.33`. Current
verdict: **V27_V5_MANUALLY_SUPERSEDED_V6_PENDING**.

#### V27 V5 post-PASS rejection and V6 closure

Implementation preflight V5 mechanically closed the V4 reference and latency
gaps and returned `READY_TO_IMPLEMENT` with 49 baselines, 28 invariants and 39
tests. It is retained as positive structural evidence but manually rejected as
implementation authority after source-level criticism found six unproved
boundaries.

First, the current IBus factory passes only `name == "lay-ime-ru"` into the
engine. Consequently, any unknown engine name becomes logical non-RU and could
be mistaken for US. V6 requires an immutable closed profile
`UsQwerty | Ru | Unknown`, minted only by exact factory name/component mapping.
`active_layout_is_ru=false` remains mutable decoder state and cannot mint that
profile; certificate authority requires both immutable `UsQwerty` and current
decoder state `Us`. Generated XML, both installed component XML copies and the
factory mapping require byte-/semantic-parity gates.

Second, byte-pinned corpora do not establish an independent expected result if
the generator imports runtime projection or certificate helpers. V6 requires a
standalone oracle with its own keyboard table and parser, no `lay` dependency,
pinned source/binary/compiler/normalization/table fingerprints, exact per-class
counts and a one-sided mutation test that must create detectable divergence.

Third, dominance must be typed rather than numerical. The proof enumerates all
current candidate lanes, origins, error classes, gate actions, transition
operators/proofs and legacy families through exhaustive Rust matches without a
wildcard. New variants fail compilation until classified. Score perturbation,
producer-order permutations and same-surface merge permutations are separate
denominators.

Fourth, `L2CandidateLattice` merges equal replacement strings. A closure bit in
the promoted producer fields could therefore be erased or inherited. V6 binds
an opaque `ClosedExactLayout` authority evidence value to canonical frame and
target bytes and requires a commutative/idempotent merge law; conflicts fail
closed.

Fifth, V5's `peak_context=None` wording was opposite to current source:
`TransitionDecisionCore::evaluate_candidates()` builds the peak context when
the option is `None` and also reads usage/L3/L4. V6 replaces the ambiguous
option convention with explicit `FullField | ClosedExact` evidence modes under
one DecisionCore owner. Only `FullField` has mutable-field dependencies.

Sixth, design receipt `route-design-v7.json` has
`source_evidence_verified=false`. After source edits and before quality/staging,
an observed-source contract must pin exact callsites and prove two material
producers, one typed constructor, one rank owner, one verifier, one mutation
owner and Rank/Verifier/Infrastructure NoApply routes. A paper topology PASS
cannot substitute for that receipt.

What was tested: route V4 design coherence and V5 implementation-preflight
mechanics. What was not tested: V27 source, factory/profile parity, independent
oracle, typed dominance/merge law, observed-source route, quality, latency,
physical input or deployment. Runtime authority changed: **no**. Installed
runtime remains Lay `1.0.33`. Current verdict:
**V27_V5_MANUALLY_SUPERSEDED_V6_PENDING**.

The resulting V6 implementation preflight stopped before code with exactly
three manifest blockers: no reused-source veto scans for a shared
oracle/runtime helper or staging without an observed-source receipt, and an
oracle/runtime comparison linked to a fault-injection test instead of a parity
test. This is negative gate evidence, not an architecture or runtime result.
V6 manifest and receipt remain immutable. V7 adds the two scans and separates
normal differential parity from the one-sided mutation fault. What was tested:
V6 baseline, reference and gate coverage. What was not tested: source,
compilation, quality, latency, physical input or deployment. Runtime authority
changed: **no**. Current verdict: **V27_V6_BLOCKED_V7_PENDING**.

#### V27 V7 manual rejection and V8 closure

Implementation preflight V7 returned `READY_TO_IMPLEMENT` with 78 baselines,
35 invariants and 49 tests, but it is not implementation authority. A manual
source attack found three remaining contract defects before code.

First, `L2CandidateSource::for_mode()` uses mutually exclusive arrays and
`NandaOnly` does not include `ExactLayout`. The shared closed-exact candidate
must enter a dedicated one-slot retained segment of the common
`L2CandidateLattice` before source-mode dispatch, deduplication, bounded
frontiers and top-k. Conflict, overflow or incomplete target evidence produces
`NoApply`; generic uncertainty cannot evict the retained candidate.

Second, current IME warmup does not initialize the exact English guard. The
guard is a `HashSet<String>` loaded from Hunspell and the plain dictionary. A
standalone probe loaded 139,370 entries and measured 11,796 KiB steady RSS
delta, 16,180 KiB process maximum RSS and about 0.04 s wall time before
user-protected extension. The user accepted this memory cost on 2026-08-22. V8
keeps the exact set instead of adding a probabilistic or separately compiled
index. EN/protection warmup becomes an explicit background startup dependency;
no partial snapshot or first-input initialization is allowed. Measured gates
are EN guard `<= 14 MiB` and total V27 incremental RSS `<= 16 MiB`.

Third, exhaustive dominance now also covers `CorrectionDecisionSource`,
`CorrectionSourceRole`, `CandidateReadoutRoute`, `ReplacementTargetEvidence`,
`LanguageActionOperator`, `EnumerationStateV1`, `CompletenessScopeKindV1` and
`IncompletenessReasonV1`, plus the new authority and evidence-mode enums.
Incomplete, overflow, failed or unproved-partition evidence cannot receive
closed-exact precedence.

What was tested: current source ownership, the warmup call graph and a
standalone RSS probe of the selected EN representation. What was not tested:
integrated V27 startup/RSS, implementation, quality, latency, physical input or
deployment. Runtime authority changed: **no**. Installed runtime remains Lay
`1.0.33`. V7 remains immutable positive mechanical evidence but is manually
superseded. Current verdict: **V27_V7_MANUALLY_REJECTED_V8_PENDING**.

V8 implementation preflight then stopped before code on one manifest-only
blocker: the EN guard snapshot identity contract referenced an integration RSS
test rather than a dedicated parity test. V8 manifest and receipt remain
immutable negative evidence. V9 separates snapshot byte/fingerprint/readiness
parity from startup latency/RSS integration. Architecture and runtime authority
did not change. Current verdict: **V27_V8_BLOCKED_V9_PENDING**.

V9 implementation preflight then returned `READY_TO_IMPLEMENT` with `85`
baselines, `38` invariants and `53` tests. Post-PASS review nevertheless rejected
it before code because the shared warmup surface did not prove a single process
owner for the accepted exact EN `HashSet<String>`. A live 2026-08-22 snapshot
measured about `539 MiB` raw RSS but `406 MiB` aggregate PSS across the active
Lay engine, daemon, L1.1 sidecar and L3 watcher; duplicate file-backed L2 pages
explain much of the RSS overcount, while a second EN HashSet would be private
memory. V10 therefore permits the new V27 exact-authority warmup only in
`lay-ibus-engine`, requires zero daemon V27 exact-guard PSS delta, exactly one
production warmup callsite, engine guard RSS delta `<= 14 MiB` and aggregate
active-Lay PSS delta `<= 16 MiB`. The generic daemon recognizer cannot publish
or consume V27 exact authority. V9 manifest and receipt remain immutable
positive mechanical but manually superseded evidence. Source and runtime
authority did not change. Current verdict:
**V27_V9_MANUALLY_REJECTED_V10_PENDING**.

V10 implementation preflight returned `READY_TO_IMPLEMENT` with
`safe_to_implement=true`, blockers `0`, `90` baselines, `18` source checks,
`21` preserved artifacts, `12` identity contracts, `39` invariants and `54`
tests. The separate post-PASS attack found no remaining paper architecture
blocker across retained exact birth, full/exact Space linearization, common
undo/feedback, and single-process EN ownership. The integrated resource proof
must report controlled per-process `Pss_Anon` delta separately from aggregate
active-Lay PSS delta. Implementation, observed source route, quality, latency,
physical input and deployment remain untested. Exact receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/implementation-preflight-v10.json`.
Runtime authority changed: **no**. Current paper verdict:
**V27_V10_PAPER_READY**.
