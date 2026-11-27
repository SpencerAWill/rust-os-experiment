# M0.1: Development environment

- **Status:** Accepted (2026-11-27).
- **Kind:** build · **Milestone:** M0 (repository foundations) · **Depends on:** M0.0 · **Decides:** no ADR · **Followed by:** M1.1
- **Issue:** none
- **Estimated size:** about 900 changed lines, excluding tests, lockfiles and generated files (xtask about 550, the research note about 200, documents about 150)

## 1. Summary

M0.1 makes the host environment for OS work explicit and checkable on the maintainer's M4 Pro. It adds the bare-metal targets and tool components to the pinned toolchain, a committed `Brewfile` of host tools, a `cargo xtask doctor` command that checks them and smoke-boots each architecture's firmware under QEMU, a local reference library of specifications that agents search before relying on memory, and a research note recording the tool survey of 2026-11-27. It comes before M1 because M1.1's spike needs QEMU, and because the survey's findings would otherwise exist only in one conversation. At the end, `cargo xtask doctor` passes on this Mac, and `cargo xtask refs --fetch` fills `reference/`.

## 2. Goals and non-goals

**Goals**

1. A fresh clone on macOS arm64 reaches a working environment with `brew bundle`, `rustup show`, `cargo xtask doctor` and `cargo xtask refs --fetch`, and `doctor` names anything missing with its install command.
2. Each architecture's firmware is shown to boot under the installed QEMU, with the serial output kept on disk for an agent to read.
3. Specifications are available locally as searchable text, pinned by version and checksum, so a claim about a register or a protocol can cite a document and section that was actually read.
4. The tool survey is a cited research note that M1.1 and M1.2 start from.

**Non-goals**

- No kernel, boot image, linker script or `cargo xtask build/run` (M2). No QEMU in CI (M2).
- No decision on the bootloader, `build-std` or nightly (M1.2), or on test strategy and verification tools (M1.3). The note records options; it decides nothing.
- No second emulators (Bochs, Spike, Arm FVP), Docker recipes or MCP servers. The note lists them as leads.
- No rule enforcing register citations in code; there is no kernel code yet (deferred to M2).
- `doctor` is not part of `cargo xtask ci`, and nothing in CI depends on Homebrew.

## 3. Guide-level explanation

```sh
brew bundle                 # host tools from Brewfile
rustup show                 # toolchain, components and targets from rust-toolchain.toml
cargo xtask doctor          # checks tools, firmware, accelerators; smoke-boots each architecture
cargo xtask refs --fetch    # downloads the reference library into reference/ (gitignored)
cargo xtask refs            # offline: which documents are present, missing or changed
```

`doctor` prints one line per item (`<status> <check> <item>: <detail>`, status `ok`, `warn`, `skip` or `FAIL`), and exits non-zero if any line is `FAIL`. The serial log of each smoke boot is written to `target/doctor/<name>.log`. To check a register's layout, search `reference/text/` (for example `rg -n 'HCR_EL2' reference/text/arm-arm.txt`) and cite the document id, version and section.

## 4. Reference-level design

### 4.1 Toolchain

