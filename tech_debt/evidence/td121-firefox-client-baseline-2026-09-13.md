# TD-121 Firefox client baseline — 2026-09-13

The installed C20 executable bytes reproduce a Firefox failure in an isolated
textarea: **1/3** original cases pass with synchronous IBus input; **0/3** pass
with the user's observed Firefox input environment. This is synthetic client-visible
evidence. The user's earlier report remains the human keyboard evidence:
Double Shift works elsewhere, but rapid repetition in Firefox fails and then
stops working. TD-121 remains open.

| Original case | Visible result | Required / observed toggles | Verdict |
| --- | --- | --- | --- |
| `ghbdtn_fast_lshift_enter` | `привет` | 1 / 1 | PASS |
| `ghbdtn_extra_lshift_enter` | `ghbdtn` | 2 / 0 | FAIL despite matching final text |
| `ime_autocomplete_then_double_shift_enter` | `про`, expected `ghjdthrf` | 1 / 0 | FAIL; autocomplete acceptance was not reached |

Four fast taps must cause two transformations; unchanged visible text alone
cannot distinguish two transformations from zero. The existing trace assertion
caught that difference. The autocomplete case has an earlier failed precondition
and does not establish a defect in replay of an already accepted completion.

## Exact evidence and ownership

Evidence root:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-c20-baseline`.
Its `RECEIPT.json` SHA-256 is
`7f0d5702e1da7c4733c227d38e0e2b2276fd790f35df2e4c51bf364341ed9be2`:
`all_passed=false`, `fatal_error=null`,
`desktop_restoration_verified=true`. Every case has `FIREFOX-OWNER.json`,
`FIREFOX-DOM-EVENTS.json`, `OWNER-IDENTITY.json` and the bounded IME trace.

The external adapter reused the accepted C20 process/IME-owner wrapper with
SHA-256 `7429ef711f8bad5b0604472a7e7a11fd5f0ccaa44790004e0800f4aad744a403`.
It replaced only the dialog command with a fresh Firefox profile and a
loopback-only page. Before sender creation it checked the focused DOM textarea,
the nonce-bearing GNOME window, Firefox executable, exact profile argument,
PID/start identity and process group. Each managed IME PID matched its D-Bus
bridge owner; each daemon's loaded hash matched C20 and used only its owned
synthetic input device `/dev/input/event21`.

The accepted GUI wrapper forces `IBUS_ENABLE_SYNC_MODE=1`, which this first
Firefox process inherited. A subsequent allowlisted read of the user's Firefox
environment found `GDK_BACKEND=wayland`, `GTK_IM_MODULE=ibus`, and no
`IBUS_ENABLE_SYNC_MODE` or `MOZ_ENABLE_WAYLAND`. Thus the first run is a valid
Firefox synchronous-input baseline but does not reproduce the user's complete
input environment. A separate comparison changes only those Firefox-child
environment values, preserving the same C20 binaries and owned harness; its
result must be recorded separately.

External helper files beside the evidence root:

- `firefox-entry-capture.py` SHA-256
  `eaa47ca791b9c92f784ea7ec9870c4f495190508ebfd95ac65a575d94958bbdf`.
- `run-firefox-smoke-owned.py` SHA-256
  `749bace5d8a1040d6c34c616ce5a9c944dabacc9180f7ba9e023ff990e862ab7`.
- `firefox-smoke-preflight.md` records the isolation, focus and cleanup boundary.

All three browser processes were captured and terminated; a subsequent process
inspection found zero remaining Firefox processes with those owned profiles.
The user's Firefox PID 3124490/start 83088446 remained live. Global IBus remained
PID 4715/start 2261. The harness restored C20 IME PID 2382946/start 91621663 and
daemon PID 2382907/start 91621654; root independently checked loaded hashes
against installed bytes and selected engine `lay-ime-ru`. Installed executable
bytes did not change. Temporary managed IME/daemon ownership changed during
the test and was restored; process identities are therefore new.

## First observed loss in the four-tap case

The 178-row trace is
`ghbdtn_extra_lshift_enter-8dbd8632f59ee8a7f82b/ibus_engine_debug.jsonl`,
SHA-256 `ff38f7916c33f3fda1ff4c0c8c40c5916cfbc8469b2873790c1459cfb547147d`.
Zero-based rows 42–43 publish source-free owner 2; rows 44–45 observe
`FocusOut` serial 29 and `FocusIn` serial 30 before row 46 installs that outcome.
The callbacks then run at rows 47–50. Request 3 arms and emits its marker at
rows 51–52. No later publication is recorded. All 20 legacy key callbacks are
refused admission; no callback settlement or reset re-receipt is recorded.
Both physical trigger pairs reach the daemon FSM but fail with
`context admission denied; identity-free cancellation forbidden`.

The page remained focused from readiness at monotonic 916195.615 seconds to
completion at 916201.521 seconds; the first key arrived at 916200.135 seconds.
That interval does not prove acquisition met its original deadline: production
`ACQUISITION_BUDGET` is 5 ms. Marker emission alone also does not prove timely
marker observation. The unresolved discriminator is ordered acquisition state,
including callback/seal order, context reply, profile and deadline.

This is distinct from the earlier 2,727-row physical-session trace that lost an
advanced exact SurroundingText receipt as `second_surrounding_receipt`. The C1
source repair of that mechanism has its own RED/GREEN and C2 focused evidence;
it has not yet been exercised in this Firefox client. Do not attribute the
four-tap admission failure to that repair or claim it resolved by the C2 PASS.

## Comparison with the user's input environment

The separate `firefox-c20-user-env/RECEIPT.json`, adjacent to the first evidence
root, has SHA-256
`8a5f6f0d5b0ef3a7a4350b7b4fdc3b3c4cba8d19f775e074de1c91485c09f381`.
The same three C20 cases pass **0/3**, with no fatal helper error and verified
desktop restoration. Fast and Extra both leave `ghbdtn` with zero toggles;
the autocomplete case again stops before acceptance and leaves `про`.

The new child adapter is `firefox-entry-capture-user-env.py`, launched by
`run-firefox-smoke-user-env-owned.py` SHA-256
`0179d4c1488922ce641b7f0c73149c63e1b7611ebda2aed034e531c4607a8201`.
It imports the unchanged original capture helper. Actual Firefox `/proc`
environment checks before readiness and sender creation verify Wayland,
`GTK_IM_MODULE=ibus`, absent `IBUS_ENABLE_SYNC_MODE`, and absent
`MOZ_ENABLE_WAYLAND`; the fixed allowlist is embedded in each browser identity
receipt. These are observed input settings, not a claim that the new profile
duplicates the user's browsing state.

The Extra trace has 202 rows, SHA-256
`0d42a6000342c03d30dc6ecb65b7b954ecf7812b115528447b39947727b5614d`,
at `ghbdtn_extra_lshift_enter-28bcd15e7765d7040d10/ibus_engine_debug.jsonl`.
All 20 key callbacks are refused; request 4 emits a marker without a recorded
publication. This preserves the earlier acquisition failure under the matched
input environment.

The Fast trace has 195 rows, SHA-256
`72297a544d831bad661ca8faa7c1d8d603e0aa7b5fe61b53620afb8961a03d8b`,
at `ghbdtn_fast_lshift_enter-89c9e61c9516dd2f6aa5/ibus_engine_debug.jsonl`.
All 16 key callbacks are accepted. Six reset receipts are armed and confirmed,
ending with the complete six-character word; all four Shift callbacks are
accepted and settled. Nevertheless the daemon receives
`context admission denied; identity-free cancellation forbidden` from the
manual bridge route. The order of these records alone does not prove every
callback had settled when the bridge marker was processed. The missing evidence
is the exact bridge refusal phase and decision-time state.

All three new browser processes were captured and cleaned up; root found no
remaining Firefox process with an owned test profile. Global IBus and the
user's Firefox retain their original PID/start identities. The restored IME is
PID 2484947/start 91729386, with the same installed C20 executable hash. The
first comparison's restored PID recorded above is historical.

## Scope and remaining evidence

### C1 plus diagnostics client run — 2026-09-13, 11:09 UTC

The inactive diagnostic IME was exercised with the unchanged C20 daemon/input
binaries and the same Firefox child environment. This run passes **1/3**:

| Case | Visible result | Observed / required toggles | Result |
| --- | --- | --- | --- |
| Fast pair | `привет` | 1 / 1 | PASS |
| Four fast taps | `привет` | 1 / 2 | FAIL after the first successful replay |
| Autocomplete then pair | `про` | 0 / 1 | FAIL before autocomplete acceptance |

Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-c1-diagnostics/RECEIPT.json`,
SHA-256 `47a9dfe427cf4ba4ba5856d0f7b129433ec3e02c024a1a011550a9e8c81f93c4`.
It records `fatal_error=null` and `desktop_restoration_verified=true`.
All three IME PIDs match their D-Bus bridge owners and the candidate hash
`5f94d028d197a49c76d7dfd7eac034471ace0cee78762aa6b9da44553b993956`.
Candidate build binding:
`/home/ubu/.cache/lay/development/td121-diagnostics-candidate-20260913/artifact-binding.json`,
SHA-256 `aab59360949d7bcd534af9c8724a891c8acb7e04ce922176d562b85f5a4ed2b1`.
Root verified all referenced artifact/log hashes and identical local/remote
706-row build manifests. The later documentation refresh changes only the
generated architecture receipt among those source rows; the artifact retains
its explicitly bound build snapshot. This is a diagnostic build, not release
acceptance or an installation.

