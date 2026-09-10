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

### 2026-08-03 active-layout preservation at Space: PASS_CODE

Live logs showed that both opposing edits were independently authorized:

```text
pdf -> зва    source=layout_then_known_word  proof=layout
зва -> pdf    source=layout_ru_to_en          proof=layout
```

The direction varied by window because the Space decision did not receive the
layout that produced the token. The canonical boundary contract is now:

```text
token matches active layout
+ token is independently known in that layout
+ proposed automatic transition has layout proof
-> preserve the token

unknown token in active layout
+ known opposite-layout projection
-> normal DecisionCore authority remains available

manual double Shift
-> unchanged; this preservation rule is Space-only
```

The rule is generic and contains no `pdf` or `зва` special case. It is owned by
`src/ime_correction.rs` and is used by both live mutation routes:

```text
lay-ibus-engine active/committed composition
lay-daemon typing_assist_ime fallback
```

Measured facts:

```text
active EN: pdf -> preserve                    PASS
active RU: зва -> pdf                         PASS
active RU: прохоил -> проходил                PASS
daemon active EN: pdf -> preserve             PASS
daemon active RU: зва -> pdf                  PASS
```

What was tested: five isolated route tests covering both directions, both
executors, and a non-layout typo control. What was not tested: physical entry
in every application window and the fixed L1.1 damage-class proof, because this
change does not alter L1.1 candidates or package data.

The broad parallel `ime_correction::tests` invocation produced `21/29 PASS`
with eight existing online-state/order-sensitive failures. Every new test
passed in that run and again in an isolated process; therefore this checkpoint
does not claim a clean broad-suite result.

Verdict scope: `PASS_CODE`, `WATCH_PHYSICAL`. Runtime authority changed: `yes`,
only to veto an automatic layout-proven edit when independent lexical evidence
and active-layout evidence both support preserving the original token.

Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_ACTIVE_LAYOUT_PRESERVATION_2026-08-03.json`.

Installed state:

```text
version                         1.0.2
lay-daemon PID                  3500657
lay-ibus-engine PID             3350505 -> 3504744
lay-ibus-engine SHA-256         35831e689b1fbbc79e33c51986ccb65274a31a9657be7af68d462f5d1f83cfa4
global ibus-daemon PID          3702 -> 3702
active engine                   lay-ime-ru
```

### 2026-08-03 imperative final-consonant repair: PASS_CODE

Observed failure:

```text
читайл -> no_decision
L1.1 lattice contained читай as extra-letter
deterministic final-letter safety removed that candidate before readout
```

The previous safety rule rejected every final-letter deletion. The accepted
class is now narrower than general final deletion:

```text
dirty input is not independently known
+ deleting exactly the final character yields a morphology-backed imperative
+ deleted character is a Cyrillic consonant
+ deterministic scorer has exactly one surviving known candidate
-> ExtraLetter candidate may enter DecisionCore
```

This is class-conditioned morphology evidence, not a `читайл` word rule.
Examples covered by the same route: `читайл -> читай`,
`сделайл -> сделай`. Known clean `читал` remains preserved.

Measured facts:

```text
final-consonant imperative candidates       2/2 PASS
known past-tense preservation               1/1 PASS
shared IME Space authorization              1/1 PASS
```

Not tested: a corpus-wide final-letter deletion sweep or the fixed L1.1
`13 x 20,000` proof. Runtime authority changed: `yes`, only for the constrained
extra-letter class above. Verdict scope: `PASS_CODE`, `WATCH_CLASS_SWEEP`.

Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_IMPERATIVE_FINAL_CONSONANT_REPAIR_2026-08-03.json`.

Installed verification:

```text
version                         1.0.3
remote build CPUs              20
remote release build           1 min 55 s + 47.18 s remaining bins
lay --explain-correct читайл    extra_letters -> читай, Eligible
lay-daemon PID                  3742331
lay-ibus-engine PID             3742363
global ibus-daemon PID          3702 -> 3702
active engine                   lay-ime-ru
```

### 2026-08-03 canonical L2 outage authority: PASS_CODE

Live-log failure:

```text
Пиши -> Приши     missing_letter
Нахуя -> Нахоя    vowel_confusion
```

The installed IBus process requested canonical L2, but its 12 ms L1.1 seed
request occasionally timed out. The empty readout was represented by the same
state as an intentionally unrequested field. A lone deterministic candidate
therefore retained `class_allows_apply`. A 40-process reproduction produced
three empty L1.1 readouts and one false automatic `Пиши -> Приши` decision.

The authority states are now distinct:

```text
canonical L2 not requested
-> deterministic-only authority is unchanged

canonical L2 requested and available
-> Winner | Tied | Abstain owns lexical authority

canonical L2 requested but unavailable
-> deterministic lexical typo is SuggestOnly
-> independent layout/boundary evidence remains eligible
```

