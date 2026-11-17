# The shim (interposition layer)

Status: Reference: not a decision (2026-11-17).

> **Provenance.** Distilled on 2026-11-17 from a conversation the maintainer had with an AI model (Google Gemini) in November 2026, rewritten in our own words. Nothing here is verified unless a source is cited; the conversation cited none. Code from the conversation is not reused ([ADR 0001](../adr/0001-licence-and-originality.md)). Research chunks check these claims before a decision relies on them.

## The idea

A layer between a Ring −1 hypervisor and the exokernel (the conversation called it "Layer −0.5") that shapes what the exokernel sees of the hardware: time, faults, trust or topology. It would not be called a kernel: the usual names are **monitor** (security or reference monitor), **shim** or **VMM extension**, **interposition layer**, **paravirtualization interface**, **fabric**, and occasionally **nanokernel** or **picokernel**.

## Candidate features and the costs claimed for them

| Feature | What it does | Claimed cost (unsourced) |
|---|---|---|
| Deterministic record and replay | Logs every nondeterministic input (interrupts, DMA arrivals, timestamps, hardware random numbers) so a run can be replayed, rewound or forked | Moderate, 10-30 %, from frequent exits |
| Chaos and fault injection | Drops packets, flips bits, stalls interrupts, kills cores, to harden error paths | Not stated; debug-only |
| Capability firewall | Checks every mapping and DMA command against what the domain was granted | Not stated |
| Confidential-compute veil | Memory encryption and attestation so hypervisor and exokernel cannot read each other (as Intel TDX, AMD SEV-SNP, Arm CCA do) | Very low, 1-5 % |
| Disaggregation | Makes memory or devices on other machines (CXL, RDMA) look local (as LegoOS, Popcorn Linux) | Low to moderate, 2-15 % |
| Live mirroring | Copies CPU state and dirty pages to a standby machine (as Remus, COLO) | High, 20-50 % |
| ISA translation | Runs the exokernel for one architecture on another | Moderate to high, 15-40 % |
| Energy or thermal governor | Reshapes the cores exposed by temperature or power | Not stated |

The rule of thumb offered: cost comes from **exits** (each a few hundred to a few thousand cycles, unsourced). A layer that sets up page tables and IOMMU rules once and then talks over shared rings costs little; one that intercepts every allocation, tick or register read costs a lot. Heavy features are for debugging and should compile out.

## Proposed placement

The conversation's final proposal: not a separate binary, but a crate (`exo-shim`) compiled into the exokernel, sitting where traps, interrupts and exokernel calls cross between the hardware and the domains:

- every crossing goes through a Rust trait in the shim;
- features are Cargo features (`time-travel`, `chaos-inject`, `strict-firewall`), so disabled ones compile to nothing;
- policies are **per domain**: a stable microkernel gets the firewall and pass-through interrupts; an experimental one gets recording and fault injection;
- it works on abstract events (exokernel calls, events, frame ranges), not registers, so it is written once for all three architectures.

## Open

- If the shim lives inside the exokernel, it is an interposition point of the exokernel, not a layer between a hypervisor and the exokernel. Whether a real separate layer (for example at EL2 under an EL1 exokernel) is worth having is part of M1.5, and depends on M1.1.
- Which nondeterministic inputs exist on each architecture and under QEMU, and whether replay is possible without the virtualization extensions.

## Corrections

- All percentages above come from the conversation without sources.
