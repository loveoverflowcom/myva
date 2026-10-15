# MyVa repair seams

Principles and evidence remain with [engineering](../../myva-engineering/SKILL.md).
Choose the production seam, not a renderer workaround:

| Symptom | Owning source / acceptance |
| --- | --- |
| Wrong tick, spend, hit, CC or rejection | [sim](../../../../packages/sim/src/lib.rs), [combat](../../../../docs/design/combat.md); exact event/state regression, phase boundaries and replay |
| Sequence, epoch, backlog or accepted-tick error | [session](../../../../packages/sim/src/session.rs), [protocol](../../../../packages/sim/src/protocol.rs); duplicate/reorder/old-epoch/out-of-window cases, no partial mutation |
| Wrong registry/loadout/pin | [registry](../../../../packages/sim/src/registry.rs), [kit contract](../../../../docs/technical/kit-registry-loadout.md); constructor and parser plus direct API/restore bypass checks |
| Wrong equipment formula/proc | [equipment](../../../../packages/sim/src/equipment.rs), [contract](../../../../docs/technical/equipment-registry.md); cap/rounding/overflow/action watermark/cooldown regression, preserve runtime rejection |
| ECS differs from core or rewards duplicate | [gameplay](../../../../packages/gameplay/src/lib.rs), [authority](../../../../packages/gameplay/src/authority.rs); per-tick core/ECS and mirror assertions, server/client separation |
| Conservation/idempotency/budget | [ledger](../../../../packages/economy/src/ledger.rs), [budget](../../../../packages/economy/src/budget.rs), [economy](../../../../docs/design/economy.md); exact transfers, same-ID replay/conflict, overflow and caps |
| Wrong asset lifecycle/presentation | [graybox](../../../../packages/graybox/src/lib.rs), [asset report](../../../../docs/reports/asset-streaming-performance.md); CPU handle ownership and runtime/GPU evidence separately |

Trace every changed value to its authoritative source. A prototype contact boolean is not a
verified combat contact; `RewardOutbox` is not a persistent server ledger. Preserve immutable
facts and explicit exact-revision rejection rather than guessing a replacement.

Regression design: derive expected output from the rule, hand-authored boundary table, independent
implementation or frozen compatibility fixture, not a second call to the same helper. Include
minimum/maximum/overflow, repeated/reordered input, phase/tick edges and rejected-transition state
as applicable. Record actual seeds/bounds and which failure mode the test would catch.

Use `cargo xtask verify sim`, `verify gameplay`, `verify economy`, `verify assets`,
`verify determinism` and `verify web` according to affected consumers. Focused package primitives
are diagnostic probes only when xtask offers no equivalent filter, not replacement orchestration.
Do not add Kani/mutation/Quint gates or pins to repair a bounded bug without the engineering
owner's production-kernel/model, runner and negative-control evidence.
