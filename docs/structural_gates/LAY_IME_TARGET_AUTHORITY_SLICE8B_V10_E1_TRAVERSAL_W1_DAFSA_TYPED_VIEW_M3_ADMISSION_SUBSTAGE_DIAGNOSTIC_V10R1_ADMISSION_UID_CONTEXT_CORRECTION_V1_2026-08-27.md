# DAFSA Typed View M3 Admission-Substage V10R1 UID-Context Correction V1

Date: 2026-08-27

## Scope

This correction repairs only the live execution-admission toolchain projection
for the sealed M3 V10 admission-substage diagnostic. It does not change the
test-only Rust observer, candidate language, admission implementation, fixed
corpus, BUILD or TRACE command, measured schedule, substage registries,
scientific gates, production claim boundary, or runtime authority.

## Immutable V10 V1 History

The V10 V1 controller transaction is terminal and remains immutable:

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10-20260827

transaction_id
  8058be994305226e9af3fbdee2e6b29bd9111ffbf8203ef20db9feeb1ca56a22

controller implementation receipt
  3b750bd1ec66d67c6d7e1dbe306b0c04a02b7ab9a839ec07bad0d7397b8652a8

execution journal SHA256SUMS
  099d3c8334b11c98c3e7f0893a11d1347648aed1746cfcc1e466a166d818f12b

terminal controller verdict
  BLOCKED_PROVENANCE
```

The journal contains one durable `live-admission` intent and no completion. It
records zero completed external actions. The V10 remote parent, state tree,
cache, markers, Cargo build, rustc compilation, test ELF, and scientific
subject remained absent. V10 V1 is never retried and its controllers, journal,
and evidence are never edited, deleted, or reinterpreted as scientific output.

## Exact Defect

The independent auditor's root host snapshot invoked the user-owned rustup
shims without the controlled build environment:

```text
/home/e/.cargo/bin/cargo -V
/home/e/.cargo/bin/rustc -Vv
```

Rustup therefore used root's unset toolchain context and returned exit `1` with
`no default is configured`. The same binaries under the frozen environment
used by the future build return the exact pinned Cargo, rustc, commit, and LLVM
identities. The actual failure class is controller admission provenance, not
toolchain drift, build failure, semantic failure, or scientific capability.

The local controller also discarded the structured auditor response on a
non-zero exit because it retained stderr but not stdout. V10R1 must retain a
bounded tail of both streams in failure evidence.

## Corrected Admission Architecture

V10R1 separates the facts that V10 V1 mixed together:

```text
root host snapshot
  hostname / kernel / machine-id / CPU topology
  namespace absence / process conflicts / free space
  runtime projection
  NO Cargo or rustc process

controlled build-toolchain snapshot
  execution UID = root, matching remote build controller
  exact /home/e/.cargo/bin/cargo and rustc
  HOME=/home/e
  LANG=C.UTF-8
  LC_ALL=C.UTF-8
  PATH=/home/e/.cargo/bin:/home/e/.local/bin:...
  RUST_BACKTRACE=0
  one read-only query before admission publication

subject UID capability
  actual SSH execution as UID e
  create / write / fsync / rename / reopen / read / unlink
  disposable path absent after PASS
```

The host snapshot is repeated after the disposable UID probe. The build route
independently checks the same exact toolchain under the same environment before
consuming `build.available`.

## Fresh Namespace

The corrected route uses a new namespace and transaction:

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r1-20260827

transaction_id
  7d067078f66e4c64724e5f6568304c0d6f6ab1c1ada92e09a98e05a19f0e1f17
```

This is not a retry of a consumed scientific route. V10 V1 created no remote
namespace and consumed no marker. V10R1 receives new controller identities,
new local evidence roots, a new remote namespace, and new one-shot markers.

## Reused Frozen Scientific Contract

V10R1 reuses without change:

```text
V10 paper SHA                 afb4709efd22a631...
test-only observer source     exact frozen three-file implementation
routes                        BUILD, TRACE
markers                       build.available, trace.available
scientific subject            m3_end_to_end_physical_proof
CPU                           0
warmup                        382 requests
measured schedule             F/R/F/R, 1,528 requests
trace rows                    exactly 1,910 V9 + 1,910 V10
substage registry             36
action registry               4
reason registry               43
production authority          not admitted
runtime mutation              forbidden
perf / PMU                    forbidden
```

All source-closure, semantic, capacity, reload-identity, RSS, environment,
immutable-evidence, marker-consumption, build-once, trace-once, independent
audit, and failure-dispatch requirements of V10 remain effective.

## Corrected State Machine

```text
V10_V1_BLOCKED_PROVENANCE (immutable)
        |
        v
V10R1_CONTROLLER_PREFLIGHT
        |
        v
V10R1_CONTROLLERS_VERIFIED_UNRUN
        |
        v
root host snapshot + controlled toolchain snapshot + UID=e probe
        | FAIL -> BLOCKED_PROVENANCE, no namespace, no markers
        v
V10R1_EXECUTION_ADMITTED
        |
        v
bootstrap -> independent audit -> two markers
        |
        v
consume build marker -> ONE Cargo build -> independent build audit
        |
        v
quiet-host audit -> consume TRACE marker -> ONE scientific subject
        |
        v
independent terminal audit
        |
        +-- PASS -> ADMISSION_SUBSTAGES_DECOMPOSED
        +-- FAIL -> exact frozen BLOCKED_* verdict
```

No failure grants a retry, marker recreation, second build, second subject,
production activation, installed-package edit, daemon restart, or runtime edit.

## Admission Verdict

This paper admits implementation of a fresh V10R1 controller set only after a
structural route PASS and implementation preflight `READY_TO_IMPLEMENT`. It does
not itself admit remote writes or scientific execution.
