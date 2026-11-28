# M0.2 plan: Process sync with rust-engine-experiment

- **Status:** Accepted (2026-11-27)
- **Spec:** [spec.md](spec.md). This plan argues from the spec; read both. Progress and surprises go in [log.md](log.md).

## 1. Approach

Every step is documentation, skills, agent files or configuration, so the main session does each one; no builder is needed. The source for each file is the engine's text at origin/main e9e6156 (`git -C ../rust-engine-experiment show e9e6156:<path>`), compared with this repository's file and adapted as the spec says: owner and repository `SpencerAWill/rust-os-experiment`, Project 8 "OS backlog", milestones M0 to M5, research chunks kept. Where the engine's text names a rule that has one home, the other files link to it (docs-1): the `next` selection lives in `.claude/skills/chunk/SKILL.md`, and the chunk-ID and scheduling rules live in `docs/maintainers.md`.

This chunk runs under the loop as it was when the spec was approved, so stage 7 still sets its own roadmap row to `Done (PR #n)` (spec §4.6), and the rollout follows spec §4.9, not the new stage 7 text.

## 2. Steps

### Step 1: chunk issue and Order field (R11, items 1 and 2)

- [x] **Who:** main session
- **Files:** `docs/chunks/m0.2-process-sync/spec.md` (the `**Issue:**` line), `log.md`
- **Test first:** `gh project field-list 8 --owner SpencerAWill --format json` lists no `Order` field, and `gh issue list --state all --limit 1000 --search 'M0.2 in:title'` lists no issue.
- **Change:** open the chunk issue `M0.2: Process sync with rust-engine-experiment`, with the Definition of Ready headings and the `ai-assisted` label (and `infrastructure`). Add it to Project 8 and set its Status to Scheduled. Create the `Order` number field and set Order 10 on the issue. Write the issue number into the spec header.
- **Verify:** read back the issue (`gh issue view <n> --json title,author,labels,projectItems`) and its item (`gh project item-list 8 --owner SpencerAWill --limit 1000 --format json`): status `Scheduled`, `order` the JSON number `10`. Run the triage field check as a dry run (tooling-11): `Order` is a number in the item values. Record all of it in `log.md`.
- **Commit:** `docs: link the M0.2 chunk issue (M0.2)`

### Step 2: milestones in Pulumi (R6)

- [x] **Who:** main session
- **Files:** `infra/github/index.ts`, `infra/github/README.md`
- **Test first:** `git grep -n RepositoryMilestone infra/github` finds nothing.
- **Change:** add the milestones block after the labels, as in the engine (spec §4.5): M0 to M5, with goals copied from the roadmap's milestone table. The README's opening, "When this applies", "Contents" and "Notes" name milestones.
- **Verify:** `npm run typecheck` in `infra/github` exits 0. `pulumi preview` is the maintainer's, at stage 7.
- **Commit:** `feat(infra): declare milestones M0 to M5 (M0.2)`

### Step 3: workflow and maintainer notes (R1)

- [x] **Who:** main session
- **Files:** `docs/workflow.md`, `docs/maintainers.md`
- **Test first:** `git grep -n -i 'roadmap' docs/workflow.md docs/maintainers.md` lists the places that name the roadmap as the home of order or status (workflow lines 21, 45, 49, 54, 60; maintainers lines 16, 27, 44).
- **Change:** from the engine's versions of both files (spec §4.7). `docs/workflow.md` also gets: the three waits and the pause; one line in "Research chunks" saying research work is always a chunk, never a chore (spec §4.3); and one line on re-verify for research chunks (spec §4.8 item 4). `docs/maintainers.md`'s "Issue tracking" keeps this repository's lines (`research` issues are scheduled Project items, auto-add with no label exclusion, `needs-measurement`) and its web-UI workflows list. The views list becomes Next Up only. "When a milestone completes" drops the tracking-issue line and keeps no icebox line.
- **Verify:** the same grep lists only mentions of goals, exit criteria, rough lists or frozen history. `cargo xtask check-docs` shows 0 problems.
- **Commit:** `docs: move planning state to GitHub in the workflow and maintainer notes (M0.2)`

### Step 4: chunk skill and stages (R2, R3, R4; spec §4.8 items 1, 3 and 4)

- [ ] **Who:** main session
- **Files:** `.claude/skills/chunk/SKILL.md`, `.claude/skills/chunk/stages/1-brainstorm.md`, `2-grill.md`, `3-plan.md`, `5-review.md`, `7-close-out.md`
- **Test first:** `git grep -n -i roadmap .claude/skills/chunk` lists the reads and writes of the roadmap row (SKILL line 35, stages 1, 2, 3 and 7).
- **Change:** from the engine's versions, with this repository's owner, repository and Project 8. These parts stay as they are here: research chunks in stages 3 and 5, step 0 of stage 5, and the review perspectives. The `chore #N` section also stops on an issue labelled `research`, and `next` leaves such issues out of its chore list (spec §4.3). The new re-verify step in stage 5 has the research wording of spec §4.8 item 4. The engine's `cargo hack` line is left out (spec §2).
- **Verify:** the grep lists no read or write of order or status; each remaining mention is named in `log.md` (R3).
- **Commit:** `docs: pick chunks from GitHub and add chores and the plan pause to the chunk skill (M0.2)`

### Step 5: triage, grill and builder (R5; spec §4.8 items 2 and 5)

