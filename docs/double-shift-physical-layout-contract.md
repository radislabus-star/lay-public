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
    ├── committed tail with an executable typed output profile
    │   ├── SurroundingText delete + commit -> IME projection
    │   └── terminal erase + commit -> IME projection
    └── otherwise -> daemon physical replay
```

Selecting `DaemonWordBuffer` before checking pending undo is forbidden: it can
turn an undo gesture into an unrelated layout projection. Cursor geometry alone
does not grant a new mutation primitive. Authority may use only an output
profile already admitted by `can_replace_committed_tail()` for the exact token
delete count; without that capability the operation remains daemon-owned.

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

1. A daemon-owned replay must first capture an IME receipt containing the typed
   tail source, focus identity, tail epoch, exact expected suffix, and suffix
   character count.
2. The receipt is checked before layout handoff and again immediately before
   mutation. A controlled RU/EN engine handoff may change only the engine object
   path; source, epoch, suffix, and character count must remain equal.
3. A missing, stale, mismatched, empty, or wrong-source receipt authorizes zero
   Backspace. There is no fallback to an unproven `WordBuffer` length.
4. No Backspace is emitted until the target layout stack is ready.
5. A layout-readiness failure leaves the original visible text untouched.
6. Backspace and replay are paced even while physical input is isolated. Input
   isolation prevents re-entry; it does not prove that the client consumes a
   zero-delay uinput burst.
7. Every emitted key tap is a closed key-down/key-up frame.
8. Replay failure after deletion is an explicit partial-output failure. It must
   release all virtual keys and must never trigger a second mutation route.
9. Global `ibus-daemon` is never restarted by this operation.

For a committed terminal tail, the admitted IME profile performs one mutation:

```text
CommitText(DEL x exact_token_chars + opposite_layout_projection)
```

It must not first delete through daemon uinput and then emit the replacement as
per-key IME commits. Clients may coalesce those independent commits even when
the engine logs every key as handled.

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

## Release gate

A Double Shift release cannot receive final PASS or be published while physical
application typing is `NOT_TESTED`. Unit, integration, D-Bus, and uinput tests
are necessary but do not replace the client-visible gate. A rollback-protected
local canary installation is allowed only to execute that physical gate; it is
not an accepted release until the gate passes.

The physical gate records the complete text before and after each operation and
must cover RU to EN, EN to RU, pending autocorrect undo, active completion,
window/focus change, and stale-receipt refusal. Only the final token may change;
the preceding text and whitespace must remain byte-for-byte equal. Any excess
deletion, missing character, duplicate output, or mutation after stale evidence
blocks installation and release publication.

Each install receipt binds the immutable source snapshot, source diff hash,
release binary hashes, installed binary hashes, and loaded `/proc` executable
hashes. Rebuilding from a different or unrecorded dirty tree is forbidden.

## 1.0.41 Atomic Terminal Owner

The 1.0.40 exact lease correctly prevented excess deletion, but Kitty still
lost visible replacement characters. The failed canary proved that the daemon
emitted eight replay taps and the target IBus engine handled eight presses and
eight releases, while the PTY displayed only four characters. The loss was
therefore after daemon lease validation and before stable client-visible text.

The systemic repair aligns manual-toggle ownership with the already admitted
committed-tail output profile:

```text
nonempty exact committed token
-> can_replace_committed_tail(exact token length)
   ├── executable -> one IME committed-tail replacement
   └── unavailable -> exact daemon delegation remains fail-closed
```

No application name, word, phrase, key sequence, new deletion primitive,
verifier relaxation, or uinput-frame change was added.

Measured candidate facts:

```text
design route gate                                  PASS
implementation preflight            READY_TO_IMPLEMENT
focused manual-toggle tests                       11/11
complete lay-ibus-engine tests                  243/243
remote test wall                                  11.64 s
remote test max RSS                           324,280 KiB
remote release build wall                         51.38 s
remote release build max RSS                1,242,976 KiB
candidate engine size                         6,750,712 B
candidate SHA-256             ba43d440f4510c45dea23a88b9ecad8c...
```

Rollback-protected Kitty observations:

```text
file ghjdthrf -> Double Shift              file проверка  PASS
the same token -> Double Shift twice       file ghjdthrf  PASS
djn -> вот -> immediate Double Shift          djn file    PASS
preceding text deletion                               0
daemon uinput replay on accepted terminal events      0
terminal committed-tail mutation count                1
mutation latency                                  47-114 us
global ibus-daemon PID                  2076194 unchanged
```

The candidate verdict proves the Kitty terminal owner and exact undo route. A
post-canary automated Chrome field was not run; the user had independently
reported Chrome working before this scoped owner change, and Chrome's observed
zero-width/no-terminal output profile remains on its prior route. Multi-day
residency and every focus permutation remain untested.

Canary rollback:
`/home/ubu/.local/lib/lay/rollback/1.0.40-pre-1.0.41-kitty-atomic-owner-20260824-032355`.

Evidence:

- `docs/structural_gates/preflights/LAY_KITTY_DOUBLE_SHIFT_ATOMIC_TERMINAL_OWNER_ROUTE_V1_2026-08-24.json`
- `docs/structural_gates/preflights/LAY_KITTY_DOUBLE_SHIFT_ATOMIC_TERMINAL_OWNER_IMPLEMENTATION_V1_2026-08-24.json`
- `docs/structural_gates/receipts/LAY_KITTY_DOUBLE_SHIFT_ATOMIC_TERMINAL_OWNER_2026-08-24/`

### Final installed 1.0.41 gate

The complete 1.0.41 release was rebuilt from baseline commit
`61081d0c71896ab881a4ae70d611ea07ab4703be` on the 20-core remote host. The
authority-bearing code diff is bound by SHA-256
`c2cf5969b8b7df20f959267ad7048aedc376e6dbed5be6bea6625a9417526899`.

```text
focused manual-toggle tests                              11/11 PASS
complete lay-ibus-engine tests                         243/243 PASS
cold English latency budget                                1/1 PASS
candidate generation latency budget                        1/1 PASS
full release build wall                                    3m 32s
release binary count                                            10
release binaries total                               47,346,800 B
lay-ibus-engine size                                  6,750,712 B
lay-ibus-engine SHA-256      db85ea7621e7991bb619ebc93d8166f4...
```

The final installed bytes then repeated the physical Kitty gate:

```text
file ghjdthrf -> Double Shift              file проверка  PASS
the same token -> Double Shift twice       file ghjdthrf  PASS
djn -> вот -> immediate Double Shift          djn file    PASS
terminal projection latencies                   133, 122, 50 us
exact autocorrect undo latency                            72 us
preceding text deletion                                      0
daemon replay on accepted Kitty events                        0
```

Installed and loaded `/proc` hashes match for both live owners. The global
`ibus-daemon` remained PID `2076194`; only `lay-daemon` and the managed
`lay-ibus-engine` were replaced, and the active engine was restored to
`lay-ime-ru`. The installed GNOME extension files are 1.0.41. The running GNOME
Shell still reports its session-cached 1.0.33 metadata; no Shell or desktop
session restart was performed for a cosmetic version refresh.

Final rollback:
`/home/ubu/.local/lib/lay/rollback/1.0.40-pre-1.0.41-final-20260824-034200`.

Final receipt:
`docs/structural_gates/receipts/LAY_KITTY_DOUBLE_SHIFT_ATOMIC_TERMINAL_OWNER_2026-08-24/final-installed-1.0.41-v1.json`.
