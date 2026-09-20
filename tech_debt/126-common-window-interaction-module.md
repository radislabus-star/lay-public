# TD-126 — Extract one common window interaction module

Priority: P0 (explicit user priority: only the common window module)
Status: `DONE_SOURCE_ONLY / RUNTIME_UNCHANGED`.
Plan reviews were 6/10 `REQUEST_CHANGES` then 8/10 `ACCEPT`; implementation
reviews were 7/10 `REQUEST_CHANGES` then 8/10 `ACCEPT`. Each review track used
one grouped repair and exactly two passes; no third review is claimed. The user
selected the recommended extraction with «дальше» after the full plan
discussion. Final acceptance details are in the
[durable closure report](evidence/td126-final-acceptance.md).
Scope: the managed IBus window/context boundary from client observation through
admission, execution or delegation, later outcome observation and existing
settlement/revocation. This is not desktop automation and does not promise every
application works.

During implementation, authority was limited to TD-126 source extraction, its
finite tests, remote guarded checks and owning documentation. Install,
activation, runtime repair, service restart, input-source change, commit and
push were outside that then-active authorization. That boundary is historical:
the user has since authorized the ordered task loop through `DONE`, commit and
push. Git checkpoint status, exact commit and the verified
`origin/codex/cleanup-20260908` remote ref belong to the external
`/home/ubu/.cache/lay/development/td126-publication-20260913/publication.json`;
this source document does not duplicate that transition state. Temporary
candidate activation requires a ready owned field/capture and activation
preflight. Permanent installation and physical promotion remain separate and
require the corresponding client and real-behavior checks to pass.

Plan review record: pass 1 scored 6/10 and required correction of route
semantics, callsite closure, RAII/partial failure, LOC accounting and proof
design. Those findings were repaired together. Pass 2 scored 8/10 and accepted
the architecture with no material replan finding. These plan scores are
separate from the later code-review scores recorded below.

## Implementation authorization and consequence check — 2026-09-12

The selected option is the reviewed real extraction. The first implementation
stage freezes the current dirty source identity because it already contains the
accepted exact native replay work; TD-126 must preserve those bytes and change
only the documented module closure.

Frozen pre-production-edit baseline:

- Git HEAD: `83ba0f42db5c87cdc4f17abf42197396566cb05b`;
- 1,042 source/test/script/owning-document files;
- manifest:
  `/home/ubu/.cache/lay/development/td126-common-window-before-20260912-V1/source-manifest.tsv`,
  SHA-256 `82f3a71932df3ca5183e0a7857ea84024ce1084d815b828519a8621d94fb00a1`;
- deterministic backup:
  `/home/ubu/.cache/lay/development/td126-common-window-before-20260912-V1/source-before.tar.gz`,
  SHA-256 `56b38bf935ee8ac44c18c8631b0dfe5dc9eda32fd52d20f0620d99a6aed26260`.

Consequences are bounded as follows. Candidate generation, ranking, L1.1–L4,
`SafetyGate`, verifier and edit-plan authority do not move. Existing feedback is
still admitted only by the existing later postcondition. Reducer ordering,
callback sequence identities, 500/1500 ms postcondition windows, focus-time
config reload, package/material generations and daemon physical replay remain
unchanged. The extraction adds no owner, cache, timer, queue, generation,
fallback, app-name branch, asynchronous worker or runtime route. Stack-local
effect progress only distinguishes failure before mutation from possible
delete-without-commit; it grants no retry or authority. Expected CPU/RSS and
allocation change is limited to typed enum values and moved calls, but remains
unmeasured until remote gates. Rollback is the TD-126 module/delegate/test diff;
the frozen pre-edit source is the exact recovery boundary.

The highest regression risks are async callback reordering, losing
`ContextBridgeOutput` Drop revocation, treating daemon dispatch as visible proof,
and changing terminal/atomic/composition/autoUndo priority. The ten named tests
and affected remote gate address these risks. Any material route or authority
change outside this list stops implementation for review.

## Focused-gate diagnosis and consequence amendment — 2026-09-12

The first complete TD-126 snapshot was
`/home/ubu/.cache/lay/development/run-7_xqay_4/source.tar` (SHA-256
`687cafddbfdc32857fe01878a003fe18b66869e7e3ae653063744441c1c0e90c`),
with remote run `/home/e/projects/lay-development-runner/run-YlHw4j` and copied
binary log
`/home/ubu/.cache/lay/development/run-7_xqay_4/tests/logs/bin-lay-ibus-engine.log`.
Discovery and compilation passed. Of 484 target-isolated tests, 100 completed
PASS, 14 completed FAIL, and the next test aborted on stack overflow; the abort
prevented the harness from printing captured assertion blocks for the 14
failures. A same-snapshot residual diagnostic preserved the one reproducible
assertion in `tests/logs/diagnostic-residuals.log`: the selected-snapshot case
returned V3 `(0, false)` instead of the pre-existing typed exact `(3, false)` at
`residuals.rs:3311`. The other 13 first-run failures passed both the isolated
residual diagnostic and the next complete snapshot, so their exact assertion
locations are not claimed. They remain a first-run async/test-process
observation rather than evidence for thirteen runtime fixes. Their cause is
`UNKNOWN`; scheduling or prior process state is only a hypothesis.

The reproducible behavior drift has one shared staging cause. The extracted
`manual_toggle_outcome_inner` asks `admit_manual_toggle` to reject a selected GUI
snapshot before the existing `VisibleTailV3` source-capture stage. The frozen
original returns typed exact delegation at that point and lets source capture
reject an absent, selected or boundary-mismatched snapshot before daemon replay.
The repair restores that sequence: manual-toggle admission may report
`DelegateExactImeTail` for the existing committed-tail owner, while the existing
source-capture lease still refuses selection and emits no text effect. This is
not visible-success authority, does not bypass `admit_replace`, and does not
allow any local destructive edit against a selection.

