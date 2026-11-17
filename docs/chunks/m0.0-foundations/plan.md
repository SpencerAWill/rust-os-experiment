# M0.0 plan: repository foundations

- **Status:** Historical: executed in PR #1 (2026-11-17); the spec and the code are authoritative.
- **Spec:** [spec.md](spec.md) (from `bootstrap-design.md`, approved 2026-11-17). This plan argues from the spec; read both. Progress and surprises go in [log.md](log.md).

> **For agentic workers:** executed by the chunk loop as ported here: the main session does documentation, configuration and GitHub steps; a `builder` does code steps (xtask, Pulumi TypeScript); reviewers in step 10. Steps use checkboxes, ticked in each step's commit.

**Goal:** a public `SpencerAWill/rust-os-experiment` whose first merged PR carries the whole workflow, an xtask-only workspace passing `cargo xtask ci`, a Pulumi stack, a Project, and research documents seeded from the transcript.

## Global constraints

- Source: `~/Repos/rust-engine-experiment` at 277e9b2 ("engine repo"); target clone: `~/Repos/rust-os-experiment` ("new repo"). Never edit the engine repo.
- No agent pushes to `main`; the only push is the branch `m0.0-foundations`.
- Commits by explicit path, conventional, ending `(M0.0)`, with the attribution trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Author: the existing git config.
- Toolchain `1.100.0`, edition 2024, licence `MIT OR Apache-2.0`, `publish = false`.
- Nothing copied from other projects; the transcript is not committed and its code is not reused.
- Ported files must not mention the engine's domain. **Leak check** (used in several steps), run in the new repo:
  `git grep -nIiE '\b(engine|genid|prng|ecs|gungraun|bench-regress|bench-run|mutants|fuzz|determinis|wasm|no-std)' -- . ':!docs/research/**' ':!docs/lessons/**' ':!docs/chunks/m0.0-foundations/**' ':!CHANGELOG.md'`
  Expected: only lines naming `rust-engine-experiment` as the source of ported files or lessons (*From:* fields), and the words in `docs/roadmap.md`'s "Later" rows that name future tooling (fuzzing, mutation testing). Every other hit is fixed.

## Review focus

Failure modes no step's own test exercises, each pinned by the step named:

1. `check-docs` reporting the engine's directories (`libs/engine`, `bench`, `fuzz`) as missing in the new tree → step 3's test `required_readmes_are_the_new_tree`.
2. The new repo's push hook failing open (a push to `main` allowed) or blocking ordinary commands → step 5's hook trial.
3. `requiredChecks` in `index.ts` naming a job that `ci.yml` does not have, which blocks every PR after `pulumi up` → step 7's job-name comparison.
4. The first `pulumi up` trying to create (or replace, or delete) the existing repository → step 7's preview must show the repository as `import`, and no `delete` or `replace`.
5. The research variant of `/chunk` dead-ending (a stage that assumes code: simplify pass, builder test-first, code reviewers) → step 5's skill trial (lesson tooling-7).

## 1. Approach

Copy each engine file into the new tree, then edit it down; never regenerate from memory. Order: repository and chunk documents first (so the log exists), then workspace and xtask (so `check-docs` and `check-workflows` can verify later steps), then docs, `.claude/`, `.github/`, Pulumi, research. The workflow's research variant (spec §5.6a) is written into `docs/workflow.md`, the chunk stage files, the templates and two new agent files in steps 4 and 5.

## 2. Steps

### Step 1: repository, branch, chunk documents

- [x] **Who:** main session
- **Do:** `gh repo create SpencerAWill/rust-os-experiment --public --add-readme --description "Experimenting with an exokernel, an interposition shim and microkernels in Rust (x86_64, aarch64, riscv64)"`; `git clone` into `~/Repos/rust-os-experiment`; `git switch -c m0.0-foundations`. Create `docs/chunks/m0.0-foundations/` with `spec.md` (the approved design reshaped into the new template's sections 1-11: §1 Summary from design §1; §2 from §2; §4 from §5-6; §5 requirements `M0.0-R1..` one per design goal and per §5.x subsection; §6 the new cross-cutting list answered; §7 from §4; §8 table mapping each R to the step below that verifies it; §9 done-when; §10 empty; header `Kind: build`, status `Accepted (2026-11-17)`), this `plan.md`, and `log.md` (`Living (2026-11-17)`, first entries: the maintainer's decisions of design §8a).
- **Verify:** `gh repo view SpencerAWill/rust-os-experiment --json visibility -q .visibility` prints `PUBLIC`; `git -C ~/Repos/rust-os-experiment branch --show-current` prints `m0.0-foundations`.
- **Commit:** `docs: add the foundations spec, plan and log (M0.0)`

