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

## Canonical installed behavior (1.0.54)

This is the complete deployed route. Later implementation work must preserve
all branches together rather than copying one branch into another:

```text
Left Shift press/release/press/release
-> lay-daemon trigger FSM                         sole legacy detector
-> ManualToggleV3
-> decision priority
   1. exact pending autocorrect undo              if one is still valid
   2. ImeActiveComposition                        IME edits live composition
   3. ImeCommittedTail                            IME edits visible committed tail
   4. DaemonWordBuffer                            daemon/uinput fallback only
-> literal opposite-layout key projection
-> exactly one text mutation backend
-> exactly one layout owner
```

After IME autocomplete is accepted, the completed token belongs to
`ImeCommittedTail`. The daemon still detects the physical gesture, but the IME
must execute `toggle_committed_tail_target` because only the IME observed the
accepted suffix. Only `DaemonWordBuffer` may return `DelegateDaemon`.

For example:

```text
autocomplete: пров + ерка -> проверка
Double Shift: проверка -> ghjdthrf
next input layout: US
```

No model, candidate search, spelling correction, morphology, ranking, or
learning runs during that projection.

## User-visible behavior

With the configured `double-lshift` trigger, the gesture is exactly:

```text
Left Shift press -> release -> Left Shift press -> release
```

The second press must begin within `shift_window_ms` of the first release. The
hold duration does not matter, so the user must not need to strike Shift hard
or unnaturally fast. An intervening ordinary key cancels the partial gesture.

One completed gesture performs one action. If the last token was typed in the
wrong layout, Lay replaces that exact visible token once and leaves the opposite
layout active for subsequent typing. The detector rearms on the second release,
so the next complete pair immediately performs the inverse projection. Four
Shift taps are two pairs and therefore two toggles; no ordinary key or quiet
window is required between them. A valid pending autocorrect undo has priority
over ordinary projection.

## Installed 1.0.54 runtime routes (historical)

This section records the installed baseline that preceded TD-009. Its
committed-tail delete-plus-commit branch is superseded by the effective
exact-observed-tail route below; it is not an alternative executable route.

```text
physical Double Shift
├── pending autocorrect undo
│   └── existing exact undo route
└── ordinary layout projection
    ├── focused IME owns active composition
    │   └── one authorized IME composition replacement
    ├── focused IME owns committed tail
    │   ├── one authorized committed-tail delete + commit
    │   ├── wait for exact full SurroundingText postcondition
    │   └── activate target GNOME source and matching IBus engine once
    └── DaemonWordBuffer fallback
        ├── prepare target layout
        ├── delete exactly N visible characters
        ├── replay exactly N captured physical keycodes
        └── update replay bookkeeping
```

The IME and daemon fallback are mutually exclusive text backends. The IME route
must not delegate `ImeCommittedTail`, and the daemon must not repeat the layout
switch after an IME-handled result. Neither projection route may call a model,
replace physical mapping with a ranked candidate, or write a learning sample.

## Physical gesture owner

This is a protected runtime invariant, not an implementation preference. The
forbidden regression signature is one physical gesture producing
`original -> replacement -> original`.

The daemon is the sole detector of physical Double Shift in the deployed
legacy input route:

```text
physical Shift events
-> lay-daemon trigger FSM
-> one daemon manual-toggle decision
-> ManualToggleV3 when the focused IME owns the visible text
-> one IBus replacement backend
```

Legacy IBus `ProcessKeyEvent` observes Shift modifier state for composition and
Alt+Shift, but it must not recognize the same Double Shift pair or invoke the
manual-toggle mutator. Otherwise one physical gesture reaches two independent
detectors and the second replacement restores the original text.

The source boundary is intentionally capability-shaped:

```text
observe_daemon_owned_legacy_shift
    inputs: keyval, keycode, state
    output: handled boolean
    EngineOutput/edit capability: absent

process_atomic_shift_gesture
    reachable only while atomic_speculation == true
    EngineOutput capability: present
```