The second complete diagnostic snapshot is
`/home/ubu/.cache/lay/development/run-hvkra22o` with remote run
`/home/e/projects/lay-development-runner/run-gei863`. It reproduced only the
typed-exact selection drift before the same stack overflow. Measured current
object/future sizes were: `LayIbusEngine` 2,672 bytes,
`process_key_event_with_output` 11,536 bytes,
`WindowInteraction::process_legacy_key` 11,984 bytes,
`observe_facts` 4,464 bytes and `observe_lifecycle` 1,240 bytes. The new legacy
wrapper therefore adds 448 bytes to that async future. The failing test future
was 51,136 bytes; the frozen pre-extraction source measured 85,328 bytes. GDB
showed one linear poll chain through `legacy_key -> ProcessKeyEvent ->
process_legacy_key -> process_key_event_with_output -> process_pressed_key`;
that trace contained no recursive call cycle. Future and object sizes do not
measure the complete poll-stack high-water mark, and this evidence cannot
exclude the wrapper frame or fact-event union from contributing. The bounded
observation is that the old combined fixture overflowed after extraction, while
the split fixtures later exercised all the same scenarios and passed. The exact
poll-stack cause remains unmeasured.

Changing `RUST_MIN_STACK`, guard budgets, baseline policy or test selection is
forbidden. Boxing the callback future is also rejected: it would add a heap
allocation and indirection to every production legacy callback, add allocator
jitter to input latency, and introduce a different cancellation/drop boundary
without changing authority. The selected repair is test-only: split independent
refusal scenarios into a second named test, preserving every assertion and the
same production path. Its effects are one manifest test identity and a smaller
test future; production allocation, latency, cancellation, owner, storage and
runtime authority effects are zero.

The repaired pre-review focused snapshot is
`/home/ubu/.cache/lay/development/run-nl27bh04/source.tar` (SHA-256
`e20730621c3b519ff44ca1402dd26132495873da33605ae752ad94dbdcd9a1b0`),
with remote run `/home/e/projects/lay-development-runner/run-4gkC8h`.
Formatting passed. The focused correctness denominator was `487/487`: `485`
tests ran in the target process and two correctness tests ran with
`isolation=process`; three performance tests were excluded by the selected
correctness lanes. All `487` passed and none failed. The copied evidence packet
is `/home/ubu/.cache/lay/development/run-nl27bh04/tests/`, including both
process-isolated logs. This receipt predates the four source/test cleanup edits
made for code review, so it is a repaired-route checkpoint, not the final
affected-gate receipt. Those cleanup edits require coverage by the final
post-review affected gate.

## Root cause

The system has the necessary owners, but no single module owns their interaction
sequence. IBus callbacks decode window facts in `ibus_interface.rs`; the existing
`client_context` state is declared in `engine/state_groups.rs`; snapshot types
and capability rules are split across `engine/types.rs` and `engine.rs`;
authenticated focus, Reset, key-callback and admission choreography lives in
`context_runtime.rs`; later client postcondition observation lives in
`tail_memory.rs`; bridge reads, mutations and cancellations live in
`bridge_actions.rs`. Local committed-tail execution remains in `state.rs`, while
the physical exact-tail executor is in the daemon.

The new `text_target.rs` is only a backend selector plus Reset re-receipt data.
It does not bind observe → admit → execute/delegate → later observation →
existing settle/revoke into one interface. Callers can still repeat or bypass
parts of the lifecycle, which is the maintenance problem the user asked to solve.

## Current routes that must not be conflated

The common boundary reports these existing routes truthfully:

1. **Delegated physical exact IME tail.** In `shift.rs:81–93`, a non-terminal
   `ImeCommittedTail` prepares the exact handoff and returns no local result.
   `bridge_actions.rs:509–515` exposes this as
   `ImeManualToggleOutcome::DelegateExactImeTail`. The daemon later calls
   `execute_exact_ime_tail_replay()` in `lay_daemon/exact_ime_tail_replay.rs`.
   That physical uinput executor returns `Option<bool>` for layout completion; it
   has no client-text receipt. The module must never relabel it as a GTK local
   delete/commit or claim visible success from daemon completion.
2. **Local terminal erase.** The IME calls its already-proven terminal
   erase/commit executor locally. This is transport execution, not an observed
   client postcondition.
3. **Other existing local delete/commit.** An exact, unselected
   `SurroundingText` route may call the existing committed-tail executor. Its
   later proof comes only from the existing pending postcondition observed in a
   subsequent `SetSurroundingText` callback.
4. **Delegated daemon buffer.** `DaemonWordBuffer` returns typed delegation and
   emits no local text effect.
5. **Active composition, pending autoUndo and AtomicV1.** These remain supported
   existing routes with their current owners. They are not blanket
   `Reject(Atomic)` or evidence that the window is unsupported.

The physical Double Shift detector is daemon-owned. `shift.rs` owns the focused
IME manual-toggle priority and local routing after the daemon invokes the bridge;
the plan must not call that the physical detector.

## Alternatives

Scores are engineering judgments for this bounded task, not measured quality or
performance.

1. **4/10 — thin facade.** Low immediate diff, but orchestration remains
   callable and scattered, so the original maintenance defect survives.
2. **9/10 — selected: real extraction behind one module boundary.** Move the
   existing interaction orchestration and make wire/bridge callsites thin
   delegates. Keep the reducer, pending postcondition, leases and executors as
   the only owners.