The Extra trace is
`firefox-c1-diagnostics/ghbdtn_extra_lshift_enter-23ca93df1573055b3d70/ibus_engine_debug.jsonl`
under the same external evidence parent, SHA-256
`777af35583541724d8c03d2bfa93998963f2ee753194d21bc84497cd02e56f33`.
Its 366 rows show source-free admission succeeding, the first six-character
manual delegation, and target owner 4 receiving a successful Transfer grant.
All six deletion presses and six replacement presses take `exact_replay_native`;
the mirrored tail ends as `привет`. The daemon then times out while waiting for
the queued second pair's exact committed tail: `passive:unknown-context`, target
epoch 21. There is no second transformation.

During deletion, Reset serial 82 arms a three-character receipt at epoch 12.
The subsequent client snapshot contains five characters and correctly fails the
exact match. After the remaining deletions, Reset 90 has no predecessor; the
client supplies an empty snapshot before replacement glyphs arrive. No final
six-character snapshot is recorded before the queued timeout. Existing replay
fixtures supply a fresh exact snapshot after every native press, which does not
exercise this coalesced Firefox callback schedule.

The browser's final field value independently confirms `привет`; all captured
field events retain focus. Event POST arrival order is not a causal ordering of
browser input events and is not used as such. No admission-diagnostic record is
emitted in any of these three traces: the earlier acquisition/bridge failures
did not recur in this run and are **not proved fixed**. The new
`advanced_confirmed` branch is also not observed here; its causal regression
proof remains separate.

