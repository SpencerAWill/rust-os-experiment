# Roadmap

Status: Accepted (2026-11-17). The source of truth for what is planned and how far along it is.

Work is organized as **milestones** (a goal with exit criteria) broken into **chunks** (one small pull request each, with its own spec and plan in [`chunks/`](chunks/)). Only the current milestone is broken into chunks; later ones carry a rough list that is refined when they become current. Chunk IDs (`M1.1`) are never reused or renumbered; order comes from the dependency column. The process is in [workflow.md](workflow.md).

Row statuses: `Planned`, `Drafting`, `Accepted`, `In progress`, `Done (PR #n)`, `Dropped (reason)`. `Drafting` to `In progress` are set on the chunk's branch, so on `main` a row normally goes from `Planned` straight to `Done`.

## Milestones

| Milestone | Goal | Exit criteria | Status |
|---|---|---|---|
| M0 | Repository foundations | The workflow, CI, settings and research seed are merged; `/chunk next` starts M1.1 | Done |
| M1 | Research and architecture decisions | ADRs accepted for the privilege model, the boot path and toolchain, the test strategy and `unsafe` policy, the exokernel interface, the shim's placement and the crate layout, each backed by a research note in `research/` | Current |
| M2 | Boot skeleton | One binary boots under QEMU on x86_64, aarch64 and riscv64, prints over serial, and is tested headless in CI | Later |
| M3 | Exokernel core | Physical memory, capabilities and domains, tested on all three architectures | Later |
| M4 | Two microkernels side by side | Two different microkernels or library OSes run at once on the exokernel and exchange a typed message (the first target milestone) | Later |
| M5 | Shim | The first interposition features at the trap boundary, enabled per domain | Later |

## M0 chunks

| Chunk | Title | Depends on | Status | Spec and plan |
|---|---|---|---|---|
| M0.0 | Repository foundations: the chunk workflow with build and research chunks, agent roles and skills, the push guard, `cargo xtask` process checks, CI, Pulumi-managed settings, the issue Project, ADRs 0001-0002, research notes from the initial architecture conversation | none | Done (PR #1) | [spec](chunks/m0.0-foundations/spec.md), [plan](chunks/m0.0-foundations/plan.md) |

## M1 chunks (research)

Each is a research chunk ([workflow](workflow.md#research-chunks)): a note in `research/`, and an ADR where it decides. The questions are in [research/open-questions.md](research/open-questions.md).

| Chunk | Title | Depends on | Status | Spec and plan |
|---|---|---|---|---|
| M1.1 | Privilege model: where the exokernel and the microkernels run on each architecture (exokernel in the kernel mode with user-mode library OSes, or an exokernel on the virtualization extensions with microkernels in guest kernel mode), and what QEMU on an M4 Pro supports for each | none | Planned | |
| M1.2 | Boot path and toolchain: bootloader, target triples, stable or nightly, linker scripts, image building | none | Planned | |
| M1.3 | Test and debug strategy (host tests, QEMU in CI, GDB, serial) and the `unsafe` policy, superseding ADR 0002 | M1.2 | Planned | |
| M1.4 | Exokernel interface: resources, capabilities, the call and event ABI, revocation | M1.1 | Planned | |
| M1.5 | Shim placement and its first features | M1.1, M1.4 | Planned | |
| M1.6 | Crate and workspace layout for several binaries and targets | M1.1, M1.2 | Planned | |

## Later milestones

Rough lists, refined when each becomes current.

| Milestone | Rough chunks | Open question |
|---|---|---|
| M2 | `cargo xtask build/run --arch`; serial output and panic handler per architecture; headless QEMU boot tests in CI | Which QEMU machines; whether CI runs all three architectures on every pull request |
| M3 | Frame allocator; page tables per architecture; capability table; domain creation; revocation | From M1.4 |
| M4 | Domain loader; scheduling between domains; two personalities; the typed message | Which two microkernels or library OSes |
| M5 | Interposition trait at the trap boundary; first features (from M1.5) | From M1.5 |
| Later | Compatibility adapters (WASI first, per the research); drivers beyond VirtIO; real hardware; fuzzing, coverage and mutation testing of host-testable code | Unnumbered until scheduled |
