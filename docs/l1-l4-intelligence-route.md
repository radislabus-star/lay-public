# L1-L4 Intelligence Route

> Execution authority: `docs/phase-word-recovery-canonical-cutover.md`.
> This document remains a product-intelligence reference. It does not define a
> second runtime pipeline or an alternative L1-L4 ownership model.

Status: implementation route. This is the target path for replacing scattered
autocorrect rules with one correction and suggestion pipeline.

This document is a contract for future lay work. Runtime behavior may improve
step by step, but the architecture must move toward this route and not back to
independent text mutation paths.

## Core Law

```text
L1 sees form.
L2 creates candidates.
L3 scores phrase meaning.
L4 holds current task state.
Bayes remembers usage.
SafetyGate blocks destructive edits.
IME only displays.
EditAction only applies.
```

Forbidden shortcut:

```text
word -> local rule -> direct text replacement
```

Required route:

```text
noisy input
-> surface signals
-> candidate lattice
-> phrase/context frame
-> task-state prior
-> usage prior
-> safety proof
-> suggest/apply/keep
```

## Current Cutover Law

The runtime has two different output routes, but it must not have two different
brains:

```text
IME / preedit route:
  unfinished token -> L2/L3/L4/Bayes field -> DecisionCore live admission
  -> visible suffix -> explicit Tab accept

Space autocorrect route:
  completed token -> L2/L3/L4/Bayes field -> DecisionCore transition decision
  -> verifier -> AuthorizedEdit
```

The shared field is central evidence, not a backend.  L2/L3/L4/Bayes may
strengthen, suppress, or reject a candidate in both routes, but they never
write text directly.  IME producers provide material; `TransitionDecisionCore`
owns visible-candidate admission.

### IME single word-field cutover (2026-08-13)

Tested: the production-configured IBus candidate route after removing the
second ASCII word query.

Measured facts:

```text
semantic_phrase_candidates()                         separate suffix producer
precognition_candidates() -> word_candidate_proposals()                    1
word_candidate_proposals() -> TypingCpu::live_completion_candidates()      1
candidate readout -> display readout                                      1
production FieldSnapshotOnly ytn -> →нет                               PASS
RU shared-authority prefix                                                PASS
EN exi -> exit completion                                                 PASS
runtime source change                                                    -65 lines
```

The short layout gate now accepts an exact keyboard projection only when the
existing compact hot field grants phase authority. It does not load the full
runtime dictionary and adds no literal surface branch. Phrase forecasting,
arrow rendering, Tab authorization, verifier ownership, and physical mutation
ownership did not change.

Not tested: no binary was installed, no IBus process was restarted, and no
physical application input was exercised in this experiment. Runtime authority
therefore remains unchanged pending the normal release/deployment gate.

Verdict scope: source-backed route structure plus focused production-configured
binary parity. This is not a broad candidate-quality or live-input proof.

Receipt:

```text
docs/structural_gates/receipts/LAY_IME_SINGLE_CANDIDATE_ROUTE_OBSERVED_2026-08-13.json
```

### Declined IME target suppression (2026-08-13)

Tested: manual continuation after a visible completion in the unified IBus
candidate route.

The accepted interaction contract is:

```text
prefix P0 -> visible full target T0
printable grapheme instead of Tab -> new prefix P1
-> T0 is declined for the remainder of the current token
-> exact T0 proposals are removed before display readout
-> another full surface may still become visible
-> the first observation remains available for censored boundary feedback
```

The decline key is the normalized full surface. For a completion it is
`current partial + suffix`; for a replacement it is the replacement surface.
No literal word, suffix, source ID, or morphology-slot exception participates
in runtime behavior. The state is bounded by the 32-character active token and
is cleared at its boundary.

Measured facts:

```text
real refresh rejects the previous full target                     1/1 PASS
target-specific helper and retained feedback                      2/2 PASS
IME candidate lifecycle focused set                               3/3 PASS
Space, autocorrect and double-Shift contracts                    19/19 PASS
typing-transition authority contracts                            20/20 PASS
focus/reset contracts                                              9/9 PASS
changed-code gate                                                      PASS
hot candidate generation p99 / max                           61 / 80 us
literal target branches in runtime                                      0
```

