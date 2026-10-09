# TD-136 implementation review — pass 2 (final)

Verdict: **ACCEPT, 8/10**, for bounded source completion once the declared final graph/canon and exact compiled-metadata gates pass. No blocking code or evidence-admission finding. This is the second and final implementation review; no further review pass is proposed. Cutoff: **2026-10-09 20:17:58 UTC**. Final generated-artifact gates remain **PENDING** at this cutoff.

The original automatic all-command remains **FAIL/exit1**, with lane **BLOCKED_CONTRACT**. The separately admitted result is **PASS_BODY_REUSE_AND_METADATA_ADMISSION**: 3045 completed, unchanged source-test bodies plus corrected metadata admission. This review does not convert the original command to PASS, declare a fresh execution, waive a release gate or establish native acceptance.

## Scope and unchanged implementation

Reviewed `/home/ubu/projects/lay-space-boundary-shift-20261007`, branch `codex/space-boundary-shift-20261007`, HEAD `60e0d095f4468d744762831d56336fe428583fb4`. Reused the instructions and production-path inspection from pass1; ran the existing Graphify query again for navigation without refreshing it. Since pass1, the new test remains SHA-256 `29e23c2c6e913c7fa251a16b78d200860a3bc9d9902fa2a1a8ea23ebb718a48f`, and production Rust logic remains unchanged. The source-tree diff is still only the appended test and generated architecture receipt. Reviewed the ledger delta, extended ADR, owning-document/result successors, original request/archive/logs and guarded adjudication implementation/receipts.

Pass1 is unchanged, SHA `f74bcdb25c325f50bc9f5627d31f0a4521138546c42fbba73a4b3b09162ac4ba`; its repository copy is byte-identical. Its earlier pending-gate statement remains historical rather than being overwritten.

## Prioritized findings

**No P0/P1/P2 blocker.** The original error is a required metadata binding failure, not a semantic regression to conceal. Repairing the sole manifest fingerprint while independently proving the body result is valid for this bounded source result. No assertion, failure row, lane, test identity, isolation rule or production authority was weakened.

The final metadata gates are a completion condition, not a request for an implementation repair. After their exact final-byte receipts pass, the bounded source result may be accepted. Larger native/data/performance cells remain separately unproved.

## Original execution and independent log verification

The immutable `run-zib5c0pj/RESULT.json` still has verdict FAIL, returncode 1 and archive `1d9baa39fb90e8ae4fe1980c1fe5f298075b51545522f6b60d58a1599465d2f6`. Its formatting and existing tooling self-test commands returned 0; the `all` command returned 1. Its `{}` test summary and empty failure list alone would not establish successful execution.

I independently parsed the retained log bundle rather than relying on those placeholders or the derived packet. Exactly **278** expected log paths match the bundle and adjudication, with all hashes equal. Their status identities are exactly **3045 unique** required manifest identities, every one `ok`; there are zero missing/extra identities, failed statuses or duplicate status lines. Each log has exactly one matching successful summary with its exact selected count. The aggregate is **3009 correctness + 36 package**, over **33 executing targets**. The 38-target/3071-test manifest inventory is a separate denominator.

The new process-isolated test is present in `tests/logs/process/bin-lay-ibus-engine-01accfcd953678e8.log`: 1 selected, 1 successful, SHA `5e4ed0ec3abb39ccee0d4d2219a074730dbcddbbbc9f38c0b4de7d6fd8bec9c0`.

The original run log has all 33 expected target completion lines, exact aggregate 3045, zero reported failures and one terminal error at `run-zib5c0pj/run.log:176`: known-failure manifest bound to a different test manifest. Source inspection establishes why this is post-body admission: `scripts/test_lanes/cli.py:146` completes each target, :160 checks source stability, and only :191 loads the ledger. `scripts/test_lanes/execution.py:162` requires every expected harness status and :178 validates each process exit. The unchanged CLI catches this ledger ContractError at :375 and writes BLOCKED_CONTRACT. No earlier infrastructure failure or missing body is inferred away.

## Source binding and ledger repair

Independently matched all **1176** relevant archived request rows, original source archive, fresh replay snapshot and current files, including modes and exact file sets. Roots are the existing `.cargo`, Cargo.lock, Cargo.toml, benches, build.rs, data, examples, scripts, src and tests closure. There are no missing/extra relevant files or hash/mode mismatches. Separately checked `rust-toolchain.toml`, which matches the executed archive although outside that closure. Both source archive hashes match their recorded requests. Test source, manifest and unchanged parser/admission/runner code match the original request; read-only inspection also confirms their key staged-remote hashes and modes.

Only `scripts/test-lanes/known_failures.json:4` changes: old manifest `e54c838bbd3033bd0cfd11a962d229adc650400b9fc763bd3317dead9d1b98ff` becomes current manifest `d25685cfce7655d0a89c59209689cf8ba83a287512c44c46d65d703361801788`. Every other ledger value is unchanged: failure_count 0, failures [], observation path and SHA. The historical observation bytes still hash to `0bcddcfabdb78fd2bbbaa8d37222eb076eed490a9b805e73dc4a73ccc99ecf3c`; they are provenance for the empty ledger, not current quality evidence.

