# Optional fragment layout contour — 2026-10-30

Status: source and scoped native verification complete; stable publication requires green public CI/Security.
Source baseline: d4dae7a027a7a9a67d6a1238f3881288dddb75a3.
Installed baseline: accepted 1.0.111, stable source 1.0.112.
Scope: correction-only plus requested GNOME/KDE configuration/menu controls.

## Verified evidence and limits

The compact model is687981bytes/40000fragments. Raw fixed heldout: RU9968/10000
correct,26wrong,6abstain; EN9800/10000correct,160wrong,40abstain. Russian data
are noun forms; English includes rare words/abbreviations. This is surface
classification, not unseen lemmas or permission to mutate text.

Rust agrees with the trainer on40000 two-interpretation comparisons. Full
common-core comparison uses the same20000 heldout surfaces in both modes:

| Direction | Legacy exact/wrong/abstain | Fragments exact/wrong/abstain |
|---|---:|---:|
| To Russian |390/97/9513|693/0/9307|
| To English |2964/105/6931|2961/0/7039|

Clean preservation: RU10000/10000 both; EN9979/10000 legacy vs9987/10000 new.
The opt-in alternative trades three English restorations for fewer wrong changes;
13 clean English surfaces still change. Do not claim universal improvement,
perfect preservation, a replacement for L1–L4 or a Wave-package promotion.
Existing common authority intentionally abstains on most unsupported rare forms.

Warm scoring median~3.5us,p95~8.4us; cold validation~0.4ms. The~140MiB process
RSS includes existing correction machinery and is not incremental model RSS.
Full details and exact measured values are in
[evidence/fragment-layout-1.0.113-proof.json](evidence/fragment-layout-1.0.113-proof.json).

The first common-core run exposed punctuation stripping before physical edge
projection. The adapter now tries unchanged whole physical-token helpers first.
The second exposed an omitted existing short-English-target context check;
english_layout_target_requires_context is now reused without modification.
Both defects were corrected inside the new adapter; no shared helper body changed.

Source:3611 selected correctness/package tests across34 targets,0 failures;
487 process-isolated plus3124 bulk test identities verified. The canonical
execution completed every target before stopping on the stale known-failure
manifest binding. Its real complete observation now binds the new registry.
The complete wrapper must also pass independently in public CI before release.
Strict Clippy default/research PASS,117/129 exact existing dead-code inventories,
0 new diagnostics. Python435 tests with10 skips; Node/syntax/installer/public
regressions/audit50 PASS. Rust1.88 default and lexical-compiler checks PASS.
All10 installed release binaries built at opt-level3,0 release warnings;
versions and SHA-256 come from compiler-artifact records and retained binaries.

Dedicated KDE:1.0.113 loaded and exact daemon/IME bytes verified. Native RFB
Chrome/Firefox checks:8/8PASS, one ordinary text field, normal windows; first
and second word, physical punctuation-key letters, Double Shift roundtrip
following Space. Space observation17–149ms,Double Shift58–198ms in this small
sample. Original220-case-per-browser protocol is unchanged; only two additional
explicitly selected diagnostic fixtures were added to a private actor copy.
Config restored byte-for-byte; global IBus and existing Firefox session preserved.
Local GNOME was not installed or restarted.

Private actual-client fixture reached no scenario: context setup failed equally
with the new flag ON and OFF. Its raw FAILED receipts are retained and are not
counted as functional PASS. No input-path fix was made to rescue that fixture.
The successful native desktop cases provide the scoped actual-client evidence.

Receipts: CACHE/fragment-layout-focused-v1..v6-{pinned,result}.json;
CROOT/fragment-layout-release-v2/{proof.log,bootstrap/logs};
CROOT/fragment-layout-release-v4/{result.json,source-return.tar.gz,binaries};
CROOT/fragment-kde-deployment-v1/result.json;CROOT/fragment-kde-native-v1/result.json.
CACHE and CROOT are the established local development cache and dedicated
remote kde-install-control-20261020 directory. Source closure:
01509c18c977a0b7890eae437514ea8a856e5648635d099a2605266af731ee26.
The historical full matrix remains Firefox159/24/37,Chrome179/0/41,total338/24/78
out of440. No new full browser matrix and no whole-Lay zero-FAIL claim.
Runtime authority and delivery routes are unchanged.
