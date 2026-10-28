# 1.0.109 — isolate KDE terminal qualification and daemon focus policy

Preflight before code. Base058a247f0200ad6139fcb2a70abd186f05d1ebff (108 sourceb31a47181e33, remote108 installed).107 is stable;108CI is independent and pending. This task is authorized modularization, not a new terminal/Firefox functional repair claim.

Current KDE terminal recognizers live in kde/desktop.rs alongside generic desktop observation. Four KDE focus-policy functions and its unavailable flag remain inside common daemon focus_guard.rs. Move these existing definitions to kde/terminal.rs and kde/focus_policy.rs. The common dispatcher remains sole desktop selector; KDE errors never select GNOME. Preserve one state flag, original atomic operations, error strings, provider/read deadlines and old test identities through necessary re-exports.

Allowed runtime: src/bin/lay_ibus_engine/kde/desktop.rs;new kde/terminal.rs;src/bin/lay_daemon/focus_guard.rs;new kde/focus_policy.rs. This explicitly includes common-source integration seams; do not call the entire source diff KDE-folder-only. GNOME function bodies, correction/IME state/ownership/verifier/executor/native_input and canonical kde_focus provider are protected. New modules own desktop-specific policy, not another owner or transport.

No added/removed/renamed test identities or category changes are planned. Compare all moved bodies (only equivalent root-qualified imports and visibility may differ), then canonical source/release gates and all10binary builds. Use actual stock-launcher Kitty/Konsole and available GTK-VTE controls for Shift, hint/Tab, Double Shift before/afterSpace/focus/latency. Keep unsupported/unobservable cases explicit; do not change terminal allowlists or tests to obtain PASS. Browser regression controls remain separate. Local GNOME and physical HID are not accepted by remote tests.

Build-cache inventory precedes cleanup: old debug executables/own rlibs freed6,322,727,334bytes; ten retained108release artifacts verified unchanged, no models/source/config/runtime deletion. Receipt isolated-release109-cache-prune-v1/result.json.

Implementation and source/native proof: PENDING.

## Source proof

Frozen1294-file sourceSHA `3c5463769bafe6b0ce5adf040f07d5adeaede758787fec99fd37d0549d16de33`. All3551canonical correctness/package tests PASS (3515+36),1113IME, zero known failures. All old test names/classes retained, no new tests. Localrun-hn_ql_x8/remoterun-uywcKX. Six moved function bodies compare exact after equivalent root-qualified imports; one original atomic availability flag remains, with the same Relaxed load/store/swap. Canonical KWin transport unchanged.

The earlier terminal harness used Konsole --fallback-profile and cannot certify an ordinary profile. The new native acceptance uses the actual Desktop Exec/default profile, own title and raw child; existing inputs/assertions/cadence remain byte-equal. Three parent/new actor hashes and rest-of-file equality are recorded in CACHE/release109-stock-terminal-actor-provenance.json. GTK VTE GNOME Terminal is an additional actual-launcher case, not an allowed-process change in product code.

Release gates and native/install scope still pending. A screenshot/InlineHintSnapshot is inspected separately from PTY committed-text PASS. Source-module separation does not close the recorded Firefox latency/iframe or omnibox limitations.

## Exact artifacts and native acceptance

All18releasegates PASS,10binaries,0warnings,MSRV1.88/default+lexical and11performance PASS. Source3c5463769baf unchanged after frozen full3551; exact109installed in remoteKDE. No local manual apply/restart.

Native33 planned cases:Kitty11PASS0FAIL;Konsole11PASS0FAIL;GTK/VTE GNOME Terminal3PASS8FAIL. Chrome3controlsPASS;Firefox20=14PASS4FAIL2BLOCKED. Source/owner/frame checks are not a substitute for these failures. Rawaccepted evidence `isolated-release109-native-acceptance-v3/result.json`.

Test-tool refusals retained:initial terminalV1 failed before typing because systemctl used GUI D-Bus instead of manager bus, nineNO_RESULT incorrectly received PASS from my aggregator. That receipt is REJECTED and superseded, not terminal proof. V2 corrects only control-manager bus; actual GUI/IME bus remains unchanged. Acceptance now rejects absent/empty/unrecognized/inconsistent/incomplete native receipts; seven direct-predicate positive/negative checks ran before input.

Old RU punctuation fixture had the identical !,Ж()_ mismatch without IBus. Actual compiled XKB maps RU Shift+6→colon andShift+7→question. Corrected fixture sends proper unshifted physical keys under heldShift for RU/US, retaining exact !?:()_ expected output; only threeShift suites repeated. All sixShift cases pass forKitty/Konsole. Original input/control/failure receipts retained.

Captured before-Tab images show own white continuation ерка next to plain prefix for allthree apps. That does not close VTE functionality:its active letters stay in preedit, PTY receives them only on Tab/Space, next-letter/pair/active-five contracts fail. Passive EXISTING canonical KWin callbacks identify VTE app_id empty,wm_class gnome-terminal. No global activation environment change is justified:IME is connected and Tab commits the expected word.110 will map the isolated qualification extension before code.

Publication pending immutable commitCI and updater verification; current109 is a mechanical isolation release with explicit unsupported VTE/Firefox scope.

Stable v1.0.109 published at 2026-10-28T07:50:07Z from96a98674f488a40615e2d4a4d7d98c1b85e219a9. Four GitHub checks SUCCESS; local/remote real updater checks verified stable109. Local installed107 was not manually changed; remote installed109 exact artifact. Publication/updater receipts are copied into tech_debt/evidence/release-1.0.109-verification.json. No functional verification reopened for this metadata update.
