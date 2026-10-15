# Routing evaluation

[Machine-readable cases](routing-cases.json) hold 11 prompts, expected primary/support
owners, rationale and explicit mutation scope. They cover Rust rules, replay v3/v4,
Bevy lifecycle, asset/performance, economy, CMP shell, web E2E, visual-only inspection,
red gates, authorized review-and-fix and read-only review.

## Evaluation protocol

1. Give an evaluator only each `id`/`prompt`, the [routing index](README.md), and canonical
   skill files. Withhold primary/support/reason so the rubric cannot answer itself.
2. Ask for the selected owner, composed support, authorization boundary and relevant
   evidence limits. Compare semantics against the withheld rubric, not token ordering.
3. Record source revision, evaluator/runtime, inputs, observed routing and differences in
   the [transfer report](../../docs/reports/agent-skills-transfer.md). A manual exercise
   measures these cases only; it is not runtime auto-discovery or task completion.
4. Validate structural integrity with `cargo xtask verify repository`. Negative fixtures
   prove rejection of malformed routing; they do not prove agent routing accuracy.

## Discovery distinctions

- Filesystem: canonical files, frontmatter/display metadata and relative symlinks can
  be checked by the validator.
- Explicit loading: an agent can open a named canonical file and follow its workflow.
- Automatic discovery: only a fresh supported runtime listing/loading the skills can
  establish discovery. Do not infer it from links, metadata, this test or validator PASS.
- Behavioral application: actual task results need the independent evidence prescribed
  by engineering/platform owners; routing selection alone establishes no product claim.

Unknown, missing or not-run stages remain stated explicitly. Do not install a runtime,
copy skills into user-global configuration, or create fake device/runner evidence merely
to make this report green.
