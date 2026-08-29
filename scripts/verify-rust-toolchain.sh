#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

profile="${1:-}"
case "$profile" in
  msrv)
    toolchain="1.88.0"
    rustc_release="1.88.0"
    rustc_commit="6b00bc3880198600130e1cf62b8f8a93494488cc"
    cargo_version="cargo 1.88.0 (873a06493 2025-05-10)"
    ;;
  lint)
    toolchain="1.97.1"
    rustc_release="1.97.1"
    rustc_commit="8bab26f4f68e0e26f0bb7960be334d5b520ea452"
    cargo_version="cargo 1.97.1 (c980f4866 2026-06-30)"
    ;;
  *)
    echo "usage: scripts/verify-rust-toolchain.sh msrv|lint" >&2
    exit 2
    ;;
esac

rustc_command=(rustc)
cargo_command=(cargo)
clippy_command=(clippy-driver)
rustfmt_command=(rustfmt)
if [[ "$profile" == "msrv" ]]; then
  rustc_command+=("+$toolchain")
  cargo_command+=("+$toolchain")
fi

rustc_identity="$("${rustc_command[@]}" -Vv)"
cargo_identity="$("${cargo_command[@]}" -V)"

grep -Fxq "release: $rustc_release" <<<"$rustc_identity"
grep -Fxq "commit-hash: $rustc_commit" <<<"$rustc_identity"
grep -Eq '^host: .+' <<<"$rustc_identity"
[[ "$cargo_identity" == "$cargo_version" ]]

if [[ "$profile" == "lint" ]]; then
  [[ "$("${clippy_command[@]}" -V)" == "clippy 0.1.97 (8bab26f4f6 2026-07-14)" ]]
  [[ "$("${rustfmt_command[@]}" -V)" == "rustfmt 1.9.0-stable (8bab26f4f6 2026-07-14)" ]]
fi

printf 'toolchain_profile=%s\n%s\n%s\n' "$profile" "$rustc_identity" "$cargo_identity"
