# Text Correction Gate Architecture

> Execution authority: `docs/phase-word-recovery-canonical-cutover.md`.
> The mutation boundary below remains valid, but implementation order and
> L1-L4 ownership come from the canonical cutover.

This note is the working architecture contract for text mutation paths.

## Pipeline Tree

```text
input stream
|
+-- L1 surface sensors
|   |
|   +-- character shape
|   +-- layout shape
|   +-- token boundary
|   +-- local n-gram evidence
|
+-- L2 candidate field
|   |
|   +-- deterministic layout/typo candidates
|   +-- learned surface candidates
|   +-- boundary candidates
|   +-- completion candidates
|
+-- L3 phrase/context gate
|   |
|   +-- boosts or suppresses L2 candidates
|   +-- may forecast phrase-local continuations
|   +-- must not directly own destructive text edits
|
+-- correction core
|   |
|   +-- builds the candidate lattice
|   +-- assigns candidate roles through correction_source_contract
|   +-- records status-only quality/latency counters
|
+-- text edit gate
|   |
|   +-- the only public owner for planned destructive replacement actions
|   +-- authorizes edit plans through text_edit safety
|
+-- output backend
    |
    +-- daemon replay
    +-- native text replace
    +-- IME backend display/commit
```

Detailed `L1.1 -> L2` contract:
`docs/l2-l11-candidate-field-contract.md`.

Canonical internal `L2` architecture above `L1.1`:
`docs/l2-l11-canonical-architecture.md`.

Current live local route on 2026-07-26:

```text
L1.1 bounded lattice
-> one real L2 local field
-> one local readout
-> L3
-> verifier
```

The old lexical `CompactL2` route is no longer an executable/public candidate
route. `FullWave` remains the compare reference only.

Short local-surface safety tightening measured on 2026-07-27 is recorded in:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_SHORT_GROWTH_GATES_2026-07-27.json`

Confirmed route facts from that receipt:

- `слои ` now stays `None` on both `FullWave` and `L2FieldShadow`;
- `ене ` now stays `None` on both `FullWave` and `L2FieldShadow`;
- `сделам ` stays in live surface parity as `сделай ` on both routes.

IME is a display and commit backend. It is not a second correction brain.

## Double Shift Undo Contract

As of 2026-07-27, the immediate double-`Shift` route gives a fresh recorded
autocorrect undo priority over the focused IME manual toggle:

```text
autocorrect apply
-> remember pending_auto_undo(original, replacement)
-> next confirmed double Shift
-> daemon checks pending_auto_undo before calling IME manual toggle
-> recorded undo edit
-> original text restored
```

Scope:

- manual text replacement;
- typing-assist autocorrect;
- layout-only typing-assist autocorrect;
- Nanda/L1.1-backed boundary autocorrect.

This means layout-only autocorrect is no longer exempt from double-`Shift`
rollback. If the user rejects the last autocorrection, the next confirmed
double `Shift` must restore the original visible text instead of forcing a new
candidate replay path.

The previously implemented storage-only change did not satisfy this contract.
The real trigger route was:

```text
double Shift
-> run_ime_manual_toggle()
-> focused IME consumes committed tail
-> daemon handle_double_shift() is skipped
-> pending_auto_undo remains unread
```

The corrected ownership rule is:

```text
fresh pending_auto_undo
-> daemon auto-undo owner

no pending_auto_undo or expired pending_auto_undo
-> existing IME-first manual-toggle route
-> daemon manual correction only when IME declines
```

### 2026-07-27 trigger-ownership proof

Tested:

- a fresh pending undo bypasses the IME-first branch without consuming the
  undo during readiness inspection;
- no pending undo preserves the existing IME-first route;
- an expired pending undo is cleared and does not steal the normal manual
  toggle;
- typing-assist layout autocorrect and manual text correction both retain the
  complete original text for undo.

Measured facts:

- `pending_auto_undo` readiness tests: `2/2 PASS`;
- daemon route and replacement-plan tests selected by `pending_auto_undo`:
  `3/3 PASS`;
- existing typing-assist and manual-correction storage regressions:
  `2/2 PASS`.

Not tested by this proof:

- a physical double-`Shift` against a live focused browser field;
- visual text restoration in every supported desktop backend.

Verdict scope:

- source-level trigger ownership changed from unconditional IME-first to
  daemon-auto-undo-first only while a fresh undo exists;
- normal manual toggle ownership is unchanged when no valid undo exists;
- release binaries were installed and `lay-daemon` plus managed
  `lay-ibus-engine` were reloaded at version `0.2.324`;
- final live behavior still requires a real focused application check; no
  synthetic keyboard injection is counted as proof.

Receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/DOUBLE_SHIFT_AUTO_UNDO_TRIGGER_OWNERSHIP_2026-07-27.json`

