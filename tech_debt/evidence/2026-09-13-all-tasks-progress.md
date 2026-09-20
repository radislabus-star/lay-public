# Все задачи tech_debt: план, отчёты и публикация

## Current TD-121 acceptance — 2026-09-13

TD-121 is **IN PROGRESS / INSTALLED_VERIFIED_PHYSICAL_PENDING**. The focused
proof on the final runtime source passed 519/519 separately. C20 then passed all
13 release commands; changed and full gates each passed 2,807/2,807 with 11
intentional performance skips, compiled-receipt verification passed, and four
isolated final-byte client cells passed. Release `RESULT.json` SHA-256:
`02458047a539fb85be82b301fd1cf38b19af1f71241f34260713cf5c5dd90534`.

The owned GTK entry matrix passed 3/3: `привет` with one manual toggle, `ghbdtn`
with two toggles and zero queued key/space/boundary passthrough, and layout
projection `ghjdthrf` after autocomplete with one toggle. GTK receipt SHA-256:
`27bf83fccd5a15e552dabd9afeada8f1bacfd9038b8f9a0046f2cb3d0fcd1eaf`.
Release 1.0.72 is installed; ten installed artifacts and four loaded owners match
C20, and the loaded extension reports 1.0.72. Global IBus identity, configuration,
input sources, immutable models, journals and learner state were preserved.
Installation receipt SHA-256:
`ca7b0cb622f862cdb9a51678e640d27953fe798f3b37f291bdebce4f5e4735a4`.
Only human physical-keyboard confirmation remains for TD-121. C12 and C18 below
are retained as historical checkpoints; they do not own current release identity.
General TD-123 answer quality remains `UNKNOWN`.

Historical attempts and their exact receipts are retained only in
[evidence/td121-private-actual-baseline-2026-09-13.md](td121-private-actual-baseline-2026-09-13.md).

## Полное дерево

| Задача и её отчёт | Текущее состояние | Ревью и публикация |
|---|---|---|
| [TD-113: hybrid Nanda](../113-restore-hybrid-nanda-autocorrect.md) | Ранее принятый DONE в указанном в задаче объёме | Сохранён существующий результат; нового ревью или новой приёмки сейчас не заявлено |
| [TD-120: lifetime suppression](../120-scope-autocorrect-suppression-to-word-lifetime.md) | Ранее принятый DONE для ограниченного source/runtime контракта | Сохранён существующий результат; это не приёмка всего выпуска |
| [TD-124: maintenance tooling](../124-reproducible-maintenance-loop.md) | Ранее принятый DONE для инструментов сопровождения | Уже опубликованный tooling checkpoint 70824529; клиентская приёмка TD-121 отдельна |
| [TD-126: единый модуль взаимодействия с окном](td126-final-acceptance.md) | DONE для исходного кода | Реальное implementation review 7/10 → 8/10 ACCEPT; 2781/2781 PASS; commit/push 6baa6544 подтверждены |
| [TD-121: целое слово при IME handoff](../121-preserve-word-across-ime-layout-handoff.md) | **INSTALLED_VERIFIED_PHYSICAL_PENDING: C20 release and GTK PASS** | Focused 519/519; changed/full 2807/2807 each; four clients; GTK 3/3; installed/loaded 1.0.72; human physical pending |
| [TD-125: левая граница автозамены](../125-preserve-autocorrection-left-boundary.md) | Следующий после TD-121; IN_PROGRESS / physical pending | Новое итоговое ревью и закрытие в этом цикле ещё не выполнены |
| [TD-122: legacy replay request](../122-bind-legacy-replay-suppression-request.md) | Затем проверить необходимость и завершить binding | Исторический DECISION_REQUIRED не является DONE; новый полный цикл пользователем разрешён |
| [TD-123: качество Wave](../123-improve-wave-restoration-quality-for-1.0.67.md) | Затем общий механизм ошибок и весь фиксированный proof | OPEN; aggregate и каждый класс проверяются отдельно, новое quality PASS не заявлено |

Порядок оставшейся работы: **121 → 125 → 122 → 123**. После каждого закрытия
отчёт должен содержать фактическую оценку ревью, результаты проверок,
commit и подтверждение push. Нельзя заменить этот цикл подготовкой установки
или остановиться после отдельного успешного теста.

