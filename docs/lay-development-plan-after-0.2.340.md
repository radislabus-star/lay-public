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

**Статус: в работе, causal-episode V3 реализован; live promotion ожидает
реальные `2` независимых эпизода в `2` разных сценах.** Автоматическое
применение больше не считается пользовательским подтверждением. Один action
получает общий `episode_id`, selector проверяет ровно одну минимальную связь,
а manifest защищён `flock + fsync + atomic rename`. На текущем реальном
журнале `9` отношений имеют по одной сцене, поэтому delta намеренно не создан.
Release `0.2.343` собран на удалённой машине, установлен; реальный state
мигрирован V2 -> V3 без изменения manifest SHA. Глобальный IBus не
перезапускался.
Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L3_ONLINE_CAUSAL_EPISODES_V3_2026-08-01.json`.

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

Результат этапа: первый реальный пользовательский delta автоматически принят и загружен без перекомпиляции базовой модели.

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

**Статус: выполнено как `PASS_SHADOW` 2026-08-01, release `0.2.346`.**
Реализованы candidate-relative encoder на `64` фазовых ячейки, причинный join,
latest-state consolidation, positive/anti/hard-negative/ambiguity banks,
направленные pair profiles, bounded binary V1 и read-only hot reload. Поле
подключено к `TransitionDecisionCore` только как диагностика
`SuggestOnly | Keep`: оно не меняет birth, rank, admission или verifier.
Полный heldout содержит `436` случаев; все `8/8` классов направления,
знака и масштаба дали `100%`, false automatic projection `0`. Без anti-centers
появились `218/218` false supports, с anti-centers осталось `0/218`.
Package roundtrip, candidate readout order и runtime/evaluator parity прошли;
пакет занимает `3 652` байта. Fixed dirty replay `2 466` случаев сохранил
полный нормализованный JSON и `negative_false_apply = 0`.

Органический live package не продвинут: из `2 437` строк текущего журнала
только `9` образуют complete causal positives, negative/reverted evidence нет.
Синтетический proof-пакет также не устанавливается. Runtime authority не
менялась. Пять release-бинарников `0.2.346` установлены атомарно; global IBus,
managed engine и daemon не перезапускались. Receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L4_CROSS_SCENE_V1_SHADOW_2026-08-01.json`.

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
