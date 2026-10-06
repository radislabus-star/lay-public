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
git worktree add --detach /workspace/local/lay-before-cleanup-view \
  cb40ef29f6c78c97757dd0059c0ba798cb1f0789
```

Этот snapshot нужно сохранять в Git history. Он содержит все 6 173
отслеживаемых и неигнорируемых проектных файла, включая изменения, которые
ещё не были закоммичены в исходной папке.

Независимая локальная копия:

```text
/home/local-user/.local/state/lay/project-snapshots/20260908-before-cleanup/
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
`/workspace/local/lay-tech-debt-20260831` также сохранена.

Игнорируемые private/model/cache artifacts не входят в Git snapshot или tar.
Они остались в исходной папке и по указанным в receipts внешним адресам.
Ранее externalized evidence лежат в
`/workspace/local/lay-immutable-evidence/content-addressed-v1`; инструмент
`scripts/research-evidence-store.py` и два inventory TSV сохранены. Новая
чистая копия не обещает наличие всех старых ignored symlink projections.
Для старого воспроизведения используйте исходную папку и её исходный контекст.

[Результат очистки](docs/project-cleanup-2026-09-08.md) ·
[Точка продолжения](CONTINUE.md) · [Текущая очередь](tech_debt/README.md)


## Очистка публичного дерева — 2026-10-06

[Аудит по 50 пунктам](docs/repository-audit-2026-10-06.md) выполнен от
коммита `7e056e17`. Из текущего дерева вынесены локальная история запросов
Graphify, его машинные кеши, старая телеметрия очистки рабочего компьютера и
пять исторических shadow-корпусов, которые не читаются исходниками,
установщиками, CI или сохранёнными инструментами. Манифесты корпусов остаются
историческими записями с исходными размерами и SHA-256; их статус относится
к соответствующему снимку, а не к наличию payload в текущем дереве.

Архивированные корпусные payload:

- `data/lexical_grokking/lay_en_lexicon_300k_shadow_v1.txt`;
- `data/lexical_grokking/lay_ru_lexicon_462k_shadow_v1.txt`;
- `data/lexical_grokking/lay_ru_lexicon_composite_shadow_v2.txt`;
- `data/lexical_grokking/lay_l11_ru_composite_en300k_shadow_v2.txt`;
- `data/lexical_grokking/lay_l11_l2_missing_lemma_seeds_ru_v1.txt`.

Восстановление любого архивированного файла:

```sh
git show 7e056e17:data/lexical_grokking/lay_en_lexicon_300k_shadow_v1.txt > /tmp/lay-en-shadow-v1.txt
```

Для старого эксперимента следует восстановить весь его исходный снимок,
а не смешивать корпус старой версии с текущим кодом. Корпус V1, используемый
сохранёнными ablation-скриптами, рабочие словари, L2/L3-пакеты и тестовые
фикстуры остаются в текущем дереве.

Четырнадцать вынесенных файлов дополнительно сохранены в независимом tar.
Все члены архива проверены по SHA-256 до удаления; SHA-256 tar:
`ce0ca28b48cc8b82d4df1625693e202f021a1d66a35c6afa3741a0bdbb5f60ab`.
Объём вынесенных файлов: **38 143 568 байт (36,38 MiB)**.
Грубые цитаты в документации отредактированы; исходные записи доступны
в предыдущем коммите. Исторические пути локальных ревью отображаются как
текстовые ссылки на снимок, а не как работающие публичные гиперссылки.
