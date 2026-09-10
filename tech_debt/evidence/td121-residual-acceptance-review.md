# TD-121 — bounded independent residual acceptance, 2026-09-07

Verdict: **ACCEPTED — 9/10, High0, Medium0**, limited to the four already
implemented pass-2 repairs and protected composition-source readiness.
Reviewer: fresh-context `td121_residual_acceptance_review`, GPT-5.6 Sol/high.
The user renewed continuation to1.0.66; the owning task records this explicit
bounded replan. This is not a third general rewrite/review cycle. No runtime
code was changed during revalidation. Prior3/10 and4/10 reports stay historical.

| Pass-2 finding | Independently checked closure |
|---|---|
| Bridge admits earlier unresolved ingress | Token issuance and consumption both require settled lifecycle and no unsettled keys; ordinary callback identity is unaffected. Tests cover both sides of the fence. |
| Native re-focus mistaken for enrichment | Enrichment requires the current settled lifecycle. Real re-focus uses existing transfer/source-free acquisition; same/different-context cases rotate activation correctly. |
| Revoked ready transfer restores old KnownStart | Ready grants bind connection, revocation, owner, activation and lineage, checked at take and install. The original grant token is retained. Reset/ContentType revocation cases cover both gaps. |
| Enter hides boundary from later Backspace | Backspace beyond an empty observed mirror revokes completeness. Enter/keypad Enter, Tab/manual refusal, feedback absence and next-boundary rearm are covered. |

The reviewer inspected source, archived source hashes and retained results;
no tests were run by the reviewer. Parent independently verified the seven
`context_admission::adapter::tests::word_scope::residuals::*` PASS rows and both
actual-Tab readiness rows in the final remote log. Nested module inclusion is
`adapter/tests.rs -> tests/word_scope.rs -> residuals.rs`; an initial reviewer
suspicion of an orphan module was disproved and withdrawn, not patched.

Authoritative retained run:
`/home/e/projects/lay-development-runner/run-BgJ1c1/tests/SUMMARY.json`,
SHA-256 `1e4c7615de11ef5925ce78ab2b6b5af117085f632ab0c8f0cef05b3227a454b4`.
Union440/440, including IME402/402. Historical residual7/7 and IME399/399 are
subsets/checkpoints, not additive denominators. Source archive/request:
`/home/ubu/.cache/lay/development/run-xu9f9sp6/` and its remote run.

Composition source `src/bin/lay_ibus_engine/composition_commit.rs` is unchanged
through this checkpoint: SHA-256
`9d94b7e75a50573a2c6686e38841e2f639cd87f1655643487d6468998a6c6694`, mode0664.
Exact verified functional test references:

- `context_runtime::tests::td121_unknown_start_actual_tab_refuses_active_composition_without_output_or_feedback`:
  original composition retained, no output, no pending/completed learning.
- `context_runtime::tests::td121_known_start_actual_tab_accepts_complete_word`:
  one full expected commit, full tracked tail, composition cleared, learning
  pending but not prematurely recorded.

The TD-113 composition-successor schema requires this source identity, passing
independent review>=8/H0/M0 and these functional references. All are met, so
the owning successor can move PROPOSED/PENDING -> ACCEPTED/PASS without changing
the gate. Predecessor TD-120 bytes/binding are untouched. This is source
checkpoint admission only, not new runtime ownership or installation authority.

Still OPEN independently: client0/5/cold `prefetch_not_ready`, correct client
capability contract, C/H final evidence aggregation, canonical/full release
checks, actual terminal/GTK and physical-key confirmation, artifact/install/push.
Do not report those as passed or relabel the old client failure.

## Final lint-only successor review — 2026-09-07

Independent scoped verdict:10/10,High0,Medium0,PASS. This supplements, not
relabels, the original9/10 runtime review above. The composition successor now
has SHA256 `33739ccb18d07fd7206f4b98e605f5e45ec5fe3041e6033f4f70190397f01d5f`.
Only a four-line function-local Clippy expectation was added; removing source
lines215-218 reproduces prior `9d94b7e75a50573a2c6686e38841e2f639cd87f1655643487d6468998a6c6694`.
No signature, executable text, authority or TD120 predecessor bytes changed.
Named KnownStart/UnknownStart actual-Tab tests passed again in final411/411:
remote `release-1066-final-IFSMv9/ime-lint-final-tests/SUMMARY.json`, SHA256
`c122443446e0618c6cdfefc6b1ad480c9254b268e850b5d0391f84ef461c9baa`.

Reviewer also checked Copy-clone removal, explicit test guard scopes at the
same pre-await boundaries, test-only noop waker, exhaustive private enum
renames, fixed wire-argument expectations, redundant reference removal and
the identical Option early return. Both final Clippy scopes PASS with zero
non-dead diagnostics; dead-code inventory counts unchanged535/359.
This is lint/source admission only. The current diagnostic client5/5 supersedes
the historical client0/5 status above only for its explicit post-ready route;
release-profile client and physical keyboard/GTK remain separate gates.
