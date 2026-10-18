# GitHub Actions v7 dependency updates

Date: 2026-10-18. Scope: public PRs #51 and #54, integrated as one CI change.

The first failures at the original October 6 heads are explicit: the canon
job rejects workflow edits without a new decision, and the Rust job rejects
the architecture receipt after `.github/workflows/ci.yml` changes. The original
failed job logs are retained under
`/home/ubu/.cache/lay/development/public-followup-20261018/`. Neither failure
establishes an application regression or permits weakening a guard.

Apply the two upstream dependency patches to the current accepted source.
Checkout moves to v7 in the three workflows; the security job retains
`persist-credentials: false` and pins upstream v7.0.1 commit
`3d3c42e5aac5ba805825da76410c181273ba90b1`. Setup Node moves to v7; the selected
Node.js version remains 22. Runner images, Rust versions, permissions, tests,
assertions and resource limits retain their existing values.

Upstream [checkout](https://github.com/actions/checkout/tree/v7) and
[setup-node](https://github.com/actions/setup-node/tree/v7) require Actions
Runner 2.327.1 or later for their Node 24 action runtime. All current jobs use
GitHub-hosted Ubuntu runners. Checkout's safer fork defaults apply to
`pull_request_target` and `workflow_run`; these workflows use neither trigger
and add no opt-out. Setup Node's removed dummy authentication token does not
affect this job: it supplies no registry or package-publishing configuration.

Acceptance requires the supported remote graph refresh, independent review
and five successful exact-head GitHub checks, including MSRV, the full Rust
correctness/package lanes, installer regressions, security and canon checks.
The public PR and private follow-up status receipt own their actual outcomes;
this decision does not predeclare PASS. A local YAML parse alone cannot prove
the new actions execute on GitHub's runners.

No runtime input authority, delivery route, model or installed binary changes.
New physical-client acceptance is NOT_TESTED. This change does not certify
the reporter's unresolved Orange Pi login failure in issue #47.
