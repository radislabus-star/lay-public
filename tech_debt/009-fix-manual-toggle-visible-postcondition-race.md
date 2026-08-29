# TD-009: Converge The Manual-Toggle Visible Postcondition

Status: `READY`
Priority: `P0`
Class: user-visible Double Shift correctness
Size: `M`
Depends on: TD-002

## Why Now

The isolated TD-002 GTK proof reproduced an existing nondeterministic failure in
both manual-toggle directions. The IME computes and dispatches the exact
replacement, but the GTK client can still contain the original token when the
scenario submits Enter 800-900 ms later. A manual toggle must either complete
its visible edit and matching layout handoff exactly once or report a bounded,
explicit failure; it must not remain indefinitely between those states.

This is not an order-contamination defect. The same case passes alone and in
one batch while failing in another fresh per-case engine lifetime. Both failed
traces reached the same dispatch stage and then stopped at a pending visible
postcondition.

## Sealed Reproduction Evidence

Evidence root:

```text
docs/structural_gates/receipts/
TD_002_RUNTIME_SMOKE_ISOLATION_2026-08-29/
```

Observed successful route:

```text
ibus_manual_toggle_plan
-> ibus_surrounding_replace:
   delete_commit_dispatched_waiting_exact_final
-> ibus_layout_sync_owner: visible_postcondition_confirmed
-> ibus_visible_postcondition: observed
-> ibus_layout_sync: ok=true
```

Observed failing route in two independent managed-engine lifetimes:

```text
ibus_manual_toggle_plan              exact
ibus_surrounding_replace             dispatched
ibus_visible_postcondition           pending
layout sync                          absent
client text                          original token
manual-toggle count                  exactly 1
malformed trace records              0
```

The failures changed direction between runs:

- `ORDER_REVERSED`: `слово -> ckjdj` remained `слово`;
- `ORDER_REVERSED_REPEAT`: `ghbdtn -> привет` remained `ghbdtn`.

The existing 800/900 ms sender delay is already much larger than the expected
interactive budget. Increasing that delay would hide rather than repair the
authority transition and is forbidden.

## Target State

For an admitted committed-tail manual toggle:

1. one exact replacement plan is produced;
2. one text mutation transaction owns delete plus commit;
3. the engine obtains an exact client-visible postcondition or a bounded,
   explicit failure;
4. layout synchronization occurs only after the exact visible postcondition;
5. Enter immediately following the existing scenario settle cannot commit the
   pre-toggle token;
6. a second Double Shift performs the inverse conversion through the same
   route, with no one-shot latch or stale authority.

No candidate/ranking behavior, keyboard mapping, or daemon-side detector rule
changes are admitted.

## Scope

- Reconstruct the exact GTK/IBus callback sequence between
  `delete_commit_dispatched_waiting_exact_final` and
  `observe_visible_postcondition()`.
- Add a deterministic engine-level regression that reproduces the missing or
  delayed exact surrounding snapshot without wall-clock sleeps.
- Make the smallest ownership correction in the committed-tail/text-output
  transaction. Prefer an already existing callback or acknowledgement path over
  polling or another timer.
- Preserve `pending_visible_postcondition` epoch, snapshot, target-layout, and
  feedback identity checks.
- Keep layout synchronization downstream of confirmed visible text.
- Verify both `US -> RU` and `RU -> US`, then four taps as two inverse toggles.
- Re-run the TD-002 managed GTK cases with fresh per-case evidence.

## Non-Goals

- Do not alter the daemon pair detector or key timing thresholds.
- Do not add a second replacement attempt, blind retry, synthetic key event,
  arbitrary sleep, or layout change before text confirmation.
- Do not accept the old token as a successful toggle.
- Do not weaken exact surrounding-text or generation checks.
- Do not combine this repair with TD-003 preedit-suffix convergence.
- Do not install or restart production Lay as part of implementation review;
  candidate binaries run only through the admitted TD-002 boundary.

## Consequence Analysis

- **Authority:** the IME remains the sole owner while it has committed-tail
  authority. The daemon route is unchanged.
- **Text safety:** one plan must still yield at most one delete and one commit.
  Any proposed fallback must prove it cannot duplicate text after a late client
  callback.
- **Layout:** target layout changes only after exact visible confirmation. A
  timeout or mismatch leaves layout unchanged and clears/quarantines pending
  authority according to the existing contract.
- **Latency:** no additional steady-state delay is admitted. The repair should
  remove an unresolved state transition, not extend the scenario deadline.
- **Terminal clients:** surrounding-text capability and terminal passthrough
  remain distinct. Unsupported clients must keep their existing safe route.
- **Learning/feedback:** system feedback cannot publish before the same visible
  postcondition that owns layout synchronization.

## TDD Plan

1. Freeze the two failed trace projections and successful projection as a
   deterministic state-machine fixture.
2. Add a failing test where delete/commit dispatch succeeds but the first
   surrounding snapshot is pre-edit or incomplete.
3. Assert a later exact snapshot completes once, syncs layout once, and clears
   pending state.
4. Assert duplicate/late callbacks cannot commit or sync twice.
5. Assert epoch, token, and layout-generation mismatch fails closed.
6. Assert Enter/boundary after the existing settle observes converted text.
7. Run current manual-toggle, text-mutation monopoly, candidate visibility,
   known-word suppression, layout-sync, and terminal-passthrough tests.
8. Run managed GTK `ghbdtn_fast_lshift_enter` and
   `slovo_ru_to_us_fast_lshift_enter` individually and in both orders. All four
   receipts must pass without changing scenario timing.

## Acceptance Gates

- The deterministic missing/delayed-postcondition test fails before and passes
  after the repair.
- Exactly one replacement plan, text mutation, visible confirmation, and layout
  synchronization occur per toggle.
- Both conversion directions and two consecutive inverse toggles pass.
- No timeout/sleep/retry threshold is increased or introduced.
- Existing stale-generation and duplicate-callback tests remain fail-closed.
- Fresh TD-002 managed GTK receipts pass individually and in both orders with:
  expected text exact, one manual toggle per case, malformed trace `0`, and
  desktop restoration verified.
- Installed runtime authority is unchanged during candidate review.

## Risks And Guardrails

- Re-dispatch after an ambiguous client callback can duplicate text. Never make
  a second mutation unless the transaction proves the first had no effect.
- Synchronizing layout early can make the next key use the wrong mapping while
  text is stale.
- Clearing pending state too early can suppress a valid late acknowledgement;
  retaining it forever can poison the next token. Tests must cover both edges.
- GTK and terminal-like clients expose different surrounding-text behavior;
  keep capability handling explicit rather than broadening one client fix.

## Independent Review Brief

Trace the exact mutation and acknowledgement ownership from plan through GTK
surrounding text, including duplicate and late callbacks. Reject sleeps,
retries, early layout sync, or any path that can apply the edit twice. Score
1-10.

## Completion Record

- Commit: pending
- Review score: pending
- Verification: pending
