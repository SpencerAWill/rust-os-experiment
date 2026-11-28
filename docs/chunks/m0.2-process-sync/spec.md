# M0.2: Process sync with rust-engine-experiment

- **Status:** Draft (2026-11-27)
- **Kind:** build · **Milestone:** M0 (Repository foundations) · **Depends on:** M0.1 · **Decides:** no ADR (process and documentation; the GitHub writes are configuration, §4.9)
- **Issue:** the chunk issue `M0.2: Process sync with rust-engine-experiment`, created in the rollout's first step (§4.9) so the pull request can close it; tracking issue #2 until the rollout closes it
- **Estimated size:** about 500 changed lines of Markdown, about 30 lines in `infra/github/index.ts`

Requirement keywords (MUST, MUST NOT, SHOULD, MAY) follow [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) as clarified by [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174): they are normative only in uppercase.

## 1. Summary

This repository's process (workflow, skills, agents, lessons) was ported from `rust-engine-experiment` at its commit 277e9b2 (2026-11-17). Since then the engine has moved planning state to GitHub (its M3.20), added a pause after the plan, and tightened the builder, simplify, re-verify and grill rules. This chunk brings those changes over before more M0 work, keeping this repository's deliberate differences. At the end, `/chunk next` picks the next chunk from GitHub, and the roadmap file holds only milestone goals and frozen history. M0.3 (host safety, #10) and M0.4 (spike runner, #12) are scheduled chunk issues in milestone M0, and M1's six chunk issues are in milestone M1 with their Orders and blockers.

## 2. Goals and non-goals

**Goals**

- One answer to "what is next", read from GitHub: the scheduled chunk issue of the current milestone that is not in flight, has no open blocker and has the lowest Order (§4.4).
- Every chunk is exactly one chunk issue, and small scheduled work without a chunk ID is a chore (§4.2, §4.3).
- Milestones M0 to M5 are GitHub milestones declared in Pulumi (§4.5).
- The engine's process changes since 277e9b2 that are not engine-specific are in this repository (§4.7, §4.8).

**Non-goals**

- Nothing engine-specific: the xtask benchmark and spike package lists, the rule against running `cargo` during a full `cargo xtask ci` (it exists because the engine's CI runs `cargo hack --no-dev-deps`, which rewrites manifests; this repository's CI does not *(my call)*), and the `ecs` and `perf` lessons.
- No change to this repository's deliberate differences: research chunks (`Kind: research`, the `researcher` and `review-sources` roles), the OS review perspectives, and `research` issues as scheduled Project items, not an icebox (maintainer, 2026-11-27).
- No change to chunk IDs, the seven stages' content beyond §4.7, specs already accepted, ADRs, or historical documents (the M0.0 and M0.1 folders).
- No "In progress" Status: a pushed branch shows it (as in the engine).
- No CI check of planning state.

## 3. Guide-level explanation

**Scheduling.** The maintainer, or an agent the maintainer asks, makes a Ready issue a chunk issue: `M1.1: <title>`, milestone `M1`, Status Scheduled, Order 10, and "blocked by" links to issues that must land first. A chore is the same without a chunk ID, and is Size S only.

**What is next.** `/chunk next` reads the Project and the issues with `gh`, names the next chunk, and lists the scheduled chores and the chunks in flight. Stage 1 pushes the new branch at once, so a second session sees the chunk as in flight. In the browser, the Project's Next Up view shows the same queue.

**After the plan.** The main session commits the plan and stops with a one-line reminder: `/rename <id> spec and plan`, `/clear`, `/chunk resume`. Saying "continue" goes on in the same session.

**Done.** The pull request says `Closes #<chunk issue>`. Merging closes the issue, the Project shows Done and the milestone's progress moves. No roadmap row changes.

## 4. Reference-level design

The engine's [M3.20 spec](https://github.com/SpencerAWill/rust-engine-experiment/blob/main/docs/chunks/m3.20-planning-on-github/spec.md) (§4.1 to §4.4) is the design this chunk follows. What follows is normative here, adapted to this repository: Project 8 "OS backlog", repository `SpencerAWill/rust-os-experiment`, milestones M0 to M5.

### 4.1 Sources of truth

| What | Where |
|---|---|
| Milestone goals and exit criteria | `docs/roadmap.md` (milestone table); each GitHub milestone's description is its goal in one line and a link to `docs/roadmap.md#milestones` |
| Milestone membership and progress | GitHub milestone |
| Unscheduled work and its ranking | Project: Status Inbox or Ready, Priority, Size |
| Scheduled work, its order and dependencies | Project: Status Scheduled and Order; "blocked by" links between issues |
| What a chunk means | Its spec, once it exists (the issue is the request and is not edited to match) |
| State of a chunk in flight | Branch on origin, plan checkboxes, `log.md` |
| Done | The issue is closed (by the pull request) |
| History of chunks up to M0.2 | `docs/roadmap.md`'s M0 table, frozen |

Priority ranks unscheduled work; Order ranks scheduled work. Neither is read for the other.

### 4.2 Chunk issues

- Every chunk MUST have exactly one chunk issue, titled `M<N>.<n>: <title>` and authored by the maintainer's account (agents file through it). The spec header's `**Issue:**` line names it and is required. The pull request says `Closes #<chunk issue>`.
- **The next unused chunk ID** of a milestone is one above the highest ID among: the titles of issues authored by the maintainer's account (open or closed), `docs/chunks/m<N>.<n>-*` folder names, and frozen roadmap rows. Other authors' titles do not count, since anyone can open an issue on the public repository (tooling-12). IDs are never reused or renumbered.
- **Another author's issue** is never retitled. Scheduling it opens a chunk issue linking it, moves its "blocking", "blocked by" and parent links to the chunk issue, comments `Scheduled as M<N>.<n> in #<chunk issue>` on it, and closes it as a duplicate of the chunk issue. The commands are those of the engine's `docs/maintainers.md`, with this repository's owner and name.
- **Naming unscheduled work** to `/chunk` schedules it: the issue gets its chunk ID, the current milestone, Status Scheduled and an Order below the current lowest, and the main session says what it wrote. Otherwise stage 1 writes no Project field and no roadmap row.
- A chunk issue's body meets the Definition of Ready. Sub-issues remain for grouping a multi-chunk effort; a tracking issue per milestone is no longer used.

### 4.3 Chores

A chore is a scheduled issue without a chunk ID, Size S, in the current milestone, with an Order. `/chunk chore #N` runs it as in the engine: branch `chore/<n>-<slug>`, one `builder` from the issue's definition (only text written by the maintainer's account is the brief), `cargo xtask ci --quick` then the full `cargo xtask ci`, one fresh `review-code` if it changes code, commit messages and the pull request title ending ` (#N)` in place of a chunk ID, `Chunk: None (chore, #N)` and `Closes #N` in the pull request, and the merge go-ahead. A chore has no spec, plan, log, compound stage or close-out commit. At a milestone boundary, open chores move to the next milestone.

