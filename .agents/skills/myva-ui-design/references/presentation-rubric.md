# MyVa presentation acceptance

Sources: [combat](../../../../docs/design/combat.md), [QA](../../../../docs/production/qa.md),
[asset pipeline](../../../../docs/technical/asset-pipeline.md) and
[assets/performance](../../../../docs/technical/assets-performance.md). Their proposed targets
remain proposed until tested; this rubric does not create a shipped feature or numerical budget.

## Shell and shared presentation

- The player can identify location, primary content, current state and primary action. Secondary
  chrome should not compete with content. Preserve MyVa identity instead of transferring Cantos's
  palette, typography, player controls or text-editor layout.
- Trace semantic colors/spacing/type to actual source owners. Keep visible and accessible names
  aligned. State must not rely only on color; distinguish selected, disabled-with-reason, busy,
  retryable error and unavailable capability. A loader is not permission or durable success.
- Inspect Vietnamese diacritics, clipping, wrapping, long kit/reason labels, safe areas and enlarged
  text. Record the actual locale/theme mechanism; do not add unsupported English/dark-mode claims.
- Keyboard focus is visible, logically ordered and restored across navigation. Shell chat/IME
  consumes text keys instead of leaking them into game controls; live interaction is required.
- Motion supports orientation and feedback, not decoration that hides state. Check reduced
  shake/flashes where implemented; proposal-only controls must not be reported as shipped.
- Assets retain anchor, world scale, facing and collision conventions. Missing art preserves
  legible gameplay geometry and explicit fallback/error state; metadata approval is not art availability.

## Combat-specific rubric

| Area | Inspect and pair with an independent observation |
| --- | --- |
| Telegraph | Startup/active/recovery, direction and danger footprint remain legible against map/VFX, on low quality and fallback; compare displayed area/timing with core data/event trace, not screenshot alone |
| Hit feedback | Hit/block/whiff/denied/guard-break/Mạch escape/CC-limited feedback matches authoritative events; no false confirmed hit, hidden denial or unreadable burst of overlapping popups |
| HUD | HP/stamina/energy/Mạch/cooldown and phase/denied reason readable at actual size; stable hierarchy, no stale value or invented refund; no promise of missing counter-hit/punish labels |
| Camera | Framing retains fighters, arena bounds and danger; shake/zoom/Low render target do not move hitboxes, hide a reaction or blur critical HUD; motion needs a runtime sequence |
| Touch | Joystick and eight-button layout follow current implementation, viable target size/safe area and finger occlusion; short/hold/cancel and multi-pointer ownership checked live, not inferred from image |
| Fallback | Required-asset failure and optional-VFX failure retain geometric telegraph, projectiles/feedback/HUD; no silent alternate kit, no disappearance of the dangerous signal |

Use [visual scenarios](../../myva-visual-review/references/scenarios.md) for comparable captures.
Browser DOM/interaction cannot establish every canvas pixel. CPU handle release cannot establish
GPU allocation/reclamation, Update/RAF rate is not presented FPS, and software rendering is not
hardware qualification. No screenshot establishes balance, online fairness or real-device operability.
