# Terminal autocorrection boundary loss, 2026-09-10

Status: verified repair installed locally; physical GNOME/Kitty acceptance pending.

The user confirmed that IME suggestions work after starting a new Kitty
process. They separately report that autocorrection can consume the preceding
space when the original and replacement have different lengths, and question
the insertion cursor position. This acceptance does not close autocorrection.

## Live evidence and first unresolved boundary

At the pre-repair capture, the installed IME was the accepted 1.0.70 binary
`f4d3c8e256a`; the daemon was `133a1f79e57b`. On that capture Kitty PID705674 maps the installed corrected
consumer module, inode57933842 / SHA `9cbbe9f79568`; another older Kitty process
still exists. Do not attribute the new report to that older process.

Private evidence:

- `~/.cache/lay/development/ime-space-20260910-exb148px/diagnosis.json` records
  the earlier reported `провреь` to `проверь ` edit and its seven-character
  deletion plan. Its exact delivery callbacks were already overwritten.
- That directory's `plan-length-audit.json` found 25 distinct active-composition
  edits: all delete counts equal the original Unicode character count; two
  shorten, one lengthens, and 22 preserve word length. These are plan facts,
  not observations of the receiving application's text.
- `~/.cache/lay/development/ime-replacement-cursor-20260910-7xc45bzi/` preserves
  the new runtime identity and log snapshots. A bounded 180-second capture of
  the existing log received no new writes. It changed no runtime configuration.

`committed_tail.rs` binds the selected action to the current input identity and
requests the original token's character count. `state.rs` executes the
authorized plan and updates its mirror using the original logical length and
complete replacement. Existing TD125 Readline tests already cover growth,
equal length, shortening, prior completion, and intentional over-deletion.
They explicitly do not prove legacy IBus delivery.

For a terminal without SurroundingText, the actual text/caret is not read back:
SetCursorLocation records x/y for diagnostics and stores cell width only. The
first unresolved boundary is the authorized edit and internal mirror versus
the bytes and caret actually accepted by the terminal. There is no measured
L1.1/L2/L3/L4 candidate-retention or ranking loss in this investigation.

## Conditional delivery mechanism

The host has GNOME Shell50.1 and `libmutter-18-0 50.1-0ubuntu2.2`. Exact upstream
50.1 source was saved under the new evidence directory's `upstream-source/`.
Its commit callback sends each text string and defers Wayland `done` using
the existing idle source at `CLUTTER_PRIORITY_EVENTS + 1`; successive IM events
can share that pending idle. Kitty0.48.2 stores one pending commit string.

The official Wayland protocols1.45 archive was verified against the pinned
SHA `4d2b2a9e3e099d017dc8107bf1c334d27bb87d9e4aff19a0c8d856d17cd41ef0`.
Its text-input-v3 specification defines pending state applied at `done`.
Concatenating arbitrary pending commit strings in Kitty is not justified by
that contract.

Hypothesis: `CommitText(last typed character)`, followed by
`CommitText(DEL × original_length + replacement)`, before one `done`, loses
the first commit. The mirror then exceeds visible text by one character and
the deletion can consume a separator. This mechanism does not inherently
favor shortening. The original incident's Wayland packet order is unobserved;
the claim must remain conditional until there is a discriminating proof.

## Bounded experiment preflight

Use unchanged extracted Mutter and Kitty callback bodies, the real GLib idle
scheduler, and the existing GNU Readline consumer. Generate terminal framing
from the unchanged Lay framing function. Compare identical inputs delivered
in one pending-state batch versus a `done` between commits. Assert bytes
delivered to the child, exact left-prefix/separator preservation, and the
cursor-marker position for growth, equal length, shortening, and a previous
shortening edit. Record the extraction and source hashes with the receipt.

