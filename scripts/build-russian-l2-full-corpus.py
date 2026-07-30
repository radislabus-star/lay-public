#!/usr/bin/env python3
"""Extend the canonical noun teacher with Russian verb, adjective and pronoun forms."""

from __future__ import annotations

import argparse
from collections import defaultdict
import json
from pathlib import Path
import re


POS = {
    "VERB": "verb", "INFN": "verb", "GRND": "verb",
    "ADJF": "adj", "ADJS": "adj", "COMP": "adj", "PRTF": "adj", "PRTS": "adj",
    "NPRO": "pron",
}
CASE = {
    "nomn": "nom", "gent": "gen", "gen2": "part", "datv": "dat",
    "accs": "acc", "ablt": "ins", "loct": "prep", "loc2": "loc2", "voct": "voc",
}
NUMBER = {"sing": "sg", "plur": "pl"}
GENDER = {"masc": "masc", "femn": "fem", "neut": "neut"}
PERSON = {"1per": "p1", "2per": "p2", "3per": "p3"}
TENSE = {"past": "past", "pres": "pres", "futr": "fut"}
MOOD = {"indc": "ind", "impr": "imp"}
ASPECT = {"perf": "perf", "impf": "imperf"}
EXCLUDED = {"Name", "Surn", "Patr", "Geox", "Orgn", "Trad", "Abbr"}
WORD_RE = re.compile(r"[а-яё]+(?:-[а-яё]+)?", re.IGNORECASE)
MAX_CONTEXTS_PER_SURFACE = 4
MAX_COMPETITORS_PER_SURFACE = 4
MAX_DELETION_BUCKET = 24


