#!/usr/bin/env python3
"""Check repository Markdown inline links, images and local heading fragments.

Uses only Python's standard library. Ignores fenced code and external URLs;
network reachability and Mermaid syntax are separate checks. With no arguments,
checks README.md, CONTRIBUTING.md and docs/**/*.md. Additional files/directories
can be passed on the command line.
"""
from __future__ import annotations

import argparse
import html
from pathlib import Path
import re
import unicodedata
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parent.parent
LINK = re.compile(r"!?\[[^\]\n]+\]\((<[^>]+>|[^)\n]+)\)")
FENCE = re.compile(r"^\s{0,3}(`{3,}|~{3,})")
HEADING = re.compile(r"^\s{0,3}#{1,6}\s+(.+?)\s*#*\s*$")


def prose_lines(text: str):
    """Skip fenced examples so sample paths/headings are not treated as docs."""
    fence = None
    for number, line in enumerate(text.splitlines(), 1):
        match = FENCE.match(line)
        if match:
            marker = match[1]
            if fence is None:
                fence = marker
            elif marker[0] == fence[0] and len(marker) >= len(fence):
                fence = None
            continue
        if fence is None:
            yield number, line


def heading_ids(path: Path) -> set[str]:
    counts: dict[str, int] = {}
    anchors = set()
    for _, line in prose_lines(path.read_text(encoding="utf-8")):
        match = HEADING.match(line)
        if not match:
            continue
        heading = re.sub(r"!?\[([^]]+)\]\([^)]*\)", r"\1", match[1])
        heading = html.unescape(re.sub(r"<[^>]+>", "", heading)).lower()
        slug = "".join(
            char for char in heading
            if unicodedata.category(char)[0] in "LMN" or char in "-_ "
        ).replace(" ", "-")
        repeat = counts.get(slug, 0)
        counts[slug] = repeat + 1
        anchors.add(slug + (f"-{repeat}" if repeat else ""))
    return anchors


def check(files: list[Path]) -> tuple[int, list[str]]:
    errors: list[str] = []
    anchors: dict[Path, set[str]] = {}
    checked = 0
    for path in files:
        for line_number, line in prose_lines(path.read_text(encoding="utf-8")):
            for match in LINK.finditer(line):
                destination = match[1].strip()
                # Angle-bracket destinations may include spaces; normal links
                # can have an optional quoted title after the path.
                url = destination[1:-1] if destination.startswith("<") else destination.split()[0]
                parsed = urlsplit(url)
                if parsed.scheme or parsed.netloc:
                    continue
                checked += 1
                target = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
                label = str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path)
                prefix = f"{label}:{line_number}"
                if not target.exists():
                    errors.append(f"{prefix}: missing target {url}")
                elif parsed.fragment and target.suffix.lower() == ".md":
                    if target not in anchors:
                        anchors[target] = heading_ids(target)
                    if unquote(parsed.fragment) not in anchors[target]:
                        errors.append(f"{prefix}: missing heading {url}")
    return checked, errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path)
    args = parser.parse_args()
    requested = args.paths or [ROOT / "README.md", ROOT / "CONTRIBUTING.md", ROOT / "docs"]
    files: set[Path] = set()
    for path in requested:
        path = path.resolve()
        if not path.exists():
            parser.error(f"Path does not exist: {path}")
        if path.is_dir():
            files.update(path.rglob("*.md"))
        else:
            files.add(path)
    checked, errors = check(sorted(files))
    for error in errors:
        print(error)
    print(f"{'FAIL' if errors else 'PASS'}: {checked} local links/anchors in {len(files)} Markdown files")
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
