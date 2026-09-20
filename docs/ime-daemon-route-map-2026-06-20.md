# IME / Daemon Route Map

Date: 2026-06-20
Runtime baseline: `0.1.233`

## Current TD-126 source consolidation

The managed IBus window/context lifecycle now has one source boundary for
observation, existing authority admission, local execution or typed delegation,
later postcondition projection and RAII settlement/revocation. Exact scope,
review and final-gate receipt routing are in
[`TD-126`](../tech_debt/126-common-window-interaction-module.md); the installed
runtime remains unchanged.

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


## 2026-09-11 Alt+Shift and ASCII layout token recheck

The first 2026-09-11 Alt+Shift owner hotfix failed physical recheck. After each
manual Alt+Shift, the first word (`tot`, `yt`, `ytgjyznyj`, `b`) installed as
source-free `UnknownStart`; the next word in the same direction applied normally.
The fresh trace
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/ibus_engine_debug.jsonl`
shows `Reset`, `FocusOut`, `FocusIn`, GUI-like capabilities (`caps=41`) and
`purpose=0`, then terminal-like capabilities (`caps=9`) and `purpose=10`, then
`source_free_ready/source_free_installed unknown_start`. Current GNOME
`switch-input-source` bindings still contained the four direct Alt+Shift gestures
before the follow-up install. Together, those facts support the popup/focus-churn
attribution; no window identity measurement was taken for the popup itself.

Rechecked reducer fact: factory handoff is not prepared by `LayoutIntentToken`.
`open_factory_request` creates a factory `HandoffTicket` whenever live owner and
activation exist; source-free is used only when owner or activation is missing.
The adapter calls that reducer on `CreateEngine` independently of whether the
layout switch was initiated by the daemon or the IME. `LayoutIntentToken` only
revalidates/cancels layout work around the switch; it does not create transfer
authority.

Viable owner alternatives after recheck were bounded to two routes. The GNOME
native owner route failed physical recheck. An IME sole-owner route remains
possible, but would require returning the local legacy Alt+Shift layout route.
The selected route keeps the existing daemon direct GNOME `ActivateLayout`,
removes only GNOME's four direct Alt+Shift `switch-input-source` bindings at
runtime, and keeps legacy non-atomic IME Alt+Shift passive. Atomic exclusive IME
processing keeps its existing speculative route; uinput/daemon-owned text keeps
the existing daemon switch. No new authority API, cache, timer, transfer
promotion, or UnknownStart recovery is introduced.

The source supports daemon direct ownership: IME `switch_complete_layout_stack`
also calls GNOME `ActivateLayout`; neither route explicitly prepares a factory
transfer. The reducer creates transfer/source-free outcome in `open_factory_request`
on `CreateEngine`. The existing 90 ms daemon reconcile remains a postcondition
risk, but it first verifies the actual selected engine and should not switch if
that engine already matches.

The same recheck exposed a separate KnownStart coordinate defect. In rows
1395..1508, the completed token `ckf,j` reaches KnownStart and comma remains a
valid ASCII layout letter symbol. After a committed Space/manual transfer, the
next word starts as a separate whitespace-delimited token, but
`ibus_prediction_outcome` records `typed_prefix=ckf,jc`, `suggested=ckf,jcat`,
`final=j`. First shared loss: a transfer/handoff rebuild could restore
`preedit_fast` from `last_tail_token_text()` even when the committed tail ended
with whitespace, reviving the closed previous token for the next word. The next
coordinate loss is IME readout/frame capture preferring a live fast suffix or
falling back to `split_last_alphabetic_token`; both can slice a whole ASCII
layout word at punctuation-shaped letter keys.

Token repair alternatives were cache-only closure versus whole-authoritative
readout plus closed cache. The selected source repair does both: keep fast state
closed across trailing-whitespace rebuilds, and read the whole last whitespace
token when it is an ASCII layout-letter surface. This preserves the completed
`ckf,j` token as one token and prevents the following `c` from borrowing that
closed token, while leaving punctuation classification, reducer authority,
verifier and SafetyGate unchanged.

Build-only completed under
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/build-result.json`
with status `PASS_RUNTIME_BUILD_ONLY_GRAPH_UPDATED` in 136.82 s. Built runtime
binaries: daemon SHA
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e`, IME SHA
`86f5ea13549ffeb473bc70959b934d734406c9ed336fb5c3a06b71415ed6b96b`. Static
review is recorded in
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/static-review.json`;
download hash verification is recorded in
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/fetch-verification.json`.

Installation completed under
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/installation.json`
with status `INSTALLED_LOADED_HASH_VERIFIED_PHYSICAL_PENDING`. Loaded daemon PID
`2128412` has SHA
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e`; loaded IME
PID `2128417` has SHA
`86f5ea13549ffeb473bc70959b934d734406c9ed336fb5c3a06b71415ed6b96b`. The global
`ibus-daemon` PID `4715` was preserved. GNOME `switch-input-source` now contains
only `['<Shift><Alt>space']`; the four direct Alt+Shift entries were removed.
Backward bindings, XKB options, input-source IDs, and config SHA were preserved.

Rollback backup for this installation is
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/backup/` with
`lay-daemon`, `lay-ibus-engine`, and `gnome-bindings.json`. Older release backups
remain historical only. Tests/CI denominator is 0. Build-only and installation do
not establish quality, RSS, latency, heldout, or physical acceptance; physical
acceptance remains `PENDING` until live recheck.

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

## 1.0.60 Technical-Debt Closure Release (2026-08-31)

Release `1.0.60` packages the completed `tech_debt/` queue without adding a
new user-visible runtime route. The exact task glob contains `20` files and
all `20` declare status `DONE`. The release binds their already-reviewed
implementation and decision evidence to the Cargo, GNOME metadata, tray,
README, and versioning surfaces for `1.0.60`. Candidate sources, candidate
ranking, verifier authority, text-output authority, the selected engine, and
the canonical L2 package are unchanged relative to `1.0.59`.

Pre-install evidence was kept separate by denominator:

```text
changed correctness/package gate     2372 / 2372 PASS
semantic failures                    0
infrastructure failures              0
full release gate                     lay full check OK
lint non-dead diagnostics            0
release build                         PASS, 442 s
pre-install architecture refresh     PASS, 21,228 nodes
```

The first forward transaction was not accepted. Its verifier treated target
artifact names `lay-l11-restore` and `lay-l11-serve` as installed names even
though the installer contract maps them to `lay-l1.1-restore` and
`lay-l1.1-serve`. The initial rollback controller then attempted direct `cp`
over executing ELF destinations and received `Text file busy`. The briefly
mixed state was therefore quarantined rather than reported as a release.

Recovery V3 stopped only Lay-managed processes, restored every protected file
through a sibling temporary file plus atomic `mv`, and proved byte/tree and
live `/proc` parity with the retained `1.0.59` snapshot. The verified backup
contains `19` binary-tree files, `9` extension files, and `7` L2 files. Global
`ibus-daemon` PID `4594` was not restarted.

The second transaction used the corrected name mapping and an explicit
return-code path with the same full atomic rollback available on every
failure. All ten installed binaries match the release build by SHA-256 and are
mode `0755`; the nine-file installed extension exactly matches the source
tree. The canonical V13 package retained SHA-256
`cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b`,
and a fresh sidecar compilation matched the installed DAFSA SHA-256
`f116a230fc05a04c375bcddf1c85169276c6d149295eed8ccf93e37672a907b9`.

Installed and observed runtime projection:

```text
CLI / GNOME DBus version             1.0.60 / 1.0.60
GNOME DBus Ping                      pong from lay-extension
lay-daemon                           active, build/install/proc hash equal
lay-l3-online                        active, build/install/proc hash equal
managed lay-ibus-engine              exactly 1, build/install/proc hash equal
selected engine                      lay-ime-ru
global ibus-daemon PID               4594, unchanged
```

What was not tested: a new behavioral live-input scenario, because this
release adds no user-visible behavior over the already verified `1.0.59`;
installation on other GNOME versions; and the `public/main` publication route.
No new candidate-quality denominator or broader quality claim is introduced.

Verdict: `LAY_1_0_60_TECH_DEBT_CLOSURE_INSTALLED_VERIFIED`. Runtime authority
changed **yes**, limited to the newly installed Lay binary bytes and loaded
extension version. Runtime behavior authority did **not** change: candidate
production/ranking, verifier, text-output route, L2 package/sidecar semantics,
selected engine, and global IBus ownership remain as in `1.0.59`. Exact
evidence:
`docs/structural_gates/receipts/LAY_1_0_60_TECH_DEBT_CLOSURE_2026-08-31/RELEASE_RECEIPT.json`
(SHA-256
`08648fe12f1c3d7633690a05f4eb0393aa391b783c49aed08cefafb9b5c7088d`).

## 1.0.61 Correction-Safety Authority Release (2026-09-01)

Release `1.0.61` publishes TD-112 after the installed `1.0.60` runtime exposed
that the visible `Осторожно`, `Норма`, and `Смелее` setting stopped affecting
ordinary NANDA/fallback candidates before the final Apply owner. The setting
loaded correctly and already filtered registered deterministic rules, but the
active IME Space route used `CorrectionMode::NandaOnly`; the request-local
`TransitionDecisionPolicy` did not carry `CorrectionSafety`, and
`candidate_has_apply_authority()` did not consume it.

The selected minimal repair carries the existing profile value into the
existing request-local lattice policy and applies one pure corroboration rule
inside the existing DecisionCore authorization path. It adds no producer,
ranker, cache, service, verifier, edit plan, mutation route, package format, or
persisted migration. Grounded candidates remain in the bounded lattice when a
profile denies automatic Apply. Ranking, replacement surfaces, producer gates,
closed exact/L1.1 certificates, structural verification, `SafetyGate`, edit
validation, and the one text-output owner remain unchanged.

Task-local measured evidence before release preparation:

```text
TD-112 focused tests                  12 / 12 PASS
correction_safety focused tests        8 / 8 PASS
pinned IME preedit tests               3 / 3 PASS
InputGate public contract              6 / 6 PASS
changed correctness/package gate    2388 / 2388 PASS
known semantic failures                    0
infrastructure failures                    0
fresh-context code review             9.0 / 10 ACCEPT
task final full gate                  lay full check OK
```

This is a task-local authority and regression proof, not broad Russian/English
quality proof. Physical desktop correction behavior and the routed L4
exact-positive case were not established by TD-112. The fixed corpus separately
reports its pure-policy and routed denominators; aggregate success does not
promote an unmeasured language-quality claim.

Release-route options were evaluated as follows: publish the changed behavior
as `1.0.61` (**10/10**, selected); silently replace installed `1.0.60` bytes
without a version change (**1/10**, rejected because provenance and rollback
would become ambiguous); or defer the already-reviewed fix (**4/10**, rejected
because the user-visible control would remain weak).

The live pre-edit baseline was source/installed/DBus `1.0.60`, one managed IME,
selected engine `lay-ime-ru`, and global `ibus-daemon` PID dynamically observed
as `5126` at that audit instant. Installation
is a bounded transaction over the ten release binaries, nine extension files,
canonical V13 package and sidecar. Before any live mutation it must preserve a
complete byte-and-mode snapshot. A failed step stops only Lay-managed daemon,
L3 trainer, and IME processes; restoration uses sibling temporary files plus
atomic `mv`, reloads only the Lay extension, and must prove complete tree parity
before returning to `lay-ime-ru`. Global IBus must never be restarted.

The final release wrapper was run once from its first step after review. It
selected `2,388/2,388` correctness/package checks with zero semantic and zero
infrastructure failures, completed the optimized release build in `5m53s`,
and exited `0` with terminal line `== lay full check OK ==`. The resulting ten
binary SHA-256 values, controller bytes, extension ZIP, source version, and
retained `1.0.60` rollback bytes were pinned in the install preflight. Historical
V1 remained `BLOCKED_BEFORE_CODE` because its intermediate failure state was
not terminal; V2 closed that paper-contract defect and returned
`READY_TO_IMPLEMENT`, `safe_to_implement=true`, blockers `0` over `24` baseline
checks and `7` forbidden side-effect classes.

The accepted live transaction retained
`/home/ubu/.local/state/lay/release-backups/1.0.61-preinstall-R9xslOXQ` with
exactly `19` binary-tree, `9` extension, and `7` L2 files. The controller exited
`0` with `FORWARD_INSTALL_1_0_61=PASS`. All ten installed files match the pinned
release build by SHA-256 and mode `0755`, including explicit
`lay-l11-restore -> lay-l1.1-restore` and
`lay-l11-serve -> lay-l1.1-serve` mapping. The installed nine-file extension
matches the source tree. The canonical V13 package stayed at SHA-256
`cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b`,
and a fresh exact-sidecar compilation byte-matched installed SHA-256
`f116a230fc05a04c375bcddf1c85169276c6d149295eed8ccf93e37672a907b9`.

Independent post-install observation, separate from the controller's own
verifier, produced:

```text
CLI / GNOME DBus version             1.0.61 / 1.0.61
GNOME DBus Ping                      pong from lay-extension
lay-daemon                           active, PID 2117405, installed/proc hash equal
lay-l3-online                        active, PID 2117342, installed/proc hash equal
managed lay-ibus-engine              exactly 1, PID 2117674, installed/proc hash equal
selected engine                      lay-ime-ru
global ibus-daemon PID               5126, unchanged
```

The independent controller correction review returned `ACCEPT`, `9/10`, with
no High or Medium findings. Its one Low item is optional finer-grained test
pinning of every check before the already-correct rollback reactivation
barrier; it is not a release correctness blocker.

Verdict: `LAY_1_0_61_CORRECTION_SAFETY_INSTALLED_VERIFIED`. Runtime authority
changed **yes**: the installed binary bytes and loaded extension now carry the
reviewed TD-112 Apply policy. The intended behavior change is limited to
ordinary non-exact automatic replacement under `Осторожно` and `Норма`;
`Смелее` preserves the accepted `1.0.60` non-exact baseline. Candidate
generation, ranking, lattice retention, closed-exact authority, verifier,
`SafetyGate`, edit-plan validation, mutation ownership, package/sidecar
semantics, selected engine, and global IBus ownership remain unchanged.

Not tested by the live release transaction: physical typing comparisons of all
three profiles in a focused desktop application, other GNOME versions, broad
Russian/English language quality, and the routed L4 exact-positive case. Those
limits remain separate from the passed task-local policy/routed corpus and the
verified installed-runtime identity. Exact release evidence:
`docs/structural_gates/receipts/LAY_RELEASE_1_0_61_2026-09-01/RELEASE_RECEIPT.json`.

## Cross-app spontaneous input — 2026-09-09

<a id="cross-app-spontaneous-input-2026-09-09"></a>

Current verdict: `SENDER_ATTRIBUTED_ROOT_CAUSE_UNKNOWN_PHYSICAL_ACCEPTANCE_OPEN`. The user
reported that IME assistance fails in Kitty, some terminal windows, Tor Browser
and WeChat, and clarified that periods appear one at a time without typing;
deleting them creates a pause before they return. The user explicitly identified
this as an old fault, not the latest changes, and instructed work on 1.0.67.

The initial rollback to 1.0.66 was an incorrect response to that report. It was
reversed by reinstalling the same ten verified 1.0.67 binaries, without rebuilding
or changing the model packages/configuration. This correction does not fix or
disprove the reported behavior. Global IBus PID 4715/start ticks 2261 and the
two Lay input sources were preserved through both transactions. Current receipt:
`/home/ubu/.cache/lay/development/ime-cross-app-failure-_ul2eo18/reinstallation-1.0.67.json`.
The earlier `rollback.json` is retained as historical evidence, not rewritten.

Measured installation identity after restoration: IME `13db8623`, PID 3787313;
L1.1 `db825d2f`, PID 3787090; daemon `4e01703e`, PID 3787309; L3 watcher
`c75f0944`, PID 3787310. All ten file hashes, the four running executable
hashes, CLI version and loaded extension version matched the checked 1.0.67
artifacts. These prove installation and liveness only. Earlier fixed89, native13
and quiet-cadence proofs retain their original scope and do not establish
physical compatibility with the reported application windows.

Read-only diagnosis through 2026-09-09 01:34 UTC:

- A private existing trace snapshot has 2,882 valid records, 343 key records,
  117 `printable_managed_commit` period presses and 116 `managed_release`
  records, all with keysym 46/keycode 53. That key maps to a period in the RU
  physical layout. No other physical letter key was observed becoming a period
  in this snapshot. The clipped final press is not proof of a missing release.
- This trace has no event timestamps or application identity. Its period run
  cannot be attributed to the user's spontaneous episode; it may contain
  ordinary user input. One incomplete initial record is expected from its
  bounded file truncation. Raw user text remains in the private diagnostic
  directory, not in project documentation or new metadata.
- Separate passive observations of the physical keyboard, Lay virtual keyboard
  and RustDesk virtual keyboard did not capture spontaneous periods. The last
  simultaneous 60-second observation and a subsequent 180-second observation
  were entirely quiet. The latter IBus monitor fell back from unsupported
  `BecomeMonitor` to eavesdropping and had no positive traffic control; its
  empty output alone cannot establish delivery coverage. No grab or synthetic
  input was used; observers closed their devices and child processes on exit.
- Loaded libraries confirm both Kitty processes use Wayland; WeChat has an X11
  window. `XIM_SERVERS` on the active X display advertises `@server=ibus`.
  The different DISPLAY inherited by ibus-x11 does not by itself prove a broken
  bridge. Tor Browser was not independently identified in the running processes.
- The observed content-purpose values 0/10 permit assistance in current Lay;
  the historical terminal-purpose-sensitive refusal is not demonstrated here.
  Background prefetch, key release ownership, lost client release and an
  external injected event remain hypotheses, not established causes.

The release-consumption hypothesis was checked against the installed Kitty
version, 0.48.2. Its upstream `keyboardHandleKey` cancels the repeat timer on a
Wayland release independently of the asynchronous IBus reply. Merely changing
Lay's release return value is therefore not a demonstrated fix. Missing
delivery of the release to the client remains untested. Primary source:
https://github.com/kovidgoyal/kitty/blob/v0.48.2/glfw/wl_init.c#L415-L449.
The inspected upstream files and hashes are stored alongside the diagnostic
summary; upstream source inspection is not a reproduction of the live failure.

Next discriminating evidence is one actual spontaneous episode with coincident
kernel/device origin, IBus numeric event metadata and focused application
identity. A request to leave focus in an affected field is already pending;
do not repeat the request or treat elapsed quiet time as a negative reproduction.
Then trace the first layer that invents or retains the key, compare fixes before
editing production behavior, and add the smallest causal regression through
the real affected adapter. A literal period filter, debounce, blind release
policy reversal or model retraining is not an admitted repair. No runtime
behavior patch or new authority mechanism was selected during this diagnosis.

Evidence directory:
`/home/ubu/.cache/lay/development/ime-cross-app-failure-_ul2eo18/`.
Compact metadata: `diagnosis-summary.json`, `device-key-state.json`,
`passive-dot-source-counts.json`, `passive-dot-source-counts-v2.json`,
`passive-xinput-source-counts.json`, `simultaneous-input-origin-v1.json`,
`simultaneous-input-origin-v2.json`; bounded observer: `passive-input-origin.py`.
Runtime authority changed only in the documented rollback/restoration
transactions; the final runtime is the previously verified 1.0.67. This
read-only diagnostic phase changed neither code nor runtime authority. The
cross-app behavior, root cause, repair and physical acceptance remain OPEN.

### User-confirmed episode and sender attribution — 2026-09-09 01:46 UTC

The user reported “точки есть” while the bounded 15-minute observer was active.
It captured 96 period presses and 96 corresponding releases at the IBus
InputContext boundary, with the same events forwarded once to the engine.
The capture interval was approximately 01:45:50.315–01:45:53.186 UTC. All
periods had keysym 46/keycode 53; median spacing was 30.2 ms (range 28.0–32.3
ms). There were zero period key events on the physical keyboard or the two
observed virtual keyboards during that episode. Ordinary physical key events
were visible on both kernel and IBus layers, establishing a positive traffic
control absent from the earlier quiet observation. Do not count the two IBus
boundaries as separate user-visible characters.

