# 1.0.106 — граница подготовки KDE

Статус: STABLE_PUBLISHED_UPDATER_VERIFIED. Авторизованная задача интеграции,
механический перенос. Baseline4ebd03f3 / accepted105 source85cf6f9f.

105 уже заморожена в коммите. Подготовка и source checks106 могут идти
параллельно внешнему105CI; установка и публикация106 ждут105 stable+updater.
Это не повторная проверка105 и не изменение принятого runtime.

## Карта и разрешённый diff

- `window_interaction/observation.rs`: убрать три KDE entrypoint; сохранить
  общий ManagedWordStartWitness, его приватные поля и конструкторы. Сделать
  два существующих конструктора crate-visible без изменения тел. Выделить
  чистую immutable-проекцию представления разделителя у прежнего witness.
- `kde/native_document/preparation.rs`: единственное размещение прежних
  arm_native_observed_word_boundary, arm_native_observed_first_append_boundary
  и acknowledge_managed_word_separator_presentation. Проверки и порядок прежние.
- Дополнительно: согласованные номера106, этот owning document, решение,
  README KDE/GNOME, roadmap/handoff, доказательства и generated graph.

Защищены GNOME, dispatch, rendering, layout, обычный Shift, ручной toggle,
решение коррекции, admission/reducer/verifier/исполнитель, модели, конфигурация
и бюджеты. Нет второго владельца, fallback или копии доставки.

## Проверка эквивалентности

Две arm-функции перенесены с теми же телами (квалифицирован только WordInputMode).
Separator-проекция сохраняет every identity/revision/expiry через clone; общая
tail-операция остаётся прежней и выполняется до установки нового witness.
При любом отказе witness не изменяется. Оба старых snapshot-конструктора
сохраняют собственную проверку свежести, вместо её дублирования в KDE.

Все прежние test identities/classifications должны сохраниться. Проверки:
канонические correctness/package, release gates и десять release-бинарников;
старые native/Space негативные контракты; семь фиксированных nativeChrome
контролей. Полную старую Chrome220 не объявлять свежей проверкой106.
Локальный GNOME не трогать; физический GNOME остаётся NOT_TESTED.

## Результаты

Source3548/3548PASS, включая1111IME; прежние test identities и классификации сохранены. Все18releasegatesPASS,10releasebinaryartifact,0compilerwarnings; MSRV1.88 и tiny offline byte parityPASS.

Remote runtime106: engine1691255, daemon1691376, tray1691382. SHA engine3c8d65db3667570246a4fe4736a2a0f2bae62f87e82f811bfec777f9b1dd5c21.
Семь nativeконтролейPASS (5iframe+rich+plain), конфигурация восстановлена; исходныйIBus, desktop и модели не менялись. НативнаяGNOME/Wayland/USB и полнаяChrome220 для106 не проверены. Историческая105Chrome180PASS0FAIL40BLOCKED относится к105.

Первыйcanon-gateV1 отказал из-за пропущенных мной обязательных полейADR. Поля дополнены; V2canonPASS. Исходники и замороженный full-testrequest не менялись; обновление ровно одного документа записано отдельным overlay. Старый отказ сохранён.

Доказательства: tech_debt/evidence/release-1.0.106-verification.json. CI 4/4 SUCCESS. Stable v1.0.106 опубликована 2026-10-28T03:26:14Z; updater проверен локально (104 → 106) и удалённо (106 актуальна), без установки на локальном GNOME.

Space latency shortcontrols: {"n": 7, "median_ms": 86.09391702339053, "p95_ms": 108.57352009043097, "max_ms": 108.57352009043097, "scope": "RFB Space release to stable corrected native field; n7 short controls, no speedup claim"}

Релиз: https://github.com/radislabus-star/lay-public/releases/tag/v1.0.106