This is a fail-closed field contract, not a word-specific rule. What was
tested: requested-unavailable demotion and deterministic-only preservation.
What was not tested yet: installed live IBus replay, fixed L1.1 damage-class
proof, and corpus-wide clean-form preservation. Runtime authority changed:
`yes`, only when canonical L2 was requested but unavailable.

Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_L2_UNAVAILABLE_FAIL_CLOSED_2026-08-03.json`.

The same live-log audit also found clean Russian forms that were absent from
the installed L1.1 corpus and from the bounded morphology recognizer:

```text
могли -> могил
скажу -> скажиу
китайцев -> китайев
Пиши -> Приши
```

Clean preservation now accepts only dictionary-attested lemma transforms:

```text
-гли/-кли -> -чь             могли -> мочь
-жу -> -зать/-дить/-деть     скажу -> сказать
imperative ш/с alternation    пиши -> писать
-йцев -> -ец                 китайцев -> китаец
```

These transforms mint no replacement candidate. They only certify the
observed surface so typo arbitration cannot silently rewrite it. Tests cover
all four observed forms and retain the previous regular imperative and noun
form checks. This is a bounded morphology extension; it is not a claim of
complete Russian morphology coverage.

Installed verification:

```text
version                         1.0.4
remote build CPUs              20
remote final build             116.14 s, peak RSS 1,659,444 KiB
Пиши / могли / скажу / китайцев preserve
Нахуя with unavailable L2      preserve
послдений -> последний         PASS
изночально -> изначально       PASS
читайл -> читай                PASS
lay-daemon PID                 4002297
lay-ibus-engine PID            4002253
global ibus-daemon PID         3702 -> 3702
active engine                  lay-ime-ru
```

### 2026-08-12 atomic uinput tap and stuck-key recovery: PASS_RUNTIME

The synthetic output owner previously submitted key-down and key-up as separate
`VirtualDevice::emit` calls. Each call publishes its own `SYN_REPORT`, so an
error or daemon exit between the calls could leave a compositor-visible key
pressed and trigger unbounded autorepeat.

The live output contract is now:

```text
plain tap
-> [key down, key up]
-> one VirtualDevice::emit
-> one SYN_REPORT

shifted tap
-> [Shift down, key down, key up, Shift up]
-> one VirtualDevice::emit
-> one SYN_REPORT

emit error | startup | SIGTERM | SIGINT | graceful shutdown
-> best-effort key-up frame for every key exposed by lay-virtual-keyboard
```

Replay, Backspace, Space, arrows, and grabbed-input forwarding use this single
emission owner. No word, phrase, source ID, or key-specific runtime exception
was added.

Tested and measured:

- focused frame and fault-injection tests: `2/2 PASS`;
- patched remote daemon suite: `199 PASS`, `6 FAIL`;
- unpatched baseline daemon suite: the same six tests failed, with no new
  failing test name in the patch;
- remote release build: `3 min 1.61 s`, peak RSS `2,382,752 KiB`, exit `0`;
- installed binary: `8,161,824 B`, SHA-256
  `e9e527b3e8c88ffa595be1eff8c8b64f43a82642cccd7c7faad368d9de584b59`;
- `lay-daemon.service`: active after replacement;
- global `ibus-daemon` PID remained `3702` and active engine remained
  `lay-ime-ru`;
- `ydotool.service`: inactive; installed `lay-virtual-keyboard` present;
- daemon journal after activation: no warning-or-higher entries.

Not tested:

- forced kernel/uinput write failure against the installed desktop session;
- physical reproduction of every supported key and every application;
- visual synthetic-text smoke while the user was actively typing.

Runtime authority changed: `no`. Candidate generation, correction admission,
and backend selection are unchanged. Only the already-authorized uinput
mutation is made closed per visible evdev frame, with fail-closed cleanup.

Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/LAY_UINPUT_STUCK_KEY_RECOVERY_2026-08-12.json`.

### 2026-09-01 correction-safety Apply authority: PASS_CODE_REVIEWED

The user-visible `strict` (`Осторожно`), `normal` (`Норма`), and
`experimental` (`Смелее`) profiles previously reached deterministic rule
filtering but were lost before the common non-exact Apply owner. The repaired
source route is:

```text
CorrectionSafety
-> CorrectionRequest
-> L2CandidateLattice
-> TransitionDecisionPolicy
-> candidate_has_apply_authority()
-> Apply | retained NoApply
```

`TransitionDecisionCore` remains the only automatic authorization owner. The
profile check executes after existing signal evaluation and structural/verifier
vetoes, and before the ranked candidate may enter the selectable Apply set. A
profile denial does not remove or rescore the candidate, change its surface,
rewrite producer evidence, or create an alternate mutation path. Closed exact
authority still bypasses ordinary profile tiering only through its existing
valid certificate route.

