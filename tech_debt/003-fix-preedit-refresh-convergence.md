# TD-003: Fix Pending Preedit Refresh Convergence

Status: `READY`
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

- Commit: pending
- Review score: pending
- Verification: pending
