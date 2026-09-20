# Lay 1.0.74 TD-125 release evidence

Status: `RELEASE_CANDIDATE`; publication forbidden until every gate below is
recorded as passing.

## Scope

This patch release publishes the already installed TD-125 repair for legacy
browser autocorrection. Chrome and Firefox had accepted the replacement commit
without applying the preceding deletion, producing duplicated text. The final
route uses whole-word preedit ownership, retires canceled mirrors, and revokes
stale context admission before callback settlement. GTK and terminal routes
remain separate.

No candidate-generation, lattice-ranking, `SafetyGate`, edit-plan validation,
model package, learning, cache or deadline policy changes in this release.

## Required release proof

- exact immutable 1.0.74 source snapshot;
- architecture wrapper PASS with its receipt and graph bound to that source;
- complete correctness/package release gate with zero semantic and
  infrastructure failures;
- all ten release binaries built from the exact snapshot and reporting the
  expected version where supported;
- atomic installation with rollback bytes and global IBus preservation;
- exact `просто которое ` in GTK, existing Chrome, isolated Firefox with the
  compatibility adapter mapped, and isolated Kitty;
- public `main`, annotated `v1.0.74` tag and GitHub Release read back to the
  same release commit.

## Pre-release evidence inherited from the accepted mechanism

The causal cursor-zero admission regression was RED at 582/583 and GREEN at
583/583 after fail-closed revocation. Independent review scored the final repair
9/10 PASS with no severity finding. The installed 1.0.73-post-tag candidate then
passed 2,884/2,884 fixed correctness/package tests and the four-client matrix.
Those receipts remain mechanism evidence; the 1.0.74 versioned source must pass
its own release and installation gates before publication.

Owning TD-125 record:
[preserve autocorrection boundary](../tech_debt/125-preserve-autocorrection-left-boundary.md).
