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

## 1.0.39 Ordered Double Shift Authority

Release `1.0.39` removes the remaining route ambiguity:

```text
Double Shift
├── pending autocorrect undo
│   └── exact source restoration before any authority selection
└── ordinary toggle
    ├── current RU -> exact Ru2Us projection
    └── current US -> exact Us2Ru projection
```

Ordinary IME and daemon routes no longer invoke script detection, mixed-script
repair, candidate ranking, morphology, context, or correction learning. An
atomic Space receipt is retained as independent proof for the immediate undo,
so the atomic route can create one delete-plus-commit frame without a legacy
D-Bus signal emitter.

The full engine suite exposed a separate multi-client race in the one global
Space prefetch slot. It is now a bounded pool of at most eight path-isolated
lanes. A newer frame can supersede only an older frame from the same IBus path;
cross-path lease consumption is impossible.

Measured gate:

```text
parallel atomic proof               5 PASS / 0 FAIL
full lay-ibus-engine                236 PASS / 0 FAIL
full lay-daemon                     202 PASS / 6 baseline FAIL
global ibus-daemon PID              2076194 -> 2076194
installed version                   1.0.39
loaded extension Version()          1.0.39
physical application typing         NOT_TESTED
```

The daemon result is not a full PASS: the same six pre-existing typing-assist
expectation failures remain. They did not increase. Runtime authority changed
to `1.0.39`, but the physical Double Shift verdict remains open until the three
cases in `installed-live-v3.json` are typed in a real application.

Receipt:
`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_2026-08-22/installed-live-v3.json`.

Contract and preflight:

- `docs/double-shift-physical-layout-contract.md`
- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_DETERMINISTIC_VISIBLE_REPLAY_V1_2026-08-23.json`
- `docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_2026-08-22/deterministic-visible-replay-preflight-v2.json`
- `docs/structural_gates/receipts/LAY_DOUBLE_SHIFT_DELEGATED_UINPUT_2026-08-22/installed-live-v2.json`

## 1.0.44 IME Suggestion Shape And Window Latency

Measured on 2026-08-28 against the installed `lay-ibus-engine` SHA-256
`2f9fb292adde023b7f0b9d57cb0b55b752f91e18d204fd14eccb981eba7cc574`.

After that live receipt, the same change received one fail-closed repair: the
`50 ms` age check is repeated under the engine lock immediately before a
background result can begin publication. The final deployed binary SHA-256 is
`c8b0d77e81d5449f2ceeb2506781136229185ff0d9f50e3af9e1d41f8d4b266f`.
The measured values below remain bound to the receipt SHA above; the final
binary passed the complete changed-file gate and release build, but was not
substituted into that already completed physical latency measurement.

The IME presentation adapter no longer exposes whole-token replacement
proposals as live preedit. The shared candidate producer is unchanged; only
the IBus display route filters proposals whose edit geometry replaces the
observed token. Ordinary suffix completion remains available. Background
results older than `50 ms` from scheduling are retained as timing evidence but
cannot update a visible preedit frame.

The physical replacement probe `hf,jftn -> рабоает` and suffix probe
`ghjd -> пров + ерить` were sent through a dedicated virtual keyboard. The
tested client classes were Chrome text/search/textarea/contenteditable,
Chrome password, Chrome address bar, GTK 4 Entry and Kitty terminal.

```text
Chrome page replacement final     рабоает in all four fields
whole-token replacement preedit   0 in all four fields
ordinary replacement-path suffix  0.3-1.1 ms browser event latency

Chrome suffix visual              проверить in all four fields
max suffix event latency           3.6 / 0.7 / 0.6 / 0.8 ms
                                   text/search/textarea/contenteditable

Chrome address bar                ContentType purpose 5
whole-token replacement preedit   0
printable engine path             126-212 us

GTK Entry replacement final       рабоает
GTK suffix                        ерить, display age 84 us
GTK printable engine path         <=287 us

