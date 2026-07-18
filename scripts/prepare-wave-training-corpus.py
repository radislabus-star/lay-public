#!/usr/bin/env python3
"""Build large cold L2/L3 corpus inputs from clean Tatoeba exports.

The generated text and TSV files are cold build inputs. Runtime packages must
still be compiled by the Rust trainers, which store phase centers and hashes
instead of corpus sentences or a raw word table.
"""

from __future__ import annotations

import argparse
import bz2
from collections import Counter, defaultdict
from concurrent.futures import ProcessPoolExecutor
import hashlib
import itertools
import json
import math
import os
from pathlib import Path
import re
import shutil
import tempfile
import unicodedata


RU_WORD = re.compile(r"[а-яё](?:[а-яё-]*[а-яё])?", re.IGNORECASE)
EN_WORD = re.compile(r"[a-z](?:[a-z'-]*[a-z])?", re.IGNORECASE)
RU_ALPHABET = "абвгдеёжзийклмнопрстуфхцчшщъыьэюя"
EN_ALPHABET = "abcdefghijklmnopqrstuvwxyz"
EN_KEYS = "qwertyuiop[]asdfghjkl;'zxcvbnm,."
RU_KEYS = "йцукенгшщзхъфывапролджэячсмитьбю"
LAYOUT_TABLE = str.maketrans(EN_KEYS + RU_KEYS, RU_KEYS + EN_KEYS)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ru-archive", type=Path, required=True)
    parser.add_argument("--en-archive", type=Path, required=True)
    parser.add_argument("--ru-lexicon", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--sentences-per-language", type=int, default=500_000)
    parser.add_argument("--words-per-language", type=int, default=100_000)
    parser.add_argument("--variants-per-word", type=int, default=3)
    parser.add_argument("--jobs", type=int, default=os.cpu_count() or 1)
    return parser.parse_args()


def normalize_sentence(raw: str, language: str) -> str | None:
    sentence = " ".join(unicodedata.normalize("NFKC", raw).split())
    if not 4 <= len(sentence) <= 400:
        return None
    tokens = sentence.split()
    if not 2 <= len(tokens) <= 64:
        return None
    letters = RU_WORD if language == "ru" else EN_WORD
    if len(letters.findall(sentence.lower())) < 2:
        return None
    return sentence


def sentence_words(sentence: str, language: str) -> list[str]:
    pattern = RU_WORD if language == "ru" else EN_WORD
    return [
        word.lower()
        for word in pattern.findall(sentence)
        if 2 <= len(word) <= 32
    ]


def extract_tatoeba(
    archive: str,
    language: str,
    limit: int,
    sentence_out: str,
    frequency_out: str,
) -> dict[str, int]:
    seen: set[str] = set()
    frequencies: Counter[str] = Counter()
    sentence_path = Path(sentence_out)
    with bz2.open(archive, "rt", encoding="utf-8") as source, sentence_path.open(
        "w", encoding="utf-8"
    ) as output:
        for row in source:
            fields = row.rstrip("\n").split("\t", 2)
            if len(fields) != 3:
                continue
            sentence = normalize_sentence(fields[2], language)
            if sentence is None or sentence in seen:
                continue
            seen.add(sentence)
            output.write(sentence + "\n")
            frequencies.update(sentence_words(sentence, language))
            if len(seen) >= limit:
                break
    with Path(frequency_out).open("w", encoding="utf-8") as output:
        for word, count in frequencies.items():
            output.write(f"{word}\t{count}\n")
    return {"sentences": len(seen), "words": len(frequencies)}


def load_frequencies(path: Path) -> Counter[str]:
    result: Counter[str] = Counter()
    with path.open(encoding="utf-8") as source:
        for line in source:
            word, count = line.rstrip("\n").split("\t", 1)
            result[word] += int(count)
    return result


def load_reference_words(path: Path) -> list[str]:
    return [
        line.strip().lower()
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    ]


def select_words(
    frequencies: Counter[str], references: list[str], language: str, limit: int
) -> list[str]:
    pattern = RU_WORD if language == "ru" else EN_WORD
    for rank, word in enumerate(references):
        # Reference order is a weak prior; corpus frequency remains dominant.
        frequencies[word] += max(1, min(100, len(references) - rank) // 1000)
    words = [
        word
        for word in frequencies
        if 4 <= len(word) <= 28 and pattern.fullmatch(word)
    ]
    words.sort(key=lambda word: (-frequencies[word], word))
    return words[:limit]


def interleave_files(left: Path, right: Path, output: Path) -> int:
    count = 0
    with left.open(encoding="utf-8") as left_rows, right.open(
        encoding="utf-8"
    ) as right_rows, output.open("w", encoding="utf-8") as out:
        for left_row, right_row in itertools.zip_longest(left_rows, right_rows):
            if left_row is not None:
                out.write(left_row)
                count += 1
            if right_row is not None:
                out.write(right_row)
                count += 1
    return count


def corpus_support_repetitions(count: int) -> int:
    # Log support is the cold Bayes prior consumed by the phase compiler.
    return 1 + min(7, int(math.log2(max(1, count))))


def write_weighted_lexicon(
    path: Path,
    ru_words: list[str],
    en_words: list[str],
    ru_frequencies: Counter[str],
    en_frequencies: Counter[str],
) -> int:
    rows = 0
    with path.open("w", encoding="utf-8") as output:
        for words, frequencies in (
            (ru_words, ru_frequencies),
            (en_words, en_frequencies),
        ):
            for word in words:
                repetitions = corpus_support_repetitions(frequencies[word])
                output.write((word + "\n") * repetitions)
                rows += repetitions
    return rows


def stable_index(word: str, modulo: int) -> int:
    digest = hashlib.blake2b(word.encode("utf-8"), digest_size=8).digest()
    return int.from_bytes(digest, "little") % modulo


def internal_index(word: str, salt: int) -> int:
    width = max(1, len(word) - 2)
    return 1 + ((stable_index(word, 1 << 31) + salt) % width)


def corruption_variants(word: str) -> list[tuple[str, str]]:
    variants: list[tuple[str, str]] = []
    projected = word.translate(LAYOUT_TABLE)
    if projected != word:
        variants.append(("layout", projected))

    if len(word) >= 4:
        index = min(internal_index(word, 1), len(word) - 2)
        if word[index] != word[index + 1]:
            swapped = word[:index] + word[index + 1] + word[index] + word[index + 2 :]
            variants.append(("adjacent_transposition", swapped))

        index = internal_index(word, 3)
        variants.append(("missing_letter", word[:index] + word[index + 1 :]))

        index = internal_index(word, 5)
        variants.append(("repeated_letter", word[:index] + word[index] + word[index:]))

        alphabet = RU_ALPHABET if RU_WORD.fullmatch(word) else EN_ALPHABET
        index = internal_index(word, 7)
        current = alphabet.find(word[index])
        replacement = alphabet[(max(0, current) + 1) % len(alphabet)]
        variants.append(("letter_substitution", word[:index] + replacement + word[index + 1 :]))

    if len(word) >= 8:
        first = internal_index(word, 11)
        second = internal_index(word, 17)
        if first == second or abs(first - second) == 1:
            second = min(len(word) - 2, first + 2)
        omitted = "".join(
            char for index, char in enumerate(word) if index not in {first, second}
        )
        if omitted != word:
            variants.append(("composite_typo", omitted))
    return variants


def build_distractors(words: list[str], language: str) -> list[str]:
    buckets: dict[tuple[int, str], list[str]] = defaultdict(list)
    for word in words:
        buckets[(len(word), word[0])].append(word)
    nearby: dict[str, str] = {}
    for bucket in buckets.values():
        if len(bucket) < 2:
            continue
        for index, word in enumerate(bucket):
            nearby[word] = bucket[(index + 1) % len(bucket)]
    distractors: list[str] = []
    for word in words:
        if word in nearby:
            distractors.append(nearby[word])
        else:
            # A different stable word is better negative evidence than a fake token.
            distractors.append(words[(stable_index(language + word, len(words) - 1) + 1) % len(words)])
    return distractors


def write_l2_shard(
    shard_index: int,
    rows: list[tuple[str, str, str]],
    output_path: str,
    variants_per_word: int,
) -> tuple[int, int]:
    groups = 0
    written = 0
    with Path(output_path).open("w", encoding="utf-8") as output:
        for language, word, distractor in rows:
            variants = corruption_variants(word)
            if not variants:
                continue
            offset = stable_index(word, len(variants))
            chosen = [variants[(offset + step) % len(variants)] for step in range(min(variants_per_word, len(variants)))]
            for variant_index, (operation, corrupted) in enumerate(chosen):
                if corrupted == word or corrupted == distractor:
                    continue
                group = f"large:{language}:{shard_index:02d}:{groups:07d}:{variant_index}"
                output.write(
                    f"{group}\t\t{corrupted}\t{word}\t{operation}\t1\tclean-corpus\tpositive-center\n"
                )
                output.write(
                    f"{group}\t\t{corrupted}\t{distractor}\t{operation}\t0\tclean-corpus\tlexical-anti-center\n"
                )
                groups += 1
                written += 2
    return groups, written


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> None:
    args = parse_args()
    args.out_dir.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="lay-wave-corpus-", dir=args.out_dir))
    try:
        extraction_jobs = []
        with ProcessPoolExecutor(max_workers=2) as executor:
            for language, archive in (("ru", args.ru_archive), ("en", args.en_archive)):
                extraction_jobs.append(
                    (
                        language,
                        executor.submit(
                            extract_tatoeba,
                            str(archive),
                            language,
                            args.sentences_per_language,
                            str(work / f"{language}.sentences"),
                            str(work / f"{language}.frequencies"),
                        ),
                    )
                )
            extraction = {language: future.result() for language, future in extraction_jobs}

        ru_frequencies = load_frequencies(work / "ru.frequencies")
        en_frequencies = load_frequencies(work / "en.frequencies")
        ru_words = select_words(
            ru_frequencies,
            load_reference_words(args.ru_lexicon),
            "ru",
            args.words_per_language,
        )
        en_words = select_words(en_frequencies, [], "en", args.words_per_language)

        l3_corpus = args.out_dir / "l3_context_large.txt"
        l3_rows = interleave_files(
            work / "ru.sentences", work / "en.sentences", l3_corpus
        )
        lexicon = args.out_dir / "lexical_words.txt"
        lexicon.write_text("\n".join(ru_words + en_words) + "\n", encoding="utf-8")
        weighted_lexicon = args.out_dir / "lexical_weighted_words.txt"
        weighted_lexicon_rows = write_weighted_lexicon(
            weighted_lexicon,
            ru_words,
            en_words,
            ru_frequencies,
            en_frequencies,
        )

        combined = [("ru", word) for word in ru_words] + [("en", word) for word in en_words]
        distractors = build_distractors(ru_words, "ru") + build_distractors(en_words, "en")
        jobs = max(1, min(args.jobs, len(combined)))
        shards: list[list[tuple[str, str, str]]] = [[] for _ in range(jobs)]
        for index, ((language, word), distractor) in enumerate(zip(combined, distractors)):
            shards[index % jobs].append((language, word, distractor))

        shard_dir = work / "l2-shards"
        shard_dir.mkdir()
        with ProcessPoolExecutor(max_workers=jobs) as executor:
            futures = [
                executor.submit(
                    write_l2_shard,
                    index,
                    rows,
                    str(shard_dir / f"{index:02d}.tsv"),
                    args.variants_per_word,
                )
                for index, rows in enumerate(shards)
            ]
            shard_reports = [future.result() for future in futures]

        # Corruptions measure lexical recovery; operator memory must not absorb word identity.
        l2_dataset = args.out_dir / "l2_corruption_eval.tsv"
        with l2_dataset.open("wb") as output:
            output.write(
                b"group_id\tcontext\toriginal\tcandidate\toperation\tlabel\tsource\treason\n"
            )
            for index in range(jobs):
                with (shard_dir / f"{index:02d}.tsv").open("rb") as source:
                    shutil.copyfileobj(source, output)

        manifest = {
            "schema": "lay.wave-training-corpus.v1",
            "sources": {
                "ru": "Tatoeba Russian per-language export, CC BY 2.0 FR",
                "en": "Tatoeba English per-language export, CC BY 2.0 FR",
            },
            "jobs": jobs,
            "extraction": extraction,
            "l3_sentences": l3_rows,
            "ru_words": len(ru_words),
            "en_words": len(en_words),
            "l2_groups": sum(report[0] for report in shard_reports),
            "l2_rows": sum(report[1] for report in shard_reports),
            "artifacts": {
                "l3_context": {"path": str(l3_corpus), "sha256": sha256(l3_corpus)},
                "lexical_words": {"path": str(lexicon), "sha256": sha256(lexicon)},
                "lexical_weighted_words": {
                    "path": str(weighted_lexicon),
                    "sha256": sha256(weighted_lexicon),
                    "rows": weighted_lexicon_rows,
                },
                "l2_corruption_eval": {"path": str(l2_dataset), "sha256": sha256(l2_dataset)},
            },
            "runtime_contract": {
                "raw_corpus_is_cold_only": True,
                "corruption_rows_are_eval_only": True,
                "runtime_package_must_store_raw_words": False,
            },
        }
        manifest_path = args.out_dir / "wave_training_corpus.manifest.json"
        manifest_path.write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        print(json.dumps(manifest, ensure_ascii=False, indent=2))
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
