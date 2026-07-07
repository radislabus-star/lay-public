# Text Mutation Monopoly Plan

Goal: every operation that can change visible text must pass through one
auditable mutation contract before any output backend presses keys, commits IME
text, or replaces a tail.

This is not a new correction brain. The brain remains:

```text
L1 surface signal
-> L2 word candidates
-> L3/L4 context and state transition scoring
-> InputGate
-> TextMutation contract
-> output backend
```

`TextMutation` is the last proof layer before side effects.

## Tree

```text
TEXT CHANGE ROUTES
|
+-- word-boundary autocorrect
|   |
|   +-- InputGate decides candidate
|   +-- edit_transition proves state transition
|   +-- TextMutation authorizes apply
|   +-- backend deletes/inserts text
|
+-- manual toggle / double Shift
|   |
|   +-- WordBuffer / decoder owns word reading
|   +-- TextMutation checks edit shape
|   +-- backend replays or replaces
|
+-- IME display
|   |
|   +-- may show candidates
|   +-- may commit active composition
|   +-- must not become a second correction brain
|
+-- completion accept
|   |
|   +-- accepts visible candidate
|   +-- publishes handoff state
|   +-- next double Shift uses the shared route
|
+-- auto-undo
    |
    +-- reverses a known previous action
    +-- must still be visible as a TextMutation route
```

## Scoreboard

```text
SCOREBOARD
|
+-- transition proof
|   +-- operator
|   +-- proof
|   +-- verified
|   +-- left_context_changed
|   +-- changed_tokens
|
+-- edit-plan safety
|   +-- deleted_text
|   +-- inserted_text
|   +-- boundary_changed
|   +-- would_touch_words
|   +-- safety_reason
|
+-- runtime output
    +-- decision_ms
    +-- output_ms
    +-- backend
```

Current diagnostic commands:

```text
lay-debug-actions --unsafe-scoreboard
lay-debug-actions --unsafe-edits
lay-nanda-wave-eval --candidate-quality-report
```

Current checkpoint:

```text
0.2.152
|
+-- runtime text edits pass through text_edit transition safety
+-- direct runtime authorize_replacement calls are blocked by a contract test
+-- DoubleShift and TabAccept are visible in InputGate trace
+-- Enter/manual/native before-apply logs now carry gate trace when the route owns one
+-- active-composition autocorrect decision lives in shared lay::ime_correction
+-- dead pending committed-tail Space autocorrect branch removed
+-- TypingErrorEvent now enters correction through L1SurfaceSignal instead of local event parsing
+-- correction_core now routes deterministic/NANDA candidate generation through L2CandidateSource
+-- candidate ranking includes verified edit-transition operator weight, so boundary/layout proof can beat L2 surface shortcuts
+-- every candidate_before_apply record now carries typed mutation_route:
    +-- enter_autocorrect
    +-- typing_assist_minimal
    +-- typing_assist_ime
    +-- manual_text_replace
    +-- manual_native_replace
    +-- auto_undo
    +-- ime_active_composition
    +-- ime_committed_tail
+-- unsafe edit scoreboard detects:
    +-- boundary_changed
    +-- multiword_touch
    +-- transition_left_context_changed
    +-- unverified_transition
    +-- selected candidate transition risk
    +-- slow_output
```

Live diagnostic sample after this checkpoint:

```text
recent_actions window: 294
unsafe records: 14
boundary_changed: 0
multiword_touch: 0
slow_output: 4
```

## Debt Queue

```text
P0: move stale legacy recent_actions records out of the active scoreboard window after release.
P1: keep IME committed-tail transition explicit as a transition route, not fake InputGate.
P1: remove duplicate local "EditPlan" names that hide the real text-edit plan.
P2: make IME a display/commit backend, not an independent correction owner.
P2: keep candidate-quality counters in release checks.
```

## Laws

1. A candidate can be clever, but an output backend must be dumb.
2. A normal autocorrect edit cannot silently change earlier words.
3. Multiword edits require a boundary-class proof.
4. IME may display candidates; it does not own candidate truth.
5. A log entry must show what would be deleted and what proof allowed it.
