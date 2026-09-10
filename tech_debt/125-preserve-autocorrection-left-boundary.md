# TD-125 — Preserve the preceding word and separator during autocorrection

Priority: P0 (reproduced terminal deletion beyond the authorized edit plan)
Status: IN_PROGRESS / INSTALLED_MANUAL_PENDING —1.0.66 installed/loaded; final gates GREEN, review10/10; physical acceptance pending
Scope: user-requested variable-length correction on the same private client
infrastructure. Not general Wave ranking or a new release feature bundle.
Owning evidence: [diagnosis](evidence/td121-client-boundary-diagnosis.md).
Executor regression: [RED/GREEN](evidence/td125-terminal-plan-execution.md).

## Required behavior

For visible `left + separator + old`, a single accepted correction produces
exactly `left + separator + new + " "`. The left text and separator are
unchanged, regardless of whether the new word is longer, shorter or equal in
length. No compensating extra Space or extra Backspace is permitted.

The user's logs contain `лово → слово ` with planned delete4. This is useful
input for a fixture, but not an observed post-edit terminal screenshot. The
logs do not establish that the preceding separator actually disappeared in
that particular event. Keep historical reproduction and constructed regression
proof distinct. Raw user logs remain private outside Git.

## Immediate work, in order

1. [x] Freeze live logs, identify real mutation route and old/new lengths.
2. [x] Explain the private client's independent process disappearance:
   missing undeclared L2 lexical artifact; SIGABRT and controlled one-artifact
   comparison recorded in owning evidence. Candidate survives when admitted.
3. [x] Make that dependency explicit in the existing harness, with early
   missing/hash-mismatch failure and remote RED/GREEN. Preserve old manifests.
   Runner patch4lines;84 tooling checks passed,1optional skip. Ordinary client
   survives; still0/5 `prefetch_not_ready`. Independent review passed10/10;
   this substep still does not imply task DONE or an accepted Git checkpoint.
4. [x] Freeze an explicit successor client contract: surrounding-text consumer
   must advertise and publish it; terminal consumer must actually execute
   terminal erase controls. The original 0/5 fixture is immutable history,
   not a positive terminal proof. The test-only Readline consumer now receives
   captured production executor payloads; it is not a new private IBus client.
5. [x] Reproduce the user's visible loss or identify the missing observation.
   Record the first mismatch: observed input, verified plan, emitted frame,
   terminal-consumed text, cursor, or subsequent Lay tracking. Constructed
   executor sequence reproduces loss; historical input sequence remains unknown.
6. [x] Only after causal evidence, score minimal runtime fixes and implement
   the smallest one. Freeze a regression that detects the old failure or
   explicitly label a controlled mutation test. Do not call them equivalent.
7. [ ] Run remote affected checks and actual-client proof; obtain fresh-context
   independent review with score1–10, at most two correction passes. Record any
   remaining finding instead of weakening the gate. Then commit/push task-only.

## Regression matrix and pitfalls

- Growing, equal-length and shrinking replacements; full left-word sentinel
  plus one Space, two Spaces and a non-ASCII context prefix.
- `old.chars().count()` and `new.chars().count()` are separate quantities.
  UTF-8 byte count, code-point count, grapheme count and terminal cell width
  are not interchangeable. Add a combining-character case if the consumer
  supports it, reporting refusal otherwise rather than guessing geometry.
- Space autocorrection and explicit Tab acceptance are distinct entrypoints.
  Active composition must not delete already committed left text. Appending a
  completion suffix must not be treated as full-token replacement.
- Assert actual final client text, cursor, delete extent, insert payload and
  number of mutation frames. Internal tail equality alone is insufficient.
- Keep legacy terminal, GTK/surrounding-text and atomic transports separate.
  Preserve daemon-only legacy Double Shift; do not introduce physical replay
  or another detector to repair this report.
- Negative control: an intentional over-delete must fail preservation of the
  exact left sentinel. Unmatched/stale input ownership must not authorize an
  edit. Do not relax SafetyGate, verifier or snapshot identity.
