#!/usr/bin/env python3
"""Independent disposable-tree negative controls for the MyVa skills validator.

Run through ``cargo xtask verify repository`` or the implementation primitive
``python3 scripts/check-skills.py --self-test``. No production skill is mutated.
"""
from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "scripts/check-skills.py"
SPEC = importlib.util.spec_from_file_location("myva_check_skills", SCRIPT)
checks = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checks)


def valid_skill(name):
    return f"""---
name: {name}
description: >-
  A fixture engineering workflow for MyVa.
  Use when testing repository validation.
metadata:
  owner: {name}
  source_repository: {checks.SOURCE_REPOSITORY}
  source_revision: {checks.SOURCE_REVISION}
---

# Fixture

Invoke ${name} for the fixture workflow.
"""


class SkillTree:
    def __init__(self, root):
        self.root = Path(root)
        self.skill = self.root / checks.SKILLS_DIR / "myva-engineering"
        for name in checks.SKILLS:
            self.write(f".agents/skills/{name}/SKILL.md", valid_skill(name))
            self.write(f".agents/skills/{name}/agents/openai.yaml", f"""interface:
  display_name: "MyVa Fixture"
  short_description: "Exercise the MyVa skill validator"
  default_prompt: "Use ${name} for this fixture."
""")
        self.write(".agents/skills/README.md", "# Skills\n\n" + "".join(
            f"- [{name}]({name}/SKILL.md)\n" for name in checks.SKILLS))
        self.write(".agents/skills/routing-cases.md", "# Routing cases\n\n[Machine fixture](routing-cases.json).\n")
        for filename in ("AGENTS.md", "CLAUDE.md"):
            self.write(filename, "# Routing\n\n[Canonical skills](.agents/skills/README.md).\n")
        self.write(".agents/skills/myva-engineering/references/technique.md", "# Technique\n\n## Invariant\n")
        self.append(".agents/skills/myva-engineering/SKILL.md", "\n[Technique](references/technique.md#invariant).\n")
        self.cases = {"version": 1, "cases": [
            {"id": case_id, "prompt": f"Evaluate {case_id} without assuming implementation.",
             "primary": "myva-engineering", "support": [], "reason": "Engineering owns the fixture method.",
             "mutation_authorized": case_id == "review-and-fix"}
            for case_id in sorted(checks.REQUIRED_CASES)
        ]}
        self.save_cases()
        claude = self.root / checks.CLAUDE_DIR
        claude.mkdir(parents=True)
        for name in checks.SKILLS:
            os.symlink(f"../../.agents/skills/{name}", claude / name)

    def write(self, relative, text):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def append(self, relative, text):
        path = self.root / relative
        path.write_text(path.read_text(encoding="utf-8") + text, encoding="utf-8")

    def replace(self, relative, old, new):
        path = self.root / relative
        content = path.read_text(encoding="utf-8")
        if old not in content:
            raise AssertionError(f"fixture mutation did not find {old!r}")
        path.write_text(content.replace(old, new), encoding="utf-8")

    def save_cases(self):
        self.write(".agents/skills/routing-cases.json", json.dumps(self.cases))

    def errors(self):
        return checks.check_skills(self.root)


class ParseFrontmatterTests(unittest.TestCase):
    def test_folded_description_and_metadata_are_parsed(self):
        fields, errors = checks.parse_frontmatter(valid_skill("myva-engineering"))
        self.assertEqual(errors, [])
        self.assertEqual(fields["description"], "A fixture engineering workflow for MyVa. Use when testing repository validation.")
        self.assertEqual(fields["metadata"]["source_revision"], checks.SOURCE_REVISION)

    def test_missing_and_unclosed_delimiters_fail(self):
        for text, reason in (("# Missing", "line 1"), ("---\nname: x", "not closed")):
            with self.subTest(text=text):
                _, errors = checks.parse_frontmatter(text)
                self.assertTrue(any(reason in error for error in errors), errors)

    def test_quoted_and_literal_scalars_are_supported(self):
        fields, errors = checks.parse_frontmatter('---\nname: "myva-a"\ndescription: |-\n  First line\n  Second line\n---\n')
        self.assertEqual(errors, [])
        self.assertEqual(fields["name"], "myva-a")
        self.assertEqual(fields["description"], "First line\nSecond line")

    def test_duplicate_top_level_and_nested_keys_fail(self):
        for addition, reason in (("name: other\n", "duplicate frontmatter key: name"),
                                 ("metadata:\n  owner: x\n  owner: y\n", "duplicate metadata key: owner")):
            with self.subTest(reason=reason):
                _, errors = checks.parse_frontmatter("---\nname: first\n" + addition + "---\n")
                self.assertIn(reason, errors)

    def test_unsupported_yaml_does_not_silently_parse(self):
        for text in ("---\nmetadata:\n    owner: x\n---\n", "---\nname: [x]\n---\n", "---\nname: null\n---\n"):
            with self.subTest(text=text):
                self.assertTrue(checks.parse_frontmatter(text)[1])


