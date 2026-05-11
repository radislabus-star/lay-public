# Как это работает

## Задача

На Windows есть Caramba/Punto Switcher: случайно набрал текст в неправильной
раскладке (`Ye djn ghbvth` вместо `Ну вот пример`), нажимаешь горячую клавишу —
и слово автоматически конвертируется. На Linux нативного аналога нет
(xneur умер в 2017, Wayland не поддерживается).

`lay` решает эту задачу тремя компонентами:

| Компонент            | Что делает                                      |
|----------------------|-------------------------------------------------|
| **`lay`**            | CLI: конвертирует текст из аргумента/stdin/clip |
| **`lay-daemon`**     | Фоновый демон: double Shift в приложениях       |
| **GNOME extension**  | Tray menu, DBus bridge, активация раскладки     |

---

## Shared Core

Код разделяется на общее ядро и desktop-интеграции.

Общее ядро:

- `src/core.rs` — стабильный facade для будущих frontend-ов;
- `src/config.rs` — единая схема настроек для daemon, GNOME tray и будущего KDE tray;
- `src/correction.rs` — общий контракт результата исправления;
- `src/desktop.rs` — выбор `gnome` / `kde` / `x11`, нормализация layout-id;
- `src/dict.rs` — физическая RU/EN конвертация клавиш;
- `src/keyboard.rs` — keycode-события, word split, replay-decision, US/RU mapping и text→uinput runs;
- `src/word_buffer.rs` — история текущего/предыдущих слов, replay-toggle и pending feedback;
- `src/lem.rs` и `src/ngram.rs` — scoring готовых кандидатов;
- `src/quality.rs` — лёгкие эвристики качества текста;
- `src/text_edit.rs` — минимальный план замены текста без лишней перепечатки.

Desktop-адаптеры:

- GNOME: Shell extension, tray, DBus bridge, `inputSources[i].activate()`;
- KDE/Plasma: должен быть отдельный adapter поверх того же ядра;
- X11: отдельный backend через X11 tools, где они доступны.

Цель такого разделения: не смешивать GNOME Shell API с логикой исправления
текста. Ядро должно быть переиспользуемым, а GNOME/KDE должны отличаться только
слоем интеграции с окружением рабочего стола.

---

## lay-daemon — как работает двойной Shift

### Общий поток

```
Физическая клавиатура
        │  evdev (/dev/input/event*)
        ▼
  lay-daemon (Rust)
  ┌──────────────────────────────────────┐
  │                                      │
  │  WordBuffer                          │
  │  Накапливает (keycode, shift, layout)│
  │  для текущего слова и короткой       │
  │  истории завершённых слов.           │
  │  Space переносит current → history.  │
  │  Полный reset: Enter/Tab/Esc/←→/BS.  │
  │                                      │
  │  FSM двойного Shift                  │
  │  Idle → FirstPress                   │
  │       → WaitingSecond                │
  │       → SecondPress                  │
  │       → DOUBLE SHIFT!                │
  │                                      │
  └──────────────┬───────────────────────┘
                 │ DOUBLE!
                 ▼
  ┌──────────────────────────────────────┐
  │  handle_double_shift                 │
  │                                      │
  │  1. uinput Backspace × N             │
  │     (стереть слово с экрана)         │
  │                                      │
  │  2. DBus → GNOME Extension           │
  │     ActivateLayout('ru'/'us')        │
  │     SetGlobalEngine (ibus tray)      │
  │                                      │
  │  3. uinput replay или minimal insert │
  │     (те же keycodes → новая          │
  │      раскладка → другие символы)     │
  └──────────────────────────────────────┘
```

### Почему evdev, а не X11/Wayland input grabbing

На **Wayland** приложения полностью изолированы друг от друга по вводу.
Нет глобального `XGrabKey`, нет `GetKeyboardState`. Единственный
способ читать все нажатия — `/dev/input/event*` (evdev), напрямую
из ядра, до любых Wayland-протоколов.

