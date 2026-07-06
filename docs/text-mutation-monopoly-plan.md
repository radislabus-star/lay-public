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

## Debt Queue

```text
P0: carry edit_transition proof to the top-level EditAction log.
P1: forbid new raw text mutation callers in architecture checks.
P2: remove duplicate local "EditPlan" names that hide the real text-edit plan.
P3: make IME a display/commit backend, not an independent correction owner.
P4: add candidate-quality counters for unsafe top-level transitions.
```

## Laws

1. A candidate can be clever, but an output backend must be dumb.
2. A normal autocorrect edit cannot silently change earlier words.
3. Multiword edits require a boundary-class proof.
4. IME may display candidates; it does not own candidate truth.
5. A log entry must show what would be deleted and what proof allowed it.
