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

## 1.0.34 Double-Shift Delegation Regression

Date measured: 2026-08-22.

The 1.0.34 cutover replaced the former IME-first branch with this nested
result:

```text
None          IME route was not selected
Some(Some)    IME handled the toggle
Some(None)    IME was selected but returned NotHandled
```

`fire.rs` returned for every `Some` value. Consequently a non-atomic IME with
no owned composition or committed tail could explicitly defer to the daemon
WordBuffer, but `Some(None)` consumed the physical gesture before the existing
daemon planner ran. Live evidence after the 1.0.34 install showed:

```text
configured trigger                         double-lshift
focused IME state                          passive:daemon-word-buffer
manual-toggle actions after install        0
global ibus-daemon PID                      unchanged
```

Restoring the old unconditional fallback is forbidden. An atomic owner also
returns no legacy result, and a D-Bus error has uncertain execution status;
either case falling through could create a second mutation owner.

The corrected protocol therefore has three explicit outcomes:

```text
Handled(target layout)  complete through the IME owner
DelegateDaemon          run the existing WordBuffer planner once
NotHandled              complete fail-closed with no second route
```

Only a non-atomic `DaemonWordBuffer` authority may produce
`DelegateDaemon`. Atomic focus always produces `NotHandled`. A malformed V3
wire value or D-Bus failure is an error and remains fail-closed. Legacy
`ManualToggleV2` stays available for compatibility but cannot represent or
authorize delegation.

Measured after implementation:

- observed-source route gate: `PASS`, all `23/23` source markers verified;
- shared V3 wire tests: `2/2 PASS`;
- daemon dispatch and D-Bus failure mapping: `1/1 PASS`;
- IBus manual-toggle authority, atomic exclusion, suppression and handoff:
  `11/11 PASS`;
- remote Cargo cache after focused tests: `888,401,920 B`, below the
  `12 GiB` guard budget.

These results prove the compiled typed delegation and fail-closed mappings in
the dedicated remote snapshot. They do not yet prove the release build,
installed 1.0.35 process continuity, or physical double-Shift behavior.

Evidence:

- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_TYPED_DELEGATION_ROUTE_V1_2026-08-22.json`
- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_TYPED_DELEGATION_IMPLEMENTATION_V1_2026-08-22.json`
- `docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_TYPED_DELEGATION_2026-08-22/implementation-preflight-v3.json`
- `docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_TYPED_DELEGATION_2026-08-22/observed-source-route-v1.json`

## 1.0.35 Release And Runtime Installation

Date measured: 2026-08-22.

The guarded remote release build completed in `3m 54s`. The ten staged
binaries were copied atomically into `/home/ubu/.local/lib/lay/bin`; byte-for-
byte comparison against the staging directory passed for all ten files. The
two staging names `lay-l11-restore` and `lay-l11-serve` map to the installed
public names `lay-l1.1-restore` and `lay-l1.1-serve`.

```text
binary                  bytes       sha256
lay                     6,387,624   7f7ccaf138593e44b7b8dd932a5cdda6bb4004b5e36b3c624fe5cc48a5280593
lay-daemon              8,510,288   30d7912bdd492ebff1e031a9839f0e3825545c99f638bd243ddc81bd21653884
lay-ibus-engine         6,741,752   ba2da60a7fe686b029f479507d808b2511bffe7aa05160809ff5876acbac1c87
lay-l1.1-restore        1,954,008   7299d5be68efe71a6b1a44c61aa2f6ab13321dbc25aaa182cad4ddf6945f569a
lay-l1.1-serve          2,165,824   ca83f0dc71cbbddd44a462563993916c94cda2e77c0725aa5fad60803d20e9b4
lay-memory-report         643,128   85115d2ed1150a1f22185b4bfa8d85cd89bba7dbc17ced70eb166575d89f08fa
lay-nanda-wave-eval     6,831,440   d95408f520994395c478b0fb5baeae8b0783f873147d5b4d4135a989ff2961bc
lay-nanda-wave-train   11,500,672   4243bfbf6cf594d03a08464624962582fd286be1f183e8aff3ce681d256c6dc2
lay-ngram-corpus          787,360   40eb70b32f03ea2c46e79e0ae9dad5ffd279f73c9fe321b4f937032b50b0c70b
lay-test-input          1,839,632   eccf6fcb6f124a7ead1ffd50be690e831e82103fe231ee3bce09fc91023ea9d3
```