This is a controlled callback/scheduler experiment, not an actual desktop
compositor session or physical keyboard acceptance. A failing grouped order
and passing separated order would identify a conditional delivery defect;
it would not identify the packet order of the already overwritten incident.
An intervening `done` and a complete visible source before deletion would
falsify this explanation for a captured incident.

All compilation and experiment execution must use the existing remote
dedicated-20cpu resource lease. Evidence belongs under the private incident
directory; no global IBus, Kitty window, input source, model, runtime authority,
SafetyGate, verifier, candidate set, learning state, or deadline changes.
No production design is selected before this experiment. Any later runtime
proposal needs its own consequence analysis, independent review, and affected
contract validation. Repeating the accepted broad test suite alone cannot
resolve this missing delivery evidence.

First execution stopped during C compilation in the private output-sink glue:
GCC's fortified `strncat` warning was an error. No callback scenario executed.
The sink now uses an explicitly bounded `memcpy` and terminator; no production
fragment, compiler warning level, payload, assertion, or event order changed.
Exact failed setup receipt:
`/home/e/projects/lay-development-runner/ime-replacement-cursor-20260910-7xc45bzi/delivery-proof-v1/receipt.json`.
The corrected experiment completed in 1.787221 seconds under the required
remote guard. Exact receipt:
`~/.cache/lay/development/ime-replacement-cursor-20260910-7xc45bzi/delivery-proof-v2/receipt.json`;
remote counterpart:
`/home/e/projects/lay-development-runner/ime-replacement-cursor-20260910-7xc45bzi/delivery-proof-v2/receipt.json`.

| Delivery order | Cases | Exact prefix/boundary/caret contract |
| --- | ---: | --- |
| Two distinct commits before one done | 12 | 0 pass, 12 fail |
| One done between the same commits | 12 | 12 pass, 0 fail |

Each order includes growth, equal length, shortening and the reported
transposition shape, across one separator, two separators and a preceding
successful shortening replacement. The real GLib2.72.4 scheduler emitted two
protocol commits in both orders. Grouping produced one done and one child
commit; separation produced two of each. The extracted production fragments
were unchanged. The copied, unchanged GNU Readline consumer supplied final
text and cursor-marker observations.

For `метка провре` followed by the final `ь` commit and the seven-delete
replacement frame, grouping produced `меткапроверь §`; separation produced
`метка проверь §`. The same one-character source loss explains all 12 grouped
boundary failures; no word-specific runtime condition was introduced.

Verdict: the conditional delivery defect is reproduced. The original desktop
incident's exact packet order remains UNOBSERVED. This result is not a model
quality score, an actual GNOME/Kitty desktop run, or a physical-input PASS.
Production runtime, authority, candidates, packages, gates and user processes
remain unchanged. A production repair still requires the consequence analysis
and independent review described above.

## Repair consequence analysis, before production edits

Independent design review identifies two additional proof boundaries: native
key delivery must precede deletion, and a cached legacy word mode must not
survive the first atomic activation. Cursor x/y notifications are presentation
updates, not text-delivery acknowledgments; waiting for them cannot prove either
boundary and would change the callback deadlines.

| Design | Consequence and selection |
| --- | --- |
| Current per-letter managed commit | Reproduced loss when the final letter and replacement share one done; keep as the negative control. |
| Reuse native terminal observation and existing Space correction | Selected subject to native-dispatch proof. Removes per-letter commit emission for explicit legacy terminals; keeps the existing correction executor, prefetch and hint owners. |
| Flush each commit in Mutter before the next commit | Viable protocol-conversion repair, but changes a system compositor and requires system-wide compatibility/deployment validation outside this application change. |
| Hold the whole word as active preedit until its boundary | Viable single-commit delivery, but changes the accepted native text/hint interaction and whole-word rendering. Reject for this repair. |