def args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--noun-corpus", type=Path, required=True)
    parser.add_argument("--dictionary", type=Path, default=Path("/usr/share/hunspell/ru_RU.dic"))
    parser.add_argument("--context-corpus", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--train-lemmas-per-pos", type=int, default=256)
    parser.add_argument(
        "--max-neighbor-surfaces",
        type=int,
        default=0,
        help="0 admits every corpus-backed near-neighbor surface",
    )
    return parser.parse_args()


def words(path: Path):
    with path.open(encoding="utf-8") as source:
        next(source, None)
        for raw in source:
            word = raw.strip().split()[0].split("/", 1)[0].lower() if raw.strip() else ""
            if len(word) >= 2 and all(ch.isalpha() or ch == "-" for ch in word):
                yield word


def feature(tag) -> str | None:
    raw_pos = str(tag.POS)
    pos = POS.get(raw_pos)
    if pos is None:
        return None
    if raw_pos in {"ADJS", "PRTS"} and tag.case is not None:
        # pymorphy exposes a few dictionary homonyms with impossible
        # short-adjective case tags (for example feminine "бела" as genitive).
        return None
    parts = [pos]
    grammemes = {str(value) for value in tag.grammemes}
    if raw_pos == "INFN":
        parts.append("inf")
    elif raw_pos == "GRND":
        parts.append("ger")
    elif raw_pos in {"ADJS", "PRTS"}:
        parts.append("short")
    elif raw_pos == "COMP":
        parts.append("comp")
    if "impr" in grammemes:
        if "incl" in grammemes:
            parts.append("imp_incl")
        elif "excl" in grammemes:
            parts.append("imp_excl")
    for table, value in (
        (CASE, str(tag.case)),
        (NUMBER, str(tag.number)),
        (GENDER, str(tag.gender)),
        (PERSON, str(tag.person)),
        (TENSE, str(tag.tense)),
        (MOOD, str(tag.mood)),
        (ASPECT, next((item for item in ("perf", "impf") if item in grammemes), "None")),
    ):
        mapped = table.get(value)
        if mapped is not None:
            parts.append(mapped)
    if any(value in parts for value in GENDER.values()) and not any(
        value in parts for value in NUMBER.values()
    ):
        parts.append("sg")
    if pos == "verb" and raw_pos not in {"INFN"} and not any(part in MOOD.values() for part in parts):
        parts.append("ind")
    if pos == "pron" and len(parts) == 1:
        return None
    return ":".join(parts)


def context_for(features: str, variant: int) -> str:
    parts = set(features.split(":"))
    if "verb" in parts:
        if "inf" in parts:
            return ("хочу _", "нужно _", "можно _", "буду _")[variant % 4]
        if "ger" in parts:
            return ("_ одновременно", "_ затем", "_ осторожно", "_ быстро")[variant % 4]
        if "imp_incl" in parts or "imp_excl" in parts:
            if "imp_incl" in parts:
                return "давайте _"
            return "вы _" if "pl" in parts else "ты _"
        subject = {
            "p1": ("я", "мы" if "pl" in parts else "я"),
            "p2": ("ты", "вы" if "pl" in parts else "ты"),
            "p3": ("он", "они" if "pl" in parts else "он"),
        }
        for person, values in subject.items():
            if person in parts:
                return f"{values[1]} _"
        if "fem" in parts:
            return "она _"
        if "neut" in parts:
            return "оно _"
        if "pl" in parts:
            return "они _"
        return "он _"
    if "adj" in parts:
        if "comp" in parts:
            return "_ чем раньше"
        if "short" in parts:
            noun = "они" if "pl" in parts else ("она" if "fem" in parts else "оно" if "neut" in parts else "он")
            return f"{noun} _"
        case_prefix = {
            "gen": "нет", "dat": "к", "acc": "вижу", "ins": "с",
            "prep": "говорю о", "loc2": "нахожусь в",
        }
        noun = "дома" if "pl" in parts else ("книге" if "fem" in parts else "окне" if "neut" in parts else "доме")
        prefix = next((value for key, value in case_prefix.items() if key in parts), "")
        return f"{prefix} _ {noun}".strip()
    if "pron" in parts:
        if "dat" in parts:
            return "передай _"
        if "gen" in parts:
            return "нет _"
        if "ins" in parts:
            return "рядом с _"
        if "prep" in parts:
            return "говорю о _"
        if "acc" in parts:
            return "вижу _"
        return "_ здесь"
    raise ValueError(features)


def collect(morph, dictionary: Path):
    forms = defaultdict(lambda: defaultdict(set))
    seen_lexemes = set()
    for word in words(dictionary):
        for parse in morph.parse(word):
            if str(parse.tag.POS) not in POS or EXCLUDED.intersection(parse.tag.grammemes):
                continue
            lemma = parse.normal_form.lower()
            identity = (lemma, str(parse.tag.POS), str(parse.tag.aspect))
            if identity in seen_lexemes:
                continue
            seen_lexemes.add(identity)
            for item in parse.lexeme:
                features = feature(item.tag)
                surface = item.word.lower()
                if features and len(surface) >= 2 and all(ch.isalpha() or ch == "-" for ch in surface):
                    forms[(POS[str(item.tag.POS)], lemma)][features].add(surface)
    return forms


def corpus_documents(path: Path):
    with path.open(encoding="utf-8", errors="ignore") as source:
        if path.suffix.lower() != ".jsonl":
            for document_index, line in enumerate(source):
                yield document_index % 5 != 0, (line,)
            return
        for document_index, raw in enumerate(source):
            try:
                record = json.loads(raw)
            except json.JSONDecodeError as error:
                raise ValueError(
                    f"{path}:{document_index + 1}: invalid context JSONL"
                ) from error
            text = record.get("text")
            if not isinstance(text, str):
                continue
            yield document_index % 5 != 0, text.splitlines()


def corpus_contexts(
    path: Path,
    known_surfaces: set[str],
) -> tuple[dict[str, list[str]], dict[str, list[str]]]:
    train_contexts: dict[str, list[str]] = defaultdict(list)
    heldout_contexts: dict[str, list[str]] = defaultdict(list)
    for is_train, lines in corpus_documents(path):
        contexts = train_contexts if is_train else heldout_contexts
        for raw in lines:
            tokens = [match.group(0).lower() for match in WORD_RE.finditer(raw)]
            for index, token in enumerate(tokens):
                if token not in known_surfaces:
                    continue
                values = contexts[token]
                if len(values) >= MAX_CONTEXTS_PER_SURFACE:
                    continue
                start = max(0, index - 2)
                end = min(len(tokens), index + 3)
                window = tokens[start:end]
                window[index - start] = "_"
                context = " ".join(window)
                if context not in values:
                    values.append(context)
    return train_contexts, heldout_contexts


def is_single_edit_neighbor(left: str, right: str) -> bool:
    if left == right or abs(len(left) - len(right)) > 1:
        return False
    if len(left) == len(right):
        mismatches = [index for index, pair in enumerate(zip(left, right)) if pair[0] != pair[1]]
        if len(mismatches) == 1:
            return True
        return (
            len(mismatches) == 2
            and mismatches[1] == mismatches[0] + 1
            and left[mismatches[0]] == right[mismatches[1]]
            and left[mismatches[1]] == right[mismatches[0]]
        )
    shorter, longer = (left, right) if len(left) < len(right) else (right, left)
    short_index = 0
    long_index = 0
    skipped = 0
    while short_index < len(shorter) and long_index < len(longer):
        if shorter[short_index] == longer[long_index]:
            short_index += 1
            long_index += 1
        else:
            skipped += 1
            long_index += 1
            if skipped > 1:
                return False
    return True


def neighbor_candidates(
    observed_surfaces: set[str],
    binding_by_surface: dict[str, list[tuple[str, str]]],
) -> dict[str, list[str]]:
    deletion_buckets: dict[str, list[str]] = defaultdict(list)
    for surface in sorted(observed_surfaces):
        if not 3 <= len(surface) <= 24:
            continue
        for index in range(len(surface)):
            key = surface[:index] + surface[index + 1 :]
            bucket = deletion_buckets[key]
            if len(bucket) < MAX_DELETION_BUCKET and surface not in bucket:
                bucket.append(surface)

    raw_neighbors: dict[str, set[str]] = defaultdict(set)
    for key, bucket in deletion_buckets.items():
        candidates = list(bucket)
        if key in observed_surfaces:
            candidates.append(key)
        candidates = sorted(set(candidates))
        if len(candidates) > MAX_DELETION_BUCKET:
            continue
        for left_index, left in enumerate(candidates):
            for right in candidates[left_index + 1 :]:
                if is_single_edit_neighbor(left, right):
                    raw_neighbors[left].add(right)
                    raw_neighbors[right].add(left)

    result: dict[str, list[str]] = {}
    for surface, candidates in sorted(raw_neighbors.items()):
        target_bindings = binding_by_surface[surface]
        target_pos = {features.split(":", 1)[0] for _, features in target_bindings}
        target_lemmas = {lemma for lemma, _ in target_bindings}
        admitted = []
        for candidate in sorted(candidates):
            candidate_bindings = binding_by_surface[candidate]
            candidate_pos = {
                features.split(":", 1)[0] for _, features in candidate_bindings
            }
            candidate_lemmas = {lemma for lemma, _ in candidate_bindings}
            if target_pos.isdisjoint(candidate_pos) or target_lemmas == candidate_lemmas:
                continue
            admitted.append(candidate)
            if len(admitted) >= MAX_COMPETITORS_PER_SURFACE:
                break
        if admitted:
            result[surface] = admitted
    return result


def learned_neighbor_scenes(
    context_corpus: Path,
    binding_by_surface: dict[str, list[tuple[str, str]]],
    max_surfaces: int,
) -> tuple[list[tuple[str, str, str, str, list[str], str]], dict[str, int]]:
    train_contexts, heldout_contexts = corpus_contexts(
        context_corpus,
        set(binding_by_surface),
    )
    observed = {
        surface
        for surface in train_contexts.keys() & heldout_contexts.keys()
        if train_contexts[surface] and heldout_contexts[surface]
    }
    candidates = neighbor_candidates(observed, binding_by_surface)
    selected = []
    for surface in sorted(candidates):
        competitors = candidates[surface]
        competitor_pos = {
            features.split(":", 1)[0]
            for competitor in competitors
            for _, features in binding_by_surface[competitor]
        }
        target = next(
            (
                (lemma, features)
                for lemma, features in sorted(binding_by_surface[surface])
                if features.split(":", 1)[0] in competitor_pos
            ),
            None,
        )
        if target is None:
            continue
        lemma, features = target
        selected.append((lemma, surface, features, competitors))
        if max_surfaces and len(selected) >= max_surfaces:
            break

    train_targets_by_context: dict[str, set[str]] = defaultdict(set)
    for _, surface, _, _ in selected:
        train_targets_by_context[train_contexts[surface][0]].add(surface)

    scenes = []
    ambiguous_heldout_contexts = 0
    surfaces_without_independent_heldout = 0
    for lemma, surface, features, competitors in selected:
        train_context = train_contexts[surface][0]
        heldout_context = next(
            (
                context
                for context in heldout_contexts[surface]
                if not (
                    (train_targets_by_context.get(context, set()) - {surface})
                    & set(competitors)
                )
            ),
            None,
        )
        ambiguous_heldout_contexts += sum(
            bool(
                (train_targets_by_context.get(context, set()) - {surface})
                & set(competitors)
            )
            for context in heldout_contexts[surface]
        )
        if heldout_context is None:
            surfaces_without_independent_heldout += 1
            continue
        scenes.append((lemma, surface, features, train_context, competitors, "NT"))
        scenes.append((lemma, surface, features, heldout_context, competitors, "NH"))
    return scenes, {
        "candidate_surfaces": len(selected),
        "admitted_surfaces": len(scenes) // 2,
        "ambiguous_heldout_contexts": ambiguous_heldout_contexts,
        "surfaces_without_independent_heldout": surfaces_without_independent_heldout,
    }


def main() -> int:
    options = args()
    if options.train_lemmas_per_pos < 4:
        raise SystemExit("--train-lemmas-per-pos must be at least 4")
    if options.max_neighbor_surfaces < 0:
        raise SystemExit("--max-neighbor-surfaces must be non-negative")
    if not options.context_corpus.is_file():
        raise SystemExit(f"context corpus is missing: {options.context_corpus}")
    try:
        import pymorphy3
    except ImportError as error:
        raise SystemExit("pymorphy3 is required to build the full L2 corpus") from error
    forms = collect(pymorphy3.MorphAnalyzer(), options.dictionary)
    by_pos = defaultdict(list)
    for (pos, lemma), slots in forms.items():
        if slots:
            by_pos[pos].append((lemma, slots))
    for paradigms in by_pos.values():
        paradigms.sort(key=lambda item: item[0])

    options.output.parent.mkdir(parents=True, exist_ok=True)
    noun = options.noun_corpus.read_text(encoding="utf-8")
    with options.output.open("w", encoding="utf-8", newline="\n") as target:
        target.write(noun)
        if not noun.endswith("\n"):
            target.write("\n")
        target.write("# Extended POS teacher generated by build-russian-l2-full-corpus.py\n")
        for pos in sorted(by_pos):
            paradigms = by_pos[pos]
            train_lemma_count = min(
                options.train_lemmas_per_pos,
                max(4, len(paradigms) * 2 // 3),
            )
            for lemma, slots in paradigms:
                for features, surfaces in sorted(slots.items()):
                    for surface in sorted(surfaces):
                        target.write(f"F\t{lemma}\t{surface}\t{features}\n")
            for index, (lemma, slots) in enumerate(paradigms):
                kind = "T" if index < train_lemma_count else "H"
                for features, surfaces in sorted(slots.items()):
                    surface = sorted(surfaces)[0]
                    target.write(
                        f"{kind}\t{lemma}\t{surface}\t{features}\t"
                        f"{context_for(features, index)}\n"
                    )
        binding_by_surface = defaultdict(list)
        for paradigms in by_pos.values():
            for lemma, slots in paradigms:
                for features, surfaces in slots.items():
                    for surface in surfaces:
                        binding_by_surface[surface].append((lemma, features))
        neighbor_scenes, neighbor_stats = learned_neighbor_scenes(
            options.context_corpus,
            binding_by_surface,
            options.max_neighbor_surfaces,
        )
        for lemma, surface, features, context, competitors, kind in neighbor_scenes:
            target.write(
                f"{kind}\t{lemma}\t{surface}\t{features}\t{context}\t"
                f"{','.join(competitors)}\n"
            )

    counts = {
        pos: {
            "lemmas": len(paradigms),
            "bindings": sum(len(surface) for _, slots in paradigms for surface in slots.values()),
            "scenes": sum(len(slots) for _, slots in paradigms),
        }
        for pos, paradigms in sorted(by_pos.items())
    }
    print(counts)
    print(f"near_neighbor_train_scenes={sum(scene[5] == 'NT' for scene in neighbor_scenes)}")
    print(f"near_neighbor_heldout_scenes={sum(scene[5] == 'NH' for scene in neighbor_scenes)}")
    print(f"near_neighbor_split={neighbor_stats}")
    print(f"context_corpus={options.context_corpus}")
    print(f"output={options.output} bytes={options.output.stat().st_size}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
