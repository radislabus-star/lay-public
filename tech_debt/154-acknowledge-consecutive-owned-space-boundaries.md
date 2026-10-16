# TD-154 — Keep exact receipts for consecutive owned Space boundaries

Status: SOURCE_NATIVE_INSTALL_DONE_1.0.89, publication pending; independent review9/10. Priority: P1, before further optional
shortcut compatibility changes. Baseline: fe614235967b4b7779085b5e784b9c5c7a2ebcde,
released/installed 1.0.88. Root implements; independent review, at most two passes.

## Measured failure and first shared transition

The complete, immutable optional normal-window matrix for the released bytes is
58 PASS / 18 FAIL / 4 BLOCKED_PRECONDITION out of 80. Double Space contributes
26 PASS / 14 FAIL out of 40. This is separate from the default browser matrix
349 PASS / 50 FAIL / 33 BLOCKED out of 432, NOT_TESTED=0.

Six Double Space failures are Firefox fields 7, 8 and 10, both space_pair and
space_rapid10. The first pair leaves the Russian word and two actual NBSPs;
no target conversion is observed. The preserved trace for field 7 rapid10 shows:
first Space commits ASCII U+0020, Reset and exact SSI acknowledge U+00A0, and
the existing receipt confirms it. Second Space commits U+0020 after that NBSP,
then Reset and the immediately adjacent exact SSI have matching identity,
length, cursor and no selection. The receipt is nevertheless discarded as
surrounding_receipt_mismatch; ManualToggleV3 then refuses context_authority.

The first source mismatch is acknowledge_owned_space_boundary in
src/bin/lay_ibus_engine/window_interaction/observation.rs. It requires the
character before the newly appended Space to be a word character. Thus the
existing exact acknowledgement supports one closing Space, but not a second
owned Space after an already acknowledged separator. The existing
acknowledge_owned_space_tail updates local/shared exact surface copies.
The source regression must establish this causally before runtime edits.

Evidence (original files remain unchanged):
- /home/ubu/.cache/lay/development/td151-install88-20261016/native-shortcuts/firefox/firefox-shortcuts88-ntzkcns9/RESULT.json
- /home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/firefox-shortcuts88-ntzkcns9/ime-trace.jsonl,
  SHA256 35041eb52e4453b42f44ebc3959f4e146073758f58d562bd29f9d8a3f5066f2f.
  Relevant preserved lines 15–60; first pair itself has rolled out of this
  bounded trace, rapid10 remains. Trace completeness remains UNKNOWN.
- Verified read-only copy:
  /home/ubu/.cache/lay/development/td154-shortcuts-diagnosis-20261016/.

## Options, scores and selected minimum

1. **9/10, selected:** keep the existing one-character, first adjacent client
   acknowledgement. Permit its unchanged prefix to end in previously retained
   ASCII Space/NBSP separators after an observed word. Compare the complete
   unchanged prefix exactly, adopt only the last newly appended U+0020 as
   U+00A0, and retain all owner/revision/token/epoch/selection/cursor/replay
   guards. No new state, normalizer, timer or executor.
2. **3/10:** normalize all whitespace in runtime snapshots or exact-tail plans.
   Rejected: loses exact surface/proof correspondence and can admit external
   edits or incorrect deletion lengths.
3. **1/10:** Firefox-specific trigger or delivery fallback. Rejected: creates
   another owner/route and does not repair the shared observation contract.

## Implementation and TDD

Extend the existing actual legacy callback regression for acknowledged NBSP,
retaining its original single-Space cases. Drive two and multiple complete Space
press/release callbacks, real Reset order variants, and successive exact SSI
receipts. After each, assert actual retained surface, unchanged acknowledgement
epoch, local/shared agreement, current exact manual admission, zero text effects
from receiving SSI, and authoritative VisibleTailV2 with exact raw separators.
Use at least two words/lengths; fixture strings never become runtime conditions.

Require a genuine baseline RED at the second acknowledgement, then the minimum
production change. Cover mixed already observed ASCII/NBSP separators without
rewriting earlier bytes. Preserve rejection of unobserved additional spaces,
changed prefix, tabs/narrow NBSP, selection, interior caret, revision gaps,
focus/content revocation, missing/mismatched local/shared owner and replay
provenance. Existing adverse tests remain authoritative; do not remove a guard
just because the positive test fails. One physical append cannot certify two
new separators or normalize a previously stored separator.

