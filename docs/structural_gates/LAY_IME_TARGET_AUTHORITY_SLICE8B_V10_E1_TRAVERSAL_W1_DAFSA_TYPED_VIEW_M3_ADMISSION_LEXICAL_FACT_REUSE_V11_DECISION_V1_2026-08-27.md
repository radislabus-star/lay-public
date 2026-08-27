# DAFSA Typed View M3 Admission Lexical Fact Reuse V11 Decision V1

Date: 2026-08-27

Status: `ADMISSION_LEXICAL_FACT_REUSE_SELECTED_FOR_ONE_EXPERIMENT`

## Scope

This paper consumes the authoritative V10R4 admission-substage decomposition
and selects at most one bounded test-only experiment. It does not reinterpret
the immutable V8R3 `BLOCKED_LATENCY` verdict, change an admission decision,
remove or reorder a predicate, edit an installed package, or grant production
authority.

Immutable predecessor evidence:

```text
V10 diagnostic paper SHA-256
  afb4709efd22a63119527d21d126629a08a811e2b4e73a68c211144df946bd27
V10R4 correction paper SHA-256
  fccddd22499cf639d595333674ed67c6ee63290a9052f719289f61e1e83bdb63
V10R4 correction route receipt SHA-256
  9d77ec2d14a6636d232804b1bfe4fc77e2bd4f8bf176191df6431ca393e80769
V10R4 implementation preflight receipt SHA-256
  da3b5c3681c7e07c11d1d497506b7f0395a750e667d71694f269cf28f7591c78
V10R4 implementation receipt SHA-256
  a219ae63f3b478cd423f0d90081afd15825b60412a5b42c66c452939aabfc477
V10R4 terminal receipt SHA-256
  7548ed321c9e2c4beb7f60d8591dc422627c04fb7b86edcfb9a15a7ce4f63ccc
V10R4 terminal SHA256SUMS SHA-256
  681782abac120aea89455477f3ad0c8bea28ae76e3d455a42e90b96464d24497
V10R4 terminal verdict
  ADMISSION_SUBSTAGES_DECOMPOSED
```

All V8, V9, V10 and V10R1-R4 namespaces, markers, receipts, journals, ELFs and
raw trace bytes remain immutable.

## Measured Facts

V10R4 retained `1,910` complete trace rows: `382` warmup rows and `1,528`
scientific rows. Candidate, certificate, structured-certificate, lattice,
emitted-surface, action and reason parity all passed. The measured denominator
contains `4,984` admission calls.

```text
stage                                      calls  hits     sum ns   admission share
known_current_surface_drift                 4,964   488  326,706,889   28.5201%
structural_infinitive_overreach             4,240     0  274,831,701   23.9916%
structural_protected_context_authority      4,240 1,444  247,531,133   21.6084%
structural_known_word_different_known       2,796     0   93,948,196    8.2013%
structural_phrase_part_growth               4,460     0   55,145,164    4.8139%
```

The first three stages account for `74.12018000771665%` of measured admission
time. The first five account for `87.13539699379602%`. In the preregistered
top-sixteen cohort the same first three stages account for
`68.31785122016285%`.

The four repeated tail cases reach the three leading stages with zero hits:

```text
case  emitted  known-current calls/hits  infinitive calls/hits  protected calls/hits
223       112       112 / 0                   112 / 0                112 / 0
366        48        48 / 0                    48 / 0                 48 / 0
371       120       120 / 0                   120 / 0                120 / 0
375       212       212 / 0                   208 / 0                208 / 0
```

This establishes repeated exhaustive negative paths, not useful
short-circuit work, in every fixed tail case.

## Source Overlap

The measured stages are separate predicates, but source inspection shows that
their dominant candidate is shared lexical fact production for the same
`original` and `replacement` pair:

```text
known_current_surface_drift
  -> last_text_word + lowercase
  -> verified_surface_to_lexical_center_repair
       -> known_russian_autocorrect_token(original/replacement)
  -> protected_current_surface_token(original)

structural_infinitive_overreach
  -> last_text_word + lowercase
  -> protected_current_surface_token(original)
  -> known_russian_autocorrect_token(replacement)

structural_protected_context_authority
  -> last_text_word + lowercase
  -> protected_current_surface_token(original)

structural_known_word_different_known
  -> last_text_word + lowercase
  -> known_russian_autocorrect_token(original/replacement)
```

`protected_current_surface_token` itself includes the same known-token test
before additional lexical-phase, form and dictionary reads. The lexical-phase
memory and word sets used by the frozen subject are process-lifetime immutable
`OnceLock` objects. The exact test route fixes one process policy for the whole
subject.

V10R4 did not time inside those helpers. Therefore it does not prove that a
specific dictionary, form rule, string allocation or lookup implementation is
individually causal. It does support one experiment that removes only repeated
production of identical per-candidate facts while retaining the first
observation and every decision predicate.

## Selected Experiment

The sole selected hypothesis is:

> Reusing lazily derived lexical facts within one `candidate_admission` call
> removes repeated normalization and lexical-authority work from the three
> dominant stages without changing the candidate language or authority result.

The implementation contract is narrow:

```text
owner                     one candidate_admission call
lifetime                  call-local only
construction              lazy, on first existing consumer
keys                      exact original/replacement byte slices
reusable facts            last word, lowercase, Cyrillic/length facts,
                          known-token and protected-current booleans
cross-request cache       forbidden
global/thread-local cache forbidden
case/word special cases   forbidden
predicate removal         forbidden
predicate reordering      forbidden
reason/action rewrite     forbidden
candidate cap/filter      forbidden
SafetyGate/verifier edit  forbidden
```

The experiment is compiled only in the test-owner route and is selected by an
explicit test environment mode. A production build must retain the unmodified
uncached route until a later production-source decision. The same test ELF must
run paired modes:

```text
B0  reuse disabled, existing behavior
B1  reuse enabled, one bounded candidate
```

Both modes use the exact M3 owner, sidecar generation, `382` cases, one warmup,
four measured rounds and fixed forward/reversed schedule. The experiment must
retain exact candidate, certificate, structured-certificate, lattice,
emitted-surface, action and reason parity. It must also prove that B0 and B1
produce identical decisions for every admission call under both frozen hot
authority modes.

## Gates

The candidate is accepted only if all existing semantic, capacity, RSS,
generation, reload and rollback gates pass and the paired B1 result satisfies:

```text
maximum round search p99              <= 3,000 us
maximum round total-material p99      <= 5,000 us
candidate/certificate/gate mismatch   0
action/reason mismatch                0
per-request fact owners               1
cross-request retained entries        0
runtime authority changed             false
installed Lay changed                 false
```

B0 is a paired current-build denominator, not a new baseline authority. The
immutable V8R3 latency failure remains historical evidence. Instrumented V10R4
times cannot replace either B0 or B1 end-to-end timing.

## Decision Tree

```text
V10R4 ADMISSION_SUBSTAGES_DECOMPOSED
        -> source-overlap decision
        -> ADMISSION_LEXICAL_FACT_REUSE_SELECTED_FOR_ONE_EXPERIMENT
        -> structural gate
        -> implementation preflight
        -> one fresh build
        -> B0 once
        -> B1 once
        +-- all gates PASS
        |     -> separate production-source/lifetime consequence
        |     -> actual owner integration
        +-- any gate FAIL
              -> terminal rejection
              -> close admission microoptimization
              -> continue typed-view integration without this edit
```

No second admission microoptimization follows a failed B1. No production,
daemon, IBus, install, restart or deployment action is admitted by this paper.

