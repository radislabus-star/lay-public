# TD-141 — Уменьшить задержку Double Shift без потери исходных PASS

Status: DONE. Priority: P1. User-approved performance follow-up.
Owner: existing daemon exact replay verifier + existing LayImeBridge backend connection; C03–C10.

## Текущее решение

Убрать запуск процесса `ibus engine` только из независимой проверки фактического движка `verify_engine_once`. Новый read-only RPC `ObservedIbusEngine` на существующем объекте LayImeBridge каждый раз вызывает штатный IBus `GetGlobalEngine` через уже принадлежащее ему подключение. Демон использует своё существующее подключение к LayIme. Нового владельца, соединения, кеша раскладки, зависимости, FFI, маршрута доставки или обработки текста нет.

Общие `read_engine`, `read_current_layout_is_ru`, `ensure_engine` сохраняют прежний CLI. Ошибка/старый IME/занятый новый запрос вызывает ровно одно прежнее CLI-чтение. Валидное несовпадение движка остаётся окончательным отказом; CLI не подбирает удобное значение. Проверяются tag `IBusEngineDesc`, точная сигнатура attachments `a{sv}` и непустое имя; дополнительные поля дескриптора допустимы.

Физическое удаление полностью возвращено к принятому исходнику: 2 мс между клавишами, 16 мс после пачки. Три файла эмиссии/точного replay совпадают с принятыми копиями побайтно; сборщик проверяет их SHA. Все lease/focus/suppression/decoder/verifier-проверки, модели, обучение, входные источники и выходные маршруты сохраняются.

## Измеренное основание и отвергнутые варианты

Старые 115/p95 222 мс — семь смешанных AT-SPI-наблюдений, не устойчивый p95 и не чистое время ядра. Приватная локализация измерила медианы dispatch 5 мс, replay 69 мс, delete 28 мс, layout 24 мс. Отдельно 30 пар чтений совпали по движку: CLI 5,362 мс, native 0,386 мс. Это основание для нового эксперимента, а не доказанный выигрыш нового RPC.

Паузы 4 мс и 8 мс отклонены. При 4 мс потерян Firefox normal/space/field6 burst PASS (`context_authority`, точная причинная связь с паузой не установлена). При 8 мс Firefox сохранил все 216 вердиктов, но Chrome потерял functional/normal/space_ru_en/field8 PASS: вместо `lay ` появился `дфнlay `. Даже при улучшении скорости такой кандидат не устанавливается. Полный 8 мс результат: Firefox 170P/27F/19B, Chrome 178P/24F/14B, всего 348P/51F/33B из 432. Чистая принятая база: 349P/50F/33B. Все исходники, сборки, сравнения и первичные receipts сохранены; повторять неизменённый кандидат до PASS нельзя.

Измерительные вставки удалены: у временного timing sink не было field-specific NoHistory fence. Пользователю они не устанавливались. Нельзя исключать их файлы из строгой проверки приватного режима. Текущий getter ничего не пишет в журнал/историю и не обращается к Shared/text/admission.

## Безопасность отправки и пределы времени

Первый getter с отменой всего `call_method` после 150 мс был отклонён на ревью 7/10: zbus может хранить позицию частичной отправки внутри отменяемого future. Новый getter никогда не отменяет отправку/ожидание backend-запроса. Существующие 250 мс демона ограничивают ожидание ответа после полной отправки session-bus frame; таймаут клиента не отменяет detached server handler.

RAII atomic-флаг допускает не более одного незавершённого backend-наблюдения на процесс IME; остальные обращения возвращаются к прежнему CLI. Это учёт ресурса запроса, не кеш или владелец входного состояния. Backend-операция завершается ответом/ошибкой/разрывом соединения. Для backend send/reply и прежнего CLI нет новой общей временной границы: обещать 150 мс или общий предел 250 мс нельзя. Старые lease/poll/deadline не меняются.

## Выполнено

- Явное решение: `docs/architecture/decisions/2026-10-15-observed-ibus-engine-readback.json`; owner и журнал экспериментов обновлены.
- Ревью плана 8/10: учтены scope только exact verifier, честные границы времени, ошибки/совместимость и полный native gate.
- 11 регрессий: свежий backend при фиксированной/poisoned Shared-проекции; типы/расширения/неверный dict; реальный RPC, старый IME, ошибка и отключение клиента; fallback один раз, mismatch без fallback, CLI failure; поздний backend-ответ; настоящая однобайтовая частичная отправка и backpressure через таймаут клиента с последующей исправной работой соединения. При провале тест снимает backpressure через RAII.
- Пять направленных транспортных/типовых проверок PASS. Полный выбранный набор IME + daemon + шесть контрактов: **1134/1134 PASS**, 0 FAIL, 271,9 с. Local `/home/ubu/.cache/lay/development/run-3recp7ey/RESULT.json`; remote `run-dWWz50`; archive `af0d9b36e84e9612b54708a485c2df608fb0a82afeedd7a6135fd20ccea67d4f`. Это development scope, не all-library/release acceptance.
- Заключительное независимое ревью кода 2/2: **9/10**, блокеров нет. `PERF-readback-review/CODE-REVIEW-2-RESULT.md`. Замечание без блокировки: отдельный backend disconnect во время запроса не выделен в самостоятельный тест; client disconnect, backend error и RAII покрыты. Native gate не отменяется.
- Встроенный запуск агента упёрся в общий лимит. Ревью выполнены отдельными ephemeral read-only процессами Codex с новым контекстом, без памяти переписки, реализации, GUI, тестов или SSH со стороны ревьюеров.

## Выполнение и итоговые доказательства

