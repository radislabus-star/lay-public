# 1.0.107 — отдельная граница клиента Chrome в KDE

Статус: SOURCE_GATES_PASS_CHROME220_NO_FAIL_OMNIBOX_FAIL. Авторизованная изоляция интеграции. Baseline — принятый
коммит 1.0.106 f88c4ba9, source df48fc2b. Подготовка 107 может идти параллельно
внешней CI 106; стабильные релизы публикуются по порядку и с собственным proof.

## Причина и область

Текущая политика проверки реального процесса Chrome и бюджета активации
находится в kde/display/chrome.rs, хотя не занимается отображением. Это
нарушение границы ответственности: правка оформления легко затрагивает
допуск клиента. Нового функционального FAIL здесь не заявлено: последний
полный Chrome105 имел 180 PASS / 0 FAIL / 40 BLOCKED; 106 прошла семь контролей.

Разрешены пять runtime paths: kde/display/chrome.rs → kde/chrome.rs,
kde/mod.rs, kde/display/mod.rs и два namespace вызова в kde/native_document.rs.
Тела функций, точный process predicate, бюджет 100 мс, ошибки, transport,
общий dispatch/owner/reducer/verifier/executor и GNOME сохраняются.
Допустимы согласованные номера версии, README, решение, manifest/known-failure
hash после ровно одного переноса test namespace, generated graph и evidence.
Никаких новых fallback, детектора среды, прав на изменение или владельцев.

## Проверка

Перенести существующий код, не переписывать его. Одно имя теста меняет
namespace display::chrome → chrome; тело, категория и отказные условия прежние.
Другие identities сохраняются. Выполнить full correctness/package, 18 release
gates и сборку 10 бинарников; сверить установленные байты на KDE. Полный
неизменённый Chrome220 и короткие Firefox-контроли; задержки, промежуточные
Double Shift и все BLOCKED сохранить. GNOME не устанавливать и не тестировать
в локальной пользовательской сессии.

## Результат

PENDING. Это разделение кода, а не обещание нового пути доставки или
устранения неподтверждённых ограничений браузера.

## C05 functional follow-up — preflight before code

Первый полный native107 завершён: 178 PASS / 1 FAIL / 41 BLOCKED / 0 NOT_TESTED.
Журнал: CACHE/release107-chrome-row17-v1.jsonl. В normal/search Space EN→RU
готовое решение получено за645мкс. После шестой буквы native показывает5;
resample не вызван, strict identity/edit gate правильно отказывает.

Карта: callback settlement меняет word token → прежний native binding хранит
старый token → read-only resample проверяет строгую edit preparation → читает
только один раз → актуальный edit source не получен → отказ identity_mismatch.
Пока это причинная гипотеза, которую должен подтвердить RED с настоящими
WordScope/reducer операциями, затем GREEN и физический результат.

Расширение области107: только kde/native_document/preparation.rs и tests.rs,
manifest/known_failures для двух новых semantic tests и это решение/документы.
Остальная механическая изоляция Chrome сохраняется. Общие committed_tail.rs,
managed.rs, preedit.rs, C03 reducer, C07 action/verifier/executor и GNOME
не редактируются. Нет нового пути доставки, таймера, очереди или ретрая.

Рекомендуемый вариант9/10: существующее повторное чтение проверяет текущий
admission token, прежние field/focus/lease/layout координаты и текущий
managed-append witness. Каждый реальный read сохраняет все native guards.
Строгий native_word_preparation_identity для редактирования не меняется.
Максимум2 чтения с тем же Instant в исходном бюджете80мс, без sleeps.

Альтернативы: общий отложенный single-word Space протокол6/10 (более широкий
C07 контракт); увеличить ожидание2/10 (не исправляет false eligibility);
пропустить source/identity guard1/10 (нарушает канон, отвергнуто).

Первый failed107 source и native receipts сохраняются. Не считать namespace
перенос доказанной причиной: тела кода эквивалентны; проверяется выявленная
гонка наблюдения и завершения callback. Firefox/omnibox ещё не запускались: native
phase остановилась на честном Chrome FAIL. Профиль обычного Chrome имел stale
lock; его живой запуск подготовить под guard перед omnibox, без сброса профиля.

