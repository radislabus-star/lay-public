# TD-009: Converge The Manual-Toggle Visible Postcondition

Status: `DONE`
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

## Effective Route Correction (2026-08-29)

The original committed-tail route is superseded after two immutable live
failures. The V2 post-mutation refresh and V4 pre-mutation refresh both relied
on `RequireSurroundingText`, but GTK emitted no new `SetSurroundingText`
callback in either position. Official IBus source also proves that consecutive
legacy engine effects can straddle the `processing_key_event` boundary. The
legacy `DeleteSurroundingText` plus `CommitText` pair therefore cannot provide
the required deterministic visible transaction for this client.

Preserved evidence:

```text
V2 failed receipt       a8965f58bd40b93e367223b613daa27107daab5e3fe0dc5a7f6222743d51f576
V2 failed trace         0ec6b7abaa25ab0433339a1c7013d54dc1b1619adc9d3f357fe13c555acb2c6d
V4 failed receipt       b04d31bcaa7aee5134715cb2d29ef430e6319ee87d26e0651b74b8e55ad75e7a
V4 failed trace         0492d72d1b7055eeaf395472143dd2a408fb8d76e6722327fa731615ce4e23e8
selected route receipt  6f26b67dd62227980726b8bf8ed9945a4003227a269a1d14353d282383f1e8f8
```

The effective committed-tail route is now:

```text
one daemon pair detector
-> one exact IME VisibleTailV2 snapshot
-> shared literal manual-toggle plan
-> physical input grab
-> exact source/focus/epoch/tail lease revalidation
-> controlled target-layout readiness
-> the same lease revalidated after the controlled engine handoff
-> one daemon uinput Backspace plus exact text replay
-> managed GTK visible-text proof
```

This correction narrowly supersedes two original non-goals for the legacy GTK
committed-tail lane: synthetic key events are admitted under an active physical
grab, and target-layout readiness occurs before replay because those key events
must be interpreted in the target layout. No deletion occurs before the exact
lease passes both validations. Active composition remains IME-owned; daemon
WordBuffer replay remains a separate route. Timers, polling, retries, a second
mutation, detector changes, and installed-runtime changes remain forbidden.

The replay plan must come from the exact IME-observed committed tail, never
from daemon WordBuffer. This is the required correction to the old 1.0.39/1.0.40
delegated-uinput route, which could lose an accepted autocomplete suffix that
the daemon had not observed.

### Candidate Live Alignment

The first V6 candidate reached the exact replay dispatcher but failed closed
before layout or text mutation. GNOME `CurrentLayout` reported the active IME
component ID `lay-ime-us`, while the one-shot verifier compared it only with
the equivalent extension layout ID `us`. A second debug-only candidate run
confirmed this exact mismatch in daemon stderr; both runs retained `ghbdtn`,
restored the desktop, and left installed binary identities unchanged.

The verifier now accepts only the two frozen identifiers for the same expected
language: the extension layout ID (`us`/`ru`) or its exact Lay IME component ID
(`lay-ime-us`/`lay-ime-ru`). It still requires exact IBus engine identity and
rejects the opposite language and unrelated XKB identifiers. This is an
identity-alignment correction, not a retry, polling, or capability fallback.

The first independent architecture alignment review scored the candidate
`6/10 BLOCKED`. Two findings were accepted and repaired: the exact lane now
uses dedicated no-fallback D-Bus calls, and the IME no longer arms suppression
before the daemon has captured and revalidated the exact tail. The claimed
missing handoff acknowledgement was rejected: the required acknowledgement is
the second `VisibleTailV2` lease from the focused target engine, which checks
target layout plus unchanged source, epoch, and suffix before mutation. The
protected Double Shift contract and its owning architecture document were
updated to remove the superseded IBus-only route.

The next candidate exposed the corresponding cross-engine requirement: the
target Lay IME instance received a new epoch because the selected route had not
published the existing tail handoff before `ActivateLayout`. The source engine
now publishes the exact tail and the existing 700-ms expiry-bounded handoff
lease before returning the typed disposition. This adds no wait, wakeup,
polling, or retry; it only prevents stale handoff state from surviving if the
immediate controlled source transition never occurs. Suppression remains
unarmed until the daemon's checked pre-delete step.

The first V6 bidirectional matrix passed each single direction and a later
inverse, but rejected the zero-pace executor on two required boundaries. In a
four-tap burst, the second pair captured the target engine after only the first
character of the first replay was visible. In the accepted-autocomplete case,
the replayed trailing Space re-applied the prepared autocorrect because the
one-shot suppression consumer had become test-only during the historical
boundary-prefetch refactor. These are executor defects, not detector or literal
mapping defects. The effective candidate reuses the existing bounded paced
Backspace/text emitters and restores production consumption of the existing
shared suppression before Space autocorrect lookup. It adds no gesture delay,
polling, retry, or second mutation.

### Final Ownership Repair And Route Selection

The independent review then found three real ownership gaps. The selected
implementation closes them without broadening the replay mechanism:

- suppression and handoff cancellation are keyed by exact engine path and
  epoch, so an older failure cannot cancel a newer gesture;
- the cleanup guard assumes suppression may have been armed before the D-Bus
  reply is received and performs exact suppression-then-handoff rollback on a
  lost or malformed reply;
- the GNOME focused-window identity is captured after input isolation and
  rechecked after layout handoff, before deletion, between deletion and insert,
  and after insert.

An attempted process-wide grab of every keyboard and pointer was rejected by
the immutable V10 matrix. It swallowed the fast boundary Enter in both
four-shift cases and passed only 5/7 scenarios. The final route therefore keeps
the proven triggering-keyboard grab and relies on exact focus identity checks
for cross-window fail-closed behavior.

The provisional V11 candidate passed its seven-scenario matrix:

```text
matrix                         7 / 7 PASS
manual-toggle gestures        10
exact suppression arms        10
generic suppression arms       0
malformed traces               0
desktop restoration           PASS
installed runtime changed     false
```

Evidence:

```text
V10 rejected global grab
docs/structural_gates/receipts/
TD_009_MANUAL_TOGGLE_EXACT_OBSERVED_REPLAY_LIVE_V10_2026-08-29/

V11 provisional route
docs/structural_gates/receipts/
TD_009_MANUAL_TOGGLE_EXACT_OBSERVED_REPLAY_LIVE_V11_2026-08-29/

V11 receipt SHA-256
e2684d6f7a0a87cc5c3ee28db25e54021a1e4494c87073087f3960c7da30d588

candidate daemon
a74e6376d2eb3704b435d4742fcf5bf864e8fad3c33dec1f12befd8897c342f0

candidate engine
649106389719d665190db878590348fc221a88c3bca0718f7b51faf41d265053
```

V11 was superseded when the independent review identified two untested input
ordering boundaries. The resulting immutable live attempts were retained:

```text
V12  5 / 8  REJECTED_PRE_GRAB_QUEUE_RACE
      fast inverse gestures were swallowed and a zero-settle ordinary key
      reached the old field before exact replay isolation
      receipt 69d7fd0c53d32970fc3c86c57f69e108b0f29a1b585ffc3900a07121c97180c6

V13  6 / 8  REJECTED_QUEUED_INVERSE_WITHOUT_SETTLEMENT
      a queued inverse captured a partial committed tail while the first replay
      was still reaching the client
      receipt 87d14e93822e3f5349497815216d9785ed01a4c960203037943d7d58cf58c41f
```

The bounded repair queues physical input only while the triggering keyboard is
isolated, forwards ordinary keys and boundaries in order after the exact
replay, and recognizes a queued second Shift pair as one inverse gesture. The
inverse gesture may wait at most 80 ms for the exact full prior tail, and only
when that inverse is already queued. D-Bus calls have a 250-ms method timeout;
exact suppression expires after 700 ms. Neither bound delays admission of an
ordinary Double Shift gesture.

The final V16 matrix passed all required boundaries:

```text
matrix                              8 / 8 PASS
manual-toggle gestures                  11
exact suppression accepts               11
generic suppression arms                 0
queued ordinary keys forwarded           1
queued inverse gestures replayed          2
malformed traces                          0
desktop restoration                  PASS
installed runtime changed             false

receipt SHA-256
e9a3468a942495215ab997ad95a945428fa0abf68c452fe5462478991bf350f5

candidate daemon
5bda386281c176c3fb502d20566c63eda280dd9dfcc589e4ed0edcb6ea949e76

candidate engine
70852988c37916020bdfbfbc5fcb73fd980e405adbcaa00b216fa413f9ada9f3

candidate test input
ab9628fd8995edfa5ecc9d9d086e308abfdf7906d9c71938e916b55d7d287b2c
```

The focused field receipt is exact when `FocusInId` supplies it. On clients
that expose only the engine-scoped fallback, the lease remains engine-scoped
and the GNOME focused-window identity is checked independently at every
mutation boundary; the evidence does not claim unavailable widget identity.

## Completion Record

- Implementation commit: recorded by the follow-up closure commit after this
  task's implementation commit is created
- First independent review: `6/10`; exact suppression rollback, no-fallback
  focus calls, and handoff cancellation findings repaired
- Final independent review: `4/10`; bounded input queueing, field identity,
  D-Bus timeout, suppression expiry, and evidence-scope findings repaired
- Corrective passes: `2/2`; no further review loop admitted
- Verification: focused daemon tests `8/8` PASS; focused engine tests `3/3`
  PASS; queued-pair and queued-settlement tests PASS; runtime-smoke isolation
  `39/39` PASS; built-in script embedding PASS; release candidate build PASS;
  final managed GTK matrix `8/8` PASS; `scripts/check-lay-changed.sh` PASS
- Selected evidence: `docs/structural_gates/receipts/`
  `TD_009_MANUAL_TOGGLE_EXACT_OBSERVED_REPLAY_LIVE_V16_2026-08-29/`
- Runtime authority: installed daemon, engine, and test-input hashes unchanged
