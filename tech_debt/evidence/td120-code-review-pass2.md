# TD-120 independent code review — pass 2

Date: 2026-09-06. Baseline: `cc1e2207519801ca0f9b7c6963897b55953a7751`.
Worktree: `/home/ubu/projects/lay-tech-debt-20260831`.

## Findings and verdict

**SOURCE_REVIEW_PASS — 8/10; High 0 / Medium 0.** No remaining actionable
High/Medium source finding was identified in the frozen TD-120 change and its
bounded repair delta. The pass-1 H1/M1 defects and M2 implementation-of-proof
omissions are addressed at source level.

**Final execution gate: PENDING.** The final test-only fixture correction has
been inspected and fingerprinted below; no canonical test receipt for these
exact final bytes was available when this report was written. This verdict is
an independent code-review result, not final TD-120 acceptance, release 1.0.66,
installation, canonical TD-121 handoff, or universal V1 correctness approval.
Runtime authority changed by this reviewer: **false**. Only this report was
written by the reviewer.

## Accepted repairs and supporting source evidence

- **H1, foreign-owner atomic capture:**
  `src/bin/lay_ibus_engine/atomic.rs:156` checks the active path while holding
  the same shared lock that supplies the cloned state and base snapshot.
  Settlement additionally requires the captured owner to equal this engine
  before comparing the live owner/tail/lease stamp and replacing shared state
  under that lock (`atomic.rs:254`). The deterministic FocusIn interleaving
  at `ibus_interface.rs:426` drives the actual proposal entry point and checks
  refusal plus the retained foreign owner/tail/guard. The existing real
  post-proposal foreign-owner and A→B→A callback tests remain.
- **M1, lexical word length:** `preedit.rs:239` now uses the same boundary
  predicate and 32-character lexical classification window as the fast token,
  while keeping an uncapped count over the bounded tail. Its counter uses
  scalar state, avoiding a second string buffer. `tail_memory.rs` uses this
  count for ordinary suppression; full deletion retires the matching guard.
  The real successful partial bridge replacement after hard punctuation
  verifies one open character, retirement after its deletion, preserved left
  context and an unprotected fresh word. The layout-letter punctuation,
  160-character trimming and long mixed-prefix classification controls remain.
- **M2, atomic/bridge evidence:**
  `td120_bridge_atomic_tests.rs` registers real engine objects on isolated p2p
  zbus connections. V1 arm/live consumption and V2 arm/revocation execute the
  production bridge inner methods between real atomic proposals and receipts.
  Real ContentType invalidation (`atomic.rs:1007`) and bridge handoff
  cancellation (`td120_bridge_atomic_tests.rs:239`) now start with an actual
  frame and a nonempty deferred learning action; settlement censors it and a
  repeated receipt produces no second frame/feedback. Pure equal-revision
  settlement tests are retained as corner-case evidence alongside these actual
  entry-point tests, not presented as transport execution.
- **M2, daemon branches:** the production-used `try_ime_replace_output_with`,
  `select_native_output_stage_with` and `run_after_native_output` carry the
  existing authorization, dispatch/reselection and native→uinput decisions.
  The tests feed actual typed dispatch receipts through these functions:
  dispatched, explicitly reselectable, rejected and indeterminate outcomes
  have distinct native/layout/GNOME/V1 counts. The latent ReplaceText test at
  `correction_runtime/output/text_replace.rs:339` now executes the real
  decoder-action, plan and authorization body, with injection only at external
  execution/layout effects. Success and failure both demonstrate before-only
  V1 and zero replay. The current caller's existing ReplayAll decision test
  remains the caller contract.
- **M2, V1/V2 selection and compatibility:**
  `layout_controller.rs:385` and its internal typed dispatch helper preserve
  the backend gate and exact suffix/epoch/path/layout arguments. Tests count
  the selected request kind; the zero-V1 check is no longer an unrelated
  unused counter. V1 receiver tests retain duplicate-call revision checks,
  place tail deletion/replay between before/after calls, and make actual late
  bridge calls after a new word and a new engine exist. Those late calls are
  explicitly **KNOWN_RESIDUAL**, not ordinary-lifetime successes.
