# TD-133 — Причина порядка первого Space в GTK/Qt

Status: IN_PROGRESS_CAUSAL_INVESTIGATION. Priority: P0. Size: unknown до trace.
Owner: existing WindowInteraction/ContextAdmissionReducer/output.
Depends: current exact-byte proof, не новая модель. C03,C05,C07,C08,C10.
Новому physical run предшествуют case-binding130 и partial-cleanup recovery137
либо доказанная эквивалентная существующая безопасная граница. Пока они open,
разрешены read-only old trace и source event-order investigation; dependent
live feedback cases не запускаются.

## Факты и нерешённый переход

На accepted2bd native matrix62/64 два invalid-left controls дали смещённый
первый пробел: GTK3 Entry `долженыбыть  `, Qt rich `долженыбы ть `.
Это six-letter LEFT control, не принятый five-letter pair target. На b59 тоже
есть этот ordering class. Ни равенство причин/частот, ни виновность pair
provenance пока не доказаны. Первый неудачный переход UNKNOWN.

## Варианты

- Увеличить delay/повторять ввод до PASS: 1/10, прячет ошибку.
- Убрать verifier/authority отказ: 1/10, нарушает canon и deletion safety.
- Frozen same-input trace и event-order RED через existing production owner:
  9/10, рекомендуется до кода.
- Новая queue/transport/controller для всех клиентов: 2/10 до причины.

## Investigation protocol до implementation

Сохранить original key timings,1.5s assertions, exact field/PID/focus/runtime.
Наблюдать raw clear, Reset/capabilities, callback serial/admission, pending
Space return, native forwarding и CommitText/real text. Пустое поле не доказывает
готовность IME. Разделить pre-key unknown readiness и wrong effect after ready.
Не restart пользовательское окно/глобальный IBus ради наблюдения.
В одном frozen frame сопоставить literal closing Space и next-letter effects.
Grouping по первому общему механизму, не отдельные literal phrases/clients.

После причинного trace выбрать >=2 реально применимых designs с оценками,
consequence analysis и rollback. RED должен управлять порядком production
reducer/adapter calls; sleep/retry или второй симулятор владельца недопустимы.
Repair сохраняет one gesture/one apply, owner/epoch revocation, lattice/rank,
verifier и существующие delivery routes; изменение route требует отдельного
обсуждения и всей затронутой physical acceptance.

## Proof/stop conditions

Source gates и exact-byte native fixed64; Kitty first pair after Enter и
existing browser controls. Все original FAIL сохранены; новые diagnostics
отдельно, не denominator подмена. Learning cleanup exact CAS, positive rows
сохранены, ambiguity останавливает зависимые cases. Latency/CPU/RSS измерять
на выбранном пути, не обещать из code size. Если после двух review passes
причина/совместимость не закрыты — REPLAN_REQUIRED, без weakened tests.
Этап2 разрешён пользователем 2026-10-09 («делай»). Изменения runtime допускаются
после причинного доказательства и consequence analysis; оно ещё не получено.

## Read-only investigation, 2026-10-09

Оба исходных SNAPSHOT.json проверены: `files=[]`. Сохранённый raw-clear
подтверждает empty client text/caret и RU mode; activation_ready остаётся
`UNKNOWN_UNLESS_MATCHED_BY_SAVED_TRACE`. Поэтому эти receipts не позволяют
назвать первый отказавший IME callback. Исходные62/64 и оба FAIL неизменны.

Отдельный trace `PENDING_QT_RICH_LIVE_TRACE.jsonl` из
`space-boundary-client-commit-physical-20261008/` содержит другой successful
prefixed scenario: и callback предшествует native Space, затем д callback.
Он показывает существующую смешанную delivery boundary, но не доказывает
причину двух first-word FAIL и не заменяет отсутствующий trace.

Перед новым прогоном исследуется безопасная feedback boundary. Existing
private-client driver уже перенаправляет все четыре usage paths в private
`/tmp/proof/usage/` и запускается через remote bwrap. Host inverse helpers
в private cache пока остаются immutable historical tools, не canonical
исполнителем нового proof. Shared live journal cleanup не считается безопасным:
две записи и snapshot checks не исключают append между check и replace.

## Разрешённый отдельный diagnostic2 — 2026-10-09

Пользователь явно разрешил один локальный прогон. Первый запуск остановился
до создания полей и control input: текущие source/engine/decoder были согласованы
в US. Пользователь самостоятельно выбрал RU; затем выполнены ровно два input
streams, по одному в новом GTK3 Entry и Qt rich. Повторов ввода не было.

Оба visible controls PASS: `должен ыбыть `, empty preedit. В каждом trace
13 press callbacks совпадают с physical keycode sequence; serial binding и
monotonic callback clocks находятся внутри наблюдаемого field interval.
Первый Space вернул native passthrough; до следующего `ы` уже получен
surrounding receipt с7 chars/cursor7. В этих двух successful streams порядок
границы сохранён. Trace не содержит привязки native field PID к client context;
activation readiness в момент raw-clear не становится доказанной автоматически.

Installed/loaded2bd, PID/starttick и RU tuple до/после совпадают; logging config
восстановлен, cleanup errors отсутствуют. Inverse/shared feedback cleanup,
installation/restart не исполнялись. Probe final review9/10, два прохода.
[Сводка с точными private receipts и hashes](evidence/2026-10-09-td133-local-diagnostic.json).

Сбой не воспроизведён: TD-133 остаётся OPEN/IN_PROGRESS, original62/64 и оба
FAIL сохранены. Fresh standalone fields, отсутствие old ActivateLayout/select
и opt-in logging отличаются от original matrix predecessor state. Ни причина,
ни runtime fix не доказаны; дополнительного local input это разрешение не даёт.
