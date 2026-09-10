# Очистка новой рабочей ветки — 2026-09-08

Статус: APPLIED_AND_VERIFIED; очистка принята в указанной ниже области.
Владелец текущей работы: эта запись. После очистки продолжить
[TD-123](../tech_debt/123-improve-wave-restoration-quality-for-1.0.67.md).

## Поручение и исходная точка

Пользователь поручил создать новую ветку, перенести проект и очистить его
с ориентиром «до 80%». Отдельно потребовал сохранить результаты автокоррекции
и продолжить их после очистки. Процент файлов, байтов и строк Rust считаем
раздельно; один показатель не заменяет другой.

- Исходное рабочее дерево: `/home/ubu/projects/lay-tech-debt-20260831`.
- Новая копия: `/home/ubu/projects/lay-cleanup-20260908`, ветка
  `codex/cleanup-20260908`.
- Полный исходный Git snapshot: `cb40ef29f6c78c97757dd0059c0ba798cb1f0789`.
  Он включает незакоммиченные изменения исходной папки.
- До очистки: 6 173 отслеживаемых и неигнорируемых файлов; в `src/`
  697 Rust-файлов, 300 913 физических строк с комментариями и тестами.
- Независимый архив: `/home/ubu/.local/state/lay/project-snapshots/20260908-before-cleanup/project-files.tar`.
  SHA-256: `e07e4e5d9f38917538571b268abdb461869a86fa05bbbbc94b01f0d75ac41996`.
  Все 6 173 файла проверены по содержимому, новая копия также проверена по
  содержимому, режимам и ссылкам. Исходная папка сохранена целиком.
- Игнорируемые build/cache/private artifacts не входят в этот tar; они
  сохраняются в исходной папке и по прежним внешним адресам.

В той же папке архива лежат `source-manifest.json`, `snapshot-receipt.json`,
`RESTORE_AND_CONTINUE.md` и `autocorrect-evidence/`: проверенные копии
установленного IME и ключевых receipts. Это постоянная точка восстановления,
не disposable cache новой ветки.

## Выбор границы

Оценки ниже — инженерное сравнение, не измерение производительности.

| Вариант | Сохранность поведения /10 | Простота продолжения /10 | Затраты | Решение |
|---|---:|---:|---|---|
| Исходное дерево без очистки | 10 | 4 | низкие | Контрольная копия |
| A. Архивировать завершённые receipts, отчёты, V10/V11 controllers и доказанно неиспользуемые данные | 10 при сохранении dependency closure | 9 | умеренные | Выбран |
| B. Разделить shared runtime helpers и cold compiler/proof существующими features внутри crate | 8 до проверки | 7 | высокие | Жизнеспособный следующий этап; перенос строк сам по себе не уменьшает код |
| C. Выделить version-pinned research-проект с общей runtime/format библиотекой | 6 до проверки | 7 | высокие | Требует миграции API, install и gates; не предпосылка продолжения TD-123 |

Измеренный source audit: тестовые spans около 82 990/300 913 строк (27,6%).
Объединение тестовых spans, существующих feature-блоков и собственных файлов
research CLI — около 116 027 строк (38,6%), с исключением пересечений.
Это граница возможного **разделения**, не разрешение удалить тесты или 38,6%
работающего кода. Основания обещать удаление 80% Rust-кода сейчас нет.

Компиляторы и proof не образуют изолированную папку: например,
`packaged_runtime -> semantic_estimator -> productive_v1::proof`, admission
читает `proof_matrix::DAMAGE_CLASSES`, V13 использует Phase7d certificate,
а online L3, reload и legacy package loaders обслуживают runtime. Названия
`proof`, `legacy`, `compiler` сами по себе не являются критерием удаления.

## Сохраняемый контракт

- Все исходники `src/`, integration tests, Cargo inputs, активные scripts,
  installers, test lanes, guards и `scripts/proof/`.
- Данные и receipts, реально читаемые retained code/tests/installers;
  удаление крупных данных допускается только после аудита их потребителей.