`load_known_failures` (`contracts.py:178`) retains its manifest hash, row/count, observation-byte, schema and exact identity checks. `compare_known_failures` (:258) retains unexpected/fixed/signature failure rejection. The private replay script :85 reconstructs the exact correctness/package selection, validates all actual successful logs, and only then calls the existing comparison with the independently established empty observed-failure set. It is not assuming that an empty ledger means tests passed. The fresh remote run returned exit 0 under the existing heavy guard; its CANON_RESULT has PASS and zero errors. The raw remote ADJUDICATION digest matches the local receipt, and every copied adjudication field equals the published packet.

The generated architecture receipt is explicitly excluded from replay's reusable semantic closure and assigned separate final validation. At this cutoff it is still byte-identical to the original all-run archive. This makes the current body reuse sound; it does not certify a future refreshed receipt. Its embedded bytes (`src/architecture_contract.rs:24`) require the declared exact compiled-metadata check after final graph refresh. No future executable-byte identity follows merely from unchanged Rust logic.

## Evidence integrity and source-only limits

All seven receipt references in the published adjudication packet have the claimed hashes, including the original automatic RESULT/request, retained bundle, fresh guarded RESULT/details/canon and private replay implementation. Original raw receipts and logs remain unchanged. The extended ADR explicitly protects both manifest and ledger and limits repair to the fingerprint, preserving historical provenance and the original FAIL. The owning result at `docs/architecture/tech-debt-maintenance-2026-10-09.md:356` records the failure, completed bodies and separate admission. The source packet and TD136 card retain pending final artifact checks rather than declaring an all-command PASS.

Pass1's measured limits remain: the freshness-floor mutation **SURVIVED**; it does not prove isolated guard sensitivity. Controlled legacy output geometry is the actual RED, not an original runtime defect. The new sequence proves production callback frame admission/retirement and exact captured output under deterministic prepared readiness. It does not execute stale Space effects, typed FocusInId wire dispatch, native focus/widget consumption, actual worker scheduling or installed model parity. Native TD128 and causal TD133 remain open; heldout/per-class quality, cold/package reader/reload behavior, latency/RSS and universal acceptance remain UNKNOWN/NOT_TESTED. The metadata repair adds no evidence in those domains.

## Recorded bindings and remaining gates

| Artifact | SHA-256 |
| --- | --- |
| Current ledger | `fd2854312fef09c3b83ea4d22eac274062f609bb4dbda5d0cb9718bafba81659` |
| Extended ADR | `0d0e27054be1251e561122b06c3483f9a7e50c4bc1ba8389be85dae414f934f9` |
| Owning document at cutoff | `0e14af8afceced3157b08359ce20635509ca793441c47d6d57dcb8db33ddce01` |
| Source-characterization successor | `381cb0467a3278c74c861f4f1cdd015827e18affcbe0376bf14f4f04952ae1e2` |
| Published fixed-proof adjudication | `62c8e3e1c750658bfea42d0a3f95763e8c9a716e239e4209758dd898187d4968` |
| Original automatic RESULT | `5aec324a3f5181469a37e86ec16ef41ec7e2f5984d06d1db94e547251356b183` |
| Original blocked lane SUMMARY | `b02aa3720ee5c1b86b8edb02c85f1ffed11335afe5a2eec6b2e0159c5c5b3e30` |
| Retained log bundle | `6a4a5c9cc50d008c8de7158f5b755d820653e05b86a031d55dd496e2aff43650` |
| Guarded raw ADJUDICATION | `e616da26680bcd7c4f756b9a941b4f598914f1e5984f3dce58e36cf34221b8cb` |

Local receipt base is `/home/ubu/.cache/lay/development/`. Original automatic receipt is `run-zib5c0pj/RESULT.json`; guarded admission/canon/details are in `tab-lane-admission-deq2sl20/`; bundle is `td136-lane-logs-20261009.tar`; replay implementation is `td136-lane-admission-20261009.py`. Repository result packet is `tech_debt/evidence/2026-10-09-td136-fixed-proof-adjudication.json`.

- **PASS, bounded:** unchanged implementation review; exact completed-body reuse; current empty-ledger admission/comparison; fresh recorded canon/link/decision admission.
- **PENDING:** final canonical graph/source binding/canon refresh and exact compiled-metadata checks against final artifact bytes, followed by recording their measured receipt in the owning result. A stale first-graph receipt cannot close these final-byte gates.
- **Retained failure:** original automatic all-command FAIL/exit1 and lane BLOCKED_CONTRACT; this status remains visible after bounded completion.

This review ran no tests/builds, graph refresh, GUI/input/service operations or subagents and made no repository edit. Only this pass2 cache report was written; original pass1 remains untouched.
