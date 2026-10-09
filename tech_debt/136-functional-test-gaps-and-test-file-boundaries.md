# TD-136 — Semantic gaps и границы больших test files

Status: DEFERRED_STAGE2_DISCUSSION. Priority: P2. Depends: causal mechanism map.
Owner: existing manifest/discovery and production reducer/adapter tests.

## Корень и evidence distinction

3044 required PASS не означает line/branch/client coverage. Residuals13039 и
terminal_delivery4159 строк усложняют поиск scenario families, но splitting
не улучшает функциональное покрытие само по себе. Source .contains/hash guards
проверяют ownership/lineage и дополняют semantic tests; их нельзя выкидывать
как «рудиментарные», чтобы пройти rename/refactor.

## Варианты

- Порезать файлы/переименовать tests по size: 3/10, identity churn без качества.
- Behavior→owner→entrypoint→positive/negative/effect→physical map, затем только
  найденные gaps: 9/10, рекомендуется.
- Придумать coverage процент по test names/file count: 1/10.

## Minimal protocol

Перечислить Space, first-after-Enter, clean/invalid pair, Tab, held Shift,
eight DoubleShift intermediates/icon/decoder, undo, focus/Reset, cold/package
reload, stale reply, partial transport и feedback cleanup. Для каждой ячейки
найти actual production entrypoint и asserted effects; UNKNOWN оставить UNKNOWN.
Имена test или API dispatch не доказывают effect. Coverage instrumenting только
если оно отвечает конкретному вопросу; бюджет/refactor не тратить на dashboard.

TDD gaps по shared first-failure mechanism, not per literal example. Managed
event order без sleeps. RED на baseline или narrow controlled violation,
GREEN whole affected fixed proof. Если нужна file extraction — один связный
scenario family с сохранением identities/isolations; test-only helper не должен
имитировать вторую runtime state machine или владеть authority.

## Pitfalls/acceptance

Process isolation не заменяется faster shared process; test names/lanes/fixture
hashes обновляются только через declared discovery contract. Historical source
reviews не переписываются; new binding/successor видимый и независимый. Не
снимать failed class или игнорировать flaky output. Source and physical verdicts
раздельны, no generic PASS. Package/delta/caches/ranking untouched до отдельной
системной задачи. Per task review<=2, score>=8 плюс objectively passed scope.
Stage2 map/spec не объявляется implementation; новую матрицу обсуждаем отдельно.