1. Новая пара по финальному source1134 собрана и прошла canonical AST/architecture.
   Нативный predecessor83: IMEf633f6aa…, daemonba6d9d77…; RUN/PERF-read-build.
   Четыре фиксированные матрицы PLAINV/READV × Firefox/Chrome завершены по216:
   одинаковые actor hashes, исходные кейсы/клавиши/часы, замороженные private data.
   Все36первичных receipts скопированы локально и проверены поSHA.
2. Полный READV: Firefox170P27F19B, Chrome179P24F13B, всего349P51F32B/432,
   NOT_TESTED0. Ни один из349 предыдущих PASS не потерян при сопоставлении всехID
   как с историческим PLAIN, так и со свежим PLAINV. Один Chrome undo/search2:
   преждеBLOCKED подготовки, теперьFAIL Space. Это дополнительный наблюдённый
   отказ; его причина не объявлена исправленной или вызванной оптимизацией.
3. Скорость на удалённом нативном стенде, PLAINV→READV:

   | Браузер | Замеров/требуется | median, мс | p95, мс | max, мс | paired gain, мс |
   |---|---:|---:|---:|---:|---:|
   | Firefox |420/420|107,254→98,507|118,259→111,112|133,326→120,746|8,235|
   | Chrome |406/420|92,161→83,306|112,631→100,073|124,885→106,859|9,801|

   14missing Chrome сохранены. Clock: evdev-write-return→AT-SPI exact target;
   это не только ядро, не kernel release и не paint. Все6timing-групп с
   denominator420/48/210/50/10/10 сохранены. Space ускорение не заявляется;
   Tab/selection недостаточно измерены. Старые7локальных115/p95222 не являются
   прямой контрольной выборкой для этих удалённых полных замеров.
4. Реальный zero-key API probe30пар на каждый браузер: getter и CLI совпали
   по имени; fresh/current source, PID/tick/cgroup/IBus checks PASS. API cost
   учитывается отдельно от нативного ввода. Обе Chrome privacy проверки PASS:
   обычныйrecent_actions positivecontrol изменяется, privatevisitedLayroots нет.
   Полнота всехtrace/sink не заявляется. Никаких исключений NoHistory нет.
5. Выпущен и установлен core1.0.84 с тремя бинарниками и синхронными двумя
   файлами метаданных расширения. Равенство всех739Rust-файлов, зависимостей и
   кода расширения83→84 доказано; меняются только version/date. Canonical
   architecturePASS, build232,239с. Функциональные проверки заново не запускались
   для номера версии; новый физический84 ввод не заявляется.
6. CLI/tray84, installed и loaded/procSHA сверены. CLI60de3f20…,
   daemon6d0237e6…, IME7e338ae2…. Сохранены24прочихфайла, globalIBus/L1.1/L3
   process identities, config, sources, layout/mode. Backup5файлов проверен;
   existingmanaged restart и channelime, безglobalIBusrestart и0функциональных
   локальныхклавиш. Receipt:
   `/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6/install83-20261015T033053Z/INSTALL.json`.
   Имя каталога историческое; внутри записана версия84.

## Сохранённые ограничения и долг

Ранний READ prefix/field1 FAIL сохранён в RUN/PERF-READ-firefox-incomplete-2.
Текст обратного преобразования был точен, фактический mode тогда не записывался;
путь — focusedIME без оптимизированного daemon consumer. Причинная связь UNKNOWN.
Один диагностический wrapper не завершился из-за __file__; его частичные
наблюдения не объявлены приёмкой. После усиления только подготовки и фиксации
mode весь новый Firefox216 совпал с обеими базами. Повтора неизменённого FAIL
до PASS не выполнялось: использован один новый согласованный протокол.

После всех9COMPLETE Chromejobs последний wrapper дал guard75 cleanup. Оставшийся
только privateL1.1 закрыт pidfdSIGTERM после UID/PID/tick/SHA/cgroup/socket
проверок; затем collector без ввода подтвердил allgone/CGempty/VT1, seat закрыт.
Guard75 сохранён, не переименован вPASS. Долг инфраструктуры: systemd249 знает
--kill-who, guard использует --kill-whom; supervisorSIGTERM способен прервать
finally гостя. Guard/runtime ради PASS не менялись. Recoveryreceipt:
RUN/PERF-MATCHED-v2/READV-chrome-closure-recovery.json.

TD-121 остаётся OPEN: прежние функциональные отказы/блокировки не входят в
ускорение TD-141. Установленный Telegram новой84 нативно не перепроверялся;
заявленная нативная область — удалённые Firefox/Chrome и metadata-only перенос.
Результат не обещает работу во всех приложениях или обучение/улучшение моделей.

## Артефакты

RUN=`/home/e/projects/lay-development-runner/firefox-legacy-native-20261012.6onP5c`;
BDIR=`/home/ubu/.cache/lay/development/remote-firefox-setup-j2he3qf6`.

- Fullnative: RUN/PERF-{PLAINV,READV}/{firefox,chrome}/RESULT.json;
  local BDIR/PERF-{PLAINV,READV}-{firefox,chrome}.
- AllIDs/speed: RUN/PERF-READV/{browser}-vs-{PLAIN,PLAINV}.json;
  local BDIR/PERF-READV-{browser}-vs-{PLAIN,PLAINV}.json.
- Integrity: BDIR/PERF-MATCHED-v2/LOCAL_PROOF_INTEGRITY.json.
- Finalbuild: RUN/release84-build/BUILD.json, localBDIR/release84-user-install/BUILD.json.
- Decisions:2026-10-15-observed-ibus-engine-readback.json,
  2026-10-15-matched-native-initial-stack-readiness.json,
  2026-10-15-observed-readback-release84-metadata-only.json.
- Currentreport: docs/development/double-shift-latency-2026-10-15.md;
  полный прежний текст — соседний experiment-history.md.