- **M2, boundary feedback:** the test at
  `td120_word_lifecycle_tests.rs:405` performs successful candidate acceptance,
  real Backspace and real boundary handling with pending completion learning
  and an armed CurrentWord. Its sink observes the existing accepted-completion
  publication with the expected context/word; the guard retires, and another
  real boundary publishes nothing. The final fixture-only rebuild before
  arming is necessary because acceptance with a trailing boundary clears the
  fast token. It does not change production lifecycle behavior.

The daemon seams retain the default production effects and public protocols;
they introduce no replay identity, retry, service, transaction manager or
additional physical Double Shift detector. No literal-word runtime condition,
candidate/ranking exception, weakened SafetyGate or verifier was found.

## Proof denominators and execution status

The final source contains **40 new TD-120 IME test functions**: 26 word-lifecycle
file tests, 2 full-frame tests, 6 bridge/atomic file tests, 4 atomic tests and
2 interface callback tests. It also contains **10 TD-120 daemon test functions**.
These are source-inspected function counts, not 50 independently executed
acceptance cases; functions contain multiple schedules and some are controls.

Keep these evidence categories separate when publishing the final gate:
ordinary producer/lifetime; V1 caller and receiver compatibility; V1 known
residuals; exact replay admission; atomic normal settlement; guard drift;
owner/tail conflict; feedback publication/censorship; full-frame correction.

The O6 tests retain three safety profiles, former-word and clean apply cases,
and protected controls. They use the production closed exact-layout lease
preparation and managed output route. Their explicit expected denominator is
6 correction frames and 3 protected no-correction frames, with preserved left
context, plus 3 separate fresh-guard-absence checks. This is deterministic
closed exact-layout evidence, not full asynchronous Nanda/heldout quality or a
live client/keyboard smoke.

Receipts independently read over read-only SSH:

| Receipt | Observed result | Scope |
|---|---|---|
| `/home/e/projects/lay-td120-repair-v3-ime.log` | 319/319 PASS, 0 ignored, 14.46 s | Earlier repair checkpoint, before final M2/fixture delta |
| `/home/e/projects/lay-td120-repair-v3-daemon.log` | 234 PASS / 2 FAIL, 0 ignored, 142.11 s | Earlier raw host-environment run; not a final PASS |
| `/home/e/projects/td120-neighbor-hermetic-2lbs0i_u/test.log` | 1/1 PASS, 2.20 s | Isolated neighboring typo test in the canonical hermetic harness |

The daemon source-ledger failure was the obsolete search for
`if let Err(e) = replay_result`; its removal was inspected, with actual effect
tests retained. The other failure was the unchanged neighboring typo case
expecting `кнопку ` from `кнорку`. The parent reports the hermetic 1/1 used the
same executable; this reviewer verified the log result, not that executable's
identity. No typo implementation or test expectation was weakened here.

Final canonical compilation/tests, source-manifest parity, architecture/changed
gates and any release/install smoke are still **PENDING** in this report.
No latency, RSS, package-size or heldout quality measurement is inferred from
the source review or earlier test timings.

## Exact read scope and fingerprints

Read the worktree `AGENTS.md`, graphify skill and query reference; used one
existing-graph query for navigation, then direct source for control flow.
No graph rebuild, reflection or saved-query write was performed. The forbidden
structural-gate skill was not used. Read the owning TD-120 task, selected
suppression admission analysis, pass-1 review and pass-2 consequence addenda.

Read every baseline-relative hunk in these 25 tracked files: `Cargo.toml`,
`Cargo.lock`, `src/bin/lay_ibus_engine.rs`; under `src/bin/lay_ibus_engine/`:
`atomic.rs`, `bridge_actions.rs`, `committed_tail.rs`, `composition_commit.rs`,
`composition_edit.rs`, `engine.rs`, `engine/state_groups.rs`, `ibus_interface.rs`,
`managed.rs`, `preedit.rs`, `protocol.rs`, `protocol/state.rs`, `shift.rs`,
`state.rs`, `tail_memory.rs`; under `src/bin/lay_daemon/`:
`correction_runtime.rs`, `correction_runtime/output.rs`,
`correction_runtime/output/native.rs`, `correction_runtime/output/native_stage.rs`,
`correction_runtime/output/replay.rs`, `correction_runtime/output/text_replace.rs`,
and `layout_controller.rs`.

