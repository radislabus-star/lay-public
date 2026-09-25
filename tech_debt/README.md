# Latest Firefox repair — 2026-09-14

Latest TD-121: **R5_FINAL_NATIVE_FAIL, 2/4; DO NOT INSTALL R5**. The final
release gate passed 2,841/2,841 and four isolated client cells. Installed C20
is preserved. See [the current analysis](evidence/td121-r5-final-native-analysis-2026-09-14.md).
The R5 status below predates the final native failure.

TD-121 remains IN PROGRESS. R5 focused checks pass 545/545; the actual Firefox
accepted-completion word now projects twice and returns exactly. Earlier R3
native evidence covers first-word, mixed-prefix and closing-Space round trips.
Final R5 release bytes must pass all four scenarios together after the complete
release gate. C20 remains installed; no new installation or physical acceptance.
Exact current source decisions and receipts: [TD-121](121-preserve-word-across-ime-layout-handoff.md).

# Lay — текущая очередь

## Текущее состояние TD-121 — 2026-09-13

TD-121 is **IN PROGRESS / INSTALLED_VERIFIED_PHYSICAL_PENDING**. C20 passed the
complete 1.0.72 release gate (13/13 commands; changed and full gates each
2,807/2,807 with 11 intentional performance skips; compiled receipt; four
final-byte client cells), followed by a 3/3 owned GTK entry smoke.
Release result SHA-256:
`02458047a539fb85be82b301fd1cf38b19af1f71241f34260713cf5c5dd90534`;
GTK receipt SHA-256:
`27bf83fccd5a15e552dabd9afeada8f1bacfd9038b8f9a0046f2cb3d0fcd1eaf`.

Release 1.0.72 is installed. All ten installed artifacts and four loaded owners
match C20, and the loaded extension reports 1.0.72. Global IBus, configuration,
input sources, immutable models, journals and learner state were preserved.
Installation receipt SHA-256:
`ca7b0cb622f862cdb9a51678e640d27953fe798f3b37f291bdebce4f5e4735a4`.
C12 remains historical source-review evidence. Human physical-keyboard acceptance
is still pending, and TD-123 answer quality remains `UNKNOWN`.

Historical attempts and their exact receipts are retained only in
[evidence/td121-private-actual-baseline-2026-09-13.md](evidence/td121-private-actual-baseline-2026-09-13.md).

[Очистка новой ветки](../docs/project-cleanup-2026-09-08.md) завершена.
Текущий приоритет пользователя: пройти оставшуюся очередь по зависимостям,
закрывая и публикуя один доказанный task checkpoint перед следующим.
[Точка продолжения](CONTINUE.md) · [История и восстановление](../ARCHIVE.md).

## Очередь после текущего checkpoint

Четыре задачи (113, 120, 124, 126) сохраняют ранее принятый ограниченный
`DONE`. TD-121 остаётся активной до release, финальной клиентской, GTK/physical,
installation и publication приёмки. Порядок: **121 → 125 → 122 → 123**.

## Текущая очередь

| Задача | Статус | Следующий доказуемый результат |
|---|---|---|
| [TD-126: common window interaction](126-common-window-interaction-module.md) | DONE, source-only; commit/push подтверждены | Сохранять принятую композицию; отдельная установка этим результатом не заявлена |
| [TD-121: целое слово при IME handoff](121-preserve-word-across-ime-layout-handoff.md) | IN_PROGRESS; final R5 native FAIL 2/4 | Доказать и исправить причины двух native сбоев; R5 не устанавливать |
| [TD-125: левая граница автозамены](125-preserve-autocorrection-left-boundary.md) | DONE / installed; immutable full gate 2,884/2,884 and GTK/Chrome/Firefox/Kitty PASS | Сохранять release3 receipts и fail-closed отзыв stale KnownStart authority; commit/push только по явному запросу |
| [TD-127: Kitty Space расходится с Tab](127-kitty-space-correction-diverges-from-tab.md) | OPEN / user-reported / not reproduced | Заморозить один физический Kitty frame и найти первую точку, где Space выбирает `котором`, а Tab — `которую`; без literal-word fix |
| [TD-128: Chrome Space после смены поля](128-chrome-focus-transfer-autocorrect.md) | OPEN / физически воспроизведено | Разделить source-free и transferred authority после Tab; обе попытки смены preedit ownership отклонены тестами |
| [TD-122: legacy replay request](122-bind-legacy-replay-suppression-request.md) | DECISION_REQUIRED | После TD-125 заново проверить, нужен ли отдельный protocol binding; старый label сам по себе не закрывает task |
| [TD-123: качество восстановления Wave](123-improve-wave-restoration-quality-for-1.0.67.md) | OPEN / ACTIVE_QUALITY | После route tasks найти первый общий механизм по полной L1.1 → L2 → L3 → L4 → DecisionCore → verifier цепочке; весь fixed proof и каждый класс |
| [TD-113: hybrid source contract](113-restore-hybrid-nanda-autocorrect.md) | DONE | Сохранять принятую композицию источников и её strict gates |
| [TD-120: lifetime suppression](120-scope-autocorrect-suppression-to-word-lifetime.md) | DONE, scoped source/runtime | Сохранять lifetime/atomic settlement contract; это не общий release PASS |
| [TD-124: воспроизводимый maintenance loop](124-reproducible-maintenance-loop.md) | DONE, tooling scope | Использовать [DEVELOPMENT.md](../DEVELOPMENT.md); focused PASS не заменяет task acceptance |

Порядок активной работы фиксирован: **TD-121 → TD-125 → TD-122 → TD-123**.
Перед каждым новым task текущие зависимости
проверяются по живому source и receipts; исторический статус не переносится
автоматически.

## Operating Rules

The user-accepted [development simplification rules](../AGENTS.md#accepted-development-simplification--2026-09-07)
apply to this queue. They constrain the work loop; they do not waive release
gates or add a broad architecture migration to the 1.0.66 scope.

1. Follow the latest user priority and the current queue above. Historical
   completed tasks and their old execution order are preserved in the archive.
2. Start with a failing test or a frozen baseline. Do not mix behavior changes
   with move-only refactors.
3. Before edits, record the base commit, exact command, environment/toolchain,
   feature set, output or receipt hash, untested scope, and revert boundary.
4. After implementation, run an independent code review in a fresh context. It
   reports findings first and a score from 1 to 10. A review track has at most
   two total passes: the initial review and, only if needed, one grouped repair
   followed by the final second review. Never invent a third review.
5. An unresolved correctness finding keeps the task open. A score below 8/10
   triggers the single repair opportunity; unresolved findings after pass 2
   move the task to `REPLAN_REQUIRED` and never weaken an acceptance gate.
6. After objective gates and review pass, mark the scoped task `DONE`, record
   receipts, commit and push the task checkpoint, verify the remote ref and clean
   worktree, then start the next task.
7. Preserve the Lay 1.0.54 Double Shift ownership contract. IME work must also
   recheck candidate visibility, layout synchronization, and terminal
   passthrough.

Historical task completions, audit baselines, Stage 1/2 tables and reviews
are preserved at their original paths in the [snapshot](../ARCHIVE.md).
A historical DONE or PASS is not a new product-acceptance result.
