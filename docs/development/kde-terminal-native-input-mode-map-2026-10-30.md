Final recheck after installed Firefox style: CROOT/kde-terminal-after-firefox-style-native-v1/result.json4/4cases20/20flips PASS on IME2201796/05d62373 +tray2201829. Deliberately mismatched source before each; complete per-case traces, next letter, physical group and state restoration verified. Accepted terminal appearance preserved. This is not a browser full matrix.

Installed/native milestone — 2026-10-29T21:33:59Z:
IME2098587 SHA447d62583383da8be7d127ad9e67794239cf4c7e58f1ff6be372104e4e73979c; source2e47eb13916bde36aa92df9681f40a18e618ae63a0981c99fe837ef521197261, zero-warning release74.683s. Only idle IME factory replaced; daemon1722873/35527e63, tray795755/c304, IBus3168326 and original Firefox988548 unchanged. First intentionally-mismatched Konsole Space case5/5 flips PASS with complete trace. Four-case Konsole/Kitty active+Space:4/4PASS,20/20 actual raw-PTY flips, next-letter f, KDE physical group0 and IBusUS all verified; original mode/layout/focus/config/runtime restored. Receipts CROOT/kde-terminal-native-mode-first-native-v1/result.json and kde-terminal-native-mode-four-native-v1/result.json. Four-case internal trace anchor lost to bounded rotation (trace_complete=false), so complete internal callback lineage is not claimed; per-case PTY and keyboard receipts retained. Median19.628ms, p95 nearest-rank25.441ms, max26.445ms from second Shift press to PTY receipt, includes12ms hold, with189-202ms inter-gesture cadence; not release-to-render or unrestricted rapid performance.
Current accepted terminal rendering unchanged. Firefox typography remains independent. Browser full historical counts unchanged159/24/37 and179/0/41.

Full source GREEN:1145/1145 IME +22/22 focused PASS, exact2 added tests and no removed/changed registry entries, architecture graph refreshed. Source2e47eb13916bde36aa92df9681f40a18e618ae63a0981c99fe837ef521197261; receipt CROOT/kde-terminal-native-mode-green-v1/result.json. All43 frozen integration files unchanged. Build/deployment/native proof pending.

Short GREEN:22/22 PASS; receipt CROOT/kde-terminal-native-mode-short-v1/result.json, source68f544cec7fe71f9f8eebc11181a33d13293da4befc9275988c91d220ea30901, all43 frozen integration files unchanged. Not installed; native reproduction after deployment pending.

REDv2:22selected/21PASS/1expectedFAIL, actual native callback correctly records привет but decoderfalse persists and cannot select existing opposite-layout plan. All43frozen files unchanged. Receipt CROOT/kde-terminal-native-mode-red-v2/result.json/sourcec49bdfbc62a06a8b90209e6f4d730f2303324cbeb944eca304f7a56d7762ddc2. Activated only KDE native-literal observation via existing preferred_layout_for_text/set_layout_is_ru, with unproved/owned-composition guards. No output call or factory switch. Quick GREEN pending.

Source REDv1 is fixture setup failure, not causal RED: first Ready emits its one InputMode UpdateProperty before native text; the new raw drain helper rejected that metadata. Reuse the neighboring type_observed_native_text helper, which explicitly validates and consumes that declared activation property sequence and rejects unexpected text output. Runtime stub remains inactive; REDv2 pending. Original failed receipt preserved.

# KDE terminal native literal mode — 2026-10-30

Canonical b835ca47 checkout lay-dependency-updates-20261027. SOLO; no Harness/agents. Actual running daemon1722873/35527e63, IME1852753/6688be16, tray795755/c304. No style or fast-daemon activation. GNOME untouched.

Measured failure: CROOT/kde-terminal-source-mode-diagnostic-20261029-v1/result.json +complete native-trace.jsonl. Entry selected lay-ime-us with KDE RU(index1). Both first Double Shift attempts leave привет unchanged, RPCstatus0; callback trace decodes and records all Cyrillic letters correctly. Original stock test forced RU and hid this entry state. Existing idle source selection alignment and mode-preserving probes restore4/4cases20/20flips +next letters: CROOT/kde-terminal-source-mode-aligned-proof-20261029-v1/result.json. KDE SwitchMode=Global; no per-window cause claim. Origin of external source desynchronization is not yet attributed.

First source loss is mode passed into existing planner: committed_tail.rs:1329 -> manual_toggle.rs:116-129 uses false and maps Cyrillic toward Russian, obtains unchanged token, returns None. No output was attempted; authority/executor are not the cause.

Before-edit map:
```text
managed.rs::TerminalPassthrough literal key observation
 -> composition_commit.rs::observe_terminal_passthrough_char
     -> kde/terminal.rs::observe_kde_native_terminal_input_mode (NEW narrow observation)
         existing preferred_layout_for_text +set_layout_is_ru
     -> unchanged tail observation/input-frame capture
 -> unchanged committed_tail.rs::committed_tail_toggle_plan
 -> unchanged manual_toggle.rs::plan_manual_toggle
 -> unchanged verifier / TerminalErase executor / KDE layout intent completion
```

Only2runtime files allowed; other integration/planner/output/GNOME/Firefox files pinned by hashes in ADR. A literal observation changes no IBus factory and grants no word or edit permission. Neutral symbols preserve existing mode; GNOME/unproved/non-native clients remain unchanged. Existing set_layout_is_ru owns decoder generation/cancellation. No guessed service-global synchronization or new output route.

RED/GREEN through real callback state, then short/full IME proof and existing native mismatched-mode reproduction. Preserve current accepted terminal presentation and working Firefox.