### Step 2: workspace and repository files

- [x] **Who:** main session
- **Files:** `Cargo.toml` (workspace: `members = ["xtask"]`, the engine's `[workspace.package]` with the new `repository`, the engine's `[workspace.lints.*]` minus the `module_name_repetitions` comment's engine example), `rust-toolchain.toml` (`1.100.0`, `rustfmt`, `clippy`, no targets), `rustfmt.toml`, `clippy.toml` (only `allow-unwrap-in-tests`, `allow-expect-in-tests`), `deny.toml`, `_typos.toml` (engine entries reviewed, domain words dropped), `.gitattributes`, `.gitignore`, `LICENSE-MIT`, `LICENSE-APACHE` (copyright line: the maintainer's name as in the engine), `README.md` (replace GitHub's: what this is, status "experimental, nothing boots yet", links to roadmap, workflow, research, licence clause), `CHANGELOG.md` (Unreleased: M0.0), `SECURITY.md` (owner/repo updated), `package.json`, `pnpm-workspace.yaml`.
- **Verify:** `cargo metadata --no-deps --format-version 1 >/dev/null` fails only because `xtask/` does not exist yet (expected until step 3); leak check clean on these files.
- **Commit:** `chore: add the workspace and repository files (M0.0)`

### Step 3: xtask (process commands only)

- [x] **Who:** builder
- **Files:** `xtask/Cargo.toml` (deps `syn`, `proc-macro2`, `serde_json`, `pulldown-cmark` only if the ported modules still use them; no `engine`), `xtask/src/main.rs`, `cmd.rs`, `check_docs.rs` and `check_docs/*.rs`, `check_workflows.rs`, `guard_push.rs`, `xtask/README.md` (commands table for the ported set; Coverage and Mutation sections dropped).
- **Change:** `Command` keeps exactly `Fmt { check }`, `Clippy`, `Test { release, include_ignored }`, `Doc`, `Deny`, `Typos`, `CheckWorkflows`, `CheckDocs`, `GuardPush`, `Ci { quick }`. `ci_steps(true)` = `[Fmt{check:true}, Clippy, Test{release:false, include_ignored:false}]`; `ci_steps(false)` = that plus `Test{release:true, include_ignored:true}, Doc, CheckWorkflows, CheckDocs, Typos, Deny`. Remove helpers only the dropped commands used (`battery_guard`, `caffeinate_args`, bench/spike/hack/no-std/wasm functions) and their tests. `REQUIRED_READMES` = `[".claude", ".github", "docs", "docs/adr", "docs/chunks", "docs/lessons", "docs/research", "infra/github", "spikes", "xtask"]` (adjust the array length); the `spikes/<topic>/` rule stays. `WRITE_ALLOWLIST` keeps only entries for `codeql.yml` and `scorecard.yml`. The "missing command" hint lists the new commands. The `// developer tool, not simulation code` comment becomes `// developer tool`.
- **Test first** (in `main.rs` and `check_docs.rs` test modules): `dropped_commands_are_unknown` — `parse(&["bench"])`, `parse(&["mutants"])`, `parse(&["no-std"])` are `Err`; `ci_steps_are_the_ported_set` — asserts both lists above exactly; `required_readmes_are_the_new_tree` — `REQUIRED_READMES` contains `"docs/research"` and `"spikes"` and none of `"libs/engine"`, `"bench"`, `"fuzz"`, `"examples"`. Expected failure before the change: the copied code still has the old commands and list.
- **Verify:** `cargo test -p xtask` passes; `cargo xtask fmt --check`, `cargo xtask clippy` pass. (`check-docs` is verified in step 4.)
- **Commit:** `feat(xtask): port the process commands from rust-engine-experiment (M0.0)`

### Step 4: process documents and first ADRs

- [x] **Who:** main session
- **Files:** `AGENTS.md` (under 80 lines, `check-docs` limit), `CLAUDE.md` (`@AGENTS.md`), `CONTRIBUTING.md`, `docs/README.md`, `docs/workflow.md`, `docs/maintainers.md`, `docs/chunks/README.md`, `docs/chunks/_template/{spec,plan,log}.md`, `docs/adr/README.md`, `docs/adr/0001-licence-and-originality.md`, `docs/adr/0002-unsafe-policy-undecided.md`, `docs/lessons/README.md`, `docs/lessons/{tooling,docs,specs,xtask}.md`, `spikes/README.md`, placeholder `docs/research/README.md` (contents list filled in step 8) and `docs/roadmap.md` (filled in step 8).
- **Change:** as spec §5.2-5.7: CONTRIBUTING's "What CI enforces" lists only the step 3 checks plus the workflow rules; its review table has the four OS perspectives (§5.4); `docs/workflow.md` gains a "Research chunks" section (spec §5.6a) and loses the icebox wording; the spec template gains `**Kind:** build | research` and the OS cross-cutting list (§5.3), and says how §4/§5/§8 read for a research chunk; `docs/maintainers.md` has the Project section with `<number>` to fill in step 11, the UI-only workflow instructions (auto-add from this repo with no label exclusion; item added and reopened set Status Inbox; closed sets Done), and drops benchmarking and the commit-email item; lessons keep IDs, *From:* becomes `rust-engine-experiment M3.xx`, and `applies_to` globs point at paths that exist in the new tree.
- **Verify:** `cargo xtask check-docs` passes (it fails before this step with the missing READMEs); `cargo xtask typos` passes; leak check clean for these paths.
- **Commit:** `docs: port the workflow, conventions and templates, and add ADRs 0001-0002 (M0.0)`

### Step 5: `.claude/`

- [x] **Who:** main session
- **Files:** `.claude/README.md`, `.claude/settings.json` (hook; allowlist = `cargo xtask` `ci`, `ci --quick`, `test`, `test *`, `clippy`, `fmt --check`, `typos`, `doc`, `deny`, `check-workflows`, `check-docs`), `.claude/hooks/guard-push.sh`, `.claude/rules/{claude,ci,docs,xtask,spikes}.md`, `.claude/skills/{chunk,grill,triage,lessons-refresh}/` (stage files: research branches in stages 3, 4 and 5; no `/bench-run` or `/mutants-run`; owner and Project number), `.claude/agents/` (the 8 ported; `review-conditional` with the four OS perspectives; new `researcher.md` and `review-sources.md` per spec §5.6a, Opus, with `tools` lists that include `WebSearch, WebFetch`, `researcher` also `Write, Edit`).
- **Verify:**
  - Hook trial, in the new repo: `printf '%s' '{"tool_input":{"command":"git push origin main"}}' | sh .claude/hooks/guard-push.sh; echo $?` prints `2`; the same with `git push -u origin m0.0-foundations` prints `0`; with a heredoc commit message that mentions `git push` prints `0`.
  - Skill trial (lesson tooling-7): a fresh read-only general-purpose agent follows `.claude/skills/chunk/` for `/chunk next` against the new repo's roadmap as it will be after step 8, and for a hypothetical research chunk through stages 3-5, and reports what it had to guess. Fix what it finds before committing; log the trial.
  - `cargo xtask check-docs` passes; leak check clean.
- **Commit:** `feat(claude): port the agent roles, skills and push guard, and add the research roles (M0.0)`

### Step 6: `.github/`

- [x] **Who:** main session
- **Files:** `.github/workflows/ci.yml` (jobs `fmt`, `clippy`, `test`, `doc`, `deny`, `typos`, `check-workflows`, `check-docs` on `ubuntu-latest`, and `macos` running clippy and test on `macos-latest`; actions and their pinned hashes copied from the engine's `ci.yml`), `codeql.yml`, `scorecard.yml`, `dependabot.yml`, `CODEOWNERS`, `ISSUE_TEMPLATE/{config,bug_report,feature_request,infrastructure,research_question}.yml` (research form: The question, Why it matters, What would answer it, Sources to start from, the assistant-drafted box; label `research`), `PULL_REQUEST_TEMPLATE.md` (Reviews section: build variant with the eight reviews of spec §5.4; research variant with `review-spec`, `review-docs`, `review-sources`, and code/security "n/a unless a spike").
- **Verify:** `cargo xtask check-workflows` passes; `cargo xtask check-docs` passes; leak check clean.
- **Commit:** `ci: add CI, code scanning, Dependabot, issue forms and the PR template (M0.0)`

### Step 7: `infra/github/`

- [x] **Who:** builder (code), then main session (stack and preview)
- **Files:** `infra/github/{index.ts,privateVulnerabilityReporting.ts,Pulumi.yaml,package.json,tsconfig.json,.gitignore,README.md}`; `Pulumi.prod.yaml` is generated by `pulumi stack init`.
- **Change (builder):** `repoName = "rust-os-experiment"`; description as in step 1; topics `["rust", "operating-system", "exokernel", "microkernel", "osdev"]`; the `repo` resource gets `{ protect: true, import: "rust-os-experiment" }` with a comment that `import` may be removed after the first `up`; labels per spec §5.6 (`IssueLabels` stays authoritative, so GitHub's default labels not in the list are removed on `up`); `requiredChecks` = the nine `ci.yml` job names; `Pulumi.yaml` `name: rust-os-github`; `package.json` name `rust-os-experiment-github-infra`; README's owner/repo and stack name.
- **Verify (builder):** `pnpm install` then `pnpm --dir infra/github exec tsc --noEmit` passes. Job-name comparison: `grep -oE '^  [a-z0-9-]+:$' .github/workflows/ci.yml | tr -d ' :' | sort` equals the sorted `requiredChecks` list.
- **Verify (main session):** `cd infra/github && pulumi stack init SpencerAWill/rust-os-github/prod && GITHUB_TOKEN="$(gh auth token)" pulumi preview --diff`: the repository shows `= import` (no `+ create`, `- delete` or `+- replace`); record the summary line in `log.md`. If the import shows a diff on the repository's settings, that is expected (Pulumi applies them on `up`); a `replace` is a stop-and-fix.
- **Commit:** `feat(infra): declare the repository settings, ruleset and labels with Pulumi (M0.0)`

### Step 8: research documents and roadmap

- [x] **Who:** main session
- **Files:** `docs/research/{README,architecture-landscape,shim-layer,toolchain-and-boot,compatibility,realities-checklist,open-questions}.md`, `docs/roadmap.md`.
- **Change:** spec §6. Source: `~/Repos/rust-engine-experiment/transcripts.txt`, read section by section; distilled in our own words, not pasted; each file opens with the status line `Reference: not a decision (2026-11-17)` and the provenance note. `open-questions.md` ranks the questions and maps each to M1.1-M1.6. The roadmap has the milestone table and M1 rows of spec §6.2 with status `Planned`, M0 row `Current`, and the row-status legend from the engine's roadmap.
- **Verify:** `cargo xtask check-docs` and `cargo xtask typos` pass; `grep -c 'Use code with caution\|Show Code' docs/research/*.md` prints 0 for every file (no pasted transcript); every `docs/research/*.md` except README contains a "Corrections" or "Unverified" note where spec §6.3 applies.
- **Commit:** `docs: seed the research notes and the roadmap from the initial architecture conversation (M0.0)`

### Step 9: full CI locally

- [x] **Who:** main session
- **Verify:** `cargo xtask ci` passes; leak check clean across the whole tree; `git status --short` empty.
- **Commit:** none unless a fix was needed (`fix: ... (M0.0)` through a builder for code).

### Step 10: review (stage 5)

- [ ] **Who:** reviewers, then builder for fixes
- **Do:** this session has the engine's agent files registered, so dispatch general-purpose agents with the agent file's model and the prompt "Work in `~/Repos/rust-os-experiment`. Read `.claude/agents/<role>.md` there and follow its body", plus the spec path, `main...HEAD`, and the lesson IDs of §4. Roles: `review-code`, `review-security`, `review-docs`, `review-spec`, `review-conditional` (Interfaces and ABI: n/a, no product code; Architecture portability: applies to CI's macOS job and scripts' line endings; Concurrency: n/a; Performance: n/a). No simplify pass over documents; one over `xtask/` and `infra/` through a builder (`/simplify`), committed as `refactor: simplify (M0.0)` if it changes anything. Triage, fix through a builder, re-review with fresh agents; two rounds without agreement → ask the maintainer. Log each one-line result.
- **Commit:** fixes as `fix: <what> (M0.0)`.

### Step 11: compound, PR, GitHub objects, close out

- [x] **Who:** main session
- **Do, in order:**
  1. Stage 6: `lesson?` lines from `log.md` through the capture gate into `docs/lessons/`; commit `docs: record the lessons of M0.0 (M0.0)`.
  2. Push the branch; open the PR from the template (title `feat: repository foundations (M0.0)`), saying an assistant was used.
  3. Create the Project: `gh project create --owner SpencerAWill --title "OS backlog"`; link it to the repo (`gh project link`); add single-select fields Status (Inbox, Ready, Scheduled, Done), Priority (Now, Next, Later), Size (S, M, L, XL) with `gh project field-create`. Views and the built-in workflows are UI-only: listed for the maintainer in the PR and in `docs/maintainers.md`.
  4. Open the M1 tracking issue and one `research` sub-issue per M1 row (`gh issue create` with the research form's headings, `--label research,ai-assisted`; link with `gh api --method POST repos/SpencerAWill/rust-os-experiment/issues/<tracking>/sub_issues -F sub_issue_id=<id>`); add them to the Project. Labels other than GitHub's defaults do not exist until `pulumi up`, so create these issues **after** the maintainer's `pulumi up` if `research` is missing, or add the labels afterwards; note which in `log.md`.
  5. Close-out commit: spec `Implemented (PR #n, date)`, plan `Historical: executed in PR #n (date); the spec and the code are authoritative` with Deviations, log `Historical: PR #n (date)`, roadmap M0 `Done (PR #n)` and M1 `Current`, Project number in `docs/maintainers.md`, the M1 rows' issue numbers in the roadmap, done-when ticked with evidence, CHANGELOG. Push.
  6. Seed the new repo's Claude memory directory (`~/.claude/projects/-Users-spencer-Repos-rust-os-experiment/memory/`) with copies of `tone-neutral-no-glazing`, `user-profile-and-work-style` (OS context noted), `no-copying-other-engines` (generalized to "other projects"), `one-pr-per-chunk-closeout-in-it`, `long-jobs-need-memory-caps`, `direct-push-to-main-ok`, and a new `project-os-goals` (targets, M4 milestone, research-first), with a `MEMORY.md` index. Not in git.
  7. Wait for the merge go-ahead; then the merge procedure of `.claude/skills/chunk/stages/7-close-out.md` step 5 (CI must be green; no ruleset exists yet, so the wait on checks is the only gate). Then the maintainer: Project views and workflows in the UI; `pulumi up` in `infra/github`; `/chunk next` in a new session in the new repo.

## 3. Final verification

`cargo xtask ci` on the final commit; CI green on the PR (all nine jobs); the spec's done-when list.

## 4. Lessons

From the engine's `docs/lessons/` (this chunk ports them; they apply to the same kinds of paths):

- tooling-1, tooling-2, tooling-6: followed in step 5 (hook fail-closed, own target dir, LF endings).
- tooling-3: followed in step 5 (allowlist only `cargo xtask` wrappers).
- tooling-7: followed in step 5 (skill trial).
- tooling-8, tooling-9, tooling-10: followed in step 5 (`triage` and stage 1 keep their guards, `--method GET`, `--limit`).
- docs-1, docs-2: followed in step 4 (one copy of each rule; new header fields go in the template).
- specs-1: followed (the leak check names its paths and exclusions).
- specs-4: followed in step 9 (evidence names its commit).
- xtask-1, xtask-2: xtask-1 kept by porting the root helper unchanged; xtask-2 deviated: no CRLF trial, because `check-docs` is ported unchanged apart from its README list.

## 5. Deviations

- **Builder steps (3, 7) were done by the main session**, and the skill trial of step 5 was skipped, because the maintainer asked to keep token use low; the hook trial was run (push to `main` blocked with 2, branch push and a heredoc mentioning `git push` allowed with 0).
- **The spec keeps the approved design's own section layout** rather than being reshaped into the template.
- **`xtask test` skips doctests while no workspace member has a library target** (`cargo test --doc` fails outright otherwise), with a test for the detection.
- **`.cargo/config.toml`** (the `xtask` alias) was missing from the port list and was added.
- **`.github` is not in `REQUIRED_READMES`:** GitHub would show `.github/README.md` instead of the root README.
- **Step 10 (review) was not run** and step 11 left the M1 issues for after `pulumi up` (the `research` label does not exist before it); both are in the PR.
- **`pulumi config set github:owner SpencerAWill`** was needed for the preview; it is in `Pulumi.prod.yaml`.
