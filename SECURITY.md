# Security policy

Please report a suspected vulnerability through GitHub's **Report a
vulnerability** feature when available. Otherwise contact the maintainer
privately before publishing exploit details or sensitive data in an issue.

## Dependency checks

The Security workflow audits `Cargo.lock` against current RustSec advisories on
every push/PR to `main` and daily at 05:17 UTC. Vulnerabilities and advisories
classified as unsoundness fail the check. No advisory exceptions are configured.
Dependabot proposes Cargo and GitHub Actions updates weekly; proposals require
review and are not automatically merged or installed.

GitHub secret scanning and push protection are enabled for the public repository.
These checks cannot prove the absence of every secret or identifying detail.

## Public diagnostic evidence

Never commit real SSH logins/addresses, credentials, private worker configuration,
user drafts, or raw identifying logs. Use display aliases such as
`builder@worker.example`, `/workspace/worker/` and `/workspace/local/` in public
examples. Retain actual machine settings outside the repository.

The 2026-10-06 cleanup replaced machine identifiers in public historical
documents. Changed sealed execution receipts carry `public_redaction` metadata
and the SHA-256 of their original bytes. These public copies are redacted
projections, not immutable execution receipts. Their historical source/result
hashes still identify original artifacts, not the redacted copies. Current
fixture `binding` and `report` references use the exact hashes of the public
files they validate, with original hashes preserved in
`public_redaction.original_references`. The zero-failure test contract pins
the projected observation bytes and still permits zero failures. Original
bytes are retained in a separately verified private archive; the previous Git
snapshot also retains them. Existing assertion results are historical evidence,
not proof of a future build or of all application fields.

## History and installed releases

Removing an identifier in a new commit leaves it accessible in old Git commits,
tags, forks and downloaded copies. This cleanup does not rewrite history. If a
credential is exposed, revoke or rotate it; deleting a file is insufficient.

Dependency fixes in source do not patch an already installed binary. Updating
the runtime remains a separate build, acceptance and installation operation.
