# Double Shift: задержка и проверка оптимизации — 15 октября 2026

**Состояние:** 1.0.84 установлена и проверена по версиям и загруженным SHA.
Все349 исходныхPASS сохранены. Полная таблица:349PASS51FAIL32BLOCKED/432;
одна прежняяBLOCKED проверка Chrome стала наблюдённымFAIL. Модели и маршруты
доставки не менялись. CLI/трей84, daemon6d0237e6…, IME7e338ae2….
Транзакция: `BDIR/install83-20261015T033053Z/INSTALL.json` (legacy имя каталога).

Нативный predecessor83: daemonba6d9d77…, IMEf633f6aa….
Версия84 меняет только метаданные: все739Rust исходников, зависимости и код
расширения совпадают. Сборка232,239с после canonical architecturePASS;
0локальных функциональных клавиш. Переиспользование доказательств83→84 явно
ограничено этим metadata-only равенством; новый физический прогон84 не заявляется.

Все сборки, проверки и нативный ввод выполняются на `e@192.168.3.94` под штатными resource/Cargo guards. RDP не используется.

## Выбранная минимальная правка

Только `verify_engine_once` заменяет запуск `ibus engine` свежим read-only RPC
`ObservedIbusEngine` через уже существующие соединения daemon → LayImeBridge → IBus.
Каждый вызов читает настоящий `GetGlobalEngine`, без кеша целевой раскладки.
Ошибка, старый IME или занятый getter вызывают прежний CLI ровно один раз.
Валидное несовпадение остаётся отказом и не перепроверяется CLI ради удобного ответа.
Общие `read_engine`/`ensure_engine`, маршруты доставки, модели и обучение не изменены.

Три файла физического вывода побайтно совпадают с принятой 1.0.83: 2 мс между
Backspace, 16 мс после пачки. Точный diff frozen accepted source `run-Qq20tO/source`
и кандидата `run-dWWz50/source` содержит только шесть предусмотренных Rust-файлов.
Решение: [observed-ibus-engine-readback](../architecture/decisions/2026-10-15-observed-ibus-engine-readback.json).

Backend-запрос не отменяется посередине общей D-Bus отправки. RAII/AtomicBool
ограничивает один незавершённый read на процесс IME; busy сохраняет старый CLI.
Существующие 250 мс ограничивают только ожидание session-bus ответа после полной
отправки. Backend и прежний CLI не имеют нового общего deadline; обещаний 150 мс
или полного 250 мс нет. Поздний ответ не кешируется и не становится результатом
следующего запроса. Дескриптор проверяет `IBusEngineDesc`, `a{sv}` и непустое имя.

## Что измерено

Часы нативного ввода: возврат записи evdev → точная поверхность AT-SPI с требуемым
курсорным положением и режимом. Это не время только ядра, физического отпускания
клавиши или отрисовки. Missing сохраняется; провал не превращается в ноль.

| Набор | Измерено | Медиана до → после, мс | p95 до → после, мс |
|---|---:|---:|---:|
| Firefox standard, PLAINV → READV | 420/420 | 107,254 → 98,507 | 118,259 → 111,112 |
| Chrome standard, PLAINV → READV | 406/420 | 92,161 → 83,306 | 112,631 → 100,073 |

Paired median improvement: Firefox **8,235 мс, n=420**; Chrome **9,801 мс,
n=406**. Chrome сохраняет 14 missing в обеих сериях, это не нулевые задержки.
Максимумы: Firefox 133,326 → 120,746 мс; Chrome 124,885 → 106,859 мс.
Все шесть timing-групп и знаменатели 420/48/210/50/10/10 сохранены в compare JSON.
Space ускорение не заявляется: например Firefox37/50 median17,503→18,324 мс;
Tab/selection имеют недостаточное покрытие для вывода о скорости.