What was not tested: no release binary was built or installed, no IBus process
was restarted, and no physical application input was exercised. Candidate
birth and L2/L3 ranking were not changed and this result is not a broad
candidate-quality proof.

Verdict scope: source and software proof for exact-surface suppression after a
manual continuation. Display readout behavior changes in the source; Tab,
verifier, text-mutation ownership, and installed runtime authority do not.

Receipts:

```text
docs/structural_gates/receipts/LAY_IME_DECLINED_TARGET_PREFLIGHT_2026-08-13.json
docs/structural_gates/receipts/LAY_IME_DECLINED_TARGET_PREFLIGHT_V2_2026-08-13.json
docs/structural_gates/receipts/LAY_IME_DECLINED_TARGET_SOFTWARE_PROOF_2026-08-13.json
```

## L3 Self Teacher Route

L3 learns context only through an offline teacher/proof loop:

```text
clean phrase source
-> generated dirty surfaces
-> compact surface mutation field
-> context phase package
-> shadow readout
-> promotion gate
-> runtime package only after PASS
```

The teacher may write cold artifacts and `.nwpc` shadow packages. It must not
install runtime authority by itself. Runtime promotion is clean-only by default:
it merges the self-teacher package into the tracked canonical L3 package and
uses local live feedback only when explicitly requested. A useful first metric
is `false_authority`: wrong candidates may have noisy raw energy during
learning, but they must not receive L3 Support.

The clean teacher corpus must repeat phrases enough to cover both profile birth
and required support. For `min_profile_support=N`, the repeat count is
`2 * N - 1`; the default `N=2` therefore uses three clean passes.

The current dirty-class route is wave/geometry based, not word-specific:

```text
missing letter
extra letter
letter substitution
sparse multi omission
adjacent transposition
full and partial layout projection
premature space
glued words
punctuation suffix
```

These classes provide surfaces for L2/L3 learning. They are not allowed to
apply text directly.

## Example Target

Raw input:

```text
так делай все что можно в инетеполучить
```

The system must not solve this as one hardcoded split rule.

It must build competing readings:

```text
так, делай все, что можно в инете получить
так, делай все, что можно получить в интернете
так делай все что можно в инетеполучить
так делай все что можно и не получить
```

Then L3/L4 should prefer the command/source frame:

```text
COMMAND
  action: do / use / retrieve
  scope: everything possible
  source: internet
```

Only after this frame has enough margin may the system suggest the cleaned
phrase. Full phrase replacement is suggest-only until a separate boundary proof
allows it.

## Pipeline Tree

```text
LAY INPUT PIPELINE
|
+-- L1 Surface Wave
|   |
|   +-- reads raw characters and key events
|   +-- emits UTF-8 validity, script classes, layout shape
|   +-- emits n-gram and position centers
|   +-- emits boundary pressure and anomaly signals
|   +-- never edits text
|
+-- L2 Candidate Lattice
|   |
|   +-- consumes L1 signals
|   +-- creates word and fragment candidates
|   +-- creates split/glue candidates
|   +-- creates layout and typo candidates
|   +-- creates prefix/completion candidates
|   +-- keeps all serious alternatives, not only one winner
|
+-- L3 Phrase Frame Field
|   |
|   +-- sees short phrase context
|   +-- scores whether a candidate is allowed in this scene
|   +-- strengthens coherent phrase readings
|   +-- suppresses route-splice and nonsense readings
|   +-- may propose phrase-level suggestions
|
+-- L4 Streaming Task State
|   |
|   +-- holds current interaction state
|   +-- chat / terminal / code / settings / command-to-agent
|   +-- aggressive or conservative mode
|   +-- current goal and edit risk context
|
+-- Bayes / Usage Prior
|   |
|   +-- learns accepted words and phrases locally
|   +-- learns rejected or reverted corrections
|   +-- updates unigram / bigram / trigram usage pressure
|   +-- never hardcodes one user's private text into source code
|
+-- Decision Core
|   |
|   +-- combines L2, L3, L4, Bayes, and risk
|   +-- emits one EditAction
|   +-- does not write text directly
|
+-- SafetyGate
|   |
|   +-- checks delete length and insert text
|   +-- checks touched word count
|   +-- blocks unsafe multiword edits
|   +-- blocks boundary changes without boundary proof
|
+-- Output Backends
    |
    +-- IME display backend
    +-- daemon/uinput backend
    +-- GNOME/IBus layout sync backend
```