The InputContext sender was `:1.1961`. IBus's private bus does not implement
GetConnectionUnixProcessID or GetConnectionCredentials. Instead, three harmless
Peer.Ping calls were correlated with bounded, payload-free syscall FD metadata:
`:1.1961 -> fd 14`, control `:1.3 -> fd 13`, `:1.1961 -> fd 14`. All three
returned empty successful replies. The independent Unix socket peer lookup
mapped IBus PID 4715/fd 14/inode 352547512 to WeChat PID 3734978/fd 99/inode
352544591, executable `/opt/wechat/wechat`. The tracer's timeout status 124 is
its declared four-second bound; it detached without restarting either process.
This establishes the process sending the periods into IBus, not the origin of
events before that process.

IBus modifier bit 30 distinguishes releases; three press/release pairs also
carried mouse Button1 bit 8. Counting only state==0 would incorrectly drop those
three presses. The complete count is 96. The subsequent final observer receipt
also includes one later ordinary physical period press/release; it is separate
from the captured 96-event spontaneous burst and must not be added to it.

Current X11 repeat rate is 33/s with 500 ms delay; GNOME's repeat interval is
30 ms and delay 500 ms. The cadence is consistent with X11 auto-repeat, but this
is still a hypothesis. A later XQueryKeymap was empty after focus had returned
to Kitty, so it cannot establish the X11 key state during the burst. The captured
focus query was also after the burst; it does not prove that WeChat was sending
into a different foreground application. A focused-app clarification is pending.

Next observation collects XInput events alongside numeric IBus event paths and
focus lifecycle metadata. The first IBus period of each burst triggers one
read-only focused-window query; no periodic focus RPC is added. This distinguishes
X11-delivered repeats from a client-local source and captures the actual affected
application at the event. No literal period filter, changed release policy,
process restart or production code patch has been applied on this evidence alone.

Exact private evidence, relative to the diagnostic directory above:
`incident-user-confirmed-0146.json`, `incident-attribution-summary.json`,
`ibus-connection-fd-attribution.json`, `ibus-connection-fd-attribution.txt`,
`x11-current-keymap.json`, final `simultaneous-input-origin-v3.json`.
Bounded follow-up observers: `passive-x11-period-origin.py` and
`passive-input-origin-v4.py`; outputs `x11-period-origin-live.json` and
`simultaneous-input-origin-v4.json`. This diagnostic milestone changes no
runtime authority and does not close the requested fix or physical acceptance.

### Coverage correction — additional receiver keyboard endpoint

The initial three-device observer did **not** cover every keyboard-capable
endpoint. Inventory of `/proc/bus/input/devices` found a fourth device advertising
KEY_DOT/KEY_SLASH: `2.4G Mouse`, event5, USB vendor/product `1ea7:0066`,
interface 00, `ID_INPUT_KEYBOARD=1`. Thus the statement that the captured periods
did not originate from any input device was premature. The actual established
negative is limited to event3 (built-in keyboard), event21 (Lay) and event22
(RustDesk). The IBus sender-to-WeChat attribution remains valid.

An event5 key-state read at 02:03:51 UTC was empty outside the incident; it does
not exclude receiver events during the burst. The corrected observer enumerates
every endpoint whose current KEY capability mask includes keycode 52 or 53,
records that inventory and monitors all four read-only. It is
`passive-input-origin-v5.py`, output `simultaneous-input-origin-v5.json`; XInput
companion `passive-x11-period-origin-v2.py` writes
`x11-period-origin-live-v2.json`. The earlier v4 observer was stopped after v5
started, and its receipt is preserved. The first XInput observation completed
quietly and is not an incident reproduction.

The source of repeated requests before WeChat remains UNKNOWN: receiver/device,
X11 or client-local. Neither the receiver nor WeChat is yet established as the
root fault. No device was disabled and no production behavior was changed on
the basis of the incomplete first coverage.
## Rare IME suggestions after the spontaneous-input report (2026-09-09)

The user reports that the periods stopped and explicitly requested that the
counter be stopped. Both transient counter units are inactive with MainPID 0;
their final v7 kernel/IBus and v5 X11 reports are FINISHED. No production input
policy was changed by the counters. Their stopping or the disappearance of
periods is not a causal repair verdict. The remaining requested behavior is
visible IME suggestions and ordinary Space autocorrection.

Current runtime observation: installed IME SHA `13db8623` is loaded by PID
123803 under the IBus session; daemon PID 123777 loads `4e01703e`. These are
the same 1.0.67 bytes. The daemon journal records three restarts at
02:47:44–02:47:48 UTC; the actor is unknown and these restarts were not part
of the counter commands. The selected engine is lay-ime-ru; suggestion and
auto_replace settings are true, safety profile experimental.

Preserved evidence lives in
`~/.cache/lay/development/ime-cross-app-failure-_ul2eo18/`:
`ime-no-hints-existing-trace.jsonl` is a private snapshot of the existing opt-in
trace, modified 02:49:51 UTC, 2690 valid rows plus one clipped initial fragment.
The focused application readback is Kitty PID 272166. This trace has no
per-row timestamps or application identities, so that readback does not label
every historical row. It contains 98 managed printable presses, 43 correction
preparations, and three completed display calculations. At rows 1730–1859,
eight letters and their releases complete with KnownStart, but no cursor
notification occurs. SetCursorLocation at row 1866 is followed by a visible
preedit and its completed worker at rows 1867–1869. Geometry is 11×24.

The current source puts a matching display frame in pending_display_frame
instead of scheduling it when FocusInId is absent, SurroundingText is absent,
and a cursor width exists. Only a later SetCursorLocation flushes that frame.
This is a candidate causal mechanism for rare hints; the controlled proof
must first reproduce it with the installed bytes. The raw trace does not
prove this is every client's only failure.

Autocorrection is a separate denominator. Nine recorded attempts contain
four rank refusals, one infrastructure refusal, and four not-ready outcomes.
All four not-ready outcomes have generation zero, including repeated spaces
and boundaries after editing. They must not be described as four measured
deadline overruns or nine failed typo restorations. Current fixed89 and
known-phrase quality receipts retain their original scope.

Preflight before any production edit: reuse the existing remote guarded
actual-IBus harness, immutable 1.0.67 candidate and nine-role dependencies.
Match the observed client capabilities (no SurroundingText, FREE_FORM,
11×24 geometry), suppress only subsequent cursor-location notifications,
then issue one explicit cursor notification for the same unchanged prefix.
Record the display before/after within the existing 150 ms product deadline,
the current owner/tail identities, real preedit signals and consumer effects.
Also run the fixed five-word cadence sequence with FREE_FORM to distinguish
delivery from model verdicts. This is a transport reproduction, not GUI or
physical-keyboard acceptance and not a heldout quality proof.

Provisional designs, ranked as engineering estimates before reproduction:

1. **9/10:** let an already admitted live word/frame schedule its worker
   without the extra cursor gate; retain legacy behavior without admission.
   Reuse ContextAdmission and existing result identity checks. No new owner,
   timer, cache, fallback or authority source.
2. **7/10:** remove cursor gating and its deferred state for every legacy
   consumer. Less state, but a wider compatibility change without current
   evidence for all clients.
3. **3/10:** add a cursor-ack timeout. Adds another timer and cancellation
   boundary, keeps the unnecessary dependency and delays hints.

The selected implementation remains pending the causal reproduction. Any
change must keep lexical candidate generation, ranking, false-authority and
SafetyGate/verifier rules unchanged; a display signal never authorizes Tab
unless current worker publication succeeds. Scheduling more admitted prefixes
can increase CPU/allocation work, so retain the bounded replacing worker and
150 ms age limit and measure completed/superseded jobs and RSS. Existing
package/config identities, reload invalidation, focus/epoch checks and online
feedback ownership stay binding. Test focus/layout changes and stale deferred
cursor notifications, plus GTK, terminal and atomic consumers. Roll back only
the new source change if fixed proof or required classes regress. Broader
removal of legacy deferred state is a separate maintenance change. No runtime
authority changed during this analysis or planned private reproduction.

Reproduction update, 03:02–03:16 UTC: the fresh-IBus v1 control completed in
5.752 s with all five correction surfaces correct and a visible prefix
suggestion before an explicit cursor notification. Its FocusInId receipt was
present, so it does not exercise the suspected legacy cursor gate. Receipt:
`ime-cross-app-failure-_ul2eo18/cursor-notification-baseline-v1.json` (local),
`autocorrect-ojoasco5/phrase-cursor-notification-baseline-v1/receipt.json`
(remote). The earlier live snapshot has 13 FocusIn, four FocusOut and zero
FocusInId callbacks; the current engine nevertheless advertises FocusId=true.
Official IBus engineproxy source caches the advertised flag per engine name in
ibus-daemon. This explains a possible persistent protocol difference; the
actual live cache value has not been read and is not claimed proven.

The next private control primes the real isolated IBus with an older false
FocusId capability before starting the unchanged candidate under the same
engine name. This is an explicitly controlled legacy cache condition, not a
production restart or a clone of the user's in-memory daemon. Initial v2/v3
seed-factory setup attempts returned Cannot find engine, and v4 timed out
before any product cases. Those receipts remain FAILED with zero cases, not
runtime failures or acceptable reproductions. Seed startup now has its own
bounded diagnostic log to identify the setup failure. Production editing and
selection of a repair remain pending a discriminating reproduction.

Causal reproduction completed at 03:17 UTC with the exact installed candidate
`13db8623`: private actual IBus, preserved across an explicit old-capability
seed, sends plain FocusIn to the replacement engine even though its current
FocusId property is true. The real admitted word has no native focus receipt.
After ` про`, no visible preedit arrives during 152033 us of observation.
One SetCursorLocation produces `верка` after 2907 us, with identical engine,
word epoch, text and focus receipt and no text edit. All five ordinary
correction controls pass under the same legacy route: both supplied typos
restore, three clean words stay intact. This establishes the cursor dependency
for suggestions, not a reproduction of the user's autocorrection refusal.
Exact local receipts: `ime-cross-app-failure-_ul2eo18/` containing
`cursor-notification-baseline-v6.json` and
`cursor-notification-regression-red.json` (`EXPECTED_RED`). Remote evidence:
`autocorrect-ojoasco5/phrase-cursor-notification-baseline-v6/`.

Setup failures v2–v5 were private seed preparation failures with zero product
cases. The decisive startup log identified an old remote libibus without the
has-focus-id property; v6 uses only a small GDBus old-capability seed and then
the actual unchanged Lay engine. The seed neither implements prediction nor
handles the tested word. All failed receipts are retained.

Selected route before code: design 1 (9/10). Add only the context-admission
condition to the existing legacy cursor gate. The sole production caller first
requires a matching InputFrameIdentity, which includes a currently revalidated
KnownStart word; the existing worker repeats identity and age checks before
publishing. No extra admission lock is added by the gate itself. Required but
cancelled/unknown admission still fails the preceding frame guard, and is never
made authoritative by cursor metadata. The no-admission legacy path keeps its
prior cursor wait. This is a scheduling change: more valid prefixes may reach
the existing replacing worker. Its bounded queue, 150 ms freshness limit,
lexical material, ranking, feedback and mutation verifiers are unchanged.
Native v6 is the fixed RED/GREEN regression; existing scoped correctness,
unknown/stale/atomic/GTK/terminal controls, fixed89 and release checks remain
required. Installation remains pending independent review and verification.

### Candidate verification, before release (03:31 UTC)

Only `engine.rs::preedit_waits_for_cursor_ack` changes runtime behavior: the
existing cursor wait also requires absent ContextAdmission. Scoped formatting,
449 discovered IME correctness tests and release IME build passed on the
remote dedicated-20cpu worker. Three performance tests were explicitly excluded
by the existing focused route. Receipt: `ime-cross-app-failure-_ul2eo18/`
`cursor-candidate-build-receipt.json`; remote `run-WMSNYB/ime-cursor-candidate/`.
Private candidate SHA `f1b78320`; all other 696 Rust files match installed 1.0.67.

Actual private IBus v6 RED/GREEN passed: the installed baseline needs cursor
notification; the candidate emits a current nonempty visible hint during
typing, before that notification, with no FocusInId and the same admitted word.
The reported 9 us wait return is not suggestion latency: the signal had already
arrived while typing. The later redundant cursor event emits no extra signal;
existing signal deduplication preserves the visible display. Both candidates
pass all five ordinary correction controls. Receipts:
`cursor-notification-candidate-v6.json` and
`cursor-notification-regression-green.json` in the diagnostic directory.

A separate width-1 terminal control changes only cursor geometry from 11 to 1;
inverse reconstruction proves all other driver bytes unchanged. Both variants
show 34 printable key events as `terminal_passthrough`, handled=false; the
client applies native characters after each RPC returns. Baseline still needs
cursor notification (151472 us observation, then 2366 us to a hint); candidate
displays a current hint before it. Their five correction surfaces are identical:
three clean preserved, two typos unchanged. This does not pass Space correction
for the narrow terminal route: `managed.rs` deliberately returns Space to the
client without IME prefetch/correction; the private harness has no physical
keyboard daemon. Full desktop terminal correction remains outside this proof.
Remote receipts: `autocorrect-ojoasco5/phrase-cursor-terminal-{baseline,candidate}-v1/`.
The first terminal command failed its candidate-file precondition before
launching any product case; its nonexistent path was corrected to the immutable
release-binaries directory. No failing product case was retried or excluded.

Independent fresh-context review, round 1: 9/10, H0/M0/L0. No repair requested.
It checked the sole caller, live KnownStart/frame capture, repeated publication
identity and age checks, pending Tab refusal, no-admission compatibility, and
atomic route. The one running and one replaceable pending worker remain bounded
in queue length; superseded computation can still finish. The 150 ms limit is
a publication freshness limit, not a CPU execution bound. More scheduling is
expected and CPU/RSS must be reported separately from correctness.

Release preparation now targets 1.0.68 because runtime behavior changed after
the installed 1.0.67. Mandatory full checks, fixed89 in all three safety profiles,
native controls and a fixed repeated legacy-focus cadence gate will bind to
the final release bytes before installation. Existing L1/L2 algorithm/package
proofs retain their prior exact source and corpus scope; no model material or
ranking change is part of this repair. No production authority changed yet.

### Mandatory release prefix and Cargo cache boundary (03:51 UTC)

The frozen 1.0.68 run `run-OfEiLW` passed architecture, both lint contracts,
2675 correctness plus 36 package tests, extension syntax and helper syntax.
The full command then exited 75 at CLI explain because Cargo target reached
12896141312 bytes, beyond the unchanged 12884901888-byte guard. This is a
build-resource failure after 2711 successful tests, not a test-quality failure.
Original receipt remains `run-OfEiLW/td123-full-acceptance-completed-identity.json`
with status FAIL; local copy is `run-hvrxwcxk/` with the same filename.
The initial full attempt took 507.450 s. The two lint inventories retain every
normalized entry (535 default, 359 research); only byte locations shifted.

Under a fresh heavy-execution lease, a separate continuation verified the
original snapshot and test summary, then removed only unused disposable
`/home/e/projects/lay-td119-gate-v1/target/test-lanes/debug` (2461863936 bytes,
directory dated September 6, no process executing from it). It preserves
source, frozen release artifacts and all proof receipts. The unchanged full
script suffix from CLI explain through the research release build passed in
220.211 s. The final default-feature IME build is the remaining step at this
checkpoint. The continuation writes a distinct
`run-OfEiLW/ime-cursor-full-acceptance-completed-identity.json`; it does not
rewrite the initial failure. Existing test results are reused only for the
same 697 Rust files, Cargo/toolchain, manifests, fixtures and environment.
The target budget was not raised. No installed runtime authority changed.

### Final 1.0.68 acceptance before installation (04:06 UTC)

The budget continuation passed in 397.347 s, including 176.519 s for the
final default-feature IME. Exact candidate SHA:
`2a2df132e44142be4b1f273122c7d3391a0ac7e1fbe7a5a1353dc65d91ac3219`.
All ten immutable binaries and all 697 source files were verified after
transfer. New L1.1 service SHA `570f35b9`; only its release-version rebuild
changes the native dependency manifest, while eight model/receipt dependencies
remain exact. The initial exit-75 receipt remains separate from the successful
`ime-cursor-full-acceptance-completed-identity.json`.

Final actual-IBus acceptance took 401.668 s: all 15 fixed-profile shards
completed, and independent comparison against installed 1.0.67 found no lost
correct outputs, new false outputs or per-class regressions. Each comparison
recomputed 178 statuses. The proven v11 incomplete-line reader correction
is normalized by inverse byte reconstruction; fixtures/events/deadlines and
all model/config inputs match. Exact comparisons: local
`autocorrect-live-ojoasco5/ime-cursor-fixed-profile-{strict,normal,experimental}-comparison-v1.json`.
These 89 curated Cyrillic fixtures are not the full L1 heldout or a universal
quality guarantee. The prior complete L1/L2 proof remains dated 1.0.67 evidence
for unchanged model code and package bytes; its existing performance failures
remain recorded, not promoted to new PASS results.

| Profile | Class | 1.0.67 correct | 1.0.68 correct | 1.0.68 percent |
|---|---|---:|---:|---:|
| strict | all_dirty | 15/47 | 15/47 | 31.9149% |
| strict | all_clean | 41/42 | 41/42 | 97.6190% |
| strict | clean_missing_letter_control | 3/4 | 3/4 | 75.0000% |
| strict | clean_repeated_letter_control | 4/4 | 4/4 | 100.0000% |
| strict | clean_valid_word | 34/34 | 34/34 | 100.0000% |
| strict | context_fixture | 0/6 | 0/6 | 0.0000% |
| strict | missing_letter | 3/8 | 3/8 | 37.5000% |
| strict | repeated_letter | 0/4 | 0/4 | 0.0000% |
| strict | restoration_regressions | 7/24 | 7/24 | 29.1667% |
| strict | transposition | 5/5 | 5/5 | 100.0000% |
| normal | all_dirty | 19/47 | 19/47 | 40.4255% |
| normal | all_clean | 41/42 | 41/42 | 97.6190% |
| normal | clean_missing_letter_control | 3/4 | 3/4 | 75.0000% |
| normal | clean_repeated_letter_control | 4/4 | 4/4 | 100.0000% |
| normal | clean_valid_word | 34/34 | 34/34 | 100.0000% |
| normal | context_fixture | 0/6 | 0/6 | 0.0000% |
| normal | missing_letter | 5/8 | 5/8 | 62.5000% |
| normal | repeated_letter | 0/4 | 0/4 | 0.0000% |
| normal | restoration_regressions | 9/24 | 9/24 | 37.5000% |
| normal | transposition | 5/5 | 5/5 | 100.0000% |
| experimental | all_dirty | 20/47 | 20/47 | 42.5532% |
| experimental | all_clean | 40/42 | 40/42 | 95.2381% |
| experimental | clean_missing_letter_control | 3/4 | 3/4 | 75.0000% |
| experimental | clean_repeated_letter_control | 4/4 | 4/4 | 100.0000% |
| experimental | clean_valid_word | 33/34 | 33/34 | 97.0588% |
| experimental | context_fixture | 0/6 | 0/6 | 0.0000% |
| experimental | missing_letter | 5/8 | 5/8 | 62.5000% |
| experimental | repeated_letter | 0/4 | 0/4 | 0.0000% |
| experimental | restoration_regressions | 10/24 | 10/24 | 41.6667% |
| experimental | transposition | 5/5 | 5/5 | 100.0000% |

Native13 passed with the final service. Eight fresh legacy-focus managed
controls at 80 ms cadence all displayed a current hint before cursor metadata,
without FocusInId, and yielded 40/40 expected correction surfaces: both typos
8/8 each, clean words 24/24. Final narrow passthrough likewise displayed the
hint; its unchanged 3/5 correction surfaces retain the missing physical-daemon
limitation above. All client processes were reaped. Local receipt hashes and
visible surfaces were independently rechecked after download.

