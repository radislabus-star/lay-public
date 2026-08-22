# IME Backend Atomic Receipt V1

Date: 2026-08-20
Status: superseded before implementation by `ime-backend-atomic-frame-v2-2026-08-20.md`
Runtime authority changed: false
Installed runtime touched: false

The V1 two-call `ProcessKeyEvent -> take` design is retained as rejected
evidence. Source inspection found that it leaves downstream transaction
capability underspecified and creates avoidable fetch, stale-take and duplicate
take states. No IBus implementation was written under V1.

## 1. Decision

The two measured local synchronous storage strategies are rejected:

```text
buffered ext4 write_at + fdatasync
-> prepare maximum 325.387 ms
-> FAIL maximum <8 ms

direct aligned O_DIRECT + O_DSYNC
-> prepare/co-commit maximum 330.324/162.240 ms
-> FAIL maximum <8 ms
```

No third local-storage strategy is admitted. Slice 7 returns to the other
already permitted `TransactionDurabilityStrategyV1` branch:

```text
BackendAtomicReceiptV1
```

The selected design candidate is
`IbusSynchronousPostProcessReceiptV1`. It uses the existing IBus synchronous
post-process owner as the transaction boundary. It is not a claim that current
IBus is already atomic and it does not authorize source edits or deployment.

## 2. Observed Current Route

Current Lay emits separate IBus signals:

```text
active composition
-> clear_preedit
-> CommitText

committed-tail correction / double-Shift rollback
-> DeleteSurroundingText
-> CommitText

cursor-bearing committed-tail plan
-> ForwardKeyEvent left*
-> DeleteSurroundingText
-> CommitText
-> ForwardKeyEvent right*
```

Source owners:

- `/home/ubu/projects/lay-l1-exact-peak-search/src/bin/lay_ibus_engine/composition_commit.rs`
- `/home/ubu/projects/lay-l1-exact-peak-search/src/bin/lay_ibus_engine/state.rs`
- `/home/ubu/projects/lay-l1-exact-peak-search/src/bin/lay_ibus_engine/committed_tail.rs`
- `/home/ubu/projects/lay-l1-exact-peak-search/src/bin/lay_ibus_engine/ibus_interface.rs`

The standard IBus protocol defines `CommitText`, `DeleteSurroundingText`,
`UpdatePreeditText` and `ForwardKeyEvent` as distinct signals. An unmodified
async route therefore has a kill point after any strict prefix of the effect
vector.

## 3. Upstream IBus Evidence

Upstream repository and immutable revision:

```text
repository  https://github.com/ibus/ibus.git
commit      4d31a1346fc8ac2063b49a2f9d70853d17057be2
```

Installed baseline:

```text
package     ibus 1.5.34~rc2-1
source      Ubuntu archive ibus_1.5.34~rc2-1
```

The Ubuntu source package was downloaded from the archive and extracted with
its Debian/Ubuntu patch series. Its four owning files independently reproduce
the queue, error and void-return behavior below. Upstream `main` is retained as
supplemental evidence only; implementation must start from the installed source
package bytes.

Pinned source facts:

1. `bus/inputcontext.c` has a bounded `queue_during_process_key_event` and sets
   `processing_key_event` when synchronous post-processing is enabled.
2. Commit, delete, preedit and forwarded-key operations are encoded into that
   queue while one `ProcessKeyEvent` is active.
3. `PostProcessKeyEvent` returns the queued operation vector only after the
   engine method finishes.
4. The current engine-error path resets `processing_key_event` but does not
   explicitly discard a partially collected vector.
5. The current client API returns `void` from
   `ibus_input_context_post_process_key_event`, so a failed vector fetch cannot
   change the already obtained handled decision.
6. The current queue check warns at `MAX_SYNC_DATA` but is not a fail-closed
   whole-frame overflow contract.
7. GTK and IBus Wayland sync clients already call post-process immediately
   after `ProcessKeyEvent`, but apply the returned operations one by one.

Pinned hashes are stored in
`docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_RECEIPT_V1_DESIGN_2026-08-20/ibus-upstream-source-manifest.json`.

## 4. No-Go Lemma For Current Async Signals

Let one authorized edit require ordered effects `E1, E2, ..., En`, where
`n >= 2`, and let each effect be emitted in a separate D-Bus signal. If the
engine can terminate after emitting `Ek` and before emitting `Ek+1`, the client
can observe a strict prefix of the intended effect vector. The method failure
does not prove whether that prefix was delivered or applied. Therefore neither
`handled` nor unhandled native replay is safe without an independent complete
frame owner.

Consequences:

- `DeleteSurroundingText -> CommitText` has no current
  `BackendAtomicReceiptV1`;
- `clear_preedit -> CommitText` has no current complete-vector receipt;
- a successful return from one signal emission proves transport enqueue only,
  not complete edit application;
- surrounding-text observation after the fact is useful evidence but cannot
  reconstruct an intent that was never transferred to an independent owner.

This is a route-level limitation, not a word-specific correction defect.

## 5. Proposed Backend Transaction

```text
InputEventIdentityV1
-> AuthorizedEdit + exact BackendAtomicCapabilityV1
-> ProcessKeyEvent starts AtomicPostProcessFrameV1
-> Lay emits typed effect operations only into that frame
-> engine success seals the complete frame
-> client takes the sealed frame by exact transaction identity
-> client validates operation set, count, order and current capability
-> one client batch owner applies the complete vector
-> client chooses handled only after successful frame take/apply
-> postcondition receipt enables success/learning
```