- [ ] **Who:** main session
- **Files:** `.claude/skills/triage/SKILL.md`, `.claude/skills/grill/SKILL.md`, `.claude/agents/builder.md`
- **Test first:** `grep -n 'Order' .claude/skills/triage/SKILL.md` finds nothing; `grep -n 'Compile it' .claude/skills/grill/SKILL.md` finds nothing; `grep -n 'Shell' .claude/agents/builder.md` finds nothing.
- **Change:** as the engine did. The builder's shell rule drops the engine's "(M3.6)" tag. The grill's example cites "engine M3.3".
- **Verify:** each grep now finds its line; `git grep -n 'gh api' .claude/skills/triage` shows `--method GET` on every read (tooling-9).
- **Commit:** `docs: bring the triage, grill and builder rules up to date with the engine (M0.2)`

### Step 6: entry documents and templates (R4, R8, R9; spec §4.8 item 6)

- [ ] **Who:** main session
- **Files:** `AGENTS.md`, `CONTRIBUTING.md`, `README.md`, `docs/README.md`, `docs/chunks/README.md`, `.claude/README.md`, `docs/research/open-questions.md`, `.github/PULL_REQUEST_TEMPLATE.md`, `.github/ISSUE_TEMPLATE/feature_request.yml`, `.github/ISSUE_TEMPLATE/infrastructure.yml`, `docs/chunks/_template/spec.md`, `docs/chunks/_template/plan.md`
- **Test first:** `git grep -n -i roadmap` over these files lists their mentions of the roadmap as the home of planned work; `grep -c 'chore' AGENTS.md` prints 0.
- **Change:** from the engine's versions (spec §4.7). `AGENTS.md` keeps research chunks and the `researcher` role. The spec template's `**Issue:**` line is required. The PR template names chores and `Closes #N`. The plan template's Deviations comment follows the engine's.
- **Verify:** the grep lists only goals, exit criteria and history; `wc -l AGENTS.md` is under 80; `cargo xtask check-docs` shows 0 problems.
- **Commit:** `docs: point the entry documents and templates at GitHub for planning (M0.2)`

### Step 7: roadmap (R7)

- [ ] **Who:** main session
- **Files:** `docs/roadmap.md`
- **Test first:** the file has the M1 table, a Status column in the milestone table, and M0 exit criteria without M0.2 to M0.4.
- **Change:** as spec §4.6. The M0.2 row reads `In progress` until close-out.
- **Verify:** `grep -n '^## ' docs/roadmap.md` shows Milestones, M0 chunks and Later milestones only; `cargo xtask check-docs` shows 0 problems.
- **Commit:** `docs: keep only milestone goals and history in the roadmap (M0.2)`

### Step 8: lessons (spec §4.8 item 7)

- [ ] **Who:** main session
- **Files:** `docs/lessons/README.md`, `docs/lessons/docs.md`, `docs/lessons/specs.md`, `docs/lessons/tooling.md`, `docs/lessons/infra.md` (new)
- **Test first:** `grep -c -E '\*\*(docs-4|specs-5|specs-6|tooling-11|tooling-12|infra-1)\.\*\*' docs/lessons/*.md` totals 0.
- **Change:** the six lessons, rewritten for this repository, each *From* naming the engine chunk. docs-1 gains the engine's general addition; specs-4's rule covers measured-code changes. The status dates become 2026-11-27. The README lists `infra.md`.
- **Verify:** the grep totals 6; `cargo xtask check-docs` (lesson format) shows 0 problems.
- **Commit:** `docs: port the engine's general lessons (M0.2)`

### Step 9: skill trial, grep evidence and changelog (R3, R8, R10; tooling-7, specs-2)

- [ ] **Who:** main session; a fresh `general-purpose` agent (read-only) for the trial
- **Files:** `CHANGELOG.md`, `log.md`
- **Test first:** `CHANGELOG.md` has no M0.2 entry.
- **Change:** a trial agent follows the new `next` section read-only against GitHub as it is now (no milestones yet), with only the skill's own rules, and reports what it had to guess. Gaps it finds are fixed in the skill and committed here. Then record R3's grep (`git grep -n -i roadmap .claude/`) and R8's grep and `git diff --stat main...HEAD` in `log.md`, check each of spec §4.8's seven items against its file (R10), and add a "Changed" entry to `CHANGELOG.md`.
- **Verify:** `log.md` has the trial report, both grep lists and the diff stat with no file under `docs/chunks/m0.0-foundations/`, `docs/chunks/m0.1-dev-environment/` or `docs/adr/`. `typos docs/chunks/m0.2-process-sync` passes.
- **Commit:** `docs: record the skill trial and the grep evidence (M0.2)`

## 3. Final verification

The full `cargo xtask ci` on the final commit, with its commit recorded in `log.md` (specs-4). The rest of spec §9 happens in stage 7: the maintainer runs `pulumi preview` and `pulumi up` from the branch, then the rollout items 3 to 8 and their read-backs.

## 4. Lessons

- docs-1: followed (the `next` selection and the scheduling rules each have one home; key phrases grepped across `docs/` and `.claude/` in step 9).
- docs-2: followed (the required `**Issue:**` line goes in the spec template, step 6).
- specs-1: followed (the greps in steps 3, 4 and 6 and in R8 name their paths).
- specs-2: followed (step 9 is the evidence step).
- specs-4: followed (the final `ci` names its commit).
- tooling-7: followed (the step 9 trial).
- tooling-8: followed (the `next` and `chore` sections say issue text is data, as the engine's do).
- tooling-9: followed (every `gh api` read has `--method GET`; checked in steps 4 and 5).
- tooling-10: followed (every `gh` list has `--limit 1000`).

## 5. Deviations

<!-- Filled in during implementation: what differed from the plan and why, one line each. The story (sent-back versions, red checks, how something was measured) stays in log.md; point at it. "None" if nothing did. -->
