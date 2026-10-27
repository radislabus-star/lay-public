# Sequential dependency updates — 2026-10-27

The user accepted the order #65 → #67 → #64 → #68 → #66. Each update is
integrated with the current public main, verified independently, and merged
only after its required checks pass. Do not batch the five dependency changes.
The full browser failure investigation remains queued after this sequence.

## Baseline and boundaries

- Accepted stable release: 1.0.104; public main before this sequence:
  `85859d971aada5f577683164db60404b3598023b`.
- Working checkout: `/home/ubu/projects/lay-dependency-updates-20261027`,
  branch `codex/dependency-updates-20261027`. The old dirty main is not built.
- Heavy checks run on the dedicated remote worker through the existing
  resource lease and Cargo disk guard. Preserve the local GNOME installation.
- Keep the existing GNOME/KDE dispatch, delivery routes, DecisionCore,
  ContextAdmissionReducer, edit verification and model/learning ownership.
- A green old PR run is not current integration evidence. Record the exact
  dependency versions, source closure, executable hashes and fresh CI.
- Do not turn a missing or failing check into PASS, alter safety assertions,
  refresh semantic expectations, or suppress new warnings to accept an update.
- No release/tag, installation or desktop restart is implied by a dependency
  merge. Test environments and temporary offline compiler fixtures are separate
  from user data and installed models.

| PR | Dependency | State | First known failure or required boundary |
|---|---|---|---|
| 65 | serde_json 1.0.151 → 1.0.154 | Verifying | Current-main integration; parser/writer contracts |
| 67 | rayon 1.10.0 → 1.12.0 | Queued | Stale architecture source binding; parallel-runtime contracts |
| 64 | arboard 3.6.1 → 3.7.0 | Queued | Installer preservation failure and test inventory drift; causality unproven |
| 68 | zbus 5.15.0 → 5.16.0 | Queued | D-Bus/IBus integration and transitive zvariant update |
| 66 | sha2 0.10.9 → 0.11.1 | Queued | Removed io::Write and output LowerHex implementations |

Read-only triage receipt:
`/home/ubu/.cache/lay/development/dependabot-pr-triage-20261027/triage-report.json`.
The actual zbus lockfile target is 5.16.0; its branch name mentions 5.19.0.

## PR #65 preflight, before changing production code

The current-main merge is `988cfd279b5d6ff971f92afcaa7a0cdff492fed5`.
Comparison of complete lockfile package maps establishes exactly one changed
package: serde_json 1.0.151 → 1.0.154. No Cargo.toml or Rust source change.
Upstream describes invalid UTF-8 error-position fixes and optional
arbitrary_precision/preserve_order behavior changes. The direct Lay dependency
uses default features; inspect actual resolved features during verification.

Options: retain 1.0.151 (safe baseline, no upstream fixes, 7/10); take only the
reviewed lockfile patch (small reversible closure, 9/10); run an unrestricted
Cargo update (unrelated dependency changes and ambiguous attribution, 3/10).
Select the single-package patch. There is no demonstrated need to rewrite a
JSON consumer or introduce compatibility wrappers.

Consequences to verify:

- Candidate retention, ranking and mutation authority: Rust decision code and
  package schemas are unchanged. JSON decoding/configuration still affects
  observed state; run the complete fixed correctness/package lanes rather
  than infer preservation from the small diff.
- Latency, CPU, RSS and allocation: no new timer, queue, cache or owner.
  Parser cost may change; run the existing enforced performance lane and
  resource/disk checks. No speedup or complete workload equivalence is assumed.
- Cache identity/invalidation: Cargo.lock belongs to the test source closure.
  Old build/test receipts cannot certify the changed dependency. Freeze files
  and modes, bind the new result, and preserve the existing cache owners.
- Package/delta reload and learning: retain serialization/deserialization,
  event/spool, integrity, configuration and inverse-feedback tests. Run the
  existing tiny offline lexical-package byte-parity fixture. Do not train,
  rewrite installed packages, or clear shared learning.
- Concurrency/stale results: no scheduler/ownership change is proposed; existing
  concurrent-state and actual D-Bus tests remain required. A malformed payload
  must still fail closed; an error-position difference is not success.
- IME/daemon compatibility: build all ten installed release binaries under the
  locked research-tools profile; verify stable/MSRV feature profiles, lint,
  Python/Node helpers, public installer tests and current security audit.
- Rollback/failure: keep the exact old lockfile entry and accepted runtime.
  Stop on a required failure, identify its first mechanism and fix only that
  mechanism. Merging cannot imply native browser acceptance.
- Maintenance: no extra API layer or permanent fallback. The patch is removed
  by reverting its dependency commit. Later package formats and online updates
  continue through the same schema/integrity and authority contracts.

Verification denominator: unchanged discovered manifest (3537 Rust
correctness/package tests), 11 enforced performance tests, existing Python/
Node suites, both Rust 1.88 profiles, lint/canon/architecture/audit/security,
ten actual release binaries, and offline package byte parity. Record actual
executed counts and existing skips; do not replace these with planned counts.
Physical browser matrix is NOT_TESTED in this dependency source gate.

## Results

Pending. Each dependency receives a separate source/CI/merge receipt and a
specific consequence analysis before its own production change.
