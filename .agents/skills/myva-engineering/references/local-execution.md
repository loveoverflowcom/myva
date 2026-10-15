# Local execution and handoff

The [xtask build contract](../../../../docs/architecture/xtask-build-contract.md) is authoritative
for commands, status, prerequisite provisioning, resource limits, process ownership and impact.
[xtask README](../../../../xtask/README.md) is the command entry. This reference adds no runner.

## Work safely and reproducibly

- Inspect branch/status/base and preserve unrelated edits. Record source SHA and dirty input
  identity before checks; review the final diff after formatting. A known baseline failure
  remains an actual failure, not an environment excuse.
- Use `cargo xtask` wherever the operation exists. Scripts/Cargo/Gradle/Node are primitives,
  not competing repository orchestration. If xtask cannot start, state that exact blocker;
  a manually run diagnostic primitive never becomes an xtask PASS.
- Check disk/memory/concurrent work before expensive lanes; one heavyweight build at a time.
  Respect existing two-job default and `CARGO_BUILD_JOBS`. Do not auto-install targets/SDKs,
  npm/browser/toolchain dependencies, kill unrelated listeners or purge shared caches.
- Use `cargo xtask doctor` for prerequisites, then the focused selected runner. Doctor is not
  runtime acceptance. `--dry-run` means NOT_RUN and no execution, even if its plan looks right.
- After the final edit run touched formatters in check mode, focused tests/lint and applicable
  repository/changed-only gates. No green test waives formatting. `verify xtask` applies to
  tooling changes; full `preflight` and `ci` share the existing full planner.
- Changed-only base is `origin/develop` unless explicitly selected. Missing merge-base,
  metadata failure or unknown impact is not an empty green plan. The planner includes
  committed/staged/unstaged/deleted/untracked paths and reverse dependencies.

Example selection for an actual replay fix, not proof that these commands have run:

```sh
cargo xtask verify sim
cargo xtask verify gameplay
cargo xtask verify determinism --ticks 3600 --seeds 1,7,12
cargo xtask fmt --check
cargo xtask verify repository
cargo xtask preflight --changed-only
```

Select web/asset/economy/mobile gates from the actual impact rather than copying this list
blindly. Native/WASM prerequisites and browser prerequisites remain independent blockers.
No unsupported `--skip`, `verify kani`, `verify behavior` or Cantos job options are implied.

## Logs and diagnostics

Keep full raw output and command exit code under ignored `target/xtask-evidence`,
`target/wasm-determinism` or `target/web-evidence` as appropriate; native evidence uses its
ignored prototype location. Never print secrets/environment dumps or commit bulky raw logs.
Read summary first, then narrow around a hypothesis. Do not pipe a check into `tail`/`grep`
and lose exit status/full logs. Reproduction details belong in a small sanitized report.
[Diagnostics](../../myva-diagnostics/SKILL.md) owns investigation/classification workflow.

## Independent handoff

Deliver only what the evidence supports. L1–L3 core evidence may be reviewed while a device,
GPU or other boundary remains unaccepted; do not call the whole feature verified. Required
runtime gates still have prerequisites, acceptance criteria and next owner. Report actual
failing checks separately from baseline, environment, flaky or introduced-regression
classification, including uncertainty rather than silently reclassifying the failure.

Commit/push/PR only within the user's authorization; retain cohesive commits and do not change
CI/protection or suppress checks to manufacture green. A requested PR can disclose incomplete
verification, but cannot claim a clean gate that failed. Read remote CI before reporting its
status. Skills validation PASS establishes structure only; static routing cases and actual
runtime discovery/evaluation are separate results. Never restore deleted source evidence or
mutate a reference repository merely to make local links resolve.
