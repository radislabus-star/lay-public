# Как работает lay

`lay` — локальный помощник ввода для Linux. Его главная задача: исправить
короткий хвост текста, набранный не в той раскладке, и сделать это без
clipboard, без облака и без попытки переписывать смысл текста.

Главный сценарий:

```text
Набрал:  ghbdtn
Нажал:   Shift Shift
Стало:   привет
```

Проект сейчас в статусе alpha: GNOME Wayland — основной проверенный путь,
KDE/Plasma, Niri и X11 поддерживаются как более молодые backend-слои.

## Компоненты

```text
lay CLI
  └─ разовая конвертация текста, explain/debug, smart CLI

lay-daemon
  ├─ слушает физическую клавиатуру через evdev
  ├─ держит короткий буфер слов
  ├─ ловит double Shift и другие триггеры
  ├─ вызывает decoder / typing pipeline
  └─ выводит исправление через uinput или экспериментальный IME path

Desktop интеграции
  ├─ GNOME Shell extension: tray, DBus bridge, GNOME layout activation
  ├─ KDE tray/helper: Plasma menu и qdbus layout backend
  ├─ Niri backend: прямой IPC через niri-ipc
  └─ X11 backend: native XKB через x11rb
```

`lay-daemon` не читает текст из активного приложения и не копирует его в буфер
обмена. Он восстанавливает хвост по собственному `WordBuffer`, который
собирается из физических key events.

## Архитектурный принцип

Код делится на три слоя:

```text
1. Core / decision
   Распознаёт хвост, строит варианты, выбирает действие.
   Не знает, что такое GNOME Shell, KDE, DBus или uinput.

2. Runtime / daemon
   Слушает evdev, ведёт состояние, вызывает core и backend.
   Не содержит словарных списков и не решает "на глаз".

3. Desktop / output backend
   Переключает раскладку и выводит готовое действие в приложение.
   Не принимает лингвистических решений.
```

Жёсткое правило: runtime-код не должен содержать пользовательские фразы как
правила. Лексика живёт в `data/lexicon/*`, тестовые сценарии — в
`tests/fixtures/*` и `data/test_input/*`, пользовательские правила — в
`~/.config/lay/*`.

## Основные модули ядра

- `src/core.rs` — компактный facade общего ядра.
- `src/config.rs` — единая схема настроек daemon/tray/CLI.
- `src/dict.rs` — детерминированная RU/EN keyboard-map конвертация.
- `src/decoder.rs` — manual double-Shift decision layer.
- `src/scoped_tail.rs` — выбор и ранжирование хвоста из 1-3 слов.
- `src/layout_autoswitch.rs` — автоперекладка ASCII↔кириллица.
- `src/typing_pipeline.rs` — общий pipeline автопомощи после пробела.
- `src/typing_rule_graph.rs` — список правил, приоритеты и включение правил.
- `src/typing_candidate.rs` — scoring кандидатов автопомощи.
- `src/typing_context.rs` — контекст короткого хвоста вокруг слова.
- `src/phrase_reader.rs` — разбор склеенных/разрезанных русских слов.
- `src/word_recognizer.rs` — классификация токенов: RU, EN, protected,
  technical, mixed, number.
- `src/lexicon.rs` — загрузка словарных данных и пользовательских protected
  words.
- `src/ru_typo.rs` — семейства русских опечаток.
- `src/russian_lexicon.rs` — Hunspell и русские формы.
- `src/ngram.rs` — char n-gram scorer.
- `src/text_edit.rs` — минимальные планы замены текста.
- `src/keyboard.rs` — key events, раскладочная карта и печать текста через
  uinput-события.

`src/typing_assist.rs` сейчас оставлен тонким публичным фасадом. Основная
логика уже вынесена в `typing_pipeline`, `typing_rule_graph`,
`typing_candidate`, `typing_context`, `phrase_reader`, `layout_autoswitch` и
`ru_typo`.

## Daemon runtime

`src/bin/lay_daemon.rs` — только точка сборки модулей. Поведение разнесено по
`src/bin/lay_daemon/*`:

- `daemon_runtime.rs` — главный event loop.
- `daemon_state.rs` — состояние процесса.
- `keyboard_io.rs` — evdev input.
- `physical_input_grab.rs` — временная изоляция физической клавиатуры при
  выводе.
- `trigger_fsm.rs` и `manual_trigger_runtime.rs` — double Shift / multi-tap FSM.
- `correction_runtime.rs` — исполнение решения decoder.
- `text_output.rs` — uinput output contract.
- `layout_controller.rs` — переключение раскладки через выбранный backend.
- `layout_kde.rs` — KDE/Plasma через qdbus/qdbus6.
- `layout_niri.rs` — Niri через прямой IPC к compositor socket.
- `layout_x11.rs` — X11 через native XKB/fallback tools.
- `focus_guard.rs` — защита буфера между окнами/полями.
- `pending_typing_assist.rs` — отложенная автопомощь после пробела.
- `learning_runtime.rs` — feedback и продвижение пользовательских правок.
- `action_log_runtime.rs` — recent actions для tray.
- `startup_runtime.rs` — warmup, backend probes, service startup.

