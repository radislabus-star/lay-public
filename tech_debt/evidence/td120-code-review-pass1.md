# TD-120 final implementation code review — pass 1

Verdict: **REPAIR_REQUIRED**. Score: **6/10**. Findings: **High 1 / Medium 2 / Low 0**.

Reviewed on 2026-09-05 against baseline/HEAD
`cc1e2207519801ca0f9b7c6963897b55953a7751`. This is an independent source review
of the current TD-120 production diff and tests, not the earlier specification
review, a release gate, or installation approval. Runtime authority changed by
this reviewer: **false**. Only this report was written.

## Findings

### H1 — Atomic owner admission and the captured base can refer to different owners

Location: `src/bin/lay_ibus_engine/atomic.rs:91`, `:147–152`, `:249–254`.

`atomic_owner_is_live()` takes and releases the shared lock. The later
`deep_atomic_clone_with_base()` takes another lock and accepts its snapshot
without requiring that snapshot's active path to be this engine's path.
Settlement checks that the live path still equals the captured path; it never
checks that the captured path belonged to the engine producing the proposal.

Reachable interleaving using the already admitted separate-engine callback
locks:

1. A's atomic request passes the owner check while A is active.
2. B's `FocusIn`/`bind_focus_path` claims shared ownership before A captures its
   shared clone. A's pending transition has not yet been inserted, and B's
   callback cannot discard A's pending entry in any case.
3. A clones its own cached engine state together with B's shared snapshot.
   Its base consequently records B's path/tail and A's local owner lease.
4. A produces a managed Space/printable frame from its old tail. The speculative
   `publish_tail_handoff()` writes A's tail into the isolated shared copy.
5. If B's live stamp then remains unchanged and a compatible submitted receipt
   arrives, all settlement comparisons pass. The assignment at `atomic.rs:306`
   replaces B's shared tail with A's speculative tail. The owner test for the
   *next* key happens after this assignment and cannot repair it.

An IME layout handoff need not change the adapter's daemon focus epoch, so the
existing receipt epoch check does not close this interval. This is a failure
of the selected bounded atomic admission guarantee, not a demand for TD-121's
canonical context protocol or a general SharedState transaction framework.

Minimal fix: validate active-path ownership under the same lock used to capture
the shared snapshot, return refusal without preparing a mutating clone when it
does not belong to A, and require the base owner to be A at settlement. The
existing base/live comparison then detects a handoff occurring after capture.

Verification: a deterministic callback interleaving at admission/capture using
the real `process_atomic_key_event` and B `FocusIn`, not a manually invented
pending record. Assert no frame from a foreign-owner snapshot, unchanged B
tail/guard/owner, no deferred effects, and no stale assignment on a receipt.
Retain the existing after-proposal foreign-owner and actual A→B→A cases.

### M1 — Open-word length still counts text before hard punctuation

Location: `src/bin/lay_ibus_engine/tail_memory.rs:841–847`; consumption lifetime
at `:793–817`; successful producer at `state.rs:630–673`.

`current_open_token_chars()` checks whether the fast token is nonempty, then
counts `last_tail_token_range()`. That range is delimited only by whitespace
(`tail_memory.rs:1207` onward), whereas `push_tail_char` treats `!` and other
hard punctuation as real boundaries. The count is therefore not the lifetime
of the lexical token protected by the new scope.

Concrete route: start with `abc!q` and successfully execute the existing typed
`daemon_bridge(1, "x", true)` replacement, producing `abc!x`. As in the existing
authorized partial-replacement test, this route emits delete/commit and then
arms the guard. The fast token is `x`, but the guard records 5 characters.
Delete that one open-token character: the tail becomes `abc!`, the fast token
is empty, but Backspace decrements 5 to 4 and retains the guard. Typing `y`
refreshes the same incarnation to 5. The next managed Space is incorrectly
suppressed for the new word. This reproduces the ordinary lifetime defect
without V1, context uncertainty, or transport replay.

Minimal fix: derive or maintain the bounded open-token length with the same
boundary/layout-symbol semantics as the existing lexical decision. Keep
incarnation continuity for partial edits and tail trimming; do not merely use
the fast token's length, since it has a separate 32-character display limit.
Do not add literal examples or unconditional punctuation splitting to runtime.

Verification: extend the real successful-producer/full-deletion cases with
left context separated by hard punctuation and no whitespace. Assert recorded
first delete/commit, open-token count, guard absence immediately after full
deletion, no inherited guard on the next word, and unchanged left context.
Retain the `;`, `[`, `]`, apostrophe layout-letter controls and the 160-character
bounded-tail test. Existing `abc`→`abc!` retirement only tests a boundary typed
*after* arming, so it does not cover this failure.

