# План разработки Lay после 0.2.340

## Целевой контур

```text
L1.1 immutable restoration
-> canonical L2 candidate field
-> L3 sentence/context field + online deltas
-> L4 causal transition memory
-> DecisionCore
-> verifier
-> IME/daemon backend
```

## Правила работы

- L1.1 и L2 заново не кристаллизуем. Их пакеты и SHA фиксируются как baseline.
- Обучение L3/L4 выполняется маленькими append-only delta-пакетами.
- Тяжёлые proof/build запускаются на `e@192.168.3.94`, до 20 workers, с CPU/RSS/time telemetry.
- Никаких правил под конкретные слова, приложения или `Apple`.
- Каждый эксперимент сразу попадает в архитектурный документ и receipt.
- Каждый этап заканчивается тестами, отдельным коммитом, push и синхронизацией версии.

## 0. Зафиксировать Baseline

**Статус: уже выполнено.**

```text
release                 0.2.340
L1.1                    13/13 классов >95%
L1.1 package            176.27 MiB
L2 V13 package          128.86 MiB
WeChat                  live PASS
HEAD == public/main
```

## 1. Закрыть Ownership L2

**Статус: выполнено 2026-08-01, release target `0.2.342`.**

Live owner переименован в `CanonicalL2Field`; test-only donor-reuse удалён,
единственный исполняемый L2 readout загружает immutable standalone V13. На
прогретом before/after snapshot совпали `8 / 8` входов и `86 / 86` candidate
records; package SHA не изменился, inherited V13 false authority остаётся `0`.
Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_LIVE_OWNER_CUTOVER_2026-08-01.json`.

1. Переименовать live-маршрут `L2FieldShadow` в `CanonicalL2Field`.
2. Удалить остаточный donor-reuse и старые fallback-маршруты.
3. Оставить единственный источник L2: standalone V13 над lattice L1.1.
4. Сохранить бинарный пакет L2 без перекомпиляции.

**Gate:** полное совпадение lattice/winner/tied/abstain до и после изменения, package SHA неизменен, false authority `0`.

## 2. Довести Онлайн-Обучение L3

**Статус: выполнено; proof-gated live admission подтверждён поколениями `9` и
`10` 2026-08-21.** Автоматическое применение не считается пользовательским
подтверждением. Один action получает общий `episode_id`, selector проверяет
ровно одну минимальную связь, а manifest защищён
`flock + fsync + atomic rename`. Для каждого из двух последних принятых
отношений наблюдались `2` независимых эпизода в `2` независимых сценах.
Targeted proof и полный differential proof дали `PASS`; `false_supports`,
`lost_supports`, `lost_top1`, `new_false_supports` и `new_false_top1` равны
нулю. Текущий live state: generation `10`, admitted deltas `7`, pending
relations `86`, proof pipeline revision `1`. После каждого admission один
малый delta был безопасно свёрнут в compact base, а runtime manifest остался
delta-free. Базовые L1.1/L2 пакеты не перекристаллизовывались.
Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L3_ONLINE_CAUSAL_EPISODES_V3_2026-08-01.json`.
Current live-state audit:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L3_ONLINE_LIVE_ADMISSION_GENERATIONS_9_10_2026-08-23.json`.

1. Разделять журнал на независимые причинные эпизоды.
2. Не считать показанную подсказку правильным ответом.
3. Принимать только подтверждённые действия: выбор, исправление, откат, редактирование completion.
4. Добавить impact-aware selector: выбирать минимальный набор отношений, который улучшает цель и не разрушает старые бассейны.
5. Компилировать один маленький delta-пакет за проход.
6. Автоматически загружать delta только после полного proof.

**Gate:**

```text
targeted improvement        >0
targeted false supports      0
80k lost supports            0
80k lost top-1               0
new false supports           0
new false top-1              0
atomic runtime reload        PASS
```

Результат этапа: реальные пользовательские deltas автоматически принимаются и
загружаются без перекомпиляции базовой модели; поколение `10` является
последним подтверждённым admission на момент этого аудита.

## 3. Расширить L3 до контекста предложения

**Статус: выполнено 2026-08-01, release `0.2.344` установлен.** Поле кодирует
левый/правый контекст, пунктуацию, порядок и морфослот в `14` ограниченных
представлениях. На фиксированном heldout `25/25` случаев прошли, включая `5/5`
неоднозначных; переносимый composite proof улучшил `20` случаев без регрессий,
а frozen 80k differential дал нули по всем пяти regression counters. Delta
занимает `79 660` байт. Receipts:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L3_SENTENCE_MULTIVIEW_V1_TARGETED_2026-08-01.json`
и
`/home/ubu/projects/lay/docs/structural_gates/receipts/L3_SENTENCE_MULTIVIEW_V1_FULL_80K_2026-08-01.json`.
Remote release gate: `83/83 + 28/28 + 5/5 + 2/2 + 20/20 + 1/1 PASS`;
build `117.25 s`, average CPU `243%`, peak RSS `1 621 868 KiB`, swaps `0`.
Все `10/10` бинарников установлены с byte parity; sentence-delta принят как
второй append-only пакет, global IBus и managed engine не перезапускались.
Installed receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L3_SENTENCE_MULTIVIEW_V1_INSTALLED_2026-08-01.json`.

1. Кодировать левый и правый контекст, пунктуацию, порядок слов и морфологический слот.
2. Строить отношения между всеми кандидатами L2, а не только выбранной парой.
3. Научить поле различать союз, предлог, окончание, форму глагола и соседние сущности.
4. Проверить обобщение на неизвестных предложениях.
5. Сохранить `Tied/ABSTAIN`, когда контекста недостаточно.

**Предлагаемый gate:** target lattice `>=99%`, однозначный top-1 `>95%` по каждому контекстному классу, ambiguity retention `>=99%`, false authority `0`.

## 4. Типизированное ядро L4

**Статус: выполнено 2026-08-01, release `0.2.345` установлен.** Live-domain
`TypingMemoryEvent` больше не принимает строковые `source/operation`: provenance,
interaction operation, transition operator, layout direction/scope и outcome
представлены типами со стабильными `u8`-кодами. Persistent `UsageEvent` V2 хранит
коды вместе с lossless legacy-label, читает V1 и fail-closed отклоняет
противоречивый V2. Read-only replay типизировал `2 456 / 2 456` реальных строк,
не переписал журнал и сохранил все `711` word states, `14 698` transition states
и `9 416` signed transition states. На фиксированном dirty replay baseline и
candidate дали одинаковый JSON SHA-256
`e20dd25f31cd64923f4061228696b836a88740ca2aba9916451c967395de5dba`;
`negative_false_apply = 0` до и после. Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L4_TYPED_EVENT_V2_REPLAY_PARITY_2026-08-01.json`.
Пять затронутых release-бинарников установлены с byte parity; global IBus,
managed engine и daemon не перезапускались.

1. Заменить строковые `source/operation` на типы: `operator`, `direction`, `scope`, `outcome`, `evidence source`.
2. Разделить `RU->EN` и `EN->RU`, whole-token и single-grapheme.
3. Замкнуть цепочку:

```text
state snapshot
-> proposal
-> verifier decision
-> AuthorizedEdit
-> observed result
-> accepted/rejected/reverted outcome
```

4. Сначала изменить только представление данных, без изменения поведения.

**Gate:** replay parity `100%`, существующие false apply остаются `0`.

## 5. Обучаемый Cross-Scene L4

**Статус: выполнено как typed cross-scene V2 `PASS_SHADOW`; organic package
установлен с release `1.0.30` и продолжает использоваться в `1.0.39`.**
Реализованы typed language/layout/script/keyboard-geometry identities,
candidate-relative encoder, причинный join, transactional episode inbox,
latest-state consolidation, positive/anti/hard-negative/ambiguity banks,
направленные pair profiles, bounded binary V2 и read-only hot reload. Поле
подключено к общему `TransitionDecisionCore` только как
`SuggestOnly | Keep`: оно может дать контекстное evidence, но не рождает слова,
не владеет вторым ranking, не обходит verifier и не применяет исправление
самостоятельно.

Фиксированный RU/EN heldout содержит `640` случаев; все `10/10` классов дали
`100%`, false supports `0`, automatic apply `0`. Ablation без anti-centers
даёт `204` false supports, с anti-centers остаётся `0`; положительные
`218/218` сохранены. Hot readout p50/p99/max равен
`3.023 / 4.061 / 7.572 us`. Установленный organic V2 package содержит `26`
profiles, `67` pair profiles и `5` typed symbols, занимает `21,445 B`, имеет
SHA-256 `1a1e926c4b4c972add54ce3235b1f1527276365abe74952abb538a3132c97e3e`
и проходит runtime reader. Полного отрицательного органического evidence пока
нет, поэтому самостоятельная live authority намеренно не выдавалась.
Deployment receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L4_MULTILINGUAL_SCENE_FIELD_V2_DEPLOYMENT_2026-08-15.json`.

1. Реализовать candidate-relative scene encoder.
2. Обучать positive, anti и ambiguity centers только по причинным receipts.
3. Сначала разрешить перенос только как `SuggestOnly`.
4. Отдельно доказать whole-token layout transfer.
5. Затем исследовать single-grapheme transfer.
6. Провести ablation: без anti-centers качество обязано измеримо ухудшаться.

**Gate:** улучшение на unseen equivalent scenes, `RU->EN` и `EN->RU` проходят отдельно, false automatic layout projection `0`.

## 6. Производительность без потери кандидатов

**Статус: выполнено 2026-08-02, release target `0.2.347`.** Канонический
lossless shard-1 V8 пакет занимает `198,233,790 B` (`189.05 MiB`) и сохраняет
все `108,156,559` forward relations. Для fixed-520 (`13 x 40`) candidate SHA
остался точным:
`f3501c6cf518ba9747b5ef4323565199dcfa49fde4068c97c2e728c451c536c2`.
Три независимых процесса на P-core CPU set `4-11` дали raw first-touch p99
`4.333 / 4.351 / 4.950 ms`, hot p99 `4.397 / 4.917 / 4.865 ms` и complete
restoration p99 `4.490 / 4.148 / 4.786 ms`. Peak RSS составил
`345,180-346,956 KiB`, warmup `1.218-1.234 s`. Runtime authority не менялась.

Принятые defaults: `4` runtime workers, `64 MiB` posting cache, `0 MiB` shard
cache, `16 MiB` reverse cache и `4096` слов warm-profile. Parallel scan
reconstruction reserve отклонён: два из трёх restoration p99 превысили `5 ms`.
На гибридном CPU без affinity два из трёх контрольных процессов также
превысили gate на `0.193` и `0.028 ms`; поэтому P-core affinity является частью
измеренного deployment contract, а не свойством бинарного формата.
Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_FIRST_TOUCH_PHASE6_2026-08-02.json`.

