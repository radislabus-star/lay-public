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