### M2 — Mandatory transport/atomic/feedback proofs are not implemented by the named tests

Locations: `src/bin/lay_ibus_engine/atomic.rs:829–899`, `:903–1015`;
`td120_word_lifecycle_tests.rs:631–644`. Requirements are the selected admission
analysis matrix at `tech_debt/evidence/td120-suppression-admission-analysis.md:304–322`.

The A2 test manually installs `PendingAtomicTransition` and writes
`speculative.composition.buffer = "accepted-speculation"`. It calls the scope
helpers directly, never prepares/emits that frame, never exercises V1/V2 bridge
admission, and despite its name has no live-consume case or prepare→V2-arm
case. The equal-final-revision test is a useful pure settlement check but also
inserts a pending entry manually. A3 sensitive/cancellation does the same;
its empty deferred-vector assertions begin with no deferred effects to lose
or duplicate. No new test counts accepted/reverted/censored feedback events.

The sole new V1 test directly arms the helper, pushes Space, and consumes it.
It does not characterize the required caller/reselection routes, before-only
abort/failure, duplicate calls, delayed after another owner/word, or the latent
ReplaceText branch and current ReplayAll caller. The production bridge's
new active-owner refusal is not exercised by that helper-only case either.
These are omitted **mandatory TD-120 compatibility proofs**, distinct from the
deliberately accepted V1 information deficit. Repairing the protocol itself
remains outside TD-120.

Minimal fix: finish the bounded matrix using existing real proposal and bridge
methods and isolated output/feedback capture. For A2/A3, prove the first frame
and its actual effects before injecting the admitted callback; check live
scope including `None`, local mirror, base revision, retained handoff fields,
duplicate receipt, next-key refusal, and zero second output/feedback. Add the
specified V1 compatibility ledger, keeping delayed/before-only leakage explicitly
`KNOWN_RESIDUAL`. A compact parameterized table is sufficient; a new production
transaction or replay manager is not required. Include the currently absent
active-composition Enter and terminal-passthrough ordinary boundary cases in
that completion, with meaningful boundary feedback assertions.

Verification: report independent denominators for ordinary lifetime, V1
compatibility/residuals, exact replay, atomic normal/guard conflict/tail conflict,
and feedback. Passing the present test-name filter cannot stand in for the
missing matrix rows. Final execution remains the parent's guarded remote task.

Smallest useful test seams:

- Register the real engines on an isolated in-process/p2p zbus connection,
  construct `LayImeBridge` with that connection/shared state, and directly call
  its existing `suppress_next_autocorrect_inner`, V2 arm, and cancellation inner
  methods between real proposals and receipts. These methods execute the real
  active-path lookup, object-server engine lock and admission code. This proves
  **bridge-method admission**, without starting a global IBus service. An
  optional proxy call over the same isolated connection proves wire dispatch;
  do not call a direct-inner-method result actual D-Bus transport coverage.
- Drive the V1 receiver schedules (before-only, duplicate, replayed boundaries,
  delayed after new word/owner) through those real bridge methods and record
  them as receiver compatibility. For the daemon caller/reselection and latent
  text-branch counts, use a narrow injection point for the existing external
  native/preflight/uinput/suppression effects and run the current branch logic.
  A test-local event list is enough; it must not be a separately reimplemented
  replay decision tree. Pure receiver tests plus a source caller ledger must
  not be described as executed daemon branch coverage.
- A cfg(test) admission interleaving callback/barrier is enough for H1; it need
  not become a production coordinator. Existing output builders and a scoped
  feedback sink can count real effects, including a nonempty speculative
  feedback action that the conflict must censor.

## Positive conclusions and limits

- The old boolean/optional-exact state has been replaced by typed scopes.
  Ordinary shared consumption requires the matching local incarnation and
  owner lease; a spent ordinary local mirror is not OR-ed back into authority.
  Existing admitted handoff transfers the incarnation and rebinds the lease.
- Arming was moved after actual committed output/tail synchronization. Rejected,
  no-op and duplicate return paths do not newly arm. The duplicate-open fixture
  now asserts a real first delete/commit before checking the no-effect duplicate.
  Active candidate/manual producers arm inside the committing implementation.
- Ordinary retirement does not take/drop transport variants merely because the
  tail becomes temporarily empty. Exact suffix/path/epoch/layout/deadline/revoke
  checks remain in place. No new physical Double Shift detector, literal-word
  runtime rule, ranking exception, or weakened verifier was found in the diff.