- Owning architecture documents, `tech_debt/113-*`, задачи 120–125,
  актуальные evidence 120–125 и release 1.0.66, текущая очередь и handoff.
- Пять обязательных `tech_debt/evidence/` inputs:
  `v28-ru-en-zero-failure-observation-v1.json`,
  `td113-implementation-preflight-v4.json`,
  `td120-composition-mutation-successor.json`,
  `td121-composition-mutation-successor.json`,
  `td121-residual-acceptance-review.md`.
- Четыре L1/L1.1 package receipts, на которые ссылаются install manifests,
  и девять fixed-case/provenance файлов `LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/`
  (семь верхнего уровня и два вложенных immutable-rerun manifests).
- `research-evidence-store.py`, его проверки и два TD-103 inventory TSV;
  внешний `lay-immutable-evidence` не очищается.

Независимый аудит выявил зависимость устаревшего
`refresh-l2-transition-phase-gate.sh` от архивируемых profile/route. Этот
неиспользуемый действующими scripts wrapper архивируется вместе с ними;
canonical CLI proof, development и release gates сохраняются.

Точный план удаления хранится рядом с исходным архивом в
`cleanup-delete-manifest.json`. Перед удалением он проходит независимый
review; каждый путь должен совпасть с исходным content hash. Удаления
разрешены только в новой копии. Исторические пути восстанавливаются по Git
snapshot, инструкции будут в корневом `ARCHIVE.md`.

## Последствия до изменений

Candidate/lattice retention, ranking, false authority: runtime-код и его
конфигурация остаются прежними; не меняем SafetyGate, verifier, границы
L1.1/L2/L3/L4 или механизм Double Shift. Риск — ошибочно удалить proof input;
его ограничивают аудит зависимостей и полный существующий correctness/package
набор после очистки. Качество не повышается от удаления файлов.

Latency/tail deadlines, CPU/RSS/allocations: новых hot-path операций нет.
Уменьшение transfer/snapshot возможно, но ускорение ввода или снижение RSS
не заявляются без измерений. Объём каталога и transfer time считаются отдельно.

Cache identity/invalidation, package/delta reload: current package bytes,
форматы и loaders сохраняются. Старые данные удаляются только при доказанном
отсутствии retained consumers. Будущая воспроизводимость исторических
экспериментов требует исходного snapshot вместе с его данными и scripts.

Learning/feedback, concurrency/stale results: код и состояния не меняются,
новые владельцы, очереди, timers, caches и fallbacks не вводятся. Online
обучение не классифицируется как отработанный исследовательский материал.

IME/daemon compatibility и rollback: установленные бинарники, процессы,
пользовательские настройки и модели не меняются. Возврат исходников — из
snapshot-коммита или проверенного tar в отдельную папку; revert нового
cleanup-коммита возвращает удалённые файлы. Исходный checkout остаётся.

Maintenance: действующие указатели переписываются на короткую очередь,
канонические документы и архив. Старые receipts остаются историческими:
их прежний PASS не становится текущей product acceptance. Главный риск
очистки — потерянный путь воспроизведения; snapshot и точный manifest
сохраняют его без дублирования тысяч файлов в рабочем дереве.

## Проверка и незакрытые вопросы

План проверки: независимый review delete-list; сверка hash всех удаляемых
файлов; нулевой diff Rust-исходников, tests, Cargo и current runtime/package inputs
(сгенерированный architecture receipt обновляется вместе с graph);
существующие remote correctness/package lanes; затронутые Python/script
contracts; remote `scripts/update-architecture-graph.sh`; final diff-check
и независимый review результата. Все исполнения только через существующие
remote guards: dedicated-20cpu, Cargo jobs 20, Rust test threads 1,
CPU 2000%, MemoryHigh 24G, MemoryMax 28G, swap 1G, TasksMax 512,
Cargo target <=12 GiB. Установка и production restart не нужны для очистки.

