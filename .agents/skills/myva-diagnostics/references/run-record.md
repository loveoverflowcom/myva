# Summary-first investigation and run record

Use the [xtask contract](../../../../docs/architecture/xtask-build-contract.md) and
[engineering](../../myva-engineering/SKILL.md), not Cantos commands/statuses/toolchain pins.

## Capture without losing the real exit code

Confirm ignore first: `git check-ignore target/xtask-evidence/diagnostics/<run-id>/probe`.
Create only this run's directory. For each command capture full combined stdout/stderr to its own
log and record the command, cwd, source SHA/dirty state, target, tool versions, start/end and exit.
With `tee`, enable `set -o pipefail` and capture the actual command status, not the last pipe tool.
Never claim an agent-written summary is runner output. Keep raw logs local; redact tokens,
signed URLs and private data from shared excerpts. Do not print environment/credential dumps.

## Progressive reads

- Summary, about 40 lines: `wc -l`, `tail`, `grep -nE '^(==> |PASS:|FAIL:|BLOCKED:|NOT_RUN:)'`.
  Add the first `error`, failing test name, `Diff in`, Python failure or Gradle failed task.
- Focus, about 150 lines: one failing stage/test with a small numbered context window.
- Deep trace, bounded slices: only for a written hypothesis. Use scoped `RUST_BACKTRACE=1`,
  runner-supported test filters or a browser/runtime trace; never persist global verbosity.
- A filter matching zero lines means verify the filter/output shape before concluding nothing failed.
  Zero selected tests never supports PASS. Prefer existing `cargo xtask verify <target>` filtering;
  do not create a second orchestration entrypoint for a primitive probe.

## Classification is separate from result

| Classification | Required discrimination |
| --- | --- |
| baseline / pre-existing | Same normalized signature on isolated pinned base with identical command/environment |
| environment | Named missing prerequisite or restriction, confirmed by doctor/probe; not just a plausible story |
| flaky | Unchanged inputs produce different outcomes in a bounded observation (normally at most three runs); retain every failure and investigate nondeterminism |
| regression | Fails on changed source, passes on base in the same environment with a smallest reproduction |
| test defect | Expected value contradicts an authoritative contract; never infer from convenient new output |
| unknown | State rival hypotheses and the discriminating observation still missing |

Normalize only volatile timestamps, PIDs, ports and checkout paths in a comparison; retain original
bytes. Use a separate worktree/target directory for baseline, never reset the user's tree. Record
baseline SHA and command/environment parity. Do not delete historical evidence or another run's files.

## Hypothesis and cleanup

Record one row per hypothesis: cause, prediction, discriminating command, observed result,
confirmed/refuted/unresolved, raw-log pointer. Three iterations with no discriminating result, or
two speculative patches with the same signature, mean stop patching and report the missing evidence.
No identical rerun except the bounded flakiness experiment.

Temporary adapter probes carry `MYVA_DIAG_TEMP` plus run ID, avoid secrets, and add no dependency,
public contract or gameplay behavior. Search outside skills for the exact marker before handoff;
remove only this run's marked lines and inspect the diff, never restore whole dirty files. Record
owned processes/worktrees cleaned up, verbosity restored and any retained evidence location.

The run's short index records question; source/environment; failure signature/classification;
hypotheses; gate/status/log pointers; cleanup; next owner/action. Raw evidence stays under ignored
`target/xtask-evidence`; promote durable knowledge into tests/owning docs only within task scope.