1. Уменьшить L1.1 diverse first-touch p99 с `24.365 ms`.
2. Использовать mmap-prefetch, background warmup, cache и bounded active sets.
3. Не удалять candidate sources и не уменьшать поле.
4. Проверить точное совпадение результатов до и после оптимизации.

**Цель:** first-touch p99 `<=5 ms`, hot p99 `<=5 ms`, candidate parity `100%`.

## 7. Финальный Product Gate

**Статус: `PASS_CODE`, release target `0.2.348`, live application matrix
остаётся отдельным gate.** Последовательный `lay-daemon` gate прошёл
`200/200`; representative adjacent-transposition sweep восстановил `487/497`
форм (`97%`). Полный прогон занял `245.60 s`, peak RSS составил `353,900 KiB`.
Порядковая утечка `HotFieldPolicy` устранена: чтение test-конфига больше не
переключает process-wide authority библиотеки.

В одном gate подтверждены сохранение пробела при удлинении слова, exact short
layout projection `дфн -> lay`, защита общего короткого RU -> ASCII шума,
начальная пропущенная гласная для сильного словарного центра и сохранение
естественных русских слов/дефисных конструкций. Регрессия `бычный -> бычиный`
устранена общим morph-class gate: сильный начальный vowel-center допускается
для прилагательного или после наблюдаемой удвоенной согласной; ложное
`лучшить -> улучшить` остаётся запрещено.

Boundary-shift readout больше не выдаёт direct authority только из-за двух
морфологически похожих половин. На clean corpus ложные применения уменьшены
`3/220 -> 0/220`; на `188` неомонимичных synthetic shifts поле предложило
правильную границу в `185` случаях (`98.4%`) и автоматически применило `156`.
Повреждения, совпавшие с самостоятельным clean surface, исключены из top-1
denominator и остаются неоднозначными.

Runtime authority изменилась: verifier-proven deterministic repair и точная
layout projection к известному центру могут пройти ранее общий hidden/Bayesian
отказ. Наблюдаемая clean surface по-прежнему имеет veto. Это не новый L1.1
quality proof: fixed heldout `13 x 20,000` и проценты по классам здесь не
перезапускались. Физическая матрица приложений ниже также не заменяется
unit/integration gate.

Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/FINAL_PRODUCT_GATE_PHASE7_2026-08-02.json`.

Технический release-контур завершён 2026-08-02. Изолированная сборка на
`e@192.168.3.94` использовала `20` Cargo jobs и закончилась за `143.52 s` при
средней загрузке `475%`, peak RSS `1,659,024 KiB`, без swap и с exit `0`.
Десять бинарников перенесены с SHA-256 parity и установлены атомарно. CLI,
daemon и GNOME extension показывают `0.2.348`; daemon перезапущен отдельно,
IBus сохранил PID `4119`. Reload extension временно изменил активный engine,
после чего прежний `lay-ime-ru` был восстановлен без перезапуска IBus.

Проверить одну матрицу:

```text
WeChat | Telegram | Chromium | GTK | Qt | Kitty
typing | preedit | Backspace | Tab | Space
double Shift | focus change | layout conversion
```

Осталось: физическая application matrix, затем commit и push. Она не заменяет
уже пройденные code gates и не заменяется ими.

### Lay 1.0 closure checkpoint: 2026-08-02

Fixed L1.1 proof завершён на `13 x 20,000`: каждый класс прошёл строгий
`unique top-1 >95%`, clean preservation `100%`, false authority `0`, false
singleton `0`. Полный список классов и процентов записан в
`/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_FIXED_13X20000_2026-08-02.json`.
Многопоточный proof latency остаётся отдельным `WATCH`: `13.014 ms` hot p99 и
`10.893 ms` L1.1 p99 не выдаются за deployment PASS.

Текущий L3 online state содержит `14` pending relations, только `4` имеют два
эпизода и ни одна не имеет две независимые сцены. Promotion eligible `0`, новые
online deltas не продвигались, пороги не снижались. L4 заморожен как
`SuggestOnly | Keep` и не получает edit authority.

Физически прошли GTK, Qt, Chromium, Kitty и ранее WeChat. Реальный Chromium
gap закрыт capability-preflight до D-Bus mutation dispatch. Telegram остался
`NOT_TESTED`, потому что безопасно наблюдаемого unsent-поля не было; сообщение
в реальный чат не отправлялось. Следующая и последняя release-операция:
`1.0.0` version sync, полный changed gate, remote release build, атомарная
установка, health/version/PID verification, commit и push.

**Release-операция выполнена.** `1.0.0` собран на удалённой машине за
`125.49 s` на `20` jobs, установлен с SHA parity, CLI/daemon/IBus engine/GNOME
extension показывают `1.0.0`. `lay-daemon` и `lay-l3-online` active, L1.1
sidecar ready, global IBus сохранил PID `3702`. Installed Chromium proof снова
дал `проверк`. Финальный receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/LAY_1_0_FINAL_2026-08-02.json`.

## Критический путь

```text
L2 ownership cleanup
-> первый принятый L3 delta
-> sentence-level L3
-> typed L4 receipts
-> cross-scene L4
-> performance
-> общий release gate
```

## 8. Release 1.0.17: Incremental DAFSA Completion

**Статус: выполнено 2026-08-10, release `1.0.17` установлен.** Холодный
completion-маршрут больше не пересчитывает полный фазовый/key-вектор отдельно
для каждой из `576` декодированных поверхностей. Один аккумулятор переносится
по DAFSA frontier с checkpoint/restore; на ребре добавляются только новые byte
4-grams, boundary-атомы добавляются временно только в terminal.

```text
remote release build                    133.16 s
build jobs                                     20
build peak RSS                     1,818,692 KiB
old full rescan                         8,427 us
incremental readout                     6,239 us
improvement                                25.97%
candidate parity                           exact
product candidate gate                26 / 26 PASS
installed binaries                    10 / 10 PASS
global IBus PID                       3702 -> 3702
active engine                         lay-ime-ru
```

L1.1 и L2 не перекристаллизовывались; лимиты `96 / 576`, candidate sources,
score, package identity и runtime authority не менялись. Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_DAFSA_INCREMENTAL_COMPLETION_1_0_17_2026-08-10.json`.

## 9. Release 1.0.18: first automatic L3 online admission

**Статус: live admission выполнен 2026-08-10.** Общая revision-схема
переоткрыла старый `WATCH` один раз после изменения proof pipeline, не добавляя
evidence и не снижая пороги. Live state перешёл `generation 2 -> 3`,
`admitted_deltas 0 -> 1`, pending `30 -> 29`. Targeted `2/2 + 5/5` и frozen
`41,064` differential прошли с пятью нулевыми regression counters. Delta
`4,372 B` сложен в inactive compact base `30,784,516 B`; опубликован delta-free
manifest `97 B`.

Release собран на `e@192.168.3.94` с `20` Cargo jobs за `141.56 s`, peak RSS
`1,776,368 KiB`, swap `0`. Global IBus PID `3702` и managed engine PID
`3950397` не менялись. Physical in-process refresh остаётся отдельным `WATCH`;
он не подменяется успешной manifest publication.

Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L3_ONLINE_PROOF_PIPELINE_REVISION_1_0_18_2026-08-10.json`.

## 10. Release Target 1.0.19: process refresh and exact rollback learning

**Статус: release 1.0.19 собран, установлен и физически проверен.** L3 получает
process-local watcher manifest с фоновой загрузкой и атомарной заменой
`Arc<L3CompositeMemory>`. Каждый клиент публикует отдельный status receipt в
`/run/user/1000/lay/l3-context/`; глобальный `ibus-daemon` для обновления модели
не нужен.

L4 получает единый причинный rollback-контур для daemon и IBus. Исторический
backfill принимает только точный цикл `lay_from -> lay_to -> lay_from`.
Текущие `172` receipts дали `176` token observations; вместе с `10` live
positives пакет содержит `186` joined observations, `16` profiles и `58` pair
profiles при размере `13,228 B`. Пакет остаётся `SuggestOnly`, automatic apply
невозможен.

Пройдены focused gates:

```text
L4 cross-scene                         13 / 13
typed rollback                          1 / 1
auto-undo contracts                    12 / 12
daemon rollback receipt                 1 / 1
IBus rollback path                       4 / 4
L3 process refresh                       2 / 2
atomic private status write              1 / 1
context phase                            88 / 88
L3 phrase gate                             8 / 8
L4 hidden state                            4 / 4
authority contract                       20 / 20
mutation monopoly                        15 / 15
unsafe-edit release gate              0 failures
```

