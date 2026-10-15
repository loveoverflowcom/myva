# Native shell, surface and lifecycle mechanics

This reference applies the foundation's [method](../../myva-engineering/SKILL.md) to existing
MyVa platform seams. Product gates remain in
[native feasibility](../../../../docs/reports/native-feasibility.md),
[architecture](../../../../docs/technical/architecture.md) and
[prototype README](../../../../prototypes/native/README.md); use their current pins rather
than copying Cantos's toolchains or renderer assumptions.

## Read the real owners

- [Shared shell/overlay](../../../../prototypes/native/android/shared/src/commonMain/kotlin/com/myva/probe/ProbeUi.kt)
  renders values and emits callbacks; it owns no game loop or combat state.
- [ShellActivity](../../../../prototypes/native/android/cmp-shell/src/main/kotlin/com/myva/cmpprobe/ShellActivity.kt)
  launches the game and validates version/result. `ProbeProtocol.started` is process-scoped
  because dropping a Winit event loop does not make recreation valid.
- [GameScreenActivity](../../../../prototypes/native/android/cmp-shell/src/main/kotlin/com/myva/cmpprobe/GameScreenActivity.kt)
  is a `GameActivity` with a ComposeView overlay and lifecycle-disposal strategy. It owns
  close/back result emission. Verify the actual path under activity recreation/process death.
- [Native Rust probe](../../../../prototypes/native/src/lib.rs) owns standalone Bevy input
  and render; pointer end/cancel and suspend must clear held movement intent.

## Do not cross an unproved integration boundary

Option A (embedded native view) remains research. `AndroidView`, `UIKitView`, a JNI pointer,
`SurfaceView` or `CAMetalLayer` alone is not an adapter. A real adapter must establish thread,
window/surface/context, event delivery, handle generations, allocation/free and teardown owners.
Do not publish a fictitious `create/attach/pause/destroy` ABI while stock plugins still own
application-global resources. No world JSON per frame or cross-runtime Rust struct layout.

Option B currently launches a separate Android native Activity once. Re-enabling its launch
button does not solve `RecreationAttempt`; force-stopping the process between attempts is not
30 successful re-entries. The Linux probe reproduces an ownership constraint, not mobile success.
On iOS stock Winit wants to call `UIApplicationMain` before UIKit exists; calling `App::run`
from an existing CMP controller does not resolve this. A custom runner or inverted application
ownership needs its own implemented/verified design before promising an iOS path.

Keep shell navigation and renderer lifetimes distinct. An Activity closing or a composable
being disposed does not prove native loop/texture/resource cleanup. Late callbacks must be
fenced to the active lifetime. Cancellation is lifecycle control, not a swallowed error;
Kotlin adapters preserve `CancellationException` and map foreign faults narrowly.

## Input, focus and ownership

- Track each pointer ID and cancel/end explicitly. A lost surface, focus transfer, pause,
  navigation away or IME activation clears held movement/actions and buffered press intent
  under the existing input contract. Do not clear only visual button state.
- Define overlay hit routing: tapping Close/HUD must not also attack or move the fighter.
  Check multitouch, pointer cancellation, edge drag and orientation/safe-area changes.
- Shell owns Vietnamese IME/chat when implemented. Keyboard open/close and composition must
  not leak movement/skill keys into Bevy. Current prototype has no IME/audio harness; record
  those gates as unimplemented/unrun rather than inventing a passing test.
- No duplicate audio owner, surface owner or render loop. Future audio interruption/mute and
  context/texture recovery need a concrete consumer plus real target evidence.

## Required acceptance observations

Use the foundation ledger; the following lists **platform acceptance criteria**, not a
parallel evidence ladder or an implemented automated runner:

| Gate | Required observation |
| --- | --- |
| Standalone first | Install/start/render/touch on the named physical Android/iPhone target; packaging alone insufficient |
| Shell/game boundary | Correct versioned command/result, overlay/back behavior, exact incompatible-result handling |
| Re-entry | 30 enter/exit cycles without process resets, crashes, stale handles or accumulating resources |
| Background/resume | 20 cycles with correct input reset and surface/context/texture recovery |
| Layout | Resize/orientation/safe area and overlay scaling at recorded density/font scale |
| Input | Multi-pointer IDs/cancel, overlay routing, keyboard/gamepad only if actually supported |
| IME | Vietnamese composition/focus trace when implemented, no ghost movement |
| Cleanup | Owned thread/loop/listener/handle/resource counts or explicit unsupported observation |
| Performance | Named physical hardware/OS/GPU, build/fixture, frame pacing and memory instrumentation |

Desktop/JVM/Studio layout captures can inspect shell appearance only. Emulator/simulator
results identify that renderer and cannot replace physical-device rows. Current iOS build
selection returns BLOCKED; do not copy `iosSimulatorArm64` framework or preview commands from
Cantos into MyVa. Read the actual Gradle targets before deciding what can run.

Examples of real command surfaces:

```sh
cargo xtask doctor mobile --platform android
cargo xtask build native-probe --platform android --mode check
cargo xtask build native-probe --platform android --mode apk
cargo xtask build native-probe --platform android --mode cmp
cargo xtask devices --platform android
```

`check` does not link, `build` links `.so`, `apk` packages standalone and `cmp` packages the
shell experiment. `devices` reads an already-running loopback ADB server; do not restart it,
boot an emulator, choose a serial, install or launch as an accidental side effect. Follow the
user's authorization for explicit runtime actions. No iOS/native device gate is green because
Android Kotlin compiled or ELF/ZIP alignment passed.
