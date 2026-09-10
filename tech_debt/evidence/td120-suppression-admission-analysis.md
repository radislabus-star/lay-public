# TD-120: admission analysis and selected ordinary-word lifecycle contract

Date: 2026-09-05. Source baseline: `cc1e2207519801ca0f9b7c6963897b55953a7751`.
Worktree: `/home/ubu/projects/lay-tech-debt-20260831`.
This is source/design evidence, not a test receipt or production admission by
itself. No Cargo, probe, service, installed binary, configuration or production
source changes were performed. `runtime_authority_changed=false`.

## Decision

Select the **ordinary-word lifecycle correction**, scored **8/10 for its stated
scope**, instead of the speculative 9/10 unified-owner migration in the original
TD-120. The main agent explicitly selected this narrower scope during this
analysis. It fixes the established ordinary IME producer → complete deletion →
new word suppression mechanism. It preserves the existing transport protocols
and atomic output/receipt protocol, with bounded suppression reconciliation at
the existing settlement. It does **not** close universal V1 lifetime,
canonical cross-context transfer, or shared atomic settlement.

Use a permanent typed distinction between `CurrentWord`, `LegacyReplayV1`, and
`ExactReplay`; replace the ambiguous boolean/optional-exact representation rather
than adding another boolean beside it. Keep the current local/shared storage
topology for this stage; this is not migration of every transport producer into
a new single shared owner. Ordinary local state is a mirror of one guard
incarnation and cannot independently resurrect a guard consumed from shared
state. The full transport-owner consolidation remains an explicit Stage-2 debt.

This result selects a design for TDD/implementation review. The fixed tests,
remote checks and independent code review must still pass before the narrower
implementation can be accepted. The parent must amend the owning TD-120 scope
and README with this decision before production edits; the old G120-1/2/3 must
not silently become PASS or disappear from the overall debt denominator.

## Exact producer inventory and reachability

Paths below are under `src/bin/` unless otherwise specified. Line numbers refer
to the source baseline, not a future implementation.

| Producer | Baseline behavior | Selected scope |
|---|---|---|
| `lay_ibus_engine/shift.rs:96` active composition manual toggle | Local boolean after `commit_verified_active_composition`; no shared ordinary publication here | `CurrentWord`, armed inside the successful commit path; typed edit intent, no string-kind inference |
| `lay_ibus_engine/shift.rs:104` `defer_committed_tail_manual_toggle_to_daemon` | Local boolean before any daemon output; no success proof | `LegacyReplayV1` compatibility state; delegation is never `CurrentWord` success |
| `lay_ibus_engine/state.rs:585` `replace_committed_tail` | Local/shared ordinary booleans set **before** duplicate check and output | `CurrentWord` only after successful output and final tail update; failed/rejected/duplicate/no-op operations do not newly arm |
| `CommittedTailReplaceRequest::ime_manual_toggle` (`state.rs:100`) | Caller provides suppression bit; terminal and surrounding paths have separate output contracts | Same typed intent and output contracts; protect only the resulting open word |
| `CommittedTailReplaceRequest::ime_auto_undo` (`state.rs:120`) | Suppression true, possibly replacement including trailing Space | No new guard when the restored result already closes its word |
| `CommittedTailReplaceRequest::ime_candidate_accept` (`state.rs:136`) | Suppression true; accepted committed-tail candidate may have trailing Space | Same open-word rule |
| `CommittedTailReplaceRequest::daemon_bridge` (`state.rs:152`), bridge `ReplaceTail*` | Guard requested by `bridge_policy.rs` kinds; actual edit still passes typed transition/verifier and output | Success in this direct IME replacement route is `CurrentWord`; do not confuse it with daemon uinput transport |
| `lay_ibus_engine/composition_commit.rs:89` active candidate acceptance | Successful typed candidate commit; currently does not set ordinary suppression | Explicit candidate acceptance without boundary gets `CurrentWord`; with boundary does not. This closes the same lifecycle contract, not a lexical change |
| `lay_ibus_engine/bridge_actions.rs:120` no-argument V1 RPC | Resolves `active_path`, gets its engine lock, consumes exact handoff, unconditionally arms local/shared nonexact state | `LegacyReplayV1`; preserve current transport behavior and report its known limitations |
| `lay_ibus_engine/bridge_actions.rs:139`, `tail_memory.rs:629` V2 | Exact suffix/path/layout/epoch/handoff admission; consumes handoff and stores deadline | `ExactReplay`; preserve all admission, timeout and exact cancellation rules |
| `lay_ibus_engine/bridge_actions.rs:190`, `tail_memory.rs:677` V2 cancellation | Exact path/epoch match revokes transport guard | Revoke that exact scope only; no ordinary lifecycle inference |

