#!/usr/bin/env python3
"""Return Rust source rows that are compiled outside ``cfg(test)`` items."""

from __future__ import annotations

import re
from pathlib import Path


CFG_TEST_ATTRIBUTE = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
RAW_STRING_PREFIX = re.compile(r'(?:br|r)(#+)?"')
RUST_ITEM_HEAD = re.compile(
    r"^(?:(?:pub(?:\([^)]*\))?|unsafe|async|default)\s+)*"
    r"(?:fn|mod|use|struct|enum|union|trait|impl|type|static|const|extern|macro_rules!)\b"
)


def rust_code_projection(source: str) -> str:
    """Preserve code punctuation/newlines while blanking comments and literals."""

    output = list(source)
    index = 0
    block_depth = 0
    string = False
    raw_hashes: int | None = None

    def blank(position: int) -> None:
        if output[position] != "\n":
            output[position] = " "

    while index < len(source):
        if block_depth:
            if source.startswith("/*", index):
                blank(index)
                blank(index + 1)
                block_depth += 1
                index += 2
            elif source.startswith("*/", index):
                blank(index)
                blank(index + 1)
                block_depth -= 1
                index += 2
            else:
                blank(index)
                index += 1
            continue

        if raw_hashes is not None:
            terminator = '"' + ("#" * raw_hashes)
            if source.startswith(terminator, index):
                for offset in range(len(terminator)):
                    blank(index + offset)
                index += len(terminator)
                raw_hashes = None
            else:
                blank(index)
                index += 1
            continue

        if string:
            if source[index] == "\\" and index + 1 < len(source):
                blank(index)
                blank(index + 1)
                index += 2
            else:
                closing = source[index] == '"'
                blank(index)
                index += 1
                if closing:
                    string = False
            continue

        if source.startswith("//", index):
            while index < len(source) and source[index] != "\n":
                blank(index)
                index += 1
            continue
        if source.startswith("/*", index):
            blank(index)
            blank(index + 1)
            block_depth = 1
            index += 2
            continue

        raw_match = RAW_STRING_PREFIX.match(source, index)
        if raw_match is not None:
            token = raw_match.group(0)
            raw_hashes = len(raw_match.group(1) or "")
            for offset in range(len(token)):
                blank(index + offset)
            index += len(token)
            continue

        if source.startswith('b"', index):
            blank(index)
            blank(index + 1)
            index += 2
            string = True
            continue
        if source[index] == '"':
            blank(index)
            index += 1
            string = True
            continue

        # A brace-shaped character literal must not affect item depth. Accept
        # only Rust's actual one-scalar or escaped forms; a lifetime/label has
        # no closing quote and must remain code.
        if source[index] == "'":
            closing: int | None = None
            if index + 2 < len(source) and source[index + 1] not in {"\\", "\n", "\r", "'"}:
                if source[index + 2] == "'":
                    closing = index + 2
            elif index + 3 < len(source) and source[index + 1] == "\\":
                escape = source[index + 2]
                if escape == "x" and index + 5 < len(source) and source[index + 5] == "'":
                    closing = index + 5
                elif escape == "u" and source.startswith("\\u{", index + 1):
                    brace = source.find("}", index + 4)
                    if brace != -1 and brace + 1 < len(source) and source[brace + 1] == "'":
                        closing = brace + 1
                elif source[index + 3] == "'":
                    closing = index + 3
            if closing is not None:
                for position in range(index, closing + 1):
                    blank(position)
                index = closing + 1
                continue

        index += 1

    return "".join(output)


def cfg_test_item_lines(source: str) -> set[int]:
    """Return zero-based line indexes owned by direct ``#[cfg(test)]`` items."""

    code_lines = rust_code_projection(source).splitlines()
    excluded: set[int] = set()
    for attribute_index, line in enumerate(code_lines):
        if CFG_TEST_ATTRIBUTE.fullmatch(line.strip()) is None:
            continue

        item_index = attribute_index + 1
        while item_index < len(code_lines):
            stripped = code_lines[item_index].strip()
            if not stripped or stripped.startswith("#["):
                item_index += 1
                continue
            break
        if item_index >= len(code_lines):
            excluded.add(attribute_index)
            continue
        if RUST_ITEM_HEAD.match(code_lines[item_index].strip()) is None:
            # Fields, variants, and match arms can also carry cfg attributes,
            # but they are comma-delimited rather than Rust items. Keeping
            # their contents in scope is conservative and cannot hide later
            # production code.
            excluded.add(attribute_index)
            continue

        opened = False
        depth = 0
        end_index = item_index
        for line_index in range(item_index, len(code_lines)):
            code = code_lines[line_index]
            if not opened and ";" in code and (
                "{" not in code or code.index(";") < code.index("{")
            ):
                end_index = line_index
                break
            if "{" in code:
                opened = True
            if opened:
                depth += code.count("{") - code.count("}")
                if depth == 0:
                    end_index = line_index
                    break
            end_index = line_index

        excluded.update(range(attribute_index, end_index + 1))
    return excluded


def production_rows(source: str) -> list[tuple[int, str]]:
    lines = source.splitlines()
    excluded = cfg_test_item_lines(source)
    return [
        (line_number, line)
        for line_number, line in enumerate(lines, 1)
        if line_number - 1 not in excluded
    ]


def production_code_projection(source: str) -> str:
    projected = rust_code_projection(source).splitlines(keepends=True)
    for line_index in cfg_test_item_lines(source):
        if line_index >= len(projected):
            continue
        projected[line_index] = "".join(
            char if char in "\r\n" else " " for char in projected[line_index]
        )
    return "".join(projected)


def production_source_rows(path: Path) -> list[tuple[int, str]]:
    return production_rows(path.read_text(encoding="utf-8"))
