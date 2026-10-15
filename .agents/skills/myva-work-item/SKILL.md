---
name: myva-work-item
description: >-
  Deliver one MyVa issue or work-plan slice from prerequisites and a bounded outcome through implementation, evidence and an authorized develop-targeted PR. Use to start, continue, implement or ship planned work, not to widen an audit into a feature.
metadata:
  owner: myva-work-item
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa work item

Own the delivery order and handoff, not engineering rules. Read
[myva-engineering](../myva-engineering/SKILL.md) for method, evidence vocabulary,
L1–L5 and the common report. Read [delivery](references/delivery.md) when slicing or publishing.

## Workflow

1. Read the [queue](../../../docs/work-plan/README.md), [planning guide](../../../docs/work-plan/planning.md),
   item, linked issue, owning docs and actual code. Record source SHA, base and dirty paths.
   GitHub owns live execution status; Markdown owns scope and dependencies. Unreadable status is unknown.
2. Verify the immediate prerequisites for this slice, with acceptance evidence. A merge is not
   automatically a passed platform gate. An explicitly scoped prototype may proceed without
   claiming the blocked consumer; report the prerequisite and exclusion before dependent work.
3. Write one review boundary: outcome, consumers, non-goals, acceptance oracle and remaining gate.
   Separate new gameplay/equipment effects, online services and toolchain changes into work items.
4. Inspect the working tree before branching. Fetch the latest `develop` when network permits;
   pin the actual base. Never reset, stash or discard unrelated edits, or silently substitute a stale base.
5. Load [engineering](../myva-engineering/SKILL.md) and only the needed owners:
   [web](../myva-web-engineering/SKILL.md), [CMP](../myva-cmp-engineering/SKILL.md),
   [UI design](../myva-ui-design/SKILL.md), or [game audit](../myva-game-audit/SKILL.md).
   Use [diagnostics](../myva-diagnostics/SKILL.md) before patching an unexplained red gate.
6. Seed the engineering evidence ledger from every in-boundary acceptance criterion. Keep claim,
   owner, failure mode, independent oracle, source revision and evidence level attributable.
   Implement in invariant-first order at the actual owner, not in a duplicate ECS/UI rule.
7. Select and execute existing runners through [test automation](../myva-test-automation/SKILL.md)
   and the [xtask contract](../../../docs/architecture/xtask-build-contract.md).
   Actual failures stay visible separately from unrun platform acceptance.
8. Update owning docs when facts or contracts change. Preserve historical evidence and distinguish
   proposed, implemented and verified. Update queue scope/dependencies only when they change;
   do not create a second live status board. Request a separate read-only [review](../myva-code-review/SKILL.md).
9. Commit, push or open/update a PR only within explicit user authorization. Target `develop`;
   use `Part of #N` for a partial issue, not a closing keyword. This skill grants no issue comments,
   labels, board changes, deployments, purchases or new external access.

## Handoff additions

Extend the engineering report once: item/issue, review boundary, prerequisite evidence, branch/base,
criterion-to-evidence disposition, changed docs/queue, authorized outward actions, deferred work and
next owner. Use exact runner statuses `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`; explain applicability and
missing implementation in prose, not invented statuses. A structure validator does not prove skill
discovery or routing quality. End with residual risks, including target/device/playtest gaps.

## Invocation

Use `$myva-work-item` and read `.agents/skills/myva-work-item/SKILL.md`. Deliver issue #7 as one skills/tooling slice from the
latest develop; preserve replay and gameplay behavior, run actual checks, and stop before commit
unless I authorize publication.
