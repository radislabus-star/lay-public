# TD-125 — Preserve the preceding word and separator during autocorrection

Priority: P0 (reproduced terminal deletion beyond the authorized edit plan)
Status: DONE / INSTALLED / FULL RELEASE GATE AND FOUR-CLIENT MATRIX PASS
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
7. [x] Run remote affected/full release checks and the actual-client matrix;
   obtain independent review with score 1–10 and record remaining limitations
   without weakening the gate.
8. [ ] Commit/push task-only when repository integration is explicitly requested.

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

## Legacy browser replacement continuation — 2026-09-20

Status: **IN_PROGRESS / PHYSICAL FAILURE REPRODUCED / FORWARDED-KEY CANDIDATE
REJECTED**. The installed runtime was changed transiently for the approved
physical experiment and restored byte-for-byte after the first failing required
client. Current runtime authority equals the pre-experiment state.
Base: `1f8416c9` (`release/public-1.0.73`, equal to `public/main`). The user
physically observed `вод -> ввод` become `водввод` in both Chrome and Firefox;
the same correction is exact in Kitty. This is direct product evidence for the
browser class, while the exact browser-side callback sequence remains
unrecorded.

### First unresolved mechanism and baseline

The accepted edit and logical state are already correct. On the legacy
SurroundingText output, `replace_committed_tail_with_effect_progress` emits two
independent signals in order:

```text
DeleteSurroundingText(-old_chars, old_chars)
CommitText(replacement)
```

`DeleteSurroundingText` has no application result; a successful signal call
only proves dispatch. The exact final snapshot is checked after both signals.
The observed `водввод` is therefore explained by the first effect being refused
or ignored while the second effect is applied. It is not explained by candidate
generation, ranking, edit-plan validation or internal tail arithmetic. Kitty
selects the separate `terminal_erase_commit` transport and does not exercise
this pair.

The smallest controlled RED will mark `TestEngineOutput` as legacy, run the
existing exact SurroundingText authority with a surrounding prefix, and require
one forwarded Backspace press/release pair per authorized deleted scalar,
followed by one `CommitText`. On the unchanged source it must instead observe
`DeleteSurroundingText` and fail. This is an executor regression, not proof that
a real browser applied the forwarded events.

### Consequence analysis before production code

Facts:

- The exact suffix, no-selection, path and epoch checks run before output, and
  `authorize_backend_edit` remains the sole edit-plan authority.
- Legacy `ForwardKeyEvent` already emits press/release pairs for cursor steps.
  Atomic output cannot represent forwarded keys and already owns a sealed
  delete-plus-commit frame. Terminal output owns one DEL-prefixed commit.
- The old legacy delete call cannot distinguish applied deletion from client
  refusal. Waiting for the later final snapshot can detect the damage but cannot
  retract the already-applied commit.

Hypothesis to verify physically: Chrome and Firefox apply IBus-forwarded native
Backspace events to the focused editable field before the following IBus
`CommitText`, so the authorized suffix is removed once and the result is
`prefix + replacement`. No GTK-wide or browser-name claim is made before the
owned-client matrix runs.

Options (engineering judgment, not measured product scores):

1. **Selected:** within the existing legacy ExactSurroundingText executor,
   replace the unreceipted `DeleteSurroundingText` effect with bounded
   `ForwardKeyEvent(BackSpace)` press/release pairs, then keep the single
   `CommitText`. The surrounding snapshot still supplies authority and the
   later exact postcondition still supplies outcome evidence. This uses the
   client's native deletion semantics without a browser name, daemon request,
   uinput device, layout mutation, retry or second mutation owner.
2. **Rejected:** dispatch `DeleteSurroundingText`, store a pending insertion,
   and commit only after a later `SetSurroundingText` reports deletion. This
   avoids the observed append on refusal, but adds a cross-callback queue,
   timeout/focus cleanup, a second mutation point and a visible partial-edit
   interval. Some refusing clients never publish the required intermediate
   snapshot, so the original Space can be consumed without its boundary.
3. **Rejected:** retain every typed token as active preedit and commit only the
   final correction. It provides a single commit, but changes ordinary typing,
   candidate visibility, reset ownership and latency for all clients to repair
   one demonstrated transport defect.
4. **Rejected:** disable legacy browser autocorrection. It prevents duplication
   by removing the requested function and leaves the broken executor contract
   unresolved.

Consequences and invariants:

- **Candidate/lattice, ranking and false authority:** unchanged. No source ID,
  token text, browser name or package score enters the branch. The same selected
  action, SafetyGate result and authorized delete count feed output. A stale,
  selected, mismatched or unsupported target still produces zero effects.
- **Latency and tail behavior:** one legacy delete signal becomes two D-Bus
  signals per scalar. Work is `O(authorized_plan.backspaces)` and bounded by the
  existing committed-tail/edit-plan limits. This may increase Space-tail
  latency, so focused checks must report the exact event sequence and physical
  browser acceptance must include growing, equal and shrinking replacements.
  No sleep, polling, retry or deadline increase is admitted.
- **CPU, RSS and allocation:** no model work, heap-backed queue, worker or cache
  is added. CPU and bus traffic grow linearly with the already-bounded deletion
  count; RSS should remain unchanged apart from existing call futures. These are
  estimates until remote checks and physical timing evidence.
- **Caches, packages and learning:** cache keys, package/delta loading, model
  identity, candidate generation and feedback payloads are unchanged. Existing
  outcome feedback still waits for the exact visible postcondition; dispatch is
  not promoted to learned success.
- **Concurrency and stale results:** authority is still consumed before the
  first effect. A focus loss or output error after any forwarded Backspace is an
  indeterminate partial edit, with no retry or fallback. The change does not
  make the multi-signal route atomic; it removes the demonstrated
  delete-refusal mode by asking the focused client to perform its native edit.
- **Compatibility:** atomic output retains typed `DeleteSurroundingText` plus
  `CommitText`; terminal output retains the single DEL-prefixed commit. Legacy
  cursor movement continues through the same forward-key primitive. The new
  deletion branch is selected by transport identity plus an already-authorized
  nonzero delete, never by application identity. GTK/native editors remain an
  explicit physical regression denominator because forwarded Backspace may
  differ from their accepted surrounding-delete behavior.
- **Maintenance, owner count and removal:** this adds one transport-specific
  emission branch, but no authority owner, detector, source of truth or fallback.
  Remove it when all supported GUI clients use an atomic frame with an applied
  delete receipt. Rollback is the legacy emission branch plus its test adapter;
  the regression remains useful to document the refusal boundary.

What can get worse: long replacements can emit more bus traffic; a client can
handle forwarded Backspace differently from direct keyboard Backspace; focus
loss mid-sequence can leave a shorter partial edit; GTK clients that currently
accept `DeleteSurroundingText` may regress. Future packages and online updates
do not change this transport conclusion because output consumes the sealed plan,
but a future planner with larger authorized extents increases the linear signal
cost. None of those risks permits weakening target identity, selection refusal,
SafetyGate, verifier authority or the exact postcondition.

### Proof denominators and promotion gates

1. Controlled old-source RED: one legacy exact replacement must fail because it
   emits surrounding-delete instead of forwarded Backspace.
2. Focused remote GREEN: the same test must prove zero surrounding deletes,
   exactly `2 * old_chars` ordered key events, one replacement commit, exact
   internal tail and an armed exact final postcondition. Atomic and terminal
   controls must retain their existing distinct effects. Stale/selection
   controls must retain zero effects.
3. Automatic remote affected check, architecture refresh and mandatory
   architecture check must pass on the same source identity. Focused PASS is not
   release acceptance.
4. Independent review must report findings first and score at least 8/10 under
   the existing two-pass limit.
5. Installation and owned physical Chrome, Firefox, GTK-native and Kitty tests
   require separate user approval because they change the active IBus runtime.
   Each GUI client must pass growing/equal/shrinking corrections with exact
   prefix, separator, final text, cursor and one replacement. Kitty must retain
   its terminal result. Until then browser quality is `UNKNOWN` for the patched
   bytes and this task cannot be marked `DONE`.

### Controlled RED — 2026-09-20

Remote focused command:
`python3 scripts/dev-check.py check --target bin:lay-ibus-engine --compact`.
The first attempt stopped at formatting and is not test evidence. After applying
only the formatter-prescribed layout, run `run-cvv9c1di` selected 565 tests:
564 passed and the single new regression
`window_interaction_legacy_exact_replace_forwards_bounded_backspaces_before_commit`
failed at `output.surrounding_deletes.is_empty()`. The unchanged executor
therefore produced the forbidden surrounding-delete effect before any
production fix. Exact receipt:
`/home/ubu/.cache/lay/development/run-cvv9c1di/RESULT.json`.

Tested: the production executor with the test output adapter marked as legacy,
an exact unselected surrounding snapshot, an authorized three-scalar replacement
and full-prefix logical state. Not tested: D-Bus delivery, browser application,
GTK compatibility, installed bytes, latency or physical input. Runtime authority
remains unchanged. The next bounded change is the selected legacy emission
branch; rollback remains that branch plus the test adapter fields.

### Focused source GREEN — 2026-09-20

The selected branch now preserves the existing `ExactSurroundingText` authority
and changes only legacy effect emission. A nonzero authorized delete emits one
`ForwardKeyEvent(BackSpace)` press/release pair per scalar before the existing
commit. Atomic output retains `DeleteSurroundingText`; terminal output remains a
single DEL-prefixed commit. Cursor steps reuse the same generic forward-key
helper. The legacy exact diagnostic route is now
`legacy_forwarded_backspace_commit`.

Remote focused run `run-9iwluvlw` passed 565/565 tests in 20.1 seconds. The new
regression proves six ordered key effects then one commit for the three-scalar
Cyrillic source, zero surrounding deletes, exact logical tail
`просто ввод `, and exact expected external snapshot/cursor. The existing real
D-Bus adapter test now proves the same six `ForwardKeyEvent` members followed by
`CommitText`; its postcondition receipt remains separate. Exact receipt:
`/home/ubu/.cache/lay/development/run-9iwluvlw/RESULT.json`.

Measured scope: source formatting and the focused `lay-ibus-engine` target only.
Not yet tested: automatic affected closure, architecture gate, installation,
client application of forwarded keys, GTK/Chrome/Firefox/Kitty physical behavior,
or latency distribution. Production source behavior changed, while authority
ownership and the installed runtime remain unchanged. Browser answer quality
remains `UNKNOWN`.

### Automatic affected GREEN and architecture state — 2026-09-20

The canonical manifest was regenerated on the remote worker and now adds the
three legacy replacement regressions. It contains 2,894 tests: 2,832
correctness, 36 package, 11 performance and 15 ignored. The zero-failure ledger
remains empty; only its manifest binding changed to SHA-256
`7457da79adb2474a03439f14cd6788849aef957d2714a1dc077069bc7bd768a9`.

The first automatic broad attempt correctly refused manifest drift before the
canonical lanes. After manifest generation, the next attempt executed every
selected target without a test failure, then refused the stale zero-failure
binding. These are fail-closed contract results, not product failures. With the
empty ledger rebound, automatic affected run `run-rkfo3u_y` passed all 2,866
selected correctness/package cases in 331.2 seconds, with zero known semantic
or infrastructure failures. Exact receipt:
`/home/ubu/.cache/lay/development/run-rkfo3u_y/RESULT.json`; archive SHA-256:
`25c09ae8a3215e31328ab983a06ff81f267cedf188703cb18ef75a9539d7b76e`.

The canonical remote architecture wrapper required a cold AST rebuild because
an ignored incremental `graphify-out/cache` snapshot dropped existing call
edges and produced the known three unrelated `edit-plan-verifier` WATCH rows.
A fresh snapshot excluding only that disposable cache re-extracted 1,064 files,
built 22,882 nodes and 60,838 edges before pruning, bound 702 Rust sources and
passed all 11 architecture checks. Exact log:
`/home/ubu/.cache/lay/development/graph-cold-uaxxfo8q/update-architecture-graph.log`,
SHA-256
`dd04db8176b66fc7a4629568b93580652c6c232eb5a058a98d945ba1e5f66190`.

Tested: formatting, lane self-tests, the complete selected correctness/package
denominator, source binding and the architecture gate on one source snapshot.
Not tested: installation, browser application of `ForwardKeyEvent`, GTK-native
compatibility, Kitty regression, physical cursor/focus behavior, or a release
gate. Installed 1.0.73 bytes and runtime authority remain unchanged. The next
promotion step is the independent review followed by separately approved
installation and the fixed physical client matrix; task status remains
`IN_PROGRESS` and patched browser quality remains `UNKNOWN`.

### Independent review pass 1 and grouped repair — 2026-09-20

Fresh-context review pass 1 found no correctness defect and returned **PASS
8/10**, with zero high or medium findings and two low findings. The first asked
for explicit coverage of cursor-moving legacy plans and forward-key partial
failure. The second found that two documented durations used outer wall time
instead of the cited `RESULT.json` elapsed time.

The grouped repair changed only test instrumentation, tests and evidence text:

- an injected failure can now target an exact forwarded-key index;
- the boundary-elided auto-undo plan proves the complete ordered sequence
  `Left press/release -> six Backspace press/releases -> CommitText -> Right
  press/release`, with no `DeleteSurroundingText` and exact logical tail;
- a failure on the first Backspace release proves
  `DeleteDispatched` / `LocalIndeterminatePartial`, no commit, no postcondition
  and unchanged internal tail;
- the focused and broad evidence durations now use the authoritative receipt
  fields.

Remote focused run `run-v6sn184o` passed 567/567 in 19.9 seconds. Exact receipt:
`/home/ubu/.cache/lay/development/run-v6sn184o/RESULT.json`. Runtime production
code did not change in this repair. The refreshed 2,894-test manifest makes the
earlier 2,866-case broad PASS a pre-repair checkpoint; a fresh automatic broad
run and final review pass 2 are required on the repaired snapshot.

The fresh automatic run `run-r68l462x` then passed all 2,868 selected
correctness/package cases from that 2,894-test manifest in 359.0 seconds. It
reported zero known semantic, infrastructure or test failures. Exact receipt:
`/home/ubu/.cache/lay/development/run-r68l462x/RESULT.json`, SHA-256
`ae6d69844c583880d7c3a291ea1b8962de5e358eb7d3e3f44fbbe187fb75b84f`;
source archive SHA-256
`a2bb1f3424cab2dc193fc23cbf81d47da64ae2acdd67cc987ffb7ee89195a5d2`.

The repaired snapshot's cold architecture refresh re-extracted the graph and
passed all 11 checks with 22,886 nodes, 60,846 edges before pruning and 702 Rust
bindings. Exact log:
`/home/ubu/.cache/lay/development/graph-cold-4v1y9j29/update-architecture-graph.log`,
SHA-256
`0bea0e917832403edc0995c3c997afe72f26ddc87dcc858bb3b47bc3c8d8ff24`.
This accepts the grouped test/evidence repair at development scope. Installation,
release artifacts and physical-client behavior remain untested.

### Final independent review — 2026-09-20

Review pass 2 found no remaining issue and returned **PASS 10/10**, with zero
high, medium or low findings. It confirmed the ordered cursor-moving sequence,
the injected forward-key failure semantics, receipt durations, manifest and
zero-failure binding, graph/source identities, and unchanged Atomic and
TerminalErase routes. The pass was read-only. This exhausts the two-pass review
track; no third review is permitted. Physical-client quality remains `UNKNOWN`
until separately approved installation and client tests.

### Source-bound full release gate and staged candidate — 2026-09-20

The final reviewed source was sealed as a tracked-file archive with SHA-256
`6d97ae28d7a9e984ccae669e3b260e34d792e5e2f48d3b9a90faf84ff36b538a`.
The first invocation stopped before the canonical product lanes because the
sealed archive deliberately omitted `.git`, while the hermetic lane discovers
tracked inputs through `git ls-files`. This is a packaging/setup refusal, not a
product test result. The same archive was placed in a fresh Git index without
changing any source file and the complete gate was restarted from the
beginning.

