# Lay — текущая очередь

[Очистка новой ветки](../docs/project-cleanup-2026-09-08.md) завершена.
Текущий приоритет пользователя: продолжить автокоррекцию с сохранённого состояния.
[Точка продолжения](CONTINUE.md) · [История и восстановление](../ARCHIVE.md).

## Текущее состояние

IME `995b6093` установлен 2026-09-08; исправлены сброс первого cold V90
результата и ложный L4 negative transition из word prior. Для этих bytes
changed/full code gates прошли 2 684/2 684 каждый, 13 private client scenarios
прошли. Это не общий quality PASS.

На фиксированных 89 существующих native fixtures: 19/47 правильных
восстановлений, 25/47 abstain, 3/47 неверных; clean сохранены 39/42.
Проверка обычного физического темпа остаётся PENDING. Полные ограничения
сравнения, per-class результаты и exact receipts — в
[журнале TD-123](evidence/td123-live-autocorrect-2026-09-08.md).

## Очередь после очистки

| Задача | Статус | Следующий доказуемый результат |
|---|---|---|
| [TD-123: качество восстановления Wave](123-improve-wave-restoration-quality-for-1.0.67.md) | OPEN, текущая работа | Обычный темп клиента; общий механизм отказов по полной цепочке L1.1 → L2 → L3 → L4 → DecisionCore → verifier; весь fixed proof и каждый класс |
| [TD-121: целое слово при IME handoff](121-preserve-word-across-ime-layout-handoff.md) | IN_PROGRESS | Сохранить исправленный первый terminal token; combined two-field/profile gap и GTK/cold-preedit не закрыты |
| [TD-125: левая граница автозамены](125-preserve-autocorrection-left-boundary.md) | IN_PROGRESS | Физическое подтверждение variable-length замены; не терять предыдущее слово и разделитель |
| [TD-120: lifetime suppression](120-scope-autocorrect-suppression-to-word-lifetime.md) | DONE в scoped source acceptance | Сохранить contract; это не завершение всего релиза 1.0.66 |
| [TD-122: legacy replay request](122-bind-legacy-replay-suppression-request.md) | DECISION_REQUIRED | Отдельное решение протокола, не текущая очистка |
| [TD-124: воспроизводимый maintenance loop](124-reproducible-maintenance-loop.md) | DONE для tooling | Использовать [DEVELOPMENT.md](../DEVELOPMENT.md); focused PASS не заменяет release gates |

[TD-113: hybrid source contract](113-restore-hybrid-nanda-autocorrect.md)
сохраняется как владелец принятой композиции источников. Все production tests,
актуальные evidence 120–125, release 1.0.66 и обязательные contract inputs
остаются в рабочем дереве. Старые задачи не потеряны: полная очередь
восстановима из snapshot по пути `tech_debt/README.md`.

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
4. After implementation, run a fresh-context independent code review. The
   reviewer reports findings first and a score from 1 to 10.
5. Allow at most two correction passes. An unresolved correctness finding keeps
   the task open. A score below 8/10 triggers correction, but the numeric score
   does not override verified fixes after the two-pass limit; record the actual
   pre-correction scores and objective final gates without inventing a third
   review. Two passes that leave findings unresolved move the task to
   `REPLAN_REQUIRED`; they do not authorize a weakened acceptance gate.
6. Mark the task `DONE`, record tests and review evidence, then commit and push
   before starting the next task.
7. Preserve the Lay 1.0.54 Double Shift ownership contract. IME work must also
   recheck candidate visibility, layout synchronization, and terminal
   passthrough.

Historical task completions, audit baselines, Stage 1/2 tables and reviews
are preserved at their original paths in the [snapshot](../ARCHIVE.md).
A historical DONE or PASS is not a new product-acceptance result.
