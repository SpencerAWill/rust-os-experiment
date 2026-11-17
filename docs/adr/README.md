# Architecture decision records

Status: Living (2026-11-17). One short file per decision that would be expensive to reverse: its context, the decision and its consequences.

## When this applies

- A change would touch one of the decisions below, or make a new decision that is hard to reverse: it needs an ADR (or one that supersedes an old one) in the same pull request.
- A research chunk decides something: its ADR is part of its spec and is accepted with it.

Accepted ADRs are not rewritten to change the decision; a new ADR supersedes the old one ([conventions](../README.md#conventions)).

## Contents

| ADR | Decision | Status |
|---|---|---|
| [`0001-licence-and-originality.md`](0001-licence-and-originality.md) | `MIT OR Apache-2.0`; no copied code; AI conversations are sources of ideas, not code; allowed dependency licences | Accepted |
| [`0002-unsafe-policy-undecided.md`](0002-unsafe-policy-undecided.md) | The `unsafe` policy is set by an M1 chunk before any kernel code | Accepted (placeholder) |