Root verified cleanup of all three owned Firefox processes, restoration of
installed C20 IME PID 2655976 and daemon PID 2655961 with matching executable
hashes, and selected engine `lay-ime-ru`. Global IBus PID 4715/start 2261,
the user's Firefox PID 3124490/start 83088446, and the existing L1.1/L3 process
identities are preserved. Installed executable bytes remain unchanged.

Next discriminator: replay the actual Reset/coalesced-SurroundingText schedule
through the existing native replay observer and locate the first authority loss.
Do not substitute an expected mirrored tail for an independently confirmed
visible postcondition or weaken mismatching-snapshot rejection.

Before the diagnostics candidate, the controlled adapter ordering passed for prompt same-context
Transfer and different-context SourceFree admission, with the original request
and nonce preserved. Its separate forced-expiry check proves cleanup semantics,
not actual wall-clock timeout. It does not reproduce the client refusal. Root
independently verified its source archive SHA-256
`cbb7ee1904ae95a46d08503540daf0a6d4e5741f9b4f14250c33730f015b265c`
and receipt
`/home/ubu/.cache/lay/development/run-01_htg7j/RESULT.json`, SHA-256
`5c243730ae16f5af7a9dad4bb83df891f82f0632ab3378c16a3d96a2af388d83`.
The exact selected and executed identities match: **519/519** correctness and
package tests pass from 522 discovered, with three performance tests excluded.
Remote summary:
`/home/e/projects/lay-development-runner/run-ZQsZs8/tests/SUMMARY.json`, SHA-256
`dcbb3cfabb558b192e799e5399ad0886196a55c8f32c7c2bf851eb67e110c289`.
The Cargo manifest, lockfile, confirmed-append source and regression fixture
match this frozen archive; the subsequent diagnostic changes require their own
verification.

The unresolved acquisition, bridge and replay observations remain separate.
Preserve the single trigger owner,
exact context/request identity, original deadlines and refusal of stale word
authority. After a justified repair, rerun the affected Firefox cases on the
exact candidate bytes, retain GTK and terminal denominators, then complete the
mandatory release and human keyboard gates. General Wave answer quality and
fixed heldout restoration percentages remain unmeasured by this experiment.

### Browser-side logging discriminator — 2026-09-13, 11:28 UTC

