# TD-130 — Привязка inverse cleanup к исполняемому case

Status: DEFERRED_STAGE2_DISCUSSION. Priority: P1. Stage: 2. Size: M/UNKNOWN.
Depends: versioned caller и immutable case/receipt binding;
recovery partial cleanup отдельно TD-137. Standalone offline tool не реализуем.
Owner: existing scripts/proof/ime-client tooling, не runtime learner.
Invariants: C02, C05, C10. Baseline: e7a25705 и immutable browser receipts.

## Доказанный корень

В Gost input inverse была отменена prefixed pair, а helper выбрал unprefixed
pair и вернул NO_OWNED_INVERSE_REJECTIONS. Две отрицательные строки остались,
последующие шесть prefixed случаев потеряли rank authority. После оригинальной
точной prefix cleanup тот же runtime прошёл зависимые20/20. Сохранены исходный
24/30 FAIL, wrong-selector driver и точный cleanup receipt.

## Варианты

- Ещё одна копия helper с literal pair: 2/10, причина воспроизводится.
- Переписать transactional live cleanup и compiler: 3/10, лишний authority/race риск.
- Только offline classifier с переданными from/to: 6/10, audit aid;
  не устраняет обход caller и не закрывает этот task.
- Deferred до versioned caller/case binding: 9/10 сейчас, рекомендуется.
- Подключённый обязательный preflight и checked result: условно8/10 после
  доказательства caller/provenance и обсуждения stage2.

## Выбранная граница и prerequisites

Нынешний consumer — приватный `gost-evidence-carrier/client_probe.py`,212–217,
269–277/284–299: helper pinned по SHA/path, expected pair не передана в finish;
receipt path принимается без проверки status. Native callers инвентаризировать.
Будущий repository consumer выбирается в существующем
`scripts/proof/ime-client/run.py`/`driver.py` только после подтверждения,
что его scenario set исполняет этот inverse. Сейчас connected host-window case
не выбран; новый вспомогательный файл сам по себе не является fix.

Единый immutable case binding должен породить набираемый input, expected
applied/undo surface, field/process receipt, interval, helper hash и selector
expectation. Caller-supplied строки или PID/time не доказывают field ownership:
episode ID не содержит case/field identity. Scope закрепить observed apply/undo
цепочкой и журналом до действия, не принимая чужие строки.
Caller читает cleanup receipt и разрешает dependent cases только после exact
proven cleanup. NO_OWNED/partial/missing/ambiguous — BLOCKED либо явно NOT_TESTED
при доказанно неактивном feedback, никогда PASS из существования path.

До кода назвать exact versioned consumer, binding IO, backward policy frozen
helpers, original timings и positive preservation. Никаких runtime rules,
model fit, новой mutation authority или cleanup retry. Live writes сохраняют
reviewed contract; partial generation не восстановлена без TD-137.
Старые helpers/receipts не переписывать.

Tests разместить в уже исполняемом `tests/test_ime_client_harness.py`, входящем
в SELF_TESTS `scripts/dev-check.py` и `scripts/check-lay-tests.sh`.
Remote route: `python3 scripts/dev-check.py self-test --compact`; nonzero new
test identities/counts сохранить. Если нужен новый module, добавить его в оба
списка до DONE; нового runner не создавать.

## Предыдущий offline вариант — не выбран для первого этапа

Следующий contract оставлен как возможный audit aid. Его реализация без
connected caller закроет только classification, не текущий TD-130.
Новых tool/tests/live cleanup изменений в stage1 нет.

## Минимальный контракт

Добавить `scripts/proof/ime-client/inverse_feedback.py` с importable pure
selection и CLI, работающим с сохранёнными files/receipts. Никаких записей в
журналы, compiler calls, sleeps, process control или service access.
Inputs: journal before/current, BEFORE scope (IME PID, began time, before SHA),
ended time, expected from/to и exact changed target words. Вывод: status и
counts, не raw typed text. CLI документировать в существующем harness README.

Проверить unique unchanged retained suffix при native rotation (500KiB contract),
новую область после before, exact rejection/reverted kind/outcome, PID episode
prefix и interval, одну episode, ровно две уникальные строки и ожидаемые слова.
Wrong-pair rejection в собственной новой области не должен тихо становиться
NO_OWNED: вернуть BLOCKED/неоднозначность. Старые/чужие/positive записи не
выбираются. Повтор идентичной строки, частичная/лишняя pair, смешанная episode,
изменённый old suffix, пустая/неоднозначная rotation — отказ, не guessed cleanup.
Допустимая нулевая выборка отдельно обозначается, не заявляет факта live cleanup.

Это preflight/audit aid, не новый исполнитель. Actual removal остаётся у
оригинального reviewed helper с compiler SHA, тройным CAS, unchanged1.5s bound.
Полный перенос live/window harness из cache — отдельная задача второго этапа;
в этом task не копировать 25 файлов, модели и профили.

## TDD и proof

Remote Python tests в новом `tests/test_inverse_feedback_binding.py` на временных
fixture bytes: correct prefixed/unprefixed pair, wrong selector, zero legitimate
rows, one/three/duplicate/mixed-episode rows, interval/PID mismatch, positive and
other-owner preservation, valid rotation и ambiguous/rewrite rejection.
Доказать на RED минимальный wrong-selector fixture до classifier implementation;
после GREEN показать immutable input hashes до/после unchanged. Fixtures искусственные;
настоящие journal bytes не попадут в Git. Тест должен проверять public selection
и CLI exit/status, не строки source или копию реализации.

## Последствия и rollback

Runtime candidates/ranking, model/packages, caches и feedback writer неизменны.
Новый cost — одно offline O(journal bytes) чтение bounded journal; не per-key path.
Главный риск — ошибочная классификация как authority удаления: API/doc прямо
не выдаёт такой authority и не содержит delete/write. Concurrent current journal
не используется как CAS proof; inputs frozen receipts. Revert tool/tests/doc
commit восстанавливает исходную tooling surface, без runtime rollback.

## Приёмка будущей выбранной реализации

- [ ] Semantic RED/GREEN и все fixed offline negative cases remote PASS.
- [ ] Ни один input не меняется; live cleanup/deadlines/compiler неизменны.
- [ ] Новый test включён в подходящий Python verification route, не orphan suite.
- [ ] Independent review >=8/10, максимум два прохода; receipts и caveats записаны.
- [ ] Versioned caller использует один binding и проверяет actual cleanup status.
- [ ] Private historical wrong-selector case проверен отдельно; input bytes сохранены.
- [ ] Commit/push. Offline/source PASS не заменяет physical/lifecycle/cleanup proof.
