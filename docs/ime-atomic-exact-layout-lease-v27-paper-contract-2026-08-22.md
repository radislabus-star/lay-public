# Lay IME Atomic Exact-Layout Lease V27: Paper Contract

> Historical scope note (2026-08-30): V27 remains the sealed `EN -> RU`
> baseline. Its prohibition on `RU -> EN` exact authority is superseded only by
> the separately reviewed V28 contract in
> `docs/ime-atomic-exact-layout-lease-ru-to-en-v28-2026-08-30.md`.

Дата: 2026-08-22
Статус: implemented, proved and deployed as Lay 1.0.34
Цель: убрать head-of-line blocking полного Space-prefetch для точного
EN-to-RU layout-перехода, не создавая второго владельца решения и не выдавая
layout+typo за точную раскладку.

## 1. Граница утверждения

Этот документ может дать только `PAPER_READY`: архитектура непротиворечива,
известные классы отказов перечислены, а реализация и proof заранее определены.
Он не может дать `IMPLEMENTED`, `QUALITY_PASS`, `LATENCY_PASS`, `DEPLOYED` или
разрешение на установку. Эти статусы появляются только после соответствующих
измерений.

`100% paper contract` здесь означает одновременно:

1. у каждого live-перехода есть один producer/evidence/rank/verifier/mutation
   маршрут;
2. все известные положительные, отрицательные, stale и failure-состояния имеют
   однозначный исход;
3. каждое утверждение привязано к отдельному denominator и gate;
4. ни один оценочный сигнал не назван authority;
5. неизвестное или непроверенное состояние всегда закрывается без замены;
6. route gate и implementation preflight проходят без `WATCH` и `VETO`.

## 2. Измеренные факты

V25 physical proof установил:

```text
final ghbdtn correction       0.511 ms
stale ghbdt correction       20.053 ms
Space lookup wait             8.080 ms
Space total                   8.159 ms
Shell RPC timeout             8.000 ms
```

Следствие: окончательный запрос сам по себе быстрый. Дедлайн съедает
неканцелируемая устаревшая работа одного background worker. Увеличение RPC
timeout не исправляет ownership и запрещено.

Первый V26 focused test завершился semantic FAIL:

```text
input:     dnjpfvtyf
raw map:   втозамена
expected:  no exact-layout decision
actual:    Some("втозамена")
```

Отдельный read-only probe показал:

```text
surface       l1 atoms  covered  residual
втозамена            21       21         0
привет               15       15         0
```

Следствие: полное покрытие атомов не доказывает существование точного центра.
Опечатка может состоять только из известных n-грамм. Условие
`l1_refs >= 12 && residual == 0` является candidate evidence, но не exact
surface authority.

На фиксированных локальных источниках найдено ещё одно обязательное
ограничение. Среди 110,644 известных ASCII-слов и 105,673 точных русских
поверхностей есть 423 пары, где известное английское слово после физической
US-to-RU проекции становится точной русской поверхностью. Из них 22 имеют длину
не меньше четырёх. Поэтому одного доказательства целевой поверхности
недостаточно: точная известная исходная ASCII-поверхность обязана блокировать
автозамену.

Источники этого измерения:

```text
911e415c517168f876d766d9de1be71edc76ef0b6f370a69242efff6ab154bc3  data/lexicon/common_ru.txt
b6055c489332e85f903dd8981beeaed8201303a05c24b6300ca415c808d67bc1  data/lexicon/l2_surface_foundation_ru_100k.txt
3130c6e6c8e38a952c4fa46f55cc00cc7b4e71616792ab02bbfc384cd1a9997f  data/lexicon/l2_surface_hot_ru.txt
829a043cf078d1e80e886289a13823454977f442a239a859d2133ea61944aa60  /usr/share/hunspell/en_US.dic
9e66281f7e51445eab6857488ff6e3d768afffadb7fb1adbef5e4617bee4a53b  /usr/share/dict/words
a21acf2a731e7842ea877b599dc54eb9841f3ddfdc50230b4bea68d3e19b1871  data/lexicon/common_en_technical.txt
```

Это измерение является нижней границей collision corpus, а не заявлением о
полном множестве всех возможных пользовательских токенов.

Отдельный standalone RSS probe воспроизвёл текущую загрузку EN guard из
`/usr/share/hunspell/en_US.dic` и `/usr/share/dict/words` без изменения Lay:

```text
loaded entries before user-protected extension       139,370
HashSet capacity                                     229,376
string capacity bytes                              1,193,385 B
steady RSS delta                                      11,796 KiB
process maximum RSS                                   16,180 KiB
wall time                                               0.04 s
```

Число `139,370` шире предыдущего denominator `110,644`, потому что runtime
loader принимает ASCII surfaces с допустимыми token separators, тогда как
collision scan считал более узкий alphabetic source domain. Главное следствие:
IME warmup сейчас не прогревает этот set, а его реальная стоимость не помещается
в прежний новый-component gate `<= 1 MiB`. Пользователь явно принял эту
стоимость 2026-08-22. V27 сохраняет точный существующий `HashSet`, не вводит
probabilistic guard или второй lexical package и меняет incremental RSS gate на
`<= 16 MiB`. Интегрированный startup/RSS proof всё равно обязателен: standalone
probe является оценкой выбранной структуры, а не product PASS.

## 3. Почему V26 отклонён

### 3.1 Смешаны plausibility и authority

V26 свёл шесть неодинаковых источников к `raw_projection_stable: bool`:

- точная словарная поверхность;
- exact terminal L2;
- сгенерированная морфологическая форма;
- reference-backed форма;
- полное n-грамм покрытие;
- hot-field phase/form authority.

Последние четыре могут рождать или поддерживать кандидата, но не доказывают,
что raw projection является точной наблюдавшейся словоформой. Булево
объединение уничтожило provenance и допустило `втозамена`.

### 3.2 Exact fallback вычисляется слишком поздно и слишком рано одновременно

V26 вычисляет fallback синхронно внутри вызова Space до проверки готового full
lease. Это:

- добавляет работу даже когда full lease уже готов;
- оставляет cold-load риск на дедлайне Space;
- не позволяет доказать, что Space только выбирает готовую authority;
- затрудняет раздельное измерение printable preparation и Space consumption.

### 3.3 Контекст ошибочно запрещён

`req.text.split_whitespace().count() == 1` проверяет весь gate text. После
`active_composition_gate_text()` в нём находится контекст фразы, поэтому fast
lane работает для первого слова, но исчезает в `check ghbdtn`. Ограничение
должно относиться к текущему token, а не ко всей фразе.

### 3.4 EN-to-RU и RU-to-EN несимметричны

ASCII branch V26 использует новый exact helper, а Cyrillic branch вызывает
старый общий `correct_wrong_layout_cyrillic_word()`. Значит, один mode обещает
exact-only, но в обратном направлении содержит более широкую эвристику.
V27 не симулирует симметрию: fast lease ограничен EN-to-RU. RU-to-EN остаётся
за полным маршрутом до отдельного доказанного контракта.

### 3.5 Edge punctuation неидентифицируема без контекста

Клавиши `;`, `,`, `.` и другие одновременно являются пунктуацией ASCII и
русскими буквами после layout projection. Например, trailing `.` может быть
реальной точкой либо клавишей `ю`. Fast lane не может надёжно различить это по
одному токену. V27 разрешает внутренние layout-letter symbols (`ye;ty`), но
отказывает токенам с неалфавитным edge symbol. Полный контекстный маршрут может
решать их позже.

### 3.6 Identity и decision material передаются раздельно

V26 передаёт `identity`, `boundary_text`, `config` и готовый fallback отдельными
значениями. Тип не запрещает случайно связать decision одного текста с identity
другого. Постпроверка уменьшает риск, но контракт должен исключить саму
возможность: exact lease строится из одного canonical frame snapshot и хранит
его identity целиком.

### 3.7 Unit parity слишком узкая

Одна пара `ghbdtn -> привет` не доказывает:

- контекст слева;
- known-English collisions;
- layout+typo;
- edge punctuation;
- cold/not-warm путь;
- package generation race;
- full/exact divergence;
- single consumption;
- atomic undo.

V27 заменяет примерный gate фиксированными сгенерированными denominators.

## 4. Выбранная архитектура V27

```text
printable input frame
├── canonical frame builder (один InputFrameIdentity)
├── latest-only full worker
│   -> TransitionDecisionCore::FullField
│      ├── Tied | ABSTAIN                 -> FullTerminal::NoApply
│      └── Winner -> verifier
│           ├── authorized               -> FullTerminal::Apply
│           └── rejected                 -> FullTerminal::NoApply
└── inline bounded exact EN-to-RU producer
    -> warm authority snapshot
    -> exact closed-contour certificate
    -> тот же L2CandidateLattice
    -> TransitionDecisionCore::ClosedExact
    -> тот же verifier
    -> exact prepared lease

Space
-> one linearized decision arbiter
   ├── current FullTerminal::Apply    -> consume full
   ├── current FullTerminal::NoApply  -> no correction (Rank | Verifier | Infrastructure)
   ├── else current exact lease       -> consume exact
   ├── else wait current full <= 4 ms
   └── else no correction; exactly one native Space
-> post-lease identity check
-> one atomic engine output
```

Главное изменение относительно V26: exact decision готовится на printable
frame, а не впервые на Space. Exact preparation выполняется inline после
фиксации printable frame: она не использует full worker, его очередь или
`Condvar`, поэтому stale full work не может занять exact lane. `Nonblocking`
здесь означает отсутствие I/O, package initialization и ожидания lock/worker;
CPU-время остаётся bounded и проверяется отдельным latency gate.

Для большинства префиксов exact terminal отсутствует, и работа заканчивается
после bounded DAFSA lookup. DecisionCore вызывается только после формирования
сертификата замкнутого exact-контура. Full producer обязан публиковать terminal
и для `Winner`, и для `Tied | ABSTAIN | rejected | infrastructure failure`:
любой готовый current `NoApply` имеет приоритет над exact lease так же, как
готовый full apply.

Shared typed exact constructor вызывается не через взаимоисключающий
`L2CandidateSource::for_mode()`. Текущий `NandaOnly` перечисляет только
`L2CandidateSource::Nanda`, поэтому отдельный `ExactLayout` mode не гарантирует
присутствие target в полном поле. V27 вставляет доказанный exact candidate в
one-slot retained segment общего `L2CandidateLattice` до source mode dispatch,
deduplication, bounded frontier и любого top-k. Затем full producer добавляет
обычные Nanda candidates в тот же lattice. Overflow, conflict frame/target или
incomplete evidence дают terminal `NoApply`, а не вытеснение.

Exact lane не строит L2 peak context, не вызывает L3/L4 и не читает mutable
usage/model donors. Closure certificate делает этот operation class локально
полным; общий DecisionCore выполняет тот же typed admission и edit selection в
явном `ClosedExact` mode. Этот mode не выражается как
`prepared_peak_context=None`: в текущем API `None` означает «построить context»
и поэтому является противоположным контрактом. Full route остаётся владельцем
всех случаев, где closure не доказана. Для closed exact evidence действует
алгебраический dominance-инвариант: произвольные finite L2/L3/L4 scores не могут
понизить или заменить target. Контекстный veto допустим только как source guard,
включённый в certificate до рождения кандидата; добавление нового вида veto
требует нового paper contract.

Оба producer могут подготовить material, но ни один не изменяет текст.
Единственный mutation owner остаётся atomic engine output после общего
DecisionCore и verifier.

## 5. Формальный exact certificate

Пусть:

- `t` — текущий observed token из canonical `InputFrameIdentity`;
- `s` — exact active source-layout profile; для V27 допускается только
  fingerprint доказанного US QWERTY profile, а не любое `layout_is_ru=false`;
- `a` — mutable decoder-layout state текущего frame; он обязан быть `Us`, но не
  может сам создать либо заменить immutable profile `s`;
- `P(t, s, m)` — единственная побуквенная US-to-RU проекция по admitted source
  profile `s` и immutable keyboard map `m`, без удаления, вставки,
  перестановки или spelling polish;
- `R(x, r)` — exact terminal membership в уже прогретом immutable Russian
  lexical package fingerprint `r`;
- `E(t, e)` — исходная ASCII-поверхность является exact English terminal в
  уже прогретом точном EN `HashSet` fingerprint `e`;
- `U(t, p)` — исходная поверхность защищена user/technical policy fingerprint
  `p`;
- `case_closed(t)` — регистр является lowercase, TitleCase или UPPERCASE и
  однозначно переносится на target; mixed-case token отвергается;
- `edge_ambiguous(t)` — первый или последний символ токена не ASCII-буква.

`s = UsQwerty` создаётся только factory при точном
`CreateEngine("lay-ime-us")` и совпадении compile-time component mapping
`lay-ime-us -> layout us`. `lay-ime-ru` создаёт профиль `Ru`, любое другое имя
создаёт `Unknown`; выражение `name != "lay-ime-ru"`, mutable
`active_layout_is_ru=false` или sanitized object path не могут создать
`UsQwerty`. Factory profile неизменяем для жизни engine object. Если decoder
временно переключён внутри другого engine до process-level handoff, exact lane
остаётся недоступен до появления admitted `lay-ime-us` object. На printable и
Space путях нет DBus/CLI-запроса профиля.

`ExactAuthoritySnapshot(s, m, r, e, p)` существует только когда source profile
точно admitted, а остальные четыре источника уже прогреты и их fingerprints
можно прочитать без инициализации, I/O или ожидания.
`ExactLayoutContourCertificate(t, snapshot)` существует
тогда и только тогда, когда одновременно истинны все условия:

```text
frame.active_composition                         true
frame.factory_engine_profile                    exact admitted US QWERTY
frame.active_decoder_layout                     US
config.auto_replace                             true
config.auto_switch_layout                       true
token script                                    ASCII layout surface
ASCII letter count                              >= 2
digits / URL / CLI / mixed script               absent
case_closed(t)                                  true
edge_ambiguous(t)                               false
user or technical protection                    false
RU terminal / EN guard / protection sets        already warm
E(lower(t), e)                                  false
U(lower(t), p)                                  false
q = P(t, s, m)                                  q != t
q script                                        Cyrillic
R(lower(q), r)                                  true
secondary typo/morphology/polish operation      absent
admissible raw-layout target count               exactly 1
```

Последнее условие является closure, а не score. Для этого operation class
разрешена ровно одна буквальная поверхность, и все независимые противопоказания
проверены до рождения кандидата. Full route обязан родить тот же typed evidence
и не может стереть его generic uncertainty или unrelated candidate basin.
Явное source protection или exact English membership предотвращает выдачу
сертификата, а не понижает уже выданный сертификат постфактум.

Разрешённый target evidence ровно один: exact terminal уже прогретого Russian
package внутри замкнутого raw-layout contour. Сам сертификат всё ещё не является
mutation authority: решение и edit plan выдают общий DecisionCore и verifier.
Следующие сигналы прямо запрещены как самостоятельный сертификат:

- n-грамм coverage/coherence;
- `is_reference_backed_russian_form`;
- morphology settlement/generation;
- decoder-only surface;
- usage count или online feedback без exact terminal;
- L2/L3/L4 score;
- spelling polish;
- layout-then-typo.

Они остаются допустимым evidence полного маршрута.

## 6. Типовой контракт

Runtime-код не должен снова кодировать политику двумя boolean-параметрами.
Минимальные смысловые типы:

```text
ExactAuthoritySnapshot
  factory_engine_profile = UsQwerty
  component_layout_mapping_fingerprint
  source_layout_profile_fingerprint
  keyboard_map_fingerprint
  russian_terminal_fingerprint
  english_guard_fingerprint
  protection_policy_fingerprint

ExactEnglishGuardSnapshot
  source_file_fingerprints
  normalized_entry_count
  exact_hash_set_ready = true
  resident_bytes_receipt

RawLayoutProjection
  original_token
  projected_token
  direction = UsToRu
  case_shape = Lower | Title | Upper
  authority_snapshot

ExactLayoutContourCertificate
  canonical_frame_ref
  projection
  source_exact_terminal = false
  source_protected = false
  target_exact_terminal = true
  admissible_target_count = 1

CandidateAuthorityEvidence
  None
  ClosedExactLayout(opaque ExactLayoutContourCertificate)

DecisionEvidenceMode
  FullField(prepared peak/context inputs)
  ClosedExact(opaque ExactLayoutContourCertificate)

PreparedCorrectionLease
  frame_identity
  request_generation
  producer_generation = FullWorker(g) | InlineExact(frame_revision)
  decision_environment
  decision
  decision_receipt
  undo_material
  kind = Full | ExactLayout

PreparedFullOutcome
  Apply(PreparedCorrectionLease)
  NoApply(stage = Rank | Verifier | Infrastructure, terminal receipt)

PreparedDecisionSlot
  identity
  full = Pending | Terminal(PreparedFullOutcome)
  exact = Absent | Prepared(PreparedCorrectionLease)
```

Raw projection не является кандидатом с authority. Exact certificate не
является mutation authority. Prepared lease не применяется без повторной
identity-проверки и atomic output transaction.

`ClosedExactLayout` создаётся только закрытым constructor сертификата и
привязан одновременно к canonical frame и replacement bytes. Он не является
новым `CandidateOrigin`, диагностическим `source_id` либо переименованием
`ReplacementTargetEvidence::ExactLayoutProjection`: существующий live-IME тип
не доказывает source guard и contour closure. При dedup/merge кандидатов с одной
replacement evidence объединяется коммутативно и идемпотентно. Сертификат
сохраняется только при byte-equal frame/target; несовпадающий сертификат делает
authority incomplete и даёт fail-closed, а не переносится на другого кандидата.

Lease store и Space arbiter также не являются authority owners. Они хранят,
выбирают и одноразово переносят уже выданный verifier receipt. В authority-графе
разрешение идёт прямо `transition verifier -> atomic engine output`; store и
arbiter существуют только в execution/state графе.

Store bounded: ровно один process-wide slot текущего active frame, ключом
которого являются engine path и focus receipt. Нет map «по slot на каждый когда-
либо виденный context», очереди exact jobs или накопления старых leases. Новый
frame/focus атомарно заменяет slot и инвалидирует authority предыдущего request
generation. `PreparedFullOutcome`
обязан хранить `NoApply`; отсутствие apply lease не может означать, что full
route ещё не закончил или что его semantic abstention можно проигнорировать.

`CorrectLayoutPolicy(bool, bool)` или аналогичный набор флагов запрещён.
Recovery и exact certification должны быть разными функциями поверх общего
чистого keyboard projection primitive.

## 7. Canonical frame contract

`boundary_text` не передаётся как независимая строка. Один builder выводит из
frame identity:

```text
context_prefix + observed_token + pending Space
```

Он обязан доказать:

- `committed_tail` оканчивается на `observed_token`;
- `context_prefix` является точным неизменяемым префиксом;
- raw token span хранится как точные UTF-8 byte bounds и Unicode-scalar delete
  count; normalized/lowercased word не может подменить редактируемую поверхность;
- replacement меняет ровно текущий token и добавляет ровно один Space;
- focus receipt, engine path, tail epoch, exact active-layout profile
  fingerprint, отдельный mutable decoder-layout state и correction-affecting
  config входят в identity;
- factory engine profile и component name/layout mapping входят в profile
  fingerprint; mutable layout boolean не может их заменить;
- printable frame revision/event sequence, active-composition state и
  output-profile/capability epoch входят в identity;
- exact authority snapshot содержит отдельные fingerprints source-layout
  profile, keyboard map, Russian terminal package, English guard set и
  protection policy;
- full material generation и exact authority snapshot не подменяют друг друга;
- full и exact producer получают один и тот же frame snapshot.

Фраза может содержать любое количество предыдущих слов. Single-token scope
означает один изменяемый хвостовой token, а не одно слово во всём gate text.
Case transfer является частью projection: lowercase, TitleCase и UPPERCASE
сохраняются детерминированно, mixed-case fast route не получает certificate.

## 8. Lease state machine

```text
Idle
  -> PrintableCommitted(identity, request_generation)
       ├── schedule latest-only FullPending
       └── inline exact evaluation
            -> ExactAbsent | ExactPrepared

FullPending
  -> FullTerminal::Apply
  -> FullTerminal::NoApply(stage = Rank | Verifier | Infrastructure)

Space(identity)
  -> identity mismatch                         Stale / no apply
  -> current FullTerminal::Apply               consume Full once
  -> current FullTerminal::NoApply             no apply
  -> current ExactPrepared while FullPending   consume Exact once
  -> ExactAbsent while FullPending             wait current Full <= 4 ms
       -> FullTerminal::Apply                  consume Full once
       -> FullTerminal::NoApply                no apply
       -> timeout                              no apply

Any consume/timeout/invalidate
  -> retire authority generation
  -> stale full work may finish as material-only
  -> stale work may not publish or mutate
```

Правила гонок:

1. Full terminal, уже готовый к linearization point Space arbitration, имеет
   приоритет независимо от того, содержит он `Apply` или `NoApply` любой стадии:
   `Rank`, `Verifier` либо `Infrastructure`.
2. Проверка full state, выбор exact и retirement request generation выполняются
   под одной synchronization boundary. TOCTOU между probe и exact consume нет:
   full либо публикуется раньше и выигрывает, либо позже и уже не может вернуть
   authority.
3. Exact lease вычисляется inline на printable frame, никогда не ждёт stale
   worker, не имеет собственной очереди и не создаётся на Space. Publication
   использует bounded `try_lock`/эквивалент; contention даёт `ExactAbsent`, а не
   ожидание на printable route.
4. Focus, tail, event revision, active composition, layout, config,
   output-profile/capability, full material или любой exact fingerprint mismatch
   дают no apply.
5. Следующий printable, Backspace, navigation, punctuation boundary, preedit
   reset/commit, Tab completion, Enter, manual/double-Shift action,
   focus/layout-profile/config change или process restart инвалидируют
   предыдущий slot до возможного consume.
6. Lease одноразовый; повторный take возвращает no correction. Lock poisoning,
   missing worker и not-warm authority snapshot fail closed.
7. После timeout stale full work может заполнить только bounded material cache;
   оно не получает apply authority и не создаёт новый lease старого frame.
8. Каждый no-apply terminal возвращает управление обычному input route, который
   обязан доставить ровно один Space. Exact/full apply включает ровно один Space
   в единственную atomic transaction; второй replay запрещён.

## 9. Warmup contract

Fast lookup использует только `OnceLock::get`-эквиваленты для Russian terminal
package, English guard set и protection policy. Он не вызывает `get_or_init`,
mmap, file open, Hunspell/plain-dictionary parsing, package parsing или
`prefetch_all` на printable или Space route.
Source-layout profile читается из immutable factory-owned engine state, а не
через `ibus engine`, GNOME DBus или shell command на hot path.

```text
all authority sets warm        exact lookup разрешён
any set absent/loading         exact lookup = Unavailable, no block
```