## One Text Mutation Contract

No module may mutate visible text directly.

All text-changing paths must emit:

```text
EditAction
```

Minimum fields:

```text
action_kind
source
confidence
from_text
to_text
delete_len
insert_text
touched_words
boundary_changed
requires_user_accept
layout_after
evidence
```

Allowed action kinds:

```text
Keep
Suggest
ReplaceLastToken
ReplaceRange
SplitToken
GlueTokens
AcceptImeCandidate
SwitchLayout
BlockUnsafe
```

Hard rule:

```text
L2 may propose.
L3 may strengthen or suppress.
DecisionCore may choose.
SafetyGate may block.
Only output backends may apply the final EditAction.
```

## IME Contract

IME is a display and acceptance backend.

IME must not own:

```text
word correction logic
double Shift logic
tail repair logic
Bayes memory
phrase meaning
layout policy
```

IME may do only:

```text
show candidate
select candidate
accept candidate
cancel candidate
report active/passive state
```

If IME needs a correction, it asks the common pipeline and receives an
EditAction.

### Completion Is Not Autocorrect

This distinction is a permanent runtime contract:

```text
IME / preedit:
  incomplete current token -> suffix candidate -> explicit Tab acceptance

Autocorrect:
  completed token + boundary -> typed correction operator -> verifier
  -> AuthorizedEdit -> backend output
```

IME must never disguise a full-token typo, layout, split, or glue replacement
as a suffix completion. A full-token proposal may be displayed separately as
`typed->candidate`, but only when the shared field supplies a typed operator
(layout, boundary, measured damage geometry, or corrected-prefix morphology)
or independent context/transition evidence. Candidate popularity and lexical
or n-gram proximity alone have no display authority.

Tab may accept either a suffix that extends the current token or a visibly
distinct full-token proposal. A full-token replacement still belongs to the
shared correction pipeline and may be executed only after its typed transition
proof and verifier succeed.

Implementation ownership:

```text
live IME readout -> L2 completion + evidence-gated display-only replacement
Space correction -> full L2 replacement / layout / boundary lattice
```

The two routes may share L2/L3/L4/Bayes field evidence, but they must not share
physical edit authority.

## Next Front: Lay Self Teacher For L3

Current L3 baseline is observed but not decisive:

```text
evidence hit: 9/24 = 37.5%
authority: 4/24 = 16.7%
output_changed: 0
improved/worsened: 0/0
verdict: L3_CONTEXT_OBSERVED_NOT_DECISIVE
```

The next planned architecture step is `lay-self-teacher`: an offline trainer that
generates dirty/clean phrase pairs, runs them through the same L1/L2/L3/L4
pipeline, and compiles local L3 phase and anti-phase context memory.  The goal is
to make L3 change decisions under shadow proof, not merely observe context.

## Safety Contract

Default autocorrect can touch only the current token.

Unsafe by default:

```text
delete text containing spaces
change non-last word
change word count
replace multiple words
insert a word into the middle of another visible word
move cursor into older text as part of correction
```

These require a separate proof:

```text
SplitToken
GlueTokens
ReplaceRange over multiple words
full phrase rewrite
```

Without proof:

```text
Suggest only.
Do not auto-apply.
```

## Candidate Sources

All candidate sources must implement one interface and return the same shape.

```text
CandidateSource
|
+-- LayoutSource
+-- TypoSource
+-- TranspositionSource
+-- SplitGlueSource
+-- PrefixCompletionSource
+-- L2SurfaceWaveSource
+-- UsageBayesSource
+-- L3PhraseSource
```

Candidate shape:

```text
replacement
source
surface_score
context_score
usage_score
risk_score
confidence
evidence
gate_action_hint
```

No candidate source applies text.

## L1 Route

L1 is the surface sensor layer.

Inputs:

```text
key stream
visible committed tail
IME active state
layout state
UTF-8 text
```

Outputs:

```text
script class
layout shape
token boundary pressure
char n-gram centers
position centers
repeat / missing / transposition pressure
split / glue pressure
```

L1 must stay cheap and hot-path friendly.

## L2 Route

L2 is the candidate lattice layer.

It must output many candidates with evidence:

```text
raw token: инетеполучить

candidates:
  инете получить
  интернете получить
  инет получить
  и не получить
  keep
```

L2 must stop behaving like a single winner picker. Ranking may exist, but
selection belongs to DecisionCore after L3/Bayes/SafetyGate are included.

## L3 Route

L3 is phrase/context meaning pressure.

Initial bounded frames:

```text
COMMAND
QUESTION
STATEMENT
REQUEST
ENUMERATION
NEGATION
CONDITION
TECH_TEXT
CHAT_TEXT
```

L3 output:

```text
frame_id
frame_margin
role bindings
candidate boosts
candidate suppressions
phrase suggestion candidates
```

L3 must not become a hardcoded phrase router. Its job is to score whether the
candidate lattice settles into a coherent phrase frame.

## L4 Route

L4 is the streaming task-state layer.

It holds:

```text
current app / input profile
terminal vs chat vs code
current user action mode
layout authority state
IME active/passive state
recent accepts/rejects
aggressiveness level
```

L4 answers:

```text
Should lay be aggressive here?
Should this be suggest-only?
Is this a command-like phrase?
Is the current field safe for preedit?
Which backend should apply the final EditAction?
```

L4 must not hardcode corrections. It only gives state/context pressure.

## Bayes / Usage Route

Bayes is a signal, not a replacement for L3.

It learns locally:

```text
accepted candidate
accepted by Tab
auto-applied and not reverted
manual correction after suggestion
rejected candidate
context left/right
input profile
```

It stores compact local counts:

```text
unigram usage
bigram usage
trigram usage
correction pair stats
context transition stats
reject/revert stats
```

It provides:

```text
P(candidate | prefix, context, user history)
```

It must be controllable by the single debug/log/privacy switch.

## Decision Formula

Working scoring shape:

```text
final_score =
  L2_surface_score
+ L3_frame_score
+ L4_task_state_prior
+ Bayes_usage_prior
- edit_risk
- boundary_risk
- hallucination_risk
```

Decision:

```text
high score + low risk:
  AutoApply last-token edit

medium score:
  IME Suggest

low score:
  Keep

boundary or multiword change:
  Suggest only unless boundary proof passes
```

## Implementation Order

```text
1. Build route map for every text mutation path.
2. Add EditAction type and dry-run rendering.
3. Put SafetyGate before every apply path.
4. Route daemon autocorrect through EditAction.
5. Route double Shift through EditAction.
6. Route IME accept through EditAction.
7. Move IME correction logic out of IME into common pipeline.
8. Create L1SurfaceSignal object.
9. Create L2CandidateLattice object.
10. Adapt existing layout/typo/split/prefix sources into CandidateSource.
11. Add Bayes/usage prior as a candidate signal.
12. Add L3 phrase frame scorer in suggest-only mode.
13. Add L4 task-state prior.
14. Move DecisionCore to one final chooser.
15. Make preedit use the same DecisionCore.
16. Delete old direct replacement paths.
17. Clean tray settings after runtime architecture is stable.
```

## Current Progress Matrix

