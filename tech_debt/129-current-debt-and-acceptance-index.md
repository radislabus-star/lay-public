# TD-129 — Одна актуальная очередь и карта доказательств

Status: DONE_METADATA_SCOPE. Priority: P0. Stage: 1. Size: S.
Baseline: e7a25705. Owner: tech_debt/README.md, tech_debt/CONTINUE.md и root CONTINUE.md.
Audit: [2026-10-09](AUDIT_2026-10-09.md). Invariants: C01, C10.

## Корень и результат

tech_debt/README.md называет R5/C20 текущим; tech_debt/CONTINUE.md уже
начинает с R12/1.0.72, но ниже оставляет latest R7. Root CONTINUE.md
начинает с C20/1.0.72. TD-121 содержит ещё более поздние repairs. На HEAD
принята 1.0.81 для отдельного pair scope.
Следующий агент может выбрать неверную ветку, бинарник и rollback.
Нужно одно верхнее current entry, priority table и ссылки на owning evidence;
исторические статусы, FAIL и scoped DONE сохраняются как история.

## Варианты

- Добавить ещё один handoff, оставив старые current headings: 3/10.
- Переписать краткий README и добавить явный current override в CONTINUE: 9/10,
  рекомендуется; owning experimental history остаётся на исходных местах.
- Создать новый tracker/database и массово переименовать tasks: 2/10.

## Изменения

1. В верхней части README назвать authoritative checkout, branch, принятую
   версию/hash и точный scope. Указать source, installed и physical отдельно.
2. Дать отсортированные stage-1/stage-2 задачи с dependencies/status.
3. Перечислить все прежние карточки 113,120–128: сохранить DONE scope;
   незакрытые 121/122/123/127/128 оставить открытыми для current proof.
   Для каждой открытой карточки дать original scenario/invariant, последний
   scoped receipt, current2bd status UNKNOWN/доказанная часть и следующий
   разрешённый investigation. TD-123 указывает один actual owner:
   `/home/ubu/projects/lay-syntax-agreement-20260929/docs/architecture/ru-agreement-roadmap-2026-09-30.md`;
   current stage читать там; не заводить второй fit/roadmap здесь.
4. В обоих CONTINUE.md добавить верхний current checkpoint со ссылкой на один
   tech_debt/README.md и маркировать последующий
   старый текст историей. Не редактировать 5000 строк старого TD-121 ради косметики.
5. Указать next task, review receipts, test route, rollback boundary и запрет
   приписывать stage-2 implementation её описанию.
   Первый этап129→131.130/132/133–137 и прежние open scopes этап2 остаются
   открытыми; TD-136 информирует выбор TD-134, TD-132 не обязательная зависимость.

## Подводные камни и проверки

Новая пользовательская приёмка пары не закрывает all-window acceptance или
TD-123 quality. Не объявлять historical failure исправленным без соответствующего
proof. Local cache receipt пути могут быть приватными и отсутствовать у другого
разработчика: описание измеренных результатов должно быть самодостаточным,
сырые журналы не коммитить. Этот task — metadata; новые behavior tests не нужны.
Проверить согласованность ссылок и explicit status каждой карточки, независимо
дать score/Findings. Architecture canon/guards/runtime не меняются.

## Последствия и откат

Candidate/rank/latency/CPU/RSS/cache/package/learning/concurrency/consumers:
изменений нет. Единственная source of truth остаётся существующая очередь и
owning documents; новый audit индексирует её, не создаёт второго runtime owner.
Откат — revert task commit, сохранив прошлые receipts. Accepted IME не трогать.

## DONE

- [x] Current index и история согласованы, все прежние tasks учтены.
- [x] Незакрытая область и stage 2 явно выделены.
- [x] Fresh-context review >=8/10, findings устранены за максимум два прохода.
- [x] Task outcome и review receipt записаны; публикация — следующий шаг этого checkpoint, с отдельным exact Git receipt.

## Outcome — 2026-10-09

Baseline e7a25705. Переписан tech_debt/README.md; current override добавлен в
оба CONTINUE.md, их прежний текст сохранён. Все10 прежних numbered cards
113,120–128 имеют scope/current map; старые121/122/123/127/128 остаются открытыми.
Stage1:129→131; Stage2 cards130/132–137 deferred. Audit и план прошли два
независимых plan passes, final9/10. Implementation review:
[evidence/2026-10-09-td129-review-pass1.md](evidence/2026-10-09-td129-review-pass1.md),
ACCEPT9/10 с первого прохода, correctness findings отсутствуют.

Production code/guards/models/runtime authority NOT_CHANGED. Installed2bd hash
прочитан и совпал; функциональная/physical приёмка не повторялась. Новые behavior
checks для metadata не требовались. Откат — один source-document revert.
Commit/push/remote refs/clean worktree после этого DONE фиксируются в
`/home/ubu/.cache/lay/development/td129-publication-20261009.json`;
этот receipt создаётся после публикации, а не объявляет её заранее.
