# L1-L4 Intelligence Route

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
checkpoint: 0.2.159

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
   status: PASS-partial
   evidence: DecisionCore rank now consumes Bayes posterior, L3 phrase pressure, L4 scene state, and L4 signed memory; input_gate/recent_actions candidate score traces expose l3_phrase_milli, l4_scene_milli, l4_signed_milli, and decision_rank_milli without widening the public scoreboard API

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
   status: PASS-basic
   evidence: tray_ui_contract keeps live suggestions under input mode, keeps debug log as action-journal only, and blocks revival of the old separate gray-suggestions switch
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