### 2026-07-27 first live check: FAIL

The first physical browser-field check disproved the source-only verdict.
Trigger ownership was correct, but the IME rejected the undo before mutation:

```text
double Shift
-> daemon takes pending_auto_undo
-> IME ReplaceTailV4
-> stale_visible_tail: expected "проверка ", actual ""
-> no visible mutation
-> pending undo was already consumed
```

The matching action log showed:

```text
auto-undo intent: "проверка " -> "проверрка "
IME guard: stale_visible_tail
```

The follow-up contract is intentionally limited to explicit recorded undo:

```text
IME Dispatched
-> success, no fallback

IME Indeterminate
-> fail closed, no fallback

IME Rejected / NotDispatched
-> backend proved no mutation
-> daemon/uinput explicit-undo fallback

no usable daemon backend
-> restore the same pending undo with its original age
```

This does not allow ordinary autocorrection or automatic destructive edits to
retry through a second backend. Only an explicit user rollback may use a
confirmed-no-mutation receipt for backend reselection.

### 2026-07-27 second live check: engine-path handoff loss

The next physical `djn -> вот` check exposed a separate failure before the
rollback planner could mutate text:

```text
djn + Space
-> IME autocorrects djn -> вот
-> layout sync switches lay-ime-us -> lay-ime-ru
-> the new engine path binds
-> bind_focus_path() clears the shared committed-tail handoff
-> double Shift sees an empty IME tail
-> ordinary layout toggle runs instead of rollback
```

The matching live log recorded:

```text
ibus_space_autocorrect authorized
ime_committed_tail: djn -> вот
ibus_layout_sync target_is_ru=true
focus_out / disable
focus_in receipt=new_path
double_shift_defer_to_daemon tail_chars=0
stale_visible_tail expected="djn " actual=""
```

The ownership correction is:

```text
layout autocorrect publishes a bounded preserve lease
-> engine profile changes
-> the new path hydrates the shared handoff tail, epoch, and focus receipt
-> immediate double Shift can read the committed replacement tail

ordinary focus/path change without a valid lease
-> quarantine the old handoff exactly as before
```

Tested:

- a fresh layout-switch preserve lease transfers `вот ` to the newly bound
  engine path;
- an expired preserve lease quarantines the handoff;
- an ordinary changed engine path without a focus receipt still quarantines the
  handoff.

Measured facts:

- fresh and expired layout-switch handoff tests: `2/2 PASS`;
- existing ordinary changed-path quarantine test: `1/1 PASS`;
- release `lay-ibus-engine 0.2.324` built and installed;
- only the managed `lay-ibus-engine` process was replaced; global IBus and
  `lay-daemon` were not restarted.

Not tested by this source proof:

- successful visible `вот -> djn` restoration in a focused live application
  after installing this exact build.

Verdict scope:

- source and installed runtime now preserve the committed tail only across the
  bounded layout-switch lease;
- runtime authority is unchanged;
- final physical-browser verdict remains open.

### 2026-07-27 delayed surrounding-text observation

The first automated GTK run after the engine-path fix still produced:

```text
вот ашду
```

The new live trace showed that `bind_focus_path()` was no longer the first
destructive event. GTK reported its pre-commit surrounding text once,
immediately after the IME committed `вот `:

```text
commit djn -> вот
-> first SetSurroundingText still describes the old visible value
-> visible postcondition mismatch
-> immediate quarantine clears handoff tail and preserve lease
-> new engine path receives no editable tail
```