Требует членства в группе `input`:
```bash
sudo usermod -aG input $USER
```

### Почему uinput, а не wtype/xdotool

`wtype` и `xdotool` работают через протоколы Wayland/X11 и не принимаются
sandbox-изолированными приложениями (Flatpak, snap, GNOME Terminal и др.).

`uinput` — виртуальное устройство ввода в ядре Linux. С точки зрения
системы это **физическая клавиатура**. Получают события все приложения,
включая sandbox-изолированные.

### FSM детектора двойного Shift

Простого «два release подряд» недостаточно: если держать Shift для
заглавной буквы, а потом отпустить — это тоже release. Нужен полный цикл
**press → release → press → release**:

```
Idle
 │ LShift press
 ▼
FirstPress { pressed_at }
 │ release, удержан ≤ 200ms (тап)      release > 200ms → Idle
 ▼                                      (держали = заглавная буква)
WaitingSecond { first_release }
 │ LShift press, < 500ms от release    press > 500ms → FirstPress (новый цикл)
 ▼
SecondPress { second_press }
 │ release, удержан ≤ 200ms (тап)      release > 200ms → Idle
 ▼
DOUBLE SHIFT ✓
```

Любая **другая клавиша** в состояниях FirstPress/WaitingSecond → Idle (отмена).
В состоянии SecondPress — игнорируется (второй Shift уже нажат, ждём release).

### Layout backend

Переключение раскладки вынесено в backend-слой:

```json
"layout_backend": "auto"
```

Поддержанные значения:

- `auto` — выбрать backend по окружению;
- `gnome` — GNOME Shell extension + DBus;
- `kde` — `qdbus/qdbus6 org.kde.keyboard /Layouts setLayout`;
- `x11` — `xkb-switch`, `xkblayout-state` или fallback через `setxkbmap`.

GNOME Wayland остаётся основной проверенной средой. KDE/X11 backend пока
экспериментальные и добавлены как отдельный слой, чтобы ядро replay/smart/typing
assist больше не было жёстко привязано к GNOME.

### ptah_alexs — жёсткая раскладка по окну

`ptah_alexs` реализован в GNOME extension, потому что именно GNOME Shell видит
активное окно и его app id / wm class. Это не “память последней раскладки”,
а policy-режим:

```json
{
  "ptah_alexs_mode": true,
  "ptah_alexs_rules": [
    {"kind": "app_id", "value": "org.gnome.Terminal.desktop", "layout": "us", "label": "Terminal"}
  ]
}
```

При смене фокуса extension получает `notify::focus-window`, находит правило для
текущего окна и вызывает `inputSources[i].activate()` для `us` или `ru`. Правило
`layout = "keep"` означает “это окно не трогать”.

Правила добавляются из трея для текущего активного окна. Заголовок окна не
используется как ключ по умолчанию, чтобы не сохранять приватный текст из title.

### Почему GNOME Extension всё ещё нужен на GNOME Wayland