```text
checkpoint: 0.2.160

1. mutation route map:
   status: PASS-basic
   evidence: text-mutation-monopoly-plan + typed mutation_route logs

2. EditAction dry-run/safety:
   status: PASS-basic
   evidence: text_edit transition safety + candidate_before_apply deleted/inserted text

3. SafetyGate before apply:
   status: PASS-basic
   evidence: runtime text edits call EditAction safety before backend output

4. daemon autocorrect through EditAction:
   status: PASS-basic
   evidence: enter_autocorrect / typing_assist routes log typed mutation_route

5. double Shift through EditAction:
   status: PASS-basic
   evidence: manual text/native replacement routes and replay/native replay routes pass through EditAction before backend output

6. IME accept through EditAction:
   status: PASS-basic
   evidence: ime_active_composition / ime_committed_tail routes are explicit; Tab/IME completion accept now creates EditAction::AcceptImeCandidate with safety/log evidence before CommitText

7. move IME correction logic out:
   status: PASS-basic
   evidence: active-composition correction decision moved to lay::ime_correction; correction regression cases live in the shared ime_correction layer; IBus autocorrect_*_text helpers are deleted and blocked by text_mutation_monopoly_contract; dead pending committed-tail Space autocorrect branch removed

8. L1SurfaceSignal object:
   status: PASS-basic
   evidence: TypingErrorEvent now comes from correction_core::l1_surface_signal

9. L2CandidateLattice object:
   status: PASS-basic
   evidence: correction_core::L2CandidateLattice owns candidate collection and dedup only

10. CandidateSource adapters:
   status: PASS-basic
   evidence: correction_core::L2CandidateSource routes deterministic/NANDA proposals by mode

11-13. Bayes/L3/L4:
   status: PASS-basic
   evidence: DecisionCore rank now consumes Bayes posterior, L3 phrase pressure, L4 scene state, and L4 signed memory; input_gate/recent_actions candidate score traces expose l3_phrase_milli, l4_scene_milli, l4_signed_milli, and decision_rank_milli without widening the public scoreboard API; candidate_quality_report exposes decision_lanes coverage for Bayes/L3/L4/DecisionRank

14. DecisionCore final chooser:
   status: PASS-basic
   evidence: correction_core::decision_core owns selected apply ranking; L2CandidateLattice only collects/dedups candidates

15. preedit from DecisionCore:
   status: PASS-basic
   evidence: contract test keeps preedit.rs display-only; active composition correction enters shared ime_correction -> input_gate/correction_core -> DecisionCore route

16. delete old direct mutation paths:
   status: PASS-basic
   evidence: manual replay and native replay bypasses now pass through EditAction with typed manual backend routes and replay transition proof; contract test blocks direct replay bypass

17. tray cleanup:
   status: PASS-1.0.27
   evidence: docs/lay-menu-settings-architecture.md owns the accepted inventory; settings.js and prefs.js are thin entrypoints over one settings_view.js; tray_ui_contract blocks model-authority knobs, duplicate settings implementations, stale research controls, and diagnostic service restarts
```

## Commit Route

```text
commit 1: mutation route inventory and docs
commit 2: EditAction + SafetyGate skeleton
commit 3: daemon apply path through EditAction
commit 4: double Shift through EditAction
commit 5: IME accept/display through EditAction
commit 6: L1SurfaceSignal
commit 7: L2CandidateLattice
commit 8: CandidateSource adapters
commit 9: Bayes usage prior
commit 10: L3 frame scorer
commit 11: L4 task-state prior
commit 12: unified DecisionCore
commit 13: preedit from DecisionCore
commit 14: remove old direct mutation paths
commit 15: tray cleanup
commit 16: Bayes/L3/L4 first-class DecisionCore lanes
commit 17: version bump and release sync
commit 18: IME completion accept through AcceptImeCandidate EditAction
commit 19: move IME autocorrect proof cases into shared ime_correction
commit 20: candidate-quality lane scoreboard for Bayes L3 L4
```

## Scoreboard

The project must track:

```text
candidate_count
selected_source
selected_score
best_score
bad_rank
unsafe_edit_plan
boundary_changed
weak_bayes
bayes_unsupported
l3_context_used
l3_phrase_milli
l4_scene_milli
l4_signed_milli
decision_rank_milli
auto_apply_rate
suggest_rate
accept_rate
revert_rate
latency p50/p90/p99/max
```

Current diagnostic owner:

```text
lay-nanda-wave-eval --candidate-quality-report
```

## Debt Queue

