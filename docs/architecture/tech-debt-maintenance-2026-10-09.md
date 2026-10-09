# TD-131 — единый bounded-tail invariant, 2026-10-09

Status: CODE_REVIEW_ACCEPTED_PUBLICATION_METADATA_PENDING; source-only.
Baseline: `2401a4e4dca5f99cc87c42415ab52c108c26bd74`.
Task: [TD-131](../../tech_debt/131-one-unicode-tail-limit.md).
Decision: [one-unicode-tail-limit](decisions/2026-10-09-one-unicode-tail-limit.json).
Contract: C01,C03,C05,C08,C10, existing preedit/tail_memory owners.

## Наблюдение и выбранная граница

Ordinary append вызывает preedit::trim_tail_buffer с PREEDIT_TAIL_LIMIT=160.
Composition commit вызывает частную копию trim_committed_tail_buffer с local160.
Оба сохраняют последние160 Unicode scalars, удаляя prefix по UTF-8 boundary.
Настоящий trim defect не наблюдался; duplicate invariant — подтверждённый долг.
WordLineage bounded offsets опираются на тот же предел. Существующее
revocation после trim/replace остаётся отдельным admission contract.

Оставить копии:4/10; открыть существующий fixed helper только sibling:9/10,
выбрано; новый BoundedTail object/owner:3/10 для этого случая. Generic token
limit остаётся private. Изменяется только helper visibility и один consumer
call; копия удаляется. Порядок invalidation, preedit reset/rebuild, trim,
suppression refresh/retirement и publish_tail_handoff сохраняется.

## Проверка и последствия до реализации

Усилить existing tail_buffer_stays_bounded, не добавляя test identity:
159/160/161/177 scalars, точный suffix ASCII/Cyrillic/emoji/mixed combining
UTF-8, ordinary append и composition commit с/без closingSpace. Test oracle
не использует проверяемый helper. Сначала current duplicate-source GREEN
characterization; это не RED/GREEN bug fix. Затем remote affected closure,
canonical graph/canon и independent fresh-context review<=2.

Новые candidate/rank/weights/model/package/learning semantics отсутствуют.
Нового owner, state field, timer, allocation algorithm, queue, fallback или
executor нет. Latency/allocation equivalence ожидается, не измерена. Unicode
normalization и grapheme geometry не добавляются: контракт в scalars. Все
outputs/DecisionCore/verifier/SafetyGate и IME delivery routes сохраняются.
Откат — один source revert; не пользовательские journal bytes и не установка.

Installed IME остаётся accepted1.0.81/2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559.
Runtime authority NOT_CHANGED; новые source bytes не установлены. Physical,
performance/RSS и all-window acceptance для refactor NOT_TESTED. Исторический
native62PASS/2FAIL/64 и original browser24/30FAIL не переименовываются в PASS.
Exact measured outcome/receipts добавляются после выполнения, до task DONE.

## Characterization и implementation

Remote current-production characterization:707/707 PASS; wrapper wall84.8s,
remote execution74.7456s;
`/home/ubu/.cache/lay/development/run-8is73yuu/RESULT.json`.
Это GREEN обеих прежних копий с усиленным existing test identity; RED defect
не заявлен. Expected suffix oracle использует reverse/take/reverse и fixed160,
а не production helper/constant. Composition cases также сохраняют prefix и
closingSpace. Тестовая матрица:4 Unicode patterns ×4 lengths × helper, ordinary
append, composition без/сSpace. Число test identities не выросло.

Production delta: existing fixed helper получил pub(super); единственный
composition consumer вызывает его на прежнем месте; duplicate/localLIMIT
удалены. Generic helper остался private; suppression/trim/handoff order не
менялся. Affected closure/graph/canon и review ещё pending.

## Pass1 и один grouped repair

Automatic remote affected contracts PASS:3044 selected correctness/package,
all-lane command0; wrapper wall428.4s, remote419.6954s.
Receipt: `/home/ubu/.cache/lay/development/run-i1rymxbv/RESULT.json`.
Graph/canon first PASS: `/home/ubu/.cache/lay/development/tab-graph-61t7f1qw/`,
8 outputs fetched with source stability. Эти факты не закрывают task сами.
Independent review pass1 REQUEST_CHANGES7/10:
[review](../../tech_debt/evidence/2026-10-09-td131-review-pass1.md).

