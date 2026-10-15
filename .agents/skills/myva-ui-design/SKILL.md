---
name: myva-ui-design
description: >-
  Design or refine MyVa shell UI and combat presentation while preserving game readability, Vietnamese text, accessibility and authoritative state. Use for HUD, layout, tokens, motion and component states; compose renderer owners and inspect real pixels.
metadata:
  owner: myva-ui-design
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa UI design

Own presentation acceptance and the design/iteration workflow, not combat rules, engineering
evidence levels or platform implementation. Read [engineering](../myva-engineering/SKILL.md),
[product rubric](references/presentation-rubric.md), [combat](../../../docs/design/combat.md) and
[QA](../../../docs/production/qa.md) for the specific surface. Do not copy Cantos branding or
assume MyVa already has a shared design-token generator or Material component system.

## Design loop

1. Pin the surface, player task, source/build, actual renderer, fixture and supported input. Separate
   shell UI from Bevy gameplay: shell owns navigation/IME/settings, game owns arena/HUD/input.
2. Read existing components/styles/assets and the current owner. Identify hierarchy, state matrix,
   accessibility/localization constraints and the one outcome. Whole-screen redesign needs that scope;
   a bounded defect instead uses [UI fix](../myva-ui-fix/SKILL.md).
3. Sketch the smallest coherent change. Use semantic roles and existing typography/spacing rather
   than arbitrary literals or imported Cantos colors. Material 3 Expressive may inform shell hierarchy
   when compatible with MyVa's existing identity; never impose decorative cards on combat HUD.
4. Implement through [web](../myva-web-engineering/SKILL.md) or [CMP](../myva-cmp-engineering/SKILL.md)
   for their mechanics; keep Bevy presentation in the existing client. Read core events/mirror facts,
   never recompute eligibility, cooldown, damage, loadout permission or online balances in the view.
5. Exercise relevant loading/ready/empty/error/disabled/paused states. Preserve exact denied reasons,
   immutable kit/revision identity and honest placeholder/fallback labels. Use realistic Vietnamese
   text, long names and text scale; do not claim locale/theme support that is not implemented.
6. Capture and OPEN rendered pixels using [visual review](../myva-visual-review/SKILL.md).
   Repeat the same build/scenario/fixture/viewport/theme/locale/text-scale matrix before/after,
   with separate source identities, then recheck affected neighbors. Record concrete observations.
7. Pair images with actual keyboard/touch/semantics checks and runtime motion where relevant.
   Select existing runners via [test automation](../myva-test-automation/SKILL.md), applying engineering's
   evidence contract. Physical native gates remain independent of browser touch emulation/desktop preview.

## Acceptance and report

Use [presentation rubric](references/presentation-rubric.md) for hierarchy, state, accessibility,
localization, motion and gameplay-specific criteria. A new palette, camera behavior or control
mechanic is a proposed product change, not automatic permission from this skill.

Extend the engineering report: intent/surface/state matrix, changed semantic sources/consumers,
scenario tuple, images actually opened and visible findings, interaction/accessibility/motion results,
exact commands/statuses, target limits and residual risks. Use `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`
for runner outcomes; distinguish designed, implemented and verified in prose. No fabricated captures,
reference fidelity, device support, GPU/performance or playtest claims.

## Invocation

Use `$myva-ui-design` and read `.agents/skills/myva-ui-design/SKILL.md`. Improve the existing Leptos shell's load/error hierarchy
without changing gameplay or catalog approval; inspect Vietnamese text and opened desktop/touch-sized
renders, and report unsupported matrix cells honestly.
