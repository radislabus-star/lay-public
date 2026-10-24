### 2026-10-24 — gate follow-up: common KDE presentation leak found

Receiver-order fix passes its existing positive and revocation tests. Heap-bound
fixture futures pass the previously overflowing test on its original stack.
Focused v1: 1084 checks, 15 FAIL; v2: 1084 checks, 14 FAIL. Attempts to adapt
owned-pair fixtures to stock commits/capability downgrade are rejected: they
changed the intended protocol and capability revocation correctly removed proof.
The original pair fixture/assertions have been restored byte-for-byte.

Source comparison with 5b3c6c5d identifies the actual shared regression: KDE
literal-prefix acquisition/retirement runs for all desktops. Restore previous
non-KDE behavior through the existing resolved presentation boundary, keep KDE
behavior, and add one actual key/observation contract across four desktops.
Existing KDE stock tests select KDE explicitly. Decision:
`2026-10-24-literal-prefix-desktop-scope.json`. New verification pending; no
native acceptance or installation is claimed by these source results.

### 2026-10-24 — full release gate: first receiver observation loses a live receipt

Baseline a8fbda3c plus the three security lockfile patches. Full hermetic run
`/home/e/.cache/lay/development/weekend-full-tests-20261024.log` reaches the
previously hidden failures after inventory regeneration. First proven defect:
`td159_exact_manual_observation_order_preserves_current_lease`, late SSI case,
fails on its first matching observation. `observe_external_surrounding_text`
sets callback_observed before capturing the current inherited manual receipt;
that capture consequently rejects the still-current receipt. Hypothesis:
capture before publishing callback facts, then use the existing matching-text,
owner, revision and unchanged-expiry refresh path. Do not relax receipt guards.

The repeated closed-source test also aborts from stack overflow. Bound nested
fixture futures on the heap, retaining the default stack, timeout, production
calls and all assertions. Pair tests additionally need an explicit GNOME inline
fixture instead of relying on the host desktop. Tests writing config under a
masked cache result directory will be rerun outside live-data masks, not given
write permission to live data. Runtime remains untouched. Verification pending.

### 2026-10-24 — KDE unavailable observation: retain only admitted literal input

CORRECTED first failure after selected44(36P/6F/2B,110.789s):all45transient
retention gates are TRUE. The next admitted key's successful read is rejected
by native install because desktop receipt identity differs while every AT-SPI
field component matches. Do not weaken the transient predicate. KWin provider
renews a local lifetime embedded in identity after errors;raw-ID components
not yet observed,so only a tightly constrained forward same-window recovery
is proposed. New owning decision2026-10-24-kde-native-field-transport-continuity.json
requires fresh current canonical/native proof,PID/bus/KWin-owner/window UUID/
AT-SPI identity equality and nondecreasing existing provider lifetime. Strict
provider and notification identities remain unchanged. RED tests added for
the actual fresh-install guard and following production key router;no runtime
change yet. All code remains in the KDE folder;common hooks still unapproved.

LATEST native verdict:ad5e full220153P/25F/42B,not accepted.13F->P but16P->F
versus6e45. All14manual failures still show5tracked versus6actual letters.
The claim that two short-run adjacent error/commit records proved the same
callback is withdrawn.49native+1desktop source PASS is a flat helper contract,
not a reproduction of the actual failed legacy key callback. Source now adds
opt-in transient_input_gate metadata for the exact refusal clause,without
changing predicate order/calls or authority. Diagnose token/scope/handoff/
notification ownership first;do not drop guards or repeat the helper-only claim.

Measured KWin5.24.7 UnknownObject /1 three times in short20 diagnostic14P/4F/2B
on installed42ab,48.377s. Busy not observed;script-ID lifecycle collision is
upstream-supported but not independently captured. Do not repeat a Busy-only
fix or increase timeouts. Earlier full row2 proves an accepted key is delegated
without a managed receipt after flattened native_no_desktop;later manual refusal
is correct. New decision2026-10-24-kde-native-observation-input-isolation.json
keeps existing canonical input ownership,clears native edit/display proof on
unavailable KWin observation,and retains ordinary managed key history only for
the same live owner. Known field/window/owner/selection changes still revoke.
No common dispatcher,correction,verifier or executor changes.

RED run-J6gkKn/localrun-1b6lydbo:623P/17F,two exact failures at the old refusal.
GREEN run-aEBiNt/localrun-1k5v07l1:626P/15oldF,49native+1desktop PASS,same
TD121 coverage error. Semantic control commits one literal key,no delete/forward,
checks stale versus fresh native snapshots,UnknownStart unchanged,11owner guards.
800Rust-file comparison proves only3KDE files changed since42ab. Build running;
native functional proof NOT_TESTED,installed42ab remains diagnostic. Full220
latest6e45 is157P/22F/41B,not overwritten by short diagnostics. Upper omnibox
geometry FAIL remains separate;common-release proposal still pending approval.

### 2026-10-24 — KDE Chrome Space refinement: native aggregate not accepted

Installed6e45 private1.0.94.1 frozen full220 is157P/22F/41B, compared with
previousDFA163P/19F/38B. Retain15PASS->FAIL,12FAIL->PASS,3PASS->BLOCKED;do not
present this as a stable improvement. Source canonical-word preparation has
47native controls PASS,unchanged15baseline failures/TD121 coverage error.
Owning evidence:desktop-presentation-isolation-2026-10-23.md and remote
chrome-941-space-word-proof-full-20261024. In exact manual row2 the first loss
is accepted press242/native_no_desktop2460us while typing в;release243 is
refused_timeout. Native actual word6chars versus managed5 explains the later
context_authority refusal. Do not weaken that refusal or assume why KWin read
failed. KDE desktop adapter drops the error;metadata-only opt-in trace added
there preserves the same result. Diagnostic not yet built/installed. No replay,
ownership,revision,timeout or delivery rule changed. Upper omnibox geometry
FAIL remains separate;two common release hooks still require the pending scope
decision. No Harness2/agents/GNOME changes or publication.

### 2026-10-22T03:41:23.274890+00:00 — v61 source463/463PASS; final scoped Harness2 review running

Remote fixed proof `stock-replay-v61-reset-affected/result.json`:463PASS/0FAIL, execution97.597s. Complete gate `stock-replay-v61-reset-source-gate/result.json`:PASS, architecture receipt PASS, diff check PASS, wall163.822s, ended2026-10-22T03:39:48.060203UTC. The same fixed denominator includes all4 unchanged regressions exposed by the two rejected variants.11focused controls PASS. Restoring the existing computation branch fixes ordinary Reset without allowing unproved completed replay; no request/edit route or authentication owner added.

Harness2 projectlay2 task `20261022T034019Z-2e25183e`, gpt-6.1-sol/xhigh, per-run480s; frozen13complete functions incl2full tests,36421inputbytes, `stock-replay-v61-refresh-final-review-r2/`. RUNNING, not accepted yet. Report expected at `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T034019Z-2e25183e/report.json`. Install only after independent report/input/current-source verification. Then remote engine-only build/install and6 short native Writer rows. Runtime still v60b, native v61 NOT_TESTED; Chrome prefix underline FAIL remains open, accepted Kitty preserved. Release/updater/GNOME/full browser matrix not revalidated.

Regular read-only upstream check2026-10-22T03:38:58.441348UTC:5open PRs, #68zbus5/5CI, #67rayon4/5, #66sha2 3/5, #65serde_json5/5, #64arboard3/5. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022T033858Z.json`. No merge/close/message/dependency change.

### 2026-10-22T03:36:13.639936+00:00 — v61 bounded replay guard: preserve the pre-existing computation branch

Measured source result: `stock-replay-v61-bounded-affected/result.json` is461PASS/2FAIL of463 (96.09s execution). The previous two replay-refusal controls now pass, but `firefox_bridge_publication_survives_shift_received_after_reset_suffix_binding` and `td121_current_metadata_unproved_release_only_retires_the_handled_press` fail at the first manual RPC (NotHandled instead of the prior pending read). Both unchanged fixtures have ordinary context-reset computation authority and no completed replay. The new missing-SSI condition unintentionally applied the stock completed-replay restriction to both branches of `context_reset_rereceipt_manual_wait_allowed`.

Chosen minimal correction: explicitly preserve `context_reset_rereceipt_computation_allowed()` inside that condition. Its existing exact-replay quarantine excludes completed replay; completed replay still needs exact-refresh capability or an already queued authenticated read. Alternative reverting the whole condition is rejected: it reintroduces the proven premature replay handoff. A new branch-specific queue/owner is unnecessary. No test assertions, receipt validation, token/lease/epoch/deadline or delivery route changes. Expected effect is only restoring the two established reset paths, not granting edits from missing SSI. No allocations, timers, cache/package, candidate/ranking or learning changes; one existing pure predicate is reused. Verify11 focused controls (all9 prior plus these2), then the same463 fixed proof. Failure rejects this correction. Runtime remains v60b PID1994712, SHA82d751b1ea32e0736a115392bb13948bd46aa364bd76b94868caa96494f2c245; source PASS is not physical-client PASS. Native/install/release/updater remain NOT_TESTED for v61.

Receipt root: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z`; runtime authority unchanged. Rollback boundary is only the added ordinary-computation alternative, preserving source-prefix A and all established refusals.

### 2026-10-22T03:28:51.078049+00:00 — v61 unrestricted refresh rejected; bounded refresh9/9GREEN

Fixed affected gate463:461PASS/2FAIL, receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v61-affected/result.json`. Both original assertions remain unchanged: `td121_inherited_full_prefix_refuses_contradictory_final_receipt` and `td121_native_replay_progress_never_authorizes_pre_final_manual_effects`. The unrestricted capability extension permitted a partial SSI request and an immediate manual handoff from missing SSI; never installed, archived `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/rejected-unrestricted-stock-refresh-v61/`.

Causal ablation restores only exact-refresh eligibility while retaining real-source-prefix fix:8/8PASS (`/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-replay-v61-ablation/result.json`). Source-prefix fix preserved. Bounded replacement requires a prior whole source surface or whole replacement word without its boundary for ordinary-client reads. Missing/partial SSI cannot start a manual gesture; existing queued wait retains its700ms deadline. Exact-refresh sentinel behavior unchanged. Focused9/9PASS, including both previously failed unchanged controls; compilation12.89s/execution2.19s. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v61-refresh-bounded/result.json`.

Next: rerun the same463proof after architecture refresh, then final bounded Harness2 review; no candidate installation before these gates. Live remains v60b. Upstream read-only rechecked2026-10-22T03:20:54.931121UTC:5open PRs68/67/66/65/64; CI5/5,4/5,3/5,5/5,3/5. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022T032054Z.json`; no merges/upgrades/messages.

### 2026-10-22T03:19:27.878934+00:00 — v61 focused GREEN7/7

Existing pending-manual refresh extension now passes all7selected checks after causal RED6/7. Tests cover original delayed and final-letter-without-Space actual receipts, empty/Unicode external prefix, ordinary/exact-refresh capabilities, Space/NBSP and incomplete/contradictory/expired/atomic/sealed/no-surrounding refusals. Source compilation12.94s, execution1.73s. Remote receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-replay-v61-refresh-green/result.json`. No runtime changed. Next fixed affected scope463checks plus architecture refresh, final bounded review, engine-only build/native qualification.

### 2026-10-22T03:18:04.121006+00:00 — v61 source context retained; release hook rejected; existing refresh under test

- Live remote engine remains v60b, PID1994712, SHA256 `82d751b1ea32e0736a115392bb13948bd46aa364bd76b94868caa96494f2c245`; no v61 installation or client claim.
- Source-prefix change: causal RED then5/5 GREEN, including empty/Unicode real external prefix and no invented target SSI/revision/ACK. Preserve this fix.
- Harness2 USED task `20261022T024721Z-67fc0e30`, report `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T024721Z-67fc0e30/report.json`: statusok,8calls,0failed, no fallback; bounded review8/10. Independently checked frozen inputs and8source excerpts; review is not native acceptance. Earlier240s-timeout task rejected.
- Rejected final-release request prototype:7th attempt still5PASS/1FAIL because observer-before-callback ordering rejects the shortcut. Never installed; archived `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/rejected-final-release-read-v61/`. Authentication guards retained; prototype metadata/hook removed without removing source-prefix fix.
- Replacement hypothesis: reuse existing pending-manual700ms wait for completed, paired, current replay and prior actual SSI. Extend request eligibility to ordinary surrounding-text capability; final exact SSI remains mandatory for edit. Existing sentinel branch retains exact-refresh capability. Causal RED:6PASS/1FAIL at ordinary-client outcome0 vs expected4, receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v61-refresh-red/result.json`. GREEN running remotely; no verdict yet.
- Native known v60b scope: LibreOffice5PASS/1FAIL, reverse closed-word gesture fails; Kitty accepted; Chrome TabPASS, typed-prefix underlineFAIL. No full browser/GNOME/release/updater result for v61.
- Upstream checked2026-10-22T02:54:36.527396UTC: open PRs68,67,66,65,64; CI5/5,4/5,3/5,5/5,3/5 respectively. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022T025436Z.json`. Regular read-only review rule is AGENTS.md; no PR state/dependency changes.
- Next: stock/managed refresh-positive and refusal controls, full affected scope, final bounded Harness2 review, engine-only candidate and short native checks if source gates pass. Keep daemon/globalIBus/tray/desktop/config untouched.

## 2026-10-22T02:39:26.091289+00:00 — causal RED: observed source prefix is lost on engine handoff

Remote `stock-replay-source-v61-red/result.json`: four existing tests PASS; new `stock_observed_boundary_handoff_preserves_real_prefix_without_target_receipt` FAILS at actual replay source prefix `None` vs `Some("")`. Fixture uses real SSI, successful local Space with no invented final SSI, authenticated manual RPC, actual factory/activation handoff and suppression RPC. Target still has no SSI/ACK; the failure is retained source provenance, not target acknowledgement. Unicode-prefix case remains behind the first RED assertion and is not yet claimed tested.

Harness2 USED task `20261022T023435Z-586d6f4c`, lay2/gpt-6.1-sol/xhigh, bounded frozen plan review RUNNING; no verdict yet. Inputs matched remote origins before launch. Proposed correction carries explicit actual-snapshot versus observed-word/local-boundary provenance through the existing reducer receipt; a separate real result-read request may be needed after final native key release. No new owner, fake SSI/ACK, relaxed verifier, client-name bypass or delivery route. Live v60b unchanged, LibreOffice reverse/Chrome styling still FAIL. Next: source fix and strict negative checks, independent Harness2 review, then short remote native proof before acceptance.

## 2026-10-22T02:22:58.218994+00:00 — v60b installed; LibreOffice reverse closed toggle remains FAIL

Remote engine-only install at2026-10-22T02:10:03.342822UTC: PID1994712/SHA25682d751b1ea32e0736a115392bb13948bd46aa364bd76b94868caa96494f2c245. Candidate/source closure matches the462/462 remote source/architecture/build proof (1258files/7db8e45bc327d05d3bbb814ab079803d3e500c5f8042b4bbe4b7f668172e5cd2). Deployment `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-local-boundary-v60b-deployment/result.json` preserves8protected processes/config/tray; no daemon/global IBus/desktop restart. No local installation or publication.

Harness2 USED task20261022T014733Z-2b7280e4, lay2/gpt-6.1-sol/xhigh: reportdegraded, coreok,8calls/1timeout/c1fallback; bounded8/10/no demonstrated source blocker. Thirteen frozen hashes/eleven full-origin excerpts/seven remote full source files independently match. Acceptance `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-local-boundary-v60b-review-acceptance.json` is SOURCE_ONLY, not client proof. Full report `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T014733Z-2b7280e4/report.json`.

Native LibreOffice v60b default profile5PASS/1FAIL: first/second hint+Tab, active Double Shift roundtrip, physical Shift+A, Space auto-flip PASS. Closed target becomes `Ghbdtn `; independent ordinary input with actualxkb:us::eng and no Lay also yields `Ghbdtn `, proving Writer sentence capitalization. Controlled NEW owned profile with only sentence capitalization disabled retains the six strict assertions:5PASS/1FAIL; first closed toggle gives exact`ghbdtn ` in180.165ms, reverse gives no change. Active toggles198.12/181.522ms. No user profile/config change. All7restorations true. Receipts `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/libreoffice-v60b/`, `libreoffice-default-capitalization-v60c/`, `libreoffice-no-capitalization-v60c/`. Actual screenshots show ordinary prefix without IME underline; application spellcheck/header marks are distinct.

Short native regression `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-local-boundary-v60b-native-controls/summary.json`: ordinary-launch Kitty hint/Tab/closed roundtrip/next decoder PASS; closed samples15.324/16.833ms. Chrome textTab1/1PASS, visual prefix underline remains separate FAIL;6restorations true. No physical-evdev/full browser/GNOME acceptance claimed. Historical browser100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320 unchanged.

Next: causal source fixture for stock-client native replay followed by second closed gesture. Actual replacement letters have real SSI, final forwarded Space has none; readout becomes unknown-context (epochs287/288). Preserve replay receipt/quarantine guards; never promote forwarded key delivery to client ACK. v60b remains live. Chrome attribute-only proposal already rejected; no repeated attribute tweaks.

Fresh read-only PR check2026-10-22T02:21:23.575226UTC:5open; #68/#65CI5/5,#67CI4/5,#66/#64CI3/5. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022T022123Z.json`. AGENTS.md330–339 mandates regular checks; no merges/messages.

## 2026-10-22T02:01:44.190595+00:00 — Chrome prefix-only wire rejects another attribute-only proposal

No production/runtime change. Re-inspected actual v59 native Chrome wire: nonempty compositions contain only owned `п`, `пр`, `про`, `пров`, plus decoder controls `z`/`я`; no combined `проверка` preedit. The continuation is the existing readonly view. Removing explicit prefix UNDERLINE_NONE therefore triggers Chromium's whole-composition default rather than leaving a separately styled suffix span. Reject the proposed attribute omission before coding. Earlier parser assertion that expected combined `проверка` is a diagnostic assumption failure, not a new client test failure. Exact retained evidence: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-prefix-attributes-v61-proposal.json`; wire SHA-256 bebaae7f360e8a8b7596ac2012a6c604eea27d848747d8dda9054732007d4b60. Chrome native presentation remains FAIL. This confirms the existing no-attribute-only-remedy finding; do not repeat it against the same prefix-only output. v60b review/native sequence remains the one active change.

## 2026-10-22T01:52:21.977835+00:00 — v60b related source proof and frozen review

v60b remote graph/build gate PASS: 462/462 selected checks, architecture PASS; engine candidate SHA-256 `82d751b1ea32e0736a115392bb13948bd46aa364bd76b94868caa96494f2c245`. Source closure1258files/7db8e45bc327d05d3bbb814ab079803d3e500c5f8042b4bbe4b7f668172e5cd2. Receipts: `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-local-boundary-v60b-postgraph/result.json` and `stock-local-boundary-v60b-build/result.json`. Existing stale-observed-client refusal retained unchanged. Installed engine remains v59/b1ae6be2; candidate native behavior NOT_TESTED.

Harness2 USED, projectlay2/modelgpt-6.1-sol/xhigh, task `20261022T014733Z-2b7280e4`, still running. Thirteen frozen inputs, eleven excerpts independently verified, seven full origin files match both workspaces. Report pending: `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T014733Z-2b7280e4/report.json`. Next only after review: engine-only install and six-row LibreOffice probe, ordinary-launch Kitty and Chrome controls. No daemon/IBus/tray/desktop restart, local installation or publication.

Fresh read-only PR check2026-10-22T01:42:22.155387UTC:5open; #68/#65 CI5/5, #67 CI4/5, #66/#64 CI3/5. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/upstream-pr-review-20261022T014222Z.json`. No merges/messages. Chrome typed-prefix underline remains separate FAIL; historical full browser100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320 unchanged.

## 2026-10-22T01:41:22.048889+00:00 — v60b source lifecycle clarified

Focused r2 proves first pre-Space word has UnknownStart despite an exact real client snapshot. Capture is inert and now requires a live matching token; post-Space manual readout still requires KnownStart. Focused r4 proves the record exists after Space, but the existing authenticated manual handoff republishes identical text at the next tail epoch. v60b carries only its already-current source across that one successful handoff; unrelated epoch changes still refuse it. No exact snapshot or delivery ACK is published. Failed compile in r3 was a missing test-only import, corrected explicitly; all receipts retained. Broader source, review and native verdicts pending.

## 2026-10-22T01:35:23.453310+00:00 — v60a rejected by the existing observed-client guard; v60b narrowed candidate

v60a related gate **460 PASS / 1 FAIL** (`stock_owned_manual_source_rejects_stale_unknown_and_observed_contexts`, observed-client case). A historical callback plus any local commit cannot prove the current word. Existing refusal/assertions are retained unchanged. Receipt: `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-local-boundary-v60-postgraph/result.json`; build stopped before release compilation. Running engine remains v59/b1ae6be2.

v60b hypothesis: preserve exact real pre-Space word observation only across one successful local ASCII Space. Capture in existing managed CommitText path; bind exact tail, existing tail epoch, focus serial, owner lease and actual observation revision. Clear at new input, observation, revocation or field close; revalidate existing KnownStart/current token and all manual handoff/refusal gates. This inert source record creates no owner, generation, timer, queue, executor, synthetic snapshot or delivery ACK. Epoch/owner/revision/text mismatch and native replay quarantine refuse it. Causal test and existing observed-client refusal must both pass; broader proof, Harness2 review and native client acceptance pending.

## 2026-10-22T01:22:13.441048+00:00 — v59 accepted source/native scope; v60 local-boundary source candidate

v59 source459/459, Harness2task20261022T004203Z-1eaf86cb8/10 and engine-only deployment SHA b1ae6be27b67b539e349819a76f51b72464eddfa5ee1cb22bb34bafeb63ef2b0: LibreOffice second-word hint/Tab repaired; latest native5/6 keeps closed Double Shift FAIL. Ordinary Kitty controls PASS, Chrome prefix underline FAIL. Owning preflight contains exact receipts.

v60 hypothesis: known current stock-client word remains a bounded explicit manual source after this engine commits Space, despite earlier letter callbacks and no post-boundary SSI. Causal baselineRED stock-local-boundary-v60-red-r4; r0-r3 invalid fixture setup is not product evidence. Candidate only changes existing stock-owned predicate: successful local commit+trailing boundary+KnownStart+no SSI/floor+no ExactReplay/quarantine, existing field/content/owner guards unchanged. No fabricated SSI/ACK or new delivery route. Decision2026-10-22-stock-local-boundary-manual-source.json.

First green source run1/2PASS: positive causal source fixed, negative replay setup tried suppression before the decoder reached the requested target. Corrected fixture now follows the existing daemon handoff/readout/target-decoder sequence; no production guard was relaxed. All broader source/review/native gates remain pending; runtime stillv59. Preserve prior native replay receipt requirements and rejected unconditional ordinary-commit mechanism.

## 2026-10-21T19:02:21.754237+00:00 — v23 startup repair accepted within scope; native replay lifecycle failure retained

No Rust/core experiment in this milestone. Existing client setup owns ordinary Kitty startup; reversible command fixes the causal direct-vs-desktop difference. Latest19/19remote sourcePASS, exact installed helpera2859159/wrapper984a84c7. Same-wrapper ordinary native hint/Tab,10/10targets andShift4/4PASS; punctuation/old geometry FAIL retained. New user-review window561418 has IBus; no protected app restart. Harness2USED20261021T183803Z-de6d1dd7:8callsOK,8/10review, sole PATH false-success finding independentlyRED/fixed/GREEN after review. Earlier timeout is not review success.

New Chrome omnibox after-Space nativeFAIL has a causal bus/engine trace: first deletion native, then Reset/FocusOut/In cycles, second FocusOut revokes owner, local/shared replay state mismatch consumes Backspace. Do not repeat frame/settle experiments: output keys arrived. No unsafe broad pass-through or retained focus authority patch. Detailed receipts, invalid-fixture distinctions and next source RED requirement are in latest [preflight](../architecture/ime-inline-hint-preflight-2026-10-20.md). Historical browser100/76/0/144 of320 unchanged. Native source pin6d488123/enginecf76bc is not replaced by the Python/doc closure; remote graph/architecture checks PASS with six-artifact receipt `shift-hint-20261021T1242Z/docgraph-v23-independent-acceptance.json`.

## 2026-10-21T18:12:18.096679+00:00 — short native v21 verification, open failures retained

GTK/Qt25/25functionalPASS; active bursts only, separate after-Space roundtrips. Five desktop hint captures reviewed independently. Browser seven-case3PASS/4FAIL remains; one-case latency profile observes a79.78mslayout-list reply and83freshKWin script loads, without proving a safe optimization. Kitty missingrectFAIL persists despite a valid diagnostic capture. Harness2 task20261021T174153Z-6dce90b1:8/8OK,7/10incomplete static review, no demonstrated defect; truncation/omitted-wrapper gaps retained. Current enginecf76bc/source6d488 and8protected processes independently pinned. No production patch at this verification milestone. Exact receipts, visual limits, restored state, untested denominator and review audit: [owning preflight](../architecture/ime-inline-hint-preflight-2026-10-20.md), newest entry; local `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/native-short-checks-v21-local-review.json`. Rejected frame/settle experiments stay rejected.

## 2026-10-21T17:54:00.062569+00:00 — ordinary Space repair passes native controls; open failures retained

Scoped development engine326051/SHA`cf76bc925c96a738e2b4a9a4a37619713e0efb3d544b54d4d9b75f56337604b0` installed17:43:55UTC. Causal real-Space RED1/1, affected isolatedGREEN266/266, graph/architecture PASS. Independently recomputed local/remote1257-file source+mode identity `6d488123b563c16a7972862a88a79cad1cd4257428db6592c067f567eea922cd`. Only engine replaced; daemon1802bb5e, globalIBus, tray, config and protected8 preserved. No new delivery/edit/learning authority.

Native normal-profile browser receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-user-space-browser-native-v21/summary.json`: Firefox iframe ordinarySpace roundtrip+decoder PASS; Firefox iframe and Chrome text Tab roundtrip+decoder2/2PASS. Four after-Space rapid-five cases remainFAIL: all20/20 intermediate targets appeared (v20:13/20), but15/16 intergesture deadlines missed. Twenty release-to-DOM samples median167.92ms,p95234.95ms,max243.08ms; retained clock uncertainty, no relaxed cadence or latePASS.

Native terminal receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-user-space-terminals-v21/summary.json`: Kitty/Konsole Tab+roundtrip+decoder2/2PASS; active/Space five-toggle4/4cases and20/20targetsPASS; heldShift RU/US left/right8/8PASS. RU punctuation2FAIL (`!,Ж()_` instead of `!?:()_`), Lay causality not proved. **Independent screenshot inspection: Konsole ordinary prefix+complete opaque suffix PASS for this capture; Kitty hint absent FAIL**, despite functionalTabPASS. Kitty snapshot has suffix`ерка` but`rect=null`; do not merge functional and presentation verdicts. Chrome prefix underline and Firefox iframe prefix overlap remainFAIL. Restoration checks alltrue.

Harness2 frozen v21 review task20261021T174153Z-6dce90b1 stillrunning; no review acceptance claimed. Local evidence manifest `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/v21-receipts-manifest.json` retains39explicitJSON/PNG files, no browser profiles. Historical full browser100PASS/76FAIL/144NOT_TESTED of320 unchanged. No fullmatrix, GNOME, release, push, local install or global restart. Next: explain first lost Kitty hint geometry and existing browser latency, without repeating rejected frame/settle experiments.

### v21 causal verdict — source only

Old implementation: 1/1 expected causal RED at exact ASCII/NBSP tail equality,
actual Space dispatch (no fixture setup failure). Corrected source: **266/266**
affected isolated checks PASS, including the previous262 and four ordinary-Space
identities. Receipts: `stock-user-space-v21-red/result.json` and
`stock-user-space-v21-green/result.json` under the existing remote audit root.
Current runtime has not yet been changed by this experiment; native pending.
No edit/delivery/learning authority changed. Appearance and historical browser
matrix are still FAIL/partial as recorded above.

## 2026-10-21 — ordinary stock-client Space first receipt: preflight

Native `browser-space-burst-current-v20` on engine882385ae: Firefox iframe after
ordinary Space retains `привет NBSP` through all five gestures (zero targets).
Tab completion passes separately. First unresolved mechanism: no-snapshot owned
Space expectation explicitly excludes UserInput. This is a hypothesis pending
an actual Space-key causal RED, not yet a client fix.

Decision and full consequence analysis:
`docs/architecture/decisions/2026-10-21-stock-user-space-first-receipt.json`.
Reuse conditional whole-field acknowledgement only; output, detector, verifier,
learning and delivery authority unchanged. Preserve prior Tab/terminal controls.
Full browser table and appearance failures remain unchanged. New focused source
and native receipts will be appended at each verdict; no release acceptance.

## 2026-10-21T17:29:19.515707+00:00 — exact new runtime checked; stock completion failure repaired in bounded native cases

Installed scoped KDE development engine249471/SHA882385aeec5233cb721998a22e22fb3f8177bf21b021061a17cce14c999d2f45 at17:20:24UTC. Guarded build and architecture PASS; local/remote canonical1257-file bytes+modes match SHA75470100e8aef679d1c7b7f6d2e897f637de881b3ac45d8f324f6ff39b1011c8. Reused262/262 isolated sourceGREEN; no redundant rerun. Daemon49153/1802bb5e, globalIBus, eight protected processes and config preserved. Delivery and mutation authority unchanged.

Actual owned normal-profile Firefox iframe-rich and Chrome text each PASS: native `пров`, hint screenshot, real Tab to `проверка `, DoubleShift both directions, next-letter decoder. Firefox raw NBSP retained. Receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-completion-space-browser-native-v17/summary.json`; repeat read-only geometry diagnostic `browser-hint-geometry-v19` also2/2functionalPASS. Source fixture setup failures v17/v17b are excluded from valid causalRED v17c.

Short current-byte terminal receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-completion-terminals-v18/`: Kitty/Konsole hintTab-to-DoubleShift-to-nextletter2/2PASS; active/Space five-bursts4/4cases with20/20 intermediate transitionsPASS; left/right ordinaryShift RU/US8/8PASS. Twenty burst transitions median19.88ms,max21.55ms from secondShift press to PTY exact target (not full-gesture/end-to-end browser latency). RU punctuation2FAIL retained; Lay causality not proved. Independently viewed current Kitty/Konsole captures: typedprefix ordinary and entire suffix black on opaquewhite. This visual result covers those two captures only; old Kitty missinghint capture remains historicalFAIL. All restoration checks true.

Current browser captures stillFAIL appearance: Chrome typedprefix underlined; Firefox iframe suffix covers/hides prefix. Prior rapid after-Space browser failures remain open. Historical fullbrowser100PASS/76FAIL/144NOT_TESTED of320 unchanged; no fullmatrix/GNOME/private/fullscreen/release gate. Harness2 new review20261021T171945Z-cece8c8c running; v16 degraded7/8 review independently audited, static scope only. No push/release/local install/global restart.

## 2026-10-21T17:17:45.055918+00:00 — stock first-receipt candidate source GREEN; native pending

`stock-completion-space-v17c-green/result.json`:262/262 exact affected identities PASS in isolated bwrap sandboxes, default stack;12.45s compile plus10.07s execution. Test artifact4041b945407ab517cc0a984fe62844fd5608dab27722dfb072fc1219b2574dd0. Source-only; currently installed174016/aecf29d3 still has the retained native Firefox failure. Conditional whole-field expectation keeps KnownStart/UserInput/output routes unchanged. New negative cases cover old field prefix, wrong word/boundary count, trailing field text, selection/caret, expiry/epoch/foreign owner, current token/mode/owned composition/exact edit and incomplete/stale observed starts.

Harness2 v16 review task20261021T165016Z-bbdbb45b completed degraded:7/8callsOK, dom1 timed out240s, no fallback; selected bounded static review9/10, no findings, explicit missing implementation proof gaps. This does not cover the new v17 delta and does not override nativeFAIL. Build and physical acceptance remain pending. Historical full browser100PASS/76FAIL/144NOT_TESTED of320 unchanged.

## 2026-10-21T17:09:17.464644+00:00 — actual Firefox fails; initial observation absent

Installed engine174016/aecf29d3 retains the v16 source257/257 PASS but Firefox iframe Tab-to-DoubleShift FAIL. Chrome text control PASS. Trace `firefox-iframe-completion-trace-v17` proves stock caps41, exact refresh unavailable, and no surrounding snapshot until AFTER Tab. The v16 regression supplied a prior snapshot, so its qualified receipt cannot run in the actual failure. This native failure is retained and is not reclassified by source success.

Short native receipt `completion-space-short-native-v16`: GTK/Qt25/25PASS; terminal20/20 individual DoubleShift transitions and ordinaryShift8/8PASS; RU punctuation2FAIL baseline. Viewed screenshots: Chrome typed-prefix underlineFAIL; Firefox iframe prefix hiddenFAIL; Kitty hint absent at capture (functionalTabPASS only); Konsole black-on-white complete suffix bounded visualPASS. All protected runtime/config/layout restorations true. Full historical browser100PASS/76FAIL/144NOT_TESTED of320 unchanged. No release/push.

Next causal regression uses stock caps41 and NO initial surrounding callback. A candidate must acknowledge only a subsequently observed exact whole-field owned word plus one boundary, without inventing a prior prefix, downgrading a known stale witness, or changing edit/learning authority. Receipt paths are under `/home/e/projects/lay-development-runner/kde-install-control-20261020/`; local copies under `shift-hint-20261021T1242Z/remote/v16/`.

## 2026-10-21T16:54:11.126138+00:00 — completion receipt source257/257; native proof pending

First failed transition proved in remote Firefox iframe: Tab commits ASCII Space, client reports NBSP, next ManualToggle delegates but V4 sees unknown-context. One real-key source regression RED; existing exact owned-Space receipt reused for matching authorized completion, with all field/owner/prefix/selection/expiry guards. Affected isolated257/257 GREEN, no runtime/delivery authority change. Decision `2026-10-21-completion-owned-space-receipt.json`; owner entry has exact receipts and limits. Harness2 task20261021T165016Z-bbdbb45b running. Current engine4076822/5934db2f and original daemon49153/1802bb5e unchanged; next scoped development engine build/deployment and native Firefox/Chrome/Kitty/Konsole controls. Presentation and browser fast-Space burst failures remain open; full historical table unchanged. Rejected frame-batching review finished8/10, independently audited, candidate still removed. No release/push/global restart.

## 2026-10-21T16:18:36Z — finite-frame fence rejected after native comparison

Candidate0941177b source52PASS including26owned-Xvfb controls, native GTKQt25PASS and terminal controls retained. Browser normal bursts remain10PASS/10FAIL per browser; Chrome45 measured Space targets median183.0→183.6ms and p95283.6→301.0ms. No useful benefit; source and daemon restored to exact prior bytes. Decision2026-10-21-x11-closed-frame-fence.json forbids silently repeating or retaining this failed experiment. Detailed owner entry records snapshots, fixture/setup failures, deployment/rollback identities, Harness2 timeout and pending narrowed review.

Fresh restored-daemon Firefox diagnostic: five DelegateExactImeTail replies1.22–2.35ms, five exact target observations163–173ms, fast deadlineFAIL. This reproduces latency after admission without production logging. Earlier missing targets and Chrome prefix underline remain open; safety/focus/settlement bounds unchanged. Full matrix remains historical, no new release acceptance.

## 2026-10-21T15:56:03.767890+00:00 — finite X11 frame experiment, not installed

Metadata-only native profile `bridge-v7b-space-dbus-profile-v10` retains five Chrome targets but misses fast deadlines. Bounded profile: 85 fresh KWin reads across the run; one getLayoutsList call80.17ms; output also performs serial same-connection fenced writes. Do not infer that frame batching fixes the whole latency. Candidate decision `2026-10-21-x11-closed-frame-fence.json` keeps pacing and all safety/settlement contracts; remote owned-Xvfb proof, source build, bounded Harness2 review and native comparison pending. Current daemon1802bb5e and engine5934db2f unchanged. Full browser table unchanged; Chrome prefix underline and Firefox iframe presentation/Tab remain unaccepted.

## 2026-10-21T15:44:34.075357+00:00 — fresh browser burst split and bounded review complete

Current engine remains PID4076822/SHA5934db2f64e5fee37e376b85eb955b6e1bdc3b2aaa7de7c681156dc846b39b01. `bridge-v7b-browser-double-shift-v9`: Firefox10PASS/10FAIL and Chrome10PASS/10FAIL of20 each, zero BLOCKED/NOT_TESTED. All ten active-word five-toggle bursts pass in each browser; all ten after-Space bursts fail. These are normal-mode diagnostics only and do not replace historical full-matrix100PASS/76FAIL/144NOT_TESTED of320. The original old-geometry and old-timing receipts are retained.

The browser fixture now scales native RFB clicks by measured DPR (Chrome1.0625), records actual gesture starts, and conservatively bounds the DOM/native clock mapping. Five exact alternating targets plus the next-letter decoder probe remain mandatory. Chrome after-Space targets generally arrive but overlap the next gesture; Firefox also loses intermediate targets. Examples: Chrome text137.62/157.31/281.75/218.88/164.62ms; Firefox URL192.07/181.60/288.85/227.38/175.06ms. These failures are not excused by a final matching string. After-Space uses existing delegated daemon replay; first expensive or lost transition still under diagnosis. No performance/runtime bypass has been added.

Harness2 USED task20261021T151518Z-2319fcc6, report/coreOK,8/8calls, zero failures/fallbacks, frozen source unchanged. Selected bounded static adapter review9/10, approve with proof gaps, no concrete findings. Independent evidence: two valid old-code RED identities,242 affected GREEN identities, native Qt former failure now PASS; dedicated cancellation/elapsed-ceiling and full-source gates are not covered by the three new regressions. Review does not accept browser or presentation failures. Report: `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261021T151518Z-2319fcc6/report.json`.

Remote receipts: `/home/e/projects/lay-development-runner/kde-install-control-20261020/bridge-v7b-browser-double-shift-v9/{summary.json,restoration.json,firefox/result.json,chrome/result.json}`. Local copies: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/remote/browser-v9/`. All restoration checks true. Chrome typed-prefix underline remainsFAIL; Firefox iframe presentation/Tab unaccepted.25GTKQt/20terminal transition proofs retain their bounded scope. GNOME, private/fullscreen and release gates not rerun. No push/release, daemon/globalIBus restart or authority/delivery-route change.

## 2026-10-21T15:33:20.095711+00:00 — activation fence source GREEN, native Qt PASS

Current remote engine PID4076822 SHA5934db2f64e5fee37e376b85eb955b6e1bdc3b2aaa7de7c681156dc846b39b01. Candidatev7b:242/242 Rust+14JS+2PythonPASS; all25GTKQt and20terminalDoubleShift transitionsPASS. Same four TD121 stack-crashing fixture cases pass after one-await-loop shaping, with default stack/assertions/deadline unchanged. Full source gate remains NOT_TESTED. Firefox retains five exact burst targets but old onset timingFAIL; Chrome focusBLOCKED due measured DPR1.0625 CSS/RFB coordinate error. Diagnostic-fixture corrections pending native acceptance. Full browser table unchanged. Owning result and receipt paths: `docs/architecture/ime-inline-hint-preflight-2026-10-20.md` latest entry. Harness task20261021T151518Z-2319fcc6 still running. No release/push or global restart.

# TD-121: журнал изменений ядра и запрет повторения отвергнутых подходов

## 2026-10-21 15:11 UTC — exact-replay activation Busy, bounded causal RED

First failed transition: selected-engine change -> VisibleTailV3 mode read
before activation marker publication -> Busy -> exact replay restored without
text mutation. Repeated Qt plain native failure and DBus/engine traces:
terminal-v6-qt-plain-bus-trace. Source RED bridge-fence-red-v7b: two Busy failures
on unchanged production code, concurrent-Bridge refusal PASS. Earlier v7 is
invalid interrupted fixture evidence. Existing-event/20ms wait is proposed
under decision2026-10-21-bridge-waits-for-existing-activation.json; GREEN and
native proof pending. No new owner or executor. Full-stack overflow remains
OPEN. See owning ime-inline-hint-preflight entry for current c542167e terminal
20-flip acceptance, browser gaps and complete receipt scope.


## 2026-10-21 14:34 UTC — current-terminal qualification before manual admission

Candidate059f8334: affected source200/200, hint JS14/14 and Python2/2,
architecture PASS. Development-only deployment-v4 pins PID3863797 and both
actual activation/installed executables; daemon/tray/config and eight protected
processes unchanged. Native `terminal-candidate-v4-{konsole,kitty}-five`:
after-Space five consecutive Double Shift PASS in each (14.36–22.10 ms visible
latency, ten samples); active-word first PASS, second FAIL in both. The table
of complete browser runs is unchanged. Do not claim terminal acceptance.

Trace `terminal-candidate-v4-konsole-active-trace/engine-trace.jsonl` shows
cursor geometry arriving before activation Ready; the second active-word RPC
returns context_authority with a live bridge token and no current terminal
qualification. The existing refresh runs only after eligibility, so UnknownStart
never reaches it. Move the same current-field probe before eligibility and
compute bridge liveness after the await. Preserve all owner/field/epoch/token,
sensitive, atomic, exact-refresh and contradictory-source guards. No new owner,
backend fallback, selection relaxation or delivery executor. Native five-toggle
control is the exact regression: both spaced and active words, then one next
letter checking the decoder. Existing affected source guards also rerun; no
source-order assertion or artificial desktop success is used as acceptance.
Decision: `2026-10-21-terminal-manual-admission-probe-order.json`.
Implementation/proof of the ordering repair PENDING. Chrome typed-prefix
underline, RU punctuation, full-source stack overflow and full GNOME/native
browser matrices remain open.

Remote evidence root:
`/home/e/projects/lay-development-runner/kde-install-control-20261020/`.

## 2026-10-21 14:02 UTC — qualified terminal display-line metadata

First failing transition: after a successful native terminal replacement and
engine handoff, Konsole's unchanged display line arrives with cursor7/anchor0.
The shared verifier rejects the return toggle as a selection despite equal
tail text. RED `shift-hint-selection-red-v2` reproduces it through production
manual execution; red-v1 had an invalid missing-output fixture and is retained
only as a failed test attempt. Do not remove the verifier's selection check or
drop contradictory text. Reuse the selected qualified executor to distinguish
terminal display-line metadata from editable-document selection. Exact-refresh,
revoked-field/probe/cell, atomic, sensitive and contradictory-tail controls are
required. Owning evidence/decision: inline-hint preflight entry14:02 and
`2026-10-21-terminal-display-line-selection.json`. Implementation proof pending.

## 2026-10-21 — ordinary Shift failure is upstream of Lay; module relocation

At09:14:43.250377Z, readback confirms live VNC modtweak=0/xkb=1 and the sole
persisted `-nomodtweak` launch flag; all ten protected process identities and
engine/daemon/config bytes remain unchanged. Baseline RU held-Shift failure
reproduced with IME disabled, with a synthetic Shift release before KeyPress.
Reject a Lay uppercase override: it would mask the transport failure and alter
genuine lowercase input. Reinitializing the original VNC mode did not repair it;
disabling XKB introduced a US punctuation regression and was rolled back.
Selected nomodtweak restores8/8 ordinary Shift controls in Kitty/Konsole. The
pre-existing RU punctuation control still fails in each six-row suite; retain
5P/1F per terminal rather than claiming universal PASS.

Separate mechanical source relocation: existing terminal focus probes moved to
small KDE/GNOME modules behind the existing backend dispatch. Predicate/probe
body parity and unchanged test assertions verified; canonical isolated source
controls7/7 before and after, rustfmt PASS. No core word ownership, replay,
learning, deletion executor, verifier or delivery route changed. No relocated
runtime installed. The last full-source stack-overflow failure remains open.
Owner and exact receipts: preflight2026-10-20, section2026-10-21; remote
`ordinary-shift-*/`, `desktop-focus-module-{before,after}-v1/` and local
`ordinary-shift-20261021T0838Z/module-mechanical-parity.json`.
Narrow Harness2 task20261021T085025Z-03eb4287: 8/8 calls OK, no failures or
fallbacks, selected transport review A8/10; inputs/report independently checked.
Post-review native Double Shift round trip remains Kitty PASS / Konsole FAIL,
so the ordinary-Shift transport repair does not close the Konsole Space defect.

## 2026-10-20 — native Konsole suffix readout regression

Current result: source controls27/27 GREEN; verified installed engine d4215880,
native Konsole hint/Tab/blur2/2 PASS, full Kitty4/4 PASS, full Konsole2/4 PASS.
Whole-word authority is unchanged; failed Double Shift/moved boundary remain
OPEN. Five GTK/Qt kinds22/25 functional PASS but GTK Entry presentation FAIL
(full field-height incorrectly scales the suffix font). The frontend-only
geometry repair and solid-gray user revision are pending native confirmation.
Full canonical source gate FAIL: unchanged deep native-transfer test stack
overflow in `suffix-readout-canonical-source-v2`; do not raise stack limits or
reuse the old943/943 gate. Exact receipt paths and current runtime identity are
recorded in `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`.

Native Konsole candidate `02fb706d1dcca49066c2e21ee95acc7251ce4b2de634ae261726060765a59038` fails the owned raw-PTY run (0/4); prefix remains ordinary but hint/Tab and Double Shift fail. The bounded cause trace establishes current KWin terminal qualification, caps41 without SurroundingText, native key callbacks and UnknownStart. The first failed display transition is the native observer asking only for KnownStart plus an erase-dependent suffix predicate. `suffix-readout-red-v1` reproduces both missing readout and rejected append-only Tab: 25/27 source checks PASS, 2 intended FAIL, compilation/discovery12.836s and execution0.572s. No rollback to owned document preedit, no fake snapshot or KnownStart, and no deletion-route change is proposed. Reuse the existing display suffix identity and isolate append/readout from the stronger existing whole-word predicate. Owning decision `2026-10-20-native-terminal-suffix-readout.json`; native promotion remains OPEN. All receipts are under `/home/e/projects/lay-development-runner/kde-install-control-20261020/`.

## 2026-10-18 — active two-environment residual reduction after frozen C

20:19:30Z: qualified the missing stock KDE prerequisite, not a Lay fix.
The previous private IBus build disabled Wayland/UI; exact Chrome155 defaults
to native text-input-v3. Same pinned IBus1.5.34-rc2 now builds with GTK3/UI and
Wayland in a separate private prefix (BUILD PASS, ldd-r clean). Retain failed
generated-header/Vala-prerequisite attempts; the final continuation follows
the exact upstream VAPI recipe. Source11/11 PASS and fresh diagnostic review9/10
preceded knic. KWin launched its native panel, but setup stopped before actor
input because host1.5.26 GSettings lacks use-custom-theme. knic CLOSED, cgroup
empty, temporary policy absent, VT1. Do not rerun it unchanged. New knid stages
the matching pinned schema privately, preserving system fallback; source13/13
PASS. Review/native pending. GNOME capability package source36/36 PASS with
controlled RED2 baseline PASS ->2 mutant FAIL, fresh review9/10; native pending.
Exact evidence/preflight/source pins are in owner entry20:19:30Z. Full grids
remain GNOME354/46/32 and KDE175/233/24, runtime authority unchanged.

19:54:36Z: revised rich observer diagnostic remains UNQUALIFIED, Firefox0P3F
and Chrome0P3F outside original432 grids. Source19PASS+RED1EXPECTED/app,
review9/10. A seat-label guard caught the first zero-input attempt; corrected
letters-only labels and independent closure are qualified. Both field8 controls
reach all7 physical steps: initial raw empty differs from post-clear LF/caret0;
real Enter gives twoLF/caret1. Fields7/10 parent Text instead contains embedded
U+FFFC objects after Enter. Do not repeat the strict raw-equality assertion or
strip LF/FFFC blindly. Next: bounded native subtree/Text/Hypertext observation
with real-newline negatives; DOM only supplemental. Exact primary hashes,
both CLOSED/no keys/VT1 and limits are in owner entry19:54:36Z. No runtime edit,
new build, installation or original regrade; GNOME354/46/32,KDE175/233/24.

19:44:30Z: first rich diagnostic Firefox0P/3F/0B. All three physical
`hello ` observations are `hello\u00a0`/caret6; the diagnostic's raw ASCII-Space
collector failed before newline qualification. Initial empties exact; no
original90 clear-case conclusion. Chrome not launched with the known-invalid
collector. Seat CLOSED/empty/keys released/VT1. Primary/evidence archive and
unchanged matrices are in owner entry2026-10-18T19:44:30Z. Changed diagnostic
reuses existing positional rich-space observer under a fresh preflight;
no runtime authority or delivery change.

19:41:16Z: trace-observer experiment COMPLETE after fresh review9/10.
Remote guarded private-file controls3/3 PASS; same-inode nonshrinking
compaction defeats legacy offsets, immutable-prefix relation detects it.
Retained URL3 audit stays CAUSAL_WINDOW_UNQUALIFIED; neither absent events
nor producer completeness is inferred. Receipts under
`kde-causal-window-next/remote-results-a/`, top SHA
`2195580bf2cbcf2eae6c58b7a7c66b6f52467a83f44ada725a35039d25f6589f`.
No native input/runtime authority change or matrix regrade. Owner entry
2026-10-18T19:41:16Z records limits and subordinate hashes. Next: reviewed
six-field reversible rich diagnostic on exact frozen C/config/data bytes.

At19:35:35Z the user's target is at most20 FAIL per environment, on the same
fixed complete grids. Actual baselines stay GNOME354P46F32B and
KDE175P233F24B of432, NOT_TESTED0 each. Root owns the remote/native lease;
shared-runtime diagnosis and observer diagnosis proceed in two disjoint streams.
No runtime/build/native change yet. Read-only Chrome Backspace trace hashes
are verified; whole-word refocus proof is absent and may not be manufactured
from a generic SSI or a live bridge token. The20 SuggestOnly typo-undo cases
retain their recorded lexical authority diagnosis, with no literal model fit.
KDE raw-newline clear90 has a written six-field/seven-stage reversible control;
remote source controls7/7 per browser PASS, fresh review/native pending.
The38 selection/focus-return cases formerly labelled seed already reached
Double Shift; preserve all original verdicts. Trace compaction qualification
is separate from producer completeness and native correction quality.
Exact preparation, alternatives, evidence and next gates are in the owning
document's19:35:35 entry and cache `browser-final-fixes-20261018/`.

## 2026-10-18 — current frozen KDE Space-backend C checkpoint

As of2026-10-18T19:14:31Z, current source3260 PASS, final fresh Space-backend code review9/10
(two passes), private C build and four actual-engine/next-letter controls PASS.
Complete KDE J175P233F24B/432, NOT_TESTED0:27 F-to-P, one P-to-F versus E,
148/149 old PASS retained. Firefox URL3/moved_boundary regression remains
UNKNOWN, so KDE acceptance is not passed. Same-byte GNOME J354P46F32B/432
retains all353 preceding E PASS; one Firefox F-to-P, Chrome unchanged.
All eight finite sessions CLOSED. No install/publication, model/lexical or
text-delivery-route change. Exact receipts/hashes, latency coverage, failed
preparations/old verdicts and limits: [current architecture owner](../architecture/firefox-input-stability-2026-10-05.md),
frozen-Space-backend-stage-complete entry; cache `browser-final-fixes-20261018/`
`kde-space-matrix-j/` and `gnome-space-regression-j/`.

Preserve the implemented backend. Next qualify the retained regression window
and independent stock-ingress/rich-clear families under TD156/159, with a fresh
bounded written plan before further runtime edits. Native provider lifecycle
controls remain open in TD157/158. Historical entries below retain their
then-current verdicts; do not restart their completed source/build stages.

## 2026-10-18 — TD-156 first transition and rejected Reset-only repair

Follow-up: final code review 8/10 after two passes. Exact replay/capability
cleanup and Reset hide-failure retry now have entrypoint controls. New atomic
handover regression is causally RED at unsupported auxiliary clear in
`/home/ubu/.cache/lay/development/run-k8bw_66_/RESULT.json` (remote `run-ezO1J4`).
Guard that output by the existing transport capability, preserving authority
retirement and actual composition clear. The unchanged ordinary-stack isolated
native-transfer regression passes after heap-pinning the shared clear-output
future; pinning only the emitter was insufficient. Focused result remains
796P/38F/834: atomic RED plus 37 old presentation assertions at four shared
sites. Align typed-only preedit/auxiliary expectations without relaxing exact
edit, refusal or learning assertions. Full source and changed-byte native
acceptance are pending; original browser matrix remains 349P/50F/33B/432.

Owner: `firefox-input-stability-2026-10-05.md`, TD-156. Unchanged menu-90
Firefox field-1 original fifteen-plan diagnostic **11P/3F/1B** at
`firefox-legacy-native-20261012.6onP5c/firefox-functional-rvdztq1d/RESULT.json`.
Native seat closed 01:43:25Z, all owned processes gone and VT1 restored.
Pinned Firefox 157 TextComposition saves the last preedit before notifying
the IME and can restore it after an empty response. Reject synchronous
Reset-only clearing before implementation; no repeat without new evidence
addressing that saved commit. Plan review reset2/auxiliary8/lookup6 of10.
Bounded next route: standard auxiliary presentation for legacy candidates,
typed owned preedit preserved, same owner/edit delivery and Tab verifier;
atomic protocol unchanged/outside acceptance claim. Full preflight and new ADR
are recorded before production code. No local runtime install/restart.
CLI undo precondition independently remains SuggestOnly/common-L3 refusal;
never convert a timing change into an asserted semantic correction.
Full historical matrix **349P/50F/33B/432** remains; no new full native claim.

Source proof: `run-nk_vyipc/RESULT.json` records827PASS plus the intended
new RED at unaccepted legacy document composition. First implementation
`run-2ii4lknx/RESULT.json` passes new legacy controls but fails the existing
atomic pending-display publication failure control; runner status coverage is
also incomplete. Do not accept that run. Keep atomic failure semantics intact,
scope the new visibility/authority cleanup to auxiliary-capable legacy output,
and cover background publication/clear failures and inert pending retention.
Affected-closure verification and independent code review are pending.

## 2026-10-17 — PR53 explicit hot-material fixture replan

CI39074346533 passes the Reset repair but fails immediate publication after a
matching handled release at `e3557fb5`. Missing exact scene/input cache readiness
is a permitted source path, not a historically timed CI diagnosis. Runtime
correctly schedules background computation on a miss. Preserve the intentional
handler-before-observer release path; do not apply Reset choreography there.

The prior review budget is closed; explicit new bounded plan review9/10 groups
three hot-material fixtures using the existing cfg(test) readout contract and
production materializer. Assert unchanged input/frame, cache readiness, no
publication/learning; keep overlap's display checks and all cold/refusal controls.
Fresh827/827 remote source gate PASS. Private omission of release-time draining
is RED at the prepared fixture's publication assertion; production restoration
and9 exact hot/cold/refusal/Reset/timeout controls PASS. `READOUT-FIXTURE-PROOF.json`
records exact bytes, identities and scope. `READOUT-SOURCE-REUSE.json` byte-verifies
unchanged Rust/dependency/config outside the two private IME test files;
combining fresh827 with prior3211 unchanged targets is composed evidence,
not a new full remote run. Current-head full CI remains required.
The [owning entry](../architecture/firefox-input-stability-2026-10-05.md)
and task receipt directory retain both failed CI runs. No runtime/installed or
native authority change is proposed.

## 2026-10-17 — PR53 handler-first Reset fixture schedule

Baseline `8af0b6b` passes3211 remote source cases; GitHub run39071732596
loses the inert phase before actual SSI in the handler-first cursor regression.
Concurrent handler/observer scheduling against the production1ms deadline is
the hypothesis. Poll the real handler to Pending, complete the actual observer,
then resume the same handler; retain all assertions and production deadlines.
This changes test causality only. Focused827/827 and subsequent full3211/3211
remote gates PASS. Controlled removal of pre-Reset live-source capture makes
the regression RED at the original inert-phase assertion; byte restoration
returns PASS. Unobserved Reset refusal and absolute-timeout/no-waiter-leak
controls also PASS; no production change is retained. The ablation is not an
exact historical revision. Receipts `FOCUSED-PASS.json`, `FIXTURE-PROOF.json`
and `REPAIRED-SOURCE-PASS.json` retain exact source, identities and durations;
keep original failed CI and pre-repair source receipts. Final docs/graph refresh
is metadata-only; current-head CI is still required before merge.
The [owning entry](../architecture/firefox-input-stability-2026-10-05.md)
records scope and `/home/ubu/.cache/lay/development/pr53-serde-20261018/` evidence.
Installed89/runtime owners/native acceptance remain unchanged.

## 2026-10-17 — `ltkfq` refusal reproduced in current learned source-state memory

User redirected work from GitHub to diagnosis only. The exact layout target
is generated and initially eligible, but current feedback has negative
state-specific aliases for complete source `ltkfq`. L4 returns `Rejected`;
DecisionCore records `reason=rejected`; committed_tail.rs:140 consumes the
resulting `NoApply(Rank)`. The prefixed source has a different full-text state
identity and remains allowed. Its apparent first-word dependence is reproduced
with current user data, not by a numeric word-index runtime branch.

Private ablation of only those six source-state rejection aliases restores
the first-word decision while retaining generic negative word feedback.
Removing only `rejected_words[делай]` does NOT restore it. Stored weight8 is
not eight proven user interactions. No production code, guard, installed
configuration/model or user learning was edited; no physical input injected.
See [the complete diagnosis and immutable receipt scopes](first-word-learning-diagnostic-2026-10-17.md).
Browser default432 remains349P/50F/33B, NOT_TESTED0. No native fix is claimed.

## 2026-10-17 — autocorrection phrase diagnostic, installed 1.0.89 unchanged

User input: `Вклюай балансировку скорость точно лучш еубдет `.
Expected streaming result: `Включай балансировку скорость точно лучше будет `.
Two damaged spans contain three edits: missing consonant, misplaced Space and
adjacent transposition. The three intervening clean words must be preserved.

Read-only executable diagnosis ran on e@192.168.3.94 under the existing
dedicated-20cpu resource guard. CLI SHA256
`7ea210de19b96e1c03ccfc6ceed0948d4db035aec56e434d1568ea06eb2c7bfe`
matches the locally installed CLI89. The actual local configuration was copied
privately, SHA256
`20fe9fa76066463698ef4edee29733d9ce61bea7f97c901cf4fce22b03856bf5`:
`auto_replace`, `typing_assist` and `nanda_autocorrect` enabled, normal safety.
Existing remote L1.1 v9, L2 v13/DAFSA/productive v90 and all three phase model
artifacts were verified against the local installed model identities. Only the
private admission receipt's artifact/proof paths were relocated; original model
and quality-proof bytes were retained. Private feedback/usage is a copy of the
pre-existing remote snapshot, not asserted identical to current local learning.

Final authoritative diagnostic receipt:
`/home/ubu/.cache/lay/development/autocorrect-phrase-20261017/RESULT-READY.json`;
remote raw output and receipt:
`/home/e/projects/lay-development-runner/autocorrect-phrase-20261017-ready/`.
The private L1.1 service reported `ready` BEFORE all measured queries; its
PID was reaped in `finally`, with `service_closed=true`. Earlier receipts
`RESULT.json` and `RESULT-MATCHED.json` remain separate: the first lacks the
matching model environment, the latter admits queries before L1.1 readiness.
The first direct L1.1 probe returned `still warming` and establishes no lexical
verdict. Do not use those earlier attempts to diagnose missing lexical support.

Final six CLI cases: 3 expected behaviors observed / 3 unresolved, not native
acceptance and not a new browser denominator:

| Case | Final selected action |
| --- | --- |
| `Вклюай ` | No apply. L2 productive typed exact retains `Включай `, but admission is `SuggestOnly/short_internal_consonant_drift`. |
| `убдет ` | Selects `будет ` through the existing adjacent-transposition candidate. |
| `лучш ебудет ` | Selects `лучше будет ` through the existing boundary candidate. |
| `лучш еубдет ` | No apply. The boundary producer offers only `лучше убдет `; `edit_transition_not_verified` rejects this intermediate pair. |
| Entire reported phrase | Same unresolved final-pair transition; no apply. A single CLI resolution acts on the final tail, and cannot establish earlier per-Space correction of its first word. |
| Entire clean expected phrase | No rewrite selected; clean control preserved. |

First semantic limitations, separately from delivery:

1. Direct warmed L1.1 restore/lattice for `вклюай` abstains
   (`outside_calibrated_basin`) without retaining `включай`. L2 recovers the
   desired candidate, so its final loss of apply eligibility is the generic
   short internal-consonant admission guard. There is no grounded L1.1 Winner
   in this measurement that could justify bypassing that guard. Inspect general
   missing-letter support/authority evidence before proposing a runtime fix;
   never add this word or remove the guard to turn the example green.
2. The existing pair producer moves a boundary character but does not compose
   that operation with repair of the residual right token. Both individual
   operations succeed in controls; their desired combined final candidate is
   absent. The verifier correctly rejects `лучше убдет`, whose right token
   remains corrupt. Any future bounded composition belongs in the existing
   candidate/decision contract with one fully validated final pair, not two
   independently emitted IME edits and not a relaxed verifier.

Next discriminating evidence, before production edits: an actual remote IME
per-Space trace on the owned Kitty test field, with the same model identities
and explicit FieldSnapshotOnly scope. CLI FullReferenceAllowed, cold process
startup/explanation timings (~1.5 s here), direct L1.1 readouts and physical
client acceptance are different scopes. No UI latency or installed-client fix
is claimed. No production code, runtime authority, delivery route, installed
configuration, learning or model bytes changed; no new architecture promoted.
Latest complete native browser receipt remains Firefox170P/27F/19B and
Chrome179P/23F/14B, total349P/50F/33B of432, NOT_TESTED0; those layout results
do not certify this autocorrection phrase.

## V whole native result rejected

Full432339P60F33B, onegain/three lostPASS vsU; all20cursorPASS preserved.
Both scopes closed/VT1/all18primaries hashed. Do not publish or count sourcePASS
as acceptance. Source firstscope-loss proof next; review unavailable, no score.
[Owning evidence](../architecture/firefox-input-stability-2026-10-05.md) and V decision contain all precise receipts.


## V canonical53 PASS; proceed scoped source gate

Final signature cef52d3805060d185cc64a325e389215fb4d08c9c0a4da5b19f969a0f1e260a7,
23.901 s, all208 Rust hashes verified. Early KnownStart routing veto remains
rejected; new fixture checks actual source/effects, including spent-vs-live
provenance. See owning document and V decision. Native unchanged; task OPEN.


## V rejected early guard; no runtime rollback

Canonical source gate841/842 rejected early KnownStart contradiction veto; that
experiment was never built/installed. Original typed routing restored; inert
actual progress repair retained. V3=3 is routing, not accepted source. See owning
V rejection evidence and decision; adverse fixture now checks actual V2/hand-off/
replay/edit-effect boundaries. Do not reintroduce the rejected early gate.


## V focused53 PASS; task OPEN

53/53 source PASS, eight actual Reset schedules and ten adverse cases; independent
review unavailable (thread limit). See [owning evidence](../architecture/firefox-input-stability-2026-10-05.md)
and `2026-10-14-private-boundary-first-target-chunk.json`; native341/58/33 unchanged.


## V adverse handoff: separate baseline-existing contradiction located

The new progress positive test passes all four prefix/source-separator variants;
actual complete SSI confirms native delivery and the real ManualToggleV3 delegates
exactly once. Test-only final readiness now uses the established closed-boundary
contract (`!completed_native_replay_awaits_exact_receipt` + native ACK + actual RPC),
instead of predicates specific to an open UnknownStart suffix.

The adverse owner-change test still FAILs at lease handoff: current actual SSI
contains only partial replacement at the old caret, yet ManualToggleV3 returns
`(3,false)`. Controlled U baseline reproduces the same rejection assertion with
all three affected U production files hash-verified, 0 PASS / 1 FAIL, 13.666 s,
signature `9f13df4fc5ab2576f71ae36d6ee45420ff60e3eea7a828f292139214069b54e3`.
Temporary U source was restored to V in a finally block; private runtime unchanged.
This is a pre-existing admission gap, not a rollback of the retained cursor fix.

First lost guard: factory transfer keeps KnownStart while old native completion
scope is retired; `context_allows_manual_toggle` treats KnownStart as sufficient
despite an actual contradictory document/caret receipt. Recommended repair (9/10)
uses the existing actual-snapshot/owned-tail agreement predicate for this manual
admission; broad pending-replay transfer (7/10) is deferred; weakening or dropping
the adverse fixture (1/10) is rejected. Keep current token/reducer and final
verifier authority, active-preedit ownership, delivery routes and model bytes.

Receipts: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/V-U-baseline-handoff1-RESULT.json`
and `V-U-baseline-handoff-wrapper.json`. Native matrix remains 341/58/33 of 432;
no new native run and TD-121 remains OPEN.


## V first clearing call measured; minimal inert receipt planned

Remote source proof 50 PASS / 1 FAIL (51), signature
`c6e0ca85fbb9cd78a0b2c5677ba6c01af64c36e8963a9132e3d98316f5cd7452`,
21.513 s, all 208 Rust source hashes match. Actual erasure SSI is present;
`exact_replay_mirror_transition` clears it on the first replacement press.
The pending Reset receipt is absent at that point and is recreated after Space.
This explains why adjacent actual target growth cannot be established from the
current snapshot alone. Native equivalence remains a required later check.

Recommended option (9/10): one inert actual progress receipt in the existing
client observation state, bound to paired replay scope, actual SSI revision,
focus and layout generation, spent at the next adjacent SSI. Shared mutable
scope witness (7/10) has a wider equality/revision impact. Shape-only admission
(1/10) is rejected. Do not restore old client text as current or grant manual,
Space/Tab, learning or edit authority from the retained witness. Final fresh
full text/end-caret receipt, reducer provenance and verifier remain mandatory.

Receipt: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/V-first-clear-red51-RESULT.json`.
No model training, delivery route change, runtime installation or native input.
U complete native result remains 341 PASS / 58 FAIL / 33 BLOCKED (432), TD-121 OPEN.


## V source-only RED51: actual prior receipt lost before first target piece

Nativeprivate7 firsttargetchunk failure has a source choreography reproducer:
realerasureSSI→nativecompletedSpace→firstgrowingtarget/stationarycaret.
50oldcontrolsPASS/1newFAIL, initial21.6367s; afterprefailurestate assertions
same50P1F21.5750s sig a923a7c6939336982a91753b38534d7dc59eaaa29c1ff5eb50fc52d92f43045d.
Prioractual=None even though actualerasureSSI had beenreceived. Pending/current
boundarytoken match KnownStartlineage7/suffix0 and completedboundaryprovenance
with fullprefix is present. NoRootruntimechange yet: locatefirstactualsnapshot
clearingcall and exactfailedprogressconjunct beforechoosingminimalretention.
Do notinfer permissionfromprefix/geometry or weaken finalidentity/SafetyGate.
CoreU20cursor gains retained; full432341P58F33B is unchanged. GUIOFF/VT1;
sourceproofs/readsfmt executedREMOTEonly. Next test-onlydiagnostic may assert
snapshot after each actualnative key; no unchangedrandomnative retry.


## Current U full browser result:341PASS/58FAIL/33BLOCKED432

Both original9-job matrices completed on Uae225/cbc060; both exactscopes
closed/allfivePID-starttick pairs gone/VT8→VT1. Firefox162P35F19B216:
all10cursor stableIDs gainedPASS, oneRoldPASS lost(privateSpacefield7
rapid5stopsafterfirstflip), no otherchanges. Chrome179P23F14B216:
all216statuses equalR. Combined10newPASS/1lostPASS/9netPASS across432.
Bothbrowsers now20/20cursorcasesPASS; incomplete/no promotion because
58FAIL33BLOCKED and one requiredclass regressed. Keep provedcapture-order
repair; do not blindlyrollback or rerununchangedcaseuntilrandomPASS.
ProofSHAs:FF152185ead8273ea3cd266052c7b3e23743267761051c5d17837b70ce4cebaf39,
Chrome5e6679c4565f1f8bef57159b2ffb4900aeb2fbdcf9127fe88d769132f58128d7.
Speeds standard:p95FF118.806ms416/420, Chrome111.629ms406/420;
functionalDS141.682ms122/210and128.741ms158/210, missingexplicit.
Clockevdevwrite-return→exactAT-SPI, kernelreleaseNOTMEASURED.
Chrome3visitedLayfilesprivateunchanged/publicrecent_actionspositive;
tracecoverageUNKNOWN/no completeauditclaim. All18primaryactorJSON retained
in immutableRAWarchives; localcopy70516pendingthenverifyhashes.
Nextfirstsharedfailure: privateFFspace7 completednativeReplay intermediate
text4/caret0 rejects currentidentity before target7arrives. Exact331-event
packet retained`U-firefox-private7-first-burst-loss.json`; no rootcause
at specific source transition yet. Source840/48PASS retained; newcausal
proof before nextcoreedit. Freshindependentreview NOTRUN agentthreadlimit.
No modelrefit/localexecution/input/install/RDP/publication/TD121DONE.


## U full Firefox complete:162PASS35FAIL19BLOCKED216; Chrome running

All10cursor stableIDs gainedPASS againstR; exactlyone oldPASS lost:
standard/private/space/field7, rapid5 burst stops after firstnativeflip.
All216IDs compared; no otherstatus changes. Uoverall stillFAIL/no promotion.
FFsourcebytesae225/cbc060; original051–059all0/noheldkeys, exactfiveGone/
cgroupempty/VT1; primary9JSON preservedinto rawarchive immutablemanifest.
Proof`nativeae225-firefox-proof.json`, compare`U-firefox-vs-R-stable-ids.json`.
Standard416/420timings:p95118.806ms; panel48/48p95117.754ms;
functionalDS122/210p95141.682ms, Space37/50p9523.996ms, Tab/selection0/10.
Missingexplicit; evdevwrite-return→AT-SPI, kernelreleaseNOTMEASURED.
Chrome samebytes liveactual33036/driver32147, zero-keyUBscope3985161;
normal100 thenprivate101sinkbrackets thenoriginal102–108, activeafseat.
NoheavywhileGUIlive. Rcompleteboth432table332P67F33B retaineduntil bothU216
complete. Source840+48causal/adversePASS does not certify all nativecases.
One private7 regression remains firstnextmechanism afterChromefinish;
do not discard10cursor gains or rerununchangedcaseforrandomPASS.


## U scoped source gate PASS840/840; native acceptance pending

Fresh immutable`run-uA2KuH`, local`run-eorp9z31/RESULT.json`, archive
b4c98b96e437fa9c978b2da6f0f944659885fa000f44236661b4aded1776a88a.
IME789+sixcontracts51PASS; all737Rust hashes match. PriorRdaemon273 reused
on unchangedsource, no repeatedexecution; development scope, not release.
Source48 causal/adversePASS retained; U AST/privatebuild LIVE56245.
Root capture-order repair only, matchedRestore/finalmanualauthorityunchanged.
No newroute/owner/deadline/model fit; Rfull332P67F33B remains native baseline.
Freshcontextfree review NOTRUN due agentthreadlimit; no score invented.
T originalASTreceiptretained; later graph/report artifactsdrifted after edits/
hooks, localcopyhashassertionfailed. Do not claimT artifactverification.
Binding/generatedreceiptstillmatch; UfreshAST willbindcurrentRust separately.
Next fresh finiteownedSeat-af after oldT exactclosed/VT1, fulloriginal9jobs
Firefox/Chrome each withspeed/private-sinks. No localinstall/input or promotion.


## U exact causal RED confirmed; minimal receive-order repair pending proof

Source47:46oldcontrolsPASS/1newcaseFAIL,19.7413s, signature
1ba5e257c6b37f064cc49875e9f71ab3effa717b9399e4d704c3e35271fd1bc4.
ActualReset handler concurrentwith canonicalobserver loses inertphase at the
firstLeftReset, matching nativeT firstloss. All208Rust hashes verified before
codeedit. Root changed only capture_native_cursor_reset: existingexactlive
preReset identity OR existingobservedReset provenance can supply the inert
clone. Restoration stillrequires independentlyauthenticatedMatchedReset,
sameactivation/currenttoken/exactreduceroldsourceprovenance. Otheroldguards,
UnknownStart, manualonly/endcaret/actualSSI, readout/Space/Tab/learningrefusal
and everyroute unchanged. This repairs eventorder; no additional owner,wait,
timeout or wordcondition. Focused47/furtheradverse/sourcegate/native pending.
Rfull332P67F33B remains acceptance; no core success or promotion claimed yet.


## U first loss localized: handler-first Reset capture; causal source running

T same35withoutwire:standard20PASS, functional9P5F1B,cursorFAIL; both
actual83008/driver60771closed0, parent9279280/noheldkeys, exactfiveGone/CGempty/VT1.
Failure trace1178:phasepresent/identitycurrent/localtokenlive, awaitingLeft
actualeffect; resetspent/atomic/sensitive/sealed/quarantine allfalse. Capture
returnsNone BEFORE Reset11620 observer_ingress1180. NextLeft has no phase.
Allothercapture conjunctions proved; required reducerReset provenance was
queried before canonicalreceive recorded it. SuccessfulSpositive logging
allowed observer-first timing; it was not a functionalrepair. Exactpacket
`T-reset-order-first-failure.json`; tracecoverageUNKNOWN/bounded512000B.
New source47 fixture calls real handler-first Reset concurrent with real
observer (futurezip), no withheldreceiver/artificialtimeout. SourceRED before
coreedit. Minimal9/10option:inertclone from existingcurrentpreReset identity
or existingobservedReset, then restore onlyafter authenticatedMatchedReset
and unchangedreducerprovenance/sameactivation guards. OtherResetowners audited,
no blanketpredicate relaxation. Freshreviewspawn again rejectedthreadlimit;
NOTRUN/no scoreinvented. Rfull332P67F33B remains solecomplete acceptance.


## T failure-only diagnostics: source gate running, no core repair

S fresh15, SHnormal20+15withwire and SI same35withoutwire allcursorPASS.
The20normal history and passive wire absence did not reproduce RcursorFAIL.
Built-in positive-path logging can still change scheduling; fullRhistory
remains untested on diagnostic bytes. T removes successful-path diagnostic
calls, retains only records after original refusal/discard/capturefailure.
Original R predicates, mutation, owner/route and deadline remain; no refit.
Fresh scoped IME+6contracts gate LIVE8158, oldSIexactcgroupempty/5PIDgone/VT1.
SI actual75 was owned-model-server descendant cleanup after both actorparents0;
PID3738319/t14390320 stopped with authenticated pidfdSIGTERM; no unrelated
process changed. Receipt`SI-owned-orphan-cleanup.json`, strongSIclosure retained.
Scores/options/limits: `decisions/2026-10-14-cursor-failure-only-diagnostics.json`.
FullR332P67F33B remains acceptance; no partial PASS promotion.


## S native diagnostic complete:10PASS4FAIL1BLOCKED15; cursor PASS only

Original public Firefox field1 15plans +passive wire on metadata-only S167ca/37082
completed. Exact five processes gone, cgroupempty, VT8→VT1. Cursor phase
seeded, allactualreceipt transitions preserved and exacttargetPASS. This is
not a full R-state-history replay or a functional repair: retained R full
cursor10FAIL remains. AllfourRShiftcallbacks in the failing excerpt were
admitted; the invalid source Shift-timeout fixture is not that native cause.
OtherS failures:prefix/undo/selection/focus, TabBLOCKED. DiagnosticDS12/21
p95142.068ms, Space4/5p9521.512ms; missingexplicit. Observer/logging overhead
NOT_MEASURED, asynctracecoverageUNKNOWN, bounded512000Bprefixnotpreserved.
Actor+wire hashes verified/local`S-diagnostic-field1/{RESULT.json,WIRE.json}`.
First failed R cursor-phase transition stillUNKNOWN. Compare retained states
before any new behavior change. Rfull332P67F33B remains the acceptance table.


## S diagnostic source gate PASS837/837; native pending

Immutable remote gate`run-QyJfLT`, local`run-9xuz3ad5/RESULT.json`, archive
925cb3c43883f4a97a7e4a7fb3b12102626811fdb9997f5a0201fa885338bb64.
IME786+sixcontracts51 PASS, all737Rust hashes match; development scope, not
release. R daemon273 evidence retained on unchanged source; not re-executed.
Remote AST/private build sequential LIVE69800. Native diagnostic not yet
started; GUIclosed/VT1. R complete332P67F33B remains sole complete baseline.
Metadata logging only, no new ownership/edit/delivery/training authority.


## S45 coverage PASS; scoped metadata diagnostic pending

Corrected source45/45 PASS19.0167s; all208 source hashes verified, no runtime
fix established. Existing trace switch now records cursor phase/grants,
actual scalar caret/count, last key and Reset capture/restore/discard.
Numeric/boolean metadata only, no word/token dump or new delivery/admission
rule. Two source files only; existing conditions and mutations unchanged.
One original public Firefox field1 diagnostic follows a fresh scoped source
and immutable private build. Full332P67F33B remains R acceptance. Diagnostic
overhead/async writer completeness UNKNOWN. Fresh context-free review spawn
was rejected by agent thread limit; no review or score is fabricated.


## S RED interpretation corrected: artificial Shift stamp timeout

The additional assertions locate loss at the first handler-first Shift
release. The helper deliberately waits for the Shift callback to return
before running ingress observation. Shift is excluded from text-inert
completion by its protected ordered-admission contract. Withholding the
observer past that deadline causes expected refusal; this RED does not prove
a runtime defect or the Firefox root cause. Keep all three RED receipts.
Corrected source fixture uses existing received observer-first Shift callbacks;
original movement/insert release-order variants remain. No production edit.


## S source-only causal RED:44PASS/1FAIL45

Actual prediction publication plus four received Shift callbacks reproduces
loss of cursor manual-source authority on unchanged R production source.
Receipt: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/S-causal-red-RESULT.json`;
signature6928aa83a832ba5a3db159add7d2261b0155e6b41590bff649b3adfe11211c04,
18.8101s. All44R controls passed. First exact event not yet localized;
assertions added before/after Shift press/release before runtime edits.
This does not change the native332P67F33B table or establish a Firefox fix.
Both GUI scopes closed/VT1, no keys running. R10Chrome gains retained;
no model refit, local execution, delivery-route edit or installation.


## Current R browser result:332PASS/67FAIL/33BLOCKED432

Both original9-job matrices completed on R64dfd/20adb; both exact scopes
closed/all5PID-starttick pairs gone/VT8→VT1. Firefox153P44F19B216,
all216verdicts equalQ. Chrome179P23F14B216:all10cursor scenarios gained
PASS, zero lostPASS across216stable IDs. Combined10newPASS/0lostPASS
across432, no promotion because67FAIL/33BLOCKED remain. This is a native
Chrome improvement, not a Firefox fix or model retraining.
Chrome receipt native64dfd-chrome-proof.json SHA
acf2ea807da3db679563bccf53f7eb17c82cb4c3e1ee25ead7a7ca2e9ac783d1.
Firefox a6747721a0a3d14daf281eb9a909c79ccdde73fcdd559f5a85cb855ead111a76.
Standard:p95Firefox118.867ms420/420, Chrome110.429ms406/420;
functionalDS Firefox142.074ms100/210, Chrome126.096ms158/210.
Space37/50and38/50, Tab0/10each, selection0/10FF10/10Chrome.
Allmissing remainexplicit;evdev write-return→exactAT-SPI, kernelrelease
NOT_MEASURED. Scopedprivate writer proof:3visitedfilesunchanged, public
recent_actions positive; tracecoverageUNKNOWN, no universalauditclaim.
Source1109/1109+review9 retained; R mechanism stays as next baseline.
Next: prove the first native Firefox cursor transition lost in R; source
fixtures currently omit actual visible prediction/gesture/interleaving.
No unchanged failed input retry, no core rollback, no production installation.


## R full Firefox native FAIL153PASS/44FAIL/19BLOCKED216

Original9 jobs finished0/noheldkeys. Exact scope closed, all5 PID/starttick
pairs gone, VT8→VT1. Full R Firefox receipt `native64dfd-firefox-proof.json`
SHAa6747721a0a3d14daf281eb9a909c79ccdde73fcdd559f5a85cb855ead111a76,
localbdir:standard60PASS, panel6PASS, functional87P44F19B. All216stableIDs
match Q:0newPASS/0lostPASS. Native cursor repair remains unproved/FAIL.
Standard420/420 timing samples:p95118.8668ms; functional DS100/210
p95142.0744ms, Space37/50p9526.0197ms; Tab/selection0/10, missingexplicit.
Clockevdev write-return→exactAT-SPI; kernel releaseNOT_MEASURED.
Raw native64dfd-firefox-raw-1791942539350123327 plus all9primary JSON
receipts retained unchanged with supplement. Source1109/review9 remain
separate. NextChrome original9 onR, then causal source/live boundary work.
No promotion, production installation, core rollback or unchanged retry.


## R current native partial: cursor still FAIL, no promotion

New R bytes pass Firefoxstandard60/60 andpanel6/6. Original functional
fields1–2 give18PASS/10FAIL/2BLOCKED30; both cursor cases still refuse the
first exact Double Shift although actual source and caret match. This is
not a native fix. Source1109/1109 and review9 are retained separately.
Excerpt `R-native-cursor-first-failure.json` under localbdir retains the
capped async trace and precise source/candidate binding. It does not log
CursorEditing presence or prove its first exact loss. At firstLeft there
is a visible display prediction, full actualSSI6/caret6 and UnknownStart;
manual RPC later refuses context_authority. No new hypothesis is promoted
without a causal source schedule. Finish original full matrices; no retry
or actor changes and no core rollback. Full Qtable stays322P77F33B432.


## R final source gate PASS1109/1109; native acceptance pending

Focused44/44 and fresh eight-target1109/1109 PASS, immutable remote
run-wznQ1e, archive `0e127076801a8318e4de8fb4d35f3089087f69f0afd42748eaafb5155a6e5e62`. Local receipt
`/home/ubu/.cache/lay/development/run-6x6vdp47/RESULT.json`; all737Rust file hashes match this checkout.
Final independent review2/2 closed9/10; no third review. Old failure receipts
and deterministic six-ID registration RED remain retained. The tested fixture
correction uses the existing exact dispatcher Ping barrier; runtime routes
are unchanged. Next: remote AST, immutable private build and full original
Firefox/Chrome9jobs each with latency/privacy. TaskOPEN; Qfull table remains
322PASS/77FAIL/33BLOCKED/432 until new native matrices complete.


## R current next gate

Final read-only review2/2 is completing. Registration causal RED is retained
and the existing strict dispatcher barrier now covers Cycle09 registration
and the interior fixture's transition to metadata. First barrier44 passed
all old controls/erasure and found one queued exact metadata reply; that
follow-up uses the same barrier, not a runtime or timeout change. Final
focused44 running; then fresh broad gate. Prior full failures remain retained.
No candidate build/staging or changed-byte native result.

## R fixture registration handoff: causal RED six IDs

Second broad1108/1109 failure is retained: manual V3 erasure refusal despite
valid snapshot/revision/epoch and exact authority. New registration oracle
proves six focused fixture IDs entered engine registration with unfinished
exact absent-object callback replies. Erasure:Reset63011+release63012; cursor:
End release95047. Receipt `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/R-registration-causal-red/RESULT.json`,38PASS/6FAIL/44. This proves
the completion gap; the historical receipt does not prove which callback was
auto-dispatched again. The strict existing standard Ping barrier is now reused
by Cycle09 manual/visible/private-visible helpers before object registration.
No runtime change or weakened expected verdict; focused44 running, then a
fresh broad gate. Final read-only review pass2/2 is running on concrete code
and this evidence. Private Q bytes and full browser table remain unchanged.

## R full source first gate:1108/1109, guard preserved

First broad R receipt `/home/ubu/.cache/lay/development/run-sgdcr974/RESULT.json`
(remote run-QFIH5m, archive f77d37cdee89ccc659cd2b403452e4a5c58144609104e3525d50a9ca1b37c00d)
executes1109, passes1108, fails one existing navigation safety test. The
explicit manual gesture correctly refused an unconfirmed/interior cursor
source but did not spend its inert pending phase. Keep the existing test
unchanged: that refusal now discards only the cursor phase. It cannot be
revived by a later SSI. Focused43 running; fresh full gate and final review
remain required. No build/native staging or authority change.

## R focused41 GREEN; full source and native still pending

All eight authenticated cursor re-edit positives now PASS in both release
orders, with empty/prefixed and restored/changed text. Q38 prior controls and
two purpose/adverse-receipt tests also PASS:41/41,18.453s. Receipt
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/R-first-green/RESULT.json`. The remaining handler-first failure came from normal
Reset cleanup publishing a new tail epoch before the inert receipt was rebound.
The same existing pending slot is now rebound before cleanup; no release
settlement, word-start grant or extra owner is invented. Generic readout,
Space, Tab and learning remain excluded until explicit manual consumption.
Additional identity/metadata checks and full source gate are pending. No new
private build/native run: full Q table remains322PASS/77FAIL/33BLOCKED.

## R cursor provenance: causal RED and unwired draft — 2026-10-14T00:46:00.937428+00:00

Actual cursor trace first loses locally owned suffix on Left, before End. The
source predecessor is UnknownStart with an existing exact manual grant; this
experiment does not grant KnownStart. Authenticated source RED39:38 prior
controls PASS, one new test FAIL across all eight prefix/edit/order variants.
Observer-first refuses authority; handler-first additionally refuses admission.
Receipt: `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/R-causal-red/RESULT.json`, proof signature
`f0f6fef006b6ddfdd9a5022092778a87d920d2879b320b81ddfc82a40eba6fc0`.

Local CursorEditing helpers occupy the existing pending receipt slot. They are
unwired, uncompiled and uninstalled. Plan review1/2 scored8/10; final review2/2
remains reserved for verified implementation. No general suffix/Space/Tab grant
is permitted. Current installed private test bytes and full432 table remain Q
322PASS/77FAIL/33BLOCKED. GUI is closed; laptop runtime authority unchanged.
Next: wire exact authenticated key effects, actual SSI, matched Reset and
manual-only consumption; reject stale or contradictory receipts.


Обновлено: 2026-10-13T23:24:26Z. Checkout: `/home/ubu/projects/lay-space-boundary-shift-20261007`,
ветка `codex/space-boundary-shift-20261007`, HEAD `d362e7c82cb14f3d2aab123b9130707f1843ba02`.
Владелец архитектуры и полных доказательств:
[firefox-input-stability-2026-10-05.md](../architecture/firefox-input-stability-2026-10-05.md).
Этот журнал индексирует эксперименты и определяет порядок следующего шага;
полные журналы, исходники и измерения остаются в указанных receipts.

## Текущая опорная версия

**Q — SOURCE PASS / BOTH FULL BROWSER MATRICES FAIL / NO PROMOTION.**
Неизменный source gate: `/home/ubu/.cache/lay/development/run-61bj0v7o/`,
remote `run-cht3Zw`, archive SHA-256
`e1f0e3864aeb3f1f9669270960b3b0119367ce2e130d17f5f7a5214a56adf8f5`.
Focused38/38 и свежие восемь development targets1105/1105 PASS;
два независимых review9/9, remote AST734 Rust hashes совпадают.
Это выбранный development scope, не весь репозиторий и не релизный PASS.

Текущие private test bytes:
IME `399ffa890002bc6ce55b869bc30138f77656574b55be173fd59c91d37f7a5291`,
daemon `c49c44cb7776051433fc7d77ad0d5ac4d5f78f04b402895ffe1363950c3e3d97`.
Оба GUI закрыты: exact scopes empty, исходные PID/starttick gone,
VT8→VT1. Локальная установленная версия не менялась.

Q Firefox: standard60PASS/60, panel6PASS/6,
functional87PASS/44FAIL/19BLOCKED/150 — 153PASS/44FAIL/19BLOCKED/216.
Все216 stable IDs сравнены с O: один новый PASS (private/active/field1),
ноль потерянных PASS; остальные215 verdicts совпадают.
Diagnostic private20/20 PASS сохранён отдельно от исходных девяти jobs.
Standard speed420/420 measured, p95 120.982ms; kernel release NOT_MEASURED.
Первичный receipt: `native399ff-firefox-proof.json`, SHA-256
`93af9b30e3faf40622f4c6bda2dee877b5766ec4b6eed1dd35ad762753b72de6`.

Q Chrome:169PASS/33FAIL/14BLOCKED/216; все216 stable-ID verdicts совпали с O.
Standard58PASS/2BLOCKED, panel6PASS, functional105PASS/33FAIL/12BLOCKED.
Standard speed406/420 measured, p95 113.861ms;14missing остаются явными.
Primary receipt `native399ff-chrome-proof.json`, SHA-256
`73a88c47a24bff2d7d54cff2e9db09bec3f06865f8f23f5d74f926a0fd3d1365`.
Private writer-root scoped proof PASS: три посещённых файла не изменились;
public positive control изменил recent_actions.jsonl. Trace coverage UNKNOWN.
Текущая полная Q таблица:322PASS/77FAIL/33BLOCKED/432;1новыйPASS/0потерянных.
Receipts и stable-ID comparison лежат в
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/`.

## Реестр последних экспериментов

| ID | Гипотеза / изменение | Проверка | Решение и запрет повторения |
|---|---|---|---|
| M | При действительном replay текст может опережать неподвижный caret; сохранить только строгий парный прогресс до точного конца | causal RED, focused5 PASS, включён в N1092; отдельный native M не заявлен | Сохранить механизм и отрицательные проверки. Не расширять произвольный caret lag |
| N | PRIVATE запрещает обучение/историю, но разрешает явный Double Shift в обычном поле | focused19 PASS, full1092 PASS, Chrome-private20 PASS + реальные writer roots; полные browser matrices FAIL | Сохранить разделение приватности и защищённых полей. Не возвращать общий запрет Double Shift по PRIVATE |
| O-R0 | Уже подтверждённое слово теряется, если managed release обрабатывается раньше observer | causal3:1FAIL/2controlsPASS; `o-late-managed-release-red-ad5ufwag/RESULT.json` | Причина воспроизведена. Старый тест closed-word/preedit не заменяет active managed-commit repro |
| O-R1 | Любой plain release текущего владельца сделать Passive, cleanup обработать раньше ожидания stamp | focused17 PASS, но full1096:1081PASS/15FAIL; `run-epln3f0l/RESULT.json`, remote `run-WE0tvu` | **REJECTED.** Ломает обычный release fence и ordered replay settlement. Не повторять широкую Passive-классификацию. Код сохранён отдельно; ни сборки, ни установки |
| O-R2 | Сохранить обычный Key/fence; только факт уже завершённого подтверждённого release хранить в существующем bounded stamp store и поглощать при точном позднем ingress | Source1101 PASS, review9; full Firefox152P45F19B/Chrome169P33F14B | Сохранён узкий механизм; source PASS не закрывает native FAIL |
| P | Поздний release после подтверждённого Reset должен завершить уже обработанный press, сохранив ранее полученные context metadata без восстановления old word authority | Focused33/full1104 PASS, review9/9; private diagnostic19P1F | Сохранён. Первый новый отказ — точный acknowledged NBSP потерян при held-Space Reset; полный native P не повторяли после диагностического FAIL |
| Q | Сохранить точное acknowledged представление owned Space, включая NBSP, через существующее одноразовое held-Space Reset | Focused38/full1105 PASS, review9/9; diagnostic20P; full Firefox153P44F19B | Сохранён; Chrome Q следующий. Никаких новых owner/route/таймеров или послаблений proof |

## Отвергнутые решения

- Увеличить stamp timeout: маскирует порядок обработки и добавляет задержку;
  не доказывает право на слово.
- Повторять ввода/refresh до случайного PASS: не исправляет первый переход,
  меняет сценарий и знаменатель доказательства.
- Восстанавливать KnownStart по наличию буфера или handled-cache: cache не
  доказывает текущего владельца, содержимое и границу слова.
- Делать observer Passive для всех ordinary releases до исполнения callback:
  отвергнуто полной source-проверкой O-R1, даже при focused17 и review9.
- Ослаблять SafetyGate, edit verifier, protected fields, точный snapshot или
  добавлять исключение по слову/браузеру: не является исправлением механизма.

## Порядок работы, предотвращающий круг

1. Перед правкой прочитать этот журнал и найти первый потерянный переход.
   Записать отдельный ID, точную опорную версию, изменяемый механизм и ожидаемый
   результат. Не начинать второй эксперимент одновременно.
2. Отвергнутый механизм повторять только с новым причинным доказательством,
   которое опровергает прежнюю причину отказа. Указать ссылку на него до правки.
3. Сохранить исходники и receipts каждой неудачи. Возвращать только изменения
   текущего отвергнутого эксперимента, сохраняя проверенные M/N и прежние fixes;
   не откатывать ядро целиком и не использовать dirty main как базу.
4. После узкого causal/controls PASS включить **все ранее упавшие проверки этого
   механизма** в обязательный набор. Затем пройти весь прежний affected scope,
   сохранить совпадающие/новые/потерянные PASS по стабильным ID.
5. Code review и native build после общего source PASS. Review score не заменяет
   проверку. Лимит review каждого описанного эксперимента — максимум два прохода;
   доказательства отвергнутого варианта не переносить на другой код.
6. Native на точных новых байтах: исходные Firefox/Chrome60+6+150, скорости с
   required/measured/missing, реальные writer roots для приватности. Source PASS
   не закрывает native FAIL/BLOCKED/NOT_TESTED.
7. Менять статус на DONE/commit/push только после обязательных доказательств.
   Перед передачей обновлять текущее состояние и ровно один следующий шаг.

## Следующий шаг

R-D0: на неизменных Q bytes выполнить один bounded public Firefox field1
диагностический corpus15 с неизменным actor/body и пассивным exact IBus wire
capture, включая FocusIn/Out, Reset, SSI, key callbacks и output signals.
Это diagnostic, не новая полная матрица. Обычный trace512KiB ротационный,
prefix_preserved=false, поэтому он не доказывает начало всех старых отказов.
Нужен первый измеренный переход для общих focus/cursor failures; лишь затем
варианты решения/score/recommendation, causal source RED и review. Не менять
preedit mode по гипотезе: текущий source уже отправляет CLEAR=0. TD-121 OPEN.

### Исторический следующий шаг O-R2

Закончить source draft O-R2 и его отрицательные проверки: observer-first fence
сохраняется; handler-first matched managed/native release не оставляет orphan;
completion одноразов, связан с полным header и key tuple, ограничен старой
capacity и не даёт прав после FocusOut/Reset/ContentType/owner replacement.
В новый focused набор обязательно включить все 15 упавших строк O-R1, затем
пройти тот же общий development scope N. GUI остаётся выключен до source PASS.

## 2026-10-13: проверка повторов и первый O-R2 source verdict

Проверены22 сохранённые candidate BUILD с неизменными source request:22 разных
отпечатка файлов IME, точных повторов0. Исключены отдельные test paths; inline
tests/comments могут оставаться в файлах. Это проверка точных байтов, не
семантической идентичности и не всех промежуточных незаписанных правок. Receipt:
`core-built-candidate-repeat-audit-20261013.json` в текущем evidence cache.

O-R2 первый source focused28:27PASS/1FAIL, 16.835374405s,
`o-completed-release-source-okgy8u25/RESULT.json`. Все ранее упавшие O-R1 test IDs
проходят. Единственный новый test incorrectly demanded identical generation
after a Reset whose local scope/metadata was not eligible for the fast path.
Требования разделены без изменения runtime: отдельный actual admitted
UnknownStart positive сохраняет точный token/scope; unproved post-Reset negative
не получает KnownStart или manual authority. Старые guard tests не изменены.
Focused29 и общий affected scope ещё не проверены; не начинать native/review.

### 2026-10-13T19:39:31Z: точный повтор найден и автоматический stop проверен

`o-completed-release-source-lw6i8hi9` и `...-y1q6d2oi` имеют одинаковые
source hashes и29 выбранных тестов; оба28PASS/1FAIL. Причина повторного запуска:
локальная замена assertion остановилась на ошибке, но отдельный следующий shell
процесс всё равно передал прежний файл и запустил тесты. Это ошибка исполнения,
не новая гипотеза. Она сохранена явно, не скрыта аудитом22 готовых сборок.

Редактирование и передача теперь выполняются одним Python orchestration: ошибка
редактирования останавливает последующие шаги. В текущий remote source helper
добавлен fingerprint205Rust source + точный набор тестов и stop для неизменённого
FAIL. Старые первичные receipts не переписывались; signatures хранятся sidecar.
Проверка стопа на заведомо прежнем source map: `...-v05x8q54/RESULT.json`,
`STOP_REPEATED_FAILED_SOURCE`, новых тестов0, до компиляции. Это self-test guard,
не новый тест ядра. Helper:
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/o-completed-release-source-probe.py`.
Новый исправленный тест требует закрытый bridge для неподтверждённого позднего
callback; допустимые managed/native/UnknownStart positive сохраняют no-orphan
и exact token/scope. Runtime код этой коррекцией тестов не менялся.

### 2026-10-13T19:40:35Z: O-R2 focused29 GREEN

29/29 PASS, 16.580660436s, source proof_signature
`605223c4c965ebf5cc94a386237f614601cfe743988b94e71a9d22348f1ba7ac`, primary
`o-completed-release-source-49f0u4qb/RESULT.json` SHA-256 `54110681c04193e4a2704f885199b166dee6b259cf9240cd21c4e79510d0a17e`.
Все8 ранее упавших стабильных test IDs O-R1 включены, исходные guard tests
не изменены. Допустимые managed/native и UnknownStart сохраняют точный token,
scope, epoch и не оставляют orphan; неподтверждённый post-Reset callback сохраняет
закрытый bridge. Fetch9files включает7 форматированныхRust + manifest3143 и
known_failures; все205Rust SHA совпадают. Общий affected source scope pending;
review/native не начинать до его PASS.

### 2026-10-13T19:46:44Z: O-R2 full development GREEN

1100 selected/executed/PASS,0FAIL,259.9s. Receipt `/home/ubu/.cache/lay/development/run-5erfwyt3/RESULT.json`,
remote immutable `/home/e/projects/lay-development-runner/run-CdwrRH`, source archive
`f30df6eefaee323eff17e0ec81acae2ec8fe7ffa93efbe7d90284870aa1f20de`. Same eight explicit targets as N; FMT PASS.
This is the full recorded affected development scope, not entire-repository
release acceptance. All former O-R1 source regressions remain in the gate.
Current replacement review1/2 starts only now; no native build/acceptance yet.

### 2026-10-13T20:03:16Z: review1 proof issue fixed, production preserved

Review1/2 score8, no confirmed production defect. Authentication negatives used
the same serial as their positive and could be masked by duplicate rejection;
now each row has its own complete HeaderKey. Focused29 refreshed PASS in
`o-completed-release-source-qx3flqni/RESULT.json`. Dirty completion original
assertions now run observer-first and handler-first; a new actual queued
Reset-before-release/handler-first schedule proves old token stays revoked and
late completion leaves no orphan or restored word. Focused30/30 PASS,
16.818284944s, `o-completed-release-source-esjw0_ey/RESULT.json`,
SHA `1cd6b66ba80eaf10b45bcfc5a255d4993ff0eca9b9795de64145a4a9bba9504f`, all205Rust match. Four production files unchanged.
General source refresh needed for new fixture set, then final review2/2.
Previous1100 source receipt worker time is251.6305s;259.9s was controller wall,
not worker latency. Neither is native response time.
Dependency: current zbus async-io observer and callback share the same single
executor thread and ingress apply/refresh/publish has no await. Cross-owner
replacement still has store-level proof; actual Reset schedule now added.
Physical matrices/speed/private-write proofs remain NOT_TESTED for O-R2.

## O-R2 supplemental full1101: fixture FIFO failure retained

`run-9fun3w7c/RESULT.json` / immutable remote `run-bFjOId`:1101 executed,
1099PASS/2FAIL, controller255.2s. The new lifecycle test expected reply92501
but got the earlier92500; handler-first dirty-preedit expected an Error reply
but received its genuine Signal first. Both are raw test-peer FIFO assumptions,
not a demonstrated runtime defect. Focused30PASS does not override fullFAIL.
The new release helper now uses the established serial-aware next_peer_message
filter via the normal output collector; it does not synchronously consume one
raw message or discard signals. Lifecycle and new post-Reset fixture use the
same reader. Production4files untouched, no build/native yet.

### 2026-10-13T20:19:12Z: refreshed source GREEN with unchanged daemon proof reused

After the fixture FIFO repair, focused30/30 PASS16.868418486s in
`o-completed-release-source-0omlkfgl/RESULT.json`, SHA
`eb01a2cdb243ec395c1bfcc17a943e346cca2adf65194abc96278ef03989bf31`.
Fresh full IME bin and six integration contracts:828/828 PASS, worker
104.381220s, local `/home/ubu/.cache/lay/development/run-1t0exf6o/RESULT.json`, immutable remote
`/home/e/projects/lay-development-runner/run-T9RNFT`, archive
`c08324a27694bf79ab515f4cf20b8d61a9ca4f94384168839f55fa1b18d6f483`. Daemon273 passed in the prior eight-target
`run-bFjOId`; its relevant source/dependencies/configuration are unchanged.
Binding receipt: `O-R2-daemon-reuse-binding.json` in current evidence cache.
Combined recorded affected scope is fresh828 plus reused273; this is not a
fresh1101 run or entire-repository release acceptance. Prior full1099PASS/2FAIL
remains FAIL on disk. Four production files unchanged since first full1100GREEN.
Review2/2 now permitted; build/native are pending, N private runtime unchanged,
GUIOFF, laptop unchanged. TD121 remains OPEN.

2026-10-13T20:25:34Z — O-R2 final independent code review2/2:9/10, no blocking production
defect, build/native permitted. Reviewer `/root/completed_release_final_code_review_20261013`
independently verified immutable source and reused daemon identity. Owner generation
replacement remains store-level proof; actual Reset-before-late-ingress tested.
Fresh AST/architecture PASS46.5811s, `O-R2-graph-1791922806460946036.log`,
4 graph/receipt artifacts fetched and hash-verified. Runtime Rust unchanged.
New native/latency/private-writer proof NOT_TESTED. Next immutable build from
`run-T9RNFT`, then private staging/original browser actors; TD121 OPEN.

2026-10-13T20:32:43Z — O-R2 immutable build PASS183.408107s from current
`run-T9RNFT`; IME `c99b31aaa4a37309bc56747c1edcfc5bad5004c010d76da054d0868ab566acce`, daemon
`6e67bfaed56891dec88cc0520a819ac9954a572ac360b2da187c811346be88aa`. Staged only in remote private RUN,
verified N backup `staged-before-O-text-inert-release-1791923355841483469`.
Original actor hashes unchanged. Fresh finite seat-z/owned keyboard ACL and
zero-key PID/tick/executable/CG attachment proof validated; original Firefox
051-059 queued216 required. Scope `0::/user.slice/user-1000.slice/user@1000.service/app.slice/lay-verify-1000-2527394.scope`.
Native semantic verdict, speeds and O private writer proof pending; no promotion,
laptop unchanged, TD121 OPEN. Controller seat activation initially used an
unsupported verb; corrected to `loginctl activate` before GUI/input. No source
edit/build/test repeat occurred because of that controller error.

2026-10-13T20:46:23Z — O-R2 Firefox full native FAIL retained. Standard59PASS/1FAIL,
normal20PASS/private19PASS1FAIL(active field1)/fullscreen20PASS; panel6PASS.
Functional150:87PASS/44FAIL/19BLOCKED, no NOT_TESTED. Source bindings and original
actors unchanged. Primary `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/nativec99b3-firefox-proof.json`; raw
`/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/nativec99b3-firefox-raw-1791924302611571081/RAW.json`. DS414/420 measured, median105.260/p95118.295/max133.731ms;
functional DS101/210 and Space37/50; Tab0/10/selection0/10. Clock evdev write-return
to exact AT-SPI target; kernel release NOT_MEASURED. Scope2527394 closed, all bound
PID/ticks gone, original VT1 restored before assembly. New private1 failed reverse
after probe removal; trace Reset reason missing_predecessor_or_post_reset_token,
live bridge token but manual authority false. This is measured sequence, not yet
a source causal root. Preserve O-R2 and preceding source fixes; do not repeat O-R1
or weaken authority. Next Chrome same exact bytes, then source causal proof for
remaining native sequence. No promotion/commit/push/laptop change; TD121 OPEN.

2026-10-13T21:02:42Z — O-R2 Chrome full native FAIL retained:58PASS/2BLOCKED standard,
private20PASS, panel6PASS; functional105PASS/33FAIL/12BLOCKED. Blocked seed fields
normal/fullscreen Space are field2 in both O-R2 and N; all216 case statuses match N.
DS406/420 measured median91.789/p95109.825/max124.717ms; functional DS138/210,
Space38/50, Tab0/10, selection10/10. Clock evdev write-return→exact AT-SPI target,
kernel release NOT_MEASURED. Primary `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/nativec99b3-chrome-proof.json`,
raw `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/nativec99b3-chrome-raw-1791925219090151196/RAW.json`. Actual private20/20 and3visited writer-root files
unchanged; enabled public positive recent_actions.jsonl changed. Four quiescent
snapshots, no extra audit input; `O-chrome-private-sink-proof.json`, trace completeness
UNKNOWN. Owned scope2584204 closed/all5PIDticks gone21:00:18UTC; VT1 restored.
Both original216row browser matrices remain FAIL. Retain source30/fresh828+reused273
and two reviews(8,9) separately; they cannot close native authority/functional
failures. Next source causal reproduction of managed append/native backspace/Reset
missing predecessor using existing reducer/real headers, then minimal mechanism
repair. No whole-core rollback/broadPassive/newowner/route/guard weakening.
TD121 OPEN/no promotion/commit/push/laptop change.

2026-10-13T21:16:21Z — P causal RED:1FAIL/2controlsPASS, 13.272402s,
`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/P-source-probes/p-confirmed-release-source-d_r20233/RESULT.json`, all205 current Rust hashes match. New actual managed press,
Reset, exact full SSI, handler-first matched release reproduces lost authority;
observer-first control reaches expected state. Production O-R2 unchanged. First
live failure is release10896 refused_timeout plus later orphan Key, before the
Backspace symptom, correcting the earlier downstream-only diagnosis. Difference
from original simple O test: actual Reset/confirmed receipt between press/release.
Decision2026-10-14-confirmed-reset-owned-release.json:9/10 current-owner handled
zero-effect completion without unrelated local word-scope equality;6/10 rebind
word scope;2/10 longer timeout. Terminal scope guard, unpaired fences, current
owner/metadata/one-shot bounds preserved. Plan review1/2 pending; no runtime edit
or new native claim. Next strengthen exact same source case through Backspace,
then implement the minimal eligibility distinction after independent review.

2026-10-13T22:03:12Z — P33 source30PASS/3FAIL, `p-confirmed-release-source-48tbgg95/RESULT.json`,
16.826417s. New independent reducer unit proves word-only Reset clears admitted
field metadata; original pending confirmed receipt republishes it only on a later
manual request. One eligibility predicate cannot repair this alone. Decision P
now retains existing field attributes only in observed_word_reset; all other revoke
paths unchanged, None never becomes Some, word token/lineage still revoked. No
new owner/cache or input transport. Original observer-first may publish a new
token/scope; exact equality applies to the new handler-first retirement branch.
O-added unproved negative will assert actual manual NotHandled/no edit/learning
instead of using an IPC readiness refusal as a proxy for word authority; all
original canonical mutation/unpaired/lifecycle guards retained. Source repair
pending, both full nativeFAILs retained and no installation/build repeated.

2026-10-13T22:08:18.907214Z — P33 metadata repair31PASS/2FAIL, `p-confirmed-release-source-9v962y99/RESULT.json`, 16.850514s. Real Reset/SSI/release/Backspace/manual positive and reducer metadata unit now PASS. Both remaining negatives stop before exercising release: matching current token/UnknownStart scope is valid and is not word edit permission. Assert explicit UnknownStart plus retained actual ManualToggle NotHandled and exact state/no-output/no-learning instead. Production unchanged for this fixture-only correction; full source/native remain pending.

2026-10-13T22:09:57.874111Z — P33 fixture clarification32PASS/1FAIL, `p-confirmed-release-source-mnfo7dfm/RESULT.json`, 16.879893s. Original O unproved-word actual NotHandled passes. New no-SSI fixture hits existing legacy LocalPending mapping to Handled(current layout), not a text effect. New negative asserts actual RequireSurroundingText/pending flag, unchanged tail/epoch, UnknownStart/no exact handoff, no Commit/Delete/Forward or learning. Runtime outcome mapping untouched; third P33 current rerun10393 has changed source/test fingerprint. Native totals retained.

2026-10-13T22:10:34.881049Z — P focused33/33PASS,16.866132s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/P-source-probes/p-confirmed-release-source-5aj0pbhy/RESULT.json`, SHAd1ab1791552004dd1b8b2848dd0b24a09f989c58f115cd5722b4afdb7c22a958. All205 remote-formatted production/test Rust hashes verified against local. Actual handler-first and observer-first Reset/SSI/release/Backspace/manual positive; private/password/None metadata lifetime unit; original O30 controls and both actual word-authority negatives pass. Next fresh full eight targets (IME,daemon,six contracts), independent context-free P review2/2, graph, immutable build. O native321P78F33B retained; no P native claim, no promotion/laptop changes.

2026-10-13T22:16:36.687991Z — P fresh eight-target source1104/1104PASS, worker256.078348s/controller264.3s. Local `/home/ubu/.cache/lay/development/run-daqc9vwf/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-qrlOJ6`, archive2c5e359a99d92b86a7c50c4a9032d59023d8bd112839020704814d2f13b908e3. All205 IME/daemon Rust hashes match local. No reused daemon denominator. Independent context-free P final code review2/2 running; no P build/native yet, previous nativeFAIL totals retained.

2026-10-13T22:18:37.210691Z — P final independent context-free code review2/2 CLOSED9/10, `/root/confirmed_reset_release_final_review_20261014`; no material findings. Existing full-token/auth/metadata/one-shot and terminal-scope fences retained; admitted metadata lifetime repair accepted; actual pending/no-edit and unproved NotHandled negatives accepted; original O controls unchanged. Source33/33 +fresh1104/1104PASS; P nativeNOT_TESTED and prior321P78F33B remain FAIL. Next remote final AST, immutable buildrun-qrlOJ6, private staging/finite seat, focused original Firefox private standard before full matrices.

2026-10-13T22:20:46.801269Z — P remote final AST PASS20.854183s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/P-final-graph-receipt.json`; all734 Rust files remain identical to immutable gate request; four generated graph artifacts fetched with SHA verification. Source33/fresh1104/reviews9,9 complete; immutable run-qrlOJ6 candidate build live session32723. GUIOFF; O binaries unchanged until explicit private staging after build. No local AST/tests/input/install, no native acceptance claim.

2026-10-13T22:27:55.785145Z — P immutable run-qrlOJ6 build183.471569s: IME48235f612f8ce4e2c1b19f9a2605ebe6d3f73efe7de3aa2652eeae2f6efca7ae/8242960B; daemon2feb97733e6ff4ac7ae56795248d7ca9e6f3bd4e3a4a29339741839cc544dfd0/9264592B. Private-only stage verified O backup `staged-before-P-confirmed-reset-release-1791930233861523463`. New finite seat-aa PID2875048/t12779527/session3050/VT8. One controller validation failed because loginctl needs repeated -p flags; same new seat verified/resumed, no GUI/input/build repeated. Owned key ACL immediate event11/PID2876945/t12783267/cg2876924. Setup002003005 allparent0/noheldkeys; P-firefox-zero-key.json all5PID/tick/exe/cg/actorpins sourcebound. Original actor private20 diagnostic910 LIVE actualexec9635/read-only monitor7603, source33+1104 and review9,9 complete. No P full matrix or promotion claim yet; prior O matrices retained.

2026-10-13T22:33:55.036992Z — P original-actor private diagnostic19PASS/1FAIL/20; `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/P-native-private-diagnostic/RESULT.json`, SHA8b6c1770c41ed84a4e9f0edc196378c9c5fde0778044483be09d265ebe349057. Old private active1 passes, but private Space8 burst allfiveFAIL while both ordinary gesturesPASS; no same-source full matrices launched. Native clock ordinary40/40 p50105.768/p95117.956/max119.609ms; burst95/100 p50108.678/p95117.061/max117.989ms, five missing samples excluded; kernelreleaseNOT_MEASURED. Matchedtrace1ee794df4feaf0a99a2517d3bf8138095b4f5a908720e797ef192d757882e17d lines2028–2035: actual owned Space NBSP acknowledged and exact SSI7 confirmed at epoch2344, then Reset9439 arms unconfirmed proof and clears snapshot; managed Space release9440 accepted observer-first, no orphan; first manual refused context_authority with bridgeTokenLIVE. Need new source causal test for confirmed owned boundary/late Reset/no newSSI; preserve P fixes and reject timeout/transport/broadPassive repair. Actual9635+monitor7603 CLOSED0, diagnostic parent0/noheldkeys; strongscope2876924empty/all5ticksGone/VT1 restored22:30:00UTC, SIGTERM15 after intentionalSTOP. Source33/fresh1104/reviews9,9 retained; PnativeFAIL/no promotion.

2026-10-13T22:33:55.036992Z — User requires the Firefox/Chrome/total pseudographic PASS/FAIL/BLOCKED/total table in EVERY Lay status. Rule added to AGENTS.md; use latest completed full receipts (currently O321P78F33B/432), display diagnostics/source gates separately, never replace full totals with partial runs.

2026-10-13T22:37:47.856794Z — Q causal plan after P diagnostic: existing owned closing-Space capture still requires literal ASCII Space despite independent exact acknowledged NBSP producer. New decision2026-10-14-acknowledged-owned-space-reset.json rates existing Unicode whitespace predicate9/10; new state4/10, normalization1/10 rejected, timeout/extra SSI3/10. No runtime edit. New source-only actual confirmed owned ASCII/NBSP Space followed by held-key Reset/no newSSI and both release orders; retain P33, original closing-Space14 and contradictory boundary19-case guards. No word/phrase/browser literal exception, transport/owner/SafetyGate change.

2026-10-13T22:39:47.262642Z — Q causal source RED37controlsPASS/1newFAIL/38,17.653259s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/Q-source-probes/q-acknowledged-space-source-baf1zjgq/RESULT.json`, SHA8a44eb0d3c3f742e8d4c0aea877f86bbba748f237c7d7be9f174e75f4202549e; all205 hashes match formattedlocal. Actual ASCII held-Space path passed; NBSP exact acknowledgedsnapshot becomesNone on matchedReset while ASCIIretained. No runtime edits yet. All P33 plus original one-shot/missing/select/cursor/foreign/navigation/content/other-separator Space guards pass. Independent Q plan review1/2 next, then minimal existing whitespace predicate if accepted. GUIOFF/VT1; Pprivatebytes unchanged/no new build/fullnative.

2026-10-13T22:44:32.335317Z — Q plan1/2 CLOSED9/10; production delta exactly existing closing-Space .ends_with(char::is_whitespace) plus explanation, no other runtime rule. Focused38/38PASS17.764434s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/Q-source-probes/q-acknowledged-space-source-x49x5hkq/RESULT.json`, SHA4548d4e7e6a30f8031ad38ac80d1aaf4f1990d07d8e8661891e7ad6d36e143f0; all205 formatted Rust hashes matchlocal. Actual ASCII/NBSP/empty+nonemptyprefix/bothreleaseorders/heldReset/no freshSSI/ManualDelegate positive passes; all P33 and original held-Space14/contradictory19 controls unchanged/pass. Next freshfull8-target gate, finalQreview2/2, AST/build/privateNative. Existing fullO432321P78F33B remainFAIL, P diagnostic19/20 retained; no Q native/build/promotion yet.

2026-10-13T22:50:25.959739Z — Q fresh eight-target1105/1105PASS, worker255.938386s/controller264.9s; `/home/ubu/.cache/lay/development/run-61bj0v7o/RESULT.json`, remote `/home/e/projects/lay-development-runner/run-cht3Zw`, archivee1f0e3864aeb3f1f9669270960b3b0119367ce2e130d17f5f7a5214a56adf8f5. All734Rust files match frozenrequest; no reused daemon/test counts. Context-free Q final review2/2 running, root production unchanged after source gate. Next final AST/immutablebuild/native; QnativeNOT_TESTED and both previous full matricesFAIL remain, no promotion.

2026-10-13T22:52:25.251774Z — Q final independent context-free code review2/2 CLOSED9/10, `/root/acknowledged_owned_space_final_review_20261014`; no material findings. Sole runtime predicate bounded by exact acknowledgement/heldkey/current snapshot/one-shot Reset; original adverse14/19 assertions unchanged. Source38/38/fresh1105/1105 and all734Rust hashbinding complete. Next finalremoteAST, immutable run-cht3Zw build, private stage/renewfiniteownedseat, originalprivate20 beforefull9browserjobs+speed. NativeQNOT_TESTED, priorfulltableunchanged/no promotion.

2026-10-13T22:56:27.366662Z — Q final remote AST PASS20.821410s, `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/Q-final-graph-receipt.json`; all734Rust remain identical to immutable run-cht3Zw. Four graph outputs fetched/SHAverified. Source38/full1105/reviews9,9 complete; private candidate immutable build16080LIVE. GUIOFF/VT1, Pprivatebytes unchanged pendingQstage. No localAST/tests/input/install or new native claims.

2026-10-13T23:00:26.868337Z — Q immutable buildrun-cht3Zw183.311931s: IME399ffa890002bc6ce55b869bc30138f77656574b55be173fd59c91d37f7a5291/8243216B, daemonc49c44cb7776051433fc7d77ad0d5ac4d5f78f04b402895ffe1363950c3e3d97/9264592B. Private stage Q backup verified `staged-before-Q-acknowledged-space-1791932221320869900`; original actors unchanged. Verifiedownedaasentinel stoppedafteroldGUIclosed; newfiniteab2979358/t12975486/session3113/VT8. Native actual74111 LIVEscope2980379, immediateownedkeyboardACL event11/PID2980400/t12977453. Setup002003005 all0/noheldkeys; zeroQ-fivePIDsource/CG/pinsPASSsetup only. Originalprivate20diagnostic911/monitor87364 LIVE, receiptfirefox-standard-private-fg01xqzw/RESULT.json; no fullQ/nativeacceptance claim. Source38/fresh1105/reviews9,9/AST734 unchanged complete; fulltablepriorO321P78F33B retained. NoheavyworkwhileGUI/no laptopinstall/input/RDP/promotion.

2026-10-13T23:02:52.488396Z — Q original private20 diagnostic20/20PASS; `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/Q-native-private-diagnostic/RESULT.json`, SHA6a200e40701f9adce1e1b0afa8b83392b5cc12b169c2210a7be9b0bdde5572de. Both prior private active1 and privateSpace8burst pass. Speedordinary40/40 p95120.396ms, burst100/100 p95118.921ms; kernelreleaseNOT_MEASURED. Parent9110/noheldkeys. SAMEownedGUI actual74111/scope2980379, all5 identities revalidated; original9 Firefox jobs051-059 nowqueued, full216 completion monitor72210LIVE/autostopafterall9parent0/noheldkeys. One controller filename lookup initially lacked leadingzero and queued zero jobs; corrected before enqueue, no native tests/build repeat. Diagnostic never aliasedas052. NoheavywhileGUI; primary432table remainsO321P78F33B until fullnewreceipt.


2026-10-13T23:24:26Z — Q full Firefox original9 complete; standard60P/panel6P/functional87P44F19B =153P44F19B/216. All216 stable IDs vs O:1newPASS private-active-field1,0lostPASS,215same. Immutable native399ff-firefox-proof.json SHA93af9b30e3faf40622f4c6bda2dee877b5766ec4b6eed1dd35ad762753b72de6; RAW native399ff-firefox-raw-1791933808802713777/RAW.json. Standard420/420 p95120.982ms, panel48/48 p95120.451ms, functional DS101/210 p95136.333ms/Space37/50 p9523.736ms/Tab0/10/selection0/10, missing remain explicit/kernelreleaseNOT_MEASURED. Diagnostic20/20 remains separately indexed. Actual74111+monitor72210 CLOSED0/ownedSTOP/SIG15; exactCGempty+fivePIDticksGone/VT8→1 receiptQ-firefox-closed.json at23:22:08UTC. No promotion; ChromeQNOT_TESTED, current mixedtable QFF/OChrome322P77F33B/432.

Collector primary-role scope fixed because Q diagnostic20 plus original3 standards produce four receipts. New assemble-scoped-candidate-browser-proof.py selects exact original9 jobs, validates parent0/noheldkeys/terminalCOMPLETE/receiptstatus/full dimensions/speed/actor+candidate pins, retains mapped diagnostic20 separately, rejects unknown same-candidate attempts and failed diagnostic evidence; never picks latest/best or discards a current attempt. Independent context-free infra reviews1/2 score6 and2/2 score8; three false-PASS gaps fixed, finalP2 historical-candidate binding fix applied after pass2/no third review. Remote10/10 fixture checks PASS Q-collector-fixture-checks.json, scopeCOLLECTOR_FIXTURES_ONLY_NOT_NATIVE_ACCEPTANCE: real216dimensions+diag40/100speed, terminalsave/cleanup/status mismatch, unknownkind, missing/duplicate diagcells, bad/nonfinite speed, older failedcandidate retained without blockingcurrent. First fixture harness glob omitted Q in diagnostic filename and failed before any collector scenario; corrected glob then10PASS, no native/code/source rerun. Next exactsameQbytes Chrome original100–108, publicpositive/privatewriter brackets, then stableID comparison before source work. Runtimeauthority/routes unchanged.


2026-10-13T23:38:07.284338+00:00 — QChrome native firstlaunch blocked sharedheavy lease beforeGUI/input; unrelated L3scope3068225 was not stopped/bypassed, resumed onlyafter lease becameavailable. Next startup actual68855/CG3079574 setup002003005/zeroSHApassed but native seat3113/VT8 Active=no/foregroundVT1 after priorFirefox restoration: allnormal20 BLOCKED exacttypedseed (340 emitted keys/firstfieldempty); private terminalBLOCKED/20NOT_TESTED. Monitor31077 CLOSED1 and ownedSTOP, actual68855CLOSED0, exactCGempty/all5ticksGone. Preserved complete attempt/archive Q-chrome-inactive-seat-attempt-1791934375202890764/ATTEMPT.json; receipt hashes0293af... and9e9129...; not erased or claimedas productregression/fullmatrix. Root preparation error: startup lacked ownedseat activation after restoringVT. Added controller require_active_seat beforeanyinput/everyoriginaljob, exactUID/PIDtick/loginctlActive+VT+Leader/foreground8. Remote no-input controls rejectinactive oldseat and acceptverifiedownednewseat, Q-chrome-active-seat-gate.json. Verifiedownedab stoppedonlyafterGUIempty; renewedfiniteac3093791/t13194367/session3181/VT8 onehour and actualActive=yes. Q-chrome-finite-seat.json properties are BEFOREactivation (Active=no); following positive gate records actualactivated seat. Source/actors/newprivatebytes unchanged. Freshactual9896/CG3094715, immediatekeyboardACL event11/owner3094736/t13196213; setup002003005all0/noheldkeys/QchromezerofivePIDpins. Originalfullmonitor92118 LIVE:100 normal19P1B, snapshotsnormal-before/after/private-beforequiescent3files, original101private running. NoheavywhileGUI. Fulltable remainsQFF/OChrome322P77F33B432 until original9 verified. Runtime/laptoproutes unchanged.

Read-only residual inspection: Firefox44F groups prefix7/undo10/cursor10/selection7/focus10;19B Tab10/prefix3/movedboundary3/selection3. Several seeds expose unaccepted suggestion after caret (e.g. fullприветное withtypedprefixпривет/caret6); focusreturnedcaret9 andselectionrawаж/caret1 are recorded observations, not root attribution. Hypothesis that output mistakenly uses PREEDIT_MODE_COMMIT is rejected by current source: preedit.rs31/632 already sends PREEDIT_MODE_CLEAR=0. No renderer or focus edit based on this hypothesis. Need actual first-transition/wire evidence after bothQmatrices before a new core experiment.


2026-10-13T23:49:40.941365+00:00 — QChrome original9 complete706.906s; primary169P33F14B/216 =standard58P2B+panel6P+functional105P33F12B. ALL216stable-ID verdicts matchO,0new/0lostPASS. Primarynative399ff-chrome-proof.json SHA73a88c47a24bff2d7d54cff2e9db09bec3f06865f8f23f5d74f926a0fd3d1365; RAWnative399ff-chrome-raw-1791935217299974275/RAW.json plus separateprimary9receiptretentionindex. Native9896/monitor92118CLOSED0/ownedSTOP/SIG15; CG3094715empty/all5ticksGone/VT8→1 at23:46:56.746492UTC. Privacy3visitedfilesunchanged+normalpositive recent_actions changed, Q-chrome-private-sink-proof.json PASS_VISITED_LAY_SINK_ROOTS, no universalcoverageclaim. Standard406/420p95113.861ms, panel48/48p95109.729; functionalDS138/210p95127.697,Space38/50p9523.970,Tab0/10,selection10/10p9521.469; allmissingexplicit/kernelreleaseNOT_MEASURED. BothfullQ432322P77F33B/0NT; across432 IDs1newPASS/0lostPASS; sourceQ38/fresh1105/reviews9,9 unchanged. Compactrepoevidence tech_debt/evidence/td121-Q-full-browser-native-2026-10-14.json; QdecisionFULL_BROWSER_NATIVE_FAIL_RETAINED_NO_PROMOTION. ResidualChrome33F undo9/cursor10/focus10/prefix1/spaceEN-RU1/spaceRU-EN1/cleanEN1;12B Tab10/undo1/movedboundary1, plusstandard2searchSpaceB. NextR-D0 diagnostic unchanged Q bytes/publicFirefoxfield1 original15plans+passive exactwire, after activeownedseat/zero setup; source edits onlyafter firstcausaltransition/plan. Routine512KiBtrace hasprefix_preserved=false so firstfield sourceattributionUNKNOWN. TD121OPEN/no publication/laptopchanges.


2026-10-14T00:02:09.383916+00:00 — R-D0 unchangedQbytes/publicFirefoxfield1/originalunmodified15plans/passiveexactIBuswire COMPLETE:9P5F1B, equals previousfullQfield1 statuses; NOTprimary216 replacement. WireR-public-functional-wire-9r0p60_3/WIRE.json, wire1698153B SHA5354a52c07cead7645253a87cede2cb5dfa2e874b5c2466cf60e6f90dc56de86; actorfirefox-functional-b38lucfg/RESULT.json, exactactor/bodypins76d329.../45ae44... preserved. Monitor89850parent9200/noheldkeys/ownedSTOP, actual38026CLOSED0, CG3151956empty/all5ticksGone/VT1restore R-firefox-closed.json. FivePIDsourcebinding R-firefox-zero-key.json, ownedseat3181Activeyesverifiedbeforeallkeys; immediateevent11ACL owner3151977/t13300102/setup002003005 all0. No source/core changes.

Measured focus: initialclientSSI textпривет/caret6, engineghostное/CLEAR0 atserial1569; clientReset2214/engine1481 -> engineclear empty,false,CLEAR0 signal1571 -> inputcontext forwards clears2785–2789. Focusreturns clientSSI2224 textприветное/caret9 beforeengineSSI1495; no engineCommitText ofное in this captured focus interval, only following typedы commit1584. Thus client text changes before new typedletter despite actualclear; further frontend/presentation root unknown, separate from cursor.

Measured cursor: sourcealreadyUnknownStart BEFORE1350 (1347/1349 settlements), exact locally observed typedtail6/currentSSI6/caret6 supplied prior suffix-manual eligibility. FirstArrow1350 clears tail6→0 via close_committed_tail_field, ghostcleared; nativearrowReset/SSIs walkcarets5/4/3, BackspaceSSI5caret2, managedиCommit+SSI6caret3 butlocalbufferonly1. End1370 -> actualSSI6/caret6 serial1371 BEFORE Reset1372, resetrejectmissingpredecessor token/clears snapshot, release1373 accepted. ActualManualRPC1257 context_authority refused, bridgealive/manualallowedfalse/refreshfalse. Cannot fix by treatingarbitraryUnknownStart asKnown or onlyretainingEndSnapshot: first owned-suffix provenance lost earlier. R-wire-focused-excerpts.json preserves all matchingclockedwire forfocus/cursor/prefix; routine512KiBtrace now containscursor tail and correspondingserials. New independentcontext-free cursorplanreview1/2 /root/cursor_authority_plan_review_20261014 READONLY running; directionsboundedexisting-editproof versusunsafe genericrefresh/no-opworkaround. No production changes until causal RED+defensible minimalplan. FullQtable still322P77F33B432; source38/1105/reviewsQ9,9 complete/no promotion.


2026-10-14T00:11:12.886616+00:00 — R cursor plan independentreview1/2 CLOSED8/10; measured initialUnknownStart plus confirmedreset exacttail6, firstArrow clearslocaltail6. Revised recommended8 typed inert manual-only editing phase in existing single receipt slot, one expected effect/currentactualsnapshot, sameowner/purpose guards, no general confirmation/Space/Tab promotion. GenericUnknown SSI1/10, End-onlysnapshot3/10, extraSpace2/10 rejected. Decision2026-10-14-manual-cursor-edit-provenance.json. Root test-only native_cursor_edit.rs/include in existing residual harness prepared: actual3Left/Reset/SSI/release, nativeBackspace, managedinsert(c or z)/Reset/SSI/release, End/SSI-before-lateReset/release, realManualToggleV3/exactVisibleTailV2; prefix/empty/bothreleaseorders; oldtokeninvalid/noKnownStart/no extra text/learning. OriginalQ38 controls selected; source R39 causalRED next remote, productionQ unchanged. Fulltable322P77F33B unchanged; GUIOFF/VT1.


2026-10-14T07:31:41.532363+00:00 — W configured repeated closed-source54 and delayed final Backspace-release54 BOTH PASS (23.965s/23.910s; receipts /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/W-configured54-pass-RESULT.json and /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/W-delayed-backspace-release54-pass-RESULT.json). Source fixture does not reproduce V fullscreenSpace9 native failure; first loss UNKNOWN. Next WD diagnostic adds only numeric/boolean metadata after existing exact-replay rejection (scope pair/current, command modifier, unexpected key or failed mirror transition), before scope revocation, no successful callback logging. No functional predicate/route/owner/timer/model changes. Decision2026-10-14-native-replay-failure-only-diagnostics; one originalstandard60 normal/private/fullscreen sequence after fresh scoped gate/AST/private byte proof, no passivewire. Partial diagnostic never replaces fullV339P60F33B432. Independent review NOT RUN threadlimit/no score; TD121OPEN.


2026-10-14T07:38:59.124904+00:00 — WD fresh scoped gate843/843 PASS (IME792+sixcontracts51; worker116.301s/controller124.8s), immutable remote run-P4dPQe/archivebec4e26e89e7271e5288992bf2d6c43b711c3e5281fb85de8a11eece70aa4847; all737Rust hashes verified against local source. Remote AST20.797s, all4 emitted artifacts fetched/hashverified. Root audit: remove only failure-only metadata helper+five calls and production tail_memory exactly equals V frozen source. No predicate/positive path/owner/timer/route/model fit changes. Private immutable build still running actual84885; no new native result yet. User says current сборка good; local installed bytes not established by that remark; preserve local runtime. Latestcomplete full432 remains339P60F33B; diagnostic-only/no publication/taskOPEN.


2026-10-14T07:49:51.288202+00:00 — WD standard60 diagnostic COMPLETE60PASS0FAIL0BLOCKED, original3parents0/noheldkeys, actual3321/driver32363 CLOSED0; exact5originalticksGone/CGempty/VT8→1. Archive /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/WD-standard-diagnostic-1791963994703664507/RESULT.json SHA66f029589a4c787c1938e184dae8b4173ad0fddb5cff04bb85cd3b12eab7beb6; all80files copied/hashverified. ZERO ibus_exact_replay_rejection records. V fullscreenSpace9 did not reproduce, causeUNKNOWN/no functional fix claimed, no unchanged repeat. Ordinary p95normal121.472/private119.020/full120.476ms all120/120; burst118.723/116.910/115.019 all300/300; kernelreleaseNOTMEASURED. Next WE one original057 field5,6 functional30 using unchanged gatedWDbytes, no passivewire; snapshot existing capped trace once at observed row completion (bounded30x512KiB) to retain failure-window callbacks missing in V. Originalactor/inputcadence untouched. Same decision extended; no source/build repeat, authority/route/model/laptop changes. Latest complete full432 still339P60F33B; taskOPEN/reviewNOTRUN/threadlimit/no publication.


2026-10-14T08:06:33.894355+00:00 — WE original057 fixed30 COMPLETE20PASS8FAIL2BLOCKED/64.105s; captured30 row traces/no errors, copied/hashverified41files; resultSHA9c3f7e23ac6a8db002b9b2ff1b76de05a35afd5cd80c2385dad9a715e54abf93 /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/WE-functional-diagnostic-1791964524400800201/RESULT.json. actual48580/driver47466CLOSED0/WE5Gone/CGempty/VT1/oneparent0/noheldkeys. ComparedV stable30 two Space losses nowPASS, allotherstatuses equalV; no functional code change, no complete matrix promotion. Prefix5 actualreplacement succeeds then layout_sync false; nextz decodedя onoldRUowner. Prefix6 layoutsyncfalse thensecondRPCbridge_admission refusal. Both start actual printable_legacy_preedit, so regular-preedit-mode speculation ruledout. First refusing ownership/dispatch predicateUNKNOWN; do not alter layout guards. Selection actualCommitа/SSI1 thenvisibleuncommittedж may be observationissue, verdictunchanged. Undo leaseNotReady3458us thenoldworksuperseded, no model/ranker/deadline fix justified yet. Allfivefailurewindows retained withactualcallbackanchors11/19/15/15/8 andcoverageUNKNOWN. LatestfullV432339P60F33B unchanged; taskOPEN/reviewNOTRUN/threadlimit.


2026-10-14T08:09:09.265446+00:00 — WF diagnostic planned from measured WEprefix loss, not an assumed functional fix. Existing owned-preedit false result with nonempty composition emits only guard booleans; existing failed blocking layout dispatch emits its enum. No successful callback logging, guard/policy/route/timer/owner/model edits. Decision2026-10-14-owned-preedit-manual-refusal-diagnostics; fresh scoped843/AST/immutableprivate build then original057fixed30/case-end trace capture once. First rejecting guard stillUNKNOWN; no broadened mode scope or optimistic decoder. GUIclosed/WEexact5Gone/CGempty/VT1. Latestfull432339P60F33B/taskOPEN/reviewNOTRUNthreadlimit.


2026-10-14T08:26:49.865780+00:00 — WF original05730 reproduces exactlyWE20P8F2B/64.100s; all30rowtraces+41files retained/hashverified, resultSHAf521df86c405005cd1744ad8cf69ae7940979f60b45ae4a8dff14e8f39babab4. actual56847/driver94588closed0/WF5Gone/CGempty/VT1. Measured prefix5/6 solefalse owned-preedit condition exact_refresh=true; all17otherguardfacts valid (legacy owner/mirror/visible/currenttoken/scope, no selection/pending/etc). Blocking dispatch ContextConflict switchedfalse follows. First producer mistake localized; no functional edit yet. WG newdecision9/10 bounded birth-witness-based admission under existing owner/InputMode transaction; broadveto removal2/10/backgroundfallback6/10. Test-only builder preserves unmarked41default and supplies markedcap from birth, real prefix/SSI/typedword,8manualgestures/decoder/Backspace/sameowner+snapshot. Nextcausal57 RED incl54controls+existingknownOwnedpositive+16refusalguards. Latestfull432339P60F33B unchanged/taskOPEN/reviewNOTRUN/threadlimit/no promotion.


2026-10-14T08:50:46.048180+00:00 — WG focused57GREEN24.179s sig4439f4e... after explicit bounded marked-prefix proof and actual localBS restoration. Focused58GREEN24.732s sigce143f87... adds12refusal classes; allold16guards (including raw marker flip) remain. New marked8gestures preserve owner/path/widgetprefix snapshot/revision and assert nextz decoder pluslocalBS, noCommit/Delete. Actual source afterprobe witness loss wasgenericBSrevocation; boundedexisting prepare/finish localBS nowretains samebirthwitness afterverifiedonechar/oneepoch, currentprefix/token/identity. Final pending-display-on-localerase check and authorize-before-negative mutation are in fresh scopedgate80258 currentlyLIVE; no nativeWG yet. Reviewdiff /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/WG-review-diff.patch relativeimmutableWF source contains only3WGsource files. LatestfullV339P60F33B unchanged; TD121OPEN/no install/promote/push/refit.


2026-10-14T08:54:27.645506+00:00 — WG first fullscoped845=844PASS1FAIL (all794IMEPASS including final pending-displayBS/new12refusals). Failure canonical typed_authority_mints_have_only_named_runtime_callers refused new fixture direct plan_ime_manual_toggle_edit. Production unchanged; no weakening of guard/canon/test. Corrected fixture calls actual named runtime manual producer, then asserts no new InputMode/visible owned-preedit effects when birthproof invalid; older independently proved routes permitted. Fresh845 source required beforebuild/native. Context-free WG reviewer spawn failed agentthreadlimit/NOT_RUN/noscore. LatestfullV339P60F33B/taskOPEN.


2026-10-14T09:01:32.977661+00:00 — WG fresh scoped845/845PASS (IME794+sixcontracts51; worker116.261s/controller124.6s), run-LItDwT/localrun-r32mz8sc/archive0d08b7e50a1996d3e17759a80686bb904a59b01a53fdb0dae5ed5256c22ea4f9/all737Rust verified. New marked-owned positive8gestures includes pending-display-beforelocalBS; 12negatives invoke named runtime producer and prove no owned InputMode/preedit transaction. Allold16guardadverses remain; no fixture mint/canon weakening. Sourcegate includes latest negativeordering and local erasure proof-only retention (no manual authority while pending). Currentbuild97576LIVE AST+immutable private binary; no nativeWG acceptanceyet. Full original9/browser protocol next, all432 stable IDs/speeds/privacy retained, no rowwatcher. ContextfreeWGreviewNOTRUN agentthreadlimit/noscore/taskOPEN; latestfullV339P60F33B/nopublish/laptopinstall/refit.


2026-10-14T09:40:16.451011+00:00 — WG Chrome zero-key setup twice rejected before any behavior input, bothclosed exact5Gone/CGempty/VT1. Root cause second capturedFocus payload unchanged (clientgnome-shell, lay_ime_ru/1), only wire serial35→47/time changed. Existing zero setup dictionary equality conflates repeated observational header with identity. Protocol decision2026-10-14-browser-zero-key-focus-identity: compare nonemptyclient+engine path and retain bothheaders; existingfreshIC/globalRU/IMEPIDticksha/focuswindow/kernelpeer/portal rechecks unchanged. Add pure helper+five semantic tests (header-only change allowed; missing/malformed/client/path changes refused), revise005pin only; no sourceRuntime/model/deliveryRoute/binary change. CurrentWGfullFF164P33F19B/216 (5Vgains1lostSpaceRuEn1), all9primary+RAWmanifest copied/hashverified, partialnotfulltable. Original432baselineV339P60F33B staysuntilChrome216. ReviewerNOTRUNthreadlimit/TD121OPEN.


2026-10-14T09:49:43.073719+00:00 — WG fullFirefox216 COMPLETE164PASS33FAIL19BLOCKED/712.986s; standard60PASS/panel6PASS/functional98P33F19B. V5gains1lostPASS(spaceRuEn1)/U3gains1loss, all10FFcursorPASS. Exact5Gone/CGempty/VT1/original9parents0/noheldkeys; all9primary+RAWmanifest locallyhashverified. Std420/420p95118.485ms/panel48/48p95118.855/functionalDS124/210p95137.233/Space36/50p9525.347/selection1/10/Tab0, missingexplicit/kernelreleaseNOTMEASURED. Oddprefix1/5stillfirsttargetFAIL, even2/6PASS; nextcauseUNKNOWN/no speculativefix/rollback/promotion.
Chrome original behavior neverstarted in two failedzero setups; bothclosedexact5Gone/CGempty/VT1. Measured Focus payloadclient+enginepathunchanged, ONLYwireheader serial/time changed. Protocolhelper5unitPASS/new005SHA dde6db54.../helperbd2bc86f...; allotherPIDticksha/window/IC/RU/kernelpeer/portalchecksunchanged. PostPythonAST20.705s/all4hashverified/all737Rustunchanged/no rebuild. Newownedfiniteaj855015/t16866706/session4022expires1791974636. Chrome ZEROreadyPASS now original100–108 LIVE actual86410/driver55556/WGfourprivacybrackets. Original normal20=19PASS1BLOCKED; privatequeued. No sourceedit/heavywhileGUI. Latest completeBOTHtable remainsV339P60F33B untilcurrentChrome216 complete. ReviewNOTRUNthreadlimit/TD121OPEN/no publish/laptopinstall/model fit.


2026-10-14T10:02:47.127755+00:00 — WG BOTH original216/browser COMPLETE432=343PASS56FAIL33BLOCKED (FF164/33/19, Chrome179/23/14). Chrome696.994s actual86410/driver55556closed0/exact5Gone/CGempty/VT1/9parents0/noheldkeys. Chrome ALL216statuses exactly V/U; FFV5gains1loss =>net4morePASS/fewerFAIL, NOT complete/promotion due56F/33B+lostFFspaceRuEn1+reviewNOTRUN. All20cursorPASS. BothRAWmanifests andALL18primaryJSON locallyhashverified WG-both-primary-retention-verified.json; FFproofSHAf8e190.../ChromeSHAba7c69c0235ae4a35a4edf61bb0e265130a2b1d972b21c13ab50941e31796a02. WGChromeRAW1791971933462849002 manifest1dc10ee... retained. StdChrome406/420p95110.127ms/panel48p95108.761/functionalDS158of210p95124.664/Space38of50p9525.409/selection10p9523.074/Tab0; FFspeedspriorentry. Missing explicit/kernelreleaseNOTMEASURED/no statisticalimprovementclaim. WGprivate3filesunchanged/publicpositive recent_actions changed; notfullaudit/modeltraining. Compactrepoevidence tech_debt/evidence/td121-WG-full-browser-native-2026-10-14.json. Nextfirst newfailedprefix-bindingpredicate +SpaceRuEn1firstauthorityloss unknown; no wordbranch/guardweakening/unchangedrandomretry/blindrollback. Latestcurrenttable343P56F33B; taskOPEN/no publish/laptopinstall/refit.


2026-10-14T10:07:50.009976+00:00 — WH failure-only diagnostic planned from completeWG343P56F33B432. Oddprefixfirsttarget failures unresolved, currentcappedfunctional traces don'testablishnewbindingpredicate. Trace birthcap/focus/lease/layout/revision/currentfullprefix andmode fields onlyafter originalmanual-entry orpost-publication refusal. No new positive-callback logging, authority/predicate/expiry/effectorder changes. Fresh scoped845/AST/privatebuildthenoriginal055fields1,2fixed30once+row-endtracecapture, nopassivewire/cadencechange/wordorfieldruntimeexceptions. Decision2026-10-14-marked-preedit-birth-binding-failure-diagnostic. FFSpaceRuEn1 firstauthorityloss remainsseparateUNKNOWN; no ranker/model/deadlinefix. AllGUIclosed/VT1/taskOPEN/reviewNOTRUN.


2026-10-14T10:28:06.548781+00:00 — WH diagnostic original055 fields1,2 complete20PASS8FAIL2BLOCKED/30 in65.101s; all41artifacthashes verified, receiptSHA2bf4a45c..., actual90438/driver30713closed0, exact5ticksGone/CGempty/originalparent0/noheldkeys/VT1. Metadata-only bytes; no full432replacement (WG343P56F33B remains). First field1prefix RPC refuses context_authority before owned manual branch, bridge token live, manualToggle/refresh false, UnknownStart. No new binding-refusal records; deeper cause UNKNOWN. Field2prefix firstownedmode/nextz/localBS pass, secondgesture sole birth-binding failure revision10→12; focus/lease/layout614/fullunchangedclientprefix/cursor/unselected/currentowner allvalid. Must reproduce actual duplicate SSI path before bounded inertbirthwitness refresh; retain stale-revision-without-real-receipt refusal and all priorguards. No dropping revisionguard/counterfeitSSI/client-specific code/deadline/model change. Diagnostic SpaceRuEn1 FAIL→PASS andprefix2 PASS→FAIL with identicalfunctional logic show timing sensitivity, not fixes. TD121OPEN/reviewNOTRUN/threadlimit/no score/no promotion.


2026-10-14T10:34:38.798112+00:00 — WI real repeated unselected prefix SSI causal RED59=58PASS1FAIL/25.088s sig627929b3... remote wi-marked-refresh-source-l2b76iqx; actual callback advances revision while same owned birth remains pinned. Earlier fixture private-field compile error is not causal RED and was corrected without exposing private fields. Minimal production observe_external_surrounding_text refreshes same birth armed_revision only if pre-ingress full marked-owned proof/currenttoken valid, previous actualsnapshot exists, new fullsnapshot exactly same, revision consecutive. No new authority/snapshot/payload/route; all old guards unchanged. Expanded positive8modegestures/probeBS/16actualreceipts and new14adverse classes now focused60 running. Field1 earlier UnknownStart context_authority remainsseparateUNKNOWN. LatestcompleteWG343P56F33B432; TD121OPEN/reviewNOTRUN/threadlimit/no nativeWIyet.


2026-10-14T10:41:00.717849+00:00 — WI fullaffected847PASS/worker118.484s/controller129.2s/sourcearchivea208be55.../all737Rustmatch, remote run-QaJV76/localrun-ow_hrxg8; source-only, nativeNOTTESTED. Before expensive candidate build, investigate separate WHprefix1 failure with source-only WJ causal actual closingSpace→Reset→prefixSSI→pairedrelease→ownedword ordering. Native KnownStart2002 becomesUnknownStart2016, fullownedword starts2025, RPCcontext_authority2208. Key preedit_chars means suggestion suffix, not compositionword; no partially-owned inference. Test-only optionalhelper keeps alloldbehaviordefaultfalse; no furtherproductionchangeyet. WJ61 focused running, causal validity notassumed. LatestcompleteWG343P56F33B unchanged; reviewNOTRUNthreadlimit/TD121OPEN.


2026-10-14T10:43:14.826506+00:00 — WJ source causal61=60PASS1FAIL/25.090s sig e13e46a7..., actual closingSpaceResetSSIrelease reproduced UnknownStart with wholeownedlegacyword/currenttoken/fullbirthsnapshot; failure is explicit marked-boundary KnownStart requirement. Minimal manual-only proof now uses exactbirth/fullprefix/revision/metadata/currentowner and permits unchangedUnknownStart; no promotion, no genericedit/Space/Tab/SafetyGate change. New sourcepositive8gestures+decoder/localBS/actualSSI asserts UnknownStart and genericmanual refusal remain; WI14adverses expanded tobothreal lineages (28variants), oldknown/unmarked controls retained. WJ61GREEN running. LatestcompleteWG343P56F33B432; source ordiagnosticnotnativeacceptance; independentreviewNOTRUNthreadlimit(no additionalspawnattempt), taskOPEN/no publication.


2026-10-14T10:58:38.956923+00:00 — WJ broad KnownStart removal FAILED full848 (844P4F) beforebuild; oldC06 append/manual-denial, staleSSI, externalepoch andowned-return epoch guards remained unchanged. Refined UnknownStart grantrequires retired handled input; focused65 (old4 failures included) passed. Additional bothlineage released-epoch adverse thencausalRED65=64P1F sig7333505b... exposed latent markedmanual missingepoch binding. Barebirth+chars guard failed three positives after actualpublish_tail_handoff increments epoch1 (signature53b57...); itwasnot promoted/retriedunchanged. Final bounded scalar manual_tail_epoch_offset inexistingbirthslot updatesONLYafter sameverifiedproducer exactone-step actualhandoff (androllbackhandoff); Space/deletion do notreadit; existing localBSrebasesoldstart andzerosmanualoffset. Focused65 passed sig052dfc..., finalheld press/release assertions+failure-onlyepoch/retired metadata passed65 sigac996d..., no new positivecallbacklogs. Nowextendexisting outputfailure proof toallthreeactualmodes (oldunmarked,markedKnown,markedUnknown), including propertyfail rollback+nextsuccess; finalfocused66 running. No newowner/SSI/knownstart/timer/model/deliveryroute, nativeWI/WJNOTTESTED, fullWG343P56F33B unchanged, reviewNOTRUNthreadlimit/TD121OPEN.


2026-10-14T11:02:05.493161+00:00 — WJ FINALfreshscoped848/848PASS: IME797+sixcontracts51, worker116.147s/controller124.700s, localrun-ab157r6z/RESULT.json remote run-w03rUs, archiveb46f7c455b4aa50f6dae7cd72df68e8f0b91feccb3b43c5d4a3c22355f089d45/all737Rusthashesmatch. Finalfocused66PASS26.469s/eae59bd0..., old4fullfailures preserved andpass; heldmutableinput/releasedepochfault/currentowner/15outputfailure variants andmodefailure rollback+nextsuccess proved. Private immutablebuild+AST requested; no GUI until actualbuild0/4ASThashesfetched/oldWHscopeempty. Stockroutes/knownstart/snapshot authority unchanged. Sourceproof isnot nativeacceptance; fullWG343P56F33B table remainsuntilneworiginalboth432. TD121OPEN/reviewNOTRUNagentthreadlimit/noscore; no laptopruntime/publication/refit.


2026-10-14T11:04:11.461242+00:00 — USER_STEERING: user requests finish build now and explanation, stops further improvement/native-test phase. Complete alreadyrunning immutableWJ build48504 from final848PASS source; produce reviewable paired artifacts and manifest. No new functional tests, browser input, runtimeinstall/restarts, commit/push/tag/release inferred from this build instruction. LatestcompleteWG432343P56F33B remains previousbytes; WJchangedbyte nativeNOT_TESTED/reviewNOTRUN preserved, TD121OPEN. Source3causalmechanisms fixed underoriginalguards; remaining native failures mixed/firstcausepartlyUNKNOWN, no browserceilingsclaim.


2026-10-14T11:14:04.842486+00:00 — USER_STEERING_ADD_NATIVE_APPS: exact completedWJ build84f851/084169 requested in Telegram, WezTerm and other GTK/Qt windows, with native IME appearance and speed. This revokes only earlier no-further-window-run scope; productionsource staysfrozen848PASS. No remote Telegram orWezTerm installed/authenticated; hostGtk3/gedit/textEditor/GNOMETerm exist, privateguest lacksQt/editors. Provision officialapp packages onremote withinowned profiles, reuse existingfiniteGNOME/IBus/keyboardfixture, no RDP/laptopruntime/IMEroutechanges. Telegramcomposerrequiresuserauth; asyncquestionpending, no message send/Enter/clearinguserdrafts. Newappresults mustseparate currentbytes frompriorWG full343P56F33B browsermatrix; no universalacceptance/platformceiling claims.


2026-10-14T11:42:47.104472+00:00 — WJ otherclient native scope LIVE54582/driver50662, zero attachment PASS exactIME84f851/daemon084169/stockGNOME50/IBus. Newfiniteak1228560/t17536201/session4242VT8. Guardedsameprivatecontext, source/scriptsfrozen6hashes WJ-native-app-protocol.json, 88scenarios290latencies planned; realGtk2/Qt3fields/defaultWezTermPTY/image+TelegramloggedoutUI. User permits actualTelegramdrafts, sendsforbidden; accountstillmissing, no credential/sessioncopy ormessageinputperformed. Full browserWG343P56F33B separate; testsQtframeworknotTelegramcomposer. Runtime/transports/model/userlaptop unchanged; no newcodefixes.


2026-10-14T11:53:02.319349+00:00 — WJ native-app setup1 BLOCKED before device open/keys: host stock IBus discoveryNone; private guest/host machine IDs differ. Archived setup +13hashes /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-setup1-1791978664353212704/ARCHIVE.json; exact5 original ticks gone/CGempty/VT8→1. No appcase verdict (88NOT_TESTED). Test-only decision2026-10-14-native-app-stock-ibus-shared-machine-identity: app-only bwrap ro-bind host machine-id, stock discovery/owner guards retained; GTK observer returnsFalse; early actor errors produce bounded receipts; missing native caret BLOCKED. ExactbuiltIME84f851/daemon084169 and source848 stayfrozen; no global IBus/laptop/production routes/model changes. Next setup must prove ordinary host discovery equals existing private GNOME-owned bus before first key. Telegramdrafts authorized/sendforbidden, no authenticated remote composer; fixtureQt acceptance separate. Latest complete WG343P56F33B432 unchanged.


2026-10-14T11:56:23.548210+00:00 — WJ native-app setup2 confirms stock host discovery now CONNECTED toexactowned private IBus. Nexttestguard failedbeforekeys: default vendor path /usr/bin points hostUbuntu22, while ownedGNOME50IBus uses officialguestUbuntu26. Exactownedsha24338c equalsrootfs-v2privatevendor, differshostcb8cca; no corefailure. Setup2 archive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-setup2-1791978934553235625/ARCHIVE.json; actual15548CLOSED0/driver64007CLOSED1/5GoneCGemptyVT1/88NOTTESTED. Extendtesthelper with explicitprivatevendorfilepath forhostactor only, retainingallotherguards/browserdefault; preflight fullhostguard before firstkey. Source848/buildbytesunchanged;WGmatrix unchanged.


2026-10-14T12:01:16.731291+00:00 — WJ native-app setup3 fullordinaryIBus/owner/package preflight PASS. Actor reached realGTK defaultwindow but aborted BEFOREfirstkey because host Atspi has clear_cache(), not clear_cache_single(); InputDevice needsEV_SYNwrite, notsyn(). MeasuredAPIinventory verifies replacement methods, old nonexistent methodsfalse; script corrected, no runtime source change. Setup3archive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-setup3-1791979158775405222/ARCHIVE.json; actual70082CLOSED0/driver66485CLOSED1/5GoneCGemptyVT1; keys0/rows0/88NOTTESTED. Terminalwholephysicalkey/windowledger enabled andperappledger retained; frozennewtestprotocol8hashesrevision4. source848/build84f851/084169/WG343P56F33B unchanged.


2026-10-14T12:11:30.937575+00:00 — WJ firstnativepartial4:4PASS0FAIL1BLOCKED83NOT_TESTED/88planned, keys380/noheldkeys. RealGTKTextView short8,long8,mixed8,english8 all8gestures+nextletter/backspace PASS; prefix8 BLOCKEDactualnativepreeditcaret unavailable. Archive remote WJ-native-app-partial4-1791979559627252021/ARCHIVE.json. Native screenshot hostCLI timedout8s beforeburst gestures, actoraborted; actual61978CLOSED0/driver66371CLOSED1/5goneCGemptyVT1. Preserve5finalized rows/noretry; remainder83 continueafter boundedtestcollectorrepair. ReadonlyprivateGNOME capturemethodusing officialShell.Screenshot(stream), before/afterFG/pixelSHA; exactremoval restoresunchangedalloriginalinputmethods; noIMEroute/source/runtimeauthority/laptopchange. Source848/build84f851/084169/fullWG343P56F33B remainseparate; TelegramcomposerBLOCKEDnoauth.


2026-10-14T12:25:46.396906+00:00 — WJ nativepartial5 completedbothGTK32:15PASS13FAIL4BLOCKED56NOT_TESTED/88; exactsamecorebytes,632newkeys plusprior380. ActualcompositorPNGs nowcaptured/noFocuschange,4originalinputmethods byte-subtraction unchanged. DefaultQtstartup writesnewIBusdiscovery1334619/addressAp3BE075 replacingbound1331237/vGKsEUpY; strictguardstopsBEFOREQtkeys, firstparentcauseUNKNOWNafterteardown. actual80654CLOSED75(descendantcleanup)/driver90161CLOSED1; lateownedL11descendantgoneBEFOREroot signal0, exact5Gone/CGempty/VT8→1. Archive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-partial5-1791980456252180239/ARCHIVE.json. App-onlysession prewarm/focusdefaultQt0keys beforeoriginal5s/finalGNOMEownedbusfreeze, reuseexactQt process; noafter-inputbusrebind/routeflag/productionchange. Retainall32GTKrows/noretry, continue56. Source848/buildfrozen/fullWG343P56F33B unchanged.


2026-10-14T12:34:57.278877+00:00 — WJ setup6 Qtzero-keyprewarm/focus andfinalGNOMEchildIBus binding PASS, but exactFirefoxbyteguard refusedactualdc8d81e againstoriginalb30896 BEFOREappactor; keys0. Archive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-setup6-1791981057620922261/ARCHIVE.json; actual10085CLOSED0/driver51423CLOSED1/5goneCGemptyVT1. Restoreoriginalcachedofficialtar0d92bf exactfirefox-binb30896 (guarded18641CLOSED0), preservechangedtree/profile, privateDisableAppUpdate policy/newemptytestprofile (no allow-downgrade/guardweakening/IMEroutechange). Finiteak retired/newownedal1393969/t17871395/session4321VT8; native7 willretain32GTK15P13F4B andrun56remaining. Source848/buildpair unchanged/fullWG343P56F33B separate.


2026-10-14T12:40:37.899485+00:00 — WJ setup7 ordinaryQtprewarm/busbind/originalFirefoxbyteguard PASS. Actornewkeys0: Qt widget grab_focus acknowledgment doesnotensuretoplevelactive; exactFGguardstops. actual41643CLOSED0/driver73536CLOSED1/5goneCGemptyVT1,56remainingNOTTESTED. Newprivatecompositorsetupmethod activatesonlyoneexistingwindowofpinnedprocessinsameownedCG; actoralso pinsPID/tick/CG, waitsactualFG+fieldfocus, no SSI/inputauthority/IMEroutechange. Capturemethodremainsreadonly; focusactivationdeclarednativeUIsetup, originalinputmethodsbodiesunchanged. Retainall32GTK including13FAIL; source848/buildpair/fullWG343P56F33B unchanged.


2026-10-14T13:13:40.965631+00:00 — WJ NATIVE88 COMPLETE scoped32PASS49FAIL7BLOCKED0NOT_TESTED. BasicWezTerm inlinepreedit/candidate-at-caret andSpacecommit image pair PASS (PTYemptybefore; preciseприветSpaceafter); focus-renderotherpositions outside88NOTTESTED. Perapp GTKTextView12P2F2B,Entry3P11F2B,Qt5Line/plain/rich each4P11F1B,Wez5P3F. Speed44/290 measured,246missing/13early-marker samples,median75.906/p9586.828/max91.946ms; notkernelrelease/paintproof. Telegramnoauth composerBLOCKED; firstlaunchthreadlimit512 refusal/noceilingbypass, send0, imageactuallyWez/notTelegramproof. Finalactual5286CLOSED0/driver33110CLOSED0/3setupparents0/noheldkeys/5GoneCGemptyVT1. Immutablearchive /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-acceptance-1791983329525303054/ARCHIVE.json35files, rawSHAec139383..., rootvisualassessmentseparate; allprior32GTKresults carriedbynativehash/noretry. Newnativefirstmeasurement/regressionbaselineWGUNKNOWN/runtimefirstrefusalUNKNOWN; no newsource/model/IMEroute/laptopchange. Owningreport docs/development/wj-native-apps-2026-10-14.md. FullWJbrowsersNOTTESTED; oldWG343P56F33B remains. TD121OPEN/reviewNOTRUN/nopublication.


2026-10-14T13:31:13.911550+00:00 — WJcore artifact packaged (guarded14642CLOSED0) andfetched/localall11payloadhashes verified: lay-1.0.82-WJ-core-candidate-20261014.tar.gz8150727B SHAe31fbb4fbc8462d8b41b5fb9b2fa06426f158535f82bd74560d88b65cc1937de. Telegramsolo4CPUaffinity underunchanged512task/20CPU ceiling failsbeforebind/keys: TDWebViewsocket pathname>108 underlongprivateXDG_RUNTIME_DIR. Failedsetup31787CLOSED1/driver82220CLOSED1/ownedCGemptyVT1/0keys/sends; archive remoteWJ-telegram-failed-launch-1791984600374689538/CLOSED.json. AddonlyhostTelegramshort/runtimealias /run/user/1000/lay-tg-6onP5c tosameexistingdirectory (sameinode/device), no endpoint/IMEroutechange; source/binaries/native88/fullWG unchanged. Newownedfiniteam seat replacesretiredal. Actor preparesloginUI only; auth remains userboundary, no composerinputbeforelogin.


2026-10-14T13:45:54.489206+00:00 — Telegramactual7.3.1 loginUI nowvisuallyconfirmed inownedprivateGNOME scope actual96785LIVE, TG1584358/0keyssends0. Separateauth-onlysetup hasnoIME/nativebehaviorverdict; original005browserattachment archived (requiredrestorebeforefunctionalinput). Reducednativeappconcurrency/TelegramCPUaffinity4 withinunchanged512task/20CPUceiling; shortexistingruntimealias sameinode/dev fixes108-byteTDWebView pathnamefatal. FirstdirectAtSpiStartMessaging returnsack butscreenstillwelcome; actuallogin remainsuseraction/authboundary. Authquestionpending; elapsedtime notauth/permission. Source848/corepair/native8832P49F7B/fulloldWG343P56F33B unchanged. Pairedcore8.15MB archiveSHAe31fbb4f...all11payloadhashesverified; no installer/publication/localruntimechange.

## Local Firefox1.0.83 iframe diagnostic — 2026-10-14T15:24:54.088336+00:00

User reports Double Shift failure and explicitly requests checking here on the local machine; identifies actual field10, “Редактор внутри iframe”. This authorizes the local diagnostic exception to the prior remote-only test scope. Installed CLI/IME/daemon bytes and user profile are preserved. Actual Firefox157.0 snap8995, PID767837; own standard test window4063878607, all native input bound to that title/window/PID/starttick, actual field focus and existing daemon-attached AT keyboard. Browser/account profiles unchanged; no native text input into WhatsApp or other user sites. No runtime install/restart, delivery-route change, model fit or learning cleanup.

Exact field10 test with simplytyped `просто`: active roundtrip PASS (both surfaces and next-character decoder/Backspace), post-Space5-gesture burst FAIL. First conversion becomes `ghjcnj ` with native caret7 in108.444ms, then gestures2–5 leave it unchanged. All4 reach a live bridge but manual RPC returns `context_authority`; first refusal at authority.rs706, word_known=false, managed_word_mode=false, snapshot present/unselected, witness absent, manual toggle/refresh both false. Between first delegation and second refusal the newUS owner109 sees Reset serials3246/3256/3260/3276/3279; final3279 has refused_timeout. Second marker313 settles in under1ms, so the observed failure is edit admission after successful dispatch, not a missing gesture. The exact first loss of replay/reset authority still requires a causal source test; timeout alone is not asserted as cause. Do not weaken the guard or use a literal word/frame bypass.

Scoped transformation timing3/7 samples,4 missing; median103.207ms, p95/max108.444ms; evdev write-return→AT-SPI observation, not human-release or paint. Input2plan verdicts1PASS1FAIL; no conversion failure excluded from timing denominator. First observed refusal, raw sequence and all earlier setup scopes retained at `/home/ubu/.cache/lay/development/firefox83-local-sequence-kwjodjvq`; root assessment `/home/ubu/.cache/lay/development/firefox83-local-sequence-kwjodjvq/ASSESSMENT.json`, raw SHA1c6e2634a55e57b0aaa831d0666d33805b3cf8a582142f1a5b0bf5a17b6e98ff. Field1/7 local active roundtrips and field1/7 five-target controls pass in separate scopes; they do not certify iframe10. Remaining8-field actor sent0keys because Firefox was behind Kitty while user replied; classify16plans BLOCKED_SETUP, not product failures. Earlier observer artifacts are explicitly retained rather than silently promoted.

Latest complete WG browser counts remain Firefox164P33F19B/216, Chrome179P23F14B/216, total343P56F33B/432. FullWJ/1.0.83 browser acceptance NOT_TESTED; TD121 OPEN and independent WJ review NOT_RUN retained. One local reproduction finished; no product source edits or rollback.


2026-10-14T16:32:23.959123+00:00 — Local iframe10 follow-up hypothesis: complete native replay delivers target to actual editor, while prior/partial same-client SSI remains unconfirmed (`observed_replay_prior_surface`); next bridge marker is live, manual/refresh admission false. Owner remains existing ContextAdmissionReducer/WindowInteraction. Proposed9/10 reuse same-gesture exact client refresh for only a current fully completed paired replay; zero text effect until actual exact final receipt. Local-buffer promotion2/10 and deadline widening3/10 rejected. Test-only late_native_replay_refresh.rs added under existing residual harness: replay completion, actual Reset and old SSI, next real ManualToggleV3 must request fresh exact receipt without Delete/Commit; exact final SSI then same gesture delegates. Prefix/ASCII-NBSP variants. Baseline1.0.83 coreb742b64/bc89b4, local RESULTkwjodjvq1P1F, full oldWG343P56F33B. No production change yet.


2026-10-14T16:48:35.800357+00:00 — CausalRED exactRPC0vs4 confirmed in iframe-refresh-red-7e7n5ns2 after preserving earlier797PASS and correcting fixtureKnownStart-vsReset provenance check. Manual-only current completed-paired-native refresh implemented via existing RequireSurroundingText/pending_manual_toggle/700ms; final exactSSI required, no sourceKnownStart/candidate/verifier/route/model change. Decision2026-10-14-completed-replay-same-gesture-refresh.json. Telegram user grants any available field/noauth needed; currentSnapATSPItree onlyapplication/nochildren, do not cloneprofile or waitlogin. Installed83 untouched; source/native/review pending.


2026-10-14T17:12:42.739789+00:00 — Completed-replay refresh: causalRED0vs4 retained; minimal four-file manual-only wait predicate, same marked client and final receipt. Focused2GREEN plus full799IMEPASS, six contracts/ASTpending; installed nativeNOT_TESTED. NoReset unmatched projection is an adverse control, not an expanded admission claim. Context-free reviewer spawn rejected threadlimit/NOT_RUN/no score. Telegram existing profile activated normally, any empty nonsensitive field allowed without auth/sends. TD121OPEN/latestcompleteWG343P56F33B/432.


2026-10-14T17:36:14.668777+00:00 — Installed330eda0c... causaliframe10 active2targets+postSpace5 nowPASS7/7 (median115.234/p95max222.170ms), oldkwjodjvq fail retained; source850/850+ASTPASS. Telegram search2endpoint/composer1endpoint visually observed, user explicitly acceptsTelegram; NO MORE Telegram tests. Scope/proof report completed-native-replay-refresh-2026-10-14.md. ReviewerNOT_RUNthreadlimit; full new-byte matrixNOT_TESTED/latestWG343P56F33B; TD121OPEN/no push/tag/release.


2026-10-14T23:03:55.418448+00:00 — Performance authorized: diagnostic two-file daemon timing only, actual first expensive stage stillUNKNOWN. Same fixed actor/native baseline next; old seven mixed samples/observer gaps retained. No accepted local/TG retest or runtime change.


Performance follow-up2026-10-15: independent fresh-context read-only review completed via separate ephemeral Codex after built-in thread quota rejection:9/10, no blocking source defect; receipt `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/PERF-pacing-review/REVIEW-1.md`. Applied its two nonblocking test-only suggestions (explicit2ms×n then16/4ms schedules; prohibit an extra ordinary delete). Runtime production sections unchanged; follow-up source gate pending. Clean PLAIN Firefox original first8 jobs186 rows complete; final30 interrupted at field9 pre-input SHOWING guard, six priorPASS/remainingNOT_TESTED retained in `firefox-functional-c3xe2k7g/RESULT.json`. Added only pre-input750ms visibility wait; no key/target/clock/deadline or guard relaxation, new SHA67f5012c... explicitly pinned. Failed zero-key pin mismatch0behavior input retained. Finish original final30 in fresh exact-owned scope; do not erase interrupted evidence or substitute it for final complete acceptance. Candidate/native speed stillPENDING; accepted local bytes untouched.


Performance attempt1 closed2026-10-15: clean PLAIN Firefox216=170PASS/27FAIL/19BLOCKED. Standard420/420 measured, median106.640ms/p95119.943ms. All216 IDs preserved in `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/PERF-PLAIN-firefox/RESULT.json`. Four-ms daemoncc65fbe7... rejected after normal20=19PASS/1FAIL: field6/space/burst lost a baselinePASS. Same-window clean baseline140 samples median106.081ms/p95117.449ms; shortened candidate137/140 observed samples around89ms, three missing. Speed does not permit promotion. First failed transition is third burst bridge_context_authority refusal (missing witness), before that gesture deletion; causal dependence on4ms NOT_ESTABLISHED. Do not weaken admission or retry unchanged untilPASS. Raw primary `firefox-standard-normal-k9zlr898/RESULT.json`, SHA7963d6c9732369749f1c3e1afe8849c1f4d7800736706e2b287c586b81587449, rejection `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/PERF-FAST4-REJECTED/RESULT.json`. Incomplete private job stopped; all pinned GUI gone/scopeempty/VT1. No current full candidate matrix, ChromeNOT_TESTED. Next single bounded replan8ms (predicted8ms saving); no other production change. Fresh327 source check running. Accepted local runtime remains unchanged; no model training or route edits.


Eight-ms Firefox A/B complete: all216 stableID verdicts identical (170PASS/27FAIL/19BLOCKED), no lostPASS or changedverdict. Same-source actors/60standard+6panel+150functional, closure5pinnedgone/CGempty/VT1/noheldkeys. Standard420/420each: median106.640→93.547ms,p95119.943→115.286ms, pairedmedian improvement11.133ms(n420). Panel48/48each:102.928→92.392ms; functionalDoubleShift134/210each:104.109→93.041ms,p95140.378→131.893ms,76missingretained. Space37/50each:17.538→18.004ms, pairedchange-0.243ms; no speedupclaim forSpace. Tab0/10 and selectionbaseline0/10/candidate1/10 remain insufficient for speedclaims, neitherconvertedtozero. Compare receipt /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/PERF-FAST-firefox-vs-PLAIN.json,SHAe0ea886f21657229fd92c4840a13ef74a22e2c89acb8378111f917282161c8b3. Raw9primaries/logs/bodies retainedlocallyBDIR/PERF-FAST-firefox and remotelyRUN/PERF-FAST/firefox. Nativeclock evdev write-return→exactAT-SPI, notkernel/paint. PLAINChrome nowrunning; both-browsercomplete current-source verdict stillPENDING, latestcompleteWGmatrixretained. Acceptedlocalruntimeunchanged.


Clean PLAIN BOTH baseline complete: Firefox170PASS/27FAIL/19BLOCKED +Chrome179PASS/23FAIL/14BLOCKED =349PASS/50FAIL/33BLOCKED of432,0NOT_TESTED. Exact acceptedBC89daemon/330IME in both, same private frozen data/usage and fixed216/browser; compare with futureFASTboth, never mix phases. Local primariesBDIR/PERF-PLAIN-firefox/RESULT.json and PERF-PLAIN-chrome/RESULT.json; remoteRUN/PERF-PLAIN/{browser}/RESULT.json. Chrome standard406/420measured,14missing retained:median92.055ms,p95111.443ms. ChromeprivacyPASS_VISITED_LAY_SINK_ROOTS normalpositivecontrolrecent_actions.jsonlchanged/privateunchanged (localBDIR/PERFPLAIN-chrome-private-sink-proof.json); scopevisitedLaywriterroots only. Closureall5goneCGemptyVT1/noheldverified. Latestcompleteboth table now this349/50/33baseline, replaceshistoricalWG343/56/33 forstatus. New8msChrome running; final8msboth gatePENDING. Runtimeauthority/installedbytes unchanged.


Eight-ms attempt REJECTED after fullBOTH: Firefox170P27F19B,Chrome178P24F14B,total348P51F33B/432,0NOT_TESTED; acceptedcleanbaseline349P50F33B. Chrome lost exactlyfunctional/normal/space_ru_en/field8: expectedlaySPACE, actualдфнlaySPACE. FirstobservedfailureSpace target, not a DoubleShift action; exactcausaldependenceon8msUNKNOWN, no waiver. Chrome406/420standardmedian92.055→86.585,p95111.443→102.286,paired6.147ms; privacybothPASS. Gains do not justifylostPASS. ComparisonlocalBDIR/PERF-FAST-chrome-vs-PLAIN.json, raw9primaries/logs/bodiesBDIR/PERF-FAST-chrome andremoteRUN/PERF-FAST/chrome. Closureall5gone/CGempty/VT1/noheld. PreparedinstallergatesrejectlostPASSand wasNOTRUN; acceptedlocalBC89daemon/330IME unchanged. RevertONLYown3-filepacingincrement toexactaccepted83copies, retainallfailedsource/build/native/reviewproofs. Do not silentlyreintroduce4or8ms or retryuntilPASS. Next bounded alternative: measuredread-onlyIBus engine query cost5.362msCLI vs0.386msnative; inspectexistingconnectionfirst, no input-route/owner/cache changes withoutdecision.


ObservedIbusEngine source1133/1133PASS,0fail,272.7s; local/home/ubu/.cache/lay/development/run-trdx7u5g/RESULT.json,remote/home/e/projects/lay-development-runner/run-d9azvU,archiveab3839443c57f28ae5f69da22a9cd31f6a83a25cfcf2e047ece79d2c8f63bf03. Selectedbothprivatebinaryowners+sixsharedcontracts, notlibrary/releaseacceptance. Tennewtyped-wire/realRPC/fallback/mismatch/timeout/latereply regressions; actualbackendfresh whileSharedprojectionpoisoned, nogetterstateaccess. Newconsumeronlyverify_engine_once. Query150ms usingexistingstdpoll/async_ioTimer, existingdaemon250ms; legacyerrorfallback remainsoldunboundedCLI. Generalread/ensure andphysical16ms/2msunchanged. Firstcompilefailureretained(run-tqfufdi9); twoAPI/dependencyassumptionscorrectedwithoutnewdependency. NoGetGlobalEngine subscriptions in existingnarrowadmissionstreams(adapter.rs:199-245), no exemptionadded. Newbinarybuild/review/nativePENDING, no localinstall, rejected4/8msstillrejected.


Observed-engine finalsource1134/1134PASS,0fail,271.9s; local/home/ubu/.cache/lay/development/run-3recp7ey/RESULT.json,remote/home/e/projects/lay-development-runner/run-dWWz50,archiveaf0d9b36e84e9612b54708a485c2df608fb0a82afeedd7a6135fd20ccea67d4f. Finalcontext-freecodereview2=9/10,noblockingfindings,receiptBDIR/PERF-readback-review/CODE-REVIEW-2-RESULT.md. Review1=7/10rejectedunsafe150mswhole-callcancellation+uncheckedDict. Corrected: no backend cancellation, one pending observation viaRAII atomic resourceflag; existingdaemon250msONLYreplywaitafterfullysentsessionframe, serverhandlerdetached. BackendcompletionandlegacyCLI remainunbounded,no150ms/overall250mspromise. TypedattachmentsEXACTa{sv}. FivefocusedtestsPASS includingactualone-bytepartialwrite/backpressurepastcaller250, continuedframeandfreshfollowingquery,latebackend350replynotcached,busybudget1,poisonedSharedandtypedwire. Nonblockinggapbackenddisconnectnotdistincttest(clientdisconnect/errorcleanupcovered)retained;notnativewaiver. GeneralCLIread/ensure,allinput/outputpaces16ms+2ms,modelsanddeliveryrouteunchanged. Newdefaultbinarypair/zero-keyactualRPCparity/native432+privacy stillPENDING. Nothinginstalled.


Safeobserved-engine defaultpairbuilt183.905s aftercanonicalAST/architecturePASS. IMEf633f6aac492817031b025c02c8746a27eac6541f345ea60873415a4b52fedc3; daemonba6d9d77163a13284c3540481806c8b710570160bb8e571164309014a318a19e; immutableRUN/PERF-read-build/BUILD.json,frozenfinalrun-dWWz50/source+explicitmetadataoverlays. Threephysical-outputsourcehashesexactaccepted16ms/2ms;no codechangedafter1134/review9. Fourgraphartifactsfetchedverified. READFirefoxzeroattachmentPASS,newpairloadedwithpinnedUID/CG/PIDtick/peer/windowIC. ActualObservedIbusEngine30pairsallmatchCLI,zero keys:RPCmedian0.541ms(first1.542),CLI5.398ms,p950RPC1.199/CLI6.130. APIprobeR/PERFREAD-firefox-engine-readback.json,notnativeinputspeed. Original9READFF216running;no install/localruntimeauthoritychange.


READFirefox attempt1 incomplete duepreinputvisibility: originalnormal20PASS,private18PASS+field9setupBLOCKED+field10NOT_TESTED,remaining176unqueued(aggregate38P1B177NT/216). No field9behaviorinput/runtimefailureobserved. RetainedRUN/PERF-READ-visibility-blocked-1 includingalloldactors/pins/rawreceipts. Strengthenedonlynativefocus_editor precondition: ownedforeground+scroll_toANYWHEREifhidden+originalfocus+750msSHOWINGwait. Guardunchanged; scopevalidationexactnonfocuscodeparityPASS,0keys; newexplicitactorhashes standard4678321d/functional33091ab0/panel850f2b1a. No source/binarychanges: samef633IME/ba6daemon,source1134/review9valid. WholefreshREADFF216restartedtoavoidmixingdifferentharnessmetadata, unitread-visible, matrix43682/GUI33921; oldpartialneverpromotedorreplaced. Decision2026-10-15-browser-fixture-native-viewport-preparation.json,publicprotocolupdated.


READ Firefox attempt2 NOT_PROMOTABLE / INCOMPLETE, retained RUN/PERF-READ-firefox-incomplete-2 (all9 primaries, jobs, actor versions, candidate metadata, logs; local BDIR mirror). Counts156PASS23FAIL15BLOCKED22NOT_TESTED/216. Fullstandard60cells andpanel6PASS, standard420/420:median98.687ms,p95111.197ms vsPLAIN106.640/119.943. Preliminarygain only, notbothbrowser acceptance. LostoldPASS functional/normal/prefix/field1: firstRU→US16.552ms, reverse textcorrectonfailure snapshotbutmode evidence absent inoldactor; daemonlog says focusedIMEhandledbothgestures, nofallback. CauseUNKNOWN; donotwaiveorinstall. Lastjobfield9/space_ru_en BLOCKEDbeforebehavioralinput: hiddenafterfocus/setup, remaining22NOT_TESTED. Boundprocessesgone/CGempty/VT1. Newboundeddiagnostic: oneprefixroundtrip with existingmodeRPCsamples;15preinputiframefocus/layoutsetups with nativeobjectidentity/states/rect, zero behaviorletters. No production sourcechanges/rebuild/review or localinstallation; source1134/review9 unchanged.


Matched native initial-stack protocol prepared, decision2026-10-15-matched-native-initial-stack-readiness.json. Exact frozen Rust parity accepted83run-Qq20tO vsgetterrun-dWWz50 verified: exactly6 intended source files differ, no physical-output changes. Old prefix loss remains unresolved/causalityUNKNOWN. Bounded diagnostic15preinputsetups/300SHOWING samples+freshobjects didnotreproduce; parent finalizationNameError __file__ meansnotcompletedPASS. Newharness strengthens preinputreadiness100ms focus/SHOWING + independent source/mode/CLI agreement withoutactivation; records existing targetmode read, no newtimedRPC. Fixed single matched PLAINV+READV216/browser, no unchanged retries; allhistorical losses retained andmustbeexplained beforeinstall. Productsource1134/review9unchanged.


PLAINVFirefox full216 complete170PASS27FAIL19BLOCKED,0NOT_TESTED; unchanged acceptedBC89daemon/330IME, newstrictsetupharness. Original9elapsed776.247s. Iframe last30complete20P5F5B; prefixfield1PASS with observedRU. AllboundgoneCGemptyVT1. Primary RUN/PERF-PLAINV/firefox/RESULT.json. PLAINVChrome nowrunning; do not mix this newFFrow with olderChrome phase. Fourfixedphase sequence continues with no core/source/review changes.


PLAINV BOTH full control complete: Firefox170P27F19B +Chrome179P23F14B =349P50F33B/432,0NOT_TESTED. RUN/PERF-PLAINV/{firefox,chrome}/RESULT.json; identical strictsetupactorhashes/frozendata; original9elapsed776.247/774.219s. ChromeprivacyPASS_VISITED_LAY_SINK_ROOTS, normalrecent_actions.jsonlchanged/privateunchanged3files. BoundgoneCGemptyVT1/noheldeach. This is latest COMPLETE BOTH status, replacing rejectedFAST8 table. READVFirefox newf633IME/ba6daemon nowstrictzeroPASS;30actualgetter/CLI namesmatch, RPCmedian0.649738/p950.757272ms vsCLI5.469076/5.757048, firstRPC1.274141ms. Not native latency/acceptance. Newfull216FFthenChrome remainsrunning. No source/core/review/install change.


READVChrome firstfunctional30:19PASS8FAIL3BLOCKED versusPLAINV19PASS7FAIL4BLOCKED. Exactchangefunctional/normal/undo/field2: baselineBLOCKED/setup exactfunctional seed(rawпревеликое,caret6);candidate obtainedexactпревет seed, thenSpace targetFAIL(rawпреветSPACE,caret7). No baselinePASS lost inthisgroup. This isadditionalobservedcoverage of previouslyBLOCKED behavior, notproof of regression orrepair. KeepB→F explicit; no correction/model/routechanges, and no retry ofthiscase. FullChrome216stillrunning; release84metadata onlyPREPARED/notbumped, no installation. VERSIONING.md requires synchronizednewreleaseversion; ifremainingproofgatespass, prepare84coreCLI/daemon/IME+extensionmetadata asbyte-identicalRust-source/dependencylogic descendant oftested83. Reusefunctionalproofpermetadata-onlyexception; build3corebinsremoteunderexistingguards, no broadbump-script/modelrefresh.


READV BOTH full native complete349PASS51FAIL32BLOCKED/432,0NOT_TESTED. All349historical/matchedPASS retained; Chromeundo/field2 BLOCKEDsetup→FAILSpace explicitlyrecorded. Firefox420/420107.254→98.507median118.259→111.112p95,paired8.235;Chrome406/420(14missing)92.161→83.306median112.631→100.073p95,paired9.801ms. BothprivacyPASS. Lastguardexit75cleanupretained; onlyownedprivateL1.1PID3971292/tick23111169 closedbyverifiedpidfdSIGTERM,CGempty/5boundgone/VT1/seatclosed,0repeatedinputs. Full9COMPLETEreceiptscollected unchanged afterrecovery. Decision2026-10-15-observed-readback-release84-metadata-only.json. Sourceversion84metadataONLY manuallysynchronized perVERSIONING; installed83unchanged(SOURCE_BUMPED_RUNTIME_STALE), full3corebins84buildpending. AllRust/input/dependencies/extensionlogic parity required; no newfunctionaltests forversion-only change. No models/globalIBus/sourceIDs/deliveryroutes changed.


TD-141 COMPLETE / installed1.0.84. Build RUN/release84-build/BUILD.json:232.239s, all739Rustfiles/dependencies/extensioncode exactlyequal tested83 except version surfaces, canonicalarchitecturePASS. CLI60de3f20…, daemon6d0237e6…, IME7e338ae2…. Local transaction /home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/install83-20261015T033053Z/INSTALL.json: CLI/tray84 and/procSHAverified,24protectedfiles, globalIBus/L11/L3processidentities+config/source/mode unchanged,0functionalkeys. NativeREADV432 logicproof349P51F32B/0NT,0lost349previousPASS, Chromeundo/searchB→F retained; metadata-only84 proofcarryexplicit, no localfunctional84claim. Freshsource/harnessreviewsfinal9/10(twoeach),1134sourcePASSunchanged. TD121legacyfunctionalworkremainsOPEN. No models/deliveryroutes/runtimeauthority change; endpointremainsfreshread-only onexistingconnections.


## TD-147 opt-in shortcuts — before production edit

2026-10-16T08:25:11.537592+00:00; baseline05e1e448 source88, installed87 unchanged. Hypothesis: new opt-in Backspace must reach the existing exact undo owner before next-key invalidation; double-Space needs ordinary boundary accounting and exact suffix count before the existing gesture dispatch. First failures and accepted plan8/10 are recorded in tech_debt/147-explicit-undo-and-double-space.md and decision2026-10-16-opt-in-existing-owner-shortcuts.json. No early KnownStart veto, parallel legacy detector, transport bypass, models or training changes. Start with real executable RED; source, native, timing and installed evidence remain separate. Historical complete matrix349P51F32B/432; new bytes NOT_TESTED.

## TD-150 — completed native replay Reset capture, focused verdict

2026-10-16; baseline e6f3227c, built88/installed87. The single same-byte full
Firefox confirmation completed216 at169P28F19B but lost fullscreen/space/field8;
its per-cell trace records late matched Reset without a predecessor and a live
bridge context_authority refusal. Both complete E6 Firefox receipts and original
short4 failure remain immutable; no third same-byte full rerun. Existing deferred
observer/callback stream reproduces the exact authority failure in
td150-observer-red-wtmv6a4m,2/3 controlsPASS. Remove only the whitespace rejection
in the existing completed-replay capture fallback, preserving all proof gates.
td150-focused-proof-vfotdhss11/11 PASS, controlled old guard1/1 detected; fresh
plan9/10 and independent code review9/10(pass1), root all implementation.
Full source3210 expected is running; no new-byte build/native/install claim.
[Owning task](../../tech_debt/150-retain-completed-native-replay-across-reset.md)
contains exact hashes, failed fixtures, adverse cases, consequences and limits.
No early KnownStart veto, new owner/route/timeout/model/learning change.

TD-150 full source verdict: td150-full-source-5m8befqy3210/3210 PASS,
3174 correctness+36 package, zero failures/exemptions,694.58s. Manifest3236
keeps all old classifications/15ignored/11performance and adds only the causal
regression. MSRV1.88 both configurations, strict lints530/354 unchanged, canon15,
architecture/audit50, Node/harness/panel PASS;994 tested files matched locally.
Fresh source review9/10; final canonical/matched build/new native pending.
Both original rejected E6 full Firefox receipts remain unchanged.


## TD-151 / TD-152 — new b411 release blockers before production edits

2026-10-16. TD-150 final build a0620bf..., IME d0ef253..., Firefox complete
169P28F19B/216. Captured fullscreen/space8 repeat now5/5, standard60/60,
420/420timings median98.694ms, p95112.273ms.
One lostPASS functional/normal/space_en_ru/7: exact inline preparations14/15
at epoch11522 precede matched Reset46045, pairedrelease and Space46047;
lookup NotReady worker15/wait0. Trace /home/ubu/.cache/lay/development/td150-install88-20261016/space7-failure/firefox-functional-l3nq1f8n-7-space_en_ru.jsonl.
Preparation trace alone precedes finish_request; causal test must establish
stored readiness and precise identity loss. Hypothesis: temporary post-Reset
capture mismatch retires a same-word prepared slot before restoring the exact
owned observation. No runtime edits; actual adapter RED pending. Chrome full
continues under sole remote heavy lease; do not splice the current pair.
[TD-151](../../tech_debt/151-preserve-prepared-space-frame-on-matched-reset.md).

Separate CI38901069439 failed overlapping_handled_releases_publish_only_the_latest_completion;
no raw panic/artifact, same remote3210 passed. Diagnose exact material readiness
and bounded callback order without assuming a runtime defect or adding retries.
[TD-152](../../tech_debt/152-control-overlapping-release-test-readiness.md).
Installed87/config/models/routes unchanged, installation blocked by both gates.


B411 complete matched new pair: Firefox169P28F19B / Chrome179P23F14B /
348P51F33B/432,0NOT_TESTED, both9jobs/emptyerrors/cleanupVT1. Only new
lostPASS Firefoxnormal/space_en_ru/7; Chrome0lost. TD-150 oldfullscreen8
now5/5; FFstandard420/420median98.694,p95112.273ms, Chrome406/420
(14missing)82.307,p9597.386. Release/installation blocked pending TD-151/152,
no unchanged-byte repeat. New source-only causal diagnosis snapshot frozen
3fc2c350..., td151-reset-diagnosis-359fg2lz, contains tests only/no runtime edit.


TD151 causal RED td151-reset-diagnosis-359fg2lz2/3PASS17.25s: real inline
exact-ready before matched Reset; after Reset slotgone although pending/current
frames exactly equal pre-Reset prepared. Old exact-refresh and CI baselinePASS.
First loss is existing soft-reset retention before owned observation restoration;
prefer existing stable pending predecessor frame, then former managed-echo
fallback. Decision2026-10-16-retain-stable-pending-space-frame-on-reset.json;
no changed identity guard, verifier, timeout, route or model. Green/control/full
and new native pending; installed87 unchanged. CI originalcause stillUNKNOWN.


TD151/152 fresh full source td151-full-source-gvt96j4h3211/3211 PASS,
3175correctness+36package,0failure/exemption,693.46s, archive844618fc... .
MSRV1.88both/strictlints530-354same/canon15/architecture-audit50/Node/harness122
(oneexplicitSkip)/panelPASS. Newmanifest3237 addsone TD151regression only;
oldclassifications/15ignored/11performanceunchanged. Finalgraph+test-only TD152
encoding assertion check pending, then matchingbuild/native/CI; installed87.


TD151 final canonical td151-final-canonical-1gumzyss PASS48.56s,17currentPython
encoding/fixed-only cases +15canon +15privatehelperAST;4canonicalmetadatafetched,
993functionalfullproofinputsidentical. Final independent TD151/TD152 code reviews
both9/10 pass2/2/noP1-P2; newcontext-free thread unavailable(platformlimit),
reused unrelated reviewthreads explicitly. Rootallimplementation. Source tasks
complete; genuine newCI/matchingbuild/native432+options80/install remainpending.
All oldrejectedreceiptsretained; no route/model/deadline/learning change.

## TD154 — consecutive owned Space acknowledgement, before production edits

2026-10-16T20:19:23Z. Baseline fe614235 / installed release88 IME8cb7bca... .
Default whole432 remains349P50F33B. Optional normal80 remains58P18F4B;
Double Space40=26P14F. Read-only preserved trace35041eb5... field7 rapid10:
first owned Space confirmed asNBSP, second owned Space then matched Reset and
adjacent SSI rejected despite matching identity/length/cursor. The source
acknowledger forbids a previously observed separator immediately before its
new final Space. Hypothesis: retain exact previous separators and acknowledge
only the newly appended character using current owner/receipt guards.
[TD154](../../tech_debt/154-acknowledge-consecutive-owned-space-boundaries.md)
records alternatives9/3/1, scope, adverse tests and native limits. Six FF rich
cases share this mechanism. Four Chrome failures are strict ASCII target
comparison despite projected surface; two fail only a later probe. Two FF
release-order cases remain independent, no causal repair claimed.
Causal source RED and plan review pending; zero new native keys/runtime edits.

TD154 causal RED td154-consecutive-space-red-bcubbzg9:1/2PASS14.724s,
archive6c666828... . Original single-boundary cases pass; actual second complete
Space callback with adjacent exact SSI leaves abc+NBSP+ASCII instead of observed
abc+NBSP+NBSP. Existing adverse receipt test PASS. Production unchanged; causal
transition identified, new protected-path decision recorded before runtime edit.

TD154 first green td154-consecutive-space-green-zftayg2t10/10PASS17.231s:
second owned boundary retained under exact raw previous-prefix equality;
mixed/multiple boundaries and all existing/new adverse controls PASS. One
runtime expression; fresh-context plan review8→9/10, two passes complete.
New1.0.89 metadata prepared, installed88 unchanged. Controlled reversion and
full source gates next; no new native success claimed.

TD154 full preflight _7ih78_h stopped before full tests at Clippy's new fixture
MutexGuard/await diagnostic. Replace explicit drop with lexical scope, retain
all assertions and strict lint gate. Controlled one-expression rollback did
detect original second-boundary failure; no runtime revert or guard weakening.

TD154 final full source td154-full-source-final-50j7ij8d PASS3211/3211,
0failure/exemption,715.353s; archiveb2793f28... . All3237 IDs unchanged, both
actualMSRV1.88/strictlints/canon/architecture-audit50/Node-harness-panel PASS.
994 functional input hashes verified locally, controlled old guard RED detected
and exact source restored. Actual private observer predicates17/17 PASS including
ghost refusal, zero native keys. Independent code review/new build/whole native
and installation pending; installed88 and historical349/50/33 unchanged.


TD151 new-byteFirefox216 complete170P27F19B/zero lostPASS, Space7 nowPASS;
standard60/panel6,420/420DS median98.548/p95111.391ms. ChromeREPAIR2running,
source e34/build119269.../IME8cb7bca..., installed87. Source3211 unchanged.
FreshCI38908900651failed exact oldoverlapfixtureNone!=Some(ерка) atlastrelease,
no timeout; newboundedreportingexposedactualpanic. Security38908900839PASS.
[TD153](../../tech_debt/153-establish-overlapping-release-material-readiness.md)
now owns controlled readiness proof/test-only repair; no runtime edit or CI retry.
If finaltest-onlybuild allsix installedpayloadsarebyteidentical, explicitlybind
wholeoldnativeevidence ratherthan repeatinput; otherwise changed-bytegate required.


TD151 paired e34 native complete349P50F33B/432, NOT_TESTED0; FF170/27/19,
Chrome179/23/14, zero lostPASS in each vsREADV. Both scopes empty/five pinned
processes gone/no heldkeys/VT1. Chrome406/420standard DS med82.533/p95100.712ms,
14missing retained. TD153 plan review1/2=9/10; narrow cfg(test) exact-input/cache
accessor allowed, actual frame/display prerequisite + MISS/materialize/HIT proof
required before fixture repair; original CI missing-cache cause not yet proven.


TD153 controlled diagnosis td153-readiness-diagnosis-hvepdrsr PASS25.411s:
unchanged baseline PASS; exact authority/display-ready + usage-refresh cacheMISS
reproduces same None/Some(ерка) RED; actual materialize/cache-onlyHIT makesboth
releaseorders PASS, originalsourceSHAs restored. OriginalCIcachecause unproved.
Minimal cfg(test) prerequisite now owns readiness atlastrelease, no runtime edit;
plan review2/2=9/10, current focused/full/CI/equivalence pending.


TD153 focused11/11PASS21.95s; code review1/2=9/10/noP1-P2. Full private-helper
preflight falsely marked unchanged manifest as changed, canon rejected correctly
before full tests; actual3237 IDs/SHA unchanged. Corrected helper does not weaken
canon and resumes remaining gates from byte-identical fe493... frozen source,
reusing only independently recorded successful MSRV/lint/discovery checks.


TD153 final full completion PASS3211/3211/0fail/exempt,564.631s, all3237IDs
unchanged,994proofinputs verified. Exact snapshotfe493... priorMSRV/lints reused;
canon/architecture/audit50/Node/harness/panel PASS. Private6payload whole432
equivalence +mandatoryoptional80/CI-firstattempt-main qualification now explicit;
actualnewbuild/equality/options/CI/install still pending, installed87 unchanged.


TD154 source76361aff built89candidate203.644s, not published/installed. First
optionalFirefox40=25P12F3BLOCKED_PRECONDITION, complete native-shortcuts/firefox/RESULT.json
under td154-install89-20261016. Raw trace field7rapid10 confirms ACK24/46,
receipt25/47, consumed56, exact delegation57/status3at59. Daemon log rejects
exact replay requires preserved ASCII spaces or one acknowledged Space. TD154
producer repair confirmed through handoff; final replacement NOT_MET. TD155
owns the next existing exact-replay consumer, no producer revert. Default432
and Chrome optional newbytes NOT_TESTED; old349/50/33 remains unchanged.

TD155 causal unchanged-production td155-exact-series-red-lggpkxju:5/6PASS,5.310s,
archivedf8eb00b703b1be766b1bfd2e108b6b16b363ace80e70291810ed040d9e62b4d.
Extended actual existing executor regression fails two-NBSP representability
after single-boundary controls; other five current safety controls PASS.
Production unchanged; next uniform exact literal/count consumer validation
requires explicit decision and actual replay-stream regression.

TD155 first focused11/12PASS17.342s, archive89cb6922..., actual daemon6/6PASS.
Actual IME completed multi-Space stream loses exact manual authority at final
raw two-NBSP receipt: existing outer acknowledgement and paired-tail mirror only
change final Space. No new native run. Task amendment requests existing complete
replay witness for exactly its newly emitted Space positions, preserving byte-
exact old prefixes and every current owner/counter/provenance guard; fresh plan
review pass2 and new protected-path decision precede any IME runtime edits.

TD155 second focused mq6xj6cd:11/12PASS16.740s, archive89875b89... .
Stopped at a new negative fixture assertion: missing boundary is inert
AwaitingExactSnapshot(status4), not NotHandled(status0). No text mutation or
daemon delegation occurred. Preserve strict0 for all original adverse cases;
new contradiction assertions permit only inert0/4 with zero text effects and
no authoritative handoff. Do not change runtime to satisfy an overspecified
fixture. Complete multi-NBSP positives remain unproved until next run.

TD155 focused td155-series-green-9qqzqfio:12/12PASS25.671s,
archive1dc6f8200beb96ac4fd2ec432df723f5c243cb36c406ca115b4f658365f7ed89.
Uniform exact consumer + existing completed paired Space-series acknowledger
pass real callback/Reset/raw-SSI and strict contradiction controls. No state,
route/model/config/default changes. Full controlled/source gates pending; no
new native or installed evidence. Only later test failure diagnostics add the
explicit boundary count; assertions/runtime unchanged.

TD155 full wstyurem FAIL716.943s: unchanged3237IDs/no waivers; IME827 has2
old owned-prefix regressions. New outer candidate confused full owned tail with
pending token suffix. Correct geometry uses suffix only, retaining exact old
prefix for the same tail/scope owner. Both original one-boundary failures and
new two-boundary owned-prefix controls join focused proof. No publication/build/
install and no producer revert. First independent code review9/10 did not
identify this gap; pass2 required after corrected source proof.

TD155 corrected prefix-focused-jnf0564c14/14PASS30.197s, archivedf903f2a... .
Both old one-Space/owned-prefix failures and added two-Space owned-prefix
variants pass. Geometry compares only current pending token suffix; full owned
prefix stays exact for the same atomic tail/scope owner. New full will detect
three independent counterfactual restrictions/errors, no waiver or route change.

TD155 final g48ali01 sourcePASS3211/3211,0failure/exemption,730.988s,
archive47036521... . All3237IDs/classifications unchanged, actualMSRV/lints/canon/
audit50/Node-harness122/panel PASS. IME827/827 including both old regressions;
daemon294/294. All3counterfactuals detected and source exactly restored.
994proof inputs matched locally after verified generated/lint metadata sync.
Final code review2/canonical+observer17 and new89 build/native/CI/install next.

TD155 final canonical rwecv4fk PASS48.657s, archive9cd6b176...,993functional
inputs equal full g48ali01;18helper AST/hash identities and actual remote
observer17/17PASS,0nativekeys. Generated graph copied back; no runtime delta
after full3211. Final code review2/build/new native/CI/install pending.

TD155 final fresh-context code review9/10 ACCEPT pass2/2, noP1/P2,
/root/td155_space_series_final_code_review. Corrected suffix/full-tail boundary
and18helper pins verified; no runtime delta after final3211. Source task DONE;
new matching build/native/CI/release/install still pending, installed88 unchanged.


TD155 source8562c2bb published normally to public/main and both working branches;
matched89 build83.417s archive48ce41a5d4ad8b23e16be3a06dfef6205879313f3c6a0452964493801cfb0737,
daemon9b9f7dca / IMEb69b4ba0. BASE td155-install89-20261017 holds BUILD/REVIEW/
SOURCE-PUSH. Setup metadata errors caught before any functional job: invalid numeric
seat label, then reused TD154 zero-key output name. Alphabetic guards retained;
only fresh private names changed, recorded separately from independent source
review. Original partial40 NOT_TESTED and prep retained as SETUP-ATTEMPT1;
current cgroup/pinned processes/keyboard owner independently confirmed gone before
restaging. No false native PASS; first actual Firefox optional40 now running.
Security main first attempt succeeds; CI running. Installed88/source3211 unchanged.


TD155 actual first functional Firefox optional40 COMPLETE27PASS/10FAIL/
3BLOCKED_PRECONDITION,0NOT_TESTED,5jobs. Single pair10/10PASS including7/8/10;
rapid10=3/10, undo-first7/10, undo-second7PASS/3BLOCKED. Old installed88 FF40
25P12F3B gives5gains but3lostPASS (rapid fields1/4/6), no causal regression verdict
or unchanged retry. Whole contrast OPTIONAL-FIREFOX-CONTRAST.json. DoubleSpace
latency106.802median/137.423p95ms,33/60 measured,27missing; not an all-client speed claim.
Captured one failed field2 rapid second pair in bounded ring: marker nonce53
sees unsettled callback741, bridge rejects with18.494ms left in20ms; same Space
release then enters/settles normally. No replay delegation for this failed gesture.
OPTIONAL-FIRST-BRIDGE-REFUSAL.json SHA da9bef0a41bf50b7c183d6b845e279653ad6e4db1abe9502fcd83d53e5bac806
under td155-install89-20261017; trace completeness UNKNOWN. No admission edit yet;
all lostPASS causality UNKNOWN. DefaultFirefox216 now running; standard normal/
private40/40PASS is partial evidence only. Installed88/default349P50F33B preserved.


TD155 new-byte default Firefox216 COMPLETE170PASS/27FAIL/19BLOCKED,
0NOT_TESTED, all9jobs and cleanup valid, no lostPASS or gains vs whole88 REPAIR2.
Native DoubleShift420/420:98.465median/111.253p95ms; functional134/210 measured,
76missing, retained separately. BASE td155-install89-20261017/native/firefox/
RESULT.json and firefox-vs-REPAIR2.json. Chrome216 now running; latest complete
both-browser table stays old349P50F33B until new Chrome whole completes.
CI38972539257 and Security38972539134 exact8562c2bb firstattempt SUCCESS,
BASE/CI.json. No source edit, v89 release/install still pending; installed88.


TD155 new-byte default BOTH COMPLETE349PASS/50FAIL/33BLOCKED/432,
0NOT_TESTED. Firefox170/27/19, Chrome179/23/14; whole comparisons both0lostPASS/
0gains, all18jobs COMPLETE, no primary splicing. BASE td155-install89-20261017/
native/{firefox,chrome}/RESULT.json and browser-vs-REPAIR2.json. Native standard
Firefox420/420:98.465median111.253p95ms; Chrome406/420:82.182/98.873ms,14missing.
Functional/other timing coverage remains separate in full receipts. Firefox
optional40 already27P10F3B; Chrome optional40 now running on same exact build.
No release/install yet, no source changes after final3211 or first-attempt CI.


TD155 new whole native432=349P50F33B,0lostdefaultPASS, optional80=64P12F4B
(FF27/10/3, Chrome37/2/1),9observedgains/3lostoptionalrapidPASS recorded explicitly.
Chrome DoubleSpace60/60 observations98.756median118.154p95ms; both remaining
ChromeFAIL are field2 Backspace-return extra raw`zoom`, not failed flips; third
case undo-second seed BLOCKED. Entire native/CI/review gate qualified and owned
finite seats CLOSED. First install FAILED panel version and automatically restored
six88 files, recovery VERIFIED; loaded old hashes independently confirmed.
Root cause exact APP_VERSION='1.0.88' while Cargo/metadata='1.0.89'. Fix only exported
version literal, preserve3exact native-tested binaries/all old receipts. ADR
2026-10-17-panel-version89-metadata-alignment.json and PANEL-VERSION-ALIGNMENT.json
pin old/new bytes; private aligned installer requires full exact parity with that
single reversible literal difference, fresh output/backup, original limits intact.
Metadata graph refresh/package review pending; no rebuild/functional retest needed.


TD154/155 final installation1.0.89 VERIFIED, publication pending. Whole new
native432349P50F33B with0lostdefaultPASS; separate normal80=64P12F4B,
9gains/3lostFFrapidPASS and all prior receipts retained. Source3211/3211,
source review9/10 pass2/2; metadata package review9/10 pass2/2 is separate.
Metadata canonical fy4002ss PASS48.443s:992exact inputs+one APP_VERSION literal,
3core binaries unchanged; graph/links/decisions valid,0newfunctionaltests/keys.
BASE td155-install89-20261017/install-aligned/INSTALL.json verifies CLI/daemon/
IME/panel89, loaded IMEb69b4ba0/daemon9b9f7dca,23protected files and identical
globalIBus/L1.1/L3 process identities, config, sources and RUmode. Initial failed
INSTALL.json remains immutable with verified88 rollback. No delivery/model/input
route changes or localfunctionalinput. All owned remote finite seats CLOSED.
Source/native mechanism scope complete; retained compatibility failures belong
also to TD121 and do not become PASS. Source-only public tag/release next.


Publication verified 2026-10-16T22:44:16.720061+00:00: source-only tagv1.0.89 ->
e9d40144fff79c59ea7189599b6957b2cec326bb, public releasehttps://github.com/radislabus-star/lay-public/releases/tag/v1.0.89.
Normal publicmain+both working-branch pushes/readback, private origin/main untouched;
no force/delete. #46 material updatehttps://github.com/radislabus-star/lay-public/issues/46#issuecomment-6226098048
posted once and exact body read back. #47 remains NEEDS_REPORTER_RETEST; prior
#59/#60/#43/#44/#48 closures unchanged. Source/runtime work DONE for TD154/155;
TD121 known compatibility limits remain OPEN. Only publication metadata changes
follow the exact tested core; no repeated tests/builds/input/install/restarts.
BASE/PUBLICATION.json, METADATA-PUSH.json, ISSUE46-REPLY.json.

### TD-156 source closure checkpoint — 2026-10-18T02:28:18+00:00

Remote focused IME closure is PASS: **834/834**, receipt
`/home/ubu/.cache/lay/development/run-aozwx8jf/RESULT.json`, remote
`run-lfWZP1`. The actual first Atomic V1 Backspace handover regression and the
unchanged ordinary-stack isolated native-transfer test both pass. No protocol,
stack limit, safety gate or delivery authority was relaxed.

The preceding `run-rg14z0xt` / `run-ftkZO2` was 809 PASS / 25 FAIL. The failures
were grouped presentation contracts and historical document-preedit fixtures.
Current legacy publication fixtures now assert auxiliary updates and exact
hide/Space/Tab effects; historical delayed-document controls replay an explicit
document-preedit publication through the existing output and witness entrypoints.
They no longer pretend that a new auxiliary hint is document composition.
Current deferred publication and stale-release tests check auxiliary effects as
well as preedit; ownership, refusal and learning assertions remain.

Canonical manifest refresh `tab-manifest-ds_n3fce` / `tab-manifest-meIIjJ`: PASS,
3245 discovered; the focused closure selects 834. Independent production review
remains the completed two passes (7/10 then 8/10); no third review was added.
Full source gate, changed-byte native acceptance, visible panel, latency and
Plasma remain pending. No local installation/restart and no runtime authority
change occurred. The historical 349 PASS / 50 FAIL / 33 BLOCKED native matrix
remains the latest complete receipt.

### TD-156 complete test execution, stale inventory binding — 2026-10-18T02:38:48.108823+00:00

Canonical full run `run-0st554ds` / remote `run-y5ihP7` returned **FAIL**,
not a release proof. Anchored per-target execution records show 33 selected
targets, 3219 tests and zero test failures. Final aggregation refused the stale
`known_failures.json.manifest_sha256` after the regression inventory refresh.
Rebind that hash only; keep zero allowed failures, the historical observation
and every runner check. No runtime source change or rerouted delivery is made.
Refresh the graph and rerun the canonical full gate before the private build.
Native changed bytes and KDE remain NOT_TESTED.

### TD-156 complete browser candidate — 2026-10-18T03:30:51Z

Both original changed-byte native matrices are complete: Firefox 176/20/20,
Chrome 178/25/13 (PASS/FAIL/BLOCKED), total 354/45/33 of 432. No missing cells;
both exact GUI scopes/seats closed, no held keys, VT1 restored. Identity delta
against the immutable local v1.0.89 receipts, successful timing coverage and
all residual first transitions are recorded in the owning stability document
under “TD-156 complete browser candidate and KDE preparation”. Supplementary
Chrome actual-panel/Tab observation is 8 PASS / 2 focus-caret FAIL, separate
from the original matrix. No passing source test or engine-only output is
claimed as client acceptance. Remote Plasma packages are installing under the
canonical resource guard; its native gate is pending. No local install/restart,
new production edit or authority change in this checkpoint.

### TD-156 source closure — 2026-10-18T02:49:43Z

`run-zri9cql9` / remote `run-y0fkQZ`: **PASS, 3219 selected**, zero allowed
failures. Only the stale inventory hash was rebound; no additional runtime edit.
Private source-bound build is now running. Exact archive/receipt and the pending
native/Plasma gates are recorded in the owning Firefox stability document under
“TD-156 full source closure”. Historical native matrix remains 349/50/33.

### TD-156 Plasma zero-key startup checkpoint — 2026-10-18T03:51:26Z

Minimal installed Plasma starts on real VKMS DRM/libinput. Old host IBus1.5.26
causes candidate zbus5.15 InvalidName panic before input; complete stack/signal
trace and exact owned-scope closure are retained. The owning stability document
records temporary setup authorization removal, package versions, and a separate
pinned IBus1.5.34-rc2 coverage plan. No production edit or native PASS is inferred.

### 2026-10-18T04:05:03Z — TD-156 sequential Plasma native proof

Matching host-built private IBus 1.5.34-rc2 build/relocations PASS; the mixed-version client diagnostic and concurrent-client 512-task-cap attempt remain unaccepted and preserved. One actual client group at a time in genuine Plasma/Wayland now runs under the same ceiling. GTK ordinary-input preflight precedes its cells. Native KDE/menu NOT_TESTED pending receipts; original full browser matrix remains 354P/45F/33B and source3219PASS. No candidate bytes, model, delivery route or local runtime changed. Full evidence/limits in the owning Firefox stability document.


### 2026-10-18T04:14:51Z — TD-156 KDE GTK and wire diagnosis

Ordinary GTK input PASS after correcting private IBUS_USE_PORTAL presence and
restoring original private Plasma setup. GTK32 raw8PASS/22FAIL/2BLOCKED;
6 Space median15.51ms/p9516.95ms. Confirmed product refusal is GNOME-only
exact-focus lease, before replay; no guard/route weakened. Remaining KDE
clients running. See owning firefox-input-stability entry at this timestamp
for raw receipts, rejected hypotheses and observation limits. Original full
browser354PASS/45FAIL/33BLOCKED unchanged, candidate bytes unchanged.

### 2026-10-18T04:32:35Z — TD-156 KDE partial native milestone

Complete genuine Plasma grids: GTK 8P/22F/2B of32, Qt12P/33F/3B of48,
Chrome21P/132F/7B of160. KWrite accessibility setup and Konsole focus setup
do not certify their functional cells. Production tray registration observed;
visible menu labels BLOCKED. Shared confirmed product refusal is exact focus
lease being GNOME-only, not a delivery-route failure. Retain 45 Chrome empty
prerequisite failures and preedit/caret gaps as raw evidence. Successful Space
medians15.29–15.90ms exclude missing operations. Firefox exit-capture diagnostic
running; no crash reason claimed. Completed scopes closed with receipts. Owning
stability document records exact paths, cleanup, limitations and next gate.
Original browser354P/45F/33B and source3219PASS unchanged; no local installation.


### 2026-10-18T04:40:53Z — TD-156 KDE baseline completed; browser repair remains open

Unchanged candidate: KDE-native grid69P312F19B32NT/432, Firefox completed
28P125F7B/160. No repeated unexpected exit in the final diagnostic; first exit
cause remains unknown. KWrite/Konsole functional coverage untested, actual tray
labels BLOCKED. Ninety browser cells failed empty-newline prerequisites before
the intended operation. Confirmed KDE Double Shift refusal is the GNOME-only
exact-focus lease; no guard bypass. Final RESULT/CLOSED and local FINAL-FETCH
are at `browser-final-fixes-20261018/kde-native/`. Empty owned scope, no held
keys, VT1 restored, finite seat stopped at04:37:20UTC. Owning architecture entry
at the same timestamp records scope and successful-only Space timing. Original
browser354P45F33B, source3219PASS unchanged; no local install, source-byte or
route/authority change. Continue the original browser first-transition work.


### 2026-10-18T05:07:00Z — editor follow-up and Search observation evidence

KDE KWrite/Konsole now have complete 16-cell native receipts: 3P11F2B and
3P10F3B. Their original setup failures remain immutable. Cumulative same-byte
KDE432 is 75PASS/333FAIL/24BLOCKED/0NOT_TESTED, not one run or KDE acceptance.
The editor first manual refusal is IME context_authority, earlier than the
GNOME-only exact-window lease observed in GTK. Clean Space timing n2 each:
KWrite median16.01/p9517.80ms; Konsole14.44/14.64ms. No correction/DS speed or
terminal visual-preedit acceptance. The finite scope/seat are closed, VT1 and
key cleanup verified; local runtime untouched.

Six native Chrome Search probes with a read-only document mirror support stale
AT-SPI text in two after-Backspace failures, including the no-Double-Shift
control. Two other mirror controls have an early-frame sampling race; this is
diagnostic evidence only, not a new PASS matrix or permission to alter delivery.
Exact receipts, hashes, setup failures and next gates are in the owning Firefox
stability entry at this timestamp. Local KDE evidence:
`browser-final-fixes-20261018/kde-editors/evidence/` and `COMBINED.json`;
Search: `browser-final-fixes-20261018/search-document/`. Original full browser
354P45F33B and source3219PASS unchanged. Main task remains IN PROGRESS.

### 2026-10-18T05:31:15Z — TD-156 focus and rich seed first-transition proof

Focus-return End diagnostic: six native FAIL remain at context_authority after
correct letter/Backspace targets. Rich seed helper: 11 guarded unit PASS; one
nine-cell supplemental native series each gives Firefox6P3F0B and Chrome9P0F0B.
Six previously blocked Firefox prefix/selection operations are now observed PASS
in this separate series; three moved-boundary failures reach the core Apply but
lose the ASCII-only physical pair scope, then correctly fail adapter verification.
Initial NBSP parsing and post-commit separator canonicalization must both be
proved before a production patch. No normalization/authority/delivery bypass.
Owning architecture entry contains exact receipts, hashes, closure, timings and
next options. Original full354P45F33B and core3219PASS retained; no local install.

### 2026-10-18T05:49:38Z — TD-156 bounded raw pair presentation

Two ASCII-only adapter parsers lose an already selected NBSP BoundaryShift.
The existing core preserves the raw separator; do not normalize or reseal that
initial decision. The existing pending client commit now validates the entire
observed snapshot with ASCII/NBSP allowed only at the two already owned Space
positions, retains every authority/deadline check and stores the actual raw
source for exact recorded inverse. Same delivery routes and decision owner.

Corrected-fixture controlled RED overlays only its test module on 1717 unchanged
preceding production archive members. First GREEN and focused 22 owner + five
sealed gate tests pass, including 14 contradictory-state rejections. Initial RED
was a fixture failure and is not counted as root evidence. Exact hashes, receipts
and scope are in the architecture owner's 05:49:01Z entry. Full new source and
changed-byte native proof remain pending; original354P45F33B retained, no install.

### 2026-10-18T06:02:41Z — NBSP full closure: protected source chain refusal

`run-mcm5gdsg/RESULT.json` FAIL, remote `run-PNDFPe`, 509.5s: one reported
TD113 source protection failure at the old gate.rs successor hash. Preserve that
valid negative result. Add the explicit October18 predecessor/successor binding
and exact pinned-chain assertions; no guard bypass, old receipt edit, production
change, failure waiver or ranking change. Focused27PASS remains unchanged; new
seven-contract/full source closure, build and native acceptance are pending.

2026-10-18T06:03:41Z: protected-chain follow-up7/7PASS, `run-6tdb482o/RESULT.json`,
12.7s. Same existing test identities, predecessor decisions immutable, actual
successor hash/mode still enforced. Full source/native acceptance pending.

### 2026-10-18T06:15:09Z — NBSP full correctness/package closure PASS

`run-kf2jyn8d/RESULT.json`, remote `run-dUQNEJ`, archive
`8688ea790ec8d4e3a545c133ed0210011cf792873ee03f084c1449a1b6343cf2`:
3222 selected tests PASS, 3186 correctness + 36 package; zero reported failures.
Worker493.16s, overall501.6s. Formatting and runner self-test also pass. This is
the first full PASS for the additional bounded NBSP source; old failed receipt
and protected artifact chain remain intact. Build and native proof are next;
preceding native354P45F33B is unchanged. No local installation or authority bypass.

### 2026-10-18T06:18:32Z — NBSP build preflight rejects test-decision schema

`build-nbsp/BUILD-RUN.log`: actual changed-file canon rejects the test-only rich
seed ADR's missing required fields before release compilation. Retain the failed
stage; repair only its unpinned metadata. Preserve the full3222PASS receipt and
require exact executable/test/package parity before a separate build stage.
No Rust change or repeated full suite; native behavior remains untested.


### 2026-10-18T06:23:30Z — NBSP private release built, native phase started

The second, distinct private build passed the strict actual changed/added-file
architecture inventory, 15 canon tests, the 50-pass audit, architecture/lint
checks and guarded offline release compilation. The preceding build's ADR
schema refusal remains preserved; its only repair was test-only ADR metadata.
No Rust bytes changed after the 3222-test source closure. The new frozen source
archive is `f4e9ac774848485d3856443cfb49656cecdda870a0fd86b72dd63d0c72e10648`.
Exact build receipts: local `browser-final-fixes-20261018/build-nbsp-b/BUILD.json`,
`RESULT.json`, `CHECK.log` and `FINAL-AUDIT.json`; remote
`/home/e/projects/lay-development-runner/td156-nbsp-build-20261018-b/`.
Build status is BUILT_VERIFIED_SOURCE_NATIVE_PENDING, version 1.0.90,
227.359 seconds including preflight; source HEAD c8b244ef609ad9c5610b3e441311342abf17e9c5
plus the frozen dirty snapshot. Exact executable SHA-256:

- lay: `f8aec338504dadd72bd06fa78f4b3e8bd1863376897fce95dd3081e2d2e0c3c9`.
- daemon: `e6792f7cb165735e5052e48865490c96110973619c5bc5656d32e7c99de68cd7`.
- IME: `959d828c2d0557281ce8a30ee31689501538bb2e89f9506f5ecfa05b9ea05d35`.

Before staging, all old pinned native processes were dead, the owned cgroup
was empty and VT1 was active. The private harness preserved its old binaries,
extension, manifest, jobs, results and learning/usage data under
`TD156-nbsp-candidate-20261018`, restored the frozen isolated test data and
original double-lshift/undo-disabled configuration, then bound the three new
executables and matching extension. Local staging receipt:
`browser-final-fixes-20261018/nbsp-native/STAGED.json`.
Only this stopped private harness changed; no local installation or service
restart. Delivery routes, runtime authority and model bytes are unchanged.

The finite nine-cell Firefox rich-editor native phase has now started. Its
result, new-byte full browser matrices and speed measurements are PENDING.
The complete preceding-byte matrix remains Firefox 176P20F20B / Chrome
178P25F13B, total354P45F33B of432. KDE remains unaccepted with its separately
recorded cumulative75P333F24B of432; do not combine these denominators.


### 2026-10-18T06:25:54Z — NBSP changed-byte rich-editor native 18/18 PASS

The two finite supplemental native phases completed against the new private
IME `959d828c2d0557281ce8a30ee31689501538bb2e89f9506f5ecfa05b9ea05d35`:
fields7/8/10, prefix + moved_boundary + selection, nine cells per browser.
Firefox9PASS/0FAIL/0BLOCKED; Chrome9/0/0. Actor errors are empty. The unchanged
key/target functions were pinned by SCOPE.json; only the previously documented
bounded raw-rich seed observer admits the actual NBSP seed. No DOM setter,
normalization of runtime text, changed expected word or output-route bypass.
All three Firefox physical pair seeds are raw `должн\u00a0ыбыть`; final exact
observed targets are `должны быть\u00a0`. The preceding-byte supplemental result
was Firefox6P3F and Chrome9P, so the three reached Firefox pair failures now
pass while all previously passing supplemental cells remain PASS.

Exact primary receipts, also fetched and individually hash-verified:

- Firefox `firefox-functional-mwmo_m9n/RESULT.json`, SHA
  `c96063ee95d9f0122f9be359f33cbdee78d8573e72a101bc21a3147416cd6aa5`.
  Local archive `nbsp-native/evidence/firefox/evidence.tar.gz`, SHA
  `775866b66279e8c6d2c75185b8aa1a877e35e38f2e8062124544608705391e85`.
- Chrome `chrome-functional-j3c2_zmp/RESULT.json`, SHA
  `c8ba1779fe03d0fa595075a20e94206c565818cf300e8ae1a79e4c1b1b2fbcd7`.
  Local archive `nbsp-native/evidence/chrome/evidence.tar.gz`, SHA
  `ee6a42d197119a62fe8dbe3bf20880ca87f6495eaed17dae9b3fa80dc54e447e`.

Each archive has19 verified files including full actor/IME trace, candidate
binding, scope and CLOSED receipt. Both phases returned parent exit0 with no
held/released keys; all pinned processes dead, owned cgroup empty, seat stopped,
VT1 restored. Closed at06:24:01.100308Z and06:24:48.011451Z respectively.

Action-only physical-write-return to exact AT-SPI target latency (ms):
Firefox DoubleShift n12 median41.61/p9580.43, Space n3 median18.95/p9519.93,
selection n3 median18.87/p9521.06. Chrome DoubleShift n12 median50.22/p9575.50,
Space n3 median18.84/p9520.33, selection n3 median17.41/p9520.01. All attempted
actions measured; n3 p95 is the sample maximum. This supplemental series is
not a matched full-matrix speed claim or evidence for the unreached old failures.

Full original-protocol216/browser remains pending. Its first controller setup
was rejected before any session/input because `tdnbsp-full-firefox` violates
the existing two-word seat-label allowlist. Preserved original phase
`TD156-nbsp-native-20261018/full-firefox/FAILURE.json` records input_keys0,
empty scope, dead old pinned processes and VT1. No runtime defect inferred.
A distinct `run-full-b.py` changes only phase directory (`full-b-`) and valid
seat label (`tdnbspfull-`), not actors/key actions/targets. Controller SHA
`278747675701756462e475decf052d08003a8f25383aa11d26e1a613af43843b`;
local FULL-CONTROLLER-CORRECTION.json records the old/new binding. Its fresh
Firefox full phase has started once; no product retry-until-green.

Verdict: bounded NBSP pair mechanism has source3222PASS plus18/18 supplemental
native acceptance. No full-matrix, focus, undo-precondition, Search-observer or
KDE acceptance is inferred. Complete preceding matrix354P45F33B remains the
status denominator until the new full closure. Runtime authority unchanged;
no local installation, model change or delivery-route change.


### 2026-10-18T06:32:35Z — controller isolation and Search causality plan

Full-controller two pre-matrix setup refusals are preserved; distinct proof tags
and a static chain audit precede the running full-c Firefox phase (40 baseline
cells PASS so far). The owning Firefox stability entry at this timestamp records
exact hashes, closure and the selected9/10 read-only Search event-barrier
diagnostic, rejected alternatives, negative gates and limits. No Rust change.

### 2026-10-18T06:55:43Z — complete NBSP-byte browser closure and diagnostic boundary

The owner's06:39:28 source-audit entry,06:44:49 Firefox entry and06:54:10 Chrome
entry retain the current3222-test source proof,18/18 supplemental rich-editor
proof and complete original browser354PASS/46FAIL/32BLOCKED of432 (zero untested).
Chrome is unchanged at178/25/13. Firefox176/21/19 includes a newly observed
email short-word FAIL and a separately reached visibility precondition; neither
is hidden by supplemental success. Six Search causal controls are BLOCKED at
the trusted-keyup evidence barrier, not a native acceptance proof. Exact hashes,
closures, fixed latency coverage and first-failure limits are owned by the
Firefox stability document at these timestamps. The two-cell original Firefox
email diagnostic is finite and separate. No Rust, model, route or local-runtime
change follows; whole candidate acceptance remains FAIL and TD156 IN PROGRESS.

### 2026-10-18T07:19:40Z — Search representation separated; first-refusal capture bounded

The owner's07:01:49 entry closes a causally fresh six-cell Search supplement:
native4PASS/2FAIL, raw document value/caret6PASS. Those two native failures show
stale AT-SPI after the actual field completed deletion. They do not justify a
production edit or relabel the original matrix. The owner's07:16:49 entry retains
the two successful short controls (two-cell and30-cell24P4F2B), the zero-input
nested-lease setup refusal and exact closure, hashes and constraints. The original
short FAIL remains unexplained. One full-prefix original Firefox216 observation
is now running with nine byte-identical native jobs and a passive capture hook in
the existing bounded controller wait; four observer source controls PASS, zero
input scope audit PASS. No extra worker, changed key/target/deadline or production
Rust change. Source3222PASS and original complete354P46F32B/432 remain the status
boundaries pending native closure. TD156 IN PROGRESS; local runtime untouched.

### 2026-10-18T07:34:56Z — finite full-prefix observation closed

The unchanged Firefox216 replay closes176P21F19B; aggregate354P46F32B/432
with Chrome remains unchanged. The numeric field4/short now passes while
field1/prefix fails reverse Double Shift after a successful first transformation
and letter/Backspace. No short capture was triggered; capped final traces do
not preserve the prefix failure. This is neither a fix nor new causal evidence.
Archive `ddc6fd32a76fa7f6e12b0beeb55dd33d5a9f2207cbe5785ffbdbb1452af9ef2b`,
38verified files in `full-prefix-capture/evidence/firefox`; CLOSED07:31:45Z,
scope empty, pins gone, VT1 restored. No more identical repeat without a causal
change. The independent first-pass NBSP static review scores9/10, no blocker;
raw physical inverse remains an explicit native gap. No runtime code change.

### 2026-10-18T07:54:28Z — distinguish native deletion from local preedit cancellation

The retained daemon excerpt now confirms both gestures in the intermittent
Firefox prefix cell: handled, then blocked by the focused IME after letter/BS.
Its IME reducer trace remains incomplete. Two invalid closed-preedit fixture
extensions are retained as exploratory failures. A dedicated active-native source
regression passes8 receipt-order controls:4 exact handoffs with an actual final
erasure receipt,4 refusals without it. It preserves CommitText plus client
Backspace and cannot authorize an unobserved return. No production edit follows.
The source receipt is `active-probe/green-6zamm44h/RESULT.json` under the TD156
cache; final formatting/full closure pending. Current visible-Tab supplements
close20BLOCKED before any Tab, due the bounded GNOME tree/application evidence
provider. Both exact owned seats close cleanly. Six separate raw-pair native
inverse cells are prepared for review. Original354P46F32B/432 and preceding-byte
KDE75P333F24B remain distinct; TD156 IN PROGRESS, local runtime untouched.


### 2026-10-18T08:02:32Z — raw inverse evidence is not yet qualified acceptance

The finite six-cell raw inverse supplement closes0P6F0B with exact cleanup.
Both first corrections pass; Chrome's immediate inverse passes, but its letter
canonicalizes NBSP. Firefox restores the word surfaces with a different internal
Space presentation. The pre-Space seed cannot establish the actual post-Space
source of the sealed inverse. The first inverse records negative learning in
both isolated phases; later fields stop before correction. Preserve all raw
failures and do not infer six new product bugs or change runtime/learning to
satisfy this oracle. The owning architecture entry at this timestamp retains
exact hashes, timings, source review10/10 and evidence limits. Source-only active
word regression formatting is complete; final source closure remains pending.
Original354P46F32B/432 and preceding KDE75P333F24B are unchanged; TD156 IN PROGRESS.

### TD-156 — 2026-10-18T08:24:09Z: Qt connection diagnostic and metadata closure

One current-byte KWrite physical `п` reaches stock Qt IBus and Lay: two accepted
press/release serials33/36; actual empty field -> `п`, caret1. Old KDE zero-key
trace is not a universal Qt-disconnection proof. No causal runtime repair or
functional acceptance follows. Actor succeeds; overall run ABORTS on launcher
descendant-cleanup refusal. Independent CLOSED verifies empty scope, pinned
processes gone, no held keys, VT1 and stopped owned seat. Receipt39files:
`browser-final-fixes-20261018/kde-link/evidence/`, archive
`e3813d3c09034910bb429de2396d6b7024f06d37bd65d172e7151f5181540c5f`.
Two static review passes6/10,8/10; final interruption finding fixed, no third
score claimed. Production Rust, runtime authority and delivery unchanged.

Full source run-r4_246uk executes3223 selected tests with zero actual target
failures, then FAILS stale known-failure/manifest binding. Corrected manifest
hash only; zero failures permitted, no guard weakened. Clean closure pending.
Original browsers354P46F32B/432; old KDE75P333F24B/432 remains separate.
See the architecture owner entry 2026-10-18T08:24:09Z for exact proof boundaries.

### TD-156 — 2026-10-18T08:36:04Z: clean source closure; bounded prefix diagnosis

Full development gate now PASS3223, zero actual test failures after the
known-failure manifest hash correction. Receipt run-90h0cm7a/RESULT.json; archive
ea727cffbf60a42c06599f5d9964da05d6870a2d1d6bf99a7057a304e872f315.
No runtime Rust change or installation. Next first-transition evidence is one
unchanged original Firefox field1 prefix round trip; the prior trace capped
before inverse refusal. New diagnostic is prepared, not yet executed; first
static review6/10, fixes pending final review. Never bypass missing erasure
authority. Source PASS does not reclassify original354P46F32B/432 or KDE
75P333F24B/432. Owning architecture entry at this timestamp retains the limits.

### TD-156 — 2026-10-18T08:53:23Z: prefix controls; complete-line log repair

One unchanged fresh original prefix control passes2/2 Double Shifts through
active composition; it does not reproduce the original refusal. Archive
1078c251aea93efcdfc74120b0f1fba01bdf91d88d8c9d82070d4e162303b3ab.
A distinct unchanged first-five-plan predecessor comparison records five raw
actor PASS but orchestration ABORTS on its fragmented first capped trace line.
Archive f72dc1943fcd25fd4d9e0ec34aa49379f8fd0c1c70f0bc7add3e095635c12be7;
41 raw checkpoints retained, independent CLOSED/VT1 verified, no rerun to seek PASS.
Both are supplements; asynchronous log delivery completeness remains UNKNOWN.

First diagnostic defect reproduced in source: the debug-log byte cap selected
an earlier newline then hard-sliced its first JSON record. Fixed by keeping the
newest whole records within the same500KiB cap; oversized record dropped whole,
exact fits and UTF-8 retained. Options5/10 collector tolerance,3/10 larger cap,
9/10 whole-line compaction selected. Remote RED0PASS/3FAIL, GREEN3/3; isolated
read-only review10/10, one pass. Receipts under browser-final-fixes-20261018/
log-compaction/{red-4826a6ya,green-mau5ulk_,format-lulrdapm}/. This repair does not
identify or fix the intermittent Firefox refusal. The previous full3223-test
receipt certifies preceding bytes only. No new candidate or installed runtime.

KDE production code has layout switching but rejects exact focused-window
identity outside GNOME. A production KDE provider needs its own contract stage;
never replace it with the observer or remove the identity guard. This source gap
does not classify all333 KDE failures. Original354P46F32B/432 and separate KDE
75P333F24B/432 unchanged. Detailed boundaries in the architecture owner08:53:23Z.

### TD-156 — 2026-10-18T09:01:51Z: inventory and KDE capability task

Remote manifest refresh tab-manifest-uctnv_cj/RESULT.json PASS:3188 correctness
+36 package =3224 selected,15 ignored,11 performance. Known-failure binding is
updated with zero failures permitted. This is inventory, not a new full-suite
PASS; retain preceding full3223 and later focused debug-log3/3 separately.
Final graph/check-architecture evidence is read through the TD156 cache
GRAPH-CLOSURE.json pointer; this planned step does not predict its verdict.

KDE exact-replay layout handoff independently requires GNOME. A foreground
provider alone cannot close it. Planned TD157 now includes stable backend-scoped
identity admission and bounded recovery from the current unavailable latch;
exact-replay handoff remains a separate capability. Isolated description review
8/10 ->10/10, two passes; no implementation/native acceptance claimed.
Latest original46 FAIL group20 Space preconditions +20 focus-return +1 prefix
+5 Chrome Search/Backspace observations; the last five are not established
product defects while raw client text contradicts stale accessibility. Original
354P46F32B/432 and separate KDE75P333F24B/432 retained. No runtime authority change.

### 2026-10-18T09:24:15Z — TD156 observer-only published Tab supplement

The first shared failure is the old accessibility traversal budget, not Tab
delivery. Bounded visible bus/path traversal, no partial evidence and post-Tab
BLOCKED classification pass source21/21; isolated review7->9 in two passes.
Three later namespace/immutable-receipt setup refusals and exact independent
closure are preserved; no diagnostic cell ran in them and the final metadata
corrections are later than the review snapshot. Native supplement completes
Firefox10P and Chrome7P3F, all20 Tab targets reached, three further probes FAIL.
The original Tab plan also requires Double Shift and is not replaced. Exact
receipts and qualified/action timing are in the architecture owner at 2026-10-18T09:24:15Z.
Permanent byte-identical Python helper+seed controls21/21PASS remotely; this is
not new runtime or Rust full-suite acceptance. Original354P46F32B/432 and older
KDE75P333F24B/432 remain separate. No core authority, route, model or local
runtime changed; TD156 remains open, KDE capability is planned TD157.

### 2026-10-18T09:48:38Z — TD156 full source closure and lexical refusal boundary

Current remote full development check PASS3224, zero failures, receipt
`run-1ygbhlgc/RESULT.json`, source archive6e8c3e89196e. It covers the later
debug-log regression; it is neither release nor physical acceptance.

Space diagnosis review7/10 ->9/10, two passes, correctly requires consumer evidence. The
ready-service read-only trace now retains72 suggestions and target `привет`;
separate direct L1.1 restore ABSTAIN/outside_calibrated_basin (not a measurement
of live composite original_l11_verdict), L3 not_applicable/no pairwise
certificate, no exact positive state-specific L4 witness, Normal safety and no
CLI lexical frame. Both resolutions keep the original. This does not establish
the native prefetch's final outcome or an undo defect. No promotion, deadline
change, model fit or delivery/authority edit is justified. Preceding mount,
absent-service and warming diagnostics remain separate. Exact hashes, unchanged
inputs, owned-service closure and limitations: architecture owner09:48:38Z,
TD156 cache `core-authority-d/`. Original354P46F32B/432 and older
KDE75P333F24B/432 remain unchanged; source PASS does not erase native failures.

### 2026-10-18T10:05:25Z — TD156 native Space lease READY, producer remains SuggestOnly

One fresh remote native Firefox normal field1 undo-plan prerequisite reproduces
`превет ` instead of `привет `; undo is NOT REACHED. Matching generation11,
tail_epoch16 and engine path show72 SuggestOnly candidates, no selected
transition, terminal no_apply/rank and a READY lease after3277us within4ms.
This resolves the old not_ready ambiguity for this cell: waiting longer cannot
repair a result which already has no eligible correction. Internal3392us Space
fallback time is not correction acceptance latency. Native candidate identities,
actual composite L1.1 verdict and the other19 prerequisites remain unmeasured.
Do not infer them from the separate CLI target or matching candidate counts.

Exact pins,19 hash-verified fetched files,435 trace records and owned closure
09:57:24.130792Z are recorded in the architecture owner10:05:25Z and TD156 cache
`space-native-authority/`. Runtime implementation final independent review9/10,
one pass, no new blocker; it does not grant native acceptance. Full source3224
and original354P46F32B/432 remain distinct from older KDE75P333F24B/432. No
model, gate, ownership, route, production install or release changed.

### 2026-10-18T10:13:25Z — TD156 full published-Tab native supplement

The new supplement reuses the original two-direction roundtrip after accepting
the actual complete visible auxiliary hint. Native Firefox10P/0F/0B and
Chrome9P/1F/0B of ten fields each; all20 Tab targets,39/40 Double Shift targets
and38/40 decoder/Backspace probes complete. Chrome Search fails first Backspace
return and never attempts its second direction. It is not a passed row or proven
IME defect; prior raw-client/AT-SPI contradiction remains relevant. Both exact
seat sessions close and are independently verified with zero keys at10:11:14Z.

Reached-action Double Shift median/p95: Firefox117.829/134.797ms n20/20;
Chrome105.440/131.041ms n19/20. Tab has n10/10 per browser. Clock, maximums,
hashes,20-file archives per browser and limitations are in architecture
owner10:13:25Z and `published-tab-roundtrip/SUMMARY.json`. Capped old native debug
logs are causally unqualified; exact client receipts are separate. Diagnostic
review9/10, one pass; terminal-receipt filtering corrected before native input,
inherited finite-seat setup-cleanup gap preserved. Original fixed-literal cells,
354P46F32B/432, older KDE75P333F24B/432 and source3224PASS stay separate. No
runtime/authority/delivery/model change, release or local installation.


### 2026-10-18T10:24:24Z — TD156 intended prefix window, diagnosis only

Prior collector selects field4/short, so it cannot explain earlier field1/prefix.
Prepared one remote96-cell original predecessor sequence; observer now captures
only fresh [1,2]/row4 prefix FAIL. Runtime/actors/cadence unchanged. Four observer
controls and context-free review before input; verified and independent closure
after. No reproduction is not repair. Owner: firefox-input-stability-2026-10-05.md
entry at2026-10-18T10:24:24Z; evidence TD156 cache prefix-window-capture/.

### 2026-10-18T10:40:17Z — TD156 prefix history finished, no reproduced inverse refusal

The single96-cell original predecessor sequence closes90P/4F/2B, no NOT_TESTED;
120original cells were not selected. Normal/private/fullscreen and panel66/66P;
functional fields1/2 close24P/4F/2B. The intended field1/prefix passes both DS
(18.542802/18.447612ms) and both decoder/Backspace returns. This is NO REPRO,
not repair. The old intermittent FAIL and full original354P46F32B/432 remain.
No further native repeat or runtime edit. Collector controls5/5P, two read-only
review passes7->9/10; selected/saved RESULT bytes are one snapshot.

Thirty-one fetched files are hash-verified; archive69e42926463113da. Finally
closure10:38:49.017535Z and independent exact-seat closure10:39:09.762746Z are
clean. End trace512000bytes/2288JSON plus one retained fragmented line; no
native failure capture, asynchronous completeness UNKNOWN. Four FAIL are the
two Space prerequisites and two focus-return cases; two BLOCKED are old fixed
literal Tab prerequisites. Full timings, identities, hashes and next ownership
gates are in architecture owner10:40:17Z and cache prefix-window-capture/SUMMARY.json.
KDE75P333F24B/432 and source3224PASS stay separate; no route/model/authority,
local install, release or publication change. TD156 remains open.

### 2026-10-18T10:57:35Z — KDE plan inventory; no runtime experiment

Six SHA-verified native receipt inputs and432unique cell identities retain
KDE75P333F24B/432. A read-only inventory separates333FAIL into90rich-clear,
6seed/return prerequisites,186post-actor-Double-Shift target failures and51Space
target failures. These are first failed actor steps, not verified runtime causal
classes; target checks combine text/caret/selection/layout. No verdict changes.

Architecture owner10:57:35Z contains the per-app breakdown and exact receipt
scope. Cache`kde-plan-triage/TRIAGE.json`, SHA-256ee84c5365b20bf25, indexes every
failed primary row. TD157 is the KDE plan entrypoint, referencing TD159 bounded
prerequisites/causal evidence, TD157 foreground identity/lifecycle, TD158 exact
layout activation/readback/restoration, then TD156 residuals. GNOME-only restore
is independently required even after a provider exists. No second reducer,
delivery route, ranker fit or weakened safety/verifier is proposed.

No runtime source/binary/data, native input or authority change; no new source
tests, installation, publication or universal latency claim. Original GNOME
354P46F32B/432 and prior3224source PASS stay separate from KDE's older bytes.
Descriptions received two context-free review passes,9/10 then10/10. The single
P2 about an unnecessary global evidence-task dependency is resolved with
boundary-specific gates; no remaining substantial P1/P2 or blockers. Review
record: cache`kde-plan-triage/REVIEW.json`. Runtime and native acceptance remain
unproved. TD156 IN PROGRESS, KDE capabilities PLANNED.

### 2026-10-18T12:47:41Z — TD159 stage-observer controls; native attempt pending

Five-case KDE witness prepared in cache`kde-witness/`, reusing the existing shared
actor/seat. Remote guarded six helper/adapter controls PASS; twelve script syntax
and SHA checks PASS. Key bodies, deadlines, semantic targets and verdicts stay
fixed; stage marks are memory-buffered, raw traces retain fragment/completeness
limits, and exact selected cell identities gate completion. Second harness
review and one finite native attempt pending. No runtime edit, data fit, install,
route/authority change or new native acceptance. KDE75P333F24B and original GNOME
354P46F32B remain historical for their own bytes. Owner entry has full scope.

### 2026-10-18T12:53:10Z — TD159 foreground failure qualified; provider starts

Single five-case budget ABORTED after three GTK FAIL observations. Native Entry
seed is committed; physical Double Shift reaches exact-tail delegation then
fails the GNOME-only focused-window read. TD157 may proceed independently of
remaining seed/browser prerequisites. TextView prefix fails before gesture;
Entry Space restores text but retains RU mode. Firefox cleanup cannot create a
thread; Chrome not started, resource cause unproved. Independent closure CLOSED
with empty cgroup, VT1 and policy absent. No success latency sample. Full hashes,
scope and owning evidence: architecture entry2026-10-18T12:53:10Z and cache
`kde-witness/CAUSAL-WITNESS.json`. No runtime/source authority change or new grid;
KDE75P333F24B and original GNOME354P46F32B unchanged.

TD157 checkpoint2026-10-18T13:18:18Z: fresh authenticated KWin provider implemented
under its preflight/decision. Daemon initial307 selected304P3F; repaired private
bus fixture yields14/14 provider PASS in0.194s execution. One controlled removal
of sender authentication produces the expected real-bus refusal-test RED and is
restored by hash. Inventory refreshed; zero known failures remains strict, no new
candidate PASS inferred from historical observation. Owning architecture entry
above records exact receipts and pending full source/review/compositor gates.
No native acceptance, delivery-route, model, installation or release change.
### 2026-10-18T13:36:58Z — TD157 full affected source gate PASS

Full development receipt `~/.cache/lay/development/run-sy49nis0/RESULT.json`:
3238 selected, zero failures; format/self-test/all PASS. Remote commands504.800s,
transport514.426s; source-only durations. Archive3010f5f94496483bb0b06860684183cf80a6da43ddc166289ba1c305c30368f8.
Graph gate `tab-graph-k0gaykxb/RESULT.json` PASS after retaining private-file guard
ownership. The architecture owner records scope and native gaps. Next: independent
TD157 review, frozen build, one changed-candidate GTK witness. Historical grids
unchanged: KDE75/333/24; GNOME354/46/32. No installation or release.


### 2026-10-18T14:01:00Z — TD157 shared-provider contention repair

Full affected source PASS3239/3239, no failures; `run-g0ss9and/RESULT.json`,
archive433caec7f63aaf4f325939144bffe6024d16a3706d373fc30d1aa10b036db47d.
Focused15PASS and controlled Busy-as-loss RED qualify the new regression.
Details and limits: owning `firefox-input-stability-2026-10-05.md` TD157 entry.
Final review, private build and changed-candidate native witness pending.
KDE75/333/24 and GNOME354/46/32 are unchanged; no runtime promotion.

### TD157/158 — 2026-10-18T14:59:37Z: exact KDE handoff source closure

TD158 follows the qualified native GNOME-only layout-handoff refusal, not a
ranking failure. Existing exact replay now has captured-backend KDE activation,
fresh group/engine/actual-decoder readback and bounded same-focus restoration.
Full source gate PASS3248 (`run-3b5ymuu_/RESULT.json`, archive
`70db3455a142e5b189a4904a19f72cfd62da982ae90d180f22775158ad1b57eb`), final code review10/10, causal
missing-decoder-readback RED with source restored. The earlier full FAIL was a
stale zero-known-failure inventory binding; no semantic failure was waived.
No ranking/training/model change, text delivery route migration, installation
or publication. Native witness and complete matrices remain pending. See the
architecture owner entry `2026-10-18T14:59:37Z` for proof boundaries.

### TD158 — 2026-10-18T15:13:57Z: declaration repair source closure

After the private build's strict visibility-lint refusal, four declaration
visibilities were repaired without changing runtime statements. Strict lint
and generated graph PASS; fresh complete source gate PASS3248/3248,
`run-0z2k24yk/RESULT.json`, archive
`4675521392446586c8c1c5c41fd1b4c895515417fe4a387471b5e887bfafb0a6`.
No semantic/infrastructure failure was accepted. Private build B and native
witness remain pending; no ranker, model, training, text route or runtime
promotion changed. Historical complete native matrices remain unchanged.

### TD159 — 2026-10-18T20:41:33Z: distinguish harness failures before further input

Zero-input knid failed on an unsupported IBus PID method after schema setup;
knie replaced it with actual GIO transport credentials and supported name calls.
Source20/20 PASS; actual panel/registration passed, but setup engine activation
was cancelled before input. Hypothesis: competing initial engine selectors.
Do not rerun unchanged. Raw Firefox source-two76/76 controls passed across both
browser packages; actual3x7 observations remain PARTIAL/UNQUALIFIED because
clear_cache_single is absent. Text/tree topology survives; no LF trimming or
semantic verdict change is justified. Chrome raw native remains NOT_TESTED.

GNOME partial observation proves IC reuse across DOM fields, not persistent
field binding; four rich-return rows expose a caret0 append-premise failure.
No authority expansion is justified. Exact receipts, hashes and independent
closure facts are in the architecture owner entry at20:41:33Z. Complete grids
remain KDE175/233/24 and GNOME354/46/32 of432, NOT_TESTED0. The browser protocol
now records zero-key API/setup preflight and first-shared-failure triage before
full replay. Runtime, model, user install and publication unchanged this step.

### TD159 — 2026-10-18T20:54:02Z: actual-API-first isolation

Raw rich observer API preflight and76 source controls PASS. Firefox/Chrome each
capture21/21 complete stable trees without issues; CAPTURED/UNQUALIFIED only,
not behavioral PASS. Both finite sessions CLOSED/empty/VT1/no held keys/policy.
Receipts: kde-rich-raw-capture-api-next/{firefox,chrome}/EVIDENCE.json. Next is
a narrowly scoped physical-clear presentation predicate, never general LF or
U+FFFC normalization. Separately knif controlled RED1/1 expected failure PASS,
but normal source controls abort after26 tests on GI reference/type assertions;
37-test suite incomplete and native NOT_TESTED. Preserve SOURCE-ABORT.json;
distinguish fixture ownership from decode lifetime with a finite zero-key API
probe before proceeding. No runtime/model/training/route/install/publication
change. Original complete GNOME354/46/32 and KDE175/233/24 of432 retained.

### 2026-10-18T20:58:52Z — TD159 GI ownership control before native replay

Hypothesis: knif's source abort comes from floating synthetic-fixture ownership.
Actual zero-key host/private IBus controls: inspect2 PASS; naive sink-pair2 FAIL
at final owner collection, after successful decoding and container release.
G_DEBUG=fatal-warnings converted the double-unref warning to exit -5. Reject
sink-pair; next is a balanced-ownership API control, then the source suite.
Receipt: kde-native-ibus-integration-ownership-probe-next/RESULT.json and4
per-run JSONs; no bus/session/keys, no client or latency verdict. Original
GNOME354/46/32 and KDE175/233/24 of432 retained. Runtime authority unchanged.

### 2026-10-18T21:02:54Z — TD159 finite ownership control passes

Different parse-owned fixture control:2 actual host/private processes,4 object
cases PASS, empty stderr, normal exit and full GC. Stock GLib creates an equal
independent nonfloating value before embedding; no manual reference operations
or decoder change. Receipt: kde-native-ibus-integration-ownership-parse-next/
RESULT.json plus2 per-provider receipts. Source fixture only is eligible to
change;37 source controls and native remain pending. No bus/session/keys or IME
latency claim. Original complete matrices and runtime authority unchanged.

### 2026-10-18T21:09:09Z — TD159 two bounded source gates pass

knig:37 source controls PASS under fatal GI warnings,22 pins,normal exit,
0 failures/errors,0.258s. Four unchanged Chrome identities are admitted only
to a separate finite native diagnostic, still pending. Receipt:
kde-native-ibus-integration-engine-fixture-next/chrome/SOURCE-CONTROLS.json.
Rich-clear:12/12 pure controls PASS,0 skips;24 excerpts independently matched
the actual pinned raw trees. Receipt: kde-rich-clear-qualified-next/
REMOTE-SOURCE-ONE.json. Prepare the native clear-only bundle separately, with
original J setup and unchanged measured cases. Neither gate regrades original
GNOME354/46/32 or KDE175/233/24, changes runtime authority or measures IME speed.

### 2026-10-18T21:10:23Z — TD159 knig native separates setup from owner/replay failures

Matching private IBus/schema/stock panel setup succeeds; original4-cell native
diagnostic completes in21.118s,4 FAIL. Three cells reach Double Shift: two
roundtrips are physically detected but blocked by focused IME owner; one Space
cell rejects exact committed-tail capture. The fourth fails rich clear on
LF/caret0 before any gesture. Reject the setup-change-alone remedy; analyze
the captured first transitions rather than rerun unchanged. Receipt:
kde-native-ibus-integration-engine-fixture-next/chrome/RUN.json and92 pinned
artifacts in EVIDENCE.json/archive SHA
fd39dccae919f294b9e235ee30ca9298dd96b0a0e92897b048374d6bbb833648.
Independent CLOSED21:10:23Z: empty cgroup, pinned processes gone, no held keys,
VT1, own seat stopped, temporary policy absent. No successful latency sample,
runtime/source authority or route change. Original full matrices retained.

### 2026-10-18T21:19:36Z — TD159 rich-clear package fails source contract before native

Controlled RED: six expected structural-LF failures/browser. Actual-host zero-key
API: PASS. Firefox source22:21 PASS,1 FAIL; version-dependent ast.dump defaults
produce different hashes for identical abort() AST. Recreating the older format
matches the exact remote hash. The initial unit-literal hypothesis was wrong;
abort() has no unit literal. Retain source-one/RESULT.json in
kde-rich-qualified-native-next. Chrome
normal source controls and both native phases NOT RUN; no keys or clients.
Correct only portable empty/default AST serialization, with mutation negatives.
Original matrices, delivery routes and runtime authority remain unchanged.

### 2026-10-18T21:21:52Z — TD159 safe capture rejection diagnostic candidate

Retained trace separates two context_authority roundtrip rejections from one
exact-tail capture rejection whose original reason was redacted. The later
actor readout cannot identify the capture-time cause. Applied the minimal
rejection-only source candidate from kde-ime-tail-capture-diagnostic-next:
static reason/source kind/permission/presence flags from the existing reply;
no text, paths, extra RPC or wait. Shared validators, guards and routes unchanged.
Source/native checks and a new private binary are NOT RUN. Existing complete
native counts and installed runtime retained; see owning architecture entry.

### 2026-10-18T21:27:27Z — TD159 qualified-clear finite native subset

Source-two portable AST contract and actual-host API preflight:22 PASS/browser,
zero skips. Original fields7/8/10, roundtrip then short: Firefox5 PASS/1 FAIL,
Chrome0 PASS/6 FAIL. Clears qualify5/6 and6/6 respectively. Firefox first field10
has empty text/caret0 but missing required attributes; guard rejects before
gesture. Chrome reaches first Double Shift and leaves text unchanged. Keep
setup failures distinct from product behavior.60 artifacts/browser SHA-verified
in kde-rich-qualified-native-source-two; independent closures21:24:47Z/21:27:27Z
prove empty cgroups, pinned processes gone, no held keys, VT1, owned seats
stopped and absent policies. No original full-matrix regrade or route change.

### 2026-10-18T21:29:31Z — TD159 formatted capture diagnostic, architecture PASS

Fresh static review9/10, no blockers. Formatted ime_bridge SHA
8eae0978dde2e3a4e1c5d2e4ea6b79812cdd409252549524c21b10ed6b2f75c7;
graph/source-binding/canonical checks PASS. Only this Rust file differs among
751 frozen-C Rust/Cargo/build files. Focused15 Rust tests/private daemon build
NOT RUN; old full source proof is not transferred to changed bytes. No
additional RPC/wait, text retention, owner or delivery route change.

### 2026-10-18T21:37:14Z — TD159 capture diagnostic source15 PASS/private build

Actual13 delegated-tail tests PASS; the first runner incorrectly expected15
from that one module and stopped before build. Preserve its failed receipt.
Continuation runs only the two omitted existing RPC tests; source inventory
union15 PASS/zero FAIL/ignored. Private daemon release build PASS, SHA-256
8a020332f85dc063ee5b7ab193ff0ef5b2d2d3f939bb125536cb6dff5a35eae7.
Verified receipt/logs: kde-ime-tail-capture-diagnostic-build-next/executed-b/;
B-RESULT SHA809525db09693d00c275789be023c56dab941d06999e0eb8900fe73c4e61bd98.
Shared target7135436800/12884901888 bytes. Native NOT RUN; no installation,
publication, delivery-route or authority change. Full native matrices retained.

### 2026-10-18T21:47:13Z — TD159 four original Chrome cells, capture reason known

Actual-host source51 PASS/zero errors/skips and one expected schema RED.
Diagnostic-only daemon B, unchanged C IME/CLI/models and knig IBus/panel setup.
Native four-cell experiment22.761s wall time,4 FAIL: two roundtrip authority
refusals; two post-Space capture rejections explicitly source_unknown,
permission denied, engine present, field receipt absent. Last plain-input trace
has managed owned7 then external SSI6, accepted same-owner Shift callbacks,
then delegation. A later successful raw tail read does not certify the refused
exact handoff. Do not weaken its validation or infer complete producer delivery.

Local evidence: kde-native-ibus-integration-daemon-diagnostic-next/chrome/;
native RESULT SHA8dbbd2bd45f65a79743fdfac170102c504f031464b93e40fdf942b0a272dc977.
Independent CLOSED21:47:13Z;94 artifacts SHA/size verified, archive SHA
36da7b2cf90b6be5c6018e19d493bb96c48a92b8282e9ce15817cd104ab80d7a.
An export-only directory selection error was corrected without repeating
native input. GNOME354/46/32 and KDE175/233/24 of432 remain unchanged;
no installation/publication, runtime authority or delivery route change.

### 2026-10-18T22:04:40Z — kwdh Chrome/KDE wire diagnostic

Corrected package:51 actual-host source PASS, zero-key GI PASS, one expected
controlled RED/zero errors, static review9/10. Earlier kwdg source50/1 is
retained and never ran native input. Same fields7/1 × roundtrip/space_burst5:
4 native FAIL in22.900s orchestration wall time, no successful latency samples.
RESULT SHAe426e98f7ab06329767023be67257036a585e16a28a25ccd2f851848c8a18519.
Independent CLOSED22:04:40Z;95 export hashes/sizes verify; archive SHA
2f98e42b3bd520b81bca92eabf0f52cc6d110c85195149859ca2077e66c4df6a.

Chrome object33 wire proves stale SSI at outgoing boundary: gestures retain
commit/done19/18,28/27,52/51,60/59. Final post-Space SSI6 versus actual7;
no done60 during gesture. Renderer-internal pending contents not proved.
Sibling WIRE-ANALYSIS-source-one.json SHA
5efbc35dd4fd3c4ccff36634824d845a330b814f4dba50c5cc26437ace6386b5.
Primary upstream KDE a74c436 provides pending/current preedit-preserving acks,
absent5.25.0/present5.25.1. Backport build/native comparison NOT RUN. Detailed
receipt/source pins and proof boundary are in the owning architecture entry
at the same timestamp. No core/model/Lay route/authority/install change.
Full GNOME354/46/32 and KDE175/233/24 of432 unchanged.

### 2026-10-18T22:24:28Z — upstream KDE protocol RED/GREEN established

Private upstream a74c436 backport: actual loaded stock library reproduces
testEvents missing acknowledgement (2 PASS/1 FAIL); candidate29 PASS/0 FAIL/
0 SKIP. All2379 exports retained, installed library SHA unchanged. Exact
receipt SHA b37d44e9c353eb0e51ad834dca475bb1f0f53851c2d297a30d4089020e3c0a2b;
local kde-upstream-done-backport-source-one/REMOTE-PROTOCOL-RESULT-source-four.json.
Earlier socket-length failure and event-loop timeout remain recorded; final
test uses a short private socket and blocking signal waits. Test adapter
preserves active preedit and forbids pending-state resurrection. Production
CPP matches pinned5.25.1; no extra production flush or client input.

Next frozen private native source-two:51 host source PASS, zero-key GI PASS,
one expected schema RED; static review9/10/no blockers. Manifest
85013e811ecf3b4df7093d45b0b587dfb315d812489ecb6219bb592d546ba7e6.
Private dependency bind and exact KWin maps/root/host identity are required
before the same four original Chrome cells. Native NOT_TESTED; detailed
provenance, failed attempts and proof boundaries are in the owning entry.
Full GNOME354/46/32 and KDE175/233/24 of432 unchanged; no install, publish,
Lay delivery-route, model or runtime-authority change.

### 2026-10-18T22:30:12Z — kbpa four Chrome failures become native PASS

Same original Chrome fields7/1 × roundtrip/space_burst5:4 PASS/0 FAIL after
the private upstream KDE dependency fix, compared with kwdh4 FAIL.14 successful
Double Shift samples: p50 172.759ms, p95/max178.456ms. No changed Lay/IME/model
bytes, keys, cadence, targets, deadlines, runtime authority or delivery route.
Before input, exact KWin process/maps and /proc/PID/root identify the bound
candidate dependency; host library is stock. Retained wire commit/last-done
serials agree at both ends of every gesture (14/14), versus stale pairs before.

RESULT SHA ae0d977155b55db1e92012a53c074a45f9f73c5cf18ed825d6e197dc51bac08a.
Independent CLOSED22:30:12Z;96 files SHA/size verified; archive SHA
5b434264834dd9ffa8939165844bb3c8f0e9112f66020c34a64edac72d7a7667.
Cache kde-private-upstream-backport-native-source-two/chrome/EVIDENCE.json;
wire analysis SHA7c3166d580a9a1ec6d2979879cfe29d4e6e5c990d3e7a95c8d8ade77b1876efa.
Next restore original J full selector in the same qualified harness. Complete
GNOME354/46/32 and KDE175/233/24 of432 unchanged. No install or publication.

### 2026-10-18T22:42:55Z — frozen original KDE432 native expansion started

Same upstream dependency and Lay/data pins as kbpa, original J432 identities
and case bodies restored;6x51 source PASS,6 expected RED,6 zero-key GI PASS.
Outer orchestration11 offline controls PASS and review9/10. Review fixed
timeout cleanup/export and missing raw snapshots before native input. Archives
stream their bytes; every referenced2MiB-bounded snapshot is hash/size checked.
Exact phase/unit/cgroup/CLOSED-SHA independent closure is required before the
next app; only all432 identities can replace the full result.

Cache kde-upstream-full-matrix-source-one; PLAN SHA
65ad5f5f2dfa688bfaf824ace0723bf0dbe45efe2b765bfc56e63f07ff501300;
ORCHESTRATION-PINS SHA
a7d6a6cf90d5d010d31f580bffaa443c6e0718b629cbb94a4e2758539bfe3993.
kbpa missing16 raw snapshots recovered separately, no repeated native input;
supplement archive SHA
a50ee97d00b20d16affe6928af1923eeb4141d89352e2c76b77af5d34a21511b.
Details and proof limits are in the owning architecture entry at this timestamp.
Chrome/kbpb RUNNING; no new complete matrix. GNOME354/46/32 and KDE175/233/24
of432 unchanged, NOT_TESTED0; no install/publish/model/Lay route/authority change.

### 2026-10-18T22:55:31Z — KDE Chrome160 complete, full expansion interrupted

Original frozen Chrome160:130 PASS/30 FAIL/0 BLOCKED, all21 former PASS retained.
FAIL families prefix10/selection10/focus_return10. Successful measured Double
Shift180 samples p50 167.812ms/p95 178.287ms; Space50 p50 29.068ms/p95 31.564ms.
Private IBus prerequisites and upstream dependency backport are included in
the comparison; this is not isolated Lay-code or installed-runtime acceptance.
RESULT SHA454d9c46392466724e2461e88efe443abc78427be24902abe33a744f5049db44.

Firefox stopped after two finished FAIL rows: first rich clear unqualified,
then field10/short exact-tail replay refused between deletion and insertion.
KWin UUID differs; AT-SPI deactivation and IME Reset/FocusOut/new FocusIn are
recorded. Cause and actual panel involvement remain UNKNOWN. Not Responding
caption alone is not a hang proof. No third behavioral cell or later app phase
completed. Frozen success collector remains failed; separate101-file aborted
forensic export SHA
bb090f96a2e9b14bc0466016afb68a708e381cb2941f423c92b42072cd61cc85
independently confirms empty cgroup/VT1/no held keys without changing original
CLOSED's null field. See owning architecture22:55:31 entry and caches
kde-upstream-full-matrix-source-one, kde-upstream-aborted-firefox-forensic-source-one.
GNOME354/46/32 and KDE175/233/24 complete denominators remain unchanged. Next
is causal context/focus evidence; no unchanged retry or promotion. No user
installation/publication, model, Lay delivery-route or authority change.

2026-10-18T23:08:06Z: prepare a bounded Firefox field10 roundtrip/short causal
diagnostic in cache kde-firefox-panel-causal-source-two. Exact original case
AST/keys/targets/waits/binaries/guards retained. Passive KWin activation/window
type events and same-panel Wayland wire logging only; no runtime repair yet.
Source-one50 PASS/1 FAIL caught integer-vs-string selector before keys and is
preserved. Source-two remote guarded51 PASS, expected RED and actual GI zero-key
PASS; native NOT_RUN. Actual KWin activation can be proved by positive event,
not inferred from a popup caption or missing signals. Detailed pins/limits in
owning architecture23:08:06 entry. No full matrix/authority/install change.

2026-10-18T23:08:55Z: kbph completed two original Firefox iframe rows as FAIL,
then ABORTED on window deactivation;20 actor events, no held keys. Actual KWin
clientActivated plus panel Wayland keyboard.enter/leave now directly establish
temporary IBus panel focus, without clock-base conversion. Which widget or
request shows it remains unproven. Required exact-foreground rejection stays.
Two-cell RESULT SHA e348805cc1ee122d6caa2e02be980ea40347e0a03d38c3d0038c61b76591abbe;
independent CLOSED_ABORTED_EVIDENCE_ONLY and92-file hash-verified export are in
cache kde-firefox-panel-causal-source-two/evidence. Original null fields retained.
See owning architecture23:08:55 entry. No full matrix, route or install change.

2026-10-18T23:25:03Z: kbpm prepares the same two original Firefox iframe cells with
passive private-IBus panel-method logging only. Exact-sender GetId readiness,
2 MiB cap and monitor exit checks; remote source53 PASS, expected RED and GI
zero-key PASS. Separate zero-key private-bus observer probe passed and closed.
Widget/auxiliary cause still a hypothesis; no runtime patch or native result
yet. See owning architecture entry for pins and alternatives. No authority,
route, installed runtime or complete-matrix change.

2026-10-18T23:31:30Z: kbpm source-two native2 FAIL then ABORTED, independently closed,
95-file export953a69a0d86abfb7968d77ac7054f05f042f77f10dceb349fae1429db0a3890e.
Positive auxiliary show/clear calls correlate with a normal IBus toplevel
activation about47ms after clear. Pending clear and pending show each have
100ms upstream timers. Plan one isolated CandidatePanel null-clear lifecycle
fix; preserve nonempty hints, native keys/verdicts and Lay authority. Legitimate
hint nonactivation remains unproved. Detailed evidence/limits/alternatives in
the matching owning architecture entry; complete matrices unchanged.

2026-10-18T23:36:34Z: private upstream panel null-clear candidate built; only
panel binary changes (e6c42ff073fe8017cffa866b1bf9be765efa4afcea95474ddb326a72dd18b013).
Actual extracted GLib lifecycle controls: qualified stock RED/candidate GREEN,
ordinary nonempty hint preserved in both. Review9/10, no P1/P2. Same two
original iframe cells frozen as kbcl; source53 PASS, expected RED, actual GI
zero-key PASS. Native NOT_RUN. See matching architecture entry for pins and
scope; no Lay route/authority or installed runtime change.

2026-10-18T23:46:02Z: kbct minimal upstream null-clear native: original Firefox
iframe short PASS, roundtrip FAIL at strict clear metadata prerequisite.
Both retain Firefox foreground; first attributes mapping empty, second valid.
Do not regrade the first row or conflate zero-input kbcl/kbcn setup errors
with native failures. Source53 PASS/expected RED/GI zero-key PASS; independent
closure and97-file SHA-verified evidence in
`kde-panel-clear-qualified-source-two/evidence`, archive
8f4657903c50cf7ed582e9bb3b188db069d10f6f27ae46cc07194ae1ff28be5a.
Two DS samples median129.321/p95147.563ms, diagnostic only. No installed Lay,
route, authority or full-matrix change. Next: cold metadata readiness within
the existing clear deadline and unchanged strict shape predicate.

2026-10-18T23:55:33Z: strict rich-clear readiness source gate58 PASS, actual
GI zero-key PASS; pinned preceding callsite yields1 expected assertion failure
on the actual cold metadata control. One clear, unchanged predicate, original
1.8s deadline/max8 captures, retained samples; hard guard/read/identity errors
and late captures cannot pass. Decision2026-10-19-native-clear-readiness-deadline;
matching owner entry and `kde-panel-readiness-stock-source-one` pin receipts.
Native NOT_TESTED; compare both panels with the same observation contract
before causal claims. Original failed rows, Lay authority/routes and complete
matrices unchanged.

2026-10-18T23:58:22Z: readiness source-one review7/10 identifies two P2:
retry after explicit raw/root focus failure, lost received tree on postcheck
error. Source-one rejected, no native input. Corrected source-three saves
before postchecks/rethrows with error record and hard-fails focus loss.
Remote59 PASS, expected old-callsite RED, actual GI zero-key PASS. New source
pins in matching owner entry and `kde-panel-readiness-stock-source-three`.
Native NOT_TESTED; unchanged original cases, strict predicate and matrix totals.

2026-10-19T00:53:44Z: exact KWin nonfocus IBus-popup rule qualifies the shared
presentation boundary: kbcy original Firefox12/Chrome12 rich controls all PASS,
including short and both five-repeat plans. Seven mapped panels wantsInput=false,
none active;84 DS samples median150.225/p95173.559/max177.705ms. Verified324-file
export8f8c9a87d0b6c5989421f61c7e9c5ab2d0867a3bb9cbafa21b70cc1779c14dad and
independent closure. Details/pins/limits in matching owner entry and decision
2026-10-19-kde-ibus-panel-focus. Production installer integration has seven
focused tests and shell syntax PASS; full Firefox160 helper-generated-rule
qualification pending. No installed runtime/route/authority or total-matrix change.

2026-10-19T01:10:37.484536+00:00 — Panel-focus fix complete Firefox160:123PASS36FAIL1BLOCKED (historical80/73/7);59mapped panels nonfocus,0activation. DS178samples median164.587ms p95178.964ms. Verified1680-file export SHA4570cf01cb37212d477b708cc8a106ca1cb31a6f561ab3c5d0b51e8b4710053d, independent CLOSED. Earlier kbcz64-row prefix ABORTED due passive2MiB log cap, retained separately;14MiB proportional retry leaves native targets/cadence/guards intact. Installer migration defect fixed from actual numeric KWin fixture,9remote tests zero skips PASS. Chrome160 next; full432 totals historical, TD159 IN_PROGRESS. No user install, IME route or edit-authority change. Owner: firefox-input-stability-2026-10-05.md, full Firefox component entry.

### 2026-10-19T01:19:00.540675+00:00 — complete KDE browser components

Firefox123PASS/36FAIL/1BLOCKED and Chrome130PASS/30FAIL/0BLOCKED,
253/66/1 of320 original browser cells. Other112 KDE and GNOME cells are
not regraded. Chrome independent closure, receipt/export hashes and timing
are in the owning Firefox stability document. Next shared mechanism is the
missing KDE group completion for owned preedit; three actual-callback source
regressions are controlled RED before implementation. TD159 stays IN_PROGRESS;
no user installation, publication, authority relaxation or delivery-route change.


2026-10-19T01:47:30.372107+00:00 — TD159 remains IN_PROGRESS.
852 correctness +3 isolated performance tests PASS; final graph-bound private
IME10155e00 qualified on original20 prefix cells:Firefox10/0/0,Chrome8/2/0.
No additive matrix claim: latest complete old-IME browser proof remains253/66/1
of320. Two Chrome residuals are separately attributed (iframe layout mismatch;
Search pre-gesture AT-SPI/IME disagreement). Native export independently CLOSED.
Exact pins, timings, first failed transitions and next passive-observer complete
320 proof are in the owning firefox-input-stability document, heading
2026-10-19T01:47:30.372107+00:00. No user install, publication or route change.


2026-10-19T02:04:16.607557+00:00 — Complete original320 new-IME browser proof:
Firefox139PASS/21FAIL/0BLOCKED,Chrome136PASS/24FAIL/0BLOCKED;275/45/0
combined (+22PASS vs preceding complete253/66/1). Independent closure and
3290-file verified export SHA1dbac3eb00698e72bb162284c48daa88228bde767fa55f6edcf47b12ca5fa6e2.
First failed mechanisms and timings are in owning firefox-input-stability doc,
matching complete qualification heading. Passive bus and existing Wayland wire
support a late old-RU modifier echo in Chrome after confirmed US; upstream
publication hypothesis is not yet implemented. Forty authority refusals, one
Search pre-gesture observation discrepancy and Firefox URL Space are distinct.
Other112 KDE/GNOME not regraded. TD159 IN_PROGRESS; no user install, publication,
route or admission relaxation. Export-only50MiB single-browser bound failure
retained, proportional100MiB two-browser export completed without input rerun.

2026-10-19T03:54:44.109339+00:00 — Fresh-focus manual-only source proposal: controlled semantic RED retained; first implementation3PASS/3FAIL, no native promotion. UnknownStart and existing exact executor remain mandatory. Full consequence analysis, alternatives, source receipts and rollback boundary in owning Firefox stability document, heading Fresh-focus explicit manual source. Last complete original KDE432360/68/4; TD159 IN_PROGRESS.


2026-10-19T05:50:23Z — TD159 before runtime edit: exact f040 full432 regressed
to342/86/4 (browser291/29/0). Retain failed seed observation cases. Next shared
mechanism is reducer-revoked native acquisition whose fence makes successor
Busy. Existing adapter retirement boundary proposed; no new owner/timer/route.
ADR2026-10-19-revoked-native-acquisition-retirement and owning stability entry
05:50:23Z bind hypothesis, three controlled P2P tests and required negatives.

2026-10-19T06:31:38.067380+00:00 — TD159 exact9804 native112 completed86/22/4;
matching browser320291/29/0 yields full432377/51/4. GTK/Qt composition observer
qualification changes measurement; not35 runtime fixes. Retained cold-start
RED and16GREEN, three aborted preparations, final76source controls plus two
controlled REDs. Fully hash-verified kbea export51e7f945... and independent
closuree184d0a1...; source/byte/receipt/latency details in owning stability doc.
Next first failed transition: GTK preedit exists before focus-away, vanishes
on return before gesture; inspect PREEDIT_CLEAR vs typed-only commit mode.
Qt preserved-word manual refusal remains separate. No route, user install,
publication or authority relaxation; TD159 remains IN_PROGRESS.

### 2026-10-19 owned preedit blur — exact native112 milestone

Exact private IME510c920197ccf59a9c2701a67feadb4a0879321b5ae38cd2f26ac5906e9fc06a:
original11287PASS/21FAIL/4BLOCKED; only GTK TextView focus_return changes FAIL
-> PASS from old11286/22/4. Entry preserves the word but reselects it on focus
return; Qt preserves it but refuses manual authority. No status relaxation.
Verified export d5fb312ad19eb9f5aa4df5e154dc508c52444d58388c2e07c8a8b77cbe54213f;
independent close2026-10-19T06:57:38.766691+00:00. Owner contains per-app timing,
full proof limits and retained exporter preparation failure. Browser320 phase
kbec pending; no combined new-byte432 or installation/publication claim.
Next investigate Qt same InputContext Reset followed by fresh exact text and
context_authority refusal; never restore historical preedit or bypass verifier.


Owned-blur original browser320 CLOSED_DIAGNOSTIC_ONLY at
2026-10-19T07:06:04.516331+00:00 on the same exact private IME510c:
Firefox150PASS/10FAIL/0BLOCKED, Chrome141PASS/19FAIL/0BLOCKED;
total291PASS/29FAIL/0BLOCKED. Exact320 identities/actions remain unchanged.
Two status changes from previous9804: Firefox field4 moved_boundary FAIL->PASS,
Chrome field8 focus_return PASS->FAIL. The Firefox improvement was previously
a late-ack refusal; one run does not establish a permanent timing repair.
The Chrome regression is unresolved; do not accept or promote this candidate.
Receipt `owned-preedit-blur-browser-source-one/RUN.json`; exact comparison
`owned-preedit-blur-browser-source-one/COMPARISON.json`.

Chrome8 focus_return preserves committed `привет`, caret6, no selection.
Physical Double Shift is recognized; IME manual outcome status3 prepares the
conversion, but the daemon cancels after layout handoff with "exact IME tail
retention policy changed". No actual privacy-policy change is established:
VisibleTailV4 also returns false for passive/unknown-context replies, and
current bound_v4_reply compares retention before classifying readiness.
The case's bounded trace snapshot has PREFIX_CHANGED, so no event absence
claim may be made from its interval. Content hints64/purpose0 do not prove
private mode. Capture exact private RPC replies before changing any guard.

Verified browser export SHA256
f9530b99a518765ef23656da03a7487f56724adf19ac706e823248eabc5816b0:
29277590bytes,3292files,1280 raw snapshots/448392601bytes. Independent closure
SHA2565843e004176e52ddec162470afee99af63a045a50bb0dbc4f89a5fb4e0ae3aa1
verifies empty cgroup, pinned processes gone, no held keys, VT1, owned seat
stopped, temporary policy absent and host libraries unchanged.
Successful Double Shift Firefox n200 median30.172ms,p95176.092ms;
Chrome n185 median30.314ms,p95175.178ms. Focus-return successful only:
Firefox n10 median164.266ms,p95200.066ms; Chrome n8 median165.826ms,p95169.999ms.
Space Firefox n50 median28.864ms,p9532.977ms; Chrome n50 median29.273ms,p9532.054ms.
Failed operations remain FAIL, not latency samples.

Both new-byte components are now independently closed. Exact matching432:
378PASS/50FAIL/4BLOCKED. Derived proof with all six receipt hashes is
`owned-preedit-blur-matching-432-source-one.json` under the development cache.
The previous instruction to avoid combining old browser bytes is superseded
only by this exact-byte closure, not by cross-build aggregation.
TD159 remains IN_PROGRESS. No installed bytes, delivery routes or authority
contracts changed. No GNOME acceptance or release qualification.

Next diagnostic kbed: original20 browser focus_return identities only, same
510c bytes/actions/cadence/deadlines. Add only a bounded passive session-bus
monitor pinned to the exact private IME owner/PID; verify a zero-key probe
before input. Source controls60PASS plus two expected RED controls passed at
2026-10-19T07:15:35.596649+00:00. This subset cannot replace browser320 or
matching432 evidence and does not authorize any permission relaxation.


### TD159 — 2026-10-19 matching target observation revokes exact manual proof

kbed diagnostic completed on private510c, with no runtime changes: Firefox
focus10PASS0FAIL, Chrome focus8PASS2FAIL. This subset does not replace the
complete browser320291PASS29FAIL0BLOCKED or matching432378PASS50FAIL4BLOCKED.
Chrome7 fails after layout handoff; Chrome2 stops at seed-readback (`приве`,
caret6), a separate measurement/seed problem. Chrome8 passes in this subset.

Passive monitor pins the private IME unique bus owner and process and records
77V4calls/77replies. At unix1792394175.367585, RU27 returns exact `привет`,
epoch405, same InputContext16, retention=true. US28 FocusIn occurs at .411020;
its actual SetSurroundingText `привет`,6,6 occurs at .412112. At .444513 V4
returns passive:unknown-context with retention=false. The actual matching
observation precedes the failed query by32ms; no changed privacy metadata
or delayed missing observation explains this failure.

First lost authority: observe_external_surrounding_text always calls
advance_surrounding_observation_revision, which drops both the inherited
target certificate and reducer exact_manual_snapshot. If observation arrives
before installation, existing installer republishes its matching receipt;
after installation it is irretrievably dropped. The former test explicitly
required even identical later callbacks to revoke: this is the policy race.

Decision2026-10-19-exact-manual-observation-refresh: retain only a current
receipt across an actual fully matching prefix/tail/collapsed-end tuple, same
owner/epoch, original expiry. Changed/selected/interior/absent/revoked/consumed/
expired/sensitive proofs cannot revive. UnknownStart remains UnknownStart;
no new owner, delivery path, output mutation or verifier relaxation.
Ratings: existing-certificate refresh9/10; extra waiting2/10 (SSI already
arrived); ignore SSI or accept unknown reply1/10 (unsafe authority bypass).
Three new semantic tests added; runtime refresh not yet implemented/tested.

Diagnostic RUN sha2569037c956839aa515844bea5c63b836fae3a2c49042f5c89c9407f466e10cd9eb.
Export bf278500efe455c34c38a411f2e5cd4952518eb509d3a4bebb6c372b25d5ae1b,
1500826bytes294files80raw18325175bytes. Closure at07:17:12.498445UTC:
00f387e1a0faa768f6f0e1167469ca0546c1f76707d1131749afdd96801933ef,
empty cgroup/pinned processes gone/no held keys/VT1/owned seat stopped/
temporary policy absent/host IBus and KWin libraries unchanged.
All receipts under development cache/owned-blur-focus-rpc-source-one.
No installation/publication/GNOME acceptance. TD159 remains IN_PROGRESS.


Exact-manual observation refresh controlled RED at2026-10-19T07:37:28.945514+00:00:2PASS1FAIL,
expected failure on the first actual target observation after install;
changed/revoked/no-gesture negatives pass. Runtime change uses existing
reducer's current certificate getter and publisher, no new state or route.
Capture precedes invalidation; refresh requires known full prefix equality,
unchanged tail/epoch, collapsed end-of-field, current owner and live handoff.
Original expiry is reused and revalidated by the reducer. Prior unconditional
repeat-revocation test now proves changed-prefix revocation plus no revival.
Focused GREEN at2026-10-19T07:38:47.040662+00:00:3PASS0FAIL.
Receipts exact-manual-observation-red-source-one/RESULT.json and
exact-manual-observation-green-source-one/RESULT.json under development cache.
Native remains NOT_TESTED for these changed bytes; complete matrix unchanged.

Formatted focused GREEN3PASS0FAIL at2026-10-19T07:40:34.855394+00:00.
Getter invalidation has one production caller, the synchronous engine
observation revision advance; callback/output consumption runs under the
same engine mutation serialization. Concurrent reducer revocation/tail/
lineage changes are revalidated at the existing publisher. No second mutable
certificate store was added. Scoped source gates do not certify physical
Chrome readiness. Official hermetic gate is now running remotely.

First official hermetic gate at2026-10-19T07:42:51.245630UTC:
877PASS1FAIL/878 correctness,881 discovered, performance NOT_RUN. Existing
td159_fresh_focus_manual_source_cannot_revive_after_receipt_change_or_reset
catches overbroad source-engine refresh. Do not weaken this source protection.
Narrow to a receiver using existing sealed source handoff path, which transfer
retains while active_path and reducer owner move to the target. Source path
equal to current engine cannot refresh; current target still needs all
certificate/owner/epoch/full-observation/original-expiry checks. No new state.
The recorded failing whole-set gate is the RED evidence for this narrowing.

Receiver-only focused GREEN3PASS0FAIL at2026-10-19T07:44:42.322630UTC.
The sealed existing source path distinguishes the admitted receiver; original
source callback protection remains in the unchanged old test. Second official
hermetic/format gate now running. No native claim or installed changes.

Receiver-only official hermetic GREEN at2026-10-19T07:47:07.618129UTC:
878PASS0FAIL/878 correctness,881 discovered,343 isolated,535 bulk;
all3 performance tests PASS. Remote formatting check passed. The unchanged
source callback/reset protection is retained. Exact receipt is development
cache/exact-manual-observation-hermetic-source-two/RESULT.json. These source
results do not establish native acceptance or browser latency. Next is the
remote architecture graph gate and a pinned private build, followed by the
unchanged original browser320 and native112 protocols. Delivery routes,
installed binaries and desktop applications remain unchanged.

Remote architecture graph gate PASS at2026-10-19T07:48:48.667743UTC;
existing scripts/update-architecture-graph.sh completed successfully and all
six generated outputs were hash-verified on export. Receipt: development
cache/exact-manual-observation-architecture-source-one/RESULT.json. Runtime
change remains limited to receiver certificate refresh; no delivery change.
Pinned private release build is next; native acceptance remains NOT_TESTED.

Pinned private IME build completed at2026-10-19T07:52:50.314986+00:00:
SHA256 fbd4fd2ed5da81857d6c70e77f93a3daf6331b79aa92217399761db484704af4, bytes8466368.
Build receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/exact-manual-observation-final-build-source-one/RESULT.json;
receipt SHA256 8c6e34f8c96a33b6b5aab06055560c0a1e40ccc2bcd0f5f0296aed3d8de09588.
All878 source correctness and3 performance checks are bound to identical
Rust sources; generated architecture receipt-only delta has its own embedded
receipt test PASS. No installation. Browser320 kbee uses this exact binary;
native112 kbef must use the same bytes before a matching432 comparison.

Receiver-only exact-manual refresh original browser320 completed; independent
closure at2026-10-19T08:00:33.456117+00:00. Exact private IME
fbd4fd2ed5da81857d6c70e77f93a3daf6331b79aa92217399761db484704af4.
Firefox150PASS10FAIL0BLOCKED/160; Chrome142PASS18FAIL0BLOCKED/160;
browser total292PASS28FAIL0BLOCKED/320. Exactly one changed identity versus
510c: Chrome8 focus_return FAIL->PASS. Remaining319 statuses unchanged, all
320 identities and input/observer protocol scripts equal after phase-only
normalization. No native112 or GNOME acceptance from this browser component.
Confirmed successful DS latency: Firefox n200 median29.955ms p95176.665ms;
Chrome n185 median30.276ms p95172.633ms. Successful focus-return Firefox
n10 median162.632ms p95180.031ms, Chrome n9 median160.834ms p95180.299ms.
Space n50 each: Firefox median29.764ms p9533.471ms; Chrome median29.392ms
p9532.349ms. Clock remains evdev-write-return to confirmed target; failed
operations remain FAIL and are excluded from successful latency samples.
Full raw export SHA25655ffa73af5fca4682d6b464d23adabd650871646cc27d45d1b6a471b963cb5c5,
28586306bytes3292files1280raw448661347bytes. Closure SHA256
93fc2f374fc74bdebd4ba5c9ee6dec89eeb943565d227f600f4b07698a27ff1c; empty owned cgroup,
pinned processes gone, keys released, VT1 restored, seat stopped, temporary
policy absent, host IBus and KWin libraries unchanged. Evidence and complete
post-row comparison: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/exact-manual-observation-browser-source-one/COMPARISON.json.
Same-byte native112 kbef is prepared; no installation/publication.

Receiver-only refresh original native112 independently closed at 2026-10-19T08:08:52.185671+00:00.
GTK29PASS3FAIL0BLOCKED/32; Qt42PASS6FAIL0BLOCKED/48;
KWrite13PASS2FAIL1BLOCKED/16; Konsole3PASS10FAIL3BLOCKED/16.
All112 identities, statuses and phase-normalized input/observer protocol
scripts are unchanged versus510c. Same fbd4 IME bytes as browser320.
Combined432 at 2026-10-19T08:09:08.028376+00:00:379PASS49FAIL4BLOCKED,
previous378PASS50FAIL4BLOCKED; exactly Chrome8 focus_return FAIL->PASS.
Receipt: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/exact-manual-observation-matching-432-source-one.json
Native raw export SHA256 99b8ff39de4d1969856c56e0300128d7d3fa6f655a86078926ead2aca9cad9a9, 8813867bytes 1225files,
448 raw endpoint snapshots/126946779bytes; each member verified.
Native closure SHA256 c319524738a7ce9e8b7695437139c853ec3008803f7f6e349a9ffb6bbac168e0;
empty owned cgroup, pinned processes gone, keys released, VT1 restored,
owned seat stopped, temporary policy absent, host IBus/KWin bytes unchanged.
Export first deferred to an unrelated L3 heavy lease; no lease bypass or
process interruption. Before export, corrected a stale template IME hash and
required agreement with the separately pinned private override; no product
code, native inputs or verdict changed by this evidence-script correction.
Native successful DS latency: GTK n40 median28.241ms p95164.401ms;
Qt n60 median28.588ms p95138.485ms; KWrite n20 median30.078ms p95140.823ms.
Konsole has no successful DS latency sample. This is not an acceleration claim.
Scope remains private KDE diagnostic: upstream KWin qualification failures
and GNOME are not accepted, no installation/publication.
Next shared failure: all three Qt focus_return plans retain committed привет
with collapsed end caret after Double Shift. GTK Entry focus_return instead
stops before gesture because the seed is selected; those mechanisms remain
separate. Analyze the real Qt Reset/observation/gesture order before editing.


### TD159 owned-preedit Reset current-field admission — design at 2026-10-19T08:23:19.659920+00:00

Original native112 on the pinned fbd4fd2e IME confirms three Qt focus_return failures: RichTextEdit, PlainTextEdit and LineEdit retain actual `привет`, collapsed end caret and focus, but the recognized Double Shift is refused with context_authority. Actual Reset callbacks retire owned preedit; source inspection identifies a second broad revocation during its cleanup. This is distinct from GTK Entry selecting its seed before the gesture. Preserve existing cancellation: no old preedit or committed prefix may become a new field source. ADR `decisions/2026-10-19-owned-preedit-reset-manual-source.json` proposes only a new exact manual read after authenticated owned-preedit retirement, actual post-reset surrounding observation and explicit gesture. Two new semantic test functions are prepared; production code is unchanged and controlled RED is pending. Latest complete diagnostic stays379PASS/49FAIL/4BLOCKED; no source/native acceptance claimed for the proposal. Solo remote-only, no installation/publication or delivery-route change.

Owned-preedit Reset first RED attempt at 2026-10-19T08:24:35.046747+00:00: the positive test reached and failed the intended missing-fresh-admission assertion; the negative test also stopped on a detached callback transport reply (serial9203 before9204). Overall source-one is FAIL and is not accepted controlled RED. The fixture now drains each real detached key callback reply before Reset; no product code or expected behavior changed. Source-two is the next independent frozen run.

Owned-preedit Reset controlled RED at 2026-10-19T08:25:08.775874+00:00: source-two1PASS/1FAIL, exactly the required new-current-field manual-read assertion fails. Negative source safety test passes after fixture reply ordering correction. Minimal reducer/state/Reset-adapter implementation prepared at 2026-10-19T08:27:08.569439+00:00; no GREEN or physical acceptance claimed. Test source and output routes are unchanged after RED.

Owned-preedit Reset first implementation at 2026-10-19T08:28:20.797486+00:00: frozen green-source-two remains1PASS/1FAIL on the same intended positive assertion, all new negative checks pass. This is not GREEN and no build/native promotion occurred. Added bounded assertion diagnostics (old/current token, live status, reducer fresh flag, actual snapshot) to identify the first failed guard before further eligibility changes. The new path explicitly excludes existing owned Space/managed-commit/replay observations and pending manual/replay state; those keep their existing contracts.

Owned-preedit Reset fixture diagnosis at 2026-10-19T08:35:08.287755+00:00: green-source-three1PASS/1FAIL identifies hints2048 applied after typing, which correctly clears prior volatile preedit at the privacy boundary. Green-source-four1PASS/1FAIL stops before Reset because actual private input uses the committed route and emits CommitText; it cannot satisfy the owned-preedit-only prerequisite. These are recorded fixture failures, not runtime GREEN. Source-five keeps ordinary/repeated Reset positive cases, establishes private metadata before actual keys, asserts the real private commits/no-owned-preedit, and checks that this path cannot invent fresh owned-source admission. Privacy changes remain a separate negative guard; no privacy/delivery contract changed. Evidence: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-green-source-three/RESULT.json and /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-green-source-four/RESULT.json. Native remains NOT_TESTED for the proposal; latest complete432 stays379PASS49FAIL4BLOCKED.

Owned-preedit Reset focused GREEN at2026-10-19T08:36:29.866135+00:00: formatted frozen source-six2PASS0FAIL; source-five also2PASS0FAIL before formatting. Positive scenarios one/two actual Reset callbacks, new Unicode-prefixed text, no old prefix/preedit, no automatic authority. Negative scenarios include unobserved Reset, selection, interior caret, sensitive field, revoked owner, privacy change and actual private committed input without owned preedit. Source-six log SHA25642b508a94c74febd2471b77016e2ad401b0a9536659061e6b75b9e0bce98175d. Remote six-file format PASS at2026-10-19T08:35:26.316860+00:00. Official880correctness+3performance gate started; physical result NOT_TESTED. Receipts: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-green-source-six/RESULT.json and /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-format-source-one/RESULT.json.

Owned-preedit Reset complete source GREEN at 2026-10-19T08:39:14.269243+00:00:883 discovered,880 correctness/package executed PASS (345 isolated,535 bulk),3 performance PASS; no failures or selection change. Source-six frozen Rust and tooling closure unchanged; six-file remote rustfmt check PASS. Receipt: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-hermetic-source-one/RESULT.json; discovery SHA256138a02c2a8868db0dd09d0267d97d7b94b00397e4bc0bdc9d9beed6089967cde. Source authority delta is only eligibility for a new actual current-field manual observation after authenticated owned-preedit retirement; no old text authority, automatic authority or delivery-route change. Architecture refresh and immutable private build are next, native acceptance remains NOT_TESTED. Latest complete432 stays379PASS49FAIL4BLOCKED.


Owned-preedit Reset architecture/build qualification recorded at 2026-10-19T08:50:12.424214+00:00: remote architecture gate PASS at2026-10-19T08:40:40.266332+00:00, receipt development cache/owned-preedit-reset-manual-architecture-source-one/RESULT.json. Pinned private build at2026-10-19T08:44:37.890917+00:00: SHA2561c1641d4e61e3c7409f93c5dd0ba0525d6cc0e4f66a613ae267b37a8bce467ba,8469312bytes; receipt development cache/owned-preedit-reset-manual-final-build-source-one/RESULT.json,880 correctness/package and3 performance source tests bound to frozen Rust/tooling, embedded architecture receipt PASS.
Browser package source-one zero-key qualification failed59/60: prepare.py expected the ADR filename stem as its semantic id, whereas the pinned ADR id is TD159-owned-preedit-reset-current-manual-source. Binary, tests and source counts were correct. Preserve this failure; no keys were sent for source-one. Immutable source-two corrects only the expected semantic id; pinned binary/source/ADR checks remain. All60 controls PASS and both intentional negative controls fail as expected. All eight input/observer protocol scripts exactly equal previous browser320 after phase-only normalization. Original browser320 kbeg is now running on the new bytes; native112 kbeh follows. No physical result yet for this proposal; latest complete matching432 remains379PASS49FAIL4BLOCKED. No installation/publication or delivery-route change.


Owned-preedit Reset original browser320 independently closed at 2026-10-19T08:56:29.418911+00:00: pinned IME1c1641d4e61e3c7409f93c5dd0ba0525d6cc0e4f66a613ae267b37a8bce467ba, Firefox149PASS11FAIL0BLOCKED/160, Chrome142PASS18FAIL0BLOCKED/160; total291PASS29FAIL0BLOCKED/320. Exact320 identity set and eight phase-normalized input/observer scripts equal fbd4 baseline. Exactly one changed identity: Firefox2 moved_boundary PASS->FAIL, actual `должн ыбыть ` remains instead of `должны быть `. Remaining319 statuses unchanged. Candidate is found and authorized; captured trace reaches active_pair_waiting_committed_snapshot and then a Reset. This is a first-failed-transition lead, not yet proof that the new owned-source path caused the failure; bounded producer completeness remains UNKNOWN. Do not accept the proposal with this browser regression or hide it by repeating until PASS.
Full verified export SHA256a333c0c19c18394b17c0db618df05bf4af4e95325f03b88a27cc5e80fdaed40a,29140833bytes3292files1280raw448607617bytes; closure verifies empty cgroup, pinned processes gone, keys released, VT1 restored, owned seat stopped, temporary policy absent and unchanged host IBus/KWin bytes. Receipt: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-browser-source-two/COMPARISON.json. Successful DS: Firefox n200 median29.727ms p95174.631ms; Chrome n185 median30.217ms p95173.488ms, evdev-write-return to confirmed target; failed operations remain in counts and this is not an acceleration claim. Native112 kbeh now runs the same pinned bytes after76 protocol controls PASS and both controlled negatives. No matching432 claim yet; GNOME and upstream KDE qualification remain outside acceptance.


2026-10-19T09:06:46.955350+00:00 TD159 owned-preedit Reset proposal REJECTED at complete physical432:378PASS50FAIL4BLOCKED vs379/49/4, all three Qt focus_return unchanged FAIL, Firefox2 moved_boundary PASS->FAIL. Source880+3 and architecture PASS do not establish physical acceptance. Matching receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-manual-matching-432-source-one.json. Actual Qt order is surrounding->Reset->surrounding->Reset->gesture with snapshot absent; source tests covered final-Reset->surrounding instead. Next causal test must reproduce actual ordering before considering one existing pending-manual fresh read. No delivery-route change, install or publication.


TD159 same-gesture fresh-read experiment: controlled source RED2026-10-19T09:09:46.055655+00:00 2PASS1FAIL confirms Qt surrounding/Reset/surrounding/Reset/gesture has fresh admission but no snapshot. New read uses existing pending700ms continuation. First implementation2026-10-19T09:14:28.698446+00:00 3PASS1FAIL at privacy cancellation; second2026-10-19T09:17:09.813213+00:00 3PASS1FAIL at expiry cancellation. Repair spends existing fresh admission before cancelling pending read, adds nine cancellation gaps plus no-renewal polls. Receipts development cache/browser-final-fixes-20261018/owned-preedit-reset-read-{red-source-one,green-source-one,green-source-two}/RESULT.json. Owner docs/architecture/firefox-input-stability-2026-10-05.md; decision2026-10-19-fresh-manual-read-same-gesture.json. Native NOT_TESTED for this source; last complete432 remains378PASS50FAIL4BLOCKED, rejected. Runtime authority changed, delivery routes unchanged.


TD159 fresh-read focused GREEN at2026-10-19T09:20:46.454849+00:00:4PASS0FAIL, exact four identities; nine cancellation gaps including callback expiry and RPC expiry plus later-poll no-renewal. Controlled RED remains2PASS1FAIL. Exactly one fresh actual read, different post-Reset word accepted only via same-gesture existing exact-tail handoff; no callback-local Delete/Commit. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-green-source-three/RESULT.json. Full hermetic source, graph, private build and physical clients pending; no installation/publication or delivery-route change.


TD159 same-gesture fresh read full hermetic source gate at2026-10-19T09:23:15.969740+00:00:882PASS0FAIL correctness/package across885 discovered identities (535 bulk and347 process-isolated),3performancePASS. Official discovery and hermetic executor over frozen Rust/tooling closure; no ignored test substitution. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-hermetic-source-one/RESULT.json. All nine cancellation gaps and actual Qt callback/Reset ordering retained. Graph/private build/physical acceptance pending; this does not improve the latest complete432378PASS50FAIL4BLOCKED. Delivery routes unchanged; no installation/publication.


TD159 same-gesture fresh-read architecture gate at2026-10-19T09:24:30.507765+00:00:PASS; graph refreshed remotely with existing AST-only gate, all owning/callsite/delivery checks retained. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-architecture-source-one/RESULT.json. Prior full source882+3PASS; next immutable private IME build and matching physical clients. No new input/output owner, timeout, fallback, client bypass, installation or publication. Firefox2 pair-space trace independently re-read under exact report and endpoint SHA pins: selected correction awaits committed snapshot then Reset9511 rejects rereceipt for missing predecessor/post-reset token. Endpoints PREFIX_CHANGED; producer/interval completeness UNKNOWN. This positive event sequence is a lead, not proof that the current manual-read change caused or fixes the pair regression.


TD159 same-gesture fresh-read private build at2026-10-19T09:28:28.359252+00:00: IME SHA256a46dd831f94f29d244671f64096ebe1a0e04ae2663fca442f9137814fa547adc,8472256bytes; source882correctness/package+3performancePASS, graphPASS, embedded architecture receipt testPASS. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-final-build-source-one/RESULT.json. Frozen identical Rust/test tooling proof reused except refreshed generated architecture receipt JSON; all source hashes checked before/after. Scope: private diagnostic build only; native NOT_TESTED, latest complete432 unchanged378PASS50FAIL4BLOCKED. No installation, publication or delivery-route change. Next browser320 kbei then native112 kbej with original input scripts.

TD159 same-gesture fresh-read complete physical milestone at2026-10-19T09:40:35.346737+00:00: matching a46dd831 IME browser320+native112 gives380PASS48FAIL4BLOCKED/432 versus fbd4 baseline379PASS49FAIL4BLOCKED. Two recovered identities: Qt5 LineEdit/PlainTextEdit focus_return. Qt5 RichTextEdit focus_return remains FAIL. New Chrome2 moved_boundary FAIL: original12 characters plus corrected12 remain in the client; actual surrounding24 does not satisfy exact expected final12 and verifier stays pending_stale_observation. This is a distinct physical outcome from the earlier Firefox2 Reset/unchanged-text failure; do not recycle that hypothesis as a cause of duplication. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/owned-preedit-reset-read-matching-432-source-one.json. Original identities/protocol scripts/cadences/deadlines retained; independent closures and export hashes verified. Verdict PARTIAL_NATIVE_IMPROVEMENT_NOT_ACCEPTED, no installation/publication or delivery-route change. Next: exact RichTextEdit positive trace and Chrome deletion-versus-insertion causal boundary. Do not undo the two observed Qt recoveries merely to make browser counts look better; require a minimal reproducer before the next runtime change.


TD159 IBus Reset delivery-cache controlled RED at2026-10-19T09:47:26.776988+00:00:9PASS3FAIL; original eight upstream cases pass, failures exactly9/10/12 prove equal actual tuple suppressed after Reset, stale delivery cache survives repeated Reset, and invalidation is needed per engine. Actual macro-generated Reset body executes; no second reset implementation. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-red-source-one/RESULT.json. Qt RichTextEdit actual read queues then cancels after about711ms; proxy-cache source defect is proved, physical causality still a hypothesis. New explicit decision 2026-10-19-ibus-reset-surrounding-delivery-cache.json limits change to cache invalidation before existing Reset, with no cached replay or Lay source authority. Next final-control RED/lifetime then GREEN/private isolated dependency build, unchanged Lay a46dd831 bytes and original physical protocol. No installation/publication or delivery-route change.


TD159 IBus Reset cache RED/GREEN and private dependency build at2026-10-19T09:50:28.580211+00:00: baseline final controls9PASS3FAIL(cases9/10/12) and actual GObject lifetime FAIL as expected; candidate12PASS0FAIL and actual GObject lifetime PASS under fatal GLib warnings. Extracted actual Reset body retires only this proxy's delivery-cache object, forwards the existing Reset, and never publishes cached text. Source SHA baseline2958803ac0d121c82be85487578569b713cf5d4d05aa4b4c28019861b8ae9c57 -> candidatec6a87f20baecc9e20a8e204ec444f47c4ff0d1d8df69629394eda150dcbecde4. New private ibus-daemon SHA0b907f7c969c6621c667a613ef0c5ad0b4cb0ecc74a7e62caeef775afab98af7,662528bytes; only prefix/bin/ibus-daemon changed, panel remains46a335becaa4ed35fc217adebd2fd206ea5dda4af8abd1a06f60136dac7e3a23. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-build-request-source-one/BUILD.json. Source PASS is not physical acceptance: Qt RichTextEdit causality remains a hypothesis, latest complete432 remains380PASS48FAIL4BLOCKED. Next unchanged Lay a46dd831 and native112 identities/cadences/deadlines with only this new private IBus dependency, then matching browser320. No installation/publication, cached snapshot authority, delivery-route change or host IBus restart.


TD159 Reset-cache original native112 physically complete at2026-10-19T09:58:18.948843+00:00: unchanged a46dd831 IME plus private IBus0b907f7c gives90PASS18FAIL4BLOCKED/112 versus same-IME previous89/19/4. Only changed identity Qt5 RichTextEdit focus_return FAIL->PASS; actual committed native text changes привет->ghbdtn, final caret6/no selection/no preedit. All original112 identities and eight phase-normalized actor/observer protocol scripts match. Independent closure checks pinned processes/cgroup gone, no held keys, owned seat/policy removed, VT1 restored, host libraries unchanged; loaded private IBus build/hash explicitly bound. Export4de89305b8205baf95290c477e3108728af3d271df0127abb2004a900c7eba09,8500318bytes,1227files,448raw snapshots127403307bytes; archive and every member verified. Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-native-source-one/COMPARISON.json. Scope: one native112 pass, source-proved reset-cache defect and observed single-field recovery. Passing RichTextEdit IME endpoints remain unchanged with producer/interval completeness UNKNOWN; no inferred callback sequence or absence. Qt successful measured DS n60 p50=28.88ms/p95=137.89ms; focus-return n3 p50=141.06ms/p95=141.44ms, newly recovered RichTextEdit137.72ms; not all-case latency or acceleration proof. Browser320 with same dependency pending, so latest complete matched432 remains previous380/48/4 and must not be mixed with this native112. No installation/publication or delivery-route/source-authority change.


TD159 IBus Reset-cache full matching physical432 completed and independently
compared at2026-10-19T10:08:39.449724+00:00:382PASS46FAIL4BLOCKED/432 versus
same-Lay previous380/48/4. Firefox150PASS10FAIL/160; Chrome142PASS18FAIL/160;
browser292PASS28FAIL/320; native90PASS18FAIL4BLOCKED/112. Exactly two changed
identities, both FAIL->PASS: Qt5 RichTextEdit focus_return and Chrome field2
moved_boundary. Remaining430 statuses are unchanged. Chrome2 actual committed
text is now "должны быть " caret12/no selection, replacing the prior duplicate
"должн ыбыть должны быть " caret24. This observed recovery does not establish
the old client's precise DeleteSurroundingText processing sequence.

Only private IBus daemon0b907f7c969c6621c667a613ef0c5ad0b4cb0ecc74a7e62caeef775afab98af7
changed; Lay IME remains a46dd831f94f29d244671f64096ebe1a0e04ae2663fca442f9137814fa547adc.
The derived join verifies complete Lay/IBus/KWin/KWayland dependency bundles
match between browser320 and native112, and between their previous baselines.
All original432 identities and eight phase-normalized input/observer scripts
are unchanged. Independent closures verify no held keys, stopped owned seat,
empty cgroups, pinned processes gone, removed temporary policy and unchanged
host libraries. Browser export SHA256
37fceb052de575197fc2de14c11ff3c95fe3b7696b8ca8b0112611acb48e6a3a,
28340016bytes3294files1280raw snapshots448631347bytes. Archive and every member
are verified. Native export remains
4de89305b8205baf95290c477e3108728af3d271df0127abb2004a900c7eba09.

Successful native Double Shift samples: Firefox n200 p50=30.091ms/p95=176.217ms;
Chrome n185 p50=30.075ms/p95=174.198ms. Successful Space n50 per browser:
Firefox29.257/32.028ms and Chrome29.645/31.691ms p50/p95. Failed operations
remain FAIL and are excluded from successful latency samples; no acceleration
or all-case-latency claim. Qt successful DS n60 p50=28.877ms/p95=137.891ms.

Exact join receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-matching-432-source-one.json; SHA256 32c67640fd0d27a9d80cc9d5a4c410729f48cbebc839c83cf39d18bd0287807d.
Component comparisons are ibus-reset-surrounding-browser-source-one/COMPARISON.json
and ibus-reset-surrounding-native-source-one/COMPARISON.json under the same
development-cache root. Verdict PHYSICAL_IMPROVEMENT_NOT_ACCEPTED: remaining
FAIL/BLOCKED and private KWin qualification gaps, GNOME NOT_TESTED. Passing Qt
trace producer/interval completeness remains UNKNOWN; no inferred missing
events, callback causality or focus theft. No installation, publication,
host IBus restart, cached snapshot authority or IME delivery-route change.
Next investigate the largest shared remaining class: selected-text conversion
(26 explicit selection FAIL identities; GTK Entry focus-return additionally
ends with a selected range). Preserve first-failed-transition evidence before
editing and keep selection authority distinct from collapsed-word authority.


Reset-cache remote AST-only architecture gate PASS at2026-10-19T10:11:15.933426+00:00; original scripts/update-architecture-graph.sh (including graphify update) returned0. All six generated outputs and export archive were hash-verified before copying back; runtime sources unchanged except generated graph receipt. Exact receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/ibus-reset-surrounding-architecture-source-one/RESULT.json; SHA256 983d275226f6fe084ab4cb0c594eb02a5a32761a5ba97fb734003ffc39c8fbca. This source/document gate does not expand physical acceptance or certify a new build. Latest matching432 remains382PASS46FAIL4BLOCKED. Next selected-range diagnostics show positive IME RPC context_authority refusal with manual_toggle_allowed=false/manual_refresh_allowed=false in Qt, Firefox and Chrome; first source admission condition remains to be identified. No installation or delivery-route change.


TD159 Chrome Search four-cell passive text-reader diagnosis, independently
closed at2026-10-19T10:35:56.651714+00:00. Firefox2 roundtrip/short PASS;
Chrome2 roundtrip/short FAIL before Double Shift. Same a46dd831 Lay plus
0b907f7c private IBus and the complete kbel dependency bundle; no runtime or
input-route change. Remote60 canonical source controls and4 actual observer,
error/bounds and entrypoint-wiring tests PASS. Canonical320 actor/controller
and their guards are retained; adjacent four-cell diagnostic wraps state(),
returns the original object unchanged and adds bounded sequential reads only.

Chrome CharacterCount5/1 and caret6/2; count-bounded, count-plus-one and
end-minus-one all return "приве"/"д". Firefox equivalents are full6/2.
This falsifies the narrow hypothesis that merely extending the GetText read
boundary repairs the observation. Positive Chrome Wayland client log entries
contain complete preedit_string("привет",12,12) and ("да",4,4), so full
composition text reaches that endpoint. DOM/rendered equality and precise
Chrome accessibility/composition transition remain UNKNOWN. Next compare the
actual DOM value and current composition without changing Lay delivery.

The preceding kben partial320 attempt was stopped after observing that the
probe had been added to a helper not executed by the actual actor. That run
contains no useful probe samples and is not promoted or silently retried as a
matrix. Independent aborted closure verifies empty cgroup, no held keys,
owned seat/policy removed, VT1 and unchanged host libraries. Its export hash
is a5bdc9fc98c8a11cc08a170bf87fce894d4128e0e80f609464fb850db3cecab8.

Four-cell exact receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/search-text-reader-four-cells-source-one/DIAGNOSIS.json,
SHA256 d9da90461c1a36c0accf374fdbf119d06f4c11883943bcc1749385b622b97a8c.
Archive75f7cf84300a6f05ec0af8a179c6e0f7fe72103f71dc5feb9e18e8e72b5c0e62,
345455bytes138files16raw snapshots1940780bytes, every member verified.
Independent closure includes pinned processes gone, no held keys, stopped
owned seat, VT1 restored and host libraries unchanged. Scope4cells/428NOT_TESTED;
latest matching complete432 stays382PASS46FAIL4BLOCKED, browser292/28/0/320.
No Lay build, installation, publication, host IBus restart or authority change.


TD159 Search DOM/AX separation, closed2026-10-19T10:53:09.429600+00:00.
Owner: docs/architecture/firefox-input-stability-2026-10-05.md, same heading.
kbes adjacent four-cell Chrome diagnostic2PASS2FAIL/428NOT_TESTED. Search AX
truncates final character and reports caret beyond length; post-verdict DOM
contains full text and collapsed end caret. Ordinary input agrees and passes
both Double Shift directions/probes. Sequential45-55ms observations do not
prove an atomic AX/DOM mismatch or precise Chromium root cause. Search gesture
NOT REACHED. Do not add Lay output, extend read boundary, rename old FAIL or
promote DOM to runtime authority. Next qualify a separately versioned shared
read-only observer and its negative controls; original native keys unchanged.
Exact cache receipt search-dom-reader-four-cells-source-five/DIAGNOSIS.json,
SHA25628c561c4508c8735721026f64d9ac18c8fe85beaa0f86ac0f37c3c68373ad44a;
verified archive1a9a2f76e80ac8439a92570ee6a752de4cb830dd69c46a426dd9d08c25c4be41.
Independent closure complete. Full432 remains382PASS46FAIL4BLOCKED. No Lay
build, installation, host IBus restart, delivery-route or authority change.


TD159 fixed client-text observer qualification, closed2026-10-19T11:15:36.909223+00:00.
Owner: docs/architecture/firefox-input-stability-2026-10-05.md, same heading.
Test-only preselected read-only DOM observer: 92 source checks PASS, four fixed
Chrome ordinary/Search native cases PASS, eight Double Shift gestures with
next-letter/Backspace checks. 428 NOT_TESTED; no old FAIL reclassification.
Exact receipt client-text-observer-four-cells-source-two/DIAGNOSIS.json,
SHA256 7a041a5935478d5834dfcea0c9a91a339f960513443460ea4103c21d67aca17c; archive 019341e72d268289a28d306bd1abbaaabc50ab8d3e48dcec3442129467181ed2.
Initial source-wrapper miscount and kbet zero-key predecessor-status abort
preserved; corrected actual closure guard has negative controls. kbeu independently
closed, bytes unchanged. Full432 stays382PASS46FAIL4BLOCKED. DOM-observation
latency includes reader overhead and is not comparable to prior native timings.
Next separately versioned full affected scope; selection/terminal defects remain.
No runtime authority, delivery route, installation, restart or publication change.


TD159 full browser observer qualification, closed2026-10-19T11:29:26.292355+00:00.
Owner: docs/architecture/firefox-input-stability-2026-10-05.md:10191.
Same320 native identities: Firefox150/10/0, Chrome150/10/0; total300PASS20FAIL0BLOCKED.
Eight Search reader failures resolved by new preselected observation; other312
verdicts unchanged. All20 remaining failures have actual full selected ranges.
96 source controls PASS. Runtime bytes/routes/authority unchanged, no install.
Receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/native-client-observer-browser-source-one/DIAGNOSIS.json;
SHA2560f9c19b93047012124b353e8380d354b145c02388db6db44f652b021d9fcbe8f. Verified archive2c29957ba2749fccf3c7e857c076f7cef0089f2223be2f657049234183d90e80;
independent closure complete. Native112 not rerun; GNOME NOT_TESTED.
Successful-only latency split by reader; DOM overhead is not runtime latency.
Next KWrite post-gate adapter/pair-scope contract and selected-range authority;
existing raw snapshots prove candidate eligible but final decision absent,
not a missing Space or proven focus theft. No safety/verifier bypass.

TD159 KWrite pair-source causal red:2026-10-19T11:44:13.139026+00:00;1PASS/3FAIL through actual client-observation ingress. See docs/architecture/firefox-input-stability-2026-10-05.md and decision2026-10-19-owned-space-pair-actual-rereceipt.json. Existing pair proof carry selected; green/native pending; output routes and verifier retained. Exact receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-repro-source-three/RESULT.json.

TD159 KWrite full source and private build2026-10-19T11:55:15.439987+00:00: official IME886/886 and performance3/3 PASS; graph, embedded architecture receipt and format PASS; private IME ff544e707102459d6af7d43e26d3e57c6294b2e0134402d9877071bacebcf8dc; native NOT_TESTED. Owner docs/architecture/firefox-input-stability-2026-10-05.md; receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-final-build-source-one/RESULT.json. Delivery routes/install unchanged.

TD159 KWrite physical native112 closed2026-10-19T12:00:09.731702+00:00:
91PASS17FAIL4BLOCKED/112 on ff544e707102459d6af7d43e26d3e57c6294b2e0134402d9877071bacebcf8dc.
Only KWrite/moved_boundary changes FAIL->PASS;111 other verdicts unchanged.
KWrite14/1/1, actual target "должны быть ", Space30.692999ms one successful
observer-inclusive sample. All1227 archive members hash-verified; closure
ad64a52b68a394fb170ddbe3615a66d325b1f7724d26cc43a4e97d0085834147.
Exact receipt /home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-native-comparison-source-one.json.
Owner docs/architecture/firefox-input-stability-2026-10-05.md, same milestone.
Same-candidate browser320 pending; historical300/20/0 is not new432 acceptance.
Source-one fixture mismatch failed before input; source-two same76 controls
PASS with correctly pinned frozen new-build metadata. No install/routes change.

TD159 changed-pair browser320 closed2026-10-19T12:13:38.046328+00:00:
Firefox149/11/0, Chrome150/10/0;299PASS21FAIL0BLOCKED/320. Only
Firefox/6/moved_boundary changes PASS->FAIL. Same-pinned native112 gives
matching432=390PASS38FAIL4BLOCKED. Candidate NOT_PROMOTED; KWrite one gain
offset by Firefox one loss. Receipt
/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kwrite-pair-prefix-browser-comparison-source-one.json.
Owner docs/architecture/firefox-input-stability-2026-10-05.md, changed-pair
closed432 entry. Actual client D-Bus SetSurroundingText already contains
full12-scalar source with cursor11/anchor11; Lay preserves and refuses this
tuple. Old successful event recovered in next trace snapshot shows12/12.
Intermediate GTK observation ordering is a hypothesis; no cursor synthesis,
contradiction-retention bypass, new runtime route or installation authorized.

TD159 Gecko diagnostic closed2026-10-19T12:32:52.182390+00:00:
Firefox6 original16=15PASS1FAIL, moved_boundary PASS, diagnostic only.
Actual GTK module1.5.26-4/private libibus1.5.34 pinned separately.
Gecko log positively measures cursor12/text11 immediately after empty
composition end then noncomposing commit; existing selection retry supplies12/12.
Failed full12/cursor11 Gecko notification order remains unknown. No regrade,
authority/output-route/install change. Owner firefox-input-stability document,
bounded Gecko diagnostic entry; cache firefox-gtk-ordering-diagnostic-source-one/evidence.

TD159 next bounded hypothesis: finish existing visible active composition with
CommitText before retiring presentation; record five production-entrypoint
red/green controls including postcommit clear errors and unchanged atomic
effects. Baseline private ff544e70 matching432390/38/4 NOT_PROMOTED.
Decision2026-10-19-active-composition-commit-retirement-order.json records
alternatives and consequences. Native/source proof pending; no cursor
synthesis, pair-proof retention bypass, new owner/timer/query/route/install.


### TD159 commit-order controlled red/green; remaining repair scope

2026-10-19: active-composition output experiment detected original order in
4/5 controls (1PASS4FAIL), then passed5/5 after a local same-route output-order
change. Exact receipts `active-commit-order-{red,green}-source-one/RESULT.json`
under `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/`.
Commit failure retains original composition; postcommit clear failures retire
the sent buffer and retry presentation only. Native benefit remains NOT_TESTED.
User defers26 selected-text failures; keep them FAIL, repair remaining12
(Firefox1, GTKfocus1, Konsole10); BLOCKED4 separate. No installation/promotion.


### 2026-10-19 — accepted publication boundary for TD159

Recorded at 2026-10-19T13:27:26.885956+00:00. Both final native components are independently
closed with the same candidate:391PASS37FAIL4BLOCKED/432, no untested rows.
Firefox field6/moved_boundary recovers after commit-before-presentation
retirement; no other row changes. Original26 selected-text FAIL are retained
and deferred; GTK focus-return1, Konsole10 and four BLOCKED remain limitations.
The user accepted this result and requested publication as1.0.91, updating later
through the menu. Repair and verification stop at that acceptance boundary.
No additional code change, check, build, native input, installation or restart
is authorized by publication. Current delta is version/release/evidence metadata.
Private KDE dependencies remain a qualification limit; fresh GNOME NOT_TESTED.
Exact receipts, closure hashes and successful-operation latency are in
`tech_debt/evidence/td159-accepted-native-432-20261019.json` and the latest owning
input-stability document entry. Existing historical red/green receipts remain.


### 2026-10-19 — stock KDE cold bootstrap, actual first refusal

User reports no hint, Tab or Double Shift on stock KDE. Preserve accepted
private-dependency receipts without applying their PASS counts to stock KDE.
The first corrected wire fixture exposes descriptor misclassification; after
its repair, independent bus capture exposes bootstrap Unset without a later
same-name signal. New hypothesis: confirm actual profile once during the
existing acquisition, before its marker and within its deadline. Controlled
real-zbus RED reproduces the missing read; five new source checks pass, actual
client verdict pending. Details, incorrect worker-count receipt and next proof
are owned by the latest stock-KDE section of the input-stability document.
No local GNOME installation, global IBus restart, identity or route migration.

### 2026-10-19 — stock-client request diagnostic corrected

Same-name bootstrap passed 898 selected IME checks but actual owned Qt/GTK
suggestion and Tab remained FAIL. The external direct Engine.Enable A/B is
invalid as a Ready-path experiment: the foreign callback cancelled the
authenticated observer. Preserve that refusal; do not relax sender binding.
Next bounded experiment uses the existing current-owner post-Ready publisher
to request actual surrounding text once, never a timer or synthetic snapshot.
Record its RED/GREEN and client verdict in the owning input-stability document.
The separate Plasma Qt.labs.platform import error was repaired by installing
the missing Qt5 QML module; offscreen component creation passed. It is not
hint/Tab or native browser acceptance. Exact receipts and recovery scope are
in the owning document's stock-KDE follow-up entry.

### 2026-10-19 — stock Qt/GTK visible hint and Tab, diagnostic verdict corrected

`stock-ready-v6-screen-acceptance-20261019.json` under the same local evidence
root binds current candidate bytes and independently inspected screenshots.
One owned Qt field and one owned GTK field on private stock KDE `:71` show
`при` with a gray auxiliary popup `вет`; Tab yields exactly `привет ` and
the ordinary Space control passes. Tab-to-client samples: Qt 4.5 ms, GTK
2.5 ms, one each; no hint-latency or p95 claim.

The first shared diagnostic defect was observation: legacy suggestions use
the existing auxiliary publisher, so empty widget preedit is not absent
panel display. Preserve v4/v5/v6 raw receipts. Other corrected probe defects
were rejection of an owned GTK focus child and a Qt snapshot preceding the
asynchronous Tab commit. The earlier empty run was independently blocked
by a verified IBus popup keyboard grab. These corrections grant no context,
edit authority or production delivery route.

Source remains four focused plus 902 selected IME PASS on exact candidate
`ed48bcf70cda5b0c3059263562578957bf295457b2863c8b40189ddf09e54743`.
Full stock browser/physical Double Shift acceptance remains NOT_TESTED.
Cold-start self-SIGABRT on child 405181 remains OPEN; surviving normal child
407179 is not killed to manufacture another run. The owning input-stability
document records traces, screen hashes, measured scope and next work.


### 2026-10-20 local date — pending literal preedit, controlled Ready race

Measured native baseline: owned stock-KDE Kitty v4 consumed physical `при`
then committed `ритм ` after Tab. Candidate was
`ed48bcf70cda5b0c3059263562578957bf295457b2863c8b40189ddf09e54743`.
The first letter existed in visible legacy preedit before SourceFree Ready;
installation reset it. One 1.469275 ms Tab-to-child sample is for the wrong
result and is not accepted performance evidence.

The intermediate patch passed eight focused and 910 selected isolated IME
checks (`activation-preedit-green-source-three/RESULT.json`) but remained
unaccepted: independent review found a receive-order race. Two real adapter
rendezvous tests then failed on that patch with controlled ordering, without
sleeps (`activation-preedit-red-source-race-one/RESULT.json`). The correction
captures immutable Native acquisition identity in typed PendingLiteral ingress
and publishes it under the existing reducer->stamps lock order. Only that
exact non-authoritative stamp can survive owner replacement within the same
live connection; key and lifecycle epoch checks are unchanged. Literal
completion never calls settle(None) to revoke a successor. SourceFree scope
remains UnknownStart; no new edit/input authority or delivery executor exists.
See decision `decisions/2026-10-20-native-pending-literal-preedit.json`.

Ten focused controls PASS in the final candidate's FOCUSED.log; its official
isolated IME gate/build is RUNNING in
`activation-preedit-green-source-race-two`. Native verification of those bytes
is NOT_TESTED. Preserve both earlier fixture failures as fixture evidence.
Evidence root: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018`.
A separate old-Kitty launch prerequisite (`GLFW_IM_MODULE=ibus`) was added only
to the owned diagnostic child; the user's existing Kitty and text remain
untouched. The independent cold-start SIGABRT and full stock browser matrix
remain open/NOT_TESTED respectively.

Harness2 task `20261019T210733Z-1565f93b`, lay2/gpt-6.1-sol/xhigh, completed
with status/core_status ok, eight calls and no failures/fallbacks. All 12 frozen
input pins and output/report hashes are independently checked in
`harness2-lay2-kitty-review-20261020/acceptance.json`. Its diagnosis is bounded
static evidence; the input predates this patch, and no speed/token benefit or
experience-library growth is demonstrated.


### 2026-10-20 local date — warm ordinary FocusIn before context reply

The earlier ten-case source gate is complete: 10 focused and 912 selected
isolated IME checks passed, release SHA-256
`6687b7f4ff486f21a364f68cb87c0170994362557d855b63206400109f7542cb`.
However, owned Kitty v5 on those exact bytes still LOST the initial `п`
from physical `при`, showed `ри` with auxiliary `тм`, and Tab committed
`ритм `. Retain this native FAIL and wrong-result timing; it is not a green
client receipt. See `kitty-native-v5-screen-acceptance-20261020.json` and
`kitty-native-diagnostic-v5-20261020-trace-delta.json` under the evidence root.

The omitted transition is a warm FocusOut/FocusIn on the same engine path.
The reducer retains the sealed source owner while its tentative Transfer
awaits the target identity. Ordinary FocusIn uses CompatibilityProperty;
when the CurrentInputContext reply names a different context, the canonical
reducer converts that exact request/nonce to SourceFree and advances the
lifecycle once. The earlier receipt checked only ownerless Native SourceFree,
so the first target letter was stamped as an old-owner key and then cleared.

The pending-target receipt now precedes the retained-owner branch for legacy
possible-mutation callbacks. It binds the exact request, nonce, target, origin
and optional context hint. For a tentative Transfer it expects the single
canonical lifecycle increment, and preservation STILL requires a current
SourceFree grant for the identical Ready request. Other revocations remove
or replace that identity. Transfer grants, reset, sensitive/unsupported input,
focus replacement and unproved preedit do not qualify. Shift retains its
existing path. Output owners, delivery routes and edit authority are unchanged.

`activation-preedit-red-source-race-three/FOCUSED.log` contains ONE valid warm
CompatibilityProperty lost-letter RED plus ONE fixture error: the new Native
FocusInId fixture expected a Get although that route already has its context
and emits a Barrier. Do not describe both failures as lost-letter proof.
`activation-preedit-green-source-race-three/RESULT.json` passed 12/13 and
retained that same fixture error. The fixture now holds the actual Native
marker; ordinary FocusIn continues to hold its real property reply. The new
13-case gate and official isolated IME/build are complete in
`activation-preedit-green-source-race-four/RESULT.json`: 13 focused PASS,
915/915 selected IME PASS; 918 discovered, three performance exclusions.
Release SHA-256 is
`955fdd5070bb83a1eec30ccf6d875bfff2e5c4b7faa6abe7dd1cfad09e8bd134`.

Normal private IBus factory activation runs those bytes as child 746568 of
3700659. A backup of its predecessor was verified; no global IBus/Plasma
restart, local-desktop installation, delivery-route or authority change occurred.
Owned Kitty v6 now shows all of `при` and gray auxiliary `вет`; physical Tab
produces exactly one `привет ` commit. The trace places PendingLiteral ingress
before SourceFree Ready. Independent source/screen/client acceptance is in
`kitty-native-v6-screen-acceptance-20261020.json`, with the raw v6 diagnostic
and PNG retained. Observed Tab-to-commit time: 1.200848 ms, one sample only.

Owned Qt and GTK on the identical bytes show `при` and gray auxiliary `вет`;
Tab and the separate clean Space control produce actual `привет ` in both
fields. `stock-ready-preedit-v1-screen-acceptance-20261020.json` checks both
screenshots and actual field events. Qt Tab time is 4.9 ms, GTK 3.1 ms,
one sample each. No median, p95 or hint-latency claim follows. The raw reader
only checks preedit, so it retains BLOCKED for auxiliary hints; the independent
screen receipt supplies the missing observation without rewriting raw evidence.
Source/debug settings and prior focus were restored. User Kitty/text remain
unchanged; GLFW_IM_MODULE=ibus was set only for the owned Kitty child.
Full stock browser matrix and physical Double Shift are NOT_TESTED for these
bytes; the separate historical cold-start self-SIGABRT remains OPEN.

At the user's explicit request, an additional real Harness2 review completed:
`20261019T215217Z-9876d10b`, project lay2, gpt-6.1-sol/xhigh. Status and core
status are ok; all eight calls succeeded, with zero failed calls or fallback
stages. The eight frozen input pins (94,528 bytes) remained intact. Selected
c1 review is 7/10: no demonstrated blocker, with significant integration
evidence missing from its excerpts. Independent acceptance is recorded in
`harness2-lay2-warm-preedit-review-20261020/acceptance.json`, binding report,
review and owner checks. The owner checked the omitted outer publication guard,
reducer lock, Ready replace_owner and actual observer-driven rendezvous tests;
the source gate and physical-client receipts supply their own separate proof.
The snapshot predates the Native fixture correction and incorrectly calls both
old failures semantic RED; acceptance explicitly corrects that premise. Inputs
were not rewritten. Library entries remain zero; acceleration and token savings
are unproven. Full stock browser/physical manual-transfer coverage and mixed
CompatibilityProperty-to-Native enrichment after early literal input were not
exercised by this bounded repair. No additional runtime change follows from
an excerpt omission or review score.

Guarded source-graph refresh PASS at 2026-10-19T22:06:15Z:
`activation-preedit-architecture-source-one/RESULT.json`. All six installed
graph artifacts match the receipt; 730 non-generated source pins still match
the source gate. This later evidence-only note changes no executable source
and establishes no additional runtime or native acceptance.

### 2026-10-20 — short Konsole presentation iteration

User accepts Kitty and takes over manual client testing. Hypothesis: canonical terminal qualification currently excludes stock Konsole caps41, forcing owned preedit which Konsole inverts. Reuse terminal delivery with exact current field and canonical KWin app/class; no fabricated SSI, word boundary or edit ACK. Separately retain only the existing pending acquisition geometry across exact Ready installation. Baseline runtime 9e751d3c remains installed until short source checks pass. Two valid geometry REDs are in run-3b4t73j7; third failed fixture is corrected to adapter revoke_current_owner.

Short verdict: `konsole-fast-v2` executes the exact 24 manifest-discovered
correctness identities and passes 24/24 in 0.369884953 s (execution only).
First v1 is retained at 23/24: its new terminal fixture did not install the
pending activation before requiring a live receipt. v2 fixes only that fixture.
Candidate release build passes in 193.955756 s; engine SHA-256
`709c6d8845298642e9f87bd3cd05eff1820952a4c0d6ae93c9e8a2231493f196`.
All 1254 source pins are unchanged and match the owning checkout. Receipts:
`/home/e/projects/lay-development-runner/kde-install-control-20261020/konsole-fast-v2/`.
The owning presentation document records effects, guards and untested scope;
manual Konsole acceptance and changed-byte browser coverage remain NOT_TESTED.
No full matrix or universal PASS is claimed.

Follow-up within the same bounded iteration: Harness2 base1 review
`20261020T172003Z-f3051d4a` identifies the late SurroundingText override.
`konsole-fast-v3-red` proves it with 25/26 controls passing and exactly the
new native-mode assertion failing. Retain native mode on that transition only
while the existing `uses_native_terminal_input()` is still true. No existing
edit/snapshot exclusion changes. `konsole-fast-v4` passes all 26 discovered
controls in 0.496244297 s execution; late exact-contract fallback also passes.
Final build pins and generated graph metadata are recorded independently in
`konsole-final-build-v1/result.json`; startup-only installation, when done, is
`konsole-engine-deployment-v1/result.json`. The 24-test candidate was never
installed. Browser totals and physical-client verdicts do not change.

### 2026-10-21 19:43 UTC — replay-orphan-v26

First failed transition is an inert native-replay veto surviving revocation
of its word owner. Chrome Reset/FO/FI is expanded by stock IBus1.5.26 into two
engine focus cycles without X11 window focus loss. The first SSI-backed fixture
was noncausal; the corrected no-surrounding stock-client fixture is RED at the
first remaining Backspace. Retire only the identical path/engine-lease marker;
do not retain or restore the old word owner. Causal GREEN plus 13/13 isolated
guards pass. Foreign path/lease markers stay intact, and old tokens stay revoked.
Native replacement and rendering with changed bytes are still NOT_TESTED.
See owning preflight section `replay-orphan-v26` and its retained v24/v25/v26
receipts. No delivery route, verifier or runtime authority changed. No installed
candidate or client acceptance is implied at this checkpoint.

## 2026-10-21T20:05:39.601697+00:00 — v26 native partial; v29 causal pending-literal repair

Remote development engine709815/SHA2dd9972b23ec0f555cc9965f9bb18320e76fa4940fcd7b61c11cb888283cd349 was installed19:48UTC; daemon49153, IBus, tray, configuration and user windows preserved. V26 native Chrome omnibox: heldShift4/4, hintTab, seven ordinaryBackspaces, active-word roundtrip and active-five PASS. After-Space first replacement now PASS; inverse and Space-five FAIL. This is seven bounded rows, not the full browser matrix. Chrome prefix underline remains visualFAIL.

V27/v28 independently paired private-marker wire/observer traces: observer processing follows wire receipt within approximately0.1ms; delay precedes wire delivery. V27 has approximately100ms marker delay beyond the unchanged50ms acquisition deadline. V28 last acquisition finishes within its deadline, but its first g/h arrive with authenticated pending_literal stamps, are committed rather than kept in preedit, and are then lost from the local mirror at Ready. Native text is whole; the next manual request owns only its suffix. Delayed versus incomplete acquisition are separate remaining mechanisms.

V29 causal source RED holds the real marker after a real context reply, sets caps41 with no SSI and enabled IME. The first letter is wrongly committed (composition empty vs p). Minimal repair removes only the live-token condition from literal-preedit eligibility; the existing request/nonce/revocation/focus receipt still governs preservation at Ready. No boundary, client-text authority, delivery route or timeout is changed. Initial isolated affected gate17/17PASS; added explicit no-CommitText/no-delete assertions are being checked before build. New-byte v29 native NOT_TESTED. Harness2 task20261021T200308Z-22f7cc91 is running; no approval inferred.

V26 Harness2 task20261021T193939Z-13826323 completed8/8calls, no failure/fallback, bounded review9/10. Independently inspected marker identity cleanup and13source checks. Frozen review covered the positive fixture; later foreign-path/lease controls were independently tested outside that snapshot. Native v26 is partial, not full acceptance. Receipts: replay-orphan-build-v26/result.json, replay-orphan-deployment-v26/result.json, replay-orphan-native-v26/chrome/result.json and replay-orphan-native-v28/chrome-wire-trace/{result.json,engine-trace.jsonl,ibus-engine.log} under the remote kde-install-control-20261020 directory. Local causal logs pending-stock-literal-{red,green}-v29.log under shift-hint-20261021T1242Z. Full historical browser100PASS/76FAIL/144NOT_TESTED of320 remains unchanged. No release, push, local install or global restart.

## 2026-10-21T20:22:19.503452+00:00 — pending literal v29 native result

Remote development engine789121/SHA1477678ecceff657b788c18cad1158b7d4b183578e6da7dde3d3401ca9a04e6f installed at2026-10-21T20:12:20.184737Z. Daemon49153, stock IBus, tray, configuration and user windows preserved. Source gate17/17PASS, compiler artifact resolved from Cargo JSON; source closure1258files/SHA c921a2880b257dedc776f139016686d7b86f7202a551e991c77232db0d17593c. Remote receipts pending-stock-literal-build-v29b/result.json and pending-stock-literal-deployment-v29/result.json. No release/push/local install.

Native Chrome v29 seven-row probe:5PASS/2FAIL. Held Shift RU/US left/right4/4, hint/Tab, seven ordinary Backspaces, active-word Double Shift roundtrip and active-five PASS. After-Space roundtrip and after-Space-five FAIL. Active observed target timings12.275–12.669ms; a successful Space target251.699ms. This small probe establishes neither an overall latency percentile nor full browser acceptance. Receipt pending-stock-literal-native-v29/chrome/result.json.

V30 independent private-bus trace identifies the next first failure: acquisition131/nonce168 Get starts1792613610246622us, marker emitted0248318 and received0288342 (~40.024ms, inside the unchanged50ms deadline). All six replacement letters remain in local preedit after v29, then Space commits the whole word before Ready. Ready clears the committed local mirror; the inverse manual request sees tail0. No X11 focus change to an IBus window occurs. Receipt pending-stock-literal-native-v30/chrome-wire-trace/{result.json,engine-trace.jsonl,ibus-engine.log,x11-focus.log}. Debug configuration restored; engine bytes unchanged. A separate earlier acquisition exceeded50ms; increasing the deadline alone cannot repair this within-budget loss.

Harness2USED task20261021T200308Z-22f7cc91, gpt-6.1-sol/xhigh,8/8callsOK, no failure/fallback,9/10 bounded static review. Independently inspected report, selected artifact and current receipt fences; later no-CommitText/no-delete assertions separately passed17/17 outside frozen snapshot. Independent acceptance under local shift-hint-20261021T1242Z/pending-stock-literal-review-v29/independent-acceptance.json. Native remains partial FAIL despite static approval.

Next unresolved mechanism: preserve only new literal input consumed within the exact pending acquisition across a closing Space; do not resurrect revoked owner/client text, add a delivery route, fake a boundary, or increase timeout as a substitute. Production patch not yet selected. Chrome prefix underline visualFAIL and Firefox iframe overlap visualFAIL remain. Full historical browser100PASS/76FAIL/144NOT_TESTED of320 unchanged; no new GNOME or full matrix acceptance.


## 2026-10-21T20:42:40.139736+00:00 — pending literal Space v31

Context-admission source gate461/461PASS; remote engine878924 SHA716232d39e3b419982e0497c1e0702d670c691485ff3ac428e70815827fa2cda. Original native Chrome5PASS/2FAIL, still after-Space manual failures; no overall acceptance. Debug-enabled v32 single Space roundtrip PASS is timing-altered evidence only. Next: passive no-debug v33 wire failure, then affected native Firefox/ordinary Kitty/Konsole. Protected10 processes/config unchanged. Harness2 task20261021T203236Z-62aa6552 running. Owning record: docs/architecture/ime-inline-hint-preflight-2026-10-20.md CURRENT; receipts pending-literal-space-build-v31, pending-literal-space-deployment-v31, pending-literal-space-native-v31. No release/push/local install; retain prior fixes and active sessions.

## 2026-10-21T21:22:21.234437+00:00 — v35 current continuation

Remote enginePID1015535/SHA1ab291b; 468/468 source PASS. Native Chrome omnibox5PASS/2FAIL after Space; short Firefox21/28 and Chrome21/28 PASS. Kitty/Konsole hint/Tab and active/Space five-toggle PASS. Ordinary uppercase Shift PASS in all four; RU punctuation oracle unverified. Browser visual FAIL. No daemon/IBus restart. Next: full passive IBus wire around failed inverse; preserve exact owner/reducer/WordScope contracts. See owning preflight entry and v35 receipts. H2v34 completed degraded, selected6/10; eviction defect fixed in v35, final follow-up pending. Historical full matrix unchanged100PASS/76FAIL/144NOT_TESTED.


## 2026-10-21T22:22:46.914117+00:00 — v39 native delivery preserved; inverse and pixels still FAIL

Installed remote engine PID1208224 SHA f36012e4874026f390f814dac563de2802a3a1226c9651be1530f281a3c68bce. Guarded source gate450/450PASS. No daemon/IBus/tray restart;10 protected processes/config unchanged. Existing descriptor delivery ordinal + explicit delivery lease retain native replacement across real Reset/refocus; old word token remains revoked. Omnibox first after-Space target now complete,233.512ms; inverse stillFAIL. Short suites: omnibox5PASS/2FAIL; Firefox21/7; Chrome20/8. Owned ordinary Kitty/Konsole hint,Tab,5xactive,5xSpace and four Shift uppercase casesPASS; Russian punctuation oracle needs real no-IME control. Old user Kitty without GLFW_IM_MODULE preserved,not certified. ScreenshotsFAIL: Chrome typed prefix underline; Firefox iframe prefix missing visually. Full320 matrixNOT_RERUN. Harness2USED20261021T214419Z-cb269f86,8calls0failed/fallback; accepted frozen pre-v39 design only,not patch/native. Owning record: docs/architecture/ime-inline-hint-preflight-2026-10-20.md CURRENT; receipts remote pending-literal-space-{build,deployment,native}-v39; local native-replay-reset-design-v38/independent-acceptance.json. Next: true no-IME punctuation control, fresh exact manual source/settlement failure, and coherent typed-prefix presentation. No push/release/local install.

## 2026-10-21T22:46:10.614072+00:00 — plain-prefix scope and Shift oracle

User accepts Kitty only; Chrome typed-prefix underline and LibreOffice presentation remain open. v39 engine f36012e4 remains unchanged. v41 physical-keycode Shift tests6/6 in each of new Kitty/Konsole windows; old RU punctuation mismatch also occurs in detached no-IME Kitty control, so old RFB oracle is invalid for attributing it. v42 existing-profile run BLOCKED with zero input because old Chrome/Kitty processes are gone. Do not retry stale PIDs. LibreOffice1285267 is GTK3 VCL. Matching Chrome155 primary source shows NONE/default native composition underlines and missing surrounding retrieval; neither removing attrs alone nor reintroducing unaccepted hints into preedit is a safe fix. Native actual-source probe v43 is in progress; Harness2 design task20261021T224439Z-58fdfa4b running (not accepted yet). Decision2026-10-21-plain-prefix-observation-probe.json permits diagnosis only. Do not implement a new owner or fabricate SSI. Owning document: docs/architecture/ime-inline-hint-preflight-2026-10-20.md.

## 2026-10-21T23:04:20.091127+00:00 — LibreOffice actual-surrounding prefix candidate

v39 f36012e4 remains installed; native LO v49 Tab PASS but prefix underline FAIL, with10 actual surrounding callbacks and7 restorations. Chrome v43–v46 accessibility source qualification BLOCKED before text input; no provider adopted. Source regression stock-observed-v50-red confirms actual stock boundary unnecessarily starts owned preedit. Minimal candidate uses existing ordinary ManagedCommit for that observed stock branch, keeps explicit exact-refresh/opaque paths and all freshness/verifier guards. Source/native checks pending. Kitty acceptance retained. See owning preflight CURRENT and decision2026-10-21-stock-observed-ordinary-prefix.json.

## 2026-10-21T23:17:43.884723+00:00: observed stock prefix fixture alignment

Hypothesis: stock clients with actual SSI can use existing ManagedCommit without native prefix decoration. Installed v39 unchanged. Causal RED v50 retained; affected source 430 PASS/21 FAIL/451. First shared failure is old owned-word fixture setup, not output guards. Preserve stock owned-mode proof via word-before-first-SSI order, keep exact-refresh owned mutation controls, add ordinary mutation stale-receipt rejection. No verifier or freshness weakening. Chrome v51 actual AT-SPI source PASS is diagnostic only; zero IBus SSI, no provider or executor adopted. Owning decision: `2026-10-21-stock-observed-ordinary-prefix.json`; candidate native acceptance pending.

## 2026-10-21T23:29:11.969895+00:00 — stock observed ordinary prefix v55 source gate

Baseline: v39 installed, real LO v49 typed-prefix underline FAIL with ten actual
SSI callbacks; Chrome v51 actual AT-SPI source qualified but zero IBus SSI.
Hypothesis: reuse existing ManagedCommit for observed stock boundary, retaining
opaque and explicit exact-refresh ownership. First failed transition was
`should_start_legacy_word_preedit` taking observed stock boundary as owned.
Single conjunct candidate now causal 26/26 and complete affected 452/452 PASS
(remote stock-observed-v55-causal/green result.json); v50 old-policy RED kept.
New ordinary insertion test preserves real Reset/no-Reset provenance and proves
no stale receipt reuse or document edit. Owned tests establish actual delayed
SSI or exact-refresh mode, with all original effect/refusal assertions retained.
Runtime v39 unchanged. Guarded build and Harness2 frozen review
20261021T232546Z-71dfa570 running; no client acceptance promotion.

## 2026-10-21T23:40:13.096653+00:00 — native v55 rejects the boundary-before-key hypothesis

Installed v55 b2b109b1; LO functional5PASS/1FAIL (closed Double Shift) and prefix presentationFAIL in both positions. First Engine SSI follows the first key; empty InputContext SSI is not forwarded. v55 predicate cannot affect the already owned first word; absence after a boundary also restarts owned mode. New hypothesis uses the existing literal commit at a matching real preedit projection, then lifetime callback capability prevents mistaken re-entry. No receipt/authority fabricated; all old source/native failures retained. Exact preflight includes alternatives, consequences and new causal proof plan. See owning CURRENT entry and remote libreoffice-presentation-probe-v55/result.json.

## Checkpoint — 2026-10-21T23:58:21.804873+00:00: v56 affected source457/457 PASS; remote build/native pending

Remote stock-observed-v56-verified/result.json:457 PASS,0 FAIL; discovery12.991s, execution93.537s. Earlier stock-observed-v56-full/result.json remains456 PASS/1 FAIL: the Reset freshness fixture required persistent owned preedit while declaring stock-client mode. It now explicitly establishes exact-refresh capability before real initial input. All Reset, fresh-read, duplicate and edit-refusal assertions remain; separate actual stock callback and revoked-owner tests prove literal transfer. The next discovery failure was a missing explicit capability-constant import; it is fixed, with no guard weakened. Exact source closure is pinned by the passing receipt.

Harness2 task20261021T235242Z-f11aa21c is running; completed base1 stage8/10 finds no demonstrated blocker in the frozen bounded design. Independently checked plain commit, live WordScope/token matching, generation cancellation, snapshot clearing and freshness floors. Clarification: the helper requests no layout switch; ActiveCompositionCommit::plain retains the existing executor's profile/layout reconciliation when required. Do not claim it performs no possible layout operation. Review is source-only; no native acceptance presumed.

Installed v55 engine1460682/b2b109b1 unchanged. Next: guarded remote graph/build, source-pinned engine-only deployment, then actual LibreOffice first/second-word screenshots, hint/Tab, Shift, active/closed Double Shift and Space; short ordinary Kitty and Chrome controls. Preserve user apps, daemon/tray/global IBus/desktop, existing delivery/authority contracts and historical browser counts100PASS/76FAIL/144NOT_TESTED of320. Chrome ordinary-prefix issue remains open.

## 2026-10-22T00:16:59.030581+00:00 — v56 installed, actual native proof remains partial

Installed remote engine PID1592735 SHA256 `6d51bef57c15490dc300cbfa8f374d0fe20b02cf7a8ee31d37c3e65bcd4f3b08` at2026-10-22T00:06:21.817892UTC, engine-only. Remote `stock-observed-v56-postgraph/result.json` pins457/457 source PASS after the generated graph receipt changed; initial failed metadata reconstruction is retained, not excluded. `stock-observed-v56-build-r2/result.json` identifies the compiler artifact (81.48s); `stock-observed-v56-deployment/result.json` verifies eight protected processes, config and tray unchanged. No daemon/global IBus/desktop restart, publication or local installation.

Harness2 USED: projectlay2, gpt-6.1-sol/xhigh, task20261021T235242Z-f11aa21c, complete statusok,8 calls/0 failed/0 fallback, selectedc1 bounded8/10. All10 frozen input hashes verified unchanged; existing plain-commit, ownership, cancellation and freshness implementations independently inspected. Local receipt `shift-hint-20261021T1242Z/stock-observed-v56-review/independent-acceptance.json`. Source review is not client acceptance.

Initial v56 native outputs (six blank LO rows, failed controls, zero Engine.ProcessKeyEvent) are setup-blocked, not six proved product defects. A screenshot contained the Lay popup, but its later X11 state was IsUnMapped; a keyboard grab was not established. The qualified `libreoffice-presentation-probe-v57/result.json` adds a native click inside its owned editable paragraph and requires ordinary `a` before running cases. On the same v56 bytes:4 PASS (first hint/Tab, active Double Shift roundtrip, physical Shift+A, Space auto-flip),2 FAIL (second-word hint absent, after-Space Double Shift unchanged). Passive wire now shows352 ProcessKeyEvent,106 SetSurroundingText; all seven restoration checks true. First-word screenshot shows the literal prefix outside the IME underline; general visual acceptance remains partial, including brief initial preedit and editor spellchecking. The click also exposed Writer's header guide; do not confuse it with IME styling.

Qualified controls `stock-observed-v57-native-controls/summary.json`: ordinary desktop-launch Kitty hint/Tab and closed Double Shift roundtrip PASS, Chrome text-field Tab1/1 PASS; six restoration checks true. Chrome ordinary-prefix underline remains OPEN: functional Tab PASS does not certify styling. Two measured active LO toggles160.747/175.966ms, no p95 claim from two samples. No full browser matrix or GNOME rerun; historical100PASS/76FAIL/144NOT_TESTED of320 unchanged. Next: identify the first shared fresh-source/word-boundary loss behind LO's two remaining failures and finish Chrome prefix styling without adding ownership or weakening the verifier.


## 2026-10-22T09:34:30.248627+00:00 — v65 native readback hypothesis failed

- Source affected gate466/466 PASS; exact build SHA256 `092198d7d7705e572ba7b259b35d57888e24ed2caabecba2a870e8070b884528`. Harness2 `20261022T090301Z-6e694fc2`: ok,8calls,0failed,no fallback; independent source review9/10, native not certified. Canonical-format-only testfile equivalence is explicitly retained in the independent review packet.
- Installed only the remote engine PID3353802; daemon/tray/global IBus/desktop unchanged. Deployment receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/stock-replay-v65-deployment/result.json`. No publication/local installation.
- Actual owned Writer6rows:5PASS/1FAIL. Closed-word first toggle149.216ms PASS, inverse FAIL. Active toggles133.397/136.580ms. Short controls: Kitty hint/Tab PASS, Konsole hint/Tab PASS, Chrome Tab PASS, Firefox Space roundtrip FAIL. Visual acceptance remains separate; no full browser matrix rerun.
- Writer bus trace proves v65 branch ran: final Space CommitText at relative592.813ms, client delivery593.638ms, followed by actual SSI `ghbdtn` caret6 at595.127ms. No final matching `ghbdtn ` SSI or ACK. The proposal that this commit representation forces final fresh SSI is rejected natively. Source PASS cannot override this result.
- Native receipts `/home/e/projects/lay-development-runner/kde-install-control-20261020/libreoffice-no-capitalization-v65/result.json` and `stock-replay-v65-native-controls/summary.json` beside it. All recorded restoration checks true.
- Next: one read-only GTK surrounding trace in a newly owned Writer window to distinguish Writer getter data from IBus caching. No extra retries, timer, fabricated SSI, owner or client-specific permission. v65 remains installed pending that observation. Chrome typed-prefix underline remains visual FAIL; no Chrome production change adopted.
- Read-only upstream review at2026-10-22T09:17:25.177798+00:00:5open PRs #64,#65,#66,#67,#68; no merge/comment/closure.


## 2026-10-22T09:38:13.025113+00:00 — v65 rejected for native ordering; bounded rollback

One owned Writer GTK trace reproduced `ghbdt n` instead of `ghbdtn ` on the first closed-word toggle. The last native letter was still pending when v65 CommitText Space reached the client; final-key hybrid delivery is not ordered with earlier GTK native fallthrough. This is a native rejection of v65 despite466 source PASS and9/10 static review. Receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/libreoffice-no-capitalization-v66-gtk-trace/result.json`, actual GTK source log beside it;94 getter observations, no fake SSI. Probe was read-only and loaded only in its own Writer process; all7 restoration checks true. Rejected mechanism: do not retry final-key CommitText, extend timers or loosen ACK. Restore exact saved v64 engine SHA256 `9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0`, engine-only, preserve desktop/daemon/tray. Then same one-row read-only probe on v64 to locate original missing boundary. This is an explicit causal rejection, not an undocumented revert.


## 2026-10-22T10:06:07.869874+00:00 — Writer source clipping established; rejected replay removed

Exact v64 engine restored at 09:38:15.076429 UTC, PID3402648, SHA256 `9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0`. Nine protected processes and user config unchanged; engine-only rollback receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/stock-replay-v65-rollback-native-ordering/result.json`. The rejected22production lines are removed locally; test cleanup preserves the original native final-key order.

Read-only owned Writer v67 on v64: first closed-word toggle PASS145.359ms; inverse FAIL. All7 restoration checks true. Native document contains `ghbdtn ` at caret7, but98 actual `gtk_im_context_set_surrounding` calls return only `ghbdtn` at caret6 through the inverse gesture. Installed-matching LibreOffice7.3.7.2 source locates the cause: Writer `GetSurroundingText` uses sentence boundaries (edtwin.cxx6428–6459), cursor is separately calculated (6461–6499), VCL clamps it to string length (winproc.cxx2257–2300), GTK forwards it unchanged (gtkframe.cxx5699–5718). This is stronger than a timing-only hypothesis. Receipts `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/libreoffice-no-capitalization-v67-v64-gtk-trace/`; primary source copies `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/lo-7.3.7.2-sw_source_uibase_docvw_edtwin.cxx` and `lo-7.3.7.2-vcl_source_window_winproc.cxx`. No fabricated SSI/ACK, producer bypass or new authority.

Restore source gate v69:465PASS/1FAIL in newly added regression expectation for expired replay. Its final key may be consumed with no engine output; source policy unchanged. Native-order regression requires pass-through for current provenance and zero CommitText/DeleteSurroundingText/ForwardKeyEvent for every tested invalidation, no ACK and no invalid final provenance. Existing negative receipt tests retained. Full affected466proof pending after correction.

Harness2 task `20261022T094446Z-6a30e4b1`: base-failed, one240.115s timeout, no deliverable, not an accepted review. Narrower17043byte frozen packet rerun as `20261022T100101Z-fca5bf95`, lay2/gpt-6.1-sol/xhigh, per-run config600s, shared settings unchanged. Review is RUNNING, no result claimed. Next decision: whether a bounded actual full-field source can bind focused native field to current IBus owner without inventing acknowledgement. Chrome typed-prefix underline remains visual FAIL; Kitty accepted behavior preserved. Full browser matrix unchanged and not rerun. No push, release or local installation.


## 2026-10-22T10:51:05.807040+00:00 — restored source green; actual-word ordering candidate under review

Restored-source v73:466/466PASS, architecturePASS; receipt `shift-hint-20261021T1242Z/stock-replay-v73-source-result.json`. This closes the source regression expectation, not native Writer inverse FAIL. v70 Harness2 task20261022T100101Z-fca5bf95 accepted diagnosis-only, full-field adapter blocked. New hypothesis: the existing paired replay holds its final synthetic Space until actual whole-word SSI; no ACK/learning or second owner. GTK primary source requests surrounding after a native character commit; existing v67 getter first sees wholeword12.467ms after final engine Space. Native held-Space behavior and asynchronous reducer settlement unproved. v74 H2 task20261022T103628Z-e7cc3589 RUNNING. Exactv64/current4source hashes,9protected processes and config verified unchanged10:50:24UTC. Full findings and receipt paths: owning `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`, same timestamp.


### 2026-10-22T11:34:49.287336+00:00 — held replay boundary v74–v78


- Harness2 USED task `20261022T103628Z-e7cc3589`, lay2 / gpt-6.1-sol / xhigh: ok, 8 calls, 0 failed, no fallback, score8/10. Independently matched frozen/current inputs. Verdict was conditional source-contract eligibility and REJECT_AS_NATIVE_FIX_ON_CURRENT_EVIDENCE; it did not review a completed implementation. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-causal-boundary-v74-review/acceptance.json`.
- v75 client instrumentation did not apply. v76 observed an owned-client-held Space and actual whole word; raw assertion rejected normal outer/inner GTK contexts. Independent audit accepted only that client prerequisite, not engine hold or an IME fix. Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/libreoffice-no-capitalization-v76-client-space-hold/independent-audit.json`.
- v77 unadopted engine diagnostic consumed only the exact plan's final Space and requested surrounding once. Actual Writer document/getter returned `ghbdtn`, caret6; whole-word getter followed last native letter dispatch by2.118311ms. Engine observation retained current paired scope and lexical token. No Space/source settlement, inverse toggle, result ACK or learning was implemented. Scope: ENGINE_HELD_BOUNDARY_ACTUAL_WORD_PREREQUISITE_OBSERVED only. Receipts `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-held-boundary-v77/independent-audit.json` and `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-held-boundary-v77-native/result.json`.
- Diagnostic candidate SHA256 `a1ffb364a37a43fe883b335a2654ea5a99a2a534a796f973bee2c12efde18b9b`; all6 temporary source files restored after compile. At2026-10-22T11:21:35.794823+00:00 engine-only rollback restored installed v64 SHA256 `9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0`, PID3719097. All9 protected processes, config, activation/stable bytes, engine selection, layout and focus restored/unchanged. No local install, daemon/global IBus/tray/desktop restart or publication.
- v78 two real callback-path tests compile and are RED at final-Space consumption:0PASS/2FAIL expected. They require actual whole-word source before exactly one local Space, no duplicate output, no ACK/visible-result/learning, and cancellation before late matching SSI. Remote receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/writer-held-boundary-v78-red/result.json`. Last green source baseline remains v73 466/466.
- Next: explicit existing-reducer observed-boundary transition, current paired replay scope and lexical successor token; keep ordinary settle_bridge_output lineage guard unchanged. New whole-word source is not a result receipt. Implement and rerun narrow tests before bounded native acceptance and Harness2 code review.
- Accepted-runtime client verdict remains Writer5PASS/1FAIL (inverse after Space); Kitty controls PASS; Chrome Tab functional PASS, typed-prefix underline visual FAIL. Browser matrix unchanged/not rerun: Firefox92PASS/68FAIL/0BLOCKED/0NOT_TESTED of160; Chrome8PASS/8FAIL/0BLOCKED/144NOT_TESTED of160. Runtime authority unchanged by prerequisite evidence.


## 2026-10-22T12:18:05.261283+00:00 — Writer held boundary current checkpoint

v64 runtime unchanged PID3719097/SHA9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0. v85 token and v87 field-identity TDD fixes implemented; current source provenance restricts held Space to ObservedWordAndLocalSpace, exactly one trailing boundary. Existing full-snapshot native delivery restored; v88b20/22 with only two expected locally sourced test migrations. v91 migrated cancellation/identity test PASS. Affected470 gate v92 RUNNING. H2 review20261022T115225Z-f7d6b553 RUNNING on earlier v83 packet; final refinements require independent review. Next graph/source linkage, compiler-artifact build and short Writer7-row native test. Full evidence/limits in owning preflight document at the same timestamp. No new installed/client PASS, push or local installation.


### 2026-10-22T12:20:01.188559+00:00 — affected source gate green

Writer held-boundary v92:470/470 affected source checks PASS,82.963s; source receipt remote `/home/e/projects/lay-development-runner/kde-install-control-20261020/writer-held-boundary-v92-affected/result.json`. All16 full-snapshot native cases remain unchanged. Final actual-word callback now emits only one retained Space, no result ACK, including paired/provenance/token/field/cancellation controls. Native Writer remains NOT_TESTED for these bytes. Next generated graph/architecture refresh records all source-file hashes and may change only `src/generated/architecture_graph_receipt.json`; build must link this receipt explicitly to the passed proof, not claim identical source hashes.


### 2026-10-22T12:30:17.946391+00:00 — Writer v92: bounded native acceptance, review follow-up open

- Remote diagnostic engine `457c02b4f9707a85415138ac550574720d49a424dae9e6b0707727198946100b`, linked to source closure `75d6d4071f38be83503072d54059396ba6c76723c764aa7f17bfbefc16a4dab0`: affected source 470/470 PASS. Graph link changes only the generated architecture receipt.
- Actual owned remote KDE Writer, isolated no-capitalization profile: 7/7 PASS (first/second-word hint+Tab, active/closed Double Shift roundtrip, five consecutive closed flips, ordinary held Shift, Space autoflip). All nine conversions assert exact document text, caret at end, no selection and owned focus. Median 144.966 ms, maximum 257.007 ms, n=9, from second Shift release to observed target. This is not a full Writer visual/client-profile proof.
- Native receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-held-boundary-v92-native/result.json`; detailed client: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/libreoffice-no-capitalization-v92-held-boundary/result.json`. Both client 7/7 and runner 9/9 restoration checks true. Previous v64 bytes restored at 2026-10-22T12:25:48.867505+00:00, engine PID3926674; daemon/tray/IBus/desktop/Kitty preserved. Runtime authority and installed release unchanged; no publication.
- Harness2 implementation review task `20261022T115225Z-f7d6b553` still running on frozen v83. Earlier token/field findings already addressed before v92. New review hypothesis: a complete-word SSI with a caret one character behind may survive generic inert-prefix tolerance while a Space is actionable. Next: reproduce this exact cancellation case in a source test before a minimal fix; do not weaken generic replay guards. Current v92 native proof remains scoped to its exact tested bytes.
- Browser table remains Firefox92/68/0/0 of160; Chrome8/8/0/144 of160. Chrome prefix underline still FAIL; no browser rerun or inferred PASS.


### 2026-10-22T12:42:06.405147+00:00 — actionable replay boundary: caret contradiction RED→GREEN

- Harness2 task `20261022T115225Z-f7d6b553` finished `ok`: eight gpt-6.1-sol/xhigh calls, zero failed calls/fallbacks, source files unchanged. Its frozen v83 review scores 6/10 REQUEST_CHANGES; current-token and pending field/privacy findings were already fixed in v85/v87. This is not a review score for current source. Final report: `/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261022T115225Z-f7d6b553/report.json`.
- Independently reproduced the additional caret cancellation finding: the generic inert historical-prefix predicate accepts a one-character caret lag, which must not retain an actionable held Space. Actual RED `writer-held-boundary-v93b-caret-red/result.json` (0/1) fails at non-end caret. The earlier v93 invocation selected an incorrect test path and ran no tests; it is a harness-selection error, not product RED.
- Minimal current fix: while a final Space is held, selection or any non-end caret cancels before historical-prefix tolerance. Generic inert native-replay tolerance and delivery remain unchanged. Regression covers partial, complete and foreign snapshots under empty/Unicode prefixes; late exact SSI cannot revive output, source authority, ACK or learning.
- v94 held boundary 5/5 PASS; full affected set 471/471 PASS in 83.721 s. Source closure b10a7f05ece5788c0efa5f9d481d2c3f85519630ac80764b20be0574b0fb1075. Remote evidence: `/home/e/projects/lay-development-runner/kde-install-control-20261020/writer-held-boundary-v94-affected/result.json`. Current bytes are not release-built or native-tested yet. Previous v92 native 7/7 remains valid only for its exact SHA457c02b4f9707a85415138ac550574720d49a424dae9e6b0707727198946100b.
- Next: graph/source linkage → compiler-artifact build → same seven owned Writer rows → independent wire/restoration audit → bounded final Harness2 follow-up. Engine-only diagnostics remain temporary; installed v64 PID3926674, config, daemon/tray/IBus/desktop/Kitty unchanged. No release/local install/push. Chrome typed-prefix underline remains FAIL. Browser matrix unchanged: Firefox92/68/0/0 of160; Chrome8/8/0/144 of160.


### 2026-10-22T12:53:20.557606+00:00 — v94 native 6/7: whole-word refresh absent

Source471/471 and compiled artifact5061dc0c000f15a15c123a7b1c102df00b440f74f02f5e760c85d70d76a7e1b0 were independently linked. Native Writer6/7 PASS; the fourth flip of the five-consecutive row loses final Space. Last actual SSI is “приве” caret5, followed by held Space/RequireSurroundingText; the whole word “привет” arrives only with a later test key, after cancellation. There is no non-end caret in this failure, so do not blame or revert the caret guard without evidence. Primary libibus RequireSurroundingText sets needs_surrounding_text only; current hold has an observation-progress gap. Previous v92 seven-row PASS remains bounded and is not repeat reliability proof. Exact receipts: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-held-boundary-v94-native/result.json` and `liveness-audit.json`. All client7/7 and runner9/9 restorations true; v64 restored PID3998187. No installed release, local install, daemon/IBus/tray/desktop restart or browser statistics change. Next bounded Harness2 review evaluates queue-preserving final-Space delivery and independent source authority; do not add delays, fake receipts or unrelated routes.


### 2026-10-22T13:13:39.118326+00:00 — v95b causal RED; ordered final-Space candidate pending checks

The original v95 test attempt hit a test peer-draining error, not the product defect; preserve it as harness error. Corrected v95b RED reproduces missing dispatch with actual partial SSI, zero forwarded keys versus the authorized Space press/release pair. Receipt: /home/e/projects/lay-development-runner/kde-install-control-20261020/writer-forward-boundary-v95b-red/result.json.

Existing v94 native remains6PASS/1FAIL; v64 restoredPID3998187. Harness2 liveness task20261022T125330Z-dca62ad9 first completed base review: current liveness6/10, native return7/10, conditional forward experiment8/10. Full aggregation still running; this does not review the new implementation.

Candidate moves only that retained Space pair to existing EngineOutput forward at the original callback; paired release is consumed, preceding native keys unchanged, no CommitText/retry. RAII covers request and both output awaits. Dispatch itself supplies no source, result ACK or learning; source-only completion additionally retains the genuine whole-word and all existing token/field/pair/caret/epoch/privacy checks. Source and actual native ordering remain unaccepted pending deterministic checks and short own Writer run. Decision: docs/architecture/decisions/2026-10-22-ordered-replay-boundary-forwarding.json. No permanent runtime adoption, release or local install. Browser totals unchanged; Chrome prefix underline remainsFAIL.


### 2026-10-22T13:24:39.060087+00:00 — ordered final boundary: delivery lease separated from source proof

v96's six held-boundary cases failed before output because the dispatch callback was incorrectly checked with the settled-source predicate. v97 keeps the existing reducer `revalidate(token)` delivery check while the current callback is outstanding; the stronger `exact_replay_source_token(token, epoch)` check still gates source settlement after actual complete-word SSI. No source guard was weakened. v97 focused6/6PASS; affected470/472PASS with two old delivery assertions rejecting the newly explicit ordered-key experiment. v98 migrates those two tests to require exactly one Space press/release only in the live paired branch, no text commit/deletion, no forwarded output in invalidated or release branches, no output from duplicate actual snapshots, and unchanged source/ACK/privacy checks. The old CommitText-on-SSI expectation is replaced by a strict no-text-output assertion. v98 affected gate is running; neither source nor native acceptance is claimed yet. Native helper now maps the existing read-only GTK surrounding/key-filter observer only into the owned Writer child, with original typing/gesture cadence unchanged. Remote accepted engine remains v64/PID3998187; no daemon/tray/IBus/desktop restart or local install/publication.

Receipts remote under `/home/e/projects/lay-development-runner/kde-install-control-20261020/`: `writer-forward-boundary-v96-held-green/result.json` (FAIL0/6), `writer-forward-boundary-v97-held-green/result.json` (PASS6/6), `writer-forward-boundary-v97-affected/result.json` (470PASS2FAIL), and pending `writer-forward-boundary-v98-affected/result.json`. Test migration preimage: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-test-migration-preimages/`. Harness2 task20261022T125330Z-dca62ad9 remains in progress on the prior v94 design packet, not current implementation certification.


### 2026-10-22T13:27:42.728814+00:00 — ordered boundary candidate v98 source closure green

472/472 affected source checks PASS; discovery13.638s, isolated execution82.883s. Source before graph `b481bbc4a9669c22abd63c3cac5a64226103e2d6ca7681f2605c614d30b5fe81`. Remote graphify/architecture update PASS, only source change `src/generated/architecture_graph_receipt.json`; compiler input closure `e43c68cca5e7ffd6c572af621dd9cbb552b05f5ad8d822ae9d824db11dea95b6`. Six generated graph/binding files synced back with preimages at `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-graph-sync/`. Candidate build is running through remote guard; no native PASS or runtime adoption. Next exactly seven own Writer checks with unchanged key timing and read-only GTK order observer.


### 2026-10-22T13:37:33.728509+00:00 — v98 Writer source/native closure, final implementation review pending

Source472/472PASS, graph/architecturePASS, compiler input SHA e43c68cca5e7ffd6c572af621dd9cbb552b05f5ad8d822ae9d824db11dea95b6. Compiled engine SHA 56e7ac524b4228cfbc7a8f789655215ca8b79beee8935fa81004d90535ed44c4; build 70.596s, artifact resolved from compiler output. Own Writer7/7PASS: prefix/second-word hint+Tab, active and closed two flips, five closed flips, ordinary held Shift+A, Space auto-flip. Nine exact conversions with end-caret/no selection, median148.489ms/max219.919ms; this is a bounded client run, not a general speed or reliability claim.

Independent read-only GTK audit confirms all five closed conversions: actual whole-word/end-caret retrieval follows the last native letter and precedes delivery of the forwarded planned Space. Each client receives exactly one Space press/release pair. This is one observed schedule, not a universal FIFO or result-ACK proof. Dispatch still supplies no source, acknowledgement or learning; existing whole-word/token/field/pair/caret/epoch/privacy settlement guards remain mandatory.

Receipts: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-build/result.json; /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-native/result.json; /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-native/ordering-audit.json. All native restoration9/9 and client restoration7/7 checks true. v64 SHA9c789fe107c660a778dea1324f6f9a4ac100c63ad0e68ab99c7cb51ce7ee6fe0 restored PID4140889 at 2026-10-22T13:28:21.088403+00:00; candidate not adopted.

Harness2 task20261022T125330Z-dca62ad9 completed ok: eight calls, zero failed, no fallback, frozen inputs/source unchanged. Selected review scores the prior v94 liveness design6/10 and denies its publication. Its request-await/drop concern is covered by current deterministic tests; this design packet does not certify v98 bytes. Acceptance: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-boundary-liveness-v94-review/acceptance.json. Next bounded review covers final v98 implementation, followed by Chrome typed-prefix underline investigation.

Chrome typed-prefix underline remains visualFAIL; no Kitty regression claim from Writer-only evidence. Browser matrix unchanged/not rerun: Firefox92PASS/68FAIL/0BLOCKED/0NOT_TESTED of160; Chrome8PASS/8FAIL/0BLOCKED/144NOT_TESTED of160. No release, push, local installation or daemon/IBus/tray/desktop restart.

### 2026-10-22T14:22:03.524764+00:00 — v98 final review and native Chrome identity/query diagnostic

Writer final review task20261022T134131Z-5580fa6d ok8/8, score8/10, zero blocking findings. Independent frozen7/source-origin9 hash checks and source472/native7/order5 evidence linkage PASS; exact baseline/candidate/config remote hashes rechecked at14:20:17UTC. Receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/writer-forward-boundary-v98-final-review/acceptance.json`. Bounded evidence accepted, candidate not adopted/published. Unmeasured successor-cleanup/failure-release schedules retained as limits; no speculative runtime fix.

Chrome v99 and v100b diagnostics PASS: real native nodes distinguish equal text/caret under one IBus context; accessibility-bus peer credentials match X11 PID71108. Focused+editable Collection identifies the intended native node6/6, median1.126709ms/max1.936602ms for the query, not conversion latency. v100 FAIL was an invalid diagnostic MATCH_EMPTY filter, corrected to MATCH_ALL in unused match dimensions. Own windows closed, all seven restoration predicates and nine protected process identities preserved. Exact receipts under the same evidence root: `chrome-two-field-binding-v99-probe/result.json` and `chrome-focused-collection-v100b-probe/result.json`.

Corrected interpretation: IBus capabilities41 includes SurroundingText32. Actual log has ten requests but zero SetSurroundingText callbacks. Existing cached Chromium GTK implementation omits retrieve/delete signal handlers; exact installed version/backend and usable deletion must be established before production changes. Accessible preedit projection is not a committed-source/result ACK. No code or runtime authority change in this diagnostic; Chrome visual underline FAIL and browser matrix unchanged.


### 2026-10-22T15:03:16.931162+00:00 — Chrome v102b: native ABA fact, no runtime fix

Hypothesis: matching IBus path/text/caret does not prove uninterrupted native-field ownership. One own Chrome with a single RU activation produced A→B→A, equal returned snapshot and distinct intermediate native node. Eight/eight focused Collection observations matched the intended node; actual Focus/Reset callbacks were also received by IBus. v102 FAIL was the test assuming Tab return preserved selection; v102b uses native End and records both states. All restoration seven/seven true, protected nine processes preserved. Existing authenticated Reset revokes canonical source tokens; inspect existing ingress/fence instead of adding a second owner or generation. Exact installed Chromium source also excludes usable GTK delete/retrieve and ATK EditableText; NONE/zero-length Pango styles cannot remove the prefix underline.

Receipt: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-native-aba-v102b-probe/identity-audit.json. Chrome visual defect remains FAIL; no runtime provider, executor, new engine name or matrix promotion. Writer v98 accepted only as bounded source/native candidate, not installed. Pending Harness2 task20261022T143718Z-b6db8805. Details and primary-source hashes belong to the IME inline-hint preflight document.


### Chrome native range/commit primitive v103b — 2026-10-22T15:26:31Z

One owned Chrome field: exact native selection [0,4] + actual existing engine/InputContext CommitText("1") produced `1 `, caret 1/no selection; other field unchanged. Selection dispatch 0.163 ms, observed commit 37.996 ms, n=1. Seven restoration guards and nine protected processes preserved. This is a native primitive only; Chrome presentation remains FAIL, browser matrix unchanged. v103 diagnostic physical-glyph assumption failure is retained.

Authority/alternatives/consequences and the next in-callback native-query discriminator are recorded once in [the owning preflight](../architecture/ime-inline-hint-preflight-2026-10-20.md). Receipt: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-native-selection-v103b-probe/acceptance.json`. No production code or installed bytes changed.


## 2026-10-22T15:40:30.803525+00:00 — Chrome callback diagnostic

v104 one in-callback native read PASS (743us,n=1); exact v64 restored PID297899,9+7 restoration checks true. Chrome underline remains FAIL. v105 exact-selection diagnostic preflight recorded in `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`; receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-native-callback-v104-probe/acceptance.json`. No production fix/adoption or matrix change.


## 2026-10-22T15:47:10.284494+00:00 — Chrome native selection refuses without ACK

v105 diagnostic FAIL `selection_not_observed`1165us; no literal commit; v64 restored PID327512,9+7 restoration checks true. New causal v107 event-driven discriminator preflight in owning IME document. No production implementation/adoption or matrix change.


## 2026-10-22T15:54:33.284419+00:00 — Chrome native event-selection primitive

v107 bounded primitive PASS2443us,n1: actual event confirms exact selection before standard CommitText; result1+Space, second field unchanged. v64 restored PID345082,9+7 checks true. See owning IME document and `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-native-selection-event-v107-probe/acceptance.json`. Chrome visual FAIL unchanged. Harness2 task20261022T154252Z-2489f752 tool-ok8/8 but deliverable blocked/unread, rejected; one corrected read-only second pass pending. No source authority/runtime adoption/matrix change.


## 2026-10-22T17:09:34+00:00 — KDE-only module boundary v109

User scope: separate KDE folder, GNOME frozen, then Chrome only; reuse existing mechanisms and do not repeat rejected experiments. Final remote compile and exact29/29 existing module tests PASS14.254s; canonical remote graph/architecture PASS51.226s with verified resource limits. Existing files retain logic/module identities, and the common KWin read body is relocated exactly into the KDE adapter; four local GNOME/five protected common files unchanged. Harness2 USED task20261022T170428Z-3a440f56,8/8 successful calls/no fallback, frozen static review9/10/no actionable defects. The final small helper extraction is independently checked outside that frozen review. Acceptance and limitations in acceptance.json; current runtimePID345082/hash/config/protected processes unchanged. Owning analysis and evidence: [IME preflight](../architecture/ime-inline-hint-preflight-2026-10-20.md), evidence root `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/kde-module-boundary-v109/`.

Retain v108 native-range478-test abort (stack overflow in pending_transfer_focus_in_then_focus_in_id_enriches_one_prearm_request after32 tests); no full-path PASS/native adoption. Chrome rendering still FAIL; no runtime/client/release change, full browser100PASS/76FAIL/0BLOCKED/144NOT_TESTED of320 unchanged.


## 2026-10-22T23:01:57Z — Chrome native observation wire discriminator

Fresh owned Chrome, actual RFB input, installed1.0.94.1: prefix remains composing; native reads expire at20ms. Passive wire capture proves KWin sends signed int32 PID while Ingress expects uint32; the private peer fixture lacks PID. Repair the measured wire boundary and fixture, without changing budgets, authority, GNOME or delivery routes. Native underline/gap remain FAIL pending repair and live verification. Details and exact receipts belong to the owning IME preflight document, section Chrome-only live root cause: signed KWin PID.

### 2026-10-22T23:21:39.426237+00:00 — Chrome-only signed-PID fix and KWin orphan hypothesis

v115 source493PASS/2retainedknownFAIL, architecturePASS; runtime1.0.94.2installed from exact artifact. Physical Chrome underline remainsFAIL. Passive v116 replies identify ID reuse/UnknownObject, and inventory finds three abandoned Lay scripts with no live bus owners and non-contiguous IDs. Before cleanup, recorded a bounded exact-name-only experiment in the owning preflight document. No GNOME, daemon, other-client route or timeout change. Harness2 task20261022T225149Z-98974412 completed8/8/no fallback; prior frozen candidate readiness3/10, not acceptance of the new patch.

### 2026-10-22T23:26:04.071437+00:00 — Chrome-only v118 native session failure diagnostic

After exact abandoned-script cleanup, desktop observation reaches native AT-SPI; one `native_ambiguous_field` followed by `native_session_bus` is measured, not a display fix. Hypothesis: inspect the underlying zbus bootstrap error rather than broadening the timeout or granting text authority. Add only opt-in connection-error evidence before a bounded Chrome probe. Owner: `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`. Runtime authority unchanged.

### 2026-10-22T23:34:34.712287+00:00 — Chrome-only v119 post-managed-release document freshness

Measured v118: prefix plain, hint absent; actual document read exists only before the following key, while the early managed-release path only flushes a stale display frame. Repair only the KDE Chrome native adapter and its existing release dispatch: fresh native read after authenticated consumed release, then existing suffix display refresh. No synthetic SSI, ACK, new owner or edit route. Physical v118 receipt remains FAIL for missing hint. Owner: `docs/architecture/ime-inline-hint-preflight-2026-10-20.md`.


### 2026-10-22T23:45:18.798014+00:00 — Chrome-only actual cursor-triggered observation (v120)

Measured v119: focused source 47 PASS / 0 FAIL, graph PASS; source receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-release-observation-v119-source/result.json`. Installed engine SHA `21119e09ed5571c31db9d039e2a3343e6a2b3c1507f1ed31ac6fd4f3cf79354e`. Physical owned Chrome result `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-release-observation-v119-stderr-native-r2/result.json`: prefix plain, composing=false, hint empty; FAIL. Config/daemon unchanged. First probe lost its engine PID before receipt; stderr unavailable, no crash cause established. Same bytes under scoped stderr capture stayed alive; not a stability acceptance.

Exact physical bus ordering: Chrome queued letter press/release before previous CommitText was applied; accessibility GetText still returned `п` after managed append to `пр`. The v119 early-release hook also missed normally admitted releases. Rejected release-triggered-only design rather than adding a second unreliable wake.

Minimal v120 change: existing CursorGeometry callback wakes the same bounded native document adapter, which authenticates actual KWin Chrome PID, focused field and current admission. Cursor geometry is a wake only and never document/commit evidence. Skip if actual native snapshot is already current. Common observed-suffix prediction runs only after a new current actual read. Removed v119 release hook. GNOME callbacks and delivery/reducer/verifier/executor contracts unchanged. Source/installed/physical results pending. No full matrix claimed.

### 2026-10-22T23:55:37.879348+00:00 — Chrome v120 measured result and bounded runtime diagnosis (v121)

Source: v120 focused 47 PASS / 0 FAIL, canonical graph PASS; receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-cursor-observation-v120-source/result.json`. Runtime SHA `ca03d094ebea436c29bb6755c8e88d4b35f851f06ca627ef4179e1d48ba6331e`, display 1.0.94.2. First owned Chrome physical result: ordinary committed prefix from the first character (`пров`, composing=false), but no hint; overall FAIL. Actual native reads alternate success and `native_bus_connect`. Receipt: `chrome-only-cursor-observation-v120-native/result.json` under the same remote root.

A bounded second owned Chrome probe with syscall-only strace captured actual `clone3 = -1 EAGAIN` when a new connection thread was requested, followed by IME SIGABRT. Receipt `chrome-only-cursor-v120-connect-diagnostic/result.json`; syscall files `chrome-only-cursor-v120-connect-syscalls.*`. No document payload was traced. Parent KDE development service has TasksMax=512; measured 503 tasks after the probe, pids.events max=1310. This proves resource denial for this probe, not the cause of every earlier presentation failure. Auto-activated replacement PID1773204 uses the same bytes.

Next bounded diagnostic reuses these exact IME bytes in a separate CPU100%/MemoryMax768MiB/TasksMax32 service with stderr capture. KDE/IBus/daemon/other windows and their limits remain unchanged. No new source mechanism, delivery path, proof authority, or GNOME behavior adopted. Chrome visual hint, Tab, and geometry remain NOT ACCEPTED pending the isolated physical result. Full browser table unchanged.

### 2026-10-23T00:01:08.139064+00:00 — Chrome-only completed-readout observation (v122)

Measured v121: same v120 bytes in bounded `lay-chrome-ime-observation-v121-r2.service`, PID1785151, protected 19 processes/config unchanged. First unit launch missed the dedicated XDG cache/IBus address and exited before activation; exact-environment correction and selecting the existing Lay engine restored it without restarting global IBus. Source was not changed for this diagnostic. Receipt `chrome-only-v121-isolated-runtime-r2/result.json`. Physical receipt `chrome-only-v121-isolated-native-r2/result.json`: 7 successful actual reads in 8.506–15.236ms; no native_bus_connect and no IME exit, but hint still absent (FAIL). Chrome DOM is `пров`, composing=false. Passive AT-SPI calls prove final pre-Space reads still have only 3 characters; 4-character document is read only at later Space. Resource denial is isolated; cursor callbacks alone do not settle Chrome's accessibility cache.

Candidate v122 reuses the existing readout-completion callback, with existing generation, freshness and input-frame authority checks before any refresh, then retains the original full candidate identity check afterward. KDE-only helper requires an existing native binding and checks actual observed-suffix readout readiness instead of treating a current binding as sufficient text proof. Refresh still authenticates the actual Chrome window/peer/field and reads its real document; no timer, second queue, new ACK or delivery path. A changed actual read schedules current material and old results retain their normal discard path. GNOME branches and other client adapters are unchanged. Results pending; no display acceptance or new full matrix.

### 2026-10-23T00:14:38.487059+00:00 — Chrome actual text notifications (v123 development trial)

- v122: remote source 50/50 PASS, canonical graph PASS; installed SHA `648ccfae61384c494334719967d4ce650b8651175b4c743449a16b4604450281`, PID 1818636. Physical fresh Chrome prefix `пров` is ordinary non-composing document text, but InlineHintSnapshot is empty: client **FAIL**. Receipt: `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-readout-observation-v122-native/result.json`. One later native read reached its 20 ms deadline. No client PASS inferred from source checks.
- First failed transition remains the actual post-insertion document observation: key, cursor, and calculation callbacks can precede Chrome's accessible-text update. The v122 worker wake is removed; the common precognition worker regains its prior behavior.
- v123 hypothesis: retain the authenticated native connection and register this adapter's own bounded TextChanged subscription, using the existing native exact-selection subscription pattern. The event payload is discarded. A callback only wakes a fresh canonical desktop / peer / single focused field / current admission / actual document read. No polling, retry timer, new authority owner, synthetic SSI, commit ACK or edit bypass is introduced.
- The subscription is a transport resource in NativeDocumentBinding, canceled when its last owning binding drops. Field identity, focus serial, lease and layout must match; per-key tail changes require a new actual read, never carry an edit grant. Reuse of the already authenticated connection still revalidates bus GUID, peer PID, single focused field, editable/nonprivate state, selection, scalar counts and text. The 20 ms I/O budget is unchanged.
- GNOME and non-Chrome runtime eligibility are unchanged. New regression tests distinguish a permitted read wake after a key from stale focus/lease/layout/field and GNOME refusal. Source, runtime, client and visual geometry gates remain separately pending. No full browser matrix or release claim.
- Harness2 task `20261023T000604Z-52f12b34` (gpt-6.1-sol/xhigh) failed at its single base-call 240 s timeout, with no output and no review score; this is not a review acceptance. A smaller follow-up review will inspect the native callback lifecycle.

- 2026-10-23T00:16:20.876137+00:00: v123 first compile correctly rejected the recursive observer/adaptor future without an explicit Send contract; no runtime installation. The registration boundary is now explicitly boxed + Send, retaining bounded native read/drop contracts. Source rerun uses a separate v123-r2 receipt.


### 2026-10-23T00:34:50.687953+00:00 — v123 failed notification, v124 compatible registration

Source v123-r2: 51/51 focused tests and architecture graph PASS; candidate and
installed SHA-256 `17afed0d29d034456300812fbb16370990031c2e163957196703ef9ab52eae55`,
engine PID 1872068. Three owned normal Chrome field-1 probes retain ordinary
typed `пров` but no suffix: native FAIL, not acceptance. Read latency from the
first probe was 7.387–17.389 ms for six successful reads; hint latency absent.
Receipts: `/home/e/projects/lay-development-runner/kde-install-control-20261020/`
`chrome-only-readout-observation-v123-r2-source/result.json`,
`chrome-only-readout-observation-v123-deployment/result.json`, and
`chrome-only-{bound-text,text-signal,signal-lifecycle}-v123-native/result.json`.

The first failed transition is registration, not Chrome's text payload: the
installed AT-SPI registry 2.44 rejects the three-argument `sass` RegisterEvent
request while returning an empty reply. Remote journal confirms three
`got RegisterEvent with invalid signature 'sass'` warnings. Official tag
AT_SPI2_CORE_2_44_0 registry.c accepts only `s` or `sas`. Passive all-signal
monitor showed no EventListenerRegistered or TextChanged for that request.
The v124 trial uses the backward-compatible two-argument `sas` ABI only for
the native text observer. Its exact sender/path match and canonical field,
focus, lease, layout, fresh-read and edit checks remain unchanged. Existing
selection delivery is outside this trial and is not modified. GNOME code is
unchanged. Source, installed and physical v124 gates pending. Full matrix
remains 100 PASS / 76 FAIL / 144 NOT_TESTED of 320.

2026-10-23T00:37:54.035328+00:00: first v124 source run: 51 PASS / 1 FAIL, no install.
The new protocol test compared zvariant structural display `(sas)` to D-Bus
wire display `sas`. Use the documented `to_string_no_parens()` for the
wire assertion; registration code and compatibility contract are unchanged.
Receipt `chrome-only-registry-abi-v124-source/result.json`; r2 pending.


### 2026-10-23T00:51:08.780821+00:00 — Chrome registry ABI root cause and v124 physical result

- Installed AT-SPI registry 2.44 rejects RegisterEvent signature `sass` but returns an empty reply; its journal confirms `got RegisterEvent with invalid signature 'sass'`. Exact upstream 2.44 registry.c supports `sas`/`s`. v124 uses the compatible two-argument `sas` body; exact sender/path signal filtering is unchanged.
- Source: remote `chrome-only-registry-abi-v124-r2-source/result.json`, 52/52 exact checks and canonical architecture graph PASS; source closure ba42ab9c35668fda468058148dbba4d1938971961187eef57b919ae99f860ac9. Earlier first-attempt test failed because zvariant Display wraps top-level types; actual wire signature assertion uses to_string_no_parens. Failed receipt retained.
- Installed engine SHA-256 86a3f9a058610c6f4f7ba01ef7224e4174e1b566bc8d63f0fae62e62a1b41b47; PID 1935224, unit lay-chrome-ime-candidate-v124.service, version 1.0.94.2. Both activation paths and running bytes verified; daemon/tray/config/protected apps unchanged. Deployment receipt: remote chrome-only-registry-abi-v124-deployment/result.json.
- Native ordinary input: actual DOM `пров`, composing=false, selection [4,4], separate hint `ерка`; screenshot has no underline, selection or gap. Tab produced `проверка `; actual native TextChanged signals were observed on the bound Chrome field. Receipt chrome-only-registry-abi-v124-native/result.json. The old 7.86ms metric starts after a key gap and is not end-to-end latency.
- Native contenteditable: actual hint appears in 51.73ms upper-bound observed time from final keydown; Tab produced `проверка` plus U+00A0 (Chrome rich-text trailing-space representation). The diagnostic's literal U+0020 wait falsely times out at 1218.86ms; that number is not product latency. Correct diagnostic accepts exactly one U+0020 or U+00A0 and retains raw DOM. Receipt chrome-only-registry-abi-v124-rich-native/result.json.
- Harness2 USED: lay2 task 20261023T001629Z-628ade67, gpt-6.1-sol/xhigh, status ok, eight calls. Frozen v123 review identifies retained completed observer, missing uniqueness on connection reuse, and no post-read field focus validation. v125 will address all three locally in the KDE native-document adapter, within the existing 20ms deadline. No GNOME/delivery/admission/reducer/verifier/ACK contract changes.
- Independent effect-boundary inspection: observe_document_snapshot installs actual native text; observe_visible_postcondition only verifies an existing pending operation against matching field/token/epoch and actual expected text. commit_observed_literal_preedit retains all existing exact-owned-preedit/admission/privacy/scope/actual-tail gates before the existing composition commit. The notification itself supplies no proof, ACK, SSI capability, edit permission or document payload.
- Scope: two short Chrome fields only. Remaining Chrome fields/modes and focus-switch lifecycle remain NOT_TESTED for these bytes; full matrix unchanged (Firefox 92/68/0/0 of160; Chrome8/8/0/144 of160). No release, updater or global desktop claim.
- Evidence root: /home/e/projects/lay-development-runner/kde-install-control-20261020/. Local screenshot: /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-only-registry-abi-v124-native/prefix.png.


### 2026-10-23T01:00:28.683556+00:00 — v125 source/installed and 14-cell native failure classification

- Source 54/54 PASS including real completed-task liveness and cancellation regressions; architecture graph PASS. Engine 42d4220e63dcc01aacf47c975a6e9f51f05e6b11a694f8216941c84ab8ae3982 installed as PID1981828, lay-chrome-ime-candidate-v125.service, 1.0.94.2. Protected20 processes/config/tray unchanged. Receipts chrome-only-observer-review-v125-source/result.json and chrome-only-observer-review-v125-deployment/result.json.
- Physical 14-case Chrome check: 4 PASS /10 FAIL, retained at chrome-only-ten-field-v125-native-summary/result.json. Normal text/URL/textarea and private rich passed. Search/tel/rich/plain/iframe-rich can retain composing=true; email/iframe-input/fullscreen rich can lack suffix. These are failures, not acceptance. All per-case restoration checks pass.
- Successful-case upper-bound observed hint median52.18ms/p95 52.38ms (n4), Tab median29.44ms/p95 34.17ms (n4). Failed-case 1.2s deadline waits are not product latency measurements. The small successful subset cannot characterize all fields.
- First shared mechanism: discovery/read deadlines are intermittently exceeded; later calls can observe GetText/count from the previous composition update while CaretOffset already includes the next character (search trace: text `п`, count1, caret2; later `пр`, count2, caret3). checked_snapshot correctly rejects these inconsistent observations.
- v126 hypothesis/minimal implementation: batch independent metadata, caret/selection/count recheck, and final focus/privacy metadata queries on the existing native connection. All checks still required, actual text still read only after initial privacy/focus checks, same20ms deadline/no polling/retry/cache/capability/ACK changes. Connection-reuse uniqueness and post-read field revalidation from Harness review remain. Run same54 source checks and same14 physical cases.
- Canonical/source verification is remote and guarded. No GNOME or other client changes; full320 browser matrix remains100PASS/76FAIL/144NOT_TESTED.


### 2026-10-23T01:08:31Z — v126 build-only correction, no client verdict

The first v126 release compile rejected futures_lite references: this existing
crate was dev-only, so the 54 passing test builds did not prove the production
binary could compile. Retain the failed receipt at
chrome-only-batched-native-v126-source/result.json. Move the same version-2
dependency into normal dependencies (no new package/version/lock change).
The independent RPC batching remains confined to the KDE native Chrome reader;
all original privacy/focus/uniqueness/scalar/deadline guards remain. The guarded
remote repeat uses chrome-only-batched-native-v126-r2-source/result.json.
No install/client claim is made while that build runs.


### 2026-10-23T01:16:47Z — v126 physical evidence, v127 observation-gap regression

- v126-r2: remote 54/54 source and architecture PASS; installed SHA b8b0a6e0701b3ca27d23b709e3258406350607024b5486114b74da021fba62c6, PID2045328. Remote compiler source closure06dbd45ff6d500f8291698ecac6a4696cba59d8c2edad5210a8ebafab6a43e53. Source and deployment receipts chrome-only-batched-native-v126-r2-source/result.json and chrome-only-batched-native-v126-deployment/result.json.
- Same14 native Chrome scenarios:7PASS/7FAIL. Successful subset hint median51.99ms/p9553.66ms (n7), Tab median34.07ms/p9534.88ms (n7). Timeouts in failed cases are not latency samples. Receipt chrome-only-ten-field-v126-native-summary/result.json. No full browser matrix improvement inferred.
- After a transient deadline/incoherent document read, the adapter currently destroys its own notification subscription and revokes the word as though an input key were missing. That conflates absent text evidence with an input gap; later actual text cannot wake the removed observer.
- Reproduced the contract failure before the fix: exact remote regression fails0PASS/1FAIL at the continuation assertion; chrome-only-observation-gap-v127-tdd/result.json is EXPECTED_REGRESSION, not product acceptance.
- v127 preserves only the same admitted field transport on deadline/document-change/incoherence. observe_document_snapshot(None) advances revision and invalidates document-derived edit evidence. Normal literal input retains the existing canonical key path; no old snapshot is used for edits. Identity/privacy/admission/selection loss still fully revokes. Add selection-specific error so a real selection cannot be mistaken for an incoherent read. A subsequent full actual read must restore evidence, not the notification payload.
- Local/remote source manifests show formatting-only differences outside KDE. Keep local GNOME bytes untouched; exact remote compiler inputs and differing files are archived under chrome-v126-remote-source in the local development evidence root. Tested source is the pinned remote workspace, not a false claim of local byte identity.
- v127 source/install/client pending. GNOME and other clients remain unchanged; full320 matrix100PASS/76FAIL/144NOT_TESTED.


### 2026-10-23T01:28:10.931350+00:00 — v127 physical result and v128 install-refusal diagnosis

- v127 source: 56/56 selected checks and architecture graph PASS; source closure 0aa5cf529c3b68d65d938d59e589d3562ee43e29ebc681ea452197326a2521d5. Installed PID2089461, SHA9207858f7ea7898ba12aab6390a6b52f252e08393100e68cdb6d6837e9b3a74d; both entrypoints verified, protected20 apps/config/tray unchanged. Receipts chrome-only-observation-gap-v127-source/result.json and chrome-only-observation-gap-v127-deployment/result.json.
- Native bounded14-case result: 10 PASS /4 FAIL, chrome-only-ten-field-v127-native-summary/result.json. Search and iframe-rich retained composing=true; ordinary rich lacked suffix. Email accepted the word but lacked trailing whitespace in DOM; distinguish HTML email value sanitization from a product failure before correcting that assertion. All restoration checks passed. Full320 matrix unchanged.
- Successful subset only: hint median52.373ms/p95 77.449ms, Tab median25.107ms/p95 35.250ms, n10. Failed waits excluded from product latency. No release or all-client acceptance.
- First remaining unresolved transition: a successful actual read can be followed by a refused install/key, with no existing reason trace. v128 adds metadata-only refusal tracing inside KDE native_document::install_native_document, preserving all guards and behavior. It is a diagnostic build, not a claimed fix. Native source/installed/physical gates pending. No GNOME or other-client source changes.


### 2026-10-23T01:44:31.610531+00:00 — v128 diagnostic result and v129 literal-transfer gate trace

- v128 is metadata-only over v127. Remote56/56 checks and graph PASS, compiler closure09e58a38f940b78d7a50964c8fa992d9718a668ea65ab96115008cd65f625440; installed PID2131179 SHA31ef785e93b701f0da796a1e27e5eeb119475da568c24872c1499fc7844221ad. Receipts chrome-only-install-trace-v128-{source,deployment}/result.json. Protected20 apps/config/tray/daemon/IBus unchanged.
- Native14-case result9PASS/5FAIL; chrome-only-ten-field-v128-native-summary/result.json. No improvement inferred from a diagnostic build. Successful n9 hint median52.600ms/p9583.315ms; Tab median24.630ms/p9534.731ms. Failed waits excluded. Full320 matrix remains100PASS/76FAIL/144NOT_TESTED.
- Email install refusal is now measured: desktop identity changed, while accessibility bus/owner/application/field still match. Canonical KWin local connection lifetime changes after failed observations; this is not proof of physical focus theft. Keep all identity refusal guards.
- Email oracle corrected narrowly: HTML email sanitizes ASCII whitespace (WHATWG input email-state). A missing DOM separator may count only with a fresh post-Tab native input event carrying exactly the delivered suffix+separator. Old receipts remain unchanged.
- Same-window focus-cycle v128 physically reproduces prefix composition despite multiple successful actual snapshots with matching scalar counts. Receipt chrome-only-focus-cycle-v128-native/result.json, transition_pass=false, restoration all true. Therefore deadline alone is insufficient as a root. v129 logs existing literal-transfer gates and focused candidate cardinality only; no authority, timeout, GNOME or other-client behavior change. Build/install/native pending.
- Harness2 task20261023T013040Z-a5285e30 produced a frozen7/10 review; final report pending. Findings distinguish actual read from installation, subscription liveness and late known-start scheduling. Review is static evidence only, not a physical result.


### 2026-10-23T01:55:09.898476+00:00 — Chrome v129 diagnosis and v130 transport-mode repair

Scope remains remote KDE Chrome only; GNOME is a read-only reference. v129 source/architecture passed 56/56 and installed SHA-256 `95da37ffa56747588f4c6266e33b85b36eedf16b40714a087fe423127ce0dd39` (PID 2196193). The same-window text/rich/iframe-rich/textarea/text-return probe passed the first field only; four subsequent fields retained the typed prefix in composition. Every failing literal-transfer observation had current admission/scope/native binding, exact owned tail, visible clean preedit and end caret, but `managed=false`.

First failed transition: `install_context_activation` preserves literal preedit acquired before Ready, then clears `word_input_mode`. The existing literal-commit helper correctly refuses that unset mode. v130 settles only an unset mode in the KDE native Chrome adapter after fresh actual data matches the complete owned literal tail, using the existing input-mode selector. An existing terminal mode is never migrated. The common commit helper, admission/reducer/verifier/executor and GNOME source remain unchanged. A regression first failed with `None` instead of `ManagedCommit` (expected red receipt); the completed test also requires exactly one literal CommitText, no deletion, unchanged token and no document-edit proof after commit. Negative cases cover missing/stale/mismatching/selected/unowned input, established terminal mode and GNOME.

Evidence: `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-focus-cycle-v129-native/result.json`; red TDD `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-preserved-mode-v130-tdd/result.json`. v130 source/build/install/physical acceptance are pending. Native selection, ACK and capability flags are unchanged. Full browser matrix remains Firefox 92 PASS/68 FAIL, Chrome 8 PASS/8 FAIL/144 NOT_TESTED of 320 overall.


### 2026-10-23T02:17:53.797168+00:00 — v130 result and v131 late-known display recovery

- v130 remote source/architecture: 58/58 PASS; source closure a1f6ff8d840ef67f16fb6619b2c3c31c12fc3ccc344527ecd7b434fc39b4e42e. Installed PID2229434, SHA5481a1be18c8a2cbd9e4c34d77c412b6c877a6c82f4d4db5466076877670ebd9. Protected20 apps/config/tray/daemon/IBus unchanged. Receipts chrome-only-preserved-mode-v130-source/result.json and chrome-only-preserved-mode-v130-deployment/result.json.
- Physical same-window five-field cycle: first run all five prefixes left composition, but rich/iframe-rich/text-return lacked hints. Second diagnostic cycle still retained composition in iframe-rich and textarea. Neither cycle passed. First-cycle successful hint/Tab checks cover text and textarea only; no final visual acceptance is claimed. Both probes restored state. Receipts chrome-only-focus-cycle-v130-native/result.json and chrome-only-focus-cycle-v130-complete-trace-native/result.json.
- Probe trace extraction was incomplete: set subtraction discarded repeated lines; byte-offset replacement also failed when the existing 500KiB log compacted. New helper uses an ordered initial-tail anchor, preserves duplicate events, and marks a missing/nonunique anchor incomplete. Product logger unchanged; DOM/screenshot evidence remains separate.
- Next first failed display transition is source-supported: a late known-word actual snapshot can yield a valid canonical input frame with no composition and pending=false after literal transfer cancelled the previous task. Common refresh_observed_suffix_precognition does not seed that state. v131 confines the repair to KDE native Chrome: require current admission/native actual unselected word matching the owned tail, obtain the unchanged canonical frame, and call the existing visible-input scheduler. The binding coalesces the exact last requested display frame only; token/owner changes do not reuse it. No synthetic pending/SSI/ACK/capability or edit grant. Existing unknown-start branch and all common/GNOME code are unchanged.
- The first v131 source attempt failed59/60: the new fixture left the default text backend uinput, so no hint worker was enabled. Correct the fixture to actual IME configuration, do not alter product eligibility. The corrected narrow red run disables only the new scheduler bridge and fails at0 instead of1 scheduled worker, as required: chrome-only-late-known-v131-r2-tdd/result.json. The guarded v131-r2 build and physical acceptance are pending.
- Harness2 USED: lay2 task20261023T013040Z-a5285e30, gpt-6.1-sol/xhigh, statusok,8 calls,0 failed calls/fallbacks. Report /home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261023T013040Z-a5285e30/report.json. Frozen adapter review7/10 distinguishes read/install, subscription liveness and the conditional late-known scheduling gap. It does not review v130/v131, run tests, or certify live Chrome; independently verify those changes.
- Scope: remote KDE Chrome only. No release/push or other-client acceptance. Full320 matrix unchanged: Firefox92PASS/68FAIL; Chrome8PASS/8FAIL/144NOT_TESTED.

### 2026-10-23T02:38:46Z — v131/v132 installed evidence; Chrome remains unaccepted

- v131 corrected source and architecture checks passed60/60. Installed SHA880b5af4 (full digest in chrome-only-late-known-v131-r2-deployment/result.json). The focus-cycle still failed. Later canonical KnownStart frames were mainly after Space, so the late-known scheduling repair alone does not explain or fix missing active-word hints. No physical acceptance is claimed.
- v132 removes the KDE native display callback's requirement for a pre-existing native binding. An actual cursor callback can now establish the first binding through the unchanged authenticated, bounded native read. Previously a missed initial read left no binding, no subscribed wake and a callback that refused to try. No timer, extra authority, timeout, GNOME or other-client change.
- v132 remote source/architecture60/60 PASS, source closure2e476f051760bd52fe085f7a7c511378f0b1a8e8ad04e20ef46bd453ce5613e8. Installed2026-10-23T02:32:18.349178+00:00, PID2343180, unitlay-chrome-ime-candidate-v132.service, version1.0.94.2, SHA4d2963d2141f944a27506e602d45972d47d08935dc2995ae90f4c31937db00d3. Protected20 processes, configuration, tray, daemon and global IBus unchanged. Receipts chrome-only-cursor-bootstrap-v132-source/result.json and chrome-only-cursor-bootstrap-v132-deployment/result.json.
- Physical same-window text/rich/iframe-rich/textarea/text-return cycle: all5 actual prefixes composing=false; hint plus exact Tab acceptance3/5 (text, rich, textarea). iframe-rich and text-return had no hint. Successful observed hint upper bounds279.086/44.518/50.861ms and Tab34.489/34.516/25.372ms. Failed1.2s waits are not product latency. Receipt chrome-only-focus-cycle-v132-native/result.json; ordered trace extraction and restoration complete.
- Visual evidence is still FAIL: prefix DOM is `пров` and hint is `ерка`, but the overlay can begin one character early and cover the final typed letter. Local inspected screenshots chrome-v131-prefix-0.png and chrome-v132-prefix-0.png under /home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/. Underline removal alone is insufficient. Do not count DOM or source success as visual acceptance.
- Next bounded investigation: confirm whether actual current native text fails the UnknownStart display-only readout predicate; inspect stale caret geometry separately. Preserve UnknownStart lineage/count/token and all whole-word edit gates. No runtime authority migration or full-matrix claim. Full320 evidence unchanged: Firefox92PASS/68FAIL; Chrome8PASS/8FAIL/144NOT_TESTED.


### 2026-10-23T02:53:12Z — v133 bounded readout and post-commit geometry trial

The v132 one-field settle probe confirms the hint rectangle remains after the third character for at least350ms while actual DOM has four ordinary characters. It is not only an immediate-screenshot race. Receipt chrome-only-geometry-settle-v132-native/result.json. Existing native terminal hint presentation already asks the client for fresh geometry with an empty HidePreedit lifecycle; reuse that exact output for an independently current native managed Chrome word. No computed character-width adjustment or suffix composition.

The retained acquisition prefix exceeds this UnknownStart lineage's observed-key count. New regression uses an actual CommitText for the next character plus a fresh actual native snapshot, and fails the previous readout gate despite exact managed text. The KDE predicate now requires BOTH the current admitted native word and existing exact post-CommitText managed proof. The common retained-count guard consults this display/append witness; all other snapshot/boundary/privacy/token checks and exact whole-word count checks remain. No lineage count/completeness or edit authority changes. Common presentation helper is renamed consistently; GNOME files and behavior remain unchanged. Decision updated in docs/architecture/decisions/2026-10-23-kde-chrome-native-text-notification.json.

Red evidence: initial diagnostic selected0 tests and was explicitly rejected as UNEXPECTED_RESULT (no PASS). Corrected discovered module route executes exactly both new positive regressions; both fail at their intended readout/geometry assertions, installed bytes unchanged: chrome-only-readout-v133-r2-tdd/result.json. v133 selected71 source tests and canonical graph PASS,11.55s test execution. Negative cases require fresh actual text/revision, no selection, unchanged focus/lease/layout/privacy and KDE. Release build, installation and physical acceptance pending. Full320 matrix unchanged100PASS/76FAIL/144NOT_TESTED.


### 2026-10-23T03:13:13.837553+00:00 — v133 physical rejection; bounded native character geometry v134

v133 selected71 source/architecture checks passed, but installed PID2412141 SHA f722cba67cec9d32e7aca0db2549f76c87b790dc411e0b101a3244b3d11e7657 fails the physical five-field Chrome cycle: all ready=false, four rect=None and a stale fifth suffix. Receipt chrome-only-focus-cycle-v133-native/result.json. The common terminal HidePreedit lifecycle does not cause Chrome to report post-commit geometry; reject this Chrome trial and restore the terminal-only helper. Keep the independently guarded UnknownStart readout repair. No client acceptance is claimed.

A read-only focused-field diagnostic returns actual text/caret but initial zero-sized character bounds. After the first character-extents query, measurements at60ms,200ms,500ms give final character [67,285,11,21], endpoint78; empty-range and at-caret queries remain zero. Receipts chrome-only-native-caret-v133-{native,settle-native,offset-native}/result.json. Component [28,278,545,35], focused editable field is VISIBLE and SHOWING. This supports querying actual last-character geometry, not a guessed width or a zero-range API. Chromium primary reference: https://raw.githubusercontent.com/chromium/chromium/main/ui/accessibility/platform/ax_platform_node_auralinux.cc (GetCharacterExtents enables inline text boxes). These diagnostics alone do not establish product latency or acceptance.

v134 trial adds one optional character-bounds query to the existing bounded KDE Chrome actual snapshot, retains geometry with that exact snapshot/field/token/lease/layout/tail, and discards it when a result reuses transport. Native geometry suppresses the stale IBus rectangle; missing/zero/invalid bounds hide the projection. It does not change actual document validity, edit admission, reducer/verifier/executor, capability/ACK/SSI, text delivery, GNOME or other-client policy. Existing ordinary native notifications provide updates; no added timer/retry/service. Source/build/install/physical gates pending. Full320 evidence unchanged100PASS/76FAIL/144NOT_TESTED.


### 2026-10-23T03:37:03.613127+00:00 — v134-r2 installed Chrome evidence and v135 review corrections

v134 initial compilation failed comparing the NativeSnapshot wrapper to a text snapshot; no installation occurred. v134-r2 repaired the comparison and selected72/72 source tests plus canonical architecture PASS. Receipt chrome-only-native-geometry-v134-r2-source/result.json; source closure82eba0235bfcedc74870b436bc0671f6396531a1c10fdab741b9d7b8a992f649. Installed only the owned development IME PID2503399, SHA94970cce6c2cea64bb1cbb6824d04a35c5320bdecc50db4d00e8e5adec02d542, display1.0.94.2. Activation/duplicate bytes match;20 protected processes, config, tray, daemon and global IBus unchanged. Receipt chrome-only-native-geometry-v134-r2-deployment/result.json.

Physical KDE Chrome only: the five-field focus/return cycle passes5/5, with actual prefix out of composition, fresh suffix/coordinates, exact Tab result and a single separator. Ten normal-window field types pass10/10; incognito text/rich/iframe-rich pass3/3. Hint observations: five-cycle50.909–79.064ms, ten-types45.450–107.771ms, incognito52.001–54.364ms; Tab24.172–45.281ms across these runs. These include probe/transport observation overhead, not isolated renderer latency. Screenshots prefix-0 and prefix-2 from the five-cycle were independently viewed: plain un-underlined prefix, joined visible completion. Receipts chrome-only-focus-cycle-v134-r2-native/result.json, chrome-only-ten-types-v134-r2-native/result.json, chrome-only-private-v134-r2-native/result.json. Ordered trace anchors are complete; config/engine/daemon restoration passes.

Fullscreen passes2/3 and FAILS iframe-rich: actual DOMпров/composing=false but stale suffixосто with rect=None. Three20ms read deadlines precede a later install_refused/field_identity_changed; application,bus,owner,field all match but canonical desktop lifetime differs. This is an observed timeout/lifetime rejection, not evidence of physical focus theft. Receipt chrome-only-fullscreen-v134-r2-native/result.json. Do not ignore desktop lifetime or weaken admission/identity to hide it. v134 is not final Chrome acceptance. Full320 matrix remains100PASS/76FAIL/144NOT_TESTED.

Harness2 task20261023T032145Z-c4544391 is still running (gpt-6.1-sol/xhigh). Its frozen base review gives6/10 with three independently verified source findings: the outer selection preflight still requests geometry, real cursor callbacks can skip missing geometry once readout is current, and geometry can finish after the last field/privacy checks. This preliminary review is not the task's final report and does not cover v135.

v135 corrections are confined to KDE native_document: propagate the geometry flag through the entire selection preflight (false); complete geometry and revalidate actual text/caret/selection before the final focused-field/privacy checks; allow an existing real cursor callback to read missing native bounds despite current readout. Same20ms bound, no timer/retry/extra observer, synthetic callbacks or authority widening. New private p2p AT-SPI test peers hold geometry while changing field/privacy/text/caret/selection, prove zero geometry calls for edit snapshots, and exercise bounded required-text/geometry failures. Existing managed fixture covers missing/ready/stale geometry callback eligibility. Source/build/installation/physical gates for v135 are pending.


### 2026-10-23T03:48:57.656314+00:00 — v135-r2 source, installation and fullscreen failure; v136 bounded observation trial

v135-r2:75/75 focused source tests and canonical architecture PASS; closure54b2385212ae01a829d8802d19ceb95b7fc4eaf2a0ac0094b12e10eb2ef5a19b. Privacy is checked before every text read, including post-geometry revalidation; held-geometry privacy race test asserts zero private text reads. Installed only the owned development engine PID2578781, SHA72d8856d177dba70ea0974e6881db49b3022150d3b40abc3fabd9205ff3d3037, display1.0.94.2.20 protected processes/config/tray/daemon/global IBus unchanged. Receipts chrome-only-geometry-order-v135-r2-source/result.json and chrome-only-geometry-order-v135-r2-deployment/result.json.

Physical fullscreen Chrome: text/rich pass with fresh suffix/native bounds/plain DOM prefix and exact Tab; iframe-rich FAILS with native_deadline then canonical desktop identity lifetime rejection. Complete trace and restoration; receipt chrome-only-fullscreen-v135-r2-native/result.json. Independently viewed prefix-0 screenshot: typed prefix ordinary and joined suffix, no underline. The full320 matrix remains100PASS/76FAIL/144NOT_TESTED. v135-r2 is not final Chrome acceptance.

Passive native bus timing identifies GetNSelections13.118ms and GetState/GetRole~7.2ms responses inside the same20ms total bound as two canonical KWin observations. v136 trial separates actual-document observation40ms from unchanged edit/subscription20ms. No identity bypass, retries, new wakeup, admission timeout or authority change. Decision2026-10-23-kde-chrome-observation-budget.json records the latency tradeoff and physical gate. Source/build/client verification pending. GNOME and all other clients untouched.


### 2026-10-23T04:26:27.550003+00:00 — v136 measured admission expiry; v137 Chrome-only timing trial

v136 source75/75 and canonical architecture PASS. Installed development PID2609412, SHA564b5c072b9b72f51fa4e20b16aa29bfda9a2ba6d6b5b300365b990230214dc9, source closurec8726c6e445dc20cdeb8b62c10e2db4f214ea7893595ffed179c939854e0775b. Protected20 processes/config/tray/daemon/global IBus unchanged. Receipts chrome-only-observation-budget-v136-source/result.json and chrome-only-observation-budget-v136-deployment/result.json under /home/e/projects/lay-development-runner/kde-install-control-20261020.

Physical Chrome: first fullscreen3/3, normal10/10, private3/3 and focus-cycle5/5 PASS; cold fullscreen repeat2/3 with iframe-rich FAIL. Total23/24, not complete acceptance. Its actual prefix remains composing/underlined; fresh inline frame is empty. The exact marker delay is61.488ms against50ms, not a geometry or identity rejection. Failure retained in chrome-only-fullscreen-cold-repeat-v136-native/result.json and prefix-2.png. Additional passive-wire fullscreen run3/3 PASS, chrome-only-marker-wire-v136-native/result.json; wire arrival and observer ingress agree within100us for the observed markers, but the expired marker was not recaptured, so internal IBus cause remains unknown. Never replace the prior failed receipt with this pass. Full320 matrix remains100PASS/76FAIL/144NOT_TESTED.

Harness2 USED task20261023T032145Z-c4544391 finished ok/ok,8 calls/no failures/no fallback, source unchanged, frozen-v134 review6/10. Report at /home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261023T032145Z-c4544391/report.json. Independent source verification confirms the three requested fixes in v135/r2: no geometry through selection preflight; real callbacks may refresh missing geometry; final field/privacy/text/caret/selection validation follows geometry, with private-text race tests. That frozen review does not certify v136/v137 and experience-library entries stayed empty.

v137 pending: reuse the same adapter acquisition with an explicit finite deadline selected only after a fresh canonical KDE observation identifies the actual Chrome executable. Chrome100ms; defaults/native-ID/GNOME remain50ms. The classification uses existing20ms native I/O bound; no retry/wake/queue/cache or new edit authority. Other KDE compatibility clients retain their deadline but may incur the additional bounded focus observation; their physical latency is outside this Chrome-only scope. Decision2026-10-23-kde-chrome-activation-budget.json records this tradeoff and supersedes only the earlier no-admission-budget-change statement. Source, installed-runtime and physical proof for v137 remain PENDING.


### Chrome activation request ordering trial — 2026-10-23T04:37:12.137353+00:00

v137 source: 157/157 selected checks and canonical architecture PASS; installed compiler artifact SHA-256 `52b5377dc6468e8a0cee968905b8835f9181593ce5f0b26daf1e88fa3e603e1c`, PID 2724331, version 1.0.94.2. Receipt: `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-activation-budget-v137-source/result.json` and `chrome-only-activation-budget-v137-deployment/result.json`. Config, tray and protected process identities preserved; only owned development IME replaced.

Physical fullscreen Chrome: 2 PASS / 1 FAIL, receipt `chrome-only-fullscreen-v137-native/result.json`. Text and rich field passed hint/Tab; iframe produced `ров` instead of `пров`. No activation expiry. Actual trace: first letter serial106 ingress Passive before FocusIn102 completed its timing classification; request10 was created afterward. Ready had no exact pending-literal receipt for that earlier letter, so installation discarded it. This is a new request-ordering regression, not a successful client acceptance. Historical browser matrix unchanged.

v138 trial moves the bounded KDE timing classification into the existing acquisition worker AFTER synchronous begin_request. The same request/reducer/Get/nonce/marker/expiry and literal receipt checks retain authority; no receipt is fabricated or relaxed. A delayed-selector regression must prove the first letter belongs to that acquisition and survives Ready. GNOME keeps the original no-probe path. Physical scope remains Chrome only; other clients untested. Source/build/installation/native verdicts for v138 remain PENDING. Decision: `docs/architecture/decisions/2026-10-23-kde-chrome-activation-budget.json`.


### Chrome queued compatibility focus trial — 2026-10-23T04:59:04.498150+00:00

v138-r2 source 158/158 PASS and installed SHA422e08ba7e8be8dfbfea9e7e4458ba123e4723dd252cb6efd89342a31c5d8ea8 are not client acceptance. Fullscreen 3/3 PASS, normal 9/10 PASS: iframe-input retains the full literal `пров` in composition but shows no suffix. Native trace requests36–39 are revoked during queued FocusIn/FocusOut processing, without expiry or accepted marker. First failed transition: a delayed obsolete FocusIn creates a request after a newer FocusOut ingress; final FocusIn revokes the duplicate pending request. Private/focus-return/cold-repeat were stopped and remain NOT_TESTED for v138-r2.

The real p2p regression reproduces the old defect (EXPECTED_RED, 2026-10-23T04:59:04.498150+00:00; exact receipt `/home/e/projects/lay-development-runner/kde-install-control-20261020/chrome-only-queued-focus-v139-red/result.json`). Trial: retain one reducer/stamp authority, publish focus metadata while holding its existing reducer lock, refuse superseded KDE platform-timing compatibility callbacks atomically before request mutation; keep the GNOME/default/native-ID API semantics and all marker/expiry rules. No new cache/queue/owner. Source, installed and physical verdicts for this trial remain PENDING. Historical failures are retained.

Harness2 task `20261023T042842Z-9b5aeab8` is USED, completed status ok, 8 calls / 0 failed / no fallback. Frozen v137 review HOLD6/10 found request-before-classification ordering and scope-cost concerns; it does not certify v138/v139 or client behavior.


### Chrome full lifecycle refusal trial — 2026-10-23T05:26:32Z

v139-r2 source160/160 and architecture PASS; installed development PID2860528, SHA926f63989037f718e6cbeb7842fe871ac3fe6f75ad41ddce64c40c4b2754b73d, source closurec37caef853d6021a7a2b95353d794aeca77fd9a61f0117e223892869a9417808. Chrome-only native24/24 PASS (normal10, private3, fullscreen3, focus-cycle5, cold fullscreen3), complete trace/restoration. Hint median51.23ms/p95 106.62ms; Tab34.73/44.84ms, including observation overhead. Initial iframe-input screenshot masks the last typed letter; follow-up screenshots at150ms/600ms show the full ordinary prefix and adjacent suffix. Preserve the anomalous frame; its cause is NOT established. Receipts: chrome-only-activation-budget-v139-r2-source/result.json, chrome-only-activation-budget-v139-r2-deployment/result.json, chrome-only-{normal,private,fullscreen,focus-cycle,fullscreen-cold-repeat}-v139-r2-native/result.json, chrome-only-v139-r2-iframe-pixels-native/result.json. The full browser matrix remains100PASS/76FAIL/144NOT_TESTED of320. No full-matrix or other-client acceptance.

Harness2 USED20261023T050346Z-c702884a failed its240s base call without a verdict. Task-local600s limit is used by still-running USED20261023T051104Z-f89a405c; its frozen base review HOLD6/10 flags stale full-callback cleanup. Independently verified actual caller: SupersededFocus still reaches finish_focus_in; missing metadata calls fail_context_activation against the currently installed owner. New real p2p full-lifecycle regressions FAIL3/3 on unchanged v139 engine source at2026-10-23T05:26:12.218939Z (chrome-only-queued-lifecycle-v140-red/result.json), proving successor-work cancellation.

v140 trial propagates a distinct ignored observation through the existing KDE compatibility FocusIn callback. Missing/evicted metadata and superseded ingress cannot revoke the current owner or run finish_focus_in cleanup. Existing bool/native-ID/default API and reducer/Get/marker/expiry authority remain; no new cache, observer, owner or delivery route. Three regression tests cover retained/revoked/evicted metadata with successor Pending, Ready and Installed, checking request/fence/ready/token and local work/snapshot preservation. Source/build/install/native gates remain PENDING. This trial changes shared lifecycle glue only for the KDE compatibility refusal; it does not prove other-client runtime behavior.

Errata preserved separately: prior prose red timestamp04:59:04 was the recording time; the actual v139 red receipt is04:57:17.535568Z. v138 deployment reused a review-field count157; its actual source count is158. First v139 compilation failed before installation; v139-r2 corrected the existing5-argument API wrapper and compiled160 tests. Historical receipts are not rewritten.


### Chrome v140-r2 focused development verification — 2026-10-23T05:40:52.712505+00:00

Chrome-only KDE development artifact1.0.94.2, PID2957531, SHA`b8249005a395273c44a35ba1b3c194e6060d8d37489d7f93eefae36c2c1ccd3a` installed. Source163/163 and canonical architecture PASS; native Chrome24/24 PASS across ten field types/private/fullscreen/focus-return/cold repeat. Ten first-ready screenshots show ordinary prefix and adjacent unadorned suffix. Hint median52.40ms/p95 82.99ms; Tab34.58/35.52ms. Full matrix remains100PASS/76FAIL/144NOT_TESTED of320; no other-client or release acceptance.

Full evidence and retained cached-test erratum: [owning display document](../architecture/ime-inline-hint-preflight-2026-10-20.md), section “Chrome v140-r2 focused development verification”; aggregate `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/shift-hint-20261021T1242Z/chrome-only-v140-r2-acceptance.json`. Harness2 USED`20261023T051104Z-f89a405c` finished8/8 calls/no fallback, frozen-v139 HOLD6/10. Both caller-cleanup concerns were independently reproduced and repaired in v140; no repeat review score claimed. GNOME-specific files unchanged, no global restart or push. Leave installed candidate for the user's Chrome check; do not revive other-client work without instruction.

### 2026-10-24 — field continuity full native verdict

Reviewed2100 installed;frozenfull220=172P/9F/39B,manual0FAIL.18oldF->P,
2oldP->F,1oldP->B retained. Nine current functional failures require causal
Space lease/verifier diagnosis;do not weaken safety or reapply disproved
transient-gate hypotheses. Frozenr19 two-word rich diagnostic1PASS.
Receipts:chrome-941-review-20261024/field-continuity-{summary,two-word-client}.json.
Upper real omnibox remains separateFAIL;two common hooks still unapproved.
No overall acceptance,no source green claim(old15FAIL+TD121error retained).

### 2026-10-24 — do not repeat rejected key-time Space producer placement

Exact RED run-pgO3io fails old slot after production native install;GREEN
run-fvKmhu52native passes. Yet frozenfull8a66=169P/10F/41B vs2100=172/9/39,
pair6F->8F,ENRU3F->2F;reject per aggregate AND class contract. Restored
compiled2100/PID3349193 and only two own Rust files;all801match old tested
source. New source/test preserved in packet/rejected-keytime-source.
Do not reapply it or increase waits based solely on its unit GREEN.
Current full proof172/9/39;post-rollback two-word1PASS separate.
Upper own suffix exists but rect:null at both positions;two common release
hooks remain unapproved. No duplicated scope question,agents or Harness.