3. **3/10 — rewrite admission/output as a new framework.** This duplicates
   proven authority, increases async and cancellation risk, and violates the
   no-second-owner constraint.

Recommendation: option 2, implemented as behavior-preserving extraction plus a
small typed result layer. Do not add a facade while leaving old orchestrators
publicly callable, and do not redesign policy during the move.

## Selected module and API

Use one Rust module directory so the extraction does not create a new god file:

```text
src/bin/lay_ibus_engine/window_interaction/
  mod.rs          public crate-internal boundary and result types
  observation.rs  window facts, lifecycle callbacks, later postconditions
  authority.rs    read guards, admission, suppression and cancellation
  execution.rs    bridge RAII, local execution and delegation receipts
  tests.rs        finite end-to-end boundary tests
```

Delete `text_target.rs` after moving its useful selector, Reset re-receipt types
and tests. Keep the existing field name `client_context`; move its existing type
and mutations only where needed. A mass rename to `window` adds no value.

The crate-internal boundary is explicit rather than a generic window framework:

```text
WindowInteraction::observe_lifecycle(engine, WindowLifecycleEvent)
    -> LifecycleReceipt
WindowInteraction::observe_facts(engine, WindowFactEvent)
    -> ObservationReceipt
WindowInteraction::read_visible_tail(engine, bridge_token)
    -> VisibleTailReceipt
WindowInteraction::admit_manual_toggle(engine, bridge_token)
    -> TextTargetAuthority
WindowInteraction::execute_local(engine, authority, request, output)
    -> Result<ExecutionReceipt, LocalExecutionFailure>
WindowInteraction::execute_manual_toggle(engine, authority, output)
    -> Result<(Option<bool>, ExecutionReceipt), LocalExecutionFailure>
WindowInteraction::observe_existing_postcondition(engine)
    -> OutcomeProof
LayImeBridge::{visible_tail_v3_inner, replace_tail_inner, manual_toggle_v3_inner}
LayImeBridge::{cancel_exact_manual_toggle_handoff_v2_inner,
    cancel_exact_manual_toggle_suppression_v2_inner}
```

`WindowLifecycleEvent` covers FocusIn/FocusOut, Disable and Reset.
`WindowFactEvent` covers capabilities, content purpose/hints, cursor geometry and
decoded `SurroundingText` including logical selection. The two named bridge
cancellation entrypoints keep the existing exact handoff and suppression state;
there is no unused generic cancellation enum or second state owner.

```text
TextTargetAuthority =
    DelegateExactImeTail(existing handoff projection)
  | LocalTerminalErase(existing edit authority)
  | LocalSurroundingDeleteCommit(existing edit authority)
  | DelegateDaemonBuffer
  | ActiveCompositionOwned
  | AtomicOwned
  | Reject(WindowRejectReason)

ExecutionReceipt =
    DelegatedExactImeTail        # no physical or client success claim
  | DelegatedDaemonBuffer        # zero local output
  | LocalComplete
  | LocalPending                 # existing IME owner awaits exact observation
  | LocalCancelled               # even-pair/local cancellation, no fallback
  | LocalIndeterminatePartial    # an earlier effect may have reached the client
  | LocalErrorBeforeMutation
  | Rejected

OutcomeProof =
    ExistingPostconditionConfirmed
  | ExistingPostconditionPending
  | ExistingPostconditionMismatch
  | ExistingPostconditionCensored
  | Rejected
```

All values project the existing `AdmissionToken`, owner/path, tail epoch,
surrounding observation revision, exact handoff/suppression state or executor
result. No new persistent receipt cache, token, renewal generation, timer or copy
of client text is introduced. `execute_local` revalidates current owners under
the existing exclusive engine guard.

`ObservationReceipt` across a later callback is a typed view of the already
stored `pending_visible_postcondition`; it does not store a second pending
record. `tail_memory.rs:468–552` already arms and consumes that state, preserving
its 500 ms settle grace, 1500 ms observation limit, epoch check, exact/boundary-
elided match, mismatch quarantine, layout sync and feedback recording. The
module moves that orchestration intact and reports its status. It does not
invent a new meaning of settlement or renewal.

## RAII and partial-effect contract

Move `ContextBridgeOutput` from `context_runtime.rs:18–82` into
`window_interaction/execution.rs` without weakening it. Its `Drop` must clear the
bridge token and revoke the context word unless `complete()` was reached.

Local `replace_committed_tail` may successfully emit `DeleteSurroundingText`
and then fail to emit `CommitText` (`state.rs:594–613`). Such an error is not
zero mutation and cannot settle authority. The local leaf reports stack-local
effect progress to the boundary. A failure after any successful effect becomes
`LocalIndeterminatePartial`; RAII remains incomplete, revokes existing word
authority and leaves a later existing observation to establish what the client
shows. No persistent progress state or retry executor is added.

Cancellation, early return, bridge token mismatch and dropped futures preserve
the same fail-closed RAII behavior. A controlled injected commit failure after a
successful delete must prove this path.

## Full callsite closure

Existing authority owners remain internal:

- `ContextAdmissionReducer`/`ContextAdmissionAdapter`: connection, callback
  ordering, owner, focus activation, lineage, revocation and token validity.
- Existing `client_context`, `context_owner`, `context_token`, word scope,
  exact-handoff/suppression state and `pending_visible_postcondition` storage.
- Committed-tail, terminal and atomic leaf executors; daemon `WordBuffer` and
  daemon physical exact replay; edit plan, `SafetyGate`, verifier, model/lattice
  and learning/feedback owners.