Kitty terminal                    ContentType purpose 10
text assistance / precognition    false / 0
```

Sensitive content is a separate fail-closed route. Password/PIN and
PRIVATE/HIDDEN_TEXT hints disable precognition and Space autocorrect. Entering
a sensitive field clears the in-memory tail and visible completion state.
Key/preedit traces redact decoded text, text length and cursor position.

The live Chrome password observation was `purpose=8`, `hints=6144`, with zero
candidates, zero decoded trace values, zero nonzero tail/preedit trace values,
zero precognition records and zero token payloads.

Not tested in this pass: real Telegram/WeChat conversations, a native Qt test
fixture, a live PIN field, or clients setting only PRIVATE/HIDDEN_TEXT hints.
The latter three sensitive variants are covered by engine tests, not claimed
as live client observations. Runtime authority changed only for
`lay-ibus-engine`; `lay-daemon`, the shared candidate producer and candidate
ranking authority were not changed.

Verdict: `IME_WINDOW_LATENCY_AND_REPLACEMENT_SHAPE_PASS`.

Evidence:

- `tests/manual/ime_latency/fields.html`
- `tests/manual/ime_latency/replacement.tsv`
- `tests/manual/ime_latency/suffix.tsv`
- `tests/manual/ime_latency/results-2026-08-28.json`

## 1.0.45 Kitty Terminal IME Regression Repair

The `1.0.44` presentation pass introduced two independent terminal
regressions. `content_allows_text_assistance()` treated IBus terminal purpose
`10` like a sensitive field, so Kitty received no completion frame. The manual
toggle authority also required SurroundingText for every committed tail, even
though the engine already had a tested terminal erase-and-commit backend.

Release `1.0.45` keeps password, PIN, PRIVATE and HIDDEN_TEXT fail-closed, but
allows ordinary text assistance for an explicit terminal purpose. A committed
tail is IME-owned when either SurroundingText exists or the client declares
terminal purpose and `CommittedTailOutputProfile` can execute. Cursor geometry
alone still grants no authority to a generic GUI client.

```text
full lay-ibus-engine                         245 pass / 0 fail
changed-file gate                            PASS
release build                                PASS, 8m 11s
installed/source binary parity               PASS, 10/10

Kitty ContentType                            purpose=10, hints=0
Kitty text assistance                        true
warm completion                              пров + ерить -> проверить
warm completion material/display age         12 us / 81 us
Double Shift                                 ghbdtn -> привет
Double Shift output route                     terminal_erase_commit
daemon-uinput fallback                        not used

installed lay-ibus-engine SHA-256             342c79f422e38769424ce9ba111c3fc607ed312725d3fd5d0fb7a955b71b48e6
installed lay-daemon SHA-256                  1160738dc8d310cb1c67883e3e7ffffceb5eade9f10b093832de4a3c8b22f446
global ibus-daemon PID                        4594 -> 4594
active engine                                 lay-ime-ru
```

The first cold `пров` materialization after restart took `231625 us` and was
correctly excluded by the existing `50 ms` stale-display gate. The repeated
warm route published `ерить` before acceptance. This repair does not weaken
that latency gate and does not change candidate production or ranking.

The isolated Kitty fixture was closed after verification. Chrome and GTK were
not physically rerun in this repair transaction; their routes remain covered
by the unchanged engine suite and the preceding `1.0.44` live matrix. Runtime
authority changed to `1.0.45`; correction policy did not change.

Verdict: `LAY_1_0_45_KITTY_IME_REGRESSION_REPAIRED`.

Evidence:

- `docs/structural_gates/receipts/LAY_1_0_45_KITTY_IME_REGRESSION_REPAIR_2026-08-28/RELEASE_RECEIPT.json`

## 1.0.46 Double Shift Key Sequence Repair

Release `1.0.46` removes hold duration from the Double Shift gesture. The
configured `double-lshift` trigger is now recognized as one exact ordered key
sequence:

```text
Left Shift press
-> Left Shift release
-> Left Shift press within shift_window_ms of the first release
-> Left Shift release
-> manual toggle
```

The duration of either press is irrelevant. Any intervening non-trigger key,
including a modifier use such as `Shift+letter`, cancels the partial sequence.
Right Shift and mixed left/right sequences cannot satisfy `double-lshift`.
The action remains release-triggered, so the second press alone never mutates
text. `tap_max_ms` is unchanged for configured single-key Shift/Ctrl/Alt
hotkeys and no longer participates in Double Shift detection.

The daemon event FSM and focused IME observer implement the same membership
contract. Candidate production, candidate ranking and the existing GTK/Kitty
output authorities are unchanged.

```text
implementation preflight                    READY_TO_IMPLEMENT
targeted daemon Double Shift tests           4 pass / 0 fail
targeted IME Double Shift tests              5 pass / 0 fail
atomic IME route tests                       9 pass / 0 fail
full lay-ibus-engine                         247 pass / 0 fail
full lay-daemon                              211 pass / 3 unrelated baseline fail
changed-file gate                            PASS
release build                                PASS, 6m 32s
installed/source binary parity               10/10 PASS

GTK ordinary Double Shift                    ghbdtn -> привет
GTK 2 ms holds                               ghbdtn -> привет
GTK 650 ms holds                             ghbdtn -> привет
Kitty 650 ms holds                           ghbdtn -> привет

