# TD-120 bounded repair pass 2 — consequence addendum

Date: 2026-09-06. Scope: repair only review findings H1, M1 and M2 within
the admitted `CurrentWord | LegacyReplayV1 | ExactReplay` design. This addendum
precedes the production edits. Runtime authority has not changed at this point.

## Facts and selected route

- H1 is a two-lock admission/capture race: an engine can pass its owner check,
  then clone another engine's shared snapshot. Admission and snapshot capture
  will become one shared-lock operation returning `None` for a foreign owner;
  settlement will additionally require the captured owner to equal the engine
  path. The existing base/live stamp remains the post-capture conflict gate.
- M1 is a boundary mismatch: `PreeditFastState` knows whether a lexical token is
  open, but `current_open_token_chars()` counts the whitespace-delimited tail.
  The repair will count the bounded suffix using the same lexical boundary
  predicate used by fast-token maintenance, without its 32-character display
  cap. Layout-letter punctuation remains part of the token.
- M2 is missing evidence, not a request for another transaction framework.
  Tests will use real atomic proposal/receipt entry points and real bridge inner
  methods against registered engine objects. Test-only callbacks/sinks may make
  the admission race and censored deferred feedback deterministic; production
  formats and routing stay unchanged.

Rejected alternatives remain: clearing on every punctuation/Backspace (breaks
layout-letter keys and replay), deriving length from the capped fast-token text
(breaks long tails), or adding request identity to V1 in this pass (a protocol
migration belonging to TD-122). No generic shared-state merge is introduced.

## Consequences and invariants

- Candidate/lattice retention, ranking and `SafetyGate` are unchanged. Removing
  a stale veto cannot itself authorize a correction; conversion quality remains
  a separate denominator.
- Hot-path cost stays O(1) over the already bounded 160-character tail. Atomic
  capture still takes one shared snapshot; the repair removes a redundant lock
  rather than adding RPC, polling, retry, workers, or deadline work. No material
  RSS/package-size change is expected; this remains unmeasured until the parent
  gate.
- Suppression identity and revision remain the only cache-like state. Package
  pointers, reload behavior and input-frame invalidation are unchanged.
- Accepted/reverted feedback is applied once only after compatible settlement.
  A rejected admission or owner/tail conflict censors speculative deferred
  layout and learning effects; it does not fabricate accepted/reverted labels.
- A handoff after capture is rejected by the existing base/live comparison; a
  handoff before capture is rejected atomically. ABA and foreign-owner snapshots
  cannot become a new owner or overwrite the live tail/guard.
- Failure is fail-closed: no proposal/pending record on foreign admission, no
  retry, and no rollback claim for an already submitted frame. Source rollback
  is limited to this helper/test change and needs no data migration.
- IME/daemon bridge consumers and public proposal/receipt formats remain
  compatible. `LegacyReplayV1` deliberately still has no request/token/phase
  identity; delayed/before-only leakage is a `KNOWN_RESIDUAL` tracked by TD-122,
  never counted as ordinary-lifetime PASS.
- Maintenance cost is one owner-checked capture helper plus focused fixtures;
  no new manager, owner, route, cache, fallback, service or source of truth.

## Fixed repair proof manifest

- `atomic.rs`: real foreign-owner admission/capture interleaving; normal
  proposal/abort/commit/duplicate; real bridge V1 arm, V2 arm/revoke, live
  consume, equal revisions, sensitive/cancel/foreign/ABA conflicts; nonempty
  speculative deferred effects censored and no second feedback/output.
- `td120_word_lifecycle_tests.rs`: successful `daemon_bridge` replacement after
  hard punctuation with no whitespace, exact one-character open-token length,
  full deletion retirement, unchanged left context, fresh next word; retain
  layout-letter punctuation and 160-character controls; active-composition
  Enter and terminal passthrough boundary feedback.
- Narrow daemon caller tests, only where an existing effect seam permits:
  configured/reselection/native/exact dispatch call counts, before-only abort
  and replay failures, duplicate calls, replay with/without boundary, delayed
  word/owner schedules, and latent `ReplaceText` versus current `ReplayAll`.
  Any unexecuted native branch remains named rather than inferred.

Denominators must remain separate: ordinary lifetime; V1 receiver/caller
compatibility; V1 known residuals; exact replay; atomic normal; atomic guard
conflict; atomic tail/owner conflict; feedback; and conversion applies/refusals/
false applies. No Cargo, service, configuration, installed-binary, live-keyboard,
release, graph refresh or runtime-authority action is performed by this pass.

## Second bounded M2 completion preflight

Review of the first repair checkpoint accepted H1/M1 but found the M2 proof
still partial. Before the additional daemon test seam, the allowed change is
narrowed again: inject results only at the existing native IME/GNOME and
output-effect calls, while executing the existing selection/reselection and
`ReplaceText | ReplayAll` branch control unchanged. The seam is test-only state
or private closure injection; production calls remain the default and public
protocols do not change. A3 fixtures must start with a real atomic proposal,
then run actual ContentType or bridge cancellation callbacks and inject a
nonempty deferred feedback action into that real pending proposal. V1 delayed
cases must make a real bridge call after the new word/owner exists.

This adds no runtime owner, queue, retry, fallback, cache, request identity or
transaction manager. The risks are test-hook leakage into production and branch
duplication; guards are `cfg(test)`, the seam calls the production branch
function, and tests assert production-default behavior remains selected. CPU,
RSS, allocation, latency, lattice/ranking, packages/reloads, learning semantics,
rollback and IME compatibility are otherwise unchanged. If the seam cannot run
the actual branch without duplicating it, that case remains an explicit gap
rather than being promoted from a source ledger. V1 identity remains TD-122.

## Architecture owner-name collision

The source-bound architecture gate found a substring collision between each
public owner name and its private injected helper: the `_with` helper names were
counted as second owners. The selected repair is a symbols-only rename to
`execute_native_output_with_effects` and
`execute_manual_text_replacement_with_effects` (9/10). Changing the generic
owner scanner is a separate architecture-policy scope (7/10); removing the
seams would discard the accepted M2 branch proof (2/10). Function signatures,
branches, effects, authority and CPU work remain unchanged. Rollback is the two
private symbol names and their local call sites only.

## Source-contract successor repair

The canonical changed gate found three stale source contracts after the accepted
M2 sequencing seam. The selected bounded repair (9/10) keeps authority and
effect ordering strict: the monopoly tests bind authorization before
`run_replay_effects`, then bind the real Preflight, Backspaces, Replay and
SuppressAfterSuccess effects to their production calls. The Replay branch also
binds its cloned `input_gate` trace to `apply_layout_replay`. Restoring arming in
wrappers is rejected because it breaks success-only settlement; blindly
rebaselining or skipping the contracts is also rejected.

TD-120 intentionally supersedes only TD-113's byte-frozen `ime-mutation`
artifact to add current-word arm/retirement at completion boundaries. The
immutable TD-113 V4 preflight remains historical evidence; a compact successor
binding records its exact predecessor identity and the reviewed TD-120 source.
The other seven protected policies gain no exception. This is provenance plus
source-contract maintenance, not functional acceptance: the existing active-
composition/completion tests and 8/10 source review remain separate, and the
canonical execution receipt must still pass. Runtime branches, effects,
authority and CPU work are unchanged. Rollback removes the successor binding
and restores the two contract-test marker blocks.