Отдельный zero-key API probe30/30 совпавших имён: Firefox getter median0,650 /
p950,757 / first1,274 мс против CLI5,469 /5,757 мс; Chrome getter0,568 /
0,774 / first1,059 против CLI5,289 /6,227 мс. Это проверка API, не пользовательская
задержка. Старые115/p95222 мс относятся к семи смешанным локальным наблюдениям
и напрямую с этими нативными удалёнными сериями не сравниваются.

## Полные результаты поведения

| Версия/серия | Firefox P/F/B | Chrome P/F/B | Всего P/F/B |
|---|---|---|---|
| PLAINV, принятая1.0.83 | 170/27/19 | 179/23/14 | 349/50/33 |
| READV, новая логика readback | 170/27/19 | 179/24/13 | 349/51/32 |

432/432 сценария завершены, NOT_TESTED0. Все216ID на браузер сопоставлены как с
историческим PLAIN, так и со свежим PLAINV; ни один исходный PASS не потерян.
Одна разница: Chrome normal/undo/field2 BLOCKED подготовки → FAIL Space.
Прежде seed был `превеликое` вместо точной опечатки; теперь seed `превет`
получен, но после Space осталось `превет `. Это дополнительный наблюдённый
отказ, не доказательство его причины или исправления. Его нельзя скрывать за
неизменным числом PASS. Все первичные receipts сохранены; повторов до PASS нет.
Обе Chrome privacy проверки PASS: обычный recent_actions изменяется, приватные
посещённые Lay roots остаются неизменными. Полнота всех trace/sink не заявляется.

## Подтверждённые проверки исходника

- 1134/1134 PASS, 0 FAIL, 271,9 с: IME 804, daemon 279, шесть контрактов 51.
- 11 новых typed-wire, реальных RPC, fallback/mismatch и транспортных регрессий.
  Включены backend при poisoned Shared, поздний ответ, busy budget и настоящий
  однобайтовый partial-send, продолжающийся через timeout вызывающего клиента.
- Итоговое независимое ревью кода: **9/10**, два прохода. Первый отклонил отмену
  partial-send и непроверенный тип dict; оба замечания исправлены. Отдельный
  backend disconnect не выделен в самостоятельный тест — зафиксированный пробел.
- Canonical AST/architecture и сборка новой пары прошли. Длительность 183,905 с.

Это выбранный development scope, не all-library/release proof и не приёмка всех
физических приложений. Принятый пользователем Telegram старой версии не переоткрывался.

## Отвергнутые попытки и незавершённые результаты

| Попытка | Firefox P/F/B | Chrome P/F/B | Решение |
|---|---|---|---|
| Принятая чистая база PLAIN | 170/27/19 | 179/23/14 | Контроль, 349/50/33 из 432 |
| Пауза 4 мс | Неполный normal: 19P/1F | NOT_TESTED | Отклонено; потеря burst field6 |
| Пауза 8 мс | 170/27/19 | 178/24/14 | Отклонено; потеря `space_ru_en/field8` |
| READ, первая подготовка | 38P/1B/177NT | NOT_TESTED | Скрытый iframe до seed |
| READ, вторая подготовка | 156P/23F/15B/22NT | NOT_TESTED | Не допускается к установке |

В READ attempt2 lost PASS — `functional/normal/prefix/field1`: обратный текст
точен, но общий предикат режима не прошёл; старый actor не сохранял фактический
mode-readback. В журнале оба жеста обработаны самим IME, без daemon fallback;
путь нового getter в этом эпизоде не исполнялся. Это сужает поиск, но не доказывает
причинную независимость. Последний setup остановился на field9 до поведенческого
ввода, 22 строки остались NOT_TESTED. Все исходные FAIL и сырые receipts сохранены.

Один ограниченный диагностический прогон не повторил сбой: prefix один PASS,
15 preinput iframe setups, 300 SHOWING samples, 16 перезагрузок с новыми native
объектами. Однако diagnostic wrapper завершился NameError `__file__`, поэтому
целого завершённого PASS нет. Это не приёмка и не отмена исходного отказа.

## Согласованная серия и подготовка установки

