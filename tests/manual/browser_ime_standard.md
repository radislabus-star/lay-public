# Стандарт проверки Lay в Firefox и Chrome

Принят пользователем 10 октября 2026 года: один экран с десятью полями,
одинаковый протокол в обоих браузерах и обязательный замер скорости.
Применяется при изменениях runtime ввода и совместимости браузеров.
Простая публикация уже принятой неизменной версии использует её прежние доказательства.

Firefox: [открыть страницу](firefox_double_shift.html).
Chrome: [открыть страницу](chrome_double_shift.html).
Chrome-вход открывает ту же страницу с меткой Chrome; поля, observer и проверки
общие. Имя входного файла не доказывает браузер: JSON сохраняет фактический
userAgent и отдельно выбранную метку. Открывать соответствующий файл в нужном
браузере его штатным способом, не меняя IME-флаги или маршруты доставки.

Базовый набор в каждом браузере сохраняет 60 ячеек:
10 полей × 3 режима окна × 2 режима слова. Окна: обычное, приватное,
полноэкранное. Слово: активное без пробела и после одного Space.
Каждая ячейка получает два отдельных сценария:

1. «Туда и обратно»: `привет → ghbdtn → привет`; после каждой пары —
   физическая клавиша Я/Z и Backspace. Observer требует обе поверхности и
   обе правильные следующие буквы.
2. «5 Double Shift подряд»: пять полных жестов, десять нажатий и отпусканий
   одного Shift. Не менять фокус, не вводить другие клавиши и не ждать
   преобразования между жестами. Требуются все пять поверхностей:
   `ghbdtn → привет → ghbdtn → привет → ghbdtn`. После пятого — Я/Z,
   латинская `z`, затем Backspace. Пробел, если выбран, сохраняется.

Начинать каждый сценарий с RU и указанного слова в собственном пустом поле.
«Следующее поле» меняет фокус последовательно. Выбор режима окна — метка:
оператор действительно открывает приватное окно или включает fullscreen.
Страница сама не определяет приватность. Escape в поле сохраняет отказ.
FAIL/BLOCKED и прежние попытки не удалять и не заменять успешным повтором.
Сохранять JSON для каждого режима, слова и сценария; неподдерживаемые поля
остаются BLOCKED. Адресная строка браузера, пароли, чужие сайты и расширения
не входят в эти десять полей. Все поля помещаются в области от900×600 при100%.

Одного базового слова недостаточно. Общий выбор «Сценарий» содержит 15 планов:

| План | Проверяемое поведение |
| --- | --- |
| Базовый Double Shift | `привет ↔ ghbdtn`, контроль следующей буквы |
| Одна буква | `а ↔ f`, короткое слово не теряется |
| Длинное слово | `параллелепипед ↔ gfhfkktktgbgtl` без обрезания |
| Смешанный регистр | `ПрИвЕт ↔ GhBdTn`, регистр сохраняется |
| Предыдущее слово и запятая | `тест, привет ↔ тест, ghbdtn`; префикс неизменен на каждом шаге |
| Начало с EN | `ghjdthrf → проверка → ghjdthrf`, следующий decoder в обе стороны |
| Space EN → RU | `ghbdtn` + Space → `привет `, ровно один пробел, следующий символ RU |
| Space RU → EN | `дфн` + Space → `lay `, ровно один пробел, следующий символ EN |
| Чистое EN | `max` + Space → `max `, корректное слово и EN decoder сохраняются |
| Опечатка и отмена | `превет` + Space → `привет `; один Double Shift возвращает `превет ` и RU decoder |
| Перенос границы | `должн ыбыть` + Space → `должны быть ` и RU decoder |
| Tab | Подсказка `проверка` после `пров`; Tab → `проверка `, затем оба Double Shift и decoder |
| Курсор | Три ArrowLeft, Backspace → `првет`, И/B → `привет`, End; затем оба Double Shift |
| Выделение | `тест привет`, Ctrl+A, А/F → ровно `а`; затем оба Double Shift |
| Возврат фокуса | Уход в другое тестовое поле, возврат, Ы/S → `приветы`, Backspace, затем оба Double Shift |