Rollback snapshot:

```text
/home/ubu/.local/lib/lay/rollback/1.0.34-20260822-133654
```

After synchronizing and reloading the GNOME extension, only the Lay processes
were restarted. The global `ibus-daemon` was not restarted.

```text
CLI version                 1.0.35
loaded extension version    1.0.35
lay-daemon PID              937186, exactly one
lay-ibus-engine PID         937223, exactly one
GNOME layout                lay-ime-ru
IBus engine                 lay-ime-ru
global ibus-daemon PID      2076194, unchanged
startup errors              none observed
```

Runtime authority is now Lay 1.0.35. This is a technical installation verdict,
not a physical input-quality verdict. The following user-visible checks remain
`NOT TESTED` after installation:

```text
ghbdtn + double left Shift -> привет
cj,frf + Space/autocorrect + immediate double Shift -> cj,frf
```

## 1.0.35 Physical False-Handled Finding

Date measured: 2026-08-22.

The physical gate failed after the 1.0.35 installation. Debug evidence proved
that the gesture and V3 wire were not the failing layers:

```text
physical double Shift FSM          fired
daemon WordBuffer cross-check      4/4 exact
ManualToggleV3                     returned Handled
selected IME output route          terminal_erase_commit
visible client mutation            absent
```

The first loss was the committed-tail output capability selector. It treated
`cursor_cell_width > 0` as proof of a terminal client and emitted
`DEL x N + replacement` through `CommitText`. Cursor geometry is not a typed
delete capability: ordinary input clients also publish a positive cursor
width. The IME then advanced its private tail and returned `Handled` even when
the visible client did not delete or replace text.

The 1.0.36 authority contract is:

```text
active composition                 -> IME owner
committed tail + SurroundingText   -> IME owner
committed tail without proven delete backend
                                   -> explicit DelegateDaemon
atomic route                       -> NotHandled, fail-closed
D-Bus error or unknown V3 status   -> fail-closed
```

The correction is scoped to physical manual toggle authority. It does not edit
the shared committed-tail output implementation, autocorrect selection,
candidate acceptance, verifier, or SafetyGate. Delegation occurs before the
pending committed-tail auto-undo lane, so the unproven terminal route cannot
mint another false `Handled` for the same gesture.

Measured after implementation:

- design route gate: `PASS` after rejecting two earlier malformed route
  drafts;
- implementation preflight: `READY_TO_IMPLEMENT` after closing all blockers;
- observed-source route gate: `PASS`, `25/25` evidence markers;
- remote IBus manual-toggle tests: `11/11 PASS`;
- remote daemon typed-delegation test: `1/1 PASS`;
- `state.rs`, autocorrect/candidate output routes and daemon dispatch bytes:
  unchanged.

Evidence:

- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_PROVEN_OUTPUT_AUTHORITY_ROUTE_V1_2026-08-22.json`
- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_PROVEN_OUTPUT_AUTHORITY_OBSERVED_V1_2026-08-22.json`
- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_PROVEN_OUTPUT_AUTHORITY_IMPLEMENTATION_V1_2026-08-22.json`
- `docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_PROVEN_OUTPUT_AUTHORITY_2026-08-22/`

These gates prove the scoped source route. The 1.0.36 release build,
rollback-protected installation and repeated physical client test remain
separate gates.

## 1.0.36 Delegated Output Re-entry Finding

Date measured: 2026-08-22.

The physical `1.0.36` check was still reported as failed. Source tracing found
that `DelegateDaemon` changed the correction owner but did not bind the output
owner. The WordBuffer planner entered the common manual output pipeline, whose
first stage was still `try_ime_replace_output`. The delegated event could
therefore re-enter IME `ReplaceTail`, select the same unproven committed-tail
backend and finish without a visible client mutation.

The corrected `1.0.37` route contract is:

```text
ManualToggleV3 Handled
-> IME mutation owner

ManualToggleV3 DelegateDaemon
-> WordBuffer planner
-> ManualCorrectionOutputRoute::DaemonUinput
-> skip all IME/GNOME native output stages
-> existing authorized uinput replacement/replay

pending autocorrect undo
-> ConfiguredBackend
-> existing auto-undo runtime, unchanged

