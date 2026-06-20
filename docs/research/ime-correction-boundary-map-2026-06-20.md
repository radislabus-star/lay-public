# IME Correction Boundary Map

Date: 2026-06-20

## Goal

Fix live IME suffix-only corrections without changing the IME output owner.

Bad shape:

```text
visible token: don
internal tail: jn
candidate: от
wrong output: dот
```

The correction engine should not replace a suffix while the visible field still
contains a larger token. The output path itself must remain unchanged.

## Routes

### IME Key Route

Owner:

- `LayIbusEngine::ProcessKeyEvent`
- `process_pressed_key`

Responsibility:

- route printable keys, spaces, Enter, Backspace, Shift triggers;
- keep terminal passthrough separate from managed IME commit.

Do not mix with:

- visible text proof;
- actual delete/commit output.

### Visible Text Route

Owner:

- `SetSurroundingText`
- `SurroundingTextSnapshot`

Responsibility:

- store visible text, cursor and anchor;
- expose the visible token before cursor;
- reject correction if selection is active.

This route proves whether internal tail memory still matches the real field.

### Committed-Tail Planner

Owner:

- `autocorrect_committed_tail_space`
- `autocorrect_committed_tail_enter`
- `committed_tail_boundary_replacement`
- `CommittedTailPlan`

Responsibility:

- read the internal last token;
- require visible-token agreement when a snapshot exists;
- build backspace count, replacement text and original token as one plan.

Rule:

```text
visible token != internal token -> no committed-tail correction
```

### Correction Safety Gate

Owner:

- `correction_core::decide_text_correction`

Responsibility:

- reject impossible final candidates before output;
- keep mixed-language text only when scripts are separated by punctuation or
  whitespace.

Rejected shapes:

```text
dот
gривет
fавтозамена
```

Allowed shapes:

```text
QR-коды
html вот
```

### IME Output Route

Owner:

- `replace_committed_tail`
- `CommitText`
- `DeleteSurroundingText`
- terminal erase fallback

Responsibility:

- perform the already-approved replacement.

Protected rule:

```text
Do not change IME output semantics while fixing candidate/range bugs.
```

## Structural Changes

- Added visible surrounding-text snapshot for range proof.
- Added `CommittedTailPlan` so replacement range and replacement text travel
  together.
- Added final mixed-script token guard in correction core.
- Split Shift trigger handling out of `managed.rs`.
- Moved ASCII trailing-punctuation recovery into its own module.

## Regression Cases

```text
don     -> keep
ghbdtn  -> привет
ghbdtn, -> привет,
ckjdf?  -> слова,
lfdfq   -> давай
djn     -> вот
```

