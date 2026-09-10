# TD-120: Ограничить обычный запрет автозамены жизнью своего слова

Status: `DONE` — scoped source/runtime acceptance; release/install отдельно
Priority: `P1`
Stage: 1 — scoped ordinary-word lifecycle, не общая миграция transport
Size: M
Base: `cc1e2207519801ca0f9b7c6963897b55953a7751`
Admission: 2026-09-05, «дальше без остановок пожалуйста».
Analysis: Astra/XHigh. Implementation: GPT-5.6 Sol/High.
Цель выпуска: 1.0.66 и push; live IBus и установленные бинарники пока неизменны.

## Проблема и доказанный результат baseline

Обычный manual edit защищает своё слово от автоматического обратного изменения.
После полного удаления слова этот guard остаётся и подавляет совершенно новый
токен. Production log: `вот→djn`, 54 Backspace (45 при пустом хвосте), затем
`gjxbnfq`, `manual_toggle_suppressed`. Статически установленный producer:
`state.rs::replace_committed_tail`; отсутствие retirement:
`composition_edit.rs::backspace_committed_tail_only`; unscoped OR consumption:
`committed_tail.rs::take_manual_toggle_autocorrect_suppression`.

[Private installed-byte baseline](evidence/td120-121-installed-baseline-phase2.md)
также выполнила actual ManualToggleV3 → удаление → новое слово → старый запрет.
Всего6 диагностических cases, не6 исправлений. Nanda/auto-replace отключены
только в private probe: победа `почитай` по полному frame ещё не проверена.

## Уточнение дизайна до production-кода

Первоначальная [unified specification](evidence/td120-unified-spec-superseded.md)
и её review8/10 сохранены как исторический дизайн. Её G120-1/2/3 **не объявлены
PASS**. Анализ показал, что V1 before/after RPC не передаёт request/token/phase,
а generic delegation не удовлетворяет existing exact V2 preconditions.
Требование универсальной точности при неизменном V1 информационно невыполнимо.

Выбран постоянный узкий дизайн **8/10**:
[полный admission analysis](evidence/td120-suppression-admission-analysis.md).
Главный агент прочитал producer inventory, варианты, последствия и bounded
atomic settlement до допуска production edits.

| Вариант | Оценка | Решение |
|---|---:|---|
| Сбрасывать любой guard на Backspace/empty tail | 2/10 | Ломает продолжаемое слово и synthetic delete/replay |
| Типизированный ordinary lifecycle + unchanged transport protocols + scoped atomic reconciliation | **8/10** | Выбран: исправляет подтверждённый механизм, без нового процесса/API/очереди |
| Единый shared owner с универсальным lifetime и неизменным V1 | 4/10 сейчас | Не решает отсутствующую identity простым переименованием полей |
| Подставить V2 вместо двух V1 calls | 5/10 | Не совместимая обёртка: отсутствуют required exact capture/handoff/isolation |
| Checked begin/finish/cancel replay request | 7/10, Stage2 | Нужны отдельные event-settlement и backward-compatibility доказательства |

Исходные обязательства распределены явно: ordinary lifecycle и его scoped
atomic compatibility — здесь; canonical context/handoff — TD-121; универсальная
legacy transport identity — [TD-122](122-bind-legacy-replay-suppression-request.md).
Это не «все suppression ошибки закрыты». Временные flags/ветки-исключения
не добавляются, transport protocol не ослабляется.

## Постоянный внутренний контракт

- Вместо boolean+optional-exact — `CurrentWord`, `LegacyReplayV1`,
  `ExactReplay` или отсутствие guard. Старые booleans не остаются fallback
  authority. Существующая local/shared storage topology сохраняется.
- Обычный local guard — mirror той же incarnation. Решение расходования
  CurrentWord проверяется в shared state; старый local mirror его не воскрешает.
  Word generation не равен tail_epoch, который меняется при каждом publish.
- Arm только внутри действительно успешного typed edit/accept с обновлённым
  хвостом. Wrapper `Ok(())` иногда означает rejected/no-op; это не success proof.
  Duplicate/failed output не создаёт новый guard и не удаляет прежний валидный.
- Полное удаление текущего открытого токена, реальная граница, Enter и потеря
  text lease retire matching ordinary guard. Повтор идентичной строки — новое
  слово; частичный Backspace/допечатывание непустого слова сохраняет incarnation.