The proposed eligibility predicate is legacy input, explicit IBus terminal
purpose 10 and absent SurroundingText. IBus purpose 10 is not Wayland purpose
13. Preserve the existing narrow-cursor compatibility behavior for other
clients. Ordinary GTK and atomic input retain their baseline delivery modes.
Use the existing word mode, reselect it on purpose changes and first atomic
activation, and preserve the tested managed-word stickiness on capability
loss. Do not reinterpret each cursor notification as a transport transition.

For eligible native printables, observe `passthrough_visible_char` and schedule
the existing Space prefetch using the same captured frame before hint work.
At Space, reuse the existing prepared-lease and committed-tail correction.
An applied correction consumes Space and emits one erase/replacement frame.
Otherwise observe exactly one Space and return unhandled for native delivery.
The historical narrow-cursor branch outside this eligibility keeps its current
Space behavior. No fallback Space may follow an applied correction.

Consequence bounds and invariants:

- Candidate/lattice and ranking: no L1.1/L2/L3/L4, package, source, weighting or
  SafetyGate change. Keep typed identity, selected action, verifier, expected
  tail and backend authorization. Native observation uses the actual keysym,
  so engine-layout disagreement cannot substitute a different mirrored glyph.
  UnknownStart retains only its existing suffix hint/explicit-acceptance scope;
  it must not acquire full-word correction authority.
- Latency/CPU/RSS: reuse existing bounded per-path workers and callback-relative
  Space wait. No timer, acknowledgment wait, synchronous model evaluation,
  polling, queue or extra frame capture. Per-letter D-Bus commit allocation is
  removed in the selected branch; quantitative improvement is unmeasured.
  Keyval decoding and existing hint work remain on the callback path.
- Cache/packages/updates: retain input, owner, word, layout, output-capability,
  configuration and material generation validation and lease invalidation.
  A newer model/package does not certify client delivery or widen the output
  predicate. Native observation must not bypass stale-result rejection.
- Learning/feedback: reuse push-tail and boundary/edit feedback. Native
  observation and optimistic mirror updates are not visible postconditions;
  add no new learning confirmation. Preserve completion editing, suppression
  consumption and immediate autocorrect undo.
- Concurrency: the existing Mutter native replay bypasses IM filtering and
  flushes pending done before forwarding. Prove final-native-letter followed
  by correction, and correction followed by native typing and another
  correction, under controlled event order. GNOME/IBus asynchronous physical
  delivery and hint visibility still require actual-client acceptance.
- Compatibility: retain daemon-only legacy Double Shift detection, exact
  committed-tail execution, layout synchronization and atomic exclusivity.
  First atomic activation must retire any cached legacy-only choice. Purpose
  changes must not retain eligibility; capability gain retains its existing
  promotion/clear behavior and capability loss retains word stickiness.
- Failure/rollback: not-ready, stale, denied, disabled, suppressed and missing
  erase geometry preserve native Space. Correction failures retain existing
  error behavior; do not add a second mutation. Revert this connected source
  change or reinstall the preserved accepted binary. Do not restart global
  IBus or close user windows.
- Maintenance: replace per-letter emission with the already present observer
  for this capability class. Add one stateless eligibility predicate, no new
  authority owner, persistent state, worker, cache or alternative executor.
  A future compositor repair does not itself justify removing compatibility;
  removal requires transport/consumer proof with the then-current contracts.

Required proof denominators are separate: callback/scheduler delivery orders
and length classes; actual legacy callback handled/effect contracts; native
keysym and mirror surfaces; Space ready/refusal outcomes; purpose/capability/
atomic transitions; hint and manual-toggle lifecycle; affected integration
and package gates; actual client text/caret; final desktop/physical acceptance.
Tests must detect the old emitter or a stated controlled violation. Existing
model quality evidence is reusable only under matching source/package/config
identities; no new quality percentage follows from these transport proofs.

