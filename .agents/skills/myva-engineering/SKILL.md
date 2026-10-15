---
name: myva-engineering
description: >-
  Invariant-first engineering foundation for MyVa Rust simulation, economy, Bevy ECS,
  replay/contracts, platform adapters and tooling. Owns architecture method, typed errors,
  construction barriers, L1–L5 verification, evidence vocabulary and the completion report.
  Use for code, tests, refactoring, verification or technical review; compose task and
  platform skills without duplicating their workflows.
metadata:
  owner: myva-engineering
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa engineering

This is the single method and evidence owner, adapted from pinned Cantos engineering and
behavior guidance. Read references only for the changed claim. Product truth remains in
[README](../../../README.md), [CONTRIBUTING](../../../CONTRIBUTING.md), the
[work plan](../../../docs/work-plan/README.md) and owning documents/code.

## Invocation

Use `$myva-engineering` for a Rust rule, replay, adapter, economy, asset or tooling change.
Example: “Use `$myva-engineering` to fix rejected loadout atomicity; retain v3/v4 replay
compatibility and show the independent regression oracle.” For a whole issue compose
[work-item](../myva-work-item/SKILL.md); for investigation use
[diagnostics](../myva-diagnostics/SKILL.md), then the authorized fix workflow.

## Required order

1. Read the owning requirement, work item, current code and callers. Record **invariant,
   failure mode, owner, cheapest independent oracle, source revision and evidence level**.
   Separate proposed design, implemented behavior and verified scope.
2. Apply [architecture and invariants](references/architecture-and-invariants.md): `sim` owns
   rules; `gameplay` adapts/mirrors; Bevy renders; Leptos/CMP own shell. No second combat
   reducer, speculative server, engine migration or new gameplay in a skills/tooling task.
3. Apply [types and errors](references/types-and-errors.md): fallible construction, closed
   outcomes, exhaustive matching and unchanged rejected state. Audit deserialize, restore,
   conversion and fixture paths, not just the public factory.
4. Follow [L1–L5 and evidence](references/verification.md) per claim. Complete core obligations
   before new effectful wiring; adapter-only work may start at L4 with a reason. For a bug,
   demonstrate the old failure with the regression before fixing it where feasible.
5. Select existing runners with [test automation](../myva-test-automation/SKILL.md) and follow
   [local execution](references/local-execution.md). `cargo xtask` is the single repository
   surface; dry-run, zero tests or missing tools never establish acceptance.
6. Report actual failures separately from environment blockers and deferred acceptance.
   A passing layer does not promote another layer or establish device/runtime acceptance.

## Non-negotiables

- Settled catalog/revision/loadout/replay facts are immutable. Owned mutable simulation,
  private algorithm state and ECS bookkeeping are legitimate; do not clone `World` each tick
  to imitate document-style immutability.
- Preserve fixed-width integer/tick/RNG/order/hash contracts in the deterministic core.
  Renderer transforms, interpolation and layout may use floats without feeding them into rules.
- Keep exact revision pins, frozen v3 bytes and v4 contract checks. Presentation metadata
  never silently changes gameplay hashes, entitlement or authority.
- A snapshot view is not a restore format. A typed wrapper is a checked barrier, not a proof.
- Prefer the cheapest semantic test; independent oracles must not recompute expected results
  with the same function. Mutation, Kani and Quint remain **proposed** for MyVa unless a real
  production seam/model, runner, negative controls and attributable run exist; see
  [selection and adoption](references/mutation-and-formal.md).
- Small readable functions, explicit imports, early typed rejection and comments explaining
  why. No expected caller-input flow through panic, `unwrap`, `expect`, unchecked indexing,
  erased errors or default-success recovery. Existing code is not automatically compliant.

## Composition

- [Game audit](../myva-game-audit/SKILL.md) supplies exploit/authority scenarios.
- [Code review](../myva-code-review/SKILL.md) owns read-only findings and review-and-fix scope.
- [CMP](../myva-cmp-engineering/SKILL.md) and [web](../myva-web-engineering/SKILL.md) own platform
  mechanics, not another evidence ladder.
- [UI design](../myva-ui-design/SKILL.md), [UI fix](../myva-ui-fix/SKILL.md) and
  [visual review](../myva-visual-review/SKILL.md) own visual intent/workflow; opened pixels,
  interaction and device behavior remain distinct evidence.

## Completion report

Use the canonical [report fields](references/verification.md#completion-report) for every
technical handoff, including scoped passes, exact commands, source/fixture identity,
actual failures, deferred gates and a nonempty residual-risk statement. Validator PASS
proves structural skills integrity only, not runtime discovery or routing correctness.
