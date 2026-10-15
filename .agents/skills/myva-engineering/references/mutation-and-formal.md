# Targeted mutation, Kani and Quint

Adapted method only. **MyVa has no adopted mutation, Kani or Quint executable gate in this
skills change.** Cantos's model catalog, harness registry, toolchain pins, production kernels
and results are not MyVa capabilities. Do not copy their commands, source IDs, solver pins,
models, tool bootstrap or reports. This reference proposes criteria, not installation.

Read [verification](verification.md) first. Begin with a real invariant, production seam and
ordinary deterministic tests. Use one technique for a concrete uncovered failure mode, not
whole-repository formalization or a percentage target.

## Selection

| Technique | Consider when | Not for |
| --- | --- | --- |
| Mutation | A stable pure kernel has tests but assertion sensitivity is uncertain; e.g. exact-pin comparison, checked budget/rounding, cooldown boundary | Generated code, incidental text/getters, broad renderer/browser or changing UI |
| Kani | Small critical Rust arithmetic/transition kernel; finite exhaustive loops are inadequate; production symbols and supported operations can be called directly | Whole ECS, GPU/FFI behavior, network/DB, Kotlin/Compose or arbitrary assumptions that erase hard cases |
| Quint | A reviewed bounded stateful protocol has nontrivial sequences/authority/re-entry/atomic refusal and an independent observation seam | Inventing gameplay requirements, hand-copied reducers, declaring a model PASS to mean implementation/device PASS |

Example proposals, not implemented targets: pin rejection without partial spawn; equipment
rounding/overflow preservation; command epoch/sequence admission; a bounded proc cooldown
state model. Confirm the owner and actual code first. Do not activate equipment combat to
justify a formal target. Simple table tests may already be sufficient.

## Adoption gate before adding an executable check

Require all of the following in one reviewable proposal/implementation:

- A named production kernel or accepted model, requirement owner and exact source reach.
  A copied demonstration function is not verification of production behavior.
- A justified tool/compiler/solver/platform compatibility decision from actual MyVa inputs,
  with explicit domain, bounds, assumptions, exclusions and resource limits.
- An existing-xtask-integrated runner with precise selection, fresh provenance/raw output,
  nonzero on missing capability, zero targets, parser/runtime failure or incomplete result.
- Positive execution plus negative controls that actually expose the chosen defect. A parser
  error, unsupported operation, timeout or illegal trace is not a mutation kill.
- Actual attributable run results and an ordinary deterministic regression for meaningful
  counterexamples. Until these exist, status is NOT_RUN and capability is **proposed**.

No speculative dependency, standalone orchestration, automatic install or production toolchain
change is authorized merely by a documentation migration. An explicit user-approved toolchain
upgrade is separate scope, not inherited proof capability. Future executable gates need their
own scoped work item and reviewer decision.

## Mutation obligations

Run baseline tests first; a red baseline invalidates a campaign. Record tool/version, source
and test revision, selected mutants, executed/killed/survived/unviable/timeout counts and raw
output. Classify survivors: genuine gap, equivalent, unreachable, verifier-only, tool limit or
low value, with a reason. A timeout may expose a termination problem. Exclude verifier-only
harness code explicitly and verify discovery; otherwise uncompiled mutations falsely survive.
A killed mutant establishes sensitivity to that defect, not correctness of the specification.

## Kani obligations

The harness must call actual production symbols. Record symbolic raw inputs (including invalid
factory inputs), assumptions and the real producer that establishes each; widths, features,
loop/sequence/allocation bounds, stubs and unsupported external behavior. Keep unwinding checks
and meaningful acceptance/rejection/boundary cover goals. An assumption cannot assert the
result, prune exhaustion or make the antecedent unreachable. Investigate vacuity, failed
covers and unexpected unreachability. Compile/source presence or ordinary tests are not
`bounded-model-checked`; only a real verifier run for these exact bounds supports that label.

## Quint obligations

Separate approved requirements, implementation facts, finite assumptions and unresolved
semantics before modeling. Unresolved decisions remain proposals, not model-created truth.
Give properties stable IDs, exact named runs, reachable antecedent witnesses and typechecking
negative controls. Control runs must violate their intended property, not merely crash.

Model simulation supports `model-sampled`, not exhaustive proof or production conformance.
For conformance, independently observe the actual production seam using event/input traces;
do not supply expected outcomes to it or maintain a second handwritten implementation reducer.
Preserve every assertion checkpoint/effect count and first semantic mismatch. Include a
control that exposes drift in the observation adapter itself, separate from model mutants.
Keep native numeric/codec/constructor tests and runtime/platform tests. A finite sequential
model cannot establish fairness, concurrency, integer completeness, GPU or device behavior.

For any real future campaign report model/kernel/tool/config/source identities, command,
selection/execution counts, bounds, assumptions, witness/control outcomes, counterexamples,
verified scope and residual risks using the foundation report. Never silently weaken a
requirement or expected oracle merely to make a counterexample disappear.
