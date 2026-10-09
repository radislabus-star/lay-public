# Lay — очередь технического долга, 2026-10-09

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

Принятый installed IME SHA-256:
`2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559`.
Приёмка пользователем закрывает конкретное исправление
`должн ыбыть → должны быть`, включая первую пару после Enter. Installed hash
повторно прочитан при TD-129; новые source-only задачи этот бинарник не заменяют.

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
Этап1 завершён: TD-129 и TD-131; следующий этап — отдельное обсуждение
открытых сложных задач. TD-131 final source packet: [evidence](evidence/2026-10-09-td131-final-source.json);
publication receipt: `/home/ubu/.cache/lay/development/td129-publication-20261009.json`.

## Этап2 — разрешённая работа, 2026-10-09

Таблица отсортирована по продуктовой важности; prerequisites определяют порядок
исполнения. Пользователь разрешил продолжение («делай»). Работа начинается с
причинного TD-133; карточка/разрешение не означают доказанный runtime fix или DONE.
Отдельный разрешённый local diagnostic batch выполнен: два visible controls
PASS, исходный сбой не повторился; [измеренная сводка](evidence/2026-10-09-td133-local-diagnostic.json).
Это узкое исключение исчерпано; остальные builds/tests/graph refresh остаются
remote-only. Original native62/64 и два FAIL сохраняются.

| Приоритет | Задача / статус | Первый результат и зависимости |
| --- | --- | --- |
| P0 | [TD-133 — rapid Space ordering](133-causal-rapid-space-ordering.md), IN_PROGRESS | Оригинальные trace отсутствуют; разрешён separate two-field diagnostic без inverse и shared cleanup. Новый полный physical proof требует130/137 либо эквивалентно доказанной безопасной границы |
| P1 | [TD-130 — case-bound feedback cleanup](130-offline-inverse-feedback-case-binding.md), DEFERRED | Named versioned consumer, immutable case binding, проверка receipt status; standalone offline selector не закрывает дефект |
| P1 | [TD-137 — partial cleanup recovery](137-owned-feedback-partial-cleanup-recovery.md), DEFERRED | Existing loader generation и failure injection на temporary data; неизвестный/частичный результат останавливает dependent cases |
| P1 | [TD-128 — Chrome после Tab](128-chrome-focus-transfer-autocorrect.md), OPEN | Точный исходный переход поля на нынешних bytes; ownership/Reset contracts сохранить |
| P1 | [TD-127 — Kitty Tab/Space](127-kitty-space-correction-diverges-from-tab.md), OPEN | Один frozen frame и first divergence, без literal-word exception |
| P1 | [TD-121 — whole-word handoff](121-preserve-word-across-ime-layout-handoff.md), OPEN_CURRENT_SCOPE | Карта исходных Firefox cases на2bd; новый pair PASS не заменяет их |
| P1 | [TD-123 — Wave quality](123-improve-wave-restoration-quality-for-1.0.67.md), OPEN_EXTERNAL_OWNER | Единственный current roadmap в syntax-agreement checkout; stage/protocol оттуда, не второй fit здесь |
| P2 | [TD-122 — LegacyV1](122-bind-legacy-replay-suppression-request.md), DECISION_REQUIRED | Current reachability, synthetic stream completion, backward policy до протокола или retirement |
| P2 | [TD-136 — functional gaps](136-functional-test-gaps-and-test-file-boundaries.md), DEFERRED | Bounded semantic test/effect map; named test counts не coverage% |
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
