<div align="center">

# lay

**Спасатель неправильной RU/EN раскладки для Linux**

**Double Shift RU/EN layout rescue for Linux desktops**

**Статус проекта: alpha.** `lay` уже можно использовать каждый день, но это
ранняя версия: автокоррекция, KDE/X11 backend и edge cases в разных
приложениях ещё активно оттачиваются.

Напечатал слово не в той раскладке? Нажми **Shift два раза** и продолжай писать.

Установить:

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

Обновить уже установленный `lay`:

```bash
cd ~/projects/lay && bash update.sh
```

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![GNOME](https://img.shields.io/badge/GNOME-45--47%2C%2050-4A86CF?logo=gnome)](https://gnome.org/)
[![Wayland](https://img.shields.io/badge/Wayland-native-blue)](https://wayland.freedesktop.org/)
[![Status](https://img.shields.io/badge/status-alpha-yellow)](#status-alpha)
[![License: MIT](https://img.shields.io/badge/License-MIT-green)](#license)

</div>

## Русский

### Что такое lay

`lay` — локальный помощник для Linux, который чинит текст, набранный не в той
RU/EN раскладке.

Главное действие:

```text
Набрал:  ghbdtn
Нажал:   Shift Shift
Стало:   привет
```

![lay demo](docs/publicity/demo.gif)

Это не облачный автокорректор и не чат-бот. `lay` работает рядом с клавиатурой:
слушает физические клавиши локально, помнит короткий хвост набора и при
команде перепечатывает его в другой раскладке. Буфер обмена для основного
сценария не используется.

По умолчанию double Shift исправляет **1 последнее слово**. Это самый быстрый и
предсказуемый режим. Области `2 слова` и `3 слова` включаются отдельно в трее,
если нужно работать со смешанными фразами.

Автоматическая помощь после пробела тоже есть, но она намеренно осторожная:
если сигнал слабый или вариантов несколько, нормальное поведение — не трогать
текст.

### Для чего это нужно

- Быстро вернуть слово, набранное не в той раскладке.
- Писать русский и английский вперемешку без постоянного удаления текста.
- Исправлять явные layout-ошибки после пробела, если включена автопомощь.
- Держать работу локально: без облака, без обязательной модели, без отправки
  набранного текста наружу.
- Настроить поведение под себя через трей: область, режим исправления,
  автопомощь, автоподмена, фиксированная раскладка для окон.

### Быстрый старт

Установить:

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

Обновить:

```bash
cd ~/projects/lay && bash update.sh
```

### Что умеет

Ручное исправление по double Shift:

- исправляет последнее слово в другой раскладке;
- по умолчанию берёт только `1 слово`;
- может работать с `2` или `3` словами, если это включено в настройках;
- остаётся полезным даже при полностью выключенной автокоррекции.

Smart-режим:

- старается не перепечатывать уже нормальные соседние слова;
- полезен для смешанного текста вроде `good ntrcn -> good текст`;
- использует локальные словари, LEM/ngram scoring и защиту хороших токенов.

Помощь при наборе после пробела:

- исправляет только завершённый хвост;
- чинит сильные RU/EN layout-сигналы и простые опечатки;
- сохраняет пробел после автозамены;
- не должна переписывать стиль, смысл или длинные фразы.

Автоподмена:

- работает по точным пользовательским правилам;
- хранится в `~/.config/lay/replacements.json`;
- подходит для повторяющихся личных ошибок, но не для широких догадок.

Дополнительные режимы:

- `ptah_alexs`: жёсткая раскладка для выбранных окон;
- прямые хоткеи RU/EN;
- optional multi-tap Shift scope;
- экспериментальный IME backend;
- optional Smart/LLM fallback для исследований, не основной путь.

### Что включено по умолчанию

Для новой установки базовый сценарий должен быть безопасным:

- double Shift включён;
- область double Shift — `1 слово`;
- автоматическая помощь после пробела выключена, пока пользователь сам её не
  включит;
- точная автоподмена выключена, пока пользователь сам её не включит;
- основной вывод идёт через `uinput`;
- сетевые LLM/API не используются.

Настройки живут в:

```text
~/.config/lay/config.json
```

Главные переключатели доступны в трее.

### Поддержка окружений

Основная проверенная среда: Ubuntu/GNOME Wayland с RU/EN раскладками.

Также есть backend'ы для KDE/Plasma и X11:

- GNOME: Shell extension, tray и DBus bridge для переключения раскладки;
- KDE/Plasma: отдельный `lay-kde-tray` и переключение через `qdbus6`;
- X11: native XKB backend через `x11rb`, fallback tools только как запасной
  путь.

KDE и X11 уже рабочие, но матрица чужих дистрибутивов и версий пока меньше,
чем у GNOME. Sway/Hyprland и раскладки кроме RU/EN пока не обещаются.

### Языки и словари

Текущая цель проекта — качественная пара **RU/EN**.

Используется:

- физическое соответствие US ↔ RU;
- Hunspell-словари, если они есть в системе;
- локальные RU/EN правила;
- char n-gram и LEM scoring;
- пользовательские точные замены;
- пользовательский список защищённых слов.

Не заявлено как готовое:

- другие пары раскладок;
- полноценная грамматика русского языка;
- “исправить весь абзац”;
- подсказки серым текстом прямо внутри поля ввода.

### Статус alpha

`lay` уже можно использовать каждый день, но проект всё ещё alpha.

Стабильное ядро: ручной double Shift и локальная RU/EN конвертация.

Активно оттачиваются:

- автопомощь после пробела;
- mixed RU/EN сценарии;
- KDE/X11 edge cases;
- работа в старых/особых текстовых полях вроде мессенджеров, терминалов и
  Electron/CEF-приложений;
- экспериментальный IME backend.

Если присылаешь баг-репорт, лучше указать:

- что набрано;
- что ожидалось;
- что получилось;
- GNOME/KDE/X11 и версия системы;
- включены ли `Помощь при наборе`, `Автоподмена`, `Smart`, LEM.

Приватный текст перед отправкой лучше заменить на безопасный пример.

### Похожие проекты

`lay` находится рядом с Punto/Caramba-style инструментами, но сделан под Linux
desktop constraints:

- **Punto / Caramba Switcher** — главный пользовательский ориентир: быстро
  исправить слово, набранное не в той раскладке.
- **xneur** — исторически важный X11-проект, но `lay` строится вокруг
  evdev/uinput, GNOME Wayland и общего backend-слоя.
- **easy-switcher**, **NSkbd**, **Tapper**, **Mahou** и похожие инструменты
  закрывают соседние сценарии: hotkeys, layout switching, Windows-подходы или
  более агрессивную автоматику.

Ниша `lay`: лёгкий локальный RU/EN helper, где ручной double Shift остаётся
главным действием, а автопомощь можно включать ровно настолько, насколько она
не мешает.

### Установка

Обычная установка ставит всё сразу: Rust-бинарники, user systemd-сервис
`lay-daemon` и GNOME Shell extension.

Короткая установка одной командой:

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

Она поставит базовые зависимости, Rust, скачает `lay` в `~/projects/lay` и
запустит `install.sh`.

Установщик умеет ставить системные пакеты через `apt`, `pacman`, `dnf` и `yum`.
На неподдерживаемом дистрибутиве он не угадывает пакеты сам: поставь Rust,
git/curl/build tools, XCB, `wl-clipboard` и `xclip` вручную, затем запусти
`bash install.sh`.

Ручной вариант:

```bash
git clone https://github.com/radislabus-star/lay-public.git ~/projects/lay
cd ~/projects/lay
bash install.sh
```

После установки выйди из сессии и зайди снова. Это нужно, чтобы применились
группа `input`, GNOME extension или KDE tray.

Потом набери слово не в той раскладке и нажми **Shift два раза**.

### Обновление

Если `lay` уже установлен стандартным способом в `~/projects/lay`, обновление
одной командой:

```bash
cd ~/projects/lay && bash update.sh
```

Скрипт сначала проверяет удалённую ветку. Если новых коммитов нет, он пишет,
что версия актуальна, и ничего не пересобирает. Если обновления есть, делает
`git pull --ff-only`, пересобирает release-бинарники, обновляет GNOME extension
и перезапускает `lay-daemon`.

Для KDE/X11 используется та же команда. GNOME extension там не нужен, но
`update.sh` всё равно обновит бинарники и перезапустит `lay-daemon`. В KDE он
также обновит KDE tray. Трей в KDE запускается через desktop autostart:
`~/.config/autostart/lay-kde-tray.desktop`, поэтому после перезагрузки он
появляется вместе с графической сессией.

То же действие есть в трее:

- GNOME: `Проверить обновления`;
- KDE: `Проверить обновления`.

Кнопка открывает терминал, показывает ход проверки/установки и пишет лог сюда:

```text
~/.local/state/lay/update.log
```

### X11 diagnostics

Если проверяешь X11 backend на новой реальной X11-сессии,
сначала собери короткий отчёт:

```bash
lay-test-input x11-diagnostics
```

Команда не создаёт виртуальную клавиатуру и ничего не печатает в активное окно.
Она показывает выбранный backend, `XDG_SESSION_TYPE`, `DISPLAY`, состояние
native XKB через `x11rb` и доступность fallback tools: `xkb-switch`,
`xkblayout-state`, `setxkbmap`.

Для GitHub issue можно сразу получить готовый блок отчёта:

```bash
lay-test-input x11-report
```

Для ручного smoke-test выстави:

```json
{
  "layout_backend": "x11"
}
```

После этого перезапусти демон:

```bash
systemctl --user restart lay-daemon
```

Проверка: набрать `ghbdtn`, нажать Shift два раза, ожидаемый результат —
`привет`.

### Что нового

Последние изменения публичной ветки:

- `0.1.193` — усилен общий safety-контракт замены текста: decoder и runtime
  проверяют, что edit-plan действительно превращает исходный хвост в ожидаемый
  результат, а after-space/Enter коррекции сохраняют пробельную границу.
  `lay --explain-correct` теперь показывает второго кандидата, `margin` и
  `confidence`; full-check и GitHub CI проверяют этот диагностический путь.
- `0.1.192` — `protected_words.txt` теперь защищает и ASCII-токены, включая
  короткие слова вроде `vs`, а раздел `Коррекция` в GNOME tray снова вынесен
  отдельным пунктом, а не спрятан внутрь `Арбитра`. `Pause` также доступен
  как основной триггер исправления, а не только как прямой RU/EN хоткей.
- `0.1.191` — live-автозамена получила контекстный `layout_en_to_ru` слой:
  одиночное `'nj` и смешанный текст вроде `worked 'nj` теперь исправляются в
  `это`, но технические хвосты вроде `git`, `wi-fi`, `status;` не запускают
  рискованную автоподмену. Добавлен короткий regression-набор из четырёх
  смешанных предложений, где каждое второе слово набрано в противоположной
  раскладке.
- `0.1.190` — начат архитектурный разнос ядра: lexical data вынесены из
  production-правил в `data/lexicon`, добавлен platform-neutral `engine`
  для manual correction, а GNOME daemon начал использовать этот общий слой.
  Также усилен общий split/scoring для склеенных коротких русских слов без
  добавления частных слов в runtime-код. Normal-режим автозамены стал
  осторожнее: спорные грамматические догадки оставлены для experimental.
- `0.1.189` — after-space автозамена больше не теряется при быстром наборе:
  daemon запускает проверку сразу на нажатии пробела, удерживая физический
  ввод на время замены, чтобы пробел в конце сохранился.
- `0.1.188` — автозамена снова строго after-space: `?`/`!` сами по себе не
  запускают typing assist. Если физический Shift ещё зажат после `Shift+7`,
  daemon ждёт отпускания Shift и только потом печатает исправление своим
  виртуальным вводом.
- `0.1.187` — typing assist научился чинить текущее слово после завершающих
  `?`/`!` без ожидания следующего пробела. Это закрывает класс ошибок вроде
  случайного внутреннего регистра `улУЧШИТЬ?` → `улучшить?`, но незавершённое
  слово без пунктуации по-прежнему не трогается во время набора. В `0.1.188`
  этот эксперимент отменён: реальная автозамена ждёт пробел.
- `0.1.186` — релиз стабилизации typing assist после боевых логов:
  автозамена стала осторожнее с опасными догадками, Enter-autocorrect теперь
  реально управляется настройкой, добавлен откат и журнал последних действий,
  исправлены ложные правки вроде небезопасного `ь -> ъ`, съедания пробела после
  автозамены и разрезания слов по коротким предлогам (`из водитель`-подобные
  случаи).
- `0.1.185` — добавлен компактный журнал последних действий и режим
  осторожности коррекции. В трее появились `Последние действия` и выбор
  `Строго / Норма / Эксп.`, а manual smart-replace теперь сохраняет полный
  undo-контекст для отката результата.
- `0.1.184` — исправлен layout после минимальной замены текущего хвоста.
  Сценарии вроде русского `;`, набранного как `Shift+4`, теперь заменяются на
  `$` и оставляют активную раскладку по вставленному хвосту, а не по всей
  окружающей русской фразе.
- `0.1.183` — auto-undo переведён на общий `TextReplacement` insert helper.
  Откат автозамены теперь использует тот же путь вставки текста и
  восстановления курсора, что typing assist, Enter-autocorrect и manual
  smart-replace.
- `0.1.182` — общий helper для успешного layout replay. IME-replay,
  GNOME-native replay и обычный uinput replay теперь одинаково обновляют
  replay-toggle buffer и learning-log, поэтому эти пути меньше рискуют
  разъехаться при следующих правках.
- `0.1.181` — manual smart-replace теперь тоже записывает learning и
  replay-memory через общий helper. IME/GNOME-native/uinput ветки больше не
  дублируют этот код; расширенный fallback для запоминания вставленного хвоста
  остаётся только в uinput-пути, где он уже был нужен для обратимого
  double-Shift.
- `0.1.180` — общий helper для layout switch/restore после text edit. Ветки
  typing assist и Enter-autocorrect больше не держат четыре копии одной
  логики: если включено автопереключение, оставляем layout под исправленный
  текст; если выключено — возвращаем исходную раскладку.
- `0.1.179` — daemon получил общий uinput edit executor для
  `TextReplacement`: typing assist, Enter-autocorrect и manual smart-replace
  теперь используют один путь вставки текста и восстановления курсора. Также
  общий helper записывает feedback/undo/replay-memory после assisted-правок,
  чтобы эти ветки не расходились по поведению.
- `0.1.178` — manual smart decoder теперь возвращает готовый
  `TextReplacement` plan вместе с решением. Daemon больше не пересобирает
  минимальную замену отдельно: один decoder-контракт решает, что менять и какой
  хвост стирать/вставлять. Regression-тесты проверяют, что в mixed 2/3-word
  случаях заменяется только BAD-хвост, например `good ntrcn`, `hello good ntrcn`
  и `делай KDE`.
- `0.1.177` — decoder получил явный ranked-candidate слой: scoped-tail
  кандидаты ранжируются через LEM/ngram, победитель применяется только при
  достаточном margin, а tests теперь покрывают 2/3 слова, `KDE -> ЛВУ` в
  русском контексте и отключение ranked-пути без LEM-флага.
- `0.1.176` — начат переход на явный decoder/edit-plan слой: manual
  double-Shift, typing assist и Enter-autocorrect теперь используют общий
  decision contract. Добавлен regression corpus для обратимого single-word
  toggle, mixed RU/EN пар и пробельных границ. Backend-слой получил
  capability-контракт для будущего атомарного IME/text replace.
- `0.1.170` — README переписан в продуктовый стиль без авторской истории;
  добавлен большой раздел сценариев использования и настроек. Публичные
  README/HOW_IT_WORKS синхронизированы со статусом поддержки: проект больше не
  описывается как GNOME Wayland only, KDE Plasma и X11 отмечены как проверенные
  backend'ы с меньшей матрицей покрытия, чем GNOME.
- `0.1.169` — добавлен GitHub Actions CI для публичных PR: форматирование,
  Rust-тесты, clippy, проверка GNOME extension, KDE tray/shell syntax,
  release build и LEM probe.
- `0.1.168` — переключатели в меню трея больше не закрывают меню при клике;
  при перестройке меню открытый раздел восстанавливается. Идея взята из PR
  #13, реализация перенесена вручную без merge.
- `0.1.167` — расширена runtime-регрессия для ведущих CLI-токенов: теперь
  тестом покрыты и `-b`, и `+x`-подобные случаи, чтобы следующий текст после
  такого токена нормально начинал новый WordBuffer.
- `0.1.166` — `lay-test-input x11-report` печатает готовый GitHub-ready блок
  для проверки экспериментального X11 backend: distro/session/backend/tools и
  поля ручного smoke-test.
- `0.1.165` — CLI-флаги и похожие токены, начинающиеся с ведущего `-` или
  `+`, больше не попадают в WordBuffer как голая буква. Это закрывает ложную
  автоподмену `git checkout -b`, где `b` могло восприниматься как отдельное
  русское `в`.
- `0.1.164` — синхронизирован `HOW_IT_WORKS.md` с текущей архитектурой:
  KDE adapter уже существует, X11 описан как native XKB через `x11rb`, а не
  как tools-first backend.
- `0.1.163` — расширен потоковый mixed-corpus regression test: проверяется,
  что пары вида `я язык`, `в версии`, `и идея`, `с системой` не разрезаются
  typing assist после пробела.
- `0.1.162` — typing assist больше не отрывает повторяющуюся первую букву у
  следующего слова после односимвольных служебных слов. Регрессия закрывает
  живой сценарий `я язык НАПИШИ`, который не должен превращаться в `я зык`.
- `0.1.161` — typing assist больше не разрезает валидное слово `язык` в
  `я зык`. Для разрезания после одиночного `я` правая часть теперь должна быть
  действительно самостоятельным частым словом, а не generated-only формой.
- `0.1.160` — добавлены регрессии на грамматически валидные формы вроде
  `нужна`, `она нужна`, `важна`, `важно`: typing assist не должен менять одну
  нормальную словоформу на другую без уверенного сигнала.
- `0.1.159` — typing assist больше не склеивает короткий русский предлог слева
  с нормальным кириллическим словом справа. Добавлена регрессия на `про сою`,
  чтобы это не возвращалось как `просою`.
- `0.1.158` — optional Smart/LLM arbiter получил настраиваемые backend/url:
  `ollama`, `direct`, `openai`, `anthropic`. API-ключи читаются только из env,
  не из `config.json`. По умолчанию backend остаётся `off`.
- `0.1.157` — `lay-test-input` теперь устанавливается в `~/.local/bin` и умеет
  печатать `x11-diagnostics`: backend, X11 env, native `x11rb` XKB status и
  fallback tools. Это нужно для нормальных отчётов по экспериментальному X11.
- `0.1.156` — optional multi-tap Shift scope включён в runtime за выключенным
  по умолчанию флагом: `2/3/4` тапа выбирают `1/2/3` слова. Default double
  Shift остался мгновенным. Также добавлена регрессия на `изменю параметры`,
  чтобы typing assist не склеивал валидные русские слова.
- `0.1.155` — зафиксирован design contract для optional multi-tap Shift scope:
  `2/3/4` тапа соответствуют `1/2/3` словам, runtime-фича пока выключена.
- `0.1.154` — typing assist больше не автопереключает shell/CLI флаги вроде
  `-f`, `-r`, `-c`, `-n` и `--color=auto` в кириллицу после пробела.
- `0.1.153` — добавлена проверка совместимости старого `config.json` без
  новых force-layout полей: режим остаётся выключенным и получает безопасные
  значения по умолчанию.
- `0.1.152` — добавлены опциональные прямые хоткеи языка: отдельная клавиша
  может всегда включать RU, другая всегда включать EN. По умолчанию выключено;
  работает через тот же backend, что и double Shift: GNOME/KDE/X11.
- `0.1.151` — X11 backend теперь сначала использует native XKB через
  `x11rb`, без запуска `setxkbmap` на каждое исправление. Старые
  `xkb-switch`/`xkblayout-state`/`setxkbmap` оставлены как fallback.
- `0.1.150` — исправлены ложные auto-layout с русскими дефисными словами
  вроде `что-то`; GNOME indicator больше не создаёт timestamp-id вида
  `lay-177...`; установщик получил поддержку зависимостей для `apt`,
  `pacman`, `dnf` и `yum`.
- `0.1.149` — KDE tray теперь ставится через desktop autostart
  `~/.config/autostart/lay-kde-tray.desktop`. Это чинит ситуацию, когда после
  рестарта KDE daemon работает, а значок Lay в трее не появляется.
- `0.1.147` — после ручного double Shift следующий пробел больше не запускает
  автопомощник на только что перевёрнутом слове. Это исправляет сценарий
  `и -> b`, где автоподмена могла сразу вернуть `b` в `в`.
- `0.1.146` — smart-scope больше не переворачивает завершённую одиночную
  кириллическую букву слева от текущего слова. Исправлен случай вроде
  `й Сщсф -> й Coca`.
- `0.1.145` — добавлен optional host VM guard для тестирования KDE в VM:
  хостовый daemon можно автоматически гасить, пока открыт VM viewer, чтобы
  double Shift не обрабатывался одновременно хостом и гостем.
- `0.1.144` — daemon перестал слушать служебные виртуальные клавиатуры
  `lay-virtual-keyboard` и `ydotoold virtual device`, чтобы убрать фантомные
  повторы при тестах и desktop-интеграции.

### Extension ZIP

Для ручной установки GNOME-расширения или отправки на extensions.gnome.org можно
собрать ZIP:

```bash
bash scripts/package-extension.sh
```

Архив появится в:

```text
dist/gnome-extension/lay@radislabus-star.github.io-<version>.zip
```

Установить только расширение можно так:

```bash
gnome-extensions install --force dist/gnome-extension/lay@radislabus-star.github.io-<version>.zip
gnome-extensions enable lay@radislabus-star.github.io
```

Но для полной работы double Shift всё равно нужен `lay-daemon`, поэтому для
обычных пользователей предпочтителен `bash install.sh`.

### Требования

- Linux
- GNOME Shell 45, 46, 47 или 50 либо KDE Plasma 6
- Wayland-сессия для GNOME/KDE или X11-сессия для X11 backend
- Rust 1.75+
- доступ к `/dev/input` через группу `input`
- доступный `/dev/uinput` для обратной печати
- IBus и `python3-gi`, если включаешь экспериментальный `text_backend = "ime"`

Для KDE backend нужен `qdbus` или `qdbus6`; для KDE tray нужен PyQt6.
Установщик ставит эти пакеты в Plasma-сессии и поддерживает Ubuntu/Debian,
Arch/Manjaro и Fedora/RHEL-like системы через `apt`, `pacman`, `dnf` или `yum`.
X11 backend использует pure-Rust `x11rb` для прямых XKB-запросов. Если native
XKB недоступен, daemon пробует старые fallback tools: `xkb-switch`,
`xkblayout-state` и `setxkbmap`.

Установщик может добавить текущего пользователя в группу `input` и поставить
udev-правило для `/dev/uinput`, но группа начинает работать только после нового
входа в систему.

### CLI

`lay` можно использовать и из терминала:

```bash
lay "Ye djn ghbvth"
# Ну вот пример

lay "руддщ цщкдв"
# hello world

echo "ghbdtn" | lay
# привет

lay --clipboard
```

CLI удобен для быстрых проверок, скриптов и конвертации буфера обмена.

### Демон

`lay-daemon` — фоновый сервис, который делает двойной Shift рабочим в обычных
приложениях.

Полезные команды:

```bash
systemctl --user status lay-daemon --no-pager
systemctl --user restart lay-daemon
systemctl --user stop lay-daemon
journalctl --user -u lay-daemon -n 120 --no-pager
```

### GNOME-расширение

Демон читает физические клавиши и перепроигрывает keycode-события, но на GNOME
Wayland переключение раскладки требует интеграции с GNOME Shell.

Исходники расширения:

```text
extension/lay@radislabus-star.github.io/
```

Установленная копия:

```text
~/.local/share/gnome-shell/extensions/lay@radislabus-star.github.io/
```

### KDE/Plasma tray

В KDE используется отдельный frontend:

```text
scripts/lay-kde-tray.py
~/.config/autostart/lay-kde-tray.desktop
```

Установленная команда:

```bash
~/.local/bin/lay-kde-tray --status
```

После установки KDE tray стартует через desktop autostart вместе с Plasma.
Отдельный `lay-kde-tray.service` больше не используется как основной механизм
автозапуска.

Tray читает и пишет тот же файл:

```text
~/.config/lay/config.json
```

Через меню KDE можно включать/выключать daemon, `Smart correction`, область
1/2/3 слова, `Typing assist`, `Auto-replace`, `Auto-switch layout`,
`Remember corrections` и LEM-арбитр.

Если ты тестируешь KDE внутри виртуальной машины с хоста, где тоже установлен
`lay`, включи host VM guard. Он временно останавливает **хостовый** daemon,
пока запущен VM viewer, чтобы double Shift не обрабатывался дважды:

```bash
systemctl --user enable --now lay-host-vm-guard.service
```

Демон внутри гостевой KDE при этом должен быть включён: именно он видит
гостевую раскладку и печатает обратно в приложения VM.

### Меню в трее

Меню держит основной сценарий коротким:

- `Помощь при наборе`: осторожная правка после пробела.
- `Автоподмена`: точные пользовательские правила и typo-правки.
- `Запоминать правки`: opt-in лог подтверждённых исправлений.
- `Режим`: Replay или Smart.
- `Область`: сколько слов брать для ручного double Shift, по умолчанию `1`.
- `Арбитр`: LEM и auto-layout настройки.
- `ptah_alexs`: жёсткая раскладка по окну.
- `Коррекция`: включение/выключение слоёв помощника.
- `Триггер`, `Тайминг`, `Daemon`, `О программе`: сервисные настройки.

В публичном режиме нет кнопки для открытия сырого debug-лога.

### Как это работает

При двойном Shift:

```text
физическая клавиатура -> evdev -> lay-daemon
                                  |
                                  v
                           буфер текущего слова
                                  |
                                  v
                        Backspace x длина слова
                                  |
                                  v
                 desktop backend переключает раскладку
                 GNOME extension или KDE qdbus6
                                  |
                                  v
                    uinput повторяет исходные keycode
```

То есть `lay` не вставляет “готовое слово” из облака или буфера. Он повторяет
те же физические клавиши уже под другой раскладкой. Поэтому `Ghbdtn` становится
`Привет`.

Общая логика живёт в библиотеке:

```text
src/core.rs       общий facade для frontend-ов
src/config.rs     единая схема config для daemon/tray/frontend-ов
src/correction.rs общий результат исправления: replay или вставка текста
src/decoder.rs    общий decision layer: manual/typing-assist/enter-autocorrect
src/desktop.rs    определение GNOME/KDE/X11 backend и layout-id helpers
src/dict.rs       RU/EN keyboard mapping
src/keyboard.rs   keycode-события, word split, replay-decision, US/RU mapping и text→uinput runs
src/word_buffer.rs история текущего/предыдущих слов, replay-toggle и feedback для обучения
src/lem.rs        арбитр кандидатов
src/ngram.rs      локальный scorer естественности
src/typing_candidate.rs ranking, margin и confidence для автопомощи
src/text_edit.rs  минимальный план замены текста без лишней перепечатки
src/text_backend.rs выбор uinput/IME способа применения готовой правки
```

Desktop-специфичная часть остаётся отдельно: GNOME использует Shell extension,
KDE/Plasma использует `lay-kde-tray`, а daemon выбирает backend для
переключения раскладки.

### Помощь при наборе

Дополнительно `lay` умеет после пробела запускать консервативный помощник.
Это отдельная функция, не основной double-Shift сценарий.

Помощник специально осторожный:

- проверяет только завершённые слова;
- исправляет только уверенные локальные ошибки;
- использует точные правила, словари и char n-gram scorer;
- не переписывает стиль и не генерирует новый текст;
- если не уверен, ничего не делает.

Примеры задуманных исправлений:

```text
ошисбя -> ошибся
я вно  -> явно
плозо  -> плохо
```

Включается и выключается в трее:

```json
{
  "typing_assist": true,
  "auto_switch_layout": true
}
```

`auto_switch_layout` управляет автоматическими layout-правками после пробела:
если слово уверенно похоже на набор в неправильной раскладке, helper заменит
его и оставит активной раскладку исправленного текста. Ручной double Shift
переключает раскладку всегда.

Для диагностики решения можно использовать CLI:

```bash
lay --explain-correct 'кторое '
```

Вывод показывает все правила, кандидатов, победителя, второго кандидата,
`margin` и `confidence`. Это помогает понять, почему автопомощь сработала или
почему она промолчала.

### Прямое включение RU / EN

Double Shift исправляет уже набранный хвост. Отдельно можно включить режим, где
одна клавиша всегда ставит русскую раскладку, а другая всегда ставит английскую.
Это не toggle, а именно принудительное состояние.

По умолчанию режим выключен:

```json
{
  "force_layout_hotkeys": false,
  "force_ru_key": "single-rctrl",
  "force_en_key": "single-ralt"
}
```

В трее это находится в разделе `Прямой язык`. Настройка не меняет основной
double Shift и использует текущий `layout_backend`.

### Автоподмена

Точные автоподмены лежат здесь:

```text
~/.config/lay/replacements.json
```

Пример:

```json
{
  "подлючись": "подключись",
  "Надйи": "Найди"
}
```

Это именно точные правила. Нечёткие исправления относятся к typing assist, а не
к словарю автоподмены.

### Режим ptah_alexs

`ptah_alexs` — это не память последней раскладки окна. Это жёсткая политика:
когда конкретное окно получает фокус, `lay` ставит назначенную раскладку.

Пример:

```text
Terminal -> EN
Browser  -> не трогать
```

Если терминал получил фокус, `lay` снова поставит EN, даже если раньше внутри
него случайно включали RU. Правила задаются из трея: `ptah_alexs -> Текущее
окно -> EN/RU/keep`.

Конфиг хранится локально:

```json
{
  "ptah_alexs_mode": true,
  "ptah_alexs_rules": [
    {"kind": "app_id", "value": "org.gnome.Terminal.desktop", "layout": "us", "label": "Terminal"}
  ]
}
```

### Приватность

К клавиатурным инструментам нужно относиться подозрительно. `lay-daemon` видит
клавиатурные события, поэтому модель данных сделана максимально скучной и
локальной.

По умолчанию `lay` никуда не отправляет набранный текст. Нормальный путь
double Shift не требует сети, облачных API или удалённой модели.

Опциональный learning log локальный. Он должен хранить пары подтверждённых
исправлений, а не полный поток набора. По умолчанию он выключен и включается в
трее через `Данные -> Запоминать правки`:

```text
~/.local/share/lay/corrections.jsonl
```

Дополнительные локальные файлы:

```text
~/.local/share/lay/recent_actions.jsonl
~/.local/share/lay/learning_candidates.json
~/.local/share/lay/stats.json
```

`recent_actions.jsonl` — короткий кольцевой журнал последних успешных действий
для диагностики в трее. `learning_candidates.json` хранит только кандидаты для
консервативного повышения повторяющихся правок в точные правила.
`stats.json` хранит только счётчики и timestamps: вызовы LLM, записи обучения,
пользовательские исправления и повышенные правила. На Unix эти файлы создаются
с правами `0600`.

Диагностический вывод тоже выключен по умолчанию. Разработчик может включить
его явно через `lay-daemon --debug-log` или `LAY_DEBUG_LOG=1`.

GNOME-расширение публикует session-local DBus bridge, чтобы `lay-daemon` мог
переключать раскладку и делать fallback-вставку текста. Это не является
security boundary против других процессов того же desktop-пользователя.

Остановить демон можно в любой момент:

```bash
systemctl --user stop lay-daemon
```

### Smart/LLM режим

Есть экспериментальный `--smart` режим, где локальная модель может быть
арбитром между уже подготовленными кандидатами.

Это не главный путь продукта, не обязательная часть double Shift и не включено
для обычного исправления раскладки.

`Ещё -> LLM` в трее влияет только на `lay-daemon`. CLI использует модельную
логику только если передан `--smart`.

Обычная сборка не компилирует direct GGUF backend и не грузит модель при
старте. Для экспериментов с Ollama:

```bash
LAY_LLM_BACKEND=ollama lay --smart "fyukbqcrbq"
```

Ollama endpoint можно переопределить:

```bash
LAY_LLM_BACKEND=ollama \
LAY_OLLAMA_URL=http://localhost:11434/api/generate \
lay --smart "fyukbqcrbq"
```

Внешние API тоже поддержаны, но только явно. Ключи держи в переменных
окружения, не в `config.json`:

```bash
LAY_LLM_BACKEND=openai \
LAY_OPENAI_API_KEY=... \
LAY_MODEL=gpt-4o-mini \
lay --smart "fyukbqcrbq"

LAY_LLM_BACKEND=anthropic \
LAY_ANTHROPIC_API_KEY=... \
LAY_MODEL=claude-3-5-haiku-latest \
lay --smart "fyukbqcrbq"
```

Часть настроек можно положить в `~/.config/lay/config.json`; секреты всё равно
остаются только в env:

```json
{
  "llm_backend": "off",
  "llm_model": "smollm:135m",
  "llm_ollama_url": "http://localhost:11434/api/generate",
  "llm_openai_url": "https://api.openai.com/v1/chat/completions",
  "llm_anthropic_url": "https://api.anthropic.com/v1/messages",
  "llm_timeout_secs": 3
}
```

Для optional direct GGUF backend:

```bash
cargo build --release --features direct-llm
LAY_LLM_BACKEND=direct LAY_GGUF_MODEL=/path/to/model.gguf lay --smart "fyukbqcrbq"
```

### Разработка

```bash
cargo test
cargo build --release
```

N-gram helpers:

```bash
cargo run --bin lay-ngram-corpus -- check-cache
cargo run --bin lay-ngram-corpus -- check --corpus corpus/ru_50mb.txt
```

Установка текущей сборки локально:

```bash
bash install.sh
```

### Roadmap

- Uninstall-команда и более дружелюбный release-пакет.
- Короткий demo GIF/video для double Shift.
- Больше регрессионных тестов из реальных принятых/отклонённых исправлений.
- Ещё более понятные privacy-настройки.
- Расширить матрицу проверок KDE/X11 на чужих системах и версиях Plasma.
- Исследование Sway/Hyprland.
- Другие раскладки после стабилизации RU/EN.

## English

`lay` is a lightweight keyboard helper for Linux users who type in two layouts,
especially **RU/EN**.

The main workflow:

```text
Typed:   Ghbdtn
Press:   Shift Shift
Result:  Привет
```

`lay` is primarily tested on GNOME Wayland, but it is no longer GNOME-only. It
uses Rust, evdev/uinput, and a layout backend layer. GNOME uses a small Shell
extension; KDE/Plasma uses a separate tray plus `qdbus6`; X11 uses native XKB
through `x11rb`. The normal path is local-first: no cloud service, no network
call, and no model required.

Quick install:

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

Manual install:

```bash
git clone https://github.com/radislabus-star/lay-public.git ~/projects/lay
cd ~/projects/lay
bash install.sh
```

After installation, log out and log back in so the `input` group, `/dev/uinput`
permissions, and GNOME extension are picked up.

Update an existing git install:

```bash
cd ~/projects/lay && bash update.sh
```

Extension ZIP for manual install or extensions.gnome.org upload:

```bash
bash scripts/package-extension.sh
gnome-extensions install --force dist/gnome-extension/lay@radislabus-star.github.io-<version>.zip
gnome-extensions enable lay@radislabus-star.github.io
```

The extension alone is only the GNOME Shell bridge and tray UI. The full
double-Shift workflow also needs `lay-daemon`, so normal users should use
`bash install.sh`.

Supported/tested target:

- Linux
- GNOME Shell 45, 46, 47, or 50
- KDE Plasma 6 through the KDE tray/backend
- GNOME/KDE Wayland sessions, plus X11 through the X11 backend
- Rust 1.75+
- RU/EN layouts and dictionaries

Validated with smaller coverage than GNOME:

- KDE backend via `qdbus/qdbus6 org.kde.keyboard /Layouts setLayout`.
- X11 backend via native XKB (`x11rb`), with `xkb-switch`, `xkblayout-state`,
  and `setxkbmap` kept as fallback tools.
- `ptah_alexs` window policy mode for GNOME: force a selected window/app to RU,
  EN, or keep.

Useful CLI examples:

```bash
lay "Ye djn ghbvth"
# Ну вот пример

lay "руддщ цщкдв"
# hello world

echo "ghbdtn" | lay
# привет
```

Language scope: current automatic correction is intentionally RU/EN-focused.
Other layout pairs are not promised yet. If you need another language pair,
please open an issue with concrete typed/expected examples and your desktop
environment.

Manual-only mode: disable `typing_assist`, `auto_replace`, and
`auto_switch_layout` if you want only the predictable double-Shift rescue
without automatic corrections after Space.

Privacy summary: `lay-daemon` reads keyboard events locally to provide the
double-Shift workflow. By default it does not send typed text anywhere, does not
require a remote model, and does not keep a full keylog. Optional learning logs
are local and disabled by default.

Experimental Smart/LLM mode exists, but it is not the default product path.
External providers are used only when `LAY_LLM_BACKEND=openai` or
`LAY_LLM_BACKEND=anthropic` is set explicitly and the matching API key is
present in env. The default build does not compile direct GGUF support; use
`--features direct-llm` and explicit `LAY_LLM_BACKEND=direct` only for local
model experiments.

## License

MIT
