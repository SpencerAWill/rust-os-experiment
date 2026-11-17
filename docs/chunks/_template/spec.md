# M<N>.<n>: <title>

- **Status:** Template (2026-11-17). Copy into the chunk folder; do not edit in place for a chunk. Spec statuses: `Draft (date)`, `Accepted (date)`, `Implemented (PR #n, date; set in the chunk's own pull request, in the close-out commit pushed as soon as the PR is open)`, `Superseded by <link> (date)`, `Dropped (reason, date)`.
- **Kind:** build | research · **Milestone:** M<N> (<milestone goal>) · **Depends on:** M<N>.<k>, or none · **Decides:** ADR NNNN, or "no ADR"
- **Issue:** #N (tracking issue #M), or none
- **Estimated size:** about <n> changed lines, excluding tests, lockfiles and generated files

Keep it as short as the change allows: write what a builder needs and cannot find in the code, and link instead of copying. For a small chunk, sections 3, 7 and 11 are optional.

For a **research** chunk ([workflow](../../workflow.md#research-chunks)): §1 is the question and the decision it unblocks, §4 the sub-questions, sources to read first and the evidence that would settle each, §5 requirements on the note and its evidence (and the ADR, if it decides), and §6 is answered for the decision being studied.

Requirement keywords (MUST, MUST NOT, SHOULD, MAY) follow [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) as clarified by [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174): they are normative only in uppercase.

## 1. Summary

<!-- Three sentences: what this chunk does, why now, and what can be run or observed at the end. -->

## 2. Goals and non-goals

<!-- Goals: what must be true afterwards. Non-goals: what a reader might expect but this chunk deliberately leaves out. -->

## 3. Guide-level explanation (optional for a small chunk)

<!-- How a user or contributor uses the result, as a short walkthrough. Code samples here become doctests. -->

## 4. Reference-level design

<!-- Normative. Types, signatures, semantics, invariants, errors and panics, with RFC 2119 keywords. Subsections as needed (4.1, 4.2, ...). -->

## 5. Requirements

<!-- Numbered M<N>.<n>-R<k>. Each one testable and using an RFC 2119 keyword. -->

- **M<N>.<n>-R1** ...

## 6. Cross-cutting concerns

<!-- Answer every line. Write "n/a, because ..." where it does not apply. -->

- **`unsafe`:** <!-- each use and the invariant it relies on -->
- **Privilege and trust boundaries:** <!-- which component trusts which input -->
- **Architecture portability:** <!-- x86_64, aarch64, riscv64: what is shared, what is per-architecture -->
- **Concurrency and memory ordering:** <!-- cores, interrupts, atomics, DMA coherence -->
- **Interfaces and ABI:** <!-- layouts or calls another component depends on; frozen or not -->
- **Hardware and specification assumptions:** <!-- manual, spec version or QEMU machine a claim relies on -->
- **Testing level:** <!-- host unit tests, QEMU, real hardware -->
- **Performance:** <!-- what is measured, against which baseline -->
- **Documentation:**
- **Policy changes:**

## 7. Alternatives considered (optional for a small chunk)

<!-- Each alternative with the reason it was rejected. -->

## 8. Verification

<!-- A table: which test, command or review step proves each requirement. -->

| Requirement | Verified by |
|---|---|

## 9. Done when

<!-- An objective checklist. -->

- [ ] 

## 10. Open questions

<!-- Must be empty, or each item deferred to a named chunk, before the spec is Accepted. -->

## 11. Prior art and sources (optional for a small chunk)

<!-- Designs, papers and posts studied, with links. Never code copied from other projects (ADR 0001). -->
