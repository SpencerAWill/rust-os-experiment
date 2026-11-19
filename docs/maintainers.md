# Maintainer notes

Status: Accepted (2026-11-17). Operational checklists for the repository; the engineering rules are in [CONTRIBUTING.md](../CONTRIBUTING.md).

## Repository settings

Declared in [`infra/github`](../infra/github/README.md) (Pulumi): repository settings, the `main` ruleset (pull request required, every `ci.yml` job a required check, no force pushes or deletion, admin bypass only through a pull request), labels, Actions permissions, Dependabot security updates, private vulnerability reporting. Change them there, not in the GitHub UI. After merging any pull request that changes `infra/github/`: `pulumi preview`, then `pulumi up`.

- [ ] **Never use a self-hosted runner for pull requests from forks.** A runner executes the pull request's code. If one is ever added (for example for hardware tests), restrict it to scheduled jobs on `main`.
- [ ] **Add `CODE_OF_CONDUCT.md`** once there is a contact method for conduct reports.

## Before merging a chunk

- [ ] `cargo xtask ci` passes and the GitHub run is green.
- [ ] The pull request title ends with the chunk ID, for example `(M1.1)`.
- [ ] The spec's status is `Implemented (PR #n, date)`; the plan's is `Historical: executed in PR #n (date)`, with its Deviations section filled in (or "None"); the log's is `Historical: PR #n (date)`; the roadmap row is `Done (PR #n)`.
- [ ] `CHANGELOG.md` is updated.
- [ ] Merge only on the maintainer's explicit go-ahead (in the chunk loop the main session squash-merges, never with `--admin`).

## Issue tracking

The path from an idea to a chunk is in [workflow.md](workflow.md#from-issue-to-chunk). This section is the Project's configuration, which lives in GitHub, not in the repository. `/triage` checks the live Project against these names and stops if they differ.

- **Project:** "OS backlog", number 8, user-owned (`SpencerAWill`; `gh project list --owner SpencerAWill` finds it by title), public, linked to this repository.
- **Status:** Inbox, Ready, Scheduled, Done. **Priority:** Now, Next, Later. **Size:** S, M, L, XL. All single-select. **Priority** Now: blocks current work; Next: a candidate for the next few chunks; Later: worth doing, not soon. **Size** S: under about 100 changed lines or a short note; M: one ordinary chunk; L: a large chunk; XL: more than one chunk, so split it first.
- **Labels** are declared in `infra/github/index.ts`. Every issue carries exactly one type label (`bug`, `enhancement`, `infrastructure`, `documentation`, `research`); `performance` and `needs-measurement` are additive; `area:*` is for code and tools. `research` issues are scheduled work like any other, not an icebox; an idea parked for later stays in the Inbox with Priority Later.
- **Tracking issues:** one per roadmap milestone and one per multi-chunk effort, with sub-issues. Close a milestone's when the milestone completes.

### Built-in workflows and views (web UI only)

`gh` cannot set these. In the Project, **⋯ → Workflows**:

1. **Auto-add to project:** on, repository `rust-os-experiment`, filter `is:issue` (no label exclusion).
2. **Item added to project:** on, set Status to **Inbox**.
3. **Item reopened:** on, set Status to **Inbox**.
4. **Item closed:** on, set Status to **Done**.
5. Turn off any other default workflow that sets a Status this Project does not have.

Views (**+ New view**): **Inbox** (table, filter `status:Inbox`), **Ready by Priority** (table, filter `status:Ready`, group by Priority), **Scheduled** (table, filter `status:Scheduled`), and a **Board** grouped by Status.

## When a milestone completes

- [ ] Tag the commit that completed it: `git tag -a m<N> -m "M<N>: <goal>"`, then `git push origin m<N>`.
- [ ] In the roadmap, mark the milestone `Done` and the next one `Current`, and break the next one into chunks.
- [ ] Run `/lessons-refresh` over `docs/lessons/`.
- [ ] If a new Claude model shipped since the last milestone, re-check the model of each role in `.claude/agents/` against one chunk's results.
- [ ] Close the milestone's tracking issue.
