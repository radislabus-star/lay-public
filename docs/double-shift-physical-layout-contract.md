# Double Shift Physical Layout Contract

## Purpose

Outside a pending autocorrect undo, physical Double Shift is a deterministic
layout projection. It is not spelling correction, lexical decoding, candidate
ranking, morphology, or learning.

```text
captured physical keycodes K[0..N]
-> opposite layout L
-> visible text produced by replaying K[0..N] under L
```

The operation preserves key count and key identity. Only the layout used to
interpret those keys changes.

## Runtime routes

```text
physical Double Shift
├── pending autocorrect undo
│   └── existing exact undo route
└── layout projection
    ├── read captured physical keycodes
    ├── compute opposite layout
    ├── prepare the target GNOME and IBus layout before mutation
    ├── delete exactly N visible characters
    ├── replay exactly N captured keycodes
    └── update replay bookkeeping
```

The layout-projection route must not call a model, replace replay output with a
ranked text candidate, or write a correction-learning sample.

## Burst membership

One exact Double Shift pair triggers one layout projection. After that action,
continued Shift-only releases within `shift_window_ms` belong to the same
physical burst and cannot trigger a second projection. Every suppressed Shift
release extends the quiet deadline.

Any ordinary key press ends the burst and rearms the gesture immediately. A
full quiet window without another Shift release also rearms it, so a deliberate
later Double Shift remains available without requiring an intervening key.

The focused IBus observer keeps this burst latch in state shared by the US and
RU engine objects. A successful layout switch therefore cannot reset burst
membership while the user's taps continue. This shared state owns gesture
timing only; it grants no correction, candidate, deletion, or replay authority.

## Decision priority

The gesture has one ordered decision tree. Output ownership is selected only
after the protected undo state has been examined.

```text
Double Shift
├── valid pending autocorrect undo
│   ├── exact visible snapshot available -> exact undo on the proven owner
│   └── snapshot not available -> preserve undo and request/await observation
└── no valid pending autocorrect undo
    ├── active IME composition -> IME projection
    ├── committed tail with SurroundingText capability -> IME projection
    └── otherwise -> daemon physical replay
```

Selecting `DaemonWordBuffer` before checking pending undo is forbidden: it can
turn an undo gesture into an unrelated layout projection. Cursor geometry alone
does not grant the IME permission to delete committed client text.

## Exact projection

Ordinary projection is driven only by the currently selected layout:

```text
current RU -> Direction::Ru2Us -> target US
current US -> Direction::Us2Ru -> target RU
```

Script detection, mixed-script repair, candidate birth, ranking, morphology,
context, and preferred-layout inference are not part of this route. Characters
without a key-map counterpart are preserved by the projection table.

## Atomic route

Atomic handling has the same decision priority and text result as legacy IME
handling. A proven undo or committed-tail projection is represented by one
atomic delete-plus-commit frame. Speculative state becomes live only after the
matching submitted receipt. A missing snapshot must not consume, discard, or
replace pending undo, and atomic output must not depend on a legacy D-Bus signal
emitter.

Cross-engine Shift handoff may move gesture state between the US and RU engine
instances, but it must preserve the same shared pending undo and exact source
surface.

## Multi-client preparation isolation

Prepared Space/autocorrect leases are single-consumer capabilities. They are
owned by the complete `InputFrameIdentity`, including engine path and focus
lineage. Scheduling work for one engine path must not supersede, consume, or
retire another path's lease.

The runtime keeps a bounded set of per-path lanes. Each lane retains the
existing latest-frame-wins rule inside that path; the bound prevents abandoned
IBus paths from creating unbounded workers or memory. Eviction may produce a
fail-closed `NotReady` for the evicted path, never a lease from another path.

This isolation is required by the multi-client gate. Serializing tests would
hide the production race and is not acceptance evidence.

## Mutation contract

1. No Backspace is emitted until the target layout stack is ready.
2. A layout-readiness failure leaves the original visible text untouched.
3. Backspace and replay are paced even while physical input is isolated. Input
   isolation prevents re-entry; it does not prove that the client consumes a
   zero-delay uinput burst.
4. Every emitted key tap is a closed key-down/key-up frame.
5. Replay failure after deletion is an explicit partial-output failure. It must
   release all virtual keys and must never trigger a second mutation route.
6. Global `ibus-daemon` is never restarted by this operation.

## Acceptance evidence

- RU to EN and EN to RU are measured separately.
- For each successful case, `N captured = N deleted = N replayed = N visible`.
- Multi-character tokens preserve first, middle, and last characters.
- There is no duplicate output and no stuck modifier or character key.
- Pending autocorrect undo remains unchanged.
- Model and correction-learning calls on ordinary Double Shift are zero.
- Full-engine tests pass when executed together; isolated targeted PASS cannot
  hide shared-state or ordering failures.

Daemon emission logs prove attempted mutation only. They are not a visible-text
PASS without an independent client-visible observation.
