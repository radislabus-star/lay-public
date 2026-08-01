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

1. Кодировать левый и правый контекст, пунктуацию, порядок слов и морфологический слот.
2. Строить отношения между всеми кандидатами L2, а не только выбранной парой.
3. Научить поле различать союз, предлог, окончание, форму глагола и соседние сущности.
4. Проверить обобщение на неизвестных предложениях.
5. Сохранить `Tied/ABSTAIN`, когда контекста недостаточно.

**Предлагаемый gate:** target lattice `>=99%`, однозначный top-1 `>95%` по каждому контекстному классу, ambiguity retention `>=99%`, false authority `0`.

## 4. Типизированное ядро L4

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

1. Реализовать candidate-relative scene encoder.
2. Обучать positive, anti и ambiguity centers только по причинным receipts.
3. Сначала разрешить перенос только как `SuggestOnly`.
4. Отдельно доказать whole-token layout transfer.
5. Затем исследовать single-grapheme transfer.
6. Провести ablation: без anti-centers качество обязано измеримо ухудшаться.

**Gate:** улучшение на unseen equivalent scenes, `RU->EN` и `EN->RU` проходят отдельно, false automatic layout projection `0`.

## 6. Производительность без потери кандидатов

1. Уменьшить L1.1 diverse first-touch p99 с `24.365 ms`.
2. Использовать mmap-prefetch, background warmup, cache и bounded active sets.
3. Не удалять candidate sources и не уменьшать поле.
4. Проверить точное совпадение результатов до и после оптимизации.

**Цель:** first-touch p99 `<=5 ms`, hot p99 `<=5 ms`, candidate parity `100%`.

## 7. Финальный Product Gate

Проверить одну матрицу:

```text
WeChat | Telegram | Chromium | GTK | Qt | Kitty
typing | preedit | Backspace | Tab | Space
double Shift | focus change | layout conversion
```

После этого: remote release build, architecture receipts, `graphify update .`, version bump, commit, push, атомарная установка и контролируемый runtime smoke.

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