Remote release build выполнен на `e@192.168.3.94` с Cargo `1.97.1` и `20`
jobs: `2:20.56`, средняя загрузка CPU `336%`, peak RSS `1,817,728 KiB`, swap
`0`. Все `10` переданных бинарников совпали с remote bundle:
`bc1ae669aeb6a9225d3e30bc133e09eec73a974073f61b080c93ccbf405c695b`.

Установленные daemon и managed IBus сохранили PID `128173` и `128200`, но оба
обновили L3 generation `1 -> 2`; refresh failures `0`. Глобальный
`ibus-daemon` сохранил PID `3702`, активный engine остался `lay-ime-ru`.
Установленный L4 пакет имеет `13,228 B` и SHA-256
`5a32cf50b94105679ec40bec7bd5c46c2937075ede864bd7961203427a6cf1b5`.

Органический promotion остаётся заблокирован: `10` positives и `4` conflict
scenes не образуют достаточный heldout/anti-ablation denominator. Это отдельный
evidence gate, а не незакрытая часть release `1.0.19`.

Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L3_PROCESS_REFRESH_L4_ROLLBACK_FEEDBACK_1_0_19_2026-08-10.json`.

Physical refresh receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L3_PROCESS_LOCAL_REFRESH_2026-08-10.json`.

## 11. L1.1 -> L2 Generative Morphology Closure

**Статус: productive birth реализован, promotion не закрыт.** Каноническая ось
сохраняет grounded target выше `95%` во всех `13` fixed damage classes. Новый
productive L2 уже композиционно находит лемму из типизированных
character/keyboard n-грамм и синтезирует ранее не сохранённую русскую словоформу
как `LemmaCenter + MorphologySlot + prefix/stem/suffix evidence`. Exact bank из
`1,875,032` форм остаётся evidence/verification bank, а не единственным
источником кандидатов.

Этот полиморфный контур пока остаётся `SuggestOnly`: строгий самостоятельный
`unique top-1 >95%` по каждому классу и `hot p99 <=5 ms` не доказаны, поэтому
runtime authority не изменена.

Текущий fixed `13 x 20,000` proof доказал target retention, но не strict
самостоятельный L2 readout. В частности:

```text
damage class                    L2 unique top-1
double substitution                    75.870%
non-adjacent transposition             77.330%
sparse multi-omission                  78.300%
```

Это не уничтожает L1.1 winner: правильная цель остаётся в bounded lattice и
может быть выбрана последующим L2/L3-контекстом. Однако такой результат не
разрешает объявить самостоятельный L2 readout новым владельцем восстановления.

Требуемое продолжение:

1. Представить продуктивную словоформу как `LemmaCenter + MorphologySlot +`
   `morpheme/ending evidence`, не требуя готовой exact surface в decoder bank.
2. Рождать ранее не сохранённые формы на лету из общего морфологического поля;
   сохранённые exact forms оставить evidence/verification bank, а не единственным
   источником кандидатов.
3. Обучать выбор морфем, слотов, positive/anti relations и tie calibration по
   evidence без word-specific runtime branches и ручных исключений.
4. Сохранять весь grounded L1.1 lattice и отдельные `Winner | Tied | ABSTAIN`;
   генеративный донор не имеет права вытеснить grounded candidate общей
   неопределённостью.
5. Провести fixed heldout proof отдельно для seen exact forms и truly unseen
   generated forms, включая все `13` классов повреждения.
6. Сравнить exact-bank-only и generative configurations по качеству, package,
   RSS, cold/hot latency и false authority до изменения runtime ownership.

**Promotion gate:**

```text
unique L2 top-1, каждый damage class       >95.0%
target retention, каждый damage class      >95.0%
clean preservation                         >=99.9%
ambiguity retention                        >=99.0%
false authority                                   0
false singleton                                   0
grounded L1.1 candidate loss                      0
hot p99                                      <=5 ms
package/RSS                      без непринятой регрессии
```

Текущий measured baseline и границы доказательства находятся в:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_CANONICAL_FIXED_RETENTION_13X20000_2026-08-10.json`.
Runtime authority этим пунктом пока не изменена.

Первый productive morphology micro proof уже выполнен на реальном V13:

```text
heldout target lemmas                         40
train/heldout overlap                          0
13 damage classes x 10                       130
context target-slot retention             90-100%
unseen generated top-16 retention          60-90%
unseen generated unique top-1              40-90%
false singleton                                 40
false authority                                   0
generated p99                              1.635 s
peak RSS                                  738.36 MiB
```

Verdict: `FAIL`, runtime authority не изменена. Общий дефект локализован в
slot-wide переборе suffix rules и неправильном порядке evidence, а не в
отдельных словах. Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEAVE_LEMMAS_OUT_MICRO_13X10_2026-08-10.json`.

Текущая работа: family-indexed longest-supported suffix postings и readout
`lemma evidence -> context slot -> family/profile -> surface geometry`, затем
повтор того же fixed denominator.

Второй micro proof (`V2`, debug, тот же `13 x 10`) отделил рождение леммы от
выбора окончания:

```text
target lemma retention, каждый класс        100.0%
generated top-16                           50-100%
generated unique top-1                     30-100%
false authority                                  0
admitted suffix profiles                 1,246,325
profile training                            97.319 s
RSS after training                     1,124,004 KiB
peak RSS                               1,182,320 KiB
generated p99                              707.402 ms
```

Verdict: `FAIL`, runtime authority не изменена. Family index оставлен как
bounded lookup, но debug timing нельзя напрямую сравнивать с release V1.
Первый общий дефект находится в выборе `MorphologySlot`: все положительные
слоты одного exact context получают одинаковый `scene_wave`, coherence `1000`
и насыщенный support `128`, то есть одинаковый score `1128` без anti-evidence.

Текущее продолжение V3:

```text
streamed train-only T rows
-> exclude selected heldout lemma names
-> project NUMBER/CASE/GENDER/PERSON/TENSE/MOOD/FORM_KIND
-> positive + anti context-slot posterior
-> Laplace suffix posterior
-> lemma x context-slot x suffix-profile x geometry
-> deterministic generated readout
```

Exact V2 receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_FAMILY_INDEX_V2_DEBUG_13X10_2026-08-10.json`.

V3 с marginal posterior и multiplicative joint evidence также выполнен и
отвергнут:

```text
target lemma retention, каждый класс        100.0%
generated top-16                           60-100%
generated unique top-1                     40-100%
false singleton                                  50
false authority                                    0
streamed T rows                                15,922
context modes / slots                       105 / 227
training                                      98.994 s
generated p99                                927.801 ms
peak RSS                                   1,184,804 KiB
```

Первый общий механизм: `T`-контексты являются multi-label. Другой положительный
слот в том же контексте не является anti-evidence для target slot. Например,
`они _` законно содержит несколько времён глагола и краткое прилагательное.
Поэтому marginal frequency и `context_total - positive` нельзя умножать на
lemma evidence как независимый posterior: частый допустимый слот вытесняет
редкий, но грамматически допустимый target.

Следующая конфигурация должна использовать set-valued compatibility lattice:
контекст отсекает доказанно несовместимые окончания, но не ранжирует несколько
совместимых окончаний по частоте учительского корпуса. Grounded L1.1 lemma и
геометрия повреждённой поверхности сохраняют приоритет. Настоящий anti-support
должен приходить из явных competitor observations, а не выводиться из других
positive labels.

Exact V3 receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_CONTEXT_POSTERIOR_V3_DEBUG_13X10_2026-08-10.json`.

V4 заменил marginal posterior на set-valued compatibility lattice:

```text
target lemma retention, каждый класс        100.0%
generated top-16                           90-100%
generated unique top-1                     60-100%
false singleton                                  17
false authority                                    0
generated p99                                672.383 ms
peak RSS                                   1,171,400 KiB
```

Это принятый архитектурный вывод, но ещё `FAIL`: `V3 -> V4` снизил false
singleton `50 -> 17`, поднял худший top-16 `60% -> 90%` и худший top-1
`40% -> 60%`. Оставшиеся top-16 потери сосредоточены в morpheme operator:
suffix-only transform не выражает bounded prefix/suffix comparative и теряет
часть возвратных глагольных форм. Следующий V5 расширяет оператор до
`prefix + retained stem + suffix`, не добавляя слов или фраз в runtime.

Exact V4 receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_COMPATIBILITY_V4_DEBUG_13X10_2026-08-10.json`.

V5 с bounded `prefix + retained stem + suffix` завершил productive birth на
micro denominator:

```text
target lemma retention, каждый класс        100.0%
generated top-16, каждый класс              100.0%
generated unique top-1                     50-100%
false singleton                                  16
false authority                                    0
admitted profiles                            1,267,969
training                                      104.809 s
generated p99                                774.631 ms
peak RSS                                   1,327,088 KiB
```

Механизм рождения ранее не сохранённой русской словоформы теперь подтверждён
на всех `13` damage classes этого micro. Оставшийся FAIL находится только в
readout: близкие леммы имеют конфликтующие независимые evidence, а часть
повреждённых поверхностей сама является допустимым словом. Следующий шаг не
меняет morpheme birth и не вводит веса; он формирует Pareto
`Winner | Tied | ABSTAIN`, передавая неоднозначный lattice в L3.

Exact V5 receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_EDGE_MORPHEME_V5_DEBUG_13X10_2026-08-10.json`.

V6 проверил глобальный Pareto-readout без изменения productive birth:

```text
generated top-16, каждый класс              100.0%
raw generated unique top-1                 50-100%
Pareto target retention                    90-100%
Winner / Tied / ABSTAIN                 108 / 21 / 1
false singleton                                  1
false authority                                    0
training                                      103.308 s
generated p99                                696.428 ms
peak RSS                                   1,326,524 KiB
```

