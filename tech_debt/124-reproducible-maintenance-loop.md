# TD-124 — Воспроизводимый короткий цикл сопровождения

Priority: P0 (user-prioritized development workflow, 2026-09-07).
Status: DONE — maintenance tooling scope only; TD-121/client acceptance stays OPEN.
Base: ad4bf0860cc2a79b3004f03b8018cc8d8cbca005, with unfinished TD-121
changes preserved. This task neither accepts TD-121 nor releases 1.0.66.

## Root cause and current baseline

The maintenance workflow is assembled manually: choose a Cargo command,
transfer matching source, reconstruct a private IBus harness from cache paths,
select dependencies and collect evidence. Written rules do not automate this.
`check-lay-changed.sh` still selects all correctness/package lanes for Rust
changes. Its name does not establish component-scoped verification.
The seven recent controlled regressions and 399-test IME run demonstrate
testability, not a measured improvement of total development time.

## Options

1. More instructions alone: 3/10. Low effort, leaves manual execution intact.
2. Reuse existing discovery, sandbox, Cargo/resource guards; one remote
   entrypoint plus repository-owned private client harness: 9/10, selected.
3. New build framework, broad crate split and runtime ownership migration:
   4/10 now. Larger compatibility and proof burden before workflow benefit.

Scores are engineering judgments, not measured performance.

## Ordered deliverables

1. A conservative affected-target plan. A demonstrably private IME source
   change can select its binary test target; shared/unknown dependencies,
   manifests and tooling changes broaden rather than silently skip tests.
   Include additions, deletions, renames, staged and unstaged work. Reuse
   Cargo discovery and existing isolation rules; zero tests or a nonexistent
   requested target is an error. Performance/ignored exclusions are explicit.
   Discovery must pass exact Cargo target flags, not build `--all-targets`
   and then filter execution. Registry checks are restricted to the requested
   targets. For this development-only command the live-discovered full target
   identities are frozen before execution and checked against actual results;
   additions/removals versus the canonical manifest are reported, not silently
   promoted to that manifest. Every selected test failure fails the focused
   command (no known-failure allowance). Canonical full-manifest equality and
   the global known-failure ledger remain unchanged in existing release lanes.
   This separation avoids requiring a full build merely to add one unit test.
   For IME edits conservatively include all integration-test targets as source
   contracts, not just the four currently observed literal-path consumers.
   Shared sources or uncertain cross-binary source inclusion broaden to all.
2. One remote-only development command: preview, source transfer, guarded
   execution and durable result paths. No local Cargo/test fallback; no live
   install/restart. Source/config/command identity, selected target/test
   denominator and elapsed time are recorded. Existing full release gates
   remain mandatory and distinct from focused development PASS.
   The transferred snapshot is complete, not a changed-file overlay; verify
   its content identity remotely before discovery. Planning includes both old
   and new rename paths as well as staged, unstaged, untracked and deletions.
3. Bring the actual-client harness out of private cache into the repository.
   Parameterize candidate, deployed IBus and dependency roots via one explicit
   configuration; preserve exact dependency hashes, private bus/filesystem
   isolation, bounded resources, no retry, assertions and cleanup. Each run
   uses fresh output. Tooling PASS does not turn the current client 0/5 FAIL
   into a product PASS. No new sleeps, warmup workaround or higher deadline.
4. Document the short operational route and measure planning, transfer/build,
   focused execution and actual-client execution separately. Do not invent a
   speedup ratio without comparable baselines. Keep advanced runtime cleanup
   as separately discussed Stage 2, not an unannounced prerequisite.

## Consequences and boundaries before code

- Runtime sources, candidate retention/ranking, SafetyGate, edit validation,
  learning, package loading, hot-path latency and application authority are
  unchanged. Fixtures cannot grant additional runtime authority.
- Under-selection is the principal correctness risk: unknown/shared paths
  fall back to broader checks. The focused result is never a release receipt.