installed lay-daemon SHA-256                 1cb2d89a8efa3d9bcc80c74045713eeb28889231e96baa6fe6919815cf9e681d
installed lay-ibus-engine SHA-256             e7a0237a578f503d33388857c4af70bc67a384a5de67242c3ebebe439d23d0b6
loaded lay-daemon PID                        1839304
loaded lay-ibus-engine PID                   1843501
global ibus-daemon PID                       4594 -> 4594
active engine                                lay-ime-ru
loaded extension                             1.0.46
live config SHA-256                          d73d5974a6b205e90db2e4562d438cea71f1c967315341fa53c4093dc73d0af4
```

The three daemon failures are the existing broad correction-core expectations
for `расчет ыприблизительные`; none enters the trigger FSM or manual-toggle
route. The live tests use a dedicated evdev/uinput keyboard and isolated GTK
and Kitty fields. A human-keyboard timing pass was not claimed.

Runtime authority changed to `1.0.46`; the canonical L2 package, exact V13
sidecar, live config and correction policy remained byte-identical.

Verdict: `LAY_1_0_46_DOUBLE_SHIFT_KEY_SEQUENCE_REPAIRED`.

Evidence:

- `docs/structural_gates/preflights/LAY_DOUBLE_SHIFT_KEY_SEQUENCE_REPAIR_V1_2026-08-28.json`
- `docs/structural_gates/receipts/LAY_1_0_46_DOUBLE_SHIFT_KEY_SEQUENCE_REPAIR_2026-08-28/RELEASE_RECEIPT.json`

## 1.0.47 Double Shift Burst Repair

Release `1.0.47` fixes the apparent need to hit Shift unusually hard or fast.
The detector was not dropping the user's releases: a continuous train of four
or more Left Shift taps contained several valid pairs, so the first pair
changed layout and the next pair immediately changed it back. The focused IBus
observer had a second version of the defect because its local pair state was
recreated when the successful toggle switched between the US and RU engine
objects.

The daemon now enters a burst latch after one completed Double Shift. Shift-only
releases inside `shift_window_ms` extend that latch and cannot trigger another
toggle. Any ordinary key rearms immediately; a full quiet window also rearms,
so a later deliberate Double Shift still works. IBus applies the same rule with
a small shared timing field that survives the US/RU engine handoff. The field
does not own correction planning or text mutation.

```text
targeted daemon burst regression              PASS
shared IBus cross-engine burst regression     PASS
full lay-ibus-engine                          248 pass / 0 fail
changed-file gate                             PASS
release build                                 PASS
installed/source binary parity                10/10 PASS

GTK four-fast-tap US -> RU                    ghbdtn -> привет
GTK four-fast-tap RU -> US                    слово -> ckjdj
GTK two pairs separated by 900 ms             п -> g -> п
Kitty four-fast-tap US -> RU                  ghbdtn -> привет
Kitty two pairs separated by 900 ms           п -> g -> п

installed lay-daemon SHA-256                  f928d1b1a405c50fac70e7f567bf5b644904f1bc36524a64b51a6c06d7132526
installed lay-ibus-engine SHA-256             76a0c6af279363d87cd96cea3d17904711e028952998fba51d1316355fa1eee6
loaded lay-daemon PID                         2291234
loaded lay-ibus-engine PID                    2275367
global ibus-daemon PID                        4594 -> 4594
loaded extension D-Bus version                1.0.47
```

The Kitty routes used the existing `terminal_erase_commit` authority; daemon
uinput fallback was not selected. The final client-visible matrix was run after
the unrelated Nando compilation had ended. It is correctness evidence, not a
new latency benchmark. The broad daemon suite was not rerun after the final
burst implementation; its focused regressions and the complete changed-file
gate passed.

Runtime authority changed to `1.0.47`. The live config, canonical L2 package,
exact V13 sidecar, candidate producer, ranking and correction policy remained
unchanged. Global IBus was not restarted.

Verdict: `LAY_1_0_47_DOUBLE_SHIFT_BURST_REPAIRED`.

Evidence:

- `docs/double-shift-physical-layout-contract.md`
- `docs/structural_gates/receipts/LAY_1_0_47_DOUBLE_SHIFT_BURST_REPAIR_2026-08-28/RELEASE_RECEIPT.json`

## 1.0.48 Double Shift Single-Owner Repair

The `1.0.47` verdict above is superseded. Its controlled GTK and Kitty tests
did not reproduce the user's physical keyboard route, where both the daemon
and legacy IBus `ProcessKeyEvent` observed the same key pair. The live trace
then recorded two complete plans for one gesture:

```text
проверка -> ghjdthrf -> проверка
```

The first plan came from local IBus Double Shift recognition. The daemon
independently recognized the same physical pair and called `ManualToggleV3`,
which applied the second plan. The burst latch suppressed later pairs but could
not make two owners of the first pair safe.

Release `1.0.48` restores one deployed owner:

```text
physical Double Shift
-> daemon trigger FSM
-> daemon manual-toggle plan
-> ManualToggleV3
-> focused IBus replacement backend
```

Legacy IBus key handling now records Shift modifier state and returns native
unhandled without producing a replacement. Alt+Shift is unchanged. The
exclusive atomic route retains its existing single atomic frame because legacy
mutation is disabled for that route and `ManualToggleV3` deliberately refuses
it.

Removing the second gesture detector exposed a separate layout-owner race. The
committed-tail backend dispatched the exact delete-plus-commit and armed its
existing client-visible postcondition, but `lay-daemon` immediately repeated
the layout switch from the successful `ManualToggleV3` reply. That focus handoff
could precede GTK's `SurroundingText` acknowledgement; the installed repetition
matrix then passed only `3/5` despite one correct text plan per iteration.

The final ownership is therefore split by responsibility, without duplication:

```text
gesture detection             one lay-daemon FSM
text mutation                 one focused IBus backend
SurroundingText acknowledgement one IBus visible postcondition
IME layout transition         one IBus postcondition owner
daemon/uinput fallback layout one daemon owner
```

For committed client text the ordered transaction is now:

```text
delete ghjdthrf
-> commit проверка
-> observe exact проверка through SurroundingText
-> switch lay-ime-us to lay-ime-ru once
```

Neither `toggle_committed_tail_target` nor the daemon's IME-handled reply path
may perform an immediate second layout sync. The no-SurroundingText fallback and
daemon/uinput delegation retain their existing immediate owner because they do
not have a client acknowledgement route.

Final verification:

```text
IBus physical_double_shift_owner_              3 pass / 0 fail
daemon physical_double_shift_owner_            1 pass / 0 fail
daemon Double Shift trigger tests              5 pass / 0 fail
full lay-ibus-engine                         251 pass / 0 fail
changed-file gate                             PASS
release build                                 PASS