Affected invariants: C01/C03/C04/C05/C07/C08/C09/C10. After causal RED and before
production edits, add a new explicit decision under docs/architecture/decisions/
covering observation.rs (and tail_memory.rs only if actually changed). These are
protected paths in check_architecture_canon.py; no earlier decision authorizes
this change. Record exact scope, RED/negative evidence, untested native boundary,
unchanged routes and the new limited runtime authority. Do not modify canon or
its guards to obtain PASS. Current owner remains the
existing inert Reset receipt and exact-tail handoff. Runtime authority changes
only by retaining an actually acknowledged consecutive owned append; no lexical
model, learning, config/default, source selection, verifier or delivery changes.
Review the previous acknowledged-owned-Space and TD150/151 replay controls.
Record every experiment in the TD121 ledger and owning document.

## Native proof and independent measurement pitfalls

The 14 old failures are not one bug. Two Firefox input/search cases also have
a pending Space-release ingress between receipt consumption and manual handoff;
their causal admission defect remains to be proved separately. Four Chrome rich
editor cases observe the correct projected word plus two NBSPs, but the actor
expects two ASCII spaces. Two Chrome search cases pass every requested pair,
then fail a next-letter Backspace return with a display suffix beyond the caret.
Do not count these six Chrome cases or the two Firefox races as fixed by this
runtime change, and never rewrite the original FAILs.

For new native proof, any actor adjustment must preserve exact whole committed
text, separator count, caret, no selection, actual mode, next-letter decoder and
latency from the final physical release. A documented rich-editor presentation
comparison may permit only U+0020/U+00A0 at expected separator positions; retain
the raw snapshot and do not normalize runtime authority. It cannot accept a
wrong word, missing/extra/changed separator, selection, middle caret or arbitrary
suffix. Display suffix treatment must have independent proof; leave it FAIL
otherwise. New actor identity and old receipts stay separate.

All executable tests/builds/graph/native keys run on e@192.168.3.94 under the
existing shared resource lease. First focused real adapter/adverse tests, then
controlled reversion, full affected source/MSRV/lint/canon gates, independent
code review (max two passes), canonical graph and matching build. New runtime
bytes require the complete default ten-field Firefox/Chrome protocol and new
optional native evidence with latency; never splice partial matrices or retry
unchanged bytes until PASS. Normal/private/fullscreen optional coverage must
remain separately labelled, with unrun modes NOT_TESTED. No local functional
input, RDP, global IBus restart, changed data/models or parallel heavy/native job.

## Completion and rollback

Only mark DONE after the causal test, source gates and independent review; report
native compatibility separately with exact new receipts and remaining failures.
Commit/push the reviewed task, retain failed evidence and install a matched
release only through the existing reversible transaction after required gates.
Rollback is the exact previous matched payload set/config; never erase learning.
No accepted version is rebuilt/retested merely to publish documentation.

## Causal source baseline, before runtime edits

td154-consecutive-space-red-bcubbzg9/RESULT.json: 1/2 PASS,14.724s,
archive6c6668282e4eaf7f3d69a854b6b117f11ee9e2732e566f5bcc942d00c16edd00.
Existing adverse test PASS. Extended actual legacy callback test fails exactly
at boundary index1 of two NBSPs, Reset=false: stored abc+NBSP+ASCII differs from
the exact observed abc+NBSP+NBSP. Original single-Space cases pass first.
No runtime edit, missing fixture prerequisite or timeout caused this RED.
New decision:2026-10-16-consecutive-owned-space-acknowledgement.json.

Fresh-context independent plan review:8→9/10, pass2/2, reviewer
/root/td154_consecutive_space_plan_review. Missing explicit new protected-path
decision/C03 step corrected before production. No remaining P1/P2. Root alone
implements. No new native input or installed payload change.

First implementation td154-consecutive-space-green-zftayg2t:10/10 PASS,17.231s,
archiveeb033647991a3fc7e2573fa98574cc0ae2a9fed7ea1437f86aee7c9e8751e3f9.
Two word lengths,1/2/4/10 retained boundaries, both actual Reset schedules,
mixed observed separators and six new contradictory consecutive receipts pass.
Previous Space/reset/replay and Atomic four-Space controls pass. Runtime change
is one expression in the existing acknowledger plus explanatory comment.
Version1.0.89 is prepared for a new matched build, not yet published/installed;
Cargo.lock changes only this package version, no dependency version.

