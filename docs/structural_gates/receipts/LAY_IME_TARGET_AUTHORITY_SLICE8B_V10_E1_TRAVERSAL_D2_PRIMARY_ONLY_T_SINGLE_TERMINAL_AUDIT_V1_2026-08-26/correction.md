# D2 T-SINGLE estimator-scope correction V1

Date: 2026-08-26

## Immutable history

The one admitted `T-SINGLE` execution and its terminal publication remain
immutable:

```text
historical verdict                 BLOCKED_PROVENANCE
remote receipt SHA-256             afaeb7d3caffb1967dd76021e42b94664803cef5d0ed72ec574fb54526a8fa0d
t-single marker                    consumed-before-exec
retry                              forbidden
T-FIXED / T-REVERSED               not admitted
```

This correction is offline interpretation only. It does not execute perf, the
D2 ELF, a D2 subject, Cargo, rustc, PMU events, parity, U, V, or another T
route, and it does not mutate any marker or remote state.

## Invalid historical reason

The T controller added this predicate:

```text
traversal sample CPUs must be a subset of the frozen subject CPU list
```

V4 does not contain that hard gate. It requires exact subject affinity,
worker inheritance, exact event identity, valid map identity, perturbation and
coverage gates. The subject receipt proves `cpus=[0]` and zero measured thread
migrations. A sample CPU outside that list is therefore not independently a
frozen provenance failure. The historical selected reason
`traversal sample CPU outside frozen mapping` is withdrawn from effective
interpretation, without modifying the historical receipt.

## Actual estimator-scope defect

The frozen V4 estimator says the traversal sample stream is exactly one
382-query warmup round plus twenty measured rounds, with no timestamp filter
or warmup subtraction. The byte-identical component route actually executes
in this order:

```text
d1_load_inputs()
d1_parse_cpus()
d1_pin_current_thread(0)
one warmup round
twenty measured rounds
```

Whole-process command wrapping therefore records `d1_load_inputs()` before
pinning and before the frozen estimator begins. `d1_load_inputs()` calls
`d1_validate_dense_alphabet()`, which calls the shared
`V13DafsaView::edge()` decoder. The sealed bucket map gives the exclusive
`edge()` and `state()` machine ranges traversal mechanism ownership wherever
they execute; it has no call-context dimension.

The sealed sample stream proves the resulting contamination:

```text
pre-pinning CPU                       6
CPU6 D2 samples                    4,395
CPU6 OUTSIDE_TRAVERSAL             4,379
CPU6 samples in traversal ranges      16
CPU6 worker event interval
                    2809072.503217..2809073.079066
first CPU0 worker event          2809073.079286
subject measured migrations                    0
```

Those sixteen samples resolve only to the shared DAFSA decoder ranges:

```text
EDGE_DECODE       6
STATE_DECODE      8
SYMBOL_DECODE     2
```

They cannot be removed under the frozen no-time-filter rule. Consequently the
observed traversal stream is not the preregistered estimator, and the sampled
bucket counts are not claim-bearing attribution evidence. This is an actual
route/evidence closure drift and dispatches to `BLOCKED_PROVENANCE` before the
lower-priority T causes.

## Lower-priority failures retained

The complete historical observation also proves:

```text
sampled-vs-paired-U delta          18.434656047164%
frozen maximum                      5.000000000000%
lost/throttle/unthrottle                0/15847/15847
traversal samples                         106,901
UNATTRIBUTED                                  0%
```

Even the forbidden counterfactual subtraction of all sixteen pre-pinning
traversal-range samples would produce `21.253089388126 ns/edge` and an
`18.446864029346%` sampled-vs-U delta, still above the frozen perturbation
limit. This counterfactual is diagnostic only; it does not repair or replace
the executed route.

## Effective verdict

```text
historical receipt verdict          BLOCKED_PROVENANCE
historical selected reason          invalid non-contractual CPU predicate
effective selected cause            estimator-scope route/evidence drift
effective terminal verdict          BLOCKED_PROVENANCE
scientific attribution              NOT ESTABLISHED
T-FIXED / T-REVERSED                forbidden
D2 retry                            forbidden
optimization authority             none
runtime authority changed          false
```

The observed bucket values remain diagnostic, scientifically invalid numbers.
They do not identify the mechanism responsible for the frozen
`+18.77 ns/edge` concurrency inflation and do not admit SWAR, decoder/layout,
rank, stack, runtime, deployment, or integration work.
