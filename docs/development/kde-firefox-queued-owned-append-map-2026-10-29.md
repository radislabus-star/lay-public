# KDE Firefox: queued owned appends before a delayed caret echo

Canonical base b835ca47; source checkout lay-dependency-updates-20261027. Current
loaded daemon515781/0a71fd83 sourcec14dc808; IME4178957/ebd750a0 source3a09d0a0.
Solo/noHarness/GNOME changes/publication. FullFirefox126P55F39B,Chrome earlier
179P0F41B; native ordinary22 on current daemon20P1F1B. All actual raw native
counts/failed candidate receipts retained; no full run after failed short gate.

## First failed transition, independently saved clean evidence

/home/ubu/.cache/lay/development/browser-matrix-repair-20261027/
raw-word-pacing-iframe-tab-failure-row20.jsonl, underlying native row20 complete,
zero malformed lines. Source trace rowstart1479535.066960469.

1. Actual prefix4/caret4 confirmed; receipt advanced_confirmed.
2. Successful owned CommitText(e), then CommitText(t) before another SSI;
   callback serial89040 at1479535.833659060 then89042 at1479535.835709277.
3. SSI5chars/caret4 arrives. Receipt rejected second_surrounding_receipt.
4. Actual5/5,then6/6 cannot restore discarded lineage.
5. Space, then handled local preedit pri with visible suffix. Current snapshot
   has7chars while whole local word extends to10; start witness unavailable.
   Native Tab callback89061,stage tab,handledfalse. No Double Shift yet in row.

This is before any daemon pacing operation in the failing row. The raw-word
pace candidate remains unpromoted; do not blame/alter its output from this gap.
Wrapper later failed parsing unrelated row22 first line; preserve failure and
refuse that trace for causal claims. This does not invalidate clean row20 or
native counts, whose controller independently wrote all22 rows.

## Map and exact code boundary

```text
managed native CommitText -> observation.rs::advance_context_reset_rereceipt_after_key
  existing sole PendingContextResetRereceipt.owned_append_actual_before
 -> native SSI selector observation.rs::kde_firefox_owned_append_caret_observation_pending
 -> kde/firefox/preedit_echo.rs pure whole-snapshot predicates
 -> observation.rs::observe_context_reset_rereceipt_surrounding_text
 -> full current actual receipt -> existing Tab/manual admission (UNCHANGED)
```

Runtime only observation.rs and existing kde/firefox/preedit_echo.rs; tests in
existing context_admission/adapter/tests/active_composition_space_receipt.rs.
All cumulative prior paths preserved in active scope. No attrs/display/commit/
output/authority edits, no general whitespace equivalence or GNOME behavior.

Minimal design9/10: retain the original actual cache only through already verified
same-owner/token/word/epoch, one-character handled appends. It remains inert.
Old-client caret echo may name a strict earlier prefix of the currently owned
word. Reuse exact actual field bytes + existing word-boundary predicate; require
no selection, nonregressing same-field observation. Intermediate end-caret prefix
may advance only inert revision state; only exact full actual target confirms.
Existing completed-native-target progress is the neighboring reference; do not
invent a parallel state machine. Reject altered prefix, duplicate/regressing
observation, owner/layout/focus/input gap/selection, unrelated client or desktop.

First source regression: reuse compose_owned_append_before_caret_echo, add one
owned character without SSI, then earlier caret echo -> actual prefix -> full
target, plus direct full-target variant. Existing single-append controls must
remain. Add negative safety cases and Unicode/external-prefix checks before GREEN.
Source gates/build/native acceptance separate; no installation before proof.

User additionally reports Tab field4(tel),first word8(plain),now6(Textarea),and
plain-looking suggestion. Source fixture title-case inputs pending native probe.
Fix receipt first; inspect/style only projected suffix afterward in KDE Firefox.
Artifact itself disappeared per user. Passive60s143samples saw no current hint,
only2 framebuffer changes, no stuck hint on direct KDE frame. Cause UNKNOWN;
not a fixed rendering claim. Original user fields never edited/cleared.

