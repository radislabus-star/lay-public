#!/usr/bin/env python3
"""Build a deterministic noun-declension teacher corpus for Lay L2.

pymorphy3 is a cold teacher only. The generated TSV stores surface/lemma/slot
evidence; Lay runtime never imports Python or pymorphy3.
"""

from __future__ import annotations

import argparse
import sys
from collections import defaultdict
from pathlib import Path

import pymorphy3


CORE_CASES = (
    ("nomn", "nom"),
    ("gent", "gen"),
    ("datv", "dat"),
    ("accs", "acc"),
    ("ablt", "ins"),
    ("loct", "prep"),
)

EXTRA_CASES = (
    ("gen2", "part"),
    ("loc2", "loc2"),
    ("voct", "voc"),
)

ALL_CASES = CORE_CASES + EXTRA_CASES

NUMBERS = (
    ("sing", "sg"),
    ("plur", "pl"),
)

CONTEXTS = {
    ("nom", "sg"): ("_ находится здесь", "_ уже готов", "_ стоит рядом", "_ появился"),
    ("gen", "sg"): ("нет _", "без _", "около _", "после _"),
    ("dat", "sg"): ("иду к _", "подошел к _", "двигаюсь к _", "приступаю к _"),
    ("acc", "sg"): ("вижу _", "посетил _", "ремонтирую _", "проверяю _"),
    ("ins", "sg"): ("любуюсь _", "пользуюсь _", "управляю _", "доволен _"),
    ("prep", "sg"): ("думаю о _", "говорю о _", "живу в _", "сосредоточен на _"),
    ("nom", "pl"): ("_ находятся здесь", "_ уже готовы", "_ стоят рядом", "_ появились"),
    ("gen", "pl"): ("несколько _", "много _", "нет нескольких _", "около многих _"),
    ("dat", "pl"): ("к нескольким _", "по многим _", "навстречу нескольким _", "благодаря многим _"),
    ("acc", "pl"): ("вижу несколько _", "проверяю многие _", "посетил разные _", "заметил эти _"),
    ("ins", "pl"): ("между несколькими _", "с многими _", "управляю этими _", "доволен многими _"),
    ("prep", "pl"): ("о нескольких _", "во многих _", "говорю об этих _", "сосредоточен на разных _"),
    ("part", "sg"): ("немного _", "чашка _", "добавить _", "ложка _"),
    ("loc2", "sg"): ("нахожусь в _", "лежит на _", "гуляю в _", "говорю на _"),
    ("voc", "sg"): ("_ иди сюда", "_ послушай", "_ привет", "_ ответь"),
    ("part", "pl"): ("немного разных _", "несколько видов _", "добавить разных _", "множество _"),
    ("loc2", "pl"): ("нахожусь в этих _", "лежит на этих _", "гуляю в разных _", "говорю на многих _"),
    ("voc", "pl"): ("_ идите сюда", "_ послушайте", "_ привет", "_ ответьте"),
}

EXCLUDED_GRAMMEMES = {"Name", "Surn", "Patr", "Geox", "Orgn", "Trad", "Abbr"}
CASE_BY_PYMORPHY = dict(ALL_CASES)
NUMBER_BY_PYMORPHY = dict(NUMBERS)
CASE_ORDER = {case: index for index, (case, _) in enumerate(ALL_CASES)}
NUMBER_ORDER = {number: index for index, (number, _) in enumerate(NUMBERS)}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dictionary", type=Path, default=Path("/usr/share/hunspell/ru_RU.dic"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--target-forms", type=int, default=300_000)
    parser.add_argument("--train-lemmas", type=int, default=128)
    return parser.parse_args()


def dictionary_words(path: Path):
    with path.open(encoding="utf-8") as source:
        next(source, None)
        for raw in source:
            value = raw.strip().split()[0] if raw.strip() else ""
            word = value.split("/", 1)[0].strip().lower()
            if word and surface_is_supported(word):
                yield word


