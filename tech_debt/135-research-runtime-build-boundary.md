# TD-135 — Граница research/runtime по реальной reachability

Status: DEFERRED_STAGE2_DISCUSSION. Priority: P3. Size: неизвестен.
Owner: nanda_wave existing package/runtime/compiler contracts.
Quality owner: authoritative syntax-agreement roadmap; никакого нового fit.

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