The daemon has **exactly two production call sites** for no-argument V1:

1. `lay_daemon/correction_runtime/output.rs:89`, after native stages and uinput
   availability, **before** uinput preparation and text-replacement/replay branch.
2. `lay_daemon/correction_runtime/output/replay.rs:59`, only after successful
   `replay_keycodes`, before recording replay success.

Both call `layout_controller.rs:381` which discards the result of the checked
helper. That helper calls the RPC only when `active_text_backend().should_try_ime()`.
V1 itself supplies no request, originating context, token, phase or success data.

The route is not universally unreachable under the IBus backend:

- `manual_trigger_runtime/ime.rs:38` maps `DelegateDaemon` to `ConfiguredBackend`.
- `output/native.rs:58` allows reselection only when the IME dispatch result
  explicitly permits it; `native_stage.rs` can then return no selected native
  output, reaching uinput and V1. Native failure after dispatch does not fall
  through. These cases must be distinguished in characterization.
- `trigger_dispatch.rs:43` creates no physical grab for `ConfiguredBackend`.
  Therefore serialization of the daemon function is not proof that user input
  or another focus callback cannot precede the V1 after-call.
- The separate `DaemonUinput` route skips native output, requires physical
  isolation and captures a delegated tail lease. A non-IBus backend skips the
  V1 RPC. Pending daemon undo returns through its dedicated undo path before
  these two call sites.
- `exact_ime_tail_replay.rs` uses checked V2, not either V1 call site. Do not move
  it onto the compatibility V1 scope.

### Before-only, after-only effects, and text replacement

The first V1 call may stand alone after authorization/plan/preflight refusal,
Backspace failure, or replay failure. In the last case partial synthetic input
may already exist. There is no V1 cancellation. Successful replay calls V1
again, whether replay contains a trailing boundary or not. A boundary processed
between calls can consume the first guard; the second can then protect a later
token. Duplicate RPC calls are uncorrelated new arming operations.

`output/text_replace.rs` has both success and failure returns with no second V1
call. **This branch is currently production-unreachable from the only caller**:
`correction_runtime.rs:147–189` forces `CorrectionEngine::Replay`,
`force_replay=true`, `auto_replace=false`, yielding `DecoderAction::ReplayAll`.
Keep a branch characterization case and a caller contract test; do not label
that branch as a observed live failure or assume it can never be reactivated.

Delayed after on a different word/owner is admitted by the RPC: it resolves the
current path at execution. Even the path lookup and asynchronous engine lock
acquisition have no second active-owner check in the baseline. This is a static
reachability result, not a new reproduction or attribution of the archived
`gjxbnfq` episode.

### Why existing V1 cannot prove universal token lifetime

An identical no-argument call can mean “prepare replay of A”, “replay of A has
finished”, or a delayed completion while B is current. The receiving engine has
no information to distinguish these histories. Protecting B preserves the last
history's baseline behavior but violates universal word lifetime; rejecting it
can lose replay protection in the other histories. Counting calls, inspecting
text, or adding a timeout does not provide the missing request identity.

Preserving V1 **can coexist with correct ordinary-word lifecycle as separate
typed scopes**. It cannot coexist with a claim that every suppression in the
system has exact token-lifetime semantics. This is the unavoidable scope
boundary, not an unresolved implementation choice for Sol.

## Alternatives and consequences of selection

