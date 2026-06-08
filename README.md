<div align="center">

# lay

**Double Shift RU/EN layout rescue для Linux**

`lay` чинит слово, набранное не в той раскладке: нажал **Shift два раза** и
продолжил писать.

**Статус: alpha.** Основной сценарий уже рабочий. Главная зона активной
доводки — автопомощь после пробела и редкие desktop edge cases.

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![GNOME](https://img.shields.io/badge/GNOME-45--47%2C%2050-4A86CF?logo=gnome)](https://gnome.org/)
[![Wayland](https://img.shields.io/badge/Wayland-native-blue)](https://wayland.freedesktop.org/)
[![Status](https://img.shields.io/badge/status-alpha-yellow)](#статус-alpha)
[![License: MIT](https://img.shields.io/badge/License-MIT-green)](#license)

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

По умолчанию double Shift исправляет **1 последнее слово**. Области `2 слова`
и `3 слова` можно включить отдельно в трее.

## Что нового в 0.1.217

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

## Что нового в 0.1.216

- исправлен mixed-word рассинхрон в KDE/Kate и других backend: если слово почти
  целиком набрано в одной раскладке, а последняя буква пришла из другой, lay
  нормализует слово в доминирующую раскладку;
- живой проверочный кейс: `привеn` по double Shift становится `привет`, без
  дробления на `приве` + `n`.

## Что нового в 0.1.215

- добавлен Niri layout backend через прямой `niri-ipc`;
- `auto` стал умнее: в KDE/Plasma VM с nested Niri он выбирает KDE, а не
  ошибается по старому `NIRI_SOCKET`;
- в GNOME и KDE tray добавлен ручной выбор среды раскладки:
  `auto / KDE / X11 / GNOME / Niri`;
- KDE tray показывает выбор среды в верхнем блоке меню и не закрывается после
  смены backend;
- Niri помечен как экспериментальный режим для реальной Niri-сессии, а не как
  обязательный выбор для KDE + nested Niri.

## Что нового в 0.1.214

- откатили небезопасную GNOME/uinput speed-оптимизацию, которая в терминале
  работала, но в браузерных полях могла ломать double Shift replay;
- перед replay снова обязательно синхронизируются GNOME Shell и IBus, чтобы
  браузеры успевали принять фактическую раскладку;
- фиксы `b`, коротких кириллических фрагментов и `on`/`off` из `0.1.213`
  сохранены.

## Что нового в 0.1.213

- standalone `b` больше не автозаменяется в `в`/`и` без фразового контекста;
- короткие кириллические фрагменты больше не улетают в случайный EN-токен,
  например `ыл` не должен превращаться в `sk`;
- технические короткие EN-слова `on`/`off` оставлены разрешёнными для
  автопереворота;
- GNOME/uinput speed-path был добавлен в этом релизе, но в `0.1.214` откатан
  как ненадёжный для браузерных полей.

## Что нового в 0.1.212

- GNOME tray разрезан на отдельные модули: DBus bridge, меню последних
  исправлений и общие helpers больше не лежат одним большим файлом;
- установщик, dev-reload и CI теперь копируют и проверяют все JS-модули
  расширения, а не только старый монолит;
- таблицы русской морфологии вынесены в `data/lexicon`, чтобы runtime-код не
  превращался в склад списков;
- встроенные сценарии `lay-test-input` вынесены в `data/test_input`;
- исправлен баг `protected_words.txt`: пользовательские защищённые ASCII-слова
  больше не перебиваются LEM/LLM-скорингом, например `cd` не должен
  превращаться в `св` после пробела.

## Что нового в 0.1.210

- KDE backend кэширует `qdbus` и список раскладок вместо повторного запроса на
  каждый double Shift;
- короткий KDE replay теперь использует тот же быстрый isolated-output путь,
  что и GNOME;
- проверено live в KDE/Plasma X11 VM.

Ориентировочные live-замеры KDE/Plasma VM:

- KDE: короткое слово `~54-67 ms`;
- KDE: backspace/replay для коротких хвостов `~0-1 ms`;
- основной остаток в KDE теперь тоже переключение layout.

## Что нового в 0.1.209

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

В GNOME и KDE обновление также доступно из меню трея: `Проверить обновления`.

## Возможности

- **Double Shift** исправляет последнее слово в другой раскладке.
- **Replay** физически перепечатывает хвост теми же keycode.
- **Smart** старается не трогать уже нормальные соседние слова.
- **Помощь при наборе** после пробела исправляет только уверенные ошибки.
- **Автоподмена** применяет точные пользовательские правила.
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
- char n-gram и LEM scoring;
- пользовательские точные замены;
- пользовательский список защищённых слов.

Не заявлено как готовое:

- другие пары раскладок;
- полноценная грамматика русского языка;
- исправление целых абзацев;
- серые inline-подсказки прямо внутри поля ввода.

## Ограничения

`lay` сознательно работает с коротким хвостом текста, который daemon видел через
evdev. Это делает основной сценарий быстрым, но задаёт границы:

- не исправляет произвольное слово под курсором после ручного перемещения;
- не меняет выделенный текст как универсальную функцию;
- не читает весь текст поля и не знает весь документ;
- Enter-autocorrect не включён в публичный стабильный UI, потому что evdev/uinput
  не может гарантировать порядок "исправить хвост, потом отправить Enter" во
  всех приложениях;
- IME/preedit и inline-подсказки рассматриваются только как отдельное
  экспериментальное направление, не как замена текущего быстрого пути;
- автопомощь после пробела остаётся консервативной: лучше пропустить сомнительный
  случай, чем самовольно испортить текст.

## Меню в трее

Основные пункты:

- `Помощь при наборе` — осторожная правка после пробела.
- `Автоподмена` — точные пользовательские правила.
- `Режим` — `Replay` или `Smart`.
- `Область` — сколько слов брать для double Shift, по умолчанию `1`.
- `Арбитр` — LEM и auto-layout настройки.
- `ptah_alexs` — жёсткая раскладка по окну.
- `Daemon` — запуск, остановка и статус сервиса.
- `О программе` — версия, ссылка на GitHub и служебная информация.

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

Стабильное ядро: ручной double Shift и локальная RU/EN конвертация.

Активно оттачиваются:

- автопомощь после пробела;
- пробелы и границы слов после автозамены;
- mixed RU/EN сценарии;
- KDE/X11 edge cases;
- работа в старых/особых текстовых полях;
- экспериментальный IME backend.

Если присылаешь bug report, укажи:

- что набрано;
- что ожидалось;
- что получилось;
- GNOME/KDE/Niri/X11 и версия системы;
- включены ли `Помощь при наборе`, `Автоподмена`, `Smart`, LEM.

Приватный текст перед отправкой лучше заменить на безопасный пример.

## Документация

- [Как это работает](HOW_IT_WORKS.md)
- [Проверочный список архитектуры](docs/architecture-checklist-2026-05-19.md)
- [Multi-tap Shift scope](docs/multi-tap-shift-scope.md)
- [Research: Linux input correction best practices](docs/research/linux-input-correction-best-practices-2026-05-17.md)
- [Публичные материалы](docs/publicity/README.md)

## Разработка

```bash
cargo test
cargo build --release
bash install.sh
```

Полная локальная проверка перед публикацией:

```bash
scripts/check-lay-full.sh
```

## English

`lay` is a local Double Shift RU/EN layout rescue tool for Linux desktops.

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
text or the whole document. Enter autocorrect and IME/preedit-style inline
assistance are experimental directions, not the stable default path.

By default `lay` does not use cloud APIs, does not require an LLM, and does not
send typed text anywhere.

## License

MIT
