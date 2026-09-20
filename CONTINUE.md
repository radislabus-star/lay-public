# Current: TD-121 acceptance, 2026-09-13

## Current TD-121 acceptance — 2026-09-13

TD-121 is **IN PROGRESS / INSTALLED_VERIFIED_PHYSICAL_PENDING**. C20 passed the
complete 1.0.72 release gate, including all 13 commands, with changed and full gates each passing
2,807/2,807 and 11 intentional performance skips, plus compiled-receipt verification and four isolated final-byte client cells. Its
immutable release `RESULT.json` SHA-256 is
`02458047a539fb85be82b301fd1cf38b19af1f71241f34260713cf5c5dd90534`.
The preceding focused 519/519 proof on the same runtime source passed separately.
The owned GTK entry smoke then passed all three exact surfaces: `привет` after
one manual toggle, `ghbdtn` after two toggles with zero queued key/space/boundary
passthrough, and layout projection `ghjdthrf` after autocomplete with one toggle. Receipt
SHA-256: `27bf83fccd5a15e552dabd9afeada8f1bacfd9038b8f9a0046f2cb3d0fcd1eaf`.

Release 1.0.72 is installed. All ten installed artifacts and all four loaded
owners match C20; the loaded extension reports 1.0.72. Global IBus identity,
configuration, input sources, immutable model dependencies, journals and learner
state were preserved. Installation receipt SHA-256:
`ca7b0cb622f862cdb9a51678e640d27953fe798f3b37f291bdebce4f5e4735a4`.
C12 remains the historical 502/502 source-review checkpoint accepted at 9/10;
it is not the release identity. The remaining TD-121 acceptance gate is a human
physical-keyboard confirmation. General TD-123 answer quality remains `UNKNOWN`.

Historical attempts and their exact receipts are retained only in
[evidence/td121-private-actual-baseline-2026-09-13.md](tech_debt/evidence/td121-private-actual-baseline-2026-09-13.md).

# Previous checkpoint: TD-126 source-only DONE

TD-126 now consolidates the managed IBus client boundary behind
`window_interaction/{observation,authority,execution}.rs`: lifecycle and fact
observation, existing authority admission, local execution or typed delegation,
later postcondition projection, cancellation and RAII settlement/revocation.
The old `bridge_actions.rs`, `context_runtime.rs` and `text_target.rs` owners are
deleted. Plan review closed 6/10 → 8/10 and implementation review closed
7/10 → 8/10, each with one grouped repair and exactly two passes; no third
review is claimed. Focused repair proof is 506/506 PASS. The final one-lease
affected denominator is 2,781/2,781 PASS; lint, all 11 architecture checks and
the compiled-receipt projection pass. The authoritative source result is
`docs/structural_gates/receipts/LAY_TD126_COMMON_WINDOW_2026-09-12/final.json`;
the compact durable account is
`tech_debt/evidence/td126-final-acceptance.md`.

TD-126 is `DONE` only for source scope. The installed runtime and runtime
authority are unchanged; installation, activation, real-client and physical
keyboard acceptance remain outside this verdict. Commit/push were outside the
authorization active during implementation, but the user has now authorized
the complete task loop. Git checkpoint status, the exact commit, verified remote
ref and clean-worktree result belong to
`/home/ubu/.cache/lay/development/td126-publication-20260913/publication.json`.
On continuation, inspect that receipt first. If the checkpoint is not verified,
run the one bounded graph/compiled-receipt/diff identity check, commit the
accepted final4 composition, push only to `origin/codex/cleanup-20260908`, verify
the remote ref, and update the receipt.
The checkpoint includes the previously accepted uncommitted
IME/context/native prerequisite base and the TD-126 extraction; it does not
claim a new quality result or runtime installation.

After a verified clean checkpoint, continue in dependency order: **TD-121 → TD-125 →
TD-122 → TD-123**. Re-evaluate live dependencies before each task. For each:
freeze the baseline, implement the bounded mechanism, use a fresh-context code
review scored 1–10 with at most two total passes, pass the objective gates, mark
the scoped task `DONE`, commit/push/verify its checkpoint, then move on.
Temporary candidate activation may begin only after its owned test field/capture
and activation preflight are ready. Permanent installation and physical
promotion require the corresponding client and real-behavior checks to pass.

# Historical predecessor: exact-replay native callback candidate accepted

The reviewed ExactReplay contour is implemented in source with status
`CANDIDATE_FOCUSED_CALLBACK_PASS_REVIEW_ACCEPTED_RUNTIME_UNCHANGED`. It routes
leased replay Backspaces and printable key presses through the legacy callback
as native-unhandled events while updating the bounded optimistic tail mirror
once per exact epoch. It emits no replay `CommitText`, does not run ordinary
`push_tail_char` bookkeeping, and quarantines replay candidate, precognition and
completion-learning effects. Mismatch revokes the contour and consumes the
offending press instead of falling through to a second text mutation path.

The common TextTarget map composes the existing owners. Public bridge
`manual_toggle_outcome_inner()` yields typed `Handled`,
`DelegateExactImeTail`, `DelegateDaemon`, or `NotHandled`; the internal
`text_target_decision()` yields typed commit-only, exact SurroundingText,
terminal erase, or unsupported route and reason. `ContextAdmissionReducer`,
`context_runtime`, bridge tokens and the route executors retain their existing
focus, Reset, owner, path, epoch, snapshot, selection, sensitive-content and
atomic guards. `replace_committed_tail()` executes an admitted edit;
`process_exact_replay_press()` owns only the active native replay contour; and a
later `set_surrounding_text()` observation must satisfy
`current_external_snapshot_agrees_with_owned_tail()` before another destructive
lease. The three capability facts in `TextTargetDecision` are not complete
TextTarget authority and no new authority owner was added.

