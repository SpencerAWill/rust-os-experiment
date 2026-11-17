## Chunk

<!-- Roadmap work: the chunk ID, its kind (build or research) and links (see docs/roadmap.md). Otherwise "None", for example for a dependency update or a small fix. The title ends with the chunk ID, for example "(M1.1)". -->

- Chunk:
- Kind:
- Spec:
- Plan:

## What and why

<!-- What does this change, and why? Link the issue if there is one ("Closes #N"). -->

## Reviews

<!-- One line each: the result, and the agent file that did it (.claude/agents/), or `n/a: <reason>`. See CONTRIBUTING.md, "Reviews before a pull request". Keep the list for the chunk's kind and delete the other. -->

Build chunk:

- Code (`review-code`):
- Security (`review-security`):
- Documentation (`review-docs`):
- Spec conformance (`review-spec`):
- Interfaces and ABI (`review-conditional`):
- Architecture portability (`review-conditional`):
- Concurrency and memory ordering (`review-conditional`):
- Performance (`review-conditional`):

Research chunk:

- Sources (`review-sources`):
- Documentation (`review-docs`):
- Spec conformance (`review-spec`):
- Code and security, spike code only (`review-code`, `review-security`):

## Lessons

<!-- Lessons added, edited or deleted in docs/lessons/, by ID, one line each. "None" if none. -->

## Follow-ups

<!-- Issues opened for work found during this chunk, by number. "None" if none. -->

## Checklist

- [ ] `cargo xtask ci` passes locally.
- [ ] New behaviour has tests; for a bug fix, the test failed first for the expected reason.
- [ ] I called out any change to an **interface another component depends on, or a policy**, and added or updated an ADR where the decision is hard to reverse.
- [ ] No code was copied from another project or from an AI research conversation (ADR 0001). If an AI assistant was used, I say so here.
- [ ] Every factual claim in a research note has a source, or is marked unverified.
- [ ] User-visible changes are in `CHANGELOG.md`.
- [ ] For a chunk: the spec is `Implemented`, the plan and log are `Historical` with the plan's Deviations filled in, and the roadmap row is `Done`.
