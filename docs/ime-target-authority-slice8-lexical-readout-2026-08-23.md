# Slice 8 Lexical Candidate-Specific Live Readout

Status: Slice 8A passed. Slice 8B V6 now preserves the full fixed denominator
and proves the produced-field comparator, but upstream field coverage and
latency remain open. Slice 8C promotion is forbidden.

## Current Fact

The installed `1.0.43` route already has one canonical candidate producer:

```text
L1.1 Phase8I V9
-> Productive V90 over immutable canonical V13 identities
-> CanonicalL2Field candidates
-> L3/L4/DecisionCore
-> verifier
-> atomic backend
```

The remaining authority split is inside that route. Production
`productive_v1/live.rs` still settles `L2FieldAuthority` through
`live_authority(lattice, common_l3_required)`. The complete target model
implemented by Slices 2-6 is still proof-only:

```text
PreparedTargetMaterialV1
-> exact frame binding
-> CandidateStateV1
-> complete ConflictCohort
-> Winner | Tied | ABSTAIN
```

Therefore candidate birth is canonical, but lexical authority is not yet owned
by the complete candidate-specific cohort. Backend atomicity was independently
closed by release `1.0.34`; it is not the current blocker.

## First Lost Invariant

The first missing relation is exact frame identity at the library boundary.
`lay-ibus-engine::InputFrameIdentity` owns path, focus receipt, tail epoch,
committed tail, context, token, layout profile, output capability and config
identity. `CorrectionRequest` currently carries only text and policy options.
Constructing a lexical certificate from `original: &str` would invent focus,
caret, selection, layout/config generation and lease identity. That is
forbidden.

## Required Route

```text
IBus InputFrameIdentity
-> typed immutable LexicalAuthorityFrameV1
-> FullField request
-> one CanonicalL2Field preparation
-> PreparedTargetMaterialV1
-> field-generation-bound lease
-> exact frame projection for every retained target
-> CandidateStateV1 for every retained target
-> one complete ConflictCohort
-> lexical Winner       -> L2Certified(candidate-specific certificate)
-> lexical Tied         -> Tied, no automatic certificate
-> incomplete/overflow  -> ABSTAIN, no automatic certificate
-> L3/L4 rank evidence
-> one TransitionDecisionCore
-> verifier
-> installed atomic backend
```

Callers without a current exact frame may receive the same bounded candidate
lattice for display or diagnostics, but they cannot receive `L2Certified`.

## Ownership

| Role | Owner |
|---|---|
| Candidate birth | one Productive V90 field over L1.1 V9 and canonical V13 identities |
| Material completeness | `PreparedTargetMaterialV1` |
| Volatile input identity | caller-produced `LexicalAuthorityFrameV1` |
| Frame binding | `ExactInputFrameV1` |
| Candidate validity | `CandidateStateV1` |
| Lexical settlement | one complete `ConflictCohort` |
| Context evidence | L3/L4, unable to mint `L2Certified` |
| Final rank/admission | one `TransitionDecisionCore` |
| Edit authorization | verifier |
| Physical mutation | installed atomic backend |

## Migration Slices

### 8A Typed Frame Carrier

- add one library-owned immutable frame type;
- convert the existing IBus identity without text reconstruction;
- pass it through active-composition, InputGate and CorrectionRequest;
- keep callers that do not own frame evidence on `None`;
- preserve all candidate and decision bytes in this slice.

### 8B Cohort Compare

- promote the proof-only material/frame/state/cohort implementation to a shared
  non-mutating library owner;
- derive old and new lexical verdicts from one prepared field;
- record candidate retention, completeness, verdict and first divergence;
- issue no new certificate and change no live decision.

### 8C Lexical Authority Flip

- require a complete lexical cohort and exact current frame;
- attach one `AuthorityCertificateV1::L2Certified` only to its Winner;
- remove `common_l3_required`, source priority and producer identity from
  automatic lexical authority;
- make the parallel legacy Boundary route non-authoritative;
- preserve explicit Tab acceptance and candidate display.

### 8D Compatibility Removal

This is Slice 10, not part of the first flip. After one bounded compare receipt,
delete the old field-wide settlement, parallel Boundary authority and compare
instrumentation. No permanent dual computation is allowed.

## Fail-Closed Rules

- no exact frame: no lexical automatic authority;
- stale focus, tail, layout, config, package or field generation: no authority;
- incomplete, failed or overflowed enumeration: `ABSTAIN`;
- multiple grounded members in one edit footprint: `Tied`;
- multiple edit components: `ABSTAIN`;
- missing grounded target or geometry replay mismatch: no certificate;
- L3/L4 uncertainty cannot erase a grounded L1.1 target;
- L3/L4 support cannot manufacture `L2Certified`;
- no package rebuild, candidate-frontier reduction or literal word branch;
- verifier and atomic backend policy remain unchanged.

## Gates

The flip is conjunctive. A narrow software PASS cannot substitute for another
row.

```text
frame producer/consumer identity parity                 100%
all retained material targets rebound                   100%
candidate retention versus current canonical field      100%
grounded L1.1 target loss                                  0
incomplete Winner / false singleton                        0 / 0
false lexical authority                                    0
fixed lexical non-context cases                           all PASS
fixed 13-class material proof                             all PASS
clean preservation / ambiguity retention                 no regression
single-client first-touch and hot p99                    <=5 ms
20-client queue-inclusive p99                            <=5 ms
printable-thread heavy synchronous work                     0
Space lost / duplicate / glued separator                    0 / 0 / 0
verifier and atomic route regression                        0
```

Context-settlement cases are not counted as lexical Slice 8 failures. They must
remain `Tied/ABSTAIN` until Slice 9 independently promotes exact context bytes.
Productive morphology top-1 is likewise not forced when multiple compatible
slots exist; target retention is an L2 requirement and contextual selection is
an L3 requirement.

## Consequences

The frame carrier expands the request contract and every constructor must be
updated explicitly. This is intentional: silently defaulting a missing frame to
synthetic identities would make tests pass while leaving production authority
unsound. Context-neutral material remains cacheable; volatile frame binding and
cohort settlement must stay bounded and must not trigger L1.1, V13 or V90
loading on Space.

The installed `1.0.43` runtime, its packages and global `ibus-daemon` stay
unchanged until 8A/8B software proofs and the full 8C promotion gate pass.

## Slice 8A Closure, 2026-08-24

Slice 8A adds library-owned immutable `LexicalAuthorityFrameV1` and
`LexicalAuthorityConfigIdentityV1`. The existing IBus `InputFrameIdentity`
converts by cloning every exact field; no text, focus, layout, capability or
config value is reconstructed from `original: &str`. The same borrowed frame
is carried through `ActiveCompositionAutocorrectRequest`, `InputGateRequest`
and `CorrectionRequest`. Every non-IME caller explicitly supplies `None`.

Measured on `e@192.168.3.94` through `scripts/cargo-guard.sh`:

```text
cargo check --lib --bin lay-ibus-engine                         PASS
cargo check --all-targets                                      PASS
engine -> library frame field parity                           PASS 13/13
InputGate -> CorrectionRequest presence and pointer parity      PASS
None remains None                                               PASS
deterministic candidate/decision differential parity            PASS
existing stale identity matrix                                  PASS 13/13
stale lock/publication/Space matrix                              PASS
```

The broad environment-dependent `ime_correction::tests` run is not a Slice 8A
PASS: it compiled, then reported `13/40` passed and `27/40` failed because the
isolated proof checkout did not reproduce the installed model/source-id
environment. Those failures were not used to claim either regression or
quality. Slice 8A instead proves its exact transport invariant and leaves the
field unread by candidate generation and settlement; the fixed Productive
quality and latency gates remain required after Slice 8B integration.

Runtime authority changed: `false`. Candidate packages changed: `false`.
Installed processes changed: `false`. Global `ibus-daemon` PID `2076194` was
not restarted. Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8a-frame-carrier-receipt.json`

## Slice 8B V5 Diagnostic, 2026-08-24

V5 separated the exact observed surface from the replacement target set,
integrity-bound its grounded L1.1 roots through `PreparedOriginalMaterialV1`,
reused the production lexical-token extractor in the fixed proof and moved
canonical L2/Productive V90 admission before the timed workers. The legacy
field authority remained the only live authority.

Remote compilation and focused tests on `e@192.168.3.94` passed:

```text
cargo check --lib --bin lay-nanda-wave-train                    PASS
material-frame tests                                          11 / 11
live cohort tests                                              10 / 10
production punctuation/token parity                            1 / 1
```

The fixed `13x1` production-route smoke passed the scoped comparator gate:

```text
attempted lemma-heldout fields                                     13
production fields                                                  13
cohort status READY                                             13 / 13
candidate-retention failures                                        0
grounded L1.1 losses                                                 0
legacy-decision parity failures                                      0
runtime authority changed                                        false
```

The fixed `13x100` run then rejected the V5 denominator assumption. It
attempted all `1,300` immutable lemma-heldout fields, but the production bridge
created a canonical field for only `918`. Every created field compared cleanly;
`382` attempts never reached the comparator:

```text
attempted fixed fields                                          1,300
production fields                                                 918
cohort status READY                                             918 / 918
candidate-retention / grounded-L1.1 / legacy-parity failures     0 / 0 / 0
EmptyL11Lattice before field construction                           377
unsupported production lexical token                                 5
wall time / CPU                                            37.64 s / 1019%
peak RSS                                                     935,816 KiB
queue-inclusive produced-field p50 / p95 / p99       23.186 / 115.118 / 179.298 ms
runtime authority changed                                        false
```

The `382` absent fields are upstream birth/coverage observations, not cohort
readout losses. They are distributed across seven damage classes plus five
layout-projection inputs. Treating them as successful comparisons would hide
coverage debt; treating them as comparator failures would make Slice 8B claim
ownership of candidate birth, contrary to the frozen first-loss split.

V5 verdict: `FAIL_MIXED_DENOMINATOR_RUNTIME_UNCHANGED`. The `13x1` receipt is a
scoped smoke PASS only. The full report remains
`FAIL_measured_shadow_gates`; no latency, promotion or deployment PASS is
claimed.

The next paper revision must preserve both denominators in one report:

```text
all fixed attempts = produced fields + explicit no-field observations
all produced fields must be READY
all produced replacement candidates must be retained
all produced grounded L1.1 roots must remain retained or original-bound
upstream no-field observations remain visible and cannot become authority
```

Exact receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v5-smoke-13x1.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v5-smoke-13x1.time.txt
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v5-fixed-13x100.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v5-fixed-13x100.time.txt
```

## Slice 8B V6 Denominator Separation, 2026-08-24

V6 changes proof accounting only. Every immutable lemma-heldout attempt has
exactly one typed outcome:

```text
ProducedField | NoField | Error
```

Only `UnsupportedInput` and `EmptyL11Lattice` are `NoField`. Service, package,
cache, bridge and readout failures remain `Error`. Comparator quality is gated
only over `ProducedField`, while overall production coverage still uses the
complete fixed denominator. `promotion_eligible` remains hard-coded `false` in
this observation-only slice.

The implementation preflight returned `READY_TO_IMPLEMENT`. Remote checks on
`e@192.168.3.94`, all through `scripts/cargo-guard.sh`, passed:

```text
cargo check --lib --bin lay-nanda-wave-train                    PASS
live cohort denominator tests                                  2 / 2
release lay-nanda-wave-train bytes                         11,595,640
release SHA-256                 b7d8f47f9a01bd8a7d89b8cd3d1d23a3958eed94d22c155e8c4ed06204364c0e
```

The first V6 smoke was rejected before any fixed claim. It counted only the
source event `proof_identity`, although one immutable event may legally produce
several damage-class cases. The result was `6` unique identities for `13`
attempts. The approved preflight and sampler contract define a proof attempt as
`(damage_class, proof_identity)`. V6 was corrected to that pair; the existing
duplicate test still rejects the same pair twice, while the conservation test
now proves that one event in two different classes represents two attempts.

The corrected fixed `13x1` smoke passed its complete small denominator:

```text
expected / attempted / unique                                  13 / 13 / 13
duplicate identities / errors                                        0 / 0
ProducedField / NoField                                            13 / 0
READY                                                            13 / 13
candidate-retention / grounded-L1.1 / legacy-parity failures      0 / 0 / 0
scoped comparator / smoke coverage                              PASS / PASS
runtime authority changed                                           false
```

The fixed `13x100` proof then produced the required separated verdict:

```text
expected / attempted / unique                           1,300 / 1,300 / 1,300
duplicate identities / errors                                        0 / 0
ProducedField / NoField                                           918 / 382
READY                                                            918 / 918
candidate-retention / grounded-L1.1 / legacy-parity failures      0 / 0 / 0
EmptyL11Lattice / UnsupportedInput                               377 / 5
scoped produced-field comparator                                    PASS
overall production field coverage                                   FAIL
promotion eligible                                                  false
wall time / CPU                                            37.04 s / 1039%
peak RSS                                                     938,372 KiB
produced-field p50 / p95 / p99 / max       24.085 / 111.808 / 197.421 / 455.680 ms
NoField p50 / p95 / p99 / max              34.617 / 109.607 / 195.442 / 441.115 ms
runtime authority changed                                           false
```

V5-to-V6 stable projection is exact. Both revisions produced `918` fields,
`918 READY`, `382` missing/no-field observations split as `377 + 5`, and zero
candidate-retention, grounded-L1.1 or legacy-parity failures. Candidate birth,
field generation and cohort computation did not change; only the report schema
and outcome partition changed.

V6 verdict scope is
`PASS_SLICE8B_PRODUCED_FIELD_COMPARATOR_UPSTREAM_COVERAGE_OPEN`. It is not an
overall Slice 8B promotion PASS. This experiment did not test or change live
authority, physical typing, candidate packages, verifier behavior, installed
processes, single-client `<=5 ms`, 20-client `<=5 ms`, or the `382` upstream
birth/coverage failures. The measured queue-inclusive latency also fails the
Slice 8 gate. Slice 8C must not start until coverage ownership and latency have
separate passing receipts.

The temporary remote L1.1 service used an isolated socket, reported zero query
failures and was stopped after the proof. Installed Lay `1.0.43`, its packages,
`lay-daemon`, `lay-ibus-engine` and global `ibus-daemon` were not restarted or
modified.

Exact receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/implementation-preflight-v6-denominator-separation.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v6-smoke-13x1-attempt1-identity-key-fail.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v6-smoke-13x1.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v6-smoke-13x1.time.txt
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v6-fixed-13x100.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v6-fixed-13x100.time.txt
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v5-v6-stable-projection.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v6-l11-stats.json
```

## Slice 8B V7 Coverage Provenance Contract, 2026-08-24

V6 proved that all `918` produced fields satisfy the scoped comparator, but it
did not preserve enough identity-level evidence to explain the `382` NoField
outcomes. The report retained only sixteen samples. Selecting a coverage fix
from those samples would be an unsupported extrapolation.

V7 is measurement-only. For every NoField attempt it records four independent
facts already owned by the fixed proof route:

```text
target exact in active L1.1 V9
target exact in canonical L2 V13
target paradigm covered by Productive V90 hypothesis
target exact surface born by Productive V90 for this damaged case
```

The last two facts are not synonyms. Hypothesis coverage means that a compatible
trained paradigm exists. Exact birth additionally proves that the existing
case-bound Productive traversal crystallized the required target surface. V7
copies both facts from the existing proof evaluation; it does not run a second
oracle route.

The exact-surface checks use the existing `ExactL11SurfaceIndexV1` and
`StandaloneL2Field::form_ref_for_surface` indexes over the unchanged normalized
target bytes. No fuzzy lookup, target rewrite, target-conditioned production
input or class-specific exception is permitted.

The fixed measurement gate is conjunctive:

```text
ProducedField + NoField + Error                         = 1,300
unique (damage_class, proof_identity)                  = 1,300
complete NoField provenance records                    = NoField
sum of every joint provenance bucket                   = NoField
sum of provenance buckets by class and availability    = NoField
V6/V7 stable outcome and comparator projection         = exact
runtime_authority_changed                               = false
promotion_eligible                                      = false
```

`UnsupportedInput` remains separate from `EmptyL11Lattice`; it is not evidence
of an L1.1 coverage defect. Latency remains outside this measurement verdict.
No coverage repair, Slice 8C work, package rebuild, deployment, process restart,
version change or installed-runtime mutation is authorized by V7.

The first implementation preflight was retained as
`BLOCKED_BEFORE_CODE`: its forbidden side effects lacked complete static scan
coverage. V2 added the missing tripwires without removing a prohibition and
returned `READY_TO_IMPLEMENT`:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/implementation-preflight-v7-coverage-provenance.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/implementation-preflight-v7-coverage-provenance-v2.json
```

The remote focused compile and three V7 unit tests passed. The fixed `13x1`
smoke passed `13/13`, and the single allowed fixed `13x100` completed with this
conserved denominator:

```text
attempted / expected                                             1,300 / 1,300
ProducedField / NoField / Error                                    918 / 382 / 0
EmptyL11Lattice / UnsupportedInput                                   377 / 5
produced-field scoped comparator                                    918 / 918 PASS
candidate-retention / grounded-L1.1 / legacy-parity failures           0 / 0 / 0
```

All `382` NoField records have complete provenance. Their independent target
support is:

```text
target exact in L1.1 V9 / absent                                 1 / 381
target exact in canonical V13 / absent                          382 / 0
Productive V90 hypothesis covered / absent                      377 / 5
Productive V90 exact target born / not born                     378 / 4
```

The dominant joint bucket contains `376/382` records where L1.1 lacks the exact
target while V13 contains it and V90 both covers its hypothesis and births its
exact surface. Within `EmptyL11Lattice`, `372/377` have that complete V90
support, and one additional record has an exact V90 birth without hypothesis
coverage. Therefore the dominant coverage failure is not `top-k=32`: the
production bridge requires an L1.1 contour seed before invoking material that
already proves it can crystallize the target.

The queue-inclusive bridge latency also fails the Slice 8 contract:

```text
ProducedField p50 / p95 / p99 / max    24.526 / 113.792 / 219.956 / 439.441 ms
NoField p50 / p95 / p99 / max           32.615 / 119.774 / 167.482 / 207.894 ms
required single-client and multi-client gate                                  <=5 ms
```

V6 to V7 outcome projection is exact, and every provenance partition conserves
all `382` NoField records. V7 passed only its measurement contract. It did not
test a repaired coverage owner, single-client or multi-client product latency,
physical typing, or authority transfer. Runtime authority changed: `false`.
Promotion eligible: `false`.

Exact receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v7-smoke-13x1.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v7-fixed-13x100.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v6-v7-stable-projection.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v7-provenance-conservation.json
```

The next allowed action is a paper contract and structural critique for one
typed contour material owner. No new implementation or heavy proof is allowed
until that contract reaches `READY_TO_IMPLEMENT`.

## Slice 8B V8 Typed Contour Material Paper Contract, 2026-08-24

### V7 claim boundary correction

V7 proves two different facts which must not be collapsed:

1. Canonical V13 contains all `382` NoField target surfaces.
2. Given the proof-selected target lemma identity, target-blind masking of its
   target form still lets V90 crystallize the exact surface in `378/382` cases.

The second fact does **not** prove target-blind lemma discovery. The fixed
productive proof begins `build_groundings` from `valid_targets` and uses the
target lemma identity to select its lexical observation before masking the
target form and slot. That is a valid paradigm-completion proof, but it cannot
be promoted into evidence that the production bridge can discover the same
lemma from the damaged token.

Therefore V7 authorizes neither direct cold-binding reuse nor a Productive
owner flip. A separate target-blind discovery denominator is mandatory.

### One-owner runtime contract

The intended route is:

```text
canonical token and context
-> one typed material acquisition owner
   -> L1.1 bounded contour evidence
   -> canonical V13 exact/layout/single-edit evidence
   -> bounded V13 compound contour evidence only when no grounded identity exists
   -> one deduplicated identity and provenance set
-> one Productive V90 field preparation
-> one immutable local lattice
-> common L3 context scoring
-> DecisionCore
-> verifier
-> mutation owner
```

The staged V13 compound lane is a material producer, not fallback authority. It
cannot emit a verdict, bypass Productive V90, rank against a second field, or
apply text. Every discovered surface is `Born` evidence until it is merged into
the same prepared material and processed by the same downstream readout.

The existing `EmptyL11Lattice` return moves after typed material acquisition.
It remains valid only when all target-blind material producers return no
grounded identity. `UnsupportedInput` remains an input-policy result and is not
silently reclassified as a coverage failure.

### Required discovery audit before runtime code

For each of the saved `382` V7 NoField records, one proof-only audit must:

```text
generate V13 contour births from damaged_surface without reading target_surface
map every born form_ref to its canonical lemma identities
only then decode target_surface and obtain its lemma identities for comparison
record exact-target discovery, same-lemma discovery and no-target-lemma discovery
partition results by availability and damage class
record births, exact lookups, operator steps and closed-call latency
```

The audit must preserve all `382` attempt identities exactly and must prove that
changing or removing `target_surface` cannot change the generated birth set,
work counters or latency path. Target bytes are labels only after generation.

The next runtime implementation is admitted only if the audit establishes one
common discovery mechanism for the `377` `EmptyL11Lattice` cases. If target
lemma discovery remains incomplete, the failure buckets return to paper
analysis; limits, coefficients, literal forms and per-class runtime branches
must not be added.

### Latency and cache contract

The full compound generator measured `4,230` exact lookups at p50 and `11,289`
at p99 in the existing material proof. Those are work counts, not milliseconds,
and therefore are not a latency PASS. The runtime design must satisfy all of:

```text
ordinary seeded route does not execute compound enumeration
compound enumeration has an explicit deterministic work ceiling
cache identity includes observed contour, typed-birth digest and V13/V90 hashes
cache reuse cannot cross token, contour, package generation or normalization identity
incomplete or overflow material cannot produce singleton authority
single-client p99 <= 5 ms
20-client p99 <= 5 ms
```

The current queue-inclusive `24.526 ms` produced-field p50 and `219.956 ms` p99
fail this contract. Cache hits, producer calls and waiter calls must be reported
separately; averaging them together cannot satisfy the gate.

### Rejected shortcuts

- increasing L1.1 or L2 `top-k`;
- treating V90 target birth after target-lemma selection as discovery evidence;
- running the complete compound generator on every ordinary token;
- a second contour verdict or a fallback winner beside Productive V90;
- promoting `Born` contour evidence directly to `Winner`;
- weakening completeness, SafetyGate, DecisionCore or verifier checks;
- another full `13x100` merely to choose the design.

Measured facts used by this contract are V7 facts already recorded above.
Not yet tested: target-blind same-lemma discovery, discovery latency, the merged
material implementation, production coverage after merge, single/multi-client
latency, physical typing and authority transfer. Runtime authority changed:
`false`. Promotion eligible: `false`.

### V8 measured target-blind audit

The required audit ran once over the conserved V7 `NoField` denominator. The
generator received only `damaged_surface`; `target_surface` was decoded after
the complete birth set and work counters existed. All partitions conserved:

```text
records / unique attempt identities                           382 / 382
exact target discovered                                       53 / 382
same-lemma proxy only                                           3 / 382
no target lemma discovered                                    326 / 382
target form absent from canonical V13                           0 / 382
target lemma absent from canonical V13                          0 / 382
overflow                                                        0 / 382
```

The `377` `EmptyL11Lattice` records partition as `53` exact target, `3`
same-lemma proxy only and `321` without any target-lemma discovery. The five
`UnsupportedInput` layout records produced no target lemma and remain outside
the supported typed-input policy.

The result is class-structural rather than a small residual:

```text
damage class                    exact target   proxy only   no target lemma
punctuation_suffix                    36/36          0              0
suffix_truncation                     17/17          0              0
double_substitution                    0/66          1             65
repeated_fragment                      0/79          1             78
sparse_multi_omission                  0/54          1             53
non_adjacent_transposition             0/59          0             59
omission_transposition                 0/66          0             66
layout_projection                      0/5           0              5
```

The closed audit performed `1,690,521` exact grounding lookups/operator steps,
with a maximum of `11,926` for one record. It emitted `390` births in total and
at most `35` for one record. The unoptimized test binary measured enumeration
at p50 `138.265 ms`, p95 `425.939 ms`, p99 `517.730 ms` and maximum
`707.087 ms`. These timings exclude queueing, Productive V90, L3 and IPC. They
describe the rejected proof mechanism and are **not** a product latency gate or
a projection of optimized runtime latency.

Receipt:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v8-target-blind-contour-lemma-audit.json
SHA-256 980be4ba16acbd12f67f81c9914d444bb84b28bf45d79c952f9121f479b071d1
```

Tested: target-blind discovery coverage, form-to-lemma mapping, attempt and
partition conservation, bounded-birth overflow, proof work and closed debug
latency. Not tested: optimized latency, a new V13 decoder/index, Productive V90
readout over newly discovered material, common L3 selection, physical input or
authority transfer. Runtime authority changed: `false`.

Verdict:

```text
REJECT_CURRENT_COMPOUND_CONTOUR_AS_DISCOVERY_OWNER
```

The compound generator is exact for punctuation removal and suffix completion,
but it discovers the target lemma in only `56/382` records and misses it in
`326/382`. Increasing its limits would spend more work on a mechanism that has
no recovery path for the dominant damage classes. It must not be copied into
the production bridge, optimized into a hot route or used as fallback
authority.

The next admitted work returns to paper design. It must define one target-blind
exact typed peak search directly over canonical V13 identities:

```text
damaged token
-> exact typed V13 peak search
-> bounded V13 form/lemma lattice + completeness certificate
-> Productive V90 morphology expansion
-> common L3 contextual selection
-> DecisionCore -> verifier -> output
```

Phase 8I supplies proven typed-basin semantics, bounded projection and
certificate obligations. It does not by itself supply a V13 decoder/index,
package/RSS admissibility or `p99 <= 5 ms`; those are separate V9 design gates.

## Slice 8B V9 Exact V13 Typed-Peak Paper Contract, 2026-08-24

### Decision

V9 will test one target-blind lexical discovery mechanism:

```text
immutable canonical V13 surfaces
-> one derived minimal lexical DAFSA sidecar

normalized damaged token
-> one Phase 8I typed-edit automaton
-> exact automaton intersection with the V13 DAFSA
-> V13 form_ref + all morphology bindings + typed witness
-> one bounded Born lattice with a completeness result
-> Productive V90
-> common L3 -> DecisionCore -> verifier -> output
```

The DAFSA is an address index over existing V13 identities. It is not a second
lexicon, morphology model, scorer or authority owner. V13, V90 and L1.1 package
bytes remain immutable during the proof slice.

The current V13 `RuntimeLemmaWaveIndex` remains useful as non-authoritative
evidence after a form or lemma is admitted. It cannot own discovery because its
current route:

- limits atom relations before complete typed verification;
- truncates lemma ranking before resolving all compatible forms;
- uses wave-band probes and may fall back to an exhaustive lemma scan;
- has no form-level typed witness or unseen-frontier completeness certificate.

Using it as the discovery owner would repeat the early-birth defect under a
different name. It may score retained material, but it may not erase a DAFSA-
grounded form or manufacture singleton authority.

### Why a minimal DAFSA sidecar

The V7/V8 canonical package is exactly:

```text
path    /home/e/.local/share/lay/nanda_wave/l2/LAY-L2-RU-FULL-v13.bin
bytes   140,556,462
SHA-256 cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
forms   1,875,032
```

Its compact decoder is a sorted block-front-coded surface store. It supports
exact binary search and bounded block reconstruction, but it has no child
transition graph for exact typed traversal. Scanning all blocks per query fails
the latency contract. Rebuilding all decoded strings into per-process maps or a
pointer-rich startup trie would duplicate the largest lexical material and
fail the RAM objective.

A minimal deterministic acyclic finite-state acceptor is selected because it:

1. is derived in one sequential pass over the already sorted immutable V13
   surfaces;
2. stores transition topology and terminal language counts, not duplicate
   surface strings;
3. supports exact intersection with the Phase 8I typed-edit automaton;
4. can be mmap-backed and shared by daemon and IBus processes;
5. can recover canonical `form_ref` by lexicographic rank without a terminal
   `u32` for every surface.

For a DAFSA state `s`, `suffix_count(s)` is the number of accepted suffixes.
During traversal, the form rank is the root terminal contribution plus the
`suffix_count` of every lower-labelled sibling edge. Because the root language
must equal the strictly ordered V13 surface sequence, the terminal rank must
equal `form_ref`. This is a proof obligation, not an assumption.

The compiler must reject invalid UTF-8, non-increasing surfaces, duplicate
surface records not represented by one canonical form identity, state or count
overflow, nondeterministic transitions and any terminal-rank mismatch. A
multi-lemma surface remains one `form_ref`; all of its V13 bindings are emitted.

### Identity and package contract

The sidecar header must bind:

```text
magic and format version
canonical V13 byte length and SHA-256
canonical V13 form count and binding count
surface normalization semantics version
typed-edit semantics digest
dense symbol-table digest
state, edge and terminal counts
root language count
payload checksum
```

Cache identity is:

```text
normalized observed token
+ input script/layout policy
+ typed-edit semantics digest
+ V13 SHA-256
+ DAFSA sidecar SHA-256
```

Productive and contextual cache identity additionally includes V90 and context
frame identities. A token-only search cache must not cache a Productive or L3
decision. An unresolved or overflow result cannot be reloaded as certified.

### Search semantics

The query is normalized once into a typed token frame. The edit automaton uses
the Phase 7D/8I operator semantics, not a new distance function:

```text
identity
input deletion / extra typed symbol
target insertion / missing symbol
character and keyboard substitution
keyboard-layout projection
adjacent and non-adjacent transposition
double substitution
repeated fragment
omission plus transposition
sparse multi-omission
prefix and suffix truncation
punctuation suffix
```

The lexical product state is:

```text
(dafsa_state,
 input_position,
 operator_program,
 position_certificate,
 path_context_ref,
 lexicographic_rank_prefix,
 accumulated_geometry)
```

No corrected surface string is constructed during search. A terminal emits:

```text
V13TypedPeak
+ form_ref
+ every (lemma_id, feature_mask) binding
+ minimum typed geometry
+ all authority-distinct operator witnesses
+ provenance = V13ExactTypedPeak
+ verdict_membership = Born
```

States may merge only when DAFSA node, input position, operator program,
position certificate, exact path context, rank prefix and future language are
identical and one accumulated geometry provably dominates the other without
removing an authority-distinct witness. Scalar edit distance alone is not a
valid merge key. Traversal order must not change terminal or certificate bytes.

`path_context_ref` names an immutable query-local predecessor record containing
the traversed symbol and previous path reference. It is required because a
minimal DAFSA may merge suffix-equivalent states that have different lexical
prefixes, while `repeated_fragment` certificates depend on an earlier target
pair and its exact position. A DAFSA state or rank prefix alone cannot replace
this evidence. The path arena is structural search state, not a generated
candidate string; it is bounded by the same scratch and closure contract.

