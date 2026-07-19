#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
URL="${LAY_L3_CORPUS_URL:-https://downloads.tatoeba.org/exports/per_language/rus/rus_sentences.tsv.bz2}"
LIMIT="${LAY_L3_MAX_FRAGMENTS:-60000}"
MIN_PROFILE_SUPPORT="${LAY_L3_MIN_PROFILE_SUPPORT:-2}"
OUT="${LAY_L3_CONTEXT_OUT:-$ROOT/data/lexicon/l3_context_phase_v1.nwpc}"
MANIFEST="${LAY_L3_CONTEXT_MANIFEST:-$ROOT/data/lexicon/l3_context_phase_v1.manifest.json}"
WORK="${LAY_L3_CONTEXT_WORK:-${TMPDIR:-/tmp}/lay-l3-context-phase}"
TRAINER="${LAY_NANDA_WAVE_TRAIN:-$ROOT/target/release/lay-nanda-wave-train}"

mkdir -p "$WORK"
ARCHIVE="$WORK/rus_sentences.tsv.bz2"
CORPUS="$WORK/rus_sentences_${LIMIT}.txt"
PROOF="$WORK/proof.json"
COMPILE="$WORK/compile.json"
ARTIFACT="$WORK/l3_context_phase_v1.nwpc"

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" -o "$ARCHIVE"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$ARCHIVE" "$URL"
else
  echo "missing downloader: install curl or wget" >&2
  exit 1
fi
python3 - "$ARCHIVE" "$CORPUS" "$LIMIT" <<'PY'
import bz2
import pathlib
import sys

source, output, limit = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), int(sys.argv[3])
written = 0
sentence_like = 0
with bz2.open(source, "rt", encoding="utf-8") as rows, output.open("w", encoding="utf-8") as out:
    for row in rows:
        fields = row.rstrip("\n").split("\t", 2)
        if len(fields) != 3:
            continue
        sentence = fields[2].strip()
        if len(sentence.split()) < 2:
            continue
        out.write(sentence + "\n")
        written += 1
        sentence_like += sentence[-1:] in ".!?…"
        if written >= limit:
            break
if written < limit:
    raise SystemExit(f"not enough usable Russian sentences: {written} < {limit}")
if sentence_like * 100 < written * 20:
    raise SystemExit(f"corpus does not look like natural sentences: punctuation={sentence_like}/{written}")
print(f"natural_corpus: rows={written} sentence_like={sentence_like}")
PY

"$TRAINER" --prove-l3-context-phase "$CORPUS" \
  --max-fragments "$LIMIT" --min-profile-support "$MIN_PROFILE_SUPPORT" > "$PROOF"
jq -e '.verdict == "PASS" and .full_false_top1 == 0 and .phase_ablation_drop > 0 and .semantic_ablation_drop > 0' "$PROOF" >/dev/null
"$TRAINER" --compile-l3-context-phase "$CORPUS" \
  --out "$ARTIFACT" --max-fragments "$LIMIT" \
  --min-profile-support "$MIN_PROFILE_SUPPORT" > "$COMPILE"

install -m 0644 "$ARTIFACT" "$OUT"
artifact_sha="$(sha256sum "$OUT" | cut -d' ' -f1)"
archive_sha="$(sha256sum "$ARCHIVE" | cut -d' ' -f1)"
corpus_sha="$(sha256sum "$CORPUS" | cut -d' ' -f1)"

jq -n \
  --arg artifact "$(basename "$OUT")" \
  --arg artifact_sha256 "$artifact_sha" \
  --arg source_url "$URL" \
  --arg source_license "CC BY 2.0 FR" \
  --arg source_license_url "https://creativecommons.org/licenses/by/2.0/fr/" \
  --arg archive_sha256 "$archive_sha" \
  --arg corpus_sha256 "$corpus_sha" \
  --argjson compile "$(cat "$COMPILE")" \
  --argjson heldout "$(cat "$PROOF")" \
  '{
    format: "LAYL3P01",
    version: 1,
    cells: 64,
    artifact: $artifact,
    artifact_bytes: $compile.artifact_bytes,
    artifact_sha256: $artifact_sha256,
    raw_words_stored: $compile.raw_words_stored,
    corpus: {
      source: "Tatoeba Russian per-language sentence export",
      source_url: $source_url,
      license: $source_license,
      license_url: $source_license_url,
      archive_sha256: $archive_sha256,
      extracted_sha256: $corpus_sha256,
      committed: false,
      fragments: $compile.corpus_fragments
    },
    transitions: $compile.transitions,
    semantic_states: $compile.semantic_states,
    candidate_profiles: $compile.candidate_profiles,
    positive_centers: $compile.positive_centers,
    anti_centers: $compile.anti_centers,
    l2_lattice_negative_examples: $compile.l2_lattice_negative_examples,
    min_profile_support: $compile.min_profile_support,
    competition_threshold_micro: $compile.competition_threshold_micro,
    heldout: $heldout
  }' > "$MANIFEST"

echo "l3_context_phase: artifact=$OUT manifest=$MANIFEST"
jq '{artifact_bytes, corpus, transitions, semantic_states, candidate_profiles, positive_centers, anti_centers, l2_lattice_negative_examples, heldout: {verdict: .heldout.verdict, support_precision_ppm: .heldout.support_precision_ppm, full_false_top1: .heldout.full_false_top1, anti_false_top1_reduction: .heldout.anti_false_top1_reduction, phase_ablation_drop: .heldout.phase_ablation_drop, semantic_ablation_drop: .heldout.semantic_ablation_drop}}' "$MANIFEST"