The following old orchestration entrypoints move behind or become thin delegates
to the new boundary:

- `ibus_interface.rs`: FocusIn/FocusInId, FocusOut/FocusOutId, Disable, Reset,
  SetCursorLocation, SetCapabilities, ContentType and SetSurroundingText decode
  wire values, call the module once and emit the returned effects.
- `context_runtime.rs`: `ContextBridgeOutput`, focus/disable/revocation
  observation, key-callback begin/settle/no-input settlement, Reset re-receipt
  capture/arm/observe/allow/consume and `revoke_context_word`. Reducer calls stay
  internal and retain their current sequence tokens and deadlines.
- `bridge_actions.rs`: context-sensitive `VisibleTailV3` and its live read guard,
  `owns_active_text`, `CanReplaceCommittedTail`, legacy/V2 suppression,
  exact-handoff and exact-suppression cancel paths, `replace_tail_inner`, and
  `manual_toggle_outcome_inner`. Object-server path lookup and acquisition of the
  exclusive engine guard remain in the bridge; after acquisition, the module is
  the only interaction boundary.
- `engine.rs`, `engine/types.rs`, `engine/state_groups.rs`: move capability,
  sensitive-content, surrounding snapshot and target-decision logic while
  retaining the single existing `client_context` field.
- `tail_memory.rs:468–552`: move existing postcondition orchestration; keep its
  pending storage and low-level tail/suppression mechanics as the authority
  owner.
- `shift.rs`: keep autoUndo, active composition and local plan priority. Its
  committed-tail/daemon decision delegates to the boundary.
- IBus key callbacks: call the boundary around the existing
  `begin_context_key_callback`/settle sequence and feed native exact-replay
  observation into it. The module observes admission and outcome; the typing
  brain, physical daemon executor and native replay contour do not move.

Leaf executors remain internal and are not reimplemented:

- `state.rs::replace_committed_tail` for local surrounding/terminal effects,
  extended only to return conservative stack-local effect progress;
- daemon `execute_exact_ime_tail_replay` for physical uinput and its existing
  cleanup guard;
- atomic effect builder/application;
- active composition executor, terminal native typing and daemon `WordBuffer`.

After extraction, old window orchestration methods are private to the module or
removed. Structural checks reject direct bridge/IBus calls to them. No runtime
branch uses application names.

## Size accounting

Current LOC is measured; move/delete/new values and future sizes are estimates.
The earlier seven-file 9,202-line number included tests and omitted participating
files, so it is not the denominator for production extraction.

| Current file/block | Current LOC | Estimated move-only LOC | Expected remaining role |
|---|---:|---:|---|
| `text_target.rs` production `1–157` | 157 | 157 | file deleted; selector/types move |
| `text_target.rs` tests `159–229` | 71 | 71 test | tests move |
| 32 selected orchestration functions | 1,038 | 700–830 | 208–338 lines of wire decoding, object lookup and thin adapters remain outside |
| `context_runtime.rs` key-callback block `716–815` | 100 | 70–100 | typing logic stays; admission wrapper moves |
| `engine.rs` client facts `269–342` | 74 | 65–74 | unrelated engine logic remains |
| `engine/types.rs` snapshot `428–464` | 37 | 37 | type moves |
| `engine/state_groups.rs` client-state block from `65` | 78-line containing block | 45–70 | retain `client_context` field name and one state owner |
| daemon exact replay | file 422 | 0 | physical leaf stays unchanged |
| `state.rs` local executor | file 1,262 | 0 | leaf stays; stack-local effect-progress delta only |

The 32-function 1,038-line measurement comprises: `bridge_actions.rs` 279,
`ibus_interface.rs` 249, `context_runtime.rs` 383 and `tail_memory.rs` 127. It is
not a move count; wire decoding, object-server acquisition and leaf adapters
remain outside. The reviewer-confirmed lower bound is 784 production lines
before RAII, key-callback and full IBus/bridge closure.

After separating retained adapters, estimated move-only production is
**1,080–1,270 lines**. Estimated deleted duplicate glue/wrappers is **120–210
lines**. Genuinely new production for typed results, central revalidation and
local partial-effect reporting is **220–340 lines**.

Estimated final production by module file:

- `window_interaction/mod.rs`: 200–260 lines;
- `observation.rs`: 430–540 lines;
- `authority.rs`: 300–390 lines;
- `execution.rs`: 370–440 lines;
- total module production: **1,300–1,630 lines**.

Move-only lines cancel in repository net size. Estimated net production change
is new production minus deleted duplicate glue: **+10 to +220 lines**. Existing
tests moved into the module are estimated at 250–400 lines; genuinely new finite
tests are 450–650 lines; final module tests are about 700–1,050 lines and net test
growth is 450–650 lines. These estimates will be replaced with measured
production/test/move/delete counts after implementation.

Current participating file sizes are: `text_target.rs` 229,
`context_runtime.rs` 1,264, `engine.rs` 560, `engine/types.rs` 464,
`engine/state_groups.rs` 142, `ibus_interface.rs` 1,274,
`bridge_actions.rs` 633, `shift.rs` 291, `state.rs` 1,262,
`committed_tail.rs` 1,001, `tail_memory.rs` 2,735 and `atomic.rs` 1,269 lines.

## Measured implementation accounting — 2026-09-12 final acceptance freeze