The policy counts a four-bit set of independent domains rather than raw
signals: `LEXICAL_FIELD`, `L3_DIRECTIONAL_PAIR`, `L4_EXACT_STATE`, and
`BOUNDARY_FIELD`. Correlated lexical alternatives collapse into one bit;
boundary evidence is mutually exclusive with lexical-field evidence. Typed
source/origin/error-class metadata and registered rule safety metadata assign
the candidate tier. No word, phrase, suffix, fixture ID, test name, or literal
source ID is used as runtime authority.

Tested and measured:

- fixed task-local corpus SHA-256
  `4f08436caaa040a44b052e1431196a0a4f5b852887b44b081306835d69be7aed`;
- `26/26` logical cases and `78/78` profile observations: `18` pure-policy
  and `60` routed-runtime observations;
- routed class denominators: missing-letter `6/6`, composite-typo `3/3`,
  letter-substitution `6/6`, boundary `9/9`, completion-suggest-only `3/3`,
  wrong-layout exact `9/9`, and clean/protected negatives `24/24` with
  `false_accepts=0`;
- Experimental compatibility observations `20/20`; closed-exact observations
  `9/9`; profile-differentiating routed observations `15/15`;
- focused TD-112 test functions `12/12`, policy tests `8/8`, pinned live
  preedit preservation `3/3`, and public `input_gate_space_contract` `6/6`;
- changed correctness/package lanes selected `2,388` tests with zero known
  semantic and zero infrastructure failures;
- observed-source code-route gate: `PASS`, 21 nodes, 33 edges, 27 named
  routes, 54/54 source markers, zero issues/warnings, and one authorization
  owner;
- post-edit implementation preflight: `READY_TO_IMPLEMENT`,
  `safe_to_implement=true`, zero blockers, manifest SHA-256
  `f03c1c842f2e52cc6a433ccc0fb89266cb82ddf35bd6557a9a71b28d7d2db052`;
- final reviewed architecture refresh: `21,328` nodes, `53,129` edges, `971`
  communities, `678` Rust sources, all 11 ownership checks `PASS`; its exact
  `611` zero-AST limitation and artifact bindings are preserved in the task
  receipt;
- final post-review full wrapper: exit `0`, `2,388` correctness/package checks,
  zero known semantic and infrastructure failures, lint/syntax/CLI/release
  checks `PASS`, and terminal line `== lay full check OK ==`;
- Cargo target usage after the final wrapper was
  `6,377,787,392 / 12,884,901,888` bytes.

The first historical wrapper attempt completed formatting and then stopped at a
stale architecture receipt before Cargo tests; its separately executed tail is
preserved as diagnostic evidence only. After review and the final architecture
refresh, a new wrapper invocation reran from the first step, exited `0`, and
printed its terminal success line. Only that final invocation owns the full-gate
acceptance claim.

Not tested by this checkpoint:

- a routed DecisionCore fixture that naturally supplies an exact-positive L4
  signed state; the pure extraction/deduplication contract for
  `L4_EXACT_STATE` is covered, but routed L4 contribution remains unproved;
- broad Russian/English language quality outside the fixed regression corpus;
- physical typing in a focused desktop application or visual comparison of all
  three profiles;
- a new isolated p99/RSS benchmark attributable only to this small policy
  helper.

Fresh-context code review V1 returned `ACCEPT`, `9.0/10`, with no High or
Medium findings. Its only Low finding was stale task-completion wording, closed
in the same documentation finalization. No production correction pass was
required.

Verdict scope: `PASS_SOURCE_FINAL`. Source Apply-authority semantics changed:
`yes`, only for ordinary non-exact automatic replacement under Strict and
Normal. Ranking, lattice retention, exact authority, verifier, SafetyGate,
edit-plan validation, and mutation ownership are unchanged. Installed runtime
authority changed: `no`; the installed 1.0.60 binaries, services,
configuration, and selected IME were not modified or restarted.

Task evidence:

- `tech_debt/evidence/td112-fixture-correction-v1.json`;
- `tech_debt/evidence/td112-red-green-tdd-v1.json`;
- `tech_debt/evidence/td112-code-route-observed-receipt-v1.json`;
- `tech_debt/evidence/td112-implementation-preflight-post-edit-receipt-v1.json`;
- `tech_debt/evidence/td112-code-review-v1.md`;
- `tech_debt/evidence/td112-verification-v1.json`;
- `tech_debt/evidence/td112-architecture-refresh-v1.md`.

### 2026-09-03 hybrid Nanda autocorrect restoration: PASS_FINAL_SOURCE_GATES_RELEASE_PENDING

