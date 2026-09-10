# TD-121: Сохранять целое слово при смене экземпляра IME

Status: `IN_PROGRESS / FIRST_WORD_FIX_INSTALLED / PHYSICAL_PENDING / COMBINED_CONTEXT_OPEN / COLD_START_OPEN`
Priority: `P0` — исправление части видимого слова как отдельного токена
Stage: 1 — сначала доказательство механизма, затем минимальная реализация
Size: M
Base commit: `cc1e2207519801ca0f9b7c6963897b55953a7751`
Initial admission: пользователь разрешил фазу 1 (анализ/проектирование).
Continuation 2026-09-05: «дальше без остановок пожалуйста» допускает phase-2
TDD/исправления после design gates; install/release остаются отдельным этапом.
Analysis model: `GPT-6 Astra / xhigh`.
Implementation model: `GPT-5.6 Sol / high`, после закрытия design gates ниже.
Текущая цель пользователя: релиз 1.0.66 и push; phase-2 допуск уже получен.

## Выбранный маршрут и текущая транспортная проверка

**Sep8, последнее сообщение пользователя:** Double Shift не работает на первом
слове без ведущего пробела. На установленном fe643c10 новый private-client
baseline0/2 FAIL. Минимальный terminal-only допуск наблюдённого suffix реализован
в существующем WordLineage; UnknownStart не получает automatic/generic authority.
Review pass1 выявил stale atomic prefix и numeric delegation; оба дефекта дали
causal RED и исправлены. Focused448/448 PASS, run-TcpGeO,15.463s; review2
9/10,H0/M0/L0; changed2683/2683. На6dc95148 отдельные холодные US/RU старты
прошли2/2,16 точных преобразований; manual/preedit3/3, lifecycle3/3,
restoration5/5. Совмещённая смена поля+профиля до текста остаётся1/2 FAIL,
её причина и baseline parity не доказаны. Full gate2683/2683 PASS495.756s.
Sep8 11:50:25 +03:00 установлен6dc95148, PID2836850; физическое подтверждение
запрошено и пока не получено. Только explicit terminal manual eligibility
изменена; это не закрытие всего lifecycle/GTK/cold-preedit контракта.
Точные доказательства и ограничения — в owning journal ниже.

**Sep8, новое наблюдение пользователя:** Double Shift срабатывает один раз,
обратное преобразование не работает. Пользователь снова разрешил тесты.
Текущий узкий маршрут — settlement вывода bridge перед передачей IME;
[наблюдение, последствия и результаты](evidence/release-1.0.66-double-shift-recurrence.md).
Прежний запрет на тесты ниже исторический. Установленные bytes пока прежние.

**Sep8 04:31 +03:00, результат:** установлен `fe643c10` с исправлением
bridge settlement и повторного ContentType. Focused442/442, полный release
gate2677/2677, review9/10,H0/M0; реальный клиент с текущими локальными
зависимостями:16 преобразований на сценарий startup schedule, post-ready
manual/preedit3/3, lifecycle3/3, restoration5/5. Немедленный cold preedit
остаётся FAIL; физическое подтверждение пользователя ожидается. Точные
ограничения и установка — в журнале выше. Старые результаты ниже исторические.

**Sep8,03:05:52 +03:00, последний результат:** по прямой команде пользователя
тесты прекращены; свежая release-сборка последних исходников12db8c00
установлена постоянно и работает как IME147597. Сборка только IME удалённо,
jobs20,58.72s; IBus4715/daemon3757261 не перезапускались. InputState отвечает
`passive:daemon-word-buffer`. Реальный ввод ещё не подтверждён; это не DONE
и не закрытие cold-start. Полный correctness/package прогон успел пройти
2672/2672, но общий release script завершился ошибкой clippy, не PASS.
[Точная точка продолжения без повторного цикла тестов](CONTINUE.md).
Описанные далее «следующие этапы» — исторические планы до этой команды.

**Sep8, текущий маршрут:** после полного чтения2878 строк пользователь
потребовал минимальный надёжный ремонт адаптера. Незавершённая отдельная
debug-надстройка остановлена, существующий debug_action_log уже включён.
Выбран локальный ремонт: отказ reducer в CreateEngine не должен завершать
общий observer и не должен выдавать этому callback никакого допуска.
Подробные последствия/границы записаны в owning journal до правок.

