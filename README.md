<div align="center">

# lay

**Double Shift RU/EN layout rescue для Linux**

`lay` чинит слово, набранное не в той раскладке: нажал **Shift два раза** и
продолжил писать.

**Статус: alpha.** Основной сценарий уже рабочий, автопомощь и разные desktop
edge cases продолжают оттачиваться.

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
- **KDE/X11 backend** есть, но покрытие меньше, чем у GNOME Wayland.

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
- сетевые LLM/API не используются.

Настройки хранятся в:

```text
~/.config/lay/config.json
```

## Поддержка окружений

Основная проверенная среда: Ubuntu/GNOME Wayland с RU/EN раскладками.

Поддерживаемые backend'ы:

- GNOME: Shell extension, tray и DBus bridge для переключения раскладки;
- KDE/Plasma: отдельный `lay-kde-tray` и переключение через `qdbus6`;
- X11: native XKB backend через `x11rb`.

KDE и X11 уже рабочие, но они моложе GNOME-пути. Sway/Hyprland и раскладки
кроме RU/EN пока не заявлены как готовые.

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
- mixed RU/EN сценарии;
- KDE/X11 edge cases;
- работа в старых/особых текстовых полях;
- экспериментальный IME backend.

Если присылаешь bug report, укажи:

- что набрано;
- что ожидалось;
- что получилось;
- GNOME/KDE/X11 и версия системы;
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

Main tested target: GNOME Wayland with RU/EN layouts. KDE/Plasma and X11
backends exist and work, but have a smaller compatibility matrix than GNOME.

By default `lay` does not use cloud APIs, does not require an LLM, and does not
send typed text anywhere.

## License

MIT
