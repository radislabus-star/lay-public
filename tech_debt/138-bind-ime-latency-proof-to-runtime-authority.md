# TD-138 — Привязать latency proof к настоящей authority IME

Status: DONE_SOURCE_SCOPE. Priority: P0, prerequisite для нового
release gate TD-133. Owner: existing HotFieldPolicy + candidate_gate tests.
Scope: existing repeated-letter operator applicability + IME proof fixtures; no model/rank/authority change. C01,C02,C07,C10.

## Наблюдаемый отказ

Полный source gate TD-133 проверил3047/3047 функциональных тестов, затем
performance11:10PASS/1FAIL. `unique_prefix_cache_misses_stay_under_hot_readout_budget`
измерил298466us против50000us. Exact baseline24b в fresh sandbox тоже FAIL:
303677us/50000us. Original receipts не перезаписываются и не становятся PASS.
[Сводка и hashes](evidence/2026-10-10-td133-managed-space-source-proof.json).

Opt-in trace candidate: lexical298096us, canonical0/layout70/boundary2us;
total299092us. Это localization до lexical material, не доказанный конкретный
дорогой subcall. Частоту live-сбоя и native latency этот trace не измеряет.

## Возможный корень, который требуется проверить до правки

`src/bin/lay_ibus_engine.rs:84` выбирает `HotFieldPolicy::ime()` до создания
factory/warmup. Это `Ime/FieldSnapshotOnly`. Обе latency fixtures в
`src/nanda_wave/candidate_gate.rs` вызывают warmup/readout без выбора policy.
В новом тестовом процессе default — `Daemon/FullReferenceAllowed`, то есть
другая authority и возможность холодных reference dependencies.

Доказано различие setup по source. Ещё НЕ доказано, что только оно объясняет
298ms: для этого нужен controlled contrast на frozen baseline, где меняется
только setup двух benchmark fixtures до warmup. Все prefix inputs, budget,
requests, cache-miss sequence, sandbox и output assertions сохраняются.
Если proper IME scope тоже FAIL — никакой benchmark migration; искать первый
дорогой lexical subcall по тому же существующему trace.

## Варианты, оценка и выбор

1. Привязать две IME latency fixtures к production `HotFieldPolicy::ime()`
   до warmup; сохранить cold/full-reference FAIL как отдельный historical
   denominator. 8/10, рекомендуется только после controlled contrast и
   независимого review scope. Это исправляет measurement context, не runtime.
2. Оптимизировать или предварительно греть весь full-reference путь. 4/10
   сейчас: изменяет cold/runtime resource contract; причина пока не localized
   ниже lexical stage. Нельзя превращать cold corpus в live authority.
3. Повысить latency budget, удалить префикс, прогреть именно failing prefix,
   срезать lattice/repair или повторять до PASS. 1/10, недопустимо: скрывает
   измеренный отказ и может уничтожить evidence/target retention.

## Минимальный порядок

- Frozen baseline contrast: два существующих exact test identities, fresh
  sandbox на distinct worker под canonical guards; только policy setup меняется
  до существующего warmup. Никакого cached-user-data/model fitting/local input.
- Сохранить old/new authority tuples, source diff/hash, все timings и эффекты.
  Это diagnostic, не новый full PASS и не install admission.
- При подтверждении оформить отдельный benchmark-contract decision. Не менять
  production HotFieldPolicy defaults, candidate ranking, generation, cache keys,
  package/model bytes, transport или budget. Runtime TD-133 bytes сохраняются.
- В actual fixtures явно проверять выбранную authority до warmup. Не допускать
  скрытого возвращения FullReferenceAllowed и не добавлять hot-prefix literals.
  Семантические и per-class quality доказательства не заменять latency test.
- Remote canonical format/graph/full source+11performance. No ignored tests,
  allowlist additions или skipped failure. Fresh-context read-only review,
  score1–10, максимум2passes; при существенном нерешённом scope — REPLAN_REQUIRED.