The existing `typed_edit_traversal.rs` operator algebra is the semantic donor.
V9 must extract a read-only lexical-automaton adapter or another equivalence-
proved shared core; copying the operator implementation into L2 is forbidden.
L1 terminal IDs, V13 form refs and V13 lemma IDs remain distinct types.

### Boundedness and completeness

Let `G(Q)` be every V13 form reachable from query `Q` under the admitted typed
operator algebra. The proof implementation first supports exhaustive certified
intersection:

```rust
enum V13SearchCompleteness {
    CertifiedExhaustive(V13CompletenessCertificate),
    Unresolved(V13UnresolvedReason),
}
```

The first prototype may not claim `CertifiedTopK`. If the frontier, terminal
set, wall deadline or scratch budget is exhausted before the product graph is
closed, the result is `Unresolved` and has no automatic edit authority. It may
emit diagnostic suggestions, but it cannot silently fall back to compound
contours, current lemma-wave ranking, legacy L2 or a larger top-k.

The certificate binds at least:

```text
query and package/cache identity
root/state/edge counts
expanded and dominated product states
terminal form and lemma counts
operator-witness counts
frontier empty at closure
overflow and unresolved reason
terminal lattice fingerprint
```

Prefix/suffix continuation may expose many exact terminals. V9 does not hide
this with a cap: exhaustive closure must fit the declared budgets, otherwise
the query remains unresolved. A future `CertifiedTopK` requires separately
proved subtree score envelopes and tie completeness; it is outside the first
prototype.

### Resource contract

The proof sidecar is derived from V13 once on the remote build host. It does not
read the training corpus, rebuild V13/V90, reinduce morphology or perform a
`13x100` proof. The first physical screen is:

```text
sidecar bytes                                      <= 32 MiB
owned loader metadata per process                   <= 4 MiB
aggregate daemon + IBus PSS delta                   <= 40 MiB
per-query scratch maximum                           <= 512 KiB
single-client exact search p99                       <= 3 ms
single-client material acquisition p99               <= 5 ms
20-client material acquisition p99                   <= 5 ms
queue timeout, frontier truncation, false certificate       0
```

The `3 ms` component budget reserves the remainder of the existing `5 ms`
material gate for form-to-binding resolution and transfer to Productive V90.
Release-mode closed-call time, queue wait and end-to-end time are reported
separately. Debug timing cannot pass a latency gate. mmap file bytes, resident
shared pages, private RSS and aggregate PSS are reported separately; summing
per-process RSS is not a physical-memory proof.

If the minimal DAFSA exceeds the sidecar or PSS screen, V9 is rejected before
query optimization. The next paper alternative would be a succinct LOUDS/rank-
select trie over the same immutable form sequence, not a larger RAM budget or
a smaller lexical denominator.

### Proof ladder

#### Gate V9-A: index identity

```text
all V13 surfaces decoded                         1,875,032 / 1,875,032
strict surface order and uniqueness                        PASS
DAFSA root language count                        1,875,032
terminal rank == form_ref                        1,875,032 / 1,875,032
surface round-trip                               1,875,032 / 1,875,032
multi-lemma binding parity                                 exact
nondeterministic edges / count overflow / corruption       0 / 0 / 0
sidecar and loader resource screen                         PASS
```

#### Gate V9-B: independent semantic oracle

Use exhaustive tiny lexicons and generated operator matrices. The oracle scans
all tiny surfaces and independently recomputes the admitted typed relation.

```text
terminal-set recall for every operator family              100%
extra terminals                                                0
operator-certificate mismatches                                0
schedule/permutation byte mismatches                            0
generated runtime strings                                      0
literal word or case-class branches                            0
fault-injected false completeness certificates                 0
```

#### Gate V9-C: conserved V8 failure denominator

Run target-blind search once over the same `382` attempt identities. Target
labels enter only after search output and counters are frozen.

```text
records and unique attempt identities                     382 / 382
target form retained                                      382 / 382
target lemma retained                                     382 / 382
emitted form failing independent witness verification             0
overflow / unresolved / false completeness certificate       0 / 0 / 0
per-class target-form and target-lemma retention                 100%
release search p99                                             <= 3 ms
release material p99                                           <= 5 ms
```

This gate proves lexical discovery and typed evidence only. It does not prove
that Productive V90 or L3 selects the target.

#### Gate V9-D: one downstream material route

Only after V9-A/B/C pass may a separate implementation preflight admit:

```text
V13TypedPeak Born lattice
-> one Productive V90 preparation
-> one common L3 readout
-> one DecisionCore
-> one verifier
```

Required downstream proof keeps lexical retention, morphology generation,
contextual selection, authority and safety as separate denominators. A target
retained by V9 but rejected by context is an L3/authority result, not permission
to alter DAFSA search. Runtime owner transfer, physical input, deployment,
version bump and push remain outside V9-A/B/C.

### Rejected alternatives and shortcuts

- current compound contour as discovery owner: `56/382` lemma discovery;
- current lemma-wave atom/band ranker as authority: no complete typed frontier;
- query-time scan of `1,875,032` forms or all decoder blocks;
- per-process decoded string table or pointer-rich full trie;
- copying Phase 8I operator code into an independent L2 implementation;
- target-conditioned build, search, work budget or cache key;
- a class-specific route, literal surface, suffix or test identity;
- top-k growth, queue growth or timeout growth as a coverage repair;
- a second readout, fallback winner or direct `Born -> Winner` promotion;
- weakening SafetyGate, DecisionCore, edit-plan validation or verifier;
- rebuilding V13/V90 or running another full `13x100` to choose the design.

### Current V9 claim boundary

Measured now: V13 byte identity, size and form count; existing Phase 8I typed
semantics; V8 discovery failure; existing V13 atom/wave index ownership.

Estimated now: minimal DAFSA sidecar, PSS and latency feasibility. These are
gates, not accepted facts.

Not tested: V13 DAFSA construction, terminal-rank parity, semantic-oracle parity,
V8-denominator coverage, release latency, sidecar/PSS budget, Productive V90
integration, L3 selection, physical input or deployment.

Runtime authority changed: `false`. Installed Lay changed: `false`. Promotion
eligible: `false`. The next allowed action is structural critique and an
implementation preflight for V9-A/B/C proof-only code.

### V9 pre-gate critique and repaired boundaries

The first paper pass was challenged against the current Phase 8I traversal and
V13 storage contracts. The following failure modes are now explicit:

1. **Merged-parent loss.** A minimal DAFSA has no unique parent chain. The
   original proposal would therefore lose repeated-fragment source evidence.
   Repair: path-dependent predecessor state is part of the product identity and
   oracle comparison.
2. **Form-rank aliasing.** A shared final DAFSA state identifies a suffix
   language, not one form. Repair: terminal identity is the path-dependent
   lexicographic rank, checked against every V13 `form_ref`.
3. **Hidden truncation.** Existing V13 atom-relation and lemma limits cannot be
   used as completeness bounds. Repair: the first prototype certifies only an
   empty exhaustive frontier; any budget hit is `Unresolved`.
4. **Large continuation basins.** Prefix/suffix operations can close over many
   forms. Repair: no automatic authority and no target-dropping cap on
   overflow. `CertifiedTopK` remains a separate future theorem.
5. **Package duplication.** A startup trie or decoded-string map could pass
   semantics while failing product memory. Repair: mmap sidecar, private/PSS
   accounting and hard package screens are conjunctive gates.
6. **Proof-label leakage.** `382/382` target retention is invalid if the target
   influences traversal or deadlines. Repair: search output, work counters and
   latency close before target bytes are decoded, as in V8.
7. **Authority collapse.** Discovery success does not prove morphology or
   context selection. Repair: V9-C ends at a `Born` lattice; V9-D is a separate
   downstream preflight and denominator.

After these repairs the paper route is coherent enough for a structural gate.
It is not yet implementation authority. A gate `WATCH` or `VETO`, or a
preflight other than `READY_TO_IMPLEMENT`, returns the work to paper repair.

## Slice 8B V9 Measured A/B/C Result, 2026-08-24

One and only one admitted remote V9 A/B/C run was executed against the exact
conserved V13 and V7 artifacts. It did not rebuild V13/V90, run `13x100`, alter
runtime authority, install a package or restart Lay.

```text
V13 SHA-256       cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
V7 SHA-256        33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
receipt SHA-256   956a329e29811b5fea8b7afce1912d2d2b5040516cbcf3160d926b969bbc3a04
sidecar SHA-256   bce27cf8205216dbef4b2c3cfd69a5692b0070d433b73f43c043f5a2eedc6b03
```

Receipt:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v9-exact-v13-typed-peak-abc.json
```

### V9-A index identity: PASS

```text
decoded / terminal-rank parity             1,875,032 / 1,875,032
rank or surface mismatches                                      0
strict surface order                                         PASS
states / edges                                    81,128 / 226,341
root language count                                      1,875,032
sidecar bytes                                      2,784,520 (2.66 MiB)
loader-owned metadata                                      104 B
binding parity                         3,255,785 / 3,255,785
multi-lemma surfaces                                      45,052
two-process aggregate PSS delta                       2,377 KiB
build / full rank roundtrip                         577 / 867 ms
```

The compactness hypothesis was conservative by more than an order of
magnitude. A minimal DAFSA is the accepted V13 address-index representation;
LOUDS is not needed for the measured package.

### V9-B semantic oracle: PASS

```text
generated cases / operator families                         52 / 13
terminal-set mismatches                                           0
certificate mismatches                                            0
schedule/permutation mismatches                                    0
corruption false accepts                                           0
false completeness certificates                                    0
maximum query scratch                                          6,992 B
```

The radius-three retrieval lane followed by the shared Phase 7D certificate
oracle is an exact implementation of the admitted semantic language on the
generated exhaustive matrix. Retrieval distance remains a non-authoritative
superset; certificates remain the only admission semantics.

### V9-C coverage: quality PASS, latency FAIL

```text
records                                                  382 / 382
target form retained                                     382 / 382
target lemma retained                                    382 / 382
false certificates                                                0
unresolved / overflow                                          0 / 0
maximum product states                                       35,590
maximum query scratch                                      17,920 B

single search p50 / p99 / max                    2.780 / 4.585 / 5.211 ms
single total p50 / p99 / max                     2.915 / 4.606 / 5.219 ms
20-client search p99 / total p99 / max          11.778 / 12.532 / 16.982 ms
```

The receipt reports `218` unique identities because the first V9 harness
incorrectly counted bare `proof_identity`. The source V7 contract defines an
attempt as `(damage_class, proof_identity)`; that exact composite has
`382/382` unique members, matching the V7 and V8 conservation code. This is a
proof-harness denominator defect, not a duplicate data record or a quality
failure. The harness is corrected before any later proof.

The global V9 verdict remains `FAIL_V9_A_B_C`: exact coverage cannot hide the
failed component and 20-client latency gates. V9-D, runtime ownership,
deployment, version bump and push remain prohibited.

### First shared latency mechanism

The fixed `382` damaged tokens have:

```text
character length p50 / p95 / p99 / max              12 / 18 / 20 / 22
```

V9 computes all `m + 1` Levenshtein cells for every traversed DAFSA edge and
then performs another random state read to obtain each child suffix count for
lexicographic rank. At p99 length `20`, only the radius-three diagonal band can
possibly remain within the admitted retrieval language, yet V9 computes `21`
cells instead of at most `7`. Under 20 concurrent clients this unnecessary row
copy and child-state traffic becomes CPU/cache bandwidth pressure. Queue
growth, a larger latency budget, fewer clients or dropped terminals would hide
the mechanism and are rejected.

## V10 Banded Exact-Intersection Optimization Contract

V10 is not a new discovery model. It must preserve the V9 sidecar language,
terminal ranks, Phase 7D certificate bytes, completeness semantics and all
`382` target-blind outputs exactly. It changes only representation of bounded
dynamic-programming state and lexicographic rank transport:

```text
V9 full DP row of m + 1 cells
-> Ukkonen radius-r band [depth-r, depth+r]
-> at most 2r+1 = 7 live cells for r <= 3

V9 edge = (symbol, target_state)
-> V10 edge = (symbol, target_state, rank_delta)
-> child rank = parent rank + rank_delta
```

`rank_delta` is the current-state terminal contribution plus the language
counts of all lower-labelled siblings. The loader must independently recompute
and reject every incorrect delta. At the measured `226,341` edges, expanding
an edge from `8` to `12` bytes projects the sidecar to about `3,689,884 B`
(`3.52 MiB`), still far below the unchanged `32 MiB` gate.

The band recurrence must treat cells outside the diagonal as `r + 1`; it may
prune a branch only when every represented cell exceeds `r`. Query depth is
part of the product state. Tokens above the declared inline-symbol limit return
`Unresolved`; they do not fall back to full-row, legacy or field-wide search.

### V10 proof obligations

```text
V9 sidecar language and terminal ranks                         exact
full-row versus banded terminal set on generated matrices      exact
full-row versus banded certificate bytes                       exact
forward versus reverse traversal bytes                         exact
rank_delta loader recomputation mismatches                         0
fault-injected band/rank false completeness                        0
382 target forms / lemmas retained                         382 / 382
false certificates / unresolved                              0 / 0
single-client exact search p99                              <= 3 ms
single-client total material p99                            <= 5 ms
20-client total material p99                               <= 5 ms
sidecar / loader / PSS / scratch budgets                     unchanged
```

No coefficient, candidate cap, target-conditioned branch, literal surface,
class exception, timeout increase or verifier weakening is admitted. A V10
implementation and second remote proof require their own structural route and
implementation preflight. Until those pass, the measured V9 receipt is the
latest authority and promotion remains false.

### V10 code-route result, 2026-08-24

Tested: the proposed V10 proof-only execution and proof topology. The checked
route has one banded exact-peak producer, one read-only rank-delta sidecar, one
separate full-row proof oracle and one proof owner. Target labels have no path
to the search producer and enter only the post-search audit. The initial draft
was rejected before code because it incorrectly asked the gate to count
observation edges as proof edges; the repaired contract preserves all three
observation inputs without mixing their graph kinds.

Measured facts:

```text
route verdict                                      PASS
declared nodes / edges                            13 / 13
mixed execution/observation/proof issues                0
ready_for_implementation_preflight                   true
```

Not tested: band recurrence correctness, rank-delta bytes, V9/V10 output
parity, resource budgets, fixed `382` quality, runtime integration or physical
input. Verdict scope: structural design only.

Receipt:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v10-exact-v13-typed-peak-route.json
sha256 9c8583e9a802ab927c104dfc783f9a4f828f1c59ea77bb31660317f60d8650a4
```

Runtime authority changed: `false`. Installed Lay, V13, V90, daemon and IBus
remain outside V10 A/B/C.

## Slice 8B V10 Measured A/B/C Result, 2026-08-24

V10 was implemented only in the proof module. Local focused checks and the
post-edit observed route passed before one bounded remote A/B/C run. V9 was not
rerun, V13/V90/L1.1 packages were not rebuilt, and the installed Lay 1.0.43 was
not changed or restarted.

```text
source SHA-256    f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c
receipt SHA-256   08cccb3c9c63a24e9dc691958fde319de2a94b762fcb9f703ac342fcb72174e1
sidecar SHA-256   a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
```

Receipt:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v10-exact-v13-typed-peak-abc.json
```

### V10-A index identity: PASS

```text
decoded / terminal-rank parity             1,875,032 / 1,875,032
binding parity                              3,255,785 / 3,255,785
rank, surface or binding mismatches                                 0
states / edges                                      81,128 / 226,341
sidecar bytes                                        3,689,884 B
two-process aggregate PSS delta                         2,939 KiB
rank-delta corruption false accepts                             0
```

### V10-B exact-kernel parity: PASS

```text
generated cases / operator families                         52 / 13
full-row/banded terminal mismatches                               0
full-row/banded certificate mismatches                            0
full-row/banded completeness mismatches                           0
full-row/banded expanded-work mismatches                          0
schedule/permutation mismatches                                   0
false completeness certificates                                  0
maximum query scratch                                         1,712 B
```

### V10-C quality: PASS; latency: FAIL

```text
records / unique (damage_class, proof_identity)             382 / 382
target form / target lemma retained                          382 / 382
false certificates / unresolved                                  0 / 0
maximum expanded product states                                35,590
maximum query scratch                                          6,656 B

single search p50 / p99 / max                    2.730 / 3.910 / 4.184 ms
single total p50 / p99 / max                     2.823 / 4.133 / 4.365 ms
20-client search p99 / total p99 / max           8.991 / 10.127 / 30.445 ms
```

The global verdict is `FAIL_V10_A_B_C`. The band reduced V9 single-search p99
from `4.585` to `3.910 ms`, 20-client total p99 from `12.532` to `10.127 ms`,
and maximum fixed-denominator scratch from `17,920` to `6,656 B`. It did not
change the `35,590` maximum expanded product states and did not meet either the
`search p99 <= 3 ms` or `20-client total p99 <= 5 ms` gate. This falsifies the
hypothesis that DP-row storage was the dominant bottleneck. Further candidate
cuts, timeout changes, client-count reductions, queue drops or gate weakening
are prohibited. Work per preserved edge is the next shared mechanism.

Tested: sidecar identity, loader validation, full-row/banded parity, all fixed
quality denominators, proof-process PSS and proof latency. Not tested: runtime
integration, Productive V90 selection, L3 authority, physical input or product
multi-client latency. Runtime authority changed: `false`. Promotion eligible:
`false`.

## V11 Packed DAFSA x Deterministic Levenshtein Automaton Contract

### Problem statement and invariant boundary

V10 still executes a seven-cell recurrence for each visited DAFSA edge. The
same `(bounded Levenshtein state, input equality class)` transition is computed
many times because a minimal DAFSA shares suffix topology while path-dependent
form rank prevents merging the full search nodes. V11 may memoize the edit
transition, but it must not merge path-dependent DAFSA rank prefixes or change
which product nodes are visited.

The preserved language is:

```text
immutable V13 surfaces and bindings
-> Phase 7D target-blind retrieval lanes
-> exact radius-3 Levenshtein language
-> exact terminal form_ref ranks
-> unchanged Phase 7D certificate oracle
-> unchanged Born lattice
```

V11 changes two internal representations only:

```text
V10 12-byte state / 12-byte edge topology
-> V11 8-byte state / 8-byte edge topology + Unicode symbol table

V10 repeated band recurrence on every DAFSA edge
-> query-local deterministic Levenshtein automaton (DLA)
-> exact packed-DAFSA x DLA intersection by table lookup
```

No target label, expected surface, damage-class exception, coefficient,
candidate cap, timeout increase, fallback route or verifier change is admitted.

### Packed topology

The V10 graph has the following measured maxima:

```text
distinct Unicode symbols             34
maximum Unicode scalar             1,105
maximum target state              81,126
maximum rank_delta             1,874,537
maximum first_edge               226,311
maximum suffix_count           1,875,032
maximum edge_count                    32
```

They fit losslessly in this versioned layout:

```text
state u64: first_edge 24 | suffix_count 24 | edge_count 15 | terminal 1
edge  u64: symbol_ref 16 | target 24 | rank_delta 24
```

`symbol_ref` indexes one strictly increasing table of Unicode scalars. The
loader must validate every width, section boundary, symbol reference, target,
acyclic-order invariant, edge order, suffix count, rank delta, root language
count, package identity, semantics digest and payload checksum independently.
An overflow at compile time or a mismatch at load time is an error, never a
truncation. The projected fixed-package size is:

```text
header                                  256 B
81,128 states x 8 B                 649,024 B
226,341 edges x 8 B               1,810,728 B
34 Unicode scalars x 4 B               136 B
total                               2,460,144 B (2.35 MiB)
```

This is a cache-pressure optimization, not a new quality claim.

### Query-local DLA

For each Phase 7D retrieval lane, V11 constructs a deterministic automaton from
the same initial band row used by V10. Input symbols are partitioned only by
their equality relation to query symbols present in the immutable sidecar
alphabet, plus one `OTHER` class. Levenshtein substitution cost depends only on
that relation, so two symbols in one class are transition-equivalent. Class
construction is deterministic and target-blind.

Each DLA state stores the exact clipped V10 band state, including depth. Each
transition is computed once with the V10 recurrence and stored as a bounded
state id or `DEAD`. Terminal acceptance is the same query-column distance and
`DEAD` is legal only when every represented cell is above the lane radius.
DAFSA traversal then carries:

```text
(packed DAFSA state, path-dependent rank_prefix, DLA state id)
```

The DLA state may be shared; `rank_prefix` may not. The old banded kernel and
full-row kernel remain proof oracles and cannot be selected by the V11 query
route. Any DLA-state, transition-table, query-symbol, product-state, terminal,
scratch or wall budget exhaustion returns `Unresolved` with no lattice and no
fallback.

A target-blind sizing pass over the conserved 382 damaged queries projected:

```text
reachable DLA states       p50 897 | p95 1,485 | p99 1,769 | max 1,939
input classes               p50 10 | p95    14 | p99    15 | max    16
table bytes              p50 25,284 | p99 61,982 | max 77,560
computed transitions      p50 9,135 | p99 24,375 | max 31,024
```

These are design estimates, not acceptance measurements. They fit the existing
`512 KiB` scratch screen and predict that repeated edge-local recurrence becomes
a bounded construction followed by compact table reads. V11-C must measure DLA
build time, transition count, intersection time and total time separately.

### Alternatives and critique

1. **Packed topology only.** It reduces topology bytes by about one third but
   preserves the seven-cell recurrence on every visited edge. V10 needs a
   further `23.3%` single-search p99 reduction and `50.6%` 20-client total p99
   reduction. Packing alone has no measured basis for both, so it is rejected
   as the complete V11 mechanism.
2. **DLA on the V10 12-byte topology.** It removes repeated recurrence but
   retains avoidable mmap/cache traffic. It is semantically viable, but the
   combined packed layout has the same language and validation obligations and
   directly addresses the concurrency pressure exposed by V10.
3. **Packed topology plus query-local DLA.** This is selected. It reduces both
   per-edge compute and shared read bandwidth while preserving the exact search
   language, output ranks, traversal frontier and downstream certificates.
4. **SIMD/SWAR band recurrence.** It creates an architecture-specific kernel
   and parity route, complicates corruption and cross-machine proof, and still
   repeats work per edge. It is deferred unless the architecture-neutral DLA
   fails after one admitted proof.
5. **Myers score-only traversal.** A terminal score alone does not prove the
   exact prefix minimum needed for safe early pruning. Adding the missing state
   reconstructs an automaton with more complex bit-width and Unicode proofs, so
   it is not selected.
6. **Deletion/signature postings.** They change the birth language and package
   economics rather than optimize the accepted exact intersection. They are
   outside Slice 8B.

### Consequence analysis

**Quality and authority.** Packed records and DLA transitions are exact
representations of V10. V11 must preserve terminal refs, certificate bytes,
completeness, expanded product-state counts, all `382/382` target forms and
lemmas, and zero false certificates. The output remains a `Born` lattice; V11
does not gain morphology, context or runtime authority.

**Latency and concurrency.** DLA construction adds fixed query-local work
before traversal and may regress short or low-branching queries. The proof must
therefore report build, intersection, material and total latency, not only the
improved segment. Twenty independent clients each own a DLA and share only the
read-only mmap. No global cache, lock, worker queue or serialized warmup is
allowed.

**Memory and package.** The sidecar shrinks by a projected `1,229,740 B`; query
scratch grows from the V10 fixed maximum `6,656 B` toward the projected DLA
maximum. Both remain conjunctively screened: sidecar `<=32 MiB`, loader-owned
metadata `<=4 MiB`, two-process PSS delta `<=40 MiB`, query scratch `<=512 KiB`.
Heap capacity, hash/index storage and transition tables all count as scratch.

**Integrity and compatibility.** The packed sidecar requires a new magic and
format version. V10 bytes must be rejected, not reinterpreted. Temporary build
and atomic rename remain proof-only. An interrupted compile, transfer, load or
receipt write leaves V10 evidence, immutable V13/V90/L1.1 packages and the
installed runtime untouched.

**Unicode and normalization.** The sidecar symbol table stores Unicode scalar
values, not bytes or locale-dependent collation. Existing normalization and
Phase 7D semantics digests remain pinned. A query symbol absent from the table
maps to `OTHER`; it must not add a sidecar symbol or mutate the package.

**Maintainability.** Packing/validation, DLA construction, intersection and
proof oracles remain explicit owners inside the existing proof module. V11 may
not add a parallel runtime route or source-specific word rules. The banded
recurrence remains the single transition oracle used to build DLA transitions
and to prove parity; duplicate scoring logic is prohibited.

**Downstream effects.** The Born lattice schema and form refs remain unchanged,
so Productive V90, L3, DecisionCore and verifier behavior are outside V11-A/B/C.
They require the later V11-D/Productive gate before any authority flip.

### V11 proof obligations

```text
packed sidecar full language / terminal-rank parity                 exact
packed loader recomputation and valid-checksum corruption rejects      0
Unicode symbol-table order/reference corruption false accepts           0
DLA versus V10 band transition and terminal parity                     exact
DLA versus V10/full-row terminal refs and certificates                 exact
forward/reverse schedule bytes and expanded product states              exact
382 target forms / lemmas retained                                 382 / 382
false certificates / unresolved                                      0 / 0
single-client exact search p99                                      <= 3 ms
single-client total material p99                                    <= 5 ms
20-client total material p99                                       <= 5 ms
sidecar / loader / PSS / scratch budgets                       unchanged
runtime, installed package and process noninterference                  exact
```

The next allowed action is a V11 design code-route gate followed by a V11
implementation preflight. Code may start only after `PASS` and
`READY_TO_IMPLEMENT`. Exactly one bounded remote V11 A/B/C proof is admitted
after focused local checks and a post-edit observed-source route. A proof FAIL
returns to analysis; it does not authorize V12, repeated tuning runs, deployment,
version bump, push or runtime fallback.

### V11 design code-route result, 2026-08-24

The V11 design route passed with execution, observation and proof graphs kept
separate. The checked execution has one immutable sidecar producer, one
query-local DLA builder, one packed-intersection producer and one Born-lattice
output. V10 banded and full-row implementations are separate parity observers.
Target labels enter only the post-search audit.

```text
route verdict                                      PASS
declared nodes / edges                            17 / 18
route issues / warnings                             0 / 0
ready_for_implementation_preflight                    true
```

Receipt:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v11-packed-dafsa-dla-route.json
sha256 1a921b8ca1fd57870d181b8c71013ce8c875caba225659faa4343eec331bacd3
```

Tested: declared call cardinality and route-role separation. Not tested: source
implementation, packed bytes, DLA exactness, resource budgets, fixed quality,
runtime integration or physical input. Runtime authority changed: `false`.

### V11 implementation preflight V1 result

The first V11 implementation preflight returned `BLOCKED_BEFORE_CODE`; no
source was edited. It exposed two contract defects:

```text
architecture-kernel veto regex     false match on assert_eq via bare "sse"
remote-proof failure state         non-terminal because rejection recording branched from it
```

The repair narrows architecture tokens to whole feature names. A successfully
written proof receipt, including a quality FAIL, follows the normal
`PROOF_COMPLETE -> RESULT_RECORDED` transition. An interrupted proof is a
terminal rejection and cannot silently continue into recording, rerun or
deployment. The failed V1 manifest and receipt remain evidence; V2 must pass
against the updated paper bytes before code starts.

### V11 implementation preflight V2 result

V2 repaired both V1 paper defects without weakening a prohibition. The
implementation preflight then admitted exactly the packed-sidecar and
query-local-DLA implementation described above:

```text
preflight verdict                    READY_TO_IMPLEMENT
safe_to_implement                                  true
baseline receipts                                    10
identity contracts                                    7
invariants                                             9
mapped tests                                          21
blockers                                               0
```

Receipt:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/implementation-preflight-v11-packed-dafsa-dla-abc-v2.json
sha256 28ffa9dc52b2f778f3bfcf7243eaca4ad7ca96c064fe1bcea023f026f1710621
```

Tested: baseline-byte bindings, producer/consumer identity, packed-format and
DLA invariants, failure transitions and mapped proof obligations. Not tested:
source implementation, semantic parity, quality, latency, package/RSS budgets,
runtime integration or physical input. Runtime authority changed: `false`.

### V11 implementation and focused local verification

V11 is implemented in the existing proof-only V13 typed-peak module. It packs
the immutable DAFSA topology into checked `u64` state and edge records, stores
Unicode symbols in a validated sorted scalar table, builds a query-local DLA
from the V10 band recurrence, and intersects the two without merging the
path-dependent `rank_prefix`. Width overflow and invalid package fields are
rejections rather than truncations. V10 banded and full-row routes remain
proof-only parity oracles; no runtime fallback, global cache, worker queue,
SIMD kernel, unsafe decoder or case-specific rule was added.

```text
source
  src/nanda_wave/l2_field/v13_typed_peak.rs
source sha256
  d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b
projected packed sidecar                         2,460,144 B
V11 module tests                                     11 / 11 PASS
Phase 7A-7D focused tests                            15 / 15 PASS
cargo fmt --check                                          PASS
git diff --check                                            PASS
forbidden runtime-pattern scan                              PASS
Cargo target after checks             5,013,438,464 B / 12 GiB
```

Tested: checked packing/decoding, corruption rejection, DLA transition and
terminal behavior, exact-intersection local invariants and unchanged Phase
7A-7D focused contracts. Not tested: the full fixed A/B/C denominators,
heldout quality, measured sidecar/RSS/scratch budgets, remote latency,
Productive V90, L3, DecisionCore, verifier or physical input. The projected
sidecar size remains an estimate until the fixed remote proof writes and
reloads the artifact. Runtime authority changed: `false`. Promotion eligible:
`false`.

### V11 post-edit observed-source route result

The post-edit source route passed after the focused checks. All declared
execution, authority, observation and proof owners were found in the edited
source, with no hidden second producer or fallback route:

```text
observed route verdict                                  PASS
source evidence verified                             23 / 23
missing source evidence                                     0
```

Receipt:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v11-packed-dafsa-dla-observed-route.json
sha256 edaa3ff378936c89c41b962884a8dce60b25832fff0fb2b002ba8623252f867a
```

Tested: actual source ownership, call cardinality and separation of the live
V11 proof producer from V10 parity observers. Not tested: fixed-proof outputs,
quality, latency, resource budgets, runtime integration or physical input.
Runtime authority changed: `false`.

The only admitted next action is one remote V11 A/B/C acceptance proof against
the pinned V13, V90 and L1.1 artifacts. A FAIL returns to analysis of the first
shared mechanism; it does not authorize a rerun, V12, deployment, version bump,
push or gate weakening. A PASS authorizes documentation and a separate
V11-D/Productive V90 paper gate, not a runtime authority flip.

## Slice 8B V11 Measured A/B/C Result, 2026-08-24

The single admitted remote V11 A/B/C run completed against the pinned V13 and
V7 bytes. No retry was made. V13/V90/L1.1 were not rebuilt, and installed Lay
`1.0.43`, runtime authority, daemon and IBus were not changed.