The dedicated-20cpu focused proof passed formatting and `476/476` selected
`lay-ibus-engine` tests. It covers six complete transactions, including the real
Reset -> exact SurroundingText -> live-token bridge re-receipt path and uppercase
`Ghbdtn -> Привет` with Shift press/release; nine concrete refusal subcases,
including separate owner, active-path and active-expiry failures; exact
interleaved SurroundingText; and ordinary-input lifecycle controls. Successful
precognition schedule/apply counters stayed `(0, 0)` on replay with enabled
production flags while the ordinary control reached `(1, 0)`. Canonical receipt:
`docs/structural_gates/receipts/LAY_EXACT_REPLAY_NATIVE_DELIVERY_2026-09-12/candidate-focused.json`.

Source and focused-test status are PASS only. The installed IME remains the
earlier Firefox soft-Reset V3 baseline below. No candidate binary was built or
installed, no process or setting changed, transport remains `UNKNOWN`, and
physical acceptance remains `PENDING`. The canonical remote graph refresh now
passes with 22,327 nodes, 59,224 edges, 814 communities, 700 bound Rust sources,
and all eleven architecture checks at zero violations. Receipt:
`docs/structural_gates/receipts/LAY_EXACT_REPLAY_NATIVE_DELIVERY_2026-09-12/graph-final.json`.
The remaining source gates are the final changed suite, the dedicated physical
Double Shift owner test, and remote release-candidate build.

Candidate activation remains a separate reviewed step. Immediately before it,
recapture the selected `lay-ime-us`/`lay-ime-ru` engine, GNOME sources/settings,
installed IME PID/start/executable hash, daemon identity and global IBus
identity, then back up the installed IME bytes. Stop only that captured old IME
and wait for its IBus/session names to disappear. Start the receipt-bound
candidate in a transient user systemd unit whose persistent parent satisfies
the `--ibus --managed` parent-death contract. Verify the Lay IBus factory,
session bridge, DBus owners/PIDs/executable hash and absence of a third IME,
then reactivate the same previously selected engine exactly once through IBus.
Any component-demand race or identity mismatch triggers candidate teardown and
restoration through installed component demand; do not start the normally
inactive `lay-ibus-engine.service`.

The client proof must use an isolated text field under the current default GTK
transport and capture the actual focused client/context. Do not use the common
runtime smoke harness because it stops the daemon and forces
`GTK_IM_MODULE=ibus` plus synchronous IBus mode. The running daemon does not
hotplug a newly created `lay-test-input` virtual keyboard, so that helper cannot
prove physical ownership without a daemon restart. Use one user-observed manual
many-cycle keyboard sequence and record client-visible text, trace and matching
SurroundingText as separate facts. Verify daemon, global IBus, engine settings
and sources are unchanged before promotion or rollback.

# Installed baseline: Firefox soft Reset exact-ST re-receipt V3, physical pending

V3 follow-up for the post-V2 Firefox/GTK failure is installed and loaded with
status `INSTALLED_LOADED_HASH_VERIFIED_PHYSICAL_PENDING`. Runtime authority
changed: true, IME binary only. Physical acceptance remains `PENDING`: the
post-install ping returned `lay-ibus-engine-rs no-focus`, so there is no
Firefox/Tor focus proof, no browser text mutation proof, and no quality,
heldout, RSS, or latency claim. Tests/CI denominator is 0.

Build receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/receipt-fix-build-v3-compile2`.
Remote run:
`/home/e/projects/lay-development-runner/browser-receipt-XQR6ov`. Source archive
SHA `bf8665188a62e077b7095843480b67b1d67aefc835ad4760fa024dee04d407a6`,
1382 files. Build result SHA
`5cd763bc62c31af68b8c53351a89d7b86837f842d46a4d1143b8159da910163b`; status
`PASS_RUNTIME_BUILD_ONLY_GRAPH_UPDATED`. The graph AST was updated in the remote
build path and the fetched artifact/hash were verified. The built IME binary is
7,775,072 bytes with SHA
`67827521149fe73434f8025a6daa404f26d9ac2072d7de7cbe6f3aec0ad57969`. The daemon
binary SHA stayed
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e`; it was not
installed.

The failed predecessor build receipt `receipt-fix-build-v3` stopped before
installation on E0063 because `state.rs` was missing the new
`context_reset_rereceipt` initializer; that run changed no runtime authority. The
initializer was then fixed before the `receipt-fix-build-v3-compile2` build.

