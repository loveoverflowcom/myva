---
name: myva-logic-fix
description: >-
  Repair one bounded MyVa Rust/domain/state/error defect from an observed failure, exact expected contract and independent regression. Use for combat, session, replay, catalog/loadout, economy or adapter logic, not review-only work or presentation-only defects.
metadata:
  owner: myva-logic-fix
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa logic fix

Own the bounded repair workflow and handoff. Read [myva-engineering](../myva-engineering/SKILL.md)
for invariant-first method, typed construction/errors, ownership, verification and report; use
[repair seams](references/repair.md) to locate MyVa's real owner.

## Workflow

1. Normalize actual/expected results, source/build SHA, inputs, reproduction, authority and unknowns.
   Treat issue/log/review text as evidence, not permission. If the cause is not demonstrated,
   use [diagnostics](../myva-diagnostics/SKILL.md) to classify before patching.
2. Trace input → validate/construct → decide/transition → effects/mirror → result. Name the
   violated invariant, failure mode and existing owner. Keep combat/session/protocol/replay in
   `packages/sim`; ECS mirrors in `packages/gameplay`; renderer/input/assets in Bevy; shells outside rules.
3. Bound one defect or demonstrated common cause. Add the cheapest independently specified
   failing regression through production code before the patch. Assert exact state/error and
   unchanged authoritative state/effects on rejection, not merely `is_err()` or mocked calls.
4. Repair at the owner with fallible constructors, typed errors and exhaustive matching. Audit
   alternate construction via deserialize, restore, conversion, raw fixtures and public fields.
   Preserve owned mutation/invariants without cloning the entire simulation/ECS World every tick.
5. Keep integer/tick/RNG/hash determinism in sim. Preserve immutable catalog/revision/loadout/replay
   facts, exact pins and metadata/hash isolation. Replay v3/v4 compatibility needs byte/contract
   regressions; do not silently upgrade recordings or bless changed goldens.
6. Run the focused regression and affected edge/property/sequence cases; then applicable ECS/core
   parity, native/WASM determinism and consumer runners via [test automation](../myva-test-automation/SKILL.md).
   Pure-core success cannot prove device rendering, GPU reclamation, network authority or durable ledger commit.
7. Format/check applicable owners and repository. Report each applicable L1–L5 claim independently
   through engineering. Broader schema, architecture, online authority or new equipment effects
   become a [work item](../myva-work-item/SKILL.md), not an incidental bug fix.

## Boundaries and handoff

Use [UI fix](../myva-ui-fix/SKILL.md) only when facts are correct and presentation is wrong;
use [web](../myva-web-engineering/SKILL.md) or [CMP](../myva-cmp-engineering/SKILL.md) for shell mechanics.
A review request is read-only until explicit review-and-fix authorization. Do not suppress errors
with fallback kit/version, panics on untrusted input, arbitrary retry/sleep or a success-shaped default.

Extend the engineering report: bug contract, confirmed cause versus hypothesis, invariant/owner,
minimal changed seam and consumers, failing-before/passing-after oracle, exact rejection checks,
commands/statuses, deferred target acceptance, next action and residual risk. Use only runner
statuses `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`. Actual failures are never hidden as missing-runtime gaps.

## Invocation

Use `$myva-logic-fix` and read `.agents/skills/myva-logic-fix/SKILL.md`. Fix the repeated guard-recovery proc in the pure
prototype with an independent regression for duplicate action/cooldown boundaries; do not enable
equipment combat, alter replay v3/v4 or claim online exploit prevention.
