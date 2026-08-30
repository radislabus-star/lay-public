# TD-102 Runtime And Research Boundary Decision V1

Date: `2026-08-30`

Base commit: `a301b82c45098279625631f1c02e2c425710db47`

## Decision

Do not admit the proposed `lay-runtime` / `lay-model-format` / `lay-research`
workspace split at the current evidence level. Keep one crate and preserve all
package schemas and public commands. Route the smaller proof/compiler isolation
question to TD-104.

Verdict candidate: `WORKSPACE_SPLIT_NOT_ADMITTED_BENEFIT_UNPROVEN`.

This closes the TD-102 decision. It does not claim that the current module tree
is ideal forever; it says the proposed multi-crate migration is not the minimum
sufficient repair now.

## Measured Facts

On the disposable mini-PC checkout, with a fresh `CARGO_TARGET_DIR` per lane,
three repeated measurements produced:

| Lane | Wall | Max RSS | Reported units | Target bytes |
|---|---:|---:|---:|---:|
| `check --locked --bin lay-daemon` | 20.11 s median | 1,295,044-1,296,140 KiB | 176 | 300,406,378 |
| `check --locked --all-targets` | 24.44 s median | 1,653,580-1,654,204 KiB | 176 | 300,909,602 |

Selecting every target added 3.91, 4.21, and 4.41 seconds in the paired runs,
and 503,224 target bytes, but no additional reported dependency units. The
runtime-only lane still type-checks the mixed `lay` library and reports its
research-owned dead-code surface. Raw outputs and their complete `SHA256SUMS`
are retained under `tech_debt/evidence/td102-remote-raw-v2/`.

The source boundary is broad: one package, 13 binaries, and 193 Rust files
under `src/nanda_wave` containing 164,071 lines. The root re-exports runtime,
compiler, proof, eval, package, and service APIs through one module. Runtime and
proof code also share package and identity types directly.

## Why The Workspace Split Is Rejected

The proposed split would require a new public contract across package formats,
runtime views, service identities, proof inputs, and compiler outputs before a
net build benefit has been demonstrated. A move-only extraction over 164k lines
would create a large review and feature-matrix surface while package byte parity
and authority ownership remain conjunctive release gates.

The repeated all-target overhead is measured, but it is addressable by targeted
command lanes. It is not evidence that three new crates would reduce the
runtime build enough to repay their migration cost. The reverse claim is also
not made: a workspace split was not prototyped, so its benefit or harm remains
unproven.

## Estimated Spectral Budget

Potential benefit: `+9`.

- `+4` clearer runtime/research navigation.
- `+3` smaller conceptual default build surface.
- `+2` explicit package-format ownership.

Migration cost: `-17`.

- `-5` package/schema and authority parity risk.
- `-4` 164k-line move and review surface.
- `-3` new public cross-crate API.
- `-3` feature/target matrix expansion.
- `-2` cycle or duplicated-type pressure.

Estimated net score: `-8`. This estimate records migration risk; it is not a
measurement and does not support the verdict by itself. No implementation is
admitted without a measured prototype.

## Unmeasured Scope

- incremental build time after a split;
- release binary size after a split;
- proof/test-lane time after a split;
- cross-crate API size and actual dependency cycles.

These are required only if a future proposal asks to reopen the workspace
split. They are not silently treated as zero.

## Selected Next Boundary

TD-104 must inventory every unconditional cold compiler, proof, and eval root
across lexical grokking, L2, L3/context, L4, and top-level eval surfaces. It may
evaluate one narrow, reversible same-crate feature or module isolation using
existing package and target ownership. It must preserve one crate unless a
separate measured prototype proves that a crate boundary pays for itself.

No source, package, runtime, command, installation, or service state changed in
TD-102.