Startup warmup остаётся отдельным background lifecycle. Его готовность не
может быть подделана первым пользовательским вводом. Warm registry публикует
fingerprints только после полной загрузки всех компонентов; частично прогретый
snapshot не существует. V27 не добавляет новый resident lexical package и
использует уже установленный immutable RU package плюс существующий exact EN
loader и protection guard sets. Для IME это новый обязательный warm asset:
текущий `warm_up_l2_for_ime()` его не прогревает. Startup lifecycle V27 обязан
явно вызвать EN/protection warmup в background и только после полной загрузки
атомарно опубликовать fingerprinted snapshot. Первый пользовательский token не
может вызвать `get_or_init`. EN guard остаётся точным `HashSet`, а не
probabilistic approximation. Ошибка чтения, fingerprint mismatch или неполная
user-protection extension оставляет snapshot отсутствующим и отключает exact
lease; full route продолжает работу.

Process ownership закрыт отдельно. Новый V27 exact-authority snapshot и новая
resident-копия EN guard принадлежат только процессу `lay-ibus-engine`.
`lay-daemon` может сохранять свой уже существующий generic word-recognizer
route, но не публикует, не потребляет и не прогревает V27 exact authority.
Exact warmup нельзя добавлять в общий `warm_up_l2_for_ime()`, вызываемый обоими
process owners; нужен IBus-engine-scoped startup entrypoint. Production source
route обязан иметь ровно один такой callsite. Повторная инициализация во втором
Lay-процессе является resource FAIL, даже если каждый процесс отдельно
укладывается в локальный RSS gate.

## 10. Decision and mutation ownership

Допустима только одна authority-цепочка:

```text
typed producer
-> typed evidence
-> L2CandidateLattice
-> TransitionDecisionCore
-> transition verifier
-> prepared lease
-> identity revalidation
-> atomic engine output
```

Full route и inline exact route обязаны использовать один constructor typed
exact candidate. Full lattice не может заменить его другим source id, снять
closure bit или удалить generic uncertainty. Если full route заканчивается
`Tied | ABSTAIN | rejected` для frame, на котором inline route выдал Apply,
это системная divergence и promotion немедленно отклоняется.

Dominance определяется до mutable score, но после source guards, и покрывает не
только score vectors. Exhaustive production match обязан явно классифицировать
каждый `LiveCandidateLane`, `CandidateOrigin`, `TypingErrorClass`,
`CandidateGateAction`, `TransitionOperator`, `TransitionProof`,
`LanguageActionProof`, `TransitionOperatorKind` и legacy
`TypingCandidateFamily`, а также `CorrectionDecisionSource`,
`CorrectionSourceRole`, `CandidateReadoutRoute`, `ReplacementTargetEvidence`,
`LanguageActionOperator`, `EnumerationStateV1`, `CompletenessScopeKindV1` и
`IncompletenessReasonV1`. Будущие `CandidateAuthorityEvidence` и
`DecisionEvidenceMode` входят в тот же exhaustive match с момента появления.
Wildcard arm запрещён: новый enum variant без решения ломает компиляцию.
`Overflow | Failed`, incomplete evidence и узкая completeness scope без
валидного partition proof дают `NoApply`. Same-surface merge, producer order и
evidence aliasing входят в тот же property proof.

Atomic output создаёт тот же pending auto-undo receipt и тот же outcome-feedback
record для Full и ExactLayout. Подготовка lease, его timeout или automatic apply
сами по себе не являются положительным learning evidence. Double Shift обязан
восстановить exact original token и зарегистрировать rejection по существующему
общему контракту.

Запрещено:

- direct commit из exact producer;
- отдельный fast ranker;
- отдельный fast verifier;
- повторный deterministic decision после DecisionCore;
- legacy key replay после atomic apply;
- fallback exact authority для RU-to-EN;
- mutation retry после consumed lease;
- positive learning из prepared/expired lease или из apply без admitted outcome;
- отдельный undo stack для exact route;
- увеличение Shell RPC timeout.

## 11. Proof denominators

### 11.1 Exact authority corpus

Корпус генерируется из pinned exact lexical surfaces, а не из runtime literal
branches.

До запуска quality proof генератор обязан один раз записать immutable manifest
с полными SHA-256 всех шести исходных файлов, версией normalization/projection,
точным ненулевым числом eligible cases и числом случаев каждого negative class.
Proof runner сначала сверяет этот manifest и только затем считает проценты.
Изменение входа, алгоритма генерации, числа случаев или denominator manifest
делает предыдущий receipt неприменимым; пустой denominator является hard FAIL.
Corpus oracle реализуется отдельно от runtime certificate/candidate helpers:
он использует собственную pinned US-QWERTY таблицу и lexical-file parser,
записывает hash своего source/binary в manifest и не импортирует runtime
projection, rank или guard code. Иначе одна ошибка могла бы одинаково исказить
expected и actual и дать ложный parity PASS.

Oracle компилируется отдельной командой из standalone source без зависимости от
crate `lay`; manifest фиксирует source SHA-256, binary SHA-256, `rustc -vV`,
версию normalization, собственный keyboard-table fingerprint и точные counts
каждого класса. Fault proof односторонне меняет одну пару keyboard map сначала
в runtime adapter, затем в oracle adapter: в обоих случаях parity runner обязан
обнаружить divergence. Самосравнение или общий generated expected file является
hard FAIL.

```text
eligible EN-to-RU exact projections
target certificate coverage                         100.000%
closed-contour coverage                             100.000%
same full/exact replacement                         100.000%
same allow_apply                                    100.000%
same transition proof                              100.000%
same changed-token set and undo kind                100.000%
full terminal disagreement                           0
arbitrary bounded L2/L3/L4 score perturbation         0 target changes
all typed candidate classes / producer permutations   0 target changes
same-surface evidence merge permutations              0 target changes
left-context byte preservation                      100.000%
```

### 11.2 Negative corpus

```text
known-English -> exact-Russian collisions           0 exact applies
EN exact guard false negatives                       0
EN/protection source fingerprint mismatch            0 exact applies
layout+typo                                          0 exact applies
unknown/non-terminal target                         0 exact applies
generated-only morphology target                    0 exact applies
URLs/CLI/digits/mixed/protected                      0 exact applies
mixed-case source                                    0 exact applies
edge-symbol ambiguity                               0 exact applies
active-layout mismatch                              0 exact applies
non-US / unknown Latin layout profile                0 exact applies
unknown engine name with active_layout_is_ru=false   0 exact applies
RU engine with active_layout_is_ru=false             0 exact applies
RU-to-EN                                            0 exact applies
```

Минимум 423 измеренных known-English collisions входит в фиксированный
denominator. Изменение корпуса изменяет manifest и не может молча изменить
знаменатель.

### 11.3 Race/fault matrix

```text
focus change                                         0 stale applies
tail epoch change                                    0 stale applies
subsequent printable / Backspace / navigation        0 stale applies
preedit reset or active-composition change            0 stale applies
layout change                                        0 stale applies
factory engine profile / decoder-state disagreement  0 stale applies
source-layout profile fingerprint change             0 stale applies
config change                                        0 stale applies
output profile/capability change                     0 stale applies
material generation change                           0 stale applies
worker generation change                             0 stale applies
RU terminal fingerprint change                       0 stale applies
EN guard fingerprint change                          0 stale applies
protection fingerprint change                        0 stale applies
package not warm                                     0 blocks / 0 applies
EN guard absent/loading/partial                       0 blocks / 0 applies
poisoned state lock                                  0 blocks / 0 applies
double take                                          1 apply maximum
atomic output failure                                0 legacy retries
partial slot publication                             0 visible partial states
cross-focus A/B replace/publish/consume              0 stale applies / 1 slot max
```

### 11.4 Context matrix

Обязательны первый token, русский левый контекст, ASCII левый контекст,
несколько предыдущих слов и punctuation before token. Во всех случаях меняется
только последний observed token.

### 11.5 Physical product proof

```text
ghbdtn -> привет -> double Shift -> ghbdtn           PASS required
legacy mutation calls                                0
paired printable releases                            514 / 514
release leftovers                                    0 / 0
proof-owned process leftovers                        0
```

Fixture `ghbdtn` доказывает общий контракт и никогда не становится runtime
условием.

### 11.6 Space, undo and learning matrix

```text
full NoApply / exact absent                           1 native Space exactly
timeout / stale / not-warm / poisoned                 1 native Space exactly
full or exact Apply                                   1 atomic trailing Space exactly
atomic refusal/failure                                1 native Space / 0 partial edit
lost or duplicated Space                              0
word-boundary glue caused by V27                      0
exact apply -> double Shift -> exact original         100.000%
prepared/expired lease -> positive learning           0
undo rejection recorded through common route          100.000%
```

No-apply проверяется как полный product effect, а не как отсутствие correction
call. Если Space потерян, продублирован или соседние слова склеились, V27 FAIL
даже при идеальной lexical parity. Atomic refusal использует уже доказанный
backend refusal/native route; legacy key replay из exact producer запрещён.

## 12. Latency contract

Все latency denominators измеряются отдельно:

```text
printable exact miss added p99                       <= 0.250 ms
printable exact hit preparation p99                  <= 2.000 ms
Space exact lease lookup p99                         <= 1.000 ms
Space no-exact full wait                              <= 4.000 ms
complete 514-event RPC p99                           <= 5.000 ms
complete 514-event RPC max                           <  8.000 ms
exact producer queue depth                            0
prepared decision slots process-wide                  1
new lexical package bytes                             0
new V27 exact EN guard process owners                  1
lay-ibus-engine EN guard steady-state RSS delta       <= 14 MiB
lay-daemon V27 exact-guard incremental PSS              0 MiB
aggregate active-Lay V27 steady-state PSS delta       <= 16 MiB
hot-route file opens / mmap / parsing / prefetch       0
new synchronous trace/log flushes on hot route         0
```

Partial latency не является PASS. Cold package load, process startup и warm
steady-state публикуются отдельными числами. Если exact preparation нарушает
printable gate, V27 отклоняется; работа не переносится обратно на Space без
нового бумажного контракта.

Latency измеряется при concurrently busy stale full worker: иначе proof не
проверяет исходный head-of-line дефект. Engine RSS delta включает EN guard,
потому что для текущего IME lifecycle он не является обязательным warm asset;
aggregate PSS delta суммируется по всем активным Lay-процессам и поэтому ловит
случайную вторую private-копию в daemon. В delta также входят V27 slots,
receipts и instrumentation. Startup EN warmup wall time, transient peak RSS,
loaded entry count и число process owners публикуются отдельно.
Каждая из четырёх latency-строк имеет собственный test ID и denominator;
overall `514-event` PASS не заменяет отдельные printable miss/hit, Space lookup
и full-wait gates.

## 13. Критика выбранного V27

### Возражение A: это всё ещё второй producer

Да, producer два, потому что material разной полноты. Но rank owner, verifier и
mutation owner по одному. Exact producer не умеет выбирать или применять.
Route gate обязан доказать эту границу.

### Возражение B: exact word не гарантирует намерение пользователя

Верно. Поэтому source exact English terminal, protected source и все известные
collisions дают abstain. Остаточный риск неизвестного identifier, который
случайно проецируется в точное русское слово, невозможно устранить из одного
token без контекста. Контракт не скрывает это: false-authority gate сравнивает
fast result с полным владельцем на фиксированном корпусе, а технические и
ambiguous формы fail closed. Универсальная гарантия намерения математически
невозможна.

### Возражение C: подготовка на каждой букве может ухудшить typing latency

Да. Поэтому terminal miss обязан завершаться до DecisionCore и имеет отдельный
`<= 0.250 ms` gate на полном 514-event маршруте. Нарушение отклоняет V27.

### Возражение D: exact package может быть не прогрет

