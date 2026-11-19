# Toolchain, boot and test environment

Status: Reference: not a decision (2026-11-17).

> **Provenance.** Distilled on 2026-11-17 from a conversation the maintainer had with an AI model (Google Gemini) in November 2026 ([transcript](sources/gemini-architecture-conversation.txt)), rewritten in our own words. Nothing here is verified unless a source is cited; the conversation cited none. Code from the conversation is not reused ([ADR 0001](../adr/0001-licence-and-originality.md)). Research chunks check these claims before a decision relies on them.

## Architectures and what the compiler does not cover

Targets: x86_64, aarch64 and riscv64. Rust's bare-metal targets (`x86_64-unknown-none`, `aarch64-unknown-none`, `riscv64gc-unknown-none-elf`) generate the instructions, but the ISA manuals are still needed for everything the compiler does not do: privilege levels and their registers, page-table formats, exception and interrupt entry, timers, interrupt controllers (APIC, GIC, PLIC/AIA), the system-call or trap instruction and its register conventions, and memory ordering. Also needed: the boot protocol, ACPI (x86, some aarch64) or device trees (aarch64, riscv64), UEFI, PCIe, VirtIO, and the RISC-V SBI.

## Bootloader

| Option | As described |
|---|---|
| Limine | A separate bootloader with one protocol on x86_64, aarch64 and riscv64 (and more): hands over a memory map, framebuffer, CPU list and modules (other ELF files to load, such as the domains' kernels) |
| The `bootloader` crate | Rust, x86_64 only, builds a disk image with the kernel |
| Our own | Possible, but a project of its own; most hobby kernels do not |

Bootloaders range from a 512-byte boot sector to tens of thousands of lines for UEFI loaders (unsourced). The conversation recommended Limine for three architectures. To verify in M1.2: Limine's licence, its architectures and their firmware requirements (UEFI on aarch64 and riscv64 under QEMU), and the stable-or-nightly question for our targets.

## How far down Rust reaches

All the software layers can be Rust: user space, microkernels, the exokernel, a hypervisor (VT-x, AMD-V, EL2, the H extension), and firmware (open-source UEFI work, oreboot, RustSBI). It stops at CPU microcode, which is vendor-signed. Each layer still needs a little inline assembly (context switches, trap entry) and linker scripts. A shared `#[repr(C)]` ABI crate imported by every layer lets the compiler catch layout mismatches between them.

## Testing on the M4 Pro

- QEMU runs all three architectures; aarch64 guests can use Apple's Hypervisor framework (`-accel hvf`) for near-native speed, the other two run under emulation (TCG). UTM is a front end to the same.
- Nothing touches the Mac's own boot; a crashed kernel only resets the VM.
- The usual pattern is `cargo xtask build/run --arch <arch>`: compile each binary for its target, build the boot image, start QEMU with the right flags. Headless tests exit QEMU with a status through a debug-exit device (`isa-debug-exit` on x86) or semihosting, so CI can run them.
- Debugging: serial logging from the first instruction, a panic handler with file and line, and QEMU's GDB stub.
- To verify in M1.1: whether QEMU with HVF exposes EL2 (nested virtualization) to a guest on an M4 Pro, which matters if the exokernel uses the virtualization extensions. This is a spike candidate.

## Corrections

- The conversation's sizes and times (bootloader sizes, "days instead of years" for VirtIO drivers) are unsourced.
