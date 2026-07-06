#!/usr/bin/env bash
set -euo pipefail

out="${1:-corpus/project_gutenberg_ru.txt}"
books="${LAY_LLMWAVE_GUTENBERG_BOOKS:-8}"
top_url="${LAY_LLMWAVE_GUTENBERG_TOP_RU:-https://www.gutenberg.org/browse/scores/top-ru.php}"

mkdir -p "$(dirname "$out")"
python3 - "$top_url" "$books" "$out" <<'PY'
import re
import sys
from urllib.error import URLError
from urllib.request import Request, urlopen

top_url, books, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]

def fetch(url: str) -> str:
    req = Request(url, headers={"User-Agent": "lay-llmwave-corpus/1.0"})
    with urlopen(req, timeout=30) as response:
        data = response.read()
    for encoding in ("utf-8", "windows-1251", "latin-1"):
        try:
            return data.decode(encoding)
        except UnicodeDecodeError:
            pass
    return data.decode("utf-8", errors="replace")

html = fetch(top_url)
ids = []
seen = set()
for item in re.findall(r'href="/ebooks/([0-9]+)"', html):
    if item not in seen:
        ids.append(item)
        seen.add(item)
    if len(ids) >= books:
        break

def clean_gutenberg(text: str) -> str:
    start = re.search(r"\*\*\* START OF (?:THE|THIS) PROJECT GUTENBERG EBOOK.*?\*\*\*", text, re.I | re.S)
    end = re.search(r"\*\*\* END OF (?:THE|THIS) PROJECT GUTENBERG EBOOK.*?\*\*\*", text, re.I | re.S)
    if start and end and end.start() > start.end():
        text = text[start.end():end.start()]
    return "\n".join(
        line.strip()
        for line in text.splitlines()
        if re.search(r"[А-Яа-яЁё]", line)
    ).strip()

written = 0
with open(out, "w", encoding="utf-8") as handle:
    for ebook_id in ids:
        urls = [
            f"https://www.gutenberg.org/cache/epub/{ebook_id}/pg{ebook_id}.txt",
            f"https://www.gutenberg.org/files/{ebook_id}/{ebook_id}-0.txt",
            f"https://www.gutenberg.org/files/{ebook_id}/{ebook_id}.txt",
        ]
        text = None
        source = None
        for url in urls:
            try:
                text = fetch(url)
                source = url
                break
            except URLError:
                continue
        if not text:
            continue
        text = clean_gutenberg(text)
        if not text:
            continue
        handle.write(f"\n\n# gutenberg:{ebook_id} {source}\n")
        handle.write(text)
        handle.write("\n")
        written += 1
print(f"downloaded={written}")
PY

lines="$(wc -l < "$out" | tr -d ' ')"
bytes="$(wc -c < "$out" | tr -d ' ')"
echo "llmwave_external_corpus: output=$out books=$books lines=$lines bytes=$bytes"
