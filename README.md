<div align="center">

# lay

**Double Shift RU/EN layout rescue для Linux**

`lay` чинит слово, набранное не в той раскладке: нажал **Shift два раза** и
продолжил писать.

**Статус: alpha.** Основной сценарий уже рабочий. Главная зона активной
доводки — автопомощь после пробела и редкие desktop edge cases.

Текущая версия: **1.0.43**.

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

The installer downloads the canonical compact L2 V13 model from the matching
GitHub Release when it is not already present locally. The 134.05 MiB asset is
cached under `~/.cache/lay/models/` and accepted only when both its byte count
and pinned SHA-256 match the released model contract.

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![GNOME](https://img.shields.io/badge/GNOME-45--47%2C%2050-4A86CF?logo=gnome)](https://gnome.org/)
[![Wayland](https://img.shields.io/badge/Wayland-native-blue)](https://wayland.freedesktop.org/)
[![Status](https://img.shields.io/badge/status-alpha-yellow)](#статус-alpha)
[![License: Non-Commercial](https://img.shields.io/badge/License-Non--Commercial-red)](#license)

</div>

## Что это

`lay` — локальный клавиатурный помощник для Linux-пользователей, которые пишут
на русском и английском вперемешку.

Главный сценарий:

```text
Набрал:  ghbdtn
Нажал:   Shift Shift
Стало:   привет
```

![lay demo](docs/publicity/demo.gif)

`lay` не использует буфер обмена для основного сценария и не требует облачной
модели. Он слушает физические клавиши локально, помнит короткий хвост набора и
при команде перепечатывает его в другой раскладке.

По умолчанию double Shift исправляет **1 последнее слово**. Параметры модели не
выставляются в пользовательский интерфейс; длительность нажатия и интервал
двойного Shift регулируются в разделе «Клавиши».

## Что нового в 1.0.43

- физический Double Shift в обычном IBus-маршруте распознаёт только
  `lay-daemon`; legacy `ProcessKeyEvent` больше не запускает вторую обратную
  замену;
- атомарный compositor-маршрут сохраняет собственного IME-владельца, поэтому
  один жест по-прежнему формирует один атомарный edit;
- тайминги Double Shift остаются регулируемыми, но больше не используются для
  маскировки конкурирующих владельцев жеста.

## Что нового в 1.0.42

- Double Shift циклически переворачивает весь текущий токен RU/EN без лимита в
  один round-trip;
- немедленный Double Shift после автозамены по-прежнему сначала возвращает
  точный исходный ввод, а следующий жест входит в обычный циклический маршрут;
- в настройки возвращены ограниченные регуляторы длительности нажатия и паузы
  между двумя Shift; они используют существующие runtime-параметры и не меняют
  L1-L4 или authority исправления.

## Что нового в 1.0.27

- tray оставляет только ежедневные рабочие действия: раскладка, включение Lay,
  режим ввода, помощь при наборе, автозамена, настройки и диагностика;
- `settings.js` и GNOME Extension Preferences используют один общий
  `settings_view.js`, поэтому две страницы настроек больше не расходятся;
- из UI удалены ручные веса L2/L3, NANDA-паспорт, внутренние timing/multi-tap
  ручки, Enter-autocorrect, git-update и управление службами из диагностики;
- сохранение настроек не удаляет скрытые runtime-ключи из пользовательского
  `config.json`;
- полный инвентарь и границы ответственности зафиксированы в
  [`docs/lay-menu-settings-architecture.md`](docs/lay-menu-settings-architecture.md).

## Что нового в 1.0.16

- L2 ищет продолжения повреждённого префикса одним bounded
  Damerau-проходом по DAFSA и сохраняет словоформы из compact
  decoder;
- L3 добавляет до четырёх целей из подтверждённой online context-memory,
  поэтому короткий префикс может получить нужное слово до обычного
  lexical top-N;
- `TransitionDecisionCore` теперь фактически отсекает запрещённые
  продолжения: полное слово не продлевается по одной лексической
  геометрии без точного context-birth или накопленной context-to-target связи.

## Что нового в 1.0.15

- L2 восстанавливает потерянную границу после короткого русского местоимения,
  когда правая словоформа независимо подтверждена lexical/morphology field:
  например, `мнесбросили` становится `мне сбросили`;
- правило не содержит исключения для конкретной фразы: защищённые и частотные
  цельные слова имеют приоритет, а `мнение` не дробится в `мне ние`;
- исправление проходит полный live-контур
  `CanonicalL2FieldBoundary -> L3 -> DecisionCore -> verifier`, а не только
  появляется в списке подсказок.

## Что нового в 1.0.14

- L3 больше не строит одинаковую context scene отдельно для каждого кандидата
  одного IME frontier: общий фазовый вектор вычисляется один раз, после чего к
  нему добавляется только кандидатная компонента;
- исправление направлено на наблюдавшийся preedit-всплеск `83.652 ms` на
  префиксе `сдела`, где `82.290 ms` принадлежали L3; набор кандидатов,
  anti-phase, pairwise readout и полномочия L3 не урезаны;
- изолированный full-sentence readout теперь имеет `p99 0.628 ms`, но редкие
  post-install GUI outliers ещё наблюдаются, поэтому общий live latency gate
  остаётся в статусе `WATCH`.

## Что нового в 1.0.13

- после layout-автокоррекции переключение process-level IBus engine больше не
  блокирует Space на `120-204 ms`: текущий decoder сразу принимает целевую
  раскладку, а внешний switch выполняет один latest-only background worker;
- это закрывает живой маршрут `Tcnm -> Есть`, где старый timeout задерживал
  Space на `204.588 ms`, возвращал `ok=false` и пропускал первую букву
  следующего слова как латинскую (`nакой` вместо `такой`).

- физический Space больше не ждёт тяжёлый `DecisionCore`: решение для
  автокоррекции готовится одним latest-only worker, а неготовый или устаревший
  результат пропускает автозамену и немедленно возвращает пробел;
- применение фонового решения защищено точным ключом
  `engine path + tail epoch + committed tail + layout`, поэтому результат от
  старого текста нельзя применить к новому;
- автозамена вместе с вызвавшим её Space обязана вернуть ровно один конечный
  пробел;
- boundary-shift умеет восстанавливать подтверждённые короткие служебные слова:
  например, физический хвост `документ ыим` может восстановиться как
  `документы им` без словарного исключения для конкретной фразы;
- немедленный double Shift после автозамены отменяет именно последнюю
  автозамену и возвращает исходный ввод;
- live lexical route закрыт в один контур:
  `L1.1 bounded lattice -> Canonical L2 V13 -> L3 -> DecisionCore -> verifier`;
- установленный Canonical L2 V13 занимает `140556462` байта (`134.05 MiB`),
  хранит hot lemma-wave index внутри compact V2 и загружается как неизменяемый
  локальный пакет без его повторного построения в каждом live-клиенте.

Измерение, из-за которого Space был переделан:

```text
старый Space total             250.388 ms
из них DecisionCore            249.579 ms
сам commit Space                 0.690 ms
```

Начиная с `1.0.12`, синхронного вызова DecisionCore в обработчике Space нет.
В `1.0.13` из той же горячей стадии убран синхронный process-level layout
switch.
Фактические GUI p50/p95/p99 и доля готовых prefetch-решений продолжают
собираться по живой телеметрии; это не объявляется закрытым quality gate без
измерений.

## Текущая архитектура

```text
повреждённая или незавершённая поверхность
-> L1.1: bounded lexical lattice
-> Canonical L2 V13: локальная конкуренция форм и соседей
-> Winner | Tied | Abstain
-> L3: давление контекста фразы
-> TransitionDecisionCore
-> structural verifier
-> AuthorizedEdit
-> uinput или IBus backend
```

Владельцы разделены:

- **L1.1** восстанавливает сигнал слова и отдаёт ограниченную решётку, а не
  единственную догадку;
- **L2** конкурирует между словоформами, морфологическими слотами и локальными
  соседями; live owner — standalone `CanonicalL2Field` V13;
- **L3** учитывает более широкий контекст фразы, но не печатает текст напрямую;
- **DecisionCore** выбирает `apply / suggest / keep / veto`;
- **verifier + AuthorizedEdit** являются единственным разрешённым путём
  изменения текста;
- **IME/uinput** исполняют уже принятое действие и не содержат отдельного
  лингвистического мозга.

Главное запрещённое сокращение:

```text
слово -> частное правило -> прямая печать
```

Все кандидаты, включая layout, typo, boundary и morphology, обязаны войти в
общую решётку и пройти DecisionCore и verifier.

## Архив изменений до 1.0

Ниже сохранены заметки ранних alpha-релизов. Они описывают историю разработки,
но не являются спецификацией текущего runtime.

### Что нового в 0.2.0

- заложен первый системный слой `Typing Correction Core`: входной текст теперь
  получает единый паспорт ошибки, класс ошибки, доску кандидатов и gate-решение;
- deterministic typing-assist и NANDA-кандидаты проходят через общий
  `CandidateBoard`, вместо разрозненного применения частных правил;
- добавлены классы ошибок: wrong layout, mixed script, missing/extra/repeated
  letter, adjacent transposition, composite typo, split/glue, grammar,
  completion-only, technical/protected token;
- IME остаётся backend отображения/вставки: новый core не переносит принятие
  решений в IME и не меняет маршрут видимого preedit.

### Что нового в 0.1.245

- общий рубильник `Журнал отладки действий` теперь реально гасит runtime
  debug JSONL-журналы, включая NANDA/IME trace;
- runtime-журналы ограничены кольцевым хвостом 500 KiB, чтобы `cell_trace` и
  `precognition` не разрастались во время живого набора;
- добавлен контекстный guard для `b -> и`: замена разрешается только внутри
  русской фразы с поддержкой слева и справа, без технических/ASCII-барьеров;
- IME profile-тесты закрепляют различие Kitty terminal passthrough и managed
  commit для клиентов с surrounding text.

### Что нового в 0.1.244

- добавлен и документирован экспериментальный IME backend: inline/preedit
  подсказки, Tab-принятие кандидата и committed-tail замены без clipboard;
- демо `docs/publicity/demo.gif` обновлено: теперь показывает double Shift,
  smart-tail, IME-подсказку и автопомощь после пробела;
- усилены guard-правила автозамены по живым логам: NANDA/L2 больше не должны
  превращать нормальные фразы в смысловой дрейф вроде `модель генерит -> модель
  генерал`.

### Что нового в 0.1.232

- в окне настроек поменяны местами блоки `Управление` и `Арбитры и каналы`,
  чтобы управление триггерами было в ожидаемой позиции.

### Что нового в 0.1.231

- IME double Shift восстанавливает потерянную первую layout-букву в
  терминальном passthrough-хвосте: `hbdtn -> привет`,
  `dnjpfvtyf -> автозамена`;
- mixed-script автозамена удаляет дублирующий латинский layout-префикс перед
  русским словом: `fавтозамена -> автозамена`;
- NANDA Wave trainer умеет дообучать Cell32-память из успешных live-действий
  `typing-assist`, `ime-typing-assist`, `layout-replay`, `smart-text`;
- ручные `user-correction` не попадают в обучение по умолчанию и требуют
  отдельного opt-in флага.

### Что нового в 0.1.230

- IME-подсказки больше не переводят уже напечатанный токен в активную
  композицию в терминальных клиентах без surrounding-text;
- это защищает Kitty/терминалы от рассинхрона preedit, потери пробела и
  склейки соседних слов после подсказки;
- генерация подсказок остаётся быстрой и работает как безопасный suffix-preedit.

### Что нового в 0.1.229

- установщик больше не проверяет и не предлагает Ollama/smollm;
- при установке или обновлении выполняется миграция старых lay-установок:
  legacy `ollama.service`, бинарник и локальные модели Ollama удаляются;
- если Ollama нужна пользователю отдельно от lay, перед установкой можно
  сохранить её через `LAY_KEEP_OLLAMA=1 bash install.sh`;
- в настройках NANDA вынесена в отдельное окно: в основном меню осталась
  кнопка `NANDA ячейки`;
- старый Expert64/microbrain runtime удалён, окно NANDA показывает только
  реальный статус NANDA Wave из `lay-nanda-wave-eval --status-json`;
- live smoke harness научился использовать `LAY_CONFIG_PATH`, чтобы тестировать
  временный config без подмены `$HOME` и без поломки session bus.

### Что нового в 0.1.222

- CI smoke-проверка на GitHub переведена на стабильный главный сценарий:
  `ghbdtn -> привет`, без зависимости от словарной autocorrect-эвристики
  runner-а.

### Что нового в 0.1.221

- починен публичный CI/status на GitHub: architecture guard синхронизирован с
  текущей структурой проекта и больше не проверяет удалённые модули;
- статус последнего коммита на странице репозитория больше не должен оставаться
  красным из-за устаревшего guard-скрипта.

### Что нового в 0.1.220

- исправлен idle busy-loop в non-GNOME backend: `lay-daemon` больше не
  зацикливает `poll()` на 1 мс из-за stale focus-poll timestamp;
- настройки демона кешируются по `mtime` с коротким интервалом проверки, чтобы
  горячий путь не читал `~/.config/lay/config.json` на каждом цикле;
- русские горячие словари упакованы компактнее: текущая память свежего демона
  снизилась примерно со 168 МБ до 139 МБ после прогрева.

### Что нового в 0.1.219

- исправлен Niri/backend-кейс без `focused_window_identity`: при смене
  текстового поля буфер теперь разделяется по `field_context_epoch`, поэтому
  хвост из одного поля не склеивается с вводом в другом поле;
- добавлен регрессионный тест на сценарий без информации об окне.

### Что нового в 0.1.218

- GNOME extension больше не пишет штатные success-сообщения в журнал при каждом
  reload: `DBus enabled`, `DBus disabled`, `LayImpl enabled`;
- диагностические сообщения в GNOME journal оставлены только для ошибок и
  реально полезных предупреждений.

### Что нового в 0.1.217

- режим `Смелее` стал действительно смелее для одиночных раскладочных букв:
  `z` может исправляться в `я`, `b` — в `и`, при этом нормальный режим эти
  короткие замены не включает;
- короткий технический RU→EN перевёртыш `ьы` теперь распознаётся как `ms`;
- усилены существующие typo-слои: соседняя клавиша ловит формы вроде
  `кнорку -> кнопку`, а фразовый missing-letter может исправлять контекст
  вроде `с фалом -> с файлом`;
- область помощи при наборе отделена от области ручного double Shift: по
  умолчанию double Shift остаётся `1 слово`, а typing assist может смотреть
  короткий хвост отдельно;
- окно настроек получило desktop entry и нормальную иконку в GNOME panel /
  Alt-Tab;
- иконки в GNOME tray, меню и окне настроек приведены к одному компактному
  размеру;
- часть пунктов меню настроек переведена на русский и дополнительно сжата по
  горизонтали.

### Что нового в 0.1.216

- исправлен mixed-word рассинхрон в KDE/Kate и других backend: если слово почти
  целиком набрано в одной раскладке, а последняя буква пришла из другой, lay
  нормализует слово в доминирующую раскладку;
- живой проверочный кейс: `привеn` по double Shift становится `привет`, без
  дробления на `приве` + `n`.

### Что нового в 0.1.215

- добавлен Niri layout backend через прямой `niri-ipc`;
- `auto` стал умнее: в KDE/Plasma VM с nested Niri он выбирает KDE, а не
  ошибается по старому `NIRI_SOCKET`;
- в GNOME и KDE tray добавлен ручной выбор среды раскладки:
  `auto / KDE / X11 / GNOME / Niri`;
- KDE tray показывает выбор среды в верхнем блоке меню и не закрывается после
  смены backend;
- Niri помечен как экспериментальный режим для реальной Niri-сессии, а не как
  обязательный выбор для KDE + nested Niri.

### Что нового в 0.1.214

- откатили небезопасную GNOME/uinput speed-оптимизацию, которая в терминале
  работала, но в браузерных полях могла ломать double Shift replay;
- перед replay снова обязательно синхронизируются GNOME Shell и IBus, чтобы
  браузеры успевали принять фактическую раскладку;
- фиксы `b`, коротких кириллических фрагментов и `on`/`off` из `0.1.213`
  сохранены.

### Что нового в 0.1.213

- standalone `b` больше не автозаменяется в `в`/`и` без фразового контекста;
- короткие кириллические фрагменты больше не улетают в случайный EN-токен,
  например `ыл` не должен превращаться в `sk`;
- технические короткие EN-слова `on`/`off` оставлены разрешёнными для
  автопереворота;
- GNOME/uinput speed-path был добавлен в этом релизе, но в `0.1.214` откатан
  как ненадёжный для браузерных полей.

### Что нового в 0.1.212

- GNOME tray разрезан на отдельные модули: DBus bridge, меню последних
  исправлений и общие helpers больше не лежат одним большим файлом;
- установщик, dev-reload и CI теперь копируют и проверяют все JS-модули
  расширения, а не только старый монолит;
- таблицы русской морфологии вынесены в `data/lexicon`, чтобы runtime-код не
  превращался в склад списков;
- встроенные сценарии `lay-test-input` вынесены в `data/test_input`;
- исправлен баг `protected_words.txt`: пользовательские защищённые ASCII-слова
  больше не перебиваются L2/L3/LLM-скорингом, например `cd` не должен
  превращаться в `св` после пробела.

### Что нового в 0.1.210

- KDE backend кэширует `qdbus` и список раскладок вместо повторного запроса на
  каждый double Shift;
- короткий KDE replay теперь использует тот же быстрый isolated-output путь,
  что и GNOME;
- проверено live в KDE/Plasma X11 VM.

Ориентировочные live-замеры KDE/Plasma VM:

- KDE: короткое слово `~54-67 ms`;
- KDE: backspace/replay для коротких хвостов `~0-1 ms`;
- основной остаток в KDE теперь тоже переключение layout.

### Что нового в 0.1.209

- double Shift стал быстрее: если daemon успешно изолировал физическую
  клавиатуру через evdev grab, короткие замены выводятся без лишнего pacing;
- длинные хвосты остаются на paced-пути, чтобы приложения не теряли
  Backspace/replay события;
- debug-лог теперь показывает разрез времени по стадиям: layout, delete,
  insert/replay, total;
- ускоренный manual replay и typing assist проверены live в GNOME.

Ориентировочные live-замеры на тестовой машине:

- GNOME: manual double Shift на 2-3 слова `~40 ms`;
- GNOME: короткое слово `~53-62 ms`, основной остаток — переключение layout;
- GNOME: typing assist после пробела `~61-75 ms`;
- длинный хвост около 240 клавиш остаётся безопасно paced: `~1.1 s`.

## Быстрый старт

Установка одной командой:

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

После установки выйди из сессии и зайди снова. Это нужно для группы `input`,
доступа к `/dev/uinput` и desktop-интеграции.

Проверка:

```text
1. Включи русскую и английскую раскладки.
2. Набери ghbdtn.
3. Нажми Shift два раза.
4. Должно получиться привет.
```

Обновление:

```bash
cd ~/projects/lay && bash update.sh
```

Если старая установка оставила локальные изменения в git-копии, updater сам
сохранит их в именованный `git stash`, обновит исходники и напечатает команду
`git stash pop` для восстановления. Пользовательские настройки лежат вне git в
`~/.config/lay` и при обновлении не удаляются.

В GNOME и KDE обновление также доступно из меню трея: `Проверить обновления`.

Полное удаление runtime, настроек, памяти, логов и чистой установочной git-копии:

```bash
cd ~/projects/lay && bash uninstall.sh --purge
```

Без `--purge` скрипт удаляет только runtime и desktop-интеграцию, сохраняя
настройки и память. Локально изменённую git-копию uninstall не удаляет.

### Bazzite / Fedora Atomic

Installer распознаёт `rpm-ostree` раньше `dnf`. Если зависимостей не хватает,
он добавит их в следующий deployment и попросит перезагрузиться; после reboot
нужно повторить ту же команду установки. Проверить маршрут без изменений:

```bash
bash install.sh --check-platform
```

## Возможности

- **Double Shift** исправляет последнее слово в другой раскладке.
- **Откат автозамены**: немедленный double Shift возвращает исходный ввод.
- **Replay** физически перепечатывает хвост теми же keycode.
- **Smart** старается не трогать уже нормальные соседние слова.
- **Помощь при наборе** после пробела исправляет только уверенные ошибки.
- **Автоподмена** включает автоматические исправления после пробела.
- **Неблокирующий Space** не ждёт тяжёлое контекстное решение.
- **IME-подсказки** показывают bounded-кандидаты; Tab принимает видимое
  продолжение явно.
- **NANDA Wave** — локальный маршрут L1.1/L2/L3, который рождает и ранжирует
  кандидаты, но не обходит DecisionCore и verifier.
- **ptah_alexs** жёстко ставит раскладку для выбранных окон.
- **Прямые RU/EN хоткеи** могут включать конкретную раскладку без toggle.
- **KDE/Niri/X11 backend** есть, но покрытие меньше, чем у GNOME Wayland.

Пример Smart-сценария:

```text
good ntrcn -> good текст
```

Здесь `good` остаётся на месте, а исправляется только `ntrcn`.

## Что включено по умолчанию

Для новой установки базовое поведение консервативное:

- double Shift включён;
- область double Shift — `1 слово`;
- автоматическая помощь после пробела выключена, пока пользователь сам её не
  включит;
- точная автоподмена выключена, пока пользователь сам её не включит;
- основной вывод идёт через `uinput`;
- среда раскладки выбирается автоматически: `layout_backend=auto`;
- сетевые LLM/API не используются.

Старые версии lay могли оставлять локальный `ollama.service` для экспериментов.
Новые установки и обновления его вычищают. Это часть перехода на локальный
deterministic/NANDA pipeline без фонового LLM-сервиса.

Настройки хранятся в:

```text
~/.config/lay/config.json
```

## Поддержка окружений

Основная проверенная среда: Ubuntu/GNOME Wayland с RU/EN раскладками.

Текущая матрица:

- **GNOME Wayland** — основной и самый зрелый путь.
- **KDE/Plasma Wayland** — поддерживается, но покрытие меньше.
- **Niri Wayland** — есть backend через прямой `niri-ipc`, требует проверки
  на реальной Niri-сессии.
- **X11** — есть native XKB backend, проверяется как экспериментальный путь.
- **Sway/Hyprland/другие WM** — пока не заявлены как поддержанные.
- **Языки** — текущая цель только RU/EN.

Поддерживаемые backend'ы:

- GNOME: Shell extension, tray и DBus bridge для переключения раскладки;
- KDE/Plasma: отдельный `lay-kde-tray` и переключение через `qdbus6`;
- Niri: прямой IPC через Unix socket и crate `niri-ipc`;
- X11: native XKB backend через `x11rb`.

По умолчанию используется `layout_backend=auto`. Обычно это правильный выбор:
GNOME выбирает GNOME, KDE/Plasma выбирает KDE, X11 выбирает X11, настоящая Niri
сессия выбирает Niri. Ручной выбор нужен только для диагностики или нестандартной
вложенной среды.

KDE, Niri и X11 моложе GNOME-пути. Если что-то ломается в другой сборке Linux,
лучше открыть issue с точным примером:
что набрано, что ожидалось, что получилось.

## Языки

Текущая цель проекта — качественная пара **RU/EN**.

Используется:

- физическое соответствие US ↔ RU;
- Hunspell-словари, если они есть в системе;
- локальные RU/EN правила;
- char n-gram и L2/L3 phase scoring;
- пользовательские точные замены;
- пользовательский список защищённых слов.

Не заявлено как готовое:

- другие пары раскладок;
- полноценная грамматика русского языка;
- исправление целых абзацев;
- универсальные серые подсказки во всех приложениях. Экспериментальный IME
  backend уже умеет preedit-кандидаты, но это отдельный режим, а не
  гарантированный путь для каждого текстового поля.

## NANDA Wave: текущий runtime

Текущий NANDA-маршрут — не набор независимых Cell32, которые напрямую решают,
что печатать. Реальный live owner выглядит так:

```text
L1.1 bounded lexical lattice
-> standalone CanonicalL2Field V13
-> one Winner | Tied | Abstain readout
-> L3 phrase context
-> TransitionDecisionCore
-> verifier
```

Canonical L2 V13 связывает существующие L1.1 terminal identities с формами,
леммами, морфологическими слотами, локальными context modes и directional
competition edges. Пакет read-only и не перекристаллизуется при каждой
пользовательской правке.

Текущие измеренные характеристики V13:

```text
source unique surfaces              1,875,032
L1.1-bound forms                      517,257
L2-materialized forms               1,357,775
lemma centers                          93,672
morphology bindings                3,255,785
directional competition edges         215,121
package bytes                     135,121,803
package size                          128.86 MiB
```

Это архитектурные и пакетные факты, а не доказательство качества L1.1.
Качество восстановления принимается только по фиксированному heldout proof с
процентами каждого класса ошибок; aggregate top-1 не может скрывать слабый
класс.

### Историческая исследовательская модель Cell32

`lay` экспериментирует с маленькой локальной NANDA Wave-архитектурой. Это не
облачная LLM и сейчас не замена основного автокорректора. Основной runtime
остаётся детерминированным: словари, L2/L3/ngram, protected words, rule graph и
safe replacement pipeline. NANDA Wave отдельно изучает клеточные признаки,
кандидаты, ablation и ансамблевые моды.

Базовая единица:

```text
Cell32 = 32 KB волновой клетки
```

Клетка маленькая по памяти, но может видеть весь короткий хвост предложения.
Она не обязана отвечать только за одно слово. Текущие зоны:

- `L1` сенсоры: UTF-8, письмо/форма, клавиатура, границы;
- `L2` кандидаты: раскладка, технические токены, фразовый сигнал;
- `L3` согласование: защитный контекст, фразовая связность, Mesh.

NANDA держится не на одном большом признаке, а на нескольких пространствах
признаков:

- `layout` — точное/обратное совпадение раскладки, расстояние до layout-кандидата;
- `script` — кириллица, ASCII, смешанные токены, переходы RU/EN;
- `token_identity` — известное русское/английское слово, technical, protected,
  CLI option;
- `boundary` — число слов, пробелы, склейка/разрыв, сохранение хвостового
  separator;
- `shape` — форма токена, регистр, цифры, пунктуация, дефисы;
- `edit` — edit distance, replacement span, длина правки;
- `risk` — plain-layout risk, technical/protected risk, prefix deletion risk;
- `context` — последовательность типов токенов в коротком хвосте.

Это важно: одинаковый сырой бюджет памяти может иметь разную топологию.
Монолит хранит всё в одном пространстве, а ансамбль раскладывает признаки по
органам: layout, guard, context/space.

Текущая цель по масштабу:

- минимальный живой мозг: `16` active-клеток;
- нормальный локальный организм: `64` warm-клетки;
- большой персональный мозг: до `1024` клеток в локальной памяти.

Это не число ради числа. Клетка считается живой только если ablation показывает
пользу: отключили клетку — ухудшился именно её класс задач. Если отключение
ничего не меняет, клетка декоративная и должна уйти в сон или быть удалена.

### Ensemble Mode

Важная часть NANDA Wave — искать моды не только внутри одной клетки, а в
ансамбле.

`Ensemble Mode` — устойчивый паттерн совместной активации Cell32-клеток,
который причинно отвечает за класс решений.

Пример безопасного wrong-layout режима:

```text
layout_writer      high
layout_signal      high
space_boundary     high
technical_guard    low
undo_guard         low
sentence_mesh      high
=> apply
```

Пример режима запрета:

```text
layout_writer      high
layout_signal      high
technical_guard    high
cli_guard          high
sentence_mesh      low
=> keep original
```

Иными словами, ответ не обязан жить в одной клетке. Он появляется как
когерентный пик нескольких независимых сигналов. Mesh делает несколько коротких
вычислительных тактов без задержек ввода: если пик устойчивый — исправление
можно применить; если клетки спорят — `lay` молчит.

### Кто учит клетки

Клетки не должны хаотично самообновляться во время печати. Обучение должно
быть отдельным локальным процессом:

- пользователь даёт главный сигнал истины: оставил, удалил, откатил,
  перепечатал, добавил protected word;
- deterministic `lay` даёт безопасные initial labels: layout, словари, n-gram,
  L2/L3 signals, protected words, replacement safety;
- старший teacher/arbiter может помогать разбирать спорные случаи и создавать
  synthetic training cases, но не должен становиться постоянным runtime-мозгом;
- offline trainer собирает fixtures, synthetic cases, learning log, user
  corrections и ablation reports;
- Mesh supervisor учит не одну клетку, а связи: какие сочетания клеток усиливать
  и какие гасить.

Схема:

```text
user feedback
+ deterministic labels
+ teacher labels
+ synthetic cases
-> trainer
-> new Cell32 candidate
-> eval
-> ablation
-> promote or reject
```

Runtime остаётся безопасным: клетки не печатают напрямую в систему. Они
предлагают или оценивают кандидаты, а вывод всё равно идёт через существующий
safe replacement pipeline.

Текущий статус NANDA: старый Expert64/microbrain runtime удалён. В коде
остался NANDA Wave-слой: Cell32-паспорт, L1/L2/L3, Wave trace, real-suite,
status JSON и ablation.

Проверка Wave-слоя:

```bash
lay-nanda-wave-eval --status-json
lay-nanda-wave-eval --trace "djn "
lay-nanda-wave-eval --real-suite --ablation
```

Клетку нельзя делать главной только потому, что она существует. Минимальный
критерий: `lay-nanda-wave-eval` не ухудшает baseline, а ablation показывает, что
отключение клетки ухудшает её конкретный класс задач.

Сравнение `64x3-vs-192` пока остаётся исследовательской задачей, а не
production-командой.

Для ансамбля критерий жёстче. Это называется **LayMesh Ensemble Mode
Criterion**:

```text
synergy(E, C) =
  quality(E, C) - max_i quality({e_i}, C)

ablation_drop(e_i, E, C) =
  quality(E, C) - quality(E without e_i, C)
```

Ансамблевая мода считается найденной только если одновременно верно:

- `synergy(E, C)` выше порога: связка лучше любой одиночной клетки;
- `ablation_drop` ключевой клетки заметный: разрушение связки ломает эффект;
- `false_positive_delta` не вырос: мода не шумит на чужих классах;
- `stability` подтверждена на разных split-корпуса;
- `locality` подтверждена: улучшение живёт в своём классе ошибок.

То есть мода — это не просто рост общей accuracy. Это устойчивый причинный
контур: несколько клеток вместе дают способность, которой нет у клеток по
отдельности, и эта способность исчезает при разрушении связки.

`lay-nanda-wave-eval --real-suite --ablation` печатает текущие признаки:

```text
synergy_vs_best_single
synergy_vs_best_pair
ablation_drop
false_positive_delta
stability
locality
mode_status
```

Если отчёт пишет `mode_status: ensemble_mode_found`, связка прошла критерий.
Если отчёт пишет `too_easy_or_redundant`, корпус не доказывает новую моду:
одиночные клетки уже справляются или связка ничего причинно не добавляет. Если
пишет `rejected_false_positives`, связка полезна в своём классе, но опасно
портит чужие случаи.

## Ограничения

`lay` сознательно работает с коротким хвостом текста, который daemon видел через
evdev. Это делает основной сценарий быстрым, но задаёт границы:

- не исправляет произвольное слово под курсором после ручного перемещения;
- не меняет выделенный текст как универсальную функцию;
- не читает весь текст поля и не знает весь документ;
- Enter-autocorrect не включён в публичный стабильный UI, потому что evdev/uinput
  не может гарантировать порядок "исправить хвост, потом отправить Enter" во
  всех приложениях;
- IME/preedit и inline-подсказки доступны как экспериментальный режим ввода, но
  не заменяют текущий быстрый uinput-путь;
- автопомощь после пробела остаётся консервативной: лучше пропустить сомнительный
  случай, чем самовольно испортить текст.

## Меню в трее

Основные пункты:

- `Раскладка` — переключить текущую RU/EN раскладку.
- `Lay включён` — единый запуск или остановка управляемого runtime.
- `Режим ввода` — быстрый uinput или IME с inline-кандидатами.
- `Помощь при наборе` — кандидаты и осторожная правка во время ввода.
- `Автозамена` — применять доказанные исправления после пробела.
- `Настройки` — единая страница ввода, клавиш, совместимости и диагностики.
- `Диагностика` — read-only статус, журнал и последние действия.

Внутренние веса модели, research-визуализаторы и service-recovery команды в
пользовательском меню не показываются.

## CLI

`lay` можно использовать из терминала:

```bash
lay "Ye djn ghbvth"
# Ну вот пример

lay "руддщ цщкдв"
# hello world

echo "ghbdtn" | lay
# привет
```

## Приватность

`lay-daemon` читает клавиатурные события локально, потому что иначе double
Shift rescue невозможен. По умолчанию он не отправляет набранный текст в сеть,
не требует удалённой модели и не ведёт полный keylog.

Опциональный learning log локальный и выключен по умолчанию:

```text
~/.local/share/lay/corrections.jsonl
```

Диагностические файлы и счётчики тоже локальные:

```text
~/.local/share/lay/recent_actions.jsonl
~/.local/share/lay/learning_candidates.json
~/.local/share/lay/stats.json
```

На Unix такие файлы создаются с правами `0600`.

## Статус alpha

Рабочее ядро: ручной double Shift, его откат последней автозамены, локальная
RU/EN-конвертация, Canonical L2 V13 и защищённый IBus/uinput output route.

Активно оттачиваются:

- автопомощь после пробела;
- hit rate фонового Space-prefetch без возврата блокировки;
- пробелы и boundary-shift после автозамены в разных приложениях;
- mixed RU/EN сценарии;
- KDE/X11 edge cases;
- работа в старых/особых текстовых полях;
- экспериментальный IME backend и preedit-кандидаты в разных приложениях.

Если присылаешь bug report, укажи:

- что набрано;
- что ожидалось;
- что получилось;
- GNOME/KDE/Niri/X11 и версия системы;
- включены ли `Помощь при наборе`, `Автоподмена`, `Smart` и IME.

Приватный текст перед отправкой лучше заменить на безопасный пример.

## Документация

- [Как это работает](HOW_IT_WORKS.md)
- [Каноническая архитектура L2 над L1.1](docs/l2-l11-canonical-architecture.md)
- [Маршрут интеллекта L1-L4](docs/l1-l4-intelligence-route.md)
- [Память кристаллического ядра L1.1](docs/l1-crystal-kernel-memory-layout.md)
- [Проверочный список архитектуры](docs/architecture-checklist-2026-05-19.md)
- [Multi-tap Shift scope](docs/multi-tap-shift-scope.md)
- [Research: Linux input correction best practices](docs/research/linux-input-correction-best-practices-2026-05-17.md)
- [Публичные материалы](docs/publicity/README.md)

## Разработка

```bash
scripts/check-lay-changed.sh
scripts/cargo-guard.sh build --release --bins
bash install.sh
```

Cargo build artifacts are limited to 12 GiB by `scripts/cargo-guard.sh`.
Installed binaries are copied to `~/.local/lib/lay/bin`, so `target/` remains
a disposable cache and can be removed without breaking the installed runtime.

Полная локальная проверка перед публикацией:

```bash
scripts/check-lay-full.sh
```

## English

`lay` 1.0.16 is a local Double Shift RU/EN layout rescue and bounded typing
correction tool for Linux desktops.

Main workflow:

```text
Typed:   ghbdtn
Press:   Shift Shift
Result:  привет
```

Quick install:

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

After installation, log out and log back in so the `input` group, `/dev/uinput`
permissions, and desktop integration are picked up.

Main tested target: GNOME Wayland with RU/EN layouts. KDE/Plasma Wayland and
Niri Wayland are supported with a smaller compatibility matrix. X11 has a native
XKB backend and is treated as experimental. Other layouts and non-RU/EN pairs
are not supported yet.

Known limitations: `lay` works on a short typed tail, not arbitrary selected
text or the whole document. Enter autocorrect is not the stable default path.
IME/preedit-style inline assistance exists as an experimental input backend, but
the fast uinput path remains the default. Space autocorrect decisions are
prefetched in the background; Space itself never waits for a slow DecisionCore
calculation, and a stale result is never applied to newer text.

By default `lay` does not use cloud APIs, does not require an LLM, and does not
send typed text anywhere.

## License

Lay Non-Commercial License v1.0. Commercial use is prohibited without prior
written permission from the copyright holders. See [LICENSE](LICENSE).