broad lay-daemon                            212 pass / 3 fail
existing unrelated fixture                  расчет ыприблизительные

installed GTK repetition matrix                5 pass / 0 fail
manual-toggle plans                             5
committed-tail replacements                     5
confirmed-positive visible postconditions       5
layout transitions                              5
inverse plans                                    0

installed lay-daemon SHA-256                  95276fa3fe2e11d016ae5386127784ccf4e165bd52c652f4001ebf70b36a41d3
installed lay-ibus-engine SHA-256             06601d99abc4b8b9ea083bbf4d7790e9eff871891d00f6e9e3c0550071721bcb
loaded lay-daemon PID                         3687447
loaded lay-ibus-engine PID                    3674893
global ibus-daemon PID                        4594 -> 4594
```

The three broad daemon failures are the pre-existing correction-core
expectations for `расчет ыприблизительные`; none reaches the trigger FSM,
`ManualToggleV3`, committed-tail mutation, or layout postcondition. They are
recorded rather than hidden and do not invalidate this isolated ownership
repair. The user's physical keyboard confirmed the single-detector behavior;
the final installed-byte postcondition proof used a controlled evdev keyboard
and a real GTK field.

Runtime authority changed to `1.0.48`. No candidate producer, ranking,
correction policy, live config, L2 package, or exact V13 sidecar changed.
Global IBus was not restarted.

Verdict: `LAY_1_0_48_DOUBLE_SHIFT_SINGLE_OWNER_REPAIRED`.

Evidence:

- `docs/double-shift-physical-layout-contract.md`
- `docs/structural_gates/receipts/LAY_1_0_48_DOUBLE_SHIFT_SINGLE_OWNER_REPAIR_2026-08-28/RELEASE_RECEIPT.json`

## 1.0.49 Exact Pairwise Double Shift Contract

Release `1.0.49` supersedes only the burst semantics retained by `1.0.48`.
Gesture and mutation ownership remain single-owner, but there is no latch after
a completed pair:

```text
Left Shift press -> release -> press -> release -> one toggle -> Idle
next press -> release -> press -> release              -> one toggle -> Idle
```

Ordinary projection is the reversible physical-key table (`а <-> f`,
`привет <-> ghbdtn`). It has no lexical, candidate, model, or learning route.
The old optional multi-tap scope cannot delay `double-lshift`, and the Double
Shift detector has no post-pair debounce or quiet-window rearm.

The client-visible postcondition now closes the complete keyboard stack. After
the exact replacement is observed, one GNOME bridge owner activates the target
Lay input source. GNOME's input-source manager owns the corresponding IBus
engine transition; the replacement route does not launch an additional
`ibus engine` process. The former 25 ms deferred readback and a second
Rust-owned engine call are both absent. Success still requires both live states
to match.

Acceptance requires adjacent-pair proof in both directions and a four-fast-tap
round trip, with one daemon plan per pair, zero legacy IBus plans, matching GNOME
and IBus state, and the global `ibus-daemon` left running.

## 1.0.50 Nonblinking IME Surface Contract

Release `1.0.50` keeps the exact pairwise Double Shift behavior and removes two
visible intermediate transitions:

```text
printable input
-> invalidate old candidate authority immediately
-> retain the current preedit surface while matching work is pending
-> replace it once, or hide it once when the result is empty