Installation receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/installation-v3.json`.
Old IME PID `4051893` SHA
`994485bf9d7379c8d820171960c61e5980e59f88341ab51d6a8b7741c08eec80`; new IME
PID `911924`, PPID `4715`, start `80439768`. `ibus-daemon` stayed PID `4715`,
start `2261`; `lay-daemon` stayed PID `3880511`, start `79007142`, SHA
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e`. Selected
engine `lay-ime-ru` and settings were preserved. Rollback backup root:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/receipt-fix-build-v3-compile2/install-backup`.

Installed V3 invariant: an authenticated soft Reset may keep one manual-only
reset re-receipt witness without republishing the tail epoch. The already-handled
printable key settlement owns the tail text/epoch in the reducer; redundant
soft-reset `publish_tail_handoff()` is skipped only while that witness is armed.
Generic Reset without the witness keeps the existing republish/revocation
behavior.

Installed source scope: the IME captures live pre-Reset eligibility from the old
token/scope only as evidence, stores the post-Reset admission token after reducer
revocation, confirms the witness only on the next surrounding-text revision when
the unselected snapshot exactly bounds the current committed-tail token,
preserves it across Shift press/release, advances it across exact handled
printable appends, and consumes it only during explicit manual Double Shift to
bind the current UnknownStart lineage in the reducer. This does not promote
`KnownStart`, does not authorize automatic hints, and does not change ranking,
verifier, SafetyGate, model paths, terminal fallback, or generic replacement
routes. First-word automatic hints/autocorrect remain outside this bounded manual
route.

# Current: exact manual handoff V2 installed, physical pending

V2 connected repair is installed with status
`INSTALLED_LOADED_HASH_VERIFIED_PHYSICAL_PENDING` for the browser exact manual
Double Shift route and bounded first-word GUI manual handoff. Runtime authority
changed: true, IME binary only. Physical acceptance remains `PENDING`: the
post-restart ping returned `('lay-ibus-engine-rs no-focus',)`, so there is no
browser-focus proof, no text mutation proof, and no quality/heldout/RSS/latency
claim. Tests/CI denominator is 0.

Build receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/receipt-fix-build-v2/build-result.json`
with status `PASS_RUNTIME_BUILD_ONLY_GRAPH_UPDATED`, elapsed 78.86 s, transport
79.29 s / worker 78.86 s as reported by root, source snapshot 1382 files,
archive SHA
`ce1e223ae79cff15064c349f6a07257bfba1fd36e5bb71d9f022aa2838b8a7c0`. Fetch
verification:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/receipt-fix-build-v2/fetch-verification.json`
with status `FETCH_HASH_VERIFIED`, build receipt SHA
`cf52594b41cdc2e182fc29e2bddc01d699ac4bc35e4947f087157a78000d514f`.

Installation receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/installation.json`
with SHA
`abc0b753a6f676fc7b6ce91c0f09405f8749244f9c871b5beae867f9a5943a22`. Runtime identity receipt:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/runtime-after.json`
with status `INSTALLED_PROCESS_IDENTITY_VERIFIED`; all 11 recorded identity and
configuration checks are true, including loaded process hashes/PIDs, DBus owner,
config/sources/bindings/XKB/selected engine, and temporary debug removal. Only
`lay-ibus-engine` changed. Loaded IME PID `4051893` SHA
`994485bf9d7379c8d820171960c61e5980e59f88341ab51d6a8b7741c08eec80`. Daemon
PID `3880511` SHA
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e` and global
`ibus-daemon` PID `4715` stayed unchanged during install. Selected
`lay-ime-us`, input sources, config SHA, direct Alt+Shift binding state, backward
bindings, and XKB options were preserved. Rollback backup root:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/backup`.
The temporary diagnostic override had already been removed before this final
install.

Installed source scope: live exact markers are preserved through valid same-field
context-admission `Transfer` while keeping the original source path for daemon
cleanup; `VisibleTailV3` and `SuppressNextAutocorrectV2` consume ready target
activation under the fenced-token target engine before live-token/shared-active
checks; first-word GUI manual Double Shift can use the existing exact handoff
only with current unselected surrounding-text suffix evidence and live bounded
lease. Generic mutation routes, automatic routes, terminal manual route,
verifier, SafetyGate, `SourceFree`/`ResetUnknown`/revocation clears, daemon
WordBuffer fallback, and stored legacy focus receipts are unchanged. Root static
source review accepted the source shape; that review is not correctness, quality,
or physical proof. First-word automatic hints/autocorrect remain OPEN pending
user clarification.


# Historical: browser legacy FocusIn exact-tail receipt projection source-only V1

Fresh physical report on 2026-09-11: `djn` was typed, the IME later delegated
Double Shift through exact committed-tail, and daemon replay was rejected before
mutation because `VisibleTailV3` had no field focus receipt. Frozen evidence:
`/home/ubu/.cache/lay/development/double-shift-window-20260912-49o8j3j9/user-failed-djn-20260911T231756Z/`.

Bounded source fix prepared for review only: context-admission `VisibleTailV3`
projects an opaque receipt from the current live `AdmissionToken` / admitted
`ContextKey` instead of storing a fallback receipt in engine state. Daemon V3
guards, tail epochs, InputState, detectors, `KnownStart`, automatic gates, ranker,
verifier and SafetyGate are unchanged. Runtime cost is one bounded `String`
allocation on an existing `VisibleTailV3` read; no extra RPC, deadline, timer,
state cache, worker, learning, package, reload, install, or binary authority
change. The scalar receipt is stable only for the same admitted `ContextKey` on
the same IBus connection. The first-word assistance concern remains OPEN and
separate. This step ran no tests/CI/build/local graph/install/smoke, per the
current instruction. Runtime authority changed: false.

# Historical: Alt+Shift/source-frame follow-up installation before V2

The first 2026-09-11 GNOME-owner Alt+Shift hotfix failed physical recheck:
after each manual Alt+Shift the first word still entered source-free
`UnknownStart`, while the next word applied normally. Fresh evidence:
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/ibus_engine_debug.jsonl`.