The accepted pass-2 review freeze contains `928` files in
`/home/ubu/.cache/lay/development/td126-review-freeze-pass2-20260912/source-tests-manifest.tsv`;
its SHA-256 is
`55b91409b82eeb1a5d16151f222816934bcd7212156da34d5ad6a878c7cda395`.
After the mechanically reviewed lint convergence, the final acceptance freeze
contains the same `928` paths in
`/home/ubu/.cache/lay/development/td126-final4-20260912-LHBMzvAX/accepted-source-manifest.tsv`;
its SHA-256 is
`8a9328b744e2845474db010a10138a249cfc1e2902035ee41a79a616360758d5`.
Exactly seven module paths differ; the production bridge, RAII, callback and
leaf-execution bodies remain unchanged.
The detailed LOC receipt is
`/home/ubu/.cache/lay/development/td126-final4-20260912-LHBMzvAX/loc-accounting.json`
(SHA-256
`23dbcbc2df1aac884072ea85814587dbafebc3748540a1087c95630118844ecf`).

Raw row counts, including comments and blank rows, are:

| Current module file | Production-owned | Test-owned | Total |
|---|---:|---:|---:|
| `mod.rs` | 44 | 8 | 52 |
| `authority.rs` | 738 | 97 | 835 |
| `execution.rs` | 193 | 4 | 197 |
| `observation.rs` | 2,048 | 21 | 2,069 |
| `tests.rs` | 0 | 804 | 804 |
| **Module total** | **3,023** | **934** | **3,957** |

`Test-owned` means rows inside a complete `#[cfg(test)]` syntax item plus the
dedicated `tests.rs`. The moved external `context_runtime/tests.rs` is outside
this folder and is excluded from this module-contained split. The result is
larger than the plan estimate because full bridge, legacy/atomic key, lifecycle,
fact and later postcondition closure moved behind the boundary; it does not add
a runtime owner.

The deleted owner files total `2,126` raw rows: `bridge_actions.rs` `633`,
`context_runtime.rs` `1,264`, and `text_target.rs` `229`. Their split is `1,960`
production-owned and `166` test-owned rows. Across the complete affected Rust
delta, the final acceptance freeze is **+519 production rows and +1,006 test rows**
against the frozen baseline. Classification uses syntax-item spans: complete
`#[cfg(test)]` items, all files under `tests/`, and `*_tests.rs`/test directories
are test-owned; complete test-only match arms and field initializers are excluded
from production. The whole IME Rust subtree is `40,594` rows versus `39,095` at
the frozen baseline, a net `+1,499`.

The measured production net exceeds the plan estimate of `+10` to `+220`. The
final code review accepted the larger result as the required full callsite
closure with no second owner. In a temporary two-commit repository built from
the frozen pre-production archive and the final freeze, `git blame -C -C -M`
attributes `2,412` raw module rows exactly to the baseline and `1,545` raw rows
to the current change. The former is a conservative move lower bound. The
latter is an upper bound on new code because split multi-source moves, signature
changes and rustfmt rewrites count as new or rewritten. Production/test ownership
uses the syntax classification above rather than line-origin heuristics.

## Capability support matrix

| Observed context/action | Route | Truthful outcome |
|---|---|---|
| Non-sensitive, unselected exact `SurroundingText`, live admission, local request | `LocalSurroundingDeleteCommit` | Later existing pending snapshot may confirm, remain pending, mismatch or censor; only confirmation is observed client proof |
| Non-terminal committed IME tail selected for physical replay | `DelegateExactImeTail` | Delegation only; daemon `Option<bool>` is layout/dispatch completion, never a client-text receipt; later IME observations may prove the result |
| Explicit terminal purpose, no SurroundingText, existing proven terminal erase facts | `LocalTerminalErase` | Transport executed but client text remains unverified unless an independent existing observation later appears |
| No local committed-text authority, daemon word route live | `DelegateDaemonBuffer` | Typed delegation, zero local effects; daemon keeps its contract |
| Active composition | `ActiveCompositionOwned` | Existing preedit route and receipts continue |
| AtomicV1 | `AtomicOwned` | Existing atomic proposal/application/postcondition continues |
| Sensitive/private/password content | `Reject(SensitiveContent)` | Text-bearing state revoked; no edit or settlement |
| Selection conflicts with destructive edit | `Reject(SelectionPresent)` | No local edit or settlement |
| Stale/missing focus, owner, path, token, Reset receipt or exact snapshot | typed `Reject` | Existing owner consumes or revokes authority |

Cursor geometry remains an input to the already-proven terminal capability
decision. A cursor-location callback alone does not create a new revocation
policy. Logical selection and snapshot rules remain exactly those already used.

## Finite proof plan

The missing API failing to compile is not a semantic RED. Reuse the fresh
accepted unchanged-source baseline. First add behavior-characterization tests
against existing entrypoints; they should pass and freeze today’s routes. The
only deliberate RED before extraction is a controlled structural boundary test
showing that listed IBus/bridge callsites still bypass the absent module; label
it architecture scope, not runtime behavior.

Planned nonzero test identities must be added to the existing manifest and
discovered explicitly:

1. `window_interaction_gui_uses_existing_later_postcondition_receipt`: callback-
   shaped Focus/Capabilities/ContentType/SurroundingText → local execution → a
   later snapshot projects the existing confirmed/pending/mismatch statuses.
2. `window_interaction_exact_tail_is_delegation_without_client_receipt`:
   `DelegateExactImeTail` reaches existing daemon dispatch while the IME reports
   no local or visible success; later native/snapshot observation stays separate.
3. `window_interaction_terminal_execution_stays_client_unverified`: exact
   terminal erase/commit payload, `LocalComplete`, no pending client
   postcondition and a `Rejected` real observer result rather than visible proof.
