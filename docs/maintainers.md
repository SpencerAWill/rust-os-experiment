# Maintainer notes

Status: Accepted (2026-11-17); amended by [M0.2](chunks/m0.2-process-sync/spec.md) (2026-11-27). Operational checklists for the repository; the engineering rules are in [CONTRIBUTING.md](../CONTRIBUTING.md).

## Repository settings

Declared in [`infra/github`](../infra/github/README.md) (Pulumi): repository settings, the `main` ruleset (pull request required, every `ci.yml` job a required check, no force pushes or deletion, admin bypass only through a pull request), labels, milestones, Actions permissions, Dependabot security updates, private vulnerability reporting. Change them there, not in the GitHub UI. For when to run `pulumi preview` and `pulumi up`, see "Before merging a chunk".

- [ ] **Never use a self-hosted runner for pull requests from forks.** A runner executes the pull request's code. If one is ever added (for example for hardware tests), restrict it to scheduled jobs on `main`.
- [ ] **Add `CODE_OF_CONDUCT.md`** once there is a contact method for conduct reports.

## Before merging a chunk

- [ ] `cargo xtask ci` passes and the GitHub run is green.
- [ ] The pull request title ends with the chunk ID, for example `(M1.1)`.
- [ ] The spec's status is `Implemented (PR #n, date)`; the plan's is `Historical: executed in PR #n (date)`, with its Deviations section filled in (or "None"); the log's is `Historical: PR #n (date)`; the pull request says `Closes #<chunk issue>`.
- [ ] `CHANGELOG.md` is updated.
- [ ] Merge only on the maintainer's explicit go-ahead (in the chunk loop the main session squash-merges, never with `--admin`).
- [ ] For any pull request (chunk or not) that changes `infra/github/`: `pulumi preview`, then `pulumi up` in `infra/github`, before the merge go-ahead from the branch when the chunk's spec rolls it out there (as M0.2 did), otherwise after the merge from `main`. Until then a new required check, label, milestone or setting does not exist in the repository.

## Issue tracking