A separate sampled legacy resource pair kept the final L1.1 service fixed.
Baseline/candidate display worker completions: 1/31, all published; total
display calculation 758/340497 us, maximum 758/41005 us. Sampled cgroup CPU
5920935/6143365 us; process VmHWM 415696/410764 KiB; sampled cgroup peaks
422756352/416997376 bytes; OOM 0/0. This is one instrumented five-word pair,
not a general CPU improvement or a hard latency/RSS bound. Quiet cadence
results have no resource sampler. Raw receipts are under
`run-hvrxwcxk/ime-cursor-native-acceptance/` and `cursor-client-evidence/`;
compact independently checked summary: `cursor-release-evidence-summary.json`.

The second fresh review covered the installation transaction: initial 7/10,
H0/M3/L0. All three findings were repaired before execution: mark the native
handoff attempt before sending it; prevent failed receipt writes from skipping
recovery; verify restored process hashes/activity and extension version before
claiming restoration. Seven synthetic fault-path controls execute the actual
exception branch and restored-runtime verifier AST against fake providers; all
passed. They issue no live service/filesystem/input operations. Receipt:
`run-hvrxwcxk/transaction-failure-paths.json`, bound to installer SHA `894654f0`.
No third review was requested; this respects the two-round limit.

Installation preflight passed read-only. The current IME is a verified child
of global IBus, so the transaction preserves that ownership: native input
handoff, stop only that exact child and three Lay services, install backed-up
verified binaries, then let preserved IBus start the new IME. Source/installed/
loaded versions, four process hashes, IBus PID/start identity, model hashes,
config and input sources must all agree before installation is accepted.
Physical input in the user's applications is still pending. No production
authority changed during the checks recorded in this section.

### Installed 1.0.68, physical acceptance pending (04:08 UTC)

The reviewed transaction completed: `INSTALLED_VERIFIED_PHYSICAL_PENDING`.
Installed source/CLI/loaded extension version is 1.0.68; all ten file hashes
and the loaded executable hashes of all four Lay processes match the immutable
release manifest. IME PID536035 (`2a2df132`) is again a child of the unchanged
global IBus PID4715/start identity. L1.1 PID535803 (`570f35b9`), daemon PID536029
(`b17a4fc7`) and L3 watcher PID536030 (`964f2265`) load accepted binaries.
Model/receipt dependencies, user config and input-source list are unchanged.
This installation changes runtime code authority; previous private proofs did
not. No model authority or package promotion occurred.

Exact receipt: `~/.cache/lay/development/run-hvrxwcxk/installation-1.0.68.json`.
Backup: `~/.local/state/lay/release-backups/1.0.68-td123-4h4357ne/`.
Two physical questions were issued only after installation: current suggestion
after a known boundary, and ordinary-tempo correction of the supplied phrase.
They remain pending. Private IBus success and process liveness do not prove
Kitty/Tor/WeChat GUI behavior, nor the broad TD-123 quality goal. The user
stopped the period counters; they remain stopped and are not re-enabled here.


## Physical acceptance and candidate counts, 2026-09-09

At 07:19 UTC the user confirmed ordinary typing with «работает !» and
explicitly authorized push. The installed 1.0.68 delivery milestone is DONE.
This accepts the current physical IME workflow; it is not a universal desktop,
restoration-quality or performance verdict. General TD-123 quality remains OPEN.
The release is prepared on `origin/codex/cleanup-20260908`; the remote ref and
publication receipt identify the published commit without another version bump.

Before publication, a read-only check again matched all 697 Rust input hashes,
ten installed executable hashes and four loaded executable hashes to the
accepted release. Global IBus PID 4715 and all four Lay process start identities
were unchanged; both cancelled input counters had MainPID=0. The first ad-hoc
readback used the build name `lay-l11-restore` as its installed filename and
stopped before completing: the existing installer uses the public alias
`lay-l1.1-restore` (and likewise `lay-l1.1-serve`). Applying that already verified
alias map completed the check. No installation or runtime mutation occurred.
Exact receipt:
`~/.cache/lay/development/run-hvrxwcxk/publication-1.0.68/pre-push-readback.json`.

The user's candidate-count question was answered by inspecting the current
readout path and the existing isolated native trace for accepted IME SHA
`2a2df132e44142be4b1f273122c7d3391a0ac7e1fbe7a5a1353dc65d91ac3219`.
This is a read-only analysis of an earlier resource-observed five-word control,
not a fresh measurement of the user's personal typing.

| Stage | Current contract / measured scope |
|---|---|
| Internal material | Variable merged field. Lexical material requests 12, expands to 24 when thin; canonical material also receives that bounded request. Verified repair, layout and boundary sources may add material; L3 context births request up to 4. These request limits are not the total raw field count. |
| Shared word decision | Returns up to 12 admitted candidates; reserves source lanes, ranks and deduplicates. |
| IME projection | Removes whole-token replacements from passive preedit; Experimental may add up to 6 phrase suffixes, then declined-target and duplicate filtering applies. Upper bound 18, not a promise of 18 candidates per letter. |
| Visible output | One selected continuation; the retained list supports Up/Down selection. A pending obsolete result has no Tab authority. |
| Scheduling | One in-flight computation and one replaceable pending request. Superseded or stale frames cannot publish, so physical key count is not completed-inference count. |

Measured final-list counts for the `проверка` fixture in its existing context:

| Prefix | Candidates delivered to IME |
|---|---:|
| п | 12 |
| пр | 11 |
| про | 11 |
| пров | 12 |
| прове | 10 |
| провер | 12 |
| проверк | 2 |
| проверка | 2 |

All 31 worker completions in this trace were applied. Final-list distribution:
0 candidates on 8 completions; 2 on 2; 9 on 1; 10 on 2; 11 on 9; 12 on 9.
The zeroes occur on non-prefix typo continuations in the fixed control; its
separate Space correction still restores the two supplied typos. Display
completion and correction are distinct contracts.

The trace's `candidates` field counts the final materialized IME list. It does
not retain raw model count per prefix. `LiveGateRecord` accumulates raw counts
in process statistics, while the per-prefix timing record lacks that field;
therefore the exact earlier raw count is UNKNOWN. Do not describe the 12/24/64
request/cap constants as measured total model output, or claim that list size
proves answer quality. No source, model, calibration, test, runtime authority or
candidate-selection policy changed for this analysis.

Exact compact analysis and frozen trace provenance:
`~/.cache/lay/development/run-hvrxwcxk/publication-1.0.68/candidate-counts.json`.
The original installation and acceptance receipts remain unchanged, including
their historically correct physical-pending status at installation time.


## Missing typed suggestions restored by arrows, 2026-09-09

After accepting and publishing 1.0.68, the user reported that `прове` has no
visible suggestion during typing, but Up/Down makes suggestions appear. This
reopens the automatic-display acceptance for that reported scenario; the
published release, its earlier user confirmation and measured fixed proofs
remain historical facts.

The existing live trace was copied without enabling another observer. For the
last retained `прове`, generation 622 / tail epoch 477, the admitted worker
returned 10 final candidates in 2,535 us and applied at age 2,572 us. The engine
emitted suffix `рка` before the Up/Down events, then those events changed the
same selected-list surface. The trace has ordering but no per-event wall time
or client render acknowledgement. Read-only focus inspection identified Kitty
PID 272166; its loaded backend is `kitty.glfw-wayland.so`, version 0.48.2.
Exact evidence: `~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/`.

The upstream source for the installed version contains a discriminating
consumer hypothesis. `glfw/wl_text_input.c::text_input_done` sends preedit before
commit, while `kitty/keys.c` clears the overlay on commit. Equal preedit text is
then deduplicated against `current_pre_edit`, which still remembers text that
has been cleared from the screen. A different arrow-selected suffix can show
again. Wayland text-input-v3 specifies insertion of committed text before the
new preedit. This explains the observed distinction if the two signals are in
one compositor packet; packet grouping was not directly captured live.

Before changing any production code, prove that sequence against the original
upstream callback bodies, including unchanged-prefix, separate-packet,
empty-commit and stale-serial controls. The proof must distinguish callback
and final display-sink effects from an actual desktop pixel observation.
No model, ranking, corpus, SafetyGate, verifier or installed runtime changes
have been made for this investigation. Both cancelled counters stay stopped.


### Original consumer proof and consequence analysis before patch

The remote ASan/UBSan callback proof compiled exact bodies of Kitty 0.48.2
`send_text`, `text_input_preedit_string`, `text_input_commit_string`,
`text_input_done`, and the three IME cases from `kitty/keys.c`. Its backend
sinks record committed bytes, final overlay text and cursor feedback; this is
an original-consumer sequence proof, not a desktop pixel test.
`baseline-v2/receipt.json`: 13 cases, 4 expected semantic failures, 9 controls
PASS; sanitizer stderr empty. Both new and unchanged preedit are lost after a
combined commit, including an empty commit and stale-serial packet. Separate
packets, explicit clears, deduplication without commit, arrow selection,
no-focus and empty packets retain expected behavior. The first `baseline-v1`
failed compilation because the diagnostic debug macro discarded arguments;
its replacement logging sink consumes them. Upstream callback bodies and
assertions were unchanged. Both attempts are preserved under the incident
folder locally and `/home/e/projects/lay-development-runner/kitty-preedit-qggkm5wl/`.

| Design | Score | Consequence |
|---|---:|---|
| Repair Kitty's Wayland commit/preedit ordering and existing display-cache invalidation | 9/10 | Fixes the failing consumer and protocol contract in one function, without a new timing owner. Chosen for an isolated candidate. |
| Delay or repeatedly republish Lay suggestions | 4/10 | Timing cannot prove compositor packet separation. Cursor acknowledgements may be absent; repeated frames can create feedback and return the starvation just repaired. Rejected. |
| Switch Kitty to another display/input backend | 6/10 | Avoids this consumer but changes a wider terminal route and requires new-window acceptance. Keep as an alternative, not the primary implementation. |

Chosen change: process pending commit before the new preedit; invalidate the
existing `current_pre_edit` cache at commit because the actual commit callback
clears that display. Then reuse the existing preedit comparison and serial
handling. No literal prefix or suggestion enters the runtime condition.

Consequence check:
- Candidate retention, ranks, false authority, model/package reloads, learning
  and feedback remain governed by the unchanged Lay pipeline. The consumer
  preserves the exact committed bytes and preedit payload it receives.
- Latency stays within the same event callback; no sleep, queue, retry, RPC or
  deadline is introduced. A combined commit can now cause its required final
  preedit update. Pure duplicate preedit still causes zero cursor feedback;
  stale-serial packets still update the display without feedback to GNOME.
- Memory: the existing cached string is freed once and nulled at commit. This
  creates no additional persistent cache, allocation class, owner or package.
  ASan/UBSan checks cover callback sequences; those checks do not establish a
  whole-terminal RSS or performance bound.
- Concurrency remains the single Wayland event callback; no worker or timer is
  added. Null focus, empty commit, identical preedit and stale serial are
  explicit controls. The independent direct-IBus transport is a separate
  compatibility surface and is not inferred from this Wayland proof.
- Installation is a separate gate: freeze the exact upstream tag and patch,
  build and compare shared-library exports/dependencies, preserve the currently
  mapped library and active Kitty/IBus processes, and first validate a separate
  process. Do not restart the user's working terminal to apply a test candidate.
- Rollback is the saved original shared library and original launcher path.
  Keep the version-specific patch removable when an upstream fix is verified;
  do not make this a permanent second Lay runtime or duplicate ranking route.

Open evidence before any installation: exact compositor packet grouping in the
reported live event was not captured; version matching is not a binary/source
provenance proof. The isolated callback proof establishes the consumer defect.
The remote worker lacks Kitty development headers; a private build sysroot and
shared-library compatibility check are prerequisites for a runnable candidate.
No installed bytes or running processes have changed.


### Consumer candidate result and independent review

The isolated patch changes only `text_input_done`; committed text is delivered
first and the existing display cache is invalidated before admitting the new
preedit. Candidate source SHA-256:
`61220ed8fb2fbba6de57d75b9f228186e5d62878d8d925d4debb176797030f1a`.
The removable, version-specific [patch](compat/kitty-0.48.2-preedit-after-commit.patch)
is preserved in the project; original upstream sources remain in the incident
folder. Guarded remote `candidate-v1/receipt.json`: 13/13 PASS, ASan/UBSan stderr
empty. The downstream keys-switch extraction and its source hash are identical
to baseline. Thus all four causal failures are repaired while nine original
controls retain their expected effects.

Fresh independent review, pass 1: 9/10, H0/M0/L0; no repair requested. Its verdict
covers the bounded patch and callback proof. The actual key callback prelude,
PTY timing and screen backend are outside the probe. Null-window handling is
not a focus enter/leave or reentrancy proof; the existing nonrecursive-callback
assumption is unchanged. Full shared-library ABI, actual pixels, focus changes,
direct-IBus and complete Wayland conformance remain unaccepted. No global IBus,
Lay process, working Kitty window, installed library or model has changed.

### Private shared-library build preparation

The full Kitty source is frozen at upstream tag 0.48.2, commit
`2cb1d95c3accadd536bd66ba6bda044973440177`; both callback source files match
the earlier downloaded proof inputs byte for byte. All compilation runs on
the remote host under the existing dedicated-20cpu resource guard. Build
dependencies are extracted/built into the incident's private sysroot; no
system packages are installed. The original installed 455,776-byte module is
preserved with SHA-256
`3aa0e71a2bbda452d963eb3e40b8c48ce1441721fd79a2a8778010a19cca940f`.

Four unsuccessful module-build setups are retained in `module-build/attempt-1`
through `attempt-4`: missing protocol pkg-config metadata, missing transitive
libffi metadata, an outdated 1.41 GitHub protocol mirror, and incompatible
Wayland 1.20 headers rejected by unchanged upstream code with `-Werror`.
No warning, assertion or runtime-source workaround was used for these failures.
Kitty's own `bypy/sources.json` pins the appropriate dependency archives:
Wayland 1.24.0 (`82892487a01ad67b334eca83b54317a7c86a03a89cfadacfef5211f11a5d0536`)
and wayland-protocols 1.45
(`4d2b2a9e3e099d017dc8107bf1c334d27bb87d9e4aff19a0c8d856d17cd41ef0`).
Those official release downloads match the pinned hashes. The intervening
official-head protocol checkout is also preserved; it is not the final build
input. All 23 required protocol files must be present before module compilation.

Private dependency-build failures are retained separately in
`module-build/matched-dependencies/prepare-attempt-1.{json,log}` through
`prepare-attempt-4.{json,log}`: embedded Kitty Python has an empty executable
path for child processes; Debian's multiarch ffi headers and linker symlink
require private search directories; a Meson reconfiguration reported `c_args`
without applying them to the generated compiler commands. The final build
uses host Python 3.10, a fresh Meson directory with verified `CFLAGS`, and
`LIBRARY_PATH` restricted to the private dependency directory. The dependency
receipt is `PASS_KITTY_PINNED_WAYLAND_1_24_PROTOCOLS_1_45`, scanner 1.24.0.
These are build-environment corrections, not candidate-patch or proof changes.

### Full module and isolated window result

`module-build/build-receipt.json`: `PASS_BUILT_SHARED_LIBRARY_PAIR`, 5.825 s
under the remote guard. The 501,680-byte candidate has SHA-256
`9cbbe9f79568ac522fdfd25a8aafa32313b7690918842abba3134322baeec61b`;
the same-build original module is
`96589192681b831130ce6c4bef6620f078e53b4988cebea06199e7bdb1fb2428`.
The two builds have identical 173 exports, 210 imports and five shared-library
dependencies, with no RPATH/RUNPATH. Exports and dependency names match the
installed module. The only installed/build import spelling difference is
the version suffix on 32 xkbcommon symbols; normalized names match exactly.
The only tracked Kitty source change remains the reviewed `wl_text_input.c`.
The final build regenerated all disposable protocol/compiler products after
changing build inputs; it did not reuse headers from the unsuccessful setup.

`module-build/loader-receipt.json`: installed original, rebuilt original and
candidate all load with `RTLD_NOW`, return the same GLFW version string, and
map the same dependencies with empty stderr. Wayland and xkbcommon come from
the copied installed Kitty bundle; dbus comes from the host, as it does for
the installed original. The first loader probe incorrectly required dbus to
be bundled; its failure is retained as `loader-attempt-1.json`. The corrected
probe compares actual mappings and results against the immutable installed
original, rather than assuming the packaging layout. These are loader/ABI
checks, not complete terminal behavior or pixel proofs.

A separate 111 MiB Kitty app copy was created without hard links to the
installed module. Only that copy's Wayland module was replaced. The native
Wayland window "Lay: проверка подсказок", PID 1891726, loaded the exact
candidate path and hash; its startup stderr is empty. Local receipt:
`~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/candidate-window.json`.
The input console reads text with readline and never executes it as a shell
command. The physical `про` / `прове` typing question is pending.
Existing Kitty PID 272166 still maps the original module; global IBus PID
4715 remains running. No installed Kitty/Lay bytes, models, input sources or
cancelled counters changed. Runtime change is limited to the new isolated
Kitty consumer window. Focus transitions, actual pixels and whole-terminal
acceptance still require native/physical observation; overall TD-123 remains
OPEN. Local copies of all build/loader receipts and failed attempts are under
`~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/module-build-evidence/`.

### User acceptance and publication, 2026-09-09

The user answered "да пушь" to the direct question whether `про` / `прове`
shows the continuation during typing without arrow keys in the isolated
"Lay: проверка подсказок" window. This accepts that physical typing scenario
and authorizes publication. It does not turn the callback probe into a full
Wayland conformance proof, establish the original compositor packet grouping,
or accept other applications, arbitrary focus transitions or all terminal
editing behavior. Overall TD-123 remains OPEN.

The separate acceptance record is
`~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/physical-acceptance.json`;
the launch and build receipts keep their historical pending status. Publication
contains the version-specific Kitty patch, this evidence and the refreshed
architecture graph on `origin/codex/cleanup-20260908`. Lay remains 1.0.68.
The accepted module runs in the isolated Kitty copy; the main Kitty installation
and existing processes have not been replaced or restarted. Publication itself
changes no runtime authority. Exact commit and verified remote reference:
`~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/publication/`.

### Normal Kitty installation after the repeated report

Later on 2026-09-09, the user reported that Tab fills `пров` to `проверка`
while the suggestion remains invisible. Native focus inspection identified
the existing working Kitty PID 272166, window 3077984638 / sequence 1131;
its mapped Wayland module was the original `3aa0e71a` version. The accepted
isolated Kitty PID 1891726 still mapped `9cbbe9f7`. Thus the latest observation
came from a client which had not received the accepted consumer patch; the
Git push did not replace its running module. No new model or timing diagnosis
was inferred from this report.

The accepted module was then installed atomically at the normal path
`~/.local/kitty.app/lib/kitty-extensions/kitty.glfw-wayland.so`, after verifying
the original file hash/inode, accepted candidate, unchanged launcher and
bundled Wayland/xkbcommon dependencies. A permanent original backup and
preflight/installation receipts are under
`~/.local/state/lay/compat-backups/kitty-0.48.2-preedit-2lmgtmmm/`.
The original inode 57960326 remains mapped in working PID 272166; its process
was not restarted. Global IBus PID 4715 is preserved. Rollback restores the
saved original module atomically for subsequent starts; it does not mutate
an already running process.

The normal Kitty launcher started native Wayland PID 3048061. Its new window
"Kitty: исправление установлено" maps installed inode 57933842 with the exact
accepted `9cbbe9f79568ac522fdfd25a8aafa32313b7690918842abba3134322baeec61b`
hash and empty startup stderr. The first observation preceded creation of
the control socket; the completed startup observation verified socket,
window identity, mapped path/inode and hash. This is installation/startup
verification of the physically accepted artifact, not a new physical typing
or full-terminal proof. Existing windows need their own process restart to
load the replacement. No Lay binaries, models, input sources or cancelled
counters changed. Authority change is confined to the Kitty consumer in
new processes using the normal installation.

