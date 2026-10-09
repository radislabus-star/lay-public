# TD-137 — Изоляция test feedback и retirement небезопасной live-очистки

Status: DONE_RETIRED_UNSAFE_SHARED_CLEANUP. Priority: P1. Stage: 2.
Owner: existing private IBus proof consumer. Invariants: C02,C05,C08,C10.
Original shared-recovery scope superseded by the user's explicit selection
«Изоляция данных тестов (рекомендую)», 2026-10-09.

## Корень и выбранное решение

Historical `owned_inverse_feedback.py` сверяет journal/counts/feedback,
записывает counters, снова сверяет journal и заменяет journal. Две atomic
replace не являются транзакцией. Owning history
`docs/architecture/space-boundary-shift-2026-10-07.md:1671–1674` уже фиксирует
отказ после counters write. Controlled experiment ниже также доказал потерю
параллельного append при PASS. Exact recovery одного historical episode не
устанавливает безопасность живой очистки или частоту ошибок.

Варианты: ещё CAS/повторы2/10; координация writer/readers/cleanup5/10;
существующая private isolation и прекращение live-cleanup9/10. Пользователь
выбрал последний вариант. Writer protocol, loader generation, learning weights,
runtime owners и IME delivery routes в этой задаче не меняются.

[Decision](../docs/architecture/decisions/2026-10-09-retire-shared-inverse-cleanup.json)
задаёт новую границу DONE: retirement поддерживаемого destructive test path,
а не исправленная shared recovery. Общая live recovery остаётся NOT_TESTED.

## Единственный поддерживаемый inverse entrypoint

`scripts/proof/ime-client/run.py --remote-worker` → V3 `driver.py`, под
existing remote resource guard, с exact `--config`, fresh `--output` и opt-in
`--scenario-set inverse-first-word` или `inverse-prefixed`. `dev-check.py client`
запускает default five cases и не передаёт scenario-set. Existing inverse runner
до запуска проверяет distinct remote machine, ресурсный допуск, точные assets
и fresh output; existing sandbox задаёт private bus/home. Driver перед factory
устанавливает все четыре existing usage paths в `/tmp/proof/usage/`.

Один apply/inverse на fresh sandbox, read-only case-bound observation; missing,
ambiguous или partial evidence дают nonzero в существующем1.5s bound. У case
нет dependent successor и полномочий удалять journal/counters. Этот connected
контракт уже доказан TD-130; exact unchanged source identities и reused receipts
перечислены в [retirement packet](evidence/2026-10-09-td137-retirement.json).
Новые runner, timer, mutex, transport, model copies или runtime build не нужны.

## Retired historical callers и stop boundary

В private historical root
`/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/`
сохраняются helper, `windows/space_boundary_matrix80.py`, четыре launcher:
`run_private_terminal_matrix.py`, `run_private_writer_matrix.py`,
`run_tor_matrix.py`, `run_tor_connected_only.py`, и соответствующие snapshots.
Bounded inventory с path/size/SHA находится в retirement packet. Repository
proof scripts и development entrypoint не dispatch эти helper/flags.

Все эти shared-learning inverse/cleanup paths — forensic-only: не запускать
для нового proof. Native/physical inverse с shared learning имеет verdict
BLOCKED до ввода текста; снятие cleanup flag не даёт безопасного допуска.
Headless private PASS не выпускает native/physical gate. Для будущего native
successor нужно доказать изоляцию именно engine process и actual usage paths,
связь с real client context и отдельно разрешённый запуск.

Исторические bytes, права, pinned callers и receipts не изменяются. Правило
ограничивает поддерживаемый маршрут, не запрещает ручное исполнение файлов
средствами ОС и не доказывает отсутствие иных scripts на всей машине.

## Измеренная characterization, 2026-10-09

[Exact packet](evidence/2026-10-09-td137-characterization.json).
Baseline fce8978b; immutable helper8875 bytes, SHA
`f4fa75e429b9f7c431f3c72aa127d3b0a61866e67ca602bf23cfd241f418f1ec`;
existing native compiler7482296 bytes, SHA
`4032e774ded9e2dc10aca01fa85f5838b22f55a817e7ac27f27455ef1e94f468`.

Семь temporary synthetic cases: no-race PASS; append до первого CAS — refusal;
append после counts — refusal/partial generation; append после последнего CAS
до journal replace — FALSE PASS с потерей injected positive row; crash после
counts — partial/no receipt; crash после journal — consistent files/no receipt;
same-length rewrite — helper refusal при совпадающем source_len и неверной
counter semantics. Original outcomes и SHA сохранены до последующего rebuild.
Все other-owner rows и static feedback сохранены. Это fixture effects, не
field/IME ownership, production loss rate или live writer exclusion.

После прекращения инъекций native compiler пересобрал counts во всех семи
temporary directories:7/7 совпадают с surviving-journal reference, journal и
static feedback byte-identical. Потерянный append не восстановлен. Hot cache,
live recovery и runtime loader execution не проверялись. Length-only loader
contract установлен по source. Failed pilot остановился до cleanup из-за
отсутствующего DEVNULL в temporary subprocess facade; это tooling failure,
не semantic RED или runtime defect. Historical originals не изменены.

## Приёмка и последствия

- [x] Пользователь явно выбрал isolation/retirement вместо shared writer protocol.
- [x] Supported entrypoint и все четыре private usage paths установлены.
- [x] Bounded exact caller inventory; historical tools/receipts неизменны.
- [x] Connected stop/feedback effects TD-130 reused только на matching bytes.
- [x] Native shared inverse BLOCKED; отсутствие OS ban и native parity явно указано.
- [x] Final independent review ACCEPT9/10,2passes; canonical remote graph/check PASS,8 stable exports; compiled metadata exact1/1 PASS.
- [x] Scoped DONE_RETIRED_UNSAFE_SHARED_CLEANUP; publication receipt records commit/push and both exact refs.

DONE этой карточки не закрывает TD-133 ordering, GTK/Qt acceptance, general
shared/live recovery или future native isolation. Installed2bd, user learning
data/config/models и IME delivery routes не меняются. Rollback — новый decision
и revert supported tooling contract; не запись старых bytes поверх user journal.