Measurement-only lead NOT VERIFIED: frozen RFB actor has no TCP_NODELAY; its
latencies use local send time, not independently captured X11 release. Queued
callbacks2ms apart despite24ms requested key spacing warrant transport-clock
validation, not a silent actor/cadence change. Keep frozenactor360e6 and full
matrix counts; no performance PASS by changing measurement conditions.


## Source experiment, 2026-10-29

RED-v1 selects16,15PASS/1expectedFAIL at the actual SSI selector after two
handled appends. Runtime unchanged. SHORT-v1 compile error from a new test
accessing a private inert-cache member was corrected by testing a genuinely
missing existing receipt; production visibility not widened. SHORT-v2=18/18PASS:
new positive direct/full and intermediate/full sequences,16 refusal variants,
Unicode whole-field checks; all previous controls retained. No text effects
before independently observed full latest field. Source1308files SHA
bff5bcf3599893a6a8a20a018b82346d1787a667923b0093bf425283c5bdbd3c.
Receipts CROOT/kde-firefox-queued-owned-append-{red-v1,short-v1,short-v2}/result.json
and corresponding CACHE/*-result.json. Next full IME source/graph/manifest gate;
installation and native acceptance still NOT TESTED for changed IME bytes.


GREEN-v1 actual functional gate1140/1140PASS, graph PASS53.832s,18/18short.
Final registry assertion refused FOUR additions: three new queued regressions
and pre-existing ignored live_kwin_focus_observation_latency, included by both
binaries but registered previously only for daemon. Its source was already in
sourcec14 and is unchanged here. Retain this failed gate receipt. GREEN-v2
finalizes exact registry delta (3correctness+1ignored,0removed,0changed), reuses
that same frozen1140-functional proof; no repeat to obtain green and no ignored
benchmark claimed as executed. Native/browser acceptance still pending.


Pre-install self review added changed_final_after_prefix to existing refusal
identity (17variants). Genuine external-RED-v1=17/18: rebasing armed_revision
alone could admit changed external context after intermediate prefix. NOT
INSTALLED. Fix shifts the existing adjacent armed/echo revision pair instead;
next exact_after_managed_prefix_echo still compares the whole previous field.
Pure monotone predicate also recognizes prior end-caret plus growing actual text;
duplicate stable prefix remains rejected. SHORT-v3=18/18PASS, source59d35ea2,
all earlier safety controls intact. Repeat full because runtime source changed.
Registry issue is resolved with exact3correctness+1preexistingignored delta;
new GREEN-v3 must now discover no removed/changed identities.


Final GREEN-v3:18/18short+1140/1140correctness/package PASS, graph53.537s,
registry MATCH with no further identity delta. Source1308files SHA
d8a90c49e41c1a24ed0a3295c3951888e55fc27296829a6b453a6f897cbe14b4.
No new runtime byte installed yet. Build/activation candidate only, not release.
Prepared unchanged Tab-v3 actor16cases includes Textarea; ordinary22 gate and
frozenfull220 follow. No widening original cadence/assertions.
Appearance source and current running tray script verified: black on white,
border:none. Runtime tray script SHA3f936370408e513b9ed764f087713a04e9521d9b31e475f5d53267c05be04e10.
White browser field can merge visually; capture actual suffix before a separate
KDE/Firefox presentation-only change. GNOME script/runtime untouched.


Installed exact IME698174/fbf3808d379298d9e907285f6f3e1908a2f4101852833bd55d41d43d93c6bb9e,
zero-warning release build74.497s; idle factory-only activation. Daemon515781,
IBus/tray/userFirefox/config unchanged. Deployment queued-owned-append-deployment-v1.
Native16Tab=16P0F0B,21.598s,text/tel/textarea/plain,first pri/prov/p and second pri.
All four Textarea rows PASS on native RFB input; NOT proof of old user profile.
Protected runtime/config exact restored. Ordinary22 next,full220 not yet rerun.
