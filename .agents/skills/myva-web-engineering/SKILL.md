---
name: myva-web-engineering
description: >-
  Implement or review MyVa Leptos CSR shell and Bevy WASM web mechanics: separate bundles,
  same-origin iframe runtime, bridge v2 identity/origin checks, focus/IME, lifecycle cleanup,
  load/error/fallback and existing Playwright integration. Keeps DOM, interaction, visual,
  determinism and hardware-performance claims separate through the engineering foundation.
metadata:
  owner: myva-web-engineering
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa web engineering

Platform mechanics only. Compose [engineering](../myva-engineering/SKILL.md) for invariants,
errors, verification vocabulary and report. [UI design](../myva-ui-design/SKILL.md) owns
visual intent; [visual review](../myva-visual-review/SKILL.md) owns inspected screenshots.

## Invocation

Use `$myva-web-engineering` for Leptos shell, web bootstrap/bridge/CSS, Bevy WASM runtime or
web lifecycle/E2E work. Example: “Use `$myva-web-engineering` to fix stale iframe error messages;
reproduce the old-session case and retain the same-origin/source/version/session checks.”
For red CI start [diagnostics](../myva-diagnostics/SKILL.md); for authorized fixes compose
[UI fix](../myva-ui-fix/SKILL.md) or [logic fix](../myva-logic-fix/SKILL.md).

## Reality and boundaries

Read [web feasibility](../../../docs/reports/web-feasibility.md),
[architecture](../../../docs/technical/architecture.md),
[xtask contract](../../../docs/architecture/xtask-build-contract.md) and current code.
`apps/web/shell` is Leptos CSR with DOM/navigation/selection/chat/IME. `apps/web/game` runs
shared Bevy graybox WASM; `apps/web/public` supplies separate shell/game documents and bootstrap.
No Trunk, SSR/hydration, account/server or browser-embedded CMP is implied.

The shell mounts one same-origin game iframe per entry. Its document creates a canvas before
Bevy starts; iframe removal is the teardown boundary. In-document rematch replaces `Session`
through the existing game path, not a second `App::run`. Shell owns IME; Bevy owns game input,
canvas/assets/render. Neither owns online economy or client-reported authority.

## Workflow

1. Name changed entrypoint/bundle and user-visible contract. Load
   [bridge and browser mechanics](references/browser-lifecycle.md), and the owning core rule
   if a pin/loadout/replay decision changes. Keep core decisions out of Leptos/CSS/JS.
2. Reproduce with the current fixture/build before editing. Use exact framework pins in
   Cargo/lockfiles and [xtask](../../../xtask/README.md), not current online examples or
   Cantos runtime commands. New JS/Rust interop is small, typed/versioned and cleanup-owned.
3. Keep version/origin/source/session checks, explicit error/fallback and pause/focus ownership.
   Invalid selection or failed startup must not silently select a default kit or show ready.
4. Select existing suites with [test automation](../myva-test-automation/SKILL.md). `verify web`
   owns build/managed server/Playwright and report validity. Filtered specs are focused evidence;
   full preflight/CI remains unfiltered. No stale report or empty/list-only suite is PASS.
5. For visual changes capture/open baseline and result in the same scenario, with viewport,
   DPR, theme, locale and text scale. Browser interaction alone does not inspect screenshots.
6. Use the foundation report and record browser/version/OS/renderer, build profile/bundle
   report identity, fixture/revision, tests executed, opened images and device/hardware gaps.

The browser's successful render, SwiftShader timing, CPU asset planner tests and keyboard
DOM-to-tick measurement do not establish GPU memory recovery, hardware performance,
input-to-photon latency or physical-touch/mobile qualification.
