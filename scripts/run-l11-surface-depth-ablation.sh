#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORPUS="${1:-$ROOT/data/lexical_grokking/lay_l11_ru462k_en300k_shadow_v1.txt}"
OUTPUT_DIR="${2:-$ROOT/artifacts/l11-surface-depth-ablation-1k}"
WAIT_PID="${3:-}"
TRAINER="${LAY_L11_TRAINER:-$ROOT/target/release/lay-nanda-wave-train}"

RU_WORDS=462314
EN_WORDS=300000
TOTAL_WORDS=$((RU_WORDS + EN_WORDS))
SAMPLE_WORDS="${LAY_L11_ABLATION_SAMPLE_WORDS:-1000}"
HELDOUT_PER_CLASS="${LAY_L11_ABLATION_HELDOUT_PER_CLASS:-2000}"
RU_SAMPLE=$(((SAMPLE_WORDS * RU_WORDS + TOTAL_WORDS / 2) / TOTAL_WORDS))
EN_SAMPLE=$((SAMPLE_WORDS - RU_SAMPLE))
DEPTHS=(2 4 8 16 32 48)
CPU_SETS=(0-2 3-5 6-8 9-11 12-14 15-17)

for command in awk jq sha256sum taskset; do
    command -v "$command" >/dev/null || {
        echo "missing required command: $command" >&2
        exit 1
    }
done

[[ -f "$CORPUS" ]] || {
    echo "corpus not found: $CORPUS" >&2
    exit 1
}
((SAMPLE_WORDS >= 8 && SAMPLE_WORDS <= TOTAL_WORDS)) || {
    echo "sample words must be between 8 and $TOTAL_WORDS, got $SAMPLE_WORDS" >&2
    exit 1
}
[[ -x "$TRAINER" ]] || {
    echo "trainer not executable: $TRAINER" >&2
    exit 1
}

mkdir -p "$OUTPUT_DIR"
SAMPLE="$OUTPUT_DIR/lay_l11_ru${RU_SAMPLE}_en${EN_SAMPLE}_fixed_${SAMPLE_WORDS}.txt"

awk \
    -v ru_words="$RU_WORDS" \
    -v en_words="$EN_WORDS" \
    -v ru_sample="$RU_SAMPLE" \
    -v en_sample="$EN_SAMPLE" '
    BEGIN {
        ru_index = 0
        en_index = 0
        next_ru = 1
        next_en = 1
    }
    NR <= ru_words {
        if (NR == next_ru) {
            print
            ru_index++
            if (ru_index < ru_sample) {
                next_ru = 1 + int(ru_index * (ru_words - 1) / (ru_sample - 1))
            }
        }
        next
    }
    {
        en_line = NR - ru_words
        if (en_line == next_en) {
            print
            en_index++
            if (en_index < en_sample) {
                next_en = 1 + int(en_index * (en_words - 1) / (en_sample - 1))
            }
        }
    }
    END {
        if (ru_index != ru_sample || en_index != en_sample) {
            printf "sample mismatch: ru=%d/%d en=%d/%d\n", ru_index, ru_sample, en_index, en_sample > "/dev/stderr"
            exit 2
        }
    }
' "$CORPUS" >"$SAMPLE"

SAMPLE_LINES="$(wc -l <"$SAMPLE")"
[[ "$SAMPLE_LINES" -eq "$SAMPLE_WORDS" ]] || {
    echo "fixed sample must contain $SAMPLE_WORDS words, got $SAMPLE_LINES" >&2
    exit 1
}

SAMPLE_SHA256="$(sha256sum "$SAMPLE" | awk '{print $1}')"
jq -n \
    --arg corpus "$CORPUS" \
    --arg sample "$SAMPLE" \
    --arg sha256 "$SAMPLE_SHA256" \
    --argjson sample_words "$SAMPLE_WORDS" \
    --argjson ru_words "$RU_SAMPLE" \
    --argjson en_words "$EN_SAMPLE" \
    --argjson heldout_per_class "$HELDOUT_PER_CLASS" \
    '{
        experiment: "L1.1 surface-depth parallel ablation",
        source_corpus: $corpus,
        sample_corpus: $sample,
        sample_sha256: $sha256,
        sample_words: $sample_words,
        ru_words: $ru_words,
        en_words: $en_words,
        heldout_per_class: $heldout_per_class,
        depths: [2, 4, 8, 16, 32, 48],
        positive_depth_varied: true,
        anti_surfaces_per_target: 1,
        runtime_authority_changed: false
    }' >"$OUTPUT_DIR/manifest.json"

if [[ -n "$WAIT_PID" ]]; then
    echo "waiting for pid $WAIT_PID before starting ablation" >&2
    while kill -0 "$WAIT_PID" 2>/dev/null; do
        sleep 30
    done
fi

pids=()
for index in "${!DEPTHS[@]}"; do
    depth="${DEPTHS[$index]}"
    cpu_set="${CPU_SETS[$index]}"
    prefix="$OUTPUT_DIR/depth-$depth"
    (
        export LAY_L11_SHADOW_ANTI_SURFACES_PER_TARGET=1
        exec /usr/bin/time -v -o "$prefix.time.txt" \
            taskset -c "$cpu_set" \
            "$TRAINER" \
            --crystallize-l1-lexical-grokking "$SAMPLE" \
            --out "$prefix.bin" \
            --heldout-per-class "$HELDOUT_PER_CLASS" \
            --training-surfaces-per-word "$depth" \
            --max-rss-mib 4096 \
            >"$prefix.receipt.json" \
            2>"$prefix.log"
    ) &
    pids+=("$!")
    echo "started depth=$depth cpus=$cpu_set pid=${pids[-1]}" >&2
done

failed=0
for index in "${!pids[@]}"; do
    if ! wait "${pids[$index]}"; then
        echo "depth ${DEPTHS[$index]} failed" >&2
        failed=1
    fi
done
[[ "$failed" -eq 0 ]] || exit 1

jq -s '
    map({
        training_surfaces_per_word: .scale_training_surfaces_per_word,
        source_words,
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
        classes
    })
    | sort_by(.training_surfaces_per_word)
' "$OUTPUT_DIR"/depth-*.receipt.json >"$OUTPUT_DIR/summary.json"

echo "$OUTPUT_DIR/summary.json"
