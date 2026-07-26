#!/usr/bin/env python3
"""Build a deterministic frequency-ranked English L1.1 lexical source."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile
import unicodedata

from wordfreq import top_n_list


PACKAGE_ID = "LAY-EN-LEXICON-300K-SHADOW-v1"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--target-words", type=int, default=300_000)
    parser.add_argument("--source-limit", type=int, default=500_000)
    parser.add_argument("--min-length", type=int, default=2)
    parser.add_argument("--max-length", type=int, default=32)
    return parser.parse_args()


def normalize_surface(raw: str, minimum: int, maximum: int) -> str | None:
    surface = unicodedata.normalize("NFC", raw.strip()).lower().replace("’", "'")
    if not minimum <= len(surface) <= maximum:
        return None
    if not all(
        character.isascii() and (character.isalpha() or character in "-'")
        for character in surface
    ):
        return None
    if not any(character.isalpha() for character in surface):
        return None
    return surface


def atomic_write_lines(path: Path, lines: list[str]) -> tuple[int, str]:
    path.parent.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256()
    byte_count = 0
    with tempfile.NamedTemporaryFile(
        "wb", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as temporary:
        temporary_path = Path(temporary.name)
        for line in lines:
            encoded = f"{line}\n".encode()
            temporary.write(encoded)
            digest.update(encoded)
            byte_count += len(encoded)
    os.replace(temporary_path, path)
    return byte_count, digest.hexdigest()


def main() -> None:
    args = parse_args()
    if args.target_words <= 0 or args.source_limit < args.target_words:
        raise SystemExit("source limit must be at least target words")
    selected: list[str] = []
    seen: set[str] = set()
    source = top_n_list("en", args.source_limit)
    for raw in source:
        surface = normalize_surface(raw, args.min_length, args.max_length)
        if surface is None or surface in seen:
            continue
        seen.add(surface)
        selected.append(surface)
        if len(selected) == args.target_words:
            break
    if len(selected) != args.target_words:
        raise SystemExit(
            f"wordfreq yielded {len(selected)} valid surfaces; "
            f"target is {args.target_words}"
        )
    byte_count, checksum = atomic_write_lines(args.output, selected)
    manifest = {
        "package_id": PACKAGE_ID,
        "layer": "L1.1 lexical source",
        "status": "SOURCE_READY",
        "runtime_authority": False,
        "teacher": "wordfreq top_n_list('en')",
        "source_limit": args.source_limit,
        "corpus_artifact": str(args.output),
        "corpus_bytes": byte_count,
        "corpus_sha256": checksum,
        "unique_surfaces": len(selected),
        "minimum_length": args.min_length,
        "maximum_length": args.max_length,
        "ordering": "wordfreq_frequency_descending",
        "l1_1_crystallized": False,
    }
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    args.manifest.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(manifest, ensure_ascii=False))


if __name__ == "__main__":
    main()