Scores are engineering judgments, not measured quality or test results.

| Design | Score | Decision |
|---|---:|---|
| Existing undifferentiated booleans | 3/10 | Confirmed leak and pre-output arming; no lifecycle contract |
| Clear all suppression at any Backspace/empty tail | 2/10 | Breaks partial-word intent and synthetic deletion in replay |
| Typed ordinary lifecycle, existing transport semantics and storage topology | **8/10 scoped** | Selected. Finite change, no new daemon protocol/worker/transaction owner; compatibility limitations remain explicit |
| Full single shared scoped owner while preserving no-arg V1 universally | 4/10 now | Not implementable under the claimed universal contract; V1 ambiguity and stale atomic settlement do not disappear by replacing the data structure |
| Replace two V1 calls with existing V2 | 5/10 | Not a wrapper substitution: V2 requires an exact suffix, target path/layout/epoch and existing exact handoff. Generic delegation does not establish that handoff; native reselection may lack physical isolation. Never weaken V2 to admit it |
| New checked begin/finish/cancel replay request using the existing owner | 7/10 for Stage 2 | Viable direction only with explicit event settlement. A returned RPC after `replay_keycodes` does not prove the IME has processed queued replay keys. Needs request identity, origin/focus binding, isolation, completion observation and old-daemon compatibility policy; larger than this root fix |

The new protocol's concrete eventual gain is attributable cancellation of
before-only failures and refusal of late after for another request. Its cost
is not merely two methods: it must establish the end of synthetic event
processing without a second mutation, hidden retry or text-based phase guess.
No version of that protocol is silently authorized by this document.

## Implementable ordinary-word contract

1. Represent armed state explicitly as `CurrentWord`, `LegacyReplayV1`, or
   `ExactReplay`; absence means no guard. Retire old boolean/optional-exact
   branches in the same change. Do not maintain old flags as fallback authority.
   Type names are descriptive, not a requirement to create a new public module.
2. `CurrentWord` has a fresh incarnation/word generation and the current runtime
   owner identity. Local and shared copies identify the same incarnation.
   Consumption of ordinary state is authoritative in shared state; a local
   mirror alone cannot rearm or consume an already spent shared incarnation.
   Transport local/shared consumption retains its existing contract for this
   stage. A generation is not `tail_epoch`, which advances on every publish.
3. Create ordinary protection only in the successful typed edit/accept path,
   after output success and tail synchronization. Move the current
   `state.rs:585` arm after the duplicate check/output/update. A helper returning
   `Ok(())` can also mean rejected/no-op today: do not treat the wrapper's `Ok`
   as sufficient success proof; arm inside the branch that actually committed.
4. A completed replacement with a terminal token boundary does not arm an open
   word. Plain Space, Enter including active-composition Enter, actual hard
   punctuation boundaries, full deletion and text-lease loss retire the matching
   ordinary incarnation. Partial deletion/insertion within the same open word
   preserves it. A subsequent identical string is a new incarnation.
5. Use the existing lexical boundary decision in `preedit.rs::push_tail_char`,
   including `ascii_layout_symbol_continues_token`. No literal word/suffix rules
   or unconditional punctuation splitting. **Do not use
   `last_tail_token_text().is_empty()` as the entire deletion check**:
   `tail_memory.rs:1065` trims trailing whitespace, so deleting `x` from `old x`
   still finds `old`. Track the open token's lifetime (or its bounded open-token
   length) and retire at the empty current token, not only empty whole history.
   Composition cursor edits and tail trimming must preserve that accounting.
6. Complete feedback for the current word before boundary retirement. Preserve
   existing censored/accepted/rejected attribution, undo records and verifier
   authority. Guard expiry alone never means an accepted correction or a
   negative label for another candidate.
7. A matching Space consumes `CurrentWord` once, then the existing managed Space
   path commits the boundary. An old engine may retire its local mirror but
   cannot clear a newer shared ordinary incarnation. Compare owner plus
   incarnation under the shared lock, not text equality or only engine path.