Текущий IME: SHA-256
`995b609343aa7b3bc9fad628cc80df09ea1b2125753c4d62c42b3b0790d50fb0`.
Для этих установленных bytes ранее прошли 2 684/2 684 code tests и 13
client scenarios. Фиксированные существующие native fixtures: восстановление 19/47, сохранение clean
39/42. `проврка` в финале фразы исправляется, `впрверка` пока нет.
Обычный темп физического ввода остаётся PENDING. Полный conjunctive contract,
включая strict unique top-1 >95% для каждого damage class, не достигнут.
Очистка не меняет эти verdicts и не завершает TD-123.

Результаты после выполнения, exact receipts, итоговые счётчики и review
будут добавлены ниже. Runtime authority changed: **false**.

## Независимый preflight review

`first_word_review`: условный PASS 9/10 для первоначального manifest
`8f8c7746754d7c088730ffcf1fd9f3aca2a833977e1f8f35d21c6f6d1000fe14`.
Проверены 4 791 пути, их SHA/bytes/mode/type, обязательные retained inputs
и восстановление. Найдены Medium: устаревший live-gate wrapper терял свои
profile/route; Low: research README предлагал удаляемый SHA manifest.
До удаления wrapper включён в тот же архивируемый набор, research README
переписан на восстановление полного snapshot. После этих коррекций H0/M0/L0.
Итоговый manifest и отдельный data audit требуют заключительного review.

`shift_fix_review`: source audit подтверждает отсутствие обоснованного
80% Rust delete-list. Data audit отдельно оставляет три raw RU/EN корпуса:
`build-bilingual-l1-corpus.py` читает их через source manifests. Сохранены
оба merged shadow corpus, V4 seed inputs, current packages и все данные
conditional/ignored tests. Список восьми устаревших файлов без retained
потребителей будет явно добавлен в manifest перед удалением; 330 055 990 bytes.

Data preflight `shift_fix_review`: PASS 9/10, H0/M0/L0. Восемь точных путей:

- `data/l2/LAY-L2-RU-FULL-v4.bin`
- `data/l2/LAY-L2-RU462K-NOUN-v1.bin`
- `data/lexical_grokking/l1_l11_crystal_e2_complete_10k.bin`
- `data/lexical_grokking/l1_l11_crystallization_10k.bin`
- `data/lexical_grokking/l1_l11_multimodal_restoration_10k.bin`
- `data/lexical_grokking/l1_lexical_grokking_10k.bin`
- `data/morphology/lay_ru_noun_morph_462k_shadow_v1.tsv`
- `data/morphology/russian_noun_cases_300k.tsv`

Проверены basename, составные/default paths, conditional/ignored tests,
directory/glob и generic manifest readers. В действующем L2 контракте V13;
старые L2/L1 bins не используются. Для morphology proof сохранены small TSV
и productive axis schema. Итоговый manifest перед удалением: `08c8840941e23fa4ba704ce555eb0d0628add2a4638856300b00dc95880885af`.


## Применённое удаление

2026-09-08T13:24:34Z: удалены 4 800/6 173 исходных файлов (77,76%),
443 055 352/646 770 880 bytes (68,50%) до учёта сокращения navigation и новых
коротких документов. Существовавших retained файлов осталось 1 373;
финальный счётчик включает новые handoff/index/receipt файлы отдельно.
Manifest SHA-256:
`08c8840941e23fa4ba704ce555eb0d0628add2a4638856300b00dc95880885af`.
Exact execution receipt:
`/home/ubu/.local/state/lay/project-snapshots/20260908-before-cleanup/cleanup-execution.json`.

Первая попытка остановилась после 4 036 удалений на архивном каталоге с
режимом 0555; неожиданных удалений не было. Все 764 оставшихся файла повторно
сверены по SHA/size/mode/type. Девять parent directories **только новой копии**
временно получили owner-write для удаления; режимы уцелевших каталогов
возвращены. Исходные protected evidence и tar не менялись. Сохранён
`cleanup-partial-failure.json`; это filesystem repair, не пропущенная проверка.

