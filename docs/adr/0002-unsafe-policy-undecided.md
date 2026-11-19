# ADR 0002: The `unsafe` policy is undecided

Status: Accepted (2026-11-17), in [M0.0](../chunks/m0.0-foundations/spec.md), as a placeholder. To be superseded by the M1 chunk that sets the policy (roadmap M1.3).

## Context

A kernel cannot be written without `unsafe`: inline assembly, page tables, memory-mapped I/O and context switches need it. How much of it to allow, where, and how it is checked (`SAFETY` comments, confinement to named modules, Miri or Kani on code that can run on the host, tests under QEMU) is a decision that shapes every later crate. The policy of an earlier project does not carry over.

## Decision

- No policy is set yet. Until an M1 chunk supersedes this ADR, the only crate, `xtask`, has `#![forbid(unsafe_code)]` as a property of that tool, and no kernel code is written.
- Spike code under `spikes/` may use `unsafe` to answer a research question; it is never built on.

## Consequences

- The first kernel code waits for the policy, so the policy is designed from the research rather than retrofitted.