8. Ordinary transfer in this narrow stage follows only the already-admitted
   baseline handoff route; move the same word generation and rebind the runtime
   owner when that route actually transfers the matching tail. This is
   **baseline parity, not canonical-context proof**. If TD-121 installs its
   stronger handoff contract in the same release, consume that decision;
   suppression must not independently admit/reconstruct a handoff. No global
   IME PID restart, FocusId capability change or new context RPC belongs here.
9. Soft Reset that preserves the same tracked word preserves ordinary state;
   callback handling from another engine cannot clear the new owner's ordinary
   guard. For unknown continuity, TD-121 owns loss of candidate/text authority.
   This stage does not falsely certify that a cleared buffer starts a known
   complete word after an unproven handoff.
10. Never let ordinary full deletion/word-boundary helpers retire
    `LegacyReplayV1` or `ExactReplay`. Exact synthetic deletion can transiently
    empty the tail; preserve exact source/target admission, deadline and revoke.
    Terminal erase/commit remains one IME output frame without physical replay.

Scope-limited owner checks above concern ordinary state. They must not be
advertised as a general repair of all shared tail/handoff writers. In particular,
`close_committed_tail_field` and handoff publication also modify unrelated
shared authority; TD-121 owns their canonical ownership discipline.

## Atomic settlement: established reachability and minimal treatment

`atomic.rs:119` clones SharedState into an isolated Arc. `:196–205` later
replaces live SharedState and the whole engine with that clone. The pending
receipt is bound to transaction identity and daemon focus epoch; there is no
shared suppression generation comparison. A per-engine zbus mutable lock only
serializes one RPC, not the interval between proposal and next receipt.

| Interleaving while engine A has pending speculation | Source evidence | Consequence |
|---|---|---|
| A legacy key / ReplaceTail / manual bridge mutator | `engine.rs:191`, `bridge_actions.rs:242,289` | Blocked while atomic active; cannot claim these ordinary producers race on A |
| A FocusIn/Out, Reset, Disable | `ibus_interface.rs:52,81,145,163` | Discards A's pending transition; old receipt cannot commit that removed proposal |
| A V1 arm | `bridge_actions.rs:120–137` | No atomic guard; new local/shared transport state can be overwritten by old speculation |
| A V2 arm / exact revoke | `bridge_actions.rs:139–214` | No atomic guard. V2 arm also requires a still-live exact handoff; prepare of a key that did not consume that handoff can coexist. Revoke can retire an existing exact guard between proposal and receipt |
| A ContentType becomes sensitive | `engine.rs:232–247` | Clears tail/suppression without `discard_atomic_pending`; old clone can restore earlier state. Existing general settlement risk, not caused by TD-120 |
| A newer SurroundingText | `ibus_interface.rs:186–206`, `atomic.rs:183–227` | Ordinary observation revision is reconciled explicitly, but visible-postcondition mismatch can also quarantine shared state. Snapshot reconciliation is not suppression settlement proof |
| Exact-handoff cancellation | `bridge_actions.rs:171–187,306–323` | Shared-only method clears handoff tail/focus without acquiring A's engine lock; it can invalidate a prepared tail even inside another engine RPC |
| B FocusIn and/or B producer before A FocusOut | Different object locks; `bind_focus_path` changes shared owner, B callbacks discard only B pending | Old A pending can survive until A receipt. Same-engine serialization does not prove shared-owner serialization |

These are executable API/source interleavings, not newly measured production
failures. The current repository's Rust daemon has no production call site for
`ProcessKeyEventAtomicV1`; the exported interface and compositor adapter route
remain supported. “The current daemon does not call it” does not justify deleting
its tests or declaring it unreachable to an installed adapter.

**Selected narrow-stage treatment, refined after the parent's atomic parity
requirement:** add suppression reconciliation to the existing settlement;
do not leave old-clone resurrection as an accepted new-patch outcome. Keep the
exported atomic proposal/receipt formats and no-retry rules unchanged.