The native-dispatch experiment passed 24/24 exact text/caret contracts in
1.146093 seconds: 12 native-final-letter/correction orders and 12 consecutive
correction/native-word/correction orders, covering the same four length/error
shapes and three prefixes. It executes unchanged Mutter replay filter and
flush callbacks, unchanged Kitty commit callbacks, real GLib and GNU Readline.
The native glyph is a supplied sink input; GNOME's asynchronous IBus roundtrip
and Kitty's native glyph encoder are not executed in that experiment.
Exact receipt:
`~/.cache/lay/development/ime-replacement-cursor-20260910-7xc45bzi/native-delivery-proof-v2/receipt.json`.
The remote path has the same suffix under
`/home/e/projects/lay-development-runner/`.

Native experiment v1 stopped at compilation due to a proof-glue class-variable
name collision. Its exception reporter also had a tuple-key serialization
error. The failed compiler log and exact input remain under the corresponding
`native-delivery-proof-v1/` and `native-delivery-inputs-v1/` directories. V2
fixes only those two glue/reporting defects; production fragments, event order,
compiler warnings and assertions are unchanged. No v1 scenario executed.

The bounded transport mechanism is now sufficient to implement the selected
application repair. First run the new actual-legacy regression against the
unchanged production source, then change that source. Desktop attribution and
physical acceptance remain open; these callback proofs do not close them.

## Implementation and first contract checks

The connected source change reuses the terminal observer and Space executor,
reselects the cached mode at purpose/first-atomic transitions, and schedules
the existing prefetch before hint work. Candidate generators, packages,
SafetyGate, verifier, correction arithmetic and deadlines are unchanged.

The new real-legacy callback tests ran against unchanged production code:
463 selected, 461 passed and exactly the two new tests failed at the unwanted
wide-terminal managed character consumption. The narrow-cursor native control
passed before the wide-cursor failure. This is old-producer detection, not an
intentional corrupted-output oracle. Exact receipt:
`~/.cache/lay/development/run-ao2eo05h/TEST_SUMMARY.json`;
remote `/home/e/projects/lay-development-runner/run-5sy6v0/tests/SUMMARY.json`.
The focused run took 14.198161 seconds including format/build/discovery.

The first source check passes both new tests, including actual native glyph
mirrors and the first atomic frame. It reports 456/463 passed, with seven
remaining failures: five legacy terminal fixtures still require per-character
CommitText/handled=true, and two source-order tests search the old condition
spelling. Receipt: `~/.cache/lay/development/run-b2dh_pvi/TEST_SUMMARY.json`;
remote `/home/e/projects/lay-development-runner/run-MGSTih/tests/SUMMARY.json`.
These failures are grouped by the changed transport contract. Update the five
fixtures to require unhandled native input and a FIFO-fenced absence of text
mutation; retain their exact suffix, completeness, refusal, manual projection,
handoff and safety assertions. Preserve the two ordering assertions with the
new shared branch anchor. This check took 13.922238 seconds.

Additional actual-legacy cases cover purpose transitions, capability gain,
managed-word stickiness on capability loss, next-word rearming, and prepared,
not-ready, stale, disabled, suppressed and missing-geometry Space outcomes.
They assert native fallback or exactly one replacement frame plus release
ownership and final mirror. Their results are pending. No installation or
physical-input acceptance has occurred.

The expanded check reports 464/465 passed. The remaining failure is in the
new fixture's release assertion: its setup pressed the leading managed Space
without releasing it, so a later native Space release correctly consumed the
earlier handled-press marker. Complete that setup press/release pair; do not
change the production release owner. Receipt:
`~/.cache/lay/development/run-j3hl0x4a/TEST_SUMMARY.json` (14.345178 seconds).

The completed setup pair passes all 465 selected tests, with zero failures:
`~/.cache/lay/development/run-borw2hgh/TEST_SUMMARY.json`, remote
`/home/e/projects/lay-development-runner/run-bnyPTn/tests/SUMMARY.json`.
Elapsed focused check: 14.251360 seconds. The intermediate
`run-g5_cx23q` stopped at rustfmt before any tests; its requested formatting was
applied without changing semantics.