Under one remote `dedicated-20cpu` lease with Cargo jobs 20, Rust test threads
1 and the guarded shared target, `scripts/check-lay-full.sh` passed. The run
included formatting, the architecture gate and all 11 architecture checks,
test-lane self-tests and manifest validation, all 2,868 selected correctness
and package cases with zero known semantic or infrastructure failures, both
lint scopes with zero non-baseline diagnostics, JavaScript/Python/shell syntax,
the Firefox compatibility adapter regressions, CLI explain smoke, `git diff
--check`, and `cargo build --release --bins --features research-tools`. The
canonical lane took 327.0 seconds and the all-bin release build took 3 minutes
34 seconds. The opt-in performance lane and n-gram cache check were not run by
this gate.

The immutable local evidence root is
`/home/ubu/.cache/lay/development/td125-release-20260920T113942Z/`; the remote
root is
`/home/e/projects/lay-development-runner/td125-release-20260920T113942Z/`.
The successful full log is `check-lay-full.log`, SHA-256
`c9e64732f4e5961d8f65e758d21de925a0f71a73a9605cab333785b2d583058e`;
the pre-lane setup refusal is retained separately as
`check-lay-full-attempt1-setup-failure.log`.

The resulting staged `lay-ibus-engine` is version 1.0.73, 7,899,752 bytes and
has SHA-256
`4a8a450ab190ccbd0de1cdbbf3afe2678391aa5655193886e7fc74551693adf9`.
Its local path is
`/home/ubu/.cache/lay/development/td125-release-20260920T113942Z/lay-ibus-engine`.
It has not been installed or executed. The active IBus engine, input sources
and runtime authority remain unchanged. D-Bus/browser application, physical
Chrome, Firefox, GTK-native and Kitty behavior, focus-loss behavior and client
latency remain untested for these staged bytes, so browser answer quality stays
`UNKNOWN` and the task remains `IN_PROGRESS`.

### Physical rejection and exact rollback — 2026-09-20

The separately approved physical matrix disproved the selected transport before
promotion. Candidate SHA-256
`4a8a450ab190ccbd0de1cdbbf3afe2678391aa5655193886e7fc74551693adf9`
was installed temporarily and only the Lay IME engine was restarted. The global
`ibus-daemon` PID remained `4062416`. In the native GTK capture, the exact input
`просто кторое` followed by Space produced `просто ктороекоторое ` at cursor
position 21 instead of replacing the six-scalar old token.

The trace proves that the client had published the exact unselected
SurroundingText and that the correction was authorized. The legacy executor then
reported `legacy_forwarded_backspace_commit`, dispatched six Backspace
press/release pairs and one commit in 88 microseconds, but the client retained the
old token. Thus forwarded-key dispatch is not evidence that the focused client
applied deletion. The first required physical client failed, so the candidate was
rejected without continuing the matrix.

The exact pre-experiment installed binary was restored immediately. Its SHA-256
is `14afbc69b48e737f5e4268ec98a21bbe3d304aceeeb8fd54ba81b48636f07607`;
the active engine executable has the same hash, the global `ibus-daemon` PID is
still `4062416`, and the selected input source remains `lay-ime-ru`. The immutable
evidence root is
`/home/ubu/.cache/lay/development/td125-live-20260920T133134Z/`; its rollback
receipt is `rollback-receipt.txt`. The captured failing trace is
`failed-candidate-ibus-trace.jsonl`, SHA-256
`577e986f3ccef2e832fbdc9058a32384f69495abed7876fdacc6f35f7c9342fe`.

Also tested: a separate Chrome X11 test profile. That client did not publish
SurroundingText, so the existing capability gate refused mutation with
`no_proven_delete_backend`; those runs prove refusal only and do not exercise
either deletion transport. Not tested: the user's original Wayland Chrome and
Firefox fields, baseline `DeleteSurroundingText` behavior in the same native GTK
capture, Kitty regression behavior, or a replacement transport. Browser answer
quality therefore remains `UNKNOWN`.

Verdict scope: the source and D-Bus tests accurately proved emitted signal order,
but they did not prove client application. The forwarded-Backspace production
change is rejected and must not be installed again. The next experiment must
first compare the restored baseline surrounding-delete route in the same GTK
client, then select a transport whose visible result is owned or acknowledged by
the client. Candidate generation, rank, SafetyGate, edit-plan validation and
verifier authority remain outside this failure.

### Restored baseline GTK control — 2026-09-20

The required same-client control passed on the restored binary. In a fresh native
GTK field, the same input `просто кторое` followed by Space produced exactly
`просто которое ` with cursor position 15. The trace records one authorized
correction, `surrounding_text_immediate_delete_commit`, six deleted scalars, one
commit payload `которое `, 64 microseconds of executor time, and the exact
15-character postcondition.

Evidence is under the same immutable experiment root in `baseline-gtk/`:
`result.out` has SHA-256
`701d96ce05a063a687e0ec2ac0517ce946b728f92067918989ae1ad0680b1344`;
the four selected causal trace records are `trace-selected.jsonl`, SHA-256
`cde83036a0531130c3fb871655fb1e18f99e9d6298f88fae028aeab842fa11a8`.
No binary, process configuration, input source or runtime authority changed for
this control.

This narrows the mechanism: the baseline delete-plus-commit route is physically
accepted by native GTK, while forwarded Backspace is not. The user's Chrome and
Firefox symptom remains a downstream client refusal of the delete effect, not a
GTK-wide IBus failure. A browser-safe correction must therefore avoid editing an
already committed token on the unacknowledged legacy route; it cannot replace the
working GTK transport globally.

### Legacy preedit word owner preflight — 2026-09-20

Status: **READY_FOR_CAUSAL_RED**. Runtime authority changed: false. The restored
installed binary remains active and no candidate from this section has been
built or installed.

The failed physical experiment reopens the earlier preedit option. Source
inspection identifies the common upstream mechanism: with an empty composition,
`process_pressed_key()` commits every managed printable character immediately.
Space must then edit application-owned text. The existing active-composition
mutator already owns live preedit, authorized correction commits, Backspace,
cursor movement, Tab acceptance, Enter and reset cleanup. The existing
latest-only Space worker already calculates the same L1.1 -> L2 -> L3 -> L4 ->
DecisionCore result before Space and binds it to the exact path, focus, epoch,
tail, layout, configuration and material generation.

Options after the physical rejection, scored as engineering judgments rather
than measured quality:

1. **9/10, selected for proof:** for a legacy `ManagedCommit` word whose client
   advertises IBus preedit support but not `SurroundingText`, start active
   composition only at an empty token boundary. Publish each alphabetic character
   as preedit, schedule the existing exact prefetch, and on Space either commit
   its authorized replacement plus one separator or commit the original
   composition plus one separator. Application-owned committed text is never
   deleted on this ordinary word path.
2. **7/10, deferred system route:** deploy the existing
   `ProcessKeyEventAtomicV1` contract through patched IBus, GNOME Shell and
   Mutter. It can own delete plus commit, but it changes system packages and the
   desktop session and does not cover unpatched direct GTK/X11 clients. It is not
   a bounded Lay 1.0.73 repair.
3. **4/10:** keep per-character commits and disable automatic replacement when
   the legacy client lacks an applied-delete receipt. This prevents duplication
   but removes the requested browser autocorrection.
4. **Rejected:** synthetic Backspace through forwarded keys, daemon replay or
   uinput. Forwarded keys already failed physically; the other variants add a
   second mutation owner and cannot produce an acknowledged same-event edit.

The admitted route is structural, not browser-named:

```text
legacy ProcessKeyEvent
+ WordInputMode::ManagedCommit
+ IBus PREEDIT_TEXT capability
+ no IBus SURROUNDING_TEXT capability
+ empty current token before the first alphabetic scalar
-> active composition owns the complete new token
-> every scalar refreshes visible preedit and exact prefetch identity
-> Space consumes only a ready identity-equal AuthorizedEdit
-> clear presentation preedit + one CommitText(corrected-or-original + " ")
```

Terminal purpose and narrow native-terminal profiles retain
`TerminalPassthrough`. Atomic output retains its existing proposal semantics.
Clients without the preedit capability and clients with the already proven
`SurroundingText` delete backend retain current managed commits and gain no new
authority. A token that was already open before this engine acquired it, or that
began with a committed nonalphabetic scalar, stays on the existing committed-tail
path for its whole lifetime; active composition must never claim a suffix of
application-owned text.

Consequences and invariants before production code:

- **Authority and quality:** the prefetch lease, selected action, SafetyGate,
  backend authorization and exact identity checks remain mandatory. The active
  composition mutator changes only how an already-authorized target is rendered.
  No token, application name, source ID or fixture becomes a runtime condition.
- **Latency:** Space must not call DecisionCore synchronously. A missing,
  pending, stale or refused lease commits the original preedit plus Space within
  the existing bounded wait. Printable keys schedule the same latest-only worker;
  no retry, sleep or larger budget is admitted.
- **Visible behavior:** ordinary managed words are underlined/live preedit until
  their boundary. Backspace and Left/Right edit that owned composition. Tab keeps
  the existing explicit candidate route. Enter commits the composition and lets
  Enter pass. A nonalphabetic printable finalizes the current composition with
  that scalar in one commit and does not start a suffix composition inside the
  same token.
- **Learning and undo:** dispatch alone remains censored. Where SurroundingText
  exists, exact postcondition observation still owns positive feedback and layout
  synchronization. A successful correction retains the existing bounded
  automatic-undo record. Caps9 clients without SurroundingText receive no false
  positive learning claim.
- **State and concurrency:** focus loss, Reset, capability/owner change and
  identity mismatch retain current invalidation. A client capability change may
  not turn an already committed token suffix into preedit. Package reload and
  material generation checks remain in the lease.
- **Resources and maintenance:** no model, worker, queue or cache is added. The
  composition string already exists; the change replaces per-character commit
  signals with preedit updates. Rollback is the boundary admission, active Space
  executor and capability bit; no stored-data migration is required.

The causal RED must drive the real `process_pressed_key()` route through a legacy
test output with caps9. On unchanged source, `ghbdtn` produces six committed
characters and no owned composition. The required RED expects zero commits,
complete preedit ownership and a ready exact lease that makes Space emit no
delete or forwarded key and exactly one `привет ` commit. Controls must prove:
caps without `PREEDIT_TEXT` and caps with `SurroundingText` keep managed
commits; atomic output keeps its current route; terminal purpose stays native
passthrough; punctuation, Backspace, arrows, Tab, Enter, focus/reset and
not-ready Space preserve their existing semantics.
After focused GREEN, run the automatic affected closure, architecture refresh,
independent review, full release gate and then a fresh separately recorded
physical GTK/Chrome/Firefox/Kitty matrix. Source tests alone do not establish
browser quality.

### Legacy preedit causal RED — 2026-09-20

The controlled test adapter was marked as legacy and given caps9
(`PREEDIT_TEXT | FOCUS`) without SurroundingText. It drove the real
`process_pressed_key()` callback for `ghbdtn`. On unchanged production source,
the test failed at the first required ownership assertion: the client had already
received six commits `["g", "h", "b", "d", "t", "n"]`, while the required
unfinished-token contract allows zero commits and owns `ghbdtn` as preedit.

Exact command:
`scripts/cargo-guard.sh test --bin lay-ibus-engine
td125_legacy_preedit_tests::td125_legacy_preedit_word_commits_ready_space_correction_without_delete
-- --exact --nocapture`. Result: one selected failure, 567 filtered out. Log:
`/home/ubu/.cache/lay/development/td125-preedit-20260920T140000Z/causal-red.log`,
SHA-256
`fd3ef288b1cd98f84f8f4a9c8fc9fd857e11dfb47a82fc35aa8db462b5b3a241`.

Tested: actual printable callback routing and emitted commit payloads on the
unchanged managed path. Not yet reached by this RED: the prepared Space lease or
final commit-only correction, because the ownership precondition failed first.
No production source, installed byte or runtime authority changed.

### Legacy preedit implementation and affected closure — 2026-09-20

Status: **SOURCE GREEN / PHYSICAL QUALITY UNKNOWN**. Runtime authority changed:
false. The restored pre-experiment binary remains installed; this implementation
has not yet been installed or executed by a desktop client.

The selected mechanism is implemented without an application-name branch. A
legacy `ManagedCommit` client may own a new alphabetic word as IBus preedit only
when it advertises `PREEDIT_TEXT`, does not advertise `SURROUNDING_TEXT`, and the
current token is empty. Each printable letter updates that owned preedit and
schedules the existing latest-only Space lease. Space reuses the common lease
admission checks for exact identity, material generation, certificate, verified
action and one trailing separator, then obtains the existing IME backend
authorization and emits one `CommitText` payload. A missing, pending, stale or
refused lease commits the original preedit plus one Space. No committed client
text is deleted and no forwarded key is emitted on this route.

Punctuation finalizes the owned token once. Backspace and Left/Right edit the
owned composition; Tab retains explicit candidate acceptance; Enter commits the
composition and passes Enter through. Focus/reset, sensitive content, owner
change and capability loss retire ownership. Atomic output, terminal passthrough,
clients without preedit, and the caps41 `SURROUNDING_TEXT` route retain their
existing owners. The shared `composition_commit.rs` mutator was not changed; its
SHA-256 remains
`344a525b388879af088ff8afaa5ec7bf8ddc9fd96c83c7ce6daa8281fa28efd7`.

The affected closure exposed and rejected two over-broad intermediate choices.
Cancelling all input-frame background work on every preedit-capability change
interfered with unrelated pending precognition work. After that was narrowed to
the Space lease, putting `PREEDIT_TEXT` into the common input-frame capability
fingerprint still changed old Firefox admission/replay identities. Full-target
failures moved between old tests while every named failure passed in isolation.
The accepted boundary invalidates only the Space lease on capability change,
cancels precognition only when an actually owned legacy preedit is lost, and
keeps the common fingerprint unchanged. The next common run then passed all 577
IBus tests immediately after the 267-test daemon target. These rejected
intermediates were never installed.

Measured source evidence:

- the causal GREEN module has 13 passing tests and 567 filtered tests, covering
  exact ready correction, no-preedit and caps41 controls, capability gain/loss,
  Atomic and terminal controls, not-ready fallback, punctuation, Backspace and
  cursor movement, Tab, Enter, soft reset and focus reset;
- the canonical focused `bin:lay-ibus-engine` target passed 577/577;
- `scripts/check-lay-changed.sh` passed with 2,878 selected cases: 2,842
  correctness and 36 package, zero failures and an empty known-failure ledger;
  its large targets included `lay-daemon` 267/267, `lay-ibus-engine` 577/577 and
  `lib:lay` 1,795/1,795;
- the complete manifest contains 2,904 tests: 2,842 correctness, 36 package, 11
  performance and 15 ignored. Its SHA-256 is
  `03e9bb54626c99f043c29010dd10dc29cf92ae172fcece0f50de4cad37659bf1`;
  the zero-failure binding SHA-256 is
  `e10a4b858668b53824b2716ce31c3c8d1207a4ac8f991920143d9233da4908`.

Evidence root:
`/home/ubu/.cache/lay/development/td125-preedit-green-20260920T142527Z/`.
The passing affected-closure log is `check-lay-changed-v3.log`, SHA-256
`6dc1c02bef9e5b3419a7f1fdc9e9b4cad0851c44f3322dbc8cfaf806f9f29529`.
The final focused module log is `focused-td125-after-identity-fix.log`, SHA-256
`80df7521e9876a4639bcbad5e12a52fe14327e50a999580b924df7972c999406`.
The canonical focused-target summary is under `focused-ibus-results-v4/`, with
summary SHA-256
`932e25b1897c6847f3b5223802abeab0114f976cf2b9505ca037292bbef55c2b`.

Not tested by this section: the opt-in performance lane, a full release build,
installation, D-Bus execution by the candidate binary, visible preedit styling,
Space latency in a real client, or physical GTK, Chrome, Firefox and Kitty
behavior. Source signal assertions prove emitted effects, not client
application. Browser answer quality therefore remains `UNKNOWN` pending the
separate release and physical matrix.

### Legacy preedit architecture refresh — 2026-09-20

