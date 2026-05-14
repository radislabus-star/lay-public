# Short posts

## Russian Telegram / Linux chat

`lay` — маленький open-source помощник раскладки для Linux desktops.
Главный сценарий: лёгкое double Shift исправление слов в RU/EN раскладке.

Сценарий простой: набрал слово не в той RU/EN раскладке, нажал Shift два раза,
слово перепечаталось в другой раскладке.

```text
ghbdtn      -> привет
good ntrcn -> good текст
wi-fi ye   -> wi-fi ну
```

Работает локально: Rust daemon + evdev/uinput + desktop backend. GNOME
использует Shell extension, KDE Plasma — отдельный tray и `qdbus6`, X11 —
native XKB через `x11rb`. Обычный double Shift не использует облако, LLM или
буфер обмена.

Пока это beta под RU/EN. GNOME проверен лучше всего, KDE Plasma и X11 уже
работают, но матрица чужих систем меньше. Особенно нужны короткие
воспроизводимые баги от тех, кто много печатает между русским и английским.

GitHub:
https://github.com/radislabus-star/lay-public

Хабр:
https://habr.com/ru/news/1033522/

## Published links

- Хабр: https://habr.com/ru/news/1033522/
- Linux.org.ru: https://www.linux.org.ru/news/opensource/18288596
- OpenNET: отправлено на модерацию, публичная ссылка появится после проверки.

## Habr intro teaser

`lay` — маленькая open-source утилита для Linux desktops: лёгкая замена
Caramba/Punto-сценария для RU/EN набора.

Главная идея простая: нажимаешь Shift два раза, и слово, набранное не в той
раскладке, перепечатывается правильно.

Самое интересное оказалось не в `ghbdtn -> привет`, а в пограничных случаях:
`good ntrcn`, `AmoCRM Z`, `wi-fi ye`, частичные слова, автопомощь после пробела
и отказ от агрессивной LLM-магии.

Ниже технический разбор архитектуры, ошибок и решений без попытки делать из
этого большой продукт.

## OpenNET / Linux.org.ru style

Выложен в open source `lay` — лёгкий помощник раскладки для Linux desktops.

Что делает: если слово набрано не в той RU/EN раскладке, нажимаешь Shift два
раза, и оно перепечатывается в другой раскладке. Основной путь локальный, без
облака, буфера обмена и LLM.

Примеры:

```text
ghbdtn      -> привет
good ntrcn -> good текст
wi-fi ye   -> wi-fi ну
```

Проект beta: GNOME проверен лучше всего, KDE Plasma и X11 уже поддержаны с
меньшей матрицей покрытия. Нужны короткие воспроизводимые баги.

GitHub:
https://github.com/radislabus-star/lay-public

Хабр:
https://habr.com/ru/news/1033522/
