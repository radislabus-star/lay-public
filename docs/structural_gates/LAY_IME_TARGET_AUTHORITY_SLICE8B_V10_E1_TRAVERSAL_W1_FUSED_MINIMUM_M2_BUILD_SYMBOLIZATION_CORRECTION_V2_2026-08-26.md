# W1 Fused-Minimum M2 Build Symbolization Correction V2

Date: 2026-08-26

## Defect

M2 V1 simultaneously requires:

```text
exact D7 release profile
symbols and DWARF presence
B/G/I symbol and machine-range identities
```

The sealed Cargo input and D7 build evidence make those statements incompatible
without an explicit exception:

```text
Cargo.toml [profile.release]   strip = true
D7 environment                no CARGO_PROFILE_RELEASE_DEBUG override
D7 environment                no CARGO_PROFILE_RELEASE_STRIP override
```

Therefore a literal copy of the D7 environment can produce neither the required
DWARF ownership closure nor stable B/G/I symbols. Letting the future controller
choose an override would violate preregistration; omitting the audit would
weaken the M2 contract.

## Immutable History

M2 V1, its structural review and implementation preflight V1 remain immutable.
Their scientific candidate set, route order, parity, denominators, physical
gates, decision rule and authority boundary remain effective. This correction
supersedes only the underspecified build-symbolization sentence.

No M2 Cargo, rustc, ELF, marker, perf, PMU or subject action has occurred. The
current six implementation sources are local and unsealed.

## Effective Build Contract

The one diagnostic build uses the exact D7 optimization and package closure,
plus the exact D2-proven symbolization exception:

```text
CARGO_BUILD_JOBS=20
CARGO_INCREMENTAL=0
CARGO_NET_OFFLINE=true
CARGO_PROFILE_RELEASE_DEBUG=2
CARGO_PROFILE_RELEASE_STRIP=none
RUSTFLAGS=""
CARGO_TARGET_DIR=<fresh isolated target>
```

Cargo argv remains:

```text
scripts/cargo-guard.sh
test
--offline
--locked
--release
--lib
--no-run
nanda_wave::l2_field::v13_typed_peak::tests::v10_m2_fused_minimum_physical
```

The following performance-affecting release settings remain exact:

```text
opt-level       3
lto             true
codegen-units   1
panic           abort
incremental     false
target features exact D7
```

Only debug-information retention and stripping policy differ. No source,
optimization, target, linker, panic, LTO, codegen or dependency setting may be
changed under this correction.

## Physical Validity

Symbol retention is not assumed free. The M2 B pair remains the physical
control for the diagnostic ELF:

```text
abs(B mean traversal - 25.923669775527927) / D7 W1 <= 5%
abs(B mean instructions - 361.20658023962375) / D7 W1 <= 1%
B traversal/cycle/instruction pair spreads <= 2%
```

Any `.text` or layout difference is published. It is not automatically called
harmless and does not identify a cause. The B gates decide whether the
symbolized build is physically comparable. Failure is
`BLOCKED_PERTURBATION`, with no rebuild.

## Build Audit

The read-only audit must now require all of:

```text
ET_DYN and exact Build ID
.text SHA / size / executable PT_LOAD closure
.symtab and .strtab present
.debug_info / .debug_line / .debug_abbrev present
B/G/I machine owners present and non-folded
assembled source and production prefix exact
ELF executed false
```

Missing symbol or DWARF material is `BLOCKED_BUILD`. It never admits a rebuild.

## Effective Verdict

```text
M2_BUILD_SYMBOLIZATION_CONTRACT_REPAIRED
```

This verdict admits only a revised offline implementation preflight. It does
not admit remote access, Cargo, rustc, markers, ELF execution, perf, PMU,
subjects, production edits, installation, restart or deployment.