The canonical architecture wrapper now passes all 11 checks, including the
three `edit-plan-verifier` capability edges. The first local refresh reused an
ignored AST cache transferred from a remote checkout. That cache preserved
checkout-prefixed node IDs and omitted the three `AuthorizedEdit`
`parameter_type` edges, so the wrapper correctly refused to replace the last
valid receipt. No gate or safety rule was changed. After discarding only that
disposable cache, the unchanged wrapper re-extracted all 1,065 inputs. After the
lint-only test cleanup it produced 22,918 nodes and 60,914 links before
self-source pruning, bound 703 Rust source files, and published a final graph
with 22,900 nodes and 60,900 links.

Measured artifacts:

- `graphify-out/graph.json`, SHA-256
  `385bb9b0209677cc3f3956369cadc08e0025d2982657835825447dfce3947aa7`;
- `graphify-out/source_graph_binding.json`, SHA-256
  `95e1fe1da799b156dc258399cd472826a0ac4ecde094b4e17c898fad6598b28b`;
- `src/generated/architecture_graph_receipt.json`, verdict `PASS`, SHA-256
  `f2e2d22183bc2dcf3a8738a298a9bbb1f624886151ea5f1119706f7e083543ee`;
- exact final check log
  `/home/ubu/.cache/lay/development/td125-preedit-green-20260920T142527Z/architecture-final-v2.log`,
  SHA-256
  `56ded340dc67b281b0f95778d877937a84c428c61f6a98159725db40613c18ec`.

Tested here: graph source freshness, source binding, receipt publication and all
architecture checks against the current TD-125 source. Not tested here: a full
release build, installation, D-Bus execution or any physical client. Runtime
authority remains unchanged and browser answer quality remains `UNKNOWN`.

### Frozen release, physical Firefox failure and rollback — 2026-09-20

The reviewed caps9 preedit source passed a fresh immutable release transaction
at
`/home/ubu/.cache/lay/development/td125-preedit-release-20260920T152913Z/`.
Its full gate passed the architecture checks, all 2,878 selected correctness
and package tests, lint, the Firefox adapter checks, release build and final
diff check. The full log SHA-256 is
`b3cd34d8bb5994c6a1cab2af7333edf1c8ae300c993d81d8f829c2049be67819`.
The exact candidate `lay-ibus-engine` is 7,909,352 bytes, reports 1.0.73 and has
SHA-256
`38f42f3c2c542fe0c99f2a7a8f24f5d9b7be3c0b5ca114ab2d72866fae8fd07e`.

The candidate was installed only for the approved client matrix. Native GTK
produced exact `просто которое ` at cursor 15. The existing Wayland Chrome
process produced the same exact value while retaining its original PID and
start identity. The existing Wayland Firefox process retained its original PID
and start identity but produced unchanged `просто кторое ` instead of the
required correction. GTK output SHA-256 is
`701d96ce05a063a687e0ec2ac0517ce946b728f92067918989ae1ad0680b1344`;
Chrome receipt SHA-256 is
`91dcced3007b1432b9381d911af00cdd1a5cfe4a0a99471c7ce99d67abbb1086`;
Firefox receipt SHA-256 is
`4e91eb3f4e997ee55d61506d5bd1d9501816386b2c5b735f3570a1b1a522a635`.
The matrix stopped at that first product failure, so Kitty was not run.

The installed engine was immediately restored byte-for-byte to SHA-256
`14afbc69b48e737f5e4268ec98a21bbe3d304aceeeb8fd54ba81b48636f07607`.
The global IBus daemon retained PID 4062416 and the Lay daemon retained PID
3949795. The selected source remained `lay-ime-ru`. Rollback receipt
`rollback-after-firefox.json` has SHA-256
`fe1ef19acf60c73b724c1986456017decc2007b0ae3e3675594f7f36279233e8`.
Runtime authority therefore equals the pre-experiment runtime; the candidate
has no installation authority.

### Firefox capability discriminator and revised owner boundary — 2026-09-20

The original Firefox process predates the installed compatibility launcher and
does not map the Lay adapter, so a fresh owned profile was required to separate
that environment fact from product behavior. The accepted launcher and adapter
were then exercised in an isolated native Wayland Firefox profile with the
candidate engine. Environment and mapping checks proved direct GTK IBus,
asynchronous mode, the exact release adapter, the Snap compatibility preload
and preservation of the original Firefox and global IBus identities. The fresh
client still returned unchanged `просто кторое `. Its receipt SHA-256 is
`d67c6bc78e1c4fe1e2974a6bf0fcdb0eb652698304b2e3a733ac3db82d3dac60`;
engine trace SHA-256 is
`25fa01dab477bc3419737b6e829e7d1470b475971e85929e5371db7629752eb9`.

The first owner mismatch is now measured. Firefox advertises capabilities 41:
`PREEDIT_TEXT | FOCUS | SURROUNDING_TEXT`. The caps9-only guard therefore
rejects legacy word-preedit ownership, commits every scalar to the client and
retains the unacknowledged surrounding delete-plus-commit route. This is not an
application-name inference. The client trace records 12
`printable_managed_commit` events, zero `printable_legacy_preedit` events and
the `managed_fallback_commit` Space result.

A separate causal discriminator added a pre-Space delay and copied the exact
current product configuration into the isolated case. It is diagnosis only and
is not acceptance evidence. The correction projection still produced no
authorized lease in that fresh profile, so the run did not exercise or prove
Firefox's application of `DeleteSurroundingText`; it does prove that the
candidate cannot move a caps41 Firefox token onto its commit-only safe route.
That discriminator receipt SHA-256 is
`16476f2f23820873fd6d006548939dfb3c978a238f1822d2c8dc41e5eb2a239d`;
trace SHA-256 is
`b25b1a8fee032b5d0c44db652a506a78e3c1a5df7ab5a66508df00dd800aed9d`.

The first revised experiment removed the `SURROUNDING_TEXT` exclusion for every
legacy `ManagedCommit` client with preedit support. Its caps41 causal test passed,
but the complete IBus target failed 72 of 580 tests. The failures crossed
Firefox reset/replay, context transfer, atomic publication and preedit ranking;
this is a systemic regression, not stale expected output. Exact log:
`/home/ubu/.cache/lay/development/td125-preedit-caps41-green-20260920T163600Z/focused-ibus.log`,
SHA-256
`980f4409173a1ce03608390b5089111bf06aa1e500accc6dfb590c3dce9ddb52`.
The broad predicate was reverted immediately and was never built as a release
or installed.

The next bounded route uses an explicit client transport contract. The existing
Firefox compatibility adapter will add private capability bit `1 << 30` when it
forwards the client's real IBus capabilities. The engine interprets that bit as
`LAY_COMMIT_ONLY_PREEDIT`: this client requests whole-word preedit because its
legacy delete-plus-commit effect has no applied-delete receipt. Caps9 clients
retain the existing safe preedit route. An unmarked caps41 client retains the
existing managed-commit route, so native GTK and every old test fixture keep
their ownership. The branch contains no application name, executable identity,
word, source ID or timing classifier; the separately installed client adapter
declares the transport contract at the same capability boundary that already
declares preedit and SurroundingText support.

Before implementation, candidate generation, lease identity, model and package
loading, SafetyGate, verifier, Space wait budget, feedback rules and focus/epoch
invalidation remain unchanged. Gaining or losing the marker invalidates only
the existing Space lease; losing it discards an actively owned word by the same
capability-loss cleanup used for preedit loss. The marker is not admitted into
the common input-frame fingerprint, so it cannot perturb old Firefox transfer
identities. The adapter preserves the real capability setter and only ORs the
private bit; a missing underlying setter is a hard startup error. CPU, RSS and
latency costs are one integer OR per capability publication plus the already
measured preedit route. There is no retry, sleep, cache, worker, queue or second
mutation owner.

The causal RED changes the caps41 test to include the explicit marker and require
whole-word preedit plus one exact commit. On unchanged production it failed at
the first ownership assertion because the application already received six
per-character commits. Exact log:
`/home/ubu/.cache/lay/development/td125-preedit-caps41-red-20260920T163531Z/causal-red.log`,
SHA-256
`8028e75e16c8a52aa6b1a5aadeca82c62e0762b7cb3aa7dc44ceda29af120997`.
The same test must retain an unmarked caps41 negative control. Promotion requires
the focused and affected closures, adapter unit test, architecture refresh, a
new frozen release, then the complete GTK/Chrome/fresh-Firefox/Kitty matrix.

### Explicit transport marker implementation and Firefox proof — 2026-09-20

The marker implementation passed both adapter unit tests and all 13 focused
TD-125 engine tests. The marked caps41 fixture now proves three separate
properties in one lifecycle: an unmarked caps41 control retains managed commits,
the marked misspelled word commits one authorized correction without a delete,
and the next word after whitespace starts a new owned preedit. Capability gain
still cannot claim a suffix that the client already owns. Candidate generation,
lease identity, the verifier, `SafetyGate`, feedback, and the Space wait budget
were not changed.

The first native adapter attempt failed before input with
`missing original capability setter`. Firefox's GTK IBus module had loaded
`libibus-1.0.so.5` into a local dynamic-loader scope, so `RTLD_NEXT` could not
see the real setter. The accepted resolver retains `RTLD_NEXT`, then looks up the
setter through an `RTLD_NOLOAD` handle to that already-loaded library and keeps
the same hard failure if resolution is still impossible. The adapter was built
on the separate 20-core worker under the `dedicated-20cpu` resource profile.
Its source SHA-256 is
`9383f4db11b651d327ea0d7f276c63fc6ef445461579bc0dd21bb3fed9367380`;
the resulting library SHA-256 is
`924b59822e26f5ad3e6e0daf2a8f762b15e6905bb0b3683ce56bee55744a6f22`.
The compact remote build receipt is
`/home/ubu/.cache/lay/development/td125-marker-diagnostic-Q0Tlq38N/adapter-v2-remote-build.txt`,
SHA-256
`7d4a2d0415d8d3d9adb874ea9d5c60db7dbb462978580c44f203b2aba3dba075`.

The next owned Firefox run proved the transport marker but exposed a second
general boundary defect. The engine received caps `1073741865`, recorded
`commit_only_preedit_requested=true`, and moved the first word to
`printable_legacy_preedit`; after committing the trailing Space it returned to
per-character commits. `last_tail_token_text()` intentionally ignores trailing
whitespace, so it still named the preceding word and rejected new ownership.
The accepted predicate now starts only when the committed tail is empty or ends
in whitespace. It neither recognizes a literal word nor claims a nonempty
suffix. The diagnostic-only receipt SHA-256 is
`b0ed119a747f036ac4353ffd73ff799268f390d06b3e2843de45a3273d22e3c8`.

With both fixes, an isolated native Wayland Firefox profile mapped the exact
adapter, kept asynchronous IBus mode, and returned exact
`просто которое ` at cursor 15. The trace has marked caps, zero managed commits,
`printable_legacy_preedit` at the start of both words, and
`space_legacy_preedit_autocorrect` for the second Space. The candidate engine
SHA-256 is
`3bed0eb3803f6a37e40db0a1c1a02bcc5ab0a3dae5647d7dae4d156822647d71`.
The authoritative receipt is
`/home/ubu/.cache/lay/development/td125-marker-diagnostic-Q0Tlq38N/native-firefox-marker-route-run4/receipt.json`,
SHA-256
`930a1608aae40a80dcbfbac38493b1bc7d24a774ef35f1c03395077f2d465989`;
the engine trace SHA-256 is
`c530a085670e5632bfd60e32323764f55b9e24e0920c76c09429a11405a5b6cf`.

The complete IBus target after the boundary fix passed 578 of 580 tests. One
reset/provenance failure passed immediately in exact isolation. The remaining
`td121_pending_worker_fills_missing_material_before_exact_receipt` failure is a
one-second outer choreography timeout during a real L2 cache-miss computation;
its helper writes `surrounding_text_supported` directly and does not execute the
changed capability path. This is recorded as an unresolved test-infrastructure
failure, not a product PASS. Exact log:
`/home/ubu/.cache/lay/development/td125-marker-diagnostic-Q0Tlq38N/full-ibus-after-boundary.log`,
SHA-256
`3078cd890b3b213508e9cb036856b3f2d5b62905b0a2c925a0c423b13fee25e8`.

Tested by this section: adapter forwarding and resolution, marked and unmarked
caps41 ownership, capability transitions, consecutive-word ownership, exact
single-commit correction, and one fresh Firefox physical case. Not tested by
this section: the affected closure, architecture refresh, immutable full release
gate, GTK/Chrome/Kitty regression matrix, or final installation. The candidate
adapter and engine were rolled back byte-for-byte after every physical run. The
original Firefox and global IBus identities were preserved, the selected engine
remained `lay-ime-ru`, and runtime authority remains unchanged.

### Affected closure and clean-HEAD harness audit — 2026-09-20

The canonical changed-file lane ran all 2,878 correctness and package tests.
All 2,301 selected tests outside `lay-ibus-engine` passed. The IBus target ran
577 selected tests and reported four TD-121 reset/replay failures; the complete
direct IBus target later ran 580 tests and reported five failures from the same
asynchronous harness family. All 13 TD-125 marker and preedit tests pass. No
failed case selected the private capability bit or entered legacy word-preedit
ownership.

This audit did not classify those failures from timing alone. A detached clean
`HEAD` checkout, before any TD-125 source is present, reproduced them. In 20
one-process exact runs, the clean baseline prior-surface replay test passed 14
and failed 6 times at its pre-existing P2P signal/readout assertions. The
candidate-tree sample passed 9 and failed 11 times. A second Firefox reset
fixture on clean `HEAD` passed 18 and failed 2 times at the same initial Reset
assertion seen in the candidate run. These samples establish a pre-existing
test-harness instability; they do not turn a failed affected lane into a product
PASS and do not justify weakening the tests. No runtime source or test was
changed by the audit.

The compact audit receipt is
`/home/ubu/.cache/lay/development/td125-marker-diagnostic-Q0Tlq38N/affected-baseline-audit/receipt.json`,
SHA-256
`26adeaeb490acfd9ba9fe2559e46b1dd0c306de56f262abce31c5c6d3f8f03da`.
It preserves the focused log, full candidate IBus log, exact-isolation logs,
clean-HEAD build/run logs, source-diff binding and per-run failure sites.

Measured here: focused TD-125 behavior, the complete local IBus surface, the
full changed-file correctness/package surface, exact isolation and clean-HEAD
reproduction. Not measured here: a passing immutable release transaction,
release binary binding, final adapter binding, installed-client regression
matrix or performance lane. Runtime authority remains unchanged. Promotion is
still gated on the source-bound remote release and physical matrix.

### Capability-publication race and ownership retention — 2026-09-20

The first marker release snapshot passed the remote architecture, lint, adapter,
release-build and all 2,878 correctness/package checks. It was installed with
byte-for-byte rollback copies of both the engine and Firefox adapter. Native GTK
then produced exact `просто которое ` at cursor 15. In the following Chrome
case, the first word appeared as `росто`: the first physical letter was lost,
while the second word was corrected once without duplication.

The engine trace identified the first shared mechanism. Chrome initially
published caps 9, so the engine legitimately owned the first `п` as preedit.
Chrome then published unmarked caps 41 before the second key. The old transition
handler treated the gain of `SURROUNDING_TEXT` as loss of the whole-word route
and discarded the already-owned preedit. The trace proves that the engine
received and decoded `п`; this was not an input-injector omission. Exact
receipts are under
`/home/ubu/.cache/lay/development/td125-marker-release-20260920T165725Z/matrix/`,
and the source-bound gate receipt is `artifact-binding.json`, SHA-256
`2118e95633cb63adf9bc57c2fae1c64f4fc85ed8fdd64a9f3bd0e44e022a2afc`.

The candidate engine and adapter were immediately rolled back to SHA-256
`14afbc69b48e737f5e4268ec98a21bbe3d304aceeeb8fd54ba81b48636f07607`
and `ce9c258eec489a95dfadb8d23ba03d6f1afc79ab093ce63b7a3a54589a0887de`.
The global IBus daemon retained PID 4062416. The failed candidate therefore has
no runtime authority.

The systemic correction separates capability to continue an owned preedit from
capability to start the next one. If `PREEDIT_TEXT` remains available, an
already-owned word survives a caps9-to-unmarked-caps41 or marker transition and
finishes at its normal boundary. After that boundary, unmarked caps41 starts no
new preedit and retains managed commits. Actual loss of `PREEDIT_TEXT` still
discards the uncommitted word. The new lifecycle proof exercises the complete
caps9 first-key, caps41 continuation, boundary commit and next managed word; all
14 TD-125 tests pass. Candidate generation, ranking, verifier and `SafetyGate`
remain unchanged.

