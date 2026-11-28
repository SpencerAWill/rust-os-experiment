# Contributing

Thanks for your interest. This is an experiment in operating-system design: nothing is stable, and most decisions are still open (see the [roadmap](docs/roadmap.md)). For anything beyond a small fix, **open an issue first** so we can agree on the direction before you spend time on it.

## How work is organized

- Work is planned as milestones (goals and exit criteria in [docs/roadmap.md](docs/roadmap.md)) broken into **chunks** of exactly one pull request each. Each chunk is a GitHub issue, scheduled with its milestone and an order in the "OS backlog" Project ([docs/workflow.md](docs/workflow.md#from-issue-to-chunk)); small scheduled build work without a chunk ID is a chore.
- Each chunk goes through the loop in [docs/workflow.md](docs/workflow.md): its `spec.md`, `plan.md` and `log.md` live in `docs/chunks/<id>-<slug>/`, copied from [docs/chunks/_template/](docs/chunks/_template/). A chunk is of kind `build` (code) or `research` (a document, and an ADR when it decides something). The maintainer approves the spec before the work starts, and gives the go-ahead to merge.
- Pull request titles end with the chunk ID, for example `docs: research the privilege model (M1.1)`; a chore's end with its issue number, for example `chore(xtask): drop an unused dependency (#42)`.
- Ideas and bugs go into issues; questions go to [Discussions](https://github.com/SpencerAWill/rust-os-experiment/discussions). When an issue is scheduled, a chunk spec links it.

## Issues

Blank issues are off. Pick the template that fits:

| You want to | Template | Label |
| --- | --- | --- |
| Report behaviour that differs from the documentation | Bug report | `bug` |
| Propose a feature, a design change, or an unmeasured performance idea | Feature request | `enhancement` |
| Ask a question that needs research before anything can be decided | Research question | `research` |
| Propose work on CI, tooling, the repository layout or settings, or this process | Infrastructure or chore | `infrastructure` |
| Ask a question about the project | none: use Discussions | |
| Report a vulnerability | none: report it privately ([SECURITY.md](SECURITY.md)) | |

The templates are issue forms in `.github/ISSUE_TEMPLATE/`; the labels they assign are declared in `infra/github/index.ts`, so a new label goes there too. How an issue becomes a chunk is in [docs/workflow.md](docs/workflow.md#from-issue-to-chunk). An issue filed with `gh issue create` (by a person or an assistant) uses the headings of the matching form. An assistant-drafted issue also says "An assistant drafted this issue." and is created with `--label ai-assisted`. To work on an open issue, assign yourself or comment first, so the work is not done twice.

## Licence and originality

The project is dual-licensed `MIT OR Apache-2.0`. Unless you state otherwise, a contribution you submit is licensed under both, with no additional terms (see the README). The rules that make that safe are in [ADR 0001](docs/adr/0001-licence-and-originality.md):

- **Do not copy code from other projects**, including other kernels and OS crates. Study designs, papers, manuals and talks, and cite them. Reading another project's source to understand its behaviour is fine; transcribing it is not.
- **AI conversations are sources of ideas, never of code.** Code an AI model produced in a research conversation is not pasted in.
- A new dependency must pass `cargo xtask deny` (allowed licences, no unknown registries or git sources).
- If you used an AI assistant, say so in the pull request.

## Setting up

Prerequisites: [Homebrew](https://brew.sh), [rustup](https://rustup.rs), and the Xcode Command Line Tools (`xcode-select --install`, for the linker).

```sh
brew bundle                 # host tools from Brewfile (QEMU, dtc, socat, poppler, acpica, gdb)
rustup show                 # installs the pinned toolchain from rust-toolchain.toml
cargo xtask doctor          # checks tools, firmware and accelerators; smoke-boots each architecture
cargo xtask refs --fetch    # downloads the reference library into reference/ (gitignored)
cargo xtask ci              # every check CI gates on, in order
cargo xtask ci --quick      # fmt check, clippy, and tests only
```

`cargo xtask` prints an install hint when a tool is missing. The extra tools are `cargo-nextest`, `cargo-deny` and `typos-cli`.

## What CI enforces

These are policy, not preferences. A pull request that breaks one will not be merged:

- Formatting (`cargo fmt`), clippy with warnings denied, docs without warnings, tests, `cargo deny`, typos.
- **Workflows** (`check-workflows`): actions pinned to commit hashes, a read-only token (a job-level `write` scope only if it is in `WRITE_ALLOWLIST` in `xtask/src/check_workflows.rs`, with a reason), no persisted checkout credentials, no `pull_request_target`.
- **Documentation layout** (`check-docs`): the conventions in [docs/README.md](docs/README.md#conventions).
- **`unsafe`:** undecided until an M1 chunk replaces [ADR 0002](docs/adr/0002-unsafe-policy-undecided.md); until then no crate other than spikes uses it.

## Tests

Changes come with tests. For a bug fix, write the test first and confirm it fails for the reason you expect, then fix the code. Prefer testing behaviour over implementation. Do not weaken or delete a failing test without saying so in the pull request. How kernel code is tested (on the host, under QEMU, on hardware) is decided in M1.

## Tools

How to run each tool, what its output means and its rules live next to it: every `cargo xtask` command in [xtask/README.md](xtask/README.md), spike code in [spikes/README.md](spikes/README.md), repository settings in [infra/github/README.md](infra/github/README.md).

## Commits and pull requests

- Use conventional commit messages (`feat:`, `fix:`, `perf:`, `refactor:`, `test:`, `docs:`, `ci:`, `chore:`), and split work into small logical commits that each pass `cargo xtask ci`.
- Explain why, not just what. Mention any change to an interface another component depends on, or to a policy, explicitly.
- Add or update an ADR in `docs/adr/` for a decision that is hard to reverse.
- Pull requests are squash-merged, the only merge method `main` allows. The pull request title becomes the commit subject on `main`, so it MUST be a conventional commit message (for a chunk, ending with the chunk ID; for a chore, with its issue number).

## Reviews before a pull request

Before opening a pull request, review the change from each perspective below, whoever or whatever wrote it. In the chunk loop each review is done by an agent that did not write the change, using the reviewer files in `.claude/agents/` (named below); a person can run the same checklist by hand.

**Always (build chunks)**

- **Code review** (`review-code`): correctness bugs, checks that cannot fail (a test that passes whatever the code does), missed edge cases, and rule breaks from "What CI enforces".
- **Security review** (`review-security`): `unsafe`, input handling and trust boundaries, dependency and licence changes (new crates, their licences against [ADR 0001](docs/adr/0001-licence-and-originality.md), their provenance), and workflows (token permissions, pinned actions, secrets, untrusted input in `run:` steps).
- **Documentation review** (`review-docs`): every new public item is documented; new documents have a status line; relative links resolve; `CHANGELOG.md` is current; the chunk is closed out in this pull request (spec `Implemented (PR #n, date)`, plan and log `Historical`, `Closes #<chunk issue>` in the description, the spec's done-when list ticked with evidence); a spec's section numbers stay stable; anything that differs from the plan is in its Deviations section; and a claim in the docs is one you ran, not one you inferred.
- **Spec conformance review** (`review-spec`): for each requirement in the chunk's spec (§5), point to the evidence in its §8 that it holds, and check that what the pull request claims is what was actually run.

**Always (research chunks):** `review-spec`, `review-docs`, and **source review** (`review-sources`): every factual claim has a source that says it, and the ADR's decision follows from the evidence. Code and security review apply only to spike code the chunk wrote.

**When the change touches it** (`review-conditional`, told which apply). These are mandatory too, but you may answer "n/a" with the reason.

| Review | Applies when | Asks |
| --- | --- | --- |
| **Interfaces and ABI** | a layout, call or message format another component (or another layer) depends on is added or changed | Is the change deliberate and recorded? Is the layout explicit (`#[repr(C)]`, sizes, alignment, endianness)? Is it versioned or marked as unstable? |
| **Architecture portability** | code or decisions that differ per architecture, or CI and build changes | Does it hold on x86_64, aarch64 and riscv64? Is per-architecture code confined, and the shared part really shared? |
| **Concurrency and memory ordering** | atomics, interrupts, multi-core data, DMA, shared rings | Are the orderings justified? What happens under interrupt, on another core, or with a device writing memory? |
| **Performance** | a hot path changes, or the pull request makes a performance claim | What was measured, on what machine and in what conditions, against what baseline, and what is the noise? |

Triage every finding: fix it, or write in the pull request why you did not. The pull request description has a Reviews section that lists each review with a one-line result, or `n/a: <reason>`.

## Security

Report vulnerabilities privately, as described in [SECURITY.md](SECURITY.md), not in public issues.