`rust-toolchain.toml` keeps `channel = "1.100.0"` and MUST list `components = ["rustfmt", "clippy", "rust-analyzer", "llvm-tools", "rust-src"]` and `targets = ["x86_64-unknown-none", "aarch64-unknown-none", "riscv64gc-unknown-none-elf"]`. CI installs the same set through `rustup show`; the change in CI job time is recorded in `log.md` (one before-and-after pair of the `test` job). This is a record, not a budget *(main session's call: one toolchain for local and CI is worth a slower job; if the increase is large, the pull request says so and a follow-up issue can propose a CI profile)*.

### 4.2 `Brewfile`

The repository root has a `Brewfile` listing exactly these formulae: `qemu`, `dtc`, `socat`, `poppler`, `acpica`, `x86_64-elf-gdb`, `aarch64-elf-gdb` and `riscv64-elf-gdb`, each with a one-line comment saying what it is for. Rust tools stay with `cargo install` (CONTRIBUTING.md). *My call:* no `limine`, `xorriso` or `mtools` until M1.2 chooses a boot path.

### 4.3 `cargo xtask doctor`

`doctor` takes no flags and rejects any, like every other command. It runs every check, even after a failure, in this order. Output is one line per item, `<status> <check> <item>: <detail>` (for example `FAIL smoke-boot x86_64-uefi: no "UEFI Interactive Shell" within 60 s, log target/doctor/x86_64-uefi.log`); a check with one item (the QEMU version, the accelerator) prints one line. The exit code is non-zero exactly when some line is `FAIL` *(main session's call, from the cold read)*. A check whose input an earlier check found missing (no QEMU, no firmware file) reports its items `FAIL` naming that earlier check, without running anything.

| # | Check | `ok` when | Otherwise |
|---|---|---|---|
| 1 | Toolchain | every component and target in `rust-toolchain.toml` appears in `rustup component list --installed` / `rustup target list --installed` for the active toolchain | `FAIL`, naming each missing item and `rustup show` |
| 2 | Host tools | each tool below runs with its version argument and prints a version | `FAIL` with the Homebrew formula to install |
| 3 | QEMU version | `qemu-system-x86_64 --version` is at least `11.1.0` | `FAIL` below it; the version is printed in every case |
| 4 | Firmware | the four files below exist in a directory QEMU searches | `FAIL`, naming the file |
| 5 | Accelerator | on macOS arm64, `qemu-system-aarch64 -accel help` lists `hvf` | `FAIL`; on any other host, `skip` |
| 6 | Smoke boots | each boot below prints its marker on serial before its deadline | `FAIL` with the log path |
| 7 | Reference library | every manifest entry is present with a matching checksum (§4.4) | `warn`, with the count missing or changed and `cargo xtask refs --fetch`; never `FAIL` |

Host tools and their version arguments: `qemu-system-x86_64`, `qemu-system-aarch64` and `qemu-system-riscv64` with `--version`; `dtc --version`; `socat -V`; `pdftotext -v`; `iasl -v`; and `x86_64-elf-gdb`, `aarch64-elf-gdb` and `riscv64-elf-gdb` with `--version`. A tool passes when the command starts, exits 0 and its combined stdout and stderr contain a version: the leftmost match of `[0-9]+(\.[0-9]+)+` or of an eight-digit date `[0-9]{8}` (`iasl -v` prints `version 20260408`, with no dot; *main session's call, from the cold read*). The real outputs, recorded in `log.md`, are the unit tests' fixtures. Only QEMU has a minimum (*my call*: behaviour this project relies on, such as HVF and firmware, changes between QEMU versions, while the other tools are only used for their basic functions).

Firmware files: `edk2-x86_64-code.fd`, `edk2-aarch64-code.fd`, `edk2-riscv-code.fd` and `opensbi-riscv64-generic-fw_dynamic.bin`. The search directories are taken from `qemu-system-x86_64 -L help`, which prints them one per line; a listed directory that does not exist is skipped (Homebrew's QEMU lists `share/qemu-firmware`, which is absent).

Smoke boots. Every boot runs with `-nographic -monitor none -no-reboot -m 256M` and the firmware path from check 4. The markers below are the ones observed in `log.md` on 2026-11-27:

| Name | Command (beyond the common flags) | Marker | Deadline |
|---|---|---|---|
| `x86_64-bios` | `qemu-system-x86_64` | `SeaBIOS` | 20 s |
| `x86_64-uefi` | `qemu-system-x86_64 -M q35 -drive if=pflash,format=raw,readonly=on,file=<edk2-x86_64-code.fd>` | `UEFI Interactive Shell` | 60 s |
| `aarch64-uefi` | `qemu-system-aarch64 -M virt -cpu host -accel hvf -bios <edk2-aarch64-code.fd>` on macOS arm64; `-cpu max -accel tcg` elsewhere | `UEFI firmware` | 30 s |
| `riscv64-opensbi` | `qemu-system-riscv64 -M virt -bios default` | `OpenSBI v` | 20 s |
| `riscv64-uefi` | `qemu-system-riscv64 -M virt -drive if=pflash,unit=0,format=raw,readonly=on,file=<edk2-riscv-code.fd>` | `UEFI v` | 60 s |

- The QEMU child MUST be killed and waited for on every path: marker found, deadline reached, read error, and a panic in `doctor` (a guard that kills on drop). No QEMU process outlives `doctor`.
- Serial output (stdout and stderr) is written to `target/doctor/<name>.log` as it is read, so a boot that fails leaves a readable log. The directory is resolved from the repository root (lesson xtask-1).
- Reading MUST NOT block past the deadline (for example, a reader thread and a channel with a receive timeout).
- The boots run one after another; *my call*, parallel boots would save seconds and make the logs harder to follow.

### 4.4 Reference library

- `reference/` at the repository root holds `README.md` (added to `REQUIRED_READMES`) and `manifest.toml`. Downloads go to `reference/files/<id>.<ext>`, and text to `reference/text/<id>.txt`. Both directories are gitignored, and no downloaded document is ever committed: the publishers' licences do not allow redistribution, and the README says so.
- `manifest.toml` is a list of `[[doc]]` tables with fields `id` (kebab-case, unique), `title`, `version` (the publisher's version or date), `url`, `sha256` (64 hex digits), `format`, `licence` (free text, as stated by the publisher), and optional `manual = true` with `how` (free text). `format` is one of `pdf`, `html`, `md`, `tar.gz` or `zip`, and is also the file's extension: the file is `files/<id>.<format>` *(main session's call, from the cold read: one field gives both the handling and the extension)*. A document published only as many HTML pages is entered as its PDF or archive edition if the publisher has one, otherwise as `manual`.
- `sha256` is required, except on a `manual` entry, where it is optional *(main session's call: pinning a click-through document needs the maintainer to download it, which is not a step of this chunk)*. `xtask` MUST reject a manifest with an unknown field, a missing required field, a duplicate `id`, an `id` that is not kebab-case, a non-`https` URL, an unknown `format`, a `sha256` that is not 64 hex digits, `how` without `manual = true`, or `manual = true` without `how`.
- `cargo xtask refs` (offline) prints one status per entry and exits 0:
  - `ok`: the file is present, its checksum matches, and for `pdf`, `html` and `md` its text is present;
  - `missing`: no file;
  - `changed`: the checksum does not match;
  - `no-text`: the file is `ok` but `text/<id>.*` is absent (an interrupted or failed conversion);
  - `unpinned`: a `manual` entry without `sha256` whose file is present; the line prints the file's digest, to be copied into the manifest.

  Doctor check 7 uses the same function and warns on any status other than `ok` and `unpinned`.
- `cargo xtask refs --fetch` downloads every entry that is `missing` or `changed` and not `manual`, and regenerates the text of every `no-text` entry:
  - It runs `curl --fail --location --proto =https --proto-redir =https --max-filesize 1073741824 --output <tmp>` with arguments passed as a vector, never through a shell.
  - It verifies the SHA-256 of the temporary file, then renames it into `files/`. On a mismatch it deletes the temporary file and fails, printing the expected and actual digests: the publisher changed the document, and updating the manifest entry's `version` and `sha256` is a deliberate commit.
  - For `pdf`, it then runs `pdftotext -layout files/<id>.pdf text/<id>.txt`, writing to a temporary name and renaming on success. `html` and `md` are copied to `text/<id>.<format>` unchanged; `tar.gz` and `zip` are not unpacked and have no text (*my call*: unpacking waits until a chunk uses the data).
  - For a `manual` entry that is `missing`, it prints `how` and the expected path; a present one is handled like any other entry (checksum if pinned, text if needed).
  - It exits non-zero if any download, checksum or text conversion failed, after attempting every entry.
- *My call:* `toml` and `sha2` are added as `xtask` dependencies (both `MIT OR Apache-2.0`); `curl` is assumed present (it ships with macOS and the CI images), and `require_tool` reports it if not.

The initial manifest covers, at the latest published version on the day the entry is written (the maintainer's choice of the full set):

- **x86:** Intel SDM (combined volumes); AMD64 Architecture Programmer's Manual (all volumes).
- **Arm:** Arm Architecture Reference Manual for A-profile (DDI 0487); GICv3 and GICv4 architecture specification; PSCI; SMCCC; AAPCS64.
- **RISC-V:** the unprivileged ISA manual; the privileged architecture manual; the SBI specification; the AIA specification; the RISC-V ELF psABI.
- **Firmware and boot:** UEFI specification; ACPI specification; Devicetree specification; the Limine boot protocol.
- **Devices and formats:** VirtIO specification; System V ABI (generic ELF) and the x86-64 psABI.
- **Machine-readable:** Arm AARCHMRS open-source package; `riscv-opcodes` (a tagged archive); a RISC-V UDB snapshot (a tagged archive).

An entry that cannot be fetched by a stable direct URL (a click-through licence, a tokenized link) is `manual` with its landing page in `url`. Where the publisher offers only a "latest" URL, the entry records the version it had when the checksum was taken.

### 4.5 The tool survey note

`docs/research/dev-environment.md`, with a provenance line, written by `researcher` agents. It covers, each claim cited or written as `measured here: <command>` (output in `log.md`), or marked unverified:

1. The host tools and what each is for, with versions measured here.
2. QEMU on this Mac: the smoke boots and accelerators (§4.3), the x86 UEFI pflash requirement, the state of HVF nested virtualisation (EL2) and its reported limits, and the x86 VT-x gap, with the options for evidence (TCG, Bochs, CI runners with KVM). The note does not decide M1.1.
3. Ground-truth sources: the reference library, AARCHMRS, UDB and riscv-opcodes, the Sail models, and second emulators (Bochs, Spike, Arm FVP via Docker), with their licences where stated.
4. Verification tools and current work: Verus and the kernels verified with it (Atmosphere, Asterinas `vostd`), Miri, Kani, `loom`; and the state of `build-std`.
5. Unusual architectures worth a later note: CHERI (the standardisation state and its QEMU).
6. What was considered and left out, with the reason (MCP documentation servers, Nix, `just`).

`docs/research/toolchain-and-boot.md` gets two Corrections entries: the UTM cask does not provide QEMU's command-line tools, and x86_64 EDK2 must be given as pflash, not `-bios` (both measured here). `docs/research/README.md` lists the new note.

### 4.6 Agent workflow

- `AGENTS.md`: `cargo xtask doctor` joins the Commands block, and one line under Always says that a claim about a specification is checked against `reference/text/` first (falling back to the web, then "unverified"), citing the document id, version and section.
- `.claude/agents/researcher.md` and `.claude/agents/review-sources.md` get the same line about `reference/`.
- `.claude/settings.json` allows exactly `Bash(cargo xtask doctor)` and `Bash(cargo xtask refs)` (lesson tooling-3); `refs --fetch` goes online and keeps its prompt.

### 4.7 Documentation

`CONTRIBUTING.md` "Setting up" gains the `brew bundle`, `doctor` and `refs --fetch` lines. `xtask/README.md` lists `doctor` and `refs [--fetch]`, and drops "Commands to build, run and test the kernels under QEMU arrive with M2" only if it becomes wrong (it stays true). The roadmap row and `CHANGELOG.md` are updated at close-out.

## 5. Requirements

- **M0.1-R1** `rust-toolchain.toml` MUST list the components and targets of §4.1, and CI MUST pass with them.
- **M0.1-R2** A `Brewfile` at the root MUST list exactly the formulae of §4.2.
- **M0.1-R3** `cargo xtask doctor` MUST run the checks of §4.3 in order, run all of them after a failure, print one `<status> <check> <item>: <detail>` line per item, and exit non-zero exactly when some line is `FAIL`.
- **M0.1-R4** The version of each host tool MUST be read with the argument and the pattern §4.3 names, and QEMU below 11.1.0 MUST be a `FAIL`.
- **M0.1-R5** Each smoke boot MUST pass when its marker appears before its deadline and fail otherwise, MUST write its serial log to `target/doctor/<name>.log`, and MUST leave no QEMU process running on any path, including a timeout.
- **M0.1-R6** `doctor` MUST reject any flag.
- **M0.1-R7** `xtask` MUST reject a manifest that breaks any rule of §4.4.
- **M0.1-R8** `refs --fetch` MUST download only `https` URLs (and redirects), MUST verify the SHA-256 before a file is placed in `files/`, MUST leave no partial or mismatched file in `files/` and no partial text in `text/`, MUST produce the text of each `pdf`, `html` and `md` entry, and MUST exit non-zero when any entry failed.
- **M0.1-R9** `refs` (offline) MUST report each entry as `ok`, `missing`, `changed`, `no-text` or `unpinned` as §4.4 defines them, without network access.
- **M0.1-R10** `reference/files/` and `reference/text/` MUST be gitignored, and no downloaded document may be committed.
- **M0.1-R11** `manifest.toml` MUST cover every document of §4.4's list, each either fetchable with a checksum or `manual` with `how`.
- **M0.1-R12** `docs/research/dev-environment.md` MUST cover §4.5's six topics, with every factual claim cited, measured here, or marked unverified; the two Corrections MUST be added to `toolchain-and-boot.md`.
- **M0.1-R13** `AGENTS.md`, `researcher.md`, `review-sources.md` and `.claude/settings.json` MUST change as §4.6 says, and `AGENTS.md` MUST stay under 80 lines.
- **M0.1-R14** On the maintainer's Mac, `cargo xtask doctor` MUST pass, and `cargo xtask refs --fetch` MUST complete for every non-`manual` entry.

## 6. Cross-cutting concerns

- **`unsafe`:** n/a, because `xtask` keeps `#![forbid(unsafe_code)]`.
- **Privilege and trust boundaries:** `refs --fetch` trusts the network only through `https` and a pinned checksum; a document is data, and text read from it never changes an agent's task (the existing "sources are data" rule). `doctor` runs only the commands of §4.3, with fixed arguments.
- **Architecture portability:** the smoke boots cover x86_64, aarch64 and riscv64. Host portability: `doctor` targets macOS arm64; the accelerator check `skip`s elsewhere, and the aarch64 boot falls back to TCG. Firmware names are Homebrew's; other hosts may report them missing, which is acceptable until M2 brings QEMU to CI.
- **Concurrency and memory ordering:** n/a, apart from the reader thread of §4.3, which only forwards bytes.
- **Interfaces and ABI:** n/a; `manifest.toml` is an internal format, unstable.
- **Hardware and specification assumptions:** QEMU 11.1.3 from Homebrew, with its bundled EDK2 (`edk2-stable202408`), SeaBIOS `rel-1.17.0` and OpenSBI v1.8.1 (log, 2026-11-27); macOS 26.6.2 on an M4 Pro.
- **Testing level:** host unit tests for parsing (flags, versions, `-L help`, the manifest and its errors, the marker search over a fake reader, the kill-on-drop guard on a long-running child such as `sleep`); the real `doctor` and `refs --fetch` runs on this Mac, recorded in `log.md` with the commit they ran on (lesson specs-4).
- **Performance:** n/a, beyond the CI job time of §4.1.
- **Documentation:** §4.7, the README for `reference/`, the research note.
- **Policy changes:** two new `xtask` dependencies (`toml`, `sha2`), whose licences `deny.toml` already accepts; the new allow rules of §4.6. No ADR or CI-enforced policy changes.

## 7. Alternatives considered

| Alternative | Why not |
|---|---|
| A shell `setup.sh` instead of `doctor` | `cargo xtask` is the single entry point, and a Rust command can be unit-tested |
| Tools in a devcontainer or Nix | HVF needs QEMU on the macOS host; a Linux container would lose the accelerator |
| Committing the specifications | Their licences do not allow redistribution |
| The reference library outside the repository (`~/Reference`) | Agents work inside the repository; a gitignored folder keeps the paths stable and the permissions simple |
| A documentation MCP server instead of local copies | Remote, unpinned and harder to cite; the note keeps it as a lead |
| Shelling out to `shasum`/`sha256sum` | The names differ between macOS and Linux; `sha2` is small and already licensed |

## 8. Verification

| Requirement | Verified by |
|---|---|
| R1 | the diff of `rust-toolchain.toml`; CI green on the pull request; the CI time pair in `log.md` |
| R2 | the diff of `Brewfile`; `brew bundle check` output in `log.md` |
| R3, R6 | unit tests: check order, continue-after-failure, exit code from the worst status; flag rejection; a `doctor` run in `log.md` |
| R4 | unit tests: version parsing for each tool's real output (the fixtures in `log.md`, including `iasl`'s dotless date), a non-zero exit, the 11.1.0 boundary |
| R5 | unit tests: marker found, deadline reached, child killed on drop; after the `doctor` run, `pgrep -f qemu-system` finds nothing (in `log.md`) |
| R7 | one unit test per rule of §4.4 (lesson specs-3) |
| R8 | unit tests: an `http` URL is rejected; the verify-and-place step, given a temporary file with a matching and with a wrong digest, places the first and leaves nothing in `files/` for the second; `curl`'s argument vector; a `refs --fetch` run in `log.md` |
| R9 | unit test against a temporary tree with one entry in each of the five statuses |
| R10 | `git check-ignore reference/files/x reference/text/x`; `git ls-files reference/` lists only `README.md` and `manifest.toml` |
| R11 | `review-spec`: manifest entries against §4.4's list |
| R12 | `review-sources` on the note and the Corrections |
| R13 | `review-docs`; `cargo xtask check-docs` |
| R14 | the `doctor` and `refs --fetch` runs in `log.md`, with the commit they ran on |

## 9. Done when

- [ ] R1-R14 hold, with the evidence of §8.
- [ ] `cargo xtask ci` passes locally and in CI.
- [ ] The pull request lists the reviews of [CONTRIBUTING.md](../../../CONTRIBUTING.md#reviews-before-a-pull-request): code, security (network fetch, new dependencies), docs, spec, sources (the note), and conditional for architecture portability.

## 10. Open questions

None. Deferred: QEMU in CI and the run command (M2); a register-citation rule (M2); Bochs, Spike or an Arm FVP if M1.1 needs a second emulator (M1.1).

## 11. Prior art and sources

The sources for the survey are cited in the note (§4.5). The smoke-boot commands and their output are in [log.md](log.md).