- DONE только в proof-fixture scope после всех gates; затем commit/push как
  отдельную задачу. Это не закрывает native64, оригинальную причинность и133.

## Подводные камни и откат

Test policy в cfg(test) thread-local; запуск warmup на другом thread требует
отдельной проверки соответствия production process policy. Поэтому contrast
должен использовать существующий synchronous warmup/test entrypoint, а не
новый background simulator. Настройка после warmup опоздает и не годится.
Changing authority changes available evidence: migration разрешена только
если benchmark действительно владеет IME route. Не объявлять full-reference
route быстрым или покрытым этим proof. Existing reference correctness coverage
и оба historical latency FAIL сохраняются. Source tests не подтверждают native
видимый текст, Wayland aggregation, real-input latency или RSS новой82.

Rollback — вернуть fixture setup и benchmark decision; никаких journal/model
восстановлений или записей в пользовательскую learning history.

## Actual contrast и пересмотр — 2026-10-10

[Controlled result](evidence/2026-10-10-td138-ime-policy-contrast.json):
proper IME policy before warmup,2selected/1PASS/1FAIL. Unique-prefix591846us
against unchanged50000us; six unchanged requests each returned12candidates,
no empty-readout explanation. Prefix-cache hits0/misses7 including warmup.
Baseline/source archives unchanged; only two fixture setup insertions in a
worker-only copy. Actual checkout benchmark fixtures remain byte-identical.

[Fresh-context pass1](evidence/2026-10-10-td138-scope-review-pass1.md) rates
conditional diagnostic plan9/10 and requires REPLAN. IME measurement scope
is legitimate, but policy-only repair is falsified and cannot unblock release.
Both original budgets/fixtures and full command FAIL remain. Native/quality
results are not inferred from this contrast. No third review round is planned.

Selected next diagnostic9/10: split existing lexical material timing into word
readout, single-deletion repair and evidence-binding costs in a frozen worker
copy. Same material calls/merge order, exact two existing benchmark identities,
inputs, fresh sandbox and budgets; generic counters/timestamps only. Eager full
reference warmup2/10 or targeted-prefix warmup1/10 remains rejected without a
causal subcall and actual authority/resource boundary. The observer has overhead
and cannot itself supply a full PASS. Require ready warmup/nonzero work for
any future performance claim. Finalcode/fullproof receives pass2 only after
root cause is localized and a minimally sufficient repair is selected.

## Root cause and minimal repair — 2026-10-10

[Substage receipts](evidence/2026-10-10-td138-cold-repeated-operator-diagnostic.json)
localize first single-deletion call: repeated-letter581798us, exact-deletion
lookups613us. Subsequent repeated-letter calls14/16us. The repeated-letter
selector loads exact full-reference membership before checking whether a
repeated run exists. Exact reference membership explicitly calls the full
dictionary even under IME policy; this is candidate material, not promotion
authority. The failing four-character token has no repeated run, so the
expensive selector returns None after the cold load.

Selected minimal remedy9/10: invoke the existing pure repeated-run candidate
generator first; if it returns empty, return None before any lexical lookup.
Reuse the same generated candidates after unchanged protections, short-function
rule, length/vowel guards and scoring. Nonempty-run behavior, candidate contents,
ordering/dedup, authority and memoization remain the existing contract.
IME-only caller guard6/10 leaves other users paying an inapplicable operation;
eager full-reference warmup3/10 adds cold resource/authority exposure and is
not selected. No fixture prefix becomes a runtime condition.

