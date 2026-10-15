# Construction barriers and typed errors

This is the shared Rust/Kotlin method. Platform skills supply concrete interop mechanics.
A private refined type is runtime-checked evidence with compiler-enforced provenance, not a
mathematical proof. Existing [registry](../../../../packages/sim/src/registry.rs) and
[equipment](../../../../packages/sim/src/equipment.rs) are useful real boundaries to inspect,
not declarations that every construction path is already hardened.

## Invariant-first construction

1. State the proposition and numeric/relational boundaries. Distinct IDs/units prevent mixups;
   an aggregate checks relationships that individually valid fields cannot prove.
2. Find **every** entry: public fields/literals, constructors, `Default`, `From`/`TryFrom`,
   parsers/`Deserialize`, restore, snapshots, foreign decoding, clones/copy, builders, fixtures,
   tests and trusted internal conversions. Search callers, not only the type declaration.
3. Prefer the smallest barrier: private newtype + fallible factory, a struct of refined fields,
   a closed enum, or an exact-bound evidence value. Typestate is optional, not a new framework.
4. Require the validated aggregate at consumers. Keep raw DTOs raw until checked conversion;
   do not pass a primitive plus a forgotten “validated” boolean.
5. Test partitions at below/at/above limits, zero/max, empty/duplicate/malformed values and
   relation failures; assert exact variants and complete unchanged rejected state/effects.

Rust `TryFrom<Raw>` and serde `try_from` can share the factory; deriving `Deserialize` on private
fields alone does not call it. `From` is only for truly infallible conversion. `Default`, setters,
`DerefMut`, mutable collection getters and unchecked trusted constructors can reopen the barrier.
Do not add one for fixture convenience. Existing raw spawn/fixture APIs are authored seams,
not authority for untrusted loadout selection; name and constrain their callers.

Kotlin uses distinct value types, private constructors/factories and sealed outcomes with
exhaustive `when`. `val` is not deep immutability; mutable collections, generated `copy`,
serialization and restore can bypass a factory. Check the actual pinned compiler's generated
copy visibility and choose an ordinary class when needed. Do not adopt Arrow or another
library solely for syntax or claim Kotlin refinement proofs.

Illustrative, not an existing MyVa API:

```rust
struct RawWindow { start: u32, end: u32 }
struct TickWindow { start: u32, end: u32 } // private fields
#[derive(Debug, PartialEq, Eq)]
enum WindowError { Reversed }
impl TryFrom<RawWindow> for TickWindow {
    type Error = WindowError;
    fn try_from(raw: RawWindow) -> Result<Self, Self::Error> {
        if raw.start > raw.end { return Err(WindowError::Reversed); }
        Ok(Self { start: raw.start, end: raw.end })
    }
}
```

A test of this constructor alone says nothing about a restore path constructing the struct
inside the owning module. Exercise each relevant bypass and each public invariant-preserving
operation. Equality, ordering, hash and serialization must match domain meaning.

## Error meaning and composition

| Kind | Owner / handling |
| --- | --- |
| Expected rule rejection | Closed domain outcome (`RegistryError`, command `Rejection`, etc.); correct/reconcile input, never retry unchanged denial blindly |
| Validation | Factory/validator with deterministic field/rule attribution; accumulate independent findings only where promised |
| Infrastructure failure | Concrete adapter maps foreign failure into typed boundary error; bounded retry depends on safety and known outcome |
| Unexpected fault/corruption | Contain at outer host, fail closed, preserve diagnostics; do not disguise as user validation or fabricate success |

Use Rust `Result<T, E>` for failure, `Option<T>` for meaningful absence. Kotlin sealed outcomes
or an already-adopted typed error API preserve alternatives; `Result<T>` carrying only
`Throwable` is not a closed domain taxonomy. Compose dependent work with `?`/`and_then` or
exhaustive `when`; `map` changes success and `map_err` translates failure at the owning boundary.
Do not turn errors into absence/default success through `.ok()`, `getOrNull` or broad fallback.

Independent validation may accumulate findings; dependent work must stop when its prerequisites
fail. Keep exact variants until a real boundary deliberately projects safe UI/CLI text. A
catch-all on a closed domain enum hides new cases; handle unknown foreign inputs only at their
adapter with an explicit unsupported/invalid outcome. Do not introduce a cross-domain mega-enum.

No expected caller-controlled path uses `unwrap`, `expect`, panic, `!!`, `require`/`check`,
unchecked indexing or exception-driven flow. Internal assertions diagnose programmer defects
only after validated premises. Catch foreign exceptions around the smallest concrete call,
not whole use cases; preserve Kotlin `CancellationException` and fatal-runtime behavior.
Rust cancellation/drop is not automatically an `Err`, and `panic=abort` cannot be unwound.

## Atomic refusal, retry and freshness

Typed refusal must precede spawning/effects or leave the agreed state unchanged. For loadout,
validate exact kit/equipment/rules pins before spawning and preserve the old bout on failed
rematch. For command rejection, preserve accepted sequence/queue state under its contract;
for resource/proc changes inspect the complete owned state, including watermarks and cooldowns.

Cancellation stops automatic retries and releases owned handles/listeners. Fence late results by
session/generation and resource/revision; a stale result must not attach to a new game. Timeout
or cancelled effect may have an **unknown** outcome. Reconcile before replaying non-idempotent
work; no fresh operation ID to dodge ambiguity. Retry classification is pure data, execution is
a shell effect. No speculative online retry service is introduced.

Evidence must separately cover factory/error partitions, bypass paths, exact boundary mappings,
unchanged rejections, cancellation/late results and real platform behavior when applicable.
