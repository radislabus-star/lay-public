# Независимый review плана долга — pass 1, 2026-10-09

## Findings

### F1 — P1: TD-130 проверяет переданные строки, но не связывает их с исполняемым case

**Доказано по плану и текущему consumer.** TD-130:24–29 принимает `expected from/to`
и слова от вызывающего, TD-130:40–43 оставляет live helper и весь его caller вне
изменения. Не назван versioned consumer нового classifier, источник ожидаемой
пары и обязательная проверка результата. README-вызов нового CLI сам по себе
не предотвращает выбор прежнего неподходящего helper.

Текущий consumer находится вне Git:

- `/home/ubu/.cache/lay/development/kitty-boundary-refusal-20261009/gost-evidence-carrier/client_probe.py:212–217`
  выбирает конкретный helper по пути и SHA;
- там же `:269–277` исполняет prefixed inverse, но передаёт в `finish()` только
  token; ожидаемая пара в этот вызов не входит;
- `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/owned_inverse_feedback.py:78–81`
  содержит собственную prefixed literal pair;
- `/home/ubu/.cache/lay/development/first-word-boundary-20261008/physical-native-clear/owned_inverse_feedback.py:78–81`
  содержит другую, unprefixed pair;
- первый helper `:97–98` возвращает `NO_OWNED_INVERSE_REJECTIONS` при пустой
  выборке, а `:153–154` возвращает путь receipt при любом нормальном исходе;
  consumer `:277,284–299` не читает status этого receipt. Его stop condition
  проверяет отсутствие пути после exception, а не доказанный результат cleanup.

Сохранённые `gost-evidence-carrier/THREE_FIELDS.json` и
`TWO_FIELDS_CLEANUP_REPAIR.json` действительно имеют 24/30 FAIL и 20/20 PASS.
В первых CLEANUP receipts есть NO_OWNED при 0 удалений; поздние имеют ровно
2 удаления и сохранение положительных/прочих строк. Поэтому это конкретный
дефект связи selector/consumer, а не только нехватка удобной утилиты.

**Последствие:** новый classifier может иметь хорошие synthetic tests и стать
DONE, пока прежний ошибочный маршрут продолжает обходить его. Caller-supplied
пара также может повторить ту же ошибку копирования. PID, интервал и совпадение
поверхности не являются новым доказательством принадлежности строк конкретному
полю: production episode содержит PID/time/ordinal
(`src/typing_memory.rs:880–887`), но не case identity.

**Минимальный repair плана:** выбрать одну честную границу:

1. **Отложить TD-130 в stage 2 до появления конкретного versioned caller и
   immutable case/receipt binding — 9/10, рекомендуется.** В stage 1 записать
   доказанный дефект и зависимость, не создавать ещё один необязательный selector.
2. **Оставить исключительно offline audit — 6/10.** Явно ограничить DONE
   классификацией frozen artifacts; не утверждать устранение ошибки harness.
   Требовать независимую связь case receipt → expected pair → фактический selector
   pinned helper и проверку настоящего исторического wrong-selector случая
   приватно. Не импортировать helper для такого аудита: его top-level/call code
   не является read-only API. Synthetic fixtures остаются в Git.
3. **Практически подключённый read-only preflight — 8/10 условно.** Нужен
   именованный существующий versioned consumer, который использует один case
   binding и блокирует зависимые действия при несовпадении. До подтверждения
   такого consumer это будущий дизайн, не готовый stage-1 task.

Во всех вариантах classifier не выдаёт deletion authority, не компилирует delta,
не пишет journal/counters и не объявляет snapshot CAS-доказательством. Переносить
в stage 1 живой cleanup или его 25-file harness ради нового tool не требуется.

### F2 — P2: путь выполнения нового Python suite пока не определён

TD-130:47 добавляет `tests/test_inverse_feedback_binding.py`, а :69 обещает
«подходящий Python verification route», не называет его и изменение его selection.
Автоматического discovery всех `test_*.py` здесь нет:

- `scripts/dev-check.py:25–27,301–302` запускает фиксированный SELF_TESTS;
- `scripts/check-lay-tests.sh:20–26` имеет второй фиксированный список;
- обычный remote `dev-check check` использует этот self-test и Rust lanes
  (`scripts/dev-check.py:306–315`);
