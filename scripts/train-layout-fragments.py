#!/usr/bin/env python3
"""Build the optional fragment scorer from pinned, non-personal lexical data.

No runtime training, downloads, example-specific weights or external packages.
Training counts word presence; inference counts every overlapping occurrence.
"""
from __future__ import annotations

import argparse
import ast
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import re
import struct

SOURCE = "data/lexical_grokking/lay_l11_ru462k_en300k_shadow_v1.txt"
SOURCE_SHA = "449386fa3b9453423792329ecab42946d490bbfe5f863ab194c2d98fb3bd2584"
SALT = b"lay-layout-fragments-v1\0"
MAGIC = b"LAYFRG01"
NONE = 65535


def keyboard_pairs(root: Path) -> dict[str, str]:
    text = (root / "src/dict.rs").read_text().split("const PAIRS:", 1)[1]
    text = text.split("const SHIFT_PAIRS:", 1)[0]
    pairs = {}
    for left, right in re.findall(r"\('((?:\\.|[^'])*)', '((?:\\.|[^'])*)'\)", text):
        pairs[ast.literal_eval("'" + left + "'")] = ast.literal_eval("'" + right + "'")
    if len(pairs) < 40:
        raise ValueError("keyboard mapping source was not understood")
    return pairs


def features(word: str) -> set[bytes]:
    keys = set()
    for n in range(2, 5):
        keys.update(word[i:i + n].encode() for i in range(len(word) - n + 1))
    for n in range(1, min(len(word), 5) + 1):
        keys.add(b"\x02" + word[:n].encode())
        keys.add(word[-n:].encode() + b"\x03")
    if len(word) <= 5:
        keys.add(b"\x02" + word.encode() + b"\x03")
    return keys


def fingerprint(word: str) -> bytes:
    return hashlib.sha256(SALT + word.encode()).digest()


def load_partition(root: Path, training_limit: int, proof_limit: int) -> tuple[list, list]:
    raw = (root / SOURCE).read_bytes()
    if hashlib.sha256(raw).hexdigest() != SOURCE_SHA:
        raise ValueError("lexical source SHA-256 mismatch")
    training, proof = [[], []], [[], []]
    for word in raw.decode().splitlines():
        lang = 0 if re.fullmatch("[а-яё]{2,32}", word) else 1
        if lang == 1 and not re.fullmatch("[a-z]{2,32}", word):
            continue
        key = fingerprint(word)
        (proof if key[0] < 26 else training)[lang].append((key, word))
    return ([[w for _, w in sorted(rows)[:training_limit]] for rows in training],
            [[w for _, w in sorted(rows)[:proof_limit]] for rows in proof])


def learn(training: list[list[str]], pairs: dict, budget: int) -> dict:
    reverse = {v: k for k, v in pairs.items()}
    convert = [str.maketrans(reverse), str.maketrans(pairs)]
    positive = [Counter(), Counter()]
    negative = [Counter(), Counter()]
    for lang, words in enumerate(training):
        for word in words:
            positive[lang].update(features(word))
            negative[1 - lang].update(features(word.translate(convert[lang])))
    learned = {}
    for lang in (0, 1):
        p, n = positive[lang], negative[lang]
        options = []
        for key in p.keys() | n.keys():
            support = p[key] + n[key]
            if support < 6:
                continue
            weight = max(-5, min(5, round(2 * math.log2((p[key] + 4) / (n[key] + 4)))))
            if not weight:
                continue
            known = key.startswith(b"\x02") and p[key] >= 20 and n[key] == 0
            importance = abs(weight) * math.log2(1 + support) * math.sqrt(len(key))
            options.append((importance, key, (0, 1, 0) if known else (weight, 0, 0)))
        for _, key, record in sorted(options, key=lambda row: (-row[0], row[1]))[:budget]:
            if key in learned and learned[key] != record:
                raise ValueError("cross-language fragment record collision")
            learned[key] = record
    return learned


def encode(learned: dict) -> tuple[bytes, dict, list]:
    # Sorted byte edges, 8-byte nodes, shared 4-byte records; no runtime heap graph.
    nodes = [{"edges": {}, "record": NONE}]
    records = sorted(set(learned.values()))
    ids = {record: i for i, record in enumerate(records)}
    for key, record in sorted(learned.items()):
        node = 0
        for byte in key:
            edges = nodes[node]["edges"]
            if byte not in edges:
                edges[byte] = len(nodes)
                nodes.append({"edges": {}, "record": NONE})
            node = edges[byte]
        nodes[node]["record"] = ids[record]
    edge_count = sum(len(node["edges"]) for node in nodes)
    if len(records) >= NONE:
        raise ValueError("too many shared records")
    max_key = max(map(len, learned))
    out = bytearray(struct.pack("<8sIIII", MAGIC, len(nodes), edge_count, len(records), max_key))
    first = 0
    for node in nodes:
        out.extend(struct.pack("<IHH", first, len(node["edges"]), node["record"]))
        first += len(node["edges"])
    for node in nodes:
        for byte, target in sorted(node["edges"].items()):
            out.extend(struct.pack("<BI", byte, target))
    for weight, flags, pair in records:
        out.extend(struct.pack("<bBH", weight, flags, pair))
    return bytes(out), {"nodes": len(nodes), "edges": edge_count,
                        "records": len(records), "max_key_bytes": max_key}, (nodes, records)


