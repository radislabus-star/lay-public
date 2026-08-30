# TD-101 Consequence Analysis V1

Base commit: `23bcd577c58e49f339a27baf1220c9490f07b628`

## Decision

Admit a move-only decomposition of the internal `LayIbusEngine` state. Keep
`LayIbusEngine` as the sole external and mutable owner. The change may group
fields and add pure state helpers, but it may not alter IBus methods, event
ordering, D-Bus effects, worker scheduling, correction decisions, or layout
ownership.

## Selected Boundaries

| State object | Owned facts | Excluded authority |
|---|---|---|
| `CompositionState` | active buffer, cursor, preedit candidates/display flags, fast preedit cache, input mode | candidate generation and D-Bus rendering algorithms |
| `CommittedTailState` | committed tail/epoch, timing, pending postconditions/learning, suppression | correction planning and client mutation |
| `ClientContextState` | focus identity, client capabilities, surrounding snapshot, factory profile, managed-input flag | focus D-Bus entrypoints |
| `LayoutGestureState` | layout generation and Shift/Alt physical gesture bookkeeping | daemon pair detection and manual-toggle planning |
| `AtomicRouteState` | atomic route/speculation and deferred effects | atomic transaction orchestration |

`path`, `shared`, and `config` remain direct `LayIbusEngine` dependencies. This
keeps transport identity, cross-engine handoff, and runtime policy visible at
the external owner.

## Consequences

- **Behavior:** no intended behavior change. Existing methods remain on
  `LayIbusEngine`; only their field paths change.
- **Double Shift:** the daemon remains the sole legacy pair detector. IBus
  remains observation-only on the legacy route. The exact left-Shift gesture,
  committed-tail mutation, visible postcondition, and one layout switch remain
  unchanged.
- **Concurrency:** no thread, task, queue, timer, lock, callback, or D-Bus path
  is added. Nested structs are inline values and add no allocation.
- **Performance:** field grouping must not add cloning or heap ownership. The
  accepted claim is structural only; no speed improvement is claimed.
- **Compatibility:** no public Rust API, serialized schema, IBus XML, config,
  CLI, package, or installation path changes.
- **Rollback:** revert the scoped state-group commit. No migration or external
  state cleanup is required.
- **Proof:** compile and focused route tests run after each milestone on the
  disposable mini-PC checkout. The final gate is the full changed check plus
  lint; no installation or live restart is admitted.

## Failure Transitions

| Transition | Required result |
|---|---|
| focus replacement/reset | composition, stale gesture, stale worker and unproven tail facts clear exactly as before |
| atomic clone/rollback | nested state deep-clones and discarded speculation cannot leak deferred effects |
| legacy Shift observation | no local pair decision or text mutation |
| committed-tail toggle | one authorized replacement, exact visible postcondition, one layout handoff |
| preedit refresh | pending frame, dirty state and candidate tracking converge without stale suffix |

## Spectral Budget

Weighted benefit: `+17`.

- `+5` one explicit external mutation owner remains.
- `+4` five state lifetimes become locally inspectable and testable.
- `+3` focus/reset invariants stop depending on a 47-field flat record.
- `+3` canonical Double Shift state becomes physically separated from display
  and committed-tail state without changing its route.
- `+2` navigation and future regression review become bounded by state owner.

Weighted cost: `-5`.

- `-3` broad mechanical field-path diff across the IBus engine modules.
- `-2` one new internal module and nested initialization boilerplate.

Net score: `12`. There is no hard veto: no public contract, authority route,
async boundary, persistence format, candidate language, or installed runtime is
changed. The implementation is rejected if it becomes wrapper-only, introduces
duplicate state, or needs behavior changes to compile.
