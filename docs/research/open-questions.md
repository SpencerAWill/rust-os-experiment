# Open questions

Status: Reference: not a decision (2026-11-17). The questions M1 answers, most blocking first. Each maps to a research chunk on the [roadmap](../roadmap.md).

| # | Question | Blocks | Chunk |
|---|---|---|---|
| 1 | **Where does each part run?** Exokernel in kernel mode (ring 0, EL1, S-mode) with the "microkernels" as user-mode library OSes that ask it for every mapping; or exokernel on the virtualization extensions (VMX root, EL2, HS-mode) with microkernels in guest kernel mode. For each: what each architecture offers, what QEMU on an M4 Pro supports (HVF and nested virtualization; TCG for x86_64 and riscv64), and what a microkernel can then still do itself | Everything below | M1.1 |
| 2 | **Boot path and toolchain.** Limine or another bootloader (licence, architectures, firmware needed under QEMU); target triples; stable or nightly; linker scripts; how images are built | M2 | M1.2 |
| 3 | **How kernel code is tested and debugged**, and the **`unsafe` policy**: what runs on the host, what under QEMU in CI, which QEMU machines, GDB and serial; where `unsafe` may live and how it is checked | M2, all code | M1.3 |
| 4 | **The exokernel interface.** Which resources it multiplexes (frames, cores or time slices, interrupts, devices and DMA), how capabilities are represented and revoked, the call ABI per architecture, the shared event page or ring | M3 | M1.4 |
| 5 | **The shim.** Inside the exokernel at the trap boundary, or a separate layer below it; which first feature (replay, fault injection, firewall) and what it needs from questions 1 and 4 | M5 | M1.5 |
| 6 | **Crate layout.** How the exokernel, the shared ABI, two kernels, the shim and spikes are organized for several binaries and targets | M2 | M1.6 |

Questions deliberately later: drivers on real hardware, the `std` port, compatibility adapters ([compatibility.md](compatibility.md)), and performance comparisons with Linux (the conversation's speedup estimates, 2x to 100x for I/O and latency-bound work, have no source and are not recorded as expectations).