```text
P0: keep direct text mutation outside EditAction blocked by contract tests
P0: block unsafe multiword autocorrect
P0: keep IME display-only
P1: make first-word IME suggestions aggressive
P1: make Bayes usage prior dominate repeat choices
P1: make L2 output a real lattice
P2: strengthen bounded L3 phrase frames with broader corpus proof
P2: tune L4 task-state pressure against dirty-log regressions
P2: remove old tail hacks
P3: tray cleanup is contract-guarded; keep future UI changes behind tray_ui_contract
```

## Done Definition

This route is implemented only when all are true:

```text
No visible text change bypasses EditAction.
IME has no correction brain.
Double Shift and autocorrect use the same pipeline.
L2 emits candidate lattice with evidence.
L3 can strengthen/suppress by phrase frame.
L4 can change aggressiveness by current scene.
Bayes raises locally accepted words and phrases.
Unsafe multiword edits are blocked or suggest-only.
First-word IME suggestions work.
After Space, stale IME suffix closes.
Candidate Quality Report explains every apply/suggest decision.
Old direct mutation paths are deleted.
Tray/runtime/version stay synchronized.
```

## Release 1.0.24: Single Word-Candidate Route

Status: `LIVE_SOFTWARE_GATE_PASS_PHYSICAL_TYPING_PENDING`.

Release `1.0.24` closes the source/runtime drift that existed after the IME
candidate-route implementation. The release contains:

```text
semantic phrase producer
-> one word_candidate_proposals()
-> one TypingCpu::live_completion_candidates()
-> one candidate readout
-> one display readout
-> existing Tab/verifier authority
```

It also makes a typed continuation decline only the currently visible full
target until the token boundary. The first prediction remains censored
feedback; absence of Tab does not become negative learning evidence.

Measured software proof before the release build:

```text
real refresh suppression                 1/1 PASS
target-specific state and feedback       2/2 PASS
Space/autocorrect/double-Shift          19/19 PASS
authority contracts                    20/20 PASS
focus/reset                              9/9 PASS
scripts/check-lay-changed.sh                 PASS
hot candidate generation p99              61 us
hot candidate generation max              80 us
```

Release build and installation facts:

```text
source commit                 773fae2b9f6f223f63283b610b779d506e94a95f
remote build host             e@192.168.3.94
remote source root            /home/e/builds/lay-release-1.0.24-20260813-033012-git
Cargo jobs                    20
release wall time             185.01 s
release CPU time              532.53 s
average release CPU           287%
release peak RSS              2,381,200 KiB
release swap                  0
remote Cargo target           9,470,091,264 B / 12,884,901,888 B
release binary parity         10/10 PASS
installed version             1.0.24
loaded extension              1.0.24
live daemon SHA parity        PASS
live IBus engine SHA parity   PASS
global ibus-daemon PID        3702 -> 3702
active engine                 lay-ime-ru
DBus Ping                     pong from lay-extension
```

Rollback snapshot:

```text
/home/ubu/.local/lib/lay/rollback/1.0.23-pre-1.0.24-20260813-033957
```

The transient `Set global engine failed: connection interrupted` journal entry
occurred while the managed engine process was replaced. Final runtime checks
show exactly one managed engine, `lay-ime-ru`, and a healthy DBus bridge. Global
`ibus-daemon` was not restarted.

Physical confirmation of the declined-target behavior remains outside this
software receipt. The user should verify that after `от{носиться}` or another
visible completion, typing the next letter instead of Tab removes that exact
target and permits a fresh candidate.

Exact release receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/LAY_RELEASE_1_0_24_INSTALLED_2026-08-13.json
```

## Release 1.0.26: Target-Bound Replacement Authority

Status: `LIVE_SOFTWARE_GATE_PASS_PHYSICAL_TYPING_PENDING`.

The visible full-token IME replacement route no longer treats an operator lane
or a strong unrelated L3 center as proof for the selected target. Candidate
generation remains broad, but display authority is now target-bound:

```text
L2 candidate material
-> LiveCandidateLane                     grouping only
-> L2ImeTargetEvidence                   producer provenance
-> ReplacementTargetEvidence
   +-- ExactLayoutProjection
   +-- VerifiedLexicalEdit
   +-- VerifiedBoundary
   `-- None
-> L3/L4 contextual evidence             rank or independent support
-> one TransitionDecisionCore admission
-> display-only typed->candidate arrow
-> existing Tab/verifier mutation route
```