The postcondition observer now treats this as an eventually consistent
observation:

```text
first mismatching observation within 500 ms
-> pending_stale_observation
-> keep postcondition and handoff

matching observation
-> confirmed_positive

mismatch after the settle grace
-> quarantine
```

This does not authorize an edit against a stale snapshot. It only keeps the
already committed tail available while the compositor publishes the matching
surrounding-text state. Every later replacement still passes the normal
visible-tail transition guard.

Measured facts:

- previous visible-postcondition tests: `4/4 PASS`;
- new stale-then-confirmed observation test: `1/1 PASS`;
- exact live GTK scenario:
  `djn + Space -> вот -> double Shift -> djn`;
- the installed engine handled the rollback as
  `double_shift_committed_tail`;
- strict continuation scenario returned `djn` instead of expected
  `djn file`: rollback passed, but the short-lived test virtual device closed
  while a cold isolated daemon was still handling the trigger.

Verdict scope:

- automated live visible rollback: `PASS`;
- immediate continuation through the short-lived runtime harness: `FAIL`;
- physical focused-application confirmation on the persistent system keyboard:
  not yet tested after this build;
- runtime authority remains unchanged.

### 2026-07-27 generic exact autocorrect undo: PASS

The accepted contract is not limited to wrong-layout recovery:

```text
typed damaged word
-> IME authorized autocorrect
-> immediate double Shift
-> restore the exact recorded original
```

The structural verifier originally projected `ImeAutoUndo` as
`Undo + UndoRecord`, but attempted to seal it through
`AutomaticDecision`. That authority correctly rejects recorded undo, so the
backend never received an executable edit and ordinary layout toggle ran next.
`ImeAutoUndo` now seals through the existing `RecordedUndo` authority.

Tested:

- live GTK4 + IBus input: `доллора `;
- authorized autocorrect: `доллора ` -> `доллара `;
- immediate double `Shift`;
- exact restoration: `доллара ` -> `доллора `;
- GTK capture after Enter: `доллора`.

Measured facts:

- `ImeAutoUndo` authority tests: `2/2 PASS`;
- pending exact-undo tests in `lay-ibus-engine`: `2/2 PASS`;
- release build of `lay-ibus-engine` and `lay-test-input`: `PASS`;
- automated live GTK result: `got='доллора' expected='доллора'`;
- installed runtime: `lay-ibus-engine 0.2.324`;
- active runtime after proof: `lay-ime-ru`;
- global IBus and `lay-daemon` were not restarted.

Runtime evidence:

```text
ibus_space_autocorrect authorized
ime_committed_tail -> "доллара "
ime_committed_tail -> "доллора "
ibus_auto_undo restored_exact_original
double_shift_auto_undo handled=true
```

The runtime harness now explicitly starts IME GTK dialogs with
`GTK_IM_MODULE=ibus`; without it GTK4 may bypass IBus and produce a false
layout-only result. It also waits for the asynchronous double-Shift edit before
submitting Enter.

Not tested:

- the same action from the persistent physical keyboard in every supported
  browser and desktop toolkit;
- non-IME/uinput exact undo in this proof.

Verdict scope:

- generic IME autocorrect undo in the live GTK runtime: `PASS`;
- ordinary layout toggle remains the fallback only when no valid pending
  autocorrection exists;
- physical user confirmation remains the final environment check.

Receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/DOUBLE_SHIFT_AUTO_UNDO_TRIGGER_OWNERSHIP_2026-07-27.json`

### 2026-07-27 L2 known inflection preservation: PASS

The observed live regression was:

```text
в коде
-> L2FieldShadowReadout / extra-letter
-> в код
```

The L2 peak was strong, L3 was neutral, and the production IME hot snapshot did
not expose `коде` as an exact authoritative input center. The readout therefore
treated the case ending as signal loss it could remove.

The accepted generic contract is now:

```text
exact compact morphology form
or
preposition + Russian token + final-vowel deletion
-> require pairwise L3 or exact state proof
-> otherwise keep the observed surface
```

This is not a phrase replacement rule. It protects the class of Russian
preposition-governed inflections from destructive L2 autocorrection while
leaving the candidate available for contextual ranking.

Tested:

- proposal admission for `коде -> код`: `1/1 PASS`;
- final known-form decision admission: `1/1 PASS`;
- complete `L2FieldShadow` correction route for `в коде`: `1/1 PASS`;
- live GTK4 + IBus capture: `got='в коде' expected='в коде'`;
- protected double-Shift undo after the change:
  `got='доллора' expected='доллора'`.

Measured facts:

- installed `lay-ibus-engine`: `0.2.324`;
- installed `lay-daemon`: `0.2.324`;
- global IBus restart: not performed;
- runtime authority changed: yes, destructive L2 inflection deletion now
  requires independent context authority.

The proof also found that `pkill -x lay-ibus-engine` cannot reliably match this
16-character executable name because Linux `comm` is limited to 15 bytes. The
managed runtime helpers now terminate only `lay-ibus-engine` by its full
executable argv, preventing tests from silently running an old deleted inode.

Not tested:

- every Russian preposition and case combination;
- physical input in every supported browser and toolkit;
- the non-IME/uinput route.

Receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_KNOWN_INFLECTION_PRESERVATION_2026-07-27.json`

## Scoreboard

```text
correction_gate
|
+-- requests
+-- total_candidates
+-- apply_candidates
+-- suggest_only_candidates
+-- keep_original_candidates
+-- veto_candidates
+-- deterministic_candidates
+-- nanda_candidates
+-- selected_apply
+-- avg_us
+-- max_us

input_gate / recent_actions
|
+-- total_candidates
+-- apply/suggest/keep/veto split
+-- deterministic vs NANDA split
+-- selected source/error class
```

These metrics are status-only. They must not log raw user text.

## Debt Queue

```text
P0: keep all destructive text mutation behind text_edit::authorize_replacement
P1: keep source role decisions behind correction_source_contract
P2: split correction_core only by route, not by file size
P3: keep IME display isolated from correction ownership
P4: make candidate quality/latency regressions visible before release
```

### 2026-08-02 committed-tail capability preflight: PASS physical Chromium

Chromium advertises no IBus surrounding-text capability. Before this change,
the daemon still dispatched `ReplaceTailV4`; the IME then discovered that it
could not delete committed text and returned `false`. The mutation-monopoly
contract correctly treated that post-dispatch rejection as terminal, so the
authorized uinput backend could not run and `ghjdthrf` became `ghjdthr` after
Backspace.

The live route is now:

```text
focused IME state
-> CanReplaceCommittedTail(backspaces), no mutation
   -> true: dispatch ReplaceTailV4; every result is terminal
   -> false/error: NotDispatched; authorized backend reselection remains legal
-> exactly one physical mutation owner
```

This is a capability correction, not a second fallback after mutation. Once
`ReplaceTailV4` is dispatched, `Rejected` and `Indeterminate` still forbid a
second backend.

Tested and measured:

- engine capability profiles: `2/2 PASS`;
- mutation-monopoly contract: `1/1 PASS`;
- committed-tail focused tests: `18/18 PASS`;
- remote candidate build: `20` jobs, `134.32 s`, `316%` average CPU,
  `1,655,472 KiB` peak RSS, `0` swaps;
- physical Chromium: `ghjdthrf -> проверка -> Backspace -> проверк`;
- physical Kitty: `ghjdthrf -> проверка -> Backspace -> проверк`;
- global `ibus-daemon` PID remained `3702` through both physical tests.