def surface_is_supported(word: str) -> bool:
    return len(word) >= 2 and all(character.isalpha() or character == "-" for character in word)


def noun_lexeme_forms(parse):
    if parse.tag.POS != "NOUN" or EXCLUDED_GRAMMEMES.intersection(parse.tag.grammemes):
        return None
    lemma = parse.normal_form.lower()
    if not surface_is_supported(lemma):
        return None
    forms = defaultdict(set)
    for form in parse.lexeme:
        case = str(form.tag.case)
        number = str(form.tag.number)
        if case not in CASE_BY_PYMORPHY or number not in NUMBER_BY_PYMORPHY:
            continue
        surface = form.word.lower()
        if surface_is_supported(surface):
            forms[(case, number)].add(surface)
    return lemma, forms


def collect_paradigms(morph, dictionary: Path, target_forms: int):
    forms_by_lemma = defaultdict(lambda: defaultdict(set))
    next_progress = 25_000
    for word_index, word in enumerate(dictionary_words(dictionary), 1):
        for parse in morph.parse(word):
            lexeme = noun_lexeme_forms(parse)
            if lexeme is None:
                continue
            lemma, observed_forms = lexeme
            for slot, surfaces in observed_forms.items():
                forms_by_lemma[lemma][slot].update(surfaces)
        if word_index >= next_progress:
            print(
                f"dictionary_rows={word_index} observed_lemmas={len(forms_by_lemma)}",
                file=sys.stderr,
            )
            next_progress += 25_000

    paradigms = []
    unique_surfaces = set()
    for lemma, observed_forms in forms_by_lemma.items():
        complete_numbers = [
            pymorphy_number
            for pymorphy_number, _ in NUMBERS
            if all(
                observed_forms.get((pymorphy_case, pymorphy_number))
                for pymorphy_case, _ in CORE_CASES
            )
        ]
        if not complete_numbers:
            continue
        forms = {
            slot: tuple(sorted(surfaces))
            for slot, surfaces in observed_forms.items()
            if slot[1] in complete_numbers
        }
        paradigms.append((lemma, forms))
        for surfaces in forms.values():
            unique_surfaces.update(surfaces)

    both_numbers = {number for number, _ in NUMBERS}
    paradigms.sort(
        key=lambda paradigm: (
            {
                number
                for _, number in paradigm[1]
            }
            != both_numbers,
            paradigm[0],
        )
    )
    if len(unique_surfaces) < target_forms:
        raise RuntimeError(
            f"dictionary yielded only {len(unique_surfaces)} unique noun forms; "
            f"target is {target_forms}"
        )
    return paradigms, unique_surfaces


def slot_sort_key(slot):
    case, number = slot
    return NUMBER_ORDER[number], CASE_ORDER[case]


def split_paradigms(paradigms, train_lemmas: int):
    required_per_slot = len(next(iter(CONTEXTS.values())))
    selected = set()
    available_slots = sorted(
        {slot for _, forms in paradigms for slot in forms},
        key=slot_sort_key,
    )
    for slot in available_slots:
        covered = 0
        for index, (_, forms) in enumerate(paradigms):
            if slot not in forms:
                continue
            selected.add(index)
            covered += 1
            if covered >= required_per_slot:
                break
    for index in range(len(paradigms)):
        if len(selected) >= train_lemmas:
            break
        selected.add(index)
    if len(selected) > train_lemmas:
        raise RuntimeError(
            f"train_lemmas={train_lemmas} is too small for morphology slot coverage; "
            f"need at least {len(selected)}"
        )
    training = [paradigm for index, paradigm in enumerate(paradigms) if index in selected]
    heldout = [paradigm for index, paradigm in enumerate(paradigms) if index not in selected]
    return training, heldout


