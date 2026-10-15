# MyVa agent skills

Twelve canonical workflows, adapted to current MyVa rather than the historical
docs-only assumptions in issue #7. Product rules remain in [docs](../../docs/README.md)
and source. Skills do not add a feature, a permission, or a passing platform gate.

## Responsibility index

| Skill | Owns |
| --- | --- |
| [myva-engineering](myva-engineering/SKILL.md) | Invariants, typed boundaries and evidence ownership |
| [myva-cmp-engineering](myva-cmp-engineering/SKILL.md) | CMP shell mechanics and honest native acceptance |
| [myva-web-engineering](myva-web-engineering/SKILL.md) | Leptos shell, Bevy bridge and browser verification |
| [myva-game-audit](myva-game-audit/SKILL.md) | Audit combat, ECS, replay, assets, economy and authority |
| [myva-code-review](myva-code-review/SKILL.md) | Read-only revision-pinned review with independent oracles |
| [myva-visual-review](myva-visual-review/SKILL.md) | Inspect captured pixels with reproducible scenario context |
| [myva-test-automation](myva-test-automation/SKILL.md) | Select existing xtask runners and retain honest outcomes |
| [myva-work-item](myva-work-item/SKILL.md) | Deliver one scoped work item through existing rule owners |
| [myva-diagnostics](myva-diagnostics/SKILL.md) | Summary-first failure triage with attributable raw logs |
| [myva-logic-fix](myva-logic-fix/SKILL.md) | Repair one invariant defect with a failing regression |
| [myva-ui-fix](myva-ui-fix/SKILL.md) | Repair one presentation defect and recheck its scenario |
| [myva-ui-design](myva-ui-design/SKILL.md) | Design clear accessible shell and gameplay presentation |

Engineering owns invariants, vocabulary, verification selection and the completion report.
Platform owners define actual CMP/Web mechanisms. Task entrypoints compose those owners;
test automation selects existing runners rather than redefining verification.
A PR/range/diff review enters code-review; a domain/capability audit, including ECS
lifecycle or asset/performance claims, enters game-audit even when read-only.

## Use and discovery

Open the relevant canonical `SKILL.md`, then load only needed references. Each entrypoint
has a literal `$myva-…` invocation. `.claude/skills/<name>` links to the same directory;
never maintain a second copy. If automatic discovery is absent, explicitly read the canonical
file as described in [Claude routing](../../CLAUDE.md).

The [routing cases](routing-cases.md) distinguish structural validation, a manual routing
exercise and automatic runtime discovery. A validator PASS proves none of the latter two.

## Provenance and capability

See the [transfer and capability report](../../docs/reports/agent-skills-transfer.md) for the
pinned source, retain/adapt/exclude/proposed mapping, actual commands and residual risks.
Use [cargo xtask](../../xtask/README.md) for repository operations. Raw logs belong in
`target/xtask-evidence/` (ignored), not committed alongside this index.