A replacement is visible only when the selected surface has a grounded L2
center and either target-specific transition evidence or independently learned
L3/L4 transition support related to the current input. Raw n-gram proximity,
candidate-source identity, lane membership, and unrelated context energy cannot
publish a `typed->candidate` arrow.

Broad lexical search is explicitly not target evidence. A lexical repair is
bound before canonical material is merged, and the typed evidence survives a
later source/rank replacement. A direct one-edit replacement is publishable only
when it is the unique nearest verified target. If an exact layout projection has
authority, same-script lexical neighbors remain internal lattice material and
cannot publish competing arrows.

Boundary evidence is also bound to the exact selected split. Both split parts
must be independent centers; decoder fragments cannot inherit the authority of
another valid split of the same token.

The readout keeps the latency contour bounded:

```text
rich exact-prefix field
-> skip mutation search
-> skip Productive V90

thin or damaged field
-> expanded bounded lexical material
-> Productive V90 only when still needed
```

Measured source/software proof:

```text
target replacement matrix                         4/4 PASS
target-bound integration matrix                    6/6 PASS
sequential candidate-gate suite                 29/29 PASS
focused IBus replacement/readout                  2/2 PASS
typing-transition authority contracts           20/20 PASS
scripts/check-lay-changed.sh                          PASS
unsafe verified-transition escapes                   0
warmed IBus candidate display p50 / p99 / max   42 / 70 / 73 us
```

The accepted matrix preserves:

```text
ytn          -> нет             exact layout projection
             -/-> yt/yen/yon     no lexical arrow beside exact layout
hf,jfntn     -> работает        layout then verified lexical edit
рабоает      -> работает        verified RU lexical edit
относитться  -> относиться      verified RU repeated-letter edit

какое       -/-> какаем         settled clean state
новая       -/-> ножовая        settled clean state
точнее      -/-> течение        settled clean state
относится   -/-> доноситься     settled clean state
```

For an incomplete damaged form, multiple surfaces may remain inside the related
morphology lattice, but unrelated lexical or morphology neighbors no longer
become visible replacements.

Release build and installation facts:

```text
remote build host             e@192.168.3.94
remote source root            /home/e/builds/lay-release-1.0.26-20260813-target-bound
Cargo jobs                    20
release wall time             203.47 s
release CPU time              795.12 s
average release CPU           390%
release peak RSS              2,386,280 KiB
release swap                  0
remote Cargo target           597,484,345 B / 12,884,901,888 B
release binary parity         10/10 PASS
installed version             1.0.26
loaded extension              1.0.26
live daemon SHA parity        PASS
live IBus engine SHA parity   PASS
global ibus-daemon PID        3702 -> 3702
active engine                 lay-ime-us
managed processes             1 daemon + 1 IBus engine
```

Rollback snapshot:

```text
/home/ubu/.local/lib/lay/rollback/1.0.25-pre-1.0.26-20260813-075855
```

Tested: source route, target evidence, candidate admission, fixed replacement
matrix, sequential candidate gate, focused IBus rendering, changed-code gate,
warmed latency, isolated release build, byte-identical installation, managed
runtime activation, version parity, and process continuity. Not tested in this
experiment: full L1 heldout restoration, package quality or size, physical
application typing, or post-install double-Shift rollback. Runtime authority is
now on installed `1.0.26`; physical behavior remains an explicit user smoke
check rather than a software claim.

Exact software receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/LAY_IME_TARGET_BOUND_REPLACEMENT_EVIDENCE_SOFTWARE_PROOF_2026-08-13.json
/home/ubu/projects/lay/docs/structural_gates/receipts/LAY_RELEASE_1_0_26_INSTALLED_2026-08-13.json
```