Verdict: `FAIL`, runtime authority не изменена. Ошибка системная: productive
L2 сравнивал разные лемменные бассейны как собственные окончания и позволял
одной лемме уничтожить другую без независимого semantic evidence. Это нарушает
ownership: L2 вправе выбирать `MorphologySlot` внутри леммы, но межлемменный
выбор принадлежит grounded L1.1 evidence и L3. V7 ограничивает Pareto
доминирование одним `lemma_id`; разные леммы сохраняются как `Tied`.

Exact V6 receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_PARETO_READOUT_V6_DEBUG_13X10_2026-08-10.json`.

V7 исправил ownership без изменения birth: Pareto-доминирование теперь
разрешено только внутри одного `lemma_id`, а разные лемменные бассейны уходят
в L3 как `Tied`.

```text
generated top-16, каждый класс              100.0%
cross-lemma target retention                100.0%
readout selected-target                    90-100%
raw generated unique top-1                 50-100%
Winner / Tied / ABSTAIN                  0 / 129 / 1
false singleton                                  0
false authority                                    0
training                                      103.749 s
generated p99                                801.219 ms
peak RSS                                   1,326,744 KiB
```

Overall promotion verdict остаётся `FAIL`, потому что strict unique top-1 ещё
ниже `>95%`. Узкий результат: `PASS_shadow_retention`. L2 уже умеет родить и
сохранить неизвестную словоформу, но выбор между разными леммами и смысловыми
окончаниями должен делать L3 по контексту предложения. Runtime authority не
изменена.

Exact V7 receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEMMA_BASIN_PARETO_V7_DEBUG_13X10_2026-08-10.json`.

V8-V19 completed the compact productive sidecar, typed context-axis transport,
release query path, and real `NT` directional same-lemma relations. The sidecar
format is backward compatible and the current V2 artifact is:

```text
bytes                              81,688,382
rules                               1,268,215
context slots                          79,706
directional pair relations              7,191
same-lemma competitor surfaces          2,451
```

The old `13 x N` proof samples only `H` rows, while pair relations are trained
from `NT`. V20 therefore added a separate full `NH` directional denominator.
The first readout produced `892` target wins but `244` reverse false supports
and was rejected. Requiring two agreeing neighbor lanes reduced the error to
`24` but did not eliminate it, so V21 was also rejected.

V22 closed the directional ownership boundary without package
recrystallization:

```text
NH rows                               42,195
same-lemma comparisons                3,483
pair coverage                          42.463%
exact target wins                          69
reverse false supports                      0
tied/no-evidence                    1,410 / 2,004
directional verdict      PASS_shadow_directional_nh
```

Only exact independently observed competitor scenes may settle a morphology
slot. Left/right neighbor relations retain candidates but cannot create
authority; unresolved forms stay `Tied` for L3. Generated forms remain
`SuggestOnly`, runtime authority is unchanged, and the overall promotion verdict
remains `FAIL` because generated unique top-1 and hot latency are still below
their gates.

Exact V20-V22 receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_RAW_V20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_TWO_LANE_V21_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_EXACT_V22_13X10_2026-08-10.json
```

V23-V26 isolated the generated-form latency without shrinking any frontier.
V26 moves decoder-block decompression outside the shard mutex, rechecks the
cache after decode and raises retained decoded forms from `16,384` to `65,536`.
The unchanged `13 x 10` denominator produced byte-identical class and
directional summaries versus V25:

```text
configuration                 workers   generated birth p50 / p99
V25 source cache                    1                3.569 / 7.335 ms
V25 source cache                   20              17.403 / 93.280 ms
V26 decoder cache                   1                3.218 / 8.336 ms
V26 decoder cache                  20              16.115 / 73.670 ms

V26 workers=20 wall / peak RSS                         6.17 s / 358,136 KiB
V26 workers=1 wall / peak RSS                          6.74 s / 337,016 KiB
class summary SHA               d4aec55925b462c54a6a1004e1e3faba0f2366a85e306f4f2c6bd2b5cfa0dcdf
directional summary SHA         72fbbbd2b9205a7cc895a432e789b219cbc7b5daab35d2f373c15a7ec2d307f0
false authority / singleton                                             0 / 0
```

V26 is rejected as the final speed solution. It reduces the contended
20-worker tail by `21.0%`, but misses the `<=5 ms` gate by `14.7x` and regresses
the single-worker p99. Runtime authority remains unchanged.

V27 replaced the associative LRU with a direct-mapped O(1) `RwLock` cache. It
preserved byte-identical class/directional summaries and zero false authority,
but collision-thrashing regressed generated p99 to `10.118 ms` on one worker
and `218.109 ms` on twenty workers. V27 is rejected and must not be promoted.
The next cache contour must preserve full shard associativity while removing
linear hit lookup and hit-time mutation.

V28 preserved full shard associativity with `HashMap block -> slot`, read-only
hits and round-robin miss eviction. It also failed: generated p99 was
`9.637 ms` on one worker and `107.674 ms` on twenty workers. V29 then forced
the inner Rayon pool to one thread without changing code. This raised p99 to
`34.380 ms` for one outer worker and `366.065 ms` for twenty outer workers.
Therefore neither cache replacement nor removal of inner parallelism closes the
budget. V26 remains the best storage baseline; the next optimization belongs
inside repeated rule expansion, surface construction and geometry scoring.

V30 kept the V26 decoder and cached generated-surface geometry inside each
lemma basin while replacing deterministic post-generation tree dedup with hash
dedup followed by the existing total sort. Class and directional summaries
remain byte-identical. One-worker p99 improved `8.336 -> 7.705 ms`; twenty-worker
p99 improved only `73.670 -> 73.085 ms`. V30 is retained as a lossless local
optimization, but the overall verdict remains `FAIL_latency`.

V31 repeated the complete generated proof inside the same process before the
measured pass. A fully warm one-worker pass remained `7.727 ms` p99 versus
`7.705 ms` cold, while the warm concurrent pass regressed to `172.221 ms`.
Cold canonical-source materialization is therefore rejected as the dominant
blocker; embedding those sources into the sidecar would add storage without
closing latency. The diagnostic warmup is not part of runtime.

V32 reuses generated geometry across lemma tasks on each Rayon worker, builds
family suffix lanes once per source and reads encoded family specificity
directly. Quality remains byte-identical. One-worker p99 improved to
`6.866 ms`; twenty-worker p99 improved to `71.872 ms`. The optimization is
retained, but both modes still fail `<=5 ms`.

V33 removes repeated Unicode lowercase work and a duplicate normalized
`String` allocation from generated-surface geometry without changing the
bounded `256 / 256 / 16 / 32 / 196608` frontier. Its class, directional,
safety and package summaries are exactly equal to V32. The fixed `13 x 10`
proof measured `2.838 / 6.987 ms` on one worker and `13.923 / 65.649 ms` on
twenty workers. Concurrent p99 improved by `8.7%`, but one-worker p99 regressed
by `1.8%`; therefore the verdict is `PASS_parity_FAIL_latency`, runtime
authority remains unchanged, and V33 is not the final speed solution. Peak RSS
was `336,852 KiB` and `382,252 KiB`, respectively. The larger fixed proof,
clean/ambiguity preservation, L3 handoff and installed-client behavior were not
tested by V33.

V34 tested an exact top-32 upper bound inside each selected family range. It
preserved exact V33 quality and `0 / 0` false authority/singleton, but regressed
generated p99 to `7.649 ms` on one worker and `69.901 ms` on twenty workers.
The code was removed. Sidecar inspection established why: exact family ranges
have p99 `1`, maximum `3` rules and no range above `32`; the real fanout is the
parent transition level, where `1,183` transitions contain mean `1,072`, p99
`20,301` and maximum `28,437` family variants. Verdict:
`REJECT_wrong_bound_level`; runtime authority remains unchanged. The next bound
must operate over complete lemma/slot hypotheses, not individual selected
family rules.

V35 tested the same proof at whole-lemma level: a sequential seed top-32
provided a mathematically safe joint-evidence cutoff before the remaining
parallel lemmas. Quality stayed exactly equal to V33 and false authority /
singleton remained `0 / 0`, but p99 regressed to `7.939 ms` on one worker and
`71.205 ms` on twenty workers. The seed serialization cost exceeded the removed
work, so V35 was removed with verdict `REJECT_seed_serialization`. The next
speed change requires a function-level CPU profile of the restored V33 path.

The V33 `perf` profile used the normal stripped proof binary over the fixed
`13 x 100` denominator. It located the first systemic hot path instead of
another frontier experiment:

```text
damerau_levenshtein_rows                         16.31% CPU cycles
allocator family                       approximately 22-24%
context_slot_evidence_for                         4.02%
decoder block cache                               3.16%
bounded_context_key                               2.30%
```

V36 retained the full `256 / 256 / 16 / 32 / 196608` contour and removed
per-comparison geometry allocations by reusing worker-local Damerau rows,
keyboard event buffers and normalized surface storage. Keyboard distance is
now evaluated only after the character score remains competitive. The fixed
`13 x 100`, one-worker proof preserved byte-identical class and directional
summaries and `0 / 0` false authority/singleton:

```text
configuration                         p50       p99       peak RSS
V33 perf baseline                   3.018     6.970 ms   336,612 KiB
V36 reusable geometry scratch       2.812     6.403 ms   337,016 KiB
```

Verdict: `PASS_lossless_geometry_optimization`, but overall
`FAIL_latency`. The measured V36 twenty-worker `13 x 10` sample remained noisy
at `11.953 / 143.717 ms`; no multi-client promotion is claimed.

V37 moved morphology-slot preparation into a worker-local cache. It preserved
quality and `0 / 0`, but repeated context-key work and cache ownership made the
one-worker micro p99 regress to `8.165 ms`. The experiment was removed with
verdict `REJECT_worker_local_slot_cache`.

V38 prepares one immutable `target_features_by_source` map per request before
the lemma `par_iter`, so all workers consume the same bounded slot evidence.
The fixed `13 x 100` class and directional summaries remain byte-identical to
V33/V36:

```text
generated birth p50 / p99                 2.328 / 5.885 ms
steady / peak RSS                       315,644 / 336,692 KiB
directional pair coverage                         42.463%
directional target wins / reverse false              69 / 0
false authority / singleton                           0 / 0
```

This is the current accepted source baseline. It improves V33 p50 by `22.9%`
and p99 by `15.6%`, but remains `0.885 ms` above the `<=5 ms` gate. The
twenty-worker `13 x 10` sample measured `9.974 / 85.832 ms`, so concurrent
latency also remains unpromoted.

V38 does not change the unresolved quality denominator:

```text
gate                                      required     current worst
generated top-16, every class             >95%         94%
generated readout target retention        >95%         91%
generated unique top-1, every class       >95%         61%
false authority / singleton               0 / 0         0 / 0
```

Not tested by V36-V38: the larger fixed denominator, clean/ambiguity
preservation after live generated-candidate integration, final L3 contextual
selection, daemon/IBus latency and physical apply authority. Runtime authority
changed: `false`; generated forms remain `SuggestOnly`.

V39 tested three lossless hot-path changes together: exact equality exits before
geometry construction, Damerau rows use the shorter sequence as their column
axis, and the tiny generated-family result set uses linear `Vec` dedup instead
of allocating a per-call surface `HashMap`. The justification for the last
change is measured sidecar structure, not an assumed bound: exact family ranges
have p99 `1` and maximum `3` rules.

The V39 class and directional summaries are byte-identical to V38 on both
`13 x 10` and `13 x 100`; false authority/singleton remain `0 / 0`. Four
sequential one-worker `13 x 100` runs measured:

```text
run                           p50       p99       peak RSS
1                           2.207     6.092 ms   336,852 KiB
2                           2.292     7.027 ms   336,648 KiB
3                           2.244     5.711 ms   337,012 KiB
4                           2.236     5.819 ms   337,016 KiB
V38 reference               2.328     5.885 ms   336,692 KiB
gate                                  <=5.000 ms
```

V39 consistently improves p50 but does not close p99; its four-run median p99
is approximately `5.96 ms`. The twenty-worker `13 x 10` probe measured
`12.976 / 59.952 ms`, also above budget. Verdict:
`PASS_quality_parity_FAIL_latency`. V39 remains a profiling candidate, not a
runtime promotion; authority changed: `false`.

An unstripped V39 metrics build then provided the next function-level profile:

```text
damerau_levenshtein_rows                         19.78%
Unicode conversion lookup                         9.52%
compact decoder block cache                       4.81%
generate_forms_prepared                           4.21%
bounded Damerau rows                              4.11%
UTF-8 conversion                                  3.85%
RU key mapping                                    3.60%
allocator family                         approximately 15%
```

V40 tested the apparent Unicode/keyboard opportunity without changing L2
semantics. The canonical keyboard mapper gained a lowercase RU fast path and a
reusable encoded key-unit output, removing the intermediate `Vec<KeyEvent>`
from generated geometry. Local keyboard/compositional/productive/format tests
passed `36 / 36`, and remote class/directional hashes remained byte-identical
to V38/V39 with `0 / 0` false authority/singleton.

Four sequential one-worker `13 x 100` release proofs rejected the optimization:

```text
run                           p50       p99       peak RSS
1                           2.282     6.054 ms   337,016 KiB
2                           2.256     6.213 ms   336,968 KiB
3                           2.236     6.331 ms   336,852 KiB
4                           2.285     6.228 ms   336,692 KiB
V39 median                  ~2.240    ~5.956 ms
gate                                  <=5.000 ms
```

The twenty-worker V40 micro also regressed to `15.114 / 97.993 ms`. Verdict:
`REJECT_lowercase_key_units_no_release_tail_gain`. The metrics-profile Unicode
cost was not the release p99 owner; V40 is removed and runtime authority remains
unchanged.

V41 then targeted the dominant exact Damerau rows by trimming equal prefix and
suffix units before DP while preserving the original similarity denominator.
An exhaustive parity test compared exact and bounded OSA distance against the
untrimmed heap reference for every pair of ternary strings through length `4`;
all local compositional/productive/format tests passed. Remote class and
directional hashes again remained byte-identical with `0 / 0` safety.

Release latency rejected the change:

```text
run                           p50       p99       peak RSS
1                           2.247     6.092 ms   337,016 KiB
2                           2.228     5.834 ms   337,000 KiB
3                           2.277     6.334 ms   336,868 KiB
4                           2.286     6.532 ms   336,856 KiB
V39 median                  ~2.240    ~5.956 ms
```

Verdict: `REJECT_common_edge_trim_no_release_tail_gain`. V41 is removed;
runtime authority remains unchanged. The next speed experiment moves to
bounded parallel lemma chunks, grounded by `crossbeam steal = 3.64%` and the
per-lemma closure cost in the V39 profile, without reducing any frontier.

Exact V39 receipt directory:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_SMALL_DEDUP_V39_2026-08-10/`.