Единственный blocking P2 — permanent guard gap: preedit.rs не входил в exact
PROTECTED; initial ADR сам по себе не защищает future one-file edits. Grouped
repair сохраняет прежнюю composition-trimmer protection: один path entry,
existing negative test с one-file/no-new-decision subcase и explicit decision
для scripts/check_architecture_canon.py/tests/test_architecture_canon.py.
Нового guard framework/import traversal/owner нет. Before guard edit — RED
existing Python canon route, затем GREEN и fresh graph/binding proof. При
неизменных behavioral Rust hashes исходный3044 proof сохраняет scope; новые
Python guard/compiled graph metadata требуют своих проверок, не quality claim.
No-prefix composition уже covered existing committed_space_keeps_next_word_separated_in_tail_memory;
новая bounded matrix с prefix не выдаётся за исчерпывающий lifecycle proof.

Guard RED на прежнем PROTECTED:15 tests,14PASS/1expectedFAIL, только subcase
preedit.rs без нового decision. Receipt:
`/home/ubu/.cache/lay/development/tab-guard-test-hxfq3jpm/RESULT.json`;
ошибка `protected_change_without_new_decision:…/preedit.rs not found in []`.
После этого добавлен ровно preedit.rs в existing exact PROTECTED. Test identity
test_protected_change_needs_new_decision сохранена; новые runner/guards нет.
Actual all-lane source log подтверждает3044 selected,0known semantic failures,
0infrastructure failures. Следующие проверки — GREEN15 и fresh graph/binding.

Guard GREEN:15/15 PASS,
`/home/ubu/.cache/lay/development/tab-guard-test-gx5pj6xl/RESULT.json`.
Current canon link/decision check также PASS на repaired checker.
[Proof reuse identity](../../tech_debt/evidence/2026-10-09-td131-proof-reuse.json)
сверяет relevant behavioral source/fixtures/Cargo/lane hashes и modes с3044
snapshot; mismatch0. Generated architecture receipt — отдельная metadata
dependency, для неё final refresh/check и exact compiled receipt identity.
Final review pass2 и завершение metadata проверок pending. Итоговый authoritative
packet — [final source evidence](../../tech_debt/evidence/2026-10-09-td131-final-source.json);
его PENDING не приравнивается к PASS или DONE.

## Итог реализации и последний publication checkpoint

Final independent [review pass2](../../tech_debt/evidence/2026-10-09-td131-review-pass2.md)
ACCEPT9/10, P2 closed, новых correctness findings нет. Всего два прохода,
один grouped repair. Reviewer независимо сверил все931 relevant files/modes,
source/core/test delta совпадает с проверенным3044 snapshot. Manifest, ledger,
quality/functional claims и installed bytes не менялись. Guard15/15 сохранил
existing identities и exact-path policy. Это source-only acceptance; full release,
installation/activation, новые physical/latency/RSS/heldout quality NOT_TESTED.

Current compiled receipt проверен отдельной existing identity
architecture_contract::tests::generated_receipt_proves_every_required_architecture_check:
1/1 PASS,0ignored,1837filtered;
`/home/ubu/.cache/lay/development/tab-compiled-receipt-test-n4ig2mkc/RESULT.json`.
Это только metadata identity, не ещё один denominator функционального качества.
Second canonical graph/canon PASS,8fetched members:
`/home/ubu/.cache/lay/development/tab-graph-q4t6db9d/RESULT.json`.

После этой owning entry выполняется canonical publication graph refresh.
Его exact receipt, hashes/freshness, необходимость нового compiled metadata
check и фактический verdict записываются в единственный
[final source packet](../../tech_debt/evidence/2026-10-09-td131-final-source.json).
Пока packet PENDING, task не получает DONE. Завершение metadata не является
новым architecture experiment и не меняет source semantics; эта entry и packet
входят в тот же task commit. Commit/push receipt создаётся отдельно после Git
operations, не объявляется заранее. Runtime authority NOT_CHANGED, accepted2bd
сохраняется, native62/64FAIL и original browser24/30FAIL остаются историей.
Rollback: source revert всего TD131 (call/visibility/removal/guard/test вместе).