Read the three new IME test files completely, then only their bounded final
changes. Supporting unchanged producer, lexical, callback and exact-route
sections were inspected where needed. TD-121 staged/cache artifacts and the
release controller were excluded. No full-review restart was performed after
the initial review; subsequent reads were the agreed repair delta.

`git diff --check`: PASS on final inspected source. Reviewer ran no Cargo,
services, input injection, Git writes, configuration or installed-runtime action.

SHA-256 anchors for the final reviewed bytes:

```text
25-file baseline-relative diff  84d4779df36ab16b4add91387304997a81f4e2721215aaa5ed51b7537e70874f
td120_word_lifecycle_tests.rs   3042814e5b3f5fdb8f4cf99ae73230f8029ea2b97b6bcc472e8c177c21608e90
td120_bridge_atomic_tests.rs    ad851073e7e315ba673a51b0b1c156f52889e42f3d1ed70bbe29eb1296dc2606
td120_full_frame_tests.rs       6386ee908edb6cad1778e30855acd2dd71a1ea0834bf3471f993d6d879abed93
```

The diff anchor is `git diff` against the stated baseline restricted to the
25 tracked paths above; untracked tests are anchored separately. The safe next
step is the parent's already authorized canonical remote execution of this
source, followed by recording its exact receipt in the owning task. Do not
promote this source-review PASS into a test, TD-121/TD-122 or release PASS.

## Mechanical continuation of pass 2 — final source binding

Date: 2026-09-06. This is a bounded validation of the post-review mechanical
delta, not a third review. **SOURCE_REVIEW_PASS remains 8/10, High 0 / Medium 0;
final execution remains PENDING.** The hashes below supersede the earlier
source binding where the files changed.

Canonical compilation exposed an invalid `.await.expect(...)` on the
synchronous boolean `cancel_exact_manual_toggle_handoff_v2_inner` in the bridge
test. The test now asserts the callback's boolean result directly. Inserting
only the removed await/expect text back into the current file reproduces its
previous reviewed SHA-256 `ad851073...` exactly; no other test behavior changed.
The earlier source verdict did not certify compilation, and this correction
does not itself constitute a successful test run.

The architecture scanner's unchanged prefix patterns at
`scripts/check-architecture.sh:676` and `:677` also matched the former private
helper names. The private helpers and their references were renamed:

- `try_ime_replace_output_with` → `execute_native_output_with_effects`
  in `correction_runtime/output/native.rs`: 4 exact substitutions.
- `try_manual_text_replacement_with` →
  `execute_manual_text_replacement_with_effects` in
  `correction_runtime/output/text_replace.rs`: 3 exact substitutions.

Programmatic comparison against the in-memory source snapshots from the
completed review confirmed exact rename-only equality, including unchanged
whitespace, signatures, bodies and default effects. Public owner function names
and the architecture scanner were not changed. The lifecycle fixture remains
at its previously accepted hash. `git diff --check` remains PASS.

```text
25-file baseline-relative diff  af3dd64556afe9886be615012f90aeb88e912f14e9dbe89c7deb8b0d573f7249
output/native.rs               6ab4b8af13737b7b3c8daaf0587484a6c57fbb8b30ce38b647d88741b19be23d
output/text_replace.rs         287660efa768db76f022fa8e7f1ca73035958e46059757e0d17e011e985598c0
td120_bridge_atomic_tests.rs    d2182fa1248045c7f3c23b19aec39f40e8b6b9ac2d0884c324467f9dd17cc02c
td120_word_lifecycle_tests.rs   3042814e5b3f5fdb8f4cf99ae73230f8029ea2b97b6bcc472e8c177c21608e90
td120_full_frame_tests.rs       6386ee908edb6cad1778e30855acd2dd71a1ea0834bf3471f993d6d879abed93
```

The canonical remote sequence is owned by the implementation agent. Its final
successful receipt was not yet available for this append; no execution,
architecture, task-completion or release PASS is added here. The reviewer ran
no Cargo and made no runtime changes during this continuation.

## Closing supplement — source contracts and canonical execution