The live `nanda_autocorrect=true` route previously selected `NandaOnly` and
therefore removed deterministic typo candidates before the common lattice and
DecisionCore could compare them. The repaired source-composition route is:

```text
nanda_autocorrect=false
-> DeterministicOnly

nanda_autocorrect=true
-> DeterministicAndNanda
-> deterministic candidates + Nanda candidates
-> one L2CandidateLattice
-> one TransitionDecisionCore rank and authorization pass
-> one event-specific edit plan and mutation backend

explicit diagnostics
-> NandaOnly remains available and isolated
```

The implementation reuses the existing candidate, verifier, rank,
authorization, edit-plan, and mutation owners. Identical unmerged
deterministic evidence may be reused within one request to avoid repeated
admission, but merged, morphology-bearing, or authority-bearing evidence is
never reused as the original producer record. The bounded L2 readout caches
immutable candidate material only; request-time authority remains outside the
cache.

The final boundary contract separates verified geometry from semantic Apply
authority:

```text
BoundaryShift + verifier proof
-> retains structural Apply authority

BoundaryMergeSplit reducing word count + verifier proof
-> retains merge authority
-> example: текст е -> тексте

BoundaryMergeSplit increasing word count + verifier proof
-> candidate remains in the lattice
-> Apply requires exact function-word split, measured context support,
   pairwise L3 authority, exact L4 authority, or target-bound BoundaryCell32
   evidence whose exact split reconstructs the current token
-> function-word and L2 target-grounded surface-lexical split authority share
   one complete exact-one-edit conflict predicate over all 33 lowercase
   Russian letters; any clean certified one-word competitor withholds only
   that surface authority while retaining the split candidate and independent
   context/L3/L4 authority
-> the L2 producer reports target-bound structural grounding only; the complete
   clean-one-word competitor predicate is consumed at DecisionCore Apply
   authority rather than erasing producer evidence
-> the clean-surface certificate validates morphology relations: true soft
   inflections cannot use incompatible hard lemmas; synthetic comparative
   `-ее` from `-ый` requires exact Hunspell `E` membership, while a regular
   non-velar `-ий` independently retains neuter `-ее`; `-ой`, unknown, or
   unavailable class data grants no comparative authority
-> bounded L2 top-k absence and candidate-material generation turnover never
   count as negative evidence
-> content-content and repaired splits without that evidence remain NoApply

boundary material cache hit
-> generation is captured before lookup
-> generation is read again after the hit
-> turnover before return yields empty material rather than stale candidates
```

This direction check prevents structural verification from being promoted
into a semantic certificate. It introduces no literal word, phrase, fixture,
test-name, or source-ID branch; the only added suffix condition is the general
comparative `-ее` relation. Verifier, SafetyGate, edit-plan validation, and
mutation ownership are unchanged.

The previously recorded focused/performance checkpoint preceded one final
repository-wide RED. Both wrappers found exactly three regressions: structural
evidence was erased for `тоесть -> то есть` and `когдая -> когда я`, while the
known state `точнее` published unrelated corrected-prefix replacements. The
systemic repair separates structure from Apply competition and comparative
`-ее` from true soft inflections.

Tested on the repaired source before restarting the final repository wrappers:

- compile-once gate: `PASS` in `26.705 s`;
- all three former repository reds: `3/3 PASS`;
- TD-113 unit filter: `23 PASS`, `0 FAIL`, `1` ignored;
- close boundary negatives `3/3`, clean-prefix negative `1/1`, live safety
  matrix `6/6`;
- executable runtime-source contract: `7/7 PASS`;
- observed-source code-route gate: `PASS`, 16 nodes, 41 edges, 15 routes,
  `57/57` source markers, zero issues and zero warnings;
- remote verification host: `e-MEGA-MINI-M1-13th`, 20 logical CPUs;
- regenerated test-lane manifest after the test-ownership correction: `2,467`
  total = `2,406` correctness + `36` package + `11` performance + `14`
  ignored, 36 targets, SHA-256
  `64511fec25873ab76fc43152787bacf5e673d0ae530751b0f82343fa3bc46e25`;
- first post-repair full attempt: architecture unit suites `22/22 + 2/2`
  passed, then the wrapper stopped on the expected stale source fingerprint;
  test lanes and performance were not run or claimed;
- architecture-refreshed full attempt: architecture unit suites `22/22 + 2/2`
  and all `11/11` named ownership checks passed, then the correctness/package
  lanes held at `2,439/2,441` on exactly two stale test-ownership assertions;
  log SHA-256
  `306edac33bd8419e63382f4c0dfe1950e023bc2279e96637f65f33cc5a8e3291`,
  unchanged `1,239`-file source closure
  `585fb665ea1e432f2f86519bb18e21e620ceed243ccbf017393371f0f71485bf`,
  unchanged Cargo target usage `2,884,943,872` bytes; performance and runtime
  smoke were not run;
