# ADR 0001: Licence and originality

Status: Accepted (2026-11-17), in [M0.0](../chunks/m0.0-foundations/spec.md).

## Context

The repository is public from its first commit and is developed with AI assistants. The architecture was first explored in a conversation with an AI model, which produced sample code. Provenance has to be clear from the start: whose code this is, and under what terms it can be reused.

## Decision

- The project is dual-licensed `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`), with the standard contribution clause in the README.
- **No code is copied from other projects**, other kernels and OS crates included (for example Redox, Theseus, seL4, Hermit, Linux, Limine). Their designs, papers, documentation and talks may be studied and cited; reading their source to understand behaviour is fine; transcribing it is not.
- **AI conversations are sources of ideas, never of code.** Code an AI model produced in a research conversation is not pasted in; its claims are checked before a decision relies on them.
- Hardware and firmware specifications (ISA manuals, UEFI, ACPI, PCIe, VirtIO, boot protocols) are implemented from the documents; where an implementation follows a published design closely, its documentation cites it.
- Dependencies are allowed when their licence is one `deny.toml` accepts (`MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `Unicode-3.0`); unknown registries and git sources are denied, and security advisories deny. `cargo xtask deny` enforces it. Using another crate as a dependency is not copying; whether kernel code may depend on external crates is decided with the `unsafe` policy (ADR 0002).

## Consequences

- Provenance is clean for anyone reusing the code.
- Copyleft dependencies are rejected by CI. A bootloader or firmware used as a separate binary (not linked in) is a tool choice, decided in M1 with its licence named.