The path from an idea to a chunk is in [workflow.md](workflow.md#from-issue-to-chunk). This section is the Project's configuration, which lives in GitHub, not in the repository. `/triage` checks the live Project against these names and stops if they differ.

- **Project:** "OS backlog", number 8, user-owned (`SpencerAWill`; `gh project list --owner SpencerAWill` finds it by title), public, linked to this repository.
- **Status:** Inbox, Ready, Scheduled, Done. **Priority:** Now, Next, Later. **Size:** S, M, L, XL. All single-select. **Priority** Now: blocks current work; Next: a candidate for the next few chunks; Later: worth doing, not soon. **Size** S: under about 100 changed lines or a short note; M: one ordinary chunk; L: a large chunk; XL: more than one chunk, so split it first. **Order** (a number field, M0.2) orders scheduled chunks and chores: spaced by 10 so an item can be inserted, ties broken by chunk ID (compared numerically: M1.7 before M1.10) and then issue number; Priority ranks unscheduled work only.
- **Labels** are declared in `infra/github/index.ts`. Every issue carries exactly one type label (`bug`, `enhancement`, `infrastructure`, `documentation`, `research`); `performance` and `needs-measurement` are additive; `area:*` is for code and tools. `research` issues are scheduled work like any other, not an icebox; an idea parked for later stays in the Inbox with Priority Later.
- **Milestones** are GitHub milestones `M0` to `M5`, declared in `infra/github/index.ts` (goal in the description; goals and exit criteria in [roadmap.md](roadmap.md#milestones)). The current milestone is the open one with the lowest number. An issue gets a milestone when it is scheduled, or when it belongs to the milestone's work without being scheduled yet.
- **Scheduling a chunk** (the maintainer, or an agent the maintainer asks): the chunk issue is authored by the maintainer's account. To schedule a chunk issue: unless its title already has its ID, rename it `M<N>.<n>: <title>` with the next unused chunk ID (one above the highest in the titles of issues authored by the maintainer's account, open or closed, in `docs/chunks/` folder names and in frozen roadmap rows; other authors' titles do not count, since anyone can open an issue), `gh issue edit <chunk issue> --milestone M<N>`, set Status Scheduled and Order on its Project item (`gh project item-edit`; a new issue gets its item from `gh project item-add 8 --owner SpencerAWill --url <issue url> --format json --jq .id`, since auto-add is asynchronous, and its Status is read back afterwards (`gh project item-list 8 --owner SpencerAWill --limit 1000 --format json`) and set again if the "Item added to project" workflow reset it to Inbox; the project id from `gh project view 8 --owner SpencerAWill --format json --jq .id`, the field and option ids from `gh project field-list 8 --owner SpencerAWill --format json`), and `gh issue edit <chunk issue> --add-blocked-by <other issue>` for each issue that must land first.
- **Another author's issue** `<issue>` is never retitled: (1) work out the next unused chunk ID `M<N>.<n>` (the rule above) and open a new chunk issue `<chunk issue>` titled with it, linking `<issue>`; (2) schedule the chunk issue as above, keeping that ID; (3) move `<issue>`'s links to the chunk issue, so nothing keyed on `<issue>` being open changes: for each issue in `gh api --method GET repos/SpencerAWill/rust-os-experiment/issues/<issue>/dependencies/blocking` and `.../blocked_by`, `gh issue edit <chunk issue> --add-blocking <that issue>` or `--add-blocked-by <that issue>`, and if `gh api --method GET repos/SpencerAWill/rust-os-experiment/issues/<issue>/parent` names a parent (a 404 means none), `gh issue edit <chunk issue> --parent <parent>`; (4) comment on `<issue>` `Scheduled as M<N>.<n> in #<chunk issue>` and close it with `gh issue close <issue> --duplicate-of <chunk issue>`.
- **Scheduling a chore:** as a chunk, without the chunk ID; Size S build work only (a `research` issue is always a chunk). A change to a scheduled item's Order or blockers comes with a one-line comment on the issue saying why. `/chunk next` skips a chunk in flight (its branch pushed to origin, or an open pull request from this repository closing it); the full selection is in [the chunk skill](../.claude/skills/chunk/SKILL.md#next).
- **Tracking issues:** one per multi-chunk effort, with sub-issues. Milestones replace the per-milestone tracking issue (#2 closed with M0.2).

### Built-in workflows and views (web UI only)

`gh` cannot set these. In the Project, **⋯ → Workflows**:

1. **Auto-add to project:** on, repository `rust-os-experiment`, filter `is:issue` (no label exclusion).
2. **Item added to project:** on, set Status to **Inbox**.
3. **Item reopened:** on, set Status to **Inbox**.
4. **Item closed:** on, set Status to **Done**.
5. Turn off any other default workflow that sets a Status this Project does not have.

Views (**+ New view**): **Next Up**, a table filtered to `status:Scheduled`, sorted by Order ascending and grouped by Milestone: the queue `/chunk next` reads, in the browser. Other views (an Inbox for triage, a board) may be added as needed; none is required.

## When a milestone completes

- [ ] Tag the commit that completed it: `git tag -a m<N> -m "M<N>: <goal>"`, then `git push origin m<N>`.
- [ ] Close the GitHub milestone (look its API number up by title, since GitHub numbers milestones in creation order: `gh api --method GET 'repos/SpencerAWill/rust-os-experiment/milestones?per_page=100' --jq '.[] | select(.title=="M<N>") | .number'`, then `gh api --method PATCH repos/SpencerAWill/rust-os-experiment/milestones/<that number> -f state=closed`; Pulumi ignores its state, and its entry in `index.ts` stays), move its open chores to the next milestone, and break the next one into chunk issues (scheduled as above) from the rough list in [roadmap.md](roadmap.md#later-milestones).
- [ ] Run `/lessons-refresh` over `docs/lessons/`.
- [ ] If a new Claude model shipped since the last milestone, re-check the model of each role in `.claude/agents/` against one chunk's results.