Double Shift
-> exact text replacement
-> one GNOME Lay-source activation
-> one matching IBus focus handoff
```

The retained preedit surface is display-only. Tab cannot accept its stale
candidate after the token changes. A matching background result remains the
only route that can install the next selectable candidate. Ordinary typing must
not emit the previous `clear -> update -> show` sequence for every character,
and layout activation must not issue a redundant `ibus engine` command from the
source-change callback.

## 1.0.51 Staged Surrounding-Text Replacement Contract (Superseded)

The live GTK gate for `1.0.50` exposed a separate client-ordering defect:
legacy IBus emitted `DeleteSurroundingText` and `CommitText` back-to-back. GTK
could publish the appended intermediate state before applying deletion, and the
old suffix-only postcondition could incorrectly accept that transient state.

Release `1.0.51` makes a surrounding-text replacement a client-acknowledged
two-phase operation:

```text
exact pre-dispatch SurroundingText snapshot
-> DeleteSurroundingText
-> exact deleted snapshot observed
-> CommitText
-> exact final snapshot observed
-> one layout synchronization
```

No timer releases either mutation. A stale, selected, geometrically invalid, or
unrelated snapshot cannot dispatch the commit and cannot confirm the layout
postcondition. Atomic-effect and terminal-erase routes retain their existing
contracts. Focus/reset/capability loss clears any pending staged commit.

## 1.0.52 Nonblinking IME Output Contract

The installed `1.0.51` route preserved exact postcondition safety but exposed
its deleted-text phase to the client. That made Double Shift visibly blank and
reappear. It also allowed a newly selected Lay engine to discard the next exact
Shift pair while waiting for its first SurroundingText snapshot.

Release `1.0.52` supersedes only that output sequencing:

```text
exact pre-dispatch SurroundingText snapshot
-> arm exact full final postcondition
-> DeleteSurroundingText + CommitText from one input handler
-> exact full final snapshot observed
-> one layout synchronization
```

There is no deleted-snapshot wait and no suffix-only acceptance. An appended
transient such as `ghbdtnпривет` cannot satisfy the exact projected snapshot.
The daemon does not grab the physical device for this non-replay IME executor,
so every following complete Shift pair remains in the normal event stream.
The atomic engine path separately retains a pending parity bit only when its
exact source snapshot is not yet available.

Preedit display uses transition-only signaling:

```text
hidden -> visible    UpdatePreeditText + ShowPreeditText
visible -> visible   UpdatePreeditText only
visible -> hidden    one empty update + HidePreeditText
hidden -> hidden     no client signal
```

Printable input invalidates stale candidate acceptance immediately while the
old display frame remains until the matching background result replaces or
hides it. The contract forbids a per-key `clear -> update -> show` sequence.

## 1.0.53 Stable IME Completion Surface Contract

The installed `1.0.52` trace confirmed transition-only IBus signaling, but the
first one- and two-letter prefixes still produced unstable top candidates. It
also showed that typing the next character of a visible completion discarded
that full target and selected another one. The resulting suffix sequence made
the internal readout work visible even without a `hide -> show` blink.

Release `1.0.53` freezes the display policy independently of candidate
generation:

```text
prefix length 1..2       no visible completion
prefix length >= 3       first admitted completion may become visible
typed character matches  retain the same full target and shorten its suffix
typed character diverges decline the old target and admit a fresh result
worker pending           keep the last frame display-only; Tab stays disabled
```

There is no debounce, sleep, or delayed publish timer. Candidate authority is
still recomputed for every current input identity. The policy only prevents
ambiguous early results and intermediate worker state from becoming client
frames. A true boundary, empty current result, focus loss, or disabled text
assistance still performs one normal visible-to-hidden transition.

## 1.0.54 Autocomplete Double Shift Tail Ownership

After an IME completion is accepted, the resulting committed token remains an
IME-owned visible tail. The daemon remains the only physical Double Shift
detector, but `ManualToggleV3` dispatches the mutation to that IME owner:

```text
physical Left Shift pair
-> daemon gesture detector
-> ManualToggleV3
-> ImeCommittedTail
-> exact physical-key projection of the visible committed token
-> exact client postcondition
-> one target-layout synchronization
```

Only `DaemonWordBuffer` may return `DelegateDaemon`. `ImeCommittedTail` must
execute `toggle_committed_tail_target` and must not delegate to a daemon buffer
that never observed the accepted completion suffix. This route performs no
candidate generation, ranking, correction, morphology, or learning.

Evidence:

- `docs/structural_gates/receipts/LAY_1_0_54_AUTOCOMPLETE_DOUBLE_SHIFT_TAIL_OWNERSHIP_2026-08-28/RELEASE_RECEIPT.json`

## Pending Preedit Convergence Repair (2026-08-29)

TD-003 separates visible pending state from completion authority. A matching
retained target may be shortened synchronously while the next worker is
pending, but that suffix is display-only:

```text
published candidate "верка" for prefix "про"
-> type "в"
-> publish shortened display "ерка"
-> clear actionable candidates
-> matching current worker may publish new authority
-> late, cancelled, stale, or failed publication cannot be accepted
```

Tab, cursor arrows, and an Alt acceptance gesture retire a pending display
before they inspect completion authority. Alt retirement covers the complete
press/release gesture. Cancellation also clears deferred cursor-flush state,
so a later cursor acknowledgement cannot resurrect the retired frame. A
background result is first projected and published on a cloned engine state;
the live engine receives the candidate authority only after publication
succeeds. The atomic frame route remains synchronous and materializes its
candidate in the same submitted frame; the legacy route has no hidden
synchronous fallback.

The final managed GTK receipt proves both sides in one isolated desktop
transaction. Prefix `про` publishes `верка` and accepts exactly one completed
worker result as `проверка`. Prefix `пров` publishes `верка -> ерка`; the
second worker is late, Alt accepts nothing, and committed text remains `пров`.
Both cases have exact managed-key traces, one clear, zero malformed records,
and exact desktop restoration. Receipt SHA-256:
`8cde9837198ec4868a4fdd91e5e22723b0b3c58af78683647675e5e9d010b58a`;
manifest SHA-256:
`5917af92a2e71ef87dd0cea45a1e60eae46ee5bbc87ffd7fb00e2eef79e7e097`.
Diagnostic V1-V15 receipts remain immutable failed or superseded evidence.

What was tested: `275/276` `lay-ibus-engine` tests passed in the final full run,
all `88` focused preedit tests passed, as did the atomic printable-frame proof,
`43` runtime-smoke isolation tests, and the two-case managed GTK route above.
The sole full-run failure was the pre-existing TD-006 wall-clock assertion
`v27_component_latency_denominators`; an isolated rerun changed which timing
sub-gate exceeded its fixed threshold and did not fail IME semantics. What was
not tested: a production install, package release, or applications outside the
managed GTK harness. Runtime authority changed: **no**.

## 1.0.56 Pending Candidate Navigation Repair (2026-08-30)

The installed `1.0.55` trace ruled out candidate-field narrowing. Across the
latest `124` completed readouts, the backend returned up to `12` candidates,
averaged `6.68`, and returned at least `6` in `76` cases. All `124` workers
published before the `50 ms` display deadline. The visible regression was in
the key route: `Up` or `Down` pressed while a display-only refresh was pending
retired the preedit and escaped unhandled to the client, even though the exact
current token could produce multiple candidates.

Release `1.0.56` keeps normal printable input asynchronous and changes only
pending candidate navigation:

```text
Up / Down while pending
-> cancel the older background generation
-> materialize candidates for the exact current input identity once
-> 2+ candidates: cycle and publish the selected current candidate
-> 0/1 candidates: retire the display and pass the key through
```

The synchronous readout is reachable only from an explicit candidate arrow.
`Tab`, Alt and cursor `Left`/`Right` retain the fail-closed pending behavior and
cannot accept display-only text. Candidate sources, ranking, the L2/L3 route,
and `PREEDIT_RU_WAVE_CANDIDATE_LIMIT = 12` are unchanged.

What was tested before release packaging: the regression first failed under
the old route, then passed after the repair; all `18` pending-state tests and
all `278` `lay-ibus-engine` tests passed. Runtime authority changed at this
point: **no**. Final install and live-trace evidence is recorded by the release
transaction before publication.

The release transaction then built and installed `1.0.56`. In the isolated
managed-desktop smoke, physical `вариан` followed immediately by `Down`
produced one handled `candidate_select` event and changed the visible suffix to
`ты`; therefore the arrow cycled a current list containing at least two
candidates instead of dismissing it. The same trace contained candidate
readouts of `9` and `7`, had `0` malformed records, and restored the normal
desktop afterward. Installed and loaded daemon/engine SHA-256 values matched
the release artifacts, while the global `ibus-daemon` PID remained `4594`.

Verdict: `LAY_1_0_56_IME_PENDING_CANDIDATE_NAVIGATION_DEPLOYED_VERIFIED`.
Receipt:
`docs/structural_gates/receipts/LAY_1_0_56_IME_PENDING_CANDIDATE_NAVIGATION_2026-08-30/RELEASE_RECEIPT.json`
(SHA-256 `806a704f855b7b5b6254915d3039269dbf00af4ada4c519e1b94cce437943bea`).
Runtime authority changed: **yes, by the verified `1.0.56` release install**;
candidate sources, ranking, and the limit of `12` did not change.

## 1.0.57 Three-Character Suggestion Onset Preflight (2026-08-31)

The intended visible threshold is exactly three characters:
`PREEDIT_VISIBLE_PREFIX_MIN_CHARS = 3`. The installed `1.0.56` trace explains
the inconsistent observed onset. In the latest bounded window, `44` worker
results still matched their current input identity and `22` of those contained
at least one candidate. The `50 ms` display deadline admitted only `17/22`
positive results. The five positive current results rejected only for age
completed at `77.554`, `82.813`, `86.620`, `88.752`, and `146.115 ms`.

This is separate from a valid empty result. A completed word or a token that is
not a prefix of an admitted completion can return zero candidates; the display
must stay empty in that case. The repair must not invent candidates or expose
whole-token replacement proposals as suffix completion.

Three designs were considered:

1. Lower the visible threshold to two characters. Rejected: `1.0.53` already
   established that one- and two-letter top candidates are unstable, and the
   route would schedule more broad-prefix work.
2. Add a synchronous fallback prefix table or a second fast ranker. Rejected:
   it would block ordinary printable input, duplicate candidate ownership, and
   risk narrowing or reordering the shared L2/L3 field.
3. Admit an exact-current background result for up to `150 ms`. Selected: the
   measured window recovers `22/22` positive current results without adding a
   producer, cache, ranking rule, or worker invocation.

The selected route changes only the bounded presentation deadline. Existing
generation checks and the complete input-frame identity check are still
required before and under the engine lock. A result for an older token, focus,
layout generation, configuration, or worker generation remains discarded.
The first visible result may now appear between `50` and `150 ms`; results older
than `150 ms` remain late and cannot publish. The prior `203 ms` stale-surface
failure therefore remains outside the admitted window.

Frozen invariants:

```text
visible prefix minimum             3
candidate limit                   12
candidate sources/ranking          unchanged
whole-token replacement preedit    forbidden
ordinary printable input           asynchronous
generation/input identity checks   unchanged
zero-candidate result              no suggestion
CPU/RSS/cache/learning semantics    unchanged
rollback                           restore 50 ms constant
```

Required proof before release: threshold boundary tests at `150/151 ms`, the
full IBus regression class, the changed-file release gate, and an installed
managed-desktop JSONL trace proving a three-character suggestion while global
`ibus-daemon` remains unchanged.

### 1.0.57 Result

The implementation changed only `PRECOGNITION_DISPLAY_DEADLINE` from `50 ms`
to `150 ms`. The boundary test was red before the production change and passed
after it: age `150 ms` is fresh and `151 ms` is late. The hermetic non-timing
gate passed all `2,370` selected correctness and package tests with zero
semantic and infrastructure failures; its `lay-ibus-engine` target passed
`275/275`. A separate full engine run passed `277` tests and hit only the
pre-existing TD-006 wall-clock p99 assertion under concurrent desktop load;
the isolated serialized rerun passed at `3.700 ms` against its `4 ms` budget.

The installed managed-GTK case typed exactly `про`, published suffix `верка`,
and accepted it once as `проверка`. Its JSONL contains one exact-current
`applied` worker with `11` candidates, one visible preedit update, one
completion accept, `51` valid records, and zero malformed records. The test
restored the desktop, retained `lay-ime-ru`, and did not restart global
`ibus-daemon` PID `4594`. Daemon, IBus engine, CLI, and GNOME extension all
reported `1.0.57`; installed release-binary hashes matched the build outputs.

What was not established: every three-character string has a completion.
Exact zero-candidate prefixes still display nothing, and an exact-current
result older than `150 ms` remains suppressed. Candidate production, ranking,
the limit of `12`, cache ownership, and printable-key scheduling were not
changed. Runtime authority changed: **yes, by the verified `1.0.57` release
install**, limited to the longer exact-current presentation window. Evidence:
`docs/structural_gates/receipts/LAY_1_0_57_IME_THREE_CHAR_ONSET_2026-08-31/`.

## 1.0.58 Terminal Double Shift Single-Commit Preflight (2026-08-31)

The installed `1.0.57` trace proves that the shared planner selected the exact
projection `rjvvbn -> коммит`, but the terminal committed-tail route did not use
the existing IME terminal executor. It switched from `lay-ime-us` to
`lay-ime-ru` first and then emitted six physical Backspace taps followed by six
ordinary printable key taps. Those printable taps re-entered normal IME input,
focus, preedit, and worker processing. The observed partial result `оммт` and
the visible intermediate activity are therefore output-route failures, not a
mapping, detector, or candidate-selection failure.

The repair is capability-scoped:

```text
ImeCommittedTail
+ no SurroundingText
+ proven terminal cursor geometry
-> one terminal_erase_commit frame (DEL x N + exact projected text)
-> one immediate IME-owned layout synchronization
-> no daemon physical grab, Backspace replay, or printable-key replay

