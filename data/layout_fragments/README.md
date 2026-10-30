# Optional layout fragment model

`model.bin` is independently trained for Lay. It is not the proprietary model
from the supplied Caramba reverse-engineering report. The report informs the
search/scoring method, not training labels or literal word exceptions.

The immutable source and trainer hashes, settings, sizes and surface-heldout
counts are in `model.json`. The Russian source contains noun forms; the English
source includes rare words and abbreviations. These are lexical-surface proofs,
not unseen-lemma, sentence-quality or physical-client acceptance claims.

Training uses real-language word-presence counts against the opposite language's
keyboard projections, balanced at 120,000 training surfaces per language. The
heldout partition is SHA-256 based and fixed before feature mining. Features
are internal 2–4-character fragments, up to 5-character boundary fragments,
and closed short surfaces. Smoothed log-ratios are quantized to signed weights;
supported unopposed start fragments produce a known-prefix record. Runtime
counts overlapping occurrences, not training-presence counts. Every byte offset
is searched, and shared records never collapse separate occurrences.

The model contains a bounds-checked sorted-edge byte trie and shared records.
It is embedded in binaries; no network, Python package, corpus scan or training
is needed while typing. The existing candidate/DecisionCore/safety/output path
retains authority. The menu switch is OFF by default. Existing short English
target context requirements remain in the automatic adapter.

## Reproduction

Run remotely under the project's resource guard:

```sh
python3 scripts/train-layout-fragments.py --output-dir /path/to/fresh/output \
  --proof-path /path/to/fresh/output/heldout.jsonl
```

`examples/fragment_layout_proof.rs` verifies both interpretations against that
proof, measures scorer latency and optionally compares both modes through the
common DecisionCore. Its hermetic proof must use fresh HOME/XDG directories.

## Data attribution

The binary/model data are offered under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
Lay's source code retains its own repository license.

The existing English lexical source derives from **wordfreq, Robyn Speer**.
Credit also belongs to Marc Brysbaert and the SUBTLEX authors; SUBTLEX is freely
available data. wordfreq also credits Google Books Ngrams, the Leeds Internet
Corpus, Wikipedia, ParaCrawl and OPUS/OpenSubtitles. See its authoritative
[data notices](https://github.com/rspeer/wordfreq/blob/master/NOTICE.md).

The existing Russian noun-form source was generated with pymorphy3 as a cold
teacher, based on the [OpenCorpora dictionary](https://opencorpora.org/?page=downloads),
whose project identifies its data as CC BY-SA 3.0. OpenCorpora and dictionary
contributors retain credit. No proprietary model, user text or personal
learning database is included in this artifact.