TDD: one new process-isolated correctness test checks no-repeat Cyrillic
inputs of several lengths and both cases, both Autocorrect and ProposalOnly,
None surfaces and reference remaining cold. The isolation declaration prevents
an earlier test's global OnceLock from making the assertion vacuous. Run RED
before the selector edit, then GREEN and existing library/source/performance
proofs. Preserve all old FAILs. Existing two latency fixtures get explicit
production IME policy before synchronous warmup, successful ready checks and
nonempty output checks; inputs, timers, budgets and candidate assertions stay.
The old full-reference benchmark also receives a separate unchanged-fixture
contrast after the systemic repair, never relabeled as covered by IME proof.
No new cache, timer, warmup prefix, rank weight, candidate veto or model.

The pure no-op optimization changes no surface/verdict/proof on any nonempty
operator geometry. Full fixed correctness/package corpus and the strict
dense-prefix deletion/repair evidence tests remain required. Native and RSS
acceptance for new bytes remain NOT_TESTED; source PASS cannot close TD133.
Final read-only code/fullproof review is pass2/2, with no extra review loop.

TDD RED observed: focused library1818selected/1817PASS/1FAIL in fresh isolated
process for the new no-repeat cold-reference assertion. Initial attempt failed
only formatting before execution and is retained separately. Production repair
now implements geometry-first, reused candidates; both IME fixtures assert
actual policy, successful ready warmup and nonempty timed readout. No timer/input/
limit/budget/semantic assertion changed; no model or authority promotion.
GREEN, unchanged-full-reference contrast and fresh full gate pending.

Controlled GREEN [receipt](evidence/2026-10-10-td138-systemic-green.json):
IME3/3 and old full-reference fixtures3/3 exact proofs PASS, no failures. Both
unique-prefix requests return12candidates each with0hits/7misses. IME max6958us,
full-reference max6938us, unchanged50000us debug limit. The former failing
request now6425/6436us. The full-reference variant restores both benchmark
functions byte-for-byte to baseline candidate_gate SHA3c76238a… while retaining
the same production repair/new no-op test; proof-scope migration alone cannot
explain its success. Startup warmup remains a separate excluded timing row
(~201ms IME/~7.66s reference) and is not a first-key latency claim.
Manifest refreshed3074identities/3048required; zero known failures and original
observation bytes unchanged. Graph/new full gate/final review pending.

Remote AST graph refresh PASS,8 stable exports:
`/home/ubu/.cache/lay/development/tab-graph-z4fes24f/RESULT.json`.
Before fresh full gate, guarded inventory9,305,727,510bytes of12GiB; removed
only disposable worker target/debug2,346,528,775bytes under the existing heavy
lease, leaving6,959,198,735bytes. Original sources/models/install/evidence,
release and test-lanes caches preserved. Cache receipt is
`/home/ubu/.cache/lay/development/td138-cache-headroom-20261010.json`.
Fresh canonical full source+11performance is now selected, not yet a PASS.

## Closure in source scope — 2026-10-10

[Complete canonical receipt](evidence/2026-10-10-td138-complete-source-proof.json):
3048/3048 correctness+package and11/11 serialized performance PASS, no failures,
33required targets/38manifest targets. Default/research lint530/354, zero
nondead diagnostics; format/canon, desktop syntax/Firefox, CLI/release and
gitdiff PASS. Worker target8,181,723,136bytes remains below12GiB. Complete
archive4c53a2e… stable; no threshold, test identity or failure allowlist changed.

[Final fresh-context code review2/2](evidence/2026-10-10-td138-final-code-review-pass2.md)
is9/10 with no material finding. Geometry allocation for real repeated runs
still occurs before protection checks; reference loading remains reachable
for applicable operators. Startup first-key latency, RSS and physical latency
are NOT_TESTED. This is the recorded resource boundary, not a broad optimization
claim. All RED and failed contrasts remain immutable. No third review pass.

TD138 is DONE in operator-correctness and proof-fixture/source-release scope.
Candidate1.0.82 SHA5fe100db… is uninstalled; TD133 remains open for changed-byte
native acceptance. Source publication must include the frozen TD133 prerequisites
referenced by this manifest; do not publish a standalone manifest that lists
missing callback tests. No82tag or accepted release is granted by this closure.
