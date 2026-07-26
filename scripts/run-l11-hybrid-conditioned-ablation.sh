#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SAMPLE="${1:-$ROOT/artifacts/l11-causal-depth-ablation-10k-2026-07-25/lay_l11_fixed_10000.txt}"
OUTPUT_DIR="${2:-$ROOT/artifacts/l11-hybrid-conditioned-ablation-10k}"
TRAINER="${LAY_L11_TRAINER:-$ROOT/target/release/lay-nanda-wave-train}"
HELDOUT_PER_CLASS="${LAY_L11_ABLATION_HELDOUT_PER_CLASS:-2000}"

LABELS=(clean-0 legacy-4 hybrid-4 hybrid-5 hybrid-6)
POLICIES=(
    legacy-alphabetical
    legacy-alphabetical
    hybrid-class-conditioned
    hybrid-class-conditioned
    hybrid-class-conditioned
)
DEPTHS=(0 4 4 5 6)

for command in jq sha256sum taskset nproc; do
    command -v "$command" >/dev/null || {
        echo "missing required command: $command" >&2
        exit 1
    }
done
[[ -f "$SAMPLE" ]] || {
    echo "fixed sample not found: $SAMPLE" >&2
    exit 1
}
[[ -x "$TRAINER" ]] || {
    echo "trainer not executable: $TRAINER" >&2
    exit 1
}

SAMPLE_WORDS="$(wc -l <"$SAMPLE")"
SAMPLE_SHA256="$(sha256sum "$SAMPLE" | awk '{print $1}')"
CPU_COUNT="$(nproc)"
VARIANT_COUNT="${#LABELS[@]}"
((CPU_COUNT >= VARIANT_COUNT)) || {
    echo "hybrid ablation needs at least $VARIANT_COUNT CPUs, got $CPU_COUNT" >&2
    exit 1
}

mkdir -p "$OUTPUT_DIR"
jq -n \
    --arg sample "$SAMPLE" \
    --arg sha256 "$SAMPLE_SHA256" \
    --argjson sample_words "$SAMPLE_WORDS" \
    --argjson heldout_per_class "$HELDOUT_PER_CLASS" \
    --argjson cpu_count "$CPU_COUNT" \
    '{
        experiment: "L1.1 hybrid class-conditioned damaged-surface ablation",
        sample_corpus: $sample,
        sample_sha256: $sha256,
        sample_words: $sample_words,
        heldout_per_class: $heldout_per_class,
        variants: [
            {label: "clean-0", policy: "legacy-alphabetical", depth: 0},
            {label: "legacy-4", policy: "legacy-alphabetical", depth: 4},
            {label: "hybrid-4", policy: "hybrid-class-conditioned", depth: 4},
            {label: "hybrid-5", policy: "hybrid-class-conditioned", depth: 5},
            {label: "hybrid-6", policy: "hybrid-class-conditioned", depth: 6}
        ],
        hybrid_schedule: [
            "layout_projection",
            "double_substitution",
            "omission_transposition",
            "sparse_multi_omission",
            "adjacent_transposition",
            "extra_letter"
        ],
        clean_surface_always_present: true,
        anti_surfaces_per_target: 1,
        anti_probe: "clean surface in every variant",
        calibration: "same deterministic split and calibration procedure",
        cpu_count: $cpu_count,
        runtime_authority_changed: false
    }' >"$OUTPUT_DIR/manifest.json"

pids=()
for index in "${!LABELS[@]}"; do
    variant="${LABELS[$index]}"
    policy="${POLICIES[$index]}"
    depth="${DEPTHS[$index]}"
    start=$((index * CPU_COUNT / VARIANT_COUNT))
    end=$(((index + 1) * CPU_COUNT / VARIANT_COUNT - 1))
    prefix="$OUTPUT_DIR/$variant"
    (
        export LAY_L11_SHADOW_ANTI_SURFACES_PER_TARGET=1
        exec /usr/bin/time -v -o "$prefix.time.txt" \
            taskset -c "$start-$end" \
            "$TRAINER" \
            --crystallize-l1-lexical-grokking "$SAMPLE" \
            --out "$prefix.bin" \
            --heldout-per-class "$HELDOUT_PER_CLASS" \
            --training-surfaces-per-word "$depth" \
            --training-surface-policy "$policy" \
            --max-rss-mib 4096 \
            >"$prefix.receipt.json" \
            2>"$prefix.log"
    ) &
    pids+=("$!")
    echo "started variant=$variant policy=$policy depth=$depth cpus=$start-$end pid=${pids[-1]}" >&2
done

failed=0
for index in "${!pids[@]}"; do
    if ! wait "${pids[$index]}"; then
        echo "variant ${LABELS[$index]} failed" >&2
        failed=1
    fi
done
[[ "$failed" -eq 0 ]] || exit 1

for variant in "${LABELS[@]}"; do
    prefix="$OUTPUT_DIR/$variant"
    rss_kib="$(awk -F: '/Maximum resident set size/ {gsub(/^[[:space:]]+/, "", $2); print $2}' "$prefix.time.txt")"
    jq \
        --arg variant "$variant" \
        --argjson peak_rss_kib "${rss_kib:-0}" \
        '. + {experiment_label: $variant, peak_rss_kib: $peak_rss_kib}' \
        "$prefix.receipt.json" >"$prefix.measured.json"
done

jq -s '
    map({
        label: .experiment_label,
        policy: .scale_training_surface_policy,
        training_surfaces_per_word: .scale_training_surfaces_per_word,
        source_words,
        training_surfaces,
        heldout_surfaces,
        heldout_top1_percent,
        clean_preservation_percent,
        l11_authority_target_winner_percent,
        l11_evidence_target_retained_percent,
        l11_false_authority_on_objective_ambiguity,
        l11_false_singleton_on_geometry_tie,
        artifact_bytes,
        compile_ms,
        proof_ms,
        peak_rss_kib,
        classes
    })
' "$OUTPUT_DIR"/*.measured.json >"$OUTPUT_DIR/summary.json"

echo "$OUTPUT_DIR/summary.json"
