# M0.0: Repository foundations

- **Status:** Accepted (2026-11-17).
- **Kind:** build · **Milestone:** M0 (repository foundations) · **Depends on:** none · **Decides:** ADR 0001, ADR 0002
- **Issue:** none

This spec is the design approved on 2026-11-17, kept in its own section layout rather than the template's (the template did not exist yet). Its sections 1-9 map to the template as: §1-2 Summary and goals, §3 constraints, §4 alternatives, §5-6 the design, §7-9 the work and the calls made.

## 1. Intent

The maintainer wants a second Rust experiment: an operating-system stack built around an exokernel, a "shim" interposition layer, and microkernels / library OSes on top, targeting x86_64, aarch64 and riscv64, developed on an M4 Pro Mac. The architecture was explored in a conversation with Gemini (`transcripts.txt` in the engine repo, not committed anywhere).

The engine repo added its AI-assisted workflow (roadmap, chunks, spec/plan/log, grill, builder and reviewer agents, lessons, issue triage) after 15+ chunks. The new repository has it **from the first commit**, so every design decision of the OS goes through it.

Nothing about the OS is designed here. Designs, specs and ADRs start from scratch in the new repository; the next sessions continue the research there, through the loop.

## 2. Goals, non-goals, anti-goals

**Goals**

1. The new repository exists on GitHub with the chunk loop usable on day one: `/chunk next` in a fresh session in the new repo picks the first M1 chunk and runs it.
2. The process layer (docs, templates, skills, agents, hook, issue forms, PR template, lessons format) is ported and made project-neutral; nothing in it refers to the engine, ECS, determinism of simulations, PRNGs, GenIds or benchmarks that do not exist.
3. `cargo xtask ci` passes locally and in GitHub Actions on a workspace that contains only `xtask`.
4. Repository settings, labels, the `main` ruleset and required checks are declared in a new Pulumi stack, and a GitHub Project for issue triage exists, as in the engine repo.
5. The transcript's content is distilled into research documents and a roadmap of open questions, so the research sessions start from it rather than from the raw conversation.

**Non-goals**

- No kernel, bootloader, target triple, linker script, QEMU runner or boot test. Those are decided by research chunks (M1) and built in later milestones.
- No ADRs from the engine repo. The new repo's ADR index starts empty except as stated in §5.7.
- No fuzzing, coverage, mutation testing, benchmarks, Miri, devcontainer or `/bench-run` / `/mutants-run` skills yet: there is no code for them to apply to. The roadmap names when each arrives.
- No changes to the engine repo.

**Anti-goals** (would make this a failure even if it technically works)

- Engine-specific rules leaking in (for example, `forbid(unsafe_code)` presented as a project policy, or the `HashMap` bans), because a kernel needs `unsafe` and global per-CPU state, and these decisions belong to the OS's own ADRs.
- Research documents that read as decisions. The transcript is one AI model's output; its claims are unverified and some are wrong (§6.3).
- Copying the raw transcript into the repository.

## 3. Constraints

- The engine repo's rules still bind the work done there: nothing in it changes.
- **Pushes to `main` and force pushes are blocked for agents** (the push guard hook of this session). The new repo is created with an initial commit made by GitHub (`gh repo create --add-readme`), so all work then arrives through a pull request and no agent pushes to `main`.
- Code ported from the engine repo is the maintainer's own; copying it is allowed. Nothing is copied from other projects (the originality rule is restated in the new repo, §5.7).
- Outward-facing actions authorized by approving this design: creating the repository, pushing the bootstrap branch, opening its pull request, creating the GitHub Project and its fields, opening the M1 tracking issue. `pulumi up` is run by the maintainer, after the merge (it needs their token and creates the ruleset).
- QEMU is not installed on the Mac; nothing in the bootstrap needs it.

## 4. Approaches

| Approach | Trade-off | Outcome |
|---|---|---|
| **A. Port and adapt the whole process layer in one bootstrap pull request** | Full workflow on day one; one large first PR | **Chosen by the maintainer** |
| B. Small seed (docs, skills, agents, shell-only hook, basic CI), then port `check-docs`, `check-workflows`, Pulumi and the Project as the first chunks through the loop | Uses the loop from day one, but costs 4-5 plumbing chunks before research and has a weaker push guard until `xtask guard-push` arrives | Rejected |
| C. Extract the process into a shared template repository both repos consume | One copy of the process; requires reworking the engine repo and maintaining a third repo | Rejected (scope) |

