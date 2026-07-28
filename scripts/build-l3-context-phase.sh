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
RELATION_TEACHER="${LAY_L3_RELATION_TEACHER:-$ROOT/data/lexicon/l3_relation_roles_teacher_v1.txt}"

mkdir -p "$WORK"
ARCHIVE="$WORK/rus_sentences.tsv.bz2"
NATURAL_CORPUS="$WORK/rus_sentences_${LIMIT}_natural.txt"
CORPUS="$WORK/rus_sentences_${LIMIT}.txt"
SURFACE_EVIDENCE="$WORK/surface_geometry.jsonl"
BUILD="$WORK/build-and-prove.json"
ARTIFACT="$WORK/l3_context_phase_v1.nwpc"

if ! bzip2 -t "$ARCHIVE" 2>/dev/null; then
  if command -v curl >/dev/null 2>&1; then
    curl --retry 5 --retry-delay 2 --retry-all-errors -fsSL "$URL" -o "$ARCHIVE"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$ARCHIVE" "$URL"
  else
    echo "missing downloader: install curl or wget" >&2
    exit 1
  fi
fi
bzip2 -t "$ARCHIVE"
python3 - "$ARCHIVE" "$NATURAL_CORPUS" "$LIMIT" <<'PY'
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

python3 - "$RELATION_TEACHER" "$NATURAL_CORPUS" "$CORPUS" "$LIMIT" <<'PY'
import pathlib
import sys

teacher = pathlib.Path(sys.argv[1])
natural = pathlib.Path(sys.argv[2])
output = pathlib.Path(sys.argv[3])
limit = int(sys.argv[4])
teacher_lines = [
    line.strip()
    for line in teacher.read_text(encoding="utf-8").splitlines()
    if len(line.split()) >= 3
]
natural_lines = natural.read_text(encoding="utf-8").splitlines()
base_teacher = teacher_lines[:148]
extension_teacher = teacher_lines[148:]
natural_budget = limit - len(teacher_lines)
if natural_budget < 0:
    raise SystemExit(
        f"relation teacher exceeds L3 fragment budget: {len(teacher_lines)} > {limit}"
    )
if len(extension_teacher) % 5 != 0:
    raise SystemExit(
        "relation teacher extension must be divisible by 5 "
        "to preserve the fixed ordinal heldout partition"
    )
combined = base_teacher + extension_teacher + natural_lines[:natural_budget]
if len(combined) < limit:
    raise SystemExit(f"not enough combined L3 fragments: {len(combined)} < {limit}")
output.write_text("\n".join(combined) + "\n", encoding="utf-8")
print(
    f"combined_corpus: rows={len(combined)} "
    f"relation_teacher_base={len(base_teacher)} "
    f"natural={natural_budget} relation_teacher_extension={len(extension_teacher)}"
)
PY

python3 - "$CORPUS" "$SURFACE_EVIDENCE" <<'PY'
import json
import pathlib
import re
import sys

corpus = pathlib.Path(sys.argv[1])
output = pathlib.Path(sys.argv[2])
seen = set()
words = []
for line in corpus.read_text(encoding="utf-8").splitlines():
    for word in re.findall(r"[А-Яа-яЁё]+", line):
        word = word.lower()
        if 5 <= len(word) <= 12 and word not in seen:
            seen.add(word)
            words.append(word)
            if len(words) >= 512:
                break
    if len(words) >= 512:
        break

