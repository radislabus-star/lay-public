# TD-136 — Semantic gaps и границы больших test files

Status: DONE_BOUNDED_MAP_AND_SOURCE_CHARACTERIZATION. Priority: P2. Depends: causal mechanism map.
Owner: existing manifest/discovery and production reducer/adapter tests.

## Корень и evidence distinction

3044 required PASS не означает line/branch/client coverage. Residuals13039 и
terminal_delivery4159 строк усложняют поиск scenario families, но splitting
не улучшает функциональное покрытие само по себе. Source .contains/hash guards
проверяют ownership/lineage и дополняют semantic tests; их нельзя выкидывать
как «рудиментарные», чтобы пройти rename/refactor.

## Варианты

- Порезать файлы/переименовать tests по size: 3/10, identity churn без качества.
- Behavior→owner→entrypoint→positive/negative/effect→physical map, затем только
  найденные gaps: 9/10, рекомендуется.
- Придумать coverage процент по test names/file count: 1/10.

## Minimal protocol

Перечислить Space, first-after-Enter, clean/invalid pair, Tab, held Shift,
eight DoubleShift intermediates/icon/decoder, undo, focus/Reset, cold/package
reload, stale reply, partial transport и feedback cleanup. Для каждой ячейки
найти actual production entrypoint и asserted effects; UNKNOWN оставить UNKNOWN.
Имена test или API dispatch не доказывают effect. Coverage instrumenting только
если оно отвечает конкретному вопросу; бюджет/refactor не тратить на dashboard.

TDD gaps по shared first-failure mechanism, not per literal example. Managed
event order без sleeps. RED на baseline или narrow controlled violation,
GREEN whole affected fixed proof. Если нужна file extraction — один связный
scenario family с сохранением identities/isolations; test-only helper не должен
имитировать вторую runtime state machine или владеть authority.

## Pitfalls/acceptance

Process isolation не заменяется faster shared process; test names/lanes/fixture
hashes обновляются только через declared discovery contract. Historical source
reviews не переписываются; new binding/successor видимый и независимый. Не
снимать failed class или игнорировать flaky output. Source and physical verdicts
раздельны, no generic PASS. Package/delta/caches/ranking untouched до отдельной
системной задачи. Per task review<=2, score>=8 плюс objectively passed scope.
Stage2 map/spec не объявляется implementation; новую матрицу обсуждаем отдельно.


## Bounded current map, 2026-10-09

[Reviewed effects and limitations](evidence/2026-10-09-td136-effect-map.md) ·
[24 exact manifest identities /13 matching source files](evidence/2026-10-09-td136-effect-map.json).
This is an audit, not task DONE or a coverage percentage. Source contracts
already cover both-order Space joins, first pair after Enter, exact Tab append,
held Shift across Reset, eight publications, stale owner, partial apply/inverse
and feedback. Original native ordering/focus cells and package-reload reader
semantics remain uncharacterized. No source or test-file split is chosen.

Candidate minimal gap is the joined caps41 second-field first-word path. The
Chrome-named existing test starts with caps9 then gains41; the caps41 test
feeds snapshots directly and checks a fresh field without proving the original
Tab transition or actual inbound SetSurroundingText. Inspect the complete
family before adding a duplicate; use real production callback helpers and
assert exact authorization/output plus stale/absent receipt negatives. Native
acceptance remains128 even if that source characterization passes. A new
ordinary native isolation protocol is a separate larger prerequisite; remote
preflight alone does not supply it. No new local input permission is inferred.

## Selected source gap after map review

The source map received ACCEPT8/10, one planning review pass:
[review](evidence/2026-10-09-td136-map-review-pass1.md).
It remains a pre-test baseline; changed test/manifest hashes are successors.
One new process-isolated test composes actual Tab, FocusOut/FocusInId to another
context, caps41 first-word commits and SetSurroundingText callbacks. Real
Backspace/retype proves equal-text ABA denial; a fresh receipt permits exact
Space geometry/output and a silent release. No production change is selected.

Remote GREEN characterization, narrowly declared private guard violation,
whole affected fixed proof, canonical discovery/graph/canon and independent
implementation review<=2 gate the bounded source result. Package reload and
native client/ordering cells stay UNKNOWN and their original tasks remain open.
[Owning result/spec](../docs/architecture/tech-debt-maintenance-2026-10-09.md#td-136--bounded-effect-map-and-one-joined-callback-characterization).


## Fixed proof successor and metadata repair

The automatic check remains FAIL/BLOCKED_CONTRACT: after every3045 body finished,
the known-failure ledger rejected the new manifest SHA. Its only successor
change is that fingerprint; the empty ledger and historical provenance remain.
[Separate guarded adjudication](evidence/2026-10-09-td136-fixed-proof-adjudication.json)
verifies278 logs/3045exact statuses and1176 unchanged relevant files, then admits
current metadata with the existing contracts. Verdict is body-proof reuse plus
metadata PASS, never an all-command rerun/PASS. Source review1 ACCEPT8/10;
review2 and final generated-artifact checks remain pending.


Final implementation [review2](evidence/2026-10-09-td136-code-review-pass2.md):
ACCEPT8/10, no blockers,2/2passes. This closes the implementation review;
final current generated-graph/embedded-receipt checks still gate source-only DONE.


## Bounded completion

[Final metadata receipts](evidence/2026-10-09-td136-finalization.json): canonical
remote graph/canon PASS,8 stable fetched artifacts; exact compiled architecture
receipt test1/1 PASS. Review8/10 in2implementation passes. The bounded map and
one demonstrated source callback gap are DONE. No file-size-driven extraction
is justified. New bodies3045/3045 and repaired metadata are separate from the
retained automatic FAIL; final documentation/artifact successors reuse those
exact behavioral source identities. Native128/133, package-reload/cold data
semantics and larger component splits keep their original open/stage2 scopes.
