# TD-137 — Partial cleanup journal/counters и точное восстановление

Status: DEFERRED_STAGE2_DISCUSSION. Priority: P1. Stage: 2. Size: UNKNOWN.
Owner: existing feedback compiler/loader generation и owned physical harness.
Case binding — отдельный TD-130. Invariants: C02,C05,C08,C10.

## Доказанный корень и отдельный scope

Original owned_inverse_feedback.py138–144 сверяет journal/counts/feedback,
записывает counters, снова сверяет journal, затем записывает journal. Две
atomic replace не являются atomic transaction. Owning history
`docs/architecture/space-boundary-shift-2026-10-07.md:1671–1674` фиксирует
CAS отказ уже после counters write и отдельную recovery. Это доказанный
partial-result механизм, не гипотеза и не ошибка wrong pair selector.
Source helper:
`/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/owned_inverse_feedback.py`.
Точные historical receipts сохраняются; current frequency/crash rate UNKNOWN.
Поздняя успешная exact recovery не переименовывает исходный FAIL в PASS.

## Варианты и рекомендация

- Игнорировать CAS failure/путь receipt считать success: 1/10.
- Retry rollback counters поверх нового journal: 2/10, можно затереть user data.
- Existing loader generation contract плюс fail-closed stop и явная recovery:
  9/10 для investigation, рекомендуется до кода.
- Новый lock/writer service/transaction DB: 3/10 до доказательства необходимости.
- Изоляция test feedback в уже существующем private-client sandbox без model
  copies: условно8/10, требуется доказать real-window scope и runtime consumers.

## До изменения writes

1. Инвентаризировать original writers/readers, counts.source_len/schema15,
   rotation, native compiler generation и cache reload. PID/time не case ownership.
2. Freeze no-race и partial timelines: append до baseline check, между counts
   write и journal check, между последним check и replace, crash после каждого
   write, concurrent rotation и same-length generation changes.
3. Любой unknown/partial result блокирует dependent cases. Доказать, что именно
   journal/positive rows/user negative rows остаются сохранены, и какие counts
   обязан пересобрать existing loader. Manual recovery — отдельный measured
   maintenance step с exact owned episode/baseline, не retry-until-green.
4. Named versioned caller читает cleanup status (TD-130); recovery receipt
   содержит immutable before/after identities всех затронутых files. Сам
   existence receipt или повторное совпадение пары не создаёт authority.

## Proof и подводные камни

Failure injection только на isolated temporary fixture data/real compiler
contract remote, не на user journal. Проверять partial effects и preserved
positive/other-owner rows, не только return code. Runtime/learner semantics,
model packages и learning weights не меняются ради cleanup. Сохранить existing
1.5s bound и stop policy; новый mutex/timer/queue требует отдельного основания.
Источник безопасного recovery сначала измерить: file atomicity не proof пары.
Independent review<=2; незакрытый correctness finding оставляет task open.

DONE требует не только binding tests, но проверенного partial-state recovery
и consumer stop contract. До обсуждения этапа2 writes/runtime actions отсутствуют.
Откат — source tooling revert, не rollback пользовательских journal bytes.