- No sleeps, retry-until-green, larger Space budgets or warm-only substitution
  for cold authority acceptance. Existing cold `prefetch_not_ready` stays OPEN.

## Scope and stop conditions

No historical root cause of the separator loss is yet established. Absence of terminal
postcondition is a coverage/contract limitation, not causal proof that its
current transport always fails. Do not disable terminal autocorrection or
introduce delayed active composition as an unannounced product change.

The one-file dependency repair only restores sandbox completeness for the
observed crash. More complex terminal ownership/acknowledgement redesigns
require their own explicit decision if a minimal correction cannot suffice.
No release/install/DONE until the respective existing gates pass. TD-121 and
TD-123 retain their own acceptance and sequencing; nothing here declares them
complete or starts general Wave improvements.

## Bounded mechanism and consequence analysis — 2026-09-07

Source fact: `ImeCandidateAccept` authorizes a minimal completion plan. For
`пров -> проверка`, the logical request covers four old characters, but the
authorized operation is delete zero and insert `ерка`. `replace_committed_tail`
uses the authorized insertion with the request's deletion count on TerminalErase.
Its internal tail then records the full logical target. This is a concrete
executor/plan mismatch, independent of cold candidate availability. A later
full-token edit can therefore erase the left boundary from the shorter actual
client text. Whether the user's historical event followed this sequence remains
unverified. The ordinary Space full-token route does not minimize this plan;
its old-count arithmetic is not implicated by this finding.

Options (engineering judgment, not measured quality scores):

1. **9/10, selected:** use the authorized deletion count for the terminal
   payload, matching the already-authorized insertion. Preserve output-profile
   admission exactly; the proposed profile-selection change was removed in review.
   Preserve the request count for logical tail replacement and snapshot identity.
   This removes the mixed-plan execution without a new owner or transport.
2. **6/10:** force candidate completions to full-token replacement. It can
   produce correct text but adds needless destructive output and changes the
   shared planner, including clients whose append-only completion already works.
3. **2/10:** disable terminal completion/correction. It avoids this path by
   removing functionality; it does not repair execution of the verified plan.

Invariants and consequences before implementation:

- Candidate/lattice retention, ranking, SafetyGate and verifier authority are
  unchanged. The existing sealed plan stays the sole physical edit authority.
  No literal fixture text enters runtime decisions. No additional false-apply
  permission is introduced; even append-only plans retain existing client
  admission. No-geometry/no-SurroundingText refusal stays unchanged.
- No new timers, waits, cache, queue, generation, model/package reads or reload
  behavior. Deadlines and cold `prefetch_not_ready` remain unchanged. CPU/RSS
  impact is bounded to existing scalar selection; completion emits fewer DEL
  bytes, not additional allocations or work. No latency improvement is claimed.
- Epoch/focus/stale-tail checks and duplicate handling remain in place. Logical
  state and learning still use the full target, not the appended suffix. A
  regression must detect accidental replacement of that logical request count.
- GTK/surrounding-text deletion already uses the plan and stays unchanged.
  Terminal remains one commit, without physical replay. Atomic output capture
  can prove emitter payload but not legacy D-Bus delivery; retain that distinction.
  Cursor-moving plans still require SurroundingText. No daemon/Double Shift
  ownership or failure/fallback policy changes.
- Rollback is the small executor hunk; regression/proof tooling remains useful.
  No installed binary is touched. Do not bundle unrelated dirty TD-121 changes.

Proof plan: first run the new tests against the unchanged executor (real RED),
then the selected fix. Assert authorized suffix-only output, full internal tail,
stale refusal with zero effects, and growth/equal/shrink preservation. Feed
actual captured runtime payloads to GNU Readline through a PTY, with a left
sentinel and cursor marker; include completion followed by a full-token edit
and an intentional over-delete negative control. This is production-executor
plus real-consumer proof, not a full IBus/Kitty or cold Space E2E claim. The
frozen private IBus driver stays byte-for-byte unchanged. The new Readline
helper is test-only and shares no runtime policy or mutation authority.