В каждом применимом поле проходить фиксированные функциональные планы; явно
записывать выбранный браузер, окно и набор. Дополнительные слова проверять в
обе стороны и на соответствующей изменению границе слова. Серия из пяти
применяется к затронутым ручным переключениям. Не перемножать автоматически
весь словарь, все функции, режимы слова и серии: базовая матрица остаётся полной,
а дополнительный набор выбирается по изменённому механизму и сохраняется в JSON.
Для фиксированных планов «Слово» и «Проверка» отключены: последовательность
Space, Tab или редактирования задаётся самим планом. Начальная раскладка EN/RU
указана в инструкции, для EN-first нельзя автоматически начинать с RU.

Каждый функциональный PASS требует точных промежуточных строк, доверенного
требуемого события и контрольной буквы там, где проверяется decoder. Курсор
и выделение проверяются по доступным caret/anchor; отсутствие этих данных
в URL/email даёт BLOCKED. Совпавшая строка без наблюдённого Space/Tab не
подтверждает действие. При Tab подсказку должен проверить оператор: HTML её
не видит. Если нужной подсказки нет, «Нет подсказки»/Esc до Tab сохраняет
BLOCKED `EXPECTED_COMPLETION_UNAVAILABLE`. После наблюдённого Tab кнопка/Esc
уже сохраняет FAIL действия. Произвольное
окончание не принимается. Отмена восстанавливает исходную опечатку, а не
переключает исправленное слово в EN.
После требуемого события нельзя вручную доводить текст до expected target:
посторонняя клавиша или лишний trigger до результата прекращают план с BLOCKED
`UNEXPECTED_KEY_DURING_ACTION`. Такой текст не даёт latency или PASS действию.

Уход фокуса допускается только на соответствующем шаге focus-return в другое
поле этой страницы; там нельзя печатать. Нужны свежее событие возврата, новая
буква и новый Double Shift. Observer требует доверенное focusin другого
зарегистрированного редактора; один щелчок по toolbar и возврат дают BLOCKED
`OTHER_FIELD_FOCUS_UNOBSERVED`. В остальных планах потеря фокуса прекращает попытку.
Переход в другое окно или скрытие страницы также не даёт успешного возврата.

Скорость сохраняется по каждому жесту и суммарно: количество замеров и
отсутствующих замеров, медиана, p95 (nearest rank), максимум. Медленные и
неудачные сценарии включаются; неизвестная скорость не превращается в0мс.
Страница измеряет `DOM second Shift keydown → первый точный target` по
`performance.now`; отдельно сохраняет подписанную разницу от DOM keyup.
Это браузерная оценка, не время отправки физической клавиши: события IME
могут прийти раньше DOM keyup. В native-сценарии нужен timestamp реальной
отправки второго Shift release, target timestamp и фактическая пауза между
жестами. Условия и coverage указывать рядом с числами.
Space, Tab и замена выделения имеют отдельные группы скорости: требуемый DOM
keydown → точный target с соблюдением условий шага. Их количество, missing,
медиана, p95 и максимум не смешиваются с жестами Double Shift. Планы без
Double Shift не добавляют выдуманные жесты к его знаменателю. Неизвестные
timestamp сохраняются как null; уже измеренные шаги не теряются при отказе.
Target, появившийся только после начала следующей пары, не даёт успешный
быстрый сценарий, даже если итоговое слово правильное. Для последнего target
native-протокол фиксирует отдельную границу (в диагностике10октября500мс).
Новый абсолютный норматив миллисекунд без измерений не вводится.

Для native-результата записать checkout/commit, хеши fixture и установленных
бинарников, PID/starttick, браузер/профиль/режим, реальные клавиши, каждый target,
следующую букву и доступные IME/daemon observations. PASS, FAIL, BLOCKED и
NOT_TESTED имеют отдельные счётчики; потеря непрерывности trace остаётся UNKNOWN.
Источник, DOM, установленный runtime и физическая приёмка — разные границы.
Этот стандарт дополняет исходные проверки C09/C10, включая восемь жестов с
декодером и штатным GNOME значком после каждой пары; их он не заменяет.
HTML не подтверждает RPC count, значок или `InputMode`.