- `scripts/check-lay-changed.sh:64–67` только компилирует changed Python,
  а :115–117 для harness запускает прежний self-test.

**Это риск плана, не уже созданный orphan test.** Требование DONE верное,
но карточка недостаточно конкретна для минимальной реализации. Если TD-130
остаётся активным, выбрать существующий remote self-test route, назвать оба
списка либо один уже существующий test module, exact command и ожидаемые nonzero
identities/counts. Не создавать новый runner. Если менять protected check/CI,
включить новый architecture decision. Если task отложен, сохранить эту
зависимость в preconditions, не менять runner заранее.

### F3 — P2: TD-131 пропускает обязательное решение; TD-132 может вынести код из защиты

TD-131:24–26 меняет `tail_memory.rs`, но :41–44 и DONE не называют новый explicit
decision и owning architecture entry. Этот файл прямо защищён
`scripts/check_architecture_canon.py:37`; `ARCHITECTURE.md:187–190` требует новое
решение для защищённого изменения. TD-132:47–49 это требование уже называет.

Возможная extraction TD-132:21 создаёт
`window_interaction/observation/retired_preedit.rs`. Guard защищает только
`observation.rs` (`scripts/check_architecture_canon.py:38`), а `protected()`
сопоставляет exact file names или CI directory (:86–87). После переноса будущая
правка только нового leaf не потребует решения этим guard. Одно initial decision
для старого parent не обеспечивает дальнейшую защиту вынесенного кода.

**Repair:** у активного TD-131 до кода назвать owning document, новый decision,
защищённые пути и source-only verdict. При будущей TD-132 сохранить покрытие
нового leaf guard-ом с новым явным решением и отрицательным guard test, а не
ослаблять действующие checks. Если TD-132 отложен, эту зависимость записать,
но guard сейчас не менять. Это небольшая конкретная цена refactor, которую
нынешний rollback «вернуть тела/import, убрать tests» также должен учитывать.

### F4 — P2: обязательный TD-132 имеет предположительный выигрыш при уже чистой границе

План TD-132:8–16 обосновывает 9/10 перенос размером observation и смешением
обязанностей. Однако выбранные функции **уже** являются соседним pure block:

- `observation.rs:117–225`: PublishedPreeditWitness, right-edge predicate и
  snapshot predicate; нет RPC, await, state writes или output dispatch;
- :2645–2657: один production caller сохраняет witness unconfirmed;
- `context_admission/adapter/tests/residuals.rs:4973–5041` проверяет реальный
  bridge, отсутствие Delete/Commit и свежий exact receipt;
- там же :5219–5265 есть positive/negative stale-publication cases, отсутствие
  edit target/authority и отсутствие mutation, а не только проверки source text.

Не найдено доказательства, что именно этот block породил defect, мешает isolated
verification или имеет ненужные зависимости. Перенос остаётся возможным
maintenance refactor, но не доказанным необходимым улучшением. Упрощение чтения
всего 4325-line file не следует автоматически из удаления примерно 100 строк.

**Repair:** предпочесть перенос task в stage 2 — **9/10** до конкретного
maintenance benefit/consumer map. Сохранение уже чистого и проверенного block
— **8/10**. Move-only extraction сейчас — **6/10**, может стать **8/10** после
обоснования пользы, reuse coverage и учёта F3. Новые controller/trait/type owners
— **2/10**. Не создавать заново полный test matrix, если существующие tests
уже проверяют нужные effects; добавить только установленные semantic gaps.
Порядок/dependencies TD-131 и TD-134 соответственно нужно перепривязать.

### F5 — P2: отдельный доказанный cleanup race не отражён как незакрытый долг

Audit:63–68 и TD-130 называют selector root и сохранение «тройного CAS».
Это не весь известный cleanup failure class. Owning document
`docs/architecture/space-boundary-shift-2026-10-07.md:1671–1674` фиксирует
CAS failure **после записи counters**, загрязнённые зависимые cases и отдельное
восстановление. Source helper `owned_inverse_feedback.py:138–144` сначала
проверяет три файла, затем заменяет counters, снова проверяет journal и только
после этого заменяет journal. Этот порядок допускает частичный результат;
он не является атомарной транзакцией двух файлов.