[Решение о начальной готовности](../architecture/decisions/2026-10-15-matched-native-initial-stack-readiness.json)
фиксирует один PLAINV и один READV по 216 сценариев на каждый браузер. Для всех
используются одинаковые actor hashes и замороженные private data/usage.
Перед первой буквой требуется стабильный FOCUSED+SHOWING и согласие фактических
source, mode и stock CLI engine. Готовность не активирует раскладку повторно.
Записывается результат уже существующего target mode RPC с отдельным временем;
число RPC внутри измеряемого предиката не меняется. Перед матрицей обе стороны
выполняют 60 реальных запросов IBus. Холодное API-чтение записывается отдельно;
холодная нативная задержка не заявляется.

Статическое ревью harness: 8/10 → **9/10**, два прохода. Исправлены stale-result
accept, atomic publication, явный pin functional body и обозначение часов.
Scope validator подтвердил точное сохранение кода вне подготовки, исходных
кейсов/клавиш/часов и всех per-key guards. Ошибка чисто скрытого setup поля остаётся
BLOCKED, но не превращает автоматически остальные независимые строки в NOT_TESTED.

Все216ID сопоставлены с обеими базами; lost_PASS пуст в четырёх сравнениях.
Любая потеря PASS или отсутствие выигрыша запрещает установку; повторов
неизменённой поведенческой проверки до PASS нет. Полный installer84 выполнен: три core бинарника и два файла метаданных расширения.
Metadata-only parity, свежие84 SHA, backup и проверка загруженных /proc байтов
подтверждены; IBus/L1.1/L3/config/source/mode и24прочихфайла сохранены.
Полный журнал и прежние числа: [experiment history](double-shift-latency-2026-10-15-experiment-history.md).

## Пути первичных доказательств

- RUN = `/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c`.
- BDIR = `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6`.
- Source: `/home/ubu/.cache/lay/development/run-3recp7ey/RESULT.json`, remote `run-dWWz50`;
  archive SHA `af0d9b36e84e9612b54708a485c2df608fb0a82afeedd7a6135fd20ccea67d4f`.
- Build: `RUN/PERF-read-build/BUILD.json`; local `BDIR/PERF-read-build.json`.
- Reviews: `BDIR/PERF-readback-review/CODE-REVIEW-2-RESULT.md`,
  `BDIR/PERF-preinput-v2-review2.md`.
- Unfinished native: `RUN/PERF-READ-firefox-incomplete-2`, local mirror same name.
- Completed series: `RUN/PERF-{PLAINV,READV}/{firefox,chrome}`; compares `RUN/PERF-READV/{browser}-vs-{PLAIN,PLAINV}.json`.

Runtime authority remains unchanged; local installed bytes are now verified84.
TD-141 DONE; TD-121 retains its separate legacy functional failures.


Последний wrapper завершился75 при очистке после всех9COMPLETE jobs. SIGTERM
supervisor прервал finally гостя; остался только его private L1.1. UID/PID/tick,
SHA, точная cgroup и test socket сверены; pidfd SIGTERM завершил только этот
процесс. Исходный guard BLOCKED_RESOURCE сохранён и не назван PASS. Collector
после восстановления без нового ввода подтвердил все216 первичных результатов,
отсутствие зажатых клавиш, исчезновение процессов, пустую cgroup и VT1; конечный
seat закрыт. Receipt: RUN/PERF-MATCHED-v2/READV-chrome-closure-recovery.json.
Обнаруженный долг инфраструктуры: systemd249 поддерживает --kill-who, тогда как
текущий guard вызывает --kill-whom; обработчик завершения guest также нуждается
в защите finally от повторного SIGTERM. Guard/runtime ради PASS не менялись.

Решение выпуска84:
[metadata-only release](../architecture/decisions/2026-10-15-observed-readback-release84-metadata-only.json).
Новая нативная проверка84 не заявляется: переиспользуется проверенная логика83
при доказанном равенстве Rust/dependencies/extension logic. Транзакция установки84 завершена; backup старых83 байтов сохранён в receipt.
