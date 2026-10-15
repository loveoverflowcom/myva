# Verification and evidence

This is MyVa's sole engineering evidence vocabulary, L1–L5 ledger and technical report owner.
Task skills select a workflow; [test automation](../../myva-test-automation/SKILL.md) selects
and executes existing runners. Neither creates another verification ladder.

## L1–L5 per claim

| Layer | Obligation | MyVa scope |
| --- | --- | --- |
| L1 — Abstraction and architecture | Name invariant, failure mode, owner and inward dependency direction | Rule in sim/economy, ECS adapter, Bevy presentation, shell or tooling; review actual imports/manifests |
| L2 — Domain, types and errors | Checked immutable facts, exclusive mutable-state ownership, fallible constructors, closed exact outcomes, atomic refusal | Pins/loadouts, integer arithmetic, command authority, cooldown/proc state; audit bypasses |
| L3 — Unit/property/static | Cheapest deterministic semantic regressions, applicable laws, format/lint and static checks | Plain core tests, bounded cases, golden replay and deterministic fixtures without GPU/network |
| L4 — Adapter/orchestration | Production adapter with deterministic named inputs/fakes, mappings, scheduling and cleanup | ECS/core per-tick parity, mirror readback, input edges, fake asset boundaries or bridge projections |
| L5 — Integration/runtime | Cross a real runtime/platform boundary for the selected claim | Native/WASM comparison, real browser flow, asset loader/render lifecycle, named desktop/mobile target; future real server/DB only when implemented |

Complete L1–L3 before new effectful wiring. A mapping-only adapter with no domain decision may
start at L4 with a recorded reason. A focused core delivery need not wait for a missing device,
but its unrun platform acceptance remains explicit. Docs/skills-only work follows repository
checks rather than inventing application tests. An inapplicable layer needs a reason, not PASS.

Maintain one row per claim with: **owner, failure mode, oracle, source revision/dirty state,
layer, evidence level, actual status, command/artifact, covered inputs and remaining gap**.
Two rows may use the same runner but support different claims. Do not mark a whole feature
verified because one runner exited successfully.

## Evidence vocabulary

| Level | What it supports |
| --- | --- |
| documented | Named requirement or manual source/ownership review, not execution |
| type-enforced | A real construction/access barrier rejects an alternative; identify scope and bypass audit |
| statically-checked | Named compiler/lint/exhaustive/static check actually executed |
| compiled | Build succeeded; no runtime implication |
| example-tested | Exact named inputs, errors and resulting state asserted |
| property-tested | A stated law over recorded generated inputs/seeds/bounds |
| differentially-tested | Independent model/golden/other execution agrees for named observations; disclose shared code |
| fault-injected | A named duplicate, timeout, crash or rejection point was actually injected |
| mock-integration-tested | Real modules composed with named external doubles, not a fake tested alone |
| integration-tested | A real boundary exercised with its actual implementation and named environment |
| mutation-tested | A real bounded campaign tested assertion sensitivity; list survivors/timeouts |
| bounded-model-checked | Actual verifier output for production symbols and stated bounds/assumptions |
| model-sampled | Actual state-model simulation/run and reachability controls; not production conformance or exhaustive proof |
| fuzz-tested | A real named parser target/corpus/budget ran for robustness, not correct semantics |
| DOM-tested / semantics-tested | Live DOM or named Compose renderer structure/semantics asserted |
| interaction-tested | Input events drove the named real flow; not visual acceptance |
| screenshot-captured | Image exists; pixels may remain unopened |
| screenshot-inspected | Named reviewer opened and judged that image |
| cross-viewport-inspected / cross-theme-inspected | Opened images at every stated viewport/theme |
| accessibility-checked | Named semantics, keyboard/focus/contrast checks; not a screen-reader claim |
| screen-reader-walked | A named reader walkthrough actually happened on the named target |
| physical-device-tested | Actual named physical Android/iPhone/other hardware run with OS/build identity |
| emulator-tested / simulator-tested | Actual named emulator or simulator run; never substitutes for physical hardware |
| production-observed | Named production observation, not a proof or available MyVa capability |