## История TD-121: C18/C19 до C20

TD-121 остаётся **IN_PROGRESS**. C18 canonical changed и full дали по
**2793/2793 PASS**, четыре actual-client cells имеют отдельные PASS receipts, но
C18 GTK synthetic-uinput остаётся **0/3 accepted**, а human physical keyboard —
**NOT_TESTED**. Поэтому прежний review C12 9/10 не принимает последующие C19/C20
bytes и не закрывает задачу.

Первый C19 механизм, typed same-context exact snapshot transfer, принят только в
focused scope: remote rustfmt PASS и шесть test functions, **16/16 concrete cases
PASS**. Immutable stage51 находится в
`/home/e/projects/lay-development-runner/td121-typed-snapshot-stage51-frozen`;
`source.tar` SHA-256
`73e15a091e08e3bd04b52c89ab3fd6304b6f85e99d2e1791449e81e67dc40131`,
1408-file manifest SHA-256
`763b84d8ff42969c800ce0b8c3a6847835dd54a6e169efeb43c3639735d510dc`.
Это не GTK, full release или runtime acceptance.

Второй causal RED воспроизводит отдельный C18 autocomplete defect через
production adapter. При `UnknownStart`, exact snapshot и stuck visible suffix
Alt-release успешно коммитит только suffix+space (`def `), owned tail становится
`abcdef `, но callback остаётся `UnknownStart`; post-effect assertion падает.
Remote test-only receipt:
`/home/e/projects/lay-development-runner/td121-completion-release-red52/57-causal-red.log`,
`rc=101`, SHA-256
`27e6e1c704f507a78a02de65de87daaf7f0deee8d99e01ff8ddee63a3e7da32b`.
После независимой проверки RED принят общий effect-based settlement fix. Focused
group65 прошёл семь test functions; отдельная A->B->A проверка подтверждает, что
старый captured target receipt не переживает возврат раскладки. Полный remote IME
target прошёл **515/515 PASS**, scoped clippy `rc=0`. Более новый surrounding
callback при atomic settlement сохраняет callback provenance и очищает
унаследованный receipt. Эти source checks не повышают installed runtime или GTK
authority; следующий gate — diff review, graph refresh и original GTK3 на новом
candidate.

## Исторический C18/C19 gate

Current TD-121 proof identity is owned by the C18 changed/full result, the C18
GTK failure and four C18 final-byte client receipts listed at the top. Earlier attempts,
failures and source archives
are retained in [the single historical evidence owner](td121-private-actual-baseline-2026-09-13.md).

Next, repair the typed same-context exact-manual snapshot transfer, pass its
grouped guarded proof, then repeat canonical changed/full and GTK acceptance on
fresh 1.0.72 bytes. Installation/process parity, task closure and publication
remain later gates.

## TD-121 C20 release and installation checkpoint

C20 completed the full 1.0.72 release proof: 13/13 commands; changed and full
gates each 2,807/2,807 with 11 intentional performance skips; compiled receipt, and four isolated final-byte client cells. The release result
SHA-256 is `02458047a539fb85be82b301fd1cf38b19af1f71241f34260713cf5c5dd90534`.
The preceding focused 519/519 proof on the same runtime source passed separately.
The subsequent owned GTK entry run passed 3/3 exact surfaces; its receipt is
`/home/ubu/.cache/lay/development/release-1.0.72-td121-c20-20260913/gui-smoke-owned-c20/RECEIPT.json`,
SHA-256 `27bf83fccd5a15e552dabd9afeada8f1bacfd9038b8f9a0046f2cb3d0fcd1eaf`.

The unchanged reviewed installer installed all ten C20 release artifacts and
verified four loaded owners plus extension version 1.0.72. Global IBus identity,
input sources, configuration, immutable models, journals and learner state were
preserved. Installation receipt:
`/home/ubu/.cache/lay/development/release-1.0.72-td121-c20-20260913/installation-1.0.72.json`,
SHA-256 `ca7b0cb622f862cdb9a51678e640d27953fe798f3b37f291bdebce4f5e4735a4`.
Runtime authority changed by installation. Human physical-keyboard confirmation
remains `PENDING`; no TD-123 quality promotion follows from these routing and
delivery denominators.
