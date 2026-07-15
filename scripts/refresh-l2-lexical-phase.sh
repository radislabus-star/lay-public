#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo run --quiet --release --features lexical-compiler \
  --bin lay-nanda-wave-train -- \
  --compile-lexical-phase \
  --out data/lexicon/l2_lexical_phase_v2.bin \
  --include-hunspell \
  data/lexicon/common_ru.txt \
  data/lexicon/l2_surface_foundation_ru_100k.txt \
  data/lexicon/l2_surface_hot_ru.txt \
  data/lexicon/common_en_technical.txt
