# docs/research/

Status: Living (2026-11-17). Research notes: what was surveyed, what it suggests, and what is still open. **Not decisions**: decisions are ADRs in [`../adr/`](../adr/README.md).

## When this applies

- You plan or run a research chunk ([workflow](../workflow.md#research-chunks)): read the notes it builds on, and write its note here.
- A spec or ADR relies on a claim: check the claim's source here first.

## Contents

- [`open-questions.md`](open-questions.md): the ranked questions M1 answers, each mapped to its chunk. **Start here.**
- [`architecture-landscape.md`](architecture-landscape.md): kernel architectures and design ideas surveyed, which ones conflict, exokernel versus hypervisor, and "layers" versus planes.
- [`shim-layer.md`](shim-layer.md): the interposition layer between a hypervisor and the exokernel: candidate features, claimed costs, names, and the proposed placement.
- [`toolchain-and-boot.md`](toolchain-and-boot.md): bootloaders, targets, how far down Rust reaches, testing on the M4 Pro, and the documents an OS needs.
- [`compatibility.md`](compatibility.md): ways to run existing software, from WASI to a Linux domain.
- [`realities-checklist.md`](realities-checklist.md): the problems that stall OS projects, as a checklist for specs.
- `sources/`: raw source material. [`gemini-architecture-conversation.txt`](sources/gemini-architecture-conversation.txt) is the original conversation the seed notes came from (the maintainer's questions start with `>`); it is model output, unverified, and its code is not reused.

## How research becomes a decision

1. A question gets a `research` issue and an M1 (or later) chunk of kind `research`.
2. The chunk writes or extends a note here, every claim cited, and reviewed by `review-sources`.
3. If it decides something, the same pull request adds an ADR that cites the note.

The six seed notes above came from one AI conversation and carry its provenance line. A research chunk that checks a claim replaces it with the cited version, or marks it wrong in the note's Corrections section.
