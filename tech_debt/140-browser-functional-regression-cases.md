# TD-140 — Расширить общий стенд Firefox/Chrome до функциональных сценариев

Status: DONE_FOCUSED_SOURCE. Owner: shared browser fixture; no runtime change.

## Причина

Пользователь указал, что одного `привет → ghbdtn → привет` недостаточно.
В `createRun` все expected surfaces заданы для одной пары слов. Даже исправный
observer не обнаруживает поломку Space, Tab, отмены, редактирования и фокуса.
Первый дефект — отсутствие таких входов в тестовом плане, не новый доказанный
дефект runtime. База: commit 92965082, установленный IME не изменяется.

## Выбранный минимальный вариант

Готовые планы в одном небольшом файле данных плюс существующий bounded observer.
Firefox и Chrome продолжают использовать одинаковые десять полей.
Оценки: только новые слова 6/10; общий набор функциональных планов 9/10;
новый native framework 4/10. Решение — второй вариант.

Добавить короткий/длинный текст, регистр, сохранение предыдущих слов и пунктуации,
начало с EN, обе стороны Space, чистое EN, исправление и отмену опечатки,
перенос границы `должн ыбыть`, Tab и переключение принятого слова, изменение
курсора с Backspace/повторным вводом, замену выделения, уход и возврат фокуса.
Каждый план задаёт точные строки и требуемые реальные события. Данные плана
не конвертируют, не печатают и не исправляют пользовательский текст.

## Критерии завершения source-задачи

- Старые 25 регрессий сохраняются. Новый EN-first тест RED на старом observer.
- Каждый новый план имеет positive semantic proof и negative tests общих
  механизмов: ключ не наблюдён, ошибочный decoder, неверный курсор/выделение,
  утрата фокуса и появление устаревшего текста. Zero selection/skip запрещены.
- Скорость Double Shift и Space/Tab измеряется отдельно с coverage и missing.
- Все десять редакторов остаются видны при 900×600. Общий Chrome-вход сохранён.
- Удалённые resource/canon checks, обязательный source-bound graph, независимое
  review максимум в два прохода; commit/push после завершения source-проверки.

## Границы и подводные камни

- Нельзя менять маршруты IME, обучение, модели, owner или safety/verifier.
- URL/email могут не предоставлять caret/selection: это BLOCKED, не нулевой
  курсор. Событие Space/Tab, поглощённое IME, не подменять выдуманным DOM key.
- Focus-return разрешает ожидаемый уход только в своём плане; обычный тест
  по-прежнему прекращается при потере фокуса. Возврат требует свежего события.
- Tab фиксирует выбранное expected completion; отсутствие подсказки — отдельный
  отказ/precondition, не разрешение принять произвольное окончание.
- Восстановление исходной опечатки после исправления отличается от проекции
  правильного слова в EN. Одного конечного совпадения для успеха недостаточно.
- Новые source PASS не отменяют прежние 54 native FAIL и 4 BLOCKED. Native
  приёмка новых сценариев остаётся отдельной открытой границей TD-121.

## План и TDD evidence

Plan review: независимый агент без контекста, 9/10. Учтены три замечания:
после возврата фокуса нужны новая буква и новый жест; скорость действий имеет
отдельные знаменатели; базовая матрица сохраняется, дополнительные планы
не перемножаются с неприменимыми контролами.

Controlled RED: 26 Node tests selected, 25 PASS / 1 FAIL / 0 SKIP. Новый
EN-first test обнаруживает старый `type_ru` вместо `toggle_ru`. Receipt:
`/home/ubu/.cache/lay/development/browser-functional-red-20261010/RESULT.json`.
Реализованы 6 manual + 9 functional profiles. Remote source GREEN:
Final 49/49 Node, 4/4 fixture Python, 2/2 compat, 15/15 canon; JS/bash/canon PASS.
240 corpus + 90 functional executions — модели observer под десятью field IDs,
а не ввод в реальные браузеры. Remote 900×600 render: видны все десять полей,
keys sent = 0. Source receipt:
`/home/ubu/.cache/lay/development/browser-functional-source-final49-20261010/RESULT.json`.
[Compact evidence](evidence/browser-functional-regression-cases-2026-10-10.json)
содержит хеши и раздельные границы.

Implementation review pass 1: 7/10. Исправлены три материальных замечания:
ручное доведение текста после Space/Tab больше не может дать PASS; нужен
trusted focusin другого зарегистрированного поля перед возвратом; отсутствие
completion до Tab получает BLOCKED, после выполненного Tab отказ остаётся FAIL.
Добавлены три отрицательные регрессии; финальные 49 checks получили GREEN.
Результат до repair: 46/46 SOURCE PASS, historical receipt
`/home/ubu/.cache/lay/development/browser-functional-source-pass1-20261010/RESULT.json`.

Final implementation review pass 2: **9/10**, all material findings closed;
две проверки, третьей нет. [Review record](evidence/browser-functional-review-2026-10-10.md).
Обязательный publication graph receipt:
`/home/ubu/.cache/lay/development/browser-functional-graph-20261010/RESULT.json`.
Граница DONE: общий стенд и source-контракты. Новые native профили NOT_TESTED;
TD-121 и прежние отказы не закрыты этой задачей. Публикация не меняет runtime.

Owning document: [Firefox input stability](../docs/architecture/firefox-input-stability-2026-10-05.md).
