#!/usr/bin/env python3
"""Validate MyVa skill structure, provenance, links, aliases and routing fixtures.

Only Python's standard library is required. PASS establishes repository structure,
not runtime skill discovery, routing behavior, application correctness or UI quality.
The portable YAML subset is deliberately small: scalar fields, folded/literal
descriptions and two-space metadata/interface mappings. Unsupported YAML fails.
Adapted from Cantos check_repository.py at SOURCE_REVISION, not its product gates.
"""
from __future__ import annotations

import argparse
from collections import Counter
import html
import importlib.util
import json
import os
from pathlib import Path
import re
import sys
import unicodedata
import unittest
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parent.parent
SOURCE_REPOSITORY = "https://github.com/loveoverflowcom/cantos"
SOURCE_REVISION = "f05b09bf24243bb99bc869f5009628294ebb15d8"
SKILLS = (
    "myva-engineering", "myva-cmp-engineering", "myva-web-engineering",
    "myva-game-audit", "myva-code-review", "myva-visual-review",
    "myva-test-automation", "myva-work-item", "myva-diagnostics",
    "myva-logic-fix", "myva-ui-fix", "myva-ui-design",
)
REQUIRED_CASES = {
    "rust-rule", "replay-compatibility", "bevy-lifecycle", "asset-performance",
    "economy", "cmp-shell", "web-e2e", "visual-only", "red-ci",
    "review-and-fix", "read-only-review",
}
SKILLS_DIR = Path(".agents/skills")
CLAUDE_DIR = Path(".claude/skills")
MAX_SKILL_LINES = 300
MAX_DESCRIPTION = 1024
TOP_KEY = re.compile(r"^([a-z][a-z0-9_-]*):(.*)$")
PAIR = re.compile(r"^  ([a-z][a-z0-9_-]*):(.*)$")
BLOCK = {">", ">-", "|", "|-"}
LINK = re.compile(r"!?\[[^\]\n]*\]\((<[^>]+>|[^)\n]+)\)")
REFERENCE = re.compile(r"^\s{0,3}\[[^\]\n]+\]:\s*(<[^>]+>|\S+)")
FENCE = re.compile(r"^\s{0,3}(`{3,}|~{3,})")
HEADING = re.compile(r"^\s{0,3}#{1,6}\s+(.+?)\s*#*\s*$")
IGNORED_DIRS = {".git", "target", "node_modules", "__pycache__", ".gradle", "build"}


def scalar(value: str) -> str:
    """Parse strings only; reject ambiguous structured YAML rather than guessing."""
    value = value.strip()
    if not value:
        raise ValueError("empty scalar")
    if value.startswith('"'):
        parsed = json.loads(value)
        if not isinstance(parsed, str):
            raise ValueError("expected a string")
        return parsed
    if value.startswith("'"):
        if not value.endswith("'") or len(value) < 2:
            raise ValueError("unclosed single-quoted scalar")
        return value[1:-1].replace("''", "'")
    if value[0] in "[{&*!|>" or value.lower() in {"null", "true", "false", "~"}:
        raise ValueError("expected a plain or quoted string")
    if ": " in value or " #" in value:
        raise ValueError("quote scalars containing ': ' or ' #'")
    return value


def parse_frontmatter(content: str) -> tuple[dict, list[str]]:
    lines = content.splitlines()
    if not lines or lines[0] != "---":
        return {}, ["frontmatter must start on line 1 with ---"]
    try:
        end = lines.index("---", 1)
    except ValueError:
        return {}, ["frontmatter is not closed with ---"]
    fields, errors = {}, []
    index = 1
    while index < end:
        line = lines[index]
        match = TOP_KEY.fullmatch(line)
        if not match:
            errors.append(f"unsupported frontmatter line {index + 1}: {line!r}")
            index += 1
            continue
        key, rest = match[1], match[2].strip()
        index += 1
        value = ""
        if key == "metadata" and not rest:
            value = {}
            while index < end and lines[index].startswith(" "):
                pair = PAIR.fullmatch(lines[index])
                if not pair:
                    errors.append(f"unsupported metadata line {index + 1}")
                else:
                    try:
                        if pair[1] in value:
                            errors.append(f"duplicate metadata key: {pair[1]}")
                        value[pair[1]] = scalar(pair[2])
                    except (ValueError, json.JSONDecodeError) as error:
                        errors.append(f"metadata.{pair[1]}: {error}")
                index += 1
        elif rest in BLOCK:
            parts = []
            while index < end and (lines[index].startswith("  ") or not lines[index].strip()):
                parts.append(lines[index].strip())
                index += 1
            value = (" " if rest.startswith(">") else "\n").join(parts).strip()
        else:
            try:
                value = scalar(rest)
            except (ValueError, json.JSONDecodeError) as error:
                errors.append(f"frontmatter {key}: {error}")
        if key in fields:
            errors.append(f"duplicate frontmatter key: {key}")
        fields[key] = value
    return fields, errors