First independent source review: PASS, H0/M0; one low validation finding. The
Space unit matrix deliberately injects a prepared lease after typing with
assistance disabled. It cannot establish that native typing actually schedules
the worker or displays hints. Reviewed runtime SHA prefixes: engine
`25e635ad2f5f`, atomic `b6cbee334f90`, composition commit `344a525b3888`, managed
`80f3e2ddbfce`. Close that evidence gap before installation.

### Actual-client extension preflight

Extend the existing opt-in private IBus/Readline harness with one explicit
terminal-delivery lane. Enable correction and hints before the first native
letter. Test the three already observed correction shapes as one fixed set:
shortening (`дподпись` to `подпись`), growth (`средсва` to `средства`), and
equal-length transposition (`провреь` to `проверь`), with an exact left prefix,
boundary, Readline caret marker and subsequent native character. A fourth case
checks visible hints during native typing. These fixture strings never enter
runtime conditions or weights.

For the correction cases, observe the existing bounded diagnostic file using
the harness's file-monitor pattern. Require a published prefetch with the
exact current engine path and tail epoch before sending Space. This supplies
controlled key-after-ready order without injecting a lease or changing runtime
timers, workers, deadlines, packages or authorization. Record the observation
and its included trace-flush delay. It is not an immediate-Space, physical
keyboard, ordinary typing latency or heldout quality proof. A prepared refusal
must not be reported as a successful correction.

The client supplies native glyphs through the existing unhandled-key sink;
GNU Readline interprets actual replacement bytes and supplies its final line
and caret marker. This does not execute GNOME's native keyboard encoder or
its asynchronous compositor roundtrip. Keep the 24-case callback-order proof,
actual IBus effects, model verdicts and desktop acceptance separate. The
accepted private dependency bundle was rechecked against all nine current
installed dependencies: 9/9 exact hashes, recorded in
`~/.cache/lay/development/ime-replacement-cursor-20260910-7xc45bzi/current-dependency-parity.json`.


Native layout assumption check: both installed component descriptors advertise
`ru` for `lay-ime-ru` and `us` for `lay-ime-us`, matching source XML. Recorded
presses were decoded using the installed libxkbcommon: 96/96 and 79/79 native
keysyms equal the managed glyphs, including 95 and 73 Cyrillic presses. These
are overlapping historical snapshots, not 175 independent keyboard trials.
One non-JSON log line was excluded in each. Exact private artifact:
`~/.cache/lay/development/ime-replacement-cursor-20260910-7xc45bzi/native-layout-readback.json`.
The client now obtains valid keysyms through `IBus.unicode_to_keyval` rather
than using bare Cyrillic codepoints. Original scenario bodies are unchanged;
this corrects their shared input encoder. It still does not execute GNOME's
physical keyboard or Kitty's native encoder.

The independently reviewed composition successor is bound by
`tech_debt/evidence/ime-terminal-native-delivery-composition-successor.json`
and its frozen source-review report. The TD-113, TD-120, TD-121 and first-word
bindings remain unchanged. The protected integration contract checks each
historical link and the current reviewed source hash. This is source acceptance,
not installation or closure of the review's low validation finding.


## Final build and client evidence

Frozen full build:
`/home/e/projects/lay-development-runner/run-sNpxXU/terminal-full-acceptance-completed-identity.json`.
The fetched copy and both lane summaries are under the private incident root's
`full-acceptance/`. Candidate SHA-256:
`4bfe47fa3db15def7a4e993198b0505a3bbbacd7e05f3139094aaadce570690e`.
This is a local transport repair retaining version label 1.0.70, not a new
published tag. Its 699 Rust sources are bound by exact hashes.

