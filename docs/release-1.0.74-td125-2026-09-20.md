# Lay 1.0.74 TD-125 release evidence

Status: `PUBLISHED_VERIFIED` on 2026-09-21.

## Scope

This patch release publishes the already installed TD-125 repair for legacy
browser autocorrection. Chrome and Firefox had accepted the replacement commit
without applying the preceding deletion, producing duplicated text. The final
route uses whole-word preedit ownership, retires canceled mirrors, and revokes
stale context admission before callback settlement. GTK and terminal routes
remain separate.

No candidate-generation, lattice-ranking, `SafetyGate`, edit-plan validation,
model package, learning, cache or deadline policy changes in this release.

## Publication record

- Exact release source snapshot SHA-256:
  `c318a5d64a5362ece1082a54379b157f04c8650caedb655333e6422660627be3`.
- Architecture wrapper: PASS, all 11 contracts.
- Correctness/package release gate: 2,884 selected, zero semantic failures and
  zero infrastructure failures.
- Ten release binaries report 1.0.74 where supported and were installed with
  source, installed and loaded hashes matched.
- Global IBus and the user's existing Firefox and Chrome processes were
  preserved during installation.
- The functional TD-125 code is the already accepted commit `14383e40`; the
  release commit changes its version and publication metadata.
- Public release commit: `ecd5af2a`; immutable annotated tag: `v1.0.74`;
  GitHub Release:
  <https://github.com/radislabus-star/lay-public/releases/tag/v1.0.74>.

## Pre-release evidence inherited from the accepted mechanism

The causal cursor-zero admission regression was RED at 582/583 and GREEN at
583/583 after fail-closed revocation. Independent review scored the final repair
9/10 PASS with no severity finding. The installed 1.0.73-post-tag candidate then
passed 2,884/2,884 fixed correctness/package tests and the four-client matrix.
Those receipts remain mechanism evidence; the 1.0.74 versioned source must pass
its own release and installation gates before publication.

Owning TD-125 record:
[preserve autocorrection boundary](../tech_debt/125-preserve-autocorrection-left-boundary.md).
