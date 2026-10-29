# KDE Firefox: a delayed echo after erasing the last local preedit character

Observer correction: some eight-gesture row traces rotate at512KiB. Absence
of a new event in those traces does not prove global nonexecution. Withdrawal
stands because native acceptance was not established and the measured failed
plain Tab row stopped before the proposed erasure mechanism. Earlier stronger
nonexecution wording below records an unsupported interpretation, not a fact.

VERDICT: NOT PROMOTED. Source1134/1134 and native short2/2 do not establish the
live root. Retained22 on these bytes is19PASS/2FAIL/1BLOCKED; no new
`ibus_owned_closed_word_erasure_echo` event occurs in either native run. Plain
fails earlier at Tab, before any probe/erasure: after handled CommitText of the
sixth letter, actual SSI has6 characters/caret5, then Reset and exact6/6. Its
old confirmed append receipt is discarded at the first SSI. URL fails before
its first gesture; it is not proved to share the plain mechanism.

The unaccepted source/test/manifest overlay was withdrawn locally with exact
before/after hash checks, preserving the native-grab fix and every older dirty
change. Audit/cache archive: `empty-preedit-experiment-withdrawal-v1.json` and
`rejected-empty-preedit-production-overlay-v1.tar.gz`. Do not reintroduce it
without new native evidence for that missing premise. Installed IME0a26... is
still the diagnostic candidate until the next verified replacement; local and
remote source diverge deliberately during preparation. No release/publication.
New active owner: `kde-firefox-owned-append-caret-map-2026-10-29.md`.

Historical experiment follows; its source PASS is restricted to its own bytes.

Current bounded task; preflight recorded before implementation. Continue solo,
without Harness or agents. Installed daemon/IME stay unchanged during source
experiments. This is explicitly a KDE/Firefox field integration task, not a
correction-only change.

## Verified baseline and first failed transition

Canonical checkout `lay-dependency-updates-20261027`, branch
`codex/browser-matrix-repair-20261027`, base
`b835ca478198d6cb765a2246329d5646966c118c`; source closure1307 files
SHA50938ebe30ad78da2d90389893b75d35273a80017e799fb018c949a02ec10ebf.
Remote KDE:71 has daemon2374910/ae77e705ba6795cfb99f1b074b027a70e2f7277c81f480380d969880fd07d644
and IME2067215/bad527220d1034e51444c567770511794f6b3c43c439599484698b84651d9bac.

Unchanged retained22 native diagnostic now20PASS/1FAIL/1BLOCKED, versus
19PASS/2FAIL/1BLOCKED previously. Both previous failures passed; the remaining
failure is plain:post-tab-eight:active, transition2. Its first target and decoder
probe passed. Native field remains focused, editable, unselected, caret14.
This separate owned Firefox profile does not replace same-user-profile proof.

Evidence root:
`/home/ubu/.cache/lay/development/browser-matrix-repair-20261027/kde-firefox-after-freeze-retained22-v1`;
failed-row trace `firefox/row-15-native-trace.jsonl` has an intact row anchor.
The whole-run trace rotated and is explicitly incomplete; do not use it as a
complete causal transcript. The outer `*-verification.json` pins both loaded
binaries, preserves IBus/tray/user Firefox and exact config, with coherent RU/RU.

The existing processed native target is followed by local preedit `z` and an
authenticated handled Backspace. Empty-preedit return restores the earlier
confirmed closed-word boundary. Firefox then reports15 characters/caret14
(erased character after the caret), followed by the actual14/14 field. The first
callback discards the receipt as `second_surrounding_receipt`; the following exact
callback cannot revive it. Next ManualToggle is refused at `context_authority`.

Existing nonempty-preedit echo retention intentionally requires a remaining
composition. Existing last-character return tests exercise no new callback or
Reset followed by an exact callback, leaving this actual intermediate echo out.
Add a failing source regression before extending any runtime behavior.

## Dependency map and scope

```text
managed.rs::prepare/finish_owned_preedit_local_backspace (UNCHANGED output)
  -> observation.rs::capture_owned_preedit_erasure_reset_rereceipt_candidate
     -> advance_context_reset_rereceipt_after_key (existing sole receipt)
        -> KDE/Firefox delayed presentation predicate in kde/firefox/preedit_echo.rs
           -> observe_context_reset_rereceipt_surrounding_text
              -> authority.rs::manual_toggle_v3_inner (UNCHANGED admission)
                 -> existing daemon/executor/output (UNCHANGED)
```

Exact runtime seams allowed: `window_interaction/observation.rs` and existing
`kde/firefox/preedit_echo.rs`. Tests remain in the existing focused
`context_admission/adapter/tests/active_composition_space_receipt.rs`, using the
existing parent fixture for successful last-character erasure. Test manifest and
generated source graph/evidence may update through canonical tooling. No new
text owner, source projection, RPC, queue, timeout, input or delivery route.
All previously verified dirty changes remain in the declared cumulative scope.

Preferred solution: retain one inert exact erased-preedit echo in the already
authenticated closed-word receipt (9/10). Bind original full field, erased
character, resulting epoch, owner/word lineage and consecutive revisions. The
echo grants no manual, Space, Tab or mutation authority; only its immediately
following exact full actual field may reconfirm the same receipt. Reject a
second echo, changed prefix/caret/selection/owner/layout, interruptions, missing
successful local erasure, unrelated client or GNOME. No callback deduplication
or extra refresh is required when the client sends no echo.