- exact ownership contract V2 after the test-only correction: `6/6 PASS`; it
  proves that L2 retains exact target-bound structural evidence, genuinely
  unproven fragment splits remain ungrounded, and DecisionCore still withholds
  surface-only Apply over a clean single-token competitor; log SHA-256
  `8d5c099ce017ebad047b6368ce094809c058e71bd54077ae0fb71d78136865f6`;
- independent code review pass 2: `HOLD`, `7/10`, with one blocking High
  finding: the intermediate `-ее` branch treated every attested `-ый` lemma as
  gradable without independent Hunspell `E` evidence. The review limit is
  exhausted, so no third score is manufactured; objective closure is recorded
  separately from the historical review verdict.

Subsequent full-wrapper chronology preserves each terminal boundary:

- V3: `2,442/2,442` Rust correctness/package `PASS`, then default Clippy
  `HOLD` on `let_and_return` and `single_element_loop`; full-log SHA-256
  `017f7d3d4c583cbbf290925739c7c773f9426fb26cf7707e301fd4b6b262456d`;
- mechanical repair: default and research-tools Clippy `PASS`, and the exact
  contract remains `7/7 PASS`;
- V4: `2,441/2,442`, with every runtime test clean and only the brittle
  `td113_boundary_competitor_veto_is_complete_and_generation_independent`
  source-shape assertion failing; full-log SHA-256
  `d5a8c0be8c8b1e190e695442d188ff8a825bb16c23431dde2e640236b742f8e1`;
- behavior-bound contract repair: source SHA-256
  `32df43279fd1de46b9d0265f817a0394b5f15cf2ef4f63e19dfb5534f97c6942`,
  integration `7/7 PASS`, default and research-tools Clippy `PASS`;
- V5: `2,442/2,442` Rust correctness/package and lint `PASS`, then
  infrastructure `HOLD` with exit `127` at `node --check`; the installed
  `/home/e/.local/bin/node` reports `v24.18.1`, but the non-login `PATH` omitted
  that directory. No install or system mutation occurred, and performance did
  not run. Full-log SHA-256:
  `a7198aa7f50b35d5254d32e62818adc9405d57490d046e94db9af4018259b7cf`;
- V6: `2,442/2,442` Rust correctness/package, lint, Node, Python, and shell
  syntax `PASS`; the CLI exited `0` with `chosen: none` and no `confidence:`,
  so the proof `grep` exited `1` before the release build. The full wrapper
  exited `1`; duration and the full unchanged source-closure digest were not
  authoritatively captured. Full-log SHA-256:
  `898e9a69b505643bb73a76f5a2dca9724b494281b917238311b0577921671435`.

V6 exposed a proof-owner defect: a repository assertion inherited mutable
per-user `HOME` configuration. The repair options were:

| Option | Score | Decision |
|---|---:|---|
| Repository-owned proof config | **9/10** | Selected; hermetic and does not change CLI semantics or user state. |
| Change CLI default or semantics | 5/10 | Rejected; product semantics must not be changed to satisfy the gate. |
| Provision remote `HOME` config | 2/10 | Rejected; retains a machine-specific hidden dependency. |

The full-gate script now binds only the CLI smoke to
`scripts/proof/autocorrect-proof-config.json`; script SHA-256
`ba6ddf988f26796847ca078b0797e93c78bec945fc67e88a724c189b85875214`.
An empty-`HOME` preflight passed with log SHA-256
`cb316767842bdd9dd843127c57de1c602418a447ac013f11d92dbeaddc1fff9c`.
No runtime CLI semantics or remote user configuration changed.

Strict full V7 then exited `0` in `459 s`. Architecture, `2,442/2,442` Rust
correctness/package tests, lint, Node/Python/shell syntax, the hermetic CLI
smoke, release build, and `git diff --check` passed. The source closure was
unchanged at
`edc6f8d9ba4836e04a6f388ac7bf0a2cff7bd861137fdabcdc8ac898f3c5c2f7`;
target usage was `3,540,627,813` bytes before and after. Full-log SHA-256:
`b0f8b41e05e09a97bfebf22c304a54945003514fe37e90e034243ffc4d49f816`.

V7 and any performance evidence bound to its source closure are historical
pre-morphology checkpoints. The next fixed proof found that adjective
certificate ownership was still too coarse:

```text
surface suffix family
-> attested lemma ending
-> velar / sibilant / restricted stem class
-> clean morphology certificate
```

The prior binary soft/hard classification rejected valid mixed-spelling
paradigms, first `русский -> русского`. The exact test
`adjective_surface_certificate_respects_lemma_paradigm_and_stem_spelling`
failed with exit `101`.