class CheckSkillsTests(unittest.TestCase):
    SKILL = ".agents/skills/myva-engineering/SKILL.md"
    DISPLAY = ".agents/skills/myva-engineering/agents/openai.yaml"

    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.tree = SkillTree(self.directory.name)

    def tearDown(self):
        self.directory.cleanup()

    def rejected(self, reason):
        errors = self.tree.errors()
        self.assertTrue(any(reason in error for error in errors), errors)

    def test_valid_complete_tree_passes(self):
        self.assertEqual(self.tree.errors(), [])

    def test_missing_skill_directory_fails(self):
        shutil.rmtree(self.tree.skill)
        self.rejected("missing canonical real skill directory")

    def test_mismatched_name_fails(self):
        self.tree.replace(self.SKILL, "name: myva-engineering", "name: myva-other")
        self.rejected("frontmatter name must equal directory name")

    def test_empty_and_overlong_descriptions_fail(self):
        for description in ('""', "x" * (checks.MAX_DESCRIPTION + 1)):
            with self.subTest(length=len(description)):
                text = valid_skill("myva-engineering")
                start, end = text.index("description:"), text.index("metadata:")
                self.tree.write(self.SKILL, text[:start] + f"description: {description}\n" + text[end:])
                self.rejected("description must contain")

    def test_metadata_must_be_mapping(self):
        self.tree.replace(self.SKILL, "metadata:\n", "metadata: wrong\n")
        self.rejected("metadata must be a mapping")

    def test_each_provenance_field_is_enforced(self):
        for field in ("owner", "source_repository", "source_revision"):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                tree = SkillTree(directory)
                old = {"owner": "myva-engineering", "source_repository": checks.SOURCE_REPOSITORY,
                       "source_revision": checks.SOURCE_REVISION}[field]
                tree.replace(self.SKILL, f"  {field}: {old}", f"  {field}: wrong")
                self.assertTrue(any(f"metadata.{field} must equal" in error for error in tree.errors()))

    def test_unknown_frontmatter_and_excessive_skill_length_fail(self):
        self.tree.replace(self.SKILL, "metadata:\n", "allowed-tools: Bash\nmetadata:\n")
        self.rejected("unsupported frontmatter keys")
        self.tree.append(self.SKILL, "line\n" * checks.MAX_SKILL_LINES)
        self.rejected("move depth into references/")

    def test_exact_skill_invocation_is_required(self):
        self.tree.replace(self.SKILL, "$myva-engineering", "$myva-engineering-extra")
        self.rejected("missing exact $myva-engineering invocation")

    def test_missing_display_metadata_fails(self):
        (self.tree.root / self.DISPLAY).unlink()
        self.rejected("agents/openai.yaml: missing regular file")

    def test_malformed_display_metadata_fails(self):
        self.tree.write(self.DISPLAY, 'interface: {display_name: "bad"}\n')
        self.rejected("unsupported display metadata line")

    def test_missing_and_oversized_display_fields_fail(self):
        self.tree.write(self.DISPLAY, 'interface:\n  short_description: "short"\n')
        for reason in ("interface.display_name is required", "short_description must contain", "default_prompt must invoke"):
            self.rejected(reason)
        self.tree.write(self.DISPLAY, 'interface:\n  short_description: "' + "x" * 65 + '"\n')
        self.rejected("short_description must contain")

    def test_mismatched_display_invocation_fails(self):
        self.tree.replace(self.DISPLAY, "$myva-engineering", "$myva-engineering-other")
        self.rejected("interface.default_prompt must invoke $myva-engineering")

    def test_duplicate_display_keys_fail(self):
        self.tree.append(self.DISPLAY, '  display_name: "Duplicate"\n')
        self.rejected("duplicate interface key")

    def test_missing_index_entry_fails(self):
        self.tree.replace(".agents/skills/README.md", "- [myva-engineering](myva-engineering/SKILL.md)\n", "")
        self.rejected("must link myva-engineering/SKILL.md exactly once (found 0)")

    def test_duplicate_index_entry_fails(self):
        self.tree.append(".agents/skills/README.md", "[Duplicate](myva-engineering/SKILL.md)\n")
        self.rejected("exactly once (found 2)")

    def test_unknown_skill_and_noncanonical_copy_fail(self):
        self.tree.write(".agents/skills/myva-copy/SKILL.md", valid_skill("myva-copy"))
        self.rejected("unknown or duplicate skill directory")
        self.tree.write(".codex/skills/myva-engineering/SKILL.md", valid_skill("myva-engineering"))
        self.rejected("noncanonical duplicate SKILL.md")

    def test_runtime_entrypoints_must_route_to_index(self):
        self.tree.write("CLAUDE.md", "# Detached routing\n")
        self.rejected("CLAUDE.md: must link the canonical")

    def test_missing_local_link_and_missing_fragment_fail(self):
        self.tree.append(self.SKILL, "[Missing](references/missing.md)\n[Bad anchor](references/technique.md#missing)\n")
        self.rejected("missing target references/missing.md")
        self.rejected("missing heading references/technique.md#missing")

    def test_external_links_and_fenced_examples_are_not_probed(self):
        self.tree.append(self.SKILL, "[External](https://invalid.example/missing)\n```md\n[Sample](missing.md)\n```\n")
        self.assertEqual(self.tree.errors(), [])

    def test_repository_escape_fails_even_when_target_exists(self):
        self.tree.append(self.SKILL, "[Escape](../../../../outside.md)\n")
        self.rejected("link escapes repository")

    def test_reference_symlink_escape_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            outside = Path(directory) / "outside.md"
            outside.write_text("# Outside\n", encoding="utf-8")
            os.symlink(outside, self.tree.skill / "references/escape.md")
            self.tree.append(self.SKILL, "[Escape](references/escape.md)\n")
            self.rejected("canonical skill entries must not be symlinks")
            self.rejected("link escapes repository")

    def test_transitive_reference_and_duplicate_heading_anchors_pass(self):
        self.tree.append(".agents/skills/myva-engineering/references/technique.md", "\n## Invariant\n[Deeper](deeper.md#more)\n")
        self.tree.write(".agents/skills/myva-engineering/references/deeper.md", "# More\n")
        self.tree.append(self.SKILL, "[Duplicate heading](references/technique.md#invariant-1)\n")
        self.assertEqual(self.tree.errors(), [])

    def test_reference_style_link_is_checked_and_reachable(self):
        self.tree.append(self.SKILL, "[More][ref]\n\n[ref]: references/more.md#more\n")
        self.tree.write(".agents/skills/myva-engineering/references/more.md", "# More\n")
        self.assertEqual(self.tree.errors(), [])

    def test_orphan_markdown_and_data_reference_fail(self):
        for suffix in ("md", "json"):
            self.tree.write(f".agents/skills/myva-engineering/references/orphan.{suffix}", "{}\n")
            self.rejected(f"references/orphan.{suffix}: orphan reference")

    def test_fenced_link_does_not_make_reference_reachable(self):
        self.tree.replace(self.SKILL, "[Technique](references/technique.md#invariant).", "```md\n[Technique](references/technique.md#invariant).\n```")
        self.rejected("orphan reference")

    def test_missing_claude_alias_fails(self):
        (self.tree.root / checks.CLAUDE_DIR / "myva-engineering").unlink()
        self.rejected("missing symlink to ../../.agents/skills/myva-engineering")

    def test_wrong_and_absolute_claude_targets_fail(self):
        link = self.tree.root / checks.CLAUDE_DIR / "myva-engineering"
        for target in ("../../elsewhere/myva-engineering", str(self.tree.skill)):
            with self.subTest(target=target):
                link.unlink()
                os.symlink(target, link)
                self.rejected("target must be exactly '../../.agents/skills/myva-engineering'")

    def test_copied_claude_skill_and_stale_alias_fail(self):
        link = self.tree.root / checks.CLAUDE_DIR / "myva-engineering"
        link.unlink()
        shutil.copytree(self.tree.skill, link)
        self.rejected("copies are not canonical")
        self.rejected("noncanonical duplicate SKILL.md")
        os.symlink("../../.agents/skills/myva-gone", link.parent / "myva-gone")
        self.rejected("myva-gone: no matching canonical skill")

    def test_symlinked_claude_directory_fails(self):
        directory = self.tree.root / checks.CLAUDE_DIR
        shutil.rmtree(directory)
        os.symlink("../.agents/skills", directory)
        self.rejected("must be a real directory of per-skill symlinks")

    def test_canonical_skill_cannot_be_alias(self):
        shutil.rmtree(self.tree.skill)
        os.symlink("myva-logic-fix", self.tree.skill)
        self.rejected("missing canonical real skill directory")

    def test_unknown_broken_canonical_alias_fails(self):
        os.symlink("missing", self.tree.root / checks.SKILLS_DIR / "myva-gone")
        self.rejected("myva-gone: canonical root entries must not be symlinks")

    def test_routing_fixture_missing_or_empty_fails(self):
        path = self.tree.root / ".agents/skills/routing-cases.json"
        path.unlink()
        self.rejected("routing-cases.json: missing regular file")
        self.tree.cases["cases"] = []
        self.tree.save_cases()
        self.rejected("cases must be a nonempty list")

    def test_routing_version_and_duplicate_json_keys_fail(self):
        self.tree.cases["version"] = True
        self.tree.save_cases()
        self.rejected("expected object with version: 1")
        self.tree.write(".agents/skills/routing-cases.json", '{"version":1,"version":1,"cases":[]}')
        self.rejected("duplicate JSON key: version")

    def test_routing_required_id_and_unique_ids_fail(self):
        self.tree.cases["cases"] = self.tree.cases["cases"][1:]
        self.tree.cases["cases"].append(self.tree.cases["cases"][0].copy())
        self.tree.save_cases()
        self.rejected("missing required cases")
        self.rejected("duplicate case id")

    def test_routing_unknown_primary_and_support_fail(self):
        case = self.tree.cases["cases"][0]
        case["primary"], case["support"] = "myva-unknown", ["myva-gone"]
        self.tree.save_cases()
        self.rejected("primary must name a canonical skill owner")
        self.rejected("support must list canonical skill owners")

    def test_routing_self_support_and_duplicate_support_fail(self):
        case = self.tree.cases["cases"][0]
        case["support"] = ["myva-engineering", "myva-engineering"]
        self.tree.save_cases()
        self.rejected("support must be unique and exclude primary")

    def test_routing_empty_text_and_nonboolean_authorization_fail(self):
        case = self.tree.cases["cases"][0]
        case["prompt"], case["reason"], case["mutation_authorized"] = " ", "", "false"
        self.tree.save_cases()
        for reason in ("prompt must be nonempty", "reason must be nonempty", "mutation_authorized must be boolean"):
            self.rejected(reason)

    def test_routing_invalid_field_types_report_errors_without_crashing(self):
        case = self.tree.cases["cases"][0]
        case["id"], case["primary"], case["support"] = [], {}, [None]
        self.tree.save_cases()
        for reason in ("id must be a lowercase", "primary must name", "support must list"):
            self.rejected(reason)

    def test_read_only_and_authorized_fix_contracts_are_not_interchangeable(self):
        for case in self.tree.cases["cases"]:
            if case["id"] in {"read-only-review", "visual-only"}:
                case["mutation_authorized"] = True
            elif case["id"] == "review-and-fix":
                case["mutation_authorized"] = False
        self.tree.save_cases()
        for reason in ("read-only-review must not authorize", "visual-only must not authorize", "review-and-fix requires explicit"):
            self.rejected(reason)


