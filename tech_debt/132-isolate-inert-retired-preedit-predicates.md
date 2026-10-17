# TD-132 — Чистая граница inert retired-preedit geometry

Status: DEFERRED_STAGE2_DISCUSSION. Priority: P2. Stage: 2. Size: S/M.
Depends: bounded TD-136 map и доказанная maintenance польза выбранного slice.
Owner: existing WindowInteraction; C03, C06, C08, C10.

## Корень и варианты

В observation.rs (~4325 строк) есть разные обязанности, но выбранные leaf functions
retired_preedit_right_edge_is_safe и snapshot_matches_retired_preedit связаны
друг с другом, стоят соседним pure block и имеют одного caller. Existing
residuals tests4973–5041/5219–5265 уже проверяют bridge, inert witness и
no Delete/Commit до нового exact receipt. Defect/помеха проверке именно в
этом block не доказаны; обязательный перенос сейчас не рекомендуется.

- Только разделители/comment headers: 5/10, dependencies остаются смешанными.
- Сохранить existing pure block и adapter coverage: 8/10 сейчас.
- Одна leaf file existing observation module: 6/10 сейчас, условно8/10
  после concrete benefit/consumer map и сохранения protection.
- Разнести всю state machine по новым controllers/traits/crates: 2/10 сейчас.

## Условия допуска и возможная минимальная граница

Сначала назвать конкретную maintenance проблему и missing semantic gap.
Reuse existing tests; новый полный matrix без gaps не создавать.
Ни spec, ни перенос задачи в stage2 не закрывают task.

`window_interaction/observation/retired_preedit.rs` содержит только эти две
pure functions. PendingContextResetRereceipt, PublishedPreeditWitness,
SurroundingTextSnapshot и lifecycle/RPC остаются у текущего owner. Никакой
новой публичной authority, EngineOutput, tokens/caches/timers. Private child
может читать parent types; наружу публикуется только нужный predicate.
Сначала функции перенести побайтно по body (visibility/import отдельно),
поведение и имена existing integration tests не менять.

## Meaningful characterization

Тестировать production predicate: exact old publication с обоими известными
cursor positions; observed append/old visible suffix; выборка с чужим prefix,
selection, out-of-bounds cursor, неравной поверхностью, отсутствующим witness,
небезопасной правой буквой; допустимые boundary/одиночный U+200B и отказ
U+200B+unbounded text. Unicode scalar cursor, не bytes/UTF-16.
Важно: true сохраняет inert witness, не подтверждает committed token. На
production adapter existing retired-preedit/Reset tests должны по-прежнему
не разрешать delete/apply до свежего exact receipt. Не удалять эти тесты,
заменяя их unit-only checks. Characterization без старого FAIL допускается;
controlled wrong-predicate mutation может доказать negative sensitivity.

## Consequences and proof

Candidate/rank/verifier/package/cache/learning unchanged. Move-only сохраняет
hot-path O(snapshot chars), allocations и call order. Главный риск: повысить
visibility до второго authority consumer или принять display как edit target;
не вводить таких exports. До кода explicit decision для protected observation
path. Новый leaf сейчас не покрыт exact-path protected(): до переноса добавить
его в canon protection с новым explicit decision и negative guard test;
проверки не ослаблять. После — owning entry, remote IME+integration tests и graph
refresh/canon. Source-bound guards должны видеть ту же production ownership;
не изменять их ради сокрытия переноса. Смена test identities требует existing
discovery/manifest contract. Все модели и installed IME остаются прежними.
Rollback — вернуть leaf тела/import, сохранить полезные characterization tests
и оформить protection/evidence successor, не удаляя semantic proof автоматически.

## DONE

- [ ] Один связный pure slice выделен; existing state owners и behavior preserved.
- [ ] Characterization и adapter negative effects remote PASS, graph/canon PASS.
- [ ] Fresh-context review >=8/10, максимум два прохода; source receipt/untested scope.
- [ ] Task source-only DONE, commit/push; более крупный split остаётся TD-134.