One preregistered Extra case used the same frozen diagnostic IME with
`MOZ_LOG=timestamp,IMEHandler:4`. It failed **0/2** toggles before replay began,
so it cannot establish the post-replay notification order. Receipt:
`/home/ubu/.cache/lay/development/td121-firefox-fast-repeat-20260913-t__qoo7d/firefox-c1-native-log/RECEIPT.json`,
SHA-256 `a2a8cc63bee751bb06d4418ce60d59f246af5f630c2a05c54b0657a87e185ef8`.
Restoration is verified and no fatal helper error occurred. The IME trace has
211 rows, SHA-256
`219fe138ad85188169dafc2cc3210253672cccbe0bafe32e5b3d0c90eeee4b35`.
Reset initially arms a two-character receipt, then rejects a stale mismatching
snapshot; later Reset callbacks cannot rearm the missing predecessor. This is
an earlier loss than the previously observed queued replay timeout.

The case directory is
`firefox-c1-native-log/ghbdtn_extra_lshift_enter-0301926a399f3d83c142`
under the external evidence parent. Its `firefox-process.log` contains 598 lines,
97,809 bytes, SHA-256
`8818620564c767c1d9be21f6002469967956128eb05d0c3a4b65ac51a4cebadc`.
Five selection notifications with explicit offsets 1, 3, 4, 5, 6 report the
retrieved-surrounding flag set; surrounding retrieval also occurs on the later
Shift events. No post-replay conclusion follows because no replay occurred.
Browser logging can perturb timing; this run proves an observed ordering, not
uninstrumented latency. Root verified exact candidate/bridge PID ownership,
owned browser cleanup, restored C20 IME PID 2758168/hash, and preservation of
global IBus and the user's Firefox process identities.

The installed Firefox metadata identifies version 155.0.1, build
20260904061719 and source revision
`5fdfd0092780e85643e2cddc0e1b590c8b9ef860`. Its exact
[GTK client source](https://hg.mozilla.org/releases/mozilla-release/raw-file/5fdfd0092780e85643e2cddc0e1b590c8b9ef860/widget/gtk/IMContextWrapper.cpp)
was inspected alongside
[upstream IBus 1.5.29 GTK input handling](https://github.com/ibus/ibus/blob/1.5.29/client/gtk2/ibusimcontext.c).
The loaded Snap libraries identify the IBus 1.5.29 family; equivalence to every
distribution source patch is not proved. These sources explain pre-key retrieval
and the one-time RequireSurroundingText callback, but do not justify an
unverified client patch or promotion of mirrored text to observed authority.
Cached source files and hashes are in the sibling `source-evidence/SOURCES.json`.

### Initial Reset loss: bounded causal analysis — 2026-09-13

The 211-row native-log trace fixes the first loss independently of queued
replay. Rows 62/76 commit two observed characters; Reset 36 arms their
two-character predecessor at row 87. The next surrounding callback reports
one character with cursor and anchor at one (row 90), and the pending witness
is cleared at row 89. The next accepted printable extends the retained engine
tail to three characters (row 96), but Reset 40 has no capturable predecessor
(row 107). The subsequent three-character surrounding callback (row 110)
cannot recover it. No replay or GUI edit was attempted in this sequence.
The trace records surrounding lengths, not its text: the interpretation as a
delayed prefix is consistent with the browser source and input order, but its
exact text identity is not independently recorded by that metadata trace.

The current source explains the persistent loss: a mismatch destroys
`PendingContextResetRereceipt`; the post-Reset scope counts only later keys,
while the engine tail still includes the earlier observed prefix.
`capture_context_reset_rereceipt_candidate` then rejects the differing suffix
counts. Even retaining the unconfirmed record alone would be insufficient:
`advance_context_reset_rereceipt_after_key` currently clears an unconfirmed
record, and capture across another Reset accepts only a confirmed record.

The smallest next discriminator must reproduce this complete order through
the existing legacy adapter and real surrounding/Reset callbacks. Compare an
exact first receipt with a delayed shorter receipt, then a proven one-character
append, another authenticated same-owner Reset, and a fresh exact receipt.
Assert candidate retention separately from VisibleTail/ManualToggle authority
and GUI effects. Selection, navigation, content/capability changes, foreign
owner, non-append input, wrong full text, duplicate receipts and attempted
manual mutation before confirmation remain negative controls.

One hypothesis is to retain bounded non-authoritative observed lineage while
fresh client evidence is incomplete; it must never authorize a mismatching
snapshot or manufacture KnownStart from a clipped/empty snapshot. Keeping a
record through verified appends and Reset requires an explicit proof of owner,
token replacement, tail epoch and word continuity before implementation. This
is an analysis direction, not an accepted production design. The alternatives
are the unchanged refusal baseline and the broader Firefox selection-refresh
change already described above. No runtime/source edit, new timer, retry,
fallback, browser patch or authority change is authorized by this evidence
entry; a separate consequence analysis and causal RED must precede a repair.