```text
source SHA-256    d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b
V13 SHA-256       cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
V7 SHA-256        33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
receipt SHA-256   6631b4fb2d0ba7d47008ab577801ee2f4bf6e2b6facc5c99b79fff7f7c2680e9
sidecar SHA-256   5ebffb813ba0ca1e0080ec01756a2dafc51346297558d37cdd135abfde6acfaa
```

Receipt:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8_LEXICAL_READOUT_2026-08-23/slice8b-v11-packed-dafsa-dla-abc.json
```

### V11-A packed index identity: PASS

```text
decoded / terminal-rank parity             1,875,032 / 1,875,032
binding parity                              3,255,785 / 3,255,785
rank, surface or binding mismatches                                 0
states / edges / symbols                         81,128 / 226,341 / 34
sidecar bytes                                        2,460,144 B
loader-owned metadata                                      104 B
two-process aggregate PSS delta                         7,918 KiB
sidecar compile                                            5,315 ms
```

The packed sidecar exactly met its projected `2,460,144 B` size and is
`1,229,740 B` smaller than V10. All package, loader and PSS screens passed.

### V11-B DLA and oracle parity: PASS

```text
generated cases / operator families                         52 / 13
V10 banded / DLA mismatches in all parity dimensions              0
full-row / DLA mismatches in all parity dimensions                0
schedule/permutation mismatches                                   0
corruption false accepts                                           0
false completeness certificates                                   0
maximum DLA states / transitions                       1,045 / 10,441
maximum query scratch                                        97,394 B
```

### V11-C quality: PASS; latency: FAIL

```text
records / unique (damage_class, proof_identity)             382 / 382
target form / target lemma retained                          382 / 382
false certificates / unresolved                                  0 / 0
maximum expanded product states                                35,590
maximum DLA states / transitions                       1,904 / 27,451
maximum query scratch                                       174,033 B

single DLA build p50 / p99 / max              4.932 / 13.233 / 14.991 ms
single intersection p50 / p99 / max            6.442 / 9.305 / 9.876 ms
single search p50 / p99 / max                11.761 / 19.955 / 22.185 ms
single total p50 / p99 / max                 12.238 / 20.008 / 22.358 ms
20-client DLA build / intersection p99              23.808 / 20.412 ms
20-client search / total p99                         39.397 / 39.625 ms
20-client errors / unresolved                                  0 / 0
```

The global verdict is `FAIL_V11_A_B_C`. V11 preserved every quality and parity
denominator, but single-search p99 regressed from V10 `3.910 ms` to
`19.955 ms`, and 20-client total p99 regressed from `10.127 ms` to
`39.625 ms`. Scratch grew from `6,656 B` to `174,033 B`. These values remain
inside integrity budgets but fail the conjunctive latency contract.

### First shared mechanism and rejection

The measured DLA state and transition maxima are inside the paper projection,
so the failure is not an unexpected state explosion or a missing cap. The
projection estimated counts correctly but assigned the wrong cost to them.

For every retrieval lane V11 eagerly closes the abstract Levenshtein automaton
over every reachable `(band row, input class)` before touching the DAFSA
intersection. It hash-interns up to `1,904` rows and computes up to `27,451`
band transitions speculatively. The subsequent intersection still expands the
same `35,590` product states as V10. Therefore V11 does not reduce the exact
search frontier: it moves most recurrence work into a query-local `HashMap` and
transition table, then adds table lookup and a much larger scratch working set.

The decomposition is decisive without another run. DLA construction alone has
`13.233 ms` p99, more than three times the complete V10 search p99. The V11
intersection alone is also slower than complete V10 search. The receipt does
not isolate the remaining intersection regression between packed-byte decode,
transition-table indirection and cache locality, so no narrower micro-cause is
claimed.

V11 is rejected as the Slice 8B latency architecture. Packed topology remains
proven as an exact compact representation, but the query-local eager DLA is not
promotion eligible. No rerun, V12 implementation, runtime route, fallback,
version bump, push or deployment is authorized. Any later proposal must first
provide a paper cost model that avoids eager closure and preserves the fixed
V10/V11 candidate language; it then requires a new independently admitted
proof contract. Runtime authority changed: `false`.

Tested: packed identity and corruption screens, exact DLA parity, all fixed
quality denominators, single-client and 20-client proof-process latency,
scratch and two-process sidecar PSS. Not tested: a lazy/on-demand alternative,
optimized release-profile runtime, Productive V90 selection, L3 authority,
DecisionCore, verifier, physical input or product multi-client latency.

## V10 Cost Characterization: P0 Executable Provenance Decision

No V12 executor or diagnostic instrumentation is admitted before a cost model
is contracted. Historical V10 evidence may be compared only through an exact
executable provenance chain, not through a source reconstruction described as
equivalent.

### P0 result, 2026-08-24

The V10 module identity was recovered through `22` ordered Codex patch events
containing `119` hunks. Automatic reverse replay used `104` exact and `15`
rustfmt-tolerant nonspace matches, but produced a syntactically incomplete
`91,416`-byte file with SHA-256 `0fd534e7...`. One omitted call hunk was then
reconstructed manually, after which historical rustfmt produced `91,518` bytes
matching the independently known source identity recorded by the V10 receipt:

```text
V10 source bytes                                      91,518 B
V10 source SHA-256    f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c
active V11 SHA-256    d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b
```

The original release test executable also survived in the remote proof tree.
It was copied without execution and sealed with its build metadata:

```text
test binary bytes                                    20,457,336 B
test binary SHA-256  1e83ef76df68cd2f0238d1334eb4049f9063608292e72e5454a09f21a4afacc1
ELF Build ID         32e47da137adff6d49f9209ccd2804b6daa728ae
Cargo.toml SHA-256   90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b
Cargo.lock SHA-256   e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1
V13 sidecar SHA-256  a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
V7 denominator SHA   33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
V10 receipt SHA      08cccb3c9c63a24e9dc691958fde319de2a94b762fcb9f703ac342fcb72174e1
```

The fingerprint records release profile, empty `rustflags`, default features
and baseline `x86_64` target features `fxsr/sse/sse2`. The pinned toolchain is
`cargo 1.97.1 c980f4866`, `rustc 1.97.1 8bab26f4`, LLVM `22.1.6`; the build used
`CARGO_BUILD_JOBS=20` on an Intel i9-13900HK with `20` logical CPUs.

The original archive's staging directory was moved to its final path by a
same-filesystem atomic rename. Its receipt, `SHA256SUMS` and modes were then
mutated at the final path before the tree became read-only. Therefore atomic
path publication is proven, but atomic publication of the final immutable bytes
is not. The current completed bytes are read-only and pass their full manifest:

```text
/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f/

archive bytes                                              26,097,501
archive files                                                      19
PROVENANCE SHA-256  af26f90714460529c5778fb415a3c0aa7eb83ef1f51fdc97230417ac8f5c9faf
SHA256SUMS result                                               18/18
preflight v1                                      BLOCKED_BEFORE_CODE
preflight v2                                     READY_TO_IMPLEMENT
archive fault tests                                            3/3 PASS
```

The overclaims in that immutable archive are superseded without rewriting its
evidence bytes by:

```text
/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f-correction-v1/

correction files                                                    12
correction file bytes                                           49,028
CORRECTION SHA-256  74f66275a6e1f00b4dedea16ea2b62ab1ad2f4fffa6fada012bca19cc7a1ac90
SHA256SUMS result                                               11/11
writable objects                                                    0
post-rename mutations                                               0
```

### Provenance boundary

The dep-info file names `509` source dependencies totalling `8,171,027`
currently observed bytes. It is not a Cargo checksum manifest and does not
prove that the surviving bytes of all dependencies equal their build-time
bytes. The active remote copy of `v13_typed_peak.rs` is already V11 and was
therefore replaced in the archive by the exact recovered V10 bytes. The other
`508` surviving dependency files are inventoried with SHA-256 but are explicitly
labelled `SURVIVING_REMOTE_BYTES_NOT_BUILD_TIME_SNAPSHOT`.

Consequently the decision is split:

```text
historical executable identity                            SEALED
historical V10 module identity       SEALED BY HISTORICAL SHA
                                      AFTER PATCH RECOVERY
                                      WITH ONE MANUAL HUNK REPAIR
full historical source closure                            WATCH
byte-reproducible V10 rebuild                             NOT PROVEN
```

The exact surviving executable may later serve as the pristine latency subject
after the C evidence contract passes. A rebuilt or instrumented executable is a
new implementation candidate and must never be called the historical V10
baseline merely because its outputs are equivalent. It requires independent
semantic parity and provenance.

### P0 provenance correction v1

The original archive remains byte-identical. Its `PROVENANCE.json`, transaction
receipt and `SHA256SUMS` retain SHA-256 values `af26f907...`, `3c101113...` and
`194ff726...`. Correction v1 binds three narrower claims:

1. Exact source identity remains `PASS` because the final bytes match the
   independently known historical SHA-256. Recovery is not described as
   deterministic helper-only replay: it required one manual hunk repair after
   the automated output failed to parse.
2. The original `rename(stage, final)` was atomic, and the current original
   archive is integral and read-only. Final-byte atomic publication is
   `NOT_PROVEN` because the final path was subsequently mutated.
3. `cargo -V` and `rustc -Vv` were executed to identify the toolchain. No Cargo
   build, check or test, V10 proof/executable, or perf measurement was executed
   during P0 provenance archiving.

Correction v1 itself was fully populated, hashed, validated and made read-only
inside a staging directory in the final parent filesystem. Its single
same-filesystem rename was the last mutating operation; all post-rename actions
were read-only observations. Four correction fault injections passed. Runtime
authority changed: `false`.

### Frozen measurement contract

The `382` quality cases remain the quality denominator, not the statistical
latency denominator. Pristine latency must freeze:

```text
fixed query identities
x deterministic query order
x preregistered repetitions
x independent process runs
```

It reports pooled request p99, per-query p99 and worst-query p99. The 20-client
route additionally reports worst-client p99 and maximum per-query client spread
so pooled latency cannot hide worker starvation.

Structural counters, hardware characterization and pristine release latency are
three separate executions. Diagnostic counters may not contribute observations
to the pristine latency denominator. CPU time and wall time, cycles,
instructions, branches, branch misses, cache/LLC behavior and work counters are
reported separately for one and twenty clients.

Any predictive cost model must preregister its features and fixed
fit/validation/held-out query partitions. Admission uses:

```text
predicted p99 + preregistered model-error upper bound <= latency gate
```

No feature or coefficient may be added after observing held-out latency. Query
lengths `23..96` form a separate stress route requiring exact V10-oracle parity,
zero overflow/unresolved, fixed scratch/stack/transition bounds and maximum
single-query wall time. No short-corpus p99 claim extends to that range.

Current gate state:

```text
P0 executable provenance                               PASS
P0 full rebuild provenance                            WATCH
A structural work                               PAPER READY
B hardware characterization            PAPER ROUTE REVIEWED
C pristine latency                              PAPER READY
predictive model                            NOT CONTRACTED
V12                                            NOT ADMITTED
```

The dedicated B evidence contract is:

`docs/ime-target-authority-slice8b-v10-hardware-characterization-contract-2026-08-24.md`

It resolves historical phase attribution as `IMPOSSIBLE`: the stripped exact
ELF exposes only one mixed A/B/C test route. One exact-ELF G0 execution may
later establish a whole-process aggregate envelope, while executor hardware
events require a separately identified source-preserving diagnostic proxy.
Historical and proxy claims cannot be merged. The contract freezes B0-B7,
hybrid CPU topology and affinity, governor/load/thermal rejection, four
non-multiplexed PMU groups, exact one/20-client schedules, three-replica
stability rules and the claim boundary. It does not authorize perf, V10,
Cargo, proxy construction or V12; those actions require a separate
implementation preflight after structural critique.

The B critique is complete at paper scope. The final global owner skeleton is
`PASS`; all `14/14` independently owned local routes are `PASS`; conflicts and
evidence gaps are zero. Every receipt has `authority_ready=false`. Earlier
monolithic worksheet VETOs are retained and are not relabelled as a size-only
split acceptance. The paper status is therefore
`PAPER_ROUTE_STRUCTURE_REVIEWED`, while B measurement remains `NOT_ADMITTED`.
The next permitted artifact is a separate implementation preflight for B0-B2
scripts and later fixed measurement actions; no perf/V10/Cargo action follows
directly from the paper result.

### B0 sequencing correction v2, 2026-08-24

The later audit found that B0 could not truthfully claim an existing schedule:
`parse_v7_cases()` is test-only source and the sealed historical ELF has no
schedule-freezer entrypoint. The executable order is now frozen as:

```text
implementation preflight
  -> B0a immutable input/provenance closure
  -> one source-preserving diagnostic build
  -> unmeasured schedule-freezer
  -> B0b immutable schedule closure
  -> B1 environment
  -> B2 benign PMU capability
  -> B3 historical aggregate
  -> same-executable proxy parity
  -> B5/B6
```

B0a, build, freezer and B0b have separate owners and one-way outputs. The
freezer uses exact recovered `parse_v7_cases()` and Phase7D retrieval lanes,
emits the 382-entry target-free measured schedule, and cannot run under perf or
rebuild/publish anything. B0b cannot execute code and is the only schedule
publication owner.

The B design-window audit also records one bare `perf` executable invocation
caused by shell command substitution at `2026-08-24T17:33:52Z`. It emitted
usage only: no `perf stat/record`, PMU event or V10 subject occurred. Therefore
the valid claim is `no PMU measurement before B2`, not `perf was never
invoked`.

The first sequencing packet retained a `VETO` because it grouped build and
freezer under one owner. Correction V2 passes the global sequence skeleton and
all `19/19` local owner gates with zero conflicts/evidence gaps and
`authority_ready=false`. B measurement and V12 remain `NOT_ADMITTED`; the next
permitted artifact is still only the separate implementation preflight.

No transition design is selected. In particular, packed scalar, SWAR and
fully-unrolled band transitions remain hypotheses until the clean V10 cost
decomposition proves that transition arithmetic occupies enough of the budget
to meet both latency gates. This P0 work ran no V10 binary or proof, no Cargo
build/check/test and no perf measurement. It ran version queries only. No
runtime command, daemon restart, package installation or authority change
occurred.

### B0-B2 implementation preflight v2, 2026-08-24

The first implementation manifest was retained with
`BLOCKED_BEFORE_CODE`. All `19` byte/mode baseline checks passed, but its
reused-source scan did not bind `17` procedural forbidden effects to explicit
veto patterns. No prohibition was removed or weakened. V2 adds the missing
tripwires and preserves the V1 manifest and failed receipt as exact negative
evidence.

V2 passed `nanda-implementation-preflight`:

```text
verdict                                      READY_TO_IMPLEMENT
safe_to_implement                                          true
baseline checks                                            21
source checks                                                3
forbidden effects                                           23
forbidden source matches                                     0
preserved artifacts                                         13
state transitions                                            7
mapped tests                                                25
blockers                                                     0
```

The admitted implementation boundary is exactly:

```text
B0a immutable closure
  -> one guarded remote diagnostic build
  -> one unmeasured schedule-freezer
  -> B0b immutable schedule closure
  -> B1 observation-only environment gate
  -> B2 fixed benign PMU capability gate
  -> STOP
```

The build and freezer markers are consumed on either success or failure. A
defect requires paper analysis and a newly named preflight; V2 grants no retry,
second variant or adaptive build. The build may run only on
`e-MEGA-MINI-M1-13th` through the SHA-pinned `scripts/cargo-guard.sh`; no local
heavy build or proof is admitted. B2 may execute only the pinned benign
`/usr/bin/yes` workload and cannot load V13 or execute historical V10, the
proxy search, parity, B3, B5 or B6.

This preflight execution performed read-only local and remote stat/hash/archive
checks plus `cargo -V` and `rustc -Vv` toolchain queries. It performed no Cargo
build/check/test, no rustc compilation, no perf execution or PMU event, no V10
or proxy execution, no remote mutation and no runtime command. B0a, the build,
freezer, B0b, B1 and B2 are still unexecuted. Full historical source closure
remains `WATCH`; B measurement and V12 remain `NOT_ADMITTED`; runtime authority
changed: `false`.

Evidence:

```text
V1 manifest
docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V1_2026-08-24.json

V1 BLOCKED_BEFORE_CODE receipt
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_PREFLIGHT_V1_2026-08-24.json

V2 manifest
docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V2_2026-08-24.json

V2 READY_TO_IMPLEMENT receipt
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_PREFLIGHT_V2_2026-08-24.json
```

### Pre-B2 perf audit correction V3, 2026-08-24

During read-only implementation research after V2, remote wrapper inspection
invoked `perf --version` at `2026-08-24T19:29:33.706Z` and printed
`perf version 6.8.12`. It completed at `2026-08-24T19:29:34.617Z`.
This was a second pre-B2 invocation of the executable, after the earlier bare
usage-only invocation at `17:33:52Z`.

Neither event ran `perf stat/record`, opened or measured a PMU event, or
executed V10, the proxy or the benign B2 workload. Controller code had not been
created and B0a had not started. Nevertheless the frozen V2 audit count was no
longer true, so V2 is retained as evidence but is superseded for execution.

```text
pre-B2 perf executable invocations                  2
pre-B2 PMU measurements                             0
controller implementation                 NOT STARTED
B0a / build / freezer / B0b                NOT STARTED
B1 / benign B2                             NOT STARTED
required next artifact          implementation preflight V3
runtime authority changed                         false
```

No further `perf` executable invocation is permitted before accepted B1 admits
the benign B2 capability route. B3, parity, B5/B6 and V12 remain
`NOT_ADMITTED`.

The structural correction was then checked with the Markdown gate. Two earlier
zero-byte outputs from the wrong JSON-on-Markdown command are retained as
failed-attempt evidence. The original correction worksheet produced a real
`VETO`: audit, admission and execution sequencing had been placed in one owner
group. A new repair worksheet split those ownership routes while preserving the
same `2` executable invocations, `0` PMU measurements, unchanged B0-B2 sequence
and unchanged STOP boundary.

```text
global route skeleton V4       PASS
local correction repair V1     PASS
authority_ready                false
owner conflicts                    0
semantic conflicts                 0
evidence gaps                      0
```

The owner-conflict `VETO` remains negative evidence. These structural passes do
not restore V2 execution authority; the next permitted action remains creation
and execution of implementation preflight V3 only.

### V10 hardware B0-B2 implementation V3, unrun

The independently named V3 preflight passed and admitted only the B0-B2
controller implementation. Static implementation verification is now closed:

```text
implementation state              B0_B2_TOOLS_IMPLEMENTED_UNRUN
controller self-check             PASS, 10 focused checks
Rust parse without Cargo          PASS
V10 production prefix             39,047 bytes byte-identical
remote actions                    0
Cargo / freezer / perf / PMU      0 / 0 / 0 / 0
runtime authority changed         false
```

The controller records all 509 source dependency-path rows while mapping one
byte-identical normalized alias pair to one of 508 unique frozen destinations.
Conflicting aliases fail closed. Full build-time source closure remains
`WATCH`.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V3_2026-08-24/IMPLEMENTATION_UNRUN_RECEIPT.json`

The next permitted action is B0a. B3, proxy parity, B5/B6 and V12 remain
`NOT_ADMITTED`.

#### B0a attempt V1 blocked before mutation

The first B0a identity probe exposed an SSH argv-quoting defect before remote
Python or any write started. Remote audit found no provenance root, state root,
stage, marker, Cargo/rustc/`perf` process or diagnostic executable. V3 is
retained; the controller is not changed or rerun until a separately named
preflight admits the narrow SSH boundary repair.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V3_2026-08-24/B0A_ATTEMPT_V1_BLOCKED_BEFORE_REMOTE_WRITE.json`

#### SSH argv repair V5, unrun

V4 is retained as `BLOCKED_BEFORE_CODE`; V5 is the active
`READY_TO_IMPLEMENT` correction. The controller now uses a byte-preserving
`shlex.join` remote boundary and requires the V5 manifest/receipt. Twelve local
focused checks pass, including multiline argv parity and raw-join, empty-argv
and NUL faults. No remote action, Cargo, `perf`, PMU, V10 or proxy run occurred
after the repair. The next permitted transition is B0a retry; the build and
freezer markers remain absent.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V5_2026-08-24/SSH_ARGV_REPAIR_UNRUN_RECEIPT.json`

#### B0a attempt V2 blocked before mutation

The quoted SSH boundary passed, but the read-only probe assumed that the future
remote `provenance` parent already existed. It does not. No stage, state root or
marker was created. A full read-only route audit then verified all 509 source
rows with zero mismatch, the V13/toolchain identities, same-filesystem nearest
existing parent and 291,327,811,584 free bytes without invoking `perf`. V5 is
retained; a new preflight must admit nearest-existing-parent device resolution
before another B0a retry.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V5_2026-08-24/B0A_ATTEMPT_V2_BLOCKED_BEFORE_REMOTE_WRITE.json`

#### Parent/device repair V6: read-only probe blocked

V6 returned `READY_TO_IMPLEMENT` and admitted only nearest-existing-parent
resolution plus pre/post-creation filesystem-device validation. The repaired
controller passes 14 focused local checks. A separate read-only call then
confirmed the complete parent route before any B0a mutation:

```text
requested provenance root exists                         false
nearest existing parent          /home/e/.local/share/lay
nearest parent device                                  66306
V13 bytes / SHA                                   exact PASS
remote writes / stages / markers                    0 / 0 / 0
```

The same probe exposed a separate latent identity bug. The pinned
`5ac0bb...` value is the SHA-256 of the exact `/etc/machine-id` file bytes,
including its final newline, while every controller owner used
`read_bytes().strip()` and obtained `ab7e08...`. The host did not change; the
normalization contract and implementation disagreed.

```text
parent/device repair                              PASS locally
read-only remote admission                       BLOCKED
failure class          MACHINE_ID_HASH_NORMALIZATION_MISMATCH
B0a started                                        false
build / freezer / perf / PMU                  0 / 0 / 0 / 0
runtime authority changed                          false
```

V6 is retained and is not reused for a retry. The next required artifact is a
new implementation preflight admitting exact-file-byte machine-id hashing
across every B0-B2 owner. B3, parity, B5/B6 and V12 remain `NOT_ADMITTED`.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V6_2026-08-24/PARENT_DEVICE_REPAIR_READ_ONLY_PROBE_BLOCKED.json`

#### Exact-byte machine-id repair V7, unrun

V7 returned `READY_TO_IMPLEMENT` and replaced normalized machine-id hashing
with one exact-file-byte contract across all B0-B2 host-identity owners. The
known stripped digest remains only in a negative-control test.

```text
controller self-check                              15/15 PASS
runtime normalized-hash owners                              0
exact-file-byte owners                                      4
remote machine-id SHA                            5ac0bb... PASS
parent / device                         /home/e/.local/share/lay / 66306
remote writes / stages / markers                    0 / 0 / 0
B0a started                                        false
```

The repaired read-only probe now passes hostname, exact machine-id, V13 and
parent/device parity together. Post-probe audit found no route process or
output root. The next permitted transition is B0a. Build, freezer, B0b, B1 and
B2 remain unrun; B3, parity, B5/B6 and V12 remain `NOT_ADMITTED`.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/EXACT_BYTE_MACHINE_ID_REPAIR_UNRUN_RECEIPT.json`

#### B0a input closure V7

B0a completed and atomically published the immutable input closure on the
admitted remote host.

```text
state                                  B0A_PASS_BUILD_UNUSED
remote receipt SHA-256                 48176ec6faae86f43ddda8404542367be5b6c9d6813762dedb829b4946593eb3
root mode / writable objects           0555 / 0
source rows / unique destinations      509 / 508
machine-id / parent device             exact PASS / 66306
build / schedule                       NOT_BUILT / NOT_CREATED
markers                                build.available, freezer.available
```

No Cargo, freezer, `perf`, PMU, V10 or proxy execution occurred. The next
permitted transition is exactly one guarded remote diagnostic build. Failure
consumes that build right and requires a new paper revision; no retry is
permitted. B3, parity, B5/B6 and V12 remain `NOT_ADMITTED`.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/B0A_PASS_BUILD_UNUSED_RECEIPT.json`

#### Diagnostic build V7

The single guarded remote build completed and consumed its marker before
Cargo. The resulting V10-derived diagnostic proxy is sealed but unexecuted.

```text
state                      DIAGNOSTIC_EXECUTABLE_SEALED_BUILD_CONSUMED
root mode / writable       0555 / 0
ELF bytes                  20,542,920
ELF SHA-256                f7bcee37d5dffd583577d66c982c2a28072889b8aac2ccbed22612aa6d4feb09
Build ID                   9829fb05f34bd353877fb6d71f1f8523e084af55
production prefix          exact PASS
executed / perf invoked    false / false
freezer marker             available
```

No rebuild is permitted. The next transition is exactly one unmeasured
schedule-freezer run. This build is not historical V10 because full build-time
source closure remains `WATCH`. B3, parity, B5/B6 and V12 remain
`NOT_ADMITTED`.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/DIAGNOSTIC_BUILD_PASS_FREEZER_UNUSED_RECEIPT.json`

#### Schedule freezer V7

The single unmeasured freezer run completed and consumed its marker before
exec. The sealed executable identity remained unchanged.

```text
state                    FREEZER_OUTPUT_STAGED_EXECUTABLE_UNCHANGED
wrapper receipt SHA      c2bd03c307e576ea8b2d8bb113856e84dfb53251decdc21d4b7ad1f75e8cb801
schedule SHA             2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
entries / unique         382 / 382
ordinals                 0..381
target keys              0
perf / PMU               false / false
B0b published            false
```

No search enumerator or latency route ran. Build and freezer cannot be
repeated. The next transition is B0b independent validation and immutable
schedule publication; B3, parity, B5/B6 and V12 remain `NOT_ADMITTED`.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/FREEZER_PASS_B0B_UNPUBLISHED_RECEIPT.json`

#### B0b schedule closure V7

B0b independently validated and published the immutable schedule closure
without executing code.

```text
state                    B0B_PASS_SCHEDULE_SEALED
closure receipt SHA      e162c301cb3fa0c557a66689275c558ea9bcba57480d0eccdbb0cfd4003a9b0f
root mode / writable     0555 / 0
entries / unique         382 / 382
target fields            false
B0a / schedule / ELF     exact binding PASS
code executed by B0b     false
```

The next transition is B1 environment observation and admission. No B3,
parity, B5/B6 or V12 action is admitted.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/B0B_PASS_B1_NOT_STARTED_RECEIPT.json`

#### B1 environment V7: BLOCKED

B1 observed for the full 120-second bound and published
`BLOCKED_ENVIRONMENT`. This is not residual build load. A persistent production
service occupied more than one CPU for the complete window:

```text
nando operator PID / cgroup   150005 / nando-operator-certification-authority.service
observed CPU                  107.837%..107.841%
allowed CPUs                  0-19
CPU PSI some avg10            10.43..11.08    gate <=2.0
IO PSI full avg10             1.71..2.08      gate <=0.10
additional busy observer      btop, occasional sshd
```

B1 changed no host policy and invoked no `perf` or PMU event. Its sealed
receipt is mode `0555`, writable objects `0`, SHA-256
`12c591b447f0025548e96272a4e8d5d4f23debc8458ae99fce61b2e751673925`.
B1 failure is terminal under V7; B2 was not executed and is not admitted.
Passing now requires an externally authorized quiet-host maintenance window or
a separately admitted host, not a weaker gate.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25/B1_BLOCKED_ENVIRONMENT_B2_NOT_ADMITTED_RECEIPT.json`

#### Dirty V10 proxy speed observation V1

At the user's explicit product-diagnostic boundary, one B5 and one B6 run were
executed on the loaded mini-PC without stopping Nando or `btop`:

```text
B5, 1 worker, 382 requests
  executor-window wall              1,032.815 ms
  average window / request              2.704 ms
  throughput                          369.863 requests/s

B6, 20 workers, 382 requests
  executor-window wall                179.461 ms
  wall / request for throughput         0.470 ms
  throughput                        2,128.596 requests/s

concurrent load
  CPU PSI some avg10                3.89..6.02
  Nando authority CPU               104.57% B5 / 462.50% B6
  btop CPU                             3.87% B5 / 22.29% B6
  maximum temperature                69 C
  throttle counters                  unchanged
```

These are dirty aggregate executor windows. `2.704 ms` is B5's corpus average,
not p99. `0.470 ms` is B6 batch wall divided by 382, not request latency.
Per-query p99 was not observed. The controller-to-subject start handshake can
add up to 10 ms to the whole window.

The sealed proxy's semantic parity was not run, so quality remains `UNKNOWN`.
V7 B1 remains `BLOCKED_ENVIRONMENT`; B2, B3, formal B promotion and V12 remain
not admitted. Installed Lay stayed `1.0.43`, active V11 stayed
`d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b`,
and runtime authority changed: `false`.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_DIRTY_SPEED_V1_2026-08-25/REMOTE_RECEIPT.json`

SHA-256:
`9255310624b55d370db55800f88eb6a04172d555235aecc6587d0ac15826af48`.

#### Clean speed comparison V2: prepared, unrun

The same sealed B5/B6 aggregate route is prepared for one clean-host run.
Readiness is read-only and repeatable; measurement markers remain absent until
three consecutive samples pass. The future route uses B5 then B6, 382 requests,
CPU `0` for B5 and CPUs `0..19` for B6 without build, `perf`, PMU, host tuning
or process control.

Current readiness is `BLOCKED_ENVIRONMENT`: Nando authority remains near one
full CPU or more, `btop` remains active, CPU PSI was about `5.4..5.9` and IO PSI
about `2.0..5.4`. A test invocation was fault-checked and stopped before any
subject, marker or remote output. The one measurement attempt remains unused.

```bash
scripts/lay-v10-hardware-clean.py ready
scripts/lay-v10-hardware-clean.py run
```

This prepared comparison does not claim per-query p99, formal B PASS, quality
parity or V12 admission.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_PREPARED_V2_2026-08-25.json`

#### Loaded V10 proxy semantic parity V1

One semantic-parity entrypoint was executed under the existing Nando and
`btop` load without repeating the dirty B5/B6 speed routes:

```text
fixed schedule / records       382 / 382
terminal mismatches                  0
peak mismatches                      0
completeness mismatches              0
work mismatches                      0
full-row mismatches                  0
target form / lemma retained   382 / 382
false certificates                   0
maximum product states          35,590
maximum scratch bytes             6,656
```

The sealed V10-derived diagnostic proxy therefore has scoped semantic parity
for this fixed denominator. The run did not establish historical source
closure and did not measure request latency. Its `4,007.824 ms` whole-process
wall value includes harness startup, input loading, all parity oracles and
evidence publication and must not be interpreted as p99.

During the run CPU PSI some avg10 was `4.90 -> 4.32`, Nando authority used
`100.30%` CPU, `btop` used `4.49%`, maximum temperature was `62 C`, and thermal
throttle counters were unchanged. No `perf`, PMU, Cargo, host tuning, process
stop, runtime deployment or B5/B6 repetition occurred.

The combined loaded evidence is now:

```text
dirty aggregate speed          OBSERVED
sealed proxy semantic parity   PASS
clean speed V2                 PREPARED, UNRUN
B1                              BLOCKED_ENVIRONMENT
formal B PASS                  false
V12                             NOT ADMITTED
runtime authority changed      false
```

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PARITY_V1_2026-08-25/REMOTE_RECEIPT.json`

SHA-256:
`3d7d15b15c57c4aa1d4e7a358a2daa022ba2a4bf005f9c3dbde608befee797ad`.
Installed Lay remained `1.0.43`, and active V11 remained
`d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b`.

#### Clean speed comparison V2: SUPERSEDED_UNRUN_BY_C1

The prepared clean aggregate B5/B6 route was physically closed before any
Clean V2 subject or measurement. Its exact remote state path now contains a
sealed supersession tombstone; its old final result path remains absent.

```text
subject executed               false
measurement produced           false
clean result published         false
old route physically runnable  false
state mode / file modes        0555 / 0444
files / writable objects       2 / 0
SHA256SUMS                      PASS
rollback                       RETAIN_TOMBSTONE
```

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_SUPERSESSION_V1_2026-08-25.json`

Receipt SHA-256:
`c6a82710d7bf4cafda33bcec21efe784605219c62d1a5fde252ff5c004351ffa`.
Full B is held until a future `C1_FAIL` paper decision. C1 build, parity and
latency remain unrun pending their separate implementation preflight; V12 is
still `NOT_ADMITTED`, installed Lay remains `1.0.43`, and runtime authority
changed: `false`.

#### C1 implementation, build and parity: PASS; latency readiness: BLOCKED

The preceding `C1 build, parity and latency remain unrun` statement is
superseded by this execution record. C1 implementation preflight V2 corrected
the audited remote V7 mode from `0400` to `0444`. It passed with 19 baseline
checks, 26 forbidden effects, 33 mapped tests, zero blockers and zero forbidden
matches. The controller local self-check passed without remote execution.

One and only one guarded offline release build was then consumed and sealed:

```text
production prefix SHA-256     ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
assembled source SHA-256      da659fcbe46fabb5c99bd7a2f491b57c540064ad6783705bcda7fa7997ce35fd
C1 ELF SHA-256                ead184029a2923cfd24c5d02e91e91f9d69fb01c7daa0fb21ba69f267234c93c
C1 ELF Build ID               665458be5064a689951c6f074276ab0bb4d2beb4
C1 ELF bytes                  20,531,304
build retry permitted         false
```

The separate unmeasured C1 parity process passed the exact 382-case semantic
prerequisite:

```text
records / schedule records    382 / 382
terminal / peak mismatch      0 / 0
completeness / work mismatch  0 / 0
target form / lemma retained  382 / 382
false certificates            0
maximum product states        35,590
maximum scratch bytes         6,656
```

The first readiness call exposed a controller-only consumer-path defect: the
sealed producer published `parity-v1/subject/SUBJECT_RECEIPT.json`, while
`remote_status_value()` looked one directory higher. No environment admission,
latency process or marker consumption occurred. Remote audit V3 and
implementation preflight V3 retained the sealed build/parity and admitted only
that reader-path repair plus the V3 evidence pins. After repair, read-only
status consumed the exact sealed parity PASS.

The clean-host readiness gate then returned `READINESS_BLOCKED_ENVIRONMENT`:

```text
Nando authority PID / CPU     150005 / 103.84%..107.88%
btop peak CPU                 7.99%
CPU PSI some avg10            5.56..5.68       gate <=2.0
IO PSI full avg10             0.00..1.63       gate <=0.10
maximum temperature           64..65 C         PASS
stable host projection        unchanged
thermal throttle counters     unchanged
overheating                    false
```

This is not a C1 latency PASS or FAIL. No S/T process executed and all ten
markers remain available in the frozen order
`S1 T1 T2 S2 S3 T3 T4 S4 S5 T5`. The next permitted action is another
read-only readiness check during an externally authorized quiet-host window;
the fixed matrix may run only on `READY_FOR_S1`. Dirty C1 acceptance, threshold
relaxation, foreign process control, full B and V12 remain forbidden.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_READINESS_BLOCKED_ENVIRONMENT_V1_2026-08-25.json`

Receipt SHA-256:
`402d342ad3778f40e5603498b0d68e1c1268f5a6b4040eafe6d5cd74515bc9d1`.
Installed Lay remained `1.0.43`, active V11 remained
`d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b`,
the IBus/Lay PIDs remained `2076194 / 3410795 / 3410820`, and runtime authority
changed: `false`.

#### C1 dirty-load direct observation: thresholds would fail

The user explicitly selected the mini-PC's current loaded operating condition
instead of a quiet-host acceptance environment. A separate dirty route reused
the sealed C1 ELF and completed parity without a build or parity rerun. It did
not call the clean C1 matrix owner and used disjoint state/result paths.

The first route packet was retained as `VETO` after placing multiple owners in
one linked group. Route V2 split provenance, state, execution, observation,
environment, decision and publication owners and passed with
`authority_ready=false`. Implementation preflights V1 and V2 were retained as
`BLOCKED_BEFORE_CODE`; V3 passed. A read-only audit then found that the remote
sealed clean controller is SHA `4c09ac00...`, while the current local controller
is SHA `e6e3db85...`. The only differences are superseded preflight pins and the
already documented status-reader path repair. AST source hashes for all eleven
reused utility functions match. Correction preflight V4 passed with 18 baseline
checks, 17 forbidden effects, 16 mapped tests and zero blockers.

The fixed matrix then completed once in the preregistered order:

```text
S1 -> T1 -> T2 -> S2 -> S3 -> T3 -> T4 -> S4 -> S5 -> T5

S samples                          191,000
T samples                          477,500
errors / unresolved                 0 / 0
started                    02:06:25 UTC
completed                  02:18:46 UTC
```

Authoritative production-owned latency fields produced:

```text
                                      observed     threshold    comparison
S pooled search p99                   3.940 ms     <=3.000 ms   FAIL
S pooled total p99                    4.098 ms     <=5.000 ms   PASS
T pooled total p99                   17.349 ms     <=5.000 ms   FAIL
max(run x worker total p99)          57.917 ms     <=5.000 ms   FAIL

S1..S5 total p99 range            4.078..4.234 ms  all PASS
T1..T5 total p99 range           15.505..19.922 ms all FAIL
worst fairness route / worker       T1 / worker 19
fairness spread                     50.828 ms
worst query ordinal / total p99      381 / 54.586 ms
```

Therefore:

```text
verdict                         DIRTY_LOAD_OBSERVATION
thresholds_would_pass           false
clean C1 PASS/FAIL              NOT_MEASURED / BLOCKED_ENVIRONMENT
formal B                        false
V12 admitted                    false
runtime authority changed       false
```

The loaded denominator was real rather than nominal. Across route windows the
Nando certification authority consumed about `88..100%` CPU, btop consumed
`4.3..7.2%`, transition serving remained active, and a separate Nando proof
process consumed up to about `31%` during later routes. CPU PSI `some avg10`
reached `11.73`. Maximum observed temperature was `78 C`; no thermal-throttle
counter changed, so overheating did not cause the verdict.

Remote final evidence contains 63 files and 15,157,521 bytes. Its manifest is
`52/52 PASS`; the separate dirty state manifest is `11/11 PASS`; writable
objects are zero. All ten clean latency markers remain `available`, the clean
C1 final path remains absent, and `clean_state_unchanged=true`.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_DIRECT_LATENCY_V1_2026-08-25/REMOTE_RESULT.json`

Result SHA-256:
`9c74bf3f74f794ddd7e5c46df56f88e1443f6e320afb425dd015d20adecb207e`.

The immutable local index used an over-narrow anchored `pgrep -f` pattern and
therefore recorded empty process arrays before and after. Their equality is not
PID-continuity proof. A superseding coverage receipt records the defect and an
independent post-run broad scan, which found the expected processes at PIDs
`2076194 / 3410795 / 3410820`. Lay remained `1.0.43` and active V11 remained
`d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b`.

Correction evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_DIRECT_LATENCY_RUNTIME_COVERAGE_CORRECTION_V1_2026-08-25.json`

This observation answers whether the current exact V10 route meets the frozen
latency targets under the user's real loaded host: it does not. It does not
attribute the miss to executor arithmetic, cache pressure, Nando scheduling or
another cause, and it does not replace the still-unrun clean C1 decision.

#### C1 dirty-load replication V2: threshold miss reproduced

A second one-shot route repeated the exact frozen `S1 T1 T2 S2 S3 T3 T4 S4
S5 T5` matrix under the mini-PC's current background load. Nando and `btop`
were neither stopped nor tuned. The sealed C1 executable was reused without a
build or parity rerun; the V2 route used disjoint state and result paths and did
not consume any clean C1 marker.

```text
                              V1          V2       threshold
S pooled search p99        3.940 ms     3.938 ms   <=3 ms FAIL / FAIL
S pooled total p99         4.098 ms     4.098 ms   <=5 ms PASS / PASS
T pooled total p99        17.349 ms    16.336 ms   <=5 ms FAIL / FAIL
max run x worker p99      57.917 ms    58.618 ms   <=5 ms FAIL / FAIL

S samples                 191,000       191,000
T samples                 477,500       477,500
errors / unresolved         0 / 0         0 / 0
```

V2 per-run total p99 was:

```text
S1 4.082   S2 4.080   S3 4.086   S4 4.237   S5 4.083 ms
T1 15.614  T2 14.059  T3 17.810  T4 13.662  T5 17.691 ms
```

The second denominator was materially loaded. Across route windows the Nando
operator certification authority consumed `87.29..99.99%` CPU, `btop`
consumed `4.21..7.22%`, and the K1 transition server reached `81.49%`. CPU PSI
`some avg10` reached `10.14`. Maximum observed temperature was `80 C`; no
thermal-throttle counter changed. The repeated miss therefore has no observed
semantic-error or thermal-throttle explanation, but C1 does not attribute the
remaining cost to any specific executor or hardware mechanism.

The V2 local index inherited V1's over-narrow anchored `pgrep -f` pattern and
again recorded empty process arrays. That field is not PID-continuity evidence.
An independent post-run snapshot observed the expected processes at PIDs
`2076194 / 3410795 / 3410820`, Lay `1.0.43`, and active V11 SHA
`d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b`;
the snapshot proves current presence only, not continuity throughout V2.

The replicated decision is:

```text
loaded V10 thresholds                 FAIL, reproduced 2/2
clean C1                              NOT_MEASURED / BLOCKED_ENVIRONMENT
formal B                              NOT RUN
V12                                  NOT ADMITTED
runtime authority changed             false
```

V2 evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_DIRECT_LATENCY_V2_2026-08-25/REMOTE_RESULT.json`

V2 result SHA-256:
`57e1de132b75f12749bedf506302791900bbf97b35796e2b90bf40806bc98a35`.
The local index SHA-256 is
`38f97cddb53af117238d5cd8b5e5ca7e9b76f67dc81fa6711a3adf8aef0494fb`;
its manifest is `52/52 PASS` remotely, the state manifest is `11/11 PASS`, and
writable objects are zero.

Comparison evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_DIRECT_LATENCY_V1_V2_COMPARISON_2026-08-25.json`

Comparison receipt SHA-256:
`01b9c10c72e9c0a19572c543197f20ce073dd71ad8406cd336e757bf6cd8868c`.
V1 remains separately retained with result SHA-256
`9c74bf3f74f794ddd7e5c46df56f88e1443f6e320afb425dd015d20adecb207e`.

#### Loaded PMU diagnosis V1: blocked by hybrid-row parser

The loaded PMU route was admitted after dirty latency replication closed at
two runs. A remote-bootstrap defect first failed before remote execution and
was repaired under implementation preflight V4. The repaired controller passed
syntax and 11 self-checks, then created the disjoint one-shot V1 state.

Same-ELF semantic parity passed all 382 records with zero mismatches and false
certificates. The following benign CPU-0 capability probe opened PMU events but
terminated before any B5/B6 executor window:

```text
cpu_core cycles          799,313,601       running 100%
cpu_core instructions  4,269,402,610       running 100%
cpu_atom cycles          <not counted>      runtime 0 / running 0%
cpu_atom instructions    <not counted>      runtime 0 / running 0%
```

The host is hybrid: `cpu_core` covers CPUs `0..11`, `cpu_atom` covers CPUs
`12..19`, and the capability workload was fixed to CPU 0. The V1 parser matched
both perf-expanded PMU rows to each logical event and incorrectly required
every row to be numeric. Therefore this is an observer defect, not V10 PMU
evidence and not an effect attributed to Nando or temperature.

V1 final and state manifests pass, writable objects are zero, parity and
capability markers are consumed, and all eight executor-window markers remain
available. V1 retry is forbidden. A separate V2 correction may use a disjoint
task identity and architecture-aware hybrid validation only after its own route
and implementation preflight pass.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_CAPABILITY_FAILURE_V1_2026-08-25.json`

Remote result SHA-256:
`80f82b9c6d7a2251ec52a076ca1496789a22d641bfc38f408d0cc59a4b592cf3`.
Formal B remains unrun, V12 remains not admitted, and runtime authority changed:
`false`.

#### Loaded PMU continuation V4: complete common-event diagnosis

V4 implementation preflight passed with 23 baseline checks and no blockers or
forbidden matches. The disjoint continuation ran only B5/B6 G2C and G3; it did
not repeat G0/G1. Parity and benign capability passed, all four one-shot
windows completed, stable host projection and throttle counters were unchanged,
and immutable remote/local publication verified.

Combined V3/V4 measurements are:

```text
metric                         B5                 B6          delta
expanded states/query         21,098.9           21,098.9     identical
instructions/query            42.379 M           42.389 M     +0.024%
cycles/query                  10.308 M           14.178 M    +37.549%
IPC                            4.111              2.990       -27.281%
branches/query                 5.594 M            5.596 M     +0.026%
branch miss rate               0.681%             0.698%
L1 data loads/query            6.677 M            6.679 M     +0.036%
LLC misses/query              71.91              18.92       -73.69%
dTLB misses/query              8.92             244.37        27.40x
```

The additional B6 cost is not additional executor work: structural states,
instructions, branches and L1 loads are effectively unchanged. Common LLC
traffic does not rise. dTLB misses do rise by about `235/query`, but explaining
all additional `3.87M cycles/query` from that count alone would require
`16,438 cycles` per extra miss. The dominant measured symptom is therefore
lower instruction throughput on the fixed mixed P/E 20-worker route, not more
instructions or branch work. This does not isolate heterogeneous execution,
scheduling, shared-memory contention or another microarchitectural cause.

The executor still costs about `42.38M instructions/query`, or about `2,008
instructions/expanded state`. That ratio cannot yet be interpreted per edge:
the sealed proxy records expanded states but not examined edges. The next useful
denominator is structural `edges examined` and transition calls on the exact
frozen trace, not another latency or PMU replication and not V12 code.

V4 evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_V4_2026-08-25/REMOTE_RESULT.json`

Combined decision:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_COMBINED_V3_V4_2026-08-25.json`

V4 result SHA-256:
`28f65ecd7cd596f7515edb635ca5fc749020d398148fa1a656374a7e5d40b016`.
Formal B remains unrun, V12 remains not admitted, and runtime authority changed:
`false`.

#### Loaded PMU diagnosis V3: G0/G1 observed, G2 event not common

V3's runtime-weighted hybrid parser passed exact V2 raw replay and completed
four authoritative windows before stopping at B5-G2:

```text
metric                         B5                 B6          delta
instructions/request          42.379 M           42.389 M     +0.024%
cycles/request                10.308 M           14.178 M    +37.549%
IPC                            4.111              2.990       -27.281%
task-clock/request             2.719 ms           4.691 ms   +72.546%

branches/request               5.594 M            5.596 M     +0.026%
branch misses/request         38,109             39,044       +2.452%
branch miss rate               0.681%             0.698%
generic cache miss rate        1.683%             0.935%
```

This rules out more executor instructions, more branches or a large branch-miss
increase as the primary aggregate B5-to-B6 cost. B6 performs effectively the
same control-flow work but spends substantially more cycles at lower IPC.
Generic cache misses do not rise in G1, although that generic counter is not a
complete data-locality proof.

B5-G2 then counted the required core L1 loads and misses at 100 percent, but
perf emitted `<not supported>` for non-required atom L1 misses. A read-only
`sudo perf list` confirmed that L1-dcache-load-misses exists only for
`cpu_core`; L1 loads, LLC loads/misses and dTLB loads/misses exist for both PMU
types. Therefore no honest B5/B6 L1-miss comparison is available from this
event.

V3 is immutable. The next scoped route may run only common G2C and G3 events
under a disjoint identity, without repeating G0/G1 or substituting the missing
L1-miss counter.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_COMMON_EVENT_DECISION_V3_2026-08-25.json`

V3 remote result SHA-256:
`fb625d92f27fc783b6d88358fb119b087fb7bca35d7bbb99ad665864f6afcc67`.
Formal B remains unrun, V12 remains not admitted, and runtime authority changed:
`false`.

#### Loaded PMU diagnosis V2: G0 exposes hybrid runtime partition

V2 corrected inactive PMU-row handling under route V5 and implementation
preflight V6. Same-ELF parity passed, benign CPU-0 capability passed, and B5-G0
completed. B6-G0 then captured both PMU types but V2 terminated because neither
row individually had 100 percent running time:

```text
                         B5-G0             B6-G0 runtime-weighted
instructions/request    42.380 M           42.388 M
cycles/request          10.132 M           14.044 M
IPC                      4.183              3.018
task-clock/request       2.673 ms           4.648 ms

B6 atom runtime          680,456,525 ns     38% reported
B6 core runtime        1,095,195,003 ns     61% reported
runtime sum            1,775,651,528 ns     exact task-clock runtime
```

Perf scales each hybrid-PMU value to the full enabled interval. Weighting each
reported value by its exact event-runtime share shows nearly identical executor
instruction count but `38.6%` more cycles and `27.8%` lower IPC in B6. This is
direct evidence against increased instruction count as the primary aggregate
B5-to-B6 difference. It does not explain the latency tail or assign causality to
Nando, cache, scheduling or heterogeneous cores alone.

G1-G3 did not run. V2 is immutable and has no retry right. A V3 correction must
pre-register exact runtime-weighted hybrid reconstruction and use disjoint
state before completing the remaining PMU characterization.

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_HYBRID_PARTITION_FAILURE_V2_2026-08-25.json`

V2 remote result SHA-256:
`0fc4b2bd7a54a2e98db9e9b0705b7eefc94b33ffb3a513f5fd12a3efceccae5d`.
Formal B remains unrun, V12 remains not admitted, and runtime authority changed:
`false`.

#### V10 structural work A2: generic executor cost confirmed

The source-preserving A2 observer completed one exact 382-record traversal
under the normal loaded host without controlling Nando, btop, K1 or any other
foreign process. The first A1 build failure remains immutable; A2 changed only
the oversized counter JSON construction and preserved the V10 production
prefix exactly:

```text
production prefix bytes         39,047
production prefix SHA-256       ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
A2 ELF SHA-256                  e6c0aa156c1c22fd8eeb8fcf56826e765d6462c0b2464f7d43b244da2f46ce24
A2 Build ID                     0f6f7732d7d13c76b000176a491dbcf7cf7ea717
```

Semantic parity remained exact:

```text
records                         382/382
terminal / peak mismatch        0 / 0
completeness mismatch           0
state / scratch mismatch        0 / 0
false certificates              0
target form / lemma             382/382
maximum states                  35,590
maximum scratch                 6,656 B
```

The deterministic work denominator is:

```text
                              total             per request
expanded states               8,059,788          21,098.9
edges / transitions          25,145,756          65,826.6
surviving edges               8,059,024          21,096.9
pruned edges                 17,086,732          44,729.7
band cells evaluated        173,652,383         454,587.4
minimum cells scanned       173,652,383         454,587.4
query comparisons           171,852,017         449,874.4
certificate calls               17,600              46.1
```

The radius-3 transition evaluates `6.906` of seven cells on average. Every
transition then invokes a separate minimum scan, and `67.95%` of the completed
transitions are immediately pruned. Combined with loaded B5 PMU this is
`643.79 instructions/examined edge` and `93.22 instructions/evaluated band
cell`, from `42.379M instructions/request`.

This confirms excessive physical work in the generic V10 executor. Foreign
load does not create these deterministic counts and does not block the next
diagnostic step. This does not claim that foreign work has zero latency effect,
that transition arithmetic is the only cost, or that removing it reaches the
latency gates.

Current decision:

```text
V10 generic executor cost             CONFIRMED
Nando as structural blocker/cause      REJECTED
ExactFusedBandTransition microproof    ADMIT NEXT PAPER GATE
full V12 executor implementation       NOT ADMITTED
full B                                 NOT ADMITTED
runtime authority changed              false
```

The next gate compares exact generic V10, equality-isolated generic recurrence,
packed fused unrolled radius-3 scalar code and an optional pre-registered SWAR
candidate on one G0-authoritative trace. Retired instructions per transition
are authoritative; loaded-host cycles and wall time are diagnostic only.

Decision receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_PMU_DECISION_2026-08-25.json`

Structural evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_WORK_A2_2026-08-25/`

M1 contract:

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_CONTRACT_2026-08-25.md`

#### Exact fused band transition M1: PASS

The run-only V2 correction reused the exact sealed M1 executable after the V1
controller terminated before package load on two nonexistent asset aliases. It
created a disjoint V2 state and consumed one fresh `parity/G0/G1/U1` marker per
route. No build, source edit, foreign-process control, host tuning, third loaded
C1 run or clean C1 marker consumption occurred.

```text
M1 ELF SHA-256                 a8fb59fb3745d5b60bf455957b0c1da200a6419b2f65ceee02a4558bf03c1e89
M1 Build ID                    31949c25f1fdb513d064b4953aea1ebc5d8828d9
V10 production prefix         39,047 B, ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
ordered transition trace      25,145,756, 6b3b5014b990c934073f1dda2972e845a6f7704c6d724c2b1e778d66ecc7cc7f
schedule parity               382/382
terminal/peak/completeness    0 / 0 / 0 mismatch
work/scratch                  0 / 0 mismatch
transition/packed state       0 / 0 mismatch
stress 23..96, radius 0..3    714,026 cases, 0 / 0 mismatch
```

The authoritative physical result is:

```text
variant   instructions/transition   reduction from predecessor   cycles/transition diagnostic
G0                    596.362                    baseline                     118.168
G1                    502.733                     15.700%                      88.717
U1                    477.598                      5.000%                      83.918

G0 -> U1 reduction                         19.915%
projected instruction delta/query       7.818 M
projected whole-query saving              18.448%
frozen promotion gate                     15.000%
verdict                                    M1_PASS
```

Equality isolation accounts for most of the measured reduction; packing,
fixed-cell unrolling and minimum fusion add another `5.00%` relative to G1.
Their combined U1 route nevertheless clears the pre-registered whole-query
instruction projection gate. Loaded-host cycles and wall time remain diagnostic
and are not used for this decision.

`M1_PASS` authorizes only a separate full-executor candidate contract and
implementation preflight. It is not a latency prediction or latency PASS, does
not admit V12, full B, runtime integration or deployment, and changes no runtime
authority. Installed Lay remained `1.0.43`; V11 SHA and the three daemon PIDs
were unchanged.

Immutable evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_2026-08-25/`

Decision SHA-256:
`f75bdc6995bcdc8553b267ae43e511321bb34fe9d4d9acb14a610104356573a1`.
Manifest SHA-256:
`775ed5125eb541b54f2e8f9a911c688258d13089ff8bbb629938395f1dbe2f94`.

#### Exact fused full executor E1: REJECT

The one-shot E1 route built one new source-preserving test executable and ran
parity, separate E0/E1 process-scoped PMU denominators and the complete loaded
latency matrix. Nando, btop and K1 remained running and were not controlled.
No clean C1 marker, third loaded E0/V10 run, full B route, V12 route or runtime
integration right was consumed.

```text
E1 ELF SHA-256                 727ba875094d3e7121330514cefee7661ecbf8dcda076a7f10631aaa2f8cd618
E1 Build ID                    47f261026bd0f3b0d2e007fe0d929aba352850a5
V10 production prefix         39,047 B, ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
schedule parity               382/382
terminal/peak/completeness    0 / 0 / 0 mismatch
work/rank/terminal rank       0 / 0 / 0 mismatch
reverse schedule              0 mismatch
stress 23..96, radius 0..3    714,026 cases, 0 / 0 mismatch
false certificates            0
maximum product states        35,590
E0 / E1 maximum scratch       6,656 / 6,144 B
```

The full physical result exceeded the pre-registered instruction gate:

```text
                              E0                 E1              reduction
instructions/query           42.910 M           23.838 M          44.446%
cycles/query diagnostic      10.289 M            6.827 M          33.653%
branches/query                5.609 M            2.572 M          54.139%

instruction saving gate                                            15.000%
model realization                                                   2.440x
physical verdict                                                    PASS
```

This proves that the generic V10 executor was a material cost, independently
of the background load. The M1 model was conservative: the integrated packed
executor removed `19.072M instructions/query`, not the projected `7.818M`.

Loaded latency improved materially relative to dirty replication V2 but did
not satisfy the conjunctive gate:

```text
metric                         V10 loaded V2       E1 loaded        gate
S pooled search p99                 3.938 ms          3.047 ms      <= 3 ms   FAIL
S pooled total p99                  4.098 ms          3.197 ms      <= 5 ms   PASS
T pooled total p99                 16.336 ms         11.512 ms      <= 5 ms   FAIL
worst run x worker total p99       58.618 ms         38.097 ms      <= 5 ms   FAIL
```

Only two of five single runs passed the search threshold; all five concurrent
runs failed the total threshold. The worst fixed-shard worker remained worker
19 and query `381` remained the worst query (`34.988 ms` total p99), so the
fairness result still mixes heavy-shard cost with concurrent execution. All ten
latency processes completed with zero errors and unresolved results. Maximum
observed temperature was `80 C`; thermal throttle counters did not change.

Post-run diagnostic aggregation of the immutable primitive samples shows that
the concurrent failure is broad rather than a worker-19-only artifact:
`90/100` `(run, worker)` p99 values exceed `5 ms`, and `187/382` query-specific
concurrent total p99 values exceed `5 ms`. In the single samples, `70/382`
query-specific search p99 values exceed `3 ms`. These are diagnostics, not new
hard conjuncts.

The frozen verdict is `E1_REJECT`: exactness and the physical instruction gate
pass, but latency is conjunctive and fails. E1 is not a production integration
candidate. The next route requires a separate paper decision for targeted
diagnosis of the remaining concurrent and non-transition costs. Full B, V12,
runtime integration and deployment remain unadmitted. Installed Lay remained
`1.0.43`; V11 SHA and the three daemon PIDs were unchanged.

Immutable evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_EXECUTOR_E1_2026-08-25/`

Decision SHA-256:
`b334c047d29b21c27923fba9b38bbf17bb642cc72c9b112add1c38d8c9b0beab`.
Manifest SHA-256:
`99c5fb3155ea0b0b66bb6295676f55f75e9aeba1ee2bc209b77d07add2f483f0`.

#### E1 Remaining-Cost D1: traversal dominates

D1 used one new source-preserving diagnostic executable after the E1 rejection.
It ran under the ordinary loaded target-host conditions without stopping,
re-affining or re-prioritizing Nando, btop or K1. The exact E1 language and
certificate route remained unchanged.

```text
D1 ELF SHA-256                 550f0d80ee49b114ac621b2f5099323480fd45847956a6807393511a8027d8fd
D1 Build ID                    9bdc0fb00420fd6358341d1a198927859cff89b8
records                        382/382
terminal/peak/work mismatch    0 / 0 / 0
false certificates             0
maximum product states        35,590
maximum scratch                6,144 B
```

The single-client thread-CPU decomposition is decisive:

```text
traversal                       95.262%
certificate                      4.458%
oracle                           0.063%
retrieval lanes                  0.043%
merge                            0.037%
EqMask                           0.030%
```

Oracle construction, lane preparation, EqMask construction and terminal merge
are therefore not material optimization levers. The remaining dominant cost is
inside packed traversal. Traversal thread CPU rose from `25.97 ns/edge` in the
single route to `44.74 ns/edge` fixed and `44.70 ns/edge` reversed under the
twenty-worker route, so the loaded degradation is not scheduler wait alone.

Moving the heavy fixed shard from E-core CPU 19 to P-core CPU 0 reduced query
`381` wall p99 from `39.655 ms` to `22.928 ms`, but thread-CPU p99 remained
`4.646 / 4.388 ms` and traversal CPU p99 remained `3.734 / 3.540 ms`.
Placement is material but insufficient.

The immutable D1 decision originally recorded
`D1_OBSERVED_WITH_CAPABILITY_GAP` because its parser incorrectly required each
separate hybrid PMU row to report `100%` running. The installed `perf 7.0.12`
manual states that hybrid `counter-value` is scaled and `pcnt-running` is the
running/enabling percentage. A separate correction reinterprets only the six
sealed `perf.raw` files with the already accepted hybrid-runtime-weighted
method; D1 was not rerun and the original decision was not modified. Correction
V1 is retained with a one-character copied subject-hash defect; correction V2
overlays only that field and is the current interpretation receipt.

```text
metric/request                   fixed           reversed
instructions                    23.935 M          23.936 M
cycles                           9.441 M           9.249 M
IPC                              2.535             2.588
branch miss rate                 1.606%            1.607%
L1 dcache loads                  4.413 M           4.412 M
LLC loads                        6,052             5,927
LLC misses                          60.9              83.3
dTLB load misses                   971               888
```

Instructions differ by only `0.003%` between mappings while reversed placement
reduces cycles by `2.03%`. The PMU correction closes the scoped parser
capability gap from sealed evidence; it does not constitute formal B.

The post-edit observed-source route is `PASS`: `10` nodes, `13` edges, `10`
routes and `20/20` exact source markers, with zero issues or warnings. This is
structural evidence only and leaves authority false.

Observed-source receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_OBSERVED_ROUTE_V3_RECEIPT_2026-08-25.json`

Receipt SHA-256:
`62c69c7b95af94dfbae7962e97cc70271a9b5cf6d620c8215140704ff9505556`.

Immutable D1 evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_2026-08-25/`

PMU interpretation correction:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_PMU_INTERPRETATION_CORRECTION_V2_2026-08-25/`

Correction SHA-256:
`004bc1f5d7cd493525cfb9287e79e8159f983b41a51a2374eaeb7931c72aad38`.
Manifest SHA-256:
`66f4d483fad44dfc92f3981a672d0a8dd0e8d04eab0f955ea9a5fb007543b180`.

The next admitted research direction is a separately contracted decomposition
inside packed traversal itself. Full B, V12, production integration and
deployment remain unadmitted.

#### D2 Traversal Internal Attribution paper contract: REVIEWED

D2 closes the paper design for external instruction-pointer sampling of the
unchanged D1 traversal. The recovered V10 source corrects one proposed premise:
`edge_range()` receives the already decoded `PackedState`, so there is one
source-level state decode per expanded state, not two. The exact 382-query
ledger is therefore:

```text
expanded states / state decodes       8,059,788 / 8,059,788
examined edges / edge decodes        25,145,756 / 25,145,756
transition calls / rank additions    25,145,756 / 25,145,756
state field-helper calls             32,239,152
edge field-helper calls              75,437,268
total field-helper calls            107,676,420
```

These are source-level calls, not machine-load or retired-instruction claims.
The future symbolized ELF and a pre-measurement `D2_BUCKET_MAP.json` must decide
the actual machine ranges for DAFSA decode/memory, transition, rank,
stack/control, terminal and unattributed work. The map is sealed before samples
are visible; no per-edge timer, counter, branch or hook is added.

The proposed primary observer is fixed-period `task-clock:u` IP sampling; the
secondary observer is a separately executed precise retired-instruction event.
Exact hybrid event syntax remains an implementation-preflight obligation. The
validity gate requires parity, exact structural counts, unsampled D1 denominator
agreement, sampled perturbation within `5%`, zero lost samples, fixed periods,
sufficient sample counts and at most `5%` unattributed traversal CPU samples.
The host remains under ordinary loaded conditions; Nando, btop and K1 are not
controlled or treated as environment blockers.

The paper review retained V1 through V3 as `VETO`. V4 uses one global sequence
skeleton plus eight independent owner routes. The skeleton is `PASS`; local
routes are `8/8 PASS`, with zero conflicts, evidence gaps, weak triads or owner
conflicts and `authority_ready=false` throughout.

Review evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_ROUTE_V4_2026-08-25/`

No implementation preflight, D2 controller, symbolized build, bucket map,
subject execution, `perf record`, PMU event or attribution result exists. D2
paper is reviewed; D2 implementation and measurement are not started. Full B,
V12, runtime integration and deployment remain unadmitted. Runtime authority
changed: `false`.

#### D2 implementation preflight P1 capability probe: BLOCKED_CAPABILITY

The two-phase implementation preflight reached `P0_STATIC_PASS_P1_REQUIRED`.
Probe-preflight V1 remains `BLOCKED_BEFORE_CODE`; V2 admitted only creation of
the benign probe controller and one three-subrun transaction. The controller
passed its unrun source/parser checks before the transaction.

The one-shot marker was consumed before the first `perf record`. `T-CAP`
started with the frozen `task-clock:u`, `100,000 ns`, CPU-0 route and produced a
`277,698`-byte `perf.data`. After the fixed two-second interval the controller
sent `SIGINT`; perf finalized the file and returned `-2`. The controller
required return code zero, classified the subrun as failed and stopped before
its perf-data readers and before `I-CORE-CAP` or `I-ATOM-CAP`.

```text
probe marker consumed                    YES
perf record invocations                    1
software-event record invocations          1
perf-data reader invocations               0
precise-instruction / PMU invocations       0
D2 ELF build / D2 subject execution      NO / NO
P1 verdict                    BLOCKED_CAPABILITY
retry permitted                           false
```

This is a controller shutdown-semantics failure, not evidence that the host
lacks task-clock or precise-instruction capability. No event identity, period,
lost/throttle, Build-ID or IP-normalization verdict is promoted from the
unread `T-CAP` data, and the two precise-instruction routes were not run.

The remote receipt originally derived invocation counts only from completed
subruns and therefore recorded zero record/software events. A local recovery
receipt preserves the immutable remote receipt and overlays only the effective
execution boundary: one task-clock record invocation, zero PMU invocations.
The first local copy failed after remote publication while renaming a copied
read-only directory. Recovery copied the already sealed remote final directly,
verified all `9/9` manifest entries and made the local evidence `0555/0444`;
it did not rerun the probe or execute perf.

Authoritative remote evidence copy:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_IMPLEMENTATION_PREFLIGHT_2026-08-25/P1_REMOTE_EVIDENCE/`

Remote receipt SHA-256:
`1c41c796458b862813601c8853788675b25b0d221b3447e16237f5f87ed6a8dc`.

Effective local recovery receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_IMPLEMENTATION_PREFLIGHT_2026-08-25/P1_CAPABILITY_PROBE_RECEIPT.json`

Recovery receipt SHA-256:
`cf8b06fb55f220d6051fcc8c1f193389481698cf0f64f31dc5109bae2e850dc5`.

Effective boundary: final D2 implementation preflight is not ready. D2 build,
bucket-map publication, subject execution and U/T/I routes remain unadmitted.
Full B, V12, runtime integration and deployment remain unadmitted. Lay remains
`1.0.43` with the installed V11 source and daemon PIDs unchanged.

#### D2 T-CAP sealed-evidence interpretation V3: RECOVERED

Shutdown-repair V2 first retained an offline-salvage V2 result of
`BLOCKED_TCAP_EVIDENCE`: all four admitted readers had completed, but its parser
required explicit zero-valued `freq` and `precise_ip` fields that the fixed-count
task-clock evlist omitted. Interpretation correction V3 admitted no new reader,
record, event, subject or D2 action. It admitted only parsing the sealed reader
outputs and copying the SHA-pinned remote `/usr/bin/yes` bytes for ET_DYN
PT_LOAD geometry.

The V3 interpreter passed its unrun source/parser checks and consumed its local
one-shot marker before copying or interpreting evidence. It recomputed:

```text
V2 reader invocations                         4
V3 perf executable / reader invocations     0 / 0
sample rows                                  4,578
yes sample rows                                927
CPU / event                         0 / task-clock:u
type / config / period                  1 / 1 / 100000
freq / exclude_kernel / precise_ip          0 / 1 / 0
lost / throttle / unthrottle                0 / 0 / 0
maps-before == maps-during                         true
normalized yes IPs                            927/927
```

The copied ELF is `31,112` bytes with SHA-256
`ee431b97fb62f59ee94fa698dbc98971001bbb1cbd9c5e32ce4ab4c5530924d8`,
Build ID `8c99ebc2c856857219acc612c2d9be3172b74be5` and type `ET_DYN`.
Every sampled yes IP joined one executable mapping to one executable PT_LOAD
and normalized inside that segment. The result is
`T_CAP_RECOVERED_FROM_SEALED_EVIDENCE`.

Authoritative V3 evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_TCAP_INTERPRETATION_V3_2026-08-25/`

Receipt SHA-256:
`f1d572a364312cc6c311ddc49316379b8b63748672c138f5fda50ba615cae2cb`.
Manifest SHA-256:
`8d4fe8a2ba0a3b7da2210ace0c9eafcee68ea39628facda3b3e8297e76a59cd3`.

Historical V1 remains `BLOCKED_CAPABILITY`; historical salvage V2 remains
`BLOCKED_TCAP_EVIDENCE`. Effective T-CAP capability is now PASS from sealed
evidence. This admits only creation of the precise-only I-CORE/I-ATOM
implementation preflight. No precise event, D2 build, bucket map, attribution,
full B, V12, integration or deployment is admitted by this result. Runtime
authority changed: `false`.

#### D2 precise-instruction capability: BLOCKED_CAPABILITY

Precise-only implementation preflight V2 was retained as
`BLOCKED_BEFORE_CODE` because two identity contracts named integration rather
than parity tests. V3 changed only those test mappings and reached
`READY_TO_IMPLEMENT` with `12` baseline checks, `17` forbidden effects, `18`
mapped tests and zero blockers.

The precise controller passed its unrun parser, route and source-veto checks.
It created separate `I-CORE` and `I-ATOM` markers and consumed only `I-CORE`.
The fixed two-second I-CORE record interval completed with perf and the benign
subject alive, unchanged CPU-0 affinity, and one controller-requested SIGINT.
Perf returned `-2`, which correctly passed the repaired controlled-shutdown
protocol, and all four mandatory readers completed with zero stderr.

The event itself matched the frozen identity:

```text
event                   cpu_core/event=0xc0/upp
type / config                        4 / 0xc0
period                              5,000,000
exclude_kernel / precise_ip              1 / 2
perf.data bytes                         60,602
sample records                              79
lost / throttle / unthrottle             0 / 0 / 0
```

However, all `79` samples carried one identical IP
`0xffffffffb3c001cd` with DSO `[unknown]`. The live `/usr/bin/yes` executable
mapping was `0x5c29e71a8000..0x5c29e71ab000`; therefore no sample IP can join the
exact yes Build ID and ET_DYN PT_LOAD. This is a genuine failure of the frozen
precise-IP capability conjunct, not a shutdown, reader, thermal, load or parser
failure.

```text
I-CORE marker                         consumed
I-CORE required IP/DSO capability    FAIL
I-ATOM marker                        available, unconsumed
I-ATOM event                         NOT RUN
verdict                              BLOCKED_CAPABILITY
retry                                forbidden
```

Authoritative precise evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRECISE_CAPABILITY_V3_2026-08-25/`

Local receipt SHA-256:
`c62ec8737cecf08e69b6b8d1ce2408e051f31086b58ff53e7b32961cad12e197`.
Remote receipt SHA-256:
`f357d86c52f478de29dda64cb9ebeb36fefee75ad7c1a7e501d9c65336ad3b3a`.
Remote manifest SHA-256:
`703d549483d30ed74a0a8c7c8e2b8940089cced65d72f1810597589b17f0d7df`.

The original D2 paper already defines
`D2_ATTRIBUTION_WITH_SECONDARY_GAP` for this case. A separate paper correction
must overlay the old final-preflight requirement before any primary-only D2
implementation can begin. D2 build and task-clock subject sampling remain
unadmitted at this point. Runtime authority changed: `false`.

#### D2 secondary-gap execution correction V5: REVIEWED

Correction V5 overlays only the obsolete requirement that both precise PMUs
must pass before any D2 implementation route can be prepared. The precise
failure proves the complete secondary channel unavailable; I-ATOM remains
unconsumed and is not run because it cannot restore the missing P-core channel.

The effective primary-only sequence is:

```text
T-CAP recovered + I-CORE required-IP FAIL
  -> D2_SECONDARY_GAP_CONFIRMED
  -> new named final implementation preflight
  -> D2-A closure
  -> one symbolized build
  -> sealed machine-code bucket map
  -> parity
  -> U-SINGLE / U-FIXED / U-REVERSED
  -> all-U perturbation gate
  -> T-SINGLE / T-FIXED / T-REVERSED
  -> at most D2_ATTRIBUTION_WITH_SECONDARY_GAP
```

No `I-*` or substitute event exists in this route. The original task-clock
period, `50,000` traversal-sample minimum, zero-loss gate, `5%` perturbation and
unattributed limits, loaded-host policy, bucket integrity and exact D1 source
requirements remain unchanged. The result cannot publish per-bucket retired-
instruction shares or admit a single-mechanism optimization paper.

The first structural worksheet V5 is retained `VETO` because it placed seven
distinct owners in one group. V6 split evidence, sequencing, decision,
admission, sampling, attribution and preflight ownership and passed with zero
conflicts, evidence gaps, weak triads, owner conflicts or negative-route hits;
`authority_ready=false`.

Correction contract SHA-256:
`c6fdde13d9a2719fd69098c35cfe34eccda1ee975111731941ed86a704332f78`.

V6 structural receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_SECONDARY_GAP_ROUTE_V6_RECEIPT_2026-08-25.json`

Receipt SHA-256:
`49be6e26558756188c697fa1ccf64378d855584c05418212913ed8c4457aa999`.

This review admits only creation of the primary-only final implementation
preflight. D2 code, build, bucket map, subject execution, task-clock sampling,
attribution, optimization, full B, V12, integration and deployment remain
unadmitted. Runtime authority changed: `false`.

#### D2 U-instruction validity correction V7: REVIEWED

Read-only source and receipt inspection found that D2 V4 attached its fixed and
reversed `<=1% instructions/request` validity conjunct to the wrong producer.
The U routes execute `d1_component_search()` with six phase clock pairs and
outer clocks, while sealed D1 G0 executes `d1_run_twenty_pmu()` through
`d1_search::<false>` and records `component_clocks_enabled=false`. Counting the
U route would include instrumentation absent from the sealed denominator;
leaving U uncounted would provide no instruction producer.

Correction V7 separates the validity families:

```text
U-SINGLE / U-FIXED / U-REVERSED
  -> exact component routes
  -> traversal thread CPU/edge delta <=5%

V-FIXED-INSTR / V-REVERSED-INSTR
  -> exact v10_d1_twenty_pmu clock-free route
  -> 20 rounds, 382 queries/round, 7,640 measured requests
  -> exact G0 events and FIFO-controlled perf stat context
  -> instructions/request delta <=1% against sealed D1 G0
```

The frozen D1 baselines are `23,934,876.5598414` fixed and
`23,935,583.225726895` reversed instructions/request. V-route hybrid rows must
use the already sealed D1 correction V2 interpretation: one counted cpu_atom
and cpu_core row per event, with scaled counts weighted by exact event-runtime
share. Cycles, branches and branch misses preserve the G0 measurement context
but do not create new hard conjuncts or scientific claims.

All U and V validity gates must pass before any T marker is consumed. Aggregate
instructions are allowed only for build-perturbation validity. Instruction-IP
attribution, per-bucket instruction shares, instruction-heavy or stall claims,
I-CORE retry, I-ATOM, event substitution and optimization remain forbidden.
The maximum result remains `D2_ATTRIBUTION_WITH_SECONDARY_GAP`.

The V7 owner-separated worksheet passed with `8/8` stable triads, zero weak
triads, conflicts, evidence gaps, foreign pulls, mixed groups, owner conflicts
or repair tasks. The receipt is coherence-only with
`authority_ready=false`.

Correction contract SHA-256:
`fe03dc42c47e3a0aac011bb24da0aeab40cf0e0c90d8bb6d6299e964b2f817af`.

V7 worksheet SHA-256:
`95f7e0f423e989d23aca629b7674bc3e60073a5ff08636aeddf3332d52c2eb2f`.

V7 structural receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_U_INSTRUCTION_VALIDITY_ROUTE_V7_RECEIPT_2026-08-25.json`

Receipt SHA-256:
`6acb942f97c78096ec9c0fc9cdaeab3bdf795c6ba1cf2ee1b84b5adafaaeb3bf`.

No `perf`, Cargo, Rust compilation, D2 subject, I-ATOM, bucket map or
measurement was executed. The next artifact may be the named primary-only
implementation preflight, but it has not been created. Full B, V12, runtime
integration and deployment remain unadmitted. Runtime authority changed:
`false`.

#### D2 primary-only implementation preflight V1: READY

The final primary-only implementation preflight froze the complete executable
graph before the one permitted symbolized build:

```text
BUILD
BUCKET-MAP
PARITY
U-SINGLE / U-FIXED / U-REVERSED
V-FIXED-INSTR / V-REVERSED-INSTR
T-SINGLE / T-FIXED / T-REVERSED
```

No other executable route is admitted. The eleven one-shot markers are consumed
before their corresponding effect, and no T marker may be consumed until parity,
all three U routes and both V routes pass. The build marker must be consumed
before Cargo; the bucket-map marker must be consumed before map generation and
publication.

The build contract permits one release build from the exact V10 prefix and exact
D1 fragment, with incremental compilation disabled and only symbols plus full
DWARF line tables intentionally retained. The assembled Rust source is
`204,722 B`, SHA-256
`6cd9edece91ac2e0c0e6dda7658e104dcf8953f1c16b1acff6108ea44ada0181`.
Any `.text` difference must be published; U and V perturbation gates, rather
than byte equality alone, decide physical admissibility.

The bucket map must be sealed before the first D2 subject. Its join identity is
the exact D2 Build ID plus normalized ELF virtual IP. PIE normalization uses an
unambiguous PT_LOAD/runtime-map load bias; pathname-base guessing and
post-sample bucket reassignment are forbidden.

U routes own only semantic, structural and traversal thread-CPU/edge validity.
V routes own aggregate instructions/request validity in the exact clock-free D1
G0 context. Only after every U/V gate passes may T use whole-process
`task-clock:u`, period `100000`, wrapping with worker inheritance. T uses no
`--pid` attach and no controller SIGINT. The frozen sampling estimator includes
one warmup traversal round plus twenty measured rounds; no time filtering or
post-hoc warmup subtraction is allowed.

NANDA implementation-preflight result:

```text
engine verdict       READY_TO_IMPLEMENT
safe_to_implement    true
blockers             0
project verdict      READY_TO_IMPLEMENT_PRIMARY_ONLY_D2
```

The engine verdict is scoped to controller creation and D2-A closure. It does
not by itself admit Cargo, a D2 subject, a PMU route, attribution, optimization,
SWAR, decoder/layout changes, full B, V12, runtime integration or deployment.
The maximum possible D2 result remains
`D2_ATTRIBUTION_WITH_SECONDARY_GAP`.

Preflight manifest:

`docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_IMPLEMENTATION_V1_2026-08-25.json`

Manifest file SHA-256:
`63c723e7ba1c5ad74ba174a3fe9100acbadbe266a8b97027cce139773a712b2f`.

Receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_IMPLEMENTATION_V1_PREFLIGHT_2026-08-25.json`

Receipt SHA-256:
`9cc3db240f4a8472a9b121b72b32acf045b74b288dcf31fa0cc109675a2389ca`.
The receipt is sealed read-only (`0444`).

No `perf`, Cargo, Rust compilation, D2 subject, I-ATOM, bucket map or D2
measurement was executed by this preflight. Runtime authority changed: `false`.

#### D2 primary-only failure dispatch correction: V4 READY

Post-preflight review found one real contradiction in V1. Its frozen taxonomy
distinguished semantic, denominator, capability, bucket-map, perturbation,
thermal, provenance and sample-coverage failures, but each U, V and T execution
step had only one unconditional `failure_state`. V1 is retained unchanged as:

```text
historical engine verdict  READY_TO_IMPLEMENT
effective status           SUPERSEDED_READY_DISPATCH_DEFECT
manifest SHA-256            63c723e7ba1c5ad74ba174a3fe9100acbadbe266a8b97027cce139773a712b2f
receipt SHA-256             9cc3db240f4a8472a9b121b72b32acf045b74b288dcf31fa0cc109675a2389ca
```

The immutable failure-state correction froze exact cause dispatch and priority:

```text
U  provenance -> thermal -> semantic -> perturbation
V  provenance -> thermal -> capability -> denominator -> perturbation
T  provenance -> thermal -> capability -> bucket-map
              -> perturbation -> sample-coverage

unknown / missing / non-unique dispatch -> BLOCKED_PROVENANCE
```

Correction V2:

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_PREFLIGHT_FAILURE_STATE_CORRECTION_V2_2026-08-25.md`

Correction SHA-256:
`53e1f0eeed7f78b597b9e3301b4f38bfa3cda562635bfd7dd9285503a5e5d0fa`.

The first corrected manifest representation used intermediate values directly
as standard NANDA `failure_state` fields. V2 was retained immutable after the
validator correctly returned eight `failure_not_terminal` plus three
`state_has_no_terminal_path` blockers:

```text
V2 verdict          BLOCKED_BEFORE_CODE
safe_to_implement   false
manifest SHA-256    fce06f65c194f48c5b98255170ffdbad556c37fe265f0e99aef4954eea8bba01
receipt SHA-256     477a2e8b70af9782250c8f84b0e3e78ce013aa2da474d7daee1667fe53e144f0
```

Schema repair V3 separated route observation from classification. Each route
first creates a sealed observed state. Guarded non-executing transitions then
select PASS, the exact frozen `BLOCKED_*` terminal, or fail-closed provenance.
An execution transition uses `BLOCKED_PROVENANCE` only when no complete sealed
observation envelope exists. The repair contract is immutable:

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_PREFLIGHT_FAILURE_STATE_SCHEMA_REPAIR_V3_2026-08-25.md`

Repair SHA-256:
`10a2f42f9a741fbaab2d7fcb79d6a5a528617f7dbd69c569458d6244be2f390c`.

V3 closed the state graph but bound one producer/consumer identity to an
integration test where NANDA requires a parity test. It was retained immutable:

```text
V3 verdict          BLOCKED_BEFORE_CODE
safe_to_implement   false
blockers             1 identity_parity_missing
manifest SHA-256    2b5c24462ea54bced01aeb00def5f7163e20b99c06093ec12a6f4543afa1d076
receipt SHA-256     35ad41d14046d97a6eced2aa69c96535819665d929290f58650481ddf09ce50a
```

V4 added the dedicated parity binding without changing dispatch or the frozen
non-dispatch core. The canonical thirteen-section core remains SHA-256
`7ec0826f0b9e954803a53b924a42bd008a9e1ff933cb3de51baf33374e24bee3`.
The executable allowlist remains exactly eleven routes. All eight U/V/T route
executions reach route-specific observed states; guarded dispatch executes no
subject, PMU event, marker consumption or rerun.

Effective preflight result:

```text
engine verdict       READY_TO_IMPLEMENT
safe_to_implement    true
blockers             0
project verdict      READY_TO_IMPLEMENT_PRIMARY_ONLY_D2
effective revision   V4
```

V4 manifest:

`docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_IMPLEMENTATION_V4_2026-08-25.json`

Manifest SHA-256:
`e9d6328b9f610ede73ae2e8d3c819b9728ac0fc5e4c263e9fe25a61978f80f5a`.

V4 receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_IMPLEMENTATION_V4_PREFLIGHT_2026-08-25.json`

Receipt SHA-256:
`740c008d59fb4689826537e46a35da554bde863358d2c18382f315395ee835e0`.

All V1 through V4 manifests and receipts, correction V2 and schema repair V3
are read-only (`0444`). V4 admits controller creation, controller self-checks
and D2-A closure only. Cargo remains forbidden until separate D2-A PASS and
prior consumption of `build.available`.

No controller, `perf`, Cargo, Rust compilation, D2 subject, I-ATOM, bucket map
or D2 measurement was executed. Runtime authority changed: `false`.

#### D2 primary-only controller and D2-A closure: PASS, STOP before Cargo

The primary-only controller was implemented with the exact eleven-route
registry frozen by effective preflight V4. Its local self-check verifies V4,
the route argv templates, forbidden-route absence, U/V/T failure dispatch,
source assembly and the zero-execution boundary. Execution actions remain hard
denied in this D2-A-only revision.

The first D2-A attempt was terminal before state publication. Controller V1
compared the full 40-hex `rustc -Vv` `commit-hash` field with the 9-hex display
hash frozen in V4. The toolchain had not drifted; the comparison representation
was wrong. V1 remains immutable and was not retried:

```text
controller V1 SHA-256       b22f895901501fee31b92d706d7b2b3e0e0977e20fe5339d2be098bbcbe206b7
V1 verdict                  D2A FAILURE: rustc commit-hash drift
state published             false
markers created/consumed    0 / 0
Cargo/rustc compilation     0 / 0
perf record/stat            0 / 0
D2 subject                  0
retry permitted             false
```

Local sealed V1 failure evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_D2A_FAILURE_V1_2026-08-25/`

Its `D2A_FAILURE.json` SHA-256 is
`5d8d1db51238adf63f0d01757b4bddca587679200091d09388919b35091d2891`;
its 33-entry `SHA256SUMS` SHA-256 is
`a0b5243c6d12383fa35c8b86819cf212b9c8b87f0b3e15c4df76e2fdf9a03473`.

Repair preflight V2 was retained as `BLOCKED_BEFORE_CODE` because its absent
baseline omitted the required empty `expect` object. Repair V3 added only that
schema repair and pinned the blocked V2 manifest/receipt. It admitted the
implementation correction:

```text
repair V3 verdict           READY_TO_IMPLEMENT
safe_to_implement           true
blockers                    0
manifest SHA-256            6589ca862a73aca61491c8b43edfc540a08ef96d8b7827a11b7ebf968b686596
receipt SHA-256             e532213da5bbe064de38f8dc0da31cc647730316f5765c5cad781e31d2252dc4
```

Controller V2 uses the disjoint task/state namespace
`slice8b-v10-e1-traversal-d2-primary-only-v2-20260825`, checks the full hash
`8bab26f4f68e0e26f0bb7960be334d5b520ea452`, and requires exact preservation
of the sealed V1 failure before V2 publication. Controller V2 SHA-256 is
`9329a32b9e4e9edf5d83ddf624e8c9ce6a456494057f4ea3ef5aff6f382d6ec0`.

V2 D2-A then closed successfully:

```text
controller state            PRIMARY_ONLY_CONTROLLER_VERIFIED_UNRUN
D2-A verdict                D2A_CLOSED_ALL_MARKERS_AVAILABLE
routes in registry          11
markers expected/created    11 / 11
markers consumed            0
marker file mode            0400

assembled source bytes      204,722
assembled source SHA-256    6cd9edece91ac2e0c0e6dda7658e104dcf8953f1c16b1acff6108ea44ada0181
production prefix bytes     39,047
production prefix SHA-256   ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
compiled                    false

Cargo invocations           0
rustc compilations          0
perf record/stat            0 / 0
PMU events opened           0
D2 ELF / subject            NO / NO
bucket map                  NO
parity / U / V / T          0 / 0 / 0 / 0
runtime authority changed   false
```

The available markers are exactly:

```text
build.available
bucket-map.available
parity.available
u-single.available
u-fixed.available
u-reversed.available
v-fixed-instr.available
v-reversed-instr.available
t-single.available
t-fixed.available
t-reversed.available
```

Local sealed V2 result:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_V2_2026-08-25/`

The authoritative copied `D2A_RECEIPT.json` SHA-256 is
`998ca180a976384acb215b9e72a8d956fd830fdf6a1c0641b59eea10cbb00e0f`;
the local audit receipt SHA-256 is
`e5277fdd7472325d6589bcfabc782d9b824fc1fe20beee2cb1ed4419e2b412bc`.
The local 41-file evidence tree passes its complete `SHA256SUMS` and is sealed
read-only. Runtime before and after remained Lay `1.0.43`, active V11 SHA-256
`d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b`
and PID sets `2076194 / 3410795 / 3410820`.

This pass did not test or execute BUILD, bucket-map generation, parity, U, V,
T, sampling, attribution, optimization or runtime integration. Its positive
scope ends at D2-A. The next admitted action is an independent audit of this
D2-A state. `build.available` remains unconsumed, and this transaction stops
before Cargo.

#### D2-A independent audit V2: PASS, build admitted but unexecuted

A separate read-only auditor was created without importing or modifying the
D2-A producer controller. Its only external command route is an SSH-wrapped
Python projection that reads the sealed D2-A tree and live marker state. It has
no remote write, Cargo, rustc, perf, PMU, subject, ELF, bucket-map, parity, U, V
or T route.

Audit V1 failed closed before its first remote read because the generated
projection placed `pathlib.Path` constants before `import pathlib`. It published
`BLOCKED_PROVENANCE`, admitted no build and changed no marker. V1 was not
repeated and remains immutable:

```text
auditor V1 SHA-256          f596ff43b216a5b2a8d20d1bc04d3fbbfb38b548676a441490a1e61059c8a028
V1 receipt SHA-256          41b6928566f577a7cd39f6ec9610a89fa4832473fbc838866a9a0626d87d6e21
verdict                     BLOCKED_PROVENANCE
checks completed            0
build admitted/executed     false / false
marker mutations/consumed   0 / 0
remote writes               0
```

V1 evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_D2A_INDEPENDENT_AUDIT_V1_2026-08-25/`

Audit V2 uses a separate source and result namespace, preserves V1, compiles
the remote projection during self-check and freezes the import before all path
constants. Auditor V2 SHA-256 is
`28199196ad542d77063962b128d8d82db92a9864ae951e151d0da2f70d361c68`.

The independent audit result is:

```text
verdict                     D2A_AUDIT_PASS_BUILD_ADMISSION
checks                      35 / 35 PASS
live projections            2
live projection stable      true

authoritative receipt       998ca180a976384acb215b9e72a8d956fd830fdf6a1c0641b59eea10cbb00e0f
producer controller         9329a32b9e4e9edf5d83ddf624e8c9ce6a456494057f4ea3ef5aff6f382d6ec0
remote evidence manifest    8d9581f8b4bce2b8cd99683c3b718ef3ea338bf97ab01f3ab14186bda09319c7
STATE.json                  fb7de0be1dbb7a99c2ddcb2bd1dbc7f469d4fc975b6a564546fd6994c196075a
route.lock                  ddfafcaec3c8068ea0b853cb8a34cf0b40408fbbdc137a6dae3932b5396c3c5d

D2-A tree                   present and immutable
D2-A failure tree           absent
marker names/routes         exact 11 / 11
marker modes                0400
markers consumed            0
build.available             available
D2 ELF under task parent    absent

remote writes               0
marker mutations            0
Cargo/rustc compilation     0 / 0
perf record/stat            0 / 0
PMU / D2 subject            0 / 0
bucket map / parity         false / false
U / V / T                   0 / 0 / 0
runtime authority changed   false
build executed              false
stop before build           true
```

Both live projections have identical evidence-file SHA-256
`7364988d70ddc32891ba827dba8bec4235e89310b7fa9b46db00ffedb4cbfbcf`.
Their canonical projection digest recorded by the auditor is
`0a4766cc6b5c50eca91b7235c4156982d6e1289ffee2179588a74798cea63cf9`.

Authoritative V2 audit evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_D2A_INDEPENDENT_AUDIT_V2_2026-08-25/`

The audit receipt SHA-256 is
`ca681196361bb434d16fc334e4481609441e891c76d4e5f3728f93be945d4168`;
its complete evidence manifest SHA-256 is
`d6140d4c18e6fd1138a670850cf1bfa7544b6fa0a21edfdd3e349b88c1080869`.
All five manifest members pass and the evidence tree is read-only.

This PASS changes only the admission boundary: a separate post-D2A controller
may now verify this receipt and live `build.available`, consume that marker
before one Cargo invocation, preserve the resulting ELF and stop at
`D2_BUILD_CREATED_UNAUDITED`. No post-D2A controller or build was created or
executed in this audit transaction. The original D2-A producer controller
remains unchanged and still rejects every execution action.

#### D2 post-D2A symbolized build V1: created, unaudited

A separate post-D2A controller was created after the independent audit admitted
the build. The D2-A producer and auditor sources remain byte-identical. The new
controller exposes only `self-check` and `build-once`; its read-only self-check
verified the authoritative D2-A/audit receipts and the live `11 available / 0
consumed` marker projection before the build transaction.

The one-shot transaction prepared and fsynced the isolated workspace and
`PREBUILD.json`, atomically renamed `build.available` to
`build.consumed-before-exec` under `route.lock`, and then made exactly one
frozen Cargo invocation. Cargo completed successfully in 6m 51s and produced
one candidate test ELF in the fresh target directory.

```text
verdict                     D2_BUILD_CREATED_UNAUDITED
post-D2A controller         b7c1ddf53678bdd63affac9df9db63a2c470af88d6104f8969e8cca4f066102f
D2-A receipt                998ca180a976384acb215b9e72a8d956fd830fdf6a1c0641b59eea10cbb00e0f
independent audit           ca681196361bb434d16fc334e4481609441e891c76d4e5f3728f93be945d4168
producer controller         9329a32b9e4e9edf5d83ddf624e8c9ce6a456494057f4ea3ef5aff6f382d6ec0