Первый unit RED: 83/85. Один новый тест уже ловит false resample permission после settlement; другой остановился на неподходящей zero-character fixture до целевого действия. Это не считается полным causal RED. Fixture уточнена до реально наблюдённого класса: существующий owned prefix длиной1 или5 и следующий собственный CommitText. Отсутствие witness отдельно остаётся отказной проверкой. Runtime код ещё не менялся.

Causal RED V2 подтверждён: 83/85, оба новых теста выявляют пропуск повторного чтения после production WordScope settlement. Reader вызван1раз вместо2; native edit permission по-прежнему false. Все83 прежних проверки проходят. Runtime код пока исходный. Следующий шаг: изменить только read eligibility в KDE preparation, затем GREEN того же набора.

GREEN V3:85/85 PASS, включая оба новых теста. Current-token read fence позволяет второе реальное чтение; строгая edit preparation остаётся false до установки полного actual snapshot. После actual install исходный InputFrameIdentity полностью совпадает с подготовленным до settlement. Scope: production WordScope operation + test reducer publisher + controlled transport, не физическая приёмка. Raw isolated-release107-key-scope-unit-v3-green/result.json. Runtime изменение ровно в одном read predicate KDE, границы AST проверены; старый edit predicate и все общие runtime файлы неизменны.

## Замороженная итоговая сборка107 V3

Source SHA-256 `708926cec7f4600a831580464ef75679242eb359b9d72c20a2480b058e3f2e7c`,1290 файлов. Полная проверка correctness+package3550/3550 PASS (3514+36),1113 IME;18 release gates PASS,11 performance,10 устанавливаемых бинарников,0 compiler warnings,MSRV1.88.0 и lexical all-targets PASS. Raw source: `/home/e/projects/lay-development-runner/run-Baprff/RESULT.json`; gates: `isolated-release107-gates-v3/result.json`.

Точные артефакты установлены на удалённой KDE: engine PID2211235, SHA-256 `18a45eb56c73f10ea4b8402210f55336078f214a6a9d8cacb5d3042d32be1b84`. Deployment: `isolated-release107-deployment-v3/result.json`. Локальный GNOME не устанавливался и не перезапускался.

Пять заранее заданных повторов прежнего native FAIL прошли5/5. Полный Chrome220, настоящая верхняя адресная строка и свежий Firefox baseline ещё выполняются; source PASS не является их приёмкой. В startup V3–V5 функциональный ввод не начинался: сохранены отдельные setup receipts. V6 исправляет только matcher X11 window/PID в диагностическом запуске; actor/assertions/cadence неизменны.

## Итоговая native107 V3

Неизменённый Chrome220:179 PASS /0 FAIL /41 BLOCKED /0 NOT_TESTED. Исходные160:126/0/34/0, дополнительные60:53/0/7/0. Каждый из пяти отдельных повторов Search/Space проходит. Raw: `isolated-release107-full-native-v5/result.json`, `isolated-release107-native-phase-v6/stress.json`.

Отдельная настоящая верхняя адресная строка:0 PASS /1 FAIL /2 NOT_TESTED. У первого слова нет свежей собственной подсказки; наблюдение возвращает `native_selection_present`, символы имеют нулевую геометрию. Config/engine/focus-owned-window/held-key restoration PASS. Этот прежний независимый класс уже записан в `docs/development/chrome-omnibox-release-proposal-2026-10-24.md`;107 его не исправляет и не получает за него PASS. Raw: `isolated-release107-omnibox-v6/result.json`. Нет ослабления selection/geometry verifier и нет повторов до зелёного.

Полный Firefox baseline запущен отдельно для108; он не является приёмкой107. Общие runtime и GNOME исходники сохранены. Публикуется конкретное исправление Search/Space и изоляция process policy, с явным ограничением адресной строки.

Stable `v1.0.107` опубликована 2026-10-28T06:00:18Z на `e178be59563fdce592921402ce9e6634e9b2e9c7`,4/4CI SUCCESS. Updater --check: local104→107, remote107current, no apply/local restart. Publication does not close the explicitly retained omnibox FAIL or BLOCKED rows. URL: https://github.com/radislabus-star/lay-public/releases/tag/v1.0.107.
