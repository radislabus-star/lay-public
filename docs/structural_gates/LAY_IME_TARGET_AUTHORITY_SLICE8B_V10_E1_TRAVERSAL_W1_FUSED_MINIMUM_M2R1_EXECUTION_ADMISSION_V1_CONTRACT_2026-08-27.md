# M2R1 execution admission V1 contract

Date: 2026-08-27

## Scope

This gate may move only:

```text
M2R1_CONTROLLER_VERIFIED_UNRUN
  -> M2R1_LIVE_PREFLIGHT_PASS_UNADMITTED
  -> M2R1_EXECUTION_ADMITTED
```

It cannot run the M2R1 controller, create its provenance/state/cache namespace,
create or consume a scientific marker, invoke Cargo/rustc compilation, open a
perf event, execute the diagnostic ELF, edit production source, or change
runtime authority.

## Pinned predecessor

```text
task_id                 slice8b-v10-e1-traversal-w1-fused-minimum-m2r1-v1-20260827
transaction_id          2dae728a39aecd422995828674d12e311ab6362ebab4013c4f2520b3f6933c5f
controller              d3540936c19e45230a70c805b77cf6dab024636457ea0577281a59be2e128106
remote controller       26bfbbf2a9626b36bad549d6827d7442d1d1b5db7ced0f65da41c099a793de09
bootstrap auditor       c0a7221eeeef460db174682dafad16a7765a970a5623e3641c0657de4bfce84a
build auditor           0f943646c0e13c6fe53c406aa542882d68ab9dbe447dc3c660719110ccae6762
terminal auditor        37b6d27eaa719ebe6347a97303602fc30b4d4ec7078a0bf49b3ce424037e23fe
Rust fragment           a6ea388d5d76f8223511fd4822cff2df9fd0c3394fc200f4fc52db956522ce5b
implementation receipt  b888f1d10f4e846bcb1d08afaab3d5f49dbad0deacd4a0916037ae7526d509f4
code-route contract     4268d960359e1e6d7cfa00985cafe8a5ea8906be2077dcc824cfe354e76c3948
code-route receipt      339475c9310865b2d2cd799919f918ee37fb06ba7905ebe3f9249638b85a3623
```

The historical M2 terminal receipt remains immutable
`BLOCKED_BUILD`; this admission grants no retry in that namespace.

## Live closure

The producer takes two complete snapshots separated by 15 seconds. The
independent auditor takes a third fresh snapshot. Every snapshot must prove:

```text
host                  e-MEGA-MINI-M1-13th
machine-id SHA-256    5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441
kernel                6.8.0-124-generic
online CPUs           0-19
cpu_core / cpu_atom   0-11 / 12-19
cargo / rustc / LLVM  1.97.1 / 1.97.1 / 22.1.6
perf                  6.8.12
sample-rate ceiling   8000
perf paranoid         4

M2R1 provenance       absent
M2R1 state            absent
M2R1 cache            absent
conflicting process   0
open perf_event fd    0
```

Each snapshot also verifies exact size, mode and SHA-256 of package, sidecar,
V7 denominator, schedule, Cargo.toml, Cargo.lock and cargo-guard. Version
queries are provenance reads, not compilations.

Conflict detection inspects the root process table and `/proc/*/fd`. Active
perf, Cargo/rustc, diagnostic test ELFs, Lay performance controllers or known
foreign experiment tokens veto admission. A dirty observation writes no
admission and consumes no scientific authority.

## UID proof

Between producer snapshots, SSH user `e` performs this disposable transaction
under `/home/e/.cache` at a path unique to the M2R1 transaction:

```text
mkdir -> create -> write -> fsync -> rename -> reopen/read
-> unlink -> fsync directory -> rmdir -> fsync parent -> verify absent
```

The probe is not the future controller cache or provenance tree. Exact future
path ownership remains the bootstrap and bootstrap-auditor obligation before
markers are created.

## Publication ownership

The producer atomically seals a local evidence tree with verdict
`M2R1_LIVE_PREFLIGHT_PASS_UNADMITTED`; its command graph has no symbol or call
path for the fixed execution-admission file. The independent auditor pins the
producer source and receipt, validates the complete evidence manifest, takes a
fresh clean snapshot, and is the sole publisher of:

```text
verdict          M2R1_EXECUTION_ADMITTED
safe_to_execute  true
```

Any missing or unknown predicate, identity drift, incomplete UID proof,
leftover probe, dirty host, manifest mismatch, or publication uncertainty
leaves admission absent and is `BLOCKED_PROVENANCE`.

## State and rollback

Producer and auditor evidence are append-only. The admission is created with
exclusive atomic publication and mode `0444`; overwrite is forbidden. A crash
before publication leaves admission absent. A crash after publication leaves
the exact durable admission for the sealed M2R1 controller to verify. Neither
failure permits recreation of old M2 or M2R1 scientific markers.

Local installed-runtime hashes must equal the M2R1 implementation receipt
before and after both admission stages. The admission transaction has zero
semantic, candidate, ranking, verifier, latency, package, deployment, or
production authority.

## Next action

After independent publication, exactly one invocation of the sealed M2R1
controller `run` action is admitted. Its fixed route remains:

```text
BOOTSTRAP -> bootstrap audit -> 8 markers -> BUILD -> build audit -> PARITY
-> B0-ITERATOR -> G0-M1-GUARDED -> I0-INTERLEAVED
-> I1-INTERLEAVED -> G1-M1-GUARDED -> B1-ITERATOR -> terminal audit
```

No intermediate number is a scientific conclusion before terminal audit.
