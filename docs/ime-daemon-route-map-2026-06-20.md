# IME / Daemon Route Map

Date: 2026-06-20
Runtime baseline: `0.1.233`

## Main Finding

IME is currently not just another output backend. It owns a separate word state
and a separate double Shift path.

This is the architectural source of repeated bugs:

- double Shift behaves differently in normal input and IME;
- automatic correction can fight a manual double Shift result;
- candidate generation differs by route;
- layout sync exists in more than one place;
- daemon and IME can observe the same physical typing through different APIs.

Target rule:

```text
IME must be presentation/output, not a separate correction brain.
```

## Route A: Normal Input

Entry:

```text
physical keyboard
-> evdev
-> lay-daemon
```

Manual double Shift route:

```text
lay-daemon
-> manual_trigger_runtime/event.rs
-> manual_trigger_runtime/fire.rs
-> trigger_dispatch.rs
-> correction_runtime.rs::handle_double_shift
-> WordBuffer
-> decoder / replay / smart text plan
-> daemon output adapter
```

Important files:

- `src/bin/lay_daemon/manual_trigger_runtime/event.rs`
- `src/bin/lay_daemon/manual_trigger_runtime/fire.rs`
- `src/bin/lay_daemon/trigger_dispatch.rs`
- `src/bin/lay_daemon/correction_runtime.rs`
- `src/word_buffer.rs`
- `src/decoder/`
- `src/bin/lay_daemon/text_output/`

State owner:

```text
WordBuffer
```

Output owners:

```text
uinput replay
GNOME ReplaceText
IME backend bridge when selected
```

Daemon has important replay-specific behavior:

- replay memory;
- scope handling;
- smart minimal replacement;
- suppress next typing-assist after manual replay;
- pending typing-assist cleanup;
- layout switch / restore after output.

## Route B: IME Input

Entry:

```text
IBus
-> lay-ibus-engine
-> org.freedesktop.IBus.Engine.ProcessKeyEvent
```

Manual double Shift route:

```text
lay-ibus-engine
-> ibus_interface.rs::process_key_event
-> managed.rs::handle_shift_release
```

If active preedit buffer exists:

```text
handle_shift_release
-> double_shift_replacement(buffer)
-> commit_active_composition
-> CommitText
```

If active preedit buffer is empty:

```text
handle_shift_release
-> toggle_committed_tail
-> committed_tail_toggle_replacement
-> replace_committed_tail
-> DeleteSurroundingText / CommitText
```

Important files:

- `src/bin/lay_ibus_engine/ibus_interface.rs`
- `src/bin/lay_ibus_engine/managed.rs`
- `src/bin/lay_ibus_engine/committed_tail.rs`
- `src/bin/lay_ibus_engine/composition_commit.rs`
- `src/bin/lay_ibus_engine/state.rs`
- `src/bin/lay_ibus_engine/tail_memory.rs`
- `src/bin/lay_ibus_engine/layout_sync.rs`

State owners:

```text
LayIbusEngine.buffer
LayIbusEngine.tail_buffer
LayIbusEngine.preedit_fast
LayIbusEngine.preedit_suffix
LayIbusEngine.word_input_mode
LayIbusEngine.layout_is_ru
```

Output owner:

```text
replace_committed_tail
commit_active_composition
CommitText
DeleteSurroundingText
UpdatePreeditText
```

## Current Overlap

Daemon still observes physical keys through evdev while IME handles text through
IBus.

Daemon tries to step back when IME owns active text:

```text
focused_ime_engine_handles_typing()
```

Known skip points:

- manual trigger fire;
- typing-assist after Space;
- deferred typing-assist;
- Enter autocorrect.

Files:

- `src/bin/lay_daemon/layout_controller.rs`
- `src/bin/lay_daemon/layout_controller/ibus_bridge.rs`
- `src/bin/lay_daemon/manual_trigger_runtime/fire.rs`
- `src/bin/lay_daemon/boundary_runtime/space.rs`
- `src/bin/lay_daemon/boundary_runtime/deferred.rs`
- `src/bin/lay_daemon/enter_autocorrect_runtime.rs`