Минимальный ремонт теперь подтверждён causal RED/GREEN:437/437 focused PASS;
независимое fresh-context review9/10,H0/M0. Оптимизированный кандидат
e596b507 собран удалённо, архитектурная проверка PASS. Следующий этап —
обязательные affected/release и actual-client проверки этих bytes, затем
контролируемая физическая проверка с откатом; постоянной установки ещё нет.
Это не закрывает прежний cold-start FAIL или отдельный physical FAIL706b.

**Sep8, финальный source checkpoint:** широкий прогон выявил гонку ожидания
в существующем тесте; oracle выровнен по реальному outbound marker, без sleep
и ослабления assertions, review10/10. Лишнее production-поле идентичности
готового запроса оставлено только под cfg(test), поскольку его единственный
потребитель — тестовый witness; финальное review10/10,H0/M0. Adapter60d50d52,
residuals28dd8b76, focused437/437 PASS. Последующий release gate завершился
ошибкой clippy; текущая установка и границы результата описаны выше.
Все неуспешные промежуточные проверки сохранены в owning journal.

**Sep8,01:30 +03:00:** пользователь подтвердил, что сам перезапустил
установленные Lay/IME. Текущий IME3757358 — старый установленный95348e4a,
не706b preview; InputState снова cancelled. В его сохранённом журнале
CreateEngine98 отказан после состояния transfer_revoked (owner5/activation5).
Это не устанавливает причину отдельного отказа706b. Новых действий с
сервисами при этой проверке не было; optional debug ещё в реализации.

**Sep8,01:11 +03:00:** пользователь сообщил «не работает!». У706b observer
снова cancelled; preview остановлен после возврата на native xkb:ru::rus.
IBus/keyboard daemon сохранены.500KiB trace вытеснил начало отказа потоком
повторных отказов. Пользователь явно запросил debug-режим как отключаемую
опцию; выбран bounded first-failure metadata recorder, default OFF. Это
диагностическая правка, не доказанный новый ремонт переходов.

**Исторический preview,22:18:58 +03:00:** после разрешения пользователя
706b4d81 запущен из cache как PID2841233, установленный файл не заменён.
Старые IBus4715/daemon272240 сохранены. Plain FocusIn работает:242/242
key admissions и242/242 settlements приняты, observer отвечает, preedit
показывается. Это не подтверждение автозамены/Double Shift: ручная приёмка
нового кандидата ожидается; предыдущий physical FAIL сохранён ниже.

