---
name: myva-game-audit
description: >-
  Audit MyVa combat, session, equipment prototypes, replay, ECS lifecycle, assets/performance or economy invariants for exploit paths and authority leaks. Use for repeated procs, overflow, cooldown, CC budget, atomic rejection, duplicate rewards and compatibility; report without implementing new gameplay.
metadata:
  owner: myva-game-audit
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa game audit

Own the audit workflow and gameplay/economy findings report. [Engineering](../myva-engineering/SKILL.md)
owns invariant method, evidence levels, L1–L5 and common reporting. Read the
[invariant matrix](references/invariant-matrix.md) plus the touched product/code owners.
Audit is report-only unless the user explicitly requests review-and-fix or implementation.
Use this entrypoint for domain/capability audits, even read-only. A PR/range/diff review
starts with [code review](../myva-code-review/SKILL.md) and composes this audit for touched claims.

## Workflow

1. Pin scope/source SHA, diff/base if any, mode (source, tests, live) and actual capabilities.
   Read [architecture](../../../docs/technical/architecture.md), [combat](../../../docs/design/combat.md),
   [economy](../../../docs/design/economy.md), [kit/replay](../../../docs/technical/kit-registry-loadout.md)
   and [equipment](../../../docs/technical/equipment-registry.md) only as touched. Separate requirements,
   proposed mechanics, implemented prototypes and verified runtime behavior.
2. Trace each claim to its sole owner: sim rules/session/protocol/replay; gameplay ECS adapter/mirror;
   Bevy input/render/assets; Leptos/CMP shells. Online authority/economy commit belongs to a future
   server, not a client bool, simulated ledger or UI eligibility check.
3. Build the engineering ledger for attack/proc identity, resource/cap/overflow, cooldown boundaries,
   target-owned CC budget, rejected-transition atomicity, reward idempotency and replay pin compatibility.
   Audit deserialize/restore/conversion/raw-fixture and direct API construction paths, not only UI entry.
4. Apply the [matrix](references/invariant-matrix.md) at integer/tick boundaries, duplicate/reordered
   sequences and multi-source interactions. Read mutable ownership/invariants without demanding an
   entire World clone per tick. Require independent expected values and negative cases, not a test
   that calls the same calculation twice.
5. Use existing runners via [test automation](../myva-test-automation/SKILL.md): pure-core tests,
   ECS/core per-tick parity, replay checks, native/WASM determinism and browser/device checks as
   separate evidence claims. A bounded search proves only its action set, seeds, depth and model.
6. Classify findings as confirmed defect, required evidence gap, unresolved risk or non-blocking
   opportunity, citing the source requirement and consequence. Use [diagnostics](../myva-diagnostics/SKILL.md)
   for unexplained failures; authorized repairs route through [logic fix](../myva-logic-fix/SKILL.md).
   New equipment combat, networking, roster/balance, formal harnesses and toolchains are separate work.

## Audit report additions

Extend engineering once: boundary/authority map; per-invariant owner, failure mode, independent
oracle, source revision and evidence level; explored input/sequence bounds; findings with location,
expected/actual, impact/confidence and reproducible trigger; exact command/status/log; implemented
versus proposed capabilities; missing acceptance, next owner and residual risk. Runtime statuses are
only `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`. “Combat integration not implemented” describes capability,
not a fifth runner status. Do not infer exploit-proof multiplayer or balance from local prototypes.

## Invocation

Use `$myva-game-audit` and read `.agents/skills/myva-game-audit/SKILL.md`. Audit equipment guard recovery and its runtime rejection
read-only: repeated action/hit identity, cooldown, resource overflow, atomic state and replay v3/v4
compatibility. Distinguish pure-prototype evidence from unimplemented combat/server authority.
