# MyVa engineering skills transfer

Status: implementation and local verification in progress, 2026-10-15. Part of #7.
No gameplay feature or native acceptance is introduced by this change.

## Provenance and surveyed reality

- Target: [loveoverflowcom/myva](https://github.com/loveoverflowcom/myva), freshly fetched
  `develop` at `7b43bc848259a80ca591b8ccd08acae2dbf9ae48`.
- Source: [loveoverflowcom/cantos](https://github.com/loveoverflowcom/cantos), exact commit
  [`f05b09bf24243bb99bc869f5009628294ebb15d8`](https://github.com/loveoverflowcom/cantos/tree/f05b09bf24243bb99bc869f5009628294ebb15d8).
  Verified as a fetched Git commit and read through pinned Git blobs, not the moving branch.
- Source inventory: `AGENTS.md`, `.agents/skills/README.md`, `routing-cases.md`;
  engineering, work-item, code-review, diagnostics, logic-fix, ui-fix, ui-design,
  ui-inspector, cmp-mobile, leptos-web and behavior; relevant types, errors, functional
  core, immutability, verification, mutation/formal and local-execution references;
  `scripts/check_repository.py` and its negative tests.
- Target survey: [README](../../README.md), [CONTRIBUTING](../../CONTRIBUTING.md),
  [architecture](../technical/architecture.md), [repository layout](../technical/repository-layout.md),
  [combat](../design/combat.md), [economy](../design/economy.md),
  [work queue](../work-plan/README.md), [kit/replay](../technical/kit-registry-loadout.md),
  [equipment](../technical/equipment-registry.md), [native feasibility](native-feasibility.md),
  [asset performance](asset-streaming-performance.md), [xtask contract](../architecture/xtask-build-contract.md),
  and the actual sim/gameplay/renderer/web/xtask sources and tests.

The issue's historical docs-only/no-web assumptions are obsolete. MyVa has a pure Rust
sim/economy core, Bevy ECS adapter, graybox renderer, Leptos shell, seven Playwright specs,
asset lifecycle/harness code and xtask orchestration. Online persistence/authority,
production equipment effects and confirmed CMP Bevy device integration are not implemented
or accepted simply because those local components exist.

## Retain / adapt / exclude / proposed mapping

| Source concern | Decision and MyVa owner | Reason |
| --- | --- | --- |
| Engineering order, pure core, types/errors, decoupling, independent layered evidence | Retain method; adapt in `myva-engineering` | Sim is the single combat/session/protocol/replay rule owner, not a service/API layer |
| Immutable editorial/release facts | Adapt to immutable catalog/revision/exact loadout/replay facts | Internal owned `World`/ECS mutation remains valid; no whole-world clone-per-tick requirement |
| Verification L1–L5 and local completion honesty | Retain vocabulary in engineering; adapt examples/runners | Core, ECS parity, native/WASM, browser, opened pixels and real devices establish different claims |
| Work-item / diagnostics / logic-fix / ui-fix / code-review | Retain workflow and report responsibility | These compose rule owners and user authorization rather than adding rules or permissions |
| ui-design / ui-inspector | Adapt to `myva-ui-design` / `myva-visual-review` | MyVa game presentation needs telegraph, hit feedback, HUD, camera, touch and fallback rubric; no Cantos brand tokens |
| cmp-mobile | Adapt to `myva-cmp-engineering` | CMP owns shell; standalone/one-shot prototype is not native embedded Bevy acceptance |
| leptos-web | Adapt to `myva-web-engineering` | Real Leptos DOM shell is separate from Bevy iframe/canvas/input/runtime, bridge v2 and teardown |
| Source test-selection patterns | Adapt to `myva-test-automation` and existing xtask | Reuse doctor/build/verify/run/ci, readiness, fresh execution reports and owned-process cleanup |
| Domain audit and boundary review | Add `myva-game-audit` | Check duplicate procs/rewards, overflow, cooldown, CC budget, atomic rejection and future server authority |
| Repository validator and negative fixtures | Adapt to MyVa stdlib validator + existing xtask | Check metadata/index/links/symlinks/orphans/routing without implying runtime discovery |
| Cantos behavior/Quint methodology | Proposed selection guidance in engineering references | No registered MyVa production model/observer/negative controls/run exists; model PASS cannot prove implementation conformance |
| Kani / mutation execution | Proposed bounded-kernel selection guidance | No copied Cantos toolchain pins or invented executable gate; require real production seam, runner, negative controls and attributable run |
| Cantos audio, publication, PostgreSQL/DBSP, identity, provider/storage and service commands | Exclude | These are unrelated product boundaries/capabilities; online MyVa authority remains future server work |
| Source global brand/palette, app paths, ADR numbers, mobile MCP/Gradle tasks and source runner statuses | Exclude exact values; inspect target | Source names or tool availability cannot establish target capability |

Canonical content lives in [the skill index](../../.agents/skills/README.md). Each of 12
skills has compact frontmatter with provenance, display metadata, an explicit invocation and
references as needed. `.claude/skills/<name>` is a relative symlink to exactly that directory.
[AGENTS](../../AGENTS.md) and [CLAUDE](../../CLAUDE.md) route only; product facts remain in docs/source.

## Rust upgrade and baseline repair

The user's later instruction upgrades MyVa to Rust `1.98.1`, matching the source compiler
choice, without importing other Cantos toolchains/capabilities. The official rustup installer
and Rust distribution were used in an isolated workspace toolchain home; no global shell
configuration was changed. Verified `rustc 1.98.1 (48a229cea 2026-09-01)`, Cargo `1.98.1`,
rustfmt `1.9.0`, clippy `0.1.98`, native and `wasm32-unknown-unknown` targets, and compiled/executed
a native assertion smoke program. No old Rust toolchain existed in the inspected environment,
so no old toolchain was removed.

Before restoration, the new formatter reproduced the four pre-existing failures named in
the brief: `packages/sim/src/bin/myva-replay.rs`, `equipment.rs`, `registry.rs`, and
`packages/sim/tests/equipment_registry.rs`. Formatting-only commit `21b07b9` repairs them
separately. Rust upgrade commit `cbb3cac` changes workspace/native manifests, toolchain,
CI provisioning pin, target probe and active instructions. Historical reports retain their
actual old compiler; they are not relabeled as new-toolchain results.

## Verification record

The runner vocabulary remains exactly `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`. Implementation
status such as proposed/not implemented is a separate capability fact, not a new xtask status.
Raw outputs are retained locally under ignored `target/xtask-evidence/agent-skills/`.
Current scoped results: structural validator PASS for all 12 skills; 49 negative/contract
tests PASS; focused `cargo xtask verify xtask` PASS (format, clippy, 25 tests);
`cargo xtask fmt --check` PASS; 188 sim/economy/assets/gameplay tests PASS on Rust 1.98.1.
The first full repository attempt actually FAILED at `libudev-sys` because development
metadata was absent; it was classified as an environment prerequisite failure, not hidden.
After explicit installation approval, official Debian libudev-dev was checksum-verified and
extracted in the workspace, npm lockfile dependencies installed with a workspace cache, and
the repository check restarted. Full repository/preflight results are still pending.
This draft section must not be read as a complete passing result.

## Explicit routing exercise

An independent evaluator was given only the 11 id/prompt pairs, routing entrypoints/index
and relevant canonical skills, with expected primary/support/reason fields withheld.
It selected the intended primary for 9 of 11 initial prompts, preserving permission and
evidence boundaries in all 11. The two differences were ECS lifecycle and asset/performance
audits: broad “read-only review” wording led to code-review rather than game-audit.

The router/index and game-audit scope were clarified to distinguish a PR/range/diff review
from a domain/capability audit, and the audit matrix now names ECS lifecycle and asset
performance owners explicitly. A supplemental evaluation of the same two prompts selected
game-audit and reported the ambiguity resolved. Other initial results remain recorded, not
retroactively relabeled. This explicit-file-loading exercise evaluates routing choices only;
no prompt action was executed and it does not establish automatic runtime discovery.

A separate read-only implementation review found no actionable findings in the 12 skills,
metadata, aliases, validator/negative controls, impact/planner and pin/provenance changes.
All ten files included by `rules_config_hash` were byte-identical to base after the formatting
repair. That review did not perform remote CI checks, GPU or mobile acceptance.

## Runtime discovery and acceptance limits

Structural validator success checks files and contracts only. Explicit file loading and the
[routing exercise](../../.agents/skills/routing-cases.md) are distinct from automatic runtime
discovery. A fresh Codex/Claude runtime must actually list/load repository skills before
claiming automatic discovery; it is not inferred from metadata/symlinks. No product UI pixels,
GPU qualification, native mobile lifecycle/device acceptance, Kani proof, mutation result,
Quint model or production server claim follows from this tooling change.

## Invocation examples

- `$myva-work-item` to deliver a bounded work-plan item with the existing owners.
- `$myva-logic-fix` to repair replay pin rejection while preserving v3 bytes and v4 exact pins.
- `$myva-game-audit` to inspect proc/resource/CC/economy invariants report-only.
- `$myva-cmp-engineering` to plan shell re-entry acceptance without claiming device support.
- `$myva-web-engineering` with `$myva-test-automation` for real shell/bridge Playwright work.
- `$myva-visual-review` for captured-and-opened same-scenario HUD evidence, not source-only UI claims.
- `$myva-diagnostics` for local red-gate triage; `$myva-code-review` remains read-only unless
  the user's current task explicitly authorizes bounded fixes.