Тогда fast lease отсутствует. Загрузка на hot path запрещена. Пользовательский
Space сохраняется, а полный worker продолжает bounded работу.

### Возражение E: отказ от RU-to-EN неполон

Это осознанное сужение claim boundary. V27 решает доказанный EN-to-RU physical
blocker для `1.0.34`. Симметричный маршрут потребует отдельного exact-English
target contract и не может быть спрятан в эту версию.

### Возражение F: полный route может позже не согласиться с exact

Тогда это false authority, и promotion запрещён. Full/exact parity измеряется
на фиксированном корпусе; runtime trace отдельно считает позднюю divergence.
Ноль divergence является gate, а не наблюдательной метрикой.

### Возражение G: 423 collision не покрывают пользовательские слова

Верно. Это pinned reproducible lower bound. User-protected ASCII words
проверяются отдельным динамическим denominator. Неподтверждённые пользовательские
идентификаторы остаются residual risk и не маскируются словом `safe`.

### Paper critique iteration 1: authority stage reversal

Первый design route gate дал `VETO`: draft ошибочно провёл authority через
`prepared-lease-store` и `space-lease-arbiter`, а затем позволил arbiter
`authorizes` mutation owner. Это делало оркестратор скрытым вторым владельцем
разрешения. Исправление разделяет графы:

```text
authority: verifier -> atomic output
execution: verifier -> lease store; Space -> arbiter -> atomic output
```

Отрицательный receipt сохранён в
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/route-design-draft-v1-veto.json`.
Он не является основанием для реализации и не перезаписывается последующим
PASS.

### Paper critique iteration 2: stale role requirement

Второй route draft сохранил `orchestrator` в `required_roles` authority-route
после того, как orchestrator был правильно удалён из authority-графа. Gate
снова дал `VETO` как противоречивой спецификации. `Orchestrator` обязателен в
execution/state route, но не в логической цепочке выдачи разрешения. Второй
отрицательный receipt сохранён в
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/route-design-draft-v2-veto.json`.

### Paper critique iteration 3: missing no-apply route

Третий draft формально прошёл route gate, но ручная враждебная проверка нашла
неполноту claim: граф показывал только `Space -> atomic output` при наличии
lease и не показывал штатный fail-closed исход. В design contract добавлен
отдельный execution route:

```text
Space -> lease arbiter -> no-correction result
```

Этот terminal не является mutation owner. Он означает, что Lay не применяет
замену, а обычная обработка Space сохраняется внешним input route. PASS draft
до этого дополнения сохранён как промежуточный receipt, но не используется как
финальная бумажная authority.

### Paper critique iteration 4: exact target без candidate closure

V4 route PASS всё ещё позволял прогнать единственный exact candidate через тот
же класс `L2CandidateLattice`, но не доказывал, что урезанный lattice эквивалентен
полному. Одинаковый ranker над разными candidate sets может дать разные ответы.
Исправление: exact evidence теперь является
`ExactLayoutContourCertificate`, который доказывает единственный допустимый
raw-layout target и отсутствие всех независимых противопоказаний. Full route
обязан рождать тот же typed candidate, а generated full/exact proof требует
нулевую terminal divergence. Exact terminal membership без closure недостаточен.

### Paper critique iteration 5: готовый full ABSTAIN был невидим

V4 моделировал только готовый full apply lease. Если full route уже закончил
`Tied | ABSTAIN | rejected`, в store не было lease и exact route мог ошибочно
считать full ещё pending. Исправление: full producer всегда публикует
`PreparedFullOutcome::Apply | NoApply`. Любой current full terminal имеет
приоритет; semantic NoApply приводит к native Space, а не к exact fallback.

### Paper critique iteration 6: одна generation смешивала четыре authority

`lexical_generation` не различала keyboard map, RU exact terminal package, EN
source guard и user/technical protection. Изменение одного источника могло
оставить формально current lease со stale отрицательным evidence. Исправление:
`ExactAuthoritySnapshot` содержит четыре независимых fingerprints; mismatch
любого из них инвалидирует certificate и prepared lease.

### Paper critique iteration 7: слово nonblocking скрывало scheduling

V4 не определял, выполняется exact producer inline или в отдельной очереди.
Очередь могла повторить исходный head-of-line defect. V5 фиксирует один вариант:
exact evaluation выполняется inline после printable commit, не использует full
worker/queue/Condvar и имеет queue depth zero. Space выполняет только bounded
state arbitration. Проверка full terminal и retirement exact generation имеют
одну linearization point.

### Paper critique iteration 8: отсутствовал полный пользовательский effect

Отсутствие correction call не доказывает, что Space доставлен. Также отдельный
fast path мог потерять общий auto-undo или создать ложное learning evidence.
Исправление: proof включает native/apply Space cardinality, word-boundary glue,
double-Shift round trip и feedback parity. Lost/duplicate Space, отдельный undo
stack или learning из prepared/expired lease являются hard FAIL.

После этих исправлений design route V2 прошёл как `route-design-v5.json`:
`PASS`, issues `0`, warnings `0`. Implementation preflight V2 также формально
вернул `READY_TO_IMPLEMENT`, но следующая ручная итерация отклонила оба как
недостаточно точные. Они сохраняются как положительные, но superseded receipts.

### Paper critique iteration 9: NoApply ошибочно требовал authorization

V2 route проводил единый `FullTerminal` через transition verifier. Для
`Tied | ABSTAIN` нет edit plan и нет мутации, следовательно authorization owner
не должен участвовать. Иначе реализация вынуждена создавать фиктивное
«разрешение на отсутствие действия». Final route V3 разделяет:

```text
Winner -> verifier -> Apply
Winner -> verifier rejection -> NoApply(stage=Verifier)
Tied | ABSTAIN -> NoApply(stage=Rank)
```

Все outcomes попадают в один prepared decision store, но только Apply имеет
authority edge к atomic output.

### Paper critique iteration 10: inline lane мог снова втянуть mutable field

Одинаковый `TransitionDecisionCore` сам по себе не запрещает exact-only вызову
строить L2 peak context, читать L3/L4/usage state или ждать общий cache lock.
Это вернуло бы latency и identity зависимости, которые closure должен удалить.
Исправление: inline exact lane использует общий typed admission/readout с
`peak_context=None`, не рождает mutable donors и публикует через bounded
`try_lock`. Raw token span и case shape входят в frame/certificate, поэтому
lowercased lexical key не может подменить фактический edit range.

Final design route V3 записан в
`docs/structural_gates/preflights/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_ROUTE_V27_V3_2026-08-22.json`.
Receipt `route-design-v6.json`: `PASS`, issues `0`, warnings `0`. Этот PASS
по-прежнему является только design coherence; final implementation preflight
должен быть пересобран против V3.

### Paper critique iteration 11: finite context corpus не доказывал dominance

Даже большой fixed corpus не перебирает все возможные L2/L3/L4 score vectors.
Если mutable context способен понизить closed exact candidate, late full
divergence останется возможной вне выборки. Исправление является property, а не
новым коэффициентом: для typed closed contour DecisionCore обязан сохранять
target при любом bounded наборе unrelated scores. Новый property proof
генерирует competing lattices и score perturbations; любое изменение target
является false authority и hard FAIL.

### Paper critique iteration 12: `not RU` не означает `US QWERTY`

Boolean `active_layout_is_ru=false` смешивал US QWERTY с German, Dvorak и любым
неизвестным Latin profile. Применять US-to-RU projection по такому evidence
нельзя. Исправление: certificate требует exact admitted source-layout profile
fingerprint; unknown/non-US profile даёт `ExactAbsent`. Profile fingerprint
входит в frame identity и race matrix.

### Paper critique iteration 13: bounded per-context всё ещё мог быть unbounded

Фраза «один slot на context» допускала рост map вместе с числом когда-либо
виденных engine contexts. V27 использует ровно один process-wide active-frame
slot, keyed by engine path/focus receipt. Новый focus заменяет старый slot;
cross-focus publication проверяется fault matrix. Это сохраняет constant memory
и совпадает с фактом одного активного keyboard focus.

### Paper critique iteration 14: Infrastructure NoApply отсутствовал в route graph

Тип `PreparedFullOutcome` уже содержал `NoApply(stage=Infrastructure)`, но V3
route graph показывал только rank- и verifier-ветви. Поэтому формальный PASS не
доказывал доставку terminal receipt при отказе full producer. V4 добавляет
отдельный execution route
`printable -> frame -> full producer -> prepared store` без rank, verifier и
authority edge. Любой current infrastructure terminal подавляет exact и ведёт
к одному native Space. Route V4 находится в
`docs/structural_gates/preflights/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_ROUTE_V27_V4_2026-08-22.json`;
receipt `route-design-v7.json` имеет `PASS`, issues `0`, warnings `0`.

### Paper critique iteration 15: aggregate latency скрывал два Space budget

Implementation preflight перечислял printable miss/hit и общий 514-event gate,
но не имел отдельных тестов для `Space exact lookup <= 1 ms` и ожидания current
full `<= 4 ms`. Общий p99 мог пройти при нарушении одного внутреннего дедлайна.
Final preflight требует два независимых test ID и запрещает выдавать partial
latency за PASS.

### Paper critique iteration 16: fixed corpus не был byte-pinned

Текст называл denominator фиксированным, но preflight не привязывал шесть
исходных lexical files полными hashes и не требовал ненулевой immutable output
manifest. Это позволяло незаметно изменить выборку и сохранить `100%`.
Final preflight pin-ит исходные bytes и требует corpus-manifest freeze до
quality run; изменение count/hash создаёт новый proof generation.

### Paper critique iteration 17: profile fingerprint не имел доказанного producer

V5 требовал `UsQwerty` fingerprint, но фактический factory передавал в engine
только `name == "lay-ime-ru"`; следовательно, любое неизвестное имя становилось
логическим US. Простое переименование boolean не исправило бы authority.
Исправление: factory создаёт закрытый enum `UsQwerty | Ru | Unknown` из exact
engine name и compile-time component mapping. Только `lay-ime-us -> us` может
создать `UsQwerty`; mutable decoder state, object path и отрицание RU не могут.
Factory/xml parity и unknown-name matrix являются отдельными gates.

### Paper critique iteration 18: generator мог разделить ошибку с runtime

Byte-pinned input corpus не гарантирует независимый expected result, если
generator импортирует тот же projection/certificate helper, который проверяет.
V6 требует отдельный oracle implementation, собственную pinned keyboard table,
source/binary hash и запрет runtime imports. Runtime и oracle сравниваются как
два независимых producer; совпадение одного кода с самим собой не считается
quality proof.

### Paper critique iteration 19: score dominance не покрывал другие operation classes

Full lattice содержит не только числовые L2/L3/L4 perturbations, но и typed
layout+typo, lexical, morphology, boundary и general candidates. Closed exact
может быть объявлен dominant только если его precedence задан отдельным typed
классом до mutable score, а property proof перечисляет все текущие candidate
lanes, origins, error classes и transition proofs. Exhaustive guard обязан
перестать компилироваться при добавлении нового варианта без явного решения.

### Paper critique iteration 20: design graph не доказывал post-edit source route

`route-design-v7.json` имеет `source_evidence_verified=false`, поэтому его PASS
доказывает только бумажную топологию. После реализации обязателен отдельный
observed-source route contract с точными callsites и теми же cardinalities:
два producer, один typed constructor, один rank owner, один verifier, один
mutation owner и все три NoApply stages. Без его PASS staging запрещён.

### Paper critique iteration 21: same-surface merge мог стереть certificate