| Option | Score | Decision |
|---|---:|---|
| Typed suffix-family × lemma-ending × stem-class mapping | **9/10** | Selected; represents the paradigm and preserves negative spellings. |
| Broad soft/hard expansion | 4/10 | Rejected; admits invalid cross-paradigm forms. |
| Literal exceptions | 1/10 | Rejected; fixture surfaces cannot become runtime authority. |

Preflight V26 returned `BLOCKED_BEFORE_CODE`, `safe_to_implement=false`, on the
unknown preservation reference `morphology-form-owner`. V27 pinned that owner
and returned `READY_TO_IMPLEMENT`, `safe_to_implement=true`, with zero blockers.
The typed repair is bound to source SHA-256
`555dfe8f8f2c25f90b12bf968f522d99d0d4b3628f90951b526a9d63ebde10a4`
and test SHA-256
`72a692fc32dbf6ba81d6077659c9a33e88af19843e741fef2851871274f2bc07`.

Focused results are exact regression `1/1 PASS` (log
`942e33929735983081f7b31e0bb81b583b6f16fc1662eb121b086357d4fa321b`),
lexicon `14/14 PASS` (log
`866ef0b10f9824a88c06a0c7ed4c05837bff571c6f8b59339e6f7588b9dd81ae`),
DecisionCore `33/33 PASS` (log
`db49b48f40d227f608583f47127a94c78ff0d78b4ed7cf3f19ac66cc535c4656`),
and executable source contract `7/7 PASS` (log
`caaec3644a06b4419bda1f40206575d255898e6e9f6202c856b8a30d2795db29`).
These focused results change morphology certification but grant no final full,
performance, Graphify, installation, or release authority.

The final review-pass-2 High concerns false comparative certainty, not the
mixed-spelling paradigm above. Its RED matrix returned `0/1`, exit `101`, first
on `почтовее`, and also covers `атомнее`, `даннее`, and `школьнее`; `точнее`,
`синее`, and `хорошее` are required preserved positives. RED log SHA-256:
`fe866cc16259d63b3e7655636553c1349f77418a79c6611392eddb91aec2b99c`.

V28 remains historical. Append-only V29 returned `READY_TO_IMPLEMENT`,
`safe_to_implement=true`, zero blockers, with canonical manifest SHA-256
`0267f17d184a3cda741f69d9fc012924175deec28a957382b50c103450b2f3c9`.
The admitted route is:

```text
Hunspell dictionary word/flags
-> A: regular -ий projection
-> O: possessive -ий projection
-> E: exact comparative-capable lemma projection, no -ий filter
-> Russian adjective class cache
-> russian_adjective_has_comparative_ee()
-> -ый + ее requires E

regular non-velar -ий + ее
-> existing regular-A neuter certificate

missing E / unknown lemma / loader failure
-> false; no comparative certificate
```

Final source SHA-256 values are
`2c982e16a21544232779075fbbb42ca8401929680768df2b35143603161eef23`
for `forms/backed.rs`,
`cd8c2cd53a06345e49c7a7c7d3782469adc40eb087a469ee88dc2706b720db78`
for `russian_lexicon_tests.rs`,
`659cc4a059c128c28a01643fa9a9b91f95800b4a3574fb3dbbcc930ce02939e7`
for the facade, and
`aefc384c73936d83d27f88003ac0e7880f465a059c26737999d912b036b53eed`
for the Hunspell loader.

Focused GREEN is exact regression `1/1` (log
`b840fc182a7dc47ea85b360b207fb497b0e674a904095e87dca53afbfd956f48`),
Hunspell `2/2` (log
`c65e4cbc0b9b134a0b5116d6f34c275c1240da04652f50197fd923f08e4eb789`),
Russian lexicon `17/17` (log
`98dbe0d1f350d2d9dcea6c9955f4090351123af36acd85387ec86dc5ac52abdd`),
DecisionCore `33/33` (log
`eafff142f5e241877ff14b0d738c6564c08edd2b3d6400e1876622113c795284`),
and source contract `7/7` (log
`fdd1bb02218e0d9c6c3a26d0de8470a697727dc638eab66ea50de71f229c122e`).
Clippy exited `0` with zero non-dead-code diagnostics and `366` dead-code
warnings; log SHA-256
`0a3bd17532d6d87b7cdb05e095f6dc684d091262cb29cfd589155d9eceaa10be`.

Cross-host projection parity is `PASS` after identical `awk` word/flag
projection, `LC_ALL=C sort -u`, and newline-terminated hashing. Regular
`A -ий` is `24,723` rows with SHA-256
`4d734e5fd62e10a2817246fdb76bb4a12ad67f740da0d524a93d9bb9309b8693`;
possessive `O -ий` is `170` rows with SHA-256
`101d65585a282daac494a3bc0216c5e240bd0d28d1eb4761e87983ff44c85f3f`;
all `E` is `1,054` rows with SHA-256
`dc3f1550cbb9f4856801e74cd9d1b684f2479522f8c5c101bdd633e2e723c102`.
The full dictionaries intentionally differ and are outside this parity claim.
Receipt:
`tech_debt/evidence/td113-hunspell-projection-parity-v1.json`.