Source-проверки выполняются удалённо существующими guards. Native-ввод — только
в разрешённых новых тестовых окнах, с проверкой фокуса и точных runtime bytes.
При удалённом evdev-вводе перед первой клавишей и каждым новым заданием
подтвердить, что разрешённая owned seat/session активна: PID/starttick и UID
совпадают, `loginctl` показывает `Active=yes`, foreground VT совпадает с её VT.
Один AT-SPI focus или подключённый IME этого не доказывает. После возврата
предыдущей VT для следующего браузера нужна новая проверенная активация тестовой
сессии. Неактивная сессия даёт BLOCKED подготовки; исходные receipts сохраняются
отдельно, ввод останавливается до устранения причины. Такой отказ не является
полной native матрицей или PASS продукта.
Стандарт не разрешает установку, рестарт IBus/Lay, изменение пользовательских
профилей, очистку общего обучения или ввод в существующие пользовательские поля.
Для native-проверок отмены учитывать влияние штатного feedback/обучения:
использовать разрешённую изоляцию тестовых данных; не чистить общую базу.
Фактические результаты и ограничения записываются в
[owning document](../../docs/architecture/firefox-input-stability-2026-10-05.md).


2026-10-14T09:40:16.451011+00:00 — WG Chrome zero-key setup twice rejected before any behavior input, bothclosed exact5Gone/CGempty/VT1. Root cause second capturedFocus payload unchanged (clientgnome-shell, lay_ime_ru/1), only wire serial35→47/time changed. Existing zero setup dictionary equality conflates repeated observational header with identity. Protocol decision2026-10-14-browser-zero-key-focus-identity: compare nonemptyclient+engine path and retain bothheaders; existingfreshIC/globalRU/IMEPIDticksha/focuswindow/kernelpeer/portal rechecks unchanged. Add pure helper+five semantic tests (header-only change allowed; missing/malformed/client/path changes refused), revise005pin only; no sourceRuntime/model/deliveryRoute/binary change. CurrentWGfullFF164P33F19B/216 (5Vgains1lostSpaceRuEn1), all9primary+RAWmanifest copied/hashverified, partialnotfulltable. Original432baselineV339P60F33B staysuntilChrome216. ReviewerNOTRUNthreadlimit/TD121OPEN.


## Native viewport preparation before typing

Before seeding each field, require the exact owned browser window in the foreground. If its native AT-SPI component is not SHOWING, use Component.scroll_to(ANYWHERE), then the existing grab-focus/native-Tab preparation. Wait at most750ms for the original SHOWING predicate before arming input. A still-hidden or wrong-focus field is BLOCKED; no behavior keys are sent there. This applies to standard, functional and C09/C10 actors. Do not add waits after a measured gesture or change its release→exact-target clock, burst cadence, intermediate text, caret, decoder or live field/foreground guards.

Keep interrupted setup receipts and their NOT_TESTED cases. When a new setup-only harness version requires restarting a matrix, pin its actor hashes and keep it separate from the interrupted receipts and binary identity. No core rebuild/retest is implied by viewport preparation alone; changed native acceptance remains required. Owning decision:2026-10-15-browser-fixture-native-viewport-preparation.json.


Для согласованного сравнения скорости подготовка обеих сборок должна совпадать.
Перед первой буквой требуется устойчивое FOCUSED+SHOWING в течение 100 мс,
затем согласие начального GNOME source, InputMode и свежего stock `ibus engine`.
Эта проверка только наблюдает: не реактивирует раскладку и не разрешает ввод в
скрытое поле. Подготовительные ожидания находятся до seed и вне измеряемого
действия; часы и темп пяти жестов сохраняются. Обе стороны получают одинаковое
число подготовительных backend-чтений; first/cold API timing записывается
отдельно и не называется холодной нативной задержкой.

Если точный текст найден, но режим не совпал, сохранять фактический mode и
отдельное время его чтения; timestamp поверхности не доказывает время режима.
Использовать уже имеющийся запрос, не добавлять диагностический RPC внутрь
измеряемого пути. Для скрытого поля, отказавшего до seed, записать BLOCKED и
сохранить оставшиеся независимые сценарии; следующий сценарий проходит все
обычные проверки заново. Потеря фокуса, вмешательство или runtime identity не
получают исключения. Job публикуется атомарно; прежний parent/probe receipt
должен вызывать явный отказ до публикации нового job, а не ложный PASS.

Решение и границы этого сравнения:
[2026-10-15-matched-native-initial-stack-readiness.json](../../docs/architecture/decisions/2026-10-15-matched-native-initial-stack-readiness.json).
Предыдущие FAIL/BLOCKED/NOT_TESTED остаются в журнале и сравниваются по тем же
ID; повторять неизменённые поведенческие отказы до PASS нельзя.