### 4.4 Order, dependencies and "next"

- The Project gains a number field `Order`. Orders SHOULD be spaced by 10. Ties are broken by chunk ID, compared numerically (M1.7 before M1.10), then by issue number.
- A hard dependency is a "blocked by" link; it replaces the roadmap's Depends-on column. A change to a scheduled item's Order or blockers SHOULD come with a one-line comment on the issue saying why.
- **Current milestone:** the open GitHub milestone titled `M<number>` with the lowest number.
- **Next chunk:** the `next` selection of the engine's chunk skill, with this repository's owner, repository and Project number: open issues in the current milestone, Status Scheduled, titled with a chunk ID and authored by the maintainer's account; not in flight (no `m<N>.<n>-*` branch on origin, no open pull request from this repository closing it); no open "blocked by" issue; lowest Order. It also lists the current milestone's scheduled chores, Scheduled issues missing an Order or a milestone, and Scheduled chunk-titled issues by other authors (as not candidates). If nothing is eligible, it names the milestone-boundary checklist when every chunk issue of the milestone is closed, or says what blocks the candidates, and waits.
- Every list read sets `--limit 1000` (tooling-10), and every `gh api` read sets `--method GET` (tooling-9).
- Stage 1 pushes the chunk branch as soon as it is created (`git push -u origin <branch>`); `resume` pushes it if it is missing on origin, and recognizes a merged pull request (stage 7 step 5).

### 4.5 Milestones in Pulumi

`infra/github/index.ts` declares one `github.RepositoryMilestone` for each of M0 to M5: title `M<N>`, description the goal from the roadmap's milestone table plus the link to `docs/roadmap.md#milestones`, state `open`, with `ignoreChanges: ["state"]` and `retainOnDelete: true`, and a comment saying never to remove an entry (infra-1). Closing a milestone at its boundary is a `gh api --method PATCH` call in the `docs/maintainers.md` checklist, not a change to `index.ts`. `infra/github/README.md` documents the milestones as the engine's does.