def parse_interface(content: str) -> tuple[dict, list[str]]:
    """Read the interface section without requiring a third-party YAML parser."""
    result, errors, current, sections = {}, [], None, set()
    for number, line in enumerate(content.splitlines(), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        top = TOP_KEY.fullmatch(line)
        if top and not top[2].strip():
            current = top[1]
            if current in sections:
                errors.append(f"duplicate metadata section: {current}")
            sections.add(current)
            continue
        pair = PAIR.fullmatch(line)
        if current != "interface" or not pair:
            errors.append(f"unsupported display metadata line {number}: {line!r}")
            continue
        try:
            if pair[1] in result:
                errors.append(f"duplicate interface key: {pair[1]}")
            result[pair[1]] = scalar(pair[2])
        except (ValueError, json.JSONDecodeError) as error:
            errors.append(f"interface.{pair[1]}: {error}")
    if sections != {"interface"}:
        errors.append("display metadata must contain only an interface mapping")
    return result, errors


def prose_lines(content: str):
    fence = None
    for number, line in enumerate(content.splitlines(), 1):
        marker = FENCE.match(line)
        if marker:
            token = marker[1]
            if fence is None:
                fence = token
            elif token[0] == fence[0] and len(token) >= len(fence):
                fence = None
            continue
        if fence is None:
            yield number, line


def links(path: Path):
    for number, line in prose_lines(path.read_text(encoding="utf-8")):
        destinations = [match[1] for match in LINK.finditer(line)]
        definition = REFERENCE.match(line)
        if definition:
            destinations.append(definition[1])
        for destination in destinations:
            destination = destination.strip()
            url = destination[1:destination.index(">")] if destination.startswith("<") else destination.split()[0]
            parsed = urlsplit(url)
            if not parsed.scheme and not parsed.netloc:
                yield number, url, parsed


def heading_ids(path: Path) -> set[str]:
    counts, anchors = {}, set()
    for _, line in prose_lines(path.read_text(encoding="utf-8")):
        match = HEADING.match(line)
        if not match:
            continue
        text = re.sub(r"!?\[([^]]+)\]\([^)]*\)", r"\1", match[1])
        text = html.unescape(re.sub(r"<[^>]+>", "", text)).lower()
        slug = "".join(char for char in text if unicodedata.category(char)[0] in "LMN" or char in "-_ ").replace(" ", "-")
        repeat = counts.get(slug, 0)
        counts[slug] = repeat + 1
        anchors.add(slug + (f"-{repeat}" if repeat else ""))
    return anchors


def target_path(path: Path, parsed) -> Path:
    return (path.parent / unquote(parsed.path)).resolve() if parsed.path else path.resolve()


def read_file(path: Path, errors: list[str], label: str) -> str | None:
    if path.is_symlink() or not path.is_file():
        errors.append(f"{label}: missing regular file (symlinks are not canonical)")
        return None
    try:
        content = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        errors.append(f"{label}: cannot read UTF-8: {error}")
        return None
    if not content.strip():
        errors.append(f"{label}: empty file")
    return content


def check_skill(root: Path, name: str, errors: list[str]) -> None:
    directory = root / SKILLS_DIR / name
    label = str(SKILLS_DIR / name)
    if directory.is_symlink() or not directory.is_dir():
        errors.append(f"{label}: missing canonical real skill directory")
        return
    content = read_file(directory / "SKILL.md", errors, f"{label}/SKILL.md")
    if content is not None:
        fields, invalid = parse_frontmatter(content)
        errors.extend(f"{label}/SKILL.md: {error}" for error in invalid)
        extra = set(fields) - {"name", "description", "metadata"}
        if extra:
            errors.append(f"{label}/SKILL.md: unsupported frontmatter keys: {', '.join(sorted(extra))}")
        if fields.get("name") != name:
            errors.append(f"{label}/SKILL.md: frontmatter name must equal directory name {name!r}")
        description = fields.get("description")
        if not isinstance(description, str) or not 1 <= len(description.strip()) <= MAX_DESCRIPTION:
            errors.append(f"{label}/SKILL.md: description must contain 1–{MAX_DESCRIPTION} characters")
        metadata = fields.get("metadata")
        if not isinstance(metadata, dict):
            errors.append(f"{label}/SKILL.md: metadata must be a mapping")
        else:
            expected = {"owner": name, "source_repository": SOURCE_REPOSITORY, "source_revision": SOURCE_REVISION}
            for key, value in expected.items():
                if metadata.get(key) != value:
                    errors.append(f"{label}/SKILL.md: metadata.{key} must equal {value!r}")
        if not re.search(r"\$" + re.escape(name) + r"(?![a-z0-9-])", content):
            errors.append(f"{label}/SKILL.md: missing exact ${name} invocation")
        if len(content.splitlines()) > MAX_SKILL_LINES:
            errors.append(f"{label}/SKILL.md: exceeds {MAX_SKILL_LINES} lines; move depth into references/")
    display = read_file(directory / "agents/openai.yaml", errors, f"{label}/agents/openai.yaml")
    if display is not None:
        interface, invalid = parse_interface(display)
        errors.extend(f"{label}/agents/openai.yaml: {error}" for error in invalid)
        if not interface.get("display_name", "").strip():
            errors.append(f"{label}/agents/openai.yaml: interface.display_name is required")
        if not 25 <= len(interface.get("short_description", "")) <= 64:
            errors.append(f"{label}/agents/openai.yaml: interface.short_description must contain 25–64 characters")
        if not re.search(r"\$" + re.escape(name) + r"(?![a-z0-9-])", interface.get("default_prompt", "")):
            errors.append(f"{label}/agents/openai.yaml: interface.default_prompt must invoke ${name}")
    allowed = {"SKILL.md", "agents", "references", "scripts", "scenarios", "evaluations"}
    for entry in sorted(directory.iterdir()):
        if entry.name not in allowed:
            errors.append(f"{label}: unexpected skill entry {entry.name!r}")
    for path in sorted(directory.rglob("*")):
        if path.is_symlink():
            errors.append(f"{path.relative_to(root)}: canonical skill entries must not be symlinks")


def check_links(root: Path, documents: list[Path], errors: list[str]) -> dict[Path, set[Path]]:
    graph, anchors = {}, {}
    for path in documents:
        graph[path.resolve()] = set()
        try:
            for number, url, parsed in links(path):
                prefix = f"{path.relative_to(root)}:{number}"
                try:
                    target = target_path(path, parsed)
                except (OSError, RuntimeError) as error:
                    errors.append(f"{prefix}: invalid target {url}: {error}")
                    continue
                if not target.is_relative_to(root):
                    errors.append(f"{prefix}: link escapes repository: {url}")
                    continue
                if not target.exists():
                    errors.append(f"{prefix}: missing target {url}")
                    continue
                graph[path.resolve()].add(target)
                if parsed.fragment and target.suffix.lower() == ".md" and target.is_file():
                    if target not in anchors:
                        anchors[target] = heading_ids(target)
                    if unquote(parsed.fragment) not in anchors[target]:
                        errors.append(f"{prefix}: missing heading {url}")
        except (OSError, UnicodeError, ValueError) as error:
            errors.append(f"{path.relative_to(root)}: cannot inspect links: {error}")
    return graph


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def check_routing(root: Path, errors: list[str]) -> int:
    path = root / SKILLS_DIR / "routing-cases.json"
    label = str(SKILLS_DIR / "routing-cases.json")
    content = read_file(path, errors, label)
    if content is None:
        return 0
    try:
        data = json.loads(content, object_pairs_hook=unique_object)
    except (ValueError, json.JSONDecodeError) as error:
        errors.append(f"{label}: invalid routing JSON: {error}")
        return 0
    if not isinstance(data, dict) or type(data.get("version")) is not int or data.get("version") != 1:
        errors.append(f"{label}: expected object with version: 1")
        return 0
    if set(data) != {"version", "cases"}:
        errors.append(f"{label}: allowed top-level keys are version and cases")
    cases = data.get("cases")
    if not isinstance(cases, list) or not cases:
        errors.append(f"{label}: cases must be a nonempty list")
        return 0
    seen = set()
    for number, case in enumerate(cases, 1):
        prefix = f"{label}: case {number}"
        if not isinstance(case, dict):
            errors.append(f"{prefix}: expected a case object")
            continue
        required = {"id", "prompt", "primary", "support", "reason", "mutation_authorized"}
        if set(case) != required:
            errors.append(f"{prefix}: keys must be {', '.join(sorted(required))}")
        case_id = case.get("id")
        if not isinstance(case_id, str) or not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", case_id):
            errors.append(f"{prefix}: id must be a lowercase hyphenated string")
        elif case_id in seen:
            errors.append(f"{prefix}: duplicate case id {case_id}")
        else:
            seen.add(case_id)
        for field in ("prompt", "reason"):
            if not isinstance(case.get(field), str) or not case[field].strip():
                errors.append(f"{prefix}: {field} must be nonempty text")
        primary = case.get("primary")
        if not isinstance(primary, str) or primary not in SKILLS:
            errors.append(f"{prefix}: primary must name a canonical skill owner")
        support = case.get("support")
        if not isinstance(support, list) or any(not isinstance(name, str) or name not in SKILLS for name in support):
            errors.append(f"{prefix}: support must list canonical skill owners")
        elif len(set(support)) != len(support) or primary in support:
            errors.append(f"{prefix}: support must be unique and exclude primary")
        authorized = case.get("mutation_authorized")
        if type(authorized) is not bool:
            errors.append(f"{prefix}: mutation_authorized must be boolean")
        elif isinstance(case_id, str) and case_id in {"read-only-review", "visual-only"} and authorized:
            errors.append(f"{prefix}: {case_id} must not authorize mutation")
        elif case_id == "review-and-fix" and not authorized:
            errors.append(f"{prefix}: review-and-fix requires explicit mutation authorization")
    missing = REQUIRED_CASES - seen
    if missing:
        errors.append(f"{label}: missing required cases: {', '.join(sorted(missing))}")
    return len(cases)


def check_skills(root: Path = ROOT) -> list[str]:
    root = root.resolve()
    errors = []
    skills = root / SKILLS_DIR
    if (root / ".agents").is_symlink() or skills.is_symlink() or not skills.is_dir():
        return [f"{SKILLS_DIR}: missing real canonical skills directory"]
    for name in SKILLS:
        check_skill(root, name, errors)
    for entry in sorted(skills.iterdir()):
        if entry.is_symlink():
            errors.append(f"{SKILLS_DIR / entry.name}: canonical root entries must not be symlinks")
        if entry.is_dir() and entry.name not in SKILLS:
            errors.append(f"{SKILLS_DIR / entry.name}: unknown or duplicate skill directory")
    index = skills / "README.md"
    read_file(index, errors, str(SKILLS_DIR / "README.md"))
    read_file(skills / "routing-cases.md", errors, str(SKILLS_DIR / "routing-cases.md"))
    for filename in ("AGENTS.md", "CLAUDE.md"):
        read_file(root / filename, errors, filename)
    documents = sorted(path for path in skills.rglob("*.md") if path.is_file() and not path.is_symlink())
    documents += [root / filename for filename in ("AGENTS.md", "CLAUDE.md") if (root / filename).is_file()]
    graph = check_links(root, documents, errors)
    if index.is_file():
        indexed = Counter(target_path(index, parsed) for _, _, parsed in links(index))
        for name in SKILLS:
            target = (skills / name / "SKILL.md").resolve()
            if indexed[target] != 1:
                errors.append(f"{SKILLS_DIR}/README.md: must link {name}/SKILL.md exactly once (found {indexed[target]})")
    for filename in ("AGENTS.md", "CLAUDE.md"):
        path = (root / filename).resolve()
        if path.is_file() and index.resolve() not in graph.get(path, set()):
            errors.append(f"{filename}: must link the canonical .agents/skills/README.md index")
    for name in SKILLS:
        directory = skills / name
        entry = (directory / "SKILL.md").resolve()
        reached, pending = {entry}, [entry]
        while pending:
            for target in graph.get(pending.pop(), set()):
                if target not in reached and target.is_relative_to(directory.resolve()):
                    reached.add(target)
                    pending.append(target)
        references = directory / "references"
        if references.is_dir():
            for path in sorted(references.rglob("*")):
                if path.is_file() and not path.is_symlink() and path.resolve() not in reached:
                    errors.append(f"{path.relative_to(root)}: orphan reference not reachable from its SKILL.md")
    claude = root / CLAUDE_DIR
    if (root / ".claude").is_symlink() or claude.is_symlink() or not claude.is_dir():
        errors.append(f"{CLAUDE_DIR}: must be a real directory of per-skill symlinks")
    else:
        for name in SKILLS:
            path = claude / name
            expected = f"../../.agents/skills/{name}"
            if not path.is_symlink():
                errors.append(f"{CLAUDE_DIR / name}: missing symlink to {expected}; copies are not canonical")
            elif os.readlink(path) != expected:
                errors.append(f"{CLAUDE_DIR / name}: target must be exactly {expected!r}")
            elif not path.is_dir():
                errors.append(f"{CLAUDE_DIR / name}: broken skill symlink")
        for path in sorted(claude.iterdir()):
            if path.name not in SKILLS:
                errors.append(f"{CLAUDE_DIR / path.name}: no matching canonical skill")
    # Do not follow Claude aliases or generated trees. A second live SKILL.md is
    # not a harmless copy: it can be discovered independently and drift.
    for directory, children, files in os.walk(root, followlinks=False):
        children[:] = [name for name in children if name not in IGNORED_DIRS]
        if "SKILL.md" in files:
            path = Path(directory) / "SKILL.md"
            expected = skills / path.parent.name / "SKILL.md"
            if path.parent.name not in SKILLS or path != expected:
                errors.append(f"{path.relative_to(root)}: noncanonical duplicate SKILL.md")
    check_routing(root, errors)
    return errors


def self_test(root: Path) -> int:
    path = root / "tests/test_check_skills.py"
    if not path.is_file():
        print(f"FAIL: missing negative-test target {path}", file=sys.stderr)
        return 1
    spec = importlib.util.spec_from_file_location("myva_skill_negative_tests", path)
    if spec is None or spec.loader is None:
        print(f"FAIL: cannot load negative-test target {path}", file=sys.stderr)
        return 1
    try:
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        suite = unittest.defaultTestLoader.loadTestsFromModule(module)
    except Exception as error:
        print(f"FAIL: cannot load negative-test target {path}: {error}", file=sys.stderr)
        return 1
    count = suite.countTestCases()
    if not count:
        print(f"FAIL: empty negative-test suite {path}", file=sys.stderr)
        return 1
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    print(f"{'PASS' if result.wasSuccessful() else 'FAIL'}: {count} skill validator tests")
    return int(not result.wasSuccessful())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT, help="repository root (defaults to script parent)")
    parser.add_argument("--self-test", action="store_true", help="run the required nonempty negative-test suite")
    args = parser.parse_args()
    try:
        if args.self_test:
            return self_test(args.root.resolve())
        errors = check_skills(args.root)
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        print(f"FAIL: cannot validate skill repository: {error}", file=sys.stderr)
        return 1
    for error in errors:
        print(error)
    print(f"{'FAIL' if errors else 'PASS'}: {len(SKILLS)} canonical skills; structure only, runtime discovery/routing NOT_RUN")
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