- Не проверять только `last_tail_token_text().is_empty()`: helper trim-ит
  whitespace, поэтому `old x→old ` снова видит `old`. Учитывать именно открытый
  токен, активную composition/cursor и существующие правила layout punctuation.
- `;`, `[`, `]`, apostrophe и другие физические RU-letter keys не становятся
  безусловными word boundaries. Никаких literal word/suffix runtime условий.
- Replacement/undo/Tab с trailing boundary не защищает следующее слово.
  Feedback текущего слова фиксируется до retirement; нет чужой rejection label.
- Existing admitted handoff переносит ту же incarnation и rebind-ит owner.
  Это baseline parity, не новая canonical identity. Старый engine не расходует
  и не очищает guard нового owner. Сильный context contract принадлежит TD-121.
- Ordinary retirement никогда не снимает transport scope лишь из-за empty
  intermediate tail. Exact V2 сохраняет source/path/layout/epoch/expiry/revoke.
  Legacy V1 сохраняется как явно названная совместимость, не universal proof.
- Active manual/candidate commit, committed manual/candidate/undo, typed
  ReplaceTail, defer-to-daemon и bridge V1/V2 учтены по отдельности в inventory.

## Scoped atomic settlement — обязательная часть, не отложенная регрессия

Сохранить публичные proposal/receipt formats, физический single-edit и no-retry.
Добавить reconciliation только текущего suppression/handoff/owner контракта:

1. Из одного snapshot в pending сохранить **base** suppression_revision,
   active owner/path, existing tail epoch/focus/buffer stamp и local owner lease.
   Speculative updates меняют только клонированный SharedState.
2. Arm/consume/revoke/clear всех scopes увеличивают revision. Сравнивать live с
   base, не с final speculative revision: обе ветки могут независимо прибавить1.
3. Проверка stamp и shared replacement — один lock/critical section. Если
   owner/tail неизменны, но live suppression новее, сохранить live block,
   включая None после revoke, transport handoff fields, revision и local mirror.
4. При owner/tail drift не записывать старый clone поверх нового состояния.
   Сохранить новый owner/tail/guard, retire только stale local authority.
   Новый key получает существующий native-unhandled/refusal outcome.
5. Предыдущий submitted frame не объявлять откаченным и не повторять.
   Deferred layout/feedback старой speculative ветки не применять; trace outcome
   censored/uncertain, без fabricated accepted/reverted learning.
6. Sensitive/quarantine/shared-only cancellation и разные engine locks входят
   в tests. No-conflict commit, abort и duplicate receipt не меняют семантику.
   Generic merge всех полей SharedState не входит в эту задачу.

## Последствия, бюджет и границы

Полная таблица — в admission analysis; обязательные ограничения:

| Измерение | Ожидание / риск / проверка |
|---|---|
| Lattice/ranking/authority | Генерация и ranking неизменны; снимается чужой veto, не выдаётся Apply. Полный frame проверяется отдельно |
| Latency/CPU/RSS | Малый tagged scope/числовые identities, O(1) обычный lifecycle; без RPC/sleep/new worker/Space deadline increase. Atomic stamp использует уже bounded tail, учитывать дополнительную копию |
| Cache/package reload | Нет package pointer в guard; existing frame invalidation сохраняется, generation не tail_epoch |
| Learning | Успех/undo/boundary feedback не дублируются; conflict censored, не чужая rejection |
| Concurrency | Mirror incarnation, scope revision, atomic base snapshot и одно critical section; no stale resurrection |
| Failure/rollback | Failed edit не создаёт guard; exact failure/revoke unchanged. Source revert без очистки словарей |
| Consumers | Managed/active/terminal, Tab/undo, legacy/exact replay, atomic — раздельные cases |
| Maintenance | Старые ambiguous flags удаляются, typed scopes постоянны. V1 retirement отдельно TD-122, без нового god-manager |

Budget judgement: +5 сохранение пользовательского intent, +5 lifecycle safety,
+5 coherence scopes, +3 testability, −5 stateful-change risk, −2 дополнительные
metadata поля = +11 при прохождении tests, без hard veto. Это проектная оценка,
не измеренная качество/скорость. Чистый move-only refactor не заявляется.

## Фиксированная TDD-матрица и критерии приёмки

Полная expansion O1–O6/V1-1..3/E1/A1..3/C1 записана в admission analysis.
Фиксировать exact test manifest до runtime patch; не прятать RED через ignore.