The current manifest contains `2,474` tests: `2,413` correctness, `36`
package, `11` performance, and `14` ignored across 36 targets; SHA-256
`bd38e606f68117929f491465fba0b377eb19a9ad676bc0a5bafa6ffc221f85aa`.
This conjunction objectively closes the High finding in focused scope. It does
not rewrite the historical `7/10 HOLD` or grant final repository authority.

Final-byte performance used unchanged source closure
`0062b6b55c6e0f9ec6d86619ea754d6693ab12ff4d25e626a259fbb40191aae9`.
Canonical V30 completed `500/500` samples, exit `0`, with p50/p90/p99/max
`1,841/1,944/2,971/3,701 us`; log SHA-256
`a5705197d3db6e607ecd2a968c3c359f8bd383b57cae9786d8096ac7c9c24f8d`.
The earlier `500`-requested/`120`-actual harness run is retained as
`NOT_FINAL`, log SHA-256
`1f7085954bf4460931d18495ff1b32ed5766b75b17dc37e042a03054d7b64a69`.
The final paired release proof passed `60` samples per mode: Nanda p50/p99
`97/446 us`, CPU `286 us`; Hybrid p50/p99 `804/1,692 us`, CPU `1,170 us`;
RSS `246,280 -> 246,284 KiB`, delta `4 KiB`. It exited `0` in about `286 s`
including LTO; log SHA-256
`8e928454ccf4c6cba82b755d123a63f6a741b1cc948130a05262a2a4b62ed2b7`.
Verdict is `PASS_FINAL_BYTES` for performance only.

The historical `2,439/2,441` result is a repository `HOLD`, not an architecture
failure and not a quality PASS. Its two tests encoded the obsolete ownership
assumption that semantic whole-word competition must erase producer evidence;
they did not expose a runtime behavior failure. The focused `6/6` result closes
only those assertion contracts. V7 proved its exact pre-morphology source
closure, but the later typed morphology repair requires every final-byte gate
to run again.

| Scope | Current evidence boundary |
|---|---|
| Earlier morphology focused proof | `1/1`, `14/14`, `33/33`, `7/7 PASS` |
| E-comparative focused proof | `1/1`, `2/2`, `17/17`, `33/33`, `7/7`, Clippy `PASS` |
| Hunspell A/O/E cross-host projection | exact counts and newline-normalized SHA-256 parity `PASS`; full dictionaries differ |
| Test inventory | current V30 bytes: `2,474` total, manifest SHA-256 `bd38e606...f85aa` |
| Runtime correctness/package | final source closure `2,449/2,449 PASS` |
| Lint, syntax, CLI smoke, release build, diff | final full wrapper `PASS` |
| Changed/full repository wrappers | final `PASS`; log SHA-256 `2819e85a...8a199` / `75966f28...41a` |
| Performance | final-byte canonical `500/500` and paired `60/mode` `PASS`; source closure unchanged |
| Observed-source route | fresh final-byte rerun `PASS`: 15 routes, 16 nodes, 41 edges, `57/57` markers, zero issues/warnings; one rank/authorization/mutation owner per event |
| Graphify/architecture freshness | final frozen-byte wrapper `PASS`: 21,559 nodes, 53,638 links, 1,001 communities, 678 bound Rust sources, `11/11` ownership checks and `22/22 + 2/2` deterministic suites |
| Release/install/live authority | `false`; 1.0.62 not built or installed |

The 1.0.62 build, rollback-safe installation, and live runtime checks remain separate
authority and are recorded only when executed. This checkpoint does not prove broad
Russian/English quality, physical focused-application typing, long-window
learning quality, or visual IME stability. IME suggestion flicker is explicitly
outside TD-113.

Verdict scope at this checkpoint:

`PASS_FINAL_SOURCE_GATES_RELEASE_PENDING`.

Source composition and bounded boundary Apply authority changed: `yes`.
Objective post-review repair tests are tracked separately from the historical
review score. Installed runtime authority changed: `no`; installed 1.0.61
binaries, services, configuration, and selected IME have not been modified by
TD-113 yet.

### V30: target validity is not input-damage authority

The post-V29 changed gate exposed one shared pre-existing fail-open mechanism,
not an adjective-classification regression. After strict Hunspell A/O/E
classification correctly stopped treating malformed broad-L2 centers as clean
morphology, `руских -> русских` lost an accidental preserve-current veto. A
clean replacement, `MissingLetter` geometry, and center support then fell
through to `class_allows_apply`; none of those observations independently
proves that the input omitted a repeated consonant.

