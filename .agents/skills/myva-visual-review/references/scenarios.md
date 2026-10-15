# Scenario binding and real MyVa inspection paths

This is a human-executable scenario contract, not a newly implemented scenario runner. Reuse real
fixtures/specs and the [web owner](../../myva-web-engineering/SKILL.md) or
[CMP owner](../../myva-cmp-engineering/SKILL.md); do not import Cantos's awaiting-app scenario files.

## Record before capture

- Surface/route and purpose; source SHA plus dirty paths; actual build/bundle identity.
- Renderer/browser/version/OS; GPU/backend when known; device versus emulator/simulator/desktop host.
- Scenario ID/version, fixture/seed, mode/round/tick/phase and state sufficient to reproduce.
- Viewport with CSS px/dp/pt units, physical resolution, DPR/render scale and orientation/safe area.
- Theme, locale and text scale with actual mechanism; use current/unsupported/unknown explicitly.
- Input modality; permitted actions, expected observations with owning contract, budget and cleanup.
- Capture path/SHA-256, tool/reviewer/time opened, concrete observations and companion interaction trace.

Unknown metadata limits the claim; never guess from file names or source alone. Do not mutate a
scenario/expected fixture between before/after; version substantive changes and disclose comparability.
Images alone cannot identify exact timing, input ownership, durable state, GPU allocation or authority.

## Reuse actual scenarios

| Question | Existing source / run |
| --- | --- |
| Host enter/exit, focus/IME, resize/DPR, touch cancel, renderer/load/context failures | [lifecycle.spec.mjs](../../../../tests/web/lifecycle.spec.mjs), `cargo xtask verify web -- tests/web/lifecycle.spec.mjs` |
| Hit/whiff, controls and replay | [combat.spec.mjs](../../../../tests/web/combat.spec.mjs) and [mach.spec.mjs](../../../../tests/web/mach.spec.mjs) |
| Selected exact kit, unresolved/placeholder labels, startup failure | [character-catalog.spec.mjs](../../../../tests/web/character-catalog.spec.mjs) |
| Real art and missing-art geometric fallback | [assets.spec.mjs](../../../../tests/web/assets.spec.mjs), [asset report](../../../../docs/reports/asset-pipeline.md) |
| High/Low/stress and lifecycle resource evidence | [runtime-performance.spec.mjs](../../../../tests/web/runtime-performance.spec.mjs), [D05-B report](../../../../docs/reports/asset-streaming-performance.md); CPU counters/software-renderer runs are not target GPU qualification |
| Desktop Bevy runtime | `cargo xtask run graybox -- --duel` or explicit `--no-art`; record actual host/display/GPU and scenario, not mobile evidence |
| CMP/native surface/lifecycle | [native feasibility](../../../../docs/reports/native-feasibility.md); package/desktop preview is not device integration. Re-entry 30x, background/resume 20x, surface ownership, pointer cancel, IME and cleanup remain target gates |

`cargo xtask run web --port <N>` owns build/server/readiness and stays foreground;
`cargo xtask app web --port <N>` opens only an existing matching bundle/server. Browser verification
owns its own server; never start a second server for that command or kill a listener to take its port.
Confirm build identity via the existing bundle report. Preserve actual test screenshot paths, then
open images individually with an image-capable tool. Automated assertions are not human pixel inspection.

## Gameplay cells

Choose cells for the question: boss telegraph startup/active, hit/block/whiff/denied feedback,
HP/stamina/energy/Mạch/cooldown, arena edges/camera, touch short/hold/cancel and art-off/required-load
failure/optional-VFX fallback. Compare the [rubric](../../myva-ui-design/references/presentation-rubric.md).
Keep state and seed controlled; a changed combat phase makes a pixel diff ambiguous. Use sequences
and actual events to judge motion/timing. Small mobile viewport emulation never closes native gate.
