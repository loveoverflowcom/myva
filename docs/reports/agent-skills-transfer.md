# MyVa engineering skills transfer

Status: skills/tooling implemented; local evidence recorded with Android and browser
acceptance gaps, 2026-10-15. Part of #7. The PR owns current merge status.
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

## Android acceptance gap

The Rust `1.98.1` pin is implemented, but **Android compile/link/package is not verified**
for this upgrade. `cargo xtask doctor mobile --platform android` returned `BLOCKED`,
exit 1: no `aarch64-linux-android` target, NDK `27.0.12077973`, SDK platform `37.0`,
build-tools `36.0.0`, ADB or SDK manager; the host has JDK 21 rather than required JDK 17.
No Android SDK/NDK was installed and no APK, CMP integration or device run is claimed.
The actual changed-only preflight retains its native prerequisite gate and must not be
reported as all-PASS when that gate is blocked.

The user explicitly accepted this documented Android verification gap and requested
merging after the other runnable local checks pass. This is acceptance of the PR's known
risk, not a successful Android gate or closure of #10. The limitation is also recorded
directly in `rust-toolchain.toml` and `prototypes/native/Cargo.toml`.

Follow-up: provision the pinned Android prerequisites on an authorized build host, then
run `cargo xtask doctor mobile --platform android` and `cargo xtask build native-probe
--platform android --mode check`, followed by `--mode apk` and `--mode cmp`; retain real
compile/link/package outputs. Physical Android and iOS lifecycle/device acceptance remains
a separate outstanding matrix in [native feasibility](native-feasibility.md).

## Verification record

The runner vocabulary remains exactly `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`. Implementation
status such as proposed/not implemented is a separate capability fact, not a new xtask status.
Logs remain under ignored `target/xtask-evidence/agent-skills/`; bulky traces/reports are not committed.

| Actual command/check | Result and scope |
| --- | --- |
| `python3 scripts/check-skills.py` | PASS: 12 canonical skills, metadata/aliases/index/links/orphans/routing fixture structure only |
| `python3 scripts/check-skills.py --self-test` | PASS: 49 negative/contract controls, including missing/empty suite nonzero behavior |
| `cargo xtask fmt --check` | PASS on Rust 1.98.1 after the separate baseline formatting repair |
| `cargo xtask verify repository` | PASS after approved libudev provisioning: full workspace clippy, 283 Rust unit/integration tests, replay smoke, six ECS/core parity/replay runs, 90-day economy invariants, asset/streaming/report contracts; 90 Python tests (including the 49 validator tests) |
| `cargo xtask verify xtask` | PASS: workspace format, xtask clippy and 25 real unit/lifecycle/impact/plan tests; final repeat also PASS |
| Focused sim/economy/assets/gameplay tests | PASS: 188 tests, a subset of the later 283-workspace result, not an additional unique count |
| Native/WASM determinism | PASS through preflight: seeds 1/7/12, standard and Mach fixtures, 3600 hashes per run |
| WASM Rust clippy + bridge / renderer-independent simulation | PASS through preflight on Rust 1.98.1 |
| Leptos + Bevy WASM bundles | PASS: actual optimized `web` profile build; cold build 16m 07s; output 89,018,280 bytes, gzip estimate 12,322,363 bytes, not measured transferred bytes |
| `cargo xtask preflight --changed-only` | FAIL, exit 1: Playwright selected 24 cases but all failed in browser startup before app assertions; subsequent native stages NOT_RUN. Not an all-PASS preflight |
| Android doctor and `build native-probe --platform android --mode check` | BLOCKED, exit 1: missing pinned target/SDK/NDK/JDK prerequisites; no Android compilation executed |
| Prototype formatting + Android script syntax diagnostic primitives | PASS: both native manifests format-check clean; `bash -n` clean. These direct focused observations are not a substitute xtask preflight PASS |
| Automatic runtime skill discovery | NOT_RUN; explicit-file-loading routing exercise is recorded separately below |
| Native desktop rendering, Android/iOS devices, GPU qualification, formal/mutation/model gates | NOT_RUN in this migration; do not infer acceptance from compilation or structural validation |