`L2CandidateLattice` deduplicate-ит по replacement и вызывает
`UnifiedCorrectionCandidate::merge_evidence()`. Если closure хранить в
`origin`, `source_id` или одном promoted owner, кандидат другого producer с той
же поверхностью может стереть сертификат либо ошибочно унаследовать его.
Исправление: opaque `CandidateAuthorityEvidence::ClosedExactLayout` привязан к
frame и target, а merge имеет проверяемый commutative/idempotent law. Любой
conflict или incomplete evidence даёт NoApply; порядок producer не влияет на
terminal disposition.

### Paper critique iteration 22: `peak_context=None` запускал тяжёлое поле

Формулировка V5 противоречила текущему source: в
`TransitionDecisionCore::evaluate_candidates()` значение `None` вызывает
`prepare_correction_peak_context()`, затем L3, usage и L4. Поэтому запрет нельзя
доказать соглашением о параметре. V6 вводит явный закрытый
`DecisionEvidenceMode::ClosedExact`; его dependency surface не содержит mutable
field inputs. Оба mode принадлежат одному `TransitionDecisionCore` и используют
один typed precedence/verifier contract, но только `FullField` может читать
L2/L3/L4/usage.

### Paper critique iteration 23: immutable profile и mutable layout были смешаны

Factory profile доказывает, какой component создал engine, а mutable
`active_layout_is_ru` описывает текущее decoder state. Ни один не заменяет
другой. Certificate требует одновременно `factory_profile=UsQwerty` и
`active_decoder_layout=Us`. `lay-ime-ru` либо `Unknown` остаются закрытыми даже
после mutable переключения в false; `lay-ime-us` не сертифицирует frame после
mutable переключения в RU. Оба значения и их generations входят в identity и
race matrix.

### Paper critique iteration 24: V6 preflight не связал три proof obligations

Первый implementation preflight после содержательной V6-критики завершился
`BLOCKED_BEFORE_CODE`, не затронув source. Он нашёл три manifest-level gap:

- `shared_oracle_runtime_helper` не имел source-scan;
- `unobserved_source_route_staging` не имел source-scan;
- runtime differential oracle был привязан к `fault_injection`, хотя comparison
  contract требует отдельный `unit | integration | parity` test.

V6 manifest и receipt сохраняются неизменными. V7 добавляет оба fail-closed
source-scan и разделяет нормальную oracle/runtime parity от одностороннего
mutation fault. Архитектура, denominators и runtime authority не меняются.

### Paper critique iteration 25: V7 не удерживал exact candidate в full lattice

Текущий `L2CandidateSource::for_mode()` использует взаимоисключающие arrays:
`NandaOnly` содержит только `Nanda`, а `ExactLayout` живёт в отдельном mode.
Поэтому promise «full route рождает тот же candidate» не следовал из change set.
V7 вручную отклонён. V8 требует one-slot retained segment до mode dispatch и
всех bounded frontier/top-k; full и closed readout потребляют один объект
target-bound evidence.

### Paper critique iteration 26: V7 скрывал фактическую цену EN guard

`warm_up_live_candidate_readout()` прогревает L2/L3, но не вызывает
`word_recognizer::warm_up()`. Standalone probe текущего `HashSet<String>` измерил
`11,796 KiB` steady RSS delta ещё до user-protected extension. Прежние обещания
«already warm» и `<= 1 MiB` были неверны для IME. Пользователь принял эту
RAM-стоимость. V8 делает exact EN set явной startup dependency и устанавливает
gates `EN <= 14 MiB`, total V27 `<= 16 MiB`; новый compact index не вводится.

### Paper critique iteration 27: V7 exhaustive taxonomy была неполной

Typed dominance пропускал decision source/role/readout route, legacy replacement
evidence и состояния полноты target evidence. Новый source либо `Overflow` мог
получить неявное поведение. V8 расширяет compile-fail taxonomy на все enums из
раздела 10 и требует `NoApply` для incomplete, overflow, failed или unproved
partition state. Это условие существования certificate до precedence, а не
новый post-ranking veto.

### Paper critique iteration 28: V8 смешал identity parity и RSS integration

Implementation preflight V8 завершился `BLOCKED_BEFORE_CODE` с одним blocker:
`exact-en-guard-startup-snapshot` ссылался на integration resource test, тогда
как каждый identity contract обязан иметь отдельный parity test. Это не дефект
runtime architecture и не повод менять EN representation. V8 manifest/receipt
сохраняются неизменными. V9 разделяет byte/fingerprint/completeness parity
snapshot от startup timing/RSS integration.

### Paper critique iteration 29: V9 не закрыл process ownership EN guard

Implementation preflight V9 формально дал `READY_TO_IMPLEMENT` (`85`
baselines, `38` invariants, `53` tests), но post-PASS атака нашла resource hole.
Общий warmup API мог быть вызван и IBus engine, и daemon, создав две private
копии принятого `HashSet<String>`. Live snapshot 2026-08-22 показал raw RSS
около `539 MiB`, но более честный aggregate PSS около `406 MiB`; одинаковые
mmap-страницы L2 входят в RSS нескольких процессов. V10 вводит единственного
нового owner `lay-ibus-engine`, запрещает daemon exact-authority warmup,
требует один production callsite, engine RSS delta `<= 14 MiB`, daemon exact
delta `0 MiB` и aggregate active-Lay PSS delta `<= 16 MiB`. V9 manifest и
receipt сохраняются как положительный mechanical, но вручную отклонённый
baseline. Код и runtime не менялись.

### Paper gate V10: single-process resource contract закрыт

V10 implementation preflight завершился `READY_TO_IMPLEMENT`,
`safe_to_implement=true`, blockers `0`: `90` baseline checks, `18` source
checks, `21` preserved artifacts, `12` identity contracts, `39` invariants и
`54` tests. Post-PASS атака повторно проверила candidate retention, full/exact
linearization, Space cardinality, common undo/feedback и cross-process memory
ownership; нового architecture blocker не найдено.

Что проверено: целостность текущих baseline bytes, полнота failure transitions,
identity-to-parity bindings, единственный IBus process owner на бумажном source
route и отдельные RSS/PSS gates. Что не проверено: Rust implementation,
observed callsites, integrated startup allocation, quality, latency, race,
physical input и deployment. Resource proof обязан отдельно публиковать
controlled per-process `Pss_Anon` delta и aggregate PSS delta; один показатель
не заменяет другой. Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/implementation-preflight-v10.json`.
Runtime authority changed: **no**. Paper verdict: **V27_V10_PAPER_READY**.

## 14. Implementation boundary

Разрешённый будущий change set:

```text
lexical_phase runtime     nonblocking exact-terminal-if-warm readout
word recognizer/lexicon   exact EN HashSet startup warmup + protection snapshot
layout autoswitch         pure projection + typed closed-contour certificate
IBus factory / XML        admitted engine profile; unknown names fail closed
engine frame identity     request revision + authority fingerprints
layout profile adapter    exact admitted US source-profile fingerprint
correction core           target-bound authority evidence + merge law
candidate taxonomy        exhaustive closed-exact precedence without wildcard
DecisionCore              explicit FullField | ClosedExact evidence mode
IME correction            canonical frame-bound exact request
Space prefetch            inline exact; Full Apply/NoApply; linearized arbiter
committed tail            common apply, native Space, undo and feedback parity
IBus startup warmup       sole process owner; publish complete snapshot only when ready
proof scripts             standalone oracle + observed route + quality/race/Space/undo/latency
architecture document     measured V27 result
```

Не разрешены SafetyGate weakening, verifier bypass, model coefficient changes,
new mutation owner, word-specific runtime conditions, local heavy build или
deployment до полного proof. Также не разрешены новый exact worker, exact job
queue, direct source-dictionary I/O на hot path или отсутствие full NoApply в
prepared state. Inline exact route не может строить L2 peak context, вызывать
L3/L4, читать mutable usage donors или синхронно flush-ить trace.
`layout_is_ru=false` не является достаточным source-profile evidence.

## 15. Promotion ladder

```text
P0 paper route gate                         required
P1 implementation preflight                 required
P2 remote compile + focused semantic tests  required
P2a observed-source route parity            required
P2b independent oracle freeze + mutation    required
P3 generated exact/negative corpus           required
P4 race/fault matrix                         required
P5 Space composite                           required
P6 double-Shift physical undo                required
P7 fixed 514-event latency                   required
P8 docs + graphify + deployment preflight    required
P9 1.0.34 install + physical validation      required
P10 commit + push                            required
```

Любой FAIL возвращает работу к первому общему механизму. Запрещено чинить
fixtures по одному или выпускать V28 до документированного объяснения, почему
формальная модель V27 была неполна.

## 16. Текущий вердикт

На момент создания документа:

```text
paper analysis complete             yes
V26 semantic gate                   FAIL
V27 code                            not implemented
V27 quality                         unknown
V27 latency                         unknown
runtime authority changed           no
installed runtime                   Lay 1.0.33
```

Route V4 прошёл structural gate: `PASS`, issues `0`, warnings `0`. Draft
implementation preflight V4 сохранён как `BLOCKED_BEFORE_CODE`, потому что в
нём отсутствовали два declared test ID. Финальный paper verdict выставляется
только после нового `READY_TO_IMPLEMENT` implementation preflight и отдельной
post-PASS критики. Implementation preflight V5 формально дал
`READY_TO_IMPLEMENT`, но вручную superseded: он не доказывал factory producer,
oracle independence, все typed candidate classes, observed-source route,
same-surface certificate merge и реальную field-free семантику exact mode.
V6 implementation preflight сохранён как `BLOCKED_BEFORE_CODE` с тремя
механическими blockers; код и runtime не менялись. V7 формально дал
`READY_TO_IMPLEMENT` (`78` baselines, `35` invariants, `49` tests), но ручная
атака отклонила его из-за отсутствия full-lattice retention, неверного EN RSS
assumption и неполной exhaustive taxonomy. V8 implementation preflight
сохранён как `BLOCKED_BEFORE_CODE` с одним identity-test-kind blocker; source и
runtime не менялись. V9 implementation preflight формально дал
`READY_TO_IMPLEMENT`, но post-PASS атака отклонила его из-за отсутствия
single-process EN guard ownership и aggregate PSS proof. V10 закрыл этот риск и
после отдельной post-PASS атаки получил paper verdict:
**V27_V10_PAPER_READY**. Это разрешение начать код, не quality или deployment
PASS.

## 17. Implementation checkpoint: FullField direct layout

Удалённый focused proof на `e@192.168.3.94` закрыл прежний единственный FAIL
широкого `layout_projection` filter. Первая потеря находилась не в exact lease:
`layout_converted_token()` строил `cnjq -> стой`, но общий admission не различал
phase evidence и более широкую RU surface-authority. Исправление ввело явные
поля evidence для strong signal, settled-after-layout и direct surface
authority. Известная английская исходная поверхность проверяется до рождения
direct candidate.

Измерено:

```text
FullField direct surface invariant             1/1 PASS
cnjq -> стой regression                        1/1 PASS
cnjq remains ClosedExactAbsent                 1/1 PASS
broad layout_projection filter               28/28 PASS
runtime authority changed                         no
installed runtime                        Lay 1.0.33
```

Это не promotion PASS. Standalone oracle, generated positive/negative corpora,
mutation/race, Space/undo, latency, RSS/PSS и физический ввод ещё не проверены.
Verdict scope: **IMPLEMENTATION_CHECKPOINT_PASS_NOT_PROMOTED**. Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/fullfield-layout-projection-checkpoint-v1.json`.

Полный focused P2 после этого checkpoint:

```text
exact-layout semantic tests                     9/9 PASS
retained exact segment                          3/3 PASS
closed-exact exhaustive taxonomy                1/1 PASS
Space prefetch and one-shot lease               8/8 PASS
complete frame identity                         1/1 PASS
```

Этот набор подтверждает внутренние Rust-инварианты, но не является независимым
quality proof: реализация и её unit-tests могут разделять одну ошибку. Runtime
authority не менялась. Verdict: **P2_FOCUSED_PASS_NOT_PROMOTED**. Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/focused-semantic-p2-v1.json`.

## 18. Independent oracle and generated corpus checkpoint

P2b/P3 выполнены на `e@192.168.3.94` независимым standalone oracle. Oracle
компилируется прямым `rustc`, не зависит от crate `lay`, не импортирует
runtime projection/guard/certificate helpers и владеет отдельными keyboard
table и lexical parsers. Remote `/usr/share/dict/words` отличался от локального
pinned-файла; proof поэтому запускался в mount namespace с byte-identical
локальной копией. Системный remote-файл и установленный Lay не менялись.

Frozen denominator:

```text
six byte-pinned lexical sources                         6
all generated rows                                243,748
eligible exact projections                         91,562
known-English collisions                              423
typed classes                                           27
empty class denominators                                 0
rows SHA-256        194fe17aea00cf11129a3be838b3255f921db09d1608fa16facbc017867ef47a
```

Первый independent run обнаружил `1,841` расхождение в oracle-классификации:
domain-like inputs с физической клавишей `.` и неверный case transfer через
punctuation keys были ошибочно названы eligible. Runtime правильно оставался
fail-closed. Исправлена классификация proof-корпуса: domain/acronym protection
получили отдельные negative classes; production guard не ослаблялся.

Итоговые измерения:

```text
certificate runtime/oracle parity       243,748 / 243,748 PASS
FullField/ClosedExact parity             243,748 / 243,748 PASS
false authority                                               0
oracle-only q<->w mutation divergences                   26,139
runtime-only q<->w mutation divergences                  26,133
restored runtime divergences                                  0
FullField proof wall                                     22.30 s
FullField proof mean CPU                                  1,142%
FullField proof max RSS                              419,916 KiB
runtime authority changed                                    no
installed runtime                                  Lay 1.0.33
```

Full/exact comparison включает terminal disposition, replacement,
`allow_apply` и transition proof. Oracle/runtime actual rows byte-equal.
Manifest:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/independent-oracle-manifest-v1.json`.
Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/independent-oracle-p2b-p3-v1.json`.

Verdict scope: **P2B_P3_PASS_NOT_PROMOTED**. Race/fault, Space, physical undo,
four separate latency denominators, controlled RSS/PSS and deployment remain
untested; этот checkpoint сам по себе не разрешает staging или install.

## 19. Race, Space, software undo and component latency checkpoint

P4 и software-части P5/P6 выполнены test-only observer-модулями поверх
production state machine. Они не создают второй runtime route и не входят в
release binary. Проверены `13` полей `InputFrameIdentity`, шесть независимых
authority fingerprints, material/worker generation, lock contention, poisoned
lock, complete publication, one-shot consume, full `NoApply` precedence и
cross-focus supersession.

Space effect matrix измеряет итоговый `AtomicProposal`, а не только возврат
correction call:

```text
identity faults rejected                              13 / 13 PASS
authority fingerprint faults rejected                   6 / 6 PASS
lock contention / poisoned lock                         2 / 2 PASS
partial published states                                    0 PASS
fallback Space outcomes                                 5 / 5 PASS
fallback lost / duplicate / glued Space                 0 / 0 PASS
exact apply trailing Space                              1 / 1 PASS
atomic refusal partial effects                          0 / 0 PASS
atomic refusal legacy retry                                  0 PASS
exact apply -> common double Shift -> exact source      1 / 1 PASS
```

Первый P7 component run выявил системный deadline-дефект: внутренний sleep
budget был равен внешнему `4 ms`, поэтому полный measured wait включал ещё lock
и scheduler overhead и дал p99 `4.220 ms`. Исправление уменьшило только
внутренний grace window до `3.500 ms`; rank, candidates, verifier и authority
не менялись. Повторный release run при concurrently busy stale FullField worker:

```text
printable exact miss                 n=2,048  p99=0.005 ms  PASS
printable exact hit                  n=2,048  p99=0.011 ms  PASS
Space exact lease lookup             n=2,048  p99<0.001 ms  PASS
Space full timeout                     n=128  p99=3.680 ms  PASS
Space full completion                  n=128  p99=0.598 ms  PASS
Space combined full wait               n=256  p99=3.680 ms  PASS
```

Что здесь не проверено: controlled engine RSS/aggregate PSS, post-edit
observed-source route, staged `514`-event RPC, compositor-level physical undo,
installed desktop runtime и deployment. Runtime authority не менялась;
установлен Lay `1.0.33`. Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/system-matrix-p4-p7-component-v1.json`.

Verdict scope:
**P4_SOFTWARE_P5_SOFTWARE_P6_SOFTWARE_P7_COMPONENT_PASS_NOT_PROMOTED**.

## 20. Post-resource oracle, controlled memory and latency checkpoint

Первоначальный exact EN guard сохранял полный набор слов в
`HashSet<String>`. Его controlled release RSS delta составил `16,136 KiB` при
лимите `14,336 KiB`; вариант `HashSet<Box<str>>` снизил максимум до
`14,408 KiB`, но всё равно превысил gate на `72 KiB`. Оба варианта отклонены.
Корпус, verifier, authority predicates и denominator при этом не менялись.

Принятый representation хранит те же `139,370` английских поверхностей и `266`
защитных записей в одном отсортированном `Box<[Box<str>]>`; membership является
точным binary search. Corpus reduction и probabilistic hash отсутствуют.
Fingerprint полного authority snapshot остался
`343338190052183766`, а логический объём строк и указателей равен
`3,430,872 B`.

Controlled release proof после baseline warmup:

```text
engine RSS budget                                  14,336 KiB
single-owner aggregate PSS budget                  16,384 KiB
maximum observed RSS delta                           9,480 KiB  PASS
maximum observed PSS delta                           9,484 KiB  PASS
maximum observed Pss_Anon delta                      7,952 KiB  PASS
second warmup Pss_Anon delta                             4 KiB  PASS
isolated release repetitions                                 5
english entries / protection entries             139,370 / 266
```

Maximum включает исходный cargo-driven release run и пять последующих
изолированных запусков того же бинарника. Максимум только среди пяти повторов:
`RSS 9,260 KiB`, `PSS 9,264 KiB`. Debug raw RSS не используется как release
gate: debug code pages и allocator layout не сопоставимы с release-LTO; его
controlled `Pss_Anon` использовался только как ранняя диагностическая метрика.

После изменения representation полный runtime oracle был повторён, а не
унаследован от предыдущего checkpoint:

```text
full runtime/oracle rows                     243,748 / 243,748 PASS
divergences                                                    0
baseline/runtime row SHA-256  194fe17aea00cf11129a3be838b3255f921db09d1608fa16facbc017867ef47a
byte comparison                                             PASS
```

Binary search также повторно прошёл component latency proof:

```text
printable exact miss                 n=2,048  p99=0.005 ms  PASS
printable exact hit                  n=2,048  p99=0.011 ms  PASS
Space exact lease lookup             n=2,048  p99<0.001 ms  PASS
Space full timeout                     n=128  p99=3.692 ms  PASS
Space full completion                  n=128  p99=0.620 ms  PASS
Space combined full wait               n=256  p99=3.692 ms  PASS
```

Что проверено: точная полнота EN guard, controlled single-owner release
RSS/PSS, allocation-stable повторный warmup, post-resource oracle parity и
component latency. Что не проверено: staged фиксированный `514`-event RPC,
физический compositor double-Shift undo, установленный desktop runtime и
post-install aggregate PSS активных Lay-процессов. Runtime authority не
менялась; установлен Lay `1.0.33`.

Verdict scope:
**P2B_P3_POST_RESOURCE_P7_COMPONENT_RESOURCE_PASS_NOT_PROMOTED**. Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/post-resource-oracle-latency-v1.json`.

## 21. Physical undo proof-route contradiction and correction

Первый post-resource physical undo run
`v27-post-resource-undo` не дошёл до expected marker:

```text
autocorrect visible                              ghbdtn -> привет PASS
physical Shift press/release pairs                               2 PASS
ManualToggleV2 handled                                          false
exact source restored                                           false
proof-owned leftovers                                           0 / 0
```

Это не отказ exact candidate, Space-frame или pending-undo policy. Proof source
после четырёх физических Shift-событий безусловно вызывал старый
`ManualToggleV2`. Но принятый V27 implementation preflight требует
`atomic double Shift ... with zero legacy mutations`, а принятый V24 modifier
route фиксирует:

```text
physical modifier event
-> Shell atomic adapter
-> Lay ProcessKeyEventAtomicV1
-> atomic proposal
-> Mutter atomic submit or exact native replay
```

В atomic focus `ManualToggleV2` намеренно возвращает `NotHandled`. Снятие этого
guard было бы неверным исправлением: оно вернуло бы второй legacy mutation
route, а live engine state до receipt-settlement может ещё не содержать
speculative Space apply и его pending undo. Это нарушило бы и zero-legacy gate,
и receipt identity.

Исправляется только proof observer. Он больше не вызывает `ManualToggleV2` и
обязан доказать одновременно:

```text
physical Shift events                                  exactly 4
atomic event-identity advance                           exactly 4
modifier entries in paired-release ledger                       0
second Shift release result                    one atomic undo frame
visible result                                     ghbdtn + one Space
legacy mutation calls                                          0
```

Первый Shift request тем самым доставляет receipt уже применённого Space-frame,
после чего pending exact undo становится live; второй Shift release строит undo
через тот же `EngineOutput::Atomic`. Daemon `ManualToggleV2` остаётся закрытым
для atomic focus и не становится вторым owner. На этом checkpoint production
Lay source, установленный `1.0.33`, Mutter и GNOME Shell runtime не менялись.
Если четыре modifier events физически не достигнут atomic adapter, proof обязан
снова упасть; тогда отдельный observed-route preflight потребуется до любого
runtime edit.

Preflight:
`docs/structural_gates/preflights/LAY_IME_ATOMIC_V27_PHYSICAL_UNDO_PROOF_ROUTE_CORRECTION_2026-08-22.json`.

Первая версия preflight была корректно остановлена до proof edit: она не
содержала source veto для reused observer и fault-injection tests для трёх
mutating steps. Blocked receipt сохранён как
`physical-undo-proof-route-preflight-v1.json`. Исправленный preflight V2:
`docs/structural_gates/preflights/LAY_IME_ATOMIC_V27_PHYSICAL_UNDO_PROOF_ROUTE_CORRECTION_V2_2026-08-22.json`.

## 22. Physical undo failure: observation/speculation reconciliation

Исправленный compositor proof `v27-atomic-modifier-undo-v2` подтвердил, что
proof observer больше не создаёт legacy route:

```text
physical Shift events                                  exactly 4 PASS
atomic event-identity advance                           exactly 4 PASS
legacy mutation calls                                          0 PASS
proof-owned leftovers                                          0 PASS
exact source restored                                      false FAIL
```

Значит, предыдущая proof-route contradiction устранена, но promotion всё ещё
запрещён. Первый потерянный invariant находится внутри settlement production
state, а не в Shell, Mutter, candidate generation, verifier или
`ManualToggleV2`.

Наблюдаемая причинная последовательность:

```text
T0 live engine: surrounding snapshot = old or missing, revision = R
T1 Space request: deep atomic clone captures revision R
T2 speculative engine emits the exact correction frame and invalidates its
   stale pre-apply surrounding snapshot