Not yet tested after this correction: affected closure, architecture refresh,
immutable release build or any physical client. Runtime authority remains the
restored pre-experiment state.

### Provisional source-bound release and installed matrix — 2026-09-20

The ownership-retention correction passed a new immutable release transaction
at
`/home/ubu/.cache/lay/development/td125-marker-release2-20260920T172153Z/`.
The source archive contains 1,412 files and is bound to base
`1f8416c98100ba12efec6cb8a434417f632a3963`; its SHA-256 is
`d0746c7a5bff1ea46e3177896edeceb1108cec992832d71e5a94e13c83178fde`.
The remote `dedicated-20cpu` gate passed 2,879/2,879 selected tests: 2,843
correctness and 36 package tests, including `lay-ibus-engine` 578/578 and
`lib:lay` 1,795/1,795. Architecture checks, lint, both adapter unit tests, the
release build and final diff check also passed. The exact gate log SHA-256 is
`ef79ac579fb22b9d13432e4b6261e7a424ef0d4475a230eaa4dc6e4f95262850`.
The empty known-failure ledger remains empty. The manifest contains 2,905
entries: 2,843 correctness, 36 package, 11 opt-in performance and 15 ignored;
its SHA-256 is
`4dab5f935c0246d757202432af4e69644a954af5409663a61d3dbb15c88e3775`.

The resulting `lay-ibus-engine` reports 1.0.73, is 7,909,608 bytes and has
SHA-256
`32b3734cd3e89cc98d429c63714824833a5553b87c649cadbe55a2b7010d2257`.
The 16,752-byte Firefox adapter has SHA-256
`924b59822e26f5ad3e6e0daf2a8f762b15e6905bb0b3683ce56bee55744a6f22`;
its source SHA-256 remains
`9383f4db11b651d327ea0d7f276c63fc6ef445461579bc0dd21bb3fed9367380`.
`artifact-binding.json` binds the snapshot, gate and both release artifacts and
has SHA-256
`a05d9ac3897aa8ade7d1040177cb276a9d3042e2bc3734b89cc96444617de402`.

Both artifacts were installed atomically after the passing gate. Exact rollback
copies are preserved under
`/home/ubu/.local/state/lay/release-backups/td125-marker-release2-20260920T174200Z/`.
The installation receipt is `installation.json`, SHA-256
`3963a661367d0e3ce1711dfebff56c8d0fe75e9ea541bc6280a9cf1d22b61100`.
It records the installed engine and adapter hashes above, selected engine
`lay-ime-ru`, and preservation of the global IBus daemon PID 4062416. Runtime
authority changed to this final engine and adapter; no rejected intermediate
has installation authority.

The installed client matrix returned exact `просто которое ` in all four
required clients:

- native Wayland GTK, cursor 15;
- the existing Wayland Chrome process, with main PID 483227 and start identity
  49964070 preserved;
- an independently launched fresh native Wayland Firefox profile that mapped
  the exact installed adapter, while the original Firefox PID 2748943 and start
  identity 110858340 remained unchanged;
- an isolated Kitty instance on the terminal transport.

The aggregate receipt is `final-matrix.json`, SHA-256
`aed1182709601a58f42705c615f9a3cac178f6c9e4d1c8103ada62c446131680`,
with verdict `PASS`. Its component receipt SHA-256 values are
`1af132ceea7a71af4d016f4576e04e8bb57deb4d635df1994f5b3f2bab1c7b88`
for GTK,
`d860b66b8a3e8b2039b65747cd0863049194160edf874381e203f4a9a664a9d8`
for Chrome,
`d6130de2d72c640705f59884a8bf4db8a1acb3e2946f01a01ef7da6f377d7648`
for Firefox and
`581883ef857d2b0144ba7f4223a2935ab3b7b8195e7b789ef5a9bcbf2cfa3ead`
for Kitty.

Two copied generic Firefox runtime-smoke attempts are retained separately under
`matrix/firefox-fresh/` and `matrix/firefox-fresh-run2/`. In both attempts the
temporary virtual input device disappeared before the first key event; the
engine traces contain no managed commit or preedit update. These are setup
failures, not Firefox product observations, and are not counted in the passing
client denominator. The independent physical Firefox run above supplies the
required product observation.

Measured by this provisional section: immutable source identity, architecture/lint and
release construction, every non-opt-in correctness/package gate, exact engine
and adapter identities, atomic installation with rollback material, and one
exact installed correction in each required client class. Not measured: the 11
opt-in performance tests, other browser versions or clients outside the frozen
matrix. No latency, throughput, RSS or broader browser-compatibility improvement
is claimed. The later independent review below rejected this acceptance before
repository integration.

At the time it was created, the immutable archive predated only the associated
evidence/status entry and graph refresh. A 20-file byte comparison then found no
runtime, test, adapter or manifest difference, and the architecture wrapper
passed all 11 checks with 703 Rust sources. The retirement repair below changes
that source after the archive, so the old release binding no longer covers the
current candidate.

### Independent-review rejection, rollback and retirement invariant — 2026-09-20

The final read-only review scored the provisional candidate **5/10 FAIL** and
found one grouped authority defect. While legacy word preedit was active,
`composition.buffer` was client-uncommitted text but was also mirrored into the
local committed tail and shared handoff for lexical work. Cursor-zero Backspace,
FocusOut/focus reset and soft Reset cleared the buffer and ownership flag without
removing that mirror. The canceled suffix could therefore survive as false
committed-tail authority and later authorize deletion against text the client
never owned. Admission FocusOut could additionally seal and transfer the false
suffix. The review also found that composition-disable cleanup reset visibility
before emitting `clear_preedit`, allowing the clear to become a no-op.

The installed engine and adapter were immediately rolled back atomically to
SHA-256
`14afbc69b48e737f5e4268ec98a21bbe3d304aceeeb8fd54ba81b48636f07607`
and
`ce9c258eec489a95dfadb8d23ba03d6f1afc79ab093ce63b7a3a54589a0887de`.
The global IBus daemon retained PID 4062416 and selected engine `lay-ime-ru`.
Rollback receipt:
`/home/ubu/.cache/lay/development/td125-marker-release2-20260920T172153Z/rollback-after-review.json`,
SHA-256
`ac4d5130afd0f84563f63128ae47399aab2b185d978d14d1c2457969234651ba`.
Runtime authority is the restored pre-experiment baseline; the provisional
release and its passing client matrix have no current installation authority.

The grouped causal RED added local-tail, shared-handoff and continued-input
postconditions for soft Reset; recent focus reset plus a new engine; committed
prefix plus owned preedit, cursor-zero Backspace and the following native
deletion; client-visible preedit clearing when composition is disabled; and
shared-tail equality on commit and capability-loss paths. On unchanged runtime
source, 19/23 selected TD-125 tests passed and the four new cancellation/clear
cases failed. Exact log:
`/home/ubu/.cache/lay/development/td125-retirement-red-20260920T183500Z/focused-red.log`,
SHA-256
`cc1be4bc02446d59ac8a0390a798f4919bd65ac5c644c1647a7642a43a51d17c`.

The systemic repair admits exactly two ownership exits. A successful
`CommitText` first reconciles the mirror to emitted text and then retires the
owned buffer. Cancellation instead strips the exact owned suffix from the local
tail, clears fail-closed on suffix mismatch, rebuilds preedit state and updates
the shared handoff before any later authority use. Soft/focus reset explicitly
invalidates any seal or reset rereceipt covering canceled text. An authenticated
admission FocusOut with owned preedit refuses text transfer and revokes the word
scope, making the successor source-free. Cursor-zero Backspace then mirrors the
native deletion of the preceding committed scalar. Composition-disable cleanup
now clears the visible client preedit before resetting local visibility.

The complete TD-125 selection now passes 24/24, including an authenticated
FocusOut/admission proof. A direct complete IBus run passed 579/585 and reported
six failures in the already identified asynchronous TD-121 reset/replay harness
family; every failed case then passed in exact one-process isolation. This is
recorded as unresolved local harness instability, not as a complete target
PASS. The complete manifest now contains 2,909 tests: 2,847 correctness, 36
package, 11 performance and 15 ignored. Its SHA-256 is
`8846011e7bb38de1c537a7c97d0de4486eba728a002457b179364b6e09852672`;
the known-failure ledger remains empty and is rebound to that manifest.

Measured here: causal RED, focused lifecycle GREEN, admission transfer refusal,
full local IBus surface and exact isolation of its reported TD-121 failures.
Not yet measured after this repair: a passing source-bound affected/full release
gate, architecture refresh, release binary, installation or physical client
matrix. No performance claim is made. Promotion remains blocked on those gates
and a new independent review PASS.

### Cursor-zero cancellation admission preflight — 2026-09-20

The follow-up review rejected the first retirement repair before release. A
cursor-zero Backspace cancels the entire client-uncommitted legacy preedit and
then passes one native Backspace to the client. The local mirror therefore
changes by more than one scalar: for example, `x ab` becomes `x`. The existing
callback settlement compares that result with the pre-cancellation mirrored
tail. It is neither an exact one-scalar Backspace nor a boundary Backspace, so a
`KnownStart` word scope can remain current and be settled at the new tail epoch.
That stale scope could authorize a later correction or deletion for the
reopened preceding word. Existing cursor-zero coverage uses no admission
adapter and therefore cannot detect this authority error.

The selected repair is to revoke the current word scope after the canceled
suffix and native deletion have been mirrored, before the accepted key callback
can settle. The cancellation crosses an uncommitted ownership boundary and
removes a committed scalar in one client event, so there is no single admitted
lineage that proves the resulting word. Reconstructing a preedit-free synthetic
`tail_before` was rejected because it would assert that the preceding word is
known without a fresh surrounding-text observation. Teaching the generic scope
advancer about this legacy-only compound edit was rejected because it would
widen a shared authority mechanism and retain authority where the client event
does not supply an exact witness.

This change does not alter candidate or lattice retention, ranking, verifier or
`SafetyGate` decisions, latency deadlines, model/package data, learning,
feedback, cache identity, reloads, allocation bounds or concurrency ownership.
It adds no timer, cache, queue, route, fallback or source of truth. It can reduce
later correction availability after this uncommon caret/cancellation sequence;
that is the required fail-closed result until new client evidence establishes a
fresh scope. The shared tail still records the exact visible result for
bookkeeping, but it carries no word authority. Capability loss, FocusOut,
ordinary exact Backspace, terminal and atomic paths remain separate. Rollback is
the current installed baseline plus removal of this source delta. The required
proof is an authenticated legacy `ProcessKeyEvent` regression that is RED while
the old callback can re-settle `KnownStart`, then GREEN only when both the local
scope and adapter token are revoked after the compound edit. Full remote gates,
review and the client matrix remain mandatory afterward.

The authenticated regression was then run through the remote development
entrypoint, not a local Cargo command. Before the repair, 582/583 selected IBus
correctness tests passed and the new case failed exactly because
`context_word_is_known()` remained true after `l ab -> l`. The RED report is
`/home/ubu/.cache/lay/development/run-0pt91o6_/RESULT.json`, SHA-256
`0c8074256572ded50d4f4d9f47b412aaf2f0c38c8ae551310f832a72cc7b7c6f`.
After the compound-edit path revoked the admitted word before callback
settlement, the same remote target passed 583/583. The GREEN report is
`/home/ubu/.cache/lay/development/run-c658ftdb/RESULT.json`, SHA-256
`eabb9979e3391098ae7b7a5b21fd4d75752c19f620b7d462f583692e7acf99ea`.
Both runs used the `dedicated-20cpu` guard, 20 build jobs, one Rust test thread
and the shared bounded target. Runtime authority did not change. This is a
development-target proof only. Remote discovery then regenerated 2,910 test
entries: 2,848 correctness, 36 package, 11 opt-in performance and 15 ignored.
The manifest SHA-256 is
`bede77165c06d9ed48062a55f0700de596eb7fc83843ff72331943086e67a578`;
the empty known-failure ledger is rebound to it. The source-bound architecture
wrapper then passed all 11 checks over 703 Rust sources. After recording the
review, the wrapper was repeated from a new immutable snapshot and again passed
all 11 checks over the same 703 sources. Its log is
`/home/ubu/.cache/lay/development/td125-release3-architecture-20260920T182854Z/update-architecture-graph.log`,
SHA-256 `55f7c95f184b5dff8807cecde2f79869447dc96801954244ca7c836958511155`;
the generated receipt SHA-256 is
`737e0e5cd387bf330b0fde13a5af5f1fa866c36cb31e5cc723f9902c702e9046`.
The final independent review found no severity issue and scored the repair
9/10 PASS. It confirmed that revocation invalidates local lineage, reducer
authority, unsettled callback state and callback stamps before the old callback
can settle; unchanged Backspace, FocusOut and capability-loss paths remain
outside the new branch. The immutable full release gate and physical clients
were then completed as recorded below.

### Final immutable release, installation and client acceptance — 2026-09-20

The final candidate is frozen at
`/home/ubu/.cache/lay/development/td125-marker-release3-20260920T183045Z/`.
Its 1,412-file source archive has SHA-256
`340d2f8e459de89dabfb6683547a7f7108536ba65c088c6655b95a56b65bdaf2`
and binds source HEAD `1f8416c98100ba12efec6cb8a434417f632a3963` plus all dirty and untracked
task source. The dedicated remote full gate passed formatting, all 11
architecture checks, hermetic runner contracts, the exact manifest, 2,884 of
2,884 fixed correctness/package tests, lint contracts, Firefox adapter tests,
desktop helper checks, CLI smoke and the optimized release build. It reported
zero known semantic and zero infrastructure failures. The gate log is
`check-lay-full.log`, SHA-256
`af51b179a3cbaadd5a5db4c287f602f8f45138724e802dfc3f66617c44d8c792`.
The 11 opt-in performance tests and opt-in ngram cache check were not run; no
performance claim is made.

The exact gated engine has SHA-256
`22b68fc1ac5e9b2d836af02707fb2e11eedee5beae7774a0aabea0bcaa180ac6`
and 7,910,632 bytes. The unchanged Firefox adapter was rebuilt under the same
remote guard from source SHA-256
`9383f4db11b651d327ea0d7f276c63fc6ef445461579bc0dd21bb3fed9367380`;
the resulting library has SHA-256
`924b59822e26f5ad3e6e0daf2a8f762b15e6905bb0b3683ce56bee55744a6f22`.
`artifact-binding.json`, SHA-256
`8334bbb328c2cddc6edd761502d55e0ad655cc911566eb207aa292ba7f383ce4`,
binds those artifacts to the immutable archive, gate, manifest and architecture
receipt.

Atomic installation passed and left a verified rollback copy at
`/home/ubu/.local/state/lay/release-backups/td125-marker-release3-20260920T184421Z/`.
The installation receipt is `installation.json`, SHA-256
`057d342720a855af9f3ae17060860cf0df79c34effae970d131e85c66815aead`.
Only the Lay IME engine restarted, from PID 3516592 to 3820910. The global IBus
daemon retained PID 4062416, the Lay daemon retained PID 3441527, the selected
engine and GNOME layout remained `lay-ime-ru`, and input sources were unchanged.
Runtime authority now uses the exact engine and adapter hashes above.

The installed physical matrix produced exact `просто которое ` in every
required client:

- native GTK4 returned the exact text at cursor 15;
- the existing Wayland Chrome process retained PID 483227 and start tick
  49964070;
- a fresh isolated Firefox process mapped the exact adapter, returned the exact
  text and exited, while the original Firefox PID 2748943 and start tick
  110858340 were preserved;
- an isolated Kitty terminal returned the exact text.

The aggregate receipt is `final-matrix.json`, SHA-256
`eba560a5f2c997ea23bb3fbe66fbfc517333b2a52beefd7f8b299323f4be447b`.
It also records one setup-only GTK launch that aborted before input and a
separate focus diagnostic; neither delivered the test sequence. All temporary
ydotool services are inactive and their sockets were removed. The final live
audit revalidated the installed files, running engine image, preserved browser
sessions and IBus process identity. TD-125 is therefore `DONE` for its scoped
source, installed runtime and client-acceptance contract. TD-121 and TD-123 keep
their separate status. Commit and push remain outside this result until
repository integration is explicitly requested.

