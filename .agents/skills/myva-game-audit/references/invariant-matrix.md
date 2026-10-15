# Gameplay and economy audit matrix

Read [engineering](../../myva-engineering/SKILL.md) for methods/evidence and actual source before
using a documented threshold. [Combat](../../../../docs/design/combat.md) mixes current GT with
future proposals; [equipment](../../../../docs/technical/equipment-registry.md) is a pure prototype.

| Invariant / failure mode | Owner and discriminating checks |
| --- | --- |
| Repeated proc/hit | [World](../../../../packages/sim/src/world.rs), [equipment](../../../../packages/sim/src/equipment.rs): one source action/target and allowed hit index; repeated contact, later hit index, same-tick multiple sources, item-origin contact, replayed old action and action-ID wrap. Equipment watermark/cooldown prototype is not authoritative combat contact |
| Resource bounds/overflow | [fighter](../../../../packages/sim/src/fighter.rs), [meter](../../../../packages/sim/src/meter.rs), equipment: zero/cap/cap-1/max integer, insufficient funds, widened intermediates, ceil reductions and positive floors; spend/refund once and no under/overflow. Retain sim fixed-width integer/tick semantics |
| Cooldown | Skill/ultimate sharing, exact eligible tick versus one before, repeated request, new source during cooldown, tick monotonicity/u32 edge with u64 cooldown arithmetic, death/rematch/reset semantics. Old denied contact must not proc when replayed after expiry |
| CC/escape budget | [control](../../../../packages/sim/src/control.rs): target-owned budget shared across melee/counter/projectile/guard break, multi-attacker same tick, one-tick gap versus full reaction window, DR history, boss poise, damage granted after budget, escape not resetting budget. Root/launch/air follow-up are not currently enabled |
| Rejected transition atomicity | [session](../../../../packages/sim/src/session.rs), [registry](../../../../packages/sim/src/registry.rs), equipment/ledger: exact typed error, state/RNG/sequence/resources/cooldowns/buffer/events/outbox unchanged except explicitly contracted rejection telemetry; validate before spawn/effects |
| Protocol authority | [protocol](../../../../packages/sim/src/protocol.rs): actor/epoch/sequence/tick window/backlog, duplicate/reorder/rebind; accepted tick chosen by owner, no client damage/hit/reward/balance authority. Offline command validation is not deployed transport/authentication |
| Reward duplication | [authority](../../../../packages/gameplay/src/authority.rs): Downed once, one RewardKey per instance/defeated including drain/retry, no client outbox; server durable unique ledger/commit still future work |
| Immutable compatibility | [registry/loadout](../../../../docs/technical/kit-registry-loadout.md): exact version/hash, unknown/missing/duplicate/extra/mismatched pins, metadata/gameplay hash isolation, frozen v3 bytes and v4 rules/kit pins, parser plus direct Replay API revalidation; hash is not authentication |
| ECS/client lifecycle | [mirror](../../../../packages/gameplay/src/mirror.rs), [runtime](../../../../packages/graybox/src/runtime.rs): one simulation step owner, ordered reset/despawn and stale-ID cleanup; repeat rematch/enter/exit, late input and cancellation. Headless mirror cleanup is not renderer surface/texture recovery or native re-entry |
| Asset/performance evidence | [streaming](../../../../packages/graybox/src/streaming.rs), [performance report](../../../../docs/reports/asset-streaming-performance.md): bounded load/retry/unload, missing/hash-invalid fallback, owned handles and identity; CPU planner/handle tests do not measure GPU allocation/reclamation, rendered frame pacing or hardware performance |
| Economy conservation | [ledger](../../../../packages/economy/src/ledger.rs): total R+N+I+X+Q+Z conservation, only allowed transitions, Removed cannot return, same EventId payload replay versus conflicting retry, overflow and failed operation atomicity |
| Economy issuance | [budget](../../../../packages/economy/src/budget.rs), [simulator](../../../../packages/economy/src/sim.rs): hourly/daily/lifetime/pool/node-space caps, exhausted budget immediate zero, demand shocks, alt/channel duplication, starvation/distribution; simulation is not concurrent server-commit evidence |

## Capability boundaries that must remain visible

- [Equipment runtime](../../../../docs/work-plan/backlog/equipment-runtime-effects.md) is not wired:
  valid nonempty loadouts return `RuntimeNotImplemented` before spawn. Pure cost/guard/proc functions
  do not prove live combat integration. Preserve that rejection; never silently ignore equipment.
- Existing replay v3/v4 do not encode equipment runtime state. Enabling it requires an explicit
  new version, compatibility fixtures and core/ECS/native-WASM evidence, not mutation of old files.
- Headless tests, ECS parity and native/WASM determinism answer different claims. Browser replay
  is wiring evidence; touch emulation is not physical-device/playtest evidence.
- Kani, mutation and Quint are proportional options owned by engineering. Until an actual
  production kernel/model, bounded assumptions, runnable harness, negative controls and real result
  exist, mark them proposed in prose and add no fictional executable gate.

Every audit row names exact explored bounds and uncovered paths. A missing test is not a proven
exploit; a source-level counterexample can be a confirmed defect even before an unavailable runner.
