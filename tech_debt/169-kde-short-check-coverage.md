# TD169 — KDE short check omits real implementation tests

P1 SOURCE_AND_INSTALL_VERIFIED;CI_PUBLICATION_PENDING. Baseline121. Existing kde-focus command selects8guard/backend
cases;40implementation tests absent (15daemon focus,15engine focus,10exactlayout).
Owner docs/development/release-1.0.122-kde-component-coverage.md. Minimal repair:
three selector families,existing runner unchanged. Required RED missing coverage
and fail-closed absent-family regression;GREEN unit contracts and actual48case
canonical execution;full release/build/CI/publication checks. No runtime change
or claim of native input correctness;TD168 remainsOPEN.

RED2methods4assertion failures0errors onold8-case recipe;GREEN5unit+48realcanonicalcases. Full3617Rust/442Python/17gates/all10zero-warning/loaded122/native2controls verified. No runtimebody change.
