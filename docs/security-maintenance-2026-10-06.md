# Public repository security maintenance — 2026-10-06

## Scope and authorization

Source: branch `codex/lay-contextual-completion-20261005`, baseline
`86290f95566a9639b5b05ad3e6cbd31718e1832b`, package version 1.0.80.
The user authorized removing service metadata from public documents, updating
the identified dependencies and enabling recurring security checks. The
installed accepted Lay is not rebuilt, installed or restarted by this task.
No history rewrite or force push is authorized.

## First failures and changes

- Public documents and two historical proof helpers contained an actual SSH
  worker address/login. The helper address and directory defaults are replaced
  with explicit private environment settings validated before contacting SSH.
- 151 documents have machine metadata removed; all 47 changed JSON explicitly
  identify themselves as public projections. Original files were archived separately and
  every member hash verified before editing. Existing results remain historical.
- `rustls` 0.23.39 has [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285.html)
  (TLS handshake level validation), fixed in 0.23.45.
- `anyhow` 1.0.102 has [RUSTSEC-2026-0190](https://rustsec.org/advisories/RUSTSEC-2026-0190.html)
  (unsound mutable downcast), fixed in 1.0.103.
- `event-listener` 5.4.1 has [RUSTSEC-2026-0221](https://rustsec.org/advisories/RUSTSEC-2026-0221.html)
  (unsound cross-thread tags), fixed in 5.4.2.
- Lock updates are restricted to these packages and required transitive changes;
  the direct `event-listener` minimum becomes 5.4.2. No layout, text ownership,
  gesture, candidate ranking, verifier or output algorithm is changed.
- GitHub Dependabot alerts/security fixes are enabled. New daily RustSec CI
  denies vulnerabilities and unsoundness; weekly Dependabot covers Cargo and
  Actions. CI uses pinned setup actions and read-only repository permission.

## Evidence policy and limits

The [decision](architecture/decisions/2026-10-06-public-security-maintenance.json)
permits public evidence projections without replacing original sealed evidence
or weakening C01–C10. Paths and original hashes in historical projections are
display references, not guarantees that public readers can open local artifacts.
Root agent navigation/runtime templates are outside this documentary path sweep.

The prior full-history signature scan found no matches in five checked secret
formats; this does not establish that every possible secret is absent. Internal
addresses were publicly accessible; there is no evidence that someone used them.
Old commits and tags retain their previous content. Installed release binaries
remain unchanged and therefore do not inherit source dependency fixes.

## Verification

- Remote precise lock update: PASS, 6.53 seconds. Only the three requested
  packages and required `rustls-webpki` 0.103.13 → 0.103.15 changed.
- Remote `cargo-audit` 0.22.2 with `--deny unsound`: PASS, **0 vulnerabilities,
  0 warnings**, 293 registry packages. RustSec database commit
  `ef6173cbc5c50ec8166f9a5b28f07834144373ee`, 1,290 advisories, updated 2026-10-03.
  Private receipt `security-maintenance-20261006/RUSTSEC-audit.json`, SHA-256
  `cb99015bdc7522abd00f2641478dd6312daec442608480f32cdd82baacb9a2ac`.
- Remote configuration tests: **3 PASS**, including 16 rejected helper/config
  combinations that never contact SSH, valid alias/path admission and shell
  syntax. Private log `security-maintenance-20261006/REMOTE-SECURITY.log`, SHA-256
  `04a7241a203b1ed51ed5dd8cc6ea5b8572676af5915ce15ca05f10cb9c00b295`.
- Operational failures retained: the local scanner binary was incompatible with
  the worker's glibc; the scanner was therefore built on the worker. First graph
  refresh was correctly refused by the shared heavy-work lease while that build
  ran (exit 75); no limit was bypassed. A fresh graph run follows lease release.
- Current tracked tree scan finds zero instances of the former worker address.
  Independent originals archive SHA-256:
  `944638d8e19c09af8a453ac72d6b699b174d3dac23f4e9a8605fd784af370f26`.

- Independent review found no shell/CI blockers. It identified two older menu
  receipt projections missing the annotation; both now retain the original SHA
  and explicitly mark the historical original-manifest relationship.
- Fresh remote graph refresh and unchanged architecture guards: PASS, 46.24s,
  all 11 receipt dimensions. Source fingerprint
  `1f0d1fba2fa55dc71d4c4764a1f3d9b3f9f6df0f407d9053477a108446bee15c`.
  Private receipt `security-maintenance-20261006/graph-refresh-v2/REFRESH-result.json`;
  generated proof is `src/generated/architecture_graph_receipt.json`.

### Replan: exact public reference bindings

The first broad source run failed after 411 seconds: all 1,816 library tests,
685 IME tests and the remaining selected runtime tests passed, while
`td113_unsuperseded_protected_artifacts_match_the_v4_preflight_baseline` correctly
rejected a changed review-document hash. The lane validator also rejected the
changed observation-file bytes. No failure was accepted or ignored.
Private failed receipt: `development/run-a2s7o4n2/RESULT.json`.

After two review passes, repair the shared mechanism rather than individual
test expectations: mark every redacted JSON, refresh seven successor-binding
documents in dependency order, and retain their original hashes/references in
`public_redaction`. Three previously unchanged binding documents consequently
become marked projections too. Runtime source hashes, verdicts, test identities,
assertions and validators remain unchanged. The current zero-failure contract
updates only `observation_sha256`; its failure count remains **0**, its failure
list remains empty, and its test-manifest hash is unchanged. The independent
archive for these original bindings has SHA-256
`6411bffbd8abeb3ff2d052a34f80b55b28d35009a17fea752d19800c0f9f1dd4`.
Private derivation receipt: `security-maintenance-20261006/REFERENCE-BINDINGS.json`.

## Final result

**PASS for source security maintenance, not new runtime release acceptance.**

- Broad remote development gate: **3,010 selected tests PASS**, 0 known semantic
  failures and 0 infrastructure failures, 378.3 seconds including transfer/setup.
  Scope: 2,974 correctness + 36 package tests; 15 ignored and 11 performance
  tests excluded by the unchanged canonical selection.
  Private receipt `development/run-opj1s8qk/RESULT.json`, SHA-256
  `0faedc693c5b3723f0a64e83f55f9a53dd7b1436d5c4e0c4b35b32c04d409cf0`.
- Final remote canon validation: 0 errors; 18 configuration/canon tests PASS.
  Final RustSec audit again reports 0 vulnerabilities and 0 warnings.
  Private log `security-maintenance-20261006/REMOTE-FINAL-CHECKS.log`, SHA-256
  `f3ed2231e39810645641418793a1464d2fb1298fb0bbf46f9ec1ee772b79e0fa`.
- Independent review: 0 blockers, 7 original binding hashes verified, 9 original
  reference hashes retained, 12 current public references verified, DAG acyclic,
  28 other SHA pins unchanged. Private receipt
  `security-maintenance-20261006/INDEPENDENT-REVIEW.json`, SHA-256
  `47a0446f1f50854106fa30ae5fb95e80bd769b837ba5624ccea8ecdac61c25b2`.
- Cargo target: 10,165,227,520 bytes, below the unchanged 12 GiB limit.
- Final current-tree scan: zero occurrences of the former worker address/login;
  no runtime Rust source or lane validator changed. Installed binary hashes are
  retained separately for the publication readback.
- GitHub secret scanning/push protection and Dependabot security updates enabled;
  scheduled Security workflow plus weekly Dependabot configuration included.

Physical client acceptance, installation, exhaustive secret detection, complete
release/performance gates and history rewriting were not performed. Old Git
history retains previous identifiers. No runtime authority changed.