1. Add one internal `suppression_revision` to SharedState. Every scope arm,
   consume, revoke and explicit clear updates it, including V1/V2, matching
   owner transfer and ordinary lifecycle helpers. A speculative mutation changes
   only the isolated shared clone. Capture the **base live** revision with the
   deep clone and retain it in `PendingAtomicTransition`; comparing live to the
   speculative final revision is wrong when each fork independently advances.
2. Also capture the existing active path and handoff tail stamp from that same
   locked snapshot: epoch, focus receipt and bounded tail buffer. These are
   stale-state detectors, not a new canonical context receipt. Preserve/check
   the local runtime owner lease too. No new global transaction owner is needed.
   All production active-path writes are `bind_focus_path` and FocusOut. An
   A→B→A switch requires A FocusIn, which discards A's pending transition before
   rebinding; assert that fact in the callback-route test rather than assuming
   path equality alone solves arbitrary ABA.
3. At settlement, first validate the existing transaction receipt. Under the
   live shared lock, check the captured owner/tail stamp and capture the current
   suppression block. Keep this check and shared-state replacement in one
   critical section; do not create a check-to-write race. An atomic request
   from an engine that no longer owns the active path must not prepare another
   mutating clone after this refusal.
4. If owner/tail stamp is unchanged and live revision equals the base revision,
   apply the speculative state normally. If only suppression revision changed,
   apply the speculative tail/output state and retain the newer live suppression
   block, including its local mirror and revision. The block includes the
   existing exact replay preserve deadline/handoff epoch/path fields that V1,
   V2 and exact cancellation mutate with suppression. Preserve the new scope
   exactly, including `None` after consumption/revocation. This avoids both
   guard loss and resurrection; do not OR an old `CurrentWord` into the result.
5. If the live owner or tail stamp changed, do not assign either speculative
   SharedState or speculative engine. The pending entry is already removed;
   return the existing refusal/native-unhandled path for the **new** key, with
   zero resubmission of the prior frame. Keep all newer live shared state,
   guard and owner. Retire only stale local cached authority without publishing
   old tail/guard back to shared state. For the same owner, synchronize local
   tail metadata from the newer shared handoff before later preparation; for a
   foreign owner, detach local authority and await normal FocusIn admission.
6. Treat prior submitted text as already submitted, not rolled back. On this
   conflict path, speculative deferred layout/learning actions are not applied;
   label their feedback outcome censored/uncertain in the existing trace. Do not
   fabricate an accepted/reverted learning event. Existing newer SurroundingText
   reconciliation remains in its present order on successful settlement.
7. A repeat receipt finds no pending entry and cannot restore the old guard or
   run feedback again. Tests must assert that normal no-conflict atomic frames
   and the next native key still occur exactly once.

This is bounded reconciliation of fields directly involved in the suppression
change. It covers ordinary guard versus reachable live transport mutations and
owner/tail invalidation. It is not a general merge of arbitrary SharedState
fields, configuration callbacks or factory allocation state. The original
G120-2 remains open for the **full unified-owner migration** until that broader
contract is separately demonstrated; the scoped atomic matrix below is a
mandatory acceptance gate for this patch.

## Consequence analysis for the selected patch

