# Lay — очередь технического долга, 2026-10-10

Это единственный текущий индекс задач. Исторические статусы внутри карточек
относятся к названным в них версиям и не переопределяют эту очередь.
[Аудит и варианты решений](AUDIT_2026-10-09.md) ·
[План: независимое ревью, ACCEPT 9/10](evidence/2026-10-09-debt-plan-review-pass2.md) ·
[Правила](../AGENTS.md) · [Архитектура](../ARCHITECTURE.md) ·
[Remote development](../DEVELOPMENT.md).

## Текущий источник и принятая версия

Authoritative checkout: `/home/ubu/projects/lay-space-boundary-shift-20261007`.
Branch: `codex/space-boundary-shift-20261007`.
Принятый исходный checkpoint:
`e7a25705bb6196776c35c0e3cb7c3b5a17d55169`, 1.0.81; опубликован в
`origin/codex/space-boundary-shift-20261007` и `public/main`.
Новые task commits добавляются поверх него; старый dirty `/home/ubu/projects/lay`
не является источником принятого runtime.

Последний принятый пользователем IME 1.0.81 SHA-256:
`2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559`.
Приёмка пользователем закрывает конкретное исправление
`должн ыбыть → должны быть`, включая первую пару после Enter. Installed hash
повторно прочитан при TD-129; receipt относится к этой точной версии.

Текущий экспериментальный IME: **1.0.82**, installed и loaded SHA-256
`5fe100db732bcc945d9c7d584a2c22bd06eb93c19bee09cc39788c85470dee7a`.
10 октября пользователь отдельно разрешил установку и один four-stream batch:
GTK3 Entry/Qt rich **4/4 PASS**, первый Space/committed text/preedit/caret.
[Точный результат](evidence/2026-10-10-td133-ime82-installed-four-stream-result.json).
Production source опубликован в `fc38abd4`; full source3048/3048 и11/11performance
PASS. Extension остаётся1.0.81; другие runtime роли/global IBus/RU/config
сохранены. Это экспериментальная установка, полная приёмка82 ещё не закрыта.

| Доказательство принятой версии | Измеренный результат | Граница |
| --- | --- | --- |
| Source release gate | 3044/3044, 3008 correctness + 36 package | Исходники, не все окна |
| Целевой Kitty | 7/7 и отдельно 3/3 | Точные сохранённые сценарии |
| Native fixed matrix | 62 PASS / 2 FAIL / 0 BLOCKED из64 | GTK3 Entry и Qt rich rapid six-letter LEFT controls остаются FAIL; причина UNKNOWN |
| Browser input | 10/10 | Исходные input cases |
| Browser dependent cases | Отдельно20/20 после exact owned cleanup | Исходный24/30 FAIL сохранён, не заменён исправленной выборкой |
| Universal client acceptance | NOT_MET | Новая версия не получает старую приёмку автоматически |