def score(word: str, graph: tuple) -> tuple[int, int]:
    nodes, records = graph
    data = b"\x02" + word.lower().encode() + b"\x03"
    total, status = 0, 1
    for start in range(len(data)):
        node = 0
        for byte in data[start:]:
            child = nodes[node]["edges"].get(byte)
            if child is None:
                break
            node = child
            rec = nodes[node]["record"]
            if rec == NONE:
                continue
            weight, flags, _pair = records[rec]
            total += weight
            if weight == 0:
                status = max(status, 2 if flags & 1 else 3)
    return total, status


def select(russian: tuple, english: tuple) -> int | None:
    a, sa = russian
    b, sb = english
    if 3 in (sa, sb) or 0 in (sa, sb):
        return None  # External meaning of the report's special state is unproven.
    if sb == 2:
        return 1
    if sa == 2:
        return 0
    if a == b == 0:
        return None
    return 0 if a >= b else 1


def prove(proof: list, pairs: dict, graph: tuple, output: Path) -> dict:
    reverse = {v: k for k, v in pairs.items()}
    translate = [str.maketrans(reverse), str.maketrans(pairs)]
    results = {}
    with output.open("w") as stream:
        for lang, words in enumerate(proof):
            counts = {"total": len(words), "correct": 0, "wrong": 0, "abstain": 0}
            for word in words:
                other = word.translate(translate[lang])
                ru, en = (word, other) if lang == 0 else (other, word)
                sr, se = score(ru, graph), score(en, graph)
                winner = select(sr, se)
                counts["abstain" if winner is None else "correct" if winner == lang else "wrong"] += 1
                stream.write(json.dumps({"language": "ru" if lang == 0 else "en", "word": word,
                                         "projected": other, "ru_score": sr, "en_score": se,
                                         "winner": winner}, ensure_ascii=False) + "\n")
            counts["correct_percent"] = round(100 * counts["correct"] / len(words), 4)
            counts["clean_preservation_percent"] = round(100 * (len(words) - counts["wrong"]) / len(words), 4)
            results["ru" if lang == 0 else "en"] = counts
    return results


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--proof-path", type=Path, required=True)
    parser.add_argument("--training-limit", type=int, default=120000)
    parser.add_argument("--proof-limit", type=int, default=10000)
    parser.add_argument("--features-per-language", type=int, default=20000)
    args = parser.parse_args()
    if min(args.training_limit, args.proof_limit, args.features_per_language) <= 0:
        parser.error("limits must be positive")
    pairs = keyboard_pairs(args.root)
    training, proof = load_partition(args.root, args.training_limit, args.proof_limit)
    learned = learn(training, pairs, args.features_per_language)
    package, stats, graph = encode(learned)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    args.proof_path.parent.mkdir(parents=True, exist_ok=True)
    quality = prove(proof, pairs, graph, args.proof_path)
    manifest = {"schema": "lay.layout-fragments.v1", "method": "overlapping byte fragments, signed sums, flagged selector",
                "training_source": SOURCE, "training_source_sha256": SOURCE_SHA,
                "source_provenance": "existing Lay RU lexical surfaces + wordfreq EN lexical surfaces; independent weights",
                "keyboard_source_sha256": hashlib.sha256((args.root / "src/dict.rs").read_bytes()).hexdigest(),
                "trainer_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "partition": "SHA256(lay-layout-fragments-v1 NUL + lowercase surface); first byte <26 heldout",
                "training_limit_per_language": args.training_limit,
                "training_counts": list(map(len, training)), "proof_limit_per_language": args.proof_limit,
                "features_per_language_budget": args.features_per_language,
                "feature_count": len(learned), **stats, "model_bytes": len(package),
                "model_sha256": hashlib.sha256(package).hexdigest(),
                "heldout_proof_sha256": hashlib.sha256(args.proof_path.read_bytes()).hexdigest(),
                "quality": quality, "scope": "surface holdout, not unseen lemmas or physical-client acceptance",
                "runtime_authority": False, "default_enabled": False}
    (args.output_dir / "model.bin").write_bytes(package)
    (args.output_dir / "model.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"model_bytes": len(package), "features": len(learned), "quality": quality}))


if __name__ == "__main__":
    main()
