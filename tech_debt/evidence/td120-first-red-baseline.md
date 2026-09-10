# TD-120 first remote RED baseline

Date2026-09-05; baseline production cc1e2207 (1.0.65). Only nine new tests and
their cfg(test) registration differed. Production authority unchanged.

Remote `e@192.168.3.94`, source
`/home/e/projects/lay-td120-121-SUdh2I`; archive baseline local remote-only
Git commit `734f52ce881181582d82623e0b267745ae28e191` represents cc1e2207,
not a new upstream release commit. External cache
`/home/e/projects/lay-td119-gate-v1/target`, bytes3551731712 of12884901888.

Command from that remote source directory:

```sh
env CARGO_TARGET_DIR=/home/e/projects/lay-td119-gate-v1/target \
    LAY_RESOURCE_PROFILE=dedicated-20cpu \
    scripts/lay-resource-guard.sh -- scripts/cargo-guard.sh \
    test --bin lay-ibus-engine td120_ -- --nocapture
```

Test module SHA256 `2cec08d82a40d7a4f551998911e4f40b49f1a9c8e6f971d786734d973356f787`;
root binary source SHA256 `c8e70f4aaa51cc926696a38d3941bd634fb0ea5e522eb72343a7c4eb758d82e4`.
Initial archive transfer preserved old mtimes and Cargo reused its old test
binary:0 selected/282 filtered. That attempt is invalid as test evidence.
After touching the two transferred test sources, Cargo rebuilt2.39s and ran
the actual9 tests in0.56s, exit101:3 PASS,6 FAIL,0 ignored,282 filtered.

Five failures reached the intended stale-suppression assertion: full erasure
then new text, left context, identical retype, undo with boundary, duplicate
output. Controls passed: partial erase/layout punctuation, rejected edit and
exact replay temporary empty/revoke.

The sixth failure (candidate acceptance) reached an incorrect test assertion:
the helper demanded a DeleteSurroundingText even for an optimized insertion.
It is a fixture defect, not a proven additional runtime failure. Sol was asked
to assert the legitimate minimal effect, use real ordinary focus binding and
correct the duplicate fixture's already-closed first result expectation before
production changes. These changes do not remove intended safety obligations.

Raw remote log `/home/e/projects/lay-td120-121-SUdh2I-td120-red-v2.log`, SHA256
`06b88e7b6f9f70f35815777046ab0d8917877de0d9c8073c07d44724a207732e`.
This is a first RED baseline, not the expanded O/E/V1/A/C acceptance suite or
post-fix conversion proof. No local Cargo or live probe was run by this step.