**Доказано:** такой partial failure уже был записан; механизм виден в source.
**Не доказано:** его нынешняя частота, crash-поведение или необходимость нового
runtime protocol. Этот review не предлагает менять working helper в stage 1.

**Repair плана:** сохранить это отдельным open stage-2 tooling debt с exact
receipt/source links, BLOCKED dependent work после unknown/partial cleanup и
явной recovery boundary. Binding-only DONE не закрывает transaction/recovery
scope. Рассмотреть shared compiler/loader generation contracts до нового lock,
queue или writer. Новая concurrency/recovery задача требует собственных
consequences, failure injection и независимого review; сейчас достаточно
явно учесть долг, не реализовывать его.

### F6 — P3: описание текущей очереди содержит небольшую фактическую ошибку

Audit:58 и TD-129:9 говорят, что README/CONTINUE называют R5 текущим.
README:1–12 действительно начинает с R5/C20. Но CONTINUE:1–8 уже начинает
с R12/1.0.72; ниже :10–13 всё ещё пишет «Latest development R7». Проблема
— устаревший верхний checkpoint относительно 1.0.81 и конкурирующие latest
headings, а не отсутствие R12 в CONTINUE. Поправить описание корня, не
переписывать исторические receipts. TD-129 остаётся действительно нужным.

### F7 — P3: stage-2 старые scopes нуждаются в current mapping, а не ещё в одной очереди

Audit:136 сводит TD-128/127/121 к reproduction исходного scope. Это хороший
предохранитель против fake closure, но недостаточно конкретный stage-2 handoff:
TD-121 имеет тысячи строк и несколько исторических current headers; TD-123:3–20
описывает 1.0.67/старые installed bytes, а реальный grammar research owner
отдельный. Нельзя дать этим старым runtime statements новую current authority.

TD-129:27–32 должен сделать для каждого такого task небольшой current map:
исходный scenario/invariant, последний scoped receipt, статус current 1.0.81
`UNKNOWN`/отдельно доказанная область, следующая нерешённая граница и owning
checkout. Не возобновлять уже закрытую пользователем приёмку целевой пары под
видом TD-121 reproduction. В current index нужен точный указатель на
`/home/ubu/projects/lay-syntax-agreement-20260929/docs/architecture/ru-agreement-roadmap-2026-09-30.md`
и branch `codex/lay-syntax-agreement-20260929`; там :46–58 определены единственный
owner и L3 scorer scope. Здесь не создавать research stage, новый fit или
второй roadmap. Возможность продолжать каждый old task после обсуждения
не равна разрешению на его старые install/restart действия.

## Вердикт

**REQUEST_CHANGES — 7/10.** Основные ограничения owners/routes/verifier и
разделение source/runtime/physical заданы хорошо. Открытые FAIL не скрыты,
проценты coverage не выдуманы. Но обязательный stage 1 пока содержит
неподключённое средство для selector defect и обязательную extraction с
необоснованным выигрышем; execution/canon детали недостаточно конкретны.
F1–F5 нужно адресовать одним grouped repair плана. F6–F7 — короткие правки
фактов и current mapping, без переписывания истории.

Рекомендуемый stage 1 после repair: **TD-129 → TD-131**. TD-130 и TD-132 сохранить
открытыми в stage 2 с явными условиями возврата. Это не закрытие отказом от
реализации. Первая обсуждаемая продуктовая работа stage 2 — причинный TD-133;
tooling binding/recovery остаются необходимой предпосылкой достоверного нового
physical proof. Mapping TD-136 должен информировать выбор границы TD-134,
а не идти после неё только по номеру. Research остаётся отдельным owner.

Не нужен новый механизм ради заполнения карточки. TD-131 имеет настоящую
duplicate-invariant цель: `preedit.rs:1617–1632` и `tail_memory.rs:1750–1759`
реализуют одну scalar-tail=160 политику. `context_admission.rs:139–142` связывает
её с u8 offsets; `observation.rs:3187–3193` использует тот же retained limit.
Переиспользование существующего fixed helper, удаление копии и сохранение
порядка :486–502/`preedit.rs:1456–1465` — минимальный разумный refactor,
**8/10** после F3. Текущего неверного trimming этот review не обнаружил.
Никаких новых bounded-tail objects или произвольного limit API не требуется.

