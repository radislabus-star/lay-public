# История и восстановление проекта

Очистка выполняется только в ветке `codex/cleanup-20260908`.
Полное дерево до очистки сохранено в Git:

```text
cb40ef29f6c78c97757dd0059c0ba798cb1f0789
```

Исторические receipts, завершённые задачи и V10/V11 scripts находятся в этом
snapshot по прежним относительным путям. Ссылки и команды внутри старых
архитектурных журналов относятся к состоянию на дату записи.

Прочитать любой исторический файл, например прежнюю очередь:

```sh
git show cb40ef29f6c78c97757dd0059c0ba798cb1f0789:tech_debt/README.md
```

Восстановить всё исходное дерево в новую папку для старого эксперимента:

```sh
git worktree add --detach /home/ubu/projects/lay-before-cleanup-view \
  cb40ef29f6c78c97757dd0059c0ba798cb1f0789
```

Этот snapshot нужно сохранять в Git history. Он содержит все 6 173
отслеживаемых и неигнорируемых проектных файла, включая изменения, которые
ещё не были закоммичены в исходной папке.

Независимая локальная копия:

```text
/home/ubu/.local/state/lay/project-snapshots/20260908-before-cleanup/
  project-files.tar
  source-manifest.json
  snapshot-receipt.json
  cleanup-delete-manifest.json
  RESTORE_AND_CONTINUE.md
  autocorrect-evidence/
```

SHA-256 tar:
`e07e4e5d9f38917538571b268abdb461869a86fa05bbbbc94b01f0d75ac41996`.
Содержимое всех файлов проверено. Исходная папка
`/home/ubu/projects/lay-tech-debt-20260831` также сохранена.

Игнорируемые private/model/cache artifacts не входят в Git snapshot или tar.
Они остались в исходной папке и по указанным в receipts внешним адресам.
Ранее externalized evidence лежат в
`/home/ubu/projects/lay-immutable-evidence/content-addressed-v1`; инструмент
`scripts/research-evidence-store.py` и два inventory TSV сохранены. Новая
чистая копия не обещает наличие всех старых ignored symlink projections.
Для старого воспроизведения используйте исходную папку и её исходный контекст.

[Результат очистки](docs/project-cleanup-2026-09-08.md) ·
[Точка продолжения](CONTINUE.md) · [Текущая очередь](tech_debt/README.md)