4. `window_interaction_daemon_buffer_delegates_with_zero_local_effects`.
5. `window_interaction_sensitive_selection_and_stale_authority_refuse`: table of
   exact reasons, surfaces and safety effects.
6. `window_interaction_reset_rereceipt_uses_existing_one_shot_token`: Reset
   invalidation, exact single re-receipt, mismatch/duplicate/second refusal.
7. `window_interaction_bridge_raii_revokes_on_cancel_or_drop`: early return,
   cancellation and dropped future keep `ContextBridgeOutput` incomplete.
8. `window_interaction_commit_failure_after_delete_is_indeterminate`: controlled
   output double accepts delete then fails commit; receipt is partial/error,
   authority is revoked, no settlement or zero-mutation claim is emitted.
9. `window_interaction_priority_routes_keep_existing_owners`: active composition,
   autoUndo, AtomicV1, terminal, exact physical replay and daemon paths retain
   their existing executors.
10. `window_interaction_only_boundary_has_full_callsite_closure`: one controlled
    structural test covers the listed IBus/bridge/read/mutate/cancel/key-callback
    callsites and forbids app-name branches or a second reducer/cache/generation.

Use exact cargo targets already selected by the repository manifest and require
discovery count greater than zero before accepting a result. Run Cargo only
through `scripts/cargo-guard.sh` on the dedicated runner. The fresh accepted
baseline is reused; unchanged full suites are not rerun until the final affected
gate. No physical/live harness belongs to this module-only task.

## Invariants, consequences and risks

- Callback order, zbus sequence comparisons, reducer deadlines and the existing
  500/1500 ms postcondition windows remain unchanged. The extraction adds no
  sleep, retry loop or asynchronous worker.
- Focus-time config reload remains at the same callback boundary. No package,
  config, model or lattice read moves into the module; hot reload behavior and
  material generation remain unchanged.
- Candidate generation/ranking, L1.1–L4, `SafetyGate`, verifier, edit plans,
  feedback semantics and answer quality are unchanged. Existing feedback is
  recorded only after the same observed positive postcondition.
- No added runtime authority: typed values project existing state and expire or
  revoke with it. No alternate text cache, token, generation, timer or client
  identity exists.
- RAII errors and partial delivery are conservative. A failed future cannot call
  `complete`; an emitted delete followed by failure is indeterminate until a
  later existing observation.
- Reject the implementation if old orchestration remains callable around the
  module, or if consolidation becomes a second framework rather than a move.
- Capability support remains fact-based. No application-name branch, all-window
  guarantee or transport-parity quality claim is permitted.
- CPU, RSS, latency and package effects are not measured by the final source
  gates and remain `NOT_TESTED`. The expected added work is enum matching and
  existing scalar checks only; no improvement is claimed.

Rollback is the task-only extraction plus its delegate changes. Do not combine
model/ranking work, daemon policy changes, installation harness or runtime repair.

## Code-review consequence amendment — truthful manual-toggle disposition

The first implementation review found one grouped result-projection defect.
The extracted bridge could collapse a local refusal, a pending/cancelled
autoUndo action, or a partial local failure into the old `Option<bool>` shape and
then infer delegation from the pre-execution authority. The repair now carries
the actual stack-local disposition and effect progress from the existing leaf
through `WindowInteraction` to the bridge. `pending`, `cancelled`, locally
executed, delegated and refused outcomes remain distinct; a local refusal must
never be reprojected as daemon delegation.

This is truthful result projection and preservation of the already required
partial-effect contract. It adds no executor, policy, timer, cache, controller,
retry, generation or persistent state. Existing pending-autoUndo wire ownership,
priority, callback order and no-fallback behavior remain unchanged. The existing
leaf exposes an internal typed-result route while keeping a thin compatibility
projection for callers that still require the old shape. Every fallible await
after output may have begun must preserve its accumulated progress instead of
converting an error through a zero-progress `From<fdo::Error>` path. No production
boxing or stack-budget change is permitted.

The repair proof exercises the real `ManualToggleV3` bridge boundary for a
known-start terminal local refusal, unknown-beginning refusal, pending autoUndo,
and controlled delete-success/commit-failure. It must assert wire disposition,
absence of daemon delegation, and existing token/word-authority revocation after
partial failure. Direct `execute_local` tests alone are insufficient.

## Implementation and review record

The implemented module owns lifecycle/fact observation, context authority,
bridge RAII, local execution or typed delegation, later existing-postcondition
projection, cancellation and settlement/revocation. Wire decoding, object-server
lookup and leaf executors remain thin callers or existing owners. The old
`bridge_actions.rs`, `context_runtime.rs` and `text_target.rs` owners are deleted.
The architecture and dynamic monopoly guards reject their reintroduction and
restrict stack-local effect progress to the existing local executor path.

Code-review pass 1 scored **7/10 `REQUEST_CHANGES`** with two P2 findings under
one mechanism: the old wire projection could infer delegation after a local
refusal, and the compatibility path discarded pending/cancelled/partial effect
progress. The grouped repair added typed `LocalPending`, `LocalCancelled`, local
refusal and partial-failure propagation through the existing leaf, module and
bridge, kept RAII revocation, and added real bridge-boundary regressions. Code-
review pass 2 scored **8/10 `ACCEPT`**, High 0 / Medium 0, with no material
finding. The two-pass implementation-review limit is complete.

The causal focused RED is local
`/home/ubu/.cache/lay/development/run-psg970b7`, remote
`/home/e/projects/lay-development-runner/run-UPGeq9`, source archive SHA-256
`90585107b5ee4d54425c0bfe2164b0c860646d9b047d10ac6b1f3c7f65bd765f`.
It executed `488` cases: `487` passed and the intended real
`residual_known_numeric_word_bridge_refuses_without_delegation_or_output` failed
because local refusal was projected as V3 exact delegation. Its `SUMMARY.json`
SHA-256 is
`c5ca789616d161de319a570e13e8754577840c21a50e7cfc19db7e54737a9b2f`.

