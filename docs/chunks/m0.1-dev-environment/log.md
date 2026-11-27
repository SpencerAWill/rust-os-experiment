# M0.1 log: Development environment

- **Status:** Living (2026-11-27).

- 2026-11-27 decision The maintainer named this chunk M0.1 (not M1): M0 reopens as Current, M1 becomes Next. Scope chosen: toolchain components and targets plus a Brewfile, `cargo xtask doctor`, a local reference library of specifications, and the tool survey as a research note.
- 2026-11-27 surprise The UTM cask ships QEMU only as frameworks inside `UTM.app`; no `qemu-system-*` was on `PATH` until `brew install qemu` (11.1.3).
- 2026-11-27 surprise The Claude Code rust-analyzer LSP plugin crashed (exit 1) because the pinned 1.100.0 toolchain had no `rust-analyzer` component; after `rustup component add rust-analyzer --toolchain 1.100.0` the LSP returns symbols.
- 2026-11-27 decision The maintainer asked for the host tools to be installed now, before the spec: `brew install qemu dtc socat poppler acpica x86_64-elf-gdb aarch64-elf-gdb riscv64-elf-gdb`; `rustup component add rust-analyzer llvm-tools` and `rustup target add x86_64-unknown-none aarch64-unknown-none riscv64gc-unknown-none-elf` for 1.100.0. Disk space was freed first (Docker pruned, 20 to 64 GiB free).
- 2026-11-27 surprise `socat`, `pdftotext` and `iasl` reject `--version` (they use `-V`, `-v`, `-v`), so `cmd::require_tool` cannot check them as written.
- 2026-11-27 done Smoke boots, QEMU 11.1.3, 6 s each, serial captured: `qemu-system-riscv64 -M virt -nographic -bios default` prints `OpenSBI v1.8.1`; `qemu-system-aarch64 -M virt -accel hvf -cpu host -bios edk2-aarch64-code.fd` prints `UEFI firmware (version edk2-stable202408-prebuilt.qemu.org ...)`; `qemu-system-x86_64` (default) prints `SeaBIOS (version rel-1.17.0-...)`; x86_64 q35 and riscv64 virt with `-drive if=pflash,format=raw,readonly=on,file=edk2-*-code.fd` reach `UEFI Interactive Shell v2.2` and `UEFI v2.70 (EDK II, ...)`. `-accel help`: aarch64 has `hvf tcg`, x86_64 has `tcg` only.
- 2026-11-27 surprise `qemu-system-x86_64 -bios /opt/homebrew/share/qemu/edk2-x86_64-code.fd` (the command in the seed transcript) fails with `could not load PC BIOS`; the x86 EDK2 image must be given as pflash. A correction for `docs/research/toolchain-and-boot.md`.
- 2026-11-27 surprise `-M virt,virtualization=on -accel hvf` is accepted on this M4 Pro (macOS 26.6.2) and EDK2 boots; whether the guest is entered at EL2 is not checked here (M1.1's spike).
- 2026-11-27 decision The maintainer has no x86 Linux hardware: x86 VT-x and KVM evidence must come from emulators or CI runners (matters for M1.1).