class CommandContractTests(unittest.TestCase):
    def run_cli(self, root, *args):
        return subprocess.run([sys.executable, str(SCRIPT), "--root", str(root), *args],
                              capture_output=True, text=True, check=False)

    def test_missing_repository_is_nonzero_and_explicit(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.run_cli(Path(directory) / "missing")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("missing real canonical skills directory", result.stdout)

    def test_structural_pass_reports_runtime_routing_not_run(self):
        with tempfile.TemporaryDirectory() as directory:
            SkillTree(directory)
            result = self.run_cli(directory)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("runtime discovery/routing NOT_RUN", result.stdout)

    def test_missing_negative_test_target_is_nonzero(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.run_cli(directory, "--self-test")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("missing negative-test target", result.stderr)

    def test_empty_negative_suite_is_nonzero(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "tests/test_check_skills.py"
            path.parent.mkdir()
            path.write_text("# No tests is not a passing suite.\n", encoding="utf-8")
            result = self.run_cli(directory, "--self-test")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("empty negative-test suite", result.stderr)

    def test_unloadable_negative_suite_is_explicit_nonzero(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "tests/test_check_skills.py"
            path.parent.mkdir()
            path.write_text("raise RuntimeError('fixture cannot load')\n", encoding="utf-8")
            result = self.run_cli(directory, "--self-test")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("FAIL: cannot load negative-test target", result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
