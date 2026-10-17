# TD-136 implementation review — pass 1

Verdict: **ACCEPT, 8/10**, for the bounded source callback characterization. No blocking implementation or evidence-honesty findings. This is implementation review pass 1; the earlier effect-map review is a separate planning review. Whole affected verification is **PENDING** at this report's cutoff, 2026-10-09 19:59:31 UTC. Acceptance of this code does not close pending gates or establish native acceptance.

## Reviewed identity and change

Checkout `/home/ubu/projects/lay-space-boundary-shift-20261007`, branch `codex/space-boundary-shift-20261007`, HEAD/baseline `60e0d095f4468d744762831d56336fe428583fb4`. Read AGENTS.md, ARCHITECTURE.md and DEVELOPMENT.md; ran the existing Graphify query first for navigation without refreshing it. Reviewed the exact baseline diff, all five untracked TD-136 artifacts, relevant production callbacks/admission/output and existing adjacent tests. The map's findings were used as a bounded planning input, not as an independent implementation verdict.

`terminal_delivery.rs` retains its entire baseline byte-for-byte and appends 243 lines: one test plus two local effect-capture helpers. Production Rust logic, Cargo.toml, Cargo.lock, rust-toolchain.toml and build.rs are unchanged. Generated graph data and `src/generated/architecture_graph_receipt.json` changed separately. The latter is embedded through `src/architecture_contract.rs:24`; this review therefore claims unchanged production logic, not identical future executable bytes. No installed runtime identity or physical behavior was verified here.

| Reviewed artifact | SHA-256 |
| --- | --- |
| `src/bin/lay_ibus_engine/context_admission/adapter/tests/terminal_delivery.rs` | `29e23c2c6e913c7fa251a16b78d200860a3bc9d9902fa2a1a8ea23ebb718a48f` |
| `scripts/test-lanes/manifest.json` | `d25685cfce7655d0a89c59209689cf8ba83a287512c44c46d65d703361801788` |
| `tech_debt/evidence/2026-10-09-td136-source-characterization.json` | `885157c8126927a253b6619451bcf6af80f370b04215fc87d0b92eb37557cba8` |
| `docs/architecture/decisions/2026-10-09-cap41-focus-callback-characterization.json` | `88800561cea3e664f4560c2c91bdbf5b194214b7e7d52591888cc8bef1b90936` |
| `docs/architecture/tech-debt-maintenance-2026-10-09.md` | `f7ced71c7e19d2ce6f80276d188c7f4fc9f4823975c3775eab4f795735baa39b` |
| `src/generated/architecture_graph_receipt.json` | `a5a9cd13a6ecd403066749d88301b4a9bcac8b60b5314670e5a2f0d36888dac2` |

## Prioritized findings

No P0/P1/P2 defect requiring a code change was found in the claimed bounded result. No cosmetic refactor or duplicate matrix is recommended. The following limits are already represented in the source packet and decision and must remain visible when finalizing the result.

1. **The joined source route is real, with a limited dispatch boundary.** At `terminal_delivery.rs:4209`, factory profile and bootstrap agree on RU. Capabilities 41 are set before both fields' first printables (:4231, :4279); restoring the controlled config after FocusIn (:4278) does not repair layout state manually. The native Tab disposition is unhandled press/release with no text edit (:4249), followed by production FocusOut and FocusInId helpers. Owner and activation generations change, the new context path is exact, the previous token fails revalidation, old prefix is excluded, and owned preedit stays absent (:4309). `word_scope.rs:454` sends a real P2P key message and consumes its admission stamp before invoking production ProcessKeyEvent; `residuals.rs:1079` invokes production FocusOut with the real observer. FocusInId uses a received header with direct callback arguments (:4271), not typed wire-body dispatch through `focus_in_id_bus`. `residuals.rs:3234` invokes the actual production SetSurroundingText callback with an IBus text value; it does not dispatch that callback over D-Bus. These boundaries are suitable for source characterization and do not prove native Tab focus motion or zbus wrapper/property publication.

2. **The negatives prove frame denial and receipt retirement.** Absence before the first new-field receipt is asserted at :4321. A real Backspace/retype sequence changes tail epoch, preserves surrounding observation revision, returns to equal text and still denies a frame (:4330–4354). No stale Space callback is executed in those states. `preedit.rs:1432` clears the snapshot on managed append, which independently explains why removing the freshness floor check at `window_interaction/observation.rs:735` survives. This test cannot isolate that guard's sensitivity. The packet's :243–245 and owning document :329–334 state this correctly. Existing stronger exact-refresh/selection/fragment/focus tests remain unchanged; their domains must not be broadened into ordinary caps41 stale-Space coverage.

