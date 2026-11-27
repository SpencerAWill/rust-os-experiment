# Development environment: host tools, QEMU on the M4 Pro, and ground truth

Status: Draft (2026-11-27). Reference: not a decision.

> **Provenance.** Written on 2026-11-27 by a `researcher` agent for M0.1 ([spec §4.5](../chunks/m0.1-dev-environment/spec.md#45-the-tool-survey-note), topics 1-3). "Measured here: `<command>`" means the command was run on the maintainer's machine (macOS 26.6.2 build 25G83, Apple M4 Pro, Homebrew 7.0.17) and its output is in the chunk's [log.md](../chunks/m0.1-dev-environment/log.md) under 2026-11-27. Every other claim cites the publisher's or project's own page, read on 2026-11-27, or a document in [`reference/`](../../reference/README.md) by id and version. Nothing is taken from the seed conversation. Claims that could not be confirmed say "unverified" and what was tried. The note records options for M1.1 and M1.2; it decides nothing.

## 1. Host tools

The machine: macOS 26.6.2, Apple M4 Pro, Homebrew 7.0.17 (measured here: `sw_vers`, `sysctl -n machdep.cpu.brand_string`, `brew --version`).

| Tool (formula) | What it is, per Homebrew | Used here for | Version | Licence, per Homebrew |
|---|---|---|---|---|
| `qemu-system-x86_64`, `-aarch64`, `-riscv64` (`qemu`) | "Generic machine emulator and virtualizer" | running each architecture's firmware and, later, the kernels | 11.1.3 | GPL-2.0-only |
| `dtc` (`dtc`) | "Device tree compiler" | reading and writing QEMU's device trees | 1.8.1 | GPL-2.0-or-later OR BSD-2-Clause |
| `socat` (`socat`) | "SOcket CAT: netcat on steroids" | QEMU's serial, monitor and QMP sockets | 1.8.1.3 | GPL-2.0-only |
| `pdftotext` (`poppler`) | "PDF rendering library (based on the xpdf-3.0 code base)" | the reference library's text | 26.11.0 | GPL-2.0-only OR GPL-3.0-only |
| `iasl` (`acpica`) | "OS-independent implementation of the ACPI specification" | disassembling and compiling ACPI tables | 20260408 | Intel-ACPI OR GPL-2.0-only OR BSD-3-Clause |
| `x86_64-elf-gdb`, `aarch64-elf-gdb`, `riscv64-elf-gdb` | "GNU debugger for <triple> cross development" | QEMU's gdbstub | 18.1 each | GPL-3.0-or-later |

- Descriptions and licences: measured here, `brew info --json=v2 <formulae>`. The "used here for" column is this project's intent, from the [`Brewfile`](../../Brewfile) comments, not a property of the tools.
- Versions: measured here, `brew list --versions` and each tool's own version argument. `socat`, `pdftotext` and `iasl` reject `--version` and take `-V`, `-v` and `-v` (measured here); `iasl` prints a dotless date, `version 20260408` (measured here: `iasl -v`).
- Rust: `rustc 1.100.0 (0ef1bc386 2026-11-09)` (measured here: `rustc --version`), with the components and targets of [`rust-toolchain.toml`](../../rust-toolchain.toml) installed (measured here: `rustup component list --installed`, `rustup target list --installed`, log of step 1).
- `curl` is macOS's own `/usr/bin/curl`, 8.7.1 with LibreSSL 3.3.6; Homebrew has no `curl` installed (measured here: `command -v curl`, `curl --version`, `brew list --versions curl`).
- The UTM cask does not put QEMU's command-line tools on `PATH`; `brew install qemu` does (measured here, log line on the UTM cask).

## 2. QEMU on this Mac

### Firmware and smoke boots

QEMU 11.1.3 searches `/opt/homebrew/Cellar/qemu/11.1.3/bin/../share/qemu-firmware`, which does not exist, and `.../share/qemu`, which holds `edk2-x86_64-code.fd`, `edk2-aarch64-code.fd`, `edk2-riscv-code.fd` and `opensbi-riscv64-generic-fw_dynamic.bin` (measured here: `qemu-system-x86_64 -L help`, `ls /opt/homebrew/share/qemu`). Each boot below printed its marker on serial (measured here; the commands are [spec §4.3](../chunks/m0.1-dev-environment/spec.md#43-cargo-xtask-doctor)'s, and `cargo xtask doctor` ran all five with `ok`, log of step 3):

| Boot | Firmware and version seen on serial | Time to marker (doctor, step 3) |
|---|---|---|
| `x86_64-bios` | SeaBIOS `rel-1.17.0` | 0.0 s |
| `x86_64-uefi` | EDK2 `UEFI Interactive Shell v2.2` | 1.1 s |
| `aarch64-uefi` (HVF) | EDK2 `edk2-stable202408-prebuilt.qemu.org` | 0.0 s |
| `riscv64-opensbi` | OpenSBI v1.8.1 | 0.0 s |
| `riscv64-uefi` | EDK2 `UEFI v2.70` | 6.0 s |

### The x86 UEFI image must be pflash

- `qemu-system-x86_64 -bios /opt/homebrew/share/qemu/edk2-x86_64-code.fd` exits 1 with `qemu: could not load PC BIOS '...edk2-x86_64-code.fd'` (measured here, twice: the log line on the seed transcript's command, and step 7).
- The same image given as `-M q35 -drive if=pflash,format=raw,readonly=on,file=...` reaches the UEFI shell (measured here). The aarch64 image does boot with `-bios` (measured here). The correction to `toolchain-and-boot.md` is step 8's.

### Accelerators

- `qemu-system-aarch64 -accel help` lists `hvf` and `tcg`; `qemu-system-x86_64` and `qemu-system-riscv64` list `tcg` only (measured here). On this Mac, x86_64 and riscv64 guests are emulated, never accelerated.
- `-accel kvm` fails with `invalid accelerator kvm` (measured here).

### HVF nested virtualisation (EL2)

What Apple documents:

- Apple's Hypervisor framework has `hv_vm_config_get_el2_supported`, which "Returns a status value that indicates whether the current platform supports Exception Level 2 (EL2)", and `hv_vm_config_set_el2_enabled`, both introduced in macOS 15.0 ([Apple, hv_vm_config_get_el2_supported](https://developer.apple.com/documentation/hypervisor/hv_vm_config_get_el2_supported(_:)); [hv_vm_config_set_el2_enabled](https://developer.apple.com/documentation/hypervisor/hv_vm_config_set_el2_enabled(_:_:)); read through the pages' JSON).
- Neither page names the chips that support EL2 (same pages); a Linaro engineer made the same observation on qemu-devel and assumed M4 ([Alex Bennée, 2025-07-15](https://mail-archive.com/qemu-devel@nongnu.org/msg1127523.html)). The "M3 or later" requirement that third parties state is unverified: it appeared only in search-result summaries of a mailing-list reply and a UTM issue, not on a page of Apple's that was read.
- The framework does not give vCPUs a reset state; the entry level is set through `HV_REG_CPSR`'s M field ([Apple Developer Forums 775546](https://developer.apple.com/forums/thread/775546), the accepted answer by an Apple engineer).

What QEMU does:

- Mohamed Mediouni's series "HVF: Add support for platform vGIC and nested virtualisation" reached v21 on 2026-04-29 ([cover letter](https://patchew.org/QEMU/20260429190532.26538-1-mohamed@unpredictable.fr/)). Peter Maydell applied it to `target-arm.next` (same page, his reply).
- The commit "hw/arm, target/arm: nested virtualisation on HVF" ([d7a75ea](https://github.com/qemu/qemu/commit/d7a75ea483a57add78b559cd181293e0136cc8c8), 2026-05-05) is in v11.1.0 and not in v11.0.0 (measured here: `gh api repos/qemu/qemu/compare/d7a75ea...v11.1.0` gives `ahead`, against v11.0.0 `behind`). It queries `hv_vm_config_get_el2_supported` and fails with "Nested virtualization not supported on this system." when the answer is no (same commit, `target/arm/hvf/hvf.c`).
- The platform vGIC is the default for `virt-11.1` and later ([37863ff](https://github.com/qemu/qemu/commit/37863fff59e0), "hvf: arm: enable vGIC by default for virt-11.1 and later").

Limits reported in the v21 cover letter ([patchew](https://patchew.org/QEMU/20260429190532.26538-1-mohamed@unpredictable.fr/)):

- "the nested virtualisation implementation shipping as of macOS 26.3 is nVHE only (but it _does_ use VNCR as shipped)". Whether macOS 26.6 changed this is unverified.
- When the guest is at EL2 as the virtual timer fires, the interrupt is not delivered back (without the platform vGIC the exit does not fire; with it, the interrupt is lost). The reference ArmVirtQemu EDK2 build uses the virtual timer at EL2, "so it gets broken unless this patch is applied to edk2", which switches its generic-timer counter library from the virtual to the physical one.
- "when nested virt is enabled, no UI response within EDK2 and a permanent wait. Workaround: -boot menu=on,splash-time=0", filed with Apple as FB21649319.
- The series disables the Scalable Matrix Extension when nested virtualisation is active (patch 12 of 15, by its title; [patchew](https://patchew.org/QEMU/20260429190532.26538-1-mohamed@unpredictable.fr/)).

What was measured on this M4 Pro (QEMU 11.1.3, macOS 26.6.2):

- `-M virt,virtualization=on -accel hvf -cpu host` starts without an error (measured here). Given the check in d7a75ea above, this suggests the framework reports EL2 support on this chip; that is an inference, not a reading of the call's result.
- With it, EDK2 prints its banner and then nothing for 30 s, never reaching the boot manager; without `virtualization=on` the shell appears within 8 s (measured here). With `-boot menu=on,splash-time=0` added, the shell appears within 20 s (measured here), matching the cover letter's known issue.
- Under TCG, `-M virt,virtualization=on -cpu max` reaches the shell (measured here).
- Whether the firmware actually ran at EL2 was not checked; that is M1.1's spike.
- GitHub's macOS arm64 runners do not support nested virtualisation, "due to the limitation of Apple's Virtualization Framework" ([GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)), so this evidence cannot come from CI.

### The x86 VT-x gap

The maintainer has no x86 Linux hardware, so VT-x and KVM evidence must come from emulators or CI runners (log, 2026-11-27 decision). VT-x is the VMX extension, which software detects with `CPUID.01H:ECX.VMX[5] = 1` (`intel-sdm` 325462-093, §26.6 "Discovering Support for VMX", in `reference/text/intel-sdm.txt`). The options:

| Option | What is known | Not known |
|---|---|---|
| QEMU TCG | `-cpu max,+vmx` warns "TCG doesn't support requested feature: CPUID[eax=01h].ECX.vmx [bit 5]" and boots without it; `-cpu max,+svm` boots with no warning (measured here) | whether an SVM (AMD-V) hypervisor actually runs correctly under TCG: unverified, not tested |
| Bochs | 3.1 released 2026-08-23 ([release](https://github.com/bochs-emu/Bochs/releases/tag/REL_3_1_FINAL)); `configure` takes `--enable-vmx=[no|1|2]`, level 2 needs x86-64 and CPU level 6, and `--enable-svm` separately ([configure.ac at REL_3_1_FINAL](https://github.com/bochs-emu/Bochs/blob/REL_3_1_FINAL/bochs/configure.ac)); Homebrew's `bochs` 3.1 (LGPL-2.0-or-later) is built with `--enable-vmx=2` and `--enable-x86-64`, without `--enable-svm` ([formula at 117ce69](https://github.com/Homebrew/homebrew-core/blob/117ce6910070ac7d1b23941c7a6e439b7f22c1e0/Formula/b/bochs.rb)); changes since 3.1 list VMX and SVM bug fixes ([CHANGES](https://github.com/bochs-emu/Bochs/blob/master/bochs/CHANGES)) | which VMX features (EPT, VPID, unrestricted guest) level 2 covers: unverified; Bochs is not installed or run here |
| CI runners with KVM | GitHub-hosted Linux runners "support hardware acceleration for Android SDK tools" ([runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)); since 2024-04-02 this includes 2-vCPU runners, after a udev rule that opens the `kvm` device to the runner user ([changelog](https://github.blog/changelog/2024-04-02-github-actions-hardware-accelerated-android-virtualization-now-available/)) | whether the runners' CPUs are Intel (VT-x) or AMD (SVM), and whether KVM offers nested virtualisation to a guest there: unverified; a job reading `/dev/kvm`, the `vmx`/`svm` flags in `/proc/cpuinfo` and `/sys/module/kvm_intel/parameters/nested` would settle it |

For comparison, RISC-V's hypervisor extension is emulated: QEMU's default `virt` CPU reports `rv64imafdch` to OpenSBI (measured here), and `h` is the extension of `riscv-isa-manual` (release 20261127), Volume II, Chapter 5 "H Extension for Hypervisor Support" (`reference/text/riscv-isa-manual.txt`).

## 3. Ground-truth sources

### The reference library

- [`reference/manifest.toml`](../../reference/manifest.toml) pins 21 documents: 13 fetched by `cargo xtask refs --fetch` with a checksum, 8 `manual` (the manifest; log of step 6). Locally 14 are present (`ok`) and 7 are `missing`: `amd-apm`, `arm-arm`, `arm-gic`, `arm-psci`, `arm-smccc`, `arm-aarchmrs` and `sysv-gabi` (measured here: `cargo xtask refs`).
- Every Arm architecture document is missing, because developer.arm.com cannot be read by a script: with a browser User-Agent it answers HTTP 403 "Access Denied", without one it serves a script-only redirector page (measured here: `curl`; log of step 6). Until the maintainer downloads them, an EL2 or GIC register claim cannot be checked in `reference/` and needs another source or "unverified". Only `arm-aapcs64` is local, from Arm's GitHub releases (manifest).
- Each entry's `licence` field quotes what its publisher states; they range from read-only (UEFI, ACPI) and no-derivatives (Intel SDM) to CC BY 4.0 (the RISC-V documents), Apache-2.0 (Devicetree) and BSD (Limine, riscv-opcodes) ([manifest](../../reference/manifest.toml)). Searching and citing is described in [`reference/README.md`](../../reference/README.md#citing-a-specification).

### Machine-readable specifications

| Source | What it is | State | Licence |
|---|---|---|---|
| Arm AARCHMRS (A-profile, open-source package) | `Features.json`, `Instructions.json` (A64, A32, T32) and `Registers.json` (AArch32, AArch64, memory-mapped), with a JSON schema; "a subset of the information provided in the packages with Arm proprietary licenses", omitting "all descriptive content" ([README in the package](https://kernel.googlesource.com/pub/scm/linux/kernel/git/maz/AARCHMRS/+/1346b16d47619400448f3cb6d4f22d62d3b76848/README.md)) | release 2026-09 in that mirror (commit "AARCHMRS_OPENSOURCE_A_profile_FAT-2026-09", 2026-10-15); "The schema is still under development and is subject to change" (same README) | `docs/notice.html` in the package is the BSD 3-clause text, "Copyright 2010-2026 Arm Limited or its affiliates" ([notice](https://kernel.googlesource.com/pub/scm/linux/kernel/git/maz/AARCHMRS/+/1346b16d47619400448f3cb6d4f22d62d3b76848/docs/notice.html)). Read in a mirror on kernel.googlesource.com, not from Arm: Arm's download page could not be read (above), so the licence is unverified at the publisher |
| `riscv-opcodes` | enumerates "standard RISC-V instruction opcodes and control and status registers" and generates C, Scala and LaTeX; Spike, PK and the ISA manual use its artifacts (README in the local archive, `reference/files/riscv-opcodes.tar.gz`) | no tags; pinned at commit 0a7bd769 (2026-11-25) ([manifest](../../reference/manifest.toml)) | BSD-3-Clause ([manifest](../../reference/manifest.toml), from its LICENSE) |
| RISC-V Unified Database (UDB) | aims to hold "*all* the information needed to describe RISC-V" and generate documents, toolchain inputs and simulators ([README at af3eaf9](https://github.com/riscv/riscv-unified-db/blob/af3eaf91a20d19d6d4ead9110f1ba3fb56a8035c/README.adoc)) | "under rapid development. Expect schemas and APIs to change"; its `spec` data "may be incorrect", but encodings are validated against riscv-opcodes and names and assembly formats against LLVM; its generated specifications are labelled "UNOFFICIAL" (same README). The repository is now `riscv/riscv-unified-db` (measured here: `gh api repos/riscv-software-src/riscv-unified-db`). The local snapshot is v0.1.0, 2025-02-01, the only plain version tag (manifest) | per file through REUSE: BSD-3-Clause-Clear for UDB's own files, CC-BY-4.0 alongside it for prose imported from the ISA manual ([REUSE.toml at af3eaf9](https://github.com/riscv/riscv-unified-db/blob/af3eaf91a20d19d6d4ead9110f1ba3fb56a8035c/REUSE.toml)); v0.1.0 is BSD-3-Clause-Clear ([manifest](../../reference/manifest.toml)) |

### Sail models

- Sail is "a language for describing the instruction-set architecture (ISA) semantics of processors"; from a model it can generate emulators, documentation snippets and theorem-prover definitions ([sail-riscv README](https://github.com/riscv/sail-riscv/blob/e81a4367075cb8ca5c1771aeaf3b9495025c4223/README.md)). The language's latest release is 0.20.3, 2026-09-23 ([releases](https://github.com/rems-project/sail/releases)); it is BSD 2-clause apart from listed exceptions, including the ASL-derived aarch64 code ([LICENSE](https://github.com/rems-project/sail/blob/996bda46dbda6fdf1290fa2ffb01a3eac7536746/LICENSE)).
- **RISC-V:** `sail-riscv` is "a formal specification of the RISC-V architecture", "adopted by RISC-V International", and builds into an emulator, `sail_riscv_sim` (README above). Release 0.16 (2026-10-16) ships `sail-riscv-Mac-arm64.tar.gz` among its assets, and dated prereleases follow it, such as 2026-11-16 and 2026-11-23 (measured here: `gh api repos/riscv/sail-riscv/releases`). Licence: BSD two-clause, except third-party dependencies ([LICENCE](https://github.com/riscv/sail-riscv/blob/e81a4367075cb8ca5c1771aeaf3b9495025c4223/LICENCE)).
- **Arm:** `sail-arm` has Sail models of Armv8.5-A, Armv9.3-A and Armv9.4-A, under the BSD 3-Clause Clear licence in each model's directory ([README at 329d500](https://github.com/rems-project/sail-arm/blob/329d5009fe33afcf6fa2cda0e52c1c1ed3d8a1d8/README.md)); the copyright holders include Arm Limited ([arm-v9.4-a/LICENSE](https://github.com/rems-project/sail-arm/blob/329d5009fe33afcf6fa2cda0e52c1c1ed3d8a1d8/arm-v9.4-a/LICENSE)).
- **x86:** `sail-x86-from-acl2` is a Sail x86 model "automatically translated from the ACL2 model", last changed 2024-11-29 (measured here: `gh api repos/rems-project/sail-x86-from-acl2`); its translated model files are BSD 3-clause plus the original ACL2 model's licence ([LICENCE at 0c6c023](https://github.com/rems-project/sail-x86-from-acl2/blob/0c6c023dccc7c20d8d9dbfc481f01d9bb49d62bf/LICENCE)).

### Second emulators

None is installed here; the spec leaves them to M1.1 (spec §2 and §10).

- **Bochs (x86):** see the VT-x table above: 3.1, LGPL-2.0-or-later, VMX emulation at build time, in Homebrew.
- **Spike (RISC-V):** "a functional model of one or more RISC-V harts", supporting among others Privileged Architecture v1.13 and the Hypervisor extension v1.0 ([README at 6e18887](https://github.com/riscv-software-src/riscv-isa-sim/blob/6e18887745c63f33048e4be3d880c65a72aad0df/README.md)); latest tag v1.1.0 (measured here: `gh api repos/riscv-software-src/riscv-isa-sim/tags`); BSD 3-clause, copyright the Regents of the University of California ([LICENSE](https://github.com/riscv-software-src/riscv-isa-sim/blob/6e18887745c63f33048e4be3d880c65a72aad0df/LICENSE)). It is not in homebrew-core (measured here: `brew info riscv-isa-sim` finds no formula); the `riscv-software-src/homebrew-riscv` tap builds it from git `main`, with bottles only for macOS 15 ([riscv-isa-sim.rb](https://github.com/riscv-software-src/homebrew-riscv/blob/98103f24b4c9009851a5f9a807af3ca488e7e64a/riscv-isa-sim.rb)).
- **Arm FVPs (aarch64):** Arm's install guide says "Linux (AArch64 and x86) and Windows (x86 only) hosts are supported" and "FVPs are license managed" ([learn.arm.com, FVP, updated 22 Jun 2026](https://learn.arm.com/install-guides/fm_fvp/fvp)); its sibling page says "Arm Ecosystem FVPs and AEMs are not license managed", that Arm provides them "free of charge", and that "Host platform support will vary by FVP" ([learn.arm.com, Ecosystem FVPs, same date](https://learn.arm.com/install-guides/fm_fvp/eco_fvp)). The two pages disagree on licensing; Arm's own download pages could not be read (above), so which applies to the A-profile Base AEM FVP is unverified, as is its EULA.
- **FVPs on macOS:** no macOS host is listed (pages above). `Arm-Examples/FVPs-on-Mac`, the repository Arm's macOS guide has you clone ([learn.arm.com, FVPs on macOS](https://learn.arm.com/install-guides/fvps-on-macos)), runs the Linux Arm Virtual Hardware FVPs in Docker Desktop or, on Apple silicon with macOS 26 or later, Apple Container; it builds Fast Models 11.32.23 and needs "an Arm user-based license" ([README at 69a59ce](https://github.com/Arm-Examples/FVPs-on-Mac/blob/69a59ceaa218ce05d9d81de34c023f9f4bc7d674/README.md)). Its examples are Cortex-M and Corstone models (same README); whether the A-profile Base AEM FVP runs this way is unverified. Arm's macOS guide adds that Docker Desktop may need a paid subscription for commercial use ([learn.arm.com, FVPs on macOS, updated 8 Oct 2026](https://learn.arm.com/install-guides/fvps-on-macos)).
