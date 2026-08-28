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
