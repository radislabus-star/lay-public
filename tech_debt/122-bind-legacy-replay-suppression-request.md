# TD-122: Привязать legacy replay suppression к конкретному запросу

Status: `DECISION_REQUIRED`
Priority: `P2`
Stage: 2 — отдельное решение, не часть автоматически допущенной TD-120
Scope: остаточный G120-1 / transport-owner contract первоначальной unified spec.

## Причина и границы

No-arg `SuppressNextAutocorrect` вызывается до replay и после успешного replay.
Он не передаёт request/token/phase; callbacks могут выполниться после смены
слова/owner. На error второго вызова нет. На ConfiguredBackend delegation нет
универсального physical grab. Поэтому receiving engine не может отличить
подготовку своего replay от запоздалого завершения старого запроса.
Подробный source inventory и reachability:
[admission analysis](evidence/td120-suppression-admission-analysis.md).

TD-120 типизирует это как LegacyReplayV1 и сохраняет совместимость; она не
решает универсальный lifecycle V1. Эта запись сохраняет исходный denominator,
а не переименовывает известную проблему в DONE.

## Варианты

| Вариант | Оценка | Решение |
|---|---:|---|
| TTL/чётность calls/угадывание строки | 2/10 | Не создаёт недостающую identity |
| Подмена V1 existing V2 | 5/10 | Generic delegation не имеет required exact handoff/capture/isolation |
| Checked begin/finish/cancel с request+origin/focus identity у existing owner | 7/10 условно | Рекомендуется исследовать: нужна доказанная обработка конца synthetic event stream |
| Переписать весь daemon/IME ввод | 3/10 | Не допущено, выгода не доказана |

## До кода

1. Измерить реальную достижимость V1 в supported clients; не путать latent
   ReplaceText branch с нынешним принудительным ReplayAll caller.
2. Frozen cases: before-only refusal/error, partial replay failure, duplicate,
   delayed after в новом поле/слове, successful replay с boundary/без, no-IME
   backend и exact V2 route. Фиксировать transport compatibility и false guard
   separately, не получать общий PASS из совместимости с известной ошибкой.
3. Доказать completion receipt synthetic events: return replay_keycodes или
   синхронного D-Bus after-call сам по себе этого не доказывает.
4. Выбрать backward policy для старого daemon/new IME и обратного сочетания.
   Rejected request не должен запускать второй output/fallback/retry.
5. Согласовать canonical focus TD-121 и atomic revision TD-120 без нового
   параллельного owner/worker. No-arg V1 retire только с проверенной migration.

## Приёмка будущего решения

Request binding, origin/token/owner validation, exact cancellation, queue/race
tests, bounded latency/resources, no learning pollution, single mutation,
unchanged verifier/SafetyGate/Double Shift. Fresh-context review и explicit
decision до реализации. Нет установки/релиза по одной design score.

Если V1 не даёт практической ценности на актуальных клиентах, оценить закрытие
самого fallback route с явной совместимостью, а не строить новый протокол ради
редкой legacy ветки. Пока production permission для этой задачи не выдан.
