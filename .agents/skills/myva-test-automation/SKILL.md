---
name: myva-test-automation
description: >-
  Select and execute MyVa existing xtask doctor/build/verify/run/ci runners for a scoped claim, diagnose failures and hand off attributable results. Use for focused regression, E2E, replay, determinism and CI work; do not redefine engineering verification or invent missing capabilities.
metadata:
  owner: myva-test-automation
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa test automation

Own runner selection/execution and the run handoff, not a second evidence framework.
[Engineering](../myva-engineering/SKILL.md) owns invariants, independent oracles, evidence vocabulary,
L1–L5, proportional verification and the common report. Read [runner selection](references/runner-selection.md)
and the canonical [xtask contract](../../../docs/architecture/xtask-build-contract.md).

## Execution workflow

1. Pin source/base SHA, dirty state, claim/owner/failure mode/oracle, fixtures/seeds/bounds, target
   and intended evidence level from engineering. Inspect manifests, actual xtask CLI/planner and
   current tests before naming a capability. Existing code and old reports are not current run evidence.
2. Use `cargo xtask doctor` or `doctor mobile --platform android|ios` for prerequisites; doctor is
   read-only discovery, not gameplay/device validation. Do not auto-install targets, browser, npm,
   wasm-bindgen, SDKs or a new toolchain to make a claim runnable.
3. Select the cheapest existing focused `verify` target, then affected consumers and changed-only/full
   contract as required. Keep build, run and verification separate. Use [selection table](references/runner-selection.md)
   and [web](../myva-web-engineering/SKILL.md)/[CMP](../myva-cmp-engineering/SKILL.md) for platform mechanics.
   Single Surface Rule: if xtask expresses an operation, call xtask, not duplicate Cargo/npm/script orchestration.
4. Capture raw stdout/stderr/exit and fresh artifacts under ignored `target/xtask-evidence`,
   `target/wasm-determinism` or `target/web-evidence`; confirm ignore and preserve historical evidence.
   Respect configured jobs and owned process cleanup. Never kill an unrelated server/ADB/Gradle daemon.
5. Verify a nonempty selected suite actually executed against the intended build/fixture. Missing
   required capability/prerequisite and empty/no-executed suite must be explicit nonzero failures or
   blockers, never PASS. A dry-run/list/discovery/compile-only result does not substitute for execution.
6. Retain exact MyVa statuses `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`. Stages after a failure remain
   unrun. A capability described by a CLI as `NOT_IMPLEMENTED` is prose/capability output, not an
   orchestrator status. Diagnose with [diagnostics](../myva-diagnostics/SKILL.md); never retry until green,
   silently filter/skip required cases, rewrite goldens or use an old report to pass a failed run.
7. Report claim-specific results through engineering; do not aggregate core, ECS, native/WASM,
   browser interaction, opened pixels, CPU assets, GPU allocation/performance, desktop preview,
   simulator and physical-device evidence into one “verified”. [Visual review](../myva-visual-review/SKILL.md)
   requires opening actual pixels; a screenshot path alone is not inspection.

## Handoff additions

Extend the engineering report: selected command and why, actual selected/executed/skipped counts,
source/build/environment/fixture identity, exact status/exit/raw-log and artifact pointers, failure
classification, required target acceptance gaps, cleanup, next owner and residual risk. Any repair,
new runner, mutation/Kani/Quint gate, software installation or CI configuration change needs its
own authorized scope. This workflow does not import Cantos pins, services or formal capabilities.

## Invocation

Use `$myva-test-automation` and read `.agents/skills/myva-test-automation/SKILL.md`. Run the existing focused replay/core and ECS
checks for this change, then native/WASM determinism if available and changed-only preflight.
Report actual nonempty execution and blockers; do not install missing software or claim GPU/device coverage.