Exact V40 receipt directory:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LOWER_KEY_UNITS_V40_2026-08-10/`.

Exact V41 receipt directory:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_COMMON_EDGES_V41_2026-08-10/`.

V42 tested deterministic bounded lemma chunks after V41 was removed. It
preserved the V39 class/directional hashes and `0 / 0` false
authority/singleton, but four one-worker `13 x 100` runs measured p99
`6.602 / 6.343 / 7.735 / 8.179 ms`; the twenty-worker `13 x 10` p99 was
`70.105 ms`. Verdict:
`REJECT_bounded_lemma_chunks_no_tail_gain`. V42 does not become the source
baseline.

The scheduler-level optimization series is now closed. The canonical next
design is a learned `LemmaParadigmBinding -> ParadigmCenter -> MorphologySlot`
field with one shared exact character/keyboard geometry traversal over an
implicit productive form graph. Full design authority, formulas, ownership,
one-pass crystallization, package requirements, proof gates, rejected routes,
and the seven-step delivery route are recorded in:

`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-canonical-design.md`.

The paper implementation is complete at specification level. Exact typed
identities, prefix-trie traversal, path-correct OSA state, evidence objective,
disjoint calibration, protected grounded/productive lanes, wire format, deltas,
concurrency ownership, numeric budgets, and proof protocol are recorded in:

`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-implementation.md`.

Paper completeness and original-review closure are recorded in:

`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-review-2026-08-10.md`.

Exact V42 receipt directory:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_BOUNDED_LEMMA_CHUNKS_V42_2026-08-10/`.

Exact V26 receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DECODER_CACHE_V26_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DECODER_CACHE_V26_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DECODER_CACHE_V26_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECT_CACHE_V27_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECT_CACHE_V27_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_ASSOCIATIVE_CACHE_V28_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_ASSOCIATIVE_CACHE_V28_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_PARALLELISM_V29_OUTER20_INNER1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_PARALLELISM_V29_OUTER1_INNER1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_CACHE_V30_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_CACHE_V30_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WARM_REPEAT_V31_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WARM_REPEAT_V31_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WORKER_CACHE_V32_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WORKER_CACHE_V32_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_NORMALIZED_GEOMETRY_V33_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_NORMALIZED_GEOMETRY_V33_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_NORMALIZED_GEOMETRY_V33_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_BOUNDED_EXPANSION_V34_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_BOUNDED_EXPANSION_V34_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_BOUNDED_EXPANSION_V34_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEMMA_BOUND_V35_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEMMA_BOUND_V35_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEMMA_BOUND_V35_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V33_PERF_SELF_2026-08-10.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V33_PERF_WORKERS1_13X100_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V33_PERF_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_SCRATCH_V36_WORKERS1_13X100_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_SCRATCH_V36_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_SCRATCH_V36_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_SLOT_CACHE_V37_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_SLOT_CACHE_V37_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_SLOT_CACHE_V37_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GLOBAL_SLOT_CACHE_V38_WORKERS1_13X100_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GLOBAL_SLOT_CACHE_V38_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GLOBAL_SLOT_CACHE_V38_BUILD_2026-08-10.time.txt
```

Next required work remains conjunctive:

1. restore V39 source behavior while preserving V42 receipts;
2. implement the paper package and prefix-trie traversal in exact parity mode;
3. fit train-only evidence and disjoint calibration without raw-corpus rescans;
4. pass strict per-class `SLOT_HELDOUT` and `LEMMA_HELDOUT` gates while
   preserving zero false authority/singleton and every grounded L1.1 candidate;
5. pass numeric package/RSS and single/multi-client latency ceilings;
6. run L3 handoff, verifier replay, and the physical product matrix;
7. do not promote or restart daemon/IBus before all gates pass.

## 12. V63 Productive Morphology Decision Point

**Статус: V63 собран и измерен, promotion отклонён; следующая версия не
запущена.**

V63 переиспользовал завершённые raw/sorted/reduced артефакты и не пересобирал
L1.1 или canonical L2. Полная reinduction заняла `38:31.45`, peak RSS был
`611,204 KiB`. Получен mmap-пакет:

```text
bytes       17,309,944
sha256      5b80513cb33d3b82b4b9829742ecab6e4fc3248694f215d252901b630b122238
authority   shadow SuggestOnly, без изменения runtime owner
```

Диагностический proof `13 classes x 100 x 2 cohorts`, `19 workers`:

```text
cohort          L lemma    S exact    top-1    top-16    readout    empty
SEEN_EXACT      100.00%     99.77%    35.00%    99.77%     99.77%       0
LEMMA_HELDOUT    96.08%     92.08%     3.08%    90.38%     92.08%      51
```

V63 закрыл основной V62 coverage-дефект: `LEMMA_HELDOUT` exact birth вырос с
`7.77%` до `92.08%`, а empty lattice уменьшился с `1,183` до `51`. Однако
строгий gate не пройден. Худшая цепочка у suffix truncation:
`L=93% -> S=76% -> top-16=63%`; максимальный class p99 `144.976 ms` против
`<=5 ms`.

Safety в измеренной области:

```text
Winner / Tied / ABSTAIN      0 / 0 / 2,600
false singleton             0
integrity errors             0
unsupported false authority NOT TESTED
runtime authority changed   false
```

Открытый proof-дефект: отдельный `B=true paradigm retained` не измеряется;
`SLOT_HELDOUT`, `MULTI_LABEL`, `UNSUPPORTED`, grounded L1.1 protection,
L3/L4 transfer, queue-inclusive multi-client latency и physical matrix также
не пройдены. Поэтому V63 остаётся архитектурной основой, но не продуктовым
owner.

До обсуждения V63 запрещены имя следующей версии, код, release build,
reinduction и полный proof. После обсуждения любой следующий эксперимент
сначала обязан выполнить pre-build gate из раздела 22 paper implementation.

Полные receipts:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V63_COLD_BINDING_2026-08-11/`.

Обязательный pre-build review до следующего кода:

`/home/ubu/projects/lay/docs/l2-productive-post-v63-prebuild-review.md`.

## 13. V64 Surface-Basin Decision Point

**Исторический статус точки: V64 собран и измерен; механизм принят, promotion
отклонён. Последующий результат V65 находится в разделе 14.**

После явного одобрения V64 реализовал coalescing по
`(lemma_id, target_slot_id, normalized_surface)` до global top-32, не меняя
`16 / 32`, коэффициенты, authority, SafetyGate или verifier. Полный raw corpus
и transition induction не запускались: пакет resumed из frozen V63 work-dir за
`64.00 s`.

```text
package bytes      17,309,944
package sha256     9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438
determinism        byte-identical repeat
proof              13 x 100 x 2, workers 19
probe parity       2,600 / 2,600
false singleton   0
integrity errors   0
runtime authority changed false
```

Главный результат:

```text
LEMMA_HELDOUT  1,300 -> H 1,280 -> B 1,219 -> S0 1,219
                     -> S1 1,219 -> S2 1,219 -> S3 1,219 -> R 1,219
```

Surface basin исправил измеренный representational crowding: exact birth
`1,197 -> 1,219`, top-16 `1,175 -> 1,218`, raw top-1 `40 -> 267`, package size
без роста. После исправления proof identity с `(lemma, paradigm)` на
`(lemma, POS, paradigm)` остаточный системный owner локализован до compatibility
birth: `20` случаев вне target-POS `H`; из оставшихся `61` теряются на `H -> B`,
в том числе `59` из-за отсутствия oracle paradigm в postings оставшихся
source-slot и `2` из-за exact exposed-form reconstruction. `B -> S0` и вся
цепочка `S0 -> S1 -> S2 -> S3 -> R` потерь не дают. Старые знаменатели
`H=1,288`, `B=1,249` и `30` потерь `B -> S0` superseded и не являются
характеристикой V64.

Promotion запрещён: strict per-class top-1/retention gates и p99 `<=5 ms` не
пройдены; `SLOT_HELDOUT`, `MULTI_LABEL`, `UNSUPPORTED`, интегрированный
L1.1/L3/L4/verifier, queue-inclusive и physical product gates не измерены.
Следующая версия до отдельного paper review и уведомления пользователя не
назначается.

Полные receipts:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/`.

Авторитетный corrected receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/hbs0-pos-diagnostic-13x100-receipt.json`.

Следующий разрешённый shadow-only micro и его причинные gates зафиксированы в:

`/home/ubu/projects/lay/docs/l2-productive-post-v64-anchor-recovery-paper.md`.

## 14. V65 Anchor-Recovery Decision Point

**Статус: V65 собран и полностью измерен; representation bounded, causal
механизм улучшен, fixed proof не пройден; интеграция запрещена.**

V65 добавил отдельный mmap sidecar `3,514,208 B` с `36,915` обученными
reverse-anchor paths. Основной пакет остался byte-identical V64:
`17,309,944 B`, sha256
`9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438`.
Установленный Lay, активные пакеты, daemon и IBus не менялись.

Полный fixed proof `13 x 100 x 2` дал:

```text
LEMMA_HELDOUT  1,300 -> H 1,281 -> B 1,277 -> S0 1,277
                     -> S1 1,277 -> S2 1,277 -> S3 1,277 -> R 1,277
top-16        1,276
raw top-1       205
outside H        19
H -> B            4
B -> S0           0
probe parity   2,600 / 2,600
false singleton 0
integrity errors 0
runtime authority changed false
```

Относительно V64 `B/R` вырос `1,219 -> 1,277`, а `H -> B` уменьшился
`61 -> 4`. Но эксперимент отклонён: четыре системных промаха остаются, oracle
`H` сам изменился `1,280 -> 1,281`, raw top-1 регрессировал `267 -> 205`,
suffix-truncation top-16 равен только `95.0%`, а LEMMA_HELDOUT latency вырос до
`p50 19.009-28.269 ms`, максимального `p99 255.827 ms` и max `483.606 ms`.
Полный proof занял `1,951.507 s` против `76.812 s` у V64. Peak proof RSS
`235,668 KiB`, package budget и false-authority invariants пройдены.

Следующий код до нового paper запрещён. Сначала требуется бумажно закрыть три
отдельных owner-дефекта:

1. frozen-H proof: базовый hypothesis denominator не должен зависеть от нового
   recovery lane;
2. bounded recovery birth: нельзя раскрывать все `(POS, source slot)` paths и
   повторно проигрывать полные парадигмы на каждом запросе;
3. rank preservation: новые recovery bindings не могут вытеснять уже
   удержанный target из top-16/top-1 без независимо измеренного evidence.

V64 остаётся канонической принятой архитектурной точкой. V65 сохраняется как
`REJECT_not_closed_and_not_latency_bounded`; shadow/L1.1/L2/L3, daemon/IBus и
live-owner route не подключаются.

Полный paper и receipts:

`/home/ubu/projects/lay/docs/l2-productive-post-v64-anchor-recovery-paper.md`

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V65_ANCHOR_RECOVERY_2026-08-11/`

## 15. V66 Frozen-H and Bounded-Recovery Decision Point

**Статус: paper завершён и подвергнут критике; разрешены только локальная
реализация инвариантов и shadow micro `13 x 10 x 2`. V66 ещё не собран.**

V65 отклонён, V64 остаётся канонической точкой. Новый paper разделяет пять
owner-задач, которые V65 смешивал в одном широком маршруте:

```text
frozen V64 H manifest
-> observed-slot intersection до reverse execution
-> generic syncretic identity bridge
-> dedup (paradigm, anchor) до exact replay
-> exposed-slot-only replay для direct и recovery lanes
-> calibrated rank-preserving readout
```

Read-only разбор всех `159` index records V65 `.p2r` дал:

```text
posting fan-out min/p50/p75/p90/p95/p99/max
                    1/211/271/386/418/543/641
mean                                      232.170
posting ranges contiguous                    true
sum fan-out                               36,915
```

Это подтверждает системный дефект: V65 раскрывает сотни paths на каждый
наблюдаемый source до проверки полной slot-совместимости. Новый route обязан
сначала пересечь независимый structural slot-license index, затем объединить
direct/recovery/identity birth, выполнить только отфильтрованные reverse
programs, дедуплицировать `(paradigm_id, recovered_anchor)` и проиграть только
программы реально открытых slots. Старые compatibility/recovery postings не
могут быть eligibility universe: оставшаяся oracle paradigm отсутствует именно
в них, и такой intersection заблокировал бы identity bridge тем же дефектом.

`H` больше не вычисляется экспериментальным runtime. Frozen manifest привязан
к V64 package sha256
`9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438` и
proof spool sha256
`6e282474b26bf90dc61ee21c93c9dd7dd727c29a2b02650c513ffdd06746e807`;
его знаменатель обязан быть ровно `1,280`. Oracle IDs остаются только в proof
comparator и запрещены в runtime birth/score/readout.

Generic identity bridge рождает anchor без изменения поверхности только из
структурного `COPY_ALL -> TERMINATE` перехода обученной парадигмы. Он не хранит
слова и не получает Winner authority. Все открытые формы всё равно должны
воспроизводиться точно; неидентифицируемые варианты остаются `Tied/ABSTAIN`.

Rank preservation формализован относительно V64. Base-кандидаты сохраняют
свой порядок; recovered-кандидат может их обойти только с независимо обученным
и откалиброванным cross-lane certificate. Без него base-order сохраняется, а
новая форма остаётся дополнительным lattice evidence. Минимальный raw top-1
gate равен V64 baseline `267`, а не вручную выбранной квоте.

Proof scheduler меняется с contiguous chunks по `137` cases на bounded dynamic
queue: target-blind cost descending, atomic single-case claims, запись результата
по исходному ordinal и детерминированный reduce. Это исправляет длинный
single-worker tail, но не считается улучшением request latency.

Обязательные V66 gates:

```text
frozen H                    exactly 1,280
H -> B                      0
B -> S0                     0
raw top-1                   >=267
each class top-16           >95.0%
false singleton             0
integrity errors            0
probe parity                exact
BaseProjection parity       exact V64
uncertified demotions       0
maximum class p99           <=5.000 ms
runtime authority changed   false
```