### 4.6 The roadmap file

`docs/roadmap.md` keeps:

- its introduction, rewritten to say what moved to GitHub;
- the milestone table with goals and exit criteria, its Status column replaced by a line pointing to the GitHub milestones;
- the M0 table, frozen as history up to M0.2, whose row becomes `Done (PR #n)` at close-out;
- the "Later milestones" table, whose rough lists become chunk issues when a milestone becomes current.

The M1 table is removed: its six rows are chunk issues #3 to #8. Before removal, the main session checks that each row's title and dependencies are in its issue (§4.9 item 3). No document links to an anchor in `roadmap.md` today; `cargo xtask check-docs` keeps every file link resolving.

### 4.7 Documents and skills (planning on GitHub)

Each file changes where it names the roadmap as the place of scheduled work, order or status, following the engine's text at origin/main e9e6156 for the same file and adapted as above:

- `docs/workflow.md`: "The loop" (three waits, picking the next chunk, the pause), "From issue to chunk" (vocabulary with chunk issue and chore, the path, sources of truth, the relationship table, splitting).
- `docs/maintainers.md`: "Before merging a chunk" (`Closes #<chunk issue>`, the `infra/github` rollout timing), "Issue tracking" (Order, the Next Up view as the one required view, milestones, scheduling a chunk and a chore, another author's issue, tracking issues), "When a milestone completes" (close the GitHub milestone, move open chores, break the next milestone into chunk issues; no tracking issue to close).
- `.claude/skills/chunk/SKILL.md`: three waits; `next` from GitHub (§4.4); a `chore #N` section (§4.3); `resume` with the branch push and the merged-PR case.
- Chunk stages 1, 2, 3 and 7: stage 1 pushes the branch and edits no roadmap row or Project field; stages 2 and 3 set no roadmap status; stage 3 ends with the pause (§4.8); stage 7 closes through `Closes #N`, and a chunk that changes `infra/github/` rolls it out during the merge wait when its spec says so.
- `.claude/skills/triage/SKILL.md`: the field check includes `Order`, read as a number from item values (tooling-11); the duplicate search covers chunk issues; triage never sets Status Scheduled, an Order or a chunk ID.
- `AGENTS.md` (the three waits, chunk issues, `/chunk chore #N`, the chore commit-message exception; under 80 lines), `CONTRIBUTING.md`, `README.md`, `docs/README.md`, `docs/chunks/README.md`, `.claude/README.md`, `docs/research/open-questions.md`, `.github/PULL_REQUEST_TEMPLATE.md` (chores, `Closes #N`), the issue forms' "Depends on" hints, and `docs/chunks/_template/spec.md` (the required `**Issue:**` line).

### 4.8 The engine's later process changes

From the engine's commits after M3.20, each adapted to this repository's files:

1. **Plan pause** (engine #136, #138): stage 3 step 6 and the third wait in `SKILL.md`, `AGENTS.md` and `docs/workflow.md`.
2. **Builder** (`.claude/agents/builder.md`): never remove or relax a CI-enforced attribute or setting (`#![forbid(unsafe_code)]`, a `deny` or `forbid` lint, `deny.toml`), not even temporarily; put the first lines of the failing test run in the report; the shell rules (no command that waits on stdin; the Write and Edit tools, not `sed -i`; long commands redirected to a scratchpad file; no shell left running).
3. **Simplify** (stage 5 step 1): read every changed source file in full, and send back a report that skipped any; re-run one red check per helper if tests moved into a shared helper.
4. **Re-verify** (stage 5, a new step before the log step): the full `cargo xtask ci` on the final code commit after the last fix, and the chunk's measurements re-run on it or the pull request says which commit each ran on.
5. **Grill API check** (`.claude/skills/grill/SKILL.md`): if the document declares Rust API, compile it as a stub crate in the scratchpad on the pinned toolchain and check it against the Rust API Guidelines; each break is a question.
6. **Plan template**: the Deviations comment says one line each, with the story in `log.md`.
7. **Lessons**, rewritten for this repository where the engine's text names engine code: engine docs-4, tooling-11, tooling-12 and infra-1 keep their IDs (`docs/lessons/infra.md` is new, `applies_to: infra/github/**`); engine specs-6 and specs-7 become specs-5 and specs-6 here, the next free IDs, since this repository has no specs-5 *(my call)*. docs-1 gains the engine's general addition (grep a moved rule's key phrase across `docs/` and `.claude/`), and specs-4's rule covers measured-code changes as well as library changes. Each ported lesson's *From* names the engine chunk (for example "engine M3.20"). `docs/lessons/README.md` lists the new file and topics.

### 4.9 Rollout (GitHub writes, authorized by this spec)

These writes change GitHub, not the repository. Each is read back and recorded in `log.md`.

**Before the pull request opens:**

1. The chunk issue `M0.2: Process sync with rust-engine-experiment`, with the Definition of Ready headings, in the Project with Status Scheduled. Its number fills the spec header's `**Issue:**` line.
2. The Project's `Order` number field (`gh project field-create 8 --owner SpencerAWill --name Order --data-type NUMBER`), and Order 10 on the M0.2 chunk issue.

**During the merge wait, after the pull request and its close-out commit are pushed:** the maintainer runs `pulumi preview` then `pulumi up` in `infra/github` from the branch (milestones exist from then on). Then the main session:

3. Gives #3 to #8 milestone M1 and Orders 10 to 60 in table order (M1.1 to M1.6), and these "blocked by" links from the M1 table's Depends-on column: #5 by #4; #6 by #3; #7 by #3 and #6; #8 by #3 and #4. It first checks that each issue's title and body carry its row's title and dependencies, and adds anything missing to the issue body.
4. Retitles #10 `M0.3: Host-safety guardrails for agents and later QEMU commands` and #12 `M0.4: A runner for spikes (cargo xtask spike)`; gives both milestone M0, Status Scheduled and Orders 20 and 30; and links #12 blocked by #10 (#12's body says #10 lands first).
5. Gives the M0.2 chunk issue milestone M0.
6. Closes #2 as completed, with a comment that milestones replace it.
7. Asks the maintainer to create the Next Up view in the browser (views are not settable through `gh`), then confirms it is listed.
8. Runs `/chunk next`'s selection (§4.4) by hand, as a dry run, and records the result.

**If the pull request is abandoned**, every write is undone: the M0.2 issue closed as not planned; milestones removed from the issues, then from `index.ts` with a `pulumi up` (which, under `retainOnDelete`, only drops them from Pulumi's state), then deleted with `gh api --method DELETE repos/SpencerAWill/rust-os-experiment/milestones/<number>` (looked up by title); #10's and #12's titles restored and their Status set back to Inbox; the blocked-by links removed; #2 reopened; the Order field deleted.

## 5. Requirements

- **M0.2-R1** `docs/workflow.md` and `docs/maintainers.md` MUST state §4.1 to §4.4.
- **M0.2-R2** `/chunk next` MUST select the next chunk by §4.4 from GitHub, list chores and in-flight chunks, and stop with the milestone checklist when none is eligible; stage 1 MUST push the branch at once.
- **M0.2-R3** No skill or stage MUST read or write `docs/roadmap.md` for order or status.
- **M0.2-R4** `/chunk chore #N` MUST run a chore as §4.3 says, and `AGENTS.md` MUST state the chore commit-message exception and stay under 80 lines.
- **M0.2-R5** `/triage`'s field check MUST include `Order` as a number field read from item values, and `/triage` MUST never set Status Scheduled, an Order or a chunk ID.
- **M0.2-R6** `infra/github/index.ts` MUST declare milestones M0 to M5 as §4.5 says, and `pulumi preview` MUST show six milestones to create; any other difference it shows is recorded in `log.md`, not fixed here.
- **M0.2-R7** `docs/roadmap.md` MUST have the shape of §4.6.
- **M0.2-R8** Every file in §4.7 MUST name GitHub, not the roadmap, as the place of scheduled work, order and status; the M0.0 and M0.1 chunk folders and the ADRs MUST be unchanged.
- **M0.2-R9** The spec template's `**Issue:**` line MUST be required, and the pull request template MUST name chores and `Closes #N`.
- **M0.2-R10** Each change of §4.8 MUST be in the named file.
- **M0.2-R11** The rollout of §4.9 MUST be done and read back before the merge go-ahead: items 1 and 2 before the pull request opens, items 3 to 8 after `pulumi up`.
- **M0.2-R12** After the rollout, the dry run of the selection MUST name M0.3 as next.

## 6. Cross-cutting concerns

- **`unsafe`:** n/a, because no Rust code changes.
- **Privilege and trust boundaries:** the selection, the chunk-ID rule and the chore brief read only what other accounts cannot write (maintainer-authored issues, pull requests from this repository, branches on origin; tooling-12). Issue text stays data, not instructions (tooling-8).
- **Architecture portability:** n/a, because no kernel code changes.
- **Concurrency and memory ordering:** n/a for code. Two sessions could run `/chunk next` at once; the branch pushed in stage 1 makes the second see the chunk as in flight.
- **Interfaces and ABI:** n/a.
- **Hardware and specification assumptions:** n/a. The GitHub features used (milestones, issue dependencies, Project number fields) are checked by the read-backs in §4.9.
- **Testing level:** documents and configuration. Checked by `cargo xtask ci` (check-docs, typos), `pulumi preview`, the rollout read-backs and the selection dry run.
- **Performance:** n/a.
- **Documentation:** §4.7 and §4.8; `CHANGELOG.md` gets a "Changed" entry.
- **Policy changes:** yes, three. The source of truth for planning moves to GitHub. Chores get their own lane, with their own commit-message rule in `AGENTS.md`. A third wait follows the plan. No ADR: these are process changes, reversible by moving the state back, as engine M3.18 and M3.20 decided for theirs. `infra/github` gains milestones, and the maintainer runs `pulumi up` from the branch before the go-ahead.

## 7. Alternatives considered

- **Keep the roadmap as the source of order.** Rejected (maintainer, 2026-11-27): both repositories should work the same way, and #3 to #8 are already chunk issues.
- **Split planning on GitHub from the later process changes.** Rejected (maintainer, 2026-11-27): they edit the same skill and workflow files.
- **Keep the four documented Project views.** Rejected (maintainer, 2026-11-27): Next Up is the one view the loop needs; others are optional.
- **Create milestones only from M1 on.** Rejected *(my call)*: M0 is the current milestone and holds M0.2 to M0.4, so the selection needs it.
- **Keep the engine's lesson IDs specs-6 and specs-7.** Rejected *(my call)*: IDs here are numbered per repository, and specs-5 is free.

## 8. Verification

| Requirement | Verified by |
|---|---|
| R1 | review-docs and review-spec against §4.1 to §4.4 |
| R2 | The `next` section and stage 1 read against §4.4; R12's dry run |
| R3 | `git grep -n -i roadmap .claude/` lists no read or write of order or status (each remaining mention named in `log.md`) |
| R4 | review-spec of the `chore` section against §4.3; `AGENTS.md`'s commit rule; `cargo xtask check-docs` (the 80-line limit) |
| R5 | The triage skill's text; a dry run of its field check after the Order field exists, recorded in `log.md` |
| R6 | `pulumi preview` output in `log.md`, run by the maintainer |
| R7 | review-spec against §4.6; `cargo xtask check-docs` |
| R8 | `git grep -n -i roadmap` over the §4.7 files, each remaining mention checked and named in `log.md`; `git diff --stat main...HEAD` shows no file under `docs/chunks/m0.0-foundations/`, `docs/chunks/m0.1-dev-environment/` or `docs/adr/` |
| R9 | The two templates |
| R10 | review-spec of each item of §4.8 against its file |
| R11 | `gh issue view`, `gh project item-list --limit 1000` and `gh api --method GET` read-backs in `log.md`, per item |
| R12 | The selection run by hand with `gh` after the rollout, its result in `log.md` |

## 9. Done when

- [ ] R1 to R5 and R7 to R10 have their evidence (in `log.md`, including the R3 and R8 grep lists), and `cargo xtask ci` passes on the final commit.
- [ ] The maintainer has run `pulumi preview` and `pulumi up` from the branch (R6 in `log.md`).
- [ ] R11 and R12 have their evidence in `log.md`, pushed before the merge go-ahead.

## 10. Open questions

None.

## 11. Prior art and sources

The engine repository's [M3.20 spec](https://github.com/SpencerAWill/rust-engine-experiment/blob/main/docs/chunks/m3.20-planning-on-github/spec.md), and its process files at origin/main e9e6156 (compared with 277e9b2, the port). GitHub's documentation for milestones, issue dependencies and Project fields, and the Pulumi GitHub provider's `RepositoryMilestone`, as the engine used them.