Both mandatory changed and full gates passed 2730/2730: 2694 correctness and
36 package tests, with zero semantic or infrastructure failures. These are
repeated checks of the same denominator, not 5460 distinct cases. Discovery
adds exactly four declared tests; all 2752 previous identities, lanes, kinds
and isolation settings remain unchanged. Fifteen ignored and eleven optional
performance tests remain outside this acceptance. No new latency or model
quality promotion follows. Existing lint inventories retain all 535 default
and 359 research entries; only source locations were rebound. Cargo target
ended at 6,292,938,752 bytes against a 12,884,901,888-byte limit.

The frozen full sequence took 944.772 seconds, including architecture refresh,
manifest discovery, both gates and exact candidate build. The two Rust lane
executions took 325.712 and 325.842 seconds. Protected successor integration
passed. The first full attempt stopped before Rust execution because the
harness identity test still pinned the old driver hash; only the declared
successor hash was updated. That failed receipt remains in remote
`run-eukSEk/terminal-full-acceptance-completed-identity.json`.

The first actual-client series stopped before any text key: Python GI's
`IBus.unicode_to_keyval` requires a character string, not its integer ordinal.
No behavior case completed. The corrected shared encoder is directly exercised
by the existing actual-installed-GI regression for ASCII and Cyrillic. Its
source hash is `3bd1d4094c03b054872c01a5779a20f88118c55a5ff0bfa84976e6097b626c84`.
Focused proof-tool self-tests passed in 1.543 seconds:
`~/.cache/lay/development/run-h83frs25/RESULT.json` (102 tests, one explicitly
excluded real-cgroup integration). This proof-only successor does not alter
any of the 699 fully checked Rust files or rebuild the candidate. V1 remains
an immutable failed setup result, not a behavior denominator.

The accepted actual-client matrix binds six exact receipts and metadata files:
`~/.cache/lay/development/ime-replacement-cursor-20260910-7xc45bzi/terminal-client-acceptance/SUMMARY.json`.
Its SHA-256 is `7d09404d052286c9ec864ba1b06259930badb4a41777c2accfddd8cd062cf847`.
Each candidate, driver, nine-file dependency manifest, private config and
Readline consumer hash was checked. All private candidate processes were
removed and their IBus daemons reaped.

| Lane | Distinct cases | Startup schedule | Result |
| --- | ---: | --- | --- |
| Native terminal correction and hint | 4 | post-exact-ready, each correction Space after its exact publication | 4 PASS |
| Restoration | 5 | post-exact-ready | 5 PASS |
| Lifecycle | 3 | immediate | 3 PASS |
| Manual conversion and handoff hint | 3 | post-exact-ready, as in the accepted baseline | 3 PASS |
| Initial US word, eight manual conversions | 1 | independent immediate process | 1 PASS |
| Initial RU word, eight manual conversions | 1 | independent immediate process | 1 PASS |

For the three native corrections, the client asserts original text before
Space, exactly one replacement commit, exact left prefix and separator,
Readline's cursor marker, and subsequent unhandled native `а` with matching
mirror. The observed replacements are `дподпись` (8) to `подпись` (7),
`средсва` (7) to `средства` (8), and `провреь` (7) to `проверь` (7).
Readline outputs are respectively `метка подпись X`, `метка средства X` and
`метка проверь X`; X is the test cursor marker. The fourth case observes two
visible hint callbacks during native ` пров` input, without a text commit.
It proves visible delivery during typing, not a separate final-prefix
freshness or physical rendering contract.

Both correction and hints were enabled before typing. The existing worker
actually published the exact engine-path/tail-epoch lease: total queue plus
evaluation/publication time was 397602, 70811 and 92253 microseconds. File
notification observation added batching delay (992417, 953818 and 980196
microseconds). Those measurements are separate and do not prove ordinary
Space latency. No lease was injected, no input retried, and no production
worker, timer, deadline, package, ranking or verifier was changed. This closes
the first review's missing native-prefetch and visible-hint evidence.

