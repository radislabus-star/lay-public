# M2 execution admission V2 contract

Date: 2026-08-27

## Scope

This gate is the independent live preflight admitted by the sealed M2 V3
implementation receipt. It may establish only:

```text
M2_CONTROLLER_V3_VERIFIED_UNRUN
        -> M2_LIVE_PREFLIGHT_PASS_UNADMITTED
        -> M2_EXECUTION_ADMITTED
```

It does not run the M2 controller, create the M2 namespace or controller cache,
create or consume markers, invoke Cargo or rustc compilation, open perf/PMU
events, execute a subject, edit production source, or change runtime authority.

The controller-compatible admission filename remains dated `2026-08-26`
because that exact path is frozen in the sealed V3 controller. This V2 contract,
producer evidence and independent audit are dated by their actual execution on
2026-08-27.

## Pinned chain

```text
M2 V3 controller                  4f2f6484d7ada483688b59a2403354548d4aaffdc594074c4447326c7b8c1f7f
remote controller                 14c7c60814927a8f3bcfc607a391d78552fdf8cdd51253eccd429d1f40c50dd6
bootstrap auditor V3              30da6d116df0fa5d5372dc8718c86bb3e46c4cb59edcc918429cceb523959869
build auditor V3                  091e97c8aa0e1d8a7455fefba0fb038ea04566996b149d4a10feaad0e1d5030e
terminal auditor V3               1656e3a2fa8b772a5ef8c01daab7daf03b751c73fc2ac4bbda02a1ed6520b92b
M2 Rust fragment                  b0a775420edf9e9d6e7f0b59f9ad840e4822cd0fdd0adc2429ea22a3e9e3a175
implementation receipt V3         55b938ef7851bcf560c1e165e0ebe3c1c5906df6f8c9c5a76559488f0ab35f0a
implementation preflight V5       60a676d2d59d296e5e8445b61311ff46edd9a62d3aa950da78a1ea0c9c3b2598
implementation V5 receipt         a33fadd7fdfe135e1f726f91b9a2cae85bc7b27343a3ab21ed55f91a9c2561c3
code-route V5 contract            0fdfa5ece5b1e53a4d4acd661c4403978f0524b086c1fd5d52ef50c1521501d3
code-route V5 receipt             54b66f7a30d6f7b6edc67c2eb07748259e78fa45f4d26c9da241325e2a455acb
```

`54b66f7a...` is the code-route receipt, not the contract. Admission must pin
both identities separately.

## Live closure

The producer performs two complete remote snapshots separated by a fixed
15-second quiet interval. The independent auditor performs a third fresh
snapshot before publishing admission. Every snapshot must prove:

```text
hostname                         e-MEGA-MINI-M1-13th
machine-id SHA-256               5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441
kernel                           6.8.0-124-generic
online CPUs                      0-19
cpu_core CPUs                    0-11
cpu_atom CPUs                    12-19

cargo                            1.97.1
rustc                            1.97.1 / 8bab26f4... / LLVM 22.1.6
perf                             6.8.12
perf_event_max_sample_rate       8000
perf_event_paranoid              4

M2 provenance parent             absent
M2 state                         absent
M2 controller cache              absent
M2 markers                       absent by namespace absence
active M2 process                0
active conflicting experiment    0
open perf_event descriptors      0
```

Conflict detection is conjunctive, not a single argv grep. It scans the full
process table as root and independently scans `/proc/*/fd` for
`anon_inode:[perf_event]`. It rejects active `perf`, Cargo/rustc compilation,
known diagnostic test ELFs, known Lay performance controllers/subjects, or a
foreign experiment token such as `leg01-v13-all-live-inputs` or
`nando-motif-shape-census`. The snapshot command itself and version-query
children are excluded only by exact controller-owned identity.

A dirty snapshot returns `WAIT_CONFLICTING_EXPERIMENT`, creates no fixed-path
admission and consumes no scientific authority. Repeating this observation
after the foreign process exits is allowed because no M2 namespace, marker,
subject, Cargo, perf or PMU state exists.

## UID boundary

Mode inspection alone is insufficient after D3. Before producer PASS, SSH user
`e` performs an actual disposable capability transaction under the exact
neutral cache ancestor `/home/e/.cache`:

```text
mkdir exclusive
create -> write -> fsync -> rename -> reopen/read -> unlink
fsync directory
rmdir
verify absent
```

The probe path is not the future M2 controller cache and not the M2 provenance
namespace. The exact future root-owned `0755` parent chain is still created and
tested by the sealed remote bootstrap, then independently checked by the sealed
bootstrap auditor before any marker exists. Mutating that future namespace in
this admission transaction is forbidden.

## Input and runtime closure

The producer and auditor verify exact size, mode and SHA-256 for the frozen
package, sidecar, V7 denominator, schedule, Cargo.toml, Cargo.lock and
cargo-guard inputs. Local installed/runtime hashes must equal the V3
implementation receipt before and after admission publication.

Version queries are provenance reads. They are not Cargo/rustc compilation.

## Independent publication

The live producer atomically seals a complete evidence tree with verdict:

```text
M2_LIVE_PREFLIGHT_PASS_UNADMITTED
```

It cannot write the controller-compatible admission path. The independent
auditor pins the producer source SHA, auditor source SHA, producer receipt SHA,
all predecessor identities, the disposable UID proof, both producer snapshots,
and its own fresh clean snapshot. Only it may atomically publish:

```text
verdict          M2_EXECUTION_ADMITTED
safe_to_execute  true
```

Any incomplete observation, unknown predicate, identity drift, dirty live
state, leftover probe, failed manifest, or conflicting process leaves the
admission absent and selects `BLOCKED_PROVENANCE` for that audit revision.

## Consequence analysis

This gate changes no candidate, lattice, ranking, verifier, learning, feedback,
package, delta, cache identity, IME behavior or daemon behavior. It therefore
cannot improve or regress semantic quality and cannot create false lexical
authority. It allocates only bounded local evidence and a disposable remote
probe directory; it adds no runtime allocation, RSS, latency or tail cost.

The concurrency hazard is admission racing a foreign experiment. Two producer
snapshots plus one independent final snapshot reduce that interval; admission
is consumed immediately by the already sealed controller. This cannot prevent
an uncooperative process from starting later, so the conclusion remains scoped
to the observed admission window. No result may be promoted if later route
evidence reports thermal, capability, measurement or provenance failure.

Failure and rollback are fail-closed: producer failure removes only its owned
disposable probe, retains diagnostics, and writes no admission. Auditor failure
retains producer evidence and writes no admission. Existing M2 bytes and the
production runtime remain immutable. Removal cost is bounded to deleting these
two admission-only scripts after the research branch is archived.

Alternatives rejected:

1. A mode-bit-only check repeats the D3 defect and does not prove UID behavior.
2. Precreating/chmodding the future M2 parent changes the namespace before
   admission and weakens bootstrap ownership.
3. The selected disposable UID proof plus sealed bootstrap proof keeps live
   admission and exact future-path ownership separate.

## Admission boundary

After independent publication, the next action is exactly one invocation of
the sealed V3 controller `run` action. Scientific route order remains:

```text
BOOTSTRAP -> bootstrap audit -> 8 markers -> BUILD -> build audit -> PARITY
-> B0-ITERATOR -> G0-M1-GUARDED -> I0-INTERLEAVED
-> I1-INTERLEAVED -> G1-M1-GUARDED -> B1-ITERATOR
-> terminal audit
```

Intermediate route values are evidence only until terminal audit. Admission
does not itself establish M2 benefit, latency improvement, production policy,
integration authority, installation or deployment authority.
