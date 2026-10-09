# TD-134 — Следующая граница god-components без нового owner

Status: DEFERRED_STAGE2_DISCUSSION. Priority: P2. Depends: bounded TD-136 map.
TD-132 не обязательная technical dependency; выбранная boundary должна иметь пользу.
Owner: WindowInteraction/ContextAdmissionReducer/TransitionDecisionCore.

## Inventory и корень

observation.rs4325, admission adapter3416, tail_memory2895, admission reducer2464
строк — сигналы для dependency review, не квота «обязательно разделить».
Нужен inventory: факты поля, inert witnesses, lifecycle, admission fencing,
effect dispatch, settlement. State owner один, группировка файла не owner.
У L2 packaged_runtime4624/material_frame4328/proof5724 другие границы:
research/proof не переносить в input controller.

## Варианты

- Массово разнести файлы по <=500 строк: 2/10, cosmetic ownership drift.
- Один connected existing-owner slice после call/side-effect map: 8/10,
  рекомендуется при измеренной maintenance пользе.
- Новые контроллеры с duplicated token/generation: 1/10.

## До кода и pitfalls

Для каждого кандидата выписать callers, state reads/writes, RPC/await точки,
borrow/lifetime и effects, protected paths и substring/hash guards. Сравнить
чистую leaf extraction и сохранение stateful method рядом с owner. Не добавлять
generic traits, event bus, registry или shared cache ради сокращения строк.
Разделить move-only и последующий behavior fix на разные tasks/commits.
Новые child files сохраняют protection: exact-path canon guard не переносит
защиту автоматически. До extraction добавить нужный path под explicit decision
и negative guard test; проверки не ослаблять и учесть guard successor в rollback.
Перенос не изменяет pending callback order, token equality, cancellation,
allocation или package lifecycle. Больше exports требует конкретного caller.

## TDD/acceptance

Characterize actual entrypoints по C03/C05/C08: old callback после new owner,
partial/mismatched effect, duplicate completion, focus/reset transitions и
no second mutation. Keep fixed manifest identity и source-bound contract
succession; не перетирать старые evidence hashes. Fresh source tests/graph
и independent score>=8 после каждого единственного slice; scope physical
NOT_TESTED до отдельного exact-byte proof. Rollback — move-only revert.
Пока не выбран обоснованный slice, task открыт и реализация не начинается.