After this acceptance record was written, a final source-bound architecture
refresh again passed all 11 checks over 703 Rust sources. Its log is
`/home/ubu/.cache/lay/development/td125-final-architecture-20260920T185030Z/update-architecture-graph.log`,
SHA-256 `6412a5106cab13abbee8330f5b4a57f0e5229b866ea2b9544ed3e3b1407332d7`.
`post-release-evidence-binding.json`, SHA-256
`17555c8d936b407e43d1720f286a7542f35b0a803aa5aaa4913f42100b414078`,
compares every frozen release member with the final tree. The only differences
are this document, `tech_debt/README.md` and the six regenerated graph/receipt
files; runtime source, tests, Firefox adapter and test-lane manifest are
byte-identical to the immutable gated release.

### Native first-token correction without owned preedit — 2026-09-21

Status: **FOCUSED SOURCE, GUARDED RELEASE AND PHYSICAL KITTY PASS / INSTALLED**.
Runtime authority now uses the 1.0.74 engine with
SHA-256 `2e6b3291318e24a69a0a23bb5d97b4e720de1dc6d93e33a8446834ff9a7a9f8e`.
The restored baseline remains available for rollback as recorded below.

The first attempted repair made a source-free `UnknownStart` terminal token an
owned legacy preedit until Space. It gave the correction path a complete local
token, but also made ordinary typed text client-visible as preedit. The user
rejected that highlighted input and the installed candidate was immediately
rolled back byte-for-byte. A trace of the complaint itself contained only
ordinary terminal passthrough; the other highlighted endings in that trace were
the existing display-only precognition suffixes. That distinction does not
rescue the attempted design: it would still display the first token differently
when its new route activated, so owned preedit is rejected for this repair.

The replacement keeps every printable key on the existing native terminal
path. The press and release remain unhandled, emit no commit or preedit signal,
and leave the composition buffer empty. Only after the accepted legacy callback
has settled its updated `UnknownStart` lineage may the engine prepare a Space
lease. The lease requires all of the following at capture and again at use:

- native terminal input with executable terminal erase geometry;
- live non-sensitive admission, focus, owner, epoch, layout, configuration and
  output-capability identity;
- a non-empty current suffix whose retained token length equals the lineage's
  complete observed-suffix count;
- a dedicated admission token distinct from the display-only suffix token.

The separate token prevents an `ibus_preedit` display frame from becoming edit
authority. Scheduling occurs after callback settlement because that callback is
what advances the observed-suffix count. On Space, the existing committed-tail
executor revalidates the exact frame, consumes the existing prepared
DecisionCore lease, and emits one terminal frame containing exactly one DEL per
observed suffix scalar followed by the authorized replacement and one space.
The word scope remains `UnknownStart`; this does not manufacture a known left
boundary. Unobserved client text to the left is never deleted. Consequently the
authority added here is limited to replacing the exact suffix typed and tracked
since activation. If activation began in the middle of an unobserved word, that
left fragment remains in the client and can concatenate with the corrected
suffix. This limitation is explicit; the terminal protocol supplies no
surrounding-text proof from which to infer more.

Candidate generation, lattice retention, ranking, verifier, `SafetyGate`, edit
plan validation, learning, package material and daemon ownership are unchanged.
The exact-layout and full typo decisions use the same existing correction
pipeline as later known-start tokens. There is no runtime word, suffix, source
ID, application name or test-case branch. The extra state is one admission token
inside the already bounded input frame; no timer, retry, queue, model load or
deadline was added. The full worker is still prepared between the final letter
and Space, and the existing Space wait budget is unchanged.

Measured facts:

- a real legacy D-Bus callback proof keeps all `ghbdtn` key presses and releases
  native with zero output/preedit, then emits one `DEL*6 + привет + Space`
  commit on Space;
- the same proof with the full correction pipeline keeps all nine typo scalars
  native, then emits one `DEL*9 + публикует + Space` commit; the fixture text is
  test evidence only;
- both proofs assert empty composition and no legacy-word-preedit ownership for
  every printable callback, exact Space-frame identity, one mutation owner and
  a consumed Space release;
- focused serialized checks passed: TD-125 legacy preedit 17/17, context runtime
  17/17, preedit 89/89, committed-tail 13/13, Space prefetch 11/11 and active
  composition route contract 3/3;
- the final two-test log is
  `/home/ubu/.cache/lay/development/first-word-native-suffix-20260921T214000Z/new-route-tests.log`,
  SHA-256 `975cfaa08985836ccd42f1a141067bd7f861f225029f397573615f4e2fe97c1a`;
- `graphify update .` rebuilt 22,950 nodes and 60,989 edges; its log SHA-256 is
  `e25cf4b8b7170e262aa7359b834dd727cba0f8afbeb96d5c341eeec9422f9036`;
- the guarded remote release build completed in 2m57s. Its log is
  `/home/ubu/.cache/lay/development/first-word-native-suffix-20260921T214000Z/release-build.log`,
  SHA-256 `53c7a97347a2187abbe951775ea823155149e3acd01f6bccb001356c460a969e`;
- the installed engine is 7,910,248 bytes, and PID 3246844 maps the exact
  installed SHA-256 above. `lay-daemon` retained PID 2391978, the global IBus
  daemon retained PID 4062416, and both GNOME and IBus remained on
  `lay-ime-ru`;
- the first isolated Kitty attempt did not establish the required activation
  precondition: the new engine object had no current admission owner, both
  Space callbacks were refused, and the client retained `публекует `. This is
  recorded as a harness-precondition failure rather than candidate acceptance;
- after an explicit `lay-ime-us -> lay-ime-ru` activation established owner
  generation 11 for the isolated Kitty object, physical keys produced nine
  `terminal_passthrough` records spelling `публекует`; every printable record
  was unhandled with `preedit_chars=0`, and there were zero
  `printable_legacy_preedit` or `printable_managed_commit` records;
- the following physical Space was authorized, emitted one
  `terminal_erase_commit` with nine backspaces plus `публикует `, completed its
  engine callback in 244 us, and the client captured exactly `публикует `;
- the 500 KiB bounded debug log compacted during that run. The exact generation
  11 activation interval was recovered from the retained log into
  `/home/ubu/.cache/lay/development/first-word-native-suffix-20260921T214000Z/physical-kitty-activated-v2/engine-trace-recovered.jsonl`,
  SHA-256 `8eeea5d562c21d99b63385cc6d7f52a07f9479c27e152c08ee03f003d64f98de`.
  Its physical receipt is in the same directory as `receipt.json`;
- the byte-identical pre-change engine is saved at
  `/home/ubu/.local/state/lay/release-backups/first-word-native-suffix-20260921T204632Z/lay-ibus-engine`,
  SHA-256 `0b855164c30913670d4a0808b5e7325d175507af1cbcd55c625a35a1299d59a3`.

Not yet tested: a sub-80 ms final-letter-to-Space physical interval or a real
focus that begins mid-word. Answer quality remains `UNKNOWN`; these routing and
executor proofs do not establish aggregate or per-error-class correction
quality. The aggregate receipt is
`/home/ubu/.cache/lay/development/first-word-native-suffix-20260921T214000Z/receipt.json`.

### Chromium owned-preedit acceptance repair — 2026-09-22

Status: **INSTALLED ORDINARY-CHROMIUM PASS**. The post-settlement candidate
SHA-256 `6f073ce3cee53bd6de1e7eed35e651e77a32ed47c3a343f57f9bf1bafbd38444`
is installed and loaded. The earlier candidate recorded below was rejected
after its required ordinary-Chromium acceptance test and was restored
byte-for-byte to stable SHA-256
`0b855164c30913670d4a0808b5e7325d175507af1cbcd55c625a35a1299d59a3`
before the scheduling repair.

The failed candidate was tested in the user's existing Chromium process, PID
483227. Physical ASCII input for two occurrences of the typo produced
`публекует публекует `: the first word was unchanged and the second candidate
was not reached in that candidate run. The receipt is
`/home/ubu/.cache/lay/development/chrome-physical-regression-20260922T001500Z/receipt.json`.
The Double-Shift result from that first run was invalid because the daemon
correctly ignores the `ydotoold virtual device`; it was not counted as a
physical-gesture result.

After rollback, a custom evdev `UInput` device named
`lay-physical-regression-keyboard` was present before daemon enumeration. On
the same Chromium page, `публекует публекует ` became
`публекует публикуем `, while physical Double Shift changed `ghbdtn` to
`привет` and synchronized US to RU. This separates the defects: rollback
restored the gesture, while Chromium first-token autocorrection remained
broken. The control receipt is
`/home/ubu/.cache/lay/development/chrome-stable-baseline-20260922T004000Z/receipt.json`.

The first Chromium printable arrives with capabilities `9`, starts the existing
owned legacy preedit, and remains in that preedit after Chromium advertises
capabilities `41`. Its word lineage is still `UnknownStart`, so the complete
token had no ordinary `capture_input_frame_identity()` and no Space lease was
scheduled. At Space the engine therefore took `space_legacy_preedit_commit`.
This is the first authority layer that lost the otherwise complete local
target.

The repair admits one separate Space identity for an `UnknownStart` token only
while either of these already owned representations remains exact:

- the complete native-terminal observed suffix from the prior repair; or
- an active legacy preedit whose non-empty composition is byte-equal to the
  current tail token.

The identity carries the live admission token and complete lexical
coordinates. Scheduling and consumption both revalidate path, focus, owner,
epoch, layout generation, configuration, output capabilities, representation
ownership and an exact recapture of the frame. Chromium capability churn,
composition edits, focus changes or a different tail make the lease stale.
Display-only preedit authority remains distinct. `SafetyGate`, backend edit
authorization, edit-plan validation, verifier authority, candidate generation,
ranking and learning are unchanged.

The initially suspected second-word ranking defect was not changed. Existing
physical traces show that a preceding corrected `публикует` makes the next
`публекует` select `публикует`; the wrong `публикуем` result occurs when the
uncorrected first typo is itself the left context. Repairing the first lost
authority therefore restores the existing contextual result without a score
exception.

Measured focused proof:

- the new real legacy-callback fixture reproduces Chromium capabilities
  `9 -> 41`, `UnknownStart`, owned preedit and physical keycodes;
- before the repair it failed because no exact owned-preedit Space frame
  existed;
- after the repair it commits the deterministic built-in correction
  `рабоает -> работает ` from the active preedit with no surrounding-text
  deletion;
- the source fixture proves only the first Chromium word. The required live
  acceptance remains two occurrences of `публекует`, with exact final text
  `публикует публикует `;
- fixture words and key sequences are evidence only; runtime code has no word,
  character, source-ID, application-name or test-name condition;
- sequential focused suites passed: native first-token `2/2`, Chromium
  owned-preedit `1/1`, TD-125 legacy preedit `17/17`, context runtime `17/17`,
  preedit `89/89`, committed tail `13/13`, Space prefetch `11/11`, and active
  composition route contract `3/3`;
- the main focused log is
  `/home/ubu/.cache/lay/development/chrome-owned-preedit-20260922T020000Z/focused-tests.log`,
  SHA-256 `87b64e2bdee634c557f9808ccdf9818c34ec0a59e44aede161c8f397b161dced`;
  the separate context-runtime log is beside it;
- both full-correction fixtures passed in separate hermetic processes with
  network disabled and home/config/cache masked. Their logs are under
  `/home/ubu/.cache/lay/development/chrome-owned-preedit-20260922T020000Z/hermetic-fixture-exact/logs/`;
- repeated shared-process engine runs exposed deadline flakes in unrelated
  adapter fixtures because background full-correction workers could deschedule
  tests with real short admission windows. The systemic test-infrastructure fix
  process-isolates all 147 `context_admission::adapter::tests::*` cases. Helpers
  that explicitly model Reset as the next callback rebind their fixture recency
  timestamp at that callback, and synthetic callback messages set
  `NoReplyExpected` so an unrelated `UnknownObject` transport reply cannot enter
  the asserted signal queue. Product deadlines and runtime logic are unchanged;
- with that isolation, the canonical correctness target passed 586/586. The
  summary is
  `/home/ubu/.cache/lay/development/chrome-owned-preedit-20260922T020000Z/focused-engine-isolated-v2/SUMMARY.json`,
  SHA-256 `93c34e8649f884952ec9ea6f5606f3dda994f22250cfed4d18293545981a4d18`;
- the refreshed manifest contains 2,913 tests: 2,851 correctness, 36 package,
  11 performance and 15 ignored. Its SHA-256 is
  `38e14510d31f82074741ab77b55b5c2c68cb4c4862ce19465e95ee1a073e5eaf`;
  the zero-failure ledger remains empty and is rebound to that manifest;
- `scripts/check-lay-changed.sh` passed with 2,887 selected correctness and
  package cases, zero semantic or infrastructure failures, `cargo check` over
  the library and all binaries, transition replay, and the unsafe-edit
  scoreboard at zero gate failures. Its log is
  `/home/ubu/.cache/lay/development/chrome-owned-preedit-20260922T020000Z/check-lay-changed-isolated-v2.log`,
  SHA-256 `6b28bc2a314de9ab07d5df6637bd83ead66f7fe8326c4f66a89af9b1b75cbc03`.

The mandatory installed ordinary-Chromium run then rejected that source
checkpoint. With candidate SHA-256
`0d755ac358f2b883eb502450c94ed8b0a54a7fdec1f105185646dea4b2996997`,
the existing Chromium process produced exact
`публекует публикует `: the first owned-preedit token was unchanged, while the
second known-start token used the prepared correction successfully. Physical
Double Shift on the pre-enumerated `lay-physical-regression-keyboard` still
changed `ghbdtn` to exact `привет`. The receipt generator itself had a Python
syntax error and its shell did not stop on that error, so this run is recorded
as a failed experiment from its bounded raw evidence, not as an acceptance
receipt. The evidence directory is
`/home/ubu/.cache/lay/development/chrome-owned-preedit-physical-20260922T025000Z/`.
The stable installed engine was then restored byte-for-byte to SHA-256
`0b855164c30913670d4a0808b5e7325d175507af1cbcd55c625a35a1299d59a3`.

The trace places the first loss after correction preparation and before Space
lease admission. Final-letter callback serial 87 scheduled worker generation 8
for tail epoch 12; the full route reached `prepared` in 46,556 us. That work was
captured inside the printable handler, before callback settlement advanced the
`UnknownStart` lineage from eight to nine observed suffix characters. At Space
serial 90 the exact recapture therefore returned `stale_lease`, after which the
engine correctly fell back to `space_legacy_preedit_commit`. No candidate,
ranking, verifier or mutation authority rejected the correction.

The systemic repair removes owned legacy-preedit Space scheduling from the
printable handler and schedules it in the existing successful post-settlement
callback hook, after the new tail and lineage token are both final. Native
terminal scheduling stays in that same hook. Consumption still requires the
exact current-frame recapture; no revalidation was weakened. The Chromium
fixture now interrogates the actually scheduled worker slot with the current
post-settlement frame and rejects `Stale` before installing its deterministic
`рабоает -> работает ` lease. That causal fixture passed 1/1.

The first complete engine reruns exposed a separate fixture-ordering defect.
The semantic `legacy_key` helper started the observer and engine callback
simultaneously, so every semantic fixture also raced the real 1 ms rendezvous
deadline. Different isolated runs failed different callback cases, while each
reported case passed alone in the same hermetic environment. Repeating one
case in 20 fresh clean processes measured 19 PASS and 1 FAIL. The systemic
test-only repair now has the observer publish the received ingress stamp before
the semantic callback consumes it. Dedicated adapter tests still own pending,
timeout, marker and revocation schedules, so rendezvous coverage remains
separate. Immediate-Reset helpers bind their recency precondition at the
modeled Reset callback. Product deadlines and runtime authority are unchanged.

After that fixture repair, the focused hermetic engine target passed 586/586
with zero failures. Its summary is
`/home/ubu/.cache/lay/development/chrome-owned-preedit-post-settlement-20260922T000400Z/focused-engine-v5/SUMMARY.json`,
SHA-256 `ec0f2ea5470913d8f1c2f3152c1168fbf767b732cc146f2fb3f68298835b94d6`.

