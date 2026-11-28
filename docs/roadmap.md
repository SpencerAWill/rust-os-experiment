# Roadmap

Status: Accepted (2026-11-17); amended by [M0.2](chunks/m0.2-process-sync/spec.md) (2026-11-27). The milestone goals and exit criteria, the later milestones' rough lists, and the history of the chunks up to M0.2. What is scheduled, its order and its status are on GitHub: chunk issues, [milestones](https://github.com/SpencerAWill/rust-os-experiment/milestones) and the "OS backlog" Project ([workflow.md](workflow.md#from-issue-to-chunk)).

Work is organized as **milestones** (a goal with exit criteria) broken into **chunks** (one small pull request each, with its own spec and plan in [`chunks/`](chunks/)). Only the current milestone is broken into chunks, each a GitHub issue titled with its ID; later ones carry a rough list below that becomes chunk issues when they become current. Chunk IDs (`M1.1`) are never reused or renumbered; order comes from the Project's Order field and "blocked by" links (the dependency column below is history). The process is in [workflow.md](workflow.md).

## Milestones

| Milestone | Goal | Exit criteria |
|---|---|---|
| M0 | Repository foundations | The workflow, CI, settings and research seed are merged; the development environment is checked by `cargo xtask doctor`; the process matches `rust-engine-experiment`'s (M0.2); the host-safety guardrails are in (M0.3); `cargo xtask spike` runs a spike (M0.4); `/chunk next` starts M1.1 |
| M1 | Research and architecture decisions | ADRs accepted for the privilege model, the boot path and toolchain, the test strategy and `unsafe` policy, the exokernel interface, the shim's placement and the crate layout, each backed by a research note in `research/` |
| M2 | Boot skeleton | One binary boots under QEMU on x86_64, aarch64 and riscv64, prints over serial, and is tested headless in CI |
| M3 | Exokernel core | Physical memory, capabilities and domains, tested on all three architectures |
| M4 | Two microkernels side by side | Two different microkernels or library OSes run at once on the exokernel and exchange a typed message (the first target milestone) |
| M5 | Shim | The first interposition features at the trap boundary, enabled per domain |

Milestones are GitHub [milestones](https://github.com/SpencerAWill/rust-os-experiment/milestones), which hold their issues and progress; the current one is the open milestone with the lowest number.

## M0 chunks

This table is frozen at M0.2: later chunks, M0.3 and M0.4 included, are GitHub issues in the [milestones](https://github.com/SpencerAWill/rust-os-experiment/milestones), ordered in the Project. Row statuses were `Planned`, `Drafting`, `Accepted`, `In progress`, `Done (PR #n)` and `Dropped (reason)`.

| Chunk | Title | Depends on | Status | Spec and plan |
|---|---|---|---|---|
| M0.0 | Repository foundations: the chunk workflow with build and research chunks, agent roles and skills, the push guard, `cargo xtask` process checks, CI, Pulumi-managed settings, the issue Project, ADRs 0001-0002, research notes from the initial architecture conversation | none | Done (PR #1) | [spec](chunks/m0.0-foundations/spec.md), [plan](chunks/m0.0-foundations/plan.md) |
| M0.1 | Development environment: host tools and Rust targets, `cargo xtask doctor`, a local reference library of specifications, and the tool survey as a research note | M0.0 | Done (PR #11) | [spec](chunks/m0.1-dev-environment/spec.md), [plan](chunks/m0.1-dev-environment/plan.md) |
| M0.2 | Process sync with rust-engine-experiment: planning state on GitHub (chunk issues, milestones, the Project's Order field), chores, the pause after the plan, the newer builder, simplify, re-verify and grill rules, and the engine's general lessons | M0.1 | In progress | [spec](chunks/m0.2-process-sync/spec.md) |

## Later milestones

Rough lists, refined when each becomes current: then its list becomes chunk issues in its GitHub milestone ([maintainers.md](maintainers.md#when-a-milestone-completes)).

| Milestone | Rough chunks | Open question |
|---|---|---|
| M2 | `cargo xtask build/run --arch`; serial output and panic handler per architecture; headless QEMU boot tests in CI | Which QEMU machines; whether CI runs all three architectures on every pull request |
| M3 | Frame allocator; page tables per architecture; capability table; domain creation; revocation | From M1.4 |
| M4 | Domain loader; scheduling between domains; two personalities; the typed message | Which two microkernels or library OSes |
| M5 | Interposition trait at the trap boundary; first features (from M1.5) | From M1.5 |
| Later | Compatibility adapters (WASI first, per the research); drivers beyond VirtIO; real hardware; fuzzing, coverage and mutation testing of host-testable code | Unnumbered until scheduled |