3. **The positive effect assertion covers scalar geometry and one text owner.** Every managed glyph has exactly one CommitText, no DeleteSurroundingText, handled release and no release output (:4187–4205). The fresh rereceipt yields a frame bound to the current revision (:4357). The existing test-only `install_full_lease` runs the production full evaluator on that frame and supplies its worker slot (`space_autocorrect_prefetch/proof.rs:546`); it controls readiness, not authority or expected text. Space then requires exactly one Delete(-7,7), followed by exactly one Commit("работает "), matching the internal tail (:4376–4388). Seven Cyrillic scalars distinguish this from a UTF-8 byte-count deletion. Its release is handled and emits no signal (:4389–4400). FIFO markers in `legacy_effects` (:555) fence output absence without sleeps or retry-until-green. Captured signals/internal tail do not establish client consumption, visible text or learning acceptance.

## Independent evidence checks

All five referenced REGRESSION.json hashes match the packet. Original final test source is byte-identical to the GREEN and both mutation snapshots. The three snapshots share archive SHA-256 `aab713767610e61437f556f58059066588da5be60b7aad52a91a6421dbc1a945`, verified locally; all 734 archived Rust files still match current Rust files. Cargo/dependency/toolchain specification files also match. Source metadata successors differ deliberately; these checks do not promote a historical focused run into a fresh full gate or a current installed-byte proof.

| Phase and raw receipt | Selected / passed / failed | Actual scope |
| --- | --- | --- |
| `tab-characterization-gk11hdws/REGRESSION.json` | 1 / 1 / 0 | Original-code GREEN characterization, 1.7603s test execution |
| `tab-controlled-red-o3f0n_hc/REGRESSION.json` | 1 / 1 / 0 | Freshness mutation **SURVIVED**, recorded UNEXPECTED_RESULT; no RED proof |
| `tab-controlled-red-laqj4f3a/REGRESSION.json` | 1 / 0 / 1 | CONTROLLED_TRANSPORT_RED at :4376, (-7,8) versus (-7,7) |

The receipt directories above are under `/home/ubu/.cache/lay/development/`. Read the retained remote mutation files and per-process logs without running anything: the freshness copy replaces only `if !fresh_local_receipt` with `if false`, leaving token/handoff checks intact; executed SHA `7b84efe7458069ccdb7a5410c00e6f43e65604b9f7b63980351849802faf02cd` matches. The geometry copy changes only the legacy emitter argument to `nchars.saturating_add(1)` in `output.rs:289`; executed SHA `bed23c33e41e3038333398fcdd9aef869aced4e1a5b2a4cde5c18036614b77b8` matches. Its actual process log shows the exact geometry failure. This is sensitivity to a controlled transport violation, not an original native failure or a production repair. Both earlier pilot failures remain recorded as keycode and factory/profile fixture failures before correction assertions.

Manifest delta independently recomputed: exactly one added correctness/test/process tuple with the qualified test name at `scripts/test-lanes/manifest.json:1861`; no removed/reclassified identity and the same 38 targets. Inventory is 3071, required correctness/package 3045 (3009+36), process 257, target 2814. Existing discovery selects process isolation by the established test-family prefix; existing execution runs it with `--exact` in its own process/sandbox. The manifest refresh receipt and fetched successor match current bytes. These are inventory/execution identities, not coverage percentages.

The original map's 24 tuples still match the manifest and declared definition lines. All 13 map hashes match the baseline and historical request; two current successors are intentionally different (test file, manifest). Thus no historical map is silently rebound. The graph refresh receipt `tab-graph-_8nnu_i_/RESULT.json` has exit 0, its CANON_RESULT is PASS, and its log records structural graph PASS. Current graph and binding fingerprints match their receipt, with zero mismatch among 707 Rust bindings. Graph node changes are test/doc additions and community metadata, not a behavioral proof.

The ADR identifies C03/C05/C06/C07/C08/C10, the existing owners, protected manifest change, rollback and explicit untested boundaries. The owning document records every experiment, including the failed freshness sensitivity; neither weakens a guard or grants runtime authority. Pending wording in the current document is honest at this cutoff and needs an evidence-only successor when the remaining gate completes.

## Gates and completion scope

- **PASS, bounded:** implementation review; focused source characterization; controlled legacy geometry sensitivity; canonical manifest delta; recorded first graph/canon refresh and matching source bindings.
- **PENDING:** automatic whole affected remote correctness/package result and its exact current-source binding. Its final receipt has not yet been supplied to this reviewer. No full-current-tree PASS is inferred from the earlier 3044 historical result.
- **NOT TESTED / UNKNOWN:** native TD-128 delivery, TD-133 first-failure causality, stale Space effects and isolated freshness-floor sensitivity in this new sequence, actual worker timing, shell/layout activation, installed/model parity, heldout/per-class restoration quality, cold/package reload, latency/RSS and universal acceptance.

The code can be accepted without repair for the selected source gap; final bounded completion must retain the pending gate until its measured result is recorded. Native and uncharacterized data/client tasks remain open. Review performed no tests/builds, graph refresh, GUI/input/service changes, repository writes or further delegation. The sole write is this cache report.
