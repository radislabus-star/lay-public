#!/usr/bin/env python3
"""Extend the canonical noun teacher with Russian verb, adjective and pronoun forms."""

from __future__ import annotations

import argparse
from collections import defaultdict
from pathlib import Path

import pymorphy3


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
NEIGHBOR_SCENES = (
    ("посмотри", ("просмотри", "подсмотри"), "_ сюда"),
    ("просмотри", ("посмотри", "подсмотри"), "_ документ"),
    ("надеть", ("одеть",), "_ пальто"),
    ("одеть", ("надеть",), "_ ребенка"),
    ("эффективный", ("эффектный",), "_ метод"),
    ("эффектный", ("эффективный",), "_ выход"),
    ("адресат", ("адресант",), "письмо получил _"),
    ("адресант", ("адресат",), "письмо отправил _"),
    ("кампания", ("компания",), "рекламная _"),
    ("компания", ("кампания",), "торговая _"),
    ("предоставить", ("представить",), "_ документы"),
    ("представить", ("предоставить",), "_ докладчика"),
    ("оплатить", ("уплатить",), "_ счет"),
    ("уплатить", ("оплатить",), "_ налог"),
    ("обсудить", ("осудить",), "_ предложение"),
    ("осудить", ("обсудить",), "_ поступок"),
    ("прибывать", ("пребывать",), "_ на станцию"),
    ("пребывать", ("прибывать",), "_ в покое"),
    ("развиваться", ("развеваться",), "быстро _"),
    ("развеваться", ("развиваться",), "_ на ветру"),
    ("проверить", ("поверить",), "_ расчеты"),
    ("поверить", ("проверить",), "_ человеку"),
    ("изменить", ("измерить",), "_ настройку"),
    ("измерить", ("изменить",), "_ длину"),
)


def args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--noun-corpus", type=Path, required=True)
    parser.add_argument("--dictionary", type=Path, default=Path("/usr/share/hunspell/ru_RU.dic"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--train-lemmas-per-pos", type=int, default=256)
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
        if "imp" in parts:
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


def main() -> int:
    options = args()
    if options.train_lemmas_per_pos < 4:
        raise SystemExit("--train-lemmas-per-pos must be at least 4")
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
        neighbor_count = 0
        for surface, competitors, context in NEIGHBOR_SCENES:
            target_bindings = sorted(binding_by_surface.get(surface, ()))
            existing_competitors = [
                competitor for competitor in competitors if binding_by_surface.get(competitor)
            ]
            if not target_bindings or not existing_competitors:
                continue
            lemma, features = target_bindings[0]
            competitors_csv = ",".join(existing_competitors)
            target.write(
                f"NT\t{lemma}\t{surface}\t{features}\t{context}\t{competitors_csv}\n"
            )
            target.write(
                f"NH\t{lemma}\t{surface}\t{features}\t{context}\t{competitors_csv}\n"
            )
            neighbor_count += 1

    counts = {
        pos: {
            "lemmas": len(paradigms),
            "bindings": sum(len(surface) for _, slots in paradigms for surface in slots.values()),
            "scenes": sum(len(slots) for _, slots in paradigms),
        }
        for pos, paradigms in sorted(by_pos.items())
    }
    print(counts)
    print(f"near_neighbor_relations={neighbor_count}")
    print(f"output={options.output} bytes={options.output.stat().st_size}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