Exact local receipt:
`~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/normal-installation.json`,
status `NORMAL_INSTALLATION_AND_NEW_PROCESS_VERIFIED`.

The user subsequently confirmed "в новом окне работает все отлично!" for the
window launched from the normal installation. Record:
`~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/normal-physical-acceptance.json`,
status `USER_ACCEPTED_NORMAL_KITTY_INSTALLATION`. This closes the reported
Kitty display scenario for the newly started client. Existing processes
still require restart; other applications and the full TD-123 contract are
outside this physical acceptance.

## WeChat report and first-word admission, 2026-09-09

**Current verdict: the user reports WeChat working and authorizes publication;
the separate native first-word refusal remains open. Runtime authority changed: false.**

The user reported "Теперь в окнах Wechat не работает ничего!" at 12:40 UTC.
The read-only capability check found compatible WeChat 4.1.1.4, PID 3734978,
`/opt/wechat/wechat`, using `QT_IM_MODULE=ibus`. Lay IME PID 536035 and the
global IBus PID 4715 remained alive; `/proc/536035/exe` still hashes to the
accepted 1.0.68 `2a2df132e44142be4b1f273122c7d3391a0ac7e1fbe7a5a1353dc65d91ac3219`.
The selected engine was `lay-ime-ru`, with both original Lay input sources.
The installed Kitty fix does not establish acceptance in other applications.

The existing opt-in rolling trace was frozen locally, without enabling a new
observer. It contains GUI input with capabilities 41 and surrounding text:
after a FocusIn, an empty snapshot reports text/cursor/anchor 0/0/0, but the
source-free activation and all three following printable settlements retain
`UnknownStart`. No candidate worker starts for that word. Other GUI intervals
contain a known word and visible-preedit output. These records have no source
application or per-row wall-clock timestamp, so neither interval is labelled
as a proved WeChat episode. Live focus checks initially saw Kitty, and a later
check saw Tor Browser. The clarification distinguishing ordinary input from
Lay features, and the requested focused WeChat repetition, remain open.

The first failing authority boundary is before candidate generation:
`source-free activation -> UnknownStart -> context_word_is_known() == false`.
`observe_external_surrounding_text()` stores a snapshot but does not change
word lineage; only an observed key boundary arms the next known word.
`context_allows_manual_toggle()` also refuses an unknown GUI word, while its
existing explicit terminal-suffix exception is separate. The latter is a
source finding, not a physical Double Shift measurement in WeChat. There is
no evidence here of a missing L1.1 candidate, L2/L3/L4 ranking loss, verifier
failure, or a change to any model/package.

### Exact installed-byte native diagnostic

The existing private IBus client harness was reused on the remote worker
under the dedicated-20cpu heavy lease and existing resource limits. The
candidate SHA and all nine dependency roles were checked by the harness.
The inherited driver is byte-identical after removing the added diagnostic
override. The GUI consumer advertises capabilities 41, publishes surrounding
text, and uses cursor geometry 2x19. The private old-FocusId cache is primed
through the same already-proved seed route; no `FocusInId` occurred. Each
prefix receives one fixed 150 ms observation window, without retrying it.

| Native client state | Expected hint | Observed hint | Result |
| --- | --- | --- | --- |
| Empty field, first `про` | visible | absent | reproduced refusal |
| Empty field, observed Space then `про` | visible | visible | positive control PASS |
| Cursor after existing `за`, then `про` | absent | absent | unknown-word control PASS |

All three literal surfaces were preserved and no deletion was emitted.
The process completed all 3 cases: 1 expected product failure and 2 controls
PASS, not release acceptance. Private service runtime was 3.254 s and CPU
time 2.888 s; all private processes were reaped. The GUI harness's initial
empty snapshot is suppressed by libibus's existing identical-value cache;
actual received snapshots of lengths 1, 2 and 3 still fail to arm the first
word. The separate live trace does contain the explicit empty snapshot.
These facts must not be merged into a claim that native setup delivered it.

Exact local evidence directory:
`~/.cache/lay/development/wechat-ime-no-functions-8isocthh/`.
`native-preflight.json` binds the helpers; `receipt.json` is native completion
SHA `463158289e73a502da9134c7d356cdbd5533392e4f115e769af810f3236cd2f2`;
`run-metadata.json` binds the candidate and dependencies;
`ibus-engine-trace.jsonl` is the synthetic private native trace.
`ibus_engine_debug.jsonl` is the initial private live snapshot, SHA
`8b0a256becab7358b3569659fc4dcbc9578c22659f4b1a9f57dd46e3f646cd52`;
its complete row 1686 is the empty snapshot. `existing-trace-v2-source.json`
records the later snapshot and its actual Tor focus observation.
The remote native run is
`/home/e/projects/lay-development-runner/autocorrect-ojoasco5/phrase-gui-empty-field-8isocthh-v1/`.
Raw personal input remained local; only synthetic diagnostic helpers were
uploaded. Neither cancelled input counter was restarted.

### Repair analysis before production changes