The release gate must run the `physical_double_shift_owner_` tests. They send
both Left and Right Shift pairs and four-tap bursts through legacy
`ProcessKeyEvent`, require native-unhandled output with zero effects, and fail
if the legacy boundary acquires any edit API.

The exclusive atomic adapter is separate: Shell sends the event through
`ProcessKeyEventAtomicV1`, legacy mutation is disabled, and IBus may return one
atomic effect frame. This does not grant legacy `ProcessKeyEvent` a second
gesture owner.

## Client-visible layout postcondition

Committed-tail mutation and layout switching form one ordered transaction:

```text
delete old committed tail
-> commit exact projected tail
-> observe the exact replacement through SurroundingText
-> one GNOME bridge request activates the target input source
-> that same owner immediately schedules the matching Lay IBus engine
```

When SurroundingText was available at dispatch, switching the engine before
the exact visible replacement is observed is forbidden. GTK may acknowledge
delete and commit asynchronously; an early engine switch can leave the old
surface visible, duplicate text, or race focus handoff. A stale pre-dispatch
snapshot keeps the postcondition pending and must not change layout.

`replace_committed_tail` is the sole owner of this ordering. It arms
`pending_visible_postcondition` with the projected text, and
`observe_visible_postcondition` performs the layout transition only after an
exact or permitted boundary-elided match. `toggle_committed_tail_target` must
not perform a second immediate layout sync after dispatch.

The daemon also must not call `switch_to_target_layout` after an IME-handled
`ManualToggleV3` reply. That reply closes daemon gesture dispatch; it does not
transfer the IBus layout postcondition back to the daemon. The daemon may keep
the target layout in its bounded gesture bookkeeping, while the focused IBus
engine remains the sole owner of the actual engine transition.

If the client does not expose SurroundingText, no acknowledgement route exists;
the already defined terminal/no-snapshot fallback may switch immediately after
successful output dispatch. This fallback does not weaken the acknowledged
client route.

### Consequence analysis

Measured before this ownership repair, an installed GTK repetition matrix
passed only `3/5`: every iteration dispatched one exact delete-plus-commit, but
the daemon's immediate layout switch could force focus handoff before GTK made
the replacement visible. The candidate set, ranking, correction policy and
manual-toggle text plan were not involved.

The selected design removes the daemon's second layout action for an
IME-handled result. Existing IBus branches remain complete:

```text
active composition                 -> IBus blocking layout sync
committed tail + SurroundingText   -> IBus visible-postcondition sync
committed tail without snapshot    -> IBus immediate fallback sync
exact autocorrect undo             -> same committed-tail postcondition owner
daemon/uinput delegation           -> existing daemon layout owner
```

Rejected alternatives were a fixed sleep before daemon switching, which would
only move the race, and a second acknowledgement protocol, which would duplicate
the existing SurroundingText postcondition state. This change does not alter
candidate/lattice retention, authority ranking, package or cache identity,
learning, RSS, or allocation behavior. It removes one process-level layout
write and therefore narrows concurrency and failure surface. The rollback
boundary is the daemon ownership call plus the committed-tail immediate sync;
acceptance requires full IBus/daemon tests and repeated installed client-visible
proof with the global `ibus-daemon` PID preserved.

## Pairwise rearm

One exact Double Shift pair triggers one layout projection and returns the
gesture state to `Idle` on its second release. The next Shift press starts a new
pair immediately, including when it arrives inside the previous
`shift_window_ms` and no ordinary key was pressed.

The route has no post-action burst membership, latch, quiet deadline, debounce,
or multi-tap delay. In particular:

```text
press release press release  -> toggle 1
press release press release  -> toggle 2
four consecutive taps        -> two toggles
```

The single-owner rule still applies independently: each pair may mint exactly
one plan, never plans from both daemon and legacy IBus recognition.

