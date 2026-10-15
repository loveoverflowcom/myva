---
name: myva-diagnostics
description: >-
  Diagnose one MyVa failing gate, flaky test, build error, runtime anomaly or long log using summary-first reads, a discriminating hypothesis and baseline comparison. Report cause and classification without silently repairing or hiding failures.
metadata:
  owner: myva-diagnostics
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa diagnostics

Own the investigation workflow and Diagnosis report. The invariant, evidence vocabulary, L1–L5
and common report stay with [myva-engineering](../myva-engineering/SKILL.md); platform mechanics
stay with [web](../myva-web-engineering/SKILL.md) or [CMP](../myva-cmp-engineering/SKILL.md).
Read [run record and triage](references/run-record.md) for bounded reads, capture and classification.

## Workflow

1. Pin the question, source/base SHA, dirty tree, exact failing command, exit code and build/target.
   Preserve full stdout/stderr in ignored `target/xtask-evidence/diagnostics/<run-id>/` after
   `git check-ignore` confirms the location. Keep command activity, verification and runtime logs separate.
2. Start with summary: stage statuses, failed tests, first meaningful error and output size.
   Read one bounded section next, not the entire log. `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN` are the
   [runner's statuses](../../../docs/architecture/xtask-build-contract.md), not classifications.
3. Name a normalized failure signature and classify baseline/pre-existing, environment, flaky,
   regression, test defect or unknown using evidence. Record every real failure even if another
   stage is blocked. Old reports describe their revision, never the current checkout automatically.
4. Write one hypothesis, its predicted observation and the smallest discriminating check. Start
   with `cargo xtask doctor` for a prerequisite hypothesis, or the owning focused verify target.
   Compare an isolated baseline only when that comparison changes the next action.
5. Increase depth only for a written hypothesis: a bounded backtrace, one test's output,
   browser console/protocol trace, or a narrowly marked temporary adapter probe. Keep sim decisions
   observable through return values/tests rather than introducing I/O into the pure core.
6. Confirm or refute, record raw-log pointer and choose a new discriminating check. Bounded flaky
   reruns may measure variation; rerun-until-green, arbitrary sleeps, skipping failures, broad
   speculative patches and altered golden expectations are not diagnosis.
7. Hand a confirmed bounded cause to [logic fix](../myva-logic-fix/SKILL.md) or
   [UI fix](../myva-ui-fix/SKILL.md) only when repair is authorized. Larger scope goes to
   [work item](../myva-work-item/SKILL.md). Remove temporary probes, restore verbosity and stop
   only owned processes/worktrees. No root cause is preferable to a fabricated one.

## Diagnosis report

Extend the engineering report with: observed/expected contract, signature, classification plus
oracle, confirmed cause versus hypothesis/confidence, commands and all printed statuses, baseline
SHA/environment comparison, hypothesis outcomes, raw-log pointers, loop progress, cleanup oracle,
next owner/action and residual risk. A missing capability is described as not implemented; do not
invent a runner status such as `UNSUPPORTED`, `SKIPPED` or `NOT_IMPLEMENTED` for MyVa.

## Invocation

Use `$myva-diagnostics` and read `.agents/skills/myva-diagnostics/SKILL.md`. Diagnose the red web gate summary-first, distinguish
an environment blocker from a regression against the pinned base, retain raw logs under target,
and report the smallest justified next action without changing gameplay.