Критический вывод paper: одной оптимизации recovery sidecar недостаточно.
У V64 maximum class p99 уже `86.001 ms`, поэтому exposed-slot-only execution
должен заменить complete-trie replay одновременно в direct и recovery paths.
Это разрешено только при точной semantic parity. Full build, full proof,
installation, daemon/IBus и live ownership до PASS предыдущего gate запрещены.

Полный paper:

`/home/ubu/projects/lay/docs/l2-productive-post-v65-bounded-recovery-paper.md`

Fan-out receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/fanout-profile.json`

## 16. Deferred IME Tail Rebirth After Manual Continuation

**Статус: исправление реализовано и прошло software gate; physical gate новой
release остаётся открытым до проверки установленного IME.**

Наблюдаемый дефект:

```text
раб{отает}
-> пользователь вводит `о`, а не принимает подсказку через Tab
-> новый prefix: рабо
-> старый target "работает" ошибочно удерживает хвост {тает}
```

Требуемый общий контракт:

```text
visible completion target T0 for prefix P0
-> printable input without Tab
-> T0 is not accepted
-> record censored feedback for T0, never negative learning evidence
-> build the complete candidate field again for the new prefix P1
-> rerank morphology and context alternatives
-> publish a fresh suffix or clear the suffix
```

Совпадение введённой буквы с первой буквой старого хвоста не является
подтверждением всего старого target. Подтверждением остаётся явный `Tab` либо
допечатывание полной поверхности. Fixture `раб{отает} -> рабо{тают}` доказывает
инвариант, но слова и окончания запрещено переносить в runtime-условия.

Перед исправлением нужно найти первый слой потери по маршруту:

```text
printable IBus event
-> PreeditFastState visible target lifecycle
-> prediction feedback classification
-> candidate rebirth for the new prefix
-> Productive V90 morphology lattice
-> shared L3 rerank
-> visible suffix publication
```

Gate задачи:

- старый target не закрепляется только из-за совпавшей следующей буквы;
- новый prefix всегда получает новый bounded readout;
- morphology alternatives могут сменить окончание;
- отсутствие `Tab` остаётся censored, а не anti-evidence;
- exact completion, Space/autocorrect и double-Shift rollback не регрессируют;
- исправление проходит агрегатный IME proof, а не список слов по одному;
- hot printable path сохраняет текущий latency budget.

### 16.1 Measured result, 2026-08-12

Trace одного физического воспроизведения локализовал первый наблюдаемый факт:

```text
prefix пров     -> 12 candidates -> top suffix ерить -> IBus update
prefix прове    -> 12 candidates -> top suffix рить  -> IBus update
```

Следовательно, `прове{}` не был потерей candidate birth: новый bounded readout
существовал. Дефект находился в lifecycle публикации preedit. Старый код посылал
`ShowPreeditText` раньше `UpdatePreeditText`; теперь один output owner сначала
атомарно устанавливает свежий payload через `UpdatePreeditText(..., true)`, а
затем посылает `ShowPreeditText`. Inactive и composition routes используют один
helper.

Что протестировано на `e@192.168.3.94`, `20` logical CPU:

- `cargo fmt --check`: PASS;
- focused fresh-rebirth contract: `1/1` PASS;
- focused publication-order contract: `1/1` PASS;
- previous target invalidation contract: `1/1` PASS;
- полный последовательный `lay-ibus-engine`: `183/183` PASS;
- Cargo target после gate: `9,191,882,752 / 12,884,901,888` bytes.

Что не тестировалось этим gate:

- физическая видимость нового хвоста после установки release;
- правильность конкретного morphology top-1 (`проверить` против `проверка`) без
  достаточного контекста. Кандидатный порядок L2/L3 этой правкой не изменён.

Runtime authority changed: `false`. Изменён только IBus output lifecycle.

Release `1.0.22` собран на удалённой машине за `186.62 s`, установлен и активен.
Глобальный `ibus-daemon` сохранил PID `3702`; перезапущены только управляемые
Lay daemon/engine. Productive V90 `.p2m/.p2r` mmap-пакеты остаются загружены
обоими процессами. Rollback:

`/home/ubu/.local/lib/lay/rollback/1.0.21-pre-preedit-20260812-134744`

Receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_PREEDIT_ATOMIC_REBIRTH_2026-08-12.json`

## 17. Unified IME Route Release Gate, 2026-08-12

Status: `RELEASE_1.0.23_INSTALLED_PHYSICAL_SMOKE_PASS`.

Completed:

- one live CandidateGate owns completion and replacement display;
- exact completion, damaged-prefix, layout, Productive V90 morphology,
  boundary, L3 and L4 evidence enter one typed bounded lattice;
- replacement is explicit-Tab-only and passes the common edit verifier;
- active exact forms stay open to morphology ending changes;
- settled clean forms remain closed to weak extension;
- morphology-slot identity preserves bounded lattice diversity without
  granting authority;
- broad per-key Cyrillic-to-English settlement was removed;
- boundary evidence now short-circuits after cheap exact-prefix evidence;
- focused damaged-prefix hot latency is approximately `3.1-3.4 ms`;
- `остан` unique cache miss improved `303.646 ms -> 9.902 ms`;
- remote changed-code, authority, mutation monopoly, input and IBus gates pass;
- full baseline comparison has zero new failure names.

Completed release actions:

1. graphify updated;
2. version bumped `1.0.22 -> 1.0.23`;
3. remote release build completed in `204.18 s`, maximum RSS
   `2,381,764 KiB`;
4. binaries installed and only Lay-managed processes restarted;
5. global `ibus-daemon` PID preserved at `3702`;
6. installed CLI and GNOME runtime report `1.0.23`;
7. Productive V90 mmap confirmed in daemon and IME engine.

Measured live main-contour RSS is `1,097,204 KiB`: daemon `399,952`, IME
`386,364`, L1.1 serve `306,504`, L3 online `4,384`.

Remaining physical gate:

1. replacement via explicit `Tab`;
2. morphology ending rerank after another printable letter;
3. responsive `Space`;
4. autocorrect rollback via double `Shift`.

Broad physical smoke is `PASS`: after installation the user reported that real
typing works very well overall. The four scenarios above remain detailed
per-contract observations, not blockers for the published `1.0.23` release.

Rollback:

`/home/ubu/.local/lib/lay/rollback/1.0.22-pre-1.0.23-20260812-194801`

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_UNIFIED_CANDIDATE_FIELD_2026-08-12.json`

## 18. Canonical Target-Authority Migration

Active implementation worktree:

`/home/ubu/projects/lay-l1-exact-peak-search`

```text
[PASS] Slice 0: immutable baseline freeze
[PASS] Slice 1: common bounded target-evidence vocabulary
[PASS] Slice 2a: deterministic enumeration-work budgets
[PASS] Slice 2b: context-neutral prepared material and exact frame shadow
  [PASS] 1,300/1,300 deterministic material pairs
  [PASS] 3,864/3,864 bindable frame targets
  [PASS] stale-frame accepts 0/3,864
  [BLOCKED FROM AUTHORITY] 877/1,300 UPSTREAM_INCOMPLETE
  [NOT CLAIMED] hot-path latency, live authority, deployment
[PASS] Slice 3: frame-bound candidate validity shadow
  [PASS] 3,864/3,864 candidate-state derivations
  [PASS] false grounding 0
  [PASS] cross-context and stale-frame accepts 0
  [PASS] original preservation 3,900/3,900 outside target set
  [NOT CLAIMED] Productive traversal p99, live authority, deployment
[PASS] Slice 4: complete conflict cohort shadow
  [PASS] 3,900/3,900 deterministic cohort derivations
  [PASS] Winner/Tied/ABSTAIN = 0/1,050/2,850
  [PASS] incomplete Winner, false singleton, lost grounded target = 0
  [PASS] multiple-component authority and preservation bypass = 0
  [NOT CLAIMED] useful Winner coverage, hot-path latency, live authority
[PASS scoped] Slice 5: missing-target birth and retention
  [PASS] typed operator implementation and exact work accounting
  [PASS] first real-package smoke measured; budget 8/8, overflow 0, authority 0
  [FAIL] canonical-L2-only birth/retention 4/8
  [PASS] exact identity union birth 8/8, work 8/8, authority 0
  [PASS] frozen 8-surface contour storage lane; retention 8/8, born-only 8/8
  [PASS] remote 13x100 material target retention 1280/1280, false singleton 0
  [PASS] no latency regression: contour 11.157 ms vs paired baseline 12.533 ms
  [OPEN release blocker] inherited absolute 20-worker p99 remains >5 ms
[PASS scoped] Slice 6: Boundary internalization shadow
  [PASS] exact BoundarySplit and BoundaryMerge enumeration
  [PASS] two-sided package grounding and separator-only geometry
  [PASS] two-surface reserve, dedup and whole-field StorageCapacity overflow
  [PASS] boundary witnesses remain Born; automatic authority grants 0
  [PASS] remote 13x100 H/B/S0 = 1280/1280/1280, false singleton 0
  [OPEN release blocker] maximum class p99 11.238 ms > 5 ms
  [UNCHANGED] live authority, daemon, IBus, packages and installed 1.0.33
