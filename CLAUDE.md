# Claude routing for MyVa

Read [AGENTS.md](AGENTS.md) and the [canonical skill index](.agents/skills/README.md).
`.claude/skills/<name>` is a relative symlink to `.agents/skills/<name>`, not a copy.
If this runtime does not discover symlinked skills, open the canonical `SKILL.md`
directly and invoke its workflow; do not create a duplicate. Structural validation
is not evidence of automatic runtime discovery or successful task execution.