Date: 2026-09-06. This bounded supplement accepts the final source-contract
delta and records the completed canonical changed gate. It supersedes the
earlier **PENDING** statements for that execution checkpoint only.
**TD-120 scoped source-and-execution checkpoint: PASS. SOURCE_REVIEW_PASS
remains 8/10, High 0 / Medium 0.** No new finding was identified.

Read the complete baseline-relative changes in
`tests/td113_hybrid_source_contract.rs` and
`tests/text_mutation_monopoly_contract.rs`, the complete new
`tech_debt/evidence/td120-composition-mutation-successor.json`, and the final
source-contract repair section of `td120-repair-pass2.md`. This was not another
full audit; no runtime change was included in this closing delta.

- The TD-113 test scopes the successor binding to `ime-mutation`, checks the
  original path, SHA-256 and mode against the immutable V4 predecessor, and
  checks the owning TD-120 task, review path, score metadata, current source
  SHA-256 and mode against the successor receipt. Other protected-artifact
  checks and the existing TD-115 exception remain unchanged. The actual
  composition source independently matched successor SHA-256
  `bfee6bdabc148de1cccf44558d176e3d78c101f42b8912e4ffa25b9de131f713`
  and mode `0664`. This binding establishes provenance, not functional
  acceptance or an independent score calculation.
- The mutation-monopoly test replaces obsolete local-variable spellings with
  checks of the production effect route: authorization precedes execution and
  deletion; the schedule is Preflight → Backspaces → Replay →
  SuppressAfterSuccess; execution branches bind the actual effects; Replay
  retains `input_gate.clone()`. The production loop stops on error. No test
  was removed or renamed, and no runtime implementation was altered by this
  contract repair.

Final local source-contract and evidence SHA-256 anchors:

```text
tests/td113_hybrid_source_contract.rs                 0a19679a438ba85e1fef467b73fa92bbe4e30b70314674cab498c543bdbf7fd2
tests/text_mutation_monopoly_contract.rs             3cf3fea05577b38eb0f87a438378d2b7e50816a3bf9baf8c4af53c61c3e6fd6c
tech_debt/evidence/td120-composition-mutation-successor.json  4768f19c1c1afa6651106aa8edd2c8a98996c6198c466d956f995b0ad1be9af0
tech_debt/evidence/td120-repair-pass2.md               59fb7a77b6eb518fb6279a80363a7a894250cc37698ef2362eb5f90b3b24a4eb
```

Independently read over read-only SSH to `e@192.168.3.94` and verified both
remote receipt hashes:

```text
/home/e/projects/lay-td119-gate-v1/target/verification-logs/td120-final-20260906T1900Z/09-check-lay-changed-final.log
SHA-256 6fd091055a369e0d4b038f6885a635e8785f3cd07a54338df41e6b3f39ad2259
/home/e/projects/lay-td119-gate-v1/target/test-lanes-results/SUMMARY.json
SHA-256 53e2932a9737e3627fa8190250e0d6017ad2a0bcfb2e433145e08e786ae47858
```

The summary verdict is **PASS: 2,554 selected = 2,518 correctness + 36 package**,
with **0 known semantic failures and 0 infrastructure failures**. Total elapsed
time is 348.743 s; the correctness and package lanes report 292.646 s and
54.823 s respectively. The final log ends `== lay changed check OK ==`.
This is the selected canonical-lane denominator, not a claim that all 2,580
discovered tests passed. The log's unsafe-edit scoreboard separately reports
PASS, 0 gate failures, observed risk 0, and 200 observations comprising
196 candidate-before-apply and 4 actions; it is not a general quality proof.

The parent independently verified source/graph parity; this reviewer verified
the remote log and summary, not a new test run or an independent remote source
manifest comparison. Only this report was written by the reviewer. No Cargo,
service, installation, Git write or graph rebuild was performed.

This checkpoint does not certify commit/push completion, TD-121 runtime wiring,
TD-122 closure, release 1.0.66, installation, live input behavior or heldout
quality. V1 late/unbound requests remain the explicit TD-122 KNOWN_RESIDUAL.
No latency, RSS or package-size gate is inferred from the canonical test timing
or package-lane count; broader quality remains UNKNOWN beyond the measured
test scopes documented above.
