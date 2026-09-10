# TD-121 layout intent repair — bounded consequence (2026-09-06)

Status: `IMPLEMENTED / VERIFICATION_PENDING`, not independent acceptance.

Scope is limited to the existing background and direct layout-switch paths in
`src/bin/lay_ibus_engine/layout_sync.rs`, their dedicated tests, and one internal
`DeferredLayoutAction::CancelBackgroundSwitch` variant. It addresses
pass-1 High 6 without changing a text-correction/verifier route, the physical
Double Shift owner, exact replay, or GNOME's single layout activation.

The initial queued-request draft carried `AdmissionToken`. That token includes word
lineage, so a legitimate Space settlement can stale the queued external-layout
switch after the local decoder has already changed. Layout request identity must
instead bind the admitted context, owner generation, and a local monotonically
increasing layout-request generation; ordinary word-boundary lineage is not a
layout cancellation signal.

Before dispatch, a request whose layout generation is superseded or whose
context/owner binding no longer validates is refused without an external
activation. If the binding becomes conflicting while a switch is in flight, the
completion must not issue a compensating switch; it revokes text authority to
the Unknown route while retaining the one existing GNOME activation owner.
Blocking/manual compatibility remains on its existing direct route.

Superseded draft, not current implementation: the background worker and
direct/blocking route shared one activation gate. The
worker checks the live generation after popping a request and again after the
single external activation; a newer explicit intent first advances that same
generation. Thus a request superseded before its linearized dispatch is refused,
while an already-dispatched completion is not compensated with another switch
and its stale matching text authority is revoked. Execution remains pending.

Required focused proofs are: a Space boundary after enqueue retains the
legitimate request; a later request supersedes the earlier one; owner transfer
refuses the old request; and an in-flight conflict performs exactly one switch
and leaves no second text authority. This note records design consequences only;
it claims no execution, review, or TD-121 acceptance.

## Reopened parent-design finding (2026-09-06, subsequently confirmed below)

That draft was not admissible: all three entrypoints advanced the global
generation before testing `atomic.speculation`, so a discarded preview can
cancel or spawn background layout work. Its activation gate also spans external
IBus/GNOME RPC and can block a direct mutable engine callback for that entire
operation. Neither effect is permitted by the accepted bounded route.

Proposed minimal replacement: retain one short coordinator-state critical
section only to reserve/invalidate a generation; defer that reservation for
speculation until `apply_deferred_layout_actions()` after accepted settlement.
Do not hold a new mutex across an external call. Both direct and background
routes take a generation snapshot immediately before dispatch and check it
again after completion. A stale completion makes only its matching text
authority Unknown and never emits a compensating layout call. The physical
modifier route reserves generation but supplies no text-authority token, so it
continues to switch layout from `UnknownStart`; background automatic work uses
the context/owner token when one is live. Required new schedules are discarded
speculation with no reservation/worker effect, modifier-after-auto latest intent,
and completion superseded without a second activation. This proposal is not an
implementation or proof until parent confirmation.

## Confirmed repair route and bounds (2026-09-06)

The parent selected the short-reservation route (9/10): one bounded coordinator
state update reserves or invalidates a layout generation; no mutex is retained
across IBus/GNOME RPC. A speculative clone does neither operation. Its accepted
deferred action performs the reservation only after settlement; an accepted
no-switch intent uses one deferred cancellation action. Background auto-sync
requires a live layout token whenever the adapter is required. Direct physical
modifier switching reserves the same generation but may proceed from
`UnknownStart`; it captures a live token when available for pre/post dispatch
validation and matching stale-authority revocation. The established manual
route still publishes its tail before external activation, so factory/focus
handoff sees the source snapshot; a no-op publishes once and makes no RPC.

Rejected alternatives: retaining the activation gate (1/10: unbounded callback
blocking); allowing `(Some adapter, None token)` for background auto-sync (1/10:
authority bypass); and treating a stale completion as success or compensating
with a second switch (1/10: stale decoder/text authority or double activation).

Bounded consequences: candidate retention/ranking, verifier/SafetyGate, exact
replay, physical Double Shift, learning, packages, cache/delta reload, rollback
data, and daemon consumers are unchanged. The route changes only layout-intent
ordering and stale text authority. It adds no RPC-held lock, no new external
call, no background worker access from preview, and no latency/CPU/RSS/allocation
budget claim. Required proof dimensions are generation order, optional versus
required context binding, one-call stale completion, no decoder promotion, and
preserved modifier behavior from `UnknownStart`; all remain unexecuted pending
the serial remote suite.
