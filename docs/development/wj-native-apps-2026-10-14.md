# WJ native application smoke — 2026-10-14

## Frozen candidate

Source checkout `/home/ubu/projects/lay-space-boundary-shift-20261007`, branch `codex/space-boundary-shift-20261007`, base HEAD `d362e7c82cb14f3d2aab123b9130707f1843ba02`, dirty source archive `b46f7c455b4aa50f6dae7cd72df68e8f0b91feccb3b43c5d4a3c22355f089d45`, version 1.0.82. This is a paired core candidate, not the full ten-binary installer or an accepted public release.

- IME SHA-256 `84f8510cc8e99ba552fe92837e71062f5121775ece1efe3588a462a668e1cccd` (8,292,112 bytes).
- Daemon SHA-256 `08416956a6a94acb2446b1124311c7975bd4c05dbcdd99fcca3f26452d4fa068` (9,264,592 bytes).
- Source gate: 848/848 PASS (797 IME + 51 contracts), immutable `run-w03rUs`, worker 116.147 s; final focused66 PASS; AST artifacts fetched and hash verified.
- Build: 183.683 s, explicit release IME/daemon targets, default features. No further production source edits, model fitting, laptop installation or production route changes during native app verification.
- Independent context-free WJ code review: NOT_RUN (agent thread limit), no fabricated score. TD-121 remains OPEN.

## Physical scope and results

Real owned evdev input on remote `192.168.3.94`; stock GTK3 3.24.33, Qt5 5.15.3/PyQt5 5.15.6, official WezTerm 20240203-110809-5046fc22. Five native fields get16 plans each; WezTerm gets7 functional plans and one basic active-preedit/closing-Space visual pair. Actual widget/accessible text/caret and owned benign PTY observations are retained. The terminal fixture never executes input. Screenshots come from the real private GNOME50 compositor.

| Actual client/field | PASS | FAIL | BLOCKED | NOT_TESTED | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| GTK3 TextView | 12 | 2 | 2 | 0 | 16 |
| GTK3 Entry | 3 | 11 | 2 | 0 | 16 |
| Qt5 RichTextEdit | 4 | 11 | 1 | 0 | 16 |
| Qt5 PlainTextEdit | 4 | 11 | 1 | 0 | 16 |
| Qt5 LineEdit | 4 | 11 | 1 | 0 | 16 |
| WezTerm | 5 | 3 | 0 | 0 | 8 |
| Total | 32 | 49 | 7 | 0 | 88 |

The 49 FAIL are scenario failures, not49 independently diagnosed bugs. This is the first matrix for these native clients; the same native set on earlier WG bytes is NOT_TESTED, so new-regression attribution is UNKNOWN.

GTK TextView passes all four8-gesture cycles (short/long/mixed/English with next-letter/Backspace probes), active/post-Space5-gesture bursts, moved boundary, clean text, Backspace and focus return. RU-to-EN closing Space and selected-text Double Shift fail. Its prefix cycle and genuine completion are BLOCKED by missing trustworthy current preedit-caret/candidate observation.

GTK Entry and each of the three Qt editors fail11 conversion/correction plans: the native target does not appear; snapshots retain original committed text. Ordinary clean typing/Backspace remain working. Qt focus return passes; GTK Entry returns a selection and is BLOCKED. Completion is BLOCKED without a genuine bounded observed candidate. First runtime predicate causing these conversion failures is UNKNOWN; no ranker or delivery change is inferred from widget surfaces.

WezTerm passes EN-to-RU closing Space, clean RU/EN, Backspace and the visual pair. Post-Space Double Shift, RU-to-EN closing Space and moved-word boundary fail. The active image shows typed prefix `привет` plus completion tail `ное` at the insertion position while PTY is empty. After Space the highlighted composition disappears and PTY/visible output is precisely `привет ` with the caret after the space. This proves one basic inline-preedit/Space case. Focus-change rendering, other positions, terminal selection and terminal completion acceptance remain NOT_TESTED outside the declared88-case smoke scope.

## Speed

Required290 transformation targets;44 measured nonnegative observations,246 missing,13 observed before the chosen closing-release marker and retained separately. Median75.906 ms, p9586.828 ms, max91.946 ms. Clock: evdev write-return to actual native widget/PTY observation, not kernel release and not per-action compositor paint timing. Incomplete coverage prevents claiming matrix-wide latency. Failed/missing actions are not discarded from the denominator.

## Telegram

Official Telegram Desktop7.3.1 is installed remotely in a private test profile. No authenticated composer exists. User authorizes draft input and forbids message sending; injected Telegram input/send keys0. First concurrent launch logs `QThread::start: Thread creation error (Resource temporarily unavailable)` under the512-task resource ceiling. No ceiling was raised. The file named `telegram-login.png` actually shows WezTerm foreground, so it is explicitly NOT a Telegram visual receipt. Qt fixture results do not certify Telegram. A separate low-concurrency launch/login is still required.

## Evidence and continuation

