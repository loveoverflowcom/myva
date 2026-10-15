# MyVa agent routing

Start with [README](README.md), [contributing](CONTRIBUTING.md), the relevant
[work item](docs/work-plan/README.md), then the [skill index](.agents/skills/README.md).
Product and architecture facts stay in `docs/` and source; skills own working method.

- Implementation: `myva-work-item`; bounded repairs: `myva-logic-fix` or `myva-ui-fix`.
- Shared engineering/evidence: `myva-engineering`; platform mechanics:
  `myva-cmp-engineering` or `myva-web-engineering`.
- PR/range/diff review: `myva-code-review`; domain/capability audit of gameplay,
  economy, ECS lifecycle or asset/performance: `myva-game-audit` (report-only by default).
  Pixels only: `myva-visual-review`; design: `myva-ui-design`.
- Red gates/logs: `myva-diagnostics`; runner selection: `myva-test-automation`.

Load the named canonical `.agents/skills/<name>/SKILL.md`, then only relevant
references. See [routing evaluation](.agents/skills/routing-cases.md).
Review or diagnosis alone does not authorize repairs or remote actions; current user
instructions determine scope. Use the existing [xtask surface](xtask/README.md).
