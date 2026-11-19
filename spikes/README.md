# spikes/

Status: Living (2026-11-17). Throwaway code that answers a research question by running it.

## When this applies

- A research chunk's plan has a spike step: code that settles a question faster than reading would (for example, whether QEMU with HVF exposes EL2 to a guest on this Mac).
- You want to re-run the experiment behind a research note.

## Contents

- `README.md`: this file. Each spike lives in its own `<topic>/` directory with a `README.md` (the question, how to run it, what it showed, and the research note or chunk it serves; `cargo xtask check-docs` requires it).

## Rules

- Spike code is never built on. Kernel code starts from an accepted ADR, not from a spike.
- A spike may use `unsafe` and nightly features if the question needs them ([ADR 0002](../docs/adr/0002-unsafe-policy-undecided.md)); it is not a workspace member unless its chunk's spec says so, so it does not run in `cargo xtask ci`.
- Its result is logged with the command and its output in the chunk's `log.md`, and cited from the research note.
- The chunk's spec says whether the spike is kept after the chunk (to re-run it) or deleted.
- No code copied from other projects ([ADR 0001](../docs/adr/0001-licence-and-originality.md)).
