# Authenticated revocation retires the old word without erasing received focus

Owner: existing WindowInteraction / ContextAdmission reducer and bounded stamp
store. Invariants: C03, C06, C08, C10. Stage: original fifth release group,
candidate 1.0.102. No new owner, input route, model or verifier permission.

## First failed transition

Baseline 1.0.101 files/modes closure:
8c25e15e9c6f9c481a0b91e5da1b59227749f19cfd6c2db187c8bf7ee27f1a9f.
The actual receive observer rejects Disable without a preceding FocusOut, or a
second FocusOut, and revokes the old reducer word. It records an authenticated
Revocation for that old owner. A later real FocusIn is received before the old
callback runs. observation.rs::observe_context_focus_out/disable validates the
exact old revocation but calls revoke_context_word again. That generic call
clears the shared stamp store, erasing the successor's already received focus.

## Minimal alternatives

9/10: only the two validated revocation branches strip the old preedit mirror
and retire local word witnesses with existing helpers. The receive-side
revocation remains final. Normal lifecycle cleanup runs; a new word still
requires actual context Get and its native marker. No old word can transfer.
2/10: globally preserve stamps for every revocation; this would weaken unknown
or malformed input handling. 5/10: manufacture another focus request from the
callback; this changes acquisition flow instead of retaining received input.

## Measured controlled experiment

Two separate tests use actual P2P receive/callback entrypoints, a genuine owned
preedit, real Disable / duplicate FocusOut and a later received FocusIn. They
assert the old token is invalid, no text edit/learning is issued, and only a
fresh context Get plus marker establishes SourceFree/UnknownStart/zero suffix.
Baseline: 0 PASS / 2 FAIL on the exact lost-successor assertion. Minimal
candidate: 2 PASS / 0 FAIL. Both experiments restore the original source and
verify the full baseline source identity in finally. Installed 1.0.101 bytes
were not changed by these experiments.

Raw receipts:
CROOT/release102-authenticated-revocation-red2/{result.json,test.log,source-restored.json}
CROOT/release102-authenticated-revocation-green/{result.json,test.log,source-restored.json}
Local: CACHE102/red-green-experiments.json.
CROOT=/home/e/projects/lay-development-runner/kde-install-control-20261020;
CACHE102=/home/ubu/.cache/lay/development/release-1.0.102-20261026.
Baseline observation SHA:2db5e35cf272f771ec9aee6b3eaba3ee44ff1419cc784287cd619a6203f10c36.
Candidate experiment SHA:79b35a8c416442b32524852307f3ec81879d9d9f16c5487f79fc4401da9c0696.

## Pending promotion proof

Add negative controls for missing/stale provenance and a superseded owner;
retain all prior tests. Full source/performance/lints/MSRV/optimized binaries,
exact loaded bytes, native controls, CI and updater acceptance remain pending.
Two source tests do not establish browser or terminal acceptance. The separate
Chrome Space stale_lease and WezTerm presentation failures remain OPEN.
Do not force an unrelated file split for TD-132/134 or claim TD-123 quality
promotion; reconcile their current owners explicitly without inventing PASS.


Implementation prepared in the two exact admitted branches, using the existing
strip/retire helpers; generic refusal branches unchanged. Added a negative
control for an unobserved header. Its first draft incorrectly required no live
token after generic revocation; existing reducer semantics deliberately retain
a new-generation UnknownStart/zero-suffix token. Corrected test asserts the
actual no-old-word permission contract, not a changed runtime rule. That revised
negative passes on unchanged101 (CROOT/release102-unobserved-revocation-baseline).
The rejected draft run has11PASS/1FAIL of12 and remains retained under
release102-authenticated-revocation-controls. Full102 verification is next.