def write_corpus(
    output: Path,
    paradigms,
    target_forms: int,
    train_lemmas: int,
):
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8", newline="\n") as target:
        target.write("# Generated by scripts/build-russian-morphology-corpus.py\n")
        target.write(f"# target_unique_forms={target_forms}\n")
        target.write("# teacher=pymorphy3; runtime_dependency=false\n")
        for lemma, forms in paradigms:
            for (pymorphy_case, pymorphy_number), surfaces in sorted(
                forms.items(), key=lambda item: slot_sort_key(item[0])
            ):
                lay_case = CASE_BY_PYMORPHY[pymorphy_case]
                lay_number = NUMBER_BY_PYMORPHY[pymorphy_number]
                for surface in surfaces:
                    target.write(
                        f"F\t{lemma}\t{surface}"
                        f"\tnoun:{lay_case}:{lay_number}\n"
                    )

        training, heldout = split_paradigms(paradigms, train_lemmas)
        for lemma_index, (lemma, forms) in enumerate(training):
            for (pymorphy_case, pymorphy_number), surfaces in sorted(
                forms.items(), key=lambda item: slot_sort_key(item[0])
            ):
                lay_case = CASE_BY_PYMORPHY[pymorphy_case]
                lay_number = NUMBER_BY_PYMORPHY[pymorphy_number]
                contexts = CONTEXTS[(lay_case, lay_number)]
                context = contexts[lemma_index % len(contexts)]
                target.write(
                    f"T\t{lemma}\t{surfaces[0]}\t"
                    f"noun:{lay_case}:{lay_number}\t{context}\n"
                )
        for lemma_index, (lemma, forms) in enumerate(heldout):
            for (pymorphy_case, pymorphy_number), surfaces in sorted(
                forms.items(), key=lambda item: slot_sort_key(item[0])
            ):
                lay_case = CASE_BY_PYMORPHY[pymorphy_case]
                lay_number = NUMBER_BY_PYMORPHY[pymorphy_number]
                contexts = CONTEXTS[(lay_case, lay_number)]
                context = contexts[lemma_index % len(contexts)]
                target.write(
                    f"H\t{lemma}\t{surfaces[0]}\t"
                    f"noun:{lay_case}:{lay_number}\t{context}\n"
                )


def main() -> int:
    args = parse_args()
    if args.target_forms <= 0 or args.train_lemmas <= 0:
        raise SystemExit("target forms and train lemmas must be positive")
    morph = pymorphy3.MorphAnalyzer()
    paradigms, unique_surfaces = collect_paradigms(
        morph, args.dictionary, args.target_forms
    )
    if len(paradigms) <= args.train_lemmas:
        raise SystemExit("not enough paradigms for a heldout split")
    dual_number_lemmas = sum(
        {
            number
            for _, number in forms
        }
        == {number for number, _ in NUMBERS}
        for _, forms in paradigms
    )
    plural_only_lemmas = sum(
        {number for _, number in forms} == {"plur"} for _, forms in paradigms
    )
    singular_only_lemmas = sum(
        {number for _, number in forms} == {"sing"} for _, forms in paradigms
    )
    bindings = sum(
        len(surfaces)
        for _, forms in paradigms
        for surfaces in forms.values()
    )
    multi_surface_slots = sum(
        len(surfaces) > 1
        for _, forms in paradigms
        for surfaces in forms.values()
    )
    write_corpus(args.output, paradigms, args.target_forms, args.train_lemmas)
    print(
        f"lemmas={len(paradigms)} "
        f"dual_number_lemmas={dual_number_lemmas} "
        f"plural_only_lemmas={plural_only_lemmas} "
        f"singular_only_lemmas={singular_only_lemmas} "
        f"bindings={bindings} multi_surface_slots={multi_surface_slots} "
        f"unique_forms={len(unique_surfaces)} train_lemmas={args.train_lemmas} "
        f"heldout_lemmas={len(paradigms) - args.train_lemmas} output={args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
