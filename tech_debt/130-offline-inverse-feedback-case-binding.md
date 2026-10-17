# TD-130 — Привязка inverse feedback к исполняемому case

Status: DONE_CONNECTED_PRIVATE_PROOF. Priority: P1. Stage: 2.
Owner: existing scripts/proof/ime-client tooling; runtime learner не меняется.
Invariants: C02, C05, C08, C10. Source baseline: 55fd32bf.
Shared cleanup/recovery остаётся отдельным TD-137; native ordering — TD-133.

## Доказанный корень

В историческом Gost input consumer отменена prefixed pair, но helper выбрал
unprefixed pair и вернул NO_OWNED_INVERSE_REJECTIONS. Caller принял существующий
receipt path без проверки status. Две отрицательные строки остались; шесть
следующих prefixed cases потеряли rank authority. Исходный24/30 FAIL и отдельный
точный cleanup + dependent20/20 сохраняются; это разные denominators.
Исторические helpers/receipts не переписывать и не запускать для новых cases.

Episode PID/time сами по себе не доказывают поле: исходная ошибка находится
между исполняемым case и selector, до cleanup. Снятие SafetyGate, смена моделей
или транспортов не исправляют эту связь. Двухфайловая shared cleanup также
имеет самостоятельный partial-write/CAS defect, который нельзя закрыть одним
совпадением case или изоляцией нового proof.

## Варианты и выбранная граница

- Новая копия literal-pair helper:2/10, повторяет исходную ошибку.
- Только offline classifier/CLI с caller-supplied from/to:6/10, audit aid без
  обязательного consumer; не выбран. Старый проект этого CLI/test module
  снят с текущей приёмки, а не реализован параллельно.
- Параметризованный existing shared cleanup:5/10, сохраняет риск TD-137.
- Immutable case и обязательная read-only проверка в existing private IBus
  consumer:9/10, выбран. Новый runner/compiler/live cleanup не нужен.

Реализация: existing `scripts/proof/ime-client/run.py` → V3 `driver.py`, opt-in
`inverse-first-word` и `inverse-prefixed`. Каждый запуск — один boundary apply
и один ManualToggleV3 inverse в fresh private D-Bus/IBus sandbox, с четырьмя
existing private usage paths. Следующих dependent cases в этом sandbox нет.
Это repository proof consumer; исторический host helper не стал безопасным
для повторного исполнения. GTK/Qt/browser/physical acceptance отдельно.

## Минимальный контракт

Один frozen case задаёт typed/applied/undo и derived changed target words.
Observed actual text-key input и applied surface должны совпасть с case.
Binding включает настоящий IBus context, candidate PID/starttick/SHA и journal
before/interval. Copied selector SHA проверяется до case. Readiness Shift,
release и other-context records не входят в набранный текст; malformed text
press вызывает отказ.

Forward/inverse — ровно один ordered delete/commit frame. Requested offset,
count и deleted text выводятся из case suffix, а не из clamped visible result.
Context, prefix, commit, итоговая поверхность и cursor должны совпасть точно.
Pending GLib callbacks обрабатываются в existing1.5s feedback bound; полный
inverse и результат перепроверяются через focus-out до PASS. Final context и
engine проверяются один раз existing snapshot route. Receipt сохраняет оба
Unix-nanosecond endpoint обратной связи.

Selector `feedback_case.py` не пишет файлы и не предоставляет cleanup authority.
Проверяются unchanged journal prefix либо unique byte-identical retained suffix
при native500KiB rotation; только новые rejected/reverted строки собственной
PID episode и interval; одна episode, ровно derived unique changed targets.
Wrong own pair, extra/duplicate/mixed episode, rewritten/ambiguous/partial bytes
— отказ. Missing/partial feedback остаётся PENDING; connected caller обязан
завершиться nonzero по существующей границе, никогда PASS из receipt path.
Positive и other-owner bytes остаются неизменными. Никакого removal/recovery.

## Последствия и backward policy

V3 — explicit successor с exact driver/module hashes. V1/V2 commit/blob/SHA
сохраняются; default five-case run_cases function неизменна. Config schema V1
сохраняется; proof/run metadata V3 не объявляют baseline parity.

Lattice/rank/weights/verifier/models/transports/owners/hot-key deadlines не
меняются. Journal observation после gesture, не per-key. Все проверки remote
через existing resource/Cargo guards. Read-only worker assets имеют собственные
pins; модельная parity с installed desktop этим не установлена. Rollback —
revert tooling commit; shared learning bytes не восстанавливать поверх чужих.

## Измерения и исправления

- Первое расширение правильно отклонено V2 identity guard: это не semantic RED.
  V3 опубликован как отдельный контракт, guard не снят.
- Controlled wrong-pair equality violation: один named semantic test FAIL как
  ожидалось; это controlled RED, а не воспроизведение runtime failure.
- Real private pilot FAIL до inverse: collector захватил fixture Shift без
  character. Исправлен text-key selector и добавлен semantic regression;
  исходный FAIL сохранён, delivery/runtime unchanged.
- Collector revision: remote116 selected/115 passed/1 skipped/0 failed,
  8 new tests PASS; оба private IBus scenarios PASS с двумя feedback rows.
- Review pass1:7.5/10, GROUPED_REPAIR_REQUIRED. Найдены oversized clamping
  false accept и delayed callback gap. Root выполнил одну grouped repair:
  strict case geometry, complete effects through closure, interval recording.
  Final remote122 selected/121 passed/1 skipped/0 failed; all14 new tests PASS.
  Три controlled guard violations дают ожидаемый semantic RED; оба final
  real private IBus cases PASS, exact effects и feedback2/2. Старые PASS
  не перепривязываются.
- Final review pass2:9/10, ACCEPT connected private proof; оба finding закрыты,
  дополнительных implementation правок не требуется.
- Canonical graph/check PASS,8 stable fetched exports; compiled generated
  receipt1/1 PASS. Exact receipts и hashes — в final evidence packet.

Owning document: [tech-debt maintenance](../docs/architecture/tech-debt-maintenance-2026-10-09.md).
Decision: [private inverse proof binding](../docs/architecture/decisions/2026-10-09-isolated-inverse-proof-binding.json).
Final packet: [connected inverse evidence](evidence/2026-10-09-td130-connected-inverse.json).
Private receipts: `/home/ubu/.cache/lay/development/td130-*` и `run-os6fuz1y`.
Raw journal/user text logs не коммитить.

## Приёмка выбранного scope

- [x] Existing remote self-test route executes all new semantic identities;
  nonzero selection, controlled RED и final GREEN bound к source/dependencies.
- [x] Oversized clamped delete и callbacks during feedback/focus-out дают отказ
  через actual Client/driver consumer; missing feedback nonzero bounded.
- [x] Final real private IBus smoke: first-word и prefixed, один apply/inverse,
  точные effects/caret и complete episode, private daemon/candidate reaped.
- [x] Final independent review >=8/10, всего максимум2passes; owning evidence,
  canonical remote graph/check и generated receipt обновлены.
- [x] Scope DONE только connected private proof; historical host reuse,
  shared cleanup/recovery и native/physical acceptance остаются отдельно.
Publication receipt с exact commit/refs после commit/push:
`/home/ubu/.cache/lay/development/td130-publication-20261009.json`.
Source/private PASS не объявлять runtime installation, physical acceptance,
quality promotion или закрытием133/137.