rows = []
for word in words:
    index = max(1, min(len(word) - 2, len(word) // 2))
    replacement = "а" if word[index] != "а" else "б"
    rows.extend(
        [
            {"class": "missing_letter", "from": word[:index] + word[index + 1 :], "to": word},
            {"class": "extra_letter", "from": word[:index] + word[index] + word[index:], "to": word},
            {
                "class": "adjacent_transposition",
                "from": word[:index] + word[index + 1] + word[index] + word[index + 2 :],
                "to": word,
            },
            {
                "class": "letter_substitution",
                "from": word[:index] + replacement + word[index + 1 :],
                "to": word,
            },
        ]
    )
    if len(word) >= 7:
        second = min(len(word) - 1, index + 2)
        rows.append(
            {
                "class": "sparse_multi_omission",
                "from": "".join(
                    char for position, char in enumerate(word) if position not in (index, second)
                ),
                "to": word,
            }
        )

with output.open("w", encoding="utf-8") as target:
    for row in rows:
        target.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")
print(f"surface_geometry: source_words={len(words)} rows={len(rows)}")
PY

"$TRAINER" --build-and-prove-l3-context-phase "$CORPUS" \
  --out "$ARTIFACT" --max-fragments "$LIMIT" \
  --min-profile-support "$MIN_PROFILE_SUPPORT" \
  --surface-evidence "$SURFACE_EVIDENCE" > "$BUILD"
jq -e '.package_published == true and .heldout.verdict == "PASS" and .heldout.full_false_top1 == 0 and .heldout.support_coverage_ppm >= .heldout.min_support_coverage_ppm and .heldout.phase_improved_cases > .heldout.phase_worsened_cases and .heldout.anti_improved_cases > .heldout.anti_worsened_cases and .heldout.candidate_permutation_mismatches == 0 and .heldout.pairwise_worsened_cases == 0' "$BUILD" >/dev/null

install -m 0644 "$ARTIFACT" "$OUT"
artifact_sha="$(sha256sum "$OUT" | cut -d' ' -f1)"
archive_sha="$(sha256sum "$ARCHIVE" | cut -d' ' -f1)"
corpus_sha="$(sha256sum "$CORPUS" | cut -d' ' -f1)"
relation_teacher_sha="$(sha256sum "$RELATION_TEACHER" | cut -d' ' -f1)"
surface_evidence_sha="$(sha256sum "$SURFACE_EVIDENCE" | cut -d' ' -f1)"

jq -n \
  --arg artifact "$(basename "$OUT")" \
  --arg artifact_sha256 "$artifact_sha" \
  --arg source_url "$URL" \
  --arg source_license "CC BY 2.0 FR" \
  --arg source_license_url "https://creativecommons.org/licenses/by/2.0/fr/" \
  --arg archive_sha256 "$archive_sha" \
  --arg corpus_sha256 "$corpus_sha" \
  --arg relation_teacher "$(basename "$RELATION_TEACHER")" \
  --arg relation_teacher_sha256 "$relation_teacher_sha" \
  --arg surface_evidence_sha256 "$surface_evidence_sha" \
  --argjson build "$(cat "$BUILD")" \
  '{
    format: "LAYL3P01",
    version: 5,
    cells: 64,
    signature_schema: $build.signature_schema,
    artifact: $artifact,
    artifact_bytes: $build.artifact_bytes,
    artifact_sha256: $artifact_sha256,
    raw_words_stored: $build.raw_words_stored,
    corpus: {
      source: "Tatoeba Russian per-language sentence export",
      source_url: $source_url,
      license: $source_license,
      license_url: $source_license_url,
      archive_sha256: $archive_sha256,
      extracted_sha256: $corpus_sha256,
      relation_teacher: $relation_teacher,
      relation_teacher_sha256: $relation_teacher_sha256,
      surface_evidence_sha256: $surface_evidence_sha256,
      committed: false,
      fragments: ($build.heldout.train_fragments + $build.heldout.heldout_fragments),
      support_fragments: $build.heldout.train_fragments,
      heldout_fragments: $build.heldout.heldout_fragments
    },
    transitions: $build.transitions,
    semantic_states: $build.semantic_states,
    candidate_profiles: $build.candidate_profiles,
    pair_profiles: $build.pair_profiles,
    pair_centers: $build.pair_centers,
    positive_centers: $build.positive_centers,
    anti_centers: $build.anti_centers,
    min_profile_support: $build.min_profile_support,
    competition_threshold_micro: $build.competition_threshold_micro,
    heldout: $build.heldout
  }' > "$MANIFEST"

echo "l3_context_phase: artifact=$OUT manifest=$MANIFEST"
jq '{artifact_bytes, corpus, transitions, semantic_states, candidate_profiles, pair_profiles, pair_centers, positive_centers, anti_centers, l2_lattice_negative_examples, heldout: {verdict: .heldout.verdict, support_precision_ppm: .heldout.support_precision_ppm, support_coverage_ppm: .heldout.support_coverage_ppm, min_support_coverage_ppm: .heldout.min_support_coverage_ppm, full_false_top1: .heldout.full_false_top1, pairwise_false_top1_reduction: .heldout.pairwise_false_top1_reduction, candidate_permutation_mismatches: .heldout.candidate_permutation_mismatches, anti_false_top1_reduction: .heldout.anti_false_top1_reduction, phase_ablation_drop: .heldout.phase_ablation_drop, semantic_ablation_drop: .heldout.semantic_ablation_drop}}' "$MANIFEST"