ManualToggleV3 NotHandled/error
-> terminal fail-closed
```

The output route is typed request data, not a new runtime owner. No global
state, lexical fixture, candidate rule, verifier change or SafetyGate change
was added.

Measured facts before release:

- design route gate: `PASS` after two `VETO` revisions corrected an invalid
  owner ordering;
- implementation preflight: `READY_TO_IMPLEMENT`;
- observed-source route gate: `PASS`, `31/31` source markers;
- remote V3 dispatch and pending-undo parity tests: `2/2 PASS`;
- remote output-route exclusion test: `1/1 PASS`;
- remote complete `lay-daemon` suite: `201 PASS`, `6 FAIL`;
- the same six typing-assist tests also fail against the exact pre-change
  daemon sources, so they are measured baseline debt and not a regression from
  the delegated-output change.

Not tested at this point:

- physical `ghbdtn + double left Shift -> привет` with installed `1.0.37`;
- physical `cj,frf + Space/autocorrect + double Shift -> cj,frf` with installed
  `1.0.37`;
- runtime authority is still installed `1.0.36` until the release transaction
  completes.

Evidence:

- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_ROUTE_V2_2026-08-22.json`
- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_OBSERVED_V2_2026-08-22.json`
- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_IMPLEMENTATION_V2_2026-08-22.json`
- `docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_2026-08-22/`

## 1.0.37 Installed Live Verdict

Installed on 2026-08-22 and observed through 2026-08-23.

The complete release was built remotely with `20` Cargo jobs in `3m 29s`.
All ten installed binaries matched the remote staging SHA-256 values. The
installation preserved the global IBus process and reloaded only Lay-owned
runtime components.

```text
source / installed version         1.0.37 / 1.0.37
installed lay-daemon SHA-256       52cedadda952c1485fbc3763f69c2c612a274f4cc4afca749b57333b051868b1
installed lay-ibus-engine SHA-256  096d554931ede3d30bc14b6325cb86305a365bd03912dc17e6a4823360add209
lay-daemon PID                     1037229
lay-ibus-engine PID                1037269
global ibus-daemon PID             2076194, unchanged
rollback                           /home/ubu/.local/lib/lay/rollback/1.0.36-20260822-151058
```

The live journal later observed five ordinary manual conversions with the
required route markers in the same event:

```text
physical manual trigger delegated to daemon WordBuffer
-> explicit IME delegation selected daemon uinput output
-> authorized uinput replay completed
```

Measured total event latency was `15, 15, 15, 19, 34 ms`. A separate mixed
digit sample took `123 ms`; it is not included in the ordinary-word latency
claim and remains a performance observation rather than a PASS criterion.
Every observed event reached exactly one daemon mutation route. No event
returned to IME `ReplaceTail`, and no duplicate mutation marker appeared.

Verdict scope:

- installed version/hash/process continuity: `PASS`;
- typed runtime route and single mutation owner: `PASS`;
- live ordinary-event completion telemetry: `FAILED_VISIBLE_POSTCONDITION`;
- visual correctness in every supported client: not implied by journal-only
  evidence and remains part of the multi-client product gate;
- autocorrect-undo physical round trip: still requires a separate observed
  event.

Installed/live receipt:

`docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_2026-08-22/installed-live-v1.json`

## 1.0.38 Deterministic Double Shift Repair

The 1.0.37 daemon journal proved only that the requested Backspace and replay
frames were emitted. Physical RU to EN testing then showed missing visible
letters. Therefore 1.0.37 is not a physical quality PASS.

The first shared mechanism was:

```text
physical Double Shift
-> shared smart manual-correction policy
-> asynchronous GNOME/IBus switch
-> zero-paced Backspace burst
-> zero-paced replay burst
-> emission logged as done without a visible postcondition
```

The 1.0.38 source candidate changes that route to:

```text
pending autocorrect undo -> unchanged exact undo route

ordinary Double Shift
-> exact captured physical keycodes
-> forced Replay policy, auto-replace disabled
-> target GNOME + IBus readiness before mutation
-> paced Backspace frames
-> paced replay frames
-> replay bookkeeping only, no correction-learning sample
```

Measured source gates on the remote 20-core build host:

```text
lay-daemon check                         PASS
deterministic runtime policy             1/1 PASS
manual-toggle focused tests             10/10 PASS
key-frame focused tests                   2/2 PASS
layout-controller focused tests           6/6 PASS
complete lay-daemon suite             202/208 PASS
new failures                                 0
baseline typing-assist failures              6
```