| Dimension | Bound, risk and required evidence |
|---|---|
| Candidate/lattice retention, rank | No generator, lattice, scorer or dictionary changes. Removing a stale ordinary pre-decision veto permits the existing pipeline; it does not prove `почитай` wins |
| False authority / safety | No Apply authority added. Ordinary guard owner/incarnation checks prevent stale local consumption. V1 and general handoff limitations remain visible; SafetyGate/verifier/edit-plan validation unchanged |
| Latency / tail | O(1) lifecycle updates; existing bounded tail scan at an edit may be reused. No per-key RPC, sleep, worker or Space deadline change. Measure focused latency/allocation regression separately |
| CPU/RSS/allocations | Small tagged state and integer identity; reuse owner identity without per-key String cloning. No new cache, event history, timer or persisted storage. Do not clone the sentence to discover word lifetime |
| Cache identity/invalidation | Word generation differs from per-publication epoch. Existing prefetch/display certificates still invalidate on input/owner changes; suppression is not a candidate certificate |
| Package/delta reload | Guard has no package pointer or lexical-version dependency. Reload still invalidates correction leases through their existing generation |
| Learning/feedback | Retire after boundary feedback; duplicate/failed output does not newly arm or duplicate learning. Active candidate acceptance preserves typed accepted target semantics |
| Concurrency | Ordinary mirror incarnation is explicit; bounded atomic base-revision/owner/tail validation preserves newer guards and rejects stale tail settlement. Generic cross-engine publishers and unrelated SharedState fields are not newly certified |
| Failure/rollback | Failed edit preserves prior valid guard but creates no new one. Partial transport failure retains existing transport semantics. Rollback is a source commit, without changing dictionaries or learned data |
| IME/daemon compatibility | Same public V1/V2/ReplaceTail/ManualToggle APIs and exact/terminal routes. Baseline admitted handoff parity required; no arbitrary new replay route |
| Maintenance/removal | Typed scopes are the permanent vocabulary. No shadow booleans or fallback owner. Stage 2 can consolidate storage and replace legacy protocol using the same scope distinction; it must explicitly retire V1 compatibility, not accumulate another transport scope indefinitely |

## Fixed TDD expansion before production edits

Use real engine/bridge/output methods and assert client surface, guard scope,
owner/incarnation, verdict/output count and feedback count. Start with baseline
characterization, then freeze the expected fixed ordinary cases before patch.
Tests below are requirements, **not executed results**.

| Group | Cases to instantiate | Required outcome |
|---|---|---|
| O1 ordinary producer | Active manual, committed manual, accepted committed candidate, accepted active candidate, undo, typed daemon ReplaceTail | Successful open-word result protected once; no new guard for rejected/no-op/duplicate/output-failed operation; previous valid guard is not accidentally erased |
| O2 full deletion | Each applicable ordinary producer → delete exactly token length; extra Backspace; same-text retype; different-text retype; history `old x` → delete only `x`; active composition cursor-backspace to empty | Ordinary guard absent before new word, previous history not mistaken for the deleted current token; next word never inherits it |
| O3 continuing word | Append, delete one of several characters, insert/remove inside active composition, backspace after tail trim | Same incarnation remains protected while its word stays open |
| O4 boundaries | Space twice, active and committed Enter, real punctuation, accepted/manual/undo output with trailing Space; layout punctuation `;`, `[`, `]`, apostrophe in applicable US/RU context | Correct ordinary retirement after feedback; layout-letter punctuation does not spuriously retire. Terminal passthrough boundary must retire even though it does not run the managed consumer |
| O5 owner/handoff parity | Baseline admitted handoff A→B; new owner consumes then old A attempts consume/clear; soft reset; stale A callback while B holds newer ordinary guard | Transfer same incarnation only through admitted existing route; one consumption; old A cannot erase B ordinary guard. Canonical same-context claim remains gated to TD-121 |
| O6 user symptom | Actual complete `gjxbnfq` frame after former-word manual edit + full deletion, all three safety profiles, clean controls | First assert no inherited ordinary suppression, then separately assert actual correction/verifier result under each profile. Do not count guard removal as conversion PASS |
| V1-1 reachable routes | Configured backend: IME applied, IME refused without reselection, explicitly permitted reselection, non-IBus backend, exact-tail delegation | Exactly the admitted V1/V2/native call counts and no second mutation; validate that the V1 fallback branch is reached only by the existing reselection contract |
| V1-2 compatibility | First call then preflight abort, Backspace failure, replay failure, duplicate calls, successful replay with/without trailing Space, delayed after another word/owner | Preserve baseline transport semantics; explicitly classify cross-word/delayed-after leakage as KNOWN_RESIDUAL, never ordinary-lifetime PASS |
| V1-3 latent text branch | Construct ReplaceText branch success/failure and freeze the real production caller's ReplayAll decision | Before-only count is characterized; branch not mislabeled as current production-reachable; enabling it later changes this compatibility denominator |
| E1 exact transport | Exact V2 → intermediate complete synthetic deletion → replay with/without trailing boundary; wrong suffix/path/epoch/layout; deadline expiry; matching/wrong revoke | Existing exact admission and protection preserved, no ordinary empty-token cancellation and no physical terminal Backspace |
| A1 normal atomic | Preview ordinary arm/retire/consume; abort; compatible submitted receipt; consumed-no-effect; duplicate receipt | Live state untouched before settlement; one committed transition/feedback, no retry and no duplicate guard consumption |
| A2 changed guard atomic | Prepare → V1 arm; prepare → valid V2 arm; prepare → exact revoke; base guard → live consume/arm/revoke combinations; identical numeric revision increments in live/spec forks | Newest live scope/mirror/revision survives exactly, including None; old ordinary scope never resurrects. Use base revision, not final speculative revision. Assert actual bridge admission for reachable RPC cases |
| A3 invalidated tail atomic | Prepare → sensitive ContentType; prepare → exact-handoff cancellation; A prepare → B FocusIn/producer → A old receipt; A→B→A callbacks | No speculative assignment over newer owner/tail/guard; zero second output/deferred layout/feedback; next-key native refusal counted separately from prior submitted output. A→B→A discards A pending through actual FocusIn |
| C1 unchanged controls | `physical_double_shift_owner_`, existing manual/exact suppression, composition edit and atomic proof suites; package reload between input and Space | Existing route/safety/profile behavior retained; no stale correction authority after reload |