assembled source            204,722 B / 6cd9edece91ac2e0c0e6dda7658e104dcf8953f1c16b1acff6108ea44ada0181
production prefix            39,047 B / ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
Cargo exit                  0
Cargo invocations           1
candidate ELF               317,706,232 B / bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178
candidate mode              0555

build marker                consumed before Cargo
other markers consumed      0
other markers available     10
remote build state          D2_BUILD_CREATED_UNAUDITED
remote/local manifest       524 / 532 entries PASS
remote build receipt        a49a05bdf95ddbcc5cad78ef2376e861498a4311828d06abec83404f66b50953
local build receipt         5bed316600b1e04b7973d78697f5133f22fc3264f7027ddeafba4ad3360df240
local evidence manifest     aead466e48392f22db9394fc601724b7d8e90930f515c993390c37ab4702f28b

ELF executed                false
ELF scientific audit        false
bucket map / parity         false / false
U / V / T                   0 / 0 / 0
perf record/stat            0 / 0
PMU / D2 subject            0 / 0
runtime authority changed   false
```

The sealed candidate is:

`/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-d2-primary-only-v2-20260825/build-v1/d2-test-elf`

Local build evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_BUILD_V1_2026-08-25/`

This transaction tested only whether the frozen source and symbol-retaining
release environment can produce one sealed candidate under the admitted
one-shot protocol. It did not test Build ID, ELF type, `.text`, `PT_LOAD`,
symbols, DWARF, semantic parity, perturbation, sampling, or latency. The only
next admitted action is an independent read-only D2 build audit; no rebuild,
bucket-map generation, subject execution, or optimization is admitted here.

#### D2 independent build audit V1: PASS, bucket-map admitted but unexecuted

A separate auditor exposed only `self-check` and `audit`. It read the sealed D2
ELF locally, queried the sealed D1 ELF and current D2 marker projection
remotely, and published an immutable local receipt. It did not execute either
ELF or invoke Cargo, rustc compilation, perf, PMU, a D2 subject, parity, U, V,
T, or bucket-map generation. The audit ledger records three `cargo -V` and
three `rustc -Vv` queries: one projection in self-check and before/after
projections in the audit transaction.

```text
verdict                     D2_BUILD_AUDITED
auditor SHA-256             b0fe23eb6ad13128bdbc10060db6632ed820221ac86eefa70b32f6073c2390d9
audit receipt SHA-256       4e19e2e806e2b3f04f8c0286a63f64631e7cf6ff143d9a90da4036654bc8339c
evidence manifest SHA-256   f8b31c0ce3bc56ca63f8f93f4ce763d54088b3e269e22594a4c23c847966725a

D2 ELF                      317,706,232 B / bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178
D2 ELF mode/type            0555 / ET_DYN PIE
D2 Build ID                 eb951f1a7526a9f1cb365040c10989aa5d3fc50f
D2 .text                    15,980,919 B / f57eba60bc4b1cadbeb2dfc524af59a7ab011a2e64afb0e1a0fe610755129d94
D1 .text                    15,938,743 B / 7336c3897a87172bf5175574d329196b84d43d499bc1f9e9274ecbd40889993b
.text byte-identical        false
.text size delta            +42,176 B / +0.2646130877%
causal explanation          NOT_ESTABLISHED
physical admissibility      owned by future U/V perturbation gates

executable PT_LOAD          unique
.text inside exec PT_LOAD   true
remaining executable slack 313 B
.symtab / .strtab           present / present
required DWARF sections     present
hot symbol start/size       0x778320 / 4,238 B
hot source resolution       v13_typed_peak.rs:3105 and inline return frames

remote projection stable    true
remote writes               0
marker mutations            0
build marker                consumed
bucket-map marker           available
other markers               9 available / 0 consumed
bucket-map artifacts        absent
runtime authority changed   false
```

The sealed evidence is:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_BUILD_AUDIT_V1_2026-08-26/`

All twelve files are read-only and all eleven non-manifest members pass the
complete `SHA256SUMS`. The receipt's `auditor.path` and `auditor.mode` describe
the staging copy before atomic publication and sealing (`0755`); the published
copy is mode `0555` with the same SHA-256 and is bound by `SHA256SUMS`. This is
a metadata timing note, not evidence of source drift.

This audit establishes ELF suitability for the next map-construction gate. It
does not establish semantic parity, build perturbation, sample coverage,
machine-range bucket correctness, attribution, concurrency cause, or an
optimization decision. A separate bucket-map controller may verify this audit,
consume `bucket-map.available`, construct and seal `D2_BUCKET_MAP.json`, and
must stop before consuming `parity.available`.

#### D2 bucket-map implementation route selected before marker consumption

The map implementation uses only the sealed D2 ELF, embedded DWARF, the sealed
build source snapshot and the effective V4/V7 contracts. Dirty live workspace
Rust is not a classification input. The selected map is a non-overlapping full
partition of the D2 `.text` virtual-address interval: the hot
`d1_enumerate_lane_prepared::<false>` instructions and its exclusive
`V13DafsaView::state` and `V13DafsaView::edge` callees receive frozen mechanism
ownership, unresolved hot instructions receive `UNATTRIBUTED`, and the
remaining `.text` complement receives `OUTSIDE_TRAVERSAL`.

Classification is frozen before samples exist. Embedded inline frames own the
unambiguous alphabet, equality, fused transition and terminal-distance spans.
Exact machine intervals own compiler-hoisted or merged blocks whose semantics
are clear from the sealed disassembly but whose line table is insufficient.
No later sample can alter a range. `REDUNDANT_STATE_DECODE` remains explicitly
absent unless the one-shot disassembly proves another state-record decoder.

The map gate requires exact ELF SHA and Build ID, executable `PT_LOAD`
containment, instruction-aligned monotonic ranges, full `.text` byte coverage,
zero overlap, zero gap, and a SHA-256 of the exact machine bytes for every
range. `bucket-map.available` is atomically renamed before authoritative map
generation. Any failure retains the consumed marker and evidence and admits no
subject or retry. A separate read-only map audit must pass before
`parity.available` can be consumed.

Source-line-only bucketing was rejected because LLVM moved transition
precomputation ahead of the edge loop and merged several operations under
coarse line records. Per-instruction full-ELF `addr2line` bucketing was rejected
because unrelated `.text` does not need artificial mechanism ownership. The
selected partition keeps deterministic joins for every D2 `.text` IP without
inventing ownership. This controller changes no runtime code, package, cache,
daemon, IME path or installed authority; its removal boundary is the sealed D2
evidence tree after this research route terminates.

#### D2 bucket-map reader scope correction V1: repaired before marker

The first bucket-map preparation attempt stopped before authority consumption.
The V4 full-ELF demangled `objdump` used one CPU for more than 1,800 seconds on
the 317,706,232-byte LTO+DWARF ELF and exceeded the local controller timeout.
At timeout `bucket-map.available` was still exact and no map, subject, Cargo,
perf or PMU action existed. The controller-owned orphan reader was terminated
after exact PID/argv verification; its parent removed the pre-marker stage.

The effective reader overlay replaces only that full disassembly with three
address-bounded disassemblies over the sealed hot, `edge` and `state` symbols.
All symbol boundaries, classification rules, direct ELF byte hashes, full
`.text` complement coverage and one-shot semantics remain unchanged. This is
an implementation reader-scope correction, not a repeat of an experiment and
not a change to the D2 scientific contract.

Correction:

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_BUCKET_MAP_READER_SCOPE_CORRECTION_V1_2026-08-26.md`

V1 proved insufficient on the remote binutils build: even an address-bounded
`objdump --line-numbers` spent more than ten minutes indexing the full DWARF
image before producing output. That second preparation was also terminated
before marker consumption and its stage was removed. Effective correction V2
drops only duplicate `objdump` line lookup. Exact instruction bytes and
assembly still come from three bounded `objdump` calls; source and inline-frame
ownership still comes from the separate per-instruction `addr2line` reader. A
read-only timing probe of the effective hot-symbol command completed in 0.69 s.

Effective correction:

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_BUCKET_MAP_READER_SCOPE_CORRECTION_V2_2026-08-26.md`

V2 still left source and inline-frame resolution on remote GNU binutils 2.38.
That bounded `addr2line` preparation remained active for more than twelve
minutes without output. It was terminated before marker consumption and its
pre-marker stage was removed. Effective correction V3 moves only this reader
to the byte-identical local sealed ELF. Remote `objdump`, symbol and ELF
geometry readers, direct byte hashes, address-list reconstruction, marker
consumption and map publication remain remote-owned.

The local reader closure was reproduced twice before implementation: 1,064
instruction starts, address-list SHA-256
`fca1804a3d0af34ae462938f970fd181706846da6e42dbb725c5ef1775581c58`,
and 697,799 bytes of `addr2line` output with SHA-256
`8b9b4767557a3ea019bbaebb280d1a56ab2180f34ad1e05aed9c2affb4c8a9e6`.
The remote producer must reproduce the same address-list SHA from its own
bounded disassembly and verify every transferred identity before consuming the
one-shot marker.

Effective correction:

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_BUCKET_MAP_READER_SCOPE_CORRECTION_V3_2026-08-26.md`

#### D2 primary-only bucket map and independent audit: PASS

The effective V3 controller passed static closure and a live read-only remote
probe, then consumed `bucket-map.available` exactly once and published the
sealed machine map. No Cargo, perf, PMU, D2 subject, parity, U, V or T route ran
during map construction. The local sealed map identity is:

```text
verdict                    D2_BUCKET_MAP_SEALED
map SHA-256                2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846
ranges                     46
.text covered bytes        15,980,919
gaps / overlaps            0 / 0
instruction starts         1,064
address-list SHA-256       fca1804a3d0af34ae462938f970fd181706846da6e42dbb725c5ef1775581c58
machine hash mismatches    0
pre-sample ambiguous bytes 139
runtime authority changed  false
```

The separate read-only auditor did not import producer code. It reran bounded
objdump and readelf over the byte-identical sealed ELF, directly rehashed all
46 machine ranges, independently compared the 42 mechanism ranges and four
`.text` complement ranges against the frozen classification, rebuilt source
frame ownership from the sealed addr2line stream, and verified identical live
remote projections before and after the audit. Its receipt is:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_BUCKET_MAP_AUDIT_V1_2026-08-26/D2_BUCKET_MAP_AUDIT_RECEIPT.json`

```text
verdict                    D2_BUCKET_MAP_AUDITED
receipt SHA-256            8d2e52574adca52d5a090067cdf071afba922363ccee8fc9eb4a4158cb92cba7
map marker                 consumed-before-exec
parity marker              available
other route markers        9 available
subject/perf/PMU           0 / 0 / 0
```

This establishes deterministic pre-sample machine-IP ownership only. It does
not establish parity, U/V build perturbation validity, sample coverage,
bucket CPU shares, concurrency inflation cause or an optimization decision.
The next admitted irreversible action is the one-shot semantic parity route.

#### D2 primary-only semantic parity: PASS

The separate parity controller verified the audited map, exact D2 ELF, sealed
D1 parity reference, four immutable inputs and the live eleven-marker
projection. It atomically consumed `parity.available`, executed only
`v10_d1_semantic_parity`, retained the complete subject evidence and stopped
before every U, V and T route.

```text
verdict                    D2_PARITY_PASS
remote receipt SHA-256    d2519797c6b21976dc6246830e187391a344acb9ec63a7b5a7816161cf306f74
semantic mismatches       0
records / schedule        382 / 382
target form retained      382
target lemma retained     382
expanded states           8,059,788
examined edges            25,145,756
stress cases              714,026
parity marker             consumed-before-exec
remaining markers         8 available / 3 consumed
Cargo / perf / PMU        0 / 0 / 0
runtime authority changed false
```

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_PARITY_V1_2026-08-26/`

This proves semantic equivalence for the frozen D2 subject only. It does not
establish build-perturbation validity, task-clock sample coverage, bucket CPU
shares, concurrency inflation cause or any optimization authority. The next
admitted route is `U-SINGLE`. The post-parity U/V controller has separately
passed AST/registry closure, offline replay of all three D1 component streams,
exact replay of both sealed runtime-weighted G0 aggregates, and a live
read-only `U-SINGLE` admission probe; no U/V marker was consumed by those
checks.

#### D2 primary-only U/V build validity: PASS

The U/V sequence consumed each admitted marker once and preserved the exact
382-query, twenty-round structure, zero errors/unresolved results, frozen CPU
mappings and zero measured worker migrations. The original U-SINGLE controller
misread the valid single-worker sentinel as a semantic failure. A separate
offline reader recovered its exact sealed component stream without another
subject execution; the historical receipt and consumed marker remain
unchanged.

```text
route              effective verdict              frozen delta
U-SINGLE           U_SINGLE_RECOVERED             0.3674526002% CPU/edge
U-FIXED            U_FIXED_PASS                    1.1703050984% CPU/edge
U-REVERSED         ALL_U_PASS                      0.4414835836% CPU/edge
V-FIXED-INSTR      V_FIXED_PASS                    0.0081367930% instructions/request
V-REVERSED-INSTR   ALL_UV_VALIDITY_PASS            0.0118180937% instructions/request
```

The authoritative result identities are:

```text
U-SINGLE salvage receipt    9617502776537ca4181bd9bf195e1fd5b8fbd2679f1dfd00737f128cb88bfe0b
U-FIXED remote receipt      0ef07db4b8a07efb2ed09c3c47d6b0a9ef88e4529dd436d5d375cecf62339b59
U-REVERSED remote receipt   080917d52f3e36abffb6eab47b9e56c4a1e771b4da21caaf6fa3e2cfe686a0fb
V-FIXED remote receipt      56c862759c95de6682571aed3d68098dab084319817fba5af3853042e0396bae
V-REVERSED remote receipt   5d75a502ad9e509dc6810b5494067f602d5ace1237e73f4d7913d5b9b1fc9de2
```

The V routes used exactly one `perf stat` each with the frozen G0 event group
and runtime-weighted hybrid aggregation. Their instructions/request values
were `23,936,824.091197852` fixed and `23,932,754.496077046` reversed. U opened
no perf event. No Cargo build, runtime integration or installed-authority
change occurred in U/V.

This establishes only that the one symbolized D2 build stays within the
frozen CPU and aggregate-instruction perturbation limits. It does not establish
machine-IP attribution, bucket shares, concurrency-inflation cause or an
optimization decision. After `ALL_UV_VALIDITY_PASS`, only `T-SINGLE` was
admitted.

#### D2 T-SINGLE execution and terminal audit: BLOCKED_PROVENANCE

The T controller's closed registry contained only T-SINGLE, T-FIXED and
T-REVERSED. T-SINGLE atomically consumed its marker and executed once through
whole-process command wrapping with `task-clock:u`, fixed period `100000`,
inheritance, no PID attach and no SIGINT lifecycle. Perf record, all four
readers and the D2 subject returned zero. The immutable historical publication
is:

```text
historical verdict          BLOCKED_PROVENANCE
remote receipt SHA-256      afaeb7d3caffb1967dd76021e42b94664803cef5d0ed72ec574fb54526a8fa0d
local receipt SHA-256       8e20921889924dc7967c770bac7827b4e4553a68aecd9c08617a17db9538b10c
local controller SHA-256    9634d03afe0280fe78a240b3c67145e80d9977679c2c3cb7f2086de7645f5014
remote controller SHA-256   428452cd2937deb644299d8d1ad0280d649eeaf7da6b3b05452a07f26b11146b
retry                       forbidden
runtime authority changed   false
```

The complete sealed evidence independently replays to:

```text
perf samples                127,143
D2 samples                  123,718
traversal-range samples     106,901
lost                        0
throttle / unthrottle       15,847 / 15,847
UNATTRIBUTED                0%
unique load bias            0x711195a00000
Build ID / map bytes        exact / 0 mismatches
subject errors/unresolved   0 / 0
subject affinity/migrations [0] / 0
thermal throttle drift      0

sampled traversal CPU/edge  21.256270839501 ns
paired U CPU/edge           26.060419547537 ns
absolute delta              18.434656047164%
frozen maximum              5%
```

The controller selected `traversal sample CPU outside frozen mapping`, but V4
contains no sample-CPU-subset hard gate. Exact subject affinity and measured
migrations already own that obligation. The historical receipt remains
immutable, while the effective interpretation withdraws that non-contractual
reason.

The offline terminal audit found a real, higher-priority provenance defect.
Whole-process recording begins before `d1_load_inputs()`, while the frozen
estimator says the traversal stream is only one warmup plus twenty measured
rounds and forbids timestamp filtering. Before `d1_pin_current_thread(0)`, the
worker produced 4,395 D2 samples on CPU6. Of these, sixteen landed in the
sealed shared `V13DafsaView::edge/state` traversal ranges:

```text
EDGE_DECODE       6
STATE_DECODE      8
SYMBOL_DECODE     2
```

They come from the dense-alphabet validation inside `d1_load_inputs()` and
cannot be separated by the IP-only bucket map or removed under the frozen
no-filter rule. The actual traversal attribution stream therefore differs
from the preregistered estimator. Effective dispatch remains
`BLOCKED_PROVENANCE`, now for estimator-scope route/evidence drift rather than
the invalid CPU predicate. Lower-priority failures remain independently true:
sampled-vs-U exceeds 5%, and throttle/unthrottle is not zero. Even a forbidden
counterfactual subtraction of all sixteen samples leaves an 18.446864029346%
perturbation failure.

Correction:

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_T_SINGLE_ESTIMATOR_SCOPE_CORRECTION_V1_2026-08-26.md`

Terminal receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_T_SINGLE_TERMINAL_AUDIT_V1_2026-08-26/T_SINGLE_TERMINAL_AUDIT_RECEIPT.json`

```text
correction SHA-256          88c12093bb3cf76395b3f5c37d48991ebb90a3c34a6581d5801f3cd4fb2001f4
terminal receipt SHA-256    75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608
historical manifests        35 local / 25 remote entries PASS
live projection             stable before/after
t-single                    consumed-before-exec
t-fixed / t-reversed        available and unconsumed
offline perf/PMU/subject    0 / 0 / 0
remote writes               0
runtime authority changed   false
```

The observed bucket values are retained only as scientifically invalid
diagnostics because estimator scope, perturbation and sample coverage all
failed:

```text
TRANSITION                  15.6998660 ns/edge
DAFSA_DECODE_MEMORY          3.5648163 ns/edge
STACK_CONTROL                1.7849931 ns/edge
TERMINAL                     0.2024198 ns/edge
RANK                         0.0041757 ns/edge
```

No D2 attribution claim is established, and these numbers do not explain the
frozen `+18.77 ns/edge` concurrency inflation. T-FIXED and T-REVERSED are not
admitted, no D2 retry is permitted, and no SWAR, decoder/layout, rank, stack,
runtime integration or deployment action follows. A new paper route would be
required for any further measurement.

#### D3 estimator recovery execution: terminal BLOCKED_PROVENANCE

D3 reused the exact audited D2 ELF and bucket map and attempted only the new
single-worker estimator envelope. Its terminal result is:

```text
verdict                     BLOCKED_PROVENANCE
selected cause / rank       provenance / 0
U2 marker                   consumed-before-exec
U2 subject executions       1
T2 marker                   retired unconsumed
T2 / perf / PMU             0 / 0 / 0
retry                       forbidden
optimization authority      false
runtime authority changed   false
```

Before bootstrap, offline replay found and repaired two controller defects:
the D3 map reader used a nonexistent nested `map.elf` schema, and the immediate
route stage was initially not traversable by subject user `e`. Parser replay
then read all 127,143 historical D2 samples, matched all raw records and
validated all 46 sealed map ranges with zero machine-byte mismatches. The
controller self-check closed the exact `U2-SINGLE -> T2-SINGLE` graph and the
independent bootstrap audit observed two exact `0400` available markers.

The bootstrap audit nevertheless admitted execution incorrectly. The
authoritative root-owned D3 parent itself had mode `0700`. Although the route
stage was `0755` and the subject directory was owned by `e`, the subject could
not traverse the parent. The same boundary first prevented the non-authority
local `scp` mirror; the audit recorded that fact but incorrectly treated it as
non-scientific.

The one-shot U2 subject then exited `101` while publishing its receipt:

```text
write D1 subject receipt: ".../subject/SUBJECT_RECEIPT.json:
Permission denied (os error 13)"
```

No subject receipt, component samples or structure evidence was produced.
The immutable U2 receipt therefore correctly selected
`BLOCKED_PROVENANCE` for unavailable required evidence. T2 was not admitted,
so `task-clock:u @ 200000 ns`, sample coverage, throttle behavior, T2/U2
perturbation and bucket attribution were not tested. This result says nothing
negative about D2 ELF semantics, task-clock capability or the role-aware
estimator itself.

The historical bootstrap audit remains immutable, but its positive execution
admission is withdrawn by:

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_BOOTSTRAP_EXECUTION_ADMISSION_CORRECTION_V2_2026-08-26.md`

Terminal receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_TERMINAL_AUDIT_V1_2026-08-26/D3_TERMINAL_AUDIT_RECEIPT.json`

```text
D3 paper SHA-256             ebe80974392a05527bea67944f381cfd2f74fb0be1c5b2ba3bf4a5aba22be11a
bootstrap receipt SHA-256    a7b921799751f38f745a2945ff8b7222428ff16c54e42c35e0e0d99019468529
bootstrap audit SHA-256      0c5a34b1809dbbfa8b3744b65d86f3dfa1d0c1bfb00a623a0bbe5089669b7bb1
correction SHA-256           b963c0059fe6efaca746fad2ad9dc4784c7a6d4b9ea524c9faa18c6858197519
U2 receipt SHA-256           7c74a689079b8c40442c8065ce73a5deb69d990daf3a6404b43461808744888e
U2 remote SHA256SUMS         76469bd56d932f27cfc10610c8829f4864d74450d17f95f5cc9fb55699ec071d
terminal receipt SHA-256     7f4c0fd9466cced361c21858ea2017910adc7b92db7c1edfe02f3be09d5c4299
terminal SHA256SUMS          5e62f3ccb5bc97e8d2a11732c02623ad312edc721c0db066b93cf9e8856f84b9
live remote projection       stable before/after
D2 marker mutations          0
```

D3 is terminal. No permission repair, U2 retry, T2 run, new ELF, new bucket
map, optimization, runtime integration or deployment follows. Any future
measurement requires a new paper namespace whose pre-marker self-check proves
the complete directory ownership and traversal chain for the actual subject
UID.

#### D4 single-worker estimator recovery: D4_SINGLE_ESTIMATOR_PASS

D4 repaired only the D3 execution-admission architecture. It reused the exact
audited D2 ELF, Build ID, bucket map, parity and U/V build-validity evidence.
D2 and D3 remained terminal and byte-immutable.

The new namespace was created without scientific markers. An actual helper
executed as UID `e` through the exact future parent chain, read and hashed the
D2 ELF and map, and completed create/write/fsync/rename/reopen/read/unlink
operations. A separate auditor then copied the sealed bootstrap through real
`scp` as `e` and found byte-identical evidence. Only that PASS admitted one
marker-creation action; a second independent audit verified two exact `0400`
markers with zero consumed before U3.

```text
D4 parent mode                  root:root 0755
UID e capability proof         PASS
scp as e                       PASS, byte-identical
markers before access audit    0
markers created / consumed     2 / 0
bootstrap audit SHA-256        233fcea000bbdf9edd6325bccf3c5179502a8ba171763d4a566bd75bf0774695
marker audit SHA-256           af9f8f13762d51a427d6bebee6c9c58fddd0741f52e4ceaafd78a2c5c8dcaf57
```

U3 then consumed its marker and executed once with no perf or PMU. It proved
the exact single-worker semantic and structural envelope with 382 queries,
one warmup, twenty measured rounds, CPU0 final affinity, zero measured
migrations, zero errors/unresolved and the exact 25,145,756 edges per round.

```text
U3 denominator                 502,915,120 edges
U3 traversal thread CPU        13,113,342,336 ns
U3 CPU/edge                    26.07466312804435 ns
U3 receipt SHA-256             db2ba1b3d4e11ac2c4edb24e382f93d282b1b5d605fcbb1636f5e346030dc000
```

Only U3 PASS admitted T3. T3 consumed its marker and executed exactly one
whole-process `task-clock:u` recording at fixed period 200,000 ns. All four
mandatory readers passed. The preregistered CPU0/Build-ID/normalized-IP stream
excluded eleven staging traversal samples and used no timestamp filter or
warmup subtraction.

```text
event                           task-clock:u
period / freq                   200000 / 0
exclude-kernel / precise-ip     1 / 0
inherit                         1
record / readers / PMU          1 / 4 / 1
record exit                     0
lost                            0
throttle / unthrottle           0 / 0
sample-rate before / after      8000 / 8000
thermal throttle drift          0
normalization                   unique
machine-byte mismatches         0
accepted traversal samples      66,543
minimum                         50,000
UNATTRIBUTED                    0 / 0%
T3 CPU/edge                     25.20277605266102 ns
paired delta                    3.343809548379481%
maximum                         5%
T3 receipt SHA-256              dd4e3b7bb49d368fe1461c36fda0968af629293e801500770ca9dc3715a96f09
```

The independent terminal audit repeated local manifests, exact live consumed
markers and route states, remote/local route receipt identities, D2 ELF/map
identity, D2 marker stability, complete D3 tree stability, absence of active D4
subjects and installed-runtime stability. Its verdict is:

```text
verdict                         D4_SINGLE_ESTIMATOR_PASS
terminal receipt SHA-256        f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b
markers created / consumed      2 / 2
retry                           forbidden
runtime authority changed       false
optimization authority          false
```

The accepted single-worker traversal attribution is now scientific evidence:

```text
TRANSITION                      49,104  73.7928858032%
DAFSA_DECODE_MEMORY             11,138  16.7380490810%
STACK_CONTROL                    5,682   8.5388395474%
TERMINAL                           610   0.9167004794%
RANK                                 9   0.0135250890%
```

Within `TRANSITION`, `FUSED_SCALAR_U64_ADVANCE` owns 44,922 accepted samples;
`EQUALITY_WINDOW` owns 3,417 and `ALPHABET_ID` owns 765. This establishes the
dominant mechanism in the valid single-worker baseline. It does not yet
explain the frozen `+18.77 ns/edge` fixed/reversed concurrency inflation,
because no corrected multiworker/TID estimator was executed in D4.

Paper:

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_TASK_CLOCK_ESTIMATOR_RECOVERY_V1_2026-08-26.md`

Terminal receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_TERMINAL_AUDIT_V1_2026-08-26/D4_TERMINAL_AUDIT_RECEIPT.json`

D4 admits only a separate paper decision for a multiworker/TID estimator or
the measured dominant bucket. It does not authorize SWAR, decoder rewrite,
another build, integration, install, restart or deployment.

#### D5 multiworker/TID implementation: local static closure, remote unrun

The effective D5 V3 preflight remains the implementation admission. Its
sealed verdict is `READY_TO_IMPLEMENT`; all fifteen split structural routes
remain PASS under the aggregate `STRUCTURALLY_ACCEPTED_WITH_SPLIT` boundary.
The five future producer/auditor sources now compile locally and expose only
the frozen four-route registry:

```text
U4-FIXED
T4-FIXED
U4-REVERSED
T4-REVERSED
```

Static command-graph validation found `perf record` reachable only from the
two T4 routes, no `perf stat`, no PID attach, no signal lifecycle, and exact
U4/T4 denominators of 502,915,120 / 528,060,876 edges. Canonical marker bytes
match all three producer, marker-auditor, and terminal-auditor registries.

The post-consumption fault path was also closed before any remote action. A
controller exception after the atomic available-to-consumed rename now emits
a sealed `BLOCKED_PROVENANCE` route receipt. It preserves partial scientific
files, retains the consumed marker, leaves later markers available, publishes
an exact state-to-receipt link, and keeps unknown execution counters as
`null` rather than claiming zero. Local fault injection passed for U4 and T4,
including the narrow failure window immediately after marker rename and before
the consumption helper returns.

An offline replay of the sealed D4 stream reproduced the exact CPU0 result:
66,543 accepted samples and byte-identical bucket/sub-bucket counts. The same
worker TID had eleven traversal-mapped samples on staging CPU6; this confirms
that the D5 worker-role estimator is materially distinct from the D4 CPU0
filter. It is not a D5 result. Synthetic twenty-worker graph tests accepted
the exact fixed/reversed singleton CPU closures and rejected a multi-CPU
worker, a missing worker sample, and an ambiguous worker parent.

```text
local controller SHA-256       6e5ea4a68d4541043ad95eda12b94e0e9efa0d21e3d4ba2c62a80565ddc626d0
remote controller SHA-256      767e2cfd907527f92bcea1db54b69a9763f29b04a8f76701940abc2336cd2714
bootstrap auditor SHA-256      2779a75160d1b8c2e0a0912582ad598335ff096aa71b781a877fa1fe53968c2a
marker auditor SHA-256         c55d8700879a05806b566e238eccf3e23a9b2b146eb9fd145cff3c3be4282df3
terminal auditor SHA-256       99a6ac3bd7b6e490db4256cbea968e3d7fce052bec305b67af1be3a08a50f1b0
local syntax/registry/fault tests  PASS
authoritative D5 receipt        none
remote D5 namespace             not reverified while host unavailable
UID/scp bootstrap               not run
markers created/consumed        not run
U4/T4 subject executions        0 locally; remote not run
perf/PMU                        0 locally; remote not run
runtime authority changed       false
```

This checkpoint is non-authoritative implementation evidence only. The next
allowed action after the target returns is the controller's read-only remote
absence/self-check. Bootstrap, UID/scp audit, marker creation, and the four
scientific routes remain unexecuted.

#### D5 multiworker/TID execution: terminal provenance block

After the target returned, the frozen controller passed its read-only remote
self-check as `D5_CONTROLLER_VERIFIED_UNRUN`. The bootstrap then proved the
actual UID access path without creating scientific markers. Its independent
audit repeated the live traversal, read, create, write, fsync, rename, reopen,
exact-read, unlink and SCP operations as subject UID 1000. Only that PASS
admitted one atomic creation of the four D5 markers, and a separate marker
audit admitted `U4-FIXED` only.

```text
bootstrap receipt SHA-256      310e6839a3d1b6f8d14a90da3b5c2b11a2368a125e3cee27032e00d722329027
bootstrap audit SHA-256        8d3be4ec3d7f44acf01bc9c10bff04b076eede4ac5869795b3502dee00d42049
marker creation SHA-256        3764d3745d50de066c66449ae8f090308727a67630b7c8374b543e137eec3624
marker audit SHA-256           8bd945422cbc14dc8f91f440e3c1c0fd253208daea1acf825acc8cae141cd32f
markers created / consumed     4 / 0 before U4
subject/perf/PMU before U4     0 / 0 / 0
runtime authority changed      false
```

`U4-FIXED` consumed its marker and executed once without perf or PMU. The exact
fixed 20-worker route completed twenty measured rounds and 7,640 request
records with zero errors, zero unresolved results, exact structure and zero
subject-reported worker migration deltas.