Immutable complete archive: `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c/WJ-native-app-acceptance-1791983329525303054/ARCHIVE.json`, 35 payload files with SHA-256. Raw actor result SHA `ec1393834c2fc7ad4f936d6aea4fe8537a3ad93f2196ca474977f6441789fc24`; `RESULT_ASSESSED.json` adds the explicitly scoped root visual verdict without overwriting raw results.

GTK cases were finalized in earlier partial scopes and carried by immutable receipt hashes, including every FAIL and BLOCKED; none retried unchanged. Later Qt/WezTerm cases use the same exact frozen core bytes. Source, partial/native and browser receipts remain distinct.

Final scope actual5286/driver33110 both exit0, all three setup parents exit0/no held keys; exact five original PID/startticks gone, cgroup empty, VT8 restored toVT1. Setup failures and their repairs are recorded in `2026-10-14-native-app-stock-ibus-shared-machine-identity.json`. Native client startup precedes final ordinary GNOME-owned IBus binding; no bus rebinding after keys. Normal Qt/WezTerm transport choices retained. Private collector adds native screenshots and explicit activation of a pinned existing owned window; original production input methods are unchanged. Restore the private helper after remaining diagnostics finish.

Latest complete browser result remains WG on earlier bytes: Firefox164P33F19B/216, Chrome179P23F14B/216, total343P56F33B/432. Full browser acceptance for changed WJ bytes is NOT_TESTED. The native88 matrix never substitutes for that browser protocol.

Next analysis should group the native failures by the first shared callback/authority loss, add a causal source regression and evaluate one general mechanism with all native88 and the original browser432 gates. Do not add literal phrase/client-specific runtime bypasses, loosen SafetyGate/verifier, train around an admission defect, retry known failures or claim universal support. Build/publication and further production fixes remain separate decisions.

## Later Telegram UI-only preparation

The standalone low-concurrency launch opens actual officialTelegram7.3.1 (binaryc8a9b525…), with4-CPU affinity under the unchanged512-task ceiling. A short alias to the same owned runtime directory fixes the measured108-byte Unix socket pathname problem; production transport choices unchanged. Real compositor images bind actual Telegram PID1584358. The screen is unauthenticated welcome/login UI. Composer input remains NOT_TESTED, messages/sends0. The UI-only phase does not run/claim the Firefox attachment acceptance guard; its unchanged original005 script is archived and must be restored before functional input. User authentication question is pending. The packaged report is an immutable earlier snapshot; this later addendum does not change the native88 result.

## User-directed local version1.0.83 installation — 2026-10-14

User ended automated client verification and explicitly requested installation with a higher version, saying they will test it. The frozen WJ source from run-w03rUs was copied immutably; only Cargo package/lock version, extension version/date and VERSIONING.md changed before the build. Remote guarded default-feature release build of existing CLI/daemon/IME took184.545s; no source test, browser/native input or model fit was repeated. Shared Cargo target10,117,763,072B remains under12GiB.

Installed CLI/daemon/IME and extension display report1.0.83. Loaded IME SHA256 `b742b64dfd8d1d570e3eb8e354b18ba7040a8941ddc1e9982ff7513bdca6767e`, daemon `bc89b4d0a0ffe253898b1b2db0ba9825a7e92d9829c53bf5537af6927846e034`; CLI `a8b160cb2b5f2e1d7b0ae6e6961bc3ae33bd83e7311eaed95c688dbc2b5b8451`. Three binaries and two extension version files were backed up and hash verified before replacement. Existing atomic-copy, extension reload and Lay channel control were reused, with a focused empty installation witness and zero input keys. Global IBus/L11/L3 PID/starttick/loaded bytes, layout, configured input sources and config remained exact.24 other installed executable/extension files retained their hashes. Delivery routes, static models and input authority were not edited.

Install receipt `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/install83-20261014T141659Z/INSTALL.json`; decision `docs/architecture/decisions/2026-10-14-user-directed-version83-install.json`. Core archive `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/lay-1.0.83-core.tar.gz`, SHA256 `95dba65eb694f24edc245461d5515dda5c837bbb84b89a2d063111017da29149`. Existing ancillary tools/models are preserved. No GitHub release/tag/push performed.

Source848PASS and native88 results remain historical WJ1.0.82 logic evidence, including49FAIL/7BLOCKED; physical acceptance of1.0.83 is USER_WILL_TEST. Latest complete browser matrix remains WG343P56F33B/432 on old bytes. Full WJ/1.0.83 browser matrix and authenticated Telegram composer are NOT_TESTED. TD121 OPEN and independent final WJ review NOT_RUN remain explicit. No new acceptance, speed sample or rollback execution is claimed.

Remote Telegram diagnostic stopped: all pinned processes gone, cgroup empty, VT1 restored, finite seat stopped. Both private screenshot/focus helper copies and original005 browser attachment guard were restored; owned short runtime alias removed. Telegram profiles/archives retained, messages/sends0. Exact remote closure receipt: `firefox-legacy-native-20261012.6onP5c/WJ-telegram-close-install83.json`.