Report independent denominators: ordinary cases passed/total; transport
compatibility cases passed/total; known residual V1 cases; atomic normal cases;
atomic guard-conflict and tail-conflict cases; canonical handoff cases gated; actual conversion
applies/refusals/false applies; neighboring-word changes; feedback events.
Do not drop original S06b/S07b/S12/S16 requirements from the overall TD-120
record merely because only their scoped subsets are admitted now.

## Handoff and evidence status

Planned production files are `protocol/state.rs`, `engine/state_groups.rs`,
`engine.rs`, `state.rs`, `shift.rs`, `composition_commit.rs`,
`composition_edit.rs`, `preedit.rs`, `managed.rs`, `tail_memory.rs`,
`committed_tail.rs`, `bridge_actions.rs`, and `atomic.rs` under
`src/bin/lay_ibus_engine/`. These are the existing producers/lifecycle/consumer
and settlement owners, not a request for a new manager module. Touch only files
whose listed behavior needs changing; daemon production code and public bridge
formats need no change for this stage. Add tests next to these owners and in the
existing daemon route tests for V1 reachability/text-branch characterization.

The parent reported a successful installed-byte private baseline after this
source analysis: 6 executed cases, manual toggle then deletion and a new
`gjxbnfq` ended in `manual_toggle_suppressed_OBSERVED`. Receipt path:
`/home/ubu/.cache/lay/layout-phase2-private-31hE0X/receipt.json`, parent-reported
SHA-256 `d75822dd10425f422b5f7a2d5b942eeb79e8f742c913446936bd740483acd82c`.
This agent did not rerun or independently hash that receipt. The probe disabled
autocorrect/model work, so it strengthens mechanism reproduction only; it does
not prove post-fix conversion quality.

Source navigation used Graphify's existing graph, expanded with vocabulary
`suppression autocorrect atomic shared replay bridge`, then bounded source
reads. Graph output was insufficient to prove control flow; every reachability
conclusion above was checked against source. No graph rebuild was performed in
this delegated analysis because its only allowed write was this evidence file.
The parent owns the required architecture-document integration and graph refresh.

Read owning TD-120, the layout audit and spec review passes 1/2. Their existing
review scores establish document quality only. This analysis changes the
selected implementation scope; it is not a third claim that the original
unified-migration admission gates have passed.

Remote Rust TDD, aggregate existing regression checks, fixed frame proof,
independent code review, commit/push and any separately authorized live proof
remain unperformed by this agent. Installed/private baseline evidence belongs
to the parent investigation and must be cited from its actual receipt.
