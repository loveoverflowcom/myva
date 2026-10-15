# Architecture and invariant ownership

Read [architecture](../../../../docs/technical/architecture.md),
[repository layout](../../../../docs/technical/repository-layout.md),
[gameplay foundation](../../../../docs/technical/gameplay-foundation.md) and the changed code.
Older reports describe their revision, not automatically today's capability. This reference
owns the engineering method; linked product documents own rules and numeric values.

## One rule owner

| Location | Owns | Must not acquire |
| --- | --- | --- |
| [sim](../../../../packages/sim/src/lib.rs) | Pure Rust combat, session, protocol, replay, catalogs and validated loadout | Bevy, browser, DB, wall clock, renderer rules |
| [gameplay](../../../../packages/gameplay/src/lib.rs) | `Input → Simulate → Mirror → Events`, one `Session::step` per fixed tick, entity mapping | A second movement/damage/cooldown/status implementation, renderer/window/assets |
| [graybox](../../../../packages/graybox/src/lib.rs) | Game-device input, presentation, assets, HUD and client lifecycle | Authoritative rewards, directly writable rule mirrors |
| [economy](../../../../packages/economy/src/lib.rs) | Offline ledger/budget simulation | A claim that online durable transactions already exist |
| [web shell](../../../../apps/web/shell/src/lib.rs), CMP shell | Navigation, selection, DOM/native shell, focus/IME | Combat state edits, online minting or entitlement |
| Future server | Online command validity, rewards/ledger, inventory and authority | Trust in client-reported damage/balance |

Dependencies point toward pure rules. Add an abstraction/module/service only for a concrete
consumer and real boundary; do not scaffold the proposed online architecture. A fixture or
headless runner is not a production authority service. A mock reward outbox is not a committed
ledger transaction. See [economy requirements](../../../../docs/design/economy.md).

## Functional core without immutable-world dogma

Core decisions take typed facts and deterministic logical time; they return typed outcomes,
state transitions and events. File I/O, browser/native calls, logs, environment and effect
execution stay in their shell. Keep pure arithmetic/validation testable without a window,
GPU, database or network. Do not build a universal effect framework for thin glue.

Distinguish **settled facts** from **owned evolving state**:

- Catalog entries, authored kit/equipment revisions, accepted exact pins, round-locked
  loadouts and exported replay inputs/checkpoints are immutable facts. Corrections create an
  explicit new revision; never resolve “latest” for an already-locked round/replay.
- `World`, `Session`, RNG, action watermarks, command queues and ECS indices evolve under an
  explicit exclusive owner. Encapsulated `&mut self` and local `mut` are normal. Document the
  invariant before/after each observable transition, not every internal intermediate value.
- Mirrored rule components are immutable to client systems. Presentation reads mirror/views
  and events; it cannot use a mutable mirror as a shortcut around core validation.
- Rejection is atomic at its contract boundary. Validate first, compute a small candidate or
  use a narrow rollback-safe operation, then commit once. Assert full affected state and
  ordered effects unchanged, not just HP or the hash if that hash omits relevant fields.
  There is no requirement to clone the whole World every tick.
- A future durable operation still needs transactions, operation IDs and reconciliation;
  returning `Result` alone makes no I/O atomic.

## Determinism and time

`sim` uses fixed-width integers, 60 Hz ticks, deterministic seeded RNG and ordered effects.
Preserve phase order in `World::step` and stable actor ordering in `Session::step`. Do not
introduce wall-clock decisions, output-affecting unordered iteration, floating-point core
arithmetic, pointer IDs or architecture-dependent `usize` encodings. Check overflow,
rounding, exhausted counters and tick boundaries explicitly.

Bevy `FixedUpdate` may run 0..n ticks per rendered frame. Input held state and press edges
must survive that distinction without double-consuming a press. Float `Transform`, camera,
VFX, interpolation and layout remain renderer concerns; presentation precision is not a
reason to replace integer simulation. Native/WASM determinism is a separate executed claim,
not a theorem implied by integers or same-process parity.

Domain IDs are not Bevy `Entity`. `Snapshot` is a minimal view, not complete recovery state;
[restore limitations](../../../../docs/technical/gameplay-foundation.md) include buffered
input, hit tracking and counters. A future restore format requires its own version/invariants.

## Replay, identity and metadata isolation

Read [kit/loadout](../../../../docs/technical/kit-registry-loadout.md),
[equipment](../../../../docs/technical/equipment-registry.md),
[registry code](../../../../packages/sim/src/registry.rs) and
[replay code](../../../../packages/sim/src/replay.rs) before edits.

- v3 is a frozen legacy path: preserve header, encoding and golden bytes. It has no wire pins
  and only accepts its frozen legacy rule/kit revisions. v1/v2/unknown remain rejected.
- v4 records exact rules version/hash and exact per-used-kit ID/version/hash. Reject missing,
  duplicate, extra, mismatched or unknown pins; reject a spawn spec divergent from registry.
  Never choose a current revision as fallback or silently promote v3 to v4.
- `Replay::parse`, `Replay::world` and `Replay::play` protect different construction paths.
  Publicly built/modified replay values must not bypass contract validation during playback.
- Canonical hashes specify widths, little-endian integer encoding, UTF-8 lengths and explicit
  tags. Hashes are fingerprints, not signatures, approval, ownership or authentication.
- Gameplay kit/config hashes, conservative rules-source fingerprint and world state hash
  have different owners. Preserve exclusions: catalog/lore/UI labels/AI/presentation must
  not silently enter rule/state hashes; equipment display names stay out of gameplay hashes.
  Prove metadata-only edits preserve gameplay/replay while exact-pin edits reject stale input.
- Do not alter fingerprinted source just to improve a comment without assessing compatibility.
  The rules-source fingerprint can reject a semantically harmless source edit. No historical
  loader or cross-build migration is implied.
- Equipment is offline validated data plus pure prototypes. A valid nonempty build currently
  returns `RuntimeNotImplemented` before combat spawn. Do not activate effects, ignore gear,
  add equipment to v3/v4 or claim exploit prevention as a side effect of this skill.

Example invariant: “A failed rematch with a stale kit pin leaves the current bout, round,
locked pins and autopilot unchanged.” Oracle: independent before/after observations plus the
exact `RegistryError`; then test core/ECS/replay separately rather than trusting UI text.