These are evidence labels, **not runner statuses**. MyVa's
[xtask contract](../../../../docs/architecture/xtask-build-contract.md) supports exactly
`PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`: a stage ran successfully, actually failed, was blocked
by prerequisites/unsupported execution, or did not run. Preserve what the runner printed.
Do not copy Cantos `UNSUPPORTED`, invent `SKIP`/`CORE_VERIFIED` statuses or synthesize a green
summary. A proposed gate is described as proposed with NOT_RUN, not as an available command.
Zero selected/executed tests, stale reports, list-only and dry-run cannot satisfy a gate;
missing capability or empty suite must be explicit and nonzero in its runner.

## Independent oracles and regression design

1. Choose an observable contract, not current implementation output as the specification.
2. Assert exact outcomes and complete affected state/effect order; `is_err()` and “no panic”
   are too weak for semantic rejection. A parser fuzzer has the narrower robustness oracle.
3. Keep expected values independent: hand-derived bounded arithmetic cases, frozen reviewed
   replay bytes, conservation identities, a simple separately justified model or real second
   execution. Calling `dash_cost` both to compute expected and actual is circular.
4. Cover meaningful boundaries: empty/min/max/overflow, before/at/after tick, duplicate versus
   new action identity, same actor versus another actor, exact/stale/missing pin, wrong epoch,
   late/out-of-order messages, repeated enter/exit and cancelled input where relevant.
5. For a bug, preserve a reproducer that fails on the old behavior and passes after the fix.
   For a refactor, characterize behavior first and avoid a simultaneous rules redesign.
6. Broaden only when justified: finite exhaustive loops → properties/state machines → replay/
   differential/fault injection → runtime. Optional formal/mutation techniques follow
   [their adoption criteria](mutation-and-formal.md), not a coverage quota.

Concrete distinctions that must survive reporting:

- Core tests assert rules. ECS/core parity checks adapter scheduling and observations but
  shares the production core; neither proves independent combat semantics. Native/WASM
  comparison proves only the executed fixtures/builds/target equivalence.
- Browser interaction and DOM assertions do not show anyone opened captured images. Open
  before/after pixels for a visual claim; cite build/scenario/fixture/viewport/theme/locale/
  text scale. Touch emulation is not physical touchscreen qualification.
- CPU asset validation/planner/handle/refcount results do not measure GPU allocation, texture
  reclamation, frame pacing or hardware performance. SwiftShader is software rendering.
- A desktop/JVM/Studio preview is not Android/iOS integration. Android package/ELF alignment
  is compiled/packaged evidence, not mobile runtime; simulator is not physical-device proof.
- Offline economy conservation is not durable online atomicity. Pure equipment arithmetic
  is not activated combat balance, verified contact authority or exploit prevention.
- A skills validator validates structure, not actual runtime discovery, routing selection or
  agent behavior. Report static routing cases versus actual runtime evaluation separately.

## Completion report

Use a concise report or equivalent ledger retaining every field:

```text
Invariant / changed behavior:
Owner / boundary / failure mode:
Oracle / independence and shared assumptions:
Source revision / dirty state / fixture and tool identities:
Evidence per claim: layer; level; PASS|FAIL|BLOCKED|NOT_RUN; exact command; artifact; covered scope
Actual failures: named assertion/command and outcome, separate from blockers
Execution blockers: exact missing prerequisite/unsupported boundary
Deferred acceptance: required unrun gate; prerequisites; command if one exists; acceptance criterion; next owner
Edge classes covered:
Not covered / residual risk:
```

A required but unavailable gate remains required. Record an unassigned owner honestly; do not
invent one. Keep raw logs in ignored evidence directories and a small sanitized summary for
review. Opening a PR does not establish CI, runtime discovery, gameplay balance or phase exit.