The final focused GREEN is local
`/home/ubu/.cache/lay/development/run-78cby16t`, remote
`/home/e/projects/lay-development-runner/run-meUGIM`, source archive SHA-256
`0004e1a73f1da4982ab57e13916b554feb65f2369a1c5141a5a759a05ccef8e6`.
It executed **506/506 PASS**: the IME target contributed `490` selected cases
(`488` in the target process plus `2` process-isolated) and the text-mutation
monopoly target contributed `16/16`. Three performance cases were discovered
and excluded. `SUMMARY.json` SHA-256 is
`43b869ee7cf0be9c06d72264764afae33d3787319acb7c2af60bb93e2ba705a2`.
The earlier `run-tiz7h9y3` `505/506` result is historical: its single failure was
a stale structural test marker after the typed helper split, while its IME target
was already `490/490`.

The canonical test manifest SHA-256 is
`f80243ff9420b7e7b1cd8476c7786ca584579c04acde8946a82d373bacf4a578`:
correctness `2,745`, package `36`, performance `11`, ignored `15`, inventory
`2,807`. The known-failure ledger SHA-256 is
`86742bdd62b301722089ced6df5193d8769fda26d27489b46e4c54a4d7fa7fbc`;
its exception count is zero. Focused GREEN proves route repair and structural
closure for its two targets. It does not replace the final affected denominator.

## Architecture-gate owner-map correction

The first final snapshot was local
`/home/ubu/.cache/lay/development/td126-final-20260912-TtqvKNCD`, remote
`/home/e/projects/lay-development-runner/td126-final-TtqvKNCD`, with source
archive SHA-256
`108316b2eccdfc0e06697ce34835942d85ff816f08c74b05f54dc4da064b67dd`
and request SHA-256
`2146455ae778ee23d4999a41f410c0e558ba702c61b0057e7c1a51e83eba63e3`.
Its single remote lease stopped at canonical architecture refresh before Cargo
tests or lints. The new graph correctly located
`arm_visible_postcondition_with_feedback`, `observe_visible_postcondition` and
`record_observed_system_outcome` in `window_interaction/observation.rs`, while
`architecture_graph_gate.py` still required their deleted orchestration owner
`tail_memory.rs`. That stale owner map produced the three WATCH entries; the
other ten architecture checks passed.

The correction changes only the architecture gate's expected owner and source
read. It still requires mismatch quarantine from the existing low-level
`tail_memory.rs` owner and rejects semantic anti-feedback in either participating
source. Accepted Rust source, tests, manifests and runtime authority are
unchanged. Failed `RESULT.json` SHA-256 is
`e3b7065a5a078974951a915c21d2d22488f459103515684955643f8cab1da7ac`;
the architecture log SHA-256 is
`2d7175d270c7c626f3191f62cda44e77b2acd4d89dcaa252048be94ff5702765`.
The shared Cargo target stayed at `8,501,657,600` of `12,884,901,888` bytes
before and after. This failed run is historical and is not retried in place.

The next fresh snapshot was local
`/home/ubu/.cache/lay/development/td126-final2-20260912-tnERB58x`, remote
`/home/e/projects/lay-development-runner/td126-final2-tnERB58x`, with archive
SHA-256 `b5bdfa1c015ee3506d4a9d8dc892725f0a125471b036ce62f9323ae41788b86d`
and request SHA-256
`eba280f0a2beb9778bdcad397ad284f7175e061625423ef012d544d32d840973`.
Its architecture refresh passed. The changed gate then passed the lane self-
tests, manifest, format, daemon `260/260`, IME `490/490`, library `1,792/1,792`
and every earlier executed target, but stopped at
`typing_transition_authority_contract` `20/21`. The unchanged structural test
searched for the old helper call `undo_last_ime_autocorrect(emitter)` after the
accepted typed-progress repair had renamed that exact call to
`undo_last_ime_autocorrect_with_effect_progress(emitter)`. The focused GREEN did
not include this integration target; the full gate correctly exposed its stale
source anchor.

The repair changes that one searched helper string. Test identity, protected
undo-before-manual ordering assertion, manifest, production and LOC are
unchanged. Run `RESULT.json` SHA-256 is
`c59dd799b4fa2d0d47d25b965178dee3f7f19e49083c14784ec715e8894a6bdc`;
architecture log SHA-256 is
`45852a62dcec5cb857328c4fd38535b991ea95a6a0e4d8be77d076b9eaae9509`;
changed log SHA-256 is
`a2b665f20d167705fe8db285eebc818c87cefb11f89aa675414d2123ab7c06e1`;
the blocked summary SHA-256 is
`2d5e3be7ee4bb7824cc3bbbfb811d6b27adf954a59056192e93fd5a337ae83d9`.
It supplies no aggregate final PASS denominator and is not retried in place.

The third fresh snapshot was local
`/home/ubu/.cache/lay/development/td126-final3-20260912-klRwgwwe`, remote
`/home/e/projects/lay-development-runner/td126-final3-klRwgwwe`, with archive
SHA-256 `bf3e10daf74f06612f6a76b983192a2f47e6d69f00b97dbaa34d2d14ce1de915`
and request SHA-256
`8045ef91e464087f25758e3efe0d2c9081d2d8c59c3c6083fb874cf05173b398`.
Its architecture refresh passed. Its canonical lanes also returned PASS with
`2,781` selected cases, `2,745` correctness and `36` package, and zero known
semantic or infrastructure failures. The canonical summary does not expose
executed/passed/failed fields, so those absent fields are not rewritten as
zero. The complete target/process logs contain no test failure.

