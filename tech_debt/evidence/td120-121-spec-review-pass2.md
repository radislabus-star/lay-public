# TD-120 / TD-121: проверка document repair, pass 2

Дата: 2026-09-05. Baseline: `cc1e2207519801ca0f9b7c6963897b55953a7751`.
Последний запланированный проход: только R1–R3 из pass 1, обновлённые TD-120,
AUDIT и Active Layout Queue README, согласованность с неизменным TD-121.
Новых source/runtime исследований, проб, Cargo и production-правок нет.
Единственная запись reviewer в этом проходе — этот файл.
`runtime_authority_changed=false`.

## Findings и разрешение предыдущих замечаний

**Оставшиеся High: 0. Medium: 0 в проверенном scope. R1–R3 устранены в
спецификации.** Это закрытие неоднозначностей документа, не production gates.

| Finding | Проверенный repair | Verdict |
|---|---|---|
| R1 — зависимость от TD-121 | TD-120:70–89 явно блокирует всю unified migration на G120-1/2/3. S06/S07 на :208–209 разделяют characterization и обязательные gated cases; :224–225 сохраняет их denominator. README:55–64 разрешает независимо лишь core failing tests/characterization после phase-2 admission | RESOLVED |
| R2 — legacy V1 | TD-120:74, :153–160 блокирует всю миграцию до доказанной совместимости no-arg RPC; запрет phase guessing сохранён. S12 на :214 включает before-only failure/abort, duplicate, delayed after чужого word/owner, text-replace и trailing boundary варианты | RESOLVED |
| R3 — stale atomic commit | TD-120:75, :104–110 и :189 различает preview isolation и settlement. S16 на :218 покрывает prepare → live arm/consume/revoke/owner transfer → old commit, отсутствие восстановления старого/потери нового guard, повторного output и feedback. Сериализация допустима лишь после доказательства для всех producers | RESOLVED |

AUDIT:94–111 и :166–170 согласован с repaired scope: atomic interleaving не
объявлен воспроизведённым production-дефектом; canonical identity — зависимость,
а не измеренный результат. TD-121 сохраняет ANALYSIS_REQUIRED, открытые
reproduction/context gates и private 0/6. Циклического требования реализовать
оба изменения одновременно нет: TD-120 зависит от принятого identity-контракта
TD-121, а TD-121 требует согласовать suppression scope, не завершить TD-120.

## Оценка документации и readiness

**Overall: 8/10. TD-120: 8/10. TD-121: 8/10, оценка pass 1 сохранена.**
Document review threshold ≥8/10, High/Medium=0 выполнен в пределах двух
оговорённых проходов. Новый полный аудит или proof не подразумеваются.

| Задача | Readiness |
|---|---|
| TD-120 | ANALYSIS_REQUIRED. Документация принята как phase-1 результат; G120-1 V1, G120-2 atomic settlement и G120-3 canonical identity остаются OPEN. Unified production migration не допущена |
| TD-121 | ANALYSIS_REQUIRED. Двухобъектное воспроизведение и безопасный canonical receipt на работающем IBus не доказаны; private probe выполнил 0/6 |

Следующий допустимый результат после отдельного phase-2 admission — core failing
tests/characterization и доказательства открытых gates. Узкий lifecycle вариант
остаётся альтернативой, которую нужно явно выбрать; этот review его не выбирает.
Ни исправление пользовательского поведения, ни release/install/DONE этим
документом не подтверждаются. Дальнейший repair R1–R3 не требуется.
