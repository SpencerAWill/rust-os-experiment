# Documentation

Status: Living (2026-11-17). Why things are the way they are, how they were designed, what was researched, and what is planned.

## When this applies

- You look for why something is the way it is, what is planned, or what was researched.
- You write or change any document in the repository: the conventions below apply everywhere, not only in `docs/`.

## Contents

| Path | What it is | Kind | How far to trust it |
|---|---|---|---|
| [`roadmap.md`](roadmap.md) | Milestones, their exit criteria, and the current milestone's chunks with their status | Reference | **Authoritative for what is planned and its status.** |
| [`workflow.md`](workflow.md) | The chunk loop (build and research chunks): stages, the two waits, agent roles and models, `log.md`, lessons, the grill; the path from issue to chunk | Reference | **Authoritative for how work is done.** The rules a change must meet are in CONTRIBUTING.md. |
| [`adr/`](adr/) | Architecture decision records: one short file per decision that is hard to reverse | Explanation | **Authoritative.** Each has a `Status` line. A later ADR may supersede an earlier one. |
| [`chunks/`](chunks/) | One flat folder per chunk, `chunks/<id>-<slug>/`, with its `spec.md`, `plan.md` and `log.md`; [`_template/`](chunks/_template/) holds the templates | Explanation | The **spec** is a living design document: where it differs from the code or an ADR, the code and the ADR win. A **plan** is a historical record once its pull request merges. |
| [`lessons/`](lessons/) | Lessons learned in earlier chunks, one file per topic, each listing the paths it applies to; read by path when planning | Reference | **Guidance, not rules:** followed by default, deviations recorded with a reason. |
| [`research/`](research/) | Research notes: surveys of designs, hardware and tools, and the open questions they leave | Reference | **Not decisions.** Read a note's sources and its provenance line before a spec relies on it. |
| [`maintainers.md`](maintainers.md) | Repository settings, the issue tracking Project, and the checklists for merging and milestones | How-to | Operational; update it when the process changes. |

Rules that are checked mechanically live in the tooling and in [CONTRIBUTING.md](../CONTRIBUTING.md), not here. Reporting a vulnerability is in [SECURITY.md](../SECURITY.md).

## Conventions

- **A README in each directory that matters.** The directories are listed in `REQUIRED_READMES` in `xtask/src/check_docs.rs`, plus every `spikes/<topic>/`; the root `README.md` is the project's front page and has its own shape. Each starts with a level-1 heading and a status line (`Status: Living (<date>)`), then `## When this applies` (the situations in which a reader needs it), then `## Contents`: one bullet or table row per file or directory in it, named first in code or as a link (`m*/` covers every name it matches). `## How to` and other sections may follow. Rules live in one place and the README links to them. `cargo xtask check-docs` enforces this, along with relative links and anchors in every Markdown file.
- **Every document states its status** near the top: `Draft`, `Accepted`, `Amended`, `Implemented`, `Superseded`, `Dropped`, `Historical` (or `Template`, for a template), with a date.
- **ADRs** are `adr/NNNN-kebab-title.md` with Status, Context, Decision and Consequences. Add one for any decision that would be expensive to reverse. Do not rewrite an accepted ADR to change the decision; write a new one that supersedes it.
- **Chunk folders** are `chunks/m<N>.<n>-<slug>/`, flat (milestones are a field in the spec header, not a folder level), holding the chunk's `spec.md` and `plan.md`, copied from [`chunks/_template/`](chunks/_template/). A spec keeps stable section numbers because code and config cite them (for example "spec §4.9"); add sections, do not renumber.
- **Plans are kept.** A plan is written before implementation and records what differed in its Deviations section. Its status becomes `Historical` in the chunk's own pull request, together with the spec's `Implemented` and the roadmap row's `Done (PR #n)`: as soon as the pull request is open (which gives `n`), without waiting for its checks, a final commit closes the chunk out, so no documentation-only pull request follows the merge. A historical plan records how the work was done, and the spec, the ADRs and the code are authoritative.
- **Write the minimum; prefer a link to a copy.** Say what the reader needs and cannot see in the code; a plan does not restate its spec; no overview of what is plain from the files. If a rule is enforced by a command, document the command and let the tool be the source of truth. The documentation review flags padding.
- **Numbers and claims need provenance.** A measurement says what was measured, on what machine, in what conditions, and what was not measured.
- **Research notes** (`research/`) open with a provenance line (where the content came from and how far it was checked) and cite a source for every factual claim, or mark it unverified.
