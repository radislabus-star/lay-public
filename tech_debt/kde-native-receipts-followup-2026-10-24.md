# Remaining native KDE acceptance — 2026-10-24

Priority: P1. Status: OPEN, not part of the accepted source-test PASS.
Owner: the next bounded desktop compatibility task, without Harness2/agents.

## Latest measured candidate100 —2026-10-26

Functional2725bcf4, exact1287file/mode closure9e1486e38397f45e824ffc4471649ce8e90f1b8fbc09abc0bf45f2347f1a426e.
Full unchanged r18: Firefox128P53F39B,Chrome174P4F42B of220each;
total302P57F81B/440. Relative98,12oldPASS-to-FAIL losses retained. A rejected
candidate108P74F38B Firefox and B1source3518P1F are not erased.
This task remains OPEN; see evidence/release-1.0.100-native.json for every
version delta, latency coverage and the separate CPU-envelope short controls.
Native trace completeness: Firefox204/220 rows,Chrome220/220; diagnostic
trace coverage is separate from actual native/DOM verdicts.

Five-scenario native PTY controls: Kitty5PASS,Konsole5PASS,WezTerm5FAIL.
Later WezTerm hint-only functional TabPASS has missing visible continuation;
visual FAIL and original functional failures remain OPEN in
[the bounded follow-up](wezterm-native-preedit-followup-2026-10-26.md).
Do not replace the original cases with this diagnostic or count native
browser rapid-five as equivalent to terminal five-with-settlement.

## Frozen evidence

Release1.0.96 source26d5ca24; final compiler and loaded byte identities are in
`evidence/release-1.0.96-verification.json`. Per-case verdicts and timing
coverage: `evidence/release-1.0.96-native.json`. Remote install-control root:
`/home/e/projects/lay-development-runner/kde-install-control-20261020`.
Original actor is frozenr18 SHA256
`17dc93b5f2769d5dde62bfe8f5a539fb68af40744139e823c5f18a848b4ce7e6`.
No source-only PASS certifies these physical client paths.

Chrome:168PASS/10FAIL/42BLOCKED of220. Manual Double Shift has154PASS,
26BLOCKED,zeroFAIL. All10FAIL are Space actions:3 layout conversions and7
moved-boundary corrections. Two retained witnesses show stale_lease and a
prepared decision arriving after Space. This proves the observed ordering,
not a license to reuse stale authority or increase waits.

Firefox:81PASS/103FAIL/36BLOCKED of220 with accessibility enabled only for
the owned Firefox process. In the default session the native observer cannot
identify the application; that first pre-input BLOCKED receipt stays separate.
The known witness accepts a genuine completion with Tab, then leaves
`проверка ` unchanged after Double Shift. Keep the output refusal, native
focus/caret and application identity evidence; identify the first production
observation/admission failure before changing code. Do not classify all103
as one proven cause solely because their final text is unchanged.

GTK/Qt25/25 and terminal20/22 are short controls, not a replacement for either
browser grid. The two RU punctuation failures reproduce as `!,Ж()_` instead
of `!?:()_` with ordinary xkb sources and no Lay engine. Preserve raw FAIL and
the no-Lay control; qualify the RFB/XKB input prerequisite before attributing
this to Lay. Physical evdev, real GNOME51 and OrangePi remain NOT_TESTED.

## Minimal next steps

1. Group the Firefox failures by their first missing production receipt,
   separately from default accessibility setup. Inspect retained logs first;
   keep DOM/client, native source, reducer token and verifier boundaries apart.
2. For Chrome inspect the complete InputFrameIdentity mismatch in one retained
   failed Space witness. Compare the existing producer and lookup identities;
   keep candidate retention and edit authority independent. The rejected
   key-time experiment in the owning desktop document must not be repeated.
3. Prove the first shared mechanism with a RED regression and revocation/
   stale-owner controls. Reuse existing reducer/pending-completion contracts.
   No new delivery route, arbitrary wait, word exception or client bypass.
4. Run that mechanism witness and original Shift/next-letter controls, then the
   unchanged full grid. Record exact changed bytes, setup assumptions, misses
   and latency. Retain prior failures; do not retry unchanged until PASS.
5. Treat #47's GNOME login/Space residual independently. Its attached settings
   pass12/12 core controls and login ordering is repaired with #69. Current
   reporter reproduction or a supported actual GNOME session is still needed
   for that residual; KDE failures alone do not prove the same GNOME cause.

## Acceptance

An implementation needs an explicit source change, a source proof and a new
native receipt for changed bytes. Full compatibility remains unaccepted while
functional FAIL persists. BLOCKED and NOT_TESTED are separate from failures and
from PASS. Report fixed denominators and per-mechanism improvements/regressions;
source or selected examples cannot hide a native regression.