Types:

```text
BackendAtomicCapabilityV1 {
    input_context_identity,
    client_adapter_identity,
    client_adapter_build_hash,
    sync_post_process_enabled,
    supported_effect_kinds,
    maximum_effect_count,
    delete_failure_is_zero_effect,
    downstream_transaction_kind,
}

AtomicPostProcessFrameV1 {
    transaction_id,
    input_event_identity,
    focus_lineage,
    authorized_edit_digest,
    ordered_effect_vector,
    effect_vector_digest,
    disposition_if_applied,
}

BackendAtomicReceiptV1 {
    transaction_id,
    input_event_identity,
    client_adapter_identity,
    effect_vector_digest,
    result: AppliedExact | RefusedUnconsumed | FocusLineageTerminated,
}
```

## 6. Required IBus Semantics

The implementation may proceed only if an isolated patched IBus proof provides
all of the following:

1. Every mutation signal produced during an admitted key event is captured in
   one frame and is never forwarded early.
2. Engine method error, connection loss, cancellation, unsupported operation or
   queue overflow discards the whole frame.
3. The sealed frame is taken once by exact transaction identity; stale or
   duplicate take is rejected.
4. Failure to fetch a sealed frame forces the client key handler to return
   unhandled and applies zero frame effects.
5. A client returns handled only after it owns the complete vector.
6. `DeleteSurroundingText` refusal stops before `CommitText` and is proved to be
   zero-effect for the admitted client profile.
7. Success and learning are published only from a client/backend receipt, never
   from engine signal emission.
8. Any client without an exact capability receipt is native-only for the
   unsupported mutation family.

## 7. Failure Matrix

| Failure point | Required result |
| --- | --- |
| capability absent or stale | zero mutation, original event unhandled |
| frame already active | zero mutation, original event unhandled |
| operation count overflow | discard whole frame, original event unhandled |
| engine death before success | discard whole frame, original event unhandled |
| engine death after seal | daemon/client frame owner continues independently |
| daemon death before frame reply | client receives no complete frame, original event unhandled |
| daemon death after complete reply | client owns complete frame and decides disposition |
| frame checksum or identity mismatch | zero mutation, original event unhandled |
| delete callback refuses | stop before commit, original event unhandled |
| client/focus dies during apply | lineage terminates; no retry, learning or cross-focus compensation |
| acknowledgement missing | no success publication or learning; never replay |

## 8. Route Restrictions

The first proof scope admits only cursor-neutral text operations:

```text
CommitText
Update/HidePreedit
DeleteSurroundingText followed by CommitText
```

It excludes:

- cursor movement through forwarded key events;
- terminal control-character erase;
- layout mutation;
- synthetic key down/up sequences;
- timer, worker or `SetSurroundingText` callback mutation outside
  `ProcessKeyEvent`;
- fallback from a failed atomic frame to the old committed-tail mutator.

The current `SetSurroundingText` callback can invoke auto-undo. That mutation
must become a deferred intent consumed by a later admitted input event; an
observation callback cannot be a second mutation owner.

## 9. Client Capability Scope

There is no global IBus capability claim. Each product client class needs its
own receipt:

```text
GNOME Wayland text-input transaction
GTK synchronous IBus module
Qt synchronous IBus module
terminal profile
Electron/Chromium profile
```

The GNOME/Wayland route is promising because compositor text-input protocols
can publish delete, commit and preedit changes as one downstream transaction.
GTK and Qt need separate proof of synchronous callback behavior and
delete-refusal semantics. A PASS for one client never promotes another.

## 10. Gates Before Code And Promotion

```text
design route gate                                      PASS
observed-source route gate                             PASS
implementation preflight                  READY_TO_IMPLEMENT
isolated engine-death prefix matrix                    100%
isolated daemon-death frame matrix                     100%
duplicate/lost original event                             0
partial external effect                                   0
unsupported-client mutation                               0
success/learning without backend receipt                   0
frame take p99 / max                           <=2 / <8 ms
integrated Space hot p99 / max                  <=5 / <8 ms
physical client classes                    each independently PASS
```

Only after the isolated proof passes may Slice 7B integrate the backend owner.
The installed `1.0.33` runtime remains unchanged until the full physical gate.

## 11. Structural Gate Result

The first route packet was retained with `VETO`: it mislabeled an evidence
transfer as producer output and reversed two proof edges. No code was written
under that packet.

The corrected V2 packet passed the design-only code-route gate:

```text
verdict                                      PASS
execution/authority/observation/proof        separated
singleton mutation owner                     client_batch_apply
issues / warnings                            0 / 0
safe_to_edit                                 false
ready_for_implementation_preflight           true
source evidence verified                     false
```

Receipts:

- `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_RECEIPT_V1_DESIGN_2026-08-20/route-design-v1-receipt.json`
- `docs/structural_gates/receipts/LAY_IME_BACKEND_ATOMIC_RECEIPT_V1_DESIGN_2026-08-20/route-design-v2-receipt.json`

The V2 PASS permits an implementation preflight, not implementation or
deployment.