Alternative: reread native text and recreate an owner (2/10, rejects existing
authority contract). Alternative: add wait/retry (3/10, cannot repair discarded
identity and repeats a rejected timing approach). Neither will be implemented.

## Verification and protected boundaries

1. Genuine RED: original production reducers, successful local single-character
   cancellation, intermediate15/14 echo, final14/14 actual field and manual
   bridge/VisibleTail proof. No literal fixture condition in runtime code.
2. GREEN short: exact ASCII/NBSP separators, inert intermediate state, full
   prefix/context integrity and negative/interruption variants; previous owned
   nonempty echo and last-character-return tests retained.
3. Full affected IME correctness/package lane, canonical manifest and architecture
   graph,0warning release. Do not repeat unchanged daemon checks without cause.
4. Install only matching idle IME, verify loaded bytes; preserve daemon, IBus,
   tray, actual user Firefox and config. Short native plain/rich/textarea controls
   precede the unchanged retained22 and later broad matrix. Keep failures/blocks.

GNOME runtime/config/behavior, Chrome integration, correction decisions,
verifier, native-grab fix, layout contracts and executors are protected. Shared
observation code is a declared seam; a KDE caller alone does not prove isolation.
Historical full440 matrix306P55F79B is not rerun or recounted by this preflight.
Source, installed bytes and native acceptance remain separate verdicts. No
publication or installation is authorized by a source PASS alone.

## Source experiment

RED-v1 is genuine:9 selected,8 passed; the new closed-word return regression
alone fails at final actual-field confirmation. Existing8 focused controls pass.
Production code was unchanged;15.16s test compilation. Source1307 files
SHAb182ff174cdd15edf1f7a78595b96d927fb071fec0a4437e44e7c5c975241b0f.
Receipt: local cache `empty-preedit-source-red-v1/result.json`, remote
`kde-firefox-empty-preedit-source-red-v1/result.json`; no installation.

Implementation records the original whole field, erased character and existing
focus/owner/layout identities only after the successful local final-character
return. It reuses the existing KDE Firefox echo predicate. Inert metadata lives
in PendingContextResetRereceipt, not a parallel owner; all other constructors
clear it. The first exact direct callback consumes eligibility and follows the
old policy. One exact echo unconfirms the receipt; its immediately following
whole actual field can confirm it. Any contradiction spends the proof, preventing
later suffix-only resurrection. No product route or admission change. GREEN and
native acceptance are pending.

SHORT-v1 did not compile: the included `native_cursor_observation.rs` has two
additional constructors of the same receipt. It requires mechanical
`owned_preedit_return_echo: None` initialization in both. This exact seam is
added to scope/ADR before that repair; no cursor-edit admission/output logic is
changed. Its compiler failure is preserved as a build failure, not source RED.
Review also adds the existing shared active-path owner check and an inactive
path refusal alongside the focus/lease/layout negatives. No installed change.

SHORT-v2 compiled in15.05s:10/11 focused tests passed. The original positive
regression now passes; the negative test identifies a Reset between echo and
actual field transferring the newly retained inert receipt into the old Reset
seed path. Before this feature that echo had already discarded the receipt.
Retain the negative assertion: the new unconfirmed echo is now explicitly
ineligible to become a Reset candidate. Existing confirmed/no-echo Reset policy
and all other reset seeds remain unchanged. This was found before installation.

SHORT-v3:11/11 PASS, including8 retained controls and3 new semantic tests.
The negative test covers18 refusal/interruption variants with both ASCII/NBSP
separators (36 subcases); the direct-exact test covers both KDE/GNOME and both
separators. No tests/assertions were weakened to pass. Source1307 files
SHA430294e42c52be3e49769ef1976a214657600a646e76c80d60294ba113ad6fd8.
Receipt local `empty-preedit-source-short-v3/result.json`; full affected IME
lane, canonical graph/manifest and release/native verification remain pending.

GREEN-v1:1134/1134 IME correctness/package PASS; short11/11 overlaps that
total and is not added to it. Canonical graph/architecture PASS; exactly3 new
test identities added, all previous rows unchanged. Three performance tests
were not executed. Final source1307 files
SHA5df213b877c535632454b4760a6af4d3a21013b72df44036b6f9c2bb0a61f4cd.
Local receipt `empty-preedit-source-green-v1/result.json`; raw remote
`kde-firefox-empty-preedit-source-green-v1/`. The baseline-relative review diff
is `empty-preedit-source-review-v1.diff` in local cache; it preserves all old
production conditions and adds only the declared receipt seam. No independent
agent review performed (solo/no Harness constraint). Next:0warning IME-only
release, preserve/restore old activation cache during build, then matching
idle IME-only installation and native controls. Daemon, IBus, tray and user's
Firefox must retain PID/start time/hash; local GNOME remains untouched.

## Build and loaded runtime

IME-only release PASS,0warnings,74.374s. Candidate/installed/loaded SHA
0a26d84e418a495b1fa13d91ed6a1745cf3a453cd3e0d17b85641bed8ff374f4,
new enginePID3669264; source matches GREEN-v1 exactly. Existing guarded idle
component activation was reused; OwnsActiveText=false before replacement, no
visible hint, saved source/layout retained. Daemon2374910/ae77e705..., IBus3168326,
tray3228081, actual user Firefox988548 and protected desktop processes retain
start time/hash. Exact config6e62eab3... unchanged, no global IBus restart.
Backups of both original activation/installed files are in
CROOT/kde-firefox-empty-preedit-deployment-v1/. Base1.0.111 unpublished; no local
GNOME install/restart or publication. Native short verification is in progress;
this installation receipt alone is not client acceptance.
