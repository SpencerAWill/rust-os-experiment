# Realities checklist

Status: Reference: not a decision (2026-11-17).

> **Provenance.** Distilled on 2026-11-17 from a conversation the maintainer had with an AI model (Google Gemini) in November 2026 ([transcript](sources/gemini-architecture-conversation.txt)), rewritten in our own words. Nothing here is verified unless a source is cited; the conversation cited none. Code from the conversation is not reused ([ADR 0001](../adr/0001-licence-and-originality.md)). Research chunks check these claims before a decision relies on them.

Problems that commonly stall OS projects. A spec's §6 should say which ones it touches.

- **Drivers.** Under QEMU only VirtIO is needed. Real hardware needs a strategy: port drivers, reuse them through a shim (NetBSD rump kernels, DDEKit), or run Linux in a driver domain (Qubes, Xen).
- **Rust `std` and the ecosystem.** Everything starts `no_std`. Running ordinary crates needs a `std` port for this OS, or a WebAssembly runtime as the first user-space format ([compatibility](compatibility.md)).
- **Build and test.** Several binaries for several targets need the xtask pattern and headless QEMU tests in CI from the start.
- **`unsafe`.** Confine it to small, reviewed modules; check what can run on the host with Miri, Kani or Verus (ADR 0002 is the placeholder).
- **Memory ordering and coherence.** Lock-free rings between domains need correct acquire and release ordering; DMA can bypass caches, so mappings need the right cache attributes; priority inversion around shared slots.
- **Time.** Timestamp counters can drift between cores or change with frequency; calibrate against a stable source (HPET, ACPI PM timer, the Arm generic timer).
- **Power and heat.** Without idle states and frequency scaling, every core runs flat out.
- **Display and input.** Framebuffer, compositor or GPU submission; fonts and Unicode; keyboard and USB HID.
- **Crashes and persistence.** Journaling or copy-on-write for any file system; memory quotas and revocation so one domain cannot starve another; a crash log that survives a warm reboot.
- **Security.** Speculative-execution mitigations at every privilege switch; entropy at boot (RDRAND/RDSEED, jitter, VirtIO-RNG); signed images.
- **Debugging.** Serial first, unwinding to file and line, a GDB stub; a triple fault resets without a message.
- **AI-specific.** Models invent register bit offsets and mix old BIOS behaviour with UEFI; keep registers in typed definitions cited to the manual, and pin the toolchain and QEMU flags.
- **Scope.** Keep milestones small and visible (text on screen, a keypress, a second task) rather than designing every layer first; define "done" for each milestone.