[IN PROGRESS] Slice 7: crash-safe event transaction
  [PASS] Slice 7A state machine, fault matrix and focused tests 8/8
  [REJECTED] buffered ext4 write_at + fdatasync
  [PASS] prepare/co-commit p99 1.265/1.164 ms <= 2 ms
  [FAIL] prepare maximum 325.387 ms >= 8 ms
  [REJECTED] DirectAlignedSlotCommitV1 O_DIRECT + O_DSYNC
  [PASS] direct prepare/co-commit p99 1.657/1.575 ms <= 2 ms
  [FAIL] direct prepare/co-commit max 330.324/162.240 ms >= 8 ms
  [UNCHANGED] live authority, daemon, IBus, packages and installed 1.0.33
  [NEXT] BackendAtomicReceiptV1 design receipt and kill-point proof
[PENDING] Slice 8: lexical live readout
[PENDING] Slice 9: separately calibrated context authority
[PENDING] Slice 10: compatibility-route removal
[PENDING] Slice 11: performance and failure proof
[PENDING] Slice 12: versioned physical release
```

Slice 2 is closed only in shadow scope. It did not change runtime authority,
installed packages, daemon/IBus processes or version `1.0.33`. Its instrumented
maximum class p99 was `19.258 ms`, so latency remains outside the Slice 2 PASS.
Slice 3 subsequently preserved the explicit `UPSTREAM_INCOMPLETE` blocker
instead of converting retained targets into a complete authority cohort.

Exact milestone receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE2_MATERIAL_FRAME_2026-08-20/final-receipt.json`

### Slice 3 Closure

Slice 3 is closed in shadow scope. Candidate validity is derived only after
exact frame binding and is source-neutral. Witness failure is local, an
incomplete target namespace remains `Born`, `Rejected` requires a complete
namespace, and field incompleteness remains an absolute authority blocker.

The fixed `13x100` proof produced `3,864/3,864` candidate-state derivations,
zero false grounding, zero cross-context mismatch, zero stale-frame accepts and
`3,900/3,900` separate original-preservation verdicts. H/B/S0, probe parity,
false singleton and integrity denominators did not regress.

The historical aggregate remains latency-failed: Productive traversal yields
instrumented maximum class p99 `16.181 ms > 5 ms`. This is not part of the Slice
3 semantic PASS and no live promotion is claimed. Runtime authority, installed
packages, daemon/IBus and version `1.0.33` were unchanged.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE3_CANDIDATE_STATE_2026-08-20/final-receipt.json`

The next code mutation requires a Slice 4 implementation preflight for complete
conflict-component construction and deterministic `Winner | Tied | ABSTAIN`
shadow settlement.

### Slice 4 Closure

Slice 4 is closed in shadow scope. Every retained material target is rebound to
each exact frame, assigned a candidate state, placed in an exact-footprint
conflict component and settled through one deterministic cohort. Duplicate
outputs merge by semantic root before canonical ordering. Original preservation
is consumed before every Winner path.

The fixed `13x100` proof derived `3,900/3,900` cohorts. It measured
`Winner/Tied/ABSTAIN = 0/1,050/2,850` with zero context/hash mismatch,
incomplete Winner, false singleton, lost grounded target, multiple-component
authority or preservation bypass. The absence of fixed-corpus Winners is not a
coverage PASS; Winner mechanics are truth-table tested and live promotion
remains unavailable.

The historical aggregate remains latency-failed at maximum class p99
`14.566 ms > 5 ms`. Runtime authority, installed packages, daemon/IBus and
version `1.0.33` were unchanged.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE4_CONFLICT_COHORT_2026-08-20/final-receipt.json`

The next code mutation requires a Slice 5 implementation preflight for bounded
missing-target birth and retention shadow.

### Slice 6 Closure

Slice 6 internalizes exact boundary birth inside the context-neutral prepared
material field. `BoundarySplit` requires exact grounding for both ordered parts;
`BoundaryMerge` requires exact source parts and an exact merged target. The
result carries `CompositeBoundaryGroundingV1`, exact separator-only geometry and
remains `Born`, so this slice cannot grant automatic authority.

The field owns one separate two-surface boundary reserve. Exact duplicate
Productive, contour and boundary surfaces merge before storage accounting. More
than two logical boundary surfaces marks the complete prepared field
`Overflow(StorageCapacity)`; no truncated subset can be mislabeled complete.
Enumeration is bounded to 64 grounding lookups and 64 operator steps.

The regenerated remote `13x100` proof measured:

```text
evaluated comparisons                              2,600
H / B / S0                           1,280 / 1,280 / 1,280
H -> B / B -> S0 losses                               0 / 0
BoundarySplit / BoundaryMerge package proofs          PASS
false-split / real multi-split overflow proofs        PASS
contour birth / retention / born-only                  8/8
boundary automatic authority grants                      0
false singleton / integrity errors                     0/0
wall / CPU / peak RSS                 34.21 s / 1026% / 650,852 KiB
maximum class p99                              11.238 ms
```

The Slice 6 boundary contract is a scoped shadow PASS. The aggregate receipt is
still `FAIL_measured_shadow_gates` because `11.238 ms > 5 ms`; that blocks live
promotion and deployment. The observed old deterministic boundary route did
not reproduce the package-derived split/merge outputs. This is recorded as a
legacy coverage difference, not used as a parity or promotion gate, and does
not alter either route.

Focused remote tests passed: boundary `3/3`, material-frame `10/10`. Not tested:
live authority transfer, crash-safe mutation, queue-inclusive IBus/daemon
latency, physical WeChat/Telegram behavior or deployment. Runtime authority
changed: `false`; installed version remains `1.0.33`.

Exact receipt directory:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE6_BOUNDARY_INTERNALIZATION_2026-08-20`

The next permitted implementation is Slice 7, beginning with the isolated
durability-strategy microproof required by the authority paper. No live-owner
change is permitted while the absolute latency gate remains open.

### Slice 7A Buffered Durability Rejection

The isolated output-transaction implementation passed its seven focused tests
and complete injected fault matrix. The invalid tmpfs run and both earlier ext4
runs are retained. The final ext4 proof used the SHA-matched remote metrics
binary, 1,000 measured samples after 64 warmups and a cold-preallocated 8 MiB
journal.

Prepare p99 passed at `1.265 ms`, co-commit p99 passed at `1.164 ms`, and
co-commit maximum passed at `1.677 ms`. Prepare maximum was `325.387 ms`, so the
strict `<8 ms` gate failed. Runtime authority and the installed system were not
changed.

The failure persisted after removing inode growth and moving allocation out of
the hot path. Buffered ext4 `write_at + fdatasync` is therefore rejected rather
than rerun or threshold-tuned. The next bounded step is one
`DirectAlignedSlotCommitV1` microproof using fixed aligned slots and
`O_DIRECT | O_DSYNC`. Its semantic and numeric gates are unchanged. If direct
I/O also fails, Slice 7 returns to backend atomicity design; no third local
storage experiment is permitted.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE7A_DURABILITY_MICROPROOF_2026-08-20/final-ext4-preallocated-receipt.json`

### Slice 7A Direct Durability Rejection

The only admitted direct-storage proof used the SHA-matched remote metrics
binary, fixed 4 KiB aligned slots in an 8 MiB ring, and one
`O_DIRECT | O_DSYNC` write for every durability unit. The focused remote tests
passed `8/8`; all checksum, torn-slot, foreign-generation, wrap, saturation,
prepare-failure and terminal-failure strata passed.

The one final ext4 proof used 64 warmups and 1,000 measured samples per stratum:

```text
prepare p50 / p95 / p99                 0.702 / 1.289 / 1.657 ms
prepare maximum                                     330.324 ms  FAIL
co-commit p50 / p95 / p99              0.694 / 1.228 / 1.575 ms
co-commit maximum                                  162.240 ms  FAIL
tail-flush p99 / maximum                    1.483 / 154.736 ms
next-native p99 / maximum                    1.593 / 14.571 ms
fault matrix                                                   PASS
wall / peak RSS                                      7.61 s / 11,940 KiB
```

The direct path satisfies the frozen p99 `<=2 ms` gate but violates the strict
maximum `<8 ms` gate for both prepare and co-commit. Removing the buffered page
cache and separate `fdatasync` therefore did not bound device/filesystem
durability tails. `DirectAlignedSlotCommitV1` is rejected without a rerun,
threshold change or third local-storage variant.

Slice 7 returns to `BackendAtomicReceiptV1`. The next work is a new paper and
route receipt proving that the complete backend effect vector is atomic or
exactly queryable/idempotently replayable after every kill point. Slice 7B and
all live-owner work remain blocked until that independent receipt passes.
Runtime authority and installed Lay `1.0.33` are unchanged.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE7A_DURABILITY_MICROPROOF_2026-08-20/final-direct-aligned-slot-receipt.json`

### Slice 7 Backend Atomic Design

The installed IBus `1.5.34~rc2-1` contains an existing synchronous
post-process queue that captures commit, delete, preedit and forwarded-key
operations during one `ProcessKeyEvent`. It is not currently an atomic receipt:
engine failure can leave a prefix in the queue, queue overflow is not a
whole-frame refusal, and the client post-process API returns `void`, so fetch
failure cannot revoke `handled`.

`IbusSynchronousPostProcessReceiptV1` is selected for paper/preflight work. It
makes `ibus-daemon` the sealed-frame owner and one capability-checked client
adapter the only mutation/disposition owner. Engine error, overflow, fetch
failure, unsupported client or delete refusal produces zero mutation and the
original event remains unhandled. Mutation outside `ProcessKeyEvent`, including
the current surrounding-text callback auto-undo path, is forbidden.

The V1 route packet was retained with `VETO` after three role-direction errors.
Corrected V2 passed the design gate with no issues or warnings. It remains
design-only: `safe_to_edit=false`, source/runtime behavior unproved. The next
gate is an implementation preflight against the exact Ubuntu source package;
working IBus installation and services are immutable in that scope.

Design document:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/ime-backend-atomic-receipt-v1-2026-08-20.md`
