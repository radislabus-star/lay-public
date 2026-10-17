# TD-139 — Учёт owned preedit в старых private-client проверках

Status: DEFERRED_STAGE2_DISCUSSION. Priority: P1. Owner: existing repository
IME-client consumer and its proof contract; no product-runtime change selected.
Depends: exact candidate/baseline failure packets and existing C03/C05/C07/C10.

## Наблюдаемый отказ

Unchanged V3 lifecycle3 on candidate82/5fe100db stops in the first case before
case completion: expected VisibleTailV3 ` j`, actual state active:composition,
text `j`. Same unchanged harness/configuration/dependencies on installed81/2bd
also stops at that exact assertion. Both have0/3 completed cases; neither is
relabeled PASS or a TD133 regression. Original candidate full3048+11 source
PASS remains a separate denominator. Restoration5 was not reached.

Private receipts:
`/home/ubu/.cache/lay/development/td133-private-client-6tugjijb/`
and `/home/ubu/.cache/lay/development/td133-private-client-46qcok0a/`.
Candidate trace shows visible committed separators unchanged, a real
`printable_legacy_preedit` callback and UpdatePreeditText/show `j`, not lost
Space. `bridge_snapshot` reads VisibleTailV3, whose existing composition source
intentionally returns owned composition only. Client advertises PREEDIT_TEXT
but its general Client class listens only for CommitText/Delete/Forward; it
models committed text as the entire visible surface. Specific preedit cases
already attach their own observer. This is the first demonstrated shared
proof-consumer mismatch; frequency/other lanes are unmeasured.

## Варианты

- Teach the existing client a strict committed-text plus owned-preedit surface
  and adapt affected semantic oracles with explicit route/ownership evidence:
  9/10, recommended for a separately agreed proof revision.
- Freeze an explicitly committed-input-only compatibility client with a
  capability/profile contract it can honestly consume: 6/10; loses preedit
  coverage and cannot replace original lifecycle/control assertions.
- Disable product preedit or accept arbitrary bridge strings: 1/10; changes
  accepted behavior or removes the invariant instead of fixing the consumer.

## Минимальная согласуемая реализация

1. Freeze V3 driver/run pins, candidate/baseline/config/dependencies and both
   failures. Record committed projection, exact preedit events and input state
   in one chronology. Do not infer product quality or client acceptance.
2. Add preedit state to the existing Client only: text, cursor, visible flag,
   update/show/hide behavior and a derived surface at its existing caret. Never
   feed uncommitted preedit into SurroundingText. Keep native replay once,
   strict delete geometry, callback multiplicity, owner/context and key schedule.
3. Write focused semantic RED tests: composed text with prefix; hidden preedit;
   empty update; focus loss; commit consumes the owned composition once;
   duplicate commit; incorrect deletion/owner; unsolicited suffix. Define text
   and preedit cursor/selection units explicitly. Add Reset/disable/destroyed
   or superseded-context revocation and late update/show after revocation as
   RED cases. Use actual observer/consumer methods, not a second editor simulator.
4. Version the proof when scenario oracles change. Retain immutable V3 identity
   in Git and exact predecessor provenance. Migrate only demonstrated obsolete
   assertions; require exact projected surface/caret, unchanged prefixes,
   revoked word authority at refocus, zero unsafe edits and useful successor
   input. A nonempty/startswith assertion cannot substitute for exact evidence.
5. Run candidate and old-byte characterization separately, then the complete
   unchanged lane list under the existing remote guards. Report3 lifecycle,
   5 restoration, terminal, manual, startup and inverse denominators separately.
   Do not turn historical FAILs into PASS or count skipped scenarios.
6. Fresh-context review1–2 passes maximum. Update decision/owning document,
   driver pins and harness tests together; commit/push after its own gates.

## Подводные камни, ограничения и откат

VisibleTailV3 is an authority projection, not the complete widget buffer.
Composition and committed prefixes have distinct owners. Preedit can be shown
before a callback settles, hidden without a commit, or cleared on focus loss.
Do not grant deletion authority from render/observer state. Preserve existing
startup deadlines, post-verdict-only trace drain and isolation of all four
usage paths. No extra warmup, key delay, retry or product fallback is selected.

This task changes only proof-consumer representation; ranking/lattice, models,
feedback and IME delivery stay unchanged. Bound observer memory and retain
drop/error counters. Repository tests run remotely; this task grants no native
input or installation. Revert the proof revision/pins together if ownership
or event multiplicity cannot be represented strictly. Native64 remains TD133.
Implementation belongs to stage2 and requires discussion; this card is an
evidenced proposal, not a new prerequisite silently added to the Space repair.

Fresh-context staged-plan review9/10: [report](evidence/2026-10-10-td133-candidate-execution-plan-review-pass2.md). Cursor units and late preedit after Reset/disable/context revocation are now explicit RED angles. No implementation selected.