The initial idea was to promote a current exact surrounding-text prefix via
the existing admission reducer. Protocol review found an unresolved limit:
IBus cursor positions are relative to the supplied surrounding fragment, not
an absolute document position. Qt separately defines surrounding text,
absolute position and text-before-cursor queries. Its IBus adapter sends the
surrounding text and relative cursor/anchor values. Consequently, treating
every fragment offset zero as a proved document start is not justified by
the current transport contract. This is an inference from the documented
protocol distinction, not a captured truncated WeChat snapshot.
Sources: [IBus InputContext API](https://ibus.github.io/docs/ibus-1.5/IBusInputContext.html#ibus-input-context-set-surrounding-text),
[Qt input-method queries](https://doc.qt.io/qt-6/qt.html#InputMethodQuery-enum),
and the previously inspected Qt 5.15 `qibusplatforminputcontext.cpp` in the
private cross-app evidence directory.

Candidate designs and scores are engineering estimates, not test results:

- Promote from a generic relative snapshot alone: **3/10**, rejected for now.
  It would fix the positive fixture but cannot distinguish a real word start
  from the beginning of an insufficient fragment. Matching visible bytes
  does not itself prove completeness or bind a delayed snapshot to a focus.
- Admit an authenticated word-start witness through the existing reducer:
  **8/10 conditional**. A delimiter inside the supplied text is one usable
  witness; document start needs an explicit, current client-origin proof.
  Reuse the existing owner, activation, word lineage and bounded callback
  ordering. Do not add a parallel authority controller. The required client
  proof for the empty-field case is not yet implemented or verified.
- Permit only explicitly requested observed-suffix operations for GUI clients,
  with exact current surrounding-text validation: **7/10 conditional** for
  manual actions. This may extend the existing manual suffix contract without
  declaring the whole word known, but does not solve automatic first-word
  correction. Its display/Tab contract must be specified separately.

No production design has been selected and no runtime code has been edited.
The missing word-start evidence must be resolved before granting automatic
authority. Consequences to check in the selected design: preserve every
candidate source/ranking/verifier contract; reject middle-word fragments,
selection, stale snapshots, focus/owner ABA, reset and input gaps; preserve
atomic, GTK and terminal routes and the sole physical Shift detector; retain
the existing key/display deadlines and bounded worker queue; measure added
callback CPU/RSS/allocation cost; bind cache invalidation to the existing
frame identity; leave packages/reloads/learning/feedback unchanged; and define
the removal boundary for any added client witness. A refusal must preserve
literal input and cannot fall back to an unverified edit or another owner.

Required proof remains the causal native failure/control set, real callback
order and stale-evidence tests, the full affected and mandatory release gates,
and actual WeChat keyboard acceptance. General TD-123 quality, physical input,
ordinary typing health in the reported field, Tab acceptance, autocorrection
and physical Double Shift have not been newly accepted by this diagnostic.

### User confirmation and publication

The user subsequently replied "да заработало пушь" to the ordinary-input
clarification. Record this as user-reported recovery in WeChat and explicit
permission to push the current work. No runtime repair was installed during
this diagnosis; no particular recovery mechanism is established. The separate
three-case native result and its first-word failure remain unchanged. This
confirmation is not an individual measurement of Tab, autocorrection or
Double Shift. The update publishes diagnosis and delivery records only.

### First-word display repair preflight, 2026-09-09 13:33 UTC

After authorizing the preceding publication (verified commit `863dc4b5`),
the user explicitly reported the missing first-word IME suggestion. The
previous 3-case native control is the causal RED for this narrower defect.
No change has yet been installed. Automatic whole-word correction and
generic GUI manual replacement remain separate authority questions.

Engineering alternatives, assessed before editing runtime code:

| Design | Estimate | Consequences |
| --- | --- | --- |
| Separate observed-suffix display and explicit append-only completion | 9/10, selected | Reuse current admission token, exact frame, worker and edit verifier; do not claim the entire word is known. |
| Add a client-provided absolute word-start witness | 6/10 for this request | Could also authorize whole-word correction, but needs client/transport work beyond displaying and accepting a suffix. |
| Treat relative surrounding offset zero as KnownStart | 3/10, rejected | Still cannot establish the beginning of the whole document. |

Selected contract: a settled contiguous typed suffix can drive the existing
candidate readout independently of whole-word authority. GUI clients must
have a matching, unselected surrounding-text token at the caret, with no
contradictory letter before or after the observed token. Terminals can reuse
the already bounded observed suffix and terminal executor contract. Matching
a relative fragment is explicitly not a completeness promotion. A display
frame carries its original admission token so revocation, lineage, owner or
activation changes cannot revive a stale result with equal text. The current
150 ms deadline and single pending worker slot remain unchanged.

The native surrounding-text callback may schedule display after committed
input is reflected by the client; it grants no mutation permission. Explicit
Tab/Alt may reuse only the existing suffix-append plan (zero deletion and
cursor movement), rechecking the current suffix and client state. Replacement
candidates, Space correction, active composition, generic manual edits and
bridge authority retain their full-word gates. Unknown suffix acceptance
must not create whole-word learning feedback. Existing candidate generation,
ranking, source admission, SafetyGate, verifier, packages and physical Shift
detector remain unchanged; no literal fixture text becomes a runtime condition.

Proof gates: repeat the exact installed-byte RED/control set against the new
candidate; exercise native Tab and lifecycle/selection/middle-word negatives;
test stale worker identity and full-word mutation refusal; run affected and
mandatory release checks remotely under the existing resource lease; obtain
fresh-context independent review (at most two rounds), then install accepted
bytes and request physical first-word acceptance. Record display correctness,
edit safety, model quality and resource measurements separately. Native GUI
behavior is not yet WeChat physical acceptance. Current verdict: REPAIR_OPEN.

### First candidate measurements and retained failure

Private candidate `100fa0a7390123132566b60180bf5f29da607f15c8b6f4926a5962c0960d8e8a`
was built from six changed runtime files plus the regression tests. The
focused runner executed 454/454 correctness tests successfully (3 performance
tests excluded); its manifest reports exactly the 5 newly added tests.
Formatting, focused tests and release IME build took 73.448 s total. A helper
preflight initially rejected a missing `build.rs` registry key before any
build/test command; that failed receipt is retained separately and the input
check was corrected to hash every requested path directly.

An expanded fixed native set contains 7 cases: first word + Tab, observed
Space control, insertion after existing letters, insertion before an existing
word, selection after hint, caret movement after hint, and Reset after hint.
It uses the same exact client/inputs for baseline and candidate, with 80 ms
between key dispatches and one 150 ms display observation per phase.
Installed baseline passes 3/7; the new candidate passes 6/7. First-word Tab
now commits exactly the displayed suffix plus a space; neither run deletes
any text. The remaining failure is Reset leaving a visible stale preedit;
Tab after Reset correctly refuses and the literal surface is unchanged.
Runtime/CPU for these separate seven-case controls were 5.578/3.208 s for
baseline and 5.593/3.225 s for candidate. These are single controls, not a
performance bound or a general correction-quality measurement.

The Reset callback previously cleared only engine state. The next revision
also clears the legacy client preedit before local reset, preserves atomic
output ownership, and performs local reset even if signal delivery fails.
Independent review also identified duplicate identical surrounding snapshots
restarting a displayed suggestion. The next revision schedules only changed
snapshots while retaining observation/undo accounting, with an actual
callback/Tab regression test. These changes have not yet passed their gates.
Exact receipts: `~/.cache/lay/development/wechat-ime-no-functions-8isocthh/first-word-fix/`
(`candidate-build.json`, `focused-tests-SUMMARY.json`,
`native-baseline-v1/receipt.json`, `native-candidate-v1/receipt.json`).
Native helper SHA `78b2c6c4c5b062da608c9faf6b8f2614b216d7e16298bf37903d91ff62252a36`.
No runtime installation or general TD-123 promotion occurred.

### Reviewed candidate and release provenance correction

Candidate `5e9c7a557453fba8bea964ee745e0975e1ce8e0ec03336b87a3f824c525afacf`
passes 455/455 focused correctness tests (3 performance tests excluded) and
7/7 cases with the unchanged native driver. This repairs both the first-word
hint/Tab failure and native Reset failure. Independent source review is
9/10 H0/M0; its exact hashes and remaining evidence limits are recorded in
`tech_debt/evidence/ime-first-word-suffix-source-review.md`. Duplicate-wire
probes were inconclusive because duplicate delivery to the engine was not
established; they are retained, not counted as a product PASS. The actual
callback/Tab unit test passes. No runtime package or candidate source changed.

The first mandatory 1.0.69 full run failed after 485.351 s. Its TD-113 protected
artifact assertion correctly detected the new composition source against the
historical TD-121 hash. The empty known-failure ledger also rejected the
regenerated test manifest (6 new correctness tests; 2737 old rows unchanged).
No semantic failure is admitted to the ledger. Keep TD-113/TD-120/TD-121 evidence
immutable and add the explicit reviewed first-word successor binding. Rebind
only the empty ledger's expected manifest SHA; retain its historical
zero-failure observation as historical evidence, then rerun the canonical
full gates. The old observation is not proof for the six new tests.

Exact failed receipt:
`/home/e/projects/lay-development-runner/run-9E6Heo/first-word-full-acceptance-completed-identity.json`.
Revised-candidate receipts are `candidate-build-v3.json`,
`focused-tests-v3-SUMMARY.json` in the private first-word directory and
`/home/e/projects/lay-development-runner/autocorrect-ojoasco5/phrase-first-word-candidate-v2/receipt.json`.
These measurements establish private candidate behavior only. Final released
bytes, resource controls, installation and WeChat physical input remain OPEN;
installed runtime authority is unchanged at this checkpoint.

The next full attempt (`run-zJmtO5`, 134.722 s) passed architecture, manifest
rebinding and unchanged lint inventory, then stopped at rustfmt on the new
successor-contract assertion. Its requested formatting was applied without
semantic or runtime changes. Receipt:
`/home/e/projects/lay-development-runner/run-zJmtO5/first-word-full-acceptance-completed-identity.json`.
The subsequent fresh run performs formatting first to reject this class
before architecture and compilation work. Installed runtime is unchanged.

### Final 1.0.69 build accepted, private client gates pending

The fresh `run-inc1l8` completes every mandatory release gate successfully:
2717/2717 correctness/package tests (2681 + 36), architecture, formatting,
lints, desktop helper checks, CLI smoke, the full release build and the final
IME build. The manifest has exactly six new correctness tests and all 2737
prior rows unchanged. The known-failure list stays empty. Formal performance
and ignored lanes were not run; sampled client resource controls are separate.
No general model-quality promotion follows from these release gates.

Total full-run time: 896.145 s, including 324.550 s for test execution,
584.256 s for the canonical full command and 176.638 s for the final IME build.
These nested timings are not additive. Target storage passed the 12 GiB guard.
All ten release binaries and all 697 runtime Rust source hashes were copied
and verified locally. Final IME SHA:
`3c71eff1688fdbae06bd3f25e684fd5f8acfec4cc3a03ad77c7dd5741437277c`.

Exact full identity:
`/home/ubu/.cache/lay/development/run-btt4ilfz/first-word-full-acceptance-completed-identity.json`;
remote origin `/home/e/projects/lay-development-runner/run-inc1l8/`.
`full-test-lanes/SUMMARY.json` owns the test denominators. Source review remains
9/10 H0/M0; the second independent review of the installer and historical
successor binding closed at 10/10 H0/M0/L0. Its two provenance findings were
repaired: hash/mode guards for the 13 consumed scripts/extension inputs and
explicit baseline/candidate identities for the GUI resource pair. A subsequent
installer path change only selects the fresh run after the rustfmt failure;
functions and the installation transaction are unchanged.

Installed runtime is still 1.0.68 at this checkpoint. The final candidate's
private IBus profiles, lifecycle/Tab controls, first-word GUI/resource pair,
installation and physical WeChat acceptance remain separate pending gates.

### Final private-client acceptance and installation, 1.0.69

The final-byte IBus acceptance completed in 357.726 s. All three fixed89
profiles have exact metric parity with installed 1.0.68: 178 independently
recomputed before/after statuses per profile, zero class regressions and zero
new false outputs. The following values apply to both baseline and candidate;
they are the existing diagnostic fixtures, not a new heldout quality proof.

| Group | Strict | Normal | Experimental |
| --- | --- | --- | --- |
| `all_dirty` | 15/47 (31.91%) | 19/47 (40.43%) | 20/47 (42.55%) |
| `all_clean` | 41/42 (97.62%) | 41/42 (97.62%) | 40/42 (95.24%) |
| `clean_missing_letter_control` | 3/4 (75.00%) | 3/4 (75.00%) | 3/4 (75.00%) |
| `clean_repeated_letter_control` | 4/4 (100.00%) | 4/4 (100.00%) | 4/4 (100.00%) |
| `clean_valid_word` | 34/34 (100.00%) | 34/34 (100.00%) | 33/34 (97.06%) |
| `context_fixture` | 0/6 (0.00%) | 0/6 (0.00%) | 0/6 (0.00%) |
| `missing_letter` | 3/8 (37.50%) | 5/8 (62.50%) | 5/8 (62.50%) |
| `repeated_letter` | 0/4 (0.00%) | 0/4 (0.00%) | 0/4 (0.00%) |
| `restoration_regressions` | 7/24 (29.17%) | 9/24 (37.50%) | 10/24 (41.67%) |
| `transposition` | 5/5 (100.00%) | 5/5 (100.00%) | 5/5 (100.00%) |

Existing wrong outputs remain: dirty 0/3/3 and clean 1/1/2 for strict, normal
and experimental respectively. Exact parity is not a general quality PASS;
TD-123 remains OPEN. The L1/L2 heldout evidence and eight immutable model inputs
were reused only under their unchanged source/dependency bindings.

Final native controls pass 13/13 (US and RU first-word controls, manual edits,
lifecycle and restoration). One quiet legacy control retains its early hint
and 5/5 correction controls. The narrow terminal retains its early hint and
3/5 controls, matching the prior isolated-harness limit without a physical
keyboard daemon; this is not a claim that all terminal behavior passes.

The separate fixed first-word GUI pair uses the same seven-case driver and
newly accepted L1.1 service for both IME versions: old 1.0.68 is 3/7 with the
first hint absent, new 1.0.69 is 7/7 with the hint and exact Tab append present.
All seven candidate cases preserve the no-deletion contract. All private
processes were reaped. Native duplicate-callback delivery remains inconclusive
as documented above; its actual callback/Tab unit contract passes.

Observed resource pair (one control each, 20 ms sampling):

| Measure | 1.0.68 | 1.0.69 |
| --- | --- | --- |
| Observation duration | 6.024 s | 6.003 s |
| Process VmHWM | 407356 KiB | 408300 KiB |
| Private cgroup memory peak | 407539712 bytes | 405880832 bytes |
| Observed cgroup CPU usage | 3.168 s | 3.141 s |
| Observed swap | 0 | 0 |

These are sampled observations, not statistical latency/CPU bounds; allocation
counts and unobserved exit intervals were not measured. No resource-limit,
worker-queue, deadline or candidate-source relaxation was needed.

Exact local receipts are under `/home/ubu/.cache/lay/development/run-btt4ilfz/`:
`first-word-native-acceptance/receipt.json`, its two `resource-*/first-word-resources.json`
files, `td123-release-native-controls.json`, and the five `td123-release-native-*/receipt.json`
files. Fixed89 comparisons and four final control directories are under
`/home/ubu/.cache/lay/development/autocorrect-live-ojoasco5/` with the
`first-word-fixed-profile-*` and `phrase-first-word-release-*` prefixes.

Installation status: **INSTALLED_VERIFIED_PHYSICAL_PENDING**. All ten accepted
release binaries are installed; the CLI and loaded extension report 1.0.69.
The four loaded executable hashes match the release manifest: IME PID 3856819,
daemon 3856813, L3 3856814, and L1.1 3856565.
Global IBus PID 4715, user configuration, input sources and all eight immutable
model inputs were preserved. WeChat and the older Kitty windows were not restarted.
Both cancelled input counters remain off. Runtime installation authority
changed at this step; whole-word completeness/mutation gates were not broadened.

Installation receipt: `/home/ubu/.cache/lay/development/run-btt4ilfz/installation-1.0.69.json`.
Backup: `/home/ubu/.local/state/lay/release-backups/1.0.69-td123-mrlex9zk`.
The user has been asked to type the first word in an empty WeChat field and
check the visible hint and Tab. Physical acceptance and publication remain
pending; do not rebuild or reinstall merely to complete those steps.

### Physical acceptance reopened before publication

The first reply to the requested scenario was "проверка да круто!". Before
commit/push the user then reported: "короче когда удалишь всегда IME есть а
вот когда набираешь НЕ ВСЕГДА ЕСТЬ!", "опять пропала!", "Илине простраслась!",
and "Нет пров(опять нет)". The next clarification was "Backspace наоборот
активирует IME": deletion activates the visible hint, rather than hiding it.
The initial confirmation is retained as history;
current status is **INSTALLED_VERIFIED_ACCEPTANCE_REOPENED**. No commit or push
of 1.0.69 occurred. The exact follow-up is in
`/home/ubu/.cache/lay/development/run-btt4ilfz/physical-acceptance-1.0.69.json`.

A bounded snapshot of the existing opt-in IME trace captures the reported
prefix: `пров` produced 12 candidates in 2.965 ms, was applied at 3.004 ms,
and sent a visible preedit update with four suffix characters. A later `(`
cleared the hint, which is a separate punctuation event. Current GNOME focus
is Kitty PID272166. It still maps the original deleted module inode57960326,
whereas the installed corrected module is inode57933842, SHA `9cbbe9f7`.
The previously accepted new Kitty PID3048061 is no longer running. This
supports investigating the already known old-client rendering path; current
focus alone does not attribute every reported failure to that application.
The user has been asked whether the intermittent failure is in Codex/Kitty or
WeChat. This observation alone does not justify another runtime change.

Evidence: `/home/ubu/.cache/lay/development/run-btt4ilfz/intermittent-hint-live/`
(trace snapshot, focused-window identity and module mapping receipt). Both
cancelled input counters remain off. Installed 1.0.69 and global IBus are
unchanged. Existing old terminal sessions are preserved. Further code work
requires a fresh causal distinction between producer, transport and consumer;
publication and physical acceptance remain open.

### User accepted hint delivery and requested publication

After the retained reopened report, the user wrote:
"Push отлично но следущая проблема это то что автопераврот не работает !".
This is explicit publication authorization and acceptance of the delivered
hint scope. The client of the final confirmation was not specified; do not
promote it to complete WeChat or desktop acceptance. The earlier trace and
old Kitty module evidence remain historical observations with that limit.
The next reported case is "зуын -> push не сработал !"; automatic layout
correction is now the active diagnosis and general TD-123 remains OPEN.

The installed 1.0.69 runtime and all 697 Rust source hashes still match the
completed full-gate identity. This publication step changes no runtime, model,
configuration, input-source or mutation authority. No old terminal, WeChat or
global IBus process is restarted; cancelled input counters remain off.
The original confirmation and reopening are preserved before the new scoped
acceptance in `/home/ubu/.cache/lay/development/run-btt4ilfz/physical-acceptance-1.0.69.json`.
The exact commit and verified remote ref belong in the separate publication
receipt directory `/home/ubu/.cache/lay/development/run-btt4ilfz/publication-1.0.69/`.

### Automatic layout correction after deleting a boundary, 2026-09-09

Publication of the accepted 1.0.69 hint scope completed as commit
`73724fa497a8d8e4b713277d4bf20d7e165bf3c2`; `origin/codex/cleanup-20260908`
was read back at the same SHA. New work starts from that clean source.

The bounded existing live trace contains two distinct mechanisms. The literal
reported `зуын` maps to `pesy`, not `push`; its prepared correction reaches a
Rank no-apply outcome (ready at Space, no timeout). This is a compound typing
error and is not evidence for a failed exact layout projection. Separately,
`ЗГыр` after an observed Space produces the automatic `Push ` replacement.
After later Backspace crosses a retained Space, `WordCompleteness` changes
from KnownStart to UnknownStart. Deleting back to a retained earlier separator
and typing the exact-layout token `згыр` never restores completeness. Its Space
has no captured correction frame or prefetch request; explicit manual toggle
then produces `push ` through the terminal executor.

The first authority loss in the latter case is
`advance_context_word_scope` on boundary deletion, before L1.1/L2/L3/L4,
DecisionCore and verifier are consulted. The next refusal is
`capture_input_frame_identity`; `managed.rs` then commits the literal Space.
All three live auto flags remain enabled; the installed executable hashes and
configuration match the accepted 1.0.69 installation. This does not establish
all-app behavior or a model-quality change.

Evidence: `/home/ubu/.cache/lay/development/auto-layout-push-k9vjrshc/`
`mechanism-evidence.json` and its hash-bound existing trace snapshot. The trace
has ordered events but no per-event wall-clock timestamps or client rendering
acknowledgement. No new observer was started; both cancelled counters stay off.

The next bounded experiment uses the installed candidate in private IBus with
real GNU Readline, four fixed cases: unknown first-word preservation, ordinary
known-boundary layout correction, correction after deleting back across a
boundary to an earlier retained separator, and recovery after a newly typed
boundary. Only the third product case is expected to fail on baseline. The
same driver will be reused for a candidate; the deliberate negative control
must still preserve unknown text. Actual callback order, visible effects and
admission evidence are recorded; this is neither physical keyboard acceptance
nor heldout model quality. Runtime authority is unchanged. A production change
still requires a provenance design that distinguishes retained observed
boundaries from unobserved mirror text; do not promote arbitrary tail contents
or weaken verifier/SafetyGate to make this example pass.

The first native attempt stopped after two passing controls, before the
boundary-deletion case: a copied generic handoff assertion required a nonempty
native FocusInId receipt, while this controlled legacy client intentionally
omits it. Its `native-baseline-v1/receipt.json` remains FAILED; all private
processes were reaped. Driver v2 explicitly asserts the empty native receipt
and exact visible tail while preserving current-context, engine-owner and
observer-marker checks. No product assertion, cadence, deadline or retry was
changed. Derivation: `native-helper-identity-v2.json` in the same evidence dir.

The v2 attempt also remains FAILED before boundary deletion: automatic handoff
supplied a native FocusInId while explicit profile selection could omit it.
The globally-empty v2 assertion was therefore also an incorrect fixture
assumption. This is an explicit harness-contract replan, not a product retry:
v3 permits absence and otherwise requires the exact current InputContext path
plus client name. Independent current-context/profile/owner/marker checks and
all four product expectations remain unchanged. Receipts for v1 (two completed
controls) and v2 (one completed control) are retained; all private processes
were reaped. See `native-helper-identity-v3.json`.

### Consequence analysis before retained-boundary implementation

Measured baseline: driver v3 completes four actual IBus/Readline cases in one
fresh 1.0.69 process. Three product cases pass; only retyping after boundary
deletion remains literal. Both ordinary correction and correction after a new
Space pass, while the unknown first-word negative control stays unchanged.
All private processes are reaped. Receipt:
`/home/ubu/.cache/lay/development/auto-layout-push-k9vjrshc/native-baseline-v3/receipt.json`.
This establishes the admission defect, not a quality defect in the exact
`згыр` projection. The distinct compound `зуын` Rank refusal remains separate.

Design comparison (engineering estimates, not measured performance scores):

| Route | Score | Consequences and verdict |
| --- | --- | --- |
| Current single completeness flag, discard provenance on boundary deletion | 4/10 | Preserves unknown-text refusal but loses an earlier observed separator; measured 3/4 native behavior. |
| A retained observed-boundary floor inside existing WordLineage | 9/10 | One fixed-size optional scalar offset, carried by the existing admission/transfer token. Exact Backspace may reuse only a retained separator at or after that floor. No new owner, queue, timer, cache or model route. Selected. |
| A bounded stack of every observed word boundary in the same admission owner | 7/10 | Viable, but more state and update/trim/transfer rules than the single earliest observed floor needed for this defect. Rejected maintenance cost. |
| Generalize the manual observed-suffix count into a whole-tail provenance span | 7/10 | Viable with complete rebasing across every variable-length bridge output; changes manual suffix semantics and more executor contracts. Rejected for this repair. |

An arbitrary separator found in an unproven mirror is not an alternative:
without an observed floor it cannot establish word start. The new field is
proof metadata within the existing WordLineage; all source-free activation,
input-gap, Reset, changed-context and failed-transfer paths must clear it.
Exact same-context transfer carries it with the existing lineage and tail seal.
It does not carry separate authority, introduce a second generation or replace
ContextAdmissionReducer. Scalar offsets count Unicode characters, not bytes.

The selected floor is the earliest retained position known to be at or after
an observed separator. Owned suffix replacement preserves that prefix or
writes known new text to its right. Prefix trimming can only move retained
text left: keeping the old floor is conservative and may discard otherwise
recoverable older-boundary evidence. It must never move the floor left by
inferring a prefix deletion from coincidentally equal strings. A newly
observed boundary supplies a fresh floor. This bounded repair makes no claim
of exhaustive restoration across a truncated 160-character mirror.

On an exact one-character terminal/managed Backspace crossing a separator,
inspect the retained tail once. An earlier separator inside the observed
range keeps KnownStart under a new lineage generation; otherwise revoke as
before. Empty mirrors, non-exact edits, command modifiers, navigation and
unknown prefixes never gain authority. Ordinary within-word Backspace remains
unchanged. No worker is scheduled from the Backspace callback and no preedit
is republished there, preserving the WeChat deletion contract. Subsequent
typed input uses the existing preparation worker. Immediate Space after a
Backspace without retyping is outside this demonstrated repair.

Candidate/lattice retention, ranking, L1.1/L2/L3/L4, DecisionCore, verifier and
SafetyGate are unchanged. Newly re-admitted words use the same bounded
candidate field and final edit authority as ordinary known-boundary words;
no token, phrase, source ID or test name becomes a runtime condition. Existing
word/frame/config/material identities and stale-result rejection remain.
A boundary crossing must invalidate earlier word tokens even when the previous
word's start is retained. Full/cold model outputs and package reload policy
are unchanged; no new package or delta is installed by this implementation.

CPU/RSS/allocation estimate: one optional u32 in existing copied lineage;
no new task, queue or RPC. Exact Backspace compares the retained prefix even
for KnownStart; crossing a boundary scans for the last retained separator.
Each observed new boundary also scans the tail to calculate its scalar offset.
These are O(n) operations over the retained mirror. Ordinary append limits it
to 160 scalars; this is not a universal hard cap immediately after every bridge
replacement. Diagnostic formatting allocates only when trace is enabled;
the new provenance operations themselves introduce no heap allocation.
Newly admitted typing performs existing bounded prefetch work previously
skipped. Native timing and static layout size are recorded separately; a single
four-case run establishes no statistical latency bound. The Space deadline
and worker limits are unchanged.

Learning remains gated by the existing verified output/postcondition route;
manual projection, suffix display and refused operations do not become new
positive feedback. Explicit append/variable-length suffix edits do not move
the floor left. Sensitive input and all genuine input gaps discard provenance.
Concurrent handoff remains owned by one engine guard and the existing source
seal; bridge settlement still cannot rewrite word provenance. Failure leaves
the prior literal/refusal behavior. Rollback is the already installed 1.0.69
binary set and the isolated source commit `73724fa`.

Required proof before promotion: actual legacy callback tests for observed
and unobserved retained separators, only-boundary deletion, input-gap/reset
revocation and stale-token invalidation; unchanged first-word hint/Tab and
physical Double Shift ownership contracts; the same native v3 four-case
baseline/candidate pair (3/4 to 4/4 with the negative control preserved);
all three fixed89 profiles with aggregate/per-class/false-output parity and
no regression; canonical full release gates; independent review with at most
two repair/review rounds; exact-binary installation followed by user physical
confirmation. Broad TD-123 quality and the compound typo report remain OPEN.
Implementation has not started at this preflight point; runtime authority is
unchanged. The ordinary starting-point release has already been pushed.


#### Retained-boundary review round 1 and test repair

Independent review: 8/10, H0/M1/L1. No production blocker was found. M1 was
the new negative fixture seeding its unobserved mirror before source-free
activation; the first callback correctly cleared it. The repaired fixture
settles activation through existing Shift callbacks, checks the live token
and UnknownStart, then seeds the mirror. It asserts the exact pre-Backspace
tail and observed floor as well as the original refusal/no-learning effects.
L1 corrected the CPU/allocation scope above.

The first focused remote run failed: 458/459 in the main test process, one
separate process passed, and the positive usage test failed on an empty file.
That test stopped at the first successful read. The unchanged persistence
writer creates the file, sets permissions, then writes a newline-terminated
event; the reader could observe the interval before writing. Its wait now
requires a complete line, retaining the original 1300 ms deadline and all
positive/negative semantic assertions. No runtime persistence change.
Receipt: `/home/ubu/.cache/lay/development/run-_iedwha7/RESULT.json`;
remote test logs: `/home/e/projects/lay-development-runner/run-Nlo1dE/tests/logs/`.
Review: `/home/ubu/.cache/lay/development/auto-layout-push-k9vjrshc/review-v1/review.md`.
Candidate native proof, repaired focused results and final review remain pending
at this record. Runtime authority and the installed 1.0.69 are unchanged.


Repaired focused check: 461/461 PASS (459 main, two isolated processes),
14.338 s total including format. Six new tests preserve all previous tests.
Receipt: `/home/ubu/.cache/lay/development/run-v250cnbd/RESULT.json`;
remote logs: `/home/e/projects/lay-development-runner/run-nkfjJj/tests/logs/`.
Versioned candidate is 1.0.70; full release, native candidate and final review
are pending. Installed runtime remains 1.0.69; runtime authority unchanged.


The initial 1.0.70 full gate stopped during test discovery at the unchanged
12 GiB Cargo target limit (64.531 s; Cargo exit 75). No test failure or
candidate release followed from that attempt. Saved under
`/home/ubu/.cache/lay/development/auto-layout-push-k9vjrshc/full-failed-budget-v1/`.
Under the existing resource lease, only disposable Lay-owned debug artifacts
and fingerprints were removed: target 12,946,116,608 -> 4,361,183,232 bytes.
Dependency/release caches, saved proof binaries and installed runtime were
preserved; the previous candidate SHA was checked after cleanup. Receipt:
`/home/ubu/.cache/lay/development/auto-layout-push-k9vjrshc/cache-cleanup.json`.
A fresh frozen full gate is required; no failed attempt is retried in place.


#### Inline callback outcome budget, before lint repair

Full attempt 2 reached default Clippy after format, architecture and manifest
checks, then stopped (107.525 s). A separate scoped diagnostic preserves the
actual error: the generic RendezvousOutcome Stamp variant is at least 208 bytes
versus one byte for Failed. The added lineage metadata crosses Clippy's
default size-difference heuristic. This is a measured compiler diagnostic,
not a measured CPU/latency regression or a runtime correctness failure.
Diagnostic: `/home/e/projects/lay-development-runner/run-3NAM4Y/clippy-bin-diagnostic.jsonl`.

Consequence comparison before changing source (engineering scores):

| Route | Score | Consequences |
| --- | --- | --- |
| Box the stamp | 3/10 | Adds allocation/indirection to callback results and changes the established ownership path solely to reduce enum stack size. |
| Compact floor into an optional nonzero u16 | 7/10 | Could reuse lineage padding, but needs checked encoding and a new conservative refusal beyond 65534 scalars. More representation/range semantics for this repair. |
| Keep the inline stamp with a local expected lint and an explicit size ceiling | 9/10 | Preserves ownership, callback API and allocation behavior; selected. Existing semantic test will enforce WordLineage <=24 and production RendezvousOutcome<Sequence> <=224 bytes and report actual sizes. |

The selected annotation is local to this enum. Global warning policy and all
runtime/authority gates stay in force; the compiler will report an unfulfilled
expectation if the lint ceases to apply. The 224-byte inline ceiling is a stated
engineering budget; actual layout and the existing 128-stamp capacity remain
separate from RSS/latency observations. No new field, owner, queue, task or heap
allocation is introduced by this lint repair. Independent final round 2 will
review this delta and refreshed source/installer bindings. No candidate release
or installed runtime changed at this point.

### Retained-boundary final validation and installation, 2026-09-10

Status: `INSTALLED_VERIFIED_PHYSICAL_PENDING`. The prior preflight and failed
attempts above are historical. The scoped repair preserves an observed earlier
boundary when Backspace removes a later one and the retained prefix is exact.
It retires the old token, creates the new lineage generation, and permits
the existing automatic-correction path after retyping. No new candidate source,
literal exception, SafetyGate/verifier relaxation or model authority is added.
Unknown first-word and unobserved mirror boundaries retain their refusal.

Final source: seven reviewed IME/test Rust files differ from accepted 1.0.69;
all 697 Rust source identities are bound to the successful full run. The final
focused gate passes 461/461 in 14.144 s. Full acceptance passes in 894.496 s:
format, architecture, test manifest, inline layout, lint and release gates.
Mandatory tests: 2723/2723 (2687 correctness + 36 package); zero semantic or
infrastructure failures. All 2743 prior manifest rows retain their scope;
six tests were added. The 15 ignored and 11 optional performance cases remain
outside this mandatory denominator. The known-failure ledger still admits
zero failures; only its manifest binding changed. Lint baselines are byte-identical.

Compiled layout: WordLineage 24 bytes; production RendezvousOutcome<Sequence>
216 bytes, within the explicit 224-byte ceiling. This validates object layout,
not allocation counts or latency. The local expected Clippy lint retains the
inline ownership representation. Final independent round 2 review:9/10,
H0/M0/L0; seven Rust hashes and the exact installer hash are bound. Review is
static evidence, separate from the executed full/native gates. The failed
Clippy run and diagnostic remain in `full-failed-clippy-v2/` under the case root.

The private final-binary native acceptance passes in 367.929 s. Its frozen
four-case boundary driver is identical for baseline and candidate: 3/4 -> 4/4.
The only changed case is retyping after boundary deletion; unknown-first-word
preservation, ordinary known-boundary conversion and recovery at a new boundary
pass in both. The same new L1.1 service binary serves both versions and all eight
model input identities are unchanged. The previous first-word GUI control stays
7/7 -> 7/7; native 13 passes (US1 + RU1 + manual3 + lifecycle3 + restoration5).
Quiet legacy hint/correction controls pass 5/5. Narrow terminal controls remain
3/5 without the physical daemon; this is an explicit prior limitation.

Fixed89 proof was rerun in strict, normal and experimental profiles, each with
47 damaged and 42 clean cases. Each comparison independently recomputes 178
statuses across its baseline/candidate pair. Input identities and fixture
manifest are bound. Outputs, false outputs and every group metric are identical;
there are zero new false outputs and zero per-class regressions. Counts and
percentages below apply to both 1.0.69 and 1.0.70:

| Fixed89 class | Strict | Normal | Experimental |
| --- | --- | --- | --- |
| All damaged | 15/47 (31.91%) | 19/47 (40.43%) | 20/47 (42.55%) |
| All clean | 41/42 (97.62%) | 41/42 (97.62%) | 40/42 (95.24%) |
| Restoration regressions | 7/24 (29.17%) | 9/24 (37.50%) | 10/24 (41.67%) |
| Missing letter | 3/8 (37.50%) | 5/8 (62.50%) | 5/8 (62.50%) |
| Repeated letter | 0/4 (0.00%) | 0/4 (0.00%) | 0/4 (0.00%) |
| Transposition | 5/5 (100.00%) | 5/5 (100.00%) | 5/5 (100.00%) |
| Context fixture | 0/6 (0.00%) | 0/6 (0.00%) | 0/6 (0.00%) |
| Clean missing-letter control | 3/4 (75.00%) | 3/4 (75.00%) | 3/4 (75.00%) |
| Clean repeated-letter control | 4/4 (100.00%) | 4/4 (100.00%) | 4/4 (100.00%) |
| Clean valid word | 34/34 (100.00%) | 34/34 (100.00%) | 33/34 (97.06%) |

Wrong outputs on damaged text remain 0/47 (0.00%),3/47 (6.38%),3/47 (6.38%)
for strict/normal/experimental; wrong outputs on clean text remain 1/42 (2.38%),
1/42 (2.38%),2/42 (4.76%). Non-restoration remains 32/47 (68.09%),25/47
(53.19%),24/47 (51.06%). These are the existing fixed fixtures with ordered
post-ready input and learning within each shard, not a new heldout quality proof
or ordinary-keyboard timing gate. General quality is not promoted by parity.

Resources were sampled every 20 ms during one unchanged seven-case GUI control
per version (206 process/cgroup samples each). These are kernel high-water
observations from private mini-PC clients, not a controlled performance gate:

| Observation | Baseline 1.0.69 | Candidate 1.0.70 |
| --- | --- | --- |
| Process VmHWM, KiB | 399752 | 412428 |
| Process RssAnon maximum, KiB | 173104 | 185900 |
| Process RssFile maximum, KiB | 226648 | 226528 |
| Maximum threads | 29 | 29 |
| Private cgroup memory.peak, bytes | 397545472 | 413048832 |
| Private cgroup CPU usage, microseconds | 3143702 | 3132878 |
| Swap current maximum, bytes | 0 | 0 |
| Observed OOM / OOM kill | 0 / 0 | 0 / 0 |

Observed candidate HWM is 12676 KiB (3.17%) higher and cgroup memory.peak is
15503360 bytes (3.90%) higher in this single pair. That difference is recorded
without attributing causality or claiming RSS parity. Final process-exit
intervals may be unobserved. No allocation/cadence/latency guarantee follows.
The prior model proof is reused by unchanged non-IME Rust and eight model
dependencies, not rerun: optional unique-prefix 301.738 ms exceeds 50 ms and
conditional L2 formal 5.276 ms exceeds 5 ms. Their gates remain FAIL/OPEN.
L1 per-class unique top-1 >95%, clean preservation, lattice coverage, false
certainty, package/RSS budgets and latency remain a conjunctive contract; this
admission repair does not independently satisfy or waive any model-quality gate.

Ten verified release binaries were installed at 2026-09-09 21:41 UTC
(2026-09-10 local date). CLI and loaded extension report 1.0.70. Installed
file hashes and four loaded process hashes match the release manifest:

| Owner | PID | Executable SHA256 prefix |
| --- | --- | --- |
| L1.1 service | 1719009 | 617d72b39710 |
| Daemon | 1719235 | a9836cee7bab |
| L3 online | 1719236 | de7b3f955852 |
| IBus engine | 1719241 | f4d3c8e256ab |

Runtime authority changed only at this verified installation, after the source,
full/native and review gates passed. Global IBus PID 4715/start 2261 survived;
configuration, input sources and eight model dependencies were preserved.
No Kitty/WeChat client restart or keyboard/clipboard automation occurred. The
rollback binary/config backup is
`/home/ubu/.local/state/lay/release-backups/1.0.70-td123-ufl_l8wy`.
The physical keyboard check requested at 21:42 UTC remains PENDING; publication
is pending. The final candidate SHA256 is
`f4d3c8e256ab4f9164007482d828a7e171a9e42770846e9bdd210724d9fd270a`.

Not tested/promoted: physical keyboard acceptance in the reporting application,
global desktop behavior, immediate Space without retyping after Backspace,
or general restoration/latency improvement. The literal layout of «зуын» is
«pesy», so «зуын» -> «push» requires two additional character corrections;
its prepared Rank refusal is a separate OPEN mechanism. TD-123 remains OPEN.
Both previously stopped input counters remain stopped.

Exact evidence roots (private artifacts stay outside the repository):

- Case: `/home/ubu/.cache/lay/development/auto-layout-push-k9vjrshc/`;
  `candidate-source-1.0.70.json`, `review-v2/{identity.json,review.md,verdict.json}`,
  `release-proof-v1/native-{baseline,candidate}-v3/receipt.json`,
  `release-proof-v1/first-word-fixed-profile-{strict,normal,experimental}-comparison-v1.json`.
- Successful run: `/home/ubu/.cache/lay/development/run-p4d_z81t/`;
  `RESULT.json`, `retained-boundary-full-acceptance-completed-identity.json`,
  `full-test-lanes/SUMMARY.json`, `retained-boundary-native-acceptance/receipt.json`,
  `td123-release-native-controls.json`,
  `retained-boundary-native-acceptance/resource-{baseline,candidate}/first-word-resources.json`,
  `full-artifact-fetch.json`, `native-artifact-fetch.json`, `installation-1.0.70.json`.
- Remote full/native run: `/home/e/projects/lay-development-runner/run-txvDIy/`;
  prior model proof: `/home/ubu/.cache/lay/development/run-o0dqrl5c/td123-full-model-proof-comparison-v1.json`.

### GitHub issues 42–44 supplement to 1.0.70, 2026-09-10

The revised daemon retains leading ordinary symbols in its existing buffer;
removing the token-wide ignore latch repairs exact replay suffix/erase identity.
The first loss was daemon admission, before manual projection or the verifier.
No model, ranking, SafetyGate, gesture detector or output transport changes.
The 32-case baseline fails and candidate passes; shortcut and sixty clean-token
configuration/context controls pass with the documented Nanda-disabled unit
runtime. Four explicit physical-owner contracts pass in both complete gates.

Fresh changed/full each pass 2726 cases (2690 correctness, 36 package), with
zero failures; fifteen ignored and eleven performance cases remain excluded.
The full frozen run takes 891.255 s. Independent round-2 review is 9/10,
H/M/L 0/0/0. Public installer progress, ten-target selection, failure/cancellation
and jq prerequisites pass eight tests; ARM cross-build and emulated startup/
version/layout/XML proof also pass with native Ubuntu outcomes unobserved.

All 698 Rust hashes are bound to the accepted build. Only the installed daemon
binary changes to `133a1f79e57b34293c496921be40d96d3bc56c5e03ea630d1d8f8939404920d5`;
the other nine hashes, including IME f4d3c8e2, match the first 1.0.70. Thus prior
fixed89, native13 IBus/Readline, retained-boundary and GUI receipts remain
applicable to the identical IME/model bytes. They do not prove a new daemon
physical gesture or production Nanda preservation for newly admitted prefixes.
No new latency/RSS or general quality improvement is claimed. The earlier
conjunctive quality/resource/performance limits remain as recorded above.

Runtime authority changes at verified installation 2026-09-10 01:07:29 UTC;
four loaded hashes, configuration, eight model files and input sources match.
Global IBus PID4715 is preserved. Physical keyboard confirmation is PENDING.
Installation: `~/.cache/lay/development/run-j4e7w_0n/installation-1.0.70.json`;
full identity and both lane summaries are beside it. Rollback:
`~/.local/state/lay/release-backups/1.0.70-td123-aeuwv34d/`.
Full consequences, failed private ARM-toolchain probes, exact proof scope and
publication receipts: [owning issue document](public-issues-42-44-release-1.0.70.md).





### Exact manual handoff V2 installed, physical pending, 2026-09-12

Final measured delivery state for the connected repair: installation completed
with status `INSTALLED_LOADED_HASH_VERIFIED_PHYSICAL_PENDING`. Runtime authority
changed: true, IME binary only. Physical acceptance remains `PENDING`; the ping
after restart returned `('lay-ibus-engine-rs no-focus',)`, so this record makes
no browser-focus, text mutation, answer-quality, heldout, RSS, or latency claim.
Tests/CI denominator is 0. Root static source review accepted the source shape;
that is source review only, not correctness or quality proof.

Build receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/receipt-fix-build-v2/build-result.json`.
The build receipt status is `PASS_RUNTIME_BUILD_ONLY_GRAPH_UPDATED`, elapsed
78.86 s, with root-reported transport 79.29 s / worker 78.86 s. Source snapshot:
1382 files, archive SHA
`ce1e223ae79cff15064c349f6a07257bfba1fd36e5bb71d9f022aa2838b8a7c0`. Built
binary hashes: daemon
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e`; IME
`994485bf9d7379c8d820171960c61e5980e59f88341ab51d6a8b7741c08eec80`. Fetch
verification:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/receipt-fix-build-v2/fetch-verification.json`
with status `FETCH_HASH_VERIFIED`, source files verified 1382, build receipt SHA
`cf52594b41cdc2e182fc29e2bddc01d699ac4bc35e4947f087157a78000d514f`.

Installation receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/installation.json`
with SHA
`abc0b753a6f676fc7b6ce91c0f09405f8749244f9c871b5beae867f9a5943a22`. Runtime identity receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/runtime-after.json`
with status `INSTALLED_PROCESS_IDENTITY_VERIFIED`; all 11 recorded identity and
configuration checks are true, covering loaded process hashes/PIDs, DBus owner,
config, sources, switch/backward bindings, XKB options, selected engine and
temporary debug removal. Only `lay-ibus-engine` changed. Old IME PID `2128417` SHA
`86f5ea13549ffeb473bc70959b934d734406c9ed336fb5c3a06b71415ed6b96b`; new IME
PID `4051893` SHA
`994485bf9d7379c8d820171960c61e5980e59f88341ab51d6a8b7741c08eec80`. Daemon
PID `3880511` SHA
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e` and global
`ibus-daemon` PID `4715` were unchanged during install. Selected engine
`lay-ime-us`, input sources `[('ibus', 'lay-ime-us'), ('ibus', 'lay-ime-ru')]`,
`switch-input-source` `['<Shift><Alt>space']`, backward binding `@as []`, XKB
options `['grp_led:scroll']`, and config SHA
`5887b077e716357cd0a622d16feda7147ff50c1f5ad2bee136f970fa095a9479` were
preserved. Rollback backup root:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/backup`.
The temporary diagnostic override was already removed before this final install.

Installed source scope: valid same-field context-admission `Transfer` preserves
live exact manual handoff markers while retaining the original source path for
daemon cleanup; `SourceFree`, `ResetUnknown`, revocation and failure clears remain
clearing. `VisibleTailV3` and `SuppressNextAutocorrectV2` resolve the engine from
the same fenced token, consume ready target activation under that target engine
guard, and then require live-token plus explicit shared-active-path match.
First-word GUI manual Double Shift is installed only for the bounded
`UnknownStart` suffix witness with current unselected surrounding-text evidence,
strict observed suffix count equal to token length, observed left/right
boundaries, and live exact handoff lease. Generic `ReplaceTail`, automatic
routes, terminal manual projection, normal `KnownStart`, verifier, SafetyGate,
daemon WordBuffer fallback, and stored legacy focus receipts are unchanged.
First-word automatic hints/autocorrect remain OPEN pending user clarification.

### Historical: connected exact manual handoff source-only preflight before V2 install, 2026-09-12

Preflight consequence analysis for the follow-up source change: the V1 receipt
projection build was reported PASS by the parent/root route, but it was not
installed and is not sufficient as a complete repair. The exact manual replay
chain has three deterministic blockers that must be closed together before a
runtime installation.

First, a context-admission `Transfer` previously copied the shared tail into the
target owner and then unconditionally cleared `preserve_active_path_until`,
`exact_manual_toggle_handoff_epoch`, and `exact_manual_toggle_handoff_path`. That
would let the daemon pass the new field receipt check but fail later at
`SuppressNextAutocorrectV2`, whose V2 arm requires the exact handoff marker to
remain live. The selected repair preserves the existing live marker only for a
valid same-context transfer where shared active path and owner generation match
the grant source, the shared handoff epoch is the grant source tail epoch, the
lease has not expired, and the exact marker still names the original source
engine path. The original source path remains intentional: daemon cleanup can
still cancel `CancelExactManualToggleHandoffV2(epoch, source_path)` if
suppression arming fails after layout. Non-transfer `SourceFree`, `ResetUnknown`,
revocation and failure clears remain unchanged.

Second, a bridge V3 read after controlled layout can happen while the reducer's
current owner is already the target, but the ready activation has not yet been
installed into that target engine or reflected in `SharedState.active_path`. A
plain `active_path()` lookup would still select the old source engine. The
selected repair derives the target engine path from the same fenced
`AdmissionToken`, consumes any already ready activation under that engine's
exclusive guard, then requires the live bridge token and explicit shared active
path match. It adds no extra RPC, wait, poll, controller, cache, or second
current-owner lookup.

Third, explicit first-word manual Double Shift in a GUI `UnknownStart` field can
reuse the same exact manual handoff as an observed-suffix lease, but only before
layout and only with full surrounding-text evidence: live context token, no
atomic/composition/sensitive/trailing-boundary state, no selection, exact suffix
before cursor, observed suffix count equal to current token length, and observed
left/right word boundaries. After a valid transfer, V3 read and exact suppression
may accept that `UnknownStart` only while the exact handoff lease is still live;
they do not require a fresh surrounding-text copy after layout. Generic
`ReplaceTail`, automatic routes, normal `KnownStart`, terminal manual projection,
verifier, SafetyGate and daemon WordBuffer fallback are unchanged. First-word
automatic hints/autocorrect remain unresolved outside this source step pending
user clarification.

This section records the historical source-only preflight before the V2 build and install. It is superseded for delivery state by the V2 installed section above.

### Historical: browser legacy FocusIn exact-tail receipt projection source-only V1, 2026-09-11

Consequence analysis before code change: the fresh physical report `djn` failed
after the IME had already admitted the word and delegated the exact committed
tail route. This is not the earlier source-free first-word `UnknownStart`
refusal. The frozen trace at
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/user-failed-djn-20260911T231756Z/`
shows caps41 browser input, managed commits for `d`, `j`, `n`, source-free
`UnknownStart` through the first word, a normal Space settlement to
`KnownStart`, then Double Shift callbacks that also settle `KnownStart` before
`ibus_manual_toggle_delegation` reports `ime_committed_tail`. The daemon journal
then rejects exact replay before mutation because `VisibleTailV3` lacks the field
focus receipt. Daemon `WordBuffer` is empty in the same cross-check, so daemon
fallback would be the wrong authority route.

The rejected install-stored repair was to fill `client_context.focus_receipt`
when it is `None`. That can preserve a stale fallback or native receipt across a
later legacy `FocusIn` context and would also make a second derived identity part
of the engine's ordinary frame state. The selected bounded repair is bridge-only:
for context-admission engines, `VisibleTailV3` exposes an opaque exact field
receipt derived from the currently revalidated `AdmissionToken`'s admitted
`ContextKey`, namely the IBus connection generation plus canonical context path.
Existing `FocusInId`/legacy stored receipts remain the non-context-admission
behavior. The value is not derived from engine path or activation generation.
It stays stable across the controlled layout handoff for the same context, and
changes across a different `ContextKey` or IBus connection.

The daemon-side V3 guards remain unchanged: source, engine path, exact field
receipt, tail epoch, suffix, layout and focused-window lease are still validated
before replay. No tail epoch, InputState, detector, `KnownStart`, automatic gate,
ranker, verifier, SafetyGate, model, or package authority changes. Existing
handoff state continues to rely on context-admission transfer independently of a
stored legacy `handoff_focus_receipt`; the new receipt is projected only at the
bridge boundary that already requires a live bridge token and `context_word_is_known()`.

The first-word concern remains OPEN and separate. The same trace proves first
word assistance can have suffix-display evidence before Space, but `djn` had no
candidate before Space. This change is not a quality proof and does not silently
expand first-word automatic authority. If first-word Double Shift before Space is
required, it must use the existing observed-suffix evidence route under its own
explicit scope, without promoting arbitrary `UnknownStart` words or falling back
to daemon `WordBuffer`.

This historical V1 source step ran no tests, CI, build, install, smoke, graph
update, or publication and is superseded by the V2 installed section above. Additional consequence
axes: the runtime cost is one bounded `String` allocation on an already requested
`VisibleTailV3` read, with no extra RPC, deadline, timer, state cache, worker,
learning, package, reload, or binary-install change; bridge token revalidation,
race invalidation, focused-window lease and daemon V3 rejection semantics remain
unchanged; the scalar key is stable only for the same admitted `ContextKey` on
the same IBus connection and changes across another context or connection;
rollback is only the two source files until a binary is built; tests denominator
is 0 and quality, RSS and latency are unmeasured. The trace is treated as a
user-reported browser / caps41 GUI episode, not proof of which Firefox/Tor window
was focused, because the current focus after failure was Kitty and the trace has
no application ID.


### Firefox soft Reset exact-ST re-receipt V3 installed, physical pending, 2026-09-12

Final measured delivery state for the V3 follow-up: installation completed with
status `INSTALLED_LOADED_HASH_VERIFIED_PHYSICAL_PENDING`. Runtime authority
changed: true, IME binary only. Physical acceptance remains `PENDING`; the
post-install ping returned `lay-ibus-engine-rs no-focus`, so this record makes no
Firefox/Tor focus, browser text mutation, answer-quality, heldout, RSS, or
latency claim. Tests/CI denominator is 0.

Measured trigger from the post-V2 physical report: Firefox/GTK sends an
authenticated `Reset` after a handled printable managed commit, and then sends
the exact `SetSurroundingText` for the same current tail. Before V3, Reset
revoked the local UnknownStart observed suffix and `reset_for_ibus_soft_reset()`
could republish the tail, so the following manual Double Shift had no full exact
suffix witness.

Build receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/receipt-fix-build-v3-compile2`.
Remote run:
`/home/e/projects/lay-development-runner/browser-receipt-XQR6ov`. Source archive:
1382 files, SHA
`bf8665188a62e077b7095843480b67b1d67aefc835ad4760fa024dee04d407a6`. Build
result SHA:
`5cd763bc62c31af68b8c53351a89d7b86837f842d46a4d1143b8159da910163b`, status
`PASS_RUNTIME_BUILD_ONLY_GRAPH_UPDATED`. The graph AST update happened in the
remote build path; the fetched artifact and hash were verified. The built IME
binary is 7,775,072 bytes with SHA
`67827521149fe73434f8025a6daa404f26d9ac2072d7de7cbe6f3aec0ad57969`. The daemon
binary SHA stayed
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e`, and the
daemon binary was not installed.

Failed predecessor: `receipt-fix-build-v3` stopped before installation on E0063
because `state.rs` was missing the new `context_reset_rereceipt` initializer.
That failed run changed no runtime authority. The initializer was then fixed and
`receipt-fix-build-v3-compile2` became the canonical V3 build receipt.

Installation receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/installation-v3.json`.
Old IME PID `4051893` SHA
`994485bf9d7379c8d820171960c61e5980e59f88341ab51d6a8b7741c08eec80`; new loaded
IME PID `911924`, PPID `4715`, start `80439768`, SHA
`67827521149fe73434f8025a6daa404f26d9ac2072d7de7cbe6f3aec0ad57969`.
`ibus-daemon` stayed PID `4715`, start `2261`; `lay-daemon` stayed PID
`3880511`, start `79007142`, SHA
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e`. Selected
engine `lay-ime-ru` and settings were preserved. Rollback backup root:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/receipt-fix-build-v3-compile2/install-backup`.

The installed invariant is manual-only re-receipt with bridge-token rebinding at
explicit consumption. The pre-Reset live token/scope can contribute only the
already observed suffix text/count and tail epoch. After the reducer handles
Reset, the pending witness is attached to the new post-Reset token and the local
post-Reset UnknownStart zero-count scope. The next surrounding-text revision must
be the one that confirms the witness, and its unselected snapshot must contain
the current committed-tail token as an exact bounded suffix with left/right word
boundaries. A second surrounding-text callback, mismatch, selection, sensitive
field, focus/content/lifecycle change, command/navigation, boundary, or non-exact
input clears the witness.

The witness is not written into `WordScope` on `SetSurroundingText`; this keeps
automatic precognition/hints on the existing generic authority. Explicit Double
Shift is the only consumer: it binds the current UnknownStart lineage to the
manual-only suffix count, settles that lineage in the admission reducer under the
still-live post-Reset bridge token, atomically rebinds the bridge output token to
the new settled token, and then reuses the existing exact manual handoff. Shift
press/release preserves the witness so the daemon gesture can reach
`ManualToggleV3`. Exact handled printable appends may advance the witness from
`N` to `N+1` only when the committed-tail epoch advances by exactly one publish,
and rebind it to the newly settled token/epoch; non-exact appends clear it.

Shared cleanup consequence: while an authenticated reset re-receipt witness is
armed, soft Reset skips the redundant tail `publish_tail_handoff()` so the local
tail text/epoch remain the values already settled by the handled printable.
Generic Reset without that witness still follows the existing
republish/revocation behavior. First-word automatic hints/autocorrect remain
outside this bounded manual route.

### Historical rejected GUI direct-edit experiment, 2026-09-12

This source experiment retained a useful capability-neutral Reset observation:
the old owner/path/lineage/epoch was only a predecessor candidate, and a distinct
live post-Reset token plus exactly one unselected, exact, word-bounded
SurroundingText revision was required before explicit manual consumption. Its
GUI executor was invalid. It returned `Handled` from `ManualToggleV3` after a
legacy sequential `DeleteSurroundingText(-N, N)` and `CommitText`, bypassing the
protected physical-input grab, two lease validations, GNOME layout handoff and
readback, checked suppression, and bounded uinput replay.

The five local focused Rust tests were measurements of that simulated direct
executor only. The six-toggle test kept one engine registered throughout,
injected Reset and SurroundingText callbacks, and asserted emitted IBus delete
and commit signals plus local tail epoch. It did not create the target engine,
perform the source FocusOut/Disable and target FocusIn transfer, call GNOME
`ActivateLayout`, validate the same source/focus/epoch/tail lease before and
after layout, hold a physical-input grab, run uinput replay, or assert the final
visible-text and layout postcondition. Local test execution also did not follow
the remote-only development contract. These measurements cannot establish GTK,
physical-input, cross-engine, or installed behavior.

The source receipt is therefore superseded with status
`REJECTED_SOURCE_EXPERIMENT_PROTECTED_ROUTE_VIOLATION`; it is retained only to
identify what the local tests exercised. No candidate binary was installed or
restarted. Runtime authority changed: false. Exact historical receipt:
`docs/structural_gates/receipts/LAY_GUI_TEXT_TARGET_RESET_RERECEIPT_2026-09-12/source-focused.json`.

### Preflight: exact external tail at both replay leases, 2026-09-12

Status is `PREFLIGHT_TEST_RED_RUNTIME_UNCHANGED`. The currently loaded IME remains
PID `911924`, SHA
`67827521149fe73434f8025a6daa404f26d9ac2072d7de7cbe6f3aec0ad57969`, from the
frozen V3 source archive SHA
`bf8665188a62e077b7095843480b67b1d67aefc835ad4760fa024dee04d407a6`.
No candidate from this preflight is installed.

#### Measured causal hole

The preserved `cycle09` trace establishes one live divergence. External
`SurroundingText` lengths move `12 -> 11 -> 10 -> 9 -> 10 -> 11` and never
return to `12`. The three handled printable callbacks for `d`, `j`, and `n`
advance the internal committed-tail length `5 -> 6 -> 7 -> 8`. Preedit is
published after `d`, cleared before `j`, becomes `inn` after `j`, and is cleared
before `n`. The final observed external tail is `j;jl вотвот`, while the owned
tracked tail is ` вот вот`; the second separator present internally is absent
externally. No Reset occurs in the failing cycle. Therefore the prior Reset
hypothesis is falsified for this trace. One handled insertion did not become
visible in the client's later `SurroundingText`; the trace does not establish
which insertion was lost or why. Client transport and delivery remain
`UNKNOWN`, and the loaded `libim-ibus.so` alone does not identify the selected
client input module or prove a GTK callback route.

`SurroundingText` batching alone does not distinguish the loss mechanism. In
passing cycles 1, 3, 6, 7, 8, 11, 13, and 14, three internal commits also precede
the three external growth receipts; cycle 9 is distinct because only two
growths arrive. This supports a controlled consumer-schedule proof and the
external-tail safety check. It does not justify per-key sleeps or establish the
selected live client transport.

The first observed loss is client delivery in cycle 9. The first demonstrated
authority defect before the next cycle's deletion is transport-independent.
`visible_tail_v3_inner()` projects the IME's internal `committed_tail.buffer`
without requiring it to match the current client `SurroundingText` snapshot.
`arm_exact_manual_toggle_autocorrect_suppression()` checks the internal tail and
the shared handoff against each other, but does not bind either value to current
external text. Thus an internally stale tail can satisfy both the source and
target lease checks. The daemon may then issue the exact number of Backspaces
for the longer internal tail against shorter client text. The delivery cause is
still unknown, but delivery uncertainty must not become deletion authority.

The causal regression must first be run against a mutable copy of the frozen V3
archive. The archive itself remains immutable. A useful RED is a nonzero false
accept in which current unselected external text disagrees with the requested
tail at either lease point, yet V3 returns `DelegateExactImeTail`, preserves the
handoff, or arms suppression. Candidate PASS can be attributed only to closing
that same false accept.

#### Design comparison and selected boundary

1. Keeping the current internal-only source and target lease is rejected. It
   compares two copies of the same internal claim and cannot detect the live
   divergence.
2. The existing reducer, bridge, factory handoff, and typed exact-tail
   delegation are retained. At both source capture and target validation, the
   existing owned tracked tail must agree with the current, unselected external
   `SurroundingText` tail before its cursor. Matching only the requested last
   token with `ends_with` is insufficient: that token must retain its owned word
   boundary and tracked left context, so a dropped separator cannot merge two
   words and still authorize deletion. KnownStart uses this owned-tail
   agreement. UnknownStart remains limited to its separately bounded exact
   receipt. Missing snapshot, selection, mismatch, stale context, or changed
   identity makes the source capture or target validation passive and clears
   exact handoff/suppression authority before any text mutation.
   `ManualToggleV3` may retain its typed exact disposition before the source
   capture. If it instead returns `NotHandled`, the current dispatcher completes
   with no action and does not fall back to the daemon WordBuffer. This is the
   selected boundary.
3. A new retry controller and a direct client edit path are rejected. Either
   would create another authority owner while the current two-lease route can
   express the required evidence. No GTK-specific branch is justified by the
   trace.

The rejected uninstalled branch is bounded precisely to the
`manual_toggle_active_text_target()` arm that consumed Reset rereceipt or a
generic observed suffix and then called `toggle_committed_tail_target()` for a
GUI `ExactSurroundingText` target. Candidate implementation removes that direct
GUI delete/commit dispatch. Retained Reset evidence may only prepare the
existing typed exact-tail handoff and return control to the daemon's protected
physical replay route. This removal does not reset or rewrite the surrounding
baseline work: terminal erase, atomic execution, the existing known-tail
handoff, and Reset observation remain in place.

The `VisibleTailV3` snapshot check is conditional on an exact handoff bound to
the current owner. Ordinary status and display readout carry no mutation lease
and remain readable without this extra precondition. Once an exact handoff is
present, its deadline, epoch, shared buffer, full external tail, and word
boundaries are conjunctive: an expired or mismatched lease is cleared and the
read becomes passive, so neither source capture nor target validation can
release physical replay from stale evidence.

No new reducer, worker, generation, cache, timer, queue, RPC, retry, polling
loop, fallback, application identity, or source of truth is admitted. The
check consumes only the latest snapshot already delivered to the adapter and
does not wait for a future receipt. Focus, owner, token, tail epoch, capability,
content type, source transition, and external snapshot revision remain identity
invalidators. The physical-input grab, one layout activation and readback, two
lease validations, suppression ordering, and bounded physical replay remain
owned by the existing exact replay path.

#### Consequence analysis

- Candidate retention, ranking, false authority, `SafetyGate`, and verifier are
  unchanged. The manual-toggle planner and the new snapshot checks remain
  literal reversible physical-key projection and enter no lexical, morphology,
  L1-L4, learning, or model decision. Successful physical replay still enters
  the existing managed printable path and may schedule its existing prefetch or
  precognition work; this preflight does not remove or measure that work.
- Latency and deadlines gain two bounded in-memory comparisons and no wait,
  retry, polling, or blocking model work. Exact latency and deadline headroom
  remain unmeasured until the remote proof.
- CPU, RSS, allocation, and package size remain unmeasured. The comparison uses
  the existing bounded snapshot lifetime and revision. No worker, queue, cache,
  package, manifest, schema, or persistent allocation is added. Configuration
  or package hot reload cannot grant authority to an old snapshot; the existing
  cleanup and identity invalidations apply without another steady-state worker.
- Missing, selected, mismatched, or stale external context fails closed before
  delete, commit, uinput, or suppression. Source mismatch refuses capture;
  target mismatch after the real factory handoff revokes the captured lease.
- Learning, feedback, automatic acceptance, candidate scores, and suppression
  learning do not change. Suppression may arm only after target identity and
  current external text validate.
- Rollback is the external-snapshot precondition slice plus its tests and this
  documentation. There is no installed-state migration. Terminal and atomic
  executors retain their protected behavior outside this GUI exact-tail
  condition.

#### Bounded proof plan

1. Add one adapter regression using the production `Harness`, `LayImeBridge`,
   factory handoff, and real `SetSurroundingText` callbacks. Two positive cases
   use a left-context prefix and an unselected snapshot whose tail before the
   cursor exactly matches the owned internal tail at source and target: one
   open token and one token with a retained trailing boundary. Both must
   delegate the exact length, accept source capture, survive source
   FocusOut/Disable plus target factory/FocusIn, accept target validation and
   suppression, and emit no local IBus delete or commit.
2. In the same table, prove seven zero-effect refusals: a shorter source text;
   a shorter target text introduced after the actual factory handoff; target
   text changed after successful target `VisibleTailV3` but before the immediate
   suppression arm; a
   same-length source mismatch whose final token matches but whose word boundary
   is missing; selected external text; missing snapshot; and stale context
   identity. Each case must assert no exact mutation authority and cleared
   handoff/suppression state.
3. Apply the test-only patch to a copy of the frozen V3 archive and record the
   exact false accepts. Run the candidate only after V3 is causally RED. Use
   `scripts/dev-check.py check` with the remote `dedicated20cpu` configuration;
   do not run local Cargo.
4. This proof has one observed live divergence and a regression denominator of
   two positive plus seven negative cases. It establishes adapter authority and
   zero local IBus mutation only. It does not establish which client insertion
   was lost, client transport, physical grab/uinput behavior, final visible
   text, latency, RSS, package size, or installed runtime behavior.

#### Frozen V3 causal result

The frozen archive remained immutable. Its source copy required only mechanical
rustfmt normalization in `context_runtime.rs`, `shift.rs`, and `state.rs` to
pass the current mandatory format gate; that normalization was committed before
the uncommitted test-only patch. The normalization and test patch have separate
hashes in the receipt.

The updated remote `dedicated20cpu` check passed formatting, selected 466
focused `lay-ibus-engine` tests, and reached the new regression. Both the open
token and retained-trailing-boundary positives completed source capture, the RU
factory handoff, target validation, and suppression. Stale context identity was
rejected. V3 then reported six causal false accepts: shorter source text;
shorter target text after the actual factory handoff; target text changed after
successful target validation but before suppression; a source text with the
same final token but a missing word boundary; selected source text; and missing
source snapshot. Each path emitted zero local IBus delete/commit effects in the
adapter harness. Local receipt is
`/home/ubu/.cache/lay/development/run-nv1eixc9`; remote run is
`/home/e/projects/lay-development-runner/run-QRC4BO`. This is the required V3
RED for the external-tail authority hole.

The same focused run also failed the unchanged historical test
`tail_memory::tests::focus_engine_can_refresh_empty_tail_from_shared_handoff`
(`left: ""`, `right: "вот"`). That failure is outside the test-only patch and is
reported as a separate baseline failure. A second normalization-only run with
the new regression absent selected 465 tests and reproduced that single failure
(`/home/ubu/.cache/lay/development/run-rivdkaou`, remote
`/home/e/projects/lay-development-runner/run-n8n3et`). The transferred buffer
and epoch are correct; only the old `preedit_fast.token() == "вот"` expectation
is stale because `"вот "` is a closed word and has no open-token candidate
authority. The candidate fixture instead asserts the visible tail and epoch,
empty closed-token state, and reopening plus IME authority after a fresh
printable character. The focused V3 target as a whole is not a PASS. Exact
receipt:
`docs/structural_gates/receipts/LAY_EXTERNAL_TAIL_LEASE_PREFLIGHT_2026-09-12/v3-red.json`.

#### Candidate focused result

The candidate adds the external-tail agreement at source `VisibleTailV3`, at
target `VisibleTailV3` after the real factory handoff, and again immediately
before suppression. The agreement covers the complete owned tail plus the
embedded token boundaries, including a retained trailing whitespace boundary.
Failure clears only an exact handoff owned by the current admission owner and a
matching `ExactReplay` suppression. A stale source and a current owner both
preserve the target's `CurrentWord` suppression.

The rejected direct GUI mutation branch was replaced by preparation of the
existing typed exact handoff. The retained Reset rereceipt regression now proves
`DelegateExactImeTail` and zero local `DeleteSurroundingText`/`CommitText`
effects. Terminal fixtures declare terminal purpose and retain the separate
erase executor; atomic contracts remain on their existing route.

The remote `dedicated20cpu` candidate check passed formatting and all 470
selected `lay-ibus-engine` tests with zero failures. Local receipt is
`/home/ubu/.cache/lay/development/run-6yjp8x5k`; remote run is
`/home/e/projects/lay-development-runner/run-8f5G0o`; remote snapshot SHA-256 is
`dbcdcc58e2bd8c27966f5df7ef3e937c339d85730cfb73227a0d3c3234aafcf7`.
Exact candidate receipt:
`docs/structural_gates/receipts/LAY_EXTERNAL_TAIL_LEASE_PREFLIGHT_2026-09-12/candidate-focused.json`.

No binary build, installation, restart, live mutation, or installed runtime
authority change occurred. Physical input grab/uinput replay, final visible
text, client transport, latency, RSS, and package size remain untested.

### Mutter/GTK event-dispatch consumer probe, 2026-09-12

#### Historical rejected handwritten mirror

The first probe is retained with status
`REJECTED_MIRROR_MODEL_LOCAL_EXECUTION`. It hashed 16 extracted upstream
functions but compiled a separately handwritten model, so the hashes did not
bind the executed algorithms. Its release control also skipped the real
asynchronous order: it supplied the callback result to the initial filter and
discarded the native copy instead of redispatching the INPUT_METHOD-flagged
event. Finally, it compiled and ran under the local workstation guard despite
the remote-only development contract. Its four rows carry no causal acceptance
and authorize no implementation. Historical receipt:
`docs/structural_gates/receipts/LAY_MUTTER_GTK_EVENT_DISPATCH_PROBE_2026-09-12/source-bound.json`.

#### Exact extracted remote probe

Status is `PASS_EXACT_EXTRACTED_SCHEDULE_EXISTS_RUNTIME_UNCHANGED`. The v2
generator inserted 17 byte-identical function definitions from the four pinned
Mutter 50.1 and GTK 4.22.4 files into the compiled translation unit. The exact
fragments total SHA-256 is
`bfcbed5f988e3b1838d46cc8836cd61e2ec27f21f0c865b1a7c38f84c64c2e15`;
the generated unit SHA-256 is
`3f122cf8140c4010de19bf96190eee15638df6d1788954a5d4d4fa970d8a6afe`.
Only platform types, Clutter queue ingress, output observation, unused
preedit/delete/pointer branches, and object accessors are stubs. The idle
scheduler is the remote worker's real GLib 2.72.4. Wayland commit and done sinks
call the exact extracted GTK callbacks.

The test now preserves the asynchronous order. Each original unflagged press or
release enters exact `meta_wayland_text_input_update()` and exact
`clutter_input_method_filter_key_event()`. Only its external virtual
`im_class->filter_key_event` callback is stubbed to accept the initial event
asynchronously. The callback later enters exact
`clutter_input_method_commit()` and
`clutter_input_method_notify_key_event()`. When that reply is unfiltered, the
exact notify callback enqueues an INPUT_METHOD-flagged native copy; its later
redispatch re-enters the exact update/filter branch as unfiltered and only then
flushes pending `done`.

The exact callback sequence admits the overwrite schedule. Three filtered
press/release pairs dispatched three `CLUTTER_IM_COMMIT` events through the
exact focus and Wayland handlers before the idle ran. Mutter emitted three
commit strings and deferred one `done`; GTK's exact `text_input_commit()`
replaced its single `pending_commit` twice. Before idle there were six initial
filter calls, three protocol commits, no native copy, no done and no GTK commit.
After the coalesced idle, the exact GTK `text_input_done()` applied one commit,
`c`. A control that drained GLib after every character emitted three done and
preserved `abc`.

The corrected release control also preserved `abc`. Each release was initially
accepted asynchronously; its later unfiltered callback enqueued one flagged
native copy. Three copies were queued and three were redispatched through the
exact update/filter branch. Each redispatch flushed the pending `done`, yielding
three GTK commits. This proves that unfiltered release redispatch can supply a
flush boundary on this tested transport. It does not establish a safe repair:
the behavior depends on the client using Mutter's Wayland text-input-v3 route
and on release-only native delivery being harmless.

Transport remains `UNKNOWN`. GTK scans all modules; a resident IBus module does
not prove selection, `GTK_IM_MODULE` and the `gtk-im-module` setting are unset,
and native Wayland priority 100 exceeds IBus priority 50. No affected GNOME
Text Editor focus observation was captured. The release-only alternative
therefore remains unselected. Exact native replay retains the protected
client-independent transaction and no production change was authored.

The final accepted run executed once on `e@192.168.3.94` under the dedicated-20cpu
resource guard. All four assertions passed, compiler and sanitizer stderr were
empty, and runtime authority did not change. The preceding fresh remote attempt
is preserved separately: it stopped at compile because original unused callback
parameters met standalone `-Werror`; v2 added only
`-Wno-unused-parameter`. A subsequent 16-function run passed but remained
incomplete because it mirrored the central INPUT_METHOD rejection branch; that
receipt is preserved as `v2-remote-16-function-incomplete.json` and carries no
acceptance. The final run inserted that original function as fragment 17 and
moved the initial/native counters outside the tested body.
Exact receipts:
`docs/structural_gates/receipts/LAY_MUTTER_GTK_EVENT_DISPATCH_PROBE_2026-09-12/v2-remote-exact.json`
and
`docs/structural_gates/receipts/LAY_MUTTER_GTK_EVENT_DISPATCH_PROBE_2026-09-12/v2-transport.json`.
Remote accepted run:
`/home/e/projects/lay-development-runner/mutter-gtk-dispatch-probe-v2/run-k1FFeX`;
local fetched artifact:
`/home/ubu/.cache/lay/development/gnome-repeat-capture-d9q6uzjm/mutter-gtk-dispatch-probe-v2-remote-run-k1FFeX`.

This result proves only that the exact callbacks admit the tested schedule. It
does not show the selected GNOME Text Editor transport, the packet order of
captured cycle 9, which insertion was lost, or any visible pixel. Live clients,
Wayland traffic, physical replay, latency, RSS, package size and installed
behavior were not tested; answer quality is `UNKNOWN`.

### Preflight: exact-replay native delivery and side-effect quarantine, 2026-09-12

This section began as `PREFLIGHT_ONLY_REVIEW_PENDING_RUNTIME_UNCHANGED`. The
reviewed implementation result is recorded separately below; the preflight text
is retained as the contract against which that candidate was measured.

#### Current consequence and rejected release-only alternative

The protected replay currently arms `ExactReplay`, emits the leased Backspaces,
and replays the replacement keycodes under the target layout. Each replayed
printable then re-enters ordinary legacy `ProcessKeyEvent`. The managed branch
emits `CommitText(ch)`, calls `push_tail_char(ch)`, schedules Space prefetch, and
refreshes precognition. `push_tail_char()` can finalize pending completion
editing, record a prediction outcome at a replayed boundary, refresh suppression,
and publish a new shared tail epoch. The outer callback performs another
observed-suffix refresh whenever the tail changes. Avoiding only one refresh is
therefore insufficient, especially when the exact replacement retains a trailing
space.

Returning replay releases unhandled is rejected as the repair. The exact
Mutter/GTK probe proves that an unfiltered release redispatch flushes pending
`done` only on its tested Wayland text-input-v3 callback route. The affected
GNOME Text Editor transport remains `UNKNOWN`: a resident `libim-ibus.so` does
not prove selection, the relevant GTK module settings are unset, native Wayland
has the higher advertised priority, and no affected editor focus observation
was captured. More fundamentally, release-only forwarding leaves every replay
press on the managed `CommitText -> push_tail_char -> worker/learning` path. It
cannot establish zero replay learning or zero candidate resurrection even if a
consumer happens to use the tested flush schedule.

Per-key sleeps, a cursor-location wait, or completion of the daemon's uinput
write are also not delivery receipts. The client applies an unhandled event
asynchronously. No timing gap, daemon write completion, or unchanged internal
tail may be promoted into a client-visible postcondition.

#### One text-target contract, existing owners

The common contract is capability- and fact-based; it is not an application
list and does not require moving every executor into one file. The existing
`manual_toggle_authority()` first preserves pending auto-undo and active
composition ownership, then distinguishes an IME committed tail from the proven
daemon WordBuffer route. `TextTargetEditRoute::select()` chooses commit-only,
exact SurroundingText, terminal erase, or unsupported from the requested delete
and available capabilities. Exact snapshot, selection, sensitive-content,
Reset re-receipt, focus/owner and atomic exclusions remain conjunctive guards.

The existing owners execute that one contract: `shift` performs gesture
routing; `committed_tail` owns authorized IME and terminal edits;
`ContextAdmissionReducer` and `context_runtime` own context authority and Reset;
the bridge binds typed leases to the active engine; and the daemon's existing
exact-replay executor owns layout handoff, physical isolation and uinput. The
native delivery contour below completes the exact GUI executor inside those
owners. It does not add a controller, timer, RPC, daemon state machine or
scheduler.

Unsupported facts remain explicit refusal outcomes. Missing exact
SurroundingText, a selection, sensitive content, stale owner/path/epoch, or an
unproven delete backend cannot authorize an exact GUI edit. Capability value 9
alone is not a blanket refusal: it may still belong to a separately proven
terminal or daemon route. A target window is supported only when its current
facts admit one of the typed executors.

Native replay and the optimistic IME tail are not client confirmation. A later
matching `SetSurroundingText` may confirm the visible postcondition. Until then,
the existing current-external-snapshot check remains the gate for any next
destructive exact cycle; daemon uinput completion cannot renew that authority.

#### Selected bounded design

Reuse the existing exact suppression arm as a transaction lease. At its already
validated target-side arm, derive the replacement again through the same literal
`ManualToggleV3` projection using the original source layout, and retain a
bounded immutable delivery contour inside `ExactReplay`:

- target path and existing armed tail epoch;
- current runtime owner lease identity and expiry;
- exact original owned tail and leased suffix;
- exact unchanged prefix before that suffix; and
- exact projected replacement, including retained trailing spaces.

This adds no new RPC, reducer, worker, queue, timer, retry, model call, candidate
source, executor, application identity, or persistent schema. The strings are
already bounded by the committed-tail and exact replay limits and exist only for
the lifetime of the current exact suppression.

The contour phase must use the existing tail epoch as well as exact text. If the
armed epoch is `E`, deletion count is `N`, and replacement length is `M`, the
only admissible states are:

1. deletion step `k`, where `0 <= k < N`, current epoch is exactly `E + k`, and
   the current mirror is the original tail with exactly `k` final characters
   removed; only the next Backspace may advance it;
2. insertion step `j`, where `0 <= j < M`, current epoch is exactly
   `E + N + j`, and the current mirror is the unchanged prefix plus exactly the
   first `j` replacement characters; only the next expected printable may
   advance it; and
3. complete, where current epoch is exactly `E + N + M` and the mirror is the
   unchanged prefix plus the complete replacement.

Use bounded wrapping epoch distance consistently with the existing wrapping
tail epoch, and reject any distance outside `N + M`. The owner identity, target
path, suppression identity, expiry, exact mirror, and phase are conjunctive.
Epoch is necessary: a digit or punctuation character unchanged by layout
projection can make a deletion state textually equal to a later insertion state.
Text alone would allow an out-of-order or repeated event to select the wrong
phase.

At an admissible insertion step, determine the glyph from
`passthrough_visible_char(keyval, keycode)` and require it to equal the next
expected replacement character. `physical_char()` is not an eligibility proof:
it prefers the selected engine layout, while the native client receives the
keysym and can therefore see a different character. An eligible press updates
the optimistic mirror once and returns `handled=false`; it emits no `CommitText`.
Because that press is not remembered as handled, its release follows the
existing unhandled release path without a new release policy.

The replay mirror update reuses the existing invalidation and mirror owners:
cancel precognition display work, invalidate the path's Space prefetch, clear
preedit candidates, replacement targets, observed prediction and pending
display identity, clear pending completion learning, invalidate the stale
surrounding snapshot, append and trim the exact visible glyph, and publish the
tail handoff once. It must not call `push_tail_char`, finalize or confirm
completion feedback, record a prediction outcome, schedule either worker, or
publish preedit. The outer observed-suffix refresh recognizes the same exact
epoch phase and remains quarantined for that replay change; otherwise it would
recreate the work just cancelled by the inner path.

Quarantine begins before the first replay Backspace. That Backspace must bypass
`begin_pending_ime_completion_edit_before_backspace()` and any completion-edit
feedback as well as candidate refresh. Deletion and insertion reuse one shared
tail-mirror transition primitive: pop or append the exact character, invalidate
the same background/candidate state, and publish one epoch. They must not copy
tail ownership into a second buffer or implement a second trim/publish path. If
a stale visible preedit must be hidden, clear it once when the contour is armed;
do not emit a clear or preedit update for every replay key.

After the final replayed character, the delivery contour has no next character
and cannot classify a later press as replay. A retained trailing space consumes
the existing one-shot exact autocorrect suppression at this boundary without
running boundary learning. For an open-token replacement, retain only the
existing next-boundary autocorrect suppression. Before the first later ordinary
printable or Space, retire the completed delivery contour, rebuild only the
open-token fast mirror from the exact tail, and enter the unchanged ordinary
managed/native route. Candidate work and learning may resume from that new user
input; no replay event itself may enqueue or confirm them.

Any path, owner, epoch, expiry, text, order, duplicate, keyval, or projected-glyph
mismatch revokes the contour through the existing identity-bound exact cleanup
and invalidates background work. While an active contour expects another replay
step, the rejected offending press is consumed with `handled=true` and zero
native, `CommitText`, or `DeleteSurroundingText` effect. It cannot fall through
to managed commit or authorize native insertion using a physical-layout guess.
Exact visible recovery after output from earlier steps is indeterminate; do not
add a second output fallback or claim rollback of text already delivered.

Reset re-receipt keeps its existing `handled=true` rule. The explicit Double
Shift path calls `consume_context_reset_rereceipt_for_exact_manual_handoff()`;
that function validates the confirmed receipt and takes it before
`prepare_exact_manual_toggle_layout_handoff()` publishes the daemon handoff.
Replay begins only after that delegation. Its later native-unhandled printable
therefore sees no pending Reset re-receipt, and
`advance_context_reset_rereceipt_after_key()` returns immediately. Widening the
re-receipt rule to accept `handled=false` would admit unrelated client input and
is not part of this repair.

#### Consequences and proof boundary

- Candidate generation, ranking, `SafetyGate`, verifier, L1.1-L4 packages, and
  manual projection remain unchanged. Replay produces no candidate denominator
  and proves no answer quality.
- Replay adds bounded string/epoch comparisons on the existing key callback and
  removes per-character `CommitText` plus replay worker scheduling. CPU, RSS,
  allocation and latency effects are unmeasured. There is no blocking model work
  or new wait.
- The mirror after an unhandled press is optimistic. Daemon/uinput completion is
  not a client acknowledgment. Current external-tail checks must still reject a
  later destructive exact handoff until a fresh unselected `SurroundingText`
  agrees. Final visible text remains an actual-client proof obligation.
- Focus, owner, content type, selection, Reset, capability, factory transfer,
  path, suppression revision, epoch, and expiry keep their existing invalidation
  authority. Configuration or material hot reload cannot widen the contour.
- Terminal native typing, atomic input, active composition, direct committed-tail
  replacement, daemon WordBuffer replay, ordinary managed typing, and the gesture
  detector remain outside this exact suppression branch.
- Rollback is the connected exact-delivery contour, quarantine checks and their
  tests. No installed-state migration is required.

#### Required causal RED and candidate proof

Before implementation, add the fixed tests to an unchanged-source copy and run
them remotely under the dedicated-20cpu guard. The useful delivery RED is the
current exact suppression accepting replay presses through managed `CommitText`
instead of required native delivery; it is linked to the accepted exact
original-consumer schedule proof. Measure worker and learning counters as
separate denominators. Their absence on a particular fixture does not erase the
delivery RED, and a model-dependent side effect is not required for RED. A
source string search alone is insufficient.

The candidate proof has separate denominators:

1. Five exact transactions: US-to-RU and RU-to-US open tokens; both directions
   with one retained trailing space; and a token containing an unchanged digit
   prefix. Drive the real press/release callback sequence.
   Require exactly `N` native Backspaces, exactly `M` unhandled printable
   presses, unhandled paired releases, zero `CommitText`, exact epoch progress,
   exact optimistic final mirror, and no duplicate/missing visible glyph in the
   controlled native sink.
2. Six ordered-progress refusals: printable before deletion completes, duplicate
   Backspace, skipped epoch, duplicate printable, wrong keysym despite a matching
   `physical_char`, and owner/path expiry or identity change. Each must release
   zero native-delivery authority from the rejected phase, clear the matching
   contour, and leave no second mutation fallback.
3. Side-effect assertions on every positive transaction: zero Space-prefetch
   schedule, zero precognition schedule/apply, no pending display frame or
   candidate/replacement target, no observed prediction outcome, no accepted or
   edited completion feedback, and no pending completion-learning record. Seed
   stale work and candidate state first so invalidation is distinguished from an
   initially empty fixture.
4. Lifecycle controls: a completed open-token replay followed by one ordinary
   letter and Space uses the ordinary path and existing one-shot suppression; a
   completed trailing-space replay followed by a new word uses ordinary typing
   immediately. A confirmed Reset re-receipt is consumed before delegation and
   remains absent throughout the later handled-false replay without changing its
   `handled=true` continuation rule.
5. Run the affected IME callback, native-transfer, residual, terminal, atomic,
   manual-toggle and package/architecture gates. Then refresh the architecture
   graph only after the final source and document are fixed. All Cargo work and
   graph acceptance run remotely; local source inspection carries no PASS.
6. Final acceptance requires a fresh physical GNOME Text Editor run that observes
   the exact resulting text and matching post-replay `SurroundingText` after many
   repeated bidirectional toggles, plus ordinary typing afterward. Keep transport
   attribution, callback correctness, final visible text, learning/candidate
   isolation, latency, RSS/package size and answer quality as separate claims.

This preflight selected only the bounded exact-replay delivery contour for a
causal RED/PASS attempt. At that point it did not authorize production edits,
build, installation, restart, or runtime mutation before review.

#### Candidate result: focused callback proof, runtime unchanged

Status is `CANDIDATE_FOCUSED_CALLBACK_PASS_REVIEW_ACCEPTED_RUNTIME_UNCHANGED`.
Production source now contains the reviewed contour, but no candidate binary was
built or installed and no process, input source, setting, package, or runtime
authority changed. Canonical evidence belongs at
`docs/structural_gates/receipts/LAY_EXACT_REPLAY_NATIVE_DELIVERY_2026-09-12/candidate-focused.json`.

The common TextTarget contract is a composition of existing public entrypoints
and owners. It is not a new authority object:

- `manual_toggle_outcome_inner()` exposes the typed gesture result
  `ImeManualToggleOutcome::{Handled, DelegateExactImeTail, DelegateDaemon,
  NotHandled}` after `manual_toggle_authority()` preserves auto-undo and active
  composition priority.
- `text_target_decision(backspaces)` maps the bounded capability facts to a
  `TextTargetDecision { route, reason }`; its route is one of `CommitOnly`,
  `ExactSurroundingText`, `TerminalErase`, or `Unsupported`, and refusal retains
  the typed `MissingProvenDeleteCapability` reason. These three capability facts
  are only the output-backend selector, not complete TextTarget authority.
- `ContextAdmissionReducer`, `context_runtime`, and the bridge token checks keep
  focus, Reset, path, owner, lineage, selection, sensitive-content, atomic, and
  exact-snapshot guards with their existing owners. `can_replace_committed_tail_inner()`
  exposes the guarded executable capability without moving those guards.
- `replace_committed_tail()` executes the admitted typed edit and records the
  selected output route or exact refusal reason. Exact daemon replay then enters
  `process_exact_replay_press()` before the ordinary managed callback; it returns
  `Native`, `Rejected`, or `Inactive` for the current immutable contour.
- `set_surrounding_text()` records the client observation. A new destructive
  lease still requires `current_external_snapshot_agrees_with_owned_tail()`;
  neither the optimistic mirror nor daemon uinput completion renews authority.

Measured facts from the fixed remote focused proof:

- Six complete bidirectional transactions passed: lower-case US-to-RU through
  the real Reset, exact SurroundingText, live-token bridge and re-receipt
  choreography; uppercase `Ghbdtn -> Привет` with real Shift press/release;
  RU-to-US; both trailing-space directions; and an unchanged digit prefix.
  Replay presses and paired releases were unhandled, the controlled
  callback sink and epoch sequence were exact, and no `CommitText` or
  `DeleteSurroundingText` was emitted.
- Six refusal mechanism classes passed through nine concrete subcases:
  premature printable, command modifier, duplicate Backspace, skipped epoch,
  duplicate printable, client-visible keysym mismatch, owner mismatch, active
  path mismatch, and active expiry. All nine offending presses were handled and
  emitted no text mutation signal. Mirror preservation was asserted directly for
  premature printable, duplicate printable, path mismatch, and active expiry;
  local suppression removal was asserted directly for premature printable,
  duplicate Backspace, skipped epoch, owner mismatch, path mismatch, and active
  expiry. The path-mismatch case also asserted preservation of the separately
  owned shared scope for the original engine. The other subcases establish
  callback refusal and signal behavior without adding broader mirror/cleanup
  denominators.
- Every positive seeded stale preedit, candidate, pending-display and completion
  learning state. In the ordinary fixtures, the first replay callback emitted
  exactly one empty preedit update plus hide; the Reset fixture had already
  emitted that single clear through the real Reset before the contour arm.
  Later replay and interleaved `SetSurroundingText` callbacks emitted no text
  effect. With production precognition flags enabled after the Reset arm, actual
  successful schedule/apply counters stayed `(0, 0)` and completion feedback
  stayed empty. The ordinary UnknownStart control reached `(1, 0)`, so the zero
  replay count was not produced by a disabled probe.
- Completed open-token and trailing-space contours retired before later ordinary
  input; completed-expired state could quarantine a late matching snapshot but
  could not consume the next ordinary key. The Reset case consumed its confirmed
  re-receipt during the live-token bridge handoff and kept it absent throughout
  unhandled replay.
- The dedicated-20cpu focused lane passed formatting and all selected
  `lay-ibus-engine` tests. The canonical receipt binds the final source snapshot,
  request, result, summary, and binary test log. Canonical-manifest drift is
  recorded because the candidate adds tests; no manifest was mutated in this
  focused lane.

Not tested here: affected-closure or full release gates, the dedicated physical
Double Shift owner test as a separate gate, final architecture graph, release
build, installed process behavior, actual GNOME Text Editor native delivery,
selected editor transport, visible pixels, many-cycle physical acceptance,
latency, RSS, package size, or answer quality. Transport and answer quality
remain `UNKNOWN`; physical acceptance remains `PENDING`.

The subsequent canonical remote graph refresh passed on the same Rust source:
22,327 nodes, 59,224 edges, 814 communities and 700 Rust sources. All eleven
architecture checks reported PASS with zero violations; the 26 architecture
unit tests, two generated-binding tests, and `check-architecture.sh` passed. The
generated graph, source binding and compiled architecture receipt were fetched
back with exact hashes. This proves AST/architecture coverage only; it does not
upgrade callback behavior, desktop delivery, physical acceptance or answer
quality. Exact receipt:
`docs/structural_gates/receipts/LAY_EXACT_REPLAY_NATIVE_DELIVERY_2026-09-12/graph-final.json`.

#### Final changed-suite structural matcher correction

The first complete documented release snapshot reached the canonical hermetic
lanes with zero reported failures in every earlier target, then stopped in
`test:typing_transition_authority_contract`: that target ran 21 tests, with 20
passing and one failing. The canonical manifest still selected 2,793 total tests
across all lanes, but the stopped run supplies no full-suite PASS denominator.
The failure was the source-order assertion in
`double_shift_exact_auto_undo_is_a_protected_first_priority_contract`. It used
the first textual occurrence of `self.manual_toggle_authority()`. The new
unknown-word Reset re-receipt branch queries that capability before entering
the separate known-word protected auto-undo branch, so the matcher selected a
capability query rather than the manual/layout fallback it was intended to
order.

Production order inside the protected branch remained
`defer_pending_ime_auto_undo_until_visible -> undo_last_ime_autocorrect ->
manual/layout fallback`. The contract now locates the explicit fallback binding
`let authority = self.manual_toggle_authority();` and retains the same
`undo < manual` assertion and `PROTECTED USER CONTRACT` marker. No runtime source,
authority, candidate, verifier, or safety behavior changed in this correction.
The failed log hashes are
`e9973125bad87ce9c73f0f1fd85c2b4115bb466dd3469a6eb5c2c8e4b9955f5f`
for the changed-suite log and
`98124e0192455f75fe3aaf04a626a29ca2d4a9dffc9794b6bac8b8ba0308d06a`
for the target log. The corrected exact contract then passed `1/1` remotely
under the dedicated-20cpu guard from source archive
`48e19e9117a9c064b78205526ab8d5980075f0b3311e02c96f33a607d3e135c2`;
its log SHA-256 is
`d113c69ef6750f73005f14e1e804ce187a5c31f8de45bcd6ac591be17e8253d3`.
The later test comment and denominator wording are documentation-only changes
outside that exact-test archive. A new complete changed suite and refreshed
graph remained required at that point; neither the failed run nor the focused
repair granted release or runtime authority.

The post-repair canonical graph refresh then passed remotely: 22,328 nodes,
59,225 edges, 842 communities and 700 Rust sources; all eleven architecture
checks had zero violations, 26 architecture tests and two generated-binding
tests passed, and `check-architecture.sh` passed. This refresh binds the changed
contract source and updates
`docs/structural_gates/receipts/LAY_EXACT_REPLAY_NATIVE_DELIVERY_2026-09-12/graph-final.json`.
It remains architecture evidence only. The complete changed suite, release
build, installed runtime and physical application proof were not tested by it.