The follow-up source/runtime fix is installed with status
`INSTALLED_LOADED_HASH_VERIFIED_PHYSICAL_PENDING`. GNOME now keeps only
`['<Shift><Alt>space']` for `switch-input-source`; the four direct Alt+Shift
bindings were removed, while backward bindings, XKB options, input-source IDs,
and config SHA were preserved. lay-daemon again performs direct GNOME
`ActivateLayout`, legacy non-atomic IME Alt+Shift remains passive, and the IME
readout closes stale `preedit_fast` after whitespace-terminated handoff rebuilds
while preserving whole ASCII layout tokens with punctuation-shaped letter keys.

Build receipt:
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/build-result.json`
with `PASS_RUNTIME_BUILD_ONLY_GRAPH_UPDATED` in 136.82 s. Installation receipt:
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/installation.json`.
Loaded daemon PID `2128412` SHA
`7680d8680563d48d8591106cc852960137339535d4ee377d86a7b5763f63780e`; loaded IME
PID `2128417` SHA
`86f5ea13549ffeb473bc70959b934d734406c9ed336fb5c3a06b71415ed6b96b`; global
`ibus-daemon` PID `4715` was preserved. Rollback backup for this installation:
`/home/ubu/.cache/lay/development/layout-recheck-20260911-sy8bmz8b/backup/`.
Tests/CI denominator is 0; physical acceptance remains `PENDING`; no quality,
latency, RSS, or heldout claim is made.

# Historical: release 1.0.71 installation before Alt+Shift owner fix

At release installation, `1.0.71` was installed locally with status
`INSTALLED_VERIFIED_PHYSICAL_PENDING`. The loaded runtime matches candidate
`2ee1479cfc84c40ce3e8673e573d2dbfc42226f0e4dbdeb7cc310f6f1a63fcc5`:
IME PID `4152536`, daemon PID `4152503`, L3 PID `4152477`, and L1.1 PID
`4152326`. The global `ibus-daemon` PID `4715` was preserved; CLI and loaded
GNOME extension both report `1.0.71`; config, input sources, and the eight immutable model/data dependency payloads were
preserved; the L1.1 executable dependency changed from the accepted 1.0.70
service hash to `db825d2f244282392fe507ebeee335ef3e66ee33fc35f6e701a688abc7845034`.

Release 1.0.71 includes the accepted repairs for terminal word-boundary
replacement, clean repeated-word handling, candidate comparison, and
within-word Backspace/pending-learning feedback. It is installed runtime
authority for the verified local processes; physical keyboard acceptance is
still `PENDING`.

Evidence: fresh full identity `7585f7df394fa6d06c15181923d68b553f07f0bf788f3184a4b2baf5da7c36e7`,
acceptance summary `b08915f4bdf76e1951d25a98feb0abf4118fe308b189c3ddf41580ddf196a68d`,
installation receipt `8e6124c06c0f41543be7c077d84964cfcb3f919cd73d84ddd4a2cc19a665b9be`,
and post-install runtime receipt `fa9a979c2c1eefd367cbe7c7f24ebe15d655d0e85db0558fb32938449122d3a8`.
Changed/full gates passed `2756/2756` each; client checks passed
`17/17`; fixed89 stayed exact-output identical across three profiles
(`267/267`, including the known wrong-output counts `1/4/5`); diagnostic18 is
`14/18` with `7/11` dirty restored and `7/7` clean preserved.

The install cut over input journals at `2026-09-11T09:54:01.913546Z`: exactly
10 old journal files were deleted without archive, and L3 observed the new
empty usage journal alone (`source_offset=0`, `source_tail_hashes=[]`) before
daemon/IME started. The second install attempt observed L1.1
`socket_refused -> warming -> ready` and then ran one strict guard; there were
no command failures. The first failed attempt and recovery receipts remain a
truthful incident record, not the final state.

Exact graph completion is established only by
`/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/final-document-graph/fetch-receipt.json`
with `PASS`. Exact publication refs are established only by
`/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/publication.json`.

Current owning release evidence:
[release 1.0.71](docs/release-1.0.71-preflight-2026-09-11.md).

# Historical: poor-input mechanics installed before 1.0.71 release

2026-09-11T01:31:13Z local install completed with status
`INSTALLED_VERIFIED_PHYSICAL_PENDING`. Candidate IME
`aec4f55310e7aded386d037176070e709523e3d4a66a231c7fdd951a24920c99`
is loaded as PID `1652608` under preserved global IBus PID `4715`.
Changed verified consumers: `lay-daemon` PID `1652602`
(`be231d78fd01`), `lay-nanda-wave-train`/L3 online PID `1652603`
(`d2df74855c83`), and `lay-ibus-engine`. Preserved L1.1 PID is
`2729596`; source, config, extension, immutable dependencies, input sources,
and all release artifact identities were verified.

The accepted repair has three mechanisms: L2 absolute phase competition keeps
actual margins, generic L4 phase-bank availability is advisory until exact
transition evidence exists, and within-word Backspace keeps raw retention plus
bounded pending-learning feedback aligned with the Lay target. Private acceptance:
composed source proof `2052/2052`, changed/full both `2756/2756` with zero
failures, fixed89 `267/267` exact-output no-regression across three profiles,
diagnostic18 `10/18 -> 14/18`, and native client matrix `17/17`.

