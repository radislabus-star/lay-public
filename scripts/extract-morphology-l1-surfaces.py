#!/usr/bin/env python3
"""Extract deterministic L1.1 lexical surfaces from a morphology teacher TSV."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile
import unicodedata


PACKAGE_ID = "LAY-RU-LEXICON-462K-SHADOW-v1"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--min-length", type=int, default=2)
    parser.add_argument("--max-length", type=int, default=32)
    return parser.parse_args()


def normalize_surface(raw: str, minimum: int, maximum: int) -> str | None:
    surface = unicodedata.normalize("NFC", raw.strip()).lower().replace("’", "'")
    if not minimum <= len(surface) <= maximum:
        return None
    if not all(character.isalpha() or character in "-'" for character in surface):
        return None
    if not any("\u0400" <= character <= "\u052f" for character in surface):
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
    surfaces: set[str] = set()
    form_rows = 0
    malformed_rows = 0
    with args.input.open(encoding="utf-8") as source:
        for line_number, line in enumerate(source, start=1):
            if not line.strip() or line.startswith("#"):
                continue
            columns = line.rstrip("\n").split("\t")
            if columns[0] != "F":
                continue
            if len(columns) != 4:
                malformed_rows += 1
                continue
            form_rows += 1
            surface = normalize_surface(columns[2], args.min_length, args.max_length)
            if surface is not None:
                surfaces.add(surface)
    if malformed_rows:
        raise SystemExit(f"malformed F rows: {malformed_rows}")
    ordered = sorted(surfaces)
    byte_count, checksum = atomic_write_lines(args.output, ordered)
    manifest = {
        "package_id": PACKAGE_ID,
        "layer": "L1.1 lexical source",
        "status": "SOURCE_READY",
        "runtime_authority": False,
        "source_package_id": "LAY-RU-NOUN-MORPH-462K-SHADOW-v1",
        "source_artifact": str(args.input),
        "corpus_artifact": str(args.output),
        "corpus_bytes": byte_count,
        "corpus_sha256": checksum,
        "source_form_rows": form_rows,
        "unique_surfaces": len(ordered),
        "minimum_length": args.min_length,
        "maximum_length": args.max_length,
        "ordering": "unicode_lexicographic",
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