The complete changed-source gate then passed with all 2,887 selected
correctness and package cases, zero semantic or infrastructure failures,
586/586 engine tests, 267/267 daemon tests, 1,795/1,795 library tests, `cargo
check` over the library and binaries, transition replay, and the unsafe-edit
scoreboard at zero gate failures. Its log is
`/home/ubu/.cache/lay/development/chrome-owned-preedit-post-settlement-20260922T000400Z/check-lay-changed-v2.log`,
SHA-256 `9c403e6323d324779bbbe156f46645cebf16ca7ca131deaa2636f242c853cba5`.

The optimized release candidate was built on `e@192.168.3.94` under the
guarded 20-CPU profile. The 7,910,504-byte artifact has SHA-256
`6f073ce3cee53bd6de1e7eed35e651e77a32ed47c3a343f57f9bf1bafbd38444`;
the build log SHA-256 is
`855c3eb888b7fef9a60fda9cefabe4340d91d512ae41b2d0b71f3006bc6ca187`.

Installed acceptance used the user's existing ordinary Google Chrome process,
PID 483227. Physical ASCII `ge,ktretn ge,ktretn ` produced exact
`публикует публикует `, including the previously failing first owned-preedit
token. On the pre-enumerated `lay-physical-regression-keyboard`, physical
Double Shift converted `ghbdtn` to exact `привет` and synchronized the layout
to `lay-ime-ru`. The installed file and loaded `/proc/426630/exe` both matched
the candidate SHA-256. The PASS receipt is
`/home/ubu/.cache/lay/development/chrome-owned-preedit-post-settlement-physical-20260922T040000Z/receipt.json`,
SHA-256 `0f4275f1a15e1ad081a6aee0c25c6b70afc55bb05a2d32e38a4b3998dceb04cb`.

The first execution of that harness was a false FAIL after both client effects
had succeeded: concurrent telemetry POSTs wrote the penultimate `приве` event
after the final `привет` event. The corrected receipt orders client events by
their browser monotonic `performance.now()` value, and the complete run was
repeated from installation. The earlier raw run is retained under `first-run/`
and is not counted as acceptance.

Not tested after this scheduling repair: sub-80 ms final-letter-to-Space
timing. The fixed L1 quality proof was not rerun because candidate production
and ranking did not change; answer quality remains `UNKNOWN` rather than being
inferred from routing and client acceptance.

### Firefox committed-prefix and suffix-only continuation — 2026-09-22

Status: **IMPLEMENTATION PREFLIGHT / WHOLE-WORD PREEDIT REJECTED BY USER**.
The installed Firefox 156 process loads the existing compatibility adapter and
engine SHA-256
`6f073ce3cee53bd6de1e7eed35e651e77a32ed47c3a343f57f9bf1bafbd38444`.
The adapter currently adds private capability bit `1 << 30`; the engine names
that bit `LAY_COMMIT_ONLY_PREEDIT` and consequently turns every managed word
into one active preedit. That behavior can autocorrect the first word, but the
user rejected its visible contract: the already typed prefix must remain
ordinary committed text and only the completion suffix may be preedit.

Two controlled Firefox 156 diagnostics separate the adapter's mechanisms. With
the installed marker adapter, an isolated field corrected exact
`публекует -> публикует`, but the whole token was preedit. With a temporary
adapter that retained only the post-release and post-Reset
`retrieve-surrounding` notifications, every scalar used the existing
`printable_managed_commit` route and the preedit contained only completion
suffixes such as `лика` and then `ика`; Space left `публекует ` unchanged. The
second result is an authority-timing failure, not a candidate-quality result:
the exact Firefox snapshot arrived after printable callback settlement, while
the existing UnknownStart Space lease admitted only a native-terminal suffix
or an owned whole-word preedit. Evidence is retained at:

- `/home/ubu/.cache/lay/development/firefox-fresh-adapter-diagnostic-20260922T043700EEST/receipt.json`;
- `/home/ubu/.cache/lay/development/firefox156-notify-only-diagnostic-20260922T050000EEST-run2/receipt.json`;
- `/home/ubu/.cache/lay/development/firefox156-notify-only-two-word-20260922T050500EEST/receipt.json`.

Selected design: rename the private bit to
`LAY_EXACT_SURROUNDING_REFRESH`. It states only that the client adapter forces
an exact surrounding-text refresh after handled printable release and Reset.
It must no longer select whole-word preedit. A marked managed-commit word may
capture an UnknownStart Space identity only when SurroundingText is advertised,
composition is empty, the current snapshot is unselected and boundary-exact,
the retained token equals the snapshot suffix, and its scalar count equals the
lineage's complete observed-suffix count. Schedule work when that exact marked
snapshot arrives, because this callback is the first point at which Firefox's
post-commit text is proved. Capture and consumption revalidate the same focus,
owner, epoch, layout, configuration, capability, token and exact snapshot.

Alternatives considered:

1. Keep the current marker as a whole-word-preedit request. It has a working
   single-commit correction transport, but violates the explicitly required
   typing surface and makes ordinary text one highlighted composition.
2. Treat every GUI SurroundingText snapshot as enough for an UnknownStart
   correction. This removes the adapter distinction and could grant an edit
   lease from clients whose snapshots are delayed or opportunistic. It broadens
   authority beyond the measured Firefox contract and is rejected.
3. Selected: reuse the marker for exact refresh and grant the bounded lease
   only from the exact marked snapshot. This adds no new text owner and keeps
   existing managed commits plus suffix-only display.
4. If Firefox still applies `CommitText` before its preceding
   `DeleteSurroundingText` mutation becomes visible, keep this source change but
   make the adapter serialize that already-authorized pair. That is a separate
   transport repair and is admitted only after the selected source route
   reproduces the physical failure; no speculative delay or queue enters this
   first change.

Consequences and invariants before production code:

- **Candidate retention, ranking and false authority:** candidate generation,
  lattice membership, ranking, DecisionCore, verifier, `SafetyGate` and sealed
  edit-plan validation do not change. No fixture word, suffix, source ID,
  application name or score enters runtime. The private bit cannot authorize an
  edit without the exact live snapshot and existing prepared decision.
- **Latency and tail behavior:** one full correction is scheduled only after
  the adapter-triggered snapshot rather than inside the printable callback.
  This shortens the available final-letter-to-Space preparation interval, but
  adds no wait or deadline. A too-fast Space keeps the current fail-closed
  behavior. Ordinary characters remain one `CommitText` each and only the
  existing display suffix may use preedit.
- **CPU, RSS and allocation:** the change adds one boolean capability check and
  one existing identity capture per changed exact snapshot. It adds no model
  work, timer, polling loop, cache, queue, worker or persistent allocation.
  Existing prefetch work is moved to the first causally valid snapshot rather
  than duplicated.
- **Cache identity, packages and reloads:** the existing input-frame identity,
  material generation, configuration digest and admission token remain the
  cache identity. Package/delta loading and reload invalidation are unchanged.
  A marker transition invalidates prepared Space work just as a transport
  capability transition does.
- **Learning and feedback:** no completion or correction is learned at
  dispatch. Existing exact visible-postcondition feedback remains required.
  A routing PASS still leaves answer quality `UNKNOWN` without the fixed heldout
  proof.
- **Concurrency and stale results:** callback order is explicit: managed commit,
  printable settlement, exact SurroundingText observation, then scheduling.
  Focus, selection, epoch, token, layout or snapshot changes make the lease
  stale. Duplicate snapshots do not add another mutation owner, and no retry or
  fallback mutation is added.
- **Failure and rollback:** missing marker, missing SurroundingText, selection,
  incomplete suffix, stale snapshot or unready worker produces no correction
  authority. Rollback is the marker semantic rename, the bounded identity
  branch and its tests; the adapter notification hooks remain independently
  useful.
- **Compatibility and maintenance:** unmarked Chromium retains its owned
  preedit route; native terminal retains terminal passthrough; atomic and daemon
  routes are unchanged. Marked Firefox uses managed commits and suffix-only
  display. The private bit and adapter remain one compatibility boundary and
  should be removed when Firefox supplies the required refresh and an applied
  replacement receipt natively.

Proof order: make the marker regression prove per-scalar commits, empty active
composition and absence of whole-token preedit; make a real legacy callback
fixture publish the exact marked Firefox snapshot and prove a current prepared
Space identity plus one authorized delete-and-commit sequence; run the focused
remote engine and adapter checks; refresh the architecture graph; then install
only the built candidate and test exact `публекует -> публикует` in Firefox.
That physical run determines whether the existing Firefox delete-plus-commit
transport is sufficient or whether option 4 needs its own measured preflight.
No runtime authority has changed at this preflight milestone.

The source implementation and proof stages are now complete. The marker
regression first failed on the unchanged mechanism with zero per-scalar commits
and a whole-token preedit; its RED receipt is
`/home/ubu/.cache/lay/development/run-sjqi6h2y/RESULT.json`, SHA-256
`1e5ed510e46f485ba2f31a9f0a9183ddaa78a8bb0ae96b27f2ef83555afc02e5`.
After the semantic change, the focused engine target passed 587/587, including
the real legacy-callback Firefox fixture. That fixture proves seven individual
commits for `рабоает`, empty active composition after every key, exact marked
snapshots, a non-stale prepared Space frame, and one authorized
`DeleteSurroundingText` followed by `CommitText("работает ")`. It proves signal
order and engine state, not Firefox's application of those two effects. The
focused receipt is `/home/ubu/.cache/lay/development/run-rrb4bd_u/RESULT.json`,
SHA-256
`c60cd3bcfb3b47d94b05026676d049f845d7a4d03a9f9b8e827ee768f076e5ce`.

The canonical manifest was regenerated remotely and contains 2,914 tests:
2,852 correctness, 36 package, 11 performance and 15 ignored. Its SHA-256 is
`96be80bb6bd52e78d8be9a24fcd2e8036ad64798466a4cdd0b9dd2f237fc371e`;
the zero-failure ledger is rebound to that exact manifest. Automatic
changed-source run `run-_5af93x7` then passed all 2,888 selected correctness and
package cases with no reported failure. Its receipt SHA-256 is
`c5ad5106f07bd09292cc5e7ab00f20b7842b8766ccb01f6fbb80f430052d2f73`.

The required remote architecture refresh passed. The generated receipt reports
source fingerprint
`6b08e1a156cd90272ef48e9184112ea3254f7bebe6fd6fb0c4323cf92a8df888`,
graph fingerprint
`464af3d054e1fa397bfb8ce3e35d80f061122433390a5634e9013d12e0b7c2ff`
and verdict `PASS`; the wrapper log is
`/home/ubu/.cache/lay/development/firefox-exact-refresh-20260922/update-architecture-graph.log`,
SHA-256
`842d93d83533f9e5eb948aba6ec3f2afb0481b5a4572bc5969b3c9ba239c165a`.

Measured scope at this milestone: source formatting, the complete engine target,
the canonical correctness/package denominator, Firefox adapter unit contract,
source binding and all architecture checks. Not yet tested: installed candidate
bytes, Firefox's physical delete-plus-commit result, sub-80 ms Space timing, or
the fixed heldout quality proof. Installed runtime authority is still the prior
whole-word-preedit candidate until the next installation step.

#### First committed-prefix physical run and Reset re-receipt preflight

The built candidate was installed without restarting Firefox or global IBus.
The engine changed from PID 426630 to PID 901820; `lay-daemon`, `ibus-daemon`,
the selected Russian source and the user's original Firefox PID 635409 with
start tick 166280532 were preserved. The installed and loaded engine was the
7,910,888-byte candidate with SHA-256
`8c026da6ca6fda3d2d267aab43aacda4703cf59a8a9f021da2d513eb0b5f375b`;
the unchanged adapter had SHA-256
`924b59822e26f5ad3e6e0daf2a8f762b15e6905bb0b3683ce56bee55744a6f22`.
The installation receipt is
`/home/ubu/.cache/lay/development/firefox-exact-refresh-20260922/installation.json`.

The first isolated Firefox 156 physical run rejected correction acceptance:
exact `публекует ` remained `публекует ` instead of `публикует `. Its FAIL
receipt is
`/home/ubu/.cache/lay/development/firefox-exact-refresh-20260922/physical-firefox-run1/receipt.json`.
The requested typing surface did pass in that same run. All nine printable keys
used `printable_managed_commit`, the active composition stayed empty, and the
only visible preedit was a completion suffix; the complete committed token was
never republished as preedit. The original Firefox identity remained unchanged.

The trace places the first loss before scheduling and after the exact client
receipt. Firefox issued an authenticated soft `Reset` after each managed
commit. After the final scalar, the engine armed and confirmed a Reset
re-receipt at tail epoch 12 with all nine suffix characters and an exact
unselected surrounding snapshot. `context_observed_suffix_is_current()` already
accepts that strict one-shot proof when the post-Reset word scope counts fewer
characters than the retained token. The newer marked-Firefox helper then
requires equality with that post-Reset scope count, rejects the same exact
proof, schedules no Space work, and Space takes `managed_fallback_commit` in 37
microseconds. Candidate generation, ranking, verifier and replacement transport
were therefore not reached.

Selected repair: keep the existing marker and current-snapshot predicates, and
let `exact_marked_surrounding_suffix_is_current()` admit either the ordinary
complete observed-suffix equality or the existing confirmed exact Reset
re-receipt. The Reset alternative remains conjunctive with marker presence,
SurroundingText support, empty composition, current owner/token/epoch/scope,
exact unselected boundary-bounded snapshot and the one-shot predecessor-token
transition. Capture, scheduled identity and Space consumption re-evaluate that
same helper. The correction path does not consume or settle the re-receipt as a
manual-toggle handoff; the next text mutation invalidates its tail epoch.

Alternatives rejected at this measured boundary:

1. Settling every Reset re-receipt into the ordinary lineage before scheduling
   would mutate reducer state for display and automatic correction users, and
   would turn a bounded Firefox compatibility decision into a general ownership
   migration.
2. Ignoring Reset and comparing only the retained token to the visible snapshot
   would discard predecessor-token, observation-revision and post-Reset token
   checks, admitting stale or duplicated client evidence.
3. Returning to whole-word preedit would hide this authority gap but restore the
   user-rejected Firefox interface.

Consequence update before the second production edit:

- **Candidate, rank and false authority:** correction candidates, lattice,
  scores, verifier, `SafetyGate` and edit validation remain unchanged. The new
  alternative can reach them only through an already measured exact Reset
  receipt plus the private adapter marker.
- **Latency and tail behavior:** scheduling starts on the same exact
  SurroundingText callback that confirms the re-receipt. There is no wait,
  timer, retry or additional mutation. A worker that is not ready by Space still
  fails closed. Per-scalar committed prefixes and suffix-only preedit remain the
  required surface.
- **CPU, RSS and allocation:** one existing boolean predicate is evaluated in a
  branch already entered for changed surrounding snapshots and again at lease
  consumption. No owner, worker, queue, cache or allocation is added.
- **Identity, invalidation and concurrency:** focus, capability, owner, token,
  epoch, scope, selection, surrounding revision, snapshot or tail changes revoke
  the proof through existing predicates. A second nonmatching receipt clears it;
  an unchanged duplicate snapshot cannot schedule a second job.
- **Packages, reloads, learning and feedback:** model packages, delta reload,
  configuration identity, learning and exact visible-postcondition feedback are
  untouched. Route completion still does not establish answer quality.
- **Compatibility, failure and rollback:** unmarked clients, native terminals,
  owned Chromium preedit, atomic processing, daemon routes and manual-toggle
  consumption retain their predicates. Missing or stale Reset evidence produces
  the current literal Space fallback. Rollback is the single predicate
  alternative plus its causal fixture; no compatibility route remains behind.
- **Maintenance and removal:** this reuses the reducer's existing strict Reset
  witness instead of adding a second source of truth. It is removed together
  with the Firefox exact-refresh marker once native Firefox provides timely
  exact replacement receipts.