**Принятый ремонт,2026-09-07:** пользователь выбрал минимальное исправление
переходов в существующем reducer. Подтверждены RED для refocus без Disable,
перекрытия пустых factory и трёх случаев потери исходного Get при ранней
клавише/Reset/ContentType. Финальный runtime:436/436 focused и2671/2671 affected
PASS, независимое source review9/10,H0/M0. Оптимизированный706b4d81:
реальный lifecycle3/3, restoration после exact-ready5/5; immediate cold0/5
(нет замены первого слова). Старый15728764 также проходит corrected lifecycle3/3:
прежний0/3 был ошибкой ожидания теста для A->dummy->A, не causal RED runtime.
Тест исправлен без изменения production-кода;29/29 tooling PASS, финальное
review исправленного witness9/10,H0/M0. Causal RED
остаются controlled production-callback regressions. Установки, новых
перезапусков на этом source-only checkpoint, commit/push и успешной физической
приёмки этого ремонта не было. Поздний разрешённый preview указан выше.
Проверенный срез20:44: выбран `lay-ime-ru`, процесс установленного95348e4a,
InputState=`metadata observer cancelled`. Native fallback ниже исторический.
[Текущий owning journal и точные RED receipts](evidence/release-1.0.66-factory-recurrence.md#accepted-systemic-lifecycle-repair-2026-09-07-2044-0300).

**Физический отказ,20:24 +03:00:** пользователь сообщил «неработает».
Кандидат15728764 остановлен после проверенного возврата на `xkb:ru::rus`.
151/151 retained key admissions отказаны; observer затем остановлен на
CreateEngine85. IBus4715/daemon272240 неизменны. Предыдущие PASS ниже не
доказывают работу на рабочем столе. Сохранённая1.0.65 проверена только по
наличию/версиям; откат не выполнен и требует отдельного решения пользователя.

**Исторический checkpoint,20:06 +03:00:** временный IME15728764 запущен из cache,
PID2128979; установленный бинарник не заменён.425/425 focused, review9/10,H0/M0,
post-exact-ready client5/5. Immediate cold client FAIL: первый Space раньше
готовности словаря. Bridge=passive:no-focus; физическая проверка ещё нужна.
Глобальный IBus4715 и клавиатурный демон272240 не перезапускались.
[Единый актуальный журнал ремонта и безопасного запуска](evidence/release-1.0.66-factory-recurrence.md).
Результаты ниже исторические; задача не DONE, новый commit/push не выполнен.

**Новый живой отказ,17:17 +03:00:** старые процессы исключены по хешам;
тот же новый IME повторно остановил observer, теперь на `CreateEngine123`
после `transfer_revoked`.526/526 последующих key admissions отказаны.
[Точная последовательность и отдельный ограниченный переплан](evidence/release-1.0.66-factory-recurrence.md).
Предыдущая установка не прошла физическую приёмку; задачу не закрывать.

**Обновление 2026-09-07, 17:03 +03:00:** исправленный IME установлен,
PID1148501, SHA256 `95348e4a...23a668`; bridge отвечает `passive:no-focus`.
416/416 focused, 2651/2651 changed, exact release client5/5, оба lint scope и
architecture PASS; финальное review9/10,H0/M0. Глобальный IBus4715 не менялся.
Физическая проверка пользователем ещё не выполнена, задача не DONE.
[Полные доказательства и два исхода установки](evidence/release-1.0.66-live-observer-incident.md).

**Инцидент 2026-09-07, 16:10 +03:00:** живой IME возвращал
`metadata observer cancelled`; журнал подтверждает остановку обработчика
контекста на `FocusOut` и последующий отказ всех 490 сохранённых key callbacks.
Предыдущие PASS ниже не являются доказательством работоспособности текущего
ввода. [Причина, альтернативы и границы исправления](evidence/release-1.0.66-live-observer-incident.md).

Исторический результат до инцидента: V2 actual-client **5/5** на final release-profile candidate
`dfeb50e8` после явной готовности exact-контура. Полный ` ljv` становится
` дом ` без потери пробела; смешанное слово сохраняется при US→RU и RU→US;
чужое поле и UnknownStart не получают права исправления. Remote changed и full
gates: каждый **2646/2646 PASS**, включая IME411 и protected7. Исправлены lifetime готового результата, enrichment позднего
native receipt для Transfer, отмена незавершённого пустого контекста новым
factory и запрет поздней публикации отменённого fence. Все исправления
связаны с RED/GREEN; pending→reducer порядок блокировок сохранён.
[Точные receipts, анализ последствий и review](evidence/release-1.0.66-final-client-route.md).
Холодные0/5 остаются отдельными failed runs, не заменены post-ready PASS.
Смешанная коррекция как отдельный verdict и физическая клавиатура этим стендом
не доказаны. Lint/architecture/full gates и exact release-profile client прошли;
комплект перенесён с полной SHA-256 проверкой, rollback snapshot готов.
По последующей прямой команде пользователя1.0.66 установлена с rollback
snapshot; загруженные процессы и их SHA-256 проверены независимо.
[Установка](evidence/release-1.0.66-installation.md). Впереди physical
confirmation; статус пока не DONE. Установленная и запущенная версия1.0.66.

Предыдущий source checkpoint: [независимая проверка четырёх исправлений](evidence/td121-residual-acceptance-review.md)
**9/10,H0/M0**, exact current source/retained tests проверены. Семь residual
tests и оба actual-Tab readiness tests присутствуют и PASS в IME402/402.
Защищённый composition successor принят как source checkpoint, не как релиз.
Последующая механическая lint-правка принята10/10,H0/M0 и перепроверена
обоими финальными gates. Следующий обязательный шаг — физическая проверка.
Описанные ниже4/10 и PROPOSED — исторические checkpoints до этой перепроверки.

Продолжение пользователя 2026-09-07: «не останавливайся к 66 версии иди».
Ограниченный план завершения: независимая перепроверка только четырёх уже
исправленных High из pass2 и их causal regressions (не третий общий аудит и
не новый неограниченный repair); отдельный явный successor некорректного
client contract; затем source-bound promotion, canonical/release gates и
проверенная доставка. Оценка TD-12510/10 не переносится на TD-121. Старый
driver и его0/5 остаются историческими доказательствами. Сначала установить
причину cold exact refusal; не повторять старый клиент и не увеличивать waits.
Все сборки/тесты/обновления графа только на remote под прежними ограничителями.
Само продолжение не переводит PROPOSED в ACCEPTED и не разрешает reboot IBus.

Новейший diagnostic checkpoint2026-09-07: отдельный `NameHasNoOwner` после
переноса стенда объяснён отсутствующим L2 lexical artifact; один hash-bound
dependency устранил SIGABRT, production-код не менялся. Обычный новый runner
с nine-role manifest оставляет IME живым и снова показывает `prefetch_not_ready`,
client0/5. Это не acceptance и не третий runtime repair.
[Доказательства и новый отчёт о потере пробела](evidence/td121-client-boundary-diagnosis.md),
[TD-125](125-preserve-autocorrection-left-boundary.md). Терминальный consumer
и противоречие capabilities/assertions frozen driver требуют отдельного proof.

Текущий checkpoint 2026-09-07: IME399/399 PASS за15.49s, final-pass residual7/7.
Настоящий клиент подтвердил перенос полного префикса ` l` → ` ljv` между
экземплярами в одном контексте. Полных cases по-прежнему0/5: и debug, и
оптимизированный кандидат отказали с `prefetch_not_ready`; бюджет не расширен.
[Точный результат](evidence/td121-private-client-proof.md#full-prefix-handoff-pass-cold-debug-correction-not_ready--2026-09-07).
[Final review pass2](evidence/td121-code-review-pass2.md):4/10,H4/M0. Нового
общего круга ревью нет: один явно ограниченный второй repair существующих
инвариантов; при новом непокрытом механизме остановить этот план и сообщить.
RED/исправление/GREEN четырёх receive-order/completeness дефектов выполнены.
Оптимизированный client proof остался красным; очередной такой же прогон
не запускается. Требуется различить холодную lexical readiness и иной отказ
exact-сертификата. Дополнительная узкая проверка исправлений независимым
ревьюером отдельно запрошена у пользователя: текущая оценка4/10 до исправления
не заменена выдуманными8/10 и release-gate не ослаблен.
Релиз/install/DONE пока запрещены.

Исторический checkpoint до legacy proof: combined adapter22/22 и IME390/390 PASS;
кандидат `b7e78375…24e81c`. Изолированный настоящий IBus-клиент завершил0/5
cases: первый literal Space виден клиенту, но следующий snapshot остался
`passive:unknown-context`. Source-free activation установлена; порядок
marker/key и причина отказа legacy callback не записаны. Поэтому этот прогон
не различает ожидаемый ввод до готовности и дефект уже готового legacy пути.
Новые word-scope tests исполняют atomic callback, а не этот legacy envelope.
Следующий узкий шаг — доказать оба порядка на actual legacy entrypoint и
добавить недостающую causal metadata; никаких новых RPC, authority owners,
таймаутов или ослабления positive assertion. Подробности и последствия —
[current analysis](evidence/td121-context-admission-analysis.md#legacy-client-first-boundary-checkpoint--2026-09-07).
Принятые правила разработки записаны в `AGENTS.md`; это не допуск широкой
миграции в1.0.66. Исторические checkpoints ниже не являются final acceptance.

Checkpoint 2026-09-06: frozen reducer/ordered-zbus/rendezvous/adapter —
30/30 PASS на exact zbus 5.15.0: 25 reducer/merge/rendezvous и 5 actual-zbus
controlled p2p tests. Это supersedes прежние helper-only 19/19, но остаётся
отдельным staged source, не подключённым runtime. Production wiring,
ProcessKeyEventAtomicV1/ContentType headers и client proof этим не покрыты.
Evidence: [helper/adapter record](evidence/td121-pure-helper-implementation.md).
Дальнейшее подключение описано в [wiring handoff](evidence/td121-runtime-wiring-sol-task.md)
и начинается после TD-120 source checkpoint и adapter gates. Ни исправление
production input, ни релиз/install/push этой записью не объявляются.

TD-120 checkpoint закрыт и запушен:
`ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`. Подключение TD-121 начато;
изменения adapter для AtomicV1/Properties.Set и actual composition guard
требуют собственных tests/review. Frozen 30/30 не покрывают эти новые bytes.
В context-admission analysis отдельно согласован узкий TD-121 successor
для protected composition source; TD-113 и TD-120 история остаётся неизменной.

Поздний checkpoint 2026-09-06: remote `lay-ibus-engine` suite 379/379 PASS
за 14.35s; [точный лог и SHA-256](evidence/release-1.0.66-execution.md).
Это предшествующие bytes: проверка родителя выявила разрыв receive-order
metadata / handler settlement и отсутствие native-path проверки FocusOutId.
Идёт один ограниченный repair этих owner/ordering инвариантов. Промежуточный
standalone 1.0.66 собран удалённо для client diagnostic, но не допускается
к установке. Реальная correction-on проверка, C/H ledger и независимое
финальное ревью остаются обязательными; 379 не является release denominator.

[Независимый pass 1](evidence/td121-code-review-pass1.md): **3/10, High 6,
Medium 2, REPAIR_REQUIRED** для frozen snapshot до текущего repair. Замечания
охватывают receive-order authority, seal/Disable epoch, old-owner shared writes,
completeness по реальному effect, atomic prior settlement, layout-request
identity, acquisition timeout/locking и native enrichment. Это static review,
не исполненные repro. Следующий проход — второй и заключительный; task DONE
и successor ACCEPTED запрещены до фактического закрытия замечаний и gates.

[Матрица закрытия C01–C30 / H01–H16](evidence/td121-promotion-coverage.md)
разделяет source pointers, исторические прогоны и ещё открытые проверки
финальных bytes. Число строк матрицы не является числом выполненных tests.

[Полный Astra/XHigh analysis](evidence/td121-context-admission-analysis.md)
выбрал единый canonical context contract: bounded metadata observer на той же
IBus connection вне speculative SharedState; native receipt либо необходимый
cached-false property adapter; один self-signal marker через очередь IBus.
Factory/focus ticket, receive/settlement ordering, generation revocation,
sticky UnknownStart и bridge/atomic/layout consumers входят в один контракт.
Оценка8/10 — проектное решение, не результат implementation review.

[Изолированная проверка установленного настоящего IBus](evidence/td121-real-ibus-transport-baseline.md)
выполнила22/22 транспортных cases. Marker после foreign ABA наблюдён20/20;
Get+marker537–1108µs на21 observation, не production percentile. Рабочий IBus
не перезапущен. Adversarial early-reply ordering0/20 наблюдений, callback/ring/
completeness и positive Lay restoration ещё требуют отдельных tests.
Этот результат не переводит C01–C30 в PASS и не закрывает задачу.

Финальный addendum того же анализа ограничил оставшийся допуск тремя tests:
P121-1 actual reducer с задержанной FIFO и положительным контролем;
P121-2 настоящий zbus OrderedStream/Join + четыре lost-wake schedules;
P121-3 sender identity и общий бюджет Space. Чистый helper/tests разрешены до
wiring. Initial missing stamp ждёт только локальное уведомление≤1ms, не RPC;
затраченное время вычитается из существующих3500µs Space wait. Не добавлять
этот budget поверх прежнего. Production authority подключается после P121-1/2/3;
полная positive restoration и C01–C30 остаются обязательными release gates.

Отдельный real-IBus header run23/23 уточнил P121-3: Factory/lifecycle methods
приходят от `org.freedesktop.DBus`, GlobalEngineChanged — от unique owner IBus,
self-marker — от собственной connection. Реальная попытка подделать Sender
другой connection переписана IBus в её настоящий sender. Не сравнивать все
три роли с одним unique name и не hardcode-ить ephemeral names. Artifact/hash
и непроверенный ProcessKeyEvent scope — в том же transport baseline документе.

## Что должно измениться

При переключении раскладки внутри одного поля текущий **полный** токен не
должен терять уже введённое начало. Если приложение отображает `ljv`, нельзя
воспринимать только `jv` как целое слово и исправлять его отдельно в `ом`.
Переход в другое поле, напротив, обязан уничтожать старое право на его текст.

Запрещены восстановление невидимой буквы из словаря, прибавление Backspace,
угадывание по pixel cursor, чтение старого DaemonWordBuffer вместо IME lease,
автоматический повтор изменения и ослабление SafetyGate/verifier.

## Подтверждённый production-эпизод и предел доказательства

Сохранённые журналы 1.0.65, 2026-09-05 12:12:12–12:12:14 EEST:

```text
ctujlyz → сегодня          Applied, 7 символов
ru layout sync ok
us layout sync ok         origin этого события не записан
новый engine us/57
его tail перед j пуст
j → jv                    длина 1 → 2
Space: jv → ом             Applied, backspaces=2
пользователь видит lом
```

Точно установлено: на correction boundary вошёл неполный по наблюдению
пользователя токен. Первый установленный разрыв находится **до**
`InputFrameIdentity → L1.1/L2/L3/L4 → DecisionCore → verifier`, в наблюдении и
переносе текста. Поздние слои проверили замену `jv`, а не `ljv`.

Не установлено: дошла ли именно историческая `l` до старого engine, прошла
native passthrough во время смены, либо её запись потерялась в lossy logger.
Нет времён каждого key/focus события и origin переключения. Координата курсора
не доказывает символ. Не называть этот эпизод timeout смешанного `lом`:
mixed-script candidate в нём вообще не является наблюдаемым входом.

## Системный дефект, установленный в коде

1. `ibus_interface.rs::focus_id` возвращает **false**, хотя `FocusInId` и
   `FocusOutId` реализованы. Это не доказанное ограничение Kitty: сам Lay
   объявляет отсутствие capability. Линия существовала с commit `427b3d7e`
   от 2026-06-19, не является доказанной регрессией только релиза 1.0.65.
2. `FocusOut → should_preserve_focus_handoff` сохраняет свежий ввод до 700 ms.
3. Новый `LayIbusEngine::new_from_component` копирует shared tail.
4. Но `FocusIn → bind_focus_path` принимает копию только при действующем
   `preserve_active_path_until`. Без него очищает **локальный и shared** хвост.
   Совпадения настоящего input-context receipt здесь недостаточно.
5. Обычный modifier layout hotkey не открывает такую же preserve lease, как
   text replacement. Сохранить на старом объекте и принять на новом — разные
   критерии. Проверка только старого объекта этот разрыв не обнаруживает.
6. `LayoutSwitchRequest` хранит target/engine без request generation и focus
   identity. Latest-only отменяет ожидающий request, но не уже выполняющийся.
   Это дополнительный race-risk; его участие в исторической `l` НЕ доказано.

Один из существующих тестов проверяет сохранение при reset старого объекта;
другой — правильное отбрасывание expired lease. Нужен тест передачи **между
двумя объектами** в одном и в разных канонических контекстах.

## Канонический источник identity и ограничение hot upgrade

Установлены `ibus` / `libibus-1.0-5` версии `1.5.34~rc2-1`.
Upstream source соответствующего tag `1.5.34-rc2`, commit
`1f7af28437afd62a6d145bfc81035e698a37411d`:

- `bus/engineproxy.c` читает `FocusId`, кэширует ответ по engine name,
  выбирает `FocusInId(context_path, client)` либо обычный `FocusIn`.
- `bus/ibusimpl.c` создаёт focus capability table при init и уничтожает при
  destroy; в просмотренном файле не найден публичный reset этой таблицы.
- Тот же IBus предоставляет read-only property `CurrentInputContext` с
  object path текущего канонического input context. Старый одноимённый метод
  deprecated; для нового кода использовать property, если путь будет принят.

Пять live read-only запросов property 2026-09-05 вернули один context path.
Round-trip: `2630, 632, 399, 280, 258 us` (первая проверка отдельно).
Это **не p95/p99**, не нагрузочный тест, не доказательство handoff или
неизменности focus между двумя ответами.

Следствие: простая правка `FocusId=true` может не изменить работу уже
запущенного IBus после перезапуска только Lay. Global IBus PID сохраняется;
рестарт IBus/session, новые engine names и изменение пользовательского списка
источников нельзя незаметно включить в этот fix.

## Варианты решения и рейтинг

Оценки ниже — design judgement до baseline proof, не разрешение реализации.

| Вариант | Оценка | Применимость и цена |
|---|---:|---|
| Native `FocusInId` capability + передача по совпадающему каноническому context | **9/10 архитектурно; 6/10 для текущего hot upgrade** | Наименьший постоянный механизм; надо доказать activation при кэшированном false без перезапуска глобального IBus |
| Тот же context-bound transfer; bounded `CurrentInputContext` property adapter там, где native receipt ещё недоступен | **8/10 условно** | Не требует нового owner текста или engine names; дополнительный RPC только на lifecycle transition. Необходимо доказать deadline, freshness и A→B→A, иначе не допускается |
| Единственный shared input-context state вместо local копий engine | 6/10 | Более широкая миграция edit/undo/prefetch/atomic/locks; второй этап, identity всё равно нужен |
| Продлить TTL либо переносить хвост при любом новом path | 2/10 | Нельзя отличить смену раскладки от другого поля; риск правки чужого текста |
| Переводить suffix `jv` и потом достраивать начало | 1/10 | Маскирует потерю наблюдения, создаёт вторую мутацию; запрещено |

Рекомендуемый принцип: **существующее состояние IME + канонический context
IBus + один generation-bound handoff**, не таймер как доказательство поля.
Способ получения canonical receipt остаётся admission gate: сначала native
handshake, затем только при доказанной необходимости bounded compatibility
adapter. Не реализовывать оба способа «на всякий случай».

## Обязательные gate до production-кода

1. Reproduce `l → new engine → [empty] → jv/ом` установленными байтами или
   source-bound тестом двух engine. Подтвердить первую букву до перехода и
   хвост после него. Same-path и different-context контроли обязательны.
2. Зафиксировать ровно один выбранный способ получения context receipt на
   **уже работающем** IBus и поведение при cold capability discovery.
3. Разрешить callback ordering: `FocusIn`, delayed `FocusInId`, `Enable`,
   `Disable`, old-path `FocusOutId`. Старый FocusOutId другого context не
   закрывает новый; одинаковый контекст не равен старому request generation.
4. Доказать отсутствие переноса на другой input context при одинаковом окне,
   app name, тексте и быстрых событиях. `A → B → A` не оживляет прежний lease.
5. Для letters during source transition нужна отдельная проверка delivery.
   Сохранение уже наблюдённого `l` не доказывает доставку клавиши, отправленной
   когда IBus переключает factory. Нельзя обещать устранение этой второй
   гипотезы без client-visible/physical evidence.
6. Согласовать реализацию со scope suppression TD-120 и пройти spec review.

Исторический checkpoint до context-admission analysis: phase1 gate1 был открыт;
private probe остановился до RPC из-за AppArmor,
0/6. В phase2 отдельный безопасный bootstrap выполнил6/6 сценариев;
[новая baseline](evidence/td120-121-installed-baseline-phase2.md) воспроизводит
потерю уже наблюдённой `l` между объектами, включая same FocusInId, с
same-path/different-context контролями. Gate1 закрыт для класса tracking,
не для точной исторической доставки. Тогда gate2 и ordering/production gates
были открыты (`ANALYSIS_REQUIRED`). Этот admission status superseded полным
context-admission analysis и последующим 30/30 frozen helper/adapter proof;
подключение production допускается после TD-120 Git checkpoint. Production
acceptance и client/physical gates по-прежнему обязательны.

## Эскиз внутреннего контракта после admission

- Разделить `engine object identity`, `canonical input context identity`,
  `word lineage`, `layout request generation`, `candidate frame generation`.
  Не использовать одну строку path или один таймер вместо всех пяти.
- Смена engine в том же подтверждённом context сохраняет актуальные текст и
  word lineage; все старые кандидаты/certificates/preedit acceptance права
  инвалидируются. Перенос текста не переносит право применения результата.
- Handoff одноразовый, привязан к origin/target/request/context. Внутренний
  таймаут ограничивает ресурс, но не доказывает совпадение поля.
- Новое поле, no-focus, sensitive purpose, несовпадающая/просроченная receipt
  не наследуют текст. При неопределённости нельзя исправлять неизвестный
  suffix как доказанно полное слово.
- Не делать get-context RPC на каждом символе. Переход имеет bounded
  acquisition; timeout не блокирует клавиатуру и не запускает retries.
- Смена раскладки остаётся одним GNOME-owned действием. Не добавлять второй
  `ibus engine` после ActivateLayout, ещё один detector или uinput replay.
- Нельзя считать чужой более поздний decoder/layout результат разрешением
  текущему engine. Pending и уже in-flight switch проверяются раздельно.

## Consequence analysis

| Измерение | Ожидание, риск и требуемое доказательство |
|---|---|
| Retention / completeness | Уже наблюдённые символы сохраняются только в том же context. Неполный хвост не становится полным через догадку; отдельно проверять незарегистрированные первые клавиши |
| Ranking / false authority | Лексическая конкуренция неизменна; вход становится полным. Копировать candidate authority между engines нельзя |
| Latency / tail | Native receipt предпочтительна; дополнительная property query только на lifecycle, с отдельным измерением cold/warm/failure. Никакого увеличения Space deadline ради handoff |
| CPU / RSS / allocations | Один bounded transfer record у существующего SharedState; без новых resident workers, очередей клавиш, polling или копий corpus |
| Cache / identity | 700ms не identity; FocusId capability cache живёт в IBus. Bus reconnect, old reply, same path/different context, A→B→A требуют отрицательных тестов |
| Package / reload | Material generation и frame сертификаты продолжают инвалидироваться; context receipt не зависит от версии словаря |
| Learning | Источник typed/accepted/censored остаётся привязан к реальному полному слову и edit receipt. Не обучать `jv` как полный ввод, если scope неполон |
| Concurrency | Порядок DBus callback/worker request закреплён тестами, не sleeps. Locks не удерживаются во время внешнего вызова и не нарушают atomic clone/commit |
| Failure / rollback | Нет focus receipt — нет догаданной мутации. Failed switch не делает второй replay и не переносит хвост в другой контекст. Откат не требует удаления словарей |
| Consumers | Both Shift/Alt orders; ManagedCommit width11, narrow TerminalPassthrough, GTK SurroundingText, daemon exact replay, Tab/undo проверяются отдельно |
| Maintenance | Native callback и optional compatibility acquisition сходятся в одну типизированную identity, не в два конкурирующих решения. Full shared-state rewrite не допускается этой задачей |

## Матрица TDD и доказательств

Записать manifest до изменения поведения; фиксировать число конкретных
раскрытых cases, а не выдавать строки таблицы за число выполненных тестов.

| ID | Обязательные ветки | Что проверяется |
|---|---|---|
| H01 | US→RU, RU→US, US→US new path | Сохранность уже наблюдённого полного токена и точные output effects |
| H02 | Same path quick FocusOut/FocusIn | Нет регрессии существующего сохранения |
| H03 | Новый path / тот же FocusInId | Canonical identity важнее object recreation; старые candidate frames непригодны |
| H04 | Different context, same app/window/text | Нулевой перенос текста/guard/edit authority |
| H05 | A→B→A / старый FocusOutId | Старый lease не воскресает и не очищает новое состояние |
| H06 | Перед 700ms / после 700ms и delayed callback | Bounded ресурс не подменяет identity; управляемое время, не flaky sleep-тест |
| H07 | Shift→Alt, Alt→Shift, пользовательский layout intent после auto-sync | Один owner, последнее действующее намерение; нет двойной смены |
| H08 | Pending switch / уже выполняющийся switch | Раздельная invalidation, без stale decoder authority |
| H09 | Буква до / во время / после смены | Доставка и tracking считаются отдельно; потеря/дублирование 0 в проверенном клиенте |
| H10 | ManagedCommit width11 / TerminalPassthrough width2 | Не путать profile с названием приложения либо purpose=terminal |
| H11 | SurroundingText on/off; FocusId cold/warm cached false | Реальные capability combinations; no global IBus restart |
| H12 | No context / disconnected bus / slow query / stale query | Нет опасного fallback, клавиатура не зависает |
| H13 | Mixed token физическая проекция, clean English, protected tokens | Полный frame проходит обычную конкуренцию, не lexical exception |
| H14 | Double Shift / 4 taps / accepted Tab / auto-undo | Один detector, один edit, round-trip без append/reversal |
| H15 | Package reload / config change между enqueue и apply | Старые correction certificates не применяются |
| H16 | Atomic prepare / abort / commit | Нет live side effects до commit и повторного расходования lease |

Для каждого case записывать отдельно: observed keys, CommitText stream,
client-visible text (если измерен), internal tail, input context, engine path,
word/request/frame generations, exact edit plan, verdict, left-context effect.

`ljv → дом` и `lом → дом` — разные корпуса: полностью неверная раскладка и
уже смешанный ввод. Не заменять проверку первого frameless CLI второго.

## Проверки / установка / review

После admission — focused tests на выделенном remote20CPU хосте, только
`scripts/cargo-guard.sh --bin/--lib ...`, затем changed gate и
`scripts/update-architecture-graph.sh`. Никакой локальной Cargo-сборки.
Существующий `scripts/runtime_smoke` управляет рабочими источниками/службами;
не запускать его в качестве «read-only isolated diagnostic».

Fresh-context review Astra/XHigh: ≥8/10, High/Medium=0, максимум 1–2 repair passes.
Release/install отдельно: global IBus PID сохраняется; cached capability
не считается обновлённой по одному version/hash бинарника. Реальная клавиатура
и видимый клиент проверяются до DONE. Статус, commit и push только после всех
обязательных доказательств; текущий документ не является исправлением.

## Отложено во второй этап

Единый экземпляр IME для обеих раскладок, перенос всего input context state
в новый owner, новые engine names ради обхода кэша, системный IBus restart,
универсальная очередь физических клавиш. Для каждого нужно отдельное
обсуждение пользы и рисков; не включать в Sol-задачу молча.

## Ревью спецификации

[Pass 1](evidence/td120-121-spec-review-pass1.md): документ **8/10**;
`ANALYSIS_REQUIRED` подтверждён как корректная граница. В
[pass 2](evidence/td120-121-spec-review-pass2.md) проверена согласованность
с repaired TD-120; оценка TD-121 сохранена, новый полный аудит не заявляется.
Оставшиеся High/Medium замечания к рассмотренным описаниям — 0/0.
На момент этих исторических reviews двухобъектное воспроизведение и canonical
receipt ещё не были доказаны. Поздние baseline/transport/helper результаты
записаны выше отдельно; они не переписывают первоначальный private 0/6 и не
объявляют непроверенное production wiring готовым.