## 5. What gets built

### 5.1 Repository and history

- `gh repo create SpencerAWill/rust-os-experiment --public --add-readme --description "<one line>"`, cloned to `~/Repos/rust-os-experiment`.
- Branch `m0.0-foundations`; small logical commits by explicit path, conventional messages ending `(M0.0)`; one pull request, squash-merged on the maintainer's go-ahead.
- Licence: `MIT OR Apache-2.0`, both licence files, the contribution clause in the README *(main session's call, see §8)*.
- Commit author email: the maintainer's current git config (personal address), **decided by the maintainer** knowing the repository is public from the first commit. The new `docs/maintainers.md` does not carry the engine's "decide the commit email" item.

### 5.2 Files ported with light edits (names, links, owner/repo, project neutral wording)

| From the engine repo | Notes |
|---|---|
| `AGENTS.md`, `CLAUDE.md` (`@AGENTS.md`) | Commands and "Rules that fail CI" rewritten to what the bootstrap CI actually enforces (§5.5); "Where to look" table for the new paths |
| `CONTRIBUTING.md` | Structure kept; the engine's policy bullets removed; "What CI enforces" lists only the bootstrap checks; the reviews section updated for the new conditional perspectives (§5.4) |
| `docs/workflow.md` | Kept nearly verbatim (it is project-neutral); M3.15/M3.18 references replaced by links to this bootstrap's spec |
| `docs/README.md`, `docs/chunks/README.md`, `docs/adr/README.md`, `docs/lessons/README.md`, `docs/maintainers.md` | Conventions kept; the milestone-renumbering note, benchmark and mutation sections dropped; Project name and number updated |
| `docs/chunks/_template/{spec,plan,log}.md` | Spec §6 "Cross-cutting concerns" rewritten for an OS (§5.3) |
| `.claude/README.md`, `.claude/settings.json`, `.claude/hooks/guard-push.sh`, `.claude/rules/*` (only for directories that exist: `xtask`, `docs`, `.claude`, CI) | Allowlist trimmed to the bootstrap's xtask commands |
| `.claude/skills/{chunk,grill,triage,lessons-refresh}` | `chunk` stage files: references to `/bench-run`, `/mutants-run`, `perf-regression-accepted` and the engine's review list removed; owner and Project number updated |
| `.claude/agents/*` (all 8) | `review-conditional` gets the OS perspectives (§5.4); two new agents for research chunks (§5.6a) |
| `.github/PULL_REQUEST_TEMPLATE.md`, `ISSUE_TEMPLATE/{bug_report,feature_request,infrastructure,config}.yml`, `dependabot.yml`, `CODEOWNERS`, `workflows/{ci,codeql,scorecard}.yml` | The `determinism_bug` and `performance_report` forms are dropped; a `research_question` form is added (§5.6); CI trimmed to the bootstrap jobs |
| `xtask/` commands `ci`, `fmt`, `clippy`, `test`, `doc`, `deny`, `typos`, `check-docs`, `check-workflows`, `guard-push` and their tests | Other commands and modules are not ported. `check-docs`'s `REQUIRED_READMES` list is the new tree's |
| `infra/github/` (Pulumi, TypeScript) | New Pulumi project name (so it does not collide with the engine's `github` project in Pulumi Cloud); labels for the new repo (§5.6); `requiredChecks` = the new `ci.yml` job names. The repository already exists (created by `gh`), so the `github.Repository` resource adopts it with the `import` resource option, and the default labels GitHub created are imported or replaced the same way. The main session runs `pulumi stack init` (the CLI is logged in as `SpencerAWill`) and `pulumi preview --diff` before the PR is opened, and records the preview in `log.md`; the maintainer runs `pulumi up` after the merge *(main session's call)* |
| `deny.toml`, `_typos.toml`, `clippy.toml` (lint settings only, no engine bans), `rustfmt.toml`, `rust-toolchain.toml`, `.gitattributes`, `.gitignore`, `package.json`, `pnpm-workspace.yaml`, `SECURITY.md`, `CHANGELOG.md` | `rust-toolchain.toml` pins the same stable as the engine (1.100.0) with no extra targets; the targets arrive with the first boot chunk |
| Process lessons `tooling-1..10`, `docs-1..2`, `specs-1..4`, `xtask-1..2` | Kept with their IDs and *From:* rewritten as `rust-engine-experiment <chunk>`; `specs-5` and `testing-*`, `perf-*`, `bench-*` dropped (engine-specific) |

### 5.3 Spec template: cross-cutting concerns for an OS

The engine's list (determinism, heap allocation, `no_std` and wasm, frozen formats, FFI) is replaced by:

- **`unsafe`** and the invariants each block relies on
- **Privilege and trust boundaries** crossed (which component trusts which input)
- **Architecture portability** (x86_64, aarch64, riscv64: what is shared, what is per-arch)
- **Concurrency and memory ordering** (multi-core, interrupts, atomics, DMA coherence)
- **Interfaces and ABI** (any layout or call another component depends on; frozen or not)
- **Hardware and spec assumptions** (which manual, spec version or QEMU machine a claim relies on)
- **Testing level** (host unit tests, QEMU, real hardware)
- **Performance** (what is measured, against what)
- **Documentation**, **Policy changes**

### 5.4 Review perspectives

Always: code, security, documentation, spec conformance (unchanged agents). Conditional (`review-conditional`): **Interfaces and ABI**, **Architecture portability**, **Concurrency and memory ordering**, **Performance**. The PR template's Reviews section lists these eight.

### 5.5 Workspace and CI

- Cargo workspace with one member, `xtask` (edition 2024, `publish = false`, `#![forbid(unsafe_code)]` in `xtask` itself as a property of that tool, not a project policy).
- `ci.yml` jobs: `fmt`, `clippy`, `test`, `doc`, `deny`, `typos`, `check-workflows`, `check-docs`, and `macos` (clippy + test on `macos-latest`, since the maintainer develops there). Every action pinned to a commit hash, read-only token, as `check-workflows` requires.
- `codeql.yml` and `scorecard.yml` as in the engine (not required checks).

### 5.6 Issue tracking

- Labels: type labels `bug`, `enhancement`, `infrastructure`, `documentation`, `research`; additive `needs-measurement`, `ai-assisted`; `area:xtask`, `area:docs`; the Dependabot labels. `area:*` labels for kernel components are added by the chunks that create them.
- A `research_question` issue form (question, why it matters, what would answer it, sources), labelled `research`. Unlike the engine (where `research` was an unscheduled icebox), **research is scheduled work here**: research issues are Project items and become M1 chunks. *(main session's call)*
- Project: user-owned, named "OS backlog", fields Status (Inbox, Ready, Scheduled, Done), Priority (Now, Next, Later), Size (S, M, L, XL) and views, created with `gh project`; its number is recorded in `docs/maintainers.md`. The built-in workflows (auto-add, item added/reopened → Inbox, closed → Done) can only be set in the web UI, so they are a maintainer step with exact instructions in `docs/maintainers.md`, done before the merge. Auto-add has **no** `research` exclusion here. *(main session's call)*
- One tracking issue for M1, with one sub-issue per M1 chunk row (§6.2), opened in stage 7 of the bootstrap.

### 5.6a Research chunks (decided by the maintainer: a research variant of the loop, in the bootstrap)

A chunk's spec header carries **Kind:** `build` (the default, the loop as ported) or `research`. A research chunk runs the same seven stages and the same two waits, with these differences, written into `docs/workflow.md`, the chunk stage files and the templates:

- **Spec.** §1 states the question and what decision it unblocks. §4 lists the sub-questions, the sources to consult first, and what evidence would settle each. §5 requirements are about the document and the evidence (for example, "R2: each option for X is described with its cost on each architecture, with a cited source"), plus "R<n>: an ADR records the decision" when the chunk decides something. §6's cross-cutting list applies to the decision being studied.
- **Work (stage 4).** A new **`researcher`** agent (`.claude/agents/researcher.md`, Opus, tools: read, web search and fetch, write within `docs/research/` and the chunk folder; no commits) writes one plan step at a time: a section of the research document, every factual claim with a citation (URL, manual section, or "measured here: <command>"). A plan step may instead be a **spike** done by a `builder`: throwaway code under `spikes/<topic>/` that answers a question by running it (for example, "does QEMU with HVF expose EL2 to a guest on this Mac?"), its result logged with the command and output. The ADR, when there is one, is a main-session step after the research steps.
- **Review (stage 5).** No simplify pass on documents. Always: `review-spec`, `review-docs`, and a new **`review-sources`** agent (Opus, web access, read-only) that checks each claim against its cited source, flags claims with no source or a source that does not say it, and checks that the ADR's decision follows from the evidence. `review-code` and `review-security` apply only if a spike was written (security also when a spike adds a dependency or a script). The PR template gets a research variant of its Reviews section.
- **Permissions.** `.claude/settings.json` does not allow web tools globally; the two agent files grant them through their `tools` lists.
- **Vocabulary.** `docs/workflow.md`'s "icebox = `research` issues" is removed: `research` issues are scheduled work, Project items like any other (§5.6). An idea parked for later stays in the Inbox with Priority Later.
- **Spikes directory.** `spikes/README.md` is ported (adapted) so spike code has a home and rules (not production, may use `unsafe` freely while ADR 0002 is a placeholder, deleted or kept per the chunk's spec), with `check-docs` requiring `spikes/<topic>/README.md` as in the engine.

### 5.7 First ADRs

Only the two that a public repository needs from its first commit, written fresh for this project:

- **ADR 0001: Licence and originality.** `MIT OR Apache-2.0`; no code copied from other projects (Redox, Theseus, seL4, Hermit, Limine and others may be studied and cited); AI output, including the Gemini conversation, is a source of ideas, never of code; allowed dependency licences as enforced by `deny.toml`.
- **ADR 0002: The `unsafe` placeholder.** States that the `unsafe` policy is undecided and will be set by an M1 chunk before the first kernel code; until then the only crate (`xtask`) forbids it. *(main session's call: this keeps the anti-goal in §2 visible.)*

Everything else (architecture, bootloader, toolchain, testing strategy) is an M1 decision.

## 6. Transcript content in the repository

### 6.1 Research documents (`docs/research/`, status `Reference: not a decision`)

Each opens with a provenance note: distilled from a conversation with an AI model (Google Gemini) on 2026-11 (exact date unknown); claims are unverified unless a source is cited; code in the conversation is not reused (ADR 0001).

| File | Content |
|---|---|
| `README.md` | Index; how research becomes a decision (a research chunk produces a document here and, when it decides something, an ADR) |
| `architecture-landscape.md` | The concepts surveyed and how they relate: monolithic gripes that motivated it; microkernel vs exokernel + library OS; SASOS, thread-per-core, capabilities, async ring IPC, typed IPC, live-swappable components, user-space drivers via IOMMU; which ideas conflict (SASOS vs hardware isolation, etc.) and the "tiny core makes the hard choice" point; exokernel vs Ring −1 hypervisor; running Linux/Windows beside it; the planes / capability-graph model versus "layers" |
| `shim-layer.md` | The "Layer −0.5" concepts (deterministic record/replay, chaos/fault injection, capability firewall, confidential-compute veil, disaggregation, live mirroring, ISA translation, energy governor), their claimed costs, naming (monitor, shim, interposition layer, nanokernel), and the transcript's final proposal: a feature-gated `exo-shim` crate compiled into the exokernel at the trap boundary, with per-domain policies |
| `toolchain-and-boot.md` | Bootloader options (Limine vs the `bootloader` crate, writing one's own); target triples; what Rust can and cannot reach (asm, linker scripts, firmware, microcode); testing on the M4 Pro (QEMU TCG vs HVF, UTM); the documentation set an OS needs (ISA manuals, ACPI/DT, UEFI, PCIe, VirtIO); the xtask build/run pattern and QEMU exit devices |
| `compatibility.md` | Running existing software: WASI runtime, libc / Rust `std` port, Linux syscall personality (Starnix-style), paravirtualized Linux domain; expected cost and speed of each |
| `realities-checklist.md` | The "have we thought of everything" lists: drivers, `std` porting, memory ordering and DMA coherence, timekeeping, power, graphics and input, crash recovery, OOM and quotas, side channels, boot entropy, signed builds, debugging (serial, unwinding, GDB stub), scope management |
| `open-questions.md` | Ranked questions for M1, each with what it blocks and the chunk that answers it (§6.2) |

Performance figures from the conversation (the speedup table, the per-layer overhead table) go in only as "claimed by the model, unsourced", or not at all.

### 6.2 Roadmap (`docs/roadmap.md`)

| Milestone | Goal | Status |
|---|---|---|
| M0 | Repository foundations: the workflow, CI, settings, research seed | Current (Done when this PR merges) |
| M1 | Research and architecture decisions | Next |
| M2 | Boot skeleton: one binary boots in QEMU on all three architectures, prints over serial, and is tested headless in CI | Later |
| M3 | Exokernel core: physical memory, capabilities, domains | Later |
| M4 | Two microkernels / library OSes running side by side, exchanging a typed message (the maintainer's v0.1 milestone) | Later |
| M5 | Shim: the first interposition features | Later |
| Later | Compatibility adapters; drivers beyond VirtIO; real hardware | Unnumbered |

M1 chunk rows (research chunks; each produces a research document and, where it decides, an ADR; order by dependency):

| Chunk | Question | Depends on |
|---|---|---|
| M1.1 | Privilege model: where the exokernel and the microkernels run (exokernel in ring 0/EL1/S-mode with user-mode library OSes, versus exokernel using VT-x/EL2/the H extension with microkernels in guest kernel mode), per architecture, and what QEMU on an M4 Pro supports for each | none |
| M1.2 | Boot path and toolchain: bootloader, target triples, stable vs nightly, linker scripts, image building | none |
| M1.3 | Test and debug strategy: host unit tests vs QEMU tests, CI with QEMU, GDB, serial; the `unsafe` policy (replaces ADR 0002) | M1.2 |
| M1.4 | Exokernel interface: resources, capabilities, the hypercall/event ABI, revocation | M1.1 |
| M1.5 | Shim placement and first features | M1.1, M1.4 |
| M1.6 | Crate and workspace layout for multiple binaries and targets | M1.1, M1.2 |

M2 onward stays rough until M1 ends.

### 6.3 Corrections recorded with the research

At least these, so the research does not inherit them:

- The conversation places the exokernel in ring 0 without virtualization while also running microkernels that need privileged control; M1.1 resolves this.
- Its `exo_call` inline assembly binds `rax` twice and would not compile; it is illustrative only.
- Speedup and overhead percentages have no sources.

## 7. How the work is done

1. Create the repository (§5.1); clone; branch.
2. Port and adapt in logical commits: process docs; templates; `.claude/`; `xtask` (with its tests passing); CI and repo files; `infra/github/`; ADRs; research documents and roadmap.
3. `cargo xtask ci` passes locally.
4. Review with the eight reviewer roles (dispatched as general-purpose agents told to follow the new repo's agent files, since this session registered the engine's), fixes, then open the PR; CI green.
5. Create the Project and the M1 tracking issue with its sub-issues.
6. Seed the new repo's Claude Code memory directory with the maintainer-level memories that apply to any project (tone, work style, no copying, one PR per chunk, long jobs need memory caps, direct push to main is not standing).
7. Merge on the go-ahead; the maintainer runs `pulumi up` in `infra/github/` (ruleset, labels, required checks).

## 8. Decided by the main session (check while reading)

- Licence `MIT OR Apache-2.0` and the two fresh ADRs (§5.7).
- Research is scheduled work (research issues are Project items) rather than an icebox.
- The OS cross-cutting concerns and review perspectives (§5.3, §5.4).
- Which engine lessons are ported (§5.2).
- Toolchain pinned to the engine's stable 1.100.0 for now.
- The M1 chunk list and order (§6.2).
- The raw transcript is not committed.
- The shape of research chunks within the maintainer's choice (§5.6a): the `researcher` and `review-sources` agents, their models and tools, spikes allowed.
- The Project's built-in workflows are a maintainer UI step; Pulumi imports the existing repository; the main session runs `pulumi preview` before the PR (§5.2, §5.6).

## 8a. Decided by the maintainer

- Approach A (§4); name `rust-os-experiment`; public.
- Commits use the personal email (§5.1).
- Research chunks are a variant of the loop built in the bootstrap (§5.6a).
- No engine ADRs; designs start from scratch; transcript content goes into documents (§6).

## 9. Left to the builder

File-by-file wording, the exact commit split, the Pulumi project name, and which memory files are copied (within §7 step 6).