Из удалённого исторического tooling: 99 801 строк Python, 455 shell,
21 770 injected Rust fragments. Это физические строки с комментариями;
они не входят в сохранённый denominator `src/` 300 913 Rust-строк.


## Итоговый review и correctness/package proof

Final `first_word_review`: PASS 9/10, H0/M0/L0. Фактические 4 800 удалений
точно совпадают с manifest. Все 1 367 retained baseline файлов вне шести
намеренных navigation edits совпадают по SHA/size/type/mode. Все 62 локальные
ссылки девяти обновлённых navigation документов разрешаются. Данные приняты
по отдельному review 9/10. Это source/dependency review, не модельный proof.

Root preservation check также подтвердил отсутствие content drift всех
6 173 файлов исходной папки и отсутствие diff retained Rust/tests/Cargo/data.
`src/` остаётся 697 Rust-файлов / 300 913 физических строк. Установленный IME
повторно имеет тот же полный SHA-256 995b6093…0fb0.
Exact receipt: `cleanup-preservation-check.json` рядом с исходным tar.

Remote `python3 scripts/dev-check.py check` после удаления: PASS.
Source archive SHA-256:
`d7fb8bbc38d02c1fc2fbf40c43da2479a269e3779101eb8d3eb4c6e563194101`.
Correctness 2 648 + package 36 = **2 684/2 684**, semantic failures 0,
infrastructure failures 0. Manifest total 2 710: performance 11 и ignored 15
остаются отдельно и не входят в этот PASS. Lane self-test: 102 выполненных
проверки с одним штатным optional skip; fmt PASS.
Worker time 377,912 s, correctness/package stage 373,923 s. Это measured
время данного запуска; сравнения ускорения разработки не проводилось.
Local: `/home/ubu/.cache/lay/development/run-xi6v16vi/{RESULT.json,run.log,request.json}`.
Remote: `/home/e/projects/lay-development-runner/run-2FzzMW/`.

Postcheck recorder v1 остановился **до первой проверки**: Python 3.10 не имеет
`hashlib.file_digest`. Использован chunked SHA-256 с тем же expected hash,
новый helper/output `cleanup-postchecks-v2`; failure сохранён. Runtime-код,
тестовые assertions и уже успешный 2 684-test набор не менялись и не
перезапускались ради recorder. Graph/tooling/install checks записываются
в `run-2FzzMW/cleanup-postchecks-v2/RESULT.json`.


## Приёмка очистки

Postchecks v2 PASS, 44,050 s: полный AST graph refresh с binding/receipt и
обязательным architecture check; 46/46 architecture-tooling tests; три
существующих L1.1/L2/release installer regression suites; синтаксис 42 Python
и 59 shell файлов. Неизменность всех snapshot inputs вне graph/generated
receipt подтверждена. Cargo target до/после 9 397 063 680 <12 884 901 888 bytes.
Existing advisory file-size budgets в architecture check остаются видимыми;
их превышения не выдаются за сокращение runtime-кода.

Независимые reviews, 2 684-test gate и postchecks дают PASS **очистки
рабочего дерева и сохранения действующих контрактов**. Новый release,
physical-input acceptance, качество модели, latency/RSS и production install
здесь не проверялись и не объявляются. Runtime authority changed: **false**.

После этой записи выполняется финальный remote graph refresh, чтобы сама
документация результата вошла в граф; итоговый source binding и receipt
остаются canonical. Его точный лог:
`/home/e/projects/lay-development-runner/run-2FzzMW/cleanup-final-graph.log`.
Итоговые счётчики и новое дерево файлов сохраняются рядом с исходным архивом
в `cleanup-final-metrics.json` и в
`/home/ubu/Загрузки/lay-cleaned-tree-with-lines-2026-09-08.txt`.

Копии компактных результатов: `cleanup-evidence/` рядом с исходным tar.
Source baseline, установленный IME, все текущие inputs и незавершённая
автокоррекция сохранены. Следующая работа — TD-123 по текущему handoff,
без возобновления архивированных экспериментов ради числа удалённых строк.