Not tested: Telegram unsent-field mutation and every focus permutation. No
Telegram or WeChat message was sent. Runtime authority changed only at backend
selection before dispatch; text-decision authority did not change.

Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/LAY_1_0_PHYSICAL_APPLICATION_MATRIX_2026-08-02.json`.

`src/keyboard/event_words/decision.rs` is route-critical because it decides
manual replay layout, but it must remain outside candidate generation and text
replacement ownership.

### 2026-08-02 Phase 7 deterministic input admission: PASS_CODE

The product-gate work did not add word-specific replacement rules. It repaired
the evidence boundaries shared by layout projection, missing-letter recovery,
clean-surface preservation and hidden-state admission:

```text
observed tail
-> preserve the left whitespace anchor in a longer replacement
-> generate deterministic and L1.1/L2 candidates
-> preserve an independently certified clean input surface
-> admit an exact known layout projection
-> admit a verifier-proven deterministic typo repair
-> otherwise retain Tied/ABSTAIN/Keep
```

Initial-vowel recovery is class-gated. A strong exact/reference center is
eligible when the damaged signal begins with a doubled consonant or the target
is an adjective lemma. This restores `ффективная -> эффективная` and
`бычный -> обычный` while preserving the observed verb-like `лучшить`.

Measured facts:

```text
serial lay-daemon tests                  200/200 PASS
representative transposition sweep       487/497 = 97%
full gate wall                           245.60 s
full gate peak RSS                       353,900 KiB
test-order HotFieldPolicy leak           fixed
double-Shift pending undo contract       PASS in daemon gate
boundary clean false applies             0/220, was 3/220
boundary unambiguous proposal recall     185/188 = 98.4%
boundary conservative direct recovery    156/188
```

Tested:

- left-space retention for longer deterministic replacements;
- exact three-letter Cyrillic-to-known-English layout projection;
- generic short Cyrillic-to-ASCII protection;
- clean Russian and natural hyphen preservation;
- missing-initial-vowel positive and negative morphology classes;
- all daemon tests in one process and one deterministic order.

Not tested by this checkpoint:

- the fixed L1.1 `13 x 20,000` per-damage-class heldout proof;
- physical interaction in every WeChat, Telegram, Chromium, GTK, Qt and Kitty
  cell of the product matrix;
- a multi-day daemon residency test.

Verdict scope: `PASS_CODE`. Runtime authority changed: `yes`, only for an exact
known layout projection or an independently verifier-proven deterministic typo
repair. Clean observed state still vetoes destructive replacement.

Exact receipts:

```text
/tmp/lay-phase7-full-gate-6-serial.log
/home/ubu/projects/lay/docs/structural_gates/receipts/FINAL_PRODUCT_GATE_PHASE7_2026-08-02.json
```

### 2026-08-03 managed key release pairing: PASS_CODE, WATCH_WECHAT

Live trace inspection of the reported WeChat repeating-space failure found:

- one `space_managed_commit` per observed physical Space press;
- no consecutive Space press run in the last 5,000 IME records;
- repeated `focus_out -> focus_in` and capability changes `41 -> 9`;
- managed presses returned `handled=true`, while their matching releases were
  returned to the client as `handled=false`.

The event contract is now:

```text
managed key press
-> Lay handles press and commits/preedits text
-> remember physical keycode
-> matching release is consumed by Lay exactly once

terminal or command passthrough press
-> handled=false
-> matching release remains passthrough

focus/reset
-> clear unmatched managed-release ledger
```

This prevents WeChat and other clients from receiving an orphan key release
for a press already consumed by the IME. It does not synthesize releases,
change text authority, or intercept terminal/command passthrough.

Tested:

- managed press/release ownership unit test: `1/1 PASS`;
- protected WeChat Backspace/preedit contract: `1/1 PASS`;
- source-level trace evidence: no repeated Space press sequence.

Not tested:

- physical WeChat hold/release after installing the new engine;
- proof that orphan releases were the only source of the reported repeated
  spaces;
- every GTK/Qt/Chromium press/release permutation.

Verdict scope: `PASS_CODE`, `WATCH_WECHAT`. Runtime text-decision authority
changed: `false`. IBus event ownership changed only for the release paired with
an already handled managed press.

Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_MANAGED_KEY_RELEASE_PAIRING_2026-08-03.json`.

Installed state:

- release build: remote `20` jobs, `39.73 s`;
- installed `lay-ibus-engine 1.0.1` SHA-256:
  `b6d33b09cb866cf5f9007b06c5dce20fc1edf9781928bd5f1271fbef06789762`;
- engine PID: `3333338 -> 3350505`;
- active engine: `lay-ime-ru`;
- global `ibus-daemon` PID before/after: `3702/3702`.
