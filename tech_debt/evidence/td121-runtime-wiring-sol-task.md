# TD-121 — final engine wiring handoff

Working checkout: `/home/ubu/projects/lay-tech-debt-20260831`.
Target: release 1.0.66, not Wave-quality work for 1.0.67.

This is an implementation handoff, not a PASS or a second architecture.
Read AGENTS.md, the owning TD-121 task, its complete context-admission analysis
including the final rendezvous addendum, and the current pure-helper evidence.
The selected design and consequences are already written. Do not reopen a
general actor/transaction migration or invent a new correction route.

## Entry conditions

- Pure reducer/real-zbus merge/rendezvous gates have passed remotely: original
  17 plus two corrected-contract tests, 19/19 on exact zbus 5.15.0. Receipt:
  `/home/e/projects/td121-proof-JGik2wUq/td121-test-zbus-5.15.0.log`, SHA-256
  `78d533125613d56fe23453a08a768dd9edaae0a23e346221adfeef1e5cf9df3f`.
- That result does NOT cover the subsequently staged observer/acquisition
  adapter, new-field activation or settled-lineage APIs. Read their separate
  exact-source results before relying on them.
- The subsequent frozen stage passed 30/30 on exact zbus 5.15.0: 25 reducer,
  merge and rendezvous tests plus 5 controlled actual-zbus p2p adapter tests.
  Manifest: `/home/ubu/.cache/lay/td121-prewire-7iGwHB/FROZEN_STAGE_MANIFEST.md`.
  Remote log: `/home/e/projects/td121-proof-XU19ZyCn/td121-adapter-test-zbus-5.15.0.log`,
  SHA-256 `b56a1d7ec2d5b3f465e07c37d192a0bf154b971c787e437c759f108779e84300`.
  The parent read all 30 test statuses. This supersedes the 19-test helper-only
  checkpoint for staged code, not the pending runtime/client proof.
- Wait for the main agent's TD-120 source checkpoint before editing its shared
  files; do not overwrite the concurrent suppression/atomic repair.

## Integration ownership map

- `server.rs` / `factory.rs`: construct the single connection-local admission
  owner outside speculative SharedState, subscribe before factory publication,
  bind each factory request's receive stamp to its new engine path.
- `ibus_interface.rs`: correlate real method headers, capture callback-entry
  time, perform only bounded local stamp rendezvous, then existing key delivery
  and separate settlement. Observe source keys before source sealing. Native
  focus payload, legacy acquisition, old FocusOutId/Reset/Disable and reconnect
  converge on the selected generation contract. Preserve implicit per-engine
  zbus locking; do not claim the write lock is released during an await.
- `engine.rs` / `state.rs` / `tail_memory.rs`: constructor text copies are not
  transfer authority. Admit latest sealed source text only once in the same
  canonical context, rebind TD-120 ordinary suppression with the same word
  incarnation, and leave exact transport proofs distinct. A failed/old source
  callback cannot clear or publish into the new owner's state.
- `preedit.rs`, composition and managed boundaries, manual methods and
  learning: sticky UnknownStart prevents suffix-only apply, Tab, whole-word
  projection and word feedback. Deliver literal input once. Save the closed
  word's old completeness before rearming the NEXT word at a real boundary.
  Backspace-to-empty is not a boundary; crossing the rearming boundary revokes
  completeness. Use existing boundary/layout-symbol semantics, not a second
  ASCII punctuation rule.
- Frame/prefetch: no candidate/prepared/Tab authority survives owner change.
  Charge time since callback entry to the existing 3,500 us Space allowance;
  never add the rendezvous budget on top or increase the product deadline.
- `bridge.rs` / `bridge_actions.rs`: marker-fenced IBus admission before
  authority-bearing session-bus calls, then revalidate after engine lookup.
  Existing typed NotHandled must end at daemon Complete(None), not fallback
  to its word buffer. Status may be Unknown; replay-capable tails may not.
- `atomic.rs`: retain TD-120 same-lock base capture and guarded settlement;
  additionally capture/revalidate live admission generation. Never clone or
  overwrite the connection-local observer with the speculative SharedState.
  Discarded work emits no second output, layout request or learning event.
- `layout_sync.rs`: bind pending/in-flight requests to current owner/context/
  request generation. Stale work is not new decoder or text authority. An
  already dispatched switch cannot be cancelled retroactively; no compensating
  second switch. Preserve GNOME's single-action ownership and Double Shift.

The helper's transfer now preserves source word lineage, but rotates owner,
activation and frame. Native receipt may share the FocusInId receive position;
only the later compatibility Get requires a strictly later reply position.
Do not regress these two repaired contracts in wiring.

### Entrypoints that must not inherit an unproved stamp claim

The frozen adapter's `begin_key_callback` accepts only `ProcessKeyEvent`.
`ProcessKeyEventAtomicV1` is a distinct existing exclusive protocol, and
`ContentType` changes arrive through `Properties.Set`, not that key callback.
Before editing their wiring, identify how the selected live-generation capture,
settlement and revocation contract covers them. Do not route them through a
helper that rejects their method name, silently bypass admission, claim the
30-test fixture observed those headers, or add per-key context RPCs. Preserve
normal atomic commit as well as revoke/foreign-owner refusal; test sensitive
content cancellation and stale-owner no-write against the actual entrypoints.
Any necessary adapter API extension is a new delta to the frozen stage and
requires its own explicit tests; it is not covered merely by copying the stage.

## Test and scope requirements

Implement the declared C01-C30 matrix against actual new methods, not a second
test-only reducer. Preserve distinct denominators for pure admission, transport,
engine state/output, bridge admission, atomic settlement, boundary/feedback,
resources and client-visible behavior. Expand H01-H16 as required by the owning
task; table row counts do not mean that many tests were executed.

Positive cases must include full observed `ljv` and mixed `lом` as separate
inputs, same context/new object and same-object controls, cached false and
native focus. Assert retained prefix, exact emitted output and left context,
plus no transfer to another field, foreign ABA or stale generation. A direct
exact-layout proof is not general Wave heldout quality or real physical input.

One metadata source, no second text owner, no literal word/suffix/test/source-id
runtime rules, no weaker SafetyGate/verifier, no per-key RPC, key queue, retries
or model/package changes. Keep test-only p2p support a dev dependency where
possible; direct adapter utility imports require explicit dependencies.

## Execution

Sol implements; main agent owns remote build lease, graph refresh, fresh-context
Astra review, explicit task staging/commit/push and release/install. No local
Cargo or test suites. Remote dedicated-20cpu uses CARGO_BUILD_JOBS=20 and the
existing serial/process-isolated test contract. Do not use nanda-structural-gate
or create filesystem artifacts under `/root`. Use apply_patch; preserve other
work. Tests not yet executed must remain NOT_RUN, not green by inspection.