Proof order for this repair: first make the real callback fixture reproduce a
soft Reset after each managed commit and fail at Space-frame capture; then change
the shared predicate and require that fixture to prove the confirmed exact
re-receipt, current scheduled identity, per-scalar commits, empty composition
and one delete-plus-commit sequence. Run the complete remote engine target and
changed-source gate, rebuild under the guard, install only that artifact, and
repeat the exact physical Firefox case. A physical duplicated replacement would
be a separate transport-order failure and would require a new preflight before
any adapter serialization change. Runtime authority has not changed at this
preflight milestone.

The upgraded callback fixture then reproduced the physical ordering after all
seven managed commits. On the unchanged predicate it reached a confirmed exact
Reset re-receipt but failed at `exact marked surrounding Space frame`, proving
that the regression detects the first measured authority loss. That remote run
also contained one unrelated peer-queue transport error, so it is retained as a
causal RED rather than a denominator PASS:
`/home/ubu/.cache/lay/development/run-f7bi260e/RESULT.json`, SHA-256
`76de88c486ccf26019bb7447b7eb36f207e42b5a08e553bc9427fee639c1c345`.
After adding only the selected predicate alternative, the hermetic engine
target passed all 587/587 tests. Its receipt is
`/home/ubu/.cache/lay/development/run-2rj32nmt/RESULT.json`, SHA-256
`e008b4adbab3618a9a3728e61b978234797e2644892bd6d13934fb851f658d87`.
No runtime artifact was built or installed at this proof milestone.

The first complete changed-source run did not establish a denominator PASS.
All listed targets completed, but the engine target reported the pre-existing
`td121_completed_replay_prior_surface_cannot_hide_contradiction` fixture as an
unexpected failure. Its effect collector received a zbus `UnknownObject` error
reply for synthetic release serial 21863 instead of an engine signal. The same
transport error had appeared once in the causal RED run; it is independent of
the Firefox predicate and the new Firefox fixture passed. The failed full-run
receipt is `/home/ubu/.cache/lay/development/run-p3wd6ehg/RESULT.json`, SHA-256
`30f977b8845937dd1a37235cd862776765b94c525604463043ecd0cfcbd6cc8d`.

Test-infrastructure preflight: semantic legacy callbacks are deliberately sent
to the observer and then executed directly on the fixture engine. Once zbus's
object dispatcher has been activated, it can asynchronously answer that
detached path with `UnknownObject` even though the synthetic callback carries
`NoReplyExpected`; the reply may arrive after the direct engine signal. The
selected repair records each callback serial issued by the common `legacy_key`
helper in the controlled peer and makes the common peer reader discard only an
`UnknownObject` error whose reply serial is in that exact pending set. All other
errors and unregistered reply serials remain assertion failures.

Broadly ignoring D-Bus errors in `legacy_effects` is rejected because it could
hide an unrelated harness failure. Adding sleeps or retrying the failed test is
rejected because arrival order, rather than product state, is the demonstrated
mechanism. Registering a second fake engine object is rejected because it would
create a second callback executor beside the directly exercised production
method. The selected change is test-only state, adds no runtime owner, cache,
worker, package or deadline, and cannot affect candidates, ranking, authority,
learning, resource use or installed behavior. Its rollback boundary is the
controlled-peer pending-serial set and common reader filter. The full gate must
be repeated after the repair; the failed run is not accepted as evidence.

The first implementation applied the pending-serial filter in the common
reader, including a helper that intentionally asserts the raw detached-object
reply. That helper consequently waited for a message already filtered out, and
the engine target failed one timeout. This rejected test-infrastructure attempt
is `/home/ubu/.cache/lay/development/run-da87dgxv/RESULT.json`, SHA-256
`cd2a9be56bce608a2a969b39517c6900a0f3d6e356db4753868ee56486e991dc`.
The repair now provides a raw reader only to the explicit transport assertion;
both raw consumption and semantic filtering remove the exact registered serial.
All other readers retain the bounded filtered contract. The engine target then
passed 587/587 at
`/home/ubu/.cache/lay/development/run-nrrutq66/RESULT.json`, SHA-256
`204dbf9d323be5c51ec91d0e5ecb745bb3bf2ef20c4c057f4cf0100ea2c15d9b`.

#### Exact-receipt scheduling latency and pending-Reset computation preflight

The repaired predicate and the test-only peer filter passed the complete remote
changed-source gate: all 2,888 selected correctness and package cases passed at
`/home/ubu/.cache/lay/development/run-yhdioapm/RESULT.json`, SHA-256
`137f03b235b640cc95950a960bd557a26e339b3b6adfd98507356b81b758de95`.
The guarded release build produced the 7,911,144-byte engine at
`/home/ubu/.cache/lay/development/firefox-reset-rereceipt-20260922/lay-ibus-engine`,
SHA-256
`495b445616cff016407df690f01972f4ac440dfd1f0e38351dd626978a852c02`.
Its build log has SHA-256
`6c9d60334b1804aa60232ce3a2f96f7efe69607fe5f9592f5db8788a3c17b5e7`.

Only that engine was installed. The engine PID changed from 901820 to 3443478;
`lay-daemon` PID 428182, `ibus-daemon` PID 4062416, the selected Russian source,
and the original Firefox PID 635409 with start tick 166280532 remained unchanged.
Installed and loaded hashes both equal the candidate hash. The unchanged adapter
hash is
`924b59822e26f5ad3e6e0daf2a8f762b15e6905bb0b3683ce56bee55744a6f22`.
The installation receipt is
`/home/ubu/.cache/lay/development/firefox-reset-rereceipt-20260922/installation.json`,
SHA-256
`d10a2c87c01346f0c5376f0aa305d7f75cb0796f29155c23f1da00dd5f4dc8c9`.

The second isolated Firefox 156 physical run again left exact `публекует `
instead of `публикует `. Its FAIL receipt is
`/home/ubu/.cache/lay/development/firefox-reset-rereceipt-20260922/physical-firefox-run2/receipt.json`,
SHA-256
`5ac3c1ccf6a7b321da45aba58294bf875b017cd9a89fe3f4e73a18dfd00df100`.
The original Firefox identity was preserved and the isolated profile was
removed. The requested interface still passed: every printable scalar remained
ordinary committed text, the engine composition remained empty, and only a
completion suffix could be displayed as preedit.

The trace establishes a second, later first loss. After the final printable
commit, the strict Reset witness advanced to confirmed tail epoch 11 with all
nine suffix scalars. The exact unselected surrounding snapshot then arrived and
scheduled correction worker generation 2. Space arrived before that worker was
ready: the existing bounded lookup waited 3,451 microseconds,
`prefetch_not_ready` selected `managed_fallback_commit`, and the complete Space
callback took 3,740 microseconds. The worker finished only afterward with
61,263 microseconds of evaluation and was correctly superseded by the literal
Space mutation. The trace is
`/home/ubu/.cache/lay/development/firefox-reset-rereceipt-20260922/physical-firefox-run2/trace-after.jsonl`,
SHA-256
`bcc2656c40c6c18767e7ae95e1bb3ab37a0fa61f2f99ccf01a50ef0355463492`.
No delete or replacement commit was attempted, so this run does not implicate
adapter serialization, effect ordering, edit-plan validation, or visible
postcondition feedback.

Selected repair: allow marked Firefox to start the existing Space computation
after a printable callback has advanced the strict pending Reset identity, and
before the client supplies its exact snapshot. The speculative frame uses the
same word-frame identity and live admission token as the later exact Space
frame. It grants computation only. Scheduling must require the private exact
refresh marker, SurroundingText support, and
`context_reset_rereceipt_computation_allowed()`. Capture and Space acceptance
continue to require `exact_marked_surrounding_suffix_is_current()`, including
the exact unselected snapshot and confirmed current Reset re-receipt. A missing,
late, mismatched, or invalidated receipt therefore leaves the prepared result
without edit authority. When the later exact callback submits the same complete
identity at the same candidate-material generation, the single worker preserves
its current pending or terminal slot instead of replacing it with a new
generation. A changed identity or material generation still supersedes the old
slot, and the existing lock-contention path still fails closed.

Alternatives rejected at this measured boundary:

1. Increasing the Space wait would add user-visible hot-key latency and would
   still make success depend on an unbounded cold computation.
2. Scheduling for every marked suffix without the pending Reset identity would
   perform work for stale or externally changed text and weaken the measured
   one-shot provenance boundary.
3. Treating the pending frame as edit authority would bypass the exact client
   receipt that bounds deletion. Returning the complete token as preedit would
   also restore the interface the user rejected.
4. Recording a second engine-side “scheduled” flag would duplicate worker state
   and create an ABA/invalidation obligation. Idempotent registration at the
   existing slot keeps one source of truth and also covers any equal-identity
   duplicate callback.

Consequence analysis before the next production edit:

- **Candidate, rank, verifier and safety:** candidate generation, lattice,
  scores, verifier, `SafetyGate`, edit plans and visible postcondition checks do
  not change. The same computation is merely started at the earlier strict
  Reset witness. Exact snapshot authority is still re-evaluated at Space.
- **Latency and work:** the final printable callback may enqueue one existing
  Space worker roughly one key interval earlier. No timer, retry, longer Space
  wait, second worker owner, queue or cache is added. Later exact-snapshot
  scheduling reuses the equal current slot. An already consumed slot, different
  identity, configuration, focus, tail or candidate-material generation still
  creates a new request. The causal fixture must prove that the pending and
  exact identities are equal and that exact publication preserves a completed
  lease.
- **CPU, RSS and allocation:** marked Firefox may perform a correction
  computation that is never consumed if its exact receipt never arrives.
  Focus, tail epoch, token, marker, capability, layout or configuration changes
  invalidate that work through the existing identity. This is bounded to one
  current job and the existing result slot. An equal duplicate now avoids
  repeating inline exact preparation and full evaluation; package or online
  candidate-material generation changes force replacement rather than reuse.
- **Display and ownership:** the committed prefix and empty engine composition
  remain unchanged. The pending Reset path already permits display-only
  precognition under the same computation witness; this change does not grant
  display publication or turn the suffix into owned preedit.
- **Packages, learning and feedback:** packages, delta reload, learning and
  outcome feedback are untouched. Routing completion remains separate from
  answer quality, which is `UNKNOWN` without the fixed heldout proof.
- **Compatibility and rollback:** unmarked clients, terminals, Chromium owned
  preedit, atomic input and manual-toggle authority keep their existing
  predicates. Rollback removes the pending marked-Space frame and the one
  post-settlement scheduling alternative. The compatibility route can be
  removed with the exact-refresh marker once Firefox supplies timely native
  receipts.

The next causal proof must withhold the final exact snapshot, show that the
pending strict Reset witness already scheduled an identity-equal Space job,
then publish the exact snapshot and prove that a zero-budget lookup is not
stale. It must still prove per-scalar commits, no whole-word preedit, one exact
delete plus replacement commit at Space, and no edit when the receipt is absent
or invalid. After the focused engine target and full changed-source denominator
pass remotely, rebuild and install only the guarded engine and repeat the same
physical Firefox timing. Runtime authority at this preflight milestone remains
the installed candidate hash above; the speculative route has not yet been
built or installed.

The test-only causal run selected all 587 engine tests and passed 586. Its only
failure was the upgraded Firefox fixture at the assertion that the derived
pending Reset Space identity already owns the current worker slot. Production
code had not yet been changed, so the exact prior-character job remained in the
slot. This proves the fixture detects the measured late-scheduling mechanism
before any receipt-preservation assertion or text effect. Receipt:
`/home/ubu/.cache/lay/development/run-f0g8hocd/RESULT.json`, SHA-256
`16e8d01d5a031be33db6b9092fc58ab3061f5aaaa67cbdeb90ac403d3419f097`.
This is a causal RED, not denominator acceptance; installed runtime authority
is unchanged.

The first implementation attempt added the pending-frame capture, post-key
schedule and equal-slot reuse, but the focused target remained 586/587 at the
same pre-receipt slot assertion. Receipt:
`/home/ubu/.cache/lay/development/run-20k7aykf/RESULT.json`, SHA-256
`e21fb8d85d4597ee7e41ef94724c7481bf0f275432bd714441f80b6fad6b1dfd`.
The schedule entrypoint was reached with the intended frame, but its shared
identity predicate deliberately required current edit authority and rejected a
pending receipt. The repair therefore separates computation admission from
acceptance: scheduling additionally accepts only an identity-equal
`capture_pending_reset_space_frame()`, while lease consumption and text effects
continue to call the original exact-authority predicate. This uses the same
pattern already established for pending display computation and does not widen
edit authority. The failed run is retained as implementation evidence, not a
PASS; runtime authority is unchanged.

After the computation/acceptance split, the fixture passed both new slot
assertions and reached the final release callback. It then failed because the
older fixture expected an unhandled post-Reset release, while this final cycle
deliberately withholds Reset and follows the measured physical order: managed
press, handled managed release, then exact surrounding receipt. Receipt:
`/home/ubu/.cache/lay/development/run-d695y21y/RESULT.json`, SHA-256
`14170cfb8e55922ded554b9e08383c061f6eadbb0113464aa8fd37c03d79afaa`.
The fixture expectation is corrected to require the handled final release and
no text output before publishing the exact receipt. Production behavior is not
changed for this harness correction, and the failed run is not a PASS.

The corrected focused run passed all 587/587 engine tests. The causal fixture
proved the pending pre-receipt slot, equality of pending and exact identities,
preservation of a completed full lease across the exact receipt, the measured
handled-release ordering, per-scalar committed text, absence of whole-word
preedit, and one authorized delete plus corrected commit at Space. Receipt:
`/home/ubu/.cache/lay/development/run-a1r3c6gq/RESULT.json`, SHA-256
`e3c320bcfa3ffb549ba327ec343c1c00bf645f54f2678c5e4a5c5fdad8d66569`.
This is focused development evidence; the full changed-source denominator,
guarded release build and physical Firefox proof remain pending. Installed
runtime authority is still candidate
`495b445616cff016407df690f01972f4ac440dfd1f0e38351dd626978a852c02`.

The complete changed-source gate then passed all 2,888 selected correctness and
package cases in 372.6 seconds. Receipt:
`/home/ubu/.cache/lay/development/run-gi6fwy3l/RESULT.json`, SHA-256
`155fc5a3e161a0078a5e75b04567dc97665977d775f77cd9627664c10aa6a755`.
This establishes the current source denominator under the remote guard; it does
not yet establish built-byte identity or physical Firefox behavior. Runtime
authority remains unchanged.

Independent review rejected promotion at 7/10 for one medium liveness race and
two proof gaps. The worker sampled candidate-material generation before taking
its slot lock; a package update in that interval could make an equal old slot
look reusable. Space would still reject the stale lease, so edit authority was
fail-closed, but the exact callback could lose its chance to enqueue current
work. The selected repair reads material generation while holding the existing
slot lock and rechecks it before returning `Reused`; an interleaving proof must
force `old -> new` between those reads and require a newly registered pending
slot at `new`. A material change after that final observation remains safely
stale at consumption and requires a later input/snapshot schedule, which is the
existing package-update contract. No package owner, notification route or
retry is added.

The Firefox fixture also disabled display precognition and only rejected a
preedit payload equal to the whole visible token. It will now require every
preedit update in this correction fixture to be empty, proving that the prefix
stays ordinary committed text here; suffix-only completion display remains a
physical-browser assertion because asynchronous candidate publication is a
separate route. Before supplying the final exact receipt, the fixture will
consume an injected ready lease directly through the production correction
admission and require refusal, zero text effects and an unchanged tail, then
reinstall the lease for the positive exact-receipt path. This proves the pending
witness grants computation but no edit. Candidate generation, rank, verifier,
deadlines, allocation bounds, packages, learning, feedback, compatibility and
rollback remain as analyzed above; only generation observation and proof
coverage change. Runtime authority is unchanged.

The first post-review invocation stopped at remote `cargo fmt --check` before
tests because three edited expressions needed canonical wrapping. It is a
formatting failure with no runtime evidence:
`/home/ubu/.cache/lay/development/run-1053zby3/RESULT.json`, SHA-256
`9dbe974c1e94d980e848f3b4209920366b11a7414ca7f642addf5e1d19de6a5b`.
After applying exactly that formatter delta, the focused engine target passed
all 588/588 tests at
`/home/ubu/.cache/lay/development/run-o402xlrc/RESULT.json`, SHA-256
`4deced669f1c40214e68031e65ebb9f0f8cf46b1032bb31a770e7b2c1dd1a7aa`.
This run includes the controlled material-generation interleaving, pending
ready-lease refusal with zero effects, exact-receipt positive correction,
per-scalar commits and empty preedit publications. The earlier 2,888-case full
gate predates the review repair and is superseded for final acceptance. A fresh
full denominator and second independent review remain required. Runtime
authority is unchanged.