```text
U4 denominator                 502,915,120 edges
U4 traversal thread CPU        22,530,597,494 ns
U4 CPU/edge                    44.80000023463204 ns
U4 receipt SHA-256             229b901d65516d7eb6041668d2974409a721e601a6acb03d52d66929895903e4
U4 verdict                     U4_FIXED_PASS
```

Only U4 PASS admitted `T4-FIXED`. T4 consumed its marker, executed one exact
whole-process inherited `task-clock:u` recording at period 200,000 ns, and all
four mandatory readers returned zero. The event identity and subject structure
were exact, the record contained 146,836 raw samples, and lost records were
zero. The preregistered worker-TID provenance closure nevertheless failed:
worker TID 568878 had traversal samples on both CPU13 and CPU18, while every
worker was required to have a singleton sample-CPU set. The controller made no
causal claim about why the contradictory CPU18 sample exists.

The same record also contained 29 throttle and 29 unthrottle records. This is a
separate `sample_coverage` violation, but the frozen dispatch correctly chose
the higher-priority provenance failure. Attribution did not complete, so no T4
bucket shares, paired perturbation value, fixed/reversed inflation, or
optimization claim is valid.

```text
T4 perf.data SHA-256           02c140c235641ed177139e8fd5b37eb4cdcc5fc952443326d944c2c7322d62b6
T4 perf.data size              7,126,578 B
record / readers / PMU         1 / 4 / 1
lost                           0
throttle / unthrottle          29 / 29
T4 receipt SHA-256             d337dcddcd74e95e8009e520347f6e3cf6c1319c46bf7e896862200af8f5cbbf
T4 verdict                     BLOCKED_PROVENANCE
```

The independent terminal audit verified every local and remote manifest, both
consumed marker identities, two untouched future markers, exact route-state to
receipt links, absence of reversed artifacts and active subjects, predecessor
tree stability, D2 ELF/map identity, and unchanged installed runtime.

```text
terminal verdict               BLOCKED_PROVENANCE
terminal route / cause         T4-FIXED / provenance
terminal receipt SHA-256       b37e24bd87d063063d83dd30f084d7fda81fc9bdd4f1759a1643b2e6809c741a
markers created / consumed     4 / 2
markers available              2, not admitted after terminal verdict
U4-REVERSED / T4-REVERSED      not executed
claim valid                    false
optimization authority         false
retry permitted                false
runtime authority changed      false
```

Receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_TERMINAL_AUDIT_V1_2026-08-26/D5_TERMINAL_AUDIT_RECEIPT.json`

D5 is terminal. It does not establish a valid multiworker bucket attribution,
does not explain the fixed/reversed `+18.77 ns/edge` inflation, and does not
authorize a route retry, SWAR, decoder rewrite, new build, integration,
installation, restart or deployment.

#### D5 T4 offline forensic: diagnostic stream recovered, D5 unchanged

The sealed T4 bytes were read offline without `perf record`, PMU activity,
subject execution or marker mutation. The audit found that the provenance
predicate which terminated D5 was broader than the scientific stream: its one
contradictory CPU18 sample belongs to startup libc code. Every sample matching
the exact D2 Build ID and sealed traversal ranges has the expected worker CPU.

```text
raw samples                       146,836
accepted traversal samples        123,499
foreign startup CPU anomaly       1
exact D2 traversal CPU anomalies  0
lost                              0
throttle / unthrottle             29 / 29
diagnostic T4 vs U4 delta          4.4074355%
```

The throttle records independently violate the frozen D5 sample-coverage
contract, so the recovered attribution remains diagnostic only. D5 stays
terminal `BLOCKED_PROVENANCE`, its old T4 route is not retried, and the two
unconsumed reversed markers remain unadmitted.

```text
forensic verdict                  D5_T4_FORENSIC_DIAGNOSTIC_COMPLETE
forensic receipt SHA-256          d44ade85316f6f6f6f6eeb0917d3cdea168fc083e1a52b6c3b5e88fdf2df80ae20
scientific authority              false
optimization authority            false
runtime authority changed          false
```

Receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_T4_OFFLINE_FORENSIC_AUDIT_V1_2026-08-26/D5_T4_FORENSIC_AUDIT_RECEIPT.json`

#### D6 concurrency accounting: +18.77 ns/edge is saturation cost

D6 joined the sealed D1 component records with the already sealed B5/B6 PMU
comparison. It did not execute a subject, Cargo, perf or PMU and did not mutate
runtime authority. The component denominator is exact: twenty measured rounds,
`502,915,120` examined edges and identical structural work in single, fixed and
reversed routes.

```text
C-SINGLE traversal                25.9650104415 ns/edge
C-FIXED traversal                 44.7350120454 ns/edge
C-REVERSED traversal              44.6967635155 ns/edge
fixed minus single                18.7700016038 ns/edge
reversed minus single             18.7317530740 ns/edge
```

The increase is inside traversal rather than duplicated setup or certificate
work. Instructions/request are effectively unchanged while IPC and effective
frequency fall under the twenty-client load:

```text
                                  B5 single          B6 twenty
instructions/request              42.378604 M        42.388812 M
IPC                                4.111348           2.989735
effective frequency                3.791352 GHz       3.022360 GHz

instruction factor                 1.000241
inverse IPC factor                 1.375154
inverse frequency factor           1.254434
predicted fixed traversal          44.8014864639 ns/edge
observed fixed traversal           44.7350120454 ns/edge
absolute residual                   0.0664744185 ns/edge
residual / observed                 0.148596%
```

A symmetric multiplicative accounting assigns approximately `10.943 ns/edge`
to IPC loss, `7.819 ns/edge` to frequency loss and `0.008 ns/edge` to changed
instruction count. These are accounting contributions, not isolated causal
labels for SMT, package power, cache or scheduling. The paired core-class view
also shows that E cores are only `3.777 ns/edge` slower than P cores under the
same loaded route, so core class alone cannot explain `+18.77`.

Twenty workers deliver `11.608x` aggregate throughput at `58.04%` parallel
efficiency. Therefore the measured delta is a concurrency-saturation penalty,
not extra semantic work. Removing the entire delta while preserving all twenty
simultaneous CPU-bound traversals would require roughly `42%` less work per
edge, a materially stronger machine, or a lower concurrency policy.

The previously sealed E1 experiment already reduced full-executor instructions
by `44.446%` with exact parity, yet still failed the loaded twenty-worker latency
contract. Repeating the old fused-scalar route or adding unmeasured SWAR is not
an admitted shortcut. The next diagnostic is a separate worker/topology sweep
at `1/6/12/14/20` workers to distinguish package loading, P-core SMT and mixed
P/E saturation before selecting a production policy.

```text
D6 verdict                        D6_CONCURRENCY_ACCOUNTING_COMPLETE
D6 receipt SHA-256                cc1fc1c7e74258cd7fec7eed5a113bbaeb3a4bf8ee3b269825f4cd282f5755dc
microarchitectural cause isolated false
production 20-worker route proven false
optimization authority            false
runtime authority changed          false
```

Receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D6_CONCURRENCY_ACCOUNTING_V1_2026-08-26/D6_CONCURRENCY_ACCOUNTING_RECEIPT.json`

#### D7 worker topology: the latency frontier is six physical P cores

D7 executed one diagnostic build and five one-shot worker placements over the
same `382 x 20` request/round schedule and exact `502,915,120` traversal-edge
denominator. Semantic parity passed before measurement. Every route produced
7,640 complete component records with zero errors, unresolved results,
structural mismatch, worker or parent migration, and thermal-throttle drift.
The independent auditor reparsed every component record and raw perf row,
reconstructed the hybrid PMU aggregates, checked all seven marker/state links,
and verified unchanged local and remote runtime authority.

```text
route   CPUs                                  traversal ns/edge   vs W1       throughput scaling
W1      0                                     25.9236697755       baseline    1.000x
W6      0,2,4,6,8,10                          26.5965681266       +0.672899   4.888x
W12     0-11                                  38.5045883051       +12.580919  7.269x
W14     0,2,4,6,8,10,12-19                    39.3010550230       +13.377385  6.843x
W20     0-19                                  45.4517288762       +19.528059  8.640x
```

The historical `+18.77 ns/edge` is therefore reproduced as `+19.53 ns/edge`
under the new exact route. `W6 -> W12` isolates the first large P-core SMT
step: traversal adds `11.908 ns/edge`, IPC falls from `3.400` to `2.363`
(`-30.5%`), and runtime-weighted effective frequency remains nearly constant
at `3.787 -> 3.776 GHz`. `W6 -> W14` instead activates all physical P and E
cores without P siblings: traversal adds `12.704 ns/edge`, IPC falls to `3.050`,
and effective frequency falls by `0.940 GHz` to `2.847 GHz`. `W14 -> W20` then
adds the six P siblings: traversal adds another `6.151 ns/edge` and IPC falls
from `3.050` to `2.530` even though effective frequency rises from `2.847` to
`2.980 GHz`.

Instructions remain nearly flat across the entire sweep:

```text
W1 instructions/edge            361.206580
W20 instructions/edge           362.743403
delta                            +0.4255%
```

This closes the mechanism-class explanation: the delta is not duplicated
traversal work, but a topology-induced microarchitectural slowdown. Both SMT
interventions directly show large IPC loss without a corresponding frequency
loss. The all-physical P+E route directly shows a separate large fall in the
runtime-weighted effective frequency. D7 did not measure package power or
separate P/E clock residency, so a shared package/power budget is a supported
interpretation of that frequency-mediated step, not an independently isolated
cause. The earlier D6 paired core-class comparison also showed that E-core class
alone is too small to explain the full increase.

`W6` is the largest preregistered point within five percent of single-worker
CPU/edge and removes `18.855 ns/edge` relative to W20. That is not a free
replacement: W6 retains `56.58%` of W20 aggregate throughput. W20 remains the
maximum-throughput point, while W6 is the latency-preserving capacity point.

The measured V13 traversal is still a test-only research module:
`src/nanda_wave/l2_field/mod.rs` includes `v13_typed_peak` only under
`#[cfg(test)]`, and its twenty-client owner is the test gate
`run_twenty_client_gate`. Production typing assist has one named boundary
worker and one collapsing pending-request slot. D7 therefore does not justify
editing or capping the production daemon. A future concurrent consumer may use
six physical P-core workers only after its own arrival-rate, queueing-latency
and throughput contract is measured.

An offline read of the already sealed component records gives the following
per-request outer-wall distribution. This was not a preregistered D7 hard gate
and is diagnostic only:

```text
route   p50            p95            p99            max
W1      1.800 ms       2.386 ms       2.793 ms       4.669 ms
W6      1.826 ms       2.762 ms       3.196 ms       8.239 ms
W12     2.690 ms       3.612 ms       4.312 ms       13.147 ms
W14     2.662 ms       4.236 ms       5.099 ms       14.514 ms
W20     3.142 ms       4.448 ms       5.564 ms       19.474 ms
```

W6 and W12 therefore both improve the old five-millisecond service-time p99,
but these samples begin when a worker starts a request. They do not include
waiting time behind a bounded shared queue. W12 retains `84.13%` of W20
aggregate throughput while removing only part of the CPU/edge inflation; W6
removes the full measured inflation while retaining `56.58%`. Selecting between
them is an arrival-rate and queueing decision, not another traversal rewrite.

The `+18.77 ns/edge` cause-search branch is closed. No D8 is admitted solely to
refine this topology diagnosis, and no traversal rewrite, daemon affinity rule
or production worker-pool policy follows from D7. Any continued V10 latency
research returns to the real single-request target: W1 traversal at approximately
`25.9 ns/edge`.

```text
D7 verdict                         D7_WORKER_TOPOLOGY_SWEEP_COMPLETE
producer decision SHA-256          90790c275adbaa08b8c95c32ddd8b9217b0bdf760f52c846fbcf8b1f6477a35a
terminal audit SHA-256             db8f8fbb2ab0bbf6ba45ca9b4d2ce7c394c3de826d82961ce938adea79024f3e
local SHA256SUMS SHA-256           18702c9ec0f5f2030bcaa8833f987b42579d357100f6936399267c2146e986d8
markers created / consumed         7 / 7
Cargo / subjects / perf stat       1 / 6 / 5
perf record                        0
production policy admitted         false
runtime authority changed          false
```

Receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D7_WORKER_TOPOLOGY_SWEEP_V1_2026-08-26/D7_TERMINAL_AUDIT.json`

#### W1 machine-cost decomposition: the separate minimum reduction is first

No new sampling run was required to order the remaining W1 mechanisms. The
invalid historical D2 T-SINGLE stream remains invalid, but D4 is a separate
single-worker experiment with the terminal verdict `D4_SINGLE_ESTIMATOR_PASS`.
Its fixed-period `task-clock:u` route had zero lost, throttle and unthrottle
records, zero unattributed samples, a `3.3438%` paired U3/T3 perturbation delta,
unique PIE normalization and zero machine-byte mismatches.

The offline auditor independently reparsed all 79,048 sealed D4 `perf script`
rows and accepted exactly the same 66,543 CPU-0 traversal samples. All 66,543
normalized IPs were exact decoded instruction starts in the audited D2 map.
The complete bucket closure reproduced D4 exactly:

```text
TRANSITION                         49,104   73.7929%
  FUSED_SCALAR_U64_ADVANCE         44,922   67.5082% of traversal
DAFSA_DECODE_MEMORY                11,138   16.7380%
STACK_CONTROL                       5,682    8.5388%
TERMINAL                              610    0.9167%
RANK                                    9    0.0135%
UNATTRIBUTED                            0    0.0000%
```

This IP evidence remains bound to the exact D2 ELF and Build ID. D7 used a
different ELF, so no D2 IP or machine range is presented as exact D7
attribution:

```text
D2 ELF SHA-256                    bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178
D2 Build ID                       eb951f1a7526a9f1cb365040c10989aa5d3fc50f
D7 ELF SHA-256                    26316b349d8192c697facf1ed5929fcc7133fc8bde15bdde6eae53a438e0f138
D7 Build ID                       4d6280e7324975076be3edc4f40802c26910180a
```

The transfer is limited to mechanism ordering. D4 U3 and D7 W1 differ by
`0.5825%` CPU/edge. D2 V-FIXED and V-REVERSED differ from D7 W1 by `0.6722%`
and `0.6551%` instructions/edge. This is sufficient to select one mechanism
paper, not to assign D7 cycles to D2 addresses.

Within `FUSED_SCALAR_U64_ADVANCE`, the three sealed machine ranges own 6,492,
27,970 and 10,460 samples. The second range contains a distinct machine block
for the source-level `cells[..len].iter().copied().min()` pass after all seven
frontier cells have already been constructed:

```text
minimum setup                     0x778d17 .. 0x778d60     1,124 samples
vector reduction                  0x778d60 .. 0x778d9d    16,739 samples
scalar tail                       0x778d9d .. 0x778db7     2,976 samples
combined                                                   20,839 samples
share of traversal                                         31.3166%
share of fused-transition                                  46.3893%
```

The largest exact IP is `0x778d74`, decoded as `pminub`, with 11,903 samples
(`17.8877%`). This does not mean that one `pminub` costs 17.8877% of traversal.
`task-clock` is non-precise, and concentration includes dynamic visits,
dependency waiting, skid and neighboring work. Likewise, the seven-cell
radius-3 recurrence is required work. Only the separate post-recurrence minimum
scan is selected as a removable-work hypothesis, and that hypothesis remains
untested.

```text
verdict                            W1_MACHINE_COST_DECOMPOSITION_COMPLETE
auditor SHA-256                    d8a45ae8e2db5b86eec82c365fc7923f021a92729516e530d6a847a6fb8a1293
receipt SHA-256                    12e806f11d921047b1437568af6aa77defaa66aba17f86560e87f0167d8d9194
SHA256SUMS SHA-256                 7038f33c4c3a5f042607ac1cd1d5997648e3b8d158b7cf04acf4cab862da7c1b
new perf / PMU / subject           0 / 0 / 0
Cargo / rustc                      0 / 0
remote commands / marker changes   0 / 0
optimization authority             false
runtime authority changed          false
next action                         FUSED_MINIMUM_MECHANISM_PAPER_ONLY
```

Receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_MACHINE_COST_DECOMPOSITION_V1_2026-08-26/W1_MACHINE_COST_DECOMPOSITION_RECEIPT.json`

#### M2 fused-minimum mechanism: controller verified, experiment unrun

M2 tests one narrow removable-work hypothesis selected by the W1 decomposition:
the separate minimum reduction after the seven frontier cells have already been
constructed. It does not change the exact candidate language, radius, frontier,
query schedule, package, or production source. Three test-only machine owners
are frozen:

```text
B  ITERATOR_BASELINE
G  M1_GUARDED_CHAIN
I  INTERLEAVED_RUNNING_MIN
```

The physical order is symmetric and immutable:

```text
BUILD
PARITY
B0-ITERATOR
G0-M1-GUARDED
I0-INTERLEAVED
I1-INTERLEAVED
G1-M1-GUARDED
B1-ITERATOR
```

The build-symbolization correction separates two obligations. The Cargo build
keeps exact frozen optimization inputs and adds only release debuginfo and
unstripped symbols. A later independent read-only audit, rather than the build
producer, must prove distinct B/G/I machine owners and unambiguous executable
ranges before parity or any physical route is admitted.

The first implementation-preflight repair remained immutable
`BLOCKED_BEFORE_CODE`: it incorrectly treated partial controller sources as
accepted implementation inputs. Effective V3 models those files as
`M2_SOURCES_CREATED_UNVERIFIED` and returned `READY_TO_IMPLEMENT`. The first
observed-source code-route gate also remains immutable `VETO`; its authority
graph incorrectly routed marker authority through the local orchestrator and
one parity evidence line was stale. V2 repaired only those structural facts and
passed with eight unique routes, 25/25 source-evidence checks and no warnings:

```text
execution admission --authorizes--> marker ledger
build producer       --observes----> build auditor
parity producer      --observes----> terminal auditor
physical producer    --observes----> terminal auditor
terminal auditor     --proves------> terminal decision
```

The local implementation pass then verified and sealed five Python sources and
one Rust test fragment. It reconstructed the exact `247,328 B` assembled source,
checked SHA-256 `8654217a1509...`, retained the exact production prefix
`ce9ea2d29060...`, parsed the assembled source with `rustfmt`, compiled every
Python source, checked the closed command graph, and exercised four synthetic
terminal-state projections. The local and remote controllers expose exactly
the frozen build, parity and six physical routes.

```text
verdict                            M2_CONTROLLER_VERIFIED_UNRUN
implementation receipt SHA-256    ecc3d5a4f99b060c7bae647674380ca313c6f30c3301e64881a9643f9f696133
local controller SHA-256          0454bdd5dc4051987cc94a293496dd4a39836cb5b981d3fd219d29bcc217e5e3
remote controller SHA-256         14c7c60814927a8f3bcfc607a391d78552fdf8cdd51253eccd429d1f40c50dd6
Rust fragment SHA-256             b0a775420edf9e9d6e7f0b59f9ad840e4822cd0fdd0adc2429ea22a3e9e3a175
Python source modes               0555
Rust fragment / receipt modes     0444 / 0444
```

The bootstrap and build auditors independently accept the sealed implementation
identity in read-only self-checks. The terminal auditor's direct self-check is a
post-bootstrap operation by contract and therefore correctly rejects the
current state because no bootstrap receipt exists. Its Python source and
terminal projection logic were checked pre-execution by the main controller;
this is not a terminal scientific audit and not an M2 failure verdict.

Nothing irreversible or remote occurred in this implementation pass:

```text
execution admission               absent
network / remote reads / writes   0 / 0 / 0
markers created / consumed        0 / 0
Cargo / rustc                      0 / 0
perf stat / perf record            0 / 0
subject executions                0
M2 ELF / parity / physical routes absent / unrun / unrun
runtime authority changed         false
optimization authority            false
```

The only admitted successor is a separate independent remote-execution
preflight after the mini-PC is available. That future gate must recheck the
sealed controller and receipt identities, live host/toolchain/input closure,
namespace absence and one-shot marker admission. It may then admit the remote
bootstrap transaction; this local receipt itself does not.

Key immutable artifacts:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_IMPLEMENTATION_SELF_CHECK_V2_2026-08-26.json`

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_CODE_ROUTE_V2_RECEIPT_2026-08-26.json`

#### M2 failure-publication correction: V3 controller verified, experiment still unrun

The V2 implementation receipt remains immutable and true in its original
offline scope, but a later source review found one execution-safety defect in
its future path. `execute_once()` had cleanup in `finally` but no durable
exception-to-receipt boundary. A lost SSH, SCP, producer or auditor response
after possible marker consumption could therefore strand the transaction
without an immutable local terminal observation. No M2 admission, namespace,
marker or execution existed, so this was repaired before scientific evidence
was created.

The failure-publication correction V3 preserves the exact remote controller,
Rust fragment, candidates, build command, parity route, physical order, PMU
events, denominators, thresholds and terminal decision logic. Only the local
controller and three local auditors moved to new V3 paths. Preflight V4 remains
immutable `BLOCKED_BEFORE_CODE` because two test bindings and one forbidden
side-effect literal were incomplete. Effective V5 returned
`READY_TO_IMPLEMENT` with 24/24 baseline checks and no blockers:

```text
correction paper SHA-256          09a29b68ab3e7ee9ea68e7d2a892080efc5fbd159d930c3881ea37241f3fb88c
correction route SHA-256          76d6e7ed6bce0eb7565f0c97a7126b77b789a3f96d24b5dd7079424544f929b8
structural review SHA-256         2e15f3e3c5a8c5a9ffda38cd616f06cdadea66958be11693412ac923ee5f1553
effective preflight V5 SHA-256    60a676d2d59d296e5e8445b61311ff46edd9a62d3aa950da78a1ea0c9c3b2598
preflight V5 receipt SHA-256      a33fadd7fdfe135e1f726f91b9a2cae85bc7b27343a3ab21ed55f91a9c2561c3
```

V3 creates and fsyncs a local journal before the first external action. Each
of the 17 declared action IDs receives an immutable intent before its callable
and a separate completion only after a structured response passes validation.
An uncompleted intent permanently forbids retry. A live controller exception
publishes one atomic `BLOCKED_PROVENANCE` receipt; state affected by a lost
response is represented as `UNKNOWN`, never as a synthetic zero.

Local disposable fault tests injected an exception at every declared action:

```text
remote cache / bootstrap upload / bootstrap
bootstrap audit / audit upload / marker creation
build / build audit / audit upload / parity
six physical producers
terminal audit

actions checked                    17 / 17
terminal observations per fault   1
later actions after fault          0
retry permitted                    false
lost BUILD cargo/marker facts      UNKNOWN
lost physical marker/perf/subject  UNKNOWN
```

The implementation-receipt publisher was also fault-injected before atomic
rename: no partial final receipt and no stage file survived. The final
observed-source code-route V5 then passed all ten separated execution,
authority, observation and proof routes with `29/29` source-evidence checks,
one path per route, no issue and no warning. Earlier V3/V4 route receipts remain
immutable intermediate history; V5 pins the final V3 source line identities.

```text
verdict                            M2_CONTROLLER_V3_VERIFIED_UNRUN
implementation receipt SHA-256    55b938ef7851bcf560c1e165e0ebe3c1c5906df6f8c9c5a76559488f0ab35f0a
local controller SHA-256          4f2f6484d7ada483688b59a2403354548d4aaffdc594074c4447326c7b8c1f7f
bootstrap auditor SHA-256         30da6d116df0fa5d5372dc8718c86bb3e46c4cb59edcc918429cceb523959869
build auditor SHA-256             091e97c8aa0e1d8a7455fefba0fb038ea04566996b149d4a10feaad0e1d5030e
terminal auditor SHA-256          1656e3a2fa8b772a5ef8c01daab7daf03b751c73fc2ac4bbda02a1ed6520b92b
remote controller SHA-256         14c7c60814927a8f3bcfc607a391d78552fdf8cdd51253eccd429d1f40c50dd6
Rust fragment SHA-256             b0a775420edf9e9d6e7f0b59f9ad840e4822cd0fdd0adc2429ea22a3e9e3a175
code-route V5 SHA-256             0fdfa5ece5b1e53a4d4acd661c4403978f0524b086c1fd5d52ef50c1521501d3
code-route V5 receipt SHA-256     54b66f7a30d6f7b6edc67c2eb07748259e78fa45f4d26c9da241325e2a455acb
Python source modes               0555
receipt / preflight modes         0444 / 0444
```

The V3 bootstrap and build auditors independently accept the sealed controller
and implementation receipt in read-only self-checks. The terminal auditor
remains predecessor-gated until a bootstrap audit exists; its source and
terminal state model were checked offline by the controller.

The correction pass changed no runtime authority and performed no external or
scientific action:

```text
execution admission               absent
execution journal / failure       absent / absent
bootstrap / build / terminal      absent / absent / absent
network / remote reads / writes   0 / 0 / 0
markers created / consumed        0 / 0
Cargo / rustc                      0 / 0
perf stat / perf record            0 / 0
subject executions                0
runtime authority changed         false
optimization authority            false
```

The only admitted successor remains a separate independent M2 V3 execution
preflight after the mini-PC is available. It must pin receipt
`55b938ef7851...`, controller `4f2f6484d7ad...`, all three V3 auditors, exact
remote controller and fragment, and re-establish live namespace/toolchain/input
absence before creating execution admission V2. This local closure does not
admit SSH, namespace creation, markers, Cargo, parity, perf or subjects.

Effective immutable artifacts:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_IMPLEMENTATION_SELF_CHECK_V3_2026-08-26.json`

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_CODE_ROUTE_V5_RECEIPT_2026-08-26.json`

#### M2 terminal build failure and M2R1 fused-minimum result

The admitted M2 transaction consumed its build marker before the single Cargo
invocation, then terminated as `BLOCKED_BUILD`. The Rust evidence publisher
attempted to serialize `Vec<V13TypedPeak>`, but `V13TypedPeak` does not implement
`serde::Serialize`; Cargo exited `101`. This was a build-evidence defect before
parity or any scientific route, not a result about the fused-minimum mechanism:

```text
M2 verdict                         BLOCKED_BUILD
Cargo invocations                  1
markers created / consumed         8 / 1
parity                             not run
perf stat / perf record            0 / 0
scientific subjects                0
retry permitted                    false
runtime authority changed          false
terminal receipt SHA-256           21c2098f7c457c4939d6aef2b36e13ed895f7e8f77e040d8b99950f5db2cf85c
```

M2 remains immutable and was not retried. M2R1 used a fresh task and marker
namespace. Its only source repair projected the already-computed peak fields
into the JSON evidence shape; the production prefix, exact candidate language,
frontier, three B/G/I machine candidates, build profile, route order and
scientific decision gates remained unchanged. A disposable local compile proof
preceded the new one-shot admission.

The independent live gate admitted exactly one M2R1 controller invocation. Its
journal closed all 17 durable intent/completion pairs. The transaction created
and consumed all eight markers, invoked Cargo once, passed semantic parity, and
ran the frozen symmetric physical order:

```text
B0-ITERATOR
G0-M1-GUARDED
I0-INTERLEAVED
I1-INTERLEAVED
G1-M1-GUARDED
B1-ITERATOR
```

All six routes measured the same `502,915,120` edges. No route had a capability,
measurement, provenance, perturbation or thermal failure. Baseline repetition
was stable: traversal spread was `0.08386%`, cycle spread `0.09139%`, and
instruction spread `0.000764%`. Pair means were:

```text
candidate                     ns/edge    cycles/edge    instructions/edge    IPC
B ITERATOR_BASELINE          26.023155     103.816440          361.196498    3.4792
G M1_GUARDED_CHAIN           27.343825     108.774647          354.754905    3.2614
I INTERLEAVED_RUNNING_MIN    27.362606     108.866258          325.256956    2.9877
```

Both lowering candidates removed instructions but made the measured traversal
slower at unchanged effective frequency:

```text
G instruction delta          -1.7834%
G traversal / cycle gain     -5.0750% / -4.7759%

I instruction delta          -9.9501%
I traversal / cycle gain     -5.1471% / -4.8642%

effective frequency          3.79135 GHz for B, G and I
```

The result rejects the selected mechanism, not exact product traversal as a
whole. Eliminating the separate minimum scan worsens dependency/branch behavior
enough to dominate the retired-instruction reduction. Neither candidate may be
promoted, the M2/M2R1 routes may not be repeated, and the old D2 sampled bucket
shares are not restored as proof for another ELF.

```text
verdict                            W1_FUSED_MINIMUM_MECHANISM_REJECTED
execution admission SHA-256       d68ece5becebe4ecc14c0282dfaa52f61039f4241c48a64bb295afb3e95b34df
bootstrap audit SHA-256           a225a326060e43ed4c01253f61585a61a2faa84e8f66ed55b24a169d2d793eae
build audit SHA-256               4edc878e94c7a477ac96e155a96d63fda00c4dae3d5ed9ead0edf5c9efad5a22
terminal receipt SHA-256          98660957aeb31eb17b332868212cbb3ca295f35b979b511ed093fe807e0ea469
journal intent / completion       17 / 17
Cargo / perf stat / perf record   1 / 6 / 0
subject executions                7
markers created / consumed        8 / 8
retry permitted                   false
production edit admitted          false
runtime authority changed         false
next action                        new DAFSA-decode paper only
```

Terminal receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_TERMINAL_AUDIT_V1_2026-08-27/M2R1_TERMINAL_AUDIT_RECEIPT.json`

#### M3 W1 DAFSA typed-view result and source boundary

M3 tested the next mechanism selected after M2R1: whether decoding every state
and edge from validated byte slices on every traversal visit is removable W1
work. It did not change the DAFSA language, product frontier, DP recurrence,
minimum scan, traversal order, rank arithmetic, candidate set or certificate
construction. One fresh test ELF compared the byte view against a safe
prevalidated typed view in the frozen one-worker CPU0 envelope.

The one-shot transaction passed full forward and reverse semantic parity:

```text
states checked                       81,128
edges checked                       226,341
state field mismatches                    0
query/result mismatches                   0
measured edges per physical route   502,915,120
```

The symmetric `B0/T0/T1/B1` physical order produced:

```text
                              byte view          typed view
traversal thread CPU/edge     26.032472651 ns    22.961831598 ns
cycles/edge                  103.702024253       92.152445235
instructions/edge            361.199825441      307.739808260
effective frequency            3.791382262 GHz    3.791263675 GHz

