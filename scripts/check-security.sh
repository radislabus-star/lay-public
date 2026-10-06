#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
command -v cargo-audit >/dev/null || {
  echo 'cargo-audit is required; install it with the guarded Cargo entrypoint' >&2
  exit 127
}
# Fetch current RustSec data. Vulnerabilities fail by default; unsoundness must
# also fail, even when RustSec categorizes it as an informational advisory.
exec cargo-audit audit --file Cargo.lock --deny unsound "$@"