The outer changed gate then stopped at its lint stage. Module extraction had
left six test-only names as unconditional re-exports. Removing those warnings
exposed four ordinary Clippy cleanups in the same changed slice and dead-code
inventory for unused placeholders and test-only readouts. The grouped repair
gates test-only exports/readouts with `cfg(test)`, uses the production observer
in the two client-receipt assertions, removes the unused generic cancellation,
settlement and delegation placeholders, keeps actual bridge cancellation and
typed execution receipts, and applies only Clippy-equivalent expression/name
cleanup. The default/research dead-code baselines shrink from `535/359` to
`534/358`; no warning is added to either baseline. Their new SHA-256 values are
`6dcd15f7927df93540c77933f7787a30b249e31c482040ed77549d9db44dcfd5`
and `7899073e7545eaf241d0c6a1241c25daebfbdefbd36d6bd9f552fa1522047fcc`.
The local full two-scope lint contract then passed.

The failed run `RESULT.json` SHA-256 is
`4d8e6250f17027adcfd7c408b587037632d1aec6cc755282466b38e636b903b2`;
architecture log SHA-256 is
`364809e9d4a9c455551eae155bab37b20edd74e4a46ce3848c0d7d02baf4f17f`;
changed log SHA-256 is
`171064eb8f23a086e36536c3fcaaf27ff5979b8746766db613b7e2a18560124d`;
canonical summary SHA-256 is
`fdb6e53eeb90efe033b64940d4346e4653789bd9e9aeb14c14859b06ea61227f`.
The Cargo target moved from `8,501,657,600` to `8,501,669,888` bytes, below the
`12,884,901,888`-byte budget. The run is historical and is not retried in place.

## Final remote gate contract and receipts

The final source and this owning document are frozen before graph refresh. One
remote outer lease at `e@192.168.3.94` uses profile `dedicated-20cpu`, Cargo jobs
`20`, Rust test threads `1`, and shared target
`/home/e/projects/lay-td119-gate-v1/target`. Within that single lease the exact
sequence is:

1. `scripts/cargo-guard.sh --status`;
2. canonical `scripts/update-architecture-graph.sh`, which writes the graph
   binding and receipt and runs the architecture gate;
3. `scripts/check-lay-lints.sh`, both default and research scopes, fail-fast
   before the expensive canonical lanes;
4. `LAY_CHANGED_CLIPPY=0 scripts/check-lay-changed.sh`, including canonical
   correctness/package lanes, lane self-test/manifest, format/check and
   historical transition/unsafe gates;
5. explicit `scripts/check-architecture.sh`;
6. `scripts/cargo-guard.sh test --locked --lib
   architecture_contract::tests::generated_receipt_proves_every_required_architecture_check
   -- --exact` for the compiled receipt projection;
7. source/graph/receipt identity comparison and final
   `scripts/cargo-guard.sh --status`, requiring the target to stay at or below
   `12 GiB`.

The immutable final input/output root is
`/home/ubu/.cache/lay/development/td126-final4-20260912-LHBMzvAX/`; the remote
root is `/home/e/projects/lay-development-runner/td126-final4-LHBMzvAX/`. Logs are
named `00-budget-before.log`, `10-architecture-update.log`, `15-lints.log`,
`20-changed.log`, `30-architecture-check.log`, `40-compiled-receipt.log`,
`50-identities.json` and `60-budget-after.log`; canonical test output is under
`20-results/`. Machine-
readable completion is `RESULT.json`. The project receipt is
`docs/structural_gates/receipts/LAY_TD126_COMMON_WINDOW_2026-09-12/final.json`.
That receipt owns the final PASS/FAIL state and separate route, execution,
later-observation, architecture, lint, source-identity and runtime-authority
fields. The receipt directory is excluded from Graphify input, so filling those
predeclared fields after the run does not stale the architecture graph.

## Completion checklist

1. [x] Discuss the repaired plan with the user before production edits.
2. [x] Complete plan reviews 6/10 `REQUEST_CHANGES` then 8/10 `ACCEPT`.
3. [x] Extract the real module, close every listed callsite, preserve existing
   owners and add typed result/effect progress without persistent state.
4. [x] Prove the review repair with the causal RED and final focused GREEN.
5. [x] Complete code reviews 7/10 `REQUEST_CHANGES` then 8/10 `ACCEPT`.
6. [x] Accept the authoritative one-lease affected, lint, architecture and
   compiled-receipt result from the predeclared project receipt above:
   `2,781/2,781 PASS` with accepted source/graph identity.
7. [x] Keep installation, activation, runtime/service/input-source changes and,
   under the authorization active during implementation, commit and push out of
   the source-only acceptance run.
8. [x] Mark TD-126 `DONE` for source-only scope and record the exact limits and
   receipts in `evidence/td126-final-acceptance.md`.
9. Git checkpoint and publication state, including the exact commit, verified
   remote ref and clean-worktree result, are owned by the external
   `/home/ubu/.cache/lay/development/td126-publication-20260913/publication.json`.
   The checkpoint scope is the previously accepted uncommitted
   IME/context/native prerequisite base plus the TD-126 extraction.

Performance/ignored lanes, installation, physical keyboard behavior, broad
application support and answer quality are not part of this source-only proof.
Routing, local execution, later client observation and verifier coverage remain
separate claims. Runtime authority and the installed runtime remain unchanged.