## По карточкам

Оценки — инженерное суждение о текущей подготовке карточки, не процент
качества runtime и не замена gates.

| Карточка | Оценка | Заключение / следующий минимальный результат |
| --- | ---: | --- |
| 129 | 8/10 | Нужна. Исправить F6, сделать current map F7 и перенастроить dependencies после решения об 130/132. Metadata проверять по ссылкам/status, без behavior rerun. |
| 130 | 5/10 как preventive task | F1/F2/F5. Предпочтительно stage 2; audit-only вариант не закрывает caller/cleanup defect. Нужны именованный consumer, case binding, exact verifier selection и честный scope. |
| 131 | 8/10 | Доказанная копия invariant, маленькая правка. F3 до кода, reuse existing coverage; характеристика обеих production entrypoints там, где есть gap. Это source refactor, не bug repair и не новая live acceptance. |
| 132 | 6/10 как обязательный stage 1 | F3/F4. Предпочтительно stage 2 до доказанного maintenance benefit. Pure block и реальные no-mutation tests уже существуют. |
| 133 | 8/10 investigation, implementation NOT_READY | Верно оставляет первый failed transition UNKNOWN и запрещает sleep/retry. Не смешивать native 64, b59 comparison и новые traces; сначала frozen event/owner order. F5 нужно учесть в достоверности physical proof. |
| 134 | 8/10 conditional investigation | Один existing-owner slice, side-effect map и move-only граница разумны. Не считать TD-132 обязательным техническим dependency; coverage map и конкретная польза должны предшествовать выбору split. Новые child paths должны сохранить guard coverage. |
| 135 | 8/10 baseline task, implementation NOT_READY | Cargo.toml:13–17 и :31–49 уже разделяют часть tools; l2_field/mod.rs:1–20 содержит ungated compiler/proof modules. Это доказывает surface, не executable bloat. Не обещать bytes/RSS/speedup до reachability/link baseline. Feature/API/package tests и обратимость обязательны. |
| 136 | 8/10 mapping task | Хорошо отличает source guards и semantic effects; большие test files не являются дефектом сами по себе. Нужен bounded mapping по unresolved mechanisms; splitting только после найденного navigation/isolation benefit. F2 не оставлять до этого общего исследования. |
| 121 | Current scope требует карты | R12 PASS и поздние entries не закрывают весь исходный handoff scope и не дают права повторить старые installs. Одно current entry из 129, без нового 5000-line task. |
| 122 | 8/10 deferred design | Existing task уже требует reachability, completion receipt и backward policy. Сохранить открытым; не строить begin/finish protocol до доказательства V1 value и конца synthetic stream. Delivery routes stage 1 сохраняются. |
| 123 | Delegated research scope | В текущей очереди нужен exact owner link и статус, а не новый эксперимент. Ranking, clean/lattice/false certainty/package/RSS/latency остаются conjunctive; общий top-1 не заменяет классы. |
| 127 | Untriaged product evidence | Пользовательский Space/Tab discrepancy остаётся отдельным неподтверждённым механизмом. Правильные pair cases 1.0.81 его автоматически не закрывают. |
| 128 | Historical reproduced defect | Исторические bytes и 11 rejected transfer contracts сохранены. Current status на 2bd не установлен этим review; новое разрешённое исследование должно начать с original same-window transfer, а не fresh-field PASS. |

## TDD, последствия и rollback

Общий цикл audit:149–162 (один initial и один final pass, потом
REPLAN_REQUIRED) принят. Review score и объективный PASS остаются разными
условиями; description или deferred spec не становятся DONE. После repair
нужен только второй plan review; третий проход не предусмотрен.