Цель такого разреза: daemon не должен снова превращаться в файл на тысячи строк,
где смешаны ввод, вывод, правила, тесты и desktop-specific код.

## Почему evdev и uinput

На Wayland приложения изолированы. У обычного процесса нет универсального API:

- прочитать, что пользователь набирает в любом окне;
- получить слово под курсором;
- атомарно удалить и вставить surrounding text в любом toolkit.

Поэтому `lay-daemon` слушает `/dev/input/event*` через evdev. Это даёт
физические события клавиатуры до compositor/toolkit.

Для вывода используется `/dev/uinput`: виртуальная клавиатура ядра. Для
приложения это выглядит как обычный ввод с клавиатуры, поэтому работает шире,
чем `xdotool`/`wtype`.

Минусы этого пути:

- нужно членство пользователя в группе `input`;
- нужно право на `/dev/uinput`;
- вывод не является настоящим committed-text API;
- старые/особые окна могут терять часть событий, если backend ведёт себя
  нестандартно.

## Double Shift flow

```text
evdev key events
  ↓
WordBuffer
  ↓
trigger_fsm: press/release/press/release
  ↓
decoder
  ├─ replay all
  ├─ smart minimal replacement
  └─ keep original
  ↓
correction_runtime
  ├─ при необходимости переключить layout
  ├─ удалить нужный хвост
  ├─ вставить/replay исправление
  └─ синхронизировать WordBuffer с тем, что реально выведено
```

Ручной double Shift — явная команда пользователя. Поэтому последнее текущее
слово/фрагмент можно переворачивать даже тогда, когда словарь считает его
нормальным. Для соседних завершённых слов smart-режим действует осторожнее:
нормальные слова оставляет, плохие меняет только при уверенном сигнале.

## Typing assist после пробела

Автопомощь работает не на каждую клавишу, а после разделителя:

```text
пользователь нажал Space
  ↓
daemon ждёт release Space
  ↓
строится snapshot завершённого хвоста
  ↓
typing_pipeline выбирает кандидата
  ↓
если кандидат безопасен — замена включает исходный separator
```

Контракт пробела жёсткий: если пользователь нажал пробел и автозамена
сработала, итоговая замена обязана вернуть пробел в конец. Пробел не
“компенсируется” произвольными задержками. Если пробел съеден, ошибка ищется в
state/replacement contract.

Минимальная проверка этого класса:

```text
html djn  -> html вот
```

То есть после исправления первого токена буфер и экран должны оставаться
синхронными, а следующий токен должен нормально отделяться пробелом.

## Protected words

Файл пользователя:

```text
~/.config/lay/protected_words.txt
```

Слова из него являются стоп-сигналом для автоперекладки ASCII→RU. Это не
“маленький вес” и не подсказка scorer-у. Если пользователь защитил токен,
pipeline обязан оставить его как есть до любого L2/L3/ngram решения.

Пример:

```text
cd  -> cd
```

Даже если раскладочная замена `cd -> св` выглядит языково правдоподобной,
пользовательская защита сильнее scorer-а.

## Smart и LLM

По умолчанию `lay` не является LLM-приложением.

Основной путь:

```text
keyboard map
  + dictionaries
  + protected/technical-token recognizer
  + L2/L3 phase and n-gram scoring
  + conservative rule graph
```

LLM — optional arbiter для уже построенных вариантов. Она не должна свободно
писать новый текст вместо пользователя. В штатном публичном поведении модель не
загружается и не нужна для double Shift.

## NANDA Wave / Cell32

NANDA Wave — экспериментальный исследовательский слой поверх текущего
детерминированного движка. Он не является включённым runtime-автокорректором и
не заменяет safe replacement pipeline. Сейчас он нужен, чтобы изучать клетки,
волновые признаки, ablation и ансамблевые режимы на реальном наборе кейсов.

Текущая схема:

```text
fixtures / typing cases
  ↓
deterministic baseline
  ↓
NANDA Wave cells
  ↓
L1 sensors / L2 candidates / L3 consensus
  ↓
eval, ablation, status JSON
```

Базовая единица:

```text
Cell32 = 32 KB wave cell
```

Текущий исследовательский статус можно посмотреть так:

```bash
lay-nanda-wave-eval --status-json
lay-nanda-wave-eval --trace "djn "
lay-nanda-wave-eval --real-suite --ablation
```

Публичная UI-модель намеренно разделена:

- `Автоподмена` — общий рубильник автоматических исправлений после пробела;
- `NANDA ячейки` — отдельное окно со статусом клеток, ролями и волновой схемой;
- старого переключателя `Автокоррекция NANDA` больше нет: старый Expert64/
  microbrain runtime удалён.

Клетки не печатают напрямую в приложение. В текущем виде они работают как
исследовательский прибор:

- L1-сенсоры выделяют символьные, письменные, клавиатурные и граничные моды;
- L2-клетки строят кандидаты;
- L3-клетки согласуют или гасят решение;
- status JSON показывает gate, клетки, роли, delta и ablation.

Главный критерий развития NANDA — не “добавить ещё правил”, а доказать
ансамблевую моду: связка клеток должна давать пользу, которую одиночная клетка
не даёт, и эта польза должна исчезать при ablation ключевой клетки.

## Desktop backend-и

Настройка:

```json
{
  "layout_backend": "auto"
}
```

Значения:

- `auto` — выбрать backend по окружению;
- `gnome` — GNOME Shell extension + DBus bridge;
- `kde` — Plasma через `qdbus/qdbus6`;
- `niri` — Niri Wayland через прямой `niri-ipc`;
- `x11` — native XKB через `x11rb` и fallback tools.

GNOME Wayland — самый зрелый путь. KDE/Niri/X11 работают через те же core-модули,
но имеют отдельные desktop edge cases, поэтому остаются менее зрелыми.

## GNOME extension

GNOME extension теперь разделён на модули:

- `extension.js` — loader.
- `lay-impl.js` — tray indicator и сборка меню.
- `tray_support.js` — config, stats, update helpers, общие константы.
- `dbus_service.js` — session-local DBus service и layout helpers.
- `recent_actions_menu.js` — меню последних исправлений.
- `prefs.js` / `settings.js` — настройки.

Установщик и dev-reload копируют все `*.js` из extension-директории. CI делает
`node --check` для всех JS-модулей, чтобы новый файл не выпал из проверки.

## DBus model

GNOME Shell нужен только потому, что именно Shell умеет вызвать
`inputSources[i].activate()` внутри сессии GNOME Wayland.

Упрощённый путь:

```text
lay-daemon
  ↓ zbus session call
GNOME extension DBus service
  ↓
GNOME Shell inputSources[i].activate()
```

DBus внутри user session не является защитой от процессов того же пользователя.
Это локальный control channel между daemon и extension.

## Text output

Production path — uinput:

```text
delete tail with Backspace
switch layout if needed
type/replay replacement
sync WordBuffer
```

Если daemon успешно изолировал физическую клавиатуру (`evdev grab`), короткий
вывод можно делать быстрее: собственный виртуальный ввод не смешивается с
зажатым физическим Shift. Это заменило старую грубую задержку ожидания Shift.

Экспериментальный IME path существует и доступен через настройку `Режим ввода`,
но не является default:

```json
{
  "text_backend": "ime"
}
```

IME нужен для committed-text замен и preedit-кандидатов прямо в поле ввода.
Он остаётся экспериментальным backend-ом: быстрый uinput-путь по-прежнему
основной и должен работать независимо от IME.

## Learning

`learning_log` — opt-in.

Что пишется:

- явные ручные исправления double Shift;
- user-correction после того, как пользователь удалил результат auto/smart
  правки и ввёл свой вариант.

Что не пишется:

- полный поток набора;
- clipboard;
- приватный текст окон;
- обратные replay-toggle пары, которые только возвращают текст назад.

Локальные файлы пишутся приватно (`0600`), где это применимо:

- `~/.local/share/lay/corrections.jsonl`;
- `~/.local/share/lay/recent_actions.jsonl`;
- `~/.local/share/lay/learning_candidates.json`;
- `~/.local/share/lay/stats.json`.

## Data files

Production runtime не хранит большие списки в `match`/`if` по словам.

Основные данные:

- `data/lexicon/common_en_technical.txt`;
- `data/lexicon/common_ru.txt`;
- `data/lexicon/russian_*`;
- `data/test_input/builtin_scripts.tsv`;
- `tests/fixtures/*`.

Если нужен новый класс поведения, сначала добавляется общий rule/scorer/guard,
а конкретная фраза остаётся только в тестах.

## Test contract

Перед push минимальный gate:

```bash
cargo fmt --all
cargo test -q --all-targets
scripts/check-lay-lints.sh
cargo build --release --bins
git diff --check
for js in extension/lay@radislabus-star.github.io/*.js; do node --check "$js"; done
bash -n install.sh update.sh dev-reload.sh scripts/*.sh
```

Для KDE/Niri/X11 изменений дополнительно:

- clean-copy проверка в VM или отдельной папке;
- Rust tests/clippy/build на этой копии;
- smoke CLI/typing-assist кейсы;
- по возможности ручная проверка tray/backend в живой desktop-сессии.

## Known limitations

- Поддержка языков сейчас рассчитана на RU/EN.
- Слово под курсором без предварительного набора `lay` не читает.
- Выделенный текст не является текущим production path.
- IME/preedit уже есть как экспериментальный backend, но не должен становиться
  отдельным "мозгом": решение строит общий core, IME только отображает и
  применяет готовое действие.
- KDE/Niri/X11 моложе GNOME Wayland и требуют больше реальных отчётов.
- Автопомощь после пробела намеренно консервативна: лучше пропустить сомнительную
  правку, чем испортить нормальный текст.
