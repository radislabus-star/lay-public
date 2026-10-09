# TD-131 — Один Unicode bounded-tail invariant

Status: DONE_SOURCE_ONLY. Priority: P1. Stage: 1. Size: S. Depends: TD-129 (DONE).
Owner: существующий preedit tail helper и tail_memory consumer.
Invariants: C01, C03, C05, C08, C10. Baseline: 2401a4e4dca5f99cc87c42415ab52c108c26bd74.
Owning architecture entry:
`docs/architecture/tech-debt-maintenance-2026-10-09.md`.
До production изменения создать новый explicit decision
`docs/architecture/decisions/2026-10-09-one-unicode-tail-limit.json` для
protected tail_memory.rs, preedit.rs и existing preedit test;
если понадобится manifest/ledger delta, перечислить их тоже. Guards не ослаблять.
Implementation pass1 выявил permanent guard gap существующего helper file.
Один grouped repair включает existing exact PROTECTED preedit.rs entry и
negative subcase в tests/test_architecture_canon.py; current decision дополнен
обоими guard/test paths до их правок. Это сохранение прежней composition
protection, не новый guard framework. Existing Python identity не меняется.

## Корень

`preedit.rs::trim_tail_buffer` использует PREEDIT_TAIL_LIMIT=160 и общий
trim_tail_buffer_to. `tail_memory.rs::trim_committed_tail_buffer` повторяет
scalar-count/drain и свою константу160. `WordLineage` u8 offsets и наблюдение
trim ссылаются на первый лимит. Доказан duplicate invariant, текущего неверного
обрезания не наблюдали. Reproduction defect не выдумывать.

## Варианты

- Оставить копии с комментарием: 4/10, будущий drift остаётся.
- Переиспользовать существующий fixed-tail helper через pub(super): 9/10,
  рекомендуется; generic token limit и ordering не изменяются.
- Новый BoundedTail object/owner и массовая замена String: 3/10 сейчас.

## Минимальная правка

Открыть только existing fixed trim_tail_buffer для sibling module; в
sync_tail_after_composition_commit вызвать его, удалить trim_committed_tail_buffer
и local LIMIT. Generic trim_tail_buffer_to и PREEDIT_TOKEN_LIMIT не расширять.
Сохранить порядок invalidating surrounding snapshot, completion learning,
preedit reset/rebuild, trimming и publish_tail_handoff. Trim не заменяет
word-range invalidation и не создаёт KnownStart. Никаких новых state fields,
packages, Unicode normalization или лимита больше160.

## TDD/characterization

Сначала выяснить existing semantic coverage. Characterization на current
production entrypoints для 159/160/161+ scalars, ASCII/Cyrillic/emoji/mixed UTF-8,
composition commit и обычного append. Проверять exact retained suffix, <=160,
существующие tail epoch/suppression и first-word trim revocation contracts.
Существующий correct source может быть GREEN: это extraction, не bug claim.
При необходимости один controlled mutation лимита в одном consumer доказывает,
что characterization замечает divergence; mutation не становится runtime source.
После правки — IME+все integration consumers через dev-check, graph refresh/canon
remote. New test identities проходят existing discovery, не ручное ослабление
manifest/known-failure contract. Full release обязателен только для будущей
installation/release; source task не заявляет physical acceptance.

## Последствия

Candidate lattice/rank/verifier, learning, cache/package identity сохраняются.
Latency/allocation алгоритм ожидается эквивалентным; новой latency/RSS метрики
не заявляется. Нет нового owner/fallback/timer. Изменение
связности: tail_memory зависит от уже существующего sibling helper; removal
cost меньше копии. Важные риски: байты вместо scalars, trim раньше suppression
refresh, zero/default limit из generic helper. Fixed160 consumer не принимает
новый произвольный лимит. Future packages не меняют tail geometry.
Откат — вернуть копию и visibility в одном revert; installed2bd bytes остаются.

## DONE

- [x] Один fixed160 helper, сохранены Unicode suffix и callback ordering.
- [x] До кода новый explicit decision; outcome в owning architecture document.
- [x] Source/affected tests и graph/canon remote PASS; exact receipts сохранены.
- [x] Fresh-context review >=8/10, максимум два прохода.
- [x] Source-only DONE, runtime/physical NOT_CHANGED/NOT_TESTED; publication completion фиксируется отдельным post-Git receipt.

## Outcome — 2026-10-09

Один existing fixed160 helper, generic helper private; копия/localLIMIT удалены.
Characterization707/707 GREEN до dedup. After-dedup automatic3044 selected
strictPASS,0known/infra; existing identity assertions strengthened, manifest и
ledger unchanged. Guard15/15 GREEN после causal14/15 RED old checker; protected
helper coverage сохранена. Review7/10→9/10, ровно два passes/один grouped repair.
[Final review](evidence/2026-10-09-td131-review-pass2.md).
[Authoritative source packet](evidence/2026-10-09-td131-final-source.json)
содержит exact receipts/hashes, finalgraph/canon8outputsPASS и current compiled
receipt1/1PASS. 931 relevant implementation/fixtures/Cargo/lane files/modes
совпадают с3044 snapshot; Python guard и generated metadata checked separately.
Счётчики этих разных proof scopes не складываются в общий quality denominator.

Runtime authority/installed2bd/delivery routes/models/rank/learning NOT_CHANGED.
Physical/full release/latencyRSS/newheldout quality NOT_TESTED. Historical
native2FAIL и browseroriginalFAIL сохранены. Source-only DONE не разрешает
installation и не переносит prior physical acceptance на будущие bytes.
Commit/push next; exact post-Git checkpoint будет в
`/home/ubu/.cache/lay/development/td131-publication-20261009.json`.
Rollback — один task revert, включая helper/consumer/guard/regressions.