Risk:

```text
The skip is runtime/session-dependent. If it misses, daemon and IME can both
react to the same typing.
```

## Decision Owners Today

### Daemon Decision Owners

```text
handle_double_shift
WordBuffer
decoder
typing_pipeline
text_edit
```

### IME Decision Owners

```text
handle_shift_release
committed_tail_toggle_replacement
autocorrect_committed_tail_text
autocorrect_active_composition_text
selected_precognition_suffix
```

This is the core problem: IME has local correction decisions.

## Output Owners Today

Daemon:

```text
text_output/*
GNOME DBus ReplaceText
uinput replay
IME bridge for text backend
```

IME:

```text
commit_active_composition
replace_committed_tail
clear_preedit
update_composition_preedit
update_precognition_preedit
```

Output owners should stay separate. They are different platform adapters.

Decision owners should become shared.

## Desired Architecture

```text
Input adapter
  - daemon evdev
  - IME IBus

-> Unified text state snapshot
   - current token
   - completed tail
   - active composition
   - scope
   - layout state
   - route kind

-> Unified correction core
   - manual double Shift
   - typing assist after Space
   - Enter autocorrect
   - NANDA candidates
   - suppress/undo policy

-> CorrectionPlan
   - original range
   - replacement text
   - target layout
   - suppress next autocorrect
   - trace kind

-> Output adapter
   - daemon uinput/GNOME
   - IME CommitText/DeleteSurroundingText/preedit
```

## Proposed Shared Types

These should be introduced before moving behavior:

```text
TextTailState
ManualToggleRequest
ManualTogglePlan
CorrectionPlan
OutputRoute
AutocorrectSuppression
```

Do not move `WordBuffer` into IME directly. Instead:

```text
WordBuffer -> TextTailState
LayIbusEngine buffer/tail_buffer -> TextTailState
```

Then both routes call the same core.

## Refactor Order

### Phase 1: Contracts Only

Add shared structs with tests.

No runtime behavior change.

Exit criteria:

- daemon compiles unchanged;
- IME compiles unchanged;
- architecture guard passes.

### Phase 2: Manual Toggle Core

Extract only the decision:

```text
TextTailState -> ManualTogglePlan
```

No output changes.

Exit criteria:

- daemon manual double Shift uses shared decision;
- IME manual double Shift uses shared decision;
- `работает -> hf,jnftn -> работает` is stable in both routes.

### Phase 3: Manual Toggle Suppression

Move suppress-after-manual-toggle into shared plan.

Exit criteria:

- after manual double Shift, immediate Space autocorrect does not undo it;
- daemon and IME both respect the same suppression rule.

### Phase 4: IME Space Autocorrect

Make IME committed-tail Space use shared correction core.

Exit criteria:

- IME no longer has independent autocorrect decision logic;
- IME only applies a `CorrectionPlan`.

### Phase 5: Candidate Expansion

Only after route unification:

- improve NANDA candidates;
- improve L2/L3 candidates;
- tune precognition.

Exit criteria:

- more candidates without changing output adapters;
- no route split reintroduced.

## What Not To Do

- Do not bind correctness to `SetSurroundingText`.
- Do not make IME parse visible field as the source of truth.
- Do not change `CommitText` / `DeleteSurroundingText` semantics during decision
  refactor.
- Do not add sleeps to hide route races.
- Do not hardcode live phrases.

## First Regression Set

Manual toggle cycle:

```text
работает -> hf,jnftn -> работает -> hf,jnftn
привет -> ghbdtn -> привет -> ghbdtn
вот -> djn -> вот -> djn
```

Manual toggle followed by Space:

```text
работает [double Shift] [Space]
expected: hf,jnftn 
not: работает 
```

Mixed route guard:

```text
IME active: daemon manual trigger must not also fire.
IME inactive: daemon manual trigger must still fire.
```

Candidate sanity:

```text
hf,jnftn -> работает
ghbdtn -> привет
djn -> вот
```