The first fresh full-gate attempt stopped before executing product tests because
the new controlled interleaving test was absent from the canonical manifest.
Self-tests passed and discovery reported exactly one added identity. Receipt:
`/home/ubu/.cache/lay/development/run-lwpqug4b/RESULT.json`, SHA-256
`ff6d7a589ab4607128443f22ddc3df914a91dac3bfdd6b35e2b565114d8bb13b`.
The guarded remote `write-manifest` operation then added only
`bin:lay-ibus-engine::space_autocorrect_prefetch::proof::equal_slot_reuse_rechecks_material_generation_under_the_slot_lock`;
zero identities were removed or changed. The canonical manifest now contains
2,915 tests: 2,853 correctness, 36 package, 11 performance and 15 ignored. Its
SHA-256 is
`de3f7a50419db26a3fa9acff23f32794b984659ee81a457c6628d225d4bce552`;
the refresh log is
`/home/ubu/.cache/lay/development/firefox-reset-rereceipt-20260922/manifest-refresh-review-repair.log`,
SHA-256
`d18cb748ce0297934390803ab026ea1031844a4a6109beb1dbf6ab4e376ec60a`.
No runtime artifact or authority changed. The full gate must be repeated against
this manifest.

The second independent review found no remaining defects and scored the
reviewed delta 9/10. It confirmed that slot registration observes and rechecks
material generation while holding the slot lock, that publication and
consumption remain fail-closed after any later material change, and that the
Firefox fixture now proves both pending-receipt refusal and exact-receipt
acceptance with empty preedit output. This source review does not replace the
full denominator or physical Firefox proof. Runtime authority is unchanged.

The next full-gate attempt executed all discovered correctness/package targets
with zero observed test failures, then stopped at the final lane contract
because the empty known-failure ledger still named the previous test-manifest
SHA-256. Receipt:
`/home/ubu/.cache/lay/development/run-b7siunv4/RESULT.json`, SHA-256
`e846792fed770c62501c916e582194a981fe4c9dae230c4fcffbffdfed751bd5`;
run log SHA-256
`0a6dac0af7ebf99da69f53033c3028123745da9cc69e1c94ed4f9125c2788155`.
This is a contract failure, not a PASS. The ledger still has zero rows and the
fixed zero-failure observation is unchanged; the bounded repair updates only
its `manifest_sha256` binding from the earlier manifest to the already audited
`de3f7a50419db26a3fa9acff23f32794b984659ee81a457c6628d225d4bce552`.
No failure was suppressed, no test identity or lane changed, and runtime
authority remains unchanged. The complete gate must be repeated from the
beginning.

The repeated complete gate then passed all 2,889 selected correctness and
package cases in 359.1 seconds. Receipt:
`/home/ubu/.cache/lay/development/run-x6evt17z/RESULT.json`, SHA-256
`142aa6ea35ed0d698d600c5778aa536ea11c8462a716da00c071354615e34d03`;
run log SHA-256
`35ee437421a8ad2916bf8f75e751d2e987fc038796bfe3807b3f633fa96e4af4`.
The guarded release build from that exact remote workspace produced a
7,912,168-byte engine with SHA-256
`a39c55b962102954a29289a17d433ff7aae27b8a251526f309b2a65d859a5aab`.
Its build log SHA-256 is
`3b3f4a5a6a5daf3ec1ef3cff6074dd4c36b440d73c9839a5832915a98632bedb`.
Only that engine was installed. The engine PID changed from 3443478 to
3810998; `lay-daemon` PID 428182, `ibus-daemon` PID 4062416, the Russian input
source and the original Firefox PID 635409 with start tick 166280532 were
preserved. Installation receipt:
`/home/ubu/.cache/lay/development/firefox-pending-reset-prefetch-20260922/installation.json`,
SHA-256
`916bee2a11051023d4507a0157e28ff186cb8d6ca1e39ac0ab43b27072825ebe`.

#### Delayed strict-prefix receipt and Reset-token rotation preflight

The first physical run of that candidate still left exact `публекует ` rather
than `публикует `. Its receipt is
`/home/ubu/.cache/lay/development/firefox-pending-reset-prefetch-20260922/physical-firefox-run1/receipt.json`,
SHA-256
`f8c1801fec8b3e343261791ef269a8212bec040f4277035a58302819b63ec300`.
The committed-prefix interface itself passed: Firefox observed zero composition
starts or ends, every printable remained a managed scalar commit, engine
composition stayed empty, and trace preedit publications were suffixes such as
`олучить` and `сть`, never the whole typed token. The isolated profile was
removed and the original Firefox identity was preserved.

The selected trace at
`/home/ubu/.cache/lay/development/firefox-pending-reset-prefetch-20260922/physical-firefox-run1/trace-selected.jsonl`,
SHA-256
`78858aebf8a5427a0cace142342b9608465824847336176a02802c237501ed55`,
places the first remaining loss after early scheduling and before Space frame
capture. The final managed press advanced the strict pending Reset identity to
tail epoch 96 and registered worker generation 9. Before the release, Firefox
delivered a delayed strict-prefix surrounding snapshot (`text_chars=5`,
`cursor_pos=3`). Because the pending witness had inherited `confirmed=true`,
the generic second-receipt branch discarded it. The following authenticated
Resets therefore reported `missing_predecessor_or_post_reset_token`; exact
prefix receipts advanced through 6, 8 and finally 9 visible characters, but no
strict predecessor lineage remained. Space consequently had no admissible
frame and used `managed_fallback_commit` in 76 microseconds. No correction
lease was consumed and no delete was attempted, so this does not implicate
candidate quality, ranking, verifier, edit validation or Firefox effect order.

The selected systemic repair has two conjunctive parts. First, a confirmed
pending Reset witness receiving exactly the next observation revision may
retain a boundary-bounded strict prefix of its current token, but it immediately
becomes unconfirmed. It grants neither display nor edit authority and can
recover only through a later authenticated Reset plus an exact full snapshot.
Selections, non-prefix text, owner/scope/token mismatch, revision gaps,
commands, external input and lifecycle revocation retain their existing
fail-closed paths. This reuses the strict-prefix predicate already accepted for
the unconfirmed first receipt.

Second, the marked pending and exact Space frames use the pending Reset chain's
stable `predecessor_token` as their computation identity across authenticated
Reset token rotations. The current live token is still required and revalidated
by the pending witness, owner, scope, admission reducer, tail epoch and exact
snapshot predicates at capture and again at Space. The predecessor token is
therefore a job identity only; it cannot authorize deletion. Without the
current exact full snapshot, `capture_space_autocorrect_frame_identity()` still
returns no accepted frame. This lets the already running final-token job remain
identity-equal after Firefox's real post-release Reset sequence instead of
starting another cold generation at the final receipt.

Candidate generation, lattice, scores, verifier, `SafetyGate`, edit plan,
replacement transport, packages, learning and feedback do not change. No
timer, retry, second worker, queue, cache or adapter behavior is added. CPU and
RSS remain the existing one current worker job and one result slot. The causal
fixture must now reproduce the measured order: final managed press, delayed
strict-prefix receipt, handled release, one or more authenticated
Reset/strict-prefix pairs, exact full receipt, identity-equal completed lease,
then exactly one authorized delete and corrected commit. It must also preserve
the existing proof that the same prepared lease produces zero effects before
the exact receipt. Contradiction and input-gap negatives remain mandatory.
Runtime authority is currently the installed failing candidate
`a39c55b962102954a29289a17d433ff7aae27b8a251526f309b2a65d859a5aab`;
no new production source has been edited at this preflight milestone.

The upgraded causal fixture then selected all 588 engine tests and passed 587
on unchanged production code. Its sole failure was
`terminal_delivery_firefox_exact_refresh_keeps_prefix_committed_and_schedules_space`
at `delayed strict prefix must retain inert Reset lineage`, before any Reset
rotation, Space lookup or text effect. Receipt:
`/home/ubu/.cache/lay/development/run-7ctw3fnk/RESULT.json`, SHA-256
`05e20a3a048eaf026501e183023a776c595d5d00b8639a343ca107af0482beb9`.
This is the required causal RED for the measured first loss, not release
acceptance. Runtime authority remains the installed failing candidate above.

After strict-prefix retention and stable predecessor job identity were added,
the causal fixture crossed both authenticated Reset token rotations and proved
the pending frames remained equal. It then failed only at the assertion that
the final exact receipt still owned the completed full-worker slot. Receipt:
`/home/ubu/.cache/lay/development/run-41ns47zf/RESULT.json`, SHA-256
`a3439c01d80f7f42be1bccc6fec177e39817c645dc8a91000027e65a6c7766e0`.
Inspection places this third loss in `reset_for_ibus_soft_reset()`: every Reset
unconditionally calls `invalidate_input_frame_background_work()`, so the
authenticated lineage survives while the identity-equal worker slot does not.

The bounded repair captures the current pending marked-Space identity after
the Reset reducer has installed its successor token and before soft-reset
cleanup. Display precognition is always cancelled. The existing Space worker
retains its slot only when path, complete identity, request generation, latest
generation and current candidate-material generation all still match while
holding the slot lock; otherwise the path is retired exactly as before. No new
slot, flag, retry, timer or owner is created. Focus loss, Disable, unmarked
clients, missing pending lineage, composition ownership, sensitive content,
tail/config/layout drift and material reload retain unconditional invalidation.
Publication and Space consumption continue their independent current identity,
material and exact-authority checks. The causal test must assert slot survival
after each authenticated Reset as well as exact-receipt reuse and the final
single text effect. Runtime authority is unchanged pending a new focused PASS.

#### Managed-word-start delayed-snapshot successor

The installed exact-snapshot candidate preserved the continuous Firefox
suggestion but still received its final surrounding snapshot only after Space.
A controlled RED therefore started from one exact empty caret boundary,
committed `рабоает` through seven managed callbacks with no later snapshot and
failed only when Space could not capture a correction frame: 589/590 at
`/home/ubu/.cache/lay/development/run-t_clj0dc/RESULT.json`, SHA-256
`abe34f467d4fa9c3d2f537e9e36efe5d59e19436f725d1e274b90657d2fea1c7`.

The successor binds that exact boundary to the uninterrupted local CommitText
chain by focus receipt/serial, owner lease, layout generation, tail epoch and
text, exact snapshot geometry and observation revision. It permits one
source-free rebind before the first character. Each later character requires
one corresponding epoch advance. A request-scoped projected snapshot feeds the
existing edit and postcondition validators and is never installed as a client
observation. Equal-text ABA, focus/owner changes, selection, cursor motion,
external text, boundary input and capability/layout/content changes fail
closed. Candidate generation, ranking, DecisionCore, SafetyGate and edit-plan
validation remain unchanged.

The first focused result passed 590/590 at
`/home/ubu/.cache/lay/development/run-ecu1938a/RESULT.json`, SHA-256
`06f5ee634ad8eb7a8c4d34f1654898f9312175f917fd502b4a8a3009e6c05e10`.
Independent review then found one high issue: command-modified, unsuccessful
Tab/navigation and generic non-printable keys could be handed to the client
without revoking the projected witness or the prepared Space slot. The client
could therefore change text or the caret before a delayed snapshot and expose
the old projection to a later Space. This review rejected the source checkpoint.

The systemic repair routes all client-owned mutation/navigation exits through
one revocation helper that clears the witness and invalidates current Space
work. One grouped causal test covers command input, Tab, candidate navigation,
generic navigation and cursor movement; after every returned key, Space must
emit zero DeleteSurroundingText effects and only the ordinary space commit.
The repaired focused gate passes 590/590 at
`/home/ubu/.cache/lay/development/run-gvlb0oqw/RESULT.json`, SHA-256
`78b097aa1a30025a0784c349bf6278cfc43193b15c655de23551e64c01bdac07`;
run-log SHA-256
`62bc9b2fb9922cf46a6a0d89eeb7fbcc6cbf4220beac1b340fb84dd7d097c2bb`.

Tested: delayed-snapshot positive correction, projected geometry, the existing
contradiction matrix, all 590 focused engine tests and the five client-owned
key classes above. Not tested by this result: the complete project gate,
release bytes, installation, physical Firefox, physical Chrome, Tor, GTK or
Kitty. Verdict: **focused PASS; successor review and full/physical gates still
pending**. Runtime authority changed only within the bounded rule that one
exact managed word start plus its uninterrupted, non-client-owned local commit
chain may authorize the existing correction transaction.

The next review round found the same revocation mechanism bypassed by two
outer `process_key_event_with_output` exits that run before managed dispatch:
failed standalone Alt/ISO-level3 completion release and disabled live
composition. This was one medium finding and rejected the checkpoint. The
repair applies witness-plus-path revocation to pre-managed client exits,
including native exact replay, while preserving harmless Shift observation.
Wrapper-level negatives install the old authorized lease first, exercise
backend disable/re-enable or standalone Alt release, and prove the witness,
frame and ready slot are absent before Space produces no deletion and one
ordinary space commit. The repeated focused gate passes 590/590 at
`/home/ubu/.cache/lay/development/run-ekb6jas1/RESULT.json`, SHA-256
`ad9442d37e357d27f1c6a9aede2cd0ae6695961dc0efe16c91277da2627dcfa9`;
run-log SHA-256
`b9f264e409f61b8779bfe351db34d64d3927cef228b52cf05107d04bc1c4b972`.
Final source review remained pending at this checkpoint.

Final read-only review passed **9/10, H0/M0/L0** and accepted the bounded source
successor. Its durable report is
`tech_debt/evidence/browser-delayed-snapshot-source-review.md`; the protected
composition chain terminates at
`tech_debt/evidence/browser-delayed-snapshot-composition-successor.json`.
The TD-113 contract passed 7/7 at
`/home/ubu/.cache/lay/development/run-35r44hn_/RESULT.json`, SHA-256
`2fb30e957372deb22df89834135fbc345670b98fb18e73685e21da99844e4e59`.

The complete project gate then passed all **2,891/2,891** selected correctness
and package cases in 352.9 seconds, after validating the 2,917-test manifest.
Result:
`/home/ubu/.cache/lay/development/run-7pv91b6z/RESULT.json`, SHA-256
`2eac3f79e371ff4f1d60bff13120462c4db5a071c932b238fb4aed9f4db9c299`;
run-log SHA-256
`1c7d4b5921ebe71a8b3e39046dca20daa1da27358b218184799c7c66ec3fda73`;
source archive SHA-256
`d4e4acff902d0bd0613504578c52105a2a02dbe357e728371a0a87a7d9b9e62f`.
Verdict: **source review and full development gate PASS**. Release build,
installation and physical Firefox/Chrome acceptance remain pending.

### Accepted browser result, 2026-09-23

The later browser display/Reset-release source passed the fixed development
gate **2,893/2,893** (`/home/ubu/.cache/lay/development/run-2nrara15/RESULT.json`,
archive SHA-256 `889c4774012d2924be4b6734b244e26ff646e90ed158924edfcd7c81eb282dba`).
The exact archive-built engine SHA-256
`daaa47bf400b8fb06d124a31c0790422f8a830aa26cecfdfd8152b2687687b88`
is installed and loaded. Firefox passed continuous suggestion plus
`публекует ` → `публикует ` in one physical run. Ordinary Chrome passed the
same two conditions in one fresh field and repeated that same-word run twice
(receipts `physical-chrome-final-same-word/receipt.json` and
`physical-chrome-final-same-word-repeat/receipt.json` under
`/home/ubu/.cache/lay/development/browser-autocorrect-20260922/`). Its
separate Tab-to-second-field probe still left `публекует ` unchanged. An attempted
surrounding-capable owned-preedit route was rejected after **10 unexpected
fixed-test failures** and reverted exactly to the passing source. The owning
architecture record, exact physical receipts, authority limits and untested
dimensions are in `docs/ime-daemon-route-map-2026-06-20.md` under “Accepted
browser display source and installed artifact”. Publication was not requested.
