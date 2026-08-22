#!/usr/bin/env bash
set -euo pipefail

remote=${LAY_PROOF_REMOTE:-e@192.168.3.94}
project_root=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)
proof=/home/e/projects/lay-atomic-full-route-20260821
target=/home/e/projects/lay-v22-canonical-20260821/target

source_inputs=(
    Cargo.toml
    Cargo.lock
    build.rs
    scripts
    src
    tests
    data/lexicon
    data/test_input
    data/morphology/russian_noun_cases_small.tsv
    data/nanda_llmwave_seed_phrases.txt
)

source_aggregate() {
    cd "$project_root"
    find "${source_inputs[@]}" -type f -print0 |
        LC_ALL=C sort -z |
        xargs -0 sha256sum |
        sha256sum |
        awk '{print $1}'
}

local_source_sha=$(source_aggregate)
source_root=/home/e/projects/lay-atomic-v25-src-$local_source_sha

ssh "$remote" "mkdir -p '$source_root' \
    '$proof/runtime/engines' '$proof/runtime/manifests' '$proof/runtime/bin' \
    '$proof/logs' '$proof/output'"

(cd "$project_root" && rsync -aR "${source_inputs[@]}" "$remote:$source_root/")
rsync -a "$project_root/scripts/proof/verify-atomic-proof-engine.sh" \
    "$remote:$proof/runtime/bin/"

remote_source_sha=$(ssh "$remote" "cd '$source_root' && \
    find Cargo.toml Cargo.lock build.rs scripts src tests data/lexicon data/test_input \
        data/morphology/russian_noun_cases_small.tsv data/nanda_llmwave_seed_phrases.txt \
        -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | \
        sha256sum | awk '{print \$1}'")
[ "$remote_source_sha" = "$local_source_sha" ] || {
    printf 'remote source parity failed: local=%s remote=%s\n' \
        "$local_source_sha" "$remote_source_sha" >&2
    exit 65
}

ssh "$remote" "cd '$source_root' && \
    CARGO_TARGET_DIR='$target' CARGO_BUILD_JOBS=20 \
        scripts/cargo-guard.sh test --lib \
        exact_scope_accepts_only_raw_known_layout_projection \
        -- --nocapture --test-threads=1 && \
    CARGO_TARGET_DIR='$target' CARGO_BUILD_JOBS=20 \
        scripts/cargo-guard.sh test --lib \
        exact_layout_scope_preserves_full_route_authority_and_proof \
        -- --nocapture --test-threads=1 && \
    CARGO_TARGET_DIR='$target' CARGO_BUILD_JOBS=20 \
        scripts/cargo-guard.sh test --lib \
        exact_layout_scope_rejects_protected_and_composite_inputs \
        -- --nocapture --test-threads=1 && \
    CARGO_TARGET_DIR='$target' CARGO_BUILD_JOBS=20 \
        scripts/cargo-guard.sh test --bin lay-ibus-engine atomic::tests \
        -- --nocapture --test-threads=1 && \
    CARGO_TARGET_DIR='$target' CARGO_BUILD_JOBS=20 \
        scripts/cargo-guard.sh test --bin lay-ibus-engine atomic::proof \
        -- --nocapture --test-threads=1 && \
    CARGO_TARGET_DIR='$target' CARGO_BUILD_JOBS=20 \
        scripts/cargo-guard.sh test --bin lay-ibus-engine engine::profile_tests \
        -- --nocapture --test-threads=1 && \
    CARGO_TARGET_DIR='$target' CARGO_BUILD_JOBS=20 \
        scripts/cargo-guard.sh test --bin lay-ibus-engine \
        space_autocorrect_prefetch::tests \
        -- --nocapture --test-threads=1 && \
    CARGO_TARGET_DIR='$target' CARGO_BUILD_JOBS=20 \
        scripts/cargo-guard.sh build --release --bin lay-ibus-engine"

ssh "$remote" "bash -s" -- "$source_root" "$target" "$proof" "$local_source_sha" <<'REMOTE'
set -euo pipefail
source_root=$1
target=$2
proof=$3
source_sha=$4
built=$target/release/lay-ibus-engine

[ -x "$built" ] || {
    echo "release atomic proof engine was not built" >&2
    exit 66
}
strings "$built" | grep -F 'ProcessKeyEventAtomicV1' >/dev/null || {
    echo "release build lacks ProcessKeyEventAtomicV1" >&2
    exit 65
}

binary_sha=$(sha256sum "$built" | awk '{print $1}')
stage=$proof/runtime/engines/$binary_sha
mkdir -p "$stage"
install -m 0755 "$built" "$stage/lay-ibus-engine.next"
mv "$stage/lay-ibus-engine.next" "$stage/lay-ibus-engine"

(cd "$stage" && sha256sum lay-ibus-engine >lay-ibus-engine.sha256)
(cd "$source_root" &&
    find Cargo.toml Cargo.lock build.rs scripts src tests data/lexicon data/test_input \
        data/morphology/russian_noun_cases_small.tsv data/nanda_llmwave_seed_phrases.txt \
        -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum \
        >"$stage/source-files.sha256")
{
    printf 'schema=lay.atomic-proof-engine.v1\n'
    printf 'binary_sha256=%s\n' "$binary_sha"
    printf 'source_aggregate_sha256=%s\n' "$source_sha"
    printf 'source_root=%s\n' "$source_root"
    printf 'required_method=ProcessKeyEventAtomicV1\n'
} >"$stage/provenance.env"

chmod 0444 \
    "$stage/lay-ibus-engine.sha256" \
    "$stage/source-files.sha256" \
    "$stage/provenance.env"
"$proof/runtime/bin/verify-atomic-proof-engine.sh" "$stage" "$binary_sha" >/dev/null

printf '%s\n' "$binary_sha" >"$proof/runtime/manifests/active-engine.next"
mv "$proof/runtime/manifests/active-engine.next" \
    "$proof/runtime/manifests/active-engine"
printf 'engine_sha256=%s\nsource_aggregate_sha256=%s\nstage=%s\n' \
    "$binary_sha" "$source_sha" "$stage"
REMOTE