Installation receipt:
`~/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/installation-poor-input.json`
sha256 `706022c30e9f9bd3423db36340be6dcfe17ffdb41f1a4f87241ae88ae6af4551`.
Runtime receipt:
`~/.cache/lay/development/poor-input-20260911-xoi17mif/post-install-runtime.json`
sha256 `858931b908302163a85a92372ad3fb70b96df821870941383e2d412b87be55bb`.
Backup: `/home/ubu/.local/state/lay/release-backups/poor-input-20260911-bgoa6qf_`.
Installation review: `9/10`, H0/M0/L0. Physical keyboard acceptance remains
`NOT_TESTED`; no general quality, RSS, latency, heldout, or physical PASS is
claimed. Final document-graph verification is recorded at
`~/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/final-graph/fetch-receipt.json`;
only an existing `PASS` receipt there establishes graph completion. Full facts and limits are in
[poor-input authority](docs/poor-input-authority-2026-09-11.md).

# Historical: terminal IME commit repair installed, physical check was pending

2026-09-10T13:17:32.727160+00:00 локально установлен IME `4bfe47fa3db1`
(PID 2182844); общий IBus4715 сохранён. Для legacy terminal обычные
буквы идут через native input, замена — одним commit. Пробел и курсор
сохранены в трёх классах длины; видимая подсказка проверена отдельно.
Changed/full: по 2730/2730; actual-client: 17 отдельных случаев;
финальное независимое ревью 9/10, H0/M0/L0. Прежний cold-hint отказ
воспроизведён на обоих бинарниках и отделён от принятого post-ready сценария.

Один owning document: [terminal commit order](docs/ime-terminal-commit-order-2026-09-10.md).
Установка и откат:
`~/.cache/lay/development/ime-replacement-cursor-20260910-7xc45bzi/installation-terminal-ime.json`.
Это локальный патч с меткой 1.0.70; новый тег и публикация не выполнялись.
Не повторять установку и пройденные проверки. Следующий шаг — пользовательский
ввод в настоящем окне; GNOME/Kitty physical acceptance пока PENDING.

# Historical: Lay 1.0.70 GitHub fixes installed runtime

Исправлен #42: обычные начальные `-`, `_`, `=` и `+` сохраняются в daemon
буфере вместе с буквами и участвуют в точном replay. Для #43/#44 установщик
собирает десять устанавливаемых целей с видимым прогрессом, сохраняет журнал
и код ошибки/отмены; добавлена пропущенная зависимость `jq`.
Changed/full: по 2726/2726; installer: 8/8; final static review: 9/10 H0/M0/L0.
Проверенный комплект установлен 2026-09-10 01:07 UTC; изменился только daemon
`133a1f79e57b`, девять остальных бинарников идентичны первой 1.0.70.
Четыре загруженных процесса проверены, общий IBus4715 сохранён.
Установка: `~/.cache/lay/development/run-j4e7w_0n/installation-1.0.70.json`.

Публичная поставка: `public/main`, тег `v1.0.70`; private источник:
`origin/codex/cleanup-20260908`. Точные commit/ref и фактическое завершение
публикации проверять по `~/.cache/lay/development/github-issues-70-3ps08xs0/publication.json`.
Полная область, ARM compatibility и ограничения:
[owning public-issues document](docs/public-issues-42-44-release-1.0.70.md).
В исходных Ubuntu логах причина остановки не установлена; повторная установка
на устройствах авторов #43/#44 UNOBSERVED. Физическая клавиатура PENDING;
разрешение публикации не считается её подтверждением. Общий TD-123 и «зуын»
→ «push» OPEN. Не повторять принятую сборку/установку и не включать счётчики.

# Предыдущая установка 1.0.70: физическая проверка автопереворота ожидалась

