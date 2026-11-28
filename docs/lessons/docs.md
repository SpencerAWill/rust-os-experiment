---
applies_to:
  - "AGENTS.md"
  - "CONTRIBUTING.md"
  - "docs/**"
  - ".claude/**"
---
# Lessons: documentation

- Status: Living (2026-11-28)

- **docs-1.** A rule that several entry files need (the waits, the escalations) has one copy; the other files link to it. When such a rule moves or changes, grep its key phrase across `docs/` and `.claude/` before committing. *Why:* the escalation list was written three times with three wordings within one chunk (rust-engine-experiment M3.15); in the engine's M3.20, three rules that other steps read (the chunk-ID rule, the stage that pushes the branch, the form of a link between issues) were each written in two or more files, and a later change updated only some of them. *From:* rust-engine-experiment M3.15, M3.20 · *Confirmed:* 2026-11-27
- **docs-2.** A header field or section a stage tells agents to add belongs in the template they copy (`docs/chunks/_template/`), so its format has one copy; when a fix changes how many items a document lists, grep the plan for the old count. *Why:* chunk stage 1 told agents to add an `**Issue:**` line the template lacked, and plan step 6 still said "the two built-in workflows" after the setup grew to four (rust-engine-experiment M3.18). *From:* rust-engine-experiment M3.18 · *Confirmed:* 2026-11-16
- **docs-3.** A research note's "measured here" claim points at the `log.md` entry that holds that command's output, and the note cites no plan step, deviation or decision of the chunk (it must read on its own after the plan is historical). Before citing `log.md` for a claim in any document, grep the log for it; a later entry saying the evidence was added is not the evidence. A literal name `typos` rejects (a tag, a commit title) is described or allow-listed, never respelt with "(sic)". *Why:* in M0.1 three "measured here" commands had no output in the log, other claims gave only "(measured here)" so a reviewer had to search the log, two sentences cited plan steps, and a respelt tag name could no longer be found; in M0.3 a README line cited "(M0.3 log)" for an observation the log did not hold, and a triage entry claimed it did. *From:* M0.1; M0.3 · *Confirmed:* 2026-11-28
- **docs-4.** Before the full `cargo xtask ci`, run `typos` over the chunk folder, and cite commits in `log.md` by the full hash when a short one reads as a word (or quote a flagged token nowhere, including the note about it). *Why:* in the engine's M3.21 a short hash in `log.md` failed `typos` twice, the second time through the note that explained the first; each failure stopped the full run before its later checks, so it was run three times. *From:* rust-engine-experiment M3.21 · *Confirmed:* 2026-11-27