T3 compositor applies the frame
T4 SetSurroundingText publishes the fresh visible text into the live engine
T5 first Shift settles the Space receipt
T6 commit_atomic_speculation replaces the complete live engine with the older
   speculative clone
T7 fresh SetSurroundingText snapshot is lost
T8 second Shift release sees pending undo but no exact snapshot and abstains
```

Фактическая trace-граница:

```text
ibus_auto_undo_lifecycle stage=remember pending=true
ibus_surrounding_text text_chars=7
ibus_auto_undo_lifecycle stage=defer reason=waiting_exact_snapshot snapshot_chars=0
ibus_auto_undo_retry status=requested_exact_snapshot
```

Текущий `commit_atomic_speculation()` корректно переносит speculative state и
его isolated shared state только после receipt, но операция `*self =
speculative` ошибочно считает speculative clone владельцем наблюдений, которые
физически могли появиться уже после proposal. Полностью сохранять live snapshot
тоже нельзя: key processing намеренно обнуляет старый pre-key snapshot, и такое
безусловное сохранение вернуло бы stale authority.

### Reconciliation contract

`LayIbusEngine` получает монотонную в пределах engine lineage
`surrounding_observation_revision`. Только внешнее surrounding observation
увеличивает revision. Обычная локальная инвалидация snapshot при обработке
клавиши revision не увеличивает.

Settlement применяет следующий порядок:

```text
capture live observation revision/support/snapshot
-> commit receipt-owned speculative engine and shared state
-> if live observation revision > speculative observation revision:
     restore exactly that newer support/snapshot/revision
     evaluate committed visible postcondition against this snapshot
-> otherwise:
     preserve speculative snapshot state, including intentional invalidation
-> apply deferred layout and learning effects once
```

Это reconciliation одного state owner, а не merge двух decision routes.
Revision не авторизует текстовую мутацию, не меняет candidate rank и не заменяет
receipt identity. Она отвечает только на один вопрос: появилось ли внешнее
наблюдение после создания speculative clone.

Внешний переход capability `surrounding text supported <-> unsupported` также
является surrounding observation: он должен сдвигать ту же revision и не может
быть затёрт более старым clone. Focus/reset сначала удаляет pending atomic
transition, поэтому state другой focus lineage никогда не участвует в merge.

### Required proof matrix

```text
newer post-proposal SetSurroundingText survives receipt settlement       required
newer snapshot evaluates the speculative visible postcondition          required
same-revision pre-proposal snapshot remains invalidated                  required
four physical Shift events restore exact source through atomic frame    required
legacy bridge calls                                                          0
partial or duplicate mutations                                                0
fixed full-route events                                                   514/514
```

Failure policy:

- missing or equal revision never resurrects a live snapshot;
- a newer missing/unsupported observation remains missing/unsupported;
- focus/receipt mismatch remains fail-closed and performs no merge;
- no fallback to `ManualToggleV2`, legacy key processing or direct D-Bus text
  mutation is permitted;
- any regression in the unchanged `514`-event route blocks deployment.

Что измерено: physical modifier cardinality, zero legacy calls, zero leftovers
и точная trace-точка потери snapshot. Что ещё не измерено: исправленный Rust
reconciliation, повторный physical exact restore, повторный `514/514`,
post-install desktop route и live RSS/PSS. Runtime authority не менялась;
установлен Lay `1.0.33`.

Failed evidence receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/physical-atomic-modifier-undo-v2-fail.json`.

Verdict scope: **ROOT_CAUSE_MEASURED_IMPLEMENTATION_NOT_AUTHORIZED**. Перед
production edit обязателен новый implementation preflight, привязанный к
фактическим baseline bytes этого settlement route.

Code-route critique V1 была отклонена за смешение receipt authorization с
freshness arbitration. V2 исправила authority flow, но не построила отдельный
observation graph. V3 разделила `execution | authority | observation | proof`,
зафиксировала по одному settlement/authority/observation path и получила
structural `PASS`. Это ещё не разрешение редактировать code; receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/surrounding-observation-reconciliation-route-v3.json`.

## 23. Reconciled observation exposes an in-flight modifier handoff loss

Reconciliation implementation из раздела 22 прошла собственные targeted tests
и неизменённый physical full-route gate:

```text
fixed key events                                              514/514 PASS
paired releases                                              514/514 PASS
legacy key calls                                                    0 PASS
proof-owned leftovers                                         0 / 0 PASS
p50                                                        1.472 ms
p99                                                        3.471 ms
max                                                        4.445 ms
```

Staged engine:

```text
binary SHA-256  57b132b5d4936df62e74944f3fbacd03cf63296fdb5d4ba413560dd19823f0a9
source aggregate 989fc83d554d13ffdd09d2ba3c2701da6a496efe844897e30c20bb7b50c55663
```

Отдельный unchanged physical undo proof
`v27-surrounding-reconcile-undo-v1` всё ещё упал:

```text
ghbdtn + Space -> привет + Space                              PASS
physical Shift event-identity delta                              4 PASS
legacy key calls                                                   0 PASS
proof-owned leftovers                                        0 / 0 PASS
exact source restored                                           false FAIL
```

Проверка `modifierEventDelta == 4` выполняется proof source до ожидания exact
original. Поэтому последующий timeout не означает, что Shell не увидел Shift.
`engine_key_events=7` также не является числом всех физических событий:
production trace намеренно не пишет обычные unhandled modifier press/release.

### First lost invariant

Новая trace отличается от предыдущей fail trace принципиально:

```text
fresh visible snapshot survives settlement                    PASS
visible postcondition confirmed                               PASS
layout postcondition requests lay-ime-ru                       PASS
lay-ime-us focus out -> lay-ime-ru focus in                    PASS
exact auto-undo                                                FAIL
```

Существующий text handoff не отсутствует. `replace_committed_tail()` уже
публикует bounded `preserve_active_path_until = now + 700 ms`, а
`bind_focus_path()` при действующем lease переносит tail, epoch, focus receipt и
shared `pending_auto_undo`. Этот механизм ранее отдельно закрывал класс
`layout autocorrect -> engine-path handoff loss` и не должен быть заменён новым
общим сохранением focus state.

Потерян другой state: in-flight modifier gesture остаётся локальным полем
конкретного `LayIbusEngine`:

```text
T0 first Shift press starts the next atomic request
T1 that request settles the accepted Space frame
T2 reconciliation confirms the visible correction
T3 background layout postcondition switches lay-ime-us -> lay-ime-ru
T4 the press observation remains in old-engine shift_pressed_at
T5 the release is processed by the new engine with no matching press
T6 the second physical tap is classified only as the first complete tap
T7 pending auto-undo remains unused and no undo frame is born
```

Это общий класс потери незавершённого modifier gesture при намеренном
process-level engine handoff. Он не относится к слову `ghbdtn`, candidate rank,
L1.1/L2/L3, verifier или Shift timing coefficients.

### Critique of possible repairs

Rejected:

- вернуть `ManualToggleV2` после физических Shift: создаёт второй mutation
  owner и нарушает zero-legacy contract;
- считать orphan Shift release обычным tap без provenance: создаёт ложные
  double-Shift срабатывания;
- сохранять весь engine object через любой focus change: переносит composition
  и authority между несвязанными focus lineages;
- задержать каждый layout switch отдельным timer route: меняет latency и
  ordering всех layout transitions, хотя потеря ограничена одним typed state;
- расширить tap/window coefficients: не восстанавливает отсутствующий press и
  маскирует state loss.

Selected repair: **one-shot typed Shift gesture handoff** поверх уже
существующего bounded layout-switch lease.

```text
old engine processes an atomic native-unhandled Shift observation
-> commit_native_observation commits only modifier observation
-> if and only if:
     pending_auto_undo exists
     preserve_active_path_until is still valid
   publish one typed gesture snapshot with source engine path

new active Lay engine processes the next event
-> if and only if:
     active path matches this engine
     source path differs
     the same bounded lease is still valid
     pending_auto_undo still exists
   consume the gesture snapshot once
-> continue the existing Shift detector
-> existing exact-snapshot auto-undo authority may produce one atomic frame
```

The typed snapshot carries the complete detector continuity needed to avoid
false taps: `shift_active`, `shift_pressed_at`, `shift_used_as_modifier`, and
`last_shift_release_at`. It contains no text, candidate, edit plan, receipt or
mutation authority. Consuming it cannot by itself authorize an undo.

Cleanup contract:

- ordinary path/focus change without a valid preserve lease clears the gesture;
- expiry, missing pending auto-undo, replacement invalidation and successful
  undo clear the gesture;
- the same engine path cannot consume its own handoff;
- consumption is one-shot;
- `tap_max_ms` and `shift_window_ms` remain unchanged;
- physical proof source remains byte-identical at
  `3f2d38ae86e5a984c055bb04837e299aee7aa41bcdf78d964670676fdb62e4bd`.

Required proof matrix:

```text
cross-engine press/release continuity under valid lease                 required
Shift used as a modifier cannot become a false tap                      required
expired or absent lease cannot transfer gesture                         required
ordinary changed path still quarantines text, undo and gesture          required
newer surrounding reconciliation remains passing                       required
physical ghbdtn round trip with four Shift events                       required
fixed full route                                                     514/514
legacy calls and leftovers                                                    0
```

Measured facts: corrected reconciliation targeted tests, unchanged
`514/514` route, physical event cardinality, exact restore failure, zero legacy
calls, zero leftovers, binary/source identities and the observed engine-path
transition. Not yet measured: typed gesture implementation, corrected physical
round trip, post-install desktop behavior, RSS/PSS. Installed runtime remains
Lay `1.0.33`; runtime authority did not change.

Failed evidence receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/physical-surrounding-reconcile-undo-v1-fail.json`.

Verdict scope: **GESTURE_HANDOFF_ROOT_CAUSE_PAPERED_NOT_AUTHORIZED_TO_EDIT**.
The route gate and a new implementation preflight must both pass before source
edits.

## 24. Typed handoff passes; the physical proof denominator is stale

Typed `ShiftGestureHandoff` implemented exactly the bounded contract from
section 23. The snapshot contains only `source_path`, `shift_active`,
`shift_pressed_at`, `shift_used_as_modifier` and `last_shift_release_at`.
Candidate rank, verifier, layout synchronization and Shift timing windows did
not change.

Targeted remote tests on `e@192.168.3.94` passed for cross-engine exact undo,
modifier use, one-shot transfer and expired/absent lease quarantine. The staged
engine identity is:

```text
binary SHA-256  01033e4b02fe39e57f229946063a68cb9c36a4d960466acd060cf5e43a7d43b9
source aggregate e8474ee0fa14142bc1e0839f85df0b821fd5715d3ddd0c7ddab6a9fece6ca3f0
```

The first fixed full-route run was functionally complete but had an isolated
scheduler maximum of `24.016 ms`. A quiescent repetition retained the same
fixed denominator and passed:

```text
fixed route                                                 514/514 PASS
p50                                                            1.472 ms
p99                                                            3.200 ms
max                                                            3.832 ms
legacy calls                                                           0
proof-owned leftovers                                                0/0
```

The isolated physical run `v27-shift-gesture-handoff-undo-v1` then reached the
exact original and returned from `proveDoubleShiftUndo()`. This ordering is
material: that function waits for visible `ghbdtn `, verifies modifier event
delta `4`, verifies an empty paired ledger, and only then returns to the outer
counter assertion. The only observed failure was the subsequent stale
denominator:

