---
name: myva-cmp-engineering
description: >-
  Implement or review MyVa Kotlin Compose Multiplatform shell and Android/iOS native
  feasibility mechanics: source sets, navigation, Bevy/Winit surface and event-loop ownership,
  input cancellation, IME, lifecycle and cleanup. Distinguishes shell preview, standalone
  packaging and actual CMP/native device acceptance; does not claim an existing iOS adapter.
metadata:
  owner: myva-cmp-engineering
  source_repository: https://github.com/loveoverflowcom/cantos
  source_revision: f05b09bf24243bb99bc869f5009628294ebb15d8
---

# MyVa CMP engineering

Platform-mechanics owner only. Compose [engineering](../myva-engineering/SKILL.md) for
invariants, types/errors, L1–L5, vocabulary and report. Compose
[UI design](../myva-ui-design/SKILL.md) for visible changes and
[visual review](../myva-visual-review/SKILL.md) for inspected pixels.

## Invocation

Use `$myva-cmp-engineering` for `prototypes/native` Kotlin/CMP, native shell or mobile
lifecycle work. Example: “Use `$myva-cmp-engineering` to investigate return-to-shell cleanup;
keep the one-shot Android limit explicit and identify the device gate before proposing a fix.”
Review-only requests use [code review](../myva-code-review/SKILL.md) without editing.

## Repository reality

Read [prototype README](../../../prototypes/native/README.md),
[native feasibility](../../../docs/reports/native-feasibility.md),
[native work item](../../../docs/work-plan/010-native-feasibility.md),
[architecture](../../../docs/technical/architecture.md) and actual Gradle/source files.
`prototypes/native/android/shared` has real `commonMain` shell/overlay UI; target configuration
is Android only. `cmp-shell` opens native `GameScreenActivity` with versioned Intent/result
and a Compose overlay. Bevy/GameActivity owns the surface. This is option B, deliberately
one launch per process. It is not an embedded option-A adapter or proven mobile integration.

Android standalone/check/link/APK/CMP packaging and Linux event-loop probe results do not
close real-device gates. iOS packaging, host target and accepted custom runner are absent;
stock runner conflicts with UIKit application ownership. Keep those boundaries BLOCKED or
NOT_RUN as actually observed. There is no inherited Cantos audio/session/preview/Quint runner.

## Platform workflow

1. Name exact entrypoint, source sets, host, target, variant and lifecycle/resource owner.
   Apply the foundation's construction/error method to any new Kotlin state, protocol or
   factory; inspect `copy`, decode/restore and cancellation paths.
2. Read [native mechanics and gates](references/native-lifecycle.md) before changing
   navigation, surface, event loop, input, background/resume, overlay, IME or cleanup.
   Do not replace one-shot gating with unsupported recreation or manufacture FFI handles.
3. Keep common UI platform-neutral and value/event based. Android `Intent`, Activity,
   `GameActivity`, JNI and native-surface details stay in the Android host. Do not create
   an `iosMain` implementation merely to claim parity. Shell never owns combat rules.
4. Discover current commands through [xtask](../../../xtask/README.md); use `doctor mobile`,
   `build native-probe` modes and `devices` as available. Selection/execution belongs to
   [test automation](../myva-test-automation/SKILL.md). No auto-install, boot, install/launch
   or device choice is implied by discovery.
5. Test the actual changed boundary, preserving foundation evidence distinctions. No shell
   preview runner is currently registered: a future preview needs an actual host/wrapper,
   verified renderer/tooling and real output, not a copied Cantos command. Preview never
   substitutes for native integration.
6. For visible changes capture/open the same before/after scenario, then report platform
   additions below with the foundation report. Unavailable device gates remain outstanding.

## Platform report additions

Record target/source set, exact artifact/build identity and current manifest/lockfile pins,
physical device versus emulator/simulator/desktop/Studio renderer, OS/GPU/ABI, viewport/density,
orientation/safe area, locale/theme/fontScale, fixture, selected/executed tests, cycle counts,
surface/input/IME/cleanup observations and exact blocker. An iOS build that never existed
cannot become a simulator PASS; a desktop shell image cannot establish touch or mobile lifecycle.
