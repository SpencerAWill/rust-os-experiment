# Architecture landscape

Status: Reference: not a decision (2026-11-17).

> **Provenance.** Distilled on 2026-11-17 from a conversation the maintainer had with an AI model (Google Gemini) in November 2026 ([transcript](sources/gemini-architecture-conversation.txt)), rewritten in our own words. Nothing here is verified unless a source is cited; the conversation cited none. Code from the conversation is not reused ([ADR 0001](../adr/0001-licence-and-originality.md)). Research chunks check these claims before a decision relies on them.

## Why not just Linux

Complaints often made about Linux development, and the opening they leave for a new design: decades of compatibility promises ("never break userspace") that keep old interfaces (`ioctl`, signals, terminals) forever; a monolithic kernel where any driver bug runs with full privilege; a large C codebase; blocking system calls, with async I/O (`epoll`, `io_uring`) added later. Hardware specifications (PCIe, NVMe, USB, ISA manuals) still bind any OS that runs on real machines.

## Design ideas surveyed

| Area | Idea | Notes |
|---|---|---|
| Security | Capabilities, no ambient authority, no root | A process starts with nothing; its parent grants unforgeable handles (seL4, Fuchsia) |
| Interface | Async-first calls over shared submission/completion rings | One mechanism for system calls, IPC and interrupts |
| Interface | Typed IPC instead of byte streams | Messages with a shared Rust definition |
| Isolation | Software-isolated processes, or a single address space (SASOS) | Relies on the compiler instead of the MMU; only works for trusted, compiler-checked code |
| Scheduling | Thread-per-core, share-nothing (multikernel) | Cores talk only by message passing (Barrelfish) |
| Modularity | Live-swappable components | Needs strict state boundaries and no global state (Theseus) |
| Drivers | User-space drivers behind the IOMMU | |
| Storage | Object or database file system; single-level store; content-addressed immutable system state | |
| Networking | Kernel bypass and zero-copy DMA rings by default; location-transparent IPC (Plan 9, Amoeba) | |
| Experience | Structured shell; deterministic record and replay | |

## Ideas that conflict

- Single address space versus hardware isolation: one relies on the compiler, the other on page tables.
- Compiler-only isolation versus running arbitrary software (C, WebAssembly, binary drivers).
- A persistent single-level store versus immutable content-addressed state.
- Thread-per-core versus one global scheduler balancing mixed work.

The conversation's conclusion: one OS can host all of them only as separate **execution domains** over a tiny core, and even the tiny core must pick its most basic rule (hardware isolation or not, shared-memory SMP or multikernel, swappable or not). This is the *mechanism versus policy* split.

## Microkernel or exokernel

- A **microkernel** keeps abstractions (threads, IPC, address spaces) in the kernel and moves servers out. The conversation sketched six core modules: boot and architecture glue, memory (frame allocator, page tables, a small kernel heap), capabilities, per-core scheduling, ring IPC, and a loader for the first user program.
- An **exokernel** (MIT, 1990s) only multiplexes raw resources securely: frames, disk blocks, NIC queues, CPU slices. Library OSes linked into applications implement the abstractions. Its mechanisms: secure bindings, *visible* revocation (the library OS is asked to give resources back), downloaded code checked at interrupt time, coarse CPU slices scheduled further by the library OS.
- Trade-off as described: an exokernel is smaller and lets each application bring its own policy, but sharing one device or file system between library OSes tends to need a shared server, which converges back towards a microkernel.
- **A microkernel on an exokernel** (the maintainer's question): the exokernel partitions the machine into domains; a microkernel runs in one domain as its library OS; applications run on the microkernel. A second, experimental kernel can run in another domain at the same time.

## Exokernel versus a Ring −1 hypervisor

| | Exokernel | Type-1 hypervisor |
|---|---|---|
| Aim | Expose real hardware, checked | Present a virtual machine to unmodified OSes |
| Privilege | Kernel mode (ring 0 / EL1 / S-mode) | Hypervisor mode (VMX root / EL2 / HS-mode) |
| Guest knows | Yes, it calls the exokernel | No (unless paravirtualized) |
| Cost | No trap-and-emulate | VM exits on sensitive operations |

They blur: paravirtualized hypervisors with device pass-through and virtio behave much like exokernels, and Dune ran a library OS on the virtualization hardware. Running Linux and Windows beside our own stack needs a hypervisor below everything; our exokernel would then be one guest. See [open question 1](open-questions.md) for the contradiction this raises.

## Layers or planes

"Layers" fits only privilege (who can preempt or revoke whom) and boot order. At run time a better model is a **control plane** (hypervisor, monitor, exokernel set up mappings and permissions once) and a **data plane** (components talk directly over shared rings and pass-through devices), or a **capability graph** where any component can hold a handle to any resource, or **concentric trust rings**. Features such as replay or fault injection are compile-time features at the trap boundary, not runtime layers.

## Corrections

- The conversation claims the exokernel needs no virtualization *and* hosts microkernels; a microkernel needs privileged control (page tables, interrupts), so either the "microkernels" are user-mode library OSes that ask the exokernel for every mapping, or the exokernel uses the virtualization extensions. M1.1 decides.
- Its line counts (exokernel 2,000-5,000 lines, microkernel 5,000-15,000) and the "under 1-2 %" layering overhead have no source.
