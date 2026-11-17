# Stage 5: review

0. **Research chunks** (spec `Kind: research`): skip the simplify pass unless a spike was written (then run it over the spike only). Reviewers are `review-spec`, `review-docs` and `review-sources`, always; `review-code` and `review-security` only for spike code; `review-conditional` only if the note recommends an interface or concurrency design (name which perspective). Dispatch them as in step 2, then continue from step 3; fixes to the note go to a `researcher`, fixes to spike code to a `builder`.
1. **Simplify.** Dispatch one `builder` to run `/simplify` over `main...HEAD` (behaviour-preserving cleanups only). Check and commit its changes on their own: `refactor: simplify (<id>)`. No commit if nothing changed.
2. **Review in parallel.** Dispatch fresh reviewers, each with the spec path, the diff range `main...HEAD` and the plan's lesson IDs, and nothing of your reasoning:
   - `review-code`, `review-security`, `review-docs`, `review-spec`, always;
   - `review-conditional`, naming which of interfaces and ABI, architecture portability, concurrency and memory ordering, and performance apply (see the table in CONTRIBUTING.md, "Reviews before a pull request"); the others are `n/a: <reason>`.
   Performance claims need what was measured, on what machine and in what conditions, against which baseline.
3. **Triage** each finding: fix, or record why not (for the PR). Log the triage.
4. **Fix.** Send the fixes to a `builder`; check and commit as in stage 4 (`fix: <what> (<id>)`).
5. **Re-review** each fixed finding with a **fresh** reviewer of the same kind, never a continued one. After two rounds on the same finding without agreement, stop and ask the maintainer.
6. Write each review's one-line result and agent file into `log.md` for the PR's Reviews section. Go to [stage 6](6-compound.md).
