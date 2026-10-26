# Retire authenticated old-word revocations without losing received focus

Status: ACTIVE_FULL. Priority: P1. Release group:1.0.102.
Owner: existing WindowInteraction / ContextAdmission. No agents or Harness2.

First failure and alternatives are in the owning architecture document:
[authenticated revocation retirement](../docs/architecture/authenticated-revocation-retirement-2026-10-26.md).
Two actual adverse receive/callback schedules fail on101 and pass with the
same two-branch local retirement repair. Both experiment copies are restored;
this task has not changed installed runtime authority.

Implement with existing strip_legacy_word_preedit_mirror and
retire_context_word_locally only after revocation_matches. All unobserved,
foreign and malformed paths keep strict revocation. Preserve every old test,
add meaningful negative/supersession checks, run full gates and short native
checks on exact loaded bytes, then publish and verify installed updater.
Do not mark unrelated quality, V1 protocol or native residual tasks DONE.

Implementation and all12 focused revocation controls PASS in CROOT/release102-full-gates/revocation-controls.log. The full fixed manifest adds exactly3 correctness tests and keeps every prior identity/lane; full3522/performance11/build/native/CI remain pending.


CandidateA32bb94f2 rejected before release: early strip skipped factory-mode
cleanup. Old/new paired control confirms oldPASS/currentFAIL. CandidateB uses
existing local activation discard before normal cleanup, strengthens both
positive semantic tests without changing identities, and reruns full proof.
Do not mark DONE from A CI or its full matrix. See owning document.
