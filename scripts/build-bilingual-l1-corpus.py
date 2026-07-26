#!/usr/bin/env python3
"""Combine canonical RU and EN lexical sources into one stable L1.1 corpus."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile


PACKAGE_ID = "LAY-L1.1-RU462K-EN300K-SHADOW-v1"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ru", type=Path, required=True)
    parser.add_argument("--ru-manifest", type=Path, required=True)
    parser.add_argument("--en", type=Path, required=True)
    parser.add_argument("--en-manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    return parser.parse_args()


def load_manifest(path: Path, expected_artifact: Path) -> dict:
    manifest = json.loads(path.read_text(encoding="utf-8"))
    artifact = Path(manifest["corpus_artifact"])
    if artifact != expected_artifact:
        raise SystemExit(
            f"manifest {path} names {artifact}, expected {expected_artifact}"
        )
    checksum = hashlib.sha256(expected_artifact.read_bytes()).hexdigest()
    if checksum != manifest["corpus_sha256"]:
        raise SystemExit(f"checksum mismatch for {expected_artifact}")
    return manifest


def append_source(
    source: Path,
    output,
    digest,
    seen: set[str],
) -> tuple[int, int]:
    accepted = 0
    duplicates = 0
    with source.open(encoding="utf-8") as rows:
        for line_number, line in enumerate(rows, start=1):
            surface = line.rstrip("\n")
            if not surface or surface != surface.strip() or "\t" in surface:
                raise SystemExit(f"invalid lexical surface at {source}:{line_number}")
            if surface in seen:
                duplicates += 1
                continue
            seen.add(surface)
            encoded = f"{surface}\n".encode()
            output.write(encoded)
            digest.update(encoded)
            accepted += 1
    return accepted, duplicates


def main() -> None:
    args = parse_args()
    ru_manifest = load_manifest(args.ru_manifest, args.ru)
    en_manifest = load_manifest(args.en_manifest, args.en)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256()
    seen: set[str] = set()
    with tempfile.NamedTemporaryFile(
        "wb", dir=args.output.parent, prefix=f".{args.output.name}.", delete=False
    ) as temporary:
        temporary_path = Path(temporary.name)
        ru_count, ru_duplicates = append_source(args.ru, temporary, digest, seen)
        en_count, en_duplicates = append_source(args.en, temporary, digest, seen)
    os.replace(temporary_path, args.output)
    manifest = {
        "package_id": PACKAGE_ID,
        "layer": "L1.1 bilingual lexical source",
        "status": "SOURCE_READY",
        "runtime_authority": False,
        "corpus_artifact": str(args.output),
        "corpus_bytes": args.output.stat().st_size,
        "corpus_sha256": digest.hexdigest(),
        "unique_surfaces": len(seen),
        "ru_surfaces": ru_count,
        "en_surfaces": en_count,
        "cross_source_duplicates": ru_duplicates + en_duplicates,
        "ru_terminal_range": [0, ru_count],
        "en_terminal_range": [ru_count, ru_count + en_count],
        "source_packages": [
            ru_manifest["package_id"],
            en_manifest["package_id"],
        ],
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