Исправлена потеря подтверждённого начала слова после удаления наблюдённого
пробела и повторного набора. Причинный native контроль: 1.0.69 3/4 → 1.0.70 4/4;
неизвестный текст остаётся без полномочия автозамены.
Полные тесты 2723/2723 PASS; три fixed89 профиля без регрессий; native 13,
first-word GUI 7/7 и quiet legacy 5/5 PASS. Финальный review 9/10, H0/M0/L0.
Все десять бинарников, четыре загруженных процесса, CLI и расширение сверены.
Общий IBus PID 4715 сохранён. Установка: `~/.cache/lay/development/run-p4d_z81t/installation-1.0.70.json`.
Физическая проверка запрошена 2026-09-09 21:42 UTC и PENDING; commit/push
1.0.70 ещё не выполнены. Не повторять установку или неизменённые проверки.
«згыр» → «push» — чистая смена раскладки; «зуын» даёт «pesy» и требует
дополнительного восстановления двух букв. Этот отказ и общий TD-123 OPEN.
Счётчики не возобновлять. Все классы, ресурсы, ограничения и receipts:
[owning IME architecture](docs/ime-daemon-route-map-2026-06-20.md#retained-boundary-final-validation-and-installation-2026-09-10).

# Lay 1.0.69: подсказки приняты, публикация разрешена

Пользователь сообщил «Push отлично», затем открыл следующую проблему:
автопереворот не срабатывает, пример «зуын» → «push». Публикуется уже
установленное исправление подсказки первого слова и явного дополнения Tab.
Это пользовательская приёмка данного результата; приложение последней
проверки не уточнено. Новый отказ автопереворота и общий TD-123 остаются OPEN.

Полные проверки: 2717/2717; final-byte native первого слова: 7/7;
три профиля по 89 фиксированным сценариям без регрессий; ещё 13/13 native
контролей. Source review 9/10 H0/M0; installer/provenance review 10/10 H0/M0/L0.
Все десять бинарников установлены; CLI, расширение и четыре загруженных
процесса проверены как 1.0.69. Общий IBus PID 4715 сохранён.
Установка: `~/.cache/lay/development/run-btt4ilfz/installation-1.0.69.json`.
Авторизация: `~/.cache/lay/development/run-btt4ilfz/physical-acceptance-1.0.69.json`.
Точный commit и remote ref проверяются в
`~/.cache/lay/development/run-btt4ilfz/publication-1.0.69/`.
Не повторять установку и неизменённые проверки.

Прежний intermittent-hint report и старая библиотека Kitty сохранены как
ограничение доказательств в owning IME architecture. Нельзя приписать все
старые жалобы одному приложению по focused-window readback. Старые вкладки
и общий IBus не перезапускались; оба отменённых счётчика остаются выключены.
Новая диагностика автопереворота:
`~/.cache/lay/development/auto-layout-push-k9vjrshc/`.

# WeChat: пользователь подтвердил работу и разрешил публикацию

2026-09-09 пользователь сообщил «Теперь в окнах Wechat не работает ничего!».
WeChat PID 3734978, общий IBus PID 4715 и Lay 1.0.68 продолжают работать;
IME в `/proc` имеет прежний SHA `2a2df132`. Активны источники `lay-ime-us/ru`.
Новый private native контроль воспроизвёл общий отказ GUI-маршрута: `про`
в пустом поле не показывает подсказку, ` про` после наблюдённого пробела
показывает, ввод в середине существующего слова остаётся без подсказки.
Три сценария завершены: один ожидаемый отказ, два контроля PASS; все private
процессы убраны. Это не физическая приёмка WeChat и не его полный диагноз.
Причинная граница — `UnknownStart` до расчёта кандидатов; снимок surrounding
text не подтверждает начало слова в текущем reducer. Проектирование открыто:
IBus передаёт относительный фрагмент, а не отдельное доказательство начала
документа, поэтому одного `cursor=0` для общей выдачи полномочий недостаточно.
Код и установленный runtime не менялись. Затем пользователь ответил
«да заработало пушь»: работа WeChat подтверждена пользователем, публикация
разрешена. Это не отменяет отдельный воспроизведённый отказ первого слова
и не является измерением всех функций. Счётчики не возобновлять.
Доказательства: `~/.cache/lay/development/wechat-ime-no-functions-8isocthh/`;
подробности и варианты — в
[owning IME architecture](docs/ime-daemon-route-map-2026-06-20.md#wechat-report-and-first-word-admission-2026-09-09).

# Kitty: подсказки при наборе приняты пользователем

2026-09-09 после публикации 1.0.68 пользователь уточнил: на «прове» нет
видимой подсказки, а Up/Down её показывает. Проверен конечный клиент Kitty
0.48.2: live worker вернул 10 кандидатов за 2.6 ms и отправил «рка» до стрелок.
Причинный контроль исходного обработчика Kitty воспроизвёл потерю preedit
после commit: 4 RED из 13, остальные 9 контролей PASS. Минимальный патч
порядка обработки и сброса существующего cache даёт 13/13 PASS, ASan/UBSan
чистые; независимый review 9/10, H0/M0/L0. Прямого захвата исходного пакета
нет. Модуль собран на закреплённых в Kitty
зависимостях Wayland 1.24.0 / protocols 1.45; ABI и загрузка проверены для
установленного оригинала, rebuilt baseline и candidate. Открыто отдельное
окно «Lay: проверка подсказок», PID 1891726, загружен candidate `9cbbe9f7`.
На вопрос о наборе «про» / «прове» без стрелок пользователь ответил
«да пушь»: физическая проверка отдельного окна принята, публикация разрешена.
Receipt: `~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/physical-acceptance.json`.
После повторного сообщения «Tab дополняет, но подсказки нет» live focus
указал прежний Kitty PID 272166 со старым модулем. 2026-09-09 12:20 UTC
принятые байты `9cbbe9f7` атомарно установлены в основной Kitty; обычный
launcher запустил PID 3048061 с теми же SHA и новым inode. Окно называется
«Kitty: исправление установлено». Прежний PID сохраняет старый mapped inode
до перезапуска; он и глобальный IBus не перезапускались.
Receipt: `~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/normal-installation.json`.
Пользователь подтвердил «в новом окне работает все отлично!»; физическая
приёмка основной установки записана в `normal-physical-acceptance.json`
в той же папке. Текущий сценарий Kitty закрыт; старые окна требуют перезапуска.
Backup: `~/.local/state/lay/compat-backups/kitty-0.48.2-preedit-2lmgtmmm/`.
Патч и доказательства уже опубликованы commit `57e91b65`; версия Lay 1.0.68
и его runtime не менялись. Owning record:
[IME architecture](docs/ime-daemon-route-map-2026-06-20.md#missing-typed-suggestions-restored-by-arrows-2026-09-09).
Счётчики не возобновлять. Приёмка ограничена этим сценарием; общий TD-123
остаётся OPEN. Ветка публикации `origin/codex/cleanup-20260908`; точный commit
и remote-ref receipt: `~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/publication/`.

# Lay 1.0.68 принят пользователем

Текущий этап 2026-09-09 07:19 UTC: `DELIVERY_ACCEPTED`; релизный milestone DONE.
Исправлена воспроизведённая зависимость IME-подсказки от необязательного
уведомления курсора при старом протоколе фокуса. Изменена одна runtime-функция;
KnownStart, идентичность слова/поля, сроки публикации и verifier сохранены.

Установлены десять проверенных бинарников 1.0.68. IME `2a2df132`, PID 536035
запущен сохранённым IBus PID 4715; L1.1 `570f35b9` PID 535803, daemon
`b17a4fc7` PID 536029, L3 watcher `964f2265` PID 536030. SHA файлов и /proc,
CLI и загруженное расширение 1.0.68 проверены; модели, config и источники ввода
сохранены. Receipt:
`~/.cache/lay/development/run-hvrxwcxk/installation-1.0.68.json`.
Backup: `~/.local/state/lay/release-backups/1.0.68-td123-4h4357ne/`.

2711 correctness/package и обязательные release gates завершены; отдельный
exit75 по лимиту Cargo сохранён, продолжение использовало те же проверенные
исходники. Fixed89 во всех трёх профилях без регрессий к 1.0.67. Native13 PASS;
legacy-focus hints 8/8, обе опечатки исправлены по 8/8, clean24/24 при 80 ms.
Narrow terminal hints PASS, а Space correction в этом private harness остаётся
3/5 как у baseline: физического daemon в нём нет. Это не приёмка всего desktop.
Review runtime 9/10 H0/M0/L0; три замечания второго review к установщику
исправлены, семь проверок его failure paths прошли. Полные факты и ограничения:
[owning IME architecture](docs/ime-daemon-route-map-2026-06-20.md#rare-ime-suggestions-after-the-spontaneous-input-report-2026-09-09).

После проверки обычным набором пользователь подтвердил «работает !» и разрешил
push. Это приёмка текущего IME-сценария; общий TD-123 quality остаётся OPEN.
Не повторять установку или неизменённые проверки. Публикационная ветка:
`origin/codex/cleanup-20260908`; исходный main сохранён. Точный результат push
проверяется удалённой ссылкой и receipt в
`~/.cache/lay/development/run-hvrxwcxk/publication-1.0.68/`.
В этой же папке `candidate-counts.json`: готовые списки на 31 расчёте содержат
0–12 вариантов; лимиты — 12 словесных + до 6 фразовых в Experimental,
на экране один выбранный. Полное внутреннее поле по буквам не измерено.
Все build/test/graph update только remote.

Пользователь сообщил, что точки прекратились, и отменил счётчик; оба наблюдателя
остановлены, MainPID=0. Не запускать их снова. Источник точек не объявлен
исправленным. Текущий маршрут — подсказки и автоисправление последней версии.

## История до исправления подсказок 1.0.68

Актуальное поручение 2026-09-09: исправлять последнюю 1.0.67. Пользователь
сообщил старый сбой IME в Kitty, отдельных окнах терминала, Tor Browser и
WeChat; точки появляются по одной без нажатий, после удаления возвращаются
с паузой. Это не установленная регрессия последнего релиза. Ошибочный откат
на 1.0.66 исправлен: возвращены те же проверенные байты 1.0.67 без сборки.
Receipt повторной установки 1.0.67:
`~/.cache/lay/development/ime-cross-app-failure-_ul2eo18/reinstallation-1.0.67.json`.
IME PID 3787313, L1.1 PID 3787090, daemon PID 3787309, L3 watcher PID 3787310
на момент повторной установки; глобальный IBus PID 4715 сохранён.

После сообщения пользователя «точки есть» захвачен эпизод 01:45:50–01:45:53 UTC:
96 нажатий точки пришли от IBus-соединения WeChat `:1.1961`, PID 3734978,
`/opt/wechat/wechat`; на трёх наблюдавшихся устройствах таких нажатий нет.
Принадлежность соединения проверена независимыми Peer.Ping/FD probes и парой
Unix-сокетов. Медианный интервал 30.2 ms совпадает с системным повтором 30 ms.
Источник запросов в IBus установлен, первопричина выше по цепочке ещё UNKNOWN.
В первой записи пропущен четвёртый клавиатурный вход: `2.4G Mouse`, event5,
USB 1ea7:0066. Поэтому физическое происхождение пока не исключено. Исправленный
наблюдатель v5 выбирает все устройства, объявляющие клавиши 52/53, из текущего
инвентаря, включая ресивер. Дальше различить устройство → X11 → WeChat → IBus.
Исправление не заявлено. Следующая пассивная запись охватывает XInput и IBus
с идентичностью поля/приложения при начале повтора; пользователь уже получил
просьбу оставить фокус в текстовом поле WeChat. Повторять откат, установку
или неизменённые полные проверки не надо.
[Доказательства и границы](docs/ime-daemon-route-map-2026-06-20.md#cross-app-spontaneous-input-2026-09-09).
Физическая приёмка и DONE/commit/push остаются открыты.

## Завершённая проверка установки до сообщения о старом сбое

Текущий этап на 2026-09-09: `INSTALLED_VERIFIED_PHYSICAL_PENDING`.
Установлены те же 10 проверенных бинарников 1.0.67; IME `13db8623`,
L1.1 service `db825d2f`. SHA файлов и /proc четырёх процессов совпали;
CLI и загруженное расширение показывают 1.0.67. Глобальный IBus PID 4715,
восемь модельных зависимостей, config и список источников ввода сохранены.
Backup: /home/ubu/.local/state/lay/release-backups/1.0.67-td123-1p1e47xu/.

Все 2711 correctness/package, lint и обязательные build gates прошли.
Исходный full-вызов и продолжение после исправления только byte-location
metadata сохранены раздельно. Fresh combined review: 9/10, H0/M0/L0.
L1 Gate C PASS: 260000 damaged + 852582 clean, точное качество 13 классов.
L2: 2600 случаев, качество baseline; формальный 5 ms FAIL при 5.276 ms.
Отдельная performance lane: 10/11 PASS, cold-prefix 301.738 ms > 50 ms FAIL.
Fixed89 во всех трёх safety profiles без регрессий; Experimental dirty
19/47 → 20/47, clean 39/42 → 40/42. Native13 PASS. В quiet 80 ms серии
обе ошибки исправлены по 8/8, clean 24/24 с конечным IME и сервисом L1.1.
Это ограниченный результат, а не общий quality PASS для любых слов.

Два вопроса о физическом наборе и Double Shift были заданы до сообщения о сбое.
Не повторять установку, сборку или неизменившиеся проверки. После ответа
выполнить соответствующую приёмку; общий TD-123, DONE/commit/push пока OPEN.
Остаточные механизмы качества описаны; переобучение не выбрано.
Точные receipts: ~/.cache/lay/development/run-o0dqrl5c/ и
~/.cache/lay/development/autocorrect-live-ojoasco5/; remote run-tovK3y.
Полная запись установки и границ доказательства — в последних разделах
[owning architecture](docs/l2-l11-canonical-architecture.md).
На переходах показывать дерево с `✓` и `← СЕЙЧАС` — указание пользователя.

Ветка: codex/cleanup-20260908. Исходный snapshot-коммит:
cb40ef29f6c78c97757dd0059c0ba798cb1f0789.

Пользователь поручил очистку новой копии, ориентир до80%, и отдельно потребовал
сохранить текущие результаты автокоррекции и продолжить их после очистки.
Работать только здесь; исходные деревья сохранены. Ограничение очистки на
install/restart соблюдено; последующая проверенная установка относится к
отдельно порученному этапу TD-123.

Полная точка восстановления и продолжения:
/home/ubu/.local/state/lay/project-snapshots/20260908-before-cleanup/RESTORE_AND_CONTINUE.md

[План и результат очистки](docs/project-cleanup-2026-09-08.md) ·
[Восстановление истории](ARCHIVE.md) ·
[Автокоррекция: сохранённое состояние](tech_debt/CONTINUE.md).

Проверенный архив6173файлов и исходный бинарник/ключевые receipts сохранены
в том же каталоге. Это не disposable workspace/cache новой ветки.
Очистка:4 800 исходных файлов удалены,2 684/2 684 correctness/package PASS,
46/46 architecture-tooling PASS,3 installer suites PASS,review9/10,H0/M0/L0.

История TD-123, до текущей установки. Установленный995b на2026-09-08:
19/47 dirty correct,39/42 clean preserved.
Частная сборка37073948 уменьшила повторы L3,2209 тестов и126 сравнений readout
прошли, но80ms всё ещё опаздывает; fixed89 выявил36/42 clean preserved.
Она ОТКЛОНЕНА для установки. Следующая частная сборкаfa15601f согласовала
существующее правило коротких layout-целей:2211 tests PASS, review9/10,
fixed89 снова19/47 dirty и39/42 clean, все классы совпали с995b;13 client cases PASS.
Фиксированные8 native повторов при80ms: своевременное исправление0/8→7/8,
clean24/24 у обеих сборок; gate8/8 НЕ пройден. Один срыв:22.3ms очередь+
80.6ms расчёт. Это метрика доставки одной фразы, не точность на8 разных словах.
Продолжить устранение опоздания к Space и отказов готового расчёта.
Правка повторного decoder reconstruction прошла component parity и review;
частнаяeea32f44 прошла fixed89 без изменений и13 native controls. В новой
парной серии80ms она успела8/8 против5/8 уfa15601f, clean24/24 у обеих.
Ограниченный native readiness gate пройден; теперь отказы готового расчёта.
Установки нет; общий TD-123, модельное качество и физическая приёмка открыты.
Ready-refusal audit завершён:23/25 целей удержаны,21 без L3/exact-L4 authority;
один проверенный lexical Winner теряет разрешение на общей L4 ambiguity.
Ограниченный перенос frame capability реализован после анализа вариантов9/8/4.
RED воспроизвёл общий отказ, итоговый scoped GREEN2 221/2 221, code review9/10,
H0/M0/L0. Частная7a58535c прошла весь fixed89:19/47→20/47 dirty,
clean39/42 без изменений, новых wrong outputs нет;13 native controls PASS,
8/8 своевременных исправлений при80ms, наблюдаемый RSS до352.53MiB.
Общий quality/release/physical PASS ещё не получен. Остальные21 missing-authority
случай открыты. Последствия и L1.1 наблюдения43 токенов — в owning architecture.
Полный результат и доказательства — в tech_debt/CONTINUE.md. Физический обычный
набор PENDING. Очистка файлов не является приростом качества модели.

Следующий общий дефект — raw-capacity L2 — исправлен в частной1bceb869.
Все оставшиеся final L2 frames теперь полные; ограничения поиска и74/4 сохранены.
Fixed89: dirty20/47 без изменения, clean40/42, новых ошибок нет. Scoped2227 PASS,
review9/10, native13 PASS; новая полная серия80ms8/8 у обеих версий. Первую
неполную серию из-за ошибки имени варианта в private harness сохранили отдельно.
Дальше: неподдерживаемая геометрия и независимый контекст для complete ties.
Точные границы, все проценты и receipts — конец owning architecture.

Все builds/tests/architecture refresh удалённо, dedicated-20cpu/Cargo guard.
Сохраняются требования AGENTS.md о последствиях, независимом review,
SafetyGate/verifier, единственном владельце Double Shift и fixed proof.