Для активного TD-131 минимальная проверка: показать прежнее production behavior
через append и composition-commit, scalar suffix/limit и уже существующую
отмену first-word authority. Existing
`context_admission/adapter/tests/terminal_delivery.rs:227–260` уже проверяет
revocation при trim. Наличие current GREEN допустимо; new test нужен лишь для
незакрытой semantic границы, не для факта появления общего helper. Controlled
limit mutation — опциональное доказательство sensitivity, не production edit.
В mutation/TDD receipts строго различать original defect RED и controlled
violation RED. Затем affected remote IME/integration contracts, canon/graph
binding и owning entry. Полный release/physical не следуют из source refactor.

Consequence analysis 131 разумно ограничивает candidate/rank/verifier,
feedback/package/cache/owners и callback order. Утверждения об идентичных
allocations/latency — ожидаемые свойства эквивалентного алгоритма, а не новые
измерения. Проверить wrapper equivalence именно при fixed160; generic helper
оставить private, не расширять его limit API ради этого consumer. Rollback —
один source commit revert с нужным decision/evidence successor, accepted bytes
не заменяются. Extraction 132, если позже будет допущена, дополнительно должна
учесть guard registration в стоимости и rollback; полезные characterization
tests не удалять автоматически только потому, что код вернулся в parent.

Stage 2 не готова к автоматической production execution, и это правильно:
133 не имеет causal root, 134 не выбрал slice, 135 не имеет measured reachability.
Нельзя заменить недостающие факты уверенностью в оценке 8/10. Не добавлять
таймеры, кеши, новые state owners, маршруты или literal exceptions. Любые
future authority/input changes требуют собственных пределов и exact-byte proof.

## Фактически изученный scope и unknowns

Read-only review выполнен на branch `codex/space-boundary-shift-20261007`, HEAD
`e7a25705bb6196776c35c0e3cb7c3b5a17d55169`. До review production checkout чист;
untracked были только audit и новые 129–136. Review не изменил ни один из них.

Прочитаны actual AGENTS.md и ARCHITECTURE.md; audit и все 129–136; верхние current
sections README/CONTINUE/121, 122/127/128 полностью и relevant 123 sections;
DEVELOPMENT.md; relevant canon/verification selector sections. Использован
существующий Graphify query по vocabulary `inverse feedback tail trim preedit
retired admission observation receipt cleanup binding scope` (BFS, 543 scoped
nodes, bounded output), после него факты подтверждались source. Graph свежесть
не переаттестована; missing `.graphify_python` не создавался ради read-only review.

Source slices: preedit trim/append, tail_memory composition commit/trim,
WordLineage first-word offsets, observation retired-preedit predicates/caller
и append detection; referenced production-adapter regression bodies; Cargo
features/targets и L2 module declarations. Read-only cache evidence: pinned
inverse helper bodies, wrong/current browser-driver binding and result handling,
CURRENT_RESULT, full-source receipt summary, original/repaired browser aggregate
и cleanup statuses. Из owning architecture прочитаны соответствующие поздние
receipt/cleanup entries. В authoritative research roadmap проверены только
owner/checkout/scope pointers; research модели и результаты не ревьюились.

Приёмка 1.0.81 использована как **сохранённая** accepted evidence:
3044/3044 source, Kitty 7/7 + 3/3, native 62/64 (2 FAIL, 0 BLOCKED), browser
input 10/10 и отдельно dependent repair 20/20 при сохранённом 24/30 FAIL.
SHA установленного в receipt IME:
`2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559`.
Universal acceptance — NOT_MET. Сам source gate receipt указывает исходный
functional carrier commit `9fdd1992…`; CURRENT_RESULT отдельно указывает exact
functional source и metadata-only owning-doc changes. Эти receipts не являются
новым прогоном на будущих refactor bytes.

**Не выполнялись:** tests/builds, graph refresh, runtime hash/readiness проверка
заново, install/restart, browser/physical input, remote desktop, network research,
новые source experiments. Доказательства прошлых PASS не переоткрывались.
Текущий физический runtime не измерялся; root двух native FAIL, current 127/128,
полное semantic/branch coverage, linked-size/RSS/latency эффект 135 и безопасность
всех строк репозитория остаются UNKNOWN/NOT_TESTED в scope этого review.

Production/delivery authority changed: **NO**. Единственная запись этого прохода —
данный review. Один grouped repair плана и один final pass остаются допустимыми.
