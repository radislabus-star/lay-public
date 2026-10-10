# TD-138 final code review — pass 2/2

**Score: 9/10. Code review PASS; no material code findings.** The fresh canonical source gate is separately **PASS**. This completes the second and final review pass; it grants no installation, runtime or native-client acceptance.

Reviewed `/home/ubu/projects/lay-space-boundary-shift-20261007`, branch `codex/space-boundary-shift-20261007`, HEAD `24bcea5256661954d5aba97ff037d6a419e1e2cb`, including the actual uncommitted TD-138 diff. Read AGENTS, ARCHITECTURE, TD-138, its owning document/decision, pass-1 report and diagnostic/RED/GREEN packets. No tests, builds, graph commands, network calls, UI actions or runtime probes were executed by this reviewer. Only this report was written.

## Code assessment

- `src/ru_typo/repeated.rs:35–69` moves the existing pure repeated-run generator before lexical lookups. Empty geometry returns `None` without loading reference membership. This applies to all eligible inputs, with no fixture literal or client-specific branch. The generator itself is unchanged and has no lexical or authority side effects.
- For actual repeated runs, the original foundation/exact-reference/live/user protections, short-function rule, length and trailing-vowel guards remain. Retaining the existing candidate vector is equivalent to the former filter/collect; sorting, deduplication, autocorrect versus proposal authority, scorer, thresholds and case restoration are unchanged. Candidate generation, scorer, policy and memo support files are byte-identical to HEAD. No shared lattice, grounded target, DecisionCore, verifier or mutation authority is removed or bypassed.
- Both latency fixtures select and assert the production `HotFieldPolicy::ime()` before the existing synchronous warmup, then assert successful warmup and ready candidate memory. Each measured call must return nonempty work. Inputs, requests, measurement intervals, budgets and existing candidate-operation assertions remain unchanged. Test-thread policy and process isolation match this bounded synchronous proof; they do not establish asynchronous production startup.
- The new cold-reference test exercises both autocorrect and proposal paths, several lengths and upper/lower case. Its explicit discovery entry produces an exact fresh-process correctness test, preventing prior global dictionary initialization from hiding the failure. Manifest totals are 3074 identities: 3012 correctness, 36 package, 11 performance, 15 previously ignored. The failure ledger remains empty, binds the actual manifest hash, and preserves the original observation bytes. No failure waiver or new ignored identity was added.

## Evidence and resource limits

The preserved RED receipt has 1818 selected, 1817 PASS and exactly the new no-repeat cold-reference assertion FAIL. The policy-only contrast remains FAIL. Controlled systemic GREEN is 6/6: actual IME 3/3 and baseline-byte-identical full-reference fixtures 3/3, with unchanged 50000us debug budgets and unique maxima 6958/6938us. The frozen variant runner restores both old functions verbatim and rebuilds from each variant's source. Thus success is not explained solely by narrowing fixture authority. Old full-gate, baseline and traced FAIL receipt hashes were independently checked and remain intact.

The final frozen archive matches all five reviewed code/discovery/manifest/ledger files. Its canonical full gate records 3048/3048 required correctness/package PASS, zero infrastructure/known failures, and the log has exactly 11/11 serialized performance PASS followed by full-check OK. Canon/format/lint/release proof completes in that same run. Canonical command time is 816.372s; wrapper time is 817.091s. Final Cargo target is 8,181,723,136 bytes below the unchanged 12GiB guard.

The inapplicable operator avoids the demonstrated cold reference load; it adds no cache, owner, timer, eager warmup or model dependency. Actual repeated inputs now materialize existing geometry before lexical protection, so their allocation cost and RSS were not separately measured. Reference loading remains reachable for applicable operators. Existing startup warmup is excluded from hot-readout timing and remains separately about 201ms IME / 7.66s reference in the controlled packet. No first-key, RSS, universal quality or native latency claim follows. The fetched 1.0.82 artifact remains uninstalled; installed/native state and TD-133's outstanding physical scope are unchanged.

## Exact binding

- Frozen archive: `tab-full-4reofpse/source.tar`, SHA-256 `4c53a2e550a38c42f4e73ed7d84e157e32e3ab169775a4e33a3a5da0361f1753`.
- Full gate: `/home/ubu/.cache/lay/development/tab-full-4reofpse/FULL_GATE.json`, SHA-256 `248d9ecbd967fd39b1985a205041b514f798798f8640c0d6527999cf9e208212`.
- Runner result: same directory `RESULT.json`, SHA-256 `b23dc047e7d0ffeecec8745a54acd7fe7c13bde6905fbf5005cf6a06ba5c48e1`.
- Controlled GREEN: `/home/ubu/.cache/lay/development/tab-manifest-su5dwwq4/TD138_SYSTEMIC_GREEN.json`, SHA-256 `fdfbff2f9f192ec78ebec47229ba47285c57ed9d9ece496d0bea386ecdc5d686`.
- Fetched uninstalled IME: 8,185,232 bytes, SHA-256 `5fe100db732bcc945d9c7d584a2c22bd06eb93c19bee09cc39788c85470dee7a`.

Verdict scope: final code review PASS; bound full source gate PASS; startup/RSS/changed-byte installed/native acceptance NOT TESTED. Runtime authority and installed runtime were not changed by TD-138 proof-scope setup or by this review.
