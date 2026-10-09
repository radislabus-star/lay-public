# Независимый final plan review — pass 2, 2026-10-09

**ACCEPT — 9/10.** Незакрытых correctness findings к исправленному плану нет.
Допущен минимальный stage 1: **TD-129 → TD-131**, в описанных metadata/source
границах. Stage 2 остаётся для отдельного обсуждения и preflight; этот вердикт
не допускает его production execution. Это второй и последний проход плана
после одного grouped repair. Третий проход не требуется и не предусматривается.

## F1–F7: результат единственного repair

| Finding pass1 | Resolution | Текущая конкретная граница |
| --- | --- | --- |
| F1: необязательный classifier без case/caller binding | RESOLVED IN PLAN | TD-130:3–5,29–50 исключает standalone tool из stage 1. Конкретный versioned consumer пока не выбран; это явно prerequisite, а не выдуманная подключённость. Один binding должен породить input/ожидания/receipt/selector; caller обязан прочитать actual cleanup status. Receipt path, PID/time и NO_OWNED не превращаются в proof успеха. |
| F2: риск orphan Python suite | RESOLVED IN PLAN | TD-130:52–56 выбирает существующий tests/test_ime_client_harness.py и remote dev-check self-test --compact; при новом module требует оба действующих списка и nonzero identities/counts. Runner сейчас не меняется. |
| F3: отсутствующий decision 131 / потеря защиты после 132 | RESOLVED IN PLAN | TD-131:6–11,66 называет owning entry и новый explicit decision до production edit. TD-132:54–62 и TD-134:30–33 требуют protection нового child, explicit decision, negative guard test и evidence successor. Deferred extraction не запускается сейчас. |
| F4: обязательная extraction без установленной пользы | RESOLVED IN PLAN | TD-132:3–4,9–26 отложен; existing pure block и adapter coverage признаны. Нужны concrete maintenance benefit и bounded TD-136 map. TD-131 зависит только от 129; 134 больше не обязан зависеть от 132. |
| F5: отдельный partial cleanup class не учтён | RESOLVED IN PLAN | Новый TD-137:7–18 фиксирует measured partial result, source и owning-history links. :30–57 требует stop зависимых cases, generation/recovery evidence, isolated failure injection и source-tooling rollback без перезаписи пользовательского journal. Binding-only closure не закрывает recovery. |
| F6: неверное описание CONTINUE | RESOLVED IN PLAN | TD-129:4,9–11,36–38 различает tech_debt/CONTINUE.md (R12, затем latest R7) и root CONTINUE.md (C20/1.0.72). В обеих точках предусмотрен current override со ссылкой на один README. Audit:57–65 согласован с этим. |
| F7: stale old scopes / второй research owner | RESOLVED IN PLAN | TD-129:29–35 требует original invariant, последний scoped receipt, current2bd UNKNOWN/доказанную часть и следующее разрешённое исследование для каждого открытого task. Exact syntax-agreement roadmap указан; нового fit/roadmap здесь нет. Старые FAIL и scoped DONE сохраняются. |

`RESOLVED IN PLAN` означает устранённую ошибку проектирования программы работ.
Сам selector defect, partial cleanup и старые product scopes остаются открытыми;
их не объявили исправленными переносом в stage 2.

## Минимальность stage 1 и факты source

TD-129 решает конкретную проблему неоднозначной точки продолжения. В этом проходе
проверены оба исходных CONTINUE: root :5–20 действительно описывает installed C20,
а tech_debt :3–13 — R12 и противоречивый latest R7. Один current README с двумя
короткими ссылочными overrides достаточен; новый tracker, пересмотр тысяч строк
истории или повторная функциональная проверка для metadata не нужны.

TD-131 имеет достаточную source цель и минимальный implementation scope:

- `src/bin/lay_ibus_engine/preedit.rs:1617–1632` и
  `src/bin/lay_ibus_engine/tail_memory.rs:1750–1759` повторяют scalar-count/drain
  с limit160; текущего неправильного suffix source review не выявил;
- `context_admission.rs:139–142` объясняет связь bounded tail с u8 offsets;
- `tail_memory.rs:486–502` задаёт composition-reset/rebuild → trim → publish,
  а `preedit.rs:1456–1465` — trim → suppression refresh/retirement → publish.
  Их порядок должен сохраниться, как и требует карточка;
- reuse только существующего fixed helper устраняет копию без нового owner,
  типа String wrapper, произвольного limit API или delivery route;
- protected tail_memory.rs действительно включён в
  `scripts/check_architecture_canon.py:37`. Новый decision/owning entry до кода
  теперь указан, а source/affected remote gates остаются обязательными.

TDD здесь — проверка существующей семантики, не выдуманный product RED. Сначала
определить закрытые и незакрытые границы existing coverage; добавить meaningful
характеризацию обеих production entrypoints только для gap. Controlled mutation
остаётся опциональной проверкой sensitivity. Existing first-word trim revocation
receipt/test не подменять helper-only assertions. Future source DONE требует
своих результатов и review; никакой новой installed/physical приёмки он не
наследует. Ожидаемая эквивалентность allocations/latency теперь явно отличается
от нового измерения (TD-131:54–56).