Three general repairs were evaluated. A preservation-only permissive
morphology predicate scored `6/10` because it would again couple safety to
noisy broad-L2 membership. A new typed positive input-damage certificate scored
`8/10` architecturally but `5/10` for this release because it widens the
authority model. The selected `9/10` repair is one source-neutral transition
predicate: a one-letter insertion that duplicates an adjacent Russian
consonant remains proposal material but cannot receive Apply without separate
positive damage evidence. Literal exceptions and global center-authority
removal scored `1/10` and `3/10` respectively and were rejected.

The predicate has two consumers and adds no producer, rank, verifier, or
mutation owner:

```text
direct correct_missing_letter
-> build the complete authoritative candidate pool
-> preserve the existing exactly-one-candidate ambiguity gate
-> lower a sole duplicated-consonant insertion to no direct autocorrect

hybrid deterministic proposal
-> retain the candidate in the bounded lattice
-> classify the same geometry as unproven_stable_surface_shape_drift
-> SuggestOnly before DecisionCore Apply
```

The ordering is contractual. An initial implementation filtered the duplicate
candidate before the uniqueness check; fresh review found that this could turn
the ambiguous `балон -> {балкон, баллон}` pool into a false singleton. The
repair now checks the full pool first. Fresh-context review after that change
returned `ACCEPT 9/10`, with zero blockers.

The design route gate passed with `9` nodes, `14` edges, and `4` routes.
Implementation preflight V30 returned `READY_TO_IMPLEMENT`,
`safe_to_implement=true`, zero blockers. Remote TDD used
`CARGO_BUILD_JOBS=20`: the bound RED was `0/2`; final focused results were the
ambiguity regression `1/1`, consonant matrix `9/9`, nonduplicate controls
`2/2`, composite regression `1/1`, and the earlier TD-113 filter `25/25` with
one ignored performance proof. Guarded format and diff checks passed. The
current inventory is `2,474 = 2,413 + 36 + 11 + 14`.

Final changed/full wrappers and final-byte performance passed against unchanged
source closure
`0062b6b55c6e0f9ec6d86619ea754d6693ab12ff4d25e626a259fbb40191aae9`.
Not tested at this checkpoint: physical focused-application typing, IME visual
stability, or the 1.0.62 build/install/rollback/live route. Installed runtime
authority remains unchanged at 1.0.61. Exact receipt:
`tech_debt/evidence/td113-missing-duplicate-authority-repair-v1.json`.

Task evidence:

- `tech_debt/evidence/td113-red-green-tdd-v1.json`;
- `tech_debt/evidence/td113-performance-v1.json`;
- `tech_debt/evidence/td113-code-route-observed-receipt-v1.json`;
- `tech_debt/evidence/td113-boundary-authority-implementation-preflight-receipt-v22.json`;
- `tech_debt/evidence/td113-final-review-repair-implementation-preflight-receipt-v24.json`;
- `tech_debt/evidence/td113-morphology-certificate-implementation-preflight-receipt-v25.json`;
- `tech_debt/evidence/td113-adjective-paradigm-implementation-preflight-receipt-v26.json`;
- `tech_debt/evidence/td113-adjective-paradigm-implementation-preflight-receipt-v27.json`;
- `tech_debt/evidence/td113-adjective-classification-review-repair-implementation-preflight-v28.json`;
- `tech_debt/evidence/td113-adjective-classification-review-repair-implementation-preflight-receipt-v28.json`;
- `tech_debt/evidence/td113-e-comparative-review-repair-implementation-preflight-v29.json`;
- `tech_debt/evidence/td113-e-comparative-review-repair-implementation-preflight-receipt-v29.json`;
- `tech_debt/evidence/td113-missing-duplicate-authority-code-route-design-v1.json`;
- `tech_debt/evidence/td113-missing-duplicate-authority-code-route-design-receipt-v1.json`;
- `tech_debt/evidence/td113-missing-duplicate-authority-implementation-preflight-v30.json`;
- `tech_debt/evidence/td113-missing-duplicate-authority-implementation-preflight-receipt-v30.json`;
- `tech_debt/evidence/td113-missing-duplicate-authority-repair-v1.json`;
- `tech_debt/evidence/td113-architecture-refresh-v2.md`;
- `tech_debt/evidence/td113-hunspell-projection-parity-v1.json`;
- `tech_debt/evidence/td113-code-review-v1.md`;
- `tech_debt/evidence/td113-verification-v1.json`;
- `tech_debt/evidence/td113-architecture-refresh-v1.md`;
- `tech_debt/evidence/td113-repository-gate-red-v1.json`.
