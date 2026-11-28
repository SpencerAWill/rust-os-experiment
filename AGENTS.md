# Guide for AI coding assistants

Read this first. The rules are in [CONTRIBUTING.md](CONTRIBUTING.md), the workflow in [docs/workflow.md](docs/workflow.md), and what is planned on GitHub (chunk issues, milestones and the "OS backlog" Project; [docs/workflow.md](docs/workflow.md#from-issue-to-chunk)), with milestone goals in [docs/roadmap.md](docs/roadmap.md).

## Commands

```sh
cargo xtask ci          # every check CI gates on; run it before saying work is done
cargo xtask ci --quick  # fmt check, clippy, and tests only
cargo xtask test        # nextest plus doctests
cargo xtask doctor      # host tools, firmware and QEMU smoke boots on this machine
```

The other commands are in [xtask/README.md](xtask/README.md).

## Rules that fail CI (do not work around them)

- Formatting, clippy with warnings denied, docs without warnings, `cargo deny`, typos.
- `check-workflows`: actions pinned to commit hashes, read-only tokens, no `pull_request_target`.
- `check-docs`: a README in each listed directory, relative links resolve, lesson and rule formats, this file under 80 lines.
- No code copied from other projects, and no code reused from AI conversations used as research ([ADR 0001](docs/adr/0001-licence-and-originality.md)).
- The `unsafe` policy is not decided yet ([ADR 0002](docs/adr/0002-unsafe-policy-undecided.md)); do not write kernel code before it is.

## The loop

Work is done in chunks, each one GitHub issue scheduled in a milestone, each through seven stages: brainstorm, grill, plan, work, review, compound, close out ([docs/workflow.md](docs/workflow.md)). A chunk is of **kind** `build` or `research`; research chunks differ in stages 4 and 5. In Claude Code, `/chunk next` starts the next scheduled one, `/chunk resume` continues one, and `/chunk chore #N` runs a small scheduled chore (build work only) without the stages.

- **Wait for the maintainer only three times:** spec approval (at the end of the grill), a pause after the plan commit (to clear the context, [docs/workflow.md](docs/workflow.md#the-loop)), and the merge go-ahead. Between them, move to the next stage without asking.
- **Roles.** The main session writes documents, dispatches agents (`.claude/agents/`) and commits. A `builder` (Sonnet) writes product and test code; a `researcher` writes research documents with citations; independent reviewers check the work and never see the author's reasoning.
- **Stop and ask** when:
  - the spec is wrong or silent on something that changes what gets built;
  - a change would touch an ADR or a CI-enforced policy the spec does not name;
  - a reviewer and the author still disagree after two fix rounds;
  - an action is outward-facing beyond the chunk's branch, pull request and follow-up issues.
- **Pushes to `main` and force pushes are blocked** for agents by a hook (`.claude/hooks/guard-push.sh`). Do not work around it; the maintainer runs such a push themselves.
- **Write it down as it happens** in the chunk's `log.md` (decisions, surprises, `lesson?` candidates). Agents get killed; what is not on disk is lost.
- **Lessons** in [docs/lessons/](docs/lessons/) are retrieved by path when planning. Follow them by default; say why when you deviate.
- **Plugins** may be used inside any stage, but this workflow decides where files go: specs, plans and logs live in `docs/chunks/<id>-<slug>/`, never in a plugin's own folder.

## Always

- **Test first.** For a bug fix, write the test, watch it fail for the reason you expect, then fix the code. Never weaken or delete a failing test without saying so.
- **Verify before claiming.** Run the command and read its output; do not infer success from the diff. In research, a claim needs a cited source or a command that was run.
- **Specifications**: check a claim against `reference/text/` first, then the publisher's site, then write it as unverified; cite the document id, version and section ([reference/README.md](reference/README.md#citing-a-specification)).
- **Small logical commits**, conventional messages ending with the chunk ID (for a chore, ` (#N)`, its issue number), files staged by explicit path. Say in the pull request that an assistant was used.
- **Report honestly**: a failed test, a skipped step or an unverified claim is said, with the evidence.
- **Issues**: the path from issue to chunk and the Definition of Ready are in [docs/workflow.md](docs/workflow.md#from-issue-to-chunk); `/triage` defines issues without building anything.
- **Docs**: new documents get a status line; hard-to-reverse decisions get an ADR; spec section numbers stay stable; write the minimum and link instead of copying.

## Where to look

In Claude Code, `.claude/rules/` loads the pointer for a path when you touch it.

| Working on | Read |
|---|---|
| `xtask/` | [xtask/README.md](xtask/README.md) |
| `spikes/` | [spikes/README.md](spikes/README.md) |
| CI, workflows, `infra/` | CONTRIBUTING.md "What CI enforces"; [infra/github/README.md](infra/github/README.md) |
| docs, a chunk's documents, research | [docs/README.md](docs/README.md); [docs/chunks/README.md](docs/chunks/README.md); [docs/research/README.md](docs/research/README.md) |
| `.claude/` | [.claude/README.md](.claude/README.md) |
