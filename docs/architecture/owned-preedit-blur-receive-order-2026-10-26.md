# Owned preedit blur: retain received lifecycle metadata

Owner: existing ContextAdmissionAdapter/ContextAdmissionReducer and
WindowInteraction. Scope: second measured mechanism of candidate1.0.100.
No delivery, presentation, model, deadline or mutation verifier change.

## Evidence and first failed transition

CandidateA full frozen Firefox matrix108P74F38B/220 is rejected relative to
98(121P63F36B). ChromeA was not run. Native row62 shows Factory and already
received FocusOut/Disable/FocusIn; later callbacks lose Disable/FocusIn stamps.
Bridge marker finishes in1.1ms with18.9ms remaining, but reducer reports
lifecycle_pending/revoked. Increasing bridge timeouts is not supported.

The legacy owned-preedit FocusOut branch cancels its word by calling general
revoke_context_word -> revoke_current_owner -> stamps.revoke. Word revocation
is required; clearing authenticated later lifecycle metadata destroys the next
field's chance to run its existing actual context acquisition.

Causal RED on candidateA: raw actual FocusOut followed by FocusIn ingress, then
old FocusOut callback. Existing owned-preedit fixture loses exactly the passive
FocusIn record (ErrDenied),1selected1failed0.0326s. Raw receipt
CROOT/release100-owned-focus-red/result.json; testdraftCACHE100/owned-preedit-focus-regression.rs.

## Minimal repair and boundaries

Recommended9/10: add one narrow adapter entrypoint validated against the exact
current owner and accepted pending FocusOut ticket/position. Revoke the old word
through the existing reducer while preserving already received bounded callback
metadata. If target acquisition has already begun, reuse
discard_pending_target_word to retain its original Get/marker identities as a
source-free UnknownStart result. Otherwise existing revoked-factory recovery
starts a fresh actual context read. Clean the old engine locally with existing
strip/discard helpers, without a second broad revocation.

The passive focus stamp gives no token, word or edit authority. Old tokens and
mirrored text must be invalid; a fresh actual context/marker remains mandatory
before a new empty UnknownStart owner. Generic unobserved/malformed revocation
continues to clear stamps. Later superseding focus must still defeat an old
focus. No new owner, queue, cache, event ledger, bypass or client-specific branch.

Alternatives: globally stop clearing stamps2/10 (weakens unrelated revocation);
treat every Revoked ticket as settled1/10 (revives unavailable ownership);
raise timeout2/10 (observed barrier has ample time); reset IME per refusal1/10
(hides lost ordering and disturbs input).

## Verification status

RED proved; candidateB2 passes3519/3519 source/package and11/11performance
checks, with zero known semantic/infrastructure failures. Python419 tests
(10skipped), guards, lints and both MSRV routes also PASS. All10 optimized
binaries built in73.5365s; final proof is CROOT/release100b2-final-gates.
Source/mode closure1287files is
9e1486e38397f45e824ffc4471649ce8e90f1b8fbc09abc0bf45f2347f1a426e.
Neither native success nor release readiness is claimed. Keep rejected A and
B1 failures. Frozen Firefox/Chrome native checks remain before100publication.

B2 installed only in the authorized remote KDE session: IME2662296,
SHA443576475a38806da3c7e3e433f91869a57ffe3855374a20fc80f27b844c3811,
daemon2662418/tray2662424. All10 retained artifacts and loaded versions match;
rootIBus3168326 and protected processes kept. Receipt
CROOT/release100b2-final-deployment/result.json. No manual local GNOME install.
Frozen full r18 completed: Firefox128P53F39B andChrome174P4F42B/220each.
Normal rich roundtrip(active), refused in candidateA, is now PASS. Total302P57F81B;
12 prior98PASS-to-FAIL losses remain, so this is scoped lifecycle proof and
not regression-free/all-client acceptance. CI40370251745 andSecurity40370251729
SUCCESS on functional2725bcf449000fbbce71770d4bbe7293ce4ad3ed.
Exact native deltas, speed coverage, trace completeness, separate dedicated-
CPU short runs and terminal failures: tech_debt/evidence/release-1.0.100-native.json.

CandidateB1 full run: all required guards/lints/MSRV/PythonPASS, actual
Rust3518PASS/1FAIL of3519. The known-failure set stays0; gate therefore reports
BLOCKED_CONTRACT for the unexpected semantic failure, not an infrastructure
error. Existing td125_focus_out_refuses_transfer_of_cancelled_owned_preedit
correctly requires local+shared committed prefix after observeFocusOut, then
clears it only at finishFocusOut. Full context discard erased that data too
early. Do not weaken this old test. Correction preserves the existing prefix
publication, factors local word-witness retirement from broad adapter
revocation, retains old inert owner identity for existing late-Disable guards,
and keeps actual source-free context acquisition mandatory. CandidateB2
focused whole owned-preedit group pending; B1failure retained.

Prefix correction focused result:43/43owned-preedit testsPASS6.801s, including
newpositive/adverse schedules and the unchangedtd125 prefix contract. Local
word-witness cleanup is factored from broad adapter revocation; committed
prefix and its existing shared publication stay until finishFocusOut. Old
local owner identity remains only for existing stale-Disable guards; token and
word witnesses are retired. No oldword gets transfer authority. The isolated
probe restored B1source/mode closurebd018e0468d09a190fa07fa94cebc83e9796ed738b1e36e6cd99d4d65457e53f.
Final B2snapshot/full gates next; noB runtime installation or native claim.