Exact receipts и последствия:
[Space boundary owning document](../docs/architecture/space-boundary-shift-2026-10-07.md),
[аудит: пути и счётчики](AUDIT_2026-10-09.md#принятая-точка-и-границы-ревью).
Полные private receipts лежат в
`/home/ubu/.cache/lay/development/kitty-boundary-refusal-20261009/`.
`CURRENT_RESULT.json` — immutable pre-publication receipt: его старое
`release_pushed=false` описывает момент записи, не сегодняшний Git checkpoint.
Рабочие журналы, профили и credentials не коммитятся.

## Этап1 — минимальная разрешённая программа

| Порядок | Задача | Приоритет / статус | Приёмка |
| --- | --- | --- | --- |
| 1 | [TD-129 — current index](129-current-debt-and-acceptance-index.md) | P0 / DONE_METADATA_SCOPE, review9/10 | Один индекс, карта всех прежних scopes; metadata review, без новых functional tests |
| 2 | [TD-131 — один Unicode tail limit](131-one-unicode-tail-limit.md) | P1 / DONE_SOURCE_ONLY, review9/10 | Characterization обоих existing producers; fixed160 helper, remote affected contracts, graph/canon, independent review |

До protected production change TD-131 нужен новый explicit decision и owning
entry, перечисленные в карточке. Граф обновляется canonical remote wrapper.
После каждого завершённого task: scoped DONE → commit → push обоим названным
remotes → exact ref verification → чистый worktree → следующий task.
TD-129 завершён и опубликован: `2401a4e4dca5f99cc87c42415ab52c108c26bd74`.
Этап1 завершён: TD-129 и TD-131; разрешённое продолжение описано в этапе2.
TD-131 final source packet: [evidence](evidence/2026-10-09-td131-final-source.json);
publication receipt: `/home/ubu/.cache/lay/development/td129-publication-20261009.json`.

## Этап2 — разрешённая работа, 2026-10-09

Таблица отсортирована по продуктовой важности; prerequisites определяют порядок
исполнения. Пользователь разрешил продолжение («делай»). Работа начинается с
причинного TD-133; карточка/разрешение не означают доказанный runtime fix или DONE.
Первый разрешённый local diagnostic batch выполнен: два visible controls
PASS, исходный сбой не повторился; [измеренная сводка](evidence/2026-10-09-td133-local-diagnostic.json).
Второй отдельно разрешённый batch восстановил исходный same-RU activation
setup: два committed-text/preedit/caret controls PASS, но GTK assertion layout
содержит лишнее `ыбыть`; поздний сохранённый layout совпал. Qt render_text
недоступен. [Раздельные наблюдения](evidence/2026-10-09-td133-original-activation-result.json)
не дают rendered/pixel acceptance или причины original FAIL.
Третий отдельно разрешённый four-stream batch также выполнен на старых2bd:
4/4committed-text/preedit/caret PASS; original failure not reproduced. Все три
прежних local grants исчерпаны. Новый отдельный install+four-stream grant
выполнен один раз на82/5fe100db:4/4PASS,0FAIL,0BLOCKED; он также исчерпан.
Остальные builds/tests/graph refresh remote-only. Original native62/64 и два
FAIL сохраняются. Source review9/10 и full3048+11performance PASS завершены;
новые четыре positive controls не закрывают original64/native acceptance.

| Приоритет | Задача / статус | Первый результат и зависимости |
| --- | --- | --- |
| P0 | [TD-138 — IME latency proof authority](138-bind-ime-latency-proof-to-runtime-authority.md), DONE_SOURCE_SCOPE | Existing repeated-run geometry before lexical lookup; isolated RED then IME/reference6/6 GREEN; full3048+11performance PASS, final review9/10,2passes. All old FAILs retained; startup/native/RSS NOT_TESTED |
| P0 | [TD-133 — rapid Space ordering](133-causal-rapid-space-ordering.md), REPLAN_REQUIRED_NATIVE_ACCEPTANCE | Scoped CommitText Space + Reset release receipt; implementation review9/10,2passes; complete source3048+11performance/lint/release PASS. Experimental82/5fe100db installed, new-byte GTK3/Qt rich4/4PASS. Original62/64 retained; all local grants consumed. Changed-byte native64 requires safe actual four-path isolation and fresh authorization |
| P1 | [TD-130 — case-bound feedback](130-offline-inverse-feedback-case-binding.md), DONE_CONNECTED_PRIVATE_PROOF | 14 new semantic tests, controlled RED,2 real private IBus cases PASS; review9/10,2passes; shared cleanup/recovery остаётся137 |
| P1 | [TD-137 — isolation / unsafe cleanup retirement](137-owned-feedback-partial-cleanup-recovery.md), DONE_RETIRED_UNSAFE_SHARED_CLEANUP, review9/10 | Пользователь выбрал isolation/retirement; guarded private entrypoint,12 historical identities, unchanged TD-130 proof reused; live recovery/native inverse не заявляются |
| P1 | [TD-128 — Chrome после Tab](128-chrome-focus-transfer-autocorrect.md), OPEN | Точный исходный переход поля на нынешних bytes; ownership/Reset contracts сохранить |
| P1 | [TD-127 — Kitty Tab/Space](127-kitty-space-correction-diverges-from-tab.md), OPEN | Один frozen frame и first divergence, без literal-word exception |
| P1 | [TD-121 — whole-word handoff](121-preserve-word-across-ime-layout-handoff.md), OPEN_CURRENT_SCOPE | Original Firefox scope остаётся открытым. Новый отдельный [экран10полей](../tests/manual/firefox_double_shift.html): оба направления/следующая буква, отдельные режимы слова и окна; fixture source verification не заменяет native acceptance |
| P1 | [TD-123 — Wave quality](123-improve-wave-restoration-quality-for-1.0.67.md), OPEN_EXTERNAL_OWNER | Единственный current roadmap в syntax-agreement checkout; stage/protocol оттуда, не второй fit здесь |
| P2 | [TD-122 — LegacyV1](122-bind-legacy-replay-suppression-request.md), DECISION_REQUIRED | Current reachability, synthetic stream completion, backward policy до протокола или retirement |
| P2 | [TD-136 — functional gaps](136-functional-test-gaps-and-test-file-boundaries.md), DONE_BOUNDED_MAP_AND_SOURCE_CHARACTERIZATION, review8/10 | 12-family map; one joined caps41 callback test;3045 successful bodies + separate metadata admission, original command FAIL preserved; native128/133 remain open |
| P2 | [TD-134 — component boundaries](134-god-component-boundaries-without-new-owners.md), DEFERRED | После136: concrete ownership/coupling benefit; новый leaf сохраняет guard protection |
| P2 | [TD-132 — retired-preedit predicates](132-isolate-inert-retired-preedit-predicates.md), DEFERRED | Уже есть pure block и semantic negatives; extraction только при измеримой maintenance пользе, не prerequisite134 |
| P3 | [TD-135 — research/runtime boundary](135-research-runtime-build-boundary.md), DEFERRED | Reachability/build baseline; public exports не доказательство RSS/bloat |

## Все прежние карточки: сохранённый scope и current map

| Task | Исторический verdict | Что известно для2bd / следующий шаг |
| --- | --- | --- |
| [113](113-restore-hybrid-nanda-autocorrect.md) | DONE, hybrid source contract | Сохранить single lattice/authority и proof gates; не новая heldout quality приёмка |
| [120](120-scope-autocorrect-suppression-to-word-lifetime.md) | DONE, bounded ordinary/atomic suppression | LegacyV1 debt отдельно122; общий transport/release PASS не приписывается |
| [124](124-reproducible-maintenance-loop.md) | DONE, remote tooling | Использовать existing dev-check; focused PASS не client acceptance |
| [125](125-preserve-autocorrection-left-boundary.md) | DONE в recorded installed/full/four-client scope | Сохранить stale KnownStart revocation; старый installed receipt не current2bd universal PASS |
| [126](126-common-window-interaction-module.md) | DONE_SOURCE_ONLY | Existing observation/authority/execution composition; это не отдельная current installation |
| [121](121-preserve-word-across-ime-layout-handoff.md) | Поздняя history до2026-09-26; не R5 latest | Original first-word/mixed-prefix/completion/closing-Space Firefox scope на2bd UNKNOWN. Last Kitty a8b receipt: `kitty-focus-proof-20260925/kitty-tab-cycle-aligned/receipt.json`, visible `просто `, two Double Shift; changed gate2904/2904 отдельно. Current2bd native fixed matrix имеет successful eight-Double-Shift cells, но не закрывает original Firefox denominator. Сначала map exact original cases/first loss, затем обсуждение нового proof |
| [127](127-kitty-space-correction-diverges-from-tab.md) | USER_REPORTED / NOT_REPRODUCED | `которую` visible/Tab против `котором` onSpace; frozen receipt отсутствует, first layer и current2bd UNKNOWN. Нужен один identity-bound frame обоих путей |
| [128](128-chrome-focus-transfer-autocorrect.md) | Physically reproduced наdaaa… | `пу` в первом textarea → Tab → `публекует ` во втором без reactivation; original receipt `browser-autocorrect-20260922/physical-chrome-final-accepted-combined/receipt.json`, SHA6f469b2a…60fa. Current2bd exact scenario UNKNOWN; fresh-field/pair proof не эквивалентен. Исследовать FocusOut/FocusIn/first-printable/Space, сохранить11 rejected-contract negatives |
| [139](139-account-for-owned-preedit-in-private-client.md) | DEFERRED_STAGE2_DISCUSSION / P1 | General V3 Client advertises preedit but ignores it; unchanged lifecycle0/3 FAIL on81 and82. Strict consumer/proof revision proposed, no runtime changes selected |
| [122](122-bind-legacy-replay-suppression-request.md) | DECISION_REQUIRED | Source no-arg LegacyV1 dispatch/handler присутствуют; live current reachability и stream-completion authority UNKNOWN. Last owning inventory: [admission analysis](evidence/td120-suppression-admission-analysis.md); protocol не изобретать до trace/backward policy |
| [123](123-improve-wave-restoration-quality-for-1.0.67.md) | ACTIVE_QUALITY / general quality OPEN | Historical poor-input aec4… installation и267/267 fixed89 outputs не general quality promotion. Current2bd heldout/per-class quality UNKNOWN. [Owning history](../docs/poor-input-authority-2026-09-11.md); current owner — `/home/ubu/projects/lay-syntax-agreement-20260929/docs/architecture/ru-agreement-roadmap-2026-09-30.md`. Читать его stage, baseline и R1/R2 gates перед любым fit |

Относительные private receipt имена выше разрешаются через исходную карточку,
где сохраняются original full path/hash. Это исторические наблюдения, не новые
проверки. Все10 прежних numbered cards113,120–128 учтены; прежние scopedDONE
не переоткрываются косметическим изменением индекса.

## Правила исполнения и откат

1. Зафиксировать actual source/task baseline, affected invariant, existing owner,
   первый провал и untested scope. Начать с RED либо честного GREEN characterization.
2. Использовать existing reducer/DecisionCore/verifier/AuthorizedEdit/output.
   Не вводить literal exceptions, второго owner, timer/queue/cache/fallback.
3. Все builds/tests/architecture refresh — remote-only под existing resource и
   Cargo guards; `python3 scripts/dev-check.py check --compact`, explicit target
   лишь inner loop. Full release нужен перед будущей installation/release.
4. После реализации — новый reviewer `fork_turns=none`, score1–10, максимум
   initial + один grouped repair/final pass. Открытый correctness finding
   сохраняет OPEN/REPLAN_REQUIRED; score не заменяет objective proof.
5. Для source-only задач runtime authority NOT_CHANGED и physical NOT_TESTED.
   Delivery routes, models, learning weights и принятый2bd runtime сохраняются.
6. Откат отдельной задачи — её Git revert; не установка старого C20, не откат
   пользовательского feedback. Raw receipts сохраняются неизменными.

[Обе точки продолжения](CONTINUE.md) · [Root checkpoint](../CONTINUE.md).
Старый индекс R5/C20 сохранён в
[immutable e7a history](https://github.com/radislabus-star/lay-public/blob/e7a25705bb6196776c35c0e3cb7c3b5a17d55169/tech_debt/README.md).