Rollback stage 1 остаётся source/metadata revert с сохранением исторических
receipts. Accepted installed bytes не заменяются. Новых state owners, caches,
timers, compiler calls, runtime package или layout initiator stage 1 не добавляет.

## Зависимости stage 2 и границы принятия

Приоритет TD-133 перед tooling не создаёт скрытого права на contaminated physical
run: TD-133:6–9 разрешает сначала old-trace/source investigation, а новый physical
proof требует 130/137 либо доказанной эквивалентной существующей безопасной границы.
«Эквивалентная» здесь требует доказательства, а не ручного обхода. Audit:144–145
повторяет этот prerequisite. Root двух native FAIL остаётся UNKNOWN.

TD-130 и TD-137 разделяют binding/checked result и partial generation/recovery.
Ни один task не объявлен DONE за описание. Current versioned private-client
harness не назван уже работающим host-window inverse consumer; выбранный caller
и применимость его scenario set нужно доказать до будущего кода. Старый offline
вариант в TD-130:58–105 явно сохранён как непринятый audit aid и не является
предписанием stage 1. При выборе будущей connected реализации её final spec
должен соответствовать установленному consumer и recovery boundary.

TD-136 информирует выбор 134/132; обязательная size-based extraction исключена.
Вынесенный leaf должен сохранить canon protection: current protected() на
`scripts/check_architecture_canon.py:86–87` действительно проверяет exact paths.
TD-135 остаётся reachability/package baseline без предположений о bloat, RSS
или dead code и без нового fit. Старые 121/122/123/127/128 получают current map,
не новую authority от исторических install/restart записей. Взаимного требования
завершить 132 перед полезным 131/134 больше нет.

Audit:12–25,174–176 сохраняет отдельные знаменатели: source3044/3044,
Kitty7/7+3/3, native62/64 с2FAIL, browser10/10 и отдельно repaired20/20 при
сохранённом original24/30FAIL. Universal acceptance остаётся NOT_MET.
Source PASS, accepted installed SHA и client acceptance не объединены в общий
PASS; будущий refactor не наследует приёмку
`2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559`.
Новая версия, installation, restart, browser/physical verification или release
этим final plan review не установлены и не выполнены.

## Оценки карточек

Оценка относится к task spec и его честно объявленной готовности, не к качеству
будущего runtime. Deferred/investigation spec может быть принят без допуска кода.

| Task | Score | Принятый scope / ещё обязательный prerequisite |
| --- | ---: | --- |
| 129 | 9/10 | Metadata current map и одна очередь; оба CONTINUE, historical evidence сохранены. |
| 130 | 8/10 | Deferred binding/checked-result spec; named versioned consumer и observed case provenance до кода. Standalone tool не закрывает task. |
| 131 | 9/10 | Малый source refactor duplicate fixed160 invariant; explicit decision/owning entry, affected gates и свой implementation review. |
| 132 | 8/10 | Deferred maintenance proposal; конкретная польза/gap, reuse coverage и сохранение leaf protection до допуска extraction. |
| 133 | 8/10 | Causal investigation; first failed transition неизвестен, physical prerequisite и прежние FAIL не скрыты. Implementation NOT_READY. |
| 134 | 8/10 | Один обоснованный existing-owner slice после bounded136 map; новый child не теряет protection. Implementation NOT_READY. |
| 135 | 8/10 | Reachability/link/package baseline; пользу и backward policy измерить до feature/API change. Implementation NOT_READY. |
| 136 | 8/10 | Bounded semantic gap map перед splitting; real entrypoints/effects, discovery и process isolation сохраняются. |
| 137 | 8/10 | Отдельный measured partial-result debt и investigation/recovery boundary; safe generation policy и failure-injection proof до writes. Implementation NOT_READY. |

## Inspected scope и пределы final verdict

HEAD по-прежнему `e7a25705bb6196776c35c0e3cb7c3b5a17d55169`; tracked diff пуст.
Прочитаны исправленные AUDIT,129–137 и полный pass1 report, including final
уточнение двух CONTINUE. Сопоставлены его F1–F7 с текущими statuses, selected
routes, acceptance и dependencies. Повторно прочитаны только нужные source
slices trim/callback order/u8 offsets, pure predicate/caller, canon guard,
existing Python self-test lists/CLI flags и верхние sections двух CONTINUE.
Relevant production facts/receipts из pass1 переиспользованы, поскольку HEAD
и tracked production source не изменились; нового semantic/physical PASS нет.

Не выполнялись production edits, tests/builds, graph queries/refresh, installs,
restarts, runtime checks, browser/physical input, remote desktop или новый audit/fit.
Единственная запись pass2 — этот review. Live runtime, причины native2FAIL,
current127/128, future connected caller/recovery, linked performance и общий
coverage не измерялись. Они не нужны для принятия ограниченного stage-1 плана
и остаются явно открытыми.

Plan review закрыт: **ACCEPT,9/10, pass2 FINAL**. Следующее допустимое действие —
реализовать TD-129, затем TD-131 с их собственными gates/reviews/commit-push,
сохранив принятые delivery routes и installed runtime. Stage 2 сначала обсуждается.
