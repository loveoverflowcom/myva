# Delivery boundaries and requested publication

[Engineering](../../myva-engineering/SKILL.md) owns the invariant/evidence method;
[planning](../../../../docs/work-plan/planning.md) owns MyVa workflow semantics.

## Boundary statement

Record: “This slice delivers <observable outcome> for <consumer>; it does not deliver <adjacent
capability>. Its oracle is <independent observation>. Prerequisites are <evidence or unknown>;
<target/gate> remains open with <owner/next action>.” One issue can need several independently
reviewable PRs. Missing native acceptance does not erase a verified web/core slice, and that slice
does not close native acceptance. Do not build a fake adapter to remove a blocker on paper.

## MyVa selection traps

- D04 core/ECS can proceed independently of mobile rendering; web, standalone Android and CMP
  integration have separate gates in the [queue](../../../../docs/work-plan/README.md).
- [Equipment](../../../../docs/technical/equipment-registry.md) validates immutable builds and
  pure formulas today. Runtime effects require a separate core/replay-version work item.
- [Online pilot](../../../../docs/work-plan/backlog/online-pilot.md) is future work, not a reason
  to scaffold Axum/PostgreSQL or claim server integration for a local authoritative-core test.
- An acceptance/design ambiguity needs an explicit proposed decision and owner. Do not invent
  roster approval, lore mappings, thresholds or a new platform contract to make a task finish.

## Publication when authorized

1. Recheck `git status --short`, diff and staged paths. Preserve unrelated work. Stage explicit
   intended files; exclude logs, credentials, downloaded media and generated build artifacts.
2. Run applicable owner gates, `cargo xtask fmt --check`, `cargo xtask verify repository`,
   `cargo xtask verify xtask` for tooling, and `cargo xtask preflight --changed-only`.
   Retain raw output under ignored `target/xtask-evidence`; disclose failure/blocker honestly.
3. Follow existing history/template conventions. State base/source SHA, scope/non-goals, provenance,
   actual commands/statuses, evidence limitations and remaining acceptance. Link the issue.
4. Push the authorized branch and create the requested PR against `develop`; do not infer permission
   for issue/Project mutations or deployment. If only part of an issue is done, say `Part of #N`.
5. Report the verified URL, commit SHA, checks, review coverage and residual risks. Pending CI is
   pending, not PASS; follow the user's requested CI monitoring scope rather than inventing a gate.