The six complete-suite failures are the same named baseline failures measured
before this repair. The rollback-protected 1.0.38 release transaction then
installed all ten remotely built binaries. The loaded daemon matches the
installed release byte-for-byte; client-visible RU to EN / EN to RU
postconditions remain untested, so the installed verdict is
`INSTALLED_AWAITING_VISIBLE_TEST`, not PASS.

```text
remote release build                 3m 51s, 20 jobs
remote Cargo target                  1,512,579,072 B / 12 GiB budget
release staging                      46 MiB, 10/10 SHA parity
installed lay-daemon SHA-256         79ebece266db8a4fc16993dc72c447e4f655586189db936fb0d3df1fd9d7d238
installed lay-ibus-engine SHA-256    b54d58f4ecbced7e0c698cfc9711912b03bb2b66ba54e04e9f6889245dd21737
loaded lay-daemon PID                2721049, exact installed-byte parity
global ibus-daemon PID               2076194, unchanged
loaded lay-ibus-engine PID           1037269, deliberately not restarted
rollback                             /home/ubu/.local/lib/lay/rollback/1.0.37-pre-1.0.38-double-shift-20260823-2242
```

Contract and preflight:

- `docs/double-shift-physical-layout-contract.md`
- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_DETERMINISTIC_VISIBLE_REPLAY_V1_2026-08-23.json`
- `docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_2026-08-22/deterministic-visible-replay-preflight-v2.json`
- `docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_2026-08-22/installed-live-v2.json`

## 1.0.40 Exact Observed Tail Lease

Release `1.0.39` could delegate ordinary Double Shift to daemon uinput while
the IME displayed a different, newer tail. The daemon then derived Backspace
count from its stale `WordBuffer`, so the projected token could be correct while
preceding visible characters were deleted.

`1.0.40` keeps the single daemon mutation owner but requires an exact IME lease:

```text
Double Shift
-> pending autocorrect undo first
-> physical input grab
-> IME typed tail receipt: source + focus + epoch + exact suffix + char count
-> target layout handoff
-> second exact receipt validation
-> one daemon uinput mutation
```

Any missing, stale, wrong-source, wrong-epoch, wrong-suffix, or wrong-length
receipt authorizes zero Backspace. There is no fallback to an unproven daemon
buffer length. A controlled US/RU engine handoff may change only the engine
object path.

Measured remote gate:

```text
focused daemon lease tests                         4/4 PASS
focused IME typed-tail test                        1/1 PASS
focused autocorrect/undo tests                     8/8 PASS
complete lay-ibus-engine                       237/237 PASS
complete lay-daemon                    206 PASS / 6 baseline FAIL
new daemon failure                                     0
check --lib --bins                                  PASS
release build                                   230.91 s
release max RSS                              2,626,380 KiB
Cargo target                          2,154,524,672 B / 12 GiB
```

The broad historical `--all-targets` gate remains non-PASS. Its failure set is
baseline-equivalent except for one RSS-budget test that is independently flaky
on the unchanged `1.0.39` snapshot (`1/3 PASS`) and on the candidate (`0/3
PASS`). This result is recorded as `BASELINE_FLAKY_NOT_PASS`; it is not used as
a quality claim for unrelated L1-L4 behavior.

Physical GTK/uinput proof after rollback-protected installation:

```text
file ghjdthrf + Double Shift             -> file проверка     PASS
file ghjdthrf + Double Shift twice       -> file ghjdthrf     PASS
djn -> вот + immediate Double Shift      -> djn file          PASS
preceding text deletion                                          0
global ibus-daemon PID                         2076194 unchanged
```

The separate `доллора` smoke did not autocorrect and therefore exercised an
ordinary layout projection (`доллора -> ljkkjhf`), not undo. It is classified
`NOT_APPLICABLE_NO_AUTOCORRECT_EVENT`, not PASS or FAIL for undo.

Installed binaries and loaded `/proc` executables have exact SHA parity. The
rollback point is
`/home/ubu/.local/lib/lay/rollback/1.0.39-pre-1.0.40-double-shift-20260824-021832`.

Receipt:
`docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_EXACT_OBSERVED_TAIL_2026-08-24/installed-live-1.0.40.json`.