```text
visible autocorrect                         ghbdtn -> привет PASS
physical exact restore                      привет -> ghbdtn PASS
paired printable releases                                  7/7 PASS
Shift release atomic RPCs                                     2
Shift release native replays                                  1
paired-release ledger entries                                 0
legacy calls                                                   0
proof-owned leftovers                                       0/0
outer proof status                                             1 FAIL
```

The old proof expected `release_rpcs=0` and `native_releases=0`. That expectation
contradicts the accepted V24 modifier route. For immediate double Shift the
correct event contract is:

```text
first Shift release  -> atomic RPC -> native replay
second Shift release -> atomic RPC -> exact undo frame, no native replay

paired_releases=7
release_rpcs=2
native_releases=1
ledger_entries=0
```

This is a proof-owner defect, not a product defect. Changing production Lay,
Mutter, GNOME Shell routing, candidate generation, verifier, Shift windows or
the staged engine would invalidate the measured isolation. The next permitted
edit is limited to the undo assertion and its shell parser in:

```text
/home/ubu/projects/gnome-shell-lay-atomic-proof/gnome-shell-50.1/tests/shell/atomicInputMethodRoute.js
/home/ubu/projects/lay-l1-exact-peak-search/scripts/proof/run-atomic-full-route-remote.sh
```

Measured evidence receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/physical-shift-gesture-handoff-undo-v1-denominator-fail.json`.

What was not tested by this failed outer verdict: a rerun with the corrected
parser, installed Lay `1.0.34`, post-install live double-Shift and post-install
RSS/PSS. Runtime authority did not change; installed runtime remains Lay
`1.0.33`.

Verdict scope: **PRODUCT_UNDO_PASS_PROOF_DENOMINATOR_FAIL**. A new code-route
gate and implementation preflight scoped only to the denominator correction
are required before editing either proof file.

Route draft V1 was rejected before code because it chained `proves` through an
`evidence_owner` and a `producer`; the proof graph requires every `proves` edge
to terminate at a `proof_owner`. V2 therefore separates physical observation,
V24 evidence transfer, JavaScript marker production and shell verdict proof
into their own typed graphs. V1 remains negative structural evidence; no
source or runtime changed.

Route V2 was also rejected before code because it placed a `proof_owner` in a
live authority graph. V3 removes that false authority claim: the accepted V24
denominator is immutable input to the JavaScript proof owner, while the route
itself contains only observation, proof, marker production and final proof.
V3 passed with zero issues and zero warnings. Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/physical-undo-denominator-route-v3.json`.

Implementation preflight V1 pinned both proof owners and the installed
`1.0.33` engine, declared twelve mapped checks and returned
`READY_TO_IMPLEMENT` with zero blockers. This authorizes only the two proof
denominator edits described above; it does not authorize deployment. Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/physical-undo-denominator-implementation-preflight-v1.json`.

The corrected observer and parser were synchronized with exact SHA parity and
run under two new append-only IDs. Both passed:

```text
physical undo run                 v27-shift-gesture-handoff-undo-v2
exact restore                                                       PASS
paired releases / RPC / native / ledger                        7/2/1/0
process / legacy / leftovers                                    0/0/0

fixed route run                 v27-shift-gesture-handoff-route-v3
events                                                         514/514
p50 / p99 / max                                1.384/3.193/3.896 ms
paired releases / RPC / native / ledger                      514/0/0/0
process / legacy / leftovers                                    0/0/0
```

Proof source SHA-256 is
`28e297a78745d113411727da341504932d26cfa098dc7fa55e0cdff7fff7ac4a`;
the shell parser SHA-256 is
`5d728b4bdca9ea4b56f591cafa38d8e3876cc80b5cb5f7cc14cd0473aa064624`.
Both runs used the unchanged staged engine
`01033e4b02fe39e57f229946063a68cb9c36a4d960466acd060cf5e43a7d43b9`.

PASS receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/physical-shift-gesture-handoff-and-route-pass-v2.json`.

Verdict scope: **V27_STAGED_PHYSICAL_AND_FIXED_ROUTE_PASS**. This closes the
proof denominator and permits a versioned release build. It does not yet prove
the final `1.0.34` binary, installed desktop behavior or post-install RSS/PSS;
runtime authority is still unchanged and installed Lay remains `1.0.33`.

### 24.1 Final 1.0.34 remote release build

Build preflight V1 was retained as `BLOCKED_BEFORE_CODE` because three declared
forbidden effects lacked static scan coverage. V2 added only those scans and
returned `READY_TO_IMPLEMENT` with zero blockers. Version surfaces were then
mechanically advanced from `1.0.33` to `1.0.34`; no production behavior source
changed after the staged physical PASS.

The complete build input aggregate was synchronized byte-for-byte to
`e@192.168.3.94` and built through `cargo-guard` with `CARGO_BUILD_JOBS=20`.
The release profile retained `codegen-units=1` and linker-plugin LTO, so the
final crate optimization intentionally used one `rustc` core rather than a
different faster build profile.

```text
source aggregate SHA-256  22f6a4198d7c2aa388e58ff806dade001e7d0d0fa16d96b7810d1bee804894cf
remote build wall                                                    3m28s
remote target before / after                              1.33 / 1.77 GiB
remote target budget                                                12 GiB
release binaries                                                        10
downloaded SHA parity                                                 PASS
installed runtime during build                                     1.0.33
```

The old third exact filter was stale and selected zero tests. It was not
counted as PASS. All focused suites were rerun with non-zero denominator checks:

```text
exact raw-known projection                                           1/1
exact full-route authority                                           1/1
exact protected/composite negative                                   1/1
atomic state tests                                                   8/8
atomic proof tests                                                   5/5
engine profile/handoff tests                                       18/18
Space prefetch tests                                                 8/8
```

Final release engine SHA-256 is
`3bb009025f3bd12c676416aff637b439ccd4f4c0c2d69d4c7c34b0f49b904691`.
The complete immutable stage is
`/home/ubu/.cache/lay/releases/1.0.34-22f6a4198d7c2aa388e58ff806dade001e7d0d0fa16d96b7810d1bee804894cf`.

Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/release-1.0.34-remote-build-v1.json`.

Verdict scope: **RELEASE_BUILT_NOT_INSTALLABLE_YET**. The final versioned
engine must repeat the physical fixed route and exact double-Shift undo before
any installed byte or live process is changed.

The final versioned engine was admitted into the isolated proof environment
and repeated both gates:

```text
engine SHA-256             3bb009025f3bd12c676416aff637b439ccd4f4c0c2d69d4c7c34b0f49b904691
source manifest SHA-256    22f6a4198d7c2aa388e58ff806dade001e7d0d0fa16d96b7810d1bee804894cf
fixed route                                                     514/514 PASS
p50 / p99 / max                                  1.357/3.250/4.091 ms
route release counters                                      514/0/0/0
exact physical double Shift                                      PASS
undo release counters                                           7/2/1/0
legacy calls / leftovers                                           0/0
```

Receipt:
`docs/structural_gates/receipts/LAY_IME_ATOMIC_EXACT_LAYOUT_LEASE_V27_2026-08-22/final-release-1.0.34-physical-pass-v1.json`.

Verdict scope: **FINAL_RELEASE_BINARY_PHYSICAL_PASS_INSTALL_NOT_STARTED**.
Installation now requires its own rollback-sensitive preflight. Installed Lay
is still `1.0.33` and runtime authority has not changed.

### 24.2 Live owner topology and memory-gate correction

The first two live installation transactions were fully rolled back because
their acceptance gate incorrectly required one `lay-l1.1-serve` process. The
release implementation intentionally uses direct IME ownership:

```text
lay-daemon          1
lay-l1.1-serve      0
lay-ibus-engine     1
lay-l3-online       1
global ibus-daemon  unchanged
```

This topology is pinned by the source ownership files and by the remote
`warmup_plan_*` proof, which passed `4/4`. Preflight V6 then admitted only the
corrected topology. A pre-install attempt stopped before byte installation
when the user-visible active layout changed to `lay-ime-us`; rollback restored
`1.0.33`, and `lay-ime-ru` was reselected without restarting global IBus.

The next V6 transaction installed the immutable release and reached live SHA
parity before the memory gate:

```text
installed binaries                                      10/10 PASS
loaded extension                                              1.0.34
live topology                                                1/0/1/1
global ibus-daemon PID                                      2076194
engine committed samples, KiB     334819/334819/334819/334819/334819
total committed samples, KiB      339411/339411/339411/339411/339415
old engine-only cap, KiB                                  282202 FAIL
old total-owner cap, KiB                                  736062 PASS
rollback to 1.0.33                                           PASS
```

The engine-only comparison was structurally stale. In `1.0.33`, the same
direct-IME ownership envelope was split between engine `265818 KiB` and
sidecar `181566 KiB`; comparing the merged `1.0.34` engine only with the old
engine erased the ownership migration from the denominator. The corrected
conjunctive memory contract is:

```text
direct-owner engine <= old engine + old sidecar   447384 KiB
all live Lay owners                               736062 KiB
```

The observed release has `112565 KiB` engine-envelope headroom and `396647
KiB` total-envelope headroom. No quality, topology, SHA, extension, IBus,
journal or rollback gate is weakened. The exact evidence is in
`evidence/live-install-attempt-v4.log` and
`evidence/live-install-attempt-v4-memory-correction.json` under the V27 receipt
directory.

What was not tested by this correction: a complete promotion transaction with
the ownership-aware per-engine bound. Runtime authority remains `1.0.33` until
that transaction passes every remaining live gate.

Verdict scope: **MEMORY_DENOMINATOR_CORRECTED_LIVE_PROMOTION_NOT_RETRIED**.

### 24.3 Final live promotion

The ownership-aware V7 preflight passed with `53` baseline checks, no blockers,
and manifest SHA-256
`5e8f9f0931aa2d1fba7ef21411bf1efc02e7a1af1a68ae4f5ccf87c602178932`.
The rollback-protected `LIVE_INSTALL_V5_START` transaction then promoted the
immutable release without restarting the global IBus daemon:

```text
installed Lay                                             1.0.34 PASS
loaded extension                                         1.0.34 PASS
active engine                                          lay-ime-ru PASS
live topology                                               1/0/1/1 PASS
global ibus-daemon PID                              2076194 -> 2076194 PASS
loaded engine SHA                                    release SHA parity PASS
engine committed max                                      250481 KiB PASS
direct-owner engine gate                                  447384 KiB
all-owner committed max                                   256694 KiB PASS
all-owner gate                                             736062 KiB
recovery watchdog                                      removed/inactive PASS
live success marker                                            present PASS
```

The corrected gate did not merely turn the previous `334819 KiB` observation
into a pass. After age-normalized startup sampling, the final engine maximum was
`250481 KiB`, leaving `196903 KiB` in the direct-owner envelope; all measured
owners used at most `256694 KiB`, leaving `479368 KiB` in the total envelope.

Rollback remains available at
`/home/ubu/.local/lib/lay/rollback/1.0.33-pre-1.0.34-v27-20260822-1116`
with `186671333` bytes and a mode-preserving `FILES.tsv` manifest. The exact
transaction evidence is `evidence/live-install-attempt-v5.log`; the structured
deployment receipt is `final-release-1.0.34-live-deployment-pass-v1.json` in
the V27 receipt directory.

What was not tested by the installation transaction: a human double-Shift undo
in an arbitrary desktop application, long-duration RSS drift, and
application-specific behavior outside the fixed `514/514` physical route.
Those limits do not change the fixed physical proof, but they remain outside
the deployment claim.

Verdict scope: **FINAL_RELEASE_1_0_34_LIVE_DEPLOYMENT_PASS**. Runtime authority
changed from Lay `1.0.33` to Lay `1.0.34`.