- Stale source/dependency reuse is a risk: bind the snapshot and dependencies;
  reject invalid/missing paths and hashes, do not guess installed artifacts.
- Remote transfer must not overwrite an active checkout or an unrelated
  project. Use task-owned staging and the existing single heavy lease.
  Commands use argument arrays/validated paths, not untrusted shell snippets.
- All builds/tests and architecture refreshes execute on the remote 20-CPU
  host under existing guards; no bypass on missing tools or unavailable host.
  Keep CPU 2000%, MemoryHigh 24G/Max 28G, swap 1G, Tasks 512, target <=12 GiB.
  Private client retains its narrower CPU200%/1536M/swap0/Tasks128/90s envelope.
- Existing GTK, terminal and atomic transports remain distinct; production
  daemon, global IBus, desktop input sources and installed binaries untouched.
- Reuse test discovery/execution rather than implement a second Rust harness.
  New orchestration replaces manual commands; avoid a plugin/config framework.
- Rollback: remove the new development entrypoint/harness and revert scoped
  test-tool extensions. Existing release commands remain available unchanged.

## Acceptance and evidence

- [x] Independent specification review: second pass 9/10, H0/M0; one repair.
- [x] Conservative routing tests, including shared/unknown/deleted paths and
      cross-target source use; exact nonzero discovery and failure propagation.
- [x] Runner tests: argument safety, local execution refusal, immutable run
      outputs, unavailable remote, guard propagation and source provenance.
- [x] Harness tooling tests: missing/hash-mismatched inputs fail before launch,
      configured paths consistently reach driver/component, fresh output and
      sandbox/cleanup semantics retained; fixture semantics unchanged.
- [x] Remote guarded focused check through the new public command.
- [x] Actual-client run through repository-owned harness (record real verdict;
      a known product failure is not a harness success proof by itself).
- [x] Owning document with commands, hashes, measured timings, untested scope
      and explicit `runtime_authority_changed=false`.
- [x] Remote architecture refresh/check, independent code review >=8/10 with
      no unresolved High/Medium, maximum two repair passes.
- [x] Explicit task-only Git checkpoint; commit/push identity is the commit
      containing this completion record, not the unrelated dirty runtime tree.

## Specification review

`td124_spec_review`, fresh-context Sol/High: 8/10, H1/M2 before clarification.
Clarified exact Cargo target selection, scoped registry checks, conservative
integration consumers and complete snapshot provenance. The proposed canonical
projection gate is intentionally replaced by live-discovery/execution equality
for development only: otherwise adding a unit test would again require a full
manifest build. This does not alter canonical release acceptance. No known
failure tolerance is admitted in the focused route. Second pass: 9/10, H0/M0,
accepted for development tooling. Record all discovered, explicitly excluded
and actually executed identities as distinct denominators.

## Completion

Implemented the selected workflow, documented in [DEVELOPMENT.md](../DEVELOPMENT.md).
Final tooling suite:83 discovered,82passed,1 explicit optional live-cgroup skip.
Actual focused IME check:396/396,3 performance exclusions,0failures; same
denominator on first and warm-cache runs. First full command55.461s, repeated
unchanged-Rust command33.246s; discovery/build21.898s ->0.337s. These compare
two runs of the new command, not old versus new development productivity.

Specification9/10; final code and narrow second-correction review9/10,H0/M0.
At most two corrective passes. Remote architecture gate PASS, advisory size
warnings retained. Detailed [execution evidence](evidence/td124-maintenance-execution.md)
and [review record](evidence/td124-maintenance-review.md) own the denominators,
hashes, failures and limits.

The client was moved into the repository and executed with verified inputs;
it still completes0/5 scenarios, with `NameHasNoOwner` after Space. Process
observation and cleanup are now accurate, but the disappearance cause remains
UNKNOWN. This task does not accept the client, exonerate its environment,
resolve TD-121 or install/release anything. Broad runtime cleanup remains a
separate decision, not a completed or silently added deliverable.
