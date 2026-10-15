# Bounded UI repair checklist

[UI design](../../myva-ui-design/SKILL.md) owns presentation criteria;
[visual review](../../myva-visual-review/SKILL.md) owns scenario/capture inspection.

Record invariant → observed cause evidence → existing owner → smallest patch → independent
regression → affected consumers. Fix unrelated symptoms together only with a demonstrated common cause.

- Bevy input/layout/HUD sources are in [graybox](../../../../packages/graybox/src/lib.rs);
  `touch.rs`, `hud.rs`, `text.rs`, `scene.rs` and `mapping.rs` have distinct responsibilities.
  Presentation does not write combat mirror state or recompute authority.
- Web shell DOM/CSS is in [shell](../../../../apps/web/shell/src/lib.rs); game document/runtime
  is separate. Use the [web owner](../../myva-web-engineering/SKILL.md) for mount, focus and bridge lifecycle.
- CMP shell is a native feasibility prototype, not completed Bevy embedding. Use the
  [CMP owner](../../myva-cmp-engineering/SKILL.md); a desktop/static preview cannot close re-entry,
  surface, cancel, IME, background/resume or physical-device acceptance.
- A shared semantic style/string change requires checking actual consumers, not assuming a
  shared generated token pipeline exists. Reuse established values; do not import Cantos's palette.
- Preserve authoritative denied reasons and fixture/unmapped/placeholder labels. Do not make
  unknown kit/art silently appear approved or turn a fallback into a success-shaped loaded state.

Before/after must retain fixture, state/tick/phase where applicable, viewport, DPR/render scale,
theme, locale, text scale and input. Record source/build separately for each. If a scenario changed,
state why and limit the comparison; never edit expectations to bless the patch. Open every cited
image and record the actual observations. A source-only fix with no render is not visually verified.

Escalate whole-screen redesign, domain-model/authority changes, new input mechanics or architecture
work through [work item](../../myva-work-item/SKILL.md) rather than expanding the repair silently.
