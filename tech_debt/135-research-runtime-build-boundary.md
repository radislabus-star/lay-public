# TD-135 — Граница research/runtime по реальной reachability

Status: ACTIVE_SCOPED_BUILD under the user's six-release order2026-10-25.
Priority: P3. Size: bounded warning/compiler/proof cleanup for1.0.101.
Owner: nanda_wave existing package/runtime/compiler contracts.
Quality owner: authoritative syntax-agreement roadmap; никакого нового fit.

2026-10-26: the explicit stage4 implementation order admits the measured
build-boundary slice, not a new quality fit or crate-wide redesign. Current
owner/probes: docs/architecture/research-build-boundaries-2026-10-26.md.
All three configurations compile; full test/build/native/publishing pending.
The historical discussion prerequisite below is retained as history and does
not revoke this later scoped user authorization.

## Факты, гипотеза и варианты

Eval частично gated research-tools/test. L2 compiler/proof/orchestrator и
public re-exports смешаны с default runtime surface. Наличие файла/экспорта
не доказывает executable bloat, RSS или dead code. Сначала измерить default
vs research-tools build graph и actual callers, с MSRV1.88/locked features.

- Удалить всё с именем legacy/proof/compiler: 1/10, типы и package compat могут быть нужны.
- Feature-gate доказанно отсутствующие default-runtime consumers, один boundary:
  8/10 условно, рекомендуется после baseline.
- Crate/workspace split всех wave modules: 3/10 сейчас, большой API и build-cache риск.

## Protocol и риски

Remote-only inventory Cargo targets/features, compile/link artifact bytes и
runtime API reachability. Проверить public CLI training/package commands,
research-tools, lexical-compiler, test cfg consumers, installed model schemas,
reload/delta/online learner. Не убрать candidate retention, contradictory
evidence или unique top-1 gates под видом старого кода. Source-only compiler
использование может быть нужно для packages/cleanup tooling.

Выбрать smallest gating patch лишь если измерены польза и backward policy.
No new package schema/cache owner/runtime fallback. TDD feature matrix и
package round-trip вместо только compile-green. Качество L1 требует весь
fixed proof, каждый damage class unique top-1>95%, clean/lattice/false certainty,
package/RSS/latency conjunctively; format parity не quality proof.
Rollback возвращает feature/public API, immutable models не заменяются.
Independent review<=2 passes, full release+physical нужны до live promotion.
До отдельного обсуждения этапа2 никаких compiler/gating edits и fit.