One additional cold control remains a failure in both binaries: the private
orchestrator initially ran manual-toggle with immediate startup, whereas its
accepted historical receipt uses post-exact-ready. Two manual-conversion cases
passed, but the following GUI handoff hint timed out with zero material and
candidates. The same additional immediate scenario fails at the same third
case on baseline `f4d3c8e256a` and candidate `4bfe47fa3db1`. The accepted
post-ready candidate lane passes all three. Preserve the separate immediate
failures in `terminal-client-acceptance-v2/manual-toggle/` and
`terminal-client-acceptance-v3/baseline-manual-immediate/`; do not relabel them
as a cold hint PASS. The successful v2 4/5/3 lanes were reused, not repeated;
v3 supplies only the missing accepted manual and two first-word lanes.

### Installer review and bounded process proof

The prepared installer replaces only the exact verified IME binary, using a
saved baseline for rollback and retaining the global IBus process, other Lay
services, component names, configured sources and immutable model dependencies.
It uses the established temporary native profile handoff and restores the
captured Lay profile. Source review found a process-exit race in its draft:
post-SIGTERM `/proc/PID/exe` reads can fail during normal exit, including while
a zombie remains. The successor pins a pidfd, validates ownership before
signaling, and waits on that pinned process. If metadata disappears before
exit readiness, it waits within the existing bounded timeout without signaling
an unverified identity. Installation and rollback share that helper.

`installer-pidfd-receipt.json` under the incident root passes four extracted
helper contracts with real private child processes/pidfds: normal exit,
existing zombie, a controlled metadata gap before exit notification, and
wrong-hash refusal. Both preserved negative controls detect their respective
draft errors. The metadata gap is controlled proof ordering, not a sampled
kernel race. Elapsed execution: 0.057571 seconds under the remote guard.
No desktop process or installed file was changed by this proof.
`installer-acceptance-binding-derivation.json` binds the final aggregate-receipt
path change and proves the tested process functions are unchanged. Final
independent acceptance and the actual installation outcome are recorded below.


## Final review and installed result

Second and final independent review: **PASS, 9/10, H0/M0/L0**. Both the
native-prefetch/hint evidence finding and the installer exit race are closed.
The exact frozen review and its source/proof/installer bindings are
`final-review.md` and `final-review.json` under the private incident root.
This uses two formal review rounds; no unresolved review blocker was waived.

Installation completed at `2026-09-10T13:17:32.727160+00:00`. Only
`~/.local/lib/lay/bin/lay-ibus-engine` changed. The running IME PID
2182844, parent 4715, start ticks
`66758707`, and session bridge owner were verified against
the accepted candidate SHA. Global IBus PID4715 retained
start ticks `2261` and its original executable hash.
All 18 other files in the installed binary directory, daemon/L3/L1.1 process
identities, component files, configured input sources and all nine immutable
dependencies remained unchanged. No user window was closed.

Exact installation receipt:
`~/.cache/lay/development/ime-replacement-cursor-20260910-7xc45bzi/installation-terminal-ime.json`.
Rollback baseline is preserved at `/home/ubu/.local/state/lay/release-backups/terminal-native-20260910-7yeimo8o/lay-ibus-engine`.
Runtime authority changed only by activating the accepted IME successor.
No model, decision/verifier owner, package or scoring authority was promoted.
This is a local repair with version label 1.0.70; no new tag or public release
was created. The original main checkout's unrelated changes were preserved.

The real GNOME/Kitty keyboard result remains PENDING. The private native glyph
sink, callback-order proof and Readline observations do not substitute for it.
The immediate cold-hint failure and unmeasured immediate-Space latency retain
their separate scope. A final document/graph refresh records these outcomes,
checks architecture-contract and all 699 source hashes against the accepted
build, and preserves the tested binary without rebuilding. Its exact result
is `final-graph/ime-diagnosis-graph.json` under the private incident root.