ImeCommittedTail + SurroundingText
-> retain the selected TD-009 exact observed-tail GTK route

DaemonWordBuffer
-> retain the explicit daemon/uinput fallback
```

This does not restore the rejected GTK legacy delete-plus-commit transaction.
The terminal executor is already a separate `CommittedTailOutputProfile` and
does not depend on a missing `SetSurroundingText` acknowledgement. The planner,
literal RU/US key mapping, physical Double Shift detector, candidate/ranking
authority, and autocomplete behavior remain unchanged.

Required regression proof before installation:

```text
rjvvbn -> коммит                         exact first/middle/last characters
коммит -> rjvvbn                         exact inverse
four Shift taps                          two inverse toggles
trailing boundary                        preserved once
terminal capability                      IME terminal executor selected
terminal daemon physical replay          unreachable
layout synchronization                   exactly once
GTK SurroundingText route                still exact observed-tail replay
global ibus-daemon restart                forbidden
```

Runtime authority changed at preflight: **no**. Rollback is the terminal-only
dispatch branch; no planner or shared correction data changes are admitted.

### 1.0.58 Result

The repair routes only the proven terminal `ImeCommittedTail` capability to
the existing `terminal_erase_commit` executor. The new regression was red
before the dispatch change (`Some(true)` expected, `None` observed) and green
after it. Targeted engine tests passed `6/6`; the terminal round-trip tests
passed `2/2`; transition-authority and input-gate contracts passed `21/21` and
`2/2`. The hermetic correctness/package lane passed all selected tests after
the test manifest was updated to `2,396` entries. The separate old TD-006
wall-clock performance assertion remains outside this semantic release gate.

An isolated installed-Kitty matrix then produced these exact client-visible
lines:

```text
rjvvbn + Double Shift                  -> коммит
коммит + Double Shift                  -> rjvvbn
rjvvbn + two Double Shift gestures     -> rjvvbn
rjvvbn<space> + Double Shift           -> коммит<space>
ghbdtn + Double Shift                  -> привет
```

The fresh engine trace contains six
`ibus_manual_toggle_dispatch(executor=terminal_erase_commit)` records, six
matching `ibus_committed_tail_replace` records, six requested and six
successful layout synchronizations, and zero committed-tail delegations. The
isolated daemon logs contain no physical Backspace or printable replay route.
The source, installed file, and loaded `/proc` engine SHA-256 are all
`b4bbdfff9fa9d2a4cd4066497413466443c56afbf6296f78030bc9087263ba55`.
Global `ibus-daemon` retained PID `4594`; only Lay-managed processes were
restarted.

What was not changed: the physical pair detector, key projection, candidate
production/ranking, correction policy, GTK/SurroundingText replay, package, and
V13 sidecar. Runtime authority changed: **yes, by the verified `1.0.58`
installation, limited to terminal committed-tail execution**. Evidence:
`docs/structural_gates/receipts/LAY_1_0_58_TERMINAL_DOUBLE_SHIFT_SINGLE_COMMIT_2026-08-31/`;
receipt SHA-256
`8c426fd6e096ed7b89677f805bbf31729f42439191f4ab8e364f07d264b8977c`.

## 1.0.59 Per-Character IME Suggestion Scheduling (2026-08-31)

The installed `1.0.58` trace separated two previously conflated causes of a
missing suggestion. Current-generation Russian completion results for the
observed `доп...` sequence arrived in `1.716-1.990 ms` with `12` candidates and
updated the visible suffix on each character after the old three-character
threshold. A separate `ghjdthrf` sequence completed in `3.081-93.612 ms` but
returned zero suffix candidates. Raising the display deadline cannot create a
candidate in the latter case.

`PRECOGNITION_DISPLAY_DEADLINE = 150 ms` is only a freshness ceiling for an
already calculated result. Printable input is committed immediately and the
readout remains asynchronous. Before publication, the worker generation and
the complete input-frame identity must still match; an older token, focus,
layout, configuration, or tail cannot publish merely because it is younger
than `150 ms`.

The actual onset defect was the independent
`PREEDIT_VISIBLE_PREFIX_MIN_CHARS = 3` UI admission gate. The shared candidate
engine already accepts one-character lexical prefixes. Release `1.0.59`
changes only that display-scheduling threshold to `1`. Every non-empty live
alphabetic prefix now schedules its own current-generation readout. Existing
matching-target retention continues to shorten a compatible visible suffix
immediately between worker completions.

Local evidence before installation:

```text
one-letter L2/L3 producer test       PASS, 12 prefix-preserving candidates
per-character display-ready test    PASS for п -> пр -> про
candidate readout warm p90           44 us
candidate readout warm max           55 us
lay-ibus-engine regression class     280 / 280 PASS
display deadline                     unchanged at 150 ms
candidate sources/ranking/limit      unchanged
whole-token replacement preedit      still forbidden
```

Scope: this guarantees a readout attempt and exact-generation publication
semantics for every non-empty live prefix. It does not fabricate a completion
for an arbitrary string whose admitted suffix set is genuinely empty. That
distinction is observable as `candidates=0` in the worker trace. Runtime
authority changed: **yes, by the verified `1.0.59` installation, limited to
first-character visible pre-cognition scheduling**. Shared candidate sources,
ranking, limits, and replacement policy remain unchanged.

Two isolated installed-runtime GTK scenarios passed. `п -> пр -> про`
published the exact visible suffix sequence `овод -> ивет -> верка`, then
accepted `проверка`. Continuing through `пров` published
`овод -> ивет -> верка -> ерка` without accepting a completion. The matching
worker generations returned `12`, `11`, and `11` candidates in the first case.
The working `lay-ime-ru` engine was restored with the exact installed engine
SHA-256
`a4b357fc58750c3fb21ba36008e89c8223bedf6ae8ec70bc23d3f85300262926`;
global `ibus-daemon` retained PID `4594`.

Evidence:
`docs/structural_gates/receipts/LAY_1_0_59_IME_FIRST_CHARACTER_SCHEDULING_2026-08-31/`;
release receipt SHA-256
`982fcc4c35cacf718f8653df21a38b63fc0fd12c22d29d5e219f60a054ea87b6`.