- Все ordinary producers: successful open word protected once; failed/duplicate
  output, trailing boundary, same-text retype, left history, partial erase,
  Enter и layout punctuation. Surfaces/output/verdict/guard/feedback, не grep.
- Existing handoff parity: old owner не clear/consume нового incarnation.
  Canonical positive/negative cases TD-121 остаются отдельным denominator.
- Exact V2 synthetic empty/replay, bad source/path/epoch/layout, expiry, revoke.
  V1 caller/reselection/before-only/duplicate/delayed-after characterization —
  отдельный ledger с KNOWN_RESIDUAL, не ordinary PASS.
- Atomic A1 нормальный clone/abort/commit/duplicate; A2 newer arm/consume/revoke
  и совпавшие final numeric revisions; A3 sensitive/cancel/foreign owner/ABA.
  Нулевой повтор output/deferred layout/feedback.
- Полный frame `gjxbnfq→почитай` после исходного механизма, три safety profiles,
  clean/protected controls. Guard removal не засчитывается как conversion PASS.
- Existing `physical_double_shift_owner_`, suppression/exact/composition/atomic,
  changed gate; исходный fixed proof denominator не уменьшается.

Remote: `e@192.168.3.94`, source checkout
`/home/e/projects/lay-td120-121-SUdh2I`; source parity до каждого запуска.
Cargo только через `scripts/lay-resource-guard.sh` с dedicated-20cpu и
`scripts/cargo-guard.sh test --bin lay-ibus-engine <exact-filter>`.
Кеш target явно scoped и ≤12GiB. Локально Cargo не запускать.

После runtime patch: focused tests → changed gate → architecture refresh →
fresh-context Astra/XHigh review ≥8/10, High/Medium0, максимум1–2 repairs.
DONE только для описанного scoped результата с evidence, commit/push;
TD-121/TD-122 автоматически не закрываются. Release/install отдельно.

## Промежуточные результаты (история, не финальный verdict)

- Baseline ordinary leak динамически подтверждён.
- [Первый remote RED baseline](evidence/td120-first-red-baseline.md):9 tests,
  3 PASS/6 FAIL;5 failures подтвердили suppression leak,1 — исправляемый
  output-helper теста. Это не расширенный acceptance denominator.
- Первый code review: [pass 1](evidence/td120-code-review-pass1.md),
  6/10, High 1 / Medium 2; результат `REPAIR_REQUIRED`.
- [Bounded repair pass 2](evidence/td120-repair-pass2.md): same-lock atomic
  owner/capture исправлен; завершаются lexical-window invariant и реальные
  bridge/replay/feedback proofs. Source ещё не зафиксирован для финальной проверки.
- Промежуточный remote IME run: 319/319 PASS, 14.46 s,
  `/home/e/projects/lay-td120-repair-v3-ime.log`. Он предшествует последнему
  repair delta и не доказывает окончательные bytes.
- Промежуточный raw daemon run: 234 PASS / 2 FAIL, 142.11 s,
  `/home/e/projects/lay-td120-repair-v3-daemon.log`. Устаревшая source-string
  assertion удалена; второй failure зависит от окружения и прошёл тем же
  executable в canonical hermetic runner (1/1,
  `/home/e/projects/td120-neighbor-hermetic-2lbs0i_u/test.log`). Исходный RED
  не скрыт; финальный gate должен пройти штатные hermetic lanes.
- На этом промежуточном checkpoint финальный pass 2 и changed gate были
  незавершены. Он superseded следующим результатом, без удаления исходных RED.

## Финальная приёмка source — 2026-09-06

[Final acceptance](evidence/td120-final-acceptance.md): canonical changed gate
**2554/2554 PASS** (2518 correctness + 36 package), known/infrastructure
failures 0; IME 319/319, daemon 239/239, lib 1757/1757. Эти target counts —
подмножества 2554, не дополнительные проверки. Architecture wrapper PASS.
Performance 11 и ignored 15 в это число не входят.

Independent pass 2: **8/10, High 0 / Medium 0**; последние два source-contract
теста и исполненные receipts приняты в bounded closing supplement, не новом
полном проходе. SHA-256 local/remote source совпали после восстановления связи.
Runtime после GREEN не меняется до task checkpoint. Отдельный task commit
содержит эту отметку DONE; его identity и push проверяются по Git, не self-hash
документа. TD-121, TD-122, release/install и физический ввод этим результатом
не закрыты; установленная версия — 1.0.65.