The configured IME executor does not grab or drain the physical input device.
It performs no key replay, so a third and fourth Shift tap remain in the normal
daemon event stream and form the next pair. Only the explicit uinput fallback
may isolate physical input while replaying keys.

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
    ├── exact IME committed tail -> leased daemon isolated replay
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

The conversion is the literal reversible physical-key table. For example,
`а <-> f`, `п <-> g`, and `привет <-> ghbdtn`. It does not ask whether the
result is a word and it does not choose among candidates.

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

## Historical rejected SurroundingText route

This route is preserved only as failed evidence. Legacy IBus clients do not
receive delete-plus-commit as one atomic effect, and two managed GTK attempts
proved that `RequireSurroundingText` does not force the callback required by
the old transaction:

```text
exact full source snapshot
-> arm exact full replacement postcondition
DeleteSurroundingText
-> CommitText
-> exact full replacement snapshot
-> layout synchronization
```

Neither pre-mutation nor post-mutation refresh produced a new exact snapshot.
This sequence is therefore not an admitted GTK committed-tail executor.

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
3. The exact committed-tail lane uses the existing bounded paced replay while
   physical input is grabbed. Delete and insert are each limited to 32 events;
   any invalid or unrepresentable plan fails before layout or text mutation.
   A grab isolates physical input but does not acknowledge that GTK consumed a
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
- One physical gesture produces exactly one manual-toggle plan and one layout
  transition; the legacy IBus key route produces zero replacement effects.
- Every adjacent pair is independent: four Shift taps produce two plans and
  restore the original text and layout.
- A committed-tail projection validates one exact IME snapshot before and
  after one GNOME input-source activation. The second lease proves target
  layout, focused IME, source, epoch, and unchanged suffix before mutation.
- The exact route uses no generic D-Bus transport fallback: a lost
  `ActivateLayout` or `CurrentLayout` response fails closed instead of invoking
  the method again.
- Checked autocorrect suppression is armed only after both lease checks and
  immediately before the first Backspace.
- A layout toggle produces one IBus focus handoff, not a visible two-stage
  source-then-engine handoff.
- Pending autocorrect undo remains unchanged.
- Model and correction-learning calls on ordinary Double Shift are zero.
- Full-engine tests pass when executed together; isolated targeted PASS cannot
  hide shared-state or ordering failures.

Daemon emission logs prove attempted mutation only. They are not a visible-text
PASS without an independent client-visible observation.

## 2026-08-29 exact observed-tail correction

The effective GTK route was selected only after the two historical
SurroundingText variants failed with immutable receipts. It is:

```text
daemon pair detector
-> ManualToggleV3 exact-tail disposition
-> one IME VisibleTailV2 snapshot and literal projection plan
-> physical input grab
-> first exact lease check
-> existing expiry-bounded cross-engine tail handoff
-> one no-fallback GNOME ActivateLayout
-> exact shell + IBus readback
-> second exact lease check on the focused target engine
-> checked suppression
-> one bounded paced Backspace + replay
-> independent managed-GTK visible result
```

`us` and `lay-ime-us` (likewise `ru` and `lay-ime-ru`) are the only accepted
GNOME readback identities for the corresponding frozen language. The exact
IBus engine identity remains mandatory. No daemon-buffer reconstruction may
replace the IME-observed suffix, including an accepted autocomplete suffix.
The handoff has the existing 700-ms safety expiry but introduces no delay,
polling, retry, or scheduled mutation.

The first live inverse matrix rejected the zero-pace candidate: the second
pair reached the daemon after only the first character of the first replay was
visible to IBus. The effective executor therefore reuses the already proven
paced key emitters; it adds no pre-trigger wait or gesture debounce. The same
matrix also exposed that an older boundary-correction refactor had left the
manual-toggle suppression consumer test-only. Production Space handling now
consumes that existing one-shot capability before consulting its prepared
autocorrect lease.

The failed implementation attempts and selected-route receipts are recorded in
`tech_debt/009-fix-manual-toggle-visible-postcondition-race.md`. Installed
runtime authority remained unchanged throughout candidate review.
