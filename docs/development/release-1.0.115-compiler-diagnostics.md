# Lay 1.0.115 — preserve compiler failure diagnostics

Preflight2026-10-30; baseline740fc6ec (completed114 publication).
Only tooling implementation scripts/test_lanes/discovery.py and its existing
tests/test_test_lanes.py. No runtime source, delivery route or configuration
change. Owner remains the existing discovery parser and CLI failure contract.

Map: check-lay-tests.sh -> test_lanes.cli.prepare -> cargo_discovery -> guarded
Cargo JSON stdout temporary file -> nonzero exit -> DiscoveryError -> deleted
temporary file. The first failed transition is raising before extracting the
compiler-message diagnostic. parse_cargo_artifacts has the same loss when a
failed build-finished reaches its parser. Reuse the neighboring failed-build
diagnostic preservation pattern in scripts/lint_inventory.py:145–155.

Measured RED: CROOT/release115-discovery-error-red-v2/result.json. A real,
dependency-free failing Rust crate produces E0425 for fixture_missing_symbol;
the production discovery reports only Cargo test discovery exited101. The
initial v1 mock intercepted sysroot lookup and is retained as SETUP_ERROR,
not a product result. Runtime/source were unchanged by the RED experiment.

Options: preserve diagnostics in existing DiscoveryError9/10 (selected);
persistent raw-log store6/10 adds retention and another artifact owner;
retry Cargo with another output format2/10 duplicates work and can change the
observed build. Never accept failed Cargo or weaken terminal-success checks.

Consequence check: candidate/lattice/ranking/authority, learning, package reload,
runtime caches, focus, IME/daemon consumers and latency deadlines are untouched.
Only failed discovery reads compiler JSON once more before cleanup; normal
successful discovery keeps its existing parser. Failure reporting can allocate
diagnostic text, bounded by the actual build log; stream lines rather than
copying the complete log. No runtime hot-path, concurrent state, fallback owner,
new dependency, persistent store or retry. Malformed/absent diagnostics must
preserve the original exit-code failure. Future Cargo diagnostics with no
rendered string retain message/code/span where available; warnings do not
obscure errors. The existing strict parser still rejects malformed streams,
missing/failed terminal records and records after build-finished.

Proof plan: regression tests for rendered error/span reaching the public
exception after temporary-file cleanup, failed-terminal parsing, fallback
without rendered text, and absent/malformed diagnostic handling. Re-run the
same real failing crate through production discovery after the fix. All-lane
Python contracts, strict source gates, complete installed-binary build and
independent public CI/Security remain release requirements. Reuse114 native
evidence explicitly; tooling-only115 does not establish a new physical matrix.

Rollback: revert the diagnostic helper, its two call sites and tests together.
No migration or compatibility branch. Remove the helper if Cargo failure
reporting is later consolidated in the existing parser; do not duplicate it
in CLI callers. No runtime mutation authority changes.

Results: pending;115 is not yet published or installed.

Short GREEN: CROOT/release115-green-short-v1/result.json;7/7 existing+new
discovery tests PASS. The same actual failing Rust crate now exposes exit101,
E0425,fixture_missing_symbol and src/lib.rs:2 in DiscoveryError. It remains
a rejected build. Runtime and canonical remote workspace unchanged. RED unit
probe ran4 methods:2 expected message assertions failed plus5 missing-helper
subtest errors; its evaluator expected2 errors and marked UNEXPECTED. Retained
raw log proves the two old-path regressions; do not call that evaluator PASS.

Final source/release gates PASS,not installed/published yet:
3611/3611 canonical correctness/package tests across34targets,487actual
process-isolated executions,canonical invocation exit0. Every test identity
verified in its actual bulk/process log. Manifest unchanged28ba8d9e...;
actual zero-failure observation evidence/release-1.0.115-tests.json bound in
known_failures.json. Python439testsPASS(10skips); strict lints/default+research
inventories,Node,public/installer,shell/GNOME syntax,architecture50 and
Rust1.88default+lexical-compilerPASS. All10installable programs built217.566s,
compiler warnings0;artifact paths resolved from actual Cargo JSON and copies
retained. Receipt CROOT/release115-full-v1/result.json;source return20files
verified against pinned local prehashes;source closureSHA1adba121f4cd228cc1f27d048edb36da4a76a549c26dad517fc5f769c53b15f3.

Runtime processes/protected37integration hashes unchanged. DedicatedKDE remains
114;no115 runtime installation or new physical matrix. Source115 changes only
tooling and version metadata. Preserve114native159/21/40Firefox and historical
179/0/41Chrome;no new physical PASS or compatibility improvement claimed.
Public independent CI/Security and publication/updater readback pending.
