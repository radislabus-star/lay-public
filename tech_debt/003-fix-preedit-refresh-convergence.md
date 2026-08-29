# TD-003: Fix Pending Preedit Refresh Convergence

Status: `DONE`
Priority: `P0`
Class: user-visible correctness and latency
Size: `S`
Depends on: TD-002

## Why Now

A prior live session observed a 203 ms worker delay where typing `пров`
temporarily showed a stale completion and produced the visible shape
`провверка`. That trace was not sealed, so the number is a reproduction lead,
not an acceptance denominator. The current transition and stale test expectation
independently establish the defect to encode deterministically.

## Evidence

- `begin_pending_precognition_refresh()` currently clears candidates and the
  displayed suffix immediately.
- `PreeditFastState` still retains the previous full target, but the pending
  refresh does not synchronously project the remaining suffix from that target.
- The current test named
  `pending_refresh_invalidates_candidates_without_hiding_the_surface` asserts
  an empty suffix and therefore preserves the defect.
- The worker generation check prevents stale acceptance, but it does not prevent
  a stale or jumping visual surface while the current generation is pending.

## Target State

When the typed prefix still matches the retained target, the visible suffix is
shortened synchronously. Candidates remain invalid for acceptance until the
matching worker result arrives. If identity or prefix does not match, the
display fails closed with no retained suffix.

## Scope

- Add one pure helper that derives the suffix only when the retained target has
  the exact current partial as a prefix.
- Make pending refresh clear actionable candidates and then install only that
  non-actionable visual suffix.
- Preserve generation, frame identity, layout generation, and output capability
  checks for worker application.
- Keep suffix-only preedit with cursor position zero for terminal-like clients.
- Recheck `managed.rs`, dirty-preedit scheduling, Backspace, candidate cycling,
  Tab acceptance, and boundary close behavior.

## Non-Goals

- Do not modify the Double Shift detector or ownership route.
- Do not make the IME a second correction/ranking authority.
- Do not change candidate language, ranking, or learning.
- Do not add sleeps, attach delays, or UI debounce heuristics.

## TDD Plan

1. Replace the stale expectation with a failing slow-worker test:
   retained `проверка`, current partial `пров`, visible suffix `ерка`.
2. Add mismatch cases for a changed token, layout generation, and target.
3. Assert Tab cannot accept the retained visual-only suffix.
4. Assert a current worker result replaces or hides it exactly once.
5. Assert the exact emitted `UpdatePreeditText` sequence and attributes; checking
   only internal `preedit_suffix` state is insufficient.
6. Run the whole IME regression class, not only the new unit test.

## Acceptance Gates

- Targeted `lay-ibus-engine` preedit tests pass.
- Slow-worker test proves no `провверка` intermediate surface.
- Double Shift owner tests pass in both directions and four taps produce two
  inverse toggles.
- Layout synchronization, known-word suppression, candidate visibility,
  Backspace, and terminal passthrough tests pass.
- Managed GTK smoke uses candidate binaries and records a fresh JSONL trace with
  no stale candidate acceptance or duplicate surface.
- Managed GTK smoke runs through the TD-002 ownership boundary and restores the
  exact pre-test desktop state. No global install is required; release remains a
  separate decision.

## Risks And Guardrails

- A retained suffix must be display-only. Reusing it as an actionable candidate
  would reintroduce stale Tab acceptance.
- Prefix comparison must use the same live partial used by candidate selection.
- Clearing a mismatched target must remain immediate.

## Independent Review Brief

Trace display state and acceptance authority separately. Look specifically for
stale Tab acceptance, layout-generation drift, and Double Shift regression.
Score 1-10.

## Completion Record

- Implementation commit:
  `e7e1167d6ac3fc9d9e1f42a52de4197c3f334b1d`
- Implementation: pending suffixes now have an explicit display-only state;
  only a matching, current, successfully published worker result installs
  completion authority. Tab, arrows, Alt, late-worker, cursor-ack, focus and
  publication-failure routes retire that state fail-closed. Active composition
  keeps its typed buffer visible while only its pending suffix is retired. The
  atomic frame route still materializes its candidate synchronously in the
  submitted proposal.
- Smoke contract: a positive `про -> проверка` route and a pending
  `пров -> пров` route now assert exact managed commits, preedit sequence,
  shortened-display event, clear count, candidate-IME key count and completion
  acceptance count. A text-only GTK bypass cannot pass this contract.
- Review pass 1: `3/10`; it found completion fallback and atomic-route gaps.
- Review pass 2: `4/10`, fresh-context agent
  `01a04db8-f4b7-7b53-8abe-30dc10fa3c8e`; it found hidden fallback after late
  clear, post-lock lateness, deferred cursor resurrection, Alt press/release
  races, pre-publication authority and missing route proofs. All findings were
  corrected. Per the two-pass limit, no third score was fabricated; closure is
  based on the corrected source and objective gates below.
- Verification: `88/88` focused preedit tests PASS; atomic submitted-frame
  proof PASS; `43/43` runtime-smoke isolation tests PASS; `fmt --check`,
  `py_compile`, `git diff --check`, and `scripts/check-lay-changed.sh` PASS.
  The final full engine run passed `275/276`; its sole failure was the existing
  TD-006 wall-clock assertion, and an isolated rerun failed a different timing
  sub-gate while all IME semantics remained green.
- Live evidence:
  `docs/structural_gates/evidence/TD003_PREEDIT_REFRESH_CONVERGENCE_V16_2026-08-29/`.
  Receipt SHA-256:
  `8cde9837198ec4868a4fdd91e5e22723b0b3c58af78683647675e5e9d010b58a`;
  manifest SHA-256:
  `5917af92a2e71ef87dd0cea45a1e60eae46ee5bbc87ffd7fb00e2eef79e7e097`.
  V1-V15 remain rejected or superseded diagnostic evidence.
- Runtime authority: unchanged; no production install or release was made.
- Push: `origin/codex/l1-exact-peak-search`.
