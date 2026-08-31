<div align="center">

# lay

**Double Shift RU/EN layout rescue для Linux**

`lay` исправляет слово, набранное не в той раскладке: нажмите
**Shift два раза** и продолжайте писать.

**Текущая версия: 1.0.58. Статус: alpha.**

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

[![Rust](https://img.shields.io/badge/Rust-1.88+-orange?logo=rust)](https://www.rust-lang.org/)
[![GNOME](https://img.shields.io/badge/GNOME-45--47%2C%2050-4A86CF?logo=gnome)](https://gnome.org/)
[![Wayland](https://img.shields.io/badge/Wayland-native-blue)](https://wayland.freedesktop.org/)
[![Status](https://img.shields.io/badge/status-alpha-yellow)](#статус-и-ограничения)
[![License: Non-Commercial](https://img.shields.io/badge/License-Non--Commercial-red)](#license)

</div>

## Что это

`lay` — локальный клавиатурный помощник для Linux-пользователей,
которые пишут на русском и английском вперемешку.

Главный сценарий:

```text
Набрано:   ghbdtn
Команда:   Shift Shift
Результат: привет
```

![lay demo](docs/publicity/demo.gif)

Основной сценарий не использует буфер обмена и не требует облачной модели.
Daemon локально слушает физические клавиши, хранит короткий хвост ввода и
передаёт уже проверенную замену в `uinput` или IBus backend.

По умолчанию double Shift исправляет одно последнее слово. Автопомощь после
пробела и автоматическое применение исправлений выключены, пока пользователь
сам их не включит.

## Что вошло в 1.0.58

- Double Shift в Kitty и других доказанных terminal-клиентах снова выполняет
  замену одним IME commit-frame, а не серией физических Backspace и посимвольных
  key events;
- точная проекция сохраняет все символы: `rjvvbn -> коммит`, обратное
  `коммит -> rjvvbn` и две последовательные пары Shift используют один и тот же
  маршрут;
- terminal-route сохраняет пробел после слова и синхронизирует раскладку один
  раз; GTK/SurroundingText и daemon fallback остаются отдельными маршрутами;
- словари, кандидаты, ранжирование, автокоррекция и физический детектор Double
  Shift не изменялись.

## Что вошло в 1.0.57

- IME-подсказка допускается уже на точном префиксе из трёх символов, даже
  если фоновый readout занял больше прежних `50 ms`;
- окно публикации точного актуального результата расширено до `150 ms`, при
  этом проверки текста, фокуса, раскладки, конфигурации и поколения сохранены;
- устаревший результат по-прежнему не показывается, а настоящий пустой набор
  кандидатов остаётся без подсказки;
- источники кандидатов, их ранжирование, предел `12` и асинхронная обработка
  обычных клавиш не изменились.

## Что вошло в 1.0.56

- `↑/↓` больше не теряют список IME-вариантов, если нажаты в коротком
  интервале между вводом буквы и завершением фонового readout;
- для стрелки отменяется старый worker и один раз материализуется список для
  точного текущего префикса; обычный ввод остаётся асинхронным;
- `Tab`, Alt и курсорные `←/→` по-прежнему не могут принять устаревшую
  display-only подсказку;
- источники кандидатов, порядок ранжирования и предел `12` не сужались.

## Что вошло в 1.0.55

- точный автоматический маршрут теперь симметричен: `Згыр -> Push` работает
  так же, как `ghbdtn -> привет`;
- автопереворот применяется только когда исходного слова нет в русском
  словаре и пользовательских исключениях, а клавиатурная проекция является
  точным английским словом;
- маршрут отключается настройками `Автозамена` и `Следовать языку
  исправления`; ручной Double Shift остаётся независимым;
- сохранены исправления Double Shift, IME preedit и синхронизации раскладки из
  `1.0.54`, а IBus state owner и runtime/research build surface разделены на
  более узкие владельцы;
- герметичный release denominator: `2,369 passed`, без semantic и
  infrastructure failures.

## Что исправлено в 1.0.46

- Double Shift теперь определяется по точной последовательности
  `левый Shift press/release` два раза, а не по силе или длительности
  удержания;
- команда подтверждается на втором отпускании, а любая другая нажатая клавиша
  отменяет незавершённую последовательность, поэтому `Shift+буква` не считается
  ручным переключением;
- правый Shift и смешанная пара Shift не завершают настроенный
  `double-lshift`;
- `tap_max_ms` сохранён только для одиночных горячих клавиш; правила выбора
  исправления и маршруты изменения текста не менялись.

## Что исправлено в 1.0.45

- IME-подсказки снова работают в Kitty и других клиентах с IBus
  `ContentType=TERMINAL`; терминал больше не ошибочно считается sensitive-полем;
- password, PIN, PRIVATE и HIDDEN_TEXT по-прежнему полностью отключают
  подсказки и очищают IME tail;
- Double Shift в Kitty заменяет последнее слово через уже проверенный
  IME-маршрут `terminal_erase_commit`, а не через daemon-uinput fallback;
- неизвестные GUI-клиенты без SurroundingText и без явного terminal purpose
  по-прежнему не получают terminal-erase authority.

Проверка релиза:

```text
lay-ibus-engine                    245 passed, 0 failed
changed-file gate                  PASS
release binary parity              PASS
Kitty suffix                       пров + ерить -> проверить
Kitty Double Shift                 ghbdtn -> привет
global ibus-daemon                 не перезапускался
```

## Что вошло в 1.0.44

- exact V13 DAFSA загружается один раз на процесс из проверенного sidecar;
- безопасный typed view переиспользуется запросами без повторного byte-slice
  decode;
- immutable lexical facts вычисляются один раз внутри обработки кандидата и
  переиспользуются всеми admission-предикатами;
- exact, grounded и contour-кандидаты входят в один bounded L2 material;
- при переполнении сначала удаляется худший productive-only tail, а exact и
  grounded цели сохраняются;
- exact-кандидат не получает самостоятельного права на автозамену: решение
  всё равно проходит L3, `TransitionDecisionCore` и verifier;
- canonical V13 package, exact sidecar и десять release-бинарников
  устанавливаются как одна проверенная версия;
- GNOME extension, daemon, IME и CLI показывают версию `1.0.44`.

Проверка релиза:

```text
productive_v1                       153 passed, 0 failed, 1 ignored
V13 generation                       12 passed, 0 failed, 7 ignored
field cache                            5 passed, 0 failed
InputGate                              7 passed, 0 failed
authority contracts                   40 passed, 0 failed
tray UI contracts                     17 passed, 0 failed
installed GTK/IBus smoke               2 passed, 0 failed
```

Старый широкий набор `correction_core` одинаково даёт
`80 passed / 31 failed / 1 ignored` на исходниках `1.0.43`
и `1.0.44`. Результат `31 failed` — унаследованный долг, а не регрессия
V13.

## Скорость 1.0.44

Два механизма, вошедшие в релиз, измерены раздельно на фиксированном
single-worker маршруте:

| Участок | До | После | Изменение |
|---|---:|---:|---:|
| V13 traversal CPU | 26.032 ns/edge | 22.962 ns/edge | -11.80% |
| final materialization p99 | 5.947 ms | 2.546 ms | -57.19% |
| полный measured route p99 | 7.986 ms | 5.354 ms | -32.96% |

Набор кандидатов и exact-сертификатов в парных измерениях совпал. Ускорение не
получено отключением источников или сужением candidate language.

Это target-host benchmark внутренних владельцев, а не queue-inclusive desktop
key-to-text p99. Холодное построение process-lifetime owner также не является
задержкой каждого нажатия. Поэтому README не превращает эти числа в
неизмеренное обещание общей GUI latency.

Исследованный штраф около `+18.77 ns/edge` при двадцати workers
оказался следствием SMT и package/topology contention диагностического
эксперимента. Production `TypingAssistWorker` использует один worker,
поэтому этот многопоточный штраф не является production-регрессией.

## Текущая архитектура

```text
физический ввод
-> короткий typed tail
-> L1.1 bounded lexical lattice
-> immutable canonical V13 identities
-> exact V13 DAFSA typed owner
-> Productive L2 V90 PreparedCanonicalTokenField
-> bounded common candidate material
-> L3 phrase/context evidence
-> TransitionDecisionCore
-> structural verifier
-> AuthorizedEdit
-> uinput или IBus backend
```

Границы владения:

- **L1.1** восстанавливает сигнал слова и отдаёт bounded lattice, а не
  единственную догадку.
- **V13 exact owner** выполняет исчерпывающий поиск по неизменяемым canonical
  identities и сохраняет certificate/provenance.
- **Productive L2 V90** объединяет surface, grounded, layout, contour и exact
  evidence в один `PreparedCanonicalTokenField`.
- **L3** добавляет контекст фразы, но не печатает текст напрямую.
- **TransitionDecisionCore** принимает `apply / suggest / keep / veto`.
- **Verifier + AuthorizedEdit** являются единственным разрешённым путём
  изменения текста.
- **IME/uinput** исполняют решение и не содержат второго correction brain.

Запрещённое сокращение:

```text
слово -> частное правило -> прямая печать
```

Layout, typo, boundary, morphology и exact-кандидаты обязаны пройти общий
candidate material, DecisionCore и verifier.

## Данные и lifetime

Релиз использует три разных immutable представления:

| Артефакт | Размер | Назначение |
|---|---:|---|
| canonical L2 V13 package | 140,556,462 B (134.05 MiB) | формы, леммы и canonical identities |
| exact V13 DAFSA sidecar | 2,460,144 B | адресный индекс exact search |
| process typed payload | 3,689,628 B | безопасный typed view hot path |

Installer загружает canonical package из соответствующего GitHub Release,
кэширует его в `~/.cache/lay/models/` и принимает только при
совпадении размера и закреплённого SHA-256. Sidecar устанавливается в:

```text
~/.local/share/lay/nanda_wave/l2/LAY-L2-RU-FULL-v13.dafsa
```

Typed owner живёт до завершения процесса. Замена поколения требует управляемого
перезапуска Lay-процесса; request-local materialization отсутствует.

## Возможности

- **Double Shift** исправляет последнее слово в другой раскладке.
- **Откат автозамены**: немедленный double Shift возвращает исходный ввод.
- **Replay** физически перепечатывает хвост теми же keycode.
- **Smart** сохраняет уже нормальные соседние слова.
- **Помощь при наборе** после пробела предлагает только bounded-кандидаты.
- **Автоподмена** применяет только допущенные общим authority route решения.
- **Точный автопереворот RU/EN** требует словарного подтверждения целевого
  слова и отсутствия исходного слова в активном языке.
- **Неблокирующий Space** не ждёт тяжёлый контекстный расчёт.
- **IME-подсказки** показывают кандидаты; Tab явно принимает продолжение.
- **Прямые RU/EN hotkeys** включают конкретную раскладку без toggle.
- **KDE, Niri и X11 backends** доступны с меньшим покрытием, чем GNOME.

Пример Smart-сценария:

```text
good ntrcn -> good текст
```

## Быстрый старт

Установка:

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

После первой установки выйдите из пользовательской сессии и войдите снова.
Это применяет группу `input`, права на `/dev/uinput` и
desktop-интеграцию.

Проверка:

```text
1. Включите русскую и английскую раскладки.
2. Наберите ghbdtn.
3. Нажмите Shift два раза.
4. Должно получиться привет.
```

Обновление:

```bash
cd ~/projects/lay
bash update.sh
```

Updater сохраняет найденные изменения исходников в именованный
`git stash`. Настройки из `~/.config/lay` не удаляются.

Полное удаление runtime, настроек, памяти, логов и чистого checkout:

```bash
cd ~/projects/lay
bash uninstall.sh --purge
```

Без `--purge` удаляются runtime и desktop-интеграция, но сохраняются
настройки и память. Локально изменённый checkout автоматически не удаляется.

### Bazzite / Fedora Atomic

Installer обнаруживает `rpm-ostree` раньше `dnf`. Если
нужные пакеты добавлены в следующий deployment, перезагрузите систему и
повторите команду установки. Read-only проверка маршрута:

```bash
bash install.sh --check-platform
```

## Настройки

Для новой установки:

- double Shift включён;
- область double Shift — одно слово;
- помощь после пробела выключена;
- автоматическое применение выключено;
- основной output backend — `uinput`;
- `layout_backend=auto`;
- сетевые LLM/API не используются.

Настройки хранятся в:

```text
~/.config/lay/config.json
```

Основные пункты tray:

- `Раскладка`;
- `Lay включён`;
- `Режим ввода`;
- `Помощь при наборе`;
- `Автозамена`;
- `Следовать языку исправления`;
- `Настройки`;
- `Диагностика`.

Веса моделей, research-визуализаторы и команды восстановления сервисов в
пользовательском меню не показываются.

## Поддержка окружений

Основная проверенная среда — Ubuntu/GNOME Wayland с RU/EN раскладками.

| Среда | Статус |
|---|---|
| GNOME Wayland | основной и наиболее проверенный маршрут |
| KDE/Plasma Wayland | поддерживается, покрытие меньше |
| Niri Wayland | прямой `niri-ipc`, требуется больше live-проверок |
| X11 | native XKB backend, экспериментальный |
| Sway/Hyprland/другие WM | пока не заявлены |

Текущая языковая цель — только RU/EN. Полная грамматика русского, исправление
абзацев и другие пары раскладок не заявлены.

## CLI

```bash
lay "Ye djn ghbvth"
# Ну вот пример

lay "руддщ цщкдв"
# hello world

echo "ghbdtn" | lay
# привет
```

## Приватность

`lay-daemon` читает клавиатурные события локально, поскольку иначе
double Shift rescue невозможен. По умолчанию он не отправляет набранный текст
в сеть, не требует удалённой модели и не ведёт полный keylog.

Опциональный learning log локальный и выключен по умолчанию:

```text
~/.local/share/lay/corrections.jsonl
```

Диагностические файлы также локальны:

```text
~/.local/share/lay/recent_actions.jsonl
~/.local/share/lay/learning_candidates.json
~/.local/share/lay/stats.json
```

На Unix приватные runtime-файлы создаются с правами `0600`.

## Статус и ограничения

Рабочее ядро:

- ручной double Shift и откат последней автозамены;
- локальная RU/EN-конвертация;
- V13 exact owner и Productive L2 V90 candidate material;
- защищённый IBus/uinput output route;
- фоновый, latest-only Space prefetch.

Активно проверяются автопомощь после пробела, mixed RU/EN, boundary-shift,
редкие desktop text fields, KDE/X11 и экспериментальный IME/preedit.

`lay` работает с коротким хвостом, который увидел daemon. Он не
редактирует произвольное слово под курсором, выделенный текст или весь
документ. Сомнительное автоматическое исправление должно быть пропущено.

## Документация

- [Как это работает](HOW_IT_WORKS.md)
- [Каноническая архитектура L2 над L1.1](docs/l2-l11-canonical-architecture.md)
- [Маршрут интеллекта L1-L4](docs/l1-l4-intelligence-route.md)
- [Архитектурное исследование V13 exact owner](docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md)
- [Память кристаллического ядра L1.1](docs/l1-crystal-kernel-memory-layout.md)
- [Архитектура меню и настроек](docs/lay-menu-settings-architecture.md)
- [Герметичные Rust test lanes](docs/test-lanes.md)
- [Публичные материалы](docs/publicity/README.md)

История старых alpha-релизов остаётся в Git и GitHub Releases; README описывает
только текущий продукт.

## Разработка

Минимальная поддерживаемая версия Rust — `1.88.0` для default features и
`lexical-compiler`; optional `direct-llm` в этот контракт не входит. Обычная
разработка, `rustfmt` и `clippy` закреплены на `1.97.1`; точные compiler
identities и процедура обновления описаны в
[Rust Toolchain Policy](docs/rust-toolchain-policy.md).
Единый локальный и CI lint-контракт описан в
[Rust Lint Policy](docs/lint-policy.md).

Обычная локальная проверка:

```bash
scripts/check-lay-changed.sh
```

Полный герметичный correctness/package denominator:

```bash
scripts/check-lay-tests.sh all
```

Timing budgets запускаются отдельно через
`scripts/check-lay-tests.sh performance`; live desktop smoke всегда opt-in.

Cargo-команды выполняются через disk guard:

```bash
scripts/cargo-guard.sh --status
scripts/cargo-guard.sh build --release --bins
```

Полный release gate:

```bash
scripts/check-lay-full.sh
```

Default `target/` budget — 12 GiB. Установленные release-бинарники
лежат в `~/.local/lib/lay/bin`, поэтому `target/` остаётся
удаляемым build cache.

После изменения кода или документации обновите knowledge graph:

```bash
graphify update .
```

## English

`lay` 1.0.58 is a local Double Shift RU/EN layout rescue and bounded
typing-correction tool for Linux desktops.

```text
Typed:   ghbdtn
Press:   Shift Shift
Result:  привет
```

The current route combines a bounded L1.1 lattice, immutable V13 identities,
an exact process-lifetime DAFSA owner, Productive L2 V90 candidate material,
L3 context, `TransitionDecisionCore`, and a structural verifier.
Exact search contributes candidates and certificates but does not bypass final
authority.

Release 1.0.58 routes proven terminal committed-tail Double Shift through one
IME erase-and-commit frame instead of physical Backspace and per-character key
replay. The exact `rjvvbn -> коммит -> rjvvbn` round trip preserves its trailing
boundary and performs one layout synchronization per gesture. Release 1.0.57
admits an exact-current three-character IME completion for up
to 150 ms while retaining full input-identity checks and the existing
12-candidate field. A genuine zero-candidate prefix still displays nothing.
Release 1.0.56 restores `Up`/`Down` candidate cycling during an in-flight IME
readout without authorizing a stale completion or narrowing the 12-candidate
field. Release 1.0.55 adds symmetric exact automatic layout correction: a token such
as `Згыр` becomes `Push` only when the source is absent from the Russian guard,
the keyboard projection is an exact English word, and both automatic settings
are enabled. Release 1.0.46 makes Double Shift an exact clean key sequence without a
per-press hold-duration limit; any intervening key cancels it. Release 1.0.45
restored Kitty terminal suggestions and routed terminal Double Shift through
the proven IME erase-and-commit backend. Release 1.0.44 reduced
the measured V13 traversal CPU cost by 11.8% and the
paired internal end-to-end p99 by 33.0%, without narrowing the candidate set.
These are fixed target-host internal benchmarks, not a desktop key-to-text
latency claim.

Quick install:

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

Log out and back in after the first installation so the `input` group,
`/dev/uinput` permissions, and desktop integration take effect.

The primary tested target is GNOME Wayland. KDE/Plasma Wayland and Niri have a
smaller compatibility matrix; X11 is experimental. Only RU/EN is currently
supported.

By default, `lay` uses no cloud API or remote LLM and sends no typed
text anywhere.

## License

Lay Non-Commercial License v1.0. Commercial use is prohibited without prior
written permission from the copyright holders. See [LICENSE](LICENSE).
