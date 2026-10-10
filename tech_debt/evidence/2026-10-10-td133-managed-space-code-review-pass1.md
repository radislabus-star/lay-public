# TD133 implementation review — pass 1 of 2

**6/10 — changes requested before source acceptance.** The selected transport repair is small and uses the existing output/word owner, but the newly handled Legacy Space is not covered across Reset, and two existing surrounding-capable Legacy contracts still require the opposite transport. No installed/native success or original-cause conclusion follows from this review.

Read-only review; no tests, builds, runtime commands, GUI, installation, restarts or sub-agents. Graphify execution was excluded by the review assignment; existing binding metadata was inspected only as navigation. The reviewer wrote only this report.

Reviewed checkout `/home/ubu/projects/lay-space-boundary-shift-20261007`, HEAD `24bcea5256661954d5aba97ff037d6a419e1e2cb`. Exact source hashes rechecked at review cutoff:

- `managed.rs`: `718bb6ba3d046e99c93f7697d38a2993b1eec2010c0d862aefe5189ae08af913`.
- `context_admission/adapter/tests/terminal_delivery.rs`: `b6252344e08a7335b635969a5140839a3cf88b5d6b098c59f33da09e90d7cca4`.

ADR/card/owning-document narrowing was observed during review; the production/test hashes remained unchanged.

## Required repairs

**R1 — P2: Reset between the new Space commit and its release loses the press receipt.**

The changed predicate at `src/bin/lay_ibus_engine/managed.rs:246` makes ordinary surrounding-capable Legacy fallback Space handled and calls `commit_managed_passthrough_char(' ')` at lines 259/344. That helper (`composition_commit.rs:223`) updates `last_commit_at`, calls `push_tail_char`, clears the managed snapshot floor for whitespace and deliberately does not arm `arm_managed_commit_reset_echo` for whitespace. `push_tail_char` (`preedit.rs:1433`) clears `managed_word_start` at the boundary and resets `word_input_mode`.

An accepted Reset enters `WindowInteraction::observe_lifecycle` (`window_interaction/observation.rs:3838`) and calls `reset_for_ibus_soft_reset`. The managed echo predicate requires a current word-start witness (`observation.rs:869`), so it is false after this Space. `state.rs:453` then clears `handled_press_keycodes`; the subsequent Space release reaches the unhandled-release path (`ibus_interface.rs:916`) and returns false. The ADR explicitly requires that release to remain consumed. Existing `terminal_delivery_midword_exact_snapshot_survives_owned_commit_reset_echo` documents Firefox Reset-before-release and preserves the receipt for glyphs, but does not test closing Space. The new test supplies release immediately after Space and cannot detect this edge.

This is a deterministic source-path finding, not an executed regression or proof of native duplicate text. The smallest first action is one callback/signal regression using the existing Reset helper between fallback Space and its release, asserting one Space commit, no deletion/additional commit, the matching handled release and correct next glyph. If confirmed, preserve only the same-field transport press receipt through the existing ownership/receipt mechanism; do not retain a closed-word correction grant, widen generic Reset authority, add an independent timer/queue or suppress arbitrary releases. Record the lifecycle consequence before any repair.

**R2 — P2: affected ordinary Legacy transport contracts remain incompatible with the new predicate.**

`context_admission/adapter/tests/residuals.rs:12597` (`native_space_observed_boundary_retains_strict_predecessor_until_exact_reset_receipt`) and line 12747 (`native_space_legacy_no_apply_and_manual_suppression_close_scope_without_edit`) use `initial_observed_tail_reset` at line 3586: ordinary purpose 0, surrounding support, ManagedCommit glyphs, followed by exact receipt. Both explicitly require unhandled Space and only preedit clear/hide effects; the new predicate instead handles and commits Space. Their suppression variants and press/release transport assertions are also affected. These are statically predicted source-test incompatibilities; the reviewer did not execute them.

The tests additionally prove inert predecessor retention, owner/epoch/revision identity, delayed exact receipt, Tab refusal and exact manual-tail replay. Preserve these substantive safety assertions. Explicitly migrate only the affected ordinary-input transport oracle under the new ADR (exact one Space commit and matching handled release), retaining separate unchanged opaque/terminal native coverage, or refine the production scope if these ordinary routes were intended to remain native. Do not delete, xfail or relax the safety/provenance assertions to obtain PASS. Name the affected protected test path in the decision as required by the canon. These tests should form part of the next affected check, including manual suppression.

## Assessment of the selected design

The production delta is one shared predicate plus its constant import: no new owner, model/ranker condition, cache, queue, timer or client name. Surrounding capability is used to choose literal user-key transport, not to grant deletion. The successful verified replacement branch returns before the changed fallback; owned-preedit and Atomic conditions retain their existing behavior. Proven native terminals are excluded by surrounding support/purpose, and the two new native-preservation scenarios verify a stored ManagedCommit word with opaque or declared-terminal Legacy input.

The existing commit helper changes `last_commit_at` and clears the whitespace floor; this does not by itself create a fresh surrounding receipt. `push_tail_char` retains the existing boundary learning and invalidation path. No native fallback is added after failed or indeterminate CommitText. The newly reachable Reset/release coupling in R1 is the remaining material defect in that state transition.

The predicate is also evaluated after `word_input_mode.get_or_insert(initial_mode)` at `managed.rs:238`: leading or repeated Space in an ordinary surrounding-capable field can now commit even without preceding ManagedCommit glyphs. Describe this scope explicitly; a short leading/repeated-Space sequence is a useful additional boundary check. No literal-word restriction is justified.

## Evidence and limits

- Baseline receipt `/home/ubu/.cache/lay/development/td133-baseline-red-s8wjawmz/REGRESSION.json`: 1 selected, 0 passed, 1 expected source-contract failure (`[]` versus `[" "]`). It does not reproduce either original native failure. Baseline regression-test bytes precede the later expanded test, and must keep their own hash.
- Candidate receipt `/home/ubu/.cache/lay/development/td133-candidate-green-9cqk5s77/REGRESSION.json`: 2 selected, 2 passed, 0 failed; production/test hashes match this review. This is the two-test transport denominator, with four internal scenarios, and excludes the R1/R2 schedules.
- Four authorized old-byte GTK/Qt controls pass; their grant is consumed. They show first native Space inserted before the next glyph in those streams and are not candidate acceptance.
- Three older failed `7b96` Qt surfaces support a mixed-transport experiment; decoded input is correct, but they have no widget negative chronology. Original `2bd` attribution remains open; original 62 PASS / 2 FAIL / 64 remains unchanged.

Consecutive CommitText ordering on the signal connection does not prove client application order or survival within a Wayland `done` batch. The current terminal contract already documents this risk. Native successor, performance/latency, affected all-field behavior, full canonical source gates and graph/manifest successor acceptance remain separate pending gates. No native result or universal acceptance was inferred. Installation must not proceed from these two source tests alone, and any native successor needs its own authorization and exact-byte evidence.
