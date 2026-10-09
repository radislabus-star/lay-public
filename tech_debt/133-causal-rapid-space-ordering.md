# TD-133 — Причина порядка первого Space в GTK/Qt

Status: DEFERRED_STAGE2_DISCUSSION. Priority: P0. Size: unknown до trace.
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
До обсуждения второго этапа production edits/install/restart здесь не выполняются.