CPU gain                      11.795426%
cycle gain                    11.137274%
instruction delta            -14.800676%
frequency delta                0.003128%
```

Baseline validity against D7 W1 passed. Both baseline and candidate pair spreads
were below `2%`; no provenance, capability, measurement, perturbation or thermal
failure was present. The result isolates `3.070641053 ns/edge` of removable
byte-view decoding and error-path cost in this exact W1 test envelope.

The typed representation itself is not free:

```text
typed states / edges                 81,128 / 226,341
typed payload                         3,689,628 B
construction wall                    1.627..1.678 ms
construction thread CPU              1.629..1.679 ms
```

Construction occurred once before the measured region. M3 therefore did not
test request-local materialization, amortized package-load cost, steady-state
RSS, package or delta reloads, production cache identity, end-to-end request
p99, queue waiting, quality gates, candidate authority or daemon behavior.

```text
verdict                              W1_DAFSA_TYPED_VIEW_PASS
terminal receipt SHA-256             a84355e42bad335d45b379c7e76d2b353bed6c23c30593e1c721be0c0058f324
execution admission SHA-256          bd77fc3d568a20cd21d108db20eb57995b32cfa320056ab37aebaca6bfec9119
Cargo / perf stat / perf record      1 / 4 / 0
subject executions                   5
markers created / consumed           6 / 6
retry permitted                      false
production edit admitted             false
runtime authority changed            false
```

The admitted source/lifetime decision selects the prevalidated typed view as the
sole future test-only V13 implementation candidate. It rejects per-request
materialization, native reinterpretation of the byte sidecar, and independently
reloadable byte/typed authorities. A future owner must create exactly one typed
view per validated sidecar generation and invalidate it atomically with that
generation.

This closes the W1 instruction-level mechanism branch without promoting the
diagnostic executor. No further remote microbenchmark follows automatically.
The next performance claim must come from end-to-end single-request measurement
inside the candidate-specific authority route that actually owns exact search,
with RSS, reload identity, quality and false-authority gates closed together.

Terminal receipt:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_TERMINAL_AUDIT_V1_2026-08-27/M3_TERMINAL_AUDIT_RECEIPT.json`

Immutable source/lifetime decision (`e7b0f66170776677c2b153254aa01a303fdf8538aec273678196dec723715b24`):

`docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_SOURCE_LIFETIME_DECISION_V1_2026-08-27.md`

The first source-decision worksheet is retained as immutable `VETO`: it mixed
measured, future-generation and production owners in shared groups. The repaired
V2 worksheet split those roles and returned coherence-only `PASS` with all seven
route groups matched at `1.0`, zero weak triads, conflicts, evidence gaps or
owner conflicts, and expected `authority_ready=false`:

```text
V1 structural receipt SHA-256        e7dfc4168c1e78a0dcbf542e61cd00b4a203c4b76e660610b9f3184b9923abbf
V1 verdict                           VETO (superseded structural formulation)
V2 structural receipt SHA-256        c3800bb5cef5ff9f35c21b904b9f64eb7a249917055ce94599cafe7f32dac51b
V2 verdict                           PASS
V2 authority_ready                   false
production implementation authority absent
```

#### M3 test-source integration result

The admitted test-only integration exposed and repaired one format-identity
defect before source execution. The sealed M3 measurement sidecar is the
historical `LAYV13D2` encoding, while the current V13 source and its archived
active V11 predecessor parse `LAYV13D3`. The V2 correction therefore preserves
the historical sidecar as evidence and permits exactly one in-memory current
V11 reconstruction before the existing byte-view validator. It does not write,
replace or publish a sidecar file.

```text
historical M3 sidecar       LAYV13D2 / 3,689,884 B / a1aa95be...
current V11 reconstruction LAYV13D3 / 2,460,144 B / 5ebffb81...
reconstruction count       1
sidecar files written      0
```

The implementation remains reachable only through the existing
`#[cfg(test)]` V13 module. It adds a pure safe typed-view module and one ignored
fixed-proof integration test. No production module boundary, daemon, bridge,
cache, package format, Cargo input or installed runtime was changed.

```text
v13_typed_peak.rs
  bytes / SHA-256          152,276 / 1cdc2ad9040f...

typed_exact.rs
  bytes / SHA-256           26,203 / 325bdd386b13...

typed materialization
  count                    1
  states / edges           81,128 / 226,341
  payload                  3,689,628 B
  root                     81,127
```

One exact Cargo route was executed through `scripts/cargo-guard.sh`. It ran the
single ignored release-mode library proof and exited zero. The proof used all
382 fixed cases in forward and reverse schedules and compared the current byte
view, the new typed view and the unchanged generic banded oracle.

```text
Cargo invocations                          1
tests passed / failed                      1 / 0
fixed cases / schedules                  382 / 2

per schedule:
  byte / typed examined edges     25,145,756 / 25,145,756
  byte / typed expanded states     8,059,788 / 8,059,788
  transition checks               25,145,756
  terminal-distance checks         8,059,788

candidate/certificate mismatches           0
generic-oracle mismatches                   0
work mismatches                             0
rank-prefix mismatches                      0
terminal-rank mismatches                    0
transition-check mismatches                 0
```

What this proves is test-source semantic and lifetime parity for one validated
generation. It does not transfer the remote M3 machine gain, establish a
production generation owner, measure RSS/reload behavior or prove end-to-end
latency. The only admitted successor is an actual-owner consequence analysis;
production authority remains absent.

```text
verdict                    M3_TEST_SOURCE_INTEGRATION_AUDITED
integration receipt       1e3e372c858e09e571590be4262e4d923a48e32dede937aac8f18556f10dfe99
integration audit         5a6deb02bc6ec703afe375e25c1cc3d40b5a1d012d90ce63914cad0d17a4ae3d
evidence SHA256SUMS       2895e9403f609bb73b9b76e55bd9b9267cc8b7e36ed9f04764903cc17ed31073
runtime authority changed false
production source changed false
network / perf / PMU      0 / 0 / 0
```

Immutable evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_TEST_SOURCE_INTEGRATION_V1_2026-08-27/INTEGRATION_RECEIPT.json`

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_TEST_SOURCE_INTEGRATION_V1_2026-08-27/INTEGRATION_AUDIT_RECEIPT.json`

#### M3 actual-owner parity result

The test-source mechanism was then carried through the real
`PreparedCanonicalTokenField` material, composite-lattice, candidate-emission
and shared-gate owners. The route retained every exact Phase 7D certificate as
structured evidence; it did not grant V13 a winner or mutation authority.

The first admitted execution stopped before any request because the current
source-bound `LAYV13D3` header differed from the historical M3 sidecar header.
Offline payload comparison proved that the machine payload was unchanged and
that only the 32-byte semantics-source identity at bytes `112..144` differed.
The immutable V5 correction therefore admitted the current validated sidecar
and required its historical-header projection to reproduce the sealed M3 SHA.

```text
current reconstructed sidecar SHA       f2abc8cb8016319a...
historical projection sidecar SHA       5ebffb813ba0ca...
payload SHA from/recomputed header       23b347c2026667... / exact
projected historical clone consumed     false
sidecar files written                    0
```

The identity-repaired proof completed all `764` owner requests but terminated
`BLOCKED_SEMANTIC`: candidate and certificate retention were exact, while
punctuation and final-token materialization disagreed with the fixed proof.
The separate V6 read-only diagnosis reconciled every mismatch:

```text
lattice marker mismatches               72 punctuation_suffix
emitted surface mismatches              72 punctuation_suffix + 2 layout_projection
gate mismatches                          72 punctuation_suffix
normalized/raw-different fixed cases     36
unexplained mismatches                     0
```

V7 repaired the mechanism, not the cases. Exact no-op exclusion now compares
against the normalized observed token. A retained structured
`KeyboardLayout` certificate selects generic full final non-whitespace-token
replacement; other exact certificates keep the existing punctuation-preserving
text-word replacement. The certificate changes replacement scope only and does
not raise candidate authority.

The single V7 owner proof then passed the full forward/reversed denominator:

```text
owner requests completed / expected     764 / 764
candidate mismatches                       0
certificate mismatches                     0
structured certificate mismatches          0
schedule / completeness mismatches          0 / 0
lattice marker mismatches                   0
emitted surface mismatches                  0
gate mismatches                             0
capacity / collision / adapter failures     0 / 0 / 0

typed materializations                      1
per-request typed materializations          0
typed payload bytes                  3,689,628
diagnostic owner prepare p99               111 us
```

```text
verdict                         M3_ACTUAL_OWNER_PARITY_PASS
V7 admission manifest SHA      639e2ba15660bbdccb2b098e3419f7045aaf0fe328a6d0e0283fba1eba5307a1
V7 admission receipt SHA       0236395f870d984b4141b4163fd48966b391783d2d1d3dba7400781cb9f9fd4d
owner receipt SHA              aa2cfc21f7f97d8502a459aeb311722836ee62c4e4724e4ce104fbadd19ad829
terminal receipt SHA           8a703dd385e7787816afa016e034284834540b53ed794d2544f638dcbd421e83
Cargo / perf / network         1 / 0 / 0
runtime authority changed      false
installed Lay changed          false
```

This result proves candidate, certificate, lattice, emission and gate parity in
the test-only actual-owner route. It does **not** prove the end-to-end exact
search p99, total material p99, process PSS/RSS, atomic reload-generation
identity, stale-reader cancellation, allocation rollback, production bridge
coverage or production authority. The only admitted successor is a separate
end-to-end latency/RSS/reload-generation consequence and implementation
preflight; no live package, daemon, IBus process or installed sidecar may change
before that gate passes.

Immutable terminal evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ACTUAL_OWNER_V4_2026-08-27/PARITY_PASS_RECEIPT.json`

#### M3 end-to-end V8 implementation boundary

The admitted successor now has a complete test-only implementation, but has
not executed its target-host scientific action. The V8 paper freezes one mmap
sidecar generation owner, the full 382-case closed-call denominator, two-process
PSS, atomic replacement, stale-reader cancellation and failed-build rollback.
The first implementation preflight remains immutable `BLOCKED_BEFORE_CODE`
because it mislabeled identity checks. Effective V2 corrected only those test
kinds and returned `READY_TO_IMPLEMENT`.

The implementation changes only the existing `#[cfg(test)]` V13 source. It
adds an immutable `Arc` generation slot, one safe typed materialization per
published generation, a direct ignored physical proof and an ignored PSS
helper. Requests borrow the current generation and cannot materialize a view.
A commit holds the generation read lock while validating the exact publication;
an old lease therefore fails closed after replacement. A failed next-generation
builder never reaches publication.

```text
v13_typed_peak.rs before       188,626 B / 385b56d819b0...
v13_typed_peak.rs after        253,080 B / 28f87a76fc19...
protected runtime/Cargo files  exact

compile-only Cargo routes      2 PASS
non-scientific unit routes     1 PASS
measured V8 / PSS helper       0 / 0
remote writes / perf / PMU     0 / 0 / 0
runtime authority changed      false
```

The generation-owner unit proof covers eight concurrent borrows of the same
publication, monotonic `A -> B`, one rejected stale-A commit, one accepted B
commit and an injected generation-C construction failure with B unchanged.
Release `--no-run`, `rustfmt --check`, `git diff --check` and the exact test
registry all pass. The local test ELF is compile evidence only; its host ABI and
bytes have no scientific execution authority.

```text
verdict                         M3_END_TO_END_V8_IMPLEMENTED_UNRUN
V8 paper SHA-256                c5f1655ce4ab91f068f0b50aff1fe5a2a01206d64786b1e23dc4396e10b840a7
V2 canonical manifest SHA       cb282f9c579b145107a1ed38a68ca3ce09d909cc3aeb1847fcfbc3bd7e2e82e1
implementation receipt SHA      cf4b38d81cc7f9ea9125855194635fafde9c76c9d022f7185635e8bb6c2f29e5
implementation SHA256SUMS SHA   c3d4193ec54eb4d0cf9facc8b0cb550dc3b6e241be68787afd2bf5f3bffd4a0f
```

What remains untested is the exact target-host closed-call p99, PSS/RSS,
physical mmap identity, full 1,528 measured requests, and target-host reload
observation. No latency, RSS or production claim follows from implementation
PASS. The only admitted successor is an independent remote execution
preflight; it must create a fresh isolated namespace and preserve installed
packages, daemon/IBus state and runtime authority.

Immutable implementation evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_IMPLEMENTATION_V8_2026-08-27/IMPLEMENTATION_RECEIPT.json`

#### M3 end-to-end V8R3 terminal result

The identity- and terminal-projection-repaired V8R3 route executed its single
target-host subject exactly once and terminated `BLOCKED_LATENCY`. This verdict
is immutable and does not admit a V8R3 retry. Provenance, semantic, capacity,
reload-identity, RSS and environment gates passed; only the frozen latency
conjunct failed.

```text
cases / measured rounds / samples       382 / 4 / 1,528
maximum round search p99                 3,238 us  (gate 3,000 us)
maximum round total-material p99         8,514 us  (gate 5,000 us)

pooled search p50 / p99                  2,127 / 3,126 us
pooled owner-prepare p50 / p99              17 /    60 us
pooled final-materialize p50 / p99          429 / 6,467 us
pooled total-material p50 / p99           2,693 / 8,380 us
final-materialize maximum                         22,838 us
```

All semantic mismatch counters were zero. The route proved one typed
materialization per published generation, zero request-local materializations,
one canceled stale-A commit, one accepted generation-B commit, no mixed
generation observation and rollback without publication. The two-process PSS
delta was `9,716 KiB`, typed ownership was `3,689,628 B` per process and maximum
query scratch was `6,144 B`; all corresponding frozen gates passed.

The measured `final_materialize` span is exactly the call to
`materialize_live_productive_v1_field` after search and owner preparation. It
therefore proves that post-search candidate materialization is the dominant
measured p99 component. V8R3 did not instrument operations inside that call and
does not prove which sub-operation caused the tail. Existing diagnostic traces
make per-candidate `TransitionDecisionCore::admit_candidate_proposal` the
leading hypothesis, not a V8R3 result.

```text
verdict                         BLOCKED_LATENCY
terminal receipt SHA-256       2115a4ce7c8a79fffde043b87c5aa462014873a0365df3a6e82892f395f6aedc
subject receipt SHA-256        65cd8a6f08d77c192ae0eb24fa3df106ee5030e7a8bbdfdf44d08429f7d9bfd5
execution journal SHA256SUMS   1c176cb2b9e986011fba901996260ad34bf59d71d864ae663045800a8bf9cdfc
markers created / consumed     1 / 1
subject executions             1
Cargo / rustc / perf / PMU      0 / 0 / 0 / 0
runtime authority changed      false
production authority admitted false
```

V8R3 did not test daemon or IBus integration, queue-inclusive latency, a
production reload path, or production authority. Its typed-view semantic,
capacity, RSS and generation-identity evidence remains valid within the frozen
test-only owner scope, but the failed latency conjunct prevents promotion.

The only next research boundary is a new diagnostic namespace that decomposes
the already-required final-materialization work without changing behavior or
reusing V8R3 authority. Another W1 traversal experiment, a V8R3 retry, runtime
optimization, install, restart or deployment is not admitted.

Immutable terminal evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R3_TERMINAL_AUDIT_V1_2026-08-27/TERMINAL_AUDIT.json`

#### M3 final-materialization V9 result and mechanism boundary

The admitted V9 diagnostic reused the exact sealed V8R3 ELF and fixed inputs.
It executed one fresh one-shot route with the existing opt-in
`LAY_L2_FIELD_TRACE=1` instrumentation, retained all raw bytes, and parsed
exactly `1,910` ordered materialization rows. The unchanged subject again ended
with its historical `BLOCKED_LATENCY` assertion; all semantic and non-latency
gates remained true.

The first terminal auditor incorrectly interpreted the post-`taskset` argument
`--test-threads=1` as an environment key because it accepted every argv token
containing `=`. That V1 `BLOCKED_PROVENANCE` receipt remains immutable. A
paper-authorized offline V2 auditor bounded environment parsing strictly to the
tokens between the unique `/usr/bin/env` and `/usr/bin/taskset` sentinels,
verified the complete retained evidence and both historical SHA256SUMS trees,
and performed no subject, network, remote, marker, Cargo, rustc, perf or PMU
action. Its authoritative verdict is `FINAL_MATERIALIZATION_DECOMPOSED`.

```text
scientific rows                              1,528
fixed tail rows                                 16

stage                    p50 us    p99 us    max us      sum us
setup                          0         1         2          31
projection                     0         0         3           9
classification                 5       153       182      18,370
candidate gate               425     6,439    22,652   1,178,683
evidence                       0         0         0           0
traced total                 429     6,477    22,826   1,197,093

gate tail aggregate share                  99.2595087301865%
gate largest-stage rows                          16 / 16
```

The expensive cases repeat in every measured round. Case ordinal `375` emits
`53` candidates and spends `22,280..22,652 us` in the aggregate gate span;
ordinal `371` emits `30` and spends `15,434..15,542 us`; ordinal `223` emits
`28` and spends `11,968..12,703 us`; ordinal `366` emits `12` and spends
`6,439..6,475 us`. Candidate count amplifies the tail but does not determine
it: other `48`- and `51`-candidate cases repeatedly cost only about
`2.1..2.3 ms`.

Source inspection binds `gate_us` to one call of
`TransitionDecisionCore::admit_candidate_proposal` per non-observed surface,
plus the following protected-surface and live-authority override. The call
enters a short-circuit chain of candidate explanation, boundary, suffix,
action-operator, context, structural, stable-shape, semantic and final-class
predicates. V9 did not time those predicates separately and cannot select one
of them as the cause.

```text
authoritative V9 verdict              FINAL_MATERIALIZATION_DECOMPOSED
authoritative V9 receipt SHA-256      7105b503ce7a0079...
authoritative V9 SHA256SUMS SHA-256   e6b9c7ef059e4968...
mechanism decision
  CANDIDATE_ADMISSION_AGGREGATE_DOMINANT_SUBMECHANISM_UNKNOWN
mechanism decision SHA-256            3c2e2792d7e6f62a...
effective structural receipt SHA-256  b83eb3096ebc676d...
runtime authority changed             false
production authority admitted         false
```

The result does not admit an admission bypass, predicate removal, cache,
candidate cap, special case or optimization edit. The only successor is a
fresh test-only admission-substage diagnostic with exact candidate,
certificate, decision and reason parity. V8R3 remains immutable
`BLOCKED_LATENCY`; traced V9 timings do not replace its latency values.

Immutable corrected evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_V9_OFFLINE_TERMINAL_AUDIT_V2_2026-08-27/TERMINAL_AUDIT.json`

#### M3 admission-substage V10R4 decomposition

V10R4 completed the admitted test-only decomposition without changing runtime
authority. It retained `1,910` rows: `382` warmup rows and `1,528` scientific
rows. The three leading stages account for `74.12018000771665%` of measured
candidate-admission time, and the leading five account for
`87.13539699379602%`.

```text
known-current lexical work       28.5201%
infinitive-overreach work        23.9916%
protected-surface work           21.6084%

verdict                          ADMISSION_SUBSTAGES_DECOMPOSED
terminal receipt SHA-256         7548ed321c9e2c4beb7f60d8591dc422627c04fb7b86edcfb9a15a7ce4f63ccc
runtime authority changed        false
```

The result localizes repeated lexical fact work across predicates. It does not
prove which operation inside a helper is causal and does not admit a predicate
bypass, candidate cap, fixture-specific branch or production authority change.

#### M3 admission lexical-fact reuse V11 local implementation

V11 selected one bounded mechanism: reuse immutable lexical facts only within
one `candidate_admission` call. The decision and implementation consequence
freeze `UNCACHED` and `REUSE` as test-only comparison modes, preserve predicate
order and preserve candidate, certificate, action and reason semantics.

```text
decision SHA-256                 d2cac2d9cf09cc85a25e5b29cb1a8f8ab11b90a31eeb33190f26808df1118f8f
consequence SHA-256              d7a6b932c50c0f43493ad7d8b673297c0a9f0fc7d899ac9af32a2b508eaf6015

preflight V1                     BLOCKED_BEFORE_CODE
V1 manifest / receipt SHA-256    5c4c0f5c653d... / 9594d11a03ea...
effective preflight V2           READY_TO_IMPLEMENT
V2 manifest / receipt SHA-256    ddf16f2da781... / 5a8d74bdf1fe...
```

The local implementation is confined to
`src/typing_transition/proposal_admission.rs`. A non-test library build passes;
the new fact tests pass `2/2`, proposal-level reuse tests pass `7/7`, and the
focused correction matrix passes `8/8` in both `UNCACHED` and `REUSE` modes.
The source moved from `88,326 B / 6169e6d89a06...` to
`119,643 B / e8a6a1827530...`.

The broader unscoped correction test invocation produced `80 passed / 31
failed / 1 ignored`. No pre-edit broad run exists, so regression attribution is
`UNKNOWN`; this is recorded rather than represented as a V11 semantic proof or
as a V11 terminal blocker. The focused action/reason parity is positive, while
full integration quality remains unproven.

```text
verdict                          V11_MECHANISM_IMPLEMENTED_UNRUN
implementation receipt SHA-256  e3e05ffbf1d25d8d2b0d2b6095e268769737c11867fd71b688db5c1d63b6a9f9
evidence SHA256SUMS SHA-256      fd763337bea717b5f2e4d5e472b267d8a05d0c6e40fc2024144a798cb781618e
remote execution                0
paired B0/B1                     not run
runtime authority changed        false
production authority admitted   false
```

The next action is an independent paired `B0/B1` execution preflight. Future
performance differences are interpreted quantitatively: a small `1-3%`
latency deviation is evidence, not by itself a reason to create another repair
namespace. Semantic mismatch, unsafe authority, lost exact candidates and
unrecoverable provenance remain hard blockers.

Immutable implementation evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_LEXICAL_FACT_REUSE_V11_IMPLEMENTATION_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json`

#### M3 admission lexical-fact reuse V11 paired result

The first paired execution namespace built the exact V11 source once, but its
sealed ELF was published as `0444`. The main test process could start through
the dynamic loader, while its PSS helper used `current_exe()` and direct
execution. B0 therefore ended before scientific measurement. That namespace is
retained as `BLOCKED_PROVENANCE`; B1 was not consumed and no result from it is
used.

R1 reused the exact built ELF bytes in a fresh namespace, changed only the
projection mode to `0555`, and performed no Cargo or rustc action. Its paired
`UNCACHED` and `REUSE` routes both completed with exact candidate, certificate,
action and reason semantics.

```text
metric                               UNCACHED   REUSE    delta
pooled search p99                    3,123 us   3,110 us  -0.42%
pooled final materialize p99         5,947 us   2,546 us -57.19%
pooled total material p99            7,986 us   5,354 us -32.96%
maximum-round total material p99     8,015 us   5,590 us -30.26%
```

The search delta is negligible; the materialization reduction is large and is
the effect the mechanism was designed to produce. The historical `3 ms` search
and `5 ms` total targets remain visible measurements, but their small remaining
miss does not justify another repair namespace. V11 lexical-fact reuse is
selected for integration into the real admission path. Semantic mismatch,
unsafe authority or lost exact candidates remain hard blockers.

```text
R1 verdict                       V11_R1_PAIRED_COMPARISON_COMPLETE
R1 local receipt SHA-256         f751af00202ea26f01141b9a60a9cedb6a4ad9b8b858f6784fb83b3c42510bbe
R1 terminal SHA-256              01d073e30c5751892afac2f06ef97c800ed26d7aab2eeec094427453c3bad11a
selection verdict                V11_LEXICAL_FACT_REUSE_SELECTED_FOR_INTEGRATION
selection receipt SHA-256        7fe7f4b0842f01d3aa3587b85c1cc3734fd7d50b82661a06d379cc563a9adc0e
subject executions               2
Cargo / rustc / perf / PMU       0 / 0 / 0 / 0 in R1
runtime authority changed        false
production authority admitted    false
```

The next code action is direct promotion of the bounded call-local owner,
followed by the existing candidate-specific typed-view integration. No further
admission microbenchmark is admitted.

#### V13 exact typed owner integration and release 1.0.44

The selected call-local lexical-fact reuse and process-lifetime V13 typed owner
were integrated into the real candidate-specific L2 material route. The first
end-to-end query exposed one real capacity defect: exact, grounded and
productive candidates together could exceed the common 74-target material
capacity. The repair preserves every exact, grounded and contour target and
drops only the worst-ranked productive-only tail. If mandatory targets alone
exceed capacity, the route still fails closed.

The final release-source checks passed:

```text
productive_v1 tests                         153 pass / 0 fail / 1 ignored
V13 generation tests                         12 pass / 0 fail / 7 ignored
field cache                                   5 pass / 0 fail
InputGate                                     7 pass / 0 fail
authority contracts                          40 pass / 0 fail
installer regression                         PASS
git diff --check                             PASS
```

The broad `correction_core::tests::` result is explicitly historical debt, not
a new-release proof. The exact installed `1.0.43` source and the final `1.0.44`
source both produce `80 passed / 31 failed / 1 ignored`; the failure-name sets
are identical, so V13 introduced zero new failures and fixed none of those 31.

The release binary compiled one reproducible exact sidecar from the unchanged
canonical package:

```text
canonical package       140,556,462 B / cce259fe0ce5...
V13 exact sidecar         2,460,144 B / a277b3c8b7d4...
typed owner payload       3,689,628 B
lifetime                  process
reload                    process restart required
status                    ready_immutable_owner
```

Both the staged and installed full route were queried with `тжял`. The exact
producer returned `56` candidates and `56` certificates, including `тяжёл`.
The bounded common material retained `74/74` targets with
`grounded_l11_loss=0`; `тяжёл` retained source
`ProductiveL2V90TypedExact` and action `SuggestOnly`. The overflow therefore
does not erase grounded or exact candidates and does not promote exact search
to independent apply authority.

Release `1.0.44` was then installed from the verified ten-binary build. The
Lay-managed daemon and IME were restarted from PID `3095 / 5043` to
`3381101 / 3381140`; the global `ibus-daemon` PID remained `4594`. The active
layout and engine remained `lay-ime-ru`. A real GTK/IBus smoke using the
installed binaries passed `2/2`: explicit conversion produced `проверка`, and
the known form `в коде` was preserved.

```text
verdict                         V13_EXACT_TYPED_OWNER_DEPLOYED_1_0_44
installed Lay                  1.0.44
daemon active                  true
managed IME processes          1
runtime authority changed      true
production policy changed      false
independent exact auto-apply   false
```

The pre-install `1.0.43` binaries and runtime configuration are retained at
`/home/ubu/.local/lib/lay/rollback/1.0.43-before-1.0.44-20260827T130554Z`.
This release does not claim to repair the 31 inherited broad-suite failures,
and its two-case live smoke is not an exhaustive desktop quality proof.

Immutable release evidence:

`docs/structural_gates/receipts/LAY_V13_EXACT_TYPED_OWNER_RELEASE_1_0_44_2026-08-27/RELEASE_RECEIPT.json`

#### Release 1.0.44 GNOME version-display correction

The installed daemon, IME and CLI were already running `1.0.44`, but the GNOME
extension remained a separately installed `1.0.43` UI artifact. The verified
local `1.0.43` extension source was synchronized into the release worktree,
versioned as `1.0.44`, checked by all `17/17` tray UI contracts and reloaded
without restarting the daemon, IME or global IBus process.

The source and installed extension files are byte-identical after deployment,
and the live extension D-Bus service returns `1.0.44`. `gnome-extensions info`
continues to expose its shell-session registration cache as `1.0.43`; this does
not describe the reloaded extension code and clears when GNOME creates a new
shell session. No correction policy or runtime authority changed.

Correction receipt:

`docs/structural_gates/receipts/LAY_1_0_44_GNOME_VERSION_DISPLAY_CORRECTION_V1_2026-08-27.json`

#### Release 1.0.45 Kitty terminal IME regression repair

The `1.0.44` IME presentation change incorrectly classified IBus terminal
purpose `10` as ineligible for text assistance. It also delegated committed
Kitty tails to daemon uinput because Kitty does not advertise SurroundingText,
despite the existing IME terminal erase-and-commit backend. The daemon detected
Double Shift and produced a valid replacement decision, but that delegated
output route did not alter the visible Kitty text.

Release `1.0.45` separates terminal from sensitive content and grants
committed-tail IME authority only when the client is explicitly terminal and
the terminal output profile can execute. Generic cursor geometry remains
insufficient. Password, PIN, PRIVATE and HIDDEN_TEXT behavior is unchanged.

```text
full lay-ibus-engine                       245 pass / 0 fail
changed-file / latency / replay gates      PASS
release build                              PASS
installed/source binaries                  10/10 byte-identical
Kitty terminal assistance                  true
Kitty warm suffix                          пров + ерить -> проверить
Kitty Double Shift                         ghbdtn -> привет
output route                               terminal_erase_commit
daemon-uinput fallback                     not used
global ibus-daemon PID                     4594 -> 4594
installed version                          1.0.45
```

The first cold `пров` result took `231625 us` and remained hidden under the
unchanged `50 ms` stale-display rule. The warm result took `12 us`, was
published at display age `81 us`, and was accepted visibly. The isolated Kitty
fixture was removed and the active engine restored to `lay-ime-ru`.

```text
lay-ibus-engine SHA-256  342c79f422e38769424ce9ba111c3fc607ed312725d3fd5d0fb7a955b71b48e6
lay-daemon SHA-256       1160738dc8d310cb1c67883e3e7ffffceb5eade9f10b093832de4a3c8b22f446
rollback                 /home/ubu/.local/lib/lay/rollback/1.0.44-before-1.0.45-20260828T052241+0300
```

Verdict: `LAY_1_0_45_KITTY_IME_REGRESSION_REPAIRED`.

Immutable release evidence:

`docs/structural_gates/receipts/LAY_1_0_45_KITTY_IME_REGRESSION_REPAIR_2026-08-28/RELEASE_RECEIPT.json`

### TD-007 milestone 2: IME authority contract reconciliation

The 28 historical `ime_authority` failures were reconciled against the active
frame-bound authority contract. A mutable layout hint, visible dictionary
projection, contextual recurrence, or frameless lexical candidate does not by
itself authorize an automatic edit. Exact layout auto-apply remains available
when the request carries a closed exact-layout contour certificate.

Three tests that asserted the retired `glued_phrase` producer now assert the
active `CanonicalL2FieldBoundary` owner without changing boundary semantics.
The IBus atomic proof no longer assumes one nondeterministic lexical suffix; it
proves that settlement preserves the exact suffix published by the same atomic
proposal.

One real safety defect was found while replacing the stale assertions. Without
the lexical package, `перхвачу` could fall through to the false boundary
`пер хвачу`. Admission now rejects an unbacked short fragment of a known Russian
derivational prefix while preserving verified boundaries including `да норм`,
`то есть`, and `Елена просит`. The rule is structural and contains no fixture
surface or test identity.

```text
baseline failures before milestone 2    81
milestone 2 ledger rows                  28
remaining exact-known failures           53
correctness selected                    2309
package selected                          35
unexpected failures                        0
infrastructure failures                    0
verdict          PASS_WITH_EXACT_KNOWN_FAILURES
installed runtime authority changed     false
```

Evidence:

- `tech_debt/evidence/td007-milestone2-ledger-v1.json`
- `tech_debt/evidence/td007-milestone2-known-failures-v1.json`
- `tech_debt/evidence/td007-milestone2-verified-v3.json`

This milestone does not promote frameless lexical candidates, change installed
Lay, or claim the final 13 x 20,000 heldout quality gate. That gate remains due
after all TD-007 runtime milestones close.
