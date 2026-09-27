# Lay project instructions

Read [AGENTS.md](AGENTS.md) and [ARCHITECTURE.md](ARCHITECTURE.md) before work.
They own the working rules and architecture contract for this checkout.
Follow the owning documents and checks they reference; do not keep a separate
architecture or infer current runtime state from an older session.

Preserve the existing product constraints: no clipboard for daemon corrections;
simple mode stays deterministic; LLM mode is opt-in; production typed-text
logging and learning logs are opt-in. Diagnostic documentation is not an
instruction to restart a service. Use the guarded test routes from AGENTS.md.
