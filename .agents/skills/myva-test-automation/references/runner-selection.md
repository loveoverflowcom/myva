# Existing runner selection, not a second verification policy

The [xtask build contract](../../../../docs/architecture/xtask-build-contract.md),
[CLI](../../../../xtask/src/cli.rs), [planner](../../../../xtask/src/plan.rs) and
[impact selection](../../../../xtask/src/impact.rs) are the source of commands/behavior.
[Engineering](../../myva-engineering/SKILL.md) decides required evidence/levels and independent oracles.

| Claim / target | Existing surface | What it does not establish |
| --- | --- | --- |
| Prerequisite readiness | `cargo xtask doctor` | No test/build/runtime/device execution |
| Core rules/registry/replay | `cargo xtask verify sim`; `run replay -- verify <path>` | ECS parity, browser/render/device and cross-build semantic migration |
| Core → ECS/mirror/authority parity | `cargo xtask verify gameplay` | Networked server authority, persistence or GPU |
| Native/WASM per-tick equality | `cargo xtask verify determinism --ticks 3600 --seeds 1,7,12` | Other seeds, future rules, browser-rendered performance |
| Economy invariant/simulator | `cargo xtask verify economy`; `run economy -- --seed 7 --days 90` | Concurrent server commit, real player economy/balance |
| Asset/provenance/budget/report contracts | `cargo xtask verify assets` | Real rendered readability, physical GPU allocation, hardware timing |
| Browser bridge/lifecycle/combat/assets | `cargo xtask verify web`; focused `-- tests/web/<name>.spec.mjs` | Full suite when filtered, opened pixel inspection, native/mobile hardware |
| Desktop runtime/build | `build graybox`; `run graybox -- --duel`; `run headless -- --seed 7 --ticks 3600` | Headless is not render/audio/GPU; desktop is not CMP embedding |
| Web build/live local server | `build web`; `run web --port 8080`; `app web --port 8080` | Build/readiness/opening browser alone is not E2E or visual PASS |
| Repository/tooling | `verify repository`; `verify xtask`; `fmt --check` | Structural skill validator is not runtime discovery/routing evaluation |
| Changed/full integration | `preflight --changed-only --base origin/develop`; `preflight`; `ci` | Native prototype/device gates are not implicit root CI coverage |
| Native prerequisite/package | `doctor mobile --platform android`; `build native-probe --platform android --mode check|build|apk|cmp`; `devices` | Listing/package/preview is not install/render/lifecycle/device PASS; no mobile install/run command exists |

All abbreviated commands in the table use the `cargo xtask` prefix. Re-check `--help` when needed;
never invent `verify visual`, `verify device`, a `rust` target, Trunk or a Cantos service runner.
iOS build/packaging is not implemented and explicit selection returns BLOCKED/nonzero. Android
builds do not solve re-entry/surface ownership. [Native feasibility](../../../../docs/reports/native-feasibility.md)
keeps Android/iPhone physical-device gates independent from host/emulator/simulator evidence.

## Nonempty/fresh execution

Web verifier owns a loopback server, checks bundle identity/readiness and fresh Playwright JSON:
passing tests must exist, with no skipped/flaky/failing/global-error results. `--list`, zero tests,
stale JSON and a later retry do not make the original run pass. Record every failure. Focused
filter success does not cover unselected specs; full preflight/CI stays unfiltered.

Changed-only requires a valid merge-base and Cargo metadata; missing base/discovery is nonzero,
not empty diff. Empty diff selects the full contract. `--dry-run` is NOT_RUN, not validation;
never claim its printed plan actually executed. Use configured concurrency, no added heavyweight
parallel builds. Any primitive-only diagnostic probe retains the xtask gate's actual outcome.

Actual statuses from MyVa are PASS/FAIL/BLOCKED/NOT_RUN. Use explanation, not additional runner
statuses, for applicability or absent implementation. Keep actual assertion failure distinct from
an environment blocker and an unrun future gate. Repair causes only within authorization.
