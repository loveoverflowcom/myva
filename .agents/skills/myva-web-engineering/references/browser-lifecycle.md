# Browser boundary and lifecycle mechanics

Read current [shell bridge](../../../../apps/web/shell/web/shell-bridge.js),
[game runtime](../../../../apps/web/public/game/runtime.js),
[Leptos shell](../../../../apps/web/shell/src/lib.rs) and
[WASM browser adapter](../../../../apps/web/game/src/browser.rs).
[Web architecture](../../../../docs/technical/architecture.md) owns the product boundary;
[engineering](../../myva-engineering/SKILL.md) owns method/evidence.

## Bundle and process ownership

The existing pipeline builds separate Leptos CSR and Bevy WASM bundles. Shell DOM/navigation
and game canvas/runtime live in separate same-origin documents, with sparse bridge messages
rather than full-world JSON per frame. Do not import a Cantos player, provider, SSR or Trunk
pipeline. If SSR is proposed later, client-only mount/hydration requires a new explicit design.

Use `cargo xtask build web`, `run web`, `app web` and `verify web` as documented in the
[build contract](../../../../docs/architecture/xtask-build-contract.md). `run web` owns server
and readiness; `app web` only opens a ready existing build. Readiness requires the current
bundle report/token, not successful spawn or a listening port. Do not start a second server
from Playwright/npm, kill another listener or pretend browser prerequisites were provisioned.

## Bridge v2 and identity

- Keep exact version, `event.origin`, `event.source` and current session checks in **both**
  directions. Use the exact same-origin target; no wildcard origin, old WindowProxy or stale
  session accepted after leave/re-enter.
- Validate payload fields before use: finite/safe integers, enums, booleans, kit ID syntax,
  hash shape and allowed command set. TypeScript/serde/Rust types do not validate arbitrary JS.
- Selection carries exact kit/version/hash. Invalid/incomplete pins fail startup explicitly;
  no fallback to another kit. Display metadata does not become a gameplay hash or authority.
- Preserve replay download v3/v4 contract checks. Safe filename/header checks in the bridge
  do not replace core parser/playback validation. Object URLs are revoked after download.
- Sparse telemetry reports observations. Client HP/hash/outcome text is not proof of server
  authority, committed reward or future online admission.
- Foreign JS failures map at the boundary into recoverable shell errors; keep original
  diagnostics without exposing credentials. No auth tokens in URL/query/log/manifest.

Existing [protocol E2E](../../../../tests/web/protocol.spec.mjs) includes synthetic hostile
messages and stale runtime cases. Keep synthetic message/visibility wiring claims separate
from actual browser security exposure, OS backgrounding and device lifecycle claims.

## Lifetimes and interaction

Shell mount creates a fresh session and one iframe; exit sends the command, removes the
browsing context and cleans shell message/visibility listeners, timeout and selected state.
Game runtime owns AbortController listeners/timer, pauses before disposal and does not restart
Bevy in the same document. Late load/export/retry callbacks must not revive an exited session.

Distinguish rematch/mode replacement within the running Bevy app from route leave/re-enter.
Test both. A rematch must use existing locked/revalidated loadout paths, keep the old bout
on rejection and not acquire a second render/input loop.

- Canvas focus owns game keyboard input; pointer interaction returns focus to the canvas.
  Focus loss/chat/IME clears held keys and pending input under the game's contract.
- Shell input receives Vietnamese composition without movement/attacks. Test keyboard
  Tab/Shift+Tab navigation and focus return, not only pointer clicks.
- Manual pause, hidden-document pause and pre-init/failure pause are distinct facts. Visibility
  resume cannot override explicit manual pause. Synthetic hidden overrides test wiring only.
- Track touch/pointer IDs/cancel and gamepad state through existing Bevy input, not a divergent
  shell combat reducer. Touch emulation remains emulation.
- Keep container dimensions independent from canvas resize to avoid feedback loops; test
  resize/DPR, narrow landscape, safe areas and text scaling. Rendering floats never change
  core integer rules.
- Loading, bounded timeout, WebGL2 absence/context loss and asset failure must leave an
  actionable error/fallback. Do not infer context recovery merely because initial load works.
  Missing art retains placeholder behavior; rule state and replay remain separate.
- Clean every added listener, observer, timer, promise continuation/resource handle at the
  right owner. Browser globals/imperative DOM access stay in small named helpers; use
  declarative bindings/NodeRef where Leptos already owns the element. Never inject raw HTML
  for content or use a DOM query to create a second owner of framework state.

## Existing acceptance surfaces

[Lifecycle](../../../../tests/web/lifecycle.spec.mjs) drives 30 document cycles, input/focus,
rematch and CPU/browser observations. [Protocol](../../../../tests/web/protocol.spec.mjs),
[combat](../../../../tests/web/combat.spec.mjs), [Mạch](../../../../tests/web/mach.spec.mjs),
[assets](../../../../tests/web/assets.spec.mjs),
[character selection](../../../../tests/web/character-catalog.spec.mjs) and
[runtime performance](../../../../tests/web/runtime-performance.spec.mjs) cover different
claims. Read assertions and report contracts before selecting a spec; filename alone is not
an evidence claim. [Asset performance report](../../../../docs/reports/asset-streaming-performance.md)
separates CPU lifecycle and render-only qualification from missing GPU/hardware evidence.

Examples, not recorded executions:

```sh
cargo xtask run web --port 8080
cargo xtask verify web --port 8081 -- tests/web/protocol.spec.mjs
cargo xtask verify web --port 8081
```

The focused spec helps reproduce a fault; it does not replace full relevant acceptance. A
fresh passing test report must contain executed passing tests, no failed/flaky/skipped/global
errors, and match the current source/build. Captures are only `screenshot-captured` until
opened. Report build/scenario/fixture/viewport/DPR/theme/locale/text scale for each inspected
image. Browser version/software renderer and real hardware results must not be merged.
