---
name: myva-ui-fix
description: >-
  Repair one bounded MyVa layout, label, focus, touch, accessibility or presentation-mapping defect with a minimal owner-local patch and opened same-scenario before/after renders. Not for report-only inspection, domain-rule changes or whole-screen redesign.
metadata:
  owner: myva-ui-fix
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa UI fix

Own the repair order and UI-fix report. [Engineering](../myva-engineering/SKILL.md) owns method,
evidence and L1–L5; [UI design](../myva-ui-design/SKILL.md) owns presentation acceptance;
[web](../myva-web-engineering/SKILL.md) and [CMP](../myva-cmp-engineering/SKILL.md) own platform mechanics.

## Workflow

1. Establish actual/expected behavior and source/build/target; infer the owning region from
   supplied evidence and code rather than demand a filename. Record unknowns and screenshot age.
   Use [diagnostics](../myva-diagnostics/SKILL.md) if an error/log does not show the cause.
2. Trace the mounted screen → state/presentation mapping → string/style/input source. Confirm
   core facts are correct. Wrong resource, cooldown, eligibility or transition goes to
   [logic fix](../myva-logic-fix/SKILL.md); never hide it with UI conditions.
3. Pin one defect and its existing owner/consumers. For Bevy HUD/VFX, read authoritative mirror/events;
   for Leptos/CMP, keep navigation/IME/host ownership in the shell. Use
   [repair checklist](references/repair.md) for a focused scope.
4. Capture and OPEN the baseline pixels when a renderer exists. Pin scenario, fixture, state,
   viewport, theme, locale, text scale, input and renderer using
   [visual scenarios](../myva-visual-review/references/scenarios.md). Add the cheapest deterministic
   presentation/interaction regression or explicitly record why a pixel-only change needs manual evidence.
5. Patch the existing component, mapping, string or semantic style source. Avoid unrelated redesign,
   dependencies, generated output edits, duplicate state owners and changes to hitboxes/rules.
6. Run focused mapping/DOM/semantics/input checks and capture/OPEN the same scenario after the fix.
   Inspect actual pixels, not an image path or automated success marker. Recheck neighboring states,
   shared consumers, long Vietnamese labels, keyboard/focus, touch cancel and reduced motion as affected.
7. Run relevant existing xtask gates through [test automation](../myva-test-automation/SKILL.md).
   If render/device access is unavailable, deliver the verified source/regression subset and name
   visual/device acceptance `BLOCKED` or `NOT_RUN` with the actual prerequisite. Compilation is not visual PASS.

## Handoff

Extend the engineering report once: bug and expected contract, owner/cause/confidence, minimal files,
consumer/state impact, exact tests/statuses, opened before/after image identities and visible observations,
scenario comparability, interaction/accessibility checks, unrun target acceptance and residual risk.
Keep real `FAIL` separate from absent runtime; report only `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN` as
runner statuses. A screenshot cannot prove motion, operability, screen-reader use or durability.
Review-only findings do not authorize edits, and this workflow authorizes no commit/push/PR by itself.

## Invocation

Use `$myva-ui-fix` and read `.agents/skills/myva-ui-fix/SKILL.md`. Fix the clipped Vietnamese cooldown label in the existing
Bevy HUD; preserve core cooldown rules and compare opened before/after images at the same touch
landscape scenario, recording any unrun physical-device acceptance.
