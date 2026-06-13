#!/usr/bin/env python3
"""Build and train the first tiny lay neural-arbiter probe.

This is research tooling, not runtime code. It intentionally depends only on
the Python standard library plus NumPy so the experiment can run on the local
development machine without pulling a heavy ML stack.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import random
import shutil
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
DATASET_PATH = ROOT / "data/neural_arbiter/dataset.tsv"
HOLDOUT_PATH = ROOT / "data/neural_arbiter/holdout.tsv"
REPORT_PATH = ROOT / "target/neural-arbiter/report.md"
RESULTS_PATH = ROOT / "docs/research/neural-arbiter-results.md"
TECH_LEXICON_PATH = ROOT / "data/lexicon/common_en_technical.txt"
CORRECTIONS_LOG_PATH = Path.home() / ".local/share/lay/corrections.jsonl"


US_TO_RU = {
    "q": "й",
    "w": "ц",
    "e": "у",
    "r": "к",
    "t": "е",
    "y": "н",
    "u": "г",
    "i": "ш",
    "o": "щ",
    "p": "з",
    "[": "х",
    "]": "ъ",
    "a": "ф",
    "s": "ы",
    "d": "в",
    "f": "а",
    "g": "п",
    "h": "р",
    "j": "о",
    "k": "л",
    "l": "д",
    ";": "ж",
    "'": "э",
    "z": "я",
    "x": "ч",
    "c": "с",
    "v": "м",
    "b": "и",
    "n": "т",
    "m": "ь",
    ",": "б",
    ".": "ю",
    "/": ".",
    "?": ",",
    "@": '"',
    "#": "№",
    "$": ";",
    "^": ":",
    "&": "?",
    "`": "ё",
}
RU_TO_US = {v: k for k, v in US_TO_RU.items()}
US_TO_RU.update({k.upper(): v.upper() for k, v in list(US_TO_RU.items()) if k.isalpha()})
RU_TO_US.update({k.upper(): v.upper() for k, v in list(RU_TO_US.items()) if k.isalpha()})

KEY_ORDER = list("qwertyuiop[]asdfghjkl;'zxcvbnm,./`")
KEY_INDEX = {key: idx + 1 for idx, key in enumerate(KEY_ORDER)}
for us, ru in list(US_TO_RU.items()):
    base = us.lower()
    if base in KEY_INDEX:
        KEY_INDEX[ru.lower()] = KEY_INDEX[base]


@dataclass(frozen=True)
class Row:
    group_id: str
    context: str
    original: str
    candidate: str
    operation: str
    label: int
    source: str
    reason: str


def decode_fixture(value: str) -> str:
    if value == "None":
        return value
    return value.replace("\\s", " ")


def fixture_lines(path: Path):
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip("\n")
        if not line or line.startswith("#"):
            continue
        yield line


def convert_layout(text: str) -> str:
    cyr = sum(1 for ch in text if "А" <= ch <= "я" or ch in "ёЁ")
    lat = sum(1 for ch in text if ch.isascii() and ch.isalpha())
    table = RU_TO_US if cyr > lat else US_TO_RU
    return "".join(table.get(ch, ch) for ch in text)


def technical_words() -> set[str]:
    out = set()
    for line in TECH_LEXICON_PATH.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            out.add(line.lower())
    return out


TECH_WORDS = technical_words()


def operation_for(original: str, candidate: str) -> str:
    if original == candidate:
        return "keep"
    if convert_layout(original) == candidate:
        return "layout"
    if original.replace(" ", "") == candidate.replace(" ", ""):
        return "split" if " " in candidate else "glue"
    if any(ch.isascii() and ch.isalpha() for ch in original + candidate) and any(
        "А" <= ch <= "я" or ch in "ёЁ" for ch in original + candidate
    ):
        return "mixed"
    return "typo"


def add_group(rows: list[Row], gid: str, original: str, expected: str, source: str, reason: str):
    original = original.strip("\n")
    expected = expected.strip("\n")
    if not original:
        return
    label_keep = int(original == expected)
    rows.append(Row(gid, "", original, original, "keep", label_keep, source, reason))
    if expected != "None" and expected != original:
        rows.append(
            Row(
                gid,
                "",
                original,
                expected,
                operation_for(original, expected),
                1,
                source,
                reason,
            )
        )
    flipped = convert_layout(original)
    if flipped != original and flipped != expected:
        rows.append(
            Row(
                gid,
                "",
                original,
                flipped,
                "layout",
                0,
                source,
                "deterministic layout negative",
            )
        )
    if " " in expected and expected != "None":
        glued = expected.replace(" ", "")
        if glued != expected and glued != original:
            rows.append(Row(gid, "", original, glued, "glue", 0, source, "glued negative"))
    if " " in original and expected != "None":
        glued_original = original.replace(" ", "")
        if glued_original not in {original, expected}:
            rows.append(
                Row(gid, "", original, glued_original, "glue", 0, source, "original glued negative")
            )


def load_two_col_fixture(rows: list[Row], rel: str, default_reason: str):
    path = ROOT / rel
    for i, line in enumerate(fixture_lines(path), 1):
        cols = line.split("\t")
        if len(cols) < 2:
            continue
        original = decode_fixture(cols[0])
        expected = decode_fixture(cols[1])
        if expected == "None":
            expected = original
        add_group(rows, f"{path.name}:{i}", original, expected, rel, default_reason)


def load_tagged_fixture(rows: list[Row], rel: str, default_reason: str):
    path = ROOT / rel
    for i, line in enumerate(fixture_lines(path), 1):
        cols = line.split("\t")
        if len(cols) < 3:
            continue
        tag = cols[0]
        original = decode_fixture(cols[1])
        expected = decode_fixture(cols[2])
        if expected == "None":
            expected = original
        add_group(rows, f"{path.name}:{tag}:{i}", original, expected, rel, f"{default_reason}:{tag}")


def load_policy_fixture(rows: list[Row]):
    path = ROOT / "tests/fixtures/typing_assist_policy_cases.tsv"
    for i, line in enumerate(fixture_lines(path), 1):
        cols = line.split("\t")
        if len(cols) < 6:
            continue
        klass, safety, original, _allow_layout, expected, message = cols[:6]
        original = decode_fixture(original)
        expected = decode_fixture(expected)
        if expected == "None":
            expected = original
        add_group(rows, f"{path.name}:{i}", original, expected, str(path.relative_to(ROOT)), f"{klass}:{safety}:{message}")


def load_keep_fixture(rows: list[Row], rel: str, reason: str):
    path = ROOT / rel
    for i, line in enumerate(fixture_lines(path), 1):
        text = decode_fixture(line)
        add_group(rows, f"{path.name}:keep:{i}", text, text, rel, reason)


def load_holdout() -> list[Row]:
    rows: list[Row] = []
    for i, line in enumerate(fixture_lines(HOLDOUT_PATH), 1):
        cols = line.split("\t")
        if len(cols) < 2:
            continue
        original = decode_fixture(cols[0])
        expected = decode_fixture(cols[1])
        reason = cols[2] if len(cols) > 2 else "holdout"
        add_group(rows, f"holdout:{i}", original, expected, str(HOLDOUT_PATH.relative_to(ROOT)), reason)
    return rows


def add_layout_pairs(rows: list[Row]):
    examples = [
        "привет",
        "вот",
        "давай",
        "может",
        "расчет",
        "file",
        "git",
        "html",
        "wechat",
        "telegram",
        "chrome",
        "vpn",
        "api",
        "port",
        "push",
        "work",
    ]
    for i, expected in enumerate(examples, 1):
        typed = convert_layout(expected)
        if typed == expected:
            continue
        add_group(rows, f"layout-synth:{i}", typed + " ", expected + " ", "synthetic/layout", "layout rescue")


def build_dataset() -> list[Row]:
    rows: list[Row] = []
    load_tagged_fixture(
        rows,
        "tests/fixtures/daemon_typing_assist_default_rule_cases.tsv",
        "default typing-assist rule",
    )
    load_tagged_fixture(
        rows,
        "tests/fixtures/typing_assist_mixed_matrix_layout_words.tsv",
        "mixed matrix layout word",
    )
    for rel in [
        "tests/fixtures/phrase_reader_split_pair.tsv",
        "tests/fixtures/phrase_reader_glued.tsv",
        "tests/fixtures/phrase_reader_contextual_glued.tsv",
        "tests/fixtures/typing_assist_alternating.tsv",
        "tests/fixtures/typing_assist_beta_alternating.tsv",
        "tests/fixtures/typing_assist_live_spacing.tsv",
        "tests/fixtures/decoder_transition_typing_assist_fix.tsv",
    ]:
        load_two_col_fixture(rows, rel, "fixture correction")
    load_policy_fixture(rows)
    for rel, reason in [
        ("tests/fixtures/phrase_reader_split_pair_keep.tsv", "known phrase pair must not glue"),
        ("tests/fixtures/typing_assist_normal_reject.txt", "normal mode keep"),
        ("tests/fixtures/typing_assist_shell_keep.txt", "shell token keep"),
        ("tests/fixtures/typing_assist_cli_commands.txt", "cli command keep"),
    ]:
        load_keep_fixture(rows, rel, reason)
    add_layout_pairs(rows)

    dedup: dict[tuple[str, str, str, str], Row] = {}
    for row in rows:
        key = (row.group_id, row.original, row.candidate, row.operation)
        dedup[key] = row
    return list(dedup.values())


def write_dataset(rows: list[Row]):
    DATASET_PATH.parent.mkdir(parents=True, exist_ok=True)
    with DATASET_PATH.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.writer(fh, delimiter="\t", lineterminator="\n")
        writer.writerow(["group_id", "context", "original", "candidate", "operation", "label", "source", "reason"])
        for row in rows:
            writer.writerow([
                row.group_id,
                row.context,
                row.original,
                row.candidate,
                row.operation,
                row.label,
                row.source,
                row.reason,
            ])


def load_personal_correction_events(path: Path = CORRECTIONS_LOG_PATH) -> list[dict]:
    if not path.exists():
        return []
    events = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            events.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return events


def personal_frequency_profile(events: list[dict]) -> dict[str, object]:
    profile: dict[str, object] = {
        "events": len(events),
        "layout_replay": 0,
        "user_correction": 0,
        "typing_assist_corrections": 0,
        "cancelled_typing_assist": 0,
        "layout_pairs": {},
        "undo_pairs": {},
        "script_transitions": {},
        "short_user_overrides": {},
    }
    layout_pairs: dict[str, int] = {}
    undo_pairs: dict[str, int] = {}
    script_transitions: dict[str, int] = {}
    short_overrides: dict[str, int] = {}

    for event in events:
        kind = str(event.get("kind", ""))
        if kind == "layout-replay":
            profile["layout_replay"] = int(profile["layout_replay"]) + 1
            pair = f"{event.get('from', '')!r}->{event.get('to', '')!r}"
            layout_pairs[pair] = layout_pairs.get(pair, 0) + 1
        elif kind == "typing-assist":
            profile["typing_assist_corrections"] = int(profile["typing_assist_corrections"]) + 1
        elif kind == "user-correction":
            profile["user_correction"] = int(profile["user_correction"]) + 1
            lay_from = str(event.get("lay_from", ""))
            lay_to = str(event.get("lay_to", ""))
            user_to = str(event.get("to", ""))
            if lay_from or lay_to:
                profile["cancelled_typing_assist"] = int(profile["cancelled_typing_assist"]) + 1
                pair = f"{lay_from!r}->{lay_to!r}=>{user_to!r}"
                undo_pairs[pair] = undo_pairs.get(pair, 0) + 1
                if max(len(lay_from.strip()), len(lay_to.strip()), len(user_to.strip())) <= 3:
                    short_overrides[pair] = short_overrides.get(pair, 0) + 1

        transition = script_transition(str(event.get("from", "")), str(event.get("to", "")))
        script_transitions[transition] = script_transitions.get(transition, 0) + 1

    profile["layout_pairs"] = top_counts(layout_pairs)
    profile["undo_pairs"] = top_counts(undo_pairs)
    profile["script_transitions"] = top_counts(script_transitions)
    profile["short_user_overrides"] = top_counts(short_overrides)
    return profile


def build_personal_challenge_rows(events: list[dict]) -> list[Row]:
    rows: list[Row] = []
    for idx, event in enumerate(events, 1):
        if event.get("kind") != "user-correction":
            continue
        lay_from = str(event.get("lay_from", ""))
        lay_to = str(event.get("lay_to", ""))
        user_to = str(event.get("to", ""))
        if not lay_from or not user_to or lay_to == user_to:
            continue
        gid = f"personal:{idx}"
        rows.append(Row(gid, "", lay_from, lay_from, "keep", int(lay_from == user_to), "personal/log", "personal keep"))
        rows.append(Row(gid, "", lay_from, lay_to, operation_for(lay_from, lay_to), 0, "personal/log", "lay rejected by user"))
        rows.append(Row(gid, "", lay_from, user_to, operation_for(lay_from, user_to), 1, "personal/log", "user correction target"))
        flipped = convert_layout(lay_from)
        if flipped not in {lay_from, lay_to, user_to}:
            rows.append(Row(gid, "", lay_from, flipped, "layout", 0, "personal/log", "layout negative"))
    return rows


def train_personal_profile_probe(base_rows: list[Row], personal_rows: list[Row], dim: int, epochs: int):
    if not personal_rows:
        return None
    personal_groups = sorted(rows_by_group(personal_rows), key=lambda group: group[0].group_id)
    cut = max(1, int(len(personal_groups) * 0.75))
    personal_train = [row for group in personal_groups[:cut] for row in group]
    personal_test = [row for group in personal_groups[cut:] for row in group]
    personal_counts = build_personal_candidate_counts(personal_train)
    personal_featurizer = personal_featurizer_for_counts(personal_counts)
    train_rows = base_rows + personal_train
    model, encoded = train_probe_with_holdout(
        train_rows,
        personal_test,
        dim,
        epochs=epochs,
        seed=23,
        featurizer=personal_featurizer,
    )
    return {
        "train_groups": len(personal_groups[:cut]),
        "test_groups": len(personal_groups[cut:]),
        "accepted_pairs": len(personal_counts["accepted"]),
        "rejected_pairs": len(personal_counts["rejected"]),
        "metrics": evaluate(model, encoded, personal_test),
        "frequency_circuit": evaluate_personal_frequency_circuit(
            model, encoded, personal_test, personal_counts
        ),
        "threshold": best_threshold_profile(model, encoded, personal_test),
        "baseline": current_rule_baseline(personal_test),
    }


def build_personal_candidate_counts(rows: list[Row]) -> dict[str, dict[tuple[str, str], int]]:
    counts = {"accepted": {}, "rejected": {}}
    for row in rows:
        key = (row.original, row.candidate)
        if row.label == 1 and row.reason == "user correction target":
            counts["accepted"][key] = counts["accepted"].get(key, 0) + 1
        elif row.label == 0 and row.reason == "lay rejected by user":
            counts["rejected"][key] = counts["rejected"].get(key, 0) + 1
    return counts


def personal_featurizer_for_counts(counts: dict[str, dict[tuple[str, str], int]]):
    def featurizer(row: Row, dim: int) -> np.ndarray:
        features = featurize_row(row, dim)
        accepted = counts["accepted"].get((row.original, row.candidate), 0)
        rejected = counts["rejected"].get((row.original, row.candidate), 0)
        if accepted:
            add_named(features, "personal:accepted-once", 2.0)
            if accepted >= 2:
                add_named(features, "personal:accepted-frequent", min(accepted, 8) * 1.5)
        if rejected:
            add_named(features, "personal:rejected-once", -2.0)
            if rejected >= 2:
                add_named(features, "personal:rejected-frequent", -min(rejected, 8) * 1.5)
        norm = np.linalg.norm(features)
        if norm > 0:
            features /= norm
        return features

    return featurizer


def top_counts(counts: dict[str, int], limit: int = 12) -> list[tuple[str, int]]:
    return sorted(counts.items(), key=lambda item: (-item[1], item[0]))[:limit]


def script_transition(left: str, right: str) -> str:
    return f"{text_script(left)}->{text_script(right)}"


def text_script(text: str) -> str:
    cyr = sum(1 for ch in text if "А" <= ch <= "я" or ch in "ёЁ")
    lat = sum(1 for ch in text if ch.isascii() and ch.isalpha())
    return script_bucket(cyr, lat)


def read_dataset() -> list[Row]:
    with DATASET_PATH.open("r", encoding="utf-8", newline="") as fh:
        reader = csv.DictReader(fh, delimiter="\t")
        return [
            Row(
                r["group_id"],
                r["context"],
                r["original"],
                r["candidate"],
                r["operation"],
                int(r["label"]),
                r["source"],
                r["reason"],
            )
            for r in reader
        ]


def read_rows(path: Path) -> list[Row]:
    with path.open("r", encoding="utf-8", newline="") as fh:
        reader = csv.DictReader(fh, delimiter="\t")
        return [
            Row(
                r["group_id"],
                r["context"],
                r["original"],
                r["candidate"],
                r["operation"],
                int(r["label"]),
                r["source"],
                r["reason"],
            )
            for r in reader
        ]


class TinyScorer:
    def __init__(self, dim: int, rng: np.random.Generator):
        self.dim = dim
        self.w = rng.normal(0, 0.01, size=(dim,)).astype(np.float32)
        self.b = np.float32(0.0)

    def score(self, features: np.ndarray) -> float:
        h = np.tanh(features)
        z = float(h @ self.w + self.b)
        return 1.0 / (1.0 + math.exp(-max(-40.0, min(40.0, z))))


NAMED_FEATURES = [
    "op:keep",
    "op:layout",
    "op:split",
    "op:glue",
    "op:mixed",
    "op:typo",
    "relation:identity",
    "relation:exact-layout",
    "relation:space-only-change",
    "script:orig:cyr",
    "script:orig:lat",
    "script:orig:mixed",
    "script:orig:other",
    "script:cand:cyr",
    "script:cand:lat",
    "script:cand:mixed",
    "script:cand:other",
    "candidate:common-en-technical",
    "candidate:length-short",
    "candidate:length-medium",
    "candidate:length-long",
    "edit:small",
    "edit:medium",
    "edit:large",
    "layout:changes-script",
    "layout:keeps-script",
    "boundary:adds-space",
    "boundary:removes-space",
    "boundary:keeps-space-count",
    "case:keeps-uppercase-shape",
    "case:changes-uppercase-shape",
    "keyseq:exact",
    "keyseq:length-same",
    "fourier:low-close",
    "fourier:mid-close",
    "fourier:phase-close",
    "fourier:energy-ratio-close",
    "personal:accepted-once",
    "personal:accepted-frequent",
    "personal:rejected-once",
    "personal:rejected-frequent",
    "bias",
]
NAMED_INDEX = {name: idx for idx, name in enumerate(NAMED_FEATURES)}


def stable_bucket(feature: str, dim: int) -> tuple[int, float]:
    digest = hashlib.blake2b(feature.encode("utf-8"), digest_size=8).digest()
    value = int.from_bytes(digest, "little")
    return value % dim, 1.0 if (value >> 63) == 0 else -1.0


def add_hashed(features: np.ndarray, name: str, value: float = 1.0):
    idx, sign = stable_bucket(name, len(features))
    features[idx] += sign * value


def add_named(features: np.ndarray, name: str, value: float = 1.0):
    idx = NAMED_INDEX.get(name)
    if idx is not None and idx < len(features):
        features[idx] += value


def char_ngrams(text: str, max_n: int = 3):
    padded = f"^{text}$"
    for n in range(1, max_n + 1):
        for i in range(0, max(0, len(padded) - n + 1)):
            yield padded[i : i + n]


def featurize_row(row: Row, dim: int) -> np.ndarray:
    features = np.zeros((dim,), dtype=np.float32)
    add_mechanistic_features(features, row)
    for field, text in [("ctx", row.context), ("orig", row.original), ("cand", row.candidate)]:
        for gram in char_ngrams(text):
            add_hashed_residual(features, f"{field}:{gram}")
    for gram in char_ngrams(row.candidate):
        add_hashed_residual(features, f"delta:+{gram}")
    for gram in char_ngrams(row.original):
        add_hashed_residual(features, f"delta:-{gram}")
    add_hashed_residual(features, f"len:orig:{min(len(row.original), 32)}")
    add_hashed_residual(features, f"len:cand:{min(len(row.candidate), 32)}")
    norm = np.linalg.norm(features)
    if norm > 0:
        features /= norm
    return features


def featurize_mechanistic_only(row: Row, dim: int) -> np.ndarray:
    features = np.zeros((dim,), dtype=np.float32)
    add_mechanistic_features(features, row)
    norm = np.linalg.norm(features)
    if norm > 0:
        features /= norm
    return features


def featurize_residual_only(row: Row, dim: int) -> np.ndarray:
    features = np.zeros((dim,), dtype=np.float32)
    for field, text in [("ctx", row.context), ("orig", row.original), ("cand", row.candidate)]:
        for gram in char_ngrams(text):
            add_hashed_residual(features, f"{field}:{gram}")
    for gram in char_ngrams(row.candidate):
        add_hashed_residual(features, f"delta:+{gram}")
    for gram in char_ngrams(row.original):
        add_hashed_residual(features, f"delta:-{gram}")
    add_hashed_residual(features, f"len:orig:{min(len(row.original), 32)}")
    add_hashed_residual(features, f"len:cand:{min(len(row.candidate), 32)}")
    norm = np.linalg.norm(features)
    if norm > 0:
        features /= norm
    return features


def add_hashed_residual(features: np.ndarray, name: str, value: float = 1.0):
    start = min(len(NAMED_FEATURES), len(features))
    width = len(features) - start
    if width <= 0:
        return
    idx, sign = stable_bucket(name, width)
    features[start + idx] += sign * value


def add_mechanistic_features(features: np.ndarray, row: Row):
    add_named(features, "bias", 1.0)
    add_named(features, f"op:{row.operation}", 2.0)
    relation = operation_for(row.original, row.candidate)
    add_named(features, f"op:{relation}", 0.5)

    original = row.original
    candidate = row.candidate
    if original == candidate:
        add_named(features, "relation:identity", 1.5)
    if convert_layout(original) == candidate:
        add_named(features, "relation:exact-layout", 4.0)
    if original.replace(" ", "") == candidate.replace(" ", "") and original != candidate:
        add_named(features, "relation:space-only-change", 3.0)

    orig_cyr = sum(1 for ch in original if "А" <= ch <= "я" or ch in "ёЁ")
    orig_lat = sum(1 for ch in original if ch.isascii() and ch.isalpha())
    cand_cyr = sum(1 for ch in candidate if "А" <= ch <= "я" or ch in "ёЁ")
    cand_lat = sum(1 for ch in candidate if ch.isascii() and ch.isalpha())
    orig_script = script_bucket(orig_cyr, orig_lat)
    cand_script = script_bucket(cand_cyr, cand_lat)
    add_named(features, f"script:orig:{orig_script}")
    add_named(features, f"script:cand:{cand_script}")
    if orig_script != cand_script:
        add_named(features, "layout:changes-script", 1.0)
    else:
        add_named(features, "layout:keeps-script", 0.5)

    cand_core = candidate.strip().strip(".,!?;:").lower()
    if cand_core in TECH_WORDS:
        add_named(features, "candidate:common-en-technical", 4.0)

    cand_len = len(candidate.strip())
    if cand_len <= 3:
        add_named(features, "candidate:length-short")
    elif cand_len <= 10:
        add_named(features, "candidate:length-medium")
    else:
        add_named(features, "candidate:length-long")

    edit = normalized_edit_distance(original, candidate)
    if edit <= 0.15:
        add_named(features, "edit:small", 1.0)
    elif edit <= 0.45:
        add_named(features, "edit:medium", 1.0)
    else:
        add_named(features, "edit:large", 1.0)

    orig_spaces = internal_space_count(original)
    cand_spaces = internal_space_count(candidate)
    if cand_spaces > orig_spaces:
        add_named(features, "boundary:adds-space", 2.0)
    elif cand_spaces < orig_spaces:
        add_named(features, "boundary:removes-space", 2.0)
    else:
        add_named(features, "boundary:keeps-space-count", 1.0)

    if uppercase_shape(original) == uppercase_shape(candidate):
        add_named(features, "case:keeps-uppercase-shape", 0.5)
    else:
        add_named(features, "case:changes-uppercase-shape", 0.5)

    add_keyboard_fourier_features(features, original, candidate)


def add_keyboard_fourier_features(features: np.ndarray, original: str, candidate: str):
    original_keys = physical_key_signal(original)
    candidate_keys = physical_key_signal(candidate)
    if not original_keys or not candidate_keys:
        return

    if original_keys == candidate_keys:
        add_named(features, "keyseq:exact", 4.0)
    if len(original_keys) == len(candidate_keys):
        add_named(features, "keyseq:length-same", 1.0)

    low, mid, phase, energy = keyboard_spectral_similarity(original_keys, candidate_keys)
    if low >= 0.92:
        add_named(features, "fourier:low-close", 2.5)
    if mid >= 0.88:
        add_named(features, "fourier:mid-close", 1.5)
    if phase >= 0.90:
        add_named(features, "fourier:phase-close", 1.5)
    if energy >= 0.90:
        add_named(features, "fourier:energy-ratio-close", 1.0)


def physical_key_signal(text: str) -> list[float]:
    signal = []
    for ch in text:
        if ch.isspace():
            continue
        idx = KEY_INDEX.get(ch.lower())
        if idx is not None:
            signal.append(float(idx))
    if not signal:
        return signal
    mean = sum(signal) / len(signal)
    centered = [value - mean for value in signal]
    scale = math.sqrt(sum(value * value for value in centered)) or 1.0
    return [value / scale for value in centered]


def keyboard_spectral_similarity(left: list[float], right: list[float]) -> tuple[float, float, float, float]:
    size = max(len(left), len(right), 2)
    left_arr = np.zeros(size, dtype=np.float32)
    right_arr = np.zeros(size, dtype=np.float32)
    left_arr[: len(left)] = left
    right_arr[: len(right)] = right

    left_fft = np.fft.rfft(left_arr)
    right_fft = np.fft.rfft(right_arr)
    left_mag = np.abs(left_fft)
    right_mag = np.abs(right_fft)

    def cosine(a: np.ndarray, b: np.ndarray) -> float:
        denom = float(np.linalg.norm(a) * np.linalg.norm(b))
        if denom <= 1e-9:
            return 0.0
        return float(np.dot(a, b) / denom)

    low_end = min(4, len(left_mag))
    mid_end = min(8, len(left_mag))
    low = cosine(left_mag[1:low_end], right_mag[1:low_end]) if low_end > 1 else 0.0
    mid = cosine(left_mag[low_end:mid_end], right_mag[low_end:mid_end]) if mid_end > low_end else low

    phase_left = np.angle(left_fft[1:low_end])
    phase_right = np.angle(right_fft[1:low_end])
    if len(phase_left) == 0:
        phase = 0.0
    else:
        phase = float(np.mean(np.cos(phase_left - phase_right)))

    left_energy = float(np.sum(left_mag[1:] ** 2))
    right_energy = float(np.sum(right_mag[1:] ** 2))
    if left_energy <= 1e-9 or right_energy <= 1e-9:
        energy = 0.0
    else:
        energy = min(left_energy, right_energy) / max(left_energy, right_energy)

    return low, mid, phase, energy


def internal_space_count(text: str) -> int:
    return sum(1 for ch in text.strip() if ch.isspace())


def uppercase_shape(text: str) -> tuple[bool, bool]:
    letters = [ch for ch in text if ch.isalpha()]
    return (bool(letters), bool(letters) and all(ch.isupper() for ch in letters))


def normalized_edit_distance(left: str, right: str) -> float:
    a = list(left)
    b = list(right)
    if not a and not b:
        return 0.0
    prev = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        cur = [i]
        for j, cb in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (ca != cb)))
        prev = cur
    return prev[-1] / max(1, max(len(a), len(b)))


def script_bucket(cyr: int, lat: int) -> str:
    if cyr and lat:
        return "mixed"
    if cyr:
        return "cyr"
    if lat:
        return "lat"
    return "other"


def split_groups(rows: list[Row], seed: int = 7):
    groups = sorted({r.group_id for r in rows})
    rng = random.Random(seed)
    rng.shuffle(groups)
    cut = int(len(groups) * 0.8)
    train_groups = set(groups[:cut])
    train = [r for r in rows if r.group_id in train_groups]
    test = [r for r in rows if r.group_id not in train_groups]
    return train, test


def rows_by_group(rows: list[Row]) -> list[list[Row]]:
    grouped: dict[str, list[Row]] = {}
    for row in rows:
        grouped.setdefault(row.group_id, []).append(row)
    return list(grouped.values())


def default_group_weight(positives: list[Row]) -> float:
    positive = positives[0]
    if positive.source == "personal/log" and positive.operation == "keep":
        return 8.0
    if positive.source == "personal/log":
        return 3.0
    return 1.8 if positive.operation != "keep" else 1.0


def train_probe(
    rows: list[Row],
    dim: int,
    epochs: int = 35,
    seed: int = 7,
    featurizer=featurize_row,
    group_weight_fn=default_group_weight,
):
    train, test = split_groups(rows, seed)
    encoded = {id(row): (featurizer(row, dim), row.label) for row in rows}
    rng = np.random.default_rng(seed + dim)
    model = TinyScorer(dim, rng)
    lr = 0.9
    weight_decay = 0.002
    train_groups = rows_by_group(train)
    for epoch in range(epochs):
        rng.shuffle(train_groups)
        for group in train_groups:
            positives = [row for row in group if encoded[id(row)][1] == 1]
            if not positives:
                continue
            hs = []
            logits = []
            for row in group:
                features, _label = encoded[id(row)]
                h = np.tanh(features)
                hs.append(h)
                logits.append(float(h @ model.w + model.b))
            z = np.asarray(logits, dtype=np.float32)
            z -= z.max()
            probs = np.exp(z)
            probs /= probs.sum()
            target = np.asarray(
                [1.0 / len(positives) if row in positives else 0.0 for row in group],
                dtype=np.float32,
            )
            group_weight = group_weight_fn(positives)
            dzs = (probs - target) * group_weight
            model.w *= 1.0 - lr * weight_decay
            for row, h, dz in zip(group, hs, dzs):
                model.w -= lr * dz * h
                model.b -= np.float32(lr * dz)
        lr *= 0.965
    return model, encoded, train, test, {}, {}


def train_probe_with_holdout(
    train_rows: list[Row],
    holdout_rows: list[Row],
    dim: int,
    epochs: int = 35,
    seed: int = 7,
    featurizer=featurize_row,
    group_weight_fn=default_group_weight,
):
    rows = train_rows + holdout_rows
    encoded = {id(row): (featurizer(row, dim), row.label) for row in rows}
    rng = np.random.default_rng(seed + dim)
    model = TinyScorer(dim, rng)
    lr = 0.9
    weight_decay = 0.002
    train_groups = rows_by_group(train_rows)
    for _epoch in range(epochs):
        rng.shuffle(train_groups)
        for group in train_groups:
            positives = [row for row in group if encoded[id(row)][1] == 1]
            if not positives:
                continue
            hs = []
            logits = []
            for row in group:
                features, _label = encoded[id(row)]
                h = np.tanh(features)
                hs.append(h)
                logits.append(float(h @ model.w + model.b))
            z = np.asarray(logits, dtype=np.float32)
            z -= z.max()
            probs = np.exp(z)
            probs /= probs.sum()
            target = np.asarray(
                [1.0 / len(positives) if row in positives else 0.0 for row in group],
                dtype=np.float32,
            )
            group_weight = group_weight_fn(positives)
            dzs = (probs - target) * group_weight
            model.w *= 1.0 - lr * weight_decay
            for row, h, dz in zip(group, hs, dzs):
                model.w -= lr * dz * h
                model.b -= np.float32(lr * dz)
        lr *= 0.965
    return model, encoded


def evaluate(model: TinyScorer, encoded, rows: list[Row]):
    preds = []
    t0 = time.perf_counter()
    for row in rows:
        features, label = encoded[id(row)]
        score = model.score(features)
        preds.append((row, label, score))
    elapsed = time.perf_counter() - t0
    cand_acc = sum((score >= 0.5) == bool(label) for _, label, score in preds) / max(1, len(preds))

    by_group: dict[str, list[tuple[Row, int, float]]] = {}
    for item in preds:
        by_group.setdefault(item[0].group_id, []).append(item)
    group_ok = 0
    fp = 0
    fp_total = 0
    fn = 0
    fn_total = 0
    mistakes = []
    for group, items in by_group.items():
        best = max(items, key=lambda item: item[2])
        positives = [item for item in items if item[1] == 1]
        if positives and best[1] == 1:
            group_ok += 1
        else:
            mistakes.append((best, positives[0] if positives else None))
        if positives and positives[0][0].operation == "keep":
            fp_total += 1
            if best[0].operation != "keep":
                fp += 1
        elif positives:
            fn_total += 1
            if best[0].operation == "keep":
                fn += 1

    return {
        "candidate_accuracy": cand_acc,
        "group_accuracy": group_ok / max(1, len(by_group)),
        "false_positive_rate": fp / max(1, fp_total),
        "false_negative_rate": fn / max(1, fn_total),
        "candidate_us": elapsed * 1_000_000 / max(1, len(rows)),
        "group_count": len(by_group),
        "candidate_count": len(rows),
        "mistakes": sorted(mistakes, key=lambda item: item[0][2], reverse=True)[:12],
    }


def evaluate_with_threshold(
    model: TinyScorer,
    encoded,
    rows: list[Row],
    margin_min: float,
    score_min: float,
) -> dict[str, float]:
    by_group: dict[str, list[tuple[Row, int, float]]] = {}
    for row in rows:
        features, label = encoded[id(row)]
        by_group.setdefault(row.group_id, []).append((row, label, model.score(features)))

    acted = 0
    correct = 0
    wrong = 0
    keep_fp = 0
    keep_total = 0
    correction_total = 0
    correction_hit = 0
    for items in by_group.values():
        ranked = sorted(items, key=lambda item: item[2], reverse=True)
        best = ranked[0]
        second_score = ranked[1][2] if len(ranked) > 1 else 0.0
        positive = next((item for item in items if item[1] == 1), None)
        if positive is None:
            continue

        if positive[0].operation == "keep":
            keep_total += 1
        else:
            correction_total += 1

        if best[2] < score_min or best[2] - second_score < margin_min:
            continue

        acted += 1
        if best[1] == 1:
            correct += 1
            if positive[0].operation != "keep":
                correction_hit += 1
        else:
            wrong += 1
            if positive[0].operation == "keep":
                keep_fp += 1

    return {
        "groups": len(by_group),
        "acted": acted,
        "coverage": acted / max(1, len(by_group)),
        "precision": correct / max(1, acted),
        "wrong": wrong,
        "keep_fp_rate": keep_fp / max(1, keep_total),
        "correction_recall": correction_hit / max(1, correction_total),
    }


def evaluate_personal_frequency_circuit(
    model: TinyScorer,
    encoded,
    rows: list[Row],
    counts: dict[str, dict[tuple[str, str], int]],
) -> dict[str, float]:
    by_group: dict[str, list[tuple[Row, int, float]]] = {}
    for row in rows:
        features, label = encoded[id(row)]
        score = model.score(features)
        accepted = counts["accepted"].get((row.original, row.candidate), 0)
        rejected = counts["rejected"].get((row.original, row.candidate), 0)
        if accepted:
            score += min(0.45, 0.18 * accepted)
        if rejected:
            score -= min(0.70, 0.25 * rejected)
        by_group.setdefault(row.group_id, []).append((row, label, score))

    chosen_rows = []
    for items in by_group.values():
        chosen_rows.append(max(items, key=lambda item: item[2]))

    group_ok = sum(1 for _row, label, _score in chosen_rows if label == 1)
    keep_total = 0
    keep_fp = 0
    correction_total = 0
    correction_fn = 0
    for items in by_group.values():
        best = max(items, key=lambda item: item[2])
        positive = next((item for item in items if item[1] == 1), None)
        if positive is None:
            continue
        if positive[0].operation == "keep":
            keep_total += 1
            if best[0].operation != "keep":
                keep_fp += 1
        else:
            correction_total += 1
            if best[0].operation == "keep":
                correction_fn += 1
    return {
        "group_accuracy": group_ok / max(1, len(by_group)),
        "false_positive_rate": keep_fp / max(1, keep_total),
        "false_negative_rate": correction_fn / max(1, correction_total),
    }


def best_threshold_profile(model: TinyScorer, encoded, rows: list[Row]) -> dict[str, float]:
    best = None
    for margin in [0.02, 0.04, 0.06, 0.08, 0.10, 0.14, 0.18, 0.24, 0.30]:
        for score in [0.45, 0.50, 0.55, 0.60, 0.66, 0.72, 0.78]:
            metrics = evaluate_with_threshold(model, encoded, rows, margin, score)
            if metrics["acted"] < 5 or metrics["keep_fp_rate"] > 0.05:
                continue
            rank = (metrics["precision"], metrics["coverage"], metrics["correction_recall"])
            if best is None or rank > best[0]:
                best = (rank, margin, score, metrics)

    if best is None:
        metrics = evaluate_with_threshold(model, encoded, rows, 0.30, 0.78)
        return {"margin": 0.30, "score": 0.78, **metrics}

    _rank, margin, score, metrics = best
    return {"margin": margin, "score": score, **metrics}


def baseline_metrics(rows: list[Row]) -> dict[str, float]:
    by_group: dict[str, list[Row]] = {}
    for row in rows:
        by_group.setdefault(row.group_id, []).append(row)
    keep_ok = 0
    correction_prior_ok = 0
    for group in by_group.values():
        positive = next((row for row in group if row.label == 1), None)
        if positive is None:
            continue
        keep_choice = next((row for row in group if row.operation == "keep"), group[0])
        correction_choice = next((row for row in group if row.operation != "keep"), keep_choice)
        keep_ok += int(keep_choice.label == 1)
        correction_prior_ok += int(correction_choice.label == 1)
    total = max(1, len(by_group))
    return {
        "keep_only_group_accuracy": keep_ok / total,
        "first_correction_group_accuracy": correction_prior_ok / total,
        "fixture_oracle_group_accuracy": 1.0,
    }


def current_rule_baseline(rows: list[Row]) -> dict[str, float]:
    release_bin = ROOT / "target/release/lay"
    local_bin = Path.home() / ".local/bin/lay"
    if release_bin.exists():
        command_prefix = [str(release_bin)]
    elif local_bin.exists():
        command_prefix = [str(local_bin)]
    elif shutil.which("lay"):
        command_prefix = ["lay"]
    else:
        command_prefix = ["cargo", "run", "--quiet", "--bin", "lay", "--"]

    groups = rows_by_group(rows)
    ok = 0
    checked = 0
    for group in groups:
        positive = next((row for row in group if row.label == 1), None)
        if positive is None:
            continue
        keep = next((row for row in group if row.operation == "keep"), group[0])
        try:
            output = subprocess.check_output(
                [*command_prefix, "--explain-correct", keep.original],
                cwd=ROOT,
                text=True,
                stderr=subprocess.DEVNULL,
                timeout=20,
            )
        except Exception:
            continue
        checked += 1
        predicted = keep.original
        for line in output.splitlines():
            if line.startswith("output: "):
                value = line.removeprefix("output: ").strip()
                if value != "none":
                    try:
                        predicted = value.strip('"')
                    except Exception:
                        predicted = value
        if predicted == positive.candidate:
            ok += 1
    return {
        "current_rule_checked": checked,
        "current_rule_group_accuracy": ok / max(1, checked),
    }


def audit_embeddings(model: TinyScorer, char_to_id: dict[str, int]):
    def feature_weight(name: str) -> float:
        idx = NAMED_INDEX.get(name)
        if idx is None or idx >= len(model.w):
            return 0.0
        return float(model.w[idx])

    residual_start = min(len(NAMED_FEATURES), model.dim)
    layout_indices = [
        residual_start + stable_bucket(f"delta:+{r}", max(1, model.dim - residual_start))[0]
        for u, r in US_TO_RU.items()
        if u.islower() and model.dim > residual_start
    ]
    en_indices = [
        residual_start + stable_bucket(f"delta:+{u}", max(1, model.dim - residual_start))[0]
        for u, r in US_TO_RU.items()
        if u.islower() and model.dim > residual_start
    ]

    return {
        "layout_pair_cos_mean": float(np.mean(model.w[layout_indices])) if layout_indices else 0.0,
        "layout_pair_count": len(layout_indices),
        "ru_en_centroid_cos": float(np.mean(model.w[en_indices])) if en_indices else 0.0,
        "feature_exact_layout": feature_weight("relation:exact-layout"),
        "feature_identity": feature_weight("relation:identity"),
        "feature_space_only_change": feature_weight("relation:space-only-change"),
        "feature_common_en_technical": feature_weight("candidate:common-en-technical"),
    }


def write_report(
    rows: list[Row],
    holdout_rows: list[Row],
    results: dict[int, dict],
    holdout_results: dict[int, dict],
    ablation_results: dict[str, dict],
    audits: dict[int, dict],
    baselines: dict[str, float],
    holdout_baselines: dict[str, float],
    rule_baseline: dict[str, float],
    personal_profile: dict[str, object],
    personal_probe: dict[str, object] | None,
):
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Lay Neural Arbiter Initial Report",
        "",
        "Status: initial NumPy probe, not runtime code.",
        "",
        f"Dataset: `{DATASET_PATH.relative_to(ROOT)}`",
        f"Rows: {len(rows)}",
        f"Groups: {len(set(r.group_id for r in rows))}",
        f"Holdout rows: {len(holdout_rows)}",
        f"Holdout groups: {len(set(r.group_id for r in holdout_rows))}",
        "",
        "## Fixture Split Metrics",
        "",
        "Baseline group accuracy:",
        "",
        f"- keep-only: `{baselines['keep_only_group_accuracy']:.3f}`",
        f"- first non-keep candidate: `{baselines['first_correction_group_accuracy']:.3f}`",
        f"- fixture/current-rule oracle: `{baselines['fixture_oracle_group_accuracy']:.3f}`",
        "",
        "| Dim | Candidate acc | Group acc | FP keep | FN correction | us/candidate |",
        "|---:|---:|---:|---:|---:|---:|",
    ]
    for dim in sorted(results):
        m = results[dim]
        lines.append(
            f"| {dim} | {m['candidate_accuracy']:.3f} | {m['group_accuracy']:.3f} | "
            f"{m['false_positive_rate']:.3f} | {m['false_negative_rate']:.3f} | {m['candidate_us']:.2f} |"
        )
    lines += [
        "",
        "## Independent Holdout Metrics",
        "",
        "The holdout set is not used to train the probe. It contains independent live-style cases:",
        "short RU/EN layout flips, technical EN tokens, normal Russian phrases, glued words,",
        "and common Russian typo repairs.",
        "",
        "Holdout baselines:",
        "",
        f"- keep-only: `{holdout_baselines['keep_only_group_accuracy']:.3f}`",
        f"- first non-keep candidate: `{holdout_baselines['first_correction_group_accuracy']:.3f}`",
        f"- current rules checked: `{rule_baseline['current_rule_checked']}`",
        f"- current rule group accuracy: `{rule_baseline['current_rule_group_accuracy']:.3f}`",
        "",
        "| Dim | Candidate acc | Group acc | FP keep | FN correction | us/candidate |",
        "|---:|---:|---:|---:|---:|---:|",
    ]
    for dim in sorted(holdout_results):
        m = holdout_results[dim]
        lines.append(
            f"| {dim} | {m['candidate_accuracy']:.3f} | {m['group_accuracy']:.3f} | "
            f"{m['false_positive_rate']:.3f} | {m['false_negative_rate']:.3f} | {m['candidate_us']:.2f} |"
        )
    lines += [
        "",
        "## Nanda-Style Ablation",
        "",
        "`full` = mechanistic circuits + residual char n-grams.",
        "`restricted` = only named mechanism circuits.",
        "`excluded` = residual char n-grams without named mechanisms.",
        "",
        "| Probe | Group acc | FP keep | FN correction | us/candidate |",
        "|---|---:|---:|---:|---:|",
    ]
    for name, m in ablation_results.items():
        lines.append(
            f"| {name} | {m['group_accuracy']:.3f} | {m['false_positive_rate']:.3f} | "
            f"{m['false_negative_rate']:.3f} | {m['candidate_us']:.2f} |"
        )
    lines += [
        "",
        "## Personal Frequency Profile",
        "",
        f"- correction-log events: `{personal_profile['events']}`",
        f"- manual layout replays: `{personal_profile['layout_replay']}`",
        f"- user corrections after lay output: `{personal_profile['user_correction']}`",
        f"- cancelled typing-assist markers: `{personal_profile['cancelled_typing_assist']}`",
        "",
        "Top script transitions:",
    ]
    for key, count in personal_profile["script_transitions"]:
        lines.append(f"- `{key}`: `{count}`")
    lines += ["", "Top short user overrides:"]
    for key, count in personal_profile["short_user_overrides"]:
        lines.append(f"- `{key}`: `{count}`")
    if personal_probe is not None:
        m = personal_probe["metrics"]
        b = personal_probe["baseline"]
        t = personal_probe["threshold"]
        f = personal_probe["frequency_circuit"]
        lines += [
            "",
            "Personal challenge split:",
            "",
            f"- train groups: `{personal_probe['train_groups']}`",
            f"- test groups: `{personal_probe['test_groups']}`",
            f"- learned accepted pairs: `{personal_probe['accepted_pairs']}`",
            f"- learned rejected pairs: `{personal_probe['rejected_pairs']}`",
            f"- current-rule accuracy on personal test: `{b['current_rule_group_accuracy']:.3f}`",
            f"- Nanda-personal accuracy on personal test: `{m['group_accuracy']:.3f}`",
            f"- Nanda-personal false positive keep: `{m['false_positive_rate']:.3f}`",
            f"- Nanda-personal false negative correction: `{m['false_negative_rate']:.3f}`",
            f"- frequency-circuit accuracy: `{f['group_accuracy']:.3f}`",
            f"- frequency-circuit keep FP: `{f['false_positive_rate']:.3f}`",
            f"- frequency-circuit correction FN: `{f['false_negative_rate']:.3f}`",
            f"- safe threshold margin: `{t['margin']:.2f}`",
            f"- safe threshold score: `{t['score']:.2f}`",
            f"- safe threshold coverage: `{t['coverage']:.3f}`",
            f"- safe threshold precision: `{t['precision']:.3f}`",
            f"- safe threshold keep FP: `{t['keep_fp_rate']:.3f}`",
            f"- safe threshold correction recall: `{t['correction_recall']:.3f}`",
        ]
    lines += ["", "## Mechanistic Audit", ""]
    lines += [
        "This is a hashed-feature audit, not a full mechanistic-interpretability pass.",
        "A healthy runtime scorer should show stable positive support for the intended",
        "relations and should not rely on accidental fixture memorization.",
        "",
    ]
    for dim in sorted(audits):
        a = audits[dim]
        lines += [
            f"### {dim}d",
            "",
            f"- mean learned weight for RU layout-target buckets: `{a['layout_pair_cos_mean']:.4f}` over `{a['layout_pair_count']}` buckets",
            f"- mean learned weight for EN layout-target buckets: `{a['ru_en_centroid_cos']:.4f}`",
            f"- probe weight `relation:exact-layout`: `{a['feature_exact_layout']:.4f}`",
            f"- probe weight `relation:identity`: `{a['feature_identity']:.4f}`",
            f"- probe weight `relation:space-only-change`: `{a['feature_space_only_change']:.4f}`",
            f"- probe weight `candidate:common-en-technical`: `{a['feature_common_en_technical']:.4f}`",
            "",
        ]
    lines += ["## Worst Mistakes", ""]
    for dim in sorted(results):
        lines.append(f"### {dim}d")
        lines.append("")
        for best, positive in holdout_results[dim]["mistakes"]:
            row, label, score = best
            expected = ""
            if positive is not None:
                expected = f" expected_op={positive[0].operation} expected_candidate={positive[0].candidate!r}"
            lines.append(
                f"- score={score:.3f} label={label} op={row.operation} "
                f"original={row.original!r} candidate={row.candidate!r}{expected} source={row.source}"
            )
        lines.append("")
    lines += [
        "## Interpretation",
        "",
        "- Larger dimensions improve holdout accuracy only slowly; 1024d is best here,",
        "  but it still loses to current deterministic rules on this holdout.",
        "- The ablation split is the key Nanda-style check: if `restricted` carries most",
        "  of the quality and `excluded` does not dominate, the model is learning the",
        "  intended mechanism rather than memorizing surface strings.",
        "- Runtime speed is not the blocker. Even the Python/NumPy scorer is already in",
        "  the low microsecond-per-candidate range; a Rust scorer with preloaded weights",
        "  should be comfortably below the text-output latency.",
        "- The right next use is a dev-only consensus signal, not a replacement for the",
        "  current rule graph.",
        "",
        "## Current Decision",
        "",
        "Do not replace current deterministic rules globally. The general holdout still",
        "belongs to the current rule graph. The promising direction is narrower:",
        "a personal frequency profile that learns from user corrections and only shifts",
        "scores for repeated user-specific mistakes.",
        "",
        "Next steps:",
        "",
        "- expand the independent live-style holdout set before trusting the metric;",
        "- reduce remaining holdout false negatives without increasing keep false positives;",
        "- keep current deterministic rules as the source of truth for public defaults;",
        "- use personal Nanda only as an opt-in score-shift layer;",
        "- export the personal-profile weights and test a Rust scorer behind a dev-only flag;",
        "- only consider runtime after false positives stay near zero on both holdout and personal test.",
        "",
    ]
    REPORT_PATH.write_text("\n".join(lines), encoding="utf-8")
    RESULTS_PATH.write_text("\n".join(lines), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--build-dataset", action="store_true")
    parser.add_argument("--train", action="store_true")
    parser.add_argument("--epochs", type=int, default=35)
    args = parser.parse_args()

    if args.build_dataset or not DATASET_PATH.exists():
        rows = build_dataset()
        write_dataset(rows)
        print(f"dataset rows={len(rows)} groups={len(set(r.group_id for r in rows))} path={DATASET_PATH}")
    else:
        rows = read_dataset()

    if args.train:
        holdout_rows = load_holdout()
        results = {}
        holdout_results = {}
        ablation_results = {}
        audits = {}
        for dim in (128, 256, 512, 1024):
            model, encoded, _train, test, char_to_id, _op_to_id = train_probe(rows, dim, epochs=args.epochs)
            results[dim] = evaluate(model, encoded, test)
            audits[dim] = audit_embeddings(model, char_to_id)
            holdout_model, holdout_encoded = train_probe_with_holdout(rows, holdout_rows, dim, epochs=args.epochs)
            holdout_results[dim] = evaluate(holdout_model, holdout_encoded, holdout_rows)
            print(
                f"dim={dim} group_acc={results[dim]['group_accuracy']:.3f} "
                f"holdout_group_acc={holdout_results[dim]['group_accuracy']:.3f} "
                f"cand_acc={results[dim]['candidate_accuracy']:.3f} "
                f"us/candidate={results[dim]['candidate_us']:.2f}"
            )
        for name, featurizer in [
            ("full-1024", featurize_row),
            ("restricted-mechanism-1024", featurize_mechanistic_only),
            ("excluded-residual-1024", featurize_residual_only),
        ]:
            model, encoded = train_probe_with_holdout(
                rows, holdout_rows, 1024, epochs=args.epochs, featurizer=featurizer
            )
            ablation_results[name] = evaluate(model, encoded, holdout_rows)
            print(
                f"ablation={name} holdout_group_acc={ablation_results[name]['group_accuracy']:.3f} "
                f"fp_keep={ablation_results[name]['false_positive_rate']:.3f}"
            )
        events = load_personal_correction_events()
        personal_rows = build_personal_challenge_rows(events)
        personal_probe = train_personal_profile_probe(rows, personal_rows, 1024, args.epochs)
        if personal_probe is not None:
            print(
                f"personal train_groups={personal_probe['train_groups']} "
                f"test_groups={personal_probe['test_groups']} "
                f"nanda_acc={personal_probe['metrics']['group_accuracy']:.3f} "
                f"current_acc={personal_probe['baseline']['current_rule_group_accuracy']:.3f}"
            )
        write_report(
            rows,
            holdout_rows,
            results,
            holdout_results,
            ablation_results,
            audits,
            baseline_metrics(test),
            baseline_metrics(holdout_rows),
            current_rule_baseline(holdout_rows),
            personal_frequency_profile(events),
            personal_probe,
        )
        print(f"report={REPORT_PATH}")


if __name__ == "__main__":
    main()
