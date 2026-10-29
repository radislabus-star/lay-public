# KDE Firefox: local owned-preedit erasure and its delayed client echo

Preflight before source edits. One defect: ordinary active-word reverse toggle
after its next-letter/Backspace control. Current source/runtime and native22-row
failure receipt are recorded in `kde-firefox-native-result-map-2026-10-29.md`.
The separate plain-field Tab refusal is not this erasure mechanism.

## Existing authority and exact files

- `managed.rs` already captures `prepare_owned_preedit_local_backspace`, awaits
  the existing owned erasure/output and calls `finish_owned_preedit_local_backspace`
  only on success. Its output route stays unchanged; no edit planned there.
- `window_interaction/observation.rs::ManagedWordStartWitness` is the existing
  sole owner of the observed preedit beginning. Retain one inert erasure echo
  inside that witness, tied to its resulting tail epoch and original revision.
  Recording only after successful local output distinguishes an authenticated
  effect from a guessed word or a newly read native field. GNOME records none.
- New `kde/firefox/preedit_echo.rs`, attached by `kde/native_document.rs`, owns
  the client-specific presentation predicate and canonical KDE/Firefox selection.
  The exact echo is original external prefix + erased character + suffix at
  the original unselected caret. It never becomes an edit target or a grant.
- Existing actual observation ingress keeps that reported snapshot unchanged.
  A recognized echo makes the saved effect inert and manual mutation refused.
  Only the immediately following exact original external snapshot, same token,
  focus/lease/layout/epoch and adjacent revisions can settle this output effect
  and carry the same witness. Any other actual snapshot discards the effect.
  No new owner, output, timer, wait, field projection or delivery route.
- `active_composition_space_receipt.rs` reuses its current real managed-preedit
  fixture for semantic order/authority negatives. Canonical manifest and graph
  track the tests. Existing manual held-key and generic observation guards stay.

Native cause: row2 of CROOT/kde-firefox-native-result-eight-v1, successful
`привет -> ghbdtn`, native next-letter `z`, local Backspace; events342/346
report text1/caret0 then text0/caret0; refusal382/383 has birth revision4 versus6.
The source test uses generic fixture text, never a runtime literal exception.

Source progress: RED-v1 is semantic7/8, failing only final erasure confirmation.
GREEN-v1 passes that cause but the owner negative detects a missing shared
active-path recheck in the new effect reader. It was not installed. The existing
generic manual predicate alone does not validate the global active engine.
The new effect producer now checks both the local token/scope and shared active
path before preparing and settling its inert receipt. SHORT-v2 is8/8 PASS,
including all16 negative variants. Its source is1306files SHA
b1252c376227cda9d5a492e88011796c649fea0fc7705eecb440a4a215b8310e.
GREEN-v2 complete IME:1131/1131 PASS, short8/8, graph/architecture PASS. Three
performance tests were not run. Existing discovery rows retained, only four
correctness tests added. Final source1306files SHA
040fb2cfafa9dcdf44e217693f55bb36c602b97d2073ff9fbc73946ac51bd757.
Build/installation/native acceptance remain pending. No GNOME recording,
selection/output or delivery route was added. Receipt:
CROOT/kde-firefox-erasure-echo-source-green-v2/result.json.

## Acceptance order

Prove existing refusal with the exact two observations; then the authorized
local effect may retain only inert provenance until the full final receipt.
Assert authority remains absent during the echo, no KnownStart/learning/native
edit binding appears, and all stale/foreign/missing/intervening inputs reject.
Run short source checks, full affected IME, warning-free build and exact idle
component install. Short native active roundtrip precedes the unchanged22-row
control. Retain old failures and measure real release-to-visible latency.
