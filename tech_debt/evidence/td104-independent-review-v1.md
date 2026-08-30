# TD-104 Independent Review V1

Reviewer: fresh-context read-only subagent `Ohm`

Score: `4/10 REVISE`

## Findings

1. The architecture receipt still carried the pre-change source fingerprint.
2. `install.sh` built featureless `--bins`, which skipped newly feature-owned
   binaries required later by the same installer.
3. The self-teacher source fallback invoked evaluator/trainer targets without
   `research-tools`.
4. The full test/lint route enabled only the research superset and did not
   separately prove the default configuration.
5. The zero-failure ledger was rebound to the new test manifest while retaining
   an older admission-only observation.

## Corrections

- The architecture graph and receipt are regenerated only after the final
  source diff, before the second review.
- `install.sh` and the self-teacher fallback now pass the exact
  `research-tools` feature; `scripts/test-public-issues.sh` pins both routes.
- `scripts/check-lay-lints.sh` now checks and runs strict Clippy for both the
  default and research configurations. The full hermetic test configuration is
  a presence-only superset: every runtime module has the same implementation,
  while the feature only admits cold modules and targets.
- `td104-zero-failure-observation-v1.json` records the actual current-manifest
  remote execution: `2,359` selected, zero semantic/infrastructure failures,
  exact source closure, summary SHA, sandbox, and untested scope. The
  known-failure ledger is bound to this observation.

No runtime, package, installation, or service state was changed while applying
the review corrections.