The first full repository attempt actually FAILED at `libudev-sys` because development
metadata was absent. This was classified as an environment prerequisite failure, not hidden
or relabeled green. After explicit approval, official Debian `libudev-dev 257.13-1~deb13u1`
was extracted into the workspace and its SHA-256 checked against the
[official download record](https://packages.debian.org/trixie/amd64/libudev-dev/download):
`73c3ed98a435ec420b327ef267d89ece81bf23c2c19391265bb4f261cbf5d558`.
The existing system shared-library ABI was linked without installing/modifying system packages;
a libudev create/unref smoke passed. Pinned registry wasm-bindgen CLI `0.2.129` and npm
lockfile Playwright `1.64.0` dependencies were installed only in workspace locations.
The later repository command passed, preserving the first failure log. Chromium is the
existing Debian `154.0.8037.57`; software rendering is not GPU qualification.

The compiled/tested production Rust/skill/tooling content is local `1f5737998269e728819230b7ea0b4b8cea36df87`,
published with identical tree as `38645a5b9465a335aa0e501dd36c9d302c97617c`. Later edits only
record evidence and acceptance-gap comments in toolchain/native manifests; no gameplay,
renderer, bridge, test oracle or runner behavior changed after those runs.

### Claim ledger

| Claim / owner / failure mode | Oracle, revision and evidence |
| --- | --- |
| Canonical skill structure — validator; malformed metadata, drifted alias/link or orphan | Valid disposable tree plus 49 independent inserted-invalid controls; skill/tooling revision above; L3 static/example-tested PASS, not discovery |
| Conservative impact/runner behavior — xtask; skipped affected gates, false-green empty suite or leaked processes | Explicit selection/planner fixtures and real bounded process/readiness tests (25); same revision; L3/L4 example/integration-tested PASS |
| Rule/replay compatibility — sim + gameplay adapter; changed legacy bytes/pins, typed rejection, schedule/hash drift | Frozen v3 oracle, exact v4 outcomes, core regressions and six per-tick ECS observations; same production revision; L3/L4 tested PASS; parity shares core and is not an independent combat rules proof |
| Native/WASM fixture equivalence — sim + determinism runner; target-specific divergence | Separate native/WASM executions for named seeds/ticks, same revision/compiler; L5 differential observations PASS for those fixtures only |
| Browser interaction — web/Playwright; startup or runtime/bridge/lifecycle failure | Real 24-case suite attempted, but no app assertion executed; same compiled code with evidence-only dirty comments/report; L5 runtime not established, runner FAIL, environment diagnosis below |
| Android upgrade — native platform; incompatible compiler/link/package or lifecycle | Pinned prerequisite probes are BLOCKED; compile/link/package/device oracle NOT_RUN. Manifest formatting does not verify Android |

## Browser acceptance gap

The real preflight selected **24 Playwright cases; 0 passed, 24 failed, 0 skipped** in about
21 seconds. Every case stopped in `browserType.launch`, before navigation or any application
assertion. Chromium reported `socket() failed: Operation not permitted`, plus unavailable
crashpad-home/database setup. No browser interaction, canvas rendering, screenshot inspection,
asset/GPU runtime, 30-cycle browser lifecycle or input acceptance is established by this run.
This is an observed environment startup failure, not evidence of an application regression.

A supported per-command permission attempt for the same approved local E2E action did not
launch its command: the execution wrapper failed on a read-only mount. No retry code or
browser action executed. No protected paths were inspected, browser security settings changed,
flag workaround added, alternate browser/executor substituted, expected result weakened or
suite skipped. Original JSON results and traces are retained in ignored evidence. The runner's
actual result remains FAIL; calling this an environment gap does not turn it into PASS.

This separate browser gap is not covered by the Android-only acceptance statement. Successful
interaction verification requires the same existing `cargo xtask verify web` on an authorized
browser-capable executor, with a fresh nonempty execution report and actual assertions. The
tracked toolchain comments also warn that WASM compile/lint/determinism do not imply E2E PASS.

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

## Publication identity

[Draft PR #31](https://github.com/loveoverflowcom/myva/pull/31) targets `develop`.
Normal Git push could not prompt for HTTPS credentials in this executor. The authorized
connected GitHub API published exact matching trees while keeping formatting, compiler
upgrade and skill/tooling work as three separate commits. API commit metadata differs:

| Local commit | Published commit | Identical tree |
| --- | --- | --- |
| `21b07b9b53a79beeb41f9981004753452c35c3b3` | `fcbd808c9007eaaea80737cf81deb6ed9f939c52` | `d8b7bbf1085ad5b8b1cf87b1e9c2322b98617caa` |
| `cbb3cac0a4845a6aa4dc81d56db2757387633cee` | `525ed73412cc5cefec87b02ad238003259382fe5` | `b14e412f03a79cbdb89952904a2b153759808cec` |
| `1f5737998269e728819230b7ea0b4b8cea36df87` | `38645a5b9465a335aa0e501dd36c9d302c97617c` | `ed3b55b7131a604f45f3fae6df28c8fbe48735f3` |

A later evidence-only follow-up records final command results and the user-accepted Android
gap in tracked toolchain/native manifest comments. Its exact head is recorded in the PR.
No source history was reset, force-pushed or discarded; GitHub CI is not checked/polled.

## Invocation examples

- `$myva-work-item` to deliver a bounded work-plan item with the existing owners.
- `$myva-logic-fix` to repair replay pin rejection while preserving v3 bytes and v4 exact pins.
- `$myva-game-audit` to inspect proc/resource/CC/economy invariants report-only.
- `$myva-cmp-engineering` to plan shell re-entry acceptance without claiming device support.
- `$myva-web-engineering` with `$myva-test-automation` for real shell/bridge Playwright work.
- `$myva-visual-review` for captured-and-opened same-scenario HUD evidence, not source-only UI claims.
- `$myva-diagnostics` for local red-gate triage; `$myva-code-review` remains read-only unless
  the user's current task explicitly authorizes bounded fixes.