First full preflight td154-full-source-_7ih78_h stops before full lanes at
strict Clippy: the new test's explicit MutexGuard drop is not accepted as a
lexical end before await. Change it to a lexical block, without allow/suppression
or runtime changes. Its controlled one-expression reversion detected the same
second-boundary failure; MSRV checks passed. This stopped preflight remains
FAIL and does not establish a full source result.

Final full source td154-full-source-final-50j7ij8d/RESULT.json:3211/3211 PASS,
3175 correctness+36 package, zero failures/exemptions,715.353s including prechecks
and controlled reversion. Archiveb2793f28a7aad540faf1e1ea0ad279d66d86931b4a35b232911a64411a4d96ca.
All3237 manifest IDs/classifications unchanged (15ignored/11performance remain
separate). Actual MSRV1.88 in both configurations, strict lints with unchanged
baseline, canon15, architecture/audit50, Node/harness122 (one explicit skip),
panel syntax PASS.994 tested functional inputs match local, including the
verified generated graph receipt copied back. Controlled one-expression
reversion detects the intended boundary1 assertion; exact source restored.

Private native observer contract:17/17 PASS on the remote resource guard;
actual matching_snapshot/decoder_snapshot/target functions, including wrong
word/count/caret/selection/mode, tab/narrow NBSP and post-Backspace ghost refusal.
This tests observation predicates, not native client behavior. New helper hashes
and raw results remain separate from old receipts. Build/install adapters reuse
the existing exact payload transaction and direct complete native selection,
not the old byte-equivalence shortcut (runtime bytes change).

Fresh-context independent code review:9/10, pass1/2 accepted, no P1/P2,
/root/td154_consecutive_space_code_review. All18 private helper SHA identities
match REVIEW-INPUTS.json SHA256c0055d38866c40b7b54b222ded578b8606bd16fca09d3b9fe77b02c78fbc4e8b.
An initial collector-index concern was withdrawn after checking the actual
specialized SHORTCUTS route; no code changed for that concern. Root implemented
all changes; no extra review pass is needed without a new change/finding.

Final canonical td154-final-canonical-ws_l_m_4/RESULT.json PASS48.392s,
993 functional inputs equal the full proof,18 helper/embedded-code AST identities
matched; generated graph fetched from the remote workspace. Source/graph gates
are complete. New exact-byte native432+80, source-main CI and reversible install
are still required; no new physical result or installation is claimed.


## First changed-byte native result — preserved failure, next owner isolated

Build source76361affeef7bb0cb9fbe35c1bf9293137b7156b/archive4088d1308056524d5f74643603f0f909d7fc05967dfe61e80f1b85aa2fc04613,
1.0.89 candidate,203.644s. Not published or installed. First complete optional
Firefox normal40:25PASS/12FAIL/3BLOCKED_PRECONDITION. Other new native modes,
Chrome optional and default432 NOT_TESTED. This is not an improvement claim.
Receipt: /home/ubu/.cache/lay/development/td154-install89-20261016/native-shortcuts/firefox/RESULT.json.
Double-Space latency30/60 samples:median112.400ms,p95137.124ms;30missing.

Preserved field7 rapid10 trace firefox-shortcuts89-hi06zvxw/ime-trace.jsonl
SHAe532cd41ac37b38780243c265a9310edc3cfba2f9d2646c3038ed6f599bdb4e8:
lines24/46 acknowledge both Spaces,25/47 confirm receipts,56 consumes,
57 delegates observed_suffix_ime_committed_tail,59 RPC status3. This confirms
the repaired acknowledgement/handoff, not a successful text replacement.
Daemon log firefox-optional-daemon.log SHAe62067fd5d875d8191b97930dc4752dde8b3f06d56253ae4261c2dfc1fc46d97
then rejects: exact replay requires preserved ASCII spaces or one acknowledged
Space. First remaining shared transition is prepare_exact_replay in
src/bin/lay_daemon/exact_ime_tail_replay.rs; TD155 owns that separate consumer.
Do not revert TD154 or relabel this failed native receipt. The bounded trace
is incomplete outside the retained interval; asynchronous completeness UNKNOWN.


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