- The settlement replacement itself is performed under the lock that checks
  base/live stamps. Guard-only drift preserves the newer suppression block and
  uses the base revision, including equal final numeric revisions. Owner/tail
  conflict drops the pending entry and returns refusal without applying the
  speculative deferred actions. H1 concerns the preceding admission snapshot.
- O6 invokes the actual manual producer, native Backspace bookkeeping, managed
  printable and Space methods. It checks concrete delete/commit payloads and
  mirrored final surfaces for three safety profiles and separate protected
  controls. Its exact lease helper calls production `prepare_inline_exact`.
  This is valid deterministic closed exact-layout route evidence when executed,
  not asynchronous full Nanda winner/heldout quality or a live client smoke.
  The proof plan explicitly makes those limits clear.
- CurrentWord owner/tail lifecycle does not certify canonical handoff identity;
  that remains TD-121. V1's unbound before/after request semantics remain TD-122.
  No new release/install/latency/RSS/heldout quality result is inferred here.

## Exact read scope and verification

Instructions read: this worktree's `AGENTS.md`; installed graphify `SKILL.md`
and `references/query.md`. Existing Graphify query located atomic/replay entry
points; direct source was required for control-flow conclusions. No graph
rebuild/feedback write was performed because this task permits only this report.
The explicitly forbidden structural-gate skill was not used.

Task documents read completely: TD-120 owning task, suppression admission
analysis, runtime implementation evidence, and O6 full-frame proof plan.

All baseline-relative Rust diff hunks read in these 16 tracked files:

`src/bin/lay_ibus_engine.rs`; under `src/bin/lay_ibus_engine/`:
`atomic.rs`, `bridge_actions.rs`, `committed_tail.rs`, `composition_commit.rs`,
`composition_edit.rs`, `engine.rs`, `engine/state_groups.rs`, `ibus_interface.rs`,
`managed.rs`, `preedit.rs`, `protocol.rs`, `protocol/state.rs`, `shift.rs`,
`state.rs`, `tail_memory.rs`.

New files read completely: `td120_word_lifecycle_tests.rs` (715 lines, 21 tests)
and `td120_full_frame_tests.rs` (337 lines, 2 tests). Also read all five new
atomic tests and the new interface callback test: **29 new TD-120 test functions
inspected, zero executed by this reviewer**. Existing tests changed in the
above diffs were inspected, not deleted from the review denominator.

Supporting current-source sections inspected: producer authorization/output
and tail synchronization; suppression helpers and whitespace token range;
fast-token boundary/layout-symbol logic; managed Enter/Space; FocusIn/Out,
Reset, Disable, surrounding callbacks; atomic preparation/settlement/deferred
effects; bridge admission/cancellation; exact-lease generation/installation in
`space_autocorrect_prefetch/proof.rs`. Bounded symbol searches of daemon
`correction_runtime.rs` and `correction_runtime/output{.rs,/native.rs,/replay.rs,
/text_replace.rs}` checked where the required V1 route tests could live; those
unchanged daemon implementations were not a full independent daemon review.

`git diff --check`: PASS. No Cargo, build, service, config, input injection,
commit, push, installed-runtime operation or subordinate agent was run.
Parent-reported intermediate test results were read only as dated evidence;
they are not claimed as verification of this final reviewed source.

After this report was drafted, the parent reported the final frozen remote
`lay-ibus-engine` bin run: **310/310 PASS, 0 ignored, 14.43 s**, compilation
2.49 s; architecture refresh PASS with existing budget warnings. The reviewer
did not independently read the remote receipt. These results support the
executed existing denominator and do not resolve H1/M1 or supply M2's absent
cases.

Source fingerprints at review (SHA-256):

- Tracked Rust diff listed above: `7e4b52463e32324ed45770b2c3dfed74ae4046449fc3a15f2b0e30eb8f80b9f0`.
- `atomic.rs`: `26d5225a055d76c7c8fb3a99154c730730d729d911fcfae9db2ba3e4122b098a`.
- `tail_memory.rs`: `b3e6d14fbb242264ccb1af26b882010a77a1785c42893c412783768902055070`.
- `td120_word_lifecycle_tests.rs`: `9cf29a053687656f257da3f225be9abebe5536644ece197e2f82b047c2b28e70`.
- `td120_full_frame_tests.rs`: `6386ee908edb6cad1778e30855acd2dd71a1ea0834bf3471f993d6d879abed93`.

Recommended next step: one bounded repair pass for H1/M1 and completion of M2,
then exact-source guarded remote verification and fresh review of the repair.
Do not mark TD-120 DONE or admit release 1.0.66 on this pass.
