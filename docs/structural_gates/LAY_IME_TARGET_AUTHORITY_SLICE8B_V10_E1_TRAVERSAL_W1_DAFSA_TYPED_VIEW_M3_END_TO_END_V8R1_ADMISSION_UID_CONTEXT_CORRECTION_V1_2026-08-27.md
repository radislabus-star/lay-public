# DAFSA Typed View M3 End-to-End V8R1 Admission UID-Context Correction V1

Date: 2026-08-27

## Scope

This correction repairs only the live execution-admission probe for the sealed
M3 V8 end-to-end experiment. It does not change the V8 Rust source, candidate
language, generation-owner model, fixed corpus, scientific test, build command,
latency/RSS/reload gates, production claim boundary, or runtime authority.

## Immutable V8 V1 History

The V8 V1 controller transaction is terminal and remains immutable:

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8-20260827

transaction_id
  e9c2674282e9feaff1e75c7ca17b74040b3a8dd8b1bbab2c26e999917a6d35ec

controller implementation receipt
  f40e3fe5c0e583d5223d8c734b671e411d74b189b9bfd98ffe7b81c9aa83c230

execution journal SHA256SUMS
  9a415838a1f18e972548bdc1b5285042c0ae049f5809b8b4520fd7867020eb81

terminal controller verdict
  BLOCKED_PROVENANCE
```

The journal contains one durable `live-admission` intent and no completion.
It records zero completed external actions. The V8 remote parent, state tree,
cache, markers, Cargo build, rustc compilation, test ELF, and scientific subject
were absent after the failure. V8 V1 is never retried and its journal is never
edited, deleted, or reinterpreted as scientific evidence.

## Exact Defect

The independent auditor's `remote_snapshot()` executed under root and invoked
the user-owned rustup shims without the controlled build environment:

```text
/home/e/.cargo/bin/cargo -V
/home/e/.cargo/bin/rustc -Vv
```

In that root process, rustup selected root's unset toolchain context and returned
exit `1` with `no default is configured`. The auditor then reported
`remote Cargo drift`. Direct read-only execution as UID `e` returned the frozen
toolchain exactly:

```text
cargo 1.97.1 (c980f4866 2026-06-30)
rustc 1.97.1 (8bab26f4f 2026-07-14)
commit 8bab26f4f68e0e26f0bb7960be334d5b520ea452
LLVM 22.1.6
```

Therefore the actual failure class is controller admission provenance, not
toolchain drift and not an end-to-end scientific failure.

## Corrected Admission Architecture

V8R1 separates three facts that V8 V1 mixed together:

```text
root host snapshot
  hostname / kernel / machine-id / CPU topology
  namespace absence / process conflicts / free space
  runtime projection
  NO Cargo or rustc process

build-toolchain snapshot
  exact /home/e/.cargo/bin/cargo and rustc
  exact controlled build environment:
    HOME=/home/e
    LANG=C.UTF-8
    LC_ALL=C.UTF-8
    PATH=/home/e/.cargo/bin:/home/e/.local/bin:...
    RUST_BACKTRACE=0
  one read-only query before admission publication

subject UID capability
  actual SSH execution as UID e
  create / write / fsync / rename / reopen / read / unlink
  exact future parent chain
  probe absent after PASS
```

The host snapshot is repeated after the disposable UID probe. The toolchain
snapshot is not repeated during admission; `build-once` independently checks
the same exact toolchain under the same controlled environment before consuming
`build.available`.

Any command failure must retain both stdout and stderr in the local controller
failure evidence. An auditor error response is structured evidence and must not
be reduced to an empty stderr string.

## Fresh Namespace

The corrected route uses a new namespace and transaction:

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r1-20260827

transaction_id
  7d6455e678c244be3c31dc52c2b64d55f34d0a91338afa1219acf06ff327ffb9
```

This is not a retry of a consumed scientific route. V8 V1 created no remote
namespace and consumed no marker. V8R1 receives new controller identities,
new local evidence roots, a new remote namespace, and new one-shot markers.

## Reused Frozen Scientific Contract

V8R1 reuses without change:

```text
V8 paper SHA                 c5f1655ce4ab91f0...
V8 test source SHA           28f87a76fc199698...
source implementation receipt cf4b38d81cc7f9ea...
routes                       BUILD, E2E
markers                      build.available, e2e.available
scientific subject           m3_end_to_end_physical_proof
CPU                          0
warmup                       1 round
measured schedule            F/R/F/R, 1,528 requests
production authority         not admitted
runtime mutation             forbidden
perf / PMU                   forbidden
```

All semantic, capacity, reload-identity, RSS, latency, environment, immutable
evidence, marker-consumption, build-once, and failure-dispatch requirements of
V8 remain effective.

## Corrected State Machine

```text
V8_V1_BLOCKED_PROVENANCE (immutable)
        |
        v
V8R1_CONTROLLER_PREFLIGHT
        |
        v
V8R1_CONTROLLERS_VERIFIED_UNRUN
        |
        v
root host snapshot + controlled toolchain snapshot + UID=e probe
        | FAIL -> BLOCKED_PROVENANCE, no namespace, no markers
        v
M3_V8R1_EXECUTION_ADMITTED
        |
        v
bootstrap -> independent audit -> two markers
        |
        v
consume build marker -> ONE Cargo build -> independent build audit
        |
        v
quiet-host audit -> consume E2E marker -> ONE scientific subject
        |
        v
independent terminal audit
        |
        +-- PASS -> M3_END_TO_END_TEST_OWNER_PASS
        +-- FAIL -> exact frozen BLOCKED_* verdict
```

No failure grants a retry, marker recreation, second build, second subject,
production activation, installed-package edit, daemon restart, or runtime edit.

## Admission Verdict

This paper admits implementation of a fresh V8R1 controller set only after a
trusted implementation preflight returns `READY_TO_IMPLEMENT`. It does not by
itself admit remote writes or scientific execution.