На Wayland нет общего API для переключения раскладки извне. `gsettings` меняет
настройки, но не применяет их к текущей сессии ([Bug #1956916](https://gitlab.gnome.org/GNOME/gnome-shell/-/issues/1956916)).
Виртуальная эмуляция `Alt+Shift` не регистрируется Mutter как акселератор.

Единственный работающий способ — вызвать `inputSources[i].activate()`
**внутри GNOME Shell** (GJS). Поэтому есть extension, который экспортирует
этот вызов через DBus. В горячем пути daemon использует постоянное `zbus`
соединение, а внешний `gdbus` остаётся только fallback/диагностикой:

```
lay-daemon → zbus session call → org.gnome.Shell
                                → extension.js / lay-impl.js
                                   → inputSources[i].activate()  ✓
```

---

## lay CLI — простая и smart-логика

### Обычный режим — детерминированная конвертация

Каждой клавише QWERTY соответствует ровно одна клавиша ЙЦУКЕН — **биекция**:

```
q↔й  w↔ц  e↔у  r↔к  t↔е  y↔н  u↔г  i↔ш  o↔щ  p↔з
a↔ф  s↔ы  d↔в  f↔а  g↔п  h↔р  j↔о  k↔л  l↔д
z↔я  x↔ч  c↔с  v↔м  b↔и  n↔т  m↔ь
```

`HashMap<char, char>` + `OnceLock`. Конвертация — микросекунды.
Обычный CLI не пытается оценивать качество и не зовёт модель:

```bash
lay "ghbdtn"      # привет
lay --no-llm ...  # legacy-safe: тоже только dict
```

`--threshold` оставлен только как legacy option для совместимости старых
командных строк; текущий простой CLI его не использует.

### Smart режим — optional arbiter

Модель не является основным путём работы. По умолчанию публичная сборка не
загружает LLM и не компилирует direct GGUF backend.

Простой режим LLM не вызывает. LLM включается только явно:
- в daemon: через `Ещё → LLM` в tray menu;
- в CLI: только через флаг `--smart` и выбранный `LAY_LLM_BACKEND`.

`lay-daemon` вызывает `lay::llm::warm_up()` только если стартует с
`correction_engine = "smart"`. Если `LAY_LLM_BACKEND` не задан, backend равен
`off`, поэтому реальная модель не грузится. Обычный CLI при этом не начинает
использовать модель сам по себе.

Поддерживаемые экспериментальные backend'ы:

```bash
LAY_LLM_BACKEND=ollama lay --smart "fyukbqcrbq"
cargo build --release --features direct-llm
LAY_LLM_BACKEND=direct LAY_GGUF_MODEL=/path/to/model.gguf lay --smart "fyukbqcrbq"
```

Объём исправления (`Ещё → 1 слово` / `Ещё → 2 слова`) независим от engine.
Engine выбирается отдельно:

- `correction_engine = "replay"`: физический replay выбранного scope;
- `correction_engine = "smart"`: scoped-tail/tokenwise решение.

Smart mode умеет оставить нормальное первое слово и исправить только плохой
хвост:

```text
Главное Вщгиду -> Главное Double
good ntrcn     -> good текст
wi-fi ye       -> wi-fi ну
```

Перед моделью есть быстрый deterministic repair для смешанного текста: если в
русском слове остались латинские клавиши, они домаппятся в RU, а
ASCII-акронимы вроде `LLM` сохраняются. Для чистой обычной перекладки
используется replay-путь, чтобы второй double-Shift мог вернуть текст обратно.

Модель не генерирует исправленный текст с нуля. Код сначала строит
детерминированные кандидаты (`оригинал` и `US↔RU`), а модель при включённом
backend выбирает один из них коротким ответом `A`/`B`. Если backend выключен
или ответ не распознан, используется deterministic fallback.

### Учебный лог

Learning log включается только опцией `learning_log`. По умолчанию в config он
выключен.

После успешного ручного double-Shift daemon может добавить строку JSONL в
`~/.local/share/lay/corrections.jsonl`. Пишутся только явные исправления:
`from`, `to`, `kind`, `replace_words`, `words`, `ts`. Обратный replay-toggle не
логируется, чтобы не добавлять пары нормального текста в мусорную раскладку.

Для auto/smart исправлений используется другой слой: daemon запоминает
результат как pending feedback и ждёт, исправит ли его пользователь. Если
пользователь удалил результат и набрал свой вариант, пишется
`kind = "user-correction"` с дополнительными полями `lay_kind`, `lay_from`,
`lay_to`. Повторные безопасные user-correction могут быть повышены в точное
правило `~/.config/lay/replacements.json`.

Лог ограничен: при размере больше 1 MB файл подрезается до последних 3000 строк.

### Автоподмена простого режима

Если в config включено `auto_replace`, replay-путь после обычной перекладки
проверяет результат по точным правилам автоподмены. Это не LLM и не fuzzy-поиск:
правило срабатывает только при точном совпадении слова/фразы. Встроенные правила
дополняются пользовательским JSON-словарём `~/.config/lay/replacements.json`.

### Автопереключение раскладки

Опция `auto_switch_layout` относится к автоматической помощи при наборе после
пробела. Когда helper видит уверенное слово, набранное в неправильной
раскладке, он заменяет его и может оставить активной раскладку исправленного
текста. Если опция выключена, layout-автоправки не применяются, а после обычной
автоматической правки daemon возвращает раскладку, которая была активна до
исправления.

Ручной double Shift не зависит от этой опции: это явная команда пользователя
переключить выбранный хвост, поэтому раскладка переключается всегда.

### DBus и модель доверия

GNOME extension экспортирует session-local DBus bridge для `lay-daemon`.
Публичными оставлены только методы, которые нужны runtime-пути: `ActivateLayout`,
`CurrentLayout`, `NextLayout`, `ListLayouts`, `TypeText` и `Ping`.

Методы прямого удаления текста (`Backspace`, `ReplaceLastN`) не экспортируются:
обычная очистка слова делается через uinput внутри daemon. `TypeText` остаётся,
потому что он нужен для typing assist и fallback-вставки, если GNOME не подтвердил
переключение раскладки после удаления слова.

DBus внутри user session не является границей безопасности от процессов того же
пользователя. Если пользователь не доверяет локальным процессам в своей сессии,
extension и daemon нужно выключить.

Если переключение раскладки не подтвердилось, daemon не делает blind replay в
старой раскладке. Он пытается вернуть ожидаемый текст через `TypeText`, не
обновляет cache активной раскладки и пишет диагностическое событие только в
debug-log, если он явно включён.

---

## Архитектура кода

```
lay/
├── src/
│   ├── main.rs          — CLI (clap), dict conversion и --smart
│   ├── dict.rs          — словарь US↔RU, detect_direction, convert
│   ├── quality.rs       — legacy/auxiliary quality heuristics
│   ├── ngram.rs         — char 3-gram scorer для typing assist
│   ├── llm.rs           — optional model arbiter вокруг готовых кандидатов
│   └── lem.rs           — lightweight scorer/ranker для готовых вариантов
│
├── src/bin/
│   ├── lay_daemon.rs        — evdev listener, FSM, layout backend, uinput replay
│   ├── lay_ngram_corpus.rs  — build/check/cache локального n-gram корпуса
│   └── lay_lem_research.rs  — локальный stress-test LEM scorer
│
└── extension/
    └── lay@radislabus-star.github.io/
        ├── extension.js — loader
        ├── lay-impl.js  — DBus service + tray indicator
        └── metadata.json
```

## Зависимости CLI

| Crate                | Зачем                                           |
|----------------------|-------------------------------------------------|
| `clap`               | CLI парсинг                                     |
| `arboard`            | Clipboard (Wayland + X11)                      |
| `ureq`               | optional HTTP backend для Ollama               |
| `llama_cpp`          | optional direct GGUF backend (`direct-llm`)    |
| `serde`/`serde_json` | Config, cache и JSON-запросы                   |
| `zbus`               | DBus-клиент daemon → GNOME Shell extension     |
| `evdev`              | Чтение физической клавиатуры и `evdev::uinput` |

## Профиль release

```toml
[profile.release]
opt-level = 3
lto = true           # link-time optimization
codegen-units = 1    # медленнее компиляция, быстрее бинарь
strip = true         # без отладочных символов
panic = "abort"      # без unwinding
```

Текущие release-бинарники на проверенной машине:

- `lay` — около 2.6 MB;
- `lay-daemon` — около 3.8 MB.

## Что не умеет (пока)

- **Дополнительные раскладки** (UK, DE, FR) — добавить таблицы в `dict.rs`
- **Автоопределение без двойного Shift** — статистический анализ потока
